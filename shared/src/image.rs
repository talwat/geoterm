use byteorder::ReadBytesExt;
use image::{Pixel, Rgb, RgbImage};
use std::io::{Read, Write};

pub fn encode<W: Write>(image: RgbImage, writer: &mut W) -> std::io::Result<()> {
    let (width, height) = image.dimensions();
    assert!(width % 320 == 0, "width is incorrect!");
    assert!(height % 240 == 0, "height is incorrect!");

    for pixel in image.pixels() {
        let [r, g, b] = pixel.channels() else {
            panic!("channels don't match up!")
        };

        let r = (*r >> 5) & 0x07;
        let g = (*g >> 5) & 0x07;
        let b = (*b >> 6) & 0x03;

        let combined = (r << 5) | (g << 2) | b;
        writer.write_all(&[combined])?;
    }

    Ok(())
}

pub fn decode<R: Read>(reader: &mut R, width: u32, height: u32) -> std::io::Result<RgbImage> {
    assert!(width % 320 == 0, "width is incorrect!");
    assert!(height % 240 == 0, "height is incorrect!");

    let mut img = RgbImage::new(width, height);

    'main: for y in 0..height {
        for x in 0..width {
            let Ok(combined) = reader.read_u8() else {
                break 'main;
            };

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
