//! Rulings batch P140 — Teferi, Timeless Voyager's phasing (control changes, a player
//! leaving the game), "whenever you cast a historic spell" (Surtr, Fiery Jötun and Glóin,
//! Dwarf Emissary) and Stridehangar Automaton with effects changing under whose control
//! tokens are created or enter.

use crate::r_p140_common::*;
use mtg_engine::ability::{Duration, Effect, PlayerRef, Sel};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// `p` gains control of `id` until end of turn.
fn take_until_eot(t: &mut TestGame, id: ObjectId, p: PlayerId) {
    let mut ctx = mtg_engine::eval::Ctx::new(None, p);
    ctx.targets = vec![vec![Entity::Object(t.g.current(id))]];
    t.g.exec(
        &Effect::GainControl {
            what: Sel::Target(0),
            who: PlayerRef::You,
            duration: Duration::EndOfTurn,
        },
        &mut ctx,
    );
    t.g.recompute();
    t.g.flush_events();
    t.settle();
}

fn phased_out(t: &TestGame, id: ObjectId) -> bool {
    t.obj(t.g.current(id)).phased_out
}

/// P0's Teferi, Timeless Voyager uses its −8 on `target`.
fn teferi_ultimate(t: &mut TestGame, target: PlayerId) {
    supported("Teferi, Timeless Voyager");
    let tef = t.battlefield(P0, "Teferi, Timeless Voyager");
    set_loyalty(t, tef, 8);
    t.activate(P0, tef, 2, &[Entity::Player(target)])
        .expect("−8");
    t.resolve_all();
}

#[test]
fn teferi_a_stolen_creature_phases_in_under_its_owner_in_the_thiefs_untap_step() {
    cr!("702.26a", "702.26f", "702.26n", "800.4a");
    ruling!(
        "Teferi, Timeless Voyager",
        "If an opponent gains control of your creature and it phases out and the duration of the control-change effect expires before it phases in, that permanent phases in under your control as that opponent's appropriate untap step begins."
    );
    ruling!(
        "Teferi, Timeless Voyager",
        "If they leave the game before that untap step, it phases in as the next untap step begins after that turn would have begun."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    take_until_eot(&mut t, bears, P1);
    assert_eq!(t.obj(bears).controller, P1);
    teferi_ultimate(&mut t, P1);
    assert!(phased_out(&t, bears));
    // P1's turn: the control effect has ended; Teferi's effect keeps it phased out.
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.obj(bears).controller, P0);
    assert!(
        phased_out(&t, bears),
        "can't phase in until the end of P0's next turn"
    );
    // P0's next turn: it phased out under P1's control, so not in P0's untap step.
    t.advance_to(P0, Step::Upkeep);
    assert!(phased_out(&t, bears));
    // P1's next untap step: it phases in, under P0's control.
    t.advance_to(P1, Step::Upkeep);
    assert!(!phased_out(&t, bears));
    assert_eq!(t.obj(bears).controller, P0);

    // The opponent leaves the game during P0's next turn, before its appropriate untap
    // step: the creature (P0's, so it stays) phases in as the next untap step after the
    // opponent's turn would have begun begins — P2's.
    let mut t = ffa(3);
    let bears = t.battlefield(P0, "Grizzly Bears");
    take_until_eot(&mut t, bears, P1);
    teferi_ultimate(&mut t, P1);
    assert!(phased_out(&t, bears));
    t.advance_to(P1, Step::Upkeep);
    assert!(phased_out(&t, bears));
    t.advance_to(P2, Step::Upkeep);
    assert!(phased_out(&t, bears));
    t.advance_to(P0, Step::Upkeep);
    assert!(
        phased_out(&t, bears),
        "Teferi's effect lasts until the end of this turn"
    );
    concede(&mut t, P1);
    assert!(t.g.is_live(t.g.current(bears)), "P0's own creature stays");
    assert!(phased_out(&t, bears));
    t.advance_to(P2, Step::Upkeep);
    assert!(!phased_out(&t, bears), "phased in during P2's untap step");
    assert_eq!(t.obj(bears).controller, P0);
}

#[test]
fn teferi_leaving_the_game_lasts_until_your_next_turn_would_have_begun() {
    cr!("800.4m", "702.26a");
    ruling!(
        "Teferi, Timeless Voyager",
        "In a multiplayer game, if you leave the game after Teferi's last ability resolves but before your next turn ends, its effect lasts until your next turn would have begun. It neither expires immediately nor lasts indefinitely."
    );
    let mut t = ffa(3);
    let bears = t.battlefield(P1, "Grizzly Bears");
    teferi_ultimate(&mut t, P1);
    assert!(phased_out(&t, bears));
    concede(&mut t, P0);
    // P1's next turn begins before P0's would have: still held.
    t.advance_to(P1, Step::Upkeep);
    assert!(phased_out(&t, bears), "doesn't expire immediately");
    // P2's turn, then P0's would have begun; P1's next untap step phases it in.
    t.advance_to(P2, Step::Upkeep);
    t.advance_to(P1, Step::Upkeep);
    assert!(!phased_out(&t, bears), "doesn't last indefinitely");
}

/// P0 gives `id` +2/+2 for `duration` (an effect P0 controls).
fn pump_for(t: &mut TestGame, id: ObjectId, duration: Duration) {
    use mtg_engine::ability::{Modification, Value};
    let mut ctx = mtg_engine::eval::Ctx::new(None, P0);
    ctx.targets = vec![vec![Entity::Object(t.g.current(id))]];
    t.g.exec(
        &Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::ModifyPT(Value::c(2), Value::c(2))],
            duration,
        },
        &mut ctx,
    );
    t.g.recompute();
    t.g.flush_events();
    t.settle();
}

#[test]
fn a_departed_players_effects_until_a_point_of_their_next_turn_end_as_it_would_have_begun() {
    cr!("800.4m");
    use mtg_engine::ability::TriggerStep;
    supported("Erhnam Djinn");
    // P0's effects on P1's creatures, each lasting until a point of P0's next turn:
    // "until your next upkeep", "until the end of your next upkeep", "until end of combat
    // on your next turn", "until the end of your next turn".
    let durations = [
        Duration::UntilYourNextStep(TriggerStep::Upkeep),
        Duration::UntilEndOfYourNextStep(TriggerStep::Upkeep),
        Duration::UntilEndOfCombatOnYourNextTurn,
        Duration::UntilEndOfYourNextTurn,
    ];
    let mut t = ffa(3);
    // A real one first: in P0's upkeep, Erhnam Djinn's "target non-Wall creature an
    // opponent controls gains forestwalk until your next upkeep".
    t.battlefield(P0, "Erhnam Djinn");
    let walker = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[obj(walker)]);
    crate::r_p141_common::into_upkeep(&mut t, P0);
    t.resolve_all();
    let forestwalk = |t: &mut TestGame| {
        t.g.recompute();
        t.obj(t.g.current(walker))
            .has_keyword(mtg_engine::keywords::KeywordKind::Landwalk)
    };
    assert!(forestwalk(&mut t), "{}", t.dump_log());
    // Then, in P0's main phase, the others.
    t.set_step(P0, Step::PrecombatMain);
    let pumped: Vec<ObjectId> = durations
        .iter()
        .map(|d| {
            let c = t.battlefield(P1, "Grizzly Bears");
            pump_for(&mut t, c, d.clone());
            c
        })
        .collect();
    for c in &pumped {
        assert_eq!(t.pt(*c), (4, 4));
    }
    concede(&mut t, P0);
    // They neither end immediately nor last indefinitely: through P1's and P2's turns,
    // until P0's next turn would have begun (before P1's next turn).
    for (who, step) in [(P1, Step::Upkeep), (P1, Step::End), (P2, Step::End)] {
        t.advance_to(who, step);
        for c in &pumped {
            assert_eq!(t.pt(*c), (4, 4), "{who} {step:?}");
        }
        assert!(forestwalk(&mut t), "{who} {step:?}");
    }
    t.advance_to(P1, Step::Upkeep);
    for (c, d) in pumped.iter().zip(durations.iter()) {
        assert_eq!(t.pt(*c), (2, 2), "{d:?} ended");
    }
    assert!(!forestwalk(&mut t), "forestwalk ended");
}

#[test]
fn historic_cast_triggers_ignore_lands_played_and_permanents_transforming() {
    cr!("700.6", "116.2a", "701.27a");
    ruling!(
        "Surtr, Fiery Jötun",
        "Lands are never cast, so abilities that trigger \"whenever you cast a historic spell\" won't trigger if you play a legendary land. They also won't trigger if a card on the battlefield transforms into a card with the legendary supertype, the artifact card type, or the Saga subtype."
    );
    ruling!(
        "Glóin, Dwarf Emissary",
        "Lands are never cast, so abilities that trigger \"whenever you cast a historic spell\" won't trigger if you play a legendary land. They also won't trigger if a card on the battlefield transforms into a card with the legendary supertype, the artifact card type, or the Saga subtype."
    );
    for name in ["Surtr, Fiery Jötun", "Glóin, Dwarf Emissary"] {
        supported(name);
        let mut t = TestGame::new(2);
        t.battlefield(P0, name);
        let ruins = t.hand(P0, "Academy Ruins");
        t.play_land(P0, ruins).expect("land play");
        t.settle();
        assert_eq!(t.stack_len(), 0, "{name}: a legendary land played");
        let abbey = t.battlefield(P0, "Westvale Abbey");
        crate::r_s17_common::transform(&mut t, abbey);
        assert_eq!(
            t.obj(t.g.current(abbey)).chars.name.as_str(),
            "Ormendahl, Profane Prince"
        );
        t.settle();
        assert_eq!(t.stack_len(), 0, "{name}: transformed into a legend");
        let heirloom = t.battlefield(P0, "Neglected Heirloom");
        crate::r_s17_common::transform(&mut t, heirloom);
        assert_eq!(
            t.obj(t.g.current(heirloom)).chars.name.as_str(),
            "Ashmouth Blade"
        );
        t.settle();
        assert_eq!(t.stack_len(), 0, "{name}: transformed into an artifact");
        // Positive control: casting an artifact spell triggers.
        t.answer_targets(P0, &[Entity::Player(P1)]);
        cast_lots(&mut t, P0, "Ornithopter", &[]);
        t.settle();
        assert_eq!(t.stack_len(), 2, "{name}: an artifact spell");
    }
}

#[test]
fn stridehangar_and_effects_changing_who_creates_or_controls_the_tokens() {
    cr!("614.1a", "614.12", "111.2");
    ruling!(
        "Stridehangar Automaton",
        "If an effect changes under whose control a token would be created, that effect applies before this creature’s effect applies."
    );
    ruling!(
        "Stridehangar Automaton",
        "If an effect changes under whose control a token would enter the battlefield, that effect applies after this creature’s effect is able to be applied."
    );
    supported("Crafty Cutpurse");
    supported("Gather Specimens");
    // Crafty Cutpurse (P0's) changes who creates P1's Thopter first. With the Automaton on
    // P1's side, the token is no longer one created under P1's control, so no extra
    // Thopter; with the Automaton on P0's side, it now is one created under P0's control.
    for automaton in [P1, P0] {
        let mut t = TestGame::new(2);
        t.set_step(P1, Step::PrecombatMain);
        t.battlefield(automaton, "Stridehangar Automaton");
        t.enter(P0, "Crafty Cutpurse");
        t.resolve_all();
        let patrol = t.graveyard(P1, "Fairgrounds Patrol");
        activate_lots(&mut t, P1, patrol, 0, &[]);
        t.resolve_all();
        assert_eq!(
            tokens_typed(&t, P1, "Thopter").len(),
            0,
            "automaton={automaton}"
        );
        let want = if automaton == P0 { 2 } else { 1 };
        assert_eq!(
            tokens_typed(&t, P0, "Thopter").len(),
            want,
            "automaton={automaton}"
        );
    }
    // Gather Specimens changes under whose control the tokens enter: the Automaton (P1's)
    // has already applied to the tokens being created under P1's control.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    t.battlefield(P1, "Stridehangar Automaton");
    cast_lots(&mut t, P0, "Gather Specimens", &[]);
    t.resolve_all();
    let patrol = t.graveyard(P1, "Fairgrounds Patrol");
    activate_lots(&mut t, P1, patrol, 0, &[]);
    t.resolve_all();
    assert_eq!(tokens_typed(&t, P1, "Thopter").len(), 0);
    assert_eq!(
        tokens_typed(&t, P0, "Thopter").len(),
        2,
        "created under P1's control (plus the Thopter), then entering under P0's"
    );
}
