//! Rulings batch P008 — burn that can hit planeswalkers: the rest. Shrines (CR 205.3),
//! delirium's card types (CR 205.2a), forced vs. chosen discards, Soulfire Eruption's
//! resolution (CR 608.2, 603.3) and play permissions (CR 400.7, 305.2), Leyline of
//! Lightning with bloodthirst, doubled doubling, metalcraft "can't block" applying to
//! creatures that arrive later (CR 611.2c), sweepers (CR 115.10), intervening-if upkeep
//! triggers (CR 603.4), and the order of simultaneous triggers (CR 603.3b).

use crate::r_p007_common::*;
use crate::r_p008_common::*;
use crate::r_s01_common::supported;
use crate::r_s02_common::{can_activate, can_cast, can_play_land};
use crate::r_s05_common::move_to;
use crate::r_s25_common::{cast_new, lands_for_cost};
use mtg_engine::decision::Answer;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const NORMAL: CastMethod = CastMethod::Normal;

/// Moves P0's turn back to its draw step, so the precombat main phase starts next.
fn before_first_main(t: &mut TestGame) {
    t.set_step(P0, Step::Draw);
}

#[test]
fn sanctums_count_only_shrines() {
    cr!("205.3h", "109.2");
    ruling!(
        "Sanctum of Calm Waters",
        "Shrines count only enchantments with the subtype Shrine. Other cards with “shrine” in their name (such as Jungle Shrine, Luxa River Shrine, and Nantuko Shrine) don't count."
    );
    ruling!(
        "Sanctum of Fruitful Harvest",
        "Shrines count only enchantments with the subtype Shrine. Other cards with “shrine” in their name (such as Jungle Shrine, Luxa River Shrine, and Nantuko Shrine) don't count."
    );
    ruling!(
        "Sanctum of Shattered Heights",
        "Shrines count only enchantments with the subtype Shrine. Other cards with “shrine” in their name (such as Jungle Shrine, Luxa River Shrine, and Nantuko Shrine) don't count."
    );
    ruling!(
        "Sanctum of Stone Fangs",
        "Shrines count only enchantments with the subtype Shrine. Other cards with “shrine” in their name (such as Jungle Shrine, Luxa River Shrine, and Nantuko Shrine) don't count."
    );
    ruling!(
        "Sanctum of Tranquil Light",
        "Shrines count only enchantments with the subtype Shrine. Other cards with “shrine” in their name (such as Jungle Shrine, Luxa River Shrine, and Nantuko Shrine) don't count."
    );
    // Each with a Jungle Shrine (and a Shrine-less enchantment) beside it: X is 1.
    let setup = |name: &str| {
        supported(name);
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Jungle Shrine");
        t.battlefield(P0, "Glorious Anthem");
        let s = t.battlefield(P0, name);
        (t, s)
    };
    // Stone Fangs: P1 loses 1, P0 gains 1.
    let (mut t, _) = setup("Sanctum of Stone Fangs");
    before_first_main(&mut t);
    t.advance_to(P0, Step::PrecombatMain);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (21, 19));
    // Calm Waters: draw 1, then discard 1.
    let (mut t, _) = setup("Sanctum of Calm Waters");
    before_first_main(&mut t);
    let library = t.library_size(P0);
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::PrecombatMain);
    t.resolve_all();
    assert_eq!(t.library_size(P0), library - 1, "drew 1");
    // Fruitful Harvest: one mana.
    let (mut t, _) = setup("Sanctum of Fruitful Harvest");
    before_first_main(&mut t);
    t.advance_to(P0, Step::PrecombatMain);
    t.resolve_all();
    assert_eq!(t.g.players[0].mana_pool.total(), 1);
    // Shattered Heights: 1 damage.
    let (mut t, s) = setup("Sanctum of Shattered Heights");
    let wurm = t.battlefield(P1, "Craw Wurm");
    let forest = t.hand(P0, "Forest");
    t.lands(P0, "Wastes", 1);
    t.answer_choose(P0, &[obj(forest)]);
    t.activate(P0, s, 0, &[obj(wurm)])
        .expect("Sanctum of Shattered Heights");
    t.resolve_all();
    assert_eq!(dmg(&t, wurm), 1);
    // Tranquil Light: {5}{W} reduced by 1 only.
    let (mut t, s) = setup("Sanctum of Tranquil Light");
    t.battlefield(P1, "Craw Wurm");
    let shrine = t.named_on_battlefield("Jungle Shrine")[0];
    t.g.tap(shrine);
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Wastes", 3);
    assert!(!can_activate(&mut t, P0, s), "{{4}}{{W}} needed");
    t.lands(P0, "Wastes", 1);
    assert!(can_activate(&mut t, P0, s));
}

#[test]
fn unholy_heat_counts_card_types_not_supertypes_or_subtypes() {
    cr!("205.2a", "205.4a", "205.3a", "207.2c");
    ruling!(
        "Unholy Heat",
        "The card types that can appear in a graveyard are artifact, battle, creature, enchantment, instant, kindred, land, planeswalker, and sorcery. Legendary, basic, and snow are supertypes, not card types; Kavu and Equipment are subtypes, not card types."
    );
    // Kindred, instant, land and battle: four types, 6 damage.
    let mut t = TestGame::new(2);
    let target = t.battlefield(P1, "Indomitable Ancients");
    t.graveyard(P0, "Tarfire");
    t.graveyard(P0, "Forest");
    t.graveyard(P0, "Invasion of Ikoria // Zilortha, Apex of Ikoria");
    cast_x(&mut t, P0, "Unholy Heat", NORMAL, None, &[obj(target)]);
    t.resolve_all();
    // (Unholy Heat itself goes to the graveyard only after it resolves.)
    assert_eq!(dmg(&t, target), 6);
    // Legendary, snow and basic lands and creatures: only land and creature, 2 damage.
    let mut t = TestGame::new(2);
    let target = t.battlefield(P1, "Indomitable Ancients");
    t.graveyard(P0, "Snow-Covered Mountain");
    t.graveyard(P0, "Isamaru, Hound of Konda");
    t.graveyard(P0, "Kavu Titan");
    t.graveyard(P0, "Forest");
    cast_x(&mut t, P0, "Unholy Heat", NORMAL, None, &[obj(target)]);
    t.resolve_all();
    assert_eq!(dmg(&t, target), 2);
}

#[test]
fn lands_edge_only_counts_its_own_discards() {
    cr!("701.9a", "602.1");
    ruling!(
        "Land's Edge",
        "The damage done when you discard a land only applies to lands which are discarded by choice using the Land's Edge ability. It does not work on forced discards of any type."
    );
    supported("Land's Edge");
    let mut t = TestGame::new(2);
    let edge = t.battlefield(P0, "Land's Edge");
    t.hand(P0, "Forest");
    t.hand(P0, "Mountain");
    t.set_step(P1, Step::PrecombatMain);
    cast_new(&mut t, P1, "Mind Rot", &[pl(P0)]);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 0);
    assert_eq!(t.life(P1), 20, "a forced discard deals no damage");
    // Its own ability: discarding a land deals 2.
    let forest = t.hand(P0, "Forest");
    t.answer_choose(P0, &[obj(forest)]);
    t.activate(P0, edge, 0, &[pl(P1)]).expect("Land's Edge");
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

#[test]
fn soulfire_eruption_resolves_without_interruption() {
    cr!("608.2", "603.3", "603.2");
    ruling!(
        "Soulfire Eruption",
        "Nothing can happen in between the damage being dealt to each target, and no player can choose to take actions. Any abilities that trigger while Soulfire Eruption is resolving go on the stack after Soulfire Eruption finishes resolving."
    );
    // P1's Sliver Construct with Spiteful Sliver: "Whenever this creature is dealt damage,
    // it deals that much damage to target player or planeswalker."
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Spiteful Sliver");
    let sliver = t.battlefield(P1, "Sliver Construct");
    t.library_top(P0, "Shock");
    t.library_top(P0, "Lightning Bolt");
    t.answer_targets(P1, &[pl(P0)]);
    t.answer_targets(P0, &[obj(sliver), pl(P1)]);
    lands_for_cost(&mut t, P0, "Soulfire Eruption");
    let card = t.hand(P0, "Soulfire Eruption");
    t.cast(P0, card).go();
    let from = t.g.turn_events.len();
    t.g.resolve_top();
    // Both targets were dealt damage before the trigger went on the stack.
    let dealt = damage_events(&t, from);
    assert_eq!(dealt.len(), 2);
    assert_eq!(t.life(P1), 19, "Lightning Bolt (1), Shock (1)");
    t.settle();
    assert_eq!(t.stack_len(), 1, "Spiteful Sliver's trigger, afterwards");
    t.resolve_all();
    assert_eq!(t.life(P0), 19);
}

#[test]
fn soulfire_eruption_exiled_cards_keep_their_timing_and_can_be_played_once() {
    cr!("400.7", "305.2", "307.1");
    ruling!(
        "Soulfire Eruption",
        "Playing an exiled card causes it to leave exile. Soulfire Eruption won't let you play it multiple times, even if the card later returns to exile somehow."
    );
    ruling!(
        "Soulfire Eruption",
        "Soulfire Eruption doesn't change when you can play the exiled cards. For example, if you exile a sorcery card, you can cast it only during your main phase. If you exile a land card, you can play it only during your main phase and only if you have an available land play remaining."
    );
    ruling!(
        "Nahiri's Warcrafting",
        "Playing the exiled card follows the normal rules for playing that card. You must pay its cost, and you must follow all applicable timing rules. For example, if it's a creature card, you can cast it by paying its mana cost only during your main phase while the stack is empty."
    );
    // Soulfire Eruption at P1 and a creature: exiles a Forest and a sorcery.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    let lava = t.library_top(P0, "Lava Axe");
    let forest = t.library_top(P0, "Forest");
    t.answer_targets(P0, &[pl(P1), obj(wurm)]);
    lands_for_cost(&mut t, P0, "Soulfire Eruption");
    let card = t.hand(P0, "Soulfire Eruption");
    t.cast(P0, card).go();
    t.resolve_all();
    let (lava, forest) = (t.g.current(lava), t.g.current(forest));
    assert_eq!((t.zone(lava), t.zone(forest)), (Zone::Exile, Zone::Exile));
    t.lands(P0, "Mountain", 5);
    assert!(can_cast(&mut t, P0, lava, NORMAL));
    t.g.players[0].lands_played_this_turn = 1;
    assert!(!can_play_land(&mut t, P0, forest), "no land play left");
    t.g.players[0].lands_played_this_turn = 0;
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!can_cast(&mut t, P0, lava, NORMAL), "a sorcery");
    assert!(!can_play_land(&mut t, P0, forest), "not a main phase");
    t.set_step(P0, Step::PostcombatMain);
    t.play_land(P0, forest).expect("play the Forest");
    let played = t.g.current(forest);
    assert!(t.on_battlefield(played));
    // Back to exile: a new object, which can't be played.
    let back = move_to(&mut t, played, Zone::Exile).expect("exiled");
    t.g.players[0].lands_played_this_turn = 0;
    assert!(!can_play_land(&mut t, P0, back));

    // Nahiri's Warcrafting at Grizzly Bears: 3 excess damage; a creature card exiled.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.library_top(P0, "Forest");
    let giant = t.library_top(P0, "Hill Giant");
    t.library_top(P0, "Forest");
    t.answer_choose_moved(P0, &[obj(giant)]);
    cast_x(&mut t, P0, "Nahiri's Warcrafting", NORMAL, None, &[obj(bears)]);
    t.resolve_all();
    let giant = t.g.current(giant);
    assert_eq!(t.zone(giant), Zone::Exile);
    assert!(!can_cast(&mut t, P0, giant, NORMAL), "its cost must be paid");
    t.lands(P0, "Mountain", 4);
    assert!(can_cast(&mut t, P0, giant, NORMAL));
    cast_x(&mut t, P0, "Shock", NORMAL, None, &[pl(P1)]);
    assert!(!can_cast(&mut t, P0, giant, NORMAL), "the stack isn't empty");
    t.resolve_all();
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!can_cast(&mut t, P0, giant, NORMAL), "not a main phase");
}

#[test]
fn leyline_of_lightning_damage_comes_before_the_bloodthirst_creature() {
    cr!("702.54a", "603.3", "601.2i");
    ruling!(
        "Leyline of Lightning",
        "If you cast a creature with bloodthirst while Leyline of Lightning is on the battlefield, you may use the Leyline's ability to deal damage to an opponent. If you do, that damage is dealt before the creature spell resolves, and the creature will enter with counters from its bloodthirst ability."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Leyline of Lightning");
    t.lands(P0, "Wastes", 1);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[pl(P1)]);
    let m = cast_x(&mut t, P0, "Gorehorn Minotaurs", NORMAL, None, &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    let m = t.g.current(m);
    assert_eq!(t.counters(m, "+1/+1"), 2);
}

#[test]
fn two_insults_quadruple_damage() {
    cr!("616.1", "614.1a");
    ruling!(
        "Insult // Injury",
        "If you cast two Insults in one turn, damage dealt by sources you control this turn will be multiplied by 4. If you cast three Insults, it will be multiplied by 8, and so on. How rude."
    );
    for (n, life) in [(2, 12), (3, 4)] {
        let mut t = TestGame::new(2);
        for _ in 0..n {
            cast_paying(
                &mut t,
                P0,
                "Insult // Injury",
                &["Wastes", "Wastes", "Mountain"],
                CastMethod::Half(0),
                &[],
            );
            t.resolve_all();
        }
        let src = t.battlefield(P0, "Raging Goblin");
        hit_player(&mut t, src, P1, 2);
        assert_eq!(t.life(P1), life, "{n} Insults");
    }
}

#[test]
fn concussive_bolt_affects_creatures_that_arrive_later() {
    cr!("611.2c", "608.2c");
    ruling!(
        "Concussive Bolt",
        "If you controlled three or more artifacts as Concussive Bolt resolved, the targeted player won’t be able to block with any creatures this turn. That is, the set of affected creatures isn’t locked in as the spell resolves."
    );
    let mut t = TestGame::new(2);
    for _ in 0..3 {
        t.battlefield(P0, "Ornithopter");
    }
    let before = t.battlefield(P1, "Grizzly Bears");
    cast_x(&mut t, P0, "Concussive Bolt", NORMAL, None, &[pl(P1)]);
    t.resolve_all();
    let later = t.enter(P1, "Hill Giant");
    t.settle();
    assert!(!t.g.can_block_at_all(before));
    assert!(!t.g.can_block_at_all(later));
}

#[test]
fn hour_of_devastation_and_magmaquake_sweep_without_targeting() {
    cr!("608.2", "115.10", "613.1f");
    ruling!(
        "Hour of Devastation",
        "Once Hour of Devastation begins to resolve, no player may take actions until it’s done. Notably, players can’t try to give a creature indestructible again to save it from the damage."
    );
    ruling!(
        "Magmaquake",
        "Magmaquake doesn't target any creature or planeswalker. For example, it will deal damage to a creature without flying that has hexproof."
    );
    let mut t = TestGame::new(2);
    let myr = t.battlefield(P1, "Darksteel Myr");
    let chandra = pw(&mut t, P1, "Chandra, Torch of Defiance", 6);
    let bolas = pw(&mut t, P1, "Nicol Bolas, Planeswalker", 6);
    cast_x(&mut t, P0, "Hour of Devastation", NORMAL, None, &[]);
    t.resolve_all();
    assert!(!t.on_battlefield(myr), "indestructible lost first");
    assert_eq!(t.counters(chandra, "loyalty"), 1);
    assert_eq!(t.counters(bolas, "loyalty"), 6);
    // Magmaquake, X = 2.
    supported("Magmaquake");
    let mut t = TestGame::new(2);
    let bogle = t.battlefield(P1, "Slippery Bogle");
    let flier = t.battlefield(P1, "Serra Angel");
    let theirs = pw(&mut t, P1, "Chandra, Torch of Defiance", 4);
    cast_x(&mut t, P0, "Magmaquake", NORMAL, Some(2), &[]);
    t.resolve_all();
    assert!(!t.on_battlefield(bogle));
    assert_eq!(dmg(&t, flier), 0);
    assert_eq!(t.counters(theirs, "loyalty"), 2);
}

#[test]
fn upkeep_hand_size_conditions_are_checked_twice() {
    cr!("603.4");
    ruling!(
        "Scalding Tongs",
        "It checks the number of cards in your hand at the beginning of upkeep and does not trigger at all if you have more than three. It checks again on resolution and does not deal damage unless you still have 3 or fewer cards."
    );
    ruling!(
        "Thumbscrews",
        "It checks the number of cards in your hand at the beginning of upkeep, and if you have 4 or fewer cards in hand it will not trigger. It checks again when it resolves and will not deal damage unless you still have 5 or more cards in hand."
    );
    // (card, hand at upkeep, change before it resolves, triggers, damage)
    for (name, hand, change, triggers, dealt) in [
        ("Scalding Tongs", 3, 0i32, true, true),
        ("Scalding Tongs", 4, 0, false, false),
        ("Scalding Tongs", 3, 1, true, false),
        ("Thumbscrews", 5, 0, true, true),
        ("Thumbscrews", 4, 0, false, false),
        ("Thumbscrews", 5, -1, true, false),
    ] {
        supported(name);
        let mut t = TestGame::new(2);
        t.battlefield(P0, name);
        t.set_step(P1, Step::End);
        // The hand as P0's upkeep begins (the turn's draw comes later).
        let mut cards: Vec<ObjectId> = (0..hand).map(|_| t.hand(P0, "Grizzly Bears")).collect();
        t.answer_targets(P0, &[pl(P1)]);
        t.advance_to(P0, Step::Upkeep);
        t.settle();
        assert_eq!(t.stack_len() == 1, triggers, "{name} {hand}");
        if change > 0 {
            t.hand(P0, "Grizzly Bears");
        } else if change < 0 {
            let c = cards.pop().unwrap();
            move_to(&mut t, c, Zone::Graveyard(P0));
        }
        t.resolve_all();
        assert_eq!(t.life(P1) == 19, dealt, "{name} {hand} {change}");
    }
}

#[test]
fn two_enters_triggers_are_ordered_by_their_controller() {
    cr!("603.3b");
    ruling!(
        "Sparkcaster",
        "It probably won’t matter much, but you choose the order in which the two triggered abilities are placed on the stack."
    );
    ruling!(
        "Razing Snidd",
        "It probably won’t matter much, but you choose the order in which the two triggered abilities are placed on the stack."
    );
    // The two orders P0 can choose give the two outcomes: the bounce (a creature moves to
    // a hand) resolves first or second.
    for name in ["Sparkcaster", "Razing Snidd"] {
        supported(name);
        let mut firsts = vec![];
        for order in [[0usize, 1], [1, 0]] {
            let mut t = TestGame::new(2);
            t.battlefield(P0, "Grizzly Bears");
            t.battlefield(P0, "Mountain");
            t.battlefield(P1, "Mountain");
            t.answer(P0, DecisionKind::Order, Answer::Indices(order.to_vec()));
            t.answer_targets(P0, &[pl(P1)]);
            t.enter(P0, name);
            t.settle();
            assert_eq!(t.stack_len(), 2, "{name}");
            let from = t.g.turn_events.len();
            t.resolve();
            firsts.push(t.g.turn_events[from..].iter().any(|e| {
                matches!(e, mtg_engine::events::Event::ZoneChange { to: Zone::Hand(_), .. })
            }));
            t.resolve_all();
        }
        assert_ne!(firsts[0], firsts[1], "{name}: {firsts:?}");
    }
}

#[test]
fn soulfire_eruption_next_target_is_chosen_after_the_previous_card_is_exiled() {
    cr!("608.2f", "401.5");
    ruling!(
        "Soulfire Eruption",
        "If you're playing with the top card of your library revealed (perhaps because of Courser of Kruphix), you'll see the new top card of your library after each time a card is exiled while Soulfire Eruption's ability is resolving."
    );
    // Three players; P0 plays with the top card revealed (Courser of Kruphix) and targets
    // P1, P2 and P2's creature. After P1 is dealt damage (Lightning Bolt exiled), P0 sees
    // Craw Wurm on top and then chooses between P2 and the creature.
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Courser of Kruphix");
    let ancients = t.battlefield(P2, "Indomitable Ancients");
    t.library_top(P0, "Shock");
    t.library_top(P0, "Craw Wurm");
    t.library_top(P0, "Lightning Bolt");
    let seen = crate::r_s01_common::watch(
        &mut t,
        P0,
        |d| {
            matches!(d, mtg_engine::decision::Decision::ChooseEntities { prompt, .. }
                if prompt == "Choose the next target")
        },
        |g| {
            let top = *g.player(P0).library.last().expect("a library");
            (g.player(P1).life, g.obj(top).chars.name.to_string())
        },
    );
    t.answer_targets(P0, &[pl(P1), pl(P2), obj(ancients)]);
    t.answer_choose(P0, &[obj(ancients)]);
    lands_for_cost(&mut t, P0, "Soulfire Eruption");
    let card = t.hand(P0, "Soulfire Eruption");
    t.cast(P0, card).go();
    t.resolve_all();
    assert_eq!(
        seen.lock().unwrap().clone(),
        vec![(19, "Craw Wurm".to_string())],
        "asked once, after P1's card"
    );
    // The creature was chosen next: Craw Wurm (6) to it, Shock (1) to P2.
    assert_eq!((dmg(&t, ancients), t.life(P2)), (6, 19));
}

#[test]
fn lithomantic_barrage_can_be_targeted_by_counterspells() {
    cr!("101.2", "608.2c");
    ruling!(
        "Lithomantic Barrage",
        "Spells and abilities that counter spells can still target Lithomantic Barrage. Such a spell or ability won’t counter Lithomantic Barrage, but any additional effects of that spell or ability will still happen."
    );
    supported("Lithomantic Barrage");
    let mut t = TestGame::new(2);
    let knight = t.battlefield(P1, "White Knight");
    let s = cast_x(
        &mut t,
        P0,
        "Lithomantic Barrage",
        NORMAL,
        None,
        &[obj(knight)],
    );
    let hand = t.hand_size(P1);
    // Dismiss: "Counter target spell. Draw a card."
    cast_new(&mut t, P1, "Dismiss", &[obj(s)]);
    t.resolve_all();
    assert_eq!(t.hand_size(P1), hand + 1, "P1 drew a card");
    assert!(t.in_graveyard(P1, "Dismiss"));
    assert!(!t.on_battlefield(knight), "Barrage resolved: 5 to a white creature");
}
