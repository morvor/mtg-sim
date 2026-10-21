//! Counters distributed among objects chosen as the spell or ability resolves (no targets,
//! CR 608.2d): "distribute X +1/+1 counters among any number of creatures you control"
//! (Blessing of Frost), "... equal to the number of +1/+1 counters on ~ ..." (Vastwood
//! Hydra), "... among any number of creatures and/or Vehicles target player controls"
//! (Invoke Justice). Each chosen object gets at least one counter.

use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::*;

fn supported(name: &str) {
    let c = card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        c.unsupported_text()
    );
}

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

#[test]
fn vastwood_hydra_distributes_its_counters_as_the_ability_resolves() {
    cr!("608.2d", "603.10a");
    ruling!(
        "Vastwood Hydra",
        "You choose how the counters will be distributed when the ability resolves."
    );
    ruling!(
        "Vastwood Hydra",
        "doesn't target any of the creatures. You can choose to put counters on a creature with protection from green"
    );
    supported("Vastwood Hydra");
    let mut t = TestGame::new(2);
    let hydra = t.battlefield(P0, "Vastwood Hydra");
    t.g.add_counters(obj(hydra), "+1/+1", 5, None);
    let a = t.battlefield(P0, "Grizzly Bears");
    // Protection from green: not targeted, so it can get counters.
    let b = t.battlefield(P0, "Kor Firewalker");
    let theirs = t.battlefield(P1, "Hill Giant");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(a), obj(b)]);
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![1, 4]));
    t.lands(P0, "Swamp", 3);
    let kill = t.hand(P0, "Murder");
    t.cast(P0, kill).target(hydra).go();
    t.resolve_all();
    assert!(!t.on_battlefield(hydra));
    assert_eq!(t.counters(a, "+1/+1"), 1);
    assert_eq!(t.counters(b, "+1/+1"), 4);
    assert_eq!(t.counters(theirs, "+1/+1"), 0);
}

#[test]
fn blessing_of_frost_distributes_one_counter_per_snow_mana_then_draws() {
    cr!("608.2d", "107.4h");
    ruling!(
        "Blessing of Frost",
        "you determine how the +1/+1 counters will be distributed as Blessing of Frost resolves"
    );
    ruling!(
        "Blessing of Frost",
        "You determine how many creatures you control with power 4 or greater as Blessing of Frost is resolving, after +1/+1 counters have been put on creatures."
    );
    supported("Blessing of Frost");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    t.lands(P0, "Snow-Covered Forest", 3);
    t.lands(P0, "Forest", 1);
    // Three snow mana: two counters on the Bears (4/4), one on the Giant (4/4).
    t.answer_choose(P0, &[obj(a), obj(b)]);
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![2, 1]));
    let hand = t.hand_size(P0);
    let s = t.hand(P0, "Blessing of Frost");
    t.cast(P0, s).go();
    t.resolve_all();
    assert_eq!(t.counters(a, "+1/+1"), 2);
    assert_eq!(t.counters(b, "+1/+1"), 1);
    assert_eq!(t.hand_size(P0), hand + 2);
}

#[test]
fn invoke_justice_distributes_among_creatures_the_target_player_controls() {
    cr!("608.2d", "115.1");
    ruling!(
        "Invoke Justice",
        "you choose which creatures and/or Vehicles to put counters on as it resolves"
    );
    supported("Invoke Justice");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Hill Giant");
    let card_in_gy = t.graveyard(P0, "Serra Angel");
    t.lands(P0, "Plains", 5);
    t.answer_choose(P0, &[obj(theirs)]);
    let s = t.hand(P0, "Invoke Justice");
    t.cast(P0, s)
        .target(card_in_gy)
        .target(Entity::Player(P1))
        .go();
    t.resolve_all();
    assert!(t.named_on_battlefield("Serra Angel").len() == 1);
    assert_eq!(t.counters(theirs, "+1/+1"), 4);
    assert_eq!(t.counters(mine, "+1/+1"), 0);
}
