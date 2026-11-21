//! Item `rulings-residue-misc-2`, pass B — mana and costs: what a mana ability produces
//! and what "tapped for mana" means (CR 106.12, 605), triggered mana abilities (605.1b),
//! commander color identity mana (903.4), cost reductions (601.2f), untap limits (502.3)
//! and mana values (202.3).

use crate::r_residue_misc_2b_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::mana::ManaType;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::CardType;
use mtg_engine::*;

fn pool(t: &TestGame, p: PlayerId, ty: ManaType) -> usize {
    t.g.player(p).mana_pool.count(ty)
}

fn pool_total(t: &TestGame, p: PlayerId) -> usize {
    t.g.player(p).mana_pool.total()
}

/// Activates the first mana ability of `source` whose text contains `needle`.
fn mana_ability(t: &mut TestGame, p: PlayerId, source: ObjectId, needle: &str) {
    use mtg_engine::ability::AbilityKind;
    t.g.turn.priority = Some(p);
    t.g.recompute();
    let source = t.g.current(source);
    let uid =
        t.g.obj(source)
            .chars
            .abilities
            .iter()
            .find(|a| {
                matches!(&a.kind, AbilityKind::Activated(x) if x.is_mana_ability)
                    && a.text.contains(needle)
            })
            .map(|a| a.uid)
            .unwrap_or_else(|| panic!("no mana ability containing {needle:?}"));
    t.g.activate_ability(p, source, uid).expect("activated");
    t.g.flush_events();
}

/// Queues `p`'s choice of a color among the five, in WUBRG order.
fn color(t: &mut TestGame, p: PlayerId, ty: ManaType) {
    let i = [
        ManaType::W,
        ManaType::U,
        ManaType::B,
        ManaType::R,
        ManaType::G,
    ]
    .iter()
    .position(|x| *x == ty)
    .unwrap();
    t.answer(p, DecisionKind::Option, Answer::Index(i));
}

// --- Benthic Explorers ----------------------------------------------------------------------

#[test]
fn benthic_explorers_looks_at_effects_not_costs_or_riders() {
    cr!("106.7", "605.1a");
    ruling!(
        "Benthic Explorers",
        "Benthic Explorers checks the effects of all mana-producing abilities of the land untapped to pay its cost, but it doesn't check the costs of those abilities. For example, Vivid Crag has the ability \"{T}, Remove a charge counter from Vivid Crag: Add one mana of any color.\" If you untap a tapped Vivid Crag an opponent controls while paying for Benthic Explorers's ability, you can add one mana of any color. It doesn't matter whether that Vivid Crag has a charge counter on it."
    );
    ruling!(
        "Benthic Explorers",
        "Benthic Explorers doesn't care about any restrictions or riders an opponent's land (such as Ancient Ziggurat or Hall of the Bandit Lord) puts on the mana it produces. It just cares about types of mana."
    );
    supported("Benthic Explorers");
    // A Vivid Crag with no charge counters: blue mana.
    let mut t = TestGame::new(2);
    let ex = t.battlefield(P0, "Benthic Explorers");
    let crag = t.battlefield(P1, "Vivid Crag");
    t.g.objects[crag.0 as usize].counters.clear();
    t.g.tap(crag);
    t.answer_choose(P0, &[obj(crag)]);
    color(&mut t, P0, ManaType::U);
    mana_ability(&mut t, P0, ex, "Untap");
    assert_eq!(pool(&t, P0, ManaType::U), 1, "{:?}", t.asked());
    assert!(!t.obj_now(crag).tapped);
    // Ancient Ziggurat's "only to cast a creature spell" doesn't come along: the red mana
    // pays for Lightning Bolt.
    let mut t = TestGame::new(2);
    let ex = t.battlefield(P0, "Benthic Explorers");
    let zig = t.battlefield(P1, "Ancient Ziggurat");
    t.g.tap(zig);
    t.answer_choose(P0, &[obj(zig)]);
    color(&mut t, P0, ManaType::R);
    mana_ability(&mut t, P0, ex, "Untap");
    assert_eq!(pool(&t, P0, ManaType::R), 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(pl(P1)).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}

// --- City of Shadows, Gauntlet of Might ---------------------------------------------------------

#[test]
fn city_of_shadows_keeps_its_counters_and_with_none_isnt_tapped_for_mana() {
    cr!("106.12", "106.12a", "605.1a");
    ruling!(
        "City of Shadows",
        "The storage counters aren't removed when you activate the mana ability."
    );
    ruling!(
        "City of Shadows",
        "You can activate City of Shadows's second ability while it has no storage counters on it, though it won't add any mana in this case. You did not tap it for mana, though, in case any abilities (such as the ability of a Fertile Ground enchanting it) care about that."
    );
    supported("City of Shadows");
    supported("Fertile Ground");
    for n in [2u32, 0] {
        let mut t = TestGame::new(2);
        let city = t.battlefield(P0, "City of Shadows");
        if n > 0 {
            t.g.objects[city.0 as usize]
                .counters
                .insert("storage".into(), n);
        }
        let fg = t.battlefield(P0, "Fertile Ground");
        t.g.attach(fg, obj(city));
        t.g.recompute();
        color(&mut t, P0, ManaType::G);
        mana_ability(&mut t, P0, city, "for each");
        assert!(t.obj_now(city).tapped);
        assert_eq!(t.counters(city, "storage"), n);
        assert_eq!(pool(&t, P0, ManaType::C), n as usize);
        // Fertile Ground adds its mana only if the City produced some.
        assert_eq!(
            pool(&t, P0, ManaType::G),
            usize::from(n > 0),
            "{n} counters"
        );
    }
}

#[test]
fn gauntlet_of_might_adds_red_for_a_dual_land_tapped_for_either_color() {
    cr!("106.12a", "305.6");
    ruling!(
        "Gauntlet of Might",
        "Dual lands which have Mountain as one of their types produce an extra red mana when tapped for either color."
    );
    supported("Gauntlet of Might");
    for ty in [ManaType::R, ManaType::G] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Gauntlet of Might");
        let taiga = t.battlefield(P0, "Taiga");
        let needle = if ty == ManaType::R { "{R}" } else { "{G}" };
        mana_ability(&mut t, P0, taiga, needle);
        t.resolve_all();
        assert_eq!(pool_total(&t, P0), 2, "{ty:?}");
        assert!(pool(&t, P0, ManaType::R) >= 1);
        assert_eq!(pool(&t, P0, ty), if ty == ManaType::R { 2 } else { 1 });
    }
}

// --- Sasaya, Orochi Ascendant // Sasaya's Essence --------------------------------------------

fn essence(t: &mut TestGame) {
    let sasaya = t.battlefield(P0, "Sasaya, Orochi Ascendant");
    assert!(mtg_engine::flip::flip(&mut t.g, sasaya));
    t.g.recompute();
}

#[test]
fn sasayas_essence_is_a_triggered_mana_ability_matching_the_mana_made() {
    cr!("605.1b", "605.3b", "106.12a");
    ruling!(
        "Sasaya, Orochi Ascendant // Sasaya's Essence",
        "Sasaya's Essence has a triggered mana ability. Like activated mana abilities, the triggered mana ability doesn't use the stack."
    );
    ruling!(
        "Sasaya, Orochi Ascendant // Sasaya's Essence",
        "The triggered mana ability counts the number of other lands with that same name and adds that much mana. The type (that is, the color of mana or colorless mana) is the same as the mana the land just produced."
    );
    ruling!(
        "Sasaya, Orochi Ascendant // Sasaya's Essence",
        "If Sasaya's Essence's controller has two Brushlands and taps one of them for {W}, Sasaya's Essence adds another {W}. It won't produce {G} or {C} unless the land was tapped for {G} or {C} instead."
    );
    ruling!(
        "Sasaya, Orochi Ascendant // Sasaya's Essence",
        "If Sasaya's Essence's controller has four Mossfire Valley and taps one of them for {R}{G}, the Essence will add three mana (one for each other Mossfire Valley) of any combination of {R} and/or {G}."
    );
    supported("Sasaya, Orochi Ascendant // Sasaya's Essence");
    // Two Brushlands, one tapped for {W}: another {W}, with nothing on the stack.
    let mut t = TestGame::new(2);
    essence(&mut t);
    let b = t.lands(P0, "Brushland", 2);
    // (Brushland offers {G} or {W}.)
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    mana_ability(&mut t, P0, b[0], "{W}");
    assert!(t.g.stack.is_empty());
    assert_eq!(pool(&t, P0, ManaType::W), 2, "{}", t.dump_log());
    assert_eq!(pool_total(&t, P0), 2);
    // Four Mossfire Valleys, one tapped for {R}{G}: three more, each {R} or {G}.
    let mut t = TestGame::new(2);
    essence(&mut t);
    let v = t.lands(P0, "Mossfire Valley", 4);
    add(&mut t, P0, ManaType::C, 1);
    mana_ability(&mut t, P0, v[0], "{R}{G}");
    assert!(t.g.stack.is_empty());
    let (r, g) = (pool(&t, P0, ManaType::R), pool(&t, P0, ManaType::G));
    assert_eq!(r + g, 5, "{}", t.dump_log());
    assert!(r >= 1 && g >= 1);
    assert_eq!(pool(&t, P0, ManaType::C), 0);
}

#[test]
fn sasaya_reveals_without_seven_lands_and_needs_no_tap() {
    cr!("608.2c", "302.6", "602.5a");
    ruling!(
        "Sasaya, Orochi Ascendant // Sasaya's Essence",
        "You don't need to have seven lands in hand to reveal your hand. You leave your hand revealed until the ability resolves. If you don't have seven or more land cards in your hand at that time, the effect does nothing."
    );
    ruling!(
        "Sasaya, Orochi Ascendant // Sasaya's Essence",
        "You don't need to tap Sasaya to use its ability, so you can activate the ability on the turn Sasaya comes under your control."
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let s = t.battlefield_sick(P0, "Sasaya, Orochi Ascendant");
    for _ in 0..6 {
        t.hand(P0, "Forest");
    }
    t.activate(P0, s, 0, &[])
        .expect("activated while summoning sick");
    // A seventh land arrives before it resolves: it flips.
    t.hand(P0, "Forest");
    t.resolve_all();
    assert_eq!(t.obj_now(s).chars.name, "Sasaya's Essence");
    // Six lands as it resolves: nothing.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let s = t.battlefield_sick(P0, "Sasaya, Orochi Ascendant");
    for _ in 0..6 {
        t.hand(P0, "Forest");
    }
    t.activate(P0, s, 0, &[]).expect("activated");
    t.resolve_all();
    assert_eq!(t.obj_now(s).chars.name, "Sasaya, Orochi Ascendant");
}

// --- C.A.M.P. -----------------------------------------------------------------------------------

#[test]
fn camp_makes_junk_if_the_creature_shares_any_of_the_colors_produced() {
    cr!("106.12a", "105.4");
    ruling!(
        "C.A.M.P.",
        "If the fortified land produced more than one color of mana, you'll create a Junk token as long as the target creature is at least one of those colors."
    );
    supported("C.A.M.P.");
    for (creature, junk) in [("Grizzly Bears", true), ("Savannah Lions", false)] {
        let mut t = TestGame::new(2);
        let camp = t.battlefield(P0, "C.A.M.P.");
        let valley = t.battlefield(P0, "Mossfire Valley");
        t.g.attach(camp, obj(valley));
        t.g.recompute();
        let c = t.battlefield(P0, creature);
        add(&mut t, P0, ManaType::C, 1);
        t.answer_targets(P0, &[obj(c)]);
        mana_ability(&mut t, P0, valley, "{R}{G}");
        t.resolve_all();
        assert_eq!(t.counters(c, mtg_engine::types::counters::PLUS1), 1);
        let junk_n = tokens(&t, P0)
            .into_iter()
            .filter(|id| t.obj_now(*id).chars.name.contains("Junk"))
            .count();
        assert_eq!(junk_n, usize::from(junk), "{creature}: {}", t.dump_log());
    }
}

// --- Opal Palace, Path of Ancestry ------------------------------------------------------------

/// A Commander game where P0's commanders are `cmdrs` (in the command zone), and P0 has
/// the land `land`; returns the game and the land.
fn identity_land(land: &str, cmdrs: &[&str]) -> (TestGame, ObjectId) {
    let mut t = commander_game(2);
    t.set_step(P0, Step::PrecombatMain);
    for c in cmdrs {
        let id = t.command(P0, c);
        make_commander(&mut t, id);
    }
    let l = t.battlefield(P0, land);
    (t, l)
}

/// The colors of mana `p` is offered as the mana ability resolves (`None` if no choice
/// was asked), and the mana added.
fn identity_mana(t: &mut TestGame, land: ObjectId, needle: &str) -> usize {
    t.g.turn.priority = Some(P0);
    mana_ability(t, P0, land, needle);
    pool_total(t, P0)
}

#[test]
fn opal_palace_and_path_of_ancestry_make_mana_only_of_commander_colors() {
    cr!("903.4", "903.4f", "106.7");
    ruling!(
        "Opal Palace",
        "If you don't have a commander, Opal Palace's last ability produces no mana."
    );
    ruling!(
        "Path of Ancestry",
        "If you don't have a commander, Path of Ancestry's ability produces no mana."
    );
    ruling!(
        "Opal Palace",
        "If your commander is a card that has no colors in its color identity, Opal Palace's last ability produces no mana. It doesn't produce {C}."
    );
    ruling!(
        "Path of Ancestry",
        "If your commander is a card that has no colors in its color identity, Path of Ancestry's ability produces no mana. It doesn't produce {C}."
    );
    ruling!(
        "Opal Palace",
        "If you have two commanders, the last ability adds one mana of any color in their combined color identities."
    );
    supported("Opal Palace");
    supported("Path of Ancestry");
    for (land, needle, generic) in [
        ("Opal Palace", "commander's color identity", 1),
        ("Path of Ancestry", "commander's color identity", 0),
    ] {
        for cmdrs in [&[][..], &["Ornithopter"][..]] {
            let (mut t, l) = identity_land(land, cmdrs);
            add(&mut t, P0, ManaType::C, generic);
            assert_eq!(identity_mana(&mut t, l, needle), 0, "{land} {cmdrs:?}");
            assert!(t.obj_now(l).tapped);
        }
    }
    // Two commanders (a green and a blue one): either color.
    for ty in [ManaType::G, ManaType::U] {
        let (mut t, l) = identity_land("Opal Palace", &["Grizzly Bears", "Storm Crow"]);
        add(&mut t, P0, ManaType::C, 1);
        let from = t.asked().len();
        // The offered colors are the combined identity: pick by name.
        t.answer(
            P0,
            DecisionKind::Option,
            Answer::Index(if ty == ManaType::G { 1 } else { 0 }),
        );
        identity_mana(&mut t, l, "commander's color identity");
        let offered = asked_since(&t, from)
            .into_iter()
            .find_map(|(_, d)| match d {
                Decision::ChooseOption { options, .. } => Some(options),
                _ => None,
            });
        let offered = offered.expect("a color choice");
        assert_eq!(offered.len(), 2, "{offered:?}");
        assert_eq!(pool_total(&t, P0), 1);
        assert!(pool(&t, P0, ManaType::G) + pool(&t, P0, ManaType::U) == 1);
    }
}

#[test]
fn path_of_ancestry_checks_the_commanders_types_as_the_spell_is_cast() {
    cr!("106.6", "903.3");
    ruling!(
        "Path of Ancestry",
        "Your commander's creature types are checked immediately after you cast a creature spell spending mana from Path of Ancestry's last ability. They aren't set before the game begins, and they may not be the same types your commander had when you activated that ability."
    );
    // A Bird commander (Storm Crow, on the battlefield) and a Bear spell: they share a
    // creature type only if Maskwood Nexus (creatures you control are every type) arrives
    // after the mana was made and before the spell is cast.
    for (nexus, scry) in [(true, 1), (false, 0)] {
        supported("Maskwood Nexus");
        let (mut t, path) = identity_land("Path of Ancestry", &["Storm Crow"]);
        let cmdr =
            t.g.find_in_zone(mtg_engine::object::Zone::Command, "Storm Crow")[0];
        mv(&mut t, cmdr, mtg_engine::object::Zone::Battlefield);
        mana_ability(&mut t, P0, path, "commander's color identity");
        assert_eq!(pool(&t, P0, ManaType::U), 1);
        if nexus {
            t.battlefield(P0, "Maskwood Nexus");
        }
        t.lands(P0, "Forest", 1);
        let bears = t.hand(P0, "Grizzly Bears");
        t.cast(P0, bears).go();
        t.resolve_all();
        let scries = t
            .asked()
            .iter()
            .filter(|(_, d)| matches!(d, Decision::Scry { .. }))
            .count();
        assert_eq!(scries, scry, "{}", t.dump_log());
    }
}

// --- Smoke --------------------------------------------------------------------------------------

#[test]
fn smoke_an_animated_land_counts_as_the_creature_and_the_land() {
    cr!("502.3", "305.9");
    ruling!(
        "Smoke",
        "Animated lands are affected by this spell. If on the battlefield with an effect that limits the number of land you untap, untapping an animated land will count as the one creature and the one land you can untap... thereby limiting you to one thing to be untapped."
    );
    supported("Smoke");
    supported("Winter Orb");
    supported("Living Lands");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Smoke");
    t.battlefield(P1, "Winter Orb");
    t.battlefield(P1, "Living Lands");
    let forest = t.battlefield(P0, "Forest");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let mountain = t.battlefield(P0, "Mountain");
    for id in [forest, bears, mountain] {
        t.g.tap(id);
    }
    t.g.recompute();
    assert!(t.obj_now(forest).is(CardType::Creature));
    t.answer_choose(P0, &[obj(forest)]);
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    assert!(!t.obj_now(forest).tapped);
    assert!(t.obj_now(bears).tapped && t.obj_now(mountain).tapped);
}

// --- Inquisitive Glimmer --------------------------------------------------------------------------

#[test]
fn inquisitive_glimmer_reductions_dont_combine() {
    cr!("601.2f", "709.5e");
    ruling!(
        "Inquisitive Glimmer",
        "The cost reduction from Inquisitive Glimmer's first ability applies only to generic mana in the total cost of enchantment spells you cast. Similarly, the cost reduction from its second ability applies only to generic mana in the unlock costs of Rooms you control. (In other words, it's impossible for these abilities to combine to make anything cost {2} less.)"
    );
    supported("Inquisitive Glimmer");
    // Glassworks {2}{R}: {R} alone isn't enough ({1}{R} is).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Inquisitive Glimmer");
    t.battlefield(P1, "Gray Ogre");
    let card = t.hand(P0, "Glassworks // Shattered Yard");
    add(&mut t, P0, ManaType::R, 1);
    assert!(!can_cast(&mut t, P0, card, CastMethod::Half(0)));
    add(&mut t, P0, ManaType::C, 1);
    assert!(can_cast(&mut t, P0, card, CastMethod::Half(0)));
    // Rancor ({G}) has no generic mana to reduce: {C} doesn't pay for it.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Inquisitive Glimmer");
    t.battlefield(P0, "Grizzly Bears");
    let ench = t.hand(P0, "Rancor");
    add(&mut t, P0, ManaType::C, 1);
    assert!(!can_cast(&mut t, P0, ench, CastMethod::Normal));
}

// --- March of Reckless Joy --------------------------------------------------------------------

#[test]
fn march_of_reckless_joy_mana_value_and_exiling_more_than_needed() {
    cr!("202.3e", "601.2f", "601.2b");
    ruling!(
        "March of Reckless Joy",
        "The mana value of March of Reckless Joy while it's on the stack is the value chosen for X plus 1, no matter how much mana you actually paid to cast it."
    );
    ruling!(
        "March of Reckless Joy",
        "You may exile more cards than necessary for March of Reckless Joy's first ability, but you can't reduce the mana it costs to less than {R} this way."
    );
    supported("March of Reckless Joy");
    let mut t = TestGame::new(2);
    let reds: Vec<ObjectId> = (0..2).map(|_| t.hand(P0, "Lightning Bolt")).collect();
    let march = t.hand(P0, "March of Reckless Joy");
    add(&mut t, P0, ManaType::R, 1);
    // X = 1 ({1}{R}), exiling two red cards: it costs {R}.
    t.answer(P0, DecisionKind::OptionalCost, Answer::Number(2));
    t.answer_choose(P0, &reds.iter().map(|c| obj(*c)).collect::<Vec<_>>());
    let spell = t.cast(P0, march).x(1).go();
    assert!(reds
        .iter()
        .all(|c| t.zone(*c) == mtg_engine::object::Zone::Exile));
    assert_eq!(pool_total(&t, P0), 0);
    assert_eq!(t.g.mana_value_of(t.g.current(spell)), 2);
}

#[test]
fn march_of_reckless_joy_cards_are_chosen_as_you_play_them() {
    cr!("601.2", "305.1");
    ruling!(
        "March of Reckless Joy",
        "You choose which cards that you are playing from exile as you play them, not as March of Reckless Joy is resolving."
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let cards = stack_library(&mut t, P0, &["Mountain", "Lightning Bolt", "Shock"]);
    let march = t.hand(P0, "March of Reckless Joy");
    t.lands(P0, "Mountain", 4);
    let from = t.asked().len();
    t.cast(P0, march).x(3).go();
    t.resolve_all();
    // No choice of cards was asked as it resolved.
    assert!(!asked_since(&t, from)
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseEntities { .. })));
    assert!(cards
        .iter()
        .all(|c| t.zone(*c) == mtg_engine::object::Zone::Exile));
    // Any two of the three: Shock and Lightning Bolt, chosen one at a time.
    t.lands(P0, "Mountain", 2);
    let bolt = t.g.current(cards[1]);
    t.cast(P0, bolt).target(pl(P1)).go();
    t.resolve_all();
    let shock = t.g.current(cards[2]);
    t.cast(P0, shock).target(pl(P1)).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 15);
    let land = t.g.current(cards[0]);
    assert!(t.play_land(P0, land).is_err(), "only two of them");
}

// --- Font of Magic ----------------------------------------------------

#[test]
fn font_of_magic_counts_a_background_commander() {
    cr!("903.8", "903.3a");
    ruling!(
        "Font of Magic",
        "This includes Backgrounds that are your commander."
    );
    supported("Font of Magic");
    let mut t = commander_game(2);
    t.set_step(P0, Step::PrecombatMain);
    let bg = t.command(P0, "Acolyte of Bahamut");
    make_commander(&mut t, bg);
    t.g.players[P0.idx()]
        .commander_casts
        .insert("Acolyte of Bahamut".into(), 2);
    t.g.recompute();
    t.battlefield(P0, "Font of Magic");
    // Divination ({2}{U}) costs {U}.
    let div = t.hand(P0, "Divination");
    add(&mut t, P0, ManaType::U, 1);
    assert!(can_cast(&mut t, P0, div, CastMethod::Normal));
}

// --- Twins of Discord -------------------------------------------------------------------------

#[test]
fn twins_of_discord_x_is_zero_and_face_down_is_zero() {
    cr!("202.3e", "708.4", "509.1b");
    ruling!(
        "Twins of Discord",
        "If a creature on the battlefield has {X} in its mana cost, X is 0 when determining its mana value."
    );
    ruling!(
        "Twins of Discord",
        "The mana value of a face-down creature is 0."
    );
    supported("Endless One");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let twins = t.battlefield(P0, "Twins of Discord");
    let endless = t.battlefield(P1, "Endless One");
    plus1(&mut t, endless, 3);
    let facedown = crate::r_s11_common::manifest_card(&mut t, P1, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    // Even: the Endless One (0) and the face-down creature (0) can't block; the Bears (2)
    // can't either; choose odd and they all can.
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    attack_with(&mut t, &[(twins, pl(P1))]);
    t.resolve_all();
    let blockers = |t: &mut TestGame| {
        [endless, facedown, bears]
            .iter()
            .map(|b| legal_blocks(t, P1, &[(*b, twins)]))
            .collect::<Vec<_>>()
    };
    let chosen = t.asked().iter().rev().find_map(|(_, d)| match d {
        Decision::ChooseOption { options, .. } => Some(options.clone()),
        _ => None,
    });
    let even = chosen
        .expect("odd or even")
        .get(1)
        .is_some_and(|o| o.to_lowercase().contains("even"));
    let want = if even {
        vec![false, false, false]
    } else {
        vec![true, true, true]
    };
    assert_eq!(blockers(&mut t), want);
}
