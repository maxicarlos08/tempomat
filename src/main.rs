use tempomat::oauth::server::OAuthServer;

#[tokio::main]
async fn main() {
    let result = OAuthServer::start(([127, 0, 0, 1], 3000).into()).await;

    println!("Result: {:?}", result.await);
}
