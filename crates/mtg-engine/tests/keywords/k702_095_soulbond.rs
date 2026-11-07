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

fn can_attack(t: &mut TestGame, id: ObjectId) -> bool {
    t.g.recompute();
    t.g.can_attack(id)
}

#[test]
fn an_ability_can_require_being_paired_with_a_soulbond_creature() {
    cr!("702.95b");
    ruling!(
        "Flowering Lumberknot",
        "If the creature Flowering Lumberknot is paired with loses soulbond, Flowering Lumberknot will remain paired but won't be able to attack or block."
    );
    assert_supported("Flowering Lumberknot");
    let mut t = TestGame::new(2);
    let knot = t.battlefield(P0, "Flowering Lumberknot");
    assert!(!can_attack(&mut t, knot));
    let wolfir = t.enter(P0, "Wolfir Silverheart");
    t.settle();
    t.answer_choose(P0, &[Entity::Object(knot)]);
    t.resolve();
    assert!(paired(&t, wolfir, knot));
    assert!(can_attack(&mut t, knot));
    // Wolfir Silverheart loses all abilities (soulbond included): still paired, but the
    // Lumberknot can't attack.
    run_effect(
        &mut t,
        None,
        P1,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::RemoveAllAbilities],
            duration: Duration::EndOfTurn,
        },
        &[Entity::Object(wolfir)],
    );
    assert!(paired(&t, wolfir, knot));
    assert!(!can_attack(&mut t, knot));
}

#[test]
fn becoming_unpaired_after_attacking_doesnt_remove_it_from_combat() {
    cr!("702.95b", "506.4a");
    ruling!(
        "Flowering Lumberknot",
        "Whether Flowering Lumberknot is paired is checked only when attackers or blockers are declared. After that point, Flowering Lumberknot becoming unpaired won't cause it to stop attacking or blocking."
    );
    let mut t = TestGame::new(2);
    let knot = t.battlefield(P0, "Flowering Lumberknot");
    let wolfir = t.enter(P0, "Wolfir Silverheart");
    t.settle();
    t.answer_choose(P0, &[Entity::Object(knot)]);
    t.resolve();
    crate::common_k702_011_017::attack_with(&mut t, &[(knot, Entity::Player(P1))]);
    assert!(t.g.is_attacking(knot));
    destroy(&mut t, wolfir);
    t.settle();
    assert!(unpaired(&t, knot));
    assert!(t.g.is_attacking(knot));
}

#[test]
fn an_effect_can_refer_to_the_creature_a_target_is_paired_with() {
    cr!("702.95b");
    ruling!(
        "Joint Assault",
        "Joint Assault checks whether the target creature is paired only when it resolves. If it becomes unpaired later in the turn, neither creature will lose the bonus."
    );
    assert_supported("Joint Assault");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let elves = t.battlefield(P0, "Llanowar Elves");
    let forcemage = t.enter(P0, "Trusted Forcemage");
    t.settle();
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.resolve();
    // Trusted Forcemage: "each of those creatures gets +1/+1".
    assert_eq!(t.pt(bears), (3, 3));
    t.lands(P0, "Forest", 1);
    let assault = t.hand(P0, "Joint Assault");
    t.cast(P0, assault).target(bears).go();
    t.resolve();
    assert_eq!(t.pt(bears), (5, 5));
    assert_eq!(t.pt(forcemage), (5, 5));
    assert_eq!(t.pt(elves), (1, 1));
    // Unpaired later: the +2/+2 bonuses stay.
    destroy(&mut t, bears);
    t.settle();
    assert_eq!(t.pt(forcemage), (4, 4));
    // An unpaired target gets the bonus alone.
    t.lands(P0, "Forest", 1);
    let assault = t.hand(P0, "Joint Assault");
    t.cast(P0, assault).target(elves).go();
    t.resolve();
    assert_eq!(t.pt(elves), (3, 3));
    assert_eq!(t.pt(forcemage), (4, 4));
}

#[test]
fn an_ability_can_trigger_on_the_creature_it_is_paired_with() {
    cr!("702.95b");
    ruling!(
        "Donna Noble",
        "If Donna is paired with another creature and they are both dealt damage at the same time, the second ability triggers twice."
    );
    // (Its "Doctor's companion" deck-building ability is another matter.)
    assert!(mtg_engine::card::card("Donna Noble")
        .front()
        .chars
        .abilities
        .iter()
        .any(|a| a.text.contains("paired with is dealt damage")
            && matches!(a.kind, AbilityKind::Triggered(_))));
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let donna = t.enter(P0, "Donna Noble");
    t.settle();
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.resolve();
    assert!(paired(&t, donna, bears));
    // 1 damage to each of them at the same time.
    let mut ctx = mtg_engine::eval::Ctx::new(None, P1);
    ctx.targets = vec![vec![Entity::Object(bears), Entity::Object(donna)]];
    let pinger = t.battlefield(P1, "Llanowar Elves");
    ctx.source = Some(pinger);
    t.g.exec(
        &Effect::DealDamage {
            source: Sel::This,
            amount: Value::c(1),
            to: Sel::Target(0),
        },
        &mut ctx,
    );
    t.g.flush_events();
    t.settle();
    let donna_triggers = t
        .g
        .stack
        .iter()
        .filter(|s| {
            matches!(&t.g.obj(**s).stack.as_ref().unwrap().kind,
                mtg_engine::object::StackKind::Triggered { source, .. } if *source == donna)
        })
        .count();
    assert_eq!(donna_triggers, 2);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    // An unpaired creature's damage doesn't trigger it.
    let elves = t.battlefield(P0, "Llanowar Elves");
    let mut ctx = mtg_engine::eval::Ctx::new(Some(pinger), P1);
    ctx.targets = vec![vec![Entity::Object(elves)]];
    t.g.exec(
        &Effect::DealDamage {
            source: Sel::This,
            amount: Value::c(1),
            to: Sel::Target(0),
        },
        &mut ctx,
    );
    t.g.flush_events();
    t.settle();
    assert!(t.g.stack.is_empty());
}
