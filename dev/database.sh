#!/bin/bash
identifier="#Koii's managed hosts"
hosts="127.0.0.1 mongo1 mongo2 mongo3"
networks=$(podman network ls)

if [[ "$networks" != *"koiiMongodbCluster"* ]]; then
  podman network create koiiMongodbCluster
fi

if [[ "$1" == "remove" ]]; then
  sudo sed -i "\#^${hosts}\$#d" "/etc/hosts"
  sudo -k
  podman exec -it mongo2 mongosh --eval 'db.shutdownServer({ force: true })'
  podman exec -it mongo3 mongosh --eval 'db.shutdownServer({ force: true })'
  podman exec -it mongo1 mongosh --eval 'db.shutdownServer({ force: true })'
  podman rm -f mongo1
  podman rm -f mongo2
  podman rm -f mongo3
  podman rm -f dragonfly1
  exit 0
fi

if [[ "$1" == "down" ]]; then
  sudo sed -i "\#^${hosts}\$#d" "/etc/hosts"
  sudo -k
  podman exec -it mongo1 mongosh --eval 'db.shutdownServer({ force: true })'
  podman exec -it mongo2 mongosh --eval 'db.shutdownServer({ force: true })'
  podman exec -it mongo3 mongosh --eval 'db.shutdownServer({ force: true })'
  podman stop mongo1
  podman stop mongo2
  podman stop mongo3
  podman stop dragonfly1
  exit 0
fi

sudo sed -i "\#^${hosts}\$#d" "/etc/hosts"
sudo -k

podman run -d -p 27017:27017 --name mongo1 --network koiiMongodbCluster mongo:8.0.4 mongod --replSet koiiReplicaSet --bind_ip localhost,mongo1
podman run -d -p 27018:27017 --name mongo2 --network koiiMongodbCluster mongo:8.0.4 mongod --replSet koiiReplicaSet --bind_ip localhost,mongo2
podman run -d -p 27019:27017 --name mongo3 --network koiiMongodbCluster mongo:8.0.4 mongod --replSet koiiReplicaSet --bind_ip localhost,mongo3
podman run -d -p 6379:6379 --name dragonfly1 --ulimit memlock=-1 docker.dragonflydb.io/dragonflydb/dragonfly

sleep 1

podman exec -it mongo1 mongosh --eval "rs.initiate({
  _id: \"koiiReplicaSet\",
  members: [
    {_id: 0, host: \"mongo1\"},
    {_id: 1, host: \"mongo2\"},
    {_id: 2, host: \"mongo3\"}
  ]
})"

sleep 1

podman exec -it mongo2 mongosh --eval "rs.status()"

if grep -qF "$hosts" "/etc/hosts"; then
  echo "Skipping hosts write."
elif grep -qF "$identifier" "/etc/hosts"; then
  sudo sed -i "/^${identifier}$/a ${hosts}" "/etc/hosts"
else
  echo "$identifier" | sudo tee -a /etc/hosts
  echo "$hosts" | sudo tee -a /etc/hosts
fi
