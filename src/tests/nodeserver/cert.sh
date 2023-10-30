################### this works for localhost ###################

openssl req -x509 -newkey rsa:4096 -days 36500 -keyout ca_key.pem -out ca_cert.pem -subj "/CN=MyCA" -nodes
openssl genpkey -algorithm RSA -out server_key.pem
openssl req -new -key server_key.pem -out server_csr.pem -subj "/CN=localhost"
openssl x509 -req -in server_csr.pem -CA ca_cert.pem -CAkey ca_key.pem -CAcreateserial -out server_cert.pem -days 36500 -extfile <(echo -e "subjectAltName=DNS:localhost\nkeyUsage=digitalSignature,keyEncipherment\nextendedKeyUsage=serverAuth")
