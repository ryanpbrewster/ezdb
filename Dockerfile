FROM centos:latest
RUN yum group install "Development Tools" -y
RUN adduser builder

USER builder
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

ENV PATH=$PATH:/home/builder/.cargo/bin USER=builder

RUN mkdir -p /home/builder/workspace
WORKDIR /home/builder/workspace
RUN cargo init --name=precompile-deps
COPY Cargo.toml Cargo.lock ./
RUN mkdir -p src/bin && mv src/main.rs src/bin/ezdb-server.rs
RUN cargo build --release

# Compile the actual binary
COPY . .
RUN cargo build --release
