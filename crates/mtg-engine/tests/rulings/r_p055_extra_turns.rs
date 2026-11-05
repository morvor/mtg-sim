//! Rulings batch P055 — extra turns (CR 500.7): what an extra turn is, the most recently
//! created one first, Lighthouse Chronologist in two- and multiplayer games, Ugin's
//! Nexus skipping them, Time Sieve's cost; and Emissary of Grudges's secretly chosen
//! opponent (CR 607.2d) and its ability's cost.

use crate::r_p054_common::{o, pool};
use crate::r_p055_common::*;
use crate::r_s01_common::supported;
use crate::r_s02_common::{can_activate, can_attack, destroy};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// `p` casts Time Walk ("Take an extra turn after this one."), and it resolves.
fn time_walk(t: &mut TestGame, p: PlayerId) {
    t.lands(p, "Island", 2);
    let tw = t.hand(p, "Time Walk");
    t.cast(p, tw).go();
    t.resolve_all();
}

#[test]
fn an_extra_turn_is_one_created_by_a_spell_or_ability() {
    cr!("500.7");
    ruling!(
        "Medomai the Ageless",
        "An \"extra turn\" is any turn created by a spell or ability. Notably, it doesn't include additional turns taken in tournaments after time expires for a round."
    );
    ruling!(
        "Trouble in Pairs",
        "An \"extra turn\" is any turn created by a spell or ability. Notably, it doesn't include additional turns taken in tournaments after time expires for a round."
    );
    ruling!(
        "Ugin's Nexus",
        "An \"extra turn\" is any turn created by a spell or ability. Notably, it doesn't include additional turns taken in tournaments after time expires for a round."
    );
    supported("Medomai the Ageless");
    supported("Trouble in Pairs");
    supported("Ugin's Nexus");
    // Medomai can't attack during the extra turn Time Walk creates; it can during a
    // normal turn.
    let mut t = TestGame::new(2);
    let medomai = t.battlefield(P0, "Medomai the Ageless");
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(can_attack(&mut t, medomai));
    t.set_step(P0, Step::PrecombatMain);
    time_walk(&mut t, P0);
    assert_eq!(next_turns(&mut t, 1), vec![P0]);
    t.advance_to(P0, Step::BeginningOfCombat);
    assert!(!can_attack(&mut t, medomai));
    // The next normal turn of P0's: it can attack again.
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::BeginningOfCombat);
    assert!(can_attack(&mut t, medomai));
    // Trouble in Pairs (P1's) skips P0's extra turns, from a spell (Time Walk) or an
    // ability (Time Sieve); Ugin's Nexus skips anyone's. Positive control: none.
    for skipper in [None, Some("Trouble in Pairs"), Some("Ugin's Nexus")] {
        for by_ability in [false, true] {
            let mut t = TestGame::new(2);
            if let Some(s) = skipper {
                t.battlefield(P1, s);
            }
            if by_ability {
                let sieve = t.battlefield(P0, "Time Sieve");
                let thopters: Vec<Entity> = (0..5)
                    .map(|_| o(t.battlefield(P0, "Ornithopter")))
                    .collect();
                t.answer_choose(P0, &thopters);
                t.activate(P0, sieve, 0, &[]).expect("Time Sieve");
                t.resolve_all();
            } else {
                time_walk(&mut t, P0);
            }
            assert_eq!(t.g.extra_turns.len(), 1);
            let turns = next_turns(&mut t, 2);
            let want = if skipper.is_some() {
                vec![P1, P0]
            } else {
                vec![P0, P1]
            };
            assert_eq!(turns, want, "{skipper:?}, ability: {by_ability}");
        }
    }
}

#[test]
fn time_sieve_can_be_one_of_the_five_artifacts() {
    cr!("602.2", "118.3");
    ruling!(
        "Time Sieve",
        "You may sacrifice Time Sieve as one of the artifacts to pay the cost of its own ability."
    );
    supported("Time Sieve");
    let mut t = TestGame::new(2);
    let sieve = t.battlefield(P0, "Time Sieve");
    let mut five = vec![o(sieve)];
    for _ in 0..4 {
        five.push(o(t.battlefield(P0, "Ornithopter")));
    }
    assert!(can_activate(&mut t, P0, sieve));
    t.answer_choose(P0, &five);
    let from = t.asked().len();
    t.activate(P0, sieve, 0, &[]).expect("Time Sieve");
    let offered = entity_choices(&t, P0, from);
    assert_eq!(offered.len(), 1);
    assert!(offered[0].contains(&o(sieve)));
    assert!(t.in_graveyard(P0, "Time Sieve"));
    assert_eq!(t.g.player(P0).graveyard.len(), 5);
    t.resolve_all();
    assert_eq!(next_turns(&mut t, 2), vec![P0, P1]);
    // With only four artifacts in all, it can't be activated.
    let mut t = TestGame::new(2);
    let sieve = t.battlefield(P0, "Time Sieve");
    for _ in 0..3 {
        t.battlefield(P0, "Ornithopter");
    }
    assert!(!can_activate(&mut t, P0, sieve));
}

#[test]
fn magistrates_scepters_extra_turns_most_recent_first() {
    cr!("500.7");
    ruling!(
        "Magistrate's Scepter",
        "If multiple extra turns would be taken after this one, perhaps because more than one player has activated the last ability of their Magistrate’s Scepter, the most recently created one is taken first."
    );
    supported("Magistrate's Scepter");
    let mut t = TestGame::new(3);
    for p in [P1, P2] {
        let s = t.battlefield(p, "Magistrate's Scepter");
        t.g.add_counters(o(s), "charge", 3, None);
        t.activate(p, s, 1, &[]).expect("Scepter");
        t.resolve_all();
    }
    // P2's was created last: P2, then P1, then the normal order resumes after P0's turn.
    assert_eq!(next_turns(&mut t, 5), vec![P2, P1, P1, P2, P0]);
}

/// A level 7 Lighthouse Chronologist for `p` ("At the beginning of each end step, if it's
/// not your turn, take an extra turn after this one.").
fn chronologist(t: &mut TestGame, p: PlayerId) -> ObjectId {
    supported("Lighthouse Chronologist");
    let c = t.battlefield(p, "Lighthouse Chronologist");
    t.g.add_counters(o(c), "level", 7, None);
    t.g.recompute();
    assert_eq!(t.pt(c), (3, 5));
    c
}

#[test]
fn lighthouse_chronologist_in_a_two_player_game() {
    cr!("500.7", "603.4");
    ruling!(
        "Lighthouse Chronologist",
        "In a two-player game, if you have a level 7 Lighthouse Chronologist and there are no other extra-turn effects, the game will proceed like this (with extra turns in brackets): your turn, your opponent’s turn, [your turn], your turn, your opponent’s turn, [your turn], your turn, your opponent’s turn, and so on."
    );
    ruling!(
        "Lighthouse Chronologist",
        "The effects of multiple level 7 Lighthouse Chronologists are cumulative. If you control two of them in a two-player game, you’ll take three turns for each turn your opponent takes."
    );
    let mut t = TestGame::new(2);
    chronologist(&mut t, P0);
    assert_eq!(next_turns(&mut t, 6), vec![P1, P0, P0, P1, P0, P0]);
    let mut t = TestGame::new(2);
    chronologist(&mut t, P0);
    chronologist(&mut t, P0);
    assert_eq!(
        next_turns(&mut t, 8),
        vec![P1, P0, P0, P0, P1, P0, P0, P0]
    );
}

#[test]
fn lighthouse_chronologist_in_a_four_player_game() {
    cr!("500.7", "603.4");
    ruling!(
        "Lighthouse Chronologist",
        "In a four-player game, if you have a level 7 Lighthouse Chronologist and there are no other extra-turn effects, the game will proceed like this (with extra turns in brackets): your turn, opponent A’s turn, [your turn], opponent B’s turn, [your turn], opponent C’s turn, [your turn], your turn, opponent A’s turn, and so on. The same principle applies to a multiplayer game with a different number of players."
    );
    let mut t = TestGame::new(4);
    chronologist(&mut t, P0);
    assert_eq!(
        next_turns(&mut t, 8),
        vec![P1, P0, P2, P0, P3, P0, P0, P1]
    );
    // Three players.
    let mut t = TestGame::new(3);
    chronologist(&mut t, P0);
    assert_eq!(next_turns(&mut t, 5), vec![P1, P0, P2, P0, P0]);
}

#[test]
fn lighthouse_chronologists_of_two_players_in_a_multiplayer_game() {
    cr!("500.7", "603.3b", "101.4");
    ruling!(
        "Lighthouse Chronologist",
        "In a multiplayer game, if multiple players each control a level 7 Lighthouse Chronologist, extra turns may sometimes be created faster than they can be taken. Keep track of them carefully. If multiple Chronologist’s abilities trigger during the same turn, the player whose turn would show up sooner in the natural turn order will get the first extra turn."
    );
    // Players A (P0), B (P1) and C (P2); B and C each control one.
    let mut t = TestGame::new(3);
    chronologist(&mut t, P1);
    chronologist(&mut t, P2);
    assert_eq!(next_turns(&mut t, 6), vec![P1, P2, P1, P2, P1, P2]);
    // Four players, B and D: at A's end step both trigger; B is sooner after A.
    let mut t = TestGame::new(4);
    chronologist(&mut t, P1);
    chronologist(&mut t, P3);
    assert_eq!(next_turns(&mut t, 2), vec![P1, P3]);
    // And with the active player's seat elsewhere: C's turn, B and D control them; D is
    // sooner after C.
    let mut t = TestGame::new(4);
    t.set_step(P2, Step::PrecombatMain);
    chronologist(&mut t, P1);
    chronologist(&mut t, P3);
    assert_eq!(next_turns(&mut t, 2), vec![P3, P1]);
}

#[test]
fn opposing_lighthouse_chronologists_alternate_turns_and_the_normal_order_resumes() {
    cr!("500.7");
    ruling!(
        "Lighthouse Chronologist",
        "In a two-player game, if you and your opponent each control a level 7 Lighthouse Chronologist, they’ll essentially cancel each other out and you’ll alternate turns. Since all the turns you’d be taking are extra turns created by these abilities, make sure you know whose normal turn is supposed to be next in case one or both of the Lighthouse Chronologists leaves the battlefield."
    );
    let mut t = TestGame::new(2);
    let a = chronologist(&mut t, P0);
    let b = chronologist(&mut t, P1);
    // P0's normal turn, then extra turns: P1, P0, P1.
    assert_eq!(next_turns(&mut t, 3), vec![P1, P0, P1]);
    // During P1's extra turn both leave: P1's normal turn (the one after P0's normal
    // turn) is next.
    destroy(&mut t, a);
    destroy(&mut t, b);
    assert_eq!(t.zone(a), Zone::Graveyard(P0));
    assert_eq!(next_turns(&mut t, 3), vec![P1, P0, P1]);
}

#[test]
fn ugins_nexus_skips_an_extra_turn_only_as_it_would_begin() {
    cr!("500.7", "614.10");
    ruling!(
        "Ugin's Nexus",
        "Extra turns can still be created while Ugin's Nexus is on the battlefield. They're not skipped until they would begin, so if Ugin's Nexus leaves the battlefield before that happens, the extra turns will be unaffected."
    );
    for removed in [false, true] {
        let mut t = TestGame::new(2);
        let nexus = t.battlefield(P1, "Ugin's Nexus");
        time_walk(&mut t, P0);
        assert_eq!(t.g.extra_turns, vec![P0]);
        if removed {
            // P0 destroys it: it's exiled instead, and P1 takes an extra turn after this
            // one (created later, so taken first).
            pool(&mut t, P0);
            let dis = t.hand(P0, "Disenchant");
            t.cast(P0, dis).target(nexus).go();
            t.resolve_all();
            assert_eq!(t.zone(nexus), Zone::Exile);
            assert_eq!(next_turns(&mut t, 3), vec![P1, P0, P1]);
        } else {
            assert_eq!(next_turns(&mut t, 2), vec![P1, P0]);
        }
    }
}

#[test]
fn a_second_ugins_nexus_is_exiled_with_an_extra_turn_that_the_first_skips() {
    cr!("704.5j", "614.6", "500.7");
    ruling!(
        "Ugin's Nexus",
        "Notably, if you control more than one Ugin's Nexus, all but one will be exiled and an extra turn created for each. If the Ugin's Nexus that remains on the battlefield isn't on the battlefield when each of those extra turns would begin, you can take those turns."
    );
    for removed in [false, true] {
        let mut t = TestGame::new(2);
        let first = t.battlefield(P0, "Ugin's Nexus");
        let second = t.battlefield(P0, "Ugin's Nexus");
        // The legend rule: P0 keeps the first; the second would go to the graveyard and
        // is exiled instead, with an extra turn.
        t.answer_choose(P0, &[o(first)]);
        t.settle();
        assert_eq!(t.zone(first), Zone::Battlefield);
        assert_eq!(t.zone(second), Zone::Exile);
        assert_eq!(t.g.extra_turns, vec![P0]);
        if removed {
            pool(&mut t, P0);
            let dis = t.hand(P0, "Disenchant");
            t.cast(P0, dis).target(first).go();
            t.resolve_all();
            assert_eq!(t.zone(first), Zone::Exile);
            assert_eq!(t.g.extra_turns, vec![P0, P0]);
            assert_eq!(next_turns(&mut t, 3), vec![P0, P0, P1]);
        } else {
            assert_eq!(next_turns(&mut t, 2), vec![P1, P0]);
        }
    }
}

// ---------------------------------------------------------------------------
// Emissary of Grudges
// ---------------------------------------------------------------------------

/// `p`'s Emissary of Grudges enters, secretly choosing `chosen`.
fn emissary(t: &mut TestGame, p: PlayerId, chosen: PlayerId) -> ObjectId {
    supported("Emissary of Grudges");
    t.answer_choose(p, &[Entity::Player(chosen)]);
    let e = t.enter(p, "Emissary of Grudges");
    t.settle();
    e
}

/// `p` casts Lightning Bolt at `at`; it stays on the stack.
fn bolt(t: &mut TestGame, p: PlayerId, at: PlayerId) -> ObjectId {
    t.lands(p, "Mountain", 1);
    let b = t.hand(p, "Lightning Bolt");
    t.cast(p, b).target(at).go()
}

#[test]
fn emissary_of_grudges_without_a_chosen_opponent_cant_pay_its_cost() {
    cr!("607.2d", "602.2b", "707.2");
    ruling!(
        "Emissary of Grudges",
        "If you didn’t choose an opponent when Emissary of Grudges entered the battlefield, you can’t pay the cost to activate its last ability. This will usually happen because you gained control of another player’s Emissary of Grudges or because a creature already on the battlefield became a copy of Emissary of Grudges."
    );
    // Positive control: P0's own Emissary (P1 chosen) redirects P1's Bolt.
    let mut t = TestGame::new(2);
    let e = emissary(&mut t, P0, P1);
    let spell = bolt(&mut t, P1, P0);
    assert!(can_activate(&mut t, P0, e));
    t.activate(P0, e, 0, &[o(spell)]).expect("Emissary");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (20, 17));
    // P1's Emissary (P0 chosen), then P0 gains control of it with Act of Treason.
    let mut t = TestGame::new(2);
    let e = emissary(&mut t, P1, P0);
    pool(&mut t, P0);
    let act = t.hand(P0, "Act of Treason");
    t.cast(P0, act).target(e).go();
    t.resolve_all();
    assert_eq!(t.obj_now(e).controller, P0);
    let _spell = bolt(&mut t, P1, P0);
    assert!(!can_activate(&mut t, P0, e));
    // A creature already on the battlefield becomes a copy (Cytoshape).
    let mut t = TestGame::new(2);
    let model = emissary(&mut t, P1, P0);
    let bear = t.battlefield(P0, "Grizzly Bears");
    pool(&mut t, P0);
    t.answer_choose(P0, &[o(model)]);
    let cyto = t.hand(P0, "Cytoshape");
    t.cast(P0, cyto).target(bear).go();
    t.resolve_all();
    assert_eq!(t.obj_now(bear).chars.name, "Emissary of Grudges");
    let _spell = bolt(&mut t, P1, P0);
    assert!(!can_activate(&mut t, P0, bear));
}

#[test]
fn each_emissary_of_grudges_has_its_own_chosen_opponent() {
    cr!("607.2d", "115.7");
    ruling!(
        "Emissary of Grudges",
        "If you have more than one Emissary of Grudges, you may choose a different opponent for each one. Be careful to keep track of which opponent is chosen for each Emissary of Grudges."
    );
    // P0's first Emissary chose P1, the second P2. P2's Bolt at P0: only the second one
    // changes its target.
    for (which, redirected) in [(0usize, false), (1, true)] {
        let mut t = TestGame::new(3);
        let ems = [emissary(&mut t, P0, P1), emissary(&mut t, P0, P2)];
        let spell = bolt(&mut t, P2, P0);
        t.activate(P0, ems[which], 0, &[o(spell)]).expect("Emissary");
        t.answer_targets(P0, &[Entity::Player(P2)]);
        t.resolve_all();
        if redirected {
            assert_eq!((t.life(P0), t.life(P2)), (20, 17));
        } else {
            assert_eq!((t.life(P0), t.life(P2)), (17, 20));
        }
    }
}
