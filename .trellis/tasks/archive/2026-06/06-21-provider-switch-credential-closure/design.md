# Design: Provider 完整切换与凭据托管闭环

## Scope

本设计覆盖 Provider 主链路：

- Provider schema
- live 配置导入
- API Relay Provider 创建
- Windows DPAPI 凭据托管
- dry-run 预览
- 自动备份
- atomic write
- apply 切换
- 前端状态展示

不覆盖：

- MCP / Skills / Prompts 管理
- Codex 会话搜索 / 导出 / 删除
- Windows 托盘
- Codex++ 注入
- 多工具扩展到 Gemini / OpenCode / Hermes

## Data Model

`provider_profiles`

- `id`: Provider 唯一 ID
- `kind`: `codex` / `claude`
- `name`: 展示名称
- `mode`: `official` / `api-relay`
- `base_url`: Relay URL，可为空
- `settings_config`: 非敏感配置 JSON；可以包含脱敏结构、TOML 片段、导入来源等
- `category`: `official` / `custom` / `imported`
- `meta`: 管理元数据，例如 `liveConfigManaged`、`secretStorage`
- `is_active`: 当前激活状态
- `health`: 健康状态
- `updated_at`: 更新时间

`provider_secrets`

- `provider_id`: 对应 Provider
- `secret_type`: 例如 `codex_config_toml`、`codex_auth_json`、`claude_config_json`
- `encrypted_value`: Windows DPAPI 密文
- `updated_at`: 更新时间

普通 SQLite 字段不得保存明文密钥。需要完整还原 live 配置的内容放入 `provider_secrets`。

## Credential Storage

使用 Windows DPAPI：

- 加密：`CryptProtectData`
- 解密：`CryptUnprotectData`
- 范围：当前 Windows 用户上下文

优点：

- 不需要项目自管 master key
- 不把明文 token/API key/auth 内容落入 SQLite
- 符合 Windows 优先定位

限制：

- 换机器或换 Windows 用户后无法直接解密
- 数据库备份只携带密文，不等同完整可迁移凭据

## Codex Flow

### Import

1. 读取 `%USERPROFILE%\.codex\config.toml`。
2. 读取 `%USERPROFILE%\.codex\auth.json`，如果存在。
3. 普通字段保存脱敏摘要和非敏感提示。
4. 完整 `config.toml` 进入 `provider_secrets.codex_config_toml`。
5. 完整 `auth.json` 进入 `provider_secrets.codex_auth_json`。

### Apply

1. 生成备份计划，覆盖 `config.toml` 和 `auth.json`。
2. 执行备份。
3. 解密 Provider secret。
4. 如果有 `codex_auth_json`，写回 `auth.json`。
5. 写回 `config.toml`。
6. 写入成功后更新 SQLite active 状态。

写入 `auth.json` 和 `config.toml` 时，如果后续步骤失败，需要尽量保留备份并避免 active 状态错误更新。

## Claude Flow

### Import

1. 读取 `%USERPROFILE%\.claude.json`。
2. 普通字段保存脱敏结构。
3. 完整 JSON 进入 `provider_secrets.claude_config_json`。

### Apply

1. 生成备份计划，覆盖 `.claude.json`。
2. 执行备份。
3. 解密 `claude_config_json`。
4. 写回 `.claude.json`。
5. 写入成功后更新 SQLite active 状态。

## API Relay Provider

Codex API Relay：

- 普通字段保存 `base_url`、`model`、脱敏配置片段。
- 如果填写 API key，完整 TOML 写入 DPAPI secret。
- 应用时优先使用 secret 生成的完整 TOML。

Claude API Relay：

- 普通字段保存 `ANTHROPIC_BASE_URL`、可选 model。
- 如果填写 API key，完整 JSON 写入 DPAPI secret。
- 应用时优先使用 secret JSON。

## Frontend UX

Provider 区域需要展示：

- 导入当前 Codex
- 导入当前 Claude
- 新增 API Relay
- 预览写入
- 应用切换
- 备份结果
- 写入结果
- 警告和错误

前端只能展示“已托管凭据 / 未托管凭据”等状态，不显示明文密钥。

## Error Handling

原则：

- 备份失败则不得写入真实配置。
- 写入失败则不得更新 active 状态。
- 解密失败要返回明确错误，例如“当前 Windows 用户无法解密该 Provider 凭据”。
- JSON/TOML 解析失败要返回配置路径和原因，但不能包含敏感内容。

## Compatibility

- 当前实现 Windows 优先，DPAPI 非 Windows 平台返回不支持。
- 后续如果要跨平台，需要抽象 credential backend，例如 macOS Keychain / Linux Secret Service。
- 当前项目不是 git 仓库，不能依赖 git diff 或 commit 验证。

## Rollback

第一层回滚：

- 每次 apply 前备份真实配置。
- apply 失败时保留备份路径并返回给前端。

后续可选增强：

- 新增一键恢复备份命令。
- 在 apply 过程中出现部分写入失败时，自动用本次备份回滚。
