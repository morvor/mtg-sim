//! Rulings batch S05 — earthbend (CR 701.66): "Target land you control becomes a 0/0
//! land creature with haste in addition to its other types. Put N +1/+1 counters on it.
//! When it dies or is exiled, return it to the battlefield tapped under your control."

use crate::r_s01_common::*;
use crate::r_s04_common::*;
use crate::r_s05_common::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// Casts Earthbending Lesson ("Earthbend 4.") targeting `land`; `resolve`: and resolves
/// it.
fn earthbending_lesson(t: &mut TestGame, land: ObjectId, resolve: bool) -> ObjectId {
    supported("Earthbending Lesson");
    // Paid from the mana pool, so no land is tapped for it.
    add_mana(t, P0, ManaType::C, 3);
    add_mana(t, P0, ManaType::G, 1);
    let spell = t.hand(P0, "Earthbending Lesson");
    t.answer_targets(P0, &[Entity::Object(land)]);
    let s = t.cast(P0, spell).go();
    if resolve {
        t.resolve_all();
    }
    s
}

#[test]
fn earthbend_makes_a_0_0_hasty_land_creature_that_returns_when_it_dies() {
    cr!("701.66a");
    ruling!(
        "Badgermole Cub",
        "\"Earthbend N\" means \"Target land you control becomes a 0/0 land creature with haste in addition to its other types. Put N +1/+1 counters on it. When it dies or is exiled, return it to the battlefield tapped under your control.\""
    );
    supported("Badgermole Cub");
    // Badgermole Cub: "When this creature enters, earthbend 1."
    let mut t = TestGame::new(2);
    let land = t.battlefield_sick(P0, "Wastes");
    t.answer_targets(P0, &[Entity::Object(land)]);
    enter(&mut t, P0, "Badgermole Cub");
    t.resolve_all();
    let o = t.obj_now(land);
    assert!(o.chars.is(CardType::Land) && o.chars.is(CardType::Creature));
    assert!(o.chars.has_keyword(KeywordKind::Haste));
    assert_eq!(t.counters(land, counters::PLUS1), 1);
    assert_eq!(t.pt(land), (1, 1));
    // It came under P0's control this turn, but with haste it can attack.
    t.set_step(P0, mtg_engine::turn::Step::BeginningOfCombat);
    assert!(crate::r_s02_common::can_attack(&mut t, land));
    // It dies: it returns to the battlefield tapped under P0's control.
    crate::r_s02_common::destroy(&mut t, land);
    t.resolve_all();
    let back = t.g.current(land);
    assert_eq!(t.zone(back), Zone::Battlefield);
    let o = t.obj_now(back);
    assert!(o.tapped && o.controller == P0);
    assert!(!o.chars.is(CardType::Creature));
}

#[test]
fn an_earthbent_land_keeps_its_types_and_mana_abilities() {
    cr!("701.66a", "305.6", "205.1b");
    ruling!(
        "Earthbending Lesson",
        "The land will retain any other types, subtypes, or supertypes it previously had. It will also retain any mana abilities it had as a result of those subtypes. For example, a Forest that's turned into a creature this way can still be tapped for {G}."
    );
    let mut t = TestGame::new(2);
    let forest = t.battlefield_sick(P0, "Forest");
    earthbending_lesson(&mut t, forest, true);
    let o = t.obj_now(forest);
    assert!(o.chars.is(CardType::Land) && o.chars.is(CardType::Creature));
    assert!(o.chars.has_subtype("Forest"));
    assert!(o.chars.supertypes.contains(Supertype::Basic));
    assert_eq!(t.pt(forest), (4, 4));
    // It has haste, so it can be tapped for {G} right away.
    let before = t.g.player(P0).mana_pool.count(ManaType::G);
    t.activate(P0, forest, 0, &[]).unwrap();
    assert_eq!(t.g.player(P0).mana_pool.count(ManaType::G), before + 1);
    assert!(t.obj_now(forest).tapped);
}

#[test]
fn earthbend_doesnt_give_the_land_a_color() {
    cr!("701.66a", "105.2c");
    ruling!(
        "Earthbending Lesson",
        "Earthbend doesn't give the land you control a color. As most lands are colorless, in most cases the resulting land creature will also be colorless."
    );
    let mut t = TestGame::new(2);
    let forest = t.battlefield(P0, "Forest");
    earthbending_lesson(&mut t, forest, true);
    assert!(t.obj_now(forest).chars.is(CardType::Creature));
    assert!(colorless(&t, forest));
}

#[test]
fn earthbend_does_nothing_if_its_target_became_illegal() {
    cr!("701.66a", "608.2b");
    ruling!(
        "Earthbending Lesson",
        "If the targeted land becomes an illegal target before the spell or ability that includes earthbend resolves, earthbend does nothing. If the spell or ability didn't have other targets, it won't resolve."
    );
    let mut t = TestGame::new(2);
    let land = t.battlefield(P0, "Wastes");
    earthbending_lesson(&mut t, land, false);
    // The land is returned to P0's hand in response.
    move_to(&mut t, land, Zone::Hand(P0));
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Earthbending Lesson"));
    let card = t.g.current(land);
    assert_eq!(t.zone(card), Zone::Hand(P0));
    assert!(!t.obj(card).chars.is(CardType::Creature));
    assert_eq!(t.obj(card).counter(counters::PLUS1), 0);
    assert!(t.g.kwa.earthbent.is_empty());
}
