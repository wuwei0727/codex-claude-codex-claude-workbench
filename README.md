# Codex Claude Workbench

Windows 优先的 Codex / Claude 日常工作台。

## 技术栈

- Tauri 2
- React + TypeScript + Vite
- shadcn/ui 风格组件 + TailwindCSS
- TanStack Query
- Rust 后端
- SQLite
- TOML / JSON / SQLite 配置管理
- Windows NSIS / MSI 打包

## 首版目标

- Provider 管理：Codex / Claude 配置占位与切换流程
- 会话管理：Codex 会话搜索、导出、删除的模块骨架
- MCP / Skills / Prompts：后续模块预留
- 设置：数据目录、自动备份、主题、注入模块开关
- Codex++ 注入功能隔离：作为可选增强模块，不影响主功能

## 开发命令

```bash
npm install
npm run dev
npm run tauri:dev
npm run build
npm run tauri:build
```
