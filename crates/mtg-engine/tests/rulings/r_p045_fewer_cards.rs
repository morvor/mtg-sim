//! Rulings batch P045 — a player told to discard (or exile, or reveal) more cards than
//! they have does as much as possible (CR 101.3, 701.9a), and the rest of the effect still
//! happens (CR 608.2c). Table-driven: each case is a spell cast by P0 at P1 with P1
//! holding a given number of cards.

use crate::r_p045_common::*;
use crate::r_s01_common::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// P0 casts `name` (paying its mana cost) targeting P1 (if it has a target) while P1 holds
/// `opp_hand` Grizzly Bears, and everything resolves. P0's hand is otherwise empty; P0
/// controls a Grizzly Bears (for sacrifice costs).
fn cast_at_p1(name: &str, opp_hand: usize, kicked: bool) -> TestGame {
    supported(name);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    bears_in_hand(&mut t, P1, opp_hand);
    let spell = t.hand(P0, name);
    give_mana_for(&mut t, P0, name);
    if kicked {
        t.lands(P0, "Mountain", 1);
    }
    let cast = t.cast(P0, spell);
    let cast = if kicked { cast.kicked(true) } else { cast };
    cast.target(Entity::Player(P1)).go();
    t.resolve_all();
    t
}

struct Case {
    card: &'static str,
    opp_hand: usize,
    kicked: bool,
    /// P1's hand size, P1's life loss, P0's life gain, P0's hand size afterwards.
    expect: (usize, i32, i32, usize),
}

fn run(c: Case) {
    let t = cast_at_p1(c.card, c.opp_hand, c.kicked);
    let got = (
        t.hand_size(P1),
        20 - t.life(P1),
        t.life(P0) - 20,
        t.hand_size(P0),
    );
    assert_eq!(got, c.expect, "{} with {} cards", c.card, c.opp_hand);
    assert_eq!(
        t.graveyard_size(P1),
        c.opp_hand - c.expect.0,
        "{}: discarded",
        c.card
    );
}

#[test]
fn discarding_fewer_cards_than_asked_and_the_rest_still_happens() {
    cr!("101.3", "701.9a", "608.2c");
    ruling!(
        "Purge the Profane",
        "If the opponent has fewer than two cards in their hand, they will discard them. You’ll still gain 2 life"
    );
    run(Case {
        card: "Purge the Profane",
        opp_hand: 1,
        kicked: false,
        expect: (0, 0, 2, 0),
    });
    ruling!(
        "Tendrils of Despair",
        "If the player has fewer than 2 cards, they discard whatever they have."
    );
    run(Case {
        card: "Tendrils of Despair",
        opp_hand: 1,
        kicked: false,
        expect: (0, 0, 0, 0),
    });
    ruling!(
        "Mental Agony",
        "If the targeted player has fewer than two cards in their hand, that player will still lose 2 life (and discard one card, if applicable)."
    );
    run(Case {
        card: "Mental Agony",
        opp_hand: 1,
        kicked: false,
        expect: (0, 2, 0, 0),
    });
    run(Case {
        card: "Mental Agony",
        opp_hand: 0,
        kicked: false,
        expect: (0, 2, 0, 0),
    });
    ruling!(
        "Davriel's Shadowfugue",
        "The target player loses 2 life even if they can discard only one or zero cards."
    );
    run(Case {
        card: "Davriel's Shadowfugue",
        opp_hand: 0,
        kicked: false,
        expect: (0, 2, 0, 0),
    });
    ruling!(
        "Skull Raid",
        "If they have only one card in hand, they’ll discard that card and you’ll draw a card. If they have no cards in hand, they can’t discard any, so you’ll draw two cards."
    );
    for (opp, draws) in [(3, 0), (1, 1), (0, 2)] {
        run(Case {
            card: "Skull Raid",
            opp_hand: opp,
            kicked: false,
            expect: (opp.saturating_sub(2), 0, 0, draws),
        });
    }
    ruling!(
        "Mindculling",
        "If the opponent has one card in hand, that player will discard that card and you will draw two cards. If the opponent has no cards in hand, you will draw two cards."
    );
    for opp in [1, 0] {
        run(Case {
            card: "Mindculling",
            opp_hand: opp,
            kicked: false,
            expect: (0, 0, 0, 2),
        });
    }
    ruling!(
        "Prying Questions",
        "Prying Questions can target an opponent who has no cards in hand. That player just loses 3 life."
    );
    run(Case {
        card: "Prying Questions",
        opp_hand: 0,
        kicked: false,
        expect: (0, 3, 0, 0),
    });
    ruling!(
        "Aggressive Sabotage",
        "If you kick Aggressive Sabotage, it deals 3 damage to that player in addition to making them discard two cards"
    );
    run(Case {
        card: "Aggressive Sabotage",
        opp_hand: 3,
        kicked: true,
        expect: (1, 3, 0, 0),
    });
    run(Case {
        card: "Aggressive Sabotage",
        opp_hand: 3,
        kicked: false,
        expect: (1, 0, 0, 0),
    });
}

#[test]
fn witness_the_end_exiles_what_there_is_and_still_drains() {
    cr!("101.3", "608.2c");
    ruling!(
        "Witness the End",
        "If that player has only one card in hand, that card will be exiled. If the player has no cards in hand, no cards will be exiled, but they will still lose 2 life."
    );
    let t = cast_at_p1("Witness the End", 1, false);
    assert_eq!(t.hand_size(P1), 0);
    assert!(t.in_exile("Grizzly Bears"));
    assert_eq!(t.life(P1), 18);
    let t = cast_at_p1("Witness the End", 0, false);
    assert_eq!(t.life(P1), 18);
}

#[test]
fn noggin_whack_with_fewer_than_three_cards_discards_them_all() {
    cr!("101.3", "701.9a", "701.20a");
    ruling!(
        "Noggin Whack",
        "If the player has fewer than three cards in their hand, the player reveals all of them, you choose all of them, and they're all discarded."
    );
    let t = cast_at_p1("Noggin Whack", 2, false);
    assert_eq!(t.hand_size(P1), 0);
    assert_eq!(t.graveyard_size(P1), 2);
    let t = cast_at_p1("Noggin Whack", 4, false);
    assert_eq!(t.hand_size(P1), 2);
}

#[test]
fn diplomacy_of_the_wastes_with_a_warrior_drains_an_empty_hand() {
    cr!("608.2c");
    ruling!(
        "Diplomacy of the Wastes",
        "If you control a Warrior as Diplomacy of the Wastes resolves, the target opponent will lose 2 life even if that player didn’t discard a card"
    );
    supported("Diplomacy of the Wastes");
    let mut t = TestGame::new(2);
    // Mardu Hordechief is a Human Warrior.
    t.battlefield(P0, "Mardu Hordechief");
    let d = t.hand(P0, "Diplomacy of the Wastes");
    give_mana_for(&mut t, P0, "Diplomacy of the Wastes");
    t.cast(P0, d).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    // Without a Warrior, no life loss.
    let t = cast_at_p1("Diplomacy of the Wastes", 1, false);
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.hand_size(P1), 0);
}

#[test]
fn mind_ravel_still_draws_if_the_opponent_had_no_cards() {
    cr!("603.7a", "608.2c");
    ruling!(
        "Mind Ravel",
        "You still draw a card if opponent had no cards in hand."
    );
    let mut t = cast_at_p1("Mind Ravel", 0, false);
    assert_eq!(t.hand_size(P0), 0);
    // "Draw a card at the beginning of the next turn's upkeep": P1's upkeep.
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 1);
}

/// An "each opponent discards" permanent enters for P0 while P1 holds `opp_hand` cards.
fn enters_vs(name: &str, opp_hand: usize) -> TestGame {
    supported(name);
    let mut t = TestGame::new(2);
    bears_in_hand(&mut t, P1, opp_hand);
    t.enter(P0, name);
    t.resolve_all();
    t
}

#[test]
fn each_opponent_discards_with_empty_or_small_hands() {
    cr!("101.3", "701.9a", "608.2c");
    ruling!(
        "Cackling Fiend",
        "If the opponent has no cards in hand, this has no effect."
    );
    let t = enters_vs("Cackling Fiend", 0);
    assert_eq!(t.hand_size(P1), 0);
    assert_eq!(t.graveyard_size(P1), 0);
    ruling!(
        "Basilica Bell-Haunt",
        "You gain 3 life even if some or all of your opponents can’t discard a card."
    );
    let t = enters_vs("Basilica Bell-Haunt", 0);
    assert_eq!(t.life(P0), 23);
    // Unnerve: "Each opponent discards two cards."
    ruling!("Unnerve", "Players with less than 2 cards discard all they have.");
    supported("Unnerve");
    let mut t = TestGame::new(3);
    bears_in_hand(&mut t, P1, 1);
    bears_in_hand(&mut t, P2, 3);
    let u = t.hand(P0, "Unnerve");
    give_mana_for(&mut t, P0, "Unnerve");
    t.cast(P0, u).go();
    t.resolve_all();
    assert_eq!(hand_sizes(&t.g), vec![0, 0, 1]);
}

#[test]
fn dying_wail_discards_what_there_is_and_triggers_only_from_the_battlefield() {
    cr!("101.3", "603.6c", "700.4");
    ruling!(
        "Dying Wail",
        "If they have less than 2 cards, they discard all the cards they have."
    );
    ruling!(
        "Dying Wail",
        "It only triggers on a creature going to the graveyard from the battlefield."
    );
    supported("Dying Wail");
    // Dying Wail: "Enchant creature. When enchanted creature dies, target player discards
    // two cards."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let wail = t.battlefield(P0, "Dying Wail");
    t.g.attach(wail, bears.into());
    bears_in_hand(&mut t, P1, 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.lands(P0, "Mountain", 1);
    t.cast(P0, bolt).target(bears).go();
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(t.hand_size(P1), 0);
    assert_eq!(t.graveyard_size(P1), 1);
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let wail = t.battlefield(P0, "Dying Wail");
    t.g.attach(wail, bears.into());
    bears_in_hand(&mut t, P1, 2);
    // The enchanted creature is returned to hand (not "dies"): the Aura goes to the
    // graveyard, no trigger.
    let unsummon = t.hand(P0, "Unsummon");
    t.lands(P0, "Island", 1);
    t.cast(P0, unsummon).target(bears).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Dying Wail"));
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.hand_size(P1), 2);
}

#[test]
fn upkeep_and_enters_self_discards_with_an_empty_hand_do_nothing() {
    cr!("101.3", "701.9a");
    ruling!(
        "Rotting Regisaur",
        "If you have no cards in hand when Rotting Regisaur’s triggered ability resolves, you simply don’t discard any cards."
    );
    supported("Rotting Regisaur");
    let mut t = TestGame::new(2);
    let reg = t.battlefield(P1, "Rotting Regisaur");
    t.advance_to(P1, Step::Upkeep);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "discard"), 1);
    t.resolve_all();
    assert!(t.on_battlefield(reg));
    assert_eq!(t.graveyard_size(P1), 0);

    ruling!(
        "Bloodrage Brawler",
        "You can cast Bloodrage Brawler even if you have no other cards in your hand. If you have no cards in hand as its ability resolves, nothing happens."
    );
    supported("Bloodrage Brawler");
    let mut t = TestGame::new(2);
    let b = t.hand(P0, "Bloodrage Brawler");
    give_mana_for(&mut t, P0, "Bloodrage Brawler");
    t.cast(P0, b).go();
    t.resolve_all();
    assert!(t.on_battlefield(b));
    assert_eq!(t.graveyard_size(P0), 0);
}

#[test]
fn rotting_regisaur_discards_in_upkeep_before_the_draw() {
    cr!("503.1a", "504.1");
    ruling!(
        "Rotting Regisaur",
        "Because the upkeep step is before the draw step, you discard for Rotting Regisaur’s triggered ability before you draw a card for your turn."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Rotting Regisaur");
    let bears = t.hand(P1, "Grizzly Bears");
    t.advance_to(P1, Step::Upkeep);
    let lib = t.library_size(P1);
    t.resolve_all();
    // The only card P1 could discard was the Bears: the draw hasn't happened yet.
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(t.zone(bears), mtg_engine::object::Zone::Graveyard(P1));
    assert_eq!(t.library_size(P1), lib);
    t.advance_to(P1, Step::Draw);
    assert_eq!(t.hand_size(P1), 1);
}
