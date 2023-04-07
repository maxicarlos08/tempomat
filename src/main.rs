use tempomat::{config::Config, oauth::actions};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    // This is only temporary
    let _ = actions::login(&Config {
        atlassian_instance: "volateq".to_string(),
    })
    .await;
}
