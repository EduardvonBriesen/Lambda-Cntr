# Example for a slim/fat container setup.

FROM rust:1.46.0 as cntr
RUN rustup target add x86_64-unknown-linux-musl

# Add containerd binaries
RUN wget https://github.com/containerd/containerd/releases/download/v1.4.6/containerd-1.4.6-linux-amd64.tar.gz \
      && tar -xvf containerd-1.4.6-linux-amd64.tar.gz 

# Add docker-pid binary
RUN curl -sL https://github.com/Mic92/docker-pid/releases/download/1.0.0/docker-pid-linux-amd64 \
      > /usr/bin/docker-pid && \
      chmod 755 /usr/bin/docker-pid

# Add cntr binary
RUN wget https://github.com/Mic92/cntr/releases/download/1.5.1/cntr-src-1.5.1.tar.gz && \
      tar -xvf cntr-src-1.5.1.tar.gz
WORKDIR  /cntr-src-1.5.1
RUN cargo build --release --target=x86_64-unknown-linux-musl || true 
RUN strip target/x86_64-unknown-linux-musl/release/cntr -o /usr/bin/cntr
RUN cargo install cntr

FROM ubuntu:groovy
WORKDIR /root/
COPY --from=cntr /usr/bin/cntr /usr/bin/cntr
COPY --from=cntr /usr/bin/docker-pid /usr/bin/docker-pid
COPY --from=cntr /bin/ctr /usr/bin/ctr
ENTRYPOINT ["/usr/bin/cntr"]
