//! Item `rulings-residue-misc-1`, pass B — mana rulings (batches P098, P099): mana filters
//! and replacements (Celestial Dawn, Pale Moon, Deep Water, Hall of Gemstone, Mycosynth
//! Lattice), mana increasers ("whenever you tap a land for mana, add ..." triggered mana
//! abilities and "produces twice as much" replacements), mana storage, and mana abilities
//! that don't use the stack (CR 106, 605).

use crate::r_residue_misc_1_common::*;
use crate::r_s05_common::colors;
use crate::r_s20_common::tap_for_mana;
use mtg_engine::ability::*;
use mtg_engine::decision::Answer;
use mtg_engine::mana::{Mana, ManaRestriction, ManaType};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

use ManaType::*;

/// Activates the first activated ability of `id` (a mana ability) for `p`.
fn tap(t: &mut TestGame, p: PlayerId, id: ObjectId) {
    t.g.turn.priority = Some(p);
    t.activate(p, id, 0, &[]).unwrap();
}

/// The mana in `p`'s pool, with its restrictions.
fn pool_mana(t: &TestGame, p: PlayerId) -> Vec<Mana> {
    t.g.player(p).mana_pool.mana.clone()
}

/// The amount of mana of type `ty` in `p`'s pool with (`true`) or without a restriction.
fn restricted(t: &TestGame, p: PlayerId, ty: ManaType, with: bool) -> usize {
    pool_mana(t, p)
        .iter()
        .filter(|m| m.ty == ty && m.restriction.is_some() == with)
        .count()
}

/// A 0/1 Eldrazi Spawn token for P0 ("Sacrifice this token: Add {C}."), made by Kozilek's
/// Predator's enters trigger.
fn spawn_token(t: &mut TestGame) -> ObjectId {
    let before = tokens(t, P0);
    t.enter(P0, "Kozilek's Predator");
    t.resolve_all();
    *tokens(t, P0)
        .iter()
        .find(|id| !before.contains(id))
        .unwrap()
}

fn clear_pool(t: &mut TestGame, p: PlayerId) {
    t.g.players[p.idx()].mana_pool.mana.clear();
}

// ---------------------------------------------------------------------------------------
// Mana filters
// ---------------------------------------------------------------------------------------

#[test]
fn celestial_dawn_white_mana_for_a_colored_ability_keeps_its_activation_limit() {
    cr!("609.4b", "602.5b");
    ruling!(
        "Celestial Dawn",
        "any limitations on activating the ability (such as the number of times it can be activated each turn) still apply"
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Celestial Dawn");
    // Snarling Wolf: "{1}{G}: This creature gets +2/+2 until end of turn. Activate only
    // once each turn."
    let wolf = t.battlefield(P0, "Snarling Wolf");
    t.g.players[0].mana_pool.add_type(W, 4);
    t.activate(P0, wolf, 0, &[]).unwrap();
    assert_eq!(pool(&t, P0), 2, "{{1}}{{G}} paid with white mana");
    t.resolve_all();
    assert_eq!(pt_now(&mut t, wolf), (3, 3));
    assert!(t.activate(P0, wolf, 0, &[]).is_err(), "only once each turn");
}

#[test]
fn celestial_dawn_text_changed_to_another_land_type() {
    cr!("612.2", "305.7");
    ruling!(
        "Celestial Dawn",
        "your lands are all of that new land type, and they will produce mana of the appropriate color for that type"
    );
    let mut t = TestGame::new(2);
    let dawn = t.battlefield(P0, "Celestial Dawn");
    let mountain = t.battlefield(P0, "Mountain");
    // White mana pays Mind Bend's {U}.
    t.g.players[0].mana_pool.add_type(W, 1);
    // Mind Bend: "Change the text of target permanent by replacing all instances of one
    // color word with another or one basic land type with another."
    // Options: the five colors, then Plains, Island, Swamp, Mountain, Forest; then the
    // other basic land types (Island, Swamp, Mountain, Forest). Plains -> Swamp.
    t.answer(P0, DecisionKind::Option, Answer::Index(5));
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    cast_resolve(&mut t, P0, "Mind Bend", &[obj(dawn)]);
    t.g.recompute();
    assert!(t.obj_now(mountain).chars.has_subtype("Swamp"));
    assert!(!t.obj_now(mountain).chars.has_subtype("Plains"));
    assert!(!t.obj_now(mountain).chars.has_subtype("Mountain"));
    clear_pool(&mut t, P0);
    tap(&mut t, P0, mountain);
    assert_eq!(mana(&t, P0, B), 1);
    assert_eq!(pool(&t, P0), 1);
}

#[test]
fn celestial_dawn_overrides_an_earlier_color_change() {
    cr!("613.1e", "613.7a");
    ruling!(
        "Celestial Dawn",
        "a color change on a permanent prior to Celestial Dawn entering the battlefield will be overridden by Celestial Dawn's effect"
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    run_from(
        &mut t,
        P0,
        None,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::SetColors(ColorSet::single(Color::Blue))],
            duration: Duration::EndOfTurn,
        },
        &[obj(bears)],
    );
    assert_eq!(colors(&t, bears), ColorSet::single(Color::Blue));
    t.battlefield(P0, "Celestial Dawn");
    t.g.recompute();
    assert_eq!(colors(&t, bears), ColorSet::single(Color::White));
}

#[test]
fn celestial_dawn_doesnt_change_snow() {
    cr!("205.4a", "305.7");
    ruling!(
        "Celestial Dawn",
        "Will not add or remove Snow Supertype from a land."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Celestial Dawn");
    let snow = t.battlefield(P0, "Snow-Covered Forest");
    let plain = t.battlefield(P0, "Forest");
    t.g.recompute();
    assert!(t.obj_now(snow).chars.has_subtype("Plains"));
    assert!(t.obj_now(snow).chars.supertypes.contains(Supertype::Snow));
    assert!(t.obj_now(plain).chars.has_subtype("Plains"));
    assert!(!t.obj_now(plain).chars.supertypes.contains(Supertype::Snow));
}

#[test]
fn celestial_dawn_mana_spent_as_another_color_is_still_white() {
    cr!("609.4b", "106.1");
    ruling!(
        "Celestial Dawn",
        "If you do, and the spell or ability checks the actual color of the mana, it can tell the difference."
    );
    // Firespout ({2}{R/G}): "deals 3 damage to each creature without flying if {R} was
    // spent to cast this spell and 3 damage to each creature with flying if {G} was spent".
    supported("Firespout");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Celestial Dawn");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let birds = t.battlefield(P1, "Birds of Paradise");
    let spout = t.hand(P0, "Firespout");
    t.g.players[0].mana_pool.add_type(W, 3);
    t.cast(P0, spout).go();
    t.resolve_all();
    // White mana was spent for {R/G}: neither {R} nor {G} was spent.
    assert!(t.on_battlefield(bears));
    assert!(t.on_battlefield(birds));
}

#[test]
fn mycosynth_lattice_colorless_mana_as_any_color_but_restrictions_remain() {
    cr!("609.4b", "106.6");
    ruling!(
        "Mycosynth Lattice",
        "Mycosynth Lattice doesn’t allow mana from Vedalken Engineer to be used to cast a nonartifact spell."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mycosynth Lattice");
    // Colorless mana pays {R}.
    let wastes = t.battlefield(P0, "Wastes");
    tap(&mut t, P0, wastes);
    let bolt = t.hand(P0, "Lightning Bolt");
    assert!(t.cast(P0, bolt).target(P1).try_go().is_ok());
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    // Vedalken Engineer: "{T}: Add two mana of any one color. Spend this mana only to cast
    // artifact spells or activate abilities of artifacts." A spell isn't a permanent, so
    // Lightning Bolt isn't an artifact spell.
    let engineer = t.battlefield(P0, "Vedalken Engineer");
    tap(&mut t, P0, engineer);
    assert_eq!(pool(&t, P0), 2);
    let bolt = t.hand(P0, "Lightning Bolt");
    assert!(t.cast(P0, bolt).target(P1).try_go().is_err());
    assert_eq!(pool(&t, P0), 2);
}

#[test]
fn pale_moon_and_deep_water_change_only_the_type() {
    cr!("106.12a", "106.12b");
    ruling!(
        "Pale Moon",
        "The ability does not change the amount of mana produced, only the color."
    );
    ruling!(
        "Deep Water",
        "The amount of mana produced is unchanged, but it will all be {U}."
    );
    // Golgari Rot Farm: "{T}: Add {B}{G}."
    let mut t = TestGame::new(2);
    let farm = t.battlefield(P0, "Golgari Rot Farm");
    t.g.players[0].mana_pool.add_type(U, 2);
    cast_resolve(&mut t, P0, "Pale Moon", &[]);
    tap(&mut t, P0, farm);
    assert_eq!(mana(&t, P0, C), 2);
    assert_eq!(pool(&t, P0), 2);
    // A basic land isn't affected.
    let forest = t.battlefield(P0, "Forest");
    tap(&mut t, P0, forest);
    assert_eq!(mana(&t, P0, G), 1);

    let mut t = TestGame::new(2);
    let farm = t.battlefield(P0, "Golgari Rot Farm");
    let water = t.battlefield(P0, "Deep Water");
    t.g.players[0].mana_pool.add_type(U, 1);
    t.activate(P0, water, 0, &[]).unwrap();
    t.resolve_all();
    tap(&mut t, P0, farm);
    assert_eq!(mana(&t, P0, U), 2);
    assert_eq!(pool(&t, P0), 2);
}

#[test]
fn false_dawn_doesnt_change_the_color_of_permanents() {
    cr!("106.12a");
    ruling!(
        "False Dawn",
        "This card’s ability does not change the color of any permanents."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.players[0].mana_pool.add_type(W, 2);
    cast_resolve(&mut t, P0, "False Dawn", &[]);
    t.g.recompute();
    assert_eq!(colors(&t, bears), ColorSet::single(Color::Green));
    // Its mana is white.
    let forest = t.battlefield(P0, "Forest");
    tap(&mut t, P0, forest);
    assert_eq!(mana(&t, P0, W), 1);
}

/// P0's Hall of Gemstone, with the color chosen in P0's upkeep (an index among the five).
fn hall_of_gemstone(t: &mut TestGame, color: usize) {
    t.battlefield(P0, "Hall of Gemstone");
    t.set_step(P0, Step::Untap);
    t.answer(P0, DecisionKind::Option, Answer::Index(color));
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
}

#[test]
fn hall_of_gemstone_keeps_restrictions_and_ignores_lands_without_mana() {
    cr!("106.12a", "106.6");
    ruling!(
        "Hall of Gemstone",
        "Although the color may be altered, any restrictions on the use of the mana are not removed."
    );
    ruling!(
        "Hall of Gemstone",
        "Will not affect lands that do not produce mana."
    );
    let mut t = TestGame::new(2);
    // Green.
    hall_of_gemstone(&mut t, 4);
    // Ancient Ziggurat: "{T}: Add one mana of any color. Spend this mana only to cast a
    // creature spell."
    let zig = t.battlefield(P0, "Ancient Ziggurat");
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    tap(&mut t, P0, zig);
    let m = pool_mana(&t, P0);
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].ty, G);
    assert_eq!(
        m[0].restriction,
        Some(ManaRestriction::SpellOfType(CardType::Creature))
    );
    // Thawing Glaciers ("{1}, {T}: Search your library for a basic land card, put that
    // card onto the battlefield tapped, then shuffle. Return this land to its owner's hand
    // at the beginning of the next cleanup step.") isn't tapped for mana: nothing is added.
    clear_pool(&mut t, P0);
    let glaciers = t.battlefield(P0, "Thawing Glaciers");
    let forest = t.library_top(P0, "Forest");
    t.g.players[0].mana_pool.add_type(C, 1);
    t.answer_choose(P0, &[obj(forest)]);
    t.activate(P0, glaciers, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(pool(&t, P0), 0);
    assert!(t.on_battlefield(forest));
}

#[test]
fn pulse_of_llanowar_can_choose_the_normal_color() {
    cr!("106.12a");
    ruling!(
        "Pulse of Llanowar",
        "You can choose the same color or mana the land would normally produce."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Pulse of Llanowar");
    let forest = t.battlefield(P0, "Forest");
    t.answer(P0, DecisionKind::Option, Answer::Index(4));
    tap(&mut t, P0, forest);
    assert_eq!(mana(&t, P0, G), 1);
    assert_eq!(pool(&t, P0), 1);
}

#[test]
fn selvala_mana_ability_with_no_power() {
    cr!("605.3a", "605.3b", "106.4");
    ruling!(
        "Selvala, Heart of the Wilds",
        "If the greatest power among creatures you control is 0 or less at that time, no mana is added."
    );
    let mut t = TestGame::new(2);
    let selvala = t.battlefield(P0, "Selvala, Heart of the Wilds");
    t.g.players[0].mana_pool.add_type(G, 1);
    for _ in 0..2 {
        t.answer(P0, DecisionKind::Option, Answer::Index(0));
    }
    t.activate(P0, selvala, 0, &[]).unwrap();
    // No stack: the mana is added at once (Selvala's power is 2).
    assert_eq!(t.stack_len(), 0);
    assert_eq!(mana(&t, P0, W), 2);
    // Power 0: no mana.
    let mut t = TestGame::new(2);
    let selvala = t.battlefield(P0, "Selvala, Heart of the Wilds");
    run_from(
        &mut t,
        P0,
        None,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::ModifyPT(Value::Const(-2), Value::Const(0))],
            duration: Duration::EndOfTurn,
        },
        &[obj(selvala)],
    );
    t.g.players[0].mana_pool.add_type(G, 1);
    t.activate(P0, selvala, 0, &[]).unwrap();
    assert!(t.obj_now(selvala).tapped);
    assert_eq!(pool(&t, P0), 0);
}

#[test]
fn terrarion_and_wizards_rockets_mana_then_the_draw() {
    cr!("605.3b", "603.6c", "106.1b");
    ruling!(
        "Terrarion",
        "You can get either two mana of the same color or one mana each of different colors."
    );
    ruling!("Terrarion", "You get the mana before you draw the card.");
    ruling!(
        "Terrarion",
        "You must choose the colors of mana Terrarion produces before its last ability has you draw a card."
    );
    ruling!(
        "Wizard's Rockets",
        "You must choose the colors of mana Wizard's Rockets produces before its last ability has you draw a card."
    );
    for (name, colors) in [
        ("Terrarion", [1, 1]),
        ("Terrarion", [0, 3]),
        ("Wizard's Rockets", [2, 4]),
    ] {
        let mut t = TestGame::new(2);
        let id = t.battlefield(P0, name);
        t.g.players[0].mana_pool.add_type(C, 2);
        if name == "Wizard's Rockets" {
            t.answer(P0, DecisionKind::X, Answer::Number(2));
        }
        for c in colors {
            t.answer(P0, DecisionKind::Option, Answer::Index(c));
        }
        let hand = t.hand_size(P0);
        t.activate(P0, id, 0, &[]).unwrap();
        // The mana is added as the mana ability resolves; the draw is a triggered ability
        // that hasn't been put on the stack yet.
        assert_eq!(pool(&t, P0), 2, "{name}");
        let types = [W, U, B, R, G];
        for c in colors {
            assert!(mana(&t, P0, types[c]) >= 1, "{name}");
        }
        assert_eq!(t.hand_size(P0), hand, "{name}");
        t.settle();
        assert_eq!(t.stack_len(), 1, "{name}");
        t.resolve_all();
        assert_eq!(t.hand_size(P0), hand + 1, "{name}");
    }
}

#[test]
fn terrarion_and_wizards_rockets_draw_however_they_go_to_the_graveyard() {
    cr!("603.6c", "603.10a");
    ruling!(
        "Terrarion",
        "Terrarion’s last ability triggers no matter how it’s put into a graveyard from the battlefield."
    );
    ruling!(
        "Terrarion",
        "You draw a card no matter how Terrarion winds up in a graveyard from the battlefield."
    );
    ruling!(
        "Wizard's Rockets",
        "Wizard's Rockets last ability triggers no matter how it's put into a graveyard from the battlefield."
    );
    for name in ["Terrarion", "Wizard's Rockets"] {
        let mut t = TestGame::new(2);
        let id = t.battlefield(P0, name);
        let hand = t.hand_size(P0);
        destroy(&mut t, id);
        t.resolve_all();
        assert_eq!(t.hand_size(P0), hand + 1, "{name}");
        // Exiled instead: no draw.
        let mut t = TestGame::new(2);
        let id = t.battlefield(P0, name);
        exile_card(&mut t, id);
        t.resolve_all();
        assert_eq!(t.hand_size(P0), hand, "{name}");
    }
}

// ---------------------------------------------------------------------------------------
// Mana increasers
// ---------------------------------------------------------------------------------------

#[test]
fn doubling_cube_doubles_each_type_but_not_the_restrictions() {
    cr!("605.1a", "605.3b", "106.6", "106.1b");
    ruling!(
        "Doubling Cube",
        "Any restrictions on the unspent mana aren't copied."
    );
    ruling!(
        "Doubling Cube",
        "Doubling Cube's ability is a mana ability."
    );
    ruling!(
        "Doubling Cube",
        "The \"type\" of mana is its color, or lack thereof (if the mana is colorless)."
    );
    let mut t = TestGame::new(2);
    let cube = t.battlefield(P0, "Doubling Cube");
    // {3} to activate it, from mana that can be spent only on abilities.
    for _ in 0..3 {
        t.g.players[0].mana_pool.add(Mana {
            restriction: Some(ManaRestriction::AbilitiesOnly),
            ..Mana::new(C)
        });
    }
    // The ruling's example: {C}{W}{W}{B} with no restrictions and {U}{U}{U} that can be
    // used only to cast artifact spells.
    for ty in [C, W, W, B] {
        t.g.players[0].mana_pool.add_type(ty, 1);
    }
    for _ in 0..3 {
        t.g.players[0].mana_pool.add(Mana {
            restriction: Some(ManaRestriction::SpellOfType(CardType::Artifact)),
            ..Mana::new(U)
        });
    }
    tap(&mut t, P0, cube);
    // A mana ability: nothing went on the stack.
    assert_eq!(t.stack_len(), 0);
    assert_eq!(restricted(&t, P0, C, true), 0, "the {{3}} was paid");
    assert_eq!(mana(&t, P0, C), 2);
    assert_eq!(mana(&t, P0, W), 4);
    assert_eq!(mana(&t, P0, B), 2);
    assert_eq!(restricted(&t, P0, U, true), 3);
    assert_eq!(restricted(&t, P0, U, false), 3);
    assert_eq!(pool(&t, P0), 14);
}

#[test]
fn colorless_land_creature_gets_forsaken_monuments_bonus() {
    cr!("105.2c", "613.4c");
    ruling!(
        "Forsaken Monument",
        "If an effect makes a colorless land into a creature without specifying that it becomes a certain color, the land creature gets +2/+2."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Forsaken Monument");
    let wastes = t.battlefield(P0, "Wastes");
    animate(&mut t, wastes, 2, 2);
    assert_eq!(pt_now(&mut t, wastes), (4, 4));
}

#[test]
fn forsaken_monuments_life_gain_doesnt_trigger_on_playing_a_land() {
    cr!("305.1", "601.2i");
    ruling!(
        "Forsaken Monument",
        "Forsaken Monument’s last ability won’t trigger when you play a land."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Forsaken Monument");
    let wastes = t.hand(P0, "Wastes");
    t.play_land(P0, wastes).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    cast_resolve(&mut t, P0, "Ornithopter", &[]);
    assert_eq!(t.life(P0), 22);
}

#[test]
fn forsaken_monument_adds_one_c_for_a_permanent_tapped_for_c() {
    cr!("106.12", "605.1b");
    ruling!(
        "Forsaken Monument",
        "If you tap a permanent for more than one {C}, you add only one additional {C}."
    );
    ruling!(
        "Forsaken Monument",
        "You “tap a permanent for {C}” only if you activate a mana ability of that permanent that includes the {T} symbol in its cost, and only if it produces one or more {C} as it resolves."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Forsaken Monument");
    // Ancient Tomb: "{T}: Add {C}{C}. This land deals 2 damage to you."
    let tomb = t.battlefield(P0, "Ancient Tomb");
    tap(&mut t, P0, tomb);
    assert_eq!(mana(&t, P0, C), 3);
    // An Eldrazi Spawn's "Sacrifice this token: Add {C}." has no {T}.
    clear_pool(&mut t, P0);
    let spawn = spawn_token(&mut t);
    tap(&mut t, P0, spawn);
    assert_eq!(mana(&t, P0, C), 1);
    // A Forest tapped for {G}: not for {C}.
    clear_pool(&mut t, P0);
    let forest = t.battlefield(P0, "Forest");
    tap(&mut t, P0, forest);
    assert_eq!(pool(&t, P0), 1);
}

#[test]
fn mana_reflection_and_tapping_for_mana() {
    cr!("106.12", "106.12b");
    ruling!(
        "Mana Reflection",
        "If an ability triggers “whenever you tap” something for mana and produces mana, that triggered mana ability won't be affected by Mana Reflection."
    );
    ruling!(
        "Mana Reflection",
        "If the mana ability of that permanent puts any restrictions or riders on the mana it produces, that will apply to all the mana it produces this way."
    );
    ruling!(
        "Mana Reflection",
        "You're “tapping a permanent for mana” only if you're activating a mana ability of that permanent that includes the {T} symbol in its cost."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mana Reflection");
    // Wild Growth: "Whenever enchanted land is tapped for mana, its controller adds an
    // additional {G}."
    let forest = t.battlefield(P0, "Forest");
    let wild = t.battlefield(P0, "Wild Growth");
    assert!(t.g.attach(wild, Entity::Object(forest)));
    tap(&mut t, P0, forest);
    assert_eq!(mana(&t, P0, G), 3);
    // Ancient Ziggurat's restriction applies to both mana.
    clear_pool(&mut t, P0);
    let zig = t.battlefield(P0, "Ancient Ziggurat");
    t.answer(P0, DecisionKind::Option, Answer::Index(2));
    tap(&mut t, P0, zig);
    assert_eq!(restricted(&t, P0, B, true), 2);
    assert_eq!(pool(&t, P0), 2);
    // A Spawn sacrificed for mana isn't tapped for mana.
    clear_pool(&mut t, P0);
    let spawn = spawn_token(&mut t);
    tap(&mut t, P0, spawn);
    assert_eq!(pool(&t, P0), 1);
}

/// For each of these "whenever you tap a land for mana, add one mana of any type that land
/// produced" cards: Dimir Aqueduct ({U}{B}) tapped with the card on the battlefield, the
/// second type chosen.
#[test]
fn one_mana_of_a_type_the_land_produced() {
    cr!("106.12", "605.1b", "106.6");
    ruling!(
        "Mirari's Wake",
        "If tapping a land produces more than one type of mana, you choose one type that was produced and add one mana of that type."
    );
    ruling!(
        "Zendikar Resurgent",
        "If the land’s mana ability produces more than one type of mana, choose one of those types to determine what mana Zendikar Resurgent produces."
    );
    for name in [
        "Mirari's Wake",
        "Zendikar Resurgent",
        "Nikya of the Old Ways",
    ] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, name);
        let aqueduct = t.battlefield(P0, "Dimir Aqueduct");
        t.answer(P0, DecisionKind::Option, Answer::Index(1));
        tap(&mut t, P0, aqueduct);
        assert_eq!(mana(&t, P0, U), 1, "{name}");
        assert_eq!(mana(&t, P0, B), 2, "{name}");
    }
}

#[test]
fn added_mana_has_no_restrictions_of_the_lands() {
    cr!("106.6", "106.12");
    ruling!(
        "Mirari's Wake",
        "If the mana produced by the land has any additional restrictions or riders, those restrictions or riders don't apply to the mana added by Mirari's Wake."
    );
    ruling!(
        "Nikya of the Old Ways",
        "Nikya doesn’t care about any restrictions or riders your lands put on the mana they produce"
    );
    for name in ["Mirari's Wake", "Nikya of the Old Ways"] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, name);
        let zig = t.battlefield(P0, "Ancient Ziggurat");
        t.answer(P0, DecisionKind::Option, Answer::Index(3));
        tap(&mut t, P0, zig);
        assert_eq!(restricted(&t, P0, R, true), 1, "{name}");
        assert_eq!(restricted(&t, P0, R, false), 1, "{name}");
        // Unclaimed Territory: "{T}: Add one mana of any color. Spend this mana only to
        // cast a creature spell of the chosen type."
        let mut t = TestGame::new(2);
        t.battlefield(P0, name);
        t.answer(P0, DecisionKind::Option, Answer::Index(0));
        let territory = t.enter(P0, "Unclaimed Territory");
        t.resolve_all();
        t.g.objects[territory.0 as usize].tapped = false;
        t.answer(P0, DecisionKind::Option, Answer::Index(4));
        assert!(tap_for_mana(&mut t, P0, territory, "any color"));
        assert_eq!(restricted(&t, P0, G, true), 1, "{name}");
        assert_eq!(restricted(&t, P0, G, false), 1, "{name}");
    }
}

#[test]
fn triggered_mana_abilities_dont_use_the_stack() {
    cr!("605.1b", "605.3b", "605.4a");
    ruling!(
        "Regal Behemoth",
        "Regal Behemoth's last ability is a triggered mana ability. It doesn't use the stack and can't be responded to."
    );
    ruling!(
        "Ultima, Origin of Oblivion",
        "Ultima's last ability is a triggered mana ability. It doesn't use the stack and can't be responded to."
    );
    ruling!(
        "Overabundance",
        "This card’s ability is a mana ability. There is no chance to respond to it in order to prevent the damage."
    );
    // Regal Behemoth, while P0 is the monarch.
    let mut t = TestGame::new(2);
    let behemoth = t.enter(P0, "Regal Behemoth");
    t.resolve_all();
    let _ = behemoth;
    let forest = t.battlefield(P0, "Forest");
    t.answer(P0, DecisionKind::Option, Answer::Index(3));
    tap(&mut t, P0, forest);
    assert_eq!(t.stack_len(), 0);
    assert_eq!(mana(&t, P0, G), 1);
    assert_eq!(mana(&t, P0, R), 1);
    // Ultima: a land tapped for {C}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ultima, Origin of Oblivion");
    let wastes = t.battlefield(P0, "Wastes");
    tap(&mut t, P0, wastes);
    assert_eq!(t.stack_len(), 0);
    assert_eq!(mana(&t, P0, C), 2);
    // Overabundance: the mana and the damage at once.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Overabundance");
    let forest = t.battlefield(P0, "Forest");
    tap(&mut t, P0, forest);
    assert_eq!(t.stack_len(), 0);
    assert_eq!(mana(&t, P0, G), 2);
    assert_eq!(t.life(P0), 19);
}

/// Overabundance's ruling "The ability deals one damage even if the land produces no mana
/// when it was tapped to use a mana ability." (2004-10-04) is outdated: since the CR effective
/// 2015-07-17 (Magic Origins, then 106.10a) a "tapped for mana" ability triggers only when
/// the mana ability "resolves and produces mana" (106.12a). Exempt in rulings-P099.tsv and
/// logged in docs/RULING_DISCREPANCIES.md; the engine follows the CR.
#[test]
fn overabundance_doesnt_trigger_for_a_land_that_produced_no_mana() {
    cr!("106.12a");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Overabundance");
    // Cabal Coffers: "{2}, {T}: Add {B} for each Swamp you control." No Swamps.
    supported("Cabal Coffers");
    let coffers = t.battlefield(P0, "Cabal Coffers");
    t.g.players[0].mana_pool.add_type(C, 2);
    tap(&mut t, P0, coffers);
    assert!(t.obj_now(coffers).tapped);
    assert_eq!(pool(&t, P0), 0);
    assert_eq!(t.life(P0), 20);
    // With a Swamp, it produces {B}: the trigger adds {B} and deals 1 damage.
    t.battlefield(P0, "Swamp");
    t.g.objects[coffers.0 as usize].tapped = false;
    t.g.players[0].mana_pool.add_type(C, 2);
    tap(&mut t, P0, coffers);
    assert_eq!(mana(&t, P0, B), 2);
    assert_eq!(t.life(P0), 19);
}

#[test]
fn two_nyxbloom_ancients_make_nine_times_as_much() {
    cr!("106.12b", "616.1");
    ruling!(
        "Nyxbloom Ancient",
        "The effects of multiple Nyxbloom Ancients are cumulative."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Nyxbloom Ancient");
    t.battlefield(P0, "Nyxbloom Ancient");
    let forest = t.battlefield(P0, "Forest");
    tap(&mut t, P0, forest);
    assert_eq!(mana(&t, P0, G), 9);
}

#[test]
fn draw_triggers_resolve_before_the_spell() {
    cr!("603.3", "405.5");
    ruling!(
        "Zendikar Resurgent",
        "The last ability will resolve before the creature spell that caused it to trigger."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Zendikar Resurgent");
    let bears = cast_new(&mut t, P0, "Grizzly Bears", &[]);
    let hand = t.hand_size(P0);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.zone(bears), Zone::Stack);
}

// ---------------------------------------------------------------------------------------
// Mana producers, rocks and storage
// ---------------------------------------------------------------------------------------

/// Queues `n` "yes" answers for `p`.
fn yes(t: &mut TestGame, p: PlayerId, n: usize) {
    for _ in 0..n {
        t.answer_yes(p, true);
    }
}

#[test]
fn pawn_of_ulamog_triggers_for_each_creature_dying_at_once() {
    cr!("603.2c", "603.10a");
    ruling!(
        "Pawn of Ulamog",
        "If multiple nontoken creatures you control (possibly including Pawn of Ulamog itself) are put into their owners' graveyards at the same time, Pawn of Ulamog's triggered ability will trigger that many times."
    );
    ruling!(
        "Pawn of Ulamog",
        "If you control more than one Pawn of Ulamog and a nontoken creature you control is put into a graveyard, each of those Pawns' abilities will trigger."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Pawn of Ulamog");
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Hill Giant");
    yes(&mut t, P0, 3);
    cast_resolve(&mut t, P0, "Day of Judgment", &[]);
    assert_eq!(tokens_of(&t, P0, "Spawn"), 3);
    // Two Pawns, one Bears dies: each Pawn triggers.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Pawn of Ulamog");
    t.battlefield(P0, "Pawn of Ulamog");
    let bears = t.battlefield(P0, "Grizzly Bears");
    yes(&mut t, P0, 2);
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(tokens_of(&t, P0, "Spawn"), 2);
}

#[test]
fn druids_repository_counts_each_attacker_and_spends_counters_as_mana_abilities() {
    cr!("508.1", "603.2c", "605.1a", "605.3a");
    ruling!(
        "Druids' Repository",
        "The first ability of Druids’ Repository will trigger once for each creature you control that attacks."
    );
    ruling!(
        "Druids' Repository",
        "You can activate the second ability of Druids’ Repository multiple times in the same turn. It’s a mana ability, doesn’t use the stack, and can’t be responded to."
    );
    let mut t = TestGame::new(2);
    let repo = t.battlefield(P0, "Druids' Repository");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    let c = t.battlefield(P0, "Elite Vanguard");
    t.attack(&[(a, P1.into()), (b, P1.into()), (c, P1.into())], &[]);
    assert_eq!(t.counters(repo, counters::CHARGE), 3);
    t.set_step(P0, Step::PostcombatMain);
    for color in [0, 4] {
        t.answer(P0, DecisionKind::Option, Answer::Index(color));
        t.activate(P0, repo, 0, &[]).unwrap();
        assert_eq!(t.stack_len(), 0);
    }
    assert_eq!(mana(&t, P0, W), 1);
    assert_eq!(mana(&t, P0, G), 1);
    assert_eq!(t.counters(repo, counters::CHARGE), 1);
}

#[test]
fn militias_pride_token_enters_attacking_without_being_declared() {
    cr!("508.4", "508.1");
    ruling!(
        "Militia's Pride",
        "The token enters attacking. It doesn't trigger \"when a creature attacks\" triggers and is not subject to restrictions on declaring attackers."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Militia's Pride");
    let repo = t.battlefield(P0, "Druids' Repository");
    // Ghostly Prison: "Creatures can't attack you unless their controller pays {2} for
    // each creature they control that's attacking you."
    t.battlefield(P1, "Ghostly Prison");
    let bears = t.battlefield(P0, "Grizzly Bears");
    // {2} for the Bears; {W} for the token.
    t.lands(P0, "Plains", 3);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.attack(&[(bears, P1.into())], &[]);
    assert_eq!(tokens_of(&t, P0, "Kithkin"), 1);
    // Only the declared attacker triggered Druids' Repository.
    assert_eq!(t.counters(repo, counters::CHARGE), 1);
    // The token was attacking without paying Ghostly Prison's cost.
    assert_eq!(t.life(P1), 17);
}

#[test]
fn star_compass_colors_of_basic_lands() {
    cr!("106.7", "305.6");
    ruling!(
        "Star Compass",
        "If a basic land you control gains another basic land type or gains another ability that lets it produce mana, consider those effects when determining what colors of mana that land can produce."
    );
    ruling!(
        "Star Compass",
        "The colors of mana are white, blue, black, red, and green. Colorless is a type but not a color."
    );
    // Urborg, Tomb of Yawgmoth makes the Forest a Swamp too: it could produce {B}.
    let mut t = TestGame::new(2);
    let compass = t.battlefield(P0, "Star Compass");
    t.battlefield(P0, "Forest");
    t.battlefield(P0, "Urborg, Tomb of Yawgmoth");
    let from = t.asked().len();
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    tap(&mut t, P0, compass);
    let opts: Vec<String> = t.asked()[from..]
        .iter()
        .find_map(|(_, d)| match d {
            mtg_engine::decision::Decision::ChooseOption { options, .. } => Some(options.clone()),
            _ => None,
        })
        .unwrap();
    assert_eq!(opts.len(), 2, "{opts:?}");
    assert_eq!(pool(&t, P0), 1);
    assert!(mana(&t, P0, B) + mana(&t, P0, G) == 1);
    // Wastes produces {C}, which isn't a color: Star Compass produces nothing.
    let mut t = TestGame::new(2);
    let compass = t.battlefield(P0, "Star Compass");
    t.battlefield(P0, "Wastes");
    let _ = t.activate(P0, compass, 0, &[]);
    assert_eq!(pool(&t, P0), 0);
}

#[test]
fn white_lotus_tile_counts_the_largest_group_sharing_a_type() {
    cr!("205.3m", "106.1a");
    ruling!(
        "White Lotus Tile",
        "You'll always use the greatest number possible for X. You don't need to choose any creature types."
    );
    let mut t = TestGame::new(2);
    let tile = t.battlefield(P0, "White Lotus Tile");
    for (name, types) in [
        ("A", "Creature — Human Soldier"),
        ("B", "Creature — Warrior Ally"),
        ("C", "Creature — Warrior"),
        ("D", "Creature — Human Warrior"),
    ] {
        let def = custom_card(name, types, "{1}", Some((1, 1)), "");
        t.custom(P0, def, Zone::Battlefield);
    }
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    tap(&mut t, P0, tile);
    assert_eq!(mana(&t, P0, U), 3);
}

#[test]
fn an_artifact_tapped_for_improvise_then_sacrificed_as_a_cost() {
    cr!("702.126a", "601.2g", "601.2h");
    ruling!(
        "Inspiring Statuary",
        "you may tap that permanent (if it's an artifact) for the spell's improvise ability before you sacrifice it to pay that cost."
    );
    let mut t = TestGame::new(2);
    let statuary = t.battlefield(P0, "Inspiring Statuary");
    let orni = t.battlefield(P0, "Ornithopter");
    let mountain = t.battlefield(P0, "Mountain");
    // Shrapnel Blast ({1}{R}): "As an additional cost to cast this spell, sacrifice an
    // artifact. Shrapnel Blast deals 5 damage to any target."
    let blast = t.hand(P0, "Shrapnel Blast");
    t.answer_choose(P0, &[obj(orni)]);
    t.answer_choose(P0, &[obj(orni)]);
    t.cast(P0, blast).target(P1).go();
    assert!(t.obj_now(mountain).tapped);
    assert!(!t.obj_now(statuary).tapped, "the Ornithopter paid {{1}}");
    assert_eq!(t.zone(orni), Zone::Graveyard(P0));
    t.resolve_all();
    assert_eq!(t.life(P1), 15);
}

#[test]
fn archway_of_innovation_and_inspiring_statuary_improvise_once() {
    cr!("702.126c");
    ruling!(
        "Archway of Innovation",
        "Multiple instances of improvise are redundant."
    );
    ruling!(
        "Inspiring Statuary",
        "Multiple instances of improvise are redundant."
    );
    let mut t = TestGame::new(2);
    let statuary = t.battlefield(P0, "Inspiring Statuary");
    let archway = t.battlefield(P0, "Archway of Innovation");
    let islands = t.lands(P0, "Island", 3);
    t.activate(P0, archway, 1, &[]).unwrap();
    t.resolve_all();
    let orni = t.battlefield(P0, "Ornithopter");
    // Divination ({2}{U}) with two Islands left: one improvise choice of up to two
    // artifacts, not two.
    let from = t.asked().len();
    t.answer_choose(P0, &[obj(statuary), obj(orni)]);
    let div = t.hand(P0, "Divination");
    t.cast(P0, div).go();
    let prompts = t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, mtg_engine::decision::Decision::ChooseEntities { prompt, .. } if prompt.contains("improvise")))
        .count();
    assert_eq!(prompts, 1);
    assert!(t.obj_now(statuary).tapped && t.obj_now(orni).tapped);
    assert_eq!(islands.iter().filter(|i| !t.obj_now(**i).tapped).count(), 1);
}

#[test]
fn prismatic_geoscope_counts_basic_land_types() {
    cr!("305.6", "106.1b", "605.3b");
    ruling!(
        "Prismatic Geoscope",
        "Most nonbasic lands don't have basic land types, even if they produce colored mana."
    );
    ruling!(
        "Prismatic Geoscope",
        "Prismatic Geoscope can't produce colorless mana."
    );
    ruling!(
        "Prismatic Geoscope",
        "The colors of mana that Prismatic Geoscope can produce aren't related to the specific basic land types among lands you control."
    );
    ruling!(
        "Prismatic Geoscope",
        "Count each subtype that appears one or more times toward the number of basic land types among lands you control."
    );
    ruling!(
        "Prismatic Geoscope",
        "You can activate Prismatic Geoscope's mana ability if you have no basic land types among lands you control, but it won't produce any mana."
    );
    // No lands: it can be activated and produces nothing.
    let mut t = TestGame::new(2);
    let scope = t.battlefield(P0, "Prismatic Geoscope");
    assert!(t.activate(P0, scope, 0, &[]).is_ok());
    assert!(t.obj_now(scope).tapped);
    assert_eq!(pool(&t, P0), 0);
    // Caves of Koilos has no basic land types.
    let mut t = TestGame::new(2);
    let scope = t.battlefield(P0, "Prismatic Geoscope");
    t.battlefield(P0, "Caves of Koilos");
    t.activate(P0, scope, 0, &[]).unwrap();
    assert_eq!(pool(&t, P0), 0);
    // Two Plains and an Island: two types; {R}{G}, colors no land of them produces.
    let mut t = TestGame::new(2);
    let scope = t.battlefield(P0, "Prismatic Geoscope");
    t.lands(P0, "Plains", 2);
    t.battlefield(P0, "Island");
    let from = t.asked().len();
    for c in [3, 4] {
        t.answer(P0, DecisionKind::Option, Answer::Index(c));
    }
    t.activate(P0, scope, 0, &[]).unwrap();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(pool(&t, P0), 2);
    assert_eq!(mana(&t, P0, R), 1);
    assert_eq!(mana(&t, P0, G), 1);
    // Only the five colors are offered.
    for (_, d) in &t.asked()[from..] {
        if let mtg_engine::decision::Decision::ChooseOption { options, .. } = d {
            assert_eq!(options.len(), 5, "{options:?}");
        }
    }
}

#[test]
fn nyx_lotus_is_a_mana_ability() {
    cr!("605.1a", "605.3b", "700.5");
    ruling!(
        "Nyx Lotus",
        "Nyx Lotus’s second ability is a mana ability. It doesn’t use the stack and can’t be responded to."
    );
    let mut t = TestGame::new(2);
    let lotus = t.battlefield(P0, "Nyx Lotus");
    // Kalonian Tusker ({G}{G}): devotion to green 2.
    t.battlefield(P0, "Kalonian Tusker");
    t.answer(P0, DecisionKind::Option, Answer::Index(4));
    tap(&mut t, P0, lotus);
    assert_eq!(t.stack_len(), 0);
    assert_eq!(mana(&t, P0, G), 2);
}

#[test]
fn deepfire_elemental_costs_twice_x_plus_one() {
    cr!("107.3", "601.2f", "602.2b");
    ruling!(
        "Deepfire Elemental",
        "for Deepfire Elemental to destroy a creature with mana value 4 would cost {4}+{4}+{1} = {9}."
    );
    let mut t = TestGame::new(2);
    let deepfire = t.battlefield(P0, "Deepfire Elemental");
    let giant = t.battlefield(P1, "Hill Giant");
    fill_pool(&mut t, P0, 3);
    let before = pool(&t, P0);
    t.answer(P0, DecisionKind::X, Answer::Number(4));
    t.activate(P0, deepfire, 0, &[obj(giant)]).unwrap();
    assert_eq!(before - pool(&t, P0), 9);
    t.resolve_all();
    assert!(!t.on_battlefield(giant));
}

#[test]
fn omnath_keeps_green_mana_with_its_restrictions() {
    cr!("106.4", "106.6");
    ruling!(
        "Omnath, Locus of Mana",
        "If a green mana you produce has certain restrictions or riders associated with it (for example, if it was produced by Ancient Ziggurat), they'll apply to that mana no matter when you spend it."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Omnath, Locus of Mana");
    let zig = t.battlefield(P0, "Ancient Ziggurat");
    t.answer(P0, DecisionKind::Option, Answer::Index(4));
    tap(&mut t, P0, zig);
    t.advance_to(P0, Step::End);
    assert_eq!(restricted(&t, P0, G, true), 1);
    t.set_step(P0, Step::PostcombatMain);
    // It can't pay for Giant Growth, an instant...
    let bears = t.battlefield(P0, "Grizzly Bears");
    let growth = t.hand(P0, "Giant Growth");
    assert!(t.cast(P0, growth).target(bears).try_go().is_err());
    // ... but can pay for a creature spell.
    let elf = t.hand(P0, "Llanowar Elves");
    assert!(t.cast(P0, elf).try_go().is_ok());
    assert_eq!(pool(&t, P0), 0);
}

#[test]
fn unspent_mana_is_lost_at_the_end_of_the_step_its_keeper_left_in() {
    cr!("106.4", "500.4");
    ruling!(
        "Omnath, Locus of Mana",
        "Once Omnath leaves the battlefield, you have until the end of the current step or phase to spend your unspent green mana before it too is lost as normal."
    );
    ruling!(
        "Upwelling",
        "once it leaves the battlefield, they have until the end of the current step (or phase) to use the mana before it disappears."
    );
    for name in ["Omnath, Locus of Mana", "Upwelling"] {
        let mut t = TestGame::new(2);
        let keeper = t.battlefield(P0, name);
        t.set_step(P0, Step::Upkeep);
        t.g.players[0].mana_pool.add_type(G, 2);
        t.advance_to(P0, Step::Draw);
        assert_eq!(mana(&t, P0, G), 2, "{name}: kept as the upkeep ended");
        destroy(&mut t, keeper);
        t.resolve_all();
        assert_eq!(mana(&t, P0, G), 2, "{name}: still in the pool this step");
        t.advance_to(P0, Step::PrecombatMain);
        assert_eq!(mana(&t, P0, G), 0, "{name}: lost as the draw step ended");
    }
}

// ---------------------------------------------------------------------------------------
// Spending mana as though it were another type
// ---------------------------------------------------------------------------------------

#[test]
fn quicksilver_elemental_unattaches_an_opponents_leonin_bola() {
    cr!("113.2c", "701.3d", "602.2b");
    ruling!(
        "Quicksilver Elemental",
        "You can actually unattach your opponent's Bola as part of the cost of activating the Elemental-copied ability."
    );
    let mut t = TestGame::new(2);
    let qe = t.battlefield(P0, "Quicksilver Elemental");
    let bears = t.battlefield(P1, "Grizzly Bears");
    // Leonin Bola: "Equipped creature has "{T}, Unattach Leonin Bola: Tap target
    // creature."" — on P1's Bears.
    let bola = attach_new(&mut t, P1, "Leonin Bola", bears);
    let victim = t.battlefield(P1, "Hill Giant");
    t.g.players[0].mana_pool.add_type(U, 1);
    t.activate(P0, qe, 0, &[obj(bears)]).unwrap();
    t.resolve_all();
    t.answer_targets(P0, &[obj(victim)]);
    activate_containing(&mut t, P0, qe, "Unattach").unwrap();
    t.resolve_all();
    assert!(t.obj_now(qe).tapped);
    assert_eq!(attached_to(&t, bola), None, "P1's Bola was unattached");
    assert!(t.obj_now(victim).tapped);
}

#[test]
fn vizier_of_the_menagerie_looks_at_the_top_card_any_time() {
    cr!("401.5", "406.3");
    ruling!(
        "Vizier of the Menagerie",
        "Vizier of the Menagerie lets you look at the top card of your library whenever you want (with one restriction—see below), even if you don't have priority."
    );
    let mut t = TestGame::new(2);
    let second = t.library_top(P0, "Grizzly Bears");
    let top = t.library_top(P0, "Hill Giant");
    assert!(!mtg_engine::zones::can_see_in_library(&t.g, P0, top));
    t.battlefield(P0, "Vizier of the Menagerie");
    t.g.recompute();
    // During the opponent's turn, with no priority.
    t.set_step(P1, Step::Upkeep);
    t.g.turn.priority = None;
    assert!(mtg_engine::zones::can_see_in_library(&t.g, P0, top));
    assert!(!mtg_engine::zones::can_see_in_library(&t.g, P0, second));
    assert!(!mtg_engine::zones::can_see_in_library(&t.g, P1, top));
}

#[test]
fn vizier_of_the_menagerie_costs_including_emerge() {
    cr!("118.9", "702.119a", "601.2f");
    ruling!(
        "Vizier of the Menagerie",
        "You'll still pay all costs for that spell, including additional costs. You may also pay alternative costs such as emerge or that of As Foretold."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Vizier of the Menagerie");
    // Elder Deep-Fiend ({8}, emerge {5}{U}{U}) from the top of the library, emerging from
    // Hill Giant (mana value 4): {1}{U}{U}.
    let fiend = t.library_top(P0, "Elder Deep-Fiend");
    let giant = t.battlefield(P0, "Hill Giant");
    t.g.players[0].mana_pool.add_type(G, 1);
    t.g.players[0].mana_pool.add_type(U, 2);
    t.answer_choose(P0, &[obj(giant)]);
    t.cast(P0, fiend)
        .method(mtg_engine::object::CastMethod::Keyword(
            keywords::KeywordKind::Emerge,
        ))
        .go();
    assert_eq!(t.zone(giant), Zone::Graveyard(P0));
    assert_eq!(pool(&t, P0), 0);
    // Without mana, it can't be cast from the library for nothing.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Vizier of the Menagerie");
    let fiend = t.library_top(P0, "Elder Deep-Fiend");
    assert!(t.cast(P0, fiend).try_go().is_err());
}

/// An agent for P0 that answers every choice of an option with the first one and records
/// P0's hand size each time it's asked to choose one (otherwise it passes and takes the
/// engine's defaults).
struct HandWatcher(std::sync::Arc<std::sync::Mutex<Vec<usize>>>);

impl mtg_engine::decision::Agent for HandWatcher {
    fn decide(
        &mut self,
        g: &mtg_engine::game::Game,
        p: PlayerId,
        d: &mtg_engine::decision::Decision,
    ) -> Answer {
        use mtg_engine::decision::{Action, Decision};
        match d {
            Decision::ChooseOption { .. } => {
                self.0.lock().unwrap().push(g.player(p).hand.len());
                Answer::Index(0)
            }
            Decision::Priority { .. } => Answer::Action(Action::Pass),
            _ => Answer::Default,
        }
    }
}

#[test]
fn manamorphose_colors_are_chosen_before_the_draw() {
    cr!("608.2c", "106.1b");
    ruling!(
        "Manamorphose",
        "You choose which color or colors of mana to add before you draw a card."
    );
    let mut t = TestGame::new(2);
    let seen = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    t.g.agents.0.lock().unwrap()[0] = Box::new(HandWatcher(seen.clone()));
    let morph = cast_new(&mut t, P0, "Manamorphose", &[]);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.zone(morph), Zone::Graveyard(P0));
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(mana(&t, P0, W), 2);
    let seen = seen.lock().unwrap().clone();
    // (A choice of how to pay {R/G} may come first, while casting.)
    assert!(seen.len() >= 2, "{seen:?}");
    assert!(
        seen.iter().all(|h| *h == hand),
        "both colors chosen before the draw: {seen:?}"
    );
}

#[test]
fn mirari_s_wake_leaving_makes_damage_lethal() {
    cr!("704.5g", "120.6", "611.3a");
    ruling!(
        "Mirari's Wake",
        "Because damage remains marked on a creature until the cleanup step or an effect removes that damage, nonlethal damage dealt to creatures you control may become lethal if Mirari's Wake leaves the battlefield."
    );
    let mut t = TestGame::new(2);
    let wake = t.battlefield(P0, "Mirari's Wake");
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_resolve(&mut t, P1, "Shock", &[obj(bears)]);
    assert!(t.on_battlefield(bears), "a 3/3 with 2 damage");
    destroy(&mut t, wake);
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
}
