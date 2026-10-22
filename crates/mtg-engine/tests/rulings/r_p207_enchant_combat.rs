//! Rulings batch P207 — enchant (CR 303.4, 702.5): Auras that care about damage and
//! combat — "you" and "an opponent" on an Aura on another player's creature, trample
//! over a creature with a destroy-when-damaged Aura, attacking an enchanted planeswalker,
//! evasion, sacrificing a bonus Aura mid-combat, granted abilities on a creature that
//! changes control, and attack-trigger targets in multiplayer.

use crate::r_s01_common::*;
use crate::r_s02_common::target_candidates;
use crate::r_s04_common::*;
use crate::r_s06_common::*;
use crate::r_s09_common::legal_attack;
use crate::r_s21_common::legal_blocks;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

// ---------------------------------------------------------------------------------------
// Curiosity
// ---------------------------------------------------------------------------------------

/// A three-player game: P0's Curiosity enchants P1's Prodigal Pyromancer.
fn curiosity_on_p1s_pyromancer() -> (TestGame, ObjectId) {
    supported("Curiosity");
    supported("Prodigal Pyromancer");
    let mut t = TestGame::new(3);
    let pyro = t.battlefield(P1, "Prodigal Pyromancer");
    attach_new(&mut t, P0, "Curiosity", pyro);
    (t, pyro)
}

/// P1's Pyromancer pings `target`; P0 would draw.
fn ping(t: &mut TestGame, pyro: ObjectId, target: Entity) -> usize {
    let hand = t.hand_size(P0);
    t.answer_yes(P0, true);
    t.activate(P1, pyro, 0, &[target]).unwrap();
    t.resolve_all();
    t.clear_answers();
    t.hand_size(P0) - hand
}

#[test]
fn curiositys_you_and_opponent_are_relative_to_its_controller() {
    cr!("109.5", "603.2", "120.3");
    ruling!(
        "Curiosity",
        "\"You\" refers to the controller of Curiosity, which may be different from the controller of the enchanted creature. \"An opponent\" refers to an opponent of Curiosity's controller."
    );
    ruling!(
        "Curiosity",
        "Any damage dealt by the enchanted creature to an opponent will cause Curiosity to trigger, not just combat damage."
    );
    // Damage to P2 (an opponent of P0): P0 draws (noncombat damage).
    let (mut t, pyro) = curiosity_on_p1s_pyromancer();
    assert_eq!(ping(&mut t, pyro, Entity::Player(P2)), 1);
    assert_eq!(t.life(P2), 19);
    // Damage to P0 (Curiosity's controller): no trigger.
    let (mut t, pyro) = curiosity_on_p1s_pyromancer();
    assert_eq!(ping(&mut t, pyro, Entity::Player(P0)), 0);
    assert_eq!(t.life(P0), 19);
    // Damage to P1 (an opponent of P0, though the creature's controller): P0 draws.
    let (mut t, pyro) = curiosity_on_p1s_pyromancer();
    assert_eq!(ping(&mut t, pyro, Entity::Player(P1)), 1);
}

#[test]
fn curiosity_doesnt_trigger_on_damage_to_a_planeswalker() {
    cr!("120.3c", "603.2");
    ruling!(
        "Curiosity",
        "Curiosity doesn't trigger if the enchanted creature deals damage to a planeswalker or to a battle."
    );
    supported("Liliana of the Veil");
    let (mut t, pyro) = curiosity_on_p1s_pyromancer();
    let lili = t.battlefield(P2, "Liliana of the Veil");
    let loyalty = t.counters(lili, counters::LOYALTY);
    assert_eq!(ping(&mut t, pyro, Entity::Object(lili)), 0);
    assert_eq!(t.counters(lili, counters::LOYALTY), loyalty - 1);
}

// ---------------------------------------------------------------------------------------
// Trample over a creature with a "destroy it when it's dealt damage" Aura
// ---------------------------------------------------------------------------------------

/// P0's Colossal Dreadmaw (6/6 trample) attacks P1 and is blocked by P1's Hill Giant
/// enchanted with `aura`. P0 tries to assign only 1 damage to the Giant; the assignment
/// is illegal and lethal damage (its toughness) must be assigned to it first. Returns the
/// lethal damage offered for the Giant.
fn trample_over(aura: &str) -> u32 {
    supported(aura);
    supported("Colossal Dreadmaw");
    let mut t = TestGame::new(2);
    let maw = t.battlefield(P0, "Colossal Dreadmaw");
    let giant = t.battlefield(P1, "Hill Giant");
    attach_new(&mut t, P1, aura, giant);
    let toughness = t.pt(giant).1;
    let from = t.asked().len();
    t.answer(P0, DecisionKind::Damage, Answer::Numbers(vec![1, 5]));
    t.attack(&[(maw, Entity::Player(P1))], &[(giant, maw)]);
    let lethal: Vec<u32> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::AssignCombatDamage { lethal, .. } => Some(lethal[0]),
            _ => None,
        })
        .collect();
    assert_eq!(lethal, vec![toughness as u32]);
    // Lethal damage to the Giant, the rest to P1.
    assert_eq!(t.life(P1), 20 - (6 - toughness));
    assert!(!t.on_battlefield(giant));
    lethal[0]
}

#[test]
fn trample_assigns_the_blockers_toughness_despite_a_destroy_when_damaged_aura() {
    cr!("702.19c", "510.1c");
    ruling!(
        "Cracked Skull",
        "A creature blocked by a creature enchanted with Cracked Skull still needs to assign damage equal to the enchanted creature's toughness (minus any damage already marked on the enchanted creature) before trampling over to a player, planeswalker, or battle."
    );
    ruling!(
        "Cryoshatter",
        "A creature with trample blocked by a creature enchanted with Cryoshatter still needs to assign damage equal to the enchanted creature’s toughness (minus any damage already marked on the enchanted creature) before trampling over to a player, planeswalker, or battle."
    );
    assert_eq!(trample_over("Cracked Skull"), 3);
    assert_eq!(trample_over("Cryoshatter"), 3);
}

// ---------------------------------------------------------------------------------------
// Attacking and blocking
// ---------------------------------------------------------------------------------------

#[test]
fn a_planeswalker_with_nahiris_binding_may_be_attacked() {
    cr!("506.3", "508.1b", "306.6");
    ruling!(
        "Nahiri's Binding",
        "A planeswalker enchanted this way may be attacked."
    );
    supported("Nahiri's Binding");
    supported("Liliana of the Veil");
    let mut t = TestGame::new(2);
    let lili = t.battlefield(P1, "Liliana of the Veil");
    attach_new(&mut t, P0, "Nahiri's Binding", lili);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(legal_attack(&mut t, &[(bears, Entity::Object(lili))]));
    t.attack(&[(bears, Entity::Object(lili))], &[]);
    assert_eq!(t.counters(lili, counters::LOYALTY), 1);
}

#[test]
fn reach_doesnt_let_a_creature_block_a_treetop_bracers_creature() {
    cr!("702.17b", "509.1b");
    ruling!(
        "Treetop Bracers",
        "Creatures with reach (such as Giant Spider) don't actually have flying, so they can't block a creature enchanted with this."
    );
    supported("Treetop Bracers");
    supported("Giant Spider");
    supported("Serra Angel");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Treetop Bracers", bears);
    let spider = t.battlefield(P1, "Giant Spider");
    let angel = t.battlefield(P1, "Serra Angel");
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    assert!(!legal_blocks(&mut t, P1, &[(spider, bears)]));
    assert!(legal_blocks(&mut t, P1, &[(angel, bears)]));
    block_and_finish(&mut t, P1, &[(spider, bears)]);
    assert_eq!(t.life(P1), 17);
}

#[test]
fn gelid_shackles_doesnt_stop_statics_or_attacking_unless_defender_is_given() {
    cr!("602.5", "702.3b", "508.1c");
    ruling!(
        "Gelid Shackles",
        "Gelid Shackles doesn’t stop static abilities or triggered abilities from working. It also won’t prevent the creature from attacking unless the enchanted creature is given defender before attackers are declared."
    );
    supported("Gelid Shackles");
    supported("Benalish Marshal");
    supported("Snow-Covered Plains");
    // Benalish Marshal: "Other creatures you control get +1/+1."
    let mut t = TestGame::new(2);
    let marshal = t.battlefield(P1, "Benalish Marshal");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let shackles = attach_new(&mut t, P0, "Gelid Shackles", marshal);
    assert_eq!(t.pt(bears), (3, 3));
    t.set_step(P1, Step::BeginningOfCombat);
    assert!(legal_attack(&mut t, &[(marshal, Entity::Player(P0))]));
    // {S}: it gains defender; now it can't attack.
    t.battlefield(P0, "Snow-Covered Plains");
    t.activate(P0, shackles, 0, &[]).unwrap();
    t.resolve_all();
    assert!(has_kw(&t, marshal, KeywordKind::Defender));
    assert!(!legal_attack(&mut t, &[(marshal, Entity::Player(P0))]));
    assert_eq!(t.pt(bears), (3, 3));
}

#[test]
fn sacrificing_inferno_fist_mid_combat_removes_its_bonus() {
    cr!("611.3a", "510.1a");
    ruling!(
        "Inferno Fist",
        "As soon as Inferno Fist leaves the battlefield, its power bonus stops applying. This means that if you sacrifice Inferno Fist before combat damage is dealt, perhaps to destroy a potential blocker, the creature that was enchanted won't have the bonus when it deals combat damage."
    );
    supported("Inferno Fist");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let fist = attach_new(&mut t, P0, "Inferno Fist", bears);
    assert_eq!(t.pt(bears), (4, 2));
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    // Sacrificed to deal 2 damage to a potential blocker.
    let elves = t.battlefield(P1, "Llanowar Elves");
    t.lands(P0, "Mountain", 1);
    t.activate(P0, fist, 0, &[Entity::Object(elves)]).unwrap();
    t.resolve_all();
    assert!(!t.on_battlefield(elves));
    assert_eq!(t.pt(bears), (2, 2));
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.life(P1), 18);
}

#[test]
fn curious_obsession_stays_if_any_creature_attacked() {
    cr!("603.4", "508.1");
    ruling!(
        "Curious Obsession",
        "Curious Obsession's last ability is satisfied if any creature has attacked, similar to raid abilities. The creature it enchants doesn't have to have attacked."
    );
    supported("Curious Obsession");
    // Another creature attacked: it stays.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let elves = t.battlefield(P0, "Llanowar Elves");
    let aura = attach_new(&mut t, P0, "Curious Obsession", elves);
    t.attack(&[(bears, Entity::Player(P1))], &[]);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.on_battlefield(aura));
    // No creature attacked: it's sacrificed.
    let mut t = TestGame::new(2);
    let elves = t.battlefield(P0, "Llanowar Elves");
    let aura = attach_new(&mut t, P0, "Curious Obsession", elves);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(!t.on_battlefield(aura));
    assert!(t.in_graveyard(P0, "Curious Obsession"));
}

#[test]
fn combat_researchs_granted_ability_draws_for_the_creatures_controller() {
    cr!("113.6", "603.3a", "109.5");
    ruling!(
        "Combat Research",
        "Because the card-draw ability is granted to the creature, if an opponent takes control of the enchanted creature and it deals combat damage while they control it, that opponent will draw a card."
    );
    supported("Combat Research");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Combat Research", bears);
    give_control(&mut t, bears, P1);
    // (P1 has controlled it continuously since their most recent turn began.)
    t.g.objects[bears.0 as usize].summoning_sick = false;
    t.set_step(P1, Step::PrecombatMain);
    let (h0, h1) = (t.hand_size(P0), t.hand_size(P1));
    t.attack(&[(bears, Entity::Player(P0))], &[]);
    assert_eq!(t.life(P0), 18);
    assert_eq!(t.hand_size(P1), h1 + 1);
    assert_eq!(t.hand_size(P0), h0);
}

#[test]
fn distracting_geist_targets_only_creatures_of_the_player_it_attacks() {
    cr!("508.5", "508.5a", "115.1");
    ruling!(
        "Distracting Geist // Clever Distraction",
        "Distracting Geist's triggered ability can only target a creature controlled by the player you are attacking with Distracting Geist, even if other creatures you control are also attacking other players. This is also true for the ability of the creature enchanted by Clever Distraction."
    );
    supported("Distracting Geist // Clever Distraction");
    let mut t = TestGame::new(3);
    let geist = t.battlefield(P0, "Distracting Geist // Clever Distraction");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let p1s = t.battlefield(P1, "Hill Giant");
    let p2s = t.battlefield(P2, "Hill Giant");
    let from = t.asked().len();
    attack_with(
        &mut t,
        &[(geist, Entity::Player(P1)), (bears, Entity::Player(P2))],
    );
    let cands = target_candidates(&t, P0, from);
    assert_eq!(cands, vec![vec![Entity::Object(p1s)]]);
    t.resolve_all();
    assert!(t.obj_now(p1s).tapped);
    assert!(!t.obj_now(p2s).tapped);
    // Clever Distraction (cast with disturb) on the Bears: the Bears' granted ability
    // targets only P2's creature, the Geist's only P1's.
    let mut t = TestGame::new(3);
    let other = t.battlefield(P0, "Grizzly Bears");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let card = t.graveyard(P0, "Distracting Geist // Clever Distraction");
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Wastes", 4);
    t.cast(P0, card)
        .method(mtg_engine::object::CastMethod::Keyword(KeywordKind::Disturb))
        .target(bears)
        .go();
    t.resolve_all();
    assert_eq!(
        t.named_on_battlefield("Clever Distraction").len(),
        1,
        "Clever Distraction isn't on the battlefield"
    );
    let p1s = t.battlefield(P1, "Hill Giant");
    let p2s = t.battlefield(P2, "Hill Giant");
    let from = t.asked().len();
    attack_with(
        &mut t,
        &[(other, Entity::Player(P1)), (bears, Entity::Player(P2))],
    );
    assert_eq!(target_candidates(&t, P0, from), vec![vec![Entity::Object(p2s)]]);
    t.resolve_all();
    assert!(!t.obj_now(p1s).tapped);
    assert!(t.obj_now(p2s).tapped);
}

#[test]
fn caught_in_the_brights_goes_to_the_graveyard_after_the_creature_is_exiled() {
    cr!("704.5m", "303.4c");
    ruling!(
        "Caught in the Brights",
        "After the enchanted creature is exiled, Caught in the Brights is put into its owner’s graveyard."
    );
    supported("Caught in the Brights");
    supported("Smuggler's Copter");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let aura = attach_new(&mut t, P0, "Caught in the Brights", giant);
    let copter = t.battlefield(P0, "Smuggler's Copter");
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert!(crew(&mut t, P0, copter, &[bears]));
    t.resolve_all();
    attack_with(&mut t, &[(copter, Entity::Player(P1))]);
    t.resolve_all();
    assert!(t.in_exile("Hill Giant"));
    assert!(!t.on_battlefield(aura));
    assert!(t.in_graveyard(P0, "Caught in the Brights"));
}
