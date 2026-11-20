//! Item ruling-research-2, interpretation `that-opponent-fixed-as-it-triggers`: a
//! player-referring "that player" / "that opponent" whose antecedent is the trigger
//! condition's "an opponent" ("[creatures] an opponent controls ...", "a land enters under an
//! opponent's control", "a spell or ability an opponent controls"; also "enchanted player
//! controls") is the player the condition named as the ability triggered, whoever controls
//! the objects when it resolves; an object-based "its controller" / "that spell's
//! controller" is the object's controller as the ability resolves, or as it last existed
//! (CR 608.2h). The initiative's steal keeps the player it triggered for (an open question).
//!
//! The two-player control changes of Norn's Decree and Coveted Jewel are tested in
//! `r_residue_reflexive_refund_3.rs`; the object-based rulings with a control change in
//! their batches: Ankh of Mishra (`r_p091_landfall.rs`), Tainted Aether (`r_p138_misc.rs`),
//! Relic Putrescence (`r_p211_enchant_last_known.rs`), Celestial Mantle (`r_p206_life.rs`),
//! Archfiend of Spite (`r_p139_resolution.rs`).

use crate::r_p015_common::{enter, gain_control};
use crate::r_p177_common::{cast_targeting, ffa};
use crate::r_s01_common::{attack_with, supported, triggers_on_stack};
use crate::r_s02_common::destroy;
use crate::r_s06_common::attach_new;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// The objects of the event that made the top triggered ability of the stack trigger, in
/// the order the ability has them.
fn trigger_objects(t: &TestGame) -> Vec<ObjectId> {
    let top = *t.g.stack.last().expect("a triggered ability on the stack");
    t.g.obj(top)
        .stack
        .as_ref()
        .and_then(|s| s.event.as_ref())
        .map(|e| e.objects.clone())
        .unwrap_or_default()
}

// --- Coveted Jewel and Norn's Decree ----------------------------------------------------

/// P1 attacks P0 with `attackers`, unblocked; the game advances into the declare blockers
/// step with Coveted Jewel's trigger on the stack ("Whenever one or more creatures an
/// opponent controls attack you and aren't blocked, that player draws three cards and gains
/// control of this artifact. Untap it."). P0 controls the Jewel, tapped.
fn jewel_triggered(t: &mut TestGame, attackers: &[ObjectId]) -> ObjectId {
    supported("Coveted Jewel");
    let jewel = t.battlefield(P0, "Coveted Jewel");
    t.g.tap(jewel);
    t.set_step(P1, Step::BeginningOfCombat);
    let attacks: Vec<(ObjectId, Entity)> =
        attackers.iter().map(|a| (*a, Entity::Player(P0))).collect();
    attack_with(t, &attacks);
    t.resolve_all();
    t.answer(P0, DecisionKind::Blockers, Answer::Blockers(vec![]));
    t.advance_to(P1, Step::DeclareBlockers);
    t.settle();
    assert_eq!(triggers_on_stack(t, "aren't blocked"), 1);
    jewel
}

/// P1's `attackers` deal combat damage to P0, who controls Norn's Decree ("Whenever one or
/// more creatures an opponent controls deal combat damage to you, that opponent gets a
/// poison counter."); its trigger is on the stack.
fn decree_triggered(t: &mut TestGame, attackers: &[ObjectId]) {
    supported("Norn's Decree");
    t.battlefield(P0, "Norn's Decree");
    t.set_step(P1, Step::BeginningOfCombat);
    let attacks: Vec<(ObjectId, Entity)> =
        attackers.iter().map(|a| (*a, Entity::Player(P0))).collect();
    attack_with(t, &attacks);
    t.resolve_all();
    t.advance_to(P1, Step::CombatDamage);
    t.settle();
    assert_eq!(t.life(P0), 20 - 2 * attackers.len() as i32);
    assert_eq!(triggers_on_stack(t, "poison counter"), 1);
}

#[test]
fn coveted_jewel_that_player_stays_the_attacker_when_another_opponent_takes_the_creature() {
    cr!("603.2", "506.4");
    interp!("that-opponent-fixed-as-it-triggers");
    // Free-for-all: P1's unblocked Bears trigger P0's Jewel. In response P2, also P0's
    // opponent, gains control of them (they leave combat, CR 506.4). "That player" is still
    // P1, whose creature attacked: P1 draws three cards and gains control of the Jewel. Read
    // as the creature's controller as it resolves, it would be P2.
    let mut t = ffa(3);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let jewel = jewel_triggered(&mut t, &[bears]);
    let hands = (t.hand_size(P1), t.hand_size(P2));
    gain_control(&mut t, P2, bears);
    assert_eq!(t.obj_now(bears).controller, P2);
    assert!(!t.g.is_attacking(t.g.current(bears)), "removed from combat");
    t.resolve_all();
    assert_eq!(t.obj_now(jewel).controller, P1);
    assert_eq!((t.hand_size(P1), t.hand_size(P2)), (hands.0 + 3, hands.1));
}

#[test]
fn norns_decree_that_opponent_stays_the_attacker_when_another_opponent_takes_the_creature() {
    cr!("603.2");
    interp!("that-opponent-fixed-as-it-triggers");
    // Free-for-all: P1's Bears deal combat damage to P0; P2 gains control of them before
    // the Decree's ability resolves. P1 gets the poison counter, not P2.
    let mut t = ffa(3);
    let bears = t.battlefield(P1, "Grizzly Bears");
    decree_triggered(&mut t, &[bears]);
    gain_control(&mut t, P2, bears);
    t.resolve_all();
    assert_eq!(t.g.player(P1).poison(), 1);
    assert_eq!(t.g.player(P2).poison(), 0);
}

#[test]
fn coveted_jewel_that_player_when_only_some_of_the_creatures_change_control() {
    cr!("603.2", "603.2c");
    interp!("that-opponent-fixed-as-it-triggers");
    // Two of P1's creatures attack P0 unblocked: one trigger, for P1 (CR 603.2c). P0 gains
    // control of the first of them in response; the other is still P1's. P1 draws three and
    // gains control of the Jewel (the first creature's controller as the ability resolves
    // would be P0).
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    let jewel = jewel_triggered(&mut t, &[a, b]);
    let creatures = trigger_objects(&t);
    assert_eq!(creatures.len(), 2);
    let hands = (t.hand_size(P0), t.hand_size(P1));
    gain_control(&mut t, P0, creatures[0]);
    t.resolve_all();
    assert_eq!(t.obj_now(jewel).controller, P1);
    assert_eq!((t.hand_size(P0), t.hand_size(P1)), (hands.0, hands.1 + 3));
}

#[test]
fn norns_decree_that_opponent_when_only_some_of_the_creatures_change_control() {
    cr!("603.2", "603.2c");
    interp!("that-opponent-fixed-as-it-triggers");
    // Two of P1's Bears deal combat damage to P0: one trigger. P0 gains control of the
    // first of them before it resolves: P1 gets the poison counter.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    decree_triggered(&mut t, &[a, b]);
    let creatures = trigger_objects(&t);
    assert_eq!(creatures.len(), 2);
    gain_control(&mut t, P0, creatures[0]);
    t.resolve_all();
    assert_eq!(t.g.player(P1).poison(), 1);
    assert_eq!(t.g.player(P0).poison(), 0);
}

#[test]
fn coveted_jewel_that_player_when_the_creature_leaves_the_battlefield() {
    cr!("603.2");
    interp!("that-opponent-fixed-as-it-triggers");
    // The Bears are destroyed before the ability resolves. "That player" is P1, the player
    // the trigger named: also the Bears' controller as they last existed, unless P0 had
    // gained control of them first, when they last existed under P0's control and "that
    // player" is still P1.
    for stolen_first in [false, true] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        let jewel = jewel_triggered(&mut t, &[bears]);
        let hands = (t.hand_size(P0), t.hand_size(P1));
        if stolen_first {
            gain_control(&mut t, P0, bears);
        }
        destroy(&mut t, bears);
        assert!(!t.on_battlefield(bears));
        t.resolve_all();
        assert_eq!(
            t.obj_now(jewel).controller,
            P1,
            "stolen first: {stolen_first}"
        );
        assert_eq!(
            (t.hand_size(P0), t.hand_size(P1)),
            (hands.0, hands.1 + 3),
            "stolen first: {stolen_first}"
        );
    }
}

#[test]
fn norns_decree_that_opponent_when_the_creature_leaves_the_battlefield() {
    cr!("603.2");
    interp!("that-opponent-fixed-as-it-triggers");
    // The Bears die before the Decree's ability resolves (after P0 gained control of them,
    // in the second game): P1 gets the poison counter.
    for stolen_first in [false, true] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        decree_triggered(&mut t, &[bears]);
        if stolen_first {
            gain_control(&mut t, P0, bears);
        }
        destroy(&mut t, bears);
        t.resolve_all();
        assert_eq!(t.g.player(P1).poison(), 1, "stolen first: {stolen_first}");
        assert_eq!(t.g.player(P0).poison(), 0, "stolen first: {stolen_first}");
    }
}

// --- "[a permanent] an opponent controls [event], ... that player" -------------------------

#[test]
fn blood_seeker_that_player_is_the_opponent_the_creature_entered_under() {
    cr!("603.6a");
    interp!("that-opponent-fixed-as-it-triggers");
    supported("Blood Seeker");
    // "Whenever a creature an opponent controls enters, you may have that player lose 1
    // life." P1's Bears enter; P0 gains control of them in response. P1, the opponent they
    // entered under, loses the life; their controller as the ability resolves is P0.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Blood Seeker");
    let bears = enter(&mut t, P1, "Grizzly Bears");
    assert_eq!(triggers_on_stack(&t, "lose 1 life"), 1);
    gain_control(&mut t, P0, bears);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (20, 19));
}

#[test]
fn trespassers_curse_that_player_is_the_enchanted_player_the_creature_entered_under() {
    cr!("603.6a", "303.4b");
    interp!("that-opponent-fixed-as-it-triggers");
    supported("Trespasser's Curse");
    // "Whenever a creature enchanted player controls enters, that player loses 1 life and
    // you gain 1 life." The Curse (P0's) enchants P1; P1's Bears enter, and P0 gains control
    // of them in response. "That player" is P1, the enchanted player they entered under, as
    // for "a creature an opponent controls": P1 loses the life. Read as the Bears'
    // controller as the ability resolves, it would be P0 (losing 1 life and gaining 1).
    let mut t = TestGame::new(2);
    attach_new(&mut t, P0, "Trespasser's Curse", P1);
    let bears = enter(&mut t, P1, "Grizzly Bears");
    assert_eq!(triggers_on_stack(&t, "loses 1 life"), 1);
    gain_control(&mut t, P0, bears);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (21, 19));
}

#[test]
fn tunnel_ignus_that_player_is_the_opponent_the_land_entered_under() {
    cr!("603.4");
    interp!("that-opponent-fixed-as-it-triggers");
    supported("Tunnel Ignus");
    // "Whenever a land enters under an opponent's control, if that player had another land
    // enter the battlefield under their control this turn, this creature deals 3 damage to
    // that player." A second land enters under P1's control; P0 gains control of it in
    // response. As the ability resolves its intervening "if" is checked again (CR 603.4),
    // for P1, who still had another land enter this turn: P1 is dealt 3 damage. Read as the
    // land's controller then (P0, who had no land enter), the ability would do nothing.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Tunnel Ignus");
    enter(&mut t, P1, "Forest");
    assert_eq!(triggers_on_stack(&t, "3 damage"), 0, "the first land");
    let land = enter(&mut t, P1, "Forest");
    assert_eq!(triggers_on_stack(&t, "3 damage"), 1);
    gain_control(&mut t, P0, land);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (20, 17));
}

#[test]
fn michiko_konda_that_player_is_the_sources_controller_as_it_dealt_damage() {
    cr!("510.2", "603.2");
    interp!("that-opponent-fixed-as-it-triggers");
    supported("Michiko Konda, Truth Seeker");
    // "Whenever a source an opponent controls deals damage to you, that player sacrifices a
    // permanent of their choice." P1's Bears deal combat damage to P0; P0 gains control of
    // them before the ability resolves. P1 sacrifices a permanent (its Forest, its only one
    // left), not P0: "that source's controller" as it resolves would be P0 (Archfiend of
    // Spite's ruling, for that wording).
    let mut t = TestGame::new(2);
    let michiko = t.battlefield(P0, "Michiko Konda, Truth Seeker");
    let forest = t.battlefield(P1, "Forest");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::BeginningOfCombat);
    attack_with(&mut t, &[(bears, Entity::Player(P0))]);
    t.resolve_all();
    t.advance_to(P1, Step::CombatDamage);
    t.settle();
    assert_eq!(t.life(P0), 18);
    assert_eq!(triggers_on_stack(&t, "sacrifices"), 1);
    gain_control(&mut t, P0, bears);
    t.resolve_all();
    assert!(!t.on_battlefield(forest), "P1 sacrificed its Forest");
    assert!(t.on_battlefield(michiko) && t.on_battlefield(bears));
    assert_eq!(t.obj_now(bears).controller, P0);
}

// --- "a spell or ability an opponent controls": ward's "that player" and "its controller" ---

/// P1 casts Lightning Bolt at P0's `name` (an ability of it triggers); in response P0 gains
/// control of the Bolt with Aethersnatch, keeping its target. P1 would pay {2} for the
/// Bolt, P0 wouldn't. Returns the game, with the stack resolved, and the creature.
fn bolt_stolen_in_response(name: &str) -> (TestGame, ObjectId) {
    supported(name);
    supported("Aethersnatch");
    let mut t = TestGame::new(2);
    let creature = t.battlefield(P0, name);
    t.lands(P1, "Mountain", 3);
    let bolt = t.hand(P1, "Lightning Bolt");
    let spell = t.cast(P1, bolt).target(creature).go();
    t.settle();
    assert_eq!(t.stack_len(), 2, "the Bolt and the triggered ability");
    cast_targeting(&mut t, P0, "Aethersnatch", &[Entity::Object(spell)]);
    // Aethersnatch: P0 keeps the Bolt's target. The payment: P0 declines, P1 would pay.
    t.answer_yes(P0, false);
    t.answer_yes(P0, false);
    t.answer_yes(P1, true);
    t.resolve();
    assert_eq!(t.obj_now(spell).controller, P0);
    t.resolve_all();
    (t, creature)
}

#[test]
fn ward_that_player_is_the_opponent_who_controlled_the_spell_as_it_targeted() {
    cr!("702.21a");
    interp!("that-opponent-fixed-as-it-triggers");
    // Ward {2}: "Whenever this permanent becomes the target of a spell or ability an
    // opponent controls, counter that spell or ability unless that player pays [cost]"
    // (CR 702.21a). "That player" is P1, who controlled the Bolt as it targeted: P1 pays
    // {2}, and the Bolt (now P0's) resolves and destroys the Honor Guard. Asked of the
    // Bolt's controller as ward resolves (P0, who declines), it would have been countered.
    let (t, guard) = bolt_stolen_in_response("Tomakul Honor Guard");
    assert!(!t.on_battlefield(guard), "the Bolt resolved");
    assert!(t.in_graveyard(P1, "Lightning Bolt"));
    assert!(
        t.g.battlefield
            .iter()
            .filter(|id| t.g.obj(**id).controller == P1)
            .all(|id| t.g.obj(*id).tapped),
        "P1 paid {{2}} with its last two Mountains"
    );
}

#[test]
fn frost_titan_its_controller_is_the_spells_controller_as_the_ability_resolves() {
    cr!("608.2h");
    interp!("that-opponent-fixed-as-it-triggers");
    // Frost Titan: "Whenever this creature becomes the target of a spell or ability an
    // opponent controls, counter that spell or ability unless its controller pays {2}." The
    // object-based "its controller" is the Bolt's controller as the ability resolves, P0
    // (CR 608.2h), who declines: the Bolt is countered. P1, who cast it, would have paid.
    let (t, titan) = bolt_stolen_in_response("Frost Titan");
    assert_eq!(t.obj_now(titan).damage, 0, "the Bolt was countered");
    assert!(t.in_graveyard(P1, "Lightning Bolt"));
    let untapped =
        t.g.battlefield
            .iter()
            .filter(|id| t.g.obj(**id).controller == P1 && !t.g.obj(**id).tapped)
            .count();
    assert_eq!(untapped, 2, "P1 paid nothing");
}

#[test]
fn bonecrusher_giant_that_spells_controller_is_read_as_the_ability_resolves() {
    cr!("608.2h");
    interp!("that-opponent-fixed-as-it-triggers");
    supported("Bonecrusher Giant // Stomp");
    supported("Aethersnatch");
    // "Whenever this creature becomes the target of a spell, this creature deals 2 damage
    // to that spell's controller." P1's Shock targets P0's Giant; P0 gains control of the
    // Shock in response. The Giant deals its damage to the Shock's controller as the
    // ability resolves (CR 608.2h), P0, not to P1, who cast it.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Bonecrusher Giant // Stomp");
    t.lands(P1, "Mountain", 1);
    let shock = t.hand(P1, "Shock");
    let spell = t.cast(P1, shock).target(giant).go();
    t.settle();
    assert_eq!(
        triggers_on_stack(&t, "2 damage to that spell's controller"),
        1
    );
    cast_targeting(&mut t, P0, "Aethersnatch", &[Entity::Object(spell)]);
    t.answer_yes(P0, false);
    t.resolve();
    assert_eq!(t.obj_now(spell).controller, P0);
    t.resolve();
    assert_eq!((t.life(P0), t.life(P1)), (18, 20));
}

// --- The monarch (CR 725.2) -------------------------------------------------------------

#[test]
fn monarch_its_controller_is_the_creatures_controller_as_the_ability_resolves() {
    cr!("725.2", "608.2h");
    interp!("that-opponent-fixed-as-it-triggers");
    // CR 725.2: "Whenever a creature deals combat damage to the monarch, its controller
    // becomes the monarch." P1's Bears deal combat damage to P0, the monarch, and P0 gains
    // control of them before the ability resolves: their controller as it resolves, P0,
    // stays the monarch (CR 608.2h). If they leave the battlefield instead, their
    // controller as they last existed, P1, becomes the monarch.
    for stolen in [true, false] {
        let mut t = TestGame::new(2);
        mtg_engine::designations::become_monarch(&mut t.g, P0);
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.set_step(P1, Step::BeginningOfCombat);
        attack_with(&mut t, &[(bears, Entity::Player(P0))]);
        t.resolve_all();
        t.advance_to(P1, Step::CombatDamage);
        t.settle();
        assert_eq!(t.life(P0), 18);
        assert_eq!(triggers_on_stack(&t, "becomes the monarch"), 1);
        if stolen {
            gain_control(&mut t, P0, bears);
        } else {
            destroy(&mut t, bears);
        }
        t.resolve_all();
        let expected = if stolen { P0 } else { P1 };
        assert_eq!(t.g.monarch, Some(expected), "stolen: {stolen}");
    }
}

#[test]
fn initiative_the_player_whose_creatures_dealt_the_damage_takes_it_after_a_control_change() {
    cr!("726.2");
    interp!("that-opponent-fixed-as-it-triggers");
    // "Whenever one or more creatures a player controls deal combat damage to you, that
    // player takes the initiative" (The Initiative; CR 726.2: "... the controller of those
    // creatures takes the initiative"; the release notes and rulings: "the first player
    // takes the initiative"). P1's Bears deal combat damage to P0, who has the initiative;
    // P0 gains control of them before the ability resolves. P1, the player the ability was
    // created for, takes the initiative (an open question: read as the creatures'
    // controller as the ability resolves, P0 would keep it). If the Bears leave the
    // battlefield instead, P1 takes it under either reading.
    for stolen in [true, false] {
        let mut t = TestGame::new(2);
        mtg_engine::designations::take_initiative(&mut t.g, P0);
        t.g.flush_events();
        t.resolve_all();
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.set_step(P1, Step::BeginningOfCombat);
        attack_with(&mut t, &[(bears, Entity::Player(P0))]);
        t.resolve_all();
        t.advance_to(P1, Step::CombatDamage);
        t.settle();
        assert_eq!(t.life(P0), 18);
        assert_eq!(triggers_on_stack(&t, "takes the initiative"), 1);
        if stolen {
            gain_control(&mut t, P0, bears);
        } else {
            destroy(&mut t, bears);
        }
        t.resolve_all();
        assert_eq!(t.g.initiative, Some(P1), "stolen: {stolen}");
    }
}
