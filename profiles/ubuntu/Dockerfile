FROM ubuntu:26.04

RUN apt-get update \
    && apt-get upgrade --yes \
    && apt-get autoremove --yes \
    && apt-get install --yes --no-install-recommends util-linux \
    && rm -rf /var/lib/apt/lists/*
