//! Rulings batch P093 — the rest: Bolas's Citadel, Call of the Ring, Bonecache Overseer,
//! Glissa Sunslayer, Blex, Shanid, Fang and Vanille, Liliana's Contract, Treacherous
//! Blessing, Barbed Servitor, Zuko, Castle Locthwain, Asylum Visitor, Savanti Romero and
//! Bebop, Shessra's end step, The Bloodsky Massacre, Be'lakor and Champion of Dusk,
//! Tinybones, Griselbrand, Bloodletter Quill, Discreet Retreat, Nashi and Bat Whisperer.

use crate::r_p093_common::*;
use mtg_engine::decision::{Action, Answer, Decision};
use mtg_engine::events::Event;
use mtg_engine::game::GameResult;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn can_activate(t: &mut TestGame, p: PlayerId, source: ObjectId) -> bool {
    crate::r_s02_common::can_activate(t, p, source)
}

// --- Bolas's Citadel --------------------------------------------------------------------------

/// P0 controls Bolas's Citadel and nine Ornithopters, and activates the Citadel's last
/// ability sacrificing all ten (the Citadel among them); it resolves.
fn citadel(t: &mut TestGame) -> ObjectId {
    supported("Bolas's Citadel");
    let c = t.battlefield(P0, "Bolas's Citadel");
    let mut all = vec![obj(c)];
    for _ in 0..9 {
        all.push(obj(t.battlefield(P0, "Ornithopter")));
    }
    t.answer_choose(P0, &all);
    crate::r_s06_common::activate_containing(t, P0, c, "Sacrifice ten").unwrap();
    t.resolve_all();
    c
}

#[test]
fn bolass_citadel_can_be_one_of_the_ten() {
    cr!("602.2b", "601.2h");
    ruling!(
        "Bolas's Citadel",
        "Bolas's Citadel may be one of the permanents you sacrifice to activate its last ability."
    );
    let mut t = TestGame::new(2);
    let c = citadel(&mut t);
    assert!(t.in_graveyard(P0, "Bolas's Citadel"));
    assert_eq!(t.zone(c), Zone::Graveyard(P0));
    assert!(t.named_on_battlefield("Ornithopter").is_empty());
    assert_eq!(t.life(P1), 10);
}

#[test]
fn bolass_citadel_in_two_headed_giant() {
    cr!("810.9");
    ruling!(
        "Bolas's Citadel",
        "In a Two-Headed Giant game, the last ability of Bolas's Citadel causes the opposing team to lose 20 life."
    );
    let mut t = crate::r_s33_common::two_headed_giant();
    let (ours, theirs) = (t.life(P0), t.life(P2));
    citadel(&mut t);
    assert_eq!(t.life(P2), theirs - 20);
    assert_eq!(t.life(P3), theirs - 20);
    assert_eq!(t.life(P0), ours);
}

// --- Call of the Ring -------------------------------------------------------------------------

#[test]
fn call_of_the_ring_triggers_on_each_choice_of_a_ring_bearer() {
    cr!("701.54a", "603.2");
    ruling!(
        "Call of the Ring",
        "Call of the Ring's last ability will trigger any time you choose a creature as your Ring-bearer, even if that creature was already your Ring-bearer."
    );
    supported("Call of the Ring");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Call of the Ring");
    t.battlefield(P0, "Grizzly Bears");
    for _ in 0..2 {
        t.answer_yes(P0, true);
    }
    crate::r_s04_common::next_upkeep(&mut t, P0);
    t.resolve_all();
    assert_eq!((t.hand_size(P0), t.life(P0)), (1, 18));
    // The same creature again next upkeep: it triggers again.
    t.advance_to(P1, Step::Upkeep);
    let hand = t.hand_size(P0);
    crate::r_s04_common::next_upkeep(&mut t, P0);
    t.resolve_all();
    assert_eq!((t.hand_size(P0), t.life(P0)), (hand + 1, 16));
}

#[test]
fn call_of_the_ring_without_a_creature() {
    cr!("701.54a");
    ruling!(
        "Call of the Ring",
        "If the Ring tempts you but you can't choose a creature as your Ring-bearer (probably because you control no creatures), Call of the Ring's last ability won't trigger."
    );
    let mut t = TestGame::new(2);
    let call = t.battlefield(P0, "Call of the Ring");
    t.answer_yes(P0, true);
    crate::r_s04_common::next_upkeep(&mut t, P0);
    t.resolve();
    assert_eq!(from_source(&t, call), 0);
    t.clear_answers();
    t.resolve_all();
    assert_eq!((t.hand_size(P0), t.life(P0)), (0, 20));
}

// --- Bonecache Overseer -----------------------------------------------------------------------

#[test]
fn bonecache_overseer_counts_each_time_a_card_leaves() {
    cr!("400.7", "602.5");
    ruling!(
        "Bonecache Overseer",
        "Each time a card leaves your graveyard in a turn, it counts as one of three cards the activated ability is looking for. For example, if you return two cards from your graveyard to your hand, discard one of them, then exile that card from your graveyard, you can activate Bonecache Overseer's ability that turn."
    );
    supported("Bonecache Overseer");
    let mut t = TestGame::new(2);
    let overseer = t.battlefield(P0, "Bonecache Overseer");
    let a = t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P0, "Hill Giant");
    let b = t.g.player(P0).graveyard[1];
    let a = crate::r_s05_common::move_to(&mut t, a, Zone::Hand(P0)).unwrap();
    crate::r_s05_common::move_to(&mut t, b, Zone::Hand(P0));
    assert!(!can_activate(&mut t, P0, overseer));
    t.g.discard(P0, a, None);
    t.g.flush_events();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert!(!can_activate(&mut t, P0, overseer));
    crate::r_s05_common::move_to(&mut t, a, Zone::Exile);
    assert!(can_activate(&mut t, P0, overseer));
    crate::r_s06_common::activate_containing(&mut t, P0, overseer, "Draw").unwrap();
    t.resolve_all();
    assert_eq!((t.hand_size(P0), t.life(P0)), (2, 19));
}

// --- Glissa Sunslayer ------------------------------------------------------------------------

#[test]
fn glissa_removes_up_to_three_counters_of_any_kinds() {
    cr!("608.2d", "700.2");
    ruling!(
        "Glissa Sunslayer",
        "For the third mode, you choose which counters to remove as the ability resolves. You may remove any three (or fewer) counters, even if the permanent has multiple kinds of counters on it."
    );
    supported("Glissa Sunslayer");
    let mut t = TestGame::new(2);
    let glissa = t.battlefield(P0, "Glissa Sunslayer");
    let giant = t.battlefield(P1, "Hill Giant");
    put_counters(&mut t, giant, counters::PLUS1, 2);
    put_counters(&mut t, giant, counters::SHIELD, 1);
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![2]));
    t.answer_targets(P0, &[obj(giant)]);
    attack_with(&mut t, &[(glissa, Entity::Player(P1))]);
    t.advance_to(P0, Step::FirstStrikeDamage);
    t.settle();
    assert_eq!(from_source(&t, glissa), 1);
    // Stun counters put on it after the ability triggered can be among those removed: the
    // choice is made on resolution. P0 removes both stun counters and the shield counter,
    // one counter at a time.
    put_counters(&mut t, giant, "stun", 2);
    t.answer(P0, DecisionKind::Number, Answer::Number(3));
    for i in [2, 2, 1] {
        t.answer(P0, DecisionKind::Option, Answer::Index(i));
    }
    let from = t.asked().len();
    t.resolve_all();
    let options: Vec<Vec<String>> = t.asked()[from..]
        .iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseOption { options, .. } if *p == P0 => Some(options.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(options.len(), 3);
    assert_eq!(
        options[0],
        ["+1/+1 counter", "shield counter", "stun counter"]
    );
    assert_eq!(options[2], ["+1/+1 counter", "shield counter"]);
    let left = [counters::PLUS1, counters::SHIELD, "stun"].map(|k| t.counters(giant, k));
    assert_eq!(left, [2, 0, 0]);
}

// --- Blex, Vexing Pest --------------------------------------------------------------------

#[test]
fn blex_gives_a_changeling_one_bonus() {
    cr!("613.4c", "702.73a");
    ruling!(
        "Blex, Vexing Pest // Search for Blex",
        "If a creature is more than one of the creature types Blex cares about, it may haunt your dreams, but it will get the +1/+1 bonus only once."
    );
    supported("Blex, Vexing Pest // Search for Blex");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Blex, Vexing Pest // Search for Blex");
    let outcast = t.battlefield(P0, "Changeling Outcast");
    assert_eq!(t.pt(outcast), (2, 2));
}

// --- Shanid, Sleepers' Scourge -------------------------------------------------------------

#[test]
fn shanids_trigger_resolves_before_and_without_the_spell() {
    cr!("603.3", "405.2");
    ruling!(
        "Shanid, Sleepers' Scourge",
        "If casting a legendary spell causes Shanid’s last ability to trigger, that ability goes on the stack above the spell that caused it to trigger and resolves first. It resolves even if that spell is countered or otherwise left the stack."
    );
    supported("Shanid, Sleepers' Scourge");
    let mut t = TestGame::new(2);
    let shanid = t.battlefield(P0, "Shanid, Sleepers' Scourge");
    let isamaru = cast_only(&mut t, P0, "Isamaru, Hound of Konda", &[]);
    t.settle();
    assert_eq!(t.g.stack.len(), 2);
    assert_eq!(t.g.stack[0], isamaru);
    assert_eq!(crate::r_s25_common::abilities_from(&t, shanid), vec![t.g.stack[1]]);
    cast_new(&mut t, P1, "Counterspell", &[obj(isamaru)]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Isamaru, Hound of Konda"));
    assert_eq!((t.hand_size(P0), t.life(P0)), (1, 19));
}

// --- Fang and Vanille -----------------------------------------------------------------------

const FANG: &str = "Fang, Fearless l'Cie";
const VANILLE: &str = "Vanille, Cheerful l'Cie";

#[test]
fn fang_triggers_once_for_cards_leaving_together() {
    cr!("603.2c", "603.10a");
    ruling!(
        "Fang, Fearless l'Cie",
        "If multiple cards leave your graveyard at the same time, Fang's ability will trigger only once."
    );
    supported("Bojuka Bog");
    let mut t = TestGame::new(2);
    let fang = t.battlefield(P0, FANG);
    for _ in 0..3 {
        t.graveyard(P0, "Grizzly Bears");
    }
    t.answer_targets(P0, &[Entity::Player(P0)]);
    crate::r_s05_common::enter(&mut t, P0, "Bojuka Bog");
    // The Bog's trigger exiles the three cards at once: one trigger.
    t.resolve();
    assert_eq!(t.graveyard_size(P0), 0);
    assert_eq!(from_source(&t, fang), 1);
    t.resolve_all();
    assert_eq!((t.hand_size(P0), t.life(P0)), (1, 19));
}

#[test]
fn vanilles_meld_trigger_checks_on_trigger_and_on_resolution() {
    cr!("603.4", "701.42a");
    // Misattached (logged in docs/RULING_DISCREPANCIES.md): this ruling describes Vanille,
    // Cheerful l'Cie's second ability ("At the beginning of your first main phase, if you
    // both own and control Vanille and a creature named Fang, Fearless l'Cie, you may pay
    // {3}{B}{G}. If you do, exile them, then meld them into Ragnarok, Divine
    // Deliverance."); the data files it under Fang, its meld partner, which takes part.
    ruling!(
        "Fang, Fearless l'Cie",
        "Vanille's second ability checks the battlefield at the moment it would trigger to see if you own and control Vanille and a creature named Fang, Fearless l'Cie. If you don't, the ability won't trigger at all. If it does trigger, the ability will check again as it tries to resolve. If you don't control both Fang and Vanille at that time, the ability won't resolve, you won't have the opportunity to pay {3}{B}{G}, and none of the ability's other effects will happen."
    );
    supported(VANILLE);
    fn first_main(t: &mut TestGame) {
        t.lands(P0, "Swamp", 1);
        t.lands(P0, "Forest", 1);
        t.lands(P0, "Wastes", 3);
        t.set_step(P0, Step::Draw);
        t.advance_to(P0, Step::PrecombatMain);
        t.settle();
    }
    // No Fang (one of P1's doesn't count: not owned): no trigger.
    let mut t = TestGame::new(2);
    let vanille = t.battlefield(P0, VANILLE);
    let theirs = t.battlefield(P1, FANG);
    crate::r_s06_common::give_control(&mut t, theirs, P0);
    first_main(&mut t);
    assert_eq!(from_source(&t, vanille), 0);
    // Both there throughout: the payment is offered, and they meld.
    let mut t = TestGame::new(2);
    let vanille = t.battlefield(P0, VANILLE);
    t.battlefield(P0, FANG);
    first_main(&mut t);
    assert_eq!(from_source(&t, vanille), 1);
    t.answer_yes(P0, true);
    let from = t.asked().len();
    t.resolve_all();
    assert_eq!(yes_nos(&t, P0, from), 1);
    assert_eq!(t.named_on_battlefield("Ragnarok, Divine Deliverance").len(), 1);
    assert_eq!(crate::r_s04_common::untapped_lands(&t, P0), 0);
    // Fang leaves with the trigger on the stack: no payment asked, no meld.
    let mut t = TestGame::new(2);
    let vanille = t.battlefield(P0, VANILLE);
    let fang = t.battlefield(P0, FANG);
    first_main(&mut t);
    assert_eq!(from_source(&t, vanille), 1);
    kill(&mut t, fang);
    t.answer_yes(P0, true);
    let from = t.asked().len();
    t.resolve_all();
    t.clear_answers();
    assert_eq!(yes_nos(&t, P0, from), 0);
    assert!(t.on_battlefield(vanille));
    assert!(t.named_on_battlefield("Ragnarok, Divine Deliverance").is_empty());
    assert_eq!(crate::r_s04_common::untapped_lands(&t, P0), 5);
}

// --- Liliana's Contract -----------------------------------------------------------------------

#[test]
fn lilianas_contract_rechecks_the_demons_on_resolution() {
    cr!("603.4", "104.2b");
    ruling!(
        "Liliana's Contract",
        "If the second ability does trigger, but you no longer control four Demons with different names as the ability resolves, you won’t win the game."
    );
    let demons = [
        "Demon of Catastrophes",
        "Archfiend of Ifnir",
        "Lord of the Void",
        "Desecration Demon",
    ];
    for remove in [true, false] {
        let mut t = TestGame::new(2);
        let contract = t.battlefield(P0, "Liliana's Contract");
        let ids: Vec<ObjectId> = demons.iter().map(|d| t.battlefield(P0, d)).collect();
        crate::r_s04_common::next_upkeep(&mut t, P0);
        assert_eq!(from_source(&t, contract), 1);
        if remove {
            kill(&mut t, ids[0]);
        }
        t.resolve_all();
        let won = t.g.result == Some(GameResult::Win(vec![P0]));
        assert_eq!(won, !remove, "{remove}");
    }
}

// --- Treacherous Blessing ---------------------------------------------------------------------

#[test]
fn treacherous_blessing_targeted_by_your_spell() {
    cr!("603.3b", "113.7a");
    ruling!(
        "Treacherous Blessing",
        "If you cast a spell that targets Treacherous Blessing, you choose whether you lose 1 life before or after you sacrifice it. You lose 1 life even if you sacrifice it first."
    );
    supported("Treacherous Blessing");
    // P0 orders the two triggers either way; whichever resolves first, the Blessing ends
    // up sacrificed and P0 loses 1 life.
    let mut firsts = Vec::new();
    for order in [vec![0, 1], vec![1, 0]] {
        let mut t = TestGame::new(2);
        let blessing = t.battlefield(P0, "Treacherous Blessing");
        let from = t.asked().len();
        t.answer(P0, DecisionKind::Order, Answer::Indices(order.clone()));
        cast_only(&mut t, P0, "Disenchant", &[obj(blessing)]);
        t.settle();
        assert!(t.asked()[from..]
            .iter()
            .any(|(p, d)| *p == P0 && matches!(d, Decision::Order { .. })));
        assert_eq!(from_source(&t, blessing), 2);
        t.resolve();
        // Which one resolved first: the sacrifice, or the life loss.
        let sacrificed_first = !t.on_battlefield(blessing);
        assert_eq!(t.life(P0), if sacrificed_first { 20 } else { 19 }, "{order:?}");
        firsts.push(sacrificed_first);
        t.resolve_all();
        assert!(t.in_graveyard(P0, "Treacherous Blessing"), "{order:?}");
        assert_eq!(t.life(P0), 19, "{order:?}");
    }
    assert_ne!(firsts[0], firsts[1]);
}

// --- Barbed Servitor ------------------------------------------------------------------------

#[test]
fn barbed_servitor_controller_losing_first() {
    cr!("704.3", "704.5a", "603.3", "800.4a");
    ruling!(
        "Barbed Servitor",
        "If your life total is brought to 0 or less at the same time that Barbed Servitor is dealt damage, you lose the game before its last ability goes on the stack."
    );
    // Three players (the game goes on after P0 loses). P1's Earthquake (X = 1) deals 1
    // damage to P0 and to the Servitor at the same time. At 1 life, P0 loses before "Whenever
    // this creature is dealt damage, target opponent loses that much life." goes on the
    // stack; at 20 (the control), it goes on the stack and P1 loses 1 life.
    for life in [1, 20] {
        let mut t = TestGame::new(3);
        let servitor = t.battlefield(P0, "Barbed Servitor");
        set_life(&mut t, P0, life);
        t.answer_targets(P0, &[Entity::Player(P1)]);
        t.set_step(P1, Step::PrecombatMain);
        crate::r_s01_common::give_mana_for(&mut t, P1, "Earthquake");
        let quake = t.hand(P1, "Earthquake");
        t.cast(P1, quake).x(1).go();
        let from = t.asked().len();
        t.resolve_all();
        let put_on_stack = t.g.turn_events.iter().any(
            |e| matches!(e, Event::AbilityTriggeredOnStack { source, .. } if *source == servitor),
        );
        let target_asked = t.asked()[from..]
            .iter()
            .any(|(p, d)| *p == P0 && matches!(d, Decision::ChooseTargets { .. }));
        if life == 1 {
            assert!(t.has_lost(P0));
            assert!(!put_on_stack && !target_asked, "it never went on the stack");
            assert_eq!((t.life(P1), t.life(P2)), (19, 19));
        } else {
            assert!(!t.has_lost(P0));
            assert!(put_on_stack && target_asked, "control: it went on the stack");
            assert_eq!((t.life(P0), t.life(P1), t.life(P2)), (19, 18, 19));
        }
    }
}

// --- Zuko, Conflicted -----------------------------------------------------------------------

const ZUKO: &str = "Zuko, Conflicted";

/// In a three-player game, P0's Zuko resolves its first-main-phase trigger choosing its
/// last mode: it returns under P1's control.
fn zuko_to_p1(t: &mut TestGame) -> ObjectId {
    supported(ZUKO);
    let zuko = t.battlefield(P0, ZUKO);
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![3]));
    t.answer_choose(P0, &[Entity::Player(P1)]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.set_step(P0, Step::Draw);
    t.advance_to(P0, Step::PrecombatMain);
    t.resolve_all();
    t.clear_answers();
    let z = t.g.current(zuko);
    assert_eq!(t.zone(z), Zone::Battlefield);
    assert_eq!(t.obj(z).controller, P1);
    assert_eq!(t.life(P0), 18);
    z
}

#[test]
fn zuko_under_an_opponents_control() {
    cr!("603.3a", "505.1");
    ruling!(
        "Zuko, Conflicted",
        "When Zuko, Conflicted is under an opponent's control, his triggered ability will trigger in their first main phase, and they will make all choices for the ability."
    );
    let mut t = TestGame::new(3);
    let zuko = zuko_to_p1(&mut t);
    let hand = t.hand_size(P1);
    let from = t.asked().len();
    t.answer(P1, DecisionKind::Modes, Answer::Indices(vec![0]));
    t.advance_to(P1, Step::PrecombatMain);
    t.resolve_all();
    assert!(t.asked()[from..]
        .iter()
        .any(|(p, d)| *p == P1 && matches!(d, Decision::ChooseModes { .. })));
    assert!(!t.asked()[from..]
        .iter()
        .any(|(p, d)| *p == P0 && matches!(d, Decision::ChooseModes { .. })));
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.life(P0), 18);
    // P1 drew for the turn and for Zuko.
    assert_eq!(t.hand_size(P1), hand + 2);
    assert_eq!(t.obj_now(zuko).controller, P1);
}

#[test]
fn zuko_leaves_with_his_owner_and_is_exiled_when_his_controller_leaves() {
    cr!("800.4a");
    ruling!(
        "Zuko, Conflicted",
        "In a multiplayer game, if Zuko, Conflicted's owner leaves the game, Zuko, Conflicted leaves the game with them. If his controller leaves the game, Zuko, Conflicted is exiled (assuming that player wasn't also Zuko, Conflicted's owner)."
    );
    // The controller (P1) leaves: exiled.
    let mut t = TestGame::new(3);
    let zuko = zuko_to_p1(&mut t);
    t.g.lose_game(P1);
    t.settle();
    assert_eq!(t.zone(zuko), Zone::Exile);
    // The owner (P0) leaves: Zuko leaves the game.
    let mut t = TestGame::new(3);
    let zuko = zuko_to_p1(&mut t);
    t.g.lose_game(P0);
    t.settle();
    assert!(!crate::r_p197_common::in_game(&t, zuko));
    assert!(t.named_on_battlefield(ZUKO).is_empty());
}

// --- Castle Locthwain -------------------------------------------------------------------------

#[test]
fn castle_locthwain_draws_and_loses_life_in_one_go() {
    cr!("608.2c", "117.3b");
    ruling!(
        "Castle Locthwain",
        "No player may take actions between the time you draw a card and the time you lose life."
    );
    supported("Castle Locthwain");
    let mut t = TestGame::new(2);
    let castle = t.battlefield(P0, "Castle Locthwain");
    t.lands(P0, "Swamp", 3);
    t.hand(P0, "Grizzly Bears");
    t.hand(P0, "Grizzly Bears");
    crate::r_s06_common::activate_containing(&mut t, P0, castle, "Draw a card").unwrap();
    t.settle();
    let events = t.g.turn_events.len();
    // One resolution draws and then loses life for the cards in hand after the draw (no
    // stack object in between, so no player gets priority between them).
    t.g.resolve_top();
    assert_eq!(drawn_since(&t, events, P0), 1);
    assert_eq!(losses_since(&t, events, P0), vec![3]);
    assert_eq!(t.life(P0), 17);
}

// --- Asylum Visitor -------------------------------------------------------------------------

#[test]
fn asylum_visitor_resolves_before_the_draw_step() {
    cr!("503.1", "504.1");
    ruling!(
        "Asylum Visitor",
        "The upkeep step is before the draw step, after the untap step. Asylum Visitor's first ability will trigger and resolve before the active player draws a card in their draw step if that player has no cards in hand."
    );
    let mut t = TestGame::new(2);
    let visitor = t.battlefield(P0, "Asylum Visitor");
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert_eq!(from_source(&t, visitor), 1);
    t.resolve_all();
    assert_eq!((t.hand_size(P0), t.life(P0)), (1, 19));
    t.advance_to(P0, Step::Draw);
    assert_eq!(t.hand_size(P0), 2);
}

#[test]
fn asylum_visitor_on_an_opponents_turn_resolves_first() {
    cr!("101.4", "603.3b");
    ruling!(
        "Asylum Visitor",
        "On an opponent's turn, triggered abilities you control will resolve before any triggered abilities of permanents that opponent controls if they trigger at the same time. This means that if you and your opponent each control an Asylum Visitor during your opponent's upkeep, and they have no cards in hand, you'll always draw a card before your opponent has a card in their hand."
    );
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Asylum Visitor");
    t.battlefield(P1, "Asylum Visitor");
    t.hand(P0, "Grizzly Bears");
    crate::r_s04_common::next_upkeep(&mut t, P1);
    assert_eq!(t.stack_len(), 2);
    assert_eq!(
        *t.g.stack.last().unwrap(),
        crate::r_s25_common::abilities_from(&t, mine)[0]
    );
    t.resolve();
    assert_eq!((t.hand_size(P0), t.hand_size(P1)), (2, 0));
    t.resolve();
    assert_eq!((t.hand_size(P0), t.hand_size(P1)), (2, 1));
}

// --- Savanti Romero and Bebop: X on resolution ------------------------------------------------

#[test]
fn savanti_romero_counts_every_counter_on_resolution() {
    cr!("122.1", "608.2h");
    ruling!(
        "Savanti Romero, Time's Exile",
        "Savanti Romero's last ability counts all counters on Savanti Romero, not just +1/+1 counters."
    );
    ruling!(
        "Savanti Romero, Time's Exile",
        "The value of X is calculated only once, as Savanti Romero's last ability resolves."
    );
    supported("Savanti Romero, Time's Exile");
    let mut t = TestGame::new(2);
    let savanti = t.battlefield(P0, "Savanti Romero, Time's Exile");
    put_counters(&mut t, savanti, counters::SHIELD, 1);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.settle();
    assert_eq!(from_source(&t, savanti), 1);
    // A counter added while the ability waits counts too.
    put_counters(&mut t, savanti, "stun", 1);
    let events = t.g.turn_events.len();
    t.resolve_all();
    // +1/+1, shield and stun: X = 3 for both the draw and the life loss.
    assert_eq!(drawn_since(&t, events, P0), 3);
    assert_eq!(losses_since(&t, events, P0), vec![3]);
}

#[test]
fn bebop_x_is_calculated_on_resolution() {
    cr!("608.2h", "603.5");
    ruling!(
        "Bebop, Skull & Crossbones",
        "The value of X is calculated only once, as Bebop's last ability resolves."
    );
    let mut t = TestGame::new(2);
    let bebop = t.battlefield(P0, "Bebop, Skull & Crossbones");
    put_counters(&mut t, bebop, counters::PLUS1, 1);
    attack_with(&mut t, &[(bebop, Entity::Player(P1))]);
    t.advance_to(P0, Step::CombatDamage);
    t.settle();
    assert_eq!(from_source(&t, bebop), 1);
    put_counters(&mut t, bebop, counters::SHIELD, 2);
    t.answer_yes(P0, true);
    let events = t.g.turn_events.len();
    t.resolve_all();
    assert_eq!(drawn_since(&t, events, P0), 3);
    assert_eq!(losses_since(&t, events, P0), vec![3]);
}

// --- Shessra's end step ---------------------------------------------------------------------

#[test]
fn shessra_end_step_trigger_once_and_only_if_something_died_before() {
    cr!("603.4", "603.2b");
    ruling!(
        "Shessra, Death's Whisper",
        "Shessra, Death's Whisper's last ability will trigger only once during your end step, no matter how many creatures died this turn. However, if no creatures have died so far this turn as the end step begins, the ability won't trigger at all. It's not possible to cause a creature to die during the end step in time to have the ability trigger."
    );
    // Two deaths: one trigger.
    let mut t = TestGame::new(2);
    let shessra = t.battlefield(P0, "Shessra, Death's Whisper");
    for _ in 0..2 {
        let b = t.battlefield(P1, "Grizzly Bears");
        kill(&mut t, b);
    }
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(from_source(&t, shessra), 1);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!((t.hand_size(P0), t.life(P0)), (1, 18));
    // None yet as the end step begins: no trigger, and a death in the end step is too
    // late.
    let mut t = TestGame::new(2);
    let shessra = t.battlefield(P0, "Shessra, Death's Whisper");
    let b = t.battlefield(P1, "Grizzly Bears");
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(from_source(&t, shessra), 0);
    kill(&mut t, b);
    assert_eq!(from_source(&t, shessra), 0);
    t.answer_yes(P0, true);
    t.resolve_all();
    t.clear_answers();
    assert_eq!((t.hand_size(P0), t.life(P0)), (0, 20));
}

// --- The Bloodsky Massacre ---------------------------------------------------------------------

#[test]
fn bloodsky_massacre_chapter_three_uses_the_stack() {
    cr!("605.1b", "714.2b", "608.2h");
    ruling!(
        "The Bloodsky Massacre",
        "The chapter III ability isn’t a mana ability. It uses the stack and can be responded to. Use the number of Berserkers you control as the ability resolves to determine how much {R} to add."
    );
    supported("The Bloodsky Massacre");
    let mut t = TestGame::new(2);
    let saga = crate::r_p197_common::blank_saga(&mut t, P0, "The Bloodsky Massacre");
    t.g.objects[saga.0 as usize]
        .counters
        .insert(counters::LORE.into(), 2);
    t.battlefield(P0, "Dragonkin Berserker");
    crate::r_s19_common::add_lore(&mut t, saga, 1);
    assert_eq!(from_source(&t, saga), 1);
    assert_eq!(t.g.player(P0).mana_pool.total(), 0);
    // A second Berserker arrives in response.
    t.battlefield(P0, "Dragonkin Berserker");
    t.resolve_all();
    assert_eq!(t.g.player(P0).mana_pool.total(), 2);
}

// --- Be'lakor and Champion of Dusk ------------------------------------------------------------

#[test]
fn counted_on_resolution_including_the_source() {
    cr!("608.2h", "603.3");
    ruling!(
        "Champion of Dusk",
        "The number of Vampires you control is counted only as Champion of Dusk's ability resolves. If Champion of Dusk is still on the battlefield, it'll count itself."
    );
    supported("Champion of Dusk");
    // Champion and Child of Night, plus a second Vampire arriving in response: 3.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Child of Night");
    let champion = crate::r_s05_common::enter(&mut t, P0, "Champion of Dusk");
    assert_eq!(from_source(&t, champion), 1);
    t.battlefield(P0, "Child of Night");
    t.resolve_all();
    assert_eq!((t.hand_size(P0), t.life(P0)), (3, 17));
    // Gone by then: it doesn't count itself.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Child of Night");
    let champion = crate::r_s05_common::enter(&mut t, P0, "Champion of Dusk");
    crate::r_s05_common::move_to(&mut t, champion, Zone::Hand(P0));
    t.resolve_all();
    assert_eq!((t.hand_size(P0), t.life(P0)), (2, 19));

    ruling!(
        "Be'lakor, the Dark Master",
        "The number of Demons you control is counted only as Be'lakor, the Dark Master's ability resolves. If Be'lakor is still on the battlefield, it will count itself."
    );
    supported("Be'lakor, the Dark Master");
    let mut t = TestGame::new(2);
    let belakor = crate::r_s05_common::enter(&mut t, P0, "Be'lakor, the Dark Master");
    assert_eq!(from_source(&t, belakor), 1);
    t.battlefield(P0, "Desecration Demon");
    t.resolve_all();
    assert_eq!((t.hand_size(P0), t.life(P0)), (2, 18));
}

// --- Tinybones, Trinket Thief ----------------------------------------------------------------

#[test]
fn tinybones_doesnt_see_the_cleanup_discard() {
    cr!("603.4", "514.1", "603.2b");
    ruling!(
        "Tinybones, Trinket Thief",
        "Tinybones’s first ability triggers only if an opponent has discarded a card before the end step begins. If an opponent has more cards in hand than their maximum hand size, they’ll discard during their cleanup step, which is after the end step."
    );
    supported("Tinybones, Trinket Thief");
    // P1 has nine cards on their turn: no trigger at its end step; it discards in cleanup.
    let mut t = TestGame::new(2);
    let tinybones = t.battlefield(P0, "Tinybones, Trinket Thief");
    t.set_step(P1, Step::PostcombatMain);
    for _ in 0..9 {
        t.hand(P1, "Grizzly Bears");
    }
    t.advance_to(P1, Step::End);
    t.settle();
    assert_eq!(from_source(&t, tinybones), 0);
    t.advance_to(P0, Step::Upkeep);
    assert_eq!(t.hand_size(P1), 7);
    assert_eq!(t.hand_size(P0), 0);
    // A discard before the end step: it triggers.
    let mut t = TestGame::new(2);
    let tinybones = t.battlefield(P0, "Tinybones, Trinket Thief");
    t.hand(P1, "Grizzly Bears");
    cast_spell(&mut t, P0, "Mind Rot", &[Entity::Player(P1)]);
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(from_source(&t, tinybones), 1);
    t.resolve_all();
    assert_eq!((t.hand_size(P0), t.life(P0)), (1, 19));
}

// --- Griselbrand ----------------------------------------------------------------------------

#[test]
fn griselbrand_needs_seven_life() {
    cr!("119.4", "602.2b");
    ruling!(
        "Griselbrand",
        "You can’t activate Griselbrand’s ability if you have 6 or less life."
    );
    let mut t = TestGame::new(2);
    let gris = t.battlefield(P0, "Griselbrand");
    set_life(&mut t, P0, 6);
    assert!(!can_activate(&mut t, P0, gris));
    set_life(&mut t, P0, 7);
    assert!(can_activate(&mut t, P0, gris));
}

// --- Bloodletter Quill ----------------------------------------------------------------------

#[test]
fn bloodletter_quill_remove_the_counter_in_response() {
    cr!("602.2b", "608.2h", "117.3c");
    ruling!(
        "Bloodletter Quill",
        "You may activate the second ability in response to activating the first one. If you do, you’ll remove the blood counter that was just added before you lose the life."
    );
    supported("Bloodletter Quill");
    let mut t = TestGame::new(2);
    let quill = t.battlefield(P0, "Bloodletter Quill");
    t.lands(P0, "Wastes", 2);
    t.lands(P0, "Island", 1);
    t.lands(P0, "Swamp", 1);
    crate::r_s06_common::activate_containing(&mut t, P0, quill, "Draw a card").unwrap();
    assert_eq!(t.counters(quill, "blood"), 1);
    crate::r_s06_common::activate_containing(&mut t, P0, quill, "Remove a blood counter")
        .unwrap();
    t.resolve();
    assert_eq!(t.counters(quill, "blood"), 0);
    t.resolve_all();
    assert_eq!((t.hand_size(P0), t.life(P0)), (1, 20));
}

// --- Discreet Retreat -----------------------------------------------------------------------

/// P0 controls a Swamp enchanted with Discreet Retreat and taps it for two black mana
/// (restricted to outlaws).
fn retreat_mana(t: &mut TestGame) {
    supported("Discreet Retreat");
    let swamp = t.battlefield(P0, "Swamp");
    crate::r_s06_common::attach_new(t, P0, "Discreet Retreat", swamp);
    t.answer(P0, DecisionKind::Option, Answer::Index(2));
    assert!(crate::r_s20_common::tap_for_mana(t, P0, swamp, "two mana"));
    assert_eq!(t.g.player(P0).mana_pool.total(), 2);
}

/// Whether P0 can cast the card in hand now with the mana in their pool.
fn castable(t: &mut TestGame, card: ObjectId) -> bool {
    crate::r_s21_common::castable(t, P0, card)
}

#[test]
fn discreet_retreat_mana_on_one_or_two_outlaw_spells() {
    cr!("106.6", "601.2h");
    ruling!(
        "Discreet Retreat",
        "You may spend the two mana added by the ability granted by Discreet Retreat on the same spell or activated ability or two different spells and/or activated abilities."
    );
    // Two Bonecache Overseers ({B}, Squirrel Warlock).
    let mut t = TestGame::new(2);
    retreat_mana(&mut t);
    for _ in 0..2 {
        let o = t.hand(P0, "Bonecache Overseer");
        t.cast(P0, o).go();
        t.resolve_all();
    }
    assert_eq!(t.named_on_battlefield("Bonecache Overseer").len(), 2);
    // One Tinybones ({1}{B}, Skeleton Rogue).
    let mut t = TestGame::new(2);
    retreat_mana(&mut t);
    let tiny = t.hand(P0, "Tinybones, Trinket Thief");
    t.cast(P0, tiny).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Tinybones, Trinket Thief").len(), 1);
}

#[test]
fn discreet_retreat_outlaw_sources() {
    cr!("106.6", "602.2b", "700.12");
    ruling!(
        "Discreet Retreat",
        "“Outlaw sources” include any objects with at least one of the Assassin, Mercenary, Pirate, Rogue, or Warlock creature types. This means you could spend the mana to activate the ability of an outlaw creature you control or an outlaw card in your hand or graveyard, for example."
    );
    supported("Burrog Banemaker");
    supported("Magmakin Artillerist");
    // Not a non-outlaw spell.
    let mut t = TestGame::new(2);
    retreat_mana(&mut t);
    let bears = t.hand(P0, "Grizzly Bears");
    assert!(!castable(&mut t, bears));
    // An outlaw creature's ability: Burrog Banemaker (Frog Warlock), "{1}{B}: This
    // creature gets +1/+1 until end of turn."
    let burrog = t.battlefield(P0, "Burrog Banemaker");
    crate::r_s06_common::activate_containing(&mut t, P0, burrog, "+1/+1").unwrap();
    t.resolve_all();
    assert_eq!(t.pt(burrog), (2, 2));
    // An outlaw card's ability in hand: Magmakin Artillerist (Elemental Pirate), cycling
    // {1}{R}.
    let mut t = TestGame::new(2);
    let swamp = t.battlefield(P0, "Swamp");
    crate::r_s06_common::attach_new(&mut t, P0, "Discreet Retreat", swamp);
    t.answer(P0, DecisionKind::Option, Answer::Index(3));
    assert!(crate::r_s20_common::tap_for_mana(&mut t, P0, swamp, "two mana"));
    let artillerist = t.hand(P0, "Magmakin Artillerist");
    crate::r_s06_common::activate_containing(&mut t, P0, artillerist, "ycling").unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Magmakin Artillerist"));
    assert_eq!(t.hand_size(P0), 1);
}

// --- Nashi, Moon Sage's Scion -----------------------------------------------------------------

#[test]
fn nashi_cards_follow_normal_timing() {
    cr!("305.2", "307.1", "601.3");
    ruling!(
        "Nashi, Moon Sage's Scion",
        "You must follow all normal timing rules when casting a spell or playing a land this way. For example, you cannot normally play a land card this way if you have already played a land this turn."
    );
    supported("Nashi, Moon Sage's Scion");
    let mut t = TestGame::new(2);
    let nashi = t.battlefield(P0, "Nashi, Moon Sage's Scion");
    let forest = t.library_top(P0, "Forest");
    let divination = t.library_top(P1, "Divination");
    attack_with(&mut t, &[(nashi, Entity::Player(P1))]);
    t.advance_to(P0, Step::CombatDamage);
    t.resolve_all();
    assert_eq!(t.zone(forest), Zone::Exile);
    assert_eq!(t.zone(divination), Zone::Exile);
    // A sorcery: not during combat.
    assert!(!crate::r_p097_common::playable(&mut t, divination));
    t.advance_to(P0, Step::PostcombatMain);
    assert!(crate::r_p097_common::playable(&mut t, divination));
    assert!(crate::r_p097_common::playable(&mut t, forest));
    // Once P0 has played a land this turn, not the exiled land.
    let land = t.hand(P0, "Swamp");
    t.play_land(P0, land).unwrap();
    assert!(!crate::r_p097_common::playable(&mut t, forest));
    // Casting the sorcery costs life.
    t.g.turn.priority = Some(P0);
    let d = t.g.current(divination);
    let cast = t
        .g
        .legal_actions(P0)
        .into_iter()
        .find(|a| matches!(a, Action::Cast { card, .. } if *card == d))
        .unwrap();
    t.g.perform_action(P0, cast).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P0), 17);
}

// --- Bat Whisperer -------------------------------------------------------------------------

#[test]
fn bat_whisperer_cares_about_life_lost() {
    cr!("119.3", "603.4");
    ruling!(
        "Bat Whisperer",
        "Bat Whisperer's ability cares whether an opponent lost life this turn, not how their life total changed. For example, an opponent who gained 2 life and lost 1 life in the same turn still lost life."
    );
    supported("Bat Whisperer");
    let mut t = TestGame::new(2);
    t.g.gain_life(P1, 2);
    t.g.lose_life(P1, 1);
    t.g.flush_events();
    crate::r_s05_common::enter(&mut t, P0, "Bat Whisperer");
    t.resolve_all();
    assert_eq!(t.life(P1), 21);
    assert_eq!(crate::r_s05_common::tokens_with_subtype(&t, P0, "Bat").len(), 1);
    // Nobody lost life: no Bat.
    let mut t = TestGame::new(2);
    crate::r_s05_common::enter(&mut t, P0, "Bat Whisperer");
    t.resolve_all();
    assert!(crate::r_s05_common::tokens_with_subtype(&t, P0, "Bat").is_empty());
}
