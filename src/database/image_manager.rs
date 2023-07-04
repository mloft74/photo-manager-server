use deadpool_diesel::postgres::Pool;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};

use crate::{database::models::InsertableImage, domain::models::Image, schema};

#[derive(Clone)]
pub struct ImageManager {
    pool: Pool,
}

// It would be nice to hide this impl behind a trait object,
// but some trait bounds require a Clone impl.
// It's possible to get around this, but the effort to do so
// is not worth it for this small project.
impl ImageManager {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }

    pub async fn get_image(&self, file_name: String) -> Result<Image, Box<dyn std::error::Error>> {
        let get_name = file_name;
        let connection = self.pool.get().await?;
        let found_name = connection
            .interact(move |conn| {
                use schema::images::dsl::*;
                images
                    .filter(file_name.eq(get_name))
                    .select(file_name)
                    .first(conn)
            })
            .await??;

        Ok(Image {
            file_name: found_name,
        })
    }

    pub async fn save_image(&self, image: &Image) -> Result<(), Box<dyn std::error::Error>> {
        let insert = InsertableImage {
            file_name: image.file_name.clone(),
        };
        let connection = self.pool.get().await?;
        let rows_affected = connection
            .interact(|conn| {
                use schema::images::dsl::*;
                diesel::insert_into(images).values(insert).execute(conn)
            })
            .await??;
        tracing::debug!("save_image affected {} rows", rows_affected);

        Ok(())
    }
}
