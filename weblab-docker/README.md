# WebLab Docker Image
This repository contains the Dockerfile for a WebLab docker image.

## Requirements
Requirements:
- GNU Tar (`brew install gnu-tar`)

## Quick Start

To build an image, go to the image directory and invoke:

```bash
make
```

To run an image's tests, invoke:

```bash
make test
```

The test results appear in the `testresults/` subdirectory.

See the `Makefile.inc` file for more commands.