//! Rulings batch P036 — other counterspell interactions: changing a spell's targets (CR
//! 115.7, 601.2d for divided damage), casting a spell during a resolution without paying
//! its mana cost (CR 608.2g, 601.2b, 118.9), choices made as a spell or delayed trigger
//! resolves (CR 608.2c), optional payments (CR 118.12), returning copies to hand (CR
//! 704.5e), votes that copy a spell (CR 701.38, 707.10), "if mana was spent" (CR 601.2h,
//! 106.6), and Equinox's "if it would destroy a land you control".

use crate::r_p036_common::*;
use crate::r_s01_common::{supported, watch};
use crate::r_s07_common::resolved;
use crate::r_s21_common::castable;
use crate::r_s25_common::{cast_new, lands_for_cost};
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::mana::ManaType;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn pool(t: &mut TestGame, p: PlayerId, ty: ManaType, n: usize) {
    crate::r_p037_common::pool(t, p, ty, n);
}

// --- Invert Polarity and Deflecting Swat ------------------------------------------------

/// P1 casts Arc Lightning ("deals 3 damage divided as you choose among one, two, or three
/// targets") on its turn: 2 at P0, 1 at P0's Grizzly Bears. Returns (the spell, Bears).
fn arc_lightning(t: &mut TestGame) -> (ObjectId, ObjectId) {
    turn_of(t, P1);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P1, &[Entity::Player(P0), obj(bears)]);
    t.answer(P1, DecisionKind::Divide, Answer::Numbers(vec![2, 1]));
    let s = cast_new(t, P1, "Arc Lightning", &[]);
    (s, bears)
}

#[test]
fn invert_polarity_changes_targets_but_not_the_division() {
    cr!("115.7d", "601.2d");
    ruling!(
        "Invert Polarity",
        "If the target spell has damage divided as it was cast, the division can't be changed although the targets receiving that damage still can. The same is true of spells that distribute counters."
    );
    supported("Invert Polarity");
    supported("Arc Lightning");
    let mut t = TestGame::new(2);
    let (arc, _bears) = arc_lightning(&mut t);
    let giant = t.battlefield(P1, "Hill Giant");
    crate::r_p017_common::load_coins(&mut t, &[true]);
    cast_new(&mut t, P0, "Invert Polarity", &[obj(arc)]);
    t.answer(P0, DecisionKind::Option, Answer::Index(0)); // call heads
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_targets(P0, &[obj(giant)]);
    // Even if P0 tried to divide differently, the division stays 2 and 1.
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![1, 2]));
    t.resolve();
    assert_eq!(t.obj(arc).controller, P0);
    t.resolve();
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.life(P1), 18);
    assert_eq!(crate::r_s07_common::damage_on(&t, giant), 1);
    // Counters: P1's Biogenic Upgrade ("Distribute three +1/+1 counters among one, two,
    // or three target creatures, then double the number of +1/+1 counters on each of those
    // creatures.") puts 2 on its Bears and 1 on its Hill Giant; P0 wins the flip and moves
    // them to its own Bears and Giant: 2 and 1, doubled.
    supported("Biogenic Upgrade");
    let mut t = TestGame::new(2);
    turn_of(&mut t, P1);
    let theirs = [t.battlefield(P1, "Grizzly Bears"), t.battlefield(P1, "Hill Giant")];
    let mine = [t.battlefield(P0, "Grizzly Bears"), t.battlefield(P0, "Hill Giant")];
    t.answer_targets(P1, &[obj(theirs[0]), obj(theirs[1])]);
    t.answer(P1, DecisionKind::Divide, Answer::Numbers(vec![2, 1]));
    let up = cast_new(&mut t, P1, "Biogenic Upgrade", &[]);
    crate::r_p017_common::load_coins(&mut t, &[true]);
    cast_new(&mut t, P0, "Invert Polarity", &[obj(up)]);
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(mine[0])]);
    t.answer_targets(P0, &[obj(mine[1])]);
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![1, 2]));
    t.resolve();
    t.resolve();
    assert_eq!(t.counters(mine[0], "+1/+1"), 4);
    assert_eq!(t.counters(mine[1], "+1/+1"), 2);
    assert_eq!(t.counters(theirs[0], "+1/+1"), 0);
}

#[test]
fn invert_polarity_asks_about_new_targets_only_after_winning() {
    cr!("608.2c", "701.6a");
    ruling!(
        "Invert Polarity",
        "You don't make any decisions about new targets for the spell until you win the flip."
    );
    for win in [true, false] {
        let mut t = TestGame::new(2);
        let bolt = cast_new(&mut t, P1, "Lightning Bolt", &[Entity::Player(P0)]);
        crate::r_p017_common::load_coins(&mut t, &[win]);
        cast_new(&mut t, P0, "Invert Polarity", &[obj(bolt)]);
        t.answer(P0, DecisionKind::Option, Answer::Index(0)); // call heads
        t.answer_yes(P0, true);
        t.answer_targets(P0, &[Entity::Player(P1)]);
        let from = t.asked().len();
        t.resolve();
        let asked: Vec<_> = t.asked()[from..]
            .iter()
            .filter(|(p, _)| *p == P0)
            .map(|(_, d)| d.clone())
            .collect();
        let flip = asked
            .iter()
            .position(|d| matches!(d, Decision::ChooseOption { prompt, .. } if prompt.contains("coin")))
            .expect("the coin was called");
        let new_targets = asked
            .iter()
            .position(|d| matches!(d, Decision::YesNo { .. } | Decision::ChooseTargets { .. }));
        if win {
            assert!(new_targets.expect("asked about new targets") > flip);
            t.resolve();
            assert_eq!(t.life(P1), 17);
        } else {
            assert_eq!(new_targets, None);
            assert!(!on_stack(&t, bolt));
            assert_eq!(t.life(P0), 20);
        }
    }
}

#[test]
fn new_targets_must_be_legal() {
    cr!("115.7d", "702.11b");
    ruling!(
        "Deflecting Swat",
        "If you choose new targets for the target spell, the new targets must be legal."
    );
    ruling!(
        "Invert Polarity",
        "If you choose new targets for the target spell, the new targets must be legal."
    );
    supported("Deflecting Swat");
    supported("Gladecover Scout");
    // Deflecting Swat: P1's Bolt (at P0) can't be changed to P0's hexproof Gladecover
    // Scout (P1 still controls the Bolt); Grizzly Bears is fine.
    let mut t = TestGame::new(2);
    let scout = t.battlefield(P0, "Gladecover Scout");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let bolt = cast_new(&mut t, P1, "Lightning Bolt", &[Entity::Player(P0)]);
    cast_new(&mut t, P0, "Deflecting Swat", &[obj(bolt)]);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(bears)]);
    let from = t.asked().len();
    t.resolve();
    let cands = offered(&t, P0, from);
    assert_eq!(cands.len(), 1);
    assert!(cands[0].contains(&obj(bears)));
    assert!(!cands[0].contains(&obj(scout)));
    t.resolve();
    assert!(!t.on_battlefield(bears));
    // Invert Polarity: after P0 gains control of the Bolt, P1's hexproof Scout is no
    // longer a legal target for it.
    let mut t = TestGame::new(2);
    let scout = t.battlefield(P1, "Gladecover Scout");
    let giant = t.battlefield(P1, "Hill Giant");
    let bolt = cast_new(&mut t, P1, "Lightning Bolt", &[Entity::Player(P0)]);
    crate::r_p017_common::load_coins(&mut t, &[true]);
    cast_new(&mut t, P0, "Invert Polarity", &[obj(bolt)]);
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(giant)]);
    let from = t.asked().len();
    t.resolve();
    let cands = offered(&t, P0, from);
    assert_eq!(cands.len(), 1);
    assert!(cands[0].contains(&obj(giant)));
    assert!(!cands[0].contains(&obj(scout)));
    t.resolve();
    assert!(!t.on_battlefield(giant));
}

// --- Counterlash ------------------------------------------------------------------------

/// P1 casts Lightning Bolt at P0 (or Grizzly Bears, with `creature`); P0 answers with
/// Counterlash ("Counter target spell. You may cast a spell that shares a card type with it
/// from your hand without paying its mana cost.") Returns the game with Counterlash on the
/// stack, the answers for its free cast queued by the caller before `go`.
fn counterlash(creature: bool) -> (TestGame, ObjectId) {
    supported("Counterlash");
    let mut t = TestGame::new(2);
    let spell = if creature {
        bears_spell(&mut t, P1)
    } else {
        cast_new(&mut t, P1, "Lightning Bolt", &[Entity::Player(P0)])
    };
    (t, spell)
}

/// The cards offered to P0 by choices asked since `from`.
fn chosen_from(t: &TestGame, from: usize) -> Vec<Vec<Entity>> {
    t.asked()[from..]
        .iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseEntities { candidates, .. } if *p == P0 => Some(candidates.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn counterlash_free_cast_pays_additional_costs_but_not_alternative_ones() {
    cr!("118.9a", "118.8d", "702.33a", "608.2g");
    ruling!(
        "Counterlash",
        "If you cast a card “without paying its mana cost,” you can’t pay any alternative costs. You can pay additional costs, such as kicker costs. If the card has mandatory additional costs, you must pay those."
    );
    supported("Fling");
    supported("Fireblast");
    // Kicker: Burst Lightning kicked for {4} deals 4.
    let (mut t, bolt) = counterlash(false);
    let burst = t.hand(P0, "Burst Lightning");
    t.lands(P0, "Wastes", 4);
    lands_for_cost(&mut t, P0, "Counterlash");
    let cl = t.hand(P0, "Counterlash");
    t.cast_with(P0, cl, &[obj(bolt)]).unwrap();
    t.answer_choose(P0, &[obj(burst)]);
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve();
    assert!(!on_stack(&t, bolt));
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    // A mandatory additional cost: Fling's "sacrifice a creature" is paid.
    let (mut t, bolt) = counterlash(false);
    let giant = t.battlefield(P0, "Hill Giant");
    let fling = t.hand(P0, "Fling");
    lands_for_cost(&mut t, P0, "Counterlash");
    cast_new(&mut t, P0, "Counterlash", &[obj(bolt)]);
    t.answer_choose(P0, &[obj(fling)]);
    t.answer_choose(P0, &[obj(giant)]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve();
    t.resolve_all();
    assert!(!t.on_battlefield(giant));
    assert_eq!(t.life(P1), 17);
    // An alternative cost isn't paid: Fireblast ("You may sacrifice two Mountains rather
    // than pay this spell's mana cost.") cast free keeps P0's Mountains.
    let (mut t, bolt) = counterlash(false);
    let mountains = t.lands(P0, "Mountain", 2);
    let fb = t.hand(P0, "Fireblast");
    lands_for_cost(&mut t, P0, "Counterlash");
    cast_new(&mut t, P0, "Counterlash", &[obj(bolt)]);
    t.answer_choose(P0, &[obj(fb)]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    let from = t.asked().len();
    t.resolve();
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    assert!(mountains.iter().all(|m| t.on_battlefield(*m)));
    assert!(!t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseCastingMethod { .. })));
}

#[test]
fn counterlash_ignores_type_timing_but_not_other_restrictions() {
    cr!("608.2g", "302.1", "601.3");
    ruling!(
        "Counterlash",
        "You cast the card in your hand as part of the resolution of Counterlash. Timing restrictions based on the card’s type (such as creature or sorcery) are ignored. Other restrictions are not (such as “Cast [this card] only during combat”)."
    );
    supported("Mandate of Peace");
    // A creature spell: P0 casts Hill Giant during P1's turn, mid-resolution.
    let (mut t, bears) = counterlash(true);
    let giant = t.hand(P0, "Hill Giant");
    lands_for_cost(&mut t, P0, "Counterlash");
    cast_new(&mut t, P0, "Counterlash", &[obj(bears)]);
    t.answer_choose(P0, &[obj(giant)]);
    let from = t.asked().len();
    t.resolve();
    assert!(chosen_from(&t, from).iter().any(|c| c.contains(&obj(giant))));
    t.resolve_all();
    let g = t.named_on_battlefield("Hill Giant");
    assert_eq!(g.len(), 1);
    assert_eq!(t.obj(g[0]).controller, P0);
    // Mandate of Peace ("Cast this spell only during combat.") can't be cast outside
    // combat, even this way.
    let (mut t, bolt) = counterlash(false);
    let mandate = t.hand(P0, "Mandate of Peace");
    lands_for_cost(&mut t, P0, "Counterlash");
    cast_new(&mut t, P0, "Counterlash", &[obj(bolt)]);
    t.answer_choose(P0, &[obj(mandate)]);
    let from = t.asked().len();
    t.resolve();
    assert!(!chosen_from(&t, from).iter().any(|c| c.contains(&obj(mandate))));
    assert!(t.in_hand(P0, "Mandate of Peace"));
    // During combat it can.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::DeclareAttackers);
    let bolt = cast_new(&mut t, P1, "Lightning Bolt", &[Entity::Player(P0)]);
    let mandate = t.hand(P0, "Mandate of Peace");
    lands_for_cost(&mut t, P0, "Counterlash");
    cast_new(&mut t, P0, "Counterlash", &[obj(bolt)]);
    t.answer_choose(P0, &[obj(mandate)]);
    let from = t.asked().len();
    t.resolve();
    assert!(chosen_from(&t, from).iter().any(|c| c.contains(&obj(mandate))));
    // Mandate of Peace was cast: it's on the stack, and resolving it ends combat, exiling
    // itself.
    let m = t.g.current(mandate);
    assert!(on_stack(&t, m));
    t.resolve();
    assert_eq!(t.zone(t.g.current(m)), mtg_engine::object::Zone::Exile);
}

#[test]
fn counterlash_free_cast_obeys_prohibitions_from_other_effects() {
    cr!("608.2g", "601.3");
    ruling!(
        "Counterlash",
        "Timing restrictions based on the card’s type (such as creature or sorcery) are ignored. Other restrictions are not"
    );
    supported("Rule of Law");
    // Rule of Law ("Each player can't cast more than one spell each turn."): Counterlash is
    // P0's spell this turn, so it can't cast another while Counterlash resolves.
    let (mut t, bolt) = counterlash(false);
    t.battlefield(P1, "Rule of Law");
    let shock = t.hand(P0, "Shock");
    lands_for_cost(&mut t, P0, "Counterlash");
    cast_new(&mut t, P0, "Counterlash", &[obj(bolt)]);
    t.answer_choose(P0, &[obj(shock)]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    let from = t.asked().len();
    t.resolve();
    assert!(!on_stack(&t, bolt));
    assert!(!chosen_from(&t, from).iter().any(|c| c.contains(&obj(shock))));
    assert!(t.in_hand(P0, "Shock"));
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    // Control: without Rule of Law the Shock is cast.
    let (mut t, bolt) = counterlash(false);
    let shock = t.hand(P0, "Shock");
    lands_for_cost(&mut t, P0, "Counterlash");
    cast_new(&mut t, P0, "Counterlash", &[obj(bolt)]);
    t.answer_choose(P0, &[obj(shock)]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve();
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

// --- Choices on resolution and optional payments -------------------------------------------

#[test]
fn arcane_denial_draw_count_is_chosen_when_the_delayed_trigger_resolves() {
    cr!("603.7a", "608.2c");
    ruling!(
        "Arcane Denial",
        "The controller of the countered spell doesn't choose how many cards to draw until the relevant ability resolves. The player may draw 0, 1, or 2 cards. They choose the number before drawing any cards."
    );
    supported("Arcane Denial");
    for n in [0i64, 1, 2] {
        let mut t = TestGame::new(2);
        let bolt = cast_new(&mut t, P1, "Lightning Bolt", &[Entity::Player(P0)]);
        cast_new(&mut t, P0, "Arcane Denial", &[obj(bolt)]);
        let from = t.asked().len();
        t.resolve();
        assert!(!on_stack(&t, bolt));
        // Nothing chosen yet.
        assert!(!t.asked()[from..]
            .iter()
            .any(|(_, d)| matches!(d, Decision::ChooseNumber { .. })));
        let seen = watch(
            &mut t,
            P1,
            |d| matches!(d, Decision::ChooseNumber { .. }),
            |g| (g.player(P1).hand.len(), g.turn.step),
        );
        t.answer(P1, DecisionKind::Number, Answer::Number(n));
        let hand = t.hand_size(P1);
        t.advance_to(P1, Step::Draw);
        // P1 chose in its upkeep, before drawing; then drew n (and its normal draw).
        let seen = seen.lock().unwrap().clone();
        assert_eq!(seen, vec![(hand, Step::Upkeep)], "n = {n}");
        assert_eq!(t.hand_size(P1), hand + n as usize + 1, "n = {n}");
    }
}

#[test]
fn force_spike_payment_is_optional() {
    cr!("118.12", "608.2c");
    ruling!("Force Spike", "The payment is optional.");
    ruling!("Mana Tithe", "The payment is optional.");
    for name in ["Force Spike", "Mana Tithe"] {
        supported(name);
        for pay in [true, false] {
            let mut t = TestGame::new(2);
            t.lands(P1, "Wastes", 1);
            let bolt = cast_new(&mut t, P1, "Lightning Bolt", &[Entity::Player(P0)]);
            t.answer_yes(P1, pay);
            cast_new(&mut t, P0, name, &[obj(bolt)]);
            t.resolve();
            assert_eq!(on_stack(&t, bolt), pay, "{name}: pay {pay}");
            assert_eq!(crate::r_s01_common::tapped_lands(&t, P1), 1 + pay as usize);
        }
    }
}

#[test]
fn sunken_field_offers_the_payment_on_resolution() {
    cr!("118.12", "608.2c");
    ruling!(
        "Sunken Field",
        "The spell’s controller gets the option to pay when the ability resolves."
    );
    supported("Sunken Field");
    // P1 has no spare mana when the ability is activated, and a land by the time it
    // resolves: it pays then.
    let mut t = TestGame::new(2);
    let island = t.battlefield(P0, "Island");
    crate::r_s06_common::attach_new(&mut t, P0, "Sunken Field", island);
    let bolt = cast_new(&mut t, P1, "Lightning Bolt", &[Entity::Player(P0)]);
    t.answer_targets(P0, &[obj(bolt)]);
    let from = t.asked().len();
    crate::r_s06_common::activate_containing(&mut t, P0, island, "Counter target spell")
        .unwrap();
    assert!(!t.asked()[from..]
        .iter()
        .any(|(p, d)| *p == P1 && matches!(d, Decision::YesNo { .. })));
    t.lands(P1, "Wastes", 1);
    t.answer_yes(P1, true);
    t.resolve();
    assert!(on_stack(&t, bolt));
    assert!(t.asked()[from..]
        .iter()
        .any(|(p, d)| *p == P1 && matches!(d, Decision::YesNo { .. })));
}

#[test]
fn aether_spike_paying_zero_energy() {
    cr!("107.14", "118.5", "118.12");
    ruling!(
        "Aether Spike",
        "You may pay zero {E}. You will get {E}{E}, and the target spell will resolve as long as its controller chooses to pay {0}. In most cases, this won't be a difficult choice for them."
    );
    for pay in [true, false] {
        let mut t = TestGame::new(2);
        let bolt = cast_new(&mut t, P1, "Lightning Bolt", &[Entity::Player(P0)]);
        cast_new(&mut t, P0, "Aether Spike", &[obj(bolt)]);
        t.answer(P0, DecisionKind::X, Answer::Number(0));
        t.answer_yes(P1, pay);
        t.resolve();
        assert_eq!(crate::r_s28_common::energy(&t, P0), 2);
        assert_eq!(on_stack(&t, bolt), pay, "pay {pay}");
    }
}

// --- Failure // Comply ------------------------------------------------------------------

#[test]
fn failure_returning_a_copy_makes_it_cease_to_exist() {
    cr!("704.5e", "707.10a");
    ruling!(
        "Failure // Comply",
        "If a copy of a spell is returned to its owner's hand, it's moved there, then it will cease to exist as a state-based action."
    );
    supported("Failure // Comply");
    let mut t = TestGame::new(2);
    let bolt = cast_new(&mut t, P1, "Lightning Bolt", &[Entity::Player(P0)]);
    crate::r_s25_common::keep_copy_targets(&mut t, P1);
    cast_new(&mut t, P1, "Twincast", &[obj(bolt)]);
    t.resolve();
    let copy = crate::r_s25_common::spell_copies(&t)[0];
    let hand = t.hand_size(P1);
    // Failure: "Return target spell to its owner's hand."
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 1);
    let card = t.hand(P0, "Failure // Comply");
    t.cast(P0, card).method(CastMethod::Half(0)).target(copy).go();
    t.resolve();
    assert!(!on_stack(&t, copy));
    assert_eq!(t.hand_size(P1), hand);
    assert!(!t.g.player(P1).hand.iter().any(|c| t.obj(*c).chars.name == "Lightning Bolt"));
    // The original is untouched.
    assert!(on_stack(&t, bolt));
}

/// P0 casts Comply (aftermath) from its graveyard on its turn, naming `name` as it
/// resolves. Returns whether the name decision came while Comply was resolving.
fn comply(t: &mut TestGame, name: &str) -> bool {
    let card = t.graveyard(P0, "Failure // Comply");
    t.lands(P0, "Plains", 1);
    let method = crate::r_s08_common::legal_cast_methods(t, P0, card)
        .into_iter()
        .find(|m| *m != CastMethod::Normal)
        .expect("aftermath");
    t.cast(P0, card).method(method).go();
    let asked_on_cast = t
        .asked()
        .iter()
        .any(|(_, d)| matches!(d, Decision::NameCard { .. }));
    t.answer(P0, DecisionKind::Name, Answer::Text(name.into()));
    t.resolve();
    let asked = t
        .asked()
        .iter()
        .any(|(_, d)| matches!(d, Decision::NameCard { .. }));
    !asked_on_cast && asked
}

#[test]
fn comply_names_a_card_on_resolution() {
    cr!("608.2c", "702.127a", "201.4");
    ruling!(
        "Failure // Comply",
        "You choose a card name as Comply resolves, not as you cast it. Opponents can't cast spells with that name after you've chosen it."
    );
    let mut t = TestGame::new(2);
    assert!(comply(&mut t, "Lightning Bolt"));
    assert!(t.in_exile("Failure // Comply"));
    let bolt = t.hand(P1, "Lightning Bolt");
    let shock = t.hand(P1, "Shock");
    t.lands(P1, "Mountain", 1);
    assert!(!castable(&mut t, P1, bolt));
    assert!(castable(&mut t, P1, shock));
}

#[test]
fn comply_can_name_a_land_which_can_still_be_played() {
    cr!("305.1", "201.4");
    ruling!(
        "Failure // Comply",
        "You may choose the name of a land card. Cards with that name can still be played since lands are never cast."
    );
    let mut t = TestGame::new(2);
    assert!(comply(&mut t, "Forest"));
    t.advance_to(P1, Step::PrecombatMain);
    let forest = t.hand(P1, "Forest");
    assert!(crate::r_s02_common::can_play_land(&mut t, P1, forest));
    t.play_land(P1, forest).unwrap();
    assert!(t.on_battlefield(t.g.current(forest)));
}

// --- Split Decision ----------------------------------------------------------------------

#[test]
fn split_decision_copy_isnt_cast_and_resolves_normally() {
    cr!("701.38a", "707.10", "707.10c");
    ruling!(
        "Split Decision",
        "If duplication gets more votes, Split Decision creates a copy of the spell. You control the copy. That copy is created on the stack, so it's not \"cast.\" Abilities that trigger when a player casts a spell won't trigger. The copy will resolve like a normal spell, after players get a chance to cast spells and activate abilities."
    );
    supported("Split Decision");
    // P1's Lightning Bolt at P0; P0 and P1 vote duplication. P0's Monastery Swiftspear
    // (prowess) triggers for Split Decision only.
    let mut t = TestGame::new(2);
    let spear = t.battlefield(P0, "Monastery Swiftspear");
    let bolt = cast_new(&mut t, P1, "Lightning Bolt", &[Entity::Player(P0)]);
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.answer(P1, DecisionKind::Option, Answer::Index(1));
    cast_new(&mut t, P0, "Split Decision", &[obj(bolt)]);
    crate::r_s25_common::change_copy_targets(&mut t, P0, &[Some(Entity::Player(P1))]);
    t.settle();
    t.resolve(); // prowess
    t.resolve(); // Split Decision
    let copies = crate::r_s25_common::spell_copies(&t);
    assert_eq!(copies.len(), 1);
    assert_eq!(t.obj(copies[0]).controller, P0);
    assert!(on_stack(&t, bolt));
    // No cast triggers for the copy.
    assert_eq!(t.stack_len(), 2);
    // Players get priority with the copy on the stack; then it resolves.
    assert!(castable_instant(&mut t, P1));
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.life(P0), 17);
    assert_eq!(t.pt(spear), (2, 3));
}

/// Whether `p` could cast a Shock now (given a Mountain).
fn castable_instant(t: &mut TestGame, p: PlayerId) -> bool {
    t.lands(p, "Mountain", 1);
    let s = t.hand(p, "Shock");
    castable(t, p, s)
}

// --- Repel Intruders: targets first, then mana ----------------------------------------

#[test]
fn repel_intruders_target_and_mana_are_independent() {
    cr!("601.2c", "601.2h");
    ruling!(
        "Repel Intruders",
        "You choose whether or not to target a creature spell before you decide whether or not to spend {U} on Repel Intruders. You may target a creature spell and then not spend {U} to cast Repel Intruders; if you do, Repel Intruders simply won’t affect that creature spell. Similarly, if you don’t target a creature spell but then you spend {U} to cast Repel Intruders, that part of Repel Intruders has no effect."
    );
    // Targeting a creature spell, paying {W}{3}: tokens, the spell isn't countered.
    let mut t = TestGame::new(2);
    let bears = bears_spell(&mut t, P1);
    pool(&mut t, P0, ManaType::W, 1);
    pool(&mut t, P0, ManaType::C, 3);
    let card = t.hand(P0, "Repel Intruders");
    let from = t.asked().len();
    let s = t.cast_with(P0, card, &[obj(bears)]).unwrap();
    assert!(offered(&t, P0, from)[0].contains(&obj(bears)));
    t.resolve();
    assert!(resolved(&t, s));
    assert!(on_stack(&t, bears));
    assert_eq!(crate::r_s01_common::with_subtype(&t, P0, "Kithkin").len(), 2);
    // No target, paying {U}{3}: nothing happens.
    let mut t = TestGame::new(2);
    let bears = bears_spell(&mut t, P1);
    pool(&mut t, P0, ManaType::U, 1);
    pool(&mut t, P0, ManaType::C, 3);
    let card = t.hand(P0, "Repel Intruders");
    t.answer(P0, DecisionKind::Targets, Answer::Entities(vec![]));
    let s = t.cast_with(P0, card, &[]).unwrap();
    t.resolve();
    assert!(resolved(&t, s));
    assert!(on_stack(&t, bears));
    assert!(crate::r_s01_common::tokens(&t, P0).is_empty());
    // Control: targeting it and paying {U}{3} counters it.
    let mut t = TestGame::new(2);
    let bears = bears_spell(&mut t, P1);
    pool(&mut t, P0, ManaType::U, 1);
    pool(&mut t, P0, ManaType::C, 3);
    let card = t.hand(P0, "Repel Intruders");
    t.cast_with(P0, card, &[obj(bears)]).unwrap();
    t.resolve();
    assert!(!on_stack(&t, bears));
    assert!(crate::r_s01_common::tokens(&t, P0).is_empty());
}

// --- Equinox ----------------------------------------------------------------------------

/// P0's Plains enchanted with Equinox ("Enchanted land has '{T}: Counter target spell if
/// it would destroy a land you control.'"), on P1's turn. Returns the Plains.
fn equinox() -> (TestGame, ObjectId) {
    supported("Equinox");
    let mut t = TestGame::new(2);
    let land = t.battlefield(P0, "Plains");
    crate::r_s06_common::attach_new(&mut t, P0, "Equinox", land);
    turn_of(&mut t, P1);
    (t, land)
}

/// P0 activates the Equinox land's ability at `spell`, and it resolves. Whether the spell
/// was countered.
fn equinox_at(t: &mut TestGame, land: ObjectId, spell: ObjectId) -> bool {
    t.answer_targets(P0, &[obj(spell)]);
    crate::r_s06_common::activate_containing(t, P0, land, "Counter target spell").unwrap();
    t.resolve();
    !on_stack(t, spell)
}

#[test]
fn equinox_ignores_damage_that_kills_a_land_creature() {
    cr!("704.5g", "701.8a");
    ruling!(
        "Equinox",
        "Equinox will not counter a spell that deals damage to an animated land, even if it would deal more damage than the land's toughness. This is because the spell itself does not destroy the land directly. The land is destroyed by a game rule."
    );
    ruling!(
        "Equinox",
        "Equinox will not counter a spell which would indirectly cause destruction of one of your lands."
    );
    supported("Mutavault");
    // Lightning Bolt at an animated Mutavault, and Pyroclasm ("2 damage to each creature")
    // with it animated.
    for name in ["Lightning Bolt", "Pyroclasm"] {
        let (mut t, land) = equinox();
        let vault = t.battlefield(P0, "Mutavault");
        // (Paid from the pool, so the Equinox land stays untapped.)
        pool(&mut t, P0, ManaType::C, 1);
        crate::r_s06_common::activate_containing(&mut t, P0, vault, "becomes").unwrap();
        t.resolve();
        assert!(t.obj_now(vault).is(mtg_engine::types::CardType::Creature));
        let spell = if name == "Pyroclasm" {
            cast_new(&mut t, P1, name, &[])
        } else {
            cast_new(&mut t, P1, name, &[obj(vault)])
        };
        assert!(!equinox_at(&mut t, land, spell), "{name}");
        t.resolve_all();
        assert!(!t.on_battlefield(vault), "{name}");
        assert!(t.in_graveyard(P0, "Mutavault"));
    }
    // Control: Stone Rain at the Mutavault is countered.
    let (mut t, land) = equinox();
    let vault = t.battlefield(P0, "Mutavault");
    let rain = cast_new(&mut t, P1, "Stone Rain", &[obj(vault)]);
    assert!(equinox_at(&mut t, land, rain));
    assert!(t.on_battlefield(vault));
}

#[test]
fn equinox_ignores_random_destruction() {
    cr!("701.8a");
    ruling!(
        "Equinox",
        "Equinox will not counter a spell that has a random chance of destroying a land."
    );
    supported("Wild Swing");
    // Wild Swing: "Choose three target nonenchantment permanents. Destroy one of them at
    // random."
    let (mut t, land) = equinox();
    let a = t.battlefield(P0, "Forest");
    let b = t.battlefield(P1, "Grizzly Bears");
    let c = t.battlefield(P1, "Hill Giant");
    let ws = cast_new(&mut t, P1, "Wild Swing", &[obj(a), obj(b), obj(c)]);
    assert!(!equinox_at(&mut t, land, ws));
    t.resolve();
    assert!(resolved(&t, ws));
}

#[test]
fn equinox_ignores_land_sacrifices() {
    cr!("701.21a", "118.8");
    ruling!(
        "Equinox",
        "Equinox will not counter a spell which requires sacrificing land when it enters or one that requires sacrificing land as part of the cost to cast it."
    );
    supported("Foul Spirit");
    supported("Crop Rotation");
    // P0's own Foul Spirit ("When this creature enters, sacrifice a land.") on P0's turn.
    let mut t = TestGame::new(2);
    let land = t.battlefield(P0, "Plains");
    crate::r_s06_common::attach_new(&mut t, P0, "Equinox", land);
    pool(&mut t, P0, ManaType::B, 1);
    pool(&mut t, P0, ManaType::C, 2);
    let fs = t.hand(P0, "Foul Spirit");
    let spirit = t.cast(P0, fs).go();
    assert!(!equinox_at(&mut t, land, spirit));
    t.resolve();
    assert!(t.on_battlefield(t.g.current(spirit)));
    // P0's Crop Rotation ("As an additional cost to cast this spell, sacrifice a land.").
    let mut t = TestGame::new(2);
    let land = t.battlefield(P0, "Plains");
    crate::r_s06_common::attach_new(&mut t, P0, "Equinox", land);
    let fodder = t.battlefield(P0, "Forest");
    pool(&mut t, P0, ManaType::G, 1);
    t.answer_choose(P0, &[obj(fodder)]);
    t.answer(P0, DecisionKind::Entities, Answer::Entities(vec![]));
    let cr = t.hand(P0, "Crop Rotation");
    let spell = t.cast(P0, cr).go();
    assert!(!t.on_battlefield(fodder));
    assert!(!equinox_at(&mut t, land, spell));
    t.resolve();
    assert!(resolved(&t, spell));
}
