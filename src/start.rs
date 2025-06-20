use core::{
    cell::OnceCell,
    ffi::{CStr, c_char},
};

use crate::{eprintln, helpers::AssertThreadSafe};

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

#[unsafe(export_name = "main")]
unsafe extern "C" fn sys_main(argc: i32, argv: *mut *mut c_char, envp: *mut *mut c_char) -> i32 {
    if argc > 0 {
        let _ = PRG_NAME.set(unsafe { CStr::from_ptr(argv.read()) });
    }
    Termination::report(crate::main())
}
