use json::JsonValue;
use shared::Coordinate;

use crate::error::Error;

#[derive(Debug)]
pub struct Image {
    pub src: String,
    pub height: u16,
    pub width: u16,
}

#[derive(Debug)]
pub struct Data {
    pub image: Image,
    pub coordinates: shared::Coordinate,
    pub country: [char; 2],
    pub address: String,
}

// impl Response {
//     fn flatten(mut self) -> Result<Data, Error> {
//         let content = self.rows.swap_remove(0).content;
//         let mut chars = content.country.chars();

//         Ok(Data {
//             image: content.image,
//             coordinates: (content.longitude.parse()?, content.latitude.parse()?),
//             country: [chars.next().unwrap(), chars.next().unwrap()],
//             address: content.address,
//         })
//     }
// }

pub fn parse(json: JsonValue) -> Option<Data> {
    let row = &json["rows"][0]["row"];

    let image = &row["image"];
    let src = image["src"].as_str()?;
    let height = image["height"].as_u16()?;
    let width = image["width"].as_u16()?;
    let image = Image {
        src: src.to_owned(),
        height,
        width,
    };

    let latitude: f32 = row["latitude"].as_str()?.parse().ok()?;
    let longitude: f32 = row["longitude"].as_str()?.parse().ok()?;
    let address: String = row["address"].as_str()?.to_owned();
    let country: [char; 2] = row["country_iso_alpha2"]
        .as_str()?
        .chars()
        .take(2)
        .collect::<Vec<_>>()
        .try_into()
        .ok()?;

    Some(Data {
        image,
        coordinates: Coordinate {
            latitude,
            longitude,
        },
        country,
        address,
    })
}

pub async fn fetch(index: usize) -> Result<Data, Error> {
    let url = format!(
        "https://datasets-server.huggingface.co/rows?dataset=yunusserhat%2Frandom_streetview_images&config=default&split=train&offset={index}&length=1"
    );
    let resp = reqwest::get(url).await?;
    let bytes = resp.bytes().await?;
    let string = str::from_utf8(&bytes)?;
    let json = json::parse(string)?;

    Ok(parse(json).unwrap())
}
