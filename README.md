```
docker login docker.io
docker-compose build
docker-compose push

docker build -t tezedge/tezos-node-bootstrap:latest . && docker push tezedge/tezos-node-bootstrap:latest
```