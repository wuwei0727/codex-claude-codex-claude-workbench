import { invoke } from "@tauri-apps/api/core";

export type ProviderKind = "codex" | "claude";

export interface ProviderProfile {
  id: string;
  kind: ProviderKind;
  name: string;
  mode: "official" | "api-relay";
  baseUrl?: string;
  category?: string;
  meta: Record<string, unknown>;
  hasLiveConfig: boolean;
  isActive: boolean;
  health: "unknown" | "ok" | "warning" | "error";
  updatedAt: string;
}

export interface WorkbenchStatus {
  appDataDir: string;
  codexConfigPath: string;
  claudeConfigPath: string;
  databasePath: string;
  injectorEnabled: boolean;
}

export interface CodexSession {
  id: string;
  title: string;
  projectPath?: string;
  updatedAt: string;
  messageCount: number;
}

export interface ConfigDiscoveryItem {
  id: string;
  label: string;
  kind: string;
  path: string;
  exists: boolean;
  isDirectory: boolean;
  sizeBytes?: number;
  structureSummary: string;
  sensitiveState: string;
}

export interface BackupPlan {
  backupDir: string;
  required: boolean;
  targets: Array<{
    label: string;
    sourcePath: string;
    plannedFileName: string;
  }>;
}

export interface ProviderSwitchPreview {
  profileId: string;
  profileName: string;
  providerKind: ProviderKind;
  mode: "official" | "api-relay";
  willWrite: boolean;
  discoveredConfigs: ConfigDiscoveryItem[];
  backupPlan: BackupPlan;
  changes: Array<{
    target: string;
    path: string;
    field: string;
    action: string;
    sensitiveValue: boolean;
  }>;
  warnings: string[];
}

export interface ProviderSwitchApplyResult {
  profileId: string;
  profileName: string;
  providerKind: ProviderKind;
  backupResults: Array<{
    label: string;
    sourcePath: string;
    backupPath: string;
  }>;
  writtenPaths: string[];
  warnings: string[];
}

export interface ProviderMutationResult {
  profileId: string;
  profileName: string;
  providerKind: ProviderKind;
  warnings: string[];
}

export interface CreateApiRelayProviderRequest {
  kind: ProviderKind;
  name: string;
  baseUrl: string;
  model?: string;
  apiKey?: string;
}

function hasTauriRuntime() {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

export async function getWorkbenchStatus() {
  if (!hasTauriRuntime()) {
    return {
      appDataDir: "%APPDATA%\\codex-claude-workbench",
      codexConfigPath: "%USERPROFILE%\\.codex\\config.toml",
      claudeConfigPath: "%USERPROFILE%\\.claude.json",
      databasePath: "%APPDATA%\\codex-claude-workbench\\workbench.sqlite",
      injectorEnabled: false,
    } satisfies WorkbenchStatus;
  }

  return invoke<WorkbenchStatus>("get_workbench_status");
}

export async function listProviderProfiles() {
  if (!hasTauriRuntime()) {
    return [
      {
        id: "codex-official",
        kind: "codex",
        name: "Codex Official",
        mode: "official",
        category: "official",
        meta: { liveConfigManaged: false },
        hasLiveConfig: false,
        isActive: true,
        health: "unknown",
        updatedAt: new Date().toISOString(),
      },
      {
        id: "claude-official",
        kind: "claude",
        name: "Claude Official",
        mode: "official",
        category: "official",
        meta: { liveConfigManaged: false },
        hasLiveConfig: false,
        isActive: true,
        health: "unknown",
        updatedAt: new Date().toISOString(),
      },
    ] satisfies ProviderProfile[];
  }

  return invoke<ProviderProfile[]>("list_provider_profiles");
}

export async function switchProviderProfile(profileId: string) {
  if (!hasTauriRuntime()) {
    console.info("Mock switch provider", profileId);
    return;
  }

  return invoke<void>("switch_provider_profile", { profileId });
}

export async function applyProviderSwitch(profileId: string) {
  if (!hasTauriRuntime()) {
    const kind: ProviderKind = profileId.startsWith("claude") ? "claude" : "codex";
    return {
      profileId,
      profileName: profileId === "claude-official" ? "Claude Official" : "Codex Official",
      providerKind: kind,
      backupResults: [],
      writtenPaths: [],
      warnings: ["浏览器预览模式不会写入真实配置"],
    } satisfies ProviderSwitchApplyResult;
  }

  return invoke<ProviderSwitchApplyResult>("apply_provider_switch", { profileId });
}

export async function importLiveProvider(kind: ProviderKind) {
  if (!hasTauriRuntime()) {
    return {
      profileId: `${kind}-live-mock`,
      profileName: kind === "codex" ? "Codex Live Import" : "Claude Live Import",
      providerKind: kind,
      warnings: ["浏览器预览模式不会读取本机配置"],
    } satisfies ProviderMutationResult;
  }

  return invoke<ProviderMutationResult>("import_live_provider", { kind });
}

export async function createApiRelayProvider(request: CreateApiRelayProviderRequest) {
  if (!hasTauriRuntime()) {
    return {
      profileId: `${request.kind}-relay-mock`,
      profileName: request.name,
      providerKind: request.kind,
      warnings: ["浏览器预览模式不会写入 SQLite"],
    } satisfies ProviderMutationResult;
  }

  return invoke<ProviderMutationResult>("create_api_relay_provider", { request });
}

export async function previewProviderSwitch(profileId: string) {
  if (!hasTauriRuntime()) {
    const kind: ProviderKind = profileId.startsWith("claude") ? "claude" : "codex";
    const configPath = kind === "codex" ? "%USERPROFILE%\\.codex\\config.toml" : "%USERPROFILE%\\.claude.json";

    return {
      profileId,
      profileName: profileId === "claude-official" ? "Claude Official" : "Codex Official",
      providerKind: kind,
      mode: "official",
      willWrite: false,
      discoveredConfigs: [
        {
          id: kind === "codex" ? "codex-config" : "claude-config",
          label: kind === "codex" ? "Codex config.toml" : "Claude .claude.json",
          kind: kind === "codex" ? "toml" : "json",
          path: configPath,
          exists: false,
          isDirectory: false,
          structureSummary: "浏览器预览 mock，未读取本机配置",
          sensitiveState: "未返回敏感内容",
        },
      ],
      backupPlan: {
        backupDir: "%APPDATA%\\codex-claude-workbench\\backups",
        required: false,
        targets: [],
      },
      changes: [
        {
          target: kind === "codex" ? "Codex config.toml" : "Claude .claude.json",
          path: configPath,
          field: "provider selection",
          action: "dry-run 预览，不实际写入",
          sensitiveValue: false,
        },
        {
          target: "Workbench SQLite",
          path: "provider_profiles",
          field: "is_active",
          action: "预览更新工作台内部 active Provider 状态",
          sensitiveValue: false,
        },
      ],
      warnings: ["浏览器预览模式未读取本机文件", "Tauri 运行环境会执行真实备份和写入"],
    } satisfies ProviderSwitchPreview;
  }

  return invoke<ProviderSwitchPreview>("preview_provider_switch", { profileId });
}

export async function listCodexSessions() {
  if (!hasTauriRuntime()) {
    return [] satisfies CodexSession[];
  }

  return invoke<CodexSession[]>("list_codex_sessions");
}
