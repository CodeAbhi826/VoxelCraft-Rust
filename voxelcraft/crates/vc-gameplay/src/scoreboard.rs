//! 3.7a: scoreboard — objectives, per-entry scores, the sidebar
//! display slot. Criteria accept any string but only `dummy`-style
//! manual scoring exists (stat criteria like deathCount have no
//! hooks — disclosed). Name limits follow vanilla (objective ≤16,
//! entry ≤40).

use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug)]
pub struct Objective {
    pub criterion: String,
    pub display: String,
}

/// 3.7b: a team — display name + member entries. Options (colors,
/// friendly fire, nametag rules) are out of scope: single-player has
/// no PvP or nametag renderer (disclosed).
#[derive(Debug)]
pub struct Team {
    pub display: String,
    pub members: BTreeSet<String>,
}

#[derive(Default, Debug)]
pub struct Scoreboard {
    objectives: BTreeMap<String, Objective>,
    scores: BTreeMap<(String, String), i32>,
    sidebar: Option<String>,
    teams: BTreeMap<String, Team>,
}

impl Scoreboard {
    /// add an objective; Err when the name exists or breaks limits.
    pub fn add_objective(
        &mut self,
        name: &str,
        criterion: &str,
        display: &str,
    ) -> Result<(), String> {
        if name.is_empty() || name.len() > 16 {
            return Err("Objective name must be 1..=16 chars".to_string());
        }
        if self.objectives.contains_key(name) {
            return Err(format!("Objective {name} already exists"));
        }
        self.objectives.insert(
            name.to_string(),
            Objective {
                criterion: criterion.to_string(),
                display: if display.is_empty() {
                    name.to_string()
                } else {
                    display.to_string()
                },
            },
        );
        Ok(())
    }

    /// remove an objective + its scores; false when missing.
    pub fn remove_objective(&mut self, name: &str) -> bool {
        if self.objectives.remove(name).is_none() {
            return false;
        }
        self.scores.retain(|(o, _), _| o != name);
        if self.sidebar.as_deref() == Some(name) {
            self.sidebar = None;
        }
        true
    }

    /// (name, criterion, display), sorted.
    pub fn list_objectives(&self) -> Vec<(&str, &str, &str)> {
        self.objectives
            .iter()
            .map(|(n, o)| (n.as_str(), o.criterion.as_str(), o.display.as_str()))
            .collect()
    }

    fn check(&self, objective: &str, entry: &str) -> Result<(), String> {
        if !self.objectives.contains_key(objective) {
            return Err(format!("Unknown objective: {objective}"));
        }
        if entry.is_empty() || entry.len() > 40 {
            return Err("Entry name must be 1..=40 chars".to_string());
        }
        Ok(())
    }

    pub fn set_score(&mut self, objective: &str, entry: &str, value: i32) -> Result<(), String> {
        self.check(objective, entry)?;
        self.scores
            .insert((objective.to_string(), entry.to_string()), value);
        Ok(())
    }

    pub fn add_score(&mut self, objective: &str, entry: &str, delta: i32) -> Result<i32, String> {
        self.check(objective, entry)?;
        let v = self
            .scores
            .get(&(objective.to_string(), entry.to_string()))
            .copied()
            .unwrap_or(0)
            .wrapping_add(delta);
        self.scores
            .insert((objective.to_string(), entry.to_string()), v);
        Ok(v)
    }

    pub fn get_score(&self, objective: &str, entry: &str) -> Option<i32> {
        self.scores
            .get(&(objective.to_string(), entry.to_string()))
            .copied()
    }

    /// reset one entry's scores (Some) or every entry (None); returns
    /// cleared count.
    pub fn reset_scores(&mut self, entry: Option<&str>) -> usize {
        let before = self.scores.len();
        match entry {
            Some(e) => self.scores.retain(|(_, en), _| en != e),
            None => self.scores.clear(),
        }
        before - self.scores.len()
    }

    /// drop one (objective, entry) score; false when absent.
    pub fn clear_score(&mut self, objective: &str, entry: &str) -> bool {
        self.scores
            .remove(&(objective.to_string(), entry.to_string()))
            .is_some()
    }

    /// sidebar slot: Some(objective) shows it, None clears.
    pub fn set_display(&mut self, objective: Option<&str>) -> Result<(), String> {
        if let Some(o) = objective {
            if !self.objectives.contains_key(o) {
                return Err(format!("Unknown objective: {o}"));
            }
            self.sidebar = Some(o.to_string());
        } else {
            self.sidebar = None;
        }
        Ok(())
    }

    /// (display name, top-15 entry lines desc) for the HUD sidebar.
    pub fn sidebar_lines(&self) -> Option<(String, Vec<(String, i32)>)> {
        let name = self.sidebar.as_ref()?;
        let obj = self.objectives.get(name)?;
        let mut lines: Vec<(String, i32)> = self
            .scores
            .iter()
            .filter(|((o, _), _)| o == name)
            .map(|((_, e), v)| (e.clone(), *v))
            .collect();
        lines.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        lines.truncate(15);
        Some((obj.display.clone(), lines))
    }

    /// 3.7b: create a team; Err when the name exists or breaks the
    /// 16-char limit.
    pub fn add_team(&mut self, name: &str, display: &str) -> Result<(), String> {
        if name.is_empty() || name.len() > 16 {
            return Err("Team name must be 1..=16 chars".to_string());
        }
        if self.teams.contains_key(name) {
            return Err(format!("Team {name} already exists"));
        }
        self.teams.insert(
            name.to_string(),
            Team {
                display: if display.is_empty() {
                    name.to_string()
                } else {
                    display.to_string()
                },
                members: BTreeSet::new(),
            },
        );
        Ok(())
    }

    /// remove a team; false when missing.
    pub fn remove_team(&mut self, name: &str) -> bool {
        self.teams.remove(name).is_some()
    }

    /// (name, display, member count), sorted.
    pub fn list_teams(&self) -> Vec<(&str, &str, usize)> {
        self.teams
            .iter()
            .map(|(n, t)| (n.as_str(), t.display.as_str(), t.members.len()))
            .collect()
    }

    /// move `entry` onto `team` (leaving any previous team); Err when
    /// the team is missing.
    pub fn join_team(&mut self, team: &str, entry: &str) -> Result<(), String> {
        if !self.teams.contains_key(team) {
            return Err(format!("Unknown team: {team}"));
        }
        for t in self.teams.values_mut() {
            t.members.remove(entry);
        }
        self.teams
            .get_mut(team)
            .expect("checked")
            .members
            .insert(entry.to_string());
        Ok(())
    }

    /// pull `entry` off whatever team holds it; false when teamless.
    pub fn leave_team(&mut self, entry: &str) -> bool {
        let mut left = false;
        for t in self.teams.values_mut() {
            left |= t.members.remove(entry);
        }
        left
    }

    /// the team holding `entry`, if any.
    pub fn team_of(&self, entry: &str) -> Option<&str> {
        self.teams
            .iter()
            .find(|(_, t)| t.members.contains(entry))
            .map(|(n, _)| n.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn objectives_add_list_remove() {
        let mut s = Scoreboard::default();
        assert!(s.add_objective("kills", "dummy", "Kills").is_ok());
        assert!(s.add_objective("kills", "dummy", "").is_err());
        assert!(s
            .add_objective("this_name_is_way_too_long", "dummy", "")
            .is_err());
        assert_eq!(s.list_objectives().len(), 1);
        assert!(s.remove_objective("kills"));
        assert!(!s.remove_objective("kills"));
    }

    #[test]
    fn scores_set_add_get_reset() {
        let mut s = Scoreboard::default();
        s.add_objective("k", "dummy", "").unwrap();
        assert_eq!(s.get_score("k", "Player"), None);
        s.set_score("k", "Player", 5).unwrap();
        assert_eq!(s.add_score("k", "Player", 3).unwrap(), 8);
        assert!(s.set_score("nope", "Player", 1).is_err());
        assert_eq!(s.reset_scores(Some("Player")), 1);
        assert_eq!(s.get_score("k", "Player"), None);
    }

    #[test]
    fn sidebar_orders_desc() {
        let mut s = Scoreboard::default();
        s.add_objective("k", "dummy", "K").unwrap();
        s.set_score("k", "B", 1).unwrap();
        s.set_score("k", "A", 9).unwrap();
        s.set_display(Some("k")).unwrap();
        let (name, lines) = s.sidebar_lines().unwrap();
        assert_eq!(name, "K");
        assert_eq!(lines[0], ("A".to_string(), 9));
        s.set_display(None).unwrap();
        assert!(s.sidebar_lines().is_none());
    }

    #[test]
    fn teams_join_move_leave() {
        let mut s = Scoreboard::default();
        assert!(s.add_team("red", "Red").is_ok());
        assert!(s.add_team("red", "").is_err());
        s.add_team("blue", "").unwrap();
        s.join_team("red", "Player").unwrap();
        assert_eq!(s.team_of("Player"), Some("red"));
        // moving teams leaves the old one
        s.join_team("blue", "Player").unwrap();
        assert_eq!(s.team_of("Player"), Some("blue"));
        assert!(s.leave_team("Player"));
        assert!(!s.leave_team("Player"));
        assert!(s.join_team("nope", "Player").is_err());
        assert!(s.remove_team("red"));
        assert_eq!(s.list_teams().len(), 1);
    }
}
