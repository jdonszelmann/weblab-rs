IMAGE_NAME := weblab/cese4000-2022-rust
DOCKERFILE_PATH := Dockerfile

MAKEFILE := $(abspath $(lastword $(MAKEFILE_LIST)))
SELF_DIR := $(dir $(MAKEFILE))
CMD := /bin/bash
BUILD_PATH := $(SELF_DIR)
BUILD_ARGS += "--no-cache"	# Crate downloaded from crates.io

all: build

include $(SELF_DIR)./Makefile.inc
