//! Rulings batch P032 — values determined as an ability resolves and kept afterwards (CR
//! 608.2h, 611.2c), last known information (CR 608.2h), "unless" choices (CR 608.2d),
//! simultaneous entry timestamps (CR 613.7m), and a legendary permanent put into the
//! graveyard by the legend rule before its own "enters" trigger is put on the stack (CR
//! 704.5j, 603.3).

use crate::r_p021_common::giant_growth;
use crate::r_p032_common::*;
use crate::r_s01_common::{asked_since, supported};
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn resilient_khenra_fixes_x_on_resolution() {
    cr!("608.2h", "611.2c");
    ruling!(
        "Resilient Khenra",
        "The value of X is determined only as Resilient Khenra's triggered ability resolves. Once that happens, the value of X won't change later in the turn even if Resilient Khenra's power changes."
    );
    supported("Resilient Khenra");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    t.answer_yes(P0, true);
    let khenra = t.enter(P0, "Resilient Khenra");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    // In response, Khenra gets +3/+3: X is 5 on resolution.
    giant_growth(&mut t, P0, khenra);
    assert_eq!(t.pt(bears), (7, 7));
    // Khenra's power changes again: the Bears keep +5/+5.
    giant_growth(&mut t, P0, khenra);
    assert_eq!(t.pt(khenra), (8, 8));
    assert_eq!(t.pt(bears), (7, 7));
}

#[test]
fn champion_of_wits_uses_power_on_resolution_or_last_known() {
    cr!("608.2h", "603.10");
    ruling!(
        "Champion of Wits",
        "Use Champion of Wits's power as the first ability resolves to determine if you want to use the ability and how many cards you draw if you do. If you use the ability, you discard two cards regardless of how many cards you draw. If Champion of Wits leaves the battlefield before that ability resolves, use its last known power."
    );
    supported("Champion of Wits");
    // Pumped to 5 power in response: draw five, discard two.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.answer_yes(P0, true);
    let champ = t.enter(P0, "Champion of Wits");
    t.settle();
    let hand = t.hand_size(P0);
    let gy = t.graveyard_size(P0);
    // (Giant Growth resolves, then the trigger.)
    giant_growth(&mut t, P0, champ);
    assert_eq!(t.hand_size(P0), hand + 5 - 2);
    // The two discarded cards and Giant Growth.
    assert_eq!(t.graveyard_size(P0), gy + 3);
    // It leaves the battlefield first: its last known power (2).
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.answer_yes(P0, true);
    let champ = t.enter(P0, "Champion of Wits");
    t.settle();
    crate::r_s02_common::destroy(&mut t, champ);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2 - 2);
    // Power 0 (made 0/1 by -2/-0 in response): draw nothing, still discard two.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.answer_yes(P0, true);
    let champ = t.enter(P0, "Champion of Wits");
    t.settle();
    crate::r_p021_common::modify(
        &mut t,
        champ,
        vec![mtg_engine::ability::Modification::ModifyPT(
            mtg_engine::ability::Value::c(-2),
            mtg_engine::ability::Value::c(0),
        )],
    );
    for _ in 0..3 {
        t.hand(P0, "Island");
    }
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand - 2);
}

#[test]
fn thornplate_intimidator_opponent_may_always_lose_3_life() {
    cr!("608.2d");
    ruling!(
        "Thornplate Intimidator",
        "While resolving Thornplate Intimidator’s last ability, the target opponent chooses a card to be discarded without revealing it, chooses a nonland permanent to be sacrificed, or chooses to do neither. Then that player discards that card, sacrifices that permanent, or loses 3 life. That player can always choose to lose 3 life, even if they have cards to discard or nonland permanents to sacrifice."
    );
    supported("Thornplate Intimidator");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let card = t.hand(P1, "Island");
    let life = t.life(P1);
    let from = t.asked().len();
    t.answer_targets(P0, &[Entity::Player(P1)]);
    // P1 declines both: no discard, no sacrifice.
    t.answer(P1, DecisionKind::YesNo, Answer::Bool(false));
    t.answer(P1, DecisionKind::YesNo, Answer::Bool(false));
    t.answer(P1, DecisionKind::Entities, Answer::Entities(vec![]));
    t.answer(P1, DecisionKind::Entities, Answer::Entities(vec![]));
    t.enter(P0, "Thornplate Intimidator");
    t.resolve_all();
    let asked: Vec<_> = asked_since(&t, from)
        .into_iter()
        .filter(|(p, _)| *p == P1)
        .collect();
    assert!(!asked.is_empty(), "P1 chose nothing");
    assert_eq!(t.life(P1), life - 3, "{asked:?}");
    assert!(t.on_battlefield(bears));
    assert_eq!(t.zone(card), Zone::Hand(P1));
}

#[test]
fn twilights_call_creatures_get_timestamps_in_apnap_order() {
    cr!("613.7m", "101.4");
    ruling!(
        "Twilight's Call",
        "For purposes of ordering any continuous effects of the creatures, the current player (not the controller of this spell) decides the ordering regardless of which player is putting the creatures onto the battlefield."
    );
    supported("Twilight's Call");
    // P1 casts Twilight's Call during P0's turn (paying {2} more for flash). P0, the
    // active player, orders P0's creatures; they get earlier timestamps than P1's.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let mine = [t.graveyard(P0, "Grizzly Bears"), t.graveyard(P0, "Hill Giant")];
    let theirs = [t.graveyard(P1, "Craw Wurm"), t.graveyard(P1, "Air Elemental")];
    t.lands(P1, "Swamp", 2);
    t.lands(P1, "Wastes", 6);
    let call = t.hand(P1, "Twilight's Call");
    let methods = crate::r_s08_common::legal_cast_methods(&mut t, P1, call);
    let flash = methods
        .into_iter()
        .find(|m| *m != mtg_engine::object::CastMethod::Normal)
        .expect("a way to cast it as though it had flash");
    // P0 orders the Hill Giant first.
    t.answer(P0, DecisionKind::Order, Answer::Indices(vec![1, 0]));
    let from = t.asked().len();
    t.cast(P1, call).method(flash).go();
    t.resolve_all();
    let ts = |t: &TestGame, id: ObjectId| t.obj_now(id).timestamp;
    let orders: Vec<PlayerId> = asked_since(&t, from)
        .into_iter()
        .filter(|(_, d)| matches!(d, Decision::Order { .. }))
        .map(|(p, _)| p)
        .collect();
    assert_eq!(orders.first(), Some(&P0));
    assert!(ts(&t, mine[1]) < ts(&t, mine[0]));
    for m in mine {
        for th in theirs {
            assert!(t.on_battlefield(m) && t.on_battlefield(th));
            assert!(ts(&t, m) < ts(&t, th));
        }
    }
}

#[test]
fn sharuum_can_return_itself_after_the_legend_rule() {
    cr!("704.5j", "603.3", "603.6a");
    ruling!(
        "Sharuum the Hegemon",
        "If Sharuum the Hegemon is put into your graveyard as a state-based action immediately after Sharuum the Hegemon enters the battlefield (most likely due to the \"legend rule\") it can be the artifact card targeted by its own ability."
    );
    supported("Sharuum the Hegemon");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let old = t.battlefield(P0, "Sharuum the Hegemon");
    let from = t.asked().len();
    // The legend rule: keep the old one.
    t.answer_choose(P0, &[obj(old)]);
    let new = t.enter(P0, "Sharuum the Hegemon");
    t.answer_yes(P0, true);
    t.settle();
    assert_eq!(t.zone(new), Zone::Graveyard(P0));
    let dead_now = t.g.current(new);
    // The trigger could target the new Sharuum, now a card in the graveyard.
    let offered = crate::r_s02_common::target_candidates(&t, P0, from);
    assert!(
        offered.iter().any(|c| c.contains(&obj(dead_now))),
        "{offered:?}"
    );
    assert!(t.on_battlefield(old));
}
