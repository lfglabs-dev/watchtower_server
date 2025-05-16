use std::sync::Arc;

use mongodb::{
    bson::{Document, RawBsonRef},
    Collection,
};

use crate::{utils::user::db::save_db::save_db, AppState};

pub async fn save_dbs(
    app_state: Arc<AppState>,
    hourly_only: bool,
) -> Result<(), mongodb::error::Error> {
    // Save every db
    let db = &app_state.db;
    let collection: Collection<Document> = db.collection("databases");
    let mut databases_cursor = collection.clone().find(None, None).await.unwrap();
    while databases_cursor.advance().await? {
        let doc = databases_cursor.current();
        let db_id = doc.get("_id").unwrap().unwrap().as_object_id().unwrap();
        let db_name = doc.get("name").unwrap().unwrap().as_str().unwrap();
        let connection_string = doc
            .get("connection_string")
            .unwrap()
            .unwrap()
            .as_str()
            .unwrap();
        // Optional fields
        let custom_name = doc
            .get("custom_name")
            .unwrap()
            .unwrap_or(doc.get("name").unwrap().unwrap())
            .as_str()
            .unwrap_or(db_name);
        let hourly_save = doc
            .get("hourly_save")
            .unwrap()
            .unwrap_or(RawBsonRef::Null)
            .as_bool()
            .unwrap_or(false);
        if hourly_only && !hourly_save {
            println!(
                "❌ Skipping db {}: hourly_only = {}",
                custom_name, hourly_only
            );
            continue;
        }
        let res = save_db(
            db,
            connection_string.to_string(),
            db_name.to_string(),
            db_id,
            false,
        )
        .await;
        if res.is_ok() {
            println!("✅ Saved db {}", custom_name);
        } else {
            println!(
                "❌ Failed to save db {}: {}",
                custom_name,
                res.err().unwrap()
            );
        }
    }
    return Ok(());
}
