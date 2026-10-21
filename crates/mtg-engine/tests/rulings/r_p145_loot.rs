//! Rulings batch P145 — "draw a card, then discard a card": both happen while the ability
//! resolves, with nothing in between.

use crate::r_p145_common::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Whether the card on top of P0's library (`Memnite`) is in P0's hand, and the number of
/// Psychosis Crawler draw triggers on the stack.
fn at_discard(g: &mtg_engine::game::Game) -> (bool, usize) {
    let drew = g
        .player(P0)
        .hand
        .iter()
        .any(|c| g.obj(*c).chars.name == "Memnite");
    let crawler = g
        .stack
        .iter()
        .filter(|id| {
            g.obj(**id).stack.as_ref().is_some_and(|si| {
                matches!(&si.kind, mtg_engine::object::StackKind::Triggered { ability, .. }
                    if ability.text.contains("Whenever you draw"))
            })
        })
        .count();
    (drew, crawler)
}

/// Sets up the looting ability of the card under test so that it's on the stack (or, for
/// combat triggers, will be put there as the game advances), then runs it to completion.
type Setup = fn(&mut TestGame);

fn tap_ability(t: &mut TestGame, name: &str, lands: usize) {
    t.lands(P0, "Island", lands);
    let c = t.battlefield(P0, name);
    t.activate(P0, c, 0, &[]).expect("activation");
    t.resolve_all();
}

fn cast_and_resolve(t: &mut TestGame, spell: &str) {
    yes(t, P0);
    cast_new(t, P0, spell, &[]);
    t.resolve_all();
}

/// Runs one case: P0 controls Psychosis Crawler ("Whenever you draw a card, each opponent
/// loses 1 life") and has Memnite on top of the library. When P0 chooses the cards to
/// discard, the drawn card is already in hand and the draw trigger hasn't been put on the
/// stack: no player could act between the draw and the discard. Afterwards the triggers
/// resolve (P1 loses `draws` life).
fn run_case(card: &str, draws: i32, setup: Setup) {
    supported(card);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Psychosis Crawler");
    t.hand(P0, "Island");
    t.hand(P0, "Swamp");
    t.hand(P0, "Plains");
    t.library_top(P0, "Memnite");
    let seen = watch(&mut t, P0, is_discard, at_discard);
    let before = t.life(P1);
    setup(&mut t);
    t.resolve_all();
    let seen = seen.lock().unwrap().clone();
    assert!(!seen.is_empty(), "{card}: no discard");
    assert_eq!(seen.last(), Some(&(true, 0)), "{card}");
    assert!(t.graveyard_size(P0) >= 1, "{card}: discarded");
    assert_eq!(before - t.life(P1), draws, "{card}: draw triggers");
}

#[test]
fn loot_draws_and_discards_within_one_resolution() {
    cr!("608.2", "608.2c", "603.3");
    let cases: Vec<(&str, i32, fn(), Setup)> = vec![
        (
            "Bonded Fetch",
            1,
            || {
                ruling!(
                    "Bonded Fetch",
                    "You draw a card and discard a card all while Bonded Fetch's ability is resolving. Nothing can happen in between, and no player can take actions."
                );
            },
            |t| tap_ability(t, "Bonded Fetch", 0),
        ),
        (
            "Facet Reader",
            1,
            || {
                ruling!(
                    "Facet Reader",
                    "You draw a card and discard a card all while Facet Reader’s ability is resolving. Nothing can happen between the two"
                );
            },
            |t| tap_ability(t, "Facet Reader", 1),
        ),
        (
            "Jalum Tome",
            1,
            || {
                ruling!(
                    "Jalum Tome",
                    "You draw a card and discard a card all while Jalum Tome's ability is resolving."
                );
            },
            |t| tap_ability(t, "Jalum Tome", 2),
        ),
        (
            "Obsessive Stitcher",
            1,
            || {
                ruling!(
                    "Obsessive Stitcher",
                    "You draw a card and discard a card all while Obsessive Stitcher's first ability is resolving."
                );
            },
            |t| tap_ability(t, "Obsessive Stitcher", 0),
        ),
        (
            "Teferi's Protege",
            1,
            || {
                ruling!(
                    "Teferi's Protege",
                    "You draw a card and discard a card all while the ability is resolving. Nothing can happen between the two"
                );
            },
            |t| tap_ability(t, "Teferi's Protege", 2),
        ),
        (
            "Erratic Visionary",
            1,
            || {
                ruling!(
                    "Erratic Visionary",
                    "You can’t do anything in between drawing a card and discarding a card, including casting the card you drew."
                );
            },
            |t| tap_ability(t, "Erratic Visionary", 2),
        ),
        (
            "Magus of the Bazaar",
            2,
            || {
                ruling!(
                    "Magus of the Bazaar",
                    "You draw two cards and discard three cards all while the activated ability is resolving."
                );
            },
            |t| tap_ability(t, "Magus of the Bazaar", 0),
        ),
        (
            "Teferi, Master of Time",
            1,
            || {
                ruling!(
                    "Teferi, Master of Time",
                    "You draw a card and discard a card all while Teferi's first loyalty ability is resolving."
                );
            },
            |t| tap_ability(t, "Teferi, Master of Time", 0),
        ),
        (
            "Greater Good",
            2,
            || {
                ruling!(
                    "Greater Good",
                    "You draw and discard cards all while Greater Good's ability is resolving."
                );
            },
            |t| {
                let bears = t.battlefield(P0, "Grizzly Bears");
                let gg = t.battlefield(P0, "Greater Good");
                t.answer_choose(P0, &[obj(bears)]);
                t.activate(P0, gg, 0, &[]).expect("activation");
                t.resolve_all();
            },
        ),
        (
            "Hard Cover",
            1,
            || {
                ruling!(
                    "Hard Cover",
                    "You draw and discard all while the granted ability is resolving."
                );
            },
            |t| {
                let bears = t.battlefield(P0, "Grizzly Bears");
                cast_resolve(t, P0, "Hard Cover", &[obj(bears)]);
                t.activate(P0, bears, 0, &[]).expect("activation");
                t.resolve_all();
            },
        ),
        (
            "Pollywog Symbiote",
            1,
            || {
                ruling!(
                    "Pollywog Symbiote",
                    "You draw a card and discard a card all while Pollywog Symbiote’s ability is resolving."
                );
            },
            |t| {
                t.battlefield(P0, "Pollywog Symbiote");
                cast_new(t, P0, "Gemrazer", &[]);
                t.resolve();
            },
        ),
        (
            "Riddlesmith",
            1,
            || {
                ruling!(
                    "Riddlesmith",
                    "You draw a card and discard a card all while Riddlesmith's ability is resolving."
                );
            },
            |t| {
                t.battlefield(P0, "Riddlesmith");
                cast_and_resolve(t, "Ornithopter");
            },
        ),
        (
            "Academy Wall",
            1,
            || {
                ruling!(
                    "Academy Wall",
                    "You draw a card and discard a card while Academy Wall's last ability is resolving."
                );
            },
            |t| {
                t.battlefield(P0, "Academy Wall");
                cast_and_resolve(t, "Dark Ritual");
            },
        ),
        (
            "Windrider Wizard",
            1,
            || {
                ruling!(
                    "Windrider Wizard",
                    "You draw a card and discard a card while Windrider Wizard’s last ability is resolving."
                );
            },
            |t| {
                t.battlefield(P0, "Windrider Wizard");
                cast_and_resolve(t, "Dark Ritual");
            },
        ),
        (
            "Coralhelm Chronicler",
            1,
            || {
                ruling!(
                    "Coralhelm Chronicler",
                    "You draw a card and discard a card while Coralhelm Chronicler's first ability is resolving."
                );
            },
            |t| {
                t.battlefield(P0, "Coralhelm Chronicler");
                t.lands(P0, "Mountain", 5);
                let bolt = t.hand(P0, "Burst Lightning");
                t.cast(P0, bolt)
                    .kicked(true)
                    .target(Entity::Player(P0))
                    .go();
                t.resolve();
            },
        ),
        (
            "Sage of the Falls",
            1,
            || {
                ruling!(
                    "Sage of the Falls",
                    "You draw a card and discard a card all while Sage of the Falls's ability is resolving."
                );
            },
            |t| {
                t.battlefield(P0, "Sage of the Falls");
                yes(t, P0);
                enter(t, P0, "Grizzly Bears");
                t.resolve();
            },
        ),
        (
            "Sanctum of Calm Waters",
            1,
            || {
                ruling!(
                    "Sanctum of Calm Waters",
                    "You draw X cards and discard a card all while the triggered ability is resolving."
                );
            },
            |t| {
                t.battlefield(P0, "Sanctum of Calm Waters");
                t.set_step(P0, Step::Draw);
                yes(t, P0);
                t.advance_to(P0, Step::PrecombatMain);
                t.resolve();
            },
        ),
        (
            // P1 also takes the looter's 1 combat damage.
            "Looter il-Kor",
            2,
            || {
                ruling!(
                    "Looter il-Kor",
                    "You draw a card and discard a card all while Looter il-Kor's ability is resolving."
                );
            },
            |t| {
                let looter = t.battlefield(P0, "Looter il-Kor");
                attack_with(t, &[(looter, Entity::Player(P1))]);
                block_and_finish(t, P1, &[]);
            },
        ),
    ];
    for (card, draws, cite, setup) in cases {
        cite();
        run_case(card, draws, setup);
    }
}
