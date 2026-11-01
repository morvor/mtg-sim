//! Rulings batch P097 — "low mana value matters": the mana value of a card with {X} in
//! its mana cost outside the stack (X is 0, CR 202.3e), of a card with no mana cost
//! (CR 202.3a), the value of X when casting without paying the mana cost (CR 107.3b), and
//! companions' deck conditions (CR 702.139).

use crate::r_p097_common::*;
use crate::r_s01_common::{attack_with, block_and_finish};
use mtg_engine::decision::Answer;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Ingenious Prodigy ({X}{U}, 0/1) and Hangarback Walker ({X}{X}, 0/0) have mana value 1
/// and 0 outside the stack; Chalice of the Void ({X}{X}) is a noncreature artifact with
/// mana value 0.
#[test]
fn x_is_zero_for_cards_outside_the_stack() {
    cr!("202.3e", "107.3");
    ruling!(
        "Dewdrop Cure",
        "If a card in a graveyard has {X} in its mana cost, X is 0 for the purpose of determining its mana value."
    );
    ruling!(
        "United Battlefront",
        "If a card in a library has {X} in its mana cost, X is 0 for the purpose of determining its mana value."
    );
    ruling!(
        "Woodland Bellower",
        "If a card has {X} in its mana cost, X is 0."
    );
    ruling!(
        "Driver of the Dead",
        "If a card in a player's graveyard has {X} in its mana cost, X is 0 for the purpose of determining its mana value."
    );
    ruling!(
        "Auriok Salvagers",
        "If a card in a player's graveyard has {X} in its mana cost, X is considered to be 0."
    );
    ruling!(
        "Call of the Death-Dweller",
        "If a card in a player’s graveyard has {X} in its mana cost, X is considered to be 0."
    );
    ruling!(
        "Chthonian Nightmare",
        "If a card in your graveyard has {X} in its mana cost, X is 0 when determining its mana value."
    );
    ruling!(
        "Raise the Past",
        "If a creature card in your graveyard has {X} in its mana cost, X is 0 for the purpose of determining its mana value."
    );
    ruling!(
        "Devoted Crop-Mate",
        "If the mana cost of a card in your graveyard includes X, X is considered to be 0."
    );
    for case in [
        "Dewdrop Cure",
        "United Battlefront",
        "Woodland Bellower",
        "Driver of the Dead",
        "Auriok Salvagers",
        "Call of the Death-Dweller",
        "Chthonian Nightmare",
        "Raise the Past",
        "Devoted Crop-Mate",
    ] {
        supported(case);
        let mut t = TestGame::new(2);
        let moved: ObjectId;
        match case {
            "Dewdrop Cure" | "Raise the Past" => {
                moved = t.graveyard(P0, "Ingenious Prodigy");
                crate::r_s01_common::give_mana_for(&mut t, P0, case);
                t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(false));
                let s = t.hand(P0, case);
                if case == "Dewdrop Cure" {
                    t.cast(P0, s).targets(&[Entity::Object(moved)]).go();
                } else {
                    t.cast(P0, s).go();
                }
            }
            "Call of the Death-Dweller" => {
                // Two of them: total mana value 2.
                moved = t.graveyard(P0, "Ingenious Prodigy");
                let other = t.graveyard(P0, "Ingenious Prodigy");
                crate::r_s01_common::give_mana_for(&mut t, P0, case);
                let s = t.hand(P0, case);
                t.cast(P0, s)
                    .targets(&[Entity::Object(moved), Entity::Object(other)])
                    .go();
            }
            "United Battlefront" => {
                moved = t.library_top(P0, "Chalice of the Void");
                crate::r_s01_common::give_mana_for(&mut t, P0, case);
                t.answer_choose(P0, &[Entity::Object(moved)]);
                let s = t.hand(P0, case);
                t.cast(P0, s).go();
            }
            "Woodland Bellower" => {
                // Mistcutter Hydra ({X}{G}): a nonlegendary green creature card with mana
                // value 1 (it dies as a 0/0 once on the battlefield).
                moved = t.library_top(P0, "Mistcutter Hydra");
                t.answer_yes(P0, true);
                t.answer_choose(P0, &[Entity::Object(moved)]);
                t.enter(P0, case);
            }
            "Driver of the Dead" => {
                moved = t.graveyard(P0, "Ingenious Prodigy");
                let d = t.battlefield(P0, case);
                t.answer_targets(P0, &[Entity::Object(moved)]);
                t.g.destroy(d, None);
            }
            "Auriok Salvagers" => {
                moved = t.graveyard(P0, "Chalice of the Void");
                let s = t.battlefield(P0, case);
                pool(&mut t, P0, &[(ManaType::W, 1), (ManaType::C, 1)]);
                t.activate(P0, s, 0, &[Entity::Object(moved)]).expect("activate");
            }
            "Chthonian Nightmare" => {
                // X = 0: Hangarback Walker's mana value.
                moved = t.graveyard(P0, "Hangarback Walker");
                let n = t.battlefield(P0, case);
                let bears = t.battlefield(P0, "Grizzly Bears");
                t.g.players[0].counters.insert("energy".into(), 3);
                t.answer(P0, DecisionKind::X, Answer::Number(0));
                t.answer_choose(P0, &[Entity::Object(bears)]);
                t.activate(P0, n, 0, &[Entity::Object(moved)]).expect("activate");
            }
            _ => {
                // Devoted Crop-Mate, exerted as it attacks.
                moved = t.graveyard(P0, "Ingenious Prodigy");
                let c = t.battlefield(P0, case);
                t.answer_yes(P0, true);
                t.answer_targets(P0, &[Entity::Object(moved)]);
                attack_with(&mut t, &[(c, Entity::Player(P1))]);
            }
        }
        t.resolve_all();
        let now = t.zone(moved);
        let expected = match case {
            "Auriok Salvagers" => Zone::Hand(P0),
            // A 0/0 that died.
            "Woodland Bellower" | "Chthonian Nightmare" => Zone::Graveyard(P0),
            _ => Zone::Battlefield,
        };
        assert_eq!(now, expected, "{case}: {}", t.dump_log());
        assert_ne!(t.g.current(moved), moved, "{case}: it moved");
    }
}

#[test]
fn inscription_of_ruin_x_is_zero_in_graveyards_and_on_the_battlefield() {
    cr!("202.3e", "700.2");
    ruling!(
        "Inscription of Ruin",
        "If a card in a player's graveyard or a creature on the battlefield has {X} in its mana cost, X is considered to be 0."
    );
    supported("Inscription of Ruin");
    // Hangarback Walker cast with X = 2 (mana value 4 on the stack) has mana value 0 on
    // the battlefield: it can be destroyed.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    pool(&mut t, P1, &[(ManaType::C, 4)]);
    let w = t.hand(P1, "Hangarback Walker");
    t.cast(P1, w).x(2).go();
    t.resolve_all();
    assert_eq!(t.counters(w, "+1/+1"), 2);
    t.set_step(P0, Step::PrecombatMain);
    let prodigy = t.graveyard(P0, "Ingenious Prodigy");
    pool(&mut t, P0, &[(ManaType::B, 3), (ManaType::C, 4)]);
    let s = t.hand(P0, "Inscription of Ruin");
    let w = t.g.current(w);
    t.cast(P0, s)
        .kicked(true)
        .modes(&[1, 2])
        .target(prodigy)
        .target(w)
        .go();
    t.resolve_all();
    assert!(t.on_battlefield(prodigy), "{}", t.dump_log());
    assert!(!t.on_battlefield(w));
}

#[test]
fn a_land_card_has_mana_value_zero() {
    cr!("202.3a");
    ruling!(
        "Sun Titan",
        "If a card in your graveyard has no mana symbols in its upper right corner (because it's a land card, for example), its mana value is 0."
    );
    supported("Sun Titan");
    let mut t = TestGame::new(2);
    let forest = t.graveyard(P0, "Forest");
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(forest)]);
    t.enter(P0, "Sun Titan");
    t.resolve_all();
    assert!(t.on_battlefield(forest));
}

#[test]
fn kellan_joins_up_can_plot_a_card_with_no_mana_cost() {
    cr!("202.3a", "702.170a");
    ruling!(
        "Kellan Joins Up",
        "If a card doesn't have a mana cost, its mana value is 0."
    );
    supported("Kellan Joins Up");
    let mut t = TestGame::new(2);
    // Ancestral Vision has no mana cost.
    let vision = t.hand(P0, "Ancestral Vision");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(vision)]);
    t.enter(P0, "Kellan Joins Up");
    t.resolve_all();
    assert_eq!(t.zone(vision), Zone::Exile, "{}", t.dump_log());
    assert!(mtg_engine::kw::plot::plotted_turn(&t.g, t.g.current(vision)).is_some());
}

/// P0 casts `blaze` (Blaze, {X}{R}, in their graveyard) as the effect of `case` lets them,
/// "without paying its mana cost", answering 5 to any choice of X. P1's life afterward.
fn free_blaze(case: &str) -> i32 {
    supported(case);
    let mut t = TestGame::new(2);
    let blaze = t.graveyard(P0, "Blaze");
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(blaze)]);
    t.answer(P0, DecisionKind::X, Answer::Number(5));
    t.answer_targets(P0, &[Entity::Player(P1)]);
    if case == "Goblin Dark-Dwellers" {
        t.enter(P0, case);
        t.resolve_all();
        assert_eq!(t.zone(blaze), Zone::Exile, "{}", t.dump_log());
        t.life(P1)
    } else {
        let s = t.battlefield(P0, case);
        attack_with(&mut t, &[(s, Entity::Player(P1))]);
        block_and_finish(&mut t, P1, &[]);
        t.resolve_all();
        assert_eq!(t.zone(blaze), Zone::Exile, "{}", t.dump_log());
        t.life(P1) + t.pt(s).0
    }
}

#[test]
fn x_is_zero_when_cast_without_paying_its_mana_cost() {
    cr!("107.3b", "118.9");
    ruling!(
        "Goblin Dark-Dwellers",
        "If the instant or sorcery card has {X} in its mana cost, you must choose 0 as its value."
    );
    ruling!(
        "Seifer Almasy",
        "If a card in your graveyard has {X} in its mana cost, X is 0 for the purpose of determining its mana value and you must choose 0 as the value of X when casting it without paying its mana cost."
    );
    assert_eq!(free_blaze("Goblin Dark-Dwellers"), 20);
    // Seifer, attacking alone, has double strike: 6 combat damage.
    assert_eq!(free_blaze("Seifer Almasy"), 17);
}

#[test]
fn chandras_target_has_x_zero_but_is_cast_with_any_x() {
    cr!("202.3e", "107.3", "601.2b");
    ruling!(
        "Chandra, Acolyte of Flame",
        "If a card in your graveyard has {X} in its mana cost, X is considered to be 0 while you target it with Chandra’s last ability; however, once you begin to cast it, you choose a value for X. If this causes the mana value to become greater than 3, you can still cast the spell."
    );
    supported("Chandra, Acolyte of Flame");
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "Chandra, Acolyte of Flame");
    let blaze = t.graveyard(P0, "Blaze");
    pool(&mut t, P0, &[(ManaType::R, 1), (ManaType::C, 5)]);
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::X, Answer::Number(5));
    t.activate(P0, c, 2, &[Entity::Object(blaze)]).expect("−2");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 15, "{}", t.dump_log());
    // "If that spell would be put into your graveyard, exile it instead."
    assert_eq!(t.zone(blaze), Zone::Exile);
}

#[test]
fn chandras_first_ability_doesnt_activate_other_planeswalkers() {
    cr!("606.3", "122.6");
    ruling!(
        "Chandra, Acolyte of Flame",
        "Chandra’s first ability will put a counter on herself as well as any other red planeswalkers you control. This doesn’t cause any other planeswalkers’ loyalty abilities to be activated."
    );
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "Chandra, Acolyte of Flame");
    let torch = t.battlefield(P0, "Chandra, Torch of Defiance");
    let (lc, lt) = (t.counters(c, "loyalty"), t.counters(torch, "loyalty"));
    t.activate(P0, c, 0, &[]).expect("0");
    t.resolve_all();
    assert_eq!(t.counters(c, "loyalty"), lc + 1);
    assert_eq!(t.counters(torch, "loyalty"), lt + 1);
    // Chandra, Torch of Defiance can still activate a loyalty ability this turn.
    t.activate(P0, torch, 0, &[]).expect("Torch's +1");
}

#[test]
fn chthonian_nightmare_needs_a_target() {
    cr!("602.2b", "601.2c");
    ruling!(
        "Chthonian Nightmare",
        "Chthonian Nightmare's last ability must have a target to be activated."
    );
    let mut t = TestGame::new(2);
    let n = t.battlefield(P0, "Chthonian Nightmare");
    t.battlefield(P0, "Grizzly Bears");
    t.g.players[0].counters.insert("energy".into(), 3);
    t.answer(P0, DecisionKind::X, Answer::Number(0));
    assert!(t.activate(P0, n, 0, &[]).is_err());
    assert!(t.on_battlefield(n));
}

#[test]
fn goblin_engineer_cant_target_the_artifact_it_sacrifices() {
    cr!("602.2b", "601.2c", "601.2h");
    ruling!(
        "Goblin Engineer",
        "Because targets are chosen for activated abilities before costs (such as \"Sacrifice an artifact\") are paid, Goblin Engineer's second ability can't target the artifact that will be sacrificed to pay its cost."
    );
    supported("Goblin Engineer");
    let mut t = TestGame::new(2);
    let e = t.battlefield(P0, "Goblin Engineer");
    let ring = t.battlefield(P0, "Sol Ring");
    pool(&mut t, P0, &[(ManaType::R, 1)]);
    t.answer_choose(P0, &[Entity::Object(ring)]);
    // No artifact card in the graveyard: no target, even though one would be once the
    // cost is paid.
    assert!(t.activate(P0, e, 0, &[]).is_err());
    assert!(t.on_battlefield(ring));
    // With another artifact card there, it can be activated (sacrificing the Sol Ring).
    let mox = t.graveyard(P0, "Mox Amber");
    t.answer_choose(P0, &[Entity::Object(ring)]);
    t.activate(P0, e, 0, &[Entity::Object(mox)])
        .expect("a legal target");
    t.resolve_all();
    assert!(!t.on_battlefield(ring));
    assert!(t.on_battlefield(mox));
}

#[test]
fn inscription_of_ruins_targets_are_all_chosen_as_it_is_cast() {
    cr!("601.2c", "700.2");
    ruling!(
        "Inscription of Ruin",
        "Because targets are chosen as you cast a spell, you can't have Inscription of Ruin return a creature card and then destroy that creature."
    );
    let mut t = TestGame::new(2);
    let bears = t.graveyard(P0, "Grizzly Bears");
    pool(&mut t, P0, &[(ManaType::B, 3), (ManaType::C, 4)]);
    let s = t.hand(P0, "Inscription of Ruin");
    let from = t.asked().len();
    t.cast(P0, s).kicked(true).modes(&[1]).target(bears).go();
    // The destroy mode couldn't be chosen: no creature was on the battlefield to target.
    let available: Vec<Vec<usize>> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            mtg_engine::decision::Decision::ChooseModes { available, .. } => {
                Some(available.clone())
            }
            _ => None,
        })
        .collect();
    assert_eq!(available, vec![vec![0, 1]]);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
}

#[test]
fn bygone_bishop_madness_doesnt_change_mana_value() {
    cr!("202.3", "118.9c", "702.35a");
    ruling!(
        "Bygone Bishop",
        "Casting a spell for an alternative cost, such as a madness cost, doesn't change its mana value."
    );
    supported("Bygone Bishop");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Bygone Bishop");
    let twins = t.hand(P0, "Twins of Maurer Estate");
    pool(&mut t, P0, &[(ManaType::B, 1), (ManaType::C, 2)]);
    t.answer_yes(P0, true);
    t.g.discard(P0, twins, None);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Twins of Maurer Estate").len(), 1);
    assert!(crate::r_s01_common::with_subtype(&t, P0, "Clue").is_empty());
    // A three-mana creature spell does investigate.
    crate::r_s01_common::give_mana_for(&mut t, P0, "Phantom Warrior");
    let g = t.hand(P0, "Phantom Warrior");
    t.cast(P0, g).go();
    t.resolve_all();
    assert_eq!(crate::r_s01_common::with_subtype(&t, P0, "Clue").len(), 1);
}

#[test]
fn each_card_needs_low_mana_value_not_their_total() {
    cr!("202.3");
    ruling!(
        "Collected Company",
        "Each of the creature cards can have mana value 3 or less. It's not the total mana value of the two cards."
    );
    ruling!(
        "Kayla's Reconstruction",
        "Each of the cards needs to have mana value 3 or less. It's not the total mana value of the cards."
    );
    for case in ["Collected Company", "Kayla's Reconstruction"] {
        supported(case);
        let mut t = TestGame::new(2);
        let cards = stack_library(
            &mut t,
            P0,
            &["Phantom Warrior", "Phantom Warrior", "Craw Wurm"],
        );
        if case == "Collected Company" {
            pool(&mut t, P0, &[(ManaType::G, 1), (ManaType::C, 3)]);
        } else {
            pool(&mut t, P0, &[(ManaType::W, 3), (ManaType::C, 2)]);
        }
        t.answer_choose(P0, &[Entity::Object(cards[0]), Entity::Object(cards[1])]);
        let s = t.hand(P0, case);
        if case == "Collected Company" {
            t.cast(P0, s).go();
        } else {
            t.cast(P0, s).x(2).go();
        }
        t.resolve_all();
        assert_eq!(t.named_on_battlefield("Phantom Warrior").len(), 2, "{case}");
    }
}

#[test]
fn emptiness_checks_all_mana_spent_and_doesnt_multiply() {
    cr!("601.2h", "603.4");
    ruling!(
        "Emptiness",
        "Emptiness's first and second abilities care about what mana was spent to pay its total cost, not just what mana was spent to pay the hybrid mana symbols in its cost."
    );
    ruling!(
        "Emptiness",
        "Emptiness's first and second abilities check to see if at least two mana of the appropriate colors were spent to pay Emptiness's cost. It doesn't matter if more than two mana of that color was spent; the effect isn't multiplied."
    );
    supported("Emptiness");
    // (mana, creatures returned, -1/-1 counters on the target)
    for (mana, returned, counters) in [
        // Two of each: both abilities, whatever paid the hybrid symbols.
        (vec![(ManaType::W, 2), (ManaType::B, 2), (ManaType::C, 2)], 1, 3),
        // Six white: only the first, once.
        (vec![(ManaType::W, 6)], 1, 0),
        // One white, five black: only the second.
        (vec![(ManaType::W, 1), (ManaType::B, 5)], 0, 3),
    ] {
        let mut t = TestGame::new(2);
        let a = t.graveyard(P0, "Grizzly Bears");
        let b = t.graveyard(P0, "Grizzly Bears");
        let giant = t.battlefield(P1, "Craw Wurm");
        pool(&mut t, P0, &mana);
        let s = t.hand(P0, "Emptiness");
        if returned > 0 {
            t.answer_targets(P0, &[Entity::Object(a)]);
        }
        if counters > 0 {
            t.answer_targets(P0, &[Entity::Object(giant)]);
        }
        t.cast(P0, s).go();
        t.resolve_all();
        let back = [a, b].iter().filter(|c| t.on_battlefield(**c)).count();
        assert_eq!(back, returned, "{mana:?}: {}", t.dump_log());
        assert_eq!(t.counters(giant, "-1/-1"), counters, "{mana:?}");
    }
}

#[test]
fn guidelight_pathmaker_shuffles_only_if_you_search() {
    cr!("701.24a", "701.23a");
    ruling!(
        "Guidelight Pathmaker",
        "Specifically, you'll only shuffle if you choose to search your library."
    );
    supported("Guidelight Pathmaker");
    let mut t = TestGame::new(2);
    stack_library(&mut t, P0, &["Forest", "Island", "Swamp", "Mountain"]);
    let before = t.g.player(P0).library.clone();
    t.answer_yes(P0, false);
    t.enter(P0, "Guidelight Pathmaker");
    t.resolve_all();
    assert_eq!(t.g.player(P0).library, before);
}

#[test]
fn lurrus_x_is_zero_in_the_starting_deck() {
    cr!("702.139a", "202.3e");
    ruling!(
        "Lurrus of the Dream-Den",
        "If a card in a player's deck has {X} in its mana cost, X is considered to be 0."
    );
    use mtg_engine::card::card;
    // Hangarback Walker ({X}{X}) has mana value 0; Hill Giant (mana value 4) breaks the
    // condition.
    for (extra, ok) in [("Hangarback Walker", true), ("Hill Giant", false)] {
        let mut deck: Vec<_> = (0..20).map(|_| card("Grizzly Bears")).collect();
        deck.extend((0..24).map(|_| card("Forest")));
        deck.extend((0..16).map(|_| card(extra)));
        let mut t = crate::r_s12_common::pregame(
            mtg_engine::game::GameConfig {
                starting_player: Some(P0),
                ..Default::default()
            },
            vec![deck, (0..60).map(|_| card("Forest")).collect()],
        );
        let side = t.g.add_to_sideboard(P0, vec![card("Lurrus of the Dream-Den")]);
        t.answer_choose(P0, &[Entity::Object(side[0])]);
        t.g.start();
        assert_eq!(t.g.companion_of(P0) == Some(side[0]), ok, "{extra}");
    }
}
