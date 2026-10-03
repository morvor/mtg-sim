//! CR 705: flipping a coin.

use crate::r703_common::*;
use mtg_engine::ability::*;
use mtg_engine::decision::Decision;
use mtg_engine::dice::CoinFlip;
use mtg_engine::events::Event;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// The coin flips of this turn: (player, heads, won, lost).
fn flips(t: &TestGame) -> Vec<(PlayerId, bool, bool, bool)> {
    t.turn_events
        .iter()
        .filter_map(|e| match e {
            Event::CoinFlipped {
                player,
                won,
                lost,
                heads,
            } => Some((*player, *heads, *won, *lost)),
            _ => None,
        })
        .collect()
}

fn load(t: &mut TestGame, heads: &[bool]) {
    t.g.dice.loaded_coins.extend(heads.iter().copied());
}

fn calls_asked(t: &TestGame) -> Vec<PlayerId> {
    t.asked()
        .into_iter()
        .filter(|(_, d)| {
            matches!(d, Decision::ChooseOption { prompt, .. } if prompt == "Call the coin flip")
        })
        .map(|(p, _)| p)
        .collect()
}

/// "Flip a coin. If the coin comes up heads, you gain 3 life. If it comes up tails, you
/// lose 1 life."
fn heads_or_tails() -> CardDef {
    oracle_card(
        "Toss",
        "Instant",
        "{0}",
        None,
        "Flip a coin. If the coin comes up heads, you gain 3 life. If it comes up tails, you lose 1 life.",
    )
}

#[test]
fn a_coin_has_two_equally_likely_sides_heads_and_tails() {
    cr!("705.1");
    let mut t = TestGame::new(2);
    let mut spec = CoinFlip::new();
    spec.call = false;
    spec.count = Value::c(400);
    run_effect(&mut t, P0, None, Effect::FlipCoins(Box::new(spec)), &[]);
    let f = flips(&t);
    assert_eq!(f.len(), 400);
    let heads = f.iter().filter(|x| x.1).count();
    // Both outcomes, in roughly equal numbers (400 fair flips: 200 ± 60 is > 4 sigma).
    assert!((140..=260).contains(&heads), "{heads} heads");
}

#[test]
fn the_flipper_calls_the_flip_and_only_they_win_or_lose() {
    cr!("705.2");
    ruling!(
        "Chance Encounter",
        "You can only win a coin flip if you are the player flipping the coin. Your opponent losing a flip does not count as you winning one."
    );
    ruling!(
        "Zndrsplt, Eye of Wisdom",
        "only the player who flipped the coin wins or loses the flip. If that player loses the flip, nobody wins the flip."
    );
    supported("Stitch in Time");
    supported("Chance Encounter");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Chance Encounter");
    let theirs = t.battlefield(P1, "Chance Encounter");
    t.lands(P0, "Island", 2);
    t.lands(P0, "Mountain", 2);
    // P0 calls heads and the coin comes up heads: P0 wins the flip.
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    load(&mut t, &[true]);
    let stitch = t.hand(P0, "Stitch in Time");
    t.cast(P0, stitch).go();
    t.resolve_all();
    assert_eq!(calls_asked(&t), vec![P0]);
    assert_eq!(flips(&t), vec![(P0, true, true, false)]);
    assert_eq!(t.g.extra_turns, vec![P0]);
    assert_eq!(t.counters(mine, "luck"), 1);
    // P0 calls tails and the coin comes up heads: P0 loses; nobody wins.
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    load(&mut t, &[true]);
    t.lands(P0, "Island", 2);
    t.lands(P0, "Mountain", 1);
    let stitch = t.hand(P0, "Stitch in Time");
    t.cast(P0, stitch).go();
    t.resolve_all();
    assert_eq!(flips(&t)[1], (P0, true, false, true));
    assert_eq!(t.g.extra_turns, vec![P0]);
    assert_eq!(t.counters(mine, "luck"), 1);
    assert_eq!(t.counters(theirs, "luck"), 0);
}

#[test]
fn a_flip_that_cares_only_about_heads_or_tails_has_no_call_winner_or_loser() {
    cr!("705.2");
    ruling!(
        "Zndrsplt, Eye of Wisdom",
        "If an effect has a player flip a coin but refers to whether the flip came up heads or tails, that flip has no winner or loser."
    );
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Chance Encounter");
    load(&mut t, &[true, false]);
    cast_and_resolve(&mut t, P0, heads_or_tails(), &[]);
    assert_eq!(t.life(P0), 23);
    cast_and_resolve(&mut t, P0, heads_or_tails(), &[]);
    assert_eq!(t.life(P0), 22);
    assert!(calls_asked(&t).is_empty());
    assert_eq!(
        flips(&t),
        vec![(P0, true, false, false), (P0, false, false, false)]
    );
    assert_eq!(t.counters(mine, "luck"), 0);
}

#[test]
fn an_effect_can_fix_the_result_and_winner_of_a_flip() {
    cr!("705.3");
    ruling!(
        "Edgar, King of Figaro",
        "Ignore the actual results of the coin or coins flipped when applying Edgar's last ability."
    );
    ruling!(
        "Edgar, King of Figaro",
        "Edgar's last ability can cause you to win coin flips that would ordinarily have no winner."
    );
    supported("Edgar, King of Figaro");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Edgar, King of Figaro");
    let luck = t.battlefield(P0, "Chance Encounter");
    // The first flip this turn: the coin comes up tails, but it's treated as heads, and
    // P0 wins a flip that couldn't otherwise be won.
    load(&mut t, &[false, false]);
    cast_and_resolve(&mut t, P0, heads_or_tails(), &[]);
    assert_eq!(t.life(P0), 23);
    assert_eq!(flips(&t), vec![(P0, true, true, false)]);
    assert_eq!(t.counters(luck, "luck"), 1);
    // Only the first time each turn.
    cast_and_resolve(&mut t, P0, heads_or_tails(), &[]);
    assert_eq!(t.life(P0), 22);
    assert_eq!(flips(&t)[1], (P0, false, false, false));
}

#[test]
fn flipping_two_coins_and_ignoring_one() {
    cr!("705.2");
    supported("Krark's Thumb");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Krark's Thumb");
    let luck = t.battlefield(P0, "Chance Encounter");
    // P0 calls heads; two coins come up tails then heads; P0 keeps the winning one. The
    // ignored flip never happened.
    load(&mut t, &[false, true]);
    t.lands(P0, "Island", 2);
    t.lands(P0, "Mountain", 2);
    let stitch = t.hand(P0, "Stitch in Time");
    t.cast(P0, stitch).go();
    t.resolve_all();
    assert_eq!(flips(&t), vec![(P0, true, true, false)]);
    assert_eq!(t.g.extra_turns, vec![P0]);
    assert_eq!(t.counters(luck, "luck"), 1);
    let _ = Zone::Battlefield;
}
