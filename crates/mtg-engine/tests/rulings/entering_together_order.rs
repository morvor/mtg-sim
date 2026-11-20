//! Item `ruling-conflicts-12`, question 2: permanents entering the battlefield at the same
//! time have their "as [this] enters" / "enters with" replacement effects applied in the
//! order their controller chooses (CR 616.1, 616.1f, the 616.1g example; Thief of Blood's
//! ruling), when the order can matter: one of them changes the game as it's applied and
//! another reads or changes it too. The default is the order of the moves
//! (interpretation `simultaneous-entries-ordered-by-controller`). Which effects apply to
//! each is determined as the event is proposed: one whose source an earlier-applied
//! effect made leave the battlefield (Rhythm of the Wild's ruling) or turned face down
//! (Ixidron's) still applies.

use crate::r_residue_counters_common::*;
use mtg_engine::ability::LibraryPosition;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::events::MoveCause;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::replacement::{EtbInfo, MoveEv};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// The options of every "which replacement effect first" question asked to `p`.
fn order_questions(t: &TestGame, p: PlayerId) -> Vec<Vec<String>> {
    t.asked()
        .into_iter()
        .filter(|(q, _)| *q == p)
        .filter_map(|(_, d)| match d {
            Decision::ChooseReplacement { options } => Some(options),
            _ => None,
        })
        .collect()
}

/// The options of every question asked to `p` about which of several permanents entering
/// together has its effects applied next (each option names a permanent, "Thief of Blood
/// #61: ..."), leaving out the choices among one permanent's own effects.
fn entry_order_questions(t: &TestGame, p: PlayerId) -> Vec<Vec<String>> {
    order_questions(t, p)
        .into_iter()
        .filter(|options| {
            options
                .iter()
                .all(|o| o.split(": ").next().is_some_and(|h| h.contains(" #")))
        })
        .collect()
}

/// Puts `p`'s cards (moved from `from`, where they're put first) onto the battlefield at
/// the same time under `p`'s control, as one simultaneous zone change. Returns each card's
/// new object, if it entered as that move.
fn enter_from(t: &mut TestGame, p: PlayerId, from: Zone, names: &[&str]) -> Vec<Option<ObjectId>> {
    let objs: Vec<ObjectId> = names
        .iter()
        .map(|n| match from {
            Zone::Hand(_) => t.hand(p, n),
            _ => t.graveyard(p, n),
        })
        .collect();
    let moves = objs
        .iter()
        .map(|o| MoveEv {
            obj: *o,
            to: Zone::Battlefield,
            pos: LibraryPosition::Top,
            cause: MoveCause::Effect,
            by: Some(p),
            etb: EtbInfo {
                controller: Some(p),
                ..Default::default()
            },
            source: None,
        })
        .collect();
    let out = t.g.move_objects(moves);
    t.g.flush_events();
    t.settle();
    out
}

#[test]
fn ixidron_and_clone_entering_together_in_the_order_their_controller_chooses() {
    cr!("616.1", "616.1f", "614.12", "707.2");
    interp!("simultaneous-entries-ordered-by-controller");
    ruling!(
        "Ixidron",
        "If Ixidron and another creature are entering at the same time, the other creature enters face up."
    );
    ruling!(
        "Ixidron",
        "These values are copiable if an object becomes a copy of one of those creatures, and their normal values are not copiable."
    );
    supported("Ixidron");
    supported("Clone");
    // Ixidron: "As this creature enters, turn all other nontoken creatures face down."
    // Clone: "You may have this creature enter as a copy of any creature on the
    // battlefield." Ixidron's applied first (P0's choice, or by default the order of the
    // moves): the Bears are face down, and Clone copies a nameless 2/2. Clone's first: it
    // copies the Bears, which Ixidron then turns face down. Either way Clone, entering at
    // the same time, enters face up.
    for pick in [None, Some(0usize), Some(1)] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let bears = t.battlefield(P1, "Grizzly Bears");
        if let Some(i) = pick {
            t.answer(P0, DecisionKind::Replacement, Answer::Index(i));
        }
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[obj(bears)]);
        let ids = enter_together(&mut t, &[(P0, "Ixidron"), (P0, "Clone")]);
        let questions = order_questions(&t, P0);
        assert_eq!(questions.len(), 1, "{pick:?}: {questions:?}");
        assert_eq!(questions[0].len(), 2);
        assert!(questions[0][0].starts_with("Ixidron") && questions[0][1].starts_with("Clone"));
        let clone = ids[1];
        assert!(t.obj_now(bears).face_down, "{pick:?}");
        assert!(
            !t.obj_now(clone).face_down,
            "{pick:?}: Clone enters face up"
        );
        if pick == Some(1) {
            assert_eq!(t.obj_now(clone).chars.name, "Grizzly Bears");
        } else {
            assert_eq!(
                t.obj_now(clone).chars.name,
                "",
                "{pick:?}: a copy of a face-down 2/2"
            );
            assert_eq!(t.pt(clone), (2, 2));
        }
    }
}

#[test]
fn thieves_of_blood_entering_together_the_chosen_one_gets_the_counters() {
    cr!("616.1", "616.1f", "614.12");
    interp!("simultaneous-entries-ordered-by-controller");
    ruling!(
        "Thief of Blood",
        "If more than one Thief of Blood enters the battlefield at the same time, you can apply the replacement effect of any of them. That one will enter with additional +1/+1 counters due to its own ability and the others will not."
    );
    supported("Thief of Blood");
    // "As this creature enters, remove all counters from all permanents. This creature
    // enters with a +1/+1 counter on it for each counter removed this way." An invalid
    // answer gets the default, the order of the moves.
    for (answer, gets) in [
        (None, 0usize),
        (Some(0), 0),
        (Some(1), 1),
        (Some(2), 0),
        (Some(7), 0),
    ] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let bears = with_counters(&mut t, P1, "Grizzly Bears", 3);
        if let Some(i) = answer {
            t.answer(P0, DecisionKind::Replacement, Answer::Index(i));
        }
        let ids = enter_together(&mut t, &[(P0, "Thief of Blood"), (P0, "Thief of Blood")]);
        assert_eq!(order_questions(&t, P0).len(), 1, "{answer:?}");
        let got: Vec<u32> = ids.iter().map(|id| plus1(&t, *id)).collect();
        let mut want = vec![0, 0];
        want[gets] = 3;
        assert_eq!(got, want, "answer {answer:?}");
        assert_eq!(plus1(&t, bears), 0);
    }
}

#[test]
fn thief_of_blood_and_sin_entering_together_in_the_chosen_order() {
    cr!("616.1", "616.1f", "614.12");
    interp!("simultaneous-entries-ordered-by-controller");
    supported("Thief of Blood");
    supported("Sin, Unending Cataclysm");
    // Sin: "As Sin enters, remove all counters from any number of artifacts, creatures,
    // and enchantments. Sin enters with X +1/+1 counters on it, where X is twice the
    // number of counters removed this way." The Bears have three counters: whichever is
    // applied first takes them (Thief 3, or Sin 6), and the other removes none.
    for (pick, thief_n, sin_n) in [(0usize, 3, 0), (1, 0, 6)] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let bears = with_counters(&mut t, P1, "Grizzly Bears", 3);
        t.answer(P0, DecisionKind::Replacement, Answer::Index(pick));
        t.answer_choose(P0, &[obj(bears)]);
        let ids = enter_together(
            &mut t,
            &[(P0, "Thief of Blood"), (P0, "Sin, Unending Cataclysm")],
        );
        assert_eq!(order_questions(&t, P0).len(), 1);
        assert_eq!(plus1(&t, ids[0]), thief_n, "pick {pick}: Thief of Blood");
        assert_eq!(plus1(&t, ids[1]), sin_n, "pick {pick}: Sin");
        assert_eq!(plus1(&t, bears), 0);
    }
}

#[test]
fn sutured_ghoul_and_golgari_grave_troll_entering_together_in_the_chosen_order() {
    cr!("616.1", "616.1f", "614.12");
    interp!("simultaneous-entries-ordered-by-controller");
    supported("Sutured Ghoul");
    supported("Golgari Grave-Troll");
    // Sutured Ghoul: "As this creature enters, exile any number of creature cards from
    // your graveyard." Golgari Grave-Troll: "This creature enters with a +1/+1 counter on
    // it for each creature card in your graveyard." With three creature cards in P0's
    // graveyard and two exiled for the Ghoul: the Troll counts three if its effect is
    // applied first, one if the Ghoul's is.
    for (pick, troll_n) in [(0usize, 1u32), (1, 3)] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let a = t.graveyard(P0, "Grizzly Bears");
        let b = t.graveyard(P0, "Hill Giant");
        t.graveyard(P0, "Grizzly Bears");
        t.answer(P0, DecisionKind::Replacement, Answer::Index(pick));
        t.answer_choose(P0, &[obj(a), obj(b)]);
        let ids = enter_together(
            &mut t,
            &[(P0, "Sutured Ghoul"), (P0, "Golgari Grave-Troll")],
        );
        assert_eq!(order_questions(&t, P0).len(), 1);
        assert_eq!(plus1(&t, ids[1]), troll_n, "pick {pick}");
        assert_eq!(t.zone(a), mtg_engine::object::Zone::Exile, "pick {pick}");
        assert_eq!(
            t.pt(ids[0]),
            (5, 5),
            "the Ghoul: the Bears' and the Giant's, added up"
        );
    }
}

#[test]
fn no_order_question_when_no_entering_permanents_effect_changes_the_game() {
    cr!("616.1");
    interp!("simultaneous-entries-ordered-by-controller");
    supported("Breeding Pool");
    supported("Lakeside Shack");
    supported("Choked Estuary");
    supported("Voice of All");
    supported("Azorius Guildgate");
    // Choices, conditions and life payments (made once every choice is made) change
    // nothing as they're applied; "enters tapped" doesn't read the game. Thief of Blood
    // changes the game, but the Guildgate's effect neither reads nor changes it.
    let cases: [&[&str]; 3] = [
        &["Breeding Pool", "Lakeside Shack", "Choked Estuary"],
        &["Voice of All", "Voice of All"],
        &["Thief of Blood", "Azorius Guildgate"],
    ];
    for names in cases {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        with_counters(&mut t, P1, "Grizzly Bears", 1);
        let cards: Vec<(PlayerId, &str)> = names.iter().map(|n| (P0, *n)).collect();
        let ids = enter_together(&mut t, &cards);
        assert_eq!(ids.len(), names.len());
        assert!(order_questions(&t, P0).is_empty(), "{names:?}");
    }
}

#[test]
fn each_players_entering_permanents_have_their_effects_applied_in_apnap_order() {
    cr!("616.1", "101.4");
    interp!("simultaneous-entries-ordered-by-controller");
    interp!("entering-would-be-controller-chooses");
    supported("Thief of Blood");
    // P0's and P1's Thieves of Blood enter at the same time: each player chooses for the
    // permanent entering under their control, the active player first (CR 616.1: "If two
    // or more players have to make these choices at the same time, choices are made in
    // APNAP order."), so the active player's Thief removes the counters. Neither player
    // has two permanents to order.
    for active in [P0, P1] {
        let mut t = TestGame::new(2);
        t.set_step(active, Step::PrecombatMain);
        with_counters(&mut t, P1, "Grizzly Bears", 3);
        let ids = enter_together(&mut t, &[(P0, "Thief of Blood"), (P1, "Thief of Blood")]);
        assert!(order_questions(&t, P0).is_empty() && order_questions(&t, P1).is_empty());
        let (mine, theirs) = if active == P0 {
            (ids[0], ids[1])
        } else {
            (ids[1], ids[0])
        };
        assert_eq!(plus1(&t, mine), 3, "the active player's ({active:?})");
        assert_eq!(plus1(&t, theirs), 0);
    }
}

#[test]
fn token_copies_doubling_season_doubles_have_their_effects_applied_in_the_chosen_order() {
    cr!("616.1", "616.1g");
    cr_example!(
        "616.1g",
        "the effects of the two Voice of All tokens may be applied in either order"
    );
    interp!("simultaneous-entries-ordered-by-controller");
    supported("Doubling Season");
    supported("Cackling Counterpart");
    // The CR's example with Thief of Blood for Voice of All: Cackling Counterpart ("Create a
    // token that's a copy of target creature you control.") with Doubling Season. Doubling
    // Season's effect is applied first (two tokens), then the two tokens' "as this enters"
    // effects in the order P0 chooses: the one applied first removes the Bears' three
    // counters and enters with them, doubled by Doubling Season; the other gets none.
    for pick in [0usize, 1] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let bears = with_counters(&mut t, P1, "Grizzly Bears", 3);
        t.battlefield(P0, "Doubling Season");
        let thief = t.battlefield(P0, "Thief of Blood");
        t.answer(P0, DecisionKind::Replacement, Answer::Index(pick));
        cast_new(&mut t, P0, "Cackling Counterpart", &[obj(thief)]);
        t.resolve_all();
        let questions = entry_order_questions(&t, P0);
        assert_eq!(questions.len(), 1, "{questions:?}");
        assert_eq!(questions[0].len(), 2);
        let tokens: Vec<ObjectId> = t
            .named_on_battlefield("Thief of Blood")
            .into_iter()
            .filter(|id| *id != thief && t.obj_now(*id).is_token())
            .collect();
        assert_eq!(tokens.len(), 2, "Doubling Season: two tokens");
        // The token chosen ("Thief of Blood #68: ...") got the counters.
        let id_of = |option: &str| -> ObjectId {
            let n = option.split(": ").next().and_then(|h| h.rsplit('#').next());
            ObjectId(n.and_then(|n| n.parse().ok()).expect("an object number"))
        };
        let chosen = t.g.current(id_of(&questions[0][pick]));
        let other = t.g.current(id_of(&questions[0][1 - pick]));
        assert!(tokens.contains(&chosen) && tokens.contains(&other));
        assert_eq!(plus1(&t, chosen), 6, "pick {pick}: 3 removed, doubled");
        assert_eq!(plus1(&t, other), 0, "pick {pick}");
        assert_eq!(plus1(&t, bears), 0);
    }
}

#[test]
fn mox_diamond_and_golgari_grave_troll_entering_together_in_the_chosen_order() {
    cr!("616.1", "616.1f", "614.12");
    interp!("simultaneous-entries-ordered-by-controller");
    supported("Mox Diamond");
    supported("Golgari Grave-Troll");
    supported("Dryad Arbor");
    // Mox Diamond: "If this artifact would enter, you may discard a land card instead. If
    // you do, put this artifact onto the battlefield. ..." What's done instead is done as
    // the effect is applied: discarding Dryad Arbor (a land card and a creature card)
    // before the Troll's effect is applied gives the Troll one more creature card to count.
    for (pick, troll_n) in [(None, 3u32), (Some(0usize), 3), (Some(1), 2)] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        t.graveyard(P0, "Grizzly Bears");
        t.graveyard(P0, "Hill Giant");
        let arbor = t.hand(P0, "Dryad Arbor");
        if let Some(i) = pick {
            t.answer(P0, DecisionKind::Replacement, Answer::Index(i));
        }
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[obj(arbor)]);
        enter_from(
            &mut t,
            P0,
            Zone::Hand(P0),
            &["Mox Diamond", "Golgari Grave-Troll"],
        );
        let questions = entry_order_questions(&t, P0);
        assert_eq!(questions.len(), 1, "{pick:?}: {questions:?}");
        assert!(questions[0][0].starts_with("Mox Diamond"));
        assert_eq!(t.named_on_battlefield("Mox Diamond").len(), 1);
        assert_eq!(t.zone(arbor), Zone::Graveyard(P0));
        let troll = t.named_on_battlefield("Golgari Grave-Troll");
        assert_eq!(plus1(&t, troll[0]), troll_n, "{pick:?}");
    }
}

#[test]
fn rhythm_of_the_wild_returned_as_another_creature_enters_still_gives_it_riot() {
    cr!("614.12", "616.1f", "702.136a");
    interp!("simultaneous-entries-ordered-by-controller");
    ruling!(
        "Rhythm of the Wild",
        "If Rhythm of the Wild leaves the battlefield at the same time that a nontoken creature enters the battlefield (most likely because that creature has a replacement effect, such as that of Rescuer Sphinx), that creature still gets a +1/+1 counter or haste."
    );
    supported("Rhythm of the Wild");
    supported("Rescuer Sphinx");
    // Rescuer Sphinx ("As this creature enters, you may return a nonland permanent you
    // control to its owner's hand. If you do, it enters with a +1/+1 counter on it.") and
    // Grizzly Bears enter together; the Sphinx returns Rhythm of the Wild ("Nontoken
    // creatures you control have riot."). Riot applied to the Bears as the event was
    // proposed: they still get it, whichever permanent's effects are applied first.
    for pick in [0usize, 1] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let rhythm = t.battlefield(P0, "Rhythm of the Wild");
        t.answer(P0, DecisionKind::Replacement, Answer::Index(pick));
        // Riot's choices (a +1/+1 counter) and the Sphinx's "you may return".
        for _ in 0..3 {
            t.answer_yes(P0, true);
        }
        t.answer_choose(P0, &[obj(rhythm)]);
        let out = enter_from(
            &mut t,
            P0,
            Zone::Graveyard(P0),
            &["Rescuer Sphinx", "Grizzly Bears"],
        );
        assert!(t.in_hand(P0, "Rhythm of the Wild"), "pick {pick}: returned");
        let bears = out[1].expect("the Bears entered");
        assert!(
            plus1(&t, bears) == 1 || crate::r_s06_common::has_kw(&t, bears, KeywordKind::Haste),
            "pick {pick}: the Bears have riot's counter or haste"
        );
    }
}

#[test]
fn an_effect_whose_source_another_entering_permanent_turns_face_down_still_applies() {
    cr!("616.1", "616.1f", "614.12");
    interp!("simultaneous-entries-ordered-by-controller");
    supported("Grumgully, the Generous");
    supported("Ixidron");
    // Grumgully, the Generous: "Each other non-Human creature you control enters with an
    // additional +1/+1 counter on it." It stays on the battlefield, but Ixidron ("As this
    // creature enters, turn all other nontoken creatures face down."), entering with the
    // Bears, turns it face down (a face-down 2/2 has no abilities). Grumgully's effect was
    // determined as the event was proposed, so it still applies, whichever permanent's
    // effects come first (decided by the user, Nov 20: before, the Bears got the counter only
    // if their effects were applied first). The player is still asked: an effect another
    // permanent generates can count something about that permanent (Master Biomancer).
    for pick in [None, Some(0usize), Some(1)] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let grumgully = t.battlefield(P0, "Grumgully, the Generous");
        if let Some(i) = pick {
            t.answer(P0, DecisionKind::Replacement, Answer::Index(i));
        }
        let ids = enter_together(&mut t, &[(P0, "Ixidron"), (P0, "Grizzly Bears")]);
        let questions = entry_order_questions(&t, P0);
        assert_eq!(questions.len(), 1, "{pick:?}: {questions:?}");
        assert!(questions[0][1].starts_with("Grizzly Bears"));
        assert!(t.obj_now(grumgully).face_down);
        assert!(!t.obj_now(ids[1]).face_down, "entering at the same time");
        assert_eq!(plus1(&t, ids[1]), 1, "{pick:?}");
    }
}

#[test]
fn an_effect_whose_source_turns_face_down_applies_whether_or_not_a_permanent_leaves() {
    cr!("616.1f", "614.12");
    interp!("simultaneous-entries-ordered-by-controller");
    supported("Grumgully, the Generous");
    supported("Ixidron");
    supported("Rescuer Sphinx");
    // Ixidron, Rescuer Sphinx and Grizzly Bears enter together, with Grumgully, the Generous
    // and Ornithopter on the battlefield. Ixidron turns Grumgully and Ornithopter face down;
    // the Sphinx ("As this creature enters, you may return a nonland permanent you control
    // to its owner's hand. If you do, this creature enters with a +1/+1 counter on it.")
    // returns Ornithopter or nothing. The Bears get Grumgully's counter either way, in
    // whichever order the effects are applied (before, the engine added back the effects
    // determined as the event was proposed only once a permanent had left the battlefield:
    // with the Sphinx returning Ornithopter the Bears got 1, without it 0).
    for returns in [false, true] {
        for pick in [None, Some(0usize), Some(1), Some(2)] {
            let mut t = TestGame::new(2);
            t.set_step(P0, Step::PrecombatMain);
            let grumgully = t.battlefield(P0, "Grumgully, the Generous");
            let thopter = t.battlefield(P0, "Ornithopter");
            if let Some(i) = pick {
                t.answer(P0, DecisionKind::Replacement, Answer::Index(i));
            }
            t.answer_yes(P0, returns);
            if returns {
                t.answer_choose(P0, &[obj(thopter)]);
            }
            let ids = enter_together(
                &mut t,
                &[
                    (P0, "Ixidron"),
                    (P0, "Rescuer Sphinx"),
                    (P0, "Grizzly Bears"),
                ],
            );
            let case = format!("returns {returns}, pick {pick:?}");
            assert!(!entry_order_questions(&t, P0).is_empty(), "{case}");
            assert!(t.obj_now(grumgully).face_down, "{case}");
            assert_eq!(t.in_hand(P0, "Ornithopter"), returns, "{case}");
            assert_eq!(plus1(&t, ids[2]), 1, "{case}: the Bears");
            // The Sphinx: its own counter if it returned a permanent, and Grumgully's.
            assert_eq!(plus1(&t, ids[1]), 1 + returns as u32, "{case}: the Sphinx");
        }
    }
}

#[test]
fn a_lone_rescuer_sphinx_returning_grumgully_still_gets_its_counter() {
    cr!("614.12", "616.1f");
    interp!("simultaneous-entries-ordered-by-controller");
    supported("Grumgully, the Generous");
    supported("Rescuer Sphinx");
    // Rescuer Sphinx enters alone and returns Grumgully, the Generous: Grumgully's effect,
    // determined as the event was proposed, still applies whichever of the two effects is
    // applied first, as Rhythm of the Wild's riot does in its ruling (the next test).
    for first in [0usize, 1] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let grumgully = t.battlefield(P0, "Grumgully, the Generous");
        t.answer(P0, DecisionKind::Replacement, Answer::Index(first));
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[obj(grumgully)]);
        let ids = enter_together(&mut t, &[(P0, "Rescuer Sphinx")]);
        assert!(t.in_hand(P0, "Grumgully, the Generous"), "first {first}");
        assert_eq!(plus1(&t, ids[0]), 2, "first {first}");
    }
}

#[test]
fn a_lone_rescuer_sphinx_returning_rhythm_of_the_wild_still_has_riot() {
    cr!("614.12", "616.1f", "702.136a");
    interp!("simultaneous-entries-ordered-by-controller");
    ruling!(
        "Rhythm of the Wild",
        "If Rhythm of the Wild leaves the battlefield at the same time that a nontoken creature enters the battlefield (most likely because that creature has a replacement effect, such as that of Rescuer Sphinx), that creature still gets a +1/+1 counter or haste."
    );
    supported("Rhythm of the Wild");
    supported("Rescuer Sphinx");
    // The ruling's own case: Rescuer Sphinx enters alone and returns Rhythm of the Wild
    // ("Nontoken creatures you control have riot."). Riot applied to the Sphinx as the event
    // was proposed, so the Sphinx still gets riot's counter or haste, whichever of its two
    // effects is applied first, besides its own counter for returning a permanent.
    for first in [0usize, 1] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let rhythm = t.battlefield(P0, "Rhythm of the Wild");
        t.answer(P0, DecisionKind::Replacement, Answer::Index(first));
        // The Sphinx's "you may return" and riot's choice (a +1/+1 counter).
        for _ in 0..2 {
            t.answer_yes(P0, true);
        }
        t.answer_choose(P0, &[obj(rhythm)]);
        let sphinx = enter_together(&mut t, &[(P0, "Rescuer Sphinx")])[0];
        assert!(t.in_hand(P0, "Rhythm of the Wild"), "first {first}");
        let n = plus1(&t, sphinx);
        assert!(
            n == 2 || (n == 1 && crate::r_s06_common::has_kw(&t, sphinx, KeywordKind::Haste)),
            "first {first}: its own counter and riot's counter or haste ({n} counters)\n{}",
            t.dump_log()
        );
    }
}

#[test]
fn an_effect_determined_at_the_proposal_no_longer_applies_once_the_permanent_enters_as_something_else(
) {
    cr!("614.12", "616.1f");
    interp!("simultaneous-entries-ordered-by-controller");
    ruling!(
        "Metallic Mimic",
        "The choice of creature type is made as Metallic Mimic enters the battlefield. Players can't respond to this choice. Metallic Mimic's second ability starts applying immediately."
    );
    supported("Grumgully, the Generous");
    supported("Metallic Mimic");
    // Metallic Mimic ("As this creature enters, choose a creature type. This creature is the
    // chosen type in addition to its other types.") enters naming Human, with Grumgully, the
    // Generous ("Each other non-Human creature you control enters with an additional +1/+1
    // counter on it.") on the battlefield. As the event is proposed both effects apply (with
    // no type chosen the Mimic is a non-Human Shapeshifter), and P0 orders them (616.1):
    // Grumgully's first, the Mimic gets the counter and then becomes a Human; the Mimic's
    // choice first, it's a Human as it would exist on the battlefield, so Grumgully's effect
    // no longer applies to it (614.12, 616.1f), although it was determined as the event was
    // proposed: what happens to an effect's source doesn't stop it, what the permanent now
    // enters as does.
    let human = mtg_engine::types::subtype_lists()
        .creature
        .iter()
        .position(|x| x == "Human")
        .expect("a creature type");
    for (pick, counters) in [(None, 1u32), (Some(0usize), 1), (Some(1), 0)] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        t.battlefield(P0, "Grumgully, the Generous");
        if let Some(i) = pick {
            t.answer(P0, DecisionKind::Replacement, Answer::Index(i));
        }
        t.answer(P0, DecisionKind::Option, Answer::Index(human));
        let mimic = enter_together(&mut t, &[(P0, "Metallic Mimic")])[0];
        let mimic = t.g.current(mimic);
        let questions = order_questions(&t, P0);
        assert_eq!(questions.len(), 1, "{pick:?}: {questions:?}");
        assert!(questions[0][0].starts_with("Grumgully"), "{questions:?}");
        assert!(questions[0][1].starts_with("Metallic Mimic"), "{questions:?}");
        assert!(t.obj_now(mimic).chars.has_subtype("Human"), "{pick:?}");
        assert_eq!(plus1(&t, mimic), counters, "{pick:?}\n{}", t.dump_log());
    }
}

#[test]
fn cards_from_both_graveyards_entering_under_your_control_are_ordered_by_you() {
    cr!("616.1", "616.1f");
    interp!("simultaneous-entries-ordered-by-controller");
    interp!("entering-would-be-controller-chooses");
    supported("Rise of the Dark Realms");
    // Rise of the Dark Realms: "Put all creature cards from all graveyards onto the
    // battlefield under your control." P1's Thief of Blood and P0's Sin, Unending Cataclysm
    // enter together under P0's control: P0, the player they'd enter under the control of,
    // orders both (CR 616.1), and P1 is asked nothing. Whichever P0 picks first takes the
    // Bears' three counters (Thief 3, or Sin twice that).
    let mut seen = Vec::new();
    for pick in [0usize, 1] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let bears = with_counters(&mut t, P1, "Grizzly Bears", 3);
        t.graveyard(P1, "Thief of Blood");
        t.graveyard(P0, "Sin, Unending Cataclysm");
        t.answer(P0, DecisionKind::Replacement, Answer::Index(pick));
        t.answer_choose(P0, &[obj(bears)]);
        cast_new(&mut t, P0, "Rise of the Dark Realms", &[]);
        t.resolve_all();
        let questions = entry_order_questions(&t, P0);
        assert_eq!(questions.len(), 1, "pick {pick}: {questions:?}");
        assert!(order_questions(&t, P1).is_empty());
        let thief = t.named_on_battlefield("Thief of Blood");
        let sin = t.named_on_battlefield("Sin, Unending Cataclysm");
        assert_eq!((thief.len(), sin.len()), (1, 1));
        assert_eq!(
            controller(&mut t, thief[0]),
            P0,
            "P1's card, under P0's control"
        );
        let thief_first = questions[0][pick].starts_with("Thief of Blood");
        seen.push(thief_first);
        let (thief_n, sin_n) = if thief_first { (3, 0) } else { (0, 6) };
        assert_eq!(plus1(&t, thief[0]), thief_n, "pick {pick}: Thief of Blood");
        assert_eq!(plus1(&t, sin[0]), sin_n, "pick {pick}: Sin");
        assert_eq!(plus1(&t, bears), 0);
    }
    assert_ne!(seen[0], seen[1], "each pick puts a different one first");
}
