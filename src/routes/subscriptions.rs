use std::result::Result;

use actix_web::{
    post,
    web::{Data, Form},
    HttpResponse,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    domain::{NewSubscriber, SubscriberEmail, SubscriberName},
    email_client::EmailClient,
};

#[derive(serde::Deserialize)]
struct FormData {
    email: String,
    name: String,
}

#[post("/subscriptions")]
#[tracing::instrument(
    name = "Adding a new subscriber.",
    skip(form, pool, email_client),
    fields(email = %form.email, name = %form.name)
)]
pub async fn subscribe(
    form: Form<FormData>,
    pool: Data<PgPool>,
    email_client: Data<EmailClient>,
) -> HttpResponse {
    let new_subscriber = match form.0.try_into() {
        Ok(subscriber) => subscriber,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };

    if insert_subscriber(&pool, &new_subscriber).await.is_err() {
        return HttpResponse::InternalServerError().finish();
    }
    if send_confirmation_email(&email_client, new_subscriber)
        .await
        .is_err()
    {
        return HttpResponse::InternalServerError().finish();
    }

    HttpResponse::Ok().finish()
}

impl TryFrom<FormData> for NewSubscriber {
    type Error = String;

    fn try_from(value: FormData) -> Result<Self, Self::Error> {
        let name = SubscriberName::parse(value.name.clone()).map_err(|e| e.to_string())?;
        let email = SubscriberEmail::parse(value.email.clone()).map_err(|e| e.to_string())?;
        Ok(NewSubscriber { email, name })
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
    INSERT INTO subscriptions (id, email, name, status)
    VALUES ($1, $2, $3, $4)
            "#,
        Uuid::new_v4(),
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
        "pending_confirmation".to_string()
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;
    Ok(())
}

#[tracing::instrument(
    name = "Sending confirmation email",
    skip(email_client, new_subscriber)
)]
pub async fn send_confirmation_email(
    email_client: &EmailClient,
    new_subscriber: NewSubscriber,
) -> Result<(), reqwest::Error> {
    let confirmation_link = "https://myapp.com/confirm";
    let html_body = format!(
        r#"
        <html>
            <body>
                <h1> Welcome to our newsletter! </h1>
                <p> You've successfully subscribed to our newsletter. </p>
                <p> Click <a href="{}"> here </a> to confirm your subscription. </p>
            </body>
        </html>
        "#,
        confirmation_link
    );
    let text_body = format!(
        r#"
        Welcome to our newsletter!
        You've successfully subscribed to our newsletter.
        Click here to confirm your subscription: {}
        "#,
        confirmation_link
    );
    email_client
        .send_email(new_subscriber.email, "Welcome!", &html_body, &text_body)
        .await
}
