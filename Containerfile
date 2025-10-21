FROM --platform=$BUILDPLATFORM ubuntu:noble AS builder
ARG TARGETPLATFORM
ARG BUILDPLATFORM

COPY ./build.sh ./
COPY ./setup.sh ./

RUN apt-get update && apt-get install -y curl build-essential

ENV PATH="$PATH:/root/.cargo/bin" 

RUN ./setup.sh
RUN cargo test
RUN ./build.sh


COPY ./wrangler /wrangler
WORKDIR /wrangler


FROM --platform=$TARGETPLATFORM gcr.io/distroless/cc-debian12 AS child_wrangler

WORKDIR /wrangler

COPY --from=builder /wrangler/target/armv7-unknown-linux-gnueabihf/release/child_wrangler /wrangler/
COPY --from=builder /wrangler/target/site  /wrangler/target/site

ENTRYPOINT ["/wrangler/child_wrangler"]

FROM --platform=$TARGETPLATFORM gcr.io/distroless/cc-debian12 AS message_daemon

COPY --from=builder /wrangler/target/armv7-unknown-linux-gnueabihf/release/message_daemon /wrangler/

ENTRYPOINT ["/wrangler/message_daemon"]

