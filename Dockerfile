FROM rust:latest as wrk-build

RUN apt-get update && apt-get install -y libssl-dev git zlib1g-dev
RUN git clone https://github.com/wg/wrk.git && cd wrk && make && cp wrk /usr/local/bin/wrk
RUN git clone https://github.com/giltene/wrk2.git && cd wrk2 && make && cp wrk /usr/local/bin/wrk2



FROM rust:latest as this-build

COPY . .
RUN cargo install --path=. --root=/usr/local

FROM rust:latest

COPY --from=wrk-build /wrk/wrk /usr/local/bin/wrk
COPY --from=wrk-build /wrk2/wrk /usr/local/bin/wrk2
COPY --from=this-build /usr/local/bin/tezos-node-bootstrap /usr/local/bin/
COPY --from=this-build /scripts /scripts

CMD ["tezos-node-bootstrap"]
