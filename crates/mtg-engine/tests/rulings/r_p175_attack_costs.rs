//! Rulings batch P175 — costs to attack (Propaganda, Windborn Muse: CR 508.1g–h) and
//! "can't attack you" (Rite of the Raging Storm's Lightning Ragers).

use crate::r_p175_common::*;
use crate::r_s09_common::{declare, legal_attack, to_combat};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// The lands `p` controls that are tapped.
fn tapped_lands(t: &TestGame, lands: &[ObjectId]) -> usize {
    lands.iter().filter(|l| tapped(t, **l)).count()
}

#[test]
fn windborn_muse_doesnt_tax_attacks_on_planeswalkers() {
    cr!("508.1c", "508.1h", "506.2");
    ruling!(
        "Windborn Muse",
        "Creatures may still attack planeswalkers you control without their controller paying a cost."
    );
    supported("Windborn Muse");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Windborn Muse");
    let pw = t.battlefield(P1, "Ajani Goldmane");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let lands = t.lands(P0, "Plains", 2);
    to_combat(&mut t, P0);
    let attacks = declare(&mut t, P0, &[(bears, obj(pw))]);
    assert_eq!(attacks, vec![(bears, obj(pw))]);
    assert_eq!(tapped_lands(&t, &lands), 0, "nothing paid");
    // Attacking P1 costs {2}.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Windborn Muse");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let lands = t.lands(P0, "Plains", 2);
    t.answer_yes(P0, true);
    let attacks = declare(&mut t, P0, &[(bears, pl(P1))]);
    assert_eq!(attacks, vec![(bears, pl(P1))]);
    assert_eq!(tapped_lands(&t, &lands), 2, "paid {{2}}");
}

#[test]
fn two_propagandas_cost_four_per_attacker() {
    cr!("508.1h", "508.1j");
    ruling!(
        "Propaganda",
        "If there is more than one Propaganda on the battlefield, the cost is cumulative."
    );
    supported("Propaganda");
    for n in [3usize, 4] {
        let mut t = TestGame::new(2);
        t.battlefield(P1, "Propaganda");
        t.battlefield(P1, "Propaganda");
        let bears = t.battlefield(P0, "Grizzly Bears");
        let lands = t.lands(P0, "Plains", n);
        to_combat(&mut t, P0);
        t.answer_yes(P0, true);
        let attacks = declare(&mut t, P0, &[(bears, pl(P1))]);
        assert_eq!(attacks.len(), usize::from(n == 4), "{n} lands");
        assert_eq!(tapped_lands(&t, &lands), if n == 4 { 4 } else { 0 });
    }
}

#[test]
fn propaganda_is_paid_while_declaring_and_for_each_attack() {
    cr!("508.1h", "508.1i", "508.1j", "500.8");
    ruling!(
        "Propaganda",
        "The payment is made during the declare attackers step at the same time you are declaring the attacker."
    );
    ruling!(
        "Propaganda",
        "If there are multiple attacks in a turn, then you have to pay for each attack."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Propaganda");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let lands = t.lands(P0, "Plains", 4);
    // Beginning of combat: nothing paid yet.
    to_combat(&mut t, P0);
    assert_eq!(tapped_lands(&t, &lands), 0);
    t.answer_yes(P0, true);
    let attacks = declare(&mut t, P0, &[(bears, pl(P1))]);
    assert_eq!(attacks.len(), 1);
    assert_eq!(tapped_lands(&t, &lands), 2);
    // The lands were tapped for mana in the declare attackers step, as the Bears were
    // being declared (before the declaration was complete).
    use mtg_engine::events::Event;
    let ev = &t.g.turn_events;
    let began = ev
        .iter()
        .position(|e| {
            matches!(
                e,
                Event::StepBegan {
                    step: Step::DeclareAttackers,
                    ..
                }
            )
        })
        .unwrap();
    let declared = ev
        .iter()
        .position(|e| matches!(e, Event::AttackersDeclared { .. }))
        .unwrap();
    let paid: Vec<usize> = ev
        .iter()
        .enumerate()
        .filter(|(_, e)| matches!(e, Event::Tapped { obj, .. } if lands.contains(obj)))
        .map(|(i, _)| i)
        .collect();
    assert_eq!(paid.len(), 2);
    assert!(paid.iter().all(|i| *i > began && *i < declared));
    // An additional combat phase (Relentless Assault untaps the Bears): attacking again
    // costs {2} again.
    t.advance_to(P0, Step::PostcombatMain);
    let spell = t.hand(P0, "Relentless Assault");
    lots(&mut t, P0);
    t.cast(P0, spell).go();
    t.resolve_all();
    t.g.players[P0.idx()].mana_pool = Default::default();
    assert!(!tapped(&t, bears));
    t.answer_yes(P0, true);
    let attacks = crate::r_s09_common::declare_in_next_combat(&mut t, P0, &[(bears, pl(P1))]);
    assert_eq!(attacks.len(), 1);
    assert_eq!(tapped_lands(&t, &lands), 4);
}

#[test]
fn lightning_ragers_and_rite_of_the_raging_storm() {
    cr!("508.1c", "111.4");
    ruling!(
        "Rite of the Raging Storm",
        "If Rite of the Raging Storm leaves the battlefield before attackers are declared, creatures named Lightning Rager can attack you or planeswalkers you control."
    );
    ruling!(
        "Rite of the Raging Storm",
        "The tokens aren't named Elemental. They are named only Lightning Rager."
    );
    let mut t = TestGame::new(2);
    let rite = t.battlefield(P1, "Rite of the Raging Storm");
    // P0's upkeep: P0 creates a Lightning Rager.
    t.set_step(P0, Step::Untap);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    let ragers = tokens(&t, P0);
    assert_eq!(ragers.len(), 1);
    let rager = ragers[0];
    let o = t.obj_now(rager);
    assert_eq!(o.chars.name.as_str(), "Lightning Rager");
    assert!(o.chars.has_subtype("Elemental"));
    t.advance_to(P0, Step::BeginningOfCombat);
    assert!(!legal_attack(&mut t, &[(rager, pl(P1))]));
    // The Rite leaves the battlefield before attackers are declared.
    exile(&mut t, rite);
    assert!(legal_attack(&mut t, &[(rager, pl(P1))]));
    let attacks = declare(&mut t, P0, &[(rager, pl(P1))]);
    assert_eq!(attacks, vec![(rager, pl(P1))]);
}
