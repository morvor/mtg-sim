//! Rulings batch S06 — enrage (ability word, CR 207.2c): "Whenever this creature is dealt
//! damage, [effect]."

use crate::r_s01_common::*;
use crate::r_s04_common::*;
use crate::r_s06_common::*;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn enrage_triggers_once_for_damage_dealt_by_several_blockers_at_once() {
    cr!("207.2c", "510.2", "603.2c");
    ruling!(
        "Ripjaw Raptor",
        "If multiple sources deal damage to a creature with an enrage ability at the same time, most likely because multiple creatures blocked that creature, the enrage ability triggers only once."
    );
    supported("Ripjaw Raptor");
    // Ripjaw Raptor (4/5): "Enrage — Whenever this creature is dealt damage, draw a card."
    let mut t = TestGame::new(2);
    let raptor = t.battlefield(P0, "Ripjaw Raptor");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    let hand = t.hand_size(P0);
    t.attack(&[(raptor, Entity::Player(P1))], &[(a, raptor), (b, raptor)]);
    assert!(t.on_battlefield(raptor));
    assert_eq!(t.obj_now(raptor).damage, 4);
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn enrage_triggers_on_lethal_damage_but_the_creature_is_gone_when_it_resolves() {
    cr!("207.2c", "603.10a", "704.5g", "608.2h");
    ruling!(
        "Siegehorn Ceratops",
        "If lethal damage is dealt to a creature with an enrage ability, that ability triggers. The creature with that enrage ability leaves the battlefield before that ability resolves, so it won’t be affected by the resolving ability."
    );
    supported("Siegehorn Ceratops");
    // Siegehorn Ceratops (2/2): "Enrage — Whenever this creature is dealt damage, put two
    // +1/+1 counters on it."
    let mut t = TestGame::new(2);
    let ceratops = t.battlefield(P0, "Siegehorn Ceratops");
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(ceratops).go();
    t.resolve();
    assert!(t.in_graveyard(P0, "Siegehorn Ceratops"));
    assert_eq!(on_stack(&t, "+1/+1 counters"), 1);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Siegehorn Ceratops"));
    assert_eq!(t.counters(ceratops, "+1/+1"), 0);
    assert!(t
        .g
        .permanents()
        .all(|o| o.counter("+1/+1") == 0));
    // Nonlethal damage: it survives and gets the counters.
    let mut t = TestGame::new(2);
    let ceratops = t.battlefield(P0, "Siegehorn Ceratops");
    let source = t.battlefield(P1, "Grizzly Bears");
    damage(&mut t, source, 1, ceratops);
    t.resolve_all();
    assert_eq!(t.counters(ceratops, "+1/+1"), 2);
}

#[test]
fn ripjaw_raptor_dealt_lethal_damage_still_draws() {
    cr!("207.2c", "603.10a", "704.5g");
    ruling!(
        "Ripjaw Raptor",
        "If lethal damage is dealt to a creature with an enrage ability, that ability triggers. The creature with that enrage ability leaves the battlefield before that ability resolves, so it won't be affected by the resolving ability."
    );
    let mut t = TestGame::new(2);
    let raptor = t.battlefield(P0, "Ripjaw Raptor");
    let source = t.battlefield(P1, "Grizzly Bears");
    let hand = t.hand_size(P0);
    damage(&mut t, source, 5, raptor);
    assert!(t.in_graveyard(P0, "Ripjaw Raptor"));
    assert_eq!(on_stack(&t, "draw a card"), 1);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}
