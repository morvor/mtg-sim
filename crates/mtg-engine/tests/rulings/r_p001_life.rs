//! Rulings batch P001 — life gain replaced by life loss (Plague Drone, Tainted Remedy) and
//! the order of replacement effects applied to one life gain (CR 616.1), and life gain
//! counted as the spell resolves (Riot Control).

use crate::r_p001_common::*;
use crate::r_s01_common::supported;
use mtg_engine::decision::Answer;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::*;

/// P1 gains `n` life, as a resolving effect would.
fn p1_gains(t: &mut TestGame, n: u32) {
    t.g.gain_life(P1, n);
    t.g.flush_events();
    t.settle();
}

#[test]
fn two_life_gain_to_loss_replacements_dont_stack() {
    cr!("614.1a", "616.1f");
    ruling!(
        "Plague Drone",
        "Having more than one Plague Drone on the battlefield doesn't have any noticeable effect on life gain. Once the effect of one Plague Drone applies, there is no life gain for the others to apply to."
    );
    ruling!(
        "Tainted Remedy",
        "Having more than one Tainted Remedy on the battlefield doesn’t have any noticeable effect on life gain. Once the effect of one Tainted Remedy applies, there is no life gain for the others to apply to."
    );
    for name in ["Plague Drone", "Tainted Remedy"] {
        supported(name);
        let mut t = TestGame::new(2);
        t.battlefield(P0, name);
        t.battlefield(P0, name);
        p1_gains(&mut t, 3);
        // 3 life lost once: not 6, and no life gained.
        assert_eq!(t.life(P1), 17, "{name}");
        // The controller's own life gain isn't replaced.
        t.g.gain_life(P0, 2);
        assert_eq!(t.life(P0), 22, "{name}");
    }
}

#[test]
fn the_gaining_player_orders_plague_drone_and_alhammarrets_archive() {
    cr!("616.1", "616.1e", "616.1f");
    ruling!(
        "Plague Drone",
        "If more than one replacement effect tries to apply to a life gain event, the player who would gain life chooses the order in which they apply."
    );
    supported("Plague Drone");
    supported("Alhammarret's Archive");
    let mut seen = vec![];
    for pick in [0, 1] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Plague Drone");
        t.battlefield(P1, "Alhammarret's Archive");
        let from = t.asked().len();
        t.answer(P1, DecisionKind::Replacement, Answer::Index(pick));
        p1_gains(&mut t, 3);
        // P1, the player who would gain life, chose (P0 was asked nothing).
        let choices = replacement_choices_since(&t, P1, from);
        assert_eq!(choices.len(), 1);
        assert!(replacement_choices_since(&t, P0, from).is_empty());
        let doubled_first = choices[0][pick].contains("twice");
        // Archive first: gain 6 becomes lose 6. Drone first: lose 3, and the Archive
        // has no life gain left to double.
        let expected = if doubled_first { 14 } else { 17 };
        assert_eq!(t.life(P1), expected);
        seen.push(doubled_first);
    }
    assert!(seen.contains(&true) && seen.contains(&false));
}

#[test]
fn riot_control_counts_opponents_creatures_as_it_resolves() {
    cr!("608.2h");
    ruling!(
        "Riot Control",
        "Count the number of creatures your opponents control as Riot Control resolves to determine how much life you gain."
    );
    supported("Riot Control");
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Grizzly Bears");
    let doomed = t.battlefield(P1, "Hill Giant");
    let spell = t.hand(P0, "Riot Control");
    crate::r_s04_common::add_mana(&mut t, P0, ManaType::W, 3);
    t.cast(P0, spell).go();
    // In response, one of P1's creatures dies and P2 gets two creatures.
    crate::r_s02_common::destroy(&mut t, doomed);
    t.battlefield(P2, "Grizzly Bears");
    t.battlefield(P2, "Ornithopter");
    t.resolve_all();
    // 1 (P1) + 2 (P2); P0's own Bears don't count.
    assert_eq!(t.life(P0), 23);
}
