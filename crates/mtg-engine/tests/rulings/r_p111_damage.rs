//! Rulings batch P111 — damage: marked damage stays until cleanup, so a toughness bonus
//! lost later in the turn can make it lethal (CR 120.6, 704.5g, 514.2); "whenever a
//! creature is dealt damage" triggers once per event and needs its source to survive to
//! get the counter (CR 120.4, 603.2, 608.2h); "a creature dealt damage by this creature
//! this turn dies" (CR 603.10a); regeneration shields (CR 701.19, 614.8); redirection to a
//! creature that's gone does nothing (CR 614.9).

use crate::r_p009_common::kill;
use crate::r_s01_common::{attack_with, block_and_finish, supported};
use crate::r_s06_common::{activate_containing, damage};
use crate::r_s09_common::to_combat;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::*;

fn mana(t: &mut TestGame, p: PlayerId, tys: &[ManaType]) {
    for ty in tys {
        t.g.players[p.idx()].mana_pool.add_type(*ty, 1);
    }
}

fn marked(t: &TestGame, id: ObjectId) -> u32 {
    t.obj_now(id).damage
}

#[test]
fn damage_marked_earlier_becomes_lethal_when_a_bonus_goes_away() {
    cr!("120.6", "704.5g", "514.2", "613.4c");
    ruling!(
        "Pride Sovereign",
        "Because damage remains marked on a creature until it’s removed as the turn ends, the damage Pride Sovereign takes during combat may become lethal if other Cats you control leave the battlefield later in the turn."
    );
    ruling!(
        "Thelon of Havenwood",
        "Because damage remains marked on a creature until the damage is removed as the turn ends, nonlethal damage dealt to a Fungus creature may become lethal if Thelon leaves the battlefield during that turn or if spore counters are removed from the Fungus during that turn."
    );
    ruling!(
        "Sedge Sliver",
        "Because damage remains marked on a creature until the damage is removed as the turn ends, nonlethal damage dealt to a Sliver may become lethal if Sedge Sliver leaves the battlefield during that turn."
    );
    supported("Pride Sovereign");
    supported("Thelon of Havenwood");
    supported("Sedge Sliver");
    let src = |t: &mut TestGame| t.battlefield(P1, "Grizzly Bears");
    // Pride Sovereign with two other Cats is 4/4: 3 damage, then a Cat dies.
    let mut t = TestGame::new(2);
    let sov = t.battlefield(P0, "Pride Sovereign");
    let lion = t.battlefield(P0, "Savannah Lions");
    t.battlefield(P0, "Savannah Lions");
    assert_eq!(t.pt(sov), (4, 4));
    let s = src(&mut t);
    damage(&mut t, s, 3, sov);
    assert!(t.on_battlefield(sov));
    assert_eq!(marked(&t, sov), 3);
    kill(&mut t, lion);
    assert!(t.in_graveyard(P0, "Pride Sovereign"));
    // A Thallid with two spore counters is 3/3 under Thelon: 2 damage, then Thelon
    // leaves (or the counters are removed).
    for remove_counters in [false, true] {
        let mut t = TestGame::new(2);
        let thelon = t.battlefield(P0, "Thelon of Havenwood");
        let thallid = t.battlefield(P0, "Thallid");
        t.g.objects[thallid.0 as usize]
            .counters
            .insert("spore".into(), 2);
        t.g.recompute();
        assert_eq!(t.pt(thallid), (3, 3));
        let s = src(&mut t);
        damage(&mut t, s, 2, thallid);
        assert!(t.on_battlefield(thallid));
        if remove_counters {
            t.g.remove_counters(Entity::Object(thallid), "spore", 2);
            t.g.flush_events();
            t.settle();
        } else {
            kill(&mut t, thelon);
        }
        assert!(t.in_graveyard(P0, "Thallid"), "removing counters: {remove_counters}");
    }
    // Metallic Sliver is 2/2 under Sedge Sliver while you control a Swamp: 1 damage, then
    // Sedge Sliver leaves.
    let mut t = TestGame::new(2);
    let sedge = t.battlefield(P0, "Sedge Sliver");
    let metallic = t.battlefield(P0, "Metallic Sliver");
    t.battlefield(P0, "Swamp");
    assert_eq!(t.pt(metallic), (2, 2));
    let s = src(&mut t);
    damage(&mut t, s, 1, metallic);
    assert!(t.on_battlefield(metallic));
    kill(&mut t, sedge);
    assert!(t.in_graveyard(P0, "Metallic Sliver"));
}

#[test]
fn kazarov_must_survive_simultaneous_damage_and_triggers_once_per_event() {
    cr!("120.4", "603.2", "603.10a", "704.5g");
    ruling!(
        "Kazarov, Sengir Pureblood",
        "If Kazarov is dealt damage at the same time that a creature an opponent controls is dealt damage, Kazarov must survive the damage to get a +1/+1 counter."
    );
    ruling!(
        "Kazarov, Sengir Pureblood",
        "If a creature is dealt an amount of damage \"for each\" of something, that damage is dealt as one event and Kazarov's triggered ability triggers only once."
    );
    supported("Kazarov, Sengir Pureblood");
    supported("Pyroclasm");
    supported("Spitting Earth");
    // Pyroclasm: "Pyroclasm deals 2 damage to each creature." Kazarov (4/4) survives and
    // gets a counter for the opponent's creature.
    for (marked_before, survives) in [(0, true), (2, false)] {
        let mut t = TestGame::new(2);
        let kaz = t.battlefield(P0, "Kazarov, Sengir Pureblood");
        t.battlefield(P1, "Hill Giant");
        if marked_before > 0 {
            let s = t.battlefield(P1, "Wall of Stone");
            damage(&mut t, s, marked_before, kaz);
        }
        let pyro = t.hand(P0, "Pyroclasm");
        mana(&mut t, P0, &[ManaType::R, ManaType::C]);
        t.cast(P0, pyro).go();
        t.resolve_all();
        assert_eq!(t.on_battlefield(kaz), survives);
        if survives {
            assert_eq!(t.counters(kaz, "+1/+1"), 1);
        }
    }
    // Spitting Earth: "Spitting Earth deals damage to target creature equal to the
    // number of Mountains you control." Three Mountains: 3 damage, one event.
    let mut t = TestGame::new(2);
    let kaz = t.battlefield(P0, "Kazarov, Sengir Pureblood");
    let giant = t.battlefield(P1, "Craw Wurm");
    t.lands(P0, "Mountain", 3);
    let spit = t.hand(P0, "Spitting Earth");
    mana(&mut t, P0, &[ManaType::R, ManaType::C]);
    t.cast(P0, spit).target(giant).go();
    t.resolve_all();
    assert_eq!(marked(&t, giant), 3);
    assert_eq!(t.counters(kaz, "+1/+1"), 1);
}

#[test]
fn vein_drinker_counts_any_creature_it_damaged_that_dies() {
    cr!("603.10a", "603.2", "120.3");
    ruling!(
        "Vein Drinker",
        "It doesn't matter whether the damage was combat damage or not. It also doesn't matter who controlled the creature or whose graveyard it was put into."
    );
    ruling!(
        "Vein Drinker",
        "If Vein Drinker's first ability causes Vein Drinker and the targeted creature to deal lethal damage to each other, they'll be destroyed at the same time. Vein Drinker's triggered ability will trigger, but it will have been put into a graveyard before it would receive any counters. The ability will do nothing when it resolves."
    );
    supported("Vein Drinker");
    // "{R}, {T}: This creature deals damage equal to its power to target creature. That
    // creature deals damage equal to its power to this creature."
    for owner in [P1, P0] {
        let mut t = TestGame::new(2);
        let vein = t.battlefield(P0, "Vein Drinker");
        let bears = t.battlefield(owner, "Grizzly Bears");
        mana(&mut t, P0, &[ManaType::R]);
        t.answer_targets(P0, &[Entity::Object(bears)]);
        activate_containing(&mut t, P0, vein, "deals damage equal").unwrap();
        t.resolve_all();
        assert!(t.in_graveyard(owner, "Grizzly Bears"));
        assert_eq!(t.counters(vein, "+1/+1"), 1, "{owner}'s creature");
        assert_eq!(marked(&t, vein), 2);
    }
    // Combat damage, and the creature dies later in the turn.
    let mut t = TestGame::new(2);
    let vein = t.battlefield(P0, "Vein Drinker");
    // Wall of Denial: a 0/8 flier.
    let wall = t.battlefield(P1, "Wall of Denial");
    to_combat(&mut t, P0);
    attack_with(&mut t, &[(vein, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[(wall, vein)]);
    assert_eq!(marked(&t, wall), 4);
    assert_eq!(t.counters(vein, "+1/+1"), 0);
    kill(&mut t, wall);
    t.resolve_all();
    assert_eq!(t.counters(vein, "+1/+1"), 1);
    // They deal lethal damage to each other: the trigger does nothing.
    let mut t = TestGame::new(2);
    let vein = t.battlefield(P0, "Vein Drinker");
    let wurm = t.battlefield(P1, "Craw Wurm");
    mana(&mut t, P0, &[ManaType::R]);
    t.answer_targets(P0, &[Entity::Object(wurm)]);
    activate_containing(&mut t, P0, vein, "deals damage equal").unwrap();
    t.resolve();
    assert!(t.in_graveyard(P0, "Vein Drinker"));
    assert!(t.in_graveyard(P1, "Craw Wurm"));
    assert_eq!(t.stack_len(), 1, "the ability triggered");
    t.resolve_all();
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn multiple_deathtouch_is_redundant_and_one_regeneration_saves() {
    cr!("702.2c", "702.2f", "704.5h", "701.19c");
    ruling!(
        "Vectis Silencers",
        "If you activate the ability multiple times during the same turn, Vectis Silencers will gain multiple instances of deathtouch; however, multiple instances of deathtouch are redundant."
    );
    supported("Vectis Silencers");
    supported("River Boa");
    let mut t = TestGame::new(2);
    let silencers = t.battlefield(P0, "Vectis Silencers");
    let boa = t.battlefield(P1, "River Boa");
    for _ in 0..2 {
        mana(&mut t, P0, &[ManaType::B, ManaType::C, ManaType::C]);
        activate_containing(&mut t, P0, silencers, "deathtouch").unwrap();
        t.resolve_all();
    }
    let n = t
        .obj_now(silencers)
        .chars
        .keywords()
        .filter(|k| k.kind == KeywordKind::Deathtouch)
        .count();
    assert_eq!(n, 2);
    // River Boa: "{G}: Regenerate this creature." One shield.
    mana(&mut t, P1, &[ManaType::G]);
    activate_containing(&mut t, P1, boa, "Regenerate").unwrap();
    t.resolve_all();
    damage(&mut t, silencers, 1, boa);
    assert!(t.on_battlefield(boa), "regenerated");
    assert!(t.obj_now(boa).tapped);
    assert_eq!(marked(&t, boa), 0);
    // Control: without a shield, it's destroyed.
    damage(&mut t, silencers, 1, boa);
    assert!(t.in_graveyard(P1, "River Boa"));
}

#[test]
fn a_regeneration_shield_works_against_destruction_and_lethal_damage() {
    cr!("701.19a", "701.19b", "614.8", "704.5g");
    ruling!(
        "Spectral Lynx",
        "This shield works against effects that try to destroy Spectral Lynx or lethal damage that would be dealt to Spectral Lynx."
    );
    supported("Spectral Lynx");
    let shield = |t: &mut TestGame, lynx: ObjectId| {
        mana(t, P0, &[ManaType::B]);
        activate_containing(t, P0, lynx, "Regenerate").unwrap();
        t.resolve_all();
        let l = t.g.current(lynx);
        t.g.objects[l.0 as usize].tapped = false;
    };
    let mut t = TestGame::new(2);
    let lynx = t.battlefield(P0, "Spectral Lynx");
    shield(&mut t, lynx);
    kill(&mut t, lynx);
    assert!(t.on_battlefield(lynx), "destroy");
    assert!(t.obj_now(lynx).tapped);
    shield(&mut t, lynx);
    // (A red source: Spectral Lynx has protection from green.)
    let s = t.battlefield(P1, "Hill Giant");
    damage(&mut t, s, 3, lynx);
    assert!(t.on_battlefield(lynx), "lethal damage");
    assert_eq!(marked(&t, lynx), 0);
    // The shield was used up.
    kill(&mut t, lynx);
    assert!(t.in_graveyard(P0, "Spectral Lynx"));
}

#[test]
fn damage_redirected_to_a_target_thats_gone_is_dealt_to_the_creature() {
    cr!("614.9", "608.2b");
    ruling!(
        "Mirrorwood Treefolk",
        "If the target creature is not on the battlefield (or is not a creature) at the time the damage would be redirected, then the damage goes on this card."
    );
    supported("Mirrorwood Treefolk");
    // "{2}{R}{W}: The next time damage would be dealt to this creature this turn, that
    // damage is dealt to any target instead."
    for gone in [false, true] {
        let mut t = TestGame::new(2);
        let tree = t.battlefield(P0, "Mirrorwood Treefolk");
        let bears = t.battlefield(P1, "Grizzly Bears");
        mana(&mut t, P0, &[ManaType::R, ManaType::W, ManaType::C, ManaType::C]);
        t.answer_targets(P0, &[Entity::Object(bears)]);
        activate_containing(&mut t, P0, tree, "damage would be dealt").unwrap();
        t.resolve_all();
        if gone {
            kill(&mut t, bears);
        }
        let s = t.battlefield(P1, "Hill Giant");
        damage(&mut t, s, 3, tree);
        if gone {
            assert_eq!(marked(&t, tree), 3);
        } else {
            assert_eq!(marked(&t, tree), 0);
            assert!(t.in_graveyard(P1, "Grizzly Bears"));
        }
    }
}
