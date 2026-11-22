//! CR 613 audit: layer 1 copy effects (CR 613.2a, 707) against the later layers: copies
//! of permanents affected by type-, color- and ability-changing effects, copy exceptions
//! (CR 707.9), characteristic-defining abilities of the copied object (CR 707.9d),
//! face-down and transformed originals.

use crate::r613_audit_common::*;
use mtg_engine::decision::Decision;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// Enters `card` under `p`'s control as a copy of `of`.
fn enter_as_copy_of(t: &mut TestGame, p: PlayerId, card: &str, of: ObjectId) -> ObjectId {
    t.answer_yes(p, true);
    t.answer_choose(p, &[Entity::Object(of)]);
    let id = t.enter(p, card);
    t.resolve_all();
    t.g.recompute();
    id
}

#[test]
fn a_clone_of_an_animated_land_is_just_the_land() {
    // The animating effect (layers 4, 6 and 7b) isn't part of the copiable values, so
    // the Clone enters as a noncreature land with the land's printed abilities.
    cr!("707.2", "613.1a", "613.2a", "613.2c", "613.1d", "208.3");
    ruling!(
        "Clone",
        "any non-copy effects that have changed its power, toughness, types, color, or so on"
    );
    let mut t = TestGame::new(2);
    let nexus = t.battlefield(P1, "Inkmoth Nexus");
    t.lands(P1, "Wastes", 1);
    activate_containing(&mut t, P1, nexus, "becomes", &[]);
    t.g.recompute();
    assert!(t.obj_now(nexus).is_creature());
    let clone = enter_as_copy_of(&mut t, P0, "Clone", nexus);
    assert_eq!(name(&t, clone), "Inkmoth Nexus");
    assert!(is_type(&t, clone, CardType::Land));
    assert!(!t.obj_now(clone).is_creature());
    assert!(!is_type(&t, clone, CardType::Artifact));
    assert!(!has_kw(&t, clone, KeywordKind::Flying));
    assert!(t.obj_now(clone).chars.power.is_none());
    assert!(has_ability_text(&t, clone, "Add {C}"));
    assert!(t.on_battlefield(clone), "a noncreature isn't a 0/0 that dies");
    // Its copied animation ability works on the copy itself.
    t.lands(P0, "Wastes", 1);
    activate_containing(&mut t, P0, clone, "becomes", &[]);
    t.g.recompute();
    assert!(t.obj_now(clone).is_creature());
    assert!(has_subtype(&t, clone, "Blinkmoth"));
    assert!(has_kw(&t, clone, KeywordKind::Infect));
    assert!(has_kw(&t, clone, KeywordKind::Flying));
    assert_eq!(t.pt(clone), (1, 1));
}

#[test]
fn mirrorweave_on_an_animated_land_turns_the_other_creatures_into_unanimated_lands() {
    // Inkmoth Nexus stands in for the ruling's Mutavault (the card data has Mutavault only
    // as a token printing).
    cr!("707.2", "613.1a", "613.2c", "613.1d", "208.3");
    ruling!("Mirrorweave", "Each other creature will become an unanimated Mutavault");
    let mut t = TestGame::new(2);
    let nexus = t.battlefield(P1, "Inkmoth Nexus");
    t.lands(P1, "Wastes", 1);
    activate_containing(&mut t, P1, nexus, "becomes", &[]);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let drake = t.battlefield(P1, "Wind Drake");
    cast_resolve(&mut t, P0, "Mirrorweave", &[Entity::Object(nexus)]);
    t.g.recompute();
    for c in [bears, drake] {
        assert_eq!(name(&t, c), "Inkmoth Nexus");
        assert!(is_type(&t, c, CardType::Land));
        assert!(!t.obj_now(c).is_creature());
        assert!(t.obj_now(c).chars.power.is_none());
        assert!(!has_kw(&t, c, KeywordKind::Flying));
        assert!(has_ability_text(&t, c, "Add {C}"));
        assert!(t.on_battlefield(c));
    }
    // The animated Nexus itself is unchanged ("each other creature").
    assert!(t.obj_now(nexus).is_creature());
    assert_eq!(t.pt(nexus), (1, 1));
    end_turn(&mut t);
    assert_eq!(name(&t, bears), "Grizzly Bears");
    assert_eq!(t.pt(bears), (2, 2));
    assert!(has_kw(&t, drake, KeywordKind::Flying));
}

#[test]
fn a_clone_copies_the_printed_values_of_an_ovinized_creature_and_humility_applies_to_both() {
    // Layer 6 and 7 effects on the original aren't copiable (CR 707.2). The copy is then
    // affected by the static Humility (current set, CR 611.3a) but not by the Ovinize
    // that was locked to the original (CR 611.2c).
    cr!("707.2", "613.2c", "613.1a", "613.1f", "613.4b", "611.2c", "611.3a");
    ruling!(
        "Clone",
        "any non-copy effects that have changed its power, toughness, types, color, or so on"
    );
    let mut t = TestGame::new(2);
    let angel = t.battlefield(P1, "Serra Angel");
    cast_resolve(&mut t, P1, "Ovinize", &[Entity::Object(angel)]);
    t.g.recompute();
    assert_eq!(t.pt(angel), (0, 1));
    let clone = enter_as_copy_of(&mut t, P0, "Clone", angel);
    assert_eq!(name(&t, clone), "Serra Angel");
    assert_eq!(t.pt(clone), (4, 4));
    assert!(has_kw(&t, clone, KeywordKind::Flying));
    assert!(has_kw(&t, clone, KeywordKind::Vigilance));
    assert_eq!(colors(&t, clone), color_set(&[Color::White]));
    let humility = t.battlefield(P1, "Humility");
    t.g.recompute();
    assert_eq!(t.pt(clone), (1, 1));
    assert_eq!(ability_count(&t, clone), 0);
    assert_eq!(t.pt(angel), (1, 1), "Humility's later base 1/1 overrides Ovinize's 0/1");
    // Humility leaves: the copy is a 4/4 with flying and vigilance again; the original
    // is still Ovinized (0/1) this turn.
    t.g.destroy(humility, None);
    t.settle();
    t.g.recompute();
    assert_eq!(t.pt(clone), (4, 4));
    assert!(has_kw(&t, clone, KeywordKind::Flying));
    assert_eq!(t.pt(angel), (0, 1));
    assert_eq!(ability_count(&t, angel), 0);
}

#[test]
fn a_clone_entering_under_humility_has_no_copy_ability_and_is_a_1_1_clone() {
    // CR 614.12: which replacement effects modify how a permanent enters is determined
    // from the permanent as it would exist on the battlefield, continuous effects that
    // already exist included. Under Humility the Clone would have no abilities, so its
    // "enter as a copy" doesn't apply and nobody is asked; it enters as a 0/0 Clone.
    cr!("614.12", "613.1f", "613.4b");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Humility");
    t.battlefield(P1, "Serra Angel");
    let asked_before = t.asked().len();
    let clone = t.enter(P0, "Clone");
    t.settle();
    let asked: Vec<_> = t.asked()[asked_before..].to_vec();
    assert!(
        !asked.iter().any(|(_, d)| matches!(d, Decision::YesNo { .. })),
        "no copy choice was offered: {asked:?}"
    );
    t.g.recompute();
    assert_eq!(name(&t, clone), "Clone");
    assert!(t.on_battlefield(clone), "Humility's base 1/1 keeps the 0/0 alive");
    assert_eq!(t.pt(clone), (1, 1));
    assert_eq!(ability_count(&t, clone), 0);
    assert!(has_subtype(&t, clone, "Shapeshifter"));
}

#[test]
fn copies_dont_copy_type_changes_or_pumps_on_the_original() {
    cr!("707.2", "613.1a", "613.1d", "613.4c");
    ruling!(
        "Clone",
        "any non-copy effects that have changed its power, toughness, types, color, or so on"
    );
    let mut t = TestGame::new(2);
    choose_creature_type(&mut t, P1, "Goblin");
    t.enter(P1, "Arcane Adaptation");
    t.resolve_all();
    let bears = t.battlefield(P1, "Grizzly Bears");
    let coating = t.battlefield(P1, "Liquimetal Coating");
    activate_containing(&mut t, P1, coating, "artifact", &[Entity::Object(bears)]);
    cast_resolve(&mut t, P1, "Giant Growth", &[Entity::Object(bears)]);
    t.g.recompute();
    assert!(is_type(&t, bears, CardType::Artifact));
    assert!(has_subtype(&t, bears, "Goblin"));
    assert_eq!(t.pt(bears), (5, 5));
    let clone = enter_as_copy_of(&mut t, P0, "Clone", bears);
    assert_eq!(name(&t, clone), "Grizzly Bears");
    assert!(!is_type(&t, clone, CardType::Artifact));
    assert!(!has_subtype(&t, clone, "Goblin"), "P1's Arcane Adaptation affects P1's creatures");
    assert!(has_subtype(&t, clone, "Bear"));
    assert_eq!(t.pt(clone), (2, 2));
}

#[test]
fn vesuvan_doppelganger_keeps_its_own_color_and_doesnt_copy_a_color_defining_ability() {
    // "Except it doesn't copy that creature's color": the Doppelganger retains its own
    // color (CR 707.9c) and the copied object's color-defining ability isn't copied
    // (CR 707.9d). The final color is part of its copiable values (CR 707.9b), so a Clone
    // of it is blue too, with the Doppelganger's triggered ability.
    cr!("707.9b", "707.9c", "707.9d", "613.2a", "613.2c");
    ruling!(
        "Vesuvan Doppelganger",
        "copies the mana cost of the creature it's copying but doesn't copy its color"
    );
    ruling!(
        "Vesuvan Doppelganger",
        "will copy Vesuvan Doppelganger's color, and will gain Vesuvan Doppelganger's triggered ability"
    );
    ruling!(
        "Transguild Courier",
        "This card is white, blue, black, red, and green in all zones"
    );
    let mut t = TestGame::new(2);
    let courier = t.battlefield(P1, "Transguild Courier");
    assert_eq!(colors(&t, courier), ColorSet::ALL);
    let dopp = enter_as_copy_of(&mut t, P0, "Vesuvan Doppelganger", courier);
    assert_eq!(name(&t, dopp), "Transguild Courier");
    assert_eq!(
        t.obj_now(dopp).chars.mana_cost.as_ref().unwrap().to_string(),
        "{4}"
    );
    assert_eq!(colors(&t, dopp), color_set(&[Color::Blue]));
    assert!(!has_ability_text(&t, dopp, "all colors"), "the color-defining ability isn't copied");
    assert!(has_ability_text(&t, dopp, "upkeep"));
    assert_eq!(t.pt(dopp), (3, 3));
    assert!(is_type(&t, dopp, CardType::Artifact));
    assert!(has_subtype(&t, dopp, "Golem"));
    // A Clone of the Doppelganger.
    let clone = enter_as_copy_of(&mut t, P0, "Clone", dopp);
    assert_eq!(name(&t, clone), "Transguild Courier");
    assert_eq!(colors(&t, clone), color_set(&[Color::Blue]));
    assert!(has_ability_text(&t, clone, "upkeep"));
    assert!(!has_ability_text(&t, clone, "all colors"));
    // The original is still all colors.
    assert_eq!(colors(&t, courier), ColorSet::ALL);
}

#[test]
fn sakashima_copying_a_changeling_keeps_the_type_defining_ability() {
    // "Legendary in addition to its other types" is an exception that only adds a
    // supertype, so the changeling characteristic-defining ability is copied (CR 707.9d).
    cr!("707.9b", "707.9d", "702.73a", "613.2a", "613.1d");
    let mut t = TestGame::new(2);
    let colossus = t.battlefield(P1, "Chameleon Colossus");
    let sak = enter_as_copy_of(&mut t, P0, "Sakashima the Impostor", colossus);
    assert_eq!(name(&t, sak), "Sakashima the Impostor");
    assert!(t.obj_now(sak).chars.is_legendary());
    assert!(has_kw(&t, sak, KeywordKind::Changeling));
    assert!(has_subtype(&t, sak, "Goblin"));
    assert!(has_subtype(&t, sak, "Elf"));
    assert!(has_subtype(&t, sak, "Shapeshifter"));
    assert!(has_kw(&t, sak, KeywordKind::Protection));
    assert!(has_ability_text(&t, sak, "Return"));
    assert_eq!(t.pt(sak), (4, 4));
    assert_eq!(colors(&t, sak), color_set(&[Color::Green]));
    assert!(!t.obj_now(colossus).chars.is_legendary());
    assert!(t.on_battlefield(colossus) && t.on_battlefield(sak));
}

#[test]
fn spark_double_isnt_legendary_and_neither_is_a_clone_of_it() {
    // The "isn't legendary" exception becomes part of the copiable values (CR 707.9b);
    // the +1/+1 counter doesn't (CR 707.2).
    cr!("707.9b", "707.2", "704.5j", "613.2a", "613.2c", "613.4c");
    ruling!("Spark Double", "this exception is copiable");
    let mut t = TestGame::new(2);
    let isamaru = t.battlefield(P0, "Isamaru, Hound of Konda");
    let double = enter_as_copy_of(&mut t, P0, "Spark Double", isamaru);
    assert_eq!(name(&t, double), "Isamaru, Hound of Konda");
    assert!(!t.obj_now(double).chars.is_legendary());
    assert_eq!(t.counters(double, counters::PLUS1), 1);
    assert_eq!(t.pt(double), (3, 3));
    assert!(t.on_battlefield(isamaru) && t.on_battlefield(double), "no legend rule");
    let clone = enter_as_copy_of(&mut t, P0, "Clone", double);
    assert_eq!(name(&t, clone), "Isamaru, Hound of Konda");
    assert!(!t.obj_now(clone).chars.is_legendary());
    assert_eq!(t.counters(clone, counters::PLUS1), 0);
    assert_eq!(t.pt(clone), (2, 2));
    assert!(t.on_battlefield(isamaru) && t.on_battlefield(double) && t.on_battlefield(clone));
}

#[test]
fn mirrorweave_copies_only_copiable_values_and_keeps_noncopy_effects_and_counters() {
    cr!("707.2", "707.4", "613.1a", "613.2a", "613.4c");
    ruling!("Mirrorweave", "it will be a 5/5 Grizzly Bears");
    ruling!("Mirrorweave", "they’ll each just retain the counters they already have");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let elves = t.battlefield(P0, "Llanowar Elves");
    put_counters(&mut t, elves, counters::PLUS1, 1);
    cast_resolve(&mut t, P0, "Giant Growth", &[Entity::Object(elves)]);
    let giant = t.battlefield(P1, "Hill Giant");
    put_counters(&mut t, giant, counters::PLUS1, 2);
    put_counters(&mut t, bears, counters::PLUS1, 1);
    cast_resolve(&mut t, P0, "Mirrorweave", &[Entity::Object(bears)]);
    t.g.recompute();
    assert_eq!(name(&t, elves), "Grizzly Bears");
    assert_eq!(t.pt(elves), (6, 6), "2/2, +1/+1 counter, Giant Growth");
    assert!(!has_ability_text(&t, elves, "Add {G}"));
    assert_eq!(name(&t, giant), "Grizzly Bears");
    assert_eq!(t.pt(giant), (4, 4), "2/2 and its own two counters, not the target's");
    assert_eq!(colors(&t, giant), color_set(&[Color::Green]));
    end_turn(&mut t);
    assert_eq!(name(&t, elves), "Llanowar Elves");
    assert_eq!(t.pt(elves), (2, 2));
    assert_eq!(name(&t, giant), "Hill Giant");
    assert_eq!(t.pt(giant), (5, 5));
}

#[test]
fn copies_of_a_face_down_creature_stay_2_2s_when_it_is_turned_face_up() {
    // The face-down characteristics are the copiable values (CR 613.2b, 613.2c); a copy
    // doesn't change when the original's copiable values change later (CR 707.2b).
    cr!("707.2", "707.2b", "707.9c", "613.2b", "613.2c", "708.2");
    ruling!(
        "Vesuvan Doppelganger",
        "it becomes a face-up 2/2 blue creature with no name and no abilities other than the one it gives itself"
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 3);
    let egotist = t.hand(P0, "Scornful Egotist");
    t.cast(P0, egotist)
        .method(CastMethod::FaceDown(KeywordKind::Morph))
        .go();
    t.resolve();
    let egotist = t.g.current(egotist);
    assert!(t.obj(egotist).face_down);
    let clone = enter_as_copy_of(&mut t, P0, "Clone", egotist);
    assert!(!t.obj_now(clone).face_down);
    assert_eq!(name(&t, clone), "");
    assert_eq!(t.pt(clone), (2, 2));
    assert_eq!(ability_count(&t, clone), 0);
    assert_eq!(colors(&t, clone), ColorSet::NONE);
    assert!(t.obj_now(clone).chars.subtypes.is_empty());
    let dopp = enter_as_copy_of(&mut t, P0, "Vesuvan Doppelganger", egotist);
    assert!(!t.obj_now(dopp).face_down);
    assert_eq!(name(&t, dopp), "");
    assert_eq!(t.pt(dopp), (2, 2));
    assert_eq!(colors(&t, dopp), color_set(&[Color::Blue]));
    assert_eq!(ability_count(&t, dopp), 1);
    assert!(has_ability_text(&t, dopp, "upkeep"));
    // The Egotist is turned face up: the copies don't change.
    assert!(mtg_engine::facedown::turn_face_up(&mut t.g, egotist, false));
    t.g.recompute();
    assert_eq!(name(&t, egotist), "Scornful Egotist");
    assert_eq!(t.pt(egotist), (1, 1));
    assert_eq!(name(&t, clone), "");
    assert_eq!(t.pt(clone), (2, 2));
    assert_eq!(name(&t, dopp), "");
    assert_eq!(t.pt(dopp), (2, 2));
}

#[test]
fn a_clone_of_a_transformed_permanent_copies_the_back_face_and_cant_transform() {
    cr!("707.8", "712.9", "707.2b", "613.2a");
    let mut t = TestGame::new(2);
    let delver = t.battlefield(P1, "Delver of Secrets // Insectile Aberration");
    assert!(mtg_engine::dfc::transform(&mut t.g, delver));
    t.g.recompute();
    assert_eq!(name(&t, delver), "Insectile Aberration");
    let clone = enter_as_copy_of(&mut t, P0, "Clone", delver);
    assert_eq!(name(&t, clone), "Insectile Aberration");
    assert_eq!(t.pt(clone), (3, 2));
    assert!(has_kw(&t, clone, KeywordKind::Flying));
    assert_eq!(colors(&t, clone), color_set(&[Color::Blue]));
    assert!(has_subtype(&t, clone, "Insect"));
    assert!(!has_ability_text(&t, clone, "look at the top card"));
    // The copy isn't a double-faced permanent: it can't transform.
    assert!(!mtg_engine::dfc::transform(&mut t.g, clone));
    t.g.recompute();
    assert_eq!(name(&t, clone), "Insectile Aberration");
    // Transforming the original back doesn't change the copy.
    assert!(mtg_engine::dfc::transform(&mut t.g, delver));
    t.g.recompute();
    assert_eq!(name(&t, delver), "Delver of Secrets");
    assert_eq!(t.pt(delver), (1, 1));
    assert_eq!(name(&t, clone), "Insectile Aberration");
    assert_eq!(t.pt(clone), (3, 2));
}

#[test]
fn phyrexian_metamorph_copying_a_noncreature_artifact_is_not_a_creature() {
    cr!("707.9b", "613.2a", "208.3");
    ruling!(
        "Phyrexian Metamorph",
        "If Phyrexian Metamorph copies a noncreature artifact, it is no longer a creature"
    );
    let mut t = TestGame::new(2);
    let stone = t.battlefield(P1, "Mind Stone");
    let meta = enter_as_copy_of(&mut t, P0, "Phyrexian Metamorph", stone);
    assert_eq!(name(&t, meta), "Mind Stone");
    assert!(!t.obj_now(meta).is_creature());
    assert!(is_type(&t, meta, CardType::Artifact));
    assert!(t.obj_now(meta).chars.power.is_none());
    assert!(t.on_battlefield(meta), "not a 0/0 creature");
    assert!(has_ability_text(&t, meta, "Add {C}"));
}

#[test]
fn a_copy_exception_setting_power_and_toughness_drops_the_copied_pt_defining_ability() {
    // Quicksilver Gargantuan enters as a copy "except it's 7/7": Tarmogoyf's power- and
    // toughness-defining ability isn't copied (CR 707.9d), so nothing overrides the 7/7
    // in layer 7a, and a Clone of the result is a 7/7 without the defining ability too
    // (CR 707.9b).
    cr!("707.9b", "707.9d", "613.2a", "613.2c", "613.4a", "604.3");
    ruling!(
        "Quicksilver Gargantuan",
        "If the chosen creature has a characteristic-defining ability that sets its power and/or toughness, that ability will not be copied"
    );
    let mut t = TestGame::new(2);
    let goyf = t.battlefield(P1, "Tarmogoyf");
    t.graveyard(P1, "Grizzly Bears");
    t.graveyard(P1, "Plains");
    t.g.recompute();
    assert_eq!(t.pt(goyf), (2, 3), "creature and land in graveyards");
    let garg = enter_as_copy_of(&mut t, P0, "Quicksilver Gargantuan", goyf);
    assert_eq!(name(&t, garg), "Tarmogoyf");
    assert_eq!(t.pt(garg), (7, 7));
    assert!(!has_ability_text(&t, garg, "card types"));
    assert!(has_subtype(&t, garg, "Lhurgoyf"));
    // More card types in graveyards change Tarmogoyf, not the copy.
    t.graveyard(P0, "Giant Growth");
    t.g.recompute();
    assert_eq!(t.pt(goyf), (3, 4));
    assert_eq!(t.pt(garg), (7, 7));
    let clone = enter_as_copy_of(&mut t, P0, "Clone", garg);
    assert_eq!(name(&t, clone), "Tarmogoyf");
    assert_eq!(t.pt(clone), (7, 7));
    assert!(!has_ability_text(&t, clone, "card types"));
    // A Clone of the original Tarmogoyf has the defining ability and counts the types.
    let clone2 = enter_as_copy_of(&mut t, P0, "Clone", goyf);
    assert!(has_ability_text(&t, clone2, "card types"));
    assert_eq!(t.pt(clone2), (3, 4));
}

#[test]
fn a_7_7_copy_exception_gives_no_power_or_toughness_to_a_copy_of_an_animated_land() {
    // The exception can't give power and toughness to a copy that isn't a creature
    // (CR 208.3); when the copy is animated later, the animating effect sets them.
    cr!("707.9b", "208.3", "613.2a", "613.1d", "613.4b");
    ruling!(
        "Quicksilver Gargantuan",
        "If it later becomes a creature, its power and toughness will be determined by the effect that causes it to become a creature; again, it won't be 7/7"
    );
    let mut t = TestGame::new(2);
    let nexus = t.battlefield(P1, "Inkmoth Nexus");
    t.lands(P1, "Wastes", 1);
    activate_containing(&mut t, P1, nexus, "becomes", &[]);
    let garg = enter_as_copy_of(&mut t, P0, "Quicksilver Gargantuan", nexus);
    assert_eq!(name(&t, garg), "Inkmoth Nexus");
    assert!(!t.obj_now(garg).is_creature());
    assert!(t.obj_now(garg).chars.power.is_none());
    assert!(t.obj_now(garg).chars.toughness.is_none());
    t.lands(P0, "Wastes", 1);
    activate_containing(&mut t, P0, garg, "becomes", &[]);
    t.g.recompute();
    assert!(t.obj_now(garg).is_creature());
    assert_eq!(t.pt(garg), (1, 1));
}
