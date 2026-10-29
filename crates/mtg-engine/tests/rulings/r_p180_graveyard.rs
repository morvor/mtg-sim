//! Rulings batch P180 — abilities that work from the graveyard or exile: Resurgent
//! Belief, Arc Blade, Kozilek's Return, Gate Colossus, Quakebringer and Thunderblade
//! Charge.

use crate::r_p180_common::*;
use crate::r_s11_common::triggered_from;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Casts the suspended Resurgent Belief (one time counter left) at P0's next upkeep.
fn resurgent_belief(t: &mut TestGame) {
    supported("Resurgent Belief");
    let belief = t.exile(P0, "Resurgent Belief");
    t.g.add_counters(obj(belief), mtg_engine::types::counters::TIME, 1, None);
    t.g.recompute();
    t.answer_yes(P0, true);
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Resurgent Belief"), "it was cast");
}

#[test]
fn resurgent_belief_returns_every_enchantment_attaching_auras_to_legal_objects() {
    cr!("303.4a", "303.4f", "702.11b", "702.16c");
    ruling!(
        "Resurgent Belief",
        "All of the enchantment cards in your graveyard that can return to the battlefield will."
    );
    ruling!(
        "Resurgent Belief",
        "If an effect puts an Aura onto the battlefield under your control, you choose what it enters attached to."
    );
    // The only creatures are the opponent's: a hexproof one and one with protection from
    // enchantments (created first, so a default first choice would be it).
    let mut t = TestGame::new(2);
    let first_wing = t.battlefield(P1, "Azorius First-Wing");
    let scout = t.battlefield(P1, "Gladecover Scout");
    let anthem = t.graveyard(P0, "Glorious Anthem");
    let strength = t.graveyard(P0, "Holy Strength");
    let from = t.asked().len();
    resurgent_belief(&mut t);
    let anthem = t.g.current(anthem);
    let strength = t.g.current(strength);
    assert!(t.on_battlefield(anthem));
    assert!(t.on_battlefield(strength), "the Aura can't stay behind");
    assert_eq!(
        t.obj_now(strength).attached_to,
        Some(obj(scout)),
        "attached to the hexproof creature"
    );
    assert_eq!(t.g.obj(strength).controller, P0);
    // The creature with protection from enchantments was never a choice.
    for (_, d) in &t.asked()[from..] {
        if let Decision::ChooseEntities { candidates, .. } = d {
            assert!(!candidates.contains(&obj(first_wing)));
        }
    }
    assert_eq!(t.pt(scout), (2, 3));
}

#[test]
fn arc_blade_with_an_illegal_target_isnt_exiled_with_time_counters() {
    cr!("608.2b");
    ruling!(
        "Arc Blade",
        "If the chosen target is an illegal target by the time Arc Blade tries to resolve, the spell doesn't resolve."
    );
    supported("Arc Blade");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let blade = cast_targeting(&mut t, P0, "Arc Blade", &[obj(bears)]);
    destroy(&mut t, bears);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Arc Blade"));
    assert!(!t.in_exile("Arc Blade"));
    let _ = blade;
    // With a legal target it's exiled with three time counters.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let blade = cast_targeting(&mut t, P0, "Arc Blade", &[obj(bears)]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    let blade = t.g.current(blade);
    assert_eq!(t.zone(blade), Zone::Exile);
    assert_eq!(t.counters(blade, mtg_engine::types::counters::TIME), 3);
}

#[test]
fn kozileks_return_from_the_graveyard_resolves_first_and_isnt_a_spell() {
    cr!("603.2", "113.3c");
    ruling!(
        "Kozilek's Return",
        "The last ability triggers only if Kozilek's Return is in your graveyard as you cast the Eldrazi creature spell."
    );
    ruling!(
        "Kozilek's Return",
        "Exiling Kozilek's Return due to the last ability isn't the same as casting it as a spell."
    );
    supported("Kozilek's Return");
    supported("Ulamog's Crusher");
    let mut t = TestGame::new(2);
    let ret = t.graveyard(P0, "Kozilek's Return");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Craw Wurm");
    let crusher = cast_targeting(&mut t, P0, "Ulamog's Crusher", &[]);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    assert_eq!(triggered_from(&t, ret), 1);
    // "Counter target spell" can only target the Eldrazi spell, not the ability.
    let targets = crate::r_s04_common::spell_targets(&mut t, P1, "Counterspell");
    assert_eq!(targets, vec![obj(crusher)]);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(t.in_exile("Kozilek's Return"));
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(t.in_graveyard(P1, "Craw Wurm"), "5 damage to a 6/4");
    let _ = giant;
    let crusher = t.g.current(crusher);
    assert!(t.on_battlefield(crusher));
    assert_eq!(damage_on(&t, crusher), 0, "the Eldrazi wasn't there yet");
    // Not in the graveyard as the spell is cast: no trigger.
    let mut t = TestGame::new(2);
    let ret = t.hand(P0, "Kozilek's Return");
    cast_targeting(&mut t, P0, "Ulamog's Crusher", &[]);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    assert_eq!(triggered_from(&t, ret), 0);
}

#[test]
fn gate_colossus_stays_blocked_when_the_blockers_power_drops() {
    cr!("509.1b", "509.1h");
    ruling!(
        "Gate Colossus",
        "Once a creature with power 3 or greater has blocked Gate Colossus, changing the power of the blocking creature won't cause Gate Colossus to become unblocked."
    );
    supported("Gate Colossus");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::BeginningOfCombat);
    let colossus = t.battlefield(P0, "Gate Colossus");
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    crate::r_s01_common::attack_with(&mut t, &[(colossus, Entity::Player(P1))]);
    assert!(!can_block(&mut t, bears, colossus), "power 2 can't block");
    assert!(can_block(&mut t, giant, colossus));
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(giant, colossus)]),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    // Disfigure: the Hill Giant becomes 1/1.
    cast_targeting(&mut t, P0, "Disfigure", &[obj(giant)]);
    t.resolve_all();
    assert_eq!(t.pt(giant), (1, 1));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20, "still blocked");
    assert!(t.in_graveyard(P1, "Hill Giant"));
}

#[test]
fn quakebringer_doesnt_undo_earlier_life_gain() {
    cr!("119.7");
    ruling!(
        "Quakebringer",
        "Quakebringer entering the battlefield won’t undo any life gained earlier in the turn."
    );
    supported("Quakebringer");
    let mut t = TestGame::new(2);
    t.g.gain_life(P1, 3);
    t.enter(P0, "Quakebringer");
    t.resolve_all();
    assert_eq!(t.life(P1), 23);
    t.g.gain_life(P1, 3);
    t.settle();
    assert_eq!(t.life(P1), 23, "can't gain life now");
}

#[test]
fn thunderblade_charge_triggers_once_per_damage_step_and_pays_before_casting() {
    cr!("603.2c", "510.2");
    ruling!(
        "Thunderblade Charge",
        "Thunderblade Charge triggers only once per combat damage step per player"
    );
    ruling!(
        "Thunderblade Charge",
        "When the ability triggers, first you choose whether to pay {2}{R}{R}{R}. If you paid, then you choose whether to cast the spell."
    );
    supported("Thunderblade Charge");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::BeginningOfCombat);
    let charge = t.graveyard(P0, "Thunderblade Charge");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    t.lands(P0, "Mountain", 3);
    t.lands(P0, "Wastes", 2);
    // Pay, then decline to cast.
    t.answer_yes(P0, true);
    t.answer_yes(P0, false);
    t.attack(&[(a, Entity::Player(P1)), (b, Entity::Player(P1))], &[]);
    assert_eq!(triggered_from(&t, charge), 1);
    assert_eq!(t.life(P1), 15, "combat damage only");
    assert_eq!(crate::r_s01_common::tapped_lands(&t, P0), 5, "paid");
    assert_eq!(t.zone(charge), Zone::Graveyard(P0), "not cast");
}
