//! Rulings batch P117 — cards newly compiled for this batch: "Choose a color. Add one mana
//! of that color for each [count]" mana abilities (Selvala, Eager Trailblazer; Astral
//! Cornucopia; CR 605.1a, 106.1b) and "if it has the least power or is tied for least
//! power among creatures on the battlefield" (Wretched Banquet; CR 608.2c).

use crate::r_p117_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::*;

/// Taps `src` for mana, choosing green; returns the green mana in P0's pool.
fn tap_for_green(t: &mut TestGame, src: ObjectId) -> u32 {
    // The color options are in WUBRG order.
    t.answer(P0, DecisionKind::Option, Answer::Index(4));
    t.activate(P0, src, 0, &[]).unwrap();
    crate::r_s24_common::pool(t, P0, ManaType::G)
}

#[test]
fn selvala_counts_different_powers_among_creatures_you_control() {
    cr!("605.1a", "608.2h");
    ruling!(
        "Selvala, Eager Trailblazer",
        "Selvala’s last ability looks at the creatures you control as it resolves to determine how much mana of the chosen color to add. For example, if you control Selvala (a 4/5 creature), a 1/1 creature, and a 2/1 creature, there are three different powers among creatures you control."
    );
    ruling!(
        "Selvala, Eager Trailblazer",
        "Selvala’s last ability doesn’t care what color or colors the creatures you control are. It cares only about the power of those creatures."
    );
    supported("Selvala, Eager Trailblazer");
    let mut t = TestGame::new(2);
    let selvala = t.battlefield(P0, "Selvala, Eager Trailblazer");
    t.battlefield(P0, "Llanowar Elves");
    t.battlefield(P0, "Elite Vanguard");
    // Another 2-power creature and an opponent's 3/3 add no new power.
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Hill Giant");
    assert_eq!(tap_for_green(&mut t, selvala), 3);
}

#[test]
fn selvala_mana_ability_doesnt_use_the_stack() {
    cr!("605.1a", "605.3a");
    ruling!(
        "Selvala, Eager Trailblazer",
        "Selvala’s last ability is a mana ability. It doesn’t use the stack and can’t be responded to."
    );
    let mut t = TestGame::new(2);
    let selvala = t.battlefield(P0, "Selvala, Eager Trailblazer");
    assert_eq!(tap_for_green(&mut t, selvala), 1);
    assert_eq!(stack(&t), 0);
}

#[test]
fn astral_cornucopia_adds_one_mana_of_a_chosen_color_per_charge_counter() {
    cr!("605.1a", "106.1b");
    supported("Astral Cornucopia");
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "Astral Cornucopia");
    put_counters(&mut t, c, "charge", 2);
    assert_eq!(tap_for_green(&mut t, c), 2);
    assert_eq!(stack(&t), 0);
}

#[test]
fn wretched_banquet_checks_power_only_as_it_resolves() {
    cr!("608.2c", "115.1");
    ruling!(
        "Wretched Banquet",
        "Wretched Banquet may target any creature. The creature’s power is checked only as Wretched Banquet resolves. If another creature has less power than the targeted creature at that time, Wretched Banquet does nothing."
    );
    supported("Wretched Banquet");
    // (pump the target in response, exile the Elves in response, destroyed)
    for (pump, exile_elves, destroyed) in [
        (false, false, false),
        (false, true, true),
        (true, true, false),
    ] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        let elves = t.battlefield(P0, "Llanowar Elves");
        t.battlefield(P1, "Hill Giant");
        // The Bears aren't the least powerful as Wretched Banquet is cast.
        cast_new(&mut t, P0, "Wretched Banquet", &[obj(bears)]);
        if exile_elves {
            exile(&mut t, elves);
        }
        if pump {
            giant_growth(&mut t, bears);
        }
        t.resolve_all();
        assert_eq!(
            t.in_graveyard(P1, "Grizzly Bears"),
            destroyed,
            "pump {pump}, exile {exile_elves}"
        );
    }
    // Tied for the least power: destroyed.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P0, "Grizzly Bears");
    cast_new(&mut t, P0, "Wretched Banquet", &[obj(bears)]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}
