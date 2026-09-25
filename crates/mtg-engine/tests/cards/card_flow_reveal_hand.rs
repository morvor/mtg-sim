//! "Target opponent reveals their hand. You choose a nonland card from it. That player
//! discards that card." — the caster chooses the discarded card (CR 701.9b).

use mtg_engine::decision::Decision;
use mtg_engine::testing::*;
use mtg_engine::*;

fn assert_supported(name: &str) {
    let c = card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        c.unsupported_text()
    );
}

/// The candidates of the most recent "choose entities" decision asked of `p`.
fn last_choice_of(t: &TestGame, p: PlayerId) -> Vec<Entity> {
    t.asked()
        .into_iter()
        .rev()
        .find_map(|(q, d)| match d {
            Decision::ChooseEntities { candidates, .. } if q == p => Some(candidates),
            _ => None,
        })
        .expect("no choice was asked")
}

#[test]
fn duress_caster_chooses_a_noncreature_nonland_card() {
    cr!("701.9b");
    assert_supported("Duress");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    let shock = t.hand(P1, "Shock");
    let bears = t.hand(P1, "Grizzly Bears");
    let forest = t.hand(P1, "Forest");
    let duress = t.hand(P0, "Duress");
    t.answer_choose(P0, &[Entity::Object(shock)]);
    t.cast(P0, duress).target(P1).go();
    t.resolve();
    // P0 chose among the noncreature, nonland cards.
    let mut offered = last_choice_of(&t, P0);
    offered.sort();
    let mut expected = vec![Entity::Object(bolt), Entity::Object(shock)];
    expected.sort();
    assert_eq!(offered, expected);
    assert!(t.in_graveyard(P1, "Shock"));
    assert!(t.in_hand(P1, "Lightning Bolt"));
    assert_eq!(t.zone(bears), mtg_engine::object::Zone::Hand(P1));
    assert_eq!(t.zone(forest), mtg_engine::object::Zone::Hand(P1));
}

#[test]
fn thoughtseize_loses_life_even_with_nothing_to_discard() {
    cr!("701.9b");
    ruling!(
        "Thoughtseize",
        "You lose 2 life even if the target player has no nonland cards in their hand to discard."
    );
    assert_supported("Thoughtseize");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 1);
    t.hand(P1, "Forest");
    let seize = t.hand(P0, "Thoughtseize");
    t.cast(P0, seize).target(P1).go();
    t.resolve();
    assert!(t.in_hand(P1, "Forest"));
    assert_eq!(t.life(P0), 18);
    // With a nonland card, it's discarded.
    let bears = t.hand(P1, "Grizzly Bears");
    let seize = t.hand(P0, "Thoughtseize");
    t.lands(P0, "Swamp", 1);
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.cast(P0, seize).target(P1).go();
    t.resolve();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(t.life(P0), 16);
}

#[test]
fn inquisition_of_kozilek_mana_value_three_or_less() {
    cr!("701.9b");
    assert_supported("Inquisition of Kozilek");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 1);
    let bears = t.hand(P1, "Grizzly Bears");
    t.hand(P1, "Shivan Dragon");
    let inq = t.hand(P0, "Inquisition of Kozilek");
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.cast(P0, inq).target(P1).go();
    t.resolve();
    assert_eq!(last_choice_of(&t, P0), vec![Entity::Object(bears)]);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(t.in_hand(P1, "Shivan Dragon"));
}

#[test]
fn castigate_exiles_the_chosen_card() {
    cr!("701.9b");
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Swamp", 1);
    let dragon = t.hand(P1, "Shivan Dragon");
    let cast = t.hand(P0, "Castigate");
    t.answer_choose(P0, &[Entity::Object(dragon)]);
    t.cast(P0, cast).target(P1).go();
    t.resolve();
    assert!(t.in_exile("Shivan Dragon"));
    assert!(!t.in_graveyard(P1, "Shivan Dragon"));
}
