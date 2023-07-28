use std::collections::HashMap;

use async_trait::async_trait;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, DbErr, EntityTrait,
    QueryFilter, TransactionTrait,
};

use crate::{
    domain::{actions::images::ImageCanonUpdater, models::Image},
    persistence::{
        db_image::active_model_for_insert_from,
        entities::{images, prelude::Images},
    },
};

#[derive(Clone)]
pub struct DbImageCanonUpdater {
    db_conn: DatabaseConnection,
}

impl DbImageCanonUpdater {
    pub fn new(db_conn: DatabaseConnection) -> Self {
        Self { db_conn }
    }
}

#[async_trait]
impl ImageCanonUpdater for DbImageCanonUpdater {
    async fn update_canon<'a, T: Iterator<Item = &'a Image> + Send>(
        &self,
        canon: T,
    ) -> Result<(), String> {
        let mut models = {
            let models = Images::find()
                .all(&self.db_conn)
                .await
                .map_err(|e| e.to_string())?;
            let mut model_map = HashMap::new();
            for model in models {
                model_map.insert(model.file_name.clone(), model);
            }
            model_map
        };

        let canon: Vec<_> = canon.collect();
        let mut updates = Vec::new();
        let mut inserts: Vec<images::ActiveModel> = Vec::new();
        for image in canon {
            let model = models.remove(&image.file_name);
            if let Some(model) = model {
                let image_width = image.width as i32;
                let image_height = image.height as i32;
                let dimm_active_values = determine_dimm_active_values(
                    (model.width, model.height),
                    (image_width, image_height),
                );
                if let Some((width, height)) = dimm_active_values {
                    updates.push(images::ActiveModel {
                        id: ActiveValue::Unchanged(model.id),
                        file_name: ActiveValue::Unchanged(model.file_name),
                        width,
                        height,
                    });
                }
            } else {
                inserts.push(active_model_for_insert_from(image));
            }
        }

        let delete_ids: Vec<_> = models.into_iter().map(|e| e.1.id).collect();

        self.db_conn
            .transaction::<_, (), DbErr>(|txn| {
                Box::pin(async move {
                    if !delete_ids.is_empty() {
                        Images::delete_many()
                            .filter(images::Column::Id.is_in(delete_ids))
                            .exec(txn)
                            .await?;
                    }
                    if !inserts.is_empty() {
                        Images::insert_many(inserts).exec(txn).await?;
                    }
                    for update in updates {
                        update.update(txn).await?;
                    }

                    Ok(())
                })
            })
            .await
            .map_err(|e| e.to_string())?;

        Ok(())
    }
}

fn determine_dimm_active_values(
    model_dimm: (i32, i32),
    image_dimm: (i32, i32),
) -> Option<(ActiveValue<i32>, ActiveValue<i32>)> {
    if model_dimm == image_dimm {
        None
    } else {
        let width = if model_dimm.0 == image_dimm.0 {
            ActiveValue::Unchanged(model_dimm.0)
        } else {
            ActiveValue::Set(image_dimm.0)
        };
        let height = if model_dimm.1 == image_dimm.1 {
            ActiveValue::Unchanged(model_dimm.1)
        } else {
            ActiveValue::Set(image_dimm.1)
        };
        Some((width, height))
    }
}
