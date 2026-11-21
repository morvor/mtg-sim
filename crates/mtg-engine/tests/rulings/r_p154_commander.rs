//! Rulings batch P154 — "the mana value of a commander you own on the battlefield or in
//! the command zone" and "the greatest mana value among your commanders" (CR 903.3: a
//! card's commander designation is an attribute of the card, kept wherever it is and
//! whoever controls it); the current characteristics of a commander that became a copy
//! (CR 707.2); with several commanders, the player chooses which one as the spell
//! resolves (`interp!("a-commander-you-own-is-chosen")`).

use crate::r_p154_common::*;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn a_commander_another_player_controls_still_sets_x() {
    cr!("903.3", "608.2h");
    ruling!(
        "Cloudkill",
        "If your commander is under the control of another player as Cloudkill resolves, it is still on the battlefield and will still be used to determine the value of X."
    );
    ruling!(
        "Majestic Genesis",
        "If your commander is under the control of another player as Majestic Genesis resolves, it is still on the battlefield and will still be used to determine the value of X."
    );
    ruling!(
        "Stinging Study",
        "If your commander is under the control of another player as Stinging Study resolves, it is still on the battlefield and will still be used to determine the value of X."
    );
    supported("Cloudkill");
    supported("Majestic Genesis");
    supported("Stinging Study");
    // Cloudkill with P0's Craw Wurm (mana value 6) under P1's control: all creatures get
    // -6/-6, the Wurm itself (6/4) included.
    let mut t = TestGame::new(2);
    let wurm = stolen_commander(&mut t, "Craw Wurm");
    let giant = t.battlefield(P1, "Hill Giant");
    cast_resolve(&mut t, P0, "Cloudkill", "4BB", &[]);
    assert!(!t.on_battlefield(giant) && !t.on_battlefield(wurm));
    // Majestic Genesis: the top 6 cards are revealed (permanent cards among them may be put
    // onto the battlefield; none are chosen).
    let mut t = TestGame::new(2);
    stolen_commander(&mut t, "Craw Wurm");
    stack_library(&mut t, P0, &["Forest", "Island", "Swamp", "Mountain", "Plains", "Forest", "Island"]);
    let from = t.asked().len();
    cast_resolve(&mut t, P0, "Majestic Genesis", "6GG", &[]);
    let offered = entity_candidates(&t, P0, from);
    assert_eq!(offered.last().map(|c| c.len()), Some(6), "{offered:?}");
    // Stinging Study: draw 6, lose 6.
    let mut t = TestGame::new(2);
    stolen_commander(&mut t, "Craw Wurm");
    let hand = t.hand_size(P0);
    cast_resolve(&mut t, P0, "Stinging Study", "4B", &[]);
    assert_eq!(t.hand_size(P0), hand + 6);
    assert_eq!(t.life(P0), 14);
}

#[test]
fn stinging_study_with_two_commanders_lets_you_choose_which() {
    cr!("903.3", "702.124e", "608.2d");
    interp!("a-commander-you-own-is-chosen");
    ruling!(
        "Stinging Study",
        "If you own multiple commanders on the battlefield and/or in the command zone, you choose one of them as Stinging Study resolves to determine the value of X."
    );
    supported("Stinging Study");
    // Partners: Thrasios (mana value 2) in the command zone and Tymna (3) on the
    // battlefield. Both are offered as the spell resolves (not as it's cast: a choice the
    // effect offers is made while applying it, CR 608.2d); choosing Thrasios, the smaller,
    // X is 2.
    let mut t = commander_game();
    let thrasios = commander(&mut t, P0, "Thrasios, Triton Hero");
    let tymna = commander_on_battlefield(&mut t, "Tymna the Weaver");
    let hand = t.hand_size(P0);
    let from = t.asked().len();
    t.answer_choose(P0, &[obj(thrasios)]);
    cast(&mut t, P0, "Stinging Study", "4B", &[]);
    assert!(
        entity_candidates(&t, P0, from).is_empty(),
        "no commander is chosen as the spell is cast: {:?}",
        asked_of(&t, P0, from)
    );
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
    assert_eq!(t.life(P0), 38);
    let offered = entity_candidates(&t, P0, from);
    let mut last = offered.last().cloned().unwrap_or_default();
    last.sort();
    let mut both = vec![obj(thrasios), obj(tymna)];
    both.sort();
    assert_eq!(last, both, "{offered:?}");
    // Choosing Tymna: X is 3.
    let mut t = commander_game();
    commander(&mut t, P0, "Thrasios, Triton Hero");
    let tymna = commander_on_battlefield(&mut t, "Tymna the Weaver");
    let hand = t.hand_size(P0);
    t.answer_choose(P0, &[obj(tymna)]);
    cast_resolve(&mut t, P0, "Stinging Study", "4B", &[]);
    assert_eq!(t.hand_size(P0), hand + 3);
    assert_eq!(t.life(P0), 37);
}

#[test]
fn stinging_study_default_choice_is_the_first_commander_offered() {
    cr!("903.3", "702.124e");
    interp!("a-commander-you-own-is-chosen");
    interp!("default-first-listed-choice");
    supported("Stinging Study");
    // With no answer, the first commander listed is chosen: X is its mana value.
    let mut t = commander_game();
    commander(&mut t, P0, "Thrasios, Triton Hero");
    commander_on_battlefield(&mut t, "Tymna the Weaver");
    let hand = t.hand_size(P0);
    let from = t.asked().len();
    cast_resolve(&mut t, P0, "Stinging Study", "4B", &[]);
    let offered = entity_candidates(&t, P0, from);
    let first = offered
        .last()
        .and_then(|c| c.first().copied())
        .and_then(|e| e.object())
        .expect("a commander is chosen");
    let x = mana_value(&t, first) as usize;
    assert!(x == 2 || x == 3);
    assert_eq!(t.hand_size(P0), hand + x);
    assert_eq!(t.life(P0), 40 - x as i32);
}

#[test]
fn stinging_study_offers_a_commander_another_player_controls() {
    cr!("903.3", "702.124e");
    interp!("a-commander-you-own-is-chosen");
    ruling!(
        "Stinging Study",
        "If your commander is under the control of another player as Stinging Study resolves, it is still on the battlefield and will still be used to determine the value of X."
    );
    supported("Stinging Study");
    // Thrasios (2) in the command zone and P0's Craw Wurm (6) under P1's control: the Wurm
    // is still a commander P0 owns on the battlefield, so it may be chosen, and X is 6.
    let mut t = commander_game();
    commander(&mut t, P0, "Thrasios, Triton Hero");
    let wurm = stolen_commander(&mut t, "Craw Wurm");
    let hand = t.hand_size(P0);
    let from = t.asked().len();
    t.answer_choose(P0, &[obj(wurm)]);
    cast_resolve(&mut t, P0, "Stinging Study", "4B", &[]);
    assert!(was_offered(&t, P0, from, wurm));
    assert_eq!(t.hand_size(P0), hand + 6);
    assert_eq!(t.life(P0), 34);
}

#[test]
fn commander_bonuses_use_the_commanders_current_characteristics_wherever_it_is() {
    cr!("903.3", "903.3e", "707.2", "613.2");
    ruling!(
        "Tangleweave Armor",
        "It doesn't matter where your commander is (or commanders are, if you have more than one because of the partner ability or similar). The bonus will be calculated using your commander's current characteristics. (Mana value typically doesn't change, but your commander may become a copy of something else.)"
    );
    ruling!(
        "Cactus Preserve",
        "It doesn’t matter where your commander is (or commanders are, if you have more than one because of the partner ability or similar). The value of X will be calculated using your commander’s current characteristics. (Mana value typically doesn’t change, but your commander may become a copy of something else.)"
    );
    supported("Tangleweave Armor");
    supported("Cactus Preserve");
    supported("Cytoshape");
    // Tangleweave Armor's Germ with the Craw Wurm (6) in the command zone: 6/6; after the
    // Wurm is cast and becomes a copy of Grizzly Bears (Cytoshape): 2/2; with the Wurm in
    // the graveyard: 6/6 again.
    let mut t = commander_game();
    let wurm = commander(&mut t, P0, "Craw Wurm");
    t.enter(P0, "Tangleweave Armor");
    t.resolve_all();
    let germ = tokens(&t, P0);
    assert_eq!(germ.len(), 1);
    assert_eq!(t.pt(germ[0]), (6, 6), "command zone");
    mana(&mut t, P0, "4GG");
    let wurm = t.cast(P0, wurm).go();
    t.resolve_all();
    let wurm = t.g.current(wurm);
    assert!(t.on_battlefield(wurm));
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_choose(P0, &[obj(bears)]);
    cast_resolve(&mut t, P0, "Cytoshape", "1GU", &[obj(wurm)]);
    assert_eq!(t.obj_now(wurm).chars.name.as_str(), "Grizzly Bears");
    assert_eq!(t.pt(germ[0]), (2, 2), "a copy of Grizzly Bears");
    // The Wurm dies and goes back to the command zone (CR 903.9a): 6/6 again.
    kill(&mut t, wurm);
    assert!(!t.on_battlefield(wurm));
    assert_eq!(t.pt(germ[0]), (6, 6), "{:?}", t.zone(wurm));
    // Cactus Preserve: an X/X Plant where X is the commander's mana value as the ability
    // resolves (CR 608.2h): 6 with the Wurm in the command zone; 2 if the Wurm is a copy of
    // Grizzly Bears then.
    let mut t = commander_game();
    commander(&mut t, P0, "Craw Wurm");
    let cactus = t.battlefield(P0, "Cactus Preserve");
    mana(&mut t, P0, "3");
    act(&mut t, P0, cactus, "becomes an X/X", &[]).expect("animate");
    t.resolve_all();
    assert_eq!(t.pt(cactus), (6, 6));
    let mut t = commander_game();
    let wurm = commander(&mut t, P0, "Craw Wurm");
    let cactus = t.battlefield(P0, "Cactus Preserve");
    mana(&mut t, P0, "4GG");
    let wurm = t.cast(P0, wurm).go();
    t.resolve_all();
    let wurm = t.g.current(wurm);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_choose(P0, &[obj(bears)]);
    cast_resolve(&mut t, P0, "Cytoshape", "1GU", &[obj(wurm)]);
    mana(&mut t, P0, "3");
    act(&mut t, P0, cactus, "becomes an X/X", &[]).expect("animate");
    t.resolve_all();
    assert_eq!(t.pt(cactus), (2, 2));
}
