//! Rulings batch S27 — alternative and additional costs (CR 118.8, 118.9): a spell cast
//! with a permission still has all its costs paid, including additional costs, and may
//! be cast for an alternative cost (CR 601.2b, 601.2f); a spell cast without paying its
//! mana cost may still have its additional costs paid (CR 118.9a); a card with several
//! flashback abilities may be cast for any of their costs; and a copy of a card that's
//! cast by paying its costs has X chosen as normal (CR 707.12, 107.3a).

use crate::r_s01_common::*;
use crate::r_s07_common::cast_methods;
use crate::r_s22_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn elven_chorus_casts_from_the_library_paying_kicker_or_an_alternative_cost() {
    cr!("601.2b", "601.2f", "118.8", "118.9", "702.33a", "702.74a");
    ruling!(
        "Elven Chorus",
        "You'll still pay all costs for the spell, including additional costs. You may also pay alternative costs if any are available."
    );
    supported("Elven Chorus");
    supported("Kavu Titan");
    supported("Mulldrifter");
    // "You may cast creature spells from the top of your library." Kavu Titan ({1}{G},
    // kicker {2}{G}: "If this creature was kicked, it enters with three +1/+1 counters on
    // it and with trample.") is cast kicked from the top.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Elven Chorus");
    let titan = t.library_top(P0, "Kavu Titan");
    t.lands(P0, "Forest", 2);
    t.lands(P0, "Wastes", 3);
    t.cast(P0, titan).kicked(true).go();
    assert_eq!(tapped_lands(&t, P0), 5);
    t.resolve_all();
    assert!(t.on_battlefield(titan));
    assert_eq!(t.counters(titan, "+1/+1"), 3);
    // Mulldrifter is cast from the top for its evoke cost {2}{U}: it draws two cards and
    // is sacrificed.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Elven Chorus");
    let drifter = t.library_top(P0, "Mulldrifter");
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 2);
    let hand = t.hand_size(P0);
    t.cast(P0, drifter)
        .method(CastMethod::Keyword(KeywordKind::Evoke))
        .go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Mulldrifter"));
    assert_eq!(t.hand_size(P0), hand + 2);
}

#[test]
fn soul_spike_is_cast_for_its_alternative_cost_with_an_additional_cost_paid() {
    cr!("118.9", "118.9d", "118.8", "601.2b", "702.56a");
    ruling!(
        "Soul Spike",
        "You may pay the alternative cost rather than the card’s mana cost. Any additional costs are paid as normal."
    );
    supported("Soul Spike");
    supported("Djinn Illuminatus");
    // "You may exile two black cards from your hand rather than pay this spell's mana
    // cost. Soul Spike deals 4 damage to any target and you gain 4 life." Djinn
    // Illuminatus gives it replicate (an additional cost) equal to its mana cost,
    // {5}{B}{B}, paid once: two Soul Spikes resolve.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Djinn Illuminatus");
    let spike = t.hand(P0, "Soul Spike");
    let ritual = t.hand(P0, "Dark Ritual");
    let duress = t.hand(P0, "Duress");
    let bolt = t.hand(P0, "Lightning Bolt");
    t.lands(P0, "Swamp", 2);
    t.lands(P0, "Wastes", 5);
    let alt = cast_methods(&mut t, P0, spike)
        .into_iter()
        .find(|m| matches!(m, CastMethod::Alternative(_)))
        .expect("the alternative cost is available");
    t.answer_choose(P0, &[Entity::Object(ritual), Entity::Object(duress)]);
    t.answer(P0, DecisionKind::OptionalCost, Answer::Number(1));
    t.cast(P0, spike).method(alt).target(Entity::Player(P1)).go();
    assert_eq!(t.zone(ritual), Zone::Exile);
    assert_eq!(t.zone(duress), Zone::Exile);
    assert_eq!(t.zone(bolt), Zone::Hand(P0));
    assert_eq!(tapped_lands(&t, P0), 7);
    t.resolve_all();
    assert_eq!(t.life(P1), 12);
    assert_eq!(t.life(P0), 28);
}

/// Galvanoth ("At the beginning of your upkeep, you may look at the top card of your
/// library. You may cast it without paying its mana cost if it's an instant or sorcery
/// spell.") on P0's battlefield, `name` on top of P0's library.
fn galvanoth(t: &mut TestGame, name: &str) -> ObjectId {
    t.battlefield(P0, "Galvanoth");
    t.library_top(P0, name)
}

fn run_galvanoth(t: &mut TestGame, _card: ObjectId, answers: &dyn Fn(&mut TestGame)) {
    answers(t);
    t.set_step(P0, Step::Untap);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    t.clear_answers();
}

#[test]
fn galvanoth_may_pay_the_kicker_cost_of_the_free_spell() {
    cr!("118.9a", "118.9b", "118.8a", "601.2b", "608.2g");
    ruling!(
        "Galvanoth",
        "You may pay additional costs, such as kicker costs, of the instant or sorcery card."
    );
    supported("Galvanoth");
    check_free_cast_costs(&FreeCaster {
        instants_only: false,
        place: galvanoth,
        run: run_galvanoth,
    });
}

/// P0's Think Twice ({1}{U}: "Draw a card." Flashback {2}{U}) in the graveyard has
/// gained flashback {1}{U} (its mana cost) from Snapcaster Mage; P0 has `islands`
/// Islands.
fn think_twice_with_two_flashbacks(islands: usize) -> (TestGame, ObjectId) {
    let mut t = TestGame::new(2);
    let twice = t.graveyard(P0, "Think Twice");
    t.answer_targets(P0, &[Entity::Object(twice)]);
    t.enter(P0, "Snapcaster Mage");
    t.resolve_all();
    t.clear_answers();
    t.lands(P0, "Island", islands);
    (t, twice)
}

#[test]
fn a_card_with_two_flashback_abilities_may_be_cast_for_either_cost() {
    cr!("702.34a", "118.9a", "601.2b");
    ruling!(
        "Snapcaster Mage",
        "If a card has multiple instances of flashback, you may choose any of its flashback costs to pay."
    );
    supported("Snapcaster Mage");
    supported("Think Twice");
    const FLASHBACK: CastMethod = CastMethod::Keyword(KeywordKind::Flashback);
    // With three Islands, P0 chooses which flashback cost to pay; the two are told apart
    // by their costs.
    for (cost, taps) in [("{2}{U}", 3), ("{1}{U}", 2)] {
        let (mut t, twice) = think_twice_with_two_flashbacks(3);
        let hand = t.hand_size(P0);
        let from = t.asked().len();
        choose_option_when_offered(&mut t, &format!("flashback {cost}"));
        t.cast(P0, twice).method(FLASHBACK).go();
        let offered: Vec<Vec<String>> = t.asked()[from..]
            .iter()
            .filter_map(|(_, d)| match d {
                Decision::ChooseCastingMethod { options, .. } => Some(options.clone()),
                _ => None,
            })
            .collect();
        assert_eq!(
            offered,
            vec![vec!["flashback {2}{U}".to_string(), "flashback {1}{U}".to_string()]]
        );
        assert_eq!(tapped_lands(&t, P0), taps);
        t.resolve_all();
        assert_eq!(t.hand_size(P0), hand + 1);
        assert!(t.in_exile("Think Twice"));
    }
    // With two Islands, only the {1}{U} flashback cost can be paid: by default, it's the
    // one used.
    let (mut t, twice) = think_twice_with_two_flashbacks(2);
    t.cast(P0, twice).method(FLASHBACK).go();
    assert_eq!(tapped_lands(&t, P0), 2);
    t.resolve_all();
    assert!(t.in_exile("Think Twice"));
}

/// Wraps P0's agent: a casting-method choice offering `label` is answered with it.
fn choose_option_when_offered(t: &mut TestGame, label: &str) {
    struct Pick {
        inner: Box<dyn mtg_engine::decision::Agent>,
        label: String,
    }
    impl mtg_engine::decision::Agent for Pick {
        fn decide(&mut self, g: &mtg_engine::game::Game, p: PlayerId, d: &Decision) -> Answer {
            // The scripted agent logs the decision.
            let a = self.inner.decide(g, p, d);
            if let Decision::ChooseCastingMethod { options, .. } = d {
                if let Some(i) = options.iter().position(|o| *o == self.label) {
                    return Answer::Index(i);
                }
            }
            a
        }
    }
    let mut agents = t.g.agents.0.lock().unwrap();
    let inner = std::mem::replace(
        &mut agents[P0.idx()],
        Box::new(mtg_engine::decision::PassiveAgent),
    );
    agents[P0.idx()] = Box::new(Pick {
        inner,
        label: label.to_string(),
    });
}

#[test]
fn jacob_frye_casts_a_copy_paying_its_costs_with_x_chosen() {
    cr!("707.12", "107.3a", "601.2b");
    ruling!(
        "Jacob Frye",
        "Because you’re paying the spell’s costs, if the spell has {X} in its mana cost, you may choose its value as normal."
    );
    supported("Jacob Frye");
    // "Whenever one or more Assassins you control deal combat damage to a player, exile
    // up to one target Assassin card or card with freerunning from your graveyard. If you
    // do, copy it. You may cast the copy." A custom Assassin with {X} in its mana cost:
    // the copy is cast with X = 2 (and becomes a token with two counters).
    let assassin = custom_card(
        "X Assassin",
        "Creature — Human Assassin",
        "{X}{B}",
        Some((0, 0)),
        "This creature enters with X +1/+1 counters on it.",
    );
    let mut t = TestGame::new(2);
    let jacob = t.battlefield(P0, "Jacob Frye");
    let card = t.custom(P0, assassin, Zone::Graveyard(P0));
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Wastes", 2);
    t.answer_targets(P0, &[Entity::Object(card)]);
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    attack_p1_unblocked(&mut t, jacob);
    assert_eq!(tapped_lands(&t, P0), 3);
    let copies: Vec<ObjectId> = t
        .named_on_battlefield("X Assassin")
        .into_iter()
        .filter(|id| t.obj(*id).is_token())
        .collect();
    assert_eq!(copies.len(), 1);
    assert_eq!(t.counters(copies[0], "+1/+1"), 2);
    assert_eq!(t.pt(copies[0]), (2, 2));
}
