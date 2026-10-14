//! Rulings batch S29 — base power and toughness that become equal to another creature's
//! (layer 7b, CR 613.4b): Galion, Elvenking's Butler ("Whenever Galion attacks, choose up
//! to one other target creature you control. Its base power and toughness become equal to
//! Galion's power and toughness until end of turn.") and Tanazir Quandrix ("Whenever
//! Tanazir Quandrix attacks, you may have the base power and toughness of other creatures
//! you control become equal to Tanazir Quandrix's power and toughness until end of
//! turn."). The values are locked in as the ability resolves (CR 608.2h); later layer 7c
//! and 7d effects and counters still apply (CR 613.4).

use crate::r_s01_common::{attack_with, supported};
use crate::r_s25_common::cast_new;
use crate::r_s29_common::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// P0's Grizzly Bears with a +1/+1 counter and Giant Growth (+3/+3).
fn pumped_bears(t: &mut TestGame) -> ObjectId {
    let bears = t.battlefield(P0, "Grizzly Bears");
    put_counters(t, bears, counters::PLUS1, 1);
    cast_and_resolve(t, P0, "Giant Growth", &[Entity::Object(bears)]);
    bears
}

/// P0 attacks with `attacker` (Galion targets `target`), and the attack trigger resolves.
fn attack(t: &mut TestGame, attacker: ObjectId, target: Option<ObjectId>) {
    if let Some(x) = target {
        t.answer_targets(P0, &[Entity::Object(x)]);
    }
    t.answer_yes(P0, true);
    t.set_step(P0, Step::BeginningOfCombat);
    attack_with(t, &[(attacker, Entity::Player(P1))]);
    t.resolve_all();
}

#[test]
fn pumps_and_counters_apply_after_the_new_base_power_and_toughness() {
    cr!("613.4", "613.4b", "613.4c");
    ruling!(
        "Galion, Elvenking's Butler",
        "Any effects that modify a creature's power and/or toughness without setting them to a specific value (i.e. ones that don't affect base power and/or toughness) will apply after its base power and toughness are set, regardless of the order those effects were created. The same is true for counters that modify its power and toughness."
    );
    supported("Galion, Elvenking's Butler");
    supported("Tanazir Quandrix");
    // Galion is a 4/4: the Bears' base becomes 4/4, plus its counter and Giant Growth.
    let mut t = TestGame::new(2);
    let bears = pumped_bears(&mut t);
    let galion = t.battlefield(P0, "Galion, Elvenking's Butler");
    attack(&mut t, galion, Some(bears));
    assert_eq!(t.pt(bears), (4 + 1 + 3, 4 + 1 + 3));
    // Tanazir Quandrix (4/4, flying, trample): the same for each other creature.
    let mut t = TestGame::new(2);
    let bears = pumped_bears(&mut t);
    let giant = t.battlefield(P0, "Hill Giant");
    let tanazir = t.battlefield(P0, "Tanazir Quandrix");
    attack(&mut t, tanazir, None);
    assert_eq!(t.pt(bears), (8, 8));
    assert_eq!(t.pt(giant), (4, 4));
    assert_eq!(t.pt(tanazir), (4, 4));
}

#[test]
fn the_attackers_actual_power_and_toughness_are_locked_in() {
    cr!("608.2h", "613.4b");
    ruling!(
        "Galion, Elvenking's Butler",
        "As Galion's ability resolves, the base power and toughness of the targeted creature are set to Galion's actual power and toughness, not its base power and toughness. If Galion's power or toughness changes later in the turn, the other creature isn't affected."
    );
    ruling!(
        "Tanazir Quandrix",
        "As the last ability resolves, the base power and toughness of other creatures you control are set to Tanazir Quandrix's actual power and toughness, not just its base power and toughness. If Tanazir Quandrix's power or toughness changes later in the turn, the other creatures you control aren't affected."
    );
    // Galion with Giant Growth (7/7): the Bears becomes 7/7; Galion's second Giant Growth
    // doesn't change the Bears.
    let mut t = TestGame::new(2);
    let galion = t.battlefield(P0, "Galion, Elvenking's Butler");
    cast_and_resolve(&mut t, P0, "Giant Growth", &[Entity::Object(galion)]);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack(&mut t, galion, Some(bears));
    assert_eq!(t.pt(bears), (7, 7));
    cast_and_resolve(&mut t, P0, "Giant Growth", &[Entity::Object(galion)]);
    assert_eq!(t.pt(galion), (10, 10));
    assert_eq!(t.pt(bears), (7, 7));
    // Tanazir with a +1/+1 counter (5/5).
    let mut t = TestGame::new(2);
    let tanazir = t.battlefield(P0, "Tanazir Quandrix");
    put_counters(&mut t, tanazir, counters::PLUS1, 1);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack(&mut t, tanazir, None);
    assert_eq!(t.pt(bears), (5, 5));
    put_counters(&mut t, tanazir, counters::PLUS1, 2);
    assert_eq!(t.pt(bears), (5, 5));
}

#[test]
fn it_overwrites_earlier_setting_effects_and_later_ones_overwrite_it() {
    cr!("613.4b", "613.7");
    ruling!(
        "Galion, Elvenking's Butler",
        "Galion's ability overwrites all previous effects that set a creature's power and toughness to specific values. Other effects that set its power or toughness to specific values that start to apply after the ability resolves will overwrite this effect."
    );
    ruling!(
        "Tanazir Quandrix",
        "Tanazir Quandrix's last ability overwrites all previous effects that set a creature's power and toughness to specific values. Other effects that set its power or toughness to specific values that start to apply after the ability resolves will overwrite this effect."
    );
    // Mind Transfer Protocol first ("base power and toughness 4/5"), then Galion (4/4),
    // then Startling Development ("base power and toughness 4/4" — a Serpent): use a
    // Galion pumped to 7/7 so each value differs.
    for attacker in ["Galion, Elvenking's Butler", "Tanazir Quandrix"] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        cast_and_resolve(
            &mut t,
            P0,
            "Mind Transfer Protocol",
            &[Entity::Object(bears)],
        );
        assert_eq!(t.pt(bears), (4, 5));
        let a = t.battlefield(P0, attacker);
        cast_and_resolve(&mut t, P0, "Giant Growth", &[Entity::Object(a)]);
        let target = (attacker == "Galion, Elvenking's Butler").then_some(bears);
        attack(&mut t, a, target);
        assert_eq!(t.pt(bears), (7, 7), "{attacker}");
        cast_new(&mut t, P0, "Startling Development", &[Entity::Object(bears)]);
        t.resolve_all();
        assert_eq!(t.pt(bears), (4, 4), "{attacker}");
    }
}
