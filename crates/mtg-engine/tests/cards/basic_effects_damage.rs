//! Damage effects: subjects ("it", a target creature, a legendary card's short name), amounts
//! ("that much", "damage equal to its power", "X plus 2"), and recipients ("each other
//! opponent", "any target that isn't a Dinosaur", "enchanted artifact's controller", "each
//! creature dealt damage this turn"); plus destroy/exile qualifiers about this turn
//! ("that dealt damage this turn", "that blocked this turn") and "except for".

use crate::basic_effects_common::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// P1 Shocks `target` (2 damage).
fn shock(t: &mut TestGame, target: ObjectId) {
    t.lands(P1, "Mountain", 1);
    let s = t.hand(P1, "Shock");
    t.cast(P1, s).target(target).go();
    t.resolve_all();
}

#[test]
fn hydra_omnivore_deals_that_much_damage_to_each_other_opponent() {
    cr!("120.3", "510.2");
    assert_supported("Hydra Omnivore");
    let mut t = TestGame::new(3);
    let hydra = t.battlefield(P0, "Hydra Omnivore");
    t.advance_to(P0, Step::BeginningOfCombat);
    t.attack(&[(hydra, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 12);
    assert_eq!(t.life(P2), 12);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn wormwood_dryad_deals_damage_to_you() {
    cr!("120.3");
    assert_supported("Wormwood Dryad");
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Wormwood Dryad");
    t.lands(P0, "Forest", 1);
    t.activate(P0, d, 0, &[]).unwrap();
    t.resolve();
    assert_eq!(t.life(P0), 19);
}

#[test]
fn rakdos_charm_each_creature_damages_its_own_controller() {
    cr!("120.2", "120.3");
    assert_supported("Rakdos Charm");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Mountain", 1);
    let rc = t.hand(P0, "Rakdos Charm");
    t.cast(P0, rc).modes(&[2]).go();
    t.resolve();
    assert_eq!(t.life(P0), 19);
    assert_eq!(t.life(P1), 18);
}

#[test]
fn mutiny_one_opponents_creature_damages_another_of_theirs() {
    cr!("120.3");
    assert_supported("Mutiny");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 1);
    let m = t.hand(P0, "Mutiny");
    t.cast(P0, m).target(giant).target(bears).go();
    t.resolve();
    assert!(!t.on_battlefield(bears));
    assert!(t.on_battlefield(giant));
}

#[test]
fn wrathful_raptors_cant_target_a_dinosaur() {
    cr!("115.1");
    ruling!(
        "Wayta, Trainer Prodigy",
        "such as the ability granted by Mephidross Vampire or the last ability of Wrathful Raptors"
    );
    assert_supported("Wrathful Raptors");
    let mut t = TestGame::new(2);
    let raptors = t.battlefield(P0, "Wrathful Raptors");
    t.battlefield(P0, "Wayta, Trainer Prodigy");
    let bears = t.battlefield(P1, "Grizzly Bears");
    // Wayta: the Raptors' ability triggers twice.
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    shock(&mut t, raptors);
    assert!(!last_target_candidates(&t, P0).contains(&Entity::Object(raptors)));
    assert!(!t.on_battlefield(bears));
    assert_eq!(t.life(P1), 18);
}

#[test]
fn needle_drop_and_inflame_look_at_damage_dealt_this_turn() {
    cr!("120.3");
    assert_supported("Needle Drop");
    assert_supported("Inflame");
    let mut t = TestGame::new(2);
    let hurt = t.battlefield(P0, "Hill Giant");
    let fresh = t.battlefield(P0, "Hill Giant");
    shock(&mut t, hurt);
    t.lands(P1, "Mountain", 1);
    let nd = t.hand(P1, "Needle Drop");
    t.cast(P1, nd).target(hurt).go();
    let cands = last_target_candidates(&t, P1);
    assert!(cands.contains(&Entity::Object(hurt)));
    assert!(!cands.contains(&Entity::Object(fresh)));
    assert!(!cands.contains(&Entity::Player(P0)));
    t.resolve();
    assert!(!t.on_battlefield(hurt));

    let mut t = TestGame::new(2);
    let fresh = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    shock(&mut t, giant);
    t.lands(P1, "Mountain", 1);
    let inf = t.hand(P1, "Inflame");
    t.cast(P1, inf).go();
    t.resolve();
    assert!(!t.on_battlefield(giant));
    assert!(t.on_battlefield(fresh));
    assert_eq!(t.obj_now(fresh).damage, 0);
}

#[test]
fn flame_sweep_spares_your_fliers() {
    cr!("120.3");
    assert_supported("Flame Sweep");
    let mut t = TestGame::new(2);
    let my_flier = t.battlefield(P0, "Wind Drake");
    let my_bears = t.battlefield(P0, "Grizzly Bears");
    let their_flier = t.battlefield(P1, "Wind Drake");
    t.lands(P0, "Mountain", 3);
    let fs = t.hand(P0, "Flame Sweep");
    t.cast(P0, fs).go();
    t.resolve();
    assert!(t.on_battlefield(my_flier));
    assert!(!t.on_battlefield(my_bears));
    assert!(!t.on_battlefield(their_flier));
}

#[test]
fn let_the_galaxy_burn_spares_creatures_that_entered_this_turn() {
    cr!("107.3", "120.3");
    let mut t = TestGame::new(2);
    let old = t.battlefield(P1, "Hill Giant");
    t.g.objects[old.0 as usize].entered_turn = 0;
    let new = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Mountain", 6);
    let s = t.hand(P0, "Let the Galaxy Burn");
    t.cast(P0, s).x(0).go();
    t.resolve_all();
    // X plus 2 = 2 damage: not enough for a Giant, so check the damage marked.
    assert_eq!(t.obj_now(old).damage, 2);
    assert_eq!(t.obj_now(new).damage, 0);
}

#[test]
fn avenging_arrow_targets_a_creature_that_dealt_damage_this_turn() {
    cr!("120.3");
    assert_supported("Avenging Arrow");
    let mut t = TestGame::new(2);
    let reckoner = t.battlefield(P1, "Prodigal Pyromancer");
    let idle = t.battlefield(P1, "Grizzly Bears");
    t.activate(P1, reckoner, 0, &[Entity::Player(P0)]).unwrap();
    t.resolve();
    t.lands(P0, "Plains", 3);
    let aa = t.hand(P0, "Avenging Arrow");
    t.cast(P0, aa).target(reckoner).go();
    let cands = last_target_candidates(&t, P0);
    assert!(!cands.contains(&Entity::Object(idle)));
    t.resolve();
    assert!(!t.on_battlefield(reckoner));
}

#[test]
fn blocked_this_turn_qualifiers() {
    cr!("509.1");
    assert_supported("Sizzling Barrage");
    assert_supported("You Cannot Pass!");
    assert_supported("Heat Stroke");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Heat Stroke");
    let attacker = t.battlefield(P0, "Grizzly Bears");
    let blocker = t.battlefield(P1, "Wall of Stone");
    let other = t.battlefield(P1, "Grizzly Bears");
    t.advance_to(P0, Step::BeginningOfCombat);
    t.attack(&[(attacker, Entity::Player(P1))], &[(blocker, attacker)]);
    // Heat Stroke destroys both at end of combat; the other Bears stay.
    t.resolve_all();
    assert!(!t.on_battlefield(attacker));
    assert!(!t.on_battlefield(blocker));
    assert!(t.on_battlefield(other));
}

#[test]
fn sizzling_barrage_needs_a_creature_that_blocked() {
    cr!("509.1");
    let mut t = TestGame::new(2);
    let attacker = t.battlefield(P0, "Hill Giant");
    let blocker = t.battlefield(P1, "Wall of Stone");
    let idle = t.battlefield(P1, "Grizzly Bears");
    t.advance_to(P0, Step::BeginningOfCombat);
    t.attack(&[(attacker, Entity::Player(P1))], &[(blocker, attacker)]);
    t.lands(P0, "Mountain", 2);
    let sb = t.hand(P0, "Sizzling Barrage");
    t.cast(P0, sb).target(blocker).go();
    let cands = last_target_candidates(&t, P0);
    assert!(!cands.contains(&Entity::Object(idle)));
    assert!(!cands.contains(&Entity::Object(attacker)));
    t.resolve();
    assert_eq!(t.obj_now(blocker).damage, 3 + 4);
}

#[test]
fn you_cannot_pass_needs_a_legendary_partner() {
    cr!("509.1");
    let mut t = TestGame::new(2);
    let legend = t.battlefield(P0, "Isamaru, Hound of Konda");
    let blocker = t.battlefield(P1, "Wall of Wood");
    t.advance_to(P0, Step::BeginningOfCombat);
    t.attack(&[(legend, Entity::Player(P1))], &[(blocker, legend)]);
    t.lands(P0, "Plains", 1);
    let ycp = t.hand(P0, "You Cannot Pass!");
    t.cast(P0, ycp).target(blocker).go();
    let cands = last_target_candidates(&t, P0);
    // The wall blocked a legendary creature; Isamaru was blocked by a nonlegendary one.
    assert!(cands.contains(&Entity::Object(blocker)));
    assert!(!cands.contains(&Entity::Object(legend)));
}

#[test]
fn scourglass_destroys_all_but_artifacts_and_lands() {
    cr!("701.8a");
    assert_supported("Scourglass");
    let mut t = TestGame::new(2);
    let sg = t.battlefield(P0, "Scourglass");
    let orni = t.battlefield(P1, "Ornithopter");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let land = t.battlefield(P1, "Forest");
    t.advance_to(P0, Step::Upkeep);
    t.activate(P0, sg, 0, &[]).unwrap();
    t.resolve();
    assert!(t.on_battlefield(orni) && t.on_battlefield(land));
    assert!(!t.on_battlefield(bears));
}

#[test]
fn ember_gale_damages_white_and_or_blue_creatures_of_that_player() {
    cr!("120.3");
    assert_supported("Ember Gale");
    let mut t = TestGame::new(2);
    let white = t.battlefield(P1, "Savannah Lions");
    let green = t.battlefield(P1, "Llanowar Elves");
    let mine = t.battlefield(P0, "Savannah Lions");
    t.lands(P0, "Mountain", 4);
    let eg = t.hand(P0, "Ember Gale");
    t.cast(P0, eg).target(P1).go();
    t.resolve();
    assert!(!t.on_battlefield(white));
    assert!(t.on_battlefield(green));
    assert!(t.on_battlefield(mine));
}

#[test]
fn gremlin_infestation_damages_the_enchanted_artifacts_controller() {
    cr!("303.4");
    assert_supported("Gremlin Infestation");
    let mut t = TestGame::new(2);
    let art = t.battlefield(P1, "Ornithopter");
    let aura = t.battlefield(P0, "Gremlin Infestation");
    assert!(t.g.attach(aura, Entity::Object(art)));
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

#[test]
fn refuse_damages_the_spells_controller_by_its_mana_value() {
    cr!("202.3");
    assert_supported("Refuse // Cooperate");
    let mut t = TestGame::new(2);
    t.lands(P1, "Forest", 6);
    let alpha = t.hand(P1, "Briarpack Alpha");
    let spell = t.cast(P1, alpha).target(alpha).go();
    t.lands(P0, "Mountain", 4);
    let r = t.hand(P0, "Refuse // Cooperate");
    t.cast(P0, r)
        .method(mtg_engine::object::CastMethod::Half(0))
        .target(spell)
        .go();
    t.resolve();
    assert_eq!(t.life(P1), 16);
}

#[test]
fn craters_claws_deals_x_plus_two_with_a_big_creature() {
    cr!("107.3");
    assert_supported("Crater's Claws");
    for big in [false, true] {
        let mut t = TestGame::new(2);
        if big {
            t.battlefield(P0, "Hill Giant");
            t.battlefield(P0, "Craw Wurm");
        }
        t.lands(P0, "Mountain", 4);
        let cc = t.hand(P0, "Crater's Claws");
        t.cast(P0, cc).x(3).target(P1).go();
        t.resolve();
        assert_eq!(t.life(P1), if big { 15 } else { 17 }, "{big}");
    }
}

#[test]
fn chocobo_kick_kicked_deals_twice_that_much() {
    cr!("702.33d");
    assert_supported("Chocobo Kick");
    for kicked in [false, true] {
        let mut t = TestGame::new(2);
        let mine = t.battlefield(P0, "Grizzly Bears");
        let theirs = t.battlefield(P1, "Hill Giant");
        t.lands(P0, "Forest", 2);
        if kicked {
            t.battlefield(P0, "Plains");
        }
        let ck = t.hand(P0, "Chocobo Kick");
        t.cast(P0, ck).kicked(kicked).target(mine).target(theirs).go();
        t.resolve();
        // 2 damage, or 4 kicked (the Giant dies).
        assert_eq!(t.on_battlefield(theirs), !kicked, "{kicked}");
    }
}

#[test]
fn triumphant_chomp_uses_the_greater_amount() {
    cr!("208.1");
    assert_supported("Triumphant Chomp");
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.lands(P0, "Mountain", 1);
    let tc = t.hand(P0, "Triumphant Chomp");
    t.cast(P0, tc).target(wurm).go();
    t.resolve();
    // No Dinosaurs: 2 damage.
    assert_eq!(t.obj_now(wurm).damage, 2);
}

#[test]
fn crackle_with_power_deals_five_times_x_to_each_of_up_to_x_targets() {
    cr!("107.3", "115.4");
    assert_supported("Crackle with Power");
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.lands(P0, "Mountain", 5);
    let c = t.hand(P0, "Crackle with Power");
    t.cast(P0, c)
        .x(1)
        .targets(&[Entity::Object(wurm)])
        .go();
    t.resolve();
    assert!(!t.on_battlefield(wurm));
    assert_eq!(t.life(P1), 20);
}

#[test]
fn lava_storm_damages_attackers_or_blockers() {
    cr!("700.2");
    assert_supported("Lava Storm");
    let mut t = TestGame::new(2);
    let attacker = t.battlefield(P0, "Grizzly Bears");
    let blocker = t.battlefield(P1, "Grizzly Bears");
    t.advance_to(P0, Step::BeginningOfCombat);
    t.answer(
        P0,
        DecisionKind::Attackers,
        mtg_engine::decision::Answer::Attackers(vec![(attacker, Entity::Player(P1))]),
    );
    t.answer(
        P1,
        DecisionKind::Blockers,
        mtg_engine::decision::Answer::Blockers(vec![(blocker, attacker)]),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    t.lands(P1, "Mountain", 5);
    let ls = t.hand(P1, "Lava Storm");
    // P1 chooses the attacking creatures.
    t.answer(P1, DecisionKind::Option, mtg_engine::decision::Answer::Index(0));
    t.cast(P1, ls).go();
    t.resolve();
    assert!(!t.on_battlefield(attacker));
    assert!(t.on_battlefield(blocker));
}

#[test]
fn redcap_melee_sacrifices_a_land_for_a_nonred_permanent() {
    cr!("701.21a");
    assert_supported("Redcap Melee");
    for red in [true, false] {
        let mut t = TestGame::new(2);
        let target = t.battlefield(P1, if red { "Raging Goblin" } else { "Grizzly Bears" });
        t.lands(P0, "Mountain", 2);
        let rm = t.hand(P0, "Redcap Melee");
        t.cast(P0, rm).target(target).go();
        t.resolve();
        assert!(!t.on_battlefield(target));
        assert_eq!(t.named_on_battlefield("Mountain").len(), if red { 2 } else { 1 });
    }
}
