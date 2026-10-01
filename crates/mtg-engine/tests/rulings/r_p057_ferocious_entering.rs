//! Rulings batch P057 — "whenever a creature you control with power 4 or greater enters"
//! (CR 603.6a, 603.10): the entering creature's power is taken as it enters, with static
//! abilities, entering counters and copy effects applied (CR 614.1c-d, 613); spells and
//! abilities can't raise or lower it in time. Once triggered, later changes don't matter.

use crate::r_p057_common::*;
use crate::r_s01_common::{supported, triggers_on_stack};
use crate::r_s02_common::destroy;
use crate::r_s05_common::enter;
use mtg_engine::testing::*;
use mtg_engine::*;

const BIG: &str = "Rumbling Baloth"; // vanilla 4/4
const GIANT: &str = "Hill Giant"; // vanilla 3/3

#[test]
fn kronch_wrangler_and_territorial_boar_trigger_on_themselves_with_static_power() {
    cr!("603.6a", "613.4c");
    ruling!(
        "Kronch Wrangler",
        "If Kronch Wrangler’s power is raised to 4 or greater as it enters the battlefield, it will cause its own ability to trigger."
    );
    ruling!(
        "Territorial Boar",
        "If Territorial Boar’s power is raised to 4 or greater as it enters the battlefield, it will cause its own ability to trigger."
    );
    supported("Kronch Wrangler");
    supported("Territorial Boar");
    supported("Glorious Anthem");
    // Kronch Wrangler (2/1): "Whenever a creature you control with power 4 or greater
    // enters, put a +1/+1 counter on this creature." Two Glorious Anthems ("Creatures you
    // control get +1/+1") make it 4/3 as it enters.
    let mut t = TestGame::new(2);
    t.lands(P0, "Glorious Anthem", 2);
    let k = enter(&mut t, P0, "Kronch Wrangler");
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.counters(k, "+1/+1"), 1);
    // Territorial Boar (2/2): "Whenever a creature you control with power 4 or greater
    // enters, this creature gets +1/+1 and gains vigilance until end of turn."
    let mut t = TestGame::new(2);
    t.lands(P0, "Glorious Anthem", 2);
    let b = enter(&mut t, P0, "Territorial Boar");
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.pt(b), (5, 5));
    // With only one Anthem, neither triggers on itself.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Glorious Anthem");
    enter(&mut t, P0, "Kronch Wrangler");
    enter(&mut t, P0, "Territorial Boar");
    assert_eq!(t.stack_len(), 0);
}

/// `name` (Kronch Wrangler or Territorial Boar) is on the battlefield; whether a creature
/// entering triggers it.
fn entering_triggers(name: &str) {
    // A 3/3 entering with Glorious Anthem (+1/+1) is 4/4 as it enters: it triggers.
    let mut t = TestGame::new(2);
    t.battlefield(P0, name);
    t.battlefield(P0, "Glorious Anthem");
    enter(&mut t, P0, GIANT);
    assert_eq!(t.stack_len(), 1, "{name}: static ability counted");
    // A 3/3 entering, then raised by an effect right away: no trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P0, name);
    let g = enter(&mut t, P0, GIANT);
    pump(&mut t, g, 1, 0);
    assert_eq!(t.stack_len(), 0, "{name}: raised too late");
    // A 4/4 enters, then shrinks: still triggered, and the trigger still does its thing.
    let mut t = TestGame::new(2);
    let me = t.battlefield(P0, name);
    let b = enter(&mut t, P0, BIG);
    assert_eq!(t.stack_len(), 1);
    pump(&mut t, b, -3, 0);
    t.resolve_all();
    if name == "Kronch Wrangler" {
        assert_eq!(t.counters(me, "+1/+1"), 1);
    } else {
        assert_eq!(t.pt(me), (3, 3));
        assert!(crate::r_s06_common::has_kw(
            &t,
            me,
            mtg_engine::keywords::KeywordKind::Vigilance
        ));
    }
}

#[test]
fn kronch_wrangler_checks_the_power_as_the_creature_enters() {
    cr!("603.6a", "603.10a", "613.4c");
    ruling!(
        "Kronch Wrangler",
        "The entering creature must have power 4 or greater as it enters the battlefield, or Kronch Wrangler’s ability won’t trigger."
    );
    ruling!(
        "Kronch Wrangler",
        "If the entering creature’s power changes to 3 or less after it has entered the battlefield, you’ll still put a +1/+1 counter on Kronch Wrangler."
    );
    supported("Kronch Wrangler");
    entering_triggers("Kronch Wrangler");
}

#[test]
fn territorial_boar_checks_the_power_as_the_creature_enters() {
    cr!("603.6a", "603.10a", "613.4c");
    ruling!(
        "Territorial Boar",
        "The entering creature must have power 4 or greater as it enters the battlefield, or Territorial Boar’s ability won’t trigger."
    );
    ruling!(
        "Territorial Boar",
        "If the entering creature’s power changes to 3 or less after it has entered the battlefield, Territorial Boar still gets +1/+1 and gains vigilance."
    );
    supported("Territorial Boar");
    entering_triggers("Territorial Boar");
}

#[test]
fn temur_ascendancy_considers_static_abilities_only_and_draws_after_lowering() {
    cr!("603.6a", "603.10a", "613.4c");
    ruling!(
        "Temur Ascendancy",
        "If a creature is entering the battlefield under your control, consider static abilities to determine whether its power is 4 or greater."
    );
    ruling!(
        "Temur Ascendancy",
        "Once the last ability of Temur Ascendancy has triggered, lowering the power of the creature won't stop you from drawing a card."
    );
    supported("Temur Ascendancy");
    // "Whenever a creature you control with power 4 or greater enters, you may draw a card."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Temur Ascendancy");
    t.battlefield(P0, "Glorious Anthem");
    enter(&mut t, P0, GIANT);
    assert_eq!(triggers_on_stack(&t, "draw a card"), 1);
    t.resolve_all();
    // Raising it with an effect after it entered is too late.
    let g = enter(&mut t, P0, "Grizzly Bears");
    pump(&mut t, g, 2, 0);
    assert_eq!(t.stack_len(), 0);
    // A 4/4 enters, then its power is lowered: still draws.
    let b = enter(&mut t, P0, BIG);
    assert_eq!(t.stack_len(), 1);
    pump(&mut t, b, -4, 0);
    t.answer_yes(P0, true);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn garruks_uprising_considers_statics_counters_and_copies_and_draws_regardless() {
    cr!("603.6a", "614.1c", "614.1d", "706.2");
    ruling!(
        "Garruk's Uprising",
        "If one or more static abilities that apply to a creature entering change its power, those abilities are considered"
    );
    ruling!(
        "Garruk's Uprising",
        "Once the last ability of Garruk's Uprising has triggered, lowering the power of the creature or removing it from the battlefield won't stop you from drawing a card."
    );
    supported("Garruk's Uprising");
    supported("Spike Colony");
    supported("Clone");
    // "Whenever a creature you control with power 4 or greater enters, draw a card."
    let draws = "Whenever a creature you control";
    // A static ability: Hill Giant (3/3) with Glorious Anthem.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Garruk's Uprising");
    t.battlefield(P0, "Glorious Anthem");
    enter(&mut t, P0, GIANT);
    assert_eq!(triggers_on_stack(&t, draws), 1);
    // Entering with counters: Spike Colony (0/0, "enters with four +1/+1 counters").
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Garruk's Uprising");
    enter(&mut t, P0, "Spike Colony");
    assert_eq!(triggers_on_stack(&t, draws), 1);
    // Entering as a copy: Clone copying Rumbling Baloth.
    let mut t = TestGame::new(2);
    let baloth = t.battlefield(P1, BIG);
    t.battlefield(P0, "Garruk's Uprising");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(baloth)]);
    enter(&mut t, P0, "Clone");
    assert_eq!(triggers_on_stack(&t, draws), 1);
    // Triggered; the creature is then destroyed: you still draw.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Garruk's Uprising");
    let b = enter(&mut t, P0, BIG);
    assert_eq!(triggers_on_stack(&t, draws), 1);
    destroy(&mut t, b);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}
