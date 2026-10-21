//! Choices among listed values (`oracle/patterns/choice_grammar_multi.rs`): "gains your
//! choice of A, B, or C".

use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn compiles(name: &str) {
    let c = mtg_engine::card::card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name}: {:?}",
        c.unsupported_text()
    );
}

#[test]
fn argivian_avenger_gains_the_chosen_ability_until_end_of_turn() {
    cr!("608.2d");
    compiles("Argivian Avenger");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let avenger = t.battlefield(P0, "Argivian Avenger");
    t.lands(P0, "Plains", 1);
    t.answer(P0, DecisionKind::Option, Answer::Index(2)); // deathtouch
    let (p, th) = t.pt(avenger);
    t.activate(P0, avenger, 0, &[]).unwrap();
    t.resolve_all();
    let o = t.obj_now(avenger);
    assert!(o.chars.has_keyword(KeywordKind::Deathtouch));
    assert!(!o.chars.has_keyword(KeywordKind::Flying));
    assert_eq!(t.pt(avenger), (p - 1, th - 1));
}

#[test]
fn jodahs_avenger_your_choice_includes_protection_from_red() {
    cr!("608.2d", "702.16a");
    compiles("Jodah's Avenger");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let avenger = t.battlefield(P0, "Jodah's Avenger");
    t.answer(P0, DecisionKind::Option, Answer::Index(1)); // protection from red
    t.activate(P0, avenger, 0, &[]).unwrap();
    t.resolve_all();
    let o = t.obj_now(avenger);
    assert!(o.chars.has_keyword(KeywordKind::Protection));
    assert!(!o.chars.has_keyword(KeywordKind::DoubleStrike));
}

#[test]
fn ballroom_brawlers_both_gain_the_same_chosen_ability() {
    cr!("608.2d");
    compiles("Ballroom Brawlers");
    let mut t = TestGame::new(2);
    let brawlers = t.battlefield(P0, "Ballroom Brawlers");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.advance_to(P0, Step::BeginningOfCombat);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.answer(P0, DecisionKind::Option, Answer::Index(1)); // lifelink
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(brawlers, Entity::Player(P1))]),
    );
    t.advance_to(P0, Step::DeclareAttackers);
    t.resolve_all();
    for id in [brawlers, bears] {
        assert!(t.obj_now(id).chars.has_keyword(KeywordKind::Lifelink));
        assert!(!t.obj_now(id).chars.has_keyword(KeywordKind::FirstStrike));
    }
}

/// The colors of the protection keywords `id` has.
fn protections(t: &TestGame, id: ObjectId) -> String {
    t.obj_now(id)
        .chars
        .keywords()
        .filter(|k| k.kind == KeywordKind::Protection)
        .map(|k| format!("{:?}", k.filter))
        .collect::<Vec<_>>()
        .join(";")
}

#[test]
fn sanctuary_blade_chooses_a_color_each_time_it_becomes_attached() {
    cr!("607.2d", "702.6a");
    ruling!(
        "Sanctuary Blade",
        "targeting the creature it's already attached to won't let you choose a new color"
    );
    compiles("Sanctuary Blade");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let blade = t.battlefield(P0, "Sanctuary Blade");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let lions = t.battlefield(P0, "Savannah Lions");
    t.lands(P0, "Plains", 9);
    t.answer(P0, DecisionKind::Option, Answer::Index(3)); // red
    t.activate(P0, blade, 0, &[Entity::Object(bears)]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 2));
    assert!(protections(&t, bears).contains("Red"));
    // Equipping the same creature again: no new choice.
    t.activate(P0, blade, 0, &[Entity::Object(bears)]).unwrap();
    t.resolve_all();
    let options = t
        .asked()
        .iter()
        .filter(|(_, d)| matches!(d, mtg_engine::decision::Decision::ChooseOption { .. }))
        .count();
    assert_eq!(options, 1);
    // Moving it: the last chosen color.
    t.answer(P0, DecisionKind::Option, Answer::Index(4)); // green
    t.activate(P0, blade, 0, &[Entity::Object(lions)]).unwrap();
    t.resolve_all();
    assert!(protections(&t, lions).contains("Green"));
    assert!(!protections(&t, lions).contains("Red"));
    assert_eq!(protections(&t, bears), "");
}

#[test]
fn chromatic_armor_prevents_damage_from_the_last_chosen_color() {
    cr!("615.1a", "607.2d");
    compiles("Chromatic Armor");
    let mut t = TestGame::new(2);
    t.lands(P0, "Tundra", 4);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let armor = t.hand(P0, "Chromatic Armor");
    t.advance_to(P0, Step::PrecombatMain);
    t.answer(P0, DecisionKind::Option, Answer::Index(3)); // red
    t.cast(P0, armor).target(bears).go();
    t.resolve_all();
    let armor = t.named_on_battlefield("Chromatic Armor")[0];
    t.lands(P1, "Mountain", 2);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(bears).go();
    t.resolve_all();
    assert_eq!(t.g.obj(bears).damage, 0, "red damage prevented");
    // {X}: choose blue instead.
    t.answer(P0, DecisionKind::Option, Answer::Index(1)); // blue
    t.activate(P0, armor, 0, &[]).unwrap();
    t.resolve_all();
    let shock = t.hand(P1, "Shock");
    t.cast(P1, shock).target(bears).go();
    t.resolve_all();
    assert_eq!(t.g.obj(bears).damage, 2, "red isn't the last chosen color");
}

fn option(t: &mut TestGame, p: PlayerId, i: usize) {
    t.answer(p, DecisionKind::Option, Answer::Index(i));
}

#[test]
fn illusionary_terrain_basic_lands_of_the_first_chosen_type_are_the_second() {
    cr!("305.7", "607.2d");
    compiles("Illusionary Terrain");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let island = t.battlefield(P1, "Island");
    let forest = t.battlefield(P1, "Forest");
    option(&mut t, P0, 1); // Island (of Plains, Island, Swamp, Mountain, Forest)
    option(&mut t, P0, 1); // Swamp (of Plains, Swamp, Mountain, Forest)
    t.enter(P0, "Illusionary Terrain");
    t.settle();
    let i = &t.obj_now(island).chars;
    assert!(i.has_subtype("Swamp") && !i.has_subtype("Island"));
    assert!(t.obj_now(forest).chars.has_subtype("Forest"));
    // The two types are different: the second choice didn't offer Island.
    let second = t.asked().into_iter().filter_map(|(_, d)| match d {
        mtg_engine::decision::Decision::ChooseOption { options, .. } => Some(options),
        _ => None,
    });
    assert!(second.last().is_some_and(|o| !o.contains(&"Island".to_string())));
}

#[test]
fn vision_charm_each_land_of_the_first_chosen_type_becomes_the_second() {
    cr!("305.7", "611.2c");
    ruling!("Vision Charm", "All lands changed are changed to the same type");
    compiles("Vision Charm");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 1);
    let forest = t.battlefield(P1, "Forest");
    let charm = t.hand(P0, "Vision Charm");
    t.advance_to(P0, Step::PrecombatMain);
    let types = mtg_engine::types::land_types();
    let fi = types.iter().position(|x| x == "Forest").unwrap();
    option(&mut t, P0, fi);
    option(&mut t, P0, 3); // Mountain
    let id = t.cast(P0, charm).modes(&[1]).go();
    t.resolve_all();
    assert_eq!(t.g.obj(id).choices.list, ["Forest", "Mountain"]);
    let f = &t.obj_now(forest).chars;
    assert!(f.has_subtype("Mountain") && !f.has_subtype("Forest"));
    t.advance_to(P1, Step::Upkeep);
    assert!(t.obj_now(forest).chars.has_subtype("Forest"), "until end of turn");
}

#[test]
fn greymond_humans_have_each_of_the_two_chosen_abilities() {
    cr!("607.2d", "613.1f");
    compiles("Greymond, Avacyn's Stalwart");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let human = t.battlefield(P0, "Thraben Inspector");
    let bears = t.battlefield(P0, "Grizzly Bears");
    option(&mut t, P0, 0); // first strike
    option(&mut t, P0, 1); // lifelink (of vigilance, lifelink)
    t.enter(P0, "Greymond, Avacyn's Stalwart");
    t.settle();
    let h = &t.obj_now(human).chars;
    assert!(h.has_keyword(KeywordKind::FirstStrike) && h.has_keyword(KeywordKind::Lifelink));
    assert!(!h.has_keyword(KeywordKind::Vigilance));
    assert!(!t.obj_now(bears).chars.has_keyword(KeywordKind::FirstStrike), "not a Human");
}

#[test]
fn void_destroys_and_discards_by_the_chosen_number() {
    cr!("608.2d", "202.3");
    ruling!("Void", "You can choose 0");
    compiles("Void");
    let mut t = TestGame::new(2);
    t.lands(P0, "Badlands", 5);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let lions = t.battlefield(P1, "Savannah Lions");
    t.hand(P1, "Grizzly Bears");
    t.hand(P1, "Forest");
    let void = t.hand(P0, "Void");
    t.advance_to(P0, Step::PrecombatMain);
    t.answer(P0, DecisionKind::Number, Answer::Number(2));
    t.cast(P0, void).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert!(t.on_battlefield(lions));
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(t.in_hand(P1, "Forest"), "lands aren't discarded");
    assert!(!t.in_hand(P1, "Grizzly Bears"));
    let min = t.asked().iter().find_map(|(_, d)| match d {
        mtg_engine::decision::Decision::ChooseNumber { min, .. } => Some(*min),
        _ => None,
    });
    assert_eq!(min, Some(0));
}

/// Mindblaze naming Lightning Bolt and `n` against P1 with two in their library.
fn mindblaze(n: i64) -> i32 {
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 6);
    t.library_top(P1, "Lightning Bolt");
    t.library_top(P1, "Lightning Bolt");
    let spell = t.hand(P0, "Mindblaze");
    t.advance_to(P0, Step::PrecombatMain);
    t.answer(P0, DecisionKind::Name, Answer::Text("Lightning Bolt".into()));
    t.answer(P0, DecisionKind::Number, Answer::Number(n));
    t.cast(P0, spell).target(Entity::Player(P1)).go();
    t.resolve_all();
    t.life(P1)
}

#[test]
fn mindblaze_deals_damage_if_the_library_has_exactly_the_chosen_number() {
    cr!("701.20a", "608.2d");
    compiles("Mindblaze");
    assert_eq!(mindblaze(2), 12);
    assert_eq!(mindblaze(3), 20);
}

/// Scrying Glass choosing `n` and red against P1 holding one red card and a land.
fn scrying_glass(n: i64) -> usize {
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let glass = t.battlefield(P0, "Scrying Glass");
    t.lands(P0, "Plains", 3);
    t.hand(P1, "Lightning Bolt");
    t.hand(P1, "Forest");
    t.answer(P0, DecisionKind::Number, Answer::Number(n));
    t.answer(P0, DecisionKind::Option, Answer::Index(3)); // red
    let hand = t.hand_size(P0);
    t.activate(P0, glass, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    t.hand_size(P0) - hand
}

#[test]
fn scrying_glass_draws_if_exactly_the_chosen_number_of_the_chosen_color() {
    cr!("608.2d");
    ruling!("Scrying Glass", "You choose a number and color during resolution");
    compiles("Scrying Glass");
    assert_eq!(scrying_glass(1), 1);
    assert_eq!(scrying_glass(2), 0);
}

/// Tablet of the Guilds with blue and red chosen; P0 casts `spell`: P0's life gained.
fn tablet(spell: &str) -> i32 {
    use mtg_engine::types::{Color, ColorSet};
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let pairs: Vec<ColorSet> = (1..32u8)
        .map(ColorSet)
        .filter(|s| s.iter().count() == 2)
        .collect();
    let i = pairs
        .iter()
        .position(|s| s.contains(Color::Blue) && s.contains(Color::Red))
        .unwrap();
    t.answer(P0, DecisionKind::Option, Answer::Index(i));
    t.enter(P0, "Tablet of the Guilds");
    t.lands(P0, "Volcanic Island", 2);
    t.lands(P0, "Tropical Island", 2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let card = t.hand(P0, spell);
    let b = t.cast(P0, card);
    let b = match spell {
        "Lightning Bolt" => b.target(Entity::Player(P1)),
        "Giant Growth" => b.target(bears),
        // Izzet Charm: draw two cards, then discard two cards.
        _ => b.modes(&[2]),
    };
    b.go();
    t.resolve_all();
    t.life(P0) - 20
}

#[test]
fn tablet_of_the_guilds_life_for_each_chosen_color_the_spell_is() {
    cr!("603.4", "105.2");
    ruling!("Tablet of the Guilds", "You must choose two different colors");
    compiles("Tablet of the Guilds");
    assert_eq!(tablet("Lightning Bolt"), 1, "red");
    assert_eq!(tablet("Izzet Charm"), 2, "blue and red");
    assert_eq!(tablet("Giant Growth"), 0, "neither");
}

#[test]
fn caller_of_the_hunt_chooses_a_type_as_it_is_cast_and_counts_it() {
    cr!("601.2b", "607.2d");
    compiles("Caller of the Hunt");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 3);
    t.battlefield(P0, "Raging Goblin");
    t.battlefield(P1, "Raging Goblin");
    t.battlefield(P1, "Grizzly Bears");
    let caller = t.hand(P0, "Caller of the Hunt");
    t.advance_to(P0, Step::PrecombatMain);
    let goblin = mtg_engine::types::subtype_lists()
        .creature
        .iter()
        .position(|x| x == "Goblin")
        .unwrap();
    option(&mut t, P0, goblin);
    let spell = t.cast(P0, caller).go();
    assert_eq!(
        t.g.obj(spell).choices.creature_type.as_deref(),
        Some("Goblin"),
        "chosen as it was cast"
    );
    t.resolve_all();
    let caller = t.named_on_battlefield("Caller of the Hunt")[0];
    assert_eq!(t.pt(caller), (2, 2), "two Goblins");
}

#[test]
fn liquid_fire_splits_five_damage_by_the_number_chosen_as_it_was_cast() {
    cr!("601.2b", "118.8");
    ruling!("Liquid Fire", "You choose the number as you cast the spell");
    compiles("Liquid Fire");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 6);
    let giant = t.battlefield(P1, "Indomitable Ancients");
    let fire = t.hand(P0, "Liquid Fire");
    t.advance_to(P0, Step::PrecombatMain);
    t.answer(P0, DecisionKind::Number, Answer::Number(2));
    let spell = t.cast(P0, fire).target(giant).go();
    assert_eq!(t.g.obj(spell).choices.number, Some(2));
    t.resolve_all();
    assert_eq!(t.g.obj(giant).damage, 2);
    assert_eq!(t.life(P1), 17);
}
