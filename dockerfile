FROM rust:1.61 AS builder
COPY . .
RUN cargo build --release

FROM debian:buster-slim
COPY --from=builder ./target/release/rusty_tarot ./target/release/rusty_tarot
CMD ["/target/release/rusty_tarot"]