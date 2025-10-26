FROM --platform=$BUILDPLATFORM ubuntu:noble AS builder
ARG TARGETPLATFORM
ARG BUILDPLATFORM

RUN apt-get update && apt-get install -y curl build-essential gcc-arm-linux-gnueabihf gcc-aarch64-linux-gnu

ENV PATH="$PATH:/root/.cargo/bin" 
ENV CARGO_TARGET_DIR=/output
ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=/usr/bin/aarch64-linux-gnu-gcc
ENV CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER=/usr/bin/arm-linux-gnueabihf-gcc

WORKDIR /wrangler

COPY ./wrangler ./
COPY ./build.sh ./
COPY ./setup.sh ./

RUN ./setup.sh

RUN ./build.sh

FROM --platform=$TARGETPLATFORM gcr.io/distroless/cc-debian12 AS child_wrangler

WORKDIR /wrangler

COPY --from=builder /output/release/child_wrangler /wrangler/
COPY --from=builder /wrangler/target/site  /wrangler/target/site

ENTRYPOINT ["/wrangler/child_wrangler"]

