//! Backpatching preserves the append cursor and rejects unwritten ranges.
use bincode::enc::write::{SizeWriter, SliceWriter, Writer};
use bincode::{Encode, enc::Encoder, error::EncodeError};
use cu_bincode as bincode;

struct Frame;
impl Encode for Frame {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        let writer = encoder.writer();
        let start = writer.position()?;
        writer.write(&[0; 4])?;
        writer.write(&[1, 2, 3])?;
        let len = writer.position()? - start - 4;
        writer.overwrite(start, &(len as u32).to_le_bytes())?;
        writer.write(&[9])
    }
}

#[test]
fn backpatch_slice_and_size_writer() {
    let mut output = [0; 10];
    let len = bincode::encode_into_slice(Frame, &mut output, bincode::config::standard()).unwrap();
    assert_eq!(len, 8);
    assert_eq!(&output[..len], &[3, 0, 0, 0, 1, 2, 3, 9]);
    let mut size = SizeWriter::default();
    Frame
        .encode(&mut bincode::enc::EncoderImpl::new(
            &mut size,
            bincode::config::standard(),
        ))
        .unwrap();
    assert_eq!(size.bytes_written, len);
    assert!(size.overwrite(usize::MAX, &[1]).is_err());
    assert!(size.overwrite(len, &[1]).is_err());
    assert_eq!(size.bytes_written, len);
}

#[cfg(feature = "alloc")]
#[test]
fn backpatch_vec() {
    assert_eq!(
        bincode::encode_to_vec(Frame, bincode::config::standard()).unwrap(),
        [3, 0, 0, 0, 1, 2, 3, 9]
    );
}

#[test]
fn overwrite_cannot_extend_or_move_cursor() {
    let mut output = [0; 8];
    let mut writer = SliceWriter::new(&mut output);
    writer.write(&[1, 2, 3]).unwrap();
    assert!(writer.overwrite(2, &[9, 9]).is_err());
    assert!(writer.overwrite(usize::MAX, &[9]).is_err());
    assert!(writer.overwrite(4, &[]).is_err());
    writer.overwrite(3, &[]).unwrap();
    writer.overwrite(1, &[8]).unwrap();
    assert_eq!(writer.bytes_written(), 3);
    writer.write(&[4]).unwrap();
    assert_eq!(output, [1, 8, 3, 4, 0, 0, 0, 0]);
}

#[test]
fn forward_only_writer_rejects_before_writing() {
    struct ForwardOnly(usize);
    impl Writer for ForwardOnly {
        fn write(&mut self, bytes: &[u8]) -> Result<(), EncodeError> {
            self.0 += bytes.len();
            Ok(())
        }
    }
    let mut writer = ForwardOnly(0);
    assert!(
        Frame
            .encode(&mut bincode::enc::EncoderImpl::new(
                &mut writer,
                bincode::config::standard()
            ))
            .is_err()
    );
    assert_eq!(writer.0, 0);
}
