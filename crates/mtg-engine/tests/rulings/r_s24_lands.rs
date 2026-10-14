//! Rulings batch S24 — lands and abilities that count the lands you control: "enters
//! tapped unless" lands (CR 614.1d, 614.12), bounce lands, land-type counts (CR 305.6,
//! 205.3i), and the alternative cost of the Borderposts (CR 118.9).

use crate::r_s01_common::supported;
use crate::r_s05_common::enter;
use crate::r_s24_common::*;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn a_bounce_land_with_no_other_lands_returns_itself() {
    cr!("603.2", "614.1d");
    ruling!(
        "Simic Growth Chamber",
        "If this land enters the battlefield and you control no other lands, its ability will force you to return it to your hand."
    );
    supported("Simic Growth Chamber");
    // "This land enters tapped. When this land enters, return a land you control to its
    // owner's hand."
    let mut t = TestGame::new(2);
    let chamber = t.hand(P0, "Simic Growth Chamber");
    t.play_land(P0, chamber).expect("play the land");
    t.settle();
    assert!(tapped(&t, chamber));
    t.resolve_all();
    assert!(t.in_hand(P0, "Simic Growth Chamber"));
    assert!(t.g.permanents().all(|o| o.controller != P0));
}

#[test]
fn a_fast_land_doesnt_count_lands_entering_at_the_same_time() {
    cr!("614.12", "614.1d");
    ruling!(
        "Blackcleave Cliffs",
        "If one of these lands enters the battlefield at the same time as one or more other lands (due to Oblivion Sower or Warp World, perhaps), it doesn't take those lands into consideration when determining how many other lands you control."
    );
    supported("Blackcleave Cliffs");
    // "This land enters tapped unless you control two or fewer other lands."
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 2);
    let entered = enter_together(
        &mut t,
        &[(P0, "Blackcleave Cliffs"), (P0, "Mountain"), (P0, "Mountain")],
    );
    // Two other lands before the simultaneous entry: untapped, although P0 now controls
    // four other lands.
    assert!(!tapped(&t, entered[0]));
    // Entering on its own with those four other lands: tapped.
    let cliffs = t.enter(P0, "Blackcleave Cliffs");
    assert!(tapped(&t, cliffs));
}

#[test]
fn a_fast_land_enters_untapped_with_two_or_fewer_other_lands() {
    cr!("614.12", "614.1d");
    ruling!(
        "Seachrome Coast",
        "If one of these lands enters the battlefield under your control and you control zero, one, or two other lands, it enters the battlefield untapped. If you control three or more other lands, however, it enters the battlefield tapped."
    );
    supported("Seachrome Coast");
    let mut t = TestGame::new(2);
    // An opponent's lands aren't "other lands" you control.
    t.lands(P1, "Plains", 5);
    for n in 0..4 {
        let coast = t.enter(P0, "Seachrome Coast");
        assert_eq!(tapped(&t, coast), n >= 3, "{n} other lands");
    }
}

#[test]
fn opponents_lands_entering_at_the_same_time_dont_count_for_a_turbulent_land() {
    cr!("614.12", "614.1d");
    ruling!(
        "Turbulent Fen",
        "If this land enters the battlefield at the same time as any number of lands your opponents control, those other lands are not counted when determining if this land enters the battlefield tapped or untapped."
    );
    supported("Turbulent Fen");
    // "This land enters tapped unless your opponents control eight or more lands."
    let mut t = TestGame::new(2);
    t.lands(P1, "Forest", 7);
    let entered = enter_together(&mut t, &[(P0, "Turbulent Fen"), (P1, "Forest")]);
    assert!(tapped(&t, entered[0]));
    // Now the opponent controls eight lands.
    let fen = t.enter(P0, "Turbulent Fen");
    assert!(!tapped(&t, fen));
}

#[test]
fn a_land_put_onto_the_battlefield_tapped_by_an_effect_enters_tapped() {
    cr!("614.12", "614.1d", "603.6a");
    ruling!(
        "Mystic Sanctuary",
        "If another effect puts these lands onto the battlefield tapped, they enter tapped, even if you control enough lands with the appropriate basic land type."
    );
    supported("Mystic Sanctuary");
    supported("Farseek");
    // "This land enters tapped unless you control three or more other Islands. When this
    // land enters untapped, you may put target instant or sorcery card from your
    // graveyard on top of your library."
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 3);
    t.lands(P0, "Forest", 2);
    t.graveyard(P0, "Lightning Bolt");
    let sanctuary = t.library_top(P0, "Mystic Sanctuary");
    // Farseek: "Search your library for a Plains, Island, Swamp, or Mountain card and put
    // it onto the battlefield tapped, then shuffle."
    let farseek = t.hand(P0, "Farseek");
    t.answer_choose(P0, &[Entity::Object(sanctuary)]);
    t.answer_yes(P0, true);
    t.cast(P0, farseek).go();
    t.resolve_all();
    let on_bf = t.named_on_battlefield("Mystic Sanctuary");
    assert_eq!(on_bf.len(), 1);
    assert!(tapped(&t, on_bf[0]));
    // It didn't enter untapped: no trigger, the Bolt stays in the graveyard.
    assert!(t.in_graveyard(P0, "Lightning Bolt"));
    // Put onto the battlefield without such an effect, it enters untapped and triggers.
    let other = t.enter(P0, "Mystic Sanctuary");
    t.settle();
    assert!(!tapped(&t, other));
    t.resolve_all();
    assert!(!t.in_graveyard(P0, "Lightning Bolt"));
}

#[test]
fn battle_lands_arent_basic_lands() {
    cr!("205.4c", "614.1d");
    ruling!(
        "Sunken Hollow",
        "Even though these lands have basic land types, they are not basic lands because \"basic\" doesn't appear on their type line. Notably, controlling two or more of them won't allow others to enter the battlefield untapped."
    );
    supported("Sunken Hollow");
    supported("Canopy Vista");
    // "This land enters tapped unless you control two or more basic lands."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sunken Hollow");
    t.battlefield(P0, "Canopy Vista");
    let third = t.enter(P0, "Sunken Hollow");
    assert!(tapped(&t, third));
    // Two basic lands do.
    t.lands(P0, "Island", 2);
    let fourth = t.enter(P0, "Canopy Vista");
    assert!(!tapped(&t, fourth));
}

#[test]
fn a_borderpost_can_return_any_basic_land_even_a_tapped_one() {
    cr!("118.9", "601.2h");
    ruling!(
        "Fieldmist Borderpost",
        "To satisfy the alternative cost, you may return any basic land you control to its owner's hand, regardless of that land's subtype or whether it's tapped."
    );
    supported("Fieldmist Borderpost");
    // "You may pay {1} and return a basic land you control to its owner's hand rather
    // than pay this spell's mana cost." A tapped Mountain can be returned.
    let mut t = TestGame::new(2);
    let mountain = t.battlefield(P0, "Mountain");
    t.g.objects[mountain.0 as usize].tapped = true;
    let wastes = t.battlefield(P0, "Wastes");
    let post = t.hand(P0, "Fieldmist Borderpost");
    let alt = t
        .cast_options(P0, post)
        .into_iter()
        .map(|o| o.method)
        .find(|m| matches!(m, CastMethod::Alternative(_)))
        .expect("the alternative cost is available");
    t.answer_choose(P0, &[Entity::Object(mountain)]);
    t.cast(P0, post).method(alt).go();
    assert!(t.in_hand(P0, "Mountain"));
    assert!(tapped(&t, wastes));
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Fieldmist Borderpost").len(), 1);
}

#[test]
fn each_hedge_mage_ability_looks_at_your_lands_individually() {
    cr!("603.2", "305.6");
    ruling!(
        "Duergar Hedge-Mage",
        "Each of the triggered abilities look at your lands individually. This means that if you only control two dual-lands of the appropriate types, both of the abilities will trigger."
    );
    supported("Duergar Hedge-Mage");
    // "When this creature enters, if you control two or more Mountains, you may destroy
    // target artifact. When this creature enters, if you control two or more Plains, you
    // may destroy target enchantment." Plateau is a Mountain Plains.
    let mut t = TestGame::new(2);
    t.lands(P0, "Plateau", 2);
    let ornithopter = t.battlefield(P1, "Ornithopter");
    let anthem = t.battlefield(P1, "Glorious Anthem");
    t.answer_targets(P0, &[Entity::Object(ornithopter)]);
    t.answer_targets(P0, &[Entity::Object(anthem)]);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    enter(&mut t, P0, "Duergar Hedge-Mage");
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Ornithopter"));
    assert!(t.in_graveyard(P1, "Glorious Anthem"));
}

#[test]
fn a_hedge_mage_ability_has_an_intervening_if_clause() {
    cr!("603.4");
    ruling!(
        "Duergar Hedge-Mage",
        "Each of the triggered abilities has an \"intervening 'if' clause.\" That means (1) the ability won't trigger at all unless you control two or more lands of the appropriate land type when the Hedge-Mage enters, and (2) the ability will do nothing if you don't control two or more lands of the appropriate land type by the time it resolves."
    );
    // One Mountain and two Plains: only the Plains ability triggers.
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Plains", 2);
    let ornithopter = t.battlefield(P1, "Ornithopter");
    let anthem = t.battlefield(P1, "Glorious Anthem");
    t.answer_targets(P0, &[Entity::Object(anthem)]);
    t.answer_yes(P0, true);
    enter(&mut t, P0, "Duergar Hedge-Mage");
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert!(t.on_battlefield(ornithopter));
    assert!(t.in_graveyard(P1, "Glorious Anthem"));
    // Two Mountains when it enters, but one is gone by the time the ability resolves.
    let mut t = TestGame::new(2);
    let mountains = t.lands(P0, "Mountain", 2);
    let ornithopter = t.battlefield(P1, "Ornithopter");
    t.answer_targets(P0, &[Entity::Object(ornithopter)]);
    t.answer_yes(P0, true);
    enter(&mut t, P0, "Duergar Hedge-Mage");
    assert_eq!(t.stack_len(), 1);
    crate::r_s05_common::move_to(&mut t, mountains[0], Zone::Hand(P0));
    t.resolve_all();
    assert!(t.on_battlefield(ornithopter));
}

#[test]
fn differently_named_lands_count_each_english_name_once() {
    cr!("201.2");
    ruling!(
        "All-Fates Scroll",
        "To determine the number of differently named lands you control, count each land you control once, but only if its English name isn’t exactly the same as another land you’ve already counted this way."
    );
    supported("All-Fates Scroll");
    // "{7}, {T}, Sacrifice this artifact: Draw X cards, where X is the number of
    // differently named lands you control." Five Forests, an Island and a Seachrome Coast:
    // three names.
    let mut t = TestGame::new(2);
    let scroll = t.battlefield(P0, "All-Fates Scroll");
    t.lands(P0, "Forest", 5);
    t.lands(P0, "Island", 1);
    t.lands(P0, "Seachrome Coast", 1);
    // An opponent's differently named land doesn't count.
    t.lands(P1, "Mountain", 1);
    let hand = t.hand_size(P0);
    crate::r_s06_common::activate_containing(&mut t, P0, scroll, "Draw X")
        .expect("activate the Scroll");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 3);
}
