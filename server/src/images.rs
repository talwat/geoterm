use std::io::Cursor;

use crate::{error::Error, images::huggingface::Data};
use bytes::{BufMut, Bytes, BytesMut};
use image::{GenericImageView, ImageReader, imageops};

pub mod huggingface;

pub async fn images() -> Result<([Bytes; 3], Data), Error> {
    let random = rand::random_range(0..11054);
    let data = huggingface::fetch(random).await?;
    let bytes = reqwest::get(data.image.src.clone()).await?.bytes().await?;
    let bytes = bytes.to_vec();
    eprintln!("-> fetched {} bytes of image data", bytes.len());

    let img = ImageReader::new(Cursor::new(&bytes))
        .with_guessed_format()?
        .decode()?;
    let img = img.as_rgb8().unwrap();

    let (width, height) = img.dimensions();
    let slice = width / 3;

    let slices: [Bytes; 3] = std::array::from_fn(|i| {
        let view = img.view(slice * i as u32, 0, slice, height).to_image();
        let resized = imageops::resize(&view, 320, 240, image::imageops::FilterType::Lanczos3);

        let bytes = BytesMut::with_capacity(320 * 240);
        let mut writer = bytes.writer();
        shared::image::encode(resized, &mut writer).unwrap();

        writer.into_inner().freeze()
    });
    eprintln!("-> sliced images at {} bytes each", slices[0].len());

    Ok((slices, data))
}
