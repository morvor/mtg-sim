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
    interp!("alteration-host-of-two-types");
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

#[test]
fn curse_of_misfortunes_finds_a_curse_with_another_name_attached_to_that_player() {
    cr!("303.4f", "701.23a");
    assert_supported("Curse of Misfortunes");
    let mut t = TestGame::new(2);
    let com = t.battlefield(P0, "Curse of Misfortunes");
    t.g.attach(com, Entity::Player(P1));
    let ph = t.battlefield(P0, "Curse of the Pierced Heart");
    t.g.attach(ph, Entity::Player(P1));
    let same = t.library_top(P0, "Curse of the Pierced Heart");
    let other = t.library_top(P0, "Curse of Stalked Prey");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(other)]);
    t.advance_to(P0, mtg_engine::turn::Step::Draw);
    let s = t.named_on_battlefield("Curse of Stalked Prey");
    assert_eq!(s.len(), 1);
    assert_eq!(t.obj_now(s[0]).attached_to, Some(Entity::Player(P1)));
    // The Curse with a name already on that player wasn't offered.
    let offered = t.asked().into_iter().find_map(|(_, d)| match d {
        decision::Decision::ChooseEntities { candidates, .. } => Some(candidates),
        _ => None,
    });
    let offered = offered.expect("a choice of Curse cards");
    assert!(!offered.contains(&Entity::Object(same)));
}

#[test]
fn flickerform_returns_the_creature_and_its_auras_attached_to_it() {
    cr!("400.7", "303.4f", "603.7a");
    assert_supported("Flickerform");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let ff = t.battlefield(P0, "Flickerform");
    t.g.attach(ff, Entity::Object(bears));
    let hs = t.battlefield(P0, "Holy Strength");
    t.g.attach(hs, Entity::Object(bears));
    t.lands(P0, "Plains", 4);
    t.activate(P0, ff, 0, &[]).unwrap();
    t.resolve();
    assert!(t.in_exile("Grizzly Bears") && t.in_exile("Flickerform") && t.in_exile("Holy Strength"));
    t.advance_to_step(mtg_engine::turn::Step::End);
    t.resolve_all();
    let b = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(b.len(), 1);
    for n in ["Flickerform", "Holy Strength"] {
        let a = t.named_on_battlefield(n);
        assert_eq!(a.len(), 1, "{n}");
        assert_eq!(t.obj_now(a[0]).attached_to, Some(Entity::Object(b[0])), "{n}");
    }
    assert_eq!(t.pt(b[0]), (3, 4));
}

#[test]
fn bruna_attaches_auras_and_puts_aura_cards_onto_the_battlefield_attached_to_it() {
    cr!("701.3a", "303.4f");
    assert_supported("Bruna, Light of Alabaster");
    let mut t = TestGame::new(2);
    let bruna = t.battlefield(P0, "Bruna, Light of Alabaster");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let on_bears = t.battlefield(P1, "Holy Strength");
    t.g.attach(on_bears, Entity::Object(bears));
    let in_gy = t.graveyard(P0, "Holy Strength");
    let in_hand = t.hand(P0, "Unholy Strength");
    t.advance_to(P0, mtg_engine::turn::Step::BeginningOfCombat);
    let (p, tough) = t.pt(bruna);
    t.answer_choose(P0, &[Entity::Object(on_bears)]);
    t.answer_choose(P0, &[Entity::Object(in_gy), Entity::Object(in_hand)]);
    t.attack(&[(bruna, Entity::Player(P1))], &[]);
    let on_bruna = t.g.attachments_of(Entity::Object(bruna));
    assert_eq!(on_bruna.len(), 3);
    // The Aura P1 controlled is still P1's, now on Bruna.
    assert_eq!(t.obj_now(on_bears).attached_to, Some(Entity::Object(bruna)));
    // Holy Strength twice (+1/+2) and Unholy Strength (+2/+1).
    assert_eq!(t.pt(bruna), (p + 1 + 1 + 2, tough + 2 + 2 + 1));
}

#[test]
fn cass_returns_the_auras_and_moves_the_equipment_that_were_attached_to_it() {
    cr!("608.2h", "303.4f", "701.3a");
    assert_supported("Cass, Hand of Vengeance");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Cass, Hand of Vengeance");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let hs = t.battlefield(P0, "Holy Strength");
    t.g.attach(hs, Entity::Object(bears));
    let other = t.graveyard(P0, "Unholy Strength");
    let sword = t.battlefield(P0, "Short Sword");
    t.g.attach(sword, Entity::Object(bears));
    t.answer_targets(P0, &[Entity::Object(giant)]);
    t.g.move_object(bears, Zone::Graveyard(P0), MoveCause::Destroy, None);
    t.settle();
    let hs_gy = t.g.current(hs);
    assert_eq!(t.zone(hs_gy), Zone::Graveyard(P0));
    t.answer_choose(P0, &[Entity::Object(hs_gy)]);
    t.answer_choose(P0, &[Entity::Object(sword)]);
    t.resolve_all();
    let h = t.named_on_battlefield("Holy Strength");
    assert_eq!(h.len(), 1);
    assert_eq!(t.obj_now(h[0]).attached_to, Some(Entity::Object(giant)));
    assert_eq!(t.obj_now(sword).attached_to, Some(Entity::Object(giant)));
    // An Aura card that wasn't attached to it stays in the graveyard (and wasn't offered).
    assert_eq!(t.zone(other), Zone::Graveyard(P0));
    let offered = t.asked().into_iter().find_map(|(_, d)| match d {
        decision::Decision::ChooseEntities { candidates, .. } => Some(candidates),
        _ => None,
    });
    assert!(!offered.unwrap().contains(&Entity::Object(other)));
}

#[test]
fn storm_herald_returns_auras_attached_to_creatures_you_control_then_exiles_them() {
    cr!("303.4f", "614.1a", "603.7a");
    interp!("auras-returned-to-chosen-hosts-one-at-a-time");
    assert_supported("Storm Herald");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    t.battlefield(P1, "Craw Wurm");
    let hs = t.graveyard(P0, "Holy Strength");
    let us = t.graveyard(P0, "Unholy Strength");
    t.answer_choose(P0, &[Entity::Object(hs), Entity::Object(us)]);
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.answer_choose(P0, &[Entity::Object(giant)]);
    t.enter(P0, "Storm Herald");
    t.resolve_all();
    assert_eq!(t.pt(bears), (3, 4));
    assert_eq!(t.pt(giant), (5, 4));
    // The host choices were only creatures P0 controls.
    let offered = t.asked().into_iter().filter_map(|(_, d)| match d {
        decision::Decision::ChooseEntities { candidates, max: 1, .. } => Some(candidates),
        _ => None,
    }).next().unwrap();
    assert_eq!(offered.len(), 3);
    // At the next end step they're exiled.
    t.advance_to_step(mtg_engine::turn::Step::End);
    t.resolve_all();
    assert!(t.in_exile("Holy Strength") && t.in_exile("Unholy Strength"));
}

#[test]
fn springheart_nantuko_copies_the_enchanted_creature_or_makes_an_insect() {
    cr!("702.103a", "111.10");
    assert_supported("Springheart Nantuko");
    // Attached to a creature P0 controls, and paid: a copy of that creature.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 4);
    let n = t.hand(P0, "Springheart Nantuko");
    t.cast(P0, n)
        .method(mtg_engine::object::CastMethod::Keyword(keywords::KeywordKind::Bestow))
        .target(Entity::Object(bears))
        .go();
    t.resolve();
    assert_eq!(t.pt(bears), (3, 3));
    t.answer_yes(P0, true);
    let land = t.hand(P0, "Forest");
    t.play_land(P0, land).unwrap();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 2);
    assert!(t.named_on_battlefield("Insect").is_empty());
    // Not attached: no payment is offered; an Insect token.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Springheart Nantuko");
    t.lands(P0, "Forest", 2);
    let land = t.hand(P0, "Forest");
    t.play_land(P0, land).unwrap();
    t.resolve_all();
    let insects: Vec<_> = t
        .g
        .battlefield
        .iter()
        .filter(|o| t.g.obj(**o).chars.has_subtype("Insect") && t.g.obj(**o).is_token())
        .collect();
    assert_eq!(insects.len(), 1);
}
