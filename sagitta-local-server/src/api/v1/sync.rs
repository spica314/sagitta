use actix_web::{post, web, Responder};
use sagitta_local_api_schema::v1::sync::{V1SyncRequest, V1SyncResponse};

use crate::api_state::ApiState;

#[post("/v1/sync")]
pub async fn v1_sync(
    _state: web::Data<ApiState>,
    _req: web::Json<V1SyncRequest>,
) -> impl Responder {
    let res = V1SyncResponse {};

    web::Json(res)
}
