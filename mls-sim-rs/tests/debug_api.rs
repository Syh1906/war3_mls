use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use http_body_util::BodyExt;
use mls_sim::config::{self, AppConfig, AutoRoomConfig, PlayerConfig};
use mls_sim::player::Player;
use mls_sim::room::{LogEntry, RoomManager};
use mls_sim::simulation::RankingEntry;
use std::collections::HashMap;
use std::fs;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use tower::ServiceExt;

fn test_app() -> axum::Router {
    let manager = Arc::new(RwLock::new(RoomManager::new()));
    let config = Arc::new(RwLock::new(AppConfig::default()));
    mls_sim::bridge::build_bridge_router(manager, config)
}

async fn json_response(
    app: axum::Router,
    method: Method,
    uri: &str,
    body: Option<serde_json::Value>,
) -> (StatusCode, serde_json::Value) {
    let mut request = Request::builder().method(method).uri(uri);
    if body.is_some() {
        request = request.header("content-type", "application/json");
    }
    let request = request
        .body(match body {
            Some(value) => Body::from(value.to_string()),
            None => Body::empty(),
        })
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let value = serde_json::from_slice(&bytes).unwrap();
    (status, value)
}

fn create_room_with_logs(manager: &Arc<RwLock<RoomManager>>) {
    let script_dir =
        std::env::temp_dir().join(format!("mls-sim-debug-api-test-{}", std::process::id()));
    fs::create_dir_all(&script_dir).unwrap();
    fs::write(script_dir.join("main.lua"), "Log.Info('boot ok')\n").unwrap();

    let mut players = HashMap::new();
    players.insert(0, Player::new(0, "Player_0".to_string()));

    let room_id = manager.write().unwrap().create_room(
        script_dir,
        0,
        players,
        std::env::temp_dir().to_string_lossy().into_owned(),
    );
    assert_eq!(room_id, "room-001");

    let manager_guard = manager.read().unwrap();
    let room = manager_guard.get_room("room-001").unwrap();
    let mut shared = room.shared.write().unwrap();
    shared.log_buffer.push_back(LogEntry {
        timestamp: 10.0,
        level: "INF".to_string(),
        source: "System".to_string(),
        message: "first boot".to_string(),
        room_id: "room-001".to_string(),
        player_index: -1,
    });
    shared.log_buffer.push_back(LogEntry {
        timestamp: 20.0,
        level: "ERR".to_string(),
        source: "Lua".to_string(),
        message: "script failed".to_string(),
        room_id: "room-001".to_string(),
        player_index: -1,
    });
}

fn wait_for_room_stop(manager: &Arc<RwLock<RoomManager>>, room_id: &str) {
    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(3) {
        let stopped = {
            let manager = manager.read().unwrap();
            let room = manager.get_room(room_id).unwrap();
            let shared = room.shared.read().unwrap();
            !matches!(
                shared.status,
                mls_sim::room::RoomStatus::Created | mls_sim::room::RoomStatus::Running
            )
        };
        if stopped {
            return;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    panic!("room {room_id} did not stop in time");
}

#[test]
fn lua_new_api_returns_default_simulation_data() {
    let manager = Arc::new(RwLock::new(RoomManager::new()));
    let script_dir =
        std::env::temp_dir().join(format!("mls-sim-new-api-default-{}", std::process::id()));
    fs::create_dir_all(&script_dir).unwrap();
    fs::write(
        script_dir.join("main.lua"),
        r#"
local result = {
    map_version = MsGetMapVersion(),
    env_type = MsGetEnvType(),
    prebook_count = MsGetPrebookCount(),
    guid = MsGetPlayerGuid(0),
    plat_level = MsGetPlayerPlatLevel(0),
    vip_level = MsGetPlayerVipLevel(0),
    map_vip_level = MsGetPlayerMapVipLevel(0),
    dev_vip = MsGetPlatVipType(0, 6),
    is_author = MsGetPlayerIsAuthor(0),
    is_collected = MsGetPlayerIsCollected(0),
    is_backflow = MsGetPlayerIsBackflow(0),
    day_rounds = MsGetPlayerDayRounds(0),
    since_last_game = MsGetPlayerSinceLastGame(0),
    lottery_count = MsGetPlayerLotteryCount(0, 1001),
    achieve_point = MsGetPlayerAchievePoint(0),
    achieve_done = MsGetPlayerAchieveDone(0, "A1"),
    task_total = MsGetPlayerTaskTotalProgress(0, 10),
    task_cur = MsGetPlayerTaskCurProgress(0, 10),
    task_done = MsGetPlayerTaskDone(0, 10),
    sign_total = MsGetPlayerSignInTotal(0),
    sign_max = MsGetPlayerSignInContMax(0),
    sign_cur = MsGetPlayerSignInContCur(0),
    has_topic = MsGetPlayerHasTopic(0),
    is_manager = MsGetPlayerIsManager(0),
    topic_count = MsGetPlayerTopicCount(0),
    comment_count = MsGetPlayerCommentCount(0),
    happy_count = MsGetPlayerHappyCount(0),
    best_count = MsGetPlayerBestCount(0),
    appraise_count = MsGetPlayerAppraiseCount(0),
    is_pinned = MsGetPlayerIsPinned(0),
    pet_adv_time = MsGetPlayerPetAdvTime(0),
    guild_level = MsGetPlayerGuildLevel(0),
    player_rank = MsGetPlayerRanking(0, -1),
    player_rank_value = MsGetPlayerRankValue(0, -1),
    rank_name = MsGetRankPlayerName(-1, 1),
    rank_value = MsGetRankValue(-1, 1),
    missing_rank_name = MsGetRankPlayerName(-1, 101),
    missing_rank_value = MsGetRankValue(-1, 101),
    common_set = MsSetCommonArchive(0, "new_api_result", "ok"),
    missing_common_set = MsSetCommonArchive(99, "new_api_result", "bad"),
    hash = md5("abc"),
    missing_guid = MsGetPlayerGuid(99),
}
MsSetCommonArchive(0, "new_api_json", json.encode(result))
MsEnd(0, "done")
"#,
    )
    .unwrap();

    let mut players = HashMap::new();
    players.insert(0, Player::new(0, "Player_0".to_string()));
    let archive_dir = std::env::temp_dir().to_string_lossy().into_owned();
    let room_id = manager
        .write()
        .unwrap()
        .create_room(script_dir, 0, players, archive_dir);
    wait_for_room_stop(&manager, &room_id);

    let manager_guard = manager.read().unwrap();
    let room = manager_guard.get_room(&room_id).unwrap();
    let shared = room.shared.read().unwrap();
    let result_json = shared
        .players
        .get(&0)
        .unwrap()
        .common_archive
        .get("new_api_json")
        .unwrap();
    let result: serde_json::Value = serde_json::from_str(result_json).unwrap();

    assert_eq!(result["map_version"], "local-dev");
    assert_eq!(result["env_type"], -1);
    assert_eq!(result["prebook_count"], 128);
    assert_eq!(result["guid"], 1_000_000_000_000i64);
    assert_eq!(result["plat_level"], 30);
    assert_eq!(result["vip_level"], 1);
    assert_eq!(result["map_vip_level"], 1);
    assert_eq!(result["dev_vip"], 1);
    assert_eq!(result["is_author"], 1);
    assert_eq!(result["is_collected"], 1);
    assert_eq!(result["is_backflow"], 0);
    assert_eq!(result["day_rounds"], 3);
    assert_eq!(result["since_last_game"], 3600);
    assert_eq!(result["lottery_count"], 1);
    assert_eq!(result["achieve_point"], 120);
    assert_eq!(result["achieve_done"], 0);
    assert_eq!(result["task_total"], 10);
    assert_eq!(result["task_cur"], 3);
    assert_eq!(result["task_done"], 0);
    assert_eq!(result["sign_total"], 7);
    assert_eq!(result["sign_max"], 5);
    assert_eq!(result["sign_cur"], 2);
    assert_eq!(result["has_topic"], 1);
    assert_eq!(result["is_manager"], 0);
    assert_eq!(result["topic_count"], 2);
    assert_eq!(result["comment_count"], 5);
    assert_eq!(result["happy_count"], 20);
    assert_eq!(result["best_count"], 1);
    assert_eq!(result["appraise_count"], 10);
    assert_eq!(result["is_pinned"], 1);
    assert_eq!(result["pet_adv_time"], -1);
    assert_eq!(result["guild_level"], 1);
    assert_eq!(result["player_rank"], 1);
    assert_eq!(result["player_rank_value"], 1);
    assert_eq!(result["rank_name"], "Player_0");
    assert_eq!(result["rank_value"], 1);
    assert_eq!(result["missing_rank_name"], "");
    assert_eq!(result["missing_rank_value"], 0);
    assert_eq!(result["common_set"], 0);
    assert_eq!(result["missing_common_set"], 3);
    assert_eq!(result["hash"], "900150983cd24fb0d6963f7d28e17f72");
    assert_eq!(result["missing_guid"], 0);
    assert_eq!(
        shared
            .players
            .get(&0)
            .unwrap()
            .common_archive
            .get("new_api_result")
            .unwrap(),
        "ok"
    );
}

#[test]
fn lua_new_api_uses_config_overrides_and_unloaded_rankings() {
    let player_config = PlayerConfig {
        index: 1,
        name: "Bob".to_string(),
        items: Default::default(),
        map_level: Some(9),
        map_exp: Some(50),
        played_count: Some(4),
        script_archive: None,
        common_archive: None,
        read_archive: None,
        cfg_archive: None,
        platform: Some(config::PlayerPlatformConfig {
            guid: Some(42),
            plat_level: Some(88),
            vip_level: Some(7),
            map_vip_level: Some(6),
            vip_types: Some(HashMap::from([(4, 3)])),
            is_author: Some(0),
            is_collected: Some(0),
            is_backflow: Some(1),
        }),
        activity: Some(config::PlayerActivityConfig {
            day_rounds: Some(11),
            since_last_game: Some(-1),
            lottery_counts: Some(HashMap::from([(5, 12)])),
        }),
        achievements: Some(config::PlayerAchievementConfig {
            point: Some(222),
            done: Some(HashMap::from([("ACH".to_string(), 1)])),
        }),
        tasks: Some(config::PlayerTaskConfig {
            progress: Some(HashMap::from([(
                7,
                mls_sim::player::PlayerTaskProgress {
                    total: 99,
                    current: 44,
                    done: 1,
                },
            )])),
        }),
        sign_in: Some(config::PlayerSignInConfig {
            total: Some(30),
            cont_max: Some(20),
            cont_cur: Some(8),
        }),
        community: Some(config::PlayerCommunityConfig {
            has_topic: Some(0),
            is_manager: Some(1),
            topic_count: Some(9),
            comment_count: Some(99),
            happy_count: Some(77),
            best_count: Some(6),
            appraise_count: Some(55),
            is_pinned: Some(0),
            pet_adv_time: Some(123456),
        }),
        guild: Some(config::PlayerGuildConfig { level: Some(5) }),
    };
    let room_config = AutoRoomConfig {
        script_dir: "D:/dummy".to_string(),
        mode_id: 0,
        map_version: Some("2.5.1".to_string()),
        env_type: Some(2),
        prebook_count: Some(456),
        rankings_loaded: Some(false),
        rankings: Some(HashMap::from([(
            0,
            vec![RankingEntry {
                player_index: 1,
                player_name: "Bob".to_string(),
                value: 999,
            }],
        )])),
        players: vec![player_config],
    };

    let players = config::build_players_from_config(&room_config.players);
    let simulation = config::build_room_simulation_from_config(Some(&room_config), &players);
    let player = players.get(&1).unwrap();

    assert_eq!(simulation.map_version, "2.5.1");
    assert_eq!(simulation.env_type, 2);
    assert_eq!(simulation.prebook_count, 456);
    assert!(!simulation.rankings_loaded);
    assert_eq!(simulation.rankings.get(&0).unwrap()[0].value, 999);
    assert_eq!(player.platform.guid, 42);
    assert_eq!(player.platform.plat_level, 88);
    assert_eq!(player.platform.vip_types.get(&4), Some(&3));
    assert_eq!(player.activity.lottery_count(5), 12);
    assert_eq!(player.achievements.done.get("ACH"), Some(&1));
    assert_eq!(player.tasks.get(7).current, 44);
    assert_eq!(player.sign_in.total, 30);
    assert_eq!(player.community.pet_adv_time, 123456);
    assert_eq!(player.guild.level, 5);

    let manager = Arc::new(RwLock::new(RoomManager::new()));
    let script_dir =
        std::env::temp_dir().join(format!("mls-sim-new-api-config-{}", std::process::id()));
    fs::create_dir_all(&script_dir).unwrap();
    fs::write(
        script_dir.join("main.lua"),
        r#"
local result = {
    map_version = MsGetMapVersion(),
    env_type = MsGetEnvType(),
    prebook_count = MsGetPrebookCount(),
    guid = MsGetPlayerGuid(1),
    plat_level = MsGetPlayerPlatLevel(1),
    vip_type = MsGetPlatVipType(1, 4),
    day_rounds = MsGetPlayerDayRounds(1),
    since_last_game = MsGetPlayerSinceLastGame(1),
    lottery_count = MsGetPlayerLotteryCount(1, 5),
    achieve_point = MsGetPlayerAchievePoint(1),
    achieve_done = MsGetPlayerAchieveDone(1, "ACH"),
    task_total = MsGetPlayerTaskTotalProgress(1, 7),
    task_cur = MsGetPlayerTaskCurProgress(1, 7),
    task_done = MsGetPlayerTaskDone(1, 7),
    sign_total = MsGetPlayerSignInTotal(1),
    is_manager = MsGetPlayerIsManager(1),
    pet_adv_time = MsGetPlayerPetAdvTime(1),
    guild_level = MsGetPlayerGuildLevel(1),
    player_rank = MsGetPlayerRanking(1, 0),
    player_rank_value = MsGetPlayerRankValue(1, 0),
    rank_name = MsGetRankPlayerName(0, 1),
    rank_value = MsGetRankValue(0, 1),
}
MsSetCommonArchive(1, "configured_api_json", json.encode(result))
MsEnd(1, "done")
"#,
    )
    .unwrap();
    let room_id = manager.write().unwrap().create_room_with_simulation(
        script_dir,
        0,
        players,
        simulation,
        std::env::temp_dir().to_string_lossy().into_owned(),
    );
    wait_for_room_stop(&manager, &room_id);

    let manager_guard = manager.read().unwrap();
    let room = manager_guard.get_room(&room_id).unwrap();
    let shared = room.shared.read().unwrap();
    let result_json = shared
        .players
        .get(&1)
        .unwrap()
        .common_archive
        .get("configured_api_json")
        .unwrap();
    let result: serde_json::Value = serde_json::from_str(result_json).unwrap();

    assert_eq!(result["map_version"], "2.5.1");
    assert_eq!(result["env_type"], 2);
    assert_eq!(result["prebook_count"], 456);
    assert_eq!(result["guid"], 42);
    assert_eq!(result["plat_level"], 88);
    assert_eq!(result["vip_type"], 3);
    assert_eq!(result["day_rounds"], 11);
    assert_eq!(result["since_last_game"], -1);
    assert_eq!(result["lottery_count"], 12);
    assert_eq!(result["achieve_point"], 222);
    assert_eq!(result["achieve_done"], 1);
    assert_eq!(result["task_total"], 99);
    assert_eq!(result["task_cur"], 44);
    assert_eq!(result["task_done"], 1);
    assert_eq!(result["sign_total"], 30);
    assert_eq!(result["is_manager"], 1);
    assert_eq!(result["pet_adv_time"], 123456);
    assert_eq!(result["guild_level"], 5);
    assert_eq!(result["player_rank"], -1);
    assert_eq!(result["player_rank_value"], -1);
    assert_eq!(result["rank_name"], "Bob");
    assert_eq!(result["rank_value"], 999);
}

#[tokio::test]
async fn debug_logs_support_filters_and_limit() {
    let manager = Arc::new(RwLock::new(RoomManager::new()));
    let config = Arc::new(RwLock::new(AppConfig::default()));
    create_room_with_logs(&manager);
    let app = mls_sim::bridge::build_bridge_router(manager, config);

    let (status, body) = json_response(
        app,
        Method::GET,
        "/api/debug/rooms/room-001/logs?level=ERR&q=script&since=15&limit=1",
        None,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["ok"], true);
    assert_eq!(body["count"], 1);
    assert_eq!(body["total"], 1);
    assert_eq!(body["logs"][0]["level"], "ERR");
    assert_eq!(body["logs"][0]["message"], "script failed");
}

#[tokio::test]
async fn clear_debug_logs_only_clears_room_log_buffer() {
    let manager = Arc::new(RwLock::new(RoomManager::new()));
    let config = Arc::new(RwLock::new(AppConfig::default()));
    create_room_with_logs(&manager);
    let app = mls_sim::bridge::build_bridge_router(manager, config);

    let (status, body) = json_response(
        app.clone(),
        Method::POST,
        "/api/debug/rooms/room-001/logs/clear",
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["cleared"], 2);

    let (status, body) =
        json_response(app, Method::GET, "/api/debug/rooms/room-001/logs", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["count"], 0);
}

#[tokio::test]
async fn restart_room_returns_new_room_id_and_service_restart_is_unsupported() {
    let manager = Arc::new(RwLock::new(RoomManager::new()));
    let config = Arc::new(RwLock::new(AppConfig::default()));
    create_room_with_logs(&manager);
    let app = mls_sim::bridge::build_bridge_router(manager, config);

    let (status, body) = json_response(
        app.clone(),
        Method::POST,
        "/api/debug/rooms/room-001/restart",
        Some(serde_json::json!({"reason": "test restart"})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["old_room_id"], "room-001");
    assert_eq!(body["room_id"], "room-002");
    assert_eq!(body["status"], "restarted");

    let (status, body) = json_response(app, Method::POST, "/api/debug/service/restart", None).await;
    assert_eq!(status, StatusCode::NOT_IMPLEMENTED);
    assert_eq!(body["ok"], false);
    assert_eq!(body["errnu"], 1);
}

#[tokio::test]
async fn debug_api_returns_not_found_for_unknown_room() {
    let app = test_app();

    let (status, body) =
        json_response(app, Method::GET, "/api/debug/rooms/missing/logs", None).await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["ok"], false);
    assert_eq!(body["errnu"], 2);
}
