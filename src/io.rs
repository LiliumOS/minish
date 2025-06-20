use alloc::{string::String, vec::Vec};
use genio::{Read, Write, bufio::BufRead};
use lilium_sys::sys::{
    handle::HandlePtr,
    io::{__HANDLE_IO_STDERR, __HANDLE_IO_STDIN, __HANDLE_IO_STDOUT, IOHandle, IORead, IOWrite},
};

mod error;

pub use error::{Error, ErrorKind};

pub type Result<T> = core::result::Result<T, Error>;

pub struct Stdio(HandlePtr<IOHandle>);

impl core::fmt::Write for Stdio {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        if unsafe { IOWrite(self.0, s.as_ptr().cast(), s.len() as _) } < 0 {
            Err(core::fmt::Error)
        } else {
            Ok(())
        }
    }
}

#[inline(always)]
pub fn stdin() -> Stdio {
    Stdio(__HANDLE_IO_STDIN)
}

#[inline(always)]
pub fn stdout() -> Stdio {
    Stdio(__HANDLE_IO_STDOUT)
}

#[inline(always)]
pub fn stderr() -> Stdio {
    Stdio(__HANDLE_IO_STDERR)
}

#[macro_export]
macro_rules! print{
    ($($tt:tt)*) => {
        {
            use core::fmt::Write as _;
            ::core::write!($crate::io::stdout(), $($tt)*).unwrap()
        }
    };
}

#[macro_export]
macro_rules! eprint{
    ($($tt:tt)*) => {
        {
            use core::fmt::Write as _;
            ::core::write!($crate::io::stderr(), $($tt)*).unwrap()
        }
    };
}

#[macro_export]
macro_rules! println{
    ($($tt:tt)*) => {
        {
            use core::fmt::Write as _;
            ::core::writeln!($crate::io::stdout(), $($tt)*).unwrap()
        }
    };
}

#[macro_export]
macro_rules! eprintln{
    ($($tt:tt)*) => {
        {
            use core::fmt::Write as _;
            ::core::writeln!($crate::io::stderr(), $($tt)*).unwrap()
        }
    };
}

impl Read for Stdio {
    type ReadError = Error;
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let len = buf.len();
        let n = unsafe { IORead(self.0, buf.as_mut_ptr().cast(), len as _) };

        if n < 0 {
            Err(Error::from_raw_os_error(n))
        } else {
            Ok(n as usize)
        }
    }
}

impl Write for Stdio {
    type WriteError = Error;
    type FlushError = Error;

    fn write(&mut self, buf: &[u8]) -> core::result::Result<usize, Self::WriteError> {
        let n = unsafe { IOWrite(self.0, buf.as_ptr().cast(), buf.len() as _) };

        if n < 0 {
            Err(Error::from_raw_os_error(n))
        } else {
            Ok(n as usize)
        }
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }

    fn size_hint(&mut self, _: usize) {
        // Cannot be implemented for `Stdio` efficiently. Only `FileHandle` has any manner of support for this
    }
}

pub struct BufReader<R> {
    underlying: R,
    buf_pos: usize,
    buf_len: usize,
    buf: [u8; 64],
}

impl<R> BufReader<R> {
    pub const fn new(inner: R) -> Self {
        Self {
            underlying: inner,
            buf_pos: 0,
            buf_len: 0,
            buf: [0; 64],
        }
    }

    pub fn into_inner(self) -> R {
        self.underlying
    }
}

impl<R: Read> BufRead for BufReader<R> {
    fn fill_buf(&mut self) -> core::result::Result<&[u8], Self::ReadError> {
        if self.buf_pos >= self.buf_len {
            self.buf_len = self.underlying.read(&mut self.buf)?;
            self.buf_pos = 0;
        }

        Ok(&self.buf[self.buf_pos..self.buf_len])
    }

    fn consume(&mut self, amount: usize) {
        self.buf_pos += amount;
    }
}

impl<R: Read> Read for BufReader<R> {
    type ReadError = R::ReadError;
    fn read(&mut self, buf: &mut [u8]) -> core::result::Result<usize, Self::ReadError> {
        let inner = self.fill_buf()?;
        let len = inner.len().min(buf.len());
        buf[..len].copy_from_slice(&inner[..len]);
        self.consume(len);
        Ok(len)
    }
}

mod ex;

pub use ex::{BufReadEx, ReadToStringError};
