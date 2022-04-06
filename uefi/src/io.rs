use {crate::error::Result, alloc::vec::Vec, core::ptr};

struct Guard<'a> {
    buf: &'a mut Vec<u8>,
    len: usize,
}

impl Drop for Guard<'_> {
    fn drop(&mut self) {
        unsafe { self.buf.set_len(self.len) };
    }
}

/// The `Read` trait allows for reading bytes from a source.
pub(crate) trait Read {
    /// Pull some bytes from this source into the specified buffer, returning
    /// how many bytes were read.
    ///
    /// This function does not provide any guarantees about whether it blocks
    /// waiting for data, but if an object needs to block for a read and cannot,
    /// it will typically signal this via an [`Err`] return value.
    ///
    /// If the return value of this method is [`Ok(n)`], then implementations must
    /// guarantee that `0 <= n <= buf.len()`. A nonzero `n` value indicates
    /// that the buffer `buf` has been filled in with `n` bytes of data from this
    /// source. If `n` is `0`, then it can indicate one of two scenarios:
    ///
    /// 1. This reader has reached its "end of file" and will likely no longer
    ///    be able to produce bytes.
    /// 2. The buffer specified was 0 bytes in length.
    ///
    /// As this trait is safe to implement, callers cannot rely on `n <= buf.len()` for safety.
    /// Extra care needs to be taken when `unsafe` functions are used to access the read bytes.
    /// Callers have to ensure that no unchecked out-of-bounds accesses are possible even if
    /// `n > buf.len()`.
    ///
    /// # Errors
    ///
    /// If this function encounters any form of I/O or other error, an error
    /// variant will be returned. If an error is returned then it must be
    /// guaranteed that no bytes were read.
    fn read(&mut self, buf: &mut [u8]) -> Result<usize>;

    /// Determines if this `Read`er can work with buffers of uninitialized
    /// memory.
    ///
    /// The default implementation returns an initializer which will zero
    /// buffers.
    ///
    /// # Safety
    ///
    /// This method is unsafe because a `Read`er could otherwise return a
    /// non-zeroing `Initializer` from another `Read` type without an `unsafe`
    /// block.
    unsafe fn initializer(&self) -> Initializer {
        Initializer::zeroing()
    }

    /// Read all bytes until EOF in this source, placing them into `buf`.
    ///
    /// All bytes read from this source will be appended to the specified buffer
    /// `buf`. This function will continuously call [`read()`] to append more data to
    /// `buf` until [`read()`] returns either [`Ok(0)`] or an error of
    /// non-[`ErrorKind::Interrupted`] kind.
    ///
    /// If successful, this function will return the total number of bytes read.
    ///
    /// # Errors
    ///
    /// If any read error is encountered then this function immediately returns.
    /// Any bytes which have already been read will be appended to `buf`.
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> Result<usize> {
        default_read_to_end(self, buf)
    }
}

/// A type used to conditionally initialize buffers passed to `Read` methods.
#[derive(Debug)]
pub(crate) struct Initializer(bool);

impl Initializer {
    /// Returns a new `Initializer` which will zero out buffers.
    #[must_use]
    #[inline]
    fn zeroing() -> Initializer {
        Initializer(true)
    }

    /// Returns a new `Initializer` which will not zero out buffers.
    ///
    /// # Safety
    ///
    /// This may only be called by `Read`ers which guarantee that they will not
    /// read from buffers passed to `Read` methods, and that the return value of
    /// the method accurately reflects the number of bytes that have been
    /// written to the head of the buffer.
    #[must_use]
    #[inline]
    pub(crate) unsafe fn nop() -> Initializer {
        Initializer(false)
    }

    /// Indicates if a buffer should be initialized.
    #[must_use]
    #[inline]
    fn should_initialize(&self) -> bool {
        self.0
    }

    /// Initializes a buffer if necessary.
    #[inline]
    fn initialize(&self, buf: &mut [u8]) {
        if self.should_initialize() {
            unsafe { ptr::write_bytes(buf.as_mut_ptr(), 0, buf.len()) }
        }
    }
}

/// This uses an adaptive system to extend the vector when it fills. We want to
/// avoid paying to allocate and zero a huge chunk of memory if the reader only
/// has 4 bytes while still making large reads if the reader does have a ton
/// of data to return. Simply tacking on an extra DEFAULT_BUF_SIZE space every
/// time is 4,500 times (!) slower than a default reservation size of 32 if the
/// reader has a very small amount of data to return.
///
/// Because we're extending the buffer with uninitialized data for trusted
/// readers, we need to make sure to truncate that if any of this panics.
pub(crate) fn default_read_to_end<R: Read + ?Sized>(r: &mut R, buf: &mut Vec<u8>) -> Result<usize> {
    let start_len = buf.len();
    let start_cap = buf.capacity();
    let mut g = Guard {
        len: buf.len(),
        buf,
    };
    loop {
        // If we've read all the way up to the capacity, reserve more space.
        if g.len == g.buf.capacity() {
            g.buf.reserve(32);
        }

        // Initialize any excess capacity and adjust the length so we can write
        // to it.
        if g.buf.len() < g.buf.capacity() {
            let capacity = g.buf.capacity();
            unsafe {
                g.buf.set_len(capacity);
                r.initializer().initialize(&mut g.buf[g.len..]);
            }
        }

        let buf = &mut g.buf[g.len..];
        match r.read(buf) {
            Ok(0) => return Ok(g.len - start_len),
            Ok(n) => {
                // We can't allow bogus values from read. If it is too large, the returned vec could have its length
                // set past its capacity, or if it overflows the vec could be shortened which could create an invalid
                // string if this is called via read_to_string.
                assert!(n <= buf.len());
                g.len += n;
            }
            Err(e) => return Err(e),
        }

        if g.len == g.buf.capacity() && g.buf.capacity() == start_cap {
            // The buffer might be an exact fit. Let's read into a probe buffer
            // and see if it returns `Ok(0)`. If so, we've avoided an
            // unnecessary doubling of the capacity. But if not, append the
            // probe buffer to the primary buffer and let its capacity grow.
            let mut probe = [0u8; 32];

            loop {
                match r.read(&mut probe) {
                    Ok(0) => return Ok(g.len - start_len),
                    Ok(n) => {
                        g.buf.extend_from_slice(&probe[..n]);
                        g.len += n;
                        break;
                    }
                    Err(e) => return Err(e),
                }
            }
        }
    }
}
