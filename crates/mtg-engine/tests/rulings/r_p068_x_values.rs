//! Rulings batch P068 — {X} in the mana cost of cards cast from exile and of the objects
//! these cards look at: a card cast without paying its mana cost has X = 0 (CR 107.3b), a
//! spell's mana value counts the chosen X (CR 202.3e) and a permanent's or card's counts it
//! as 0.

use crate::r_p068_common::*;
use crate::r_s01_common::{attack_with, block_and_finish, stack_library, supported};
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::mana::ManaType::*;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::*;

/// Queues `p`'s answer of 3 for any X it's asked to choose.
fn answer_x3(t: &mut TestGame, p: PlayerId) {
    t.answer(p, DecisionKind::X, Answer::Number(3));
}

/// P1 casts Lightning Bolt at P0; P0 counters it with Tibalt's Trickery; P1's library has
/// `card` under three Forests (at most three cards are milled).
fn tibalt(t: &mut TestGame, card: &str) -> ObjectId {
    let lib = stack_library(t, P1, &["Forest", "Forest", "Forest", card, "Hill Giant"]);
    let bolt = t.hand(P1, "Lightning Bolt");
    mana(t, P1, R, 1);
    t.g.turn.priority = Some(P1);
    let spell = t.cast(P1, bolt).target(P0).go();
    let tt = t.hand(P0, "Tibalt's Trickery");
    mana(t, P0, R, 2);
    t.g.turn.priority = Some(P0);
    t.cast(P0, tt).target(spell).go();
    t.resolve();
    lib[3]
}

/// P0 casts Transforming Flourish (not copying it) at P1's Grizzly Bears; P1's library
/// has `card` under a Forest.
fn flourish(t: &mut TestGame, card: &str) -> ObjectId {
    let lib = stack_library(t, P1, &["Forest", card]);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let tf = t.hand(P0, "Transforming Flourish");
    mana(t, P0, R, 3);
    t.answer_yes(P0, false);
    let tf = t.cast(P0, tf).target(bears).go();
    t.settle();
    while t.zone(tf) == Zone::Stack {
        t.resolve();
    }
    lib[1]
}

#[test]
fn blaze_cast_without_paying_its_mana_cost_has_x_0() {
    cr!("107.3b", "118.9", "601.2b");
    ruling!(
        "Transforming Flourish",
        "If the exiled card has {X} in its mana cost, the destroyed permanent’s controller must choose 0 as the value of X when casting it without paying its mana cost."
    );
    ruling!(
        "Dazzling Sphinx",
        "If the exiled instant or sorcery card has an {X} in its mana cost, you must choose 0 as the value of {X}."
    );
    ruling!(
        "Tibalt's Trickery",
        "If the spell has {X} in its mana cost, they must choose 0 as the value of X."
    );
    ruling!(
        "Muse Vortex",
        "If you cast a spell with {X} in its cost without paying its mana cost, you must choose 0 as the value of X."
    );
    for name in [
        "Transforming Flourish",
        "Dazzling Sphinx",
        "Tibalt's Trickery",
        "Muse Vortex",
    ] {
        supported(name);
    }
    // Transforming Flourish: P1 casts Blaze at P0, answering X = 3.
    let mut t = TestGame::new(2);
    t.answer_yes(P1, true);
    answer_x3(&mut t, P1);
    t.answer_targets(P1, &[Entity::Player(P0)]);
    let blaze = flourish(&mut t, "Blaze");
    t.resolve_all();
    assert_eq!(t.zone(blaze), Zone::Graveyard(P1), "{}", t.dump_log());
    assert_eq!(t.life(P0), 20);
    // Tibalt's Trickery: likewise.
    let mut t = TestGame::new(2);
    t.answer_yes(P1, true);
    answer_x3(&mut t, P1);
    t.answer_targets(P1, &[Entity::Player(P0)]);
    let blaze = tibalt(&mut t, "Blaze");
    t.resolve_all();
    assert_eq!(t.zone(blaze), Zone::Graveyard(P1), "{}", t.dump_log());
    assert_eq!(t.life(P0), 20);
    // Dazzling Sphinx: P0 casts P1's Blaze at P1.
    let mut t = TestGame::new(2);
    let sphinx = t.battlefield(P0, "Dazzling Sphinx");
    let blaze = t.library_top(P1, "Blaze");
    t.answer_yes(P0, true);
    answer_x3(&mut t, P0);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    attack_with(&mut t, &[(sphinx, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[]);
    t.resolve_all();
    assert_eq!(t.zone(blaze), Zone::Graveyard(P1), "{}", t.dump_log());
    assert_eq!(t.life(P1), 16, "only the Sphinx's combat damage");
    // Muse Vortex with X = 1 exiles Blaze (mana value 1 in exile) and casts it.
    let mut t = TestGame::new(2);
    let blaze = t.library_top(P0, "Blaze");
    let mv = t.hand(P0, "Muse Vortex");
    pool(&mut t, P0, &[U, U, C]);
    t.answer(P0, DecisionKind::X, Answer::Number(1));
    t.answer_choose_moved(P0, &[o(blaze)]);
    t.answer_yes(P0, true);
    answer_x3(&mut t, P0);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.cast(P0, mv).go();
    t.resolve_all();
    assert_eq!(t.zone(blaze), Zone::Graveyard(P0), "{}", t.dump_log());
    assert_eq!(t.life(P1), 20);
}

#[test]
fn herald_of_amity_casts_an_x_aura_with_x_0() {
    cr!("107.3b", "118.9");
    ruling!(
        "Herald of Amity",
        "If the exiled Aura has {X} in its mana cost, you must choose 0 as the value of X when casting it without paying its mana cost."
    );
    supported("Herald of Amity");
    // Awakened Awareness ({X}{U}{U}): "When this Aura enters, put X +1/+1 counters on
    // enchanted permanent."
    let mut t = TestGame::new(2);
    let aa = t.library_top(P0, "Awakened Awareness");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_choose_moved(P0, &[o(aa)]);
    t.answer_yes(P0, true);
    answer_x3(&mut t, P0);
    t.answer_targets(P0, &[o(bears)]);
    t.enter(P0, "Herald of Amity");
    t.resolve_all();
    assert!(t.on_battlefield(t.g.current(aa)), "{}", t.dump_log());
    assert_eq!(t.counters(bears, "+1/+1"), 0);
    assert_eq!(t.pt(bears), (1, 1));
}

#[test]
fn gales_redirection_counts_the_exiled_spells_x() {
    cr!("202.3e", "706.2");
    ruling!(
        "Gale's Redirection",
        "If the spell had {X} in its mana cost, X will be whatever value it had on the stack when calculating its mana value."
    );
    supported("Gale's Redirection");
    // P1 casts Blaze with X = 14 (mana value 15): any roll is 16 or more, so P0 may cast
    // it without paying its mana cost. Over several rolls: with the mana value 1 Blaze has
    // off the stack, a roll under 14 would give the 1-14 result.
    for seed in 0..8 {
        let mut t = TestGame::new(2);
        t.g.rng = <rand_chacha::ChaCha8Rng as rand::SeedableRng>::seed_from_u64(seed);
        let blaze = t.hand(P1, "Blaze");
        t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
        mana(&mut t, P1, R, 15);
        t.g.turn.priority = Some(P1);
        let spell = t.cast(P1, blaze).x(14).target(P0).go();
        pool(&mut t, P0, &[U, U, C, C, C]);
        let gr = t.hand(P0, "Gale's Redirection");
        t.g.turn.priority = Some(P0);
        t.cast(P0, gr).target(spell).go();
        t.resolve();
        assert!(exiled(&t, blaze), "{}", t.dump_log());
        assert_eq!(t.life(P0), 20);
        // In P0's main phase: only without paying its mana cost (the 15+ result).
        t.set_step(P0, mtg_engine::turn::Step::PostcombatMain);
        assert_eq!(
            methods(&mut t, P0, blaze),
            vec![CastMethod::Free],
            "seed {seed}: {}",
            t.dump_log()
        );
    }
}

#[test]
fn zoyowas_justice_counts_x_as_0_for_a_permanent() {
    cr!("202.3e", "115.1", "701.57a");
    ruling!(
        "Zoyowa's Justice",
        "If the target artifact or creature has X in its mana cost, that X is 0 when determining its mana value."
    );
    supported("Zoyowa's Justice");
    let mut t = TestGame::new(2);
    // Hangarback Walker ({X}{X}) has mana value 0: not a legal target. Hydroid Krasis
    // ({X}{G}{U}) has mana value 2: P1 discovers 2.
    let walker = t.battlefield(P1, "Hangarback Walker");
    t.g.obj_mut(walker).counters.insert("+1/+1".into(), 2);
    let krasis = t.battlefield(P1, "Hydroid Krasis");
    t.g.obj_mut(krasis).counters.insert("+1/+1".into(), 5);
    t.g.players[1].library.clear();
    stack_library(&mut t, P1, &["Hill Giant", "Hill Giant", "Grizzly Bears", "Hill Giant"]);
    let zj = t.hand(P0, "Zoyowa's Justice");
    pool(&mut t, P0, &[R, C]);
    t.answer_yes(P1, false);
    let from = t.asked().len();
    t.cast(P0, zj).target(krasis).go();
    let offered: Vec<Entity> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseTargets { candidates, .. } => Some(candidates.clone()),
            _ => None,
        })
        .flatten()
        .collect();
    assert!(offered.contains(&o(krasis)));
    assert!(!offered.contains(&o(walker)));
    t.resolve_all();
    // Grizzly Bears or the Krasis itself (both mana value 2) went to P1's hand, not a
    // Hill Giant (mana value 4).
    let hand: Vec<&str> = t
        .g
        .player(P1)
        .hand
        .iter()
        .map(|x| t.g.obj(*x).chars.name.as_str())
        .collect();
    assert_eq!(hand.len(), 1, "{}", t.dump_log());
    assert!(
        hand[0] == "Grizzly Bears" || hand[0] == "Hydroid Krasis",
        "{hand:?}"
    );
}

#[test]
fn hurl_into_history_counts_the_chosen_x() {
    cr!("202.3e", "701.57a");
    ruling!(
        "Hurl into History",
        "If the target spell has {X} in its mana cost, use the value chosen for X when calculating that spell's mana value."
    );
    supported("Hurl into History");
    // P1 casts Hydroid Krasis with X = 3 (mana value 5). P0 discovers 5 and finds Hill
    // Giant (mana value 4), not the filler under it.
    let mut t = TestGame::new(2);
    let giant = t.library_top(P0, "Hill Giant");
    let k = t.hand(P1, "Hydroid Krasis");
    t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
    pool(&mut t, P1, &[G, U, C, C, C]);
    t.g.turn.priority = Some(P1);
    let spell = t.cast(P1, k).x(3).go();
    t.settle();
    // The cast trigger resolves first.
    t.resolve();
    let hurl = t.hand(P0, "Hurl into History");
    pool(&mut t, P0, &[U, U, C, C, C]);
    t.answer_yes(P0, false);
    t.g.turn.priority = Some(P0);
    t.cast(P0, hurl).target(spell).go();
    t.resolve();
    assert_eq!(t.zone(spell), Zone::Graveyard(P1), "{}", t.dump_log());
    assert_eq!(t.zone(giant), Zone::Hand(P0), "{}", t.dump_log());
}
