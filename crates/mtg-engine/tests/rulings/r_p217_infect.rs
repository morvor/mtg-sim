//! Rulings batch P217 — infect (CR 702.90): damage to creatures as -1/-1 counters, to
//! players as poison counters; damage to planeswalkers and battles is as usual
//! (CR 120.3c–e).

use crate::r_s01_common::{give_mana_for, supported, triggers_on_stack};
use crate::r_s10_common::poison;
use mtg_engine::ability::{Effect, Sel, Value};
use mtg_engine::testing::*;
use mtg_engine::types::counters;
use mtg_engine::*;

const SEGOVIA: &str = "Invasion of Segovia // Caetus, Sea Tyrant of Segovia";

/// `source` deals `n` damage to `to` (as a resolving effect of its controller's would),
/// then settles.
fn deal(t: &mut TestGame, source: ObjectId, n: i32, to: Entity) {
    let mut ctx = mtg_engine::eval::Ctx::new(Some(source), t.obj_now(source).controller);
    ctx.targets = vec![vec![to]];
    t.g.exec(
        &Effect::DealDamage {
            source: Sel::This,
            amount: Value::Const(n),
            to: Sel::Target(0),
        },
        &mut ctx,
    );
    t.g.flush_events();
    t.settle();
}

/// Damage from the infect creature `name` to a creature, a player, a planeswalker and
/// (if `battle`) a battle.
fn infect_damage(name: &str, battle: bool) {
    supported(name);
    let mut t = TestGame::new(2);
    let src = t.battlefield(P0, name);
    // A creature: -1/-1 counters, no damage marked.
    let giant = t.battlefield(P1, "Craw Wurm");
    deal(&mut t, src, 2, Entity::Object(giant));
    assert_eq!(t.counters(giant, counters::MINUS1), 2);
    assert_eq!(t.obj_now(giant).damage, 0);
    assert_eq!(t.pt(giant), (4, 2));
    // A player: poison counters, no life lost.
    deal(&mut t, src, 3, Entity::Player(P1));
    assert_eq!(poison(&t, P1), 3);
    assert_eq!(t.life(P1), 20);
    // A planeswalker: it loses that many loyalty counters (Jace Beleren, loyalty 3).
    let jace = t.battlefield(P1, "Jace Beleren");
    assert_eq!(t.counters(jace, counters::LOYALTY), 3);
    deal(&mut t, src, 2, Entity::Object(jace));
    assert_eq!(t.counters(jace, counters::LOYALTY), 1);
    assert_eq!(t.counters(jace, counters::MINUS1), 0);
    if battle {
        // A battle: it loses that many defense counters.
        let b = t.battlefield(P1, SEGOVIA);
        let d = t.counters(b, counters::DEFENSE);
        assert!(d > 2);
        deal(&mut t, src, 2, Entity::Object(b));
        assert_eq!(t.counters(b, counters::DEFENSE), d - 2);
        assert_eq!(t.counters(b, counters::MINUS1), 0);
    }
}

#[test]
fn infect_damage_to_creatures_players_and_planeswalkers() {
    cr!("702.90b", "702.90c", "120.3b", "120.3c", "120.3d");
    ruling!(
        "Blightsteel Colossus",
        "Damage that a creature with infect deals doesn't result in damage being marked on a creature or a player losing life. Instead, it results in that many -1/-1 counters being put on that creature or that many poison counters being given to that player. Damage dealt to planeswalkers still results in that planeswalker losing that many loyalty counters."
    );
    infect_damage("Blightsteel Colossus", false);
}

#[test]
fn infect_damage_to_creatures_players_planeswalkers_and_battles() {
    cr!("702.90b", "702.90c", "120.3b", "120.3c", "120.3d", "120.3h");
    ruling!(
        "Skithiryx, the Blight Dragon",
        "Damage that a creature with infect deals doesn't result in damage being marked on a creature or a player losing life. Instead, it results in that many -1/-1 counters being put on that creature or that many poison counters being given to that player. Damage dealt to planeswalkers still results in that planeswalker losing that many loyalty counters. Damage dealt to battles still results in that battle losing that many defense counters."
    );
    infect_damage("Skithiryx, the Blight Dragon", true);
}

const PRAETORS: &str = "cast a creature spell with infect";

#[test]
fn casting_hand_of_the_praetors_doesnt_trigger_its_own_ability() {
    cr!("603.2", "603.6", "702.90a");
    ruling!(
        "Hand of the Praetors",
        "The last ability triggers only if Hand of the Praetors is already on the battlefield at the time you cast a creature spell with infect. Casting Hand of the Praetors itself will not cause its own last ability to trigger."
    );
    supported("Hand of the Praetors");
    let mut t = TestGame::new(2);
    give_mana_for(&mut t, P0, "Hand of the Praetors");
    let hand = t.hand(P0, "Hand of the Praetors");
    t.cast(P0, hand).go();
    t.settle();
    assert_eq!(triggers_on_stack(&t, PRAETORS), 0);
    t.resolve_all();
    assert_eq!(poison(&t, P1), 0);
    // Once it's on the battlefield, casting another one does.
    give_mana_for(&mut t, P0, "Hand of the Praetors");
    let other = t.hand(P0, "Hand of the Praetors");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.cast(P0, other).go();
    t.settle();
    assert_eq!(triggers_on_stack(&t, PRAETORS), 1);
    t.resolve_all();
    assert_eq!(poison(&t, P1), 1);
}

#[test]
fn hand_of_the_praetors_trigger_resolves_before_the_creature_spell() {
    cr!("603.3", "405.2", "601.2i");
    ruling!(
        "Hand of the Praetors",
        "Whenever you cast a creature spell with infect, Hand of the Praetors’s last ability triggers and goes on the stack on top of it. It will resolve before the creature spell does."
    );
    supported("Plague Stinger");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hand of the Praetors");
    give_mana_for(&mut t, P0, "Plague Stinger");
    let stinger = t.hand(P0, "Plague Stinger");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    let spell = t.cast(P0, stinger).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    assert_eq!(t.g.stack[0], spell);
    assert_eq!(triggers_on_stack(&t, PRAETORS), 1);
    // The trigger resolves first: P1 is poisoned while the Stinger is still a spell.
    t.resolve();
    assert_eq!(poison(&t, P1), 1);
    assert_eq!(t.g.stack, vec![spell]);
    t.resolve();
    assert!(!t.named_on_battlefield("Plague Stinger").is_empty());
}
