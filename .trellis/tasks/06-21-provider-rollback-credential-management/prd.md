# Provider 回滚与凭据管理

## Goal

补齐 Provider 管理的可恢复性和凭据维护能力。当前项目已经支持 Codex / Claude Provider 的真实配置写入、导入、备份、dry-run、DPAPI 密文托管和 API Relay 创建；本任务在此基础上增加：

- 备份列表。
- 从备份恢复配置。
- Provider 凭据托管状态展示。
- 更新 / 删除已托管凭据。
- 同步更新过期文档。

## Requirements

- 用户能在前端查看 Provider 配置备份列表，至少包含备份文件路径、目标类型、原始配置路径、时间信息和文件存在状态。
- 用户能从某条备份恢复到对应的 live 配置文件。
- 恢复前必须生成当前 live 文件的保护性备份。
- 用户能看到某个 Provider 是否托管了 Codex `config.toml`、Codex `auth.json`、Claude `.claude.json` 或 API key 类密文。
- 用户能更新已托管凭据。
- 用户能删除已托管凭据；删除后切换逻辑不得写回该 secret 对应文件。
- SQLite 和日志不得保存或输出 token、Cookie、API key、auth 内容明文。
- 更新 `docs/project-context-for-codex.md`，去掉“Provider 只更新 SQLite”的过期描述。
- 保持 Windows 优先；DPAPI 仍作为 Windows 凭据保护方案。

## Acceptance Criteria

- [x] 后端提供列出备份、恢复备份、列出凭据状态、更新凭据、删除凭据的 Tauri command。
- [x] 前端 Provider 页面能触发上述能力并展示结果。
- [x] 恢复备份时会先备份当前 live 配置，再覆盖目标配置。
- [x] 凭据更新 / 删除只影响 `provider_secrets`，不把明文写入普通 Provider profile 字段。
- [x] `npm run build` 通过。
- [x] `cargo fmt` 通过。
- [x] `cargo check` 通过。
- [x] `cargo test` 通过或明确说明无测试。
- [x] 敏感字面量扫描无真实 token / API key / Cookie 形态命中。

## Notes

- 明确不做：切换失败诊断面板。
- 明确不做：临时目录 E2E 测试。
- 明确不做：真实账号 live apply 手测。
- 明确不做：明文展示已托管凭据内容。
