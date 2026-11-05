//! Rulings batch P029 — any instant or sorcery spell can be copied, whether it has targets
//! or not (CR 707.10, 115.1), and whoever controls it when the copying card doesn't say
//! "you control"; the copy's controller (CR 707.10, 110.2).

use crate::r_p028_common::*;
use crate::r_p029_common::*;
use crate::r_p030_common::resolve_until_copies;
use crate::r_s01_common::{attack_with, block_and_finish, supported, with_subtype};
use crate::r_s06_common::activate_containing;
use crate::r_s18_common::ADVENTURE;
use crate::r_s25_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn copies_of_a_spell_without_targets() {
    cr!("707.10", "115.1");
    ruling!(
        "League Guildmage",
        "League Guildmage's second ability can copy any instant or sorcery spell you control, not just one with targets."
    );
    copies_divination(&LEAGUE_GUILDMAGE, P0, P0, 1);
    ruling!(
        "Lutri, the Spellchaser",
        "Lutri's ability can copy any instant or sorcery spell you control, not just one with targets."
    );
    copies_divination(&LUTRI, P0, P0, 1);
    ruling!(
        "Naru Meha, Master Wizard",
        "Naru's triggered ability can copy any instant or sorcery spell, not just one with targets."
    );
    copies_divination(&NARU_MEHA, P0, P0, 1);
    ruling!(
        "Reiterate",
        "Reiterate can copy any instant or sorcery spell, not just one with targets."
    );
    copies_divination(&REITERATE, P0, P0, 1);
    ruling!(
        "Swarm Intelligence",
        "Swarm Intelligence can copy any instant or sorcery spell, not just one with targets."
    );
    copies_divination(&SWARM_INTELLIGENCE, P0, P0, 1);
    ruling!(
        "The Mirari Conjecture",
        "The Mirari Conjecture’s final chapter ability copies any instant or sorcery spell you cast, not just those with targets."
    );
    copies_divination(&MIRARI_CONJECTURE, P0, P0, 1);
    ruling!(
        "Thousand-Year Storm",
        "Thousand-Year Storm's ability will copy any instant or sorcery spell, not just one with targets."
    );
    copies_divination(&THOUSAND_YEAR_STORM, P0, P0, 1);
    ruling!(
        "Twinning Staff",
        "Twinning Staff's last ability can copy any instant or sorcery spell you control, not just one with targets."
    );
    // The Staff's own replacement effect adds a copy.
    copies_divination(&TWINNING_STAFF, P0, P0, 2);
}

#[test]
fn copies_of_another_player_s_spell_without_targets() {
    cr!("707.10", "115.1", "110.2");
    // Reverberate and Twincast: the copy of P1's spell is P0's.
    ruling!(
        "Reverberate",
        "Reverberate can target (and copy) any instant or sorcery spell, not just one with targets. It doesn't matter who controls it."
    );
    copies_divination(&REVERBERATE, P0, P0, 1);
    copies_divination(&REVERBERATE, P1, P0, 1);
    ruling!(
        "Twincast",
        "Twincast can copy any instant or sorcery spell, not just one with targets."
    );
    copies_divination(&TWINCAST, P0, P0, 1);
    copies_divination(&TWINCAST, P1, P0, 1);
    ruling!(
        "Insidious Will",
        "The third mode can copy any instant or sorcery spell, not just one with targets."
    );
    ruling!("Insidious Will", "If you copy a spell, you control the copy.");
    copies_divination(&INSIDIOUS_WILL, P0, P0, 1);
    copies_divination(&INSIDIOUS_WILL, P1, P0, 1);
    // Meletis Charlatan: "The controller of target instant or sorcery spell copies it."
    ruling!(
        "Meletis Charlatan",
        "Meletis Charlatan's ability can target any instant or sorcery spell, not just one with targets and not just one that you control."
    );
    copies_divination(&MELETIS_CHARLATAN, P0, P0, 1);
    copies_divination(&MELETIS_CHARLATAN, P1, P1, 1);
}

#[test]
fn howl_of_the_horde_s_two_delayed_triggers_copy_a_spell_without_targets() {
    cr!("707.10", "603.7", "115.1");
    ruling!(
        "Howl of the Horde",
        "The delayed triggered abilities will copy any instant or sorcery spell, not just one with targets."
    );
    // Raid: P0 attacked this turn, so both delayed triggered abilities are created.
    let howl = Copier {
        name: "Howl of the Horde",
        setup: |t| {
            let attacker = t.battlefield(P0, "Hill Giant");
            attack_with(t, &[(attacker, Entity::Player(P1))]);
            block_and_finish(t, P1, &[]);
            t.advance_to(P0, Step::PostcombatMain);
            cast_new(t, P0, "Howl of the Horde", &[]);
            t.resolve_all();
        },
        start: |t, _| {
            keep_copy_targets(t, P0);
            keep_copy_targets(t, P0);
        },
    };
    let mut t = TestGame::new(2);
    (howl.setup)(&mut t);
    let div = cast_new(&mut t, P0, "Divination", &[]);
    (howl.start)(&mut t, div);
    let hand = t.hand_size(P0);
    let copies = resolve_collecting_copies(&mut t);
    assert_eq!(copies.len(), 2);
    assert_eq!(t.hand_size(P0), hand + 6);
    // Without raid, one.
    let mut t = TestGame::new(2);
    cast_new(&mut t, P0, "Howl of the Horde", &[]);
    t.resolve_all();
    cast_new(&mut t, P0, "Divination", &[]);
    keep_copy_targets(&mut t, P0);
    assert_eq!(resolve_collecting_copies(&mut t).len(), 1);
}

#[test]
fn lucky_clover_copies_an_adventure_without_targets() {
    cr!("707.10", "715.3", "115.1");
    ruling!(
        "Lucky Clover",
        "Lucky Clover can copy any Adventure spell, not just one with targets."
    );
    supported("Lucky Clover");
    supported("Lovestruck Beast // Heart's Desire");
    // Heart's Desire ({G}, Sorcery — Adventure): "Create a 1/1 white Human creature
    // token."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Lucky Clover");
    t.lands(P0, "Forest", 1);
    let beast = t.hand(P0, "Lovestruck Beast // Heart's Desire");
    t.cast(P0, beast).method(ADVENTURE).go();
    keep_copy_targets(&mut t, P0);
    let copies = resolve_collecting_copies(&mut t);
    assert_eq!(copies.len(), 1);
    assert_eq!(with_subtype(&t, P0, "Human").len(), 2);
    // A non-Adventure sorcery isn't copied.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Lucky Clover");
    cast_new(&mut t, P0, "Divination", &[]);
    assert!(resolve_collecting_copies(&mut t).is_empty());
}

#[test]
fn unbound_flourishing_copies_x_spells_and_abilities_without_targets() {
    cr!("707.10", "107.3", "115.1");
    ruling!(
        "Unbound Flourishing",
        "Unbound Flourishing's ability will copy any instant spell, sorcery spell, or activated ability whose mana cost or activation cost contains {X}, not just one with targets."
    );
    supported("Unbound Flourishing");
    supported("Sphinx's Revelation");
    supported("Mind Spring");
    supported("Helix Pinnacle");
    // An instant: Sphinx's Revelation ({X}{W}{U}{U}: "You gain X life and draw X
    // cards."), X = 1.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Unbound Flourishing");
    lands_for_cost(&mut t, P0, "Sphinx's Revelation");
    t.lands(P0, "Wastes", 1);
    let card = t.hand(P0, "Sphinx's Revelation");
    t.cast(P0, card).x(1).go();
    keep_copy_targets(&mut t, P0);
    let hand = t.hand_size(P0);
    assert_eq!(resolve_collecting_copies(&mut t).len(), 1);
    assert_eq!((t.hand_size(P0), t.life(P0)), (hand + 2, 22));
    // A sorcery: Mind Spring ({X}{U}{U}: "Draw X cards."), X = 2.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Unbound Flourishing");
    lands_for_cost(&mut t, P0, "Mind Spring");
    t.lands(P0, "Wastes", 2);
    let card = t.hand(P0, "Mind Spring");
    t.cast(P0, card).x(2).go();
    keep_copy_targets(&mut t, P0);
    let hand = t.hand_size(P0);
    assert_eq!(resolve_collecting_copies(&mut t).len(), 1);
    assert_eq!(t.hand_size(P0), hand + 4);
    // An activated ability: Helix Pinnacle ("{X}: Put X tower counters on this
    // enchantment."), X = 1.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Unbound Flourishing");
    let pinnacle = t.battlefield(P0, "Helix Pinnacle");
    t.lands(P0, "Wastes", 1);
    t.answer(P0, DecisionKind::X, Answer::Number(1));
    activate_containing(&mut t, P0, pinnacle, "tower").unwrap();
    keep_copy_targets(&mut t, P0);
    t.settle();
    assert_eq!(abilities_from(&t, pinnacle).len(), 1);
    t.resolve_all();
    assert_eq!(t.counters(pinnacle, "tower"), 2);
    // A spell without {X} isn't copied.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Unbound Flourishing");
    cast_new(&mut t, P0, "Divination", &[]);
    assert!(resolve_until_copies(&mut t).is_empty());
}
