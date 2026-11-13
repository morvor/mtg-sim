//! Shared helpers for the tests of rulings batch S18 (`r_s18_*.rs`): unearth, unleash,
//! valiant, venture into the dungeon, vigilance, vivid, ward, waterbend, web-slinging,
//! will of the council, and adventurer cards. (The helpers of batches S01–S16 are used
//! too.)

#![allow(dead_code)]

use crate::r_s04_common::activate_named;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

/// Basic lands (Wastes for generic mana) to pay the mana cost written as `cost`
/// ("{3}{R}{R}").
pub fn lands_for(t: &mut TestGame, p: PlayerId, cost: &str) {
    let mut generic = 0usize;
    for sym in cost.split('}').filter_map(|s| s.strip_prefix('{')) {
        let land = match sym {
            "W" => "Plains",
            "U" => "Island",
            "B" => "Swamp",
            "R" => "Mountain",
            "G" => "Forest",
            n => {
                generic += n.parse::<usize>().expect("a generic mana symbol");
                continue;
            }
        };
        t.lands(p, land, 1);
    }
    t.lands(p, "Wastes", generic);
}

/// Activates the unearth ability of `card` (in `p`'s graveyard).
pub fn unearth(
    t: &mut TestGame,
    p: PlayerId,
    card: ObjectId,
) -> Result<Option<ObjectId>, mtg_engine::casting::Illegal> {
    activate_named(t, p, card, "Unearth", 0)
}

/// Puts the real card `name` into `p`'s graveyard with lands for its unearth cost `cost`,
/// unearths it, and resolves the ability (and the abilities that trigger): the permanent
/// it became is returned.
pub fn unearthed(t: &mut TestGame, p: PlayerId, name: &str, cost: &str) -> ObjectId {
    let card = t.graveyard(p, name);
    lands_for(t, p, cost);
    unearth(t, p, card).expect("unearth");
    t.resolve_all();
    let now = t.g.current(card);
    assert_eq!(t.g.obj(now).zone, Zone::Battlefield, "{name} wasn't unearthed");
    now
}

/// Casting an adventurer card as its Adventure.
pub const ADVENTURE: mtg_engine::object::CastMethod = mtg_engine::object::CastMethod::Half(1);

/// Whether the object (followed across zone changes) "has an Adventure" (CR 715.2a).
pub fn has_adventure(t: &mut TestGame, id: ObjectId) -> bool {
    t.g.recompute();
    let id = t.g.current(id);
    t.g.matches(
        id,
        &mtg_engine::ability::Filter::Custom(mtg_engine::adventure::HAS_ADVENTURE.into()),
        &mtg_engine::eval::Ctx::new(None, P0),
    )
}

/// Lands of every basic type, `n` of each, for `p`: mana for anything.
pub fn rainbow_lands(t: &mut TestGame, p: PlayerId, n: usize) {
    for l in ["Plains", "Island", "Swamp", "Mountain", "Forest"] {
        t.lands(p, l, n);
    }
}

/// The objects in exile named `name`.
pub fn exiled_named(t: &TestGame, name: &str) -> Vec<ObjectId> {
    t.g.find_in_zone(Zone::Exile, name)
}
