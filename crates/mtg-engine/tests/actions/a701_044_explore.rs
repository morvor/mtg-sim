//! CR 701.44: explore.

use crate::a701_028_071_common::*;
use mtg_engine::ability::*;
use mtg_engine::events::MoveCause;
use mtg_engine::kwa::explore::{EXPLORED, REVEALED_LAND, REVEALED_NONLAND, REVEALED_NOTHING};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// River Herald Scout: "When this creature enters, it explores."
fn scout_enters(t: &mut TestGame) -> ObjectId {
    supported("River Herald Scout");
    let scout = t.enter(P0, "River Herald Scout");
    t.settle();
    scout
}

#[test]
fn a_land_goes_to_hand_otherwise_a_counter_and_the_card_may_go_to_the_graveyard() {
    cr!("701.44a");
    ruling!(
        "Jadelight Ranger",
        "If it's a land card, they'll put it into their hand. Otherwise, they'll put a +1/+1 counter on that creature, then choose to either leave that card on top of their library or put it into their graveyard."
    );
    // A land card: into the hand, no counter.
    let mut t = TestGame::new(2);
    let forest = t.library_top(P0, "Forest");
    let scout = scout_enters(&mut t);
    t.resolve_all();
    assert_eq!(t.zone(forest), Zone::Hand(P0));
    assert_eq!(t.counters(scout, "+1/+1"), 0);
    // A nonland card: a counter, and the player chooses to put it into the graveyard.
    let mut t = TestGame::new(2);
    let bears = t.library_top(P0, "Grizzly Bears");
    let scout = scout_enters(&mut t);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.counters(scout, "+1/+1"), 1);
    assert_eq!(t.pt(scout), (2, 3));
    assert_eq!(t.zone(bears), Zone::Graveyard(P0));
    // ... or to leave it on top of the library.
    let mut t = TestGame::new(2);
    let bears = t.library_top(P0, "Grizzly Bears");
    let scout = scout_enters(&mut t);
    t.answer_yes(P0, false);
    t.resolve_all();
    assert_eq!(t.counters(scout, "+1/+1"), 1);
    assert_eq!(t.g.library_top(P0), Some(bears));
    assert!(!t.in_hand(P0, "Grizzly Bears"));
}

#[test]
fn exploring_again_reveals_the_card_left_on_top() {
    cr!("701.44a");
    ruling!(
        "Jadelight Ranger",
        "If you reveal a nonland card the first time Jadelight Ranger explores and leave it on top of your library, you'll reveal the same card the second time it explores."
    );
    supported("Jadelight Ranger");
    let mut t = TestGame::new(2);
    let bears = t.library_top(P0, "Grizzly Bears");
    let ranger = t.enter(P0, "Jadelight Ranger");
    t.answer_yes(P0, false);
    t.answer_yes(P0, false);
    t.resolve_all();
    assert_eq!(t.counters(ranger, "+1/+1"), 2);
    assert_eq!(t.g.library_top(P0), Some(bears));
    assert_eq!(custom_events(&t, EXPLORED).len(), 2);
}

#[test]
fn a_permanent_explores_even_if_nothing_is_revealed() {
    cr!("701.44a", "701.44b");
    ruling!(
        "Jadelight Ranger",
        "If no card is revealed, most likely because that player's library is empty, the exploring creature receives a +1/+1 counter."
    );
    supported("Wildgrowth Walker");
    let mut t = TestGame::new(2);
    let walker = t.battlefield(P0, "Wildgrowth Walker");
    t.g.players[0].library.clear();
    let scout = scout_enters(&mut t);
    t.resolve_all();
    // It explored: "whenever a creature you control explores" triggered.
    assert_eq!(
        custom_events(&t, EXPLORED),
        vec![(Some(P0), Some(scout), REVEALED_NOTHING)]
    );
    assert_eq!(t.counters(scout, "+1/+1"), 1);
    assert_eq!(t.counters(walker, "+1/+1"), 1);
    assert_eq!(t.life(P0), 23);
}

#[test]
fn a_permanent_that_left_the_battlefield_still_explores() {
    cr!("701.44c");
    ruling!(
        "Lurking Chupacabra",
        "If a resolving spell or ability instructs a specific creature to explore but that creature has left the battlefield, the creature still explores."
    );
    supported("Lurking Chupacabra");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Lurking Chupacabra");
    let target = t.battlefield(P1, "Hill Giant");
    let bears = t.library_top(P0, "Grizzly Bears");
    let scout = scout_enters(&mut t);
    assert_eq!(t.stack_len(), 1);
    // The Scout leaves before its ability resolves.
    t.g.move_object(scout, Zone::Hand(P0), MoveCause::Effect, None);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(target)]);
    t.resolve_all();
    // Its last known information says who controlled it: that player revealed, and may
    // put the card into their graveyard, but there's nothing to put a counter on.
    assert_eq!(t.zone(bears), Zone::Graveyard(P0));
    assert_eq!(
        custom_events(&t, EXPLORED),
        vec![(Some(P0), Some(scout), REVEALED_NONLAND)]
    );
    // "Whenever a creature you control explores" saw it.
    assert_eq!(t.pt(target), (1, 1));
}

#[test]
fn several_permanents_explore_one_at_a_time_in_apnap_order() {
    cr!("701.44d");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let forest = t.library_top(P0, "Forest");
    let island = t.library_top(P1, "Island");
    let _ = (forest, island);
    t.set_step(P1, Step::PrecombatMain);
    // P1 is the active player: they choose among theirs first; then P0 chooses which of
    // theirs explores first (the Hill Giant).
    choose(&mut t, P0, &[b]);
    run(
        &mut t,
        P0,
        None,
        ka(
            KeywordAction::Explore,
            Sel::All(Filter::creature()),
            1,
        ),
        &[],
    );
    let order: Vec<Option<ObjectId>> = custom_events(&t, EXPLORED)
        .into_iter()
        .map(|(_, o, _)| o)
        .collect();
    assert_eq!(order, vec![Some(theirs), Some(b), Some(a)]);
    // Each explored with its own controller's library.
    assert!(t.in_hand(P1, "Island"));
    assert!(t.in_hand(P0, "Forest"));
    let revealed: Vec<i32> = custom_events(&t, EXPLORED).into_iter().map(|e| e.2).collect();
    assert_eq!(revealed[0], REVEALED_LAND);
    assert_eq!(revealed[1], REVEALED_LAND);
}

#[test]
fn explores_x_times() {
    cr!("701.44a", "701.44b");
    supported("Jadelight Spelunker");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 3);
    let card = t.hand(P0, "Jadelight Spelunker");
    let spell = t.cast(P0, card).x(2).go();
    t.answer_yes(P0, false);
    t.answer_yes(P0, false);
    t.resolve_all();
    let perm = t.g.current(spell);
    assert_eq!(t.counters(perm, "+1/+1"), 2);
    assert_eq!(custom_events(&t, EXPLORED).len(), 2);
}
