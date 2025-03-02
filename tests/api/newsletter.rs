use wiremock::{
    Mock, ResponseTemplate,
    matchers::{method, path},
};

use crate::helpers::spawn_app;

#[tokio::test]
async fn newsletters_are_not_delivered_to_pending_subscribers() {
    // Arrange
    let app = spawn_app().await;
    create_unconfirmed_subscriber(&app).await;

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&app.email_server)
        .await;

    let newsletter_request_body = serde_json::json!({
        "title": "Welcome!",
        "content": {
            "text":"Welcome to our newsletter!",
            "html": "<h1>Welcome to our newsletter!</h1>"
        },
    });

    let response = reqwest::Client::new()
        .post(&format!("{}/newsletter", app.address))
        .json(&newsletter_request_body)
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(response.status().as_u16(), 200);
}

async fn create_unconfirmed_subscriber(app: &crate::helpers::TestApp) {
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let _mock_gaurd = Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .named("Create unconfirmed subscriber")
        .expect(1)
        .mount_as_scoped(&app.email_server)
        .await;

    app.post_subscriptions(body.into())
        .await
        .error_for_status()
        .unwrap();
}
