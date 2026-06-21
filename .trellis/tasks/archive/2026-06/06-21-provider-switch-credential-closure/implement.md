# Implement Plan: Provider 完整切换与凭据托管闭环

## Phase 0: Baseline Audit

- [x] 读取 `AGENTS.md`、`docs/project-context-for-codex.md`、当前 Provider 相关代码。
- [x] 确认当前 `provider_profiles`、`provider_secrets`、DPAPI、备份、live 写入实现状态。
- [x] 记录当前已通过验证命令。

## Phase 1: Schema And Secret Storage

- [x] 确认迁移能兼容旧 SQLite：缺列自动补齐，`provider_secrets` 自动创建。
- [x] 确认 `provider_secrets.encrypted_value` 不会被前端返回。
- [x] 确认 DPAPI 加密/解密错误信息不包含明文 secret。
- [x] 确认非 Windows 平台错误清晰，且不会误写明文。

## Phase 2: Import Providers

- [x] Codex 导入：完整 `config.toml` 存入 secret，脱敏摘要存入 `settings_config`。
- [x] Codex 导入：存在 `auth.json` 时完整内容存入 secret，不返回前端。
- [x] Claude 导入：完整 `.claude.json` 存入 secret，脱敏结构存入 `settings_config`。
- [x] 导入结果展示 Provider ID、名称、警告，不展示明文。
- [x] 重复导入不会因为同秒 ID 冲突失败。

## Phase 3: API Relay Provider

- [x] Codex Relay：保存 `base_url`、可选 `model`。
- [x] Codex Relay：填写 API key 时，完整 TOML 写入 DPAPI secret。
- [x] Claude Relay：保存 `base_url`、可选 `model`。
- [x] Claude Relay：填写 API key 时，完整 JSON 写入 DPAPI secret。
- [x] 前端 API key 输入框为 password，保存成功后清空。

## Phase 4: Dry-run And Apply

- [x] dry-run 正确识别是否有可写入 live 配置，包括 secret-backed Provider。
- [x] dry-run 显示目标文件、备份计划、写入状态、警告。
- [x] apply 顺序固定为：加载 Provider -> 备份 -> 解密 secret -> atomic write -> 更新 SQLite active。
- [x] 写入失败时不更新 active 状态。
- [x] apply 结果返回备份文件路径、写入文件路径、警告。

## Phase 5: UI Polish And Diagnostics

- [x] Provider 卡片展示是否托管凭据或至少展示是否携带 live config。
- [x] 导入、新增、预览、应用的错误状态可见。
- [x] 表单字段在移动和桌面视口不溢出。
- [x] 旧文案不再描述“后续才写真实配置”。

## Phase 6: Validation

- [x] `npm run build`
- [x] `cargo check`
- [x] `cargo fmt`
- [x] `cargo test`
- [ ] 可选：在测试配置目录上手动验证导入 -> dry-run -> apply，不使用真实生产凭据。

## Execution Notes

- `rustfmt` 已通过 `rustup component add rustfmt` 安装。
- 已执行敏感字面量扫描，未发现真实 token / access token / refresh token / Cookie 形态匹配。
- 未对用户真实 Codex / Claude 账号配置执行 live apply 手测，避免未经明确单独确认写入真实账号配置。

## Rollback Points

- 数据库迁移问题：回退 schema 相关改动，保留原 `provider_profiles` 字段。
- DPAPI 问题：禁用 secret-backed apply，保留非敏感 Provider 配置。
- 写入问题：使用 backups 目录中的最近备份手动恢复。

## Done Criteria

- PRD 所有 acceptance criteria 达成。
- 验证命令通过或明确记录环境阻塞。
- 未向前端、日志、普通 SQLite 字段输出明文 token/API key/auth 内容。
