//! CR 113.6e's second sentence: "An object’s ability that grants it another ability that
//! restricts or modifies how that particular object can be played or cast functions only on
//! the stack." A card whose own ability gives it flash while a condition holds ("This spell
//! has flash as long as you control a Merfolk.", Crashing Tide; "Colossal Rattlewurm has
//! flash as long as you control a Desert.") doesn't have flash in a hand, a library, a
//! graveyard or on the battlefield; CR 601.3d lets it be begun as though it had flash when
//! the condition holds. (Found in the review of work item ruling-conflicts-13: the engine
//! gave such cards flash wherever they were.)

use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn a_self_granted_flash_functions_only_on_the_stack() {
    cr!("113.6e", "601.3d");
    // Crashing Tide in hand with a Merfolk: no flash there, but it may be cast on the
    // opponent's turn, and on the stack it has flash.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Merfolk Looter");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let tide = t.hand(P0, "Crashing Tide");
    t.lands(P0, "Island", 3);
    t.set_step(P1, Step::PrecombatMain);
    t.g.turn.priority = Some(P0);
    t.g.recompute();
    assert!(!t.obj(tide).has_keyword(KeywordKind::Flash));
    t.cast(P0, tide).target(bears).go();
    assert!(t.obj(t.g.current(tide)).has_keyword(KeywordKind::Flash));
    t.resolve_all();
    assert!(t.in_hand(P1, "Grizzly Bears"));
    // Graveyard Shift in a graveyard with five mana values there: no flash in the graveyard.
    let mut t = TestGame::new(2);
    for n in [
        "Ornithopter",
        "Llanowar Elves",
        "Grizzly Bears",
        "Divination",
    ] {
        t.graveyard(P0, n);
    }
    let gs = t.graveyard(P0, "Graveyard Shift");
    t.g.recompute();
    assert!(!t.obj(gs).has_keyword(KeywordKind::Flash));
}

#[test]
fn a_search_for_a_card_with_flash_doesnt_find_a_self_granted_flash() {
    cr!("113.6e", "702.8a");
    // Mystical Teachings: "Search your library for an instant card or a card with flash,
    // reveal it, put it into your hand, then shuffle." With a Merfolk, Crashing Tide in the
    // library still has no flash: it can't be found; Ambush Viper (flash) can.
    for (name, found) in [("Crashing Tide", false), ("Ambush Viper", true)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Merfolk Looter");
        let card = t.library_top(P0, name);
        t.library_top(P0, "Grizzly Bears");
        t.lands(P0, "Island", 4);
        let teachings = t.hand(P0, "Mystical Teachings");
        t.answer_choose(P0, &[Entity::Object(card)]);
        t.cast(P0, teachings).go();
        t.resolve_all();
        assert_eq!(t.in_hand(P0, name), found, "{name}");
        if !found {
            assert_eq!(t.zone(t.g.current(card)), Zone::Library(P0), "{name}");
        }
    }
}

#[test]
fn a_permanent_with_a_self_granted_flash_isnt_a_permanent_with_flash() {
    cr!("113.6e", "601.3d");
    // Mutual Destruction: "This spell has flash as long as you control a permanent with
    // flash. / As an additional cost to cast this spell, sacrifice a creature. / Destroy
    // target creature." On P1's turn: Colossal Rattlewurm on the battlefield (with a
    // Desert) doesn't have flash, so Mutual Destruction can't be cast; Ambush Viper does.
    for (name, castable) in [("Colossal Rattlewurm", false), ("Ambush Viper", true)] {
        let mut t = TestGame::new(2);
        let permanent = t.battlefield(P0, name);
        t.battlefield(P0, "Desert");
        let fodder = t.battlefield(P0, "Grizzly Bears");
        let target = t.battlefield(P1, "Hill Giant");
        let md = t.hand(P0, "Mutual Destruction");
        t.lands(P0, "Swamp", 1);
        t.set_step(P1, Step::PrecombatMain);
        t.g.turn.priority = Some(P0);
        t.g.recompute();
        assert_eq!(
            t.obj(permanent).has_keyword(KeywordKind::Flash),
            castable,
            "{name}"
        );
        t.answer_choose(P0, &[Entity::Object(fodder)]);
        let cast = t.cast(P0, md).target(target).try_go();
        assert_eq!(cast.is_ok(), castable, "{name}: {cast:?}");
        if castable {
            t.resolve_all();
            assert!(t.in_graveyard(P1, "Hill Giant"));
        } else {
            assert!(t.on_battlefield(target));
        }
    }
}
