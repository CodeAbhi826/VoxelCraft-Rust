//! 3.6a: command line parser — strip the `/` prefix, quote-aware
//! arg split, and the command table (name + usage for `/help`).
//! Handlers live in game.rs (they mutate the app); this crate owns
//! only the pure parsing. No cheat gate: commands are always
//! available in this engine's single-player world (disclosed —
//! vanilla gates them behind cheats-enabled/op level).

/// split a command line into argv; `"double quotes"` group words,
/// backslash escapes the next char. The leading `/` must already be
/// stripped (bare words are tolerated — the chat layer routes only
/// `/` lines here).
pub fn split_args(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut in_quotes = false;
    let mut escaped = false;
    // `has` marks a pending arg (set by quotes too, so `""` keeps an
    // empty arg while bare spacing emits nothing)
    let mut has = false;
    for ch in line.chars() {
        if escaped {
            cur.push(ch);
            escaped = false;
            has = true;
            continue;
        }
        match ch {
            '\\' => escaped = true,
            '"' => {
                in_quotes = !in_quotes;
                has = true;
            }
            c if c.is_whitespace() && !in_quotes => {
                if has {
                    out.push(std::mem::take(&mut cur));
                    has = false;
                }
            }
            c => {
                cur.push(c);
                has = true;
            }
        }
    }
    if has {
        out.push(cur);
    }
    out
}

/// (name, usage) for every implemented command — `/help` lists these;
/// `/help <name>` shows the usage.
pub const COMMANDS: &[(&str, &str)] = &[
    (
        "gamemode",
        "gamemode <survival|creative|adventure|spectator|0|1|2|3>",
    ),
    ("help", "help [<command>]"),
    ("give", "give <target> <item> [count]"),
    ("kill", "kill [target]"),
    ("me", "me <action>"),
    ("say", "say <message>"),
    ("seed", "seed"),
    ("time", "time <set <day|noon|night|midnight|ticks>|query>"),
    ("tp", "tp <x> <y> <z> | tp <target> [<x> <y> <z>]"),
    ("weather", "weather <clear|rain|thunder> [seconds]"),
];

/// 3.6b: target selectors. Single-player resolution (game.rs): every
/// player selector is the local player; `@e` is the nearest living
/// mob. Bare `[...]` args are NOT parsed (disclosed).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Selector {
    This,
    NearestPlayer,
    AllPlayers,
    RandomPlayer,
    NearestEntity,
}

/// parse `@p @s @a @e @r` (anything else = None, not a selector).
pub fn parse_selector(s: &str) -> Option<Selector> {
    match s {
        "@s" => Some(Selector::This),
        "@p" => Some(Selector::NearestPlayer),
        "@a" => Some(Selector::AllPlayers),
        "@r" => Some(Selector::RandomPlayer),
        "@e" => Some(Selector::NearestEntity),
        _ => None,
    }
}

/// snake_case registry name (mirrors lang::key_for's transform so
/// `give` accepts the same shape vanilla's item ids use).
fn snake(name: &str) -> String {
    let mut s = String::with_capacity(name.len());
    for c in name.chars() {
        if c.is_ascii_alphanumeric() {
            s.push(c.to_ascii_lowercase());
        } else if c == ' ' || c == '-' {
            s.push('_');
        }
    }
    s
}

/// 3.6b: `/tp` coordinate — absolute or `~`-relative to `base`
/// (vanilla notation; bare `~` = the base itself).
pub fn parse_coord(s: &str, base: f32) -> Option<f32> {
    if let Some(rel) = s.strip_prefix('~') {
        if rel.is_empty() {
            Some(base)
        } else {
            rel.parse::<f32>().ok().map(|d| base + d)
        }
    } else {
        s.parse::<f32>().ok()
    }
}

/// 3.6b: item lookup for `/give` — numeric id or registry snake name
/// with an optional `namespace:` prefix (`voxelcraft:stone` works).
pub fn item_by_name(s: &str) -> Option<u16> {
    use vc_blocks::blocks::{def, BLOCK_COUNT};
    if let Ok(id) = s.parse::<u16>() {
        if (id as usize) < BLOCK_COUNT {
            return Some(id);
        }
        return None;
    }
    let bare = s.rsplit(':').next().unwrap_or(s);
    (0..BLOCK_COUNT as u16).find(|&id| snake(def(id).name) == bare)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_plain_words() {
        assert_eq!(split_args("time set day"), vec!["time", "set", "day"]);
    }

    #[test]
    fn split_quotes_and_escapes() {
        assert_eq!(
            split_args("say \"hello brave world\""),
            vec!["say", "hello brave world"]
        );
        assert_eq!(split_args("say a\\\"b"), vec!["say", "a\"b"]);
    }

    #[test]
    fn table_covers_implemented_commands() {
        let names: Vec<&str> = COMMANDS.iter().map(|(n, _)| *n).collect();
        for need in [
            "help", "seed", "gamemode", "time", "weather", "say", "me", "give", "tp", "kill",
        ] {
            assert!(names.contains(&need), "missing {need}");
        }
    }

    #[test]
    fn selectors_parse_and_reject() {
        use super::Selector::*;
        assert_eq!(parse_selector("@s"), Some(This));
        assert_eq!(parse_selector("@p"), Some(NearestPlayer));
        assert_eq!(parse_selector("@a"), Some(AllPlayers));
        assert_eq!(parse_selector("@r"), Some(RandomPlayer));
        assert_eq!(parse_selector("@e"), Some(NearestEntity));
        assert_eq!(parse_selector("SomePlayer"), None);
        assert_eq!(parse_selector("@p[type=zombie]"), None);
    }

    #[test]
    fn item_lookup_spot_checks() {
        use vc_blocks::blocks::{IRON_SWORD, STONE};
        assert_eq!(item_by_name("3"), Some(STONE));
        assert_eq!(item_by_name("stone"), Some(STONE));
        assert_eq!(item_by_name("voxelcraft:stone"), Some(STONE));
        assert_eq!(item_by_name("iron_sword"), Some(IRON_SWORD));
        assert_eq!(item_by_name("not_an_item"), None);
        assert_eq!(item_by_name("9999"), None);
    }

    #[test]
    fn coords_absolute_and_relative() {
        assert_eq!(parse_coord("10", 5.0), Some(10.0));
        assert_eq!(parse_coord("~", 5.0), Some(5.0));
        assert_eq!(parse_coord("~-3", 5.0), Some(2.0));
        assert_eq!(parse_coord("~2.5", 5.0), Some(7.5));
        assert_eq!(parse_coord("abc", 5.0), None);
    }
}
