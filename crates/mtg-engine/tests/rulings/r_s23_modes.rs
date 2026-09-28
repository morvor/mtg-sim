//! Rulings batch S23 — modal spells (CR 700.2): the modes are chosen as the spell is cast
//! (CR 601.2b) and can't be changed afterward; "choose two" means two different modes
//! unless the spell says a mode may be chosen more than once (CR 700.2d).

use crate::r_s01_common::*;
use crate::r_s07_common::chosen_modes;
use mtg_engine::decision::Decision;
use mtg_engine::testing::*;
use mtg_engine::*;

/// The (min, max, allow_repeat) of the mode choices asked since `from`.
fn mode_choices(t: &TestGame, from: usize) -> Vec<(u32, u32, bool)> {
    t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseModes {
                min,
                max,
                allow_repeat,
                ..
            } => Some((*min, *max, *allow_repeat)),
            _ => None,
        })
        .collect()
}

#[test]
fn ojutais_command_takes_two_different_modes_chosen_as_its_cast() {
    cr!("700.2", "700.2d", "601.2b");
    ruling!(
        "Ojutai's Command",
        "You choose the two modes as you cast the spell. You must choose two different modes. Once modes are chosen, they can’t be changed."
    );
    supported("Ojutai's Command");
    // Ojutai's Command: "Choose two — • Return target creature card with mana value 2 or
    // less from your graveyard to the battlefield. • You gain 4 life. • Counter target
    // creature spell. • Draw a card."
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 2);
    t.lands(P0, "Island", 2);
    let command = t.hand(P0, "Ojutai's Command");
    let from = t.asked().len();
    // "You gain 4 life" twice isn't a legal choice: two different modes are chosen.
    let spell = t.cast(P0, command).modes(&[1, 1]).go();
    assert_eq!(mode_choices(&t, from), vec![(2, 2, false)]);
    let modes = chosen_modes(&t, spell);
    assert_eq!(modes.len(), 2);
    assert_ne!(modes[0], modes[1]);
    // Two legal different modes: gain 4 life and draw a card.
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 2);
    t.lands(P0, "Island", 2);
    let command = t.hand(P0, "Ojutai's Command");
    let spell = t.cast(P0, command).modes(&[1, 3]).go();
    assert_eq!(chosen_modes(&t, spell), vec![1, 3]);
    // A creature card that would make the first mode possible arrives in the graveyard
    // afterward: the modes stay as chosen.
    t.graveyard(P0, "Grizzly Bears");
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.life(P0), 24);
    assert_eq!(t.hand_size(P0), hand + 1);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
}

#[test]
fn winterflame_one_or_both_modes_are_chosen_as_its_cast() {
    cr!("700.2", "601.2b", "601.2c");
    ruling!(
        "Winterflame",
        "You choose which mode you’re using—or that you’re using both modes—as you’re casting the spell. Once this choice is made, it can’t be changed later while the spell is on the stack."
    );
    supported("Winterflame");
    // "Choose one or both — • Tap target creature. • Winterflame deals 2 damage to target
    // creature."
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 1);
    t.lands(P0, "Mountain", 2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let elves = t.battlefield(P1, "Llanowar Elves");
    let card = t.hand(P0, "Winterflame");
    let from = t.asked().len();
    let spell = t
        .cast(P0, card)
        .modes(&[0, 1])
        .targets(&[Entity::Object(bears)])
        .targets(&[Entity::Object(elves)])
        .go();
    assert_eq!(mode_choices(&t, from), vec![(1, 2, false)]);
    assert_eq!(chosen_modes(&t, spell), vec![0, 1]);
    t.resolve_all();
    assert!(t.obj_now(bears).tapped);
    assert!(t.in_graveyard(P1, "Llanowar Elves"));
    // Only the damage mode: nothing is tapped.
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 1);
    t.lands(P0, "Mountain", 2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let card = t.hand(P0, "Winterflame");
    let spell = t
        .cast(P0, card)
        .modes(&[1])
        .targets(&[Entity::Object(bears)])
        .go();
    assert_eq!(chosen_modes(&t, spell), vec![1]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn wretched_confluence_modes_are_chosen_as_its_cast() {
    cr!("700.2", "700.2d", "601.2b");
    ruling!(
        "Wretched Confluence",
        "You choose the modes as you cast the spell. Once modes are chosen, they can’t be changed."
    );
    supported("Wretched Confluence");
    // "Choose three. You may choose the same mode more than once. • Target player draws a
    // card and loses 1 life. • Target creature gets -2/-2 until end of turn. • Return
    // target creature card from your graveyard to your hand."
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 5);
    let giant = t.battlefield(P1, "Hill Giant");
    let card = t.hand(P0, "Wretched Confluence");
    let from = t.asked().len();
    let spell = t
        .cast(P0, card)
        .modes(&[0, 0, 1])
        .targets(&[Entity::Player(P0)])
        .targets(&[Entity::Player(P0)])
        .targets(&[Entity::Object(giant)])
        .go();
    let asked = mode_choices(&t, from);
    assert_eq!(asked.len(), 1);
    assert_eq!((asked[0].0, asked[0].2), (3, true));
    assert_eq!(chosen_modes(&t, spell), vec![0, 0, 1]);
    // A creature card reaches P0's graveyard afterward: the third mode isn't added.
    t.graveyard(P0, "Grizzly Bears");
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
    assert_eq!(t.life(P0), 18);
    assert_eq!(t.pt(giant), (1, 1));
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
}
