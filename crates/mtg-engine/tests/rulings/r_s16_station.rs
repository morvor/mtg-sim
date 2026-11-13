//! Rulings on station (CR 702.184) and station cards (CR 721): what a station symbol
//! means, copies of station permanents, and attacking once it becomes a creature.

use crate::r_s01_common::*;
use crate::r_s02_common::*;
use crate::r_s16_common::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::types::counters;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Atmospheric Greenhouse ({4}{G} Artifact — Spacecraft): "When this Spacecraft enters,
/// put a +1/+1 counter on each creature you control. Station. 8+ | Flying, trample" with a
/// 5/4 box in that striation.
const GREENHOUSE: &str = "Atmospheric Greenhouse";

fn is_creature(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).chars.is(CardType::Creature)
}

fn has(t: &TestGame, id: ObjectId, kw: KeywordKind) -> bool {
    t.obj_now(id).chars.has_keyword(kw)
}

#[test]
fn a_station_symbol_grants_its_striation_at_n_or_more_charge_counters() {
    cr!("721.2", "721.2a", "721.2b");
    ruling!(
        "Atmospheric Greenhouse",
        "Each station symbol represents an ability. A station symbol means “As long as this permanent has N or more charge counters on it, it has [abilities],” where N is the number inside the symbol and [abilities] are all the abilities found inside the same striation as that symbol."
    );
    supported(GREENHOUSE);
    let mut t = TestGame::new(2);
    let ship = t.battlefield(P0, GREENHOUSE);
    add_charge(&mut t, ship, 7);
    assert!(!is_creature(&t, ship));
    assert!(!has(&t, ship, KeywordKind::Flying) && !has(&t, ship, KeywordKind::Trample));
    // The eighth counter: flying and trample, and the 5/4 creature of that striation.
    add_charge(&mut t, ship, 1);
    assert!(is_creature(&t, ship));
    assert!(has(&t, ship, KeywordKind::Flying) && has(&t, ship, KeywordKind::Trample));
    assert_eq!(t.pt(ship), (5, 4));
    // It still has the abilities outside the striations.
    assert!(has(&t, ship, KeywordKind::Station));
}

#[test]
fn a_copy_of_a_station_permanent_uses_its_own_charge_counters() {
    cr!("707.2", "721.2a");
    ruling!(
        "Atmospheric Greenhouse",
        "If another permanent becomes a copy of a permanent represented by a station card, all of its printed abilities, including the ones represented by station symbols, are copied. Its current characteristics and the number of charge counters on it are not copied."
    );
    supported("Mirrormade");
    let mut t = TestGame::new(2);
    let ship = t.battlefield(P0, GREENHOUSE);
    add_charge(&mut t, ship, 8);
    assert!(is_creature(&t, ship));
    // Mirrormade: "You may have this enchantment enter as a copy of an artifact or
    // enchantment on the battlefield."
    t.lands(P0, "Island", 3);
    let mirror = t.hand(P0, "Mirrormade");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(ship)]);
    t.cast(P0, mirror).go();
    t.resolve_all();
    let copy = t.g.current(mirror);
    assert!(t.on_battlefield(copy));
    assert_eq!(t.obj(copy).chars.name, GREENHOUSE);
    // No charge counters on the copy: not a creature, no flying.
    assert_eq!(t.counters(copy, counters::CHARGE), 0);
    assert!(!is_creature(&t, copy));
    assert!(!has(&t, copy, KeywordKind::Flying));
    assert!(t.obj(copy).chars.is(CardType::Artifact));
    // Its station symbol was copied: with eight charge counters it's a 5/4 flier.
    add_charge(&mut t, copy, 8);
    assert!(is_creature(&t, copy));
    assert!(has(&t, copy, KeywordKind::Flying) && has(&t, copy, KeywordKind::Trample));
    assert_eq!(t.pt(copy), (5, 4));
}

#[test]
fn a_station_permanent_that_becomes_a_creature_can_attack_if_controlled_since_the_turn_began()
{
    cr!("508.1a", "721.2b");
    ruling!(
        "Atmospheric Greenhouse",
        "If a permanent with station becomes a creature, it will be able to attack if it’s been under your control continuously since the turn began. That is, it doesn’t matter how long it’s been a creature, just how long it’s been on the battlefield."
    );
    let mut t = TestGame::new(2);
    // One Greenhouse was on the battlefield when the turn began; the other entered this
    // turn. Both become creatures now.
    let old = t.battlefield(P0, GREENHOUSE);
    let new = t.battlefield_sick(P0, GREENHOUSE);
    add_charge(&mut t, old, 8);
    add_charge(&mut t, new, 8);
    assert!(is_creature(&t, old) && is_creature(&t, new));
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(can_attack(&mut t, old));
    assert!(!can_attack(&mut t, new));
}
