//! CR 702.132 Assist.

use crate::common_k702_125_139::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::game::GameConfig;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::*;

fn untapped(t: &TestGame, ids: &[ObjectId]) -> usize {
    ids.iter().filter(|id| !t.obj_now(**id).tapped).count()
}

#[test]
fn another_player_may_pay_generic_mana_of_a_spell_with_assist() {
    cr!("702.132", "702.132a");
    assert_supported_card("Bring Down");
    ruling!(
        "Huddle Up",
        "Targets are chosen for that spell before you choose another player to help you pay for it and before that player has committed any mana to doing so."
    );
    let mut t = TestGame::new(2);
    // Bring Down: {3}{W} sorcery, assist, "Destroy target creature with power 4 or
    // greater."
    let mine = t.lands(P0, "Plains", 1);
    let theirs = t.lands(P1, "Forest", 3);
    let wurm = t.battlefield(P1, "Craw Wurm");
    let bd = t.hand(P0, "Bring Down");
    t.answer_choose(P0, &[Entity::Player(P1)]);
    t.answer(P1, DecisionKind::Number, Answer::Number(3));
    t.cast(P0, bd).target(Entity::Object(wurm)).go();
    assert_eq!(untapped(&t, &mine), 0);
    assert_eq!(untapped(&t, &theirs), 0);
    t.resolve_all();
    assert!(!t.on_battlefield(wurm));
    // The assisting player chose how much to pay after the targets were chosen.
    let asked: Vec<&str> = t
        .asked()
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseTargets { .. } => Some("targets"),
            Decision::ChooseEntities { prompt, .. } if prompt.contains("assist") => Some("assist"),
            Decision::ChooseNumber { prompt, .. } if prompt.starts_with("Assist") => Some("amount"),
            _ => None,
        })
        .collect();
    assert_eq!(asked, vec!["targets", "assist", "amount"]);
}

#[test]
fn the_assisting_player_may_pay_any_amount_of_the_generic_mana_only() {
    cr!("702.132a");
    ruling!(
        "Huddle Up",
        "Only the generic mana portion of a spell’s cost can be paid with assist."
    );
    let mut t = TestGame::new(2);
    let mine = t.lands(P0, "Plains", 3);
    let theirs = t.lands(P1, "Plains", 5);
    let wurm = t.battlefield(P1, "Craw Wurm");
    let bd = t.hand(P0, "Bring Down");
    t.answer_choose(P0, &[Entity::Player(P1)]);
    t.answer(P1, DecisionKind::Number, Answer::Number(1));
    t.cast(P0, bd).target(Entity::Object(wurm)).go();
    // P1 paid {1}; P0 paid {2}{W}.
    assert_eq!(untapped(&t, &mine), 0);
    assert_eq!(untapped(&t, &theirs), 4);
    // P1 could have paid at most the three generic mana.
    let max: Vec<i64> = t
        .asked()
        .iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseNumber { max, .. } if *p == P1 => Some(*max),
            _ => None,
        })
        .collect();
    assert_eq!(max, vec![3]);
}

#[test]
fn the_caster_may_choose_no_one() {
    cr!("702.132a");
    let mut t = TestGame::new(2);
    let mine = t.lands(P0, "Plains", 4);
    let theirs = t.lands(P1, "Plains", 3);
    let wurm = t.battlefield(P1, "Craw Wurm");
    let bd = t.hand(P0, "Bring Down");
    t.answer_choose(P0, &[]);
    t.cast(P0, bd).target(Entity::Object(wurm)).go();
    assert_eq!(untapped(&t, &mine), 0);
    assert_eq!(untapped(&t, &theirs), 3);
}

#[test]
fn assist_pays_part_of_x_and_cost_changes() {
    cr!("702.132a");
    ruling!(
        "Gang Up",
        "If an effect changes the cost of the spell, the amount that player may pay will be more or less than the amount in the spell’s reminder text."
    );
    assert_supported_card("Gang Up");
    let mut t = TestGame::new(2);
    // Gang Up: {X}{B} instant, assist, "Destroy target creature with power X or less."
    t.lands(P0, "Swamp", 1);
    t.lands(P1, "Swamp", 3);
    let giant = t.battlefield(P1, "Hill Giant");
    let gu = t.hand(P0, "Gang Up");
    t.answer(P0, DecisionKind::X, Answer::Number(3));
    t.answer_choose(P0, &[Entity::Player(P1)]);
    t.answer(P1, DecisionKind::Number, Answer::Number(3));
    t.cast(P0, gu).target(Entity::Object(giant)).go();
    t.resolve_all();
    assert!(!t.on_battlefield(giant));
}

#[test]
fn a_teammate_helps_by_default_and_counts_for_castability() {
    cr!("702.132a");
    assert_supported_card("Charging Binox");
    let mut t = TestGame::with_config(
        4,
        GameConfig {
            teams: Some(vec![0, 1, 0, 1]),
            ..Default::default()
        },
    );
    // Charging Binox: {7}{G} 7/7 trample, assist.
    t.lands(P0, "Forest", 1);
    let theirs = t.lands(P2, "Forest", 7);
    let binox = t.hand(P0, "Charging Binox");
    assert!(castable(&mut t, P0, binox, CastMethod::Normal));
    t.cast(P0, binox).go();
    assert_eq!(untapped(&t, &theirs), 0);
    t.resolve_all();
    assert!(t.on_battlefield(binox));
    assert_eq!(t.obj_now(binox).controller, P0);
    // Without a teammate's mana, it couldn't be cast.
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 1);
    t.lands(P1, "Forest", 7);
    let binox = t.hand(P0, "Charging Binox");
    assert!(!castable(&mut t, P0, binox, CastMethod::Normal));
}

#[test]
fn huddle_up_with_help() {
    cr!("702.132a");
    assert_supported_card("Huddle Up");
    ruling!(
        "Huddle Up",
        "You can’t target the same player twice to have them draw two cards."
    );
    let mut t = TestGame::new(2);
    // Huddle Up: {2}{U} sorcery, assist, "Two target players each draw a card."
    let mine = t.lands(P0, "Island", 1);
    let theirs = t.lands(P1, "Forest", 2);
    let hu = t.hand(P0, "Huddle Up");
    t.answer_choose(P0, &[Entity::Player(P1)]);
    t.answer(P1, DecisionKind::Number, Answer::Number(2));
    t.cast(P0, hu)
        .targets(&[Entity::Player(P0), Entity::Player(P1)])
        .go();
    assert_eq!(untapped(&t, &mine), 0);
    assert_eq!(untapped(&t, &theirs), 0);
    let (h0, h1) = (t.hand_size(P0), t.hand_size(P1));
    t.resolve_all();
    assert_eq!(t.hand_size(P0), h0 + 1);
    assert_eq!(t.hand_size(P1), h1 + 1);
    // The same player can't be both targets: an answer naming P0 twice isn't taken.
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 3);
    let hu = t.hand(P0, "Huddle Up");
    let spell = t
        .cast(P0, hu)
        .targets(&[Entity::Player(P0), Entity::Player(P0)])
        .go();
    let chosen: Vec<Entity> = t.g.obj(spell).stack.as_deref().unwrap().chosen[0]
        .targets
        .iter()
        .flatten()
        .copied()
        .collect();
    assert_eq!(chosen.len(), 2);
    assert_ne!(chosen[0], chosen[1]);
}
