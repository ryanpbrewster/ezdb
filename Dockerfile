FROM rust:1.46
WORKDIR /rpb/build
RUN cargo build --release
