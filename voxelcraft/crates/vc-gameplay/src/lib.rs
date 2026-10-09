//! vc-gameplay — gameplay systems (§27/§29): crafting, furnaces,
//! brewing, enchanting, villagers (trade tables, wander AI).

pub mod anvil;
pub mod beacon;
pub mod bees;
pub mod brewing;
pub mod campfire;
pub mod combat;
pub mod craft;
pub mod dragon;
pub mod effects;
pub mod enchanting;
pub mod entity_model;
pub mod fishing;
pub mod furnace;
pub mod grindstone;
pub mod hunger;
pub mod mobs;
pub mod modes;
/// Round K (the Nether portal): the frame validation, the portal search,
/// and the far-side build-spot scan (pure over the World; the game layer
/// owns the world edits + the light hooks)
pub mod portal;
pub mod sleep;
pub mod spawners;
/// 3.1b: tool/weapon stats tables (tier speed/damage/durability + the
/// tool-class mapping for the 540..=569 registry window)
pub mod tools;
pub mod villagers;
pub mod weather;
pub mod wither;
