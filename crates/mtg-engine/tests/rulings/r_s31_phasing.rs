//! Rulings batch S31 — phasing (CR 702.26): phasing out and in isn't leaving or entering
//! the battlefield (CR 702.26d), Auras and Equipment attached to a permanent phase out and
//! in with it, still attached (CR 702.26g), and choices made as a permanent entered are
//! remembered.

use crate::r_s01_common::supported;
use crate::r_s06_common::{attach_new, attached_to};
use crate::r_s11_common::triggered_from;
use crate::r_s25_common::lands_for_cost;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn phased_out(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).phased_out
}

/// Advances to P0's next upkeep (P0's permanents phase in during the untap step).
fn to_next_upkeep(t: &mut TestGame) {
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
}

/// P0 casts the real card `name` (with lands for its mana cost) with the given targets,
/// one per target slot, and everything on the stack resolves.
fn cast(t: &mut TestGame, name: &str, targets: &[Entity]) {
    supported(name);
    lands_for_cost(t, P0, name);
    let card = t.hand(P0, name);
    t.cast_with(P0, card, targets)
        .unwrap_or_else(|e| panic!("casting {name} failed: {e:?}"));
    t.resolve_all();
}

/// P0's Grizzly Bears with P0's Bonesplitter and P1's Pacifism attached.
fn equipped_bears(t: &mut TestGame) -> (ObjectId, ObjectId, ObjectId) {
    let bears = t.battlefield(P0, "Grizzly Bears");
    let splitter = attach_new(t, P0, "Bonesplitter", bears);
    let pacifism = attach_new(t, P1, "Pacifism", bears);
    (bears, splitter, pacifism)
}

/// The Bears, Bonesplitter and Pacifism are all phased out, then all phased in with both
/// still attached to the Bears.
fn phase_together(t: &mut TestGame, (bears, splitter, pacifism): (ObjectId, ObjectId, ObjectId)) {
    for id in [bears, splitter, pacifism] {
        assert!(phased_out(t, id));
    }
    to_next_upkeep(t);
    for id in [bears, splitter, pacifism] {
        assert!(!phased_out(t, id));
        assert!(t.on_battlefield(id));
    }
    assert_eq!(attached_to(t, splitter), Some(Entity::Object(bears)));
    assert_eq!(attached_to(t, pacifism), Some(Entity::Object(bears)));
    t.g.recompute();
    assert_eq!(t.pt(bears), (4, 2));
}

#[test]
fn slip_out_the_back_doesnt_trigger_leaves_or_enters_abilities() {
    cr!("702.26d", "603.6a", "603.6c");
    ruling!(
        "Slip Out the Back",
        "Phasing out doesn't cause any “leaves the battlefield” abilities to trigger. Similarly, phasing in won't cause any “enters the battlefield” abilities to trigger."
    );
    supported("Outpost Siege");
    // Slip Out the Back: "Put a +1/+1 counter on target creature. It phases out."
    // Outpost Siege (Dragons): "Whenever a creature you control leaves the battlefield,
    // this enchantment deals 1 damage to any target." Elvish Visionary: "When this
    // creature enters, draw a card." P1's Soul Warden: "Whenever another creature enters,
    // you gain 1 life."
    let mut t = TestGame::new(2);
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    let siege = t.enter(P0, "Outpost Siege");
    t.settle();
    let warden = t.battlefield(P1, "Soul Warden");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let visionary = t.battlefield(P0, "Elvish Visionary");
    cast(&mut t, "Slip Out the Back", &[Entity::Object(visionary)]);
    assert!(phased_out(&t, visionary));
    assert_eq!(triggered_from(&t, siege), 0);
    assert_eq!(t.life(P1), 20);
    let hand = t.hand_size(P0);
    to_next_upkeep(&mut t);
    assert!(!phased_out(&t, visionary));
    assert_eq!(t.counters(visionary, counters::PLUS1), 1);
    assert_eq!(triggered_from(&t, warden), 0);
    assert_eq!(triggered_from(&t, visionary), 0);
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.hand_size(P0), hand);
    assert_eq!(t.life(P1), 20);
    // A creature that really leaves does trigger the Siege.
    t.answer_targets(P0, &[Entity::Player(P1)]);
    crate::r_s02_common::destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(triggered_from(&t, siege), 1);
    assert_eq!(t.life(P1), 19);
}

#[test]
fn unite_the_coalition_phases_out_a_creature_with_its_aura_and_equipment() {
    cr!("702.26d", "702.26g", "700.2d");
    ruling!(
        "Unite the Coalition",
        "Phasing out doesn’t cause any “leaves the battlefield” abilities to trigger. Similarly, phasing in won’t cause any “enters the battlefield” abilities to trigger."
    );
    ruling!(
        "Unite the Coalition",
        "As a permanent is phased out, Auras and Equipment attached to it also phase out at the same time. Those Auras and Equipment will phase in at the same time that creature does, and they’ll phase in still attached to that permanent."
    );
    // "Choose five. You may choose the same mode more than once. • Target permanent phases
    // out. • Target player draws a card. ..."
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Soul Warden");
    let set = equipped_bears(&mut t);
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0, 1, 1, 1, 1]));
    let hand = t.hand_size(P0);
    cast(
        &mut t,
        "Unite the Coalition",
        &[
            Entity::Object(set.0),
            Entity::Player(P0),
            Entity::Player(P0),
            Entity::Player(P0),
            Entity::Player(P0),
        ],
    );
    assert_eq!(t.hand_size(P0), hand + 4);
    phase_together(&mut t, set);
    // Soul Warden didn't see anything enter.
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn teferis_protection_doesnt_trigger_leaves_or_enters_abilities() {
    cr!("702.26d", "603.6a", "603.6c");
    ruling!(
        "Teferi's Protection",
        "Phasing out doesn't cause any \"leaves the battlefield\" abilities to trigger. Similarly, phasing in won't cause any \"enters\" abilities to trigger."
    );
    supported("Fiend Hunter");
    // Fiend Hunter: "When this creature enters, you may exile another target creature.
    // When this creature leaves the battlefield, return the exiled card to the battlefield
    // under its owner's control."
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Soul Warden");
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[Entity::Object(giant)]);
    t.answer_yes(P0, true);
    let hunter = t.enter(P0, "Fiend Hunter");
    t.resolve_all();
    assert_eq!(t.zone(giant), mtg_engine::object::Zone::Exile);
    let life = t.life(P1);
    // Teferi's Protection: "... All permanents you control phase out. ..."
    cast(&mut t, "Teferi's Protection", &[]);
    assert!(phased_out(&t, hunter));
    // The Giant didn't return.
    assert_eq!(t.zone(giant), mtg_engine::object::Zone::Exile);
    to_next_upkeep(&mut t);
    assert!(!phased_out(&t, hunter));
    // Neither Fiend Hunter's enters ability nor Soul Warden triggered.
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.life(P1), life);
    assert_eq!(t.zone(giant), mtg_engine::object::Zone::Exile);
}

#[test]
fn teferis_protection_phases_in_auras_and_equipment_still_attached() {
    cr!("702.26g", "702.26d");
    ruling!(
        "Teferi's Protection",
        "Each Aura and Equipment that phases out attached to a permanent that's phasing out phases in with that permanent and still attached to it."
    );
    let mut t = TestGame::new(2);
    let set = equipped_bears(&mut t);
    cast(&mut t, "Teferi's Protection", &[]);
    phase_together(&mut t, set);
}

#[test]
fn clever_concealment_phases_in_auras_and_equipment_still_attached() {
    cr!("702.26g", "702.26d");
    ruling!(
        "Clever Concealment",
        "As a permanent is phased out, Auras and Equipment attached to it also phase out at the same time. Those Auras and Equipment will phase in at the same time that permanent does, and they'll phase in still attached to that permanent."
    );
    // "Any number of target nonland permanents you control phase out."
    let mut t = TestGame::new(2);
    let set = equipped_bears(&mut t);
    cast(&mut t, "Clever Concealment", &[Entity::Object(set.0)]);
    phase_together(&mut t, set);
}

#[test]
fn a_choice_made_as_it_entered_is_remembered_after_it_phases_in() {
    cr!("702.26d", "607.2a", "614.12");
    ruling!(
        "King of the Oathbreakers",
        "Choices made for permanents as they entered the battlefield are remembered even after they phase in."
    );
    supported("King of the Oathbreakers");
    supported("Chameleon Spirit");
    // King of the Oathbreakers: "Whenever King of the Oathbreakers or another Spirit you
    // control becomes the target of a spell, it phases out." Chameleon Spirit (a Spirit):
    // "As this creature enters, choose a color. Chameleon Spirit's power and toughness are
    // each equal to the number of permanents of the chosen color your opponents control."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "King of the Oathbreakers");
    t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Hill Giant");
    let green = Color::ALL.iter().position(|c| *c == Color::Green).unwrap();
    t.answer(P0, DecisionKind::Option, Answer::Index(green));
    let spirit = t.enter(P0, "Chameleon Spirit");
    t.settle();
    t.g.recompute();
    assert_eq!(t.pt(spirit), (2, 2));
    // P0 targets it with Giant Growth: it phases out, and the spell doesn't resolve.
    cast(&mut t, "Giant Growth", &[Entity::Object(spirit)]);
    assert!(phased_out(&t, spirit));
    to_next_upkeep(&mut t);
    t.resolve_all();
    assert!(!phased_out(&t, spirit));
    // Still green: two green permanents.
    assert_eq!(t.obj_now(spirit).choices.color, Some(Color::Green));
    t.g.recompute();
    assert_eq!(t.pt(spirit), (2, 2));
}
