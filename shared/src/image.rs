use std::io::{Cursor, Read, Write};
use image::{Pixel, Rgba, RgbaImage};

pub fn encode(image: RgbaImage, mut buf: Cursor<&mut [u8]>) -> std::io::Result<()> {
    let (width, height) = image.dimensions();
    assert!(width % 320 == 0, "width is incorrect!");
    assert!(height % 240 == 0, "height is incorrect!");

    for pixel in image.pixels() {
        let [r, g, b, _a] = pixel.channels() else {
            panic!("channels don't match up!")
        };
        let [r, g, b] = [*r as u16, *g as u16, *b as u16];

        let combined = (r << 11) | (g << 5) | b;
        buf.write_all(&combined.to_be_bytes())?;
    }

    Ok(())
}

pub fn decode(mut buf: Cursor<&[u8]>, width: u32, height: u32) -> std::io::Result<RgbaImage> {
    assert!(width % 320 == 0, "width is incorrect!");
    assert!(height % 240 == 0, "height is incorrect!");

    let mut img = RgbaImage::new(width, height);

    for y in 0..height {
        for x in 0..width {
            let mut bytes = [0u8; 2];
            buf.read_exact(&mut bytes)?;

            let combined = u16::from_be_bytes(bytes);

            let r = ((combined >> 11) & 0x1F) as u8;
            let g = ((combined >> 5) & 0x3F) as u8;
            let b = (combined & 0x1F) as u8;

            let r = (r as u16 * 255 / 31) as u8;
            let g = (g as u16 * 255 / 63) as u8;
            let b = (b as u16 * 255 / 31) as u8;

            img.put_pixel(x, y, Rgba([r, g, b, 255]));
        }
    }

    Ok(img)
}