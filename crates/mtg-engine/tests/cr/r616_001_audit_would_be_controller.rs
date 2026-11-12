//! CR 616.1 audit (review): for a permanent entering the battlefield, the player who
//! chooses among the replacement effects is the one it would enter under the control of,
//! as the effects applied so far have it: after a control-changing effect (616.1b), the new
//! would-be controller makes the next choice, not the controller of the spell.

use crate::r614_audit_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const P3: PlayerId = PlayerId(3);

/// In a four-player game on P2's turn, P0, P1 and P3 have each cast Gather Specimens. P2
/// casts Grizzly Bears; `answers` are the players' answers to choices among replacement
/// effects, in the order given.
fn bears_under_three_gather_specimens(answers: &[(PlayerId, usize)]) -> (TestGame, ObjectId) {
    supported("Gather Specimens");
    let mut t = TestGame::new(4);
    t.set_step(P2, Step::PrecombatMain);
    for p in [P0, P1, P3] {
        cast_resolve(&mut t, p, "Gather Specimens", &[]);
        t.set_step(P2, Step::PrecombatMain);
    }
    for (p, i) in answers {
        t.answer(*p, DecisionKind::Replacement, Answer::Index(*i));
    }
    let bears = cast_resolve(&mut t, P2, "Grizzly Bears", &[]);
    let bears = t.g.current(bears);
    (t, bears)
}

#[test]
fn the_new_would_be_controller_chooses_among_the_remaining_gather_specimens() {
    cr!("616.1", "616.1b", "616.1f");
    interp!("entering-would-be-controller-chooses");
    ruling!(
        "Gather Specimens",
        "Then the new would-be controller of the creature repeats this process among the remaining Gather Specimens"
    );
    // The options name the effects in the order they were created: P0's, P1's, P3's.
    let (probe, _) = bears_under_three_gather_specimens(&[]);
    let choices = replacement_choices(&probe);
    assert_eq!(choices.len(), 2, "{choices:?}");
    assert_eq!(choices[0].0, P2, "the controller of the creature spell chooses first");
    assert_eq!(choices[0].1.len(), 3);
    // P2 applies P0's Gather Specimens first: P0, the new would-be controller, chooses
    // which of P1's and P3's applies next (the last one applied wins).
    for (second, winner) in [(0usize, P3), (1usize, P1)] {
        let (mut t, bears) = bears_under_three_gather_specimens(&[(P2, 0), (P0, second)]);
        let choices = replacement_choices(&t);
        assert_eq!(
            choices.iter().map(|c| c.0).collect::<Vec<_>>(),
            vec![P2, P0],
            "{choices:?}"
        );
        assert_eq!(choices[1].1.len(), 2);
        assert_eq!(controller(&mut t, bears), winner, "P0 picked option {second}");
    }
}

/// P0 controls Essence of the Wild and Infinite Reflection (on P0's Grizzly Bears) and
/// casts Gather Specimens on P1's turn. P1 casts Hill Giant; `answers` are the players'
/// answers to choices among replacement effects.
fn giant_under_specimens_and_two_copy_effects(
    answers: &[(PlayerId, usize)],
) -> (TestGame, ObjectId) {
    supported("Infinite Reflection");
    supported("Essence of the Wild");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Essence of the Wild");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let reflection = t.battlefield(P0, "Infinite Reflection");
    t.g.attach(reflection, obj(bears));
    t.set_step(P1, Step::PrecombatMain);
    cast_resolve(&mut t, P0, "Gather Specimens", &[]);
    t.set_step(P1, Step::PrecombatMain);
    for (p, i) in answers {
        t.answer(*p, DecisionKind::Replacement, Answer::Index(*i));
    }
    let giant = cast_resolve(&mut t, P1, "Hill Giant", &[]);
    let giant = t.g.current(giant);
    (t, giant)
}

#[test]
fn after_gather_specimens_its_caster_orders_the_copy_effects() {
    cr!("616.1", "616.1b", "616.1c", "616.1f");
    interp!("entering-would-be-controller-chooses");
    ruling!(
        "Infinite Reflection",
        "Replacement effects that modify how a permanent enters are now applied in the following order: control-changing effects (such as Gather Specimens), then copy effects (such as the abilities of Infinite Reflection and Clone), then all other effects."
    );
    let (probe, _) = giant_under_specimens_and_two_copy_effects(&[]);
    let choices = replacement_choices(&probe);
    assert_eq!(choices.len(), 1, "{choices:?}");
    // The Giant would enter under P0's control: P0 orders P0's two copy effects, not P1,
    // who controlled the spell.
    assert_eq!(choices[0].0, P0);
    let essence = option_index(&choices[0].1, "Essence of the Wild");
    let reflection = option_index(&choices[0].1, "Infinite Reflection");
    // P1's answers, if P1 were asked, would pick the other order.
    for (first, last_copied) in [
        (essence, "Grizzly Bears"),
        (reflection, "Essence of the Wild"),
    ] {
        let other = if first == essence { reflection } else { essence };
        let (mut t, giant) =
            giant_under_specimens_and_two_copy_effects(&[(P0, first), (P1, other)]);
        assert_eq!(controller(&mut t, giant), P0);
        assert_eq!(t.obj_now(giant).chars.name.as_str(), last_copied);
        assert!(replacement_choices(&t).iter().all(|c| c.0 == P0));
    }
}

#[test]
fn a_card_reanimated_from_an_opponents_graveyard_has_its_entry_ordered_by_the_reanimator() {
    cr!("616.1", "616.1c", "614.12");
    interp!("entering-would-be-controller-chooses");
    supported("Reanimate");
    supported("Infinite Reflection");
    supported("Essence of the Wild");
    // P0 reanimates P1's Hill Giant under P0's control, with P0's two copy effects. The
    // card has no controller in P1's graveyard; P0, under whose control it would enter,
    // orders them, and the last one applied wins.
    let run = |answers: &[(PlayerId, usize)]| {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Essence of the Wild");
        let bears = t.battlefield(P0, "Grizzly Bears");
        let reflection = t.battlefield(P0, "Infinite Reflection");
        t.g.attach(reflection, obj(bears));
        let giant = t.graveyard(P1, "Hill Giant");
        for (p, i) in answers {
            t.answer(*p, DecisionKind::Replacement, Answer::Index(*i));
        }
        cast_resolve(&mut t, P0, "Reanimate", &[obj(giant)]);
        let giant = t.g.current(giant);
        (t, giant)
    };
    let (probe, _) = run(&[]);
    let choices = replacement_choices(&probe);
    assert_eq!(choices.len(), 1, "{choices:?}");
    assert_eq!(choices[0].0, P0, "the player it enters under, not its owner");
    let essence = option_index(&choices[0].1, "Essence of the Wild");
    let reflection = option_index(&choices[0].1, "Infinite Reflection");
    for (first, other, copied) in [
        (essence, reflection, "Grizzly Bears"),
        (reflection, essence, "Essence of the Wild"),
    ] {
        let (mut t, giant) = run(&[(P0, first), (P1, other)]);
        assert_eq!(controller(&mut t, giant), P0);
        assert_eq!(t.obj_now(giant).chars.name.as_str(), copied);
    }
}
