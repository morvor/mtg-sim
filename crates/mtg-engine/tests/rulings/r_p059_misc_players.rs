//! Rulings batch P059 — effects on every player: maximum hand size (CR 402.2, 514.1),
//! The Great Aurora's draws and simultaneous lands (CR 111.8, 101.4, 614.12), choices in
//! turn order followed by simultaneous discards and sacrifices (CR 101.4), Chandra
//! Ablaze's discard, Blue Sun's Zenith following its instructions in order (CR 608.2c),
//! life totals becoming a number (CR 119.5), Drown in Dreams' commander condition, Test of
//! Talents finding the countered card, and Mathas's bounty counters (CR 611.2b, 122.1).

use crate::r_p059_common::*;
use crate::r_s01_common::{attack_with, give_mana_for, stack_library, supported, watch};
use crate::r_s02_common::{create_token, destroy};
use crate::r_s06_common::give_control;
use crate::r_s11_common::empty_library;
use mtg_engine::decision::Decision;
use mtg_engine::events::Event;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn is_choice(d: &Decision) -> bool {
    matches!(d, Decision::ChooseEntities { .. })
}

#[test]
fn maximum_hand_size_is_checked_only_in_its_players_cleanup_step() {
    cr!("402.2", "514.1");
    ruling!(
        "Winter, Misanthropic Guide",
        "A player's maximum hand size is checked only during the cleanup step of their turn. At other times, they may have more cards in hand than their maximum hand size."
    );
    // "Delirium — As long as there are four or more card types among cards in your
    // graveyard, each opponent's maximum hand size is equal to seven minus the number of
    // those card types." Four types: P1's maximum is 3.
    supported("Winter, Misanthropic Guide");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Winter, Misanthropic Guide");
    for c in ["Forest", "Grizzly Bears", "Lightning Bolt", "Mind Rot"] {
        t.graveyard(P0, c);
    }
    for _ in 0..6 {
        t.hand(P1, "Grizzly Bears");
    }
    // Through P0's cleanup step: P1 keeps 6.
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.hand_size(P1), 6);
    t.advance_to(P1, Step::End);
    assert_eq!(t.hand_size(P1), 7, "drew; more than the maximum");
    t.advance_to(P0, Step::Upkeep);
    assert_eq!(t.hand_size(P1), 3, "discarded in P1's cleanup step");
}

#[test]
fn the_great_aurora_counts_tokens_and_puts_lands_down_together() {
    cr!("111.8", "111.2", "101.4", "614.12");
    ruling!(
        "The Great Aurora",
        "A token is owned by the player under whose control it entered the battlefield. Tokens shuffled into a library count toward the number of cards drawn, though they cease to exist upon becoming part of a library. Regardless of what you’re using to represent tokens, you won’t shuffle that physical object into your library."
    );
    ruling!(
        "The Great Aurora",
        "As The Great Aurora resolves, first you choose any number of land cards in your hand to put onto the battlefield, and then each other player in turn order does the same. The lands enter the battlefield simultaneously."
    );
    // "Each player shuffles all cards from their hand and all permanents they own into
    // their library, then draws that many cards. Each player may put any number of land
    // cards from their hand onto the battlefield. Exile The Great Aurora."
    supported("The Great Aurora");
    supported("Botanical Sanctum");
    let mut t = TestGame::new(2);
    // P0's library and mana: nine Botanical Sanctums ("This land enters tapped unless you
    // control two or fewer other lands.").
    empty_library(&mut t, P0);
    t.lands(P0, "Botanical Sanctum", 9);
    // A token P1 created, now controlled by P0: P1 owns it.
    let token = create_token(&mut t, P1, "Soldier");
    give_control(&mut t, token, P0);
    // P1's library: an Island; P1 owns a Grizzly Bears, the token and a Forest in hand.
    empty_library(&mut t, P1);
    stack_library(&mut t, P1, &["Island"]);
    t.battlefield(P1, "Grizzly Bears");
    t.hand(P1, "Forest");
    let spell = t.hand(P0, "The Great Aurora");
    let p0_count = t.hand_size(P0) - 1 + 9;
    let p1_count = t.hand_size(P1) + 2;
    // P1 puts no land down.
    let seen = watch(&mut t, P1, is_choice, |g| {
        g.battlefield
            .iter()
            .filter(|id| g.obj(**id).chars.name == "Botanical Sanctum")
            .count()
    });
    t.answer_choose(P1, &[]);
    t.cast(P0, spell).go();
    // P0 puts every land from hand onto the battlefield.
    let p0_lands = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    {
        let lands = p0_lands.clone();
        let mut agents = t.g.agents.0.lock().unwrap();
        let inner = std::mem::replace(
            &mut agents[0],
            Box::new(mtg_engine::decision::PassiveAgent),
        );
        struct PickAll {
            inner: Box<dyn mtg_engine::decision::Agent>,
            lands: std::sync::Arc<std::sync::Mutex<Vec<usize>>>,
        }
        impl mtg_engine::decision::Agent for PickAll {
            fn decide(
                &mut self,
                g: &mtg_engine::game::Game,
                p: PlayerId,
                d: &Decision,
            ) -> mtg_engine::decision::Answer {
                if let Decision::ChooseEntities {
                    candidates, max, ..
                } = d
                {
                    self.lands.lock().unwrap().push(candidates.len());
                    let n = (*max as usize).min(candidates.len());
                    return mtg_engine::decision::Answer::Entities(candidates[..n].to_vec());
                }
                self.inner.decide(g, p, d)
            }
        }
        agents[0] = Box::new(PickAll { inner, lands });
    }
    t.resolve_all();
    assert!(t.in_exile("The Great Aurora"));
    assert!(super_token_gone(&t, token));
    // P1 drew one card per card and permanent P1 owned, the token included.
    assert_eq!(t.hand_size(P1), p1_count, "{}", t.dump_log());
    // P0 drew nine (the Sanctums) and put them all onto the battlefield, all untapped:
    // they entered at the same time, so none saw the others.
    let mine: Vec<&mtg_engine::object::GameObject> = t
        .g
        .permanents()
        .filter(|o| o.chars.name == "Botanical Sanctum")
        .collect();
    assert_eq!(mine.len(), p0_count, "{}", t.dump_log());
    assert!(mine.iter().all(|o| !o.tapped), "simultaneously");
    // P0 chose first: when P1 chose, P0's lands weren't on the battlefield yet.
    assert_eq!(*seen.lock().unwrap(), vec![0]);
    assert_eq!(*p0_lands.lock().unwrap(), vec![9]);
}

fn super_token_gone(t: &TestGame, token: ObjectId) -> bool {
    token_gone(t, token) && !t.g.players.iter().any(|p| p.library.contains(&t.g.current(token)))
}

#[test]
fn chandra_ablaze_discards_and_deals_damage_only_for_a_red_card() {
    cr!("701.9a", "608.2c");
    ruling!(
        "Chandra Ablaze",
        "As the first ability resolves, nothing happens if your hand is empty. But if you have any cards in hand, you must discard one. If you discard a nonred card, Chandra doesn't deal any damage."
    );
    // "+1: Discard a card. If a red card is discarded this way, Chandra deals 4 damage to
    // any target."
    supported("Chandra Ablaze");
    for hand in [None, Some("Grizzly Bears"), Some("Lightning Bolt")] {
        let mut t = TestGame::new(2);
        let chandra = t.battlefield(P0, "Chandra Ablaze");
        if let Some(c) = hand {
            t.hand(P0, c);
        }
        crate::r_p040_common::loyalty(&mut t, P0, chandra, "+1", &[pl(P1)]);
        assert_eq!(t.hand_size(P0), 0);
        match hand {
            None => assert_eq!(t.life(P1), 20),
            Some("Grizzly Bears") => {
                assert!(t.in_graveyard(P0, "Grizzly Bears"));
                assert_eq!(t.life(P1), 20, "nonred");
            }
            _ => {
                assert!(t.in_graveyard(P0, "Lightning Bolt"));
                assert_eq!(t.life(P1), 16);
            }
        }
    }
}

/// P0's Rankle deals combat damage to P1, and P0 chooses the modes `modes`.
fn rankle(t: &mut TestGame, modes: &[usize]) {
    let rankle = t.battlefield(P0, "Rankle, Master of Pranks");
    t.answer(
        P0,
        DecisionKind::Modes,
        mtg_engine::decision::Answer::Indices(modes.to_vec()),
    );
    attack_with(t, &[(rankle, pl(P1))]);
    t.advance_to(P0, Step::CombatDamage);
    t.resolve_all();
}

#[test]
fn rankle_discards_chosen_in_turn_order_then_all_at_once() {
    cr!("101.4", "701.9a");
    ruling!(
        "Rankle, Master of Pranks",
        "As the first mode is performed, first the player whose turn it is chooses a card in hand without revealing it, then each other player in turn order does the same. Then all the chosen cards are discarded at the same time."
    );
    supported("Rankle, Master of Pranks");
    let mut t = TestGame::new(3);
    let a = t.hand(P0, "Grizzly Bears");
    t.hand(P0, "Hill Giant");
    t.hand(P1, "Gray Ogre");
    t.hand(P1, "Craw Wurm");
    t.hand(P2, "Llanowar Elves");
    t.hand(P2, "Kird Ape");
    t.answer_choose(P0, &[o(a)]);
    // When P1 and P2 choose, nothing has been discarded yet.
    let seen1 = watch(&mut t, P1, is_choice, |g| {
        g.players.iter().map(|p| p.graveyard.len()).sum::<usize>()
    });
    let seen2 = watch(&mut t, P2, is_choice, |g| {
        g.players.iter().map(|p| p.graveyard.len()).sum::<usize>()
    });
    let mark = t.asked().len();
    rankle(&mut t, &[0]);
    assert_eq!(*seen1.lock().unwrap(), vec![0]);
    assert_eq!(*seen2.lock().unwrap(), vec![0]);
    let order: Vec<PlayerId> = t.asked()[mark..]
        .iter()
        .filter(|(_, d)| is_choice(d))
        .map(|(p, _)| *p)
        .collect();
    assert_eq!(order, vec![P0, P1, P2], "turn order");
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    for p in [P0, P1, P2] {
        assert_eq!(t.hand_size(p), 1, "{p}");
        assert_eq!(t.g.player(p).graveyard.len(), 1, "{p}");
    }
}

#[test]
fn rankle_sacrifices_chosen_in_turn_order_then_all_at_once() {
    cr!("101.4", "701.21a", "603.10a");
    ruling!(
        "Rankle, Master of Pranks",
        "As the third mode is performed, first the player whose turn it chooses a creature they control, then each other player in turn order does the same, knowing the choices made before them. Then all the chosen creatures are sacrificed at the same time."
    );
    // P0 sacrifices Blood Artist ("Whenever this creature or another creature dies, target
    // player loses 1 life and you gain 1 life."), P1 a Grizzly Bears: sacrificed together,
    // Blood Artist sees both die.
    let mut t = TestGame::new(2);
    let artist = t.battlefield(P0, "Blood Artist");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Hill Giant");
    t.answer_choose(P0, &[o(artist)]);
    t.answer_choose(P1, &[o(bears)]);
    let seen = watch(&mut t, P1, is_choice, |g| {
        g.battlefield
            .iter()
            .any(|id| g.obj(*id).chars.name == "Blood Artist")
    });
    let mark = t.asked().len();
    t.answer_targets(P0, &[pl(P1)]);
    t.answer_targets(P0, &[pl(P1)]);
    rankle(&mut t, &[2]);
    assert_eq!(*seen.lock().unwrap(), vec![true], "P0's choice made, not yet sacrificed");
    let order: Vec<PlayerId> = t.asked()[mark..]
        .iter()
        .filter(|(_, d)| is_choice(d))
        .map(|(p, _)| *p)
        .collect();
    assert_eq!(order, vec![P0, P1]);
    assert!(t.in_graveyard(P0, "Blood Artist"));
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    // Rankle's 3 combat damage, and two Blood Artist triggers.
    assert_eq!(t.life(P1), 20 - 3 - 2, "{}", t.dump_log());
    assert_eq!(t.life(P0), 22);
}

#[test]
fn blue_suns_zenith_draws_before_it_is_shuffled_in() {
    cr!("608.2c", "704.5b", "121.4");
    ruling!(
        "Blue Sun's Zenith",
        "Because you follow the spell's instructions in order, you won't be able to draw the same Blue Sun's Zenith that you cast."
    );
    // "Target player draws X cards. Shuffle Blue Sun's Zenith into its owner's library."
    // With an empty library, P0 draws from it (and loses), though the Zenith is then
    // shuffled into it.
    supported("Blue Sun's Zenith");
    let mut t = TestGame::new(2);
    empty_library(&mut t, P0);
    give_mana_for(&mut t, P0, "Blue Sun's Zenith");
    t.lands(P0, "Island", 1);
    let z = t.hand(P0, "Blue Sun's Zenith");
    t.cast(P0, z).x(1).target(P0).go();
    t.g.resolve_top();
    t.g.flush_events();
    assert_eq!(t.zone(z), Zone::Library(P0), "shuffled in");
    assert_eq!(t.hand_size(P0), 0, "the Zenith wasn't drawn");
    t.settle();
    assert!(t.has_lost(P0));
    // With a library: the drawn card isn't the Zenith.
    let mut t = TestGame::new(2);
    empty_library(&mut t, P0);
    stack_library(&mut t, P0, &["Grizzly Bears"]);
    give_mana_for(&mut t, P0, "Blue Sun's Zenith");
    t.lands(P0, "Island", 1);
    let z = t.hand(P0, "Blue Sun's Zenith");
    t.cast(P0, z).x(1).target(P0).go();
    t.resolve_all();
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert_eq!(t.g.player(P0).library.len(), 1);
    assert_eq!(t.zone(z), Zone::Library(P0));
}

#[test]
fn sway_of_the_stars_life_changes_are_gains_and_losses() {
    cr!("119.5");
    ruling!(
        "Sway of the Stars",
        "Changes in life total are treated as life gain and life loss."
    );
    supported("Sway of the Stars");
    let mut t = TestGame::new(2);
    crate::r_p042_common::set_life(&mut t, P1, 3);
    let mark = t.g.turn_events.len();
    cast_resolve(&mut t, P0, "Sway of the Stars", &[]);
    assert_eq!((t.life(P0), t.life(P1)), (7, 7));
    let lost: Vec<(PlayerId, u32)> = t.g.turn_events[mark..]
        .iter()
        .filter_map(|e| match e {
            Event::LifeLost { player, amount, .. } => Some((*player, *amount)),
            _ => None,
        })
        .collect();
    assert_eq!(lost, vec![(P0, 13)]);
    assert_eq!(crate::r_s33_common::life_gains_since(&t, mark, P1), vec![4]);
    assert!(crate::r_s33_common::life_gains_since(&t, mark, P0).is_empty());
}

#[test]
fn drown_in_dreams_needs_a_commander_permanent_you_control() {
    cr!("903.3d", "700.2");
    ruling!(
        "Drown in Dreams",
        "Controlling a commander means that you control a permanent on the battlefield that is someone's commander. It can be yours or another player's. Controlling a commander spell on the stack will not allow you to choose both modes."
    );
    // "Choose one. If you control a commander as you cast this spell, you may choose both
    // instead. • Target player draws X cards. • Target player mills twice X cards."
    supported("Drown in Dreams");
    /// The maximum number of modes P0 may choose for Drown in Dreams now.
    fn max_modes(t: &mut TestGame) -> u32 {
        crate::r_s01_common::give_mana_for(t, P0, "Drown in Dreams");
        let s = t.hand(P0, "Drown in Dreams");
        let mark = t.asked().len();
        t.answer(
            P0,
            DecisionKind::Modes,
            mtg_engine::decision::Answer::Indices(vec![0]),
        );
        t.answer(P0, DecisionKind::X, mtg_engine::decision::Answer::Number(0));
        t.answer_targets(P0, &[pl(P1)]);
        t.cast(P0, s).go();
        t.asked()[mark..]
            .iter()
            .find_map(|(_, d)| match d {
                Decision::ChooseModes { max, .. } => Some(*max),
                _ => None,
            })
            .expect("modes asked")
    }
    use crate::r_s13_common::{commander, commander_game};
    // P1's commander, controlled by P0.
    let mut t = commander_game();
    let c = commander(&mut t, P1, "Grizzly Bears");
    crate::r_s05_common::move_to(&mut t, c, Zone::Battlefield);
    let c = t.g.current(c);
    give_control(&mut t, c, P0);
    assert_eq!(max_modes(&mut t), 2, "another player's commander");
    // P0's own commander on the battlefield.
    let mut t = commander_game();
    let c = commander(&mut t, P0, "Grizzly Bears");
    crate::r_s05_common::move_to(&mut t, c, Zone::Battlefield);
    assert_eq!(max_modes(&mut t), 2, "your commander");
    // P0's commander as a spell on the stack: one mode only.
    let mut t = commander_game();
    let c = commander(&mut t, P0, "Ambush Viper");
    t.lands(P0, "Forest", 2);
    t.cast(P0, c).go();
    assert_eq!(t.stack_len(), 1);
    assert_eq!(max_modes(&mut t), 1, "a commander spell");
    // No commander.
    let mut t = commander_game();
    assert_eq!(max_modes(&mut t), 1);
}

#[test]
fn test_of_talents_finds_the_countered_card_in_the_graveyard() {
    cr!("701.6a", "701.23a");
    ruling!(
        "Test of Talents",
        "Countering a spell normally puts that card into the graveyard, so that card will be there to find and exile."
    );
    // "Counter target instant or sorcery spell. Search its controller's graveyard, hand,
    // and library for any number of cards with the same name as that spell and exile
    // them. That player shuffles, then draws a card for each card exiled from their hand
    // this way."
    supported("Test of Talents");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let in_hand = t.hand(P1, "Lightning Bolt");
    let in_library = t.library_top(P1, "Lightning Bolt");
    let bolt = cast(&mut t, P1, "Lightning Bolt", &[pl(P0)]);
    t.answer_choose_moved(P0, &[o(bolt), o(in_hand), o(in_library)]);
    t.answer_choose_moved(P0, &[o(in_hand)]);
    t.answer_choose_moved(P0, &[o(in_library)]);
    let hand = t.hand_size(P1);
    cast_resolve(&mut t, P0, "Test of Talents", &[o(bolt)]);
    assert_eq!(t.life(P0), 20, "countered");
    for b in [bolt, in_hand, in_library] {
        assert_eq!(t.zone(b), Zone::Exile, "{}", t.dump_log());
    }
    assert_eq!(t.hand_size(P1), hand, "drew one for the one from hand");
}

#[test]
fn only_mathass_own_counter_grants_the_ability_once() {
    cr!("611.2b", "614.1a");
    ruling!(
        "Mathas, Fiend Seeker",
        "Creatures that receive a bounty counter other than from Mathas's ability resolving won't gain the triggered ability. Creatures that receive more than one bounty counter as Mathas's ability resolves (most likely due to Doubling Season) won't have the ability multiple times."
    );
    // "At the beginning of your end step, put a bounty counter on target creature an
    // opponent controls. For as long as that creature has a bounty counter on it, it has
    // 'When this creature dies, each opponent draws a card and gains 2 life.'"
    // A bounty counter from elsewhere: nothing when it dies.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mathas, Fiend Seeker");
    let bears = t.battlefield(P1, "Grizzly Bears");
    put(&mut t, bears, "bounty", 1);
    let hand = t.hand_size(P0);
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!((t.life(P0), t.hand_size(P0)), (20, hand));
    // Two bounty counters from Mathas's ability (P1's Doubling Season): one ability.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mathas, Fiend Seeker");
    t.battlefield(P1, "Doubling Season");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[o(bears)]);
    end_step_of(&mut t, P0);
    assert_eq!(t.counters(bears, "bounty"), 2);
    let hand = t.hand_size(P0);
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.life(P0), 22, "{}", t.dump_log());
    assert_eq!(t.hand_size(P0), hand + 1);
}
