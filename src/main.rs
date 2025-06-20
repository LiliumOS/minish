#![no_std]
#![no_main]
#![feature(never_type, pattern, allocator_api, alloc_layout_extra)]

extern crate alloc;

unsafe extern "C" {
    pub safe fn exit(x: i32) -> !;
}

mod helpers;
mod io;
mod shell;
mod start;
mod system;

use core::panic::PanicInfo;

use alloc::string::String;
use error_repr::Error;
use io::{BufReadEx, BufReader, ReadToStringError, stderr, stdin};
use lilium_sys::{
    sys::except::{ExceptionStatusInfo, UnmanagedException},
    uuid::parse_uuid,
};
use shell::{parse_shell, split_shell};

use crate::shell::exec_line;

fn main() -> io::Result<i32> {
    let mut line = String::new();
    let mut reader = BufReader::new(stdin());
    loop {
        line.clear();
        print!("# ");
        let n = reader.read_line(&mut line).map_err(|e| match e {
            ReadToStringError::Read(r) => r,
            ReadToStringError::InvalidUtf8 => {
                Error::new_with_message(io::ErrorKind::InvalidData, "Invalid UTF-8 Text")
            }
        })?;
        if n == 0 {
            println!("exit");
            return Ok(0);
        }

        let line = parse_shell(split_shell(&line));

        if let Some(_) = line.command {
            eprintln!("{line}");
            match exec_line(line) {
                Ok(()) => {}
                Err(e) => {
                    println!("{e}")
                }
            }
        }
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    use core::fmt::Write;
    let _ = writeln!(stderr(), "Panicked at {}", info.message());
    unsafe {
        UnmanagedException(&ExceptionStatusInfo {
            except_code: parse_uuid("4c0c6658-59ae-5675-90c3-ffcc0a7219ad"),
            except_info: 0,
            except_reason: 0,
        })
    }
}
