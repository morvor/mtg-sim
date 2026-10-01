//! Rulings batch P020 — copy effects on objects: a copy gets only the copiable values of
//! what it copies (printed values plus copy effects, CR 707.2, 707.3), not its status,
//! counters or other effects; a permanent that becomes a copy keeps its own counters and
//! the non-copy effects applying to it (CR 613.1a, 613.2), and doesn't "enter", so the
//! copied enters abilities don't apply (CR 603.6a, 614.1c); "until end of turn" copy
//! effects end in the cleanup step (CR 514.2); choosing what to copy isn't targeting
//! (CR 115.1, 707.9).

use crate::r_p020_common::*;
use crate::r_s01_common::supported;
use crate::r_s06_common::{activate_containing, damage};
use crate::r_s25_common::{cast_new, lands_for_cost};
use crate::r_s26_common::{dress_up, fresh};
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Puts `n` +1/+1 counters on the permanent.
fn plus1(t: &mut TestGame, id: ObjectId, n: u32) {
    let id = t.g.current(id);
    t.g.add_counters(Entity::Object(id), counters::PLUS1, n, None);
    t.g.recompute();
}

#[test]
fn likeness_looter_keeps_its_effects_and_counters_after_becoming_a_copy() {
    cr!("707.2", "613.1a", "613.4c", "122.1a");
    ruling!(
        "Likeness Looter",
        "Any effects that applied to Likeness Looter before it becomes a copy of another card will continue to apply after it becomes a copy. The same is true of any counters that are on Likeness Looter."
    );
    ruling!(
        "Likeness Looter",
        "Because Likeness Looter isn't entering the battlefield when it becomes a copy of a card, any \"When [this creature] enters the battlefield\" or \"[This creature] enters the battlefield with\" abilities of the copied card won't apply."
    );
    // "{X}: This creature becomes a copy of target creature card in your graveyard with mana
    // value X, except it has flying and this ability. Activate only as a sorcery."
    supported("Likeness Looter");
    let mut t = TestGame::new(2);
    let looter = t.battlefield(P0, "Likeness Looter");
    let card = t.graveyard(P0, RIFTWATCHER);
    plus1(&mut t, looter, 1);
    cast_new(&mut t, P0, "Giant Growth", &[Entity::Object(looter)]);
    t.resolve_all();
    let life = t.life(P0);
    t.lands(P0, "Wastes", 3);
    t.answer(P0, DecisionKind::X, Answer::Number(3));
    t.answer_targets(P0, &[Entity::Object(card)]);
    activate_containing(&mut t, P0, looter, "becomes a copy").expect("activation");
    t.resolve_all();
    assert_eq!(t.obj_now(looter).chars.name, RIFTWATCHER);
    // Aven Riftwatcher's 2/3, its +1/+1 counter and Giant Growth's +3/+3.
    assert_eq!(t.pt(looter), (2 + 1 + 3, 3 + 1 + 3));
    // It didn't enter: no time counters (vanishing) and no life (its enters trigger).
    assert_eq!(t.counters(looter, counters::TIME), 0);
    assert_eq!(t.life(P0), life);
}

#[test]
fn fleeting_reflection_target_keeps_its_effects_and_counters() {
    cr!("707.2", "613.1a", "613.4c", "611.2c");
    ruling!(
        "Fleeting Reflection",
        "Any non-copy effects that applied to the first target creature before it becomes a copy of another creature will continue to apply once it becomes a copy. The same is true of any counters that are on the first creature."
    );
    // "Target creature you control gains hexproof until end of turn. Untap that creature.
    // Until end of turn, it becomes a copy of up to one other target creature."
    supported("Fleeting Reflection");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    plus1(&mut t, bears, 1);
    cast_new(&mut t, P0, "Giant Growth", &[Entity::Object(bears)]);
    t.resolve_all();
    cast_new(
        &mut t,
        P0,
        "Fleeting Reflection",
        &[Entity::Object(bears), Entity::Object(giant)],
    );
    t.resolve_all();
    assert_eq!(t.obj_now(bears).chars.name, "Hill Giant");
    assert_eq!(t.pt(bears), (3 + 1 + 3, 3 + 1 + 3));
    // The hexproof it gained from the same spell before becoming a copy still applies.
    assert!(t
        .obj_now(bears)
        .has_keyword(mtg_engine::keywords::KeywordKind::Hexproof));
}

#[test]
fn artisan_of_forms_copies_printed_values_and_keeps_its_counters() {
    cr!("707.2", "707.3", "613.1a", "122.1a");
    ruling!(
        "Artisan of Forms",
        "Artisan of Forms copies the printed values of the creature plus any copy effects that have been applied to it. It won't copy any other effects that have changed that creature's power, toughness, color, and so on. Artisan of Forms won't copy any counters on the creature, but Artisan of Forms will retain any counters it already had on it."
    );
    // "Heroic — Whenever you cast a spell that targets this creature, you may have this
    // creature become a copy of target creature, except it has this ability." The copied
    // creature is a Clone copying Hill Giant, then dressed up.
    supported("Artisan of Forms");
    let mut t = TestGame::new(2);
    let artisan = t.battlefield(P0, "Artisan of Forms");
    plus1(&mut t, artisan, 1);
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_choose(P1, &[Entity::Object(giant)]);
    let clone = t.enter(P1, "Clone");
    dress_up(&mut t, clone);
    t.answer_yes(P0, true);
    t.lands(P0, "Forest", 1);
    let growth = t.hand(P0, "Giant Growth");
    t.cast(P0, growth).target(Entity::Object(artisan)).go();
    t.answer_targets(P0, &[Entity::Object(t.g.current(clone))]);
    t.settle();
    // The heroic trigger resolves first, then Giant Growth.
    t.resolve_all();
    let o = t.obj_now(artisan);
    assert_eq!(o.chars.name, "Hill Giant");
    assert!(o.chars.colors.contains(Color::Red) && !o.chars.colors.contains(Color::Blue));
    assert_eq!(t.counters(artisan, counters::PLUS1), 1);
    assert_eq!(t.pt(artisan), (3 + 1 + 3, 3 + 1 + 3));
    // It still has its heroic ability.
    assert!(o.chars.abilities.iter().any(|a| a.text.contains("cast a spell that targets")));
}

/// Advances to P1's next upkeep (after P0's cleanup step).
fn next_turn(t: &mut TestGame) {
    t.advance_to(P1, Step::Upkeep);
}

#[test]
fn two_mirrorweaves_wear_off_at_the_same_time() {
    cr!("514.2", "611.2a", "707.2");
    ruling!(
        "Mirrorweave",
        "As the turn ends, the other creatures revert to what they were before. If two Mirrorweaves are cast on the same turn, they’ll both wear off at the same time."
    );
    // "Each other creature becomes a copy of target nonlegendary creature until end of
    // turn."
    supported("Mirrorweave");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let elves = t.battlefield(P1, "Llanowar Elves");
    cast_new(&mut t, P0, "Mirrorweave", &[Entity::Object(giant)]);
    t.resolve_all();
    assert_eq!(t.obj_now(bears).chars.name, "Hill Giant");
    assert_eq!(t.obj_now(elves).chars.name, "Hill Giant");
    cast_new(&mut t, P0, "Mirrorweave", &[Entity::Object(bears)]);
    t.resolve_all();
    assert_eq!(t.obj_now(elves).chars.name, "Hill Giant");
    next_turn(&mut t);
    assert_eq!(t.obj_now(bears).chars.name, "Grizzly Bears");
    assert_eq!(t.obj_now(giant).chars.name, "Hill Giant");
    assert_eq!(t.obj_now(elves).chars.name, "Llanowar Elves");
    assert_eq!(t.pt(elves), (1, 1));
}

#[test]
fn two_nanogene_conversions_wear_off_at_the_same_time() {
    cr!("514.2", "611.2a", "707.2", "707.9b");
    ruling!(
        "Nanogene Conversion",
        "As the turn ends, the other creatures revert to what they were before. If two Nanogene Conversions are cast on the same turn, they'll both wear off at the same time."
    );
    // "Choose target creature you control. Each other creature becomes a copy of that
    // creature until end of turn, except it isn't legendary."
    supported("Nanogene Conversion");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let elves = t.battlefield(P1, "Llanowar Elves");
    cast_new(&mut t, P0, "Nanogene Conversion", &[Entity::Object(giant)]);
    t.resolve_all();
    assert_eq!(t.obj_now(bears).chars.name, "Hill Giant");
    cast_new(&mut t, P0, "Nanogene Conversion", &[Entity::Object(bears)]);
    t.resolve_all();
    assert_eq!(t.obj_now(elves).chars.name, "Hill Giant");
    next_turn(&mut t);
    assert_eq!(t.obj_now(bears).chars.name, "Grizzly Bears");
    assert_eq!(t.obj_now(giant).chars.name, "Hill Giant");
    assert_eq!(t.obj_now(elves).chars.name, "Llanowar Elves");
}

#[test]
fn two_cytoshapes_on_one_creature_wear_off_at_the_same_time() {
    cr!("514.2", "611.2a", "707.2", "613.2a");
    ruling!(
        "Cytoshape",
        "At the end of the turn, the creature reverts to what it was before. If two Cytoshapes affect the same creature on the same turn, they’ll both wear off at the same time."
    );
    // "Choose a nonlegendary creature on the battlefield. Target creature becomes a copy of
    // that creature until end of turn."
    supported("Cytoshape");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let elves = t.battlefield(P1, "Llanowar Elves");
    t.answer_choose(P0, &[Entity::Object(giant)]);
    cast_new(&mut t, P0, "Cytoshape", &[Entity::Object(bears)]);
    t.resolve_all();
    assert_eq!(t.obj_now(bears).chars.name, "Hill Giant");
    // The later one wins while both apply (timestamp order, CR 613.7).
    t.answer_choose(P0, &[Entity::Object(elves)]);
    cast_new(&mut t, P0, "Cytoshape", &[Entity::Object(bears)]);
    t.resolve_all();
    assert_eq!(t.obj_now(bears).chars.name, "Llanowar Elves");
    next_turn(&mut t);
    assert_eq!(t.obj_now(bears).chars.name, "Grizzly Bears");
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn cemetery_puca_copies_printed_values_plus_copy_effects_and_keeps_its_counters() {
    cr!("707.2", "707.3", "603.10a", "122.1a");
    ruling!(
        "Cemetery Puca",
        "Cemetery Puca copies the printed values of the creature, plus any copy effects that have been applied to it. It won’t copy other effects that have changed the creature’s power, toughness, types, color, or so on. It also won’t copy counters on the creature (it’ll just retain the counters it already has)."
    );
    // "Whenever a creature dies, you may pay {1}. If you do, this creature becomes a copy
    // of that creature, except it has this ability."
    supported("Cemetery Puca");
    let mut t = TestGame::new(2);
    let puca = t.battlefield(P0, "Cemetery Puca");
    plus1(&mut t, puca, 1);
    t.lands(P0, "Wastes", 1);
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_choose(P1, &[Entity::Object(giant)]);
    let clone = t.enter(P1, "Clone");
    dress_up(&mut t, clone);
    t.answer_yes(P0, true);
    t.g.destroy(t.g.current(clone), None);
    t.resolve_all();
    let o = t.obj_now(puca);
    assert_eq!(o.chars.name, "Hill Giant");
    assert!(o.chars.colors.contains(Color::Red) && !o.chars.colors.contains(Color::Green));
    assert_eq!(t.counters(puca, counters::PLUS1), 1);
    assert_eq!(t.pt(puca), (4, 4));
}

#[test]
fn cemetery_puca_can_copy_a_creature_with_protection() {
    cr!("707.2", "115.1", "702.16b");
    ruling!(
        "Cemetery Puca",
        "Cemetery Puca’s ability isn’t targeted. It can copy a creature with shroud or protection."
    );
    // White Knight has protection from black; Cemetery Puca is blue and black.
    supported("White Knight");
    let mut t = TestGame::new(2);
    let puca = t.battlefield(P0, "Cemetery Puca");
    t.lands(P0, "Wastes", 1);
    let knight = t.battlefield(P1, "White Knight");
    t.answer_yes(P0, true);
    t.g.destroy(knight, None);
    t.resolve_all();
    assert_eq!(t.obj_now(puca).chars.name, "White Knight");
}

#[test]
fn clone_copies_only_what_was_printed() {
    cr!("707.2", "707.3");
    ruling!(
        "Clone",
        "Clone copies exactly what was printed on the original creature and nothing else (unless that creature is copying something else or is a token; see below). It doesn't copy whether that creature is tapped or untapped, whether it has any counters on it or Auras and Equipment attached to it, or any non-copy effects that have changed its power, toughness, types, color, or so on."
    );
    supported("Clone");
    supported("Holy Strength");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    crate::r_s06_common::attach_new(&mut t, P1, "Holy Strength", giant);
    dress_up(&mut t, giant);
    t.answer_choose(P0, &[Entity::Object(giant)]);
    let c = t.enter(P0, "Clone");
    t.settle();
    let o = t.obj_now(c);
    assert_eq!(o.chars.name, "Hill Giant");
    assert!(o.chars.colors.contains(Color::Red) && !o.chars.colors.contains(Color::Green));
    assert_eq!(t.pt(c), (3, 3));
    assert!(fresh(&t, c));
}

#[test]
fn clones_choice_isnt_targeted() {
    cr!("707.2", "115.1", "702.11b");
    ruling!("Clone", "Clone's ability doesn't target the chosen creature.");
    // Gladecover Scout has hexproof: it can't be the target of the opponent's spells or
    // abilities, but Clone can copy it.
    supported("Gladecover Scout");
    let mut t = TestGame::new(2);
    let scout = t.battlefield(P1, "Gladecover Scout");
    t.answer_choose(P0, &[Entity::Object(scout)]);
    lands_for_cost(&mut t, P0, "Clone");
    let clone = t.hand(P0, "Clone");
    t.cast(P0, clone).go();
    t.resolve_all();
    let now = t.g.current(clone);
    assert!(t.on_battlefield(now));
    assert_eq!(t.obj_now(now).chars.name, "Gladecover Scout");
}

#[test]
fn augmenter_pugilist_marked_damage_becomes_lethal_when_lands_drop_below_eight() {
    cr!("704.5g", "120.6", "613.4c");
    ruling!(
        "Augmenter Pugilist // Echoing Equation",
        "Because damage remains marked on creatures until the damage is removed as the turn ends, nonlethal damage dealt to Augmenter Pugilist may become lethal if the number of lands you control falls below eight."
    );
    // "As long as you control eight or more lands, this creature gets +5/+5." (3/3)
    supported("Augmenter Pugilist // Echoing Equation");
    let mut t = TestGame::new(2);
    let lands = t.lands(P0, "Forest", 8);
    let pug = t.battlefield(P0, "Augmenter Pugilist // Echoing Equation");
    assert_eq!(t.pt(pug), (8, 8));
    let src = t.battlefield(P1, "Grizzly Bears");
    damage(&mut t, src, 5, pug);
    assert!(t.on_battlefield(pug));
    t.g.destroy(lands[0], None);
    t.settle();
    assert!(!t.on_battlefield(pug));
    assert!(t.g.player(P0).graveyard.contains(&t.g.current(pug)));
}
