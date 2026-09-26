//! "Prevent all [combat | noncombat] damage" by and to groups of objects and players
//! (patterns in `src/oracle/patterns/replacements_prevent_groups.rs`).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn assert_compiles(names: &[&str]) {
    for n in names {
        let u = card(n).unsupported_text().join(" | ");
        assert!(u.is_empty(), "{n} has unsupported text: {u}");
    }
}

#[test]
fn prevention_group_cards_compile() {
    assert_compiles(&[
        "Divine Light",
        "Thwart the Enemy",
        "Deep Wood",
        "Harmless Assault",
        "Vine Snare",
        "Tanglesap",
        "Galadhrim Ambush",
        "Safe Passage",
        "Endure",
        "Eerie Interference",
        "Repel the Abominable",
        "Luminesce",
        "Chameleon Blur",
        "Haze Frog",
        "Frontline Strategist",
        "Al-abara's Carpet",
        "Scarecrow",
        "Radiant Kavu",
        "Pack Leader",
        "Crystal Barricade",
        "Mark of Asylum",
        "Drogskol Reinforcements",
        "Magebane Armor",
        "Blessed Sanctuary",
        "The Wanderer",
        "Energy Field",
        "Argothian Treefolk",
        "Artifact Ward",
        "Light of Sanction",
        "Indentured Oaf",
        "Goblin Furrier",
        "Armored Transport",
        "Personal Sanctuary",
        "Fog of War",
        "Songstitcher",
        "Stonewise Fortifier",
        "Guardian Naga // Banishing Coils",
    ]);
}

fn cast_resolve(t: &mut TestGame, p: PlayerId, name: &str, land: &str, n: usize) {
    t.lands(p, land, n);
    let c = t.hand(p, name);
    t.cast(p, c).go();
    t.resolve();
}

#[test]
fn divine_light_protects_your_creatures_even_ones_that_arrive_later() {
    cr!("615.1a", "611.2c");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    cast_resolve(&mut t, P0, "Divine Light", "Plains", 1);
    t.g.deal_damage(giant, Entity::Object(bears), 3, false);
    assert!(t.on_battlefield(bears));
    assert_eq!(t.obj_now(bears).damage, 0);
    // A creature that arrives afterward is protected too (a prevention effect isn't
    // locked to the objects there as it resolved).
    let late = t.battlefield(P0, "Grizzly Bears");
    t.g.deal_damage(giant, Entity::Object(late), 3, true);
    assert!(t.on_battlefield(late));
    // Damage to P0 and to P1's creatures isn't prevented.
    t.g.deal_damage(giant, Entity::Player(P0), 3, true);
    assert_eq!(t.life(P0), 17);
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.g.deal_damage(giant, Entity::Object(theirs), 1, false);
    assert_eq!(t.obj_now(theirs).damage, 1);
    // It ends with the turn.
    t.advance_to(P1, Step::Upkeep);
    t.g.deal_damage(giant, Entity::Object(bears), 1, false);
    assert_eq!(t.obj_now(bears).damage, 1);
}

#[test]
fn safe_passage_protects_you_and_your_creatures_but_not_planeswalkers() {
    cr!("615.1a");
    ruling!(
        "Safe Passage",
        "Safe Passage doesn’t prevent damage that would be dealt to planeswalkers you control."
    );
    ruling!(
        "Safe Passage",
        "Safe Passage prevents all damage, not just combat damage, that would be dealt to you and creatures you control this turn."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let jace = t.battlefield(P0, "Jace Beleren");
    let loyalty = t.counters(jace, "loyalty");
    let giant = t.battlefield(P1, "Hill Giant");
    cast_resolve(&mut t, P0, "Safe Passage", "Plains", 3);
    t.g.deal_damage(giant, Entity::Player(P0), 3, true);
    t.g.deal_damage(giant, Entity::Object(bears), 3, false);
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.obj_now(bears).damage, 0);
    t.g.deal_damage(giant, Entity::Object(jace), 1, false);
    assert_eq!(t.counters(jace, "loyalty"), loyalty - 1);
}

#[test]
fn harmless_assault_prevents_combat_damage_by_attackers_only() {
    cr!("615.1a");
    ruling!(
        "Harmless Assault",
        "Combat damage dealt by blocking creatures isn’t prevented."
    );
    let mut t = TestGame::new(2);
    let attacker = t.battlefield(P1, "Hill Giant");
    let blocker = t.battlefield(P0, "Craw Wurm");
    let unblocked = t.battlefield(P1, "Grizzly Bears");
    cast_resolve(&mut t, P0, "Harmless Assault", "Plains", 4);
    t.set_step(P1, Step::BeginningOfCombat);
    t.attack(
        &[
            (attacker, Entity::Player(P0)),
            (unblocked, Entity::Player(P0)),
        ],
        &[(blocker, attacker)],
    );
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.obj_now(blocker).damage, 0);
    // The Wurm's 6 combat damage to the attacking Giant isn't prevented.
    assert!(!t.on_battlefield(attacker));
}

#[test]
fn vine_snare_checks_power_as_the_damage_would_be_dealt() {
    cr!("615.1a");
    ruling!(
        "Vine Snare",
        "Check the power of each creature as it would deal combat damage to determine if that damage is prevented."
    );
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    cast_resolve(&mut t, P0, "Vine Snare", "Forest", 3);
    t.g.deal_damage(giant, Entity::Player(P0), 3, true);
    assert_eq!(t.life(P0), 20);
    // Noncombat damage isn't prevented.
    t.g.deal_damage(giant, Entity::Player(P0), 3, false);
    assert_eq!(t.life(P0), 17);
    // Pumped to 6/6 after Vine Snare resolved: no longer prevented.
    t.lands(P1, "Forest", 1);
    let growth = t.hand(P1, "Giant Growth");
    t.cast(P1, growth).target(giant).go();
    t.resolve();
    t.g.deal_damage(giant, Entity::Player(P0), 6, true);
    assert_eq!(t.life(P0), 11);
}

#[test]
fn haze_frog_prevents_combat_damage_by_other_creatures_including_later_ones() {
    cr!("615.1a", "611.2c");
    ruling!(
        "Haze Frog",
        "Combat damage dealt by Haze Frog itself during the turn it enters isn't prevented."
    );
    ruling!(
        "Haze Frog",
        "If two Haze Frogs enter during the same turn, each one's ability will prevent the other one's combat damage."
    );
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let frog = t.enter(P0, "Haze Frog");
    t.resolve_all();
    t.g.deal_damage(giant, Entity::Player(P0), 3, true);
    assert_eq!(t.life(P0), 20);
    t.g.deal_damage(frog, Entity::Player(P1), 2, true);
    assert_eq!(t.life(P1), 18);
    let late = t.battlefield(P1, "Grizzly Bears");
    t.g.deal_damage(late, Entity::Player(P0), 2, true);
    assert_eq!(t.life(P0), 20);
    // A second Haze Frog's ability prevents the first one's combat damage.
    let frog2 = t.enter(P1, "Haze Frog");
    t.resolve_all();
    t.g.deal_damage(frog, Entity::Player(P1), 2, true);
    assert_eq!(t.life(P1), 18);
    t.g.deal_damage(frog2, Entity::Player(P0), 2, true);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn mark_of_asylum_prevents_only_noncombat_damage() {
    cr!("615.1a", "120.2b");
    ruling!(
        "Mark of Asylum",
        "“Noncombat damage” is damage dealt as the result of a spell or ability."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mark of Asylum");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(bears).go();
    t.resolve();
    assert!(t.on_battlefield(bears));
    assert_eq!(t.obj_now(bears).damage, 0);
    let giant = t.battlefield(P1, "Hill Giant");
    t.g.deal_damage(giant, Entity::Object(bears), 1, true);
    assert_eq!(t.obj_now(bears).damage, 1);
}

#[test]
fn the_wanderer_protects_you_and_other_permanents_but_not_itself() {
    cr!("615.1a", "120.2b");
    let mut t = TestGame::new(2);
    let w = t.battlefield(P0, "The Wanderer");
    let loyalty = t.counters(w, "loyalty");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.g.deal_damage(giant, Entity::Player(P0), 3, false);
    t.g.deal_damage(giant, Entity::Object(bears), 3, false);
    assert_eq!(t.life(P0), 20);
    assert!(t.on_battlefield(bears));
    t.g.deal_damage(giant, Entity::Object(w), 1, false);
    assert_eq!(t.counters(w, "loyalty"), loyalty - 1);
    // Combat damage isn't prevented.
    t.g.deal_damage(giant, Entity::Player(P0), 3, true);
    assert_eq!(t.life(P0), 17);
}

#[test]
fn energy_field_prevents_damage_from_sources_you_dont_control() {
    cr!("615.1a");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Energy Field");
    let theirs = t.battlefield(P1, "Hill Giant");
    let mine = t.battlefield(P0, "Hill Giant");
    t.g.deal_damage(theirs, Entity::Player(P0), 3, true);
    assert_eq!(t.life(P0), 20);
    t.g.deal_damage(mine, Entity::Player(P0), 3, false);
    assert_eq!(t.life(P0), 17);
}

#[test]
fn argothian_treefolk_prevents_damage_from_artifact_sources() {
    cr!("615.1a");
    let mut t = TestGame::new(2);
    let tree = t.battlefield(P0, "Argothian Treefolk");
    let thopter = t.battlefield(P1, "Ornithopter");
    let giant = t.battlefield(P1, "Hill Giant");
    t.g.deal_damage(thopter, Entity::Object(tree), 4, true);
    assert_eq!(t.obj_now(tree).damage, 0);
    t.g.deal_damage(giant, Entity::Object(tree), 3, true);
    assert_eq!(t.obj_now(tree).damage, 3);
}

#[test]
fn personal_sanctuary_works_only_during_your_turn() {
    cr!("615.1a");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Personal Sanctuary");
    let giant = t.battlefield(P1, "Hill Giant");
    t.set_step(P0, Step::PrecombatMain);
    t.g.deal_damage(giant, Entity::Player(P0), 3, false);
    assert_eq!(t.life(P0), 20);
    t.set_step(P1, Step::PrecombatMain);
    t.g.deal_damage(giant, Entity::Player(P0), 3, false);
    assert_eq!(t.life(P0), 17);
}

#[test]
fn luminesce_prevents_damage_from_black_and_red_sources() {
    cr!("615.1a");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_resolve(&mut t, P0, "Luminesce", "Plains", 1);
    t.g.deal_damage(giant, Entity::Player(P0), 3, true);
    assert_eq!(t.life(P0), 20);
    t.g.deal_damage(bears, Entity::Player(P0), 2, true);
    assert_eq!(t.life(P0), 18);
}

#[test]
fn chameleon_blur_prevents_creature_damage_to_players_only() {
    cr!("615.1a");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_resolve(&mut t, P0, "Chameleon Blur", "Forest", 4);
    t.g.deal_damage(giant, Entity::Player(P0), 3, true);
    t.g.deal_damage(bears, Entity::Player(P1), 2, true);
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.life(P1), 20);
    t.g.deal_damage(giant, Entity::Object(bears), 1, true);
    assert_eq!(t.obj_now(bears).damage, 1);
}

#[test]
fn armored_transport_prevents_combat_damage_from_its_blockers() {
    cr!("615.1a");
    let mut t = TestGame::new(2);
    let transport = t.battlefield(P0, "Armored Transport");
    let blocker = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(transport, Entity::Player(P1))], &[(blocker, transport)]);
    assert!(t.on_battlefield(transport));
    assert_eq!(t.obj_now(transport).damage, 0);
    // Its 2 damage killed the blocker.
    assert!(!t.on_battlefield(blocker));
    // Other damage to it isn't prevented.
    let giant = t.battlefield(P1, "Hill Giant");
    t.g.deal_damage(giant, Entity::Object(transport), 1, true);
    assert_eq!(t.obj_now(transport).damage, 1);
}

#[test]
fn stonewise_fortifier_prevents_damage_from_the_target_creature() {
    cr!("615.1a", "115.1");
    let mut t = TestGame::new(2);
    let fort = t.battlefield(P0, "Stonewise Fortifier");
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Plains", 5);
    t.activate(P0, fort, 0, &[Entity::Object(giant)]).unwrap();
    t.resolve();
    t.g.deal_damage(giant, Entity::Object(fort), 3, true);
    assert!(t.on_battlefield(fort));
    assert_eq!(t.obj_now(fort).damage, 0);
    t.g.deal_damage(bears, Entity::Object(fort), 1, true);
    assert_eq!(t.obj_now(fort).damage, 1);
}
