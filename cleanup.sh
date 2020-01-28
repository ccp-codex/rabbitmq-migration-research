#!/usr/bin/env bash

set -x

docker rm -f node-a node-b

docker network rm rabbit
