//! Rulings batch P131 — the rest of the "refund" group: playing exiled cards follows the
//! normal timing rules, choices made on announcement or on resolution, planeswalker
//! abilities, an ability that puts a commander onto the battlefield, and split cards.

use crate::r_p131_common::*;
use mtg_engine::decision::{Action, Answer, Decision};
use mtg_engine::mana::ManaType;
use mtg_engine::object::{StackKind, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Whether `p` could cast `card` now by any method.
fn castable(t: &mut TestGame, p: PlayerId, card: ObjectId) -> bool {
    t.g.turn.priority = Some(p);
    t.g.recompute();
    let card = t.g.current(card);
    t.g.legal_actions(p)
        .iter()
        .any(|a| matches!(a, Action::Cast { card: c, .. } if *c == card))
}

/// Replaces `p`'s library with the real cards `top_first`.
fn library(t: &mut TestGame, p: PlayerId, top_first: &[&str]) {
    t.g.players[p.idx()].library.clear();
    stack_library(t, p, top_first);
}

/// The exiled Forest P0 may play: playable only in P0's main phase with an empty stack and
/// a land play left (CR 305.1, 305.2, 116.2a).
fn land_timing(t: &mut TestGame, what: &str) {
    let forest = t.g.find_in_zone(Zone::Exile, "Forest");
    assert_eq!(forest.len(), 1, "{what}: no Forest exiled");
    let forest = forest[0];
    assert!(can_play_land(t, P0, forest), "{what}: main phase");
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!can_play_land(t, P0, forest), "{what}: combat");
    t.set_step(P0, Step::PostcombatMain);
    let other = t.hand(P0, "Plains");
    t.play_land(P0, other).unwrap();
    assert!(!can_play_land(t, P0, forest), "{what}: land already played");
}

#[test]
fn exiled_lands_you_may_play_follow_the_normal_timing() {
    cr!("305.1", "305.2", "116.2a");
    ruling!(
        "Chandra, Legacy of Fire",
        "You must pay all costs and follow all normal timing rules for cards played with Chandra, Legacy of Fire's last ability."
    );
    ruling!(
        "Meria, Scholar of Antiquity",
        "You must pay all costs and follow all timing restrictions for cards you play with Meria’s last ability."
    );
    ruling!(
        "Inspired Tinkering",
        "You must pay all costs and follow all timing rules for cards played this way."
    );
    ruling!(
        "Urza, Lord High Artificer",
        "If you exile a land card, you can play it only during your main phase and only if you have an available land play remaining."
    );
    for name in [
        "Chandra, Legacy of Fire",
        "Meria, Scholar of Antiquity",
        "Inspired Tinkering",
        "Urza, Lord High Artificer",
    ] {
        supported(name);
        let mut t = TestGame::new(2);
        library(&mut t, P0, &["Forest", "Island", "Swamp"]);
        match name {
            "Chandra, Legacy of Fire" => {
                let c = t.battlefield(P0, name);
                t.answer_choose(P0, &[obj(c)]);
                activate_containing(&mut t, P0, c, "Remove a loyalty counter").unwrap();
            }
            "Meria, Scholar of Antiquity" => {
                let m = t.battlefield(P0, name);
                let a = t.battlefield(P0, "Sol Ring");
                let b = t.battlefield(P0, "Ornithopter");
                t.answer_choose(P0, &[obj(a), obj(b)]);
                activate_containing(&mut t, P0, m, "Exile the top card").unwrap();
            }
            "Inspired Tinkering" => {
                cast(&mut t, P0, name, &[]);
            }
            _ => {
                library(&mut t, P0, &["Forest"]);
                let u = t.battlefield(P0, name);
                t.lands(P0, "Wastes", 5);
                activate_containing(&mut t, P0, u, "Shuffle").unwrap();
            }
        }
        t.resolve_all();
        land_timing(&mut t, name);
    }
}

#[test]
fn urzas_exiled_sorcery_is_cast_only_at_sorcery_speed() {
    cr!("307.1", "601.3");
    ruling!(
        "Urza, Lord High Artificer",
        "Urza's last ability doesn't change when you can play the exiled card. For example, if you exile a sorcery card, you can cast it only during your main phase when the stack is empty."
    );
    let mut t = TestGame::new(2);
    library(&mut t, P0, &["Divination"]);
    let u = t.battlefield(P0, "Urza, Lord High Artificer");
    t.lands(P0, "Wastes", 5);
    activate_containing(&mut t, P0, u, "Shuffle").unwrap();
    t.resolve_all();
    let div = t.g.find_in_zone(Zone::Exile, "Divination")[0];
    assert!(castable(&mut t, P0, div));
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!castable(&mut t, P0, div));
    t.set_step(P0, Step::PostcombatMain);
    // Not with something on the stack either.
    t.lands(P0, "Island", 1);
    let opt = t.hand(P0, "Opt");
    t.cast(P0, opt).go();
    assert!(!castable(&mut t, P0, div));
    t.resolve_all();
    assert!(castable(&mut t, P0, div));
    t.cast(P0, div)
        .method(mtg_engine::object::CastMethod::Free)
        .try_go()
        .or_else(|_| t.cast(P0, div).try_go())
        .expect("casting the exiled Divination");
}

#[test]
fn culmination_of_studies_exiles_everything_before_acting() {
    cr!("608.2c");
    ruling!(
        "Culmination of Studies",
        "You will exile all X cards from the top of your library before taking any of the listed actions."
    );
    supported("Culmination of Studies");
    let mut t = TestGame::new(2);
    library(
        &mut t,
        P0,
        &["Island", "Opt", "Shock", "Grizzly Bears", "Hill Giant"],
    );
    lands_for_cost(&mut t, P0, "Culmination of Studies");
    t.lands(P0, "Wastes", 3);
    let c = t.hand(P0, "Culmination of Studies");
    t.cast(P0, c).x(3).go();
    t.resolve_all();
    // The card drawn for Opt is the fourth card, not one of the three exiled.
    assert_eq!(hand_names(&t, P0), vec!["Grizzly Bears"]);
    for n in ["Island", "Opt", "Shock"] {
        assert!(t.in_exile(n), "{n}");
    }
    assert_eq!(tokens_with(&t, P0, "Treasure"), 1);
    assert_eq!(t.life(P1), 19);
}

// --- Derevi ----------------------------------------------------------------------------------

#[test]
fn derevis_command_zone_ability_isnt_casting() {
    cr!("903.8", "113.6", "602.1");
    ruling!(
        "Derevi, Empyrial Tactician",
        "When you activate Derevi's last ability, you're not casting Derevi as a spell. The ability can't be countered by something that counters only spells. The ability isn't subject to the commander tax, nor does it increase the tax if you cast Derevi from the command zone later in the game."
    );
    ruling!(
        "Derevi, Empyrial Tactician",
        "You can activate Derevi's last ability only when it is in the command zone."
    );
    supported("Derevi, Empyrial Tactician");
    let mut t = crate::r_s13_common::commander_game();
    let derevi = crate::r_s13_common::commander(&mut t, P0, "Derevi, Empyrial Tactician");
    // A previous cast from the command zone: the tax would be {2}.
    t.g.players[0]
        .commander_casts
        .insert("Derevi, Empyrial Tactician".into(), 1);
    mana(&mut t, P0, ManaType::G, 1);
    mana(&mut t, P0, ManaType::W, 1);
    mana(&mut t, P0, ManaType::U, 1);
    mana(&mut t, P0, ManaType::C, 1);
    assert!(can_activate(&mut t, P0, derevi));
    activate_containing(&mut t, P0, derevi, "from the command zone").unwrap();
    // {1}{G}{W}{U} paid: no tax.
    assert_eq!(pool_total(&t, P0), 0);
    let ab = top(&t);
    assert!(matches!(
        t.g.obj(ab).stack.as_deref().map(|s| &s.kind),
        Some(StackKind::Activated { .. })
    ));
    // Cancel can't target it.
    t.lands(P1, "Island", 3);
    let cancel = t.hand(P1, "Cancel");
    assert!(!castable(&mut t, P1, cancel));
    t.resolve_all();
    assert!(t.on_battlefield(t.g.current(derevi)));
    assert_eq!(
        t.g.players[0]
            .commander_casts
            .get("Derevi, Empyrial Tactician"),
        Some(&1)
    );
    // In a hand, it can't be activated.
    let mut t = crate::r_s13_common::commander_game();
    let d = t.hand(P0, "Derevi, Empyrial Tactician");
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 1);
    assert!(!can_activate(&mut t, P0, d));
}

#[test]
fn a_split_cards_mana_value_off_the_stack_is_both_halves() {
    cr!("709.4", "202.3");
    ruling!(
        "Indulge // Excess",
        "While not on the stack, the characteristics of a split card are the combination of its two halves. For example, Indulge /// Excess has a mana value of 5."
    );
    supported("Indulge // Excess");
    let mut t = TestGame::new(2);
    let c = t.hand(P0, "Indulge // Excess");
    assert_eq!(t.g.mana_value_of(c), 5);
    let g = t.graveyard(P0, "Indulge // Excess");
    assert_eq!(t.g.mana_value_of(g), 5);
}

#[test]
fn lord_windgrace_makes_six_cats_unless_every_target_is_illegal() {
    cr!("608.2b", "115.10");
    ruling!(
        "Lord Windgrace",
        "While resolving Lord Windgrace's last ability, you create six Cat Warrior tokens even if fewer than six nonland permanents are destroyed."
    );
    supported("Lord Windgrace");
    for case in 0..3 {
        let mut t = TestGame::new(2);
        let lw = t.battlefield(P0, "Lord Windgrace");
        loyalty(&mut t, lw, 11);
        let bears = t.battlefield(P1, "Grizzly Bears");
        let targets: Vec<Entity> = if case == 0 { vec![] } else { vec![obj(bears)] };
        t.answer_targets(P0, &targets);
        activate_containing(&mut t, P0, lw, "Destroy up to six").unwrap();
        if case == 2 {
            kill(&mut t, bears);
        }
        t.resolve_all();
        let cats = tokens_with(&t, P0, "Cat");
        assert_eq!(cats, if case == 2 { 0 } else { 6 }, "case {case}");
        assert!(!t.on_battlefield(bears) || case == 0, "case {case}");
    }
}

#[test]
fn reality_spasm_chooses_its_mode_on_cast_and_targets_any_permanents() {
    cr!("700.2a", "601.2b", "115.1");
    ruling!(
        "Reality Spasm",
        "You choose Reality Spasm’s mode as you cast it."
    );
    ruling!(
        "Reality Spasm",
        "You may target any permanents with Reality Spasm, regardless of whether they’re tapped or untapped."
    );
    supported("Reality Spasm");
    let mut t = TestGame::new(2);
    let land = t.battlefield(P1, "Forest");
    t.g.tap(land);
    let bears = t.battlefield(P1, "Grizzly Bears");
    lands_for_cost(&mut t, P0, "Reality Spasm");
    t.lands(P0, "Wastes", 2);
    let c = t.hand(P0, "Reality Spasm");
    let s = t
        .cast(P0, c)
        .x(2)
        .modes(&[0])
        .targets(&[obj(land), obj(bears)])
        .go();
    let chosen = &t.g.obj(s).stack.as_deref().unwrap().chosen;
    assert_eq!(chosen.len(), 1);
    assert_eq!(chosen[0].mode, Some(0));
    assert_eq!(targets_of(&t, s), vec![obj(land), obj(bears)]);
    t.resolve_all();
    assert!(t.obj(land).tapped && t.obj(bears).tapped);
}

#[test]
fn jaya_ballard_may_discard_and_draw_zero_cards() {
    cr!("606.3", "608.2c");
    ruling!(
        "Jaya Ballard",
        "You choose how many cards to discard while Jaya’s second ability is resolving. You can choose to discard zero cards this way (and then draw zero cards) if you wish."
    );
    supported("Jaya Ballard");
    let mut t = TestGame::new(2);
    let jaya = t.battlefield(P0, "Jaya Ballard");
    t.hand(P0, "Grizzly Bears");
    t.hand(P0, "Hill Giant");
    activate_containing(&mut t, P0, jaya, "Discard up to three").unwrap();
    let from = t.asked().len();
    t.answer_choose(P0, &[]);
    t.resolve_all();
    // The discard choice was asked as it resolved.
    assert!(t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseEntities { .. })));
    assert_eq!(hand_names(&t, P0), vec!["Grizzly Bears", "Hill Giant"]);
    assert_eq!(t.library_size(P0), 30);
}

#[test]
fn pore_over_the_pages_untaps_any_lands_after_drawing() {
    cr!("608.2c");
    ruling!(
        "Pore Over the Pages",
        "You choose which lands to untap after you have drawn three cards. Lands controlled by any player may be untapped this way."
    );
    supported("Pore Over the Pages");
    let mut t = TestGame::new(2);
    let theirs = t.battlefield(P1, "Forest");
    t.g.tap(theirs);
    let seen = crate::r_s01_common::watch(
        &mut t,
        P0,
        |d| matches!(d, Decision::ChooseEntities { prompt, .. } if prompt == "Choose"),
        |g| g.players[0].hand.len(),
    );
    cast(&mut t, P0, "Pore Over the Pages", &[]);
    t.answer_choose(P0, &[obj(theirs)]);
    t.resolve_all();
    assert_eq!(*seen.lock().unwrap(), vec![3]);
    assert!(!t.obj(theirs).tapped);
    assert_eq!(t.hand_size(P0), 2);
}

#[test]
fn natures_lore_finds_any_land_with_the_forest_type() {
    cr!("701.23a", "205.3i");
    ruling!(
        "Nature's Lore",
        "You may find a basic Forest or any land card with the land type Forest."
    );
    supported("Nature's Lore");
    let mut t = TestGame::new(2);
    let ground = t.library_top(P0, "Stomping Ground");
    t.library_top(P0, "Forest");
    t.library_top(P0, "Mountain");
    let from = t.asked().len();
    t.answer_choose(P0, &[obj(ground)]);
    t.answer_yes(P0, false);
    cast(&mut t, P0, "Nature's Lore", &[]);
    t.resolve_all();
    let offered = t.asked()[from..]
        .iter()
        .find_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some(candidates.len()),
            _ => None,
        })
        .unwrap();
    assert_eq!(offered, 2);
    assert_eq!(t.named_on_battlefield("Stomping Ground").len(), 1);
}

#[test]
fn lost_jitte_may_untap_an_untapped_land() {
    cr!("115.1");
    ruling!(
        "Lost Jitte",
        "You may target a land that is already untapped with the first mode of Lost Jitte’s activated ability."
    );
    let mut t = TestGame::new(2);
    let jitte = t.battlefield(P0, "Lost Jitte");
    put_counters(&mut t, jitte, counters::CHARGE, 1);
    let forest = t.battlefield(P0, "Forest");
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0]));
    act(&mut t, P0, jitte, "Remove a charge counter", &[obj(forest)]).unwrap();
    let ab = top(&t);
    assert_eq!(targets_of(&t, ab), vec![obj(forest)]);
    t.resolve_all();
    assert!(!t.obj(forest).tapped);
    assert_eq!(t.counters(jitte, counters::CHARGE), 0);
}

#[test]
fn toils_of_night_and_day_has_two_different_targets_and_chooses_later() {
    cr!("115.3", "601.2c", "608.2c");
    ruling!(
        "Toils of Night and Day",
        "You must choose two different targets when you cast Toils of Night and Day. The targets are chosen on announcement. The tap or untap choice is made on resolution."
    );
    let mut t = TestGame::new(2);
    let land = t.battlefield(P1, "Forest");
    t.g.tap(land);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let from = t.asked().len();
    let s = cast(
        &mut t,
        P0,
        "Toils of Night and Day",
        &[obj(land), obj(bears)],
    );
    let offered = target_candidates(&t, P0, from);
    assert_eq!(offered.len(), 2);
    assert!(offered[0].contains(&obj(land)));
    assert!(!offered[1].contains(&obj(land)));
    assert_eq!(targets_of(&t, s), vec![obj(land), obj(bears)]);
    let during_cast = t.asked().len();
    assert!(t.asked()[from..]
        .iter()
        .all(|(_, d)| !matches!(d, Decision::ChooseOption { .. } | Decision::YesNo { .. })));
    t.resolve_all();
    assert!(t.asked()[during_cast..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseOption { .. } | Decision::YesNo { .. })));
}
