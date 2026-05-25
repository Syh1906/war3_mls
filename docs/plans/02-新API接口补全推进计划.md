# 新API接口补全推进计划

## 元信息

| 项目 | 值 |
| --- | --- |
| 计划编号 | 02 |
| 文档类型 | 项目计划 |
| 当前状态 | 进行中 |
| 当前阶段 | 阶段 3：文档与用户验收 |
| 首次建立 | 2026-05-25 |
| 最后更新 | 2026-05-25 |
| 计划范围 | 补齐 `api.lua` 中当前模拟器缺失的 38 个 Lua API，并为模糊平台数据提供默认模拟数据 |
| 明确不含 | 对接真实平台服务、远程数据同步、线上鉴权、生产排行榜、真实社区/成就/任务系统 |
| 关联目录 | `api.lua`、`mls-sim-rs/src/player.rs`、`mls-sim-rs/src/config.rs`、`mls-sim-rs/src/room/`、`mls-sim-rs/config.example.json`、`docs/api/`、`docs/user-guide/` |

## 概述

当前模拟器已经注入日志、定时器、事件、玩家基础查询、房间基础查询、道具、存档、控制和 JSON API。`api.lua` 中还包含一批平台信息、环境信息、玩家行为、成就、任务、签到、社区、公会、排行榜和工具库 API。本文档把补齐范围收敛为本地模拟器能力：新增 API 通过配置和默认模拟数据返回稳定结果，不连接真实平台。

## 最终目标

模拟器补齐 `api.lua` 中当前缺失的 38 个 API。脚本在未显式配置平台数据时也能拿到非空、可预测的模拟数据；开发者可以通过 `config.json` 覆盖玩家、房间、排行榜和环境数据，并通过文档知道每个字段对应哪个 Lua API。

## 范围边界

### 本期范围

- 把 Lua API 注入逻辑从 `mls-sim-rs/src/room/mod.rs` 拆到 `mls-sim-rs/src/room/lua_api.rs` 或同职责模块，避免继续扩大超长文件。
- 扩展玩家、房间、环境和排行榜模拟数据结构。
- 在 `PlayerConfig`、`AutoRoomConfig` 和 `config.example.json` 中提供可覆盖字段。
- 为缺省配置提供非空默认模拟数据，避免平台类 API 默认全空。
- 实现缺失的存档、平台信息、环境信息、行为统计、成就、任务、签到、社区、公会、排行榜和 `md5` API。
- 更新 Lua API 文档、用户配置说明、架构差异说明和 LLM 维护约束。
- 增加 Rust 测试覆盖 API 注入、默认模拟数据和配置覆盖行为。

### 明确不在本期

- 不连接真实 MLS 平台、社区、排行榜或成就服务。
- 不新增生产级数据权限、远程认证、审计日志或遥测。
- 不改变已有 API 的参数名、返回类型和错误码语义。
- 不把默认模拟数据写入真实用户存档。
- 不在未确认的情况下加入额外兜底逻辑；默认模拟数据是本计划明确要求的本地模拟能力。

## 实施前代码基线

| 领域 | 当前事实 | 证据 |
| --- | --- | --- |
| Lua 注入入口 | Lua 全局 API 主要在房间线程初始化阶段直接注入。 | `mls-sim-rs/src/room/mod.rs` 中 `Install Log API`、`Install Player API`、`Install Room API`、`Install Archive API` 等代码块。 |
| 文件规模 | `mls-sim-rs/src/room/mod.rs` 已达到 1595 行，继续堆叠 API 会加重维护成本。 | 2026-05-25 执行 `(Get-Content -LiteralPath 'mls-sim-rs/src/room/mod.rs' | Measure-Object).Count` 得到 `1595`。 |
| 玩家数据 | `Player` 目前只有基础信息、道具和三类存档。 | `mls-sim-rs/src/player.rs` 中 `Player` 字段。 |
| 配置入口 | `PlayerConfig` 目前只覆盖基础玩家字段、道具和存档。 | `mls-sim-rs/src/config.rs` 中 `PlayerConfig` 与 `build_players_from_config`。 |
| 房间数据 | `RoomSharedState` 当前保存 `mode_id`、时间戳、玩家集合和事件/日志缓存。 | `mls-sim-rs/src/room/mod.rs` 中 `RoomSharedState`。 |
| 已实现 API | 当前已实现 31 个 `api.lua` 声明中的 API。 | 2026-05-25 对 `api.lua` 函数声明与 Rust 注入代码比对。 |
| 缺失 API | 当前缺失 38 个 API，集中在平台信息、行为统计、排行榜和 `md5`。 | 2026-05-25 差集结果：`MsSetCommonArchive`、`MsGetPlayerGuid`、`MsGetMapVersion`、`MsGetPlayerRanking`、`md5` 等。 |
| 依赖 | `Cargo.toml` 没有直接声明 `md5` crate。 | `mls-sim-rs/Cargo.toml` 当前依赖列表。 |

## 实施前缺口

1. `MsSetCommonArchive` 在 `api.lua` 中存在声明，但当前运行时只实现了普通存档读取，没有普通存档写入。
2. 平台信息 API 缺少数据字段来源，包括 GUID、平台等级、VIP、作者、收藏和回流标记。
3. 地图与环境 API 缺少房间级模拟字段，包括地图版本、环境类型和预约人数。
4. 玩家行为、成就、任务、签到、社区和公会 API 缺少玩家侧模拟数据结构。
5. 排行榜 API 缺少房间级排行榜数据结构和按排名/玩家查询的统一实现。
6. `md5` API 缺少依赖和 Lua 全局函数注入。
7. 默认配置数据不足，未配置时平台类 API 容易返回空值，影响本地脚本调试。
8. 文档尚未说明这些新增配置字段、默认模拟数据和本地环境与线上环境的差异。

## 当前执行结果

| 领域 | 当前结果 | 证据 |
| --- | --- | --- |
| API 注入结构 | 已新建 `mls-sim-rs/src/room/lua_api.rs` 承载 Lua API 安装逻辑，房间事件循环仍留在 `room/mod.rs`。 | 2026-05-25 代码实现。 |
| 文件规模 | `mls-sim-rs/src/room/mod.rs` 已降到 987 行。 | 2026-05-25 执行 `(Get-Content -LiteralPath 'mls-sim-rs/src/room/mod.rs' | Measure-Object).Count` 得到 `987`。 |
| 缺失 API | 38 个缺失 API 已补齐，并通过 Lua 脚本级测试覆盖默认值和配置覆盖。 | `mls-sim-rs/tests/debug_api.rs` 中新增 `lua_new_api_*` 测试。 |
| 默认模拟数据 | 平台、行为、成就、任务、签到、社区、公会、房间环境和排行榜默认数据已集中在数据结构中维护。 | `mls-sim-rs/src/player.rs`、`mls-sim-rs/src/simulation.rs`。 |
| 错误码 | 错误码常量编号已对齐 `api.lua` 的 `MsErrorCode` 注解；道具“缺物品”返回 `10133`，数量不足返回 `1259`。 | `mls-sim-rs/src/room/mod.rs`、`mls-sim-rs/src/room/lua_api.rs`。 |
| 文档 | API 参考、用户指南、架构差异、LLM 维护约束和目录 README 已同步。 | `docs/api/`、`docs/user-guide/`、`docs/architecture/`、`docs/llm/`。 |

## 全局冻结契约

> 以下条款一经确认即为冻结项，后续阶段不得偏离，除非走变更记录流程。

1. 新增 API 必须以 `api.lua` 的函数名、参数和返回类型为准。
2. 缺省配置必须提供非空、可预测的默认模拟数据；不能让平台类、行为类、社区类和排行榜类 API 默认全部为空。
3. 默认模拟数据只用于本地模拟，不能覆盖用户显式配置，也不能写入真实用户存档。
4. 查询类 API 遇到不存在的玩家时返回该 API 的自然空值，不引入新的异常中断行为。
5. 可配置字段必须向后兼容；旧 `config.json` 能继续加载。
6. `mls-sim-rs/src/room/mod.rs` 不再继续承载大段新增 API 注入代码；先抽出注入模块再补新增 API。
7. 错误码必须对齐 `api.lua` 的 `MsErrorCode` 注解解释，不能新增或改写同编号语义，避免本地模拟器和线上平台分叉。
8. 任一未在本文档冻结的兜底或回退策略，实施前必须先确认。

## 首轮待冻结决策

- [x] 缺省配置需要填充默认模拟数据，避免默认返回全空。
- [x] 默认模拟数据采用确定性非空默认值，并允许用户配置覆盖。
- [x] 排行榜增加 `rankings_loaded` 配置项；默认 `true`，可配置为 `false` 模拟未加载状态。
- [x] `md5` 使用现成 crate 实现，不手写 MD5 算法；具体版本在实现时由 Cargo 解析确认。

### 已冻结默认模拟数据

| 范围 | 默认值 |
| --- | --- |
| 地图环境 | `map_version = "local-dev"`、`env_type = -1`、`prebook_count = 128` |
| 玩家身份 | `guid = 1000000000000 + playerIndex`、`plat_level = 30` |
| VIP | `vip_level = 1`、`map_vip_level = 1`、`vip_types[6]` 对 `playerIndex = 0` 返回 `1`，其他类型默认 `0` |
| 玩家标记 | `is_author` 仅 `playerIndex = 0` 为 `1`、`is_collected = 1`、`is_backflow = 0` |
| 行为数据 | `day_rounds = 3`、`since_last_game = 3600`、未配置宝箱的抽取次数默认 `1` |
| 成就 | `achieve_point = 120`、未配置成就完成状态默认 `0` |
| 任务 | 未配置任务默认 `total = 10`、`current = 3`、`done = 0` |
| 签到 | `sign_in_total = 7`、`sign_in_cont_max = 5`、`sign_in_cont_cur = 2` |
| 社区 | `has_topic = 1`、`is_manager = 0`、`topic_count = 2`、`comment_count = 5`、`happy_count = 20`、`best_count = 1`、`appraise_count = 10`、`is_pinned = 1`、`pet_adv_time = -1` |
| 公会 | `guild_level = 1` |

### 已冻结排行榜规则

- `rankings_loaded` 默认 `true`。
- `rankings_loaded = false` 时，`MsGetPlayerRanking` 和 `MsGetPlayerRankValue` 返回 `-1`。
- 未配置榜单但 `rankings_loaded = true` 时，按空榜处理：玩家排名返回 `0`，榜单名次玩家名返回空串，榜单名次数值返回 `0`。
- 默认榜单从当前房间玩家生成，至少提供 `rankingNum = -1` 和 `rankingNum = 0` 两组样例榜单，并按分值降序排序。

## 里程碑总览

| 里程碑 | 目标 | 截止日期 | 负责人 | 状态 |
| --- | --- | --- | --- | --- |
| M1 | 拆分 Lua API 注入模块并建立模拟数据结构 | 待排期 | 待定 | 未开始 |
| M2 | 实现 38 个缺失 API 与默认模拟数据 | 待排期 | 待定 | 未开始 |
| M3 | 补齐配置示例、文档和测试验收 | 待排期 | 待定 | 未开始 |

## 阶段详述

### 阶段 1：数据模型与注入结构设计

**目标**：拆出 API 注入模块，定义玩家、房间、排行榜和默认模拟数据结构。

**阶段冻结契约**：
> 1. 注入模块只负责 Lua API 安装和小型访问 helper，不接管房间事件循环。
> 2. 默认模拟数据必须通过普通数据结构生成，不能散落在每个 API 闭包里。

**待办清单**：
- [x] 新建 `mls-sim-rs/src/room/lua_api.rs`，迁移现有 API 注入代码。
- [x] 为 `Player` 增加平台、行为、成就、任务、签到、社区、公会字段。
- [x] 为房间增加地图版本、环境类型、预约人数和排行榜字段。
- [x] 定义默认模拟数据生成函数，并让配置覆盖默认值。
- [x] 更新 `config.example.json` 展示新增字段。

**验收规格**：
- [x] `cargo check` 通过。
- [x] 旧配置文件仍可加载并自动获得默认模拟数据。
- [x] `mls-sim-rs/src/room/mod.rs` 行数降到 1500 行以下。
- [x] 默认模拟数据集中维护，不在 38 个 API 闭包中重复硬编码。

**阶段出口条件**：阶段 1 待办和验收全部完成，且默认模拟数据数值表已冻结。

**依赖**：默认模拟数据数值表已冻结；实现前不再等待额外确认。

### 阶段 2：缺失 API 实现

**目标**：按 `api.lua` 补齐当前缺失的 38 个 API。

**阶段冻结契约**：
> 1. API 签名和返回类型对齐 `api.lua`。
> 2. 现有 API 行为保持不变。
> 3. API 错误码对齐 `api.lua` 的 `MsErrorCode` 注解解释。

**待办清单**：
- [x] 实现 `MsSetCommonArchive`，更新普通存档并持久化。
- [x] 实现玩家平台信息 API。
- [x] 实现地图和环境信息 API。
- [x] 实现玩家行为、成就、任务、签到、社区和公会 API。
- [x] 实现排行榜查询 API。
- [x] 增加 `md5` 依赖并注入全局 `md5(str)`。

**验收规格**：
- [x] 每个缺失 API 都有至少一个运行时测试或脚本级验证。
- [x] 未显式配置时，新增 API 返回默认模拟数据。
- [x] 显式配置时，新增 API 返回配置值。
- [x] 不存在玩家时，查询类 API 返回自然空值。
- [x] 所有返回 `MsErrorCode` 的 API 使用与 `api.lua` 注解一致的错误码编号和含义。
- [x] `cargo check` 通过。

**阶段出口条件**：38 个 API 全部实现并通过自动验收。

**依赖**：阶段 1 的数据结构和默认模拟数据已落地。

### 阶段 3：文档与用户验收

**目标**：让使用者知道如何配置新增模拟数据，并明确本地模拟与线上真实数据的差异。

**阶段冻结契约**：
> 1. 文档必须说明默认模拟数据不是线上真实数据。
> 2. API 文档必须覆盖新增 API 的本地返回规则。

**待办清单**：
- [x] 更新 `docs/api/Lua接口参考.md`。
- [x] 更新 `docs/user-guide/程序使用说明.md` 或快速开始中的配置说明。
- [x] 更新 `docs/architecture/与线上环境的差异.md`。
- [x] 更新 `docs/llm/项目维护约束.md`，记录新增 API 维护口径。
- [x] 同步相关目录 `README.md`。

**验收规格**：
- [x] 文档列明新增配置字段与对应 Lua API。
- [x] 文档列明默认模拟数据的用途和覆盖方式。
- [x] 文档列明不对接真实平台服务。
- [x] docs impact review 完成。

**阶段出口条件**：长期事实已回填正式文档，计划可进入归档准备。

**依赖**：阶段 2 实现细节稳定。

## 风险

| 风险 | 可能性 | 影响 | 缓解措施 |
| --- | --- | --- | --- |
| 默认模拟数据被误认为线上真实数据 | 中 | 中 | 文档和配置示例明确标注本地模拟；字段命名避免写成真实来源。 |
| 新字段过多导致配置难读 | 中 | 中 | 使用分组结构，基础示例只展示常用字段，完整示例展示全部字段。 |
| API 注入迁移引入回归 | 中 | 高 | 先迁移现有注入并跑测试，再实现新增 API。 |
| 排行榜语义与线上不一致 | 中 | 中 | 冻结本地模拟规则，文档标注差异。 |
| `md5` 新依赖带来构建变化 | 低 | 低 | 选择小型稳定 crate，并让 `cargo check` 和测试覆盖。 |

## 相关文件

| 文件 | 角色 |
| --- | --- |
| `api.lua` | 新 API 签名和 LuaLS 注解来源 |
| `mls-sim-rs/src/room/mod.rs` | 当前房间生命周期和 API 注入入口 |
| `mls-sim-rs/src/room/lua_api.rs` | 计划新增的 Lua API 注入模块 |
| `mls-sim-rs/src/player.rs` | 玩家模拟数据结构 |
| `mls-sim-rs/src/config.rs` | 配置结构和默认数据构造 |
| `mls-sim-rs/config.example.json` | 用户可复制的配置示例 |
| `docs/api/Lua接口参考.md` | Lua API 长期参考 |
| `docs/architecture/与线上环境的差异.md` | 本地模拟差异说明 |
| `docs/user-guide/程序使用说明.md` | 用户配置说明 |
| `docs/llm/项目维护约束.md` | Agent 维护约束 |

## 文件变更清单

### 新建

| 文件 | 用途 |
| --- | --- |
| `docs/plans/02-新API接口补全推进计划.md` | 记录新 API 补全范围、阶段、默认模拟数据要求和验收口径 |
| `mls-sim-rs/src/room/lua_api.rs` | 后续实现阶段承载 Lua API 注入逻辑 |

### 修改

| 文件 | 变更 |
| --- | --- |
| `docs/plans/README.md` | 增加当前计划导航 |
| `mls-sim-rs/src/room/mod.rs` | 后续实现阶段迁出 API 注入代码 |
| `mls-sim-rs/src/player.rs` | 后续实现阶段扩展玩家模拟字段 |
| `mls-sim-rs/src/config.rs` | 后续实现阶段扩展配置和默认模拟数据 |
| `mls-sim-rs/config.example.json` | 后续实现阶段展示新增配置字段 |
| `mls-sim-rs/Cargo.toml` | 后续实现阶段增加 `md5` 依赖 |
| `docs/api/Lua接口参考.md` | 后续实现阶段补齐新增 API 文档 |
| `docs/user-guide/程序使用说明.md` | 后续实现阶段补齐配置说明 |
| `docs/architecture/与线上环境的差异.md` | 后续实现阶段补齐本地模拟差异 |
| `docs/llm/项目维护约束.md` | 后续实现阶段补齐维护约束 |

### 删除

- 无。

## 变更记录

| 日期 | 变更内容 | 影响范围 | 原因 | 审批人 |
| --- | --- | --- | --- | --- |
| 2026-05-25 | 创建计划 | `docs/plans/` | 建立新 API 补全推进入口，并冻结默认模拟数据要求 | 用户 |
| 2026-05-25 | 冻结首轮决策 | 默认模拟数据、排行榜加载状态、`md5` 选型、错误码对齐 | 用户确认推荐方案，并要求错误码对齐 `api.lua` 注解解释 | 用户 |
| 2026-05-25 | 推进实现 | 运行时 API、配置模型、示例配置、测试和长期文档 | 按冻结方案补齐缺失 API，并保持错误码解释对齐 `api.lua` | 用户 |
