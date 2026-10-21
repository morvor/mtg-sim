//! Bidding life (`bidding.rs`, `oracle/patterns/choice_grammar_bidding.rs`): Illicit
//! Auction, Pain's Reward, Mages' Contest.

use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn compiles(name: &str) {
    let c = mtg_engine::card::card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name}: {:?}",
        c.unsupported_text()
    );
}

/// `p` tops the high bid with `n` life (or declines, with `None`).
fn top(t: &mut TestGame, p: PlayerId, n: Option<i64>) {
    t.answer_yes(p, n.is_some());
    if let Some(n) = n {
        t.answer(p, DecisionKind::Number, Answer::Number(n));
    }
}

#[test]
fn illicit_auction_the_high_bidder_loses_the_bid_and_gains_control() {
    cr!("101.4", "608.2c");
    compiles("Illicit Auction");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 5);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let spell = t.hand(P0, "Illicit Auction");
    t.advance_to(P0, Step::PrecombatMain);
    top(&mut t, P1, Some(3));
    top(&mut t, P0, Some(5));
    top(&mut t, P1, None);
    t.cast(P0, spell).target(bears).go();
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (15, 20));
    assert_eq!(t.obj_now(bears).controller, P0);
}

#[test]
fn illicit_auction_the_bidding_ends_once_the_high_bid_stands() {
    cr!("101.4");
    ruling!(
        "Pain's Reward",
        "Bidding continues until all players except the one with the current high bid have declined"
    );
    let mut t = TestGame::new(3);
    t.lands(P0, "Mountain", 5);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let spell = t.hand(P0, "Illicit Auction");
    t.advance_to(P0, Step::PrecombatMain);
    // Opening bid 0 by P0; P1 bids 1; P2 declines; P0 declines: P1 wins.
    top(&mut t, P1, Some(1));
    top(&mut t, P2, None);
    top(&mut t, P0, None);
    t.cast(P0, spell).target(bears).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    assert_eq!(t.obj_now(bears).controller, P1);
    let yes_no = t
        .asked()
        .iter()
        .filter(|(_, d)| matches!(d, mtg_engine::decision::Decision::YesNo { .. }))
        .count();
    assert_eq!(yes_no, 3, "P1 isn't asked to top their own bid");
}

#[test]
fn pains_reward_you_open_with_any_number_and_may_bid_more_life_than_you_have() {
    cr!("101.4", "119.3");
    ruling!("Pain's Reward", "A player may bid more life than they have");
    ruling!(
        "Illicit Auction",
        "This is a life loss and not a life payment so you can bid more life than you have"
    );
    compiles("Pain's Reward");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 3);
    let spell = t.hand(P0, "Pain's Reward");
    t.advance_to(P0, Step::PrecombatMain);
    t.g.players[1].life = 3;
    t.answer(P0, DecisionKind::Number, Answer::Number(2));
    top(&mut t, P1, Some(10));
    top(&mut t, P0, None);
    let hand = t.hand_size(P1);
    t.cast(P0, spell).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.hand_size(P1), hand + 4);
    assert!(t.has_lost(P1), "lost 10 life with 3");
}

#[test]
fn mages_contest_counters_the_spell_if_you_win_the_bidding() {
    cr!("101.4", "701.6a");
    compiles("Mages' Contest");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P1, "Mountain", 1);
    t.lands(P0, "Mountain", 3);
    let bolt = t.hand(P1, "Lightning Bolt");
    let spell = t.cast(P1, bolt).target(Entity::Player(P0)).go();
    let contest = t.hand(P0, "Mages' Contest");
    top(&mut t, P1, Some(2));
    top(&mut t, P0, Some(3));
    top(&mut t, P1, None);
    t.cast(P0, contest).target(spell).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 17, "lost the bid, not the Bolt");
    assert!(t.in_graveyard(P1, "Lightning Bolt"));
    assert_eq!(t.life(P1), 20);
}

#[test]
fn mages_contest_loses_the_bidding_and_the_spell_resolves() {
    cr!("101.4");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P1, "Mountain", 1);
    t.lands(P0, "Mountain", 3);
    let bolt = t.hand(P1, "Lightning Bolt");
    let spell = t.cast(P1, bolt).target(Entity::Player(P0)).go();
    let contest = t.hand(P0, "Mages' Contest");
    top(&mut t, P1, Some(4));
    top(&mut t, P0, None);
    t.cast(P0, contest).target(spell).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 16, "the high bidder loses the bid");
    assert_eq!(t.life(P0), 17, "the Bolt resolves");
}
