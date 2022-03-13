FROM rust:1-slim-buster
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
 && addgroup -g ${GID} "${GNAME}" \
 && adduser -h "${UHOME}" -D -u ${UID} --ingroup "${GNAME}" "${UNAME}"

# Copy user code
USER ${UNAME}
COPY --chown=${UNAME} user_code/ ${UHOME}
WORKDIR ${UHOME}

# Remove write permission on truncate_output.sh
RUN chmod u-w truncate_output.sh

# Run
ENV HOME ${UHOME}
ENV CK_XML_LOG_FILE_NAME=results.xml
RUN mkdir ${UHOME}/output
CMD ( { \
    mv solution.txt solution.rs       2>/dev/null; true \
 && mv library.txt  library.rs        2>/dev/null; true \
 && mv test.txt     test.rs           2>/dev/null; true \
 && cargo run --offline \
 && cd output \
 && ./../run.sh \
 ; } | tee output/stdout.txt ) 3>&1 1>&2 2>&3 | tee output/stderr.txt \
 && ./truncate_output.sh




