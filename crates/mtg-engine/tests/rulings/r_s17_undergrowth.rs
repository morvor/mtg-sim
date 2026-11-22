//! Rulings batch S17 — undergrowth ("the number of creature cards in your graveyard"):
//! creature cards of any other types count, tokens never do, and the number is counted as
//! the ability resolves.

use crate::r_s01_common::*;
use crate::r_s02_common::{create_token, destroy};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn artifact_creature_cards_count_for_undergrowth() {
    cr!("205.2a", "404.1");
    ruling!(
        "Lotleth Giant",
        "Creature cards with other types, such as artifact creature cards, count for undergrowth abilities."
    );
    supported("Lotleth Giant");
    // Lotleth Giant: "When this creature enters, it deals 1 damage to target opponent for
    // each creature card in your graveyard."
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Ornithopter");
    t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P0, "Sol Ring");
    t.graveyard(P0, "Lightning Bolt");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.enter(P0, "Lotleth Giant");
    t.resolve_all();
    // Ornithopter (an artifact creature card) and Grizzly Bears: 2.
    assert_eq!(t.life(P1), 18);
}

#[test]
fn tokens_arent_cards_and_never_count_for_undergrowth() {
    cr!("111.1", "111.8", "704.5d");
    ruling!(
        "Izoni, Thousand-Eyed",
        "Because tokens aren't cards, they never count for undergrowth abilities."
    );
    ruling!(
        "Lotleth Giant",
        "Because tokens aren’t cards, they never count for undergrowth abilities."
    );
    supported("Izoni, Thousand-Eyed");
    // Izoni: "When Izoni enters, create a 1/1 black and green Insect creature token for
    // each creature card in your graveyard." A creature token died just before: it isn't
    // counted even while it's in the graveyard (before state-based actions).
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Grizzly Bears");
    let tok = create_token(&mut t, P0, "Bear");
    t.enter(P0, "Izoni, Thousand-Eyed");
    t.g.flush_events();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    let dead = t.g.current(tok);
    let in_yard = t
        .g
        .move_object(dead, Zone::Graveyard(P0), mtg_engine::events::MoveCause::Effect, None)
        .expect("the token didn't move");
    assert_eq!(t.zone(in_yard), Zone::Graveyard(P0));
    assert!(t.obj(in_yard).is_creature());
    let before = with_subtype(&t, P0, "Insect").len();
    t.g.resolve_top();
    t.settle();
    assert_eq!(with_subtype(&t, P0, "Insect").len(), before + 1);
    // The same for Lotleth Giant's damage.
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Grizzly Bears");
    let tok = create_token(&mut t, P0, "Bear");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.enter(P0, "Lotleth Giant");
    t.g.flush_events();
    t.settle();
    let dead = t.g.current(tok);
    t.g.move_object(dead, Zone::Graveyard(P0), mtg_engine::events::MoveCause::Effect, None);
    t.g.resolve_top();
    t.settle();
    assert_eq!(t.life(P1), 19);
    // A token that died earlier isn't in the graveyard at all.
    let mut t = TestGame::new(2);
    let tok = create_token(&mut t, P0, "Bear");
    destroy(&mut t, tok);
    assert_eq!(t.graveyard_size(P0), 0);
}

#[test]
fn x_is_counted_as_the_undergrowth_ability_resolves_and_doesnt_change_later() {
    cr!("608.2h", "611.2c");
    ruling!(
        "Necrotic Wound",
        "The value of X is determined only as the undergrowth ability resolves. If the number of creature cards in your graveyard changes later in the turn, the target creature is unaffected."
    );
    supported("Necrotic Wound");
    // Necrotic Wound: "Target creature gets -X/-X until end of turn, where X is the number
    // of creature cards in your graveyard."
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.graveyard(P0, "Grizzly Bears");
    t.lands(P0, "Swamp", 1);
    let wound = t.hand(P0, "Necrotic Wound");
    t.cast(P0, wound).target(giant).go();
    // In response, another creature card is put into P0's graveyard: it counts.
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.pt(giant), (1, 1));
    // Later creature cards don't change it.
    t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P0, "Grizzly Bears");
    t.g.recompute();
    t.settle();
    assert!(t.on_battlefield(giant));
    assert_eq!(t.pt(giant), (1, 1));
}
