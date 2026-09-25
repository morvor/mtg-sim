//! CR 105: Colors.

use super::r105_util::*;
use mtg_engine::ability::*;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// A sorcery whose only effect is "Choose a color."
fn color_chooser() -> CardDef {
    card_with(
        "Color Chooser",
        "{0}",
        "Sorcery",
        None,
        vec![spell_ab(
            vec![],
            Effect::Choose {
                who: PlayerRef::You,
                kind: ChoiceKind::Color,
            },
        )],
    )
}

#[test]
fn choosing_a_color_offers_exactly_the_five_colors() {
    cr!("105.1", "105.4");
    let mut t = TestGame::new(2);
    let spell = put_in_hand(&mut t, P0, color_chooser());
    let id = t.cast(P0, spell).go();
    // Choose "green" (the fifth option).
    t.answer(P0, DecisionKind::Option, Answer::Index(4));
    t.resolve();
    let opts = last_options(&t, P0);
    assert_eq!(opts, vec!["white", "blue", "black", "red", "green"]);
    // "Multicolored" and "colorless" aren't colors and aren't offered.
    assert!(!opts.iter().any(|o| o == "multicolored" || o == "colorless"));
    assert_eq!(t.g.obj(id).choices.color, Some(Color::Green));
}

#[test]
fn an_invalid_color_answer_still_yields_one_of_the_five_colors() {
    cr!("105.4");
    let mut t = TestGame::new(2);
    let spell = put_in_hand(&mut t, P0, color_chooser());
    let id = t.cast(P0, spell).go();
    // A sixth option ("multicolored") doesn't exist: the answer is rejected.
    t.answer(P0, DecisionKind::Option, Answer::Index(5));
    t.resolve();
    let chosen = t.g.obj(id).choices.color.expect("a color was chosen");
    assert!(Color::ALL.contains(&chosen));
}

#[test]
fn mana_of_any_color_is_one_of_the_five_colors() {
    cr!("105.1");
    let mut t = TestGame::new(2);
    // Birds of Paradise: "{T}: Add one mana of any color."
    let birds = t.battlefield(P0, "Birds of Paradise");
    t.answer(P0, DecisionKind::Option, Answer::Index(2));
    t.activate(P0, birds, 0, &[]).unwrap();
    let opts = last_options(&t, P0);
    assert_eq!(opts.len(), 5, "{opts:?}");
    assert!(!opts.iter().any(|o| o.contains('C')));
    assert_eq!(pool_count(&t, P0, ManaType::B), 1);
}

#[test]
fn object_is_the_colors_of_its_mana_symbols() {
    cr!("105.2");
    let mut t = TestGame::new(2);
    let bolt = t.hand(P0, "Lightning Bolt");
    let bears = t.battlefield(P0, "Grizzly Bears");
    // Hybrid {R/W}: the object is both of the symbol's colors (CR 107.4e).
    let recruit = t.battlefield(P0, "Boros Recruit");
    // Only colorless symbols in its cost ({0}): no color.
    let thopter = t.battlefield(P0, "Ornithopter");
    assert_eq!(colors(&t, bolt), cs("R"));
    assert_eq!(colors(&t, bears), cs("G"));
    assert_eq!(colors(&t, recruit), cs("RW"));
    assert_eq!(colors(&t, thopter), ColorSet::NONE);
    // The engine's mana-cost color derivation (used for faces without printed colors).
    let c = mtg_engine::mana::ManaCost::parse("{2}{U}{B}")
        .unwrap()
        .colors();
    assert_eq!(c, cs("UB"));
    let c = mtg_engine::mana::ManaCost::parse("{2/W}{G/P}")
        .unwrap()
        .colors();
    assert_eq!(c, cs("WG"));
}

#[test]
fn color_indicator_defines_color_of_card_without_mana_cost() {
    cr!("105.2", "107.13");
    let mut t = TestGame::new(2);
    // Dryad Arbor has no mana cost and a green color indicator.
    let arbor = t.battlefield(P0, "Dryad Arbor");
    assert!(t.obj_now(arbor).chars.mana_cost.is_none());
    assert_eq!(t.obj_now(arbor).chars.color_indicator, Some(cs("G")));
    assert_eq!(colors(&t, arbor), cs("G"));
    // Ancestral Vision: blue color indicator, no mana cost, in any zone.
    let vision = t.hand(P0, "Ancestral Vision");
    assert_eq!(colors(&t, vision), cs("U"));
    // "Destroy target monocolored creature" can target the green Dryad Arbor.
    t.lands(P0, "Swamp", 2);
    let price = t.hand(P0, "Ultimate Price");
    t.cast(P0, price).target(arbor).go();
    t.resolve();
    assert!(t.in_graveyard(P0, "Dryad Arbor"));
}

#[test]
fn characteristic_defining_ability_sets_color_in_every_zone() {
    cr!("105.2", "113.6a");
    let mut t = TestGame::new(2);
    // "~ is all colors." with a colorless mana cost: the CDA defines its color.
    let courier = card_from_text(
        "Courier",
        "{4}",
        "Artifact Creature — Golem",
        Some((3, 3)),
        "~ is all colors.",
    );
    let in_hand = put_in_hand(&mut t, P0, courier.clone());
    let in_gy = put_in_graveyard(&mut t, P0, courier.clone());
    let on_bf = put(&mut t, P0, courier);
    t.g.recompute();
    for id in [in_hand, in_gy, on_bf] {
        assert_eq!(colors(&t, id), ColorSet::ALL);
    }
    // "~ is colorless." overrides the red mana symbol in its cost.
    let ghost = card_from_text(
        "Ghost Bolt",
        "{2}{R}",
        "Instant",
        None,
        "~ is colorless.\n~ deals 3 damage to any target.",
    );
    let g = put_in_hand(&mut t, P0, ghost);
    t.g.recompute();
    assert_eq!(colors(&t, g), ColorSet::NONE);
    // Real cards: Transguild Courier and Ghostfire.
    let tc = t.battlefield(P0, "Transguild Courier");
    let gf = t.hand(P0, "Ghostfire");
    assert_eq!(colors(&t, tc), ColorSet::ALL);
    assert_eq!(colors(&t, gf), ColorSet::NONE);
}

#[test]
fn monocolored_multicolored_and_colorless_objects() {
    cr!("105.2a", "105.2b", "105.2c");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let recruit = t.battlefield(P1, "Boros Recruit");
    let courier = t.battlefield(P1, "Transguild Courier");
    let thopter = t.battlefield(P1, "Ornithopter");
    let mono = Filter::Monocolored;
    let multi = Filter::Multicolored;
    let colorless = Filter::Colorless;
    // 105.2a: exactly one color.
    assert!(matches(&t, bears, &mono, P0));
    assert!(!matches(&t, recruit, &mono, P0));
    assert!(!matches(&t, thopter, &mono, P0));
    // 105.2b: two or more colors.
    assert!(matches(&t, recruit, &multi, P0));
    assert!(matches(&t, courier, &multi, P0));
    assert!(!matches(&t, bears, &multi, P0));
    // 105.2c: no color.
    assert!(matches(&t, thopter, &colorless, P0));
    assert!(!matches(&t, bears, &colorless, P0));

    // Ultimate Price ("Destroy target monocolored creature") can target only the Bears.
    t.lands(P0, "Swamp", 2);
    let price = t.hand(P0, "Ultimate Price");
    assert_eq!(
        spell_target_candidates(&t, P0, price, 0),
        vec![Entity::Object(bears)]
    );
    t.cast(P0, price).target(bears).go();
    t.resolve();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    // Infernal Reckoning ("Exile target colorless creature") can target the Ornithopter.
    t.lands(P0, "Swamp", 1);
    let reckoning = t.hand(P0, "Infernal Reckoning");
    t.cast(P0, reckoning).target(thopter).go();
    t.resolve();
    assert!(t.in_exile("Ornithopter"));
}

#[test]
fn becoming_a_color_replaces_previous_colors() {
    cr!("105.3");
    let mut t = TestGame::new(2);
    let recruit = t.battlefield(P1, "Boros Recruit");
    t.lands(P0, "Island", 1);
    // Cerulean Wisps: "Target creature becomes blue until end of turn."
    let wisps = t.hand(P0, "Cerulean Wisps");
    t.cast(P0, wisps).target(recruit).go();
    t.resolve();
    assert_eq!(colors(&t, recruit), cs("U"));
    // The effect ends at end of turn.
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(colors(&t, recruit), cs("RW"));
}

#[test]
fn becoming_a_color_in_addition_keeps_other_colors() {
    cr!("105.3");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    // Indigo Faerie: "{U}: Target permanent becomes blue in addition to its other colors
    // until end of turn."
    let faerie = t.battlefield(P0, "Indigo Faerie");
    t.lands(P0, "Island", 1);
    t.activate(P0, faerie, 0, &[Entity::Object(bears)]).unwrap();
    t.resolve();
    assert_eq!(colors(&t, bears), cs("UG"));
}

#[test]
fn effects_can_make_a_colored_object_colorless_or_color_a_colorless_one() {
    cr!("105.3");
    let mut t = TestGame::new(2);
    // Raging Spirit: "{2}: This creature becomes colorless until end of turn."
    let spirit = t.battlefield(P0, "Raging Spirit");
    t.lands(P0, "Mountain", 2);
    t.activate(P0, spirit, 0, &[]).unwrap();
    t.resolve();
    assert_eq!(colors(&t, spirit), ColorSet::NONE);
    // Chaoslace: "Target spell or permanent becomes red." on a colorless artifact.
    let thopter = t.battlefield(P1, "Ornithopter");
    t.lands(P0, "Mountain", 1);
    let lace = t.hand(P0, "Chaoslace");
    t.cast(P0, lace).target(thopter).go();
    t.resolve();
    assert_eq!(colors(&t, thopter), cs("R"));
    // Moonlace: "Target spell or permanent becomes colorless."
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Island", 1);
    let moon = t.hand(P0, "Moonlace");
    t.cast(P0, moon).target(bears).go();
    t.resolve();
    assert_eq!(colors(&t, bears), ColorSet::NONE);
    // The effect has no duration: it lasts indefinitely.
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(colors(&t, bears), ColorSet::NONE);
    assert_eq!(colors(&t, thopter), cs("R"));
}

#[test]
fn there_are_exactly_ten_color_pairs() {
    cr!("105.5");
    let pairs = ColorSet::color_pairs();
    assert_eq!(pairs.len(), 10);
    for p in pairs {
        assert_eq!(p.count(), 2);
        assert!(p.is_color_pair());
    }
    let mut uniq = pairs.to_vec();
    uniq.sort_by_key(|c| c.0);
    uniq.dedup();
    assert_eq!(uniq.len(), 10);
    assert!(!ColorSet::single(Color::Red).is_color_pair());
    assert!(!cs("WUB").is_color_pair());
}

#[test]
fn niv_mizzet_counts_color_pairs_among_permanents_that_are_exactly_two_colors() {
    cr!("105.5");
    let mut t = TestGame::new(2);
    // "you gain X life, where X is the number of different color pairs among permanents
    // you control that are exactly two colors."
    let counter = card_with(
        "Pair Counter",
        "{0}",
        "Sorcery",
        None,
        vec![spell_ab(
            vec![],
            Effect::GainLife {
                who: PlayerRef::You,
                n: Value::ColorPairsAmong(Filter::Type(CardType::Creature).you_control()),
            },
        )],
    );
    t.battlefield(P0, "Boros Recruit"); // red-white
    t.battlefield(P0, "Boros Recruit"); // same pair again: counts once
    t.battlefield(P0, "Azorius Guildmage"); // white-blue
    t.battlefield(P0, "Sprouting Thrinax"); // three colors: not a color pair
    t.battlefield(P0, "Grizzly Bears"); // one color
    t.battlefield(P1, "Azorius Guildmage"); // not controlled by P0
    let s = put_in_hand(&mut t, P0, counter);
    t.cast(P0, s).go();
    t.resolve();
    assert_eq!(t.life(P0), 22);
    let _ = Zone::Battlefield;
}
