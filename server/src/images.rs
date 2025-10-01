use crate::{error::Error, images::huggingface::Data};

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

    // TODO: Actually split the image into 3...
    Ok(([bytes.clone(), bytes.clone(), bytes], data))
}
