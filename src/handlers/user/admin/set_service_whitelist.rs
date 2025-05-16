use std::sync::Arc;

use axum::{extract::State, response::IntoResponse, Json};
use mongodb::bson::{doc, Document};
use serde::Deserialize;

use crate::{
    utils::{
        check_auth_token::check_auth_token, get_token_data::get_token_data,
        has_permission::has_permission,
    },
    AppState,
};

#[derive(Deserialize)]
pub struct SetServiceWhitelistInput {
    token: String,
    new_whitelist: Vec<String>,
    service_id: String,
}

pub async fn set_service_whitelist_handler(
    State(app_state): State<Arc<AppState>>,
    Json(body): Json<SetServiceWhitelistInput>,
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

    let has_perm = has_permission(
        token_data.user_id,
        "administrator".to_string(),
        app_state.clone(),
    )
    .await;

    if !has_perm {
        let json_response = serde_json::json!({
            "status": "error",
            "message": "You don't have administrator permission",
            "error_code": "permission_denied"
        });

        return Json(json_response);
    }

    let service_id = mongodb::bson::oid::ObjectId::parse_str(&body.service_id).unwrap();
    let new_whitelist = body.new_whitelist;

    let db = app_state.db.clone();

    let collection: mongodb::Collection<Document> = db.collection("services");
    let res = collection
        .update_one(
            doc! { "_id": service_id },
            doc! { "$set": { "whitelist": new_whitelist } },
            None,
        )
        .await
        .unwrap();

    if res.modified_count == 0 {
        let json_response = serde_json::json!({
            "status": "error",
            "message": "Service not found",
            "error_code": "service_not_found"
        });

        return Json(json_response);
    }
    return Json(serde_json::json!({
        "status": "success",
    }));
}
