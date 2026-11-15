# syntax=docker/dockerfile:1
FROM eclipse-temurin:21-jre-jammy

LABEL maintainer="Kvbi213 <lisjakub2106@gmail.com>"
LABEL description="Qubit Headless Reverse Engineering Sandbox with Ghidra and MCP Server"

ENV GHIDRA_VERSION=11.0.1
ENV GHIDRA_RELEASE=ghidra_11.0.1_PUBLIC_20240130
ENV GHIDRA_HOME=/opt/ghidra
ENV PATH="${GHIDRA_HOME}/support:${PATH}"

RUN apt-get update && apt-get install -y --no-install-recommends \
    wget \
    unzip \
    python3 \
    python3-pip \
    binutils \
    file \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /opt

RUN wget -q https://github.com/NationalSecurityAgency/ghidra/releases/download/Ghidra_11.0.1_build/ghidra_11.0.1_PUBLIC_20240130.zip \
    && unzip -q ghidra_11.0.1_PUBLIC_20240130.zip \
    && mv ${GHIDRA_RELEASE} ghidra \
    && rm ghidra_11.0.1_PUBLIC_20240130.zip

WORKDIR /workspace

# Install python dependencies for MCP bridge
RUN pip3 install --no-cache-dir mcp

COPY ./scripts/ /opt/qubit-re/

ENTRYPOINT ["python3", "-m", "qubit_re_mcp"]
