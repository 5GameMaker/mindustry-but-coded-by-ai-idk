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
    ("connecting", "[accent]Connecting..."),
    ("reconnecting", "[accent]Reconnecting..."),
    ("connecting.data", "[accent]Loading world data..."),
    ("disconnect", "Disconnected."),
    ("disconnect.error", "Connection error."),
    ("disconnect.closed", "Connection closed."),
    ("disconnect.timeout", "Timed out."),
    ("disconnect.data", "Failed to load world data!"),
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
    ("players", "{0} players"),
    ("players.single", "{0} player"),
    ("save.map", "Map: {0}"),
    ("server.version", "[gray]v{0} {1}"),
    ("server.versions", "Your version:[accent] {0}[]\\nServer version:[accent] {1}[]"),
    ("server.refreshing", "Refreshing server"),
    ("server.kicked.kick", "You have been kicked from the server!"),
    ("server.kicked.whitelist", "You are not whitelisted here."),
    ("server.kicked.serverClose", "Server closed."),
    ("server.kicked.vote", "You have been vote-kicked. Goodbye."),
    ("server.kicked.clientOutdated", "Outdated client! Update your game!"),
    ("server.kicked.serverOutdated", "Outdated server! Ask the host to update!"),
    ("server.kicked.banned", "You are banned on this server."),
    ("server.kicked.typeMismatch", "This server is not compatible with your build type."),
    ("server.kicked.playerLimit", "This server is full. Wait for an empty slot."),
    ("server.kicked.recentKick", "You have been kicked recently.\\nWait before connecting again."),
    ("server.kicked.nameInUse", "There is someone with that name\\nalready on this server."),
    ("server.kicked.nameEmpty", "Your chosen name is invalid."),
    ("server.kicked.idInUse", "You are already on this server! Connecting with two accounts is not permitted."),
    ("server.kicked.customClient", "This server does not support custom builds. Download an official version."),
    ("server.kicked.gameover", "Game over!"),
    ("server.kicked.serverRestarting", "The server is restarting."),
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
    ("editor.exists", "A map with that name already exists."),
    ("editor.errorimage", "That's an image, not a map."),
    ("editor.errornot", "This is not a map file."),
    (
        "editor.errorname",
        "Map has no name defined. Are you trying to load a save file?",
    ),
    ("editor.importmap", "Import Map"),
    (
        "editor.import.exists",
        "[scarlet]Unable to import:[] a built-in map with that name already exists!",
    ),
    (
        "editor.overwrite.confirm",
        "[scarlet]Warning![] A map with this name already exists. Are you sure you want to overwrite it?",
    ),
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
    ("waves.edit", "Edit..."),
    ("waves.copy", "Copy to Clipboard"),
    ("waves.load", "Load from Clipboard"),
    ("rules.invaliddata", "Invalid clipboard data."),
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
    ("rules.wavelimit", "Map Ends After Wave"),
    ("rules.wavespacing", "Wave Spacing:[lightgray] (sec)"),
    (
        "rules.initialwavespacing",
        "Initial Wave Spacing:[lightgray] (sec)",
    ),
    ("rules.dropzoneradius", "Drop Zone Radius:[lightgray] (tiles)"),
    ("rules.waitForWaveToEnd", "Waves Wait for Enemies"),
    ("rules.attack", "Attack Mode"),
    ("rules.corecapture", "Capture Core On Destruction"),
    ("rules.infiniteresources", "Infinite Resources"),
    ("rules.schematic", "Schematics Allowed"),
    ("rules.allowedit", "Allow Editing Rules"),
    (
        "rules.allowedit.info",
        "When enabled, the player can edit rules in-game via the button in the bottom left corner of the Pause menu.",
    ),
    (
        "rules.alloweditworldprocessors",
        "Allow Editing World Processors",
    ),
    (
        "rules.alloweditworldprocessors.info",
        "When enabled, world logic blocks can be placed and edited even outside the editor.",
    ),
    ("rules.hidebannedblocks", "Hide Banned Blocks"),
    ("bannedblocks", "Banned Blocks"),
    ("unbannedblocks", "Unbanned Blocks"),
    ("bannedunits", "Banned Units"),
    ("unbannedunits", "Unbanned Units"),
    ("bannedblocks.whitelist", "Banned Blocks As Whitelist"),
    ("bannedunits.whitelist", "Banned Units As Whitelist"),
    ("addall", "Add All"),
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
    ("rules.title.teams", "Teams"),
    ("rules.enemyteam", "Enemy Team"),
    ("rules.playerteam", "Player Team"),
    ("rules.weather", "Weather"),
    ("rules.weather.frequency", "Frequency:"),
    ("rules.weather.always", "Always"),
    ("rules.weather.duration", "Duration:"),
    (
        "rules.protectcores.info",
        "When disabled, the core no-build radius won't affect this team.\\nPlayers won't be assigned to unprotected teams.",
    ),
    (
        "rules.checkplacement.info",
        "When disabled, buildings of this team are ignored in placement range checks.",
    ),
    (
        "rules.randomwaveai.info",
        "Makes units spawned in waves target random structures instead of directly attacking the core or power generators.",
    ),
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
    ("abandon", "Abandon"),
    ("add", "Add..."),
    ("all", "All"),
    ("be.check", "Check for updates"),
    ("be.noupdates", "No updates found."),
    ("category.blocks.name", "Block Select"),
    ("category.command.name", "Unit Command"),
    ("category.general.name", "General"),
    ("category.multiplayer.name", "Multiplayer"),
    ("category.view.name", "View"),
    ("copied", "Copied."),
    ("crash.export", "Export Crash Logs"),
    ("data.export", "Export Data"),
    ("data.import", "Import Data"),
    ("data.openfolder", "Open Data Folder"),
    ("editor.worldprocessors", "World Processors"),
    ("globalitems", "[accent]Planet Items"),
    ("host", "Host"),
    (
        "host.info",
        "The [accent]host[] button hosts a server on the specified port.\\nAnybody on the same [lightgray]wifi or local network[] should be able to see your server in their server list.\\n\\nIf you want people to be able to connect from anywhere by IP, [accent]port forwarding[] is required.\\n\\n[lightgray]Note: If someone is experiencing trouble connecting to your LAN game, make sure you have allowed Mindustry access to your local network in your firewall settings. Note that public networks sometimes do not allow server discovery.",
    ),
    ("hostserver.mobile", "Host Game"),
    (
        "join.info",
        "Here, you can enter a [accent]server IP[] to connect to, or discover [accent]local network[] or [accent]global[] servers to connect to.\\nBoth LAN and WAN multiplayer is supported.\\n\\n[lightgray]If you want to connect to someone by IP, you would need to ask the host for their IP, which can be found by googling \"my ip\" from their device.",
    ),
    ("joingame.title", "Join Game"),
    ("keybind.chat.name", "Chat"),
    ("keybind.move_x.name", "Move X"),
    (
        "language.restart",
        "Restart your game for the language settings to take effect.",
    ),
    (
        "link.discord.description",
        "The official Mindustry Discord chatroom",
    ),
    (
        "linkfail",
        "Failed to open link!\\nThe URL has been copied to your clipboard.",
    ),
    ("load", "Load"),
    ("loading", "[accent]Loading..."),
    (
        "mod.noerrorplay",
        "[red]You have mods with errors.[] Either disable the affected mods or fix the errors before playing.",
    ),
    ("noname", "Pick a[accent] player name[] first."),
    ("objective", "Map Objective"),
    ("off", "Off"),
    ("on", "On"),
    ("overwrite", "Overwrite"),
    ("players.search", "search"),
    ("quit.confirm", "Are you sure you want to quit?"),
    ("save", "Save"),
    ("save.autosave", "Autosave: {0}"),
    ("save.date", "Last Saved: {0}"),
    ("save.delete", "Delete"),
    ("save.delete.confirm", "Are you sure you want to delete this save?"),
    ("save.import", "Import Save"),
    ("save.import.invalid", "[accent]This save is invalid!"),
    ("save.new", "New Save"),
    ("save.newslot", "Save name:"),
    (
        "save.nocampaign",
        "Individual save files from the campaign cannot be imported.",
    ),
    ("save.none", "No saves found!"),
    ("save.overwrite", "Are you sure you want to overwrite\\nthis save slot?"),
    ("save.playtime", "Playtime: {0}"),
    ("save.rename", "Rename"),
    ("save.rename.text", "New name:"),
    ("save.search", "Search saved games..."),
    ("save.wave", "Wave {0}"),
    ("savefail", "Failed to save game!"),
    ("saving", "[accent]Saving..."),
    ("server.invalidport", "Invalid port number!"),
    ("server.port", "Port:"),
    ("server.shown", "Shown"),
    ("server.hidden", "Hidden"),
    ("server.favorite", "Favorite"),
    (
        "setting.communityservers.name",
        "Fetch Community Server List",
    ),
    ("setting.fullscreen.name", "Fullscreen"),
    ("setting.fpscap.name", "Max FPS"),
    ("setting.fpscap.none", "None"),
    ("setting.fpscap.text", "{0} FPS"),
    ("setting.macnotch.name", "Adapt interface to display notch"),
    ("setting.musicvol.name", "Music Volume"),
    ("setting.playerlimit.name", "Player Limit"),
    ("setting.saveinterval.name", "Save Interval"),
    ("setting.seconds", "{0} seconds"),
    ("stat.armor", "Armor"),
    ("stat.damage", "Damage"),
    ("stat.explosiveness", "Explosiveness"),
    ("stat.flammability", "Flammability"),
    ("stat.health", "Health"),
    ("stat.healthmultiplier", "Health Multiplier"),
    ("stat.heatcapacity", "Heat Capacity"),
    ("stat.radioactivity", "Radioactivity"),
    ("stat.range", "Range"),
    ("stat.reloadmultiplier", "Reload Multiplier"),
    ("stat.size", "Size"),
    ("stat.speed", "Speed"),
    ("stat.speedmultiplier", "Speed Multiplier"),
    ("stat.temperature", "Temperature"),
    ("stat.viscosity", "Viscosity"),
    ("stat.atmosphere", "Atmosphere"),
    ("stat.capturewave", "Capture Wave"),
    ("stat.difficulty", "Difficulty"),
    ("stat.duration", "Duration"),
    ("stat.hardness", "Hardness"),
    ("stat.id", "ID"),
    ("stat.landable", "Landable"),
    ("stat.opacity", "Opacity"),
    ("stat.placeable", "Placeable On"),
    ("stat.planet", "Planet"),
    ("stat.sectors", "Sectors"),
    ("stat.status", "Status"),
    ("server.invalidaddress", "Invalid address!"),
    (
        "abandon.confirm",
        "This sector's core(s) will self-destruct.\\nContinue?",
    ),
    ("steam.friendsonly", "Friends Only"),
    (
        "steam.friendsonly.tooltip",
        "Whether only Steam friends will be able to join your game.\\nUnchecking this box will make your game public - anyone can join.",
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
    ("connecting", "[accent]连接中…"),
    ("reconnecting", "[accent]重新连接…"),
    ("connecting.data", "[accent]加载世界数据…"),
    ("disconnect", "已断开连接。"),
    ("disconnect.error", "连接出错。"),
    ("disconnect.closed", "连接已关闭。"),
    ("disconnect.timeout", "连接超时。"),
    ("disconnect.data", "加载世界数据失败！"),
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
    ("players", "{0} 名玩家"),
    ("players.single", "{0} 名玩家"),
    ("save.map", "地图：{0}"),
    ("server.version", "[gray]v{0} {1}"),
    ("server.versions", "你的版本：[accent] {0}[]\\n服务器版本：[accent] {1}[]"),
    ("server.refreshing", "刷新服务器中…"),
    ("server.kicked.kick", "你已被踢出服务器。"),
    ("server.kicked.whitelist", "你不在服务器白名单中。"),
    ("server.kicked.serverClose", "服务器已关闭。"),
    ("server.kicked.vote", "你已被投票踢出。"),
    ("server.kicked.clientOutdated", "客户端版本过旧！请更新游戏！"),
    ("server.kicked.serverOutdated", "服务器版本过旧！请让房主更新！"),
    ("server.kicked.banned", "你已被服务器封禁。"),
    ("server.kicked.typeMismatch", "此服务器与你的游戏版本不兼容。"),
    ("server.kicked.playerLimit", "服务器已满。请稍后再试。"),
    ("server.kicked.recentKick", "你刚刚被踢出。\\n请稍后再试。"),
    ("server.kicked.nameInUse", "该名称已被占用。"),
    ("server.kicked.nameEmpty", "你的名称无效。"),
    ("server.kicked.idInUse", "你已在服务器中。无法重复连接。"),
    ("server.kicked.customClient", "此服务器禁止自定义客户端。请使用官方版本。"),
    ("server.kicked.gameover", "游戏已结束！"),
    ("server.kicked.serverRestarting", "重启服务器中…"),
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
    ("editor.exists", "已存在同名地图。"),
    ("editor.errorimage", "这是一个图像，不是地图。"),
    ("editor.errornot", "这不是一个地图文件。"),
    ("editor.errorname", "地图没有定义名称。你是想加载一个存档吗？"),
    ("editor.importmap", "加载地图"),
    ("editor.import.exists", "[scarlet]无法导入！[] 已存在同名内置地图！"),
    (
        "editor.overwrite.confirm",
        "[scarlet]警告！[] 已存在同名地图。确定要覆盖吗？",
    ),
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
    ("waves.edit", "编辑"),
    ("waves.copy", "复制到剪贴板"),
    ("waves.load", "从剪贴板读取"),
    ("rules.invaliddata", "剪贴板数据无效。"),
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
    ("rules.wavelimit", "波数到达后结束地图"),
    ("rules.wavespacing", "波次间隔：[lightgray]（秒）"),
    ("rules.initialwavespacing", "初始波次间隔：[lightgray]（秒）"),
    ("rules.dropzoneradius", "空降区半径：[lightgray]（格）"),
    ("rules.waitForWaveToEnd", "等待波次结束"),
    ("rules.attack", "进攻模式"),
    ("rules.corecapture", "摧毁后占领核心"),
    ("rules.infiniteresources", "无限资源"),
    ("rules.schematic", "允许使用蓝图"),
    ("rules.allowedit", "允许规则编辑"),
    (
        "rules.allowedit.info",
        "启用后，玩家可以通过暂停菜单左下角的按钮编辑游戏中的规则。",
    ),
    ("rules.alloweditworldprocessors", "允许编辑世界处理器"),
    (
        "rules.alloweditworldprocessors.info",
        "启用后，即使不在编辑器中，也可以放置和编辑世界处理器。",
    ),
    ("rules.hidebannedblocks", "隐藏禁用的建筑"),
    ("bannedblocks", "禁用建筑"),
    ("unbannedblocks", "可用建筑"),
    ("bannedunits", "禁用单位"),
    ("unbannedunits", "可用单位"),
    ("bannedblocks.whitelist", "将禁用建筑列入白名单"),
    ("bannedunits.whitelist", "将禁用单位列入白名单"),
    ("addall", "全部添加"),
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
    ("rules.title.teams", "队伍"),
    ("rules.enemyteam", "敌方队伍"),
    ("rules.playerteam", "玩家队伍"),
    ("rules.weather", "天气"),
    ("rules.weather.frequency", "周期："),
    ("rules.weather.always", "永久"),
    ("rules.weather.duration", "时长："),
    (
        "rules.protectcores.info",
        "禁用后，核心禁建区将不会影响该队伍。\\n玩家不会被分配到不受保护的队伍。",
    ),
    (
        "rules.checkplacement.info",
        "禁用后，该队伍的建筑物在放置范围检查中将被忽略。",
    ),
    (
        "rules.randomwaveai.info",
        "使波次中生成的单位随机攻击建筑，而非直接攻击核心或发电建筑。",
    ),
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
    ("abandon", "放弃"),
    ("add", "添加"),
    ("all", "所有"),
    ("be.check", "检测更新"),
    ("be.noupdates", "未发现更新。"),
    ("category.blocks.name", "建筑选择"),
    ("category.command.name", "单位指挥"),
    ("category.general.name", "常规"),
    ("category.multiplayer.name", "多人游戏"),
    ("category.view.name", "视图"),
    ("copied", "已复制。"),
    ("crash.export", "导出崩溃日志"),
    ("data.export", "导出数据"),
    ("data.import", "导入数据"),
    ("data.openfolder", "打开数据文件夹"),
    ("editor.worldprocessors", "世界处理器"),
    ("globalitems", "[accent]行星物品"),
    ("host", "创建"),
    (
        "host.info",
        "点击[accent]主机[]按钮可在指定端口创建服务器。\\n同一[lightgray] Wi-Fi 或局域网[]内的其他玩家将能看到你的服务器。\\n\\n如果你希望通过 IP 让他人连接，[accent]需进行端口转发[]。\\n\\n[lightgray]注意：若他人无法加入你的局域网游戏，请确认已在防火墙中允许 Mindustry 访问局域网。\\n部分公共网络可能不支持服务器发现。",
    ),
    ("hostserver.mobile", "创建联机游戏"),
    (
        "join.info",
        "你可以在此输入[accent]服务器 IP[]进行连接，\\n或搜索[accent]局域网[]或[accent]全球[]服务器加入。\\n支持局域网和公网联机。\\n\\n[lightgray]若要通过 IP 加入他人服务器，你需要向房主询问其 IP，\\n可在其设备上搜索“我的 IP”获取。",
    ),
    ("joingame.title", "加入游戏"),
    ("keybind.chat.name", "聊天"),
    ("keybind.move_x.name", "水平移动"),
    ("language.restart", "重启游戏后语言设置才会生效。"),
    (
        "link.discord.description",
        "Mindustry 官方的 Discord 聊天室",
    ),
    (
        "linkfail",
        "无法打开链接！\\n链接已复制到你的剪贴板。",
    ),
    ("load", "读取"),
    ("loading", "[accent]加载中…"),
    (
        "mod.noerrorplay",
        "[red]你的模组发生了错误。[] 请禁用受影响的模组或修复错误后再进行游戏。",
    ),
    ("noname", "请先设置[accent]玩家名称[]。"),
    ("objective", "任务目标"),
    ("off", "关闭"),
    ("on", "开启"),
    ("overwrite", "覆盖"),
    ("players.search", "搜索"),
    ("quit.confirm", "确定要退出吗？"),
    ("save", "保存"),
    ("save.autosave", "自动保存：{0}"),
    ("save.date", "最后保存时间：{0}"),
    ("save.delete", "删除"),
    ("save.delete.confirm", "确定要删除这个存档吗？"),
    ("save.import", "导入存档"),
    ("save.import.invalid", "[accent]此存档无效！"),
    ("save.new", "新的存档"),
    ("save.newslot", "存档名称："),
    ("save.nocampaign", "无法导入战役模式的独立存档文件。"),
    ("save.none", "没有找到任何存档！"),
    ("save.overwrite", "确定要覆盖\\n这个存档吗？"),
    ("save.playtime", "游戏时长：{0}"),
    ("save.rename", "重命名"),
    ("save.rename.text", "新名称："),
    ("save.search", "搜索存档游戏"),
    ("save.wave", "波次 {0}"),
    ("savefail", "保存失败！"),
    ("saving", "[accent]保存中…"),
    ("server.invalidport", "端口号无效！"),
    ("server.port", "端口："),
    ("server.shown", "已显示"),
    ("server.hidden", "已隐藏"),
    ("server.favorite", "收藏"),
    ("setting.communityservers.name", "获取社区服务器列表"),
    ("setting.fullscreen.name", "全屏"),
    ("setting.fpscap.name", "最大帧数"),
    ("setting.fpscap.none", "无"),
    ("setting.fpscap.text", "{0} FPS"),
    ("setting.macnotch.name", "适应刘海显示"),
    ("setting.musicvol.name", "音乐音量"),
    ("setting.playerlimit.name", "玩家数量限制"),
    ("setting.saveinterval.name", "自动保存间隔"),
    ("setting.seconds", "{0} 秒"),
    ("stat.armor", "护甲"),
    ("stat.damage", "伤害"),
    ("stat.explosiveness", "爆炸性"),
    ("stat.flammability", "燃烧性"),
    ("stat.health", "生命值"),
    ("stat.healthmultiplier", "生命值倍率"),
    ("stat.heatcapacity", "热容量"),
    ("stat.radioactivity", "放射性"),
    ("stat.range", "范围"),
    ("stat.reloadmultiplier", "开火速率倍率"),
    ("stat.size", "尺寸"),
    ("stat.speed", "移动速度"),
    ("stat.speedmultiplier", "移动速度倍率"),
    ("stat.temperature", "温度"),
    ("stat.viscosity", "粘度"),
    ("stat.atmosphere", "大气"),
    ("stat.capturewave", "占领波次"),
    ("stat.difficulty", "难度"),
    ("stat.duration", "持续时间"),
    ("stat.hardness", "硬度"),
    ("stat.id", "ID"),
    ("stat.landable", "可着陆"),
    ("stat.opacity", "透明度"),
    ("stat.placeable", "可放置于"),
    ("stat.planet", "星球"),
    ("stat.sectors", "区块"),
    ("stat.status", "状态"),
    ("server.invalidaddress", "地址无效！"),
    ("abandon.confirm", "该区块的核心将自毁。\\n确定吗？"),
    ("steam.friendsonly", "仅限好友"),
    (
        "steam.friendsonly.tooltip",
        "是否只有 Steam 好友才能加入你的游戏。\\n取消选中该选项将使你的游戏公开 - 任何人都可以加入。",
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
    ("connecting", "[accent]連線中……"),
    ("reconnecting", "[accent]重新連接中……"),
    ("connecting.data", "[accent]地圖資料載入中……"),
    ("disconnect", "已中斷連線。"),
    ("disconnect.error", "連線錯誤。"),
    ("disconnect.closed", "連線關閉。"),
    ("disconnect.timeout", "連線逾時。"),
    ("disconnect.data", "無法載入地圖資料！"),
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
    ("players", "{0} 名玩家"),
    ("players.single", "{0} 名玩家"),
    ("save.map", "地圖：{0}"),
    ("server.version", "[gray]v{0} {1}"),
    ("server.versions", "你的版本：[accent] {0}[]\\n伺服器版本：[accent] {1}[]"),
    ("server.refreshing", "伺服器重新整理中"),
    ("server.kicked.kick", "您已被踢出伺服器！"),
    ("server.kicked.whitelist", "您不在這裡的白名單內。"),
    ("server.kicked.serverClose", "伺服器已關閉。"),
    ("server.kicked.vote", "您已被投票踢出伺服器，再見。"),
    ("server.kicked.clientOutdated", "客戶端版本過舊！請更新遊戲！"),
    ("server.kicked.serverOutdated", "伺服器版本過舊！請讓房主更新！"),
    ("server.kicked.banned", "您已經在這個伺服器中被封鎖。"),
    ("server.kicked.typeMismatch", "該伺服器與您的版本不相容。"),
    ("server.kicked.playerLimit", "該伺服器已滿。請等待玩家離開。"),
    ("server.kicked.recentKick", "您最近曾被踢出伺服器。\\n請稍後再進行連線。"),
    ("server.kicked.nameInUse", "伺服器中已經\\n有人有相同的名稱了。"),
    ("server.kicked.nameEmpty", "您的名稱必須至少包含一個字母或數字。"),
    ("server.kicked.idInUse", "您已經在伺服器中！不允許使用兩個帳號。"),
    ("server.kicked.customClient", "這個伺服器不支援自訂的客戶端，請下載官方版本。"),
    ("server.kicked.gameover", "遊戲結束！"),
    ("server.kicked.serverRestarting", "伺服器正在重新啟動。"),
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
    ("editor.exists", "已存在同名地圖。"),
    ("editor.errorimage", "這是圖片檔，而非地圖。"),
    ("editor.errornot", "這不是地圖檔。"),
    ("editor.errorname", "地圖沒有定義名稱。"),
    ("editor.importmap", "匯入地圖"),
    ("editor.import.exists", "[scarlet]匯入失敗：[]已存在同名內建地圖！"),
    (
        "editor.overwrite.confirm",
        "[scarlet]警告！[] 已存在同名地圖。確定要覆蓋嗎？",
    ),
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
    ("waves.edit", "編輯……"),
    ("waves.copy", "複製到剪貼簿"),
    ("waves.load", "從剪貼簿載入"),
    ("rules.invaliddata", "無效的剪貼板數據"),
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
    ("rules.wavelimit", "到達指定波次後結束地圖"),
    ("rules.wavespacing", "波次間隔：[lightgray]（秒）"),
    ("rules.initialwavespacing", "初始波次間隔：[lightgray]（秒）"),
    ("rules.dropzoneradius", "空降區半徑：[lightgray]（格）"),
    ("rules.waitForWaveToEnd", "等待所有敵人毀滅才開始下一波次"),
    ("rules.attack", "攻擊模式"),
    ("rules.corecapture", "佔領摧毀的核心"),
    ("rules.infiniteresources", "無限資源"),
    ("rules.schematic", "允許使用藍圖"),
    ("rules.allowedit", "Allow Editing Rules"),
    (
        "rules.allowedit.info",
        "When enabled, the player can edit rules in-game via the button in the bottom left corner of the Pause menu.",
    ),
    (
        "rules.alloweditworldprocessors",
        "Allow Editing World Processors",
    ),
    (
        "rules.alloweditworldprocessors.info",
        "When enabled, world logic blocks can be placed and edited even outside the editor.",
    ),
    ("rules.hidebannedblocks", "隱藏禁用的建築"),
    ("bannedblocks", "禁用方塊"),
    ("unbannedblocks", "Unbanned Blocks"),
    ("bannedunits", "禁用單位"),
    ("unbannedunits", "Unbanned Units"),
    ("bannedblocks.whitelist", "Banned Blocks As Whitelist"),
    ("bannedunits.whitelist", "Banned Units As Whitelist"),
    ("addall", "全部加入"),
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
    ("rules.title.teams", "分隊"),
    ("rules.enemyteam", "敵方隊伍"),
    ("rules.playerteam", "玩家隊伍"),
    ("rules.weather", "天氣"),
    ("rules.weather.frequency", "頻率："),
    ("rules.weather.always", "永遠"),
    ("rules.weather.duration", "持續時間："),
    (
        "rules.protectcores.info",
        "When disabled, the core no-build radius won't affect this team.\\nPlayers won't be assigned to unprotected teams.",
    ),
    (
        "rules.checkplacement.info",
        "When disabled, buildings of this team are ignored in placement range checks.",
    ),
    (
        "rules.randomwaveai.info",
        "Makes units spawned in waves target random structures instead of directly attacking the core or power generators.",
    ),
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
    ("abandon", "放棄"),
    ("add", "新增……"),
    ("all", "All"),
    ("be.check", "檢查是否有新的更新"),
    ("be.noupdates", "沒有新的更新。"),
    ("category.blocks.name", "選取方塊"),
    ("category.command.name", "單位指揮"),
    ("category.general.name", "一般"),
    ("category.multiplayer.name", "多人"),
    ("category.view.name", "查看"),
    ("copied", "已複製。"),
    ("crash.export", "匯出當機報告。"),
    ("data.export", "匯出數據"),
    ("data.import", "匯入數據"),
    ("data.openfolder", "開啟檔案資料夾"),
    ("editor.worldprocessors", "World Processors"),
    ("globalitems", "[accent]全域物品"),
    ("host", "伺服器"),
    (
        "host.info",
        "[accent]建立伺服器[]按鍵會在連接埠[scarlet]6567[]建立一個伺服器。\\n所有跟您在同一個[lightgray]網路或區域網路[]環境的玩家應該能在他們的伺服器清單中找到您的伺服器。\\n\\n如果您希望網際網路上的玩家透過 IP 位置連線到您的伺服器，您必須設定[accent]連接埠轉發[]。\\n\\n[lightgray]注意：如果區域網路內有玩家無法連線至您的伺服器，請務必確認您已於防火牆設定中開放 Mindustry 存取您的區域網路。請注意公用網路有時不允許搜尋伺服器。",
    ),
    ("hostserver.mobile", "建立\\n伺服器"),
    (
        "join.info",
        "您可以在此輸入欲連線的[accent]伺服器 IP 位置[]，或尋找[accent]區域網路[]／[accent]網際網路[]內的伺服器。目前支援區域網路與網際網路連線。\\n\\n[lightgray]如果您想透過 IP 位置連線到他人的伺服器，您必須向他們詢問 IP 位置。自己的 IP 位置可以從 Google 上搜尋到。",
    ),
    ("joingame.title", "加入遊戲"),
    ("keybind.chat.name", "聊天"),
    ("keybind.move_x.name", "水平移動"),
    (
        "language.restart",
        "請重新啟動遊戲以使選取的語言生效。",
    ),
    (
        "link.discord.description",
        "官方 Mindustry Discord 聊天室",
    ),
    (
        "linkfail",
        "無法打開連結！\\n我們已將該網址複製到您的剪貼簿。",
    ),
    ("load", "載入"),
    ("loading", "[accent]載入中……"),
    (
        "mod.noerrorplay",
        "[scarlet]您使用了有問題的模組。[] 遊戲前請先停用相關模組或修正問題。",
    ),
    ("noname", "請先選擇一個[accent]玩家名稱[]。"),
    ("objective", "地圖目標"),
    ("off", "關閉"),
    ("on", "開啟"),
    ("overwrite", "覆寫"),
    ("players.search", "搜尋"),
    ("quit.confirm", "您確定要結束嗎？"),
    ("save", "儲存"),
    ("save.autosave", "自動存檔：{0}"),
    ("save.date", "最後存檔時間：{0}"),
    ("save.delete", "刪除"),
    ("save.delete.confirm", "您確定要刪除這個存檔嗎？"),
    ("save.import", "匯入存檔"),
    ("save.import.invalid", "[accent]這是個無效的存檔！"),
    ("save.new", "新存檔"),
    ("save.newslot", "存檔名稱："),
    ("save.nocampaign", "無法匯入單一戰役中的存檔。"),
    ("save.none", "找不到存檔！"),
    ("save.overwrite", "您確定要覆寫存檔嗎？"),
    ("save.playtime", "遊玩時間：{0}"),
    ("save.rename", "重新命名"),
    ("save.rename.text", "新名稱："),
    ("save.search", "搜尋儲存的遊戲……"),
    ("save.wave", "波次：{0}"),
    ("savefail", "存檔失敗！"),
    ("saving", "[accent]儲存中……"),
    ("server.invalidport", "無效的連接埠！"),
    ("server.port", "連接埠："),
    ("server.shown", "已顯示"),
    ("server.hidden", "已隱藏"),
    ("server.favorite", "收藏"),
    (
        "setting.communityservers.name",
        "Fetch Community Server List",
    ),
    ("setting.fullscreen.name", "全螢幕"),
    ("setting.fpscap.name", "最大FPS"),
    ("setting.fpscap.none", "無"),
    ("setting.fpscap.text", "{0}FPS"),
    ("setting.macnotch.name", "使界面適應顯示槽口"),
    ("setting.musicvol.name", "音樂音量"),
    ("setting.playerlimit.name", "玩家數限制"),
    ("setting.saveinterval.name", "自動存檔間隔"),
    ("setting.seconds", "{0}秒"),
    ("stat.armor", "裝甲"),
    ("stat.damage", "傷害"),
    ("stat.explosiveness", "爆炸性"),
    ("stat.flammability", "易燃性"),
    ("stat.health", "耐久度"),
    ("stat.healthmultiplier", "血量加成"),
    ("stat.heatcapacity", "熱容量"),
    ("stat.radioactivity", "輻射性"),
    ("stat.range", "範圍"),
    ("stat.reloadmultiplier", "射速加成"),
    ("stat.size", "大小"),
    ("stat.speed", "速度"),
    ("stat.speedmultiplier", "速度加成"),
    ("stat.temperature", "溫度"),
    ("stat.viscosity", "黏度"),
    ("stat.atmosphere", "大氣"),
    ("stat.capturewave", "佔領波次"),
    ("stat.difficulty", "難度"),
    ("stat.duration", "持續時間"),
    ("stat.hardness", "硬度"),
    ("stat.id", "ID"),
    ("stat.landable", "可著陸"),
    ("stat.opacity", "透明度"),
    ("stat.placeable", "可放置於"),
    ("stat.planet", "星球"),
    ("stat.sectors", "區塊"),
    ("stat.status", "狀態"),
    ("server.invalidaddress", "地址無效！"),
    ("abandon.confirm", "該區塊的核心將自毀。\\n確定嗎？"),
    ("steam.friendsonly", "僅限好友"),
    (
        "steam.friendsonly.tooltip",
        "是否只有 Steam 好友可以加入您的遊戲。\\n取消勾選後，遊戲為公開的，任何人都可以加入。",
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
            ("server.refreshing", "Refreshing server"),
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
            ("data.export", "Export Data"),
            ("host.info", "The [accent]host[] button hosts a server on the specified port.\\nAnybody on the same [lightgray]wifi or local network[] should be able to see your server in their server list.\\n\\nIf you want people to be able to connect from anywhere by IP, [accent]port forwarding[] is required.\\n\\n[lightgray]Note: If someone is experiencing trouble connecting to your LAN game, make sure you have allowed Mindustry access to your local network in your firewall settings. Note that public networks sometimes do not allow server discovery."),
            ("keybind.chat.name", "Chat"),
            ("save.search", "Search saved games..."),
            ("stat.id", "ID"),
            ("stat.placeable", "Placeable On"),
            ("quit", "Quit"),
        ] {
            assert_eq!(upstream_bundle_en_value(key), Some(expected));
        }
        assert_eq!(upstream_bundle_en_value("waves.edit"), Some("Edit..."));
        assert_eq!(
            upstream_bundle_en_value("waves.copy"),
            Some("Copy to Clipboard")
        );
        assert_eq!(
            upstream_bundle_en_value("waves.load"),
            Some("Load from Clipboard")
        );
        assert_eq!(
            upstream_bundle_en_value("rules.invaliddata"),
            Some("Invalid clipboard data.")
        );
        assert_eq!(
            upstream_bundle_en_value("rules.wavelimit"),
            Some("Map Ends After Wave")
        );
        assert_eq!(
            upstream_bundle_en_value("rules.wavespacing"),
            Some("Wave Spacing:[lightgray] (sec)")
        );
        assert_eq!(
            upstream_bundle_en_value("rules.initialwavespacing"),
            Some("Initial Wave Spacing:[lightgray] (sec)")
        );
        assert_eq!(
            upstream_bundle_en_value("rules.dropzoneradius"),
            Some("Drop Zone Radius:[lightgray] (tiles)")
        );
        assert_eq!(
            upstream_bundle_en_value("rules.hidebannedblocks"),
            Some("Hide Banned Blocks")
        );
        assert_eq!(
            upstream_bundle_en_value("bannedblocks.whitelist"),
            Some("Banned Blocks As Whitelist")
        );
        assert_eq!(
            upstream_bundle_en_value("bannedunits.whitelist"),
            Some("Banned Units As Whitelist")
        );
        assert_eq!(
            upstream_bundle_en_value("rules.allowedit"),
            Some("Allow Editing Rules")
        );
        assert_eq!(
            upstream_bundle_en_value("rules.allowedit.info"),
            Some("When enabled, the player can edit rules in-game via the button in the bottom left corner of the Pause menu.")
        );
        assert_eq!(upstream_bundle_en_value("rules.title.teams"), Some("Teams"));
        assert_eq!(
            upstream_bundle_en_value("rules.playerteam"),
            Some("Player Team")
        );
        assert_eq!(
            upstream_bundle_en_value("rules.enemyteam"),
            Some("Enemy Team")
        );
        assert_eq!(upstream_bundle_en_value("rules.weather"), Some("Weather"));
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
            upstream_menu_bundle_value_for_locale("zh_TW", "server.refreshing"),
            Some("伺服器重新整理中")
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
            upstream_menu_bundle_value_for_locale("zh_CN", "data.export"),
            Some("导出数据")
        );
        assert_eq!(
            upstream_menu_bundle_value_for_locale("zh_CN", "waves.edit"),
            Some("编辑")
        );
        assert_eq!(
            upstream_menu_bundle_value_for_locale("zh_CN", "waves.copy"),
            Some("复制到剪贴板")
        );
        assert_eq!(
            upstream_menu_bundle_value_for_locale("zh_CN", "waves.load"),
            Some("从剪贴板读取")
        );
        assert_eq!(
            upstream_menu_bundle_value_for_locale("zh_CN", "rules.invaliddata"),
            Some("剪贴板数据无效。")
        );
        assert_eq!(
            upstream_menu_bundle_value_for_locale("zh_CN", "rules.wavelimit"),
            Some("波数到达后结束地图")
        );
        assert_eq!(
            upstream_menu_bundle_value_for_locale("zh_CN", "rules.wavespacing"),
            Some("波次间隔：[lightgray]（秒）")
        );
        assert_eq!(
            upstream_menu_bundle_value_for_locale("zh_CN", "rules.initialwavespacing"),
            Some("初始波次间隔：[lightgray]（秒）")
        );
        assert_eq!(
            upstream_menu_bundle_value_for_locale("zh_CN", "rules.dropzoneradius"),
            Some("空降区半径：[lightgray]（格）")
        );
        assert_eq!(
            upstream_menu_bundle_value_for_locale("zh_CN", "rules.hidebannedblocks"),
            Some("隐藏禁用的建筑")
        );
        assert_eq!(
            upstream_menu_bundle_value_for_locale("zh_CN", "bannedblocks.whitelist"),
            Some("将禁用建筑列入白名单")
        );
        assert_eq!(
            upstream_menu_bundle_value_for_locale("zh_CN", "bannedunits.whitelist"),
            Some("将禁用单位列入白名单")
        );
        assert_eq!(
            upstream_menu_bundle_value_for_locale("zh_CN", "rules.allowedit"),
            Some("允许规则编辑")
        );
        assert_eq!(
            upstream_menu_bundle_value_for_locale("zh_CN", "rules.allowedit.info"),
            Some("启用后，玩家可以通过暂停菜单左下角的按钮编辑游戏中的规则。")
        );
        assert_eq!(
            upstream_menu_bundle_value_for_locale("zh_CN", "rules.title.teams"),
            Some("队伍")
        );
        assert_eq!(
            upstream_menu_bundle_value_for_locale("zh_CN", "rules.playerteam"),
            Some("玩家队伍")
        );
        assert_eq!(
            upstream_menu_bundle_value_for_locale("zh_CN", "rules.enemyteam"),
            Some("敌方队伍")
        );
        assert_eq!(
            upstream_menu_bundle_value_for_locale("zh_CN", "rules.weather"),
            Some("天气")
        );
        assert_eq!(
            upstream_menu_bundle_value_for_locale("zh_CN", "keybind.chat.name"),
            Some("聊天")
        );
        assert_eq!(
            upstream_menu_bundle_value_for_locale("zh_CN", "setting.fullscreen.name"),
            Some("全屏")
        );
        assert_eq!(
            upstream_menu_bundle_value_for_locale("zh_CN", "stat.placeable"),
            Some("可放置于")
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
            upstream_menu_bundle_value_for_locale("zh_TW", "waves.edit"),
            Some("編輯……")
        );
        assert_eq!(
            upstream_menu_bundle_value_for_locale("zh_TW", "waves.copy"),
            Some("複製到剪貼簿")
        );
        assert_eq!(
            upstream_menu_bundle_value_for_locale("zh_TW", "waves.load"),
            Some("從剪貼簿載入")
        );
        assert_eq!(
            upstream_menu_bundle_value_for_locale("zh_TW", "rules.invaliddata"),
            Some("無效的剪貼板數據")
        );
        assert_eq!(
            upstream_menu_bundle_value_for_locale("zh_TW", "rules.wavelimit"),
            Some("到達指定波次後結束地圖")
        );
        assert_eq!(
            upstream_menu_bundle_value_for_locale("zh_TW", "rules.wavespacing"),
            Some("波次間隔：[lightgray]（秒）")
        );
        assert_eq!(
            upstream_menu_bundle_value_for_locale("zh_TW", "rules.initialwavespacing"),
            Some("初始波次間隔：[lightgray]（秒）")
        );
        assert_eq!(
            upstream_menu_bundle_value_for_locale("zh_TW", "rules.dropzoneradius"),
            Some("空降區半徑：[lightgray]（格）")
        );
        assert_eq!(
            upstream_menu_bundle_value_for_locale("zh_TW", "rules.hidebannedblocks"),
            Some("隱藏禁用的建築")
        );
        assert_eq!(
            upstream_menu_bundle_value_for_locale("zh_TW", "bannedblocks.whitelist"),
            Some("Banned Blocks As Whitelist")
        );
        assert_eq!(
            upstream_menu_bundle_value_for_locale("zh_TW", "bannedunits.whitelist"),
            Some("Banned Units As Whitelist")
        );
        assert_eq!(
            upstream_menu_bundle_value_for_locale("zh_TW", "rules.allowedit"),
            Some("Allow Editing Rules")
        );
        assert_eq!(
            upstream_menu_bundle_value_for_locale("zh_TW", "rules.title.teams"),
            Some("分隊")
        );
        assert_eq!(
            upstream_menu_bundle_value_for_locale("zh_TW", "rules.playerteam"),
            Some("玩家隊伍")
        );
        assert_eq!(
            upstream_menu_bundle_value_for_locale("zh_TW", "rules.enemyteam"),
            Some("敵方隊伍")
        );
        assert_eq!(
            upstream_menu_bundle_value_for_locale("zh_TW", "rules.weather"),
            Some("天氣")
        );
        assert_eq!(
            upstream_menu_bundle_value_for_locale("zh_TW", "server.invalidaddress"),
            Some("地址無效！")
        );
        assert_eq!(
            upstream_menu_bundle_value_for_locale("unknown", "database"),
            Some("Core Database")
        );
    }
}
