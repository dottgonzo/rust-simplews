openssl genpkey -algorithm RSA -out ca.key
openssl genpkey -algorithm RSA -out server.key

openssl req -new -x509 -days 3650 -key ca.key -out ca.crt -extensions v3_ca


openssl req -newkey rsa:2048 -nodes -keyout server.key -out server.csr -config openssl.cnf -extensions v3_ca


# openssl x509 -req -days 365 -in server.csr -signkey server.key -out server.crt -extensions v3_ca -extfile openssl.cnf


openssl x509 -req -in server.csr -CA ca.crt -CAkey ca.key -CAcreateserial -out server.crt -days 3650 -extensions v3_ca 

###################

openssl req -x509 -newkey rsa:4096 -days 3650 -keyout ca_key.pem -out ca_cert.pem -subj "/CN=MyCA" -nodes
openssl genpkey -algorithm RSA -out server_key.pem
openssl req -new -key server_key.pem -out server_csr.pem -subj "/CN=localhost"
openssl x509 -req -in server_csr.pem -CA ca_cert.pem -CAkey ca_key.pem -CAcreateserial -out server_cert.pem -days 3650 -extfile <(echo -e "subjectAltName=DNS:localhost\nkeyUsage=digitalSignature,keyEncipherment\nextendedKeyUsage=serverAuth")
