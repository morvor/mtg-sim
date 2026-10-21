//! Rulings batch P145 — phasing: Teferi, Master of Time's −3 and Kaito Shizuki phasing
//! out at the end of the turn he entered.

use crate::r_p145_common::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

const TEFERI: &str = "Teferi, Master of Time";

/// P0's Teferi phases out `target` (a permanent P0 doesn't control) with his −3.
fn teferi_phases_out(t: &mut TestGame, teferi: ObjectId, target: ObjectId) {
    t.answer_targets(P0, &[obj(target)]);
    activate_containing(t, P0, teferi, "phases out").expect("−3");
    t.resolve();
    assert!(t.obj_now(target).phased_out, "phased out");
}

#[test]
fn for_as_long_as_effects_end_when_the_tracked_permanent_phases_out() {
    cr!("702.26f", "702.26b", "611.2b");
    ruling!(
        "Teferi, Master of Time",
        "Any continuous effects with a \"for as long as\" duration such as that of Awakener Druid ignore phased-out objects."
    );
    supported(TEFERI);
    supported("Awakener Druid");
    let mut t = TestGame::new(2);
    let teferi = t.battlefield(P0, TEFERI);
    let forest = t.battlefield(P1, "Forest");
    t.answer_targets(P1, &[obj(forest)]);
    let druid = enter(&mut t, P1, "Awakener Druid");
    t.resolve_all();
    assert_eq!(t.pt(forest), (4, 5));
    teferi_phases_out(&mut t, teferi, druid);
    t.g.recompute();
    assert!(!t.obj_now(forest).is(mtg_engine::types::CardType::Creature));
    // The Druid phases back in during P1's untap step: the effect stays ended.
    t.advance_to(P1, Step::Upkeep);
    assert!(!t.obj_now(druid).phased_out);
    assert!(!t.obj_now(forest).is(mtg_engine::types::CardType::Creature));
}

#[test]
fn until_leaves_one_shot_effects_wait_through_phasing() {
    cr!("702.26d", "610.3");
    ruling!(
        "Teferi, Master of Time",
        "Any one-shot effects that are waiting \"until [this] leaves the battlefield,\" such as that of Kitesail Freebooter, won't happen when a permanent phases out."
    );
    supported("Kitesail Freebooter");
    let mut t = TestGame::new(2);
    let teferi = t.battlefield(P0, TEFERI);
    let ritual = t.hand(P0, "Dark Ritual");
    t.answer_targets(P1, &[Entity::Player(P0)]);
    t.answer_choose(P1, &[obj(ritual)]);
    let freebooter = enter(&mut t, P1, "Kitesail Freebooter");
    t.resolve_all();
    assert!(t.in_exile("Dark Ritual"));
    teferi_phases_out(&mut t, teferi, freebooter);
    assert!(t.in_exile("Dark Ritual"));
    t.advance_to(P1, Step::Upkeep);
    assert!(!t.obj_now(freebooter).phased_out);
    assert!(t.in_exile("Dark Ritual"));
    // Leaving the battlefield for real returns it.
    destroy(&mut t, freebooter);
    assert!(t.in_hand(P0, "Dark Ritual"));
}

#[test]
fn a_stolen_creature_phases_in_under_its_owners_control_in_the_thiefs_untap_step() {
    cr!("702.26a", "702.26d", "514.2");
    ruling!(
        "Teferi, Master of Time",
        "If an opponent gains control of your creature and it phases out and the duration of the control-change effect expires before it phases in, that permanent phases in under your control as that opponent's next untap step begins."
    );
    let mut t = TestGame::new(2);
    let teferi = t.battlefield(P0, TEFERI);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P1, Step::PrecombatMain);
    cast_new(&mut t, P1, "Act of Treason", &[obj(bears)]);
    t.resolve_all();
    assert_eq!(t.obj_now(bears).controller, P1);
    // P0 phases it out during P1's turn (Teferi's loyalty abilities at instant speed).
    teferi_phases_out(&mut t, teferi, bears);
    // P0's turn: it stays phased out.
    t.advance_to(P0, Step::Upkeep);
    assert!(t.obj_now(bears).phased_out);
    // P1's untap step: it phases in, under P0's control (Act of Treason has ended).
    t.advance_to(P1, Step::Upkeep);
    assert!(!t.obj_now(bears).phased_out);
    assert_eq!(t.obj_now(bears).controller, P0);
}

#[test]
fn teferi_still_activates_one_loyalty_ability_per_turn() {
    cr!("606.3", "606.6");
    ruling!(
        "Teferi, Master of Time",
        "Teferi's first ability doesn't change how many of his abilities you can activate during each turn."
    );
    let mut t = TestGame::new(2);
    let teferi = t.battlefield(P0, TEFERI);
    t.set_step(P1, Step::Upkeep);
    assert!(can_activate(&mut t, P0, teferi));
    activate_containing(&mut t, P0, teferi, "Draw a card").expect("+1");
    t.resolve_all();
    assert!(!can_activate(&mut t, P0, teferi));
    t.set_step(P1, Step::End);
    assert!(!can_activate(&mut t, P0, teferi));
    // A new turn: again one.
    t.advance_to(P0, Step::Upkeep);
    assert!(can_activate(&mut t, P0, teferi));
}

#[test]
fn kaito_phases_out_with_his_auras_and_back_in_unchanged() {
    cr!("702.26g", "702.26d", "502.1", "603.4");
    ruling!(
        "Kaito Shizuki",
        "As Kaito Shizuki phases out, Auras attached to him also phase out at the same time. Those Auras will phase in at the same time Kaito does, and they'll phase in still attached to Kaito."
    );
    ruling!(
        "Kaito Shizuki",
        "Permanents phase back in during their controller's untap step, immediately before that player untaps their permanents. Kaito phases back in with the same number of loyalty counters that he phased out with."
    );
    ruling!(
        "Kaito Shizuki",
        "Phasing out doesn't cause permanents to leave or re-enter the battlefield, so no \"leaves the battlefield\" or \"enters the battlefield\" abilities will trigger, and Kaito's first ability won't trigger again at the end of a turn in which he phased in."
    );
    supported("Kaito Shizuki");
    let mut t = TestGame::new(2);
    let kaito = enter(&mut t, P0, "Kaito Shizuki");
    t.resolve_all();
    activate_containing(&mut t, P0, kaito, "Draw a card").expect("+1");
    t.resolve_all();
    let loyalty = t.counters(kaito, counters::LOYALTY);
    lands_for_cost(&mut t, P0, "Indestructibility");
    let aura = t.hand(P0, "Indestructibility");
    t.cast_with(P0, aura, &[obj(kaito)]).expect("cast");
    t.resolve_all();
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.obj_now(kaito).phased_out);
    assert!(t.obj_now(aura).phased_out);
    // P0's next untap step: both phase in, still attached, with the same loyalty.
    t.advance_to(P0, Step::Upkeep);
    let k = t.g.current(kaito);
    assert!(!t.obj_now(kaito).phased_out);
    assert!(!t.obj_now(aura).phased_out);
    assert_eq!(t.g.current(kaito), kaito, "no zone change");
    assert_eq!(t.obj_now(aura).attached_to, Some(Entity::Object(k)));
    assert_eq!(t.counters(kaito, counters::LOYALTY), loyalty);
    assert_eq!(t.stack_len(), 0, "no enters triggers");
    // He didn't enter this turn: no phasing out at this end step.
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "phases out"), 0);
    t.resolve_all();
    assert!(!t.obj_now(kaito).phased_out);
}
