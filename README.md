# Rust mTLS REST Server

A simple HTTPS server in Rust with x.509 client certificate (mutual TLS) authentication.

## Endpoints

- `GET /data` → returns JSON response after mTLS handshake.

## Generate Certificates

To generate the necessary certificates for mTLS, you can use the `./k8s/generate_certs.sh` script. This script will create a self-signed CA certificate, a server certificate signed by the CA, and a client certificate also signed by the CA.

Run the script with:
```bash
./k8s/generate-v3-certs.sh
```

## Build Docker Image

To build the Docker image, run:
```
docker build -t rust-mtls-server .
```

Tag with ACR
```
docker tag rust-mtls-server <your_acr_name>.azurecr.io/rust-mtls-server:latest
```

To push the image to Azure Container Registry, run:
```
docker push <your_acr_name>.azurecr.io/rust-mtls-server:latest
```

