//! Rulings batch P195 — "enters untapped/tapped" replacement effects (Spelunking,
//! Archelos, Lagoon Mystic) against "enters tapped" abilities and effects that put a
//! permanent onto the battlefield tapped, Amulet of Vigor's trigger, Curse of Inertia,
//! Wellspring and Sands of Time.

use crate::r_p195_common::*;
use crate::r_s01_common::stack_library;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// P0 casts Genesis Wave for `x`, with `cards` on top of their library, putting all of
/// them onto the battlefield at once.
fn genesis_wave(t: &mut TestGame, x: i64, cards: &[&str]) -> Vec<ObjectId> {
    let ids = stack_library(t, P0, cards);
    lots(t, P0);
    let c = t.hand(P0, "Genesis Wave");
    t.cast(P0, c).x(x).go();
    t.answer_choose_moved(P0, &ids.iter().map(|i| obj(*i)).collect::<Vec<_>>());
    t.resolve_all();
    ids.iter().map(|i| t.g.current(*i)).collect()
}

/// Whether `p` was asked to order replacement effects since decision `from`.
fn asked_replacement(t: &TestGame, p: PlayerId, from: usize) -> bool {
    t.asked()[from..]
        .iter()
        .any(|(q, d)| *q == p && matches!(d, Decision::ChooseReplacement { .. }))
}

/// P0 puts a Tranquil Cove onto the battlefield (a real zone change), choosing the
/// `pick`th replacement effect first. (tapped, asked to choose)
fn cove_enters(t: &mut TestGame, pick: usize) -> (bool, bool) {
    let from = t.asked().len();
    t.answer(P0, DecisionKind::Replacement, Answer::Index(pick));
    let c = t.enter(P0, "Tranquil Cove");
    t.clear_answers();
    (tapped(t, c), asked_replacement(t, P0, from))
}

/// P0 casts Rampant Growth, putting a Forest onto the battlefield "tapped" (no
/// replacement effect). Whether the Forest is tapped.
fn rampant_growth(t: &mut TestGame) -> bool {
    let f = t.library_top(P0, "Forest");
    cast_at(t, P0, "Rampant Growth", &[]);
    t.answer_choose(P0, &[obj(f)]);
    t.resolve_all();
    let f = t.g.current(f);
    assert!(t.on_battlefield(f));
    tapped(t, f)
}

#[test]
fn spelunking_against_enters_tapped() {
    cr!("614.12", "616.1", "614.1c");
    ruling!(
        "Spelunking",
        "If a land has an ability that says it enters the battlefield tapped, you choose the order in which that ability's effect and Spelunking's effect apply. This means you can choose to have the land enter tapped or untapped. If a land you control is simply put onto the battlefield tapped without a replacement effect being applied, it always enters untapped if you control Spelunking."
    );
    ruling!(
        "Spelunking",
        "If Spelunking would enter the battlefield at the same time a land you control would enter the battlefield tapped, that land still enters the battlefield tapped."
    );
    supported("Spelunking");
    supported("Genesis Wave");
    // A land with "enters tapped": its controller chooses the order, so either outcome.
    let mut outcomes = vec![];
    for pick in [0, 1] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Spelunking");
        let (tp, asked) = cove_enters(&mut t, pick);
        assert!(asked);
        outcomes.push(tp);
    }
    outcomes.sort();
    assert_eq!(outcomes, vec![false, true]);
    // Simply put onto the battlefield tapped: enters untapped.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Spelunking");
    assert!(!rampant_growth(&mut t));
    // Entering at the same time as Spelunking: the land enters tapped.
    let mut t = TestGame::new(2);
    t.library_top(P0, "Island"); // drawn by Spelunking
    let ids = genesis_wave(&mut t, 3, &["Spelunking", "Tranquil Cove"]);
    assert!(t.on_battlefield(ids[0]) && t.on_battlefield(ids[1]));
    assert!(tapped(&t, ids[1]));
}

#[test]
fn archelos_against_enters_tapped() {
    cr!("614.12", "616.1", "614.1c");
    ruling!(
        "Archelos, Lagoon Mystic",
        "If another replacement effect says that a permanent enters the battlefield tapped while Archelos is untapped, the entering permanent's controller chooses whether the permanent enters tapped or untapped. If a permanent is simply put onto the battlefield tapped without a replacement effect being applied, it always enters untapped if Archelos is untapped."
    );
    ruling!(
        "Archelos, Lagoon Mystic",
        "Archelos's abilities don't apply to itself as it enters the battlefield, nor do they apply to any permanents entering the battlefield at the same time."
    );
    ruling!(
        "Archelos, Lagoon Mystic",
        "If more than one Archelos is on the battlefield, the controller of an entering permanent chooses how to order the replacement effects."
    );
    supported("Archelos, Lagoon Mystic");
    let mut outcomes = vec![];
    for pick in [0, 1] {
        let mut t = TestGame::new(2);
        t.battlefield(P1, "Archelos, Lagoon Mystic");
        let (tp, asked) = cove_enters(&mut t, pick);
        assert!(asked);
        outcomes.push(tp);
    }
    outcomes.sort();
    assert_eq!(outcomes, vec![false, true]);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Archelos, Lagoon Mystic");
    assert!(!rampant_growth(&mut t));
    // Entering together with Archelos: the Cove enters tapped.
    let mut t = TestGame::new(2);
    let ids = genesis_wave(&mut t, 4, &["Archelos, Lagoon Mystic", "Tranquil Cove"]);
    assert!(t.on_battlefield(ids[0]) && t.on_battlefield(ids[1]));
    assert!(tapped(&t, ids[1]));
    // Two Archelos, one tapped and one untapped (different controllers): the entering
    // permanent's controller (P0) orders them.
    let mut outcomes = vec![];
    for pick in [0, 1] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Archelos, Lagoon Mystic");
        let a1 = t.battlefield(P1, "Archelos, Lagoon Mystic");
        tap(&mut t, a1);
        let from = t.asked().len();
        t.answer(P0, DecisionKind::Replacement, Answer::Index(pick));
        let f = t.enter(P0, "Forest");
        assert!(asked_replacement(&t, P0, from));
        assert!(!asked_replacement(&t, P1, from));
        outcomes.push(tapped(&t, f));
    }
    outcomes.sort();
    assert_eq!(outcomes, vec![false, true]);
}

#[test]
fn amulet_of_vigor_needs_a_permanent_that_enters_tapped() {
    cr!("603.2", "603.6a", "110.5b");
    ruling!(
        "Amulet of Vigor",
        "For Amulet of Vigor's ability to trigger, a permanent must enter tapped due to an effect that says \"put [the permanent] onto the battlefield tapped,\" \"[this permanent] enters tapped,\" or the like. If it enters untapped, the ability won't trigger, even if you tap that permanent afterward."
    );
    ruling!(
        "Amulet of Vigor",
        "If you control more than one Amulet of Vigor, each Amulet's ability triggers when a permanent enters tapped and under your control. The first ability that resolves will untap that permanent. If the permanent somehow becomes tapped again before the next ability resolves, the next ability will untap it as well (and so on)."
    );
    supported("Amulet of Vigor");
    // Enters tapped: triggers and untaps it.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Amulet of Vigor");
    let c = t.enter(P0, "Tranquil Cove");
    t.resolve_all();
    assert!(!tapped(&t, c));
    // Put onto the battlefield tapped (Rampant Growth): untapped.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Amulet of Vigor");
    assert!(!rampant_growth(&mut t));
    // Enters untapped, then tapped: no trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Amulet of Vigor");
    let f = t.enter(P0, "Forest");
    t.g.flush_events();
    t.settle();
    assert_eq!(t.stack_len(), 0);
    tap(&mut t, f);
    assert_eq!(t.stack_len(), 0);
    assert!(tapped(&t, f));
    // Two Amulets: two triggers; tapped again between them, untapped again.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Amulet of Vigor");
    t.battlefield(P0, "Amulet of Vigor");
    let c = t.enter(P0, "Tranquil Cove");
    t.settle();
    // Two Amulet triggers and the Cove's life-gain trigger.
    assert_eq!(t.stack_len(), 3);
    // Resolve until the first Amulet trigger untaps the Cove; one Amulet trigger is
    // still waiting.
    while tapped(&t, c) {
        t.resolve();
    }
    assert!(t.stack_len() >= 1);
    tap(&mut t, c);
    t.resolve_all();
    assert!(!tapped(&t, c));
}

#[test]
fn curse_of_inertia_triggers_once_and_its_controller_controls_it() {
    cr!("508.1m", "603.3a", "702.11b", "115.2");
    ruling!(
        "Curse of Inertia",
        "Curse of Inertia’s ability will trigger only once per attacking player per combat phase, no matter how many creatures that player attacks with."
    );
    ruling!(
        "Curse of Inertia",
        "Even though the attacking player chooses the target of the ability, the ability is still controlled by the player who controls Curse of Inertia. Notably, a creature with hexproof controlled by the player who controls Curse of Inertia could be chosen, even though another player may be choosing the target."
    );
    supported("Curse of Inertia");
    supported("Gladecover Scout");
    // P0's Curse on P1; P2 attacks P1 with two creatures.
    let mut t = TestGame::new(3);
    let curse = t.hand(P0, "Curse of Inertia");
    lots(&mut t, P0);
    t.cast(P0, curse).target(pl(P1)).go();
    t.resolve_all();
    let scout = t.battlefield(P0, "Gladecover Scout");
    let a = t.battlefield(P2, "Grizzly Bears");
    let b = t.battlefield(P2, "Hill Giant");
    t.set_step(P2, Step::BeginningOfCombat);
    t.answer(
        P2,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(a, pl(P1)), (b, pl(P1))]),
    );
    let from = t.asked().len();
    t.answer_targets(P2, &[obj(scout)]);
    t.answer(P2, DecisionKind::Option, Answer::Index(0));
    t.answer_yes(P2, true);
    t.advance_to(P2, Step::DeclareAttackers);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    let trig = *t.g.stack.last().unwrap();
    assert_eq!(t.g.obj(trig).controller, P0);
    assert!(target_choices(&t, P2, from, "")
        .iter()
        .any(|c| c.contains(&obj(scout))));
    assert_eq!(stack_targets(&t, trig), vec![obj(scout)]);
    t.resolve_all();
    assert!(tapped(&t, scout));
}

#[test]
fn wellspring_control_outlasts_the_aura() {
    cr!("611.2a", "613.1b");
    ruling!(
        "Wellspring",
        "If Wellspring leaves the battlefield, the control effect still lasts until end of turn."
    );
    supported("Wellspring");
    let mut t = TestGame::new(2);
    let land = t.battlefield(P1, "Forest");
    let w = cast_at(&mut t, P0, "Wellspring", &[obj(land)]);
    t.resolve_all();
    assert_eq!(ctl(&t, land), P0);
    kill(&mut t, w);
    assert!(t.in_graveyard(P0, "Wellspring"));
    assert_eq!(ctl(&t, land), P0);
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(ctl(&t, land), P1);
}

#[test]
fn sands_of_time_skipped_untap_and_upkeep_toggle() {
    cr!("502.1", "614.10", "106.4", "500.4");
    ruling!(
        "Sands of Time",
        "Skipping the untap step means that phased out cards won't phase in, and permanents with phasing won't phase out."
    );
    ruling!(
        "Sands of Time",
        "This card still affects players when it is tapped."
    );
    ruling!(
        "Sands of Time",
        "You can tap lands for mana before Sands of Time toggles your permanents. The mana you gain is available during your upkeep step, but it will disappear before your draw step if you don't spend it. You cannot save it to be spent on the card you will draw this turn."
    );
    supported("Sands of Time");
    supported("Breezekeeper");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Sands of Time");
    tap(&mut t, s);
    let bk = t.battlefield(P1, "Breezekeeper");
    let f = t.battlefield(P1, "Forest");
    // P1's untap step is skipped: Breezekeeper doesn't phase out.
    t.advance_to(P1, Step::Upkeep);
    assert!(!t.obj_now(bk).phased_out);
    // The tapped Sands still toggles: tap the Forest for mana in response to the trigger.
    t.settle();
    assert!(top_is_trigger_of(&t, s));
    let forest_ability = 0;
    t.activate(P1, f, forest_ability, &[]).unwrap();
    assert!(tapped(&t, f));
    assert_eq!(t.g.player(P1).mana_pool.total(), 1);
    t.resolve_all();
    // Toggled: the tapped Forest untapped, untapped Breezekeeper tapped.
    assert!(!tapped(&t, f));
    assert!(tapped(&t, bk));
    assert_eq!(t.g.player(P1).mana_pool.total(), 1);
    t.advance_to(P1, Step::Draw);
    assert_eq!(t.g.player(P1).mana_pool.total(), 0);
}
