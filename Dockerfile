FROM rust:bookworm
RUN apt update && apt install -y build-essential cmake make llvm clang pandoc

COPY . /opt/testapi
WORKDIR /opt/testapi


RUN make all
CMD ["/opt/testapi/target/release/test-api", "-b", "http://0.0.0.0:80"]