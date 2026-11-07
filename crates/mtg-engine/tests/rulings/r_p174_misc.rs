//! Rulings batch P174 — other rulings of the batch: copies of spells and abilities
//! (CR 707.10, 707.10c, 603.2), characteristic-defining abilities in every zone
//! (CR 604.3), type-changing effects (CR 205.1b, 305.9), dividing damage on resolution
//! (CR 608.2d), excess noncombat damage (CR 120.10), costs that unattach (CR 602.2b),
//! delayed triggers and copies (CR 603.7, 707.2), last known information (CR 113.7a,
//! 608.2h), affected sets locked in on resolution (CR 611.2c), deck construction
//! (CR 100.2a, 100.6, 113.6n, 207.5), and "can't be countered" (CR 101.2).

use crate::r_p212_common::{back_face, boost};
use crate::r_s01_common::{asked_since, give_mana_for, supported, tokens};
use crate::r_s02_common::{can_cast, can_play_land, destroy};
use crate::r_s06_common::activate_containing;
use crate::r_s20_common::tap_for_mana;
use crate::r_s24_common::tapped;
use crate::r_s26_common::modify_until_eot;
use mtg_engine::ability::Modification;
use mtg_engine::card::card;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::deck::{check_commander, check_constructed, check_format_legality, circle_colors, DeckProblem};
use mtg_engine::object::{CastMethod, StackKind, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;
use std::sync::Arc;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

/// A creature P0 controls enchanted with Dual Casting ("{R}, {T}: Copy target instant or
/// sorcery spell you control. You may choose new targets for the copy.") and a Mountain.
fn dual_caster(t: &mut TestGame) -> ObjectId {
    supported("Dual Casting");
    let c = t.battlefield(P0, "Grizzly Bears");
    let aura = t.battlefield(P0, "Dual Casting");
    assert!(t.g.attach(aura, obj(c)));
    c
}

/// P0 activates Dual Casting's granted ability (with a new Mountain for {R}) targeting
/// `spell`.
fn copy_with_dual_casting(t: &mut TestGame, c: ObjectId, spell: ObjectId) {
    t.battlefield(P0, "Mountain");
    t.answer_targets(P0, &[obj(spell)]);
    activate_containing(t, P0, c, "Copy target instant or sorcery").expect("activates");
}

// --- Copies -------------------------------------------------------------------------------

#[test]
fn dual_casting_copies_a_spell_without_targets_and_the_copy_isnt_cast() {
    cr!("707.10", "707.10c", "603.2");
    ruling!(
        "Dual Casting",
        "The ability granted by Dual Casting can target (and copy) any instant or sorcery spell you control, not just one with targets."
    );
    ruling!(
        "Dual Casting",
        "The copy is created on the stack, so it’s not “cast.” Abilities that trigger when a player casts a spell won’t trigger."
    );
    supported("Young Pyromancer");
    let mut t = TestGame::new(2);
    let c = dual_caster(&mut t);
    // "Whenever you cast an instant or sorcery spell, create a 1/1 red Elemental token."
    t.battlefield(P0, "Young Pyromancer");
    give_mana_for(&mut t, P0, "Divination");
    let d = t.hand(P0, "Divination");
    let spell = t.cast(P0, d).go();
    t.resolve(); // the Young Pyromancer trigger
    assert_eq!(tokens(&t, P0).len(), 1);
    copy_with_dual_casting(&mut t, c, spell);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 4, "Divination and its copy");
    assert_eq!(tokens(&t, P0).len(), 1, "the copy wasn't cast");
}

#[test]
fn dual_casting_copy_has_the_originals_additional_costs() {
    cr!("707.10", "601.2b");
    ruling!(
        "Dual Casting",
        "effects based on any additional costs that were paid for the original spell are copied as though those same costs were paid for the copy too."
    );
    supported("Fling");
    let mut t = TestGame::new(2);
    let c = dual_caster(&mut t);
    let giant = t.battlefield(P0, "Hill Giant");
    give_mana_for(&mut t, P0, "Fling");
    let f = t.hand(P0, "Fling");
    t.answer_choose(P0, &[obj(giant)]);
    let spell = t.cast(P0, f).target(P1).go();
    assert!(!t.on_battlefield(giant));
    copy_with_dual_casting(&mut t, c, spell);
    t.resolve_all();
    assert_eq!(t.life(P1), 14, "3 from the copy, 3 from Fling");
}

#[test]
fn a_copy_of_a_boseiju_spell_can_be_countered() {
    cr!("707.10", "101.2");
    ruling!(
        "Boseiju, Who Shelters All",
        "Suppose mana produced by Boseiju is spent on a spell and an effect creates a copy of that spell. The copy can be countered, even though the original can't."
    );
    supported("Boseiju, Who Shelters All");
    supported("Cancel");
    let mut t = TestGame::new(2);
    let c = dual_caster(&mut t);
    let b = t.battlefield(P0, "Boseiju, Who Shelters All");
    t.battlefield(P0, "Island");
    t.battlefield(P0, "Wastes");
    assert!(tap_for_mana(&mut t, P0, b, "Add {C}"));
    let d = t.hand(P0, "Divination");
    let spell = t.cast(P0, d).go();
    copy_with_dual_casting(&mut t, c, spell);
    t.resolve(); // the copy is on the stack
    let copy = *t.g.stack.last().unwrap();
    assert_ne!(copy, spell);
    // P1 counters both: only the copy is countered.
    t.lands(P1, "Island", 6);
    for target in [copy, spell] {
        let cancel = t.hand(P1, "Cancel");
        t.cast(P1, cancel).target(target).go();
        t.resolve();
    }
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 2, "only the original Divination resolved");
}

#[test]
fn kirols_copy_has_the_originals_source() {
    cr!("707.10", "113.7");
    ruling!(
        "Kirol, Attentive First-Year",
        "The source of the copy created by Kirol's ability is the same as the source of the original ability."
    );
    supported("Kirol, Attentive First-Year");
    let mut t = TestGame::new(2);
    let k = t.battlefield(P0, "Kirol, Attentive First-Year");
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    let caller = t.enter(P0, "Tempest Caller");
    t.settle();
    let trig = *t.g.stack.last().unwrap();
    t.answer_targets(P0, &[obj(trig)]);
    activate_containing(&mut t, P0, k, "Copy target triggered ability").expect("activates");
    t.resolve(); // Kirol's ability: the copy is on the stack
    let copy = *t.g.stack.last().unwrap();
    assert_ne!(copy, trig);
    let source = |id: ObjectId| match &t.obj_now(id).stack.as_ref().unwrap().kind {
        StackKind::Triggered { source, .. } | StackKind::Activated { source, .. } => *source,
        StackKind::Spell => id,
    };
    assert_eq!(source(copy), caller);
}

#[test]
fn giantbaiting_populate_copy_isnt_exiled() {
    cr!("603.7", "707.2", "701.36a");
    ruling!(
        "Giantbaiting",
        "The delayed triggered ability that exiles the token isn’t a characteristic of the token. If an effect such as populate copies the token, the new token won’t be exiled."
    );
    supported("Giantbaiting");
    supported("Rootborn Defenses");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 3);
    let s = t.hand(P0, "Giantbaiting");
    t.cast(P0, s).go();
    t.resolve_all();
    let original = tokens(&t, P0);
    assert_eq!(original.len(), 1);
    give_mana_for(&mut t, P0, "Rootborn Defenses");
    let r = t.hand(P0, "Rootborn Defenses");
    t.cast(P0, r).go();
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 2);
    t.advance_to(P1, Step::Upkeep);
    let left = tokens(&t, P0);
    assert_eq!(left.len(), 1);
    assert_ne!(left[0], original[0]);
}

// --- Characteristic-defining abilities and type changes ------------------------------------

#[test]
fn power_and_toughness_cdas_work_in_every_zone() {
    cr!("604.3", "604.3a");
    ruling!(
        "Ashaya, Soul of the Wild",
        "The ability that defines Ashaya's power and toughness applies in all zones, not just the battlefield."
    );
    ruling!(
        "Veteran Warleader",
        "The ability that defines Veteran Warleader's power and toughness works in all zones, not just the battlefield. If Veteran Warleader is on the battlefield, it will count itself."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 3);
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Grizzly Bears");
    let a = t.hand(P0, "Ashaya, Soul of the Wild");
    let w = t.graveyard(P0, "Veteran Warleader");
    t.g.recompute();
    assert_eq!(t.pt(a), (3, 3));
    assert_eq!(t.pt(w), (2, 2));
    let w2 = t.battlefield(P0, "Veteran Warleader");
    assert_eq!(t.pt(w2), (3, 3));
}

#[test]
fn ashaya_creatures_are_forest_lands_but_still_cast_as_spells() {
    cr!("305.9", "205.1b", "305.1");
    ruling!(
        "Ashaya, Soul of the Wild",
        "You can't play creature cards as lands; you'll still have to cast them as spells, and they'll enter the battlefield as lands (in addition to their other types)."
    );
    supported("Ashaya, Soul of the Wild");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ashaya, Soul of the Wild");
    let bears = t.hand(P0, "Grizzly Bears");
    assert!(!can_play_land(&mut t, P0, bears));
    give_mana_for(&mut t, P0, "Grizzly Bears");
    t.cast(P0, bears).go();
    t.resolve_all();
    let b = t.named_on_battlefield("Grizzly Bears")[0];
    t.g.recompute();
    let o = t.obj_now(b);
    assert!(o.is(CardType::Land) && o.is(CardType::Creature));
    assert!(o.chars.subtypes.iter().any(|s| s.as_str() == "Forest"));
    assert!(o.chars.subtypes.iter().any(|s| s.as_str() == "Bear"));
}

#[test]
fn ygra_food_creatures_keep_their_types_and_abilities() {
    cr!("205.1b", "613.1d");
    ruling!(
        "Ygra, Eater of All",
        "The affected creatures retain their other types and abilities."
    );
    supported("Ygra, Eater of All");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ygra, Eater of All");
    let angel = t.battlefield(P1, "Serra Angel");
    t.g.recompute();
    let o = t.obj_now(angel);
    assert!(o.is(CardType::Artifact) && o.is(CardType::Creature));
    assert!(o.chars.subtypes.iter().any(|s| s.as_str() == "Food"));
    assert!(o.chars.subtypes.iter().any(|s| s.as_str() == "Angel"));
    assert!(o.has_keyword(mtg_engine::keywords::KeywordKind::Flying));
    assert!(o.has_keyword(mtg_engine::keywords::KeywordKind::Vigilance));
}

#[test]
fn thousand_moons_smithy_token_pt_changes() {
    cr!("604.3", "611.3a");
    ruling!(
        "Thousand Moons Smithy",
        "The power and toughness of the Gnome Soldier token will change as the number of artifacts and/or creatures you control changes."
    );
    supported("Thousand Moons Smithy // Barracks of the Thousand");
    let mut t = TestGame::new(2);
    t.enter(P0, "Thousand Moons Smithy // Barracks of the Thousand");
    t.resolve_all();
    let gnome = tokens(&t, P0)[0];
    // Smithy and the token.
    assert_eq!(t.pt(gnome), (2, 2));
    let b = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Ornithopter");
    assert_eq!(t.pt(gnome), (4, 4));
    destroy(&mut t, b);
    assert_eq!(t.pt(gnome), (3, 3));
}

// --- Damage -------------------------------------------------------------------------------

#[test]
fn master_of_the_wild_hunt_divides_damage_on_resolution() {
    cr!("608.2d", "601.2d");
    ruling!(
        "Master of the Wild Hunt",
        "The controller of the target creature doesn't divide that creature's damage as the ability is activated"
    );
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Master of the Wild Hunt");
    let w1 = t.battlefield(P0, "Watchwolf");
    let w2 = t.battlefield(P0, "Watchwolf");
    let giant = t.battlefield(P1, "Craw Wurm");
    let from = t.asked().len();
    t.answer_targets(P0, &[obj(giant)]);
    activate_containing(&mut t, P0, m, "Tap all untapped Wolf").expect("activates");
    let divided = |t: &TestGame, from: usize| {
        asked_since(t, from)
            .iter()
            .any(|(p, d)| *p == P1 && matches!(d, Decision::Divide { .. }))
    };
    assert!(!divided(&t, from), "nothing divided on activation");
    // As it resolves, P1 divides the Wurm's 6 damage: all of it to one Wolf.
    let from = t.asked().len();
    t.answer(P1, DecisionKind::Divide, Answer::Numbers(vec![6, 0]));
    t.resolve_all();
    assert!(divided(&t, from), "P1 divided the damage as the ability resolved");
    // The Wurm (6/4) took 3 + 3 and died; exactly one Wolf took the 6.
    assert!(!t.on_battlefield(giant));
    let dead = [w1, w2].iter().filter(|w| !t.on_battlefield(**w)).count();
    assert_eq!(dead, 1, "{}", t.dump_log());
}

#[test]
fn toralf_redirects_excess_noncombat_damage_which_can_trigger_again() {
    cr!("120.10", "120.2b", "603.2");
    ruling!(
        "Toralf, God of Fury",
        "The damage Toralf deals because of its triggered ability is noncombat damage. That damage may cause Toralf's ability to trigger again."
    );
    ruling!(
        "Toralf, God of Fury",
        "Toralf's ability doesn't affect the damage dealt to the creature or planeswalker an opponent controls. That damage will still be dealt as normal."
    );
    supported("Toralf, God of Fury // Toralf's Hammer");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Toralf, God of Fury // Toralf's Hammer");
    let elf = t.battlefield(P1, "Llanowar Elves");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P0, "Mountain");
    let bolt = t.hand(P0, "Lightning Bolt");
    // Bolt (3) the Elves (1 toughness): 2 excess, to the Bears.
    t.cast(P0, bolt).target(elf).go();
    t.answer_targets(P0, &[obj(bears)]);
    t.resolve_all();
    assert!(!t.on_battlefield(elf), "the Elves were still dealt the damage");
    assert!(!t.on_battlefield(bears), "2 excess to the Bears");
    // Chain: Bolt a 1/1 (2 excess) at another 1/1 (1 excess) at P1.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Toralf, God of Fury // Toralf's Hammer");
    let e1 = t.battlefield(P1, "Llanowar Elves");
    let e2 = t.battlefield(P1, "Llanowar Elves");
    t.battlefield(P0, "Mountain");
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(e1).go();
    t.answer_targets(P0, &[obj(e2)]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve_all();
    assert!(!t.on_battlefield(e1) && !t.on_battlefield(e2));
    assert_eq!(t.life(P1), 19, "the excess of Toralf's own damage, again");
}

#[test]
fn toralfs_hammer_isnt_reattached_if_the_ability_doesnt_resolve() {
    cr!("602.2b", "608.2b");
    ruling!(
        "Toralf, God of Fury",
        "You unattach Toralf's Hammer as part of the cost of activating the ability. If the ability doesn't resolve, Toralf's Hammer doesn't become reattached."
    );
    let mut t = TestGame::new(2);
    let hammer = back_face(&mut t, P0, "Toralf, God of Fury // Toralf's Hammer");
    let c = t.battlefield(P0, "Grizzly Bears");
    assert!(t.g.attach(hammer, obj(c)));
    t.lands(P0, "Mountain", 2);
    let target = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(target)]);
    activate_containing(&mut t, P0, c, "Unattach").expect("activates");
    assert_eq!(t.obj_now(hammer).attached_to, None, "unattached as a cost");
    destroy(&mut t, target);
    t.resolve_all();
    assert!(t.on_battlefield(hammer), "not returned to hand");
    assert_eq!(t.obj_now(hammer).attached_to, None);
}

#[test]
fn arlinn_emblem_uses_power_as_it_resolves_or_last_known() {
    cr!("608.2h", "113.7a");
    ruling!(
        "Arlinn Kord",
        "Use the power of the creature as the activated ability resolves to determine how much damage is dealt. If the creature isn’t on the battlefield at that time, use its power as it last existed on the battlefield."
    );
    supported("Arlinn Kord // Arlinn, Embraced by the Moon");
    for leaves in [false, true] {
        let mut t = TestGame::new(2);
        let arlinn = back_face(&mut t, P0, "Arlinn Kord // Arlinn, Embraced by the Moon");
        t.g.add_counters(obj(arlinn), counters::LOYALTY, 6, None);
        activate_containing(&mut t, P0, arlinn, "emblem").expect("-6");
        t.resolve_all();
        let c = t.battlefield(P0, "Grizzly Bears");
        t.answer_targets(P0, &[Entity::Player(P1)]);
        activate_containing(&mut t, P0, c, "damage equal to its power").expect("emblem ability");
        boost(&mut t, c, 3, 0);
        if leaves {
            destroy(&mut t, c);
        }
        t.resolve_all();
        assert_eq!(t.life(P1), 15, "leaves: {leaves}");
    }
}

// --- Affected sets and values locked in on resolution ---------------------------------------

#[test]
fn pump_all_affects_only_creatures_there_as_it_resolves() {
    cr!("611.2c");
    ruling!(
        "Arlinn Kord",
        "The set of creatures affected by the first ability of Arlinn, Embraced by the Moon is determined as the ability resolves."
    );
    ruling!(
        "Makindi Stampede",
        "Makindi Stampede affects only creatures you control at the time it resolves. Creatures you begin to control later in the turn won't get +2/+2."
    );
    let mut t = TestGame::new(2);
    let arlinn = back_face(&mut t, P0, "Arlinn Kord // Arlinn, Embraced by the Moon");
    t.g.add_counters(obj(arlinn), counters::LOYALTY, 3, None);
    let a = t.battlefield(P0, "Grizzly Bears");
    activate_containing(&mut t, P0, arlinn, "Creatures you control get +1/+1").expect("+1");
    t.resolve_all();
    let late = t.battlefield(P0, "Grizzly Bears");
    assert_eq!(t.pt(a), (3, 3));
    assert_eq!(t.pt(late), (2, 2));
    supported("Makindi Stampede // Makindi Mesas");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    give_mana_for(&mut t, P0, "Makindi Stampede // Makindi Mesas");
    let s = t.hand(P0, "Makindi Stampede // Makindi Mesas");
    t.cast(P0, s).go();
    t.resolve_all();
    let late = t.battlefield(P0, "Grizzly Bears");
    assert_eq!(t.pt(a), (4, 4));
    assert_eq!(t.pt(late), (2, 2));
}

#[test]
fn sorcerer_class_counts_spells_cast_before_its_trigger_resolves() {
    cr!("608.2h", "603.2");
    ruling!(
        "Sorcerer Class",
        "The last ability counts the number of spells that were cast before it resolved, not the number of spells that were cast before it was placed on the stack."
    );
    supported("Sorcerer Class");
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "Sorcerer Class");
    t.g.objects[c.0 as usize].class_level = 3;
    t.g.recompute();
    give_mana_for(&mut t, P0, "Divination");
    give_mana_for(&mut t, P0, "Opt");
    let d = t.hand(P0, "Divination");
    t.cast(P0, d).go();
    t.settle();
    // In response to the trigger, an instant.
    let o = t.hand(P0, "Opt");
    t.cast(P0, o).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 16, "both triggers deal 2");
}

#[test]
fn the_archimandrite_never_loses_life() {
    cr!("107.1b", "119.3");
    ruling!(
        "The Archimandrite",
        "You can’t gain negative amounts of life. If you have fewer than four cards in hand as the first ability resolves, you won’t gain any life, but you won’t lose any life either."
    );
    supported("The Archimandrite");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "The Archimandrite");
    t.hand(P0, "Grizzly Bears");
    t.set_step(P0, Step::Untap);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
}

// --- Targets ------------------------------------------------------------------------------

#[test]
fn rishkar_cant_target_one_creature_twice() {
    cr!("115.3");
    ruling!(
        "Rishkar, Peema Renegade",
        "You can't target the same creature twice to have one recipient get two +1/+1 counters."
    );
    supported("Rishkar, Peema Renegade");
    let mut t = TestGame::new(2);
    let b = t.battlefield(P0, "Grizzly Bears");
    let from = t.asked().len();
    t.answer_targets(P0, &[obj(b), obj(b)]);
    t.enter(P0, "Rishkar, Peema Renegade");
    t.resolve_all();
    assert!(asked_since(&t, from)
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseTargets { .. })));
    assert_ne!(t.counters(b, counters::PLUS1), 2, "the same creature twice");
    // Two different creatures: one counter each.
    let mut t = TestGame::new(2);
    let b = t.battlefield(P0, "Grizzly Bears");
    let other = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[obj(b), obj(other)]);
    t.enter(P0, "Rishkar, Peema Renegade");
    t.resolve_all();
    assert_eq!(t.counters(b, counters::PLUS1), 1);
    assert_eq!(t.counters(other, counters::PLUS1), 1);
}

#[test]
fn ruins_of_oran_rief_checks_colorless_only_now() {
    cr!("115.1", "608.2b");
    ruling!(
        "Ruins of Oran-Rief",
        "The target of the last ability didn’t necessarily have to be a colorless creature as it entered the battlefield, provided it entered the battlefield during that turn. However, it does have to be a colorless creature to be a legal target of the ability."
    );
    supported("Ruins of Oran-Rief");
    let mut t = TestGame::new(2);
    let r = t.battlefield(P0, "Ruins of Oran-Rief");
    // Entered green, now colorless: a legal target.
    let b = t.enter(P0, "Grizzly Bears");
    modify_until_eot(&mut t, b, vec![Modification::SetColors(ColorSet::NONE)]);
    // Entered colorless, now blue: not.
    let o = t.enter(P0, "Ornithopter");
    modify_until_eot(
        &mut t,
        o,
        vec![Modification::SetColors(ColorSet::single(Color::Blue))],
    );
    let from = t.asked().len();
    t.answer_targets(P0, &[obj(b)]);
    activate_containing(&mut t, P0, r, "+1/+1 counter").expect("activates");
    let cands: Vec<Entity> = asked_since(&t, from)
        .into_iter()
        .find_map(|(_, d)| match d {
            Decision::ChooseTargets { candidates, .. } => Some(candidates),
            _ => None,
        })
        .unwrap_or_default();
    assert!(!cands.contains(&obj(o)));
    t.resolve_all();
    assert_eq!(t.counters(b, counters::PLUS1), 1);
}

#[test]
fn moonsnare_prototype_owner_chooses_top_or_bottom() {
    cr!("608.2d", "401.4");
    ruling!(
        "Moonsnare Prototype",
        "The owner of that permanent chooses whether it goes on top or bottom."
    );
    supported("Moonsnare Prototype");
    let mut t = TestGame::new(2);
    let m = t.hand(P0, "Moonsnare Prototype");
    let b = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 4);
    let from = t.asked().len();
    t.answer_targets(P0, &[obj(b)]);
    activate_containing(&mut t, P0, m, "owner of target nonland permanent").expect("channel");
    t.resolve_all();
    assert!(!t.on_battlefield(b));
    assert_eq!(t.zone(t.g.current(b)), Zone::Library(P1));
    assert!(asked_since(&t, from)
        .iter()
        .any(|(p, d)| *p == P1 && !matches!(d, Decision::Priority { .. })));
}

#[test]
fn arachnus_spinner_finds_at_most_one_web() {
    cr!("701.23a");
    ruling!(
        "Arachnus Spinner",
        "You may search both your graveyard and your library, but you can find a maximum of one card named Arachnus Web."
    );
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Arachnus Spinner");
    t.graveyard(P0, "Arachnus Web");
    t.library_top(P0, "Arachnus Web");
    let b = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(b)]);
    activate_containing(&mut t, P0, s, "Tap an untapped Spider").expect("activates");
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Arachnus Web").len(), 1);
}

#[test]
fn summoning_materia_casts_with_normal_timing_and_costs() {
    cr!("601.3", "307.1", "702.33a");
    ruling!(
        "Summoning Materia",
        "You must follow all normal timing rules for spells cast using the second ability and must pay all costs for spells cast this way."
    );
    supported("Summoning Materia");
    supported("Kavu Titan");
    let setup = || {
        let mut t = TestGame::new(2);
        let eq = t.battlefield(P0, "Summoning Materia");
        let c = t.battlefield(P0, "Grizzly Bears");
        assert!(t.g.attach(eq, obj(c)));
        let k = t.library_top(P0, "Kavu Titan");
        (t, k)
    };
    // No mana: can't.
    let (mut t, k) = setup();
    assert!(!can_cast(&mut t, P0, k, CastMethod::Normal));
    // On P1's turn: can't (sorcery timing for a creature).
    let (mut t, k) = setup();
    t.lands(P0, "Forest", 5);
    t.set_step(P1, Step::PrecombatMain);
    t.g.turn.priority = Some(P0);
    assert!(!can_cast(&mut t, P0, k, CastMethod::Normal));
    // In P0's main phase, kicked: it enters with the counters.
    let (mut t, k) = setup();
    t.lands(P0, "Forest", 5);
    assert!(can_cast(&mut t, P0, k, CastMethod::Normal));
    t.cast(P0, k).kicked(true).go();
    t.resolve_all();
    let titan = t.named_on_battlefield("Kavu Titan")[0];
    assert_eq!(t.counters(titan, counters::PLUS1), 3);
}

// --- Deck construction --------------------------------------------------------------------

fn deck(of: &[(&str, usize)]) -> Vec<Arc<mtg_engine::card::CardDef>> {
    of.iter()
        .flat_map(|(n, k)| std::iter::repeat(card(n)).take(*k))
        .collect()
}

#[test]
fn any_number_of_copies_but_still_format_legality() {
    cr!("100.2a", "100.6", "113.6n");
    ruling!(
        "Persistent Petitioners",
        "The last ability of Persistent Petitioners lets you ignore the \"four-of\" rule. It doesn't let you ignore format legality."
    );
    ruling!(
        "Templar Knight",
        "The last ability of Templar Knight lets you ignore the “four-of” rule. It doesn’t let you ignore format legality."
    );
    ruling!(
        "Templar Knight",
        "The last ability of Templar Knight also lets you ignore the “singleton” rule in formats like Commander."
    );
    for name in ["Persistent Petitioners", "Templar Knight"] {
        let d = deck(&[(name, 20), ("Plains", 40)]);
        let copies = |ps: &[DeckProblem]| {
            ps.iter()
                .any(|p| matches!(p, DeckProblem::TooManyCopies { .. }))
        };
        assert!(!copies(&check_constructed(&d)), "{name}");
        let c = card(name);
        for format in ["standard", "pioneer", "vintage"] {
            let legal = c.is_legal_in(format);
            let problems = check_format_legality(&d, &[], format);
            let barred = problems
                .iter()
                .any(|p| matches!(p, DeckProblem::NotLegalInFormat { .. }));
            assert_eq!(barred, !legal, "{name} {format}");
        }
    }
    assert!(!card("Templar Knight").is_legal_in("standard"));
    // Commander: 30 Templar Knights with Thalia.
    let thalia = card("Thalia, Guardian of Thraben");
    let d = deck(&[("Templar Knight", 30), ("Plains", 69)]);
    let problems = check_commander(&d, &thalia, &[], false);
    assert!(
        !problems
            .iter()
            .any(|p| matches!(p, DeckProblem::TooManyCopies { .. })),
        "{problems:?}"
    );
}

#[test]
fn cryptic_spires_colors_are_circled_per_card_before_the_game() {
    cr!("207.5");
    ruling!(
        "Cryptic Spires",
        "You can choose the same two colors for each of them, or you can make different choices for different cards."
    );
    ruling!(
        "Cryptic Spires",
        "You circle two colors as you put Cryptic Spires into your deck before any games begin."
    );
    ruling!(
        "Cryptic Spires",
        "The mana ability is linked to the colors circled on the card. If a permanent somehow gains that mana ability without becoming a copy of Cryptic Spires, the ability it gained won't produce any mana as it resolves."
    );
    let printed = card("Cryptic Spires");
    let wu = circle_colors(&printed, [Color::White, Color::Blue]).unwrap();
    let rg = circle_colors(&printed, [Color::Red, Color::Green]).unwrap();
    // Circled once only.
    assert!(circle_colors(&wu, [Color::Black, Color::Green]).is_none());
    let mut t = TestGame::new(2);
    let a = t.custom(P0, wu, Zone::Battlefield);
    let b = t.custom(P0, rg, Zone::Battlefield);
    assert!(tap_for_mana(&mut t, P0, a, "Add"));
    assert!(tap_for_mana(&mut t, P0, b, "Add"));
    let pool = &t.g.player(P0).mana_pool;
    let wu_mana = pool.count(mtg_engine::mana::ManaType::W)
        + pool.count(mtg_engine::mana::ManaType::U);
    let rg_mana = pool.count(mtg_engine::mana::ManaType::R)
        + pool.count(mtg_engine::mana::ManaType::G);
    assert_eq!((wu_mana, rg_mana), (1, 1));
    // The uncircled ability (no colors circled for it) produces no mana.
    let mut t = TestGame::new(2);
    let u = t.battlefield(P0, "Cryptic Spires");
    let _ = tap_for_mana(&mut t, P0, u, "Add");
    assert_eq!(t.g.player(P0).mana_pool.total(), 0);
    let _ = Answer::Default;
}
