FROM rust:1.94.1

WORKDIR /source
COPY . .

RUN cargo build --release
RUN mv target/release/koii /backend

WORKDIR /backend

