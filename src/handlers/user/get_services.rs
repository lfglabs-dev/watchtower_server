use std::sync::Arc;

use axum::{extract::State, response::IntoResponse, Json};
use mongodb::bson::{doc, Document, RawBsonRef};

use crate::{
    structs,
    utils::{
        check_auth_token::check_auth_token, get_token_data::get_token_data,
        has_permission::has_permission,
    },
    AppState,
};

pub async fn get_services_handler(
    State(app_state): State<Arc<AppState>>,
    Json(body): Json<structs::AuthTokenJSON>,
) -> impl IntoResponse {
    let token = body.token;
    let valid = check_auth_token(app_state.clone(), token.clone());
    if !valid {
        let json_response = serde_json::json!({
            "status": "error",
            "message": "Invalid token or token expired",
            "error_code": "invalid_token"
        });

        return Json(json_response);
    }

    let token_data = get_token_data(app_state.clone(), token);
    let user_id = token_data.user_id;

    // get from mongodb
    let services: Vec<structs::Service> = get_services(app_state, user_id).await.unwrap();

    return Json(serde_json::json!({
        "status": "success",
        "services": services,
    }));
}

async fn get_services(
    app_state: Arc<AppState>,
    user_id: String,
) -> Result<Vec<structs::Service>, mongodb::error::Error> {
    let db = &app_state.db;
    let collection: mongodb::Collection<Document> = db.collection("services");

    let mut cursor = collection.find(doc! {}, None).await?;

    let mut result: Vec<structs::Service> = Vec::new();
    while cursor.advance().await? {
        let doc = cursor.current();
        let _id = doc.get("_id").unwrap().unwrap().as_object_id().unwrap();
        let app_name = doc.get("app_name").unwrap().unwrap().as_str().unwrap();
        let whitelist = match doc.get("whitelist") {
            Ok(Some(RawBsonRef::Array(arr))) => {
                let mut whitelist: Vec<String> = Vec::new();
                for item in arr.into_iter() {
                    if let Ok(RawBsonRef::String(s)) = item {
                        whitelist.push(s.to_string());
                    }
                }
                whitelist
            }
            _ => {
                let empty_array: Vec<String> = Vec::new();
                empty_array
            }
        };
        let service = structs::Service {
            _id: Some(_id.to_hex()),
            app_name: Some(app_name.to_string()),
            whitelist: Some(whitelist),
        };
        let is_admin = has_permission(
            user_id.clone(),
            "administrator".to_string(),
            app_state.clone(),
        )
        .await;
        if !is_admin {
            if let Some(whitelist) = &service.whitelist {
                if !whitelist.contains(&user_id) {
                    continue;
                }
            }
        }
        result.push(service);
    }

    return Ok(result);
}
