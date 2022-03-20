FROM rust:1.59-buster
WORKDIR /home/

# install packages
RUN apt-get update && apt-get install -y \
  perl \
  && rm -rf /var/lib/apt/lists/*

# Switch to bash
SHELL ["/bin/bash", "-c"]

# Create student user
ARG GNAME=student
ARG GID=1000
ARG UNAME=student
ARG UID=1000
ARG UHOME=/user_code
RUN set -o errexit -o nounset \
 && addgroup --gid ${GID} "${GNAME}" \
 && adduser --home "${UHOME}" --disabled-password --gecos "" --uid ${UID} --ingroup "${GNAME}" "${UNAME}"

# Copy user code
USER ${UNAME}
COPY --chown=${UNAME} user_code/ ${UHOME}
WORKDIR ${UHOME}

# prebuild dependencies
RUN mv src/main.rs src/actual_main.rs && \
    mv src/fake_main.rs src/main.rs &&  \
    cargo fetch && cargo build && cargo install cargo-weblabtest && \
    mv src/actual_main.rs src/main.rs

# Remove write permission on truncate_output.sh
RUN chmod u-w+x truncate_output.sh

# Run
ENV HOME ${UHOME}
RUN mkdir ${UHOME}/output
CMD cp solution.txt src/solution.rs       2>/dev/null; true \
 && cp library.txt  src/library.rs        2>/dev/null; true \
 && cp test.txt     src/test.rs           2>/dev/null; true \
 && echo "running tests, writing to ${HOME}/output" \
 && cargo weblabtest ${HOME}/output \
 && ls output && pwd \
 && ./truncate_output.sh




