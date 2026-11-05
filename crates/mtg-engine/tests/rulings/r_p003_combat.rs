//! Rulings batch P003 — attacking matters: "whenever you attack" triggers (CR 508.3d) and
//! what's counted as they resolve (CR 608.2h) and fixed from then on;
//! counters from an Equipment's trigger stay (CR 122); combat damage assigned by
//! toughness (CR 510.1a) but fights by power (CR 701.14); "nonattacking, nonblocking"
//! targets (CR 115.1); Magus of the Arena's targets (CR 115.7, 608.2b).

use crate::r_p003_common::*;
use crate::r_s01_common::{supported, with_subtype};
use crate::r_s06_common::attach_new;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// P0 declares the attackers (at P1); the attack triggers are on the stack and P0 has
/// priority in the declare attackers step.
fn declare(t: &mut TestGame, attackers: &[ObjectId]) {
    let atk: Vec<(ObjectId, Entity)> = attackers.iter().map(|a| (*a, pl(P1))).collect();
    t.answer(P0, DecisionKind::Attackers, Answer::Attackers(atk));
    t.advance_to(P0, Step::DeclareAttackers);
}

fn bears(t: &mut TestGame, n: usize) -> Vec<ObjectId> {
    (0..n).map(|_| t.battlefield(P0, "Grizzly Bears")).collect()
}

#[test]
fn razorkin_hordecaller_triggers_whoever_attacks() {
    cr!("508.3d");
    ruling!(
        "Razorkin Hordecaller",
        "Attacking with any creatures will cause Razorkin Hordecaller's last ability to trigger. Razorkin Hordecaller doesn't have to be among them."
    );
    supported("Razorkin Hordecaller");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Razorkin Hordecaller");
    let b = bears(&mut t, 1);
    declare(&mut t, &b);
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Gremlin").len(), 1);
}

#[test]
fn rubblebelt_raiders_and_akroan_hoplite_count_attackers_as_they_resolve() {
    cr!("608.2h");
    ruling!(
        "Rubblebelt Raiders",
        "Count the number of attacking creatures you control when Rubblebelt Raiders’s ability resolves, including Rubblebelt Raiders itself, to determine how many +1/+1 counters to put on it."
    );
    ruling!(
        "Akroan Hoplite",
        "Count the number of attacking creatures you control when the ability resolves to determine the value of X."
    );
    ruling!(
        "Akroan Hoplite",
        "Once the ability resolves, the bonus won't change even if the number of attacking creatures you control does."
    );
    supported("Rubblebelt Raiders");
    supported("Akroan Hoplite");
    // Each attacks with two Bears; one Bears leaves before the trigger resolves, the other
    // after.
    for (name, base) in [("Rubblebelt Raiders", 3), ("Akroan Hoplite", 1)] {
        let mut t = TestGame::new(2);
        let c = t.battlefield(P0, name);
        let b = bears(&mut t, 2);
        declare(&mut t, &[c, b[0], b[1]]);
        kill(&mut t, b[0]);
        t.resolve_all();
        assert_eq!(t.pt(c).0, base + 2, "{name}: itself and one Bears");
        kill(&mut t, b[1]);
        assert_eq!(t.pt(c).0, base + 2, "{name}: fixed");
    }
}

#[test]
fn attack_triggers_dont_care_what_happens_to_the_attackers_afterwards() {
    cr!("508.3d");
    ruling!(
        "Seasoned Consultant",
        "It doesn't matter what happens to the creatures in response. As long as you attacked with at least three creatures, Seasoned Consultant will still get +2/+0 until end of turn when its triggered ability resolves."
    );
    ruling!(
        "Armasaur Guide",
        "It doesn't matter what happens to the creatures you attacked with once Armasaur Guide's last ability has triggered. When that ability resolves, you'll put a +1/+1 counter on the target creature regardless of whether the creatures you attacked with (or Armasaur Guide) are still on the battlefield."
    );
    ruling!(
        "Military Intelligence",
        "Once the ability triggers, it doesn’t matter how many creatures are still attacking when the ability resolves."
    );
    supported("Seasoned Consultant");
    supported("Armasaur Guide");
    supported("Military Intelligence");
    // Seasoned Consultant (1/3) stays home; three Bears attack and die in response.
    let mut t = TestGame::new(2);
    let sc = t.battlefield(P0, "Seasoned Consultant");
    let b = bears(&mut t, 3);
    declare(&mut t, &b);
    for x in &b {
        kill(&mut t, *x);
    }
    t.resolve_all();
    assert_eq!(t.pt(sc), (3, 3));
    // Armasaur Guide and two Bears attack; Hill Giant stays home and is the target; the
    // attackers and the Guide die in response.
    let mut t = TestGame::new(2);
    let guide = t.battlefield(P0, "Armasaur Guide");
    let giant = t.battlefield(P0, "Hill Giant");
    let b = bears(&mut t, 2);
    t.answer_targets(P0, &[obj(giant)]);
    declare(&mut t, &[guide, b[0], b[1]]);
    for x in [guide, b[0], b[1]] {
        kill(&mut t, x);
    }
    t.resolve_all();
    assert_eq!(t.counters(giant, mtg_engine::types::counters::PLUS1), 1);
    // Military Intelligence: two Bears attack and die in response; P0 draws.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Military Intelligence");
    let b = bears(&mut t, 2);
    declare(&mut t, &b);
    let hand = t.hand_size(P0);
    for x in &b {
        kill(&mut t, *x);
    }
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn armory_of_iroas_counters_stay_when_it_moves() {
    cr!("122.1", "702.6a");
    ruling!(
        "Armory of Iroas",
        "If you move Armory of Iroas from one creature to another, any +1/+1 counters on the first creature remain where they are."
    );
    supported("Armory of Iroas");
    let mut t = TestGame::new(2);
    let b = bears(&mut t, 2);
    let armory = attach_new(&mut t, P0, "Armory of Iroas", obj(b[0]));
    declare(&mut t, &[b[0]]);
    t.resolve_all();
    assert_eq!(t.counters(b[0], mtg_engine::types::counters::PLUS1), 1);
    t.advance_to(P0, Step::PostcombatMain);
    activate_resolve(&mut t, P0, armory, "Equip", "2", &[obj(b[1])]);
    assert_eq!(t.obj_now(armory).attached_to, Some(obj(b[1])));
    assert_eq!(t.counters(b[0], mtg_engine::types::counters::PLUS1), 1);
    assert_eq!(t.counters(b[1], mtg_engine::types::counters::PLUS1), 0);
}

#[test]
fn assault_formation_changes_only_combat_damage_not_power() {
    cr!("510.1a", "701.14a");
    ruling!(
        "Assault Formation",
        "Assault Formation’s first ability doesn’t actually change any creature’s power. It changes only the amount of combat damage it assigns. All other rules and effects that check power or toughness use the real values. For example, Hunt the Weak won’t cause a creature to fight with its toughness."
    );
    supported("Assault Formation");
    supported("Hunt the Weak");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Assault Formation");
    // Horned Turtle (1/4) attacks: 4 combat damage. Its power is still 1.
    let turtle = t.battlefield(P0, "Horned Turtle");
    t.attack(&[(turtle, pl(P1))], &[]);
    assert_eq!(t.life(P1), 16);
    assert_eq!(t.pt(turtle), (1, 4));
    // Hunt the Weak: +1/+1 counter, then it fights a Hill Giant (3/3) with power 2.
    t.advance_to(P0, Step::PostcombatMain);
    let giant = t.battlefield(P1, "Hill Giant");
    cast_resolve(
        &mut t,
        P0,
        "Hunt the Weak",
        "3G",
        &[obj(turtle), obj(giant)],
    );
    assert!(t.on_battlefield(giant));
    assert_eq!(dmg(&t, giant), 2);
}

#[test]
fn unlikely_alliance_bonus_stays_when_the_creature_attacks() {
    cr!("611.2c", "115.1");
    ruling!(
        "Unlikely Alliance",
        "The bonus is not lost if the creature later becomes an attacker or blocker."
    );
    supported("Unlikely Alliance");
    let mut t = TestGame::new(2);
    let ua = t.battlefield(P0, "Unlikely Alliance");
    let b = bears(&mut t, 2);
    activate_resolve(&mut t, P0, ua, "nonattacking", "1W", &[obj(b[0])]);
    assert_eq!(t.pt(b[0]), (2, 4));
    declare(&mut t, &[b[0], b[1]]);
    assert_eq!(t.pt(b[0]), (2, 4));
    // An attacking creature isn't a legal target.
    mana(&mut t, P0, "1W");
    t.answer_targets(P0, &[obj(b[1])]);
    let r = crate::r_s06_common::activate_containing(&mut t, P0, ua, "nonattacking");
    assert!(r.is_err(), "no nonattacking creature to target");
    assert_eq!(t.pt(b[1]), (2, 2));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.pt(b[0]), (2, 4));
}

#[test]
fn magus_of_the_arena_with_a_target_gone_taps_the_other_and_deals_no_damage() {
    cr!("608.2b", "701.14b");
    ruling!(
        "Magus of the Arena",
        "If either target leaves the battlefield before the ability resolves, the ability taps the remaining target, but no damage is dealt. If both targets leave the battlefield, the ability doesn't resolve."
    );
    // Which leave: the opponent's creature, P0's, both.
    for gone in [1usize, 0, 2] {
        let mut t = TestGame::new(2);
        let magus = t.battlefield(P0, "Magus of the Arena");
        let mine = t.battlefield(P0, "Hill Giant");
        let theirs = t.battlefield(P1, "Craw Wurm");
        t.answer_targets(P0, &[obj(mine)]);
        t.answer_targets(P1, &[obj(theirs)]);
        activate(&mut t, P0, magus, "fight", "3", &[]);
        let ab = *t.g.stack.last().unwrap();
        if gone != 0 {
            kill(&mut t, theirs);
        }
        if gone != 1 {
            kill(&mut t, mine);
        }
        t.resolve_all();
        assert!(!t.g.stack.contains(&ab));
        let left = if gone == 1 { mine } else { theirs };
        if gone < 2 {
            assert!(t.obj_now(left).tapped, "gone {gone}: the other is tapped");
            assert_eq!(dmg(&t, left), 0, "gone {gone}: no damage");
        }
        assert!(crate::r_p034_common::damage_events(&t).is_empty());
    }
}

#[test]
fn magus_of_the_arena_second_target_changes_only_to_the_chosen_opponents_creature() {
    cr!("115.7a", "115.7b");
    ruling!(
        "Magus of the Arena",
        "If an effect changes the targets of this ability, the second target can be changed to any creature the chosen opponent controls. The choice of opponent can't be changed."
    );
    supported("Spellskite");
    // Spellskite: "{U/P}: Change a target of target spell or ability to Spellskite."
    // Three players; P1's Craw Wurm is the second target. P2's Spellskite can't become
    // it (P2 isn't the chosen opponent); P1's can.
    for owner in [P2, P1] {
        let mut t = TestGame::new(3);
        let magus = t.battlefield(P0, "Magus of the Arena");
        let mine = t.battlefield(P0, "Hill Giant");
        let theirs = t.battlefield(P1, "Craw Wurm");
        let skite = t.battlefield(owner, "Spellskite");
        t.answer_targets(P0, &[obj(mine)]);
        t.answer(P0, DecisionKind::Entities, Answer::Entities(vec![pl(P1)]));
        t.answer_targets(P1, &[obj(theirs)]);
        activate(&mut t, P0, magus, "fight", "3", &[]);
        let ab = *t.g.stack.last().unwrap();
        assert_eq!(
            crate::r_s25_common::targets_of(&t, ab),
            [obj(mine), obj(theirs)]
        );
        mana(&mut t, owner, "U");
        t.answer_targets(owner, &[obj(ab)]);
        activate(&mut t, owner, skite, "Change a target", "", &[]);
        t.resolve();
        let second = if owner == P1 { skite } else { theirs };
        assert_eq!(
            crate::r_s25_common::targets_of(&t, ab),
            [obj(mine), obj(second)]
        );
        t.resolve_all();
        if owner == P1 {
            assert_eq!(dmg(&t, mine), 0, "fought Spellskite (0/4)");
            assert_eq!(dmg(&t, skite), 3);
            assert_eq!(dmg(&t, theirs), 0);
        } else {
            assert_eq!(dmg(&t, theirs), 3);
            assert_eq!(dmg(&t, mine), 0, "the Wurm's 6 killed the Giant");
            assert!(!t.on_battlefield(mine));
            assert_eq!(dmg(&t, skite), 0);
        }
    }
}
