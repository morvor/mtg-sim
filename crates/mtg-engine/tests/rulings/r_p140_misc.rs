//! Rulings batch P140 — Teferi, Timeless Voyager's phasing (control changes, a player
//! leaving the game), "whenever you cast a historic spell" (Surtr, Fiery Jötun and Glóin,
//! Dwarf Emissary) and Stridehangar Automaton with an effect changing under whose control
//! tokens enter.

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
    cr!("702.26a", "702.26f");
    ruling!(
        "Teferi, Timeless Voyager",
        "If an opponent gains control of your creature and it phases out and the duration of the control-change effect expires before it phases in, that permanent phases in under your control as that opponent's appropriate untap step begins."
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
    assert!(phased_out(&t, bears), "can't phase in until the end of P0's next turn");
    // P0's next turn: it phased out under P1's control, so not in P0's untap step.
    t.advance_to(P0, Step::Upkeep);
    assert!(phased_out(&t, bears));
    // P1's next untap step: it phases in, under P0's control.
    t.advance_to(P1, Step::Upkeep);
    assert!(!phased_out(&t, bears));
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
        // Positive control: casting an artifact spell triggers.
        t.answer_targets(P0, &[Entity::Player(P1)]);
        cast_lots(&mut t, P0, "Ornithopter", &[]);
        t.settle();
        assert_eq!(t.stack_len(), 2, "{name}: an artifact spell");
    }
}

#[test]
fn stridehangar_applies_before_an_effect_changing_who_controls_entering_tokens() {
    cr!("616.1", "614.12");
    ruling!(
        "Stridehangar Automaton",
        "If an effect changes under whose control a token would enter the battlefield, that effect applies after this creature’s effect is able to be applied."
    );
    supported("Gather Specimens");
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
