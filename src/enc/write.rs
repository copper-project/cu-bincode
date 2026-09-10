//! This module contains writer-based structs and traits.
//!
//! Because `std::io::Write` is only limited to `std` and not `core`, we provide our own [Writer].

use crate::error::EncodeError;

/// Trait that indicates that a struct can be used as a destination to encode data too. This is used by [Encode]
///
/// [Encode]: ../trait.Encode.html
pub trait Writer {
    /// Write `bytes` to the underlying writer. Exactly `bytes.len()` bytes must be written, or else an error should be returned.
    fn write(&mut self, bytes: &[u8]) -> Result<(), EncodeError>;

    /// Return the append position for writers that support overwriting earlier bytes.
    /// Positions are relative to the start of this writer's output.
    /// Forward-only writers return an error without changing their output.
    #[inline]
    fn position(&self) -> Result<usize, EncodeError> {
        Err(EncodeError::Other("Writer does not support backpatching"))
    }

    /// Replace bytes entirely within the already-written output, leaving the append
    /// position unchanged. Reject invalid ranges without changing the output.
    /// Wrappers must forward this operation without counting it as appended bytes.
    #[inline]
    fn overwrite(&mut self, _position: usize, _bytes: &[u8]) -> Result<(), EncodeError> {
        Err(EncodeError::Other("Writer does not support backpatching"))
    }
}

impl<T: Writer> Writer for &mut T {
    #[inline]
    fn write(&mut self, bytes: &[u8]) -> Result<(), EncodeError> {
        (**self).write(bytes)
    }

    #[inline]
    fn position(&self) -> Result<usize, EncodeError> {
        (**self).position()
    }

    #[inline]
    fn overwrite(&mut self, position: usize, bytes: &[u8]) -> Result<(), EncodeError> {
        (**self).overwrite(position, bytes)
    }
}

/// A helper struct that implements `Writer` for a `&[u8]` slice.
///
/// ```
/// # extern crate cu_bincode as bincode;
/// use bincode::enc::write::{Writer, SliceWriter};
///
/// let destination = &mut [0u8; 100];
/// let mut writer = SliceWriter::new(destination);
/// writer.write(&[1, 2, 3, 4, 5]).unwrap();
///
/// assert_eq!(writer.bytes_written(), 5);
/// assert_eq!(destination[0..6], [1, 2, 3, 4, 5, 0]);
/// ```
pub struct SliceWriter<'storage> {
    slice: &'storage mut [u8],
    position: usize,
}

impl<'storage> SliceWriter<'storage> {
    /// Create a new instance of `SliceWriter` with the given byte array.
    pub fn new(bytes: &'storage mut [u8]) -> SliceWriter<'storage> {
        SliceWriter {
            slice: bytes,
            position: 0,
        }
    }

    /// Return the amount of bytes written so far.
    pub fn bytes_written(&self) -> usize {
        self.position
    }
}

impl Writer for SliceWriter<'_> {
    #[inline(always)]
    fn write(&mut self, bytes: &[u8]) -> Result<(), EncodeError> {
        let output = self.slice[self.position..]
            .get_mut(..bytes.len())
            .ok_or(EncodeError::UnexpectedEnd)?;
        output.copy_from_slice(bytes);
        self.position += bytes.len();
        Ok(())
    }

    #[inline]
    fn position(&self) -> Result<usize, EncodeError> {
        Ok(self.position)
    }

    #[inline]
    fn overwrite(&mut self, position: usize, bytes: &[u8]) -> Result<(), EncodeError> {
        let output = self.slice[..self.position]
            .get_mut(position..)
            .and_then(|tail| tail.get_mut(..bytes.len()))
            .ok_or(EncodeError::UnexpectedEnd)?;
        output.copy_from_slice(bytes);
        Ok(())
    }
}

/// A writer that counts how many bytes were written. This is useful for e.g. pre-allocating buffers bfeore writing to them.
#[derive(Default)]
pub struct SizeWriter {
    /// the amount of bytes that were written so far
    pub bytes_written: usize,
}
impl Writer for SizeWriter {
    #[inline(always)]
    fn write(&mut self, bytes: &[u8]) -> Result<(), EncodeError> {
        self.bytes_written = self
            .bytes_written
            .checked_add(bytes.len())
            .ok_or(EncodeError::UnexpectedEnd)?;

        Ok(())
    }

    #[inline]
    fn position(&self) -> Result<usize, EncodeError> {
        Ok(self.bytes_written)
    }

    #[inline]
    fn overwrite(&mut self, position: usize, bytes: &[u8]) -> Result<(), EncodeError> {
        if position > self.bytes_written || bytes.len() > self.bytes_written - position {
            return Err(EncodeError::UnexpectedEnd);
        }
        Ok(())
    }
}
