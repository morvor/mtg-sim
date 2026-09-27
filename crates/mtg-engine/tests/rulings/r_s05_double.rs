//! Rulings batch S05 — doubling (CR 701.10): doubling counters, power and toughness, and
//! damage.

use crate::r_s01_common::*;
use crate::r_s04_common::*;
use crate::r_s05_common::*;
use mtg_engine::ability::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::types::counters;
use mtg_engine::*;

/// Gives `id` +`p`/+`tough` until end of turn (as a resolving effect would).
fn pump(t: &mut TestGame, id: ObjectId, p: i32, tough: i32) {
    run_from(
        t,
        P0,
        None,
        Effect::Modify {
            what: Sel::All(Filter::Objects(vec![id])),
            mods: vec![Modification::ModifyPT(Value::c(p), Value::c(tough))],
            duration: Duration::EndOfTurn,
        },
        &[],
    );
}

#[test]
fn doubling_counters_puts_that_many_counters_which_replacement_effects_modify() {
    cr!("701.10e", "122.6", "614.1a");
    ruling!(
        "Dragonsguard Elite",
        "To double the number of +1/+1 counters on a permanent, put a number of +1/+1 counters on it equal to the number it already has. Other cards that interact with putting counters on it will interact with this effect accordingly."
    );
    supported("Dragonsguard Elite");
    supported("Hardened Scales");
    // Dragonsguard Elite: "{4}{G}{G}: Double the number of +1/+1 counters on this
    // creature." With two counters, two more are put on it — three with Hardened Scales
    // ("that many plus one").
    let mut t = TestGame::new(2);
    let elite = t.battlefield(P0, "Dragonsguard Elite");
    t.g.add_counters(Entity::Object(elite), counters::PLUS1, 2, None);
    t.battlefield(P0, "Hardened Scales");
    add_mana(&mut t, P0, ManaType::G, 6);
    t.activate(P0, elite, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(elite, counters::PLUS1), 5);
    assert_eq!(t.pt(elite), (7, 7));
}

#[test]
fn putting_a_counter_then_doubling_counts_the_counters_already_there() {
    cr!("701.10e", "122.6", "614.1a");
    ruling!(
        "Invigorating Surge",
        "To double the number of +1/+1 counters on a creature, put a number of +1/+1 counters on it equal to the number it already has. Other cards that interact with putting counters on it will interact with this effect accordingly."
    );
    supported("Invigorating Surge");
    // "Put a +1/+1 counter on target creature you control, then double the number of
    // +1/+1 counters on that creature." With Hardened Scales: 1+1 = 2, then 2+1 more.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Hardened Scales");
    give_mana_for(&mut t, P0, "Invigorating Surge");
    let surge = t.hand(P0, "Invigorating Surge");
    t.cast(P0, surge).target(bears).go();
    t.resolve_all();
    assert_eq!(t.counters(bears, counters::PLUS1), 5);
    // Without it: 1, then 1 more.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    give_mana_for(&mut t, P0, "Invigorating Surge");
    let surge = t.hand(P0, "Invigorating Surge");
    t.cast(P0, surge).target(bears).go();
    t.resolve_all();
    assert_eq!(t.counters(bears, counters::PLUS1), 2);
}

#[test]
fn distributing_then_doubling_counters_with_a_replacement_effect() {
    cr!("701.10e", "122.6", "614.1a");
    ruling!(
        "Omnivorous Flytrap",
        "To double the number of +1/+1 counters on a permanent, put a number of +1/+1 counters on it equal to the number it already has. Other effects that interact with putting counters on it will interact with this effect accordingly."
    );
    supported("Omnivorous Flytrap");
    // Six card types in the graveyard: creature, artifact, instant, sorcery, enchantment,
    // land.
    let mut t = TestGame::new(2);
    for c in [
        "Ornithopter",
        "Shock",
        "Forked Bolt",
        "Glorious Anthem",
        "Forest",
    ] {
        t.graveyard(P0, c);
    }
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Hardened Scales");
    // Both counters on the Bears: 2+1 = 3; doubled: 3+1 more = 7.
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![2]));
    enter(&mut t, P0, "Omnivorous Flytrap");
    t.resolve_all();
    assert_eq!(t.counters(bears, counters::PLUS1), 7);
}

#[test]
fn doubling_power_gives_plus_x_where_x_is_its_power_as_it_begins_to_apply() {
    cr!("701.10b", "613.4c");
    ruling!(
        "Dragonclaw Strike",
        "If an effect instructs you to “double” a creature’s power, that creature gets +X/+0, where X is its power as that effect begins to apply. Similarly, a creature whose toughness is doubled gets +0/+X, where X is its toughness as the effect begins to apply."
    );
    ruling!(
        "Reckless Amplimancer",
        "If an effect instructs you to \"double\" a creature's power, that creature gets +X/+0, where X is its power as that effect begins to apply. The same is true for toughness."
    );
    supported("Dragonclaw Strike");
    supported("Reckless Amplimancer");
    // Dragonclaw Strike: "Double the power and toughness of target creature you control
    // until end of turn. Then it fights up to one target creature an opponent controls."
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    pump(&mut t, giant, 1, 0);
    assert_eq!(t.pt(giant), (4, 3));
    // {2/G}{2/U}{2/R}.
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Island", 1);
    t.lands(P0, "Mountain", 1);
    let strike = t.hand(P0, "Dragonclaw Strike");
    t.cast(P0, strike).targets(&[Entity::Object(giant)]).go();
    t.resolve_all();
    assert_eq!(t.pt(giant), (8, 6));
    // Later changes aren't doubled: it got +4/+3, and a later +1/+1 applies on top.
    pump(&mut t, giant, 1, 1);
    assert_eq!(t.pt(giant), (9, 7));
    // Reckless Amplimancer: "{4}{G}: Double this creature's power and toughness until end
    // of turn." 2/2, then 4/4, then 8/8.
    let amp = t.battlefield(P0, "Reckless Amplimancer");
    add_mana(&mut t, P0, ManaType::G, 10);
    t.activate(P0, amp, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(amp), (4, 4));
    t.activate(P0, amp, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(amp), (8, 8));
}

#[test]
fn doubling_a_negative_power_makes_it_more_negative() {
    cr!("701.10c");
    ruling!(
        "Choose Your Weapon",
        "If a creature's power is less than 0 when it's doubled, instead that creature gets -X/-0, where X is how much less than 0 its power is. For example, if an effect has given a 2/2 creature -4/-0 so that it's a -2/2 creature, doubling its power and toughness gives it -2/+2, and it becomes a -4/4 creature."
    );
    supported("Choose Your Weapon");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    pump(&mut t, bears, -4, 0);
    assert_eq!(t.pt(bears), (-2, 2));
    give_mana_for(&mut t, P0, "Choose Your Weapon");
    let spell = t.hand(P0, "Choose Your Weapon");
    t.cast(P0, spell).modes(&[0]).target(bears).go();
    t.resolve_all();
    assert_eq!(t.pt(bears), (-4, 4));
}

#[test]
fn divided_damage_is_divided_before_it_is_doubled() {
    cr!("701.10g", "601.2d", "614.1a");
    ruling!(
        "Angrath's Marauders",
        "If an effect such as that of Chandra's Pyrohelix asks you to divide damage among targets, you must divide the unmodified damage before doubling it."
    );
    supported("Angrath's Marauders");
    supported("Forked Bolt");
    // Forked Bolt: "deals 2 damage divided as you choose among one or two targets": 1 and
    // 1, each doubled to 2.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Angrath's Marauders");
    let bears = t.battlefield(P1, "Grizzly Bears");
    give_mana_for(&mut t, P0, "Forked Bolt");
    let bolt = t.hand(P0, "Forked Bolt");
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![1, 1]));
    let from = t.asked().len();
    t.cast(P0, bolt)
        .targets(&[Entity::Object(bears), Entity::Player(P1)])
        .go();
    let totals: Vec<u32> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::Divide { total, .. } => Some(*total),
            _ => None,
        })
        .collect();
    assert_eq!(totals, vec![2]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(t.life(P1), 18);
}

#[test]
fn combat_damage_is_assigned_before_it_is_doubled() {
    cr!("701.10g", "510.1a", "702.19b");
    ruling!(
        "Twinflame Tyrant",
        "If damage dealt by a source you control is being divided or assigned among multiple permanents and/or players, that damage is divided or assigned before doubling. For example, if you attack with a 5/5 creature with trample and it's blocked by a 2/2 creature, you can assign 2 damage to the blocker and 3 damage to the defending player. Those amounts are then doubled to 4 and 6, respectively."
    );
    supported("Twinflame Tyrant");
    // Colossal Dreadmaw (6/6 trample) blocked by Grizzly Bears: 2 to the Bears (lethal
    // damage before doubling) and 4 to the player, dealt as 4 and 8.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Twinflame Tyrant");
    let maw = t.battlefield(P0, "Colossal Dreadmaw");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer(P0, DecisionKind::Damage, Answer::Numbers(vec![2, 4]));
    let from = t.asked().len();
    attack_with(&mut t, &[(maw, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[(bears, maw)]);
    // 6 damage to assign; 2 is lethal for the Bears.
    let assigned: Vec<(u32, Vec<u32>)> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::AssignCombatDamage { amount, lethal, .. } => {
                Some((*amount, lethal.clone()))
            }
            _ => None,
        })
        .collect();
    assert_eq!(assigned.len(), 1);
    assert_eq!(assigned[0].0, 6);
    assert_eq!(assigned[0].1.first(), Some(&2));
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(t.life(P1), 12);
}

#[test]
fn doubling_each_kind_of_counter_with_a_replacement_effect() {
    cr!("701.10e", "122.6", "614.1a");
    ruling!(
        "Zimone, Paradox Sculptor",
        "To double the number of each kind of counter on a permanent, put another counter on it for each counter it already has. Effects that interact with counters being put onto permanents, such as the effect of Branching Evolution, apply as appropriate."
    );
    supported("Zimone, Paradox Sculptor");
    supported("Branching Evolution");
    // Zimone: "{G}{U}, {T}: Double the number of each kind of counter on up to two target
    // creatures and/or artifacts you control." Two +1/+1 counters and one charge counter:
    // two more +1/+1 counters (four with Branching Evolution) and one more charge counter.
    let mut t = TestGame::new(2);
    let zimone = t.battlefield(P0, "Zimone, Paradox Sculptor");
    let ballista = t.battlefield(P0, "Walking Ballista");
    t.g.add_counters(Entity::Object(ballista), counters::PLUS1, 2, None);
    t.g.add_counters(Entity::Object(ballista), "charge", 1, None);
    t.battlefield(P0, "Branching Evolution");
    add_mana(&mut t, P0, ManaType::G, 1);
    add_mana(&mut t, P0, ManaType::U, 1);
    t.activate(P0, zimone, 0, &[Entity::Object(ballista)]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(ballista, counters::PLUS1), 6);
    assert_eq!(t.counters(ballista, "charge"), 2);
}
