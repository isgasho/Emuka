#/bin/bash
set -e

#https://stackoverflow.com/a/246128
DOCKER_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"

docker build --build-arg USER=$(id -u) -t emuka-windows -f $DOCKER_DIR/windows.Dockerfile $DOCKER_DIR/..

docker run --user $(id -u):$(id -g) -ti -v $DOCKER_DIR/out/windows:/out emuka-windows

