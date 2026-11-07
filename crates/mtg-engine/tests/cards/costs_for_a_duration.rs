//! Spell cost changes a resolving spell or ability creates for a duration (pattern in
//! `src/oracle/patterns/costs_for_a_duration.rs`, applied as
//! `PlayerModification::CostModifier` player effects): Will Kenrith's −2, Tax Collector's
//! "Tax" mode (CR 601.2f, 611.2a).

use mtg_engine::decision::Answer;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn assert_compiles(names: &[&str]) {
    for n in names {
        let u = card(n).unsupported_text().join(" | ");
        assert!(u.is_empty(), "{n} has unsupported text: {u}");
    }
}

/// Whether `p` could begin casting `card` normally now, paying for it with what they have.
fn castable(t: &mut TestGame, p: PlayerId, card: ObjectId) -> bool {
    t.g.recompute();
    t.g.turn.priority = Some(p);
    t.g.cast_options(p, card)
        .into_iter()
        .any(|o| o.method == CastMethod::Normal && t.g.can_begin_cast(p, card, &o))
}

#[test]
fn cost_for_a_duration_cards_compile() {
    assert_compiles(&["Will Kenrith", "Tax Collector"]);
}

#[test]
fn will_kenrith_makes_that_players_spells_cheaper_until_your_next_turn() {
    cr!("601.2f", "611.2a");
    // "−2: Target player draws two cards. Until your next turn, instant, sorcery, and
    // planeswalker spells that player casts cost {2} less to cast."
    let mut t = TestGame::new(2);
    let will = t.battlefield(P0, "Will Kenrith");
    t.activate(P0, will, 1, &[Entity::Player(P1)])
        .expect("Will Kenrith's −2");
    t.resolve_all();
    assert_eq!(t.hand_size(P1), 2);
    // P0's own spells aren't cheaper.
    let p0_div = t.hand(P0, "Divination");
    t.lands(P0, "Island", 1);
    assert!(!castable(&mut t, P0, p0_div));
    // On P1's turn: Divination ({2}{U}) costs {U}; Grizzly Bears (a creature) doesn't
    // cost less.
    t.advance_to(P1, Step::PrecombatMain);
    let div = t.hand(P1, "Divination");
    let bears = t.hand(P1, "Grizzly Bears");
    t.lands(P1, "Island", 1);
    assert!(castable(&mut t, P1, div));
    assert!(!castable(&mut t, P1, bears));
    t.cast(P1, div).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P1), 2 + 1 + 1 + 2);
    // It ends as P0's next turn begins.
    t.advance_to(P0, Step::Upkeep);
    let div = t.hand(P1, "Divination");
    t.lands(P1, "Island", 1);
    t.g.turn.priority = Some(P1);
    let opts = t.g.cast_options(P1, div);
    // (Divination is a sorcery: check the cost, not the timing.)
    let chars = t.obj(div).chars.clone();
    let cost = t.g.base_total_cost(P1, div, &chars, &opts[0], 0);
    assert_eq!(cost.mana.map(|m| m.mana_value()), Some(3));
}

#[test]
fn tax_collector_makes_opponents_spells_cost_more_until_your_next_turn() {
    cr!("601.2f", "611.2a");
    // "When this creature enters, choose one — • Tax — Until your next turn, spells your
    // opponents cast cost {1} more to cast. • Arrest — ..."
    let mut t = TestGame::new(2);
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0]));
    t.enter(P0, "Tax Collector");
    t.resolve_all();
    let bolt = t.hand(P1, "Lightning Bolt");
    t.lands(P1, "Mountain", 1);
    assert!(!castable(&mut t, P1, bolt));
    t.lands(P1, "Mountain", 1);
    assert!(castable(&mut t, P1, bolt));
    // Its controller's spells cost the same.
    let own = t.hand(P0, "Lightning Bolt");
    t.lands(P0, "Mountain", 1);
    assert!(castable(&mut t, P0, own));
}
