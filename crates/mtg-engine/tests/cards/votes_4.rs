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

#[test]
fn davros_each_opponent_who_lost_three_life_faces_a_villainous_choice() {
    cr!("701.55a", "701.55d");
    supported("Davros, Dalek Creator");
    let mut t = TestGame::new(4);
    t.battlefield(P0, "Davros, Dalek Creator");
    t.g.lose_life(P1, 3);
    t.g.lose_life(P2, 2);
    t.g.lose_life(P3, 4);
    let card = t.hand(P1, "Grizzly Bears");
    // P1 chooses "that player discards a card"; P3 chooses "You draw a card".
    t.answer(P1, DecisionKind::Option, Answer::Index(1));
    t.answer_choose(P1, &[Entity::Object(card)]);
    t.answer(P3, DecisionKind::Option, Answer::Index(0));
    t.advance_to(P0, mtg_engine::turn::Step::End);
    let hand = t.hand_size(P0);
    t.resolve_all();
    let daleks =
        t.g.battlefield
            .iter()
            .filter(|o| t.obj_now(**o).chars.name.starts_with("Dalek"))
            .count();
    assert_eq!(daleks, 1);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(t.hand_size(P0), hand + 1);
    let faced: Vec<PlayerId> = t
        .asked()
        .into_iter()
        .filter_map(|(p, d)| match d {
            decision::Decision::ChooseOption { prompt, .. } if prompt == "Villainous choice" => {
                Some(p)
            }
            _ => None,
        })
        .collect();
    assert_eq!(faced, vec![P1, P3]);
}

#[test]
fn seize_the_spotlight_fame_steals_and_untaps_fortune_draws() {
    cr!("101.4", "608.2c");
    supported("Seize the Spotlight");
    let mut t = TestGame::new(4);
    t.lands(P0, "Mountain", 3);
    let spell = t.hand(P0, "Seize the Spotlight");
    let b1 = t.battlefield(P1, "Grizzly Bears");
    let b3 = t.battlefield(P3, "Hill Giant");
    t.g.objects[b1.0 as usize].tapped = true;
    t.g.objects[b3.0 as usize].tapped = true;
    // P1 and P3 choose fame, P2 fortune.
    t.answer(P1, DecisionKind::Option, Answer::Index(0));
    t.answer(P2, DecisionKind::Option, Answer::Index(1));
    t.answer(P3, DecisionKind::Option, Answer::Index(0));
    t.answer_choose(P0, &[Entity::Object(b1)]);
    t.answer_choose(P0, &[Entity::Object(b3)]);
    let hand = t.hand_size(P0);
    t.cast(P0, spell).go();
    t.resolve();
    for c in [b1, b3] {
        let o = t.obj_now(c);
        assert_eq!(o.controller, P0);
        assert!(!o.tapped);
        assert!(o.has_keyword(keywords::KeywordKind::Haste));
    }
    // One fortune: one card and one Treasure.
    assert_eq!(t.hand_size(P0), hand - 1 + 1);
    let treasures =
        t.g.battlefield
            .iter()
            .filter(|o| t.obj_now(**o).chars.name.starts_with("Treasure"))
            .count();
    assert_eq!(treasures, 1);
}
