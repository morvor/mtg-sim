//! Rulings batch S06 — enchant land: the Zendikons ("Enchanted land is a 6/4 green
//! Elemental creature. It's still a land. When enchanted land dies, return that card to
//! its owner's hand.").

use crate::r_s01_common::*;
use crate::r_s02_common::destroy;
use crate::r_s03_common::{in_hand_with_mana, to_blockers};
use crate::r_s04_common::*;
use crate::r_s05_common::move_to;
use crate::r_s06_common::*;
use mtg_engine::ability::{Effect, Sel, Value};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn a_zendikon_overwrites_earlier_pt_setting_but_modifications_still_apply() {
    cr!("613.4b", "613.4c", "613.4d", "613.7a");
    ruling!(
        "Vastwood Zendikon",
        "An ability that turns a land into a creature also sets that creature's power and toughness. If the land was already a creature, this will overwrite the previous effect that set its power and toughness. Effects that modify its power or toughness, such as the effects of Disfigure or Glorious Anthem, will continue to apply, no matter when they started to take effect. The same is true for counters that change its power or toughness (such as +1/+1 counters) and effects that switch its power and toughness."
    );
    supported("Vastwood Zendikon");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Glorious Anthem");
    let vault = t.battlefield(P0, "Mutavault");
    run_with(
        &mut t,
        P0,
        Effect::AddCounters {
            what: Sel::Target(0),
            kind: "+1/+1".into(),
            n: Value::Const(1),
        },
        &[Entity::Object(vault)],
    );
    // Mutavault becomes a 2/2 creature: 2/2, +1/+1 counter, +1/+1 from the Anthem.
    t.lands(P0, "Wastes", 1);
    activate_containing(&mut t, P0, vault, "becomes").expect("animate Mutavault");
    t.resolve();
    assert_eq!(t.pt(vault), (4, 4));
    // Disfigure: -2/-2 until end of turn.
    t.lands(P0, "Swamp", 1);
    let disfigure = t.hand(P0, "Disfigure");
    t.cast(P0, disfigure).target(vault).go();
    t.resolve();
    assert_eq!(t.pt(vault), (2, 2));
    // The Zendikon's 6/4 overwrites Mutavault's 2/2; the rest still applies.
    attach_new(&mut t, P0, "Vastwood Zendikon", vault);
    assert_eq!(t.pt(vault), (6, 4));
    assert!(t.obj_now(vault).is(CardType::Land));
    // Switching its power and toughness applies last.
    t.lands(P0, "Island", 2);
    let inside = t.hand(P0, "Inside Out");
    t.cast(P0, inside).target(vault).go();
    t.resolve();
    assert_eq!(t.pt(vault), (4, 6));
}

#[test]
fn a_zendikon_destroyed_with_its_land_still_returns_the_land() {
    cr!("603.10a", "603.6c", "700.4");
    ruling!(
        "Wind Zendikon",
        "If a Zendikon and the land it's enchanting are destroyed at the same time (due to Akroma's Vengeance, for example), the Zendikon's last ability will still trigger."
    );
    supported("Wind Zendikon");
    supported("Akroma's Vengeance");
    let mut t = TestGame::new(2);
    let forest = t.battlefield(P0, "Forest");
    attach_new(&mut t, P0, "Wind Zendikon", forest);
    let wrath = in_hand_with_mana(&mut t, P0, "Akroma's Vengeance");
    t.cast(P0, wrath).go();
    t.resolve();
    assert!(t.in_graveyard(P0, "Wind Zendikon"));
    assert_eq!(on_stack(&t, "return that card"), 1);
    t.resolve_all();
    assert!(t.in_hand(P0, "Forest"));
    assert!(!t.in_graveyard(P0, "Forest"));
}

#[test]
fn the_land_card_leaving_the_graveyard_makes_the_return_do_nothing() {
    cr!("400.7", "608.2b");
    ruling!(
        "Crusher Zendikon",
        "If a Zendikon's last ability triggers, but the land card it refers to leaves the graveyard before it resolves, it will resolve but do nothing."
    );
    supported("Crusher Zendikon");
    let mut t = TestGame::new(2);
    let mountain = t.battlefield(P0, "Mountain");
    attach_new(&mut t, P0, "Crusher Zendikon", mountain);
    destroy(&mut t, mountain);
    assert_eq!(on_stack(&t, "return that card"), 1);
    let card = t.g.find_in_zone(Zone::Graveyard(P0), "Mountain")[0];
    move_to(&mut t, card, Zone::Exile);
    t.resolve_all();
    assert!(t.in_exile("Mountain"));
    assert!(!t.in_hand(P0, "Mountain"));
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn a_land_that_stops_being_a_creature_is_removed_from_combat() {
    cr!("506.4", "506.4a", "509.1h", "510.1c");
    ruling!(
        "Guardian Zendikon",
        "An attacking or blocking creature that stops being a creature is removed from combat. This can happen if a Zendikon enchanting an attacking or blocking creature leaves the battlefield, for example. The permanent that was removed from combat neither deals nor is dealt combat damage. Any attacking creature that the land creature was blocking remains blocked, however."
    );
    supported("Guardian Zendikon");
    // Blocking: Guardian Zendikon makes a 2/6 Wall with defender.
    let mut t = TestGame::new(2);
    let plains = t.battlefield(P1, "Plains");
    let zendikon = attach_new(&mut t, P1, "Guardian Zendikon", plains);
    let giant = t.battlefield(P0, "Hill Giant");
    to_blockers(&mut t, &[(giant, Entity::Player(P1))], &[(plains, giant)]);
    assert!(t.g.is_blocking(plains));
    destroy(&mut t, zendikon);
    assert!(!is_creature(&t, plains));
    assert!(!t.g.is_blocking(plains));
    assert!(t.g.combat.as_ref().unwrap().is_blocked(giant));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.obj_now(plains).damage, 0);
    assert_eq!(t.obj_now(giant).damage, 0);
    // Attacking: Vastwood Zendikon makes a 6/4; with the Zendikon gone, the land is
    // removed from combat and deals no damage.
    let mut t = TestGame::new(2);
    let forest = t.battlefield(P0, "Forest");
    let zendikon = attach_new(&mut t, P0, "Vastwood Zendikon", forest);
    attack_with(&mut t, &[(forest, Entity::Player(P1))]);
    assert!(t.g.is_attacking(forest));
    destroy(&mut t, zendikon);
    assert!(!t.g.is_attacking(forest));
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.life(P1), 20);
}
