use core::{
    cell::OnceCell,
    ffi::{CStr, c_char},
};

use crate::{eprintln, helpers::AssertThreadSafe, println};

pub trait Termination {
    fn report(self) -> i32;
}

impl Termination for () {
    fn report(self) -> i32 {
        0
    }
}

impl Termination for ! {
    fn report(self) -> i32 {
        match self {}
    }
}

impl Termination for i32 {
    fn report(self) -> i32 {
        self
    }
}

impl<T: Termination, E: core::fmt::Debug> Termination for Result<T, E> {
    fn report(self) -> i32 {
        match self {
            Ok(val) => val.report(),
            Err(e) => {
                eprintln!(
                    "{}: {e:?}",
                    PRG_NAME
                        .get()
                        .copied()
                        .unwrap_or(c"minish")
                        .to_string_lossy()
                );
                -1
            }
        }
    }
}

// SAFETY:
// We only write this from the main thread
static PRG_NAME: AssertThreadSafe<OnceCell<&CStr>> =
    unsafe { AssertThreadSafe::new_unchecked(OnceCell::new()) };

static ARGS: AssertThreadSafe<OnceCell<(usize, *mut *mut c_char)>> =
    unsafe { AssertThreadSafe::new_unchecked(OnceCell::new()) };

static ENV: AssertThreadSafe<OnceCell<*mut *mut c_char>> =
    unsafe { AssertThreadSafe::new_unchecked(OnceCell::new()) };

#[unsafe(export_name = "main")]
unsafe extern "C" fn sys_main(argc: isize, argv: *mut *mut c_char, envp: *mut *mut c_char) -> i32 {
    println!("{argc}");
    let _ = ARGS.set((argc as usize, argv));
    let _ = ENV.set(envp);
    if argc > 0 {
        let _ = PRG_NAME.set(unsafe { CStr::from_ptr(argv.read()) });
    }
    Termination::report(crate::main())
}

pub struct Vars(*mut *mut c_char);

pub fn vars() -> Vars {
    Vars(ENV.get().copied().unwrap())
}

impl Iterator for Vars {
    type Item = (&'static str, &'static str);

    fn next(&mut self) -> Option<Self::Item> {
        let val = unsafe { self.0.read() };
        if val.is_null() {
            return None;
        }
        self.0 = unsafe { self.0.add(1) };

        let cstr = unsafe { CStr::from_ptr(val) };
        let bytes = cstr.to_bytes();

        let str = unsafe { core::str::from_utf8_unchecked(bytes) };

        let (var, val) = str.split_once('=').unwrap();

        Some((var, val))
    }
}

pub fn var(var: &str) -> Option<&'static str> {
    vars().find_map(|(key, val)| (key == var).then_some(val))
}
