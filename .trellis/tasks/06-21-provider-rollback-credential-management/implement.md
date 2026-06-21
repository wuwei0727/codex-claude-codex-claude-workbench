# Provider 回滚与凭据管理 Implementation

## Checklist

- [x] Validate existing backup filename format and config item ids.
- [x] Add backup listing and restore helpers to `backup_service`.
- [x] Add Provider backup commands in `providers.rs`.
- [x] Add secret status / update / delete commands in `providers.rs`.
- [x] Register commands in `lib.rs`.
- [x] Add frontend types and invoke wrappers in `src/lib/tauri.ts`.
- [x] Add dashboard UI for backup restore and credential management.
- [x] Update `docs/project-context-for-codex.md`.
- [x] Run validation commands.
- [x] Update this checklist with results.

## Validation Plan

- `npm run build`
- `cargo fmt`
- `cargo check`
- `cargo test`
- Sensitive literal scan excluding `.git`, `node_modules`, `dist`, and `src-tauri/target`.

## Boundaries

- Do not add temporary-directory E2E.
- Do not add switch failure diagnostics panel.
- Do not run real account live apply manual test.
- Do not print decrypted secrets.

## Execution Notes

- Added backup listing and restore support. Restore only accepts `.bak` files inside the workbench backup directory and maps file names back to known config discovery item ids before writing.
- Added protective backup before restore when the target live config file currently exists.
- Added Provider secret status, update, and delete commands. Secret values are encrypted with the existing DPAPI service and are never returned to the frontend.
- Added dashboard UI for backup restore and per-provider credential management.
- Updated project context docs to remove the old "Provider only updates SQLite" limitation.
- Validation passed: `npm run build`, `cargo fmt`, `cargo check`, `cargo test`.
- Sensitive literal scan found no real token / API key / Cookie shaped matches.
