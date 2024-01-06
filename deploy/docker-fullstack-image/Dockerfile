FROM ubuntu:22.04

# see https://github.com/moby/moby/issues/4032#issuecomment-192327844
ARG DEBIAN_FRONTEND=noninteractive

# ARG for quick switch to a given ubuntu mirror
ARG apt_archive="http://archive.ubuntu.com"
RUN sed -i "s|http://archive.ubuntu.com|${apt_archive}|g" /etc/apt/sources.list 

# USERS Setup

# user/group precreated explicitly with fixed uid/gid on purpose.
# It is especially important for rootless containers: in that case entrypoint
# can't do chown and owners of mounted volumes should be configured externally.
# We do that in advance at the begining of Dockerfile before any packages will be
# installed to prevent picking those uid / gid by some unrelated software.
# The same uid / gid (101) is used both for alpine and ubuntu.
RUN groupadd -r redpanda --gid=101 \
    && useradd -r -g redpanda --uid=101 --home-dir=/var/lib/redpanda --shell=/bin/bash redpanda

# Some common dependencies
RUN apt-get update \
    && apt-get upgrade -yq \
    && apt-get install --yes --no-install-recommends \
        ca-certificates \
        locales \
        tzdata \
        wget \
        curl \
        supervisor \
    && apt-get clean \
    && rm -rf \
        /var/lib/apt/lists/* \
        /var/cache/debconf \
        /tmp/*

RUN mkdir -p /var/log/supervisor

# Install Redpandas
RUN curl -1sLf 'https://dl.redpanda.com/nzc4ZYQK3WRGd9sy/redpanda/cfg/setup/bash.deb.sh' | bash && apt install redpanda -y

RUN curl -1sLf 'https://dl.redpanda.com/nzc4ZYQK3WRGd9sy/redpanda/cfg/setup/bash.deb.sh' | bash && apt install redpanda-console -y

RUN rpk redpanda mode production && \ 
    rpk redpanda config bootstrap --self 127.0.0.1 --ips 127.0.0.1 && \
    rpk redpanda config set redpanda.empty_seed_starts_cluster false

RUN mkdir -p /var/lib/redpanda/data

# Install Clickhouse

# Install Moose


# Run it all
COPY supervisord.conf /etc/supervisor/conf.d/supervisord.conf

CMD ["/usr/bin/supervisord"]