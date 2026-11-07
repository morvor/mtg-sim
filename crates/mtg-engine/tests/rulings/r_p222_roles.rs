//! Rulings batch P222 — Role tokens (CR 111.10j–r, 303.7): Roles created attached to "up to
//! one target" creature are created only if a target was chosen and is still legal; and
//! Not Dead After All's Wicked Role.

use crate::r_s01_common::*;
use crate::r_s02_common::destroy;
use crate::r_s03_common::in_hand_with_mana;
use crate::r_s06_common::{activate_containing, attach_new};
use crate::r_s15_common::{role_names_on, roles_on};
use mtg_engine::card::card;
use mtg_engine::merge;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// All Role permanents on the battlefield.
fn all_roles(t: &TestGame) -> usize {
    t.g.permanents()
        .filter(|o| o.chars.has_subtype("Role"))
        .count()
}

/// `p` casts the real card `name` from hand, choosing `slots` as its targets (one answer
/// per target slot, empty for "up to one" with nothing chosen).
fn cast_slots(t: &mut TestGame, p: PlayerId, name: &str, slots: &[&[Entity]]) -> ObjectId {
    let c = in_hand_with_mana(t, p, name);
    for s in slots {
        t.answer_targets(p, s);
    }
    t.g.turn.priority = Some(p);
    t.cast(p, c).go()
}

#[test]
fn spells_whose_second_target_is_not_chosen_create_no_role() {
    cr!("601.2c", "115.1", "608.2b");
    ruling!("Cut In", "If you don't choose a second target for Cut In, the Young Hero Role token won't be created.");
    ruling!("Shatter the Oath", "If you don't choose a second target for Shatter the Oath or that target is illegal as the spell resolves, the Wicked Role token won't be created.");
    ruling!("Twisted Fealty", "If you don't choose a second target for Twisted Fealty or that target is illegal as the spell resolves, the Wicked Role token won't be created.");
    ruling!("Eriette's Whisper", "If you don't choose a target creature, the Wicked Role token won't be created.");
    for name in ["Cut In", "Shatter the Oath", "Twisted Fealty", "Eriette's Whisper"] {
        supported(name);
        // 0: no second target; 1: a second target that becomes illegal; 2: a legal one.
        for case in 0..3 {
            if case == 1 && matches!(name, "Cut In" | "Eriette's Whisper") {
                continue;
            }
            let mut t = TestGame::new(2);
            t.set_step(P0, Step::PrecombatMain);
            let first: Entity = if name == "Eriette's Whisper" {
                t.hand(P1, "Forest");
                t.hand(P1, "Forest");
                Entity::Player(P1)
            } else {
                t.battlefield(P1, "Hill Giant").into()
            };
            let mine = t.battlefield(P0, "Grizzly Bears");
            let second: &[Entity] = if case == 0 { &[] } else { &[Entity::Object(mine)] };
            cast_slots(&mut t, P0, name, &[&[first], second]);
            if case == 1 {
                destroy(&mut t, mine);
            }
            t.resolve_all();
            // The rest of the spell happens.
            match name {
                "Eriette's Whisper" => assert_eq!(t.hand_size(P1), 0, "{name}"),
                "Twisted Fealty" => {
                    let Entity::Object(g) = first else { unreachable!() };
                    assert_eq!(t.obj_now(g).controller, P0, "{name}");
                }
                _ => assert!(t.in_graveyard(P1, "Hill Giant"), "{name}"),
            }
            assert_eq!(all_roles(&t), (case == 2) as usize, "{name} case {case}");
            if case == 2 {
                assert_eq!(roles_on(&t, mine).len(), 1, "{name}");
            }
        }
    }
}

#[test]
fn abilities_with_no_target_chosen_create_no_role() {
    cr!("601.2c", "602.2b", "603.3d");
    ruling!("Living Lectern", "If you don't choose a target creature, the Sorcerer Role token won't be created.");
    ruling!("Protective Parents", "If you don't choose a target for Protective Parents's ability, the Young Hero Role token won't be created.");
    ruling!("Splashy Spellcaster", "If you don't choose a target for Splashy Spellcaster's ability, the Sorcerer Role token won't be created.");
    for name in ["Living Lectern", "Protective Parents", "Splashy Spellcaster"] {
        supported(name);
        for chosen in [false, true] {
            let mut t = TestGame::new(2);
            t.set_step(P0, Step::PrecombatMain);
            let src = t.battlefield(P0, name);
            let bears = t.battlefield(P0, "Grizzly Bears");
            let tgt: &[Entity] = if chosen { &[Entity::Object(bears)] } else { &[] };
            let hand = t.hand_size(P0);
            match name {
                "Living Lectern" => {
                    t.lands(P0, "Wastes", 1);
                    t.answer_targets(P0, tgt);
                    activate_containing(&mut t, P0, src, "Sacrifice").unwrap();
                }
                "Protective Parents" => {
                    t.answer_targets(P0, tgt);
                    destroy(&mut t, src);
                }
                _ => {
                    let opt = in_hand_with_mana(&mut t, P0, "Opt");
                    t.answer_targets(P0, tgt);
                    t.g.turn.priority = Some(P0);
                    t.cast(P0, opt).go();
                }
            }
            t.resolve_all();
            assert_eq!(all_roles(&t), chosen as usize, "{name} chosen {chosen}");
            if chosen {
                assert_eq!(roles_on(&t, bears).len(), 1, "{name}");
            }
            if name == "Living Lectern" {
                // It still draws a card.
                assert_eq!(t.hand_size(P0), hand + 1);
            }
        }
    }
}

#[test]
fn gadwicks_first_duel_chapter_one_without_a_target() {
    cr!("714.2b", "603.3d");
    ruling!("Gadwick's First Duel", "If you don't choose a target for the first chapter ability, the Cursed Role token won't be created.");
    supported("Gadwick's First Duel");
    for chosen in [false, true] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let giant = t.battlefield(P1, "Hill Giant");
        let tgt: &[Entity] = if chosen { &[Entity::Object(giant)] } else { &[] };
        t.answer_targets(P0, tgt);
        cast_slots(&mut t, P0, "Gadwick's First Duel", &[]);
        t.resolve_all();
        assert_eq!(all_roles(&t), chosen as usize);
        assert_eq!(t.pt(giant), if chosen { (1, 1) } else { (3, 3) });
    }
}

#[test]
fn giant_inheritance_without_an_attacking_target() {
    cr!("603.3d", "506.4");
    ruling!("Giant Inheritance", "If you don't choose a target attacking creature with the ability granted by Giant Inheritance, the Monster Role token won't be created.");
    supported("Giant Inheritance");
    for chosen in [false, true] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        attach_new(&mut t, P0, "Giant Inheritance", bears);
        let tgt: &[Entity] = if chosen { &[Entity::Object(bears)] } else { &[] };
        t.answer_targets(P0, tgt);
        t.set_step(P0, Step::BeginningOfCombat);
        attack_with(&mut t, &[(bears, Entity::Player(P1))]);
        t.resolve_all();
        assert_eq!(all_roles(&t), chosen as usize);
        if chosen {
            assert_eq!(role_names_on(&t, bears), vec!["Monster".to_string()]);
        }
    }
}

#[test]
fn witchs_mark_without_a_target_or_with_an_illegal_one() {
    cr!("608.2b", "601.2c");
    ruling!("Witch's Mark", "If you don't choose a target for Witch's Mark, the Wicked Role token won't be created.");
    ruling!("Witch's Mark", "You can cast Witch's Mark without a target just to discard a card and draw two cards. However, if you do choose a target, and that target is illegal at the time Witch's Mark tries to resolve, the spell won't resolve and none of its effects will happen. You won't discard, draw, or create a Wicked Role token.");
    supported("Witch's Mark");
    // 0: no target; 1: a target that becomes illegal; 2: a legal target.
    for case in 0..3 {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let bears = t.battlefield(P0, "Grizzly Bears");
        t.hand(P0, "Forest");
        let tgt: &[Entity] = if case == 0 { &[] } else { &[Entity::Object(bears)] };
        cast_slots(&mut t, P0, "Witch's Mark", &[tgt]);
        if case == 1 {
            destroy(&mut t, bears);
        }
        let hand = t.hand_size(P0);
        t.answer_yes(P0, true);
        t.resolve_all();
        if case == 1 {
            assert_eq!(t.hand_size(P0), hand);
            assert!(!t.in_graveyard(P0, "Forest"));
        } else {
            // Discarded the Forest, drew two.
            assert_eq!(t.hand_size(P0), hand + 1, "case {case}");
            assert!(t.in_graveyard(P0, "Forest"));
        }
        assert_eq!(all_roles(&t), (case == 2) as usize, "case {case}");
    }
}

#[test]
fn croaking_curse_on_an_already_tapped_creature_still_creates_the_role() {
    cr!("701.26a", "608.2c");
    ruling!("Vantress Transmuter // Croaking Curse", "You may target a creature that is already tapped with Croaking Curse. If the target creature is already tapped as it resolves, you will still create a Cursed Role token attached to it.");
    supported("Vantress Transmuter // Croaking Curse");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let giant = t.battlefield(P1, "Hill Giant");
    t.g.tap(giant);
    let c = t.hand(P0, "Vantress Transmuter // Croaking Curse");
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 1);
    t.cast(P0, c).method(CastMethod::Half(1)).target(giant).go();
    t.resolve_all();
    assert!(t.obj(giant).tapped);
    assert_eq!(role_names_on(&t, giant), vec!["Cursed".to_string()]);
    assert_eq!(t.pt(giant), (1, 1));
}

#[test]
fn not_dead_after_all_needs_a_returned_creature() {
    cr!("603.10a", "608.2c", "303.4i");
    ruling!("Not Dead After All", "If the creature doesn't return to the battlefield or returns as a noncreature permanent, the Wicked Role token won't be created.");
    supported("Not Dead After All");
    // 0: it returns as a creature; 1: it's exiled from the graveyard first; 2: a crewed
    // Vehicle returns as a noncreature artifact.
    for case in 0..3 {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let c = if case == 2 {
            let copter = t.battlefield(P0, "Smuggler's Copter");
            let bears = t.battlefield(P0, "Grizzly Bears");
            t.answer_choose(P0, &[bears.into()]);
            activate_containing(&mut t, P0, copter, "Crew").unwrap();
            t.resolve_all();
            assert!(t.obj(copter).is_creature());
            copter
        } else {
            t.battlefield(P0, "Hill Giant")
        };
        cast_slots(&mut t, P0, "Not Dead After All", &[&[c.into()]]);
        t.resolve_all();
        destroy(&mut t, c);
        assert_eq!(t.stack_len(), 1, "case {case}");
        if case == 1 {
            let card = t.g.current(c);
            t.g.exile_object(card, None);
        }
        t.resolve_all();
        let now = t.g.current(c);
        match case {
            0 => {
                assert!(t.on_battlefield(now));
                assert_eq!(role_names_on(&t, now), vec!["Wicked".to_string()]);
                assert_eq!(t.pt(now), (4, 4));
            }
            1 => assert_eq!(t.zone(now), Zone::Exile),
            _ => {
                assert!(t.on_battlefield(now));
                assert!(!t.obj(now).is_creature());
            }
        }
        assert_eq!(all_roles(&t), (case == 0) as usize, "case {case}");
    }
}

#[test]
fn not_dead_after_all_on_a_melded_permanent_creates_a_role_for_each_card() {
    cr!("712.4", "603.10a", "111.10m");
    ruling!("Not Dead After All", "If multiple creatures return to the battlefield (possibly because Not Dead After All targeted a melded permanent), a Wicked Role token will be created attached to each of them.");
    supported("Not Dead After All");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let rats = t.exile(P0, "Graf Rats");
    let scav = t.exile(P0, "Midnight Scavengers");
    let host = merge::meld(&mut t.g, rats, scav, card("Chittering Host"), P0).expect("meld");
    t.g.recompute();
    cast_slots(&mut t, P0, "Not Dead After All", &[&[host.into()]]);
    t.resolve_all();
    destroy(&mut t, host);
    t.resolve_all();
    let back: Vec<ObjectId> = ["Graf Rats", "Midnight Scavengers"]
        .iter()
        .flat_map(|n| t.named_on_battlefield(n))
        .collect();
    assert_eq!(back.len(), 2);
    for c in back {
        assert_eq!(role_names_on(&t, c), vec!["Wicked".to_string()]);
    }
    assert_eq!(all_roles(&t), 2);
}
