//! Rulings batch P112 — activated and triggered abilities whose effects are fixed as they
//! resolve: counts and choices made on resolution (CR 608.2h, 611.2c), last known
//! information of a source that left (CR 608.2h), illegal targets (CR 608.2b), effects
//! without a duration (CR 611.2a), type-changing and P/T-setting effects (CR 205.1a,
//! 613.4b), control exchanges (CR 701.12) and redundant lifelink (CR 702.15f).

use crate::r_p023_common::unsick;
use crate::r_p160_common::{helpers_supported, layer7};
use crate::r_s01_common::*;
use crate::r_s02_common::{can_play_land, destroy};
use crate::r_s04_common::ability_targets;
use crate::r_s06_common::{attach_new, attached_to, has_kw};
use crate::r_s07_common::damage_on;
use crate::r_s17_common::transform;
use crate::r_s21_common::legal_blocks;
use crate::r_s24_common::controller;
use crate::r_s25_common::cast_new;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn has_subtype(t: &TestGame, id: ObjectId, s: &str) -> bool {
    t.obj_now(id).chars.has_subtype(s)
}

/// The options of the `ChooseOption` decisions with this prompt asked since `from`.
fn options_asked(t: &TestGame, from: usize, prompt: &str) -> Vec<Vec<String>> {
    t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseOption {
                prompt: p, options, ..
            } if p.as_str() == prompt => Some(options.clone()),
            _ => None,
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Counts made as the ability resolves
// ---------------------------------------------------------------------------

#[test]
fn ishkanah_counts_spiders_as_it_resolves_including_itself() {
    cr!("608.2h");
    ruling!(
        "Ishkanah, Grafwidow",
        "The number of Spiders you control is checked only as Ishkanah's last ability resolves. The ability will count Ishkanah itself if it's still on the battlefield."
    );
    supported("Ishkanah, Grafwidow");
    let mut t = TestGame::new(2);
    let ishkanah = t.battlefield(P0, "Ishkanah, Grafwidow");
    let spider = t.battlefield(P0, "Graverobber Spider");
    t.lands(P0, "Swamp", 7);
    t.activate(P0, ishkanah, 0, &[Entity::Player(P1)]).unwrap();
    // Another Spider dies in response: only Ishkanah is counted.
    destroy(&mut t, spider);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
}

#[test]
fn graverobber_spider_counts_creature_cards_once_as_it_resolves() {
    cr!("608.2h", "611.2c");
    ruling!(
        "Graverobber Spider",
        "The number of creature cards in your graveyard is counted only as the ability resolves. Once the ability resolves, the bonus won't change, even if the number of creature cards in your graveyard changes later in the turn."
    );
    supported("Graverobber Spider");
    let mut t = TestGame::new(2);
    let spider = t.battlefield(P0, "Graverobber Spider");
    t.graveyard(P0, "Grizzly Bears");
    t.lands(P0, "Swamp", 4);
    t.activate(P0, spider, 0, &[]).unwrap();
    // A creature card hits the graveyard in response: it's counted.
    t.graveyard(P0, "Craw Wurm");
    t.resolve_all();
    assert_eq!(t.pt(spider), (4, 6));
    // Later changes don't matter.
    t.graveyard(P0, "Hill Giant");
    t.g.recompute();
    assert_eq!(t.pt(spider), (4, 6));
}

#[test]
fn kenrith_first_ability_affects_only_creatures_there_as_it_resolves() {
    cr!("611.2c");
    ruling!(
        "Kenrith, the Returned King",
        "Kenrith's first ability affects only creatures on the battlefield at the time it resolves. Creatures that enter the battlefield later in the turn won't gain trample or haste."
    );
    supported("Kenrith, the Returned King");
    let mut t = TestGame::new(2);
    let kenrith = t.battlefield(P0, "Kenrith, the Returned King");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 1);
    t.activate(P0, kenrith, 0, &[]).unwrap();
    t.resolve_all();
    for id in [kenrith, theirs] {
        assert!(has_kw(&t, id, KeywordKind::Trample));
        assert!(has_kw(&t, id, KeywordKind::Haste));
    }
    let later = t.battlefield_sick(P0, "Hill Giant");
    assert!(!has_kw(&t, later, KeywordKind::Trample));
    assert!(!has_kw(&t, later, KeywordKind::Haste));
}

#[test]
fn kenrith_reanimates_any_graveyards_creature_under_its_owners_control() {
    cr!("115.1", "800.4a");
    ruling!(
        "Kenrith, the Returned King",
        "Kenrith's last ability can target a creature card in any player's graveyard. Its owner will control the creature, and it will remain on the battlefield even if you leave the game (if you don't own it)."
    );
    let mut t = TestGame::new(3);
    let kenrith = t.battlefield(P0, "Kenrith, the Returned King");
    let wurm = t.graveyard(P1, "Craw Wurm");
    t.lands(P0, "Swamp", 5);
    t.activate(P0, kenrith, 4, &[wurm.into()]).unwrap();
    t.resolve_all();
    assert!(t.on_battlefield(wurm));
    assert_eq!(controller(&mut t, wurm), P1);
    // Kenrith's controller leaves the game; the creature stays.
    t.g.lose_game(P0);
    t.settle();
    assert!(t.on_battlefield(wurm));
    assert_eq!(controller(&mut t, wurm), P1);
}

#[test]
fn huatli_ultimate_affects_only_dinosaurs_there_as_it_resolves() {
    cr!("611.2c");
    ruling!(
        "Huatli, Dinosaur Knight",
        "Huatli’s last ability affects only Dinosaurs you control at the time it resolves. Dinosaurs you begin to control later in the turn or creatures that become Dinosaurs later in the turn won’t get +4/+4."
    );
    let mut t = TestGame::new(2);
    let huatli = t.battlefield(P0, "Huatli, Dinosaur Knight");
    t.g.objects[huatli.0 as usize]
        .counters
        .insert(counters::LOYALTY.into(), 7);
    let dino = t.battlefield(P0, "Colossal Dreadmaw");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.activate(P0, huatli, 2, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(dino), (10, 10));
    // A new Dinosaur, and a creature that becomes a Dinosaur (an artifact creature 4/3).
    let later = t.battlefield(P0, "Colossal Dreadmaw");
    assert_eq!(t.pt(later), (6, 6));
    cast_new(&mut t, P0, "Relic's Roar", &[bears.into()]);
    t.resolve_all();
    assert!(has_subtype(&t, bears, "Dinosaur"));
    assert_eq!(t.pt(bears), (4, 3));
}

// ---------------------------------------------------------------------------
// Sources that left, and targets that became illegal
// ---------------------------------------------------------------------------

#[test]
fn vein_drinker_needs_its_target_but_not_itself() {
    cr!("608.2b", "608.2h");
    ruling!(
        "Vein Drinker",
        "When the first ability resolves, if the targeted creature is no longer on the battlefield, the ability doesn't resolve for having no legal targets. No damage is dealt."
    );
    ruling!(
        "Vein Drinker",
        "When the first ability resolves, if the targeted creature is still on the battlefield but Vein Drinker is not, Vein Drinker will still deal damage equal to its power as it last existed on the battlefield to the targeted creature."
    );
    supported("Vein Drinker");
    // The target is gone: Vein Drinker isn't dealt damage.
    let mut t = TestGame::new(2);
    let drinker = t.battlefield(P0, "Vein Drinker");
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.lands(P0, "Mountain", 1);
    t.activate(P0, drinker, 0, &[wurm.into()]).unwrap();
    destroy(&mut t, wurm);
    t.resolve_all();
    assert_eq!(damage_on(&t, drinker), 0);
    // Vein Drinker is gone (after a pump to 7/7): it still deals 7.
    let mut t = TestGame::new(2);
    let drinker = t.battlefield(P0, "Vein Drinker");
    let maw = t.battlefield(P1, "Colossal Dreadmaw");
    cast_new(&mut t, P0, "Giant Growth", &[drinker.into()]);
    t.resolve_all();
    t.lands(P0, "Mountain", 1);
    t.activate(P0, drinker, 0, &[maw.into()]).unwrap();
    destroy(&mut t, drinker);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Colossal Dreadmaw"));
}

/// P0 attacks P1 with Cyclops Gladiator; its trigger targets P1's Hill Giant (3/3).
fn gladiator_attacks() -> (TestGame, ObjectId, ObjectId, ObjectId) {
    supported("Cyclops Gladiator");
    let mut t = TestGame::new(2);
    let gladiator = t.battlefield(P0, "Cyclops Gladiator");
    let giant = t.battlefield(P1, "Hill Giant");
    let blocker = t.battlefield(P1, "Ornithopter");
    t.answer_targets(P0, &[giant.into()]);
    t.answer_yes(P0, true);
    attack_with(&mut t, &[(gladiator, Entity::Player(P1))]);
    assert_eq!(t.stack_len(), 1);
    (t, gladiator, giant, blocker)
}

#[test]
fn cyclops_gladiator_fights_before_blockers_and_keeps_attacking() {
    cr!("508.1m", "509.1", "608.2c");
    ruling!(
        "Cyclops Gladiator",
        "Cyclops Gladiator’s ability triggers and resolves during the declare attackers step, before blockers are declared. If it survives, it continues to attack (presumably with some damage marked on it) and may be blocked."
    );
    let (mut t, gladiator, _, blocker) = gladiator_attacks();
    t.resolve();
    assert_eq!(t.g.turn.step, Step::DeclareAttackers);
    assert!(t.in_graveyard(P1, "Hill Giant"));
    assert_eq!(damage_on(&t, gladiator), 3);
    assert!(t
        .g
        .combat
        .as_ref()
        .unwrap()
        .attackers
        .iter()
        .any(|a| a.id == t.g.current(gladiator)));
    assert!(legal_blocks(&mut t, P1, &[(blocker, gladiator)]));
    block_and_finish(&mut t, P1, &[(blocker, gladiator)]);
    assert!(t.in_graveyard(P1, "Ornithopter"));
    assert!(t.on_battlefield(gladiator));
}

#[test]
fn cyclops_gladiator_target_gone_no_damage_but_its_own_absence_doesnt_matter() {
    cr!("608.2b", "608.2h");
    ruling!(
        "Cyclops Gladiator",
        "If the targeted creature leaves the battlefield (or otherwise becomes an illegal target) before the ability resolves, the ability doesn’t resolve. Cyclops Gladiator isn’t dealt damage."
    );
    ruling!(
        "Cyclops Gladiator",
        "On the other hand, if Cyclops Gladiator leaves the battlefield before the ability resolves, the ability continues to resolve. Cyclops Gladiator deals damage to the targeted creature equal to the power it had as it last existed on the battlefield."
    );
    let (mut t, gladiator, giant, _) = gladiator_attacks();
    destroy(&mut t, giant);
    t.resolve_all();
    assert_eq!(damage_on(&t, gladiator), 0);
    let (mut t, gladiator, _, _) = gladiator_attacks();
    destroy(&mut t, gladiator);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Hill Giant"));
}

#[test]
fn malady_invoker_uses_its_last_known_power() {
    cr!("608.2h");
    ruling!(
        "Herbology Instructor // Malady Invoker",
        "Use Malady Invoker’s power as its ability resolves to determine the value of X. If Malady Invoker is no longer on the battlefield at that time, use its power when it was last on the battlefield."
    );
    supported("Herbology Instructor // Malady Invoker");
    for gone in [false, true] {
        let mut t = TestGame::new(2);
        let invoker = t.battlefield(P0, "Herbology Instructor // Malady Invoker");
        let maw = t.battlefield(P1, "Colossal Dreadmaw");
        t.answer_targets(P0, &[maw.into()]);
        transform(&mut t, invoker);
        assert_eq!(t.pt(invoker), (3, 3));
        assert_eq!(t.stack_len(), 1);
        // Pumped to 6 power in response.
        cast_new(&mut t, P0, "Giant Growth", &[invoker.into()]);
        t.resolve();
        assert_eq!(t.pt(invoker), (6, 6));
        if gone {
            destroy(&mut t, invoker);
        }
        t.resolve_all();
        assert!(t.in_graveyard(P1, "Colossal Dreadmaw"), "gone: {gone}");
    }
}

// ---------------------------------------------------------------------------
// Choices made on resolution
// ---------------------------------------------------------------------------

#[test]
fn caldera_kavu_chooses_its_color_on_resolution() {
    cr!("608.2d", "105.4");
    ruling!("Caldera Kavu", "You choose a color on resolution.");
    supported("Caldera Kavu");
    let mut t = TestGame::new(2);
    let kavu = t.battlefield(P0, "Caldera Kavu");
    t.lands(P0, "Forest", 1);
    let from = t.asked().len();
    t.activate(P0, kavu, 1, &[]).unwrap();
    assert!(options_asked(&t, from, "Choose a color").is_empty());
    let colors = [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green];
    let blue = colors.iter().position(|c| *c == Color::Blue).unwrap();
    t.answer(P0, DecisionKind::Option, Answer::Index(blue));
    t.resolve_all();
    assert_eq!(options_asked(&t, from, "Choose a color").len(), 1);
    assert_eq!(t.obj_now(kavu).chars.colors, ColorSet::single(Color::Blue));
}

#[test]
fn swirling_spriggan_any_colors_but_not_colorless() {
    cr!("105.4", "105.2c");
    ruling!(
        "Swirling Spriggan",
        "You can choose any color or combination of colors. You can’t choose colorless."
    );
    supported("Swirling Spriggan");
    let mut t = TestGame::new(2);
    let spriggan = t.battlefield(P0, "Swirling Spriggan");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 2);
    let from = t.asked().len();
    t.activate(P0, spriggan, 0, &[bears.into()]).unwrap();
    // Every nonempty set of colors, in the order of their bits: white and black = 0b101.
    t.answer(P0, DecisionKind::Option, Answer::Index(0b101 - 1));
    t.resolve_all();
    let mut wb = ColorSet::single(Color::White);
    wb.insert(Color::Black);
    assert_eq!(t.obj_now(bears).chars.colors, wb);
    let offered = options_asked(&t, from, "Choose a color or colors");
    assert_eq!(offered.len(), 1);
    assert_eq!(offered[0].len(), 31);
    assert!(offered[0].iter().all(|o| !o.contains("colorless")));
}

#[test]
fn pixie_illusionist_land_becomes_the_basic_type_chosen_on_resolution() {
    cr!("305.7", "608.2d");
    ruling!(
        "Pixie Illusionist",
        "The affected land loses its existing land types and any abilities printed on it. It becomes the chosen basic land type and now has the ability to tap to add one mana of the appropriate color. Pixie Illusionist's ability doesn't change the affected land's name or whether it's legendary or basic."
    );
    ruling!(
        "Pixie Illusionist",
        "You choose a basic land type as the last ability resolves."
    );
    supported("Pixie Illusionist");
    supported("Shizo, Death's Storehouse");
    let mut t = TestGame::new(2);
    let pixie = t.battlefield(P0, "Pixie Illusionist");
    let shizo = t.battlefield(P0, "Shizo, Death's Storehouse");
    let from = t.asked().len();
    t.activate(P0, pixie, 0, &[shizo.into()]).unwrap();
    // Nothing chosen yet.
    assert!(t.asked()[from..]
        .iter()
        .all(|(_, d)| !matches!(d, Decision::ChooseOption { .. })));
    let island = |d: &Decision| match d {
        Decision::ChooseOption { options, .. } => options.iter().position(|o| o == "Island"),
        _ => None,
    };
    let seen = watch(
        &mut t,
        P0,
        |d| matches!(d, Decision::ChooseOption { .. }),
        |g| g.stack.len(),
    );
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.resolve_all();
    // The type was chosen while the ability was resolving.
    assert_eq!(seen.lock().unwrap().as_slice(), &[1]);
    let asked: Vec<_> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| island(d))
        .collect();
    assert_eq!(asked, vec![1], "Island is the second basic land type offered");
    let o = t.obj_now(shizo);
    assert!(o.chars.has_subtype("Island"));
    assert_eq!(o.chars.name, "Shizo, Death's Storehouse");
    assert!(o.chars.supertypes.contains(Supertype::Legendary));
    assert!(!o.chars.abilities.iter().any(|a| a.text.contains("fear")));
    assert!(crate::r_s20_common::tap_for_mana(&mut t, P0, shizo, "Add"));
    assert_eq!(t.g.player(P0).mana_pool.count(ManaType::U), 1);
    assert_eq!(t.g.player(P0).mana_pool.count(ManaType::B), 0);
}

#[test]
fn pixie_illusionist_basic_land_stays_basic() {
    cr!("305.7");
    ruling!(
        "Pixie Illusionist",
        "Pixie Illusionist's ability doesn't change the affected land's name or whether it's legendary or basic."
    );
    let mut t = TestGame::new(2);
    let pixie = t.battlefield(P0, "Pixie Illusionist");
    let forest = t.battlefield(P0, "Forest");
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.activate(P0, pixie, 0, &[forest.into()]).unwrap();
    t.resolve_all();
    let o = t.obj_now(forest);
    assert!(o.chars.has_subtype("Island") && !o.chars.has_subtype("Forest"));
    assert_eq!(o.chars.name, "Forest");
    assert!(o.chars.supertypes.contains(Supertype::Basic));
}

#[test]
fn pixie_illusionist_kicked_enters_with_counters_without_a_trigger() {
    cr!("614.1c", "702.33d");
    ruling!(
        "Pixie Illusionist",
        "Pixie Illusionist's third ability isn't a triggered ability. If it was kicked, there is no point where an opponent can respond while it is on the battlefield but doesn't yet have counters on it."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 1);
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Wastes", 3);
    let card = t.hand(P0, "Pixie Illusionist");
    t.cast(P0, card).kicked(true).go();
    t.g.resolve_top();
    t.g.flush_events();
    let pixie = t.named_on_battlefield("Pixie Illusionist")[0];
    assert_eq!(t.counters(pixie, counters::PLUS1), 2);
    t.settle();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.pt(pixie), (3, 3));
}

// ---------------------------------------------------------------------------
// X
// ---------------------------------------------------------------------------

#[test]
fn chamber_sentry_counters_come_from_colors_not_x() {
    cr!("107.3", "601.2f");
    ruling!(
        "Chamber Sentry",
        "You can choose any value for X as you cast Chamber Sentry. The value chosen for X doesn't directly affect the number of +1/+1 counters Chamber Sentry enters the battlefield with, but it does let you pay more mana and thus spend more colors of mana to cast it."
    );
    supported("Chamber Sentry");
    for (lands, n) in [(["Forest", "Forest", "Forest"], 1), (["Plains", "Island", "Swamp"], 3)] {
        let mut t = TestGame::new(2);
        for l in lands {
            t.lands(P0, l, 1);
        }
        let card = t.hand(P0, "Chamber Sentry");
        t.cast(P0, card).x(3).go();
        t.resolve_all();
        let sentry = t.named_on_battlefield("Chamber Sentry")[0];
        assert_eq!(t.counters(sentry, counters::PLUS1), n, "{lands:?}");
    }
}

#[test]
fn chamber_sentry_activation_x_is_independent_of_cast_x() {
    cr!("107.3", "107.3a");
    ruling!(
        "Chamber Sentry",
        "The value of X chosen when you activate Chamber Sentry's first activated ability doesn't have to be the same value of X that you chose when you cast it."
    );
    let mut t = TestGame::new(2);
    for l in ["Plains", "Island", "Swamp"] {
        t.lands(P0, l, 1);
    }
    let card = t.hand(P0, "Chamber Sentry");
    t.cast(P0, card).x(3).go();
    t.resolve_all();
    let sentry = t.named_on_battlefield("Chamber Sentry")[0];
    unsick(&mut t, sentry);
    t.lands(P0, "Wastes", 1);
    t.answer(P0, DecisionKind::X, Answer::Number(1));
    t.activate(P0, sentry, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    assert_eq!(t.counters(sentry, counters::PLUS1), 2);
}

// ---------------------------------------------------------------------------
// Effects without a duration, type and P/T setting
// ---------------------------------------------------------------------------

#[test]
fn warden_of_the_first_tree_abilities_have_no_duration() {
    cr!("611.2a");
    ruling!(
        "Warden of the First Tree",
        "Neither the first nor the second ability has a duration. If one of them resolves, it will remain in effect until Warden of the First Tree leaves the battlefield or some subsequent effect changes its characteristics."
    );
    supported("Warden of the First Tree");
    let mut t = TestGame::new(2);
    let warden = t.battlefield(P0, "Warden of the First Tree");
    t.lands(P0, "Plains", 6);
    t.activate(P0, warden, 0, &[]).unwrap();
    t.resolve_all();
    t.activate(P0, warden, 1, &[]).unwrap();
    t.resolve_all();
    t.advance_to(P1, Step::Upkeep);
    t.g.recompute();
    assert_eq!(t.pt(warden), (3, 3));
    for s in ["Human", "Spirit", "Warrior"] {
        assert!(has_subtype(&t, warden, s), "{s}");
    }
    assert!(has_kw(&t, warden, KeywordKind::Trample));
    assert!(has_kw(&t, warden, KeywordKind::Lifelink));
}

#[test]
fn warden_of_the_first_tree_sets_base_pt_in_timestamp_order_and_overwrites_types() {
    cr!("613.4b", "613.7", "205.1a");
    ruling!(
        "Warden of the First Tree",
        "The first ability overwrites any previous effects that set the creature's base power and toughness to specific values. Any power- or toughness-setting effects that start to apply after the first ability resolves will overwrite this effect."
    );
    ruling!(
        "Warden of the First Tree",
        "The first and second activated abilities cause Warden of the First Tree to lose any other creature types it has. It retains any card types or supertypes it may have."
    );
    helpers_supported();
    let mut t = TestGame::new(2);
    let warden = t.battlefield(P0, "Warden of the First Tree");
    // Relic's Roar (earlier): a 4/3 Dinosaur artifact creature. Then Warden's first
    // ability (3/3 Human Warrior), then Square Up (later, 4/4).
    layer7(
        &mut t,
        warden,
        "Relic's Roar",
        false,
        |t| {
            assert!(has_subtype(t, warden, "Dinosaur"));
            t.lands(P0, "Plains", 1);
            t.lands(P0, "Wastes", 1);
            t.activate(P0, warden, 0, &[]).unwrap();
            t.resolve_all();
            let o = t.obj_now(warden);
            assert!(o.chars.has_subtype("Warrior") && o.chars.has_subtype("Human"));
            assert!(!o.chars.has_subtype("Dinosaur"));
            assert!(o.is(CardType::Artifact) && o.is(CardType::Creature));
        },
        (3, 3),
        "Square Up",
    );
}

#[test]
fn memnarch_effects_last_indefinitely_and_combine() {
    cr!("611.2a", "613.1d", "613.1b");
    ruling!(
        "Memnarch",
        "The effects of Memnarch’s abilities don’t end at end of turn, and they don’t end when Memnarch leaves the battlefield. Both effects last until the affected permanent leaves the battlefield."
    );
    ruling!(
        "Memnarch",
        "You can use the first ability on a nonartifact permanent, wait for the ability to resolve, and then use the second ability on that permanent."
    );
    supported("Memnarch");
    let mut t = TestGame::new(2);
    let memnarch = t.battlefield(P0, "Memnarch");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Island", 7);
    // The second ability can't target the Bears yet (not an artifact).
    assert!(!ability_targets(&mut t, memnarch, 1).contains(&Entity::Object(bears)));
    t.activate(P0, memnarch, 0, &[bears.into()]).unwrap();
    t.resolve_all();
    assert!(t.obj_now(bears).is(CardType::Artifact));
    t.activate(P0, memnarch, 1, &[bears.into()]).unwrap();
    t.resolve_all();
    assert_eq!(controller(&mut t, bears), P0);
    destroy(&mut t, memnarch);
    t.advance_to(P1, Step::Upkeep);
    t.g.recompute();
    assert!(t.obj_now(bears).is(CardType::Artifact));
    assert_eq!(controller(&mut t, bears), P0);
}

#[test]
fn phyrexian_infiltrator_responding_to_itself_swaps_any_two_creatures() {
    cr!("701.12b", "405.5");
    ruling!(
        "Phyrexian Infiltrator",
        "It is possible to activate this ability in response to itself and generate some odd combinations."
    );
    ruling!(
        "Phyrexian Infiltrator",
        "There is no effect if the same player controls both creatures when it resolves."
    );
    supported("Phyrexian Infiltrator");
    let mut t = TestGame::new(2);
    let inf = t.battlefield(P0, "Phyrexian Infiltrator");
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Craw Wurm");
    t.lands(P0, "Island", 8);
    t.activate(P0, inf, 0, &[mine.into()]).unwrap();
    t.activate(P0, inf, 0, &[theirs.into()]).unwrap();
    // The second activation resolves first: Infiltrator for Craw Wurm.
    t.resolve();
    assert_eq!(controller(&mut t, inf), P1);
    assert_eq!(controller(&mut t, theirs), P0);
    // Then the first: Infiltrator back for the Bears.
    t.resolve();
    assert_eq!(controller(&mut t, inf), P0);
    assert_eq!(controller(&mut t, mine), P1);
    assert_eq!(controller(&mut t, theirs), P0);
    // Both creatures controlled by the same player: nothing happens.
    let mut t = TestGame::new(2);
    let inf = t.battlefield(P0, "Phyrexian Infiltrator");
    let mine = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Island", 4);
    t.activate(P0, inf, 0, &[mine.into()]).unwrap();
    t.resolve_all();
    assert_eq!(controller(&mut t, inf), P0);
    assert_eq!(controller(&mut t, mine), P0);
}

#[test]
fn kjeldoran_pride_moves_the_same_aura() {
    cr!("701.3a");
    ruling!(
        "Kjeldoran Pride",
        "The activated ability moves this card to a new permanent."
    );
    supported("Kjeldoran Pride");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let pride = attach_new(&mut t, P0, "Kjeldoran Pride", bears);
    assert_eq!(t.pt(bears), (3, 4));
    t.lands(P0, "Island", 3);
    t.activate(P0, pride, 0, &[giant.into()]).unwrap();
    t.resolve_all();
    // Still the same object, now attached to the Hill Giant.
    assert_eq!(t.g.current(pride), pride);
    assert!(t.on_battlefield(pride));
    assert_eq!(attached_to(&t, pride), Some(Entity::Object(giant)));
    assert_eq!(t.pt(bears), (2, 2));
    assert_eq!(t.pt(giant), (4, 5));
}

// ---------------------------------------------------------------------------
// Lifelink, vigilance, "must be blocked", putting lands onto the battlefield
// ---------------------------------------------------------------------------

#[test]
fn multiple_instances_of_lifelink_are_redundant() {
    cr!("702.15f", "702.15b");
    ruling!(
        "Arctic Aven",
        "Multiple instances of lifelink aren't cumulative. Activating Arctic Aven's ability more than once during a single turn won't cause you to gain more life."
    );
    ruling!(
        "Stonefare Crocodile",
        "Multiple instances of lifelink on the same creature are redundant. Activating Stonefare Crocodile's ability more than once during a single turn won't cause you to gain more life."
    );
    ruling!(
        "Jorubai Murk Lurker",
        "Multiple instances of lifelink on a creature are redundant. Giving the same creature lifelink more than once won’t cause its controller to gain additional life."
    );
    // (creature, lifelink source, lands for two activations, power)
    for (name, lands, power) in [
        ("Arctic Aven", vec![("Plains", 2)], 3),
        ("Stonefare Crocodile", vec![("Swamp", 6)], 3),
    ] {
        supported(name);
        let mut t = TestGame::new(2);
        let c = t.battlefield(P0, name);
        for (l, n) in lands {
            t.lands(P0, l, n);
        }
        for _ in 0..2 {
            t.activate(P0, c, 0, &[]).unwrap();
            t.resolve_all();
        }
        t.attack(&[(c, Entity::Player(P1))], &[]);
        assert_eq!(t.life(P1), 20 - power, "{name}");
        assert_eq!(t.life(P0), 20 + power, "{name}");
    }
    supported("Jorubai Murk Lurker");
    let mut t = TestGame::new(2);
    let lurker = t.battlefield(P0, "Jorubai Murk Lurker");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Swamp", 4);
    for _ in 0..2 {
        t.activate(P0, lurker, 0, &[bears.into()]).unwrap();
        t.resolve_all();
    }
    t.attack(&[(bears, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.life(P0), 22);
}

#[test]
fn towering_thunderfist_needs_vigilance_before_attackers_are_declared() {
    cr!("702.20b", "508.1f");
    ruling!(
        "Towering Thunderfist",
        "You must activate Towering Thunderfist's ability before you declare attackers (at the latest, during the beginning of combat step) in order to have it attack without becoming tapped."
    );
    supported("Towering Thunderfist");
    // In the beginning of combat step: it attacks untapped.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Towering Thunderfist");
    t.lands(P0, "Plains", 1);
    t.set_step(P0, Step::BeginningOfCombat);
    t.activate(P0, giant, 0, &[]).unwrap();
    t.resolve_all();
    attack_with(&mut t, &[(giant, Entity::Player(P1))]);
    assert!(!t.obj_now(giant).tapped);
    // After attackers are declared: too late, it's tapped.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Towering Thunderfist");
    t.lands(P0, "Plains", 1);
    attack_with(&mut t, &[(giant, Entity::Player(P1))]);
    t.activate(P0, giant, 0, &[]).unwrap();
    t.resolve_all();
    assert!(has_kw(&t, giant, KeywordKind::Vigilance));
    assert!(t.obj_now(giant).tapped);
}

#[test]
fn loathsome_catoblepas_needs_only_one_blocker_however_often_activated() {
    cr!("509.1c");
    ruling!(
        "Loathsome Catoblepas",
        "The activated ability doesn't target any creature or make any specific creature have to block. Activating it more than once in a turn provides no additional benefit."
    );
    supported("Loathsome Catoblepas");
    let mut t = TestGame::new(2);
    let cato = t.battlefield(P0, "Loathsome Catoblepas");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Forest", 6);
    for _ in 0..2 {
        let from = t.asked().len();
        t.activate(P0, cato, 0, &[]).unwrap();
        assert!(target_choices(&t, from).is_empty());
        t.resolve_all();
    }
    attack_with(&mut t, &[(cato, Entity::Player(P1))]);
    assert!(!legal_blocks(&mut t, P1, &[]));
    assert!(legal_blocks(&mut t, P1, &[(a, cato)]));
    assert!(legal_blocks(&mut t, P1, &[(b, cato)]));
    assert!(legal_blocks(&mut t, P1, &[(a, cato), (b, cato)]));
}

fn target_choices(t: &TestGame, from: usize) -> Vec<()> {
    t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseTargets { .. }))
        .map(|_| ())
        .collect()
}

#[test]
fn firebrand_ranger_land_is_not_a_land_play() {
    cr!("305.4", "305.2");
    ruling!(
        "Firebrand Ranger",
        "This does not count as your one land you can normally play each turn."
    );
    supported("Firebrand Ranger");
    let mut t = TestGame::new(2);
    let ranger = t.battlefield(P0, "Firebrand Ranger");
    t.lands(P0, "Forest", 1);
    let first = t.hand(P0, "Mountain");
    let second = t.hand(P0, "Island");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[first.into()]);
    t.activate(P0, ranger, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.on_battlefield(first));
    assert!(can_play_land(&mut t, P0, second));
    t.play_land(P0, second).unwrap();
    assert!(t.on_battlefield(second));
}
