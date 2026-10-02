//! Rulings batch P208 — enchant: Aura triggers about tapping, upkeep, entering creatures
//! and casting an Aura from the graveyard.

use crate::r_p209_common::cast_spell;
use crate::r_s01_common::*;
use crate::r_s04_common::*;
use crate::r_s06_common::*;
use crate::r_s21_common::castable;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::{Stage, Step};
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn relic_putrescence_trigger_after_a_mana_ability_and_before_other_abilities() {
    cr!("605.3a", "605.3b", "603.3", "602.2");
    ruling!(
        "Relic Putrescence",
        "If the enchanted artifact is tapped as a cost to activate a mana ability, the mana ability resolves immediately, then Relic Putrescence’s ability goes on the stack."
    );
    ruling!(
        "Relic Putrescence",
        "If the enchanted artifact is tapped as a cost to activate an ability that’s not a mana ability, Relic Putrescence’s ability will go on the stack on top of that activated ability and resolve first."
    );
    supported("Relic Putrescence");
    supported("Mind Stone");
    supported("Rod of Ruin");
    let mut t = TestGame::new(2);
    let stone = t.battlefield(P0, "Mind Stone");
    attach_new(&mut t, P1, "Relic Putrescence", stone);
    t.activate(P0, stone, 0, &[]).expect("mana ability");
    assert_eq!(t.g.player(P0).mana_pool.total(), 1);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.g.player(P0).poison(), 1);

    let mut t = TestGame::new(2);
    let rod = t.battlefield(P0, "Rod of Ruin");
    t.lands(P0, "Wastes", 3);
    attach_new(&mut t, P1, "Relic Putrescence", rod);
    t.activate(P0, rod, 0, &[Entity::Player(P1)]).unwrap();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    assert_eq!(t.g.player(P0).poison(), 1);
    assert_eq!(t.life(P1), 20);
    t.resolve();
    assert_eq!(t.life(P1), 19);
}

#[test]
fn disruption_aura_x_in_the_artifacts_mana_cost_is_zero() {
    cr!("107.3b", "202.1", "118.5");
    ruling!(
        "Disruption Aura",
        "If the enchanted artifact has X in its mana cost, X is 0."
    );
    supported("Disruption Aura");
    supported("Chalice of the Void");
    supported("Mind Stone");
    for (artifact, kept) in [("Chalice of the Void", true), ("Mind Stone", false)] {
        let mut t = TestGame::new(2);
        let a = t.battlefield(P0, artifact);
        attach_new(&mut t, P1, "Disruption Aura", a);
        next_upkeep(&mut t, P0);
        assert_eq!(t.stack_len(), 1, "{artifact}");
        t.answer_yes(P0, true);
        t.resolve_all();
        assert_eq!(t.on_battlefield(a), kept, "{artifact}");
    }
}

#[test]
fn illusory_gains_stays_put_when_it_cant_enchant_the_entering_creature() {
    cr!("303.4d", "702.16c", "701.3b");
    ruling!(
        "Illusory Gains",
        "If Illusory Gains can't legally enchant the creature that enters the battlefield (perhaps because it has protection from blue), it remains where it is. You retain control of the creature it's currently enchanting."
    );
    supported("Illusory Gains");
    supported("Guma");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let gains = attach_new(&mut t, P0, "Illusory Gains", bears);
    assert_eq!(t.obj_now(bears).controller, P0);
    t.enter(P1, "Guma");
    t.g.flush_events();
    t.settle();
    t.resolve_all();
    assert_eq!(attached_to(&t, gains), Some(Entity::Object(t.g.current(bears))));
    assert_eq!(t.obj_now(bears).controller, P0);
    // A creature it can enchant: it moves, and control of the Bears returns.
    let giant = t.enter(P1, "Hill Giant");
    t.g.flush_events();
    t.settle();
    t.resolve_all();
    assert_eq!(attached_to(&t, gains), Some(Entity::Object(giant)));
    assert_eq!(t.obj_now(giant).controller, P0);
    assert_eq!(t.obj_now(bears).controller, P1);
}

#[test]
fn paradox_haze_doesnt_trigger_when_the_upkeep_is_skipped() {
    cr!("614.10", "503.1");
    ruling!(
        "Paradox Haze",
        "If an effect causes the enchanted player to skip their upkeep step, this ability won't trigger."
    );
    supported("Paradox Haze");
    supported("Eon Hub");
    // Without an effect skipping the upkeep, it triggers.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::End);
    attach_new(&mut t, P0, "Paradox Haze", P0);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    // Eon Hub: no upkeep step, no trigger, no additional upkeep.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::End);
    attach_new(&mut t, P0, "Paradox Haze", P0);
    t.battlefield(P1, "Eon Hub");
    let mut triggered = false;
    let ok = t.g.run_until(10_000, |g| {
        triggered |= !g.stack.is_empty();
        g.turn.active == P0 && g.turn.step == Step::Draw && g.turn.stage == Stage::Priority
    });
    assert!(ok);
    assert!(!triggered);
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn demonic_embrace_can_be_cast_from_the_graveyard_before_anyone_else_acts() {
    cr!("117.3b", "117.3c", "601.3");
    ruling!(
        "Demonic Embrace",
        "If the enchanted creature dies during your turn, you receive priority before any other player does."
    );
    supported("Demonic Embrace");
    supported("Murder");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Hill Giant");
    attach_new(&mut t, P0, "Demonic Embrace", bears);
    t.hand(P0, "Hill Giant");
    let murder = cast_spell(&mut t, P1, "Murder", &[Entity::Object(bears)]);
    t.g.turn.priority = Some(P0);
    let from = t.asked().len();
    let ok = t.g.run_until(10_000, |g| {
        g.obj(g.current(murder)).zone != Zone::Stack
            && !g.find_in_zone(Zone::Graveyard(P0), "Demonic Embrace").is_empty()
            && g.turn.stage == Stage::Priority
            && g.turn.priority.is_some()
    });
    assert!(ok);
    assert!(t.in_graveyard(P0, "Demonic Embrace"));
    // P0 and P1 passed, Murder resolved, then P0 (the active player) got priority first.
    let asks = crate::r_s03_common::priority_asked_since(&t, from);
    assert_eq!(&asks[..3], &[P0, P1, P0]);
    let embrace = t.g.find_in_zone(Zone::Graveyard(P0), "Demonic Embrace")[0];
    t.lands(P0, "Swamp", 3);
    assert!(castable(&mut t, P0, embrace));
}
