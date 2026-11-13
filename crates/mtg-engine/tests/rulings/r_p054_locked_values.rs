//! Rulings batch P054 — values and affected sets of pump effects are locked in as the
//! spell or ability resolves (CR 608.2h, 611.2c): counts of creatures, cards in a
//! graveyard, devotion and basic land types, cards revealed on resolution, and the set
//! of creatures a "creatures you control get ..." effect affects.

use crate::r_p017_common::animate;
use crate::r_p054_common::*;
use crate::r_s06_common::give_control;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// The "X is the number of creatures you control" pumps: the count includes the target
/// itself and is locked in on resolution.
#[test]
fn creature_count_pumps_count_the_target_and_lock_in_on_resolution() {
    cr!("608.2h", "611.2c");
    ruling!(
        "Get a Leg Up",
        "The number of creatures you control is checked only once, as Get a Leg Up resolves."
    );
    ruling!(
        "Get a Leg Up",
        "If you target a creature you control with Get a Leg Up, remember to count that creature."
    );
    ruling!(
        "Might of the Masses",
        "If you target a creature you control with Might of the Masses, remember to count that creature when determining the amount of the bonus."
    );
    ruling!(
        "Might of the Masses",
        "The size of the bonus is determined as Might of the Masses resolves; it won't change if the number of creatures you control changes later in the turn."
    );
    ruling!(
        "Chorus of Might",
        "The number of creatures you control is counted only when Chorus of Might resolves."
    );
    ruling!(
        "Frontline Rush",
        "The value of X is calculated only once, as Frontline Rush resolves."
    );
    ruling!(
        "Appeal // Authority",
        "The value of X is determined only as Appeal begins to resolve. It won’t change later in the turn if the number of creatures you control changes."
    );
    ruling!(
        "Pollen-Shield Hare // Hare Raising",
        "The value of X is determined only once, as Hare Raising resolves. If the number of creatures you control changes after that time, it won't change the bonus granted."
    );
    let cases: &[(&str, CastMethod, Option<&[usize]>)] = &[
        ("Get a Leg Up", CastMethod::Normal, None),
        ("Might of the Masses", CastMethod::Normal, None),
        ("Chorus of Might", CastMethod::Normal, None),
        ("Frontline Rush", CastMethod::Normal, Some(&[1])),
        ("Appeal // Authority", CastMethod::Half(0), None),
        (
            "Pollen-Shield Hare // Hare Raising",
            CastMethod::Half(1),
            None,
        ),
    ];
    for (name, method, modes) in cases {
        let mut t = TestGame::new(2);
        let bear = t.battlefield(P0, "Grizzly Bears");
        t.battlefield(P0, "Hill Giant");
        // An opponent's creature isn't counted.
        t.battlefield(P1, "Hill Giant");
        cast_how(
            &mut t,
            P0,
            name,
            method.clone(),
            &[&[o(bear)]],
            None,
            *modes,
        );
        // A creature entering in response is counted.
        t.battlefield(P0, "Grizzly Bears");
        t.resolve_all();
        assert_eq!(t.pt(bear), (5, 5), "{name}: the bear and two others");
        t.battlefield(P0, "Grizzly Bears");
        t.battlefield(P0, "Grizzly Bears");
        t.g.recompute();
        assert_eq!(t.pt(bear), (5, 5), "{name}: locked in");
    }
}

#[test]
fn growth_cycle_counts_growth_cycles_as_it_begins_to_resolve() {
    cr!("608.2h");
    ruling!(
        "Growth Cycle",
        "The size of the bonus is determined as Growth Cycle begins to resolve; it won’t change later in the turn if the number of Growth Cycles in your graveyard changes."
    );
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P0, "Grizzly Bears");
    t.graveyard(P0, "Growth Cycle");
    cast_at(&mut t, P0, "Growth Cycle", bear);
    t.resolve_all();
    // +3/+3 and +2/+2 for the one in the graveyard; the resolving one went to the
    // graveyard only after it resolved.
    assert!(t.in_graveyard(P0, "Growth Cycle"));
    assert_eq!(t.graveyard_size(P0), 2);
    assert_eq!(t.pt(bear), (7, 7));
    t.graveyard(P0, "Growth Cycle");
    t.g.recompute();
    assert_eq!(t.pt(bear), (7, 7));
}

#[test]
fn aspect_of_hydra_devotion_is_locked_in_on_resolution() {
    cr!("608.2h", "700.5");
    ruling!(
        "Aspect of Hydra",
        "The value of X is calculated as Aspect of Hydra resolves. The bonus won’t change later in the turn, even if your devotion to green does."
    );
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P0, "Grizzly Bears");
    cast_at(&mut t, P0, "Aspect of Hydra", bear);
    // Devotion rises in response: counted.
    t.battlefield(P0, "Llanowar Elves");
    t.resolve_all();
    assert_eq!(t.pt(bear), (4, 4));
    t.battlefield(P0, "Llanowar Elves");
    t.g.recompute();
    assert_eq!(t.pt(bear), (4, 4));
}

/// "Creatures you control get ..." affects only the creatures you control as it resolves
/// (CR 611.2c): not creatures that enter, that come under your control, or that become
/// creatures later in the turn.
#[test]
fn creatures_you_control_pumps_lock_in_the_affected_set() {
    cr!("611.2c");
    ruling!(
        "Vitalizing Wind",
        "Only affects creatures you control when the spell resolves."
    );
    ruling!(
        "Preposterous Proportions",
        "Preposterous Proportions affects only creatures you control at the time it resolves. It won't affect creatures that come under your control later in the turn."
    );
    ruling!(
        "Engineered Might",
        "The set of creatures affected by Engineered Might's second mode is determined as the spell resolves. Creatures you begin to control later in the turn and noncreature permanents that become creatures later in the turn won't get +2/+2 or gain vigilance."
    );
    ruling!(
        "Nissa, Nature's Artisan",
        "The set of creatures affected by Nissa's last ability is determined as the ability resolves. Creatures you begin to control later in the turn and noncreature permanents that become creatures later in the turn won't get +5/+5 or gain trample."
    );
    // (card, bonus, how it's used)
    for (name, bonus) in [
        ("Vitalizing Wind", 7),
        ("Preposterous Proportions", 10),
        ("Engineered Might", 2),
        ("Nissa, Nature's Artisan", 5),
    ] {
        let mut t = TestGame::new(2);
        let bear = t.battlefield(P0, "Grizzly Bears");
        let theirs = t.battlefield(P1, "Grizzly Bears");
        let land = t.battlefield(P0, "Forest");
        if name == "Nissa, Nature's Artisan" {
            let nissa = t.battlefield(P0, name);
            t.g.add_counters(o(nissa), "loyalty", 20, None);
            t.activate(P0, nissa, 2, &[]).expect("-12");
        } else if name == "Engineered Might" {
            cast_how(&mut t, P0, name, CastMethod::Normal, &[], None, Some(&[1]));
        } else {
            cast_how(&mut t, P0, name, CastMethod::Normal, &[], None, None);
        }
        t.resolve_all();
        assert_eq!(t.pt(bear), (2 + bonus, 2 + bonus), "{name}");
        // Later: a creature enters, one comes under P0's control, a land becomes one.
        let late = t.battlefield(P0, "Grizzly Bears");
        give_control(&mut t, theirs, P0);
        animate(&mut t, land, 1);
        t.g.recompute();
        assert_eq!(t.pt(late), (2, 2), "{name}: entered later");
        assert_eq!(t.pt(theirs), (2, 2), "{name}: controlled later");
        assert_eq!(t.pt(land), (1, 1), "{name}: became a creature later");
        assert_eq!(t.pt(bear), (2 + bonus, 2 + bonus), "{name}");
    }
}

#[test]
fn briar_hydra_counts_basic_land_types_as_its_trigger_resolves() {
    cr!("608.2h", "510.3a");
    ruling!(
        "Briar Hydra",
        "The value of X is determined as Briar Hydra’s last ability resolves."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Forest");
    let hydra = t.battlefield(P0, "Briar Hydra");
    let bear = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[o(bear)]);
    crate::r_s01_common::attack_with(&mut t, &[(hydra, pl(P1))]);
    t.advance_to(P0, Step::CombatDamage);
    t.settle();
    assert_eq!(t.life(P1), 14);
    assert_eq!(t.stack_len(), 1, "the Domain trigger");
    // A land with two new basic land types arrives before it resolves.
    t.battlefield(P0, "Plains");
    t.battlefield(P0, "Island");
    t.resolve_all();
    assert_eq!(t.counters(bear, "+1/+1"), 3);
}

/// Scent of Ivy / Scent of Cinder: the cards are revealed, and X set, on resolution: a
/// green (red) card that came into the hand after casting can be revealed.
#[test]
fn scent_cards_reveal_and_set_x_on_resolution() {
    cr!("608.2h", "701.20a");
    ruling!(
        "Scent of Ivy",
        "You reveal cards, and thereby set the value of X, on resolution."
    );
    ruling!(
        "Scent of Cinder",
        "You reveal cards, and thereby set the value of X, on resolution."
    );
    // Scent of Ivy
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P0, "Grizzly Bears");
    cast_at(&mut t, P0, "Scent of Ivy", bear);
    let late = t.hand(P0, "Llanowar Elves");
    let late2 = t.hand(P0, "Giant Growth");
    t.answer_choose(P0, &[o(late), o(late2)]);
    t.resolve_all();
    assert_eq!(t.pt(bear), (4, 4));
    // Scent of Cinder
    let mut t = TestGame::new(2);
    cast_how(
        &mut t,
        P0,
        "Scent of Cinder",
        CastMethod::Normal,
        &[&[pl(P1)]],
        None,
        None,
    );
    let late = t.hand(P0, "Shock");
    t.answer_choose(P0, &[o(late)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
}

#[test]
fn sages_reverie_counts_auras_as_its_etb_resolves_including_itself() {
    cr!("608.2h", "603.2");
    ruling!(
        "Sage's Reverie",
        "Count the number of Auras you control attached to creatures as the enters-the-battlefield ability resolves to determine how many cards to draw. This will include Sage's Reverie as long as it's still on the battlefield at that time."
    );
    for leaves in [false, true] {
        let mut t = TestGame::new(2);
        for _ in 0..5 {
            t.library_top(P0, "Island");
        }
        let bear = t.battlefield(P0, "Grizzly Bears");
        crate::r_s06_common::attach_new(&mut t, P0, "Holy Strength", bear);
        // An opponent's Aura on P0's creature doesn't count.
        crate::r_s06_common::attach_new(&mut t, P1, "Holy Strength", bear);
        let rev = cast_at(&mut t, P0, "Sage's Reverie", bear);
        t.resolve(); // the Aura resolves; its ETB trigger goes on the stack
        assert_eq!(t.stack_len(), 1);
        let _ = rev;
        if leaves {
            let r =
                t.g.permanents()
                    .find(|p| p.chars.name == "Sage's Reverie")
                    .map(|p| p.id)
                    .unwrap();
            move_to(&mut t, r, Zone::Exile);
        }
        let before = t.hand_size(P0);
        t.resolve_all();
        assert_eq!(
            t.hand_size(P0) - before,
            if leaves { 1 } else { 2 },
            "Reverie left: {leaves}"
        );
    }
}

#[test]
fn team_avatar_damage_is_counted_as_its_ability_resolves() {
    cr!("608.2h", "602.1");
    ruling!(
        "Team Avatar",
        "Similarly, the damage dealt by Team Avatar's activated ability is calculated during the resolution of the ability."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Craw Wurm");
    let avatar = t.hand(P0, "Team Avatar");
    pool(&mut t, P0);
    t.activate(P0, avatar, 0, &[o(giant)])
        .expect("activate from hand");
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Grizzly Bears");
    t.resolve_all();
    assert_eq!(crate::r_s07_common::damage_on(&t, giant), 3);
}
