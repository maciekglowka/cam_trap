FROM rust:latest

RUN dpkg --add-architecture armhf
RUN rustup target add armv7-unknown-linux-gnueabihf
RUN apt-get update && apt-get install gcc-arm-linux-gnueabihf -yy
RUN apt-get update && apt-get install libclang-dev build-essential libc6-dev:armhf -yy
# RUN export CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER=/usr/bin/arm-linux-gnueabihf-gcc

WORKDIR /app
CMD ["cargo", "build", "--target=armv7-unknown-linux-gnueabihf", "--release"]