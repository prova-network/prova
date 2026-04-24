# SPDX-License-Identifier: MIT
# Multi-stage Dockerfile for provad.
#
# Build stage: compile statically-linkable binary under a well-defined Go toolchain.
# Runtime stage: minimal distroless base with the binary, an unprivileged user,
# and only the CA certs we need for outbound HTTPS calls to clients.

# ── Build stage ────────────────────────────────────────────────────────────
FROM golang:1.25-alpine AS build

# Install git for go modules that need VCS info
RUN apk add --no-cache git ca-certificates

WORKDIR /src

# Cache module fetches separately from the source build.
COPY go.mod go.sum ./
RUN go mod download

# Copy the rest of the source.
COPY . .

# Build arguments for embedding version + commit.
ARG VERSION=dev
ARG COMMIT=unknown

# Build statically; CGO disabled so the output works on distroless/scratch.
RUN CGO_ENABLED=0 GOOS=linux go build \
    -trimpath \
    -ldflags "-s -w -X main.version=${VERSION} -X main.commit=${COMMIT}" \
    -o /out/provad \
    ./cmd/provad

# ── Runtime stage ──────────────────────────────────────────────────────────
FROM gcr.io/distroless/static:nonroot

COPY --from=build /out/provad /usr/local/bin/provad
COPY --from=build /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt

# Default TOML path; override via --config or a volume mount.
USER nonroot:nonroot
ENTRYPOINT ["/usr/local/bin/provad"]
CMD ["--config", "/etc/prova/prover.toml", "start"]

# Document exposed ports; operator maps at runtime.
#   8443 = retrieval HTTP(S)
#   9090 = Prometheus metrics (bind 127.0.0.1 in practice)
EXPOSE 8443 9090

LABEL org.opencontainers.image.title="prova-prover"
LABEL org.opencontainers.image.description="Prova Network prover daemon"
LABEL org.opencontainers.image.source="https://github.com/prova-network/prova"
LABEL org.opencontainers.image.licenses="MIT"
