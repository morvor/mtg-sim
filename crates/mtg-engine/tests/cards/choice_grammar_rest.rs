//! Choices among objects and what wasn't chosen: "An opponent chooses one of them. Put
//! that card into your graveyard and the rest into your hand.", "Return the other to the
//! battlefield", "Leave the chosen cards in your graveyard and put the rest into your
//! hand." (CR 608.2d), and the chosen permanents phasing out.

use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn compiles(name: &str) {
    let def = card(name);
    assert!(
        def.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        def.unsupported_text()
    );
}

fn e(id: ObjectId) -> Entity {
    Entity::Object(id)
}

#[test]
fn murmurs_from_beyond_the_chosen_card_to_the_graveyard_the_rest_to_hand() {
    cr!("608.2d", "701.20a");
    compiles("Murmurs from Beyond");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 3);
    let a = t.library_top(P0, "Grizzly Bears");
    let b = t.library_top(P0, "Forest");
    let c = t.library_top(P0, "Ornithopter");
    t.answer_choose(P1, &[e(b)]);
    let spell = t.hand(P0, "Murmurs from Beyond");
    t.cast(P0, spell).go();
    t.resolve_all();
    let _ = (a, c);
    assert!(t.in_graveyard(P0, "Forest"), "{}", t.dump_log());
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert!(t.in_hand(P0, "Ornithopter"));
}

#[test]
fn karn_scion_of_urza_put_that_card_into_your_hand_and_exile_the_other() {
    cr!("608.2d", "606.3");
    compiles("Karn, Scion of Urza");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let karn = t.battlefield(P0, "Karn, Scion of Urza");
    t.library_top(P0, "Grizzly Bears");
    let b = t.library_top(P0, "Forest");
    t.answer_choose(P1, &[e(b)]);
    t.activate(P0, karn, 0, &[]).expect("activates");
    t.resolve_all();
    assert!(t.in_hand(P0, "Forest"), "{}", t.dump_log());
    assert!(t.in_exile("Grizzly Bears"));
    let exiled = t.g.find_in_zone(mtg_engine::object::Zone::Exile, "Grizzly Bears")[0];
    assert_eq!(t.counters(exiled, "silver"), 1);
}

#[test]
fn wake_to_slaughter_one_to_hand_the_other_returns_with_haste() {
    cr!("608.2d", "702.10b");
    compiles("Wake to Slaughter");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 3);
    t.lands(P0, "Mountain", 2);
    let a = t.graveyard(P0, "Grizzly Bears");
    let b = t.graveyard(P0, "Hill Giant");
    t.answer_choose(P1, &[e(a)]);
    let spell = t.hand(P0, "Wake to Slaughter");
    t.cast(P0, spell).targets(&[e(a), e(b)]).go();
    t.resolve_all();
    assert!(t.in_hand(P0, "Grizzly Bears"), "{}", t.dump_log());
    let giant = t.named_on_battlefield("Hill Giant");
    assert_eq!(giant.len(), 1);
    let dbg = format!("{:?}", t.obj_now(giant[0]).chars.abilities);
    assert!(dbg.contains("Haste"), "it gains haste: {dbg}");
    // Exiled at the beginning of the next end step.
    t.advance_to_step(Step::End);
    t.resolve_all();
    assert!(t.in_exile("Hill Giant"));
}

#[test]
fn wake_to_slaughter_with_one_target_it_goes_to_your_hand() {
    cr!("608.2d");
    ruling!(
        "Wake to Slaughter",
        "If only one target is still legal as Wake to Slaughter resolves (or if you only chose one target), that's the card your opponent will have to choose to put into your hand."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 3);
    t.lands(P0, "Mountain", 2);
    let a = t.graveyard(P0, "Grizzly Bears");
    let spell = t.hand(P0, "Wake to Slaughter");
    t.cast(P0, spell).targets(&[e(a)]).go();
    t.resolve_all();
    assert!(t.in_hand(P0, "Grizzly Bears"), "{}", t.dump_log());
}

#[test]
fn deliver_unto_evil_an_opponent_chooses_two_to_leave() {
    cr!("608.2d", "608.2c");
    ruling!(
        "Deliver Unto Evil",
        "your opponent will choose two of however many cards you did target"
    );
    compiles("Deliver Unto Evil");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 3);
    let a = t.graveyard(P0, "Grizzly Bears");
    let b = t.graveyard(P0, "Hill Giant");
    let c = t.graveyard(P0, "Forest");
    let d = t.graveyard(P0, "Ornithopter");
    t.answer_choose(P1, &[e(a), e(c)]);
    let spell = t.hand(P0, "Deliver Unto Evil");
    t.cast(P0, spell).targets(&[e(a), e(b), e(c), e(d)]).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"), "{}", t.dump_log());
    assert!(t.in_graveyard(P0, "Forest"));
    assert!(t.in_hand(P0, "Hill Giant"));
    assert!(t.in_hand(P0, "Ornithopter"));
    assert!(t.in_exile("Deliver Unto Evil"));

    // With two targets, the opponent chooses both: none returns.
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 3);
    let a = t.graveyard(P0, "Grizzly Bears");
    let b = t.graveyard(P0, "Hill Giant");
    let spell = t.hand(P0, "Deliver Unto Evil");
    t.cast(P0, spell).targets(&[e(a), e(b)]).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert!(t.in_graveyard(P0, "Hill Giant"));
}

#[test]
fn temporal_firestorm_the_chosen_permanents_phase_out() {
    cr!("702.26a", "608.2d");
    compiles("Temporal Firestorm");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 5);
    t.lands(P0, "Plains", 2);
    let kept = t.battlefield(P0, "Grizzly Bears");
    let other = t.battlefield(P0, "Hill Giant");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let spell = t.hand(P0, "Temporal Firestorm");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(false));
    t.answer_choose(P0, &[e(kept)]);
    t.cast(P0, spell).go();
    t.resolve_all();
    assert!(t.obj_now(kept).phased_out, "{}", t.dump_log());
    assert!(!t.on_battlefield(other) || t.obj_now(other).damage >= 4);
    assert!(!t.on_battlefield(theirs));
}

#[test]
fn consuming_tide_keep_one_bounce_the_rest_draw_for_bigger_hands() {
    cr!("608.2d", "101.4");
    compiles("Consuming Tide");
    let mut t = TestGame::new(3);
    t.lands(P0, "Island", 4);
    let my_keep = t.battlefield(P0, "Grizzly Bears");
    let my_other = t.battlefield(P0, "Ornithopter");
    let p1_keep = t.battlefield(P1, "Hill Giant");
    let p1_other = t.battlefield(P1, "Llanowar Elves");
    let p2_only = t.battlefield(P2, "Grizzly Bears");
    t.hand(P2, "Forest");
    t.hand(P1, "Forest");
    t.hand(P1, "Forest");
    t.library_top(P0, "Island");
    t.answer_choose(P0, &[e(my_keep)]);
    t.answer_choose(P1, &[e(p1_keep)]);
    let spell = t.hand(P0, "Consuming Tide");
    t.cast(P0, spell).go();
    t.resolve_all();
    assert!(t.on_battlefield(my_keep) && t.on_battlefield(p1_keep) && t.on_battlefield(p2_only));
    assert!(t.in_hand(P0, "Ornithopter") && t.in_hand(P1, "Llanowar Elves"));
    let _ = (my_other, p1_other);
    // After the bounce: P0 has 1 card (Ornithopter), P1 has 3, P2 has 1: one opponent
    // has more cards in hand than P0, who draws one card.
    assert_eq!(t.hand_size(P0), 2, "{}", t.dump_log());
}
