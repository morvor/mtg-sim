//! CR 701.6 (counter) and 701.8 (destroy): which spell or ability countered a spell or
//! destroyed a permanent. Only an effect that says "destroy" destroys (CR 701.8b): not
//! sacrificing, and not the state-based actions for lethal damage (CR 704.5g), which no
//! spell or ability performs. A replacement effect's modified event is caused by what
//! caused the event it replaced (CR 614.6).

use super::r703_common::*;
use mtg_engine::events::Event;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// The (cause, controller) of each Destroyed event of this turn about `obj`.
fn destroyed(t: &TestGame, obj: ObjectId) -> Vec<(Option<ObjectId>, Option<PlayerId>)> {
    t.turn_events
        .iter()
        .filter_map(|e| match e {
            Event::Destroyed { obj: o, cause, by } if *o == obj => Some((*cause, *by)),
            _ => None,
        })
        .collect()
}

/// `p` casts the real spell `name` from hand (with free mana) at `targets`.
fn cast(t: &mut TestGame, p: PlayerId, name: &str, targets: &[Entity]) -> ObjectId {
    let c = t.hand(p, name);
    let cost = mtg_engine::card::card(name)
        .front()
        .chars
        .mana_cost
        .clone()
        .unwrap_or_default();
    for sym in format!("{cost}")
        .split('}')
        .filter_map(|s| s.strip_prefix('{'))
    {
        let land = match sym {
            "W" => "Plains",
            "U" => "Island",
            "B" => "Swamp",
            "R" => "Mountain",
            "G" => "Forest",
            n => {
                for _ in 0..n.parse::<usize>().unwrap_or(1) {
                    t.battlefield(p, "Wastes");
                }
                continue;
            }
        };
        t.battlefield(p, land);
    }
    t.cast_with(p, c, targets)
        .unwrap_or_else(|e| panic!("casting {name}: {e:?}"))
}

#[test]
fn a_destroy_effect_records_its_spell_and_lethal_damage_records_none() {
    cr!("701.8a", "701.8b", "704.5g");
    // "Whenever a spell or ability an opponent controls destroys a creature you control,
    // draw a card."
    let watcher = oracle_card(
        "Grudge Keeper",
        "Enchantment",
        "{0}",
        None,
        "Whenever a spell or ability an opponent controls destroys a creature you control, draw a card.",
    );
    let mut t = TestGame::new(2);
    t.custom(P0, watcher, Zone::Battlefield);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    let c = t.battlefield(P0, "Grizzly Bears");
    let hand = t.hand_size(P0);
    t.set_step(P1, Step::PrecombatMain);
    // Murder destroys: by P1's spell.
    let murder = cast(&mut t, P1, "Murder", &[Entity::Object(a)]);
    t.resolve_all();
    assert_eq!(destroyed(&t, a), vec![(Some(murder), Some(P1))]);
    assert_eq!(t.hand_size(P0), hand + 1);
    // Lightning Bolt's lethal damage: the state-based action destroys it.
    cast(&mut t, P1, "Lightning Bolt", &[Entity::Object(b)]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(destroyed(&t, b), vec![(None, None)]);
    assert_eq!(t.hand_size(P0), hand + 1);
    // P0's own Murder: P0's spell, not an opponent's.
    t.set_step(P0, Step::PrecombatMain);
    let own = cast(&mut t, P0, "Murder", &[Entity::Object(c)]);
    t.resolve_all();
    assert_eq!(destroyed(&t, c), vec![(Some(own), Some(P0))]);
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn a_regenerated_permanent_isnt_destroyed() {
    cr!("701.8c", "701.19a");
    let watcher = oracle_card(
        "Grudge Keeper",
        "Enchantment",
        "{0}",
        None,
        "Whenever a spell or ability an opponent controls destroys a creature you control, draw a card.",
    );
    let mut t = TestGame::new(2);
    t.custom(P0, watcher, Zone::Battlefield);
    let troll = t.battlefield(P0, "Cudgel Troll");
    t.battlefield(P0, "Forest");
    let hand = t.hand_size(P0);
    let regen = t
        .g
        .obj(troll)
        .chars
        .abilities
        .iter()
        .find(|a| a.text.contains("Regenerate"))
        .unwrap()
        .uid;
    t.g.turn.priority = Some(P0);
    t.g.activate_ability(P0, troll, regen).unwrap();
    t.resolve_all();
    t.set_step(P1, Step::PrecombatMain);
    cast(&mut t, P1, "Murder", &[Entity::Object(troll)]);
    t.resolve_all();
    assert!(t.on_battlefield(troll));
    assert!(destroyed(&t, troll).is_empty());
    assert_eq!(t.hand_size(P0), hand);
}

#[test]
fn a_countered_spell_records_what_countered_it() {
    cr!("701.6a");
    // "Whenever a spell or ability you control counters a spell, you gain 3 life."
    let watcher = oracle_card(
        "Counter Keeper",
        "Enchantment",
        "{0}",
        None,
        "Whenever a spell or ability you control counters a spell, you gain 3 life.",
    );
    let mut t = TestGame::new(2);
    t.custom(P0, watcher, Zone::Battlefield);
    t.set_step(P1, Step::PrecombatMain);
    let bolt = cast(&mut t, P1, "Lightning Bolt", &[Entity::Player(P0)]);
    let cancel = cast(&mut t, P0, "Cancel", &[Entity::Object(bolt)]);
    t.resolve_all();
    let countered: Vec<_> = t
        .turn_events
        .iter()
        .filter_map(|e| match e {
            Event::Countered { what, cause, by } => Some((*what, *cause, *by)),
            _ => None,
        })
        .collect();
    assert_eq!(countered, vec![(bolt, Some(cancel), Some(P0))]);
    assert!(t.in_graveyard(P1, "Lightning Bolt"));
    assert_eq!(t.life(P0), 23);
    // P1 countering P0's spell: not "a spell or ability you control".
    t.set_step(P0, Step::PrecombatMain);
    let bolt = cast(&mut t, P0, "Lightning Bolt", &[Entity::Player(P1)]);
    cast(&mut t, P1, "Cancel", &[Entity::Object(bolt)]);
    t.resolve_all();
    assert_eq!(t.life(P0), 23);
}
