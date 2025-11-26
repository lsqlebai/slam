# SLAM —— 跨平台个人运动数据管理中心

> 轻量、易部署、高性能。通过 AI 能力打通各厂商/平台数据壁垒，让个人运动数据真正归档、可视、可用。

## 项目概览

- 目标：实现一个可在低性能服务器上运行的跨平台个人运动数据管理中心，前后端均以轻量与高性能为优先。
- 架构：后端 `Rust + Axum + SQLite`，前端 `Modern.js + React + MUI`，通过 `Nginx` 统一对外服务。
- AI 能力：接入字节跳动火山引擎 Doubao（Ark）多模态接口，支持图片文字识别，将运动数据结构化入库。
- 部署：提供本地开发与 Docker 一键部署，默认持久化到本地卷。

## 功能特性

- 账号与认证：注册、登录、退出；登录后通过 `Cookie: slam=<JWT>` 进行鉴权（`slam_server/src/handlers/jwt.rs:43`）。
- 运动记录：新增、修改、删除、分页查询，兼容多类型运动（`slam_server/src/handlers/sport_handler.rs:26`）。
- 批量导入：支持厂商 CSV 导入，当前实现 `Xiaomi` 游泳记录解析，支持时间戳单位自动识别（`slam_server/src/service/sport_service.rs:401`）。
- 数据统计：年/月/周/总维度聚合统计，类型分桶，支持最早年份查询（`slam_server/src/service/sport_service.rs:127`）。
- AI 图片识别：将运动截图/照片识别为结构化运动条目（`slam_server/src/service/ai_service.rs:69`）。
- 前端体验：Modern.js 应用，MUI 组件，内置代理到后端，易于本地开发与打包分发（`slam_web/modern.config.ts:9`）。
- 文档与自描述：后端自动生成 OpenAPI 并提供 Swagger UI，访问 `/docs`（`slam_server/src/app/setup.rs:80`）。

## 目录结构

```
slam/
├─ slam_server/         # Rust 后端（Axum + SQLite）
│  ├─ src/
│  │  ├─ app/           # 应用启动、路由与 OpenAPI
│  │  ├─ handlers/      # 鉴权/用户/运动/AI 等接口
│  │  ├─ service/       # 业务服务（AI、统计、图片处理）
│  │  ├─ dao/           # SQLite DAO 实现
│  │  └─ model/         # 领域模型
│  ├─ config/app.yml    # 本地配置
│  └─ tests/            # 接口级测试（cargo test）
├─ slam_web/            # 前端（Modern.js + React + MUI）
│  ├─ src/              # 页面、组件、服务、状态
│  ├─ modern.config.ts  # Dev 代理与构建配置
│  └─ scripts/          # 打包与发布工具
├─ deploy/              # Docker 构建与编排
│  ├─ server/Dockerfile # 后端镜像构建
│  ├─ web/Dockerfile    # 前端镜像构建（从 Release 取包）
│  ├─ config/           # Nginx 与容器内配置
│  └─ docker-compose.yml
└─ LICENSE
```

## 快速开始

### 本地开发

1. 后端（Rust）：
   - 安装 Rust toolchain（推荐稳定版）。
   - 配置 `slam_server/config/app.yml` 中的 `db.path` 与 `security.key`（默认 key 为占位，需手动替换）。
   - 如需启用 AI，设置环境变量 `AI_API_KEY`。
   - 启动：
     ```bash
     cd slam_server
     cargo run
     ```
   - 默认监听 `127.0.0.1:3000`，Swagger UI: `http://127.0.0.1:3000/docs`。

2. 前端（Modern.js）：
   - 安装 Node.js（推荐 >= 16.18，脚本打包使用 22.16）。
   - 使用 `pnpm`（可通过 `corepack enable` 激活）。
   - 启动：
     ```bash
     cd slam_web
     pnpm install
     pnpm dev
     ```
   - 开发环境已代理后端：`/api -> http://127.0.0.1:3000`（`slam_web/modern.config.ts:9`）。

3. 运行测试（后端）：
   ```bash
   cd slam_server
   cargo test
   ```

### Docker 部署

- 先设置环境变量并启动：
  ```bash
  cd deploy
  export AI_API_KEY="<你的密钥>"
  docker compose up -d
  ```
- 访问：
  - Web：`http://localhost:8080`
  - 后端 API（经 Nginx 反代）：`http://localhost:8080/api/...`
  - Swagger UI：`http://localhost:8080/docs`
- 数据持久化：卷 `./db -> /data`（`deploy/docker-compose.yml:31`）。容器内配置挂载至 `/app/config/app.yml`（`deploy/docker-compose.yml:12`）。

### 前端发布产物打包

- 生成 tar 包供 Docker 拉取：
  ```bash
  bash slam_web/scripts/compress_web_dist.sh
  ```
- 输出路径示例：`slam_web/release/slam_web-v0.2.0.tar.gz`；`deploy/web/Dockerfile` 会从 GitHub Release 取对应资产。

## 配置说明

- 本地配置文件：`slam_server/config/app.yml`
  - `server.ip/port`：监听地址与端口。
  - `db.path`：SQLite 文件路径（本地示例 `sport.db`）。
  - `ai.key`：可留空，优先从环境变量 `AI_API_KEY` 读取。
  - `security.salt/key`：用于派生 JWT 密钥与加解密，务必更换默认值（`change-me-key`）。
- 容器内配置：`deploy/config/app.container.yml`（`db.path` 已指向 `/data/sport.db`）。
- Nginx：静态资源与反代（`deploy/config/nginx.conf:6`）。

## API 速览

- 路由常量见：`slam_server/src/app/routes.rs`
  - 状态：`GET /api/status`
  - AI 图片识别：`POST /api/ai/image-parse`
  - 用户注册：`POST /api/user/register`
  - 用户登录：`POST /api/user/login`
  - 用户信息：`GET /api/user/info`
  - 退出登录：`POST /api/user/logout`
  - 头像上传：`POST /api/user/avatar/upload`
  - 运动新增：`POST /api/sport/insert`
  - 运动列表：`GET /api/sport/list?page=0&size=20`
  - 统计：`GET /api/sport/stats?kind=year|month|week|total&year=2025[&month=11][&week=47]`
  - 更新：`POST /api/sport/update`
  - 删除：`POST /api/sport/delete`

### 示例：注册登录与新增运动

```bash
# 注册
curl -s -X POST http://127.0.0.1:3000/api/user/register \
  -H 'Content-Type: application/json' \
  -d '{"name":"alice","password":"p@ssw0rd","nickname":"Alice"}' -i

# 从 Set-Cookie 中提取 slam=...

# 新增运动（带 Cookie）
curl -s -X POST http://127.0.0.1:3000/api/sport/insert \
  -H 'Content-Type: application/json' \
  -H 'Cookie: slam=<你的token>' \
  -d '{"type":"Swimming","start_time":1731888000,"calories":120,"distance_meter":1000,"duration_second":600,"heart_rate_avg":120,"heart_rate_max":140,"pace_average":"3\'59\''"}'
```

### 示例：CSV 批量导入（小米）

```bash
curl -s -X POST http://127.0.0.1:3000/api/sport/import \
  -H 'Cookie: slam=<你的token>' \
  -F 'vendor=xiaomi' \
  -F 'file=@/path/to/sports.csv;type=text/csv'
```

## 设计与实现要点

- 高性能与轻量：Axum + Tokio 异步，SQLite 本地存储，内存缓存减少重复统计（`slam_server/src/service/sport_service.rs:13`）。
- 统一数据模型：`Sport` 统一承载核心字段与类型扩展（如游泳），便于不同来源数据适配（`slam_server/src/model/sport.rs:1`）。
- 安全性：JWT 写入 `HttpOnly` Cookie，过期与校验在服务端完成（`slam_server/src/handlers/jwt.rs:23`）。
- 可观测性：自动生成 API 文档，便于前端与第三方集成（`slam_server/src/app/setup.rs:36`）。

## Roadmap

- 前端响应式布局完善与适配更多终端尺寸。
- 提供 Android / iOS 壳程序（WebView + 原生桥接），支持离线缓存与文件导入。
- 厂商数据适配器扩展：`Huawei`、`Garmin`、`Apple Health`、`Strava` 等。
- 更多 AI 能力：支持多图片合并解析、对话式校正与补录。
- 导入导出：支持全数据导出（CSV/JSON），多源合并与冲突解决。
- 多语言与无障碍：完善 i18n 与可访问性支持。
- 监控与运维：容器健康检查、日志聚合与指标采集。

## 许可证

- 项目采用 MIT 许可证，详见 `LICENSE`。

## 致谢

- 后端基础设施：Axum、Tokio、Utoipa、Rusqlite 等优秀开源项目。
- 前端框架：Modern.js、MUI、React 社区。
- 多模态 AI：Doubao（Ark）能力支持。
