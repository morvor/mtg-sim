//! Structure-coverage tests, round 5 (item 9): static- and replacement-ability structures
//! shared by several cards that no other test exercised (see `docs/STRUCTURE_COVERAGE.md`).
//! Each test checks a real card's ability does what its text says, including when it
//! doesn't apply.

use super::structure_activated_2::{next_turn, with_p1p1};
use super::structure_activated_6::{attack, entity_choices};
use super::structure_seed_common::*;
use super::structure_static_3::enchant;
use super::structure_triggered_5::{etb, obj};
use super::structure_triggered_7::kill;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::{counters, CardType};
use mtg_engine::*;

// ---------------------------------------------------------------------------
// Characteristic-changing statics
// ---------------------------------------------------------------------------

#[test]
fn abzan_falconer_gives_your_creatures_with_counters_flying() {
    cr!("611.3a", "613.1f");
    // "Each creature you control with a +1/+1 counter on it has flying."
    supported("Abzan Falconer");
    let mut t = TestGame::new(2);
    let f = t.battlefield(P0, "Abzan Falconer");
    let bears = with_p1p1(&mut t, P0, "Grizzly Bears", 1);
    let other = t.battlefield(P0, "Grizzly Bears");
    let theirs = with_p1p1(&mut t, P1, "Grizzly Bears", 1);
    assert!(t.obj_now(bears).has_keyword(KeywordKind::Flying));
    assert!(!t.obj_now(other).has_keyword(KeywordKind::Flying));
    assert!(
        !t.obj_now(f).has_keyword(KeywordKind::Flying),
        "no counter on it"
    );
    assert!(
        !t.obj_now(theirs).has_keyword(KeywordKind::Flying),
        "you control"
    );
    // A counter put on later counts at once.
    t.g.add_counters(obj(other), counters::PLUS1, 1, None);
    t.g.recompute();
    assert!(t.obj_now(other).has_keyword(KeywordKind::Flying));
}

#[test]
fn dralnus_crusade_pumps_all_goblins() {
    cr!("611.3a", "613.4c");
    // "All Goblins get +1/+1."
    supported("Dralnu's Crusade");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Dralnu's Crusade");
    let mine = t.battlefield(P0, "Goblin Hero");
    let theirs = t.battlefield(P1, "Goblin Hero");
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert_eq!(t.pt(mine), (3, 3));
    assert_eq!(t.pt(theirs), (3, 3), "all Goblins, the opponent's too");
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn grasping_scoundrel_gets_plus_1_while_attacking() {
    cr!("611.3a", "613.4c", "511.3");
    // "This creature gets +1/+0 as long as it's attacking."
    supported("Grasping Scoundrel");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Grasping Scoundrel");
    assert_eq!(t.pt(s), (1, 1));
    attack(&mut t, P0, P1, &[s]);
    assert_eq!(t.pt(s), (2, 1));
    go_to(&mut t, Step::PostcombatMain);
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.pt(s), (1, 1), "no longer attacking");
    // Blocking isn't attacking.
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Grasping Scoundrel");
    t.set_step(P1, Step::BeginningOfCombat);
    let goblin = t.battlefield(P1, "Raging Goblin");
    declare(&mut t, &[(goblin, Entity::Player(P0))]);
    block(&mut t, P0, &[(s, goblin)]);
    go_to(&mut t, Step::DeclareBlockers);
    assert_eq!(t.pt(s), (1, 1));
}

#[test]
fn orthodoxy_enforcer_gets_plus_2_with_two_artifacts() {
    cr!("611.3a", "613.4c");
    // "This creature gets +2/+0 as long as you control two or more artifacts."
    supported("Orthodoxy Enforcer");
    let mut t = TestGame::new(2);
    let e = t.battlefield(P0, "Orthodoxy Enforcer");
    t.battlefield(P0, "Sol Ring");
    t.battlefield(P1, "Sol Ring");
    t.battlefield(P1, "Sol Ring");
    assert_eq!(t.pt(e), (2, 4), "the opponent's artifacts don't count");
    let o = t.battlefield(P0, "Ornithopter");
    assert_eq!(t.pt(e), (4, 4));
    kill(&mut t, o);
    assert_eq!(t.pt(e), (2, 4));
}

#[test]
fn colossus_has_indestructible_during_your_turn() {
    cr!("611.3a", "702.12b");
    // "During your turn, Colossus has indestructible."
    supported("Colossus, Steel Stalwart");
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "Colossus, Steel Stalwart");
    let theirs = t.battlefield(P1, "Colossus, Steel Stalwart");
    assert!(t.obj_now(c).has_keyword(KeywordKind::Indestructible));
    assert!(!t.obj_now(theirs).has_keyword(KeywordKind::Indestructible));
    kill(&mut t, c);
    assert!(t.on_battlefield(c));
    next_turn(&mut t);
    assert!(!t.obj_now(c).has_keyword(KeywordKind::Indestructible));
    assert!(t.obj_now(theirs).has_keyword(KeywordKind::Indestructible));
    kill(&mut t, c);
    assert!(!t.on_battlefield(c));
}

#[test]
fn siege_modification_makes_an_enchanted_vehicle_a_creature() {
    cr!("611.3a", "613.1d", "303.4a");
    // "As long as enchanted permanent is a Vehicle, it's a creature in addition to its
    // other types. / Enchanted creature gets +3/+0 and has first strike."
    supported("Siege Modification");
    let mut t = TestGame::new(2);
    let v = t.battlefield(P0, "Smuggler's Copter");
    let other = t.battlefield(P0, "Smuggler's Copter");
    assert!(!t.obj_now(v).is_creature());
    enchant(&mut t, "Siege Modification", "Mountain", 3, v);
    let o = t.obj_now(v);
    assert!(o.is_creature() && o.is(CardType::Artifact));
    assert!(o.chars.has_subtype("Vehicle"));
    assert_eq!(t.pt(v), (6, 3));
    assert!(o.has_keyword(KeywordKind::FirstStrike));
    assert!(!t.obj_now(other).is_creature());
    // On a creature that isn't a Vehicle, only the second ability matters.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    enchant(&mut t, "Siege Modification", "Mountain", 3, bears);
    assert_eq!(t.pt(bears), (5, 2));
    assert!(!t.obj_now(bears).chars.has_subtype("Vehicle"));
}

// ---------------------------------------------------------------------------
// Ward
// ---------------------------------------------------------------------------

#[test]
fn chains_of_custody_gives_the_enchanted_creature_ward_2() {
    cr!("702.21a", "613.1f");
    // "Enchanted creature has ward {2}."
    supported("Chains of Custody");
    // (mountains for P1, whether P1 pays, the Bolt resolves)
    for (mountains, pays, resolves) in [(3, true, true), (3, false, false), (1, true, false)] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let other = t.battlefield(P0, "Grizzly Bears");
        let ring = t.battlefield(P1, "Sol Ring");
        t.lands(P0, "Plains", 3);
        let aura = t.hand(P0, "Chains of Custody");
        t.cast(P0, aura).target(bears).go();
        t.answer_targets(P0, &[obj(ring)]);
        t.resolve_all();
        assert_eq!(t.zone(ring), Zone::Exile);
        assert!(t.obj_now(bears).has_keyword(KeywordKind::Ward));
        assert!(!t.obj_now(other).has_keyword(KeywordKind::Ward));
        t.set_step(P1, Step::PrecombatMain);
        t.lands(P1, "Mountain", mountains);
        let b = t.hand(P1, "Lightning Bolt");
        t.answer_yes(P1, pays);
        t.cast(P1, b).target(bears).go();
        t.resolve_all();
        assert_eq!(!t.on_battlefield(bears), resolves, "{mountains} {pays}");
        assert!(t.in_graveyard(P1, "Lightning Bolt"));
    }
    // The other creature has no ward.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let other = t.battlefield(P0, "Grizzly Bears");
    let ring = t.battlefield(P1, "Sol Ring");
    t.lands(P0, "Plains", 3);
    let aura = t.hand(P0, "Chains of Custody");
    t.cast(P0, aura).target(bears).go();
    t.answer_targets(P0, &[obj(ring)]);
    t.resolve_all();
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Mountain", 1);
    let b = t.hand(P1, "Lightning Bolt");
    t.cast(P1, b).target(other).go();
    t.resolve_all();
    assert!(!t.on_battlefield(other));
}

// ---------------------------------------------------------------------------
// Blocking restrictions
// ---------------------------------------------------------------------------

#[test]
fn spire_tracer_can_be_blocked_only_by_flying_or_reach() {
    cr!("509.1b");
    // "This creature can't be blocked except by creatures with flying or reach."
    supported("Spire Tracer");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Spire Tracer");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let drake = t.battlefield(P1, "Wind Drake");
    let spider = t.battlefield(P1, "Giant Spider");
    let ground = t.battlefield(P1, "Grizzly Bears");
    attack(&mut t, P0, P1, &[s, bears]);
    assert!(t.g.can_block(drake, s), "flying");
    assert!(t.g.can_block(spider, s), "reach");
    assert!(!t.g.can_block(ground, s));
    assert!(t.g.can_block(ground, bears));
}

#[test]
fn amrou_seekers_can_be_blocked_only_by_artifact_or_white_creatures() {
    cr!("509.1b");
    // "This creature can't be blocked except by artifact creatures and/or white
    // creatures."
    supported("Amrou Seekers");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Amrou Seekers");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let white = t.battlefield(P1, "Knight Errant");
    let artifact = t.battlefield(P1, "Ornithopter");
    let green = t.battlefield(P1, "Grizzly Bears");
    attack(&mut t, P0, P1, &[s, bears]);
    assert!(t.g.can_block(white, s));
    assert!(t.g.can_block(artifact, s));
    assert!(!t.g.can_block(green, s));
    assert!(t.g.can_block(green, bears));
}

#[test]
fn air_bladder_lets_the_enchanted_creature_block_only_fliers() {
    cr!("509.1b", "613.1f");
    // "Enchanted creature has flying. / Enchanted creature can block only creatures with
    // flying."
    supported("Air Bladder");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let other = t.battlefield(P0, "Grizzly Bears");
    enchant(&mut t, "Air Bladder", "Island", 1, mine);
    assert!(t.obj_now(mine).has_keyword(KeywordKind::Flying));
    let drake = t.battlefield(P1, "Wind Drake");
    let bears = t.battlefield(P1, "Grizzly Bears");
    attack(&mut t, P1, P0, &[drake, bears]);
    assert!(t.g.can_block(mine, drake));
    assert!(!t.g.can_block(mine, bears));
    assert!(t.g.can_block(other, bears));
}

// ---------------------------------------------------------------------------
// "Enters with" replacement effects
// ---------------------------------------------------------------------------

#[test]
fn bloodcrazed_paladin_enters_with_a_counter_per_creature_that_died_this_turn() {
    cr!("614.1c", "122.1");
    // "This creature enters with a +1/+1 counter on it for each creature that died this
    // turn."
    supported("Bloodcrazed Paladin");
    let mut t = TestGame::new(2);
    let p0 = etb(&mut t, P0, "Bloodcrazed Paladin");
    assert_eq!(t.counters(p0, counters::PLUS1), 0);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    let land = t.battlefield(P1, "Forest");
    kill(&mut t, a);
    kill(&mut t, b);
    kill(&mut t, land);
    let p = etb(&mut t, P0, "Bloodcrazed Paladin");
    assert_eq!(
        t.counters(p, counters::PLUS1),
        2,
        "both players' creatures, not the land"
    );
    assert_eq!(t.pt(p), (3, 3));
    assert_eq!(t.counters(p0, counters::PLUS1), 0, "it doesn't grow");
    // A new turn: nothing died this turn.
    next_turn(&mut t);
    let p = etb(&mut t, P0, "Bloodcrazed Paladin");
    assert_eq!(t.counters(p, counters::PLUS1), 0);
}

// ---------------------------------------------------------------------------
// Additional costs
// ---------------------------------------------------------------------------

#[test]
fn final_vengeance_needs_a_creature_or_enchantment_sacrificed() {
    cr!("118.8", "601.2h", "701.21a");
    // "As an additional cost to cast this spell, sacrifice a creature or enchantment. /
    // Exile target creature."
    supported("Final Vengeance");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let anthem = t.battlefield(P0, "Glorious Anthem");
    let ring = t.battlefield(P0, "Sol Ring");
    let swamp = t.lands(P0, "Swamp", 1)[0];
    let giant = t.battlefield(P1, "Hill Giant");
    let theirs = t.battlefield(P1, "Glorious Anthem");
    let fv = t.hand(P0, "Final Vengeance");
    t.answer_choose(P0, &[obj(anthem)]);
    t.cast(P0, fv).target(giant).go();
    t.resolve_all();
    assert_eq!(t.zone(giant), Zone::Exile);
    assert!(t.in_graveyard(P0, "Glorious Anthem"));
    assert!(t.on_battlefield(bears) && t.on_battlefield(ring) && t.on_battlefield(theirs));
    let offered = entity_choices(&t, P0).concat();
    assert!(offered.contains(&obj(bears)) && offered.contains(&obj(anthem)));
    for no in [ring, swamp, theirs, giant] {
        assert!(!offered.contains(&obj(no)), "{no:?}");
    }
    // Without a creature or enchantment to sacrifice it can't be cast.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sol Ring");
    t.lands(P0, "Swamp", 1);
    let giant = t.battlefield(P1, "Hill Giant");
    let fv = t.hand(P0, "Final Vengeance");
    assert!(t.cast(P0, fv).target(giant).try_go().is_err());
    assert!(t.in_hand(P0, "Final Vengeance"));
    assert!(t.on_battlefield(giant));
}

#[test]
fn angelic_purge_needs_a_permanent_sacrificed() {
    cr!("118.8", "601.2h", "701.21a");
    // "As an additional cost to cast this spell, sacrifice a permanent. / Exile target
    // artifact, creature, or enchantment."
    supported("Angelic Purge");
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 3);
    let forest = t.battlefield(P0, "Forest");
    let ring = t.battlefield(P1, "Sol Ring");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let p = t.hand(P0, "Angelic Purge");
    t.answer_choose(P0, &[obj(forest)]);
    t.cast(P0, p).target(ring).go();
    t.resolve_all();
    assert_eq!(t.zone(ring), Zone::Exile);
    assert!(t.in_graveyard(P0, "Forest"), "a land is a permanent");
    assert!(t.on_battlefield(theirs));
    let offered = entity_choices(&t, P0).concat();
    assert!(offered.contains(&obj(forest)));
    assert!(!offered.contains(&obj(ring)) && !offered.contains(&obj(theirs)));
}
