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
    ("about.button", "About"),
    ("discord", "Join the Mindustry Discord!"),
    ("editor", "Editor"),
    ("maps", "Maps"),
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
    ("joingame.ip", "Address:"),
    ("none.found", "[lightgray]<none found>"),
    ("none", "<none>"),
    ("schematic.search", "Search schematics..."),
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
    ("editor.import", "Import..."),
    ("editor.export", "Export..."),
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
    ("unknown", "Unknown"),
    ("workshop", "Workshop"),
    ("mods", "Mods"),
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
    ("about.button", "关于"),
    ("discord", "加入 Mindustry 的 Discord！"),
    ("editor", "地图编辑器"),
    ("maps", "地图"),
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
    ("joingame.ip", "地址："),
    ("none.found", "[lightgray]< 未找到 >"),
    ("none", "< 无 >"),
    ("schematic.search", "搜索蓝图"),
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
    ("editor.import", "导入"),
    ("editor.export", "导出"),
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
    ("unknown", "未知"),
    ("workshop", "创意工坊"),
    ("mods", "模组"),
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
    ("about.button", "關於"),
    ("discord", "加入 Mindustry 的 Discord 聊天室！"),
    ("editor", "地圖編輯器"),
    ("maps", "地圖"),
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
    ("joingame.ip", "IP 位置："),
    ("none.found", "[lightgray]〈查無結果〉"),
    ("none", "〈沒有〉"),
    ("schematic.search", "搜尋藍圖..."),
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
    ("editor.import", "匯入……"),
    ("editor.export", "匯出……"),
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
    ("unknown", "未知"),
    ("workshop", "工作坊"),
    ("mods", "模組"),
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
