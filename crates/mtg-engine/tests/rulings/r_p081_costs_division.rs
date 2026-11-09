//! Rulings batch P081 — total costs and mana value of spells that punish attackers
//! (CR 601.2f, 202.3), damage divided among attacking or blocking creatures as the spell
//! is cast (CR 601.2d, 608.2b), and "counter unless its controller pays" activated several
//! times (CR 602.2, 608.2).

use crate::r_p081_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// The number of tapped lands `p` controls.
fn tapped_lands(t: &TestGame, p: PlayerId) -> usize {
    crate::r_s01_common::tapped_lands(t, p)
}

#[test]
fn run_behind_reduction_applies_after_increases_and_leaves_the_mana_value() {
    cr!("601.2f", "202.3", "118.7");
    ruling!(
        "Run Behind",
        "To determine the total cost of a spell, start with the mana cost or alternative cost you're paying, add any cost increases, then apply any cost reductions (such as that of Run Behind's first ability). The mana value of the spell is determined by only its mana cost, no matter what the total cost to cast that spell was."
    );
    supported("Run Behind");
    supported("Thalia, Guardian of Thraben");
    // Targeting an attacker: {2}{U} of the {3}{U}; the mana value stays 4.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    blocked_by_p0(&mut t, &[a], &[]);
    t.lands(P0, "Island", 3);
    let card = t.hand(P0, "Run Behind");
    let spell = t.cast(P0, card).target(a).go();
    assert_eq!(tapped_lands(&t, P0), 3);
    assert_eq!(crate::r_p001_common::mv(&t, spell), 4);
    // Not an attacker: four lands are needed.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Island", 3);
    let card = t.hand(P0, "Run Behind");
    assert!(t.cast(P0, card).target(a).try_go().is_err());
    // Thalia's increase ({1} more) then Run Behind's reduction: {3}{U} for an attacker.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Thalia, Guardian of Thraben");
    let a = t.battlefield(P1, "Grizzly Bears");
    blocked_by_p0(&mut t, &[a], &[]);
    t.lands(P0, "Island", 4);
    let card = t.hand(P0, "Run Behind");
    let spell = t.cast(P0, card).target(a).go();
    assert_eq!(tapped_lands(&t, P0), 4);
    assert_eq!(crate::r_p001_common::mv(&t, spell), 4);
}

#[test]
fn witchstalker_frenzy_costs_at_least_r_and_keeps_mana_value_4() {
    cr!("601.2f", "202.3", "118.7a");
    ruling!(
        "Witchstalker Frenzy",
        "Witchstalker Frenzy's first ability can't reduce its cost to less than {R}."
    );
    ruling!(
        "Witchstalker Frenzy",
        "Witchstalker Frenzy's first ability doesn't change its mana cost or mana value, only the total cost you pay. Specifically, the mana value of Witchstalker Frenzy is always 4."
    );
    supported("Witchstalker Frenzy");
    let mut t = TestGame::new(2);
    let at: Vec<ObjectId> = (0..5).map(|_| t.battlefield(P1, "Grizzly Bears")).collect();
    blocked_by_p0(&mut t, &at, &[]);
    t.lands(P0, "Mountain", 1);
    let card = t.hand(P0, "Witchstalker Frenzy");
    let spell = t.cast(P0, card).target(at[0]).go();
    assert_eq!(tapped_lands(&t, P0), 1);
    assert_eq!(crate::r_p001_common::mv(&t, spell), 4);
    t.resolve_all();
    assert!(died(&t, at[0]));
    // One attacker: {2}{R}.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    blocked_by_p0(&mut t, &[a], &[]);
    t.lands(P0, "Mountain", 3);
    let card = t.hand(P0, "Witchstalker Frenzy");
    let spell = t.cast(P0, card).target(a).go();
    assert_eq!(tapped_lands(&t, P0), 3);
    assert_eq!(crate::r_p001_common::mv(&t, spell), 4);
}

/// The `Divide` decisions asked of `p` since `from`: (total, recipients, min_each).
fn divisions_since(t: &TestGame, p: PlayerId, from: usize) -> Vec<(u32, usize, u32)> {
    t.asked()[from..]
        .iter()
        .filter_map(|(q, d)| match d {
            Decision::Divide {
                total,
                recipients,
                min_each,
                ..
            } if *q == p => Some((*total, recipients.len(), *min_each)),
            _ => None,
        })
        .collect()
}

#[test]
fn damage_is_divided_among_the_targets_as_the_spell_is_cast_at_least_1_each() {
    cr!("601.2d", "608.2b", "601.2c");
    ruling!(
        "Deft Dismissal",
        "You choose how many targets Deft Dismissal has and how the damage is divided as you cast the spell. Each target must receive at least 1 damage."
    );
    ruling!(
        "Roil's Retribution",
        "You choose how many targets Roil’s Retribution has and how the damage is divided as you cast the spell. Each target must receive at least 1 damage."
    );
    ruling!(
        "Arrow Volley Trap",
        "You divide the damage as you cast Arrow Volley Trap, not as it resolves. Each target must be assigned at least 1 damage."
    );
    ruling!(
        "Hail of Arrows",
        "You choose how the damage will be divided among the target creatures at the time you cast Hail of Arrows. Each target must be dealt at least 1 damage. If any of those creatures becomes an illegal target before Hail of Arrows resolves, the division of damage among the remaining creatures doesn’t change."
    );
    for (name, total, x) in [
        ("Deft Dismissal", 3u32, None),
        ("Roil's Retribution", 5, None),
        ("Arrow Volley Trap", 5, None),
        ("Hail of Arrows", 3, Some(3i64)),
    ] {
        supported(name);
        let mut t = TestGame::new(2);
        let a = t.battlefield(P1, "Colossal Dreadmaw");
        let b = t.battlefield(P1, "Colossal Dreadmaw");
        blocked_by_p0(&mut t, &[a, b], &[]);
        lands_for_cost(&mut t, P0, name);
        if let Some(x) = x {
            t.lands(P0, "Wastes", x as usize);
        }
        let card = t.hand(P0, name);
        let from = mark(&t);
        t.answer_targets(P0, &[obj(a), obj(b)]);
        t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![total as i64 - 1, 1]));
        let mut c = t.cast(P0, card);
        if let Some(x) = x {
            c = c.x(x);
        }
        c.go();
        // Divided while casting: before anything resolves.
        assert_eq!(divisions_since(&t, P0, from), vec![(total, 2, 1)], "{name}");
        // One target leaves in response: the other's share doesn't change.
        cast_new(&mut t, P1, "Unsummon", &[obj(a)]);
        t.resolve();
        t.resolve_all();
        assert_eq!(marked(&t, b), 1, "{name}");
        assert!(t.in_hand(P1, "Colossal Dreadmaw"), "{name}");
    }
    // A division giving a target 0 is not accepted: the engine divides legally instead.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Colossal Dreadmaw");
    let b = t.battlefield(P1, "Colossal Dreadmaw");
    blocked_by_p0(&mut t, &[a, b], &[]);
    lands_for_cost(&mut t, P0, "Deft Dismissal");
    let card = t.hand(P0, "Deft Dismissal");
    t.answer_targets(P0, &[obj(a), obj(b)]);
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![3, 0]));
    t.cast(P0, card).go();
    t.resolve_all();
    assert!(marked(&t, a) >= 1 && marked(&t, b) >= 1);
    assert_eq!(marked(&t, a) + marked(&t, b), 3);
}

#[test]
fn fire_at_will_may_target_attackers_and_blockers_alike() {
    cr!("115.1a", "601.2d");
    ruling!(
        "Fire at Will",
        "You may target both attacking creatures and blocking creatures as you cast Fire at Will."
    );
    supported("Fire at Will");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Hill Giant");
    let b = t.battlefield(P0, "Hill Giant");
    let bystander = t.battlefield(P0, "Grizzly Bears");
    blocked_by_p0(&mut t, &[a], &[(b, a)]);
    let cands = spell_targets(&mut t, P0, "Fire at Will");
    assert!(cands.contains(&obj(a)) && cands.contains(&obj(b)));
    assert!(!cands.contains(&obj(bystander)));
    lands_for_cost(&mut t, P0, "Fire at Will");
    let card = t.hand(P0, "Fire at Will");
    t.answer_targets(P0, &[obj(a), obj(b)]);
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![2, 1]));
    t.cast(P0, card).go();
    t.resolve_all();
    assert_eq!((marked(&t, a), marked(&t, b)), (2, 1));
}

#[test]
fn thrull_wizard_may_be_activated_again_after_the_payment() {
    cr!("602.2", "608.2", "117.3b");
    ruling!(
        "Thrull Wizard",
        "You can use this ability multiple times on one spell to force your opponent to use more than one mana to prevent the spell from being countered. You can even use this ability, let them pay, then use this ability repeatedly."
    );
    supported("Thrull Wizard");
    let mut t = TestGame::new(2);
    let wiz = t.battlefield(P0, "Thrull Wizard");
    t.lands(P0, "Swamp", 4);
    t.set_step(P1, Step::PrecombatMain);
    // P1 casts Black Knight ({B}{B}) with one Swamp to spare.
    t.lands(P1, "Swamp", 3);
    let knight = t.hand(P1, "Black Knight");
    let spell = t.cast(P1, knight).go();
    assert_eq!(tapped_lands(&t, P1), 2);
    // First activation: P1 pays {B}.
    t.activate(P0, wiz, 0, &[obj(spell)]).unwrap();
    t.answer_yes(P1, true);
    t.resolve();
    assert_eq!(tapped_lands(&t, P1), 3);
    assert_eq!(t.stack_len(), 1, "the Knight is still on the stack");
    // Second activation: P1 can't pay any more.
    t.activate(P0, wiz, 0, &[obj(spell)]).unwrap();
    t.answer_yes(P1, true);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Black Knight"));
    assert_eq!(t.named_on_battlefield("Black Knight").len(), 0);
}
