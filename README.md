store-frontend - Baby steps towards a application backend

(c) 2017 Muhammad Anjum Kaiser <anjumbutt@gmail.com>




Generate a certificate:
openssl req -x509 -newkey rsa:4096 -nodes -keyout localhost.key -out localhost.crt -days 3650
openssl pkcs12 -export -out identity.p12 -inkey localhost.key -in localhost.crt


Database setup
---------------

Postgresql:

Setup Location:
# mkdir -m 750 --context="system_u:object_r:postgresql_db_t:s0" /data/postgresql
# chattr +C /data/postgresql
# chown postgres:postgres /data/postgresql

Attach SELINUX context
# semanage fcontext -a -t postgresql_db_t "/data/postgresql(/.*)?"

Change to postgresql
# su - postgres

Create database owner in pgsql
postgresql=# CREATE USER store WITH PASSWORD 'store';

Create tablespace
postgresql=# create tablespace store owner store location '/data/postgresql/store';

Create Database
postgresql=# create database store owner store tablespace store;




Building on Windows

- Install Microsoft Visual Studio 2017 with VC++
- Install Win64 OpenSSL v1.1.0f from slproweb.com/products/Win32OpenSSL.html
- Add cacerts.pem to C:\OpenSSL-Win64\certs

- Install Redis

- Install Postgresql (9.6.3) Win64 binaries from EnterpriseDB [.zip file]
- Copy pgsql\lib\libpq.lib to pgsql\lib\pq.lib

- Add C:\pgsql\bin to PATH
- Make sure pg_config.exe command runs and displays output
- Add following entry to .cargo/config:

[target.x86_64-pc-windows-msvc.pq]
rustc-link-search = ["C:\\pgsql\\lib"]
rustc-link-lib = ["pq"]

- Clean previous builds
cargo clean

- Build:
cargo build