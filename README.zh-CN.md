# Camera Connector 中文说明

Camera Connector 是一个本地相机导入和照片筛选系统。相机通过本地网络把
JPEG、RAW、视频文件推送到接收端；共享 Rust 核心记录传输事实，把文件索引到
明确的拍摄项目中，完成资产分组，并把同一套项目模型提供给桌面端、Android
端和 CLI。

当前产品主路径是 FTP push。Android 端把 FTP 作为可见接收方式，未来
STC-style 路径保持禁用状态。SFTP 仍保留在 core/CLI 里作为工程验证能力，不
是当前 Android 用户主路径。

## 仓库结构

```text
core/                  共享 Rust 领域核心、存储、接收器、服务、分析
core-ffi/              移动端 C ABI 和 JNI facade
apps/android/          原生 Android：Kotlin、Compose、前台服务
apps/desktop/          Tauri 桌面端：TypeScript 工作台 + Rust commands
tools/cli/             基于共享 core 的命令行适配器
scripts/               验证、Android 构建、设备和 smoke 脚本
docs/                  产品、架构、协议、开发和验证文档
prototypes/            静态产品原型
```

这些不属于产品架构本身：`target/`、Android Gradle build 目录、桌面端
`node_modules/`、IDE 配置目录和 `.git/`。它们是构建、工具或版本控制状态。

## 语义拆分

现在项目最重要的边界不是“桌面/Android/CLI”，而是这些产品语义：

- 接收事实：FTP/SFTP listener 生命周期、认证连接、传输记录、已连接设备和
  runtime status。
- 资产事实：存储对象、RAW/JPEG/video 角色、分组、重复导入和来源元数据。
- 项目范围：用户创建的拍摄项目，拥有导入资产、dashboard、扫描、同步和评估
  设置。
- 用户判断：favorite、marked、guest mark、手动 burst 调整和删除动作。
- 本地技术评估：客观 CV 风险和 gate 上下文。
- 模型评估：provider 支持的摄影评分、tier 和 summary。
- 选择推荐：模型推荐输出，只表示推荐结果。
- 发布写入：staged upload bytes、最终平台存储和写入重试状态。
- 分享同步：LAN share session、guest mark 和 project snapshot。
- 平台壳：desktop、Android、CLI 只是同一套 core 上的适配层。

更完整的模块归属见 `docs/architecture.md`。

## 主要入口文件

- 共享服务门面：`core/src/service.rs`
- core 对外导出：`core/src/lib.rs`
- SQLite store 和 schema：`core/src/storage/`
- receiver runtime：`core/src/runtime.rs`
- 移动端 facade：`core-ffi/src/lib.rs`
- Android gateway 边界：`apps/android/app/src/main/java/com/cameraconnector/app/core/CoreGateway.kt`
- Android 前台接收服务：`apps/android/app/src/main/java/com/cameraconnector/app/service/ReceiverForegroundService.kt`
- Desktop Tauri 后端：`apps/desktop/src-tauri/src/lib.rs`
- Desktop TypeScript API：`apps/desktop/src/desktopApi.ts`
- CLI 入口：`tools/cli/src/main.rs`

## 开发环境

Windows 开发需要：

- Rust stable through `rustup`
- Visual Studio 2022 Build Tools with C++ workload
- PowerShell
- Android 验证所需的 JDK 17、Android SDK 36、Gradle
- 桌面 TypeScript 工作台所需的 Node.js/npm

详细命令见 `docs/development.md`。

## 验证命令

完整 core/CLI 验证：

```powershell
powershell -ExecutionPolicy Bypass -File scripts\verify.ps1
```

常用聚焦检查：

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\verify_mobile_ffi_contract.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\verify_android_skeleton.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\verify_android_build.ps1
```

桌面端逻辑测试在 `apps/desktop` 下运行：

```powershell
cmd /c npm ci
cmd /c npm run test:logic
```

## 文档入口

建议阅读顺序：

- `docs/README.md`：文档索引
- `docs/architecture.md`：语义和模块边界
- `docs/product/PRD.md`：产品需求
- `docs/protocol.md`：接收器和存储协议说明
- `apps/android/README.md`：Android 构建和设备验证
- `docs/development.md`：本地开发和验证
- `docs/compatibility.md`：真实相机和 Android 物理设备记录
- `docs/troubleshooting.md`：接收器排障
