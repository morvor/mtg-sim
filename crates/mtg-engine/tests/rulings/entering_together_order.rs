//! Item `ruling-conflicts-12`, question 2: permanents entering the battlefield at the same
//! time have their "as [this] enters" / "enters with" replacement effects applied in the
//! order their controller chooses (CR 616.1, 616.1f, the 616.1g example; Thief of Blood's
//! ruling), when the order can matter: one of them changes the game as it's applied and
//! another reads or changes it too. The default is the order of the moves
//! (interpretation `simultaneous-entries-ordered-by-controller`).

use crate::r_residue_counters_common::*;
use mtg_engine::decision::{Answer, Decision};
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

#[test]
fn ixidron_and_clone_entering_together_in_the_order_their_controller_chooses() {
    cr!("616.1", "616.1f", "614.12", "707.2");
    cr_example!(
        "616.1g",
        "the effects of the two Voice of All tokens may be applied in either order"
    );
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
    for (answer, gets) in [(None, 0usize), (Some(0), 0), (Some(1), 1), (Some(7), 0)] {
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
    cr!("616.1", "614.12b");
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
