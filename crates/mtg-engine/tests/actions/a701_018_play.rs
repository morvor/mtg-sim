//! CR 701.18: play.

use crate::a701_common::*;
use mtg_engine::object::Zone;
use mtg_engine::reveal::is_revealed;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn lands_played(t: &TestGame, p: PlayerId) -> u32 {
    t.g.history.lands_played.get(&p).copied().unwrap_or(0)
}

#[test]
fn playing_a_land_is_a_special_action_with_timing_restrictions() {
    cr!("701.18", "701.18a");
    let mut t = TestGame::new(2);
    let forest = t.hand(P0, "Forest");
    // Not during an opponent's turn, not outside a main phase, not with a nonempty
    // stack.
    t.set_step(P1, Step::PrecombatMain);
    assert!(t.play_land(P0, forest).is_err());
    t.set_step(P0, Step::Upkeep);
    assert!(t.play_land(P0, forest).is_err());
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    assert!(t.play_land(P0, forest).is_err());
    t.resolve();
    // Then it's played: straight onto the battlefield, without using the stack.
    t.play_land(P0, forest).unwrap();
    assert_eq!(t.stack_len(), 0);
    assert!(t.on_battlefield(t.g.current(forest)));
    assert_eq!(lands_played(&t, P0), 1);
    // Only one land per turn.
    let island = t.hand(P0, "Island");
    assert!(t.play_land(P0, island).is_err());
}

#[test]
fn putting_a_land_onto_the_battlefield_isnt_playing_it() {
    cr!("701.18a");
    supported("Rampant Growth");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 2);
    t.library_top(P0, "Forest");
    let growth = t.hand(P0, "Rampant Growth");
    t.cast(P0, growth).go();
    t.resolve();
    assert_eq!(t.named_on_battlefield("Forest").len(), 3);
    assert_eq!(lands_played(&t, P0), 0);
    // The land play is still available.
    let island = t.hand(P0, "Island");
    t.play_land(P0, island).unwrap();
    assert_eq!(lands_played(&t, P0), 1);
}

#[test]
fn playing_a_card_plays_a_land_or_casts_a_spell() {
    cr!("701.18b");
    supported("Light Up the Stage");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 4);
    t.library_top(P0, "Lightning Bolt");
    t.library_top(P0, "Forest");
    // "Exile the top two cards of your library. Until the end of your next turn, you may
    // play those cards."
    let stage = t.hand(P0, "Light Up the Stage");
    t.cast(P0, stage).go();
    t.resolve();
    let forest = t.g.find_in_zone(Zone::Exile, "Forest")[0];
    let bolt = t.g.find_in_zone(Zone::Exile, "Lightning Bolt")[0];
    // Playing the land card is playing a land: it uses the land play.
    t.play_land(P0, forest).unwrap();
    assert_eq!(lands_played(&t, P0), 1);
    // Playing the other card is casting it.
    let s = t.cast(P0, bolt).target(P1).go();
    assert!(t.obj(s).is_spell());
    t.resolve();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn play_with_the_top_card_revealed_means_playing_the_game_that_way() {
    cr!("701.18c");
    supported("Courser of Kruphix");
    let mut t = TestGame::new(2);
    let top = t.library_top(P0, "Grizzly Bears");
    // "Play with the top card of your library revealed." isn't an instruction to play
    // that card: it stays on top of the library, revealed.
    t.battlefield(P0, "Courser of Kruphix");
    t.g.recompute();
    t.settle();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.zone(top), Zone::Library(P0));
    assert!(is_revealed(&t.g, t.g.current(top)));
    // As long as the effect lasts: each new top card is revealed.
    t.g.draw_cards(P0, 1);
    t.g.recompute();
    let new_top = *t.g.player(P0).library.last().unwrap();
    assert!(is_revealed(&t.g, new_top));
}
