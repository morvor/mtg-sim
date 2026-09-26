//! CR 702.95 Soulbond.

use crate::common_k702_011_017::assert_supported;
use crate::common_k702_027_037::activate_named;
use crate::common_k702_052_066::{destroy, run_effect, stack_triggers};
use mtg_engine::ability::*;
use mtg_engine::kw::soulbond::partner;
use mtg_engine::testing::*;
use mtg_engine::types::CardType;
use mtg_engine::*;

fn paired(t: &TestGame, a: ObjectId, b: ObjectId) -> bool {
    let (a, b) = (t.g.current(a), t.g.current(b));
    partner(&t.g, a) == Some(b) && partner(&t.g, b) == Some(a)
}

fn unpaired(t: &TestGame, a: ObjectId) -> bool {
    partner(&t.g, t.g.current(a)).is_none()
}

/// Wolfir Silverheart enters under `p`'s control and is paired with `with` (its enters
/// trigger resolved).
fn silverheart_paired_with(t: &mut TestGame, p: PlayerId, with: ObjectId) -> ObjectId {
    let wolfir = t.enter(p, "Wolfir Silverheart");
    t.settle();
    t.answer_choose(p, &[Entity::Object(with)]);
    t.resolve();
    assert!(paired(t, wolfir, with));
    wolfir
}

#[test]
fn a_soulbond_creature_entering_may_pair_with_an_unpaired_creature() {
    cr!("702.95", "702.95a", "702.95b");
    assert_supported("Wolfir Silverheart");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let elves = t.battlefield(P0, "Llanowar Elves");
    let wolfir = t.enter(P0, "Wolfir Silverheart");
    t.settle();
    assert_eq!(stack_triggers(&t, "Soulbond").len(), 1);
    // The creature to pair with is chosen as the ability resolves.
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.resolve();
    assert!(paired(&t, wolfir, bears));
    assert!(unpaired(&t, elves));
    // "Each of those creatures gets +4/+4."
    assert_eq!(t.pt(wolfir), (8, 8));
    assert_eq!(t.pt(bears), (6, 6));
    assert_eq!(t.pt(elves), (1, 1));
}

#[test]
fn pairing_is_optional() {
    cr!("702.95a");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let wolfir = t.enter(P0, "Wolfir Silverheart");
    t.answer_choose(P0, &[]);
    t.resolve_all();
    assert!(unpaired(&t, wolfir));
    assert!(unpaired(&t, bears));
    assert_eq!(t.pt(wolfir), (4, 4));
}

#[test]
fn soulbond_doesnt_trigger_without_another_unpaired_creature() {
    cr!("702.95a");
    let mut t = TestGame::new(2);
    // Only an opponent's creature.
    t.battlefield(P1, "Grizzly Bears");
    t.enter(P0, "Wolfir Silverheart");
    t.settle();
    assert!(stack_triggers(&t, "Soulbond").is_empty());
}

#[test]
fn another_creature_entering_may_pair_with_an_unpaired_soulbond_creature() {
    cr!("702.95a");
    let mut t = TestGame::new(2);
    let wolfir = t.battlefield(P0, "Wolfir Silverheart");
    // An opponent's creature entering doesn't trigger it.
    t.enter(P1, "Hill Giant");
    t.settle();
    assert!(stack_triggers(&t, "Soulbond").is_empty());
    let bears = t.enter(P0, "Grizzly Bears");
    t.settle();
    assert_eq!(stack_triggers(&t, "Soulbond").len(), 1);
    t.answer_yes(P0, true);
    t.resolve();
    assert!(paired(&t, wolfir, bears));
    assert_eq!(t.pt(bears), (6, 6));
}

#[test]
fn no_pair_if_a_creature_left_before_the_ability_resolves() {
    cr!("702.95c");
    let mut t = TestGame::new(2);
    let wolfir = t.battlefield(P0, "Wolfir Silverheart");
    let bears = t.enter(P0, "Grizzly Bears");
    t.settle();
    assert_eq!(stack_triggers(&t, "Soulbond").len(), 1);
    destroy(&mut t, bears);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(unpaired(&t, wolfir));
    assert_eq!(t.pt(wolfir), (4, 4));
}

#[test]
fn no_pair_if_a_creature_stopped_being_a_creature_or_changed_control() {
    cr!("702.95c");
    // It stopped being a creature.
    let mut t = TestGame::new(2);
    let wolfir = t.battlefield(P0, "Wolfir Silverheart");
    let bears = t.enter(P0, "Grizzly Bears");
    t.settle();
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
            duration: Duration::EndOfTurn,
        },
        &[Entity::Object(bears)],
    );
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(unpaired(&t, wolfir));
    assert!(unpaired(&t, bears));
    // Another player gained control of it.
    let mut t = TestGame::new(2);
    let wolfir = t.battlefield(P0, "Wolfir Silverheart");
    let bears = t.enter(P0, "Grizzly Bears");
    t.settle();
    run_effect(
        &mut t,
        None,
        P1,
        Effect::GainControl {
            what: Sel::Target(0),
            who: PlayerRef::You,
            duration: Duration::EndOfTurn,
        },
        &[Entity::Object(bears)],
    );
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(unpaired(&t, wolfir));
    assert!(unpaired(&t, bears));
}

#[test]
fn a_creature_can_be_paired_with_only_one_other_creature() {
    cr!("702.95d");
    assert_supported("Druid's Familiar");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let wolfir = silverheart_paired_with(&mut t, P0, bears);
    // A paired soulbond creature doesn't trigger for new creatures.
    t.enter(P0, "Llanowar Elves");
    t.settle();
    assert!(stack_triggers(&t, "Soulbond").is_empty());
    let elves = t.named_on_battlefield("Llanowar Elves")[0];
    // Another soulbond creature can pair only with an unpaired creature.
    let familiar = t.enter(P0, "Druid's Familiar");
    t.settle();
    // Offering Wolfir Silverheart isn't allowed: the answer is invalid and nothing is
    // chosen.
    t.answer_choose(P0, &[Entity::Object(wolfir)]);
    t.resolve();
    assert!(paired(&t, wolfir, bears));
    assert!(unpaired(&t, familiar));
    let offered = t
        .asked()
        .into_iter()
        .rev()
        .find_map(|(_, d)| match d {
            mtg_engine::decision::Decision::ChooseEntities { candidates, .. } => {
                Some(candidates)
            }
            _ => None,
        })
        .unwrap();
    assert_eq!(offered, vec![Entity::Object(elves)]);
}

#[test]
fn a_pair_breaks_when_another_player_gains_control_of_either_creature() {
    cr!("702.95e");
    ruling!(
        "Druid's Familiar",
        "If Druid’s Familiar becomes unpaired, it will immediately lose the +2/+2 bonus. If this causes it to have damage marked on it equal to it or greater than its new toughness, it will be destroyed. The same is true for the creature it was paired with."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let familiar = t.enter(P0, "Druid's Familiar");
    t.settle();
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.resolve();
    assert!(paired(&t, familiar, bears));
    assert_eq!(t.pt(familiar), (4, 4));
    t.g.objects[familiar.0 as usize].damage = 3;
    // The opponent gains control of the Bears.
    run_effect(
        &mut t,
        None,
        P1,
        Effect::GainControl {
            what: Sel::Target(0),
            who: PlayerRef::You,
            duration: Duration::EndOfTurn,
        },
        &[Entity::Object(bears)],
    );
    assert!(unpaired(&t, familiar));
    assert!(unpaired(&t, bears));
    assert_eq!(t.pt(bears), (2, 2));
    t.settle();
    // 3 damage on a 2/2: destroyed.
    assert!(t.in_graveyard(P0, "Druid's Familiar"));
    // Gaining control of both creatures at once breaks the pair as well.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let wolfir = silverheart_paired_with(&mut t, P0, bears);
    run_effect(
        &mut t,
        None,
        P1,
        Effect::GainControl {
            what: Sel::AllTargets,
            who: PlayerRef::You,
            duration: Duration::Permanent,
        },
        &[Entity::Object(bears), Entity::Object(wolfir)],
    );
    assert_eq!(t.obj_now(wolfir).controller, P1);
    assert_eq!(t.obj_now(bears).controller, P1);
    assert!(unpaired(&t, wolfir));
    assert!(unpaired(&t, bears));
}

#[test]
fn a_pair_breaks_when_either_stops_being_a_creature_or_leaves() {
    cr!("702.95e");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let wolfir = silverheart_paired_with(&mut t, P0, bears);
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
            duration: Duration::EndOfTurn,
        },
        &[Entity::Object(bears)],
    );
    assert!(unpaired(&t, wolfir));
    assert_eq!(t.pt(wolfir), (4, 4));
    // It stays unpaired even once the Bears are a creature again at end of turn.
    t.advance_to(P1, mtg_engine::turn::Step::Upkeep);
    assert!(t.obj_now(bears).is(CardType::Creature));
    assert!(unpaired(&t, wolfir));
    // Leaving the battlefield breaks a pair too.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let wolfir = silverheart_paired_with(&mut t, P0, bears);
    destroy(&mut t, bears);
    t.settle();
    assert!(unpaired(&t, wolfir));
    assert_eq!(t.pt(wolfir), (4, 4));
}

#[test]
fn a_flickered_paired_creature_can_be_paired_again() {
    cr!("702.95a", "702.95e");
    ruling!(
        "Deadeye Navigator",
        "If you activate the ability granted by Deadeye Navigator, the creature will be exiled, the pair will immediately be broken, and then the card will be returned to the battlefield. Deadeye Navigator’s soulbond ability triggers when that card enters the battlefield and the pair can then be reunited."
    );
    assert_supported("Deadeye Navigator");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let navigator = t.enter(P0, "Deadeye Navigator");
    t.settle();
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.resolve();
    assert!(paired(&t, navigator, bears));
    // The Bears have "{1}{U}: Exile this creature, then return it to the battlefield
    // under your control."
    t.lands(P0, "Island", 2);
    let granted = t
        .obj_now(bears)
        .chars
        .abilities
        .iter()
        .find(|a| matches!(a.kind, AbilityKind::Activated(_)))
        .map(|a| a.text.clone())
        .expect("granted ability");
    activate_named(&mut t, P0, bears, &granted, 0).expect("flicker");
    t.resolve();
    let back = t.g.current(bears);
    assert_ne!(back, bears);
    assert!(t.on_battlefield(back));
    assert!(unpaired(&t, navigator));
    // Deadeye Navigator's soulbond ability triggers for the returned Bears.
    t.settle();
    assert_eq!(stack_triggers(&t, "Soulbond").len(), 1);
    t.answer_yes(P0, true);
    t.resolve();
    assert!(paired(&t, navigator, back));
}
