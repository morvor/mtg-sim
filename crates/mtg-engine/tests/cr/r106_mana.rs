//! CR 106: Mana.

use super::r105_util::*;
use mtg_engine::ability::*;
use mtg_engine::mana::{ManaCost, ManaRestriction, ManaType};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

use ManaType::*;

#[test]
fn mana_pays_costs_of_spells_and_abilities() {
    cr!("106.1");
    let mut t = TestGame::new(2);
    let forests = t.lands(P0, "Forest", 2);
    let bears = t.hand(P0, "Grizzly Bears");
    t.cast(P0, bears).go();
    // The Forests were tapped for mana, which was spent to pay {1}{G}.
    assert!(forests.iter().all(|f| t.obj_now(*f).tapped));
    assert_eq!(pool_total(&t, P0), 0);
    t.resolve();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    // Activating an ability: Indigo Faerie's {U} ability is paid from the pool.
    let faerie = t.battlefield(P0, "Indigo Faerie");
    add_pool(&mut t, P0, &[U]);
    t.activate(P0, faerie, 0, &[Entity::Object(faerie)])
        .unwrap();
    assert_eq!(pool_total(&t, P0), 0);
}

#[test]
fn five_colors_and_six_types_of_mana() {
    cr!("106.1a", "106.1b");
    // Five colors of mana, one per color; the sixth type is colorless.
    assert_eq!(ManaType::ALL.len(), 6);
    let colored: Vec<ManaType> = ManaType::ALL
        .iter()
        .copied()
        .filter(|t| t.color().is_some())
        .collect();
    assert_eq!(colored, vec![W, U, B, R, G]);
    assert_eq!(C.color(), None);
    for c in Color::ALL {
        assert_eq!(ManaType::from_color(c).color(), Some(c));
    }
    // Colorless is a type of mana: Wastes adds {C}, which can pay a {C} cost that no
    // colored mana can pay.
    let mut t = TestGame::new(2);
    let wastes = t.battlefield(P0, "Wastes");
    t.activate(P0, wastes, 0, &[]).unwrap();
    assert_eq!(pool_count(&t, P0, C), 1);
    let colorless_cost = ManaCost::parse("{C}").unwrap();
    let ctx = mtg_engine::mana::SpendContext::default();
    assert!(mtg_engine::mana::find_payment(
        &t.g.player(P0).mana_pool.mana,
        &colorless_cost,
        &ctx,
        0
    )
    .is_some());
    let colored_pool = vec![mtg_engine::mana::Mana::new(G)];
    assert!(mtg_engine::mana::find_payment(&colored_pool, &colorless_cost, &ctx, 0).is_none());
}

#[test]
fn mana_symbols_represent_mana_and_costs() {
    cr!("106.2");
    let mut t = TestGame::new(2);
    // "Add {B}{B}{B}": the symbols represent the mana added.
    let ritual = t.hand(P0, "Dark Ritual");
    add_pool(&mut t, P0, &[B]);
    t.cast(P0, ritual).go();
    t.resolve();
    assert_eq!(pool_count(&t, P0, B), 3);
    // The symbols in a mana cost represent the cost: {1}{B} parses to one generic and
    // one black symbol.
    let c = ManaCost::parse("{1}{B}").unwrap();
    assert_eq!(c.to_string(), "{1}{B}");
    assert_eq!(c.mana_value(), 2);
}

#[test]
fn mana_from_spells_and_from_non_mana_abilities() {
    cr!("106.3");
    let mut t = TestGame::new(2);
    // Produced by a spell: the source of the mana is that spell.
    add_pool(&mut t, P0, &[B]);
    let ritual = t.hand(P0, "Dark Ritual");
    let spell = t.cast(P0, ritual).go();
    t.resolve();
    let pool = t.g.player(P0).mana_pool.mana.clone();
    assert_eq!(pool.len(), 3);
    assert!(pool.iter().all(|m| m.source == Some(spell)));
    // Produced by an ability that isn't a mana ability (a loyalty ability uses the
    // stack): the source is the ability's source, Chandra.
    t.g.players[0].mana_pool.mana.clear();
    let chandra = t.battlefield(P0, "Chandra, Torch of Defiance");
    let ab = t.activate(P0, chandra, 0, &[]).unwrap();
    assert!(ab.is_some(), "loyalty abilities use the stack");
    assert_eq!(pool_total(&t, P0), 0);
    t.resolve();
    let pool = t.g.player(P0).mana_pool.mana.clone();
    assert_eq!(pool.len(), 2);
    assert!(pool.iter().all(|m| m.ty == R && m.source == Some(chandra)));
    // Produced by a mana ability: the source is the permanent.
    let elves = t.battlefield(P0, "Llanowar Elves");
    t.activate(P0, elves, 0, &[]).unwrap();
    assert!(t
        .g
        .player(P0)
        .mana_pool
        .mana
        .iter()
        .any(|m| m.ty == G && m.source == Some(elves)));
}

#[test]
fn mana_pool_holds_unspent_mana_until_the_step_ends() {
    cr!("106.4");
    let mut t = TestGame::new(2);
    let ritual = t.hand(P0, "Dark Ritual");
    add_pool(&mut t, P0, &[B]);
    t.cast(P0, ritual).go();
    t.resolve();
    assert_eq!(pool_count(&t, P0, B), 3);
    // Spend one immediately; the rest stays in the pool as unspent mana.
    let pet = t.hand(P0, "Blood Pet");
    t.cast(P0, pet).go();
    t.resolve();
    assert_eq!(pool_count(&t, P0, B), 2);
    // At the end of the step (precombat main → beginning of combat), it empties.
    t.advance_to_step(Step::BeginningOfCombat);
    assert_eq!(pool_total(&t, P0), 0);
}

#[test]
fn mana_pool_empties_between_steps_of_the_same_phase() {
    cr!("106.4");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::Upkeep);
    add_pool(&mut t, P0, &[G, G]);
    add_pool(&mut t, P1, &[R]);
    t.advance_to_step(Step::Draw);
    assert_eq!(pool_total(&t, P0), 0);
    assert_eq!(pool_total(&t, P1), 0);
}

#[test]
fn undefined_mana_type_produces_no_mana() {
    cr!("106.5");
    ruling!(
        "Exotic Orchard",
        "Exotic Orchard can't be tapped for colorless mana"
    );
    let mut t = TestGame::new(2);
    // Meteor Crater with no colored permanents: no mana.
    let crater = t.battlefield(P0, "Meteor Crater");
    t.activate(P0, crater, 0, &[]).unwrap();
    assert_eq!(pool_total(&t, P0), 0);
    // With a green permanent, it adds {G}.
    t.g.objects[crater.0 as usize].tapped = false;
    t.battlefield(P0, "Grizzly Bears");
    t.activate(P0, crater, 0, &[]).unwrap();
    assert_eq!(pool_count(&t, P0, G), 1);
    // Exotic Orchard when the only land an opponent controls could produce only {C}:
    // colorless isn't a color, so no mana.
    let orchard = t.battlefield(P0, "Exotic Orchard");
    t.battlefield(P1, "Wastes");
    t.g.players[0].mana_pool.mana.clear();
    t.activate(P0, orchard, 0, &[]).unwrap();
    assert_eq!(pool_total(&t, P0), 0);
}

#[test]
fn restricted_mana_keeps_its_type() {
    cr!("106.6");
    let mut t = TestGame::new(2);
    // Ancient Ziggurat: "Add one mana of any color. Spend this mana only to cast a
    // creature spell."
    let zig = t.battlefield(P0, "Ancient Ziggurat");
    t.answer(P0, DecisionKind::Option, Answer::Index(4)); // green
    t.activate(P0, zig, 0, &[]).unwrap();
    let m = &t.g.player(P0).mana_pool.mana[0];
    assert_eq!(m.ty, G);
    assert_eq!(
        m.restriction,
        Some(ManaRestriction::SpellOfType(CardType::Creature))
    );
    // It can't be spent on a noncreature spell (Giant Growth)...
    let growth = t.hand(P0, "Giant Growth");
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert!(t.cast(P0, growth).target(bears).try_go().is_err());
    // ...but, being green mana, it pays the {G} of a creature spell.
    let elves = t.hand(P0, "Llanowar Elves");
    t.cast(P0, elves).go();
    assert_eq!(pool_total(&t, P0), 0);
}

#[test]
fn doubling_cube_doubles_restricted_mana_without_the_restriction() {
    cr!("106.6");
    let mut t = TestGame::new(2);
    let zigs = t.lands(P0, "Ancient Ziggurat", 2);
    t.answer(P0, DecisionKind::Option, Answer::Index(3)); // red
    t.activate(P0, zigs[0], 0, &[]).unwrap();
    t.answer(P0, DecisionKind::Option, Answer::Index(4)); // green
    t.activate(P0, zigs[1], 0, &[]).unwrap();
    // Doubling Cube's {3} is paid with other lands.
    t.lands(P0, "Wastes", 3);
    let cube = t.battlefield(P0, "Doubling Cube");
    t.activate(P0, cube, 0, &[]).unwrap();
    let pool = t.g.player(P0).mana_pool.mana.clone();
    assert_eq!(pool.iter().filter(|m| m.ty == R).count(), 2);
    assert_eq!(pool.iter().filter(|m| m.ty == G).count(), 2);
    // {R}{G} of it can be spent on anything.
    assert_eq!(pool.iter().filter(|m| m.restriction.is_none()).count(), 2);
    assert_eq!(pool.iter().filter(|m| m.restriction.is_some()).count(), 2);
    let restricted: Vec<ManaType> = pool
        .iter()
        .filter(|m| m.restriction.is_some())
        .map(|m| m.ty)
        .collect();
    assert!(restricted.contains(&R) && restricted.contains(&G));
}

#[test]
fn increased_mana_keeps_the_restrictions() {
    cr!("106.6a", "106.12b");
    ruling!(
        "Nyxbloom Ancient",
        "that will apply to all the mana it produces this way"
    );
    let mut t = TestGame::new(2);
    // Mana Reflection: "If you tap a permanent for mana, it produces twice as much of that
    // mana instead."
    t.battlefield(P0, "Mana Reflection");
    let zig = t.battlefield(P0, "Ancient Ziggurat");
    t.answer(P0, DecisionKind::Option, Answer::Index(0)); // white
    t.activate(P0, zig, 0, &[]).unwrap();
    let pool = t.g.player(P0).mana_pool.mana.clone();
    assert_eq!(pool.len(), 2);
    for m in &pool {
        assert_eq!(m.ty, W);
        assert_eq!(
            m.restriction,
            Some(ManaRestriction::SpellOfType(CardType::Creature))
        );
        assert_eq!(m.source, Some(zig));
    }
}

#[test]
fn mana_can_carry_a_delayed_trigger_for_when_it_is_spent() {
    cr!("106.6");
    let mut t = TestGame::new(2);
    // Pyromancer's Goggles: "{T}: Add {R}. When that mana is spent to cast a red instant
    // or sorcery spell, copy that spell and you may choose new targets for the copy."
    let goggles = t.battlefield(P0, "Pyromancer's Goggles");
    t.activate(P0, goggles, 0, &[]).unwrap();
    assert_eq!(pool_count(&t, P0, R), 1);
    // The rider doesn't change the mana's type.
    assert!(t.g.player(P0).mana_pool.mana[0].rider.is_some());
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    t.settle();
    // The spell and the delayed trigger on top of it.
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(t.life(P1), 14);

    // Spent on a spell that doesn't match (a green instant): no trigger.
    t.g.objects[goggles.0 as usize].tapped = false;
    t.activate(P0, goggles, 0, &[]).unwrap();
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 1);
    let growth = t.hand(P0, "Giant Growth");
    t.cast(P0, growth).target(bears).go();
    t.settle();
    assert_eq!(t.stack_len(), 1);
}

#[test]
fn increased_mana_creates_a_delayed_trigger_for_each_mana() {
    cr!("106.6a");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mana Reflection");
    let goggles = t.battlefield(P0, "Pyromancer's Goggles");
    t.activate(P0, goggles, 0, &[]).unwrap();
    let pool = t.g.player(P0).mana_pool.mana.clone();
    assert_eq!(pool.len(), 2);
    let ids: Vec<u64> = pool.iter().map(|m| m.rider.as_ref().unwrap().id).collect();
    assert_ne!(
        ids[0], ids[1],
        "a separate delayed triggered ability for each mana"
    );
    // Both {R} are spent on Searing Spear ({1}{R}): two triggers, two copies.
    let spear = t.hand(P0, "Searing Spear");
    t.cast(P0, spear).target(P1).go();
    t.settle();
    assert_eq!(t.stack_len(), 3);
    t.resolve_all();
    assert_eq!(t.life(P1), 11);
}

#[test]
fn could_produce_exotic_orchard_examples() {
    cr!("106.7");
    ruling!(
        "Exotic Orchard",
        "Lands that produce mana based only on what other lands \"could produce\" won't help each other"
    );
    // The opponent controls no lands: no mana.
    let mut t = TestGame::new(2);
    let orchard = t.battlefield(P0, "Exotic Orchard");
    t.activate(P0, orchard, 0, &[]).unwrap();
    assert_eq!(pool_total(&t, P0), 0);

    // Each player controls only an Exotic Orchard: no mana.
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Exotic Orchard");
    let theirs = t.battlefield(P1, "Exotic Orchard");
    t.activate(P0, mine, 0, &[]).unwrap();
    assert_eq!(pool_total(&t, P0), 0);

    // You control a Forest and an Exotic Orchard, the opponent an Exotic Orchard: each
    // Exotic Orchard could produce {G}.
    t.g.objects[mine.0 as usize].tapped = false;
    t.battlefield(P0, "Forest");
    t.activate(P0, mine, 0, &[]).unwrap();
    assert_eq!(pool_count(&t, P0, G), 1);
    t.g.turn.priority = Some(P1);
    t.activate(P1, theirs, 0, &[]).unwrap();
    assert_eq!(pool_count(&t, P1, G), 1);
}

#[test]
fn could_produce_includes_colorless_for_any_type_and_ignores_costs() {
    cr!("106.7");
    ruling!(
        "Exotic Orchard",
        "It doesn't matter whether Vivid Crag has a charge counter on it"
    );
    let mut t = TestGame::new(2);
    // Reflecting Pool: "any type" includes colorless.
    let pool_land = t.battlefield(P0, "Reflecting Pool");
    t.battlefield(P0, "Wastes");
    t.activate(P0, pool_land, 0, &[]).unwrap();
    assert_eq!(pool_count(&t, P0, C), 1);
    // Exotic Orchard ignores the costs of the opponent's abilities: a Vivid Crag with no
    // charge counters could still produce any color.
    let orchard = t.battlefield(P0, "Exotic Orchard");
    let crag = t.battlefield(P1, "Vivid Crag");
    assert_eq!(t.counters(crag, "charge"), 0);
    t.answer(P0, DecisionKind::Option, Answer::Index(1)); // blue
    t.activate(P0, orchard, 0, &[]).unwrap();
    assert_eq!(pool_count(&t, P0, U), 1);
}

#[test]
fn could_produce_accounts_for_replacement_effects() {
    cr!("106.7");
    ruling!("Exotic Orchard", "such as Contamination's effect");
    let mut t = TestGame::new(2);
    // Contamination: lands tapped for mana produce {B} instead. The opponent's Forest
    // could therefore produce only {B}.
    t.battlefield(P1, "Contamination");
    t.battlefield(P1, "Forest");
    let orchard = t.battlefield(P0, "Exotic Orchard");
    let opts_before = t.asked().len();
    t.activate(P0, orchard, 0, &[]).unwrap();
    // Only one option ({B}) was possible, so no color choice was asked, and
    // Contamination also replaces the Orchard's own production.
    assert_eq!(t.asked().len(), opts_before);
    assert_eq!(pool_count(&t, P0, B), 1);
    assert_eq!(pool_total(&t, P0), 1);
}

/// A permanent whose mana cost exercises every kind of mana symbol.
fn fancy_cost() -> CardDef {
    card_with(
        "Fancy Idol",
        "{2}{S}{R/W}{2/G}{W/P}{C/U}",
        "Artifact",
        None,
        vec![],
    )
}

#[test]
fn adding_mana_represented_by_symbols() {
    cr!("106.8", "106.9", "106.10", "106.11");
    let mut t = TestGame::new(2);
    let idol = put(&mut t, P0, fancy_cost());
    let res = t.battlefield(P0, "Elemental Resonance");
    assert!(t.g.attach(res, Entity::Object(idol)));
    t.set_step(P0, Step::Upkeep);
    // Choices, in symbol order: {R/W} → W (106.8), {2/G} → the generic half (106.8),
    // {C/U} → U.
    t.answer(P0, DecisionKind::Option, Answer::Index(1)); // {R/W}: [R, W] → W
    t.answer(P0, DecisionKind::Option, Answer::Index(1)); // {2/G}: [G, C] → C (×2)
    t.answer(P0, DecisionKind::Option, Answer::Index(1)); // {C/U}: [C, U] → U
    t.advance_to(P0, Step::PrecombatMain);
    t.resolve();
    // {2} → CC (106.10); {S} → C (106.11); {2/G} generic half → CC; {W/P} → W (106.9).
    assert_eq!(pool_count(&t, P0, C), 5);
    assert_eq!(pool_count(&t, P0, W), 2);
    assert_eq!(pool_count(&t, P0, U), 1);
    assert_eq!(pool_total(&t, P0), 8);
}

#[test]
fn hybrid_mana_colored_half_adds_one_mana_of_that_color() {
    cr!("106.8");
    let mut t = TestGame::new(2);
    let recruit = t.battlefield(P0, "Boros Recruit");
    let res = t.battlefield(P0, "Elemental Resonance");
    assert!(t.g.attach(res, Entity::Object(recruit)));
    t.set_step(P0, Step::Upkeep);
    t.answer(P0, DecisionKind::Option, Answer::Index(0)); // {R/W}: R
    t.advance_to(P0, Step::PrecombatMain);
    t.resolve();
    assert_eq!(pool_count(&t, P0, R), 1);
    assert_eq!(pool_total(&t, P0), 1);
}

#[test]
fn tapping_for_mana_means_activating_a_mana_ability_with_tap_symbol() {
    cr!("106.12");
    let mut t = TestGame::new(2);
    // Leyline of Abundance: "Whenever you tap a creature for mana, add an additional {G}."
    t.battlefield(P0, "Leyline of Abundance");
    let elves = t.battlefield(P0, "Llanowar Elves");
    let pet = t.battlefield(P0, "Blood Pet");
    // Blood Pet's mana ability has no {T}: it isn't tapped for mana.
    t.activate(P0, pet, 0, &[]).unwrap();
    assert_eq!(pool_count(&t, P0, B), 1);
    assert_eq!(pool_count(&t, P0, G), 0);
    // Llanowar Elves' "{T}: Add {G}." is.
    t.activate(P0, elves, 0, &[]).unwrap();
    assert_eq!(pool_count(&t, P0, G), 2);
}

#[test]
fn tapping_a_land_by_other_means_isnt_tapping_it_for_mana() {
    cr!("106.12");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Manabarbs");
    let forest = t.battlefield(P1, "Forest");
    // An effect that just taps the land.
    let tapper = card_with(
        "Tapper",
        "{0}",
        "Instant",
        None,
        vec![spell_ab(
            vec![TargetSpec::object(
                Filter::Type(CardType::Land),
                "target land",
            )],
            Effect::Tap {
                what: Sel::Target(0),
            },
        )],
    );
    let s = put_in_hand(&mut t, P0, tapper);
    t.cast(P0, s).target(forest).go();
    t.resolve_all();
    assert!(t.obj_now(forest).tapped);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn tapped_for_mana_triggers_when_mana_is_produced() {
    cr!("106.12a");
    ruling!(
        "Manabarbs",
        "Manabarbs’s triggered abilities are put on the stack on top of it"
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Manabarbs");
    // Tapping lands to pay for a spell: the triggers wait until the spell is cast, then
    // go on the stack on top of it.
    t.lands(P1, "Mountain", 2);
    t.set_step(P1, Step::PrecombatMain);
    let shock = t.hand(P1, "Shock");
    t.g.turn.priority = Some(P1);
    t.cast(P1, shock).target(P0).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    assert_eq!(t.life(P1), 19);
    t.resolve();
    assert_eq!(t.life(P0), 18);
    // A mana ability that produces no mana doesn't count as tapping for mana (106.12a).
    let crater = t.battlefield(P1, "Meteor Crater");
    t.activate(P1, crater, 0, &[]).unwrap();
    t.settle();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.life(P1), 19);
}

#[test]
fn tapped_for_mana_of_a_specified_type() {
    cr!("106.12a");
    let mut t = TestGame::new(2);
    // "Whenever you tap a permanent for {C}, add an additional {C}."
    let horizon = card_from_text(
        "Horizon",
        "{2}",
        "Enchantment",
        None,
        "Whenever you tap a permanent for {C}, add an additional {C}.",
    );
    put(&mut t, P0, horizon);
    let forest = t.battlefield(P0, "Forest");
    let wastes = t.battlefield(P0, "Wastes");
    t.activate(P0, forest, 0, &[]).unwrap();
    assert_eq!(pool_total(&t, P0), 1);
    t.activate(P0, wastes, 0, &[]).unwrap();
    assert_eq!(pool_count(&t, P0, C), 2);
    assert_eq!(pool_total(&t, P0), 3);
}

#[test]
fn tapped_for_mana_triggered_mana_ability_adds_type_produced() {
    cr!("106.12a");
    let mut t = TestGame::new(2);
    // Dictate of Karametra: "Whenever a player taps a land for mana, that player adds one
    // mana of any type that land produced."
    t.battlefield(P0, "Dictate of Karametra");
    let island = t.battlefield(P1, "Island");
    t.g.turn.priority = Some(P1);
    t.activate(P1, island, 0, &[]).unwrap();
    assert_eq!(pool_count(&t, P1, U), 2);
    assert_eq!(pool_total(&t, P0), 0);
}

#[test]
fn mana_production_replacement_effects() {
    cr!("106.12b");
    ruling!(
        "Virtue of Strength",
        "it causes basic lands you tap for mana to produce more mana"
    );
    let mut t = TestGame::new(2);
    // Virtue of Strength: basic lands you tap for mana produce three times as much.
    t.battlefield(P0, "Virtue of Strength");
    let forest = t.battlefield(P0, "Forest");
    let arbor = t.battlefield(P0, "Dryad Arbor"); // a nonbasic Forest
    t.activate(P0, forest, 0, &[]).unwrap();
    assert_eq!(pool_count(&t, P0, G), 3);
    t.activate(P0, arbor, 0, &[]).unwrap();
    assert_eq!(pool_count(&t, P0, G), 4);
    // Another player's basic land isn't affected ("If you tap").
    let island = t.battlefield(P1, "Island");
    t.g.turn.priority = Some(P1);
    t.activate(P1, island, 0, &[]).unwrap();
    assert_eq!(pool_count(&t, P1, U), 1);
}

#[test]
fn several_mana_replacements_apply_in_the_chosen_order() {
    cr!("106.12b");
    let mut t = TestGame::new(2);
    // Contamination ({B} instead of any other type and amount) and Mana Reflection
    // (twice as much): the order matters.
    t.battlefield(P0, "Contamination");
    t.battlefield(P0, "Mana Reflection");
    let forests = t.lands(P0, "Forest", 2);
    // Contamination first, then Mana Reflection: {B}{B}.
    t.answer(P0, DecisionKind::Replacement, Answer::Index(0));
    t.activate(P0, forests[0], 0, &[]).unwrap();
    assert_eq!(pool_count(&t, P0, B), 2);
    // Mana Reflection first, then Contamination: {B}.
    t.g.players[0].mana_pool.mana.clear();
    t.answer(P0, DecisionKind::Replacement, Answer::Index(1));
    t.activate(P0, forests[1], 0, &[]).unwrap();
    assert_eq!(pool_count(&t, P0, B), 1);
    assert_eq!(pool_total(&t, P0), 1);
}

#[test]
fn triggered_mana_abilities_are_not_multiplied() {
    cr!("106.12b");
    ruling!(
        "Nyxbloom Ancient",
        "that triggered mana ability won’t be affected by Nyxbloom Ancient"
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Nyxbloom Ancient");
    let forest = t.battlefield(P0, "Forest");
    let wild = t.battlefield(P0, "Wild Growth");
    assert!(t.g.attach(wild, Entity::Object(forest)));
    t.activate(P0, forest, 0, &[]).unwrap();
    // {G}×3 from the Forest, plus one additional {G} from Wild Growth.
    assert_eq!(pool_count(&t, P0, G), 4);
}

#[test]
fn drain_power_moves_unspent_mana_with_its_sources_and_restrictions() {
    cr!("106.13");
    let mut t = TestGame::new(2);
    let forest = t.battlefield(P1, "Forest");
    let zig = t.battlefield(P1, "Ancient Ziggurat");
    // P1 already has an unspent {R} in their pool.
    add_pool(&mut t, P1, &[R]);
    t.answer(P1, DecisionKind::Option, Answer::Index(0)); // Ziggurat: white
    t.lands(P0, "Island", 2);
    let drain = t.hand(P0, "Drain Power");
    t.cast(P0, drain).target(P1).go();
    t.resolve();
    assert!(t.obj_now(forest).tapped && t.obj_now(zig).tapped);
    assert_eq!(pool_total(&t, P1), 0);
    let pool = t.g.player(P0).mana_pool.mana.clone();
    assert_eq!(pool.len(), 3);
    let g = pool.iter().find(|m| m.ty == G).unwrap();
    assert_eq!(g.source, Some(forest));
    let w = pool.iter().find(|m| m.ty == W).unwrap();
    assert_eq!(w.source, Some(zig));
    assert_eq!(
        w.restriction,
        Some(ManaRestriction::SpellOfType(CardType::Creature))
    );
    assert!(pool.iter().any(|m| m.ty == R));
}

#[test]
fn drain_power_on_yourself() {
    cr!("106.13");
    let mut t = TestGame::new(2);
    let islands = t.lands(P0, "Island", 3);
    let drain = t.hand(P0, "Drain Power");
    // Pay {U}{U} with two Islands; the third is tapped by Drain Power itself.
    t.cast(P0, drain).target(P0).go();
    t.resolve();
    assert!(islands.iter().all(|i| t.obj_now(*i).tapped));
    assert_eq!(pool_count(&t, P0, U), 1);
    let _ = Zone::Battlefield;
}
