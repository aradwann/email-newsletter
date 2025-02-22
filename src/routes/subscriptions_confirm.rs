use actix_web::{get, web, HttpResponse};
use uuid::Uuid;

#[derive(serde::Deserialize)]
struct ConfirmParams {
    token: Uuid,
}

#[get("/subscriptions/confirm")]
#[tracing::instrument(name = "Confirm a pending subscriber", skip(_params))]
pub async fn confirm(_params: web::Query<ConfirmParams>) -> HttpResponse {
    HttpResponse::Ok().finish()
}
