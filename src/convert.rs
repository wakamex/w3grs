//! Conversion helpers ported from `w3gjs/src/convert.ts`.

pub fn player_color(color: u8) -> &'static str {
    match color {
        0 => "#ff0303",
        1 => "#0042ff",
        2 => "#1ce6b9",
        3 => "#540081",
        4 => "#fffc00",
        5 => "#fe8a0e",
        6 => "#20c000",
        7 => "#e55bb0",
        8 => "#959697",
        9 => "#7ebff1",
        10 => "#106246",
        11 => "#4a2a04",
        12 => "#9b0000",
        13 => "#0000c3",
        14 => "#00eaff",
        15 => "#be00fe",
        16 => "#ebcd87",
        17 => "#f8a48b",
        18 => "#bfff80",
        19 => "#dcb9eb",
        20 => "#282828",
        21 => "#ebf0ff",
        22 => "#00781e",
        23 => "#a46f33",
        _ => "000000",
    }
}

pub fn game_version(version: u32) -> String {
    if version == 10030 {
        "1.30.2+".to_string()
    } else if version > 10030 && version < 10100 {
        let str_version = version.to_string();
        format!("1.{}", &str_version[str_version.len() - 2..])
    } else if version >= 10100 {
        let str_version = version.to_string();
        format!("2.{}", &str_version[str_version.len() - 2..])
    } else {
        format!("1.{version}")
    }
}

pub fn map_filename(map_path: &str) -> String {
    let filename = map_path
        .rsplit(|char| ['\\', '/'].contains(&char))
        .next()
        .unwrap_or("");

    let lower = filename.to_ascii_lowercase();
    if lower.ends_with(".w3x") || lower.ends_with(".w3m") {
        filename.to_string()
    } else {
        String::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_game_versions_like_w3gjs() {
        assert_eq!(game_version(26), "1.26");
        assert_eq!(game_version(10030), "1.30.2+");
        assert_eq!(game_version(10032), "1.32");
        assert_eq!(game_version(10202), "2.02");
    }

    #[test]
    fn extracts_map_filename_with_mixed_separators() {
        assert_eq!(map_filename("Maps\\test\\somemap.w3x"), "somemap.w3x");
        assert_eq!(map_filename("Maps//test//somemap.w3x"), "somemap.w3x");
        assert_eq!(map_filename("Maps//test\\somemap.w3x"), "somemap.w3x");
        assert_eq!(map_filename("Maps\\test//somemap.w3x"), "somemap.w3x");
    }
}
