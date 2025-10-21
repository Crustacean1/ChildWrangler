FROM --platform=$BUILDPLATFORM ubuntu:noble AS builder
ARG TARGETPLATFORM
ARG BUILDPLATFORM

RUN apt-get update && apt-get install -y curl build-essential

ENV PATH="$PATH:/root/.cargo/bin" 

WORKDIR /wrangler

COPY ./wrangler ./
COPY ./build.sh ./
COPY ./setup.sh ./


RUN ./setup.sh
RUN cargo test
ENV CARGO_TARGET_DIR=/output
RUN ./build.sh

FROM --platform=$TARGETPLATFORM gcr.io/distroless/cc-debian12 AS child_wrangler

WORKDIR /wrangler

COPY --from=builder /output/release/child_wrangler /wrangler/
COPY --from=builder /output  /wrangler/target/site

ENTRYPOINT ["/wrangler/child_wrangler"]

FROM --platform=$TARGETPLATFORM gcr.io/distroless/cc-debian12 AS message_daemon

COPY --from=builder /wrangler/target/armv7-unknown-linux-gnueabihf/release/message_daemon /wrangler/

ENTRYPOINT ["/wrangler/message_daemon"]

