use alloc::{string::String, vec::Vec};
use genio::bufio::BufRead;
use memchr::memchr;

pub enum ReadToStringError<R> {
    Read(R),
    InvalidUtf8,
}

pub trait BufReadEx: BufRead {
    fn read_until(&mut self, buf: &mut Vec<u8>, b: u8) -> Result<usize, Self::ReadError>;
    fn read_line(&mut self, st: &mut String) -> Result<usize, ReadToStringError<Self::ReadError>>;
}

impl<R: BufRead> BufReadEx for R {
    fn read_until(&mut self, buf: &mut Vec<u8>, b: u8) -> Result<usize, Self::ReadError> {
        let mut total_len = 0;
        loop {
            let inner_buf = self.fill_buf()?;

            if inner_buf.len() == 0 {
                return Ok(total_len);
            }

            match memchr(b, inner_buf) {
                Some(n) => {
                    let n = n + 1;
                    eprintln!("Read {n} bytes");
                    total_len += n + 1;
                    buf.extend_from_slice(&inner_buf[..n]);
                    self.consume(n);
                    break Ok(total_len);
                }
                None => {
                    let n = inner_buf.len();
                    total_len += n;
                    buf.extend_from_slice(inner_buf);
                    self.consume(n);
                }
            }
        }
    }
    fn read_line(&mut self, st: &mut String) -> Result<usize, ReadToStringError<Self::ReadError>> {
        unsafe { append_to_string(st, |buf| self.read_until(buf, b'\n')) }
    }
}

// Taken from std::io impl

struct Guard<'a> {
    buf: &'a mut Vec<u8>,
    len: usize,
}

impl Drop for Guard<'_> {
    fn drop(&mut self) {
        unsafe {
            self.buf.set_len(self.len);
        }
    }
}

//
// Several `read_to_string` and `read_line` methods in the standard library will
// append data into a `String` buffer, but we need to be pretty careful when
// doing this. The implementation will just call `.as_mut_vec()` and then
// delegate to a byte-oriented reading method, but we must ensure that when
// returning we never leave `buf` in a state such that it contains invalid UTF-8
// in its bounds.
//
// To this end, we use an RAII guard (to protect against panics) which updates
// the length of the string when it is dropped. This guard initially truncates
// the string to the prior length and only after we've validated that the
// new contents are valid UTF-8 do we allow it to set a longer length.
//
// The unsafety in this function is twofold:
//
// 1. We're looking at the raw bytes of `buf`, so we take on the burden of UTF-8
//    checks.
// 2. We're passing a raw buffer to the function `f`, and it is expected that
//    the function only *appends* bytes to the buffer. We'll get undefined
//    behavior if existing bytes are overwritten to have non-UTF-8 data.
pub(crate) unsafe fn append_to_string<F, E>(
    buf: &mut String,
    f: F,
) -> Result<usize, ReadToStringError<E>>
where
    F: FnOnce(&mut Vec<u8>) -> Result<usize, E>,
{
    let mut g = Guard {
        len: buf.len(),
        buf: unsafe { buf.as_mut_vec() },
    };
    let ret = f(g.buf).map_err(ReadToStringError::Read);

    // SAFETY: the caller promises to only append data to `buf`
    let appended = unsafe { g.buf.get_unchecked(g.len..) };
    if str::from_utf8(appended).is_err() {
        ret.and_then(|_| Err(ReadToStringError::InvalidUtf8))
    } else {
        g.len = g.buf.len();
        ret
    }
}
