//! Rulings batch P089 — "humble" effects: losing all abilities (layer 6) and having base
//! power and toughness set (layer 7b). Effects and counters that modify power and
//! toughness apply on top whenever they began (CR 613.4c, 613.4d); later setting effects
//! overwrite earlier ones (CR 613.7); abilities gained later are kept; the affected set of
//! a resolved effect is locked in (CR 611.2c).

use crate::r_p089_common::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Applies the humble effect of the real card `name` to P1's creature `c` (P0 being its
/// controller or caster): Auras are attached, Curses enchant P1, spells are cast and
/// resolved, attack triggers are set up and resolved in P0's combat.
fn humble(t: &mut TestGame, name: &str, c: ObjectId) {
    supported(name);
    match name {
        "Illusory Wrappings" | "Kasmina's Transmutation" | "Reduce in Stature"
        | "Stasis Field" | "Utter Insignificance" | "Mystic Subdual" | "Stop Cold" => {
            attach_new(t, P0, name, obj(c));
        }
        "Overwhelming Splendor" => {
            attach_new(t, P0, name, player(P1));
        }
        "Diminish" | "Ovinize" => cast_and_resolve(t, P0, name, &[obj(c)]),
        "Mass Diminish" | "Sudden Spoiling" => cast_and_resolve(t, P0, name, &[player(P1)]),
        "Flatline" => cast_and_resolve(t, P0, name, &[]),
        "Warkite Marauder" => {
            let w = t.battlefield(P0, name);
            t.answer_targets(P0, &[obj(c)]);
            attack_and_resolve(t, w, P1);
        }
        "Vedalken Humiliator" => {
            let v = t.battlefield(P0, name);
            t.lands(P0, "Sol Ring", 3);
            attack_and_resolve(t, v, P1);
        }
        _ => panic!("no setup for {name}"),
    }
    t.g.recompute();
    t.settle();
}

/// The base power and toughness `name` sets.
fn base_of(name: &str) -> (i32, i32) {
    match name {
        "Illusory Wrappings" | "Reduce in Stature" | "Stasis Field" | "Sudden Spoiling" => (0, 2),
        "Ovinize" | "Warkite Marauder" | "Flatline" => (0, 1),
        _ => (1, 1),
    }
}

#[test]
fn modifying_effects_and_counters_apply_on_top_of_the_new_base() {
    cr!("613.4b", "613.4c", "613.4d");
    ruling!(
        "Vedalken Humiliator",
        "Any power- or toughness-modifying effects (those that give +N/+N, for example) and counters will apply to the creature's new base power and toughness, even if they started to apply before Vedalken Humiliator's ability resolved."
    );
    ruling!(
        "Illusory Wrappings",
        "Any power- or toughness-modifying effects (those that give +N/+N, for example) and counters will apply to the creature’s new base power and toughness, even if they started to apply before Illusory Wrappings became attached."
    );
    ruling!(
        "Kasmina's Transmutation",
        "Effects that modify a creature's power and/or toughness, such as the effect of Bleeding Edge, will apply to the creature no matter when they started to take effect."
    );
    ruling!(
        "Warkite Marauder",
        "The same is true for any counters that change its power and/or toughness and effects that switch its power and toughness."
    );
    ruling!(
        "Overwhelming Splendor",
        "Effects that modify a creature's power and/or toughness, such as the effect of Titanic Growth, will apply to the creatures no matter when they started to take effect."
    );
    ruling!(
        "Ovinize",
        "Effects that modify a creature’s power and/or toughness without setting them to specific values will apply to the creature no matter when those effects began."
    );
    ruling!(
        "Reduce in Stature",
        "The same is true for counters that change the creature’s power or toughness (such as +1/+1 counters) and effects that switch its power and toughness."
    );
    ruling!(
        "Diminish",
        "Effects that modify the target creature’s power or toughness, such as the effects of Elesh Norn, Grand Cenobite or prowess, will apply to it no matter when they started to take effect."
    );
    ruling!(
        "Mass Diminish",
        "Effects that raise or lower a creature's power and/or toughness, such as the effect of Titanic Growth, will apply to that creature no matter when they started to take effect."
    );
    ruling!(
        "Flatline",
        "Any existing effects or counters that raise, lower, or switch a creature's power and/or toughness continue to apply to the creature's newly-set power and toughness."
    );
    ruling!(
        "Sudden Spoiling",
        "Any existing effects or counters that raise, lower, or switch a creature's power and/or toughness continue to apply to the creature's newly-set power and toughness."
    );
    // (card, whether a switch effect is also applied first)
    for (name, switch) in [
        ("Vedalken Humiliator", false),
        ("Illusory Wrappings", false),
        ("Kasmina's Transmutation", false),
        ("Warkite Marauder", true),
        ("Overwhelming Splendor", false),
        ("Ovinize", false),
        ("Reduce in Stature", true),
        ("Diminish", false),
        ("Mass Diminish", false),
        ("Flatline", true),
        ("Sudden Spoiling", true),
    ] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        // Before the effect: two +1/+1 counters and Giant Growth (+3/+3).
        put_counters(&mut t, bears, "+1/+1", 2);
        cast_and_resolve(&mut t, P1, "Giant Growth", &[obj(bears)]);
        if switch {
            cast_and_resolve(&mut t, P1, "Twisted Image", &[obj(bears)]);
        }
        humble(&mut t, name, bears);
        let (p, tough) = base_of(name);
        let want = if switch {
            (tough + 5, p + 5)
        } else {
            (p + 5, tough + 5)
        };
        assert_eq!(t.pt(bears), want, "{name}");
    }
}

#[test]
fn later_setting_effects_overwrite_earlier_ones() {
    cr!("613.4b", "613.7");
    ruling!(
        "Diminish",
        "Diminish overwrites all previous effects that set the target creature’s power and toughness to specific values."
    );
    ruling!(
        "Illusory Wrappings",
        "Any power- or toughness-setting effects that start to apply to a creature after Illusory Wrappings becomes attached to it will overwrite this effect."
    );
    ruling!(
        "Kasmina's Transmutation",
        "Kasmina's Transmutation overwrites all previous effects that set the creature's base power and toughness to specific values."
    );
    ruling!(
        "Mass Diminish",
        "Mass Diminish overwrites all previous effects that set a creature's base power and toughness to specific values."
    );
    ruling!(
        "Ovinize",
        "Ovinize overwrites all previous effects that set the creature’s base power and toughness to specific values."
    );
    ruling!(
        "Reduce in Stature",
        "Reduce in Stature overrides all previous effects that set the creature’s power or toughness to specific values."
    );
    ruling!(
        "Stasis Field",
        "Stasis Field overwrites all previous effects that set the creature’s base power and toughness to specific values."
    );
    ruling!(
        "Utter Insignificance",
        "Utter Insignificance overwrites all previous effects that set the creature's base power and toughness to specific values."
    );
    ruling!(
        "Vedalken Humiliator",
        "Vedalken Humiliator's ability overwrites all previous effects that set an affected creature's base power and toughness to specific values."
    );
    ruling!(
        "Warkite Marauder",
        "Warkite Marauder's ability overwrites all previous effects that set the creature's base power and toughness to specific values."
    );
    ruling!(
        "Flatline",
        "Flatline overwrites any effects that set a creature's power and toughness."
    );
    ruling!(
        "Sudden Spoiling",
        "Sudden Spoiling overwrites any effects that set a creature's power and toughness."
    );
    for name in [
        "Diminish",
        "Illusory Wrappings",
        "Kasmina's Transmutation",
        "Mass Diminish",
        "Ovinize",
        "Reduce in Stature",
        "Stasis Field",
        "Utter Insignificance",
        "Vedalken Humiliator",
        "Warkite Marauder",
        "Flatline",
        "Sudden Spoiling",
    ] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        // Square Up: base 4/4 until end of turn, before the effect...
        cast_and_resolve(&mut t, P1, "Square Up", &[obj(bears)]);
        assert_eq!(t.pt(bears), (4, 4));
        humble(&mut t, name, bears);
        assert_eq!(t.pt(bears), base_of(name), "{name}");
        // ...and again after it: the later one wins.
        cast_and_resolve(&mut t, P1, "Square Up", &[obj(bears)]);
        assert_eq!(t.pt(bears), (4, 4), "{name}");
    }
}

#[test]
fn illusory_wrappings_overwrites_a_characteristic_defining_ability() {
    cr!("604.3", "613.4a", "613.4b");
    ruling!(
        "Illusory Wrappings",
        "including those that define a * such as that of Enigma Drake"
    );
    supported("Enigma Drake");
    let mut t = TestGame::new(2);
    let drake = t.battlefield(P1, "Enigma Drake");
    t.graveyard(P1, "Shock");
    t.graveyard(P1, "Divination");
    t.g.recompute();
    assert_eq!(t.pt(drake), (2, 4));
    humble(&mut t, "Illusory Wrappings", drake);
    assert_eq!(t.pt(drake), (0, 2));
}

#[test]
fn abilities_gained_afterwards_are_kept() {
    cr!("613.1f", "613.7");
    ruling!(
        "Overwhelming Splendor",
        "If an effect grants a creature an ability after Overwhelming Splendor has become attached to its controller, that creature won't lose that ability."
    );
    ruling!(
        "Kasmina's Transmutation",
        "If the affected creature gains an ability after Kasmina's Transmutation becomes attached to it, it will keep that ability."
    );
    ruling!(
        "Mystic Subdual",
        "If the affected creature gains an ability after Mystic Subdual becomes attached to it, it will keep that ability."
    );
    ruling!(
        "Warkite Marauder",
        "If the affected creature gains an ability after Warkite Marauder's ability resolves, it will keep that ability."
    );
    ruling!(
        "Stop Cold",
        "If the affected permanent gains an ability after Stop Cold becomes attached to it, it will keep that ability."
    );
    for name in [
        "Overwhelming Splendor",
        "Kasmina's Transmutation",
        "Mystic Subdual",
        "Warkite Marauder",
        "Stop Cold",
    ] {
        let mut t = TestGame::new(2);
        let angel = t.battlefield(P1, "Serra Angel");
        humble(&mut t, name, angel);
        assert!(!t.obj_now(angel).has_keyword(KeywordKind::Flying), "{name}");
        assert!(
            !t.obj_now(angel).has_keyword(KeywordKind::Vigilance),
            "{name}"
        );
        // Jump: target creature gains flying until end of turn.
        cast_and_resolve(&mut t, P1, "Jump", &[obj(angel)]);
        assert!(t.obj_now(angel).has_keyword(KeywordKind::Flying), "{name}");
        assert!(
            !t.obj_now(angel).has_keyword(KeywordKind::Vigilance),
            "{name}"
        );
    }
}

#[test]
fn warkite_marauder_stops_an_ability_granting_abilities_to_others() {
    cr!("613.1f", "613.4c");
    ruling!(
        "Warkite Marauder",
        "If the affected creature has an ability that grants abilities to other objects, Warkite Marauder's effect will stop it from doing so."
    );
    supported("Lord of Atlantis");
    let mut t = TestGame::new(2);
    let lord = t.battlefield(P1, "Lord of Atlantis");
    let merfolk = t.battlefield(P1, "Merfolk of the Pearl Trident");
    assert_eq!(t.pt(merfolk), (2, 2));
    assert!(t.obj_now(merfolk).has_keyword(KeywordKind::Landwalk));
    humble(&mut t, "Warkite Marauder", lord);
    assert_eq!(t.pt(lord), (0, 1));
    assert_eq!(t.pt(merfolk), (1, 1));
    assert!(!t.obj_now(merfolk).has_keyword(KeywordKind::Landwalk));
}

#[test]
fn affected_set_is_locked_in_on_resolution() {
    cr!("611.2c");
    ruling!(
        "Flatline",
        "Flatline affects only creatures your opponents control at the time it resolves. Creatures they begin to control later in the turn won't change power and toughness."
    );
    ruling!(
        "Sudden Spoiling",
        "Sudden Spoiling affects only creatures the target player controls at the time it resolves."
    );
    ruling!(
        "Vedalken Humiliator",
        "Vedalken Humiliator's ability affects only creatures your opponents control at the time it resolves."
    );
    for name in ["Flatline", "Sudden Spoiling", "Vedalken Humiliator"] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        let angel = t.battlefield(P0, "Serra Angel");
        humble(&mut t, name, bears);
        assert_eq!(t.pt(bears), base_of(name), "{name}");
        // P1 gains control of P0's Serra Angel afterwards: it isn't affected.
        give_control(&mut t, angel, P1);
        assert_eq!(t.obj_now(angel).controller, P1);
        assert_eq!(t.pt(angel), (4, 4), "{name}");
        assert!(t.obj_now(angel).has_keyword(KeywordKind::Flying), "{name}");
    }
}

#[test]
fn vedalken_humiliator_metalcraft_checked_on_resolution_only() {
    cr!("603.4", "611.2c");
    ruling!(
        "Vedalken Humiliator",
        "If you don't control three artifacts as Vedalken Humiliator's ability resolves, it has no effect. If you do, the effect continues to apply even if you lose control of Vedalken Humiliator or your artifacts later in the turn."
    );
    supported("Vedalken Humiliator");
    // Three artifacts as it attacks, two as the ability resolves: no effect.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let v = t.battlefield(P0, "Vedalken Humiliator");
    let rings = t.lands(P0, "Sol Ring", 3);
    to_beginning_of_combat(&mut t, P0);
    attack_with(&mut t, &[(v, player(P1))]);
    assert_eq!(t.stack_len(), 1);
    crate::r_s05_common::move_to(&mut t, rings[0], Zone::Graveyard(P0));
    t.resolve_all();
    assert_eq!(t.pt(bears), (2, 2));
    // With all three: the effect stays after P0 loses the artifacts and the Humiliator.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    humble(&mut t, "Vedalken Humiliator", bears);
    assert_eq!(t.pt(bears), (1, 1));
    let v = t.named_on_battlefield("Vedalken Humiliator")[0];
    give_control(&mut t, v, P1);
    for r in t.named_on_battlefield("Sol Ring") {
        crate::r_s05_common::move_to(&mut t, r, Zone::Graveyard(P0));
    }
    assert_eq!(t.pt(bears), (1, 1));
}

#[test]
fn vedalken_humiliator_affects_every_opponent() {
    cr!("611.2c");
    ruling!(
        "Vedalken Humiliator",
        "All creatures your opponents control are affected by Vedalken Humiliator's ability, not just those controlled by the player it's attacking."
    );
    supported("Vedalken Humiliator");
    let mut t = TestGame::new(3);
    let b1 = t.battlefield(P1, "Grizzly Bears");
    let b2 = t.battlefield(P2, "Serra Angel");
    let mine = t.battlefield(P0, "Hill Giant");
    let v = t.battlefield(P0, "Vedalken Humiliator");
    t.lands(P0, "Sol Ring", 3);
    attack_and_resolve(&mut t, v, P1);
    assert_eq!(t.pt(b1), (1, 1));
    assert_eq!(t.pt(b2), (1, 1));
    assert!(no_abilities(&t, b2));
    assert_eq!(t.pt(mine), (3, 3));
    assert_eq!(t.pt(v), (3, 4));
}

#[test]
fn marked_damage_can_become_lethal() {
    cr!("704.5g", "613.4b");
    ruling!(
        "Mass Diminish",
        "Because damage remains marked on a creature until it's removed as the turn ends, nonlethal damage dealt to a creature the target player controls may become lethal once its base toughness becomes 1."
    );
    ruling!(
        "Utter Insignificance",
        "nonlethal damage dealt to a creature may become lethal if Utter Insignificance becomes attached to it during that turn."
    );
    for name in ["Mass Diminish", "Utter Insignificance"] {
        let mut t = TestGame::new(2);
        let giant = t.battlefield(P1, "Hill Giant");
        let other = t.battlefield(P1, "Grizzly Bears");
        cast_and_resolve(&mut t, P0, "Shock", &[obj(giant)]);
        assert!(t.on_battlefield(giant));
        humble(&mut t, name, giant);
        assert!(!t.on_battlefield(giant), "{name}");
        assert!(t.on_battlefield(other), "{name}");
    }
}

#[test]
fn graven_dominator_with_night_of_souls_betrayal() {
    cr!("613.4b", "613.4c", "704.5f");
    ruling!(
        "Graven Dominator",
        "If Night of Souls' Betrayal is on the battlefield when Graven Dominator's ability resolves, all other creatures will become 0/0 and die."
    );
    supported("Graven Dominator");
    supported("Night of Souls' Betrayal");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Night of Souls' Betrayal");
    let giant = t.battlefield(P1, "Hill Giant");
    let mine = t.battlefield(P0, "Serra Angel");
    let dom = crate::r_s05_common::enter(&mut t, P0, "Graven Dominator");
    t.resolve_all();
    assert!(!t.on_battlefield(giant));
    assert!(!t.on_battlefield(mine));
    assert_eq!(t.pt(dom), (3, 3));
}

#[test]
fn characteristic_defining_pt_becomes_zero_when_abilities_are_lost() {
    cr!("604.3", "613.1f", "613.4a", "208.2b");
    ruling!(
        "Mystic Subdual",
        "Mystic Subdual then makes it a -2/0. If its power and toughness are written as */*+1, it’s -2/1, and so on."
    );
    ruling!(
        "Merfolk Trickster",
        "it’s 0/0 when it loses all abilities. If its power and toughness are written as */*+1, it’s 0/1, and so on."
    );
    supported("Nightmare");
    supported("Tarmogoyf");
    supported("Mystic Subdual");
    supported("Merfolk Trickster");
    // Mystic Subdual: Nightmare (*/*) dies as a -2/0; Tarmogoyf (*/1+*) is -2/1.
    let mut t = TestGame::new(2);
    t.lands(P1, "Swamp", 3);
    let nightmare = t.battlefield(P1, "Nightmare");
    let goyf = t.battlefield(P1, "Tarmogoyf");
    t.graveyard(P1, "Shock");
    t.graveyard(P1, "Grizzly Bears");
    t.g.recompute();
    assert_eq!(t.pt(nightmare), (3, 3));
    assert_eq!(t.pt(goyf), (2, 3));
    humble(&mut t, "Mystic Subdual", nightmare);
    assert!(!t.on_battlefield(nightmare));
    humble(&mut t, "Mystic Subdual", goyf);
    assert_eq!(t.pt(goyf), (-2, 1));
    // Merfolk Trickster: Nightmare dies as a 0/0; Tarmogoyf is 0/1.
    let mut t = TestGame::new(2);
    t.lands(P1, "Swamp", 3);
    let nightmare = t.battlefield(P1, "Nightmare");
    let goyf = t.battlefield(P1, "Tarmogoyf");
    t.graveyard(P1, "Shock");
    t.graveyard(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(goyf)]);
    crate::r_s05_common::enter(&mut t, P0, "Merfolk Trickster");
    t.resolve_all();
    assert_eq!(t.pt(goyf), (0, 1));
    assert!(t.obj_now(goyf).tapped);
    t.answer_targets(P0, &[obj(nightmare)]);
    crate::r_s05_common::enter(&mut t, P0, "Merfolk Trickster");
    t.resolve_all();
    assert!(!t.on_battlefield(nightmare));
}

#[test]
fn merfolk_trickster_becomes_tapped_trigger_still_triggers() {
    cr!("603.2", "603.10");
    ruling!(
        "Merfolk Trickster",
        "If the target creature has an ability that triggers when it becomes tapped, that ability triggers before it loses all abilities."
    );
    supported("Merfolk Trickster");
    supported("Night Market Lookout");
    let mut t = TestGame::new(2);
    let lookout = t.battlefield(P1, "Night Market Lookout");
    t.answer_targets(P0, &[obj(lookout)]);
    crate::r_s05_common::enter(&mut t, P0, "Merfolk Trickster");
    t.resolve_all();
    assert!(t.obj_now(lookout).tapped);
    assert!(no_abilities(&t, lookout));
    // Night Market Lookout: each opponent loses 1 life and its controller gains 1.
    assert_eq!(t.life(P0), 19);
    assert_eq!(t.life(P1), 21);
}

#[test]
fn activated_ability_resolves_after_its_source_loses_it() {
    cr!("113.7a", "602.2");
    ruling!(
        "Merfolk Trickster",
        "Once activated, the ability on the stack will resolve even if the creature loses the ability."
    );
    supported("Merfolk Trickster");
    supported("Prodigal Sorcerer");
    let mut t = TestGame::new(2);
    let tim = t.battlefield(P1, "Prodigal Sorcerer");
    t.activate(P1, tim, 0, &[player(P0)]).unwrap();
    assert_eq!(t.stack_len(), 1);
    // In response, P0 flashes in Merfolk Trickster targeting the Sorcerer.
    t.answer_targets(P0, &[obj(tim)]);
    crate::r_s05_common::enter(&mut t, P0, "Merfolk Trickster");
    t.resolve();
    assert!(no_abilities(&t, tim));
    t.resolve_all();
    assert_eq!(t.life(P0), 19);
}

#[test]
fn humility_removes_mana_abilities_of_creature_lands() {
    cr!("613.1f", "305.6");
    ruling!(
        "Humility",
        "Removes all creature abilities. This includes mana abilities. Animated lands will also lose the ability to tap for mana."
    );
    supported("Humility");
    supported("Dryad Arbor");
    let mut t = TestGame::new(2);
    let arbor = t.battlefield(P0, "Dryad Arbor");
    let forest = t.battlefield(P0, "Forest");
    assert!(!no_abilities(&t, arbor));
    t.battlefield(P1, "Humility");
    t.g.recompute();
    assert!(no_abilities(&t, arbor));
    assert_eq!(t.pt(arbor), (1, 1));
    // A noncreature land keeps its mana ability.
    assert!(!no_abilities(&t, forest));
}

#[test]
fn humility_and_two_opalescences_by_timestamp() {
    cr!("613.1d", "613.1f", "613.4b", "613.6", "613.7");
    ruling!(
        "Humility",
        "With a Humility and two Opalescences on the battlefield, if Humility has the latest timestamp"
    );
    ruling!(
        "Opalescence",
        "With a Humility and two Opalescences on the battlefield, if Humility has the latest timestamp"
    );
    supported("Humility");
    supported("Opalescence");
    // order: indices into [Humility, Opalescence A, Opalescence B]; the expected P/T of
    // each (in the same order).
    for (order, want) in [
        ([1, 2, 0], [(1, 1), (1, 1), (1, 1)]),
        ([1, 0, 2], [(4, 4), (4, 4), (1, 1)]),
        ([0, 1, 2], [(4, 4), (4, 4), (4, 4)]),
    ] {
        let mut t = TestGame::new(2);
        let names = ["Humility", "Opalescence", "Opalescence"];
        let mut ids = [ObjectId(0); 3];
        for i in order {
            ids[i] = t.battlefield(P0, names[i]);
        }
        t.g.recompute();
        for i in 0..3 {
            assert_eq!(t.pt(ids[i]), want[i], "order {order:?}, card {i}");
        }
        // In the mixed order: the second Opalescence is 1/1 without abilities.
        if order == [1, 0, 2] {
            assert!(no_abilities(&t, ids[2]));
        }
    }
}

#[test]
fn ray_of_frost_on_a_self_animated_permanent() {
    cr!("613.1d", "613.1f", "613.6");
    ruling!(
        "Ray of Frost",
        "that permanent will remain a creature even though it loses the ability that is making it a creature after that ability applies."
    );
    supported("Ray of Frost");
    supported("Grand Master of Flowers");
    supported("Chaoslace");
    let mut t = TestGame::new(2);
    let gm = t.battlefield(P1, "Grand Master of Flowers");
    put_counters(&mut t, gm, "loyalty", 4);
    t.g.recompute();
    assert!(t.obj_now(gm).is(types::CardType::Creature));
    attach_new(&mut t, P0, "Ray of Frost", obj(gm));
    // Not red: it keeps its abilities.
    assert!(!no_abilities(&t, gm));
    // It becomes red: it loses its abilities but stays a creature.
    cast_and_resolve(&mut t, P0, "Chaoslace", &[obj(gm)]);
    t.g.recompute();
    assert!(no_abilities(&t, gm));
    assert!(t.obj_now(gm).is(types::CardType::Creature));
}

#[test]
fn mass_diminish_targets_the_player() {
    cr!("115.10a", "702.11b", "702.16b");
    ruling!(
        "Mass Diminish",
        "Mass Diminish targets the player, not their creatures. A creature with hexproof or protection from blue may be shrunk this way."
    );
    supported("Mass Diminish");
    let mut t = TestGame::new(2);
    let hexproof = t.battlefield(P1, "Gladecover Scout");
    let pro_blue = t.battlefield(P1, "Nacatl Outlander");
    humble(&mut t, "Mass Diminish", hexproof);
    assert_eq!(t.pt(hexproof), (1, 1));
    assert_eq!(t.pt(pro_blue), (1, 1));
}

#[test]
fn mass_diminish_lasts_until_its_casters_next_turn_would_begin() {
    cr!("800.4m", "611.2a");
    ruling!(
        "Mass Diminish",
        "In a multiplayer game, if you leave the game after Mass Diminish resolves but before your next turn begins, its effect lasts until your next turn would have begun."
    );
    supported("Mass Diminish");
    let mut t = TestGame::new(3);
    let giant = t.battlefield(P1, "Hill Giant");
    cast_and_resolve(&mut t, P0, "Mass Diminish", &[player(P1)]);
    assert_eq!(t.pt(giant), (1, 1));
    t.advance_to(P1, Step::Upkeep);
    t.g.player_loses(P0);
    t.settle();
    assert!(!t.g.player(P0).in_game());
    // Still in effect during P1's and P2's turns...
    assert_eq!(t.pt(giant), (1, 1));
    t.advance_to(P2, Step::Upkeep);
    assert_eq!(t.pt(giant), (1, 1));
    // ...and over when P0's turn would have begun.
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.pt(giant), (3, 3));
}

#[test]
fn awakened_awareness_with_x_zero() {
    cr!("107.3", "613.4b");
    ruling!(
        "Awakened Awareness",
        "You may cast this spell with X equal to 0 if, for example, you just want an opponent's creature to have base power and toughness 1/1 but not give it counters."
    );
    supported("Awakened Awareness");
    let mut t = TestGame::new(2);
    let angel = t.battlefield(P1, "Serra Angel");
    t.lands(P0, "Island", 2);
    let aura = t.hand(P0, "Awakened Awareness");
    t.cast(P0, aura).x(0).target(angel).go();
    t.resolve_all();
    assert_eq!(t.pt(angel), (1, 1));
    assert_eq!(t.counters(angel, "+1/+1"), 0);
    assert_eq!(t.obj_now(angel).controller, P1);
}

#[test]
fn overwhelming_splendor_restricts_activated_abilities() {
    cr!("602.5", "605.1a", "606.3");
    ruling!(
        "Overwhelming Splendor",
        "Triggered abilities (starting with \"when,\" \"whenever,\" or \"at\") of noncreature permanents are unaffected by the last ability of Overwhelming Splendor."
    );
    ruling!(
        "Overwhelming Splendor",
        "An activated mana ability is one that produces mana as it resolves, not one that costs mana to activate."
    );
    supported("Overwhelming Splendor");
    // Control: without the Curse, P1 can equip in their main phase.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let sword = t.battlefield(P1, "Bonesplitter");
    crate::r_p076_common::mana(&mut t, P1, mana::ManaType::C, 1);
    assert!(t.activate(P1, sword, 0, &[obj(bears)]).is_ok());
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    attach_new(&mut t, P0, "Overwhelming Splendor", player(P1));
    // An equip ability (activated, with a colon in its reminder text) can't be activated.
    let bears = t.battlefield(P1, "Grizzly Bears");
    let sword = t.battlefield(P1, "Bonesplitter");
    crate::r_p076_common::mana(&mut t, P1, mana::ManaType::C, 1);
    assert!(t.activate(P1, sword, 0, &[obj(bears)]).is_err());
    // A mana ability can.
    let ring = t.battlefield(P1, "Sol Ring");
    assert!(t.activate(P1, ring, 0, &[]).is_ok());
    // A loyalty ability can.
    let pw = t.battlefield(P1, "Ajani Goldmane");
    assert!(t.activate(P1, pw, 0, &[]).is_ok());
    t.resolve_all();
    assert_eq!(t.life(P1), 22);
    // A triggered ability of a noncreature permanent still triggers.
    t.battlefield(P1, "Impact Tremors");
    crate::r_s05_common::enter(&mut t, P1, "Grizzly Bears");
    t.resolve_all();
    assert_eq!(t.life(P0), 19);
}
