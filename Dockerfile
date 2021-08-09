# Example for a slim/fat container setup.

FROM rust:1.46.0 as cntr
RUN rustup target add x86_64-unknown-linux-musl
RUN curl -sL https://github.com/Mic92/docker-pid/releases/download/1.0.0/docker-pid-linux-amd64 \
      > /usr/bin/docker-pid && \
      chmod 755 /usr/bin/docker-pid
COPY Cargo.toml Cargo.lock ./
# weird trick to cache crates
RUN cargo build --release --target=x86_64-unknown-linux-musl || true
COPY src ./src/
RUN cargo build --release --target=x86_64-unknown-linux-musl
RUN strip target/x86_64-unknown-linux-musl/release/cntr -o /usr/bin/cntr

# Added containerd binaries
RUN wget https://github.com/containerd/containerd/releases/download/v1.4.6/containerd-1.4.6-linux-amd64.tar.gz \
&& tar -xvf containerd-1.4.6-linux-amd64.tar.gz 


FROM ubuntu:groovy
WORKDIR /root/
COPY --from=cntr /usr/bin/cntr /usr/bin/cntr
COPY --from=cntr /usr/bin/docker-pid /usr/bin/docker-pid
COPY --from=cntr /bin/ctr /usr/bin/ctr
ENTRYPOINT ["/usr/bin/cntr"]

# Build with:
# $ docker build . -t cntr
# Assuming you have a container called mycontainer, you want to attach to (docker run --name mycontainer -ti --rm busybox sh)
# you can then run:
# $ sudo docker run --pid=host --privileged=true -v /var/run/docker.sock:/var/run/docker.sock -ti --rm cntr attach mycontainer /bin/sh