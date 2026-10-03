//! CR 702.55 Haunt.

use crate::common_k702_011_017::assert_supported;
use crate::common_k702_052_066::*;
use mtg_engine::ability::*;
use mtg_engine::decision::Answer;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::CardType;
use mtg_engine::*;

const HAUNT: &str = "Haunt";

/// The card named `name` in exile.
fn exiled(t: &TestGame, name: &str) -> ObjectId {
    *t.g.exile
        .iter()
        .find(|o| t.g.obj(**o).chars.name == name)
        .expect("not in exile")
}

/// Puts `name` onto the battlefield for P0 and lets it die, haunting `victim`.
fn haunt(t: &mut TestGame, name: &str, victim: ObjectId) -> ObjectId {
    let card = t.battlefield(P0, name);
    // Targets are chosen as the haunt ability is put on the stack.
    t.answer_targets(P0, &[Entity::Object(victim)]);
    destroy(t, card);
    t.settle();
    assert_eq!(stack_triggers(t, HAUNT).len(), 1);
    t.resolve();
    exiled(t, name)
}

#[test]
fn a_permanent_with_haunt_is_exiled_haunting_a_creature_when_it_dies() {
    cr!("702.55", "702.55a", "702.55c");
    assert_supported("Blind Hunter");
    let mut t = TestGame::new(2);
    // "When this creature enters or the creature it haunts dies, target player loses 2
    // life and you gain 2 life."
    t.answer_targets(P0, &[Entity::Player(P1)]);
    let hunter = t.enter(P0, "Blind Hunter");
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (22, 18));
    // It dies: haunt exiles it haunting target creature (an opponent's here).
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    destroy(&mut t, hunter);
    t.settle();
    assert_eq!(stack_triggers(&t, HAUNT).len(), 1);
    t.resolve();
    let ghost = exiled(&t, "Blind Hunter");
    assert!(!t.in_graveyard(P0, "Blind Hunter"));
    // When the creature it haunts dies, its ability triggers from exile.
    t.answer_targets(P0, &[Entity::Player(P1)]);
    destroy(&mut t, bears);
    t.settle();
    let triggers = stack_triggers(
        &t,
        "When ~ enters or the creature it haunts dies, target player loses 2 life and you gain 2 life.",
    );
    assert_eq!(triggers.len(), 1);
    assert_eq!(triggers[0].1, ghost);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (24, 16));
    // The card stays in exile, haunting nothing.
    assert_eq!(t.zone(ghost), Zone::Exile);
    let other = t.battlefield(P1, "Grizzly Bears");
    destroy(&mut t, other);
    t.settle();
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn an_instant_or_sorcery_with_haunt_haunts_after_resolving() {
    cr!("702.55a", "702.55c");
    assert_supported("Cry of Contrition");
    let mut t = TestGame::new(2);
    t.hand(P1, "Grizzly Bears");
    t.hand(P1, "Grizzly Bears");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Swamp", 1);
    let cry = t.hand(P0, "Cry of Contrition");
    t.cast(P0, cry).target(P1).go();
    // Put into the graveyard as it resolved: haunt triggers, targeting a creature.
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.resolve();
    assert_eq!(t.hand_size(P1), 1);
    assert_eq!(stack_triggers(&t, HAUNT).len(), 1);
    t.resolve();
    exiled(&t, "Cry of Contrition");
    // "When the creature this card haunts dies, target player discards a card." The
    // haunted creature can be the caster's own.
    t.answer_targets(P0, &[Entity::Player(P1)]);
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.hand_size(P1), 0);
}

#[test]
fn a_sorcery_haunts_a_creature_and_repeats_its_effect_when_it_dies() {
    cr!("702.55a", "702.55c");
    assert_supported("Benediction of Moons");
    let mut t = TestGame::new(3);
    let bears = t.battlefield(P2, "Grizzly Bears");
    t.lands(P0, "Plains", 1);
    let moons = t.hand(P0, "Benediction of Moons");
    // "You gain 1 life for each player."
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.cast(P0, moons).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 23);
    exiled(&t, "Benediction of Moons");
    // "When the creature this card haunts dies, you gain 1 life for each player."
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.life(P0), 26);
}

#[test]
fn a_spell_that_doesnt_resolve_doesnt_haunt() {
    cr!("702.55a");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Swamp", 4);
    let seize = t.hand(P0, "Seize the Soul");
    t.cast(P0, seize).target(bears).go();
    // Its only target leaves the battlefield: it doesn't resolve (CR 608.2b).
    run_effect(
        &mut t,
        None,
        P1,
        Effect::Move {
            what: Sel::Target(0),
            to: Destination::zone(ZoneKind::Hand),
        },
        &[Entity::Object(bears)],
    );
    t.resolve();
    assert!(t.in_graveyard(P0, "Seize the Soul"));
    t.settle();
    assert!(stack_triggers(&t, HAUNT).is_empty());
    assert!(t.in_graveyard(P0, "Seize the Soul"));
}

#[test]
fn haunt_needs_a_target_and_the_card_still_in_the_graveyard() {
    cr!("702.55a");
    let mut t = TestGame::new(2);
    // No creature to target: the ability is removed from the stack; the card stays.
    let thrull = t.battlefield(P0, "Absolver Thrull");
    destroy(&mut t, thrull);
    t.settle();
    assert!(stack_triggers(&t, HAUNT).is_empty());
    assert!(t.in_graveyard(P0, "Absolver Thrull"));
    // The card leaves the graveyard before the ability resolves: nothing is exiled.
    let bears = t.battlefield(P1, "Grizzly Bears");
    let thrull = t.battlefield(P0, "Absolver Thrull");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    destroy(&mut t, thrull);
    t.settle();
    assert_eq!(stack_triggers(&t, HAUNT).len(), 1);
    let in_gy = t.g.current(thrull);
    run_effect(
        &mut t,
        None,
        P0,
        Effect::Move {
            what: Sel::Target(0),
            to: Destination::zone(ZoneKind::Hand),
        },
        &[Entity::Object(in_gy)],
    );
    t.resolve_all();
    assert_eq!(t.zone(thrull), Zone::Hand(P0));
    assert!(t.g.exile.is_empty());
}

#[test]
fn the_creature_it_haunts_is_the_targeted_object_even_if_no_longer_a_creature() {
    cr!("702.55b");
    assert_supported("Belfry Spirit");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    haunt(&mut t, "Belfry Spirit", bears);
    // The haunted permanent stops being a creature, then is put into a graveyard.
    run_effect(
        &mut t,
        None,
        P1,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![
                Modification::RemoveTypes(vec![CardType::Creature]),
                Modification::AddTypes(vec![CardType::Artifact]),
            ],
            duration: Duration::Permanent,
        },
        &[Entity::Object(bears)],
    );
    assert!(!t.obj_now(bears).is_creature());
    destroy(&mut t, bears);
    t.resolve_all();
    // "create two 1/1 black Bat creature tokens with flying"
    assert_eq!(t.g.permanents().filter(|o| o.is_token()).count(), 2);
}

#[test]
fn a_haunted_creature_that_changes_zones_is_no_longer_haunted() {
    cr!("702.55b");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    haunt(&mut t, "Belfry Spirit", bears);
    // Returned to its owner's hand and put back onto the battlefield: a new object.
    run_effect(
        &mut t,
        None,
        P0,
        Effect::Move {
            what: Sel::Target(0),
            to: Destination::zone(ZoneKind::Hand),
        },
        &[Entity::Object(bears)],
    );
    let in_hand = t.g.current(bears);
    run_effect(
        &mut t,
        None,
        P1,
        Effect::Move {
            what: Sel::Target(0),
            to: Destination::battlefield(),
        },
        &[Entity::Object(in_hand)],
    );
    assert!(t.on_battlefield(bears));
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.g.permanents().filter(|o| o.is_token()).count(), 0);
}

#[test]
fn the_haunt_trigger_chooses_its_mode_and_targets_anew() {
    cr!("702.55c");
    ruling!(
        "Orzhov Pontiff",
        "The mode chosen when this creature enters may be different than the mode chosen when the creature it haunts is put into a graveyard."
    );
    assert_supported("Orzhov Pontiff");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Hill Giant");
    // Enters: "Creatures you control get +1/+1 until end of turn."
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0]));
    let pontiff = t.enter(P0, "Orzhov Pontiff");
    t.resolve_all();
    assert_eq!(t.pt(mine), (3, 3));
    assert_eq!(t.pt(theirs), (3, 3));
    // Dies, haunting the opponent's Hill Giant.
    t.answer_targets(P0, &[Entity::Object(theirs)]);
    destroy(&mut t, pontiff);
    t.resolve_all();
    // The haunted creature dies: "Creatures you don't control get -1/-1."
    let other = t.battlefield(P1, "Grizzly Bears");
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1]));
    destroy(&mut t, theirs);
    t.resolve_all();
    assert_eq!(t.pt(other), (1, 1));
    assert_eq!(t.pt(mine), (3, 3));
}

#[test]
fn a_haunt_trigger_with_no_legal_target_does_nothing() {
    cr!("702.55c");
    ruling!(
        "Seize the Soul",
        "If you can't choose a nonwhite nonblack creature to target when the haunted creature is put into a graveyard, or if the target becomes illegal, you don't get a token."
    );
    assert_supported("Seize the Soul");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let angel = t.battlefield(P1, "Serra Angel");
    t.lands(P0, "Swamp", 4);
    let seize = t.hand(P0, "Seize the Soul");
    t.cast(P0, seize).target(bears).go();
    // After resolving, it haunts Serra Angel.
    t.answer_targets(P0, &[Entity::Object(angel)]);
    t.resolve();
    // Bears destroyed and a Spirit token created.
    assert_eq!(t.g.permanents().filter(|o| o.is_token()).count(), 1);
    t.resolve();
    exiled(&t, "Seize the Soul");
    // Serra Angel dies; the only creature left is the white Spirit token: no target, so
    // the ability does nothing (no token).
    destroy(&mut t, angel);
    t.resolve_all();
    assert_eq!(t.g.permanents().filter(|o| o.is_token()).count(), 1);
}
