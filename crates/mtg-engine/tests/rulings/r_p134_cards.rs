//! Rulings batch P134 — the removal cards' own rules: Calix, Banishing Stroke, Nahiri,
//! Soltari Visionary, Divine Gambit, Cloudchaser Eagle, Green Slime, Kalemne's Captain,
//! Elesh Norn, Fall of the Impostor.

use crate::r_p113_common::cast_modes;
use crate::r_p134_common::*;
use crate::r_s01_common::{attack_with, block_and_finish, stack_library, supported};
use crate::r_s02_common::destroy;
use crate::r_s13_common::add;
use crate::r_s25_common::lands_for_cost;
use mtg_engine::ability::Modification;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::CardType;
use mtg_engine::*;

#[test]
fn calix_aura_cant_enchant_an_enchantment_creature_entering_with_it() {
    cr!("303.4f", "303.4g", "614.12");
    ruling!("Calix, Destiny's Hand", "If an Aura is being put onto the battlefield at the same time as an enchantment creature, that Aura can’t enchant that creature.");
    supported("Calix, Destiny's Hand");
    for other_creature in [false, true] {
        let mut t = TestGame::new(2);
        let c = t.battlefield(P0, "Calix, Destiny's Hand");
        add(&mut t, c, "loyalty", 3);
        t.graveyard(P0, "Nyx-Fleece Ram");
        t.graveyard(P0, "Holy Strength");
        let bears = other_creature.then(|| t.battlefield(P0, "Grizzly Bears"));
        t.activate(P0, c, 2, &[]).expect("-7");
        t.resolve_all();
        let ram = t.named_on_battlefield("Nyx-Fleece Ram");
        assert_eq!(ram.len(), 1);
        match bears {
            None => assert!(t.in_graveyard(P0, "Holy Strength"), "nothing to enchant"),
            Some(b) => {
                let hs = t.named_on_battlefield("Holy Strength");
                assert_eq!(hs.len(), 1);
                assert_eq!(t.obj_now(hs[0]).attached_to, Some(Entity::Object(b)));
            }
        }
    }
}

#[test]
fn banishing_stroke_isnt_drawn_when_put_into_hand_otherwise() {
    cr!("702.94a", "121.1");
    ruling!("Banishing Stroke", "If an effect puts a card into your hand without using the word “draw,” the card wasn’t drawn.");
    supported("Banishing Stroke");
    supported("Impulse");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Grizzly Bears");
    let lib = stack_library(
        &mut t,
        P0,
        &["Banishing Stroke", "Island", "Forest", "Swamp"],
    );
    t.lands(P0, "Plains", 1);
    t.answer_choose(P0, &[ob(lib[0])]);
    t.answer_yes(P0, true);
    cast_modes(&mut t, P0, "Impulse", &[], &[]);
    t.resolve_all();
    assert!(t.in_hand(P0, "Banishing Stroke"));
    assert_eq!(t.stack_len(), 0, "no miracle trigger");
    assert!(t.named_on_battlefield("Grizzly Bears").len() == 1);
}

#[test]
fn nahiri_abilities() {
    cr!("606.4", "611.2c", "610.3", "702.10b");
    ruling!("Nahiri, the Harbinger", "You can activate Nahiri’s first ability with no intention of discarding a card just to add loyalty to Nahiri.");
    ruling!("Nahiri, the Harbinger", "If the artifact or creature put onto the battlefield with Nahiri’s third ability leaves the battlefield before the next end step, it won’t be returned to your hand from its new zone.");
    ruling!("Nahiri, the Harbinger", "If you find a noncreature artifact with Nahiri’s third ability, it will still gain haste, even though it won’t be meaningful unless that artifact becomes a creature.");
    supported("Nahiri, the Harbinger");
    // +2 without discarding.
    let mut t = TestGame::new(2);
    let n = t.battlefield(P0, "Nahiri, the Harbinger");
    t.hand(P0, "Island");
    let l = t.counters(n, "loyalty");
    t.answer_yes(P0, false);
    t.activate(P0, n, 0, &[]).expect("+2");
    t.resolve_all();
    assert_eq!(t.counters(n, "loyalty"), l + 2);
    assert_eq!(t.hand_size(P0), 1);
    assert!(t.in_hand(P0, "Island"));
    // -8: a noncreature artifact gains haste; a creature that left isn't returned.
    for (card, leaves) in [
        ("Sol Ring", false),
        ("Grizzly Bears", true),
        ("Grizzly Bears", false),
    ] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let n = t.battlefield(P0, "Nahiri, the Harbinger");
        add(&mut t, n, "loyalty", 4);
        let c = t.library_top(P0, card);
        t.answer_choose(P0, &[ob(c)]);
        t.activate(P0, n, 2, &[]).expect("-8");
        t.resolve_all();
        let it = t.g.current(c);
        assert!(t.on_battlefield(it), "{card}");
        assert!(
            t.obj_now(it).chars.has_keyword(KeywordKind::Haste),
            "{card}: haste"
        );
        if leaves {
            destroy(&mut t, it);
        }
        t.advance_to(P0, Step::End);
        t.settle();
        t.resolve_all();
        if leaves {
            assert!(t.in_graveyard(P0, card), "{card}: stays in the graveyard");
        } else {
            assert!(t.in_hand(P0, card), "{card}: returned");
        }
    }
}

#[test]
fn soltari_visionary_destroys_an_enchantment_of_the_damaged_player() {
    cr!("603.3d", "115.1", "702.28b");
    ruling!("Soltari Visionary", "If the damaged player controls any enchantments, you must choose one to be destroyed. If they do not, nothing happens.");
    supported("Soltari Visionary");
    for theirs in [true, false] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let sv = t.battlefield(P0, "Soltari Visionary");
        let mine = t.battlefield(P0, "Bad Moon");
        let anthem = theirs.then(|| t.battlefield(P1, "Glorious Anthem"));
        attack_with(&mut t, &[(sv, Entity::Player(P1))]);
        t.advance_to(P0, Step::CombatDamage);
        t.settle();
        assert_eq!(t.life(P1), 18);
        assert_eq!(t.stack_len(), theirs as usize);
        t.resolve_all();
        assert!(t.on_battlefield(mine), "only the damaged player's");
        if let Some(a) = anthem {
            assert!(!t.on_battlefield(a));
        }
    }
}

#[test]
fn divine_gambit_double_faced_cards_enter_front_face_up_if_a_permanent() {
    cr!("712.14", "712.13", "110.1");
    ruling!("Divine Gambit", "If the player chooses to put a double-faced card onto the battlefield, it enters with its front face up. They may do this only if the front face of the card is a permanent card.");
    supported("Divine Gambit");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let delver = t.hand(P1, "Delver of Secrets // Insectile Aberration");
    let mdfc = t.hand(P1, "Bridgeworks Battle // Tanglespan Bridgeworks");
    t.answer_yes(P1, true);
    t.answer_choose(P1, &[ob(delver)]);
    let from = t.asked().len();
    cast_modes(&mut t, P0, "Divine Gambit", &[], &[ob(bears)]);
    t.resolve_all();
    let d = t.g.current(delver);
    assert!(t.on_battlefield(d));
    assert_eq!(t.obj_now(d).chars.name, "Delver of Secrets");
    // The modal double-faced card with a sorcery front face wasn't offered.
    for (_, dec) in &t.asked()[from..] {
        if let mtg_engine::decision::Decision::ChooseEntities { candidates, .. } = dec {
            assert!(!candidates.contains(&ob(mdfc)));
        }
    }
    assert_eq!(t.zone(mdfc), Zone::Hand(P1));
}

#[test]
fn cloudchaser_eagle_must_target_an_enchantment_even_yours() {
    cr!("603.3d", "115.1");
    ruling!(
        "Cloudchaser Eagle",
        "It destroys one of your enchantments if you are the only player with any enchantments."
    );
    ruling!(
        "Cloudchaser Eagle",
        "Nothing happens if there are no legal enchantments to target when it enters."
    );
    supported("Cloudchaser Eagle");
    let mut t = TestGame::new(2);
    let moon = t.battlefield(P0, "Bad Moon");
    t.answer_targets(P0, &[]);
    t.enter(P0, "Cloudchaser Eagle");
    t.resolve_all();
    assert!(!t.on_battlefield(moon));
    let mut t = TestGame::new(2);
    t.enter(P0, "Cloudchaser Eagle");
    t.settle();
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn green_slime_counters_triggered_abilities_not_mana_abilities() {
    cr!("603.1", "605.3a", "701.6b");
    ruling!("Green Slime", "Triggered abilities use the word “when,” “whenever,” or “at.” They're often written as “[Trigger condition], [effect].”");
    ruling!("Green Slime", "Mana abilities don't use the stack and can't be responded to, so they cannot be countered this way.");
    supported("Green Slime");
    // Ichor Wellspring's enters trigger is countered, and the Wellspring destroyed (its
    // "put into a graveyard" trigger then draws one card).
    let mut t = TestGame::new(2);
    t.enter(P1, "Ichor Wellspring");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    let trig = *t.g.stack.last().unwrap();
    lands_for_cost(&mut t, P0, "Green Slime");
    let gs = t.hand(P0, "Green Slime");
    t.answer_targets(P0, &[ob(trig)]);
    t.cast(P0, gs).go();
    t.resolve(); // Green Slime
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Ichor Wellspring"));
    assert_eq!(t.hand_size(P1), 1, "only the graveyard trigger drew");
    // Only mana abilities around: nothing to target.
    let mut t = TestGame::new(2);
    let ring = t.battlefield(P1, "Sol Ring");
    t.enter(P0, "Green Slime");
    t.settle();
    assert_eq!(t.stack_len(), 0);
    assert!(t.on_battlefield(ring));
}

#[test]
fn kalemnes_captain_monstrous_once_and_kept() {
    cr!("701.37b", "701.37c");
    ruling!("Kalemne's Captain", "Monstrous isn’t an ability that the creature has. It’s just something true about that creature. If the creature stops being a creature or loses its abilities, it continues to be monstrous.");
    ruling!("Kalemne's Captain", "Once a creature becomes monstrous, it can’t become monstrous again. If Kalemne’s Captain is already monstrous, the activated ability won’t do anything.");
    supported("Kalemne's Captain");
    let mut t = TestGame::new(2);
    let k = t.battlefield(P0, "Kalemne's Captain");
    let ring = t.battlefield(P1, "Sol Ring");
    t.lands(P0, "Plains", 7);
    t.activate(P0, k, 0, &[]).expect("monstrosity");
    t.resolve_all();
    assert!(t.obj_now(k).monstrous);
    assert_eq!(t.counters(k, "+1/+1"), 3);
    assert!(!t.on_battlefield(ring));
    // Again: nothing happens (no counters, no trigger).
    let ring2 = t.battlefield(P1, "Sol Ring");
    t.lands(P0, "Plains", 7);
    t.activate(P0, k, 0, &[]).expect("monstrosity again");
    t.resolve_all();
    assert_eq!(t.counters(k, "+1/+1"), 3);
    assert!(t.on_battlefield(ring2));
    // It loses its abilities and stops being a creature: still monstrous.
    modify(
        &mut t,
        P0,
        k,
        vec![
            Modification::RemoveAllAbilities,
            Modification::RemoveTypes(vec![CardType::Creature]),
        ],
    );
    assert!(t.obj_now(k).monstrous);
}

#[test]
fn elesh_norn_triggers_per_permanent_and_per_source() {
    cr!("603.2c", "120.3", "510.2");
    ruling!("Elesh Norn // The Argent Etchings", "Elesh Norn’s triggered ability will trigger separately for each permanent dealt damage by a source an opponent controls, plus once if you’re dealt damage by that source.");
    ruling!("Elesh Norn // The Argent Etchings", "Similarly, the triggered ability will trigger separately for each source an opponent controls that deals damage to you or a permanent you control.");
    supported("Elesh Norn // The Argent Etchings");
    supported("Volcanic Fallout");
    // One source, two permanents and the player: three triggers; P1 can't pay.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Elesh Norn // The Argent Etchings");
    t.battlefield(P0, "Hill Giant");
    cast_modes(&mut t, P1, "Volcanic Fallout", &[], &[]);
    t.resolve();
    assert_eq!(t.stack_len(), 3);
    t.resolve_all();
    assert_eq!(t.life(P1), 20 - 2 - 6);
    // Three blockers dealing damage to one attacker: three triggers.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Elesh Norn // The Argent Etchings");
    let wurm = t.battlefield(P0, "Craw Wurm");
    let bs: Vec<ObjectId> = (0..3).map(|_| t.battlefield(P1, "Grizzly Bears")).collect();
    attack_with(&mut t, &[(wurm, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[(bs[0], wurm), (bs[1], wurm), (bs[2], wurm)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 14);
}

/// Fall of the Impostor under P0's control with chapter III triggered, targeting P1.
fn impostor_iii(t: &mut TestGame) {
    supported("Fall of the Impostor");
    let s = t.battlefield(P0, "Fall of the Impostor");
    let n = t.counters(s, "lore");
    for _ in n..2 {
        t.answer_targets(P0, &[]);
    }
    t.answer_targets(P0, &[Entity::Player(P1)]);
    add(t, s, "lore", 3 - n);
    t.resolve_all();
}

#[test]
fn fall_of_the_impostor_chapter_three() {
    cr!("714.2b", "115.1", "702.16b");
    ruling!("Fall of the Impostor", "The chapter III ability doesn’t target any creatures. For example, a creature with protection from white could be exiled this way.");
    ruling!("Fall of the Impostor", "The greatest power among creatures target opponent controls is determined solely by evaluating the creatures that player controls at that time.");
    ruling!("Fall of the Impostor", "If the target player controls more than one creature tied for the greatest power among creatures that player controls, you choose which one of them is exiled.");
    // P1's only creature has protection from white; P2's bigger creature doesn't count.
    let mut t = TestGame::new(3);
    let bk = t.battlefield(P1, "Black Knight");
    let wurm = t.battlefield(P2, "Craw Wurm");
    impostor_iii(&mut t);
    assert!(!t.on_battlefield(bk));
    assert!(t.in_exile("Black Knight"));
    assert!(t.on_battlefield(wurm));
    // A tie: P0 chooses.
    let mut t = TestGame::new(2);
    let _g1 = t.battlefield(P1, "Hill Giant");
    let g2 = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_choose(P0, &[ob(g2)]);
    impostor_iii(&mut t);
    assert!(!t.on_battlefield(g2));
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
    assert!(t.on_battlefield(bears));
}
