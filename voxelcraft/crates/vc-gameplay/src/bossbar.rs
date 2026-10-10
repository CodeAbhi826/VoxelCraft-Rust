//! 3.7d: custom boss bars — id-keyed bars with value/max/visible.
//! Subset: no players/color/style options (single viewer, one style —
//! disclosed). Rendering reuses the dragon-bar look with a custom
//! label (`UiCanvas::custom_bar`).

use std::collections::BTreeMap;

#[derive(Debug)]
pub struct BossBar {
    pub name: String,
    pub value: i32,
    pub max: i32,
    pub visible: bool,
}

impl BossBar {
    pub fn frac(&self) -> f32 {
        if self.max <= 0 {
            return 0.0;
        }
        (self.value as f32 / self.max as f32).clamp(0.0, 1.0)
    }
}

#[derive(Default, Debug)]
pub struct BossBars {
    bars: BTreeMap<String, BossBar>,
}

impl BossBars {
    /// create a bar (default 0/100, visible); Err when the id exists.
    pub fn add(&mut self, id: &str, name: &str) -> Result<(), String> {
        if id.is_empty() || id.len() > 64 {
            return Err("Bar id must be 1..=64 chars".to_string());
        }
        if self.bars.contains_key(id) {
            return Err(format!("Bar {id} already exists"));
        }
        self.bars.insert(
            id.to_string(),
            BossBar {
                name: if name.is_empty() {
                    id.to_string()
                } else {
                    name.to_string()
                },
                value: 0,
                max: 100,
                visible: true,
            },
        );
        Ok(())
    }

    pub fn remove(&mut self, id: &str) -> bool {
        self.bars.remove(id).is_some()
    }

    pub fn get(&self, id: &str) -> Option<&BossBar> {
        self.bars.get(id)
    }

    pub fn list(&self) -> Vec<(&str, &BossBar)> {
        self.bars.iter().map(|(k, v)| (k.as_str(), v)).collect()
    }

    fn get_mut(&mut self, id: &str) -> Result<&mut BossBar, String> {
        self.bars
            .get_mut(id)
            .ok_or_else(|| format!("Unknown bar: {id}"))
    }

    pub fn set_value(&mut self, id: &str, value: i32) -> Result<(), String> {
        self.get_mut(id)?.value = value.max(0);
        Ok(())
    }

    pub fn set_max(&mut self, id: &str, max: i32) -> Result<(), String> {
        let b = self.get_mut(id)?;
        b.max = max.max(1);
        Ok(())
    }

    pub fn set_visible(&mut self, id: &str, visible: bool) -> Result<(), String> {
        self.get_mut(id)?.visible = visible;
        Ok(())
    }

    pub fn set_name(&mut self, id: &str, name: &str) -> Result<(), String> {
        self.get_mut(id)?.name = name.to_string();
        Ok(())
    }

    /// visible bars in id order (HUD stacking).
    pub fn visible(&self) -> Vec<(&str, &BossBar)> {
        self.bars
            .iter()
            .filter(|(_, b)| b.visible)
            .map(|(k, v)| (k.as_str(), v))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bars_add_set_render_order() {
        let mut b = BossBars::default();
        assert!(b.add("hp", "HP").is_ok());
        assert!(b.add("hp", "").is_err());
        b.set_value("hp", 30).unwrap();
        b.set_max("hp", 60).unwrap();
        assert!((b.get("hp").unwrap().frac() - 0.5).abs() < 1e-6);
        b.set_visible("hp", false).unwrap();
        assert!(b.visible().is_empty());
        b.set_name("hp", "Health").unwrap();
        assert_eq!(b.get("hp").unwrap().name, "Health");
        assert!(b.set_value("nope", 1).is_err());
        assert!(b.remove("hp"));
    }
}
