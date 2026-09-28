//! Rulings batch S17 — transform (CR 310, 506.4, 712): battles and Sieges (March of the
//! Machine's Invasions): attacking and blocking, damage and defense counters, protectors,
//! the Siege's intrinsic ability, and permanents that become copies of Sieges.
//!
//! These rulings exist in two spellings (curly and straight apostrophes); both are cited.

use crate::r_s01_common::*;
use crate::r_s02_common::{can_attack, create_token};
use crate::r_s06_common::damage;
use crate::r_s10_common::attacking;
use crate::r_s12_common::attack_target;
use crate::r_s17_common::*;
use mtg_engine::ability::*;
use mtg_engine::battle;
use mtg_engine::decision::Decision;
use mtg_engine::game::GameConfig;
use mtg_engine::object::{FaceState, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Invasion of Theros ({2}{W} Siege, defense 4) // Ephara, Ever-Sheltering (4/4 God).
const THEROS: &str = "Invasion of Theros // Ephara, Ever-Sheltering";
/// Invasion of Segovia ({2}{U} Siege, defense 4) // Caetus, Sea Tyrant of Segovia.
const SEGOVIA: &str = "Invasion of Segovia // Caetus, Sea Tyrant of Segovia";

fn three_players() -> TestGame {
    TestGame::with_config(
        3,
        GameConfig {
            attack_multiple_players: true,
            ..Default::default()
        },
    )
}

/// `controller`'s Siege `name` entering the battlefield with `protector` chosen, its enters
/// ability resolved.
fn enter_siege(t: &mut TestGame, controller: PlayerId, protector: PlayerId, name: &str) -> ObjectId {
    t.answer_choose(controller, &[Entity::Player(protector)]);
    let id = t.enter(controller, name);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(battle::protector(&t.g, id), Some(protector));
    id
}

/// Applies `mods` to the permanent until end of turn, then settles.
fn modify(t: &mut TestGame, id: ObjectId, mods: Vec<Modification>) {
    let mut ctx = mtg_engine::eval::Ctx::new(None, P0);
    ctx.targets = vec![vec![Entity::Object(t.g.current(id))]];
    t.g.exec(
        &Effect::Modify {
            what: Sel::Target(0),
            mods,
            duration: Duration::EndOfTurn,
        },
        &mut ctx,
    );
    t.g.recompute();
    t.g.flush_events();
    t.settle();
}

/// `p` gains control of the permanent, then settles.
fn gain_control(t: &mut TestGame, p: PlayerId, id: ObjectId) {
    let mut ctx = mtg_engine::eval::Ctx::new(None, p);
    ctx.targets = vec![vec![Entity::Object(t.g.current(id))]];
    t.g.exec(
        &Effect::GainControl {
            what: Sel::Target(0),
            who: PlayerRef::You,
            duration: Duration::Permanent,
        },
        &mut ctx,
    );
    t.g.recompute();
    t.g.flush_events();
    t.settle();
}

/// The permanent `id` becomes a copy of `of` and gets `defense` defense counters before
/// state-based actions are checked, then settles.
fn become_siege_with_defense(t: &mut TestGame, id: ObjectId, of: ObjectId, defense: u32) {
    let mut ctx = mtg_engine::eval::Ctx::new(Some(id), t.obj(id).controller);
    ctx.targets = vec![vec![Entity::Object(of)]];
    t.g.exec(
        &Effect::BecomeCopy {
            what: Sel::This,
            of: Sel::Target(0),
            duration: Duration::Permanent,
        },
        &mut ctx,
    );
    t.g.recompute();
    if defense > 0 {
        t.g.add_counters(Entity::Object(id), counters::DEFENSE, defense, None);
    }
    t.g.recompute();
    t.g.flush_events();
    t.settle();
}

#[test]
fn battles_cant_attack_or_block_and_a_creature_that_becomes_a_battle_leaves_combat() {
    cr!("506.3", "506.4");
    ruling!(
        "Invasion of Theros // Ephara, Ever-Sheltering",
        "Battles can’t attack or block, even if one also becomes a creature."
    );
    ruling!(
        "Invasion of Segovia // Caetus, Sea Tyrant of Segovia",
        "Battles can't attack or block, even if one also becomes a creature."
    );
    supported(THEROS);
    let mut t = TestGame::new(2);
    let siege = enter_siege(&mut t, P0, P1, THEROS);
    // P0's Siege becomes a 4/4 creature in addition to its other types.
    modify(
        &mut t,
        siege,
        vec![
            Modification::AddTypes(vec![CardType::Creature]),
            Modification::SetPT(Some(Value::c(4)), Some(Value::c(4))),
        ],
    );
    assert!(t.obj(siege).is_creature());
    t.g.objects[siege.0 as usize].summoning_sick = false;
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!can_attack(&mut t, siege));
    // P1's Segovia Siege creature can't block either.
    let theirs = enter_siege(&mut t, P1, P0, SEGOVIA);
    modify(
        &mut t,
        theirs,
        vec![
            Modification::AddTypes(vec![CardType::Creature]),
            Modification::SetPT(Some(Value::c(4)), Some(Value::c(4))),
        ],
    );
    let hill_giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    assert!(t.on_battlefield(theirs) && t.obj(theirs).is_creature());
    assert!(!t.g.can_block(theirs, bears));
    assert!(t.g.can_block(hill_giant, bears));
    // An attacking creature that becomes a battle is removed from combat.
    modify(
        &mut t,
        bears,
        vec![Modification::AddTypes(vec![CardType::Battle])],
    );
    assert!(!attacking(&t, bears));
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn a_battle_can_be_dealt_damage_and_targeted_by_any_target() {
    cr!("115.4", "310.6", "120.3h");
    ruling!(
        "Invasion of Theros // Ephara, Ever-Sheltering",
        "A battle can be dealt damage and be target of spells and/or abilities that target “any target.”"
    );
    ruling!(
        "Invasion of Segovia // Caetus, Sea Tyrant of Segovia",
        "A battle can be dealt damage and be target of spells and/or abilities that target \"any target.\""
    );
    let mut t = TestGame::new(2);
    let siege = enter_siege(&mut t, P1, P0, THEROS);
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    let from = t.asked().len();
    t.cast(P0, bolt).target(siege).go();
    let offered = crate::r_s02_common::target_candidates(&t, P0, from);
    assert!(offered.iter().any(|c| c.contains(&Entity::Object(siege))));
    t.resolve_all();
    assert_eq!(t.counters(siege, counters::DEFENSE), 1);
    assert_eq!(t.obj(siege).damage, 0);
}

#[test]
fn a_battle_being_attacked_leaves_combat_if_it_stops_being_a_battle_or_changes_control() {
    cr!("506.4", "506.4c");
    ruling!(
        "Invasion of Theros // Ephara, Ever-Sheltering",
        "If a battle that’s being attacked somehow stops being a battle, it is removed from combat."
    );
    ruling!(
        "Invasion of Segovia // Caetus, Sea Tyrant of Segovia",
        "If a battle that's being attacked somehow stops being a battle, it is removed from combat."
    );
    // It stops being a battle: the creature attacking it keeps attacking, but attacks
    // nothing and deals no combat damage.
    let mut t = TestGame::new(2);
    let siege = enter_siege(&mut t, P0, P1, THEROS);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(bears, Entity::Object(siege))]);
    assert_eq!(attack_target(&t, bears), Some(Entity::Object(siege)));
    modify(
        &mut t,
        siege,
        vec![Modification::SetTypes {
            types: vec![CardType::Artifact],
            subtypes: vec![],
        }],
    );
    assert!(attacking(&t, bears));
    assert_eq!(attack_target(&t, bears), None);
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.counters(siege, counters::DEFENSE), 4);
    assert_eq!(t.life(P1), 20);
    // Its controller changes: the same.
    let mut t = TestGame::new(2);
    let siege = enter_siege(&mut t, P0, P1, SEGOVIA);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(bears, Entity::Object(siege))]);
    t.answer_choose(P1, &[Entity::Player(P0)]);
    gain_control(&mut t, P1, siege);
    assert!(attacking(&t, bears));
    assert_eq!(attack_target(&t, bears), None);
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.counters(siege, counters::DEFENSE), 4);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn a_siege_that_never_had_defense_counters_goes_to_the_graveyard_without_triggering() {
    cr!("310.7", "704.5v", "310.12b");
    ruling!(
        "Invasion of Theros // Ephara, Ever-Sheltering",
        "If a Siege never had defense counters on it (perhaps because a permanent became a copy of one), it can’t have its last defense counter removed."
    );
    ruling!(
        "Invasion of Segovia // Caetus, Sea Tyrant of Segovia",
        "If a Siege never had defense counters on it (perhaps because a permanent became a copy of one), it can't have its last defense counter removed."
    );
    ruling!(
        "Invasion of Theros // Ephara, Ever-Sheltering",
        "If a non-battle permanent that is already on the battlefield become a copy of a Siege, its controller chooses one of their opponents to be that battle’s protector."
    );
    ruling!(
        "Invasion of Segovia // Caetus, Sea Tyrant of Segovia",
        "If a non-battle permanent that is already on the battlefield become a copy of a Siege, its controller chooses one of their opponents to be that battle's protector."
    );
    let mut t = three_players();
    let siege = enter_siege(&mut t, P1, P2, THEROS);
    // P0's Grizzly Bears (a creature card, a transforming double-faced card or not) becomes
    // a copy of it: a Siege with no defense counters.
    let bears = t.battlefield(P0, "Grizzly Bears");
    let from = t.asked().len();
    become_copy(&mut t, bears, siege);
    // P0 chose a protector among their opponents...
    let offered: Vec<Vec<Entity>> = t.asked()[from..]
        .iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseEntities {
                candidates, prompt, ..
            } if *p == P0 && prompt.contains("protector") => Some(candidates.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(offered.len(), 1);
    assert_eq!(
        offered[0].iter().collect::<std::collections::BTreeSet<_>>(),
        [Entity::Player(P1), Entity::Player(P2)].iter().collect()
    );
    // ... and it's put into its owner's graveyard: it's not exiled and nothing is cast.
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert!(t.g.stack.is_empty());
    assert!(!t.in_exile("Grizzly Bears"));
}

#[test]
fn a_permanent_entering_as_a_copy_of_a_battle_gets_its_defense_counters() {
    cr!("310.4b", "707.2");
    ruling!(
        "Invasion of Theros // Ephara, Ever-Sheltering",
        "If another permanent enters the battlefield as a copy of a battle, it also enters with that number of defense counters."
    );
    let mut t = TestGame::new(2);
    let siege = enter_siege(&mut t, P1, P0, THEROS);
    damage(&mut t, siege, 1, siege);
    assert_eq!(t.counters(siege, counters::DEFENSE), 3);
    // A token copy of it (created by P0) enters with 4 defense counters (the printed
    // defense, not the number on the original).
    let tok = token_copy(&mut t, P0, siege);
    assert_eq!(tok.len(), 1);
    assert_eq!(t.counters(tok[0], counters::DEFENSE), 4);
    assert!(t.obj(tok[0]).is(CardType::Battle));
    assert_eq!(battle::protector(&t.g, tok[0]), Some(P1));
}

#[test]
fn only_the_protectors_creatures_can_block_attackers_of_a_battle() {
    cr!("310.9c");
    ruling!(
        "Invasion of Theros // Ephara, Ever-Sheltering",
        "Only creatures controlled by a battle’s protector can block creatures that are attacking that battle. This means a Siege’s controller can never assign creatures to block for it."
    );
    let mut t = three_players();
    // P0 controls the Siege, P1 protects it, P2 attacks it.
    let siege = enter_siege(&mut t, P0, P1, THEROS);
    let raider = t.battlefield(P2, "Grizzly Bears");
    let p0_blocker = t.battlefield(P0, "Hill Giant");
    let p1_blocker = t.battlefield(P1, "Hill Giant");
    t.set_step(P2, Step::BeginningOfCombat);
    attack_with(&mut t, &[(raider, Entity::Object(siege))]);
    assert!(t.g.can_block(p1_blocker, raider));
    assert!(!t.g.can_block(p0_blocker, raider));
}

#[test]
fn a_defeated_siege_is_exiled_and_may_be_cast_transformed() {
    cr!("310.12b", "712.11a");
    ruling!(
        "Invasion of Theros // Ephara, Ever-Sheltering",
        "Sieges each have an intrinsic triggered ability. That ability is “When the last defense counter is removed from this permanent, exile it, then you may cast it transformed without paying its mana cost.”"
    );
    let mut t = TestGame::new(2);
    let siege = enter_siege(&mut t, P0, P1, THEROS);
    let giant = t.battlefield(P0, "Hill Giant");
    t.g.add_counters(Entity::Object(giant), counters::PLUS1, 1, None);
    t.answer_yes(P0, true);
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(giant, Entity::Object(siege))], &[]);
    t.resolve_all();
    let ephara = t.named_on_battlefield("Ephara, Ever-Sheltering");
    assert_eq!(ephara.len(), 1);
    assert_eq!(t.obj(ephara[0]).face, FaceState::Back);
    assert!(t.obj(ephara[0]).cast.as_ref().is_some_and(|c| c.was_cast));
    assert_eq!(untapped(&t, P0), 0);
    // Declining: it stays in exile.
    let mut t = TestGame::new(2);
    let siege = enter_siege(&mut t, P0, P1, THEROS);
    let source = t.battlefield(P1, "Grizzly Bears");
    t.answer_yes(P0, false);
    damage(&mut t, source, 4, siege);
    t.resolve_all();
    assert!(t.in_exile("Invasion of Theros"));
}

fn untapped(t: &TestGame, p: PlayerId) -> usize {
    crate::r_s04_common::untapped_lands(t, p)
}

#[test]
fn every_player_but_the_protector_may_attack_a_battle_its_controller_included() {
    cr!("310.9b");
    ruling!(
        "Invasion of Theros // Ephara, Ever-Sheltering",
        "A battle can be attacked by all players other than its protector. Notably, this means a Siege’s controller can attack it."
    );
    let mut t = three_players();
    let siege = enter_siege(&mut t, P0, P1, THEROS);
    let targets_for = |t: &mut TestGame, p: PlayerId| {
        t.g.combat = None;
        t.set_step(p, Step::BeginningOfCombat);
        mtg_engine::combat::attack_targets(&t.g)
    };
    assert!(targets_for(&mut t, P0).contains(&Entity::Object(siege)));
    assert!(targets_for(&mut t, P2).contains(&Entity::Object(siege)));
    assert!(!targets_for(&mut t, P1).contains(&Entity::Object(siege)));
}

#[test]
fn a_copy_of_a_siege_thats_not_a_transforming_double_faced_card_stays_in_exile() {
    cr!("310.12b", "111.8", "704.5d");
    ruling!(
        "Invasion of Theros // Ephara, Ever-Sheltering",
        "If a token or a card that isn’t represented by a transforming double-faced card becomes a copy of a Siege, it can’t be cast as its triggered ability resolves. It will remain in exile."
    );
    ruling!(
        "Invasion of Segovia // Caetus, Sea Tyrant of Segovia",
        "If a token or a card that isn't represented by a transforming double-faced card becomes a copy of a Siege, it can't be cast as its triggered ability resolves. It will remain in exile."
    );
    // A single-faced card (Grizzly Bears) and a meld card (Graf Rats) that became copies of
    // Invasion of Theros and got a defense counter: defeated, they're exiled and stay
    // there.
    for name in ["Grizzly Bears", "Graf Rats"] {
        let mut t = TestGame::new(2);
        let siege = enter_siege(&mut t, P1, P0, THEROS);
        let perm = t.battlefield(P0, name);
        t.answer_choose(P0, &[Entity::Player(P1)]);
        become_siege_with_defense(&mut t, perm, siege, 1);
        assert!(t.on_battlefield(perm), "{name}");
        assert_eq!(t.counters(perm, counters::DEFENSE), 1);
        let source = t.battlefield(P1, "Grizzly Bears");
        t.answer_yes(P0, true);
        damage(&mut t, source, 1, perm);
        t.resolve_all();
        assert!(t.in_exile(name), "{name}");
        assert_eq!(t.zone(perm), Zone::Exile, "{name}");
        assert!(t.named_on_battlefield("Ephara, Ever-Sheltering").is_empty());
        assert!(t.g.stack.is_empty());
    }
    // A Bear token: exiled, and it ceases to exist.
    let mut t = TestGame::new(2);
    let siege = enter_siege(&mut t, P1, P0, THEROS);
    let tok = create_token(&mut t, P0, "Bear");
    t.answer_choose(P0, &[Entity::Player(P1)]);
    become_siege_with_defense(&mut t, tok, siege, 1);
    assert!(t.on_battlefield(tok));
    let source = t.battlefield(P1, "Grizzly Bears");
    t.answer_yes(P0, true);
    damage(&mut t, source, 1, tok);
    t.resolve_all();
    assert!(!t.g.is_live(t.g.current(tok)) || t.zone(tok) != Zone::Battlefield);
    assert!(t.g.exile.iter().all(|o| !t.obj(*o).is_token()));
    assert!(t.g.stack.is_empty());
    // A token copy of the Siege is double-faced, but a token can't be cast: it's exiled
    // and ceases to exist.
    let mut t = TestGame::new(2);
    let siege = enter_siege(&mut t, P1, P0, THEROS);
    let tok = token_copy(&mut t, P0, siege)[0];
    t.resolve_all();
    let source = t.battlefield(P1, "Grizzly Bears");
    t.answer_yes(P0, true);
    let from = t.asked().len();
    damage(&mut t, source, 4, tok);
    assert_eq!(battle::defeat_triggers_on_stack(&t.g).len(), 1);
    t.resolve_all();
    assert!(!t.on_battlefield(tok));
    assert!(t.named_on_battlefield("Ephara, Ever-Sheltering").is_empty());
    assert!(t.g.exile.iter().all(|o| !t.obj(*o).is_token()));
    // Its controller wasn't even offered to cast it.
    assert!(!t.asked()[from..].iter().any(|(_, d)| matches!(d,
        Decision::YesNo { prompt, .. } if prompt.contains("Cast it transformed"))));
}

#[test]
fn a_transforming_double_faced_card_that_became_a_copy_of_a_siege_is_cast_transformed() {
    cr!("310.12b", "712.11a", "707.2");
    ruling!(
        "Invasion of Theros // Ephara, Ever-Sheltering",
        "If a permanent that is represented by a transforming double-faced card becomes a copy of a Siege, it will be exiled as that Siege’s triggered ability resolves, then it will be cast transformed."
    );
    ruling!(
        "Invasion of Segovia // Caetus, Sea Tyrant of Segovia",
        "If a permanent that is represented by a transforming double-faced card becomes a copy of a Siege, it will be exiled as that Siege's triggered ability resolves, then it will be cast transformed."
    );
    // P0's Ashling, Rekindled becomes a copy of Invasion of Theros, with a defense
    // counter. Defeated, it's exiled and cast transformed: as Ashling, Rimebound (its own
    // back face), not as Ephara.
    let mut t = TestGame::new(2);
    let siege = enter_siege(&mut t, P1, P0, THEROS);
    let ashling = t.battlefield(P0, "Ashling, Rekindled // Ashling, Rimebound");
    t.answer_choose(P0, &[Entity::Player(P1)]);
    become_siege_with_defense(&mut t, ashling, siege, 1);
    assert_eq!(name_of(&t, ashling), "Invasion of Theros");
    let source = t.battlefield(P1, "Grizzly Bears");
    t.answer_yes(P0, true);
    damage(&mut t, source, 1, ashling);
    t.resolve_all();
    let rimebound = t.named_on_battlefield("Ashling, Rimebound");
    assert_eq!(rimebound.len(), 1);
    assert_eq!(t.obj(rimebound[0]).face, FaceState::Back);
    assert!(t.obj(rimebound[0])
        .cast
        .as_ref()
        .is_some_and(|c| c.was_cast));
    assert!(t.named_on_battlefield("Ephara, Ever-Sheltering").is_empty());
}

#[test]
fn a_sieges_controller_cant_protect_it() {
    cr!("310.12a", "310.11", "704.5y");
    ruling!(
        "Invasion of Theros // Ephara, Ever-Sheltering",
        "A Siege’s controller can’t be its protector. If a Siege’s protector ever gains control of it, they choose a new player to be its protector. This is a state-based action."
    );
    let mut t = three_players();
    // As it enters, P0 chooses among their opponents only.
    let siege = enter_siege(&mut t, P0, P1, THEROS);
    assert_eq!(protector_choices(&t, 0, P0), vec![vec![P1, P2]]);
    // Each time its protector gains control of it, that player chooses a new protector
    // among their opponents: never themselves.
    for (new_controller, choice, offered) in [
        (P1, P0, vec![P0, P2]),
        (P0, P2, vec![P1, P2]),
        (P2, P1, vec![P0, P1]),
    ] {
        let from = t.asked().len();
        t.answer_choose(new_controller, &[Entity::Player(choice)]);
        gain_control(&mut t, new_controller, siege);
        assert_eq!(t.obj(siege).controller, new_controller);
        assert_eq!(protector_choices(&t, from, new_controller), vec![offered]);
        assert_eq!(battle::protector(&t.g, siege), Some(choice));
    }
    // P0 gains control while P1 protects it: P1 is still a legal protector, nothing to
    // choose.
    let from = t.asked().len();
    gain_control(&mut t, P0, siege);
    assert!(protector_choices(&t, from, P0).is_empty());
    assert_eq!(battle::protector(&t.g, siege), Some(P1));
}

/// The players `p` was offered to choose from as a battle's protector, in each such
/// decision asked since `from`.
fn protector_choices(t: &TestGame, from: usize, p: PlayerId) -> Vec<Vec<PlayerId>> {
    t.asked()[from..]
        .iter()
        .filter_map(|(q, d)| match d {
            Decision::ChooseEntities {
                candidates, prompt, ..
            } if *q == p && prompt.contains("protector") => {
                let mut ps: Vec<PlayerId> = candidates
                    .iter()
                    .filter_map(|e| match e {
                        Entity::Player(x) => Some(*x),
                        _ => None,
                    })
                    .collect();
                ps.sort();
                Some(ps)
            }
            _ => None,
        })
        .collect()
}

#[test]
fn a_battle_with_no_defense_counters_goes_to_the_graveyard_once_its_trigger_left_the_stack() {
    cr!("310.7", "704.5v");
    ruling!(
        "Invasion of Theros // Ephara, Ever-Sheltering",
        "If a battle has no defense counters, and it isn’t the source of a triggered ability that has triggered but not yet left the stack, that battle is put into its owner’s graveyard. This is a state-based action. This doesn’t cause a Siege’s intrinsic triggered ability to trigger."
    );
    let mut t = TestGame::new(2);
    let siege = enter_siege(&mut t, P0, P1, THEROS);
    let source = t.battlefield(P1, "Grizzly Bears");
    damage(&mut t, source, 4, siege);
    // The last counter was removed: its ability triggered, and it stays while that
    // ability is on the stack.
    let waiting = battle::defeat_triggers_on_stack(&t.g);
    assert_eq!(waiting.len(), 1);
    assert!(t.on_battlefield(siege));
    // The ability is countered: the Siege is put into its owner's graveyard, and that
    // doesn't trigger it again.
    t.g.counter(waiting[0], None);
    t.settle();
    assert!(t.in_graveyard(P0, "Invasion of Theros"));
    assert!(t.g.stack.is_empty());
}
