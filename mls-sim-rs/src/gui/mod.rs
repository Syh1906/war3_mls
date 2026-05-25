mod console;
mod events;
mod profiler;
mod rooms;
mod settings;
mod state_view;

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use tokio::sync::broadcast;

use eframe::egui;

use crate::config::AppConfig;
use crate::player::Player;
use crate::room::{LogEntry, OutEvent, RoomManager};
use crate::storage;

#[derive(PartialEq)]
pub enum Tab {
    Rooms,
    Console,
    Events,
    State,
    Profiler,
    Settings,
}

struct CommandHint {
    name: &'static str,
    usage: &'static str,
}

const COMMAND_HINTS: &[CommandHint] = &[
    CommandHint {
        name: "rooms",
        usage: "rooms - 列出所有房间",
    },
    CommandHint {
        name: "clear",
        usage: "clear - 清空控制台日志",
    },
    CommandHint {
        name: "create",
        usage: "create <脚本目录> [模式] - 创建房间",
    },
    CommandHint {
        name: "select",
        usage: "select <房间> - 选择房间",
    },
    CommandHint {
        name: "stop",
        usage: "stop <房间> - 停止房间",
    },
    CommandHint {
        name: "destroy",
        usage: "destroy <房间> - 删除房间",
    },
    CommandHint {
        name: "restart",
        usage: "restart <房间> - 重启房间",
    },
    CommandHint {
        name: "join",
        usage: "join <房间> <槽位> [名称] - 玩家上线",
    },
    CommandHint {
        name: "leave",
        usage: "leave <房间> <槽位> - 玩家离线",
    },
    CommandHint {
        name: "exit",
        usage: "exit <房间> <槽位> - 玩家退出",
    },
    CommandHint {
        name: "event",
        usage: "event <房间> <事件名> [数据] [玩家] - 发送事件",
    },
];

pub struct GuiApp {
    pub manager: Arc<RwLock<RoomManager>>,
    pub config: Arc<RwLock<AppConfig>>,
    pub config_path: String,
    pub active_tab: Tab,

    // Rooms tab
    pub selected_room_id: Option<String>,
    pub show_create_room: bool,
    pub new_room_script_dir: String,
    pub new_room_mode_id: i32,
    pub add_player_index: String,
    pub add_player_name: String,
    pub event_name: String,
    pub event_data: String,
    pub event_player_idx: i32,

    // Console
    pub logs: Vec<LogEntry>,
    pub out_events: Vec<OutEvent>,
    subscribed_rooms: HashSet<String>,
    log_receivers: HashMap<String, broadcast::Receiver<LogEntry>>,
    event_receivers: HashMap<String, broadcast::Receiver<OutEvent>>,
    pub log_level_filter: String,
    pub log_search: String,
    pub log_room_filter: String,
    pub auto_scroll: bool,

    // State viewer
    pub state_room_id: Option<String>,
    pub state_json_text: String,

    // Settings
    pub settings_host: String,
    pub settings_port: String,
    pub settings_archive_dir: String,

    // Profiler
    pub profiler_room_id: Option<String>,
    pub profiler_hook_count: i32,
    pub profiler_window: i32,
    pub profiler_frame_ms: f32,
    pub profiler_hover_info: String,

    // Status
    pub save_msg: Option<(String, bool, f64)>,
    pub command_input: String,
    pub command_msg: Option<(String, bool, f64)>,
}

impl GuiApp {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        manager: Arc<RwLock<RoomManager>>,
        config: Arc<RwLock<AppConfig>>,
        config_path: String,
    ) -> Self {
        setup_fonts(&cc.egui_ctx);
        setup_style(&cc.egui_ctx);

        let (host, port, archive_dir) = {
            let cfg = config.read().unwrap();
            (
                cfg.host.clone(),
                cfg.port.to_string(),
                cfg.archive_dir.clone(),
            )
        };

        Self {
            manager,
            config,
            config_path,
            active_tab: Tab::Rooms,
            selected_room_id: None,
            show_create_room: false,
            new_room_script_dir: String::new(),
            new_room_mode_id: 0,
            add_player_index: "0".into(),
            add_player_name: String::new(),
            event_name: String::new(),
            event_data: String::new(),
            event_player_idx: -1,
            logs: Vec::new(),
            out_events: Vec::new(),
            subscribed_rooms: HashSet::new(),
            log_receivers: HashMap::new(),
            event_receivers: HashMap::new(),
            log_level_filter: String::new(),
            log_search: String::new(),
            log_room_filter: String::new(),
            auto_scroll: true,
            state_room_id: None,
            state_json_text: String::new(),
            profiler_room_id: None,
            profiler_hook_count: 5000,
            profiler_window: 15,
            profiler_frame_ms: 50.0,
            profiler_hover_info: String::new(),
            settings_host: host,
            settings_port: port,
            settings_archive_dir: archive_dir,
            save_msg: None,
            command_input: String::new(),
            command_msg: None,
        }
    }

    fn sync_subscriptions(&mut self) {
        let manager = match self.manager.try_read() {
            Ok(m) => m,
            Err(_) => return,
        };
        let current: HashSet<String> = manager.rooms.keys().cloned().collect();

        for id in &current {
            if !self.subscribed_rooms.contains(id) {
                if let Some(room) = manager.rooms.get(id) {
                    if let Ok(shared) = room.shared.try_read() {
                        self.logs.extend(shared.log_buffer.iter().cloned());
                        self.out_events.extend(shared.event_buffer.iter().cloned());
                    }
                    self.log_receivers
                        .insert(id.clone(), room.log_tx.subscribe());
                    self.event_receivers
                        .insert(id.clone(), room.out_event_tx.subscribe());
                }
            }
        }

        let removed: Vec<String> = self
            .subscribed_rooms
            .difference(&current)
            .cloned()
            .collect();
        for id in &removed {
            self.log_receivers.remove(id);
            self.event_receivers.remove(id);
        }

        self.subscribed_rooms = current;
    }

    fn drain_channels(&mut self) {
        for rx in self.log_receivers.values_mut() {
            loop {
                match rx.try_recv() {
                    Ok(entry) => self.logs.push(entry),
                    Err(broadcast::error::TryRecvError::Lagged(_)) => continue,
                    _ => break,
                }
            }
        }
        for rx in self.event_receivers.values_mut() {
            loop {
                match rx.try_recv() {
                    Ok(event) => self.out_events.push(event),
                    Err(broadcast::error::TryRecvError::Lagged(_)) => continue,
                    _ => break,
                }
            }
        }

        const MAX_LOGS: usize = 10000;
        if self.logs.len() > MAX_LOGS {
            self.logs.drain(0..self.logs.len() - MAX_LOGS);
        }
        if self.out_events.len() > MAX_LOGS {
            self.out_events.drain(0..self.out_events.len() - MAX_LOGS);
        }
    }

    fn command_line(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("command_line")
            .frame(
                egui::Frame::none()
                    .inner_margin(egui::Margin::symmetric(10.0, 6.0))
                    .fill(ctx.style().visuals.window_fill()),
            )
            .show(ctx, |ui: &mut egui::Ui| {
                ui.horizontal(|ui: &mut egui::Ui| {
                    ui.label(egui::RichText::new("命令:").strong());

                    let input_width = (ui.available_width()).max(160.0);
                    let response = ui.add_sized(
                        [input_width, 24.0],
                        egui::TextEdit::singleline(&mut self.command_input)
                            .font(egui::TextStyle::Monospace)
                    );
                    let enter_pressed = ui.input(|i| i.key_pressed(egui::Key::Enter))
                        && (response.has_focus() || response.lost_focus());
                    if enter_pressed {
                        self.execute_command_input();
                    } else {
                        show_command_hint(ui.ctx(), &response, &self.command_input);
                    }
                });

                if let Some((ref msg, is_err, ts)) = self.command_msg {
                    if current_ts() - ts < 6.0 {
                        let color = if is_err {
                            egui::Color32::from_rgb(255, 95, 95)
                        } else {
                            egui::Color32::from_rgb(75, 215, 105)
                        };
                        ui.colored_label(color, msg);
                    }
                }
            });
    }

    fn execute_command_input(&mut self) {
        let command = self.command_input.trim().to_string();
        if command.is_empty() {
            return;
        }

        self.command_input.clear();
        let now = current_ts();
        let (message, is_err) = match self.execute_command(&command) {
            Ok(message) => (message, false),
            Err(message) => (message, true),
        };

        self.logs.push(LogEntry {
            timestamp: now,
            level: if is_err { "ERR" } else { "INF" }.into(),
            source: "GUI".into(),
            message: format!("$ {} -> {}", command, message),
            room_id: "gui".into(),
            player_index: -1,
        });
        self.command_msg = Some((message, is_err, now));
    }

    fn execute_command(&mut self, command: &str) -> Result<String, String> {
        let args = split_command_args(command)?;
        if args.is_empty() {
            return Ok(String::new());
        }

        match args[0].as_str() {
            "rooms" => self.command_list_rooms(),
            "clear" => {
                self.logs.clear();
                Ok("日志已清空".into())
            }
            "create" => self.command_create_room(&args),
            "select" => self.command_select_room(&args),
            "stop" => self.command_stop_room(&args),
            "destroy" => self.command_destroy_room(&args),
            "restart" => self.command_restart_room(&args),
            "join" => self.command_join_player(&args),
            "leave" => self.command_leave_player(&args),
            "exit" => self.command_exit_player(&args),
            "event" => self.command_send_event(&args),
            other => Err(format!("未知命令: {}", other)),
        }
    }

    fn command_list_rooms(&self) -> Result<String, String> {
        let manager = self.manager.read().unwrap();
        if manager.rooms.is_empty() {
            return Ok("暂无房间".into());
        }

        let mut rooms: Vec<String> = manager
            .rooms
            .iter()
            .map(|(id, room)| {
                let shared = room.shared.read().unwrap();
                format!("{}({})", id, shared.status)
            })
            .collect();
        rooms.sort();
        Ok(format!("房间: {}", rooms.join(", ")))
    }

    fn command_create_room(&mut self, args: &[String]) -> Result<String, String> {
        if args.len() < 2 {
            return Err("用法: create <脚本目录> [模式]".into());
        }

        let script_dir = PathBuf::from(&args[1]);
        if !script_dir.is_dir() {
            return Err(format!("脚本目录不存在: {}", args[1]));
        }

        let mode_id = parse_optional_i32(args.get(2), 0, "模式")?;
        let archive_dir = self.config.read().unwrap().archive_dir.clone();
        let mut players = HashMap::new();
        players.insert(0, Player::new(0, "Player_0".into()));
        storage::apply_saved_archives(&archive_dir, &args[1], &mut players);

        let room_id =
            self.manager
                .write()
                .unwrap()
                .create_room(script_dir, mode_id, players, archive_dir);
        self.selected_room_id = Some(room_id.clone());
        self.active_tab = Tab::Rooms;
        Ok(format!("已创建房间 {}", room_id))
    }

    fn command_select_room(&mut self, args: &[String]) -> Result<String, String> {
        let room_id = required_arg(args, 1, "房间")?;
        let exists = self.manager.read().unwrap().rooms.contains_key(room_id);
        if !exists {
            return Err(format!("房间不存在: {}", room_id));
        }

        self.selected_room_id = Some(room_id.to_string());
        self.active_tab = Tab::Rooms;
        Ok(format!("已选择房间 {}", room_id))
    }

    fn command_stop_room(&self, args: &[String]) -> Result<String, String> {
        let room_id = required_arg(args, 1, "房间")?;
        let manager = self.manager.read().unwrap();
        let room = manager
            .rooms
            .get(room_id)
            .ok_or_else(|| format!("房间不存在: {}", room_id))?;
        room.stop("GuiCommand".into());
        Ok(format!("已停止房间 {}", room_id))
    }

    fn command_destroy_room(&mut self, args: &[String]) -> Result<String, String> {
        let room_id = required_arg(args, 1, "房间")?;
        let destroyed = self.manager.write().unwrap().destroy_room(room_id);
        if !destroyed {
            return Err(format!("房间不存在: {}", room_id));
        }
        if self.selected_room_id.as_deref() == Some(room_id) {
            self.selected_room_id = None;
        }
        Ok(format!("已删除房间 {}", room_id))
    }

    fn command_restart_room(&mut self, args: &[String]) -> Result<String, String> {
        let room_id = required_arg(args, 1, "房间")?;
        let archive_dir = self.config.read().unwrap().archive_dir.clone();
        let new_room_id = self
            .manager
            .write()
            .unwrap()
            .restart_room(room_id, archive_dir, "GuiCommand".into())
            .ok_or_else(|| format!("房间不存在: {}", room_id))?;
        self.selected_room_id = Some(new_room_id.clone());
        self.active_tab = Tab::Rooms;
        Ok(format!("已重启房间 {} -> {}", room_id, new_room_id))
    }

    fn command_join_player(&self, args: &[String]) -> Result<String, String> {
        let room_id = required_arg(args, 1, "房间")?;
        let player_index = parse_required_i32(args, 2, "槽位")?;
        let name = if args.len() > 3 {
            args[3..].join(" ")
        } else {
            format!("Player_{}", player_index)
        };

        let manager = self.manager.read().unwrap();
        let room = manager
            .rooms
            .get(room_id)
            .ok_or_else(|| format!("房间不存在: {}", room_id))?;
        room.join_player(player_index, name, "Connect".into());
        Ok(format!("玩家 {} 已加入 {}", player_index, room_id))
    }

    fn command_leave_player(&self, args: &[String]) -> Result<String, String> {
        let room_id = required_arg(args, 1, "房间")?;
        let player_index = parse_required_i32(args, 2, "槽位")?;
        let manager = self.manager.read().unwrap();
        let room = manager
            .rooms
            .get(room_id)
            .ok_or_else(|| format!("房间不存在: {}", room_id))?;
        let errnu = room.leave_player(player_index, "Disconnect".into());
        if errnu != crate::room::ERR_OK {
            return Err(format!("玩家离线失败: errnu={}", errnu));
        }
        Ok(format!("玩家 {} 已离线", player_index))
    }

    fn command_exit_player(&self, args: &[String]) -> Result<String, String> {
        let room_id = required_arg(args, 1, "房间")?;
        let player_index = parse_required_i32(args, 2, "槽位")?;
        let manager = self.manager.read().unwrap();
        let room = manager
            .rooms
            .get(room_id)
            .ok_or_else(|| format!("房间不存在: {}", room_id))?;
        let errnu = room.exit_player(player_index, "Logout".into());
        if errnu != crate::room::ERR_OK {
            return Err(format!("玩家退出失败: errnu={}", errnu));
        }
        Ok(format!("玩家 {} 已退出", player_index))
    }

    fn command_send_event(&self, args: &[String]) -> Result<String, String> {
        let room_id = required_arg(args, 1, "房间")?;
        let event_name = required_arg(args, 2, "事件名")?;
        let mut player_index = -1;
        let data_end = if args.len() > 4 {
            match args.last().unwrap().parse::<i32>() {
                Ok(idx) => {
                    player_index = idx;
                    args.len() - 1
                }
                Err(_) => args.len(),
            }
        } else {
            args.len()
        };
        let event_data = if data_end > 3 {
            args[3..data_end].join(" ")
        } else {
            String::new()
        };

        let manager = self.manager.read().unwrap();
        let room = manager
            .rooms
            .get(room_id)
            .ok_or_else(|| format!("房间不存在: {}", room_id))?;
        let errnu = room.send_event(event_name.to_string(), event_data, player_index);
        if errnu != crate::room::ERR_OK {
            return Err(format!("发送事件失败: errnu={}", errnu));
        }
        Ok(format!("已发送事件 {} 到 {}", event_name, room_id))
    }
}

impl Drop for GuiApp {
    fn drop(&mut self) {
        if let Ok(mut mgr) = self.manager.write() {
            mgr.shutdown_all();
        }
    }
}

impl eframe::App for GuiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // DPI: sync to native pixels_per_point for crisp rendering
        if let Some(native_ppp) = ctx.native_pixels_per_point() {
            if (ctx.pixels_per_point() - native_ppp).abs() > 0.01 {
                ctx.set_pixels_per_point(native_ppp);
            }
        }

        self.sync_subscriptions();
        self.drain_channels();

        let room_count = self.subscribed_rooms.len();

        egui::TopBottomPanel::top("tab_bar")
            .frame(
                egui::Frame::none()
                    .inner_margin(egui::Margin::symmetric(8.0, 6.0))
                    .fill(ctx.style().visuals.window_fill()),
            )
            .show(ctx, |ui: &mut egui::Ui| {
                ui.horizontal(|ui: &mut egui::Ui| {
                    ui.spacing_mut().item_spacing.x = 4.0;
                    for (tab, label) in [
                        (Tab::Rooms, "房间"),
                        (Tab::Console, "控制台"),
                        (Tab::Events, "出站事件"),
                        (Tab::State, "状态"),
                        (Tab::Profiler, "性能分析"),
                        (Tab::Settings, "设置"),
                    ] {
                        let selected = self.active_tab == tab;
                        let text = egui::RichText::new(label).size(15.0);
                        let text = if selected { text.strong() } else { text };
                        if ui.selectable_label(selected, text).clicked() {
                            self.active_tab = tab;
                        }
                    }

                    ui.with_layout(
                        egui::Layout::right_to_left(egui::Align::Center),
                        |ui: &mut egui::Ui| {
                            ui.label(
                                egui::RichText::new(format!(
                                    "房间: {}  |  日志: {}",
                                    room_count,
                                    self.logs.len()
                                ))
                                .small()
                                .color(egui::Color32::from_rgb(140, 148, 165)),
                            );
                        },
                    );
                });
            });

        self.command_line(ctx);

        match self.active_tab {
            Tab::Rooms => self.rooms_tab(ctx),
            Tab::Console => self.console_tab(ctx),
            Tab::Events => self.events_tab(ctx),
            Tab::State => self.state_tab(ctx),
            Tab::Profiler => self.profiler_tab(ctx),
            Tab::Settings => self.settings_tab(ctx),
        }

        ctx.request_repaint_after(std::time::Duration::from_millis(100));
    }
}

fn setup_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    let font_paths = [
        "C:/Windows/Fonts/msyh.ttc",
        "C:/Windows/Fonts/simhei.ttf",
        "C:/Windows/Fonts/simsun.ttc",
    ];

    for path in &font_paths {
        if let Ok(data) = std::fs::read(path) {
            fonts
                .font_data
                .insert("chinese".to_owned(), egui::FontData::from_owned(data));
            if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
                family.insert(0, "chinese".to_owned());
            }
            if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
                family.push("chinese".to_owned());
            }
            break;
        }
    }

    ctx.set_fonts(fonts);
}

fn setup_style(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();

    style.text_styles = [
        (
            egui::TextStyle::Small,
            egui::FontId::new(13.0, egui::FontFamily::Proportional),
        ),
        (
            egui::TextStyle::Body,
            egui::FontId::new(15.0, egui::FontFamily::Proportional),
        ),
        (
            egui::TextStyle::Button,
            egui::FontId::new(15.0, egui::FontFamily::Proportional),
        ),
        (
            egui::TextStyle::Heading,
            egui::FontId::new(20.0, egui::FontFamily::Proportional),
        ),
        (
            egui::TextStyle::Monospace,
            egui::FontId::new(14.0, egui::FontFamily::Monospace),
        ),
    ]
    .into();

    style.spacing.item_spacing = egui::vec2(8.0, 6.0);
    style.spacing.window_margin = egui::Margin::same(10.0);
    style.spacing.button_padding = egui::vec2(10.0, 5.0);

    let v = &mut style.visuals;
    v.window_fill = egui::Color32::from_rgb(22, 22, 28);
    v.panel_fill = egui::Color32::from_rgb(28, 28, 35);
    v.faint_bg_color = egui::Color32::from_rgb(38, 40, 50);
    v.extreme_bg_color = egui::Color32::from_rgb(14, 14, 18);
    v.code_bg_color = egui::Color32::from_rgb(38, 40, 50);

    v.error_fg_color = egui::Color32::from_rgb(255, 100, 100);
    v.warn_fg_color = egui::Color32::from_rgb(255, 200, 80);
    v.hyperlink_color = egui::Color32::from_rgb(100, 170, 255);

    v.selection.bg_fill = egui::Color32::from_rgb(45, 75, 135);
    v.selection.stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(120, 165, 255));

    v.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(35, 36, 44);
    v.widgets.noninteractive.weak_bg_fill = egui::Color32::from_rgb(32, 33, 40);
    v.widgets.noninteractive.bg_stroke =
        egui::Stroke::new(1.0, egui::Color32::from_rgb(62, 64, 80));
    v.widgets.noninteractive.fg_stroke =
        egui::Stroke::new(1.0, egui::Color32::from_rgb(200, 205, 220));
    v.widgets.noninteractive.rounding = egui::Rounding::same(4.0);

    v.widgets.inactive.bg_fill = egui::Color32::from_rgb(45, 46, 58);
    v.widgets.inactive.weak_bg_fill = egui::Color32::from_rgb(40, 41, 52);
    v.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(70, 72, 90));
    v.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(215, 218, 228));
    v.widgets.inactive.rounding = egui::Rounding::same(4.0);

    v.widgets.hovered.bg_fill = egui::Color32::from_rgb(58, 60, 76);
    v.widgets.hovered.weak_bg_fill = egui::Color32::from_rgb(52, 54, 68);
    v.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(100, 120, 190));
    v.widgets.hovered.fg_stroke = egui::Stroke::new(1.5, egui::Color32::from_rgb(240, 242, 250));
    v.widgets.hovered.rounding = egui::Rounding::same(4.0);

    v.widgets.active.bg_fill = egui::Color32::from_rgb(65, 75, 110);
    v.widgets.active.weak_bg_fill = egui::Color32::from_rgb(58, 66, 96);
    v.widgets.active.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(120, 145, 220));
    v.widgets.active.fg_stroke = egui::Stroke::new(2.0, egui::Color32::WHITE);
    v.widgets.active.rounding = egui::Rounding::same(4.0);

    v.widgets.open.bg_fill = egui::Color32::from_rgb(48, 50, 64);
    v.widgets.open.weak_bg_fill = egui::Color32::from_rgb(44, 46, 58);
    v.widgets.open.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(82, 85, 105));
    v.widgets.open.fg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(222, 225, 240));
    v.widgets.open.rounding = egui::Rounding::same(4.0);

    v.window_rounding = egui::Rounding::same(6.0);
    v.window_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(55, 58, 72));
    v.striped = true;

    ctx.set_style(style);
}

pub fn section_heading(ui: &mut egui::Ui, text: &str) {
    ui.add_space(2.0);
    ui.label(egui::RichText::new(text).strong().size(16.0));
    ui.add_space(2.0);
}

pub fn format_time(ts: f64) -> String {
    let secs = ts as i64;
    let nanos = ((ts - secs as f64) * 1e9) as u32;
    chrono::DateTime::from_timestamp(secs, nanos)
        .map(|dt| {
            dt.with_timezone(&chrono::Local)
                .format("%H:%M:%S")
                .to_string()
        })
        .unwrap_or_else(|| format!("{:.1}", ts))
}

fn current_ts() -> f64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64()
}

fn show_command_hint(ctx: &egui::Context, response: &egui::Response, input: &str) {
    if !response.has_focus() {
        return;
    }

    let hints = matching_command_hints(input);
    if hints.is_empty() {
        return;
    }

    let row_height = 20.0;
    let popup_height = 10.0 + row_height * hints.len() as f32;
    let pos = egui::pos2(
        response.rect.left(),
        (response.rect.top() - popup_height - 6.0).max(0.0),
    );

    egui::Area::new(egui::Id::new("command_hint_popup"))
        .order(egui::Order::Foreground)
        .fixed_pos(pos)
        .show(ctx, |ui| {
            egui::Frame::popup(ui.style()).show(ui, |ui| {
                ui.set_min_width(response.rect.width().min(640.0));
                for hint in hints {
                    ui.label(
                        egui::RichText::new(hint.usage)
                            .monospace()
                            .size(13.0),
                    );
                }
            });
        });
}

fn matching_command_hints(input: &str) -> Vec<&'static CommandHint> {
    let trimmed = input.trim_start();
    if trimmed.is_empty() {
        return COMMAND_HINTS.iter().collect();
    }

    let command = trimmed
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase();
    if command.is_empty() {
        return Vec::new();
    }

    if let Some(hint) = COMMAND_HINTS.iter().find(|hint| hint.name == command) {
        return vec![hint];
    }

    COMMAND_HINTS
        .iter()
        .filter(|hint| hint.name.starts_with(&command))
        .take(5)
        .collect()
}

fn split_command_args(command: &str) -> Result<Vec<String>, String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut quote: Option<char> = None;
    let mut token_started = false;

    for ch in command.chars() {
        match (quote, ch) {
            (Some(q), c) if c == q => {
                quote = None;
            }
            (Some(_), c) => {
                token_started = true;
                current.push(c);
            }
            (None, '"') | (None, '\'') => {
                quote = Some(ch);
                token_started = true;
            }
            (None, c) if c.is_whitespace() => {
                if token_started {
                    args.push(std::mem::take(&mut current));
                    token_started = false;
                }
            }
            (None, c) => {
                token_started = true;
                current.push(c);
            }
        }
    }

    if let Some(q) = quote {
        return Err(format!("缺少结束引号: {}", q));
    }
    if token_started {
        args.push(current);
    }
    Ok(args)
}

fn required_arg<'a>(args: &'a [String], index: usize, name: &str) -> Result<&'a str, String> {
    args.get(index)
        .map(|s| s.as_str())
        .ok_or_else(|| format!("缺少参数: {}", name))
}

fn parse_required_i32(args: &[String], index: usize, name: &str) -> Result<i32, String> {
    required_arg(args, index, name)?
        .parse::<i32>()
        .map_err(|_| format!("{} 必须是整数", name))
}

fn parse_optional_i32(value: Option<&String>, default: i32, name: &str) -> Result<i32, String> {
    match value {
        Some(value) => value
            .parse::<i32>()
            .map_err(|_| format!("{} 必须是整数", name)),
        None => Ok(default),
    }
}

pub fn format_duration(secs: i64) -> String {
    let h = secs / 3600;
    let m = (secs % 3600) / 60;
    let s = secs % 60;
    format!("{:02}:{:02}:{:02}", h, m, s)
}
