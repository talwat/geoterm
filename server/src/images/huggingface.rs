use serde::Deserialize;

use crate::error::Error;

#[derive(Debug, Deserialize)]
pub struct Image {
    pub src: String,
    pub height: u16,
    pub width: u16,
}

#[derive(Debug)]
pub struct Data {
    pub image: Image,
    pub coordinates: (f32, f32),
    pub country: [char; 2],
}

#[derive(Debug, Deserialize)]
struct RowContent {
    image: Image,
    longitude: String,
    latitude: String,
    #[serde(rename = "country_iso_alpha2")]
    country: String,
}

#[derive(Debug, Deserialize)]
struct Row {
    #[serde(rename = "row")]
    content: RowContent,
}

#[derive(Debug, Deserialize)]
struct Response {
    rows: Vec<Row>,
}

impl Response {
    fn flatten(mut self) -> Result<Data, Error> {
        let content = self.rows.swap_remove(0).content;
        let mut chars = content.country.chars();

        Ok(Data {
            image: content.image,
            coordinates: (content.longitude.parse()?, content.latitude.parse()?),
            country: [chars.next().unwrap(), chars.next().unwrap()],
        })
    }
}

pub async fn fetch(index: usize) -> Result<Data, Error> {
    let url = format!(
        "https://datasets-server.huggingface.co/rows?dataset=yunusserhat%2Frandom_streetview_images&config=default&split=train&offset={index}&length=1"
    );
    let resp = reqwest::get(url).await?;
    let parsed: Response = resp.json().await?;

    parsed.flatten()
}
