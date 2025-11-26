FROM alpine:3.19 AS fetcher
RUN apk add --no-cache ca-certificates curl jq tar
ARG GITHUB_REPO="lsqlebai/slam"
ARG WEB_VERSION="latest"
ENV GH_ACCEPT="application/vnd.github+json"
WORKDIR /tmp
RUN set -eux; \
    if [ "${WEB_VERSION}" = "latest" ]; then API="https://api.github.com/repos/${GITHUB_REPO}/releases/latest"; PATTERN="slam_web-.*\\.tar\\.gz"; else API="https://api.github.com/repos/${GITHUB_REPO}/releases/tags/${WEB_VERSION}"; PATTERN="slam_web-${WEB_VERSION}\\.tar\\.gz"; fi; \
    RELEASE_JSON="$(curl -s -H "Accept: ${GH_ACCEPT}" -H "User-Agent: docker-build" "${API}")"; \
    ASSET_URL="$(echo "${RELEASE_JSON}" | jq -r --arg pattern "${PATTERN}" '.assets[] | select(.name | test($pattern)) | .browser_download_url' | head -n 1)"; \
    test -n "${ASSET_URL}"; \
    echo "Downloading: ${ASSET_URL}"; \
    curl -L "${ASSET_URL}" -o web.tar.gz; \
    mkdir -p /tmp/webdist; \
    tar -xzf web.tar.gz -C /tmp/webdist

FROM nginx:1.25-alpine
COPY config/nginx.conf /etc/nginx/nginx.conf
RUN mkdir -p /usr/share/nginx/slam_web/dist
COPY --from=fetcher /tmp/webdist/dist/ /usr/share/nginx/slam_web/dist/
EXPOSE 80
