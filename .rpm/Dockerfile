FROM centos:latest

ENV PATH "$PATH:/root/.cargo/bin"
RUN yum -y update && \
    yum -y install gcc make cmake libX11-devel rpm-build rpmdevtools && \
    curl https://sh.rustup.rs -sSf | sh -s -- -y && \
    cargo install cargo-rpm
