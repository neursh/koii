FROM rust:1.93.1

WORKDIR /source
COPY . .

RUN cargo build --release
RUN mv target/release/koii-auth /backend

WORKDIR /backend

