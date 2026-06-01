// Mirrors upstream core/src/mindustry/ui. Implemented incrementally from D:\MDT\mindustry-upstream-v157.4.

pub mod bar;
pub mod dialogs;
pub mod displayable;
pub mod fonts;
pub mod mobile_button;
pub mod styles;
pub mod warning_bar;

pub const UPSTREAM_BUNDLE_PROPERTIES_SOURCE_PATH: &str = "core/assets/bundles/bundle.properties";

pub const UPSTREAM_MENU_BUNDLE_ENTRIES: &[(&str, &str)] = &[
    ("play", "Play"),
    ("level.highscore", "High Score: [accent]{0}"),
    ("level.mode", "Gamemode:"),
    ("campaign", "Campaign"),
    ("joingame", "Join Game"),
    ("hostserver", "Host Multiplayer Game"),
    ("customgame", "Custom Game"),
    ("savegame", "Save Game"),
    ("loadgame", "Load Game"),
    ("database.button", "Database"),
    ("database", "Core Database"),
    ("schematics", "Schematics"),
    ("techtree", "Tech Tree"),
    ("techtree.select", "Tech Tree Selection"),
    ("locked", "Locked"),
    ("completed", "[accent]Researched"),
    ("complete", "[lightgray]Complete:"),
    ("about.button", "About"),
    ("discord", "Join the Mindustry Discord!"),
    ("editor", "Editor"),
    ("maps", "Maps"),
    ("maps.none", "[lightgray]No maps found!"),
    ("name", "Name:"),
    ("search", "Search:"),
    ("refresh", "Refresh"),
    ("hosts.refresh", "Refresh"),
    ("hosts.discovering.any", "Discovering games"),
    ("hosts.none", "[lightgray]No local games found!"),
    ("servers.local", "Local Servers"),
    ("servers.local.steam", "Open Games & Local Servers"),
    ("servers.remote", "Remote Servers"),
    ("servers.global", "Community Servers"),
    ("servers.community", "Community Servers"),
    ("servers.showhidden", "Show Hidden Servers"),
    ("server.saved", "Saved Servers"),
    ("server.add", "Add Server"),
    ("server.edit", "Edit Server"),
    ("server.delete", "Are you sure you want to delete this server?"),
    ("server.version", "[gray]v{0} {1}"),
    ("server.versions", "Your version:[accent] {0}[]\\nServer version:[accent] {1}[]"),
    ("server.kicked.clientOutdated", "Outdated client! Update your game!"),
    ("server.kicked.serverOutdated", "Outdated server! Ask the host to update!"),
    ("joingame.ip", "Address:"),
    ("none.found", "[lightgray]<none found>"),
    ("none", "<none>"),
    ("schematic.search", "Search schematics..."),
    ("schematic.info", "{0}x{1}, {2} blocks"),
    (
        "schematic.disabled",
        "[scarlet]Schematics disabled[]\\nYou are not allowed to use schematics on this [accent]map[] or [accent]server.",
    ),
    ("schematic.import", "Import Schematic..."),
    ("schematic.exportfile", "Export File"),
    ("schematic.importfile", "Import File"),
    ("schematic.browseworkshop", "Browse Workshop"),
    ("schematic.copy", "Copy to Clipboard"),
    ("schematic.copy.import", "Import from Clipboard"),
    ("schematic.shareworkshop", "Share on Workshop"),
    ("schematic.edit", "Edit Schematic"),
    ("schematic.tags", "Tags:"),
    ("schematic.edittags", "Edit Tags"),
    ("schematic.addtag", "Add Tag"),
    ("schematic.texttag", "Text Tag"),
    ("schematic.icontag", "Icon Tag"),
    ("schematic.renametag", "Rename Tag"),
    ("schematic.tagged", "{0} tagged"),
    ("schematic.tagdelconfirm", "Delete this tag completely?"),
    ("schematic.tagexists", "That tag already exists."),
    ("editor.import", "Import..."),
    ("editor.export", "Export..."),
    ("edit", "Edit"),
    ("database.patched", "Modified by data patches."),
    ("viewfields", "View Content Fields"),
    ("info.title", "Info"),
    ("category.purpose", "Purpose"),
    ("category.general", "General"),
    (
        "credits.text",
        "Created by [royal]Anuken[] - [sky]anukendev@gmail.com[]",
    ),
    ("credits", "Credits"),
    ("contributors", "Translators and Contributors"),
    (
        "link.discord.description",
        "The official Mindustry Discord chatroom",
    ),
    ("link.reddit.description", "The Mindustry subreddit"),
    ("link.github.description", "Game source code"),
    ("link.changelog.description", "List of update changes"),
    (
        "link.dev-builds.description",
        "Unstable development builds",
    ),
    (
        "link.trello.description",
        "Official Trello board for planned features",
    ),
    (
        "link.itch.io.description",
        "itch.io page with PC downloads",
    ),
    ("link.google-play.description", "Google Play store listing"),
    ("link.f-droid.description", "F-Droid listing"),
    ("link.wiki.description", "Official Mindustry wiki"),
    ("link.suggestions.description", "Suggest new features"),
    ("link.bug.description", "Found one? Report it here"),
    ("host.invalid", "[scarlet]Can't connect to host."),
    ("server.waiting", "Waiting for server"),
    ("warning", "Warning!"),
    ("servers.disclaimer", "Community servers are [accent]not[] owned or controlled by the developer.\n\nServers may contain user-generated content that is not appropriate for all ages."),
    ("confirm", "Confirm"),
    ("ok", "OK"),
    ("cancel", "Cancel"),
    ("back", "Back"),
    ("delete", "Delete"),
    ("view.workshop", "View In Workshop"),
    ("customize", "Customize Rules"),
    ("unknown", "Unknown"),
    ("workshop", "Workshop"),
    ("custom", "Custom"),
    ("builtin", "Built-In"),
    ("modded", "Modded"),
    ("mods", "Mods"),
    ("mods.name", "Mod:"),
    ("mods.none", "[lightgray]No mods found!"),
    ("mods.guide", "Modding Guide"),
    ("mods.openfolder", "Open Folder"),
    ("mods.viewcontent", "View Content"),
    ("mods.browser", "Mod Browser"),
    ("mods.browser.add", "Install"),
    ("mods.browser.reinstall", "Reinstall"),
    ("mods.browser.view-releases", "View Releases"),
    ("mods.browser.latest", "[lightgray][Latest]"),
    ("mods.browser.releases", "Releases"),
    ("mods.browser.fetching", "Fetching Releases..."),
    (
        "mods.browser.noreleases",
        "[scarlet]No Releases Found\\n[accent]Couldn't find any releases for this mod. Check if the mod's repository has any releases published.",
    ),
    ("mods.browser.sortdate", "Sort by recent"),
    ("mods.browser.sortstars", "Sort by stars"),
    ("mods.github.open", "Repo"),
    ("mods.github.open-release", "Release Page"),
    ("mods.search", "Search mods..."),
    ("mod.import", "Import Mod"),
    ("mod.import.file", "Import File"),
    ("mod.import.github", "Import From GitHub"),
    ("mod.version", "Version:"),
    ("mod.content", "Content:"),
    ("editor.name", "Name:"),
    ("editor.author", "Author:"),
    ("editor.description", "Description:"),
    ("editor.openin", "Open In Editor"),
    ("editor.mapinfo", "Map Info"),
    ("editor.mapname", "Map Name:"),
    ("editor.newmap", "New Map"),
    ("editor.importmap", "Import Map"),
    ("editor.search", "Search maps..."),
    ("editor.filters", "Filter Maps"),
    ("editor.filters.mode", "Gamemodes:"),
    ("editor.filters.priorities", "Priorities:"),
    ("editor.filters.type", "Map Type:"),
    ("editor.filters.search", "Search In:"),
    ("editor.filters.author", "Author"),
    ("editor.filters.description", "Description"),
    ("editor.filters.modname", "Mod Name"),
    ("editor.filters.prioritizemod", "Mod Priority"),
    ("editor.filters.prioritizecustom", "Custom Priority"),
    ("editor.filters.planetselect", "Planet Selection"),
    ("rules.anyenv", "<Any>"),
    ("mode.help.title", "Description of modes"),
    ("mode.custom", "Custom Rules"),
    ("mode.survival.name", "Survival"),
    (
        "mode.survival.description",
        "The normal mode. Limited resources and automatic incoming waves.\\n[gray]Requires enemy spawns in the map to play.",
    ),
    ("mode.sandbox.name", "Sandbox"),
    (
        "mode.sandbox.description",
        "Infinite resources and no timer for waves.",
    ),
    ("mode.pvp.name", "PvP"),
    (
        "mode.pvp.description",
        "Fight against other players locally.\\n[gray]Requires at least 2 differently-colored cores in the map to play.",
    ),
    ("mode.attack.name", "Attack"),
    (
        "mode.attack.description",
        "Destroy the enemy's base. \\n[gray]Requires a red core in the map to play.",
    ),
    ("rules.waves", "Waves"),
    ("rules.wavesending", "Wave Sending"),
    ("rules.wavetimer", "Wave Timer"),
    ("rules.waitForWaveToEnd", "Waves Wait for Enemies"),
    ("rules.attack", "Attack Mode"),
    ("rules.corecapture", "Capture Core On Destruction"),
    ("rules.infiniteresources", "Infinite Resources"),
    ("rules.schematic", "Schematics Allowed"),
    ("rules.buildcostmultiplier", "Build Cost Multiplier"),
    ("rules.buildspeedmultiplier", "Build Speed Multiplier"),
    (
        "rules.enemycorebuildradius",
        "Core No-Build Radius:[lightgray] (tiles)",
    ),
    ("rules.title.waves", "Waves"),
    ("rules.title.resourcesbuilding", "Resources & Building"),
    ("rules.title.enemy", "Enemies"),
    ("rules.title.planet", "Planet"),
    ("settings", "Settings"),
    ("settings.game", "Game"),
    ("settings.graphics", "Graphics"),
    ("settings.sound", "Sound"),
    ("settings.language", "Language"),
    ("settings.controls", "Controls"),
    ("settings.data", "Game Data"),
    ("settings.reset", "Reset to Defaults"),
    ("settings.rebind", "Rebind"),
    ("settings.resetKey", "Reset"),
    ("settings.cleardata", "Clear Game Data..."),
    ("settings.clearplanetdata", "Clear Planet Data"),
    ("settings.clearsaves", "Clear Saves"),
    ("settings.clearresearch", "Clear Research"),
    ("settings.clearcampaignsaves", "Clear Campaign Saves"),
    ("settings.clearplanetresearch", "Clear Planet Research"),
    (
        "settings.clearplanetcampaignsaves",
        "Clear Planet Campaign Saves",
    ),
    ("settings.planetselect", "Planet: {0}"),
    ("settings.clearall.confirm", "[scarlet]WARNING![]\nThis will clear all data, including saves, maps, unlocks and keybinds.\nOnce you press 'ok' the game will wipe all data and automatically exit."),
    (
        "settings.clearsaves.confirm",
        "Are you sure you want to clear all your saves?",
    ),
    (
        "settings.clearresearch.confirm",
        "Are you sure you want to clear all of your campaign research?",
    ),
    (
        "settings.clearcampaignsaves.confirm",
        "Are you sure you want to clear all of your campaign saves?",
    ),
    (
        "settings.clearplanetresearch.confirm",
        "Are you sure you want to clear {0}'s research?",
    ),
    (
        "settings.clearplanetcampaignsaves.confirm",
        "Are you sure you want to clear {0}'s campaign saves?",
    ),
    ("quit", "Quit"),
];

pub const UPSTREAM_MENU_BUNDLE_ZH_CN_ENTRIES: &[(&str, &str)] = &[
    ("play", "开始游戏"),
    ("level.highscore", "最高分：[accent]{0}"),
    ("level.mode", "游戏模式"),
    ("campaign", "战役模式"),
    ("joingame", "加入游戏"),
    ("hostserver", "创建联机游戏"),
    ("customgame", "自定义游戏"),
    ("savegame", "保存游戏"),
    ("loadgame", "载入游戏"),
    ("database.button", "数据库"),
    ("database", "核心数据库"),
    ("schematics", "蓝图"),
    ("techtree", "科技树"),
    ("techtree.select", "切换科技树"),
    ("locked", "锁定"),
    ("completed", "[accent]已研究"),
    ("complete", "[lightgray]解锁条件："),
    ("about.button", "关于"),
    ("discord", "加入 Mindustry 的 Discord！"),
    ("editor", "地图编辑器"),
    ("maps", "地图"),
    ("maps.none", "[lightgray]< 未找到地图 >"),
    ("name", "名称："),
    ("search", "搜索："),
    ("refresh", "刷新"),
    ("hosts.refresh", "刷新"),
    ("hosts.discovering.any", "搜索服务器中…"),
    ("hosts.none", "[lightgray]未找到局域网服务器！"),
    ("servers.local", "本地服务器"),
    ("servers.local.steam", "社区服务器与本地服务器"),
    ("servers.remote", "远程服务器"),
    ("servers.global", "社区服务器"),
    ("servers.community", "社区服务器"),
    ("servers.showhidden", "显示隐藏服务器"),
    ("server.saved", "已保存的服务器"),
    ("server.add", "添加服务器"),
    ("server.edit", "编辑服务器"),
    ("server.delete", "确定要删除这个服务器吗？"),
    ("server.version", "[gray]v{0} {1}"),
    ("server.versions", "你的版本：[accent] {0}[]\\n服务器版本：[accent] {1}[]"),
    ("server.kicked.clientOutdated", "客户端版本过旧！请更新游戏！"),
    ("server.kicked.serverOutdated", "服务器版本过旧！请让房主更新！"),
    ("joingame.ip", "地址："),
    ("none.found", "[lightgray]< 未找到 >"),
    ("none", "< 无 >"),
    ("schematic.search", "搜索蓝图"),
    ("schematic.info", "{0}x{1}，{2} 个方块"),
    (
        "schematic.disabled",
        "[scarlet]蓝图已禁用[]\\n你无法在这个[accent]地图[]或[accent]服务器[]上使用蓝图。",
    ),
    ("schematic.import", "导入蓝图"),
    ("schematic.exportfile", "导出文件"),
    ("schematic.importfile", "导入文件"),
    ("schematic.browseworkshop", "浏览创意工坊"),
    ("schematic.copy", "复制到剪贴板"),
    ("schematic.copy.import", "从剪贴板导入"),
    ("schematic.shareworkshop", "分享到创意工坊"),
    ("schematic.edit", "编辑蓝图"),
    ("schematic.tags", "标签："),
    ("schematic.edittags", "编辑标签"),
    ("schematic.addtag", "添加标签"),
    ("schematic.texttag", "文本标签"),
    ("schematic.icontag", "图标标签"),
    ("schematic.renametag", "重命名标签"),
    ("schematic.tagged", "{0} 个蓝图已标记"),
    ("schematic.tagdelconfirm", "确定删除这个标签吗？"),
    ("schematic.tagexists", "该标签已存在。"),
    ("editor.import", "导入"),
    ("editor.export", "导出"),
    ("edit", "编辑"),
    ("database.patched", "已被数据包修改。"),
    ("viewfields", "查看内容字段"),
    ("info.title", "详情"),
    ("category.purpose", "用途"),
    ("category.general", "基础"),
    ("credits.text", "作者 [royal]Anuken[] - [sky]anukendev@gmail.com[]"),
    ("credits", "致谢"),
    ("contributors", "翻译者和贡献者"),
    (
        "link.discord.description",
        "Mindustry 官方的 Discord 聊天室",
    ),
    ("link.reddit.description", "Mindustry 的 Reddit 板块"),
    ("link.github.description", "游戏源代码"),
    ("link.changelog.description", "更新日志"),
    ("link.dev-builds.description", "不稳定的开发版本"),
    ("link.trello.description", "Trello 上的规划表"),
    ("link.itch.io.description", "itch.io 电脑版下载页面"),
    ("link.google-play.description", "Google Play 页面"),
    ("link.f-droid.description", "F-Droid 页面"),
    ("link.wiki.description", "Mindustry 官方 Wiki"),
    ("link.suggestions.description", "提出新功能"),
    ("link.bug.description", "发现了错误？在这里报告"),
    ("host.invalid", "[scarlet]无法连接到服务器。"),
    ("server.waiting", "等待服务器"),
    ("warning", "警告！"),
    ("servers.disclaimer", "社区服务器[accent]不由开发者拥有或管理[]。\n\n这些服务器可能包含并不适合所有年龄段的玩家生成内容。"),
    ("confirm", "确认"),
    ("ok", "确定"),
    ("cancel", "取消"),
    ("back", "返回"),
    ("delete", "删除"),
    ("view.workshop", "在创意工坊中查看"),
    ("customize", "自定义规则"),
    ("unknown", "未知"),
    ("workshop", "创意工坊"),
    ("custom", "自定义"),
    ("builtin", "内置"),
    ("modded", "模组"),
    ("mods", "模组"),
    ("mods.name", "模组："),
    ("mods.none", "[lightgray]未找到模组！"),
    ("mods.guide", "模组指南"),
    ("mods.openfolder", "打开文件夹"),
    ("mods.viewcontent", "查看内容"),
    ("mods.browser", "模组浏览器"),
    ("mods.browser.add", "安装"),
    ("mods.browser.reinstall", "重新安装"),
    ("mods.browser.view-releases", "查看版本"),
    ("mods.browser.latest", "[lightgray][最新]"),
    ("mods.browser.releases", "版本"),
    ("mods.browser.fetching", "获取版本中..."),
    (
        "mods.browser.noreleases",
        "[scarlet]找不到任何版本\\n[accent]无法找到该模组的任何版本。请检查一下该模组的仓库是否发布了版本。",
    ),
    ("mods.browser.sortdate", "按最近排序"),
    ("mods.browser.sortstars", "按星标数排序"),
    ("mods.github.open", "查看"),
    ("mods.github.open-release", "发布页面"),
    ("mods.search", "搜索模组..."),
    ("mod.import", "导入模组"),
    ("mod.import.file", "导入文件"),
    ("mod.import.github", "从 GitHub 导入"),
    ("mod.version", "版本："),
    ("mod.content", "内容："),
    ("editor.name", "名称："),
    ("editor.author", "作者："),
    ("editor.description", "描述："),
    ("editor.openin", "在地图编辑器打开"),
    ("editor.mapinfo", "地图信息"),
    ("editor.mapname", "地图名称："),
    ("editor.newmap", "新地图"),
    ("editor.importmap", "加载地图"),
    ("editor.search", "搜索地图"),
    ("editor.filters", "筛选地图"),
    ("editor.filters.mode", "游戏模式："),
    ("editor.filters.priorities", "优先级："),
    ("editor.filters.type", "地图类型："),
    ("editor.filters.search", "在特定关键词中进行搜索："),
    ("editor.filters.author", "作者"),
    ("editor.filters.description", "描述"),
    ("editor.filters.modname", "模组名称"),
    ("editor.filters.prioritizemod", "模组优先"),
    ("editor.filters.prioritizecustom", "自定义优先"),
    ("editor.filters.planetselect", "选择星球"),
    ("rules.anyenv", "< 任意 >"),
    ("mode.help.title", "游戏模式说明"),
    ("mode.custom", "自定义模式"),
    ("mode.survival.name", "生存"),
    (
        "mode.survival.description",
        "通常的游戏模式，资源有限，自动生成敌人波次。\\n[gray]需要地图中有敌方出生点和己方核心。",
    ),
    ("mode.sandbox.name", "沙盒"),
    ("mode.sandbox.description", "无限资源，不会自动生成敌人波次。"),
    ("mode.pvp.name", "PvP"),
    (
        "mode.pvp.description",
        "与其他玩家对战。\\n[gray]需要地图中至少有两种不同队伍的核心。",
    ),
    ("mode.attack.name", "进攻"),
    (
        "mode.attack.description",
        "摧毁敌人的基地。\\n[gray]需要地图中有敌方队伍的核心。",
    ),
    ("rules.waves", "波次"),
    ("rules.wavesending", "可跳过波次"),
    ("rules.wavetimer", "波次计时器"),
    ("rules.waitForWaveToEnd", "等待波次结束"),
    ("rules.attack", "进攻模式"),
    ("rules.corecapture", "摧毁后占领核心"),
    ("rules.infiniteresources", "无限资源"),
    ("rules.schematic", "允许使用蓝图"),
    ("rules.buildcostmultiplier", "建造花费倍率"),
    ("rules.buildspeedmultiplier", "建造速度倍率"),
    (
        "rules.enemycorebuildradius",
        "敌方核心周围禁建半径：[lightgray]（格）",
    ),
    ("rules.title.waves", "波次"),
    ("rules.title.resourcesbuilding", "资源与建筑"),
    ("rules.title.enemy", "敌对阵营"),
    ("rules.title.planet", "行星"),
    ("settings", "设置"),
    ("settings.game", "游戏"),
    ("settings.graphics", "图形"),
    ("settings.sound", "声音"),
    ("settings.language", "语言"),
    ("settings.controls", "控制"),
    ("settings.data", "数据"),
    ("settings.reset", "全部恢复默认"),
    ("settings.rebind", "重新绑定"),
    ("settings.resetKey", "恢复默认"),
    ("settings.cleardata", "清除游戏数据"),
    ("settings.clearplanetdata", "清除星球数据"),
    ("settings.clearsaves", "清除存档"),
    ("settings.clearresearch", "清除研究进度"),
    ("settings.clearcampaignsaves", "清除战役进度"),
    ("settings.clearplanetresearch", "清除星球研究进度"),
    ("settings.clearplanetcampaignsaves", "清除星球战役进度"),
    ("settings.planetselect", "星球: {0}"),
    ("settings.clearall.confirm", "[scarlet]警告！[]\n这将清除所有数据，包括存档、地图、解锁内容和键位绑定。\n一旦点击“确定”，游戏将清除所有数据并自动退出。"),
    ("settings.clearsaves.confirm", "确认要清除所有存档吗？"),
    ("settings.clearresearch.confirm", "确认要清除战役研究进度吗？"),
    ("settings.clearcampaignsaves.confirm", "确认要清除所有战役进度吗？"),
    (
        "settings.clearplanetresearch.confirm",
        "确认要清除{0}的研究进度吗？",
    ),
    (
        "settings.clearplanetcampaignsaves.confirm",
        "确认要清除{0}的战役进度吗？",
    ),
    ("quit", "退出"),
];

pub const UPSTREAM_MENU_BUNDLE_ZH_TW_ENTRIES: &[(&str, &str)] = &[
    ("play", "開始遊戲"),
    ("level.highscore", "最高分：[accent]{0}"),
    ("level.mode", "遊戲模式："),
    ("campaign", "戰役"),
    ("joingame", "多人連線"),
    ("hostserver", "建立伺服器"),
    ("customgame", "自訂遊戲"),
    ("savegame", "儲存遊戲"),
    ("loadgame", "載入遊戲"),
    ("database.button", "資料庫"),
    ("database", "核心資料庫"),
    ("schematics", "藍圖"),
    ("techtree", "科技樹"),
    ("techtree.select", "選擇科技樹"),
    ("locked", "鎖定"),
    ("completed", "[accent]完成"),
    ("complete", "[lightgray]完成："),
    ("about.button", "關於"),
    ("discord", "加入 Mindustry 的 Discord 聊天室！"),
    ("editor", "地圖編輯器"),
    ("maps", "地圖"),
    ("maps.none", "[lightgray]找不到地圖!"),
    ("name", "名稱："),
    ("search", "搜尋:"),
    ("refresh", "刷新"),
    ("hosts.refresh", "刷新"),
    ("hosts.discovering.any", "搜尋遊戲"),
    ("hosts.none", "[lightgray]找不到區域網路伺服器！"),
    ("servers.local", "區域伺服器"),
    ("servers.local.steam", "開放遊戲間與區域伺服器"),
    ("servers.remote", "遠端伺服器"),
    ("servers.global", "社群伺服器"),
    ("servers.community", "社群伺服器"),
    ("servers.showhidden", "顯示被隱藏的伺服器"),
    ("server.saved", "已儲存伺服器"),
    ("server.add", "新增伺服器"),
    ("server.edit", "編輯伺服器"),
    ("server.delete", "您確定要刪除這個伺服器嗎？"),
    ("server.version", "[gray]v{0} {1}"),
    ("server.versions", "你的版本：[accent] {0}[]\\n伺服器版本：[accent] {1}[]"),
    ("server.kicked.clientOutdated", "客戶端版本過舊！請更新遊戲！"),
    ("server.kicked.serverOutdated", "伺服器版本過舊！請讓房主更新！"),
    ("joingame.ip", "IP 位置："),
    ("none.found", "[lightgray]〈查無結果〉"),
    ("none", "〈沒有〉"),
    ("schematic.search", "搜尋藍圖..."),
    ("schematic.info", "{0}x{1}, {2}方塊"),
    (
        "schematic.disabled",
        "[scarlet]藍圖被進用[]\\n您不能在這個[accent]地圖[]或[accent]伺服器中使用藍圖.",
    ),
    ("schematic.import", "匯入藍圖……"),
    ("schematic.exportfile", "匯出檔案"),
    ("schematic.importfile", "匯入檔案"),
    ("schematic.browseworkshop", "瀏覽工作坊"),
    ("schematic.copy", "複製到剪貼簿"),
    ("schematic.copy.import", "從剪貼簿匯入"),
    ("schematic.shareworkshop", "分享到工作坊"),
    ("schematic.edit", "編輯藍圖"),
    ("schematic.tags", "標籤："),
    ("schematic.edittags", "編輯標籤"),
    ("schematic.addtag", "新增標籤"),
    ("schematic.texttag", "文字標籤"),
    ("schematic.icontag", "圖像標籤"),
    ("schematic.renametag", "重新命名"),
    ("schematic.tagged", "{0} 已被加上標籤"),
    ("schematic.tagdelconfirm", "確認刪除此標籤？"),
    ("schematic.tagexists", "該標籤已存在。"),
    ("editor.import", "匯入……"),
    ("editor.export", "匯出……"),
    ("edit", "編輯……"),
    ("database.patched", "Modified by data patches."),
    ("viewfields", "View Content Fields"),
    ("info.title", "資訊"),
    ("category.purpose", "用途"),
    ("category.general", "一般"),
    (
        "credits.text",
        "由[royal]Anuken[]製作 - [sky]anukendev@gmail.com[]",
    ),
    ("credits", "感謝名單"),
    ("contributors", "翻譯員和貢獻者"),
    (
        "link.discord.description",
        "官方 Mindustry Discord 聊天室",
    ),
    ("link.reddit.description", "Mindustry Reddit論壇"),
    ("link.github.description", "遊戲原始碼"),
    ("link.changelog.description", "遊戲更新日誌"),
    ("link.dev-builds.description", "開發中版本"),
    ("link.trello.description", "官方 Trello 功能規劃看板"),
    ("link.itch.io.description", "itch.io 電腦版下載網頁"),
    ("link.google-play.description", "Google Play 商店頁面"),
    ("link.f-droid.description", "F-Droid 目錄頁面"),
    ("link.wiki.description", "官方 Mindustry 維基"),
    ("link.suggestions.description", "建議新功能"),
    ("link.bug.description", "發現錯誤？在這裡回報"),
    ("host.invalid", "[scarlet]無法連線至伺服器。"),
    ("server.waiting", "等待伺服器"),
    ("warning", "警告。"),
    ("servers.disclaimer", "社群伺服器[accent]不是[]由開發者擁有或管理。\n\n伺服器可能會出現由其他玩家製作的不適當內容。"),
    ("confirm", "確認"),
    ("ok", "確定"),
    ("cancel", "取消"),
    ("back", "返回"),
    ("delete", "刪除"),
    ("view.workshop", "在工作坊中查看"),
    ("customize", "自訂"),
    ("unknown", "未知"),
    ("workshop", "工作坊"),
    ("custom", "自訂"),
    ("builtin", "内建"),
    ("modded", "Modded"),
    ("mods", "模組"),
    ("mods.name", "Mod:"),
    ("mods.none", "[lightgray]找不到模組!"),
    ("mods.guide", "模組指南"),
    ("mods.openfolder", "開啟模組資料夾"),
    ("mods.viewcontent", "查看內容"),
    ("mods.browser", "模組瀏覽器"),
    ("mods.browser.add", "安裝"),
    ("mods.browser.reinstall", "重新安裝"),
    ("mods.browser.view-releases", "檢視發行"),
    ("mods.browser.latest", "<最新>"),
    ("mods.browser.releases", "所有發行"),
    ("mods.browser.fetching", "Fetching Releases..."),
    (
        "mods.browser.noreleases",
        "[scarlet]無發行紀錄\\n[accent]無法找到該模組的任何發行。請確認此模組是否有發表。",
    ),
    ("mods.browser.sortdate", "以最近篩選"),
    ("mods.browser.sortstars", "以星數篩選"),
    ("mods.github.open", "查看Github"),
    ("mods.github.open-release", "發行網頁"),
    ("mods.search", "搜尋模組..."),
    ("mod.import", "匯入模組"),
    ("mod.import.file", "匯入檔案"),
    ("mod.import.github", "匯入GitHub模組"),
    ("mod.version", "Version:"),
    ("mod.content", "內容："),
    ("editor.name", "名稱："),
    ("editor.author", "作者："),
    ("editor.description", "描述："),
    ("editor.openin", "在編輯器中開啟"),
    ("editor.mapinfo", "地圖資訊"),
    ("editor.mapname", "地圖名稱："),
    ("editor.newmap", "新地圖"),
    ("editor.importmap", "匯入地圖"),
    ("editor.search", "尋找地圖…"),
    ("editor.filters", "篩選地圖"),
    ("editor.filters.mode", "遊戲模式："),
    ("editor.filters.priorities", "Priorities:"),
    ("editor.filters.type", "地圖種類："),
    ("editor.filters.search", "搜尋的資料夾："),
    ("editor.filters.author", "作者"),
    ("editor.filters.description", "描述"),
    ("editor.filters.modname", "Mod Name"),
    ("editor.filters.prioritizemod", "Mod Priority"),
    ("editor.filters.prioritizecustom", "Custom Priority"),
    ("editor.filters.planetselect", "Planet Selection"),
    ("rules.anyenv", "<任意>"),
    ("mode.help.title", "模式說明"),
    ("mode.custom", "自訂規則"),
    ("mode.survival.name", "生存"),
    (
        "mode.survival.description",
        "一般模式。有限的資源與自動來襲的波次。\\n[gray]地圖中需要敵人生成點。",
    ),
    ("mode.sandbox.name", "沙盒"),
    ("mode.sandbox.description", "無限的資源與不倒數計時的波次。"),
    ("mode.pvp.name", "對戰"),
    (
        "mode.pvp.description",
        "和其他玩家競爭、戰鬥。\\n[gray]地圖中需要至少兩個不同顏色的核心。",
    ),
    ("mode.attack.name", "進攻"),
    (
        "mode.attack.description",
        "目標是摧毀敵人的基地。\\n[gray]地圖中需要有一個紅色核心。",
    ),
    ("rules.waves", "波次"),
    ("rules.wavesending", "Wave Sending"),
    ("rules.wavetimer", "波次時間"),
    ("rules.waitForWaveToEnd", "等待所有敵人毀滅才開始下一波次"),
    ("rules.attack", "攻擊模式"),
    ("rules.corecapture", "佔領摧毀的核心"),
    ("rules.infiniteresources", "無限資源"),
    ("rules.schematic", "允許使用藍圖"),
    ("rules.buildcostmultiplier", "建設成本倍數"),
    ("rules.buildspeedmultiplier", "建設速度倍數"),
    (
        "rules.enemycorebuildradius",
        "敵人核心禁止建設半徑︰[lightgray]（格）",
    ),
    ("rules.title.waves", "波次"),
    ("rules.title.resourcesbuilding", "資源與建築"),
    ("rules.title.enemy", "敵人"),
    ("rules.title.planet", "星球"),
    ("settings", "設定"),
    ("settings.game", "遊戲"),
    ("settings.graphics", "圖形"),
    ("settings.sound", "音效"),
    ("settings.language", "語言"),
    ("settings.controls", "控制"),
    ("settings.data", "遊戲資料"),
    ("settings.reset", "重設為預設設定"),
    ("settings.rebind", "重新綁定"),
    ("settings.resetKey", "重設按鍵"),
    ("settings.cleardata", "清除遊戲資料……"),
    ("settings.clearplanetdata", "清除行星資料"),
    ("settings.clearsaves", "清除存檔"),
    ("settings.clearresearch", "清除研究"),
    ("settings.clearcampaignsaves", "清除戰役紀錄"),
    ("settings.clearplanetresearch", "清除行星研究"),
    (
        "settings.clearplanetcampaignsaves",
        "清除行星戰役紀錄",
    ),
    ("settings.planetselect", "行星: {0}"),
    ("settings.clearall.confirm", "[scarlet]警告！[]\n這會清除所有資料，包括存檔、地圖、解鎖項目和快捷鍵綁定。\n按「確定」後，遊戲將刪除所有資料並自動結束。"),
    (
        "settings.clearsaves.confirm",
        "您確定您想要清除所有存檔嗎？",
    ),
    ("settings.clearresearch.confirm", "您確定要清除所有研究？"),
    (
        "settings.clearcampaignsaves.confirm",
        "您確定要清除所有戰役紀錄？",
    ),
    (
        "settings.clearplanetresearch.confirm",
        "您確定要清除{0}的研究？",
    ),
    (
        "settings.clearplanetcampaignsaves.confirm",
        "您確定要清除{0}的戰役紀錄？",
    ),
    ("quit", "退出"),
];

pub fn upstream_bundle_en_value(key: &str) -> Option<&'static str> {
    UPSTREAM_MENU_BUNDLE_ENTRIES
        .iter()
        .find_map(|(candidate, value)| (*candidate == key).then_some(*value))
}

fn upstream_bundle_value_from_entries(
    entries: &'static [(&'static str, &'static str)],
    key: &str,
) -> Option<&'static str> {
    entries
        .iter()
        .find_map(|(candidate, value)| (*candidate == key).then_some(*value))
}

pub fn upstream_menu_bundle_entries_for_locale(
    locale: &str,
) -> &'static [(&'static str, &'static str)] {
    let locale = locale.trim().replace('-', "_");
    if locale.eq_ignore_ascii_case("zh_TW") || locale.eq_ignore_ascii_case("zh_HK") {
        UPSTREAM_MENU_BUNDLE_ZH_TW_ENTRIES
    } else if locale.eq_ignore_ascii_case("zh_CN")
        || locale.eq_ignore_ascii_case("zh")
        || locale.eq_ignore_ascii_case("zh_Hans")
    {
        UPSTREAM_MENU_BUNDLE_ZH_CN_ENTRIES
    } else {
        UPSTREAM_MENU_BUNDLE_ENTRIES
    }
}

pub fn upstream_menu_bundle_value_for_locale(locale: &str, key: &str) -> Option<&'static str> {
    upstream_bundle_value_from_entries(upstream_menu_bundle_entries_for_locale(locale), key)
        .or_else(|| upstream_bundle_en_value(key))
}

pub fn upstream_menu_bundle_format_for_locale(
    locale: &str,
    key: &str,
    args: &[&str],
) -> Option<String> {
    let mut value = upstream_menu_bundle_value_for_locale(locale, key)?.to_string();
    for (index, arg) in args.iter().enumerate() {
        value = value.replace(&format!("{{{index}}}"), arg);
    }
    Some(value)
}

pub use bar::{
    Bar, BarBackgroundDraw, BarDrawCommand, BarDrawPlan, BarFillDraw, BarFrameState, BarLayout,
    BarOutlineDraw, BarTextDraw,
};
pub use dialogs::BaseDialog;
pub use displayable::{DisplayTable, Displayable};
pub use fonts::{
    parse_upstream_icon_properties, upstream_font_asset, upstream_font_asset_by_name,
    upstream_font_assets, upstream_font_source_paths, upstream_ui_icon_glyph,
    upstream_ui_icon_glyph_char, upstream_ui_icon_glyph_string, IconPropertiesParseError,
    UpstreamContentIcon, UpstreamFontAsset, UpstreamFontRole, UpstreamUiIconGlyph,
    UPSTREAM_FONT_ASSETS, UPSTREAM_ICONS_PROPERTIES_SOURCE_PATH, UPSTREAM_ICON_FONT_SOURCE_PATH,
    UPSTREAM_JAPANESE_FONT_SOURCE_PATH, UPSTREAM_LOGIC_FONT_CHARACTERS,
    UPSTREAM_LOGIC_FONT_SOURCE_PATH, UPSTREAM_MAIN_FONT_SOURCE_PATH,
    UPSTREAM_MONOSPACE_FONT_SOURCE_PATH, UPSTREAM_TECH_FONT_SOURCE_PATH,
    UPSTREAM_UI_ICON_FONTGEN_CONFIG_SOURCE_PATH, UPSTREAM_UI_ICON_GLYPHS,
};
pub use mobile_button::{MobileButton, MobileButtonLayout};
pub use styles::{
    upstream_check_box_style_skin, upstream_image_button_style_skin,
    upstream_scroll_pane_style_skin, upstream_slider_style_skin, upstream_text_button_style_skin,
    upstream_text_field_style_skin, upstream_ui_drawable_alias, upstream_ui_skin_sprite,
    upstream_ui_skin_sprite_source_paths, upstream_ui_skin_sprites, UiCheckBoxStyleSkin,
    UiDrawableAlias, UiDrawableTint, UiImageButtonStyleSkin, UiScrollPaneStyleSkin, UiSkinGroup,
    UiSkinSprite, UiSliderStyleSkin, UiTextButtonStyleSkin, UiTextFieldStyleSkin,
    UPSTREAM_CHECK_BOX_STYLE_SKINS, UPSTREAM_IMAGE_BUTTON_STYLE_SKINS,
    UPSTREAM_SCROLL_PANE_STYLE_SKINS, UPSTREAM_SLIDER_STYLE_SKINS,
    UPSTREAM_TEXT_BUTTON_STYLE_SKINS, UPSTREAM_TEXT_FIELD_STYLE_SKINS,
    UPSTREAM_UI_DRAWABLE_ALIASES, UPSTREAM_UI_SKIN_SPRITES,
};
pub use warning_bar::{
    LineSegment, Quad, WarningBar, WarningBarDrawCommand, WarningBarDrawPlan, WarningBarLayout,
    WarningBarLineDraw, WarningBarStripeDraw,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upstream_menu_bundle_entries_cover_menu_fragment_buttons() {
        assert_eq!(
            UPSTREAM_BUNDLE_PROPERTIES_SOURCE_PATH,
            "core/assets/bundles/bundle.properties"
        );
        for (key, expected) in [
            ("play", "Play"),
            ("campaign", "Campaign"),
            ("joingame", "Join Game"),
            ("hostserver", "Host Multiplayer Game"),
            ("customgame", "Custom Game"),
            ("savegame", "Save Game"),
            ("loadgame", "Load Game"),
            ("database.button", "Database"),
            ("database", "Core Database"),
            ("discord", "Join the Mindustry Discord!"),
            ("about.button", "About"),
            ("maps", "Maps"),
            ("server.add", "Add Server"),
            (
                "server.kicked.clientOutdated",
                "Outdated client! Update your game!",
            ),
            (
                "server.kicked.serverOutdated",
                "Outdated server! Ask the host to update!",
            ),
            ("ok", "OK"),
            ("back", "Back"),
            ("settings", "Settings"),
            ("settings.game", "Game"),
            ("settings.language", "Language"),
            ("quit", "Quit"),
        ] {
            assert_eq!(upstream_bundle_en_value(key), Some(expected));
        }
        assert_eq!(upstream_bundle_en_value("missing.menu.key"), None);
    }

    #[test]
    fn upstream_menu_bundle_locale_entries_cover_chinese_menu_fragment_buttons() {
        assert_eq!(
            upstream_menu_bundle_value_for_locale("zh_CN", "play"),
            Some("开始游戏")
        );
        assert_eq!(
            upstream_menu_bundle_value_for_locale("zh_CN", "database"),
            Some("核心数据库")
        );
        assert_eq!(
            upstream_menu_bundle_value_for_locale("zh_CN", "hostserver"),
            Some("创建联机游戏")
        );
        assert_eq!(
            upstream_menu_bundle_value_for_locale("zh_CN", "savegame"),
            Some("保存游戏")
        );
        assert_eq!(
            upstream_menu_bundle_value_for_locale("zh_CN", "maps"),
            Some("地图")
        );
        assert_eq!(
            upstream_menu_bundle_value_for_locale("zh_CN", "server.add"),
            Some("添加服务器")
        );
        assert_eq!(
            upstream_menu_bundle_value_for_locale("zh_CN", "servers.local"),
            Some("本地服务器")
        );
        assert_eq!(
            upstream_menu_bundle_value_for_locale("zh_CN", "settings.language"),
            Some("语言")
        );
        assert_eq!(
            upstream_menu_bundle_value_for_locale("zh_CN", "settings.clearresearch"),
            Some("清除研究进度")
        );
        assert_eq!(
            upstream_menu_bundle_format_for_locale("zh_CN", "settings.planetselect", &["Serpulo"])
                .as_deref(),
            Some("星球: Serpulo")
        );
        assert_eq!(
            upstream_menu_bundle_format_for_locale(
                "en",
                "settings.clearplanetresearch.confirm",
                &["Erekir"]
            )
            .as_deref(),
            Some("Are you sure you want to clear Erekir's research?")
        );
        assert_eq!(
            upstream_menu_bundle_format_for_locale("en", "server.versions", &["158", "157"])
                .as_deref(),
            Some("Your version:[accent] 158[]\\nServer version:[accent] 157[]")
        );
        assert_eq!(
            upstream_menu_bundle_value_for_locale("zh_CN", "server.kicked.serverOutdated"),
            Some("服务器版本过旧！请让房主更新！")
        );
        assert_eq!(
            upstream_menu_bundle_value_for_locale("zh_CN", "schematic.import"),
            Some("导入蓝图")
        );
        assert_eq!(
            upstream_menu_bundle_value_for_locale("zh_CN", "info.title"),
            Some("详情")
        );
        assert_eq!(
            upstream_menu_bundle_value_for_locale("zh_CN", "credits"),
            Some("致谢")
        );
        assert_eq!(
            upstream_menu_bundle_value_for_locale("zh_CN", "link.github.description"),
            Some("游戏源代码")
        );
        assert_eq!(
            upstream_menu_bundle_value_for_locale("zh-TW", "joingame"),
            Some("多人連線")
        );
        assert_eq!(
            upstream_menu_bundle_value_for_locale("zh_TW", "discord"),
            Some("加入 Mindustry 的 Discord 聊天室！")
        );
        assert_eq!(
            upstream_menu_bundle_value_for_locale("zh_TW", "settings"),
            Some("設定")
        );
        assert_eq!(
            upstream_menu_bundle_value_for_locale("unknown", "database"),
            Some("Core Database")
        );
    }
}
