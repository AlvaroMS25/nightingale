FROM docker.io/rust:1-bookworm AS build

RUN apt-get update && apt-get install -y \
    cmake \
    build-essential \
    libssl-dev \
    pkg-config \
    libopus-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /nightingale
COPY . .
RUN cargo b --release --features docker

FROM debian:bookworm-slim AS runtime
WORKDIR /nightingale

# Install certificates, yt-dlp and dependencies
RUN apt-get update && apt-get install -y \
    wget \
    python3 \
    python3-pip \
    ca-certificates \
    && pip3 install --break-system-packages --no-cache-dir yt-dlp \
    && rm -rf /var/lib/apt/lists/*

COPY --from=build /nightingale/target/release/nightingale .

ENTRYPOINT ["./nightingale"]

EXPOSE 8081/tcp