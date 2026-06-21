CREATE TABLE IF NOT EXISTS provider_profiles (
  id TEXT PRIMARY KEY,
  kind TEXT NOT NULL,
  name TEXT NOT NULL,
  mode TEXT NOT NULL,
  base_url TEXT,
  settings_config TEXT NOT NULL DEFAULT '{}',
  category TEXT,
  meta TEXT NOT NULL DEFAULT '{}',
  is_active INTEGER NOT NULL DEFAULT 0,
  health TEXT NOT NULL DEFAULT 'unknown',
  updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS app_settings (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS provider_secrets (
  provider_id TEXT NOT NULL,
  secret_type TEXT NOT NULL,
  encrypted_value BLOB NOT NULL,
  updated_at TEXT NOT NULL,
  PRIMARY KEY (provider_id, secret_type)
);

CREATE TABLE IF NOT EXISTS codex_sessions (
  id TEXT PRIMARY KEY,
  title TEXT NOT NULL,
  project_path TEXT,
  updated_at TEXT NOT NULL,
  message_count INTEGER NOT NULL DEFAULT 0
);
