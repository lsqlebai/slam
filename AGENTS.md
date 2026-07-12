# AGENTS.md

本文件是仓库级总则，适用于整个 SLAM 仓库。进入子目录工作时，必须继续读取并遵守离目标文件最近的 `AGENTS.md`；子目录文档补充或细化本文件，冲突时以更具体的文档为准。

## 文档继承关系

- 修改 `slam_server/**`：本文件 → `slam_server/AGENTS.md`。
- 修改 `slam_web/**`（不含 `android/**`）：本文件 → `slam_web/AGENTS.md`。
- 修改 `slam_web/android/**`：本文件 → `slam_web/AGENTS.md` → `slam_web/android/AGENTS.md`。
- 修改 `deploy/**` 或根目录文件：仅使用本文件。
- 跨端功能必须同时应用所有受影响目录的规则。例如新增运动字段通常涉及后端、Web，必要时还涉及 Android 打包验证。

## 项目与边界

SLAM 是跨平台个人运动数据管理应用：

- `slam_server/`：Rust/Axum/SQLite API，负责认证、业务规则、数据持久化、统计和 AI 图片识别。详细逻辑见 `slam_server/AGENTS.md`。
- `slam_web/`：Modern.js/React/MUI Web 应用，负责页面、状态、国际化和 API 交互。详细逻辑见 `slam_web/AGENTS.md`。
- `slam_web/android/`：Capacitor 生成并维护的 Android 壳及少量原生定制。详细逻辑见 `slam_web/android/AGENTS.md`。
- `deploy/`：后端镜像、前端 Release 产物、Nginx 和 Docker Compose。
- `README.md` / `README.cn.md`：用户侧英文/中文说明；功能、命令、配置或 API 变化应同步两份。

浏览器和 Android 共用前端业务代码，后端是唯一业务数据源。不要在某一端复制一套不同的领域规则。

## 跨端契约

- API 统一位于 `/api`。浏览器依靠开发代理或 Nginx；Android 使用 `MODERN_PUBLIC_API_BASE` 指定绝对地址。
- 登录态由后端 `slam` HttpOnly Cookie 表示。后端身份隔离、前端 `withCredentials`、Android Cookie 行为必须一起考虑。
- 后端时间戳为 Unix 秒。前端展示/编辑统一复用时间工具，避免毫秒/秒混用。
- `Sport` 是跨端核心模型。类型、`extra`、`tracks`、统计含义或字段变化，必须同步数据库兼容层、API/OpenAPI、前端类型/表单/展示和测试。
- 用户可见文本进入现有 i18n；README 的公共行为说明保持中英文一致。
- 不记录或提交 API key、JWT、Cookie、完整图片/base64、真实用户数据、SQLite 数据库和构建产物。

## 部署逻辑

`deploy/docker-compose.yml` 从指定 Git ref 构建后端，并从 GitHub Release 下载前端 tar 包。容器后端监听 `0.0.0.0:3000`、数据库为 `/data/sport.db`；Nginx 在 8080 提供 SPA，将 `/api/` 转发到后端。版本发布时核对 Cargo/package 版本、Compose 的 `GIT_REF`/`WEB_VERSION` 和 Release 资产名。

Compose 使用 bind-backed volume，首次启动前确保 `deploy/db/` 存在。示例 `security.key`/`salt` 仅用于开发，不能直接用于生产。

## 通用工作方式

- 先阅读目标目录 README、manifest、入口和相邻模块，再修改；不要仅凭根 README 推断实现。
- 保留用户已有改动，不编辑无关文件，不提交生成物或做全仓库无关格式化。
- 新文本统一 UTF-8。源码存在历史乱码；不要扩大问题，触及相关行时可在行为不变的前提下局部修复。
- 改动放在职责正确的层级；跨端契约变更必须一次性闭环，不留下静默不兼容。
- 验证以目标目录 `AGENTS.md` 为准；交付时列出已运行命令和因凭据、平台或外部服务未运行的项目。

部署文件修改至少执行：

```bash
cd deploy
docker compose config
```

## 完成标准

- 功能逻辑、API/数据契约、UI 和持久化修改一致。
- 数据模型变化兼顾已有 SQLite/JSON 数据；需要迁移时提供明确兼容或迁移路径。
- 受影响的 OpenAPI、i18n、README、示例配置和测试已同步。
- 没有加入密钥、数据库、用户数据、`dist/`、`node_modules/` 或 Android build 输出。
- 已执行与风险相称的格式、编译、测试或手工验证，并明确披露限制。
