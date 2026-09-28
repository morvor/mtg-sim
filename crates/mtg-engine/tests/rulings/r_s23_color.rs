//! Rulings batch S23 — color (CR 105, 202.2, 204): colors from mana symbols (hybrid and
//! Phyrexian), color indicators, colorless objects, the types of mana (CR 106.1), devotion
//! (CR 700.5), and effects that change a permanent's colors (CR 105.3).

use crate::r_s01_common::*;
use crate::r_s04_common::spell_targets;
use crate::r_s06_common::activate_containing;
use crate::r_s05_common::enter;
use crate::r_s14_common::cast_from_hand;
use crate::r_s23_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::text_change::TextWords;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// The options of the "Choose mana type" decisions asked since `from`.
fn mana_type_options(t: &TestGame, from: usize) -> Vec<Vec<String>> {
    t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseOption {
                prompt, options, ..
            } if prompt.as_str() == "Choose mana type" => Some(options.clone()),
            _ => None,
        })
        .collect()
}

/// `p` taps `source` for mana with its first activated ability, choosing the `pick`th
/// offered type if asked; returns the mana added and empties the pool again.
fn tap_for(t: &mut TestGame, p: PlayerId, source: ObjectId, pick: usize) -> Vec<ManaType> {
    t.answer(p, DecisionKind::Option, Answer::Index(pick));
    t.activate(p, source, 0, &[]).unwrap();
    t.clear_answers();
    let pool = &t.g.player(p).mana_pool;
    let out: Vec<ManaType> = ManaType::ALL
        .iter()
        .flat_map(|ty| std::iter::repeat_n(*ty, pool.count(*ty)))
        .collect();
    t.g.players[p.idx()].mana_pool.empty();
    out
}

#[test]
fn hybrid_monocolored_hybrid_and_phyrexian_symbols_count_toward_devotion() {
    cr!("700.5", "107.4e", "107.4f", "202.2d");
    ruling!(
        "Gray Merchant of Asphodel",
        "Hybrid mana symbols, monocolored hybrid mana symbols, and Phyrexian mana symbols do count toward your devotion to their color(s)."
    );
    supported("Gray Merchant of Asphodel");
    let mut t = TestGame::new(2);
    // {B/R}{B/R}: two black symbols. {2/W}{2/B}{2/G}: one. {1}{B/P}: one.
    t.battlefield(P0, "Rakdos Shred-Freak");
    t.battlefield(P0, "Reputable Merchant");
    t.battlefield(P0, "Vault Skirge");
    // Gray Merchant's own {B}{B} count too: devotion to black is 2 + 2 + 1 + 1 = 6.
    enter(&mut t, P0, "Gray Merchant of Asphodel");
    t.resolve_all();
    assert_eq!(t.life(P1), 14);
    assert_eq!(t.life(P0), 26);
}

#[test]
fn phyrexian_mana_is_not_a_color_and_cant_be_produced() {
    cr!("107.4f", "105.1", "106.1b", "202.2d");
    ruling!(
        "Mental Misstep",
        "Phyrexian mana is not a new color. Players can't produce Phyrexian mana."
    );
    supported("Mental Misstep");
    let mut t = TestGame::new(2);
    // Mental Misstep ({U/P}) is blue, and only blue.
    let misstep = t.hand(P0, "Mental Misstep");
    assert_eq!(colors_now(&mut t, misstep), colors_of(&[Color::Blue]));
    // Its {U/P} is paid with blue mana (the only mana that can pay it) ...
    t.lands(P0, "Island", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.lands(P1, "Mountain", 1);
    t.cast(P1, bolt).target(Entity::Player(P0)).go();
    t.cast(P0, misstep).target(Entity::Object(bolt)).go();
    assert_eq!(t.life(P0), 20);
    assert_eq!(tapped_lands(&t, P0), 1);
    // ... or with 2 life: other mana (red here) can't pay for it.
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 1);
    let misstep = t.hand(P0, "Mental Misstep");
    let bolt = t.hand(P1, "Lightning Bolt");
    t.lands(P1, "Mountain", 1);
    t.cast(P1, bolt).target(Entity::Player(P0)).go();
    t.cast(P0, misstep).target(Entity::Object(bolt)).go();
    assert_eq!(t.life(P0), 18);
    assert_eq!(tapped_lands(&t, P0), 0);
    t.resolve_all();
    assert_eq!(t.life(P0), 18);
    // The types of mana a player can have are the five colors and colorless: no
    // Phyrexian mana.
    assert_eq!(
        ManaType::ALL.to_vec(),
        vec![
            ManaType::W,
            ManaType::U,
            ManaType::B,
            ManaType::R,
            ManaType::G,
            ManaType::C
        ]
    );
}

#[test]
fn reflecting_pool_can_produce_colorless_mana_a_type_of_mana() {
    cr!("106.1b", "106.7");
    ruling!(
        "Reflecting Pool",
        "The types of mana are white, blue, black, red, green, and colorless."
    );
    supported("Reflecting Pool");
    // With only Wastes ({C}), Reflecting Pool produces colorless mana.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Wastes");
    let pool = t.battlefield(P0, "Reflecting Pool");
    assert_eq!(tap_for(&mut t, P0, pool, 0), vec![ManaType::C]);
    // With Plains and Wastes, the types it could produce are white and colorless.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Plains");
    t.battlefield(P0, "Wastes");
    let pool = t.battlefield(P0, "Reflecting Pool");
    let from = t.asked().len();
    assert_eq!(tap_for(&mut t, P0, pool, 1), vec![ManaType::C]);
    assert_eq!(mana_type_options(&t, from), vec![vec!["W", "C"]]);
}

#[test]
fn dryad_arbors_color_indicator_isnt_changed_by_text_changes_but_its_color_can_be() {
    cr!("204.2", "612.1", "105.3", "613.1e");
    ruling!(
        "Dryad Arbor",
        "This color indicator can't be affected by text-changing effects (such as the one created by Crystal Spray), although color-changing effects can still overwrite it."
    );
    supported("Dryad Arbor");
    let mut t = TestGame::new(2);
    let arbor = t.battlefield(P0, "Dryad Arbor");
    assert_eq!(colors_now(&mut t, arbor), colors_of(&[Color::Green]));
    // A text-changing effect replaces "green" with "blue": the color indicator isn't
    // text, so Dryad Arbor stays green.
    change_text(
        &mut t,
        P0,
        arbor,
        TextWords::Color,
        color_word_idx(Color::Green, None),
        color_word_idx(Color::Blue, Some(Color::Green)),
    );
    assert_eq!(colors_now(&mut t, arbor), colors_of(&[Color::Green]));
    // Changing "Forest" to "Island" changes its land type, not its color.
    change_text(&mut t, P0, arbor, TextWords::BasicLandType, 4, 1);
    t.g.recompute();
    assert!(t.obj_now(arbor).chars.has_subtype("Island"));
    assert_eq!(colors_now(&mut t, arbor), colors_of(&[Color::Green]));
    // A color-changing effect overwrites it: Cerulean Wisps makes it blue.
    cast_from_hand(&mut t, P0, "Cerulean Wisps", &[Entity::Object(arbor)]);
    t.resolve_all();
    assert_eq!(colors_now(&mut t, arbor), colors_of(&[Color::Blue]));
}

#[test]
fn crookshank_kobolds_color_indicator_isnt_changed_by_text_changes_but_its_color_can_be() {
    cr!("204.2", "612.1", "105.3");
    ruling!(
        "Crookshank Kobolds",
        "This color indicator can’t be affected by text-changing effects (such as the one created by Crystal Spray), although color-changing effects can still overwrite it."
    );
    supported("Crookshank Kobolds");
    let mut t = TestGame::new(2);
    // Crookshank Kobolds costs {0} and has a red color indicator.
    let kobolds = t.battlefield(P0, "Crookshank Kobolds");
    assert_eq!(colors_now(&mut t, kobolds), colors_of(&[Color::Red]));
    change_text(
        &mut t,
        P0,
        kobolds,
        TextWords::Color,
        color_word_idx(Color::Red, None),
        color_word_idx(Color::White, Some(Color::Red)),
    );
    assert_eq!(colors_now(&mut t, kobolds), colors_of(&[Color::Red]));
    cast_from_hand(&mut t, P0, "Niveous Wisps", &[Entity::Object(kobolds)]);
    t.resolve_all();
    assert_eq!(colors_now(&mut t, kobolds), colors_of(&[Color::White]));
}

#[test]
fn a_keyrune_is_colorless_until_its_ability_resolves() {
    cr!("202.2b", "105.3", "613.1e");
    ruling!(
        "Dimir Keyrune",
        "Until the ability that turns the Keyrune into a creature resolves, the Keyrune is colorless."
    );
    supported("Dimir Keyrune");
    let mut t = TestGame::new(2);
    let keyrune = t.battlefield(P0, "Dimir Keyrune");
    assert!(colors_now(&mut t, keyrune).is_colorless());
    t.lands(P0, "Island", 1);
    t.lands(P0, "Swamp", 1);
    activate_containing(&mut t, P0, keyrune, "becomes a")
    .unwrap();
    // While the ability is on the stack, it's still a colorless noncreature artifact.
    assert_eq!(t.stack_len(), 1);
    assert!(colors_now(&mut t, keyrune).is_colorless());
    assert!(!t.obj_now(keyrune).is(CardType::Creature));
    t.resolve_all();
    assert_eq!(
        colors_now(&mut t, keyrune),
        colors_of(&[Color::Blue, Color::Black])
    );
    assert!(t.obj_now(keyrune).is(CardType::Creature));
}

#[test]
fn colorless_is_not_a_color_crimson_wisps_gives_a_colorless_creature_a_color() {
    cr!("105.2c", "105.2a", "105.3");
    ruling!("Crimson Wisps", "Colorless is not a color.");
    supported("Crimson Wisps");
    supported("Ultimate Price");
    let mut t = TestGame::new(2);
    // Memnite is colorless: it has no color, so it isn't monocolored (Ultimate Price
    // can't target it).
    let memnite = t.battlefield(P1, "Memnite");
    assert_eq!(colors_now(&mut t, memnite).count(), 0);
    assert!(!spell_targets(&mut t, P0, "Ultimate Price").contains(&Entity::Object(memnite)));
    // Crimson Wisps gives it a color: now it's red, one color.
    cast_from_hand(&mut t, P0, "Crimson Wisps", &[Entity::Object(memnite)]);
    t.resolve_all();
    assert_eq!(colors_now(&mut t, memnite), colors_of(&[Color::Red]));
    assert!(spell_targets(&mut t, P0, "Ultimate Price").contains(&Entity::Object(memnite)));
}

#[test]
fn a_multicolored_creature_spell_gets_both_monuments_discounts() {
    cr!("105.2b", "202.2c", "601.2f");
    ruling!(
        "Oketra's Monument",
        "A creature spell that's multiple colors is each of those colors. For example, Ahn-Crop Champion is a white creature and a green creature, so it can benefit from either Oketra's Monument or Rhonas's Monument—or both at once."
    );
    supported("Oketra's Monument");
    supported("Rhonas's Monument");
    supported("Ahn-Crop Champion");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Oketra's Monument");
    t.battlefield(P0, "Rhonas's Monument");
    // Ahn-Crop Champion ({2}{G}{W}) costs {1} less for each Monument: {G}{W}.
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Plains", 1);
    let champion = t.hand(P0, "Ahn-Crop Champion");
    t.cast(P0, champion).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Ahn-Crop Champion").len(), 1);
    // With only one Monument, it costs {1}{G}{W}: two lands aren't enough.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Oketra's Monument");
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Plains", 1);
    let champion = t.hand(P0, "Ahn-Crop Champion");
    assert!(t.cast(P0, champion).try_go().is_err());
}

#[test]
fn a_dragonlord_monument_is_colorless_until_its_ability_makes_it_two_colors() {
    cr!("202.2b", "105.3", "611.2a");
    ruling!(
        "Atarka Monument",
        "Each Monument is colorless, although the last ability will make each of them two colors until end of turn."
    );
    supported("Atarka Monument");
    let mut t = TestGame::new(2);
    let monument = t.battlefield(P0, "Atarka Monument");
    assert!(colors_now(&mut t, monument).is_colorless());
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Wastes", 4);
    activate_containing(&mut t, P0, monument, "becomes a")
    .unwrap();
    t.resolve_all();
    assert_eq!(
        colors_now(&mut t, monument),
        colors_of(&[Color::Red, Color::Green])
    );
    assert_eq!(t.pt(monument), (4, 4));
    // Until end of turn: it's colorless again on the next turn.
    t.advance_to(P1, Step::Upkeep);
    assert!(colors_now(&mut t, monument).is_colorless());
}

#[test]
fn a_creature_land_is_colorless_until_its_ability_gives_it_colors() {
    cr!("202.2b", "105.3");
    ruling!(
        "Lumbering Falls",
        "This land is colorless until the last ability gives it colors."
    );
    supported("Lumbering Falls");
    let mut t = TestGame::new(2);
    let falls = t.battlefield(P0, "Lumbering Falls");
    assert!(colors_now(&mut t, falls).is_colorless());
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 2);
    activate_containing(&mut t, P0, falls, "becomes a")
    .unwrap();
    assert!(colors_now(&mut t, falls).is_colorless());
    t.resolve_all();
    assert_eq!(
        colors_now(&mut t, falls),
        colors_of(&[Color::Green, Color::Blue])
    );
}

#[test]
fn turning_a_liege_blue_doesnt_change_the_colors_it_affects() {
    cr!("105.3", "612.1", "613.1e");
    ruling!(
        "Cerulean Wisps",
        "Changing a permanent’s color won’t change its text. If you turn Wilt-Leaf Liege blue, it will still affect green creatures and white creatures."
    );
    supported("Cerulean Wisps");
    supported("Boartusk Liege");
    let mut t = TestGame::new(2);
    // Boartusk Liege: "Other red creatures you control get +1/+1. Other green creatures
    // you control get +1/+1."
    let liege = t.battlefield(P0, "Boartusk Liege");
    let goblin = t.battlefield(P0, "Raging Goblin");
    let elves = t.battlefield(P0, "Llanowar Elves");
    let lions = t.battlefield(P0, "Savannah Lions");
    cast_from_hand(&mut t, P0, "Cerulean Wisps", &[Entity::Object(liege)]);
    t.resolve_all();
    assert_eq!(colors_now(&mut t, liege), colors_of(&[Color::Blue]));
    // It still affects red and green creatures (and not blue or white ones).
    assert_eq!(t.pt(goblin), (2, 2));
    assert_eq!(t.pt(elves), (2, 2));
    assert_eq!(t.pt(lions), (2, 1));
}

#[test]
fn turning_a_liege_red_doesnt_change_the_colors_it_affects() {
    cr!("105.3", "612.1", "613.1e");
    ruling!(
        "Crimson Wisps",
        "Changing a permanent's color won't change its text. If you turn Wilt-Leaf Liege blue, it will still affect green creatures and white creatures."
    );
    supported("Glen Elendra Liege");
    let mut t = TestGame::new(2);
    // Glen Elendra Liege: "Other blue creatures you control get +1/+1. Other black
    // creatures you control get +1/+1."
    let liege = t.battlefield(P0, "Glen Elendra Liege");
    let infiltrator = t.battlefield(P0, "Dimir Infiltrator");
    let goblin = t.battlefield(P0, "Raging Goblin");
    cast_from_hand(&mut t, P0, "Crimson Wisps", &[Entity::Object(liege)]);
    t.resolve_all();
    assert_eq!(colors_now(&mut t, liege), colors_of(&[Color::Red]));
    // Dimir Infiltrator (blue and black, 1/3) still gets +1/+1 for each of its colors;
    // the red Goblin gets nothing.
    assert_eq!(t.pt(infiltrator), (3, 5));
    assert_eq!(t.pt(goblin), (1, 1));
}

#[test]
fn cerulean_wisps_overwrites_all_old_colors() {
    cr!("105.3", "613.1e");
    ruling!(
        "Cerulean Wisps",
        "An effect that changes a permanent’s colors overwrites all its old colors unless it specifically says “in addition to its other colors.” For example, after Cerulean Wisps resolves, the affected creature will just be blue."
    );
    let mut t = TestGame::new(2);
    // Dimir Infiltrator is blue and black; after Cerulean Wisps it's just blue.
    let infiltrator = t.battlefield(P1, "Dimir Infiltrator");
    assert_eq!(
        colors_now(&mut t, infiltrator),
        colors_of(&[Color::Blue, Color::Black])
    );
    cast_from_hand(&mut t, P0, "Cerulean Wisps", &[Entity::Object(infiltrator)]);
    t.resolve_all();
    assert_eq!(colors_now(&mut t, infiltrator), colors_of(&[Color::Blue]));
}

#[test]
fn crimson_wisps_overwrites_all_old_colors() {
    cr!("105.3", "613.1e");
    ruling!(
        "Crimson Wisps",
        "An effect that changes a permanent's colors overwrites all its old colors unless it specifically says \"in addition to its other colors.\""
    );
    let mut t = TestGame::new(2);
    let infiltrator = t.battlefield(P1, "Dimir Infiltrator");
    cast_from_hand(&mut t, P0, "Crimson Wisps", &[Entity::Object(infiltrator)]);
    t.resolve_all();
    assert_eq!(colors_now(&mut t, infiltrator), colors_of(&[Color::Red]));
    // Until end of turn: then it's blue and black again.
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(
        colors_now(&mut t, infiltrator),
        colors_of(&[Color::Blue, Color::Black])
    );
}

#[test]
fn ultimate_price_monocolored_is_exactly_one_color_and_colorless_isnt() {
    cr!("105.2a", "105.2b", "105.2c", "115.1");
    ruling!(
        "Ultimate Price",
        "A monocolored creature is exactly one color. Colorless creatures aren't monocolored."
    );
    supported("Ultimate Price");
    let mut t = TestGame::new(2);
    let goblin = t.battlefield(P1, "Raging Goblin");
    let guildmage = t.battlefield(P1, "Dimir Guildmage");
    let memnite = t.battlefield(P1, "Memnite");
    let targets = spell_targets(&mut t, P0, "Ultimate Price");
    assert!(targets.contains(&Entity::Object(goblin)));
    assert!(!targets.contains(&Entity::Object(guildmage)));
    assert!(!targets.contains(&Entity::Object(memnite)));
    cast_from_hand(&mut t, P0, "Ultimate Price", &[Entity::Object(goblin)]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Raging Goblin"));
}

#[test]
fn solar_array_offers_the_five_colors_and_colorless_mana_adds_no_sunburst_counter() {
    cr!("105.1", "106.1a", "702.44a", "702.44b");
    ruling!(
        "Solar Array",
        "The five colors are white, blue, black, red, and green. Colorless is not a color."
    );
    supported("Solar Array");
    supported("Bottle Gnomes");
    let mut t = TestGame::new(2);
    let array = t.battlefield(P0, "Solar Array");
    t.lands(P0, "Wastes", 2);
    // "Add one mana of any color": the choices are the five colors, not colorless.
    let from = t.asked().len();
    t.answer(P0, DecisionKind::Option, Answer::Index(4));
    t.activate(P0, array, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(
        mana_type_options(&t, from),
        vec![vec!["W", "U", "B", "R", "G"]]
    );
    assert_eq!(t.g.player(P0).mana_pool.count(ManaType::G), 1);
    // Bottle Gnomes gains sunburst: {G} and {C}{C} were spent, one color, one counter.
    let gnomes = t.hand(P0, "Bottle Gnomes");
    t.cast(P0, gnomes).go();
    t.resolve_all();
    let gnomes = t.named_on_battlefield("Bottle Gnomes")[0];
    assert_eq!(t.counters(gnomes, "+1/+1"), 1);
}

#[test]
fn turn_to_frog_keeps_other_card_types_and_supertypes() {
    cr!("205.1a", "205.1b", "105.3", "613.1d", "613.1f", "613.4b");
    ruling!(
        "Turn to Frog",
        "The creature will lose all other colors and creature types, but it will retain any other card types (such as artifact) or supertypes (such as legendary) it may have."
    );
    supported("Turn to Frog");
    supported("Sharuum the Hegemon");
    // "Until end of turn, target creature loses all abilities and becomes a blue Frog with
    // base power and toughness 1/1." Sharuum the Hegemon is a white, blue, and black
    // legendary artifact creature — Sphinx, 5/5 with flying.
    let mut t = TestGame::new(2);
    let sharuum = t.battlefield(P1, "Sharuum the Hegemon");
    cast_from_hand(&mut t, P0, "Turn to Frog", &[Entity::Object(sharuum)]);
    t.resolve_all();
    t.g.recompute();
    let o = t.obj_now(sharuum);
    assert_eq!(o.chars.colors, colors_of(&[Color::Blue]));
    assert!(o.chars.has_subtype("Frog"));
    assert!(!o.chars.has_subtype("Sphinx"));
    assert!(o.is(CardType::Artifact) && o.is(CardType::Creature));
    assert!(o.chars.supertypes.contains(Supertype::Legendary));
    assert!(!o.chars.has_keyword(mtg_engine::keywords::KeywordKind::Flying));
    assert_eq!(t.pt(sharuum), (1, 1));
    // Until end of turn.
    t.advance_to(P1, Step::Upkeep);
    t.g.recompute();
    assert_eq!(t.pt(sharuum), (5, 5));
    assert!(t.obj_now(sharuum).chars.has_subtype("Sphinx"));
}

#[test]
fn okos_elk_loses_other_card_types_but_keeps_supertypes_indefinitely() {
    cr!("205.1a", "105.3", "613.1d", "613.1f", "613.4b", "611.2a");
    ruling!(
        "Oko, Thief of Crowns",
        "Oko's second ability overwrites all colors and creature types the affected creature has. It's just a green Elk. The creature keeps any supertypes (such as legendary) it has, but loses any other card types it has (such as artifact)."
    );
    ruling!(
        "Oko, Thief of Crowns",
        "The effects of Oko's second ability lasts indefinitely."
    );
    supported("Oko, Thief of Crowns");
    // "+1: Target artifact or creature loses all abilities and becomes a green Elk
    // creature with base power and toughness 3/3."
    let mut t = TestGame::new(2);
    let oko = t.battlefield(P0, "Oko, Thief of Crowns");
    let sharuum = t.battlefield(P1, "Sharuum the Hegemon");
    t.activate(P0, oko, 1, &[Entity::Object(sharuum)]).unwrap();
    t.resolve_all();
    t.advance_to(P1, Step::Upkeep);
    t.g.recompute();
    let o = t.obj_now(sharuum);
    assert_eq!(o.chars.colors, colors_of(&[Color::Green]));
    assert!(o.chars.has_subtype("Elk") && !o.chars.has_subtype("Sphinx"));
    assert!(o.is(CardType::Creature) && !o.is(CardType::Artifact));
    assert!(o.chars.supertypes.contains(Supertype::Legendary));
    assert!(!o.chars.has_keyword(mtg_engine::keywords::KeywordKind::Flying));
    assert_eq!(t.pt(sharuum), (3, 3));
}

/// The options of the "Choose a color or colors" decisions asked since `from`.
fn color_choices(t: &TestGame, from: usize) -> Vec<Vec<String>> {
    t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseOption {
                prompt, options, ..
            } if prompt.as_str() == "Choose a color or colors" => Some(options.clone()),
            _ => None,
        })
        .collect()
}

/// Index of a set of colors ("white and blue") among the offered choices: every nonempty
/// set of colors, in the order of their bits.
fn colors_idx(words: &str) -> usize {
    (1..32u8)
        .map(ColorSet)
        .position(|s| {
            s.iter()
                .map(|c| c.word())
                .collect::<Vec<_>>()
                .join(" and ")
                == words
        })
        .unwrap()
}

#[test]
fn quickchange_any_single_color_or_combination_but_not_colorless() {
    cr!("105.4", "105.3", "613.1e");
    ruling!(
        "Quickchange",
        "You can choose any single color or any combination of more than one color. You can’t choose colorless."
    );
    supported("Quickchange");
    supported("Prismwake Merrow");
    // Quickchange: "Target creature becomes the color or colors of your choice until end
    // of turn."
    let mut t = TestGame::new(2);
    let memnite = t.battlefield(P1, "Memnite");
    let from = t.asked().len();
    t.answer(
        P0,
        DecisionKind::Option,
        Answer::Index(colors_idx("white and blue")),
    );
    cast_from_hand(&mut t, P0, "Quickchange", &[Entity::Object(memnite)]);
    t.resolve_all();
    assert_eq!(
        colors_now(&mut t, memnite),
        colors_of(&[Color::White, Color::Blue])
    );
    // The choices: each single color and each combination (31), not colorless.
    let offered = color_choices(&t, from);
    assert_eq!(offered.len(), 1);
    assert_eq!(offered[0].len(), 31);
    assert!(offered[0].iter().all(|o| !o.contains("colorless")));
    assert!(offered[0].contains(&"red".to_string()));
    assert!(offered[0].contains(&"white and blue and black and red and green".to_string()));
    // Until end of turn: Memnite is colorless again.
    t.advance_to(P1, Step::Upkeep);
    assert!(colors_now(&mut t, memnite).is_colorless());
    // Prismwake Merrow's enters ability: a single color.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.answer(
        P0,
        DecisionKind::Option,
        Answer::Index(colors_idx("red")),
    );
    enter(&mut t, P0, "Prismwake Merrow");
    t.resolve_all();
    assert_eq!(colors_now(&mut t, bears), colors_of(&[Color::Red]));
}

#[test]
fn shyft_cant_become_colorless() {
    cr!("105.4", "105.3");
    ruling!(
        "Shyft",
        "Shyft’s ability won’t let you make it colorless. Colorless is not a color."
    );
    supported("Shyft");
    // "At the beginning of your upkeep, you may have this creature become the color or
    // colors of your choice. (This effect lasts indefinitely.)"
    let mut t = TestGame::new(2);
    let shyft = t.battlefield(P0, "Shyft");
    t.set_step(P1, Step::End);
    t.answer_yes(P0, true);
    t.answer(
        P0,
        DecisionKind::Option,
        Answer::Index(colors_idx("black and green")),
    );
    let from = t.asked().len();
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(
        colors_now(&mut t, shyft),
        colors_of(&[Color::Black, Color::Green])
    );
    let offered = color_choices(&t, from);
    assert_eq!(offered.len(), 1);
    assert!(offered[0].iter().all(|o| !o.is_empty() && !o.contains("colorless")));
    // It lasts indefinitely.
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(
        colors_now(&mut t, shyft),
        colors_of(&[Color::Black, Color::Green])
    );
}
