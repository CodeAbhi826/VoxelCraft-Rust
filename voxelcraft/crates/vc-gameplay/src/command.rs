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
    (
        "effect",
        "effect <give <target> <effect> [seconds] [amplifier]|clear [target]>",
    ),
    ("enchant", "enchant <target> <enchantment> [level]"),
    ("summon", "summon <entity> [x y z]"),
    ("setblock", "setblock <x> <y> <z> <block>"),
    ("fill", "fill <x1> <y1> <z1> <x2> <y2> <z2> <block>"),
    (
        "clone",
        "clone <x1> <y1> <z1> <x2> <y2> <z2> <dx> <dy> <dz>",
    ),
    (
        "scoreboard",
        "scoreboard <objectives <add|list|remove|setdisplay>|players <set|add|remove|get|list|reset>>",
    ),
    (
        "team",
        "team <add <name> [display]|remove <name>|list [name]|join <team> <target>|leave <target>>",
    ),
    ("tellraw", "tellraw <target> <json>"),
    (
        "title",
        "title <target> <title|subtitle|actionbar|clear|times> [...]",
    ),
    (
        "bossbar",
        "bossbar <add|remove|list|get|set> [...]",
    ),
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

/// 3.6d: world-edit volume cap (vanilla fill/clone limit).
pub const EDIT_VOLUME_LIMIT: usize = 32768;

/// 3.7c: tellraw text extraction — plain strings pass through;
/// objects/arrays concatenate every `"text"` value in order.
/// Formatting keys (color/bold/hover...) are ignored: the chat
/// renderer is unformatted (disclosed — nested `text` keys leak
/// into the output by the same rule).
pub fn tellraw_text(json: &str) -> String {
    let t = json.trim();
    if !(t.starts_with('{') || t.starts_with('[')) {
        return unquote(t);
    }
    let c: Vec<char> = t.chars().collect();
    let key: Vec<char> = vec!['"', 't', 'e', 'x', 't', '"'];
    let mut out = String::new();
    let mut i = 0;
    while i + key.len() <= c.len() {
        if c[i..i + key.len()] == key {
            let mut j = i + key.len();
            while j < c.len() && c[j].is_whitespace() {
                j += 1;
            }
            if c.get(j) == Some(&':') {
                j += 1;
                while j < c.len() && c[j].is_whitespace() {
                    j += 1;
                }
                if c.get(j) == Some(&'"') {
                    j += 1;
                    let mut val = String::new();
                    let mut esc = false;
                    while j < c.len() {
                        let ch = c[j];
                        if esc {
                            val.push(match ch {
                                'n' => '\n',
                                _ => ch,
                            });
                            esc = false;
                        } else if ch == '\\' {
                            esc = true;
                        } else if ch == '"' {
                            break;
                        } else {
                            val.push(ch);
                        }
                        j += 1;
                    }
                    out.push_str(&val);
                    i = j + 1;
                    continue;
                }
            }
        }
        i += 1;
    }
    out
}

/// strip surrounding quotes + unescape `\"`/`\\` (tellraw plain form).
fn unquote(s: &str) -> String {
    let t = s.trim();
    if t.len() >= 2 && t.starts_with('"') && t.ends_with('"') {
        t[1..t.len() - 1]
            .replace("\\\"", "\"")
            .replace("\\\\", "\\")
    } else {
        t.to_string()
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
            "help",
            "seed",
            "gamemode",
            "time",
            "weather",
            "say",
            "me",
            "give",
            "tp",
            "kill",
            "effect",
            "enchant",
            "summon",
            "setblock",
            "fill",
            "clone",
            "scoreboard",
            "team",
            "tellraw",
            "title",
            "bossbar",
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

    #[test]
    fn tellraw_extracts_text() {
        assert_eq!(tellraw_text("hello"), "hello");
        assert_eq!(tellraw_text("\"quoted\""), "quoted");
        assert_eq!(tellraw_text("{\"text\":\"hi\"}"), "hi");
        assert_eq!(
            tellraw_text("{\"text\":\"a\",\"extra\":[{\"text\":\"b\"}]}"),
            "ab"
        );
        assert_eq!(tellraw_text("[\"\",{\"text\":\"x\"}]"), "x");
        assert_eq!(tellraw_text("{\"text\":\"a\\\"b\"}"), "a\"b");
    }
}
