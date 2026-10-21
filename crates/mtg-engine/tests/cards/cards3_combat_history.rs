//! Combat grammar IV: combat history and combat-scoped qualities (CR 506.7, 508.1d,
//! 508.1h, 509.1c): Firkraag, Tolsimir, Cathedral Membrane, Graxiplon, Onakke Oathkeeper.

use crate::combat2_common::*;
use mtg_engine::combat::required_attack_cost;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn multi(n: usize) -> TestGame {
    TestGame::with_config(
        n,
        GameConfig {
            attack_multiple_players: true,
            ..Default::default()
        },
    )
}

#[test]
fn firkraag_rewards_only_creatures_that_had_to_attack() {
    cr!("506.7");
    supported("Firkraag, Cunning Instigator");
    let mut t = multi(3);
    let firk = t.battlefield(P0, "Firkraag, Cunning Instigator");
    let cyclops = t.battlefield(P1, "Bloodrock Cyclops");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let hand = t.hand_size(P0);
    declare(
        &mut t,
        P1,
        &[(cyclops, Entity::Player(P2)), (bears, Entity::Player(P2))],
    );
    t.advance_to(P1, Step::EndOfCombat);
    t.resolve_all();
    assert_eq!(t.life(P2), 20 - 3 - 2);
    assert_eq!(t.counters(firk, "+1/+1"), 1);
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn tolsimir_forces_a_block_on_the_wolf_only_if_he_attacked() {
    cr!("509.1c");
    supported("Tolsimir, Midnight's Light");
    let mut t = TestGame::new(2);
    let tol = t.battlefield(P0, "Tolsimir, Midnight's Light");
    let wolf = t.battlefield(P0, "Timber Wolves");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    declare(
        &mut t,
        P0,
        &[(tol, Entity::Player(P1)), (wolf, Entity::Player(P1))],
    );
    t.resolve_all();
    assert!(!legal_blocks(&mut t, &[]));
    assert!(!legal_blocks(&mut t, &[(bears, tol)]));
    assert!(legal_blocks(&mut t, &[(bears, wolf)]));
}

#[test]
fn tolsimir_doesnt_force_a_block_if_he_didnt_attack() {
    cr!("509.1c");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Tolsimir, Midnight's Light");
    let wolf = t.battlefield(P0, "Timber Wolves");
    t.battlefield(P1, "Grizzly Bears");
    declare(&mut t, P0, &[(wolf, Entity::Player(P1))]);
    t.resolve_all();
    assert!(legal_blocks(&mut t, &[]));
}

#[test]
fn cathedral_membrane_damages_the_creature_it_blocked() {
    cr!("509.1", "608.2h");
    supported("Cathedral Membrane");
    let mut t = TestGame::new(2);
    let wall = t.battlefield(P0, "Cathedral Membrane");
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    declare(
        &mut t,
        P1,
        &[(giant, Entity::Player(P0)), (bears, Entity::Player(P0))],
    );
    declare_blocks(&mut t, P0, &[(wall, giant)]);
    t.advance_to(P1, Step::EndOfCombat);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Cathedral Membrane"));
    assert!(t.in_graveyard(P1, "Hill Giant"));
    assert!(t.on_battlefield(bears));
    assert_eq!(t.life(P0), 18);
}

#[test]
fn graxiplon_can_be_blocked_only_by_a_player_with_three_creatures_sharing_a_type() {
    cr!("509.1b");
    supported("Graxiplon");
    let mut t = TestGame::new(2);
    let grax = t.battlefield(P0, "Graxiplon");
    let b1 = t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Hill Giant");
    declare(&mut t, P0, &[(grax, Entity::Player(P1))]);
    assert!(!legal_blocks(&mut t, &[(b1, grax)]));
    t.battlefield(P1, "Grizzly Bears");
    assert!(legal_blocks(&mut t, &[(b1, grax)]));
}

#[test]
fn onakke_taxes_only_attacks_on_planeswalkers() {
    cr!("508.1d", "508.1h");
    supported("Onakke Oathkeeper");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Onakke Oathkeeper");
    let pw = t.battlefield(P1, "Ajani Goldmane");
    let bears = t.battlefield(P0, "Grizzly Bears");
    to_combat(&mut t, P0);
    t.g.recompute();
    let to_pw = required_attack_cost(&t.g, bears, Entity::Object(pw)).expect("taxed");
    assert_eq!(to_pw.mana.map(|m| m.mana_value()), Some(1));
    assert!(required_attack_cost(&t.g, bears, Entity::Player(P1)).is_none());
}

#[test]
fn tidal_flats_blockers_of_an_unpaid_attacker_gain_first_strike() {
    cr!("509.1g", "510.4");
    supported("Tidal Flats");
    let mut t = TestGame::new(2);
    let flats = t.battlefield(P0, "Tidal Flats");
    t.lands(P0, "Island", 2);
    let lions = t.battlefield(P0, "Savannah Lions");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P1, "Forest", 1);
    declare(&mut t, P1, &[(bears, Entity::Player(P0))]);
    declare_blocks(&mut t, P0, &[(lions, bears)]);
    t.answer_yes(P1, false);
    t.activate(P0, flats, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.g.obj(lions).has_keyword(mtg_engine::keywords::KeywordKind::FirstStrike));
    t.advance_to(P1, Step::EndOfCombat);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(t.on_battlefield(lions));
}

#[test]
fn tidal_flats_paying_keeps_the_blockers_as_they_are() {
    cr!("509.1g");
    let mut t = TestGame::new(2);
    let flats = t.battlefield(P0, "Tidal Flats");
    t.lands(P0, "Island", 2);
    let lions = t.battlefield(P0, "Savannah Lions");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P1, "Forest", 1);
    declare(&mut t, P1, &[(bears, Entity::Player(P0))]);
    declare_blocks(&mut t, P0, &[(lions, bears)]);
    t.answer_yes(P1, true);
    t.activate(P0, flats, 0, &[]).unwrap();
    t.resolve_all();
    assert!(!t.g.obj(lions).has_keyword(mtg_engine::keywords::KeywordKind::FirstStrike));
}

#[test]
fn sewers_of_estark_on_a_blocker_prevents_its_and_its_attackers_damage() {
    cr!("615.1a", "509.1g");
    ruling!(
        "Sewers of Estark",
        "The check for whether that creature is attacking or blocking made on resolution"
    );
    supported("Sewers of Estark");
    let mut t = TestGame::new(2);
    let lions = t.battlefield(P0, "Savannah Lions");
    let spell = t.hand(P0, "Sewers of Estark");
    t.lands(P0, "Swamp", 4);
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    declare(
        &mut t,
        P1,
        &[(giant, Entity::Player(P0)), (bears, Entity::Player(P0))],
    );
    declare_blocks(&mut t, P0, &[(lions, giant)]);
    t.cast(P0, spell).target(Entity::Object(lions)).go();
    t.resolve_all();
    to_end_of_combat(&mut t);
    assert!(t.on_battlefield(lions));
    assert_eq!(t.g.obj(giant).damage, 0);
    // The unblocked bears still deal their damage.
    assert_eq!(t.life(P0), 18);
}

#[test]
fn sewers_of_estark_on_an_attacker_makes_it_unblockable() {
    cr!("509.1b");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let spell = t.hand(P0, "Sewers of Estark");
    t.lands(P0, "Swamp", 4);
    let wall = t.battlefield(P1, "Hill Giant");
    declare(&mut t, P0, &[(bears, Entity::Player(P1))]);
    t.cast(P0, spell).target(Entity::Object(bears)).go();
    t.resolve_all();
    assert!(!legal_blocks(&mut t, &[(wall, bears)]));
}

#[test]
fn camel_prevents_desert_damage_to_itself_and_its_band_while_attacking() {
    cr!("702.22a", "615.1a");
    supported("Camel");
    let mut t = TestGame::new(2);
    let camel = t.battlefield(P0, "Camel");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let lions = t.battlefield(P0, "Savannah Lions");
    let d1 = t.battlefield(P1, "Desert");
    let d2 = t.battlefield(P1, "Desert");
    t.answer_choose(P0, &[Entity::Object(bears)]);
    declare(
        &mut t,
        P0,
        &[
            (camel, Entity::Player(P1)),
            (bears, Entity::Player(P1)),
            (lions, Entity::Player(P1)),
        ],
    );
    t.advance_to(P0, Step::EndOfCombat);
    t.activate(P1, d1, 1, &[Entity::Object(bears)]).unwrap();
    t.resolve_all();
    assert_eq!(t.g.obj(bears).damage, 0);
    t.activate(P1, d2, 1, &[Entity::Object(lions)]).unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Savannah Lions"));
}

#[test]
fn creepy_puppeteer_resizes_the_only_other_attacker() {
    cr!("508.1", "613.4b");
    supported("Creepy Puppeteer");
    let mut t = TestGame::new(2);
    let pup = t.battlefield(P0, "Creepy Puppeteer");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    declare(
        &mut t,
        P0,
        &[(pup, Entity::Player(P1)), (bears, Entity::Player(P1))],
    );
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 3));
    assert_eq!(t.pt(pup), (4, 3));
}

#[test]
fn creepy_puppeteer_does_nothing_with_two_other_attackers() {
    cr!("508.1");
    let mut t = TestGame::new(2);
    let pup = t.battlefield(P0, "Creepy Puppeteer");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let lions = t.battlefield(P0, "Savannah Lions");
    t.answer_yes(P0, true);
    declare(
        &mut t,
        P0,
        &[
            (pup, Entity::Player(P1)),
            (bears, Entity::Player(P1)),
            (lions, Entity::Player(P1)),
        ],
    );
    t.resolve_all();
    assert_eq!(t.pt(bears), (2, 2));
    assert_eq!(t.pt(lions), (2, 1));
}

#[test]
fn jurin_pumps_each_attacker_by_its_defenders_creature_count() {
    cr!("508.1", "608.2h");
    supported("Jurin, Leading the Charge");
    let mut t = TestGame::with_config(
        3,
        GameConfig {
            attack_multiple_players: true,
            ..Default::default()
        },
    );
    let jurin = t.battlefield(P0, "Jurin, Leading the Charge");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Hill Giant");
    t.battlefield(P1, "Hill Giant");
    t.battlefield(P2, "Savannah Lions");
    let (jp, jt) = t.pt(jurin);
    declare(
        &mut t,
        P0,
        &[(jurin, Entity::Player(P1)), (bears, Entity::Player(P2))],
    );
    t.resolve_all();
    assert_eq!(t.pt(jurin), (jp + 2, jt));
    assert_eq!(t.pt(bears), (3, 2));
}

#[test]
fn mishra_melds_with_an_attacking_dragon_engine_and_enters_attacking() {
    cr!("701.42a", "508.4");
    ruling!(
        "Mishra, Claimed by Gix",
        "You choose which player or planeswalker Mishra, Lost to Phyrexia is attacking"
    );
    supported("Mishra, Claimed by Gix");
    let mut t = TestGame::new(2);
    let mishra = t.battlefield(P0, "Mishra, Claimed by Gix");
    let engine = t.battlefield(P0, "Phyrexian Dragon Engine");
    // Mishra, Lost to Phyrexia's enters trigger: discard, menace and trample, Powerstones.
    t.answer(
        P0,
        DecisionKind::Modes,
        mtg_engine::decision::Answer::Indices(vec![0, 3, 5]),
    );
    declare(
        &mut t,
        P0,
        &[(mishra, Entity::Player(P1)), (engine, Entity::Player(P1))],
    );
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.life(P0), 22);
    let m = t.named_on_battlefield("Mishra, Lost to Phyrexia");
    assert_eq!(m.len(), 1);
    assert!(t.g.obj(m[0]).tapped);
    assert!(t.g.is_attacking(m[0]));
}

#[test]
fn mishra_doesnt_meld_when_the_engine_isnt_attacking() {
    cr!("701.42a");
    let mut t = TestGame::new(2);
    let mishra = t.battlefield(P0, "Mishra, Claimed by Gix");
    let engine = t.battlefield(P0, "Phyrexian Dragon Engine");
    declare(&mut t, P0, &[(mishra, Entity::Player(P1))]);
    t.resolve_all();
    assert!(t.named_on_battlefield("Mishra, Lost to Phyrexia").is_empty());
    assert!(t.on_battlefield(engine));
    assert_eq!(t.life(P1), 19);
}
