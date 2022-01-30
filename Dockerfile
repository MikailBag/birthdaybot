FROM rust:1.58.1-slim as builder
RUN apt-get update && apt-get install -y libssl-dev pkg-config

WORKDIR /app

ADD . /app/
RUN mkdir /out
ENV RUSTC_BOOTSTRAP=1
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release \
    -Zunstable-options --out-dir /out

FROM gcr.io/distroless/cc
COPY --from=builder /out/birthdaybot  /usr/local/bin/bot
ENTRYPOINT [ "/usr/local/bin/bot" ]