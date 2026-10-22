//! Rulings batch S27 — cost reductions (CR 118.7, 601.2f): a reduction of generic mana
//! reduces only the generic component of the total cost, never its colored mana
//! (CR 118.7a), not below zero, and it applies to alternative costs such as flashback and
//! emerge costs too (CR 118.9d). A reduction that doesn't say "may" isn't optional.

use crate::r_s01_common::*;
use crate::r_s21_common::castable;
use mtg_engine::decision::Decision;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::*;

/// Casts `card` for P0 (paying automatically with P0's lands); whether it was cast.
fn try_cast(t: &mut TestGame, card: ObjectId, method: CastMethod, targets: &[Entity]) -> bool {
    for e in targets {
        t.answer_targets(P0, &[*e]);
    }
    let ok = t.cast(P0, card).method(method).try_go().is_ok();
    t.clear_answers();
    ok
}

#[test]
fn ruby_medallion_reduces_a_flashback_cost() {
    cr!("118.9d", "601.2f", "702.34a");
    ruling!(
        "Ruby Medallion",
        "The cost reduction can apply to alternative costs such as flashback costs."
    );
    supported("Ruby Medallion");
    supported("Firebolt");
    // "Red spells you cast cost {1} less to cast." Firebolt's flashback {4}{R} costs
    // {3}{R}.
    let flashback = CastMethod::Keyword(KeywordKind::Flashback);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ruby Medallion");
    let firebolt = t.graveyard(P0, "Firebolt");
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Wastes", 3);
    assert!(try_cast(
        &mut t,
        firebolt,
        flashback.clone(),
        &[Entity::Player(P1)]
    ));
    assert_eq!(tapped_lands(&t, P0), 4);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert!(t.in_exile("Firebolt"));
    // Without the Medallion, four lands aren't enough.
    let mut t = TestGame::new(2);
    let firebolt = t.graveyard(P0, "Firebolt");
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Wastes", 3);
    assert!(!try_cast(
        &mut t,
        firebolt,
        flashback,
        &[Entity::Player(P1)]
    ));
    assert_eq!(t.zone(firebolt), Zone::Graveyard(P0));
}

#[test]
fn ruby_medallion_can_t_reduce_colored_mana() {
    cr!("118.7a", "601.2f");
    ruling!(
        "Ruby Medallion",
        "The ability can't reduce the amount of colored mana you pay for a spell. It reduces only the generic mana component of that cost."
    );
    supported("Ruby Medallion");
    // Lightning Bolt ({R}) still costs {R}; Ball Lightning ({R}{R}{R}) still costs
    // {R}{R}{R}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ruby Medallion");
    let bolt = t.hand(P0, "Lightning Bolt");
    assert!(!castable(&mut t, P0, bolt));
    t.lands(P0, "Mountain", 1);
    assert!(try_cast(
        &mut t,
        bolt,
        CastMethod::Normal,
        &[Entity::Player(P1)]
    ));
    t.resolve_all();
    let ball = t.hand(P0, "Ball Lightning");
    t.lands(P0, "Mountain", 2);
    t.lands(P0, "Wastes", 3);
    assert!(!try_cast(&mut t, ball, CastMethod::Normal, &[]));
    t.lands(P0, "Mountain", 1);
    assert!(try_cast(&mut t, ball, CastMethod::Normal, &[]));
}

#[test]
fn helm_of_awakening_s_reduction_isn_t_optional() {
    cr!("601.2f", "118.7");
    ruling!(
        "Helm of Awakening",
        "The lower cost is not optional like with some other cost reducers."
    );
    supported("Helm of Awakening");
    // "Spells cost {1} less to cast." Grizzly Bears ({1}{G}) costs {G}: only one of the
    // two Forests is tapped, and no choice is offered.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Helm of Awakening");
    t.lands(P0, "Forest", 2);
    let bears = t.hand(P0, "Grizzly Bears");
    let from = t.asked().len();
    assert!(try_cast(&mut t, bears, CastMethod::Normal, &[]));
    assert_eq!(tapped_lands(&t, P0), 1);
    assert!(!t.asked()[from..].iter().any(|(_, d)| matches!(
        d,
        Decision::YesNo { .. } | Decision::OptionalCost { .. } | Decision::ChooseOption { .. }
    )));
}

#[test]
fn helm_of_awakening_lowers_a_cost_to_zero_but_not_below() {
    cr!("601.2f", "118.7", "118.7a");
    ruling!(
        "Helm of Awakening",
        "This can lower the cost to zero, but not below zero."
    );
    supported("Helm of Awakening");
    // Two Helms: Bonesplitter ({1}) costs {0} — no mana is left over — and Grizzly Bears
    // ({1}{G}) costs {G}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Helm of Awakening");
    t.battlefield(P0, "Helm of Awakening");
    let blade = t.hand(P0, "Bonesplitter");
    assert!(try_cast(&mut t, blade, CastMethod::Normal, &[]));
    assert_eq!(t.g.player(P0).mana_pool.total(), 0);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Bonesplitter").len(), 1);
    let bears = t.hand(P0, "Grizzly Bears");
    assert!(!castable(&mut t, P0, bears));
    t.lands(P0, "Forest", 1);
    assert!(try_cast(&mut t, bears, CastMethod::Normal, &[]));
}

#[test]
fn thornscape_familiar_can_t_reduce_the_colored_part_of_a_cost() {
    cr!("118.7a", "601.2f");
    ruling!(
        "Thornscape Familiar",
        "Can never affect the colored part of the cost."
    );
    supported("Thornscape Familiar");
    // "Red spells and white spells you cast cost {1} less to cast." Lightning Bolt still
    // costs {R}; Hill Giant ({3}{R}) costs {2}{R}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Thornscape Familiar");
    let bolt = t.hand(P0, "Lightning Bolt");
    assert!(!castable(&mut t, P0, bolt));
    t.lands(P0, "Mountain", 1);
    assert!(try_cast(
        &mut t,
        bolt,
        CastMethod::Normal,
        &[Entity::Player(P1)]
    ));
    t.resolve_all();
    let giant = t.hand(P0, "Hill Giant");
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Wastes", 2);
    assert!(try_cast(&mut t, giant, CastMethod::Normal, &[]));
    assert_eq!(tapped_lands(&t, P0), 4);
}

#[test]
fn the_magic_mirror_s_reduction_leaves_its_blue_mana() {
    cr!("118.7a", "601.2f");
    ruling!(
        "The Magic Mirror",
        "The cost reduction ability reduces only the generic mana in the relic's cost. The colored mana must still be paid."
    );
    supported("The Magic Mirror");
    // "This spell costs {1} less to cast for each instant and sorcery card in your
    // graveyard." With nine in the graveyard, {6}{U}{U}{U} costs {U}{U}{U}.
    let mirror_with = |islands: usize, wastes: usize| {
        let mut t = TestGame::new(2);
        for _ in 0..9 {
            t.graveyard(P0, "Lightning Bolt");
        }
        t.lands(P0, "Island", islands);
        t.lands(P0, "Wastes", wastes);
        let mirror = t.hand(P0, "The Magic Mirror");
        let ok = try_cast(&mut t, mirror, CastMethod::Normal, &[]);
        (ok, tapped_lands(&t, P0))
    };
    assert_eq!(mirror_with(3, 0), (true, 3));
    assert_eq!(mirror_with(2, 6).0, false);
}

#[test]
fn planar_gate_reduces_only_the_generic_mana_of_a_creature_spell() {
    cr!("118.7a", "601.2f");
    ruling!(
        "Planar Gate",
        "Only reduces the generic mana portion of a spell's cost. If the cost does not include generic mana or includes less than {2}, you get a reduced or null effect from this card."
    );
    supported("Planar Gate");
    // "Creature spells you cast cost {2} less to cast." Llanowar Elves ({G}) and Grizzly
    // Bears ({1}{G}) each cost {G}; Hill Giant ({3}{R}) costs {1}{R}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Planar Gate");
    t.lands(P0, "Forest", 2);
    let elves = t.hand(P0, "Llanowar Elves");
    assert!(try_cast(&mut t, elves, CastMethod::Normal, &[]));
    assert_eq!(tapped_lands(&t, P0), 1);
    t.resolve_all();
    let bears = t.hand(P0, "Grizzly Bears");
    assert!(try_cast(&mut t, bears, CastMethod::Normal, &[]));
    assert_eq!(tapped_lands(&t, P0), 2);
    t.resolve_all();
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Wastes", 2);
    let giant = t.hand(P0, "Hill Giant");
    assert!(try_cast(&mut t, giant, CastMethod::Normal, &[]));
    assert_eq!(tapped_lands(&t, P0), 4);
}

#[test]
fn herald_of_kozilek_reduces_an_emerge_cost() {
    cr!("118.9d", "601.2f", "702.119a");
    ruling!(
        "Herald of Kozilek",
        "The cost reduction can apply to alternative costs."
    );
    supported("Herald of Kozilek");
    supported("Elder Deep-Fiend");
    // "Colorless spells you cast cost {1} less to cast." Elder Deep-Fiend ({8},
    // colorless) cast for its emerge cost {5}{U}{U} sacrificing Grizzly Bears (mana value
    // 2) costs {2}{U}{U}.
    let emerge = CastMethod::Keyword(KeywordKind::Emerge);
    let deep_fiend = |herald: bool, wastes: usize| {
        let mut t = TestGame::new(2);
        if herald {
            t.battlefield(P0, "Herald of Kozilek");
        }
        let bears = t.battlefield(P0, "Grizzly Bears");
        t.lands(P0, "Island", 2);
        t.lands(P0, "Wastes", wastes);
        let fiend = t.hand(P0, "Elder Deep-Fiend");
        t.answer_choose(P0, &[Entity::Object(bears)]);
        try_cast(&mut t, fiend, emerge.clone(), &[])
    };
    assert!(deep_fiend(true, 2));
    assert!(!deep_fiend(false, 2));
    assert!(deep_fiend(false, 3));
}
