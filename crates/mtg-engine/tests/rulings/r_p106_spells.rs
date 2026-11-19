//! Rulings batch P106 — spells and triggered abilities of the batch's cards: cast
//! triggers that outlive their spell, "until this leaves the battlefield" exiles, spells
//! and abilities whose targets become illegal, decayed tokens, searching for land types,
//! copies of tokens with {X}, and cascade into a card with two faces (CR 603.3, 610.3c,
//! 608.2b, 702.147, 205.3i, 707.2, 702.85).

use crate::r_p106_common::*;
use crate::r_s01_common::attack_with;
use crate::r_s06_common::attach_new;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// P1 counters the top spell of the stack with Counterspell.
fn p1_counters(t: &mut TestGame, spell: ObjectId) {
    mana(t, P1, ManaType::U, 2);
    let cs = t.hand(P1, "Counterspell");
    t.cast_with(P1, cs, &[obj(spell)]).unwrap();
}

/// The Eldrazi Spawn tokens P0 controls.
fn spawns(t: &TestGame) -> usize {
    crate::r_p108_common::tokens_with(t, P0, "Spawn")
}

#[test]
fn cast_triggers_resolve_even_if_the_spell_is_countered() {
    cr!("603.3", "601.2i", "603.4");
    ruling!(
        "Drowner of Truth // Drowned Jungle",
        "Drowner of Truth's triggered ability will resolve before Drowner of Truth does. If Drowner of Truth is countered or otherwise leaves the stack in response to that triggered ability, the triggered ability will still resolve as normal."
    );
    ruling!(
        "Eldrazi Repurposer",
        "Eldrazi Repurposer's triggered ability will resolve before Eldrazi Repurposer does. If Eldrazi Repurposer is countered or otherwise leaves the stack in response to that triggered ability, the triggered ability will still resolve as normal."
    );
    // Drowner of Truth paid with {C}: two Spawn although it's countered.
    let mut t = TestGame::new(2);
    mana(&mut t, P0, ManaType::C, 5);
    mana(&mut t, P0, ManaType::G, 2);
    let d = cast_from_hand(&mut t, P0, "Drowner of Truth // Drowned Jungle", &[]);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    p1_counters(&mut t, d);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Drowner of Truth"));
    assert_eq!(spawns(&t), 2);
    // Eldrazi Repurposer: one Spawn although it's countered.
    let mut t = TestGame::new(2);
    mana(&mut t, P0, ManaType::C, 2);
    mana(&mut t, P0, ManaType::G, 1);
    let r = cast_from_hand(&mut t, P0, "Eldrazi Repurposer", &[]);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    p1_counters(&mut t, r);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Eldrazi Repurposer"));
    assert_eq!(spawns(&t), 1);
}

#[test]
fn sowing_mycospawns_cast_triggers_resolve_even_if_it_is_countered() {
    cr!("603.3", "601.2i", "603.4");
    ruling!(
        "Sowing Mycospawn",
        "Sowing Mycospawn's triggered abilities will resolve before Sowing Mycospawn does. If Sowing Mycospawn is countered or otherwise leaves the stack in response to those triggered abilities, the triggered abilities will still resolve as normal."
    );
    let mut t = TestGame::new(2);
    let forest = t.library_top(P0, "Forest");
    let island = t.battlefield(P1, "Island");
    mana(&mut t, P0, ManaType::C, 5);
    mana(&mut t, P0, ManaType::G, 1);
    t.answer_targets(P0, &[obj(island)]);
    t.answer_choose(P0, &[obj(forest)]);
    let card = t.hand(P0, "Sowing Mycospawn");
    let s = t.cast(P0, card).kicked(true).go();
    assert_eq!(pool_total(&t, P0), 0);
    t.settle();
    assert_eq!(t.stack_len(), 3);
    p1_counters(&mut t, s);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Sowing Mycospawn"));
    assert_eq!(controlled(&t, P0, "Forest").len(), 1);
    assert!(t.in_exile("Island"));
}

#[test]
fn until_it_leaves_exiles_do_nothing_if_the_source_is_already_gone() {
    cr!("610.3c", "603.4");
    ruling!(
        "Buried in the Garden",
        "If Buried in the Garden leaves the battlefield before its first triggered ability resolves, the target permanent won't be exiled."
    );
    ruling!(
        "White Auracite",
        "If White Auracite leaves the battlefield before its triggered ability resolves, the target permanent won't be exiled."
    );
    // Buried in the Garden, cast on P0's Forest, aims its trigger at P1's Bears.
    let mut t = TestGame::new(2);
    let forest = t.battlefield(P0, "Forest");
    let bears = t.battlefield(P1, "Grizzly Bears");
    mana(&mut t, P0, ManaType::G, 1);
    mana(&mut t, P0, ManaType::W, 1);
    mana(&mut t, P0, ManaType::C, 2);
    let aura = cast_from_hand(&mut t, P0, "Buried in the Garden", &[obj(forest)]);
    t.answer_targets(P0, &[obj(bears)]);
    t.resolve();
    assert_eq!(t.stack_len(), 1, "the enters trigger");
    destroy(&mut t, aura);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    // White Auracite.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    let wa = t.enter(P0, "White Auracite");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    destroy(&mut t, wa);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    // Control: with White Auracite still there, the Bears are exiled.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    t.enter(P0, "White Auracite");
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
}

#[test]
fn prismari_command_with_some_or_all_targets_illegal() {
    cr!("608.2b");
    ruling!(
        "Prismari Command",
        "If at least one target is still legal as Prismari Command tries to resolve, it will resolve and do as much as it can."
    );
    ruling!(
        "Prismari Command",
        "If all of Prismari Command's targets are illegal as it tries to resolve, it will do nothing."
    );
    // 2 damage to the Bears and a Treasure for P0; the Bears are gone: P0 still gets the
    // Treasure.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    mana(&mut t, P0, ManaType::U, 1);
    mana(&mut t, P0, ManaType::R, 1);
    mana(&mut t, P0, ManaType::C, 1);
    let cmd = t.hand(P0, "Prismari Command");
    let s = t
        .cast(P0, cmd)
        .modes(&[0, 2])
        .targets(&[obj(bears)])
        .targets(&[Entity::Player(P0)])
        .go();
    destroy(&mut t, bears);
    t.resolve_all();
    assert!(resolved(&t, s));
    assert_eq!(crate::r_p108_common::tokens_with(&t, P0, "Treasure"), 1);
    // Damage to the Bears and destroy Sol Ring; both gone: it doesn't resolve.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let ring = t.battlefield(P1, "Sol Ring");
    mana(&mut t, P0, ManaType::U, 1);
    mana(&mut t, P0, ManaType::R, 1);
    mana(&mut t, P0, ManaType::C, 1);
    let cmd = t.hand(P0, "Prismari Command");
    let s = t
        .cast(P0, cmd)
        .modes(&[0, 3])
        .targets(&[obj(bears)])
        .targets(&[obj(ring)])
        .go();
    destroy(&mut t, bears);
    crate::r_s05_common::move_to(&mut t, ring, Zone::Hand(P1));
    t.resolve_all();
    assert!(!resolved(&t, s));
    assert!(t.in_graveyard(P0, "Prismari Command"));
    assert!(t.in_hand(P1, "Sol Ring"));
}

#[test]
fn the_grand_evolution_loses_the_counters_of_an_illegal_target() {
    cr!("601.2d", "608.2b", "714.3b");
    ruling!(
        "Vorinclex // The Grand Evolution",
        "If some of the creatures are illegal targets as the chapter II ability tries to resolve, the original distribution of counters still applies and the counters that would have been put on the illegal targets are lost. They won’t be put instead on a legal target."
    );
    let mut t = TestGame::new(2);
    let saga = crate::r_s17_common::enter_transformed(&mut t, P0, "Vorinclex // The Grand Evolution");
    t.resolve_all();
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    t.answer_targets(P0, &[obj(a), obj(b)]);
    crate::r_s29_common::divide(&mut t, P0, &[5, 2]);
    crate::r_s19_common::add_lore(&mut t, saga, 1);
    assert_eq!(t.stack_len(), 1);
    destroy(&mut t, b);
    t.resolve_all();
    assert_eq!(t.counters(a, counters::PLUS1), 5);
}

#[test]
fn kicked_hunting_wilds_forests_stay_creatures() {
    cr!("611.2a", "702.33d");
    ruling!(
        "Hunting Wilds",
        "If the Hunting Wilds is kicked, the Forests become creatures with haste permanently."
    );
    let mut t = TestGame::new(2);
    let lib = stack_library(&mut t, P0, &["Forest", "Forest"]);
    mana(&mut t, P0, ManaType::G, 2);
    mana(&mut t, P0, ManaType::C, 6);
    t.answer_choose(P0, &[obj(lib[0]), obj(lib[1])]);
    let hw = t.hand(P0, "Hunting Wilds");
    t.cast(P0, hw).kicked(true).go();
    t.resolve_all();
    let forests = controlled(&t, P0, "Forest");
    assert_eq!(forests.len(), 2);
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    for f in forests {
        assert!(is_creature(&t, f));
        assert_eq!(t.pt(f), (3, 3));
        assert!(t.obj_now(f).has_keyword(KeywordKind::Haste));
        assert!(t.obj_now(f).is(CardType::Land));
    }
}

#[test]
fn copies_of_a_token_with_x_in_its_mana_cost_have_x_zero() {
    cr!("707.2", "107.3b", "202.3e");
    ruling!(
        "City of Death",
        "If the copied token has {X} in its mana cost, X is considered to be 0."
    );
    ruling!(
        "Romana II",
        "If the copied token has {X} in its mana cost, X is considered to be 0."
    );
    // A token copy of Chimeric Mass ({X}: "This artifact enters with X charge counters").
    let setup = |t: &mut TestGame| -> ObjectId {
        mana(t, P0, ManaType::C, 3);
        let mass = t.hand(P0, "Chimeric Mass");
        t.cast(P0, mass).x(3).go();
        t.resolve_all();
        let mass = t.named_on_battlefield("Chimeric Mass")[0];
        assert_eq!(t.counters(mass, counters::CHARGE), 3);
        crate::r_s17_common::token_copy(t, P0, mass)[0]
    };
    let new_copy = |t: &TestGame, before: &[ObjectId]| -> ObjectId {
        tokens(t, P0)
            .into_iter()
            .find(|id| !before.contains(id) && t.obj(*id).chars.name == "Chimeric Mass")
            .expect("a new copy")
    };
    // City of Death's chapter II.
    let mut t = TestGame::new(2);
    let city = t.enter(P0, "City of Death");
    t.resolve_all();
    let tok = setup(&mut t);
    let before = tokens(&t, P0);
    t.answer_targets(P0, &[obj(tok)]);
    crate::r_s19_common::add_lore(&mut t, city, 1);
    t.resolve_all();
    let c = new_copy(&t, &before);
    t.g.recompute();
    assert_eq!(t.g.mana_value_of(c), 0);
    assert_eq!(t.counters(c, counters::CHARGE), 0);
    // Romana II.
    let mut t = TestGame::new(2);
    let romana = t.battlefield(P0, "Romana II");
    let tok = setup(&mut t);
    let before = tokens(&t, P0);
    mana(&mut t, P0, ManaType::C, 1);
    act(&mut t, P0, romana, "Create a tapped token", &[obj(tok)]).unwrap();
    t.resolve_all();
    let c = new_copy(&t, &before);
    t.g.recompute();
    assert_eq!(t.g.mana_value_of(c), 0);
    assert_eq!(t.counters(c, counters::CHARGE), 0);
}

#[test]
fn single_target_spells_with_an_illegal_target_do_nothing() {
    cr!("608.2b", "303.4f");
    ruling!(
        "Gild",
        "If the creature isn't a legal target as Gild tries to resolve, the spell won't resolve and none of its effects will happen. No token will be created."
    );
    ruling!(
        "Twitch",
        "If the targeted permanent is an illegal target by the time Twitch resolves, the entire spell doesn’t resolve. You don’t draw a card."
    );
    ruling!(
        "Verdant Haven",
        "If the land targeted by Verdant Haven is an illegal target when Verdan Haven tries to resolve, Verdant Haven won't resolve. It won't enter the battlefield and its enters-the-battlefield ability won't trigger."
    );
    ruling!(
        "Gift of Paradise",
        "If the target land is an illegal target by the time Gift of Paradise tries to resolve, it doesn't resolve. It won't enter the battlefield, so its enters-the-battlefield ability won't trigger."
    );
    // Gild: no Gold token.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    mana(&mut t, P0, ManaType::B, 1);
    mana(&mut t, P0, ManaType::C, 3);
    let s = cast_from_hand(&mut t, P0, "Gild", &[obj(bears)]);
    destroy(&mut t, bears);
    t.resolve_all();
    assert!(!resolved(&t, s));
    assert!(t.named_on_battlefield("Gold").is_empty());
    // Twitch: no card drawn.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    mana(&mut t, P0, ManaType::U, 1);
    mana(&mut t, P0, ManaType::C, 2);
    let s = cast_from_hand(&mut t, P0, "Twitch", &[obj(bears)]);
    destroy(&mut t, bears);
    t.resolve_all();
    assert!(!resolved(&t, s));
    assert_eq!(t.hand_size(P0), 0);
    // The Auras: they go to the graveyard and no life is gained.
    for (aura, cost) in [("Verdant Haven", 2), ("Gift of Paradise", 2)] {
        let mut t = TestGame::new(2);
        let forest = t.battlefield(P0, "Forest");
        mana(&mut t, P0, ManaType::G, 1);
        mana(&mut t, P0, ManaType::C, cost);
        cast_from_hand(&mut t, P0, aura, &[obj(forest)]);
        destroy(&mut t, forest);
        t.resolve_all();
        assert!(t.in_graveyard(P0, aura), "{aura}");
        assert_eq!(t.life(P0), 20, "{aura}");
    }
}

#[test]
fn searching_for_a_basic_land_type_finds_any_land_with_it() {
    cr!("205.3i", "701.23a");
    ruling!(
        "Spoils of Victory",
        "Spoils of Victory allows you to search your library for any card with the land type Plains, Island, Swamp, Mountain, or Forest, not just a card with one of those names."
    );
    ruling!(
        "Vorinclex // The Grand Evolution",
        "The Forest cards you find can be any land cards with the Forest land type, not just ones named Forest."
    );
    // Spoils of Victory finds Tropical Island (Forest Island).
    let mut t = TestGame::new(2);
    let trop = t.library_top(P0, "Tropical Island");
    mana(&mut t, P0, ManaType::G, 1);
    mana(&mut t, P0, ManaType::C, 2);
    t.answer_choose(P0, &[obj(trop)]);
    cast_from_hand(&mut t, P0, "Spoils of Victory", &[]);
    t.resolve_all();
    assert_eq!(controlled(&t, P0, "Tropical Island").len(), 1);
    // Vorinclex finds Stomping Ground (Mountain Forest) and Tropical Island.
    let mut t = TestGame::new(2);
    let a = t.library_top(P0, "Stomping Ground");
    let b = t.library_top(P0, "Tropical Island");
    t.library_top(P0, "Island");
    t.answer_choose(P0, &[obj(a), obj(b)]);
    t.enter(P0, "Vorinclex // The Grand Evolution");
    t.resolve_all();
    assert!(t.in_hand(P0, "Stomping Ground"));
    assert!(t.in_hand(P0, "Tropical Island"));
}

#[test]
fn bigger_on_the_insides_cascade_may_cast_either_face_with_lower_mana_value() {
    cr!("702.85a", "712.11b");
    ruling!(
        "Bigger on the Inside",
        "Some cards, such as modal double-faced cards or adventurer cards, can be cast as one of two different spells, each with different mana costs. If the card you're casting as the cascade ability resolves is one of those cards, you may choose to cast it as either of those spells that has a lower mana value than the spell with cascade."
    );
    // Craw Wurm (mana value 6) gets cascade; Rowan (3) // Will (5): either may be cast.
    let mut t = TestGame::new(2);
    let forest = t.battlefield(P0, "Forest");
    attach_new(&mut t, P0, "Bigger on the Inside", forest);
    t.library_top(P0, "Rowan, Scholar of Sparks // Will, Scholar of Frost");
    act(&mut t, P0, forest, "Target player adds two mana", &[Entity::Player(P0)]).unwrap();
    t.resolve_all();
    assert_eq!(pool_total(&t, P0), 2);
    mana(&mut t, P0, ManaType::G, 2);
    mana(&mut t, P0, ManaType::C, 2);
    let seen = crate::r_p185_common::pick_option_containing(&mut t, P0, "Cast Will");
    t.answer_yes(P0, true);
    cast_from_hand(&mut t, P0, "Craw Wurm", &[]);
    t.resolve_all();
    let offered: Vec<Vec<String>> = seen.lock().unwrap().clone();
    assert_eq!(
        t.named_on_battlefield("Will, Scholar of Frost").len(),
        1,
        "{offered:?}\n{}",
        t.dump_log()
    );
    assert!(offered
        .iter()
        .any(|o| o.iter().any(|s| s.contains("Rowan")) && o.iter().any(|s| s.contains("Will"))));
    assert_eq!(t.named_on_battlefield("Craw Wurm").len(), 1);
}

#[test]
fn decayed_tokens_cant_block_and_are_sacrificed_after_attacking() {
    cr!("702.147a", "509.1b");
    ruling!(
        "Crowded Crypt",
        "Decayed represents a static ability and a triggered ability. “Decayed” means “This creature can't block” and “When this creature attacks, sacrifice it at end of combat.”"
    );
    ruling!(
        "Curse of the Restless Dead",
        "Decayed represents a static ability and a triggered ability. “Decayed” means “This creature can't block” and “When this creature attacks, sacrifice it at end of combat.”"
    );
    ruling!(
        "Wilhelt, the Rotcleaver",
        "Decayed represents a static ability and a triggered ability. “Decayed” means “This creature can't block” and “When this creature attacks, sacrifice it at end of combat.”"
    );
    for maker in [
        "Crowded Crypt",
        "Curse of the Restless Dead",
        "Wilhelt, the Rotcleaver",
    ] {
        let mut t = TestGame::new(2);
        match maker {
            "Crowded Crypt" => {
                let crypt = t.battlefield(P0, maker);
                put_counters(&mut t, crypt, "corpse", 1);
                mana(&mut t, P0, ManaType::B, 2);
                mana(&mut t, P0, ManaType::C, 4);
                act(&mut t, P0, crypt, "Create a 2/2", &[]).unwrap();
            }
            "Curse of the Restless Dead" => {
                attach_new(&mut t, P0, maker, Entity::Player(P1));
                t.enter(P1, "Forest");
            }
            _ => {
                t.battlefield(P0, maker);
                let corpse = t.battlefield(P0, "Walking Corpse");
                destroy(&mut t, corpse);
            }
        }
        t.resolve_all();
        let zombies: Vec<ObjectId> = tokens(&t, P0)
            .into_iter()
            .filter(|z| t.obj(*z).chars.has_subtype("Zombie"))
            .collect();
        assert_eq!(zombies.len(), 1, "{maker}");
        let z = zombies[0];
        assert!(t.obj_now(z).has_keyword(KeywordKind::Decayed), "{maker}");
        // It can't block P1's Bears.
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.advance_to(P1, Step::PrecombatMain);
        attack_with(&mut t, &[(bears, Entity::Player(P0))]);
        assert!(
            !crate::r_s21_common::legal_blocks(&mut t, P0, &[(z, bears)]),
            "{maker}"
        );
        // In P0's next turn it attacks and is sacrificed at end of combat.
        t.advance_to(P0, Step::PrecombatMain);
        let life = t.life(P1);
        t.attack(&[(z, Entity::Player(P1))], &[]);
        t.advance_to(P0, Step::PostcombatMain);
        assert_eq!(t.life(P1), life - 2, "{maker}");
        assert!(!t.g.is_live(z) || !t.on_battlefield(z), "{maker}");
    }
}
