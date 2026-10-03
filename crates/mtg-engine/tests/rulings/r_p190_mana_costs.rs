//! Rulings batch P190 — unusual mana costs: Phyrexian and monocolored hybrid symbols
//! (CR 107.4, 202.2, 202.3), {X}{X} (CR 107.3), cost reductions locked in before mana
//! abilities (CR 601.2f–g), and Blinkmoth Infusion.

use crate::r_p076_common::mana;
use crate::r_s01_common::{creatures, supported, with_subtype};
use mtg_engine::decision::Answer;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// Answers the next "How will you pay {..}?" choice (0 = either way, then each half, then
/// life for a Phyrexian symbol).
fn pay_way(t: &mut TestGame, p: PlayerId, how: usize) {
    t.answer(p, DecisionKind::Option, Answer::Index(how));
}

/// Asserts that `name`'s only unsupported text is the one block containing `needle` (an
/// ability the rulings tested with it don't concern).
pub fn only_unsupported(name: &str, needle: &str) {
    let c = card(name);
    let u = c.unsupported_text();
    assert_eq!(u.len(), 1, "{name}: {u:?}");
    assert!(u[0].contains(needle), "{name}: {u:?}");
}

#[test]
fn phyrexian_symbols_count_one_toward_mana_value_even_paid_with_life() {
    cr!("107.4f", "202.3", "702.150a");
    ruling!(
        "Jace, the Perfected Mind",
        "A Phyrexian mana symbol contributes 1 toward the mana value of a card, even if life is paid for it. Specifically, Jace's mana value is always 4."
    );
    ruling!(
        "Nissa, Ascended Animist",
        "A Phyrexian mana symbol contributes 1 toward the mana value of a card, even if life is paid for it. Specifically, Nissa's mana value is always 7."
    );
    supported("Jace, the Perfected Mind");
    supported("Nissa, Ascended Animist");
    // Jace {2}{U}{U/P}: {U/P} paid with 2 life.
    let mut t = TestGame::new(2);
    let jace = t.hand(P0, "Jace, the Perfected Mind");
    assert_eq!(t.g.mana_value_of(jace), 4);
    mana(&mut t, P0, ManaType::U, 1);
    mana(&mut t, P0, ManaType::C, 2);
    pay_way(&mut t, P0, 2);
    let spell = t.cast(P0, jace).go();
    assert_eq!(t.life(P0), 18);
    assert_eq!(t.g.mana_value_of(spell), 4);
    t.resolve_all();
    assert!(t.on_battlefield(jace));
    assert_eq!(t.g.mana_value_of(t.g.current(jace)), 4);
    // Compleated: two fewer loyalty counters for the symbol paid with life.
    assert_eq!(t.counters(jace, "loyalty"), 3);
    // Nissa {3}{G}{G}{G/P}{G/P}: both Phyrexian symbols paid with life.
    let mut t = TestGame::new(2);
    let nissa = t.hand(P0, "Nissa, Ascended Animist");
    assert_eq!(t.g.mana_value_of(nissa), 7);
    mana(&mut t, P0, ManaType::G, 2);
    mana(&mut t, P0, ManaType::C, 3);
    pay_way(&mut t, P0, 2);
    pay_way(&mut t, P0, 2);
    let spell = t.cast(P0, nissa).go();
    assert_eq!(t.life(P0), 16);
    assert_eq!(t.g.mana_value_of(spell), 7);
    t.resolve_all();
    assert!(t.on_battlefield(nissa));
    assert_eq!(t.g.mana_value_of(t.g.current(nissa)), 7);
    assert_eq!(t.counters(nissa, "loyalty"), 3);
}

#[test]
fn monocolored_hybrid_cards_keep_their_color_and_mana_value_however_paid() {
    cr!("107.4e", "202.2", "202.3f", "601.2f");
    ruling!(
        "Advice from the Fae",
        "A card with a monocolored hybrid mana symbol in its mana cost is each of the colors that appears in its mana cost, regardless of what mana was spent to cast it. Thus, Advice from the Fae is blue, even if you spend six black mana to cast it."
    );
    ruling!(
        "Beseech the Queen",
        "A card with a monocolored hybrid mana symbol in its mana cost is each of the colors that appears in its mana cost, regardless of what mana was spent to cast it. Thus, Beseech the Queen is black even if you spend six red mana to cast it."
    );
    ruling!(
        "Tower Above",
        "A card with a monocolored hybrid mana symbol in its mana cost is each of the colors that appears in its mana cost, regardless of what mana was spent to cast it. Thus, Tower Above is green even if you spend six white mana to cast it."
    );
    ruling!(
        "Advice from the Fae",
        "A card with monocolored hybrid mana symbols in its mana cost has a mana value equal to the highest possible cost it could be cast for. Its mana value never changes. Thus, Advice from the Fae has a mana value of 6, even if you spend {U}{U}{U} to cast it."
    );
    ruling!(
        "Beseech the Queen",
        "A card with monocolored hybrid mana symbols in its mana cost has a mana value equal to the highest possible cost it could be cast for. Its mana value never changes. Thus, Beseech the Queen has a mana value of 6, even if you spend {B}{B}{B} to cast it."
    );
    ruling!(
        "Tower Above",
        "A card with monocolored hybrid mana symbols in its mana cost has a mana value equal to the highest possible cost it could be cast for. Its mana value never changes. Thus, Tower Above has a mana value of 6, even if you spend {G}{G}{G} to cast it."
    );
    supported("Beseech the Queen");
    supported("Tower Above");
    only_unsupported("Advice from the Fae", "Look at the top five cards");
    // (card, its color, its color's mana, another color's mana)
    let cases = [
        ("Advice from the Fae", Color::Blue, ManaType::U, ManaType::B),
        ("Beseech the Queen", Color::Black, ManaType::B, ManaType::R),
        ("Tower Above", Color::Green, ManaType::G, ManaType::W),
    ];
    for (name, color, own, other) in cases {
        for (ty, n) in [(other, 6), (own, 3)] {
            let mut t = TestGame::new(2);
            let bears = t.battlefield(P0, "Grizzly Bears");
            let c = t.hand(P0, name);
            mana(&mut t, P0, ty, n);
            let spell = if name == "Tower Above" {
                t.cast(P0, c).target(bears).go()
            } else {
                t.cast(P0, c).go()
            };
            assert_eq!(t.g.player(P0).mana_pool.total(), 0, "{name}: paid {n}");
            assert_eq!(t.obj(spell).chars.colors, ColorSet::single(color), "{name}");
            assert_eq!(t.g.mana_value_of(spell), 6, "{name}: paid {n}");
        }
    }
}

#[test]
fn x_x_means_paying_twice_x() {
    cr!("107.3", "107.3a", "601.2f");
    ruling!(
        "Entreat the Angels",
        "A mana cost of {X}{X} means that you pay twice X. If you want X to be 3, you pay {6}{W}{W}{W} to cast Entreat the Angels."
    );
    ruling!(
        "Decree of Justice",
        "A mana cost of {X}{X} means that you pay twice X. If you want X to be 3, you pay {8}{W}{W} to cast Decree of Justice."
    );
    supported("Entreat the Angels");
    supported("Decree of Justice");
    // (card, generic, white) for X = 3.
    for (name, generic, white) in [("Entreat the Angels", 6, 3), ("Decree of Justice", 8, 2)] {
        // One generic mana short: it can't be cast with X = 3.
        let mut t = TestGame::new(2);
        let c = t.hand(P0, name);
        mana(&mut t, P0, ManaType::C, generic - 1);
        mana(&mut t, P0, ManaType::W, white);
        assert!(t.cast(P0, c).x(3).try_go().is_err(), "{name}");
        // Exactly twice X plus the rest.
        let mut t = TestGame::new(2);
        let c = t.hand(P0, name);
        mana(&mut t, P0, ManaType::C, generic);
        mana(&mut t, P0, ManaType::W, white);
        t.cast(P0, c).x(3).go();
        assert_eq!(t.g.player(P0).mana_pool.total(), 0, "{name}");
        t.resolve_all();
        assert_eq!(with_subtype(&t, P0, "Angel").len(), 3, "{name}");
    }
}

#[test]
fn wildgrowth_archaic_two_hybrid_paid_with_two_green() {
    cr!("107.4e", "601.2f", "601.2h");
    ruling!(
        "Wildgrowth Archaic",
        "For each of the hybrid mana symbols in Wildgrowth Archaic's mana cost, you can choose to pay either {2} or {G}. If you choose to pay {2}, you can still use green mana to pay for one or both of the generic mana in that cost."
    );
    supported("Wildgrowth Archaic");
    // {2/G}{2/G}: both paid as {2}, with four green mana.
    let mut t = TestGame::new(2);
    let w = t.hand(P0, "Wildgrowth Archaic");
    mana(&mut t, P0, ManaType::G, 4);
    pay_way(&mut t, P0, 2);
    pay_way(&mut t, P0, 2);
    t.cast(P0, w).go();
    assert_eq!(t.g.player(P0).mana_pool.total(), 0);
    // One as {G}, one as {2} paid with one green and one colorless.
    let mut t = TestGame::new(2);
    let w = t.hand(P0, "Wildgrowth Archaic");
    mana(&mut t, P0, ManaType::G, 2);
    mana(&mut t, P0, ManaType::C, 1);
    pay_way(&mut t, P0, 1);
    pay_way(&mut t, P0, 2);
    t.cast(P0, w).go();
    assert_eq!(t.g.player(P0).mana_pool.total(), 0);
    t.resolve_all();
    assert!(t.on_battlefield(w));
}

#[test]
fn khalni_hydra_reduction_counts_green_creatures_before_mana_abilities() {
    cr!("601.2f", "601.2g", "601.2h");
    ruling!(
        "Khalni Hydra",
        "For the purpose of determining the cost reduction, the number of green creatures you control is checked as you cast Khalni Hydra, before your last chance to activate mana abilities to pay for it."
    );
    supported("Khalni Hydra");
    supported("Wild Cantor");
    // Wild Cantor (red and green) reduces the cost to {G}x7, then is sacrificed for mana
    // to help pay for it: six Forests and the Cantor pay for the Hydra.
    let mut t = TestGame::new(2);
    let cantor = t.battlefield(P0, "Wild Cantor");
    t.lands(P0, "Forest", 6);
    let hydra = t.hand(P0, "Khalni Hydra");
    t.cast(P0, hydra).go();
    assert!(!t.on_battlefield(cantor));
    t.resolve_all();
    assert!(t.on_battlefield(hydra));
    assert_eq!(creatures(&t, P0), vec![t.g.current(hydra)]);
}

#[test]
fn blinkmoth_infusion_untaps_every_artifact() {
    cr!("701.26b");
    ruling!(
        "Blinkmoth Infusion",
        "Blinkmoth Infusion untaps all artifacts on the battlefield, not just artifacts you control."
    );
    supported("Blinkmoth Infusion");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Ornithopter");
    let theirs = t.battlefield(P1, "Ornithopter");
    for id in [mine, theirs] {
        t.g.objects[id.0 as usize].tapped = true;
    }
    let c = t.hand(P0, "Blinkmoth Infusion");
    mana(&mut t, P0, ManaType::U, 2);
    // Affinity counts only P0's Ornithopter: {11}{U}{U}.
    mana(&mut t, P0, ManaType::C, 11);
    t.cast(P0, c).go();
    t.resolve_all();
    assert!(!t.obj(mine).tapped);
    assert!(!t.obj(theirs).tapped);
}
