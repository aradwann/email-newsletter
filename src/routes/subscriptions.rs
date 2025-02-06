use actix_web::{
    post,
    web::{Data, Form},
    HttpResponse,
};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(serde::Deserialize)]
struct FormData {
    email: String,
    name: String,
}

#[post("/subscriptions")]
pub async fn subscribe(form: Form<FormData>, pool: Data<PgPool>) -> HttpResponse {
    let request_id = Uuid::new_v4();
    let requset_span = tracing::info_span!(
        "Handling a new subscription request.",
        %request_id,
        email = %form.email,
        name = %form.name
    );
    let _request_span_guard = requset_span.enter();
    tracing::info!(
        "request_id '{}' - Adding '{}' '{}' as a new subscriber.",
        request_id,
        form.email,
        form.name
    );

    match sqlx::query!(
        r#"INSERT INTO subscriptions (id, email, name) VALUES ($1, $2, $3)"#,
        Uuid::new_v4(),
        form.email,
        form.name,
    )
    .execute(pool.get_ref())
    .await
    {
        Ok(_) => {
            tracing::info!(
                "request_id '{}' - New subscriber details have been saved.",
                request_id
            );
            HttpResponse::Ok().finish()
        }
        Err(e) => {
            tracing::error!(
                "request_id '{}' - Failed to execute query: {:?}",
                request_id,
                e
            );
            HttpResponse::InternalServerError().finish()
        }
    }
}
