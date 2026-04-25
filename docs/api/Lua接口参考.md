# Lua API 参考


模拟器注入到每个房间 Lua VM 的全部全局 API，与平台 MLS 运行时保持一致。

官方 API 文档见 `参考/mls-master/API.md`。

---

## 日志

```lua
Log.Debug(fmt, ...)   -- 调试日志，正式环境忽略
Log.Info(fmt, ...)    -- 信息日志
Log.Error(fmt, ...)   -- 错误日志
```

- 支持 `string.format` 风格的格式化：`Log.Info("玩家 %s 等级 %d", name, level)`
- 单条日志最大 2000 字节
- 频率限制：100 秒内最多 1000 条，超限后熔断至下一周期

---

## 定时器

```lua
-- 延后执行（单位：秒）
Timer.After(10, function()
    Log.Info("10 秒后执行")
end)

-- 循环定时器（单位：秒）
local ticker = Timer.NewTicker(1, function()
    Log.Info("每 1 秒执行一次")
end)

-- 取消循环定时器
ticker:Cancel()
```

---

## 事件系统

### 注册事件回调

```lua
local id = RegisterEvent("buy_tower", function(id, ename, evalue, player_index)
    local data = json.decode(evalue)
    -- 处理事件逻辑
end)
```

回调参数：

| 参数 | 类型 | 说明 |
| --- | --- | --- |
| `id` | int | 注册编号 |
| `ename` | string | 事件名称 |
| `evalue` | string | 事件数据 |
| `player_index` | int | 玩家槽位，-1 表示房间事件 |

### 取消注册

```lua
UnregisterEvent(id)
```

### 发送事件给客户端

```lua
local err = MsSendMlEvent(player_index, "asset_update", json.encode({gold = 100}))
-- err = 0 表示成功
```

事件名限制：最大 32 字节，不能以 `_` 开头。

事件数据限制：最大 900 字节。

---

## 内置系统事件

以下事件由模拟器自动触发，需要通过 `RegisterEvent` 注册才能收到：

| 事件名 | 触发时机 | 数据示例 |
| --- | --- | --- |
| `_roomloaded` | 房间加载完成 | `{"players": [0, 1]}` |
| `_roomover` | 房间结束 | `{"reason": "GameEnd"}` |
| `_playerexit` | 玩家退出 | `{"reason": "Logout"}` |
| `_playerleave` | 玩家断线 | `{"reason": "Disconnect"}` |
| `_playerjoin` | 玩家重连 | `{"reason": "Connect"}` |

---

## 玩家查询

```lua
MsGetPlayerName(player_index)       -- 返回昵称 (string)
MsGetPlayerMapLevel(player_index)   -- 返回地图等级 (int)
MsGetPlayerMapExp(player_index)     -- 返回地图经验 (int)
MsGetPlayedTime(player_index)       -- 返回游玩时间/秒 (int)
MsGetTestPlayTime(player_index)     -- 返回测试大厅时间/秒 (int)
MsGetPlayedCount(player_index)      -- 返回游玩次数 (int)
```

---

## 房间查询

```lua
MsGetRoomStartTs()       -- 游戏开始时间戳（秒）
MsGetRoomLoadedTs()      -- 加载完成时间戳（秒）
MsGetRoomGameTime()      -- 已过去的游戏时间（秒，从加载完成开始计时）
MsGetRoomPlayerCount()   -- 当前在线玩家数（不含 NPC 和 AI）
MsGetRoomModeId()        -- 游戏模式 ID
```

---

## 道具

```lua
-- 查询道具数量
local count = MsGetPlayerItem(player_index, "VIP001")

-- 消耗道具（异步操作）
local trans_id = MsConsumeItem(player_index, '{"VIP001": 1, "GOLD": 100}')
```

消耗结果通过 `_citemret` 事件异步回调：

```lua
RegisterEvent("_citemret", function(id, ename, evalue, player_index)
    local result = json.decode(evalue)
    -- result.trans_id  业务 ID，与 MsConsumeItem 返回值对应
    -- result.errnu     错误码，0 = 成功
    -- result.iteminfo  本次消耗的道具信息
end)
```

---

## 存档

### 脚本存档（最大 1MB）

```lua
-- 读取
local data = MsGetScriptArchive(player_index)  -- 返回 string 或 nil

-- 保存
local err = MsSaveScriptArchive(player_index, json.encode(save_data))
```

### 普通存档

```lua
local value = MsGetCommonArchive(player_index, "key")  -- 返回 string 或 nil
```

### 只读存档

```lua
-- 读取
local value = MsGetReadArchive(player_index, "boss_kill")

-- 写入（会触发 _rdata 事件通知客户端）
local err = MsSetReadArchive(player_index, "boss_kill", "5")
```

### 全局只读存档

```lua
local value = MsGetCfgArchive(player_index, "global_config")  -- 返回 string 或 nil
```

### 存档持久化

- 房间停止时自动保存到 `archives/<脚本目录名>.json`
- 下次创建房间时自动加载上次保存的存档
- 脚本中应在 `_playerexit` 事件中调用 `MsSaveScriptArchive` 保存

---

## 控制

```lua
MsEnd(player_index, "reason")  -- 停止脚本运行
```

调用后脚本立即停止执行。

---

## JSON

```lua
local str = json.encode({key = "value", list = {1, 2, 3}})
local tbl = json.decode('{"key":"value"}')
```

Rust 注入实现，自动提供 `json` 全局表，无需 `require`。`json.encode` 对 Lua 字符串按字节输出并只转义 JSON 必需字符，`json.decode` 使用标准 JSON 解析。

---

## 模块加载

```lua
require("event.ms_event_api")   -- 加载 event/ms_event_api.lua
require("../map/xxx")           -- 支持相对路径
require("dao/dao_player")       -- 加载 dao/dao_player.lua
```

搜索顺序：
1. 当前模块所在目录
2. 脚本根目录

支持 `init.lua` 目录模块（如 `require("event")` 会尝试 `event/init.lua`）。

---

## 错误码

| 错误码 | 说明 |
| --- | --- |
| 0 | 成功 |
| 1 | 未知错误 |
| 2 | 房间不存在 |
| 3 | 玩家不存在 |
| 4 | 事件名长度超限 |
| 5 | 事件名内容不合规 |
| 6 | 事件数据长度超限 |
| 7 | 事件数据内容不合规 |
| 8 | 存档 Key 长度超限 |
| 9 | 存档 Value 长度超限 |
| 10 | 文本内容超限 |
| 11 | 脚本存档超过 1MB |
| 1259 | 道具数量不足 |
| 10133 | 包裹内没有指定物品 |
