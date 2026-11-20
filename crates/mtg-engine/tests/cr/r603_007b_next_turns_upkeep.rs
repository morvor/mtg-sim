//! CR 603.7b: a delayed triggered ability triggers "the next time its trigger event
//! occurs". "At the beginning of the next upkeep" (Archangel Avacyn, Aang, at the
//! Crossroads) is the next upkeep step to begin, an additional one later in the same turn
//! included (CR 500.9, Paradox Haze); "at the beginning of the next turn's upkeep" (Heal,
//! Mishra's Bauble and the other cantrips) is the first upkeep of the turn after the one
//! the delayed ability was created in.


use mtg_engine::object::FaceState;
use mtg_engine::testing::*;
use mtg_engine::turn::{Stage, Step};
use mtg_engine::*;

/// P0's turn, in its first upkeep step, with an additional upkeep step to come after it
/// (Paradox Haze enchanting P0, its trigger resolved).
fn first_of_two_upkeeps(t: &mut TestGame) {
    let haze = t.battlefield(P0, "Paradox Haze");
    t.g.objects[haze.0 as usize].attached_to = Some(Entity::Player(P0));
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.g.turn.upkeeps, 1);
}

/// Advances to the beginning of the second upkeep step of this turn and resolves what
/// triggered.
fn to_second_upkeep(t: &mut TestGame) {
    let turn = t.g.turn.number;
    let ok = t.g.run_until(10_000, |g| {
        g.turn.number == turn
            && g.turn.step == Step::Upkeep
            && g.turn.upkeeps == 2
            && g.turn.stage == Stage::Priority
    });
    assert!(ok, "no second upkeep");
    t.resolve_all();
}

#[test]
fn the_next_upkeep_can_be_an_additional_upkeep_of_the_same_turn() {
    cr!("603.7b", "500.9");
    ruling!(
        "Paradox Haze",
        "Abilities that trigger \"at the beginning of [your] upkeep\" will trigger at the beginning of each additional upkeep as well."
    );
    ruling!(
        "Archangel Avacyn // Avacyn, the Purifier",
        "Archangel Avacyn's delayed triggered ability triggers at the beginning of the next upkeep regardless of whose turn it is."
    );
    let mut t = TestGame::new(2);
    let avacyn = t.battlefield(P0, "Archangel Avacyn // Avacyn, the Purifier");
    let bears = t.battlefield(P0, "Grizzly Bears");
    first_of_two_upkeeps(&mut t);
    // A non-Angel creature dies during the first upkeep: Avacyn transforms at the
    // beginning of the next upkeep, the second one of this turn.
    t.g.destroy(bears, None);
    t.resolve_all();
    assert_eq!(t.obj_now(avacyn).face, FaceState::Front);
    to_second_upkeep(&mut t);
    assert_eq!(t.g.turn.active, P0);
    assert_eq!(t.obj_now(avacyn).face, FaceState::Back);
}

#[test]
fn aang_transforms_at_an_additional_upkeep_of_the_same_turn() {
    cr!("603.7b", "500.9");
    ruling!(
        "Aang, at the Crossroads // Aang, Destined Savior",
        "Aang, at the Crossroads's delayed triggered ability triggers at the beginning of the next upkeep regardless of whose turn it is."
    );
    let mut t = TestGame::new(2);
    let aang = t.battlefield(P0, "Aang, at the Crossroads // Aang, Destined Savior");
    let bears = t.battlefield(P0, "Grizzly Bears");
    first_of_two_upkeeps(&mut t);
    t.g.destroy(bears, None);
    t.resolve_all();
    assert_eq!(t.obj_now(aang).chars.name, "Aang, at the Crossroads");
    to_second_upkeep(&mut t);
    assert_eq!(t.obj_now(aang).chars.name, "Aang, Destined Savior");
}

#[test]
fn the_next_turns_upkeep_is_the_first_upkeep_of_the_next_turn() {
    cr!("603.7b", "500.9");
    interp!("next-turns-upkeep-is-the-next-turns-first");
    ruling!(
        "Heal",
        "You draw a card at the beginning of the next turn’s upkeep after Heal resolves, regardless of whether 1 damage was actually prevented."
    );
    // Heal cast during the first of two upkeep steps: no card at the second one, a card
    // at the beginning of P1's turn's upkeep.
    let mut t = TestGame::new(2);
    first_of_two_upkeeps(&mut t);
    t.lands(P0, "Plains", 1);
    let heal = t.hand(P0, "Heal");
    t.cast(P0, heal).target(Entity::Player(P0)).go();
    t.resolve_all();
    let hand = t.hand_size(P0);
    to_second_upkeep(&mut t);
    assert_eq!(t.hand_size(P0), hand);
    // P0's draw step, then P1's turn.
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.hand_size(P0), hand + 1);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
}

#[test]
fn a_cantrip_cast_in_the_main_phase_draws_at_the_next_turns_upkeep() {
    cr!("603.7b");
    interp!("next-turns-upkeep-is-the-next-turns-first");
    ruling!(
        "Infuse",
        "you’ll still draw a card at the beginning of the next turn’s upkeep"
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Island", 3);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let infuse = t.hand(P0, "Infuse");
    t.cast(P0, infuse).target(bears).go();
    t.resolve_all();
    let hand = t.hand_size(P0);
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.hand_size(P0), hand);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    // Once: not at P0's next upkeep too.
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn the_next_turns_upkeep_skipped_means_no_card() {
    cr!("603.7b", "614.10");
    interp!("next-turns-upkeep-is-the-next-turns-first");
    // Eon Hub ("Players skip their upkeep steps."): the next turn has no upkeep, so Heal's
    // delayed ability never triggers, not even at a later turn's upkeep once the Hub is
    // gone.
    let mut t = TestGame::new(2);
    let hub = t.battlefield(P0, "Eon Hub");
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Plains", 1);
    let heal = t.hand(P0, "Heal");
    t.cast(P0, heal).target(Entity::Player(P0)).go();
    t.resolve_all();
    let hand = t.hand_size(P0);
    t.advance_to(P1, Step::Draw);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
    t.g.destroy(hub, None);
    t.resolve_all();
    // P0's next turn: an upkeep again, and P0's draw step.
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
    t.advance_to(P0, Step::Draw);
    assert_eq!(t.hand_size(P0), hand + 1);
}
