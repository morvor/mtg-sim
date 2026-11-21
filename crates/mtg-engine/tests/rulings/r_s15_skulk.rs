//! Rulings batch S15 — skulk (CR 702.118): Persistent Nightmare, the back face of Startled
//! Awake.

use crate::r_s01_common::*;
use crate::r_s03_common::{run_effect, to_blockers};
use crate::r_s10_common::{attacking, blocking};
use crate::r_s11_common::triggered_from;
use mtg_engine::ability::*;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// P0's Persistent Nightmare (1/1, skulk, "When this creature deals combat damage to a
/// player, return it to its owner's hand."), put onto the battlefield transformed by
/// Startled Awake's "{3}{U}{U}: Put this card from your graveyard onto the battlefield
/// transformed. Activate only as a sorcery." It has been under P0's control since the
/// turn began.
fn nightmare(t: &mut TestGame) -> ObjectId {
    supported("Startled Awake // Persistent Nightmare");
    let card = t.graveyard(P0, "Startled Awake // Persistent Nightmare");
    t.lands(P0, "Island", 2);
    t.lands(P0, "Wastes", 3);
    t.activate(P0, card, 0, &[]).unwrap();
    t.resolve_all();
    let n = t.g.current(card);
    assert!(t.on_battlefield(n));
    assert_eq!(t.obj(n).chars.name.as_str(), "Persistent Nightmare");
    assert!(t.obj(n).has_keyword(KeywordKind::Skulk));
    t.g.obj_mut(n).summoning_sick = false;
    n
}

/// `id` gets +p/+0 until end of turn.
fn pump(t: &mut TestGame, id: ObjectId, p: i32) {
    run_effect(
        t,
        None,
        P0,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::ModifyPT(Value::c(p), Value::c(0))],
            duration: Duration::EndOfTurn,
        },
        &[Entity::Object(id)],
    );
}

#[test]
fn skulk_matters_only_as_blockers_are_chosen() {
    cr!("702.118b", "509.1b");
    ruling!(
        "Startled Awake // Persistent Nightmare",
        "Skulk matters only as blockers are chosen. Modifying either creature's power after blockers are chosen won't cause the attacking creature to become unblocked."
    );
    // A 1/1 can block the 1/1 Nightmare; a 2/2 can't.
    let mut t = TestGame::new(2);
    let n = nightmare(&mut t);
    let elves = t.battlefield(P1, "Llanowar Elves");
    let bears = t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(n, Entity::Player(P1))]);
    assert!(t.g.can_block(elves, n));
    assert!(!t.g.can_block(bears, n));
    // Once the Elves block, raising the blocker's power (or lowering the attacker's)
    // doesn't make the Nightmare unblocked.
    for attacker_shrinks in [false, true] {
        let mut t = TestGame::new(2);
        let n = nightmare(&mut t);
        let elves = t.battlefield(P1, "Llanowar Elves");
        to_blockers(&mut t, &[(n, Entity::Player(P1))], &[(elves, n)]);
        if attacker_shrinks {
            pump(&mut t, n, -1);
        } else {
            pump(&mut t, elves, 2);
        }
        assert!(t.g.combat.as_ref().unwrap().is_blocked(n));
        assert!(blocking(&t, elves));
        t.advance_to(P0, Step::EndOfCombat);
        assert_eq!(t.life(P1), 20);
        assert!(!t.on_battlefield(elves) || attacker_shrinks);
    }
}

#[test]
fn a_skulk_creature_with_negative_power_uses_the_actual_value_and_deals_no_damage() {
    cr!("702.118b", "510.1a");
    ruling!(
        "Startled Awake // Persistent Nightmare",
        "If you cause a creature to have 0 power or less, use the actual value (which may be negative) to determine whether it can block or be blocked. A creature with skulk and 0 or less power most likely won't be blocked, but it won't deal combat damage and won't trigger any abilities that trigger when combat damage is dealt."
    );
    let mut t = TestGame::new(2);
    let n = nightmare(&mut t);
    // The Nightmare becomes -1/1. A 0/8 wall has greater power; a creature at -2 power
    // doesn't.
    let wall = t.battlefield(P1, "Wall of Stone");
    let bears = t.battlefield(P1, "Grizzly Bears");
    pump(&mut t, n, -2);
    pump(&mut t, bears, -4);
    assert_eq!(t.pt(n).0, -1);
    attack_with(&mut t, &[(n, Entity::Player(P1))]);
    assert!(!t.g.can_block(wall, n));
    assert!(t.g.can_block(bears, n));
    // Unblocked, it deals no combat damage, so its "deals combat damage to a player"
    // ability doesn't trigger: it stays on the battlefield.
    t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![]));
    t.advance_to(P0, Step::EndOfCombat);
    assert!(attacking(&t, n));
    assert_eq!(t.life(P1), 20);
    assert_eq!(triggered_from(&t, n), 0);
    assert!(t.on_battlefield(n));
    // At 1 power it deals damage, and the ability returns it to its owner's hand.
    let mut t = TestGame::new(2);
    let n = nightmare(&mut t);
    attack_with(&mut t, &[(n, Entity::Player(P1))]);
    t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![]));
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    assert_eq!(t.zone(n), Zone::Hand(P0));
}
