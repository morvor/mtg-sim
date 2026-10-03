//! Rulings batch S25 — a copy of a modal spell has the same mode or modes as the original
//! (CR 707.10, 700.2): no modes are chosen for the copy, although new targets may be.

use crate::r_s01_common::supported;
use crate::r_s06_common::activate_containing;
use crate::r_s07_common::chosen_modes;
use crate::r_s25_common::*;
use mtg_engine::decision::Decision;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

fn modes_asked_since(t: &TestGame, from: usize) -> usize {
    t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseModes { .. }))
        .count()
}

/// P1 controls two Grizzly Bears. After `setup`, P0 casts Izzet Charm choosing its second
/// mode ("deals 2 damage to target creature") at the first Bears; `start` puts on the
/// stack what copies it (nothing when a triggered ability already does), which resolves:
/// the copy gets the second Bears as its new target. The copy has the same mode — none is
/// chosen for it — and both Bears die.
fn izzet_charm_copy_keeps_its_mode(
    setup: impl FnOnce(&mut TestGame),
    start: impl FnOnce(&mut TestGame, ObjectId),
) {
    supported("Izzet Charm");
    let mut t = TestGame::new(2);
    setup(&mut t);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    lands_for_cost(&mut t, P0, "Izzet Charm");
    let card = t.hand(P0, "Izzet Charm");
    let charm = t.cast(P0, card).modes(&[1]).target(a).go();
    let from = t.asked().len();
    start(&mut t, charm);
    change_copy_targets(&mut t, P0, &[Some(Entity::Object(b))]);
    t.resolve();
    let copy = spell_copies(&t)[0];
    assert_eq!(chosen_modes(&t, copy), vec![1]);
    assert_eq!(chosen_modes(&t, copy), chosen_modes(&t, charm));
    assert_eq!(targets_of(&t, copy), vec![Entity::Object(b)]);
    assert_eq!(modes_asked_since(&t, from), 0);
    t.resolve_all();
    assert!(!t.on_battlefield(a));
    assert!(!t.on_battlefield(b));
}

#[test]
fn a_delayed_copy_has_the_same_modes() {
    cr!("707.10", "700.2");
    ruling!(
        "Galvanic Iteration",
        "If the spell that's copied is modal (that is, it says \"Choose one —\" or the like), the copy will have the same mode or modes. You can't choose different ones."
    );
    supported("Galvanic Iteration");
    izzet_charm_copy_keeps_its_mode(
        |t| {
            cast_new(t, P0, "Galvanic Iteration", &[]);
            t.resolve_all();
        },
        |_, _| {},
    );
}

#[test]
fn a_copy_by_a_spell_has_the_same_mode() {
    cr!("707.10", "700.2");
    ruling!(
        "Twincast",
        "If the spell that's copied is modal (that is, it says \"Choose one —\" or the like), the copy will have the same mode. A different mode can't be chosen."
    );
    supported("Twincast");
    izzet_charm_copy_keeps_its_mode(
        |_| {},
        |t, charm| {
            cast_new(t, P0, "Twincast", &[Entity::Object(charm)]);
        },
    );
}

#[test]
fn a_copy_by_fury_storm_has_the_same_mode() {
    cr!("707.10", "700.2");
    ruling!(
        "Fury Storm",
        "If the spell that’s copied is modal (that is, it says “Choose one —” or the like), the copy will have the same mode. A different mode can’t be chosen."
    );
    supported("Fury Storm");
    // "Copy target instant or sorcery spell. You may choose new targets for the copy."
    // (Its cast trigger copies it once per commander cast: none here.)
    izzet_charm_copy_keeps_its_mode(
        |_| {},
        |t, charm| {
            cast_new(t, P0, "Fury Storm", &[Entity::Object(charm)]);
            t.resolve();
        },
    );
}

#[test]
fn a_doublecast_copy_has_the_same_modes() {
    cr!("707.10", "700.2");
    ruling!(
        "Doublecast",
        "If the spell that’s copied is modal (that is, it says “Choose one —” or the like), the copy will have the same mode or modes. You can’t choose different ones."
    );
    supported("Doublecast");
    izzet_charm_copy_keeps_its_mode(
        |t| {
            cast_new(t, P0, "Doublecast", &[]);
            t.resolve_all();
        },
        |_, _| {},
    );
}

#[test]
fn a_copy_from_the_graveyard_ability_has_the_same_mode() {
    cr!("707.10", "700.2");
    ruling!(
        "Geistblast",
        "If the spell being copied is modal (that is, it says \"Choose one —\" or the like), the copy will have the same mode. You can't choose a different one."
    );
    supported("Geistblast");
    // "{2}{U}, Exile this card from your graveyard: Copy target instant or sorcery spell
    // you control. You may choose new targets for the copy."
    izzet_charm_copy_keeps_its_mode(
        |t| {
            t.graveyard(P0, "Geistblast");
            t.lands(P0, "Island", 3);
        },
        |t, charm| {
            let blast = t.g.find_in_zone(mtg_engine::object::Zone::Graveyard(P0), "Geistblast")[0];
            t.answer_targets(P0, &[Entity::Object(charm)]);
            activate_containing(t, P0, blast, "Copy target").unwrap();
        },
    );
}

#[test]
fn a_dual_strike_copy_has_the_same_modes() {
    cr!("707.10", "700.2");
    ruling!(
        "Dual Strike",
        "If the spell that's copied is modal (that is, it has a bulleted list of modes), the copy will have the same mode or modes. You can't choose different ones."
    );
    supported("Dual Strike");
    izzet_charm_copy_keeps_its_mode(
        |t| {
            cast_new(t, P0, "Dual Strike", &[]);
            t.resolve_all();
        },
        |_, _| {},
    );
}

#[test]
fn a_copied_charm_has_the_same_mode() {
    cr!("707.10", "700.2");
    ruling!(
        "Grixis Charm",
        "If this spell is copied, the copy will have the same mode as the original."
    );
    supported("Grixis Charm");
    // "Choose one — • Return target permanent to its owner's hand. • Target creature gets
    // -4/-4 until end of turn. • Creatures you control get +2/+0 until end of turn."
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    lands_for_cost(&mut t, P0, "Grixis Charm");
    let card = t.hand(P0, "Grixis Charm");
    let charm = t.cast(P0, card).modes(&[1]).target(giant).go();
    let from = t.asked().len();
    cast_new(&mut t, P0, "Twincast", &[Entity::Object(charm)]);
    change_copy_targets(&mut t, P0, &[Some(Entity::Object(bears))]);
    t.resolve();
    let copy = spell_copies(&t)[0];
    assert_eq!(chosen_modes(&t, copy), vec![1]);
    assert_eq!(modes_asked_since(&t, from), 0);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Hill Giant"));
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn a_copied_command_keeps_both_modes_but_may_get_new_targets() {
    cr!("707.10", "707.10c", "700.2");
    ruling!(
        "Kolaghan's Command",
        "If a Command is copied, the effect that creates the copy will usually allow you to choose new targets for the copy, but you can't choose new modes."
    );
    supported("Kolaghan's Command");
    // Modes: "Target player discards a card." and "deals 2 damage to any target."
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    for _ in 0..3 {
        t.hand(P1, "Island");
    }
    lands_for_cost(&mut t, P0, "Kolaghan's Command");
    let card = t.hand(P0, "Kolaghan's Command");
    let cmd = t
        .cast(P0, card)
        .modes(&[1, 3])
        .targets(&[Entity::Player(P1)])
        .target(a)
        .go();
    let from = t.asked().len();
    cast_new(&mut t, P0, "Twincast", &[Entity::Object(cmd)]);
    change_copy_targets(&mut t, P0, &[None, Some(Entity::Object(b))]);
    t.resolve();
    let copy = spell_copies(&t)[0];
    assert_eq!(chosen_modes(&t, copy), vec![1, 3]);
    assert_eq!(modes_asked_since(&t, from), 0);
    assert_eq!(
        targets_of(&t, copy),
        vec![Entity::Player(P1), Entity::Object(b)]
    );
    t.resolve_all();
    assert!(!t.on_battlefield(a));
    assert!(!t.on_battlefield(b));
    assert_eq!(t.hand_size(P1), 1);
}

#[test]
fn a_copied_season_keeps_its_pawprint_modes() {
    cr!("707.10", "707.10c", "700.2i");
    ruling!(
        "Season of the Burrow",
        "If a Season is copied, the effect that creates the copy will usually allow you to choose new targets, but you can't choose new modes."
    );
    supported("Season of the Burrow");
    // {P} — Create a 1/1 white Rabbit creature token. {P}{P} — Exile target nonland
    // permanent. Its controller draws a card.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    lands_for_cost(&mut t, P0, "Season of the Burrow");
    let card = t.hand(P0, "Season of the Burrow");
    let season = t.cast(P0, card).modes(&[1, 0, 0, 0]).target(giant).go();
    let modes = chosen_modes(&t, season);
    assert_eq!(modes.iter().filter(|m| **m == 0).count(), 3);
    let from = t.asked().len();
    let p1_hand = t.hand_size(P1);
    cast_new(&mut t, P0, "Twincast", &[Entity::Object(season)]);
    change_copy_targets(&mut t, P0, &[Some(Entity::Object(bears))]);
    t.resolve();
    let copy = spell_copies(&t)[0];
    assert_eq!(chosen_modes(&t, copy), modes);
    assert_eq!(modes_asked_since(&t, from), 0);
    t.resolve_all();
    assert!(t.in_exile("Hill Giant"));
    assert!(t.in_exile("Grizzly Bears"));
    assert_eq!(creature_tokens(&t, P0), 6);
    assert_eq!(t.hand_size(P1), p1_hand + 2);
}
