//! Rulings batch P080 — triggers on attacks: once per attacking creature (CR 508.3a),
//! "attacks you" versus your planeswalkers, triggers of any player's attacks, Isshin's
//! additional triggers (CR 603.2d), "for the rest of the turn" effects on creatures that
//! enter later (CR 611.2c), counters put on before blockers, tokens that aren't attacking
//! and are exiled at end of combat, abilities of permanents with no choice made for them,
//! and Sarkhan's planeswalkers that stop being planeswalkers.

use crate::r_p079_common::*;
use crate::r_s01_common::{attack_with, block_and_finish, supported, triggers_on_stack};
use crate::r_s02_common::destroy;
use crate::r_s06_common::activate_containing;
use crate::r_s07_common::damage_on;
use crate::r_s09_common::legal_attack;
use crate::r_s10_common::attacking;
use crate::r_s13_common::add as add_counters;
use crate::r_s17_common::become_copy;
use crate::r_s21_common::legal_blocks;
use mtg_engine::combat::remove_from_combat;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::CardType;
use mtg_engine::*;

#[test]
fn kazuul_triggers_per_attacker_and_its_ogre_can_block() {
    cr!("508.3a", "118.12", "509.1a");
    ruling!(
        "Kazuul, Tyrant of the Cliffs",
        "If you're the defending player, Kazuul's ability triggers once for each creature that attacks, not just once per combat. The attacking player chooses whether to pay {3} or let you have an Ogre token each time one of those abilities resolves."
    );
    ruling!(
        "Kazuul, Tyrant of the Cliffs",
        "The ability triggers and resolves during the declare attackers step. If you put an Ogre token onto the battlefield as a result, you may block with it during that combat phase."
    );
    supported("Kazuul, Tyrant of the Cliffs");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Kazuul, Tyrant of the Cliffs");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let lions = t.battlefield(P0, "Savannah Lions");
    t.lands(P0, "Wastes", 6);
    attack_with(
        &mut t,
        &[(bears, Entity::Player(P1)), (lions, Entity::Player(P1))],
    );
    assert_eq!(t.stack_len(), 2);
    // P0 pays for one and declines the other.
    t.answer_yes(P0, true);
    t.answer_yes(P0, false);
    let from = t.asked().len();
    t.resolve_all();
    assert_eq!(t.g.turn.step, Step::DeclareAttackers);
    let yes_no = t.asked()[from..]
        .iter()
        .filter(|(p, d)| *p == P0 && matches!(d, Decision::YesNo { .. }))
        .count();
    assert_eq!(yes_no, 2, "the attacking player chooses each time");
    let ogres: Vec<ObjectId> = crate::r_s01_common::tokens(&t, P1);
    assert_eq!(ogres.len(), 1);
    block_and_finish(&mut t, P1, &[(ogres[0], bears)]);
    assert!(!t.on_battlefield(bears), "blocked and killed by the Ogre");
}

#[test]
fn mangara_draws_when_attacked_and_your_planeswalker_attacked() {
    cr!("508.3c", "603.4");
    ruling!(
        "Mangara, the Diplomat",
        "If your opponent attacks you with one creature and your planeswalker with another, you draw a card."
    );
    supported("Mangara, the Diplomat");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Mangara, the Diplomat");
    let jace = t.battlefield(P1, "Jace Beleren");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Savannah Lions");
    attack_with(&mut t, &[(a, Entity::Player(P1)), (b, obj(jace))]);
    let hand = t.hand_size(P1);
    t.resolve_all();
    assert_eq!(t.hand_size(P1), hand + 1);
}

#[test]
fn vengeful_pharaoh_triggers_on_your_own_attackers_damage_to_you() {
    cr!("603.2", "614.1a", "510.2");
    ruling!(
        "Vengeful Pharaoh",
        "If your own attacking creatures deal combat damage to you or a planeswalker you control (perhaps because combat damage was redirected), Vengeful Pharaoh will still trigger. You must target and destroy one of your own attacking creatures."
    );
    supported("Vengeful Pharaoh");
    supported("Harm's Way");
    // P1's Grizzly Bears attack P0; P0's Harm's Way redirects their damage to P1.
    let mut t = TestGame::new(2);
    t.graveyard(P1, "Vengeful Pharaoh");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::BeginningOfCombat);
    attack_with(&mut t, &[(bears, Entity::Player(P0))]);
    t.answer_choose(P0, &[obj(bears)]);
    cast_card(&mut t, P0, "Harm's Way", &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    t.answer_targets(P1, &[obj(bears)]);
    t.advance_to(P1, Step::EndOfCombat);
    assert_eq!((t.life(P0), t.life(P1)), (20, 18), "redirected");
    t.resolve_all();
    assert!(!t.on_battlefield(bears), "P1 destroyed its own attacker");
    let lib = &t.g.player(P1).library;
    assert_eq!(t.g.obj(*lib.last().unwrap()).chars.name, "Vengeful Pharaoh");
}

#[test]
fn isshin_additional_trigger_chooses_targets_separately() {
    cr!("603.2d", "603.3d");
    ruling!(
        "Isshin, Two Heavens as One",
        "Isshin's effect doesn't copy the triggered ability; it just causes the ability to trigger twice. Any choices made as you put the ability onto the stack, such as modes and targets, are made separately for each instance of the ability."
    );
    supported("Isshin, Two Heavens as One");
    supported("Goblin Heelcutter");
    // Goblin Heelcutter: "Whenever this creature attacks, target creature can't block
    // this turn." Two instances, one at each of P1's creatures.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Isshin, Two Heavens as One");
    let heel = t.battlefield(P0, "Goblin Heelcutter");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let wall = t.battlefield(P1, "Wall of Omens");
    t.answer_targets(P0, &[obj(bears)]);
    t.answer_targets(P0, &[obj(wall)]);
    attack_with(&mut t, &[(heel, Entity::Player(P1))]);
    assert_eq!(triggers_on_stack(&t, "can't block"), 2);
    assert!(t
        .g
        .stack
        .iter()
        .all(|id| !crate::r_s04_common::is_spell_copy(&t, *id)));
    t.resolve_all();
    t.g.recompute();
    assert!(!t.g.can_block_at_all(bears) && !t.g.can_block_at_all(wall));
}

#[test]
fn kardur_affects_creatures_that_enter_later() {
    cr!("611.2c", "508.1d");
    ruling!(
        "Kardur, Doomscourge",
        "Kardur's first ability affects all creatures your opponents control, including any that enter the battlefield after the ability resolves."
    );
    supported("Kardur, Doomscourge");
    let mut t = TestGame::new(3);
    t.answer(P0, DecisionKind::Any, Answer::Default);
    t.enter(P0, "Kardur, Doomscourge");
    t.resolve_all();
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.objects[bears.0 as usize].summoning_sick = false;
    t.g.combat = None;
    t.set_step(P1, Step::BeginningOfCombat);
    assert!(!legal_attack(&mut t, &[]), "the new creature must attack");
    assert!(!legal_attack(&mut t, &[(bears, Entity::Player(P0))]));
    assert!(legal_attack(&mut t, &[(bears, Entity::Player(P2))]));
}

#[test]
fn kardur_triggers_for_any_attacking_creature_dying() {
    cr!("603.10a", "506.4");
    ruling!(
        "Kardur, Doomscourge",
        "Kardur's last ability will trigger no matter who controlled the attacking creature and no matter who it was attacking."
    );
    // P1 attacks P2; then P0's own attacker in P0's turn.
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Kardur, Doomscourge");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::BeginningOfCombat);
    attack_with(&mut t, &[(bears, Entity::Player(P2))]);
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1), t.life(P2)), (21, 19, 19));
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Kardur, Doomscourge");
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (21, 19));
}

#[test]
fn lightmine_field_triggers_on_your_own_attack() {
    cr!("508.3d");
    ruling!(
        "Lightmine Field",
        "Lightmine Field’s ability triggers when any creatures attack, including your own."
    );
    supported("Lightmine Field");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Lightmine Field");
    let giant = t.battlefield(P0, "Hill Giant");
    let lions = t.battlefield(P0, "Savannah Lions");
    attack_with(
        &mut t,
        &[(giant, Entity::Player(P1)), (lions, Entity::Player(P1))],
    );
    t.resolve_all();
    assert_eq!(damage_on(&t, giant), 2);
    assert!(!t.on_battlefield(lions));
}

#[test]
fn lulu_and_orims_prayer_ignore_creatures_attacking_planeswalkers() {
    cr!("508.3c", "506.2");
    ruling!(
        "Lulu, Stern Guardian",
        "Lulu's first ability only cares about creatures attacking you, not planeswalkers you control or battles you protect."
    );
    ruling!(
        "Orim's Prayer",
        "The ability only triggers when one or more creatures attack you. It will not trigger on creatures attacking a planeswalker you control."
    );
    for name in ["Lulu, Stern Guardian", "Orim's Prayer"] {
        supported(name);
        // Only P1's planeswalker is attacked: nothing.
        let mut t = TestGame::new(2);
        t.battlefield(P1, name);
        let jace = t.battlefield(P1, "Jace Beleren");
        let a = t.battlefield(P0, "Grizzly Bears");
        let b = t.battlefield(P0, "Savannah Lions");
        attack_with(&mut t, &[(a, obj(jace)), (b, obj(jace))]);
        assert_eq!(t.stack_len(), 0, "{name}");
        // P1 and the planeswalker: Lulu can target only the creature attacking P1; the
        // Prayer counts each attacking creature.
        let mut t = TestGame::new(2);
        t.battlefield(P1, name);
        let jace = t.battlefield(P1, "Jace Beleren");
        let a = t.battlefield(P0, "Grizzly Bears");
        let b = t.battlefield(P0, "Savannah Lions");
        let from = t.asked().len();
        attack_with(&mut t, &[(a, Entity::Player(P1)), (b, obj(jace))]);
        assert_eq!(t.stack_len(), 1, "{name}");
        t.resolve_all();
        if name.starts_with("Lulu") {
            let cands = crate::r_s02_common::target_candidates(&t, P1, from);
            assert_eq!(cands, vec![vec![obj(a)]]);
            assert_eq!(t.counters(a, "stun"), 1);
        } else {
            assert_eq!(t.life(P1), 22);
        }
    }
}

#[test]
fn okagachi_exiles_one_permanent_however_many_attacked() {
    cr!("603.4", "115.1");
    ruling!(
        "O-Kagachi, Vengeful Kami",
        "O-Kagachi's last ability exiles only one nonland permanent, no matter how many creatures attacked you or how many times that player attacked you during their last turn."
    );
    let mut t = TestGame::new(2);
    let ok = t.battlefield(P0, "O-Kagachi, Vengeful Kami");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let lions = t.battlefield(P1, "Savannah Lions");
    let giant = t.battlefield(P1, "Hill Giant");
    t.advance_to(P1, Step::BeginningOfCombat);
    t.attack(
        &[(bears, Entity::Player(P0)), (lions, Entity::Player(P0))],
        &[],
    );
    t.advance_to(P0, Step::BeginningOfCombat);
    t.answer_targets(P0, &[obj(giant)]);
    t.attack(&[(ok, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert!(!t.on_battlefield(giant));
    assert!(t.on_battlefield(bears) && t.on_battlefield(lions));
}

#[test]
fn slumbering_dragon_counter_before_blocks_counts_all_counters() {
    cr!("508.3a", "509.1b", "122.1");
    ruling!(
        "Slumbering Dragon",
        "Slumbering Dragon's third ability will trigger once for each creature that attacks you or a planeswalker you control."
    );
    ruling!(
        "Slumbering Dragon",
        "Slumbering Dragon's second ability counts all +1/+1 counters on it, not just the ones added by its third ability."
    );
    ruling!(
        "Slumbering Dragon",
        "The +1/+1 counter is put on Slumbering Dragon before blockers are declared. This may allow Slumbering Dragon to \"wake up\" and block the creature that attacked and caused its ability to trigger."
    );
    supported("Slumbering Dragon");
    // Two attackers (one at P1's planeswalker): two counters. Starting with three from
    // elsewhere, it has five and can block.
    let mut t = TestGame::new(2);
    let dragon = t.battlefield(P1, "Slumbering Dragon");
    add_counters(&mut t, dragon, "+1/+1", 3);
    let jace = t.battlefield(P1, "Jace Beleren");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Savannah Lions");
    attack_with(&mut t, &[(a, Entity::Player(P1)), (b, obj(jace))]);
    assert_eq!(triggers_on_stack(&t, "+1/+1 counter"), 2);
    assert!(!legal_blocks(&mut t, P1, &[(dragon, a)]));
    t.resolve_all();
    assert_eq!(t.counters(dragon, "+1/+1"), 5);
    assert!(legal_blocks(&mut t, P1, &[(dragon, a)]));
    block_and_finish(&mut t, P1, &[(dragon, a)]);
    assert!(!t.on_battlefield(a));
}

#[test]
fn struggle_for_project_purity_enclave_counts_declared_attackers() {
    cr!("508.3c", "508.4", "122.1");
    ruling!(
        "Struggle for Project Purity",
        "The number of rad counters the attacking player gets when the “Enclave” triggered ability resolves is based on the number of creatures they declared as attackers. Removing creatures from combat won’t reduce the number of rad counters they get."
    );
    supported("Struggle for Project Purity");
    let mut t = TestGame::new(2);
    t.answer(P1, DecisionKind::Option, Answer::Index(1));
    t.enter(P1, "Struggle for Project Purity");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Savannah Lions");
    attack_with(&mut t, &[(a, Entity::Player(P1)), (b, Entity::Player(P1))]);
    assert_eq!(t.stack_len(), 1);
    remove_from_combat(&mut t.g, a);
    enter_attacking(&mut t, P0, "Hill Giant", Entity::Player(P1));
    enter_attacking(&mut t, P0, "Memnite", Entity::Player(P1));
    t.resolve_all();
    assert_eq!(crate::r_s28_common::player_counters(&t, P0, "rad"), 4);
}

#[test]
fn struggle_for_project_purity_without_a_choice_has_neither_ability() {
    cr!("607.2a", "707.2");
    ruling!(
        "Struggle for Project Purity",
        "If you somehow control Struggle for Project Purity and no choice was made for it (perhaps because another permanent on the battlefield became a copy of it), it has neither of the two triggered abilities."
    );
    let mut t = TestGame::new(2);
    let original = t.custom(
        P0,
        (*mtg_engine::card::card("Struggle for Project Purity")).clone(),
        Zone::Exile,
    );
    let copy = t.battlefield(P1, "Memnite");
    become_copy(&mut t, copy, original);
    assert_eq!(
        t.obj_now(copy).chars.name,
        "Struggle for Project Purity",
        "a copy"
    );
    // P0 attacks P1: no Enclave trigger.
    let a = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(a, Entity::Player(P1))]);
    assert_eq!(t.stack_len(), 0);
    t.resolve_all();
    assert_eq!(crate::r_s28_common::player_counters(&t, P0, "rad"), 0);
    // P1's upkeep: no Brotherhood trigger.
    crate::r_s04_common::next_upkeep(&mut t, P1);
    assert_eq!(triggers_on_stack(&t, "draws a card"), 0);
}

#[test]
fn gravity_well_flying_is_lost_before_blockers() {
    cr!("508.2", "509.1b", "702.9b");
    ruling!(
        "Gravity Well",
        "This ability triggers and resolves during the declare attackers step. The creature will no longer have flying by the time blockers are declared."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Gravity Well");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let angel = t.battlefield(P0, "Serra Angel");
    attack_with(&mut t, &[(angel, Entity::Player(P1))]);
    assert!(!legal_blocks(&mut t, P1, &[(bears, angel)]));
    t.resolve_all();
    assert_eq!(t.g.turn.step, Step::DeclareAttackers);
    assert!(legal_blocks(&mut t, P1, &[(bears, angel)]));
}

#[test]
fn riddlekeeper_isnt_a_cost_to_attack() {
    cr!("508.1h", "701.17b");
    ruling!(
        "Riddlekeeper",
        "Riddlekeeper doesn’t impose a cost to attack. Creatures can attack you or a planeswalker you control even if their controller has no cards left in their library."
    );
    supported("Riddlekeeper");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Riddlekeeper");
    crate::r_s11_common::empty_library(&mut t, P0);
    let a = t.battlefield(P0, "Grizzly Bears");
    t.g.combat = None;
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(legal_attack(&mut t, &[(a, Entity::Player(P1))]));
    attack_with(&mut t, &[(a, Entity::Player(P1))]);
    assert_eq!(triggers_on_stack(&t, "mills"), 1);
    t.resolve_all();
    assert!(attacking(&t, a));
    assert!(!t.has_lost(P0));
}

#[test]
fn mirror_mockery_token_isnt_attacking_and_is_exiled_anyway() {
    cr!("508.4", "603.7c", "111.7");
    ruling!(
        "Mirror Mockery",
        "Notably, the token won’t be attacking."
    );
    ruling!(
        "Mirror Mockery",
        "The ability that exiles the token will trigger at end of combat no matter what happens to the original creature or to Mirror Mockery."
    );
    supported("Mirror Mockery");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let mockery = crate::r_s06_common::attach_new(&mut t, P1, "Mirror Mockery", bears);
    t.answer_yes(P1, true);
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    t.resolve_all();
    let tokens = crate::r_s01_common::tokens(&t, P1);
    assert_eq!(tokens.len(), 1);
    assert!(!attacking(&t, tokens[0]));
    assert!(attacking(&t, bears));
    destroy(&mut t, mockery);
    destroy(&mut t, bears);
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    assert!(!t.on_battlefield(tokens[0]));
}

#[test]
fn goblin_kaboomist_land_mines_keep_their_ability() {
    cr!("111.4", "113.7");
    ruling!(
        "Goblin Kaboomist",
        "The Land Mine tokens have the ability even if Goblin Kaboomist blows himself up or otherwise leaves the battlefield."
    );
    supported("Goblin Kaboomist");
    let mut t = TestGame::new(2);
    let kaboomist = t.battlefield(P0, "Goblin Kaboomist");
    crate::r_s04_common::next_upkeep(&mut t, P0);
    t.resolve_all();
    let mines = t.named_on_battlefield("Land Mine");
    assert_eq!(mines.len(), 1);
    if t.on_battlefield(kaboomist) {
        destroy(&mut t, kaboomist);
    }
    // P1's Grizzly Bears attack P0; the Land Mine kills them.
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.combat = None;
    t.set_step(P1, Step::BeginningOfCombat);
    attack_with(&mut t, &[(bears, Entity::Player(P0))]);
    t.lands(P0, "Mountain", 1);
    t.answer_targets(P0, &[obj(bears)]);
    activate_containing(&mut t, P0, mines[0], "damage").expect("activated");
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
}

#[test]
fn sarkhan_planeswalkers_stop_being_planeswalkers_but_keep_loyalty() {
    cr!("205.1a", "306.8", "120.3c");
    ruling!(
        "Sarkhan the Masterless",
        "Once Sarkhan’s first loyalty ability has resolved, each planeswalker you control (including Sarkhan) is no longer a planeswalker for the rest of the turn. They don’t lose any loyalty counters or abilities, and you can still activate their loyalty abilities if you haven’t done so yet this turn. They don’t lose loyalty if they’re dealt damage while they’re not planeswalkers."
    );
    let mut t = TestGame::new(2);
    let sarkhan = t.battlefield(P0, "Sarkhan the Masterless");
    let jace = t.battlefield(P0, "Jace Beleren");
    t.activate(P0, sarkhan, 0, &[]).unwrap();
    t.resolve_all();
    for pw in [sarkhan, jace] {
        let o = t.obj_now(pw);
        assert!(!o.is(CardType::Planeswalker));
        assert!(o.is(CardType::Creature));
    }
    assert_eq!(t.counters(sarkhan, "loyalty"), 6);
    assert_eq!(t.counters(jace, "loyalty"), 3);
    // Shock Jace: damage is marked, no loyalty is lost.
    cast_card(&mut t, P1, "Shock", &[obj(jace)]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(jace, "loyalty"), 3);
    assert_eq!(damage_on(&t, jace), 2);
    // Jace's loyalty ability can still be activated.
    assert!(crate::r_s27_common::can_activate_containing(
        &mut t, P0, jace, "+2"
    ));
    let hand = t.hand_size(P0);
    activate_containing(&mut t, P0, jace, "+2").expect("activated");
    t.resolve_all();
    assert_eq!(t.counters(jace, "loyalty"), 5);
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn stalking_leonin_without_a_chosen_opponent_cant_activate() {
    cr!("607.2a", "707.2", "602.1");
    ruling!(
        "Stalking Leonin",
        "If you didn't choose an opponent when Stalking Leonin entered the battlefield, you can't pay the cost to activate its last ability."
    );
    supported("Stalking Leonin");
    // Entered (P1 chose P0): exiles P0's attacker.
    let mut t = TestGame::new(2);
    t.answer(P1, DecisionKind::Any, Answer::Default);
    let leonin = t.enter(P1, "Stalking Leonin");
    t.resolve_all();
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    t.answer_targets(P1, &[obj(bears)]);
    activate_containing(&mut t, P1, leonin, "Exile").expect("activated");
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    // A creature that became a copy of it: no opponent was chosen.
    let mut t = TestGame::new(2);
    let original = t.custom(
        P1,
        (*mtg_engine::card::card("Stalking Leonin")).clone(),
        Zone::Exile,
    );
    let copy = t.battlefield(P1, "Memnite");
    become_copy(&mut t, copy, original);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    assert!(!crate::r_s02_common::can_activate(&mut t, P1, copy));
}
