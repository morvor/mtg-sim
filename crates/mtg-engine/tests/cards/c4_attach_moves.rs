//! Attaching as cards move (round-4 card flow, CR 303.4f, 701.3): an Aura returned to a
//! random creature it can enchant (Infectious Rage), a Curse moved to another opponent at
//! random (Maddening Hex), an Aura moved to another permanent of its host's type
//! (Enchantment Alteration), an Aura found attached to the source if it's still on the
//! battlefield (Auratouched Mage).

use crate::basic_effects_common::*;
use mtg_engine::events::MoveCause;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn infectious_rage_returns_to_a_creature_it_can_enchant() {
    cr!("303.4f", "702.16c");
    ruling!("Infectious Rage", "you can't place this card on a permanent with protection from red");
    ruling!("Infectious Rage", "so it can be placed on a creature with Shroud");
    assert_supported("Infectious Rage");
    for with_giant in [true, false] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        // Protection from red: the Aura can't enchant it.
        t.battlefield(P1, "Paladin en-Vec");
        if with_giant {
            t.battlefield(P1, "Hill Giant");
        }
        let rage = t.hand(P0, "Infectious Rage");
        t.lands(P0, "Mountain", 2);
        t.cast(P0, rage).target(Entity::Object(bears)).go();
        t.resolve();
        // The Bears die; the Aura returns to a creature it can enchant.
        t.g.move_object(bears, Zone::Graveyard(P1), MoveCause::Effect, None);
        t.resolve_all();
        let r = t.named_on_battlefield("Infectious Rage");
        if with_giant {
            assert_eq!(r.len(), 1);
            let host = t.obj_now(r[0]).attached_to.and_then(|e| e.object()).unwrap();
            assert_eq!(t.obj_now(host).name(), "Hill Giant");
        } else {
            assert!(r.is_empty());
            assert!(t.in_graveyard(P0, "Infectious Rage"));
        }
    }
}

#[test]
fn maddening_hex_moves_to_another_opponent_at_random() {
    cr!("701.3a", "706.2");
    ruling!("Maddening Hex", "If Maddening Hex is attached to your only opponent, it stays attached");
    assert_supported("Maddening Hex");
    let mut t = TestGame::new(3);
    let hex = t.battlefield(P0, "Maddening Hex");
    t.g.attach(hex, Entity::Player(P1));
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(Entity::Player(P0)).go();
    t.resolve_all();
    // P1 took 1-6 damage; the Hex is now on P2, the other opponent.
    assert!(t.life(P1) < 20 && t.life(P1) >= 14);
    assert_eq!(t.obj_now(hex).attached_to, Some(Entity::Player(P2)));
    // With only one opponent it stays.
    let mut t = TestGame::new(2);
    let hex = t.battlefield(P0, "Maddening Hex");
    t.g.attach(hex, Entity::Player(P1));
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(Entity::Player(P0)).go();
    t.resolve_all();
    assert_eq!(t.obj_now(hex).attached_to, Some(Entity::Player(P1)));
}

#[test]
fn enchantment_alteration_moves_an_aura_to_a_permanent_of_its_hosts_type() {
    cr!("701.3a", "115.1");
    ruling!("Enchantment Alteration", "Only targets the Aura");
    assert_supported("Enchantment Alteration");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.battlefield(P0, "Forest");
    let aura = t.battlefield(P0, "Holy Strength");
    t.g.attach(aura, Entity::Object(bears));
    t.lands(P0, "Island", 1);
    let ea = t.hand(P0, "Enchantment Alteration");
    t.answer_choose(P0, &[Entity::Object(giant)]);
    t.cast(P0, ea).target(Entity::Object(aura)).go();
    t.resolve();
    assert_eq!(t.obj_now(aura).attached_to, Some(Entity::Object(giant)));
    // Only creatures (not the Forest) were offered.
    let offered = t.asked().into_iter().find_map(|(_, d)| match d {
        decision::Decision::ChooseEntities { candidates, .. } => Some(candidates),
        _ => None,
    });
    assert_eq!(offered, Some(vec![Entity::Object(giant)]));
}

#[test]
fn auratouched_mage_attaches_the_aura_if_still_on_the_battlefield() {
    cr!("701.23a", "303.4f");
    ruling!("Auratouched Mage", "or as it most recently existed on the battlefield");
    assert_supported("Auratouched Mage");
    let mut t = TestGame::new(2);
    let hs = t.library_top(P0, "Holy Strength");
    t.answer_choose(P0, &[Entity::Object(hs)]);
    let mage = t.enter(P0, "Auratouched Mage");
    t.resolve_all();
    let a = t.named_on_battlefield("Holy Strength");
    assert_eq!(a.len(), 1);
    assert_eq!(t.obj_now(a[0]).attached_to, Some(Entity::Object(mage)));
    // If it left the battlefield, the Aura goes to the hand.
    let mut t = TestGame::new(2);
    let hs = t.library_top(P0, "Holy Strength");
    t.answer_choose(P0, &[Entity::Object(hs)]);
    let mage = t.enter(P0, "Auratouched Mage");
    t.settle();
    t.g.move_object(mage, Zone::Graveyard(P0), MoveCause::Effect, None);
    t.resolve_all();
    assert!(t.in_hand(P0, "Holy Strength"));
}
