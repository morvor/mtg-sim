//! Rulings batch S20 — split cards (CR 709): Fuss // Bother and Expansion // Explosion,
//! and effects that let a player cast a spell with certain characteristics (the Expertise
//! cycle: "You may cast a spell with mana value N or less from your hand without paying
//! its mana cost.", CR 601.3e), which look only at the half being cast
//! (CR 709.3a).

use crate::r_s01_common::*;
use crate::r_s04_common::add_mana;
use crate::r_s08_common::mana_value;
use crate::r_s20_common::sram_expertise;
use mtg_engine::ability::{Cmp, Filter, Value};
use mtg_engine::decision::Answer;
use mtg_engine::eval::Ctx;
use mtg_engine::mana::ManaType;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// The value of `v` for P0 now.
fn value(t: &mut TestGame, v: Value) -> i64 {
    t.g.recompute();
    t.g.eval_value(&v, &Ctx::new(None, P0))
}

#[test]
fn fuss_bother_is_a_single_card() {
    cr!("709.1", "709.4");
    ruling!(
        "Fuss // Bother",
        "Each split card is a single card. For example, if you discard a split card, you've discarded one card, not two. If an effect counts the number of instant and sorcery cards in your graveyard, Cease // Desist counts once, not twice."
    );
    supported("Fuss // Bother");
    supported("Enigma Drake");
    let mut t = TestGame::new(2);
    // Mind Rot: "Target player discards two cards." P1 discards Fuss // Bother and a
    // Forest: two cards, not three.
    let fb = t.hand(P1, "Fuss // Bother");
    t.hand(P1, "Forest");
    t.hand(P1, "Island");
    t.lands(P0, "Swamp", 3);
    let rot = t.hand(P0, "Mind Rot");
    t.answer_choose(P1, &[Entity::Object(fb)]);
    t.cast(P0, rot).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P1), 1);
    assert!(t.in_graveyard(P1, "Fuss // Bother"));
    // Enigma Drake: "power is equal to the number of instant and sorcery cards in your
    // graveyard": the instant and sorcery card counts once.
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Fuss // Bother");
    let drake = t.battlefield(P0, "Enigma Drake");
    t.g.recompute();
    assert_eq!(t.pt(drake).0, 1);
    assert_eq!(
        value(
            &mut t,
            Value::CardsInGraveyard(
                mtg_engine::ability::PlayerRef::You,
                Filter::Or(vec![
                    Filter::Type(CardType::Instant),
                    Filter::Type(CardType::Sorcery)
                ])
            )
        ),
        1
    );
}

#[test]
fn fuss_bother_is_cast_as_one_of_its_halves() {
    cr!("709.3", "709.3a", "709.3b");
    ruling!(
        "Fuss // Bother",
        "To cast a split card, choose one of its halves to cast. There's no way to cast both halves of any of the split cards featured in this set."
    );
    supported("Fuss // Bother");
    let mut t = TestGame::new(2);
    let fb = t.hand(P0, "Fuss // Bother");
    // Its only ways to cast it are its two halves (no fuse).
    let methods: Vec<CastMethod> =
        t.g.cast_options(P0, fb)
            .into_iter()
            .map(|o| o.method)
            .collect();
    assert_eq!(methods, vec![CastMethod::Half(0), CastMethod::Half(1)]);
    // Casting Bother: only Bother is on the stack.
    add_mana(&mut t, P0, ManaType::W, 6);
    let spell = t.cast(P0, fb).method(CastMethod::Half(1)).go();
    assert_eq!(t.obj(spell).chars.name, "Bother");
    assert!(!t.obj(spell).chars.has_name("Fuss"));
    assert!(t.obj(spell).chars.is(CardType::Sorcery));
    assert!(!t.obj(spell).chars.is(CardType::Instant));
    assert_eq!(mana_value(&t, spell), 6);
    t.answer(
        P0,
        DecisionKind::Surveil,
        Answer::Split(vec![], vec![]),
    );
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Thopter").len(), 3);
}

#[test]
fn off_the_stack_fuss_bother_has_the_combined_characteristics() {
    cr!("709.4", "709.4b");
    ruling!(
        "Fuss // Bother",
        "A split card's characteristics are a combination of its two halves while it is not on the stack. For example, Cease // Desist has a mana value of 8 while it is in your library. If an effect allows you to search your library for a card with mana value 4 or less, you can't find Cease // Desist."
    );
    supported("Fuss // Bother");
    let mut t = TestGame::new(2);
    let fb = t.library_top(P0, "Fuss // Bother");
    t.g.recompute();
    // {2}{R/W} + {4}{W/U}{W/U}: mana value 9, red, white, and blue, instant and sorcery.
    assert_eq!(mana_value(&t, fb), 9);
    let c = &t.obj(fb).chars;
    assert!(c.is(CardType::Instant) && c.is(CardType::Sorcery));
    for color in [Color::Red, Color::White, Color::Blue] {
        assert!(c.colors.contains(color));
    }
    // A search for a card with mana value 4 or less can't find it.
    let small = Filter::ManaValue(Cmp::Le, Box::new(Value::c(4)));
    assert!(!t.g.matches(fb, &small, &Ctx::new(None, P0)));
}

#[test]
fn sram_expertise_can_cast_fuss_but_not_bother() {
    cr!("601.3e", "709.3a", "118.9");
    ruling!(
        "Fuss // Bother",
        "If an effect allows you to cast a spell with certain characteristics, consider only the characteristics of the half you're casting. For example, if an effect allows you to cast an instant or sorcery spell with mana value 2 or less from among cards in your graveyard, you could cast Cease this way, but not Desist."
    );
    supported("Fuss // Bother");
    supported("Sram's Expertise");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.hand(P0, "Fuss // Bother");
    // Fuss (mana value 3) can be cast this way; Bother (6) can't. P0 casts Fuss.
    let options = sram_expertise(&mut t, 1);
    assert_eq!(options, vec!["Don't cast a spell", "Cast Fuss"]);
    assert_eq!(t.stack_len(), 1);
    let fuss = t.g.stack[0];
    assert_eq!(t.obj(fuss).chars.name, "Fuss");
    // It was cast from P0's hand without paying its mana cost: no mana was spent on it.
    let cast = &t.obj(fuss).stack.as_ref().expect("a spell").cast;
    assert_eq!(cast.method, CastMethod::Free);
    assert!(cast.mana_spent.is_empty());
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Fuss // Bother"));
    // "Put a +1/+1 counter on each attacking creature you control": none attacking.
    assert_eq!(t.counters(bears, "+1/+1"), 0);
}

#[test]
fn sram_expertise_can_cast_expansion_but_not_explosion() {
    cr!("601.3e", "709.3a");
    ruling!(
        "Expansion // Explosion",
        "If an effect allows you to cast a spell with certain characteristics, consider only the characteristics of the half you're casting. For example, if an effect allows you to cast a sorcery spell with mana value 2 or less from among cards in your graveyard, you could cast Assault this way, but not Battery."
    );
    supported("Expansion // Explosion");
    supported("Sram's Expertise");
    let mut t = TestGame::new(2);
    let ee = t.hand(P0, "Expansion // Explosion");
    // Off the stack, Expansion // Explosion has mana value 6 ({U/R}{U/R} + {X}{U}{U}{R}{R}),
    // but Expansion is a spell with mana value 2; Explosion's is at least 4.
    t.g.recompute();
    assert_eq!(mana_value(&t, ee), 6);
    let options = sram_expertise(&mut t, 0);
    assert_eq!(options, vec!["Don't cast a spell", "Cast Expansion"]);
    // P0 declined: nothing was cast.
    assert_eq!(t.stack_len(), 0);
    assert!(t.in_hand(P0, "Expansion // Explosion"));
}

#[test]
fn a_spell_with_x_cast_with_sram_expertise_has_x_zero() {
    cr!("107.3b");
    ruling!(
        "Sram's Expertise",
        "If the card has {X} in its mana cost, you must choose 0 as the value of X when casting it without paying its mana cost."
    );
    supported("Sram's Expertise");
    supported("Endless One");
    let mut t = TestGame::new(2);
    // Endless One {X}: "This creature enters with X +1/+1 counters on it." (A 0/0.)
    t.hand(P0, "Endless One");
    // P0 would choose X = 5 if the choice were P0's.
    t.answer(P0, DecisionKind::X, Answer::Number(5));
    let options = sram_expertise(&mut t, 1);
    assert_eq!(options, vec!["Don't cast a spell", "Cast Endless One"]);
    assert_eq!(t.stack_len(), 1);
    let spell = t.g.stack[0];
    let cast = &t.obj(spell).stack.as_ref().expect("a spell").cast;
    assert_eq!(cast.x.unwrap_or(0), 0);
    // It enters with no counters and dies as a 0/0.
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Endless One"));
}
