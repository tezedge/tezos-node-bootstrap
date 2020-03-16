FROM rust:1.39

RUN apt-get update && apt-get install -y libssl-dev git zlib1g-dev
# RUN git clone https://github.com/giltene/wrk2.git && cd wrk2 && make && cp wrk /usr/local/bin
RUN git clone https://github.com/wg/wrk.git && cd wrk && make && cp wrk /usr/local/bin


COPY . .
RUN ls -la
RUN pwd
RUN cargo install --path .

# CMD ["tezos-node-bootstrap"]
