use bytes::{Buf, BufMut, Bytes, BytesMut};
use image::{Pixel, Rgb, RgbImage};

use crate::compression::{compress, decompress};

pub const WIDTH: u32 = 320;
pub const HEIGHT: u32 = 240;
pub const SIZE: u32 = 320 * 240;

pub fn encode(image: RgbImage) -> std::io::Result<Bytes> {
    let mut bytes = BytesMut::with_capacity(SIZE as usize);

    let (width, height) = image.dimensions();
    assert!(width % WIDTH == 0, "width is incorrect!");
    assert!(height % HEIGHT == 0, "height is incorrect!");

    for pixel in image.pixels() {
        let [r, g, b] = pixel.channels() else {
            panic!("channels don't match up!")
        };

        let r = (*r >> 5) & 0x07;
        let g = (*g >> 5) & 0x07;
        let b = (*b >> 6) & 0x03;

        let combined = (r << 5) | (g << 2) | b;
        bytes.put_u8(combined);
    }

    compress(&mut bytes);
    Ok(bytes.freeze())
}

pub fn decode(mut bytes: BytesMut, width: u32, height: u32) -> std::io::Result<RgbImage> {
    decompress(&mut bytes);
    assert!(width % WIDTH == 0, "width is incorrect!");
    assert!(height % HEIGHT == 0, "height is incorrect!");

    let mut img = RgbImage::new(width, height);

    for y in 0..height {
        for x in 0..width {
            let combined = bytes.get_u8();
            let r = (combined >> 5) & 0x07;
            let g = (combined >> 2) & 0x07;
            let b = combined & 0x03;

            let r = (r as u16 * 255 / 7) as u8;
            let g = (g as u16 * 255 / 7) as u8;
            let b = (b as u16 * 255 / 3) as u8;

            img.put_pixel(x, y, Rgb([r, g, b]));
        }
    }

    Ok(img)
}
