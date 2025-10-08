use image::{Pixel, Rgb, RgbImage, RgbaImage};
use std::io::{Cursor, Read, Write};

pub fn encode(image: RgbaImage, mut buf: Cursor<&mut [u8]>) -> std::io::Result<()> {
    let (width, height) = image.dimensions();
    assert!(width % 320 == 0, "width is incorrect!");
    assert!(height % 240 == 0, "height is incorrect!");

    for pixel in image.pixels() {
        let [r, g, b, _a] = pixel.channels() else {
            panic!("channels don't match up!")
        };

        let r = (*r >> 5) & 0x07;
        let g = (*g >> 5) & 0x07;
        let b = (*b >> 6) & 0x03;

        let combined = (r << 5) | (g << 2) | b;
        buf.write_all(&[combined])?;
    }

    Ok(())
}

pub fn decode(mut buf: Cursor<&[u8]>, width: u32, height: u32) -> std::io::Result<RgbImage> {
    assert!(width % 320 == 0, "width is incorrect!");
    assert!(height % 240 == 0, "height is incorrect!");

    let mut img = RgbImage::new(width, height);

    for y in 0..height {
        for x in 0..width {
            let mut byte = [0u8; 1];
            buf.read_exact(&mut byte)?;
            let combined = byte[0];

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
