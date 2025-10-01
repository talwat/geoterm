use crate::error::Error;

// TODO: Add params...
pub async fn image() -> Result<Vec<u8>, Error> {
    const LATITUDE: f32 = 48.8584;
    const LONGITUDE: f32 = 2.2945;

    let url = format!(
        "https://maps.googleapis.com/maps/api/streetview?size=320x240&location={LATITUDE},{LONGITUDE}&key={}",
        env!("GOOGLE_API_KEY")
    );

    let resp = reqwest::get(url).await?;
    let bytes = resp.bytes().await?;

    Ok(bytes.to_vec())
}
