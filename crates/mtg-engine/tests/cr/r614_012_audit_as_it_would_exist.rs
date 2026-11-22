//! CR 614.12 audit (review): which replacement effects apply to a permanent entering the
//! battlefield is checked against the permanent as it would exist there, taking into
//! account the replacement effects that have already modified how it enters: a type it
//! enters with because of one ("as a Mutant", an "as this enters, it becomes" choice) makes
//! an effect for permanents of that type applicable (616.2).

use crate::r614_audit_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::*;

/// P0 controls Metallic Mimic naming `ty`.
fn mimic_naming(t: &mut TestGame, ty: &str) -> ObjectId {
    supported("Metallic Mimic");
    let i = mtg_engine::types::subtype_lists()
        .creature
        .iter()
        .position(|x| x == ty)
        .unwrap_or_else(|| panic!("no creature type {ty}"));
    t.answer(P0, DecisionKind::Option, Answer::Index(i));
    let mimic = t.enter(P0, "Metallic Mimic");
    t.settle();
    assert!(t.obj_now(mimic).chars.has_subtype(ty));
    mimic
}

#[test]
fn a_creature_entering_as_a_mutant_under_master_biomancer_gets_metallic_mimics_counter() {
    cr!("614.12", "616.2", "614.1c");
    supported("Master Biomancer");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Master Biomancer");
    mimic_naming(&mut t, "Mutant");
    let bears = cast_resolve(&mut t, P0, "Grizzly Bears", &[]);
    let bears = t.g.current(bears);
    assert!(t.obj_now(bears).chars.has_subtype("Mutant"));
    // Master Biomancer's two (its power), then, entering as a Mutant, Metallic Mimic's.
    assert_eq!(plus_counters(&t, bears), 3);
    // Spike Feeder: its own two, Master Biomancer's two and Metallic Mimic's one.
    let feeder = cast_resolve(&mut t, P0, "Spike Feeder", &[]);
    assert_eq!(plus_counters(&t, feeder), 2 + 2 + 1);
}

#[test]
fn primal_clay_choosing_the_wall_enters_as_one_for_metallic_mimic() {
    cr!("614.12", "616.2", "614.1c", "707.2");
    supported("Primal Clay");
    for (mode, counters) in [(2usize, 1u32), (0, 0)] {
        let mut t = TestGame::new(2);
        mimic_naming(&mut t, "Wall");
        // "As this creature enters, it becomes your choice of a 3/3 artifact creature, a
        // 2/2 artifact creature with flying, or a 1/6 Wall artifact creature with
        // defender in addition to its other types."
        t.answer(P0, DecisionKind::Option, Answer::Index(mode));
        let clay = cast_resolve(&mut t, P0, "Primal Clay", &[]);
        let clay = t.g.current(clay);
        assert_eq!(
            t.obj_now(clay).chars.has_subtype("Wall"),
            mode == 2,
            "mode {mode}"
        );
        assert_eq!(plus_counters(&t, clay), counters, "mode {mode}");
        let (p, tough) = if mode == 2 { (1, 6) } else { (3, 3) };
        assert_eq!(t.pt(clay), (p + counters as i32, tough + counters as i32));
    }
}

#[test]
fn containment_priest_exiles_a_noncreature_card_entering_as_a_creature() {
    cr!("614.12", "614.1a");
    ruling!(
        "Containment Priest",
        "If a noncreature card wasn't cast and is entering the battlefield as a creature (due to an effect such as that of March of the Machines), it will be exiled."
    );
    supported("Containment Priest");
    supported("March of the Machines");
    supported("Brilliant Restoration");
    for march in [false, true] {
        let mut t = TestGame::new(2);
        t.battlefield(P1, "Containment Priest");
        if march {
            t.battlefield(P1, "March of the Machines");
        }
        // "Return all artifact and enchantment cards from your graveyard to the
        // battlefield": Mind Stone, a noncreature artifact card, would be a 2/2 artifact
        // creature on the battlefield under March of the Machines.
        let stone = t.graveyard(P0, "Mind Stone");
        cast_resolve(&mut t, P0, "Brilliant Restoration", &[]);
        let stone = t.g.current(stone);
        assert_eq!(t.in_exile("Mind Stone"), march, "march {march}");
        assert_eq!(t.on_battlefield(stone), !march, "march {march}");
    }
}

#[test]
fn containment_priest_doesnt_exile_a_creature_card_entering_as_a_noncreature() {
    cr!("614.12", "614.1a");
    ruling!(
        "Containment Priest",
        "Conversely, if a creature card wasn't cast and is entering the battlefield as a noncreature permanent (for example, Heliod, Sun-Crowned with insufficient devotion), it won't be exiled."
    );
    supported("Containment Priest");
    supported("Heliod, Sun-Crowned");
    supported("Show and Tell");
    // P0's devotion to white on the battlefield would be Heliod's own 1: it isn't a
    // creature there. Grizzly Bears, put onto the battlefield the same way, is exiled.
    for (name, exiled) in [("Heliod, Sun-Crowned", false), ("Grizzly Bears", true)] {
        let mut t = TestGame::new(2);
        t.battlefield(P1, "Containment Priest");
        let card = t.hand(P0, name);
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[obj(card)]);
        cast_resolve(&mut t, P0, "Show and Tell", &[]);
        let card = t.g.current(card);
        assert_eq!(t.in_exile(name), exiled, "{name}");
        assert_eq!(t.on_battlefield(card), !exiled, "{name}");
        if !exiled {
            assert!(!t.obj_now(card).is_creature());
        }
    }
}
