use std::fmt::Display;

use color_eyre::Result;
use mongodb::{
    bson::{self, doc, Document},
    options::FindOneAndUpdateOptions,
    Client,
};
use serenity::prelude::Context;
use tracing::info;

use crate::{
    api::schema::{circle::Circle, coper::Coper, response::Response},
    settings::Settings,
};

pub struct Mongo {
    pub client: Client,
}

impl Mongo {
    pub async fn new(settings: &Settings) -> Self {
        let client = Client::with_uri_str(settings.database_url.clone())
            .await
            .expect("Unable to connect to database");
        Self { client }
    }

    /// Add a response into the database
    /// # Arguments
    /// * `ctx` - The context of the command
    /// * `tipe` - The type of the response
    /// * `msg` - The message of the response
    /// # Errors
    /// * If the database is unable to insert the response
    /// * If the cache is unable to insert the response
    pub async fn response_add(&self, ctx: Context, tipe: ResponseType, msg: &str) -> Result<()> {
        let db = self.client.database("discord");
        let new_doc = doc! {
            "type": tipe.to_string(),
            "message": msg
        };

        let _ = db.collection("response").insert_one(new_doc, None).await?;

        let mut data = ctx.data.write().await;
        let cache = data
            .get_mut::<Response>()
            .ok_or(eyre::eyre!("Unable to get cache"))?;
        cache.insert(
            tipe.to_string(),
            Response {
                type_field: tipe.to_string(),
                message: msg.to_string(),
            },
        );
        Ok(())
    }

    /// Delete a response from the database
    /// # Arguments
    /// * `ctx` - The context of the command
    /// * `msg` - The message of the response
    /// # Errors
    /// * If the database is unable to delete the response
    /// * If the cache is unable to delete the response
    /// * If the response is not found
    pub async fn response_delete(&self, ctx: Context, msg: &str) -> Result<()> {
        let db = self.client.database("discord");
        let res: Document = db
            .collection("response")
            .find_one_and_delete(doc! { "message": msg}, None)
            .await?
            .ok_or(eyre::eyre!("Unable to find response"))?;

        let res_id = res.get_object_id("_id")?.to_string();

        let mut data = ctx.data.write().await;
        let cache = data
            .get_mut::<Response>()
            .ok_or(eyre::eyre!("Unable to get cache"))?;
        cache.remove(&res_id);

        Ok(())
    }

    /// Either add or update a `coper` in the database
    /// # Arguments
    /// * `ctx` - The context of the command
    /// * `coper` - The coper to add or update
    /// # Errors
    /// * If the database is unable to add or update the coper
    /// * If the cache is unable to add or update the coper
    /// * If the coper is not found (in cache or database)
    pub async fn coper_increment(&self, ctx: Context, coper_id: &str) -> Result<()> {
        let db = self.client.database("discord");
        let coper_doc: Option<Document> = db
            .collection("copers")
            .find_one(doc! { "id": coper_id }, None)
            .await?;

        let mut data = ctx.data.write().await;
        let cache = data
            .get_mut::<Coper>()
            .ok_or(eyre::eyre!("Unable to get cache"))?;
        if let Some(coper_doc) = coper_doc {
            let coper_id = coper_doc.get_object_id("_id")?.to_string();
            let entry = cache
                .get_mut(&coper_id)
                .ok_or(eyre::eyre!("Unable to find coper"))?;
            entry.score += 1;
            let res: Option<Document> = db
                .collection("copers")
                .find_one_and_update(
                    doc! { "id": &coper_id },
                    doc! { "$inc": { "score": 1 } },
                    FindOneAndUpdateOptions::builder().upsert(true).build(),
                )
                .await?;

            if let Some(res) = res {
                info!("Update coper {:#?}", res)
            } else {
                info!("Coper with id: {coper_id} was not found in the database")
            }
        } else {
            let new_coper = Coper {
                id: coper_id.to_string(),
                score: 1,
            };
            let res = db
                .collection("copers")
                .insert_one(new_coper.clone(), None)
                .await?;
            let res_id = res
                .inserted_id
                .as_object_id()
                .ok_or(eyre::eyre!("Unable to get id"))?
                .to_string();
            cache.insert(res_id, new_coper);
        }

        Ok(())
    }
    pub async fn coper_add(&self, ctx: Context, coper: Document) -> Result<()> {
        let new_coper = bson::from_document::<Coper>(coper.clone())?;

        let db = self.client.database("discord");
        let res = db.collection("copers").insert_one(new_coper, None).await?;

        let res_id = res.inserted_id.as_object_id().unwrap().to_string();
        let mut data = ctx.data.write().await;
        let cache = data.get_mut::<Coper>().unwrap();
        cache.insert(res_id, bson::from_document::<Coper>(coper)?);
        Ok(())
    }

    pub async fn coper_update(&self, ctx: Context, id: &str, new_data: Document) -> Result<()> {
        let db = self.client.database("discord");
        let res: Document = db
            .collection("copers")
            .find_one_and_update(doc! { "id": id }, new_data, None)
            .await?
            .ok_or(eyre::eyre!("Unable to find coper"))?;

        let res_id = res.get_object_id("_id").unwrap().to_string();
        let mut data = ctx.data.write().await;
        let cache = data
            .get_mut::<Coper>()
            .ok_or(eyre::eyre!("Unable to get cache"))?;
        cache.insert(res_id, bson::from_document::<Coper>(res)?);
        Ok(())
    }

    pub async fn circle_add(&self, ctx: &Context, circle_data: Document) -> Result<()> {
        let db = self.client.database("discord");
        let res = db
            .collection("circle")
            .insert_one(circle_data.clone(), None)
            .await?;

        let res_id = res
            .inserted_id
            .as_object_id()
            .ok_or(eyre::eyre!("Unable to get the new id"))?
            .to_string();

        let mut circle_data = circle_data.clone();
        circle_data.insert("_id", res_id.clone());

        let new_circle = bson::from_document::<Circle>(circle_data)?;

        let mut data = ctx.data.write().await;
        let cache = data
            .get_mut::<Circle>()
            .ok_or(eyre::eyre!("Unable to get cache"))?;
        cache.insert(res_id, new_circle);
        info!("Added circle to cache");
        Ok(())
    }

    pub async fn circle_remove(&self, ctx: Context, circle_id: &str) -> Result<()> {
        let db = self.client.database("discord");
        let _: Document = db
            .collection("circle")
            .find_one_and_delete(doc! { "id": circle_id}, None)
            .await
            .expect("Unable to find circle")
            .ok_or_else(|| eyre::eyre!("Unable to find circle"))?;

        let mut data = ctx.data.write().await;
        let cache = data.get_mut::<Circle>().unwrap();
        cache.remove(circle_id);
        Ok(())
    }

    pub async fn circle_update(
        &self,
        ctx: Context,
        circle_id: &str,
        new_data: Document,
    ) -> Result<()> {
        let db = self.client.database("discord");
        let res: Document = db
            .collection("circle")
            .find_one_and_update(doc! { "id": circle_id }, new_data, None)
            .await?
            .ok_or(eyre::eyre!("Unable to find circle"))?;

        let res_id = res.get_object_id("_id")?.to_string();
        let mut data = ctx.data.write().await;
        let cache = data
            .get_mut::<Circle>()
            .ok_or(eyre::eyre!("Unable to get cache"))?;
        cache.insert(res_id, bson::from_document::<Circle>(res)?);

        Ok(())
    }
}

pub enum ResponseType {
    Error,
    Basic,
    Cors,
    Default,
    Opaque,
    OpaqueDirect,
}

impl Display for ResponseType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Error => write!(f, "error"),
            Self::Basic => write!(f, "basic"),
            Self::Cors => write!(f, "cors"),
            Self::Default => write!(f, "default"),
            Self::Opaque => write!(f, "opaque"),
            Self::OpaqueDirect => write!(f, "opaquedirect"),
        }
    }
}
