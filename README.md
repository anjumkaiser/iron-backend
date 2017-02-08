store-frontend - Baby steps towards a application backend

(c) Muhammad Anjum Kaiser <anjumbutt@gmail.com>




Generate a certificate:
openssl req -x509 -newkey rsa:4096 -nodes -keyout localhost.key -out localhost.crt -days 3650
openssl pkcs12 -export -out identity.p12 -inkey localhost.key -in localhost.crt