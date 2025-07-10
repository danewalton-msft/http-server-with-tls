use http_body_util::Full;
use hyper::body::Bytes;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto::Builder;
use rustls_pki_types::{CertificateDer, PrivateKeyDer};
use std::convert::Infallible;
use std::fs;
use std::sync::Arc;
use tokio_rustls::rustls::server::WebPkiClientVerifier;
use tokio_rustls::rustls::{RootCertStore, ServerConfig};
use tokio_rustls::TlsAcceptor;

async fn handle(req: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    println!("Received request: {} {}", req.method(), req.uri());
    println!("Headers: {:?}", req.headers());
    Ok(Response::new(Full::new(Bytes::from(
        r#"{"value":"rust-mtls-data"}"#,
    ))))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("Starting mTLS server...");

    let certs = load_certs("/certs/server-v3.crt")?;
    let key = load_private_key("/certs/server-v3.key")?;
    let client_ca = load_certs("/certs/ca-v3.crt")?;

    println!("Loaded {} server certificates", certs.len());
    println!("Loaded server private key");
    println!("Loaded {} client CA certificates", client_ca.len());

    let mut client_root_store = RootCertStore::empty();
    for cert in client_ca {
        client_root_store.add(cert)?;
    }

    let client_verifier = WebPkiClientVerifier::builder(Arc::new(client_root_store)).build()?;

    let config = ServerConfig::builder()
        .with_client_cert_verifier(client_verifier)
        .with_single_cert(certs, key)?;

    let acceptor = TlsAcceptor::from(Arc::new(config));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8443").await?;

    println!("Listening on https://0.0.0.0:8443 with mTLS");

    loop {
        let (stream, addr) = listener.accept().await?;
        println!("Accepted connection from: {}", addr);
        let acceptor = acceptor.clone();

        tokio::spawn(async move {
            let tls_stream = match acceptor.accept(stream).await {
                Ok(tls_stream) => {
                    println!("TLS handshake successful for {}", addr);
                    tls_stream
                }
                Err(err) => {
                    eprintln!("Failed to perform TLS handshake for {}: {:?}", addr, err);
                    return;
                }
            };

            let service = service_fn(handle);
            let builder = Builder::new(TokioExecutor::new());
            let conn = builder.serve_connection(TokioIo::new(tls_stream), service);

            if let Err(err) = conn.await {
                eprintln!("Failed to serve connection for {}: {:?}", addr, err);
            } else {
                println!("Connection served successfully for {}", addr);
            }
        });
    }
}

fn load_certs(
    path: &str,
) -> Result<Vec<CertificateDer<'static>>, Box<dyn std::error::Error + Send + Sync>> {
    let certfile = fs::read(path)?;
    let certs = rustls_pemfile::certs(&mut &*certfile).collect::<Result<Vec<_>, _>>()?;
    Ok(certs)
}

fn load_private_key(
    path: &str,
) -> Result<PrivateKeyDer<'static>, Box<dyn std::error::Error + Send + Sync>> {
    let keyfile = fs::read(path)?;
    let mut keys =
        rustls_pemfile::pkcs8_private_keys(&mut &*keyfile).collect::<Result<Vec<_>, _>>()?;
    if keys.is_empty() {
        return Err(format!("no keys found in {}", path).into());
    }
    Ok(PrivateKeyDer::Pkcs8(keys.remove(0)))
}
