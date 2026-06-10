//! External link registry mirroring upstream `mindustry.ui.Links`.

use crate::mindustry::ui::upstream_menu_bundle_value_for_locale;

pub const UPSTREAM_LINK_SPECS: &[(&str, &str, &str, u32)] = &[
    (
        "discord",
        "https://discord.gg/mindustry",
        "discord",
        0x7289daff,
    ),
    (
        "changelog",
        "https://github.com/Anuken/Mindustry/releases",
        "list",
        0xffd37fff,
    ),
    ("trello", "https://trello.com/b/aE2tcUwF", "trello", 0x026aa7ff),
    (
        "wiki",
        "https://mindustrygame.github.io/wiki/",
        "book",
        0x0f142fff,
    ),
    (
        "suggestions",
        "https://github.com/Anuken/Mindustry-Suggestions/issues/new/choose/",
        "add",
        0xebebebff,
    ),
    (
        "reddit",
        "https://www.reddit.com/r/Mindustry/",
        "redditAlien",
        0xee593bff,
    ),
    (
        "itch.io",
        "https://anuke.itch.io/mindustry",
        "itchio",
        0xfa5c5cff,
    ),
    (
        "google-play",
        "https://play.google.com/store/apps/details?id=io.anuke.mindustry",
        "googleplay",
        0x689f38ff,
    ),
    (
        "f-droid",
        "https://f-droid.org/packages/io.anuke.mindustry/",
        "android",
        0x026aa7ff,
    ),
    (
        "github",
        "https://github.com/Anuken/Mindustry/",
        "github",
        0x24292eff,
    ),
    (
        "dev-builds",
        "https://github.com/Anuken/MindustryBuilds",
        "githubSquare",
        0xfafbfcff,
    ),
    (
        "bug",
        "https://github.com/Anuken/Mindustry/issues/new?assignees=&labels=bug&projects=&template=bug_report.yml",
        "wrench",
        0xcbd97fff,
    ),
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkEntry {
    pub name: String,
    pub title: String,
    pub description: String,
    pub link: String,
    pub icon_name: String,
    pub color_rgba: u32,
}

pub fn get_links() -> Vec<LinkEntry> {
    get_links_for_locale("en")
}

pub fn get_links_for_locale(locale: &str) -> Vec<LinkEntry> {
    UPSTREAM_LINK_SPECS
        .iter()
        .map(|(name, link, icon, color)| {
            let title_key = format!("link.{name}.title");
            let description_key = format!("link.{name}.description");
            LinkEntry {
                name: (*name).to_string(),
                title: upstream_menu_bundle_value_for_locale(locale, &title_key)
                    .map(str::to_string)
                    .unwrap_or_else(|| capitalize_words(&name.replace('-', " "))),
                description: upstream_menu_bundle_value_for_locale(locale, &description_key)
                    .unwrap_or("")
                    .to_string(),
                link: (*link).to_string(),
                icon_name: (*icon).to_string(),
                color_rgba: *color,
            }
        })
        .collect()
}

fn capitalize_words(value: &str) -> String {
    value
        .split(' ')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn links_match_upstream_order_urls_icons_and_colors() {
        let links = get_links();

        assert_eq!(links.len(), 12);
        assert_eq!(links[0].name, "discord");
        assert_eq!(links[0].link, "https://discord.gg/mindustry");
        assert_eq!(links[0].icon_name, "discord");
        assert_eq!(links[0].color_rgba, 0x7289daff);
        assert_eq!(links[11].name, "bug");
        assert!(links[11].link.contains("template=bug_report.yml"));
    }

    #[test]
    fn link_titles_use_bundle_or_java_capitalized_fallback() {
        let links = get_links();
        let dev_builds = links.iter().find(|link| link.name == "dev-builds").unwrap();
        assert_eq!(dev_builds.title, "Dev Builds");

        let itch = links.iter().find(|link| link.name == "itch.io").unwrap();
        assert_eq!(itch.title, "Itch.io");
    }
}
