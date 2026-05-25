use clap::Parser;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::player::{
    Player, PlayerAchievementData, PlayerActivityData, PlayerCommunityData, PlayerGuildData,
    PlayerPlatformData, PlayerSignInData, PlayerTaskData,
};
use crate::simulation::{RankingEntry, RoomSimulationData};

#[derive(Parser, Debug)]
#[command(name = "mls-sim", version, about = "MLS 云脚本本地模拟环境")]
pub struct Cli {
    #[arg(long, default_value = "127.0.0.1")]
    pub host: String,

    #[arg(long, short, default_value_t = 5000)]
    pub port: u16,

    #[arg(long, short)]
    pub script_dir: Option<String>,

    #[arg(long, default_value = "config.json")]
    pub config: String,

    #[arg(long, help = "Hide the console window (Windows only)")]
    pub console_notwrte: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoRoomConfig {
    pub script_dir: String,
    #[serde(default)]
    pub mode_id: i32,
    #[serde(default)]
    pub map_version: Option<String>,
    #[serde(default)]
    pub env_type: Option<i32>,
    #[serde(default)]
    pub prebook_count: Option<i32>,
    #[serde(default)]
    pub rankings_loaded: Option<bool>,
    #[serde(default)]
    pub rankings: Option<HashMap<i32, Vec<RankingEntry>>>,
    #[serde(default = "default_players")]
    pub players: Vec<PlayerConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerConfig {
    #[serde(default)]
    pub index: i32,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub items: std::collections::HashMap<String, i32>,
    #[serde(default)]
    pub map_level: Option<i32>,
    #[serde(default)]
    pub map_exp: Option<i32>,
    #[serde(default)]
    pub played_count: Option<i32>,
    #[serde(default)]
    pub script_archive: Option<String>,
    #[serde(default)]
    pub common_archive: Option<std::collections::HashMap<String, String>>,
    #[serde(default)]
    pub read_archive: Option<std::collections::HashMap<String, String>>,
    #[serde(default)]
    pub cfg_archive: Option<std::collections::HashMap<String, String>>,
    #[serde(default)]
    pub platform: Option<PlayerPlatformConfig>,
    #[serde(default)]
    pub activity: Option<PlayerActivityConfig>,
    #[serde(default)]
    pub achievements: Option<PlayerAchievementConfig>,
    #[serde(default)]
    pub tasks: Option<PlayerTaskConfig>,
    #[serde(default)]
    pub sign_in: Option<PlayerSignInConfig>,
    #[serde(default)]
    pub community: Option<PlayerCommunityConfig>,
    #[serde(default)]
    pub guild: Option<PlayerGuildConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlayerPlatformConfig {
    #[serde(default)]
    pub guid: Option<i64>,
    #[serde(default)]
    pub plat_level: Option<i32>,
    #[serde(default)]
    pub vip_level: Option<i32>,
    #[serde(default)]
    pub map_vip_level: Option<i32>,
    #[serde(default)]
    pub vip_types: Option<HashMap<i32, i32>>,
    #[serde(default)]
    pub is_author: Option<i32>,
    #[serde(default)]
    pub is_collected: Option<i32>,
    #[serde(default)]
    pub is_backflow: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlayerActivityConfig {
    #[serde(default)]
    pub day_rounds: Option<i32>,
    #[serde(default)]
    pub since_last_game: Option<i32>,
    #[serde(default)]
    pub lottery_counts: Option<HashMap<i32, i32>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlayerAchievementConfig {
    #[serde(default)]
    pub point: Option<i32>,
    #[serde(default)]
    pub done: Option<HashMap<String, i32>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlayerTaskConfig {
    #[serde(default)]
    pub progress: Option<HashMap<i32, crate::player::PlayerTaskProgress>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlayerSignInConfig {
    #[serde(default)]
    pub total: Option<i32>,
    #[serde(default)]
    pub cont_max: Option<i32>,
    #[serde(default)]
    pub cont_cur: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlayerCommunityConfig {
    #[serde(default)]
    pub has_topic: Option<i32>,
    #[serde(default)]
    pub is_manager: Option<i32>,
    #[serde(default)]
    pub topic_count: Option<i32>,
    #[serde(default)]
    pub comment_count: Option<i32>,
    #[serde(default)]
    pub happy_count: Option<i32>,
    #[serde(default)]
    pub best_count: Option<i32>,
    #[serde(default)]
    pub appraise_count: Option<i32>,
    #[serde(default)]
    pub is_pinned: Option<i32>,
    #[serde(default)]
    pub pet_adv_time: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlayerGuildConfig {
    #[serde(default)]
    pub level: Option<i32>,
}

fn default_players() -> Vec<PlayerConfig> {
    vec![PlayerConfig {
        index: 0,
        name: "Player_0".into(),
        items: Default::default(),
        map_level: None,
        map_exp: None,
        played_count: None,
        script_archive: None,
        common_archive: None,
        read_archive: None,
        cfg_archive: None,
        platform: None,
        activity: None,
        achievements: None,
        tasks: None,
        sign_in: None,
        community: None,
        guild: None,
    }]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_true")]
    pub auto_open_browser: bool,
    #[serde(default = "default_archive_dir")]
    pub archive_dir: String,
    pub auto_room: Option<AutoRoomConfig>,
}

fn default_host() -> String {
    "127.0.0.1".into()
}
fn default_port() -> u16 {
    5000
}
fn default_true() -> bool {
    true
}
fn default_archive_dir() -> String {
    "./archives".into()
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
            auto_open_browser: true,
            archive_dir: default_archive_dir(),
            auto_room: None,
        }
    }
}

impl AppConfig {
    pub fn load(cli: &Cli) -> Self {
        let config_path = PathBuf::from(&cli.config);
        let mut config = if config_path.exists() {
            match std::fs::read_to_string(&config_path) {
                Ok(text) => serde_json::from_str::<AppConfig>(&text).unwrap_or_default(),
                Err(_) => AppConfig::default(),
            }
        } else {
            AppConfig::default()
        };

        if cli.host != "127.0.0.1" {
            config.host = cli.host.clone();
        }
        if cli.port != 5000 {
            config.port = cli.port;
        }
        if let Some(ref sd) = cli.script_dir {
            config.auto_room = Some(AutoRoomConfig {
                script_dir: sd.clone(),
                mode_id: 0,
                map_version: None,
                env_type: None,
                prebook_count: None,
                rankings_loaded: None,
                rankings: None,
                players: default_players(),
            });
        }

        config
    }

    pub fn save(&self, path: &str) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)
    }
}

pub fn build_players_from_config(configs: &[PlayerConfig]) -> HashMap<i32, Player> {
    let configs = if configs.is_empty() {
        vec![PlayerConfig {
            index: 0,
            name: "Player_0".into(),
            items: Default::default(),
            map_level: None,
            map_exp: None,
            played_count: Some(0),
            script_archive: None,
            common_archive: None,
            read_archive: None,
            cfg_archive: None,
            platform: None,
            activity: None,
            achievements: None,
            tasks: None,
            sign_in: None,
            community: None,
            guild: None,
        }]
    } else {
        configs.to_vec()
    };

    let mut players = HashMap::new();
    for pc in &configs {
        let mut p = Player::new(pc.index, pc.name.clone());
        if !pc.items.is_empty() {
            p.items = pc.items.clone();
        }
        if let Some(v) = pc.map_level {
            p.map_level = v;
        }
        if let Some(v) = pc.map_exp {
            p.map_exp = v;
        }
        if let Some(v) = pc.played_count {
            p.played_count = v;
        }
        if let Some(ref v) = pc.script_archive {
            p.script_archive = Some(v.clone());
        }
        if let Some(ref v) = pc.common_archive {
            p.common_archive = v.clone();
        }
        if let Some(ref v) = pc.read_archive {
            p.read_archive = v.clone();
        }
        if let Some(ref v) = pc.cfg_archive {
            p.cfg_archive = v.clone();
        }
        apply_player_simulation_config(&mut p, pc);
        players.insert(pc.index, p);
    }
    players
}

pub fn build_room_simulation_from_config(
    config: Option<&AutoRoomConfig>,
    players: &HashMap<i32, Player>,
) -> RoomSimulationData {
    let mut data = RoomSimulationData::default_for_players(players);
    if let Some(config) = config {
        if let Some(ref v) = config.map_version {
            data.map_version = v.clone();
        }
        if let Some(v) = config.env_type {
            data.env_type = v;
        }
        if let Some(v) = config.prebook_count {
            data.prebook_count = v;
        }
        if let Some(v) = config.rankings_loaded {
            data.rankings_loaded = v;
        }
        if let Some(ref v) = config.rankings {
            data.rankings = v.clone();
            data.sort_rankings();
        }
    }
    data
}

fn apply_player_simulation_config(player: &mut Player, config: &PlayerConfig) {
    if let Some(ref v) = config.platform {
        apply_platform_config(&mut player.platform, v);
    }
    if let Some(ref v) = config.activity {
        apply_activity_config(&mut player.activity, v);
    }
    if let Some(ref v) = config.achievements {
        apply_achievement_config(&mut player.achievements, v);
    }
    if let Some(ref v) = config.tasks {
        apply_task_config(&mut player.tasks, v);
    }
    if let Some(ref v) = config.sign_in {
        apply_sign_in_config(&mut player.sign_in, v);
    }
    if let Some(ref v) = config.community {
        apply_community_config(&mut player.community, v);
    }
    if let Some(ref v) = config.guild {
        apply_guild_config(&mut player.guild, v);
    }
}

fn apply_platform_config(data: &mut PlayerPlatformData, config: &PlayerPlatformConfig) {
    if let Some(v) = config.guid {
        data.guid = v;
    }
    if let Some(v) = config.plat_level {
        data.plat_level = v;
    }
    if let Some(v) = config.vip_level {
        data.vip_level = v;
    }
    if let Some(v) = config.map_vip_level {
        data.map_vip_level = v;
    }
    if let Some(ref v) = config.vip_types {
        data.vip_types = v.clone();
    }
    if let Some(v) = config.is_author {
        data.is_author = v;
    }
    if let Some(v) = config.is_collected {
        data.is_collected = v;
    }
    if let Some(v) = config.is_backflow {
        data.is_backflow = v;
    }
}

fn apply_activity_config(data: &mut PlayerActivityData, config: &PlayerActivityConfig) {
    if let Some(v) = config.day_rounds {
        data.day_rounds = v;
    }
    if let Some(v) = config.since_last_game {
        data.since_last_game = v;
    }
    if let Some(ref v) = config.lottery_counts {
        data.lottery_counts = v.clone();
    }
}

fn apply_achievement_config(data: &mut PlayerAchievementData, config: &PlayerAchievementConfig) {
    if let Some(v) = config.point {
        data.point = v;
    }
    if let Some(ref v) = config.done {
        data.done = v.clone();
    }
}

fn apply_task_config(data: &mut PlayerTaskData, config: &PlayerTaskConfig) {
    if let Some(ref v) = config.progress {
        data.progress = v.clone();
    }
}

fn apply_sign_in_config(data: &mut PlayerSignInData, config: &PlayerSignInConfig) {
    if let Some(v) = config.total {
        data.total = v;
    }
    if let Some(v) = config.cont_max {
        data.cont_max = v;
    }
    if let Some(v) = config.cont_cur {
        data.cont_cur = v;
    }
}

fn apply_community_config(data: &mut PlayerCommunityData, config: &PlayerCommunityConfig) {
    if let Some(v) = config.has_topic {
        data.has_topic = v;
    }
    if let Some(v) = config.is_manager {
        data.is_manager = v;
    }
    if let Some(v) = config.topic_count {
        data.topic_count = v;
    }
    if let Some(v) = config.comment_count {
        data.comment_count = v;
    }
    if let Some(v) = config.happy_count {
        data.happy_count = v;
    }
    if let Some(v) = config.best_count {
        data.best_count = v;
    }
    if let Some(v) = config.appraise_count {
        data.appraise_count = v;
    }
    if let Some(v) = config.is_pinned {
        data.is_pinned = v;
    }
    if let Some(v) = config.pet_adv_time {
        data.pet_adv_time = v;
    }
}

fn apply_guild_config(data: &mut PlayerGuildData, config: &PlayerGuildConfig) {
    if let Some(v) = config.level {
        data.level = v;
    }
}
