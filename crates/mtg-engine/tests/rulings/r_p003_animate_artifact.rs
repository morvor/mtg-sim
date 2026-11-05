//! Rulings batch P003 — effects that make artifacts creatures: base power and toughness
//! set in layer 7b under counters and modifications (7c) and switches (7d) whenever they
//! began (CR 613.4); a later setting effect overwrites an earlier one (CR 613.7); crewing
//! sets no power and toughness (CR 702.122); summoning sickness of an artifact that
//! becomes a creature (CR 302.6); durations (CR 611.2a).

use crate::r_p003_common::*;
use crate::r_s01_common::supported;
use crate::r_s02_common::can_attack;
use crate::r_s04_common::crew;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Workshop Elders's beginning-of-combat trigger resolves, targeting `target` (P0's).
fn elders_animate(t: &mut TestGame, target: ObjectId) {
    t.answer_targets(P0, &[obj(target)]);
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert!(is_creature(t, target), "Workshop Elders animated it");
}

#[test]
fn workshop_elders_artifact_has_summoning_sickness_and_the_effect_lasts() {
    cr!("302.6", "508.1a", "611.2a");
    ruling!(
        "Workshop Elders",
        "An artifact that becomes a creature due to Workshop Elders can't attack unless you've controlled it continuously since your turn began."
    );
    ruling!(
        "Workshop Elders",
        "The effect of Workshop Elders's triggered ability lasts indefinitely. It doesn't wear off during the cleanup step."
    );
    supported("Workshop Elders");
    for new in [true, false] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Workshop Elders");
        let stone = if new {
            t.battlefield_sick(P0, "Mind Stone")
        } else {
            t.battlefield(P0, "Mind Stone")
        };
        elders_animate(&mut t, stone);
        assert_eq!(t.pt(stone), (4, 4));
        assert_eq!(can_attack(&mut t, stone), !new, "new this turn: {new}");
        if !new {
            // Still a 4/4 creature after the cleanup step and on later turns.
            t.advance_to(P1, Step::Upkeep);
            assert!(is_creature(&t, stone));
            t.advance_to(P0, Step::Upkeep);
            assert!(is_creature(&t, stone));
            assert_eq!(t.pt(stone), (4, 4));
        }
    }
}

/// `p` casts Ensoul Artifact on `target`.
fn ensoul(t: &mut TestGame, target: ObjectId, name: &str) {
    let p = t.obj_now(target).controller;
    cast_resolve(t, p, name, "3U", &[obj(target)]);
}

#[test]
fn a_later_setting_effect_wins_over_animating_faerie_and_workshop_elders_crewing_doesnt() {
    cr!("613.4b", "613.7a", "702.122a");
    ruling!(
        "Animating Faerie // Bring to Life",
        "If another effect later causes the artifact to become a creature and sets its power and toughness as it does so, that creature will have that base power and toughness; it won't be 0/0. Notably, crewing a Vehicle does not set its power and toughness, so a Vehicle's base power and toughness remain 0/0 if you crew it."
    );
    ruling!(
        "Workshop Elders",
        "If another effect later causes the artifact to become a creature and sets its power and toughness as it does so, that creature will have that base power and toughness; it won't be 0/0. Notably, crewing a Vehicle does not set its power and toughness, so a Vehicle's base power and toughness remain 0/0 if you crew it."
    );
    supported("Animating Faerie // Bring to Life");
    supported("Smuggler's Copter");
    for elders in [false, true] {
        let mut t = TestGame::new(2);
        // Smuggler's Copter: a 3/3 Vehicle with crew 1.
        let copter = t.battlefield(P0, "Smuggler's Copter");
        if elders {
            t.battlefield(P0, "Workshop Elders");
            elders_animate(&mut t, copter);
        } else {
            let faerie = t.hand(P0, "Animating Faerie // Bring to Life");
            mana(&mut t, P0, "2U");
            t.cast(P0, faerie)
                .method(CastMethod::Half(1))
                .target(obj(copter))
                .go();
            t.resolve_all();
        }
        assert_eq!(t.pt(copter), (4, 4), "elders: {elders}");
        // Crewing makes it an artifact creature again, setting nothing: still 0/0 base.
        let bear = t.battlefield(P0, "Grizzly Bears");
        assert!(crew(&mut t, P0, copter, &[bear]));
        t.resolve_all();
        assert_eq!(t.pt(copter), (4, 4), "elders: {elders}");
        // Ensoul Artifact sets base 5/5 with a later timestamp.
        if elders {
            t.advance_to(P0, Step::PostcombatMain);
            assert!(is_creature(&t, copter), "still crewed");
        }
        ensoul(&mut t, copter, "Ensoul Artifact");
        assert_eq!(t.pt(copter), (9, 9), "elders: {elders}");
    }
}

#[test]
fn modifications_counters_and_switches_apply_to_animated_artifacts_whenever_they_began() {
    cr!("613.4b", "613.4c", "613.4d");
    ruling!(
        "Skilled Animator",
        "Effects that modify a creature's power and/or toughness, such as the ones created by Titanic Growth or a +/1+1 counter, will apply to the creature no matter when they started to take effect."
    );
    ruling!(
        "Tezzeret, Cruel Machinist",
        "Effects that modify a creature's power and/or toughness, such as the ones created by Titanic Growth or a +/1+1 counter, will apply to the creature no matter when they started to take effect."
    );
    ruling!(
        "The Mechanist, Aerial Artisan",
        "Effects that modify a creature's power and/or toughness, such as the ones created by Titanic Growth or a +/1+1 counter, will apply to the creature no matter when they started to take effect."
    );
    ruling!(
        "Ensoul Artifact",
        "Effects that modify a creature's power and/or toughness, such as the ones created by Titanic Growth or a +1/+1 counter, will apply to the creature no matter when they started to take effect."
    );
    supported("Skilled Animator");
    supported("Tezzeret, Cruel Machinist");
    supported("The Mechanist, Aerial Artisan");
    supported("Ensoul Artifact");
    supported("Giant Growth");
    supported("Twisted Image");
    // Each animator: the base power and toughness it sets.
    for (who, base) in [
        ("Skilled Animator", (5, 5)),
        ("Tezzeret", (5, 5)),
        ("The Mechanist", (3, 1)),
        ("Ensoul Artifact", (5, 5)),
    ] {
        let mut t = TestGame::new(2);
        // An artifact creature (Ornithopter, 0/2; an artifact token for The Mechanist)
        // with a +1/+1 counter and Giant Growth from before it's animated.
        let thing = if who == "The Mechanist" {
            let mech = t.battlefield(P0, "The Mechanist, Aerial Artisan");
            // Its first ability creates a Clue token for a noncreature spell.
            cast_resolve(&mut t, P0, "Opt", "U", &[]);
            let clue = *crate::r_s01_common::with_subtype(&t, P0, "Clue")
                .first()
                .expect("a Clue");
            (clue, Some(mech))
        } else {
            (t.battlefield(P0, "Ornithopter"), None)
        };
        let (art, mech) = thing;
        put(&mut t, art, counters::PLUS1, 1);
        if mech.is_none() {
            giant_growth(&mut t, art);
        }
        match who {
            "Skilled Animator" => {
                t.answer_targets(P0, &[obj(art)]);
                enter(&mut t, P0, "Skilled Animator");
                t.resolve_all();
            }
            "Tezzeret" => {
                let tez = t.battlefield(P0, "Tezzeret, Cruel Machinist");
                t.activate(P0, tez, 1, &[obj(art)]).expect("0 ability");
                t.resolve_all();
            }
            "The Mechanist" => {
                activate_resolve(&mut t, P0, mech.unwrap(), "Construct", "", &[obj(art)]);
                giant_growth(&mut t, art);
            }
            _ => ensoul(&mut t, art, "Ensoul Artifact"),
        }
        let want = (base.0 + 4, base.1 + 4);
        assert_eq!(t.pt(art), want, "{who}");
        // A switch, whenever it began, applies last.
        switch_pt(&mut t, art);
        assert_eq!(t.pt(art), (want.1, want.0), "{who}");
    }
}

#[test]
fn an_aura_or_tezzeret_overwrites_earlier_setting_effects_and_later_ones_overwrite_it() {
    cr!("613.4b", "613.7a", "613.7e");
    ruling!(
        "Mightstone's Animation",
        "If the artifact was already a creature, its base power and toughness will each become 4. This overwrites any previous effects that set the creature's base power and toughness to specific values. Any power- or toughness-setting effects that start to apply after Mightstone's Animation becomes attached to the artifact will overwrite this effect."
    );
    ruling!(
        "Ensoul Artifact",
        "If the artifact was already a creature, its base power and toughness will each become 5. This overwrites any previous effects that set the creature's base power and toughness to specific values. Any power- or toughness-setting effects that start to apply after Ensoul Artifact becomes attached to the artifact will overwrite this effect."
    );
    ruling!(
        "Tezzeret, Cruel Machinist",
        "If the artifact was already a creature, its base power and toughness will each become 5. This overwrites any previous effects that set the creature's base power and toughness to specific values. Any power- or toughness-setting effects that start to apply after Tezzeret's ability resolves will overwrite this effect."
    );
    supported("Mightstone's Animation");
    supported("Diminish");
    for (who, n) in [
        ("Mightstone's Animation", 4),
        ("Ensoul Artifact", 5),
        ("Tezzeret", 5),
    ] {
        let mut t = TestGame::new(2);
        // Mimic made itself a 3/3 Shapeshifter artifact creature; then Diminish
        // ("Target creature becomes 1/1 until end of turn").
        let mimic = t.battlefield(P0, "Mimic");
        activate_resolve(&mut t, P0, mimic, "Shapeshifter", "2", &[]);
        assert_eq!(t.pt(mimic), (3, 3));
        cast_resolve(&mut t, P0, "Diminish", "U", &[obj(mimic)]);
        assert_eq!(t.pt(mimic), (1, 1));
        if who == "Tezzeret" {
            let tez = t.battlefield(P0, "Tezzeret, Cruel Machinist");
            t.activate(P0, tez, 1, &[obj(mimic)]).expect("0 ability");
            t.resolve_all();
        } else {
            ensoul(&mut t, mimic, who);
        }
        assert_eq!(t.pt(mimic), (n, n), "{who} overwrites Diminish");
        cast_resolve(&mut t, P0, "Diminish", "U", &[obj(mimic)]);
        assert_eq!(t.pt(mimic), (1, 1), "a later Diminish overwrites {who}");
    }
}

#[test]
fn tezzeret_zero_keeps_abilities_and_ends_before_your_turn_begins() {
    cr!("611.2a", "613.1d", "613.4b");
    ruling!(
        "Tezzeret, Cruel Machinist",
        "Tezzeret's second ability doesn't remove any abilities the target artifact has."
    );
    ruling!(
        "Tezzeret, Cruel Machinist",
        "The artifact stops being a creature immediately before your turn begins. It will be a noncreature artifact during your untap step."
    );
    let mut t = TestGame::new(2);
    let tez = t.battlefield(P0, "Tezzeret, Cruel Machinist");
    // Mind Stone: "{T}: Add {C}." and "{1}, {T}, Sacrifice this artifact: Draw a card."
    let stone = t.battlefield(P0, "Mind Stone");
    t.activate(P0, tez, 1, &[obj(stone)]).expect("0 ability");
    t.resolve_all();
    assert!(is_creature(&t, stone));
    assert!(is_type(&t, stone, CardType::Artifact));
    assert_eq!(t.pt(stone), (5, 5));
    assert_eq!(activated_with(&t, stone, "Draw a card"), 1);
    assert_eq!(activated_with(&t, stone, "Add {C}"), 1);
    // Through the opponent's whole turn.
    t.advance_to(P1, Step::End);
    assert!(is_creature(&t, stone));
    // P1's Meekstone: "Creatures with power 3 or greater don't untap during their
    // controllers' untap steps." The tapped Mind Stone untaps in P0's untap step: it's no
    // longer a 5/5 creature then.
    t.battlefield(P1, "Meekstone");
    t.g.tap(stone);
    t.advance_to(P0, Step::Upkeep);
    assert!(!is_creature(&t, stone));
    assert!(!t.obj_now(stone).tapped);
}
