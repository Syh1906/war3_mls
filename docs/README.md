# MLS 云脚本文档中心

本文档中心服务 MLS 云脚本本地模拟器的使用、接口、架构和维护流程。它同时承载项目文档和个人 Fork 的开发治理文档。

## 给 LLM 的快速索引

| 任务 | 入口 |
| --- | --- |
| 理解项目定位 | [项目总览](项目总览.md) |
| 编译、运行、排查问题 | [用户指南](user-guide/README.md) |
| 理解线程、Lua VM、存档和环境差异 | [架构文档](architecture/README.md) |
| 查询 HTTP 与 Lua API | [接口文档](api/README.md) |
| 编写或维护客户端桥接代码 | [客户端文档](client/README.md) |
| 修改 docs 结构、模板或规范 | [治理入口](governance/README.md) |
| 查项目内 docs 写作规则 | [规范入口](standards/README.md) |
| 创建新文档 | [模板入口](templates/README.md) |
| 推进计划和归档 | [计划入口](plans/README.md) |
| 沉淀 LLM 约束 | [LLM 知识入口](llm/README.md) |

## 给人类的导航

| 目录 | 作用 | 主要读者 |
| --- | --- | --- |
| `user-guide/` | 使用手册、快速开始、常见问题 | 使用者、测试者 |
| `architecture/` | 模拟器架构和本地环境边界 | 维护者、贡献者 |
| `api/` | HTTP 接口和 Lua API 参考 | 云脚本作者、客户端作者 |
| `client/` | War3 客户端或测试脚本对接说明 | 客户端开发者 |
| `governance/` | docs 治理画像和维护入口 | 文档维护者、Agent |
| `standards/` | 项目内文档规范 | 文档维护者、Agent |
| `templates/` | 新文档模板 | 文档维护者、Agent |
| `plans/` | 计划、执行记录和归档 | 维护者 |
| `llm/` | LLM 约束和易错点 | Agent |

## 同步要求

修改代码、配置、接口、UI、目录结构或计划状态后，需要做 docs impact review。至少检查当前正文、所在目录 `README.md`、本文件、`docs/standards/`、`docs/templates/`、计划状态和 `docs/llm/` 是否需要同步。
