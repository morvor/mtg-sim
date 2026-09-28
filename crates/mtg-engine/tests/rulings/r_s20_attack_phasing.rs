//! Rulings batch S20 — attacking: phased-out permanents are treated as though they don't
//! exist (CR 702.26b): they can't be targeted, their static abilities don't apply, their
//! triggered abilities don't trigger, and they can't attack or block; permanents that
//! phase in during their controller's untap step can attack that turn (Teferi's
//! Protection).

use crate::r_s01_common::*;
use crate::r_s04_common::{add_mana, spell_targets};
use crate::r_s09_common::legal_attack;
use crate::r_s20_common::*;
use mtg_engine::combat::block_options;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// P0 casts Teferi's Protection ("Until your next turn, your life total can't change and
/// you gain protection from everything. All permanents you control phase out.") in P0's
/// main phase, and it resolves.
fn teferis_protection(t: &mut TestGame) {
    add_mana(t, P0, ManaType::W, 1);
    add_mana(t, P0, ManaType::C, 2);
    let tp = t.hand(P0, "Teferi's Protection");
    t.cast(P0, tp).go();
    t.resolve_all();
}

#[test]
fn creatures_phasing_in_with_the_untap_step_can_attack_and_tap_that_turn() {
    cr!("702.26a", "702.26d", "302.6");
    ruling!(
        "Teferi's Protection",
        "Any creatures that phase in under your control as your next untap step begins will be able to attack and pay a cost of {T} during that turn."
    );
    supported("Teferi's Protection");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    // Llanowar Elves entered this turn: summoning sick now.
    let elves = entered_this_turn(&mut t, P0, "Llanowar Elves");
    teferis_protection(&mut t);
    assert!(phased_out(&t, bears) && phased_out(&t, elves));
    t.advance_to(P1, Step::PrecombatMain);
    assert!(phased_out(&t, bears) && phased_out(&t, elves));
    // They phase in as P0's untap step begins: the same objects, which P0 has controlled
    // continuously since the turn began.
    t.advance_to(P0, Step::PrecombatMain);
    assert!(!phased_out(&t, bears) && !phased_out(&t, elves));
    assert!(tap_for_mana(&mut t, P0, elves, "Add {G}"));
    to_beginning_of_combat(&mut t, P0);
    assert!(legal_attack(&mut t, &[(bears, Entity::Player(P1))]));
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    assert!(t.g.is_attacking(bears));
}

#[test]
fn life_gain_and_loss_have_no_effect_while_your_life_total_cant_change() {
    cr!("119.3", "608.2c");
    ruling!(
        "Teferi's Protection",
        "Spells and abilities that would normally cause you to gain or lose life still resolve while your life total can't change, but the life-gain or life-loss part simply has no effect."
    );
    supported("Teferi's Protection");
    supported("Revitalize");
    let mut t = TestGame::new(2);
    teferis_protection(&mut t);
    // Revitalize: "You gain 3 life. Draw a card." The card is drawn; no life is gained.
    let hand = t.hand_size(P0);
    add_mana(&mut t, P0, ManaType::W, 1);
    add_mana(&mut t, P0, ManaType::C, 1);
    let revitalize = t.hand(P0, "Revitalize");
    t.cast(P0, revitalize).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.hand_size(P0), hand + 1);
    // On P0's next turn, the life total can change again.
    t.advance_to(P1, Step::PrecombatMain);
    t.advance_to(P0, Step::PrecombatMain);
    add_mana(&mut t, P0, ManaType::W, 1);
    add_mana(&mut t, P0, ManaType::C, 1);
    let revitalize = t.hand(P0, "Revitalize");
    t.cast(P0, revitalize).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 23);
}

#[test]
fn a_spell_targeting_you_has_an_illegal_target_once_you_gain_protection_from_everything() {
    cr!("702.16b", "702.16j", "608.2b");
    ruling!(
        "Teferi's Protection",
        "Gaining protection from everything causes a spell or ability on the stack to have an illegal target if it targets you."
    );
    supported("Teferi's Protection");
    let mut t = TestGame::new(2);
    // P1's Lightning Bolt targets P0; P0 responds with Teferi's Protection.
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(Entity::Player(P0)).go();
    add_mana(&mut t, P0, ManaType::W, 1);
    add_mana(&mut t, P0, ManaType::C, 2);
    let tp = t.hand(P0, "Teferi's Protection");
    t.cast(P0, tp).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    assert!(t.in_graveyard(P1, "Lightning Bolt"));
    // P0 can't be targeted now.
    let targets = spell_targets(&mut t, P1, "Lightning Bolt");
    assert!(!targets.contains(&Entity::Player(P0)));
}

fn phased_out(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).phased_out
}

/// Whether P1 could choose `id` as the target of a Lightning Bolt now.
fn bolt_can_target(t: &mut TestGame, id: ObjectId) -> bool {
    spell_targets(t, P1, "Lightning Bolt").contains(&Entity::Object(id))
}

/// P1 attacks P0 with `attacker` in P1's turn: whether `blocker` could block it.
fn could_block(t: &mut TestGame, attacker: ObjectId, blocker: ObjectId) -> bool {
    t.g.combat = None;
    t.set_step(P1, Step::BeginningOfCombat);
    attack_with(t, &[(attacker, Entity::Player(P0))]);
    block_options(&t.g, &[P0])
        .iter()
        .any(|(b, attackers)| *b == blocker && attackers.contains(&attacker))
}

#[test]
fn creatures_phased_out_by_the_moment_dont_exist_until_it_leaves() {
    cr!("702.26b", "610.4");
    ruling!(
        "The Moment",
        "While a permanent is phased out, it's treated as though it doesn't exist. It can't be the target of spells or abilities, its static abilities have no effect on the game, its triggered abilities can't trigger, it can't attack or block, and so on."
    );
    supported("The Moment");
    let mut t = TestGame::new(2);
    // Benalish Marshal: "Other creatures you control get +1/+1." Soul Warden: "Whenever
    // another creature enters, you gain 1 life."
    let marshal = t.battlefield(P0, "Benalish Marshal");
    let warden = t.battlefield(P0, "Soul Warden");
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert_eq!(t.pt(bears), (3, 3));
    assert!(bolt_can_target(&mut t, marshal));
    // The Moment (legendary): "{2}, {T}: Untap target creature you control. It phases out
    // until The Moment leaves the battlefield."
    let moment_on = |t: &mut TestGame, creature: ObjectId| -> ObjectId {
        let moment = t.battlefield(P0, "The Moment");
        add_mana(t, P0, ManaType::C, 2);
        t.answer_targets(P0, &[Entity::Object(creature)]);
        crate::r_s06_common::activate_containing(t, P0, moment, "phases out")
            .expect("activate");
        t.resolve_all();
        assert!(phased_out(t, creature));
        moment
    };
    let moment = moment_on(&mut t, marshal);
    // Its static ability has no effect.
    t.g.recompute();
    assert_eq!(t.pt(bears), (2, 2));
    // It can't be targeted, and it can't attack.
    assert!(!bolt_can_target(&mut t, marshal));
    to_beginning_of_combat(&mut t, P0);
    assert!(!legal_attack(&mut t, &[(marshal, Entity::Player(P1))]));
    // It stays phased out through P0's untap step; in P1's turn it can't block.
    t.advance_to(P1, Step::PrecombatMain);
    t.advance_to(P0, Step::PrecombatMain);
    assert!(phased_out(&t, marshal));
    let giant = t.battlefield(P1, "Hill Giant");
    assert!(could_block(&mut t, giant, warden));
    assert!(!could_block(&mut t, giant, marshal));
    // The Moment leaves the battlefield: the Marshal phases in.
    crate::r_s02_common::destroy(&mut t, moment);
    t.g.recompute();
    assert!(!phased_out(&t, marshal));
    assert_eq!(t.pt(bears), (3, 3));
    // A phased-out Soul Warden's triggered ability doesn't trigger.
    t.set_step(P0, Step::PostcombatMain);
    moment_on(&mut t, warden);
    let life = t.life(P0);
    t.enter(P0, "Hill Giant");
    t.settle();
    t.resolve_all();
    assert_eq!(t.life(P0), life);
}

#[test]
fn a_creature_phased_out_by_slip_out_the_back_is_treated_as_though_it_doesnt_exist() {
    cr!("702.26b");
    ruling!(
        "Slip Out the Back",
        "Phased-out permanents are treated as though they don't exist. They can't be the target of spells or abilities, their static abilities have no effect on the game, their triggered abilities can't trigger, they can't attack or block, and so on."
    );
    supported("Slip Out the Back");
    let mut t = TestGame::new(2);
    // Benalish Marshal: "Other creatures you control get +1/+1."
    let marshal = t.battlefield(P0, "Benalish Marshal");
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert_eq!(t.pt(bears), (3, 3));
    assert!(bolt_can_target(&mut t, marshal));
    // "Put a +1/+1 counter on target creature. It phases out."
    add_mana(&mut t, P0, ManaType::U, 1);
    let slip = t.hand(P0, "Slip Out the Back");
    t.cast(P0, slip).target(marshal).go();
    t.resolve_all();
    assert!(phased_out(&t, marshal));
    // Its static ability has no effect.
    t.g.recompute();
    assert_eq!(t.pt(bears), (2, 2));
    // It can't attack.
    to_beginning_of_combat(&mut t, P0);
    assert!(!legal_attack(&mut t, &[(marshal, Entity::Player(P1))]));
    // It can't be the target of spells.
    assert!(!bolt_can_target(&mut t, marshal));
    // It can't block in P1's turn.
    let giant = t.battlefield(P1, "Hill Giant");
    assert!(could_block(&mut t, giant, bears));
    assert!(!could_block(&mut t, giant, marshal));
}

#[test]
fn king_of_the_oathbreakers_phased_out_cant_be_targeted_or_block() {
    cr!("702.26b");
    ruling!(
        "King of the Oathbreakers",
        "Phased-out permanents are treated as though they don't exist. They can't be the target of spells or abilities"
    );
    supported("King of the Oathbreakers");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let king = t.battlefield(P0, "King of the Oathbreakers");
    // "Whenever King of the Oathbreakers or another Spirit you control becomes the target
    // of a spell, it phases out." P1's Lightning Bolt is countered on resolution: its
    // target doesn't exist.
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(king).go();
    t.resolve_all();
    assert!(phased_out(&t, king));
    assert_eq!(t.obj_now(king).damage, 0);
    assert!(t.in_graveyard(P1, "Lightning Bolt"));
    // It can't be targeted again, and it can't block P1's attacker.
    assert!(!bolt_can_target(&mut t, king));
    let giant = t.battlefield(P1, "Hill Giant");
    assert!(!could_block(&mut t, giant, king));
}

#[test]
fn renegade_silent_phased_out_cant_be_targeted_or_block() {
    cr!("702.26b");
    ruling!(
        "Renegade Silent",
        "Phased-out permanents are treated as though they don't exist. They can't be the targets of spells or abilities, their static abilities have no effect on the game, their triggered abilities can't trigger, they can't attack or block, and so on."
    );
    supported("Renegade Silent");
    let mut t = TestGame::new(2);
    // "At the beginning of your end step, goad up to one target creature you don't
    // control and put a +1/+1 counter on this creature. This creature phases out."
    let silent = t.battlefield(P0, "Renegade Silent");
    t.answer_targets(P0, &[]);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(phased_out(&t, silent));
    // In P1's turn it can't be targeted and can't block.
    t.advance_to(P1, Step::PrecombatMain);
    assert!(phased_out(&t, silent));
    assert!(!bolt_can_target(&mut t, silent));
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert!(could_block(&mut t, giant, bears));
    assert!(!could_block(&mut t, giant, silent));
}

#[test]
fn permanents_phased_out_by_unite_the_coalition_dont_exist() {
    cr!("702.26b");
    ruling!(
        "Unite the Coalition",
        "Phased-out permanents are treated as though they don’t exist. They can’t be the targets of spells or abilities, their static abilities have no effect on the game, their triggered abilities can’t trigger, they can’t attack or block, and so on."
    );
    supported("Unite the Coalition");
    let mut t = TestGame::new(2);
    let anthem = t.battlefield(P0, "Glorious Anthem");
    let warden = t.battlefield(P0, "Soul Warden");
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert_eq!(t.pt(bears), (3, 3));
    // Three times "Target permanent phases out", and "Target player draws a card" twice.
    for ty in [
        ManaType::W,
        ManaType::U,
        ManaType::B,
        ManaType::R,
        ManaType::G,
    ] {
        add_mana(&mut t, P0, ty, 1);
    }
    add_mana(&mut t, P0, ManaType::C, 2);
    let unite = t.hand(P0, "Unite the Coalition");
    t.cast(P0, unite)
        .modes(&[0, 0, 0, 1, 1])
        .targets(&[Entity::Object(anthem)])
        .targets(&[Entity::Object(warden)])
        .targets(&[Entity::Object(bears)])
        .targets(&[Entity::Player(P0)])
        .targets(&[Entity::Player(P0)])
        .go();
    t.resolve_all();
    for p in [anthem, warden, bears] {
        assert!(phased_out(&t, p));
    }
    // The Anthem's static ability doesn't apply: a new creature is 2/2.
    let life = t.life(P0);
    let ogre = t.enter(P0, "Gray Ogre");
    t.settle();
    t.resolve_all();
    assert_eq!(t.pt(ogre), (2, 2));
    // Soul Warden's ability ("Whenever another creature enters, you gain 1 life.")
    // didn't trigger.
    assert_eq!(t.life(P0), life);
    // The Bears can't attack.
    to_beginning_of_combat(&mut t, P0);
    assert!(!legal_attack(&mut t, &[(bears, Entity::Player(P1))]));
}
