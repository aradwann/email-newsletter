use email_newsletter::run;
use reqwest::Client;
use std::net::TcpListener;

pub struct TestApp {
    pub address: String,
    pub client: Client,
}

async fn spawn_app() -> TestApp {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    let server = run(listener).expect("Failed to bind address");
    tokio::spawn(server);

    TestApp {
        address,
        client: Client::new(), // Move the client here for reuse
    }
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
