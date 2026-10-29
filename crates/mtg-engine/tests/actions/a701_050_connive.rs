//! CR 701.50: connive.

use crate::a701_028_071_common::*;
use mtg_engine::ability::*;
use mtg_engine::events::MoveCause;
use mtg_engine::kwa::connive::CONNIVED;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Raffine's Informant: "When this creature enters, it connives."
fn informant_enters(t: &mut TestGame) -> ObjectId {
    supported("Raffine's Informant");
    let id = t.enter(P0, "Raffine's Informant");
    t.settle();
    id
}

#[test]
fn draw_then_discard_and_a_counter_for_a_nonland_card() {
    cr!("701.50a");
    // Discarding a nonland card: a +1/+1 counter.
    let mut t = TestGame::new(2);
    let top = t.library_top(P0, "Forest");
    let bolt = t.hand(P0, "Lightning Bolt");
    let informant = informant_enters(&mut t);
    choose(&mut t, P0, &[bolt]);
    t.resolve_all();
    assert_eq!(t.zone(top), Zone::Hand(P0));
    assert!(t.in_graveyard(P0, "Lightning Bolt"));
    assert_eq!(t.counters(informant, "+1/+1"), 1);
    assert_eq!(t.hand_size(P0), 1);
    // Discarding a land card: no counter. (The card discarded by default is the first
    // one in hand.)
    let mut t = TestGame::new(2);
    t.library_top(P0, "Lightning Bolt");
    t.hand(P0, "Forest");
    let informant = informant_enters(&mut t);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Forest"));
    assert!(t.in_hand(P0, "Lightning Bolt"));
    assert_eq!(t.counters(informant, "+1/+1"), 0);
    assert_eq!(custom_events(&t, CONNIVED), vec![(Some(P0), Some(informant), 0)]);
}

#[test]
fn a_permanent_that_left_the_battlefield_still_connives() {
    cr!("701.50b");
    ruling!(
        "Raffine's Informant",
        "If a resolving spell or ability instructs a specific creature to connive but that creature has left the battlefield, the creature still connives."
    );
    supported("Iron Monger, Sadistic Tycoon");
    let mut t = TestGame::new(2);
    let monger = t.battlefield(P0, "Iron Monger, Sadistic Tycoon");
    let bolt = t.hand(P0, "Lightning Bolt");
    let informant = informant_enters(&mut t);
    t.g.move_object(informant, Zone::Hand(P0), MoveCause::Effect, None);
    choose(&mut t, P0, &[bolt]);
    t.resolve_all();
    // Its last controller drew and discarded; there was nothing to put a counter on.
    assert!(t.in_graveyard(P0, "Lightning Bolt"));
    assert_eq!(
        custom_events(&t, CONNIVED),
        vec![(Some(P0), Some(informant), 1)]
    );
    // "Whenever a creature you control connives" (Iron Monger is a Villain).
    assert_eq!(t.counters(monger, "+1/+1"), 1);
}

#[test]
fn several_permanents_connive_one_at_a_time_in_apnap_order() {
    cr!("701.50c");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    // P0 is the active player: their creature connives first.
    run(
        &mut t,
        P0,
        None,
        ka(KeywordAction::Connive, Sel::All(Filter::creature()), 1),
        &[],
    );
    let order: Vec<Option<ObjectId>> = custom_events(&t, CONNIVED)
        .into_iter()
        .map(|(_, o, _)| o)
        .collect();
    assert_eq!(order, vec![Some(mine), Some(theirs)]);
    // Each creature's controller drew and discarded (the filler cards are nonland).
    assert_eq!(t.graveyard_size(P0), 1);
    assert_eq!(t.graveyard_size(P1), 1);
    assert_eq!(t.counters(mine, "+1/+1"), 1);
    assert_eq!(t.counters(theirs, "+1/+1"), 1);
    // With P1 active, P1's connives first.
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::PrecombatMain);
    run(
        &mut t,
        P0,
        None,
        ka(KeywordAction::Connive, Sel::All(Filter::creature()), 1),
        &[],
    );
    let order: Vec<Option<ObjectId>> = custom_events(&t, CONNIVED)
        .into_iter()
        .map(|(_, o, _)| o)
        .collect();
    assert_eq!(order, vec![Some(theirs), Some(mine)]);
}

#[test]
fn connive_n_draws_and_discards_n_with_a_counter_per_nonland_card() {
    cr!("701.50d");
    ruling!(
        "Raffine, Scheming Seer",
        "If a creature connives X, its controller will draw X cards, discard X cards, then then put a number of +1/+1 counters on the conniving permanent equal to the number of nonland cards discarded this way."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.hand(P0, "Mountain");
    t.library_top(P0, "Forest");
    t.library_top(P0, "Lightning Bolt");
    t.library_top(P0, "Shock");
    // Draws Shock, Lightning Bolt and Forest, then discards three cards: by default the
    // first three in hand (the Mountain, Shock, and Lightning Bolt).
    run(
        &mut t,
        P0,
        None,
        ka(KeywordAction::Connive, Sel::Target(0), 3),
        &[Entity::Object(bears)],
    );
    assert_eq!(t.hand_size(P0), 1);
    assert!(t.in_hand(P0, "Forest"));
    assert_eq!(t.graveyard_size(P0), 3);
    assert_eq!(t.counters(bears, "+1/+1"), 2);
}

#[test]
fn conniving_zero_is_no_connive_event() {
    cr!("701.50e");
    supported("Iron Monger, Sadistic Tycoon");
    let mut t = TestGame::new(2);
    let monger = t.battlefield(P0, "Iron Monger, Sadistic Tycoon");
    let hand = t.hand_size(P0);
    run(
        &mut t,
        P0,
        None,
        ka(KeywordAction::Connive, Sel::Target(0), 0),
        &[Entity::Object(monger)],
    );
    t.settle();
    assert_eq!(t.hand_size(P0), hand);
    assert!(custom_events(&t, CONNIVED).is_empty());
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.counters(monger, "+1/+1"), 0);
}

#[test]
fn a_permanent_connives_even_if_nothing_could_be_drawn_or_discarded() {
    cr!("701.50f");
    ruling!(
        "Raffine's Informant",
        "If no card is discarded, most likely because that player's hand is empty and an effect says they can't draw cards, the conniving creature does not receive a +1/+1 counter."
    );
    supported("Iron Monger, Sadistic Tycoon");
    let mut t = TestGame::new(2);
    let monger = t.battlefield(P0, "Iron Monger, Sadistic Tycoon");
    // An empty hand, and an effect says P0 can't draw cards.
    run(
        &mut t,
        P0,
        None,
        Effect::AddRestriction {
            restriction: Restriction::MaxDrawsPerTurn(PlayerFilter::You, 0),
            duration: Duration::EndOfTurn,
        },
        &[],
    );
    let informant = informant_enters(&mut t);
    t.resolve_all();
    // Nothing was drawn or discarded, so no counter; but it connived: Iron Monger's
    // ability triggered (it puts a counter on each Villain, itself included).
    assert_eq!(t.graveyard_size(P0), 0);
    assert_eq!(t.counters(informant, "+1/+1"), 0);
    assert_eq!(
        custom_events(&t, CONNIVED),
        vec![(Some(P0), Some(informant), 0)]
    );
    assert_eq!(t.counters(monger, "+1/+1"), 1);
}
