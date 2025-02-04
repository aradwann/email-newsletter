use email_newsletter::{configuration::get_configuration, startup::run};
use reqwest::Client;
use sqlx::PgPool;
use std::net::TcpListener;
use uuid::Uuid;

pub struct TestApp {
    pub address: String,
    pub client: Client,
    pub db_pool: PgPool,
}

/// spin up an instance of our app and return its address and client for it
async fn spawn_app() -> TestApp {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    let mut configuration = get_configuration().expect("Failed to read configuration.");
    configuration.database.database_name = Uuid::new_v4().to_string();
    let connection_pool = configure_database(&configuration.database).await;

    let server = run(listener, connection_pool.clone()).expect("Failed to bind address");
    tokio::spawn(server);

    TestApp {
        address,
        client: Client::new(),
        db_pool: connection_pool,
    }
}

pub async fn configure_database(
    config: &email_newsletter::configuration::DatabaseSettings,
) -> PgPool {
    let connection_pool = PgPool::connect(&config.connection_string_without_db())
        .await
        .expect("Failed to connect to Postgres.");

    sqlx::query(&format!(r#"CREATE DATABASE "{}";"#, config.database_name))
        .execute(&connection_pool)
        .await
        .expect("Failed to create database.");

    let connection_pool = PgPool::connect(&config.connection_string())
        .await
        .expect("Failed to connect to Postgres.");

    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database.");

    connection_pool
}

#[tokio::test]
async fn health_check_works() {
    let app = spawn_app().await;

    let response = app
        .client
        .get(format!("{}/health-check", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    let status = response.status();

    if !status.is_success() {
        let body = response.text().await.expect("Failed to read response body");
        panic!("Request failed: status = {:?}, body = {:?}", status, body);
    }

    assert!(status.is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    let app = spawn_app().await;

    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    let response = app
        .client
        .post(format!("{}/subscriptions", &app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions")
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    let app = spawn_app().await;

    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = app
            .client
            .post(format!("{}/subscriptions", &app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request.");

        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not return a 400 when the payload was {}.",
            error_message
        );
    }
}
