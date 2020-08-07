use super::{
    random_table::RandomTable, AreaOfEffect, BlocksTile, CombatStats, Confusion, Consumable,
    InflictsDamage, Item, Monster, Name, Player, Position, ProvidesHealing, Ranged, Rect,
    Renderable, SerializeMe, Viewshed, MAPWIDTH,
};
use crate::{DefenseBonus, EquipmentSlot, Equippable, MeleePowerBonus};
use rltk::{RandomNumberGenerator, RGB};
use specs::prelude::*;
use specs::saveload::{MarkedBuilder, SimpleMarker};
use std::collections::HashMap;

const MAX_MONSTERS: i32 = 4;
const MAX_ITEMS: i32 = 2;

/// Spawns the player and returns their entity object.
pub fn player(ecs: &mut World, player_x: i32, player_y: i32) -> Entity {
    ecs.create_entity()
        .with(Position {
            x: player_x,
            y: player_y,
        })
        .with(Renderable {
            glyph: 3,
            fg: RGB::named(rltk::WHITE),
            bg: RGB::named(rltk::BLACK),
            render_order: 0,
        })
        .with(Player {})
        .with(Viewshed {
            visible_tiles: Vec::new(),
            range: 8,
            dirty: true,
        })
        .with(Name {
            name: "Player".to_string(),
        })
        .with(CombatStats {
            max_hp: 30,
            hp: 30,
            defense: 2,
            power: 5,
        })
        .marked::<SimpleMarker<SerializeMe>>()
        .build()
}

fn room_table(map_depth: i32) -> RandomTable {
    RandomTable::new()
        .add("Broccoli", 10)
        .add("Potato", 1 + map_depth)
        .add("Ketchup", 7)
        .add("Fireball Scroll", 2 + (map_depth / 2))
        .add("Food Coma Scroll", 2 + (map_depth / 2))
        .add("Meat Beam Scroll", 4)
        .add("Spatula", 3)
        .add("Fork", map_depth - 1)
        .add("Bread Knife", map_depth - 2)
        .add("Shield", 3)
        .add("Tower Shield", map_depth - 1)
}

#[allow(clippy::map_entry)]
pub fn spawn_room(ecs: &mut World, room: &Rect, map_depth: i32) {
    let spawn_table = room_table(map_depth);
    let mut spawn_points: HashMap<usize, String> = HashMap::new();

    // Scope to keep the borrow checker happy
    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        let num_spawns = rng.roll_dice(1, MAX_MONSTERS + 3) - 3 + (map_depth - 1);

        for _i in 0..num_spawns {
            let mut added = false;
            let mut tries = 0;
            while !added && tries < 20 {
                let x = (room.x1 + rng.roll_dice(1, i32::abs(room.x2 - room.x1))) as usize;
                let y = (room.y1 + rng.roll_dice(1, i32::abs(room.y2 - room.y1))) as usize;
                let idx = (y * MAPWIDTH) + x;
                if !spawn_points.contains_key(&idx) {
                    spawn_points.insert(idx, spawn_table.roll(&mut rng));
                    added = true;
                } else {
                    tries += 1;
                }
            }
        }
    }

    for spawn in spawn_points.iter() {
        let x = (*spawn.0 % MAPWIDTH) as i32;
        let y = (*spawn.0 / MAPWIDTH) as i32;

        match spawn.1.as_ref() {
            "Potato" => potato(ecs, x, y),
            "Broccoli" => broccoli(ecs, x, y),
            "Ketchup" => ketchup(ecs, x, y),
            "Fireball Scroll" => fireball_scroll(ecs, x, y),
            "Food Coma Scroll" => food_coma_scroll(ecs, x, y),
            "Meat Beam Scroll" => meat_beam_scroll(ecs, x, y),
            "Spatula" => spatula(ecs, x, y),
            "Fork" => fork(ecs, x, y),
            "Bread Knife" => bread_knife(ecs, x, y),
            "Shield" => shield(ecs, x, y),
            "Tower Shield" => tower_shield(ecs, x, y),
            _ => {}
        }
    }
}

fn broccoli(ecs: &mut World, x: i32, y: i32) {
    monster(ecs, x, y, 4, "Broccoli");
}

fn potato(ecs: &mut World, x: i32, y: i32) {
    monster(ecs, x, y, 9, "Potato");
}

fn monster<S: ToString>(ecs: &mut World, x: i32, y: i32, glyph: u16, name: S) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph,
            fg: RGB::named(rltk::WHITE),
            bg: RGB::named(rltk::BLACK),
            render_order: 1,
        })
        .with(Viewshed {
            visible_tiles: Vec::new(),
            range: 8,
            dirty: true,
        })
        .with(Monster {})
        .with(Name {
            name: name.to_string(),
        })
        .with(BlocksTile {})
        .with(CombatStats {
            max_hp: 16,
            hp: 16,
            defense: 1,
            power: 4,
        })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn entity(ecs: &mut World, x: i32, y: i32, name: String, glyph: u16) -> EntityBuilder {
    return ecs
        .create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: glyph,
            fg: RGB::named(rltk::WHITE),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name { name: name })
        .marked::<SimpleMarker<SerializeMe>>();
}

fn item(ecs: &mut World, x: i32, y: i32, name: String, glyph: u16) -> EntityBuilder {
    return entity(ecs, x, y, name, glyph)
        .with(Item {})
        .marked::<SimpleMarker<SerializeMe>>();
}

fn ketchup(ecs: &mut World, x: i32, y: i32) {
    item(ecs, x, y, "Ketchup".to_string(), 5)
        .with(Consumable {})
        .with(ProvidesHealing { heal_amount: 8 })
        .build();
}

fn scroll(ecs: &mut World, x: i32, y: i32, name: String, glyph: u16) -> EntityBuilder {
    return item(ecs, x, y, name, glyph).with(Consumable {});
}

fn meat_beam_scroll(ecs: &mut World, x: i32, y: i32) {
    scroll(ecs, x, y, "Meat Beam Scroll".to_string(), 7)
        .with(Ranged { range: 6 })
        .with(InflictsDamage { damage: 8 })
        .build();
}

fn fireball_scroll(ecs: &mut World, x: i32, y: i32) {
    scroll(ecs, x, y, "Fireball Scroll".to_string(), 6)
        .with(Ranged { range: 6 })
        .with(InflictsDamage { damage: 20 })
        .with(AreaOfEffect { radius: 3 })
        .build();
}

fn food_coma_scroll(ecs: &mut World, x: i32, y: i32) {
    scroll(ecs, x, y, "Food Coma Scroll".to_string(), 8)
        .with(Ranged { range: 6 })
        .with(Confusion { turns: 4 })
        .build();
}

fn melee_weapon(
    ecs: &mut World,
    x: i32,
    y: i32,
    name: String,
    glyph: u16,
    power: i32,
) -> EntityBuilder {
    item(ecs, x, y, name, glyph)
        .with(Equippable {
            slot: EquipmentSlot::Melee,
        })
        .with(MeleePowerBonus { power: power })
        .marked::<SimpleMarker<SerializeMe>>()
}

fn spatula(ecs: &mut World, x: i32, y: i32) {
    melee_weapon(ecs, x, y, "Spatula".to_string(), 1, 1).build();
}

fn fork(ecs: &mut World, x: i32, y: i32) {
    melee_weapon(ecs, x, y, "Fork".to_string(), 2, 2).build();
}

fn bread_knife(ecs: &mut World, x: i32, y: i32) {
    melee_weapon(ecs, x, y, "Bread Knife".to_string(), 0, 4).build();
}

fn shield(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437('('),
            fg: RGB::named(rltk::CYAN),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name {
            name: "Shield".to_string(),
        })
        .with(Item {})
        .with(Equippable {
            slot: EquipmentSlot::Shield,
        })
        .with(DefenseBonus { defense: 1 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn tower_shield(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437('('),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name {
            name: "Tower Shield".to_string(),
        })
        .with(Item {})
        .with(Equippable {
            slot: EquipmentSlot::Shield,
        })
        .with(DefenseBonus { defense: 3 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}
