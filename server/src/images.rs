use std::io::Cursor;

use crate::{error::Error, images::huggingface::Data};
use image::{imageops, GenericImageView, ImageReader};

pub mod huggingface;

// fn google() -> String {
//     const LATITUDE: f32 = 48.8584;
//     const LONGITUDE: f32 = 2.2945;

//     let url = format!(
//         "https://maps.googleapis.com/maps/api/streetview?size=320x240&location={LATITUDE},{LONGITUDE}&key={}",
//         env!("GOOGLE_API_KEY")
//     );

//     url
// }

pub async fn images() -> Result<([Vec<u8>; 3], Data), Error> {
    let random = rand::random_range(0..11054);
    let data = huggingface::fetch(random).await?;
    let bytes = reqwest::get(data.image.src.clone()).await?.bytes().await?;
    let bytes = bytes.to_vec();
    eprintln!("-> fetched {} bytes of image data", bytes.len());

    let img = ImageReader::new(Cursor::new(&bytes))
        .with_guessed_format()?
        .decode()?;

    let (width, height) = img.dimensions();
    let slice = width / 3;

    let slices: [Vec<u8>; 3] = std::array::from_fn(|i| {
        let view = img.view(slice * i as u32, 0, slice, height).to_image();
        let resized = imageops::resize(
            &view,
            320 * 2,
            240 * 2,
            image::imageops::FilterType::Lanczos3,
        );

        let mut buf = Vec::with_capacity(320*240);
        shared::image::encode(resized, Cursor::new(&mut buf)).unwrap();

        buf
    });
    eprintln!("-> sliced images at {} bytes each", slices[0].len());

    Ok((slices, data))
}
