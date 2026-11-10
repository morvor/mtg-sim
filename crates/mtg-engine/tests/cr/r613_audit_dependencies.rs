//! CR 613 audit: dependencies between continuous effects (CR 613.8) and the layer order
//! when one effect removes the ability generating another (CR 613.1), with real cards.

use crate::r613_audit_common::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn blood_moon_stops_urborg_making_swamps_in_either_timestamp_order() {
    // Urborg's "each land is a Swamp" depends on Blood Moon's effect, which removes
    // Urborg's abilities (CR 305.7); both apply in layer 4, so Blood Moon applies first
    // whichever entered later, and Urborg's effect then doesn't exist.
    cr!("613.8", "613.8a", "613.8b", "613.1d", "305.7");
    ruling!(
        "Urborg, Tomb of Yawgmoth",
        "no matter in what order those effects started to apply"
    );
    ruling!("Blood Moon", "Nonbasic lands will lose any other land types and abilities they had");
    for urborg_first in [true, false] {
        let mut t = TestGame::new(2);
        let urborg = if urborg_first {
            let u = t.battlefield(P0, "Urborg, Tomb of Yawgmoth");
            t.battlefield(P1, "Blood Moon");
            u
        } else {
            t.battlefield(P1, "Blood Moon");
            t.battlefield(P0, "Urborg, Tomb of Yawgmoth")
        };
        let plains = t.battlefield(P0, "Plains");
        let mine = t.battlefield(P0, "Urza's Mine");
        t.g.recompute();
        // Urborg: a legendary Mountain with only the Mountain's mana ability.
        assert!(has_subtype(&t, urborg, "Mountain"), "urborg_first={urborg_first}");
        assert!(!has_subtype(&t, urborg, "Swamp"), "urborg_first={urborg_first}");
        assert!(t.obj_now(urborg).chars.is_legendary());
        assert!(has_ability_text(&t, urborg, "Add {R}"));
        assert!(!has_ability_text(&t, urborg, "Swamp"));
        // A basic Plains is neither a Mountain nor a Swamp.
        assert!(has_subtype(&t, plains, "Plains"));
        assert!(!has_subtype(&t, plains, "Swamp"));
        assert!(!has_subtype(&t, plains, "Mountain"));
        assert!(!has_ability_text(&t, plains, "Add {B}"));
        // Urza's Mine loses its land type and its own mana ability.
        assert!(has_subtype(&t, mine, "Mountain"));
        assert!(!t.obj_now(mine).chars.subtypes.iter().any(|s| s.contains("Urza")));
        assert!(!has_ability_text(&t, mine, "{C}"));
        assert!(has_ability_text(&t, mine, "Add {R}"));
    }
}

#[test]
fn dryad_of_the_ilysian_grove_and_blood_moon_are_independent_so_the_later_one_wins() {
    // Neither effect changes what the other applies to or what it does (CR 613.8a), so
    // they apply in timestamp order (CR 613.7): the Dryad's "every basic land type in
    // addition" after Blood Moon adds the types back; before it, Blood Moon removes them.
    cr!("613.7", "613.7a", "613.8a", "613.1d", "305.7");
    ruling!(
        "Dryad of the Ilysian Grove",
        "Each land you control will have the land types Plains, Island, Swamp, Mountain, and Forest"
    );
    let all = ["Plains", "Island", "Swamp", "Mountain", "Forest"];
    // Blood Moon first, then the Dryad.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Blood Moon");
    let dryad = t.battlefield(P0, "Dryad of the Ilysian Grove");
    let mine = t.battlefield(P0, "Urza's Mine");
    t.g.recompute();
    for s in all {
        assert!(has_subtype(&t, mine, s), "{s}");
    }
    assert!(has_ability_text(&t, mine, "Add {G}"));
    // Its own mana ability is still gone: Blood Moon removed the rules text (CR 305.7).
    assert!(!has_ability_text(&t, mine, "{C}"));
    // Blood Moon doesn't touch the Dryad, a creature.
    assert!(has_ability_text(&t, dryad, "every basic land type"));
    assert_eq!(t.pt(dryad), (2, 4));
    // The Dryad first, then Blood Moon.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Dryad of the Ilysian Grove");
    let mine = t.battlefield(P0, "Urza's Mine");
    t.battlefield(P1, "Blood Moon");
    t.g.recompute();
    assert!(has_subtype(&t, mine, "Mountain"));
    for s in ["Plains", "Island", "Swamp", "Forest"] {
        assert!(!has_subtype(&t, mine, s), "{s}");
    }
    assert!(!has_ability_text(&t, mine, "Add {G}"));
    assert!(has_ability_text(&t, mine, "Add {R}"));
    // A basic Forest its controller controls is every basic type either way (a static
    // ability's effect applies to the current set of objects, CR 611.3a).
    let forest = t.battlefield(P0, "Forest");
    t.g.recompute();
    for s in all {
        assert!(has_subtype(&t, forest, s), "{s}");
    }
}

#[test]
fn magus_of_the_moon_keeps_making_mountains_after_losing_its_abilities() {
    // Layer 4 (types) is applied before layer 6 (ability removal), so an effect that
    // removes the Magus's abilities doesn't stop its type-changing effect (CR 613.1).
    cr!("613.1", "613.1d", "613.1f", "613.8a");
    ruling!(
        "Magus of the Moon",
        "If Magus of the Moon loses its abilities, it continues to turn nonbasic lands into Mountains"
    );
    // Humility entered after the Magus.
    let mut t = TestGame::new(2);
    let magus = t.battlefield(P0, "Magus of the Moon");
    let mine = t.battlefield(P1, "Urza's Mine");
    t.battlefield(P1, "Humility");
    t.g.recompute();
    assert_eq!(ability_count(&t, magus), 0);
    assert_eq!(t.pt(magus), (1, 1));
    assert!(has_subtype(&t, mine, "Mountain"));
    assert!(!has_ability_text(&t, mine, "{C}"));
    // Humility entered before the Magus: the same.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Humility");
    let mine = t.battlefield(P1, "Urza's Mine");
    let magus = t.battlefield(P0, "Magus of the Moon");
    t.g.recompute();
    assert_eq!(ability_count(&t, magus), 0);
    assert!(has_subtype(&t, mine, "Mountain"));
    assert!(!has_ability_text(&t, mine, "{C}"));
    // Turn to Frog on the Magus: a 1/1 blue Frog with no abilities whose effect still
    // applies this turn.
    let mut t = TestGame::new(2);
    let magus = t.battlefield(P0, "Magus of the Moon");
    let mine = t.battlefield(P1, "Urza's Mine");
    cast_resolve(&mut t, P1, "Turn to Frog", &[Entity::Object(magus)]);
    t.g.recompute();
    assert!(has_subtype(&t, magus, "Frog"));
    assert_eq!(ability_count(&t, magus), 0);
    assert_eq!(t.pt(magus), (1, 1));
    assert!(has_subtype(&t, mine, "Mountain"));
    assert!(!has_ability_text(&t, mine, "{C}"));
    // When the turn ends, the Magus has its ability again and the land is still a Mountain.
    end_turn(&mut t);
    assert_eq!(ability_count(&t, magus), 1);
    assert!(has_subtype(&t, mine, "Mountain"));
}

#[test]
fn urzas_saga_under_blood_moon_is_a_mountain_that_keeps_granted_abilities_and_stays() {
    // Blood Moon removes the Saga's rules text (its chapter abilities) but not the ability
    // chapter I granted (CR 305.7, a layer 6 effect). A Saga with no chapter abilities
    // has final chapter number 0 (714.2d) but is neither sacrificed (714.4, 704.5s) nor
    // given lore counters (714.3c): both rules apply only to Sagas with chapter abilities.
    cr!("305.7", "613.1d", "613.1f", "714.2d", "714.3c", "714.4", "704.5s");
    ruling!("Urza's Saga", "It keeps those abilities for as long as it's on the battlefield");
    let mut t = TestGame::new(2);
    let saga = t.enter(P0, "Urza's Saga");
    t.resolve_all();
    t.g.recompute();
    assert_eq!(t.counters(saga, counters::LORE), 1);
    assert!(has_ability_text(&t, saga, "Add {C}"));
    assert!(has_ability_text(&t, saga, "Construct"));
    t.battlefield(P1, "Blood Moon");
    t.settle();
    t.g.recompute();
    assert!(t.on_battlefield(saga), "not sacrificed");
    assert!(has_subtype(&t, saga, "Mountain"));
    assert!(!t.obj_now(saga).chars.subtypes.iter().any(|s| s.contains("Urza")));
    assert!(has_subtype(&t, saga, "Saga"), "an enchantment subtype stays");
    assert!(is_type(&t, saga, CardType::Enchantment));
    assert!(is_type(&t, saga, CardType::Land));
    assert!(has_ability_text(&t, saga, "Add {R}"));
    assert!(has_ability_text(&t, saga, "Add {C}"), "the granted ability stays (305.7)");
    assert!(!has_ability_text(&t, saga, "Construct"), "chapter II is gone");
    // The next precombat main phase of its controller puts no lore counter on it, and it
    // isn't sacrificed.
    t.advance_to(P1, Step::PrecombatMain);
    t.advance_to(P0, Step::PrecombatMain);
    t.settle();
    assert_eq!(t.counters(saga, counters::LORE), 1);
    assert!(t.on_battlefield(saga));
    assert!(t.stack_len() == 0, "no chapter ability triggered");
}

#[test]
fn march_of_the_machines_waits_for_mycosynth_lattice_whatever_their_timestamps() {
    // March's "each noncreature artifact" depends on the Lattice's "all permanents are
    // artifacts" (CR 613.8a: it changes what March applies to), so the Lattice applies
    // first even with the later timestamp, and lands become 0/0 artifact creatures.
    cr!("613.8", "613.8a", "613.8b", "613.6", "613.4b", "704.5f");
    ruling!("March of the Machines", "Each artifact land has a mana value of 0");
    for march_first in [true, false] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let plains = t.battlefield(P0, "Plains");
        let stone = t.battlefield(P0, "Mind Stone");
        let (march, lattice) = if march_first {
            let m = t.battlefield(P0, "March of the Machines");
            (m, t.battlefield(P1, "Mycosynth Lattice"))
        } else {
            let l = t.battlefield(P1, "Mycosynth Lattice");
            (t.battlefield(P0, "March of the Machines"), l)
        };
        t.settle();
        t.g.recompute();
        // The Plains became a 0/0 artifact creature and died.
        assert!(!t.on_battlefield(plains), "march_first={march_first}");
        assert_eq!(t.zone(plains), Zone::Graveyard(P0));
        // Mind Stone (mana value 2) is a 2/2 artifact creature.
        assert!(t.obj_now(stone).is_creature());
        assert_eq!(t.pt(stone), (2, 2));
        // So are March itself (4/4) and the Lattice (6/6).
        assert_eq!(t.pt(march), (4, 4));
        assert_eq!(t.pt(lattice), (6, 6));
        // Grizzly Bears is an artifact creature now, but March's layer 4 set didn't
        // include it (it was a creature), so its power and toughness aren't set.
        assert!(is_type(&t, bears, CardType::Artifact));
        assert_eq!(t.pt(bears), (2, 2));
        assert_eq!(colors(&t, bears), ColorSet::NONE, "the Lattice makes it colorless");
    }
}

#[test]
fn opalescence_waits_for_enchanted_evening_so_lands_become_0_0_creatures() {
    // Opalescence's "each other non-Aura enchantment" depends on Enchanted Evening's "all
    // permanents are enchantments" (CR 613.8a), whichever is newer.
    cr!("613.8", "613.8a", "613.8b", "613.6", "613.4b", "704.5f");
    ruling!(
        "Enchanted Evening",
        "This has no effect on those permanents other than letting them interact with things that care about enchantments"
    );
    for opalescence_first in [true, false] {
        let mut t = TestGame::new(2);
        let giant = t.battlefield(P0, "Hill Giant");
        let pacifism = t.battlefield(P0, "Pacifism");
        assert!(t.g.attach(pacifism, Entity::Object(giant)));
        let plains = t.battlefield(P1, "Plains");
        let (opal, evening) = if opalescence_first {
            let o = t.battlefield(P0, "Opalescence");
            (o, t.battlefield(P1, "Enchanted Evening"))
        } else {
            let e = t.battlefield(P1, "Enchanted Evening");
            (t.battlefield(P0, "Opalescence"), e)
        };
        t.settle();
        t.g.recompute();
        assert!(
            !t.on_battlefield(plains),
            "a 0/0 enchantment creature land died (opalescence_first={opalescence_first})"
        );
        assert!(t.obj_now(evening).is_creature());
        assert_eq!(t.pt(evening), (5, 5));
        assert!(!t.obj_now(opal).is_creature(), "\"each other\"");
        // Hill Giant (3/3, mana value 4) is an enchantment creature with base 4/4.
        assert!(is_type(&t, giant, CardType::Enchantment));
        assert_eq!(t.pt(giant), (4, 4));
        // The Aura is an enchantment already and isn't animated.
        assert!(t.on_battlefield(pacifism));
        assert!(!t.obj_now(pacifism).is_creature());
        assert_eq!(t.obj_now(pacifism).attached_to, Some(Entity::Object(giant)));
    }
}

#[test]
fn humility_and_opalescence_animate_and_set_in_timestamp_order_across_layers() {
    // CR 613.6: Opalescence's effect starts in layer 4 (Humility and Worship become
    // creatures), so it still sets their base power and toughness in layer 7b after
    // Humility's effect removed their abilities in layer 6 (Humility's own included, so
    // its 7b part continues the same way); the two settings apply in timestamp order.
    cr!("613.6", "613.1d", "613.1f", "613.4b", "613.7", "613.7a");
    ruling!("Humility", "This is the current interaction between Humility and Opalescence");
    ruling!("Opalescence", "This is the current interaction between Humility and Opalescence");
    for humility_first in [false, true] {
        let mut t = TestGame::new(2);
        let (humility, opal) = if humility_first {
            let h = t.battlefield(P1, "Humility");
            (h, t.battlefield(P0, "Opalescence"))
        } else {
            let o = t.battlefield(P0, "Opalescence");
            (t.battlefield(P1, "Humility"), o)
        };
        let worship = t.battlefield(P0, "Worship");
        t.settle();
        t.g.recompute();
        let want = if humility_first { (4, 4) } else { (1, 1) };
        for id in [humility, worship] {
            assert!(t.obj_now(id).is_creature(), "humility_first={humility_first}");
            assert!(is_type(&t, id, CardType::Enchantment));
            assert_eq!(ability_count(&t, id), 0, "humility_first={humility_first}");
            assert_eq!(t.pt(id), want, "humility_first={humility_first}");
        }
        assert!(!t.obj_now(opal).is_creature(), "\"each other\"");
        assert!(has_ability_text(&t, opal, "non-Aura enchantment"), "keeps its ability");
    }
}

#[test]
fn humility_removes_a_creature_lords_ability_before_its_effect_applies() {
    // Archetype of Imagination's "creatures you control have flying" (layer 6) depends on
    // Humility's "all creatures lose all abilities" (layer 6): applying Humility's effect
    // removes the Archetype's ability, so Humility applies first even when the Archetype
    // entered later (CR 613.8a, 613.8b), and no creature has flying.
    cr!("613.8", "613.8a", "613.8b", "613.1f");
    ruling!("Humility", "Removes all creature abilities");
    for archetype_first in [true, false] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let drake = t.battlefield(P1, "Wind Drake");
        let archetype = if archetype_first {
            let a = t.battlefield(P0, "Archetype of Imagination");
            t.battlefield(P1, "Humility");
            a
        } else {
            t.battlefield(P1, "Humility");
            t.battlefield(P0, "Archetype of Imagination")
        };
        t.g.recompute();
        assert!(!has_kw(&t, bears, KeywordKind::Flying), "archetype_first={archetype_first}");
        assert!(!has_kw(&t, archetype, KeywordKind::Flying));
        assert!(!has_kw(&t, drake, KeywordKind::Flying));
        assert_eq!(ability_count(&t, archetype), 0);
        assert_eq!(t.pt(archetype), (1, 1));
    }
    // Without Humility, the Archetype gives its controller's creatures flying and takes
    // it from the opponent's.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let drake = t.battlefield(P1, "Wind Drake");
    t.battlefield(P0, "Archetype of Imagination");
    t.g.recompute();
    assert!(has_kw(&t, bears, KeywordKind::Flying));
    assert!(!has_kw(&t, drake, KeywordKind::Flying));
}

#[test]
fn arcane_adaptation_and_turn_to_frog_are_independent_and_apply_in_timestamp_order() {
    // "Creatures you control are Goblins in addition to their other types" neither
    // changes nor is changed by "becomes a blue Frog" in what it applies to or does
    // (CR 613.8a), so the later effect wins (CR 613.7).
    cr!("613.7", "613.7a", "613.7b", "613.8a", "613.1d");
    ruling!("Turn to Frog", "The creature will lose all other colors and creature types");
    // Arcane Adaptation first, then Turn to Frog: just a blue Frog.
    let mut t = TestGame::new(2);
    choose_creature_type(&mut t, P0, "Goblin");
    t.enter(P0, "Arcane Adaptation");
    t.resolve_all();
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.recompute();
    assert!(has_subtype(&t, bears, "Goblin") && has_subtype(&t, bears, "Bear"));
    cast_resolve(&mut t, P1, "Turn to Frog", &[Entity::Object(bears)]);
    t.g.recompute();
    assert!(has_subtype(&t, bears, "Frog"));
    assert!(!has_subtype(&t, bears, "Goblin"));
    assert!(!has_subtype(&t, bears, "Bear"));
    assert_eq!(colors(&t, bears), color_set(&[Color::Blue]));
    assert_eq!(t.pt(bears), (1, 1));
    // Turn to Frog first, then Arcane Adaptation enters: a Frog Goblin.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_resolve(&mut t, P1, "Turn to Frog", &[Entity::Object(bears)]);
    choose_creature_type(&mut t, P0, "Goblin");
    t.enter(P0, "Arcane Adaptation");
    t.resolve_all();
    t.g.recompute();
    assert!(has_subtype(&t, bears, "Frog"));
    assert!(has_subtype(&t, bears, "Goblin"));
    assert!(!has_subtype(&t, bears, "Bear"));
    assert_eq!(t.pt(bears), (1, 1));
}

#[test]
fn ashaya_and_life_and_limb_form_a_dependency_loop_resolved_by_timestamps() {
    // Ashaya makes nontoken creatures Forest lands; Life and Limb makes Forests 1/1
    // Saproling creatures. With a basic Forest on the battlefield each effect changes
    // what the other applies to (CR 613.8a), a loop, so they apply in timestamp order
    // (CR 613.8b).
    cr!("613.8a", "613.8b", "613.1d", "613.4b", "613.4a");
    ruling!(
        "Life and Limb",
        "Each Forest and Saproling will have the card types creature and land, and the subtypes Forest and Saproling"
    );
    ruling!(
        "Ashaya, Soul of the Wild",
        "As long as Ashaya is on the battlefield, it's affected by its second ability and thus its first ability counts itself"
    );
    // Ashaya first: its effect makes Grizzly Bears a Forest, then Life and Limb makes
    // every Forest a 1/1 green Saproling.
    let mut t = TestGame::new(2);
    let forest = t.battlefield(P0, "Forest");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let ashaya = t.battlefield(P0, "Ashaya, Soul of the Wild");
    t.battlefield(P0, "Life and Limb");
    t.g.recompute();
    assert!(has_subtype(&t, bears, "Forest") && has_subtype(&t, bears, "Saproling"));
    assert!(is_type(&t, bears, CardType::Land));
    assert_eq!(t.pt(bears), (1, 1));
    assert_eq!(colors(&t, bears), color_set(&[Color::Green]));
    assert!(t.obj_now(forest).is_creature() && has_subtype(&t, forest, "Saproling"));
    assert_eq!(t.pt(forest), (1, 1));
    // Ashaya is a Forest land too, so Life and Limb's base 1/1 (layer 7b) overrides its
    // characteristic-defining power and toughness (layer 7a).
    assert!(has_subtype(&t, ashaya, "Saproling"));
    assert_eq!(t.pt(ashaya), (1, 1));
    // Life and Limb first: the Bears isn't a Forest when it applies, so Ashaya then makes
    // it a Forest land that isn't a Saproling and keeps its 2/2.
    let mut t = TestGame::new(2);
    let forest = t.battlefield(P0, "Forest");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Life and Limb");
    let ashaya = t.battlefield(P0, "Ashaya, Soul of the Wild");
    t.g.recompute();
    assert!(has_subtype(&t, bears, "Forest"));
    assert!(!has_subtype(&t, bears, "Saproling"));
    assert!(is_type(&t, bears, CardType::Land));
    assert_eq!(t.pt(bears), (2, 2));
    assert_eq!(colors(&t, bears), color_set(&[Color::Green]));
    assert!(t.obj_now(forest).is_creature() && has_subtype(&t, forest, "Saproling"));
    // Ashaya counts the lands it controls: itself, the Bears and the Forest.
    assert!(!has_subtype(&t, ashaya, "Saproling"));
    assert_eq!(t.pt(ashaya), (3, 3));
}

#[test]
fn humility_then_an_animated_land_keeps_the_abilities_its_animation_granted() {
    // Humility (layer 6, earlier timestamp) removes the land's printed abilities, but the
    // flying and infect the later animating effect grants apply after it (CR 613.7b);
    // both effects set base 1/1 in layer 7b, and the +1/+1 counter applies on top (7c).
    cr!("613.1d", "613.1f", "613.4b", "613.4c", "613.7", "613.7b");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Humility");
    let nexus = t.battlefield(P0, "Inkmoth Nexus");
    put_counters(&mut t, nexus, counters::PLUS1, 1);
    t.lands(P0, "Wastes", 1);
    activate_containing(&mut t, P0, nexus, "becomes", &[]);
    t.g.recompute();
    assert!(t.obj_now(nexus).is_creature());
    assert!(has_kw(&t, nexus, KeywordKind::Flying));
    assert!(has_kw(&t, nexus, KeywordKind::Infect));
    assert!(!has_ability_text(&t, nexus, "Add {C}"), "its printed mana ability is lost");
    assert_eq!(t.pt(nexus), (2, 2));
}
