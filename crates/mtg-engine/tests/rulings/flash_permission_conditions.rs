//! Item `ruling-conflicts-13` — a condition on casting a spell as though it had flash is
//! judged as the player begins to cast the spell (CR 601.3), and the player may continue
//! to cast it as though it had flash even if those conditions stop being met (CR 601.6a,
//! added as 601.4a by the CR effective 2022-04-29), whatever lets the card be cast from
//! where it is. Swift Reckoning's ruling of 2018-12-07 ("you need two other instant and/or
//! sorcery cards in your graveyard") is outdated by that rule (docs/RULING_DISCREPANCIES.md,
//! `docs/rulings-exemptions/rulings-P011.tsv`); the main test is in
//! `r_residue_misc_1_stack.rs` (`swift_reckoning_cast_from_the_graveyard_counts_itself_as_casting_begins`).

use crate::r_residue_misc_1_common::*;
use mtg_engine::decision::Action;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// The ways the list of legal actions offers to cast `card` now.
fn offered(t: &mut TestGame, p: PlayerId, card: ObjectId) -> Vec<CastMethod> {
    t.g.turn.priority = Some(p);
    t.g.recompute();
    t.g.legal_actions(p)
        .into_iter()
        .filter_map(|a| match a {
            Action::Cast { card: c, method } if c == card => Some(method),
            _ => None,
        })
        .collect()
}

/// P1's tapped Grizzly Bears, Swift Reckoning's target.
fn tapped_bears(t: &mut TestGame) -> ObjectId {
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.objects[bears.0 as usize].tapped = true;
    bears
}

// ---------------------------------------------------------------------------------------
// Swift Reckoning: "Spell mastery — If there are two or more instant and/or sorcery cards
// in your graveyard, you may cast this spell as though it had flash. ... Destroy target
// tapped creature."
// ---------------------------------------------------------------------------------------

#[test]
fn swift_reckoning_counts_itself_whatever_lets_it_be_cast_from_a_graveyard() {
    cr!("601.3", "601.6a", "702.34a");
    interp!("flash-permission-conditions-judged-as-casting-begins");
    ruling!(
        "Swift Reckoning",
        "The number of instant and/or sorcery cards in your graveyard matters only as you begin to cast Swift Reckoning."
    );
    supported("Swift Reckoning");
    supported("Lier, Disciple of the Drowned");
    supported("Kess, Dissident Mage");
    // Lier ("Each instant and sorcery card in your graveyard has flashback. The flashback
    // cost is equal to that card's mana cost."), on P1's turn; Kess ("Once during each of
    // your turns, you may cast an instant or sorcery spell from your graveyard. If a spell
    // cast this way would be put into your graveyard, exile it instead."), in P0's
    // beginning of combat. As the casting begins, the card is in the graveyard and counts
    // itself: with one other instant or sorcery card there it's offered and cast, though
    // moving it to the stack leaves one; with none, neither.
    for kess in [false, true] {
        for others in [0usize, 1] {
            let mut t = TestGame::new(2);
            t.battlefield(
                P0,
                if kess {
                    "Kess, Dissident Mage"
                } else {
                    "Lier, Disciple of the Drowned"
                },
            );
            let reck = t.graveyard(P0, "Swift Reckoning");
            for _ in 0..others {
                t.graveyard(P0, "Shock");
            }
            let bears = tapped_bears(&mut t);
            if kess {
                t.set_step(P0, Step::BeginningOfCombat);
            } else {
                t.set_step(P1, Step::PrecombatMain);
            }
            t.lands(P0, "Plains", 2);
            let ways = offered(&mut t, P0, reck);
            let case = format!("Kess {kess}, others {others}: {ways:?}");
            assert_eq!(ways.is_empty(), others == 0, "{case}");
            let method = if kess {
                CastMethod::Normal
            } else {
                CastMethod::Keyword(KeywordKind::Flashback)
            };
            let cast = t.cast(P0, reck).method(method).target(bears).try_go();
            assert_eq!(cast.is_ok(), others == 1, "{case}: {cast:?}");
            if others == 1 {
                t.resolve_all();
                assert!(!t.on_battlefield(bears), "{case}");
                assert_eq!(t.zone(reck), Zone::Exile, "{case}");
            } else {
                assert_eq!(t.zone(reck), Zone::Graveyard(P0), "{case}");
                assert!(t.on_battlefield(bears), "{case}");
            }
        }
    }
}

#[test]
fn swift_reckonings_count_isnt_judged_again_while_its_costs_are_paid() {
    cr!("601.2g", "601.6", "601.6a");
    interp!("flash-permission-conditions-judged-as-casting-begins");
    supported("Swift Reckoning");
    supported("Molt Tender");
    // Cast with Snapcaster Mage's flashback on P1's turn, with one other instant card in
    // the graveyard (two as the casting begins, one once Swift Reckoning is on the stack),
    // which Molt Tender's mana ability ("{T}, Exile a card from your graveyard: Add one
    // mana of any color.") exiles to pay: none left, and the casting goes on.
    let mut t = TestGame::new(2);
    let reck = t.graveyard(P0, "Swift Reckoning");
    let shock = t.graveyard(P0, "Shock");
    let bears = tapped_bears(&mut t);
    t.set_step(P1, Step::PrecombatMain);
    t.answer_targets(P0, &[obj(reck)]);
    enter(&mut t, P0, "Snapcaster Mage");
    t.resolve_all();
    t.battlefield(P0, "Molt Tender");
    t.lands(P0, "Plains", 1);
    t.answer_choose(P0, &[obj(shock)]);
    t.cast(P0, reck)
        .method(CastMethod::Keyword(KeywordKind::Flashback))
        .target(bears)
        .go();
    assert_eq!(t.zone(shock), Zone::Exile);
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert_eq!(t.zone(reck), Zone::Exile);
}

// ---------------------------------------------------------------------------------------
// Serpent of the Pass: "If there are three or more Lesson cards in your graveyard, you may
// cast this spell as though it had flash. / This spell costs {1} less to cast for each
// noncreature, nonland card in your graveyard."
// ---------------------------------------------------------------------------------------

#[test]
fn serpent_of_the_pass_counts_lesson_cards_as_its_casting_begins() {
    cr!("601.3", "601.2g", "601.6");
    interp!("flash-permission-conditions-judged-as-casting-begins");
    supported("Serpent of the Pass");
    supported("Environmental Sciences");
    // On P1's turn, with two or three Lesson cards (Environmental Sciences, a sorcery) in
    // P0's graveyard and five Islands: it costs {3}{U}{U} or {2}{U}{U}, affordable either
    // way; only with three may it be cast as though it had flash.
    for lessons in [2usize, 3] {
        let mut t = TestGame::new(2);
        for _ in 0..lessons {
            t.graveyard(P0, "Environmental Sciences");
        }
        let serpent = t.hand(P0, "Serpent of the Pass");
        t.set_step(P1, Step::PrecombatMain);
        t.lands(P0, "Island", 5);
        let ways = offered(&mut t, P0, serpent);
        assert_eq!(ways.is_empty(), lessons < 3, "{lessons}: {ways:?}");
        let cast = t.cast(P0, serpent).try_go();
        assert_eq!(cast.is_ok(), lessons == 3, "{lessons}: {cast:?}");
        if lessons == 3 {
            t.resolve_all();
            assert_eq!(t.named_on_battlefield("Serpent of the Pass").len(), 1);
        }
    }
    // Three Lesson cards as the casting begins, one of them exiled to pay ({2}{U}{U}: three
    // Islands and Molt Tender's mana ability): two left, and the casting goes on.
    let mut t = TestGame::new(2);
    let lessons: Vec<ObjectId> = (0..3)
        .map(|_| t.graveyard(P0, "Environmental Sciences"))
        .collect();
    let serpent = t.hand(P0, "Serpent of the Pass");
    t.set_step(P1, Step::PrecombatMain);
    t.battlefield(P0, "Molt Tender");
    t.lands(P0, "Island", 3);
    t.answer_choose(P0, &[obj(lessons[0])]);
    t.cast(P0, serpent).go();
    assert_eq!(t.zone(lessons[0]), Zone::Exile);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Serpent of the Pass").len(), 1);
}
