#!/usr/bin/env bash

function wait () {
  local node=$1

  while ! docker exec "$node" rabbitmqctl status &>/dev/null
  do
    echo "Waiting for $node"
    sleep 5s
  done
  echo "$node is running"
}

set -ex

docker network create rabbit

docker run -d --network rabbit --name "node-a" \
    -p "5001:5672" -p "15001:15672" \
    -e RABBITMQ_NODENAME="rabbit@node-a" \
    -e RABBITMQ_ERLANG_COOKIE="fed" \
    -v "$PWD"/rabbitmq.conf:/etc/rabbitmq/rabbitmq.conf \
    rabbitmq:3.8-alpine

# docker network create b
docker run -d --network rabbit --name "node-b" \
    -p "5002:5672" -p "15002:15672" \
    -e RABBITMQ_NODENAME="rabbit@node-b" \
    -e RABBITMQ_ERLANG_COOKIE="fed" \
    -v "$PWD"/rabbitmq.conf:/etc/rabbitmq/rabbitmq.conf \
    rabbitmq:3.8-alpine

# RabbitMQ need some time to startup
wait node-a
wait node-b

docker exec node-a rabbitmq-plugins enable rabbitmq_management
docker exec node-a rabbitmq-plugins enable rabbitmq_federation_management
docker exec node-a rabbitmqctl stop_app
docker exec node-a rabbitmqctl reset
docker exec node-a rabbitmqctl start_app
docker exec node-a rabbitmqctl add_vhost fed-test
docker exec node-a rabbitmqctl set_permissions -p fed-test guest ".*" ".*" ".*"
docker exec node-a rabbitmqctl set_parameter -p fed-test federation-upstream upstream-from-b '{"uri":"amqp://node-b/fed-test","expires":3600000}'
docker exec node-a rabbitmqctl set_policy -p fed-test --apply-to exchanges federate-on-a ".*" '{"federation-upstream-set":"all"}'

docker exec node-b rabbitmq-plugins enable rabbitmq_management
docker exec node-b rabbitmq-plugins enable rabbitmq_federation_management
docker exec node-b rabbitmqctl stop_app
docker exec node-b rabbitmqctl reset
docker exec node-b rabbitmqctl start_app
docker exec node-b rabbitmqctl add_vhost fed-test
docker exec node-b rabbitmqctl set_permissions -p fed-test guest ".*" ".*" ".*"
docker exec node-b rabbitmqctl set_parameter -p fed-test federation-upstream upstream-from-a '{"uri":"amqp://node-a/fed-test","expires":3600000}'
docker exec node-b rabbitmqctl set_policy -p fed-test --apply-to exchanges federate-on-b ".*" '{"federation-upstream-set":"all"}'
