//! Rulings batch S22 — the Lorwyn Bannerets: "[A] spells and [B] spells you cast cost {1}
//! less to cast." A spell of both kinds costs {1} less, not {2}; the reduction applies to
//! the total cost, whatever alternative or additional costs were chosen (CR 601.2f).

use crate::r_s01_common::*;
use crate::r_s07_common::cast_methods;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const PROWL: CastMethod = CastMethod::Keyword(KeywordKind::Prowl);

#[test]
fn frogtosser_banneret_a_spell_of_both_types_costs_one_less_not_two() {
    cr!("601.2f", "205.3m");
    ruling!(
        "Frogtosser Banneret",
        "A spell you cast that’s both creature types costs {1} less to cast, not {2} less."
    );
    supported("Frogtosser Banneret");
    supported("Stinkdrinker Bandit");
    // "Goblin spells and Rogue spells you cast cost {1} less to cast." Stinkdrinker
    // Bandit {3}{B} is a Goblin Rogue: {2}{B}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Frogtosser Banneret");
    t.lands(P0, "Swamp", 2);
    let bandit = t.hand(P0, "Stinkdrinker Bandit");
    assert!(t.cast(P0, bandit).try_go().is_err());
    t.lands(P0, "Swamp", 1);
    t.cast(P0, bandit).go();
    assert_eq!(tapped_lands(&t, P0), 3);
    // A Goblin that isn't a Rogue, and a Rogue that isn't a Goblin, cost {1} less too.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Frogtosser Banneret");
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Swamp", 1);
    let wort = t.hand(P0, "Wort, Boggart Auntie");
    // {2}{B}{R}: {1}{B}{R} isn't payable with two lands; three are needed.
    assert!(t.cast(P0, wort).try_go().is_err());
    t.lands(P0, "Swamp", 1);
    t.cast(P0, wort).go();
    assert_eq!(tapped_lands(&t, P0), 3);
    // Not a Goblin or a Rogue: full cost.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Frogtosser Banneret");
    t.lands(P0, "Forest", 1);
    let bears = t.hand(P0, "Grizzly Bears");
    assert!(t.cast(P0, bears).try_go().is_err());
}

#[test]
fn frogtosser_banneret_reduces_a_prowl_cost() {
    cr!("601.2f", "702.76a", "118.9");
    ruling!(
        "Frogtosser Banneret",
        "The effect reduces the total cost of the spell, regardless of whether you chose to pay additional or alternative costs. For example, if you cast a Rogue spell by paying its prowl cost, Frogtosser Banneret causes that spell to cost {1} less."
    );
    // Frogtosser Banneret (a Goblin Rogue) deals combat damage to P1: Stinkdrinker Bandit's
    // prowl {1}{B} is available, and costs {B}.
    let mut t = TestGame::new(2);
    let banneret = t.battlefield(P0, "Frogtosser Banneret");
    let bandit = t.hand(P0, "Stinkdrinker Bandit");
    attack_with(&mut t, &[(banneret, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.life(P1), 19);
    t.g.combat = None;
    t.set_step(P0, Step::PostcombatMain);
    t.lands(P0, "Swamp", 4);
    assert!(cast_methods(&mut t, P0, bandit).contains(&PROWL));
    t.cast(P0, bandit).method(PROWL).go();
    assert_eq!(tapped_lands(&t, P0), 1);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Stinkdrinker Bandit").len(), 1);
}
