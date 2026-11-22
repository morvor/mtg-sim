//! Structure-coverage tests, round 4 (item 7): triggered-ability structures shared by
//! several cards that no other test exercised (see `docs/STRUCTURE_COVERAGE.md`). Each test
//! checks a real card's ability does what its text says: the right objects and players
//! affected and not others, amounts, durations ending, conditions true and false, and the
//! targets the engine offers. Combat triggers are in `structure_triggered_7_combat.rs`.

use super::structure_activated_2::{act, target_choices};
use super::structure_activated_6::entity_choices;
use super::structure_seed_common::*;
use super::structure_triggered_5::{etb, next_turn, obj};
use mtg_engine::decision::Answer;
use mtg_engine::events::MoveCause;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::{counters, CardType, Color};
use mtg_engine::*;

/// Destroys `id` (it dies) and resolves the triggers.
pub fn kill(t: &mut TestGame, id: ObjectId) {
    let cur = t.g.current(id);
    t.g.destroy(cur, None);
    t.resolve_all();
}

/// Casts `name` from `p`'s hand (paid with `land`s added for it) with `targets`, and
/// resolves everything.
pub fn cast_card(t: &mut TestGame, p: PlayerId, name: &str, land: &str, n: usize, targets: &[Entity]) {
    t.lands(p, land, n);
    let c = t.hand(p, name);
    for e in targets {
        t.answer_targets(p, &[*e]);
    }
    t.cast(p, c).go();
    t.resolve_all();
}

/// Casts a Lightning Bolt from `p`'s hand at `target` and resolves everything.
pub fn bolt(t: &mut TestGame, p: PlayerId, target: Entity) {
    t.lands(p, "Mountain", 1);
    let b = t.hand(p, "Lightning Bolt");
    t.cast(p, b).target(target).go();
    t.resolve_all();
}

/// The tokens with subtype `sub` controlled by `p`.
pub fn tokens(t: &TestGame, p: PlayerId, sub: &str) -> Vec<ObjectId> {
    subtype_ids(t, p, sub)
        .into_iter()
        .filter(|id| t.obj_now(*id).is_token())
        .collect()
}

// ---------------------------------------------------------------------------
// "When ~ enters, ..."
// ---------------------------------------------------------------------------

#[test]
fn final_sting_faerie_destroys_a_creature_dealt_damage_this_turn() {
    cr!("603.6a", "115.1", "701.8a", "603.3d");
    // "When this creature enters, destroy target creature that was dealt damage this
    // turn."
    supported("Final-Sting Faerie");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let shock = t.hand(P0, "Shock");
    t.lands(P0, "Mountain", 1);
    t.cast(P0, shock).target(giant).go();
    t.resolve_all();
    assert_eq!(t.obj_now(giant).damage, 2);
    t.answer_targets(P0, &[obj(giant)]);
    let f = etb(&mut t, P0, "Final-Sting Faerie");
    assert!(!t.on_battlefield(giant));
    assert!(t.on_battlefield(bears) && t.on_battlefield(f));
    let offered = &target_choices(&t, P0)[1];
    assert!(offered.contains(&obj(giant)), "the Shock's target choice came first");
    assert!(!offered.contains(&obj(bears)), "not dealt damage");
    assert!(!offered.contains(&obj(f)));
    // Damage dealt on an earlier turn doesn't count: no legal target, so the ability is
    // removed from the stack.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Mountain", 1);
    let shock = t.hand(P0, "Shock");
    t.cast(P0, shock).target(giant).go();
    t.resolve_all();
    next_turn(&mut t);
    t.answer_targets(P0, &[obj(giant)]);
    etb(&mut t, P0, "Final-Sting Faerie");
    assert!(t.on_battlefield(giant));
    assert!(target_choices(&t, P0).len() == 1, "only the Shock asked for a target");
}

#[test]
fn goblin_goliath_creates_a_goblin_per_opponent() {
    cr!("603.6a", "111.1", "102.2");
    // "When this creature enters, create a number of 1/1 red Goblin creature tokens equal
    // to the number of opponents you have."
    supported("Goblin Goliath");
    for (players, n) in [(2, 1), (3, 2), (4, 3)] {
        let mut t = TestGame::new(players);
        etb(&mut t, P0, "Goblin Goliath");
        let gobs = tokens(&t, P0, "Goblin");
        assert_eq!(gobs.len(), n, "{players} players");
        for g in &gobs {
            assert_eq!(t.pt(*g), (1, 1));
            assert!(t.obj_now(*g).chars.colors.contains(Color::Red));
        }
        assert_eq!(tokens(&t, P1, "Goblin").len(), 0);
    }
}

#[test]
fn gorehorn_raider_deals_2_if_you_attacked_this_turn() {
    cr!("603.4", "603.6a", "115.1", "115.4");
    // "Raid — When this creature enters, if you attacked this turn, this creature deals 2
    // damage to any target."
    supported("Gorehorn Raider");
    // You haven't attacked: it doesn't trigger, so nothing is targeted.
    let mut t = TestGame::new(2);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    etb(&mut t, P0, "Gorehorn Raider");
    assert_eq!(t.life(P1), 20);
    assert!(target_choices(&t, P0).is_empty());
    // After attacking.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(bears, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 18);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    let r = etb(&mut t, P0, "Gorehorn Raider");
    assert_eq!(t.life(P1), 16);
    assert_eq!(t.life(P0), 20);
    let offered = &target_choices(&t, P0)[0];
    for e in [Entity::Player(P0), Entity::Player(P1), obj(giant), obj(bears), obj(r)] {
        assert!(offered.contains(&e), "any target: {e:?}");
    }
}

#[test]
fn hunted_lammasu_gives_target_opponent_a_horror() {
    cr!("603.6a", "115.1", "111.1");
    // "When this creature enters, target opponent creates a 4/4 black Horror creature
    // token."
    supported("Hunted Lammasu");
    let mut t = TestGame::new(3);
    t.answer_targets(P0, &[Entity::Player(P2)]);
    etb(&mut t, P0, "Hunted Lammasu");
    let h = tokens(&t, P2, "Horror");
    assert_eq!(h.len(), 1);
    assert_eq!(t.pt(h[0]), (4, 4));
    assert!(t.obj_now(h[0]).chars.colors.contains(Color::Black));
    assert_eq!(t.obj_now(h[0]).owner, P2);
    assert_eq!(tokens(&t, P0, "Horror").len() + tokens(&t, P1, "Horror").len(), 0);
    let offered = &target_choices(&t, P0)[0];
    assert!(offered.contains(&Entity::Player(P1)) && offered.contains(&Entity::Player(P2)));
    assert!(!offered.contains(&Entity::Player(P0)), "an opponent");
}

#[test]
fn moonsnare_specialist_returns_up_to_one_target_creature() {
    cr!("603.6a", "115.1", "603.3d");
    // "When this creature enters, return up to one target creature to its owner's hand."
    supported("Moonsnare Specialist");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let forest = t.battlefield(P1, "Forest");
    t.answer_targets(P0, &[obj(bears)]);
    let m = etb(&mut t, P0, "Moonsnare Specialist");
    assert_eq!(t.zone(bears), Zone::Hand(P1), "its owner's hand");
    assert!(t.on_battlefield(giant) && t.on_battlefield(m));
    let offered = &target_choices(&t, P0)[0];
    assert!(offered.contains(&obj(giant)) && offered.contains(&obj(m)));
    assert!(!offered.contains(&obj(forest)));
    // Zero targets: nothing returns.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[]);
    let m = etb(&mut t, P0, "Moonsnare Specialist");
    assert!(t.on_battlefield(bears) && t.on_battlefield(m));
    assert_eq!(t.hand_size(P0) + t.hand_size(P1), 0);
}

#[test]
fn kor_skyfisher_returns_a_permanent_you_control() {
    cr!("603.6a");
    // "When this creature enters, return a permanent you control to its owner's hand."
    supported("Kor Skyfisher");
    let mut t = TestGame::new(2);
    let forest = t.battlefield(P0, "Forest");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.answer_choose(P0, &[obj(forest)]);
    let k = etb(&mut t, P0, "Kor Skyfisher");
    assert_eq!(t.zone(forest), Zone::Hand(P0));
    assert!(t.on_battlefield(bears) && t.on_battlefield(theirs) && t.on_battlefield(k));
    assert!(target_choices(&t, P0).is_empty(), "not targeted");
    let offered = &entity_choices(&t, P0)[0];
    assert!(offered.contains(&obj(bears)) && offered.contains(&obj(k)));
    assert!(!offered.contains(&obj(theirs)), "you control");
    // Alone, it returns itself.
    let mut t = TestGame::new(2);
    let k = etb(&mut t, P0, "Kor Skyfisher");
    assert_eq!(t.zone(k), Zone::Hand(P0));
}

#[test]
fn lava_hounds_deals_4_damage_to_you() {
    cr!("603.6a", "120.3a");
    // "When this creature enters, it deals 4 damage to you."
    supported("Lava Hounds");
    let mut t = TestGame::new(2);
    let h = etb(&mut t, P0, "Lava Hounds");
    assert_eq!(t.life(P0), 16);
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.obj_now(h).damage, 0);
    // "You" is its controller.
    let mut t = TestGame::new(2);
    etb(&mut t, P1, "Lava Hounds");
    assert_eq!((t.life(P0), t.life(P1)), (20, 16));
}

#[test]
fn mechanozoa_taps_and_stuns_an_opponents_artifact_or_creature() {
    cr!("603.6a", "115.1", "701.26a", "122.1d");
    // "When this creature enters, tap target artifact or creature an opponent controls and
    // put a stun counter on it."
    supported("Mechanozoa");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let stone = t.battlefield(P1, "Mind Stone");
    let forest = t.battlefield(P1, "Forest");
    let mine = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[obj(stone)]);
    let m = etb(&mut t, P0, "Mechanozoa");
    assert!(t.obj_now(stone).tapped);
    assert_eq!(t.counters(stone, counters::STUN), 1);
    assert!(!t.obj_now(bears).tapped);
    assert_eq!(t.counters(bears, counters::STUN), 0);
    let offered = &target_choices(&t, P0)[0];
    assert!(offered.contains(&obj(bears)), "a creature");
    assert!(!offered.contains(&obj(forest)), "neither artifact nor creature");
    assert!(!offered.contains(&obj(mine)) && !offered.contains(&obj(m)), "an opponent's");
    // The stun counter keeps it tapped through its controller's untap step.
    t.advance_to(P1, Step::Upkeep);
    assert!(t.obj_now(stone).tapped);
    assert_eq!(t.counters(stone, counters::STUN), 0);
}

#[test]
fn momo_deals_4_to_a_tapped_creature_an_opponent_controls() {
    cr!("603.6a", "115.1", "120.3e");
    // "When Momo enters, he deals 4 damage to target tapped creature an opponent
    // controls."
    supported("Momo, Rambunctious Rascal");
    let mut t = TestGame::new(2);
    let tapped = t.battlefield(P1, "Hill Giant");
    t.g.tap(tapped);
    let tapped2 = t.battlefield(P1, "Air Elemental");
    t.g.tap(tapped2);
    let untapped = t.battlefield(P1, "Grizzly Bears");
    let mine = t.battlefield(P0, "Hill Giant");
    t.g.tap(mine);
    t.answer_targets(P0, &[obj(tapped)]);
    let m = etb(&mut t, P0, "Momo, Rambunctious Rascal");
    assert!(!t.on_battlefield(tapped));
    assert_eq!(t.obj_now(tapped2).damage, 0);
    assert_eq!(t.life(P1), 20);
    let offered = &target_choices(&t, P0)[0];
    assert!(offered.contains(&obj(tapped2)));
    assert!(!offered.contains(&obj(untapped)), "untapped");
    assert!(!offered.contains(&obj(mine)) && !offered.contains(&obj(m)), "an opponent's");
    assert!(!offered.contains(&Entity::Player(P1)));
}

#[test]
fn harrier_strix_taps_target_permanent() {
    cr!("603.6a", "115.1", "701.26a");
    // "When this creature enters, tap target permanent."
    supported("Harrier Strix");
    let mut t = TestGame::new(2);
    let forest = t.battlefield(P1, "Forest");
    let stone = t.battlefield(P1, "Mind Stone");
    let mine = t.battlefield(P0, "Grizzly Bears");
    let gy = t.graveyard(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(forest)]);
    let s = etb(&mut t, P0, "Harrier Strix");
    assert!(t.obj_now(forest).tapped);
    assert!(!t.obj_now(stone).tapped && !t.obj_now(mine).tapped && !t.obj_now(s).tapped);
    let offered = &target_choices(&t, P0)[0];
    for e in [obj(stone), obj(mine), obj(s)] {
        assert!(offered.contains(&e), "any permanent: {e:?}");
    }
    assert!(!offered.contains(&obj(gy)) && !offered.contains(&Entity::Player(P1)));
}

#[test]
fn quirion_sentinel_adds_one_mana_of_any_color() {
    cr!("603.6a", "106.4");
    // "When this creature enters, add one mana of any color."
    supported("Quirion Sentinel");
    // Options are W, U, B, R, G: index 0 is white.
    let mut t = TestGame::new(2);
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    etb(&mut t, P0, "Quirion Sentinel");
    assert_eq!(t.g.player(P0).mana_pool.count(ManaType::W), 1);
    assert_eq!(t.g.player(P0).mana_pool.mana.len(), 1);
    assert!(t.g.player(P1).mana_pool.mana.is_empty());
    // Any color: index 4 is green.
    let mut t = TestGame::new(2);
    t.answer(P0, DecisionKind::Option, Answer::Index(4));
    etb(&mut t, P0, "Quirion Sentinel");
    assert_eq!(t.g.player(P0).mana_pool.count(ManaType::G), 1);
    assert_eq!(t.g.player(P0).mana_pool.mana.len(), 1);
}

#[test]
fn monk_idealist_returns_an_enchantment_card_from_your_graveyard() {
    cr!("603.6a", "115.1");
    // "When this creature enters, return target enchantment card from your graveyard to
    // your hand."
    supported("Monk Idealist");
    let mut t = TestGame::new(2);
    let ench = t.graveyard(P0, "Mass Hysteria");
    let other = t.graveyard(P0, "Grizzly Bears");
    let theirs = t.graveyard(P1, "Mass Hysteria");
    let on_bf = t.battlefield(P0, "Mass Hysteria");
    t.answer_targets(P0, &[obj(ench)]);
    etb(&mut t, P0, "Monk Idealist");
    assert_eq!(t.zone(ench), Zone::Hand(P0));
    assert_eq!(t.zone(other), Zone::Graveyard(P0));
    let offered = &target_choices(&t, P0)[0];
    assert!(!offered.contains(&obj(other)), "not an enchantment");
    assert!(!offered.contains(&obj(theirs)), "your graveyard");
    assert!(!offered.contains(&obj(on_bf)), "a card in a graveyard");
}

#[test]
fn petravark_exiles_target_land_until_it_leaves() {
    cr!("603.6a", "115.1", "701.13a", "603.6c");
    // "When this creature enters, exile target land. / When this creature leaves the
    // battlefield, return the exiled card to the battlefield under its owner's control."
    supported("Petravark");
    let mut t = TestGame::new(2);
    let forest = t.battlefield(P1, "Forest");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let mine = t.battlefield(P0, "Mountain");
    t.answer_targets(P0, &[obj(forest)]);
    let p = etb(&mut t, P0, "Petravark");
    assert_eq!(t.zone(forest), Zone::Exile);
    assert!(t.on_battlefield(bears) && t.on_battlefield(mine));
    let offered = &target_choices(&t, P0)[0];
    assert!(offered.contains(&obj(mine)), "any land");
    assert!(!offered.contains(&obj(bears)) && !offered.contains(&obj(p)));
    kill(&mut t, p);
    assert!(t.on_battlefield(forest));
    assert_eq!(t.obj_now(forest).controller, P1);
}

#[test]
fn portable_hole_exiles_a_cheap_nonland_permanent_until_it_leaves() {
    cr!("603.6a", "115.1", "610.3");
    // "When this artifact enters, exile target nonland permanent an opponent controls with
    // mana value 2 or less until this artifact leaves the battlefield."
    supported("Portable Hole");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let stone = t.battlefield(P1, "Mind Stone");
    let giant = t.battlefield(P1, "Hill Giant");
    let forest = t.battlefield(P1, "Forest");
    let mine = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    let h = etb(&mut t, P0, "Portable Hole");
    assert_eq!(t.zone(bears), Zone::Exile);
    let offered = &target_choices(&t, P0)[0];
    assert!(offered.contains(&obj(stone)), "mana value 2");
    assert!(!offered.contains(&obj(giant)), "mana value 4");
    assert!(!offered.contains(&obj(forest)), "a land");
    assert!(!offered.contains(&obj(mine)) && !offered.contains(&obj(h)), "an opponent's");
    kill(&mut t, h);
    assert!(t.on_battlefield(bears));
    assert_eq!(t.obj_now(bears).controller, P1);
}

#[test]
fn reclamation_sage_may_destroy_an_artifact_or_enchantment() {
    cr!("603.6a", "115.1", "701.8a", "603.5");
    // "When this creature enters, you may destroy target artifact or enchantment."
    supported("Reclamation Sage");
    for yes in [true, false] {
        let mut t = TestGame::new(2);
        let stone = t.battlefield(P1, "Mind Stone");
        let ench = t.battlefield(P1, "Mass Hysteria");
        let bears = t.battlefield(P1, "Grizzly Bears");
        let forest = t.battlefield(P1, "Forest");
        let mine = t.battlefield(P0, "Mind Stone");
        t.answer_targets(P0, &[obj(ench)]);
        t.answer_yes(P0, yes);
        etb(&mut t, P0, "Reclamation Sage");
        assert_eq!(t.on_battlefield(ench), !yes, "destroyed: {yes}");
        assert!(t.on_battlefield(stone) && t.on_battlefield(mine));
        let offered = &target_choices(&t, P0)[0];
        assert!(offered.contains(&obj(stone)) && offered.contains(&obj(mine)));
        assert!(!offered.contains(&obj(bears)) && !offered.contains(&obj(forest)));
    }
}

#[test]
fn rishadan_cutpurse_makes_each_opponent_sacrifice_unless_they_pay_1() {
    cr!("603.6a", "118.12a", "701.21a");
    // "When this creature enters, each opponent sacrifices a permanent of their choice
    // unless they pay {1}."
    supported("Rishadan Cutpurse");
    let mut t = TestGame::new(3);
    let f1 = t.battlefield(P1, "Forest");
    let b1 = t.battlefield(P1, "Grizzly Bears");
    let f2 = t.battlefield(P2, "Forest");
    let b2 = t.battlefield(P2, "Grizzly Bears");
    let mine = t.battlefield(P0, "Grizzly Bears");
    t.answer_yes(P1, true);
    t.answer_yes(P2, false);
    t.answer_choose(P2, &[obj(b2)]);
    let c = etb(&mut t, P0, "Rishadan Cutpurse");
    // P1 paid with their Forest.
    assert!(t.on_battlefield(f1) && t.obj_now(f1).tapped && t.on_battlefield(b1));
    // P2 didn't and sacrificed the permanent they chose.
    assert!(!t.on_battlefield(b2) && t.on_battlefield(f2));
    assert!(t.on_battlefield(mine) && t.on_battlefield(c));
    let offered = entity_choices(&t, P2);
    assert!(offered[0].contains(&obj(f2)));
    assert!(!offered[0].contains(&obj(mine)), "their own permanents");
    assert!(entity_choices(&t, P0).is_empty(), "the opponents choose");
    // An opponent who wants to pay but can't sacrifices.
    let mut t = TestGame::new(2);
    let b1 = t.battlefield(P1, "Grizzly Bears");
    t.answer_yes(P1, true);
    etb(&mut t, P0, "Rishadan Cutpurse");
    assert!(!t.on_battlefield(b1));
}

// ---------------------------------------------------------------------------
// "When ~ dies, ..."
// ---------------------------------------------------------------------------

#[test]
fn forsaken_drifters_mills_four_when_it_dies() {
    cr!("603.6c", "700.4", "701.17a");
    // "When this creature dies, mill four cards."
    supported("Forsaken Drifters");
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Forsaken Drifters");
    kill(&mut t, d);
    assert_eq!(t.library_size(P0), 26);
    assert_eq!(t.graveyard_size(P0), 5, "itself and the four milled cards");
    assert_eq!(t.library_size(P1), 30);
    // Exiled, it doesn't die.
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Forsaken Drifters");
    t.g.exile_object(d, None);
    t.resolve_all();
    assert_eq!(t.library_size(P0), 30);
}

#[test]
fn goblin_gardener_destroys_target_land_when_it_dies() {
    cr!("603.6c", "700.4", "115.1", "701.8a");
    // "When this creature dies, destroy target land."
    supported("Goblin Gardener");
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Goblin Gardener");
    let forest = t.battlefield(P1, "Forest");
    let keep = t.battlefield(P1, "Forest");
    let mine = t.battlefield(P0, "Mountain");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(forest)]);
    kill(&mut t, g);
    assert!(!t.on_battlefield(forest));
    assert!(t.on_battlefield(keep) && t.on_battlefield(mine) && t.on_battlefield(bears));
    let offered = &target_choices(&t, P0)[0];
    assert!(offered.contains(&obj(mine)) && offered.contains(&obj(keep)));
    assert!(!offered.contains(&obj(bears)));
}

#[test]
fn festering_goblin_gives_target_creature_minus_1_when_it_dies() {
    cr!("603.6c", "700.4", "115.1", "611.2a", "514.2");
    // "When this creature dies, target creature gets -1/-1 until end of turn."
    supported("Festering Goblin");
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Festering Goblin");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let other = t.battlefield(P1, "Hill Giant");
    let forest = t.battlefield(P1, "Forest");
    t.answer_targets(P0, &[obj(bears)]);
    kill(&mut t, g);
    assert_eq!(t.pt(bears), (1, 1));
    assert_eq!(t.pt(other), (3, 3));
    let offered = &target_choices(&t, P0)[0];
    assert!(offered.contains(&obj(other)));
    assert!(!offered.contains(&obj(forest)) && !offered.contains(&Entity::Player(P1)));
    next_turn(&mut t);
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn junk_diver_returns_another_artifact_card_when_it_dies() {
    cr!("603.6c", "700.4", "115.1", "603.10a");
    // "When this creature dies, return another target artifact card from your graveyard to
    // your hand."
    supported("Junk Diver");
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Junk Diver");
    let stone = t.graveyard(P0, "Mind Stone");
    let bears = t.graveyard(P0, "Grizzly Bears");
    let theirs = t.graveyard(P1, "Mind Stone");
    t.answer_targets(P0, &[obj(stone)]);
    kill(&mut t, d);
    assert_eq!(t.zone(stone), Zone::Hand(P0));
    assert_eq!(t.zone(d), Zone::Graveyard(P0));
    let offered = &target_choices(&t, P0)[0];
    assert!(!offered.contains(&obj(t.g.current(d))), "another");
    assert!(!offered.contains(&obj(bears)), "an artifact card");
    assert!(!offered.contains(&obj(theirs)), "your graveyard");
}

#[test]
fn merfolk_coralsmith_scries_2_when_it_dies() {
    cr!("603.6c", "700.4", "701.22a");
    // "When this creature dies, scry 2."
    supported("Merfolk Coralsmith");
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "Merfolk Coralsmith");
    kill(&mut t, c);
    assert_eq!(scries(&t), vec![2]);
    // Returned to hand, it doesn't die.
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "Merfolk Coralsmith");
    t.g.move_object(c, Zone::Hand(P0), MoveCause::Effect, None);
    t.resolve_all();
    assert!(scries(&t).is_empty());
}

#[test]
fn murderous_rider_goes_to_the_bottom_of_its_owners_library_when_it_dies() {
    cr!("603.6c", "700.4", "603.10a");
    // "When this creature dies, put it on the bottom of its owner's library."
    supported("Murderous Rider // Swift End");
    let mut t = TestGame::new(2);
    let r = t.battlefield(P0, "Murderous Rider // Swift End");
    kill(&mut t, r);
    assert_eq!(t.zone(r), Zone::Library(P0));
    assert_eq!(t.g.player(P0).library[0], t.g.current(r), "on the bottom");
    assert_eq!(t.library_size(P0), 31);
    assert_eq!(t.graveyard_size(P0), 0);
}

#[test]
fn patron_of_the_arts_creates_a_treasure_when_it_enters_or_dies() {
    cr!("603.6a", "603.6c", "111.10a");
    // "When this creature enters or dies, create a Treasure token."
    supported("Patron of the Arts");
    let mut t = TestGame::new(2);
    let p = etb(&mut t, P0, "Patron of the Arts");
    let tr = tokens(&t, P0, "Treasure");
    assert_eq!(tr.len(), 1);
    let c = &t.obj_now(tr[0]).chars;
    assert!(c.card_types.contains(CardType::Artifact) && c.colors.is_colorless());
    kill(&mut t, p);
    assert_eq!(tokens(&t, P0, "Treasure").len(), 2);
    assert_eq!(tokens(&t, P1, "Treasure").len(), 0);
    // Bounced: no Treasure.
    let mut t = TestGame::new(2);
    let p = t.battlefield(P0, "Patron of the Arts");
    t.g.move_object(p, Zone::Hand(P0), MoveCause::Effect, None);
    t.resolve_all();
    assert_eq!(tokens(&t, P0, "Treasure").len(), 0);
}

// ---------------------------------------------------------------------------
// Spells cast, cards drawn and discarded
// ---------------------------------------------------------------------------

#[test]
fn frenzied_devils_grows_when_you_cast_a_noncreature_spell() {
    cr!("601.2i", "603.2", "611.2a", "514.2");
    // "Whenever you cast a noncreature spell, this creature gets +2/+2 until end of turn."
    supported("Frenzied Devils");
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Frenzied Devils");
    let theirs = t.battlefield(P1, "Frenzied Devils");
    bolt(&mut t, P0, Entity::Player(P1));
    assert_eq!(t.pt(d), (5, 5));
    assert_eq!(t.pt(theirs), (3, 3), "only your spells");
    cast_card(&mut t, P0, "Raging Goblin", "Mountain", 1, &[]);
    assert_eq!(t.pt(d), (5, 5), "not a creature spell");
    next_turn(&mut t);
    assert_eq!(t.pt(d), (3, 3));
}

#[test]
fn geistflame_reservoir_counts_instants_and_sorceries() {
    cr!("601.2i", "603.2", "122.1");
    // "Whenever you cast an instant or sorcery spell, put a charge counter on this
    // artifact."
    supported("Geistflame Reservoir");
    let mut t = TestGame::new(2);
    let r = t.battlefield(P0, "Geistflame Reservoir");
    bolt(&mut t, P0, Entity::Player(P1));
    assert_eq!(t.counters(r, counters::CHARGE), 1, "an instant");
    cast_card(&mut t, P0, "Divination", "Island", 3, &[]);
    assert_eq!(t.counters(r, counters::CHARGE), 2, "a sorcery");
    cast_card(&mut t, P0, "Mind Stone", "Island", 2, &[]);
    cast_card(&mut t, P0, "Raging Goblin", "Mountain", 1, &[]);
    assert_eq!(t.counters(r, counters::CHARGE), 2, "an artifact and a creature");
    t.set_step(P1, Step::PrecombatMain);
    bolt(&mut t, P1, Entity::Player(P0));
    assert_eq!(t.counters(r, counters::CHARGE), 2, "an opponent's");
}

#[test]
fn kindlespark_duo_untaps_when_you_cast_a_noncreature_spell() {
    cr!("601.2i", "603.2", "701.26b");
    // "{T}: This creature deals 1 damage to target opponent. / Whenever you cast a
    // noncreature spell, untap this creature."
    supported("Kindlespark Duo");
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Kindlespark Duo");
    let other = t.battlefield(P0, "Llanowar Elves");
    t.g.tap(other);
    act(&mut t, P0, d, "damage", &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    assert!(t.obj_now(d).tapped);
    cast_card(&mut t, P0, "Raging Goblin", "Mountain", 1, &[]);
    assert!(t.obj_now(d).tapped, "a creature spell");
    bolt(&mut t, P0, Entity::Player(P1));
    assert!(!t.obj_now(d).tapped);
    assert!(t.obj_now(other).tapped, "only this creature");
    act(&mut t, P0, d, "damage", &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 20 - 1 - 3 - 1);
}

#[test]
fn thunder_drake_grows_on_your_second_spell_each_turn() {
    cr!("601.2i", "603.2", "122.1");
    // "Whenever you cast your second spell each turn, put a +1/+1 counter on this
    // creature."
    supported("Thunder Drake");
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Thunder Drake");
    bolt(&mut t, P0, Entity::Player(P1));
    assert_eq!(t.counters(d, counters::PLUS1), 0, "the first spell");
    cast_card(&mut t, P0, "Raging Goblin", "Mountain", 1, &[]);
    assert_eq!(t.counters(d, counters::PLUS1), 1, "the second (a creature spell)");
    bolt(&mut t, P0, Entity::Player(P1));
    assert_eq!(t.counters(d, counters::PLUS1), 1, "not the third");
    // Each turn: on the opponent's turn, your second instant counts again.
    next_turn(&mut t);
    bolt(&mut t, P1, Entity::Player(P0));
    assert_eq!(t.counters(d, counters::PLUS1), 1, "an opponent's spell");
    bolt(&mut t, P0, Entity::Player(P1));
    assert_eq!(t.counters(d, counters::PLUS1), 1);
    bolt(&mut t, P0, Entity::Player(P1));
    assert_eq!(t.counters(d, counters::PLUS1), 2);
}

#[test]
fn gixian_puppeteer_drains_on_your_second_draw_each_turn() {
    cr!("603.2", "121.2", "119.3");
    // "Whenever you draw your second card each turn, each opponent loses 2 life and you
    // gain 2 life."
    supported("Gixian Puppeteer");
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Gixian Puppeteer");
    t.g.draw_cards(P0, 1);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1), t.life(P2)), (20, 20, 20), "the first card");
    t.g.draw_cards(P1, 1);
    t.g.draw_cards(P1, 1);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1), t.life(P2)), (20, 20, 20), "an opponent's draws");
    t.g.draw_cards(P0, 1);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1), t.life(P2)), (22, 18, 18));
    t.g.draw_cards(P0, 2);
    t.resolve_all();
    assert_eq!(t.life(P0), 22, "not the third or fourth");
    // Next turn it counts again (drawing during an opponent's turn).
    t.advance_to(P1, Step::PrecombatMain);
    t.g.draw_cards(P0, 2);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1), t.life(P2)), (24, 16, 16));
}

#[test]
fn silvergill_peddler_loots_when_it_becomes_tapped() {
    cr!("603.2", "701.26a", "121.1", "701.9a");
    // "Whenever this creature becomes tapped, draw a card, then discard a card."
    supported("Silvergill Peddler");
    let mut t = TestGame::new(2);
    let p = t.battlefield(P0, "Silvergill Peddler");
    let other = t.battlefield(P0, "Grizzly Bears");
    let forest = t.hand(P0, "Forest");
    t.g.tap(other);
    t.resolve_all();
    assert_eq!(t.library_size(P0), 30, "another creature");
    t.answer_choose(P0, &[obj(forest)]);
    t.g.tap(p);
    t.resolve_all();
    assert_eq!(t.library_size(P0), 29);
    assert_eq!(t.hand_size(P0), 1);
    assert_eq!(t.zone(forest), Zone::Graveyard(P0));
    // Already tapped: it can't become tapped again.
    t.g.tap(p);
    t.resolve_all();
    assert_eq!(t.library_size(P0), 29);
}

#[test]
fn hobgoblin_gets_plus_2_when_you_discard() {
    cr!("603.2", "701.9a", "611.2a", "514.2");
    // "Whenever you discard a card, Hobgoblin gets +2/+0 until end of turn."
    supported("Hobgoblin, Mantled Marauder");
    let mut t = TestGame::new(2);
    let h = t.battlefield(P0, "Hobgoblin, Mantled Marauder");
    let a = t.hand(P0, "Forest");
    let b = t.hand(P0, "Forest");
    let theirs = t.hand(P1, "Forest");
    t.g.discard(P1, theirs, None);
    t.resolve_all();
    assert_eq!(t.pt(h), (1, 2), "an opponent's discard");
    t.g.discard(P0, a, None);
    t.resolve_all();
    assert_eq!(t.pt(h), (3, 2));
    t.g.discard(P0, b, None);
    t.resolve_all();
    assert_eq!(t.pt(h), (5, 2), "each card");
    next_turn(&mut t);
    assert_eq!(t.pt(h), (1, 2));
}

#[test]
fn lilianas_caress_drains_an_opponent_who_discards() {
    cr!("603.2", "701.9a", "119.3");
    // "Whenever an opponent discards a card, that player loses 2 life."
    supported("Liliana's Caress");
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Liliana's Caress");
    let a = t.hand(P2, "Forest");
    let b = t.hand(P2, "Forest");
    let mine = t.hand(P0, "Forest");
    t.g.discard(P2, a, None);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1), t.life(P2)), (20, 20, 18));
    t.g.discard(P0, mine, None);
    t.resolve_all();
    assert_eq!(t.life(P0), 20, "not your own discard");
    // Mind Rot: two cards, two triggers.
    let c = t.hand(P1, "Forest");
    let d = t.hand(P1, "Forest");
    cast_card(&mut t, P0, "Mind Rot", "Swamp", 3, &[Entity::Player(P1)]);
    assert_eq!(t.zone(c), Zone::Graveyard(P1));
    assert_eq!(t.zone(d), Zone::Graveyard(P1));
    assert_eq!(t.life(P1), 16);
    assert_eq!(t.zone(b), Zone::Hand(P2));
}

#[test]
fn razormane_masticore_is_sacrificed_unless_you_discard() {
    cr!("503.1a", "603.2", "118.12a", "701.9a", "701.21a");
    // "At the beginning of your upkeep, sacrifice this creature unless you discard a
    // card."
    supported("Razormane Masticore");
    // Discarding keeps it. (The "yes" is queued before the opponent's upkeep, which
    // doesn't trigger it.)
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Razormane Masticore");
    let forest = t.hand(P0, "Forest");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(forest)]);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert!(t.on_battlefield(m));
    assert_eq!(t.zone(forest), Zone::Graveyard(P0));
    // Declining sacrifices it.
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Razormane Masticore");
    let forest = t.hand(P0, "Forest");
    t.answer_yes(P0, false);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert!(!t.on_battlefield(m));
    assert_eq!(t.zone(forest), Zone::Hand(P0));
    // With no card to discard, it's sacrificed.
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Razormane Masticore");
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert!(!t.on_battlefield(m));
}

// ---------------------------------------------------------------------------
// Other events
// ---------------------------------------------------------------------------

#[test]
fn verdant_force_makes_a_saproling_each_upkeep() {
    cr!("503.1a", "603.2", "111.1");
    // "At the beginning of each upkeep, create a 1/1 green Saproling creature token."
    supported("Verdant Force");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Verdant Force");
    assert_eq!(tokens(&t, P0, "Saproling").len(), 0);
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    let s = tokens(&t, P0, "Saproling");
    assert_eq!(s.len(), 1, "the opponent's upkeep");
    assert_eq!(t.pt(s[0]), (1, 1));
    assert!(t.obj_now(s[0]).chars.colors.contains(Color::Green));
    assert_eq!(tokens(&t, P1, "Saproling").len(), 0, "its controller creates it");
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(tokens(&t, P0, "Saproling").len(), 2, "your upkeep");
}

#[test]
fn kor_entanglers_taps_an_opponents_creature_when_an_ally_enters() {
    cr!("603.2", "603.6a", "115.1", "701.26a");
    // "Rally — Whenever this creature or another Ally you control enters, tap target
    // creature an opponent controls."
    supported("Kor Entanglers");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    let forest = t.battlefield(P1, "Forest");
    let mine = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[obj(a)]);
    let k = etb(&mut t, P0, "Kor Entanglers");
    assert!(t.obj_now(a).tapped && !t.obj_now(b).tapped);
    let offered = &target_choices(&t, P0)[0];
    assert!(offered.contains(&obj(b)));
    assert!(!offered.contains(&obj(forest)) && !offered.contains(&obj(mine)));
    assert!(!offered.contains(&obj(k)));
    // Another Ally you control.
    t.answer_targets(P0, &[obj(b)]);
    etb(&mut t, P0, "Makindi Patrol");
    assert!(t.obj_now(b).tapped);
    assert_eq!(target_choices(&t, P0).len(), 2);
    // A non-Ally, or an opponent's Ally, doesn't trigger it.
    etb(&mut t, P0, "Grizzly Bears");
    etb(&mut t, P1, "Makindi Patrol");
    assert_eq!(target_choices(&t, P0).len(), 2);
}

#[test]
fn necrosquito_gets_an_oil_counter_when_your_creature_or_artifact_dies() {
    cr!("603.2", "603.6c", "603.10a", "122.1", "614.1c");
    // "This creature enters with two oil counters on it. / This creature gets +1/+1 for
    // each oil counter on it. / Whenever another creature or artifact you control is put
    // into a graveyard from the battlefield, put an oil counter on this creature."
    supported("Necrosquito");
    let mut t = TestGame::new(2);
    let n = etb(&mut t, P0, "Necrosquito");
    assert_eq!(t.counters(n, counters::OIL), 2);
    assert_eq!(t.pt(n), (2, 2));
    let bears = t.battlefield(P0, "Grizzly Bears");
    let stone = t.battlefield(P0, "Mind Stone");
    let forest = t.battlefield(P0, "Forest");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let exiled = t.battlefield(P0, "Grizzly Bears");
    kill(&mut t, bears);
    assert_eq!(t.counters(n, counters::OIL), 3, "a creature");
    kill(&mut t, stone);
    assert_eq!(t.counters(n, counters::OIL), 4, "an artifact");
    kill(&mut t, forest);
    kill(&mut t, theirs);
    t.g.exile_object(exiled, None);
    t.resolve_all();
    assert_eq!(t.counters(n, counters::OIL), 4, "a land, an opponent's, exiled");
    assert_eq!(t.pt(n), (4, 4));
}

#[test]
fn roil_cartographer_gets_energy_when_a_land_you_control_enters() {
    cr!("603.2", "603.6a", "122.1");
    // "Landfall — Whenever a land you control enters, you get {E}."
    supported("Roil Cartographer");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Roil Cartographer");
    let land = t.hand(P0, "Island");
    t.play_land(P0, land).unwrap();
    t.resolve_all();
    assert_eq!(t.g.player(P0).counter(counters::ENERGY), 1);
    etb(&mut t, P1, "Forest");
    etb(&mut t, P0, "Grizzly Bears");
    assert_eq!(t.g.player(P0).counter(counters::ENERGY), 1, "an opponent's land, a creature");
    assert_eq!(t.g.player(P1).counter(counters::ENERGY), 0);
    etb(&mut t, P0, "Forest");
    assert_eq!(t.g.player(P0).counter(counters::ENERGY), 2);
}

#[test]
fn paradox_shaper_becomes_prepared_at_your_upkeep_if_it_isnt() {
    cr!("503.1a", "603.4", "722.3a", "722.3c");
    // "At the beginning of your upkeep, if this creature isn't prepared, it becomes
    // prepared."
    supported("Paradox Shaper // Omit Variables");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Paradox Shaper // Omit Variables");
    assert!(t.obj_now(s).prepared.is_none());
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert!(t.obj_now(s).prepared.is_none(), "not the opponent's upkeep");
    t.advance_to(P0, Step::Upkeep);
    // Upkeep triggers wait to be put on the stack when a player would receive priority.
    let queued = |t: &TestGame| t.stack_len() + t.g.pending_triggers.len();
    assert_eq!(queued(&t), 1, "it triggered at your upkeep");
    t.resolve_all();
    let copy = t.obj_now(s).prepared.expect("prepared");
    assert_eq!(t.obj(copy).zone, Zone::Exile);
    assert_eq!(t.obj(copy).chars.name, "Omit Variables");
    // Already prepared: the ability doesn't trigger (no second copy).
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    assert_eq!(queued(&t), 0, "nothing triggered");
    t.resolve_all();
    assert_eq!(t.obj_now(s).prepared, Some(copy));
    let copies = t.g.exile.iter().filter(|c| t.obj(**c).chars.name == "Omit Variables").count();
    assert_eq!(copies, 1);
}

#[test]
fn wall_of_hope_gains_life_equal_to_the_damage_dealt_to_it() {
    cr!("603.2", "120.3e", "119.3");
    // "Whenever this creature is dealt damage, you gain that much life."
    supported("Wall of Hope");
    let mut t = TestGame::new(2);
    let w = t.battlefield(P0, "Wall of Hope");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Wall of Hope");
    cast_card(&mut t, P0, "Shock", "Mountain", 1, &[obj(w)]);
    assert_eq!(t.obj_now(w).damage, 2);
    assert_eq!((t.life(P0), t.life(P1)), (22, 20));
    bolt(&mut t, P0, obj(bears));
    assert_eq!(t.life(P0), 22, "another creature");
    // Lethal damage still counts (it triggered before the Wall died).
    bolt(&mut t, P0, obj(w));
    assert!(!t.on_battlefield(w));
    assert_eq!(t.life(P0), 25);
    // The opponent's Wall gains its controller life.
    bolt(&mut t, P0, obj(theirs));
    assert_eq!((t.life(P0), t.life(P1)), (25, 23));
}
