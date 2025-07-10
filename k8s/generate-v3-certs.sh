#!/bin/bash

# Generate proper X.509 v3 certificates for mTLS testing

set -eox

mkdir -p certs
cd certs

echo "Generating new X.509 v3 certificates..."

# Create a config file for CA v3 extensions
cat > ca-v3.ext << EOF
[v3_ca]
basicConstraints = critical,CA:true
keyUsage = critical, digitalSignature, cRLSign, keyCertSign
subjectKeyIdentifier = hash
authorityKeyIdentifier = keyid:always,issuer
EOF

# Create a config file for server v3 extensions
cat > server-v3.ext << EOF
[v3_req]
authorityKeyIdentifier=keyid,issuer
basicConstraints=CA:FALSE
keyUsage = digitalSignature, nonRepudiation, keyEncipherment, dataEncipherment
subjectAltName = @alt_names

[alt_names]
DNS.1 = localhost
DNS.2 = rest-server-service-x509
DNS.3 = rest-server-service-x509.default.svc.cluster.local
IP.1 = 127.0.0.1
IP.2 = ::1
EOF

# Generate new CA key and certificate (v3)
echo "1. Generating new CA..."
openssl genrsa -out ca-v3.key 4096
openssl req -new -x509 -days 3650 -key ca-v3.key -out ca-v3.crt -subj "/CN=Test Root CA v3" -extensions v3_ca -config ca-v3.ext

# Generate server key and certificate (v3)
echo "2. Generating new server certificate..."
openssl genrsa -out server-v3.key 2048
openssl req -new -key server-v3.key -out server-v3.csr -subj "/CN=localhost"
openssl x509 -req -in server-v3.csr -CA ca-v3.crt -CAkey ca-v3.key -CAcreateserial -out server-v3.crt -days 365 -extensions v3_req -extfile server-v3.ext

# Create a config file for client v3 extensions
cat > client-v3.ext << EOF
[v3_req]
authorityKeyIdentifier=keyid,issuer
basicConstraints=CA:FALSE
keyUsage = digitalSignature, nonRepudiation, keyEncipherment, dataEncipherment
EOF

# Generate client key and certificate (v3)
echo "3. Generating new client certificate..."
openssl genrsa -out client-v3.key 2048
openssl req -new -key client-v3.key -out client-v3.csr -subj "/CN=test-client-v3"
openssl x509 -req -in client-v3.csr -CA ca-v3.crt -CAkey ca-v3.key -CAcreateserial -out client-v3.crt -days 365 -extensions v3_req -extfile client-v3.ext

echo "4. Verifying new certificates..."
openssl verify -CAfile ca-v3.crt server-v3.crt
openssl verify -CAfile ca-v3.crt client-v3.crt

echo "5. Certificate details:"
echo "Server certificate:"
openssl x509 -in server-v3.crt -text -noout | grep -E "(Version|Subject:|Issuer:|DNS:|IP:)"

echo -e "\nClient certificate:"
openssl x509 -in client-v3.crt -text -noout | grep -E "(Version|Subject:|Issuer:)"

cat client-v3.crt client-v3.key > client-v3-combo.crt

echo -e "\nDone! Use the *-v3.* certificate files."
