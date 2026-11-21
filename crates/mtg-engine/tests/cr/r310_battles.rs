//! CR 310: battles — casting and resolving, battle types, defense, attacking battles,
//! protectors, and Sieges.

use crate::r300_common::*;
use mtg_engine::ability::*;
use mtg_engine::battle;
use mtg_engine::decision::{Action, Answer, Decision};
use mtg_engine::game::GameConfig;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

const SEGOVIA: &str = "Invasion of Segovia";

/// A custom battle with the given defense (a Siege, or with no battle type).
fn custom_battle(name: &str, siege: bool, defense: i32, text: &str) -> CardDef {
    let line = if siege { "Battle — Siege" } else { "Battle" };
    let mut d = oracle_card(name, line, "{0}", None, text);
    d.faces[0].chars.defense = Some(defense);
    d
}

/// `controller`'s Invasion of Segovia entering the battlefield with `protector` chosen.
fn enter_siege(t: &mut TestGame, controller: PlayerId, protector: PlayerId) -> ObjectId {
    t.answer_choose(controller, &[Entity::Player(protector)]);
    let id = t.enter(controller, SEGOVIA);
    assert_eq!(battle::protector(&t.g, id), Some(protector));
    id
}

fn three_players() -> TestGame {
    TestGame::with_config(
        3,
        GameConfig {
            attack_multiple_players: true,
            ..Default::default()
        },
    )
}

/// The protector choices `p` was offered most recently.
fn protector_candidates(t: &TestGame, p: PlayerId) -> Vec<Entity> {
    t.asked()
        .iter()
        .rev()
        .find_map(|(q, d)| match d {
            Decision::ChooseEntities {
                candidates, prompt, ..
            } if *q == p && prompt.contains("protector") => Some(candidates.clone()),
            _ => None,
        })
        .unwrap_or_default()
}

#[test]
fn battle_spells_are_cast_at_sorcery_speed_and_use_the_stack() {
    cr!("310.1");
    check_sorcery_timing(SEGOVIA, "{2}{U}");
}

#[test]
fn a_battle_spell_resolves_onto_the_battlefield_under_its_controllers_control() {
    cr!("310.2");
    let mut t = TestGame::new(2);
    let b = cast_others_card(&mut t, P0, P1, SEGOVIA, "{2}{U}", &[]);
    assert!(t.on_battlefield(b));
    assert_eq!((t.obj(b).controller, t.obj(b).owner), (P0, P1));
    // Its controller's opponent protects it.
    assert_eq!(battle::protector(&t.g, b), Some(P1));
}

#[test]
fn battle_subtypes_are_battle_types() {
    cr!("310.3");
    assert_eq!(subtypes_of(SEGOVIA), vec!["Siege"]);
    assert_eq!(subtype_kind("Siege"), Some(SubtypeKind::Battle));
}

#[test]
fn defense_is_printed_off_the_battlefield_and_counters_on_it() {
    cr!("310.4", "310.4a", "310.4b", "310.4c");
    ruling!("Invasion of Segovia", "A battle enters the battlefield with that number of defense counters");
    let mut t = TestGame::new(2);
    let card = t.hand(P0, SEGOVIA);
    assert_eq!(t.obj(card).chars.defense, Some(4));
    // It enters with defense counters equal to its printed defense.
    let b = enter_siege(&mut t, P0, P1);
    assert_eq!(t.counters(b, counters::DEFENSE), 4);
    assert_eq!(t.obj(b).defense(), 4);
    // Its defense is the number of defense counters on it.
    t.g.add_counters(Entity::Object(b), counters::DEFENSE, 2, None);
    assert_eq!(t.obj(b).defense(), 6);
    // Those counters are put on it by a replacement effect as it enters: other effects
    // modifying the counters put on it apply.
    let mut t = TestGame::new(2);
    t.custom(
        P0,
        CB::new("Reinforcements")
            .enchantment()
            .ability(stat(StaticEffect::Replacement(ReplacementDef {
                event: ReplacementEvent::PutCounters {
                    on_objects: Some(Filter::Type(CardType::Battle)),
                    on_players: None,
                    kind: Some(counters::DEFENSE.into()),
                },
                action: ReplacementAction::Add(Value::c(1)),
                self_replacement: false,
                optional: false,
            })))
            .build(),
        Zone::Battlefield,
    );
    let b = enter_siege(&mut t, P0, P1);
    assert_eq!(t.obj(b).defense(), 5);
}

#[test]
fn battles_can_be_attacked_and_damage_removes_defense_counters() {
    cr!("310.5", "310.6");
    ruling!("Invasion of Segovia", "Damage dealt to a battle causes that many defense counters to be removed from it");
    let mut t = TestGame::new(2);
    let b = enter_siege(&mut t, P0, P1);
    t.resolve_all();
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(bears, Entity::Object(b))], &[]);
    assert_eq!(t.obj(b).defense(), 2);
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.obj(b).damage, 0, "damage isn't marked on it");
    // Noncombat damage too.
    t.set_step(P0, Step::PostcombatMain);
    deal_damage(&mut t, P0, Entity::Object(b), 1);
    assert_eq!(t.obj(b).defense(), 1);
}

#[test]
fn a_siege_with_no_defense_waits_for_its_triggered_abilities() {
    cr!("310.7");
    ruling!("Invasion of Segovia", "the source of a triggered ability that has triggered but not yet left the stack, that battle is put into its owner");
    let mut t = TestGame::new(2);
    let b = enter_siege(&mut t, P0, P1);
    t.resolve_all();
    deal_damage(&mut t, P1, Entity::Object(b), 4);
    t.settle();
    // Its "last defense counter" ability triggered: the Siege stays for now.
    assert_eq!(t.obj(b).defense(), 0);
    let waiting = battle::defeat_triggers_on_stack(&t.g);
    assert_eq!(waiting.len(), 1);
    assert!(t.on_battlefield(b));
    // Once it has left the stack (here: countered), the Siege is put into its owner's
    // graveyard.
    t.g.counter(waiting[0], None);
    t.settle();
    assert!(t.in_graveyard(P0, SEGOVIA));
}

#[test]
fn a_non_siege_battle_with_no_defense_is_put_into_the_graveyard() {
    cr!("310.8");
    let mut t = TestGame::new(2);
    let b = t.custom(
        P0,
        custom_battle(
            "Plain Battle",
            false,
            2,
            "Whenever ~ is dealt damage, you gain 1 life.",
        ),
        Zone::Battlefield,
    );
    t.settle();
    deal_damage(&mut t, P1, Entity::Object(b), 2);
    t.settle();
    // It doesn't wait for its triggered ability.
    assert!(t.in_graveyard(P0, "Plain Battle"));
    assert_eq!(t.stack_len(), 1);
}

#[test]
fn a_battle_with_no_battle_type_is_protected_by_its_controller() {
    cr!("310.9", "310.9a");
    let mut t = three_players();
    let def = custom_battle("Home Front", false, 3, "");
    let id = t.g.create_card_object(std::sync::Arc::new(def), P0, Zone::Nowhere);
    t.answer_choose(P0, &[Entity::Player(P1)]);
    let b = t
        .g
        .move_object_ev(mtg_engine::replacement::MoveEv {
            obj: id,
            to: Zone::Battlefield,
            pos: LibraryPosition::Top,
            cause: mtg_engine::events::MoveCause::Effect,
            by: Some(P0),
            etb: mtg_engine::replacement::EtbInfo {
                controller: Some(P0),
                ..Default::default()
            },
            source: None,
        })
        .unwrap();
    // As it entered, its controller chose its protector: only they could be chosen.
    assert_eq!(protector_candidates(&t, P0), vec![Entity::Player(P0)]);
    assert_eq!(battle::protector(&t.g, b), Some(P0));
}

#[test]
fn a_siege_is_protected_by_an_opponent_of_its_controller() {
    cr!("310.12", "310.12a", "310.9a");
    ruling!("Invasion of Segovia", "As a Siege enters the battlefield, its controller chooses an opponent to be its protector");
    let mut t = three_players();
    let b = enter_siege(&mut t, P0, P2);
    // The choice was among P0's opponents only.
    let offered = protector_candidates(&t, P0);
    assert_eq!(offered.len(), 2);
    assert!(offered.contains(&Entity::Player(P1)) && offered.contains(&Entity::Player(P2)));
    assert_eq!(battle::protector(&t.g, b), Some(P2));
}

fn can_attack_battle(t: &mut TestGame, active: PlayerId, b: ObjectId) -> bool {
    t.g.combat = None;
    t.set_step(active, Step::BeginningOfCombat);
    mtg_engine::combat::attack_targets(&t.g).contains(&Entity::Object(b))
}

#[test]
fn a_battles_protector_cant_attack_it_but_other_players_can() {
    cr!("310.9b");
    ruling!("Invasion of Segovia", "A battle can be attacked by all players other than its protector");
    let mut t = three_players();
    let b = enter_siege(&mut t, P0, P1);
    // Its own controller can attack a Siege: its protector is a defending player for them.
    assert!(can_attack_battle(&mut t, P0, b));
    // Another opponent of the protector can attack it.
    assert!(can_attack_battle(&mut t, P2, b));
    // The protector can't.
    assert!(!can_attack_battle(&mut t, P1, b));
}

#[test]
fn only_the_protector_may_block_creatures_attacking_a_battle() {
    cr!("310.9c");
    ruling!("Invasion of Segovia", "Only creatures controlled by a battle");
    let mut t = three_players();
    let b = enter_siege(&mut t, P0, P1);
    t.resolve_all();
    let attacker = t.battlefield(P0, "Grizzly Bears");
    let p1_blocker = t.battlefield(P1, "Hill Giant");
    let p2_blocker = t.battlefield(P2, "Hill Giant");
    t.set_step(P0, Step::BeginningOfCombat);
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(attacker, Entity::Object(b))]),
    );
    t.advance_to(P0, Step::DeclareAttackers);
    assert!(t.g.can_block(p1_blocker, attacker));
    assert!(!t.g.can_block(p2_blocker, attacker));
}

#[test]
fn the_defending_player_of_an_attack_on_a_battle_is_its_protector() {
    cr!("310.9d");
    let mut t = three_players();
    // P0 controls the Siege; P1 protects it; P2 attacks it.
    let b = enter_siege(&mut t, P0, P1);
    t.resolve_all();
    let raider = t.custom(
        P2,
        oracle_card(
            "Needling Raider",
            "Creature — Human",
            "{1}",
            Some((1, 1)),
            "Whenever ~ attacks, defending player loses 1 life.",
        ),
        Zone::Battlefield,
    );
    t.set_step(P2, Step::BeginningOfCombat);
    t.attack(&[(raider, Entity::Object(b))], &[]);
    assert_eq!(t.life(P1), 19);
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.obj(b).defense(), 3);
}

#[test]
fn effects_referring_to_who_protects_a_battle_mean_its_protector() {
    cr!("310.9e");
    supported("Portent Tracker");
    let mut t = TestGame::new(2);
    // P0 controls this Siege and P1 protects it; P1 controls the other, P0 protects it.
    let mine = enter_siege(&mut t, P0, P1);
    let theirs = enter_siege(&mut t, P1, P0);
    t.resolve_all();
    let tracker = t.battlefield(P0, "Portent Tracker");
    // "{T}: Choose target battle. If an opponent protects it, remove a defense counter
    // from it. Otherwise, put a defense counter on it."
    t.activate(P0, tracker, 1, &[Entity::Object(mine)]).unwrap();
    t.resolve();
    assert_eq!(t.obj(mine).defense(), 3);
    t.g.objects[tracker.0 as usize].tapped = false;
    t.activate(P0, tracker, 1, &[Entity::Object(theirs)]).unwrap();
    t.resolve();
    assert_eq!(t.obj(theirs).defense(), 5);
}

#[test]
fn a_battle_has_one_protector_at_a_time() {
    cr!("310.9f", "310.11");
    ruling!("Invasion of Segovia", "protector ever gains control of it, they choose a new player to be its protector");
    let mut t = three_players();
    let b = enter_siege(&mut t, P0, P1);
    t.resolve_all();
    // P1 gains control of the Siege: P1 can't protect a Siege they control, so they
    // choose a new protector among their opponents.
    t.answer_choose(P1, &[Entity::Player(P2)]);
    run_effect(
        &mut t,
        P1,
        None,
        Effect::GainControl {
            what: Sel::Target(0),
            who: PlayerRef::You,
            duration: Duration::Permanent,
        },
        &[Entity::Object(b)],
    );
    t.settle();
    assert_eq!(t.obj(b).controller, P1);
    assert_eq!(protector_candidates(&t, P1).len(), 2);
    assert_eq!(battle::protector(&t.g, b), Some(P2));
    // P1 is no longer its protector: they may attack it now, and P2 may not.
    assert!(can_attack_battle(&mut t, P1, b));
    assert!(!can_attack_battle(&mut t, P2, b));
}

#[test]
fn a_battles_protector_doesnt_change_when_it_stops_being_a_battle_or_copies_one() {
    cr!("310.9g");
    let mut t = three_players();
    let b = enter_siege(&mut t, P0, P1);
    t.resolve_all();
    run_effect(
        &mut t,
        P0,
        None,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::SetTypes {
                types: vec![CardType::Artifact],
                subtypes: vec![],
            }],
            duration: Duration::EndOfTurn,
        },
        &[Entity::Object(b)],
    );
    t.settle();
    assert!(!t.obj(b).is(CardType::Battle));
    assert_eq!(battle::protector(&t.g, b), Some(P1));
    // A battle that becomes a copy of another battle keeps its protector: P0's Siege
    // (protected by P1) becomes a copy of P2's (protected by P0).
    let mut t = three_players();
    let b = t.custom(
        P0,
        custom_battle("Copying Siege", true, 3, ""),
        Zone::Battlefield,
    );
    t.g.objects[b.0 as usize].choices.player = Some(P1);
    let other = enter_siege(&mut t, P2, P0);
    t.resolve_all();
    run_effect(
        &mut t,
        P0,
        Some(b),
        Effect::BecomeCopy {
            what: Sel::This,
            of: Sel::Target(0),
            duration: Duration::Permanent,
        },
        &[Entity::Object(other)],
    );
    t.settle();
    assert_eq!(t.obj(b).chars.name.as_str(), SEGOVIA);
    assert_eq!(battle::protector(&t.g, b), Some(P1));
    assert_eq!(battle::protector(&t.g, other), Some(P0));
}

#[test]
fn a_battle_cant_be_attached_to_anything() {
    cr!("310.10");
    let mut t = TestGame::new(2);
    // An artifact battle that's also an Equipment.
    let mut def = custom_battle("Siege Engine", false, 3, "");
    def.faces[0].chars.card_types.insert(CardType::Artifact);
    def.faces[0].chars.subtypes.push("Equipment".into());
    let engine = t.custom(P0, def, Zone::Battlefield);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.settle();
    run_effect(
        &mut t,
        P0,
        Some(engine),
        Effect::Attach {
            what: Sel::This,
            to: Sel::Target(0),
        },
        &[Entity::Object(bears)],
    );
    assert_eq!(t.obj(engine).attached_to, None);
    // If it's somehow attached, it becomes unattached.
    t.g.objects[engine.0 as usize].attached_to = Some(Entity::Object(bears));
    t.settle();
    assert_eq!(t.obj(engine).attached_to, None);
    assert!(t.on_battlefield(engine));
}

#[test]
fn a_battle_without_a_protector_gets_one_once_it_isnt_being_attacked() {
    cr!("310.11");
    ruling!("Invasion of Segovia", "if the protector of a battle leaves the game and that battle is not currently being attacked");
    let mut t = three_players();
    let b = enter_siege(&mut t, P0, P1);
    t.resolve_all();
    let raider = t.battlefield(P2, "Grizzly Bears");
    t.set_step(P2, Step::BeginningOfCombat);
    t.answer(
        P2,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(raider, Entity::Object(b))]),
    );
    t.advance_to(P2, Step::DeclareAttackers);
    // Its protector leaves the game while a creature is attacking the battle: no new
    // protector is chosen while it's being attacked.
    t.g.perform_action(P1, Action::Concede).unwrap();
    t.settle();
    assert!(!t.player(P1).in_game());
    assert!(t.g.is_attacking(raider));
    assert_eq!(battle::protector(&t.g, b), Some(P1));
    // After combat, its controller chooses a new protector among their opponents.
    t.answer_choose(P0, &[Entity::Player(P2)]);
    t.advance_to(P2, Step::PostcombatMain);
    t.settle();
    assert_eq!(battle::protector(&t.g, b), Some(P2));
    assert_eq!(protector_candidates(&t, P0), vec![Entity::Player(P2)]);
}

#[test]
fn a_defeated_siege_is_exiled_and_may_be_cast_transformed_for_free() {
    cr!("310.12b");
    ruling!("Invasion of Segovia", "Sieges each have an intrinsic triggered ability");
    let mut t = TestGame::new(2);
    let b = enter_siege(&mut t, P0, P1);
    t.resolve_all();
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.add_counters(Entity::Object(bears), counters::PLUS1, 2, None);
    t.set_step(P0, Step::BeginningOfCombat);
    t.answer_yes(P0, true);
    t.attack(&[(bears, Entity::Object(b))], &[]);
    t.resolve_all();
    // The last defense counter was removed: it was exiled, then cast transformed without
    // paying its mana cost — Caetus, Sea Tyrant of Segovia.
    let caetus = t.named_on_battlefield("Caetus, Sea Tyrant of Segovia");
    assert_eq!(caetus.len(), 1);
    let o = t.obj(caetus[0]);
    assert!(o.is_creature());
    assert_eq!(o.controller, P0);
    assert!(o.cast.as_ref().is_some_and(|c| c.was_cast));
    // Declining to cast it leaves it in exile.
    let mut t = TestGame::new(2);
    let b = enter_siege(&mut t, P0, P1);
    t.resolve_all();
    t.answer_yes(P0, false);
    deal_damage(&mut t, P1, Entity::Object(b), 4);
    t.resolve_all();
    assert!(t.in_exile(SEGOVIA));
    assert!(t.named_on_battlefield("Caetus, Sea Tyrant of Segovia").is_empty());
}
