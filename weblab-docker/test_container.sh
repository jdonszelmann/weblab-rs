#!/usr/bin/env bash

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

docker build -t jonay2000/weblab-rs $SCRIPT_DIR

docker run -it \
  -v $SCRIPT_DIR/test/output:/user_code/output \
  -v $SCRIPT_DIR/test/library.txt:/user_code/library.txt \
  -v $SCRIPT_DIR/test/solution.txt:/user_code/solution.txt \
  -v $SCRIPT_DIR/test/test.txt:/user_code/test.txt \
  jonay2000/weblab-rs

