FROM rust:1.39
COPY . .

RUN cargo install --path .

# CMD ["tezos-node-bootstrap"]
