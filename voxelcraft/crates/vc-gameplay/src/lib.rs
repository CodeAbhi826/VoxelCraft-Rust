//! vc-gameplay — gameplay systems (§27/§29): crafting, furnaces,
//! brewing, enchanting, villagers (trade tables, wander AI).

pub mod anvil;
pub mod beacon;
pub mod bees;
pub mod brewing;
pub mod campfire;
/// 3.5a: chat log (capped buffer + TTL; HUD/input owned by game.rs)
pub mod chat;
pub mod combat;
/// 3.6a: command line parser (split + table; handlers in game.rs)
pub mod command;
pub mod craft;
pub mod dragon;
pub mod effects;
pub mod enchanting;
pub mod entity_model;
pub mod fishing;
pub mod furnace;
pub mod grindstone;
pub mod hunger;
/// 3.4a: lang-key registry + lookup (`block/item.voxelcraft.<snake>`)
pub mod lang;
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
