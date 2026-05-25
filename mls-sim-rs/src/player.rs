use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

fn now_secs() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub index: i32,
    pub name: String,
    #[serde(default = "default_one")]
    pub map_level: i32,
    #[serde(default)]
    pub map_exp: i32,
    #[serde(default)]
    pub played_count: i32,
    #[serde(default)]
    pub test_play_time: i32,
    #[serde(skip_deserializing)]
    pub joined_at: f64,
    #[serde(skip_deserializing)]
    pub is_connected: bool,
    #[serde(default)]
    pub items: HashMap<String, i32>,
    #[serde(default)]
    pub script_archive: Option<String>,
    #[serde(default)]
    pub common_archive: HashMap<String, String>,
    #[serde(default)]
    pub read_archive: HashMap<String, String>,
    #[serde(default)]
    pub cfg_archive: HashMap<String, String>,
    #[serde(default)]
    pub platform: PlayerPlatformData,
    #[serde(default)]
    pub activity: PlayerActivityData,
    #[serde(default)]
    pub achievements: PlayerAchievementData,
    #[serde(default)]
    pub tasks: PlayerTaskData,
    #[serde(default)]
    pub sign_in: PlayerSignInData,
    #[serde(default)]
    pub community: PlayerCommunityData,
    #[serde(default)]
    pub guild: PlayerGuildData,
}

fn default_one() -> i32 {
    1
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerPlatformData {
    pub guid: i64,
    pub plat_level: i32,
    pub vip_level: i32,
    pub map_vip_level: i32,
    pub vip_types: HashMap<i32, i32>,
    pub is_author: i32,
    pub is_collected: i32,
    pub is_backflow: i32,
}

impl PlayerPlatformData {
    pub fn default_for_index(index: i32) -> Self {
        let mut vip_types = HashMap::new();
        vip_types.insert(6, if index == 0 { 1 } else { 0 });
        Self {
            guid: 1_000_000_000_000 + index as i64,
            plat_level: 30,
            vip_level: 1,
            map_vip_level: 1,
            vip_types,
            is_author: if index == 0 { 1 } else { 0 },
            is_collected: 1,
            is_backflow: 0,
        }
    }
}

impl Default for PlayerPlatformData {
    fn default() -> Self {
        Self::default_for_index(0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerActivityData {
    pub day_rounds: i32,
    pub since_last_game: i32,
    pub lottery_counts: HashMap<i32, i32>,
}

impl Default for PlayerActivityData {
    fn default() -> Self {
        Self {
            day_rounds: 3,
            since_last_game: 3600,
            lottery_counts: HashMap::new(),
        }
    }
}

impl PlayerActivityData {
    pub fn lottery_count(&self, cfg_index: i32) -> i32 {
        self.lottery_counts.get(&cfg_index).copied().unwrap_or(1)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerAchievementData {
    pub point: i32,
    pub done: HashMap<String, i32>,
}

impl Default for PlayerAchievementData {
    fn default() -> Self {
        Self {
            point: 120,
            done: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerTaskProgress {
    pub total: i32,
    pub current: i32,
    pub done: i32,
}

impl Default for PlayerTaskProgress {
    fn default() -> Self {
        Self {
            total: 10,
            current: 3,
            done: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlayerTaskData {
    pub progress: HashMap<i32, PlayerTaskProgress>,
}

impl PlayerTaskData {
    pub fn get(&self, task_id: i32) -> PlayerTaskProgress {
        self.progress.get(&task_id).cloned().unwrap_or_default()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerSignInData {
    pub total: i32,
    pub cont_max: i32,
    pub cont_cur: i32,
}

impl Default for PlayerSignInData {
    fn default() -> Self {
        Self {
            total: 7,
            cont_max: 5,
            cont_cur: 2,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerCommunityData {
    pub has_topic: i32,
    pub is_manager: i32,
    pub topic_count: i32,
    pub comment_count: i32,
    pub happy_count: i32,
    pub best_count: i32,
    pub appraise_count: i32,
    pub is_pinned: i32,
    pub pet_adv_time: i64,
}

impl Default for PlayerCommunityData {
    fn default() -> Self {
        Self {
            has_topic: 1,
            is_manager: 0,
            topic_count: 2,
            comment_count: 5,
            happy_count: 20,
            best_count: 1,
            appraise_count: 10,
            is_pinned: 1,
            pet_adv_time: -1,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerGuildData {
    pub level: i32,
}

impl Default for PlayerGuildData {
    fn default() -> Self {
        Self { level: 1 }
    }
}

impl Player {
    pub fn new(index: i32, name: String) -> Self {
        let name = if name.is_empty() {
            format!("Player_{}", index)
        } else {
            name
        };
        Self {
            index,
            name,
            map_level: 1,
            map_exp: 0,
            played_count: 0,
            test_play_time: 0,
            joined_at: now_secs(),
            is_connected: true,
            items: HashMap::new(),
            script_archive: None,
            common_archive: HashMap::new(),
            read_archive: HashMap::new(),
            cfg_archive: HashMap::new(),
            platform: PlayerPlatformData::default_for_index(index),
            activity: PlayerActivityData::default(),
            achievements: PlayerAchievementData::default(),
            tasks: PlayerTaskData::default(),
            sign_in: PlayerSignInData::default(),
            community: PlayerCommunityData::default(),
            guild: PlayerGuildData::default(),
        }
    }

    pub fn played_time(&self) -> i32 {
        (now_secs() - self.joined_at) as i32
    }

    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "index": self.index,
            "name": self.name,
            "map_level": self.map_level,
            "map_exp": self.map_exp,
            "played_time": self.played_time(),
            "played_count": self.played_count,
            "test_play_time": self.test_play_time,
            "is_connected": self.is_connected,
            "items": self.items,
            "script_archive": self.script_archive,
            "common_archive": self.common_archive,
            "read_archive": self.read_archive,
            "cfg_archive": self.cfg_archive,
            "platform": self.platform,
            "activity": self.activity,
            "achievements": self.achievements,
            "tasks": self.tasks,
            "sign_in": self.sign_in,
            "community": self.community,
            "guild": self.guild,
        })
    }
}
