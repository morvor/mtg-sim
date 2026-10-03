//! CR 702.52 Dredge.

use crate::common_k702_011_017::assert_supported;
use crate::common_k702_052_066::*;
use mtg_engine::ability::*;
use mtg_engine::decision::Decision;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Makes `p` draw `n` cards (as an effect would).
fn draw(t: &mut TestGame, p: PlayerId, n: i32) {
    run_effect(
        t,
        None,
        p,
        Effect::Draw {
            who: PlayerRef::You,
            n: Value::c(n),
        },
        &[],
    );
}

/// Makes `p` mill `n` cards.
fn mill(t: &mut TestGame, p: PlayerId, n: i32) {
    run_effect(
        t,
        None,
        p,
        Effect::Mill {
            who: PlayerRef::You,
            n: Value::c(n),
        },
        &[],
    );
}

/// Number of times `p` was offered to apply a dredge ability.
fn dredge_offers(t: &TestGame, p: PlayerId) -> usize {
    yes_no_asked(t, p, "Dredge")
}

#[test]
fn dredge_replaces_a_draw_with_milling_and_returning_the_card() {
    cr!("702.52", "702.52a");
    assert_supported("Life from the Loam");
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Life from the Loam");
    let library = t.library_size(P0);
    let hand = t.hand_size(P0);
    t.answer_yes(P0, true);
    draw(&mut t, P0, 1);
    assert_eq!(dredge_offers(&t, P0), 1);
    // Three cards milled, Life from the Loam returned instead of drawing.
    assert_eq!(t.library_size(P0), library - 3);
    assert_eq!(hand_names(&t, P0), vec!["Life from the Loam".to_string()]);
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.graveyard_size(P0), 3);
    assert_eq!(t.g.history.cards_drawn.get(&P0).copied().unwrap_or(0), 0);
}

#[test]
fn dredge_is_optional() {
    cr!("702.52a");
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Life from the Loam");
    let library = t.library_size(P0);
    t.answer_yes(P0, false);
    draw(&mut t, P0, 1);
    assert_eq!(dredge_offers(&t, P0), 1);
    assert_eq!(t.library_size(P0), library - 1);
    assert_eq!(hand_names(&t, P0), vec!["Filler".to_string()]);
    assert!(t.in_graveyard(P0, "Life from the Loam"));
}

#[test]
fn dredge_can_replace_any_draw() {
    cr!("702.52a");
    ruling!(
        "Life from the Loam",
        "Dredge can replace any card draw, not only the one during your draw step."
    );
    // The draw step's draw.
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Life from the Loam");
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::Draw);
    assert_eq!(dredge_offers(&t, P0), 1);
    assert!(t.in_hand(P0, "Life from the Loam"));
    assert_eq!(t.hand_size(P0), 1);
    assert_eq!(t.graveyard_size(P0), 3);
    // A spell's draw.
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Life from the Loam");
    t.lands(P0, "Island", 3);
    let div = t.hand(P0, "Divination");
    t.cast(P0, div).go();
    t.answer_yes(P0, true);
    t.answer_yes(P0, false);
    t.resolve();
    assert!(t.in_hand(P0, "Life from the Loam"));
    assert_eq!(t.hand_size(P0), 2);
}

#[test]
fn only_a_draw_can_be_replaced_by_dredge() {
    cr!("702.52a");
    ruling!(
        "Golgari Grave-Troll",
        "If an effect puts a card into your hand without specifically using the word \"draw,\" you're not drawing a card."
    );
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Golgari Grave-Troll");
    // "Put the top card of your library into your hand" isn't a draw.
    run_effect(
        &mut t,
        None,
        P0,
        Effect::Move {
            what: Sel::TopOfLibrary(PlayerRef::You, Value::c(1)),
            to: Destination::zone(ZoneKind::Hand),
        },
        &[],
    );
    assert_eq!(dredge_offers(&t, P0), 0);
    assert_eq!(t.hand_size(P0), 1);
    assert!(t.in_graveyard(P0, "Golgari Grave-Troll"));
}

#[test]
fn a_player_with_too_few_cards_in_their_library_cant_dredge() {
    cr!("702.52b");
    ruling!(
        "Golgari Grave-Troll",
        "You can't attempt to use a dredge ability if you don't have enough cards in your library."
    );
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Golgari Grave-Troll");
    // Dredge 6 with five cards in the library.
    let n = t.library_size(P0) as i32 - 5;
    mill(&mut t, P0, n);
    t.answer_yes(P0, true);
    draw(&mut t, P0, 1);
    assert_eq!(dredge_offers(&t, P0), 0);
    assert_eq!(t.library_size(P0), 4);
    assert!(t.in_graveyard(P0, "Golgari Grave-Troll"));
    // With exactly six it can.
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Golgari Grave-Troll");
    let n = t.library_size(P0) as i32 - 6;
    mill(&mut t, P0, n);
    t.answer_yes(P0, true);
    draw(&mut t, P0, 1);
    assert_eq!(t.library_size(P0), 0);
    assert!(t.in_hand(P0, "Golgari Grave-Troll"));
    assert!(!t.has_lost(P0));
}

#[test]
fn one_draw_is_replaced_by_at_most_one_dredge_ability() {
    cr!("702.52a");
    ruling!(
        "Stinkweed Imp",
        "One card draw can't be replaced by multiple dredge abilities."
    );
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Stinkweed Imp");
    t.graveyard(P0, "Darkblast");
    // The player chooses which one replaces the draw.
    t.answer(P0, DecisionKind::Replacement, decision::Answer::Index(1));
    t.answer_yes(P0, true);
    draw(&mut t, P0, 1);
    let chosen = t
        .asked()
        .into_iter()
        .find_map(|(_, d)| match d {
            Decision::ChooseReplacement { options } => Some(options),
            _ => None,
        })
        .expect("a choice between the dredge abilities");
    assert_eq!(chosen.len(), 2);
    assert_eq!(hand_names(&t, P0).len(), 1);
    let returned = hand_names(&t, P0)[0].clone();
    assert!(chosen[1].starts_with(&returned));
    // Only one card was returned, and only its dredge number of cards was milled.
    let milled = if returned == "Stinkweed Imp" { 5 } else { 3 };
    assert_eq!(t.graveyard_size(P0), 1 + milled);
}

#[test]
fn each_draw_of_several_is_replaced_separately() {
    cr!("702.52a");
    ruling!(
        "Dakmor Salvage",
        "another card with a dredge ability (including one that was milled by the first dredge ability) may be used to replace the second draw"
    );
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Dakmor Salvage");
    // Golgari Thug is milled by the first dredge (dredge 2 mills the top two cards).
    on_top(&mut t, P0, "Golgari Thug");
    on_top(&mut t, P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    draw(&mut t, P0, 2);
    let mut hand = hand_names(&t, P0);
    hand.sort();
    assert_eq!(hand, vec!["Dakmor Salvage", "Golgari Thug"]);
    // Two cards milled by Dakmor Salvage, then four by Golgari Thug.
    assert_eq!(t.graveyard_size(P0), 5);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
}

#[test]
fn a_card_returned_by_dredge_was_put_into_its_owners_hand_from_the_graveyard() {
    cr!("702.52a");
    ruling!(
        "Golgari Brownscale",
        "triggers when Golgari Brownscale returns to your hand from your graveyard for any reason"
    );
    assert_supported("Golgari Brownscale");
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Golgari Brownscale");
    t.answer_yes(P0, true);
    draw(&mut t, P0, 1);
    t.resolve_all();
    assert!(t.in_hand(P0, "Golgari Brownscale"));
    assert_eq!(t.life(P0), 22);
    // Returned another way, it triggers too.
    let scale = t.graveyard(P0, "Golgari Brownscale");
    run_effect(
        &mut t,
        None,
        P0,
        Effect::Move {
            what: Sel::Target(0),
            to: Destination::zone(ZoneKind::Hand),
        },
        &[Entity::Object(scale)],
    );
    t.resolve_all();
    assert_eq!(t.life(P0), 24);
}
