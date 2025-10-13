use bytes::BytesMut;

use crate::image;

type Lzss = lzss::Lzss<10, 4, 0x20, { 1 << 10 }, { 2 << 10 }>;

pub fn compress(bytes: &mut BytesMut) {
    let output = Lzss::compress_stack(
        lzss::SliceReader::new(&bytes),
        lzss::VecWriter::with_capacity(image::SIZE as usize),
    )
    .unwrap();

    *bytes = BytesMut::from(output.as_slice());
}

pub fn decompress(bytes: &mut BytesMut) {
    let output = Lzss::decompress_stack(
        lzss::SliceReader::new(&bytes),
        lzss::VecWriter::with_capacity(image::SIZE as usize),
    )
    .unwrap();

    *bytes = BytesMut::from(output.as_slice());
}
