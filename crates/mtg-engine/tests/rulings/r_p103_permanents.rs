//! Rulings batch P103 — permanents that mill or exile from libraries: Glacierwood Siege's
//! chosen ability (CR 614.12, 305.1, 707.2), Ashiok, Dream Render (CR 701.23), Founding
//! the Third Path's free cast and copy (CR 601.2b, 118.9, 707.12), Relic Golem (CR
//! 506.4, 509.1), Dread Summons (CR 614.6, 603.6), Crumbling Sanctuary (CR 614.1a),
//! Psychic Surgery (CR 701.24), Mind Grind's X (CR 107.3b), and Undead Alchemist (CR
//! 614.5).

use crate::r_p103_common::*;
use crate::r_s01_common::{stack_library, supported, with_subtype};
use crate::r_s02_common::{can_activate, can_cast, can_play_land};
use crate::r_s07_common::cast_methods;
use crate::r_s25_common::cast_new;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

// --- Glacierwood Siege ------------------------------------------------------------------

/// Glacierwood Siege enters under P0's control with `mode` chosen (0: Temur, 1: Sultai).
fn siege(t: &mut TestGame, mode: usize) -> ObjectId {
    supported("Glacierwood Siege");
    t.answer(P0, DecisionKind::Option, Answer::Index(mode));
    let s = t.enter(P0, "Glacierwood Siege");
    t.g.flush_events();
    t.settle();
    s
}

#[test]
fn glacierwood_siege_sultai_plays_lands_only_as_land_plays() {
    cr!("305.1", "305.2", "505.6b");
    ruling!(
        "Glacierwood Siege",
        "Glacierwood Siege’s Sultai ability doesn’t change the times when you can play those land cards. You can still play only one land per turn, and only during your main phase when you have priority and the stack is empty."
    );
    let mut t = TestGame::new(2);
    siege(&mut t, 1);
    let a = t.graveyard(P0, "Forest");
    let b = t.graveyard(P0, "Island");
    assert!(can_play_land(&mut t, P0, a));
    // Not with a spell on the stack.
    let shock = cast_new(&mut t, P1, "Shock", &[pl(P1)]);
    assert!(on_stack(&t, shock));
    assert!(!can_play_land(&mut t, P0, a));
    t.resolve_all();
    // Not in combat.
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!can_play_land(&mut t, P0, a));
    t.set_step(P0, Step::PostcombatMain);
    t.play_land(P0, a).unwrap();
    assert!(t.on_battlefield(a));
    // Only one land per turn.
    assert!(!can_play_land(&mut t, P0, b));
}

#[test]
fn glacierwood_siege_sultai_doesnt_allow_activating_from_the_graveyard() {
    cr!("602.2", "702.29a");
    ruling!(
        "Glacierwood Siege",
        "Glacierwood Siege’s Sultai ability doesn’t allow you to activate abilities (such as cycling) of land cards in your graveyard."
    );
    let mut t = TestGame::new(2);
    siege(&mut t, 1);
    let moor = t.graveyard(P0, "Barren Moor");
    t.lands(P0, "Swamp", 2);
    assert!(can_play_land(&mut t, P0, moor));
    assert!(!can_activate(&mut t, P0, moor));
    // In the hand, it could be cycled.
    let moor2 = t.hand(P0, "Barren Moor");
    assert!(can_activate(&mut t, P0, moor2));
}

#[test]
fn glacierwood_siege_without_a_choice_has_neither_ability() {
    cr!("614.12", "707.2");
    ruling!(
        "Glacierwood Siege",
        "If you somehow control Glacierwood Siege and no choice was made for it (perhaps because another permanent on the battlefield became a copy of it), it has neither of the two abilities."
    );
    supported("Glacierwood Siege");
    let mut t = TestGame::new(2);
    // Put onto the battlefield with no choice made.
    t.battlefield(P0, "Glacierwood Siege");
    let forest = t.graveyard(P0, "Forest");
    assert!(!can_play_land(&mut t, P0, forest));
    cast_new(&mut t, P0, "Opt", &[]);
    t.settle();
    assert_eq!(t.stack_len(), 1, "no Temur trigger");
    t.resolve_all();
    // With Temur chosen, the same spell triggers it (and Sultai's lands stay put).
    let mut t = TestGame::new(2);
    siege(&mut t, 0);
    let forest = t.graveyard(P0, "Forest");
    assert!(!can_play_land(&mut t, P0, forest));
    t.answer_targets(P0, &[pl(P1)]);
    cast_new(&mut t, P0, "Opt", &[]);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    // With Sultai chosen, the land can be played.
    let mut t = TestGame::new(2);
    siege(&mut t, 1);
    let forest = t.graveyard(P0, "Forest");
    assert!(can_play_land(&mut t, P0, forest));
}

// --- Ashiok, Dream Render ---------------------------------------------------------------

#[test]
fn ashiok_minus_one_exiles_the_cards_it_milled() {
    cr!("608.2c", "701.17a");
    ruling!(
        "Ashiok, Dream Render",
        "If Ashiok’s last ability targets an opponent, the cards put into their graveyard will be among the cards exiled by the ability."
    );
    supported("Ashiok, Dream Render");
    let mut t = TestGame::new(2);
    let ashiok = t.battlefield(P0, "Ashiok, Dream Render");
    let old = t.graveyard(P1, "Grizzly Bears");
    let top = stack_library(&mut t, P1, &["Island", "Forest", "Swamp", "Mountain"]);
    t.activate(P0, ashiok, 0, &[pl(P1)]).unwrap();
    t.resolve_all();
    assert_eq!(t.graveyard_size(P1), 0);
    for c in top.iter().chain([&old]) {
        assert_eq!(t.zone(*c), Zone::Exile);
    }
}

#[test]
fn ashiok_stops_only_opponents_searching_their_own_libraries() {
    cr!("701.23a", "101.2");
    ruling!(
        "Ashiok, Dream Render",
        "Spells and abilities your opponents control may cause other players to search their libraries, and spells and abilities you control may cause your opponents to search their libraries."
    );
    supported("Ashiok, Dream Render");
    // Path to Exile: "Exile target creature. Its controller may search their library for a
    // basic land card, put that card onto the battlefield tapped, then shuffle."
    // (caster, the creature's controller, whether that player finds the land)
    for (caster, victim, finds) in [(P0, P1, true), (P1, P2, true), (P1, P1, false)] {
        let mut t = TestGame::new(3);
        t.battlefield(P0, "Ashiok, Dream Render");
        let bears = t.battlefield(victim, "Grizzly Bears");
        let plains = t.library_top(victim, "Plains");
        t.answer_yes(victim, true);
        t.answer_choose(victim, &[obj(plains)]);
        cast_new(&mut t, caster, "Path to Exile", &[obj(bears)]);
        t.resolve_all();
        assert_eq!(t.zone(bears), Zone::Exile);
        assert_eq!(t.on_battlefield(plains), finds, "{caster:?} -> {victim:?}");
    }
}

// --- Founding the Third Path --------------------------------------------------------

/// Founding the Third Path enters under P0's control on chapter `chapter` (read ahead).
fn founding(t: &mut TestGame, chapter: i64) -> ObjectId {
    supported("Founding the Third Path");
    t.answer(P0, DecisionKind::Number, Answer::Number(chapter));
    let s = t.enter(P0, "Founding the Third Path");
    t.g.flush_events();
    s
}

#[test]
fn founding_the_third_path_free_cast_pays_additional_costs() {
    cr!("118.9", "118.9a", "118.8", "601.2b");
    ruling!(
        "Founding the Third Path",
        "If you cast a card “without paying its mana cost,” you can’t choose to cast it for any alternative costs. You can, however, pay additional costs. If the card has any mandatory additional costs, you must pay those to cast the card."
    );
    // Optional additional cost: Burst Lightning kicked ({4}) deals 4.
    let mut t = TestGame::new(2);
    t.lands(P0, "Wastes", 4);
    let burst = t.hand(P0, "Burst Lightning");
    founding(&mut t, 1);
    t.answer_choose(P0, &[obj(burst)]);
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.answer_targets(P0, &[pl(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    // Mandatory additional cost: Village Rites needs a creature to sacrifice.
    let mut t = TestGame::new(2);
    let rites = t.hand(P0, "Village Rites");
    founding(&mut t, 1);
    t.answer_choose(P0, &[obj(rites)]);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.zone(rites), Zone::Hand(P0), "no creature to sacrifice");
    assert_eq!(t.hand_size(P0), hand);
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let rites = t.hand(P0, "Village Rites");
    founding(&mut t, 1);
    t.answer_choose(P0, &[obj(rites)]);
    t.answer_choose(P0, &[obj(bears)]);
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Graveyard(P0));
    assert_eq!(t.zone(rites), Zone::Graveyard(P0));
    // No alternative cost: Daze (mana value 2) is cast with no cost choice, so no Island
    // is returned.
    let mut t = TestGame::new(2);
    let island = t.battlefield(P0, "Island");
    let bolt = cast_new(&mut t, P1, "Shock", &[pl(P0)]);
    let daze = t.hand(P0, "Daze");
    founding(&mut t, 1);
    t.answer_choose(P0, &[obj(daze)]);
    t.answer_targets(P0, &[obj(bolt)]);
    let from = t.asked().len();
    t.resolve();
    assert!(on_stack(&t, t.g.current(daze)));
    assert!(t.on_battlefield(island));
    assert!(!t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseCastingMethod { .. })));
}

#[test]
fn founding_the_third_path_copy_is_cast_then_or_never_and_paid_for() {
    cr!("707.12", "608.2g", "601.2h");
    ruling!(
        "Founding the Third Path",
        "You must pay all costs for the copy of a spell you cast as Founding the Third Path’s last chapter ability resolves. If you choose to cast it, you must cast as the ability resolves. You can’t wait and cast it at a later time."
    );
    // With {R} available: the copy of Lightning Bolt is cast and paid for.
    let mut t = TestGame::new(2);
    let mountain = t.battlefield(P0, "Mountain");
    let bolt = t.graveyard(P0, "Lightning Bolt");
    t.answer_targets(P0, &[obj(bolt)]);
    founding(&mut t, 3);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[pl(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert!(t.obj_now(mountain).tapped);
    assert_eq!(t.zone(bolt), Zone::Exile);
    // Without mana: it can't be cast, and it's gone.
    let mut t = TestGame::new(2);
    let bolt = t.graveyard(P0, "Lightning Bolt");
    t.answer_targets(P0, &[obj(bolt)]);
    founding(&mut t, 3);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[pl(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.stack_len(), 0);
    // Mana arriving later doesn't help: there's nothing left to cast.
    t.lands(P0, "Mountain", 1);
    t.g.turn.priority = Some(P0);
    let acts = t.g.legal_actions(P0);
    assert!(!acts
        .iter()
        .any(|a| matches!(a, mtg_engine::decision::Action::Cast { .. })));
}

// --- Relic Golem ------------------------------------------------------------------------

#[test]
fn relic_golem_stays_in_combat_when_the_graveyard_shrinks() {
    cr!("506.4", "508.1c", "509.1b");
    ruling!(
        "Relic Golem",
        "Once Relic Golem has attacked, causing its first ability to apply by removing cards from graveyards won’t cause Relic Golem to be removed from combat."
    );
    ruling!(
        "Relic Golem",
        "Similarly, once Relic Golem has blocked an attacking creature, causing its first ability to apply won’t cause that attacking creature to become unblocked or remove Relic Golem from combat."
    );
    supported("Relic Golem");
    // Attacking.
    let mut t = TestGame::new(2);
    let golem = t.battlefield(P0, "Relic Golem");
    for _ in 0..8 {
        t.graveyard(P1, "Grizzly Bears");
    }
    crate::r_s01_common::attack_with(&mut t, &[(golem, pl(P1))]);
    crate::r_p189_common::exile_graveyard(&mut t, P1);
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 14);
    // Blocking.
    let mut t = TestGame::new(2);
    let attacker = t.battlefield(P0, "Hill Giant");
    let golem = t.battlefield(P1, "Relic Golem");
    for _ in 0..8 {
        t.graveyard(P0, "Grizzly Bears");
    }
    crate::r_s01_common::attack_with(&mut t, &[(attacker, pl(P1))]);
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(golem, attacker)]),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    crate::r_p189_common::exile_graveyard(&mut t, P0);
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20, "still blocked");
    assert_eq!(t.zone(attacker), Zone::Graveyard(P0));
    assert!(t.on_battlefield(golem));
}

// --- Dread Summons ----------------------------------------------------------------------

#[test]
fn dread_summons_counts_cards_put_into_graveyards() {
    cr!("614.6", "603.6a", "701.17c");
    ruling!(
        "Dread Summons",
        "If a creature card has an ability that replaces going to the graveyard with moving somewhere else \"instead,\" that card won't count toward the number of Zombies you get. Conversely, creature cards with a triggered ability that removes them from the graveyard when they're put there from anywhere (or your library) will count toward that number."
    );
    supported("Dread Summons");
    let mut t = TestGame::new(2);
    // Blightsteel Colossus is shuffled into its owner's library instead; Narcomoeba is
    // put into the graveyard (and may then come back).
    let top = stack_library(&mut t, P1, &["Blightsteel Colossus", "Narcomoeba"]);
    t.lands(P0, "Wastes", 2);
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    cast_new(&mut t, P0, "Dread Summons", &[]);
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Zombie").len(), 1);
    assert_eq!(t.zone(top[0]), Zone::Library(P1));
    assert_ne!(t.zone(top[1]), Zone::Library(P1));
}

// --- Crumbling Sanctuary ----------------------------------------------------------------

#[test]
fn crumbling_sanctuary_with_an_empty_library_makes_damage_do_nothing() {
    cr!("614.1a", "120.3");
    ruling!(
        "Crumbling Sanctuary",
        "If you have no cards in your library, then damage done to you will have no effect since it will get changed into a removal that can’t happen."
    );
    supported("Crumbling Sanctuary");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Crumbling Sanctuary");
    empty_library(&mut t, P1);
    cast_new(&mut t, P0, "Lightning Bolt", &[pl(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    assert!(!t.has_lost(P1));
    // With cards in the library, they're exiled instead.
    library_of(&mut t, P1, "Island", 5);
    cast_new(&mut t, P0, "Lightning Bolt", &[pl(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.library_size(P1), 2);
}

// --- Psychic Surgery --------------------------------------------------------------------

#[test]
fn psychic_surgery_triggers_on_a_one_card_shuffle() {
    cr!("701.24a", "603.2");
    ruling!(
        "Psychic Surgery",
        "If your opponent’s library has one card in it and they are instructed to shuffle that library, Psychic Surgery will trigger. You may’ exile the one card in that library."
    );
    supported("Psychic Surgery");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Psychic Surgery");
    let only = library_of(&mut t, P1, "Grizzly Bears", 1);
    t.g.shuffle_library(P1);
    t.g.flush_events();
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(only[0])]);
    t.resolve_all();
    assert_eq!(t.zone(only[0]), Zone::Exile);
}

#[test]
fn psychic_surgery_triggers_twice_for_green_suns_zenith() {
    cr!("701.24a", "603.2c");
    ruling!(
        "Psychic Surgery",
        "In most cases, Green Sun’s Zenith instructs you to shuffle your library twice. Although shuffling that library only once is an acceptable shortcut, Psychic Surgery’s ability will still trigger twice."
    );
    supported("Psychic Surgery");
    supported("Green Sun's Zenith");
    let mut t = TestGame::new(2);
    let surgery = t.battlefield(P0, "Psychic Surgery");
    let bears = t.library_top(P1, "Grizzly Bears");
    t.lands(P1, "Forest", 3);
    let gsz = t.hand(P1, "Green Sun's Zenith");
    t.set_step(P1, Step::PrecombatMain);
    t.answer_choose(P1, &[obj(bears)]);
    t.g.turn.priority = Some(P1);
    t.cast(P1, gsz).x(2).go();
    t.resolve();
    assert!(t.on_battlefield(bears));
    assert_eq!(crate::r_p189_common::triggers_of(&t, surgery), 2);
}

// --- Mind Grind ---------------------------------------------------------------------

#[test]
fn mind_grind_cant_be_cast_without_paying_its_mana_cost() {
    cr!("107.3b", "118.9");
    ruling!(
        "Mind Grind",
        "If another spell or ability instructs you to cast Mind Grind “without paying its mana cost,” you won’t be able to. You must pick 0 as the value of X in the mana cost of a spell being cast “without paying its mana cost,” but the X in Mind Grind’s mana cost can’t be 0."
    );
    supported("Mind Grind");
    let mut t = TestGame::new(2);
    // Omniscience: "You may cast spells from your hand without paying their mana costs."
    t.battlefield(P0, "Omniscience");
    let grind = t.hand(P0, "Mind Grind");
    let shock = t.hand(P0, "Shock");
    assert!(cast_methods(&mut t, P0, grind).is_empty());
    assert!(!cast_methods(&mut t, P0, shock).is_empty());
    // With mana, it's cast normally.
    t.lands(P0, "Island", 1);
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Wastes", 1);
    assert!(can_cast(&mut t, P0, grind, CastMethod::Normal));
}

// --- Undead Alchemist -------------------------------------------------------------------

#[test]
fn two_undead_alchemists_replace_the_damage_once() {
    cr!("614.5", "616.1");
    ruling!(
        "Undead Alchemist",
        "If you control multiple Undead Alchemists, the multiple replacement abilities will have no added effect. Combat damage dealt to a player by a Zombie you control will be replaced only once with cards being put into that player's graveyard."
    );
    supported("Undead Alchemist");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Undead Alchemist");
    t.battlefield(P0, "Undead Alchemist");
    let zombie = t.battlefield(P0, "Walking Corpse");
    library_of(&mut t, P1, "Island", 10);
    t.attack(&[(zombie, pl(P1))], &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.graveyard_size(P1), 2);
}
