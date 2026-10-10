//! 3.5a: chat log — capped message buffer with per-line TTL. The HUD
//! renders recent lines bottom-left (vanilla position); the input row
//! renders while open. `/` lines route to the 3.6 command parser; this
//! slice echoes player lines and stubs commands as unknown.

/// max retained lines (vanilla keeps 100 in the chat history)
pub const CHAT_CAP: usize = 100;
/// lines fade after ~10 s (vanilla 200 ticks at 20 tps)
pub const CHAT_TTL: f32 = 10.0;
/// vanilla chat input caps at 256 chars (1.16.5)
pub const CHAT_MAX_CHARS: usize = 256;

#[derive(Default, Debug)]
pub struct ChatLog {
    lines: std::collections::VecDeque<(String, f32, bool)>,
}

impl ChatLog {
    /// player or system line with a fresh TTL; drops oldest past the cap.
    pub fn push(&mut self, text: String) {
        self.push_sys(text, false);
    }

    /// system line (survives Commands Only visibility).
    pub fn system(&mut self, text: String) {
        self.push_sys(text, true);
    }

    fn push_sys(&mut self, text: String, sys: bool) {
        if self.lines.len() >= CHAT_CAP {
            self.lines.pop_front();
        }
        self.lines.push_back((text, CHAT_TTL, sys));
    }

    /// `<name> text` player line.
    pub fn say(&mut self, name: &str, text: &str) {
        self.push(format!("<{name}> {text}"));
    }

    /// decay TTLs; expired lines vanish (the toast/caption pattern).
    pub fn tick(&mut self, dt: f32) {
        for l in self.lines.iter_mut() {
            l.1 -= dt;
        }
        while self.lines.front().is_some_and(|l| l.1 <= 0.0) {
            self.lines.pop_front();
        }
    }

    /// newest-first visible lines, up to `n`. Commands Only mode passes
    /// `only_sys = true` (player lines hidden, system lines stay).
    pub fn recent(&self, n: usize) -> Vec<&str> {
        self.recent_sys(n, false)
    }

    pub fn recent_sys(&self, n: usize, only_sys: bool) -> Vec<&str> {
        self.lines
            .iter()
            .rev()
            .filter(|(_, _, sys)| !only_sys || *sys)
            .take(n)
            .map(|(s, _, _)| s.as_str())
            .collect()
    }

    pub fn len(&self) -> usize {
        self.lines.len()
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn say_formats_with_brackets() {
        let mut c = ChatLog::default();
        c.say("Player", "hello");
        assert_eq!(c.recent(1), vec!["<Player> hello"]);
    }

    #[test]
    fn cap_drops_oldest() {
        let mut c = ChatLog::default();
        for i in 0..CHAT_CAP + 5 {
            c.push(format!("m{i}"));
        }
        assert_eq!(c.len(), CHAT_CAP);
        assert_eq!(c.recent(CHAT_CAP).last().copied(), Some("m5"));
    }

    #[test]
    fn ttl_expiry_removes_lines() {
        let mut c = ChatLog::default();
        c.push("old".to_string());
        c.tick(CHAT_TTL + 1.0);
        assert!(c.is_empty());
        c.push("new".to_string());
        c.tick(1.0);
        assert_eq!(c.len(), 1);
    }

    #[test]
    fn commands_only_hides_player_lines() {
        let mut c = ChatLog::default();
        c.say("Player", "hi");
        c.system("Server rules".to_string());
        assert_eq!(c.recent(2).len(), 2);
        assert_eq!(c.recent_sys(2, true), vec!["Server rules"]);
    }
}
