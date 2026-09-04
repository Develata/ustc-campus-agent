FROM ubuntu:24.04

ARG UCA_SOURCE_COMMIT=unknown
LABEL org.opencontainers.image.title="USTC Campus Agent three-plugin MVP" \
      org.opencontainers.image.source="https://github.com/Develata/ustc-campus-agent" \
      org.opencontainers.image.revision="${UCA_SOURCE_COMMIT}"

RUN apt-get update \
    && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
       ca-certificates curl socat util-linux \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --chmod=0755 bin/ustc-agentd /app/ustc-agentd
COPY --chmod=0755 container-entrypoint.sh /app/container-entrypoint.sh
COPY fixtures /app/fixtures
COPY market /app/market

RUN install -d -o 65532 -g 65532 -m 0700 /data

EXPOSE 8787
HEALTHCHECK --interval=3s --timeout=2s --start-period=15s --retries=20 \
  CMD setpriv --reuid=65532 --regid=65532 --clear-groups --no-new-privs \
      curl --fail --silent --show-error http://127.0.0.1:8787/healthz >/dev/null || exit 1

ENTRYPOINT ["/app/container-entrypoint.sh"]
