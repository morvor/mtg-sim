//! Rulings batch P192 — choices made as spells and abilities resolve (CR 608.2c–d):
//! Clarion Ultimatum, Time Wipe, Genesis Ultimatum, Reap Intellect, Another Round, Oona,
//! Biomantic Mastery's targets, Dregs of Sorrow's X, Enter the Infinite's hand size,
//! Hugs's and Bloodsoaked Insight's play permissions (CR 601.3, 305.1), Klauth's Will's
//! modes, Dawnglow Infusion's spent mana, Vish Kal's last known power, Deity of Scars's
//! regeneration, Leviathan's untap and Hour of Eternity's token copies.

use crate::r_p194_common::mana;
use crate::r_s01_common::*;
use crate::r_s02_common::{can_cast, can_play_land, destroy, target_candidates};
use crate::r_s05_common::move_to;
use crate::r_s06_common::activate_containing;
use crate::r_s08_common::mana_value;
use crate::r_s29_common::{choose_modes, put_counters};
use crate::r_s35_common::pick_options;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::events::Event;
use mtg_engine::game::GameConfig;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

fn named_on_bf(t: &TestGame, p: PlayerId, name: &str) -> Vec<ObjectId> {
    t.g.permanents()
        .filter(|o| o.controller == p && o.chars.name == name)
        .map(|o| o.id)
        .collect()
}

// ---------------------------------------------------------------- Clarion Ultimatum

#[test]
fn clarion_ultimatum_chooses_on_resolution_and_searches_once() {
    cr!("608.2c", "701.23a", "115.1");
    ruling!(
        "Clarion Ultimatum",
        "This spell has no targets. You don't choose five permanents you control until Clarion Ultimatum resolves. If you control fewer than five permanents at that time, choose each permanent you do control."
    );
    ruling!("Clarion Ultimatum", "You make just one search.");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Llanowar Elves");
    stack_library(&mut t, P0, &["Grizzly Bears", "Llanowar Elves", "Shock"]);
    let card = t.hand(P0, "Clarion Ultimatum");
    mana(&mut t, P0, ManaType::G, 2);
    mana(&mut t, P0, ManaType::W, 3);
    mana(&mut t, P0, ManaType::U, 2);
    let from = t.asked().len();
    t.cast(P0, card).go();
    assert!(
        t.asked()[from..]
            .iter()
            .all(|(_, d)| !matches!(d, Decision::ChooseTargets { .. })),
        "no targets"
    );
    // A permanent that arrives before it resolves can be chosen.
    t.battlefield(P0, "Shock Troops");
    let from = t.asked().len();
    t.resolve_all();
    // With fewer than five permanents, each is chosen (no choice to make, or all of them).
    let chosen: Vec<_> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities {
                candidates, min, ..
            } if candidates.contains(&obj(bears)) => Some((candidates.len(), *min)),
            _ => None,
        })
        .collect();
    assert!(
        chosen.iter().all(|(n, min)| *n as u32 == *min),
        "{chosen:?}"
    );
    assert_eq!(named_on_bf(&t, P0, "Grizzly Bears").len(), 2);
    assert_eq!(named_on_bf(&t, P0, "Llanowar Elves").len(), 2);
    let searches =
        t.g.turn_events
            .iter()
            .filter(|e| matches!(e, Event::Searched { player } if *player == P0))
            .count();
    assert_eq!(searches, 1);
}

// ---------------------------------------------------------------- Time Wipe

#[test]
fn time_wipe_chooses_the_creature_to_return_as_it_resolves() {
    cr!("608.2c", "608.2f");
    ruling!(
        "Time Wipe",
        "You chose which creature to return to its owner's hand while Time Wipe is resolving."
    );
    supported("Time Wipe");
    let mut t = TestGame::new(2);
    let angel = t.battlefield(P0, "Serra Angel");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let seen = watch(
        &mut t,
        P0,
        |d| matches!(d, Decision::ChooseEntities { .. }),
        |g| {
            (
                g.stack.len(),
                g.permanents().filter(|o| o.is(CardType::Creature)).count(),
            )
        },
    );
    t.answer_choose(P0, &[obj(angel)]);
    let card = t.hand(P0, "Time Wipe");
    mana(&mut t, P0, ManaType::W, 2);
    mana(&mut t, P0, ManaType::U, 1);
    mana(&mut t, P0, ManaType::C, 2);
    t.cast(P0, card).go();
    t.resolve_all();
    // Chosen with Time Wipe on the stack and every creature still there.
    assert_eq!(*seen.lock().unwrap(), vec![(1, 3)]);
    assert!(t.in_hand(P0, "Serra Angel"));
    assert_eq!(t.zone(bears), Zone::Graveyard(P0));
    assert_eq!(t.zone(theirs), Zone::Graveyard(P1));
}

// ---------------------------------------------------------------- Genesis Ultimatum

#[test]
fn genesis_ultimatum_may_put_no_permanent_cards_onto_the_battlefield() {
    cr!("608.2c", "701.20a");
    ruling!(
        "Genesis Ultimatum",
        "You can choose to not put a permanent card onto the battlefield this way."
    );
    ruling!(
        "Genesis Ultimatum",
        "You can choose to not put any permanent cards onto the battlefield this way."
    );
    for put in [false, true] {
        let mut t = TestGame::new(2);
        let lib = stack_library(
            &mut t,
            P0,
            &["Grizzly Bears", "Forest", "Shock", "Serra Angel", "Island"],
        );
        if put {
            t.answer_choose(P0, &[obj(lib[0])]);
        } else {
            t.answer_choose(P0, &[]);
        }
        let card = t.hand(P0, "Genesis Ultimatum");
        mana(&mut t, P0, ManaType::G, 2);
        mana(&mut t, P0, ManaType::U, 3);
        mana(&mut t, P0, ManaType::R, 2);
        let hand = t.hand_size(P0);
        t.cast(P0, card).go();
        t.resolve_all();
        let on_bf = usize::from(put);
        assert_eq!(named_on_bf(&t, P0, "Grizzly Bears").len(), on_bf);
        assert_eq!(t.hand_size(P0), hand - 1 + 5 - on_bf);
        assert!(t.in_hand(P0, "Serra Angel") && t.in_hand(P0, "Forest"));
        assert!(t.in_exile("Genesis Ultimatum"));
    }
}

// ---------------------------------------------------------------- Reap Intellect

#[test]
fn reap_intellect_may_leave_cards_with_the_same_name() {
    cr!("608.2c", "701.23a");
    ruling!(
        "Reap Intellect",
        "You can choose to leave cards with those names in the zone they are in."
    );
    supported("Reap Intellect");
    for take in [false, true] {
        let mut t = TestGame::new(2);
        let in_hand = t.hand(P1, "Grizzly Bears");
        let in_gy = t.graveyard(P1, "Grizzly Bears");
        t.answer_choose(P0, &[obj(in_hand)]);
        if take {
            t.answer_choose(P0, &[obj(in_gy)]);
        } else {
            t.answer_choose(P0, &[]);
        }
        let card = t.hand(P0, "Reap Intellect");
        mana(&mut t, P0, ManaType::U, 1);
        mana(&mut t, P0, ManaType::B, 1);
        mana(&mut t, P0, ManaType::C, 3);
        t.cast(P0, card).x(1).target(Entity::Player(P1)).go();
        t.resolve_all();
        assert_eq!(t.zone(in_hand), Zone::Exile);
        assert_eq!(t.zone(in_gy) == Zone::Exile, take, "take: {take}");
        assert_eq!(t.in_graveyard(P1, "Grizzly Bears"), !take);
    }
}

// ---------------------------------------------------------------- Another Round

#[test]
fn another_round_may_exile_a_different_set_each_time() {
    cr!("608.2c", "400.7");
    ruling!(
        "Another Round",
        "You don’t have to choose the same set of creatures each time you repeat this process."
    );
    supported("Another Round");
    supported("Lone Missionary");
    let mut t = TestGame::new(2);
    let missionary = t.battlefield(P0, "Lone Missionary");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_choose(P0, &[obj(missionary)]);
    t.answer_choose(P0, &[obj(bears)]);
    let card = t.hand(P0, "Another Round");
    mana(&mut t, P0, ManaType::W, 1);
    mana(&mut t, P0, ManaType::C, 4);
    t.cast(P0, card).x(1).go();
    t.resolve_all();
    // Lone Missionary was flickered once (4 life), the Bears once.
    assert_eq!(t.life(P0), 24);
    assert_ne!(t.g.current(bears), bears);
    assert!(t.on_battlefield(bears) && t.on_battlefield(missionary));
}

// ---------------------------------------------------------------- Oona

#[test]
fn oona_targets_on_activation_and_chooses_the_color_on_resolution() {
    cr!("601.2c", "608.2c");
    ruling!(
        "Oona, Queen of the Fae",
        "You target the opponent when you activate Oona's ability. You don't choose the color until the ability resolves."
    );
    supported("Oona, Queen of the Fae");
    let mut t = TestGame::new(2);
    let oona = t.battlefield(P0, "Oona, Queen of the Fae");
    stack_library(&mut t, P1, &["Island", "Grizzly Bears", "Serra Angel"]);
    let color_asks = |t: &TestGame, from: usize| {
        t.asked()[from..]
            .iter()
            .filter(|(p, d)| {
                *p == P0
                    && matches!(d, Decision::ChooseOption { options, .. }
                        if options.iter().any(|o| o == "white"))
            })
            .count()
    };
    let from = t.asked().len();
    mana(&mut t, P0, ManaType::U, 1);
    mana(&mut t, P0, ManaType::C, 3);
    t.answer(P0, DecisionKind::X, Answer::Number(3));
    t.answer_targets(P0, &[Entity::Player(P1)]);
    activate_containing(&mut t, P0, oona, "Choose a color").unwrap();
    assert_eq!(color_asks(&t, from), 0);
    pick_options(&mut t, P0, &["white"]);
    t.resolve_all();
    assert_eq!(color_asks(&t, from), 1);
    // Serra Angel was the only white card exiled.
    assert_eq!(tokens(&t, P0).len(), 1);
}

// ---------------------------------------------------------------- Biomantic Mastery

#[test]
fn biomantic_mastery_targets_two_different_players_one_may_be_you() {
    cr!("115.3", "601.2c");
    ruling!(
        "Biomantic Mastery",
        "You choose both targets before you draw any cards. The targets must be different. One of them may be you."
    );
    supported("Biomantic Mastery");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Grizzly Bears");
    let card = t.hand(P0, "Biomantic Mastery");
    mana(&mut t, P0, ManaType::G, 3);
    mana(&mut t, P0, ManaType::C, 4);
    let from = t.asked().len();
    let hand = t.hand_size(P0);
    t.cast(P0, card)
        .targets(&[Entity::Player(P0), Entity::Player(P1)])
        .go();
    let cands = target_candidates(&t, P0, from);
    assert_eq!(cands.len(), 2);
    assert!(cands[0].contains(&Entity::Player(P0)));
    assert_eq!(cands[1], vec![Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand - 1 + 3);
}

// ---------------------------------------------------------------- Dregs of Sorrow

#[test]
fn dregs_of_sorrow_x_can_be_zero() {
    cr!("107.3", "601.2c");
    ruling!("Dregs of Sorrow", "X can be zero.");
    supported("Dregs of Sorrow");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let card = t.hand(P0, "Dregs of Sorrow");
    mana(&mut t, P0, ManaType::B, 1);
    mana(&mut t, P0, ManaType::C, 4);
    let hand = t.hand_size(P0);
    t.cast(P0, card).x(0).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Dregs of Sorrow"));
    assert_eq!(t.hand_size(P0), hand - 1);
    assert!(t.on_battlefield(bears));
}

// ---------------------------------------------------------------- Enter the Infinite

#[test]
fn enter_the_infinite_no_maximum_hand_size_until_your_next_turn() {
    cr!("402.2", "514.1", "611.2b");
    ruling!(
        "Enter the Infinite",
        "You won't have to discard any cards during the cleanup step of the turn you cast Enter the Infinite. You will have to discard down to your maximum hand size during the cleanup step of your next turn."
    );
    supported("Enter the Infinite");
    let mut t = TestGame::new(2);
    let card = t.hand(P0, "Enter the Infinite");
    mana(&mut t, P0, ManaType::U, 4);
    mana(&mut t, P0, ManaType::C, 8);
    let lib = t.library_size(P0);
    t.cast(P0, card).go();
    t.resolve_all();
    assert_eq!(t.library_size(P0), 1);
    assert_eq!(t.hand_size(P0), lib - 1);
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.hand_size(P0), lib - 1);
    t.advance_to(P0, Step::End);
    t.advance_to(P1, Step::Upkeep);
    assert!(!t.has_lost(P0));
    // P0's next turn: drew the card on top, then discarded down to seven.
    assert_eq!(t.hand_size(P0), 7);
}

// ---------------------------------------------------------------- play permissions

#[test]
fn hugs_exiled_cards_stay_playable_after_hugs_leaves() {
    cr!("611.2a", "601.3");
    ruling!(
        "Hugs, Grisly Guardian",
        "You may play the exiled cards even if Hugs is no longer on the battlefield or under your control."
    );
    let mut t = TestGame::new(2);
    let lib = stack_library(&mut t, P0, &["Forest", "Grizzly Bears"]);
    let card = t.hand(P0, "Hugs, Grisly Guardian");
    mana(&mut t, P0, ManaType::R, 2);
    mana(&mut t, P0, ManaType::G, 2);
    mana(&mut t, P0, ManaType::C, 2);
    let hugs = t.cast(P0, card).x(2).go();
    t.resolve_all();
    assert_eq!(t.zone(lib[0]), Zone::Exile);
    assert_eq!(t.zone(lib[1]), Zone::Exile);
    destroy(&mut t, hugs);
    let forest = t.g.current(lib[0]);
    assert!(can_play_land(&mut t, P0, forest));
    t.play_land(P0, forest).unwrap();
    mana(&mut t, P0, ManaType::G, 2);
    let bears = t.g.current(lib[1]);
    assert!(can_cast(&mut t, P0, bears, CastMethod::Normal));
    t.cast(P0, bears).go();
    t.resolve_all();
    assert!(t.on_battlefield(lib[1]));
}

#[test]
fn bloodsoaked_insight_lands_follow_normal_timing() {
    cr!("305.1", "305.2", "116.2a");
    ruling!(
        "Bloodsoaked Insight // Sanguine Morass",
        "if one of the exiled cards is a land card, you may play it only during your main phase while the stack is empty."
    );
    supported("Bloodsoaked Insight // Sanguine Morass");
    let mut t = TestGame::new(2);
    let lib = stack_library(&mut t, P1, &["Forest", "Grizzly Bears", "Shock"]);
    let card = t.hand(P0, "Bloodsoaked Insight // Sanguine Morass");
    mana(&mut t, P0, ManaType::B, 2);
    mana(&mut t, P0, ManaType::C, 5);
    t.cast(P0, card).target(Entity::Player(P1)).go();
    t.resolve_all();
    let forest = t.g.current(lib[0]);
    assert_eq!(t.zone(forest), Zone::Exile);
    // Not with a spell on the stack.
    let bolt = t.hand(P0, "Shock");
    mana(&mut t, P0, ManaType::R, 1);
    t.cast(P0, bolt).target(Entity::Player(P1)).go();
    assert!(!can_play_land(&mut t, P0, forest));
    t.resolve_all();
    // Not in combat.
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!can_play_land(&mut t, P0, forest));
    // Not on the opponent's turn.
    t.set_step(P1, Step::PrecombatMain);
    assert!(!can_play_land(&mut t, P0, forest));
    // In P0's next main phase with an empty stack: yes.
    t.set_step(P0, Step::PostcombatMain);
    assert!(can_play_land(&mut t, P0, forest));
    t.play_land(P0, forest).unwrap();
    assert_eq!(t.obj_now(forest).controller, P0);
}

// ---------------------------------------------------------------- Klauth's Will

#[test]
fn klauths_will_modes_are_fixed_once_chosen() {
    cr!("700.2a", "601.2b");
    ruling!(
        "Klauth's Will",
        "Whether or not you control a commander is determined only once, as you choose the modes for this spell."
    );
    supported("Klauth's Will");
    for commander in [false, true] {
        let mut t = TestGame::with_config(
            2,
            GameConfig {
                variant: mtg_engine::game::Variant::Commander,
                ..Default::default()
            },
        );
        let dragon = t.battlefield(P0, "Shivan Dragon");
        if commander {
            t.g.objects[dragon.0 as usize].is_commander = true;
            t.g.players[P0.idx()]
                .commander_names
                .push("Shivan Dragon".into());
            t.g.dirty = true;
            t.g.recompute();
        }
        t.battlefield(P1, "Grizzly Bears");
        let relic = t.battlefield(P1, "Sol Ring");
        let card = t.hand(P0, "Klauth's Will");
        mana(&mut t, P0, ManaType::R, 2);
        mana(&mut t, P0, ManaType::G, 1);
        mana(&mut t, P0, ManaType::C, 2);
        choose_modes(&mut t, P0, &[0, 1]);
        t.answer_targets(P0, &[obj(relic)]);
        t.cast(P0, card).x(2).go();
        // The commander leaves before the spell resolves: both modes still happen.
        move_to(&mut t, dragon, Zone::Hand(P0));
        t.resolve_all();
        assert!(t.in_graveyard(P1, "Grizzly Bears"), "commander {commander}");
        assert_eq!(t.on_battlefield(relic), !commander, "commander {commander}");
    }
}

// ---------------------------------------------------------------- Dawnglow Infusion

#[test]
fn dawnglow_infusion_x_doesnt_depend_on_how_the_hybrid_was_paid() {
    cr!("107.4e", "601.2h");
    ruling!(
        "Dawnglow Infusion",
        "If you end up spending {W}{W}{R}{R}{G} to pay the cost, X is still 4 and you’ve spent both {G} and {W}. You’ll gain a total of 8 life."
    );
    supported("Dawnglow Infusion");
    // (pool, life gained)
    for (pool, gained) in [
        (
            vec![(ManaType::W, 2), (ManaType::R, 2), (ManaType::G, 1)],
            8,
        ),
        (vec![(ManaType::R, 4), (ManaType::G, 1)], 4),
        (vec![(ManaType::R, 4), (ManaType::W, 1)], 4),
    ] {
        let mut t = TestGame::new(2);
        let card = t.hand(P0, "Dawnglow Infusion");
        for (ty, n) in &pool {
            mana(&mut t, P0, *ty, *n);
        }
        t.cast(P0, card).x(4).go();
        assert_eq!(t.g.player(P0).mana_pool.total(), 0);
        t.resolve_all();
        assert_eq!(t.life(P0), 20 + gained, "{pool:?}");
    }
}

// ---------------------------------------------------------------- Vish Kal, Deity of Scars

#[test]
fn vish_kal_uses_the_sacrificed_creatures_last_known_power() {
    cr!("608.2h", "118.3", "113.7a");
    ruling!(
        "Vish Kal, Blood Arbiter",
        "Use the sacrificed creature's power as it last existed on the battlefield"
    );
    supported("Vish Kal, Blood Arbiter");
    let mut t = TestGame::new(2);
    let vish = t.battlefield(P0, "Vish Kal, Blood Arbiter");
    let bears = t.battlefield(P0, "Grizzly Bears");
    put_counters(&mut t, bears, counters::PLUS1, 2);
    let giant = t.hand(P0, "Giant Growth");
    mana(&mut t, P0, ManaType::G, 1);
    t.cast(P0, giant).target(bears).go();
    t.resolve_all();
    assert_eq!(t.pt(bears).0, 7);
    t.answer_choose(P0, &[obj(bears)]);
    activate_containing(&mut t, P0, vish, "Sacrifice a creature").unwrap();
    t.resolve_all();
    assert_eq!(t.counters(vish, counters::PLUS1), 7);
}

#[test]
fn deity_of_scars_can_regenerate_at_any_time() {
    cr!("701.19a", "602.1");
    ruling!(
        "Deity of Scars",
        "Deity of Scars doesn't need to be in danger of being destroyed."
    );
    supported("Deity of Scars");
    let mut t = TestGame::new(2);
    let deity = t.enter(P0, "Deity of Scars");
    t.settle();
    assert_eq!(t.counters(deity, counters::MINUS1), 2);
    mana(&mut t, P0, ManaType::B, 1);
    activate_containing(&mut t, P0, deity, "Regenerate").unwrap();
    t.resolve_all();
    assert_eq!(t.counters(deity, counters::MINUS1), 1);
    assert_eq!(t.pt(deity), (6, 6));
    // The shield it made replaces a later destruction.
    destroy(&mut t, deity);
    assert!(t.on_battlefield(deity));
    assert!(t.obj_now(deity).tapped);
}

#[test]
fn leviathan_untapped_by_an_effect_needs_no_islands() {
    cr!("502.3", "701.26b");
    ruling!(
        "Leviathan",
        "You don't have to pay the untap cost if it is untapped outside the upkeep phase or is untapped by an effect."
    );
    supported("Leviathan");
    supported("Twiddle");
    let mut t = TestGame::new(2);
    let lev = t.battlefield(P0, "Leviathan");
    let islands = t.lands(P0, "Island", 2);
    t.g.objects[lev.0 as usize].tapped = true;
    let twiddle = t.hand(P0, "Twiddle");
    mana(&mut t, P0, ManaType::U, 1);
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.cast(P0, twiddle).target(lev).go();
    t.resolve_all();
    assert!(!t.obj_now(lev).tapped);
    assert!(islands.iter().all(|i| t.on_battlefield(*i)));
    // Control: the real Leviathan (8/8 trample) doesn't untap in its controller's untap
    // step unless the Islands are sacrificed.
    assert!(crate::r_s06_common::has_kw(
        &t,
        lev,
        mtg_engine::keywords::KeywordKind::Trample
    ));
    let now = t.g.current(lev);
    t.g.objects[now.0 as usize].tapped = true;
    t.answer_yes(P0, false);
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Draw);
    assert!(t.obj_now(lev).tapped);
    assert!(islands.iter().all(|i| t.on_battlefield(*i)));
}

// ---------------------------------------------------------------- Hour of Eternity

#[test]
fn hour_of_eternity_tokens_keep_the_mana_cost() {
    cr!("707.9b", "202.3");
    ruling!(
        "Hour of Eternity",
        "Unlike the tokens created by an eternalize ability, the tokens have the mana cost and thus mana value of the cards they're copying."
    );
    supported("Hour of Eternity");
    let mut t = TestGame::new(2);
    let angel = t.graveyard(P0, "Serra Angel");
    let card = t.hand(P0, "Hour of Eternity");
    mana(&mut t, P0, ManaType::U, 3);
    mana(&mut t, P0, ManaType::C, 2);
    t.cast(P0, card).x(1).target(angel).go();
    t.resolve_all();
    let toks = tokens(&t, P0);
    assert_eq!(toks.len(), 1);
    let tok = toks[0];
    assert_eq!(t.pt(tok), (4, 4));
    assert!(t.obj_now(tok).chars.has_subtype("Zombie"));
    assert!(!t.obj_now(tok).chars.has_subtype("Angel"));
    assert_eq!(mana_value(&t, tok), 5);
    assert_eq!(
        format!("{}", t.obj_now(tok).chars.mana_cost.clone().unwrap()),
        "{3}{W}{W}"
    );
}
