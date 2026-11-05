//! Rulings batch P178 — "whenever a player casts ..." triggers: they go on the stack above
//! the spell and resolve first, even if the spell is countered (the Horns, Kor
//! Firewalker, Managorger Hydra, Nezahal, Lurking Predators, Silverquill Silencer, Mold
//! Adder, Mona Lisa, Taurean Mauler, April O'Neil), whose spells count (Soldier of the
//! Pantheon, Witchstalker), multiplayer opponents (Warmth, Powerleech, Citanul Druid),
//! and Runic Armasaur's activation trigger.

use crate::r_p178_common::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// The spell `caster` casts in a case: Opt, Dark Ritual, Grizzly Bears, or Lightning
/// Bolt at the other player. Returns it, with the triggers put on the stack.
fn cast_spell(t: &mut TestGame, caster: PlayerId, name: &str) -> ObjectId {
    let other = if caster == P0 { P1 } else { P0 };
    let targets: Vec<Entity> = if name == "Lightning Bolt" {
        vec![Entity::Player(other)]
    } else {
        vec![]
    };
    t.set_step(caster, Step::PrecombatMain);
    let spell = cast_targeting(t, caster, name, &targets);
    t.settle();
    spell
}

#[test]
fn cast_triggers_resolve_before_the_spell() {
    cr!("603.3", "405.5", "117.3b");
    ruling!(
        "Demon's Horn",
        "If a player casts a black spell, Demon’s Horn’s ability triggers and is put on the stack on top of that spell. Demon’s Horn’s ability will resolve (causing you to gain 1 life) before the spell does."
    );
    ruling!(
        "Kraken's Eye",
        "If a player casts a blue spell, Kraken’s Eye’s ability triggers and is put on the stack on top of that spell. Kraken’s Eye’s ability will resolve (causing you to gain 1 life) before the spell does."
    );
    ruling!(
        "Dragon's Claw",
        "If a player casts a red spell, Dragon’s Claw’s ability triggers and is put on the stack on top of that spell. Dragon’s Claw’s ability will resolve (causing you to gain 1 life) before the spell does."
    );
    ruling!(
        "Demon's Horn",
        "The ability triggers whenever any player, not just you, casts a black spell."
    );
    ruling!(
        "Kraken's Eye",
        "The ability triggers whenever any player, not just you, casts a blue spell."
    );
    ruling!(
        "Dragon's Claw",
        "The ability triggers whenever any player, not just you, casts a red spell."
    );
    ruling!(
        "Kor Firewalker",
        "Kor Firewalker's triggered ability resolves before the red spell that caused it to trigger."
    );
    ruling!(
        "Managorger Hydra",
        "Managorger Hydra's last ability will resolve before the spell that caused it to trigger."
    );
    ruling!(
        "Nezahal, Primal Tide",
        "Nezahal's triggered ability resolves before the noncreature spell that caused it to trigger."
    );
    ruling!(
        "Lurking Predators",
        "If an opponent casts a spell, this ability triggers and is put on the stack on top of that spell. This ability will resolve (meaning that a revealed creature card will enter) before the spell resolves."
    );
    // (permanent P0 controls, casters, spell, what the trigger did)
    type Check = fn(&TestGame, ObjectId) -> bool;
    let gained: Check = |t, _| t.life(P0) == 21;
    let cases: Vec<(&str, Vec<PlayerId>, &str, Check)> = vec![
        ("Demon's Horn", vec![P0, P1], "Dark Ritual", gained),
        ("Kraken's Eye", vec![P0, P1], "Opt", gained),
        ("Dragon's Claw", vec![P0, P1], "Lightning Bolt", |t, _| {
            // P0 gained 1 before any Bolt resolved.
            t.life(P0) == 21
        }),
        ("Kor Firewalker", vec![P0, P1], "Lightning Bolt", gained),
        ("Managorger Hydra", vec![P0, P1], "Opt", |t, src| {
            t.counters(src, "+1/+1") == 1
        }),
        ("Nezahal, Primal Tide", vec![P1], "Opt", |t, _| {
            t.hand_size(P0) == 1
        }),
        ("Lurking Predators", vec![P1], "Opt", |t, _| {
            !t.g.find_in_zone(Zone::Battlefield, "Grizzly Bears")
                .is_empty()
        }),
    ];
    for (perm, casters, spell, check) in cases {
        for caster in casters {
            supported(perm);
            let mut t = TestGame::new(2);
            let src = t.battlefield(P0, perm);
            t.library_top(P0, "Grizzly Bears");
            t.answer_yes(P0, true);
            let s = cast_spell(&mut t, caster, spell);
            assert_eq!(triggers_from(&t, src), 1, "{perm}: {caster} casts {spell}");
            assert_eq!(
                *t.g.stack.first().unwrap(),
                s,
                "{perm}: the trigger is above"
            );
            t.resolve();
            assert_eq!(t.zone(s), Zone::Stack, "{perm}: the spell hasn't resolved");
            assert!(check(&t, src), "{perm}: {caster} casts {spell}");
        }
    }
}

#[test]
fn cast_triggers_resolve_even_if_the_spell_is_countered() {
    cr!("603.3", "701.6a");
    ruling!(
        "Mold Adder",
        "Mold Adder's ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered or otherwise leaves the stack without resolving."
    );
    ruling!(
        "Mona Lisa, Ever Adaptable",
        "Mona Lisa's ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered or otherwise leaves the stack."
    );
    ruling!(
        "Taurean Mauler",
        "Taurean Mauler's last ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered or otherwise leaves the stack without resolving."
    );
    ruling!(
        "April O'Neil, Human Element",
        "April's ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered or otherwise leaves the stack."
    );
    // (The check gets whether the spell was countered by P0's Counterspell, an instant
    // that April O'Neil also triggers on.)
    type Check = fn(&TestGame, ObjectId, bool) -> bool;
    let counter: Check = |t, src, _| t.counters(src, "+1/+1") == 1;
    let mutagen: Check = |t, _, _| tokens(t, P0).len() == 1;
    let april: Check = |t, _, countered| tokens(t, P0).len() == if countered { 2 } else { 1 };
    let cases: Vec<(&str, &str, Check)> = vec![
        ("Mold Adder", "Opt", counter),
        ("Taurean Mauler", "Opt", counter),
        ("Mona Lisa, Ever Adaptable", "Grizzly Bears", mutagen),
        ("April O'Neil, Human Element", "Opt", april),
    ];
    for (perm, spell, check) in cases {
        for counter_first in [false, true] {
            supported(perm);
            let mut t = TestGame::new(2);
            let src = t.battlefield(P0, perm);
            t.answer_yes(P0, true);
            let s = cast_spell(&mut t, P1, spell);
            assert_eq!(triggers_from(&t, src), 1, "{perm}");
            if counter_first {
                // Countered in response to the trigger.
                // (April O'Neil also triggers on Counterspell.)
                cast_targeting(&mut t, P0, "Counterspell", &[obj(s)]);
                t.resolve_all();
                assert!(t.in_graveyard(P1, spell), "{perm}: countered");
            } else {
                t.resolve();
                assert_eq!(t.zone(s), Zone::Stack, "{perm}: resolved first");
            }
            assert!(check(&t, src, counter_first), "{perm}");
        }
    }
}

#[test]
fn nezahal_you_may_respond_with_the_card_you_drew() {
    cr!("117.3b", "603.3");
    ruling!(
        "Nezahal, Primal Tide",
        "Players can cast spells and activate abilities after Nezahal's triggered ability resolves but before the spell that caused it to trigger does. Notably, the card you draw may be able to counter that spell"
    );
    supported("Nezahal, Primal Tide");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Nezahal, Primal Tide");
    let cs = t.library_top(P0, "Counterspell");
    let opt = cast_spell(&mut t, P1, "Opt");
    t.resolve();
    assert_eq!(t.zone(cs), Zone::Hand(P0), "drew Counterspell");
    t.lands(P0, "Island", 2);
    t.answer_targets(P0, &[obj(opt)]);
    t.cast(P0, t.g.current(cs)).go();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Opt"), "countered with the card drawn");
}

#[test]
fn silverquill_silencer_trigger_resolves_before_the_named_spell() {
    cr!("603.3");
    ruling!(
        "Silverquill Silencer",
        "Silverquill Silencer’s second triggered ability resolves before the spell that caused it to trigger."
    );
    supported("Silverquill Silencer");
    let mut t = TestGame::new(2);
    t.answer(
        P0,
        DecisionKind::Name,
        mtg_engine::decision::Answer::Text("Opt".into()),
    );
    let silencer = t.enter(P0, "Silverquill Silencer");
    t.resolve_all();
    let opt = cast_spell(&mut t, P1, "Opt");
    assert_eq!(triggers_from(&t, silencer), 1);
    let hand = t.hand_size(P0);
    t.resolve();
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.zone(opt), Zone::Stack);
}

#[test]
fn mold_adder_triggers_once_for_a_blue_and_black_spell() {
    cr!("603.2c", "105.2b");
    ruling!(
        "Mold Adder",
        "If an opponent casts a spell that's both blue and black, Mold Adder's ability triggers only once."
    );
    let mut t = TestGame::new(2);
    let adder = t.battlefield(P0, "Mold Adder");
    let land = t.battlefield(P1, "Plains");
    t.answer_yes(P0, true);
    cast_targeting(&mut t, P1, "Recoil", &[obj(land)]);
    t.settle();
    assert_eq!(triggers_from(&t, adder), 1);
    t.resolve_all();
    assert_eq!(t.counters(adder, "+1/+1"), 1);
}

#[test]
fn soldier_of_the_pantheon_hybrid_spells_are_multicolored() {
    cr!("105.2b", "202.2");
    ruling!(
        "Soldier of the Pantheon",
        "Most hybrid spells and permanents, including the ones in the Return to Ravnica block, are multicolored, even if you cast them with one color of mana."
    );
    supported("Soldier of the Pantheon");
    let mut t = TestGame::new(2);
    let soldier = t.battlefield(P0, "Soldier of the Pantheon");
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Forest", 3);
    let finks = t.hand(P1, "Kitchen Finks");
    t.cast(P1, finks).go();
    t.settle();
    assert_eq!(triggers_from(&t, soldier), 1, "paid with green only");
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
}

#[test]
fn soldier_of_the_pantheon_protection_from_multicolored() {
    cr!("702.16b", "702.16c", "702.16e", "702.16f");
    ruling!(
        "Soldier of the Pantheon",
        "Soldier of the Pantheon can't be enchanted or equipped by multicolored Auras and Equipment, it can't be blocked by multicolored creatures, it can't be targeted by multicolored spells or abilities from multicolored sources, and all damage dealt to it by multicolored sources is prevented."
    );
    // Can't be targeted by a multicolored spell.
    let mut t = TestGame::new(2);
    let soldier = t.battlefield(P0, "Soldier of the Pantheon");
    let from = t.asked().len();
    give_mana_for(&mut t, P1, "Lightning Helix");
    let helix = t.hand(P1, "Lightning Helix");
    t.answer_targets(P1, &[obj(soldier)]);
    let _ = t.cast_with(P1, helix, &[]);
    let candidates = crate::r_s02_common::target_candidates(&t, P1, from);
    assert!(
        candidates.iter().flatten().any(|e| *e == Entity::Player(P0)),
        "P1 was asked for the Helix's target"
    );
    assert!(!candidates.iter().flatten().any(|e| *e == obj(soldier)));
    assert!(t.on_battlefield(soldier));

    // Can't be blocked by a multicolored creature; damage from one is prevented.
    let mut t = TestGame::new(2);
    let soldier = t.battlefield(P0, "Soldier of the Pantheon");
    let finks = t.battlefield(P1, "Kitchen Finks");
    t.set_step(P0, Step::BeginningOfCombat);
    attack_with(&mut t, &[(soldier, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[(finks, soldier)]);
    assert_eq!(t.life(P1), 18, "unblocked");
    // (A fresh game: the Soldier is still tapped from attacking.)
    let mut t = TestGame::new(2);
    let soldier = t.battlefield(P0, "Soldier of the Pantheon");
    let finks = t.battlefield(P1, "Kitchen Finks");
    t.set_step(P1, Step::BeginningOfCombat);
    attack_with(&mut t, &[(finks, Entity::Player(P0))]);
    block_and_finish(&mut t, P0, &[(soldier, finks)]);
    assert_eq!(t.life(P0), 20, "the Soldier blocked the Finks");
    assert!(t.on_battlefield(soldier), "the Finks' damage was prevented");
    assert_eq!(t.obj_now(soldier).damage, 0);

    // Can't be enchanted by a multicolored Aura.
    let mut t = TestGame::new(2);
    let soldier = t.battlefield(P0, "Soldier of the Pantheon");
    let mono = t.battlefield(P1, "Spirit Link");
    let multi = t.battlefield(P1, "Armadillo Cloak");
    assert!(mtg_engine::attach::can_attach(&t.g, mono, obj(soldier)));
    assert!(!mtg_engine::attach::can_attach(&t.g, multi, obj(soldier)));
}

#[test]
fn warmth_powerleech_and_citanul_druid_trigger_for_every_opponent() {
    cr!("102.3");
    ruling!("Warmth", "It triggers on all opposing players.");
    ruling!(
        "Powerleech",
        "In a multi-player game it affects all opponents."
    );
    ruling!(
        "Citanul Druid",
        "In multiplayer games it affects all players."
    );
    // Warmth: each opponent's red spell.
    let mut t = ffa(3);
    supported("Warmth");
    t.battlefield(P0, "Warmth");
    for p in [P1, P2] {
        t.set_step(p, Step::PrecombatMain);
        cast_targeting(&mut t, p, "Lightning Bolt", &[Entity::Player(P0)]);
        t.resolve_all();
    }
    assert_eq!(t.life(P0), 20 - 6 + 4);
    // Citanul Druid: each opponent's artifact spell.
    let mut t = ffa(3);
    supported("Citanul Druid");
    let druid = t.battlefield(P0, "Citanul Druid");
    for p in [P1, P2] {
        t.set_step(p, Step::PrecombatMain);
        cast_targeting(&mut t, p, "Ornithopter", &[]);
        t.resolve_all();
    }
    assert_eq!(t.counters(druid, "+1/+1"), 2);
    // Powerleech: each opponent's artifact becoming tapped.
    let mut t = ffa(3);
    supported("Powerleech");
    t.battlefield(P0, "Powerleech");
    for p in [P1, P2] {
        let ring = t.battlefield(p, "Sol Ring");
        t.set_step(p, Step::PrecombatMain);
        t.activate(p, ring, 0, &[]).unwrap();
        t.resolve_all();
    }
    assert_eq!(t.life(P0), 22);
}

#[test]
fn runic_armasaur_ignores_abilities_of_cards_and_of_noncreature_permanents() {
    cr!("602.1", "603.2");
    ruling!(
        "Runic Armasaur",
        "Runic Armasaur's ability doesn't trigger when an opponent activates an ability of a card in hand (such as a cycling ability from the Amonkhet block) or a card in a graveyard"
    );
    ruling!(
        "Runic Armasaur",
        "Runic Armasaur's ability doesn't trigger when an opponent activates an ability of a noncreature, nonland permanent that causes it to become a creature (such as a crew ability of a Vehicle)."
    );
    supported("Runic Armasaur");
    let mut t = TestGame::new(2);
    let saur = t.battlefield(P0, "Runic Armasaur");
    t.set_step(P1, Step::PrecombatMain);
    // Cycling from hand.
    let cycler = t.hand(P1, "Desert Cerodon");
    t.lands(P1, "Wastes", 1);
    t.lands(P1, "Mountain", 1);
    activate_containing(&mut t, P1, cycler, "Cycling").unwrap();
    assert_eq!(triggers_from(&t, saur), 0, "cycling");
    t.resolve_all();
    // A creature card's ability in a graveyard.
    let skel = t.graveyard(P1, "Reassembling Skeleton");
    t.lands(P1, "Swamp", 2);
    activate_containing(&mut t, P1, skel, "Return").unwrap();
    assert_eq!(triggers_from(&t, saur), 0, "graveyard");
    t.resolve_all();
    assert!(t.on_battlefield(skel), "it returned");
    // Crewing a Vehicle.
    let vehicle = t.battlefield(P1, "Smuggler's Copter");
    let crew = t.battlefield(P1, "Grizzly Bears");
    assert!(crate::r_s04_common::crew(&mut t, P1, vehicle, &[crew]));
    assert_eq!(triggers_from(&t, saur), 0, "crew");
}

#[test]
fn runic_armasaur_resolves_first_and_players_may_act_in_between() {
    cr!("603.3", "117.3b");
    ruling!(
        "Runic Armasaur",
        "Runic Armasaur's ability resolves before the ability that caused it to trigger. Players can cast spells and activate abilities after the triggered ability resolves but before the activated ability that caused it to trigger does."
    );
    let mut t = TestGame::new(2);
    let saur = t.battlefield(P0, "Runic Armasaur");
    let pyro = t.battlefield(P1, "Prodigal Pyromancer");
    t.set_step(P1, Step::PrecombatMain);
    t.activate(P1, pyro, 0, &[Entity::Player(P0)]).unwrap();
    t.settle();
    assert_eq!(triggers_from(&t, saur), 1);
    t.answer_yes(P0, true);
    let hand = t.hand_size(P0);
    t.resolve();
    assert_eq!(t.hand_size(P0), hand + 1, "drew first");
    assert_eq!(
        t.stack_len(),
        1,
        "the Pyromancer's ability is still on the stack"
    );
    assert_eq!(t.life(P0), 20);
    // P0 responds now, before the Pyromancer's ability resolves.
    cast_targeting(&mut t, P0, "Lightning Bolt", &[obj(pyro)]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Prodigal Pyromancer"));
    assert_eq!(t.life(P0), 19, "its ability still resolved");
}
