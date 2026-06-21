# 完善 Provider 完整切换与凭据托管闭环

## Goal

把 Codex / Claude Provider 管理收口为可日常使用的核心链路：支持从当前 live 配置导入 Provider，支持新增 API Relay Provider，支持完整切换真实配置，切换前自动备份，敏感凭据使用 Windows DPAPI 加密托管，并在前端明确展示 dry-run、备份、写入和风险状态。

本任务优先保证 Provider 主链路稳定、可诊断、可回滚，不继续扩展 MCP、Skills、会话、托盘或 Codex++ 注入模块。

## Current Baseline

- 项目已经初始化 Trellis。
- Provider 表已扩展出 `settings_config` / `category` / `meta`。
- 已新增 `provider_secrets` 密文表。
- 已新增 Windows DPAPI 加密/解密服务。
- 已实现 `preview_provider_switch(profile_id)` dry-run。
- 已实现 `apply_provider_switch(profile_id)`，会先备份再写真实配置并更新 SQLite active 状态。
- 已实现从 live 配置导入 Provider 的初版。
- 已实现新增 API Relay Provider 的初版。
- 前端已有“导入当前配置”“新增 API Relay”“预览写入”“应用切换”入口。
- `npm run build` 和 `cargo check` 已通过。
- `cargo fmt` 受本机 Rust 工具链缺少 `rustfmt` 阻塞。

## Requirements

- Provider 列表必须能区分 Codex / Claude、official / api-relay、是否 active、是否携带可写入 live 配置、是否托管凭据。
- 从 live 配置导入 Provider 时，必须完整捕获切换所需配置；敏感内容不得明文写入 SQLite、前端或日志。
- Codex 导入必须覆盖 `config.toml`，如果存在 `auth.json`，必须能作为密文凭据托管并在切换时写回。
- Claude 导入必须能托管完整 JSON 配置；敏感字段不得明文落普通配置字段。
- 新增 API Relay Provider 必须支持 `base_url`、可选 `model`、可选 API key；API key 必须加密保存。
- `apply_provider_switch` 必须先执行备份，再写真实配置，再更新 SQLite active 状态。
- 写真实配置必须使用 atomic write 或等价机制，避免半写状态。
- 备份必须写入 `%APPDATA%\codex-claude-workbench\backups`，不得写入项目目录。
- dry-run 必须显示将影响哪些文件、是否有备份计划、是否会写入、是否涉及托管凭据。
- 前端不得显示真实 token、Cookie、API key、auth.json 内容。
- 失败时必须返回可诊断错误；如果写入失败，不得把 SQLite active 状态错误更新为成功。
- 必须保留 Codex++ 注入模块独立性；Provider 管理不能依赖注入。

## Non-goals

- 不实现 MCP / Skills / Prompts 管理。
- 不实现 Codex 会话搜索 / 导出 / 删除。
- 不实现 Windows 托盘。
- 不实现 Codex++ 注入。
- 不扩展到 Gemini / OpenCode / Hermes。

## Acceptance Criteria

- [x] 可以导入当前 Codex live 配置为 Provider，并在 Provider 列表中看到该 Provider。
- [x] 可以导入当前 Claude live 配置为 Provider，并在 Provider 列表中看到该 Provider。
- [x] 导入后的敏感内容只存在于 DPAPI 加密密文中，普通 SQLite 字段和前端结果不包含明文 token/API key/auth 内容。
- [x] 可以新增 Codex API Relay Provider，填写 base URL、model、API key 后保存；API key 不明文落库。
- [x] 可以新增 Claude API Relay Provider，填写 base URL、model、API key 后保存；API key 不明文落库。
- [x] 对任意 Provider 点击 dry-run，能看到目标文件、备份计划、写入状态和警告信息。
- [x] 对携带完整配置的 Provider 点击应用切换，会先备份，再写真实 Codex / Claude 配置，再更新 active 状态。
- [x] Codex 切换能够按 Provider 恢复 `config.toml`，并在该 Provider 托管 `auth.json` 时恢复 `auth.json`。
- [x] Claude 切换能够按 Provider 恢复对应 JSON 配置。
- [x] 写入失败时保留备份，错误提示可见，active 状态不被误更新。
- [x] `npm run build` 通过。
- [x] `cargo check` 通过。
- [x] `cargo fmt` 通过。

## Notes

- 安全底线：不把 token、Cookie、auth.json 内容明文写入 SQLite 或日志。
- 本任务可以修改真实 Codex / Claude 配置，但必须经过 dry-run 和备份链路。
- 当前项目不是 git 仓库，Trellis 的 commit 阶段可能需要跳过或由用户后续初始化 git。
