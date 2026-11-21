//! Rulings batch P155 — reflexive triggered abilities (CR 603.12): the target is chosen as
//! the reflexive ability goes on the stack, not as the original ability triggers or is
//! activated, and players may respond to it; choices made as an ability resolves ("you
//! may sacrifice", "you may pay", "you may add") aren't made before; a reflexive ability
//! triggers even if its source left; counters removed "X counters" of chosen kinds; a
//! distributed triggered ability's targets and division chosen as it's put on the stack
//! (CR 601.2d, 603.3d).

use crate::r_p155_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// The target choices asked of `p` since decision `from`.
fn targets_asked(t: &TestGame, p: PlayerId, from: usize) -> usize {
    target_candidates(t, p, from).len()
}

#[test]
fn narset_discards_and_the_reflexive_ability_triggers_after_she_left() {
    cr!("603.12", "608.2h", "113.7a");
    ruling!(
        "Narset of the Ancient Way",
        "You can discard a card even if Narset leaves the battlefield before her middle ability resolves. The reflexive triggered ability will still trigger if you discard a nonland card."
    );
    supported("Narset of the Ancient Way");
    let mut t = TestGame::new(2);
    let narset = t.battlefield(P0, "Narset of the Ancient Way");
    let ae = t.battlefield(P1, "Air Elemental");
    let giant = t.hand(P0, "Hill Giant");
    t.activate(P0, narset, 1, &[]).unwrap();
    t.settle();
    assert_eq!(t.counters(narset, counters::LOYALTY), 2);
    kill(&mut t, narset);
    assert!(t.in_graveyard(P0, "Narset of the Ancient Way"));
    assert_eq!(t.stack_len(), 1);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(giant)]);
    t.answer_targets(P0, &[obj(ae)]);
    t.resolve();
    assert!(t.in_graveyard(P0, "Hill Giant"), "{}", t.dump_log());
    assert_eq!(t.stack_len(), 1, "the reflexive ability: {}", t.dump_log());
    t.resolve_all();
    assert!(
        t.in_graveyard(P1, "Air Elemental"),
        "4 damage: {}",
        t.dump_log()
    );
}

#[test]
fn argentum_masticore_targets_as_the_reflexive_ability_goes_on_the_stack() {
    cr!("603.12", "603.3d");
    ruling!(
        "Argentum Masticore",
        "You don't choose a target for Argentum Masticore's upkeep triggered ability at the time it triggers. Rather, a second \"reflexive\" ability triggers when you discard a card this way. You choose a target for that ability as it goes on the stack. Each player may respond to this triggered ability as normal."
    );
    supported("Argentum Masticore");
    let mut t = TestGame::new(2);
    let masticore = t.battlefield(P0, "Argentum Masticore");
    let giant_card = t.hand(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(giant_card)]);
    t.answer_targets(P0, &[obj(bears)]);
    crate::r_s04_common::next_upkeep(&mut t, P0);
    assert_eq!(t.stack_len(), 1);
    assert_eq!(targets_asked(&t, P0, from), 0, "no target as it triggers");
    t.resolve();
    assert!(t.in_graveyard(P0, "Hill Giant"));
    assert!(t.on_battlefield(masticore));
    // The reflexive ability is on the stack with its target, and can be responded to.
    assert_eq!(t.stack_len(), 1, "{}", t.dump_log());
    assert_eq!(targets_asked(&t, P0, from), 1);
    assert!(t.on_battlefield(bears));
    cast_resolve(&mut t, P1, "Giant Growth", "G", &[obj(bears)]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn cait_sith_targets_as_the_reflexive_ability_goes_on_the_stack() {
    cr!("603.12", "603.3d");
    ruling!(
        "Cait Sith, Fortune Teller",
        "You don't choose a target for Cait Sith's ability at the time it triggers. Rather, a second \"reflexive\" ability triggers when you exile a card this way. You choose a target for that ability as it goes on the stack. Each player may respond to this triggered ability as normal."
    );
    supported("Cait Sith, Fortune Teller");
    let mut t = TestGame::new(2);
    let cait = t.battlefield(P0, "Cait Sith, Fortune Teller");
    t.library_top(P0, "Hill Giant");
    let from = t.asked().len();
    t.answer_targets(P0, &[obj(cait)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    assert_eq!(targets_asked(&t, P0, from), 0, "no target as it triggers");
    t.resolve();
    assert!(t.in_exile("Hill Giant"));
    assert_eq!(t.stack_len(), 1, "the reflexive ability: {}", t.dump_log());
    assert_eq!(targets_asked(&t, P0, from), 1);
    assert_eq!(t.pt(cait), (3, 3), "not yet");
    t.resolve_all();
    assert_eq!(t.pt(cait), (7, 3));
}

#[test]
fn the_ancient_one_targets_as_the_reflexive_ability_goes_on_the_stack() {
    cr!("603.12", "603.3d");
    ruling!(
        "The Ancient One",
        "You don't choose a target for The Ancient One's activated ability at the time you activate it. Rather, a second \"reflexive\" ability triggers when you discard a card this way. You choose a target for that ability as it goes on the stack. Each player may respond to this triggered ability as normal."
    );
    supported("The Ancient One");
    let mut t = TestGame::new(2);
    let one = t.battlefield(P0, "The Ancient One");
    let giant_card = t.hand(P0, "Hill Giant");
    let from = t.asked().len();
    t.answer_choose(P0, &[obj(giant_card)]);
    t.answer_targets(P0, &[pl(P1)]);
    activate(&mut t, P0, one, "Draw a card", "2UB", &[]);
    assert_eq!(targets_asked(&t, P0, from), 0, "no target on activation");
    t.resolve();
    assert!(t.in_graveyard(P0, "Hill Giant"));
    assert_eq!(t.stack_len(), 1, "the reflexive ability: {}", t.dump_log());
    assert_eq!(targets_asked(&t, P0, from), 1);
    assert_eq!(t.library_size(P1), 30);
    t.resolve_all();
    assert_eq!(t.library_size(P1), 26, "milled four");
}

#[test]
fn yunas_whistle_targets_as_the_reflexive_ability_goes_on_the_stack() {
    cr!("603.12", "603.3d");
    ruling!(
        "Yuna's Whistle",
        "You don't choose a target for Yuna's Whistle at the time you cast it. Rather, a second \"reflexive\" ability triggers when you reveal a creature card this way. You choose a target for that ability as it goes on the stack. Each player may respond to this triggered ability as normal."
    );
    supported("Yuna's Whistle");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    stack_library(&mut t, P0, &["Forest", "Hill Giant"]);
    let from = t.asked().len();
    t.answer_targets(P0, &[obj(bears)]);
    let spell = cast(&mut t, P0, "Yuna's Whistle", "1GG", &[]);
    assert!(stack_targets(&t, spell).is_empty());
    assert_eq!(targets_asked(&t, P0, from), 0, "no target as it's cast");
    t.resolve();
    assert!(t.in_hand(P0, "Hill Giant"));
    assert_eq!(t.stack_len(), 1, "the reflexive ability: {}", t.dump_log());
    assert_eq!(targets_asked(&t, P0, from), 1);
    assert_eq!(t.counters(bears, counters::PLUS1), 0);
    t.resolve_all();
    assert_eq!(t.counters(bears, counters::PLUS1), 4);
}

#[test]
fn daretti_targets_as_it_triggers_but_sacrifices_as_it_resolves() {
    cr!("603.3d", "608.2c");
    ruling!(
        "Daretti, Rocketeer Engineer",
        "You choose the target card in your graveyard as Daretti’s second ability is put on the stack, but you don’t choose whether to sacrifice an artifact or which one to sacrifice until the ability resolves."
    );
    supported("Daretti, Rocketeer Engineer");
    let mut t = TestGame::new(2);
    let memnite = t.battlefield(P0, "Memnite");
    let jugg = t.graveyard(P0, "Juggernaut");
    let from = t.asked().len();
    t.answer_targets(P0, &[obj(jugg)]);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(memnite)]);
    t.enter(P0, "Daretti, Rocketeer Engineer");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    assert_eq!(targets_asked(&t, P0, from), 1, "the target as it triggers");
    assert_eq!(yes_nos_asked(&t, P0, from), 0, "no sacrifice decision yet");
    assert!(t.on_battlefield(memnite));
    t.resolve_all();
    assert_eq!(yes_nos_asked(&t, P0, from), 1);
    assert!(t.in_graveyard(P0, "Memnite"));
    assert_eq!(count_named(&t, "Juggernaut"), 1, "{}", t.dump_log());
}

#[test]
fn arni_metalbrow_decides_whether_to_pay_as_the_ability_resolves() {
    cr!("608.2c", "118.12");
    ruling!(
        "Arni Metalbrow",
        "You decide whether to pay {1}{R} as the triggered ability resolves."
    );
    supported("Arni Metalbrow");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Arni Metalbrow");
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.hand(P0, "Grizzly Bears");
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(bears)]);
    attack_with(&mut t, &[(giant, pl(P1))]);
    assert_eq!(t.stack_len(), 1);
    assert_eq!(yes_nos_asked(&t, P0, from), 0, "not asked as it triggers");
    // The mana is there as the ability resolves.
    mana(&mut t, P0, "1R");
    assert_eq!(pool(&t, P0), 2);
    t.resolve_all();
    assert!(
        yes_nos_asked(&t, P0, from) >= 1,
        "{:?}",
        asked_of(&t, P0, from)
    );
    assert_eq!(pool(&t, P0), 0, "{{1}}{{R}} paid");
    assert!(t.on_battlefield(bears), "{}", t.dump_log());
    assert!(t.obj_now(bears).tapped);
    assert!(crate::r_p195_common::attacking(&t, bears));
}

#[test]
fn scattering_stroke_adds_all_the_mana_or_none_as_the_delayed_ability_resolves() {
    cr!("603.7", "608.2c", "505.1");
    ruling!(
        "Scattering Stroke",
        "You decide whether or not to add that much {C} as the delayed triggered ability resolves at the beginning of your next main phase. You don't decide before then."
    );
    ruling!(
        "Scattering Stroke",
        "You can't add just some of the mana. You either add all the mana or none of it."
    );
    supported("Scattering Stroke");
    for add in [true, false] {
        let mut t = TestGame::new(2);
        t.library_top(P0, "Craw Wurm");
        t.library_top(P1, "Forest");
        t.set_step(P1, Step::PrecombatMain);
        let giant = cast(&mut t, P1, "Hill Giant", "3R", &[]);
        let from = t.asked().len();
        cast(&mut t, P0, "Scattering Stroke", "2UU", &[obj(giant)]);
        t.resolve_all();
        assert!(t.in_graveyard(P1, "Hill Giant"));
        // Craw Wurm (6) beats Forest (0): the delayed trigger waits for P0's next main
        // phase; nothing is asked of P0 but the clash's choice until then.
        assert_eq!(
            yes_nos_asked(&t, P0, from),
            1,
            "{:?}",
            asked_of(&t, P0, from)
        );
        t.answer_yes(P0, add);
        t.advance_to(P0, Step::PrecombatMain);
        t.settle();
        assert_eq!(t.stack_len(), 1, "add: {add}: {}", t.dump_log());
        assert_eq!(yes_nos_asked(&t, P0, from), 1);
        t.resolve_all();
        assert_eq!(yes_nos_asked(&t, P0, from), 2);
        assert_eq!(pool(&t, P0), if add { 4 } else { 0 }, "add: {add}");
        assert!(
            !asked_of(&t, P0, from)
                .iter()
                .any(|d| matches!(d, Decision::ChooseNumber { .. })),
            "all or none: no amount asked"
        );
    }
}

#[test]
fn cemetery_desecrator_removes_x_counters_of_chosen_kinds() {
    cr!("603.12", "608.2c");
    ruling!(
        "Cemetery Desecrator",
        "The controller of Cemetery Desecrator chooses which counters are removed, and they may remove more than one kind of counter. If the target permanent has fewer than X counters, all of them are removed."
    );
    supported("Cemetery Desecrator");
    // Centaur Courser (3) exiled: three counters from P1's Grizzly Bears with two +1/+1
    // and two shield counters: two +1/+1 and one shield.
    let mut t = TestGame::new(2);
    let courser = t.graveyard(P0, "Centaur Courser");
    let bears = t.battlefield(P1, "Grizzly Bears");
    put(&mut t, bears, counters::PLUS1, 2);
    put(&mut t, bears, "shield", 2);
    t.answer_choose(P0, &[obj(courser)]);
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0]));
    t.answer_targets(P0, &[obj(bears)]);
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.enter(P0, "Cemetery Desecrator");
    t.resolve_all();
    assert_eq!(t.zone(courser), Zone::Exile);
    assert_eq!(t.counters(bears, counters::PLUS1), 0, "{}", t.dump_log());
    assert_eq!(t.counters(bears, "shield"), 1);
    // With one counter, it's removed and nothing is asked about which.
    let mut t = TestGame::new(2);
    let courser = t.graveyard(P0, "Centaur Courser");
    let bears = t.battlefield(P1, "Grizzly Bears");
    put(&mut t, bears, counters::PLUS1, 1);
    let from = t.asked().len();
    t.answer_choose(P0, &[obj(courser)]);
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0]));
    t.answer_targets(P0, &[obj(bears)]);
    t.enter(P0, "Cemetery Desecrator");
    t.resolve_all();
    assert_eq!(t.counters(bears, counters::PLUS1), 0);
    assert!(
        !asked_of(&t, P0, from)
            .iter()
            .any(|d| matches!(d, Decision::ChooseOption { .. })),
        "{:?}",
        asked_of(&t, P0, from)
    );
}

#[test]
fn jade_seedstones_chooses_its_targets_and_the_division_as_it_goes_on_the_stack() {
    cr!("601.2d", "603.3d", "608.2c");
    ruling!(
        "Jade Seedstones // Jadeheart Attendant",
        "You choose how many targets the first ability of Jade Seedstones has and how the counters will be distributed as that ability is put on the stack.  Each target must receive at least one counter."
    );
    supported("Jade Seedstones // Jadeheart Attendant");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let from = t.asked().len();
    t.answer_targets(P0, &[obj(bears), obj(giant)]);
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![2, 1]));
    t.enter(P0, "Jade Seedstones // Jadeheart Attendant");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    let divided = t.asked()[from..]
        .iter()
        .filter(|(p, d)| *p == P0 && matches!(d, Decision::Divide { .. }))
        .count();
    assert_eq!(
        divided,
        1,
        "divided as it was put on the stack: {:?}",
        asked_of(&t, P0, from)
    );
    t.resolve_all();
    assert_eq!(t.counters(bears, counters::PLUS1), 2);
    assert_eq!(t.counters(giant, counters::PLUS1), 1);
    // One target: all three.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Hill Giant");
    t.answer_targets(P0, &[obj(bears)]);
    t.enter(P0, "Jade Seedstones // Jadeheart Attendant");
    t.resolve_all();
    assert_eq!(t.counters(bears, counters::PLUS1), 3);
}
