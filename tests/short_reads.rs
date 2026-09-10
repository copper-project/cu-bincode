//! Streaming adapters can read a partial final buffer without reading past EOF.
use bincode::de::read::{Reader, SliceReader};
use cu_bincode as bincode;

#[test]
fn slice_short_reads_and_eof() {
    let mut source = SliceReader::new(&[1, 2, 3]);
    let reader = &mut source;
    let mut bytes = [0; 8];
    assert_eq!(reader.read_some(&mut []).unwrap(), 0);
    assert_eq!(reader.read_some(&mut bytes).unwrap(), 3);
    assert_eq!(&bytes[..3], &[1, 2, 3]);
    assert_eq!(reader.read_some(&mut bytes).unwrap(), 0);
}

#[cfg(feature = "std")]
#[test]
fn std_reader_short_reads_and_eof() {
    struct ShortRead;
    impl bincode::Decode<()> for ShortRead {
        fn decode<D: bincode::de::Decoder<Context = ()>>(
            decoder: &mut D,
        ) -> Result<Self, bincode::error::DecodeError> {
            let mut bytes = [0; 8];
            assert_eq!(decoder.reader().read_some(&mut bytes)?, 3);
            assert_eq!(&bytes[..3], &[1, 2, 3]);
            assert_eq!(decoder.reader().read_some(&mut bytes)?, 0);
            Ok(Self)
        }
    }
    bincode::decode_from_std_read::<ShortRead, _, _>(
        &mut &[1, 2, 3][..],
        bincode::config::standard(),
    )
    .unwrap();
    let mut buffered = std::io::BufReader::new(&[1, 2, 3][..]);
    let mut bytes = [0; 8];
    assert_eq!(Reader::read_some(&mut buffered, &mut bytes).unwrap(), 3);
    assert_eq!(Reader::read_some(&mut buffered, &mut bytes).unwrap(), 0);
}
