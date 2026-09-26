//! Rulings batch S01 — aftermath (CR 702.127) and the split card rules its rulings restate
//! (CR 709).

use crate::r_s01_common::*;
use mtg_engine::ability::{Cmp, Filter, Value};
use mtg_engine::eval::Ctx;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

fn colors(cs: &[Color]) -> ColorSet {
    cs.iter()
        .fold(ColorSet::default(), |s, c| s.union(ColorSet::single(*c)))
}

/// Player `p`'s answer to "choose a color" (Iona, Shield of Emeria).
fn choose_color(t: &mut TestGame, p: PlayerId, c: Color) {
    let i = Color::ALL.iter().position(|x| *x == c).unwrap();
    t.answer(p, DecisionKind::Option, Answer::Index(i));
}

#[test]
fn a_split_card_is_one_card() {
    cr!("709.1");
    ruling!(
        "Consign // Oblivion",
        "Each split card is a single card. For example, if you discard one, you've discarded one card, not two. If an effect counts the number of instant and sorcery cards in your graveyard, Destined // Lead counts once, not twice."
    );
    ruling!(
        "Claim // Fame",
        "Each split card is a single card. For example, if you discard one, you’ve discarded one card, not two."
    );
    supported("Enigma Drake");
    let mut t = TestGame::new(2);
    // "Enigma Drake's power is equal to the number of instant and sorcery cards in your
    // graveyard." Consign // Oblivion is an instant card and a sorcery card: it counts
    // once.
    let drake = t.battlefield(P0, "Enigma Drake");
    let co = t.hand(P0, "Consign // Oblivion");
    t.hand(P0, "Island");
    t.g.discard(P0, co, None);
    assert_eq!(t.hand_size(P0), 1);
    assert_eq!(t.graveyard_size(P0), 1);
    t.g.recompute();
    assert_eq!(t.pt(drake).0, 1);
    t.graveyard(P0, "Claim // Fame");
    t.g.recompute();
    assert_eq!(t.pt(drake).0, 2);
}

#[test]
fn off_the_stack_a_split_card_combines_both_halves() {
    cr!("709.4");
    ruling!(
        "Consign // Oblivion",
        "While not on the stack, the characteristics of a split card are the combination of its two halves. For example, Destined // Lead is a green and black card, it is both an instant card and a sorcery card, and its mana value is 6."
    );
    ruling!(
        "Claim // Fame",
        "While not on the stack, the characteristics of a split card are the combination of its two halves. For example, Destined // Lead is a green and black card"
    );
    let mut t = TestGame::new(2);
    let co = t.hand(P0, "Consign // Oblivion");
    let cf = t.hand(P0, "Claim // Fame");
    t.g.recompute();
    let c = t.obj(co).chars.clone();
    assert_eq!(c.colors, colors(&[Color::Blue, Color::Black]));
    assert!(c.is(CardType::Instant) && c.is(CardType::Sorcery));
    assert_eq!(t.g.mana_value_of(co), 7);
    let c = t.obj(cf).chars.clone();
    assert_eq!(c.colors, colors(&[Color::Black, Color::Red]));
    assert_eq!(t.g.mana_value_of(cf), 3);
    let ctx = Ctx::new(None, P0);
    assert!(!t
        .g
        .matches(co, &Filter::ManaValue(Cmp::Le, Box::new(Value::c(2))), &ctx));
}

#[test]
fn cascade_skips_a_split_card_whose_combined_mana_value_is_too_high() {
    cr!("709.4", "702.85a");
    ruling!(
        "Consign // Oblivion",
        "This means that if an effect allows you to cast a card with mana value 2 from your hand, you can't cast Destined."
    );
    let mut t = TestGame::new(2);
    // Consign alone has mana value 2, but the card's mana value is 7: not less than 4.
    stack_library(&mut t, P0, &["Consign // Oblivion", "Grizzly Bears"]);
    give_mana_for(&mut t, P0, "Bloodbraid Elf");
    let elf = t.hand(P0, "Bloodbraid Elf");
    t.cast(P0, elf).go();
    t.settle();
    t.resolve();
    let top = *t.g.stack.last().unwrap();
    assert_eq!(t.g.obj(top).chars.name, "Grizzly Bears");
    t.resolve_all();
    let bottom = t.g.player(P0).library[0];
    assert_eq!(t.g.obj(bottom).chars.name, "Consign // Oblivion");
}

#[test]
fn on_the_stack_only_the_cast_half_counts() {
    cr!("709.3b", "709.4");
    ruling!(
        "Consign // Oblivion",
        "All split cards have two card faces on a single card, and you put a split card onto the stack with only the half you're casting. The characteristics of the half of the card you didn't cast are ignored while the spell is on the stack."
    );
    ruling!(
        "Claim // Fame",
        "All split cards have two card faces on a single card, and you put a split card onto the stack with only the half you’re casting."
    );
    supported("Iona, Shield of Emeria");
    let mut t = TestGame::new(2);
    // "Your opponents can't cast spells of the chosen color": black. Consign // Oblivion is
    // a blue and black card, but Consign is a blue spell.
    choose_color(&mut t, P1, Color::Black);
    t.enter(P1, "Iona, Shield of Emeria");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Island", 2);
    let co = t.hand(P0, "Consign // Oblivion");
    let spell = t.cast(P0, co).method(CastMethod::Half(0)).target(bears).go();
    let c = t.obj(spell).chars.clone();
    assert_eq!(c.name, "Consign");
    assert_eq!(c.colors, ColorSet::single(Color::Blue));
    assert!(c.is(CardType::Instant) && !c.is(CardType::Sorcery));
    assert_eq!(t.g.mana_value_of(spell), 2);
    t.resolve_all();
    assert!(t.in_hand(P1, "Grizzly Bears"));
    // Claim // Fame is black and red; with red chosen, Claim (black) can be cast.
    let mut t = TestGame::new(2);
    choose_color(&mut t, P1, Color::Red);
    t.enter(P1, "Iona, Shield of Emeria");
    t.graveyard(P0, "Grizzly Bears");
    let bears = t.g.player(P0).graveyard[0];
    t.lands(P0, "Swamp", 1);
    let cf = t.hand(P0, "Claim // Fame");
    t.cast(P0, cf).method(CastMethod::Half(0)).target(bears).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}
