# Agent 指令

## 命令
- Rust crate 根目录是 `mls-sim-rs/`；Cargo 命令在该目录执行，不在仓库根目录执行。
- 检查：`cargo check`
- 构建：`cargo build --release`
- 运行：`cargo run -- --script-dir "D:/path/to/script"`
- 运行后健康检查：`curl http://127.0.0.1:5000/api/health`

## 架构
- 应用入口：`mls-sim-rs/src/main.rs`。
- 原生 GUI 在 `mls-sim-rs/src/gui/`。
- Bridge HTTP 接口在 `mls-sim-rs/src/bridge.rs`，用于客户端登录、发送事件、轮询事件和健康检查。
- 房间和 Lua 运行时逻辑在 `mls-sim-rs/src/room/`。
- `参考/mls-master/` 是平台参考资料，不是模拟器源码。

## 提交署名
AI 提交必须包含：

    Co-Authored-By: <model name> <noreply@provider-domain>

## Git 远端
- `origin`：个人 Fork，用于日常开发。
- `upstream`：原始仓库；上游贡献分支从 `upstream/main` 新建，再 cherry-pick 或移植适合的变更。

## 文档治理
- 治理标记：`docs-governance: managed`。该精确文本必须保留，不能改成中文冒号、同义表达或“已托管”。
- 初始化或维护项目级 docs 体系时使用全局 `docs-governance` skill。
- 项目存在 `docs/standards/README.md`、`docs/templates/README.md`、`docs/governance/project-docs-profile.md` 等治理标记时，正式 docs 任务必须先走 `docs-governance`。
- 初始化 `docs/standards/` 与 `docs/templates/` 时，必须以 `docs-governance` skill 内建资产为基线落盘；只能替换项目字段，不能重写章节、表头、模板骨架或加入未登记历史样式。
- 正式 docs 默认先遵守项目内 `docs/standards/` 与 `docs/templates/`；项目内缺失或漂移时先用 `docs-governance` skill 内建规范补齐治理层；只有两者都未覆盖时才回退到 `doc-writing`。
- 代码、配置、脚本、接口、UI、行为、目录结构、测试口径或计划状态变化后，收尾前必须做 docs impact review，并同步检查当前正文、所在目录 `README.md`、`docs/README.md`、`docs/standards/*.md`、`docs/templates/*.md`、计划状态与归档索引、API 文档与接口变更记录、用户指南和 `docs/llm`。
- 启用 `docs/plans/` 后，新建计划必须使用 `NN-中文计划名.md`，按当前和归档最大编号递增；归档必须使用 `YYYYMMDD-NN-中文计划名.md`，先回填长期事实，再同步 `docs/plans/README.md` 和 `docs/plans/archive/README.md`。

## 本地文件
- 本机笔记放在 `.local/`。
- 不提交 `.vscode/`、`.env`、`mls-sim-rs/config.json`、`archives/`、`mls-sim-rs/archives/`。
- 共享配置结构变化时，提交 `mls-sim-rs/config.example.json`。
