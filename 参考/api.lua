--- @diagnostic disable: missing-return

--- @alias MsErrorCode
--- | 0	正常
--- | 1	未知错误
--- | 2	房间不存在
--- | 3	玩家不存在
--- | 4	事件Key长度不符合规范
--- | 5	事件Key内容不符合规范
--- | 6	事件Value长度不符合规范
--- | 7	事件Value内容不符合规范
--- | 8	存档Key长度不符合规范
--- | 9	存档Value长度不符合规范
--- | 10	文本内容超过限制
--- | 11	脚本存档超过长度限制
--- | 1259	道具数量不足
--- | 10133	包裹内没有指定的物品

--- @class real : number

--- @alias int64 integer
--- @alias uint32 integer

--- 云脚本服务端运行限制
--- 所有服务端框架和服务端业务代码都运行在受限环境中，编写时必须控制单次执行耗时、内存占用和日志量。
--- * 单次脚本执行超过5秒会退出，原因是 UnsafeCostTime。
--- * 脚本累计超时20次后会退出，原因是 UnsafeTimeout。
--- * 脚本内存占用超过10M会退出，原因是 UnsafeMemory。
--- * 同一位置的脚本报错日志5分钟只记录1条。
--- * 常规日志100秒最多1000条，超过后会熔断；下一个周期恢复记录。
--- * 单条日志内容上限为2000字节，正式环境会忽略 debug 级别日志。

--- 日志
do
    --- @class MsLogger
    Log = {}

    --- 本地调试日志
    --- 正式环境会忽略 debug 级别日志；单条日志内容上限为2000字节。
    --- @param str string|number 格式化字符串
    --- @param ... any 参数
    function Log.Debug(str, ...) end

    --- INFO级日志
    --- 跟踪脚本的核心数据修改；常规日志100秒最多1000条，超过后熔断到下一个周期；单条日志内容上限为2000字节。
    --- @param str string|number 格式化字符串
    --- @param ... any 参数
    function Log.Info(str, ...) end

    --- 脚本错误日志
    --- 记录需要作者修复的逻辑错误或异常；同一位置5分钟只记录1条；单条日志内容上限为2000字节。
    --- @param str string|number 格式化字符串
    --- @param ... any 参数
    function Log.Error(str, ...) end
end

--- 定时器
do
    --- @class MsTimerManager
    Timer = {}

    --- 延后执行
    --- @param time number 延迟时间
    --- @param callback fun() 回调函数
    function Timer.After(time, callback) end

    --- 循环定时器
    --- @param time number 间隔时间
    --- @param callback fun() 回调函数
    --- @return MsTimer timer 定时器对象
    function Timer.NewTicker(time, callback) end

    --- @class MsTimer
    local msTimerMethod = {}

    --- 取消执行
    function msTimerMethod:Cancel() end
end

--- json
do
    --- @class MsJson
    json = {}

    --- 序列化
    --- @param str table 待序列化数据
    --- @return string data 序列化后的数据
    function json.encode(str) end

    --- 反序列化
    --- @param str string 反序列化数据
    --- @return table data 数据
    function json.decode(str) end
end

--- 脚本事件
do
    --- @alias MsEventCallback fun(eventId:integer, eventName:string, eventData:string, playerIndex:integer)

    --- 注册脚本事件
    --- @param eventName string 事件名 不能以_开头 可用数字|字母|冒号 最大32字节
    --- @param eventCallback MsEventCallback 事件回调函数
    --- @overload fun(eventName: '_roomloaded', eventCallback: roomLoadedCallback)
    --- @overload fun(eventName: '_roomover', eventCallback: roomOverCallback)
    --- @overload fun(eventName: '_playerexit', eventCallback: playerExitCallback)
    --- @overload fun(eventName: '_playerleave', eventCallback: playerLeaveCallback)
    --- @overload fun(eventName: '_playerjoin', eventCallback: playerJoinCallback)
    function RegisterEvent(eventName, eventCallback) end

    --- @alias roomLoadedCallback fun(eventId:integer, eventName:'_roomloaded', eventData:'{ "players": [0] }', playerIndex: integer)
    --- @alias roomOverCallback fun(eventId:integer, eventName:'_roomover', eventData:'{ "reason": "GameEnd" }', playerIndex: integer)
    --- @alias playerExitCallback fun(eventId:integer, eventName:'_playerexit', eventData:'{ "reason": "Logout" }', playerIndex: integer)
    --- @alias playerLeaveCallback fun(eventId:integer, eventName:'_playerleave', eventData:'{ "reason": "Disconnect" }', playerIndex: integer)
    --- @alias playerJoinCallback fun(eventId:integer, eventName:'_playerjoin', eventData:'{ "reason": "Connect" }', playerIndex: integer)

    --- 取消脚本事件注册
    --- @param eventId integer 事件ID
    function UnregisterEvent(eventId) end
end

--- 玩家相关
do
    --- 获取玩家昵称
    --- @param playerIndex integer 玩家槽位
    --- @return string name 玩家昵称
    function MsGetPlayerName(playerIndex) end

    --- 获取玩家当前地图等级
    --- @param playerIndex integer 玩家槽位
    --- @return integer level 玩家地图等级
    function MsGetPlayerMapLevel(playerIndex) end

    --- 获取玩家当前地图经验
    --- @param playerIndex integer 玩家槽位
    --- @return integer exp 玩家地图经验
    function MsGetPlayerMapExp(playerIndex) end

    --- 获取玩家当前地图时间
    --- @param playerIndex integer 玩家槽位
    --- @return integer time 玩家地图时间（秒）
    function MsGetPlayedTime(playerIndex) end

    --- 获取玩家测试大厅游玩时间
    --- @param playerIndex integer 玩家槽位
    --- @return integer time 玩家测试大厅游玩时间（秒）
    function MsGetTestPlayTime(playerIndex) end

    --- 获取玩家当前地图次数
    --- @param playerIndex integer 玩家槽位
    --- @return integer count 玩家地图次数
    function MsGetPlayedCount(playerIndex) end
end

--- 对局相关
do
    --- 获取游戏开始时间
    --- @return integer timestamp 游戏开始时间戳
    function MsGetRoomStartTs() end

    --- 获取游戏加载完成时间
    --- @return integer timestamp 游戏加载完成时间戳
    function MsGetRoomLoadedTs() end

    --- 获取游戏已经过去多长时间
    --- @return integer time 游戏对局已过去时间（秒）
    function MsGetRoomGameTime() end

    --- 获取对局中玩家个数
    --- @return integer number 玩家个数
    function MsGetRoomPlayerCount() end

    --- 获取对局模式ID
    --- @return integer id 对局模式id
    function MsGetRoomModeId() end
end

--- 道具相关
do
    --- 获取玩家道具\
    --- 永久道具：-1\
    --- 　未拥有：0\
    --- 叠加道具：道具数量
    --- @param playerIndex integer 玩家槽位
    --- @param key string 道具key
    --- @return integer number 道具状态
    function MsGetPlayerItem(playerIndex, key) end

    --- 消耗玩家道具
    --- @param playerIndex integer 玩家槽位
    --- @param itemInfo string 道具信息（json字符串)
    --- itemInfo示例：{"key1":1, "key2":1}
    --- @return uint32 trans_id 业务id(uint32)
    function MsConsumeItem(playerIndex, itemInfo) end
end

--- 存档相关
do
    --- 获取脚本存档数据\
    --- 脚本不存在或为空时返回nil
    --- @param playerIndex integer 玩家槽位
    --- @return string|nil data 玩家存脚本档数据
    function MsGetScriptArchive(playerIndex) end

    --- 保存脚本存档数据\
    --- 在玩家退出时需要调用保存脚本存档
    --- @param playerIndex integer 玩家槽位
    --- @param scriptData string 脚本存档序列化后的数据（最大1M）
    --- @return MsErrorCode result 操作结果（成功返回0）
    function MsSaveScriptArchive(playerIndex, scriptData) end

    --- 获取普通存档数据\
    --- 存档key不存在，或者存档key对应的数据为空，返回nil\
    --- （普通存档指地图侧的老存档系统）
    --- @param playerIndex integer 玩家槽位
    --- @param key string 存档key
    --- @return string|nil data 存档数据
    function MsGetCommonArchive(playerIndex, key) end

    --- 保存普通存档数据\
    --- 返回值为0，表示操作成功，错误码见错误码列表\
    --- （普通存档指地图侧的老存档系统）
    --- @param playerIndex integer 玩家槽位
    --- @param key string 存档key
    --- @param value string 存档数据
    --- @return MsErrorCode result 操作结果（成功返回0）
    function MsSetCommonArchive(playerIndex, key, value) end

    --- 获取只读存档数据\
    --- 存档key不存在，或者存档key对应的数据为空，返回nil
    --- @param playerIndex integer 玩家槽位
    --- @param key string 存档key
    --- @return string|nil data 存档数据
    function MsGetReadArchive(playerIndex, key) end

    --- 保存只读存档数据\
    --- 返回值为0，表示操作成功，错误码见错误码列表\
    --- （可以通过设置覆盖地图侧的存档，被覆盖后地图侧对该存档只读不可修改）
    --- @param playerIndex integer 玩家槽位
    --- @param key string 存档key
    --- @param value string 存档数据
    --- @return MsErrorCode result 操作结果（成功返回0）
    function MsSetReadArchive(playerIndex, key, value) end

    --- 获取全局只读存档数据\
    --- 存档key不存在，或者存档key对应的数据为空，返回nil
    --- @param playerIndex integer 玩家槽位
    --- @param key string 存档key
    --- @return string|nil data 存档数据
    function MsGetCfgArchive(playerIndex, key) end
end

--- 脚本API
do
    --- 发送脚本事件
    --- @param playerIndex integer 玩家槽位
    --- @param eventName string 事件名 不能以_开头 可用数字|字母|冒号 最大32字节
    --- @param eventValue string 事件数据 最大长度 900字节
    --- @return MsErrorCode result 操作结果（成功返回0）
    function MsSendMlEvent(playerIndex, eventName, eventValue) end

    --- 停止脚本执行\
    --- 会立即停止脚本房间的执行
    --- @param playerIndex integer 玩家槽位
    --- @param reason string 原因
    --- @return MsErrorCode result 操作结果（成功返回0）
    function MsEnd(playerIndex, reason) end
end

--- 玩家平台信息
do
    --- 获取玩家地图UID
    --- @param playerIndex integer 玩家槽位
    --- @return int64 guid 玩家平台UID
    function MsGetPlayerGuid(playerIndex) end

    --- 获取玩家平台等级
    --- @param playerIndex integer 玩家槽位
    --- @return integer level 玩家平台等级
    function MsGetPlayerPlatLevel(playerIndex) end

    --- 获取玩家平台VIP等级
    --- @param playerIndex integer 玩家槽位
    --- @return integer level 玩家VIP等级
    function MsGetPlayerVipLevel(playerIndex) end

    --- 获取玩家地图VIP等级
    --- @param playerIndex integer 玩家槽位
    --- @return integer level 玩家地图VIP等级
    function MsGetPlayerMapVipLevel(playerIndex) end

    --- 获取玩家指定类型的平台VIP状态\
    --- vipType=4 职业选手\
    --- vipType=6 开发者\
    --- vipType=8 新人主播\
    --- vipType=9 闪耀主播\
    --- vipType=10 社区管家
    --- @param playerIndex integer 玩家槽位
    --- @param vipType integer VIP类型编号
    --- @return integer level 对应类型的VIP等级，0表示未开通
    function MsGetPlatVipType(playerIndex, vipType) end

    --- 判断玩家是否是当前地图作者
    --- @param playerIndex integer 玩家槽位
    --- @return integer result 1=是作者，0=非作者
    function MsGetPlayerIsAuthor(playerIndex) end

    --- 判断玩家是否收藏过当前地图
    --- @param playerIndex integer 玩家槽位
    --- @return integer result 1=已收藏，0=未收藏
    function MsGetPlayerIsCollected(playerIndex) end

    --- 判断玩家是否是当前地图的回流用户\
    --- 回流用户定义：曾经流失后重新回到该地图游玩的玩家
    --- @param playerIndex integer 玩家槽位
    --- @return integer result 1=是回流用户，0=非回流用户
    function MsGetPlayerIsBackflow(playerIndex) end
end

--- 地图 / 环境信息
do
    --- 获取当前地图版本号
    --- @return string version 地图版本号，如"1.23"
    function MsGetMapVersion() end

    --- 获取当前运行环境类型\
    --- 0=正式服，1=maptest，2=测试大厅，-1=本地测试
    --- @return integer envType 运行环境类型
    function MsGetEnvType() end

    --- 获取当前地图的测试大厅预约人数\
    --- 该API仅在地图正式上架后下有效
    --- @return integer count 预约玩家人数
    function MsGetPrebookCount() end
end

--- 玩家游戏行为数据
do
    --- 获取玩家当天玩当前地图的总游戏局数\
    --- 统计范围为自然日（按服务器时间）
    --- @param playerIndex integer 玩家槽位
    --- @return integer count 当天游戏总局数
    function MsGetPlayerDayRounds(playerIndex) end

    --- 获取玩家本局游戏距上一局游戏的时间差\
    --- 若玩家是第一次玩该地图，返回-1
    --- @param playerIndex integer 玩家槽位
    --- @return integer time 距上一局游戏的时间差（秒）
    function MsGetPlayerSinceLastGame(playerIndex) end

    --- 获取玩家在当前地图抽取指定宝箱的总次数\
    --- 数据来源：Player_Lottery_Info.lottery_list，进房时随玩家数据一次性下发，对局中为静态快照
    --- @param playerIndex integer 玩家槽位
    --- @param cfgIndex integer 宝箱配置编号（Player_Lottery_Info.cfg_index）
    --- @return integer count 累计抽取总次数
    function MsGetPlayerLotteryCount(playerIndex, cfgIndex) end
end

--- 玩家成就
do
    --- 获取玩家在当前地图的成就点数
    --- @param playerIndex integer 玩家槽位
    --- @return integer point 地图成就点数
    function MsGetPlayerAchievePoint(playerIndex) end

    --- 判断玩家是否完成了指定成就
    --- @param playerIndex integer 玩家槽位
    --- @param achId string 成就ID
    --- @return integer result 1=已完成，0=未完成
    function MsGetPlayerAchieveDone(playerIndex, achId) end
end

--- 玩家地图任务
do
    --- 获取玩家指定地图任务的总进度
    --- @param playerIndex integer 玩家槽位
    --- @param taskId integer 任务ID
    --- @return integer progress 任务总进度值
    function MsGetPlayerTaskTotalProgress(playerIndex, taskId) end

    --- 获取玩家指定地图任务的当前进度
    --- @param playerIndex integer 玩家槽位
    --- @param taskId integer 任务ID
    --- @return integer progress 任务当前进度值
    function MsGetPlayerTaskCurProgress(playerIndex, taskId) end

    --- 判断玩家指定地图任务是否已完成
    --- @param playerIndex integer 玩家槽位
    --- @param taskId integer 任务ID
    --- @return integer result 1=已完成，0=未完成
    function MsGetPlayerTaskDone(playerIndex, taskId) end
end

--- 玩家签到
do
    --- 获取玩家在当前地图的总签到天数
    --- @param playerIndex integer 玩家槽位
    --- @return integer count 总签到天数
    function MsGetPlayerSignInTotal(playerIndex) end

    --- 获取玩家在当前地图的最大连续签到天数
    --- @param playerIndex integer 玩家槽位
    --- @return integer count 最大连续签到天数
    function MsGetPlayerSignInContMax(playerIndex) end

    --- 获取玩家在当前地图的当前连续签到天数
    --- @param playerIndex integer 玩家槽位
    --- @return integer count 当前连续签到天数
    function MsGetPlayerSignInContCur(playerIndex) end
end

--- 玩家社区 / 论坛数据
do
    --- 判断玩家是否在当前地图的社区发过帖子
    --- @param playerIndex integer 玩家槽位
    --- @return integer result 1=发过，0=未发过
    function MsGetPlayerHasTopic(playerIndex) end

    --- 判断玩家是否是当前地图社区的版主
    --- @param playerIndex integer 玩家槽位
    --- @return integer result 1=是版主，0=非版主
    function MsGetPlayerIsManager(playerIndex) end

    --- 获取玩家在当前地图社区的发帖数量
    --- @param playerIndex integer 玩家槽位
    --- @return integer count 发帖数量（上限10）
    function MsGetPlayerTopicCount(playerIndex) end

    --- 获取玩家在当前地图社区的发表回复次数
    --- @param playerIndex integer 玩家槽位
    --- @return integer count 回复次数（上限100）
    function MsGetPlayerCommentCount(playerIndex) end

    --- 获取玩家在当前地图社区累计收到的欢乐数
    --- @param playerIndex integer 玩家槽位
    --- @return integer count 累计收到欢乐数
    function MsGetPlayerHappyCount(playerIndex) end

    --- 获取玩家在当前地图社区的精华帖数量
    --- @param playerIndex integer 玩家槽位
    --- @return integer count 精华帖数量
    function MsGetPlayerBestCount(playerIndex) end

    --- 获取玩家在当前地图社区累计获得的赞数
    --- @param playerIndex integer 玩家槽位
    --- @return integer count 累计获得赞数
    function MsGetPlayerAppraiseCount(playerIndex) end

    --- 判断玩家是否将当前地图在游戏大厅置顶
    --- @param playerIndex integer 玩家槽位
    --- @return integer result 1=当前置顶，0=非置顶
    function MsGetPlayerIsPinned(playerIndex) end

    --- 获取玩家平台宠物探险时间\
    --- 返回-1表示宠物未在探险中
    --- @param playerIndex integer 玩家槽位
    --- @return int64 timestamp 宠物探险开始时间戳（秒）
    function MsGetPlayerPetAdvTime(playerIndex) end
end

--- 玩家公会
do
    --- 获取玩家在当前地图公会中的等级
    --- @param playerIndex integer 玩家槽位
    --- @return integer level 公会等级，未加入公会返回0
    function MsGetPlayerGuildLevel(playerIndex) end
end

--- 地图排行榜
do
    --- 获取玩家当前在指定排行榜上的名次\
    --- rankingNum=-1表示等级榜，0及以上为地图作者自定义存档榜编号\
    --- 返回-1表示排行榜数据尚未加载完成，需稍后重试；返回0表示玩家未上榜
    --- @param playerIndex integer 玩家槽位
    --- @param rankingNum integer 排行榜编号
    --- @return integer rank 当前名次
    function MsGetPlayerRanking(playerIndex, rankingNum) end

    --- 获取玩家在指定排行榜上的分值\
    --- rankingNum=-1表示等级榜，0及以上为地图作者自定义存档榜编号
    --- @param playerIndex integer 玩家槽位
    --- @param rankingNum integer 排行榜编号
    --- @return integer value 该玩家在排行榜上的分值，0表示未上榜
    function MsGetPlayerRankValue(playerIndex, rankingNum) end

    --- 获取指定排行榜指定名次的玩家名称\
    --- rankingNum=-1表示等级榜，0及以上为地图作者自定义存档榜编号\
    --- rank有效范围1~100，超出范围返回空串
    --- @param rankingNum integer 排行榜编号
    --- @param rank integer 名次（1~100）
    --- @return string name 该名次玩家的昵称，空串表示该名次无数据
    function MsGetRankPlayerName(rankingNum, rank) end

    --- 获取指定排行榜指定名次的数值\
    --- rankingNum=-1表示等级榜，0及以上为地图作者自定义存档榜编号\
    --- rank有效范围1~100，超出范围返回0
    --- @param rankingNum integer 排行榜编号
    --- @param rank integer 名次（1~100）
    --- @return integer value 该名次对应的排行榜数值，0表示无数据
    function MsGetRankValue(rankingNum, rank) end
end

--- 工具库
do
    --- 计算字符串的MD5哈希值
    --- @param str string 需要计算的字符串
    --- @return string hash 32位小写MD5哈希字符串
    --- @diagnostic disable-next-line: lowercase-global
    function md5(str) end
end

--- 脚本内置事件
do
    --- @alias MsServerEvent
    --- | '_roomloaded' 房间加载完成
    --- | '_roomover' 房间结束
    --- | '_playerexit' 玩家退出房间
    --- | '_playerleave' 玩家暂时离开游戏，等待断线重连
    --- | '_playerjoin' 玩家重新加入游戏
end

--- 客户端内置事件
do
    --- @alias MsClientEvent
    --- | '_rdata' 可读存档更新事件 事件内容 "可读存档key\t可读存档value"
    --- | '_citemret' 云脚本道具消耗结果 事件内容 '{"trans_id": 1, "errnu":0, "iteminfo": {"itemkey1": 10, "item_key2":20}}'
    --- | '_mlroomfail' 云脚本启动失败（脚本下载失败|脚本加载失败|脚本存档加载失败）	事件内容 " 失败原因"

    --- 云脚本道具消耗结果数据
    --- {
    --- 	"trans_id": 1,				-- 业务id，与MsConsumeItem返回值相同
    --- 	"errnu": 0,					-- 错误码 0：未消耗成功 1259：道具数量不足 10133：包裹内没有指定的物品
    --- 	"iteminfo":					-- 此次业务消耗道具信息，与MsConsumeItem传入参数iteminfo相同
    --- 	{
    --- 		"itemkey1": 10,
    --- 		"item_key2": 20
    --- 	}
    --- }
end
