//! Rulings batch P136 (removal) — "each player sacrifices": the players choose in APNAP
//! order knowing the earlier choices, then everything is sacrificed at once (CR 101.4,
//! 701.21a); choices made as the ability resolves (CR 603.7c, 608.2d); processes repeated
//! during one resolution with no player acting in between (CR 608.2, 603.3).

use crate::r_p136_common::*;
use crate::r_s01_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn is_choose(d: &Decision) -> bool {
    matches!(d, Decision::ChooseEntities { .. })
}

/// The players asked to choose objects since decision `from`, in order.
fn choosers_since(t: &TestGame, from: usize) -> Vec<PlayerId> {
    asked_since(t, from)
        .into_iter()
        .filter(|(_, d)| is_choose(d))
        .map(|(p, _)| p)
        .collect()
}

/// The candidates of the first object choice `p` was asked since `from`.
fn first_offer(t: &TestGame, p: PlayerId, from: usize) -> Vec<ObjectId> {
    asked_since(t, from)
        .into_iter()
        .find_map(|(q, d)| match d {
            Decision::ChooseEntities { candidates, .. } if q == p => {
                Some(candidates.iter().filter_map(|e| e.object()).collect())
            }
            _ => None,
        })
        .unwrap_or_default()
}

/// Watches every player's object choices, recording the number of permanents then.
fn watch_all(t: &mut TestGame, players: &[PlayerId]) -> Vec<Seen<usize>> {
    players
        .iter()
        .map(|p| watch(t, *p, is_choose, |g| g.permanents().count()))
        .collect()
}

fn card_type_index(ty: CardType) -> usize {
    CardType::ALL.iter().position(|c| *c == ty).unwrap()
}

// ---------------------------------------------------------------------------
// World Queller
// ---------------------------------------------------------------------------

/// A three-player game on P0's turn before the upkeep: P0 controls World Queller, P1 two
/// creatures, P2 one.
fn queller_game() -> (TestGame, ObjectId, [ObjectId; 3]) {
    let mut t = TestGame::new(3);
    t.set_step(P0, Step::Untap);
    let q = t.battlefield(P0, "World Queller");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    let c = t.battlefield(P2, "Gray Ogre");
    (t, q, [a, b, c])
}

fn to_upkeep(t: &mut TestGame) {
    t.advance_to(P0, Step::Upkeep);
    t.settle();
}

#[test]
fn world_queller_chooses_in_order_then_sacrifices_together() {
    cr!("101.4", "701.21a");
    ruling!(
        "World Queller",
        "First you choose which permanent you'll sacrifice (if you control any of the chosen type), then each other player in turn order does the same, then all chosen permanents are sacrificed at the same time."
    );
    supported("World Queller");
    let (mut t, q, [a, b, c]) = queller_game();
    let p0_bear = t.battlefield(P0, "Savannah Lions");
    to_upkeep(&mut t);
    t.answer_yes(P0, true);
    t.answer(
        P0,
        DecisionKind::Option,
        Answer::Index(card_type_index(CardType::Creature)),
    );
    t.answer_choose(P0, &[o(q)]);
    t.answer_choose(P1, &[o(b)]);
    let seen = watch_all(&mut t, &[P0, P1, P2]);
    let before = t.g.permanents().count();
    let from = t.asked().len();
    t.resolve_all();
    let order = choosers_since(&t, from);
    assert_eq!(order[..2], [P0, P1]);
    for s in &seen[..2] {
        assert_eq!(s.lock().unwrap().first().copied(), Some(before));
    }
    assert!(t.on_battlefield(p0_bear) && t.on_battlefield(a));
    for id in [q, b, c] {
        assert!(!t.on_battlefield(id));
    }
}

#[test]
fn world_queller_type_is_chosen_on_resolution_and_may_be_any_card_type() {
    cr!("603.7c", "608.2d", "205.2a");
    ruling!(
        "World Queller",
        "You don't choose a card type until the ability resolves. Once you choose a type, it's too late for players to respond."
    );
    ruling!(
        "World Queller",
        "You may choose any card type. Choosing anything other than artifact, battle, creature, enchantment, kindred, land, or planeswalker will result in no permanents being sacrificed"
    );
    let (mut t, _q, [a, b, c]) = queller_game();
    let from = t.asked().len();
    to_upkeep(&mut t);
    // On the stack: no type chosen yet.
    assert_eq!(triggers_on_stack(&t, "choose a card type"), 1);
    assert!(asked_since(&t, from)
        .iter()
        .all(|(_, d)| !matches!(d, Decision::ChooseOption { .. })));
    t.answer_yes(P0, true);
    t.answer(
        P0,
        DecisionKind::Option,
        Answer::Index(card_type_index(CardType::Instant)),
    );
    let from = t.asked().len();
    t.resolve_all();
    // The type was offered among all card types; nothing was sacrificed, and no
    // priority was given between the choice and the sacrifice.
    let opts = asked_since(&t, from)
        .into_iter()
        .find_map(|(_, d)| match d {
            Decision::ChooseOption { options, .. } => Some(options),
            _ => None,
        })
        .unwrap();
    assert!(opts.iter().any(|o| o.eq_ignore_ascii_case("instant")));
    assert!(opts.iter().any(|o| o.eq_ignore_ascii_case("sorcery")));
    for id in [a, b, c] {
        assert!(t.on_battlefield(id));
    }
}

// ---------------------------------------------------------------------------
// Plaguecrafter and Demon's Disciple
// ---------------------------------------------------------------------------

#[test]
fn plaguecrafter_each_player_chooses_then_those_who_cant_discard() {
    cr!("101.4", "701.21a", "701.9b");
    ruling!(
        "Plaguecrafter",
        "As Plaguecrafter's ability resolves, first the player whose turn it is chooses a creature or planeswalker they control, then each other player in turn order does the same, knowing the choices made before them. Then all the chosen permanents are sacrificed at the same time. Next, each player in the same order who couldn't sacrifice a permanent chooses a card in hand without revealing it, then the chosen cards are discarded at the same time."
    );
    ruling!(
        "Plaguecrafter",
        "Each player chooses a permanent to sacrifice from among the creatures and planeswalkers they control. You don't choose which type of permanent any other player has to sacrifice."
    );
    supported("Plaguecrafter");
    let mut t = TestGame::new(3);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let walker = t.battlefield(P1, "Liliana, Dreadhorde General");
    let creature = t.battlefield(P1, "Hill Giant");
    // P2 controls neither and discards; P0 keeps its hand.
    t.hand(P2, "Lightning Bolt");
    t.hand(P0, "Shock");
    t.answer_choose(P0, &[o(mine)]);
    t.answer_choose(P1, &[o(walker)]);
    let seen = watch_all(&mut t, &[P0, P1]);
    let from = t.asked().len();
    let pc = t.enter(P0, "Plaguecrafter");
    t.settle();
    let before = t.g.permanents().count();
    t.resolve_all();
    assert_eq!(choosers_since(&t, from)[..2], [P0, P1]);
    for s in &seen {
        assert_eq!(s.lock().unwrap().first().copied(), Some(before));
    }
    // P1 chose which of its two kinds to sacrifice.
    let offered = first_offer(&t, P1, from);
    assert!(offered.contains(&walker) && offered.contains(&creature));
    assert!(t.on_battlefield(pc) && t.on_battlefield(creature));
    assert!(!t.on_battlefield(mine) && !t.on_battlefield(walker));
    assert_eq!(t.hand_size(P2), 0);
    assert_eq!(t.hand_size(P0), 1);
}

#[test]
fn demons_disciple_can_be_sacrificed_to_its_own_ability() {
    cr!("101.4", "701.21a");
    ruling!(
        "Demon's Disciple",
        "Demon's Disciple can be the creature sacrificed for its own ability."
    );
    ruling!(
        "Demon's Disciple",
        "As the triggered ability resolves, first the player whose turn it is chooses a creature or planeswalker they control, then each other player in turn order does the same, knowing the choices made before them. Then all the chosen creatures and/or planeswalkers are sacrificed at the same time."
    );
    supported("Demon's Disciple");
    let mut t = TestGame::new(3);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    let c = t.battlefield(P2, "Liliana, Dreadhorde General");
    t.answer_choose(P1, &[o(b)]);
    let seen = watch_all(&mut t, &[P1, P2]);
    let from = t.asked().len();
    let dd = t.enter(P0, "Demon's Disciple");
    t.settle();
    let before = t.g.permanents().count();
    t.resolve_all();
    let order = choosers_since(&t, from);
    // P0 (whose only creature is the Disciple) chooses before P1.
    let p1_at = order.iter().position(|p| *p == P1).unwrap();
    if let Some(p0_at) = order.iter().position(|p| *p == P0) {
        assert!(p0_at < p1_at);
    }
    assert_eq!(seen[0].lock().unwrap().first().copied(), Some(before));
    // The Disciple, P0's only creature, was sacrificed.
    assert!(!t.on_battlefield(dd));
    assert!(t.in_graveyard(P0, "Demon's Disciple"));
    assert!(t.on_battlefield(a));
    assert!(!t.on_battlefield(b) && !t.on_battlefield(c));
}

// ---------------------------------------------------------------------------
// Forbidden Ritual
// ---------------------------------------------------------------------------

#[test]
fn forbidden_ritual_target_on_casting_sacrifice_on_resolution_same_target_on_repeat() {
    cr!("601.2c", "608.2c");
    ruling!(
        "Forbidden Ritual",
        "You choose the target opponent on casting."
    );
    ruling!(
        "Forbidden Ritual",
        "You sacrifice during resolution, not during casting."
    );
    ruling!(
        "Forbidden Ritual",
        "If you choose to repeat the process, you repeat the effect but do not get to pick a different target opponent."
    );
    supported("Forbidden Ritual");
    let mut t = TestGame::new(3);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    let from = t.asked().len();
    let spell = cast(&mut t, P0, "Forbidden Ritual", &[&[Entity::Player(P2)]]);
    // A target was chosen while casting; nothing was sacrificed yet.
    assert!(asked_since(&t, from)
        .iter()
        .any(|(p, d)| *p == P0 && matches!(d, Decision::ChooseTargets { .. })));
    assert!(t.on_battlefield(a) && t.on_battlefield(b));
    let _ = spell;
    // On resolution: sacrifice the Bears, P2 (no permanents, no cards) loses 2; repeat
    // with the Giant, P2 loses 2 again; then stop.
    t.answer_choose(P0, &[o(a)]);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[o(b)]);
    t.answer_yes(P0, false);
    let from = t.asked().len();
    t.resolve_all();
    assert!(!t.on_battlefield(a) && !t.on_battlefield(b));
    assert_eq!(t.life(P2), 16);
    assert_eq!(t.life(P1), 20);
    assert!(asked_since(&t, from)
        .iter()
        .all(|(_, d)| !matches!(d, Decision::ChooseTargets { .. })));
}

// ---------------------------------------------------------------------------
// Remorseless Punishment
// ---------------------------------------------------------------------------

#[test]
fn remorseless_punishment_triggers_wait_until_it_finishes() {
    cr!("603.3", "608.2");
    ruling!(
        "Remorseless Punishment",
        "Any abilities that trigger while Remorseless Punishment is resolving will be put on the stack after it has completely resolved."
    );
    ruling!(
        "Remorseless Punishment",
        "No player may take any actions between the two times the player goes through the process."
    );
    supported("Remorseless Punishment");
    supported("Doomed Traveler");
    let mut t = TestGame::new(2);
    let traveler = t.battlefield(P1, "Doomed Traveler");
    // P1 has no cards: the first time, it sacrifices Doomed Traveler. Its Spirit isn't
    // created until Remorseless Punishment has finished, so the second time P1 has
    // nothing to sacrifice and loses 5 life.
    t.answer_yes(P1, true);
    t.answer_choose(P1, &[o(traveler)]);
    t.answer_yes(P1, true);
    cast(&mut t, P0, "Remorseless Punishment", &[&[Entity::Player(P1)]]);
    t.resolve();
    assert!(t.in_graveyard(P1, "Doomed Traveler"));
    assert_eq!(t.life(P1), 15);
    assert_eq!(triggers_on_stack(&t, "Spirit"), 1);
    t.resolve_all();
    assert_eq!(with_subtype(&t, P1, "Spirit").len(), 1);
}
