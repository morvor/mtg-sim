//! CR 801.7: a delayed triggered ability that triggers only once doesn't trigger on an
//! event outside its controller's range of influence either; it waits for the next one.

use super::r800_common::*;
use mtg_engine::ability::*;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn a_once_only_delayed_trigger_ignores_events_outside_range() {
    cr!("801.7", "603.7c");
    let mut t = ranged(5, 1);
    // "When a creature dies [the next time], you gain 1 life."
    apply(
        &mut t,
        P0,
        Effect::DelayedTrigger {
            trigger: TriggerCond::Dies(Filter::Type(CardType::Creature)),
            body: Box::new(Body::effect(Effect::GainLife {
                who: PlayerRef::You,
                n: Value::c(1),
            })),
            once: true,
        },
        &[],
    );
    let far = bear(&mut t, P2);
    let near = bear(&mut t, P1);
    let before = t.life(P0);
    // P2 is outside P0's range of influence: no trigger, and it keeps waiting.
    t.g.destroy(far, None);
    t.settle();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.g.delayed_triggers.len(), 1);
    // P1 is within range: it triggers, once.
    t.g.destroy(near, None);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.life(P0), before + 1);
    assert!(t.g.delayed_triggers.is_empty());
}
