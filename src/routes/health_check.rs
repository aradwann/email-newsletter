use actix_web::{get, HttpResponse};

#[get("/health-check")]
pub async fn health_check() -> HttpResponse {
    HttpResponse::Ok().finish()
}
