use std::fmt::Display;

use color_eyre::Result;
use firestore::{struct_path::path, FirestoreDb};
use serenity::prelude::Context;
use tracing::info;

use crate::api::schema::{circle::Circle, coper::Coper, response::Response};

use super::super::settings::Settings;

pub struct FSManager {
    pub client: FirestoreDb,
    #[allow(dead_code)]
    key_file: String,
}

impl FSManager {
    pub async fn new() -> Self {
        let settings = Settings::new();
        std::env::set_var(
            "GOOGLE_APPLICATION_CREDENTIALS",
            &settings.firestore.key_filename,
        );
        let client = FirestoreDb::new(&settings.firestore.project_id)
            .await
            .expect("Failed to create Firestore client");
        Self {
            client,
            key_file: settings.firestore.key_filename.clone(),
        }
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
        let client = self.client.fluent();
        let res: Response = client
            .insert()
            .into("response")
            .document_id(tipe.to_string())
            .object(&Response {
                type_field: tipe.to_string(),
                message: msg.to_string(),
            })
            .execute()
            .await?;

        let mut data = ctx.data.write().await;
        let cache = data
            .get_mut::<Response>()
            .ok_or(eyre::eyre!("Unable to get cache"))?;
        cache.insert(tipe.to_string(), res);
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
        let client = self.client.fluent();
        client
            .delete()
            .from("response")
            .document_id(msg)
            .execute()
            .await?;

        let mut data = ctx.data.write().await;
        let cache = data
            .get_mut::<Response>()
            .ok_or(eyre::eyre!("Unable to get cache"))?;
        cache.remove(&msg.to_string());

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
        let db = self.client.fluent();
        let res: Option<Coper> = db
            .clone()
            .select()
            .by_id_in("coper")
            .obj()
            .one(coper_id)
            .await?;
        let res: Coper = match res {
            Some(coper) => {
                let updated_coper = Coper {
                    id: coper.id,
                    score: coper.score + 1,
                };
                self.coper_update(&ctx, coper_id, &updated_coper).await?;
                updated_coper
            }
            None => {
                let new_coper = Coper {
                    id: coper_id.to_string(),
                    score: 1,
                };
                self.coper_add(&ctx, &new_coper).await?;
                new_coper
            }
        };
        let mut data = ctx.data.write().await;
        let cache = data
            .get_mut::<Coper>()
            .ok_or(eyre::eyre!("Unable to get cache"))?;

        cache.insert(coper_id.to_string(), res);
        Ok(())
    }

    /// Add a coper into the database
    /// # Arguments
    /// * `ctx` - The context of the command
    /// * `coper` - The coper to add
    /// # Errors
    /// * If the database is unable to add the coper
    /// * If the cache is unable to add the coper
    async fn coper_add(&self, ctx: &Context, coper: &Coper) -> Result<String> {
        let db = self.client.fluent();
        let res: Coper = db
            .insert()
            .into("coper")
            .document_id(&coper.id)
            .object(coper)
            .execute()
            .await?;

        let res_id = res.id.clone();
        let mut data = ctx.data.write().await;
        let cache = data.get_mut::<Coper>().unwrap();
        cache.insert(res_id.to_string(), res);
        Ok(res_id)
    }

    /// Update a coper in the database
    /// # Arguments
    /// * `ctx` - The context of the command
    /// * `id` - The id of the coper to update
    /// * `new_data` - The new data to update the coper with
    /// # Errors
    /// * If the database is unable to update the coper
    async fn coper_update(&self, ctx: &Context, id: &str, new_data: &Coper) -> Result<String> {
        let db = self.client.fluent();
        let res: Coper = db
            .update()
            .fields(vec![path!(Coper::score)])
            .in_col("coper")
            .document_id(id)
            .object(new_data)
            .execute()
            .await?;
        let res_id = res.id.clone();
        let mut data = ctx.data.write().await;
        let cache = data
            .get_mut::<Coper>()
            .ok_or(eyre::eyre!("Unable to get cache"))?;
        cache.insert(id.to_string(), res);
        Ok(res_id)
    }

    pub async fn circle_add(&self, ctx: &Context, circle_data: Circle) -> Result<()> {
        let db = self.client.fluent();
        let res: Circle = db
            .insert()
            .into("circle")
            .document_id(&circle_data.id)
            .object(&circle_data)
            .execute()
            .await?;

        let mut data = ctx.data.write().await;
        let cache = data
            .get_mut::<Circle>()
            .ok_or(eyre::eyre!("Unable to get cache"))?;
        cache.insert(res.id.clone(), res);
        info!("Added circle to cache");
        Ok(())
    }

    pub async fn circle_remove(&self, ctx: Context, circle_id: &str) -> Result<()> {
        let db = self.client.fluent();
        db.delete()
            .from("circle")
            .document_id(circle_id)
            .execute()
            .await?;

        let mut data = ctx.data.write().await;
        let cache = data
            .get_mut::<Circle>()
            .ok_or(eyre::eyre!("Unable to get cache"))?;
        cache.remove(circle_id);
        Ok(())
    }

    pub async fn circle_update(
        &self,
        ctx: Context,
        circle_id: &str,
        new_data: Circle,
    ) -> Result<()> {
        let db = self.client.fluent();
        let res: Circle = db
            .update()
            .fields(vec![
                path!(Circle::name),
                path!(Circle::description),
                path!(Circle::image_url),
                path!(Circle::emoji),
            ])
            .in_col("circle")
            .document_id(circle_id)
            .object(&new_data)
            .execute()
            .await?;

        let res_id = res.id.clone();
        let mut data = ctx.data.write().await;
        let cache = data
            .get_mut::<Circle>()
            .ok_or(eyre::eyre!("Unable to get cache"))?;
        cache.insert(res_id, res);
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
