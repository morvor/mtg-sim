//! Rulings residue (batches P207–P213, Auras and Equipment): copy effects of Auras and
//! Equipment — Infinite Reflection, Metamorphic Alteration, Blade of Shared Souls,
//! Dinosaur Headdress, Mechanized Production, Moonlit Meditation — and copies of spells
//! and abilities (Rowan's Talent, Curse of Echoes).

use crate::r_p212_common::boost;
use crate::r_residue_enchant_equip_common::*;
use crate::r_s01_common::*;
use crate::r_s02_common::create_token;
use crate::r_s06_common::*;
use crate::r_s08_common::mana_value;
use crate::r_s25_common::{cast_new, spell_copies};
use crate::r_s03_common::respond;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

const REFLECTION: &str = "Infinite Reflection";
const BLADE: &str = "Blade of Shared Souls";

// --- Infinite Reflection ------------------------------------------------------------------

/// P0's Infinite Reflection enchanting `host` (already there: its enters trigger doesn't
/// happen).
fn reflection_on(t: &mut TestGame, host: ObjectId) -> ObjectId {
    supported(REFLECTION);
    aura_on(t, P0, REFLECTION, host)
}

#[test]
fn infinite_reflection_enters_triggers_are_those_of_the_enchanted_creature() {
    cr!("614.1c", "614.12", "707.2", "603.6a");
    ruling!(
        "Infinite Reflection",
        "any \"enters\" triggered abilities printed on such creatures won't trigger. However, if the enchanted creature has any \"enters\" triggered abilities, those will trigger."
    );
    supported("Elvish Visionary");
    // Elvish Visionary: "When this creature enters, draw a card." It enters as a copy of
    // the Bears: no card.
    let mut t = TestGame::new(2);
    t.library_top(P0, "Island");
    let bears = t.battlefield(P0, "Grizzly Bears");
    reflection_on(&mut t, bears);
    let hand = t.hand_size(P0);
    let ev = t.enter(P0, "Elvish Visionary");
    t.settle();
    t.resolve_all();
    assert_eq!(name(&t, ev), "Grizzly Bears");
    assert_eq!(t.hand_size(P0), hand);
    // The Bears enter as a copy of the enchanted Elvish Visionary: a card.
    let mut t = TestGame::new(2);
    t.library_top(P0, "Island");
    let ev = t.battlefield(P0, "Elvish Visionary");
    reflection_on(&mut t, ev);
    let hand = t.hand_size(P0);
    let bears = t.enter(P0, "Grizzly Bears");
    t.settle();
    t.resolve_all();
    assert_eq!(name(&t, bears), "Elvish Visionary");
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn infinite_reflection_copies_only_the_copiable_values() {
    cr!("707.2", "707.2a", "122.1");
    ruling!(
        "Infinite Reflection",
        "Creatures you control don't copy whether the enchanted creature is tapped or untapped, whether it has any counters on it, whether it has any Auras and Equipment attached to it, or any noncopy effects"
    );
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    reflection_on(&mut t, giant);
    t.g.obj_mut(giant).tapped = true;
    t.g.add_counters(obj(giant), counters::PLUS1, 2, None);
    boost(&mut t, giant, 3, 3);
    attach_new(&mut t, P0, "Holy Strength", giant);
    assert_eq!(t.pt(giant), (3 + 2 + 3 + 1, 3 + 2 + 3 + 2));
    let bears = t.enter(P0, "Grizzly Bears");
    t.settle();
    assert_eq!(name(&t, bears), "Hill Giant");
    assert_eq!(t.pt(bears), (3, 3));
    assert!(!t.obj_now(bears).tapped);
    assert_eq!(t.counters(bears, counters::PLUS1), 0);
}

#[test]
fn infinite_reflection_external_enters_effects_still_apply() {
    cr!("614.12", "707.2");
    ruling!(
        "Infinite Reflection",
        "External abilities may still affect how a creature enters."
    );
    supported("Urabrask the Hidden");
    let mut t = TestGame::new(2);
    let angel = t.battlefield(P0, "Serra Angel");
    reflection_on(&mut t, angel);
    // Urabrask the Hidden: "Creatures your opponents control enter tapped."
    t.battlefield(P1, "Urabrask the Hidden");
    let bears = t.enter(P0, "Grizzly Bears");
    t.settle();
    assert_eq!(name(&t, bears), "Serra Angel");
    assert!(t.obj_now(bears).tapped);
}

#[test]
fn infinite_reflection_and_clone_both_copy_the_enchanted_creature() {
    cr!("616.1", "707.2", "614.12");
    ruling!(
        "Infinite Reflection",
        "No matter what order these effects are applied, the creature will be a copy of the enchanted creature when it enters."
    );
    supported("Clone");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    reflection_on(&mut t, giant);
    let angel = t.battlefield(P1, "Serra Angel");
    // Clone's controller would have it copy the Angel.
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(angel)]);
    let clone = t.enter(P0, "Clone");
    t.settle();
    assert_eq!(name(&t, clone), "Hill Giant", "{}", t.dump_log());
    assert_eq!(t.pt(clone), (3, 3));
}

#[test]
fn infinite_reflection_can_enchant_another_players_creature() {
    cr!("303.4a", "303.4", "707.2");
    ruling!(
        "Infinite Reflection",
        "Infinite Reflection can enchant a creature controlled by another player."
    );
    supported(REFLECTION);
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let giant = t.battlefield(P1, "Hill Giant");
    cast_new(&mut t, P0, REFLECTION, &[obj(giant)]);
    t.resolve_all();
    let aura = t.named_on_battlefield(REFLECTION)[0];
    assert_eq!(attached_to(&t, aura), Some(obj(giant)));
    assert_eq!(t.obj_now(aura).controller, P0);
    // P0's creatures enter as copies of P1's Hill Giant.
    let bears = t.enter(P0, "Grizzly Bears");
    t.settle();
    assert_eq!(name(&t, bears), "Hill Giant");
    assert_eq!(t.obj_now(bears).controller, P0);
    // P1's don't.
    let theirs = t.enter(P1, "Grizzly Bears");
    t.settle();
    assert_eq!(name(&t, theirs), "Grizzly Bears");
}

#[test]
fn infinite_reflection_printed_enters_replacements_do_not_apply() {
    cr!("614.12", "614.1c", "707.2");
    ruling!(
        "Infinite Reflection",
        "a creature that normally enters tapped will enter as an untapped copy of Serra Angel, and a creature that would normally enter with counters on it will enter as a copy of Serra Angel with no counters."
    );
    supported("Diregraf Ghoul");
    supported("Kalonian Hydra");
    let mut t = TestGame::new(2);
    let angel = t.battlefield(P0, "Serra Angel");
    reflection_on(&mut t, angel);
    // Diregraf Ghoul: "This creature enters tapped."
    let ghoul = t.enter(P0, "Diregraf Ghoul");
    t.settle();
    assert_eq!(name(&t, ghoul), "Serra Angel");
    assert!(!t.obj_now(ghoul).tapped);
    // Kalonian Hydra: "This creature enters with four +1/+1 counters on it."
    let hydra = t.enter(P0, "Kalonian Hydra");
    t.settle();
    assert_eq!(name(&t, hydra), "Serra Angel");
    assert_eq!(t.counters(hydra, counters::PLUS1), 0);
}

// --- Metamorphic Alteration ---------------------------------------------------------------

#[test]
fn metamorphic_alteration_cannot_choose_a_creature_entering_with_it() {
    cr!("614.12", "614.1c", "303.4a");
    ruling!(
        "Metamorphic Alteration",
        "it can't have the enchanted creature become a copy of that creature. You may choose only a creature that's already on the battlefield."
    );
    supported("Metamorphic Alteration");
    supported("Open the Vaults");
    // Open the Vaults: "Return all artifact and enchantment cards from all graveyards to
    // the battlefield under their owners' control." Ornithopter and the Aura enter at the
    // same time: the Aura (enchanting the Bears) can choose only the Hill Giant.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    t.graveyard(P0, "Ornithopter");
    t.graveyard(P0, "Metamorphic Alteration");
    // P0 has the Aura enchant the Bears and chooses the Hill Giant if offered.
    respond(&mut t, P0, |g, d| match d {
        Decision::ChooseEntities {
            candidates, prompt, ..
        } => {
            let want = if prompt == "Choose what the Aura will enchant" {
                "Grizzly Bears"
            } else {
                "Hill Giant"
            };
            let pick = candidates
                .iter()
                .copied()
                .find(|e| matches!(e, Entity::Object(o) if g.obj(*o).chars.name == want));
            pick.map(|e| Answer::Entities(vec![e]))
        }
        _ => None,
    });
    let from = t.asked().len();
    cast_new(&mut t, P0, "Open the Vaults", &[]);
    t.resolve_all();
    let offered: Vec<(String, Vec<Entity>)> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities {
                candidates, prompt, ..
            } => Some((prompt.clone(), candidates.clone())),
            _ => None,
        })
        .collect();
    assert_eq!(offered.len(), 2, "{offered:?}");
    assert!(offered.iter().any(|(_, c)| c.contains(&obj(giant))));
    let thopter = t.named_on_battlefield("Ornithopter")[0];
    for (_, c) in &offered {
        assert!(!c.contains(&obj(thopter)), "{c:?}");
    }
    let aura = t.named_on_battlefield("Metamorphic Alteration")[0];
    assert_eq!(attached_to(&t, aura), Some(obj(bears)));
    assert_eq!(name(&t, bears), "Hill Giant");
}

#[test]
fn metamorphic_alteration_copying_a_token_copies_its_original_characteristics() {
    cr!("707.2", "111.4", "707.2a");
    ruling!(
        "Metamorphic Alteration",
        "If the chosen creature is a token, Metamorphic Alteration copies the original characteristics of that token"
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let soldier = create_token(&mut t, P1, "Soldier");
    boost(&mut t, soldier, 2, 2);
    alter(&mut t, P0, bears, soldier);
    assert_eq!(name(&t, bears), "Soldier");
    assert_eq!(t.pt(bears), (1, 1));
    assert!(!t.obj_now(bears).is_token());
}

/// `p`'s Treetop Village, animated ("{1}{G}: This land becomes a 3/3 green Ape creature
/// with trample until end of turn. It's still a land.").
fn animated_village(t: &mut TestGame, p: PlayerId) -> ObjectId {
    supported("Treetop Village");
    let v = t.battlefield(p, "Treetop Village");
    t.lands(p, "Forest", 2);
    activate_containing(t, p, v, "becomes").expect("animate");
    t.resolve_all();
    assert!(t.obj_now(v).is(CardType::Creature));
    v
}

#[test]
fn metamorphic_alteration_copying_an_animated_land_falls_off() {
    cr!("707.2", "704.5m", "303.4d");
    ruling!(
        "Metamorphic Alteration",
        "the enchanted creature becomes a copy of that noncreature permanent. This causes Metamorphic Alteration to be illegally attached"
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let village = animated_village(&mut t, P1);
    let aura = alter(&mut t, P0, bears, village);
    assert!(in_zone(&t, aura, Zone::Graveyard(P0)), "{}", t.dump_log());
    assert_eq!(name(&t, bears), "Grizzly Bears");
    assert!(t.obj_now(bears).is(CardType::Creature));
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn metamorphic_alteration_dies_triggers_are_the_copys() {
    cr!("603.10a", "707.2", "603.6c");
    ruling!(
        "Metamorphic Alteration",
        "any \"when this creature dies\" abilities it gains from the copy effect will trigger. Any \"when this creature dies\" abilities it normally has will not trigger."
    );
    supported("Doomed Traveler");
    // Doomed Traveler: "When this creature dies, create a 1/1 white Spirit creature token
    // with flying." The Bears copying it make a Spirit as they die.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let traveler = t.battlefield(P1, "Doomed Traveler");
    alter(&mut t, P0, bears, traveler);
    assert_eq!(name(&t, bears), "Doomed Traveler");
    t.g.destroy(bears, None);
    t.settle();
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Spirit").len(), 1);
    // A Doomed Traveler copying the Bears doesn't.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let traveler = t.battlefield(P0, "Doomed Traveler");
    let bears = t.battlefield(P1, "Grizzly Bears");
    alter(&mut t, P0, traveler, bears);
    assert_eq!(name(&t, traveler), "Grizzly Bears");
    t.g.destroy(traveler, None);
    t.settle();
    t.resolve_all();
    assert!(with_subtype(&t, P0, "Spirit").is_empty());
}

// --- Blade of Shared Souls ---------------------------------------------------------------

/// Attaches P0's Blade of Shared Souls to `host`, having it become a copy of `target`.
fn blade_copy(t: &mut TestGame, blade: ObjectId, host: ObjectId, target: ObjectId) {
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(target)]);
    assert!(t.g.attach(blade, obj(host)));
    t.g.flush_events();
    t.settle();
    t.resolve_all();
}

#[test]
fn blade_of_shared_souls_copies_the_face_shown() {
    cr!("712.8e", "707.2", "202.3e");
    ruling!(
        "Blade of Shared Souls",
        "If the target creature is a double-faced or melded permanent, the equipped creature copies the face shown. If the equipped creature copies a melded permanent or the back face of a double-faced card, its mana value is 0."
    );
    supported(BLADE);
    supported("Delver of Secrets // Insectile Aberration");
    let mut t = TestGame::new(2);
    let blade = t.battlefield(P0, BLADE);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let delver = t.battlefield(P0, "Delver of Secrets // Insectile Aberration");
    assert!(mtg_engine::dfc::transform(&mut t.g, delver));
    t.g.recompute();
    assert_eq!(name(&t, delver), "Insectile Aberration");
    assert_eq!(mana_value(&t, delver), 1);
    blade_copy(&mut t, blade, bears, delver);
    assert_eq!(name(&t, bears), "Insectile Aberration");
    assert_eq!(t.pt(bears), (3, 2));
    assert_eq!(mana_value(&t, bears), 0);
}

#[test]
fn blade_of_shared_souls_and_tokens() {
    cr!("707.2", "111.4", "707.2a");
    ruling!(
        "Blade of Shared Souls",
        "If the target creature is a token, the equipped creature copies the original characteristics of that token as stated by the effect that created it. If the equipped creature isn't a token, copying a token won't cause it to become a token. Similarly, if the equipped creature is a token, copying a nontoken won't cause it to become a nontoken."
    );
    let mut t = TestGame::new(2);
    let blade = t.battlefield(P0, BLADE);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let soldier = create_token(&mut t, P0, "Soldier");
    boost(&mut t, soldier, 2, 2);
    blade_copy(&mut t, blade, bears, soldier);
    assert_eq!(name(&t, bears), "Soldier");
    assert_eq!(t.pt(bears), (1, 1));
    assert!(!t.obj_now(bears).is_token());
    // A token copying a nontoken stays a token.
    let mut t = TestGame::new(2);
    let blade = t.battlefield(P0, BLADE);
    let soldier = create_token(&mut t, P0, "Soldier");
    let giant = t.battlefield(P0, "Hill Giant");
    blade_copy(&mut t, blade, soldier, giant);
    assert_eq!(name(&t, soldier), "Hill Giant");
    assert!(t.obj_now(soldier).is_token());
}

#[test]
fn blade_of_shared_souls_copying_a_copy() {
    cr!("707.3", "707.2");
    ruling!(
        "Blade of Shared Souls",
        "If the target creature is copying something else (for example, if that creature is equipped by another Blade of Shared Souls), then the equipped creature becomes a copy of whatever the target creature copied."
    );
    let mut t = TestGame::new(2);
    let blade1 = t.battlefield(P0, BLADE);
    let blade2 = t.battlefield(P0, BLADE);
    let giant = t.battlefield(P0, "Hill Giant");
    let angel = t.battlefield(P0, "Serra Angel");
    let bears = t.battlefield(P0, "Grizzly Bears");
    blade_copy(&mut t, blade1, giant, angel);
    assert_eq!(name(&t, giant), "Serra Angel");
    blade_copy(&mut t, blade2, bears, giant);
    assert_eq!(name(&t, bears), "Serra Angel");
    assert_eq!(t.pt(bears), (4, 4));
}

#[test]
fn blade_of_shared_souls_copying_an_animated_land_falls_off() {
    cr!("707.2", "704.5n", "611.2b");
    ruling!(
        "Blade of Shared Souls",
        "the equipped creature will become a copy of the noncreature permanent. Blade of Shared Souls will become unattached and the copy effect will end."
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let blade = t.battlefield(P0, BLADE);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let village = animated_village(&mut t, P0);
    blade_copy(&mut t, blade, bears, village);
    assert!(t.on_battlefield(blade));
    assert_eq!(t.obj_now(blade).attached_to, None, "{}", t.dump_log());
    assert_eq!(name(&t, bears), "Grizzly Bears");
    assert!(t.obj_now(bears).is(CardType::Creature));
}

#[test]
fn blade_of_shared_souls_copies_only_printed_values_and_keeps_non_copy_effects() {
    cr!("707.2", "613.1a", "613.4c");
    ruling!(
        "Blade of Shared Souls",
        "Non-copy effects that have already applied to the equipped creature will continue to apply to it. For example, +1/+1 counters on it will still affect it."
    );
    ruling!(
        "Blade of Shared Souls",
        "It doesn't copy whether that creature is tapped or untapped, whether it has any counters on it or any Auras and Equipment attached to it, or any non-copy effects that have changed its power, toughness, types, color, or so on."
    );
    let mut t = TestGame::new(2);
    let blade = t.battlefield(P0, BLADE);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.add_counters(obj(bears), counters::PLUS1, 1, None);
    let giant = t.battlefield(P0, "Hill Giant");
    t.g.obj_mut(giant).tapped = true;
    t.g.add_counters(obj(giant), counters::PLUS1, 2, None);
    boost(&mut t, giant, 5, 5);
    attach_new(&mut t, P0, "Holy Strength", giant);
    blade_copy(&mut t, blade, bears, giant);
    assert_eq!(name(&t, bears), "Hill Giant");
    // Hill Giant 3/3 with its own +1/+1 counter.
    assert_eq!(t.pt(bears), (4, 4));
    assert!(!t.obj_now(bears).tapped);
}

// --- Dinosaur Headdress -------------------------------------------------------------------

#[test]
fn dinosaur_headdress_copies_the_exiled_card_not_the_permanent_it_was() {
    cr!("707.2", "702.167c", "400.7");
    ruling!(
        "Paleontologist's Pick-Axe // Dinosaur Headdress",
        "The equipped creature copies exactly what's printed on the exiled card and nothing else. It doesn't copy any information about the object the card was before it was put into exile."
    );
    const AXE: &str = "Paleontologist's Pick-Axe // Dinosaur Headdress";
    supported(AXE);
    // The Hill Giant used to craft was a copy of a Serra Angel (Metamorphic Alteration),
    // with counters and a pump: the Bears copy the Hill Giant card.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let axe = t.battlefield(P0, AXE);
    let giant = t.battlefield(P0, "Hill Giant");
    let angel = t.battlefield(P1, "Serra Angel");
    alter(&mut t, P0, giant, angel);
    assert_eq!(name(&t, giant), "Serra Angel");
    t.g.add_counters(obj(giant), counters::PLUS1, 2, None);
    boost(&mut t, giant, 1, 1);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Wastes", 5);
    t.answer_choose(P0, &[obj(giant)]);
    t.answer_targets(P0, &[obj(bears)]);
    activate_containing(&mut t, P0, axe, "Craft").expect("craft");
    t.settle();
    t.resolve_all();
    assert_eq!(name(&t, bears), "Hill Giant", "{}", t.dump_log());
    assert_eq!(t.pt(bears), (3, 3));
}

// --- Mechanized Production ----------------------------------------------------------------

const PRODUCTION: &str = "Mechanized Production";

/// P0 controls Mechanized Production enchanting an Ornithopter and `treasures` Treasure
/// tokens; the game is at P0's upkeep with the trigger on the stack.
fn production(treasures: usize) -> (TestGame, ObjectId, ObjectId) {
    supported(PRODUCTION);
    let mut t = TestGame::new(2);
    let thopter = t.battlefield(P0, "Ornithopter");
    let aura = aura_on(&mut t, P0, PRODUCTION, thopter);
    for _ in 0..treasures {
        create_token(&mut t, P0, "Treasure");
    }
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "copy of enchanted"), 1, "{}", t.dump_log());
    (t, aura, thopter)
}

fn thopter_tokens(t: &TestGame) -> usize {
    t.named_on_battlefield("Ornithopter")
        .into_iter()
        .filter(|o| t.g.obj(*o).is_token())
        .count()
}

#[test]
fn mechanized_production_eight_artifacts_need_not_share_the_enchanted_ones_name() {
    cr!("104.2b", "608.2c");
    ruling!(
        "Mechanized Production",
        "The eight artifacts with the same name don't have to have the same name as the enchanted artifact."
    );
    let (mut t, _, _) = production(8);
    t.resolve_all();
    assert_eq!(thopter_tokens(&t), 1);
    assert!(t.has_lost(P1));
    // Seven Treasures: no win.
    let (mut t, _, _) = production(7);
    t.resolve_all();
    assert!(!t.has_lost(P1));
}

#[test]
fn mechanized_production_leaving_with_its_artifact_copies_the_last_known_artifact() {
    cr!("608.2h", "707.2", "603.10");
    ruling!(
        "Mechanized Production",
        "then the effect creates a token that's a copy of the artifact as it last existed on the battlefield."
    );
    let (mut t, aura, thopter) = production(0);
    t.g.destroy_all(vec![aura, thopter], None, false);
    t.g.flush_events();
    t.settle();
    t.resolve_all();
    assert_eq!(thopter_tokens(&t), 1, "{}", t.dump_log());
}

#[test]
fn mechanized_production_artifact_leaving_alone_makes_no_token() {
    cr!("704.5m", "608.2h");
    ruling!(
        "Mechanized Production",
        "The triggered ability creates no token, but you can still win the game if you control enough artifacts with the same name."
    );
    let (mut t, aura, thopter) = production(8);
    t.g.destroy(thopter, None);
    t.g.flush_events();
    t.settle();
    assert!(in_zone(&t, aura, Zone::Graveyard(P0)));
    t.resolve_all();
    assert_eq!(thopter_tokens(&t), 0, "{}", t.dump_log());
    assert!(t.has_lost(P1));
}

// --- Moonlit Meditation -------------------------------------------------------------------

#[test]
fn moonlit_meditation_replaces_any_token_even_a_shard() {
    cr!("614.1a", "111.10e", "707.2");
    ruling!(
        "Moonlit Meditation",
        "you could replace creating a Shard token (a predefined enchantment token) with creating a copy of the enchanted permanent."
    );
    supported("Moonlit Meditation");
    let mut t = TestGame::new(2);
    let thopter = t.battlefield(P0, "Ornithopter");
    aura_on(&mut t, P0, "Moonlit Meditation", thopter);
    t.answer_yes(P0, true);
    create_token(&mut t, P0, "Shard");
    assert!(with_subtype(&t, P0, "Shard").is_empty(), "{}", t.dump_log());
    assert_eq!(thopter_tokens(&t), 1);
}

// --- Rowan's Talent -----------------------------------------------------------------------

#[test]
fn rowans_talent_copy_of_a_loyalty_ability_changes_no_loyalty() {
    cr!("707.10", "606.4", "602.2");
    ruling!(
        "Rowan's Talent",
        "Copying a loyalty ability doesn't cause any loyalty counters to be added to or removed from the enchanted planeswalker."
    );
    supported("Rowan's Talent");
    supported("Liliana of the Veil");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let lili = t.battlefield(P0, "Liliana of the Veil");
    let bears = t.battlefield(P0, "Grizzly Bears");
    aura_on(&mut t, P0, "Rowan's Talent", lili);
    let loyalty = t.counters(lili, counters::LOYALTY);
    t.answer_targets(P0, &[obj(bears)]);
    activate_containing(&mut t, P0, lili, "+2/+0").expect("+1");
    assert_eq!(t.counters(lili, counters::LOYALTY), loyalty + 1);
    t.settle();
    t.resolve_all();
    // Both the ability and its copy resolved: +4/+0, and the loyalty is unchanged.
    assert_eq!(t.pt(bears), (6, 2), "{}", t.dump_log());
    assert_eq!(t.counters(lili, counters::LOYALTY), loyalty + 1);
}

// --- Curse of Echoes ----------------------------------------------------------------------

#[test]
fn curse_of_echoes_copies_are_created_in_turn_order_from_the_active_player() {
    cr!("707.10", "101.4", "405.2");
    ruling!(
        "Curse of Echoes",
        "The active player creates their copy and may choose new targets first (if that player isn’t the enchanted player), then each other player (except the enchanted player) creates their copy and may choose new targets in turn order. The last player who creates their copy controls the copy that will resolve first."
    );
    supported("Curse of Echoes");
    for (active, order) in [(P0, [P0, P2, P3]), (P2, [P2, P3, P0]), (P1, [P2, P3, P0])] {
        let mut t = TestGame::new(4);
        t.set_step(active, Step::PrecombatMain);
        aura_on(&mut t, P0, "Curse of Echoes", P1);
        for p in [P0, P2, P3] {
            t.answer_yes(p, true);
            t.answer_yes(p, false);
        }
        t.g.turn.priority = Some(P1);
        cast_new(&mut t, P1, "Lightning Bolt", &[Entity::Player(P1)]);
        t.settle();
        // Curse of Echoes's trigger resolves.
        t.g.resolve_top();
        t.settle();
        let copies: Vec<PlayerId> = spell_copies(&t)
            .into_iter()
            .map(|c| t.g.obj(c).controller)
            .collect();
        assert_eq!(copies, order.to_vec(), "active {active:?}");
    }
}

#[test]
fn curse_of_echoes_copies_keep_the_paid_additional_costs() {
    cr!("707.10", "601.2f", "118.8");
    ruling!(
        "Curse of Echoes",
        "effects based on any additional costs that were paid for the original spell are copied as though those same costs were paid for the copy too."
    );
    supported("Fling");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    aura_on(&mut t, P0, "Curse of Echoes", P1);
    let giant = t.battlefield(P1, "Hill Giant");
    // P0 copies and has the copy target P1.
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_choose(P1, &[obj(giant)]);
    t.g.turn.priority = Some(P1);
    cast_new(&mut t, P1, "Fling", &[Entity::Player(P0)]);
    t.settle();
    t.resolve_all();
    assert_eq!(t.life(P0), 17);
    assert_eq!(t.life(P1), 17, "{}", t.dump_log());
}
