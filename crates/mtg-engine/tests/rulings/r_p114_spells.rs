//! Rulings batch P114 — partner commanders that count spells and life over the turn
//! (Kraum, Alisaie, Alphinaud, Ludevic, Regna), change costs (Esior, Alisaie), care about
//! mana spent (Gilanra), hide a chosen card (The Seventh Doctor), or resolve without
//! interruption (Thrasios).

use crate::r_p114_common::*;
use crate::r_s01_common::{asked_since, watch};
use crate::r_s25_common::lands_for_cost;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::facedown::can_look_at;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

// --- Kraum, Ludevic's Opus: "Whenever an opponent casts their second spell each turn, draw
// a card." -------------------------------------------------------------------------------------

/// P1 (or another player) casts Opt ({U} instant: "Scry 1. Draw a card.").
fn opt(t: &mut TestGame, p: PlayerId) -> ObjectId {
    cast_instant(t, p, "Opt", &[])
}

#[test]
fn kraum_counts_spells_cast_before_it_was_on_the_battlefield() {
    cr!("603.2", "603.10");
    ruling!(
        "Kraum, Ludevic's Opus",
        "Kraum's ability looks at the entire turn to determine which spell is a player's second spell. It doesn't matter whether Kraum was on the battlefield when the first spell was cast."
    );
    supported("Kraum, Ludevic's Opus");
    let mut t = TestGame::new(2);
    opt(&mut t, P1);
    t.resolve_all();
    t.battlefield(P0, "Kraum, Ludevic's Opus");
    let hand = t.hand_size(P0);
    opt(&mut t, P1);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn kraum_triggers_once_per_opponent_each_turn() {
    cr!("603.2");
    ruling!(
        "Kraum, Ludevic's Opus",
        "Kraum's triggered ability can trigger only once each turn for each opponent."
    );
    ruling!(
        "Kraum, Ludevic's Opus",
        "One opponent must cast two spells for Kraum's triggered ability to trigger. Two spells from different opponents won't cause it to trigger."
    );
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Kraum, Ludevic's Opus");
    let hand = t.hand_size(P0);
    // P1 and P2 cast one spell each: no second spell.
    opt(&mut t, P1);
    t.resolve_all();
    opt(&mut t, P2);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
    // P1's second and third spells: one card.
    opt(&mut t, P1);
    t.resolve_all();
    opt(&mut t, P1);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    // P2's second spell: another.
    opt(&mut t, P2);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
}

#[test]
fn kraum_resolves_whatever_happened_to_the_spells_and_before_the_second() {
    cr!("603.3", "405.5");
    ruling!(
        "Kraum, Ludevic's Opus",
        "Kraum's triggered ability will resolve regardless of whether the first or second spell that a given opponent cast that turn has resolved, was countered, or is still on the stack. Notably, the ability will always resolve before the second spell resolves."
    );
    for first in ["resolved", "countered", "on the stack"] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Kraum, Ludevic's Opus");
        let hand = t.hand_size(P0);
        let one = opt(&mut t, P1);
        match first {
            "resolved" => t.resolve_all(),
            "countered" => counter(&mut t, one),
            _ => {}
        }
        let two = opt(&mut t, P1);
        let below = t.stack_len();
        assert_eq!(t.g.stack.last().map(|s| *s != two), Some(true), "{first}");
        t.resolve();
        assert_eq!(t.hand_size(P0), hand + 1, "{first}");
        assert_eq!(t.stack_len(), below - 1);
        assert!(t.g.stack.contains(&two), "the second spell hasn't resolved");
        // The second spell is countered too: the card is still drawn (already was).
        counter(&mut t, two);
        t.resolve_all();
        assert_eq!(t.hand_size(P0), hand + 1, "{first}");
    }
}

// --- Alisaie and Alphinaud Leveilleur ------------------------------------------------------

#[test]
fn alisaie_and_alphinaud_count_spells_cast_before_they_entered() {
    cr!("601.2f", "603.2");
    ruling!(
        "Alisaie Leveilleur",
        "Spells that were cast before Alisaie Leveilleur entered count. If Alisaie Leveilleur was the first spell you cast this turn, the next spell you cast this turn is your second spell."
    );
    ruling!(
        "Alphinaud Leveilleur",
        "Spells that were cast before Alphinaud Leveilleur entered count. If Alphinaud Leveilleur was the first spell you cast this turn, the next spell you cast this turn is your second spell."
    );
    supported("Alisaie Leveilleur");
    supported("Alphinaud Leveilleur");
    for (name, cast_it) in [
        ("Alisaie Leveilleur", true),
        ("Alisaie Leveilleur", false),
        ("Alphinaud Leveilleur", true),
        ("Alphinaud Leveilleur", false),
    ] {
        let mut t = TestGame::new(2);
        if cast_it {
            // It's the first spell.
            lands_for_cost(&mut t, P0, name);
            let c = t.hand(P0, name);
            t.cast(P0, c).go();
            t.resolve_all();
        } else {
            // A spell was cast before it was on the battlefield.
            lands_for_cost(&mut t, P0, "Grizzly Bears");
            let bears = t.hand(P0, "Grizzly Bears");
            t.cast(P0, bears).go();
            t.resolve_all();
            t.battlefield(P0, name);
        }
        let hand = t.hand_size(P0);
        // The second spell: Hill Giant ({3}{R}).
        let giant = t.hand(P0, "Hill Giant");
        if name.starts_with("Alisaie") {
            // {2} less: {1}{R}.
            t.lands(P0, "Mountain", 1);
            t.lands(P0, "Wastes", 1);
            assert!(t.cast(P0, giant).try_go().is_ok(), "{name} {cast_it}");
        } else {
            t.lands(P0, "Mountain", 1);
            t.lands(P0, "Wastes", 3);
            t.cast(P0, giant).go();
            t.settle();
            t.resolve_all();
            assert_eq!(t.hand_size(P0), hand + 1, "{name} {cast_it}: drew one");
        }
    }
}

#[test]
fn alisaie_reduces_only_generic_mana() {
    cr!("601.2f", "118.7");
    ruling!(
        "Alisaie Leveilleur",
        "The cost reduction applies only to generic mana in the total cost of the second spell you cast each turn."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Alisaie Leveilleur");
    lands_for_cost(&mut t, P0, "Grizzly Bears");
    let bears = t.hand(P0, "Grizzly Bears");
    t.cast(P0, bears).go();
    t.resolve_all();
    // Lightning Bolt ({R}) as the second spell still costs {R}.
    let bolt = t.hand(P0, "Lightning Bolt");
    assert!(t
        .cast(P0, bolt)
        .target(Entity::Player(P1))
        .try_go()
        .is_err());
    let bolt = t.g.current(bolt);
    t.lands(P0, "Mountain", 1);
    t.cast(P0, bolt).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}

// --- Ludevic, Necro-Alchemist: "At the beginning of each player's end step, that player may
// draw a card if a player other than you lost life this turn." -----------------------------

/// Advances to `active`'s end step and resolves it with every "may draw" answered yes.
/// Returns how many cards the active player drew.
fn ludevic_end_step(t: &mut TestGame, active: PlayerId) -> usize {
    t.advance_to(active, Step::End);
    let hand = t.hand_size(active);
    t.settle();
    t.answer_yes(active, true);
    t.resolve_all();
    t.hand_size(active) - hand
}

#[test]
fn ludevic_triggers_each_end_step_and_checks_as_it_resolves() {
    cr!("603.2", "608.2c", "513.1");
    ruling!(
        "Ludevic, Necro-Alchemist",
        "Ludevic's triggered ability triggers at the beginning of each player's end step, including yours, even if no player has lost life that turn. Whether or not a player has lost life is checked only as the triggered ability resolves."
    );
    ruling!(
        "Ludevic, Necro-Alchemist",
        "Ludevic's triggered ability will allow the player to draw a card if any player other than Ludevic's controller, including the player whose turn it is, lost life this turn."
    );
    supported("Ludevic, Necro-Alchemist");
    // No life lost: it triggers, and P1 loses life in response.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ludevic, Necro-Alchemist");
    let hand = t.hand_size(P0);
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(crate::r_s01_common::triggers_on_stack(&t, "lost life"), 1);
    lose(&mut t, P1, 1);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    // On P1's turn, P1 (the active player, not Ludevic's controller) loses life: P1 draws.
    t.advance_to(P1, Step::Upkeep);
    lose(&mut t, P1, 2);
    assert_eq!(ludevic_end_step(&mut t, P1), 1);
    // Only Ludevic's controller lost life: no card.
    t.advance_to(P0, Step::Upkeep);
    lose(&mut t, P0, 2);
    assert_eq!(ludevic_end_step(&mut t, P0), 0);
}

#[test]
fn ludevic_looks_at_the_whole_turn_and_life_lost_not_net() {
    cr!("119.3", "603.2");
    ruling!(
        "Ludevic, Necro-Alchemist",
        "Ludevic's ability looks at the entire turn to determine whether the player may draw a card or not. It doesn't matter whether Ludevic was on the battlefield when the opponent lost life."
    );
    ruling!(
        "Ludevic, Necro-Alchemist",
        "Ludevic's ability checks only if another player lost life during the turn, not whether that player's life total decreased over the course of the turn. For example, if a player other than Ludevic's controller lost 2 life and then gained 8 life this turn, the player whose turn it is may draw a card."
    );
    ruling!(
        "Ludevic, Necro-Alchemist",
        "Only one card may be drawn no matter how much life was lost or how many players lost life."
    );
    let mut t = TestGame::new(3);
    lose(&mut t, P1, 2);
    gain(&mut t, P1, 8);
    lose(&mut t, P2, 5);
    assert_eq!(t.life(P1), 26);
    t.battlefield(P0, "Ludevic, Necro-Alchemist");
    assert_eq!(ludevic_end_step(&mut t, P0), 1);
}

// --- Regna, the Redeemer: "At the beginning of each end step, if your team gained life this
// turn, create two 1/1 white Warrior creature tokens." ------------------------------------

#[test]
fn regna_checks_the_whole_team_and_the_whole_turn() {
    cr!("603.4", "810.2", "119.10");
    ruling!(
        "Regna, the Redeemer",
        "Regna's last ability checks if any player on your team gained life at any point during the turn. It doesn't matter if the player also lost life or whether the team's life total is greater than it was at the beginning of the turn. It also doesn't matter whether Regna was on the battlefield when the life gain happened."
    );
    ruling!(
        "Regna, the Redeemer",
        "Regna's last ability has you create two tokens, regardless of which player on your team gained life, how many players on your team gained life, or how much life was gained."
    );
    supported("Regna, the Redeemer");
    for both in [false, true] {
        let mut t = two_headed_giant();
        let before = t.life(P0);
        // P0's teammate gains 3 and loses 5 before Regna is on the battlefield.
        gain(&mut t, P1, 3);
        lose(&mut t, P1, 5);
        if both {
            gain(&mut t, P0, 10);
        }
        assert!(t.life(P0) < before + if both { 10 } else { 0 });
        t.battlefield(P0, "Regna, the Redeemer");
        t.advance_to(P0, Step::End);
        t.resolve_all();
        assert_eq!(tokens_named(&t, P0, "Warrior").len(), 2, "both {both}");
    }
}

#[test]
fn regna_doesnt_trigger_for_life_gained_during_the_end_step() {
    cr!("603.4", "513.1");
    ruling!(
        "Regna, the Redeemer",
        "If your team didn't gain life during the turn before the end step begins, Regna's last ability won't trigger at all. Gaining life during the end step won't cause the ability to trigger."
    );
    let mut t = two_headed_giant();
    t.battlefield(P0, "Regna, the Redeemer");
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 0);
    gain(&mut t, P1, 4);
    t.resolve_all();
    t.advance_to(P2, Step::Upkeep);
    assert!(tokens_named(&t, P0, "Warrior").is_empty());
}

// --- Esior, Wardwing Familiar: "Spells your opponents cast that target one or more
// commanders you control cost {3} more to cast." ------------------------------------------

/// A Commander game where P0 controls Esior and Kediss as commanders, and P1 has `lands`
/// Mountains (and the permanents `extra`).
fn esior_game(lands: usize, extra: &[&str]) -> (TestGame, ObjectId, ObjectId) {
    let mut t = commander_game_n(2);
    let esior = commander_on_battlefield(&mut t, P0, "Esior, Wardwing Familiar");
    let kediss = commander_on_battlefield(&mut t, P0, "Kediss, Emberclaw Familiar");
    t.lands(P1, "Mountain", lands);
    for e in extra {
        t.battlefield(P1, e);
    }
    (t, esior, kediss)
}

#[test]
fn esior_adds_three_once_for_two_commanders() {
    cr!("601.2f", "118.7");
    ruling!(
        "Esior, Wardwing Familiar",
        "Spells that target more than one commander you control cost only {3} more to cast."
    );
    // Electrolyze ({1}{U}{R} instant): 2 damage divided between Esior and Kediss, 1 each:
    // {4}{U}{R} in all.
    supported("Electrolyze");
    for (lands, ok) in [(5, false), (6, true)] {
        let (mut t, esior, kediss) = esior_game(1, &[]);
        t.lands(P1, "Island", 1);
        t.lands(P1, "Wastes", lands - 2);
        t.answer_targets(P1, &[ent(esior), ent(kediss)]);
        t.answer(P1, DecisionKind::Divide, Answer::Numbers(vec![1, 1]));
        let card = t.hand(P1, "Electrolyze");
        t.g.turn.priority = Some(P1);
        let r =
            t.g.cast_spell(P1, card, mtg_engine::object::CastMethod::Normal);
        assert_eq!(r.is_ok(), ok, "{lands} lands: {r:?}");
        if ok {
            assert_eq!(crate::r_s25_common::targets_of(&t, r.unwrap()).len(), 2);
        }
    }
}

#[test]
fn esior_increase_applies_before_reductions_and_mana_value_is_unchanged() {
    cr!("601.2f", "202.3");
    ruling!(
        "Esior, Wardwing Familiar",
        "To determine the total cost of an opponent's spell that targets a commander you control, start with the mana cost or alternative cost that player is paying, add any cost increases (such as that of Esior's middle ability), then apply any cost reductions. The mana value of the spell is determined only by its mana cost, no matter what the total cost to cast the spell was."
    );
    // Goblin Electromancer: "Instant and sorcery spells you cast cost {1} less to cast."
    // Lightning Bolt ({R}) at Esior: {R} + {3} − {1} = {2}{R} (reducing first would leave
    // {3}{R}).
    supported("Goblin Electromancer");
    for (lands, ok) in [(2, false), (3, true)] {
        let (mut t, esior, _) = esior_game(lands, &["Goblin Electromancer"]);
        let card = t.hand(P1, "Lightning Bolt");
        t.answer_targets(P1, &[ent(esior)]);
        t.g.turn.priority = Some(P1);
        let r =
            t.g.cast_spell(P1, card, mtg_engine::object::CastMethod::Normal);
        assert_eq!(r.is_ok(), ok, "{lands} lands");
        if let Ok(spell) = r {
            assert_eq!(crate::r_s26_common::mv(&mut t, spell), 1);
        }
    }
}

// --- Gilanra, Caller of Wirewood: "{T}: Add {G}. When you spend this mana to cast a spell
// with mana value 6 or greater, draw a card." -------------------------------------------

/// P0 casts Walking Ballista ({X}{X}) for `x` with Gilanra's mana and `wastes` Wastes
/// (and Mana Reflection if `reflection`). Returns the cards drawn.
fn gilanra_ballista(x: i64, wastes: usize, reflection: bool) -> usize {
    supported("Gilanra, Caller of Wirewood");
    let mut t = TestGame::new(2);
    let gilanra = t.battlefield(P0, "Gilanra, Caller of Wirewood");
    if reflection {
        t.battlefield(P0, "Mana Reflection");
    }
    t.lands(P0, "Wastes", wastes);
    let ballista = t.hand(P0, "Walking Ballista");
    let hand = t.hand_size(P0);
    t.cast(P0, ballista).x(x).go();
    assert!(t.obj(gilanra).tapped, "Gilanra's mana was spent");
    t.resolve_all();
    t.hand_size(P0) + 1 - hand
}

#[test]
fn gilanra_uses_x_for_the_mana_value() {
    cr!("202.3e", "107.3");
    ruling!(
        "Gilanra, Caller of Wirewood",
        "For a spell with {X} in its mana cost, use the value chosen for X to determine the spell's mana value."
    );
    assert_eq!(gilanra_ballista(3, 5, false), 1, "X=3: mana value 6");
    assert_eq!(gilanra_ballista(2, 3, false), 0, "X=2: mana value 4");
}

#[test]
fn gilanra_with_mana_reflection_draws_for_each_mana_spent() {
    cr!("106.6a", "603.7");
    ruling!(
        "Gilanra, Caller of Wirewood",
        "If an effect causes Gilanra's ability to produce more than one mana (such as that of Mana Reflection), the delayed triggered ability will be created for each of those mana, and you'll draw a card each time one of those mana is spent to cast a spell with mana value 6 or greater. If you spend two of those mana to cast a single spell with mana value 6 or greater, you'll draw two cards."
    );
    supported("Mana Reflection");
    // {G}{G} from Gilanra and {C}{C}{C}{C} from two Wastes pay for X=3.
    assert_eq!(gilanra_ballista(3, 2, true), 2);
}

// --- The Seventh Doctor -------------------------------------------------------------------

#[test]
fn the_seventh_doctor_x_is_zero() {
    cr!("107.3b", "601.2b");
    ruling!(
        "The Seventh Doctor",
        "If the card you chose has an {X} in its mana cost, the value of X is 0."
    );
    supported("The Seventh Doctor");
    let mut t = TestGame::new(2);
    let doctor = t.battlefield(P0, "The Seventh Doctor");
    // Endless One ({X}): "This creature enters with X +1/+1 counters on it." Its mana
    // value in hand is 0, not greater than 0 artifacts: P1 guesses "greater" (wrong).
    let one = t.hand(P0, "Endless One");
    t.answer_choose(P0, &[ent(one)]);
    t.answer(P1, DecisionKind::Option, Answer::Index(0));
    t.answer_yes(P0, true);
    attack_with(&mut t, &[(doctor, Entity::Player(P1))]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Endless One"), "a 0/0 with no counters");
    assert!(!t
        .asked()
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseX { .. })));
}

#[test]
fn the_seventh_doctor_keeps_the_chosen_card_hidden_unless_cast() {
    cr!("608.2c", "402.3");
    ruling!(
        "The Seventh Doctor",
        "To choose a card in your hand for The Seventh Doctor's ability, take a card in your hand and separate it from the others, face down. Then the defending player guesses whether the mana value of that card is greater than the number of artifacts you control. You do not have to reveal whether they guess right or wrong unless you are casting the card."
    );
    let mut t = TestGame::new(2);
    let doctor = t.battlefield(P0, "The Seventh Doctor");
    let bears = t.hand(P0, "Grizzly Bears");
    t.answer_choose(P0, &[ent(bears)]);
    // P1 guesses "greater" (right: 2 > 0), without seeing the card.
    t.answer(P1, DecisionKind::Option, Answer::Index(0));
    let seen = watch(
        &mut t,
        P1,
        |d| matches!(d, Decision::ChooseOption { .. }),
        |g| {
            g.find_in_zone(Zone::Hand(P0), "Grizzly Bears")
                .iter()
                .any(|b| can_look_at(g, P1, *b))
        },
    );
    attack_with(&mut t, &[(doctor, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(*seen.lock().unwrap(), vec![false], "P1 guessed blind");
    // Not cast: P0 investigates, and the card stays hidden in hand.
    assert_eq!(tokens_named(&t, P0, "Clue").len(), 1);
    assert_eq!(t.zone(bears), Zone::Hand(P0));
    assert!(!can_look_at(&t.g, P1, bears));
}

// --- Thrasios, Triton Hero: "{4}: Scry 1, then reveal the top card of your library. If it's
// a land card, put it onto the battlefield tapped. Otherwise, draw a card." -----------------

#[test]
fn thrasios_resolves_without_interruption_and_draws_the_revealed_card() {
    cr!("608.2", "117.3");
    ruling!(
        "Thrasios, Triton Hero",
        "No player may take another action while you're resolving the activated ability."
    );
    ruling!(
        "Thrasios, Triton Hero",
        "The nonland card you reveal will be the card you draw."
    );
    supported("Thrasios, Triton Hero");
    for land in [false, true] {
        let mut t = TestGame::new(2);
        let thrasios = t.battlefield(P0, "Thrasios, Triton Hero");
        t.lands(P0, "Wastes", 4);
        let top = t.library_top(P0, if land { "Forest" } else { "Grizzly Bears" });
        let hand = t.hand_size(P0);
        // Every time either player gets priority: (objects on the stack, P0's hand).
        let look = |g: &mtg_engine::game::Game| (g.stack.len(), g.player(P0).hand.len());
        let is_priority = |d: &Decision| matches!(d, Decision::Priority { .. });
        let p0 = watch(&mut t, P0, is_priority, look);
        let p1 = watch(&mut t, P1, is_priority, look);
        let from = t.asked().len();
        t.activate(P0, thrasios, 0, &[]).expect("Thrasios");
        let ok = t.g.run_until(10_000, |g| g.stack.is_empty());
        assert!(ok);
        // The only decision during the resolution was P0's scry.
        assert!(asked_since(&t, from)
            .iter()
            .any(|(p, d)| *p == P0 && matches!(d, Decision::Scry { .. })));
        let after = if land { hand } else { hand + 1 };
        for seen in p0.lock().unwrap().iter().chain(p1.lock().unwrap().iter()) {
            assert!(
                *seen == (1, hand) || *seen == (0, after),
                "priority in the middle of the resolution: {seen:?}"
            );
        }
        if land {
            assert!(t.on_battlefield(t.g.current(top)));
            assert!(t.obj_now(top).tapped);
        } else {
            assert_eq!(t.zone(t.g.current(top)), Zone::Hand(P0));
        }
        assert_eq!(t.hand_size(P0), after);
    }
}
