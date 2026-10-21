//! roundtrip-tail-6 (docs/agents/items/roundtrip-tail-6.md): every 3-card round-trip
//! cluster, plus Celestial Dawn's spells you own that another player controls.

use mtg_engine::ability::AbilityKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::Color;
use mtg_engine::*;

/// Activates the first activated ability of `source` whose text starts with `prefix`.
fn activate(t: &mut TestGame, p: PlayerId, source: ObjectId, prefix: &str) {
    t.g.recompute();
    let uid = t
        .g
        .obj(source)
        .chars
        .abilities
        .iter()
        .find(|a| matches!(a.kind, AbilityKind::Activated(_)) && a.text.starts_with(prefix))
        .map(|a| a.uid)
        .expect("no such ability");
    t.g.turn.priority = Some(p);
    t.g.activate_ability(p, source, uid).expect("activation");
    t.g.flush_events();
}

#[test]
fn celestial_dawn_makes_a_spell_you_own_white_while_another_player_controls_it() {
    cr!("112.1", "611.3a");
    // "Nonland permanents you control are white. The same is true for spells you control
    // and nonland cards you own that aren't on the battlefield." A spell is a card on the
    // stack (CR 112.1): P0's card cast by P1 is a card P0 owns that isn't on the
    // battlefield.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    t.battlefield(P0, "Celestial Dawn");
    t.lands(P1, "Mountain", 1);
    let goblin = t.hand(P1, "Raging Goblin");
    t.g.obj_mut(goblin).owner = P0;
    let spell = t.cast(P1, goblin).go();
    t.g.recompute();
    let o = t.obj_now(spell);
    assert_eq!(o.zone, Zone::Stack);
    assert_eq!(o.controller, P1);
    assert!(o.chars.colors.contains(Color::White), "{:?}", o.chars.colors);
    assert!(!o.chars.colors.contains(Color::Red), "{:?}", o.chars.colors);
    // A spell P1 owns stays red.
    t.resolve_all();
    t.lands(P1, "Mountain", 1);
    let other = t.hand(P1, "Raging Goblin");
    let other = t.cast(P1, other).go();
    t.g.recompute();
    assert!(t.obj_now(other).chars.colors.contains(Color::Red));
}

#[test]
fn deafening_silence_counts_a_cast_copy_of_a_card() {
    cr!("112.1b", "707.12", "101.2");
    ruling!(
        "Deafening Silence",
        "Players may cast any number of creature spells plus one noncreature spell each turn."
    );
    // "Each player can't cast more than one noncreature spell each turn." A copy of a card
    // cast with Isochron Scepter is a noncreature spell (CR 112.1b): after it, no other
    // noncreature spell can be cast that turn.
    let mut t = TestGame::new(2);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(bolt)]);
    let scepter = t.enter(P0, "Isochron Scepter");
    t.resolve_all();
    assert_eq!(t.zone(bolt), Zone::Exile);
    t.battlefield(P1, "Deafening Silence");
    t.lands(P0, "Mountain", 3);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    activate(&mut t, P0, scepter, "{2}");
    t.resolve_all();
    assert_eq!(t.life(P1), 17, "{}", t.dump_log());
    let shock = t.hand(P0, "Shock");
    assert!(
        t.cast(P0, shock).target(Entity::Player(P1)).try_go().is_err(),
        "{}",
        t.dump_log()
    );
    assert_eq!(t.zone(shock), Zone::Hand(P0));
}

#[test]
fn deafening_silence_stops_casting_a_copy_after_a_noncreature_spell() {
    cr!("112.1b", "707.12", "101.2");
    // The copy can't be cast after a noncreature spell this turn: it isn't cast (and
    // ceases to exist, CR 707.12, 704.5d).
    let mut t = TestGame::new(2);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(bolt)]);
    let scepter = t.enter(P0, "Isochron Scepter");
    t.resolve_all();
    t.battlefield(P1, "Deafening Silence");
    t.lands(P0, "Mountain", 3);
    let shock = t.hand(P0, "Shock");
    t.cast(P0, shock).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    activate(&mut t, P0, scepter, "{2}");
    t.resolve_all();
    assert_eq!(t.life(P1), 18, "{}", t.dump_log());
}
