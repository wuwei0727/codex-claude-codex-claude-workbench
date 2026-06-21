# Provider 回滚与凭据管理 Design

## Scope

This task extends the existing Provider management flow. It reuses:

- `backup_service` for backup planning and file copies.
- `credential_service` for Windows DPAPI protect / unprotect.
- `provider_secrets` for encrypted secret storage.
- `providers.rs` as the Tauri command boundary.
- `src/lib/tauri.ts` as the frontend RPC contract.
- `dashboard.tsx` as the first UI surface.

## Non-Goals

- No switch failure diagnostics panel.
- No temporary-directory E2E.
- No real account live apply manual test.
- No plaintext token / auth / API key display.

## Backend Contract

Add commands:

- `list_provider_backups() -> Vec<ProviderBackupEntry>`
- `restore_provider_backup(backup_path: String) -> ProviderBackupRestoreResult`
- `list_provider_secret_status(profile_id: String) -> Vec<ProviderSecretStatus>`
- `update_provider_secret(profile_id: String, secret_type: String, secret_value: String) -> ProviderSecretMutationResult`
- `delete_provider_secret(profile_id: String, secret_type: String) -> ProviderSecretMutationResult`

`ProviderBackupEntry` should expose safe metadata only:

- `backup_path`
- `target_id`
- `target_label`
- `source_path`
- `created_at`
- `exists`
- `size_bytes`

`ProviderSecretStatus` should expose:

- `secret_type`
- `label`
- `exists`
- `updated_at`

## Backup Restore

Current backup names include the config item id and timestamp. Restore should:

1. Parse backup filename into target config item id.
2. Resolve that id through `config_discovery::discover_config_items`.
3. Ensure both backup file and target path exist or target parent can be created.
4. Create a protective backup of the current target before overwriting.
5. Copy the selected backup to the target path.

If the backup filename cannot map to a known target id, return a clear error without writing.

## Secret Management

Use the same `provider_secrets` table. The UI must never receive decrypted secret values.

Supported secret types:

- `codex_config_toml`
- `codex_auth_json`
- `claude_config_json`

API Relay providers already store API keys by reusing these full config secret slots. For this task, the update form will allow replacing a supported secret payload as text. The backend protects it with DPAPI before writing.

Deletion should remove the specific `(provider_id, secret_type)` row.

## Frontend UX

Add a compact Provider maintenance section:

- Backup list with restore buttons.
- Per-provider secret status rows.
- Secret update form using password / textarea input.
- Delete buttons for existing secrets.

The UI should avoid large marketing panels and keep the dense dashboard style.

## Documentation

Update `docs/project-context-for-codex.md` to reflect that Provider switching now has real config write support, DPAPI secret storage, and backup / restore management.
