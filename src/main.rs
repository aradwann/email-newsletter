use std::net::TcpListener;

use email_newsletter::email_client::EmailClient;
use email_newsletter::telemetry::{get_subscriber, init_subscriber};
use email_newsletter::{configuration::get_configuration, startup::run};
#[tokio::main]
async fn main() -> std::io::Result<()> {
    // setup tracing/logger subscriber
    let subscriber = get_subscriber("email_newsletter".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    // fetch configuration
    let configuration = get_configuration().expect("Failed to read configuration.");

    // connect to Postgres
    let connection_pool = sqlx::PgPool::connect_lazy_with(configuration.database.with_db());

    // create an email client
    let sender_email = configuration
        .email_client
        .sender()
        .expect("invalid sender email");

    let timeout = configuration.email_client.timeout();
    let email_client = EmailClient::new(
        configuration.email_client.base_url,
        sender_email,
        configuration.email_client.auth_token,
        timeout,
    );

    // bind to the address
    let address = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    );
    let listener = TcpListener::bind(address)?;

    // run the application
    run(listener, connection_pool, email_client)?.await?;

    Ok(())
}
