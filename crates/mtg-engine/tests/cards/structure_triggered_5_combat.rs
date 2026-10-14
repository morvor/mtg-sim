//! Structure-coverage tests, round 3 (item 5): combat triggered-ability structures shared
//! by many cards that no other test exercised (see `docs/STRUCTURE_COVERAGE.md`).

use super::structure_activated_2::target_choices;
use super::structure_seed_common::*;
use super::structure_triggered_5::{next_turn, obj};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Runs a combat on P0's turn: `attackers` attack P1, who blocks with `blocks`.
fn fight(t: &mut TestGame, attackers: &[ObjectId], blocks: &[(ObjectId, ObjectId)]) {
    let a: Vec<(ObjectId, Entity)> = attackers.iter().map(|a| (*a, Entity::Player(P1))).collect();
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&a, blocks);
}

#[test]
fn giant_badger_and_shu_defender_grow_when_they_block() {
    cr!("509.1i", "509.3a", "611.2a");
    // Giant Badger: "Whenever this creature blocks, it gets +2/+2 until end of turn."
    // Shu Defender: "Whenever this creature blocks, it gets +0/+2 until end of turn."
    for (name, base, grown) in [
        ("Giant Badger", (2, 2), (4, 4)),
        ("Shu Defender", (2, 2), (2, 4)),
    ] {
        supported(name);
        let mut t = TestGame::new(2);
        let b = t.battlefield(P1, name);
        let idle = t.battlefield(P1, name);
        let giant = t.battlefield(P0, "Hill Giant");
        fight(&mut t, &[giant], &[(b, giant)]);
        assert_eq!(t.pt(b), grown, "{name}");
        assert!(t.on_battlefield(b), "{name} survives 3 damage");
        assert_eq!(t.pt(idle), base, "{name}: only the blocker");
        next_turn(&mut t);
        assert_eq!(t.pt(b), base, "{name}: until end of turn");
        // Attacking doesn't trigger it.
        let mut t = TestGame::new(2);
        let b = t.battlefield(P0, name);
        fight(&mut t, &[b], &[]);
        assert_eq!(t.pt(b), base, "{name}");
        assert_eq!(t.life(P1), 20 - base.0);
    }
}

#[test]
fn furtive_courier_loots_when_it_attacks() {
    cr!("508.1m", "508.3a", "121.1", "701.9a");
    // "Whenever this creature attacks, draw a card, then discard a card."
    supported("Furtive Courier");
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "Furtive Courier");
    let other = t.battlefield(P0, "Grizzly Bears");
    let bolt = t.hand(P0, "Lightning Bolt");
    t.answer_choose(P0, &[obj(bolt)]);
    fight(&mut t, &[c], &[]);
    assert_eq!(t.library_size(P0), 29);
    assert_eq!(t.hand_size(P0), 1);
    assert_eq!(t.zone(bolt), Zone::Graveyard(P0));
    assert_eq!(t.life(P1), 17);
    // Another creature attacking alone doesn't trigger it.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Furtive Courier");
    let _ = other;
    let bears = t.battlefield(P0, "Grizzly Bears");
    fight(&mut t, &[bears], &[]);
    assert_eq!(t.library_size(P0), 30);
}

#[test]
fn trusted_pegasus_gives_an_attacking_creature_without_flying_flying() {
    cr!("508.1m", "115.1", "611.2a");
    // "Whenever this creature attacks, target attacking creature without flying gains
    // flying until end of turn."
    supported("Trusted Pegasus");
    let mut t = TestGame::new(2);
    let p = t.battlefield(P0, "Trusted Pegasus");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let home = t.battlefield(P0, "Hill Giant");
    let flier = t.battlefield(P0, "Wind Drake");
    let wall = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[obj(bears)]);
    fight(&mut t, &[p, bears, flier], &[]);
    assert!(t.obj_now(bears).has_keyword(KeywordKind::Flying));
    assert!(!t.obj_now(home).has_keyword(KeywordKind::Flying));
    let offered = &target_choices(&t, P0)[0];
    assert!(offered.contains(&obj(bears)));
    assert!(!offered.contains(&obj(home)), "not attacking");
    assert!(!offered.contains(&obj(flier)) && !offered.contains(&obj(p)), "has flying");
    assert!(!offered.contains(&obj(wall)));
    assert_eq!(t.life(P1), 20 - 2 - 2 - 2);
    next_turn(&mut t);
    assert!(!t.obj_now(bears).has_keyword(KeywordKind::Flying));
}

#[test]
fn virtus_makes_the_damaged_player_lose_half_their_life_rounded_up() {
    cr!("510.3a", "603.2", "107.1a", "119.3");
    // "Whenever Virtus deals combat damage to a player, that player loses half their life,
    // rounded up."
    supported("Virtus the Veiled");
    let mut t = TestGame::new(2);
    let v = t.battlefield(P0, "Virtus the Veiled");
    fight(&mut t, &[v], &[]);
    t.resolve_all();
    // 20 - 1 = 19, then half of 19 rounded up (10).
    assert_eq!(t.life(P1), 9);
    assert_eq!(t.life(P0), 20);
    // Blocked: no damage to a player, no trigger.
    let mut t = TestGame::new(2);
    let v = t.battlefield(P0, "Virtus the Veiled");
    let wall = t.battlefield(P1, "Hill Giant");
    fight(&mut t, &[v], &[(wall, v)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    assert!(!t.on_battlefield(wall), "deathtouch");
}

#[test]
fn locke_cole_loots_on_combat_damage_to_a_player() {
    cr!("510.3a", "603.2", "121.1", "701.9a");
    // "Whenever Locke Cole deals combat damage to a player, draw a card, then discard a
    // card."
    supported("Locke Cole");
    let mut t = TestGame::new(2);
    let l = t.battlefield(P0, "Locke Cole");
    let bolt = t.hand(P0, "Lightning Bolt");
    t.answer_choose(P0, &[obj(bolt)]);
    fight(&mut t, &[l], &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.life(P0), 22, "lifelink");
    assert_eq!(t.library_size(P0), 29);
    assert_eq!(t.hand_size(P0), 1);
    assert_eq!(t.zone(bolt), Zone::Graveyard(P0));
    // Blocked: no loot.
    let mut t = TestGame::new(2);
    let l = t.battlefield(P0, "Locke Cole");
    let wall = t.battlefield(P1, "Hill Giant");
    fight(&mut t, &[l], &[(wall, l)]);
    t.resolve_all();
    assert_eq!(t.library_size(P0), 30);
}

#[test]
fn chambered_nautilus_may_draw_when_blocked() {
    cr!("509.1i", "509.3c", "603.5", "121.1");
    // "Whenever this creature becomes blocked, you may draw a card."
    supported("Chambered Nautilus");
    let mut t = TestGame::new(2);
    let n = t.battlefield(P0, "Chambered Nautilus");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    t.answer_yes(P0, true);
    fight(&mut t, &[n], &[(a, n), (b, n)]);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 1, "once, however many blockers");
    // Declining.
    let mut t = TestGame::new(2);
    let n = t.battlefield(P0, "Chambered Nautilus");
    let a = t.battlefield(P1, "Grizzly Bears");
    t.answer_yes(P0, false);
    fight(&mut t, &[n], &[(a, n)]);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 0);
    // Unblocked: no trigger.
    let mut t = TestGame::new(2);
    let n = t.battlefield(P0, "Chambered Nautilus");
    t.answer_yes(P0, true);
    fight(&mut t, &[n], &[]);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 0);
    assert_eq!(t.life(P1), 18);
}

#[test]
fn contaminated_bond_drains_the_enchanted_creatures_controller_on_attack_or_block() {
    cr!("508.1m", "509.1i", "603.2", "119.3");
    // "Whenever enchanted creature attacks or blocks, its controller loses 3 life."
    supported("Contaminated Bond");
    // Attacking.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let bond = t.battlefield(P1, "Contaminated Bond");
    t.g.attach(bond, obj(bears));
    t.g.recompute();
    fight(&mut t, &[bears], &[]);
    t.resolve_all();
    assert_eq!(t.life(P0), 17, "the creature's controller, not the Aura's");
    assert_eq!(t.life(P1), 18);
    // Blocking.
    let mut t = TestGame::new(2);
    let blocker = t.battlefield(P1, "Hill Giant");
    let bond = t.battlefield(P0, "Contaminated Bond");
    t.g.attach(bond, obj(blocker));
    t.g.recompute();
    let attacker = t.battlefield(P0, "Grizzly Bears");
    fight(&mut t, &[attacker], &[(blocker, attacker)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.life(P0), 20);
    // Another creature attacking doesn't trigger it.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let other = t.battlefield(P0, "Grizzly Bears");
    let bond = t.battlefield(P1, "Contaminated Bond");
    t.g.attach(bond, obj(bears));
    t.g.recompute();
    fight(&mut t, &[other], &[]);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
}

#[test]
fn dread_specter_destroys_nonblack_creatures_it_fights_at_end_of_combat() {
    cr!("509.1i", "509.3f", "603.7a");
    // "Whenever this creature blocks or becomes blocked by a nonblack creature, destroy
    // that creature at end of combat."
    supported("Dread Specter");
    // Becomes blocked by a nonblack creature (which survives the combat damage).
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Dread Specter");
    let wall = t.battlefield(P1, "Wall of Stone");
    fight(&mut t, &[s], &[(wall, s)]);
    assert!(t.on_battlefield(wall), "still there during the end of combat step");
    t.resolve_all();
    assert!(!t.on_battlefield(wall), "destroyed at end of combat");
    assert!(t.on_battlefield(s));
    // Blocks a nonblack creature.
    let mut t = TestGame::new(2);
    let s = t.battlefield(P1, "Dread Specter");
    let giant = t.battlefield(P0, "Craw Wurm");
    fight(&mut t, &[giant], &[(s, giant)]);
    t.resolve_all();
    assert!(!t.on_battlefield(giant));
    assert!(!t.on_battlefield(s), "the Wurm killed it in combat");
    // A black blocker isn't destroyed.
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Dread Specter");
    let black = t.battlefield(P1, "Bog Wraith");
    fight(&mut t, &[s], &[(black, s)]);
    t.resolve_all();
    assert_eq!(t.obj_now(black).damage, 2, "survived the combat damage");
    assert!(t.on_battlefield(black));
    assert!(!t.on_battlefield(s), "the Wraith's 3 damage killed it");
}
