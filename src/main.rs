use std::net::TcpListener;

use email_newsletter::telemetry::{get_subscriber, init_subscriber};
use email_newsletter::{configuration::get_configuration, startup::run};
#[tokio::main]
async fn main() -> std::io::Result<()> {
    // setup tracing/logger subscriber
    let subscriber = get_subscriber("email_newsletter".into(), "info".into());
    init_subscriber(subscriber);

    // fetch configuration
    let configuration = get_configuration().expect("Failed to read configuration.");

    // connect to Postgres
    let connection_pool = sqlx::PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to Postgres.");

    // bind to the address
    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(address)?;

    // run the application
    run(listener, connection_pool)?.await?;

    Ok(())
}
