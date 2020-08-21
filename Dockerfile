FROM rust:1 as dependencies
WORKDIR /usr/src/app
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build
RUN cargo install --path .

FROM rust:1 as build
RUN apt-get update \
  && apt-get install -q -y --no-install-recommends ffmpeg \
  && rm -rf /var/lib/apt/lists/*
WORKDIR /usr/src/app
COPY --from=dependencies /usr/src/app/Cargo.toml /usr/src/app/Cargo.lock /usr/src/app/target ./
COPY --from=dependencies /usr/local/cargo /usr/local/cargo
COPY src ./src
RUN cargo build
RUN cargo install --path .
ENV RUST_BACKTRACE=1 RUST_LOG=video-processor=info
ENTRYPOINT ["/usr/local/cargo/bin/video-processor"]
