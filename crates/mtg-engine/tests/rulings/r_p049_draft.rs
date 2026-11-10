//! Rulings batch P049 — the Conspiracy draft-matters cards (CR 905.2): the numbers noted
//! as Custodi Peacekeeper, Lurking Automaton and Pyretic Hunter are drafted and how the
//! game reads them (the controller's notes, CR 905.2b); the cards Cogwork Grinder removes
//! from the draft (not in any card pool, hidden from the other players unless Cogwork Spy
//! looks, still drafted); and Paliano, the High City's three different colors.

use crate::r_p049_common::*;
use mtg_engine::card::{card, CardDef};
use mtg_engine::draft::{Draft, DraftInfo, DraftOption, DraftStyle};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;
use std::sync::Arc;

/// A one-player draft of one pack holding the named cards, every card drafted in order.
fn solo_draft(pack: &[&str], remove: &[usize]) -> DraftInfo {
    let cards: Vec<Arc<CardDef>> = pack.iter().map(|n| card(n)).collect();
    let mut d = Draft::new(vec![vec![cards]], DraftStyle::Conspiracy, 1);
    for k in 0..pack.len() {
        let opt = remove.contains(&k).then(|| DraftOption::Remove {
            with: "Cogwork Grinder".into(),
        });
        d.pick_with(P0, &[0], opt).unwrap();
    }
    assert!(d.is_complete());
    d.finish().1
}

/// A three-player game that follows the draft `info`.
fn game_after(info: DraftInfo) -> TestGame {
    let mut t = TestGame::new(3);
    t.g.start.draft = info;
    t.set_step(P0, Step::PrecombatMain);
    t
}

/// A creature of `p`'s with the given power (an Ornithopter, a Llanowar Elves, or a
/// Grizzly Bears with +1/+1 counters).
fn creature_with_power(t: &mut TestGame, p: PlayerId, power: i32) -> ObjectId {
    let c = match power {
        0 => t.battlefield(p, "Ornithopter"),
        1 => t.battlefield(p, "Llanowar Elves"),
        _ => t.battlefield(p, "Grizzly Bears"),
    };
    if power > 2 {
        put_counters(t, c, counters::PLUS1, (power - 2) as u32);
    }
    assert_eq!(t.pt(c).0, power);
    c
}

/// Whether `p` can activate Custodi Peacekeeper `cp` targeting `target` now: whether the
/// target is offered (an illegal target isn't a candidate, CR 115.3); if so, the ability
/// is activated targeting it.
fn peacekeeper_can_tap(t: &mut TestGame, p: PlayerId, cp: ObjectId, target: ObjectId) -> bool {
    t.lands(p, "Plains", 1);
    let cp = t.g.current(cp);
    t.g.objects[cp.0 as usize].tapped = false;
    t.g.objects[cp.0 as usize].summoning_sick = false;
    t.clear_answers();
    let from = t.asked().len();
    let r = t.activate(p, cp, 0, &[obj(target)]);
    t.clear_answers();
    let offered = target_candidates(t, p, from);
    let legal = offered.iter().any(|c| c.contains(&obj(target)));
    if !legal {
        // Activated with another target, or not at all: undo any activation.
        if r.is_ok() {
            t.resolve_all();
        }
        return false;
    }
    assert!(r.is_ok(), "{r:?}");
    true
}

#[test]
fn custodi_peacekeeper_uses_the_highest_number_noted_for_its_name() {
    cr!("905.2b");
    ruling!(
        "Custodi Peacekeeper",
        "each Custodi Peacekeeper's ability could tap a creature with power 6 or less."
    );
    supported("Custodi Peacekeeper");
    // Drafted as the third and the sixth card of a round: 3 and 6 are noted; both use 6.
    let info = solo_draft(
        &[
            "Grizzly Bears",
            "Grizzly Bears",
            "Custodi Peacekeeper",
            "Grizzly Bears",
            "Grizzly Bears",
            "Custodi Peacekeeper",
        ],
        &[],
    );
    assert_eq!(
        info.notes
            .iter()
            .filter(|n| n.card == "Custodi Peacekeeper")
            .filter_map(|n| n.number)
            .collect::<Vec<_>>(),
        vec![3, 6]
    );
    let mut t = game_after(info);
    let six = creature_with_power(&mut t, P1, 6);
    let seven = creature_with_power(&mut t, P1, 7);
    for _ in 0..2 {
        let cp = t.battlefield(P0, "Custodi Peacekeeper");
        assert!(!peacekeeper_can_tap(&mut t, P0, cp, seven));
        assert!(peacekeeper_can_tap(&mut t, P0, cp, six));
        t.resolve_all();
        assert!(tapped(&t, six));
        untap(&mut t, six);
    }
}

#[test]
fn custodi_peacekeeper_gained_by_a_player_who_drafted_none_taps_only_power_0() {
    cr!("905.2b");
    ruling!(
        "Custodi Peacekeeper",
        "If you gain control of a Custodi Peacekeeper, but you didn't draft one, the highest noted number is considered to be 0."
    );
    supported("Custodi Peacekeeper");
    // P1 drafted it as the second card of the round (from the pack P0 passed them); P0
    // drafted none.
    let cards: Vec<Arc<CardDef>> = vec![card("Grizzly Bears"), card("Custodi Peacekeeper")];
    let filler = || -> Vec<Arc<CardDef>> { vec![card("Grizzly Bears"), card("Grizzly Bears")] };
    let mut d = Draft::new(
        vec![vec![cards], vec![filler()], vec![filler()]],
        DraftStyle::Conspiracy,
        1,
    );
    for _ in 0..2 {
        for p in [P0, P1, P2] {
            d.pick(p, &[0]).unwrap();
        }
    }
    let (_, info) = d.finish();
    assert!(info
        .notes
        .iter()
        .any(|n| n.player == P1 && n.card == "Custodi Peacekeeper" && n.number == Some(2)));
    let mut t = game_after(info);
    let cp = t.battlefield(P1, "Custodi Peacekeeper");
    let zero = creature_with_power(&mut t, P2, 0);
    let one = creature_with_power(&mut t, P2, 1);
    let two = creature_with_power(&mut t, P2, 2);
    // P1, who noted 2, can tap the 2-power creature.
    assert!(peacekeeper_can_tap(&mut t, P1, cp, two));
    t.resolve_all();
    untap(&mut t, two);
    // P0 gains control: P0 noted nothing, so only power 0 or less.
    give_control(&mut t, cp, P0);
    assert!(!peacekeeper_can_tap(&mut t, P0, cp, two));
    assert!(!peacekeeper_can_tap(&mut t, P0, cp, one));
    assert!(peacekeeper_can_tap(&mut t, P0, cp, zero));
    t.resolve_all();
    assert!(tapped(&t, zero));
}

#[test]
fn lurking_automaton_enters_with_the_highest_number_its_controller_noted() {
    cr!("905.2b", "614.1c");
    ruling!(
        "Lurking Automaton",
        "each Lurking Automaton that enters the battlefield under your control does so with six +1/+1 counters on it."
    );
    supported("Lurking Automaton");
    let info = solo_draft(
        &[
            "Grizzly Bears",
            "Grizzly Bears",
            "Lurking Automaton",
            "Grizzly Bears",
            "Grizzly Bears",
            "Lurking Automaton",
        ],
        &[],
    );
    let mut t = game_after(info);
    for _ in 0..2 {
        let a = t.enter(P0, "Lurking Automaton");
        assert_eq!(t.counters(a, counters::PLUS1), 6);
        assert_eq!(t.pt(a), (6, 6));
    }
    ruling!(
        "Lurking Automaton",
        "If a Lurking Automaton owned by another player enters the battlefield under your control, the number of counters it enters with will be based on the highest number you noted"
    );
    // P1 owns one; it enters under P0's control with P0's six. P0's own, entering under
    // P1's control (P1 drafted none), enters with none and dies.
    let theirs = enter_owned_by(&mut t, P1, P0, "Lurking Automaton");
    assert_eq!(t.counters(theirs, counters::PLUS1), 6);
    assert_eq!(t.obj_now(theirs).controller, P0);
    let mine = enter_owned_by(&mut t, P0, P1, "Lurking Automaton");
    assert!(!t.on_battlefield(mine));
    assert!(t.in_graveyard(P0, "Lurking Automaton"));
}

#[test]
fn pyretic_hunter_enters_with_the_highest_number_noted_or_zero() {
    cr!("905.2b", "614.1c", "704.5f");
    ruling!(
        "Pyretic Hunter",
        "each will enter the battlefield with four +1/+1 counters."
    );
    supported("Pyretic Hunter");
    // Drafted as the third and fourth cards: 4 is the highest.
    let info = solo_draft(
        &[
            "Grizzly Bears",
            "Grizzly Bears",
            "Pyretic Hunter",
            "Pyretic Hunter",
        ],
        &[],
    );
    let mut t = game_after(info);
    for _ in 0..2 {
        let h = t.enter(P0, "Pyretic Hunter");
        assert_eq!(t.counters(h, counters::PLUS1), 4);
    }
    ruling!(
        "Pyretic Hunter",
        "It will enter with zero +1/+1 counters and will be put into its owner's graveyard (unless something else is raising its toughness)."
    );
    // P1 drafted none: theirs enters with no counters and dies to state-based actions...
    let h = t.enter(P1, "Pyretic Hunter");
    t.settle();
    assert!(!t.on_battlefield(h));
    assert!(t.in_graveyard(P1, "Pyretic Hunter"));
    // ...unless something raises its toughness.
    supported("Glorious Anthem");
    t.battlefield(P1, "Glorious Anthem");
    let h = t.enter(P1, "Pyretic Hunter");
    t.settle();
    assert!(t.on_battlefield(h));
    assert_eq!(t.counters(h, counters::PLUS1), 0);
    assert_eq!(t.pt(h), (1, 1));
}

#[test]
fn cogwork_grinder_counts_the_cards_its_controller_removed() {
    cr!("905.2", "905.2b", "614.1c");
    ruling!(
        "Cogwork Grinder",
        "If you draft multiple Cogwork Grinders, the value for X will be the same for each of them."
    );
    supported("Cogwork Grinder");
    // Draft a Grinder, remove a card, draft a second Grinder, remove another: X is 2 for
    // both, although the first card was removed before the second Grinder was drafted.
    let info = solo_draft(
        &[
            "Cogwork Grinder",
            "Serra Angel",
            "Cogwork Grinder",
            "Shock",
            "Grizzly Bears",
        ],
        &[1, 3],
    );
    assert_eq!(info.removed.len(), 2);
    let mut t = game_after(info);
    for _ in 0..2 {
        let g = t.enter(P0, "Cogwork Grinder");
        assert_eq!(t.counters(g, counters::PLUS1), 2);
    }
    ruling!(
        "Cogwork Grinder",
        "If a Cogwork Grinder owned by another player enters the battlefield under your control, the number of counters it enters with will be based on the number of cards you've removed from the draft with Cogwork Grinders, not the number its owner removed."
    );
    let theirs = enter_owned_by(&mut t, P1, P0, "Cogwork Grinder");
    assert_eq!(t.counters(theirs, counters::PLUS1), 2);
    // P1 removed nothing: P0's Grinder entering under P1's control gets 0 and dies.
    let mine = enter_owned_by(&mut t, P0, P1, "Cogwork Grinder");
    assert!(!t.on_battlefield(mine));
    assert!(t.in_graveyard(P0, "Cogwork Grinder"));
}

#[test]
fn cards_removed_from_the_draft_are_in_no_card_pool_and_hidden() {
    cr!("905.1c", "905.1d", "905.2", "905.2c");
    ruling!(
        "Cogwork Grinder",
        "A card that is removed from the draft isn't in any player's card pool."
    );
    ruling!(
        "Cogwork Grinder",
        "No other player may look at the cards you remove from the draft (unless Cogwork Spy allows a player to)."
    );
    supported("Cogwork Grinder");
    let filler = |k: usize| -> Vec<Arc<CardDef>> { (0..k).map(|_| card("Grizzly Bears")).collect() };
    // P0 drafts the Grinder (face up: everyone sees it), then removes Serra Angel from
    // the pack P2 passes them.
    let mut d = Draft::new(
        vec![
            vec![vec![card("Cogwork Grinder"), card("Shock"), card("Opt")]],
            vec![filler(3)],
            vec![vec![card("Serra Angel"), card("Grizzly Bears"), card("Grizzly Bears")]],
        ],
        DraftStyle::Conspiracy,
        1,
    );
    d.pick(P0, &[0]).unwrap();
    d.pick(P1, &[0]).unwrap();
    d.pick(P2, &[1]).unwrap();
    assert!(d.can_see(P1, "Cogwork Grinder"));
    // P2's pack (Serra Angel, Grizzly Bears) is passed left to P0.
    let serra = d
        .pack(P0)
        .iter()
        .position(|c| c.name == "Serra Angel")
        .expect("P0 drafts from P2's pack");
    d.pick_with(
        P0,
        &[serra],
        Some(DraftOption::Remove {
            with: "Cogwork Grinder".into(),
        }),
    )
    .unwrap();
    // Nobody else may look at it, now or later.
    assert!(!d.can_see(P1, "Serra Angel"));
    assert!(!d.can_see(P2, "Serra Angel"));
    while !d.is_complete() {
        for p in [P0, P1, P2] {
            if !d.pack(p).is_empty() {
                let _ = d.pick(p, &[0]);
            }
        }
        assert!(!d.can_see(P1, "Serra Angel"));
        assert!(!d.can_see(P2, "Serra Angel"));
    }
    let (pools, info) = d.finish();
    assert!(pools
        .iter()
        .all(|pool| pool.iter().all(|c| c.name != "Serra Angel")));
    assert_eq!(info.removed.len(), 1);
    assert!(!info.removed[0].face_up);
    assert_eq!(info.removed[0].card.name, "Serra Angel");
}

#[test]
fn cogwork_spy_may_look_at_a_card_removed_face_down_by_cogwork_grinder() {
    cr!("905.1c", "905.2");
    ruling!(
        "Cogwork Spy",
        "You may look at the card even if it's removed from the draft face down by Cogwork Grinder."
    );
    supported("Cogwork Spy");
    supported("Cogwork Grinder");
    let filler = |k: usize| -> Vec<Arc<CardDef>> { (0..k).map(|_| card("Grizzly Bears")).collect() };
    // P2 drafts Cogwork Spy from their pack, which passes left to P0, who drafts the
    // next card from it (Serra Angel) and removes it with the Grinder drafted first.
    let mut d = Draft::new(
        vec![
            vec![vec![card("Cogwork Grinder"), card("Shock"), card("Opt")]],
            vec![filler(3)],
            vec![vec![card("Cogwork Spy"), card("Serra Angel"), card("Grizzly Bears")]],
        ],
        DraftStyle::Conspiracy,
        1,
    );
    d.pick(P0, &[0]).unwrap();
    d.pick(P1, &[0]).unwrap();
    d.pick(P2, &[0]).unwrap();
    let serra = d
        .pack(P0)
        .iter()
        .position(|c| c.name == "Serra Angel")
        .expect("P0 drafts from P2's pack");
    assert!(!d.can_see(P2, "Serra Angel"));
    d.pick_with(
        P0,
        &[serra],
        Some(DraftOption::Remove {
            with: "Cogwork Grinder".into(),
        }),
    )
    .unwrap();
    // The Spy's drafter looks at it; the other player doesn't.
    assert!(d.can_see(P2, "Serra Angel"));
    assert!(!d.can_see(P1, "Serra Angel"));
    assert_eq!(d.info.removed.len(), 1);
    assert!(!d.info.removed[0].face_up);
}

#[test]
fn a_card_removed_by_cogwork_grinder_is_still_drafted() {
    cr!("905.2", "905.2b", "905.2c");
    ruling!(
        "Cogwork Grinder",
        "You still draft the card that you remove. This may matter if an ability refers to \"the next card you draft\" or similar."
    );
    supported("Cogwork Grinder");
    supported("Aether Searcher");
    supported("Lurking Automaton");
    // Aether Searcher: "Reveal the next card you draft and note its name." The next card
    // (Lurking Automaton, drafted third) is removed, but it was drafted: it's noted for
    // Aether Searcher, and it noted 3 (revealed as it's drafted, then removed face
    // down); the second Grinder, drafted face up, is removed face down too.
    let info = solo_draft(
        &[
            "Cogwork Grinder",
            "Aether Searcher",
            "Lurking Automaton",
            "Cogwork Grinder",
            "Grizzly Bears",
        ],
        &[2, 3],
    );
    assert!(info.notes.iter().any(|n| n.player == P0
        && n.card == "Aether Searcher"
        && n.noted_name.as_deref() == Some("Lurking Automaton")));
    assert!(info
        .notes
        .iter()
        .any(|n| n.player == P0 && n.card == "Lurking Automaton" && n.number == Some(3)));
    assert_eq!(info.removed.len(), 2);
    assert!(info.removed.iter().all(|r| !r.face_up));
    assert!(info.removed.iter().any(|r| r.card.name == "Cogwork Grinder"));
    // Both removed cards count for the Grinder that stayed in the pool.
    let mut t = game_after(info);
    let g = t.enter(P0, "Cogwork Grinder");
    assert_eq!(t.counters(g, counters::PLUS1), 2);
    // Aether Searcher's note is of a card that isn't in the pool; a Lurking Automaton
    // that does enter still uses the noted 3.
    let a = t.enter(P0, "Lurking Automaton");
    assert_eq!(t.counters(a, counters::PLUS1), 3);
}

#[test]
fn paliano_the_high_citys_three_colors_are_different() {
    cr!("905.2b");
    ruling!("Paliano, the High City", "The three chosen colors must be different.");
    supported("Paliano, the High City");
    // The default choices: three different colors.
    for seed in 1..6 {
        let mut d = Draft::new(
            vec![vec![vec![card("Paliano, the High City")]]],
            DraftStyle::Conspiracy,
            seed,
        );
        d.pick(P0, &[0]).unwrap();
        let mut c = d.info.notes[0].colors.clone();
        assert_eq!(c.len(), 3);
        c.sort();
        c.dedup();
        assert_eq!(c.len(), 3, "seed {seed}");
    }
    // Choices naming the same color twice aren't accepted: three different colors are
    // chosen instead.
    let mut d = Draft::new(
        vec![vec![vec![card("Paliano, the High City")]]],
        DraftStyle::Conspiracy,
        3,
    );
    d.preset_colors(P0, [Color::Red, Color::Red, Color::Green]);
    d.pick(P0, &[0]).unwrap();
    let mut c = d.info.notes[0].colors.clone();
    assert_eq!(c.len(), 3);
    c.sort();
    c.dedup();
    assert_eq!(c.len(), 3);
}
