//! The permanent whose ability was countered: "If a permanent's ability is countered this
//! way, activated abilities of that permanent can't be activated this turn." (Trickbind),
//! "If an ability of an artifact, creature, or planeswalker is countered this way, that
//! permanent loses all abilities for as long as ~ remains on the battlefield." (Tishana's
//! Tidebinder).

use crate::combat2_common::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// P0 activates Rod of Ruin ({3}, {T}: 1 damage to any target) at P1.
fn rod_ability(t: &mut TestGame) -> (ObjectId, ObjectId) {
    let rod = t.battlefield(P0, "Rod of Ruin");
    t.lands(P0, "Mountain", 6);
    let ab = t
        .activate(P0, rod, 0, &[Entity::Player(P1)])
        .unwrap()
        .expect("on the stack");
    (ab, rod)
}

#[test]
fn trickbind_stops_the_permanents_activated_abilities_this_turn() {
    cr!("701.6a", "602.5a");
    ruling!(
        "Trickbind",
        "This includes activated abilities other than the one that was countered, as well as any mana abilities that permanent may have."
    );
    supported("Trickbind");
    let mut t = TestGame::new(2);
    let (ab, rod) = rod_ability(&mut t);
    t.lands(P1, "Island", 2);
    let tb = t.hand(P1, "Trickbind");
    t.cast_with(P1, tb, &[Entity::Object(ab)]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    t.g.untap(rod);
    assert!(t.activate(P0, rod, 0, &[Entity::Player(P1)]).is_err());
    // Next turn it can be activated again.
    t.advance_to(P1, Step::Upkeep);
    t.lands(P0, "Mountain", 3);
    assert!(t.activate(P0, rod, 0, &[Entity::Player(P1)]).is_ok());
}

#[test]
fn trickbind_countering_a_trigger_stops_mana_abilities_too() {
    cr!("602.5a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 3);
    t.lands(P1, "Island", 2);
    let prism = t.enter(P0, "Prophetic Prism");
    t.settle();
    let trig = *t.g.stack.last().expect("enters trigger");
    let hand = t.hand_size(P0);
    let tb = t.hand(P1, "Trickbind");
    t.cast_with(P1, tb, &[Entity::Object(trig)]).unwrap();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
    assert!(t.activate(P0, prism, 0, &[]).is_err());
}

#[test]
fn trickbind_countering_a_spell_ability_without_a_permanent_restricts_nothing() {
    cr!("701.6a");
    let mut t = TestGame::new(2);
    let (ab, rod) = rod_ability(&mut t);
    // The Rod leaves before Trickbind resolves: the ability is still countered, and the
    // new Rod (a new object) isn't "that permanent".
    t.lands(P1, "Island", 2);
    let tb = t.hand(P1, "Trickbind");
    t.cast_with(P1, tb, &[Entity::Object(ab)]).unwrap();
    t.g.move_object(rod, Zone::Hand(P0), events::MoveCause::Effect, Some(P0));
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    let rod2 = t.battlefield(P0, "Rod of Ruin");
    assert!(t.activate(P0, rod2, 0, &[Entity::Player(P1)]).is_ok());
}

#[test]
fn tishanas_tidebinder_removes_abilities_while_it_remains() {
    cr!("701.6a", "613.1f");
    ruling!(
        "Tishana's Tidebinder",
        "If the affected permanent gains an ability after Tishana's Tidebinder's effect begins to apply to it, it will keep that ability."
    );
    supported("Tishana's Tidebinder");
    let mut t = TestGame::new(2);
    let (ab, rod) = rod_ability(&mut t);
    t.answer_targets(P1, &[Entity::Object(ab)]);
    let tish = t.enter(P1, "Tishana's Tidebinder");
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    assert!(t.obj_now(rod).chars.abilities.is_empty());
    // Tishana leaves: the Rod has its ability again.
    t.g.destroy(tish, None);
    t.resolve_all();
    assert_eq!(t.obj_now(rod).chars.abilities.len(), 1);
}

#[test]
fn tishanas_tidebinder_checks_the_sources_type_and_needs_to_remain() {
    cr!("701.6a");
    ruling!(
        "Tishana's Tidebinder",
        "the target activated or triggered ability will still be countered, but the source of the ability won't lose its abilities at all"
    );
    ruling!(
        "Tishana's Tidebinder",
        "checks to see if the source of the ability is an artifact, creature, or planeswalker as it resolves"
    );
    // An enchantment's ability: countered, the enchantment keeps its abilities.
    let mut t = TestGame::new(2);
    let pest = t.battlefield(P0, "Pestilence");
    t.lands(P0, "Swamp", 1);
    let ab = t.activate(P0, pest, 0, &[]).unwrap().expect("on the stack");
    t.answer_targets(P1, &[Entity::Object(ab)]);
    t.enter(P1, "Tishana's Tidebinder");
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    assert!(!t.obj_now(pest).chars.abilities.is_empty());
    // Tishana gone before its ability resolves: countered, no ability loss.
    let mut t = TestGame::new(2);
    let (ab, rod) = rod_ability(&mut t);
    t.answer_targets(P1, &[Entity::Object(ab)]);
    let tish = t.enter(P1, "Tishana's Tidebinder");
    t.settle();
    t.g.destroy(tish, None);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.obj_now(rod).chars.abilities.len(), 1);
}
