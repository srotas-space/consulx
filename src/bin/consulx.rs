use consulx::{start_repl, ConsulXClient};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let addr = std::env::var("CONSUL_HTTP_ADDR")
        .unwrap_or_else(|_| "http://127.0.0.1:8500".to_string());

    println!("Connecting to Consul at: {}", addr);

    let client = ConsulXClient::new(&addr)?;
    start_repl(client).await?;

    Ok(())
}
