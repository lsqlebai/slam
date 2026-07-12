# Web AGENTS.md

本文件适用于 `slam_web/**`，并继承仓库根目录 `../AGENTS.md`。修改 `android/**` 时还必须读取 `android/AGENTS.md`；Android 子文档只补充原生壳规则，本文件的 React/TypeScript 规则仍然有效。

## 前端逻辑总览

应用是 Modern.js 文件路由 SPA：

1. `src/routes/layout.tsx` 提供全局 MUI theme、CSS reset 和通知容器。
2. `src/routes/page.tsx` 是登录后的主界面，通过本地 tab 在运动列表、统计和设置间切换，并按断点显示侧栏或底部导航。
3. `/login`、`/register` 调用用户 service；`useUserStore` 保存当前用户资料并负责刷新/登出。
4. `/addsports` 支持手动创建或选择多图调用 AI；AI 结果通过 router state 传给 `/sport/detail` 复核和保存。
5. 运动列表/详情/统计组件调用 `src/services/sport.ts`；服务统一使用 `src/services/http.ts` 的 Axios 实例。
6. `src/i18n.ts` 和 `useLangStore` 提供中英文文案/语言状态；`PageBase` 提供页面级通知能力。

Web 和 Android 复用这些页面与 service。平台差异应集中在 Capacitor 检测、adapter、hook 或原生目录，不要在每个组件散布平台分支。

## 目录职责

- `src/routes/`：页面入口和路由级组合，不堆可复用领域 UI。
- `src/components/common/`：跨功能控件；`home/`、`sport/`、`stats/`：领域组件。
- `src/services/`：后端 DTO、请求封装、统一错误处理；组件不得创建第二个 Axios client。
- `src/stores/`：Zustand 全局状态。用户资料和语言只有一个来源，不复制到新的全局 store。
- `src/utils/`：时间和通知等无 UI 工具。
- `src/hooks/`：Web/Android 生命周期和返回行为。
- `src/services/capacitor/`：原生 HTTP/Cookie 兼容实验与 adapter；修改时按 `android/AGENTS.md` 验证。
- `modern.config.ts`：路由、Rspack 和开发代理；`capacitor.config.ts`：Android WebView/插件配置。

## 状态与导航

- 用户身份的权威来源是后端 Cookie。Zustand 仅缓存 `nickname/avatar` 等展示信息；应用启动或需要保护页面时通过 `/user/info` 验证会话。
- 统一 Axios response interceptor 捕获 401 并跳转 `/login`，其他 HTTP 错误进入全局通知。service 可补充领域错误，但不要让每个页面重复实现 401 逻辑。
- 登录/注册成功后依赖后端 `Set-Cookie`；登出既调用 API，也清理本地用户状态。改认证流程时同时验证浏览器与 Android。
- 路由数据使用 `useNavigate`/location state。AI 识别结果只是待编辑数据，只有详情页提交后才持久化。
- object URL 在图片删除或页面卸载时应 `URL.revokeObjectURL`，避免多图预览造成内存泄漏。

## API 与模型

- 浏览器 Axios 默认 `baseURL: '/api'`、`withCredentials: true`、15 秒超时；开发代理在 `modern.config.ts` 指向 `127.0.0.1:3000`。
- service 路径相对于 `/api`，例如 `/user/info`、`/sport/list`；不要在组件拼接 API host。
- 后端使用 snake_case 和 Unix 秒，前端 DTO 要保持契约一致。转换显示值时复用 `src/utils/time.ts`。
- `Sport` 类型、extra 和 tracks 的前端定义必须与 Rust `Sport`/`SportType` 一致。新增字段要同步详情表单、列表、统计、AI 结果回填和空值默认行为。
- multipart 字段名是契约：AI 多图重复追加 `image`，头像/CSV 沿用后端定义。不要手工设置 FormData boundary。
- 错误对象可能来自 Axios、adapter 或后端 `{ error }`；统一在 service/http 层规范化为可展示错误。

## UI 与国际化

- 页面负责数据装配，显示组件通过 props 接收数据；复杂运动逻辑放 service/领域组件，不塞进 JSX。
- 使用 MUI theme、`sx` 和已有响应式断点。主页面同时支持小屏底部导航和较宽屏侧栏；涉及布局时至少检查手机和桌面宽度。
- 全屏页使用 `100dvh`、滚动容器和 `env(safe-area-inset-*)`；修改固定底栏/页头时防止内容被遮挡。
- 用户可见文案全部进入 `TEXTS` 并提供 zh/en。不要把中文或英文 toast 硬编码在业务分支中。
- 无障碍至少保留 input label、button 文本/aria-label、键盘 Escape/返回行为和合理焦点。

## 代码风格

- TypeScript/React 18，组件使用函数和 hooks。避免 `any`；API DTO 使用明确 interface/type。
- 遵循 `biome.json`：单引号、空格缩进、约 80 列、组织 imports。不要修改 `dist/`、`node_modules/` 或生成的 Capacitor 文件来修 Web 问题。
- effect 中注册事件、Capacitor listener、object URL 或异步资源时实现清理；依赖数组保持真实依赖，避免陈旧闭包。
- 通知复用 `PageBase/useToast` 或 `utils/notify`，不要引入新的全局提示体系。

## 常见改动闭环

- 新增页面：创建文件路由 → 复用 `PageBase`/header → 接 i18n → 从现有导航进入 → 验证刷新和返回。
- 新增 API：先确认后端 OpenAPI → 在 `services/` 定义 DTO/方法 → 页面/组件调用 → 处理 loading、空、错误、401 → 更新测试或手工用例。
- 新增运动字段/类型：同步 service DTO、详情编辑、列表/统计展示、ExtraConfig/track 逻辑、i18n，并按根文档检查后端。
- 修改认证或 HTTP：同时读取 Android 文档，验证浏览器 Cookie、原生 host、multipart 和 401 跳转。

## 验证

从 `slam_web/` 执行：

```bash
pnpm install
pnpm lint
pnpm build
```

项目当前没有前端自动化测试脚本。按改动手工验证：

- 登录、注册、401 跳转和登出；
- 运动列表、详情新增/编辑/删除及统计刷新；
- AI 多图选择、预览、失败/超时和识别后编辑；
- zh/en 文案、手机/桌面布局、加载/空/错误态；
- 涉及 Android 平台代码时，继续执行 `android/AGENTS.md` 的同步和 Gradle 验证。
