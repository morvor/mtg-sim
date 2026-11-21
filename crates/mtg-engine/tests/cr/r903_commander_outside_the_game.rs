//! CR 113.6c and 903.3: an ability that states the zone it doesn't function in works
//! everywhere else, even outside the game and before the game begins — so Grist, the
//! Hunger Tide ("As long as Grist isn't on the battlefield, it's a 1/1 Insect creature in
//! addition to its other types") is a legendary creature card while its deck is built,
//! and can be a commander.

use mtg_engine::card::{card, CardDef};
use mtg_engine::deck::{check_commander, DeckProblem};
use mtg_engine::events::MoveCause;
use mtg_engine::kw::partner::{can_be_commander, characteristics_outside_the_game};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;
use std::sync::Arc;

const GRIST: &str = "Grist, the Hunger Tide";

fn is_insect_creature(t: &TestGame, id: ObjectId) -> bool {
    let c = &t.obj_now(id).chars;
    c.is(CardType::Creature)
        && c.is(CardType::Planeswalker)
        && c.has_subtype("Insect")
        && c.has_subtype("Grist")
        && c.power == Some(1)
        && c.toughness == Some(1)
}

#[test]
fn grist_is_a_creature_everywhere_but_the_battlefield() {
    cr!("113.6c");
    ruling!(
        "Grist, the Hunger Tide",
        "Anywhere but on the battlefield, Grist is a Legendary Planeswalker Creature"
    );
    let d = card(GRIST);
    assert!(d.is_fully_supported(), "{:?}", d.unsupported_text());
    let mut t = TestGame::new(2);
    let hand = t.hand(P0, GRIST);
    let gy = t.graveyard(P0, GRIST);
    let ex = t.exile(P0, GRIST);
    let lib = t.library_top(P0, GRIST);
    t.g.recompute();
    for id in [hand, gy, ex, lib] {
        assert!(is_insect_creature(&t, id), "{:?}", t.zone(id));
    }
    // On the battlefield it's just a planeswalker.
    let bf = t.battlefield(P0, GRIST);
    t.g.recompute();
    let c = &t.obj_now(bf).chars;
    assert!(c.is(CardType::Planeswalker));
    assert!(!c.is(CardType::Creature));
    assert!(!c.has_subtype("Insect"));
    // On the stack it's a creature spell: Essence Scatter can counter it.
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Forest", 2);
    let g = t.hand(P0, GRIST);
    let spell = t.cast(P0, g).go();
    assert!(is_insect_creature(&t, spell));
    t.lands(P1, "Island", 2);
    let scatter = t.hand(P1, "Essence Scatter");
    t.cast(P1, scatter).target(spell).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, GRIST));
    assert!(t.named_on_battlefield(GRIST).is_empty());
}

#[test]
fn grist_in_the_graveyard_counts_as_a_creature_card() {
    cr!("113.6c");
    ruling!(
        "Grist, the Hunger Tide",
        "If Grist is in your graveyard at this time, it'll be a creature card and will contribute to the count."
    );
    let mut t = TestGame::new(2);
    let grist = t.battlefield(P0, GRIST);
    t.g.add_counters(Entity::Object(grist), "loyalty", 5, None);
    t.graveyard(P0, "Grizzly Bears");
    // −5: Each opponent loses life equal to the number of creature cards in your graveyard.
    t.activate(P0, grist, 2, &[]).unwrap();
    // In response, Grist leaves the battlefield for the graveyard.
    let g = t.g.current(grist);
    t.g.move_object(g, Zone::Graveyard(P0), MoveCause::Effect, None);
    t.resolve_all();
    assert!(t.in_graveyard(P0, GRIST));
    assert_eq!(t.life(P1), 18);
}

#[test]
fn grist_can_be_a_commander() {
    cr!("903.3", "113.6c");
    ruling!(
        "Grist, the Hunger Tide",
        "can be your commander as its first ability works before the game begins during deck construction"
    );
    let grist = card(GRIST);
    let c = characteristics_outside_the_game(&grist);
    assert!(c.is(CardType::Creature) && c.is(CardType::Planeswalker));
    assert_eq!((c.power, c.toughness), (Some(1), Some(1)));
    assert!(can_be_commander(&grist, false));
    // A legendary planeswalker without such an ability can't (outside Brawl).
    assert!(!can_be_commander(&card("Jace Beleren"), false));
    let mut deck: Vec<Arc<CardDef>> = vec![grist.clone()];
    deck.extend((0..99).map(|_| card("Swamp")));
    let problems = check_commander(&deck, &grist, &[], false);
    assert!(
        !problems
            .iter()
            .any(|p| matches!(p, DeckProblem::InvalidCommanders { .. })),
        "{problems:?}"
    );
    // Its printed characteristics alone wouldn't allow it.
    assert!(!grist.front().chars.is(CardType::Creature));
}
