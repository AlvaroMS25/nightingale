FROM docker.io/rust:1-bullseye AS build

RUN apt-get update && apt-get install -y \
    cmake \
    build-essential \
    libssl-dev \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

# armv7 has errors cloning under qemu, this makes it use git executable instead
ENV CARGO_NET_GIT_FETCH_WITH_CLI=true

WORKDIR /nightingale
COPY . .
RUN cargo b --release --features docker

FROM debian:bullseye-slim AS runtime
WORKDIR /nightingale

# Install certificates, yt-dlp and dependencies
RUN apt-get update && apt-get install -y \
    wget \
    python3 \
    python3-pip \
    ca-certificates \
    && pip3 install --no-cache-dir yt-dlp \
    && rm -rf /var/lib/apt/lists/*

COPY --from=build /nightingale/target/release/nightingale .

ENTRYPOINT ["./nightingale"]

EXPOSE 8081/tcp