#/bin/bash
set -e


#https://stackoverflow.com/a/246128
DOCKER_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"

docker build --build-arg USER=$(id -u) -t emuka-linux -f $DOCKER_DIR/linux.Dockerfile $DOCKER_DIR/..

docker run --user $(id -u):$(id -g) -ti -v $DOCKER_DIR/out/linux:/out emuka-linux

