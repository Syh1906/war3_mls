use crate::player::Player;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const DEFAULT_MAP_VERSION: &str = "local-dev";
pub const DEFAULT_ENV_TYPE: i32 = -1;
pub const DEFAULT_PREBOOK_COUNT: i32 = 128;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RankingEntry {
    #[serde(default = "default_unbound_player_index")]
    pub player_index: i32,
    #[serde(default)]
    pub player_name: String,
    #[serde(default)]
    pub value: i32,
}

fn default_unbound_player_index() -> i32 {
    -1
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomSimulationData {
    pub map_version: String,
    pub env_type: i32,
    pub prebook_count: i32,
    pub rankings_loaded: bool,
    pub rankings: HashMap<i32, Vec<RankingEntry>>,
}

impl RoomSimulationData {
    pub fn default_for_players(players: &HashMap<i32, Player>) -> Self {
        let mut rankings = HashMap::new();
        rankings.insert(-1, build_player_ranking(players, |p| p.map_level.max(0)));
        rankings.insert(
            0,
            build_player_ranking(players, |p| {
                p.map_level.max(0) * 1000 + p.map_exp.max(0) + p.played_count.max(0)
            }),
        );

        Self {
            map_version: DEFAULT_MAP_VERSION.to_string(),
            env_type: DEFAULT_ENV_TYPE,
            prebook_count: DEFAULT_PREBOOK_COUNT,
            rankings_loaded: true,
            rankings,
        }
    }

    pub fn sort_rankings(&mut self) {
        for entries in self.rankings.values_mut() {
            sort_ranking_entries(entries);
        }
    }
}

impl Default for RoomSimulationData {
    fn default() -> Self {
        Self {
            map_version: DEFAULT_MAP_VERSION.to_string(),
            env_type: DEFAULT_ENV_TYPE,
            prebook_count: DEFAULT_PREBOOK_COUNT,
            rankings_loaded: true,
            rankings: HashMap::new(),
        }
    }
}

pub fn sort_ranking_entries(entries: &mut [RankingEntry]) {
    entries.sort_by(|a, b| {
        b.value
            .cmp(&a.value)
            .then_with(|| a.player_index.cmp(&b.player_index))
            .then_with(|| a.player_name.cmp(&b.player_name))
    });
}

fn build_player_ranking<F>(players: &HashMap<i32, Player>, value_fn: F) -> Vec<RankingEntry>
where
    F: Fn(&Player) -> i32,
{
    let mut entries: Vec<RankingEntry> = players
        .values()
        .map(|player| RankingEntry {
            player_index: player.index,
            player_name: player.name.clone(),
            value: value_fn(player),
        })
        .collect();
    sort_ranking_entries(&mut entries);
    entries
}
