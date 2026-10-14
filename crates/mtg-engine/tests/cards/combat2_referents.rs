//! Combat taxes for a turn, players' attacks and combat referents (CR 506.2, 508.1c-d,
//! 509.1b-d, 608.2h): War Tax, War Cadence, Sandswirl Wanderglyph, Walking Desecration,
//! Kookus, Tadeas, Sting, Wall of Caltrops, Beckoning Will-o'-Wisp, Triarch Stalker,
//! Rampaging Geoderm, Namor.

use crate::combat2_common::*;
use mtg_engine::combat::{block_costs_by_player, total_attack_cost};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn mana_value(c: &mtg_engine::ability::Cost) -> u32 {
    c.mana.as_ref().map(|m| m.mana_value()).unwrap_or(0)
}

#[test]
fn war_tax_taxes_each_attacking_creature_this_turn() {
    cr!("508.1d", "508.1h", "608.2h");
    ruling!(
        "War Tax",
        "The ability only applies to attackers declared after it resolves."
    );
    supported("War Tax");
    let mut t = TestGame::new(2);
    let tax = t.battlefield(P1, "War Tax");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    t.set_step(P0, Step::BeginningOfCombat);
    t.lands(P1, "Island", 3);
    t.answer(P1, DecisionKind::X, mtg_engine::decision::Answer::Number(2));
    t.activate(P1, tax, 0, &[]).unwrap();
    t.resolve();
    let p1 = Entity::Player(P1);
    assert_eq!(mana_value(&total_attack_cost(&t.g, &[(a, p1)])), 2);
    assert_eq!(mana_value(&total_attack_cost(&t.g, &[(a, p1), (b, p1)])), 4);
    // P0 has two lands: one creature attacks, paying {2}.
    let lands = t.lands(P0, "Forest", 2);
    let attacks = declare(&mut t, P0, &[(a, p1)]);
    assert_eq!(attacks, vec![(a, p1)]);
    assert!(lands.iter().all(|l| t.obj_now(*l).tapped));
    // Next turn, no tax.
    t.advance_to(P1, Step::Upkeep);
    assert!(total_attack_cost(&t.g, &[(a, p1)]).is_free());
}

#[test]
fn war_cadence_taxes_each_blocking_creature_this_turn() {
    cr!("509.1c", "509.1d", "608.2h");
    supported("War Cadence");
    let mut t = TestGame::new(2);
    let cadence = t.battlefield(P0, "War Cadence");
    let a = t.battlefield(P0, "Hill Giant");
    let w = t.battlefield(P1, "Wall of Wood");
    t.set_step(P0, Step::BeginningOfCombat);
    t.lands(P0, "Mountain", 4);
    t.answer(P0, DecisionKind::X, mtg_engine::decision::Answer::Number(3));
    t.activate(P0, cadence, 0, &[]).unwrap();
    t.resolve();
    declare(&mut t, P0, &[(a, Entity::Player(P1))]);
    let costs = block_costs_by_player(&t.g, &[(w, a)]);
    assert_eq!(mana_value(&costs[0].1), 3);
    // P1 can't pay: no block.
    declare_blocks(&mut t, P1, &[(w, a)]);
    assert!(!t.g.is_blocking(w));
}

#[test]
fn sandswirl_wanderglyph_stops_attacks_after_an_opponent_casts_a_spell() {
    cr!("508.1c", "611.2c");
    let mut t = TestGame::new(3);
    let glyph = t.battlefield(P0, "Unstable Glyphbridge // Sandswirl Wanderglyph");
    mtg_engine::dfc::transform(&mut t.g, glyph);
    t.settle();
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Forest", 2);
    let spell = t.hand(P1, "Giant Growth");
    t.cast(P1, spell).target(bears).go();
    t.resolve_all();
    // A creature P1 gains control of later is covered too.
    let late = t.battlefield(P1, "Hill Giant");
    t.advance_to(P1, Step::BeginningOfCombat);
    assert!(!t.g.can_attack_target(bears, Entity::Player(P0)));
    assert!(!t.g.can_attack_target(late, Entity::Player(P0)));
    assert!(t.g.can_attack_target(bears, Entity::Player(P2)));
}

#[test]
fn walking_desecration_makes_the_chosen_type_attack() {
    cr!("508.1d", "608.2h");
    supported("Walking Desecration");
    let mut t = TestGame::new(2);
    let wd = t.battlefield(P0, "Walking Desecration");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P0, "Swamp", 1);
    let bear = mtg_engine::types::subtype_lists()
        .creature
        .iter()
        .position(|s| s == "Bear")
        .unwrap();
    t.answer(
        P0,
        DecisionKind::Option,
        mtg_engine::decision::Answer::Index(bear),
    );
    t.activate(P0, wd, 0, &[]).unwrap();
    t.resolve();
    to_combat(&mut t, P1);
    assert!(!legal_attack(&mut t, &[]));
    assert!(!legal_attack(&mut t, &[(giant, Entity::Player(P0))]));
    assert!(legal_attack(&mut t, &[(bears, Entity::Player(P0))]));
}

#[test]
fn kookus_hits_you_and_attacks_without_its_keeper() {
    cr!("508.1d", "603.4");
    supported("Kookus");
    let mut t = TestGame::new(2);
    let kookus = t.battlefield(P0, "Kookus");
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.life(P0), 17);
    t.advance_to(P0, Step::BeginningOfCombat);
    assert!(!legal_attack(&mut t, &[]));
    assert!(legal_attack(&mut t, &[(kookus, Entity::Player(P1))]));
}

#[test]
fn kookus_with_its_keeper_does_nothing() {
    cr!("603.4");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Kookus");
    t.battlefield(P0, "Keeper of Kookus");
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    t.advance_to(P0, Step::BeginningOfCombat);
    assert!(legal_attack(&mut t, &[]));
}

#[test]
fn tadeas_reach_attackers_cant_be_blocked_by_greater_power() {
    cr!("509.1b");
    supported("Tadeas, Juniper Ascendant");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Tadeas, Juniper Ascendant");
    let spider = t.battlefield(P0, "Giant Spider");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let small = t.battlefield(P1, "Llanowar Elves");
    let equal = t.battlefield(P1, "Grizzly Bears");
    let big = t.battlefield(P1, "Hill Giant");
    t.set_step(P0, Step::BeginningOfCombat);
    declare(
        &mut t,
        P0,
        &[(spider, Entity::Player(P1)), (bears, Entity::Player(P1))],
    );
    t.resolve_all();
    // Untapped (it has no vigilance).
    assert!(!t.obj_now(spider).tapped);
    assert!(legal_blocks(&mut t, &[(small, spider)]));
    assert!(legal_blocks(&mut t, &[(equal, spider)]));
    assert!(!legal_blocks(&mut t, &[(big, spider)]));
    // Without reach: no restriction.
    assert!(legal_blocks(&mut t, &[(big, bears)]));
}

#[test]
fn tadeas_compares_with_the_attackers_power_as_blockers_are_declared() {
    cr!("509.1b");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Tadeas, Juniper Ascendant");
    let spider = t.battlefield(P0, "Giant Spider");
    let big = t.battlefield(P1, "Hill Giant");
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, P0, &[(spider, Entity::Player(P1))]);
    t.resolve_all();
    assert!(!legal_blocks(&mut t, &[(big, spider)]));
    // The Spider gets +3/+3: the Giant's power isn't greater any more.
    t.lands(P0, "Forest", 1);
    let growth = t.hand(P0, "Giant Growth");
    t.cast(P0, growth).target(spider).go();
    t.resolve();
    assert!(legal_blocks(&mut t, &[(big, spider)]));
}

#[test]
fn sting_gives_first_strike_against_goblins() {
    cr!("611.3a", "509.1g");
    supported("Sting, the Glinting Dagger");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let sting = t.battlefield(P0, "Sting, the Glinting Dagger");
    assert!(t.g.attach(sting, Entity::Object(bears)));
    let goblin = t.battlefield(P1, "Goblin Piker");
    let other = t.battlefield(P1, "Grizzly Bears");
    t.settle();
    assert!(!t.obj_now(bears).has_keyword(KeywordKind::FirstStrike));
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, P0, &[(bears, Entity::Player(P1))]);
    declare_blocks(&mut t, P1, &[(goblin, bears)]);
    t.g.recompute();
    assert!(t.obj_now(bears).has_keyword(KeywordKind::FirstStrike));
    to_end_of_combat(&mut t);
    // 3/3 first striker kills the 2/1 Goblin before it deals damage.
    assert!(!t.on_battlefield(goblin));
    assert_eq!(t.g.obj(bears).damage, 0);
    let _ = other;
}

#[test]
fn sting_doesnt_give_first_strike_against_others() {
    cr!("611.3a");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let sting = t.battlefield(P0, "Sting, the Glinting Dagger");
    assert!(t.g.attach(sting, Entity::Object(bears)));
    let other = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, P0, &[(bears, Entity::Player(P1))]);
    declare_blocks(&mut t, P1, &[(other, bears)]);
    t.g.recompute();
    assert!(!t.obj_now(bears).has_keyword(KeywordKind::FirstStrike));
}

#[test]
fn wall_of_caltrops_gains_banding_alongside_other_walls() {
    cr!("603.4", "509.1g");
    ruling!(
        "Wall of Caltrops",
        "Banding is gained when the trigger resolves in the declare blockers step of the combat phase if the criterion is met."
    );
    supported("Wall of Caltrops");
    let mut t = TestGame::new(2);
    let caltrops = t.battlefield(P1, "Wall of Caltrops");
    let wall = t.battlefield(P1, "Wall of Wood");
    let giant = t.battlefield(P0, "Hill Giant");
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, P0, &[(giant, Entity::Player(P1))]);
    declare_blocks(&mut t, P1, &[(caltrops, giant), (wall, giant)]);
    t.resolve_all();
    assert!(t.obj_now(caltrops).has_keyword(KeywordKind::Banding));
}

#[test]
fn wall_of_caltrops_without_banding_when_a_non_wall_also_blocks() {
    cr!("603.4");
    let mut t = TestGame::new(2);
    let caltrops = t.battlefield(P1, "Wall of Caltrops");
    let wall = t.battlefield(P1, "Wall of Wood");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, P0, &[(giant, Entity::Player(P1))]);
    declare_blocks(
        &mut t,
        P1,
        &[(caltrops, giant), (wall, giant), (bears, giant)],
    );
    t.resolve_all();
    assert!(!t.obj_now(caltrops).has_keyword(KeywordKind::Banding));
    // Alone (no other Wall): no banding either.
    let mut t = TestGame::new(2);
    let caltrops = t.battlefield(P1, "Wall of Caltrops");
    let giant = t.battlefield(P0, "Hill Giant");
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, P0, &[(giant, Entity::Player(P1))]);
    declare_blocks(&mut t, P1, &[(caltrops, giant)]);
    t.resolve_all();
    assert!(!t.obj_now(caltrops).has_keyword(KeywordKind::Banding));
}

#[test]
fn beckoning_will_o_wisp_pumps_creatures_attacking_the_chosen_player() {
    cr!("506.2", "607.2d");
    supported("Beckoning Will-o'-Wisp");
    let mut t = TestGame::with_config(
        3,
        GameConfig {
            attack_multiple_players: true,
            ..Default::default()
        },
    );
    t.battlefield(P0, "Beckoning Will-o'-Wisp");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    t.answer_choose(P0, &[Entity::Player(P2)]);
    t.set_step(P0, Step::Upkeep);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    declare(
        &mut t,
        P0,
        &[(a, Entity::Player(P2)), (b, Entity::Player(P1))],
    );
    assert_eq!(t.pt(a), (3, 2));
    assert_eq!(t.pt(b), (2, 2));
}

#[test]
fn triarch_stalker_gives_menace_to_creatures_attacking_the_chosen_player() {
    cr!("506.2", "702.111b");
    supported("Triarch Stalker");
    let mut t = TestGame::with_config(
        3,
        GameConfig {
            attack_multiple_players: true,
            ..Default::default()
        },
    );
    t.battlefield(P0, "Triarch Stalker");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    t.answer_choose(P0, &[Entity::Player(P2)]);
    t.set_step(P0, Step::Upkeep);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    declare(
        &mut t,
        P0,
        &[(a, Entity::Player(P2)), (b, Entity::Player(P1))],
    );
    assert!(t.obj_now(a).has_keyword(KeywordKind::Menace));
    assert!(!t.obj_now(b).has_keyword(KeywordKind::Menace));
}

#[test]
fn rampaging_geoderm_counters_a_creature_attacking_a_battle() {
    cr!("506.2", "310.8", "608.2c");
    supported("Rampaging Geoderm");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rampaging Geoderm");
    let bears = t.battlefield(P0, "Grizzly Bears");
    // P0's Siege enters; P1, the only opponent, protects it (CR 310.8).
    let battle = t.enter(P0, "Invasion of Ikoria // Zilortha, Apex of Ikoria");
    t.resolve_all();
    assert_eq!(mtg_engine::battle::protector(&t.g, battle), Some(P1));
    to_combat(&mut t, P0);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    declare(&mut t, P0, &[(bears, Entity::Object(battle))]);
    t.resolve_all();
    assert_eq!(t.counters(bears, "+1/+1"), 1);
    assert_eq!(t.pt(bears), (3, 3));
}

#[test]
fn rampaging_geoderm_pumps_a_creature_attacking_a_player() {
    cr!("608.2c");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rampaging Geoderm");
    let bears = t.battlefield(P0, "Grizzly Bears");
    to_combat(&mut t, P0);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    declare(&mut t, P0, &[(bears, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(t.counters(bears, "+1/+1"), 0);
    assert_eq!(t.pt(bears), (3, 3));
}

#[test]
fn namor_pumps_the_others_attacking_the_richer_player() {
    cr!("508.3a", "506.2");
    supported("Namor, Atlantean King");
    let mut t = TestGame::with_config(
        3,
        GameConfig {
            attack_multiple_players: true,
            ..Default::default()
        },
    );
    let namor = t.battlefield(P0, "Namor, Atlantean King");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    t.g.players[0].life = 15;
    to_combat(&mut t, P0);
    declare(
        &mut t,
        P0,
        &[
            (namor, Entity::Player(P1)),
            (a, Entity::Player(P1)),
            (b, Entity::Player(P2)),
        ],
    );
    t.resolve_all();
    assert_eq!(t.pt(a), (4, 2));
    assert_eq!(t.pt(b), (2, 2));
    assert_eq!(t.pt(namor), (2, 2));
}
