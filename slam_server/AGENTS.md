# Backend AGENTS.md

本文件适用于 `slam_server/**`，并继承仓库根目录 `../AGENTS.md`。根文档规定跨端契约和通用要求；本文件只细化 Rust 后端逻辑。若修改同时影响 Web 或 Android，还要读取对应目录的 `AGENTS.md`。

## 后端逻辑总览

请求主链路为：

1. `src/main.rs` 初始化 tracing，调用 `app::run()`。
2. `src/app/setup.rs` 从当前工作目录创建 `AppConfig`（优先读取已忽略的 `config/app.local.yml`，否则读取 `config/app.yml`），初始化 SQLite repository、JWT、统计缓存和各 service，组合 Router 与 Swagger UI。
3. `src/app/routes.rs` 保存 `/api` 路径常量；handler 通过 Axum extractor 解析请求。
4. 受保护 handler 使用 `handlers/jwt.rs::Context` 从 `slam` Cookie 验证 JWT 并取得 `uid`。
5. handler 完成 HTTP 层校验/响应转换，将工作交给 service。
6. service 编排领域校验、DAO、缓存、图片或 LLM；DAO trait 在 `src/dao/idl.rs`，SeaORM/SQLite 实现在 `src/dao/repository/`。
7. `src/dao/repository/schema.rs` 启动时建表并做轻量兼容；模型和数据库 JSON 的兼容转换位于 `repository/compat.rs`、`dao/entities.rs`。

Swagger UI 是 `/docs`，OpenAPI JSON 是 `/api-docs/openapi.json`。默认监听 `127.0.0.1:3000`。

## 模块职责

- `src/app/`：依赖装配、共享 `AppState`、路由、中间件、OpenAPI；不要放领域计算。
- `src/handlers/`：HTTP extractor、multipart、基础输入校验、状态码和 DTO；保持薄层。
- `src/service/`：业务规则。`SportService` 管理运动 CRUD、CSV 导入、统计及缓存；`UserService` 管理密码处理和用户信息；`AIService`、`ImageService`、`llm.rs` 管理图片和外部模型。
- `src/model/`：领域/API 模型、运动类型一致性和 XML 解析；`Sport.type`、`extra`、`tracks` 必须匹配。
- `src/dao/`：DAO trait、数据库实体、持久化映射、schema、兼容读取和缓存实现。
- `macros/`：`#[inject_ctx]` 过程宏。它会把 `ctx: &Context` 追加到函数签名，因此源码里 service 方法看似没有 ctx 参数，调用处仍需传入 `&ctx`。修改签名或排查编译错误时要考虑宏展开。
- `tests/tests.rs`：配置、DAO、CSV 和真实 LLM 等集成测试；`tests/api_tests.rs`：内存 Router API 测试；模型单测位于 `src/model/sport.rs`。

## 核心业务规则

### 认证与用户隔离

- 注册/登录成功后由 `token_response` 设置 `slam`、`HttpOnly`、`SameSite=Lax` Cookie；登出用 `Max-Age=0` 清除。
- JWT 使用 HS256，TTL 和 secret 来自 `security` 配置。仓库默认 secret 只是占位值。
- 任何运动查询、更新、删除和统计都必须携带 `ctx.uid` 到 DAO。即使已按 id 查询，也不能省略 uid 过滤。
- 用户密码经 `UserService` 使用 security salt/key 处理；不要在 handler 或 DAO 另建一套密码规则。

### 运动数据

- 写接口在入库前调用 `Sport::validate_type_consistency()`。新增运动类型时同步 `SportType` 字符串映射、extra/track 枚举、XML、CSV、数据库 tagged JSON、统计及前端。
- `insert_many` 使用事务；批量导入应保持全成或全败，不能留下半批数据。
- repository 读取 `extra`/`tracks` 时保留旧 JSON 兼容路径。改变序列化结构不能只改新写入格式，必须验证历史数据仍可读取。
- 列表按开始时间倒序，page 从 0 开始，size 默认 20 且上限 100。
- 时间范围和统计使用 Unix 秒；年/月/ISO 周边界在 service 计算。

### 统计缓存

- total 缓存键为 uid；year 缓存键为 `uid@year`。month/week 当前从数据计算。
- insert/import/delete 后失效 total 和受影响年份；update 必须同时失效旧年份与新年份。
- 不要绕过 `SportService` 直接写 DAO，否则缓存可能陈旧。新增写入口必须实现相同失效语义。

### AI 与图片

- `/api/ai/image-parse` 接收一个或多个 multipart `image`，Router body limit 为 50 MiB；头像上传 limit 为 20 MiB。
- `ImageService` 解码、切分/压缩并产生 base64；`AIService` 构造多模态请求，LLM 返回 XML 后解析为 `Sport`。
- Doubao key 优先从 `AI_API_KEY` 读取，否则使用配置的 `ai.key`；模型来自 `config.ai.model`。仓库配置中的 key 必须为空，本机密钥只能放入已忽略的 `config/app.local.yml`，且不得进入日志、测试输出或响应。
- 保持 `LLMError → ErrorResponse/HTTP` 映射和 OpenAPI 一致；真实外部调用与 mock 单测明确分离。

## 新增或修改 API

按以下顺序闭环：

1. 在 `src/app/routes.rs` 增加/修改路径常量。
2. 在 `src/app/setup.rs` 注册 Router，并在 `OpenApi` 的 paths/schemas 中登记。
3. 在 handler 定义请求/响应 DTO、认证 extractor、multipart/body limit 和 utoipa 注解。
4. 在 service 实现业务规则；需要持久化时先扩展 DAO trait，再实现 repository。
5. 模型变更同步 `model`、`dao/entities.rs`、repository 映射、schema/兼容读取。
6. 添加 handler/service/model/DAO 级测试，并同步 Web service 与根 README 中的公共 API 说明。

不要在多个文件重复裸路径，也不要让 handler 直接执行 SQL。

## 配置与数据库

- 所有 cargo 命令从 `slam_server/` 执行；`AppConfig::default()` 按当前工作目录优先读取 `config/app.local.yml`，不存在时回退到 `config/app.yml`。本地配置已被 Git 忽略，只能保存本机开发密钥，不能强制添加到版本控制。
- 配置缺失或解析失败会静默回落默认值。修改配置加载逻辑时添加缺失、部分字段和非法 YAML 测试。
- 本地数据库默认是 `sport.db`，容器数据库是 `/data/sport.db`。不要删除或覆盖用户已有数据库。
- schema 目前用 `CREATE TABLE IF NOT EXISTS` 和容错 `ALTER TABLE`，没有完整迁移框架。破坏性 schema 变化必须设计显式迁移，不能依赖忽略 ALTER 错误。

## 编码风格

- 使用稳定 Rust 与 edition 2024；运行 rustfmt，不手工对齐格式。
- 错误在 DAO 转为可诊断字符串，在 service 转为 `ServiceError`，在 handler 映射 HTTP；不要 `unwrap` 用户输入或外部响应。
- `Arc<dyn Trait + Send + Sync>` 用于可替换 DAO/LLM/cache，测试优先注入 mock。
- 不输出 JWT、密码、key、base64 或原始图片。tracing 字段只保留必要的请求元数据。

## 验证

常规离线验证：

```bash
cd slam_server
cargo fmt --all -- --check
cargo check --all-targets
cargo test --lib
cargo test --test api_tests -- --skip test_image_endpoint --skip test_image_endpoint_comparison
```

注意：

- 完整 `cargo test` 会运行 `tests/tests.rs::test_doubao_request`，它要求 `AI_API_KEY` 并访问真实服务。
- `api_tests` 的 `test_image_endpoint`、`test_image_endpoint_comparison` 也会访问真实模型；`test_image_running_recognition` 已 ignored。
- API/DAO 集成测试默认共用 `sport.db`，且存在固定用户名。重复或并行运行可能受旧数据影响；需要隔离时传入临时 `AppConfig` 和临时数据库，绝不删除用户数据库。
- 只有任务涉及 AI、用户授权联网并提供测试凭据时才运行真实 AI 回归；交付时说明未运行原因。
