//! cards4-referents-statics, family `batch-referents` (and Twilight Diviner's group
//! condition): the objects of a "one or more" trigger (CR 603.2c) as a group a choice is
//! made from on resolution.

use mtg_engine::testing::*;
use mtg_engine::events::MoveCause;
use mtg_engine::object::Zone;
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
fn rinoa_returns_one_of_the_attackers_that_died_tapped_with_a_flying_counter() {
    cr!("603.2c", "603.10a", "400.7", "122.1b");
    supported("Rinoa, Angel Wing");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rinoa, Angel Wing");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_yes(P0, true);
    t.attack(&[(bears, Entity::Player(P1))], &[(giant, bears)]);
    t.resolve_all();
    let back = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(back.len(), 1, "{}", t.dump_log());
    let b = back[0];
    assert!(t.g.obj(b).tapped);
    assert_eq!(t.counters(b, "flying"), 1);
    assert!(t.g.obj(b).has_keyword(mtg_engine::keywords::KeywordKind::Flying));
}

#[test]
fn rinoa_returns_a_creature_only_once_each_turn() {
    cr!("603.2h");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rinoa, Angel Wing");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_yes(P0, true);
    t.attack(&[(bears, Entity::Player(P1))], &[(giant, bears)]);
    t.resolve_all();
    let b = t.named_on_battlefield("Grizzly Bears")[0];
    // It dies again this turn (not attacking: doesn't trigger anyway); a second attacker
    // dying this turn isn't returned.
    t.g.destroy(b, None);
    t.settle();
    t.resolve_all();
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
}

#[test]
fn rinoa_doesnt_return_creatures_that_werent_attacking() {
    cr!("603.2c");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rinoa, Angel Wing");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.g.destroy(bears, None);
    t.settle();
    t.resolve_all();
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
}

fn diviner_sees(from_graveyard: bool) -> usize {
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Twilight Diviner");
    let bears = if from_graveyard {
        t.graveyard(P0, "Grizzly Bears")
    } else {
        t.hand(P0, "Grizzly Bears")
    };
    // Put onto the battlefield by an effect.
    t.g.move_object(bears, Zone::Battlefield, MoveCause::Effect, Some(P0));
    t.settle();
    t.resolve_all();
    t.named_on_battlefield("Grizzly Bears").len()
}

#[test]
fn twilight_diviner_copies_a_creature_that_entered_from_a_graveyard() {
    cr!("603.2c", "603.4", "707.2");
    interp!("batch-they-condition-all");
    supported("Twilight Diviner");
    assert_eq!(diviner_sees(true), 2);
    // From the hand: no copy.
    assert_eq!(diviner_sees(false), 1);
}
