FROM ricaps/gstreamer-rust:latest

ARG BUILD_DIRECTORY=/usr/src/erotic-hub
ARG RUN_DIRECTORY=/usr/erotic-hub
ARG INIT_DATA_MOUNT=/usr/init-data

ENV SQLX_OFFLINE=true
WORKDIR ${BUILD_DIRECTORY}

COPY . .
RUN cargo build --release --verbose

RUN mkdir ${RUN_DIRECTORY}
RUN mkdir -p ${INIT_DATA_MOUNT}
RUN cp target/release/erotic-hub ${RUN_DIRECTORY}

WORKDIR ${RUN_DIRECTORY}
COPY docker/erotic-hub/run.sh ./
COPY config.yaml ./
COPY migrations ./migrations
COPY static ./static

RUN rm -rf ${BUILD_DIRECTORY}

RUN chmod +x run.sh
CMD ["/usr/erotic-hub/run.sh"]