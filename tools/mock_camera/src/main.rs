use clap::Parser;
use tokio::net::TcpListener;

mod fixtures;
mod protocol;

#[derive(Debug, Parser)]
#[command(name = "mock-camera")]
#[command(about = "Mock Nikon PTP/IP camera server scaffold")]
struct Args {
    #[arg(long, default_value = "127.0.0.1")]
    host: String,
    #[arg(long, default_value_t = 15740)]
    port: u16,
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let args = Args::parse();
    let addr = format!("{}:{}", args.host, args.port);
    let listener = TcpListener::bind(&addr).await?;
    println!("mock-camera listening on {addr}");

    loop {
        let (stream, peer) = listener.accept().await?;
        println!("accepted connection from {peer}");
        tokio::spawn(async move {
            if let Err(error) = protocol::handle_connection(stream).await {
                eprintln!("mock connection error: {error}");
            }
        });
    }
}
