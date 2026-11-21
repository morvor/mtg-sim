//! Rulings batch P107 — spells: "up to" targets and illegal targets (CR 608.2b), "unless
//! its controller pays", searching for a card with a land type or a mana value, an
//! additional discard cost, putting a land onto the battlefield (not playing it), modes
//! chosen in any number, and the +1/+1 / -1/-1 counter state-based action.

use crate::r_p107_common::*;
use crate::r_s02_common::can_cast;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// `p`'s mana pool gets `n` of each listed type.
fn pool(t: &mut TestGame, p: PlayerId, types: &[(ManaType, u32)]) {
    for (ty, n) in types {
        mana(t, p, *ty, *n);
    }
}

#[test]
fn adverse_conditions_with_no_targets_still_makes_a_scion_but_not_with_all_targets_illegal() {
    cr!("608.2b", "115.1");
    ruling!(
        "Adverse Conditions",
        "You can cast Adverse Conditions with no targets. When it resolves, you'll get an Eldrazi Scion."
    );
    let mut t = TestGame::new(2);
    pool(&mut t, P0, &[(ManaType::U, 1), (ManaType::C, 3)]);
    let spell = cast_from_hand(&mut t, P0, "Adverse Conditions", &[]);
    t.resolve_all();
    assert!(resolved(&t, spell));
    assert_eq!(with_subtype(&t, P0, "Scion").len(), 1);
    // One target, which becomes illegal: it doesn't resolve, no Scion.
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P1, "Grizzly Bears");
    pool(&mut t, P0, &[(ManaType::U, 1), (ManaType::C, 3)]);
    t.answer_targets(P0, &[obj(bear)]);
    let c = t.hand(P0, "Adverse Conditions");
    let spell = t.cast_with(P0, c, &[]).unwrap();
    destroy(&mut t, bear);
    t.resolve_all();
    assert!(!resolved(&t, spell));
    assert!(with_subtype(&t, P0, "Scion").is_empty());
}

#[test]
fn abstruse_interference_gives_a_scion_even_if_the_tax_is_paid() {
    cr!("608.2c");
    ruling!(
        "Abstruse Interference",
        "You get the Eldrazi Scion even if the controller of the spell pays {1}."
    );
    let mut t = TestGame::new(2);
    pool(&mut t, P1, &[(ManaType::R, 1)]);
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    let spell = t.cast_with(P1, bolt, &[Entity::Player(P0)]).unwrap();
    pool(&mut t, P0, &[(ManaType::U, 1), (ManaType::C, 2)]);
    t.answer_yes(P1, true);
    cast_from_hand(&mut t, P0, "Abstruse Interference", &[obj(spell)]);
    t.resolve_all();
    assert_eq!(t.life(P0), 17);
    assert_eq!(with_subtype(&t, P0, "Scion").len(), 1);
}

#[test]
fn three_visits_finds_any_card_with_the_forest_land_type() {
    cr!("205.3i", "701.23a");
    ruling!(
        "Three Visits",
        "Three Visits allows you to search your library for any card with the land type Forest, not just a card with the name Forest."
    );
    let mut t = TestGame::new(2);
    let pool_ = t.library_top(P0, "Breeding Pool");
    t.answer_choose(P0, &[obj(pool_)]);
    pool(&mut t, P0, &[(ManaType::G, 1), (ManaType::C, 1)]);
    cast_from_hand(&mut t, P0, "Three Visits", &[]);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Breeding Pool").len(), 1);
}

#[test]
fn wargate_with_x_0_finds_lands_and_zero_cost_permanents() {
    cr!("202.3a", "202.3e", "107.3b");
    ruling!(
        "Wargate",
        "You may always find a permanent card with mana value 0."
    );
    for name in ["Forest", "Ornithopter"] {
        let mut t = TestGame::new(2);
        let target = t.library_top(P0, name);
        t.library_top(P0, "Grizzly Bears");
        t.answer_choose(P0, &[obj(target)]);
        pool(
            &mut t,
            P0,
            &[(ManaType::G, 1), (ManaType::W, 1), (ManaType::U, 1)],
        );
        let c = t.hand(P0, "Wargate");
        t.cast(P0, c).x(0).go();
        t.resolve_all();
        assert_eq!(t.named_on_battlefield(name).len(), 1, "{name}");
    }
    // With X = 0, a card with mana value 2 isn't among the choices.
    let mut t = TestGame::new(2);
    t.library_top(P0, "Grizzly Bears");
    pool(
        &mut t,
        P0,
        &[(ManaType::G, 1), (ManaType::W, 1), (ManaType::U, 1)],
    );
    let c = t.hand(P0, "Wargate");
    t.cast(P0, c).x(0).go();
    t.resolve_all();
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
}

#[test]
fn seize_the_spoils_discards_exactly_one_card() {
    cr!("601.2b", "601.2h");
    ruling!(
        "Seize the Spoils",
        "You must discard exactly one card to cast Seize the Spoils; you can't cast it without discarding a card, and you can't discard additional cards."
    );
    // Alone in hand: it can't be cast.
    let mut t = TestGame::new(2);
    pool(&mut t, P0, &[(ManaType::R, 1), (ManaType::C, 2)]);
    let seize = t.hand(P0, "Seize the Spoils");
    assert!(!can_cast(&mut t, P0, seize, CastMethod::Normal));
    // With three other cards: exactly one is discarded.
    t.hand(P0, "Plains");
    t.hand(P0, "Island");
    t.hand(P0, "Swamp");
    assert!(can_cast(&mut t, P0, seize, CastMethod::Normal));
    t.cast(P0, seize).go();
    assert_eq!(t.graveyard_size(P0), 1);
    assert_eq!(t.hand_size(P0), 2);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 4);
    assert_eq!(with_subtype(&t, P0, "Treasure").len(), 1);
}

#[test]
fn lessons_from_life_puts_a_land_even_after_the_land_drop() {
    cr!("305.2", "305.4");
    ruling!(
        "Lessons from Life",
        "Lessons from Life's effect doesn't count as playing a land."
    );
    let mut t = TestGame::new(2);
    let first = t.hand(P0, "Forest");
    t.play_land(P0, first).unwrap();
    let second = t.hand(P0, "Island");
    t.answer_choose(P0, &[obj(second)]);
    t.answer_yes(P0, true);
    pool(
        &mut t,
        P0,
        &[(ManaType::G, 1), (ManaType::U, 1), (ManaType::C, 2)],
    );
    cast_from_hand(&mut t, P0, "Lessons from Life", &[]);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Island").len(), 1);
    assert!(t.obj_now(second).tapped);
    // Still one land played this turn.
    let third = t.hand(P0, "Plains");
    assert!(t.play_land(P0, third).is_err());
}

#[test]
fn new_horizons_can_be_cast_without_creatures() {
    cr!("603.3d", "303.4a");
    ruling!(
        "New Horizons",
        "You can cast New Horizons even if you control no creatures."
    );
    let mut t = TestGame::new(2);
    let land = t.battlefield(P0, "Forest");
    pool(&mut t, P0, &[(ManaType::G, 1), (ManaType::C, 2)]);
    let c = t.hand(P0, "New Horizons");
    assert!(can_cast(&mut t, P0, c, CastMethod::Normal));
    t.cast_with(P0, c, &[obj(land)]).unwrap();
    t.resolve_all();
    let aura = t.named_on_battlefield("New Horizons");
    assert_eq!(aura.len(), 1);
    assert_eq!(t.obj_now(aura[0]).attached_to, Some(obj(land)));
}

#[test]
fn rankle_and_torbran_modes_in_order_and_sacrifices_chosen_in_turn_order() {
    cr!("700.2d", "101.4", "608.2c");
    ruling!(
        "Rankle and Torbran",
        "To resolve the second mode, starting with you (or, in unusual cases, the player whose turn it is if not you) and proceeding in turn order, each player chooses a creature they control to sacrifice. Then those creatures are sacrificed at the same time."
    );
    ruling!(
        "Rankle and Torbran",
        "You may choose none of the modes, some of them, or all of them. Any modes chosen will happen in order."
    );
    // Modes 1 and 2: each player creates a Treasure, then each sacrifices a creature. P1's
    // Blood Artist ("Whenever this creature or another creature dies, target player loses
    // 1 life and you gain 1 life") sees both creatures die at the same time.
    let mut t = TestGame::new(2);
    let rt = t.battlefield(P0, "Rankle and Torbran");
    let bear = t.battlefield(P0, "Grizzly Bears");
    let artist = t.battlefield(P1, "Blood Artist");
    t.answer(
        P0,
        DecisionKind::Modes,
        mtg_engine::decision::Answer::Indices(vec![0, 1]),
    );
    t.answer_choose(P0, &[obj(bear)]);
    t.answer_targets(P1, &[Entity::Player(P0)]);
    t.answer_targets(P1, &[Entity::Player(P0)]);
    let from = t.asked().len();
    crate::r_s01_common::attack_with(&mut t, &[(rt, Entity::Player(P1))]);
    t.advance_to(P0, mtg_engine::turn::Step::EndOfCombat);
    t.resolve_all();
    assert_eq!(t.life(P1), 20 - 3 + 2);
    assert_eq!(with_subtype(&t, P0, "Treasure").len(), 1);
    assert_eq!(with_subtype(&t, P1, "Treasure").len(), 1);
    assert!(!t.on_battlefield(bear) && !t.on_battlefield(artist));
    assert!(t.on_battlefield(rt));
    // P0 chose first, then P1.
    let choosers: Vec<PlayerId> = t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, mtg_engine::decision::Decision::ChooseEntities { .. }))
        .map(|(p, _)| *p)
        .collect();
    assert_eq!(choosers, vec![P0, P1]);
    // Both died at once: Blood Artist triggered for both (two drains).
    assert_eq!(t.life(P0), 18);
}

#[test]
fn rankle_and_torbran_may_choose_no_modes() {
    cr!("700.2d");
    let mut t = TestGame::new(2);
    let rt = t.battlefield(P0, "Rankle and Torbran");
    t.answer(
        P0,
        DecisionKind::Modes,
        mtg_engine::decision::Answer::Indices(vec![]),
    );
    crate::r_s01_common::attack_with(&mut t, &[(rt, Entity::Player(P1))]);
    t.advance_to(P0, mtg_engine::turn::Step::EndOfCombat);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert!(tokens(&t, P0).is_empty() && tokens(&t, P1).is_empty());
}

#[test]
fn unbounded_potential_counters_annihilate_in_pairs() {
    cr!("704.5q", "122.3");
    ruling!(
        "Unbounded Potential",
        "If a permanent has +1/+1 counters and -1/-1 counters on it, they're removed in pairs as a state-based action"
    );
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    put_counters(&mut t, giant, counters::MINUS1, 2);
    pool(&mut t, P0, &[(ManaType::W, 1), (ManaType::C, 1)]);
    let c = t.hand(P0, "Unbounded Potential");
    t.answer_targets(P0, &[obj(giant)]);
    t.cast(P0, c).modes(&[0]).go();
    t.resolve_all();
    assert_eq!(t.counters(giant, counters::PLUS1), 0);
    assert_eq!(t.counters(giant, counters::MINUS1), 1);
    assert_eq!(t.pt(giant), (2, 2));
    assert_eq!(t.zone(giant), Zone::Battlefield);
}
