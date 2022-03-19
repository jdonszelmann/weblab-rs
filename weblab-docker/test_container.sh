#!/usr/bin/env bash

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

docker build -t jonay2000/weblab-rs $SCRIPT_DIR

docker run -it \
  -v $SCRIPT_DIR/test/output:output \
  -v $SCRIPT_DIR/test/library.txt:library.txt \
  -v $SCRIPT_DIR/test/solution.txt:solution.txt \
  -v $SCRIPT_DIR/test/test.txt:test.txt \
  jonay2000/weblab-rs

