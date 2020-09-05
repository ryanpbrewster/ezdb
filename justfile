build:
  #!/usr/bin/env bash
  set -eux
  docker build -t ezdb-build .
  cid=$(docker create ezdb-build)
  docker cp $cid:/home/builder/workspace/target/release/ezdb-server ./target/release/ezdb-server
  docker rm $cid
