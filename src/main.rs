use http_body_util::Full;
use hyper::body::Bytes;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto::Builder;
use rand::Rng;
use std::convert::Infallible;

fn get_current_timestamp() -> String {
    chrono::Utc::now()
        .format("%Y-%m-%dT%H:%M:%S%.3fZ")
        .to_string()
}

async fn handle(req: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    println!("Received request: {} {}", req.method(), req.uri());
    println!("Headers: {:?}", req.headers());
    let mut rand_handle = rand::rng();
    let temp_value = rand_handle.random_range(20..100);
    let current_time = get_current_timestamp();
    let payload = format!(
        "{{\"temperature\":{{\"Value\":{temp_value},\"SourceTimestamp\":\"{current_time}\"}}}}"
    );
    Ok(Response::new(Full::new(Bytes::from(payload))))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("Starting HTTP server...");

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;

    println!("Listening on http://0.0.0.0:8080");

    loop {
        let (stream, addr) = listener.accept().await?;
        println!("Accepted connection from: {}", addr);

        tokio::spawn(async move {
            let service = service_fn(handle);
            let builder = Builder::new(TokioExecutor::new());
            let conn = builder.serve_connection(TokioIo::new(stream), service);

            if let Err(err) = conn.await {
                eprintln!("Failed to serve connection for {}: {:?}", addr, err);
            } else {
                println!("Connection served successfully for {}", addr);
            }
        });
    }
}
