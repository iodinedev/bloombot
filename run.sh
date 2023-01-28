#!/bin/bash

PROJECT_NAME=$(basename `pwd`)
docker build . -t "$PROJECT_NAME" &&

echo "
" &&

docker run \
  --rm \
  --name "$PROJECT_NAME" \
  "$PROJECT_NAME"