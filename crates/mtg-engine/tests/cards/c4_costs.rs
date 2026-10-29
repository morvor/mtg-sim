//! Costs and activation permissions, round 4 (cards4-costs-mana-replacements,
//! `oracle/patterns/costs_5.rs`, `activation_restrictions.rs`): an escape cost exiling any
//! number of cards with four or more card types among them, granted abilities only the
//! granting player may activate, and abilities only the owner may activate.

use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::*;

fn assert_supported(names: &[&str]) {
    for n in names {
        let u = card(n).unsupported_text().join(" | ");
        assert!(u.is_empty(), "{n} has unsupported text: {u}");
    }
}

#[test]
fn nethergoyf_escapes_by_exiling_cards_with_four_card_types_among_them() {
    cr!("702.138a", "205.2a", "601.2h");
    assert_supported(&["Nethergoyf"]);
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 3);
    let goyf = t.graveyard(P0, "Nethergoyf");
    let cards = [
        t.graveyard(P0, "Grizzly Bears"),
        t.graveyard(P0, "Lightning Bolt"),
        t.graveyard(P0, "Forest"),
        t.graveyard(P0, "Ornithopter"),
    ];
    let es: Vec<Entity> = cards.iter().map(|o| Entity::Object(*o)).collect();
    t.answer_choose(P0, &es);
    t.cast(P0, goyf)
        .method(CastMethod::Keyword(KeywordKind::Escape))
        .x(4)
        .go();
    t.resolve_all();
    assert!(t.on_battlefield(t.g.current(goyf)));
    for c in cards {
        assert_eq!(t.zone(t.g.current(c)), Zone::Exile);
    }
    // Three card types only (creature, instant, land): it can't escape.
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 3);
    let goyf = t.graveyard(P0, "Nethergoyf");
    let cards = [
        t.graveyard(P0, "Grizzly Bears"),
        t.graveyard(P0, "Lightning Bolt"),
        t.graveyard(P0, "Forest"),
    ];
    let es: Vec<Entity> = cards.iter().map(|o| Entity::Object(*o)).collect();
    t.answer_choose(P0, &es);
    let r = t
        .cast(P0, goyf)
        .method(CastMethod::Keyword(KeywordKind::Escape))
        .x(3)
        .try_go();
    assert!(r.is_err() || !t.on_battlefield(t.g.current(goyf)));
    t.resolve_all();
    assert!(!t.on_battlefield(t.g.current(goyf)));
}

#[test]
fn martyrdoms_granted_ability_can_be_activated_only_by_its_caster() {
    cr!("602.2", "614.9");
    assert_supported(&["Martyrdom"]);
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 3);
    let spell = t.hand(P0, "Martyrdom");
    t.cast(P0, spell).target(bears).go();
    t.resolve();
    // Another player gains control of the creature: they can't activate it; P0 can.
    t.g.objects[bears.0 as usize].controller = P1;
    t.g.dirty = true;
    t.g.recompute();
    assert!(t.activate(P1, bears, 0, &[Entity::Player(P1)]).is_err());
    t.clear_answers();
    t.activate(P0, bears, 0, &[Entity::Player(P0)]).unwrap();
    t.resolve();
    // The next 1 damage to P0 is dealt to the Bears instead.
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(P0).go();
    t.resolve();
    assert_eq!(t.life(P0), 18);
    assert_eq!(t.g.obj(bears).damage, 1);
}

#[test]
fn personal_incarnation_only_its_owner_activates_and_damage_goes_to_its_owner() {
    cr!("602.2", "614.9");
    assert_supported(&["Personal Incarnation"]);
    let mut t = TestGame::new(2);
    let inc = t.battlefield(P0, "Personal Incarnation");
    t.g.objects[inc.0 as usize].controller = P1;
    t.g.dirty = true;
    t.g.recompute();
    // Its controller (not its owner) can't activate it.
    assert!(t.activate(P1, inc, 0, &[]).is_err());
    t.activate(P0, inc, 0, &[]).unwrap();
    t.resolve();
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(inc).go();
    t.resolve();
    assert_eq!(t.life(P0), 19, "1 damage dealt to its owner");
    assert_eq!(t.g.obj(inc).damage, 2);
}
