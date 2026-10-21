//! Votes whose results name the chosen spell ("If denial gets more votes, counter the
//! spell", CR 701.38).

use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::*;

fn supported(name: &str) {
    let def = card(name);
    assert!(
        def.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        def.unsupported_text()
    );
}

/// P0 casts Lightning Bolt at P1, then Split Decision on it; votes: 0 = denial, 1 =
/// duplication.
fn split(votes: &[usize]) -> TestGame {
    supported("Split Decision");
    let mut t = TestGame::new(votes.len());
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Island", 2);
    let bolt = t.hand(P0, "Lightning Bolt");
    let sd = t.hand(P0, "Split Decision");
    t.cast(P0, bolt).target(Entity::Player(P1)).go();
    for (p, v) in [P0, P1, P2].iter().zip(votes) {
        t.answer(*p, DecisionKind::Option, Answer::Index(*v));
    }
    // The copy keeps its target.
    t.answer_yes(P0, false);
    t.cast(P0, sd).target(Entity::Object(bolt)).go();
    t.resolve_all();
    t
}

#[test]
fn split_decision_denial_counters_the_spell() {
    cr!("701.38a");
    let t = split(&[0, 0, 1]);
    assert_eq!(t.life(P1), 20);
    assert!(t.in_graveyard(P0, "Lightning Bolt"));
}

#[test]
fn split_decision_tie_copies_the_spell() {
    cr!("701.38a", "707.10");
    ruling!(
        "Split Decision",
        "The phrase \"the vote is tied\" refers only to when there is more than one choice that received the most votes."
    );
    // 1 to 1: tied, so the spell is copied.
    let t = split(&[0, 1]);
    assert_eq!(t.life(P1), 14);
    // 1 to 2 among three players: duplication gets more votes.
    let t = split(&[0, 1, 1]);
    assert_eq!(t.life(P1), 14);
}
