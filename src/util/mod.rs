mod response;

use std::collections::HashMap;
use color_eyre::Result;
use mongodb::Cursor;
use crate::api::schema::circle::Circle;

pub async fn fetch_circles(cursor: &mut Cursor<Circle>) -> Result<HashMap<String, Circle>> {
    let mut new_circles = HashMap::new();
    while cursor.advance().await? {
        let doc = cursor.current();
        let circle = Circle {
            id: doc.get_str("_id")?.to_string(),
            name: doc.get_str("name")?.to_string(),
            description: doc.get_str("description")?.to_string(),
            image_url: doc.get_str("imageUrl")?.to_string(),
            channel: doc.get_str("channel")?.to_string(),
            emoji: doc.get_str("emoji")?.to_string(),
            owner: doc.get_str("owner")?.to_string(),
            created_on: doc.get_datetime("createdOn")?,
            sub_channels: doc
                .get_array("subChannels")?
                .into_iter()
                .map(|x| x.unwrap().as_str().unwrap().to_string())
                .collect(),
        };
        new_circles.insert(circle.id.clone(), circle);
    };
    Ok(new_circles)
}