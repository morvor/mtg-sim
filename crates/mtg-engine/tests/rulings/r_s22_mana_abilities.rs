//! Rulings batch S22 — an activated ability that adds mana and draws a card (Mossfire
//! Egg) isn't a mana ability under the current rules (CR 605.1a); the choice of friend or
//! foe (Pir's Whim) lasts only for its spell.

use crate::r_s01_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn an_egg_that_draws_a_card_is_not_a_mana_ability() {
    cr!("605.1a", "605.5", "602.2");
    // The Eggs' ruling "This is a mana ability, which means it can be activated as part of
    // the process of casting a spell" predates the current CR 605.1a: an ability whose
    // effect moves a card from a library (draws) isn't a mana ability, and the Eggs'
    // Oracle text now reads "(Activate only as an instant.)" (the ruling is exempt, see
    // docs/rulings-exemptions/rulings-S22.tsv). Mossfire Egg: "{2}, {T}, Sacrifice this
    // artifact: Add {R}{G}. Draw a card."
    supported("Mossfire Egg");
    let mut t = TestGame::new(2);
    let egg = t.battlefield(P0, "Mossfire Egg");
    assert!(!t.obj_now(egg).chars.abilities[0].is_mana_ability());
    t.lands(P0, "Wastes", 2);
    let hand = t.hand_size(P0);
    t.activate(P0, egg, 0, &[]).expect("activate Mossfire Egg");
    // It uses the stack: no mana and no card until it resolves.
    assert_eq!(t.stack_len(), 1);
    assert_eq!(t.g.player(P0).mana_pool.total(), 0);
    assert_eq!(t.hand_size(P0), hand);
    t.resolve();
    assert_eq!(t.hand_size(P0), hand + 1);
    let pool = &t.g.player(P0).mana_pool;
    assert_eq!(
        (pool.count(ManaType::R), pool.count(ManaType::G)),
        (1, 1)
    );
}

/// P0 casts Pir's Whim ("For each player, choose friend or foe. Each friend searches
/// their library for a land card, puts it onto the battlefield tapped, then shuffles.
/// Each foe sacrifices an artifact or enchantment of their choice."), calling P0 and P1
/// friends or foes as given (options: 0 friend, 1 foe).
fn pirs_whim(t: &mut TestGame, p0: usize, p1: usize) {
    give_mana_for(t, P0, "Pir's Whim");
    t.answer(P0, DecisionKind::Option, Answer::Index(p0));
    t.answer(P0, DecisionKind::Option, Answer::Index(p1));
    let whim = t.hand(P0, "Pir's Whim");
    t.cast(P0, whim).go();
    t.resolve_all();
    t.clear_answers();
}

#[test]
fn friend_or_foe_is_chosen_anew_for_each_spell() {
    cr!("701.23a", "102.2");
    ruling!(
        "Pir's Whim",
        "The designation of friend or foe is only relevant to the spell that asks you to choose. A player you call your friend doesn’t become your teammate, and the next “friend or foe” spell you cast could name that player your foe."
    );
    supported("Pir's Whim");
    let mut t = TestGame::new(2);
    let ring = t.battlefield(P1, "Sol Ring");
    let forest = t.library_top(P1, "Forest");
    // P1 is called a friend: they search for a land; their Sol Ring stays.
    t.answer_choose(P1, &[Entity::Object(forest)]);
    pirs_whim(&mut t, 0, 0);
    assert!(t.on_battlefield(forest));
    assert_eq!(t.obj_now(forest).controller, P1);
    assert!(t.on_battlefield(ring));
    // P1 hasn't become P0's teammate.
    assert!(t.g.are_opponents(P0, P1));
    assert!(!t.g.teammates(P0).contains(&P1));
    // The next Pir's Whim calls P1 a foe: they sacrifice Sol Ring.
    pirs_whim(&mut t, 0, 1);
    assert!(!t.on_battlefield(ring));
    assert!(t.in_graveyard(P1, "Sol Ring"));
}
