//! Structure-coverage tests, stream 2 (item `structure-tests-2`): combat triggered-ability
//! structures that alone kept their cards from being verified (see
//! `docs/CARD_COVERAGE.md`).

use super::structure_activated_2::{act, next_turn, target_choices};
use super::structure_activated_6::entity_choices;
use super::structure_seed_common::*;
use super::structure_triggered_5::obj;
use super::structure_triggered_7_combat::fight;
use mtg_engine::combat::{block_declaration_legal, block_options};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Declares `attackers` attacking P1 on P0's turn and resolves the attack triggers, stopping
/// in the declare attackers step.
fn attack_and_trigger(t: &mut TestGame, attackers: &[ObjectId]) {
    t.set_step(P0, Step::BeginningOfCombat);
    let decl: Vec<_> = attackers.iter().map(|a| (*a, Entity::Player(P1))).collect();
    declare(t, &decl);
    go_to(t, Step::DeclareAttackers);
    t.resolve_all();
}

#[test]
fn audacious_thief_draws_and_loses_life_when_it_attacks() {
    cr!("508.1m", "603.2", "121.1", "119.3");
    // "Whenever this creature attacks, you draw a card and you lose 1 life."
    supported("Audacious Thief");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Audacious Thief");
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_and_trigger(&mut t, &[bears]);
    assert_eq!(
        (t.hand_size(P0), t.life(P0)),
        (0, 20),
        "another creature attacking"
    );
    let mut t = TestGame::new(2);
    let a2 = t.battlefield(P0, "Audacious Thief");
    attack_and_trigger(&mut t, &[a2]);
    assert_eq!((t.hand_size(P0), t.life(P0)), (1, 19));
    assert_eq!((t.hand_size(P1), t.life(P1)), (0, 20));
}

#[test]
fn phyrexian_pegasus_gives_another_attacker_without_flying_flying() {
    cr!("508.1m", "115.1", "611.2a");
    // "Whenever this creature attacks, another target attacking creature without flying
    // gains flying until end of turn."
    supported("Phyrexian Pegasus");
    let mut t = TestGame::new(2);
    let p = t.battlefield(P0, "Phyrexian Pegasus");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let drake = t.battlefield(P0, "Wind Drake");
    let home = t.battlefield(P0, "Hill Giant");
    t.answer_targets(P0, &[obj(bears)]);
    attack_and_trigger(&mut t, &[p, bears, drake]);
    let ch = target_choices(&t, P0).pop().unwrap();
    assert_eq!(
        ch,
        vec![obj(bears)],
        "another attacking creature without flying"
    );
    assert!(t.obj_now(bears).has_keyword(KeywordKind::Flying));
    assert!(!t.obj_now(home).has_keyword(KeywordKind::Flying));
    next_turn(&mut t);
    assert!(!t.obj_now(bears).has_keyword(KeywordKind::Flying));
}

#[test]
fn bolt_hound_pumps_your_other_creatures_when_it_attacks() {
    cr!("508.1m", "611.2c");
    // "Whenever this creature attacks, other creatures you control get +1/+0 until end of
    // turn."
    supported("Bolt Hound");
    let mut t = TestGame::new(2);
    let h = t.battlefield(P0, "Bolt Hound");
    let home = t.battlefield(P0, "Hill Giant");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    attack_and_trigger(&mut t, &[h]);
    assert_eq!(t.pt(home), (4, 3), "attacking or not");
    assert_eq!(t.pt(h), (2, 2), "not itself");
    assert_eq!(t.pt(theirs), (2, 2));
    next_turn(&mut t);
    assert_eq!(t.pt(home), (3, 3));
}

#[test]
fn moonfolk_puzzlemaker_scries_whenever_it_becomes_tapped() {
    cr!("603.2", "701.22a");
    // "Whenever this creature becomes tapped, scry 1."
    supported("Moonfolk Puzzlemaker");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Moonfolk Puzzlemaker");
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_and_trigger(&mut t, &[bears]);
    assert!(scries(&t).is_empty(), "another creature tapped");
    let mut t = TestGame::new(2);
    let m2 = t.battlefield(P0, "Moonfolk Puzzlemaker");
    attack_and_trigger(&mut t, &[m2]);
    assert_eq!(scries(&t), vec![1], "tapped by attacking");
    // Tapped by an opponent's effect.
    let mut t = TestGame::new(2);
    let m3 = t.battlefield(P0, "Moonfolk Puzzlemaker");
    let servant = t.battlefield(P1, "Air Servant");
    t.lands(P1, "Island", 3);
    act(&mut t, P1, servant, "tap target", &[obj(m3)]).unwrap();
    t.resolve_all();
    assert!(t.obj_now(m3).tapped);
    assert_eq!(scries(&t), vec![1]);
}

#[test]
fn rotting_giant_is_sacrificed_unless_you_exile_a_card_from_your_graveyard() {
    cr!("508.1m", "509.1i", "118.12a");
    // "Whenever this creature attacks or blocks, sacrifice it unless you exile a card from
    // your graveyard."
    supported("Rotting Giant");
    // Attacks, pays: it stays and the card is exiled.
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Rotting Giant");
    let card = t.graveyard(P0, "Forest");
    let theirs = t.graveyard(P1, "Forest");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(card)]);
    attack_and_trigger(&mut t, &[g]);
    let ch = entity_choices(&t, P0).pop().unwrap();
    assert!(
        ch.contains(&obj(card)) && !ch.contains(&obj(theirs)),
        "your graveyard"
    );
    assert!(t.on_battlefield(g));
    assert!(t.in_exile("Forest"));
    assert!(t.in_graveyard(P1, "Forest"));
    // Attacks, declines: sacrificed.
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Rotting Giant");
    t.graveyard(P0, "Forest");
    t.answer_yes(P0, false);
    attack_and_trigger(&mut t, &[g]);
    assert!(t.in_graveyard(P0, "Rotting Giant"));
    assert!(t.in_graveyard(P0, "Forest"));
    // Blocks with an empty graveyard: sacrificed.
    let mut t = TestGame::new(2);
    let g = t.battlefield(P1, "Rotting Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, &[(bears, Entity::Player(P1))]);
    block(&mut t, P1, &[(g, bears)]);
    go_to(&mut t, Step::DeclareBlockers);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Rotting Giant"));
}

#[test]
fn caustic_wasps_may_destroy_an_artifact_of_the_player_it_hit() {
    cr!("510.3a", "603.5", "115.1");
    // "Whenever this creature deals combat damage to a player, you may destroy target
    // artifact that player controls."
    supported("Caustic Wasps");
    for yes in [true, false] {
        let mut t = TestGame::new(2);
        let w = t.battlefield(P0, "Caustic Wasps");
        let ring = t.battlefield(P1, "Sol Ring");
        let mine = t.battlefield(P0, "Sol Ring");
        t.answer_targets(P0, &[obj(ring)]);
        t.answer_yes(P0, yes);
        fight(&mut t, &[w], &[]);
        assert_eq!(t.life(P1), 19);
        let ch = target_choices(&t, P0).pop().unwrap();
        assert_eq!(ch, vec![obj(ring)], "that player's artifacts");
        assert_eq!(t.on_battlefield(ring), !yes, "destroyed: {yes}");
        assert!(t.on_battlefield(mine));
    }
    // Blocked: no damage to the player, no trigger.
    let mut t = TestGame::new(2);
    let w = t.battlefield(P0, "Caustic Wasps");
    let ring = t.battlefield(P1, "Sol Ring");
    let wall = t.battlefield(P1, "Wall of Air");
    fight(&mut t, &[w], &[(wall, w)]);
    assert!(target_choices(&t, P0).is_empty());
    assert!(t.on_battlefield(ring));
}

#[test]
fn heat_of_battle_damages_each_blocking_creatures_controller() {
    cr!("509.1i", "603.2c", "120.3a");
    // "Whenever a creature blocks, this enchantment deals 1 damage to that creature's
    // controller."
    supported("Heat of Battle");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Heat of Battle");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    let c = t.battlefield(P0, "Grizzly Bears");
    let w1 = t.battlefield(P1, "Wall of Wood");
    let w2 = t.battlefield(P1, "Wall of Wood");
    fight(&mut t, &[a, b, c], &[(w1, a), (w2, b)]);
    assert_eq!(
        t.life(P1),
        20 - 2 - 2,
        "1 per blocker, plus the unblocked Bears' 2"
    );
    assert_eq!(
        t.life(P0),
        20,
        "the attackers' controller isn't dealt damage"
    );
}

#[test]
fn clamor_shaman_stops_a_creature_from_blocking() {
    cr!("508.1m", "509.1b", "611.2a");
    // "Whenever this creature attacks, target creature an opponent controls can't block
    // this turn."
    supported("Clamor Shaman");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Clamor Shaman");
    let mine = t.battlefield(P0, "Hill Giant");
    let wall = t.battlefield(P1, "Wall of Wood");
    let other = t.battlefield(P1, "Wall of Air");
    t.answer_targets(P0, &[obj(wall)]);
    attack_and_trigger(&mut t, &[s]);
    let ch = target_choices(&t, P0).pop().unwrap();
    assert!(ch.contains(&obj(wall)) && ch.contains(&obj(other)));
    assert!(
        !ch.contains(&obj(mine)) && !ch.contains(&obj(s)),
        "an opponent's creature"
    );
    let opts = block_options(&t.g, &[P1]);
    assert!(
        !block_declaration_legal(&t.g, &opts, &[(wall, s)]),
        "it can't block"
    );
    assert!(block_declaration_legal(&t.g, &opts, &[(other, s)]));
}

#[test]
fn stinkweed_imp_destroys_creatures_it_deals_combat_damage_to() {
    cr!("510.3a", "603.2");
    // "Whenever this creature deals combat damage to a creature, destroy that creature."
    supported("Stinkweed Imp");
    let mut t = TestGame::new(2);
    let imp = t.battlefield(P0, "Stinkweed Imp");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let spider = t.battlefield(P1, "Giant Spider");
    let wall = t.battlefield(P1, "Wall of Stone");
    fight(&mut t, &[imp, bears], &[(spider, imp), (wall, bears)]);
    assert!(
        t.in_graveyard(P1, "Giant Spider"),
        "1 damage from the Imp destroys the 2/4"
    );
    assert!(
        t.on_battlefield(wall),
        "damage from another creature doesn't"
    );
    // Unblocked, it damages the player and destroys nothing.
    let mut t = TestGame::new(2);
    let imp = t.battlefield(P0, "Stinkweed Imp");
    let wurm = t.battlefield(P1, "Craw Wurm");
    fight(&mut t, &[imp], &[]);
    assert_eq!(t.life(P1), 19);
    assert!(t.on_battlefield(wurm));
}

#[test]
fn no_mercy_destroys_each_creature_that_damages_you() {
    cr!("603.2", "510.3a");
    ruling!("No Mercy", "not only combat damage");
    // "Whenever a creature deals damage to you, destroy it."
    supported("No Mercy");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "No Mercy");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let wall = t.battlefield(P0, "Wall of Stone");
    t.set_step(P1, Step::BeginningOfCombat);
    t.attack(
        &[(bears, Entity::Player(P0)), (giant, Entity::Player(P0))],
        &[(wall, giant)],
    );
    t.resolve_all();
    assert_eq!(t.life(P0), 18);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(
        t.on_battlefield(giant),
        "it dealt damage to a creature, not to you"
    );
    // Noncombat damage from a creature counts; damage to the opponent doesn't.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "No Mercy");
    let pyro = t.battlefield(P1, "Prodigal Pyromancer");
    let other = t.battlefield(P1, "Prodigal Pyromancer");
    act(&mut t, P1, other, "damage", &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    assert!(t.on_battlefield(other));
    act(&mut t, P1, pyro, "damage", &[Entity::Player(P0)]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P0), 19);
    assert!(t.in_graveyard(P1, "Prodigal Pyromancer"));
    assert_eq!(count_named(&t, P1, "Prodigal Pyromancer"), 1);
}

#[test]
fn foot_elite_makes_another_creature_indestructible_when_it_attacks() {
    cr!("508.1m", "702.12b", "611.2a");
    // "Whenever this creature attacks, another target creature you control gets +1/+0
    // and gains indestructible until end of turn."
    supported("Foot Elite");
    let mut t = TestGame::new(2);
    let f = t.battlefield(P0, "Foot Elite");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[obj(bears)]);
    attack_and_trigger(&mut t, &[f]);
    let ch = target_choices(&t, P0).pop().unwrap();
    assert!(ch.contains(&obj(bears)));
    assert!(
        !ch.contains(&obj(f)) && !ch.contains(&obj(theirs)),
        "another, you control"
    );
    assert_eq!(t.pt(bears), (3, 2));
    assert!(t.obj_now(bears).has_keyword(KeywordKind::Indestructible));
    assert!(!t.obj_now(f).has_keyword(KeywordKind::Indestructible));
    next_turn(&mut t);
    assert_eq!(t.pt(bears), (2, 2));
    assert!(!t.obj_now(bears).has_keyword(KeywordKind::Indestructible));
}
