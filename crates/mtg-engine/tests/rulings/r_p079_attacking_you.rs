//! Rulings batch P079 — "target creature [without flying] that's attacking you [or a
//! planeswalker you control]" (pattern `r508_that_is_attacking_you`): a creature attacking
//! a planeswalker isn't attacking you (CR 506.2, 508.1b), and the target is checked again
//! on resolution (CR 608.2b).

use crate::r_p079_common::*;
use crate::r_s01_common::{attack_with, supported};
use crate::r_s04_common::ability_targets;
use crate::r_s06_common::activate_containing;
use mtg_engine::testing::*;
use mtg_engine::*;

/// P0 attacks with a Grizzly Bears at P1, a Hill Giant at P1's planeswalker and a Wind
/// Drake (flying) at P1. Returns (bears, giant, drake, planeswalker).
fn three_attackers(t: &mut TestGame) -> (ObjectId, ObjectId, ObjectId, ObjectId) {
    let pw = t.battlefield(P1, "Jace Beleren");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let drake = t.battlefield(P0, "Wind Drake");
    attack_with(
        t,
        &[
            (bears, Entity::Player(P1)),
            (giant, obj(pw)),
            (drake, Entity::Player(P1)),
        ],
    );
    (bears, giant, drake, pw)
}

#[test]
fn creatures_attacking_you_as_targets() {
    cr!("506.2", "508.1b", "115.1b");
    // (card, mana, may target the creature attacking P1's planeswalker, may target the
    // flyer)
    let cases: [(&str, &[(&str, usize)], bool, bool); 5] = [
        ("Soul Snare", &[("Plains", 1)], true, true),
        (
            "Hunting Kavu",
            &[("Mountain", 1), ("Forest", 1), ("Wastes", 1)],
            false,
            false,
        ),
        (
            "Giant Trap Door Spider",
            &[("Mountain", 1), ("Forest", 1), ("Wastes", 1)],
            false,
            false,
        ),
        ("Snow Fortress", &[("Wastes", 3)], false, false),
        ("Ice Floe", &[], false, false),
    ];
    for (name, mana, pw_ok, flyer_ok) in cases {
        supported(name);
        let mut t = TestGame::new(2);
        let src = t.battlefield(P1, name);
        // The activated ability with a target.
        let index = t
            .obj_now(src)
            .chars
            .abilities
            .iter()
            .filter_map(|a| match &a.kind {
                mtg_engine::ability::AbilityKind::Activated(x) => Some(x),
                _ => None,
            })
            .position(|a| !a.body.targets.is_empty())
            .expect("a targeted activated ability");
        for (land, n) in mana {
            t.lands(P1, land, *n);
        }
        let (bears, giant, drake, _) = three_attackers(&mut t);
        let c = ability_targets(&mut t, src, index);
        assert!(c.contains(&obj(bears)), "{name}");
        assert_eq!(
            c.contains(&obj(giant)),
            pw_ok,
            "{name}: attacking a planeswalker"
        );
        assert_eq!(c.contains(&obj(drake)), flyer_ok, "{name}: flying");
        // Activate it at the Bears.
        t.answer_targets(P1, &[obj(bears)]);
        t.g.turn.priority = Some(P1);
        let uid = t
            .obj_now(src)
            .chars
            .abilities
            .iter()
            .filter(|a| matches!(a.kind, mtg_engine::ability::AbilityKind::Activated(_)))
            .nth(index)
            .unwrap()
            .uid;
        let src_now = t.g.current(src);
        t.g.activate_ability(P1, src_now, uid)
            .unwrap_or_else(|e| panic!("{name}: {e:?}"));
        t.g.flush_events();
        t.resolve_all();
        match name {
            "Soul Snare" | "Hunting Kavu" | "Giant Trap Door Spider" => {
                assert!(!t.on_battlefield(bears), "{name}: exiled");
                assert!(t.in_exile("Grizzly Bears"), "{name}");
            }
            "Snow Fortress" => {
                assert_eq!(crate::r_s07_common::damage_on(&t, bears), 1, "{name}")
            }
            _ => assert!(crate::r_s08_common::is_tapped(&t, bears), "{name}"),
        }
        if matches!(name, "Hunting Kavu" | "Giant Trap Door Spider") {
            assert!(t.in_exile(name), "{name} exiles itself too");
        }
    }
}

#[test]
fn soul_snare_target_attacking_a_planeswalker_that_left_is_illegal() {
    cr!("506.4", "608.2b");
    ruling!(
        "Soul Snare",
        "If Soul Snare's ability targets a creature that's attacking a planeswalker you control, but that planeswalker is no longer on the battlefield as the ability tries to resolve, the attacking creature will be an illegal target"
    );
    let mut t = TestGame::new(2);
    let snare = t.battlefield(P1, "Soul Snare");
    t.lands(P1, "Plains", 1);
    let (_, giant, _, pw) = three_attackers(&mut t);
    t.answer_targets(P1, &[obj(giant)]);
    activate_containing(&mut t, P1, snare, "Exile").unwrap();
    crate::r_s02_common::destroy(&mut t, pw);
    t.resolve_all();
    assert!(t.on_battlefield(giant));
}

#[test]
fn hunting_kavu_target_that_gains_flying_makes_it_not_resolve() {
    cr!("608.2b", "115.1b");
    ruling!(
        "Hunting Kavu",
        "Being “without flying” is a targeting restriction. The ability won’t resolve if the target gains flying before the ability resolves."
    );
    let mut t = TestGame::new(2);
    let kavu = t.battlefield(P1, "Hunting Kavu");
    t.lands(P1, "Mountain", 1);
    t.lands(P1, "Forest", 1);
    t.lands(P1, "Wastes", 1);
    let (bears, _, _, _) = three_attackers(&mut t);
    t.answer_targets(P1, &[obj(bears)]);
    activate_containing(&mut t, P1, kavu, "Exile").unwrap();
    cast_card(&mut t, P0, "Jump", &[obj(bears)]).unwrap();
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert!(
        t.on_battlefield(kavu),
        "the ability didn't resolve: the Kavu stays too"
    );
}
