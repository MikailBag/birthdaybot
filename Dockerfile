FROM ekidd/rust-musl-builder

WORKDIR /app

ADD --chown=rust:rust Cargo.toml Cargo.lock /app/
RUN mkdir src && echo 'fn main(){}' > src/main.rs
RUN cargo build --release
ADD --chown=rust:rust src /app/src

RUN cargo build --release