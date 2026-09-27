//! Rulings batch S03 — converge (an ability word): "Converge — ... for each color of mana
//! spent to cast this spell" / "where X is the number of colors of mana spent to cast this
//! spell".

use crate::r_s01_common::*;
use crate::r_s03_common::*;
use mtg_engine::ability::{Effect, PlayerRef, Sel};
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::*;

/// Adds one mana of each of `types` to `p`'s mana pool.
fn float(t: &mut TestGame, p: PlayerId, types: &[ManaType]) {
    for ty in types {
        t.g.players[p.idx()].mana_pool.add_type(*ty, 1);
    }
}

fn pool(t: &TestGame, p: PlayerId) -> usize {
    t.g.player(p).mana_pool.total()
}

const WUBRG: [ManaType; 5] = [
    ManaType::W,
    ManaType::U,
    ManaType::B,
    ManaType::R,
    ManaType::G,
];

#[test]
fn a_copy_of_a_converge_spell_counts_no_colors() {
    cr!("707.10", "207.2c", "601.2h");
    ruling!(
        "Painful Truths",
        "If a spell with a converge ability is copied, no mana was spent to cast the copy, so the number of colors of mana spent to cast the spell will be zero. The number of colors spent to cast the original spell is not copied."
    );
    supported("Painful Truths");
    supported("Mica, Reader of Ruins");
    // Painful Truths ({2}{B}): "Converge — You draw X cards and lose X life, where X is
    // the number of colors of mana spent to cast this spell." Mica: "Whenever you cast an
    // instant or sorcery spell, you may sacrifice an artifact. If you do, copy that spell
    // ..."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mica, Reader of Ruins");
    let thopter = t.battlefield(P0, "Ornithopter");
    let truths = t.hand(P0, "Painful Truths");
    float(&mut t, P0, &[ManaType::W, ManaType::U, ManaType::B]);
    let hand = t.hand_size(P0) - 1;
    t.cast(P0, truths).go();
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(thopter)]);
    t.settle();
    // The copy resolves first: nothing.
    t.resolve(); // Mica's trigger: the copy is put on the stack
    assert!(t.in_graveyard(P0, "Ornithopter"));
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    assert_eq!(t.hand_size(P0), hand);
    assert_eq!(t.life(P0), 20);
    // The original: three colors.
    t.resolve();
    assert_eq!(t.hand_size(P0), hand + 3);
    assert_eq!(t.life(P0), 17);
}

#[test]
fn a_converge_spell_cast_without_spending_mana_counts_no_colors() {
    cr!("207.2c", "118.9");
    ruling!(
        "Crystalline Crawler",
        "If you cast a spell with converge without spending any mana to cast it (perhaps because an effect allowed you to cast it without paying its mana cost), then the number of colors spent to cast it will be zero."
    );
    supported("Crystalline Crawler");
    supported("Unified Front");
    // Crystalline Crawler: "Converge — This creature enters with a +1/+1 counter on it for
    // each color of mana spent to cast it." Unified Front: "Converge — Create a 1/1 white
    // Kor Ally creature token for each color of mana spent to cast this spell."
    let mut t = TestGame::new(2);
    float(&mut t, P0, &WUBRG);
    for name in ["Crystalline Crawler", "Unified Front"] {
        let c = t.hand(P0, name);
        run_effect(
            &mut t,
            None,
            P0,
            Effect::CastCard {
                who: PlayerRef::You,
                what: Sel::Target(0),
                free: true,
                optional: false,
            },
            &[Entity::Object(c)],
        );
        t.resolve_all();
    }
    assert_eq!(pool(&t, P0), 5);
    let crawler = t.named_on_battlefield("Crystalline Crawler")[0];
    assert_eq!(t.counters(crawler, "+1/+1"), 0);
    assert!(tokens(&t, P0).is_empty());
}

#[test]
fn you_cant_pay_more_mana_or_ignore_a_cost_reduction_to_spend_more_colors() {
    cr!("207.2c", "601.2f", "601.2h", "118.7");
    ruling!(
        "Crystalline Crawler",
        "Unless a spell or ability allows you to, you can't choose to pay more mana for a spell with a converge ability just to spend more colors of mana. Likewise, if a spell or ability reduces the amount of mana it costs you to cast a spell with converge, you can't ignore that cost reduction in order to spend more colors of mana."
    );
    ruling!(
        "Unified Front",
        "Unless a spell or ability allows you to, you can’t choose to pay more mana for a spell with a converge ability just to spend more colors of mana. Likewise, if a spell or ability reduces the amount of mana it costs you to cast a spell with converge, you can’t ignore that cost reduction in order to spend more colors of mana."
    );
    supported("Etherium Sculptor");
    supported("Goblin Electromancer");
    // With five colors of mana available, the Crawler ({4}) is paid with four of them.
    let mut t = TestGame::new(2);
    float(&mut t, P0, &WUBRG);
    let c = t.hand(P0, "Crystalline Crawler");
    t.cast(P0, c).go();
    t.resolve_all();
    assert_eq!(pool(&t, P0), 1);
    assert_eq!(t.counters(c, "+1/+1"), 4);
    // Etherium Sculptor ("Artifact spells you cast cost {1} less to cast"): three.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Etherium Sculptor");
    float(&mut t, P0, &WUBRG);
    let c = t.hand(P0, "Crystalline Crawler");
    t.cast(P0, c).go();
    t.resolve_all();
    assert_eq!(pool(&t, P0), 2);
    assert_eq!(t.counters(c, "+1/+1"), 3);
    // Unified Front ({3}{W}): four tokens; with Goblin Electromancer ("Instant and
    // sorcery spells you cast cost {1} less to cast"), three.
    for (reduced, n) in [(false, 4), (true, 3)] {
        let mut t = TestGame::new(2);
        if reduced {
            t.battlefield(P0, "Goblin Electromancer");
        }
        float(&mut t, P0, &WUBRG);
        let c = t.hand(P0, "Unified Front");
        t.cast(P0, c).go();
        t.resolve_all();
        assert_eq!(pool(&t, P0), 5 - n);
        assert_eq!(tokens(&t, P0).len(), n);
    }
}

#[test]
fn colors_of_mana_spent_on_additional_costs_count() {
    cr!("207.2c", "601.2f", "118.8");
    ruling!(
        "Sweep the Skies",
        "If there are any alternative or additional costs to cast a spell with a converge ability, the colors of mana spent to pay those costs will count."
    );
    ruling!(
        "Radiant Flames",
        "For example, if an effect makes sorcery spells cost {1} more to cast, you could pay {W}{U}{B}{R} to cast Radiant Flames and deal 4 damage to each creature."
    );
    supported("Sweep the Skies");
    supported("Radiant Flames");
    supported("Sphere of Resistance");
    // Sweep the Skies ({X}{U}{U}) with X = 1: "Create a 1/1 colorless Thopter artifact
    // creature token with flying for each color of mana spent to cast this spell." With
    // Sphere of Resistance ("Spells cost {1} more to cast"), {1}{U}{U} becomes
    // {2}{U}{U}: blue, white and black mana pay it.
    for (sphere, n) in [(false, 2), (true, 3)] {
        let mut t = TestGame::new(2);
        if sphere {
            t.battlefield(P1, "Sphere of Resistance");
        }
        float(
            &mut t,
            P0,
            &[ManaType::U, ManaType::U, ManaType::W, ManaType::B],
        );
        let c = t.hand(P0, "Sweep the Skies");
        t.cast(P0, c).x(1).go();
        t.resolve_all();
        assert_eq!(tokens(&t, P0).len(), n);
        assert_eq!(pool(&t, P0), if sphere { 0 } else { 1 });
    }
    // Radiant Flames ({2}{R}) with Sphere of Resistance: {W}{U}{B}{R} pays {3}{R}, 4
    // damage to each creature.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Sphere of Resistance");
    let giant = t.battlefield(P1, "Hill Giant"); // 3/3
    let wall = t.battlefield(P1, "Wall of Stone"); // 0/8
    float(
        &mut t,
        P0,
        &[ManaType::W, ManaType::U, ManaType::B, ManaType::R],
    );
    let c = t.hand(P0, "Radiant Flames");
    t.cast(P0, c).go();
    t.resolve_all();
    assert!(!t.on_battlefield(giant));
    assert_eq!(t.obj(wall).damage, 4);
}
