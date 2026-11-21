//! Rulings batch S32 — lands and mana: triggered mana abilities that add "one mana of any
//! type that land produced" (CR 605.1b, 106.12), a land's mana ability that adds more
//! mana under a condition (Urza's lands), "any type that a land could produce"
//! (CR 106.7), choosing a basic land type on resolution (CR 608.2c), and playing a land
//! exiled with a permission (CR 305.2, 305.3).

use crate::r_s01_common::*;
use crate::r_s02_common::can_play_land;
use crate::r_s04_common::add_mana;
use crate::r_s20_common::tap_for_mana;
use crate::r_s24_common::pool;
use crate::r_s32_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// The mana in `p`'s pool, by type.
fn pool_now(t: &TestGame, p: PlayerId) -> Vec<(ManaType, u32)> {
    ManaType::ALL
        .iter()
        .map(|ty| (*ty, pool(t, p, *ty)))
        .filter(|(_, n)| *n > 0)
        .collect()
}

#[test]
fn a_land_tapped_for_two_mana_gets_one_more_of_a_type_it_produced() {
    cr!("605.1b", "106.12a", "605.4a");
    ruling!(
        "Mana Flare",
        "If you tap a land for more than one mana, you choose one type that was produced and add one mana of that type."
    );
    supported("Mana Flare");
    supported("Boros Garrison");
    // "Whenever a player taps a land for mana, that player adds one mana of any type that
    // land produced." Boros Garrison: "{T}: Add {R}{W}."
    for (pick, expected) in [
        (0, vec![(ManaType::W, 1), (ManaType::R, 2)]),
        (1, vec![(ManaType::W, 2), (ManaType::R, 1)]),
    ] {
        let mut t = TestGame::new(2);
        t.battlefield(P1, "Mana Flare");
        let garrison = t.battlefield(P0, "Boros Garrison");
        let from = t.asked().len();
        t.answer(P0, DecisionKind::Option, Answer::Index(pick));
        assert!(tap_for_mana(&mut t, P0, garrison, "{R}{W}"));
        t.settle();
        let offered = options_offered(&t, from);
        assert_eq!(offered.len(), 1, "{offered:?}");
        assert_eq!(offered[0].len(), 2, "red or white: {offered:?}");
        assert_eq!(pool_now(&t, P0), expected);
    }
}

#[test]
fn urzas_mine_adds_two_mana_when_you_have_all_three() {
    cr!("605.1a", "106.4");
    ruling!(
        "Urza's Mine",
        "If you have at least one of each of the three Urza's lands on the battlefield, you must take the 2 mana instead of just one."
    );
    supported("Urza's Mine");
    supported("Urza's Power Plant");
    supported("Urza's Tower");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Urza's Mine");
    let plant = t.battlefield(P0, "Urza's Power Plant");
    assert!(tap_for_mana(&mut t, P0, mine, "{C}"));
    assert_eq!(pool_now(&t, P0), vec![(ManaType::C, 1)]);
    t.battlefield(P0, "Urza's Tower");
    t.g.players[P0.idx()].mana_pool = Default::default();
    let from = t.asked().len();
    assert!(tap_for_mana(&mut t, P0, plant, "{C}"));
    assert_eq!(pool_now(&t, P0), vec![(ManaType::C, 2)]);
    assert_eq!(t.asked().len(), from, "no choice to take one");
}

#[test]
fn changing_a_lands_type_changes_what_it_could_produce() {
    cr!("106.7", "305.7");
    ruling!(
        "Naga Vitalist",
        "Any change to a land's type or abilities gained by a land can affect the types of mana a land can produce."
    );
    supported("Naga Vitalist");
    // "{T}: Add one mana of any type that a land you control could produce." P0's only land
    // is a Forest that Phantasmal Terrain made an Island.
    let mut t = TestGame::new(2);
    let naga = t.battlefield(P0, "Naga Vitalist");
    let forest = t.battlefield(P0, "Forest");
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    add_mana(&mut t, P0, ManaType::U, 2);
    let terrain = t.hand(P0, "Phantasmal Terrain");
    t.cast(P0, terrain).target(forest).go();
    t.resolve_all();
    assert_eq!(subtypes_now(&t, forest), vec!["Island"]);
    t.g.players[P0.idx()].mana_pool = Default::default();
    let from = t.asked().len();
    assert!(tap_for_mana(&mut t, P0, naga, "Add one mana"));
    // Only blue was possible.
    for opts in options_offered(&t, from) {
        assert!(!opts.iter().any(|o| o.contains('G') || o.contains("green")), "{opts:?}");
    }
    assert_eq!(pool_now(&t, P0), vec![(ManaType::U, 1)]);
}

#[test]
fn the_basic_land_type_is_chosen_as_the_ability_resolves() {
    cr!("608.2c", "305.7", "601.2c");
    ruling!(
        "Unstable Frontier",
        "You choose a basic land type as the ability resolves."
    );
    supported("Unstable Frontier");
    supported("Grixis Illusionist");
    // "{T}: Target land you control becomes the basic land type of your choice until end
    // of turn."
    let mut t = TestGame::new(2);
    let frontier = t.battlefield(P0, "Unstable Frontier");
    let forest = t.battlefield(P0, "Forest");
    let from = t.asked().len();
    t.activate(P0, frontier, 1, &[Entity::Object(forest)])
        .expect("activate Unstable Frontier");
    assert!(options_offered(&t, from).is_empty(), "nothing chosen yet");
    // Swamp.
    t.answer(P0, DecisionKind::Option, Answer::Index(2));
    t.resolve_all();
    assert_eq!(options_offered(&t, from).len(), 1);
    assert_eq!(subtypes_now(&t, forest), vec!["Swamp"]);
    // Until end of turn.
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(subtypes_now(&t, forest), vec!["Forest"]);
}

#[test]
fn a_land_played_from_exile_follows_the_normal_land_rules() {
    cr!("305.2", "305.3", "116.2a");
    ruling!(
        "Scion of Opulence",
        "Playing a card this way follows all the normal timing rules for that card. For example, if you play a land this way, you may do so only during your main phase while the stack is empty and only if you haven't yet played a land"
    );
    supported("Scion of Opulence");
    // "{R}, Sacrifice two artifacts: Exile the top card of your library. You may play
    // that card this turn."
    let setup = || {
        let mut t = TestGame::new(2);
        let scion = t.battlefield(P0, "Scion of Opulence");
        t.battlefield(P0, "Ornithopter");
        t.battlefield(P0, "Ornithopter");
        let card = t.library_top(P0, "Forest");
        add_mana(&mut t, P0, ManaType::R, 1);
        t.activate(P0, scion, 0, &[]).expect("activate Scion");
        t.resolve_all();
        let card = t.g.current(card);
        assert!(t.in_exile("Forest"));
        (t, card)
    };
    // In the main phase with the stack empty: yes.
    let (mut t, card) = setup();
    assert!(can_play_land(&mut t, P0, card));
    t.play_land(P0, card).expect("play the exiled Forest");
    assert_eq!(t.named_on_battlefield("Forest").len(), 1);
    // Not after playing a land this turn.
    let (mut t, card) = setup();
    let other = t.hand(P0, "Plains");
    t.play_land(P0, other).unwrap();
    assert!(!can_play_land(&mut t, P0, card));
    // Not while the stack isn't empty, nor outside a main phase.
    let (mut t, card) = setup();
    t.lands(P1, "Mountain", 1);
    let shock = t.hand(P1, "Shock");
    t.cast(P1, shock).target(P0).go();
    assert!(!can_play_land(&mut t, P0, card));
    t.resolve_all();
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!can_play_land(&mut t, P0, card));
    t.set_step(P0, Step::PostcombatMain);
    assert!(can_play_land(&mut t, P0, card));
}
