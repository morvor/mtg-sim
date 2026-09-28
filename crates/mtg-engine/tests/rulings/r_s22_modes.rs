//! Rulings batch S22 — modal spells (CR 700.2): the modes are chosen as the spell is cast
//! and can't be changed (CR 601.2b, 700.2a); the chosen modes are performed in the order
//! printed, whatever the order they were chosen in (CR 700.2, 608.2c); no player can act
//! between the modes of a resolving spell (CR 608.2); "choose two" means two different
//! modes unless the spell allows repeats (CR 700.2d).

use crate::r_s01_common::*;
use crate::r_s07_common::chosen_modes;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::*;

/// The lines of the game log from index `from`.
fn log_from(t: &TestGame, from: usize) -> Vec<String> {
    t.g.log[from..].iter().map(|l| l.text.clone()).collect()
}

/// The index of the first log line containing `needle`.
fn line_of(lines: &[String], needle: &str) -> usize {
    lines
        .iter()
        .position(|l| l.contains(needle))
        .unwrap_or_else(|| panic!("no log line with {needle:?} in {lines:#?}"))
}

#[test]
fn fiery_confluence_modes_are_chosen_as_its_cast_and_cant_be_changed() {
    cr!("601.2b", "700.2", "700.2a", "608.2b");
    ruling!(
        "Fiery Confluence",
        "You choose the modes as you cast the spell. Once modes are chosen, they can't be changed."
    );
    supported("Fiery Confluence");
    // Modes: 2 damage to each opponent twice, and destroy target artifact (Ornithopter).
    let mut t = TestGame::new(2);
    let thopter = t.battlefield(P1, "Ornithopter");
    t.lands(P0, "Mountain", 4);
    let conf = t.hand(P0, "Fiery Confluence");
    let spell = t
        .cast(P0, conf)
        .modes(&[1, 1, 2])
        .target(Entity::Object(thopter))
        .go();
    assert_eq!(chosen_modes(&t, spell), vec![1, 1, 2]);
    // As it resolves, the modes chosen are the ones performed; nothing is chosen again.
    let from = t.asked().len();
    t.resolve();
    assert_eq!(t.life(P1), 16);
    assert!(t.in_graveyard(P1, "Ornithopter"));
    assert!(!t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseModes { .. })));
}

#[test]
fn no_player_can_act_between_the_modes_of_a_resolving_confluence() {
    cr!("608.2", "608.2c", "117.1");
    ruling!(
        "Fiery Confluence",
        "No player can cast spells or activate abilities in between the modes of a resolving spell."
    );
    // Three times "1 damage to each creature" at P1's Hill Giant (3/3), cast and resolved
    // through the priority loop with both players passing: whenever a player gets
    // priority, the Giant has no damage marked (before the spell resolves) or is gone
    // (after) — never 1 or 2, as it would between the modes.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Mountain", 4);
    let conf = t.hand(P0, "Fiery Confluence");
    /// The stack size and the damage marked on Hill Giant (if it's on the battlefield).
    fn snapshot(g: &mtg_engine::game::Game) -> (usize, Option<u32>) {
        let giant = g.find_in_zone(mtg_engine::object::Zone::Battlefield, "Hill Giant");
        (g.stack.len(), giant.first().map(|id| g.obj(*id).damage))
    }
    fn is_priority(d: &Decision) -> bool {
        matches!(d, Decision::Priority { .. })
    }
    let seen = [
        watch(&mut t, P0, is_priority, snapshot),
        watch(&mut t, P1, is_priority, snapshot),
    ];
    t.answer(
        P0,
        DecisionKind::Priority,
        Answer::Action(mtg_engine::decision::Action::Cast {
            card: conf,
            method: mtg_engine::object::CastMethod::Normal,
        }),
    );
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0, 0, 0]));
    let ok = t.g.run_until(1000, |g| {
        g.stack.is_empty()
            && !g
                .find_in_zone(mtg_engine::object::Zone::Graveyard(P1), "Hill Giant")
                .is_empty()
    });
    assert!(ok);
    let seen: Vec<(usize, Option<u32>)> = seen
        .iter()
        .flat_map(|s| s.lock().unwrap().clone())
        .collect();
    // P1 got priority with the Confluence on the stack, the Giant undamaged.
    assert!(seen.contains(&(1, Some(0))), "{seen:?}");
    assert!(
        seen.iter().all(|(_, d)| matches!(d, None | Some(0))),
        "a player got priority between the modes: {seen:?}"
    );
}

#[test]
fn a_confluence_performs_its_modes_in_printed_order() {
    cr!("700.2", "608.2c");
    ruling!(
        "Fiery Confluence",
        "No matter which combination of modes you choose, you always follow the instructions of a Confluence in the order they are written. If the same mode is chosen more than once, you choose their relative order as you cast the spell."
    );
    // Chosen as "destroy target artifact", "1 damage to each creature", "2 damage to each
    // opponent": performed as written — damage to creatures, damage to opponents, then
    // the artifact is destroyed.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Grizzly Bears");
    let relic = t.battlefield(P1, "Mind Stone");
    t.lands(P0, "Mountain", 4);
    let conf = t.hand(P0, "Fiery Confluence");
    let spell = t
        .cast(P0, conf)
        .modes(&[2, 0, 1])
        .target(Entity::Object(relic))
        .go();
    assert_eq!(chosen_modes(&t, spell), vec![0, 1, 2]);
    let from = t.g.log.len();
    t.resolve();
    let lines = log_from(&t, from);
    let creature = line_of(&lines, "deals 1 damage");
    let player = line_of(&lines, "deals 2 damage");
    let destroyed = line_of(&lines, "Mind Stone");
    assert!(creature < player && player < destroyed, "{lines:#?}");
    assert!(t.in_graveyard(P1, "Mind Stone"));
    assert_eq!(t.life(P1), 18);
}

#[test]
fn a_season_performs_its_modes_in_printed_order() {
    cr!("700.2", "700.2i", "608.2c");
    ruling!(
        "Season of Weaving",
        "No matter which combination of modes you choose, you always follow the instructions of a Season in the order they are written. If the same mode is chosen more than once, you choose their relative order as you cast the spell."
    );
    supported("Season of Weaving");
    // Chosen as "{P}{P}{P} — Return each nonland, nontoken permanent to its owner's hand"
    // then "{P}{P} — Choose an artifact or creature you control. Create a token that's a
    // copy of it": performed as written, the token copy of Grizzly Bears is created first
    // and stays (it's a token) while the Bears is returned.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Island", 6);
    let season = t.hand(P0, "Season of Weaving");
    t.answer_choose(P0, &[Entity::Object(bears)]);
    let spell = t.cast(P0, season).modes(&[2, 1]).go();
    assert_eq!(chosen_modes(&t, spell), vec![1, 2]);
    t.resolve_all();
    assert!(t.in_hand(P0, "Grizzly Bears"));
    let copies = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(copies.len(), 1);
    assert!(t.obj(copies[0]).is_token());
}

#[test]
fn kolaghans_command_needs_two_different_modes_chosen_as_its_cast() {
    cr!("700.2", "700.2a", "700.2d", "601.2b");
    ruling!(
        "Kolaghan's Command",
        "You choose the two modes as you cast the spell. You must choose two different modes. Once modes are chosen, they can't be changed."
    );
    supported("Kolaghan's Command");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Mountain", 2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.hand(P1, "Forest");
    let cmd = t.hand(P0, "Kolaghan's Command");
    let from = t.asked().len();
    // "2 damage to any target" twice isn't a legal choice.
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![3, 3]));
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1, 3]));
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    let spell = t.cast(P0, cmd).go();
    let asked: Vec<(u32, u32, bool)> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseModes {
                min,
                max,
                allow_repeat,
                ..
            } => Some((*min, *max, *allow_repeat)),
            _ => None,
        })
        .collect();
    assert_eq!(asked[0], (2, 2, false));
    let modes = chosen_modes(&t, spell);
    assert_eq!(modes.len(), 2);
    assert_ne!(modes[0], modes[1]);
}
