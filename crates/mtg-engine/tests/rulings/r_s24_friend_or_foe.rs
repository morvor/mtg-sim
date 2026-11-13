//! Rulings batch S24 — "For each player, choose friend or foe." (Battlebond): each friend
//! and each foe performs what the spell instructs, in APNAP order (CR 101.4), if they can.

use crate::r_s01_common::{give_mana_for, supported};
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::*;

/// P0 casts Pir's Whim ("For each player, choose friend or foe. Each friend searches
/// their library for a land card, puts it onto the battlefield tapped, then shuffles.
/// Each foe sacrifices an artifact or enchantment of their choice."), calling P0 and P1
/// friends or foes as given (options: 0 friend, 1 foe).
fn pirs_whim(t: &mut TestGame, p0: usize, p1: usize) {
    supported("Pir's Whim");
    give_mana_for(t, P0, "Pir's Whim");
    t.answer(P0, DecisionKind::Option, Answer::Index(p0));
    t.answer(P0, DecisionKind::Option, Answer::Index(p1));
    let whim = t.hand(P0, "Pir's Whim");
    t.cast(P0, whim).go();
    t.resolve_all();
}

#[test]
fn a_player_may_be_called_friend_or_foe_even_if_they_cant_do_what_is_instructed() {
    cr!("101.4", "701.23a");
    ruling!(
        "Pir's Whim",
        "You may call a player a friend or a foe even if that player will be instructed to perform an impossible action."
    );
    // P0 is a friend with a Forest in their library; P1, a foe with no artifact or
    // enchantment to sacrifice.
    let mut t = TestGame::new(2);
    let forest = t.library_top(P0, "Forest");
    t.answer_choose(P0, &[Entity::Object(forest)]);
    let from = t.asked().len();
    pirs_whim(&mut t, 0, 1);
    let prompts: Vec<(PlayerId, String, Vec<String>)> = t.asked()[from..]
        .iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseOption {
                prompt, options, ..
            } if prompt.contains("friend or foe") => Some((*p, prompt.clone(), options.clone())),
            _ => None,
        })
        .collect();
    // P0 was asked about each player, and could call each a friend or a foe.
    assert_eq!(prompts.len(), 2);
    assert!(prompts.iter().all(|(p, _, o)| *p == P0 && o.len() == 2));
    assert!(t.on_battlefield(forest));
    assert!(t.obj_now(forest).tapped);
    assert_eq!(t.obj_now(forest).controller, P0);
    // The other way around: P1, a friend, has no land card in their library; P0, a foe,
    // has nothing to sacrifice. Nothing happens to either, and P1's artifact stays.
    let mut t = TestGame::new(2);
    let ring = t.battlefield(P1, "Sol Ring");
    pirs_whim(&mut t, 1, 0);
    assert!(t.g.permanents().all(|o| o.controller != P1 || o.id == ring));
    assert!(t.on_battlefield(ring));
    // Called a foe, P1 sacrifices it.
    let mut t = TestGame::new(2);
    let ring = t.battlefield(P1, "Sol Ring");
    pirs_whim(&mut t, 0, 1);
    assert!(!t.on_battlefield(ring));
    assert!(t.in_graveyard(P1, "Sol Ring"));
}
