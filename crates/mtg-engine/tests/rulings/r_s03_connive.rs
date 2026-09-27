//! Rulings batch S03 — connive (CR 701.50): "Draw a card, then discard a card. If you
//! discarded a nonland card, put a +1/+1 counter on this creature."

use crate::r_s01_common::*;
use crate::r_s03_common::*;
use mtg_engine::decision::Decision;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn nobody_can_act_while_a_connive_ability_resolves() {
    cr!("701.50a", "608.2c", "117.4");
    ruling!(
        "Ledger Shredder",
        "Once an ability that causes a creature to connive begins to resolve, no player may take any other actions until it's done. Notably, opponents can't try to remove the conniving creature after you discard a nonland card but before it receives a counter."
    );
    supported("Ledger Shredder");
    // Ledger Shredder: "Whenever a player casts their second spell each turn, this
    // creature connives."
    let mut t = TestGame::new(2);
    let shredder = t.battlefield(P0, "Ledger Shredder");
    let bolt = t.hand(P0, "Lightning Bolt");
    for _ in 0..2 {
        let shock = in_hand_with_mana(&mut t, P0, "Shock");
        t.cast(P0, shock).target(P1).go();
        t.settle();
    }
    assert_eq!(triggers_on_stack(&t, "connives"), 1);
    // Every time P1 is asked anything, note the Shredder's counters and whether the Bolt
    // has been discarded.
    let seen = watch(
        &mut t,
        P1,
        |_| true,
        |g| {
            let shredder = g.find_in_zone(Zone::Battlefield, "Ledger Shredder")[0];
            (
                g.obj(shredder).counter("+1/+1"),
                g.find_in_zone(Zone::Graveyard(P0), "Lightning Bolt").len(),
                g.stack.len(),
            )
        },
    );
    let from = t.asked().len();
    t.answer_choose(P0, &[Entity::Object(bolt)]);
    let ok = t.g.run_until(1_000, |g| g.stack.is_empty());
    assert!(ok);
    assert!(t.in_graveyard(P0, "Lightning Bolt"));
    assert_eq!(t.counters(shredder, "+1/+1"), 1);
    // P1 had priority with the trigger on the stack, and again once it had resolved, but
    // never between the discard and the counter.
    let seen = seen.lock().unwrap().clone();
    assert!(seen.contains(&(0, 0, 3)));
    assert!(seen.iter().any(|s| s.0 == 1 && s.1 == 1));
    assert!(seen.iter().all(|s| s.1 == 0 || s.0 == 1));
    // While it resolved, the only decision was P0's discard.
    let asked = t.asked()[from..].to_vec();
    let discard = asked
        .iter()
        .position(|(_, d)| matches!(d, Decision::ChooseEntities { .. }))
        .unwrap();
    assert_eq!(asked[discard].0, P0);
    assert!(matches!(asked[discard + 1], (p, Decision::Priority { .. }) if p == P0));
}

#[test]
fn a_creature_that_left_the_battlefield_still_connives_and_triggers_connive_abilities() {
    cr!("701.50a", "701.50b", "701.50f", "603.10a");
    ruling!(
        "Doc Ock's Henchmen",
        "If a resolving spell or ability instructs a specific creature to connive but that creature has left the battlefield, the creature still connives. If you discard a nonland card this way, you won't put a +1/+1 counter on anything. Abilities that trigger \"when [that creature] connives\" will trigger."
    );
    supported("Doc Ock's Henchmen");
    supported("Iron Monger, Sadistic Tycoon");
    // Doc Ock's Henchmen: "Whenever this creature attacks, it connives." Iron Monger:
    // "Whenever a creature you control connives, put a +1/+1 counter on each Villain you
    // control."
    let mut t = TestGame::new(2);
    let monger = t.battlefield(P0, "Iron Monger, Sadistic Tycoon");
    let henchmen = t.battlefield(P0, "Doc Ock's Henchmen");
    let bolt = t.hand(P0, "Lightning Bolt");
    attack_with(&mut t, &[(henchmen, Entity::Player(P1))]);
    assert_eq!(triggers_on_stack(&t, "connives"), 1);
    // In response, the Henchmen are destroyed.
    let h = t.g.current(henchmen);
    t.g.destroy(h, None);
    t.settle();
    assert!(t.in_graveyard(P0, "Doc Ock's Henchmen"));
    let hand = t.hand_size(P0);
    t.answer_choose(P0, &[Entity::Object(bolt)]);
    t.resolve_all();
    // P0 drew a card and discarded the Bolt (a nonland card) ...
    assert!(t.in_graveyard(P0, "Lightning Bolt"));
    assert_eq!(t.hand_size(P0), hand);
    // ... no +1/+1 counter was put on anything but by Iron Monger's trigger, which
    // triggered: the Henchmen connived. (Iron Monger is a Villain.)
    assert_eq!(t.counters(monger, "+1/+1"), 1);
    let henchmen_now = t.g.current(henchmen);
    assert_eq!(t.obj(henchmen_now).counter("+1/+1"), 0);
    assert!(t
        .g
        .permanents()
        .filter(|o| o.id != monger)
        .all(|o| o.counter("+1/+1") == 0));
}
