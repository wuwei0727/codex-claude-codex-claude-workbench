# Codex Claude Workbench 项目上下文

本文档给新 Codex 会话读取，用于快速理解项目来源、技术选型、功能取舍和下一步实现边界。

## 1. 项目来源

本项目是把下面两个项目中日常需要的能力整合到一个新的 Windows 优先桌面工具中：

| 来源项目 | 地址 | 角色 |
|---|---|---|
| cc-switch | https://github.com/farion1231/cc-switch/ | 作为主要参考底座，重点参考 Provider 管理、配置切换、托盘、MCP / Skills / Prompts、配置备份等能力 |
| CodexPlusPlus | https://github.com/BigPizzaV3/CodexPlusPlus/ | 作为 Codex 增强参考，重点参考 Codex 启动器、会话管理、Markdown 导出、Timeline、注入诊断等能力 |

整合原则：

- 不直接把两个项目硬拼到一起。
- 以新的 `codex-claude-workbench` 项目为主，按需迁移功能思想和模块边界。
- 优先服务个人日常使用：Codex + Claude。
- 暂不追求支持所有 AI CLI、云同步、多语言和复杂插件市场。

## 2. 已确认技术栈

| 模块 | 技术 |
|---|---|
| 框架 | Tauri 2 |
| 前端 | React + TypeScript + Vite |
| UI | shadcn/ui + TailwindCSS |
| 数据请求 | TanStack Query |
| 后端 | Rust |
| 数据库 | SQLite |
| 配置管理 | TOML / JSON / SQLite |
| 安装包 | Windows MSI 或 NSIS |

技术取舍说明：

- `Tauri 2` 适合桌面工具，体积小，和两个来源项目的技术路线接近。
- `React + TypeScript + Vite` 适合快速实现复杂设置界面和状态管理。
- `shadcn/ui + TailwindCSS` 用于做 Windows 11 / Fluent 风格的现代界面。
- `TanStack Query` 用于管理 Tauri command 的请求状态、缓存和刷新。
- `Rust` 负责文件、进程、配置、SQLite、备份、启动器等系统能力。
- `SQLite` 存储结构化状态，例如 Provider、会话索引、设置、历史记录。
- `TOML / JSON` 用于读取和写入 Codex / Claude 原生配置。
- `NSIS` 优先面向普通 Windows 用户安装；`MSI` 适合企业或标准部署场景。

## 3. 产品定位

项目定位为：

```text
Windows 优先的 Codex / Claude 日常工作台
```

核心目标：

- 统一管理 Codex 和 Claude 的 Provider。
- 支持官方登录配置和 API Relay 配置共存。
- 支持一键切换 Provider。
- 支持配置自动备份。
- 支持 MCP / Skills / Prompts 管理。
- 支持 Codex 会话搜索、导出和删除。
- 将 Codex++ 注入能力隔离为可选增强模块。

非目标：

- 第一版不做 Gemini / OpenCode / OpenClaw / Hermes 等更多工具支持。
- 第一版不做复杂云同步。
- 第一版不做插件市场。
- 第一版不做完整多语言 i18n。
- 第一版不做复杂 API 聚合计费。

## 4. 推荐保留功能

### 4.1 P0 必留功能

| 功能 | 来源倾向 | 说明 |
|---|---|---|
| Codex Provider 管理 | cc-switch | 管理 Codex 官方登录和 API Relay 配置 |
| Claude Provider 管理 | cc-switch | 管理 Claude 官方登录和 API Relay 配置 |
| 一键切换 Provider | cc-switch | 主界面切换当前 Codex / Claude Provider |
| 托盘快速切换 | cc-switch | 后续提供 Windows 托盘快捷入口 |
| 配置导入 / 导出 / 自动备份 | cc-switch | 修改真实配置前必须先备份 |
| Codex / Claude 启动器 | 两边整合 | 从工作台启动 Codex App / Claude Code |
| Codex 会话搜索 | CodexPlusPlus | 方便找历史会话 |
| Codex 会话删除 | CodexPlusPlus | 删除前需要确认或备份 |
| Codex Markdown 导出 | CodexPlusPlus | 将会话导出为 Markdown |
| MCP 管理 | cc-switch | 管理 Codex / Claude 使用的 MCP Server |
| Skills / Prompts 管理 | cc-switch | 管理日常 Prompt Preset 和 Skill |

### 4.2 P1 推荐功能

| 功能 | 来源倾向 | 说明 |
|---|---|---|
| Provider 健康检查 | cc-switch | 检测 base_url、配置完整性、可用性 |
| 本地代理 / 热切换 / Failover | cc-switch | 如果用户使用多个 relay，再逐步实现 |
| Codex Provider metadata 同步 | CodexPlusPlus | 辅助 Codex App 侧显示和识别 Provider |
| Codex++ 外部 launcher | CodexPlusPlus | 作为可选增强，不能影响主流程 |
| 注入诊断 | CodexPlusPlus | 只做状态检查和日志，不做强依赖 |
| Worktree 创建助手 | CodexPlusPlus | 如果后续经常用 Codex 做项目，可加入 |
| 会话 Timeline | CodexPlusPlus | 作为会话增强视图 |

### 4.3 P2 暂缓功能

| 功能 | 原因 |
|---|---|
| 云同步 / WebDAV | 单机 Windows 日常使用暂不需要 |
| Deep Link 导入 | 第一版优先级不高 |
| 完整 i18n | 个人工具先中文/英文即可 |
| 多 AI 工具全量支持 | 会拖慢 Codex / Claude 核心体验 |
| 用户脚本注入市场 | 风险高，版本适配成本高 |

## 5. Codex++ 注入功能边界

Codex++ 注入能力必须作为独立模块，不允许成为主程序核心依赖。

推荐模块边界：

```text
src-tauri/src/services/
  codex_service.rs
  codex_session_service.rs
  codex_injector_service.rs
```

```text
src/features/codex/
  sessions/
  launcher/
  settings/

src/features/codex-plus/
  injector/
  diagnostics/
  scripts/
```

原则：

- Provider 管理、配置备份、MCP 管理不依赖注入。
- 注入模块可开启、可关闭。
- 注入失败不影响主程序启动。
- 注入逻辑必须有版本检测、日志和诊断。
- Codex App 更新后，注入模块可以单独适配。

## 6. 当前已完成状态

当前项目路径：

```text
D:\cusor-project\codex-claude-workbench
```

已完成：

- 创建 Tauri 2 + React + Vite 项目骨架。
- 接入 TypeScript。
- 接入 TailwindCSS。
- 手写 shadcn/ui 风格基础组件：
  - Button
  - Card
  - Badge
  - Input
- 接入 TanStack Query。
- 新增 Dashboard 首页。
- 新增 Rust Tauri commands。
- 新增 SQLite 初始化服务。
- 新增数据库迁移：
  - `provider_profiles`
  - `app_settings`
  - `codex_sessions`
- 新增默认 Provider：
  - `codex-official`
  - `claude-official`
- Provider 列表已从 SQLite 读取。
- Provider 切换已写入 SQLite 状态。
- Codex 会话列表已从 SQLite 表读取。
- 浏览器预览环境使用 mock 数据。
- Tauri 运行环境使用 Rust invoke。

## 7. 当前限制

当前 Provider 切换只更新应用自己的 SQLite：

```text
provider_profiles.is_active
```

尚未写入真实配置：

```text
%USERPROFILE%\.codex\config.toml
%USERPROFILE%\.codex\auth.json
%USERPROFILE%\.claude.json
%APPDATA%\Claude
```

原因：

- 写真实配置属于高风险操作。
- 必须先实现配置发现。
- 必须先实现自动备份。
- 必须先实现 dry-run diff。
- 必须避免输出或记录真实 token、Cookie、密钥。

## 8. 下一步建议

下一步优先做：

1. 配置发现
   - 检测 Codex / Claude 配置路径。
   - 只读取非敏感结构。
   - auth 文件只展示存在性和脱敏状态。

2. 自动备份
   - 修改配置前自动备份。
   - 备份目录建议放在：
     `%APPDATA%\codex-claude-workbench\backups`
   - 不把备份放入项目目录。

3. dry-run diff
   - 新增 `preview_provider_switch(profile_id)`。
   - 展示将修改哪些文件和字段。
   - 敏感值必须脱敏。
   - 不实际写入。

4. 真实切换
   - 等配置发现、备份、dry-run 都完成后，再实现 `apply_provider_switch(profile_id)`。
   - 写入失败要保留备份，必要时支持回滚。

5. Codex 会话解析
   - 发现本地 Codex 会话目录。
   - 建立会话索引。
   - 支持搜索、Markdown 导出、删除前确认。

## 9. 验证命令

常用验证：

```bash
npm run build
cargo check
```

开发启动：

```bash
npm run dev
npm run tauri:dev
```

打包：

```bash
npm run tauri:build
```

## 10. 安全要求

- 不要自动 commit、push、merge。
- 不要直接修改真实 Codex / Claude 配置，除非已经完成备份和 dry-run。
- 不要输出真实 token、Cookie、密钥、auth.json 内容。
- 不要把敏感配置明文写入 SQLite。
- 不要把 Codex++ 注入做成主流程强依赖。
- 不要做无关大范围格式化。

