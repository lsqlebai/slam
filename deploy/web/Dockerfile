FROM alpine:3.19 AS fetcher
RUN apk add --no-cache ca-certificates curl jq tar
ARG GITHUB_REPO="lsqlebai/slam"
ARG ASSET_PATTERN="slam_web-.*\\.tar\\.gz"
ENV GH_ACCEPT="application/vnd.github+json"
WORKDIR /tmp
RUN set -eux; \
    RELEASE_JSON="$(curl -s -H "Accept: ${GH_ACCEPT}" -H "User-Agent: docker-build" "https://api.github.com/repos/${GITHUB_REPO}/releases/latest")"; \
    ASSET_URL="$(echo "${RELEASE_JSON}" | jq -r --arg pattern "${ASSET_PATTERN}" '.assets[] | select(.name | test($pattern)) | .browser_download_url' | head -n 1)"; \
    test -n "${ASSET_URL}"; \
    echo "Downloading: ${ASSET_URL}"; \
    curl -L "${ASSET_URL}" -o web.tar.gz; \
    mkdir -p /tmp/webdist; \
    tar -xzf web.tar.gz -C /tmp/webdist

FROM nginx:1.25-alpine
COPY deploy/config/nginx.conf /etc/nginx/nginx.conf
RUN mkdir -p /usr/share/nginx/slam_web/dist
COPY --from=fetcher /tmp/webdist/dist/ /usr/share/nginx/slam_web/dist/
EXPOSE 80
