//! Rulings batch P069 — miscellaneous rulings of cards that cast from exile: who makes a
//! choice in multiplayer (CR 800.4g-style opponent choices), land cards in exile (CR 305),
//! characteristics as a card enters (CR 614.12), untap restrictions (CR 502.3), cards
//! drawn later (CR 121), mana values of spells cast (CR 601.3e), copiable values (CR
//! 707.2), card types (CR 205.2a), cards exiled instead of drawn, copies of cards cast from
//! exile (CR 707.12), and once-a-turn permissions.

use crate::r_p067_common::move_to;
use crate::r_p069_common::*;
use crate::r_s01_common::{stack_library, supported, watch};
use crate::r_s02_common::can_attack;
use crate::r_s04_common::can_cycle;
use mtg_engine::card::card;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::facedown;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

#[test]
fn plargg_and_nassari_you_choose_the_opponent_who_chooses() {
    cr!("608.2d");
    ruling!(
        "Plargg and Nassari",
        "In a multiplayer game, you choose an opponent to choose a nonland card exiled with Plargg and Nassari."
    );
    supported("Plargg and Nassari");
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Plargg and Nassari");
    stack_library(&mut t, P0, &["Grizzly Bears"]);
    stack_library(&mut t, P1, &["Hill Giant"]);
    stack_library(&mut t, P2, &["Serra Angel"]);
    t.set_step(P2, Step::End);
    let from = t.asked().len();
    t.answer_choose(P0, &[Entity::Player(P2)]);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    let asked = &t.asked()[from..];
    // P0 was offered both opponents, and the one it chose, P2, chose a card.
    let offered_players = asked.iter().any(|(p, d)| {
        *p == P0
            && matches!(d, Decision::ChooseEntities { candidates, .. }
                if candidates.contains(&Entity::Player(P1)) && candidates.contains(&Entity::Player(P2)))
    });
    assert!(offered_players, "{asked:#?}");
    let chose = |q: PlayerId| {
        asked.iter().any(|(p, d)| {
            *p == q
                && matches!(d, Decision::ChooseEntities { candidates, .. }
                    if candidates.iter().any(|e| matches!(e, Entity::Object(_))))
        })
    };
    assert!(chose(P2), "{asked:#?}");
    assert!(!chose(P1), "{asked:#?}");
}

#[test]
fn abstruse_appropriation_cant_play_an_exiled_land_card() {
    cr!("305.1", "707.2", "400.7");
    ruling!(
        "Abstruse Appropriation",
        "In the rare case where the exiled card is a land card (probably because that land became a copy of a permanent while it was still on the battlefield), you won't be able to play it from exile."
    );
    // P1's Forest is a copy of a Grizzly Bears: a nonland permanent Abstruse Appropriation
    // can target. In exile it's a Forest card again.
    let mut t = TestGame::new(2);
    let forest = t.battlefield(P1, "Forest");
    let bears = t.battlefield(P1, "Grizzly Bears");
    crate::r_s17_common::become_copy(&mut t, forest, bears);
    assert_eq!(t.obj_now(forest).chars.name, "Grizzly Bears");
    let aa = t.hand(P0, "Abstruse Appropriation");
    mana(&mut t, P0, ManaType::W, 1);
    mana(&mut t, P0, ManaType::B, 1);
    mana(&mut t, P0, ManaType::C, 2);
    t.cast(P0, aa).target(forest).go();
    t.resolve_all();
    assert!(exiled(&t, forest), "{}", t.dump_log());
    assert_eq!(t.obj_now(forest).chars.name, "Forest");
    assert!(!playable_land(&mut t, P0, forest));
    mana(&mut t, P0, ManaType::C, 5);
    assert!(!castable(&mut t, P0, forest));
}

#[test]
fn laughing_jasper_flint_makes_a_stolen_creature_an_outlaw_as_it_enters() {
    cr!("614.12", "603.6a");
    ruling!(
        "Laughing Jasper Flint",
        "Laughing Jasper Flint’s first ability applies to creatures you control but don’t own as they enter the battlefield. For example, if you control Laughing Jasper Flint and Vial Smasher, Gleeful Grenadier, having a creature an opponent owns that normally isn’t an outlaw enter the battlefield under your control will cause Vial Smasher’s ability to trigger."
    );
    supported("Laughing Jasper Flint");
    supported("Vial Smasher, Gleeful Grenadier");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Laughing Jasper Flint");
    t.battlefield(P0, "Vial Smasher, Gleeful Grenadier");
    // P0's own Grizzly Bears: not an outlaw.
    t.enter(P0, "Grizzly Bears");
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    // P1's Grizzly Bears, reanimated under P0's control: a Mercenary as it enters.
    let bears = t.graveyard(P1, "Grizzly Bears");
    let r = t.hand(P0, "Reanimate");
    mana(&mut t, P0, ManaType::B, 1);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.cast(P0, r).target(bears).go();
    t.resolve_all();
    assert!(t.obj_now(bears).chars.has_subtype("Mercenary"));
    assert_eq!(t.life(P1), 19, "{}", t.dump_log());
}

#[test]
fn maelstrom_wanderer_gives_itself_haste() {
    cr!("302.6", "702.10a");
    ruling!("Maelstrom Wanderer", "Maelstrom Wanderer gives itself haste.");
    let mut t = TestGame::new(2);
    let mw = t.battlefield_sick(P0, "Maelstrom Wanderer");
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(can_attack(&mut t, mw));
}

#[test]
fn hazorets_undying_fury_no_land_untaps_even_untapped_or_new_ones() {
    cr!("502.3", "611.2a");
    ruling!(
        "Hazoret's Undying Fury",
        "No lands that you control will untap during your next untap step, even lands that aren't tapped as this spell resolves. This includes lands that enter the battlefield after this spell resolves."
    );
    supported("Hazoret's Undying Fury");
    let mut t = TestGame::new(2);
    let tapped = t.battlefield(P0, "Mountain");
    t.g.tap(tapped);
    let untapped = t.battlefield(P0, "Mountain");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.tap(bears);
    let h = t.hand(P0, "Hazoret's Undying Fury");
    mana(&mut t, P0, ManaType::R, 6);
    t.answer_choose(P0, &[]);
    t.cast(P0, h).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Hazoret's Undying Fury"), "{}", t.dump_log());
    // Afterwards: the untapped Mountain is tapped, and a new Forest enters and is tapped.
    t.g.tap(untapped);
    let forest = t.hand(P0, "Forest");
    t.play_land(P0, forest).unwrap();
    let forest = t.g.current(forest);
    t.g.tap(forest);
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    for (i, land) in [tapped, untapped, forest].into_iter().enumerate() {
        assert!(t.obj_now(land).tapped, "land {i}: {}", t.dump_log());
    }
    assert!(!t.obj_now(bears).tapped);
}

#[test]
fn hazorets_undying_fury_no_actions_between_shuffle_and_exile() {
    cr!("608.2", "117.1");
    ruling!(
        "Hazoret's Undying Fury",
        "No player may take any actions between the time you shuffle your library and the time you exile the top four cards."
    );
    // Nobody gets priority (or any other decision) while it resolves before the cards are
    // exiled: every decision during its resolution comes after the four cards are exiled.
    let mut t = TestGame::new(2);
    let h = t.hand(P0, "Hazoret's Undying Fury");
    mana(&mut t, P0, ManaType::R, 6);
    t.cast(P0, h).go();
    t.settle();
    let seen = [P0, P1].map(|p| watch(&mut t, p, |_| true, |g| g.exile.len()));
    t.g.resolve_top();
    assert!(seen[1].lock().unwrap().is_empty());
    let s0 = seen[0].lock().unwrap().clone();
    assert!(!s0.is_empty(), "P0 chooses among the exiled cards");
    assert!(s0.iter().all(|n| *n == 4), "{s0:?}");
}

#[test]
fn apple_of_eden_cards_drawn_later_go_to_hand() {
    cr!("121.1", "406.6");
    ruling!(
        "Apple of Eden, Isu Relic",
        "Only cards in the target opponent’s hand when Apple of Eden’s ability resolves will be exiled face down. Any cards they draw later in the turn, including cards drawn because of Apple of Eden’s first delayed triggered ability, will simply go to their hand. You won’t see those cards, they won’t be exiled face down, and you won’t be able to play them."
    );
    supported("Apple of Eden, Isu Relic");
    let mut t = TestGame::new(2);
    let apple = t.battlefield(P0, "Apple of Eden, Isu Relic");
    let bears = t.hand(P1, "Grizzly Bears");
    let bolt = t.library_top(P1, "Lightning Bolt");
    t.activate(P0, apple, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    assert!(exiled(&t, bears), "{}", t.dump_log());
    assert!(t.obj_now(bears).face_down);
    // Mana of any type.
    mana(&mut t, P0, ManaType::B, 2);
    cast_now(&mut t, P0, bears, &[]);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    // P1 drew the Lightning Bolt: it's in P1's hand, not exiled, and P0 can't play it.
    assert_eq!(t.zone(bolt), Zone::Hand(P1), "{}", t.dump_log());
    let b = t.g.current(bolt);
    assert!(!t.obj(b).face_down);
    assert!(!facedown::can_look_at(&t.g, P0, b));
    mana(&mut t, P0, ManaType::R, 1);
    assert!(!castable(&mut t, P0, bolt));
}

#[test]
fn primeval_spawn_uses_the_mana_values_of_the_spells_cast() {
    cr!("601.3e", "712.8f", "712.11c", "202.3");
    interp!("total-mana-value-choice-fitted");
    ruling!(
        "Primeval Spawn",
        "Primeval Spawn’s last ability checks the mana values and types of the spells on the stack, not the mana values and types of the cards in exile. Notably, this means that you may cast the back face of a modal double-faced card or either face of a split card as long as the spells you are casting together have a total mana value of 10 or less."
    );
    // Serra Angel (5), Grizzly Bears (2) and Sword of the Realms (2), the back face of
    // Halvar, God of Battle (a card with mana value 4): 9 as spells, 11 as cards. Halvar
    // can only be cast as Sword of the Realms. With Hill Giant (4) instead: 11 as spells,
    // so the Giant, chosen last, isn't cast.
    for third in ["Halvar, God of Battle // Sword of the Realms", "Hill Giant"] {
        let mut t = TestGame::new(2);
        let spawn = t.battlefield(P0, "Primeval Spawn");
        let cards = stack_library(&mut t, P0, &["Serra Angel", "Grizzly Bears", third]);
        t.answer_choose_moved(P0, &cards.iter().map(|c| o(*c)).collect::<Vec<_>>());
        t.g.destroy(spawn, None);
        t.resolve_all();
        assert!(t.on_battlefield(cards[0]) && t.on_battlefield(cards[1]), "{}", t.dump_log());
        if third == "Hill Giant" {
            assert!(exiled(&t, cards[2]), "{}", t.dump_log());
        } else {
            assert!(t.on_battlefield(cards[2]), "{}", t.dump_log());
            assert_eq!(t.obj_now(cards[2]).chars.name, "Sword of the Realms");
        }
    }
}

#[test]
fn primeval_spawn_fits_an_answer_over_the_total_in_the_order_listed() {
    cr!("601.3e", "712.11c", "608.2g");
    interp!("total-mana-value-choice-fitted");
    // Each case: the cards exiled (and chosen, in this order), and which of them are cast
    // (as which spell). Serra Angel (5), Craw Wurm (6), Grizzly Bears (2): the Wurm doesn't
    // fit after the Angel, but the Bears still do. Halvar, God of Battle (4) cast first,
    // before Serra Angel and Grizzly Bears: only as Sword of the Realms (2), so that the
    // other two still fit.
    let cases: [(&[&str], &[Option<&str>]); 2] = [
        (
            &["Serra Angel", "Craw Wurm", "Grizzly Bears"],
            &[Some("Serra Angel"), None, Some("Grizzly Bears")],
        ),
        (
            &[
                "Halvar, God of Battle // Sword of the Realms",
                "Serra Angel",
                "Grizzly Bears",
            ],
            &[
                Some("Sword of the Realms"),
                Some("Serra Angel"),
                Some("Grizzly Bears"),
            ],
        ),
    ];
    for (names, cast) in cases {
        let mut t = TestGame::new(2);
        let spawn = t.battlefield(P0, "Primeval Spawn");
        let cards = stack_library(&mut t, P0, names);
        t.answer_choose_moved(P0, &cards.iter().map(|c| o(*c)).collect::<Vec<_>>());
        t.g.destroy(spawn, None);
        t.resolve_all();
        for (c, name) in cards.iter().zip(cast) {
            match name {
                Some(n) => {
                    assert!(t.on_battlefield(*c), "{n}: {}", t.dump_log());
                    assert_eq!(t.obj_now(*c).chars.name, *n);
                }
                None => assert!(exiled(&t, *c), "{}", t.dump_log()),
            }
        }
    }
}

#[test]
fn sakashimas_protege_copies_only_copiable_values() {
    cr!("707.2");
    ruling!(
        "Sakashima's Protege",
        "Sakashima's Protege copies exactly what was printed on the original permanent (unless that permanent is copying something else or is a token; see below). It doesn't copy whether that permanent is tapped or untapped, whether it has any counters on it or any Auras and Equipment attached to it, or any non-copy effects that have changed its power, toughness, types, color, or so on."
    );
    supported("Sakashima's Protege");
    // P1's Grizzly Bears entered this turn: tapped, with a +1/+1 counter, Giant Growth and
    // Rancor ("Enchanted creature gets +2/+0 and has trample.") on it.
    let mut t = TestGame::new(2);
    let bears = t.enter(P1, "Grizzly Bears");
    t.resolve_all();
    t.g.tap(bears);
    t.g.add_counters(Entity::Object(bears), counters::PLUS1, 1, None);
    let rancor = t.battlefield(P1, "Rancor");
    t.g.attach(rancor, Entity::Object(bears));
    let gg = t.hand(P1, "Giant Growth");
    mana(&mut t, P1, ManaType::G, 1);
    t.g.turn.priority = Some(P1);
    t.cast(P1, gg).target(bears).go();
    t.resolve_all();
    assert_eq!(t.pt(bears), (8, 6));
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[o(bears)]);
    let sp = t.enter(P0, "Sakashima's Protege");
    t.resolve_all();
    assert_eq!(t.obj_now(sp).chars.name, "Grizzly Bears");
    assert_eq!(t.pt(sp), (2, 2));
    assert!(!t.obj_now(sp).tapped);
    assert_eq!(t.counters(sp, counters::PLUS1), 0);
    assert_eq!(t.obj_now(rancor).attached_to, Some(Entity::Object(bears)));
}

#[test]
fn ashiok_minus_three_may_exile_the_returned_card() {
    cr!("608.2c");
    ruling!(
        "Ashiok, Nightmare Muse",
        "The card exiled while resolving Ashiok's second ability may be the one that was returned to its owner's hand."
    );
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Ashiok, Nightmare Muse");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.hand(P1, "Hill Giant");
    t.answer_choose_moved(P1, &[o(bears)]);
    t.activate(P0, a, 1, &[o(bears)]).unwrap();
    t.resolve_all();
    assert!(exiled(&t, bears), "{}", t.dump_log());
    assert!(t.in_hand(P1, "Hill Giant"));
}

#[test]
fn bloodbraid_marauder_counts_card_types_not_supertypes_or_subtypes() {
    cr!("205.2a", "205.4a", "702.85a");
    ruling!(
        "Bloodbraid Marauder",
        "The card types that can appear in a graveyard are artifact, battle, creature, enchantment, instant, kindred, land, planeswalker, and sorcery. Legendary, basic, and snow are supertypes, not card types; Human and Berserker are subtypes, not card types."
    );
    // Nameless Inversion is a kindred instant (two types): with Snow-Covered Forest and
    // Isamaru, Hound of Konda (legendary), four card types. With Lightning Bolt instead,
    // three, however many supertypes and subtypes.
    for (spell, cascade) in [("Nameless Inversion", true), ("Lightning Bolt", false)] {
        let mut t = TestGame::new(2);
        for c in [spell, "Snow-Covered Forest", "Isamaru, Hound of Konda", "Grizzly Bears"] {
            t.graveyard(P0, c);
        }
        t.library_top(P0, "Ornithopter");
        let m = t.hand(P0, "Bloodbraid Marauder");
        mana(&mut t, P0, ManaType::R, 2);
        t.cast(P0, m).go();
        t.settle();
        assert_eq!(t.stack_len(), if cascade { 2 } else { 1 }, "{spell}: {}", t.dump_log());
    }
}

#[test]
fn portent_of_calamity_exiles_a_kindred_card_as_its_own_type() {
    cr!("205.2a", "308.1");
    ruling!(
        "Portent of Calamity",
        "The card types that can appear on cards you reveal with Portent of Calamity are artifact, battle, creature, enchantment, instant, kindred, land, planeswalker, and sorcery. Legendary, basic, and snow are supertypes, not card types. Raccoon and Scout are subtypes, not card types."
    );
    supported("Portent of Calamity");
    // Nameless Inversion (kindred instant), Lightning Bolt (instant), Grizzly Bears and a
    // Snow-Covered Forest: one card each of kindred, instant, creature and land — four
    // cards, so P0 may cast one of them for free.
    let mut t = TestGame::new(2);
    let cards = stack_library(
        &mut t,
        P0,
        &[
            "Nameless Inversion",
            "Lightning Bolt",
            "Grizzly Bears",
            "Snow-Covered Forest",
        ],
    );
    let p = t.hand(P0, "Portent of Calamity");
    mana(&mut t, P0, ManaType::U, 5);
    t.answer_choose_moved(P0, &cards.iter().map(|c| o(*c)).collect::<Vec<_>>());
    // Cast Grizzly Bears for free.
    t.answer_choose_moved(P0, &[o(cards[2])]);
    t.answer_yes(P0, true);
    t.cast(P0, p).x(4).go();
    t.resolve_all();
    assert!(t.on_battlefield(cards[2]), "{}", t.dump_log());
    // None went to the graveyard: all four were exiled, then cast or put into the hand.
    for c in &cards {
        assert_ne!(t.zone(*c), Zone::Graveyard(P0), "{}", t.dump_log());
    }
}

#[test]
fn shared_fate_cards_are_exiled_not_drawn_and_can_only_be_played() {
    cr!("614.1a", "702.29a", "121.1");
    ruling!(
        "Shared Fate",
        "The cards are exiled, not put onto the players' hands. Players can look at and play the exiled cards, but can't do anything else with them (the exiled cards can't be discarded or cycled, for example)."
    );
    supported("Shared Fate");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Shared Fate");
    let cave = t.library_top(P1, "Forgotten Cave");
    let bolt = t.library_top(P1, "Lightning Bolt");
    t.g.draw_cards(P0, 2);
    t.settle();
    assert_eq!(t.hand_size(P0), 0, "{}", t.dump_log());
    for c in [bolt, cave] {
        assert!(exiled(&t, c));
        assert!(t.obj_now(c).face_down);
        assert!(facedown::can_look_at(&t.g, P0, t.g.current(c)));
        assert!(!facedown::can_look_at(&t.g, P1, t.g.current(c)));
    }
    mana(&mut t, P0, ManaType::R, 1);
    assert!(castable(&mut t, P0, bolt));
    assert!(playable_land(&mut t, P0, cave));
    assert!(!can_cycle(&mut t, P0, cave));
}

/// Exiles `name` from P0's sideboard as if with a card named `with` (CR 905.2b-style
/// draft abilities, Arcane Savant).
fn exiled_with(t: &mut TestGame, name: &str, with: &str) -> ObjectId {
    let side = t.g.add_to_sideboard(P0, vec![card(name)]);
    let x = t
        .g
        .move_object(side[0], Zone::Exile, mtg_engine::events::MoveCause::Effect, Some(P0))
        .unwrap();
    t.g.named_exiles.push((P0, with.into(), x));
    x
}

#[test]
fn arcane_savant_copy_ignores_type_timing_but_not_other_restrictions() {
    cr!("707.12", "608.2g", "307.1");
    ruling!(
        "Arcane Savant",
        "The copy is created in and cast from exile. You cast the copy during the resolution of Arcane Savant's last ability. You ignore any timing restrictions based on the card type of the card you copied. However, you must still follow any other timing restrictions, such as \"Cast [this spell] only during combat.\""
    );
    supported("Arcane Savant");
    // Divination (a sorcery) in combat: cast.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::DeclareAttackers);
    let div = exiled_with(&mut t, "Divination", "Arcane Savant");
    t.answer_choose(P0, &[o(div)]);
    t.answer_yes(P0, true);
    let hand = t.hand_size(P0);
    t.enter(P0, "Arcane Savant");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2, "{}", t.dump_log());
    assert!(exiled(&t, div));
    // Panic ("Cast this spell only during combat before blockers are declared.") in a
    // main phase: not cast.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let panic = exiled_with(&mut t, "Panic", "Arcane Savant");
    t.answer_choose(P0, &[o(panic)]);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[o(bears)]);
    t.enter(P0, "Arcane Savant");
    t.resolve_all();
    assert!(!t.dump_log().contains("casts Panic"), "{}", t.dump_log());
    let _ = bears;
}

#[test]
fn elite_arcanist_copy_follows_cast_only_during_combat() {
    cr!("707.12", "601.3");
    ruling!(
        "Elite Arcanist",
        "The copy is created in and cast from exile. You must still follow any timing restrictions, such as \"Cast [this spell] only during combat.\""
    );
    // Panic: not in a main phase; in the beginning of combat step, yes.
    for combat in [false, true] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        let panic = t.hand(P0, "Panic");
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[o(panic)]);
        let ea = t.enter(P0, "Elite Arcanist");
        t.resolve_all();
        assert!(exiled(&t, panic), "{}", t.dump_log());
        t.g.obj_mut(ea).summoning_sick = false;
        if combat {
            t.set_step(P0, Step::BeginningOfCombat);
        }
        mana(&mut t, P0, ManaType::C, 1);
        t.answer_yes(P0, true);
        t.answer_targets(P0, &[o(bears)]);
        t.activate(P0, ea, 0, &[]).unwrap();
        t.resolve_all();
        assert_eq!(
            t.dump_log().contains("casts Panic"),
            combat,
            "{}",
            t.dump_log()
        );
        assert!(exiled(&t, panic));
    }
}

#[test]
fn hedonists_trove_once_a_turn_limit_is_only_for_nonland_cards() {
    cr!("305.2", "601.3");
    ruling!(
        "Hedonist's Trove",
        "The once-a-turn restriction of the last ability applies only to nonland cards. You can play a land card exiled with Hedonist’s Trove on the same turn you cast a nonland card exiled that way."
    );
    supported("Hedonist's Trove");
    let mut t = TestGame::new(2);
    let forest = t.graveyard(P1, "Forest");
    let bolt = t.graveyard(P1, "Lightning Bolt");
    let shock = t.graveyard(P1, "Shock");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.enter(P0, "Hedonist's Trove");
    t.resolve_all();
    for c in [forest, bolt, shock] {
        assert!(exiled(&t, c), "{}", t.dump_log());
    }
    mana(&mut t, P0, ManaType::R, 1);
    cast_now(&mut t, P0, bolt, &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    mana(&mut t, P0, ManaType::R, 1);
    assert!(!castable(&mut t, P0, shock));
    t.play_land(P0, t.g.current(forest)).unwrap();
    assert!(t.on_battlefield(forest));
}

#[test]
fn ensnared_by_the_mara_the_opponent_chooses_only_the_option() {
    cr!("701.55a", "608.2d");
    ruling!(
        "Ensnared by the Mara",
        "The opponent chooses whether to exile cards from the top of their library until they exile a nonland card or to exile the top four cards of their library. They don't choose whether you cast the cards exiled by the first option."
    );
    supported("Ensnared by the Mara");
    let mut t = TestGame::new(2);
    let cards = stack_library(&mut t, P1, &["Forest", "Grizzly Bears"]);
    let e = t.hand(P0, "Ensnared by the Mara");
    mana(&mut t, P0, ManaType::R, 4);
    t.answer(P1, DecisionKind::Option, Answer::Index(0));
    t.answer_yes(P0, true);
    let from = t.asked().len();
    t.cast(P0, e).go();
    t.resolve_all();
    let asked = &t.asked()[from..];
    assert!(asked
        .iter()
        .any(|(p, d)| *p == P1 && matches!(d, Decision::ChooseOption { .. })));
    assert!(!asked
        .iter()
        .any(|(p, d)| *p == P1 && matches!(d, Decision::YesNo { .. } | Decision::ChooseEntities { .. })));
    assert!(t.on_battlefield(cards[1]), "{}", t.dump_log());
    assert_eq!(t.obj_now(cards[1]).controller, P0);
}

#[test]
fn chaos_wand_stops_at_the_first_instant_or_sorcery() {
    cr!("701.13a", "608.2c");
    ruling!(
        "Chaos Wand",
        "The opponent stops exiling cards as soon as they exile any instant or sorcery card. You don't choose which card type to look for."
    );
    supported("Chaos Wand");
    let mut t = TestGame::new(2);
    let w = t.battlefield(P0, "Chaos Wand");
    let cards = stack_library(&mut t, P1, &["Hill Giant", "Divination", "Lightning Bolt"]);
    mana(&mut t, P0, ManaType::C, 4);
    t.answer_yes(P0, true);
    let hand = t.hand_size(P0);
    let from = t.asked().len();
    t.activate(P0, w, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2, "{}", t.dump_log());
    assert!(t.in_graveyard(P1, "Divination"));
    assert_eq!(t.g.library_top(P1), Some(cards[2]));
    assert!(!t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseOption { .. })));
}

#[test]
fn living_lore_becomes_0_0_if_the_card_leaves_exile() {
    cr!("604.3", "704.5f");
    ruling!(
        "Living Lore",
        "Similarly, if the exiled card somehow leaves exile while Living Lore is on the battlefield, it will become a 0/0 creature."
    );
    let mut t = TestGame::new(2);
    let div = t.graveyard(P0, "Divination");
    let ll = t.enter(P0, "Living Lore");
    t.resolve_all();
    assert_eq!(t.pt(ll), (3, 3));
    move_to(&mut t, div, Zone::Graveyard(P0));
    assert!(t.in_graveyard(P0, "Living Lore"), "{}", t.dump_log());
}
