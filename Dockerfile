FROM clux/muslrust:stable as builder

COPY src/ src/
COPY Cargo.lock Cargo.toml ./

RUN cargo install --locked --path .

FROM alpine:3.12

WORKDIR /data

COPY config/ /data/config/
COPY --from=builder /root/.cargo/bin/aoc_bot /bin/

STOPSIGNAL SIGINT

ENTRYPOINT ["/bin/aoc_bot"]
