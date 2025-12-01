# <img src="slam.png" alt="SLAM" height="28"> SLAM — Cross-Platform Personal Sports Data Hub

[English](README.md) | [简体中文](README.cn.md)

Changed your sports watch, fitness band, or cycling computer — or switched platforms — and now your history is scattered and hard to consolidate? SLAM is built for this — effortlessly unify and connect your workout data so your records are truly archivable, visualized, and usable.

> Lightweight, easy to deploy, and high-performance. Break platform/vendor data silos with AI so your personal workout data is archived, visible, and usable.

## Overview

- Goal: Build a cross-platform personal sports data hub that runs on low-end servers, prioritizing lightweight and performance across both backend and frontend.
- Architecture: Backend `Rust + Axum + SQLite`, frontend `Modern.js + React + MUI`, unified serving via `Nginx`.
- AI Capability: Integrates ByteDance Volcano Engine Doubao (Ark) multimodal APIs for image/OCR to structure workout data into the database.
- Deployment: Supports local development and one-click Docker deployment, with default persistence to local volume.
- Sport types support: currently only Swimming; Running (incomplete, basic data only); Cycling (incomplete, basic data only).

## Features

- Accounts & Auth: Register/login/logout; after login, authentication via `Cookie: slam=<JWT>` (`slam_server/src/handlers/jwt.rs:43`).
- Workout Records: Create/update/delete/paginated list, multi-sport types supported (`slam_server/src/handlers/sport_handler.rs:26`).
- Bulk Import: Vendor CSV import; Xiaomi swimming records parsing with automatic timestamp unit detection (`slam_server/src/service/sport_service.rs:401`).
- Stats: Aggregations by year/month/week/total, type buckets, earliest year supported (`slam_server/src/service/sport_service.rs:127`).
- AI Image Parsing: Recognize workout screenshots/photos into structured records (`slam_server/src/service/ai_service.rs:69`).
- Frontend DX: Modern.js app with MUI, built-in proxy to backend for easy local dev and packaging (`slam_web/modern.config.ts:9`).
- OpenAPI & Swagger: Backend auto-generates API docs, available at `/docs` (`slam_server/src/app/setup.rs:80`).
- Responsive layout improvements and better device coverage.
- Android app (native HTTP; configurable backend; automatic cross-origin handling).

## Repository Layout

```
slam/
├─ slam_server/         # Rust backend (Axum + SQLite)
│  ├─ src/
│  │  ├─ app/           # App bootstrap, routes, OpenAPI
│  │  ├─ handlers/      # Auth/User/Sport/AI endpoints
│  │  ├─ service/       # Business services (AI, stats, image processing)
│  │  ├─ dao/           # SQLite DAO implementations
│  │  └─ model/         # Domain models
│  ├─ config/app.yml    # Local config
│  └─ tests/            # API-level tests (cargo test)
├─ slam_web/            # Frontend (Modern.js + React + MUI)
│  ├─ src/              # Pages, components, services, state
│  ├─ modern.config.ts  # Dev proxy & build
│  └─ scripts/          # Packaging & release scripts
├─ deploy/              # Docker build & compose
│  ├─ server/Dockerfile # Backend image build
│  ├─ web/Dockerfile    # Frontend image build (pulls Release assets)
│  ├─ config/           # Nginx & in-container config
│  └─ docker-compose.yml
└─ LICENSE
```

## Getting Started

### Local Development

1. Backend (Rust):
   - Install the Rust toolchain (stable recommended).
   - Configure `db.path` and `security.key` in `slam_server/config/app.yml` (default key is a placeholder; replace it).
   - To enable AI, set env var `AI_API_KEY`.
   - Run:
     ```bash
     cd slam_server
     cargo run
     ```
   - Default listen `127.0.0.1:3000`, Swagger UI: `http://127.0.0.1:3000/docs`.

2. Frontend (Modern.js):
   - Install Node.js (>= 16.18; release script uses 22.16).
   - Use `pnpm` (activate via `corepack enable`).
   - Run:
     ```bash
     cd slam_web
     pnpm install
     pnpm dev
     ```
   - Dev proxy to backend: `/api -> http://127.0.0.1:3000` (`slam_web/modern.config.ts:9`).

3. Backend tests:
   ```bash
   cd slam_server
   cargo test
   ```

### Docker Deployment

- Set env and start:
  ```bash
  cd deploy
  export AI_API_KEY="<your-key>"
  docker compose up -d
  ```
- Access:
  - Web: `http://localhost:8080`
  - Backend API (via Nginx reverse proxy): `http://localhost:8080/api/...`
  - Swagger UI: `http://localhost:8080/docs`
- Data persistence: volume `./db -> /data` (`deploy/docker-compose.yml:31`). In-container config mounted at `/app/config/app.yml` (`deploy/docker-compose.yml:12`).

### Build Frontend Release Artifact

- Generate tarball for Docker to fetch:
  ```bash
  bash slam_web/scripts/compress_web_dist.sh
  ```
- Example output: `slam_web/release/slam_web-v0.2.0.tar.gz`; `deploy/web/Dockerfile` pulls the matching asset from GitHub Release.

## Configuration

- Local config file: `slam_server/config/app.yml`
  - `server.ip/port`: listen address and port.
  - `db.path`: SQLite file path (e.g., `sport.db`).
  - `ai.key`: optional; prefer reading from env `AI_API_KEY`.
  - `security.salt/key`: derive JWT secrets and (de)encryption; replace the defaults (`change-me-key`).
- In-container config: `deploy/config/app.container.yml` (`db.path` points to `/data/sport.db`).
- Nginx: static assets and reverse proxy (`deploy/config/nginx.conf:6`).

### Android Configuration

- Set backend base in `slam_web/.env`:
  ```bash
  MODERN_PUBLIC_API_BASE=https://<your-host-or-ip>/api
  ```
- On Android native (Capacitor), the frontend reads this variable and uses a native HTTP adapter to bypass browser CORS (`slam_web/src/services/http.ts:11`, `slam_web/src/services/axiosCapacitorAdapter.ts:23`).
- When unset, default `baseURL='/api'` relies on dev proxy or Nginx reverse proxy.

## API Quick Reference

- Route constants: `slam_server/src/app/routes.rs`
  - Status: `GET /api/status`
  - AI image parse: `POST /api/ai/image-parse`
  - User register: `POST /api/user/register`
  - User login: `POST /api/user/login`
  - User info: `GET /api/user/info`
  - Logout: `POST /api/user/logout`
  - Avatar upload: `POST /api/user/avatar/upload`
  - Sport insert: `POST /api/sport/insert`
  - Sport list: `GET /api/sport/list?page=0&size=20`
  - Stats: `GET /api/sport/stats?kind=year|month|week|total&year=2025[&month=11][&week=47]`
  - Update: `POST /api/sport/update`
  - Delete: `POST /api/sport/delete`

### Example: Register, Login, and Insert Sport

```bash
# Register
curl -s -X POST http://127.0.0.1:3000/api/user/register \
  -H 'Content-Type: application/json' \
  -d '{"name":"alice","password":"p@ssw0rd","nickname":"Alice"}' -i

# Extract slam=... from Set-Cookie

# Insert a sport (with Cookie)
curl -s -X POST http://127.0.0.1:3000/api/sport/insert \
  -H 'Content-Type: application/json' \
  -H 'Cookie: slam=<your-token>' \
  -d '{"type":"Swimming","start_time":1731888000,"calories":120,"distance_meter":1000,"duration_second":600,"heart_rate_avg":120,"heart_rate_max":140,"pace_average":"3\'59\''"}'
```

### Example: CSV Bulk Import (Xiaomi)

```bash
curl -s -X POST http://127.0.0.1:3000/api/sport/import \
  -H 'Cookie: slam=<your-token>' \
  -F 'vendor=xiaomi' \
  -F 'file=@/path/to/sports.csv;type=text/csv'
```

## Design Notes

- Performance & Lightweight: Axum + Tokio async, SQLite local store, in-memory caching to avoid repeated aggregation (`slam_server/src/service/sport_service.rs:13`).
- Unified Model: `Sport` centralizes core fields with type extensions (e.g., swimming), enabling easy adaptation across sources (`slam_server/src/model/sport.rs:1`).
- Security: JWT stored in `HttpOnly` cookie; expiration and verification handled server-side (`slam_server/src/handlers/jwt.rs:23`).
- Observability: Auto-generated API docs easing frontend/third-party integration (`slam_server/src/app/setup.rs:36`).

## Roadmap

- More sport types.
- More AI capabilities: multi-image merge parsing, dialog-style correction and completion.
- Import/Export: full data export (CSV/JSON), multi-source merge and conflicts.
- i18n & Accessibility enhancements.
- iOS app shell (WebView + native bridge), offline cache and file import.

## License

- MIT License, see `LICENSE`.

## Acknowledgements

- Backend Infra: Axum, Tokio, Utoipa, Rusqlite, and more great OSS.
- Frontend Framework: Modern.js, MUI, React community.
- Multimodal AI: Doubao (Ark).
