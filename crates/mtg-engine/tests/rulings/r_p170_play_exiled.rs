//! Rulings batch P170 — playing cards exiled with a permission (CR 601.2, 305.1, 305.2,
//! 307.1): the normal timing rules and costs apply, a land needs an available land play,
//! and a cast spell doesn't depend on the permission's condition anymore (CR 601.2).

use crate::r_p170_common::*;
use crate::r_s01_common::stack_library;
use crate::r_s02_common::{can_cast, can_play_land};
use crate::r_s21_common::go_to;
use mtg_engine::decision::Answer;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Whether `p` could play (cast or, for a land, play) `card` now.
fn can_play(t: &mut TestGame, p: PlayerId, card: ObjectId) -> bool {
    can_cast(t, p, card, CastMethod::Normal) || can_play_land(t, p, card)
}

/// P0 holds priority with an Opt on the stack.
fn opt_on_stack(t: &mut TestGame) {
    cast_card(t, P0, "Opt", &[]);
    assert!(!t.g.stack.is_empty());
}

/// Abbot of Keral Keep enters with `card` on top of P0's library; returns the exiled card.
fn abbot_exile(t: &mut TestGame, card: &str) -> ObjectId {
    supported("Abbot of Keral Keep");
    let c = t.library_top(P0, card);
    t.enter(P0, "Abbot of Keral Keep");
    t.resolve_all();
    let c = t.g.current(c);
    assert_eq!(t.zone(c), Zone::Exile);
    c
}

#[test]
fn abbot_of_keral_keep_exiles_face_up_and_normal_timing_applies() {
    cr!("601.2", "601.3", "302.1", "305.2", "406.3");
    ruling!(
        "Abbot of Keral Keep",
        "The card exiled by Abbot of Keral Keep's ability is exiled face up."
    );
    ruling!(
        "Abbot of Keral Keep",
        "Playing the card exiled with Abbot of Keral Keep's ability follows the normal rules for playing that card. You must pay its costs, and you must follow all applicable timing rules. For example, if the card is a creature card, you can cast that card by paying its mana cost only during your main phase while the stack is empty."
    );
    ruling!(
        "Abbot of Keral Keep",
        "Unless an effect allows you to play additional lands that turn, you can play a land card exiled with Abbot of Keral Keep's ability only if you haven't played a land yet that turn."
    );
    // A creature card: face up; castable only with its mana, in a main phase with an
    // empty stack.
    let mut t = TestGame::new(2);
    let bears = abbot_exile(&mut t, "Grizzly Bears");
    assert!(!t.obj_now(bears).face_down);
    assert!(!can_play(&mut t, P0, bears), "no mana to pay its cost");
    t.lands(P0, "Forest", 2);
    assert!(can_play(&mut t, P0, bears));
    opt_on_stack(&mut t);
    assert!(!can_play(&mut t, P0, bears), "the stack isn't empty");
    t.resolve_all();
    assert!(can_play(&mut t, P0, bears));
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!can_play(&mut t, P0, bears), "not a main phase");
    t.set_step(P0, Step::PostcombatMain);
    assert!(t.cast(P0, bears).try_go().is_ok());
    t.resolve_all();
    assert!(t.on_battlefield(t.g.current(bears)));
    // A land card: only with a land play left.
    let mut t = TestGame::new(2);
    let forest = abbot_exile(&mut t, "Forest");
    assert!(can_play(&mut t, P0, forest));
    let mountain = t.hand(P0, "Mountain");
    t.play_land(P0, mountain).unwrap();
    assert!(!can_play(&mut t, P0, forest), "the land play was used");
    // The Forest first, then no other land.
    let mut t = TestGame::new(2);
    let forest = abbot_exile(&mut t, "Forest");
    t.play_land(P0, forest).unwrap();
    assert!(t.on_battlefield(t.g.current(forest)));
    let mountain = t.hand(P0, "Mountain");
    assert!(!can_play(&mut t, P0, mountain));
}

/// P0's Party Thrasher trigger at the beginning of P0's precombat main phase: P0
/// discards a Forest and exiles the top two cards of the library (`card` on top),
/// choosing `card`. Returns `card` in exile.
fn thrasher_exile(t: &mut TestGame, card: &str) -> ObjectId {
    supported("Party Thrasher");
    t.battlefield(P0, "Party Thrasher");
    let forest = t.hand(P0, "Forest");
    let c = stack_library(t, P0, &[card, "Grizzly Bears"])[0];
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(forest)]);
    t.answer_choose_moved(P0, &[obj(c)]);
    t.set_step(P0, Step::Draw);
    go_to(t, Step::PrecombatMain);
    t.resolve_all();
    let c = t.g.current(c);
    assert_eq!(t.zone(c), Zone::Exile);
    c
}

#[test]
fn party_thrasher_lands_follow_the_normal_timing_rules() {
    cr!("305.1", "505.6b");
    ruling!(
        "Party Thrasher",
        "You pay all costs and follow all normal timing rules for cards played using Party Thrasher's second ability. For example, if the chosen exiled card is a land card, you may play it only during your main phase while the stack is empty."
    );
    let mut t = TestGame::new(2);
    let land = thrasher_exile(&mut t, "Mountain");
    assert!(can_play(&mut t, P0, land));
    opt_on_stack(&mut t);
    assert!(!can_play(&mut t, P0, land), "the stack isn't empty");
    t.resolve_all();
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!can_play(&mut t, P0, land), "not a main phase");
    t.set_step(P0, Step::PostcombatMain);
    t.play_land(P0, land).unwrap();
    assert!(t.on_battlefield(t.g.current(land)));
    // A creature card costs its mana: Hill Giant with no lands can't be cast.
    let mut t = TestGame::new(2);
    let bears = thrasher_exile(&mut t, "Hill Giant");
    assert!(!can_play(&mut t, P0, bears));
}

#[test]
fn party_thrasher_convoke_with_x_chooses_x_first() {
    cr!("702.51a", "601.2b", "601.2f", "601.2g", "107.3a");
    ruling!(
        "Party Thrasher",
        "When using convoke to cast a spell with {X} in its cost, first choose the value for X. That choice, plus any cost increases or decreases, will determine the spell's total cost. Then you can tap creatures you control to help pay that cost. For example, if you cast Wrath of the Skies (a spell with mana cost {X}{W}{W}) from exile, choosing 3 as the value of X, the total cost is {3}{W}{W}. If you tap a white creature and a red creature, you'll have to pay {2}{W}."
    );
    // (lands, whether it can be cast): a white creature and the red Party Thrasher pay
    // {W} and {1} of {3}{W}{W}; {2}{W} remains.
    for (lands, ok) in [
        (&["Plains", "Wastes", "Wastes"][..], true),
        (&["Plains", "Wastes"][..], false),
    ] {
        let mut t = TestGame::new(2);
        let wrath = thrasher_exile(&mut t, "Wrath of the Skies");
        let thrasher = t.named_on_battlefield("Party Thrasher")[0];
        let lions = t.battlefield(P0, "Savannah Lions");
        let ls: Vec<ObjectId> = lands.iter().map(|l| t.lands(P0, l, 1)[0]).collect();
        t.answer(
            P0,
            DecisionKind::Entities,
            Answer::Entities(vec![obj(lions), obj(thrasher)]),
        );
        let r = t.cast(P0, wrath).x(3).try_go();
        assert_eq!(r.is_ok(), ok, "{lands:?}");
        if ok {
            assert!(t.obj_now(lions).tapped && t.obj_now(thrasher).tapped);
            assert!(ls.iter().all(|l| t.obj_now(*l).tapped));
            let spell = r.unwrap();
            assert_eq!(crate::r_s25_common::x_of(&t, spell), Some(3));
            t.resolve_all();
            assert_eq!(crate::r_p015_common::energy(&t, P0), 3);
        }
    }
}

#[test]
fn rassilons_exiled_land_waits_for_the_main_phase() {
    cr!("305.1");
    ruling!(
        "Rassilon, the War President",
        "You must pay all costs and follow all normal timing rules for playing a card exiled this way. For example, you may play a land this way only during your main phase while the stack is empty."
    );
    supported("Rassilon, the War President");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rassilon, the War President");
    let forest = t.library_top(P0, "Forest");
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.life(P0), 18);
    let forest = t.g.current(forest);
    assert_eq!(t.zone(forest), Zone::Exile);
    assert!(!can_play(&mut t, P0, forest), "upkeep");
    go_to(&mut t, Step::PrecombatMain);
    assert!(can_play(&mut t, P0, forest));
    opt_on_stack(&mut t);
    assert!(!can_play(&mut t, P0, forest), "the stack isn't empty");
    t.resolve_all();
    t.play_land(P0, forest).unwrap();
    assert!(t.on_battlefield(t.g.current(forest)));
}

/// P0 casts Flameshape with Lightning Bolt and a Forest on top of the library; returns
/// them in exile (Bolt, Forest).
fn flameshape(t: &mut TestGame) -> (ObjectId, ObjectId) {
    supported("Gandalf, Goblins' Bane // Flameshape");
    let top = stack_library(t, P0, &["Lightning Bolt", "Forest"]);
    t.lands(P0, "Mountain", 2);
    let card = t.hand(P0, "Gandalf, Goblins' Bane // Flameshape");
    t.cast(P0, card).method(CastMethod::Half(1)).go();
    t.resolve_all();
    let (bolt, forest) = (t.g.current(top[0]), t.g.current(top[1]));
    assert_eq!(t.zone(bolt), Zone::Exile);
    assert!(t.obj_now(bolt).face_down);
    assert_eq!(t.zone(forest), Zone::Exile);
    (bolt, forest)
}

#[test]
fn flameshape_needs_a_wizard_to_cast_but_not_to_resolve() {
    cr!("601.2", "601.3", "608.2");
    ruling!(
        "Gandalf, Goblins' Bane // Flameshape",
        "You must control a Wizard to cast spells exiled by Flameshape, but losing control of your last Wizard while a spell is on the stack will not prevent the spells from resolving."
    );
    let mut t = TestGame::new(2);
    let (bolt, _) = flameshape(&mut t);
    t.lands(P0, "Mountain", 1);
    assert!(!can_play(&mut t, P0, bolt), "no Wizard");
    let wizard = t.battlefield(P0, "Mysidian Elder");
    assert!(can_play(&mut t, P0, bolt));
    t.cast(P0, bolt).target(pl(P1)).go();
    // The Wizard leaves in response: the Bolt still resolves.
    crate::r_s02_common::destroy(&mut t, wizard);
    assert!(!t.on_battlefield(wizard));
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn flameshape_lands_need_a_main_phase_an_empty_stack_and_a_land_play() {
    cr!("305.1", "305.2");
    ruling!(
        "Gandalf, Goblins' Bane // Flameshape",
        "You pay all costs and follow all timing rules for cards exiled with Flameshape. For example, if an exiled card is a land card, you may play it only during your main phase while the stack is empty and only if you have an available land play remaining."
    );
    let mut t = TestGame::new(2);
    let (_, forest) = flameshape(&mut t);
    t.battlefield(P0, "Mysidian Elder");
    assert!(can_play(&mut t, P0, forest));
    opt_on_stack(&mut t);
    assert!(!can_play(&mut t, P0, forest), "the stack isn't empty");
    t.resolve_all();
    t.set_step(P0, Step::End);
    assert!(!can_play(&mut t, P0, forest), "not a main phase");
    t.set_step(P0, Step::PostcombatMain);
    assert!(can_play(&mut t, P0, forest));
    let mountain = t.hand(P0, "Mountain");
    t.play_land(P0, mountain).unwrap();
    assert!(!can_play(&mut t, P0, forest), "no land play left");
}
