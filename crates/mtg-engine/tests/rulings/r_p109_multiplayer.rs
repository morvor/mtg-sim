//! Rulings batch P109 — multiplayer cards: choices made in turn order during resolution
//! (CR 101.4, 608.2), "until ~ leaves the battlefield" (CR 610.3), players leaving the game
//! (CR 800.4a), attack triggers that look at any player's attack (CR 508.1m, 603.2),
//! intervening "if" clauses (CR 603.4), and lands with two basic land types (CR 205.3i).

use crate::r_p009_common::{kill, obj, pl};
use crate::r_s01_common::{attack_with, supported, watch};
use crate::r_s26_common::modify_until_eot;
use mtg_engine::ability::Modification;
use mtg_engine::decision::Decision;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

// --- Manifold Insights ------------------------------------------------------------------

/// Three players; P0's library, top first: the given cards. P1 and P2 choose `p1` and
/// `p2`. Returns the game after Manifold Insights resolved and the library cards.
fn manifold(top_first: &[&str], p1: Option<usize>, p2: Option<usize>) -> (TestGame, Vec<ObjectId>) {
    supported("Manifold Insights");
    let mut t = TestGame::new(3);
    let mut cards: Vec<ObjectId> = top_first
        .iter()
        .rev()
        .map(|n| t.library_top(P0, n))
        .collect();
    cards.reverse();
    if let Some(i) = p1 {
        t.answer_choose(P1, &[obj(cards[i])]);
    }
    if let Some(i) = p2 {
        t.answer_choose(P2, &[obj(cards[i])]);
    }
    t.lands(P0, "Island", 3);
    let mi = t.hand(P0, "Manifold Insights");
    t.cast(P0, mi).go();
    t.resolve();
    (t, cards)
}

const TEN: [&str; 10] = [
    "Grizzly Bears",
    "Lightning Bolt",
    "Hill Giant",
    "Forest",
    "Forest",
    "Forest",
    "Forest",
    "Forest",
    "Forest",
    "Forest",
];

#[test]
fn manifold_insights_opponents_choose_during_resolution() {
    cr!("608.2c", "101.4");
    ruling!(
        "Manifold Insights",
        "Once the top ten cards of your library are revealed, no player may take any other actions until Manifold Insights has been completely resolved."
    );
    supported("Manifold Insights");
    // Each opponent's choice is made while the spell is still resolving (it's on the
    // stack, nobody gets priority in between), the next opponent in turn order first.
    let mut t = TestGame::new(3);
    let seen1 = watch(
        &mut t,
        P1,
        |d| matches!(d, Decision::ChooseEntities { .. }),
        |g| (g.stack.len(), g.turn.priority),
    );
    let seen2 = watch(
        &mut t,
        P2,
        |d| matches!(d, Decision::ChooseEntities { .. }),
        |g| (g.stack.len(), g.turn.priority),
    );
    let mut cards: Vec<ObjectId> = TEN.iter().rev().map(|n| t.library_top(P0, n)).collect();
    cards.reverse();
    t.lands(P0, "Island", 3);
    let mi = t.hand(P0, "Manifold Insights");
    t.cast(P0, mi).go();
    t.resolve();
    let s1 = seen1.lock().unwrap().clone();
    let s2 = seen2.lock().unwrap().clone();
    assert_eq!(s1.len(), 1);
    assert_eq!(s2.len(), 1);
    let order: Vec<PlayerId> = t
        .asked()
        .iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseEntities { .. }))
        .map(|(p, _)| *p)
        .collect();
    assert_eq!(order, vec![P1, P2], "starting with the next opponent");
    assert_eq!(t.hand_size(P0), 2);

    ruling!(
        "Manifold Insights",
        "After Manifold Insights resolves, if it’s your turn, you may play one of the cards if it’s legal to do so before any other player can take any action."
    );
    // P1 chose the Lightning Bolt for P0: P0 receives priority first and casts it.
    let (mut t, cards) = manifold(&TEN, Some(1), Some(0));
    assert_eq!(t.zone(cards[1]), Zone::Hand(P0));
    assert_eq!(t.zone(cards[0]), Zone::Hand(P0));
    t.g.settle();
    assert_eq!(t.g.turn.priority, Some(P0));
    t.lands(P0, "Mountain", 1);
    let bolt = t.g.current(cards[1]);
    t.cast(P0, bolt).target(pl(P1)).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn manifold_insights_with_too_few_nonland_cards() {
    cr!("608.2c");
    ruling!(
        "Manifold Insights",
        "If there aren’t enough nonland cards for each opponent to choose one, all of the nonland cards are put into your hand."
    );
    // One nonland card among the ten, two opponents: it goes to P0's hand.
    let mut ten = ["Forest"; 10];
    ten[4] = "Grizzly Bears";
    let (t, cards) = manifold(&ten, Some(4), None);
    assert_eq!(t.zone(cards[4]), Zone::Hand(P0));
    assert_eq!(t.hand_size(P0), 1);
    assert!(cards
        .iter()
        .filter(|c| **c != cards[4])
        .all(|c| t.zone(*c) == Zone::Library(P0)));
}

// --- Grasp of Fate and Battle at the Helvault ------------------------------------------

#[test]
fn grasp_of_fate_leaving_first_exiles_nothing() {
    cr!("610.3c");
    ruling!(
        "Grasp of Fate",
        "If Grasp of Fate leaves the battlefield before its triggered ability resolves, no nonland permanents will be exiled."
    );
    supported("Grasp of Fate");
    let mut t = TestGame::new(3);
    let b1 = t.battlefield(P1, "Grizzly Bears");
    let b2 = t.battlefield(P2, "Hill Giant");
    t.answer_targets(P0, &[obj(b1)]);
    t.answer_targets(P0, &[obj(b2)]);
    let grasp = crate::r_s05_common::enter(&mut t, P0, "Grasp of Fate");
    assert_eq!(t.stack_len(), 1);
    kill(&mut t, grasp);
    t.resolve_all();
    assert!(t.on_battlefield(b1) && t.on_battlefield(b2));
}

/// Grasp of Fate (P0) exiles P1's Grizzly Bears and P2's Hill Giant. Returns the game and
/// the Grasp.
fn grasped() -> (TestGame, ObjectId) {
    supported("Grasp of Fate");
    let mut t = TestGame::new(3);
    let b1 = t.battlefield(P1, "Grizzly Bears");
    let b2 = t.battlefield(P2, "Hill Giant");
    t.answer_targets(P0, &[obj(b1)]);
    t.answer_targets(P0, &[obj(b2)]);
    let grasp = crate::r_s05_common::enter(&mut t, P0, "Grasp of Fate");
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears") && t.in_exile("Hill Giant"));
    (t, grasp)
}

#[test]
fn grasp_of_fate_returns_cards_immediately() {
    cr!("610.3a", "610.3");
    ruling!(
        "Grasp of Fate",
        "The exiled cards return to the battlefield immediately after Grasp of Fate leaves the battlefield. Nothing happens between the two events, including state-based actions."
    );
    let (mut t, grasp) = grasped();
    let g = t.g.current(grasp);
    t.g.destroy(g, None);
    t.g.flush_events();
    // No trigger, no priority: they're already back before anything else happens.
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
    assert_eq!(t.stack_len(), 0);
    t.settle();
    assert_eq!(t.stack_len(), 0, "the return isn't a triggered ability");
    let bears = t.named_on_battlefield("Grizzly Bears")[0];
    assert_eq!(t.obj(bears).controller, P1);
}

#[test]
fn grasp_of_fate_owner_leaving_the_game_returns_the_cards() {
    cr!("800.4a", "610.3a");
    ruling!(
        "Grasp of Fate",
        "In a multiplayer game, if Grasp of Fate's owner leaves the game, the exiled cards will return to the battlefield."
    );
    let (mut t, _) = grasped();
    t.g.lose_game(P0);
    t.g.flush_events();
    t.settle();
    let bears = t.named_on_battlefield("Grizzly Bears");
    let giant = t.named_on_battlefield("Hill Giant");
    assert_eq!(bears.len(), 1);
    assert_eq!(giant.len(), 1);
    assert_eq!(t.obj(bears[0]).controller, P1);
    assert_eq!(t.obj(giant[0]).controller, P2);
}

#[test]
fn battle_at_the_helvault_exiled_permanents_return_as_new_objects() {
    cr!("400.7", "704.5m", "704.5n", "610.3a");
    ruling!(
        "Battle at the Helvault",
        "Auras attached to the exiled permanents will be put into their owners' graveyards. Any Equipment will become unattached and remain on the battlefield. Any counters on the exiled permanents will cease to exist."
    );
    supported("Battle at the Helvault");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let aura = t.battlefield(P1, "Holy Strength");
    let eq = t.battlefield(P1, "Bonesplitter");
    t.g.attach(aura, obj(bears));
    t.g.attach(eq, obj(bears));
    crate::r_s13_common::add(&mut t, bears, "+1/+1", 2);
    // P0 controls no non-Saga nonland permanent: only P1's slot is asked.
    t.answer_targets(P0, &[obj(bears)]);
    let saga = crate::r_s05_common::enter(&mut t, P0, "Battle at the Helvault");
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    assert!(t.in_graveyard(P1, "Holy Strength"));
    assert!(t.on_battlefield(eq));
    assert_eq!(t.obj_now(eq).attached_to, None);
    // The Saga leaves: the Bears returns as a new object, no counters, nothing attached.
    kill(&mut t, saga);
    let back = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(back.len(), 1);
    assert_ne!(back[0], bears);
    assert_eq!(t.counters(back[0], "+1/+1"), 0);
    assert_eq!(t.pt(back[0]), (2, 2));
    assert_eq!(t.obj_now(eq).attached_to, None);
}

// --- Breena, the Demagogue --------------------------------------------------------------

/// Three players at the given life totals; P0 controls Breena (and a Grizzly Bears if
/// `bears`). P1 attacks `defender` with a Grizzly Bears.
fn breena(life: [i32; 3], defender: PlayerId, bears: bool) -> (TestGame, ObjectId, Option<ObjectId>) {
    supported("Breena, the Demagogue");
    let mut t = TestGame::new(3);
    for (i, l) in life.iter().enumerate() {
        t.g.players[i].life = *l;
    }
    let breena = t.battlefield(P0, "Breena, the Demagogue");
    let mine = bears.then(|| t.battlefield(P0, "Grizzly Bears"));
    let attacker = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::BeginningOfCombat);
    attack_with(&mut t, &[(attacker, pl(defender))]);
    (t, breena, mine)
}

#[test]
fn breena_triggers_when_an_opponent_attacks_a_richer_opponent() {
    cr!("603.4", "508.1m");
    ruling!(
        "Breena, the Demagogue",
        "If the attacking player is one of your opponents and has less life than the player being attacked, then the condition is satisfied."
    );
    // P1 (10 life) attacks P2 (20 life): P2 has more life than P1, another opponent.
    let (mut t, b, _) = breena([20, 10, 20], P2, false);
    assert_eq!(t.stack_len(), 1, "{}", t.dump_log());
    let hand = t.hand_size(P1);
    t.resolve_all();
    assert_eq!(t.hand_size(P1), hand + 1, "the attacking player draws");
    assert_eq!(t.counters(b, "+1/+1"), 2);
    // P1 (20) attacks P2 (10): the condition isn't satisfied.
    let (t, _, _) = breena([20, 20, 10], P2, false);
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn breena_counters_are_put_on_a_creature_chosen_on_resolution() {
    cr!("608.2c", "603.4");
    ruling!(
        "Breena, the Demagogue",
        "You choose which creature to put +1/+1 counters on as the ability is resolving. Once you make the choice, no player may take actions until the counters are placed."
    );
    let (mut t, b, mine) = breena([20, 10, 20], P2, true);
    let mine = mine.unwrap();
    assert_eq!(t.stack_len(), 1);
    let before = t.asked().len();
    t.answer_choose(P0, &[obj(mine)]);
    t.resolve_all();
    assert!(t.asked()[before..]
        .iter()
        .any(|(p, d)| *p == P0 && matches!(d, Decision::ChooseEntities { .. })));
    assert_eq!(t.counters(mine, "+1/+1"), 2);
    assert_eq!(t.counters(b, "+1/+1"), 0);

    ruling!(
        "Breena, the Demagogue",
        "Breena's triggered ability will trigger even if you control no creatures."
    );
    // Breena itself isn't a creature right now; P0 controls no creatures.
    supported("Breena, the Demagogue");
    let mut t = TestGame::new(3);
    t.g.players[1].life = 10;
    let b = t.battlefield(P0, "Breena, the Demagogue");
    modify_until_eot(&mut t, b, vec![Modification::RemoveTypes(vec![CardType::Creature])]);
    let attacker = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::BeginningOfCombat);
    attack_with(&mut t, &[(attacker, pl(P2))]);
    assert_eq!(t.stack_len(), 1, "it triggers anyway");
    let hand = t.hand_size(P1);
    t.resolve_all();
    assert_eq!(t.hand_size(P1), hand + 1);
}

// --- Deathreap Ritual -------------------------------------------------------------------

#[test]
fn deathreap_ritual_checks_deaths_this_turn_at_the_end_step() {
    cr!("603.4", "513.1");
    supported("Deathreap Ritual");
    ruling!(
        "Deathreap Ritual",
        "If a creature didn't die before a turn's end step begins, Deathreap Ritual's ability doesn't trigger at all. The creature may have died before Deathreap Ritual entered the battlefield, however."
    );
    ruling!(
        "Deathreap Ritual",
        "You draw one card when the ability resolves, not one card per creature that died during the turn."
    );
    // Two creatures die, then the Ritual enters: one draw at the end step.
    let mut t = TestGame::new(2);
    for _ in 0..2 {
        let b = t.battlefield(P1, "Grizzly Bears");
        kill(&mut t, b);
    }
    t.battlefield(P0, "Deathreap Ritual");
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1, "one card");
    // Nothing died: it doesn't trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Deathreap Ritual");
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 0);
}

// --- Roar of Resistance and Hydra Omnivore ----------------------------------------------

#[test]
fn roar_of_resistance_triggers_on_any_players_attack() {
    cr!("603.2", "508.1m");
    ruling!(
        "Roar of Resistance",
        "Roar of Resistance's last ability triggers whenever any creatures attack, not just whenever creatures you control attack."
    );
    supported("Roar of Resistance");
    // P1 attacks P2 (both P0's opponents): P0 may pay, and P1's attacker gets +2/+0.
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Roar of Resistance");
    t.lands(P0, "Mountain", 2);
    let a = t.battlefield(P1, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.set_step(P1, Step::BeginningOfCombat);
    attack_with(&mut t, &[(a, pl(P2))]);
    assert_eq!(t.stack_len(), 1, "it triggers on P1's attack");
    t.resolve_all();
    assert_eq!(t.pt(a), (4, 2), "attacking one of P0's opponents");
}

#[test]
fn hydra_omnivore_extra_damage_isnt_combat_damage() {
    cr!("510.2", "603.2");
    ruling!(
        "Hydra Omnivore",
        "The damage dealt by Hydra Omnivore as a result of its triggered ability is not combat damage (and doesn't cause the ability to trigger again)."
    );
    supported("Hydra Omnivore");
    let mut t = TestGame::new(4);
    let hydra = t.battlefield(P0, "Hydra Omnivore");
    t.advance_to(P0, Step::BeginningOfCombat);
    t.attack(&[(hydra, pl(P1))], &[]);
    t.resolve_all();
    // 8 combat damage to P1, 8 non-combat damage to each other opponent, once.
    assert_eq!(t.life(P1), 12);
    assert_eq!(t.life(P2), 12);
    assert_eq!(t.life(P3), 12);
    assert_eq!(t.life(P0), 20);
}

// --- Turbulent lands --------------------------------------------------------------------

#[test]
fn turbulent_lands_have_two_basic_land_types_but_arent_basic() {
    cr!("205.3i", "205.4c", "305.6");
    supported("Rampant Growth");
    for (land, snarl, types, line) in [
        ("Turbulent Fen", "Necroblossom Snarl", ["Swamp", "Forest"], 0),
        ("Turbulent Moor", "Shineshadow Snarl", ["Plains", "Swamp"], 1),
        ("Turbulent Springs", "Frostboil Snarl", ["Island", "Mountain"], 2),
        ("Turbulent Steppe", "Furycalm Snarl", ["Mountain", "Plains"], 3),
        ("Turbulent Wilderness", "Vineglimmer Snarl", ["Forest", "Island"], 4),
    ] {
        let _ = match line {
            0 => ruling!(
                "Turbulent Fen",
                "Unlike some other dual lands, Turbulent Fen has two basic land types. It's not basic, so effects that search for basic lands can't find it, but it does have the appropriate land types for effects such as that of Necroblossom Snarl"
            ),
            1 => ruling!(
                "Turbulent Moor",
                "Unlike some other dual lands, Turbulent Moor has two basic land types. It's not basic, so effects that search for basic lands can't find it, but it does have the appropriate land types for effects such as that of Shineshadow Snarl"
            ),
            2 => ruling!(
                "Turbulent Springs",
                "Unlike some other dual lands, Turbulent Springs has two basic land types. It's not basic, so effects that search for basic lands can't find it, but it does have the appropriate land types for effects such as that of Frostboil Snarl"
            ),
            3 => ruling!(
                "Turbulent Steppe",
                "Unlike some other dual lands, Turbulent Steppe has two basic land types. It's not basic, so effects that search for basic lands can't find it, but it does have the appropriate land types for effects such as that of Furycalm Snarl"
            ),
            _ => ruling!(
                "Turbulent Wilderness",
                "Unlike some other dual lands, Turbulent Wilderness has two basic land types. It's not basic, so effects that search for basic lands can't find it, but it does have the appropriate land types for effects such as that of Vineglimmer Snarl"
            ),
        };
        supported(land);
        supported(snarl);
        // Two basic land types, not basic.
        let mut t = TestGame::new(2);
        let l = t.battlefield(P0, land);
        for ty in types {
            assert!(
                t.obj(l).chars.subtypes.iter().any(|s| s.as_str() == ty),
                "{land} is a {ty}"
            );
        }
        assert!(!t.obj(l).chars.supertypes.contains(Supertype::Basic));
        // A search for a basic land card can't find it.
        let mut t = TestGame::new(2);
        let lib = t.library_top(P0, land);
        t.lands(P0, "Forest", 2);
        t.answer_choose(P0, &[obj(lib)]);
        let rg = t.hand(P0, "Rampant Growth");
        t.cast(P0, rg).go();
        t.resolve_all();
        assert!(!t.on_battlefield(lib), "{land} isn't basic");
        // Revealing it lets the snarl enter untapped.
        let mut t = TestGame::new(2);
        let c = t.hand(P0, land);
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[obj(c)]);
        let s = t.enter(P0, snarl);
        assert!(!t.obj(s).tapped, "{snarl} with {land} revealed");
    }
}
