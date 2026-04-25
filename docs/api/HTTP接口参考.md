# HTTP 接口参考


默认地址：`http://127.0.0.1:5000`

当前原生 GUI 版本只保留 Bridge HTTP 接口和健康检查。房间、玩家、日志、状态和设置管理入口在原生 GUI 中，不再通过旧版 `/api/rooms`、`/api/settings` 或 `/ws` 暴露。

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
