//! Rulings batch P207 — enchant land you control (CR 303.4): Crackling Emergence and
//! Harmonious Emergence, "If enchanted land would be destroyed, instead sacrifice this
//! Aura and that land gains indestructible until end of turn." (CR 614.1a, 614.6).

use crate::r_s01_common::*;
use crate::r_s02_common::destroy;
use crate::r_s03_common::in_hand_with_mana;
use crate::r_s06_common::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// P0's Forest enchanted with `name`.
fn emerged(name: &str) -> (TestGame, ObjectId, ObjectId) {
    supported(name);
    let mut t = TestGame::new(2);
    let forest = t.battlefield(P0, "Forest");
    let aura = attach_new(&mut t, P0, name, forest);
    assert!(t.obj_now(forest).is(CardType::Creature));
    (t, forest, aura)
}

/// The replacement applies to a destroy instruction and to lethal damage; afterward the
/// land is indestructible until end of turn.
fn replaces_destruction(name: &str) {
    // Murder.
    let (mut t, forest, aura) = emerged(name);
    let murder = in_hand_with_mana(&mut t, P1, "Murder");
    t.g.turn.priority = Some(P1);
    t.cast(P1, murder).target(forest).go();
    t.resolve_all();
    assert!(t.on_battlefield(forest), "{name}: destroyed");
    assert!(!t.on_battlefield(aura));
    assert!(t.in_graveyard(P0, name));
    assert!(has_kw(&t, forest, KeywordKind::Indestructible));
    // It's no longer a creature, and it's indestructible until end of turn.
    assert!(!t.obj_now(forest).is(CardType::Creature));
    destroy(&mut t, forest);
    assert!(t.on_battlefield(forest));
    t.advance_to(P1, Step::Upkeep);
    assert!(!has_kw(&t, forest, KeywordKind::Indestructible));
    // Lethal damage.
    let (mut t, forest, aura) = emerged(name);
    let giant = t.battlefield(P1, "Craw Wurm");
    damage(&mut t, giant, 6, forest);
    assert!(t.on_battlefield(forest), "{name}: destroyed by lethal damage");
    assert!(!t.on_battlefield(aura));
}

/// Sacrificing the land isn't destroying it: Zuran Orb ("Sacrifice a land: You gain 2
/// life").
fn doesnt_replace_sacrifice(name: &str) {
    supported("Zuran Orb");
    let (mut t, forest, aura) = emerged(name);
    let orb = t.battlefield(P0, "Zuran Orb");
    t.answer_choose(P0, &[Entity::Object(forest)]);
    t.activate(P0, orb, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Forest"), "{name}: the land wasn't sacrificed");
    assert!(!t.on_battlefield(aura));
    assert!(t.in_graveyard(P0, name));
    assert_eq!(t.life(P0), 22);
}

#[test]
fn emergence_auras_replace_any_destruction_of_the_land() {
    cr!("614.1a", "614.6", "701.8a", "704.5g", "702.12b");
    ruling!(
        "Crackling Emergence",
        "Crackling Emergence's replacement effect applies whether the enchanted land would be destroyed due to lethal damage or any other reason."
    );
    ruling!(
        "Harmonious Emergence",
        "Harmonious Emergence's replacement effect applied whether the enchanted land would be destroyed due to lethal damage or any other reason."
    );
    replaces_destruction("Crackling Emergence");
    replaces_destruction("Harmonious Emergence");
}

#[test]
fn emergence_auras_dont_replace_sacrificing_the_land() {
    cr!("701.21a", "614.6");
    ruling!(
        "Crackling Emergence",
        "Crackling Emergence's replacement effect does not replace effects or costs that require you to sacrifice the enchanted land."
    );
    ruling!(
        "Harmonious Emergence",
        "Harmonious Emergence's replacement effect does not replace effects or costs that require you to sacrifice the enchanted land."
    );
    doesnt_replace_sacrifice("Crackling Emergence");
    doesnt_replace_sacrifice("Harmonious Emergence");
}
