import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  Activity,
  Bot,
  Boxes,
  CheckCircle2,
  CloudCog,
  Code2,
  Database,
  FileArchive,
  FolderCog,
  KeyRound,
  LayoutDashboard,
  Play,
  Search,
  Settings,
  ShieldCheck,
  Sparkles,
  TerminalSquare,
} from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import {
  applyProviderSwitch,
  createApiRelayProvider,
  getWorkbenchStatus,
  importLiveProvider,
  listCodexSessions,
  listProviderProfiles,
  previewProviderSwitch,
  type ProviderKind,
  type ProviderMutationResult,
  type ProviderSwitchApplyResult,
  type ProviderSwitchPreview,
} from "@/lib/tauri";
import { cn } from "@/lib/utils";

const navItems = [
  { label: "总览", icon: LayoutDashboard, active: true },
  { label: "Provider", icon: CloudCog },
  { label: "Codex+", icon: Sparkles },
  { label: "MCP / Skills", icon: Boxes },
  { label: "设置", icon: Settings },
];

const modules = [
  {
    title: "Provider 管理",
    desc: "Codex / Claude 官方登录与 API Relay 配置共存。",
    status: "核心模块",
    icon: KeyRound,
  },
  {
    title: "一键切换",
    desc: "主界面和托盘快速切换当前 Provider。",
    status: "核心模块",
    icon: Activity,
  },
  {
    title: "会话管理",
    desc: "Codex 会话搜索、Markdown 导出、删除与 Timeline。",
    status: "首版骨架",
    icon: FileArchive,
  },
  {
    title: "注入增强隔离",
    desc: "Codex++ 注入作为可选模块，失败不影响主功能。",
    status: "可关闭",
    icon: ShieldCheck,
  },
];

export function Dashboard() {
  const queryClient = useQueryClient();
  const [relayKind, setRelayKind] = useState<ProviderKind>("codex");
  const [relayName, setRelayName] = useState("");
  const [relayBaseUrl, setRelayBaseUrl] = useState("");
  const [relayModel, setRelayModel] = useState("");
  const [relayApiKey, setRelayApiKey] = useState("");
  const statusQuery = useQuery({ queryKey: ["workbench-status"], queryFn: getWorkbenchStatus });
  const providersQuery = useQuery({ queryKey: ["provider-profiles"], queryFn: listProviderProfiles });
  const sessionsQuery = useQuery({ queryKey: ["codex-sessions"], queryFn: listCodexSessions });
  const previewSwitchMutation = useMutation({ mutationFn: previewProviderSwitch });
  const importProviderMutation = useMutation({
    mutationFn: importLiveProvider,
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ["provider-profiles"] }),
  });
  const createRelayMutation = useMutation({
    mutationFn: createApiRelayProvider,
    onSuccess: () => {
      setRelayName("");
      setRelayBaseUrl("");
      setRelayModel("");
      setRelayApiKey("");
      queryClient.invalidateQueries({ queryKey: ["provider-profiles"] });
    },
  });
  const applySwitchMutation = useMutation({
    mutationFn: applyProviderSwitch,
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ["provider-profiles"] }),
  });

  const codexProvider = providersQuery.data?.find((item) => item.kind === "codex" && item.isActive);
  const claudeProvider = providersQuery.data?.find((item) => item.kind === "claude" && item.isActive);

  return (
    <div className="min-h-screen overflow-hidden bg-[radial-gradient(circle_at_top_left,_rgba(59,130,246,0.16),_transparent_32%),linear-gradient(135deg,_hsl(var(--background)),_hsl(var(--muted)))]">
      <div className="flex min-h-screen">
        <aside className="hidden w-72 border-r bg-white/55 p-4 backdrop-blur-xl dark:bg-slate-950/55 lg:block">
          <div className="mb-8 flex items-center gap-3 px-2">
            <div className="flex h-11 w-11 items-center justify-center rounded-2xl bg-primary text-primary-foreground shadow-lg shadow-primary/25">
              <TerminalSquare className="h-5 w-5" />
            </div>
            <div>
              <div className="font-semibold">Codex Claude</div>
              <div className="text-xs text-muted-foreground">Workbench</div>
            </div>
          </div>

          <nav className="space-y-1">
            {navItems.map((item) => (
              <button
                key={item.label}
                className={cn(
                  "flex w-full items-center gap-3 rounded-xl px-3 py-2.5 text-sm transition",
                  item.active ? "bg-primary text-primary-foreground shadow" : "text-muted-foreground hover:bg-accent hover:text-accent-foreground",
                )}
              >
                <item.icon className="h-4 w-4" />
                {item.label}
              </button>
            ))}
          </nav>

          <div className="mt-8 rounded-2xl border bg-card/80 p-4 shadow-sm">
            <div className="mb-2 flex items-center gap-2 text-sm font-medium">
              <CheckCircle2 className="h-4 w-4 text-emerald-500" />
              首版范围
            </div>
            <p className="text-xs leading-5 text-muted-foreground">先聚焦 Codex / Claude 日常 Provider 切换、配置备份、会话和 MCP 管理。</p>
          </div>
        </aside>

        <main className="flex-1 p-4 sm:p-6 lg:p-8">
          <header className="mb-8 flex flex-col gap-4 lg:flex-row lg:items-center lg:justify-between">
            <div>
              <Badge variant="secondary" className="mb-3">Windows 优先 · Tauri 2</Badge>
              <h1 className="text-3xl font-bold tracking-tight sm:text-4xl">Codex / Claude 日常工作台</h1>
              <p className="mt-2 max-w-2xl text-muted-foreground">统一管理 Provider、配置备份、MCP / Skills / Prompts 和 Codex 会话增强。</p>
            </div>
            <div className="flex flex-wrap gap-2">
              <Button variant="secondary"><Bot className="mr-2 h-4 w-4" />启动 Claude</Button>
              <Button><Code2 className="mr-2 h-4 w-4" />启动 Codex</Button>
            </div>
          </header>

          <section className="mb-6 grid gap-4 md:grid-cols-2 xl:grid-cols-4">
            <StatusCard title="Codex Provider" value={codexProvider?.name ?? "未配置"} hint={codexProvider?.mode ?? "等待添加"} />
            <StatusCard title="Claude Provider" value={claudeProvider?.name ?? "未配置"} hint={claudeProvider?.mode ?? "等待添加"} />
            <StatusCard title="SQLite" value="已预留" hint={statusQuery.data?.databasePath ?? "初始化后写入"} icon={Database} />
            <StatusCard title="注入模块" value={statusQuery.data?.injectorEnabled ? "已启用" : "默认关闭"} hint="独立增强，不影响主功能" icon={Sparkles} />
          </section>

          <section className="grid gap-6 xl:grid-cols-[1.35fr_0.9fr]">
            <Card className="bg-card/85 backdrop-blur">
              <CardHeader>
                <CardTitle>Provider 切换</CardTitle>
                <CardDescription>切换前可 dry-run 预览；应用切换会先备份，再写入 Codex / Claude 配置。</CardDescription>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="grid gap-3 lg:grid-cols-[0.9fr_1.1fr]">
                  <div className="rounded-2xl border bg-background/65 p-4">
                    <div className="mb-3 font-semibold">导入当前配置</div>
                    <p className="mb-4 text-sm leading-6 text-muted-foreground">从本机 live 配置创建 Provider；敏感字段会跳过，不写入 SQLite。</p>
                    <div className="grid gap-2 sm:grid-cols-2">
                      <Button
                        variant="outline"
                        disabled={importProviderMutation.isPending}
                        onClick={() => importProviderMutation.mutate("codex")}
                      >
                        导入 Codex
                      </Button>
                      <Button
                        variant="outline"
                        disabled={importProviderMutation.isPending}
                        onClick={() => importProviderMutation.mutate("claude")}
                      >
                        导入 Claude
                      </Button>
                    </div>
                  </div>

                  <div className="rounded-2xl border bg-background/65 p-4">
                    <div className="mb-3 flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
                      <div className="font-semibold">新增 API Relay</div>
                      <div className="flex rounded-xl border bg-card/70 p-1">
                        {(["codex", "claude"] as ProviderKind[]).map((kind) => (
                          <button
                            key={kind}
                            className={cn(
                              "rounded-lg px-3 py-1.5 text-xs font-medium transition",
                              relayKind === kind ? "bg-primary text-primary-foreground" : "text-muted-foreground hover:text-foreground",
                            )}
                            onClick={() => setRelayKind(kind)}
                          >
                            {kind === "codex" ? "Codex" : "Claude"}
                          </button>
                        ))}
                      </div>
                    </div>
                    <div className="grid gap-2 sm:grid-cols-2">
                      <Input value={relayName} onChange={(event) => setRelayName(event.target.value)} placeholder="Provider 名称" />
                      <Input value={relayModel} onChange={(event) => setRelayModel(event.target.value)} placeholder="模型，可选" />
                      <Input
                        value={relayBaseUrl}
                        onChange={(event) => setRelayBaseUrl(event.target.value)}
                        placeholder="https://relay.example.com/v1"
                        className="sm:col-span-2"
                      />
                      <Input
                        value={relayApiKey}
                        onChange={(event) => setRelayApiKey(event.target.value)}
                        placeholder="API key，可选，加密保存"
                        type="password"
                        className="sm:col-span-2"
                      />
                    </div>
                    <Button
                      className="mt-3 w-full"
                      disabled={createRelayMutation.isPending}
                      onClick={() =>
                        createRelayMutation.mutate({
                          kind: relayKind,
                          name: relayName,
                          baseUrl: relayBaseUrl,
                          model: relayModel || undefined,
                          apiKey: relayApiKey || undefined,
                        })
                      }
                    >
                      保存 Provider
                    </Button>
                  </div>
                </div>

                {importProviderMutation.data && <ProviderMutationPanel result={importProviderMutation.data} title="导入结果" />}
                {createRelayMutation.data && <ProviderMutationPanel result={createRelayMutation.data} title="新增结果" />}
                {importProviderMutation.isError && (
                  <div className="rounded-xl border border-destructive/30 bg-destructive/10 p-3 text-sm text-destructive">
                    导入失败：{String(importProviderMutation.error)}
                  </div>
                )}
                {createRelayMutation.isError && (
                  <div className="rounded-xl border border-destructive/30 bg-destructive/10 p-3 text-sm text-destructive">
                    新增失败：{String(createRelayMutation.error)}
                  </div>
                )}

                <div className="grid gap-3 md:grid-cols-2">
                  {(providersQuery.data ?? []).map((profile) => (
                    <div key={profile.id} className="rounded-2xl border bg-background/65 p-4">
                      <div className="mb-4 flex items-start justify-between gap-3">
                        <div className="flex h-10 w-10 items-center justify-center rounded-xl bg-primary/10 text-primary">
                          {profile.kind === "codex" ? <Code2 className="h-5 w-5" /> : <Bot className="h-5 w-5" />}
                        </div>
                        <div className="flex flex-col items-end gap-1">
                          <Badge variant={profile.isActive ? "success" : "secondary"}>{profile.isActive ? "当前启用" : profile.mode}</Badge>
                          <Badge variant={profile.hasLiveConfig ? "warning" : "secondary"}>{profile.hasLiveConfig ? "可写入配置" : "状态占位"}</Badge>
                        </div>
                      </div>
                      <h3 className="font-semibold">{profile.name}</h3>
                      <p className="mt-2 text-sm leading-6 text-muted-foreground">
                        {profile.kind === "codex" ? "Codex" : "Claude"} · {profile.baseUrl ?? "官方配置"} · {profile.health}
                      </p>
                      <div className="mt-4 grid gap-2 sm:grid-cols-2">
                        <Button
                          variant="outline"
                          disabled={previewSwitchMutation.isPending}
                          onClick={() => previewSwitchMutation.mutate(profile.id)}
                        >
                          预览写入
                        </Button>
                        <Button
                          variant={profile.isActive ? "secondary" : "outline"}
                          disabled={profile.isActive || applySwitchMutation.isPending}
                          onClick={() => applySwitchMutation.mutate(profile.id)}
                        >
                          {profile.isActive ? "已启用" : "应用切换"}
                        </Button>
                      </div>
                    </div>
                  ))}
                </div>
                {previewSwitchMutation.data && <ProviderSwitchPreviewPanel preview={previewSwitchMutation.data} />}
                {applySwitchMutation.data && <ProviderSwitchApplyPanel result={applySwitchMutation.data} />}
                {previewSwitchMutation.isError && (
                  <div className="rounded-xl border border-destructive/30 bg-destructive/10 p-3 text-sm text-destructive">
                    预览失败：{String(previewSwitchMutation.error)}
                  </div>
                )}
                {applySwitchMutation.isError && (
                  <div className="rounded-xl border border-destructive/30 bg-destructive/10 p-3 text-sm text-destructive">
                    切换失败：{String(applySwitchMutation.error)}
                  </div>
                )}
              </CardContent>
            </Card>

            <Card className="bg-card/85 backdrop-blur">
              <CardHeader>
                <div className="flex items-center justify-between gap-3">
                  <div>
                    <CardTitle>Codex 会话</CardTitle>
                    <CardDescription>后续接入本地会话解析、导出和删除。</CardDescription>
                  </div>
                  <Button variant="outline" size="sm"><Search className="mr-2 h-4 w-4" />搜索</Button>
                </div>
              </CardHeader>
              <CardContent>
                <Input placeholder="搜索会话标题或项目路径" className="mb-4" />
                <div className="space-y-3">
                  {(sessionsQuery.data ?? []).map((session) => (
                    <div key={session.id} className="rounded-xl border bg-background/65 p-3">
                      <div className="font-medium">{session.title}</div>
                      <div className="mt-1 text-xs text-muted-foreground">{session.projectPath ?? "未绑定项目"} · {session.messageCount} 条消息</div>
                    </div>
                  ))}
                  {sessionsQuery.data?.length === 0 && (
                    <div className="rounded-xl border border-dashed p-6 text-center text-sm text-muted-foreground">暂无会话数据，当前为服务骨架。</div>
                  )}
                </div>
              </CardContent>
            </Card>
          </section>

          <section className="mt-6">
            <Card className="bg-card/85 backdrop-blur">
              <CardHeader>
                <CardTitle>核心功能路线</CardTitle>
                <CardDescription>按日常 Codex / Claude 使用频率排序，先做最有价值的部分。</CardDescription>
              </CardHeader>
              <CardContent className="grid gap-3 md:grid-cols-2 xl:grid-cols-4">
                {modules.map((module) => (
                  <div key={module.title} className="rounded-2xl border bg-background/65 p-4">
                    <div className="mb-4 flex items-start justify-between gap-3">
                      <div className="flex h-10 w-10 items-center justify-center rounded-xl bg-primary/10 text-primary">
                        <module.icon className="h-5 w-5" />
                      </div>
                      <Badge variant={module.status === "核心模块" ? "success" : "secondary"}>{module.status}</Badge>
                    </div>
                    <h3 className="font-semibold">{module.title}</h3>
                    <p className="mt-2 text-sm leading-6 text-muted-foreground">{module.desc}</p>
                  </div>
                ))}
              </CardContent>
            </Card>
          </section>

          <section className="mt-6 grid gap-6 xl:grid-cols-3">
            <Card className="xl:col-span-2 bg-card/85 backdrop-blur">
              <CardHeader>
                <CardTitle>配置路径</CardTitle>
                <CardDescription>只显示路径，不展示敏感 token / auth 内容。</CardDescription>
              </CardHeader>
              <CardContent className="grid gap-3 text-sm">
                <PathRow label="App Data" value={statusQuery.data?.appDataDir} />
                <PathRow label="Codex config" value={statusQuery.data?.codexConfigPath} />
                <PathRow label="Claude config" value={statusQuery.data?.claudeConfigPath} />
              </CardContent>
            </Card>

            <Card className="bg-card/85 backdrop-blur">
              <CardHeader>
                <CardTitle>快速动作</CardTitle>
                <CardDescription>后续接入 Tauri 命令。</CardDescription>
              </CardHeader>
              <CardContent className="space-y-2">
                <Button variant="outline" className="w-full justify-start"><FolderCog className="mr-2 h-4 w-4" />打开数据目录</Button>
                <Button variant="outline" className="w-full justify-start"><FileArchive className="mr-2 h-4 w-4" />立即备份配置</Button>
                <Button variant="outline" className="w-full justify-start"><Play className="mr-2 h-4 w-4" />运行健康检查</Button>
              </CardContent>
            </Card>
          </section>
        </main>
      </div>
    </div>
  );
}

function StatusCard({ title, value, hint, icon: Icon = CheckCircle2 }: { title: string; value: string; hint: string; icon?: typeof CheckCircle2 }) {
  return (
    <Card className="bg-card/85 backdrop-blur">
      <CardContent className="p-5">
        <div className="mb-4 flex items-center justify-between">
          <div className="text-sm text-muted-foreground">{title}</div>
          <Icon className="h-4 w-4 text-primary" />
        </div>
        <div className="truncate text-xl font-semibold">{value}</div>
        <div className="mt-1 truncate text-xs text-muted-foreground">{hint}</div>
      </CardContent>
    </Card>
  );
}

function PathRow({ label, value }: { label: string; value?: string }) {
  return (
    <div className="grid gap-1 rounded-xl border bg-background/65 p-3 sm:grid-cols-[140px_1fr] sm:items-center">
      <div className="text-muted-foreground">{label}</div>
      <code className="truncate text-xs">{value ?? "加载中..."}</code>
    </div>
  );
}

function ProviderSwitchPreviewPanel({ preview }: { preview: ProviderSwitchPreview }) {
  return (
    <div className="rounded-2xl border bg-background/65 p-4">
      <div className="mb-4 flex flex-col gap-2 sm:flex-row sm:items-start sm:justify-between">
        <div>
          <div className="text-sm text-muted-foreground">Dry-run 预览</div>
          <h3 className="font-semibold">{preview.profileName}</h3>
        </div>
        <Badge variant={preview.willWrite ? "warning" : "secondary"}>{preview.willWrite ? "会写入" : "不会写入"}</Badge>
      </div>

      <div className="mb-4 grid gap-3 lg:grid-cols-2">
        {preview.discoveredConfigs.map((item) => (
          <div key={item.id} className="rounded-xl border bg-card/70 p-3">
            <div className="mb-1 flex items-center justify-between gap-3">
              <div className="font-medium">{item.label}</div>
              <Badge variant={item.exists ? "success" : "secondary"}>{item.exists ? "已发现" : "未发现"}</Badge>
            </div>
            <code className="block truncate text-xs text-muted-foreground">{item.path}</code>
            <div className="mt-2 text-xs leading-5 text-muted-foreground">
              {item.structureSummary} · {item.sensitiveState}
            </div>
          </div>
        ))}
      </div>

      <div className="grid gap-3 lg:grid-cols-2">
        <div className="rounded-xl border bg-card/70 p-3">
          <div className="mb-2 font-medium">计划改动</div>
          <div className="space-y-2">
            {preview.changes.map((change) => (
              <div key={`${change.target}-${change.field}`} className="text-xs leading-5 text-muted-foreground">
                <span className="font-medium text-foreground">{change.target}</span> · {change.field} · {change.action}
              </div>
            ))}
          </div>
        </div>

        <div className="rounded-xl border bg-card/70 p-3">
          <div className="mb-2 font-medium">备份计划</div>
          <code className="block truncate text-xs text-muted-foreground">{preview.backupPlan.backupDir}</code>
          <div className="mt-2 space-y-2">
            {preview.backupPlan.targets.length > 0 ? (
              preview.backupPlan.targets.map((target) => (
                <div key={target.plannedFileName} className="text-xs leading-5 text-muted-foreground">
                  {target.label} · {target.plannedFileName}
                </div>
              ))
            ) : (
              <div className="text-xs text-muted-foreground">当前没有需要备份的已存在配置文件。</div>
            )}
          </div>
        </div>
      </div>

      <div className="mt-3 space-y-1 text-xs text-muted-foreground">
        {preview.warnings.map((warning) => (
          <div key={warning}>{warning}</div>
        ))}
      </div>
    </div>
  );
}

function ProviderSwitchApplyPanel({ result }: { result: ProviderSwitchApplyResult }) {
  return (
    <div className="rounded-2xl border border-emerald-500/30 bg-emerald-500/10 p-4">
      <div className="mb-3 flex flex-col gap-2 sm:flex-row sm:items-start sm:justify-between">
        <div>
          <div className="text-sm text-muted-foreground">应用结果</div>
          <h3 className="font-semibold">{result.profileName}</h3>
        </div>
        <Badge variant="success">已完成</Badge>
      </div>

      <div className="grid gap-3 lg:grid-cols-2">
        <div className="rounded-xl border bg-card/70 p-3">
          <div className="mb-2 font-medium">备份文件</div>
          <div className="space-y-2">
            {result.backupResults.length > 0 ? (
              result.backupResults.map((backup) => (
                <code key={backup.backupPath} className="block truncate text-xs text-muted-foreground">
                  {backup.backupPath}
                </code>
              ))
            ) : (
              <div className="text-xs text-muted-foreground">没有发现需要备份的已存在配置文件。</div>
            )}
          </div>
        </div>

        <div className="rounded-xl border bg-card/70 p-3">
          <div className="mb-2 font-medium">写入文件</div>
          <div className="space-y-2">
            {result.writtenPaths.length > 0 ? (
              result.writtenPaths.map((path) => (
                <code key={path} className="block truncate text-xs text-muted-foreground">
                  {path}
                </code>
              ))
            ) : (
              <div className="text-xs text-muted-foreground">该 Provider 未携带 live config，未写入真实配置文件。</div>
            )}
          </div>
        </div>
      </div>

      {result.warnings.length > 0 && (
        <div className="mt-3 space-y-1 text-xs text-muted-foreground">
          {result.warnings.map((warning) => (
            <div key={warning}>{warning}</div>
          ))}
        </div>
      )}
    </div>
  );
}

function ProviderMutationPanel({ result, title }: { result: ProviderMutationResult; title: string }) {
  return (
    <div className="rounded-2xl border bg-background/65 p-4">
      <div className="mb-2 flex flex-col gap-2 sm:flex-row sm:items-start sm:justify-between">
        <div>
          <div className="text-sm text-muted-foreground">{title}</div>
          <h3 className="font-semibold">{result.profileName}</h3>
        </div>
        <Badge variant="success">已保存</Badge>
      </div>
      <code className="block truncate text-xs text-muted-foreground">{result.profileId}</code>
      {result.warnings.length > 0 && (
        <div className="mt-3 space-y-1 text-xs text-muted-foreground">
          {result.warnings.map((warning) => (
            <div key={warning}>{warning}</div>
          ))}
        </div>
      )}
    </div>
  );
}
