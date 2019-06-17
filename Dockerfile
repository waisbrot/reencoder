FROM rust:1.34 as dependencies
WORKDIR /usr/src/app
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build

FROM rust:1.34 as build
RUN apt-get update && apt-get install -q -y ffmpeg
COPY --from=dependencies /usr/src/app/Cargo.toml /usr/src/app/Cargo.lock ./
COPY --from=dependencies /usr/local/cargo /usr/local/cargo
COPY . .
RUN cargo install --path .
ENV RUST_BACKTRACE=1 RUST_LOG=scan_to_postgres=info
ENTRYPOINT ["/usr/local/cargo/bin/scan-to-postgres"]
