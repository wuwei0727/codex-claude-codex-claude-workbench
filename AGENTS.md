# 项目规则

- 默认中文输出。
- 本项目定位为 Windows 优先的 Codex / Claude 桌面工作台。
- 不把 token、Cookie、auth.json 内容明文写入 SQLite 或日志。
- Codex++ 注入能力必须保持可关闭、可诊断、可降级，不能影响 Provider 管理等核心功能。
- 优先最小改动，不做无关重构。
