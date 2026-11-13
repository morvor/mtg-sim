//! Rulings batch S26 — "becomes a copy of [object], except it has this ability" (CR 707.9a,
//! 707.4): the copy effect lasts indefinitely and keeps the ability, so the permanent can
//! become a copy of something else later; each new copy has new instances of the copied
//! abilities (CR 602.5b).

use crate::r_s01_common::supported;
use crate::r_s06_common::activate_containing;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn artisan_of_forms_copy_lasts_until_it_copies_something_else() {
    cr!("707.9a", "707.4", "611.2a");
    ruling!(
        "Artisan of Forms",
        "The copy effect lasts indefinitely. Often, it will last until it is overwritten by another copy effect (if it copies another creature on a future turn, perhaps)."
    );
    supported("Artisan of Forms");
    // "Heroic — Whenever you cast a spell that targets this creature, you may have this
    // creature become a copy of target creature, except it has this ability."
    let mut t = TestGame::new(2);
    let artisan = t.battlefield(P0, "Artisan of Forms");
    let angel = t.battlefield(P1, "Serra Angel");
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Forest", 2);
    let growth = t.hand(P0, "Giant Growth");
    t.cast(P0, growth).target(artisan).go();
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(angel)]);
    t.resolve_all();
    assert_eq!(t.obj_now(artisan).chars.name, "Serra Angel");
    // It lasts into later turns, and it still has the heroic ability.
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    assert_eq!(t.obj_now(artisan).chars.name, "Serra Angel");
    assert_eq!(t.pt(artisan), (4, 4));
    // On a later turn, another copy effect overwrites it.
    let growth = t.hand(P0, "Giant Growth");
    t.cast(P0, growth).target(artisan).go();
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(giant)]);
    t.resolve_all();
    assert_eq!(t.obj_now(artisan).chars.name, "Hill Giant");
    assert!(!t.obj_now(artisan).has_keyword(KeywordKind::Flying));
}

#[test]
fn copying_a_card_again_gives_a_new_once_each_turn_ability() {
    cr!("602.5b", "707.4", "707.9a");
    ruling!(
        "Likeness Looter",
        "If the copied card has an ability that can be activated only once each turn, copying that card a second time will allow you to activate the new instance of that ability."
    );
    supported("Likeness Looter");
    supported("Putrid Leech");
    // Likeness Looter: "{X}: This creature becomes a copy of target creature card in your
    // graveyard with mana value X, except it has flying and this ability. Activate only
    // as a sorcery." Putrid Leech: "Pay 2 life: This creature gets +2/+2 until end of
    // turn. Activate only once each turn."
    let mut t = TestGame::new(2);
    let looter = t.battlefield(P0, "Likeness Looter");
    let leech = t.graveyard(P0, "Putrid Leech");
    t.lands(P0, "Swamp", 4);
    let become_leech = |t: &mut TestGame| {
        t.answer(P0, DecisionKind::X, Answer::Number(2));
        t.answer_targets(P0, &[Entity::Object(leech)]);
        activate_containing(t, P0, looter, "becomes a copy").expect("copy ability");
        t.resolve_all();
        assert_eq!(t.obj_now(looter).chars.name, "Putrid Leech");
    };
    become_leech(&mut t);
    assert!(t.obj_now(looter).has_keyword(KeywordKind::Flying));
    // Pay 2 life: +2/+2, once.
    activate_containing(&mut t, P0, looter, "Pay 2 life").expect("pump");
    t.resolve_all();
    // Putrid Leech is a 2/2.
    assert_eq!(t.pt(looter), (4, 4));
    assert!(activate_containing(&mut t, P0, looter, "Pay 2 life").is_err());
    // Copying the card again: the new instance can be activated this turn.
    become_leech(&mut t);
    activate_containing(&mut t, P0, looter, "Pay 2 life").expect("new instance");
    t.resolve_all();
    assert_eq!(t.pt(looter), (6, 6));
    assert_eq!(t.life(P0), 16);
}
