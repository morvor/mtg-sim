//! Per-player outcomes of an instruction several players performed (CR 101.4, 608.2c,
//! 608.2e): each player's own number of cards ("3 minus the number of cards they
//! discarded this way", "for each card less than two a player draws this way") and the
//! number of players who did it ("the number of opponents who investigated this way").

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

#[test]
fn mind_bomb_damage_is_three_minus_each_players_own_discards() {
    cr!("101.4", "608.2e");
    supported("Mind Bomb");
    let mut t = TestGame::new(3);
    t.lands(P0, "Island", 1);
    let spell = t.hand(P0, "Mind Bomb");
    // P0 discards one card, P1 discards three, P2 declines.
    let a = t.hand(P0, "Grizzly Bears");
    let b1 = t.hand(P1, "Grizzly Bears");
    let b2 = t.hand(P1, "Hill Giant");
    let b3 = t.hand(P1, "Lightning Bolt");
    t.hand(P2, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(a)]);
    t.answer_yes(P1, true);
    t.answer_choose(
        P1,
        &[Entity::Object(b1), Entity::Object(b2), Entity::Object(b3)],
    );
    t.answer_yes(P2, false);
    t.cast(P0, spell).go();
    t.resolve();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(t.graveyard_size(P1), 3);
    assert_eq!(t.hand_size(P2), 1);
    assert_eq!((t.life(P0), t.life(P1), t.life(P2)), (18, 20, 17));
}

#[test]
fn truce_each_player_gains_two_life_per_card_less_than_two_drawn() {
    cr!("101.4", "608.2e");
    supported("Truce");
    let mut t = TestGame::new(3);
    t.lands(P0, "Plains", 3);
    let spell = t.hand(P0, "Truce");
    // P0 draws one card, P1 draws two, P2 declines (draws none).
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::Number, Answer::Number(1));
    t.answer_yes(P1, true);
    t.answer(P1, DecisionKind::Number, Answer::Number(2));
    t.answer_yes(P2, false);
    let hands = (t.hand_size(P0), t.hand_size(P1), t.hand_size(P2));
    t.cast(P0, spell).go();
    t.resolve();
    assert_eq!(
        (t.hand_size(P0), t.hand_size(P1), t.hand_size(P2)),
        (hands.0, hands.1 + 2, hands.2)
    );
    assert_eq!((t.life(P0), t.life(P1), t.life(P2)), (22, 20, 24));
}

#[test]
fn wernog_investigates_one_plus_the_opponents_who_investigated() {
    cr!("118.12", "701.16a");
    supported("Wernog, Rider's Chaplain");
    let mut t = TestGame::new(4);
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Swamp", 1);
    let w = t.hand(P0, "Wernog, Rider's Chaplain");
    // Two of three opponents investigate; the third loses 1 life.
    t.answer_yes(P1, true);
    t.answer_yes(P2, false);
    t.answer_yes(P3, true);
    t.cast(P0, w).go();
    t.resolve_all();
    let clues = |t: &TestGame, p: PlayerId| {
        t.g.battlefield
            .iter()
            .filter(|c| {
                let o = t.obj_now(**c);
                o.chars.name.starts_with("Clue") && o.controller == p
            })
            .count()
    };
    assert_eq!(clues(&t, P0), 3);
    assert_eq!((clues(&t, P1), clues(&t, P2), clues(&t, P3)), (1, 0, 1));
    assert_eq!((t.life(P1), t.life(P2), t.life(P3)), (20, 19, 20));
}

#[test]
fn tangle_wire_player_taps_one_permanent_per_fade_counter() {
    cr!("701.26a");
    supported("Tangle Wire");
    let mut t = TestGame::new(2);
    let w = t.battlefield(P0, "Tangle Wire");
    t.g.add_counters(Entity::Object(w), "fade", 2, None);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let lands = t.lands(P1, "Forest", 2);
    let rock = t.battlefield(P1, "Mind Stone");
    t.answer_choose(P1, &[Entity::Object(bears), Entity::Object(rock)]);
    t.advance_to(P1, mtg_engine::turn::Step::Upkeep);
    t.resolve_all();
    assert!(t.obj_now(bears).tapped);
    assert!(t.obj_now(rock).tapped);
    assert!(lands.iter().all(|l| !t.obj_now(*l).tapped));
    assert!(t
        .asked()
        .iter()
        .any(|(p, d)| *p == P1
            && matches!(d, decision::Decision::ChooseEntities { min: 2, max: 2, .. })));
}

#[test]
fn malignant_growth_damage_equals_the_cards_drawn_this_way() {
    cr!("504.1");
    ruling!(
        "Malignant Growth",
        "The triggered ability is put onto the stack after you have already drawn your card for the turn."
    );
    supported("Malignant Growth");
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Malignant Growth");
    t.g.add_counters(Entity::Object(m), "growth", 2, None);
    t.advance_to(P1, mtg_engine::turn::Step::Draw);
    let hand = t.hand_size(P1);
    t.resolve_all();
    assert_eq!(t.hand_size(P1), hand + 2);
    assert_eq!(t.life(P1), 18);
}
