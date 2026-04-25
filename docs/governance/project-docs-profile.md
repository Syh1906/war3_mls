# 项目 docs 画像

> Parent: [README.md](README.md)

## 元信息

| 项目 | 值 |
| --- | --- |
| 项目名称 | MLS 云脚本本地模拟器 |
| 默认文档语言 | 中文 |
| 当前状态 | 已启用 |
| 首次建立 | 2026-04-25 |
| 最后更新 | 2026-04-25 |

## 项目定位

- 项目一句话定位：用于本地模拟 MLS 云脚本运行环境的 Rust 工具。
- 正式 docs 主要服务读者：云脚本作者、模拟器维护者、客户端对接者、Agent。
- 当前 docs 治理目标：同时维护项目使用文档和个人 Fork 的开发治理体系。

## docs 目录规划

| 目录 | 作用 | 是否启用 |
| --- | --- | --- |
| `docs/README.md` | 总导航 | 是 |
| `docs/项目总览.md` | 项目定位、边界和真相源 | 是 |
| `docs/user-guide/` | 使用指南和常见问题 | 是 |
| `docs/architecture/` | 架构和环境差异 | 是 |
| `docs/api/` | HTTP 接口和 Lua API | 是 |
| `docs/client/` | 客户端对接 | 是 |
| `docs/governance/` | 治理画像与治理入口 | 是 |
| `docs/standards/` | 项目内规范 | 是 |
| `docs/templates/` | 项目内模板 | 是 |
| `docs/plans/` | 计划与归档 | 是 |
| `docs/llm/` | 轻量知识与约束 | 是 |

## 真相源优先级

| 层级 | 来源 | 负责内容 |
| --- | --- | --- |
| 1 | `mls-sim-rs/src/`、`mls-sim-rs/web/` | 运行行为、接口、状态、页面功能 |
| 2 | `mls-sim-rs/config.example.json` | 配置示例和字段形态 |
| 3 | `参考/mls-master/` | 平台 MLS API 和 demo 行为参考 |
| 4 | `docs/` | 长期说明、指南、规范和计划 |

## README 规则

- `docs/` 体系 README 默认采用导航型。
- 导航型 README 固定包含“给 LLM 的快速索引”和“给人类的导航”。
- 代码目录 README 是否采用目录索引型：按需启用。
- 当前稳定 README 样式：`docs/README.md`、`docs/**/README.md`。

## 文档类型与同步点

| 文档类型 | 是否启用 | 主要同步点 |
| --- | --- | --- |
| README 导航 | 是 | 子文档增删改、目录迁移 |
| 长期说明 | 是 | 架构、目录职责、运行语义变化 |
| API 文档 | 是 | HTTP 路由、请求响应、Lua API 行为变化 |
| 用户指南 | 是 | 编译、运行、配置、Web 面板流程变化 |
| 计划文档 | 是 | 阶段、验收、归档状态变化 |
| LLM 知识条目 | 是 | 稳定约束、易错点、维护规则沉淀 |

## 风格约束

- 稳定表头：`文件 / 作用`、`目录 / 作用 / 是否启用`、`任务 / 入口`、`日期 / 变更`。
- 禁词：简单、显然、强大、优雅、完美、深入探讨、赋能、全面。
- 术语统一：MLS、云脚本、模拟器、房间、玩家、Bridge、docs impact review。
- 图示最低要求：新增长期说明或大幅重写时，前半段至少有一个 ASCII 图、流程图或结构矩阵。

## docs impact review

以下改动必须触发 docs impact review：代码、配置、脚本、接口、UI、行为、目录结构、测试口径、计划状态。

命中后至少检查：当前正文、所在目录 `README.md`、`docs/README.md`、`docs/standards/`、`docs/templates/`、计划状态与归档索引、API 文档、用户指南、`docs/governance/` 和 `docs/llm/`。

## 与 skill 内建规范的对齐策略

- 必须强对齐：治理标记、README 导航结构、项目画像、模板入口、同步门禁。
- 可以项目化改写：目录名、术语、文档域、示例命令、真相源。
- 允许回退到 `doc-writing` 的类型：调研、外部代码说明、讨论整理、端到端推演。
