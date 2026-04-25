# HTTP 接口参考


默认地址：`http://127.0.0.1:5000`

## 健康检查

### `GET /api/health`

返回模拟器状态。

```json
{
  "ok": true,
  "name": "mls-sim",
  "version": "0.3.0",
  "host": "127.0.0.1",
  "room_count": 1,
  "rooms": [...]
}
```

---

## 房间管理

### `POST /api/rooms` — 创建房间

```json
{
  "script_dir": "D:/脚本目录/script",
  "mode_id": 0,
  "players": [
    {"index": 0, "name": "玩家1", "items": {"VIP001": 1}},
    {"index": 1, "name": "玩家2"}
  ],
  "auto_start": true
}
```

玩家配置支持的字段：`index`、`name`、`items`、`map_level`、`map_exp`、`played_count`、`script_archive`、`common_archive`、`read_archive`、`cfg_archive`。

创建时自动从 `archives/` 加载上次保存的存档。

### `GET /api/rooms` — 列出所有房间

### `GET /api/rooms/{room_id}` — 获取房间详情

### `DELETE /api/rooms/{room_id}` — 销毁房间

### `POST /api/rooms/{room_id}/start` — 启动房间

房间创建时默认自动启动。

### `POST /api/rooms/{room_id}/stop` — 停止房间

停止时自动保存存档。可选请求体：

```json
{"reason": "GameEnd"}
```

### `GET /api/rooms/{room_id}/state` — 获取房间完整状态

返回房间信息和所有玩家数据的 JSON。

### `POST /api/rooms/{room_id}/events` — 发送事件给云脚本

```json
{
  "ename": "buy_tower",
  "evalue": "{\"id\":1}",
  "player_index": 0
}
```

---

## 玩家管理

### `POST /api/rooms/{room_id}/players` — 添加玩家

```json
{"index": 2, "name": "玩家3", "items": {"VIP001": 1}}
```

### `PUT /api/rooms/{room_id}/players/{idx}` — 更新玩家属性

支持：`name`、`items`、`map_level`、`map_exp`、`script_archive`、`common_archive`、`read_archive`、`cfg_archive`。

### `DELETE /api/rooms/{room_id}/players/{idx}` — 移除玩家

### `POST /api/rooms/{room_id}/players/{idx}/leave` — 模拟玩家断线

触发 `_playerleave` 事件。

### `POST /api/rooms/{room_id}/players/{idx}/join` — 模拟玩家重连

触发 `_playerjoin` 事件。

### `POST /api/rooms/{room_id}/players/{idx}/exit` — 模拟玩家退出

触发 `_playerexit` 事件。

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
  "events": [
    {"player_index": 0, "ename": "asset_update", "evalue": "1000", ...}
  ]
}
```

### `GET /api/bridge/rooms` — 列出可用房间（简化格式）

### `POST /api/bridge/config` — 生成 Bridge 配置

```json
{"room_id": "room-001", "player_index": 0, "poll_interval": 0.05}
```

---

## 存档

### `GET /api/archives` — 列出所有已保存的存档

### `GET /api/archives/{script_name}` — 获取指定脚本的存档数据

---

## 设置

### `GET /api/settings` — 获取当前配置

### `PUT /api/settings` — 更新配置并保存到文件

---

## WebSocket

### `GET /ws` — WebSocket 连接

详见 [模拟器架构](../architecture/模拟器架构.md) 中的 WebSocket 章节。
