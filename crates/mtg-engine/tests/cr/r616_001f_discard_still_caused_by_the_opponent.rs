//! A discard an opponent's spell causes is still that spell's doing once another
//! replacement effect has changed where the card goes (CR 614.6, 616.1f): with Darksteel
//! Colossus's own "If Darksteel Colossus would be put into a graveyard from anywhere,
//! reveal Darksteel Colossus and shuffle it into its owner's library instead." applied
//! first, Nephalia Academy's "If a spell or ability an opponent controls causes you to
//! discard a card, you may reveal that card and put it on top of your library instead of
//! putting it anywhere else." still applies to the discard Mind Rot causes. So it does with
//! the discarding player's own Rest in Peace or the card's madness applied first (each
//! exiles the card, which is then "exiled with" that effect's source, CR 607.2b).

use crate::r614_audit_common::*;
use mtg_engine::decision::{Agent, Answer, Decision, PassiveAgent};
use mtg_engine::game::Game;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Wraps `p`'s agent: a choice among replacement effects (`Decision::ChooseReplacement`)
/// takes the option containing `effect`. Every decision is still logged by the scripted
/// agent, which answers the others.
fn applies_first(t: &mut TestGame, p: PlayerId, effect: &'static str) {
    struct First {
        inner: Box<dyn Agent>,
        effect: &'static str,
    }
    impl Agent for First {
        fn decide(&mut self, g: &Game, p: PlayerId, d: &Decision) -> Answer {
            let scripted = self.inner.decide(g, p, d);
            match d {
                Decision::ChooseReplacement { options } => options
                    .iter()
                    .position(|o| o.contains(self.effect))
                    .map_or(scripted, Answer::Index),
                _ => scripted,
            }
        }
    }
    let mut agents = t.g.agents.0.lock().unwrap();
    let inner = std::mem::replace(&mut agents[p.idx()], Box::new(PassiveAgent));
    agents[p.idx()] = Box::new(First { inner, effect });
}

#[test]
fn nephalia_academy_still_applies_to_a_darksteel_colossus_its_own_effect_shuffles_away() {
    cr!("616.1", "616.1e", "616.1f", "614.6");
    ruling!(
        "Nephalia Academy",
        "When you apply Nephalia Academy’s effect, the card was still discarded."
    );
    supported("Nephalia Academy");
    supported("Darksteel Colossus");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Nephalia Academy");
    let colossus = t.hand(P0, "Darksteel Colossus");
    // P0 applies the Colossus's effect first, then accepts Nephalia Academy's.
    applies_first(&mut t, P0, "Darksteel Colossus:");
    t.answer_yes(P0, true);
    t.set_step(P1, Step::PrecombatMain);
    cast_resolve(&mut t, P1, "Mind Rot", &[Entity::Player(P0)]);
    let choices = replacement_choices(&t);
    assert_eq!(choices.len(), 1, "{choices:?}");
    assert_eq!(choices[0].0, P0);
    assert_eq!(choices[0].1.len(), 2, "{choices:?}");
    let offered: Vec<String> = t
        .asked()
        .into_iter()
        .filter_map(|(p, d)| match d {
            Decision::YesNo { prompt, .. } if p == P0 && prompt.contains("Nephalia Academy") => {
                Some(prompt)
            }
            _ => None,
        })
        .collect();
    assert_eq!(
        offered.len(),
        1,
        "Nephalia Academy's effect is offered once the Colossus's has been applied: {:?}",
        t.asked()
    );
    let c = t.g.current(colossus);
    assert_eq!(t.zone(c), Zone::Library(P0));
    assert_eq!(
        t.g.players[P0.idx()].library.last(),
        Some(&c),
        "on top of P0's library"
    );
}

/// Regression test for an engine bug found triaging `replacement.rs:2206:49`. P0's own Rest
/// in Peace ("If a card or token would be put into a graveyard from anywhere, exile it
/// instead.") applied first to the discard P1's Mind Rot causes made Rest in Peace the
/// move's source (to link the exiled card to it, CR 607.2b), which is also what Nephalia
/// Academy's "If a spell or ability an opponent controls causes you to discard a card"
/// reads as the discard's cause, so Nephalia Academy's effect was no longer offered. The
/// discard is still Mind Rot's doing (CR 614.6), and "instead of putting it anywhere else"
/// applies to a card going to exile (CR 616.1f). The link is now kept apart from the
/// move's cause (`EtbInfo::exiled_with`).
#[test]
fn nephalia_academy_still_applies_to_a_discard_its_controllers_rest_in_peace_exiles() {
    cr!("616.1f", "614.6");
    ruling!(
        "Nephalia Academy",
        "When you apply Nephalia Academy’s effect, the card was still discarded."
    );
    supported("Rest in Peace");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Nephalia Academy");
    t.battlefield(P0, "Rest in Peace");
    let bears = t.hand(P0, "Grizzly Bears");
    applies_first(&mut t, P0, "Rest in Peace:");
    t.answer_yes(P0, true);
    t.set_step(P1, Step::PrecombatMain);
    cast_resolve(&mut t, P1, "Mind Rot", &[Entity::Player(P0)]);
    let choices = replacement_choices(&t);
    assert_eq!(choices.len(), 1, "{choices:?}");
    let offered = t
        .asked()
        .into_iter()
        .filter(|(p, d)| {
            *p == P0
                && matches!(d, Decision::YesNo { prompt, .. } if prompt.contains("Nephalia Academy"))
        })
        .count();
    assert_eq!(
        offered,
        1,
        "Nephalia Academy's effect is offered once Rest in Peace's has been applied: {:?}",
        t.asked()
    );
    let b = t.g.current(bears);
    assert_eq!(t.zone(b), Zone::Library(P0));
    assert_eq!(t.g.players[P0.idx()].library.last(), Some(&b));
}

/// The same with the discarded card's own madness ability applied first ("If a player
/// would discard this card, that player discards it, but exiles it instead of putting it
/// into their graveyard", CR 702.35a): the discard is still Mind Rot's doing, so Nephalia
/// Academy's effect is still offered, and applied it puts Biting Rain on top of P0's
/// library, where madness's "When this card is exiled this way" doesn't see it: P0 isn't
/// offered to cast it (Nephalia Academy's ruling).
#[test]
fn nephalia_academy_still_applies_to_a_discard_madness_exiles_and_then_it_cant_be_cast() {
    cr!("616.1f", "614.6", "702.35a");
    ruling!(
        "Nephalia Academy",
        "If you discard a card with madness, you’ll choose whether to exile it or put it on top of your library. If you put it on top of your library, you can’t cast it using madness."
    );
    supported("Nephalia Academy");
    supported("Biting Rain");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Nephalia Academy");
    let rain = t.hand(P0, "Biting Rain");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Swamp", 3);
    applies_first(&mut t, P0, "Madness");
    t.answer_yes(P0, true);
    t.set_step(P1, Step::PrecombatMain);
    cast_resolve(&mut t, P1, "Mind Rot", &[Entity::Player(P0)]);
    let choices = replacement_choices(&t);
    assert_eq!(choices.len(), 1, "{choices:?}");
    assert_eq!(choices[0].1.len(), 2, "madness or Nephalia Academy: {choices:?}");
    let asked = |what: &str| {
        t.asked()
            .into_iter()
            .filter(|(p, d)| {
                *p == P0 && matches!(d, Decision::YesNo { prompt, .. } if prompt.contains(what))
            })
            .count()
    };
    assert_eq!(
        asked("Nephalia Academy"),
        1,
        "Nephalia Academy's effect is offered once madness's has been applied: {:?}",
        t.asked()
    );
    assert_eq!(asked("madness cost"), 0, "{:?}", t.asked());
    let r = t.g.current(rain);
    assert_eq!(t.zone(r), Zone::Library(P0));
    assert_eq!(t.g.players[P0.idx()].library.last(), Some(&r));
    assert!(t.on_battlefield(bears), "Biting Rain wasn't cast");
}
