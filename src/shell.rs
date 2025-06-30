use core::{cell::LazyCell, ops::Deref};

use alloc::{borrow::Cow, string::String, vec::Vec};
use bytemuck::Zeroable;
use lilium_sys::sys::{
    fs::{ACCESS_READ, FileHandle, FileOpenOptions, OP_DIRECTORY_ACCESS, OpenFile},
    handle::HandlePtr,
    io::MODE_BLOCKING,
    kstr::{KCSlice, KStrCPtr},
    option::ExtendedOptionHead,
    process::{
        CREATE_PROCESS_OPTION_ARGS, CreateProcess, CreateProcessOption, CreateProcessOptionArgs,
        JoinProcess,
    },
    thread::JoinStatus,
};

use crate::{
    eprintln, exit,
    helpers::SplitOnceOwned,
    io::{self, Error},
    println,
};

pub fn split_shell(x: &str) -> SplitShell {
    SplitShell(x)
}

enum State {
    Normal,
    Escape,
    DQuote,
    EscapeDQuote,
    SQuote,
    EscapeSQuote,
}

pub struct SplitShell<'a>(&'a str);

impl<'a> Iterator for SplitShell<'a> {
    type Item = Cow<'a, str>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut state = State::Normal;

        let s = self.0.trim();
        if s.is_empty() {
            self.0 = s;
            return None;
        }

        let mut buf = String::new();

        for (n, c) in s.char_indices() {
            match state {
                State::Normal => match c {
                    c if c.is_whitespace() => {
                        self.0 = &s[n..];

                        if !buf.is_empty() {
                            return Some(Cow::Owned(buf));
                        } else {
                            return Some(Cow::Borrowed(&s[..n]));
                        }
                    }
                    '\\' => {
                        buf.push_str(&s[..n]);
                        state = State::Escape;
                    }
                    '"' => {
                        buf.push_str(&s[..n]);
                        state = State::DQuote;
                    }
                    '\'' => {
                        buf.push_str(&s[..n]);
                        state = State::SQuote;
                    }
                    ';' => {
                        if n == 0 {
                            self.0 = &s[1..];

                            if !buf.is_empty() {
                                return Some(Cow::Owned(buf));
                            } else {
                                return Some(Cow::Borrowed(&s[..1]));
                            }
                        } else {
                            self.0 = &s[n..];
                            if !buf.is_empty() {
                                return Some(Cow::Owned(buf));
                            } else {
                                return Some(Cow::Borrowed(&s[..n]));
                            }
                        }
                    }
                    _ => continue,
                },
                State::Escape => {
                    buf.push(c);
                    state = State::Normal;
                }
                State::EscapeDQuote => {
                    buf.push(c);
                    state = State::DQuote;
                }
                State::EscapeSQuote => {
                    buf.push(c);
                    state = State::SQuote;
                }
                State::DQuote => match c {
                    '"' => state = State::Normal,
                    '\\' => state = State::EscapeDQuote,
                    _ => buf.push(c),
                },
                State::SQuote => match c {
                    '\'' => state = State::Normal,
                    '\\' => state = State::EscapeSQuote,
                    _ => buf.push(c),
                },
            }
        }
        self.0 = &self.0[self.0.len()..];

        Some(Cow::Borrowed(s))
    }
}

pub struct EnvVar<'a> {
    pub key: Cow<'a, str>,
    pub val: Cow<'a, str>,
}

pub struct ShellLine<'a> {
    pub env: Vec<EnvVar<'a>>,
    pub command: Option<Cow<'a, str>>,
    pub args: Vec<Cow<'a, str>>,
}

impl<'a> core::fmt::Display for ShellLine<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut sep = "";
        for v in &self.env {
            f.write_str(sep)?;
            sep = " ";
            f.write_str(&v.key)?;
            f.write_str("=")?;
            f.write_str(&v.val)?;
        }

        if let Some(cmd) = &self.command {
            f.write_str(sep)?;
            f.write_str(cmd)?;
        }

        for a in &self.args {
            f.write_str(" ")?;
            f.write_str(a)?;
        }
        Ok(())
    }
}

pub fn parse_shell<'a, I: Iterator<Item = Cow<'a, str>>>(mut iter: I) -> ShellLine<'a> {
    let mut line = ShellLine {
        env: Vec::new(),
        command: None,
        args: Vec::new(),
    };
    for c in &mut iter {
        match c.split_once_owned("=") {
            Ok((k, v)) => line.env.push(EnvVar { key: k, val: v }),
            Err(e) => {
                line.command = Some(e);
                break;
            }
        }
    }

    line.args.extend(iter);

    line
}

#[thread_local]
static PATH: LazyCell<Vec<HandlePtr<FileHandle>>> = LazyCell::new(|| {
    eprintln!("Opening PATH");
    let v = crate::start::var("PATH")
        .into_iter()
        .flat_map(|v| v.split(':'))
        .filter_map(|v| {
            eprintln!("Opening {v}");
            let mut hdl = HandlePtr::null();
            lilium_sys::result::Error::from_code(unsafe {
                OpenFile(
                    &mut hdl,
                    HandlePtr::null(),
                    KStrCPtr::from_str(v),
                    &FileOpenOptions {
                        stream_override: KStrCPtr::empty(),
                        access_mode: ACCESS_READ,
                        op_mode: OP_DIRECTORY_ACCESS,
                        create_acl: HandlePtr::null(),
                        blocking_mode: MODE_BLOCKING,
                        extended_options: KCSlice::empty(),
                    },
                )
            })
            .ok()
            .map(|_| hdl)
        })
        .collect();
    eprintln!("Closing PATH");
    v
});

pub fn exec_line(line: &ShellLine) -> io::Result<Option<JoinStatus>> {
    match line.command.as_deref() {
        Some(c @ ("return" | "exit" | "logout")) => {
            println!("exit command: {c}");
            let status = if let Some(status) = line.args.get(0).map(|v| &**v) {
                let val = status
                    .parse()
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
                val
            } else {
                0
            };
            exit(status)
        }
        Some(n) => {
            println!("Running Command: {n}");
            let mut hdl = HandlePtr::null();
            let args = line
                .command
                .iter()
                .chain(line.args.iter())
                .map(Deref::deref)
                .map(KStrCPtr::from_str)
                .collect::<Vec<_>>();

            let opts = [CreateProcessOption {
                args: CreateProcessOptionArgs {
                    header: ExtendedOptionHead {
                        ty: CREATE_PROCESS_OPTION_ARGS,
                        ..ExtendedOptionHead::ZERO
                    },
                    arguments: KCSlice::from_slice(&args),
                },
            }];
            if !n.contains('/') {
                'a: {
                    let mut res = lilium_sys::sys::error::DOES_NOT_EXIST;
                    for path_ent in PATH.iter().copied() {
                        res = unsafe {
                            CreateProcess(
                                &mut hdl,
                                path_ent,
                                &KStrCPtr::from_str(n),
                                &KCSlice::from_slice(&opts),
                            )
                        };
                        if res == 0 {
                            break 'a;
                        }
                    }
                    return Err(io::Error::from_raw_os_error(res));
                }
            } else {
                let res = unsafe {
                    CreateProcess(
                        &mut hdl,
                        HandlePtr::null(),
                        &KStrCPtr::from_str(n),
                        &KCSlice::from_slice(&opts),
                    )
                };
                if res < 0 {
                    return Err(io::Error::from_raw_os_error(res));
                }
            }
            let mut status = bytemuck::zeroed();
            let res = unsafe { JoinProcess(hdl, &mut status) };

            if res < 0 {
                Err(io::Error::from_raw_os_error(res))
            } else {
                Ok(Some(status))
            }
        }
        None => Ok(None),
    }
}
