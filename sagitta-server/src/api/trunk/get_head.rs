use actix_web::{post, web, Responder};
use sagitta_api_schema::trunk::get_head::{TrunkGetHeadRequest, TrunkGetHeadResponse};

use crate::state::ApiState;

#[post("/trunk/get-head")]
pub async fn trunk_get_head(
    state: web::Data<ApiState>,
    _req: web::Json<TrunkGetHeadRequest>,
) -> impl Responder {
    let res = state
        .remote_system_workspace_manager
        .get_head(None)
        .unwrap();
    let res = TrunkGetHeadResponse { id: res };
    web::Json(res)
}
