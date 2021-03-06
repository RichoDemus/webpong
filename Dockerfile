FROM rust as build

# todo make slimmer server builds without all these libs
RUN apt-get update && apt-get -y install libudev-dev zlib1g-dev libasound2-dev && rm -rf /var/lib/apt/lists/*

COPY ./ ./

#RUN rustup target install x86_64-unknown-linux-musl
#RUN CC_x86_64_unknown_linux_musl="x86_64-linux-musl-gcc" cargo build --release --target=x86_64-unknown-linux-musl
RUN cargo build --release #--target=x86_64-unknown-linux-musl

RUN mkdir -p /build-out

RUN cp target/release/webpong /build-out/

# Ubuntu 18.04
FROM debian
# todo make slimmer server builds without all these libs
RUN apt-get update && apt-get -y install libssl-dev && rm -rf /var/lib/apt/lists/*

#ENV DEBIAN_FRONTEND=noninteractive
#RUN apt-get update && apt-get -y install ca-certificates libssl-dev && rm -rf /var/lib/apt/lists/*
#RUN apt-get update && apt-get -y install curl && rm -rf /var/lib/apt/lists/*
COPY --from=build /build-out/webpong /rust

EXPOSE 8080
CMD /rust --server
# ENTRYPOINT ["/rust"]
