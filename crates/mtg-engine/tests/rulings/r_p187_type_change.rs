//! Rulings batch P187 — type-changing spells and Auras that set base power and toughness
//! and remove abilities: setting effects apply in timestamp order in layer 7b while
//! modifying effects, counters and switches apply whenever they started (CR 613.4, 613.7);
//! abilities gained later are kept (CR 613.1f, 613.7); damage stays marked, so a smaller
//! toughness can make it lethal (CR 120.6, 704.5g); an effect that started applying keeps
//! applying in later layers even if its ability is removed (CR 613.6).

use crate::r_s01_common::supported;
use crate::r_s06_common::{attach_new, damage, has_kw};
use crate::r_s09_common::{legal_attack, to_combat};
use crate::r_s21_common::legal_blocks;
use crate::r_s28_common::cast_card;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// `p` casts `name` targeting `target` (lands for it are added) and it resolves.
fn cast_at(t: &mut TestGame, p: PlayerId, name: &str, target: ObjectId) {
    t.answer_targets(p, &[Entity::Object(target)]);
    cast_card(t, p, name);
    t.resolve_all();
}

fn plus1(t: &mut TestGame, id: ObjectId, n: u32) {
    t.g.add_counters(Entity::Object(id), "+1/+1", n, None);
    t.g.recompute();
}

/// Applies the type-changing card `name` (an Aura, attached by P0; or an instant or
/// sorcery, cast by P0) to `target`.
fn apply(t: &mut TestGame, name: &str, target: ObjectId) {
    let c = mtg_engine::card::card(name);
    if c.front().chars.card_types.contains(CardType::Enchantment) {
        attach_new(t, P0, name, target);
    } else {
        cast_at(t, P0, name, target);
    }
}

/// An earlier base power and toughness setting effect (Wings of Velis Vel, 4/4) is
/// overwritten by `name` (which sets base `pt`); a later one (Wings again) overwrites it.
fn overwrites_earlier_setting_effects_only(name: &str, pt: (i32, i32)) {
    supported(name);
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    cast_at(&mut t, P0, "Wings of Velis Vel", giant);
    assert_eq!(t.pt(giant), (4, 4));
    apply(&mut t, name, giant);
    let giant = t.g.current(giant);
    assert_eq!(t.pt(giant), pt, "{name}");
    cast_at(&mut t, P0, "Wings of Velis Vel", giant);
    assert_eq!(t.pt(giant), (4, 4), "{name}: a later setting effect wins");
}

#[test]
fn coerced_to_kill_overwrites_earlier_setting_effects() {
    cr!("613.4b", "613.7", "613.7e");
    ruling!(
        "Coerced to Kill",
        "Coerced to Kill overwrites all previous effects that set the creature's base power and toughness to specific values."
    );
    overwrites_earlier_setting_effects_only("Coerced to Kill", (1, 1));
}

#[test]
fn deep_freeze_overwrites_earlier_setting_effects() {
    cr!("613.4b", "613.7", "613.7e");
    ruling!(
        "Deep Freeze",
        "Deep Freeze overwrites all previous effects that set the creature’s base power and toughness to specific values."
    );
    overwrites_earlier_setting_effects_only("Deep Freeze", (0, 4));
}

#[test]
fn eaten_by_piranhas_overwrites_earlier_setting_effects() {
    cr!("613.4b", "613.7", "613.7e");
    ruling!(
        "Eaten by Piranhas",
        "Eaten by Piranhas overwrites all previous effects that set the creature's base power and toughness to specific values."
    );
    overwrites_earlier_setting_effects_only("Eaten by Piranhas", (1, 1));
}

#[test]
fn eye_of_nidhogg_overwrites_earlier_setting_effects() {
    cr!("613.4b", "613.7", "613.7e");
    ruling!(
        "Eye of Nidhogg",
        "Eye of Nidhogg overwrites all previous effects that set the creature's base power and toughness to specific values."
    );
    overwrites_earlier_setting_effects_only("Eye of Nidhogg", (4, 2));
}

#[test]
fn frogify_overwrites_earlier_setting_effects() {
    cr!("613.4b", "613.7", "613.7e");
    ruling!(
        "Frogify",
        "Frogify overwrites all previous effects that set the creature's base power and toughness to specific values."
    );
    overwrites_earlier_setting_effects_only("Frogify", (1, 1));
}

#[test]
fn gift_of_tusks_overwrites_earlier_setting_effects() {
    cr!("613.4b", "613.7", "613.7b");
    ruling!(
        "Gift of Tusks",
        "Gift of Tusks overwrites all previous effects that set the creature’s base power and toughness to specific values."
    );
    overwrites_earlier_setting_effects_only("Gift of Tusks", (3, 3));
}

#[test]
fn ichthyomorphosis_overwrites_earlier_setting_effects() {
    cr!("613.4b", "613.7", "613.7e");
    ruling!(
        "Ichthyomorphosis",
        "Ichthyomorphosis overwrites all previous effects that set the creature’s base power and toughness to specific values."
    );
    overwrites_earlier_setting_effects_only("Ichthyomorphosis", (0, 1));
}

/// A +1/+1 counter, Sure Strike (+3/+0) and Twisted Image (a switch) applied before `name`
/// still apply afterwards, as does a later Giant Growth.
fn modifiers_counters_and_switches_still_apply(name: &str, pt: (i32, i32), switch: bool) {
    supported(name);
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    plus1(&mut t, giant, 1);
    cast_at(&mut t, P1, "Sure Strike", giant);
    if switch {
        cast_at(&mut t, P1, "Twisted Image", giant);
    }
    apply(&mut t, name, giant);
    let giant = t.g.current(giant);
    let (p, tt) = (pt.0 + 1 + 3, pt.1 + 1);
    let expect = if switch { (tt, p) } else { (p, tt) };
    assert_eq!(t.pt(giant), expect, "{name}");
    cast_at(&mut t, P1, "Giant Growth", giant);
    let expect = (expect.0 + 3, expect.1 + 3);
    assert_eq!(t.pt(giant), expect, "{name}: a later pump");
}

#[test]
fn coerced_to_kill_keeps_modifiers_counters_and_switches() {
    cr!("613.4b", "613.4c", "613.4d");
    ruling!(
        "Coerced to Kill",
        "Effects that modify a creature's power and/or toughness, such as the effect of Auspicious Arrival, will apply to the creature no matter when they started to take effect."
    );
    modifiers_counters_and_switches_still_apply("Coerced to Kill", (1, 1), true);
}

#[test]
fn frogify_keeps_modifiers_and_counters() {
    cr!("613.4b", "613.4c");
    ruling!(
        "Frogify",
        "Effects that modify a creature's power and/or toughness, such as the effect of Dead Weight, will apply to the creature no matter when they started to take effect."
    );
    modifiers_counters_and_switches_still_apply("Frogify", (1, 1), false);
}

#[test]
fn ichthyomorphosis_keeps_modifiers_and_counters() {
    cr!("613.4b", "613.4c");
    ruling!(
        "Ichthyomorphosis",
        "Effects that modify a creature’s power and/or toughness will apply to the creature no matter when they started to take effect."
    );
    modifiers_counters_and_switches_still_apply("Ichthyomorphosis", (0, 1), false);
}

#[test]
fn noggle_the_mind_keeps_modifiers_counters_and_switches() {
    cr!("613.4b", "613.4c", "613.4d");
    ruling!(
        "Noggle the Mind",
        "Effects that modify the creature's power and/or toughness, such as the effect of Appeal to Eirdu, will apply to the creature no matter when they started to take effect."
    );
    modifiers_counters_and_switches_still_apply("Noggle the Mind", (1, 1), true);
}

#[test]
fn scale_up_keeps_modifiers_and_counters_also_overloaded() {
    cr!("613.4b", "613.4c", "702.96a");
    ruling!(
        "Scale Up",
        "Effects that modify the affected creatures' power or toughness (such as the effects of Force of Virtue or Giant Growth) will apply no matter when they started to take effect."
    );
    // "Until end of turn, target creature you control becomes a green Wurm with base power
    // and toughness 6/4."
    supported("Scale Up");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    plus1(&mut t, giant, 1);
    cast_at(&mut t, P0, "Sure Strike", giant);
    cast_at(&mut t, P0, "Scale Up", giant);
    assert_eq!(t.pt(giant), (10, 5));
    assert!(t.obj_now(giant).chars.has_subtype("Wurm"));
    cast_at(&mut t, P0, "Giant Growth", giant);
    assert_eq!(t.pt(giant), (13, 8));
    // Overloaded: each creature you control.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Hill Giant");
    let b = t.battlefield(P0, "Grizzly Bears");
    plus1(&mut t, b, 2);
    crate::r_s25_common::lands_for_cost(&mut t, P0, "Scale Up");
    t.lands(P0, "Forest", 6);
    let card = t.hand(P0, "Scale Up");
    t.cast(P0, card)
        .method(mtg_engine::object::CastMethod::Keyword(
            KeywordKind::Overload,
        ))
        .go();
    t.resolve_all();
    assert_eq!(t.pt(a), (6, 4));
    assert_eq!(t.pt(b), (8, 6));
}

/// A creature with 2 damage marked on it (Hill Giant, 3/3) dies once `name` makes its
/// toughness 2 or less.
fn marked_damage_becomes_lethal(name: &str) {
    supported(name);
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let src = t.battlefield(P0, "Grizzly Bears");
    damage(&mut t, src, 2, giant);
    assert!(t.on_battlefield(giant));
    apply(&mut t, name, giant);
    t.settle();
    assert!(t.in_graveyard(P1, "Hill Giant"), "{name}");
}

#[test]
fn eye_of_nidhogg_can_make_marked_damage_lethal_attached_or_unattached() {
    cr!("120.6", "704.5g", "613.4b");
    ruling!(
        "Eye of Nidhogg",
        "Because damage remains marked on a creature until the damage is removed as the turn ends, nonlethal damage dealt to a creature may become lethal if Eye of Nidhogg becomes attached to it or unattached from it during that turn."
    );
    marked_damage_becomes_lethal("Eye of Nidhogg");
    // Unattached: a 1/1 that was a 4/2 with 1 damage marked.
    let mut t = TestGame::new(2);
    let elves = t.battlefield(P1, "Llanowar Elves");
    let eye = attach_new(&mut t, P0, "Eye of Nidhogg", elves);
    let src = t.battlefield(P0, "Grizzly Bears");
    damage(&mut t, src, 1, elves);
    assert!(t.on_battlefield(elves));
    t.g.unattach(eye);
    t.g.recompute();
    t.settle();
    assert!(t.in_graveyard(P1, "Llanowar Elves"));
}

#[test]
fn frogify_can_make_marked_damage_lethal() {
    cr!("120.6", "704.5g", "613.4b");
    ruling!(
        "Frogify",
        "nonlethal damage dealt to a creature may become lethal if Frogify becomes attached to it during that turn."
    );
    marked_damage_becomes_lethal("Frogify");
}

#[test]
fn ichthyomorphosis_can_make_marked_damage_lethal() {
    cr!("120.6", "704.5g", "613.4b");
    ruling!(
        "Ichthyomorphosis",
        "nonlethal damage dealt to a creature may become lethal if Ichthyomorphosis becomes attached to it during that turn."
    );
    marked_damage_becomes_lethal("Ichthyomorphosis");
}

#[test]
fn kenriths_transformation_can_make_marked_damage_lethal() {
    cr!("120.6", "704.5g", "613.4b");
    ruling!(
        "Kenrith's Transformation",
        "nonlethal damage dealt to a creature may become lethal if Kenrith's Transformation becomes attached to it during that turn."
    );
    // Base 3/3: a 7/7 with 3 damage dies.
    supported("Kenrith's Transformation");
    let mut t = TestGame::new(2);
    let baloth = t.battlefield(P1, "Enormous Baloth");
    let src = t.battlefield(P0, "Grizzly Bears");
    damage(&mut t, src, 3, baloth);
    assert!(t.on_battlefield(baloth));
    attach_new(&mut t, P0, "Kenrith's Transformation", baloth);
    t.settle();
    assert!(t.in_graveyard(P1, "Enormous Baloth"));
}

#[test]
fn startling_development_can_make_marked_damage_lethal() {
    cr!("120.6", "704.5g", "613.4b");
    ruling!(
        "Startling Development",
        "nonlethal damage dealt to the target creature may become lethal once its base toughness becomes 4."
    );
    supported("Startling Development");
    let mut t = TestGame::new(2);
    let baloth = t.battlefield(P1, "Enormous Baloth");
    let src = t.battlefield(P0, "Grizzly Bears");
    damage(&mut t, src, 4, baloth);
    assert!(t.on_battlefield(baloth));
    cast_at(&mut t, P0, "Startling Development", baloth);
    assert!(t.in_graveyard(P1, "Enormous Baloth"));
}

/// After `name` applies to a creature, it gains flying (Jump) and keeps it.
fn keeps_abilities_gained_later(name: &str) {
    supported(name);
    supported("Jump");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Serra Angel");
    apply(&mut t, name, giant);
    let giant = t.g.current(giant);
    assert!(
        !has_kw(&t, giant, KeywordKind::Flying),
        "{name}: lost flying"
    );
    assert!(!has_kw(&t, giant, KeywordKind::Vigilance), "{name}");
    cast_at(&mut t, P0, "Jump", giant);
    assert!(
        has_kw(&t, giant, KeywordKind::Flying),
        "{name}: kept a later ability"
    );
    assert!(!has_kw(&t, giant, KeywordKind::Vigilance), "{name}");
}

#[test]
fn frogify_lets_the_creature_keep_abilities_gained_later() {
    cr!("613.1f", "613.7", "613.7e");
    ruling!(
        "Frogify",
        "If the affected creature gains an ability after Frogify becomes attached to it, it will keep that ability."
    );
    keeps_abilities_gained_later("Frogify");
}

#[test]
fn gift_of_tusks_lets_the_creature_keep_abilities_gained_later() {
    cr!("613.1f", "613.7", "613.7b");
    ruling!(
        "Gift of Tusks",
        "If the affected creature gains an ability after Gift of Tusks resolves, it will keep that ability."
    );
    keeps_abilities_gained_later("Gift of Tusks");
}

#[test]
fn ichthyomorphosis_lets_the_creature_keep_abilities_gained_later() {
    cr!("613.1f", "613.7", "613.7e");
    ruling!(
        "Ichthyomorphosis",
        "If the affected creature gains an ability after Ichthyomorphosis becomes attached to it, it will keep that ability."
    );
    keeps_abilities_gained_later("Ichthyomorphosis");
}

#[test]
fn kenriths_transformation_lets_the_creature_keep_abilities_gained_later() {
    cr!("613.1f", "613.7", "613.7e");
    ruling!(
        "Kenrith's Transformation",
        "If the affected creature gains an ability after Kenrith's Transformation becomes attached to it, it will keep that ability."
    );
    keeps_abilities_gained_later("Kenrith's Transformation");
}

#[test]
fn snakeform_lets_the_creature_keep_abilities_gained_later() {
    cr!("613.1f", "613.7", "613.7b");
    ruling!(
        "Snakeform",
        "If the affected creature gains an ability after Snakeform resolves, it will keep that ability."
    );
    keeps_abilities_gained_later("Snakeform");
}

#[test]
fn turn_to_frog_lets_the_creature_keep_abilities_gained_later() {
    cr!("613.1f", "613.7", "613.7b");
    ruling!(
        "Turn to Frog",
        "If the affected creature gains an ability after Turn to Frog resolves, it will keep that ability."
    );
    keeps_abilities_gained_later("Turn to Frog");
}

#[test]
fn snakeform_removes_changeling_and_its_creature_types() {
    cr!("702.73a", "613.1d", "613.1f", "613.6");
    ruling!(
        "Snakeform",
        "If Snakeform affects a creature with changeling, the creature will lose its changeling ability, and will no longer be all creature types."
    );
    let mut t = TestGame::new(2);
    let outcast = t.battlefield(P1, "Changeling Outcast");
    assert!(t.obj_now(outcast).chars.has_subtype("Goblin"));
    cast_at(&mut t, P0, "Snakeform", outcast);
    let o = t.obj_now(outcast);
    assert!(!o.has_keyword(KeywordKind::Changeling));
    assert!(o.chars.has_subtype("Snake"));
    assert!(!o.chars.has_subtype("Goblin"));
    assert!(!o.chars.has_subtype("Shapeshifter"));
    assert_eq!(t.pt(outcast), (1, 1));
    // Snakeform also draws a card.
    assert_eq!(t.hand_size(P0), 1);
}

#[test]
fn lignify_ends_when_it_leaves_but_other_effects_keep_applying() {
    cr!("613.4b", "613.4c", "611.3a");
    ruling!(
        "Lignify",
        "If Lignify leaves the battlefield, the creature goes back to being what it was before. Any effects that applied while the Lignify was on the battlefield, such as Giant Growth or Lace with Moonglove, will continue to apply."
    );
    supported("Lignify");
    let mut t = TestGame::new(2);
    let angel = t.battlefield(P1, "Serra Angel");
    let lignify = attach_new(&mut t, P0, "Lignify", angel);
    assert_eq!(t.pt(angel), (0, 4));
    assert!(t.obj_now(angel).chars.has_subtype("Treefolk"));
    assert!(!has_kw(&t, angel, KeywordKind::Flying));
    cast_at(&mut t, P1, "Giant Growth", angel);
    cast_at(&mut t, P1, "Jump", angel);
    assert_eq!(t.pt(angel), (3, 7));
    crate::r_s02_common::destroy(&mut t, lignify);
    let o = t.obj_now(angel);
    assert!(!o.chars.has_subtype("Treefolk"));
    assert!(o.chars.has_subtype("Angel"));
    assert!(o.has_keyword(KeywordKind::Vigilance));
    assert!(o.has_keyword(KeywordKind::Flying));
    assert_eq!(t.pt(angel), (7, 7));
}

#[test]
fn lignify_applies_an_earlier_switch_to_its_new_power_and_toughness() {
    cr!("613.4b", "613.4d");
    ruling!(
        "Lignify",
        "If a previous effect switched the creature's power and toughness, that effect still applies (though it will apply to the creature's new power and toughness)."
    );
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    cast_at(&mut t, P1, "Twisted Image", giant);
    attach_new(&mut t, P0, "Lignify", giant);
    assert_eq!(t.pt(giant), (4, 0));
    t.settle();
    assert!(t.in_graveyard(P1, "Hill Giant"));
}

#[test]
fn futurist_operatives_effect_has_the_timestamp_of_its_entering() {
    cr!("613.7a", "613.7d", "613.4b", "613.4c");
    ruling!(
        "Futurist Operative",
        "The timestamp of Futurist Operative's first ability is when it entered the battlefield, not when it became tapped."
    );
    supported("Futurist Operative");
    // "As long as this creature is tapped, it's a Human Citizen with base power and
    // toughness 1/1 and can't be blocked."
    let mut t = TestGame::new(2);
    let op = t.battlefield(P0, "Futurist Operative");
    plus1(&mut t, op, 1);
    t.g.tap(op);
    t.g.recompute();
    assert_eq!(t.pt(op), (2, 2));
    assert!(t.obj_now(op).chars.has_subtype("Citizen"));
    t.g.untap(op);
    t.g.recompute();
    assert_eq!(t.pt(op), (4, 5));
    // A setting effect after it entered wins even when it becomes tapped later.
    cast_at(&mut t, P0, "Wings of Velis Vel", op);
    assert_eq!(t.pt(op), (5, 5));
    t.g.tap(op);
    t.g.recompute();
    assert_eq!(t.pt(op), (5, 5));
    assert!(t.obj_now(op).chars.has_subtype("Citizen"));
}

#[test]
fn turn_to_frog_on_a_god_keeps_its_not_a_creature_type_change() {
    cr!("613.1d", "613.1f", "613.6");
    ruling!(
        "Turn to Frog",
        "The way continuous effects work, the God’s type-changing ability is applied before the effect that removes that ability is applied."
    );
    supported("Thassa, God of the Sea");
    // Devotion to blue 5: Thassa ({2}{U}) and two Air Elementals ({3}{U}{U}).
    let mut t = TestGame::new(2);
    let thassa = t.battlefield(P0, "Thassa, God of the Sea");
    let devotion = [
        t.battlefield(P0, "Air Elemental"),
        t.battlefield(P0, "Air Elemental"),
    ];
    t.g.recompute();
    assert!(t.obj_now(thassa).is(CardType::Creature));
    cast_at(&mut t, P1, "Turn to Frog", thassa);
    let o = t.obj_now(thassa);
    assert!(o.is(CardType::Creature));
    assert!(o.is(CardType::Enchantment));
    assert!(o.chars.is_legendary());
    assert!(o.chars.has_subtype("Frog"));
    assert!(o.chars.abilities.is_empty());
    assert_eq!(t.pt(thassa), (1, 1));
    // Devotion drops: it's no longer a creature, though its abilities are gone.
    crate::r_s02_common::destroy(&mut t, devotion[0]);
    let o = t.obj_now(thassa);
    assert!(!o.is(CardType::Creature));
    assert!(o.is(CardType::Enchantment));
    assert!(o.chars.abilities.is_empty());
    assert_eq!(o.chars.colors, ColorSet::single(Color::Blue));
}

#[test]
fn graaz_losing_its_abilities_keeps_its_type_and_pt_effect_on_others() {
    cr!("613.6", "613.1d", "613.1f", "613.4b", "508.1d");
    ruling!(
        "Graaz, Unstoppable Juggernaut",
        "If Graaz loses its abilities for some reason, then Juggernauts you control will not have to attack each combat and will be able to be blocked by Walls."
    );
    supported("Graaz, Unstoppable Juggernaut");
    let mut t = TestGame::new(2);
    let graaz = t.battlefield(P0, "Graaz, Unstoppable Juggernaut");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.recompute();
    assert_eq!(t.pt(bears), (5, 3));
    assert!(t.obj_now(bears).chars.has_subtype("Juggernaut"));
    to_combat(&mut t, P0);
    assert!(!legal_attack(&mut t, &[]), "Juggernauts must attack");
    // Graaz loses all abilities until end of turn.
    t.set_step(P0, mtg_engine::turn::Step::PrecombatMain);
    cast_at(&mut t, P1, "Turn to Frog", graaz);
    assert!(t.obj_now(graaz).chars.abilities.is_empty());
    assert_eq!(t.pt(bears), (5, 3));
    assert!(t.obj_now(bears).chars.has_subtype("Juggernaut"));
    assert!(t.obj_now(bears).chars.has_subtype("Bear"));
    to_combat(&mut t, P0);
    assert!(legal_attack(&mut t, &[]), "no attack requirement any more");
    // A Wall can block it.
    let wall = t.battlefield(P1, "Wall of Stone");
    crate::r_s01_common::attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    assert!(legal_blocks(&mut t, P1, &[(wall, bears)]));
}
