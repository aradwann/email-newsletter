#[tokio::test]
async fn health_check_works() {
    let app = crate::helpers::spawn_app().await;

    let response = app.health_check().await;

    let status = response.status();

    if !status.is_success() {
        let body = response.text().await.expect("Failed to read response body");
        panic!("Request failed: status = {:?}, body = {:?}", status, body);
    }

    assert!(status.is_success());
    assert_eq!(Some(0), response.content_length());
}
