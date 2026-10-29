//! CR 613.8: dependency between continuous effects within a layer, dependency loops,
//! and reevaluation after each effect is applied. Also CR 613.3 (characteristic-defining
//! abilities first).

use crate::r609_common::*;
use mtg_engine::ability::*;
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

fn blood_moon() -> CardDef {
    permanent(
        "Blood Moon",
        &[CardType::Enchantment],
        vec![continuous(
            Filter::and(vec![
                Filter::Type(CardType::Land),
                Filter::not(Filter::Supertype(Supertype::Basic)),
            ]),
            vec![Modification::SetBasicLandType(vec!["Mountain".into()])],
        )],
    )
}

fn urborg() -> CardDef {
    let mut d = permanent(
        "Urborg, Tomb of Yawgmoth",
        &[CardType::Land],
        vec![continuous(
            Filter::Type(CardType::Land),
            vec![Modification::AddSubtypes(vec!["Swamp".into()])],
        )],
    );
    d.faces[0].chars.supertypes.insert(Supertype::Legendary);
    d
}

#[test]
fn dependency_on_existence_overrides_timestamps() {
    // CR 613.8a: Urborg's "each land is a Swamp" depends on Blood Moon's effect, which
    // removes Urborg's ability (CR 305.7) — both apply in layer 4. The dependency overrides
    // timestamp order (CR 613.8), so Blood Moon applies first whichever is newer, and
    // Urborg's effect then doesn't exist.
    cr!("613.8", "613.8a", "613.8b", "613.1d");
    for urborg_first in [true, false] {
        let mut t = TestGame::new(2);
        let (u, _moon) = if urborg_first {
            let u = t.custom(P0, urborg(), Zone::Battlefield);
            (u, t.custom(P1, blood_moon(), Zone::Battlefield))
        } else {
            let m = t.custom(P1, blood_moon(), Zone::Battlefield);
            (t.custom(P0, urborg(), Zone::Battlefield), m)
        };
        let plains = t.battlefield(P0, "Plains");
        t.recompute();
        assert!(t.obj_now(u).chars.has_subtype("Mountain"));
        assert!(!t.obj_now(u).chars.has_subtype("Swamp"));
        assert!(!t.obj_now(plains).chars.has_subtype("Swamp"));
        assert!(t.obj_now(plains).chars.has_subtype("Plains"));
    }
}

#[test]
fn dependency_on_what_an_effect_applies_to() {
    // CR 613.8a(b), 613.8b: "All white creatures are blue" (older) depends on a newer
    // "target creature becomes white" effect, because applying the latter changes what the
    // former applies to. It waits until just after that effect has been applied.
    cr!("613.8a", "613.8b", "613.1e");
    let mut t = TestGame::new(2);
    t.custom(
        P0,
        permanent(
            "Blue Wash",
            &[CardType::Enchantment],
            vec![continuous(
                Filter::and(vec![Filter::creature(), Filter::Color(Color::White)]),
                vec![Modification::SetColors(colors(&[Color::Blue]))],
            )],
        ),
        Zone::Battlefield,
    );
    let c = t.custom(
        P0,
        creature("Red Bear", 2, 2, &[Color::Red]),
        Zone::Battlefield,
    );
    modify_target(
        &mut t,
        P0,
        c,
        vec![Modification::SetColors(colors(&[Color::White]))],
    );
    assert_eq!(color_of(&t, c), colors(&[Color::Blue]));
}

#[test]
fn dependency_loop_uses_timestamp_order() {
    // CR 613.8b: "Artifacts are creatures" and "Noncreature permanents are artifacts" each
    // depend on the other (a loop), so they're applied in timestamp order.
    cr!("613.8b", "613.8a");
    let artifacts_are_creatures = || {
        permanent(
            "Animator",
            &[CardType::Enchantment],
            vec![continuous(
                Filter::Type(CardType::Artifact),
                vec![Modification::AddTypes(vec![CardType::Creature])],
            )],
        )
    };
    let noncreatures_are_artifacts = || {
        permanent(
            "Artificer",
            &[CardType::Enchantment],
            vec![continuous(
                Filter::and(vec![Filter::Permanent, Filter::not(Filter::creature())]),
                vec![Modification::AddTypes(vec![CardType::Artifact])],
            )],
        )
    };
    for animator_first in [true, false] {
        let mut t = TestGame::new(2);
        let x = t.custom(
            P0,
            permanent("Rock", &[CardType::Land], vec![]),
            Zone::Battlefield,
        );
        let y = t.custom(
            P0,
            permanent("Relic", &[CardType::Artifact], vec![]),
            Zone::Battlefield,
        );
        if animator_first {
            t.custom(P0, artifacts_are_creatures(), Zone::Battlefield);
            t.custom(P0, noncreatures_are_artifacts(), Zone::Battlefield);
        } else {
            t.custom(P0, noncreatures_are_artifacts(), Zone::Battlefield);
            t.custom(P0, artifacts_are_creatures(), Zone::Battlefield);
        }
        t.recompute();
        assert!(t.obj_now(y).is_creature());
        assert!(t.obj_now(x).chars.is(CardType::Artifact));
        // Animator first: it animates only the original artifact; the land becomes an
        // artifact afterwards and stays a noncreature. Artificer first: the land becomes an
        // artifact and is then animated.
        assert_eq!(t.obj_now(x).is_creature(), !animator_first);
    }
}

#[test]
fn order_is_reevaluated_after_each_effect() {
    // CR 613.8c: after "target creature becomes a Goblin" is applied, "Elves are Soldiers"
    // (older) becomes dependent on "Goblins are Elves" (newer), which was independent of it
    // before. So the creature ends up a Goblin Elf Soldier.
    cr!("613.8c", "613.8b");
    let mut t = TestGame::new(2);
    let c = t.custom(P0, creature("Human", 1, 1, &[]), Zone::Battlefield);
    modify_target(
        &mut t,
        P0,
        c,
        vec![Modification::AddSubtypes(vec!["Goblin".into()])],
    );
    t.custom(
        P0,
        permanent(
            "Drill Sergeant",
            &[CardType::Enchantment],
            vec![continuous(
                Filter::Subtype("Elf".into()),
                vec![Modification::AddSubtypes(vec!["Soldier".into()])],
            )],
        ),
        Zone::Battlefield,
    );
    t.custom(
        P0,
        permanent(
            "Elf Mask",
            &[CardType::Enchantment],
            vec![continuous(
                Filter::Subtype("Goblin".into()),
                vec![Modification::AddSubtypes(vec!["Elf".into()])],
            )],
        ),
        Zone::Battlefield,
    );
    t.recompute();
    let ch = &t.obj_now(c).chars;
    assert!(ch.has_subtype("Goblin"));
    assert!(ch.has_subtype("Elf"));
    assert!(ch.has_subtype("Soldier"));
}

#[test]
fn characteristic_defining_abilities_apply_first() {
    // CR 613.3: in layers 2–6, effects from characteristic-defining abilities apply first,
    // then all others in timestamp order. CR 613.8a(c): an effect from a CDA and one that
    // isn't are independent, so Blood Moon removing the land's CDA (layer 4) doesn't stop
    // the CDA's effect, which already applied.
    cr!("613.3", "613.8a");
    let mut t = TestGame::new(2);
    t.custom(P1, blood_moon(), Zone::Battlefield);
    let land = t.custom(
        P0,
        permanent(
            "Shifting Grove",
            &[CardType::Land],
            vec![cda(vec![Modification::AddSubtypes(vec!["Forest".into()])])],
        ),
        Zone::Battlefield,
    );
    t.recompute();
    let ch = &t.obj_now(land).chars;
    assert!(ch.has_subtype("Mountain"));
    assert!(!ch.has_subtype("Forest"));

    // Layer 5: a CDA making the object blue applies before an older "becomes red"
    // effect, so it's red.
    let mut t = TestGame::new(2);
    let c = t.custom(
        P0,
        creature_with(
            "Sea Sprite",
            1,
            1,
            &[],
            vec![cda(vec![Modification::SetColors(colors(&[Color::Blue]))])],
        ),
        Zone::Battlefield,
    );
    assert_eq!(color_of(&t, c), colors(&[Color::Blue]));
    modify_target(
        &mut t,
        P0,
        c,
        vec![Modification::SetColors(colors(&[Color::Red]))],
    );
    assert_eq!(color_of(&t, c), colors(&[Color::Red]));
}

#[test]
fn cda_power_toughness_then_setting_effects() {
    // CR 613.4a: layer 7a CDAs defining P/T apply first; 613.4b: then effects setting P/T;
    // 613.4c: then modifications and counters. A 7b effect overrides the CDA regardless of
    // timestamps.
    cr!("613.4a", "613.4b", "613.4c");
    let mut t = TestGame::new(2);
    let count = Value::Count(Filter::creature().you_control());
    let c = t.custom(
        P0,
        creature_with(
            "Crowd Avatar",
            0,
            0,
            &[],
            vec![cda(vec![Modification::CdaPT(
                Some(count.clone()),
                Some(count),
            )])],
        ),
        Zone::Battlefield,
    );
    t.custom(P0, creature("Friend", 1, 1, &[]), Zone::Battlefield);
    t.recompute();
    assert_eq!(t.pt(c), (2, 2));
    put_counters(&mut t, c, counters::PLUS1, 1);
    assert_eq!(t.pt(c), (3, 3));
    modify_target(&mut t, P0, c, vec![set_pt(0, 1)]);
    assert_eq!(t.pt(c), (1, 2));
    // A new creature doesn't matter any more (the CDA is overridden by 7b).
    t.custom(P0, creature("Friend", 1, 1, &[]), Zone::Battlefield);
    t.recompute();
    assert_eq!(t.pt(c), (1, 2));
}
