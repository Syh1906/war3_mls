# MLS 云脚本本地模拟器

魔兽争霸 III 云脚本（MLS）本地测试工具，Rust 编写，单文件运行，自带 Web 控制面板。

用于在本地模拟平台云脚本运行环境，支持多客户端连接、事件收发、存档读写，无需连接线上平台。

## 项目结构

```
mls-sim-rs/          Rust 模拟器源码
  src/               核心代码
  web/               Web 前端（编译时嵌入二进制）
  config.example.json 配置示例
docs/                使用文档
参考/mls-master/     平台官方 API 文档和 demo 脚本
```

## 快速开始

### 1. 编译

需要 Rust 工具链（`rustup` + `cargo`）。

```powershell
cd mls-sim-rs
cargo build --release
```

产物：`target/release/mls-sim.exe`，约 5MB，无外部依赖。

### 2. 运行

**方式一：命令行指定脚本目录**

```powershell
mls-sim.exe --script-dir "D:/你的脚本目录/script"
```

启动后自动创建房间、加载 `main.lua`、打开浏览器。

**方式二：使用配置文件**

复制 `config.example.json` 为 `config.json`，修改里面的路径：

```powershell
copy config.example.json config.json
mls-sim.exe
```

**方式三：纯 Web 面板操作**

```powershell
mls-sim.exe
```

打开 `http://127.0.0.1:5000`，点击"+ 新建"手动创建房间。

### 3. 验证

```powershell
curl http://127.0.0.1:5000/api/health
```

### 4. 命令行参数

| 参数 | 默认值 | 说明 |
| --- | --- | --- |
| `--host` | `127.0.0.1` | 监听地址 |
| `--port` / `-p` | `5000` | 监听端口 |
| `--script-dir` / `-s` | 无 | 云脚本目录，启动后自动建房 |
| `--config` | `config.json` | 配置文件路径 |

## 配置文件

```json
{
  "host": "127.0.0.1",
  "port": 5000,
  "auto_open_browser": true,
  "archive_dir": "./archives",
  "auto_room": {
    "script_dir": "D:/你的脚本目录/script",
    "mode_id": 0,
    "players": [
      {"index": 0, "name": "玩家1", "items": {"VIP001": 1}},
      {"index": 1, "name": "玩家2"}
    ]
  }
}
```

| 字段 | 说明 |
| --- | --- |
| `host` | 监听地址 |
| `port` | 监听端口 |
| `auto_open_browser` | 启动后自动打开浏览器 |
| `archive_dir` | 存档保存目录 |
| `auto_room` | 启动后自动创建的房间配置（可不填） |
| `auto_room.script_dir` | 云脚本目录，目录下必须有 `main.lua` |
| `auto_room.mode_id` | 游戏模式 ID |
| `auto_room.players` | 玩家列表，含槽位、昵称、道具、存档等 |

## Web 控制面板

启动后访问 `http://127.0.0.1:5000`，功能包括：

- 创建/停止/销毁房间
- 查看玩家状态（在线/离线、等级、道具）
- 模拟玩家断线、重连、退出
- 向云脚本发送自定义事件
- 实时查看脚本日志（支持级别过滤和搜索）
- 实时查看出站事件（脚本发给客户端的事件）
- 查看房间完整状态 JSON

## 客户端对接

客户端（War3 地图脚本或 Python/其他语言模拟）通过 HTTP 与模拟器通信：

```
发送事件给云脚本:  POST /api/bridge/event
轮询云脚本返回:    GET  /api/bridge/poll/{room_id}/{player_index}
```

Python 示例：

```python
import requests, time

BASE = "http://127.0.0.1:5000"
ROOM = "room-001"

# 发送事件
requests.post(f"{BASE}/api/bridge/event", json={
    "room_id": ROOM, "player_index": 0,
    "ename": "buy_tower", "evalue": '{"id":1}'
})

# 轮询
while True:
    r = requests.get(f"{BASE}/api/bridge/poll/{ROOM}/0")
    for ev in r.json().get("events", []):
        print(f"[{ev['ename']}] {ev['evalue']}")
    time.sleep(0.05)
```

## 存档系统

- 云脚本通过 `MsSaveScriptArchive` 等 API 写入存档
- 房间停止时自动保存到 `archives/<脚本目录名>.json`
- 下次创建同名脚本房间时自动读取上次存档
- 配置文件中手动指定的存档数据优先级高于自动读取

## 支持的 Lua API

完整复刻平台 MLS 运行时 API，详见 [Lua 接口参考](docs/api/Lua接口参考.md)：

- `Log.Debug/Info/Error` — 日志
- `Timer.After/NewTicker` — 定时器
- `RegisterEvent/UnregisterEvent` — 事件注册
- `MsSendMlEvent` — 发送事件给客户端
- `MsGetPlayerName/MapLevel/MapExp/PlayedTime/PlayedCount` — 玩家查询
- `MsGetRoomStartTs/LoadedTs/GameTime/PlayerCount/ModeId` — 房间查询
- `MsGetPlayerItem/MsConsumeItem` — 道具
- `MsGet/SaveScriptArchive` — 脚本存档
- `MsGetCommonArchive/ReadArchive/CfgArchive` — 普通/只读/全局存档
- `MsSetReadArchive` — 设置只读存档（触发 `_rdata`）
- `MsEnd` — 停止脚本
- `json.encode/decode` — JSON
- `require` — 模块加载（支持相对路径和 `.` 分隔）

## 详细文档

见 [docs 文档中心](docs/README.md)：

- [项目总览](docs/项目总览.md)
- [用户指南](docs/user-guide/README.md)
- [快速开始](docs/user-guide/快速开始.md)
- [程序使用说明](docs/user-guide/程序使用说明.md)
- [模拟器架构](docs/architecture/模拟器架构.md)
- [HTTP 接口参考](docs/api/HTTP接口参考.md)
- [Lua 接口参考](docs/api/Lua接口参考.md)
- [客户端对接](docs/client/客户端对接.md)
- [常见问题](docs/user-guide/常见问题.md)
- [与线上环境的差异](docs/architecture/与线上环境的差异.md)
