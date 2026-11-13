//! CR 613 audit: layer 1a timestamps between a static and a resolved copy effect, layer
//! 3 text changes feeding later layers (CR 613.1c, 612), a resolved effect's locked set
//! of objects (CR 611.2c) against a static one, and a creature turned into a land losing
//! its creature types (CR 205.3d) until the effect ends.

use crate::r613_audit_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// Casts Metamorphic Alteration on `host`, choosing `chosen` as it enters.
fn alter(t: &mut TestGame, p: PlayerId, host: ObjectId, chosen: ObjectId) -> ObjectId {
    t.answer_choose(p, &[Entity::Object(chosen)]);
    cast_resolve(t, p, "Metamorphic Alteration", &[Entity::Object(host)])
}

#[test]
fn a_static_copy_effect_and_a_resolved_one_apply_in_timestamp_order_in_layer_1() {
    // CR 613.2a: copy effects apply in timestamp order within layer 1a. Metamorphic
    // Alteration's effect has the Aura's timestamp (613.7a, 613.7e); Mirrorweave's has the
    // time it resolved (613.7b). The later one determines the copiable values.
    cr!("613.2a", "613.2c", "613.7", "613.7a", "613.7b", "613.7e", "707.2");
    // The Aura first, Mirrorweave later: an Elves copy until end of turn, then the Giant.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let elves = t.battlefield(P1, "Llanowar Elves");
    let aura = alter(&mut t, P0, bears, giant);
    t.g.recompute();
    assert_eq!(name(&t, bears), "Hill Giant");
    assert_eq!(t.pt(bears), (3, 3));
    cast_resolve(&mut t, P1, "Mirrorweave", &[Entity::Object(elves)]);
    t.g.recompute();
    assert_eq!(name(&t, bears), "Llanowar Elves");
    assert_eq!(t.pt(bears), (1, 1));
    assert_eq!(colors(&t, bears), color_set(&[Color::Green]));
    assert_eq!(name(&t, giant), "Llanowar Elves");
    assert_eq!(t.obj_now(aura).attached_to, Some(Entity::Object(bears)));
    end_turn(&mut t);
    assert_eq!(name(&t, bears), "Hill Giant");
    assert_eq!(t.pt(bears), (3, 3));
    assert_eq!(name(&t, giant), "Hill Giant");
    // Mirrorweave first, the Aura later: the Aura's copy wins at once. The chosen
    // creature entered after Mirrorweave resolved, so it isn't in Mirrorweave's locked
    // set (CR 611.2c) and its copiable values are its own.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let elves = t.battlefield(P1, "Llanowar Elves");
    cast_resolve(&mut t, P1, "Mirrorweave", &[Entity::Object(giant)]);
    t.g.recompute();
    assert_eq!(name(&t, bears), "Hill Giant");
    assert_eq!(name(&t, elves), "Hill Giant");
    let angel = t.battlefield(P1, "Serra Angel");
    t.g.recompute();
    assert_eq!(name(&t, angel), "Serra Angel");
    alter(&mut t, P0, bears, angel);
    t.g.recompute();
    assert_eq!(name(&t, bears), "Serra Angel");
    assert_eq!(t.pt(bears), (4, 4));
    assert!(has_kw(&t, bears, KeywordKind::Flying));
    assert_eq!(name(&t, elves), "Hill Giant", "Mirrorweave's copy still applies to the Elves");
    end_turn(&mut t);
    assert_eq!(name(&t, bears), "Serra Angel");
    assert_eq!(name(&t, elves), "Llanowar Elves");
}

#[test]
fn a_text_change_in_layer_3_feeds_type_changes_in_layer_4() {
    // Mind Bend replaces "Mountain" with "Forest" in Blood Moon's text (layer 3, CR 612),
    // so nonbasic lands become Forests (layer 4) with the Forest's mana ability.
    cr!("613.1c", "613.1d", "612.1", "305.7");
    ruling!(
        "Blood Moon",
        "Nonbasic lands will lose any other land types and abilities they had"
    );
    let mut t = TestGame::new(2);
    let moon = t.battlefield(P1, "Blood Moon");
    let mine = t.battlefield(P0, "Urza's Mine");
    t.g.recompute();
    assert!(has_subtype(&t, mine, "Mountain"));
    // Options: the five colors, then Plains, Island, Swamp, Mountain, Forest; then the
    // other words of the same kind (Plains, Island, Swamp, Forest).
    t.answer(P0, DecisionKind::Option, Answer::Index(8));
    t.answer(P0, DecisionKind::Option, Answer::Index(3));
    cast_resolve(&mut t, P0, "Mind Bend", &[Entity::Object(moon)]);
    t.g.recompute();
    // The displayed text changes too (a plural keeps its "s").
    assert!(t.obj_now(moon).chars.rules_text.contains("Forests"));
    assert!(!t.obj_now(moon).chars.rules_text.contains("Mountain"));
    assert!(has_ability_text(&t, moon, "are Forests"));
    assert!(has_subtype(&t, mine, "Forest"));
    assert!(!has_subtype(&t, mine, "Mountain"));
    assert!(has_ability_text(&t, mine, "Add {G}"));
    assert!(!has_ability_text(&t, mine, "Add {R}"));
    assert!(!has_ability_text(&t, mine, "{C}"));
    // A nonbasic land entering later is a Forest too (CR 611.3a).
    let tower = t.battlefield(P0, "Reliquary Tower");
    t.g.recompute();
    assert!(has_subtype(&t, tower, "Forest"));
    // The displayed plural: "Plains" is its own plural, so "Islands" becomes "Plains"; a
    // word the basic land type only starts isn't changed; landwalk is.
    use mtg_engine::text_change::replace_words;
    assert_eq!(replace_words("Islands you control", "Island", "Plains"), "Plains you control");
    assert_eq!(replace_words("Mountains", "Mountain", "Forest"), "Forests");
    assert_eq!(replace_words("Mountainside", "Mountain", "Forest"), "Mountainside");
    assert_eq!(replace_words("Forestwalk", "Forest", "Plains"), "Plainswalk");
    // Island Fish Jasconius's "When you control no Islands, sacrifice this creature" reads
    // "no Plains" and its state trigger watches Plains: losing the Plains sacrifices it.
    let mut t = TestGame::new(2);
    let fish = t.battlefield(P0, "Island Fish Jasconius");
    let plains = t.battlefield(P0, "Plains");
    t.battlefield(P0, "Island");
    // Island, then Plains among the other basic land types (Plains, Swamp, Mountain, Forest).
    t.answer(P0, DecisionKind::Option, Answer::Index(6));
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    cast_resolve(&mut t, P0, "Mind Bend", &[Entity::Object(fish)]);
    t.g.recompute();
    let text = t.obj_now(fish).chars.rules_text.to_string();
    assert!(text.contains("no Plains,"), "{text}");
    assert!(!text.contains("Island"), "{text}");
    assert!(t.on_battlefield(fish), "a Plains is controlled");
    t.g.destroy(plains, None);
    t.settle();
    t.resolve_all();
    assert!(!t.on_battlefield(fish), "no Plains: sacrificed");
}

#[test]
fn a_text_change_in_layer_3_feeds_color_conditions_in_layer_7() {
    // Crusade's "White creatures get +1/+1" reads "Black creatures" after Mind Bend; the
    // Aura Aspect of Wolf counts Islands instead of Forests after its text changes
    // (an Aura's own characteristics changing what it does to the enchanted creature).
    cr!("613.1c", "613.4c", "612.1", "613.9", "611.3a");
    let mut t = TestGame::new(2);
    let crusade = t.battlefield(P0, "Crusade");
    let knight = t.battlefield(P0, "White Knight");
    let black = t.battlefield(P1, "Black Knight");
    t.g.recompute();
    assert_eq!(t.pt(knight), (3, 3));
    assert_eq!(t.pt(black), (2, 2));
    // White, then Black among the other colors (Blue, Black, Red, Green).
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    cast_resolve(&mut t, P0, "Mind Bend", &[Entity::Object(crusade)]);
    t.g.recompute();
    assert_eq!(name(&t, crusade), "Crusade");
    assert_eq!(t.pt(knight), (2, 2));
    assert_eq!(t.pt(black), (3, 3));
    // Aspect of Wolf: +X/+Y for half the Forests you control, rounded down and up.
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 3);
    t.lands(P0, "Island", 1);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let aspect = t.battlefield(P0, "Aspect of Wolf");
    assert!(t.g.attach(aspect, Entity::Object(bears)));
    t.g.recompute();
    assert_eq!(t.pt(bears), (3, 4));
    // Forest, then Island among the other basic land types (Plains, Island, Swamp,
    // Mountain).
    t.answer(P0, DecisionKind::Option, Answer::Index(9));
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    // (Casting Mind Bend added an Island for its cost: two Islands now.)
    cast_resolve(&mut t, P0, "Mind Bend", &[Entity::Object(aspect)]);
    t.g.recompute();
    assert!(t.obj_now(aspect).chars.rules_text.contains("Islands"));
    assert_eq!(t.pt(bears), (3, 3), "two Islands: +1/+1");
    t.lands(P0, "Island", 1);
    t.g.recompute();
    assert_eq!(t.pt(bears), (3, 4), "three Islands: +1/+2");
    t.lands(P0, "Forest", 2);
    t.g.recompute();
    assert_eq!(t.pt(bears), (3, 4), "Forests no longer count");
}

#[test]
fn a_resolved_animation_is_locked_to_the_artifacts_it_applied_to() {
    // Tezzeret's "artifacts you control become artifact creatures with base power and
    // toughness 5/5 until end of turn" applies to the artifacts there as it resolved
    // (CR 611.2c), unlike a static ability's effect (CR 611.3a); an artifact land among
    // them is a 5/5 creature land.
    cr!("611.2c", "611.3a", "613.6", "613.1d", "613.4b", "208.3");
    let mut t = TestGame::new(2);
    let tezzeret = t.battlefield(P0, "Tezzeret the Seeker");
    let stone = t.battlefield(P0, "Mind Stone");
    let citadel = t.battlefield(P0, "Darksteel Citadel");
    let theirs = t.battlefield(P1, "Mind Stone");
    put_counters(&mut t, tezzeret, counters::LOYALTY, 1);
    assert_eq!(t.obj_now(tezzeret).loyalty(), 5);
    t.activate(P0, tezzeret, 2, &[]).expect("the -5 ability");
    t.resolve_all();
    t.g.recompute();
    assert!(t.obj_now(stone).is_creature());
    assert_eq!(t.pt(stone), (5, 5));
    assert!(t.obj_now(citadel).is_creature());
    assert_eq!(t.pt(citadel), (5, 5));
    assert!(is_type(&t, citadel, CardType::Land));
    assert!(has_kw(&t, citadel, KeywordKind::Indestructible));
    assert!(!t.obj_now(theirs).is_creature(), "\"you control\"");
    // An artifact entering afterwards isn't affected.
    let later = t.battlefield(P0, "Mind Stone");
    t.g.recompute();
    assert!(!t.obj_now(later).is_creature());
    assert!(t.obj_now(later).chars.power.is_none());
    end_turn(&mut t);
    assert!(!t.obj_now(stone).is_creature());
    assert!(t.obj_now(stone).chars.power.is_none());
}

#[test]
fn a_creature_turned_into_a_land_loses_its_creature_types_until_the_effect_ends() {
    // Imprisoned in the Moon: a colorless land with "{T}: Add {C}" that loses all other
    // card types and abilities. With the creature type removed its creature subtypes are
    // removed too (CR 205.1a), it has no power or toughness (CR 208.3), and "creatures you
    // control are Goblins" no longer applies to it; everything comes back when the Aura
    // leaves.
    cr!("613.1d", "613.1e", "613.1f", "205.1a", "208.3", "611.3a");
    ruling!(
        "Imprisoned in the Moon",
        "The permanent will keep any supertypes it previously had"
    );
    let mut t = TestGame::new(2);
    choose_creature_type(&mut t, P0, "Goblin");
    t.enter(P0, "Arcane Adaptation");
    t.resolve_all();
    let isamaru = t.battlefield(P0, "Isamaru, Hound of Konda");
    put_counters(&mut t, isamaru, counters::PLUS1, 1);
    t.g.recompute();
    assert!(has_subtype(&t, isamaru, "Goblin") && has_subtype(&t, isamaru, "Dog"));
    let moon = cast_resolve(&mut t, P0, "Imprisoned in the Moon", &[Entity::Object(isamaru)]);
    t.g.recompute();
    assert!(is_type(&t, isamaru, CardType::Land));
    assert!(!is_type(&t, isamaru, CardType::Creature));
    assert!(!has_subtype(&t, isamaru, "Dog"));
    assert!(!has_subtype(&t, isamaru, "Goblin"));
    assert!(t.obj_now(isamaru).chars.is_legendary(), "supertypes stay");
    assert_eq!(colors(&t, isamaru), ColorSet::NONE);
    assert!(t.obj_now(isamaru).chars.power.is_none());
    assert_eq!(ability_count(&t, isamaru), 1);
    assert!(has_ability_text(&t, isamaru, "Add {C}"));
    assert_eq!(t.counters(isamaru, counters::PLUS1), 1, "counters stay");
    cast_resolve(&mut t, P0, "Disenchant", &[Entity::Object(moon)]);
    t.g.recompute();
    assert!(t.obj_now(isamaru).is_creature());
    assert!(!is_type(&t, isamaru, CardType::Land));
    assert!(has_subtype(&t, isamaru, "Dog") && has_subtype(&t, isamaru, "Goblin"));
    assert_eq!(colors(&t, isamaru), color_set(&[Color::White]));
    assert_eq!(t.pt(isamaru), (3, 3));
    assert_eq!(ability_count(&t, isamaru), 0);
}
