FROM rust:1.58.1-slim as builder

WORKDIR /app

ADD . /app/

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release

FROM gcr.io/distroless/cc
COPY --from=builder /app/target/release/birthdaybot  /usr/local/bin/bot
ENTRYPOINT [ "/usr/local/bin/bot" ]