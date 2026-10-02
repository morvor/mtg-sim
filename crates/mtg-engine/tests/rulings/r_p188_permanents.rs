//! Rulings batch P188 — type-changing permanents and their abilities: Nim Deathmantle,
//! Roaming Throne, Kargan Intimidator, the "choose a creature type" permanents (Adaptive
//! Automaton, Metallic Mimic, Titan of Littjara), Rimefeather Owl, Sensei Golden-Tail,
//! Beorn the Fierce, Thran Portal, Vault 87: Forced Evolution, Applied Geometry and
//! Juniper Order Ranger.

use crate::r_s01_common::supported;
use crate::r_s03_common::to_blockers;
use crate::r_s06_common::{attach_new, attached_to, has_kw};
use crate::r_s20_common::tap_for_mana;
use crate::r_s24_common::choose_creature_type;
use crate::r_s25_common::targets_of;
use crate::r_s28_common::cast_card;
use crate::r_s35_common::creature_type_options;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn o(id: ObjectId) -> Entity {
    Entity::Object(id)
}

fn sub(t: &TestGame, id: ObjectId, s: &str) -> bool {
    t.obj_now(id).chars.has_subtype(s)
}

/// `p` casts `name` targeting `target` (lands for it are added) and it resolves.
fn cast_at(t: &mut TestGame, p: PlayerId, name: &str, target: ObjectId) {
    t.answer_targets(p, &[o(target)]);
    cast_card(t, p, name);
    t.resolve_all();
}

/// `p` destroys the permanent as an effect would (it dies if it's a creature).
fn destroy(t: &mut TestGame, id: ObjectId) {
    let id = t.g.current(id);
    t.g.destroy(id, None);
    t.g.flush_events();
    t.settle();
}

// ---------------------------------------------------------------------------------------
// Nim Deathmantle
// ---------------------------------------------------------------------------------------

/// "Whenever a nontoken creature is put into your graveyard from the battlefield, you may
/// pay {4}. If you do, return that card to the battlefield and attach this Equipment to
/// it."
fn yes_no_asked(t: &TestGame, from: usize) -> usize {
    t.asked()[from..]
        .iter()
        .filter(|(p, d)| *p == P0 && matches!(d, Decision::YesNo { .. }))
        .count()
}

#[test]
fn nim_deathmantle_stops_affecting_a_creature_it_becomes_unattached_from() {
    cr!("301.5a", "611.3a", "613.1d", "613.1e");
    ruling!(
        "Nim Deathmantle",
        "Once Nim Deathmantle becomes unattached from a creature, its color-changing and type-changing effects stop affecting that creature."
    );
    supported("Nim Deathmantle");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let elves = t.battlefield(P0, "Llanowar Elves");
    let nim = t.battlefield(P0, "Nim Deathmantle");
    t.lands(P0, "Wastes", 4);
    t.activate(P0, nim, 0, &[o(bears)]).expect("equip");
    t.resolve_all();
    let b = t.obj_now(bears);
    assert_eq!(b.chars.colors, ColorSet::single(Color::Black));
    assert!(b.chars.has_subtype("Zombie") && !b.chars.has_subtype("Bear"));
    assert!(b.has_keyword(KeywordKind::Intimidate));
    assert_eq!(t.pt(bears), (4, 4));
    // Moved to the Elves: the Bears is a green Bear again.
    t.lands(P0, "Wastes", 4);
    t.activate(P0, nim, 0, &[o(elves)]).expect("equip");
    t.resolve_all();
    let b = t.obj_now(bears);
    assert_eq!(b.chars.colors, ColorSet::single(Color::Green));
    assert!(!b.chars.has_subtype("Zombie") && b.chars.has_subtype("Bear"));
    assert_eq!(t.pt(bears), (2, 2));
    assert!(sub(&t, elves, "Zombie"));
}

#[test]
fn a_card_nim_deathmantle_returned_stays_and_reverts_when_unattached() {
    cr!("301.5a", "603.5", "608.2c", "611.3a", "613.1d");
    ruling!(
        "Nim Deathmantle",
        "Once Nim Deathmantle returns a card from your graveyard to the battlefield, it will remain on the battlefield indefinitely, even if Nim Deathmantle becomes unattached from it."
    );
    ruling!(
        "Nim Deathmantle",
        "This is true even if Nim Deathmantle returned that creature to the battlefield from the graveyard."
    );
    ruling!(
        "Nim Deathmantle",
        "You choose whether to pay {4} as Nim Deathmantle's second ability resolves."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let elves = t.battlefield(P0, "Llanowar Elves");
    let nim = t.battlefield(P0, "Nim Deathmantle");
    t.lands(P0, "Wastes", 4);
    let from = t.asked().len();
    destroy(&mut t, bears);
    // The ability is on the stack; nothing has been asked yet.
    assert_eq!(t.stack_len(), 1);
    assert_eq!(yes_no_asked(&t, from), 0);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(yes_no_asked(&t, from), 1);
    let back = t.g.current(bears);
    assert!(t.on_battlefield(back));
    assert_eq!(attached_to(&t, nim), Some(o(back)));
    assert!(sub(&t, back, "Zombie"));
    // Unattached (moved to the Elves): it stays, and is a green Bear again.
    t.lands(P0, "Wastes", 4);
    t.activate(P0, nim, 0, &[o(elves)]).expect("equip");
    t.resolve_all();
    assert!(t.on_battlefield(back));
    assert!(sub(&t, back, "Bear") && !sub(&t, back, "Zombie"));
    assert_eq!(t.obj_now(back).chars.colors, ColorSet::single(Color::Green));
    assert_eq!(t.pt(back), (2, 2));
}

#[test]
fn nim_deathmantle_declined_returns_nothing() {
    cr!("603.5", "608.2c");
    ruling!(
        "Nim Deathmantle",
        "once it begins to resolve and you decide whether to pay, it's too late for players to respond."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let nim = t.battlefield(P0, "Nim Deathmantle");
    let lands = t.lands(P0, "Wastes", 4);
    destroy(&mut t, bears);
    t.answer_yes(P0, false);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(attached_to(&t, nim), None);
    assert!(lands.iter().all(|l| !t.obj(*l).tapped));
}

#[test]
fn nim_deathmantle_can_be_paid_for_a_card_that_left_the_graveyard() {
    cr!("400.7", "608.2b", "608.2c");
    ruling!(
        "Nim Deathmantle",
        "If the nontoken creature that caused Nim Deathmantle's second ability to trigger is somehow removed from your graveyard before that ability resolves, you may still pay {4} as it resolves. Even if you do, however, no card will be returned to the battlefield."
    );
    supported("Tormod's Crypt");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let nim = t.battlefield(P0, "Nim Deathmantle");
    let crypt = t.battlefield(P0, "Tormod's Crypt");
    let lands = t.lands(P0, "Wastes", 4);
    destroy(&mut t, bears);
    assert_eq!(t.stack_len(), 1);
    // In response, Tormod's Crypt exiles P0's graveyard.
    t.activate(P0, crypt, 0, &[Entity::Player(P0)])
        .expect("crypt");
    t.resolve();
    assert!(t.in_exile("Grizzly Bears"));
    t.answer_yes(P0, true);
    t.resolve_all();
    // The {4} was paid, but the Bears stays in exile.
    assert!(lands.iter().all(|l| t.obj(*l).tapped));
    assert!(t.in_exile("Grizzly Bears"));
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
    assert_eq!(attached_to(&t, nim), None);
}

// ---------------------------------------------------------------------------------------
// Roaming Throne
// ---------------------------------------------------------------------------------------

fn throne(t: &mut TestGame, ty: &str) -> ObjectId {
    choose_creature_type(t, P0, ty);
    let id = t.enter(P0, "Roaming Throne");
    t.g.flush_events();
    t.settle();
    id
}

#[test]
fn each_roaming_throne_adds_one_more_trigger() {
    cr!("603.2", "603.2d");
    ruling!(
        "Roaming Throne",
        "If you control two Roaming Thrones with the same chosen creature type, triggered abilities of other creatures you control of the chosen type trigger three times. Three such Roaming Thrones result in four triggered abilities, and so on."
    );
    supported("Roaming Throne");
    // Elvish Visionary: "When this creature enters, draw a card."
    for thrones in 0..=3usize {
        let mut t = TestGame::new(2);
        for _ in 0..thrones {
            throne(&mut t, "Elf");
        }
        t.enter(P0, "Elvish Visionary");
        t.g.flush_events();
        t.settle();
        assert_eq!(t.stack_len(), thrones + 1, "{thrones} thrones");
        let hand = t.hand_size(P0);
        t.resolve_all();
        assert_eq!(t.hand_size(P0), hand + thrones + 1);
    }
}

#[test]
fn roaming_throne_extra_triggers_make_their_own_choices() {
    cr!("603.2d", "603.3d", "608.2c");
    ruling!(
        "Roaming Throne",
        "Any choices made as you put the ability onto the stack, such as modes and targets, are made separately for each instance of the ability. Any choices made on resolution, such as whether to put counters on a permanent, are also made individually."
    );
    supported("Gravedigger");
    // Gravedigger: "When this creature enters, you may return target creature card from
    // your graveyard to your hand."
    let mut t = TestGame::new(2);
    throne(&mut t, "Zombie");
    let bears = t.graveyard(P0, "Grizzly Bears");
    let elves = t.graveyard(P0, "Llanowar Elves");
    t.answer_targets(P0, &[o(bears)]);
    t.answer_targets(P0, &[o(elves)]);
    t.enter(P0, "Gravedigger");
    t.g.flush_events();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    let mut targets: Vec<Entity> = t
        .g
        .stack
        .iter()
        .flat_map(|s| targets_of(&t, *s))
        .collect();
    targets.sort();
    let mut want = vec![o(bears), o(elves)];
    want.sort();
    assert_eq!(targets, want);
    // The top one (the Elves) is declined, the other one (the Bears) accepted.
    t.answer_yes(P0, false);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert!(t.in_graveyard(P0, "Llanowar Elves"));
}

// ---------------------------------------------------------------------------------------
// Kargan Intimidator
// ---------------------------------------------------------------------------------------

/// The modes `p` could choose in the last "choose modes" decision since `from`.
fn last_available_modes(t: &TestGame, from: usize) -> Vec<usize> {
    t.asked()[from..]
        .iter()
        .rev()
        .find_map(|(_, d)| match d {
            Decision::ChooseModes { available, .. } => Some(available.clone()),
            _ => None,
        })
        .expect("no modes asked")
}

#[test]
fn two_kargan_intimidators_track_their_modes_separately() {
    cr!("700.2", "700.2a");
    ruling!(
        "Kargan Intimidator",
        "If you control two or more Kargan Intimidators, track which modes have been chosen each turn for each one's ability separately."
    );
    supported("Kargan Intimidator");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Kargan Intimidator");
    let b = t.battlefield(P0, "Kargan Intimidator");
    t.lands(P0, "Wastes", 3);
    // A: "This creature gets +1/+1 until end of turn."
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0]));
    t.activate(P0, a, 0, &[]).expect("a");
    t.resolve_all();
    assert_eq!(t.pt(a), (4, 2));
    // B can still choose that mode.
    let from = t.asked().len();
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0]));
    t.activate(P0, b, 0, &[]).expect("b");
    assert!(last_available_modes(&t, from).contains(&0));
    t.resolve_all();
    assert_eq!(t.pt(b), (4, 2));
    // A can't choose it again this turn.
    let from = t.asked().len();
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![2]));
    t.answer_targets(P0, &[o(a)]);
    t.activate(P0, a, 0, &[]).expect("a again");
    assert!(!last_available_modes(&t, from).contains(&0));
    t.resolve_all();
    assert_eq!(t.pt(a), (4, 2));
    assert!(has_kw(&t, a, KeywordKind::Trample));
}

#[test]
fn a_blocker_that_becomes_a_coward_keeps_blocking_the_warrior() {
    cr!("506.4", "509.1h");
    ruling!(
        "Kargan Intimidator",
        "Once a creature has blocked a Warrior, turning that creature into a Coward won't remove it from combat or cause the Warrior to become unblocked."
    );
    let mut t = TestGame::new(2);
    let k = t.battlefield(P0, "Kargan Intimidator");
    let wall = t.battlefield(P1, "Wall of Stone");
    to_blockers(&mut t, &[(k, Entity::Player(P1))], &[(wall, k)]);
    assert!(t.g.combat.as_ref().unwrap().is_blocked(k));
    t.lands(P0, "Wastes", 1);
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1]));
    t.activate(P0, k, 0, &[o(wall)]).expect("coward");
    t.resolve_all();
    assert!(sub(&t, wall, "Coward"));
    let c = t.g.combat.as_ref().unwrap();
    assert!(c.is_blocked(k));
    assert_eq!(c.blockers_of(k), vec![wall]);
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.obj_now(wall).damage, 3);
}

// ---------------------------------------------------------------------------------------
// The creature type is chosen as the permanent enters
// ---------------------------------------------------------------------------------------

#[test]
fn adaptive_automaton_applies_as_soon_as_it_enters() {
    cr!("614.12", "614.12a");
    ruling!(
        "Adaptive Automaton",
        "The choice of creature type is made as Adaptive Automaton enters. Players can't take any actions between the time the choice is made and the time the appropriate creatures begin to get +1/+1."
    );
    supported("Adaptive Automaton");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let from = t.asked().len();
    choose_creature_type(&mut t, P0, "Bear");
    let aa = t.enter(P0, "Adaptive Automaton");
    // Nothing used the stack and nobody got priority in between.
    assert_eq!(t.stack_len(), 0);
    assert!(!t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::Priority { .. })));
    assert_eq!(creature_type_options(&t, P0, from).len(), 1);
    assert!(sub(&t, aa, "Bear") && sub(&t, aa, "Construct"));
    assert_eq!(t.pt(bears), (3, 3));
}

#[test]
fn metallic_mimic_is_the_chosen_type_as_it_enters() {
    cr!("614.12", "614.12a", "614.1c");
    ruling!(
        "Metallic Mimic",
        "The choice of creature type is made as Metallic Mimic enters the battlefield. Players can't respond to this choice. Metallic Mimic's second ability starts applying immediately."
    );
    supported("Metallic Mimic");
    let mut t = TestGame::new(2);
    let from = t.asked().len();
    choose_creature_type(&mut t, P0, "Bear");
    let mm = t.enter(P0, "Metallic Mimic");
    assert_eq!(t.stack_len(), 0);
    assert!(!t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::Priority { .. })));
    assert!(sub(&t, mm, "Bear") && sub(&t, mm, "Shapeshifter"));
    let bears = t.enter(P0, "Grizzly Bears");
    assert_eq!(t.counters(bears, "+1/+1"), 1);
}

#[test]
fn metallic_mimic_offers_only_creature_types() {
    cr!("205.3m", "205.2a");
    ruling!(
        "Metallic Mimic",
        "You must choose an existing creature type. \"Artifact\" and \"Vehicle\" aren't creature types."
    );
    let mut t = TestGame::new(2);
    let from = t.asked().len();
    choose_creature_type(&mut t, P0, "Golem");
    t.enter(P0, "Metallic Mimic");
    let opts = creature_type_options(&t, P0, from);
    assert_eq!(opts.len(), 1);
    for bad in ["Artifact", "Vehicle", "Creature", "Equipment", "Legendary"] {
        assert!(!opts[0].iter().any(|x| x == bad), "{bad} offered");
    }
    assert!(opts[0].iter().any(|x| x == "Golem"));
}

#[test]
fn titan_of_littjara_shares_the_chosen_type_for_its_enters_trigger() {
    cr!("614.12", "614.12a", "603.2");
    ruling!(
        "Titan of Littjara",
        "The choice of creature type is made as Titan of Littjara enters the battlefield. Players can't respond to this choice. Titan of Littjara's second ability starts applying immediately."
    );
    supported("Titan of Littjara");
    // "Whenever this creature enters or attacks, you may draw a card for each other
    // creature you control that shares a creature type with it. If you do, discard a
    // card."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Llanowar Elves");
    choose_creature_type(&mut t, P0, "Bear");
    let titan = t.enter(P0, "Titan of Littjara");
    assert!(sub(&t, titan, "Bear") && sub(&t, titan, "Illusion"));
    t.g.flush_events();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.answer_yes(P0, true);
    let lib = t.library_size(P0);
    t.resolve_all();
    // Two cards drawn (the Bears share the chosen type; the Elves don't), one discarded.
    assert_eq!(t.library_size(P0), lib - 2);
    assert_eq!(t.hand_size(P0), 1);
}

// ---------------------------------------------------------------------------------------
// Rimefeather Owl, Sensei Golden-Tail, Beorn the Fierce, Thran Portal
// ---------------------------------------------------------------------------------------

#[test]
fn rimefeather_owl_makes_every_permanent_with_an_ice_counter_snow() {
    cr!("205.4a", "611.3a", "613.1d");
    ruling!(
        "Rimefeather Owl",
        "Rimefeather Owl’s last ability affects all permanents with ice counters on them, whether or not it put the ice counters on them."
    );
    supported("Rimefeather Owl");
    let mut t = TestGame::new(2);
    let owl = t.battlefield(P0, "Rimefeather Owl");
    let forest = t.battlefield(P1, "Forest");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let snow = |t: &TestGame, id: ObjectId| t.obj_now(id).chars.supertypes.contains(Supertype::Snow);
    assert!(!snow(&t, forest) && !snow(&t, bears));
    // An ice counter some other effect put on the Forest.
    t.g.add_counters(o(forest), "ice", 1, None);
    t.g.recompute();
    assert!(snow(&t, forest));
    // One put by the Owl's own ability (paid with the snow Forest's mana... P0's own).
    t.lands(P0, "Snow-Covered Island", 2);
    t.activate(P0, owl, 0, &[o(bears)]).expect("owl");
    t.resolve_all();
    assert!(snow(&t, bears));
    // The Owl, the Forest, the Bears and the two snow lands.
    assert_eq!(t.pt(owl), (5, 5));
}

#[test]
fn removing_the_training_counter_doesnt_undo_sensei_golden_tail() {
    cr!("611.2a", "613.1d", "613.1f");
    ruling!(
        "Sensei Golden-Tail",
        "The training counter just marks which creatures have been changed; an effect that removes the counter doesn’t change the creature’s types or abilities."
    );
    supported("Sensei Golden-Tail");
    let mut t = TestGame::new(2);
    let sensei = t.battlefield(P0, "Sensei Golden-Tail");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 2);
    t.activate(P0, sensei, 0, &[o(bears)]).expect("train");
    t.resolve_all();
    assert_eq!(t.counters(bears, "training"), 1);
    assert!(sub(&t, bears, "Samurai") && sub(&t, bears, "Bear"));
    assert!(has_kw(&t, bears, KeywordKind::Bushido));
    t.g.remove_counters(o(bears), "training", 1);
    t.g.recompute();
    assert_eq!(t.counters(bears, "training"), 0);
    assert!(sub(&t, bears, "Samurai"));
    assert!(has_kw(&t, bears, KeywordKind::Bushido));
}

#[test]
fn beorns_bear_lasts_after_the_turn_and_after_beorn_leaves() {
    cr!("611.2a", "613.1d");
    ruling!(
        "Beorn the Fierce",
        "The type-changing effect of the triggered ability lasts indefinitely. It doesn't wear off during the cleanup step or when Beorn the Fierce leaves the battlefield."
    );
    supported("Beorn the Fierce");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::Upkeep);
    let beorn = t.battlefield(P0, "Beorn the Fierce");
    let giant = t.battlefield(P0, "Hill Giant");
    t.answer_targets(P0, &[o(giant)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.settle();
    t.resolve_all();
    assert!(sub(&t, giant, "Bear") && sub(&t, giant, "Giant"));
    assert_eq!(t.counters(giant, "trample"), 1);
    // Other Bears get +2/+2.
    assert_eq!(t.pt(giant), (5, 5));
    t.advance_to(P1, Step::Upkeep);
    assert!(sub(&t, giant, "Bear"));
    destroy(&mut t, beorn);
    assert!(sub(&t, giant, "Bear"));
    assert_eq!(t.pt(giant), (3, 3));
}

#[test]
fn thran_portal_taps_for_its_chosen_type_for_one_life() {
    cr!("305.6", "205.1b");
    ruling!(
        "Thran Portal",
        "Thran Portal will have the appropriate intrinsic mana ability for the basic land type chosen as it enters the battlefield. It is still a Gate and still has its other abilities"
    );
    supported("Thran Portal");
    let mut t = TestGame::new(2);
    // Index 1: Island.
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    let portal = t.enter(P0, "Thran Portal");
    assert!(sub(&t, portal, "Gate") && sub(&t, portal, "Island"));
    assert!(t.obj_now(portal).is(CardType::Land));
    let portal = t.g.current(portal);
    t.g.objects[portal.0 as usize].tapped = false;
    assert!(tap_for_mana(&mut t, P0, portal, "{U}"));
    assert_eq!(t.g.player(P0).mana_pool.count(ManaType::U), 1);
    assert_eq!(t.life(P0), 19);
}

// ---------------------------------------------------------------------------------------
// Vault 87: Forced Evolution
// ---------------------------------------------------------------------------------------

#[test]
fn vault_87_keeps_control_of_a_creature_that_becomes_a_mutant_and_draws_on_resolution() {
    cr!("714.2b", "608.2b", "611.2b", "608.2h", "714.4");
    ruling!(
        "Vault 87: Forced Evolution",
        "The target of Vault 87's first chapter ability only needs to be a non-Mutant when that ability is put onto the stack and when that ability resolves. The control effect won't end if it later becomes a Mutant"
    );
    ruling!(
        "Vault 87: Forced Evolution",
        "Use the greatest power among Mutants you control as Vault 87's last ability resolves to determine how many cards to draw."
    );
    supported("Vault 87: Forced Evolution");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[o(giant)]);
    let saga = t.enter(P0, "Vault 87: Forced Evolution");
    t.g.flush_events();
    t.settle();
    t.resolve_all();
    assert_eq!(t.obj_now(giant).controller, P0);
    // Chapter II: the stolen Giant gets a counter and becomes a Mutant.
    t.advance_to(P1, Step::Upkeep);
    t.answer_targets(P0, &[o(giant)]);
    t.advance_to(P0, Step::PrecombatMain);
    t.settle();
    t.resolve_all();
    assert!(sub(&t, giant, "Mutant"));
    assert_eq!(t.obj_now(giant).controller, P0);
    assert_eq!(t.pt(giant), (4, 4));
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.obj_now(giant).controller, P0);
    // Chapter III: in response, the Giant gets +3/+3; seven cards are drawn.
    t.advance_to(P0, Step::PrecombatMain);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.answer_targets(P0, &[o(giant)]);
    cast_card(&mut t, P0, "Giant Growth");
    t.resolve();
    assert_eq!(t.pt(giant), (7, 7));
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 7);
    // The Saga is sacrificed after chapter III, ending the control effect.
    assert!(!t.on_battlefield(saga));
    assert_eq!(t.obj_now(giant).controller, P1);
}

// ---------------------------------------------------------------------------------------
// Applied Geometry, Juniper Order Ranger
// ---------------------------------------------------------------------------------------

#[test]
fn applied_geometry_copies_only_the_printed_values() {
    cr!("707.2", "707.9b");
    ruling!(
        "Applied Geometry",
        "It doesn't copy whether that permanent is tapped or untapped, whether it has any counters on it or Auras and Equipment attached to it, or any non-copy effects that have changed its power, toughness, types, color, or so on."
    );
    supported("Applied Geometry");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    attach_new(&mut t, P0, "Holy Strength", giant);
    t.g.add_counters(o(giant), "+1/+1", 2, None);
    cast_at(&mut t, P0, "Giant Growth", giant);
    t.g.objects[giant.0 as usize].tapped = true;
    // Hill Giant 3/3 + 2 counters + Holy Strength (+1/+2) + Giant Growth.
    assert_eq!(t.pt(giant), (9, 10));
    cast_at(&mut t, P0, "Applied Geometry", giant);
    let tokens: Vec<ObjectId> = t
        .g
        .permanents()
        .filter(|x| x.is_token())
        .map(|x| x.id)
        .collect();
    assert_eq!(tokens.len(), 1);
    let tok = tokens[0];
    let o2 = t.obj_now(tok);
    assert_eq!(o2.chars.name, "Hill Giant");
    assert!(!o2.tapped);
    assert!(o2.chars.has_subtype("Giant") && o2.chars.has_subtype("Fractal"));
    assert_eq!(o2.chars.colors, ColorSet::single(Color::Red));
    // 0/0 with exactly the six counters Applied Geometry put on it.
    assert_eq!(t.counters(tok, "+1/+1"), 6);
    assert_eq!(t.pt(tok), (6, 6));
    assert!(!t.g.permanents().any(|x| x.attached_to == Some(o(tok))));
}

#[test]
fn juniper_order_ranger_counters_come_after_entering_and_each_still_gets_one() {
    cr!("603.2", "603.6a", "608.2b");
    ruling!(
        "Juniper Order Ranger",
        "The creature doesn't enter with a +1/+1 counter on it. It enters, then the ability triggers. If either that creature or Juniper Order Ranger leaves the battlefield before the ability resolves, the remaining creature will still get a +1/+1 counter."
    );
    supported("Juniper Order Ranger");
    for gone in ["none", "creature", "ranger"] {
        let mut t = TestGame::new(2);
        let ranger = t.battlefield(P0, "Juniper Order Ranger");
        let bears = t.enter(P0, "Grizzly Bears");
        assert_eq!(t.counters(bears, "+1/+1"), 0);
        t.g.flush_events();
        t.settle();
        assert_eq!(t.stack_len(), 1);
        match gone {
            "creature" => destroy(&mut t, bears),
            "ranger" => destroy(&mut t, ranger),
            _ => {}
        }
        t.resolve_all();
        if gone != "creature" {
            assert_eq!(t.counters(bears, "+1/+1"), 1, "{gone}");
        }
        if gone != "ranger" {
            assert_eq!(t.counters(ranger, "+1/+1"), 1, "{gone}");
        }
        if gone == "creature" {
            assert_eq!(t.zone(bears), Zone::Graveyard(P0));
        }
    }
}
