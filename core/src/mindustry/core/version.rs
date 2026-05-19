#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionInfo {
    pub build_type: String,
    pub modifier: String,
    pub commit_hash: String,
    pub build_date: String,
    pub number: i32,
    pub build: i32,
    pub revision: i32,
    pub is_steam: bool,
    pub enabled: bool,
}

impl Default for VersionInfo {
    fn default() -> Self {
        Self {
            build_type: "unknown".into(),
            modifier: "unknown".into(),
            commit_hash: "unknown".into(),
            build_date: "unknown".into(),
            number: 0,
            build: 0,
            revision: 0,
            is_steam: false,
            enabled: true,
        }
    }
}

impl VersionInfo {
    pub fn from_properties<'a, I>(properties: I) -> Self
    where
        I: IntoIterator<Item = (&'a str, &'a str)>,
    {
        let map: std::collections::BTreeMap<&str, &str> = properties.into_iter().collect();
        let modifier = map
            .get("modifier")
            .copied()
            .unwrap_or("unknown")
            .to_string();
        let (build, revision) = parse_build_revision(map.get("build").copied().unwrap_or("0"));
        Self {
            build_type: map.get("type").copied().unwrap_or("unknown").to_string(),
            modifier: modifier.clone(),
            commit_hash: map
                .get("commitHash")
                .copied()
                .unwrap_or("unknown")
                .to_string(),
            build_date: map
                .get("buildDate")
                .copied()
                .unwrap_or("unknown")
                .to_string(),
            number: parse_i32(map.get("number").copied().unwrap_or("4"), 4),
            build,
            revision,
            is_steam: modifier.contains("steam"),
            enabled: true,
        }
    }

    pub fn is_at_least_current(&self, version: &str) -> bool {
        is_at_least(self.build, self.revision, version)
    }

    pub fn build_string(&self) -> String {
        build_string(self.build, self.revision)
    }

    pub fn combined(&self) -> String {
        combined(
            &self.build_type,
            &self.modifier,
            self.build,
            self.revision,
            &self.commit_hash,
        )
    }
}

pub fn parse_build_revision(build: &str) -> (i32, i32) {
    if let Some((major, minor)) = build.split_once('.') {
        match (major.parse::<i32>(), minor.parse::<i32>()) {
            (Ok(build), Ok(revision)) => (build, revision),
            _ => (-1, 0),
        }
    } else if build.parse::<i32>().is_ok() {
        (build.parse::<i32>().unwrap_or(-1), 0)
    } else {
        (-1, 0)
    }
}

pub fn is_at_least(build: i32, revision: i32, version: &str) -> bool {
    if build <= 0 || version.is_empty() {
        return true;
    }
    if let Some((major, minor)) = version.split_once('.') {
        let major = parse_i32(major, 0);
        let minor = parse_i32(minor, 0);
        build > major || (build == major && revision >= minor)
    } else {
        build >= parse_i32(version, 0)
    }
}

pub fn build_string(build: i32, revision: i32) -> String {
    if build < 0 {
        "custom".to_string()
    } else if revision == 0 {
        build.to_string()
    } else {
        format!("{build}.{revision}")
    }
}

pub fn combined(
    build_type: &str,
    modifier: &str,
    build: i32,
    revision: i32,
    commit_hash: &str,
) -> String {
    if build == -1 {
        return "custom build".to_string();
    }
    let prefix = if build_type == "official" {
        modifier
    } else {
        build_type
    };
    let commit = if commit_hash == "unknown" {
        String::new()
    } else {
        format!(" ({commit_hash})")
    };
    format!("{prefix} build {}{commit}", build_string(build, revision))
}

fn parse_i32(value: &str, fallback: i32) -> i32 {
    value.parse::<i32>().unwrap_or(fallback)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_properties_parse_build_revision_and_steam_like_java() {
        let info = VersionInfo::from_properties([
            ("type", "official"),
            ("number", "7"),
            ("modifier", "release-steam"),
            ("commitHash", "abc123"),
            ("buildDate", "2024-01-01"),
            ("build", "157.4"),
        ]);
        assert_eq!(info.build_type, "official");
        assert_eq!(info.number, 7);
        assert_eq!(info.modifier, "release-steam");
        assert_eq!(info.commit_hash, "abc123");
        assert_eq!(info.build_date, "2024-01-01");
        assert_eq!(info.build, 157);
        assert_eq!(info.revision, 4);
        assert!(info.is_steam);
        assert!(info.enabled);
        assert_eq!(info.build_string(), "157.4");
        assert_eq!(info.combined(), "release-steam build 157.4 (abc123)");
    }

    #[test]
    fn version_helpers_follow_upstream_edge_cases() {
        assert_eq!(parse_build_revision("157"), (157, 0));
        assert_eq!(parse_build_revision("157.4"), (157, 4));
        assert_eq!(parse_build_revision("oops"), (-1, 0));
        assert_eq!(parse_build_revision("157.oops"), (-1, 0));

        assert!(is_at_least(-1, 0, "999"));
        assert!(is_at_least(0, 0, "999"));
        assert!(is_at_least(157, 4, ""));
        assert!(is_at_least(157, 4, "157.3"));
        assert!(is_at_least(157, 4, "157.4"));
        assert!(!is_at_least(157, 3, "157.4"));
        assert!(is_at_least(158, 0, "157.9"));
        assert!(is_at_least(157, 0, "not-a-number"));

        assert_eq!(build_string(-1, 0), "custom");
        assert_eq!(build_string(157, 0), "157");
        assert_eq!(build_string(157, 4), "157.4");
        assert_eq!(
            combined("official", "release", 157, 0, "unknown"),
            "release build 157"
        );
        assert_eq!(
            combined("bleeding-edge", "release", 157, 4, "abc123"),
            "bleeding-edge build 157.4 (abc123)"
        );
        assert_eq!(
            combined("official", "release", -1, 0, "abc123"),
            "custom build"
        );
    }
}
