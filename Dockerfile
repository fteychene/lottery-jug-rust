FROM rust:stretch AS builder

COPY . /workdir

RUN cd /workdir && cargo build --release


FROM ubuntu:latest

ENV DATABASE_URL "/var/data/lottery-jug/lottery.db"
ENV RUST_LOG "info"

VOLUME /var/data/lottery-jug/

RUN apt-get update && apt-get install -y libsqlite3-dev libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /workdir/target/release/lottery-presentation /lottery-presentation

CMD ["/lottery-presentation"]