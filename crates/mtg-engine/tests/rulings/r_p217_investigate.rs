//! Rulings batch P217 — investigate (CR 701.16): "Create a Clue token."

use crate::r_s01_common::{attack_with, give_mana_for, supported, triggers_on_stack};
use crate::r_s02_common::destroy;
use crate::r_s04_common::add_mana;
use crate::r_s05_common::{colorless, enter, move_to, tokens_with_subtype};
use crate::r_s06_common::activate_containing;
use crate::r_s07_common::resolved;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn clues(t: &TestGame, p: PlayerId) -> usize {
    tokens_with_subtype(t, p, "Clue").len()
}

#[test]
fn investigate_creates_a_colorless_clue_artifact_token_that_draws_a_card() {
    cr!("701.16a", "111.10f");
    ruling!(
        "The Fugitive Doctor",
        "\"Investigate\" means \"Create a Clue token.\" A Clue token is a colorless Clue artifact token with \"{2}, Sacrifice this artifact: Draw a card.\""
    );
    supported("The Fugitive Doctor");
    let mut t = TestGame::new(2);
    enter(&mut t, P0, "The Fugitive Doctor");
    t.resolve_all();
    let clue = tokens_with_subtype(&t, P0, "Clue");
    assert_eq!(clue.len(), 1);
    let c = clue[0];
    assert!(t.obj_now(c).is_token());
    assert!(colorless(&t, c));
    let types: Vec<CardType> = t.obj_now(c).chars.card_types.iter().collect();
    assert_eq!(types, vec![CardType::Artifact]);
    // "{2}, Sacrifice this artifact: Draw a card."
    let hand = t.hand_size(P0);
    add_mana(&mut t, P0, ManaType::C, 2);
    activate_containing(&mut t, P0, c, "Draw a card").expect("activate");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(clues(&t, P0), 0);
}

/// P0 casts `name` ("Tap up to two target creatures ... Investigate.") with `targets`
/// among P1's two Grizzly Bears; if `remove`, they leave the battlefield in response.
/// Returns (whether it resolved, P0's Clues).
fn up_to_two(name: &str, n_targets: usize, remove: bool) -> (bool, usize) {
    supported(name);
    let mut t = TestGame::new(2);
    let bears: Vec<ObjectId> = (0..2).map(|_| t.battlefield(P1, "Grizzly Bears")).collect();
    give_mana_for(&mut t, P0, name);
    let card = t.hand(P0, name);
    let targets: Vec<Entity> = bears[..n_targets].iter().map(|b| Entity::Object(*b)).collect();
    t.answer_targets(P0, &targets);
    let spell = t.cast(P0, card).go();
    if remove {
        for b in &bears[..n_targets] {
            move_to(&mut t, *b, Zone::Hand(P1));
        }
    }
    t.resolve_all();
    (resolved(&t, spell), clues(&t, P0))
}

#[test]
fn expose_evil_with_all_targets_illegal_doesnt_investigate() {
    cr!("608.2b", "701.16a");
    ruling!(
        "Expose Evil",
        "If Expose Evil has at least one target and all of those targets become illegal, the spell doesn't resolve and you won't investigate."
    );
    assert_eq!(up_to_two("Expose Evil", 2, true), (false, 0));
    assert_eq!(up_to_two("Expose Evil", 1, true), (false, 0));
    assert_eq!(up_to_two("Expose Evil", 2, false), (true, 1));
}

#[test]
fn expose_evil_can_be_cast_with_no_targets_to_investigate() {
    cr!("601.2c", "115.1", "701.16a");
    ruling!(
        "Expose Evil",
        "You can cast Expose Evil with no targets if you only want to investigate."
    );
    assert_eq!(up_to_two("Expose Evil", 0, false), (true, 1));
}

#[test]
fn out_cold_needs_no_targets_but_doesnt_investigate_if_all_become_illegal() {
    cr!("601.2c", "608.2b", "701.16a");
    ruling!(
        "Out Cold",
        "You don't have to choose any targets for Out Cold. However, if you do, and all of those creatures are illegal targets at the time Out Cold tries to resolve, it won't resolve and none of its effects will happen. You won't investigate."
    );
    assert_eq!(up_to_two("Out Cold", 0, false), (true, 1));
    assert_eq!(up_to_two("Out Cold", 2, true), (false, 0));
    assert_eq!(up_to_two("Out Cold", 2, false), (true, 1));
}

#[test]
fn meddling_youths_investigates_whatever_happens_to_the_attackers() {
    cr!("508.1m", "603.2", "701.16a");
    ruling!(
        "Meddling Youths",
        "It doesn't matter what happens to the creatures in response. As long as you attacked with at least three creatures, you'll investigate when Meddling Youths's triggered ability resolves."
    );
    supported("Meddling Youths");
    let mut t = TestGame::new(2);
    let youths = t.battlefield(P0, "Meddling Youths");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    attack_with(
        &mut t,
        &[
            (youths, Entity::Player(P1)),
            (a, Entity::Player(P1)),
            (b, Entity::Player(P1)),
        ],
    );
    assert_eq!(triggers_on_stack(&t, "three or more creatures"), 1);
    // All three attackers leave in response.
    destroy(&mut t, a);
    destroy(&mut t, b);
    destroy(&mut t, youths);
    assert!(!t.on_battlefield(youths));
    t.resolve_all();
    assert_eq!(clues(&t, P0), 1);
}

#[test]
fn syndicate_heavy_investigates_once_however_much_life_was_gained() {
    cr!("603.4", "701.16a");
    ruling!(
        "Syndicate Heavy",
        "You investigate just once, no matter how much life you've gained past 4."
    );
    supported("Syndicate Heavy");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Syndicate Heavy");
    t.g.gain_life(P0, 12);
    t.g.flush_events();
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "investigate"), 1);
    t.resolve_all();
    assert_eq!(clues(&t, P0), 1);
}
