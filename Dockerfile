FROM rust:1.46 AS builder

RUN adduser builder
USER builder
ENV USER=builder
WORKDIR /home/builder

RUN cargo init --name=precompile-deps
COPY Cargo.toml Cargo.lock ./
RUN mkdir -p src/bin && mv src/main.rs src/bin/ezdb-server.rs
RUN cargo build --release

# Compile the actual binary
COPY . .
RUN cargo build --release
