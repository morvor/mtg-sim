//! Rulings batch P078 — group hug: Abyssal Persecutor (opponents can't lose), All
//! Hallow's Eve, Grismold, the Dreadsower, mana doublers (Mana Flare, Heartbeat of Spring,
//! Lavaleaper), Conundrum Sphinx, Wild Evocation, votes (Erestor of the Council, Sail into
//! the West), "each opponent may" (Shah of Naar Isle, Iwamori of the Open Fist), Two-Headed
//! Giant teams (Centaur Peacemaker, Aku Djinn), Fecundity, Oviya, Greener Pastures,
//! Elephant Resurgence, Party Dude, Planar Birth and Yurlok of Scorch Thrash.

use crate::r_p078_common::*;
use crate::r_s01_common::{attack_with, supported, tokens, watch};
use crate::r_s02_common::{can_activate, destroy};
use crate::r_s20_common::tap_for_mana;
use crate::r_s25_common::cast_new;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::events::Event;
use mtg_engine::game::GameResult;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

// ---------------------------------------------------------------------------------
// Abyssal Persecutor: "You can't win the game and your opponents can't lose the game."
// ---------------------------------------------------------------------------------

#[test]
fn abyssal_persecutor_lets_an_opponents_life_reach_zero_without_losing() {
    cr!("104.3b", "704.5a", "101.2");
    ruling!(
        "Abyssal Persecutor",
        "Abyssal Persecutor won't preclude an opponent's life total from reaching 0 or less. It will just preclude that player from losing the game as a result."
    );
    supported("Abyssal Persecutor");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Abyssal Persecutor");
    t.g.players[1].life = 2;
    cast_new(&mut t, P0, "Lightning Bolt", &[pl(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), -1);
    assert!(!t.has_lost(P1));
    assert_eq!(t.g.result, None);
}

#[test]
fn abyssal_persecutor_doesnt_stop_a_draw() {
    cr!("104.4c");
    ruling!(
        "Abyssal Persecutor",
        "Effects that say the game is a draw, such as the _Legends_(TM) card Divine Intervention, are not affected by Abyssal Persecutor."
    );
    supported("Divine Intervention");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Abyssal Persecutor");
    let di = t.battlefield(P0, "Divine Intervention");
    t.g.objects[di.0 as usize]
        .counters
        .insert("intervention".into(), 1);
    to_upkeep(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.g.result, Some(GameResult::Draw));
}

#[test]
fn abyssal_persecutor_doesnt_let_a_player_pay_more_life_than_they_have() {
    cr!("119.4", "119.4b");
    ruling!(
        "Abyssal Persecutor",
        "Even though your opponents can't lose the game, a player can't pay an amount of life that's greater than their life total. If a player's life total is 0 or less, that player can't pay life at all, with one exception: a player may always pay 0 life."
    );
    supported("Toxic Deluge");
    supported("Polluted Delta");
    // At 1 life, P1 can't pay 2 life, but can pay 1.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Abyssal Persecutor");
    t.set_step(P1, Step::PrecombatMain);
    t.g.players[1].life = 1;
    let deluge = t.hand(P1, "Toxic Deluge");
    add_mana(&mut t, P1, &[mtg_engine::mana::ManaType::B], 3);
    assert!(t.cast(P1, deluge).x(2).try_go().is_err());
    assert!(t.cast(P1, deluge).x(1).try_go().is_ok());
    t.resolve_all();
    assert_eq!(t.life(P1), 0);
    assert!(!t.has_lost(P1));
    // At 0 life: no life can be paid (Polluted Delta's 1 life), but 0 life can.
    let delta = t.battlefield(P1, "Polluted Delta");
    assert!(!can_activate(&mut t, P1, delta));
    let deluge = t.hand(P1, "Toxic Deluge");
    add_mana(&mut t, P1, &[mtg_engine::mana::ManaType::B], 3);
    assert!(t.cast(P1, deluge).x(0).try_go().is_ok());
    t.resolve_all();
    assert_eq!(t.life(P1), 0);
}

#[test]
fn abyssal_persecutor_leaving_makes_a_dead_opponent_lose_at_once() {
    cr!("104.3b", "704.3", "704.5a");
    ruling!(
        "Abyssal Persecutor",
        "If Abyssal Persecutor leaves the battlefield while an opponent has 0 or less life, that opponent will lose the game as a state-based action. No player can respond between the time Abyssal Persecutor leaves the battlefield and the time that player loses the game."
    );
    let mut t = TestGame::new(2);
    let ap = t.battlefield(P0, "Abyssal Persecutor");
    t.g.players[1].life = -3;
    t.settle();
    assert!(!t.has_lost(P1));
    let from = t.asked().len();
    destroy(&mut t, ap);
    assert!(t.has_lost(P1));
    let priority = deciders_since(&t, from, |d| matches!(d, Decision::Priority { .. }));
    assert!(priority.is_empty(), "no one could respond");
}

#[test]
fn abyssal_persecutor_stops_every_way_to_win_or_lose() {
    cr!("104.3b", "104.3c", "104.3d", "104.3e", "104.2b", "101.2");
    ruling!(
        "Abyssal Persecutor",
        "No game effect can cause you to win the game or cause any opponent to lose the game while you control Abyssal Persecutor. It doesn't matter whether an opponent has 0 or less life, an opponent is forced to draw a card while their library is empty, an opponent has ten or more poison counters, an opponent is dealt combat damage by Phage the Untouchable, you control Felidar Sovereign and have 40 or more life, or so on. You keep playing."
    );
    supported("Phage the Untouchable");
    supported("Felidar Sovereign");
    let fresh = || {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Abyssal Persecutor");
        t
    };
    // 0 or less life.
    let mut t = fresh();
    t.g.players[1].life = 0;
    t.settle();
    assert!(!t.has_lost(P1) && t.g.result.is_none(), "0 life");
    // Drawing from an empty library.
    let mut t = fresh();
    t.g.players[1].library.clear();
    t.g.draw_cards(P1, 1);
    t.settle();
    assert!(!t.has_lost(P1) && t.g.result.is_none(), "empty library");
    // Ten poison counters.
    let mut t = fresh();
    t.g.add_counters(pl(P1), "poison", 10, None);
    t.settle();
    assert!(!t.has_lost(P1) && t.g.result.is_none(), "poison");
    // Combat damage from Phage the Untouchable.
    let mut t = fresh();
    let phage = t.battlefield(P0, "Phage the Untouchable");
    t.attack(&[(phage, pl(P1))], &[]);
    t.settle();
    assert_eq!(t.life(P1), 16);
    assert!(!t.has_lost(P1) && t.g.result.is_none(), "Phage");
    // Felidar Sovereign with 40 life.
    let mut t = fresh();
    t.battlefield(P0, "Felidar Sovereign");
    t.g.players[0].life = 40;
    to_upkeep(&mut t, P0);
    t.resolve_all();
    assert!(!t.g.player(P0).has_won && t.g.result.is_none(), "Felidar");
}

// ---------------------------------------------------------------------------------
// All Hallow's Eve
// ---------------------------------------------------------------------------------

#[test]
fn all_hallows_eve_only_exiles_itself_with_two_scream_counters() {
    cr!("608.2", "122.1");
    ruling!(
        "All Hallow's Eve",
        "The only thing that happens when All Hallow's Eve resolves is that it's exiled with two scream counters on it."
    );
    supported("All Hallow's Eve");
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P1, "Hill Giant");
    let eve = cast_new(&mut t, P0, "All Hallow's Eve", &[]);
    t.resolve_all();
    let eve = t.g.current(eve);
    assert_eq!(t.zone(eve), Zone::Exile);
    assert_eq!(t.counters(eve, "scream"), 2);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert!(t.in_graveyard(P1, "Hill Giant"));
    assert!(t
        .g
        .battlefield
        .iter()
        .all(|id| !t.g.obj(*id).is(CardType::Creature)));
}

#[test]
fn all_hallows_eve_triggers_only_from_exile_with_a_scream_counter() {
    cr!("603.4", "113.6");
    ruling!(
        "All Hallow's Eve",
        "All Hallow's Eve's triggered ability functions from the exile zone. This ability has an \"intervening 'if' clause.\" It won't trigger at all unless All Hallow's Eve is exiled and has a scream counter on it (which can happen only if it resolves as a spell)."
    );
    // Exiled without scream counters: nothing triggers.
    let mut t = TestGame::new(2);
    t.exile(P0, "All Hallow's Eve");
    to_upkeep(&mut t, P0);
    t.settle();
    assert_eq!(t.stack_len(), 0);
    // In the graveyard: nothing either.
    let mut t = TestGame::new(2);
    t.graveyard(P0, "All Hallow's Eve");
    to_upkeep(&mut t, P0);
    t.settle();
    assert_eq!(t.stack_len(), 0);
    // Cast and resolved: exiled with scream counters, it triggers from exile.
    let mut t = TestGame::new(2);
    let eve = cast_new(&mut t, P0, "All Hallow's Eve", &[]);
    t.resolve_all();
    to_upkeep(&mut t, P0);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.counters(t.g.current(eve), "scream"), 1);
}

#[test]
fn all_hallows_eve_lets_you_sacrifice_creatures_before_the_last_counter_goes() {
    cr!("603.3", "117.3a");
    ruling!(
        "All Hallow's Eve",
        "Since you can respond to triggered abilities, it is legal to sacrifice creatures using some spell or ability prior to resolving the ability that removes the final counter from All Hallow's Eve."
    );
    supported("Viscera Seer");
    let mut t = TestGame::new(2);
    let eve = cast_new(&mut t, P0, "All Hallow's Eve", &[]);
    t.resolve_all();
    let eve = t.g.current(eve);
    t.g.objects[eve.0 as usize]
        .counters
        .insert("scream".into(), 1);
    let seer = t.battlefield(P0, "Viscera Seer");
    let bears = t.battlefield(P0, "Grizzly Bears");
    to_upkeep(&mut t, P0);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    // In response, sacrifice Grizzly Bears.
    t.answer_choose(P0, &[obj(bears)]);
    t.activate(P0, seer, 0, &[]).unwrap();
    t.resolve();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    t.resolve_all();
    assert!(t.in_graveyard(P0, "All Hallow's Eve"));
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

// ---------------------------------------------------------------------------------
// Grismold, the Dreadsower
// ---------------------------------------------------------------------------------

#[test]
fn grismolds_plants_are_owned_by_whoever_created_them() {
    cr!("111.2");
    ruling!(
        "Grismold, the Dreadsower",
        "Each Plant token is owned by the player who created it. You don't own them all."
    );
    supported("Grismold, the Dreadsower");
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Grismold, the Dreadsower");
    t.advance_to(P0, Step::End);
    t.resolve_all();
    for p in [P0, P1, P2] {
        let plants = tokens(&t, p);
        assert_eq!(plants.len(), 1, "{p:?}");
        assert_eq!(t.g.obj(plants[0]).owner, p);
    }
}

#[test]
fn grismold_dying_with_a_token_gets_no_counter_in_time() {
    cr!("603.10a", "704.3");
    ruling!(
        "Grismold, the Dreadsower",
        "If Grismold is dealt lethal damage at the same time as a creature token, Grismold won't receive a counter from its ability in time to save it."
    );
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Grismold, the Dreadsower");
    t.advance_to(P0, Step::End);
    t.resolve_all();
    t.set_step(P0, Step::PrecombatMain);
    let quake = t.hand(P0, "Earthquake");
    t.lands(P0, "Mountain", 4);
    t.cast(P0, quake).x(3).go();
    t.resolve();
    // Both died; Grismold's ability triggered for the Plants but can't save it.
    assert!(t.in_graveyard(P0, "Grismold, the Dreadsower"));
    assert!(t.stack_len() >= 1);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grismold, the Dreadsower"));
    assert_eq!(t.counters(t.g.current(g), "+1/+1"), 0);
}

#[test]
fn grismold_sees_a_token_die_as_its_controller_loses() {
    cr!("603.10a", "704.3", "800.4a");
    ruling!(
        "Grismold, the Dreadsower",
        "In a multiplayer game, if a creature token dies at the same time its controller loses the game, Grismold's last ability triggers."
    );
    let mut t = TestGame::new(3);
    let g = t.battlefield(P0, "Grismold, the Dreadsower");
    t.advance_to(P0, Step::End);
    t.resolve_all();
    t.set_step(P0, Step::PrecombatMain);
    // P2 is at 1 life with a 1/1 Plant: Earthquake for 1 kills both at once (and only P2's
    // and P1's Plants among the creatures).
    t.g.players[2].life = 1;
    let quake = t.hand(P0, "Earthquake");
    t.lands(P0, "Mountain", 2);
    t.cast(P0, quake).x(1).go();
    t.resolve_all();
    assert!(t.has_lost(P2));
    assert!(t.on_battlefield(g));
    // Three Plants died (P0's, P1's and P2's): three counters.
    assert_eq!(t.counters(g, "+1/+1"), 3);
}

// ---------------------------------------------------------------------------------
// Mana doublers
// ---------------------------------------------------------------------------------

/// P0 controls `doubler` and an Unclaimed Territory (Elf) or Cavern of Souls (Elf), taps
/// it for red with its restricted ability; returns the game.
fn doubled_restricted_mana(doubler: &str, land: &str) -> TestGame {
    let mut t = TestGame::new(2);
    t.battlefield(P0, doubler);
    let i = mtg_engine::types::subtype_lists()
        .creature
        .iter()
        .position(|s| s == "Elf")
        .expect("creature type");
    t.answer(P0, DecisionKind::Option, Answer::Index(i));
    let land = t.enter(P0, land);
    t.settle();
    // Red.
    t.answer(P0, DecisionKind::Any, Answer::Index(3));
    assert!(tap_for_mana(&mut t, P0, land, "any color"));
    t
}

#[test]
fn mana_flare_and_heartbeat_add_mana_without_the_lands_restrictions_or_riders() {
    cr!("106.6", "106.12a", "605.1b");
    ruling!(
        "Mana Flare",
        "Mana Flare doesn't care about any restrictions or riders your lands put on the mana they produce, such as those of Unclaimed Territory and Cavern of Souls. It just produces one mana of the appropriate type, with no restrictions or riders."
    );
    ruling!(
        "Heartbeat of Spring",
        "Heartbeat of Spring doesn't care about any restrictions or riders your lands put on the mana they produce, such as those of Unclaimed Territory and Cavern of Souls. It just produces one mana of the appropriate type, with no restrictions or riders."
    );
    supported("Cavern of Souls");
    for doubler in ["Mana Flare", "Heartbeat of Spring"] {
        supported(doubler);
        for land in ["Unclaimed Territory", "Cavern of Souls"] {
            let mut t = doubled_restricted_mana(doubler, land);
            let pool = &t.g.player(P0).mana_pool.mana;
            assert_eq!(pool.len(), 2, "{doubler}, {land}");
            assert!(pool.iter().all(|m| m.ty == mtg_engine::mana::ManaType::R));
            let free = pool
                .iter()
                .filter(|m| m.restriction.is_none() && m.rider.is_none())
                .count();
            assert_eq!(free, 1, "{doubler}, {land}: one unrestricted mana");
            // The extra red pays for Lightning Bolt, which isn't an Elf spell.
            let bolt = t.hand(P0, "Lightning Bolt");
            assert!(t.cast(P0, bolt).target(pl(P1)).try_go().is_ok());
            assert_eq!(t.g.player(P0).mana_pool.total(), 1);
        }
    }
}

#[test]
fn lavaleapers_extra_mana_is_a_mana_ability_that_doesnt_use_the_stack() {
    cr!("605.1b", "605.4a");
    ruling!(
        "Lavaleaper",
        "Lavaleaper's last ability is a mana ability. It doesn't use the stack and can't be responded to."
    );
    supported("Lavaleaper");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Lavaleaper");
    let forest = t.battlefield(P0, "Forest");
    assert!(tap_for_mana(&mut t, P0, forest, "{G}"));
    t.settle();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(
        t.g.player(P0)
            .mana_pool
            .count(mtg_engine::mana::ManaType::G),
        2
    );
}

// ---------------------------------------------------------------------------------
// Conundrum Sphinx
// ---------------------------------------------------------------------------------

#[test]
fn conundrum_sphinx_only_your_own_named_card_goes_to_your_hand() {
    cr!("701.20a");
    ruling!(
        "Conundrum Sphinx",
        "If a card a player reveals this way is the card that particular player named, they must put it into their hand. Otherwise, they must put it on the bottom of their library, even if it’s the card a different player named. In no cases can a player leave the revealed card on top of their library."
    );
    supported("Conundrum Sphinx");
    let mut t = TestGame::new(2);
    let sphinx = t.battlefield(P0, "Conundrum Sphinx");
    let mine = t.library_top(P0, "Grizzly Bears");
    let theirs = t.library_top(P1, "Grizzly Bears");
    t.answer(
        P0,
        DecisionKind::Name,
        Answer::Text("Lightning Bolt".into()),
    );
    t.answer(P1, DecisionKind::Name, Answer::Text("Grizzly Bears".into()));
    attack_with(&mut t, &[(sphinx, pl(P1))]);
    t.resolve_all();
    // P0 named something else: Grizzly Bears (which P1 named) goes to the bottom.
    assert_eq!(
        t.g.player(P0).library.first().copied(),
        Some(t.g.current(mine))
    );
    assert_ne!(
        t.g.player(P0).library.last().copied(),
        Some(t.g.current(mine))
    );
    assert!(matches!(t.zone(t.g.current(theirs)), Zone::Hand(_)));
}

// ---------------------------------------------------------------------------------
// Wild Evocation: "At the beginning of each player's upkeep, that player reveals a card
// at random from their hand. If it's a land card, the player puts it onto the
// battlefield. Otherwise, the player casts it without paying its mana cost if able."
// ---------------------------------------------------------------------------------

/// P0 controls Wild Evocation, holds only `card`, and their upkeep's trigger is on the
/// stack. Returns the game and the card.
fn evocation_with(card: &str, setup: fn(&mut TestGame)) -> (TestGame, ObjectId) {
    supported("Wild Evocation");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Wild Evocation");
    setup(&mut t);
    let c = t.hand(P0, card);
    to_upkeep(&mut t, P0);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    (t, c)
}

#[test]
fn wild_evocation_puts_the_spell_on_the_stack_and_finishes_first() {
    cr!("608.2g", "117.3b");
    ruling!(
        "Wild Evocation",
        "If a player casts a nonland card revealed with Wild Evocation's ability, it's put on the stack as a spell, then Wild Evocation's ability finishes resolving. The spell will then resolve as normal, after players get a chance to cast spells and activate abilities."
    );
    let (mut t, bears) = evocation_with("Grizzly Bears", |_| {});
    t.resolve();
    // The ability is gone; Grizzly Bears is a spell on the stack ...
    assert_eq!(t.stack_len(), 1);
    let spell = t.g.stack[0];
    assert_eq!(t.g.obj(spell).chars.name, "Grizzly Bears");
    assert_eq!(t.zone(t.g.current(bears)), Zone::Stack);
    // ... and players can respond to it: P1's Lightning Bolt resolves first.
    let bolt = t.hand(P1, "Lightning Bolt");
    t.lands(P1, "Mountain", 1);
    t.cast(P1, bolt).target(pl(P0)).go();
    t.resolve();
    assert_eq!(t.life(P0), 17);
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 0);
    t.resolve();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

#[test]
fn wild_evocation_ignores_timing_but_not_other_restrictions_or_targets() {
    cr!("608.2g", "601.2c", "601.3");
    ruling!(
        "Wild Evocation",
        "If a player reveals a nonland card, that player must cast that card if it's possible to do so, even if they don't want to. The player casts it as part of the resolution of Wild Evocation's ability. Timing restrictions based on the card's type (such as creature or sorcery) are ignored. Other play restrictions are not (such as \"Cast [this card] only during combat\")."
    );
    ruling!(
        "Wild Evocation",
        "If a player reveals a nonland card, but can't cast it due to a lack of legal targets or a play restriction, it simply remains in their hand."
    );
    supported("Panic");
    // A sorcery is cast during the upkeep, even though P0 says no.
    let (mut t, _) = evocation_with("Divination", |_| {});
    t.answer_yes(P0, false);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Divination"));
    assert_eq!(t.hand_size(P0), hand - 1 + 2);
    // "Cast this spell only during combat before blockers are declared": not in the
    // upkeep.
    let (mut t, _) = evocation_with("Panic", |_| {});
    t.resolve_all();
    assert!(t.in_hand(P0, "Panic"));
    // No legal target: Doom Blade stays in hand.
    let (mut t, _) = evocation_with("Doom Blade", |_| {});
    t.resolve_all();
    assert!(t.in_hand(P0, "Doom Blade"));
}

#[test]
fn wild_evocation_casts_without_paying_mana_cost_x_is_zero_and_kicker_may_be_paid() {
    cr!("118.9", "107.3b", "601.2b");
    ruling!(
        "Wild Evocation",
        "If, as the result of Wild Evocation's ability, a player casts a card \"without paying its mana cost,\" any X in the mana cost will be 0, and that player can't pay any alternative costs (such as Demon of Death's Gate's alternative cost). On the other hand, if the card has optional additional costs (such as kicker or multikicker), the player may pay those."
    );
    supported("Demon of Death's Gate");
    // X is 0: Blaze deals no damage.
    let (mut t, _) = evocation_with("Blaze", |_| {});
    t.answer_targets(P0, &[pl(P1)]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Blaze"));
    assert_eq!(t.life(P1), 20);
    // Kicker can be paid: Burst Lightning deals 4.
    let (mut t, _) = evocation_with("Burst Lightning", |t| {
        t.lands(P0, "Mountain", 4);
    });
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.answer_targets(P0, &[pl(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    // Demon of Death's Gate's alternative cost isn't offered: no life paid, no
    // sacrifices.
    let (mut t, _) = evocation_with("Demon of Death's Gate", |t| {
        for _ in 0..3 {
            t.battlefield(P0, "Typhoid Rats");
        }
    });
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Demon of Death's Gate").len(), 1);
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.named_on_battlefield("Typhoid Rats").len(), 3);
}

/// Whether P0 was asked, since decision `from`, whether to activate mana abilities for a
/// spell they're casting "if able".
fn asked_to_tap(t: &TestGame, from: usize) -> bool {
    t.asked()[from..].iter().any(|(p, d)| {
        *p == P0 && matches!(d, Decision::YesNo { prompt, .. } if prompt.contains("mana abilities"))
    })
}

/// After a Wild Evocation cast, an ordinary spell P0 casts is paid for with mana abilities,
/// as always: the "if able" choice is over. P0 needs an untapped Mountain.
fn casts_bolt_as_usual(t: &mut TestGame) {
    let life = t.life(P1);
    let tapped = crate::r_s01_common::tapped_lands(t, P0);
    let from = t.asked().len();
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(pl(P1)).go();
    t.resolve_all();
    assert_eq!(t.life(P1), life - 3);
    assert_eq!(crate::r_s01_common::tapped_lands(t, P0), tapped + 1);
    assert!(!asked_to_tap(t, from));
}

#[test]
fn wild_evocation_pays_a_mandatory_additional_cost_if_able() {
    cr!("601.2b", "601.2f", "601.2g", "601.2h", "118.9", "733.1");
    ruling!(
        "Wild Evocation",
        "If casting the revealed card involves paying a mandatory additional cost (such as the one Fling has), the player casting that card must pay that cost if able. If they can't, the card remains uncast in their hand. If the mandatory additional cost includes a mana payment, the situation is more complex. If the player has enough unspent mana to pay the cost, that player must do so. If the player can't possibly pay the cost, the card remains uncast in their hand. However, if the player has the means to produce enough mana to pay the cost, then they have a choice: The player may cast the card, produce mana, and pay the cost. Or the player may choose to activate no mana abilities, thus making the card impossible to cast because the additional mana can't be paid."
    );
    interp!("cast-if-able-mana-abilities-default");
    supported("Fling");
    // Fling with a creature to sacrifice: cast, the creature sacrificed.
    let (mut t, _) = evocation_with("Fling", |t| {
        t.battlefield(P0, "Hill Giant");
    });
    t.answer_targets(P0, &[pl(P1)]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Hill Giant"));
    assert!(t.in_graveyard(P0, "Fling"));
    assert_eq!(t.life(P1), 17);
    // Without one: it stays in hand.
    let (mut t, _) = evocation_with("Fling", |_| {});
    t.resolve_all();
    assert!(t.in_hand(P0, "Fling"));
    // Lash of the Balrog: "As an additional cost to cast this spell, sacrifice a creature
    // or pay {4}. Destroy target creature." P0 controls no creature, so the cost is {4}.
    supported("Lash of the Balrog");
    let lash = |t: &mut TestGame| {
        t.battlefield(P1, "Grizzly Bears");
    };
    let lash_with_mountains = |t: &mut TestGame| {
        t.battlefield(P1, "Grizzly Bears");
        t.lands(P0, "Mountain", 5);
    };
    // Unspent mana in the pool: it must be spent; the spell is cast.
    let (mut t, _) = evocation_with("Lash of the Balrog", lash);
    add_mana(&mut t, P0, &[mtg_engine::mana::ManaType::C], 4);
    let from = t.asked().len();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Lash of the Balrog"));
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(t.g.player(P0).mana_pool.total(), 0);
    assert!(!asked_to_tap(&t, from));
    // No way to pay: it stays in hand.
    let (mut t, _) = evocation_with("Lash of the Balrog", lash);
    t.resolve_all();
    assert!(t.in_hand(P0, "Lash of the Balrog"));
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    // Lands that could pay: the player chooses. Declining to activate mana abilities
    // leaves the card in hand and the lands untapped ...
    let (mut t, _) = evocation_with("Lash of the Balrog", lash_with_mountains);
    let from = t.asked().len();
    t.answer_yes(P0, false);
    t.resolve_all();
    assert!(asked_to_tap(&t, from));
    assert!(t.in_hand(P0, "Lash of the Balrog"));
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert_eq!(crate::r_s01_common::tapped_lands(&t, P0), 0);
    casts_bolt_as_usual(&mut t);
    // ... and agreeing (also the default answer) casts it, four lands tapped for the {4}.
    for answer in [Some(true), None] {
        let (mut t, _) = evocation_with("Lash of the Balrog", lash_with_mountains);
        if let Some(yes) = answer {
            t.answer_yes(P0, yes);
        }
        t.resolve_all();
        assert!(t.in_graveyard(P0, "Lash of the Balrog"), "{answer:?}");
        assert!(t.in_graveyard(P1, "Grizzly Bears"), "{answer:?}");
        assert_eq!(crate::r_s01_common::tapped_lands(&t, P0), 4);
        casts_bolt_as_usual(&mut t);
    }
}

#[test]
fn wild_evocation_declining_mana_abilities_for_a_kicker_still_casts_the_spell() {
    cr!("601.2b", "601.2g", "733.1", "733.2");
    ruling!(
        "Wild Evocation",
        "that player must cast that card if it's possible to do so, even if they don't want to"
    );
    ruling!(
        "Wild Evocation",
        "Or the player may choose to activate no mana abilities, thus making the card impossible to cast because the additional mana can't be paid."
    );
    interp!("cast-if-able-mana-abilities-default");
    // Burst Lightning (kicker {4}), kicked, but P0 then won't tap the Mountains: the
    // casting is reversed, and the card is cast again with the mana P0's pool has (none),
    // so unkicked. Declining mana abilities doesn't make a card uncastable whose
    // mandatory costs need none.
    let (mut t, _) = evocation_with("Burst Lightning", |t| {
        t.lands(P0, "Mountain", 5);
    });
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.answer_yes(P0, false);
    t.answer_targets(P0, &[pl(P1)]);
    t.answer_targets(P0, &[pl(P1)]);
    let from = t.asked().len();
    t.resolve_all();
    assert!(asked_to_tap(&t, from));
    assert!(t.in_graveyard(P0, "Burst Lightning"));
    assert_eq!(t.life(P1), 18, "cast unkicked");
    assert_eq!(crate::r_s01_common::tapped_lands(&t, P0), 0);
    // Kicker isn't offered again: the pool alone can't pay it.
    let kicker_offers = t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::OptionalCost { .. }))
        .count();
    assert_eq!(kicker_offers, 1);
    casts_bolt_as_usual(&mut t);
    // With {4} floating, the pool pays the kicker: nothing to decline.
    let (mut t, _) = evocation_with("Burst Lightning", |_| {});
    add_mana(&mut t, P0, &[mtg_engine::mana::ManaType::C], 4);
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.answer_targets(P0, &[pl(P1)]);
    let from = t.asked().len();
    t.resolve_all();
    assert!(!asked_to_tap(&t, from));
    assert_eq!(t.life(P1), 16, "kicked");
}

// ---------------------------------------------------------------------------------
// Votes
// ---------------------------------------------------------------------------------

#[test]
fn erestor_counts_an_opponent_with_two_votes_both_ways() {
    cr!("701.38a", "701.38d");
    ruling!(
        "Erestor of the Council",
        "If an opponent has multiple votes, it's possible for them to count as both an opponent who voted for a choice you voted for and an opponent who voted for a choice you didn't vote for."
    );
    interp!("opponent-with-several-votes-counts-both-ways");
    supported("Erestor of the Council");
    supported("Council Guardian");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Erestor of the Council");
    t.battlefield(P1, "Brago's Representative");
    // Council Guardian's vote: blue, black, red or green.
    t.answer(P0, DecisionKind::Option, Answer::Index(2)); // red
    t.answer(P1, DecisionKind::Option, Answer::Index(2)); // red
    t.answer(P1, DecisionKind::Option, Answer::Index(3)); // green
    t.enter(P0, "Council Guardian");
    t.resolve_all();
    let treasures =
        t.g.permanents()
            .filter(|o| o.controller == P1 && o.chars.has_subtype("Treasure"))
            .count();
    assert_eq!(treasures, 1, "P1 voted for red, as P0 did");
    let scried = t
        .asked()
        .iter()
        .any(|(p, d)| *p == P0 && matches!(d, Decision::Scry { cards } if cards.len() == 1));
    assert!(scried, "and for green, which P0 didn't: scry 1");
    // The same wording on Grudge Keeper: P1 voted for a choice P0 didn't vote for, too.
    supported("Grudge Keeper");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grudge Keeper");
    t.battlefield(P1, "Brago's Representative");
    t.answer(P0, DecisionKind::Option, Answer::Index(2)); // red
    t.answer(P1, DecisionKind::Option, Answer::Index(2)); // red
    t.answer(P1, DecisionKind::Option, Answer::Index(3)); // green
    t.enter(P0, "Council Guardian");
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

/// Grudge Keeper's ruling "If you have additional votes because of Brago’s Representative,
/// and you vote for more than one choice, an opponent will lose 2 life only if they voted
/// for none of those choices." (2014-05-29, cited for an opponent with one vote in
/// `r_p172_combat_votes.rs`) is narrower than stated: when the opponent has several votes
/// too, Erestor of the Council's ruling on the same words applies (docs/
/// RULING_DISCREPANCIES.md).
#[test]
fn grudge_keeper_counts_an_opponent_whose_several_votes_partly_match_yours() {
    cr!("701.38a", "701.38d");
    ruling!(
        "Erestor of the Council",
        "it's possible for them to count as both an opponent who voted for a choice you voted for and an opponent who voted for a choice you didn't vote for"
    );
    interp!("opponent-with-several-votes-counts-both-ways");
    // P0 votes red and green, P1 red and blue: P1 voted for red, one of P0's choices, and
    // for blue, a choice P0 didn't vote for.
    let vote = |t: &mut TestGame| {
        t.battlefield(P0, "Brago's Representative");
        t.battlefield(P1, "Brago's Representative");
        t.answer(P0, DecisionKind::Option, Answer::Index(2)); // red
        t.answer(P0, DecisionKind::Option, Answer::Index(3)); // green
        t.answer(P1, DecisionKind::Option, Answer::Index(2)); // red
        t.answer(P1, DecisionKind::Option, Answer::Index(0)); // blue
        t.enter(P0, "Council Guardian");
        t.resolve_all();
    };
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grudge Keeper");
    vote(&mut t);
    assert_eq!(t.life(P1), 18);
    // Erestor: a Treasure for P1 and scry 1 for P0.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Erestor of the Council");
    vote(&mut t);
    let treasures =
        t.g.permanents()
            .filter(|o| o.controller == P1 && o.chars.has_subtype("Treasure"))
            .count();
    assert_eq!(treasures, 1);
    assert!(t
        .asked()
        .iter()
        .any(|(p, d)| *p == P0 && matches!(d, Decision::Scry { cards } if cards.len() == 1)));
    // P1 votes red and green, as P0 does: not a vote for a choice P0 didn't vote for.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grudge Keeper");
    t.battlefield(P0, "Brago's Representative");
    t.battlefield(P1, "Brago's Representative");
    t.answer(P0, DecisionKind::Option, Answer::Index(2));
    t.answer(P0, DecisionKind::Option, Answer::Index(3));
    t.answer(P1, DecisionKind::Option, Answer::Index(3));
    t.answer(P1, DecisionKind::Option, Answer::Index(2));
    t.enter(P0, "Council Guardian");
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
}

fn sail_into_the_west(t: &mut TestGame, vote: usize) {
    supported("Sail into the West");
    t.answer(P0, DecisionKind::Option, Answer::Index(vote));
    t.answer(P1, DecisionKind::Option, Answer::Index(vote));
    t.answer(P2, DecisionKind::Option, Answer::Index(vote));
    cast_new(t, P0, "Sail into the West", &[]);
}

#[test]
fn sail_into_the_west_embark_decisions_go_in_turn_order_then_all_discard() {
    cr!("101.4", "701.9a");
    ruling!(
        "Sail into the West",
        "If embark gets more votes or the vote is tied, players decide and announce in turn order, starting with the active player, whether or not they want to discard their hand. Then, all players who chose to do so discard their hands and draw seven cards. Players later in the turn order will know if players before them are discarding, but they don't get to see what those players will discard to help them decide whether to discard themselves."
    );
    let mut t = TestGame::new(3);
    t.set_step(P1, Step::PrecombatMain);
    fill_hand(&mut t, P0, 2);
    fill_hand(&mut t, P1, 1);
    fill_hand(&mut t, P2, 3);
    // P1 is the active player; P0 casts it (instant).
    t.answer_yes(P1, true);
    t.answer_yes(P2, false);
    t.answer_yes(P0, true);
    let seen = watch(
        &mut t,
        P0,
        |d| matches!(d, Decision::YesNo { .. }),
        |g| g.player(P1).hand.len(),
    );
    let from = t.asked().len();
    sail_into_the_west(&mut t, 1);
    t.resolve_all();
    let order = deciders_since(&t, from, |d| matches!(d, Decision::YesNo { .. }));
    assert_eq!(order, vec![P1, P2, P0]);
    // When P0 (last) decided, P1 hadn't discarded yet.
    assert_eq!(seen_values(&seen), vec![1]);
    assert_eq!(t.hand_size(P0), 7);
    assert_eq!(t.hand_size(P1), 7);
    assert_eq!(t.hand_size(P2), 3);
    assert_eq!(t.graveyard_size(P1), 1);
}

#[test]
fn sail_into_the_west_return_choices_in_turn_order_then_all_at_once() {
    cr!("101.4");
    ruling!(
        "Sail into the West",
        "If return gets more votes, first the active player chooses which cards they'll return from their graveyard to their hand, then each other player in turn order does the same, knowing choices made before their choice. Then all those cards are returned from graveyards to hands simultaneously."
    );
    let mut t = TestGame::new(3);
    let a = t.graveyard(P0, "Grizzly Bears");
    let b = t.graveyard(P0, "Hill Giant");
    let c = t.graveyard(P1, "Lightning Bolt");
    let d = t.graveyard(P2, "Shock");
    t.answer_choose(P0, &[obj(a), obj(b)]);
    t.answer_choose(P1, &[obj(c)]);
    t.answer_choose(P2, &[obj(d)]);
    let seen = watch(
        &mut t,
        P2,
        |d| matches!(d, Decision::ChooseEntities { .. }),
        |g| {
            (
                g.player(PlayerId(0)).hand.len(),
                g.player(PlayerId(1)).hand.len(),
            )
        },
    );
    let from = t.asked().len();
    sail_into_the_west(&mut t, 0);
    t.resolve_all();
    let order = deciders_since(&t, from, |d| matches!(d, Decision::ChooseEntities { .. }));
    assert_eq!(order, vec![P0, P1, P2]);
    // When P2 chose, the others' cards were still in their graveyards.
    assert_eq!(seen_values(&seen), vec![(0, 0)]);
    assert!(t.in_hand(P0, "Grizzly Bears") && t.in_hand(P0, "Hill Giant"));
    assert!(t.in_hand(P1, "Lightning Bolt") && t.in_hand(P2, "Shock"));
    assert!(t.in_exile("Sail into the West"));
}

// ---------------------------------------------------------------------------------
// "Each opponent may"
// ---------------------------------------------------------------------------------

#[test]
fn shah_of_naar_isle_each_opponent_chooses_in_turn_order() {
    cr!("101.4", "702.30a");
    ruling!(
        "Shah of Naar Isle",
        "If you have multiple opponents, each one in turn order chooses whether or not to draw cards, and if so, how many to draw (from zero to three)."
    );
    supported("Shah of Naar Isle");
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Shah of Naar Isle");
    to_upkeep(&mut t, P0);
    let (h1, h2) = (t.hand_size(P1), t.hand_size(P2));
    t.answer_yes(P0, true);
    t.answer(P2, DecisionKind::Number, Answer::Number(0));
    t.answer(P1, DecisionKind::Number, Answer::Number(3));
    let from = t.asked().len();
    t.resolve_all();
    let order = deciders_since(&t, from, |d| matches!(d, Decision::ChooseNumber { .. }));
    assert_eq!(order, vec![P1, P2]);
    assert_eq!((t.hand_size(P1), t.hand_size(P2)), (h1 + 3, h2));
}

#[test]
fn iwamori_opponents_choose_from_the_active_player_in_turn_order_then_all_enter() {
    cr!("101.4", "101.4a");
    ruling!(
        "Iwamori of the Open Fist",
        "When Iwamori's triggered ability resolves, first the player whose turn it is (if that player is an opponent) may choose a legendary creature card in hand without revealing it, then each other opponent in turn order may do the same, then all chosen cards are put onto the battlefield at the same time."
    );
    supported("Iwamori of the Open Fist");
    // On P2's turn, P0's Iwamori enters: P2 chooses first, then P1.
    let mut t = TestGame::new(3);
    t.set_step(P2, Step::PrecombatMain);
    let i1 = t.hand(P1, "Isamaru, Hound of Konda");
    let i2 = t.hand(P2, "Isamaru, Hound of Konda");
    for (p, c) in [(P1, i1), (P2, i2)] {
        t.answer_yes(p, true);
        t.answer_choose(p, &[obj(c)]);
    }
    let seen = watch(
        &mut t,
        P1,
        |d| matches!(d, Decision::ChooseEntities { .. }),
        |g| g.battlefield.len(),
    );
    let from = t.asked().len();
    let events = t.g.turn_events.len();
    t.enter(P0, "Iwamori of the Open Fist");
    let before = t.g.battlefield.len();
    t.resolve_all();
    let yes = deciders_since(&t, from, |d| matches!(d, Decision::YesNo { .. }));
    assert_eq!(yes, vec![P2, P1]);
    let chose = deciders_since(&t, from, |d| matches!(d, Decision::ChooseEntities { .. }));
    assert_eq!(chose, vec![P2, P1]);
    // When P1 chose, P2's card hadn't entered yet.
    assert_eq!(seen_values(&seen), vec![before]);
    assert!(t.on_battlefield(i1) && t.on_battlefield(i2));
    // The cards were chosen without being revealed (CR 101.4a).
    let revealed = t.g.turn_events[events..].iter().any(|e| {
        matches!(e, Event::Custom { name, obj: Some(o), .. }
            if name == mtg_engine::reveal::REVEALED && (*o == i1 || *o == i2))
    });
    assert!(!revealed);
}

// ---------------------------------------------------------------------------------
// Two-Headed Giant
// ---------------------------------------------------------------------------------

#[test]
fn centaur_peacemaker_each_team_gains_eight_in_two_headed_giant() {
    cr!("810.9");
    ruling!(
        "Centaur Peacemaker",
        "In a Two-Headed Giant game, Centaur Peacemaker’s ability causes each team to gain 8 life."
    );
    supported("Centaur Peacemaker");
    let mut t = thg();
    let (a, b) = (t.life(P0), t.life(P2));
    t.enter(P0, "Centaur Peacemaker");
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P2)), (a + 8, b + 8));
    assert_eq!((t.life(P1), t.life(P3)), (a + 8, b + 8));
}

#[test]
fn aku_djinn_skips_your_teammates_creatures_in_two_headed_giant() {
    cr!("102.3");
    ruling!(
        "Aku Djinn",
        "Only puts counters on creatures your opponents control, so in the Two-Headed Giant format, won’t put counters on your teammate’s creatures."
    );
    supported("Aku Djinn");
    let mut t = thg();
    t.battlefield(P0, "Aku Djinn");
    let mate = t.battlefield(P1, "Grizzly Bears");
    let foe = t.battlefield(P2, "Grizzly Bears");
    let foe2 = t.battlefield(P3, "Grizzly Bears");
    t.set_step(P2, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.counters(mate, "+1/+1"), 0);
    assert_eq!(t.counters(foe, "+1/+1"), 1);
    assert_eq!(t.counters(foe2, "+1/+1"), 1);
}

// ---------------------------------------------------------------------------------
// Others
// ---------------------------------------------------------------------------------

#[test]
fn fecundity_trigger_is_removed_when_its_controller_loses() {
    cr!("800.4a", "104.3b");
    ruling!(
        "Fecundity",
        "In a multiplayer game, if a player loses the game, triggered abilities that player controls are removed from the stack and no more from that player can be added. This means that if a creature an opponent controls dies while you control Fecundity, and you lose the game before Fecundity’s triggered ability resolves (perhaps because you lost the game at the same time that the creature died), that player won’t draw a card."
    );
    supported("Fecundity");
    // P0 loses at the same time P1's creature dies.
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Fecundity");
    t.battlefield(P1, "Grizzly Bears");
    t.g.players[0].life = 2;
    let hand = t.hand_size(P1);
    t.answer_yes(P1, true);
    let quake = t.hand(P0, "Earthquake");
    t.lands(P0, "Mountain", 3);
    t.cast(P0, quake).x(2).go();
    t.resolve_all();
    assert!(t.has_lost(P0));
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(t.hand_size(P1), hand);
    // P0 loses after the trigger is on the stack: it's removed.
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Fecundity");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let hand = t.hand_size(P1);
    t.answer_yes(P1, true);
    destroy(&mut t, bears);
    assert_eq!(t.stack_len(), 1);
    t.g.players[0].life = 0;
    t.settle();
    assert!(t.has_lost(P0));
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.hand_size(P1), hand);
}

#[test]
fn oviya_gives_counters_to_a_card_that_becomes_an_artifact_on_the_battlefield() {
    cr!("611.3a", "608.2h");
    ruling!(
        "Oviya, Automech Artisan",
        "In some rare cases, a nonartifact card put onto the battlefield this way will get two +1/+1 counters because some other effect (like that of Mycosynth Lattice) makes it an artifact on the battlefield."
    );
    supported("Oviya, Automech Artisan");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mycosynth Lattice");
    let oviya = t.battlefield(P0, "Oviya, Automech Artisan");
    let bears = t.hand(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 1);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(bears)]);
    t.activate(P0, oviya, 0, &[]).unwrap();
    t.resolve_all();
    let bears = t.g.current(bears);
    assert!(t.on_battlefield(bears));
    assert_eq!(t.counters(bears, "+1/+1"), 2);
    // Without Mycosynth Lattice: no counters.
    let mut t = TestGame::new(2);
    let oviya = t.battlefield(P0, "Oviya, Automech Artisan");
    let bears = t.hand(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 1);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(bears)]);
    t.activate(P0, oviya, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(t.g.current(bears), "+1/+1"), 0);
}

#[test]
fn greener_pastures_checks_the_lands_again_on_resolution() {
    cr!("603.4");
    ruling!(
        "Greener Pastures",
        "The ability does not trigger unless the player has more lands at the beginning of upkeep. This is checked again on resolution, and if the player does not still have more lands, no token is generated."
    );
    supported("Greener Pastures");
    // Not more lands: no trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Greener Pastures");
    t.lands(P0, "Forest", 2);
    t.lands(P1, "Forest", 2);
    to_upkeep(&mut t, P0);
    t.settle();
    assert_eq!(t.stack_len(), 0);
    // More lands, still more on resolution: a Saproling.
    t.lands(P0, "Forest", 1);
    to_upkeep(&mut t, P0);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 1);
    // No longer more lands on resolution: nothing.
    to_upkeep(&mut t, P0);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.lands(P1, "Forest", 1);
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 1);
}

#[test]
fn elephant_resurgence_tokens_keep_adjusting() {
    cr!("611.3a", "604.3");
    ruling!(
        "Elephant Resurgence",
        "The token creatures' power and toughness continuously adjust. They are not \"locked in\" when the spell resolves. This is because the tokens get an ability, rather than simply having their power and toughness set."
    );
    supported("Elephant Resurgence");
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Grizzly Bears");
    cast_new(&mut t, P0, "Elephant Resurgence", &[]);
    t.resolve_all();
    let mine = tokens(&t, P0)[0];
    assert_eq!(t.pt(mine), (1, 1));
    t.graveyard(P0, "Hill Giant");
    assert_eq!(t.pt(mine), (2, 2));
}

#[test]
fn party_dude_locks_in_x_as_its_ability_resolves() {
    cr!("608.2h", "611.2c");
    ruling!(
        "Party Dude",
        "The value of X is calculated each time Party Dude's last ability resolves. Changing the number of cards in your hand after the ability resolves won't change the bonus granted to that creature that turn."
    );
    supported("Party Dude");
    let mut t = TestGame::new(2);
    let dude = t.battlefield(P0, "Party Dude");
    mtg_engine::classes::set_level(&mut t.g, dude, 3);
    let bears = t.battlefield(P0, "Grizzly Bears");
    fill_hand(&mut t, P0, 2);
    t.answer_targets(P0, &[obj(bears)]);
    attack_with(&mut t, &[(bears, pl(P1))]);
    // More cards before it resolves count.
    fill_hand(&mut t, P0, 1);
    t.resolve_all();
    assert_eq!(t.pt(bears), (5, 5));
    // More after don't.
    fill_hand(&mut t, P0, 2);
    assert_eq!(t.pt(bears), (5, 5));
}

#[test]
fn planar_birth_lands_enter_tapped_rather_than_tapping_after() {
    cr!("110.5b");
    ruling!(
        "Planar Birth",
        "They enter tapped. They do not enter and then tap afterwards."
    );
    supported("Planar Birth");
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Forest");
    t.graveyard(P1, "Island");
    let events = t.g.turn_events.len();
    cast_new(&mut t, P0, "Planar Birth", &[]);
    let lands_before: Vec<ObjectId> = t.g.battlefield.clone();
    t.resolve_all();
    let new: Vec<ObjectId> =
        t.g.battlefield
            .iter()
            .copied()
            .filter(|id| !lands_before.contains(id))
            .collect();
    assert_eq!(new.len(), 2);
    assert!(new.iter().all(|id| t.g.obj(*id).tapped));
    let tapped_after = t.g.turn_events[events..]
        .iter()
        .any(|e| matches!(e, Event::Tapped { obj, .. } if new.contains(obj)));
    assert!(!tapped_after, "no tap event: they entered tapped");
}

#[test]
fn yurlok_life_loss_happens_with_the_mana_loss_without_the_stack() {
    cr!("500.5", "106.4");
    ruling!(
        "Yurlok of Scorch Thrash",
        "Yurlok's middle ability is similar to the \"mana burn\" rule from years past. The loss of life happens at the same time the mana is lost. This loss of life doesn't use the stack and can't be responded to."
    );
    supported("Yurlok of Scorch Thrash");
    let mut t = TestGame::new(2);
    let y = t.battlefield(P0, "Yurlok of Scorch Thrash");
    t.lands(P0, "Wastes", 1);
    assert!(tap_for_mana(&mut t, P0, y, "Each player adds"));
    assert_eq!(t.g.player(P1).mana_pool.total(), 3);
    t.advance_to(P0, Step::BeginningOfCombat);
    assert_eq!(t.life(P0), 17);
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.stack_len(), 0);
    t.settle();
    assert_eq!(t.stack_len(), 0);
}
