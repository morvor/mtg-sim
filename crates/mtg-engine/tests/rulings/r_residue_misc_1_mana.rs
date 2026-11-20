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
    *tokens(t, P0).iter().find(|id| !before.contains(id)).unwrap()
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
    for (name, colors) in [("Terrarion", [1, 1]), ("Terrarion", [0, 3]), ("Wizard's Rockets", [2, 4])] {
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
    ruling!("Doubling Cube", "Any restrictions on the unspent mana aren't copied.");
    ruling!("Doubling Cube", "Doubling Cube's ability is a mana ability.");
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
    for name in ["Mirari's Wake", "Zendikar Resurgent", "Nikya of the Old Ways"] {
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
