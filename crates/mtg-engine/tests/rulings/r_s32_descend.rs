//! Rulings batch S32 — "descended this turn" and "the number of times you descended this
//! turn" (CR 700.11): a permanent card was put into your graveyard from anywhere this
//! turn, whether or not it's still there; "At the beginning of your end step, if you
//! descended this turn" is an intervening-if trigger (CR 603.4).

use crate::r_s01_common::*;
use crate::r_s04_common::graveyard_names;
use crate::r_s05_common::move_to;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Moves a real card from `p`'s hand to their graveyard by discarding it.
fn discard_new(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    let card = t.hand(p, name);
    let new = t.g.discard(p, card, None).expect("discarded");
    t.g.flush_events();
    t.settle();
    new
}

/// Mills the real card `name` (put on top of `p`'s library first).
fn mill_new(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    t.library_top(p, name);
    let milled = t.g.mill(p, 1);
    t.g.flush_events();
    t.settle();
    milled[0]
}

/// Advances to `p`'s end step (its beginning-of-step triggers on the stack) and resolves
/// everything.
fn to_end_step(t: &mut TestGame, p: PlayerId) {
    t.advance_to(p, Step::End);
    t.resolve_all();
}

#[test]
fn descending_means_a_permanent_card_went_to_your_graveyard_from_anywhere() {
    cr!("700.11", "110.4", "603.4");
    ruling!(
        "Enterprising Scallywag",
        "Some cards refer to a player who has \"descended this turn.\" This means that a permanent card has been put into that player's graveyard from anywhere this turn."
    );
    supported("Enterprising Scallywag");
    // "At the beginning of your end step, if you descended this turn, create a Treasure
    // token."
    // Nonpermanent cards, tokens, and cards put into another player's graveyard don't
    // count.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Enterprising Scallywag");
    discard_new(&mut t, P0, "Lightning Bolt");
    mill_new(&mut t, P0, "Divination");
    let token = crate::r_s02_common::create_token(&mut t, P0, "Grizzly Bears");
    move_to(&mut t, token, Zone::Graveyard(P0));
    mill_new(&mut t, P1, "Forest");
    to_end_step(&mut t, P0);
    assert!(with_subtype(&t, P0, "Treasure").is_empty(), "no descent");
    // A land card milled from the library (next turn) counts.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Enterprising Scallywag");
    mill_new(&mut t, P0, "Forest");
    to_end_step(&mut t, P0);
    assert_eq!(
        with_subtype(&t, P0, "Treasure").len(),
        1,
        "milled a land card"
    );
    // So does a creature card discarded from the hand, or an artifact put into the
    // graveyard from the battlefield.
    for how in ["discard", "destroy"] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Enterprising Scallywag");
        match how {
            "discard" => {
                discard_new(&mut t, P0, "Grizzly Bears");
            }
            _ => {
                let stone = t.battlefield(P0, "Mind Stone");
                crate::r_s02_common::destroy(&mut t, stone);
            }
        }
        to_end_step(&mut t, P0);
        assert_eq!(with_subtype(&t, P0, "Treasure").len(), 1, "{how}");
    }
}

#[test]
fn the_number_of_times_you_descended_counts_permanent_cards() {
    cr!("700.11", "110.4", "111.4");
    ruling!(
        "The Mycotyrant",
        "Some cards refer to the number of times a player descended this turn. Those cards care about the number of permanent cards put into that player's graveyard from anywhere this turn."
    );
    supported("The Mycotyrant");
    // "At the beginning of your end step, create X 1/1 black Fungus creature tokens with
    // "This token can't block," where X is the number of times you descended this turn."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "The Mycotyrant");
    // Three permanent cards (a land, a creature, an enchantment) and two that aren't.
    mill_new(&mut t, P0, "Forest");
    discard_new(&mut t, P0, "Grizzly Bears");
    let fervor = t.battlefield(P0, "Fervor");
    crate::r_s02_common::destroy(&mut t, fervor);
    discard_new(&mut t, P0, "Lightning Bolt");
    mill_new(&mut t, P0, "Divination");
    // The opponent's permanent cards count for them, not for P0.
    mill_new(&mut t, P1, "Hill Giant");
    to_end_step(&mut t, P0);
    let fungi = with_subtype(&t, P0, "Fungus");
    assert_eq!(
        fungi.len() - 1,
        3,
        "three Fungus tokens (plus The Mycotyrant)"
    );
    let token = *fungi.iter().find(|id| t.obj_now(**id).is_token()).unwrap();
    assert_eq!(t.pt(token), (1, 1));
    // The tokens have "This token can't block."
    assert!(t
        .obj_now(token)
        .chars
        .abilities
        .iter()
        .any(|a| a.text.contains("can't block")));
}

#[test]
fn the_descended_permanent_card_neednt_have_been_yours_to_control_at_the_time() {
    cr!("700.11", "603.4");
    ruling!(
        "Stalactite Stalker",
        "These cards don't need to have been under your control at the time you descended."
    );
    supported("Stalactite Stalker");
    // "At the beginning of your end step, if you descended this turn, put a +1/+1 counter
    // on this creature." A permanent card goes to P0's graveyard in the first main phase;
    // Stalactite Stalker is cast in the second main phase.
    let mut t = TestGame::new(2);
    mill_new(&mut t, P0, "Forest");
    t.advance_to(P0, Step::PostcombatMain);
    let stalker = crate::r_s25_common::cast_new(&mut t, P0, "Stalactite Stalker", &[]);
    t.resolve_all();
    let stalker = t.g.current(stalker);
    assert!(t.on_battlefield(stalker));
    to_end_step(&mut t, P0);
    assert_eq!(t.counters(stalker, "+1/+1"), 1);
    assert_eq!(t.pt(stalker), (2, 2));
}

#[test]
fn the_cards_neednt_still_be_in_the_graveyard() {
    cr!("700.11");
    ruling!(
        "Enterprising Scallywag",
        "In either case, it doesn't matter if those cards are still in that player's graveyard."
    );
    // Both kinds: "descended this turn" (Enterprising Scallywag) and "the number of times"
    // (The Mycotyrant). The milled cards are exiled from the graveyard before the end
    // step.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Enterprising Scallywag");
    t.battlefield(P0, "The Mycotyrant");
    let forest = mill_new(&mut t, P0, "Forest");
    let bears = mill_new(&mut t, P0, "Grizzly Bears");
    move_to(&mut t, forest, Zone::Exile);
    move_to(&mut t, bears, Zone::Exile);
    assert!(graveyard_names(&t, P0).is_empty());
    to_end_step(&mut t, P0);
    assert_eq!(with_subtype(&t, P0, "Treasure").len(), 1);
    assert_eq!(with_subtype(&t, P0, "Fungus").len() - 1, 2);
}

#[test]
fn the_end_step_trigger_happens_once_and_only_if_you_descended_before_it() {
    cr!("700.11", "603.4", "513.1a");
    ruling!(
        "Enterprising Scallywag",
        "will trigger only once during your end step, no matter how many times you descended this turn. However, if you haven't descended this turn as your end step begins, the ability won't trigger at all."
    );
    // Enterprising Scallywag: descending three times makes one Treasure.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Enterprising Scallywag");
    for name in ["Forest", "Grizzly Bears", "Mind Stone"] {
        mill_new(&mut t, P0, name);
    }
    to_end_step(&mut t, P0);
    assert_eq!(with_subtype(&t, P0, "Treasure").len(), 1);
    // Descending during the end step is too late: the ability didn't trigger as the step
    // began, and it doesn't trigger later.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Enterprising Scallywag");
    t.advance_to(P0, Step::End);
    assert_eq!(t.stack_len(), 0, "didn't trigger");
    mill_new(&mut t, P0, "Forest");
    t.resolve_all();
    t.advance_to(P1, Step::Upkeep);
    assert!(with_subtype(&t, P0, "Treasure").is_empty());
}
