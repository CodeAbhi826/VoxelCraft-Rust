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
    ("me", "me <action>"),
    ("say", "say <message>"),
    ("seed", "seed"),
    ("time", "time <set <day|noon|night|midnight|ticks>|query>"),
    ("weather", "weather <clear|rain|thunder> [seconds]"),
];

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
        for need in ["help", "seed", "gamemode", "time", "weather", "say", "me"] {
            assert!(names.contains(&need), "missing {need}");
        }
    }
}
