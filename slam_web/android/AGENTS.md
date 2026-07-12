# Android AGENTS.md

本文件适用于 `slam_web/android/**`。它依次继承仓库根目录 `../../AGENTS.md` 和 Web 文档 `../AGENTS.md`：React 页面、API、i18n 和前端构建仍按 Web 文档执行；本文件仅补充 Capacitor/Gradle/原生 Android 规则。

## Android 逻辑总览

Android 不是独立业务客户端，而是 SLAM Web SPA 的 Capacitor 壳：

1. `pnpm build` 生成 Modern.js Web 产物。
2. `package.json` 的 `android:build:web` 使用已忽略的 `android/.env` 构建 Web 产物；`android:sync` 将 `dist/html/main/index.html` 复制为 `dist/index.html`，再执行 `cap sync android`，把 Web 资源和插件配置同步进 Android 工程。
3. `capacitor.config.ts` 定义 `appId=com.slam.web`、`webDir=dist`、HTTP scheme/cleartext 和 Capacitor HTTP/Cookie 插件。
4. `MainActivity` 继承 `BridgeActivity`，仅设置 system window 行为并保持屏幕常亮；页面和业务仍运行在 WebView。
5. React 侧用 `Capacitor.isNativePlatform()` 判断 Android，读取 `MODERN_PUBLIC_API_BASE` 作为绝对 API 地址；返回键由 `@capacitor/app` hooks 处理。
6. Gradle 将 Web 资源和 Capacitor 插件打包为 APK；release 输出名为 `slam_v<versionName>.apk`。

当前 `src/services/http.ts` 中自定义 `axiosAndroidAdapter` 和 `initHttp()` 接线被注释，实际请求仍主要走 Axios/WebView 行为。不要把“存在 adapter/cookie 文件”误认为它们已经启用；启用前必须完成端到端 Cookie、multipart、错误对象和超时验证。

## 关键配置

- `android/app/build.gradle`：applicationId、versionCode/versionName、release 文件名和依赖。
- `android/variables.gradle`：minSdk 23、compileSdk/targetSdk 35 及 AndroidX 版本。
- `android/app/src/main/AndroidManifest.xml`：Launcher Activity、FileProvider 和 INTERNET 权限。
- `capacitor.config.ts`：Web 产物目录、Android HTTP scheme、cleartext 和插件开关。
- `android/app/src/main/java/com/slam/web/MainActivity.java`：原生 Activity 定制。
- `src/services/http.ts`、`src/services/capacitor/`：API base、潜在原生 adapter 和 Cookie 持久化。
- `src/hooks/useUnifiedBack.ts`、`useAndroidDoubleBackExit.ts`：详情返回与首页双击退出。

## 网络、Cookie 与安全

- Android 构建从已忽略的 `android/.env` 读取 `MODERN_PUBLIC_API_BASE=https://host/api`；必须是绝对 URL，并包含 `/api`。`pnpm android:build:web`、`android:sync` 和 `android:build` 会自动加载它。未设置时 `/api` 会相对于 WebView origin，通常无法访问远端后端。
- 生产环境优先 HTTPS。`androidScheme: 'http'` 和 `cleartext: true` 只表示当前允许 HTTP，不应成为生产默认安全假设。
- 后端认证依赖 `slam` HttpOnly Cookie。任何 adapter 切换都要验证登录响应的 `Set-Cookie`、后续请求 Cookie、App 重启、登出清理、不同 origin 隔离和 401 跳转。
- `@capacitor-community/http` 的 Cookie 保存代码使用 localStorage 镜像；其中可能包含会话信息。若重新启用，需评估安全性和 HttpOnly 语义，不能记录或展示 Cookie。
- 自定义 Axios adapter 必须保持 baseURL 合并、query、headers、JSON、FormData 多文件、响应解析、status 和 timeout 行为与浏览器端兼容。

## 返回键与生命周期

- 普通子页面使用 `useUnifiedBack`，同时支持 Web Escape 和 Android backButton；effect 卸载时移除 listener。
- 首页使用 `useAndroidDoubleBackExit`：首次返回提示，指定时间内第二次调用 `App.exitApp()`。不要在同一页面叠加两个 backButton listener。
- `MainActivity` 当前设置 `FLAG_KEEP_SCREEN_ON`。移除或改为按页面控制属于产品行为变化，需要说明耗电/体验影响。
- 页面布局需考虑状态栏、导航栏、键盘和 safe-area；固定页头/底栏修改后在实体设备或模拟器验证。

## 生成文件与原生改动边界

- `cap sync android` 会更新 `capacitor.settings.gradle`、`capacitor.build.gradle`、插件和资源。同步后只提交任务需要且可解释的差异。
- 不手工编辑 `android/app/src/main/assets/public`、`build/`、`.gradle/` 等生成输出；Web UI 问题应回到 `slam_web/src/` 修复。
- 原生权限、Activity、provider、deep link 或插件配置可以直接修改对应 manifest/Java/Gradle，但必须确认 `cap sync` 不会覆盖。
- 新权限遵循最小权限原则，并同步用户说明/隐私文档。当前仅声明 INTERNET。
- 应用版本发布必须同时递增整数 `versionCode` 和用户可见 `versionName`；不要只改 APK 文件名。

## 构建命令

从 `slam_web/` 执行，使用 JDK 21 与已配置的 Android SDK（当前 Capacitor 7 Android 模块以 Java 21 为 source compatibility）：

```bash
pnpm install
pnpm lint
pnpm build
pnpm android:sync
cd android
./gradlew assembleDebug
```

Windows PowerShell/CMD 使用：

```powershell
pnpm build
Copy-Item dist/html/main/index.html dist/index.html -Force
npx cap sync android
cd android
.\gradlew.bat assembleDebug
```

Android 专用构建要求支持 `--env-file` 的 Node.js 20.6+（项目本地推荐 Node 22.16）。发布前仍需显式检查并更新 `versionCode`/`versionName`。

## 验证矩阵

涉及 Android 的改动至少验证：

- Gradle debug 构建成功，必要时再构建 release；
- App 冷启动、前后台切换和 Web 资源加载；
- 使用实际 `MODERN_PUBLIC_API_BASE` 登录、保持会话、401 和登出；
- 运动 CRUD、统计、头像/CSV，以及 AI 多图片 multipart（若受影响）；
- 系统返回键、首页双击退出、Escape 的 Web 行为不回归；
- 小屏/刘海/safe-area、软键盘、旋转或配置变化；
- 若改版本，检查 APK manifest 中的 versionCode/versionName 和 release 文件名。

没有可用 Android SDK、JDK、模拟器或签名材料时，不要伪称已验证；完成 Web lint/build 和静态配置检查，并在交付中明确列出未执行的原生验证。
