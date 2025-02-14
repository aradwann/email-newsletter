use std::result::Result;

use actix_web::{
    post,
    web::{Data, Form},
    HttpResponse,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::{NewSubscriber, SubscriberName};

#[derive(serde::Deserialize)]
struct FormData {
    email: String,
    name: String,
}

#[post("/subscriptions")]
#[tracing::instrument(
    name = "Adding a new subscriber.",
    skip(form, pool),
    fields(email = %form.email, name = %form.name)
)]
pub async fn subscribe(form: Form<FormData>, pool: Data<PgPool>) -> HttpResponse {
    let subscriber_name = SubscriberName::parse(form.name.clone());
    if subscriber_name.is_err() {
        return HttpResponse::BadRequest().finish();
    }
    let new_subscriber = NewSubscriber {
        email: form.email.clone(),
        name: subscriber_name.unwrap(),
    };

    match insert_subscriber(&pool, &new_subscriber).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(new_subscriber, pool)
)]
pub async fn insert_subscriber(
    pool: &PgPool,
    new_subscriber: &NewSubscriber,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
    INSERT INTO subscriptions (id, email, name)
    VALUES ($1, $2, $3)
            "#,
        Uuid::new_v4(),
        new_subscriber.email,
        new_subscriber.name.inner_ref(),
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;
    Ok(())
}
