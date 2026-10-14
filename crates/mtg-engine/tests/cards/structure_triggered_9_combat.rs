//! Structure-coverage tests, round 5 (item 9): combat triggered-ability structures shared
//! by several cards that no other test exercised (see `docs/STRUCTURE_COVERAGE.md`).

use super::structure_activated_2::{next_turn, target_choices};
use super::structure_activated_6::attack;
use super::structure_seed_common::*;
use super::structure_triggered_5::obj;
use super::structure_triggered_7::tokens;
use super::structure_triggered_7_combat::fight;
use super::structure_triggered_9::waiting;
use mtg_engine::dungeons;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::{counters, Color};
use mtg_engine::*;

/// Runs a combat on P1's turn: `attackers` attack P0, who blocks with `blocks`.
fn fight_back(t: &mut TestGame, attackers: &[ObjectId], blocks: &[(ObjectId, ObjectId)]) {
    let a: Vec<(ObjectId, Entity)> = attackers.iter().map(|a| (*a, Entity::Player(P0))).collect();
    t.set_step(P1, Step::BeginningOfCombat);
    t.attack(&a, blocks);
    t.resolve_all();
}

// ---------------------------------------------------------------------------
// "Whenever ~ attacks, ..."
// ---------------------------------------------------------------------------

#[test]
fn mardu_strike_leader_creates_a_warrior_when_it_attacks() {
    cr!("508.3a", "111.1");
    // "Whenever this creature attacks, create a 2/1 black Warrior creature token."
    supported("Mardu Strike Leader");
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Mardu Strike Leader");
    fight(&mut t, &[m], &[]);
    let w = tokens(&t, P0, "Warrior");
    assert_eq!(w.len(), 1);
    assert_eq!(t.pt(w[0]), (2, 1));
    let c = t.obj_now(w[0]).chars.colors;
    assert!(c.contains(Color::Black) && c.count() == 1);
    assert_eq!(t.life(P1), 17, "the token isn't attacking");
    assert!(tokens(&t, P1, "Warrior").is_empty());
    // Blocking doesn't trigger it.
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Mardu Strike Leader");
    let goblin = t.battlefield(P1, "Raging Goblin");
    fight_back(&mut t, &[goblin], &[(m, goblin)]);
    assert!(tokens(&t, P0, "Warrior").is_empty());
}

#[test]
fn raging_regisaur_deals_1_damage_to_any_target_when_it_attacks() {
    cr!("508.3a", "115.4");
    // "Whenever this creature attacks, it deals 1 damage to any target."
    supported("Raging Regisaur");
    let mut t = TestGame::new(2);
    let r = t.battlefield(P0, "Raging Regisaur");
    let goblin = t.battlefield(P1, "Raging Goblin");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(goblin)]);
    fight(&mut t, &[r], &[]);
    assert!(!t.on_battlefield(goblin));
    assert_eq!(t.obj_now(bears).damage, 0);
    assert_eq!(t.life(P1), 16);
    let offered = &target_choices(&t, P0)[0];
    for e in [
        obj(goblin),
        obj(bears),
        obj(r),
        Entity::Player(P0),
        Entity::Player(P1),
    ] {
        assert!(offered.contains(&e), "{e:?}");
    }
    // At a player.
    let mut t = TestGame::new(2);
    let r = t.battlefield(P0, "Raging Regisaur");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    fight(&mut t, &[r], &[]);
    assert_eq!(t.life(P1), 15);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn trained_condor_gives_another_creature_you_control_flying() {
    cr!("508.3a", "115.1", "514.2");
    // "Whenever this creature attacks, another target creature you control gains flying
    // until end of turn."
    supported("Trained Condor");
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "Trained Condor");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    fight(&mut t, &[c], &[]);
    assert!(t.obj_now(bears).has_keyword(KeywordKind::Flying));
    assert!(!t.obj_now(theirs).has_keyword(KeywordKind::Flying));
    let offered = &target_choices(&t, P0)[0];
    assert!(offered.contains(&obj(bears)));
    assert!(!offered.contains(&obj(c)), "another");
    assert!(!offered.contains(&obj(theirs)), "you control");
    next_turn(&mut t);
    assert!(!t.obj_now(bears).has_keyword(KeywordKind::Flying));
}

#[test]
fn agate_blade_assassin_drains_the_defending_player() {
    cr!("508.3a", "119.3");
    // "Whenever this creature attacks, defending player loses 1 life and you gain 1 life."
    supported("Agate-Blade Assassin");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Agate-Blade Assassin");
    fight(&mut t, &[a], &[]);
    assert_eq!((t.life(P0), t.life(P1)), (21, 18));
    // Blocked: the trigger still drains.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Agate-Blade Assassin");
    let wall = t.battlefield(P1, "Wall of Stone");
    fight(&mut t, &[a], &[(wall, a)]);
    assert_eq!((t.life(P0), t.life(P1)), (21, 19));
    // Blocking doesn't.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Agate-Blade Assassin");
    let goblin = t.battlefield(P1, "Raging Goblin");
    fight_back(&mut t, &[goblin], &[(a, goblin)]);
    assert_eq!((t.life(P0), t.life(P1)), (20, 20));
}

#[test]
fn ancestor_dragon_gains_1_life_for_each_attacking_creature() {
    cr!("508.3a", "603.2c", "119.3");
    // "Whenever one or more creatures you control attack, you gain 1 life for each
    // attacking creature."
    supported("Ancestor Dragon");
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Ancestor Dragon");
    let b1 = t.battlefield(P0, "Grizzly Bears");
    let b2 = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Grizzly Bears");
    attack(&mut t, P0, P1, &[d, b1, b2]);
    assert_eq!(waiting(&t), 1, "one trigger for the whole attack");
    t.resolve_all();
    assert_eq!(t.life(P0), 23);
    assert_eq!(t.life(P1), 20);
    // Without the Dragon attacking.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ancestor Dragon");
    let b1 = t.battlefield(P0, "Grizzly Bears");
    attack(&mut t, P0, P1, &[b1]);
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
    // The opponent's attack doesn't.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ancestor Dragon");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    fight_back(&mut t, &[theirs], &[]);
    assert_eq!(t.life(P0), 18);
}

#[test]
fn seifer_almasy_gives_a_creature_attacking_alone_double_strike() {
    cr!("506.5", "508.3a", "702.4b", "514.2");
    // "Whenever a creature you control attacks alone, it gains double strike until end of
    // turn."
    supported("Seifer Almasy");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Seifer Almasy");
    let bears = t.battlefield(P0, "Grizzly Bears");
    fight(&mut t, &[bears], &[]);
    assert_eq!(t.life(P1), 16);
    assert!(t.obj_now(bears).has_keyword(KeywordKind::DoubleStrike));
    assert!(!t.obj_now(s).has_keyword(KeywordKind::DoubleStrike));
    next_turn(&mut t);
    assert!(!t.obj_now(bears).has_keyword(KeywordKind::DoubleStrike));
    // Two attackers: neither attacks alone.
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Seifer Almasy");
    let bears = t.battlefield(P0, "Grizzly Bears");
    fight(&mut t, &[bears, s], &[]);
    assert_eq!(t.life(P1), 15);
    assert!(!t.obj_now(bears).has_keyword(KeywordKind::DoubleStrike));
    // An opponent's creature attacking alone doesn't.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Seifer Almasy");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    fight_back(&mut t, &[theirs], &[]);
    assert_eq!(t.life(P0), 18);
}

#[test]
fn angelic_exaltation_pumps_a_lone_attacker_by_your_creature_count() {
    cr!("506.5", "508.3a", "613.4c", "514.2");
    // "Whenever a creature you control attacks alone, it gets +X/+X until end of turn,
    // where X is the number of creatures you control."
    supported("Angelic Exaltation");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Angelic Exaltation");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Hill Giant");
    t.battlefield(P1, "Grizzly Bears");
    fight(&mut t, &[bears], &[]);
    assert_eq!(
        t.pt(bears),
        (5, 5),
        "three creatures you control, not the opponent's"
    );
    assert_eq!(t.life(P1), 15);
    next_turn(&mut t);
    assert_eq!(t.pt(bears), (2, 2));
    // Two attackers: no pump.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Angelic Exaltation");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    fight(&mut t, &[bears, giant], &[]);
    assert_eq!(t.life(P1), 15);
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn phantom_whelp_returns_to_hand_at_end_of_combat_after_attacking_or_blocking() {
    cr!("508.3a", "509.3a", "603.7a");
    // "When this creature attacks or blocks, return it to its owner's hand at end of
    // combat."
    supported("Phantom Whelp");
    let mut t = TestGame::new(2);
    let w = t.battlefield(P0, "Phantom Whelp");
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack(&mut t, P0, P1, &[w, bears]);
    t.resolve_all();
    go_to(&mut t, Step::CombatDamage);
    assert!(t.on_battlefield(w), "it deals combat damage first");
    assert_eq!(t.life(P1), 16);
    go_to(&mut t, Step::EndOfCombat);
    t.resolve_all();
    assert!(t.in_hand(P0, "Phantom Whelp"));
    assert!(t.on_battlefield(bears));
    // Blocking.
    let mut t = TestGame::new(2);
    let w = t.battlefield(P0, "Phantom Whelp");
    let goblin = t.battlefield(P1, "Raging Goblin");
    fight_back(&mut t, &[goblin], &[(w, goblin)]);
    assert!(!t.on_battlefield(goblin));
    assert!(t.in_hand(P0, "Phantom Whelp"));
    // Neither: it stays.
    let mut t = TestGame::new(2);
    let w = t.battlefield(P0, "Phantom Whelp");
    let bears = t.battlefield(P0, "Grizzly Bears");
    fight(&mut t, &[bears], &[]);
    assert!(t.on_battlefield(w));
}

// ---------------------------------------------------------------------------
// Combat damage triggers
// ---------------------------------------------------------------------------

#[test]
fn shriekgeist_makes_the_damaged_player_mill_two() {
    cr!("510.3a", "701.17a");
    // "Whenever this creature deals combat damage to a player, that player mills two
    // cards."
    supported("Shriekgeist");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Shriekgeist");
    fight(&mut t, &[s], &[]);
    assert_eq!(t.life(P1), 19);
    assert_eq!((t.library_size(P1), t.graveyard_size(P1)), (28, 2));
    assert_eq!((t.library_size(P0), t.graveyard_size(P0)), (30, 0));
    // Blocked: no mill.
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Shriekgeist");
    let spider = t.battlefield(P1, "Giant Spider");
    fight(&mut t, &[s], &[(spider, s)]);
    assert_eq!(t.library_size(P1), 30);
}

#[test]
fn sphere_grid_puts_a_counter_on_your_creature_that_dealt_combat_damage_to_a_player() {
    cr!("510.3a", "122.1");
    // "Whenever a creature you control deals combat damage to a player, put a +1/+1
    // counter on that creature."
    supported("Sphere Grid");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sphere Grid");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let idle = t.battlefield(P0, "Grizzly Bears");
    let wall = t.battlefield(P1, "Wall of Stone");
    fight(&mut t, &[bears, giant], &[(wall, giant)]);
    assert_eq!(t.counters(bears, counters::PLUS1), 1);
    assert_eq!(t.counters(giant, counters::PLUS1), 0, "blocked");
    assert_eq!(t.counters(idle, counters::PLUS1), 0);
    // An opponent's creature doesn't get one.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sphere Grid");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    fight_back(&mut t, &[theirs], &[]);
    assert_eq!(t.life(P0), 18);
    assert_eq!(t.counters(theirs, counters::PLUS1), 0);
}

#[test]
fn yuan_ti_malison_ventures_into_the_dungeon() {
    cr!("510.3a", "701.49a");
    // "Whenever this creature deals combat damage to a player, venture into the dungeon."
    supported("Yuan-Ti Malison");
    let mut t = TestGame::new(2);
    t.custom(
        P0,
        (*card("Tomb of Annihilation")).clone(),
        Zone::Outside(P0),
    );
    let y = t.battlefield(P0, "Yuan-Ti Malison");
    fight(&mut t, &[y], &[]);
    let (d, room) = dungeons::marker(&t.g, P0).expect("ventured");
    assert_eq!(t.obj_now(d).chars.name.as_str(), "Tomb of Annihilation");
    assert_eq!(room, 0);
    // Its first room: "Each player loses 1 life."
    assert_eq!((t.life(P0), t.life(P1)), (19, 17));
    assert!(dungeons::marker(&t.g, P1).is_none());
    // Blocked (it can be blocked when it isn't attacking alone): no venture.
    let mut t = TestGame::new(2);
    t.custom(
        P0,
        (*card("Tomb of Annihilation")).clone(),
        Zone::Outside(P0),
    );
    let y = t.battlefield(P0, "Yuan-Ti Malison");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let wall = t.battlefield(P1, "Wall of Stone");
    fight(&mut t, &[y, bears], &[(wall, y)]);
    assert_eq!(t.life(P1), 18, "the unblocked Bears' damage");
    assert!(dungeons::marker(&t.g, P0).is_none());
}

#[test]
fn skirk_commando_may_damage_a_creature_the_damaged_player_controls() {
    cr!("510.3a", "603.5", "115.1");
    // "Whenever this creature deals combat damage to a player, you may have it deal 2
    // damage to target creature that player controls."
    supported("Skirk Commando");
    for yes in [true, false] {
        let mut t = TestGame::new(2);
        let c = t.battlefield(P0, "Skirk Commando");
        let mine = t.battlefield(P0, "Grizzly Bears");
        let giant = t.battlefield(P1, "Hill Giant");
        t.answer_targets(P0, &[obj(giant)]);
        t.answer_yes(P0, yes);
        fight(&mut t, &[c], &[]);
        assert_eq!(t.life(P1), 18);
        assert_eq!(t.obj_now(giant).damage, if yes { 2 } else { 0 });
        assert_eq!(t.obj_now(mine).damage, 0);
        let offered = &target_choices(&t, P0)[0];
        assert!(offered.contains(&obj(giant)));
        assert!(!offered.contains(&obj(mine)) && !offered.contains(&obj(c)));
    }
}

#[test]
fn scion_of_calamity_destroys_an_artifact_or_enchantment_of_the_damaged_player() {
    cr!("510.3a", "701.8a", "115.1");
    // "Whenever this creature deals combat damage to a player, destroy target artifact or
    // enchantment that player controls."
    supported("Scion of Calamity");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Scion of Calamity");
    let ring = t.battlefield(P1, "Sol Ring");
    let anthem = t.battlefield(P1, "Glorious Anthem");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let mine = t.battlefield(P0, "Sol Ring");
    t.answer_targets(P0, &[obj(anthem)]);
    fight(&mut t, &[s], &[]);
    assert!(!t.on_battlefield(anthem));
    assert!(t.on_battlefield(ring) && t.on_battlefield(mine) && t.on_battlefield(bears));
    let offered = &target_choices(&t, P0)[0];
    assert!(offered.contains(&obj(ring)) && offered.contains(&obj(anthem)));
    assert!(!offered.contains(&obj(mine)), "that player controls");
    assert!(!offered.contains(&obj(bears)));
}

#[test]
fn umezawas_jitte_gets_two_charge_counters_when_equipped_creature_deals_combat_damage() {
    cr!("510.3a", "301.5a", "122.1");
    // "Whenever equipped creature deals combat damage, put two charge counters on
    // Umezawa's Jitte."
    supported("Umezawa's Jitte");
    let mut t = TestGame::new(2);
    let j = t.battlefield(P0, "Umezawa's Jitte");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let other = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 2);
    t.activate(P0, j, 1, &[obj(bears)]).unwrap(); // Equip {2}
    t.resolve_all();
    assert_eq!(t.obj_now(j).attached_to, Some(obj(bears)));
    // The unequipped creature's damage doesn't count.
    fight(&mut t, &[other], &[]);
    assert_eq!(t.counters(j, counters::CHARGE), 0);
    next_turn(&mut t);
    // Combat damage to a creature counts too.
    let wall = t.battlefield(P1, "Wall of Stone");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(bears, Entity::Player(P1))], &[(wall, bears)]);
    t.resolve_all();
    assert_eq!(t.counters(j, counters::CHARGE), 2);
}

#[test]
fn thassas_emissary_draws_when_it_or_the_enchanted_creature_deals_combat_damage() {
    cr!("510.3a", "702.103b");
    // "Whenever this creature or enchanted creature deals combat damage to a player, draw
    // a card. / Enchanted creature gets +3/+3."
    supported("Thassa's Emissary");
    let mut t = TestGame::new(2);
    let e = t.battlefield(P0, "Thassa's Emissary");
    fight(&mut t, &[e], &[]);
    assert_eq!(t.life(P1), 17);
    assert_eq!((t.hand_size(P0), t.hand_size(P1)), (1, 0));
    // Blocked: no card.
    let mut t = TestGame::new(2);
    let e = t.battlefield(P0, "Thassa's Emissary");
    let wall = t.battlefield(P1, "Wall of Stone");
    fight(&mut t, &[e], &[(wall, e)]);
    assert_eq!(t.hand_size(P0), 0);
    // Bestowed on a creature.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let other = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Island", 6);
    let e = t.hand(P0, "Thassa's Emissary");
    t.cast(P0, e)
        .method(CastMethod::Keyword(KeywordKind::Bestow))
        .target(bears)
        .go();
    t.resolve_all();
    assert_eq!(t.pt(bears), (5, 5));
    fight(&mut t, &[other], &[]);
    assert_eq!(t.hand_size(P0), 0, "not the enchanted creature");
    next_turn(&mut t);
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(bears, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 13);
    assert_eq!(t.hand_size(P0), 1);
}
