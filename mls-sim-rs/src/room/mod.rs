pub mod json_lua;
pub mod lua_api;
mod profiler;

use crate::player::Player;
use crate::simulation::RoomSimulationData;
use mlua::prelude::*;
use serde::Serialize;
use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::{broadcast, mpsc};

const MAX_EVENT_NAME_LEN: usize = 32;
const MAX_EVENT_DATA_LEN: usize = 900;
const MAX_SCRIPT_ARCHIVE_LEN: usize = 1024 * 1024;
const MAX_LOG_LEN: usize = 2000;
const MAX_BUFFER: usize = 500;

pub const ERR_OK: i32 = 0;
pub const ERR_UNKNOWN: i32 = 1;
pub const ERR_ROOM_NOT_EXIST: i32 = 2;
pub const ERR_PLAYER_NOT_EXIST: i32 = 3;
pub const ERR_EVENT_KEY_LEN: i32 = 4;
pub const ERR_EVENT_KEY_INVALID: i32 = 5;
pub const ERR_EVENT_VALUE_LEN: i32 = 6;
pub const ERR_EVENT_VALUE_INVALID: i32 = 7;
pub const ERR_ARCHIVE_KEY_LEN: i32 = 8;
pub const ERR_ARCHIVE_VALUE_LEN: i32 = 9;
pub const ERR_TEXT_TOO_LONG: i32 = 10;
pub const ERR_SCRIPT_ARCHIVE_TOO_LONG: i32 = 11;
pub const ERR_ITEM_NOT_ENOUGH: i32 = 1259;
pub const ERR_ITEM_NOT_FOUND: i32 = 10133;

fn now_ts() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64()
}

fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum RoomStatus {
    Created,
    Running,
    Stopped,
    Error,
}

impl std::fmt::Display for RoomStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RoomStatus::Created => write!(f, "created"),
            RoomStatus::Running => write!(f, "running"),
            RoomStatus::Stopped => write!(f, "stopped"),
            RoomStatus::Error => write!(f, "error"),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct LogEntry {
    pub timestamp: f64,
    pub level: String,
    pub source: String,
    pub message: String,
    pub room_id: String,
    pub player_index: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct OutEvent {
    pub timestamp: f64,
    pub player_index: i32,
    pub ename: String,
    pub evalue: String,
    pub room_id: String,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct ProfileNode {
    pub id: String,
    pub name: String,
    pub count: u64,
    pub self_count: u64,
    pub children: Vec<ProfileNode>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct ProfileData {
    pub root: ProfileNode,
    pub total_samples: u64,
    pub window: u64,
    pub bucket_count: u64,
    pub running: bool,
}

pub enum RoomCommand {
    DispatchEvent {
        ename: String,
        evalue: String,
        player_index: i32,
    },
    TimerCallback {
        func_key: mlua::RegistryKey,
    },
    TickerFire {
        ticker_id: i32,
    },
    PlayerJoin {
        player_index: i32,
        name: String,
        reason: String,
    },
    PlayerLeave {
        player_index: i32,
        reason: String,
    },
    PlayerExit {
        player_index: i32,
        reason: String,
    },
    Stop {
        reason: String,
    },
    Destroy,
    ProfilerStart {
        hook_count: i32,
        window_seconds: i32,
    },
    ProfilerStop,
    ProfilerReset,
}

pub fn validate_user_event(ename: &str, evalue: &str) -> i32 {
    if ename.is_empty() || ename.as_bytes().len() > MAX_EVENT_NAME_LEN {
        return ERR_EVENT_KEY_LEN;
    }
    if evalue.as_bytes().len() > MAX_EVENT_DATA_LEN {
        return ERR_EVENT_VALUE_LEN;
    }
    if ename.starts_with('_')
        || !ename
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b':' || b == b'_' || !b.is_ascii())
    {
        return ERR_EVENT_KEY_INVALID;
    }
    ERR_OK
}

pub struct RoomSharedState {
    pub id: String,
    pub status: RoomStatus,
    pub error_message: String,
    pub script_dir: PathBuf,
    pub mode_id: i32,
    pub start_ts: i64,
    pub loaded_ts: i64,
    pub simulation: RoomSimulationData,
    pub players: HashMap<i32, Player>,
    pub out_queues: HashMap<i32, VecDeque<OutEvent>>,
    pub log_buffer: VecDeque<LogEntry>,
    pub event_buffer: VecDeque<OutEvent>,
    pub profile_data: Option<ProfileData>,
    pub profiler_available: bool,
    pub profiler_running: bool,
}

impl RoomSharedState {
    pub fn to_json(&self) -> serde_json::Value {
        let game_time = if self.loaded_ts > 0 {
            now_secs() - self.loaded_ts
        } else {
            0
        };
        let players: serde_json::Map<String, serde_json::Value> = self
            .players
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_json()))
            .collect();
        serde_json::json!({
            "id": self.id,
            "script_dir": self.script_dir.to_string_lossy(),
            "mode_id": self.mode_id,
            "status": self.status.to_string(),
            "error_message": self.error_message,
            "start_ts": self.start_ts,
            "loaded_ts": self.loaded_ts,
            "game_time": game_time,
            "simulation": self.simulation,
            "player_count": self.players.values().filter(|p| p.is_connected).count(),
            "players": players,
        })
    }
}

pub struct Room {
    pub shared: Arc<RwLock<RoomSharedState>>,
    pub command_tx: mpsc::UnboundedSender<RoomCommand>,
    pub log_tx: broadcast::Sender<LogEntry>,
    pub out_event_tx: broadcast::Sender<OutEvent>,
    pub join_handle: Option<std::thread::JoinHandle<()>>,
}

impl Room {
    pub fn send_event(&self, ename: String, evalue: String, player_index: i32) -> i32 {
        let err = validate_user_event(&ename, &evalue);
        if err != ERR_OK {
            return err;
        }
        if player_index >= 0
            && !self
                .shared
                .read()
                .unwrap()
                .players
                .contains_key(&player_index)
        {
            return ERR_PLAYER_NOT_EXIST;
        }
        let _ = self.command_tx.send(RoomCommand::DispatchEvent {
            ename,
            evalue,
            player_index,
        });
        ERR_OK
    }

    pub fn stop(&self, reason: String) {
        let _ = self.command_tx.send(RoomCommand::Stop { reason });
    }

    pub fn destroy(&self) {
        let _ = self.command_tx.send(RoomCommand::Destroy);
    }

    pub fn join_player(&self, player_index: i32, name: String, reason: String) -> i32 {
        let _ = self.command_tx.send(RoomCommand::PlayerJoin {
            player_index,
            name,
            reason,
        });
        ERR_OK
    }

    pub fn leave_player(&self, player_index: i32, reason: String) -> i32 {
        if !self
            .shared
            .read()
            .unwrap()
            .players
            .contains_key(&player_index)
        {
            return ERR_PLAYER_NOT_EXIST;
        }
        let _ = self.command_tx.send(RoomCommand::PlayerLeave {
            player_index,
            reason,
        });
        ERR_OK
    }

    pub fn exit_player(&self, player_index: i32, reason: String) -> i32 {
        if !self
            .shared
            .read()
            .unwrap()
            .players
            .contains_key(&player_index)
        {
            return ERR_PLAYER_NOT_EXIST;
        }
        let _ = self.command_tx.send(RoomCommand::PlayerExit {
            player_index,
            reason,
        });
        ERR_OK
    }

    pub fn has_player(&self, player_index: i32) -> bool {
        self.shared
            .read()
            .unwrap()
            .players
            .contains_key(&player_index)
    }

    pub fn profiler_start(&self, hook_count: i32, window_seconds: i32) {
        let _ = self.command_tx.send(RoomCommand::ProfilerStart {
            hook_count,
            window_seconds,
        });
    }

    pub fn profiler_stop(&self) {
        let _ = self.command_tx.send(RoomCommand::ProfilerStop);
    }

    pub fn profiler_reset(&self) {
        let _ = self.command_tx.send(RoomCommand::ProfilerReset);
    }

    pub fn poll_events(&self, player_index: i32) -> Vec<serde_json::Value> {
        let mut shared = self.shared.write().unwrap();
        if let Some(q) = shared.out_queues.get_mut(&player_index) {
            let events: Vec<serde_json::Value> = q
                .drain(..)
                .map(|e| {
                    serde_json::json!({
                        "timestamp": e.timestamp,
                        "player_index": e.player_index,
                        "ename": e.ename,
                        "evalue": e.evalue,
                        "room_id": e.room_id,
                    })
                })
                .collect();
            events
        } else {
            Vec::new()
        }
    }
}

pub struct RoomManager {
    pub rooms: HashMap<String, Room>,
    next_id: u32,
}

impl RoomManager {
    pub fn new() -> Self {
        Self {
            rooms: HashMap::new(),
            next_id: 0,
        }
    }

    pub fn create_room(
        &mut self,
        script_dir: PathBuf,
        mode_id: i32,
        players: HashMap<i32, Player>,
        archive_dir: String,
    ) -> String {
        let simulation = RoomSimulationData::default_for_players(&players);
        self.create_room_with_simulation(script_dir, mode_id, players, simulation, archive_dir)
    }

    pub fn create_room_with_simulation(
        &mut self,
        script_dir: PathBuf,
        mode_id: i32,
        players: HashMap<i32, Player>,
        simulation: RoomSimulationData,
        archive_dir: String,
    ) -> String {
        self.next_id += 1;
        let id = format!("room-{:03}", self.next_id);
        let (command_tx, command_rx) = mpsc::unbounded_channel();
        let (log_tx, _) = broadcast::channel(256);
        let (out_event_tx, _) = broadcast::channel(256);

        let out_queues = players
            .keys()
            .map(|idx| (*idx, VecDeque::with_capacity(MAX_BUFFER)))
            .collect();

        let shared = Arc::new(RwLock::new(RoomSharedState {
            id: id.clone(),
            status: RoomStatus::Created,
            error_message: String::new(),
            script_dir: script_dir.clone(),
            mode_id,
            start_ts: now_secs(),
            loaded_ts: 0,
            simulation,
            players,
            out_queues,
            log_buffer: VecDeque::new(),
            event_buffer: VecDeque::new(),
            profile_data: None,
            profiler_available: false,
            profiler_running: false,
        }));

        let shared_for_room = shared.clone();
        let command_tx_for_room = command_tx.clone();
        let log_tx_for_room = log_tx.clone();
        let out_event_tx_for_room = out_event_tx.clone();

        // Spawn room thread
        let room_id = id.clone();
        let handle = std::thread::Builder::new()
            .name(format!("room-{}", id))
            .spawn(move || {
                room_thread(
                    room_id,
                    script_dir,
                    shared,
                    command_tx,
                    command_rx,
                    log_tx,
                    out_event_tx,
                    archive_dir,
                );
            })
            .expect("failed to spawn room thread");

        let room = Room {
            shared: shared_for_room,
            command_tx: command_tx_for_room,
            log_tx: log_tx_for_room,
            out_event_tx: out_event_tx_for_room,
            join_handle: Some(handle),
        };

        self.rooms.insert(id.clone(), room);
        id
    }

    pub fn get_room(&self, id: &str) -> Option<&Room> {
        self.rooms.get(id)
    }

    pub fn list_rooms(&self) -> Vec<serde_json::Value> {
        self.rooms
            .values()
            .map(|r| r.shared.read().unwrap().to_json())
            .collect()
    }

    pub fn destroy_room(&mut self, id: &str) -> bool {
        if let Some(room) = self.rooms.remove(id) {
            room.destroy();
            true
        } else {
            false
        }
    }

    pub fn restart_room(
        &mut self,
        id: &str,
        archive_dir: String,
        reason: String,
    ) -> Option<String> {
        let room = self.rooms.remove(id)?;
        let (script_dir, mode_id, players, simulation) = {
            let shared = room.shared.read().unwrap();
            (
                shared.script_dir.clone(),
                shared.mode_id,
                shared.players.clone(),
                shared.simulation.clone(),
            )
        };
        room.stop(reason);
        Some(self.create_room_with_simulation(
            script_dir,
            mode_id,
            players,
            simulation,
            archive_dir,
        ))
    }

    pub fn shutdown_all(&mut self) {
        for room in self.rooms.values() {
            let _ = room.command_tx.send(RoomCommand::Stop {
                reason: "Shutdown".into(),
            });
        }
        for room in self.rooms.values_mut() {
            if let Some(handle) = room.join_handle.take() {
                let start = std::time::Instant::now();
                while !handle.is_finished() {
                    if start.elapsed() > std::time::Duration::from_secs(3) {
                        break;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(50));
                }
                if handle.is_finished() {
                    let _ = handle.join();
                }
            }
        }
    }
}

impl Drop for RoomManager {
    fn drop(&mut self) {
        self.shutdown_all();
    }
}

struct EventHandler {
    id: i32,
    func_key: mlua::RegistryKey,
}

fn room_thread(
    room_id: String,
    script_dir: PathBuf,
    shared: Arc<RwLock<RoomSharedState>>,
    command_tx: mpsc::UnboundedSender<RoomCommand>,
    mut command_rx: mpsc::UnboundedReceiver<RoomCommand>,
    log_tx: broadcast::Sender<LogEntry>,
    out_event_tx: broadcast::Sender<OutEvent>,
    archive_dir: String,
) {
    let running = Arc::new(AtomicBool::new(true));
    let next_event_id = Arc::new(AtomicI32::new(1));
    let trans_id_counter = Arc::new(AtomicI32::new(0));
    let log_count = Arc::new(AtomicI32::new(0));
    let log_window_start = Arc::new(Mutex::new(now_ts()));
    let log_fused = Arc::new(AtomicBool::new(false));

    let event_handlers: Arc<Mutex<HashMap<String, Vec<EventHandler>>>> =
        Arc::new(Mutex::new(HashMap::new()));

    let emit_log = {
        let log_tx = log_tx.clone();
        let shared = shared.clone();
        let room_id = room_id.clone();
        let log_count = log_count.clone();
        let log_window_start = log_window_start.clone();
        let log_fused = log_fused.clone();
        move |level: &str, source: &str, message: String| {
            let now = now_ts();
            {
                let mut ws = log_window_start.lock().unwrap();
                if now - *ws > 100.0 {
                    *ws = now;
                    log_count.store(0, Ordering::Relaxed);
                    log_fused.store(false, Ordering::Relaxed);
                }
            }
            if log_fused.load(Ordering::Relaxed) {
                return;
            }
            let count = log_count.fetch_add(1, Ordering::Relaxed);
            let message = if count >= 1000 {
                log_fused.store(true, Ordering::Relaxed);
                "[LOG FUSE] Log rate exceeded 1000/100s".to_string()
            } else {
                message
            };

            let entry = LogEntry {
                timestamp: now,
                level: level.to_string(),
                source: source.to_string(),
                message: message.clone(),
                room_id: room_id.clone(),
                player_index: -1,
            };
            {
                let mut s = shared.write().unwrap();
                s.log_buffer.push_back(entry.clone());
                if s.log_buffer.len() > MAX_BUFFER {
                    s.log_buffer.pop_front();
                }
            }
            let _ = log_tx.send(entry);
            tracing::info!("[{}] [{}] {} {}", room_id, level, source, message);
        }
    };

    let emit_out_event = {
        let out_event_tx = out_event_tx.clone();
        let shared = shared.clone();
        let room_id = room_id.clone();
        move |player_index: i32, ename: String, evalue: String| {
            let ev = OutEvent {
                timestamp: now_ts(),
                player_index,
                ename,
                evalue,
                room_id: room_id.clone(),
            };
            {
                let mut s = shared.write().unwrap();
                s.event_buffer.push_back(ev.clone());
                if s.event_buffer.len() > MAX_BUFFER {
                    s.event_buffer.pop_front();
                }
                if player_index >= 0 {
                    s.out_queues
                        .entry(player_index)
                        .or_insert_with(|| VecDeque::with_capacity(MAX_BUFFER))
                        .push_back(ev.clone());
                } else {
                    let indices: Vec<i32> = s.players.keys().cloned().collect();
                    for idx in indices {
                        s.out_queues
                            .entry(idx)
                            .or_insert_with(|| VecDeque::with_capacity(MAX_BUFFER))
                            .push_back(ev.clone());
                    }
                }
            }
            let _ = out_event_tx.send(ev);
        }
    };

    // Initialize Lua VM
    let lua = match Lua::new() {
        l => l,
    };
    lua.set_memory_limit(10 * 1024 * 1024).ok();

    // Ticker callbacks stored by ID (since RegistryKey is not Clone)
    let ticker_callbacks: Arc<Mutex<HashMap<i32, mlua::RegistryKey>>> =
        Arc::new(Mutex::new(HashMap::new()));
    let next_ticker_id = Arc::new(AtomicI32::new(1));

    if let Err(e) = json_lua::install_json_lib(&lua).and_then(|_| {
        lua_api::install_lua_apis(
            &lua,
            lua_api::InstallLuaApiContext {
                shared: shared.clone(),
                command_tx: command_tx.clone(),
                running: running.clone(),
                event_handlers: event_handlers.clone(),
                ticker_callbacks: ticker_callbacks.clone(),
                next_event_id: next_event_id.clone(),
                next_ticker_id: next_ticker_id.clone(),
                trans_id_counter: trans_id_counter.clone(),
                emit_log: emit_log.clone(),
                emit_out_event: emit_out_event.clone(),
                archive_dir: archive_dir.clone(),
                script_dir: script_dir.clone(),
            },
        )
    }) {
        emit_log("ERR", "System", format!("Lua API install failed: {}", e));
        let mut s = shared.write().unwrap();
        s.status = RoomStatus::Error;
        s.error_message = e.to_string();
        return;
    }

    // Load and run main.lua
    let main_lua = script_dir.join("main.lua");
    if !main_lua.exists() {
        let msg = format!("main.lua not found in {}", script_dir.display());
        emit_log("ERR", "System", msg.clone());
        let emit_out = emit_out_event.clone();
        emit_out(-1, "_mlroomfail".to_string(), msg.clone());
        let mut s = shared.write().unwrap();
        s.status = RoomStatus::Error;
        s.error_message = msg;
        return;
    }

    let code = match std::fs::read_to_string(&main_lua) {
        Ok(c) => c,
        Err(e) => {
            let msg = format!("Failed to read main.lua: {}", e);
            emit_log("ERR", "System", msg.clone());
            let emit_out = emit_out_event.clone();
            emit_out(-1, "_mlroomfail".to_string(), msg.clone());
            let mut s = shared.write().unwrap();
            s.status = RoomStatus::Error;
            s.error_message = msg;
            return;
        }
    };

    if let Err(e) = lua.load(&code).set_name("@main.lua").exec() {
        let msg = format!("Failed to execute main.lua: {}", e);
        emit_log("ERR", "System", msg.clone());
        let emit_out = emit_out_event.clone();
        emit_out(-1, "_mlroomfail".to_string(), msg.clone());
        let mut s = shared.write().unwrap();
        s.status = RoomStatus::Error;
        s.error_message = msg;
        return;
    }

    // Room loaded
    {
        let mut s = shared.write().unwrap();
        s.status = RoomStatus::Running;
        s.loaded_ts = now_secs();
    }
    emit_log("INF", "System", "Room started successfully".to_string());

    // Profiler (Rust-native)
    let profiler_state = Arc::new(Mutex::new(profiler::ProfilerState::new()));
    {
        shared.write().unwrap().profiler_available = true;
    }
    let mut last_profile_extract = std::time::Instant::now();

    // Fire _roomloaded
    {
        let player_indices: Vec<i32> = shared.read().unwrap().players.keys().cloned().collect();
        let data = serde_json::json!({"players": player_indices}).to_string();
        dispatch_lua_event(&lua, &event_handlers, "_roomloaded", &data, -1, &emit_log);
    }

    // Event loop
    while running.load(Ordering::Relaxed) {
        match command_rx.try_recv() {
            Ok(cmd) => match cmd {
                RoomCommand::DispatchEvent {
                    ename,
                    evalue,
                    player_index,
                } => {
                    dispatch_lua_event(
                        &lua,
                        &event_handlers,
                        &ename,
                        &evalue,
                        player_index,
                        &emit_log,
                    );
                }
                RoomCommand::PlayerJoin {
                    player_index,
                    name,
                    reason,
                } => {
                    let should_dispatch = {
                        let mut s = shared.write().unwrap();
                        let existed = s.players.contains_key(&player_index);
                        let player = s
                            .players
                            .entry(player_index)
                            .or_insert_with(|| Player::new(player_index, name));
                        let was_connected = player.is_connected;
                        player.is_connected = true;
                        s.out_queues
                            .entry(player_index)
                            .or_insert_with(|| VecDeque::with_capacity(MAX_BUFFER));
                        !existed || !was_connected
                    };
                    if should_dispatch {
                        let data = serde_json::json!({"reason": reason}).to_string();
                        dispatch_lua_event(
                            &lua,
                            &event_handlers,
                            "_playerjoin",
                            &data,
                            player_index,
                            &emit_log,
                        );
                    }
                }
                RoomCommand::PlayerLeave {
                    player_index,
                    reason,
                } => {
                    let exists = {
                        let mut s = shared.write().unwrap();
                        if let Some(player) = s.players.get_mut(&player_index) {
                            player.is_connected = false;
                            true
                        } else {
                            false
                        }
                    };
                    if exists {
                        let data = serde_json::json!({"reason": reason}).to_string();
                        dispatch_lua_event(
                            &lua,
                            &event_handlers,
                            "_playerleave",
                            &data,
                            player_index,
                            &emit_log,
                        );
                    }
                }
                RoomCommand::PlayerExit {
                    player_index,
                    reason,
                } => {
                    let exists = shared.read().unwrap().players.contains_key(&player_index);
                    if exists {
                        let data = serde_json::json!({"reason": reason}).to_string();
                        dispatch_lua_event(
                            &lua,
                            &event_handlers,
                            "_playerexit",
                            &data,
                            player_index,
                            &emit_log,
                        );
                        {
                            let s = shared.read().unwrap();
                            let _ = crate::storage::save_room_archives(
                                &archive_dir,
                                &s.script_dir.to_string_lossy(),
                                &s.players,
                            );
                        }
                        let mut s = shared.write().unwrap();
                        s.players.remove(&player_index);
                        s.out_queues.remove(&player_index);
                    }
                }
                RoomCommand::TimerCallback { func_key } => {
                    if let Ok(func) = lua.registry_value::<LuaFunction>(&func_key) {
                        if let Err(e) = func.call::<()>(()) {
                            emit_log("ERR", "Timer", format!("Timer callback error: {}", e));
                        }
                    }
                }
                RoomCommand::TickerFire { ticker_id } => {
                    let func_opt = {
                        let cbs = ticker_callbacks.lock().unwrap();
                        cbs.get(&ticker_id)
                            .and_then(|key| lua.registry_value::<LuaFunction>(key).ok())
                    };
                    if let Some(func) = func_opt {
                        if let Err(e) = func.call::<()>(()) {
                            emit_log("ERR", "Timer", format!("Ticker callback error: {}", e));
                        }
                    }
                }
                RoomCommand::ProfilerStart {
                    hook_count,
                    window_seconds,
                } => {
                    {
                        let mut state = profiler_state.lock().unwrap();
                        state.configure(window_seconds, 32);
                    }
                    let state_for_hook = profiler_state.clone();
                    lua.set_hook(
                        mlua::HookTriggers::new().every_nth_instruction(hook_count as u32),
                        move |lua_ctx, _debug| {
                            state_for_hook.lock().unwrap().record_sample(lua_ctx);
                            Ok(mlua::VmState::Continue)
                        },
                    );
                    shared.write().unwrap().profiler_running = true;
                }
                RoomCommand::ProfilerStop => {
                    lua.remove_hook();
                    shared.write().unwrap().profiler_running = false;
                }
                RoomCommand::ProfilerReset => {
                    lua.remove_hook();
                    profiler_state.lock().unwrap().reset();
                    let mut s = shared.write().unwrap();
                    s.profiler_running = false;
                    s.profile_data = None;
                }
                RoomCommand::Stop { reason } => {
                    let data = serde_json::json!({"reason": reason}).to_string();
                    dispatch_lua_event(&lua, &event_handlers, "_roomover", &data, -1, &emit_log);
                    break;
                }
                RoomCommand::Destroy => {
                    break;
                }
            },
            Err(mpsc::error::TryRecvError::Empty) => {
                if last_profile_extract.elapsed() > std::time::Duration::from_millis(500) {
                    let is_running = shared.read().unwrap().profiler_running;
                    if is_running {
                        let mut state = profiler_state.lock().unwrap();
                        let data = state.to_profile_data(true);
                        shared.write().unwrap().profile_data = Some(data);
                    }
                    last_profile_extract = std::time::Instant::now();
                }
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
            Err(mpsc::error::TryRecvError::Disconnected) => {
                break;
            }
        }
    }

    // Save archives
    {
        let s = shared.read().unwrap();
        if let Err(e) = crate::storage::save_room_archives(
            &archive_dir,
            &s.script_dir.to_string_lossy(),
            &s.players,
        ) {
            emit_log("ERR", "System", format!("Archive save failed: {}", e));
        }
    }

    running.store(false, Ordering::Relaxed);
    {
        let mut s = shared.write().unwrap();
        if s.status == RoomStatus::Running {
            s.status = RoomStatus::Stopped;
        }
    }
    emit_log("INF", "System", "Room stopped".to_string());
}

fn dispatch_lua_event<F>(
    lua: &Lua,
    event_handlers: &Arc<Mutex<HashMap<String, Vec<EventHandler>>>>,
    ename: &str,
    evalue: &str,
    player_index: i32,
    emit_log: &F,
) where
    F: Fn(&str, &str, String),
{
    // Collect registry keys we need to call, then drop the lock
    let keys: Vec<i32> = {
        let handlers = event_handlers.lock().unwrap();
        match handlers.get(ename) {
            Some(handler_list) => handler_list.iter().map(|h| h.id).collect(),
            None => return,
        }
    };

    for id in keys {
        // Re-acquire lock briefly to get the registry key reference
        let func_result = {
            let handlers = event_handlers.lock().unwrap();
            if let Some(handler_list) = handlers.get(ename) {
                handler_list
                    .iter()
                    .find(|h| h.id == id)
                    .and_then(|h| lua.registry_value::<LuaFunction>(&h.func_key).ok())
            } else {
                None
            }
        };

        if let Some(func) = func_result {
            if let Err(e) =
                func.call::<()>((id, ename.to_string(), evalue.to_string(), player_index))
            {
                emit_log(
                    "ERR",
                    "Event",
                    format!("Handler error for '{}': {}", ename, e),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_user_event_accepts_documented_boundaries() {
        assert_eq!(validate_user_event("A123:boss", ""), ERR_OK);
        assert_eq!(validate_user_event("war3_data", ""), ERR_OK);
        assert_eq!(
            validate_user_event(&"a".repeat(32), &"b".repeat(900)),
            ERR_OK
        );
    }

    #[test]
    fn validate_user_event_rejects_invalid_key_and_value() {
        assert_eq!(validate_user_event("", ""), ERR_EVENT_KEY_LEN);
        assert_eq!(validate_user_event(&"a".repeat(33), ""), ERR_EVENT_KEY_LEN);
        assert_eq!(
            validate_user_event("_roomloaded", ""),
            ERR_EVENT_KEY_INVALID
        );
        assert_eq!(validate_user_event("bad-key", ""), ERR_EVENT_KEY_INVALID);
        assert_eq!(
            validate_user_event("ok", &"v".repeat(901)),
            ERR_EVENT_VALUE_LEN
        );
    }
}
