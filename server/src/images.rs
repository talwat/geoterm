use std::io::Cursor;

use crate::{error::Error, images::huggingface::Data};
use bytes::Bytes;
use image::{GenericImageView, ImageReader, imageops};
use shared::image::{HEIGHT, WIDTH};

pub mod huggingface;

pub async fn images() -> Result<([Bytes; 3], Data), Error> {
    let random = rand::random_range(0..11054);
    let data = huggingface::fetch(random).await?;
    eprintln!("-> {}", data.image.src.clone());
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
        let resized = imageops::resize(&view, WIDTH, HEIGHT, image::imageops::FilterType::Lanczos3);
        let bytes = shared::image::encode(resized).unwrap();

        bytes
    });

    eprintln!(
        "-> compressed images at {}, {}, {} bytes",
        slices[0].len(),
        slices[1].len(),
        slices[2].len()
    );

    Ok((slices, data))
}
