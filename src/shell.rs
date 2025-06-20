use alloc::{borrow::Cow, string::String, vec::Vec};

use crate::{exit, helpers::SplitOnceOwned, io, println};

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

pub fn exec_line(line: ShellLine) -> io::Result<()> {
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
        Some(n) => Err(io::Error::new(
            io::ErrorKind::NotFound,
            alloc::format!("Command {n} not found"),
        )),
        None => Ok(()),
    }
}
