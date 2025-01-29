FROM docker.io/rust:1-bullseye AS build

RUN apt-get update && apt-get install -y \
    cmake \
    build-essential \
    libssl-dev \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /nightingale
COPY . .
RUN cargo b --release

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

# assuming we leave port to default
EXPOSE 8081/tcp