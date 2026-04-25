# HTTP 接口参考

默认地址：`http://127.0.0.1:5000`

当前原生 GUI 版本保留 Bridge HTTP 接口和健康检查，并新增本地自动化调试接口。GUI 继续负责人工调试；HTTP 调试接口供 LLM、脚本和外部工具读取日志、清空房间侧日志缓存和请求房间级重启。

## 元信息

| 项目 | 值 |
| --- | --- |
| 文档类型 | API 契约说明 |
| 最后更新 | 2026-04-25 |
| 真相源 | `mls-sim-rs/src/bridge.rs`、`mls-sim-rs/src/room/mod.rs` |

## 通用契约

| 命名空间 | 调用方 | 说明 |
| --- | --- | --- |
| `/api/bridge/*` | War3 客户端或测试脚本 | 客户端登录、发送事件和轮询出站事件。 |
| `/api/debug/*` | 本地调试脚本、LLM、外部工具 | 自动化调试控制入口，不改变 Bridge 客户端契约。 |
| `/api/health` | 人工或脚本健康检查 | 返回模拟器进程和房间数量状态。 |

调试接口当前只面向本机开发调试，不包含线上运营鉴权、多用户权限、审计日志或访问令牌轮换。不要把 `/api/debug/*` 暴露到公网。

## 健康检查

### `GET /api/health`

返回模拟器状态。

```json
{
  "ok": true,
  "name": "mls-sim",
  "version": "0.4.3",
  "room_count": 1
}
```

---

## Bridge 接口（客户端通信）

用于 War3 客户端或测试脚本与模拟器通信。

### `POST /api/bridge/login` — 客户端登录

```json
{"room_id": "room-001", "player_index": 0, "name": "玩家名"}
```

### `POST /api/bridge/event` — 客户端发送事件给云脚本

```json
{
  "room_id": "room-001",
  "player_index": 0,
  "ename": "buy_tower",
  "evalue": "{}"
}
```

### `GET /api/bridge/poll/{room_id}/{player_index}` — 客户端轮询事件

返回云脚本发给该玩家的待取事件，取完自动清空队列：

```json
{
  "ok": true,
  "errnu": 0,
  "events": [
    {"player_index": 0, "ename": "asset_update", "evalue": "1000"}
  ]
}
```

### `GET /api/bridge/rooms` — 列出可用房间（简化格式）

### `POST /api/bridge/config` — 生成 Bridge 配置

```json
{"room_id": "room-001", "player_index": 0, "poll_interval": 0.05}
```

---

## 调试接口（本地自动化调试）

用于本地脚本、LLM 或调试工具读取房间侧缓存、清空房间侧日志缓存和重启房间。调试接口不会替代 GUI；GUI 控制台清空的是 GUI 本地显示列表，调试接口清空的是房间侧 `log_buffer`。

### `GET /api/debug/rooms/{room_id}/logs` — 读取房间日志

从指定房间的房间侧日志缓存读取日志。缓存最多保留 500 条，接口默认返回最近 100 条匹配记录。

| 查询参数 | 默认值 | 说明 |
| --- | --- | --- |
| `limit` | `100` | 返回数量，范围为 `1` 到 `500`。 |
| `level` | 无 | 按日志级别过滤，例如 `DBG`、`INF`、`ERR`。 |
| `q` | 无 | 按日志消息或来源做包含匹配。 |
| `since` | 无 | 只返回 `timestamp >= since` 的日志。 |

成功响应：

```json
{
  "ok": true,
  "errnu": 0,
  "room_id": "room-001",
  "limit": 100,
  "count": 1,
  "total": 1,
  "logs": [
    {
      "timestamp": 1777046400.0,
      "level": "INF",
      "source": "System",
      "message": "Room started successfully",
      "room_id": "room-001",
      "player_index": -1
    }
  ]
}
```

房间不存在时返回 `404`，`ok` 为 `false`，`errnu` 为 `2`。

### `POST /api/debug/rooms/{room_id}/logs/clear` — 清空房间侧日志缓存

清空指定房间的 `RoomSharedState.log_buffer`。该操作不清空 GUI 控制台已收集到的本地显示列表，也不清空出站事件缓存。

成功响应：

```json
{
  "ok": true,
  "errnu": 0,
  "room_id": "room-001",
  "cleared": 42
}
```

房间不存在时返回 `404`，`ok` 为 `false`，`errnu` 为 `2`。

### `POST /api/debug/rooms/{room_id}/restart` — 房间级重启

基于旧房间的脚本目录、模式和玩家快照创建新房间，并停止旧房间。新房间会分配新的 `room_id`；调用方后续应改用响应中的新房间 ID。

请求体可为空，也可以传入重启原因：

```json
{"reason": "reload after script change"}
```

成功响应：

```json
{
  "ok": true,
  "errnu": 0,
  "old_room_id": "room-001",
  "room_id": "room-002",
  "status": "restarted"
}
```

房间不存在时返回 `404`，`ok` 为 `false`，`errnu` 为 `2`。

### `POST /api/debug/service/restart` — 服务级重启占位

模拟器进程内不提供服务级重启。跨进程停止和重新拉起由外部进程管理工具负责。

固定响应状态码为 `501`：

```json
{
  "ok": false,
  "errnu": 1,
  "error": "Service restart is not supported by the simulator HTTP API; restart the process with an external tool."
}
```

## 变更记录

### 2026-04-25 — 新增本地自动化调试接口

| 路由 | 方法 | 变更内容 |
| --- | --- | --- |
| `/api/debug/rooms/{room_id}/logs` | `GET` | 新增房间侧日志读取接口。 |
| `/api/debug/rooms/{room_id}/logs/clear` | `POST` | 新增房间侧日志缓存清空接口。 |
| `/api/debug/rooms/{room_id}/restart` | `POST` | 新增房间级重启接口，返回新房间 ID。 |
| `/api/debug/service/restart` | `POST` | 明确服务级重启不由模拟器进程内 HTTP API 承担。 |
