//! Rulings batch P090 — impulsive draw: cards exiled from the top of a library that may be
//! played for a while (Magus of the Mind, Aminatou's Augury, Act on Impulse, Irascible
//! Wolverine): free casting (CR 118.9), timing (CR 305.1, 305.2, 307.1), the land put
//! onto the battlefield on resolution (CR 305.4), and Gunner Conscript's bonus (CR 120.6).

use crate::r_s01_common::*;
use crate::r_s02_common::{can_cast, can_play_land};
use crate::r_s04_common::add_mana;
use crate::r_s11_common::empty_library;
use crate::r_s22_common::{check_free_cast_costs, FreeCaster};
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Runs `f` with every card but the top one taken out of P0's library, then puts them
/// back under it (so an effect that shuffles and exiles the top card exiles that one).
fn with_only_top(t: &mut TestGame, f: impl FnOnce(&mut TestGame)) {
    let lib = t.g.player(P0).library.clone();
    let Some((top, rest)) = lib.split_last() else {
        f(t);
        return;
    };
    t.g.players[P0.idx()].library = vec![*top];
    f(t);
    let mut now = rest.to_vec();
    now.extend(t.g.player(P0).library.iter().copied());
    t.g.players[P0.idx()].library = now;
}

fn magus_place(t: &mut TestGame, name: &str) -> ObjectId {
    t.library_top(P0, name)
}

fn magus_run(t: &mut TestGame, card: ObjectId, answers: &dyn Fn(&mut TestGame)) {
    let magus = t.battlefield(P0, "Magus of the Mind");
    with_only_top(t, |t| {
        add_mana(t, P0, ManaType::U, 1);
        t.activate(P0, magus, 0, &[]).expect("activate Magus of the Mind");
        t.resolve_all();
    });
    assert_eq!(t.zone(card), Zone::Exile);
    answers(t);
    let c = t.g.current(card);
    let _ = t.cast(P0, c).method(CastMethod::Free).try_go();
    t.resolve_all();
    t.clear_answers();
}

fn augury_place(t: &mut TestGame, name: &str) -> ObjectId {
    t.library_top(P0, name)
}

fn augury_run(t: &mut TestGame, card: ObjectId, answers: &dyn Fn(&mut TestGame)) {
    with_only_top(t, |t| {
        add_mana(t, P0, ManaType::U, 8);
        let aug = t.hand(P0, "Aminatou's Augury");
        t.cast(P0, aug).go();
        t.resolve_all();
    });
    assert_eq!(t.zone(card), Zone::Exile);
    answers(t);
    let c = t.g.current(card);
    let _ = t.cast(P0, c).method(CastMethod::Free).try_go();
    t.resolve_all();
    t.clear_answers();
}

#[test]
fn magus_and_augury_free_casts_pay_additional_costs_only() {
    cr!("118.9", "118.9a", "118.9b", "118.8a");
    ruling!(
        "Magus of the Mind",
        "If you cast a card “without paying its mana cost,” you can’t pay any alternative costs. You can, however, pay additional costs. If the card has any mandatory additional costs, such as that of Tormenting Voice, those must be paid to cast the card."
    );
    ruling!(
        "Aminatou's Augury",
        "If you cast a card \"without paying its mana cost,\" you can't choose to cast it for any alternative costs, such as evoke costs. You can, however, pay additional costs. If the card has any mandatory additional costs, such as that of Tormenting Voice, those must be paid to cast the card."
    );
    supported("Magus of the Mind");
    supported("Aminatou's Augury");
    check_free_cast_costs(&FreeCaster {
        instants_only: false,
        place: magus_place,
        run: magus_run,
    });
    check_free_cast_costs(&FreeCaster {
        instants_only: false,
        place: augury_place,
        run: augury_run,
    });
    // No evoke: Mulldrifter cast from Aminatou's Augury is a creature that stays.
    let mut t = TestGame::new(2);
    let m = augury_place(&mut t, "Mulldrifter");
    augury_run(&mut t, m, &|_| {});
    assert_eq!(t.named_on_battlefield("Mulldrifter").len(), 1, "{}", t.dump_log());
}

/// A game where P0's library is three Forests and P0 controls Magus of the Mind.
fn magus_game() -> (TestGame, Vec<ObjectId>, ObjectId) {
    let mut t = TestGame::new(2);
    empty_library(&mut t, P0);
    let forests: Vec<ObjectId> = (0..3).map(|_| t.library_top(P0, "Forest")).collect();
    let magus = t.battlefield(P0, "Magus of the Mind");
    (t, forests, magus)
}

fn exiled_of(t: &TestGame, ids: &[ObjectId]) -> Vec<ObjectId> {
    ids.iter()
        .copied()
        .filter(|f| t.zone(*f) == Zone::Exile)
        .collect()
}

#[test]
fn magus_of_the_mind_x_counts_spells_cast_this_turn() {
    cr!("608.2h", "700.4");
    ruling!(
        "Magus of the Mind",
        "If no spells have been cast in the turn you activate Magus of the Mind’s ability, X is one."
    );
    let (mut t, forests, magus) = magus_game();
    add_mana(&mut t, P0, ManaType::U, 1);
    t.activate(P0, magus, 0, &[]).expect("activate");
    t.resolve_all();
    assert_eq!(exiled_of(&t, &forests).len(), 1);
    // An opponent's spell counts too: X is two.
    let (mut t, forests, magus) = magus_game();
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(P0).go();
    t.resolve_all();
    add_mana(&mut t, P0, ManaType::U, 1);
    t.activate(P0, magus, 0, &[]).expect("activate");
    t.resolve_all();
    assert_eq!(exiled_of(&t, &forests).len(), 2);
}

#[test]
fn magus_of_the_mind_timing_and_cards_left_in_exile() {
    cr!("307.1", "302.1", "611.2a");
    ruling!(
        "Magus of the Mind",
        "You must follow the normal timing permissions and restrictions for cards you cast this way."
    );
    ruling!(
        "Magus of the Mind",
        "Any of the cards you don’t play will remain in exile."
    );
    let mut t = TestGame::new(2);
    let bears = t.library_top(P0, "Grizzly Bears");
    let magus = t.battlefield(P0, "Magus of the Mind");
    add_mana(&mut t, P0, ManaType::U, 1);
    t.set_step(P0, Step::BeginningOfCombat);
    with_only_top(&mut t, |t| {
        t.activate(P0, magus, 0, &[]).expect("activate");
        t.resolve_all();
    });
    assert_eq!(t.zone(bears), Zone::Exile);
    // A creature can't be cast during combat, nor with something on the stack.
    assert!(!can_cast(&mut t, P0, bears, CastMethod::Free));
    t.g.combat = None;
    t.set_step(P0, Step::PostcombatMain);
    assert!(can_cast(&mut t, P0, bears, CastMethod::Free));
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    assert!(!can_cast(&mut t, P0, bears, CastMethod::Free));
    t.resolve_all();
    // Not cast this turn: it stays in exile, and can't be cast next turn.
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.zone(bears), Zone::Exile);
    t.advance_to(P0, Step::PrecombatMain);
    assert_eq!(t.zone(bears), Zone::Exile);
    assert!(!can_cast(&mut t, P0, bears, CastMethod::Free));
    assert!(!can_cast(&mut t, P0, bears, CastMethod::Normal));
}

/// Casts Aminatou's Augury with the top eight cards of P0's library being `top`
/// (the first on top), P0 putting `land` (an index into `top`) onto the battlefield.
fn augury(t: &mut TestGame, top: &[&str], land: Option<usize>) -> Vec<ObjectId> {
    let ids = stack_library(t, P0, top);
    if let Some(i) = land {
        t.answer_choose_moved(P0, &[Entity::Object(ids[i])]);
    } else {
        t.answer_choose(P0, &[]);
    }
    add_mana(t, P0, ManaType::U, 8);
    let aug = t.hand(P0, "Aminatou's Augury");
    t.cast(P0, aug).go();
    t.resolve();
    ids
}

const EIGHT: [&str; 8] = [
    "Forest",
    "Dryad Arbor",
    "Darksteel Citadel",
    "Grizzly Bears",
    "Shock",
    "Divination",
    "Pacifism",
    "Island",
];

#[test]
fn aminatous_augury_lands_only_on_resolution() {
    cr!("305.4", "305.2", "608.2c");
    ruling!(
        "Aminatou's Augury",
        "If you wish to put a land card onto the battlefield, you do so while Aminatou's Augury is resolving, before you're allowed to cast any of the other cards. You may do this even if you've already played a land this turn."
    );
    ruling!(
        "Aminatou's Augury",
        "If you exile a land with another card type, you can't play it later in the turn. You can only put it onto the battlefield at the time Aminatou's Augury instructs you to."
    );
    // A land already played this turn: the Forest is still put onto the battlefield.
    let mut t = TestGame::new(2);
    let mountain = t.hand(P0, "Mountain");
    t.play_land(P0, mountain).expect("play a land");
    let ids = augury(&mut t, &EIGHT, Some(0));
    assert!(t.on_battlefield(ids[0]), "{}", t.dump_log());
    // Before casting: the land's already there when the first spell may be cast.
    // (Nothing else came onto the battlefield.)
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 0);

    // Dryad Arbor (a land creature) and Darksteel Citadel (an artifact land) can't be
    // played or cast later, even with a land play available; Dryad Arbor can be put onto
    // the battlefield as Aminatou's Augury resolves.
    let mut t = TestGame::new(2);
    let ids = augury(&mut t, &EIGHT, None);
    for i in [1, 2, 7] {
        assert_eq!(t.zone(ids[i]), Zone::Exile);
        assert!(!can_play_land(&mut t, P0, ids[i]), "{}", EIGHT[i]);
        assert!(!can_cast(&mut t, P0, ids[i], CastMethod::Free), "{}", EIGHT[i]);
    }
    // A nonland card may be cast.
    assert!(can_cast(&mut t, P0, ids[3], CastMethod::Free));
    let mut t = TestGame::new(2);
    let ids = augury(&mut t, &EIGHT, Some(1));
    assert!(t.on_battlefield(ids[1]), "{}", t.dump_log());
}

#[test]
fn act_on_impulse_lands_need_a_land_play() {
    cr!("305.2", "305.1", "116.2a");
    ruling!(
        "Act on Impulse",
        "Under normal circumstances, you can play a land card exiled with Act on Impulse only if you haven't played a land yet that turn."
    );
    supported("Act on Impulse");
    for played_first in [false, true] {
        let mut t = TestGame::new(2);
        let ids = stack_library(&mut t, P0, &["Forest", "Shock", "Island"]);
        if played_first {
            let m = t.hand(P0, "Mountain");
            t.play_land(P0, m).expect("play a land");
        }
        give_mana_for(&mut t, P0, "Act on Impulse");
        let spell = t.hand(P0, "Act on Impulse");
        t.cast(P0, spell).go();
        t.resolve();
        assert_eq!(t.zone(ids[0]), Zone::Exile);
        assert_eq!(can_play_land(&mut t, P0, ids[0]), !played_first);
        if !played_first {
            let f = t.g.current(ids[0]);
            t.play_land(P0, f).expect("play the exiled Forest");
            assert!(!can_play_land(&mut t, P0, ids[2]), "one land play only");
        }
    }
}

#[test]
fn irascible_wolverine_plays_follow_normal_costs_and_timing() {
    cr!("305.1", "305.2", "601.2f", "116.2a");
    ruling!(
        "Irascible Wolverine",
        "You pay all costs and follow all normal timing rules for a card played from exile with Irascible Wolverine’s first ability. For example, if the exiled card is a land card, you may play it only during your main phase while the stack is empty."
    );
    supported("Irascible Wolverine");
    // A land: only in a main phase, with the stack empty.
    let mut t = TestGame::new(2);
    let forest = t.library_top(P0, "Forest");
    t.enter(P0, "Irascible Wolverine");
    t.resolve_all();
    assert_eq!(t.zone(forest), Zone::Exile);
    assert!(can_play_land(&mut t, P0, forest));
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    assert!(!can_play_land(&mut t, P0, forest));
    t.resolve_all();
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!can_play_land(&mut t, P0, forest));
    t.g.combat = None;
    t.set_step(P0, Step::PostcombatMain);
    assert!(can_play_land(&mut t, P0, forest));
    // A creature: its mana cost is paid.
    let mut t = TestGame::new(2);
    let bears = t.library_top(P0, "Grizzly Bears");
    t.enter(P0, "Irascible Wolverine");
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Exile);
    assert!(!can_cast(&mut t, P0, bears, CastMethod::Normal));
    t.lands(P0, "Forest", 2);
    assert!(can_cast(&mut t, P0, bears, CastMethod::Normal));
    let b = t.g.current(bears);
    t.cast(P0, b).go();
    assert_eq!(tapped_lands(&t, P0), 2);
}

#[test]
fn gunner_conscript_losing_an_attachment_can_make_damage_lethal() {
    cr!("120.6", "704.5g", "613.4c");
    ruling!(
        "Gunner Conscript",
        "Because damage remains marked on a creature until the damage is removed as the turn ends, nonlethal damage dealt to Gunner Conscript may become lethal if an Aura or Equipment attached to it leaves the battlefield or becomes unattached from Gunner Conscript during that turn."
    );
    supported("Gunner Conscript");
    // An Aura leaving the battlefield.
    let mut t = TestGame::new(2);
    let gc = t.battlefield(P0, "Gunner Conscript");
    let base = t.pt(gc);
    let aura = crate::r_s06_common::attach_new(&mut t, P0, "Pacifism", gc);
    assert_eq!(t.pt(gc), (base.0 + 1, base.1 + 1));
    let src = t.battlefield(P1, "Grizzly Bears");
    crate::r_s06_common::damage(&mut t, src, base.1, gc);
    t.settle();
    assert!(t.on_battlefield(gc));
    crate::r_s02_common::destroy(&mut t, aura);
    t.settle();
    assert!(!t.on_battlefield(gc));
    // Equipment becoming unattached.
    let mut t = TestGame::new(2);
    let gc = t.battlefield(P0, "Gunner Conscript");
    let sword = crate::r_s06_common::attach_new(&mut t, P0, "Short Sword", gc);
    let src = t.battlefield(P1, "Grizzly Bears");
    crate::r_s06_common::damage(&mut t, src, base.1, gc);
    t.settle();
    assert!(t.on_battlefield(gc));
    t.g.unattach(sword);
    t.g.flush_events();
    t.settle();
    assert!(!t.on_battlefield(gc), "{}", t.dump_log());
}
