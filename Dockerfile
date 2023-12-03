FROM rust:1.74 as builder

WORKDIR /volume

RUN apt-get update && \
    apt-get install -y --no-install-recommends musl-tools=1.2.3-1 && \
    rustup target add x86_64-unknown-linux-musl

COPY src/ src/
COPY Cargo.lock Cargo.toml ./

RUN cargo build --release --target x86_64-unknown-linux-musl && \
    strip --strip-all target/x86_64-unknown-linux-musl/release/aoc_bot

FROM alpine:3.14 as newuser

RUN echo "aoc_bot:x:1000:" > /tmp/group && \
    echo "aoc_bot:x:1000:1000::/dev/null:/sbin/nologin" > /tmp/passwd

FROM scratch

WORKDIR /data

COPY --from=builder /volume/target/x86_64-unknown-linux-musl/release/aoc_bot /bin/
COPY --from=newuser /tmp/group /tmp/passwd /etc/

STOPSIGNAL SIGINT
USER aoc_bot

ENTRYPOINT ["/bin/aoc_bot"]
