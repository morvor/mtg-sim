//! CR 508.1–508.3: declaring attackers, attack restrictions/requirements/costs, and
//! attack triggers.

use crate::r506_common::*;
use mtg_engine::ability::*;
use mtg_engine::combat::{attack_declaration_legal, attack_options};
use mtg_engine::decision::Decision;
use mtg_engine::events::Event;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn berserker() -> CardDef {
    custom_card(
        "Eager Berserker",
        "Creature — Human Berserker",
        Some((2, 2)),
        "This creature attacks each combat if able.",
    )
}

#[test]
fn illegal_declarations_are_undone() {
    cr!("508.1");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    t.g.tap(giant);
    // Attacking with a tapped creature is illegal: the whole declaration is undone.
    declare(
        &mut t,
        &[(bears, Entity::Player(P1)), (giant, Entity::Player(P1))],
    );
    go_to(&mut t, Step::DeclareAttackers);
    assert!(t.g.attackers().is_empty());
    assert!(!t.obj_now(bears).tapped, "nothing was tapped");
    // The declaration is a turn-based action: nothing was put on the stack.
    assert_eq!(t.stack_len(), 0);

    // Declaring the same creature twice is illegal too.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    declare(
        &mut t,
        &[(bears, Entity::Player(P1)), (bears, Entity::Player(P1))],
    );
    go_to(&mut t, Step::DeclareAttackers);
    assert!(t.g.attackers().is_empty());
}

#[test]
fn attackers_must_be_untapped_nonbattle_and_controlled_since_turn_began() {
    cr!("508.1a");
    let mut t = TestGame::new(2);
    let ready = t.battlefield(P0, "Grizzly Bears");
    let tapped = t.battlefield(P0, "Hill Giant");
    t.g.tap(tapped);
    let sick = t.battlefield_sick(P0, "Craw Wurm");
    let hasty = t.battlefield_sick(P0, "Raging Goblin");
    let odd = bf(
        &mut t,
        P0,
        with_counters_base(
            custom_with("Odd Siege", "Battle Creature — Siege", Some((3, 3)), vec![]),
            None,
            Some(3),
        ),
    );
    to_combat(&mut t, P0);
    let opts: Vec<ObjectId> = attack_options(&t.g).into_iter().map(|(c, _)| c).collect();
    assert!(opts.contains(&ready));
    assert!(opts.contains(&hasty), "haste");
    assert!(!opts.contains(&tapped));
    assert!(!opts.contains(&sick));
    assert!(!opts.contains(&odd));
    // A creature P0 gained control of this turn can't attack.
    let stolen = t.battlefield(P1, "Grizzly Bears");
    apply(
        &mut t,
        P0,
        Effect::GainControl {
            what: Sel::Target(0),
            who: PlayerRef::You,
            duration: Duration::EndOfTurn,
        },
        &[stolen],
    );
    assert!(!t.g.can_attack(stolen));
}

#[test]
fn creatures_that_cant_attack_alone_can_attack_together() {
    cr!("508.1c");
    // Trusty Companion: "This creature can't attack alone."
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Trusty Companion");
    let b = t.battlefield(P0, "Trusty Companion");
    to_combat(&mut t, P0);
    let opts = attack_options(&t.g);
    let p1 = Entity::Player(P1);
    assert!(!attack_declaration_legal(&t.g, &opts, &[(a, p1)]));
    assert!(attack_declaration_legal(&t.g, &opts, &[(a, p1), (b, p1)]));
    assert!(attack_declaration_legal(&t.g, &opts, &[]));
    // Restrictions that forbid attacking entirely.
    let wall = t.battlefield(P0, "Wall of Stone");
    assert!(!t.g.can_attack(wall));
    // The engine enforces it when the declaration is made.
    declare(&mut t, &[(a, p1)]);
    go_to(&mut t, Step::DeclareAttackers);
    assert!(t.g.attackers().is_empty());
}

#[test]
fn requirements_are_maximized_without_breaking_restrictions() {
    cr!("508.1d");
    // CR 508.1d example: one creature "attacks if able", one with no abilities, and "No more
    // than one creature can attack each combat" (Silent Arbiter).
    let mut t = TestGame::new(2);
    let must = bf(&mut t, P0, berserker());
    let other = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Silent Arbiter");
    to_combat(&mut t, P0);
    let opts = attack_options(&t.g);
    let p1 = Entity::Player(P1);
    assert!(attack_declaration_legal(&t.g, &opts, &[(must, p1)]));
    assert!(!attack_declaration_legal(&t.g, &opts, &[(other, p1)]));
    assert!(!attack_declaration_legal(&t.g, &opts, &[(must, p1), (other, p1)]));
    assert!(!attack_declaration_legal(&t.g, &opts, &[]));
    // Declaring with the other creature is illegal; the engine declares the legal attack.
    declare(&mut t, &[(other, p1)]);
    go_to(&mut t, Step::DeclareAttackers);
    assert_eq!(t.g.attackers(), vec![must]);
}

#[test]
fn costs_to_attack_arent_required_to_obey_requirements() {
    cr!("508.1d");
    // Propaganda: "Creatures can't attack you unless their controller pays {2} for each
    // creature they control that's attacking you."
    let mut t = TestGame::new(2);
    let must = bf(&mut t, P0, berserker());
    t.lands(P0, "Plains", 2);
    t.battlefield(P1, "Propaganda");
    to_combat(&mut t, P0);
    let opts = attack_options(&t.g);
    assert!(attack_declaration_legal(&t.g, &opts, &[]));
    assert!(attack_declaration_legal(&t.g, &opts, &[(must, Entity::Player(P1))]));
    // The player may choose not to attack.
    go_to(&mut t, Step::DeclareAttackers);
    assert!(t.g.attackers().is_empty());
}

#[test]
fn attacks_if_able_this_turn_applies_to_each_combat() {
    cr!("508.1d");
    let mut t = TestGame::new(2);
    // Serra Angel has vigilance, so it can attack in both combats.
    let angel = t.battlefield(P0, "Serra Angel");
    apply(
        &mut t,
        P1,
        Effect::AddRestriction {
            restriction: Restriction::MustAttack(Filter::In(Box::new(Sel::Target(0)))),
            duration: Duration::EndOfTurn,
        },
        &[angel],
    );
    // P0 tries not to attack each time; the requirement forces the attack.
    declare(&mut t, &[]);
    go_to(&mut t, Step::DeclareAttackers);
    assert_eq!(t.g.attackers(), vec![angel]);
    t.g.add_extra_combat(true);
    declare(&mut t, &[]);
    go_to(&mut t, Step::EndOfCombat);
    go_to(&mut t, Step::DeclareAttackers);
    assert_eq!(t.g.turn.combat_phases, 2);
    assert_eq!(t.g.attackers(), vec![angel]);
}

#[test]
fn declared_attackers_become_tapped() {
    cr!("508.1f");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let angel = t.battlefield(P0, "Serra Angel");
    declare(
        &mut t,
        &[(bears, Entity::Player(P1)), (angel, Entity::Player(P1))],
    );
    go_to(&mut t, Step::DeclareAttackers);
    assert!(t.obj_now(bears).tapped);
    assert!(!t.obj_now(angel).tapped, "vigilance");
    // Tapping isn't a cost: it isn't mana-related and happens as part of attacking.
    assert!(t.g.turn_events.iter().any(|e| matches!(
        e,
        Event::Tapped {
            obj,
            for_mana: false
        } if *obj == bears
    )));
}

#[test]
fn optional_costs_as_a_creature_attacks() {
    cr!("508.1g");
    ruling!(
        "Nef-Crop Entangler",
        "creatures put onto the battlefield attacking can't be exerted"
    );
    // Nef-Crop Entangler: "You may exert this creature as it attacks. When you do, it gets
    // +1/+2 until end of turn."
    let mut t = TestGame::new(2);
    let e = t.battlefield(P0, "Nef-Crop Entangler");
    declare(&mut t, &[(e, Entity::Player(P1))]);
    t.answer_yes(P0, true);
    go_to(&mut t, Step::DeclareAttackers);
    t.resolve_all();
    assert!(t.obj_now(e).exerted);
    assert_eq!(t.pt(e), (3, 3));

    // Declining the optional cost.
    let mut t = TestGame::new(2);
    let e = t.battlefield(P0, "Nef-Crop Entangler");
    declare(&mut t, &[(e, Entity::Player(P1))]);
    t.answer_yes(P0, false);
    go_to(&mut t, Step::DeclareAttackers);
    t.resolve_all();
    assert!(!t.obj_now(e).exerted);
    assert_eq!(t.pt(e), (2, 1));

    // Put onto the battlefield attacking: no chance to exert.
    let mut t = TestGame::new(2);
    to_combat(&mut t, P0);
    let id = t.hand(P0, "Nef-Crop Entangler");
    let _ = id;
    let new = enter_with(
        &mut t,
        P0,
        (*card("Nef-Crop Entangler")).clone(),
        Some(Entity::Player(P1)),
        None,
    );
    assert!(t.g.is_attacking(new));
    assert_eq!(
        count_asked(&t, P0, |d| matches!(d, Decision::YesNo { .. })),
        0
    );
    assert!(!t.obj_now(new).exerted);
}

#[test]
fn total_attack_cost_is_paid_with_mana_abilities() {
    cr!("508.1h", "508.1i");
    // Two Propagandas: each creature attacking P1 costs {4}.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let lands = t.lands(P0, "Plains", 5);
    t.battlefield(P1, "Propaganda");
    t.battlefield(P1, "Propaganda");
    to_combat(&mut t, P0);
    let total = mtg_engine::combat::total_attack_cost(&t.g, &[(bears, Entity::Player(P1))]);
    assert_eq!(total.mana.as_ref().unwrap().mana_value(), 4);
    declare(&mut t, &[(bears, Entity::Player(P1))]);
    go_to(&mut t, Step::DeclareAttackers);
    assert!(t.g.is_attacking(bears));
    // Mana abilities of four lands were activated to pay.
    let tapped = lands.iter().filter(|l| t.obj_now(**l).tapped).count();
    assert_eq!(tapped, 4);
    go_to(&mut t, Step::EndOfCombat);
    assert_eq!(t.life(P1), 18);
}

#[test]
fn partial_payment_of_attack_costs_isnt_allowed() {
    cr!("508.1j");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let lands = t.lands(P0, "Plains", 3);
    t.battlefield(P1, "Propaganda");
    // Two attackers cost {4}; P0 has only three lands.
    declare(
        &mut t,
        &[(bears, Entity::Player(P1)), (giant, Entity::Player(P1))],
    );
    go_to(&mut t, Step::DeclareAttackers);
    assert!(t.g.attackers().is_empty());
    assert!(lands.iter().all(|l| !t.obj_now(*l).tapped), "nothing was paid");
    assert!(!t.obj_now(bears).tapped && !t.obj_now(giant).tapped);
}

#[test]
fn chosen_creatures_still_controlled_become_attacking_until_combat_ends() {
    cr!("508.1k");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    // "Creatures can't attack you unless their controller sacrifices a creature."
    let tax = restriction(Restriction::AttackCost {
        attackers: Filter::creature(),
        defender: PlayerFilter::You,
        planeswalkers: false,
        cost: Cost::default().with(CostPart::Sacrifice {
            filter: Filter::creature(),
            count: Value::c(1),
        }),
    });
    bf(&mut t, P1, custom_with("Grim Toll", "Enchantment", None, vec![tax]));
    // Only `a` attacks; P0 sacrifices `b` to pay.
    declare(&mut t, &[(a, Entity::Player(P1))]);
    t.answer_choose(P0, &[Entity::Object(b)]);
    go_to(&mut t, Step::DeclareAttackers);
    assert!(!t.on_battlefield(b));
    assert!(t.g.is_attacking(a));
    go_to(&mut t, Step::EndOfCombat);
    assert!(t.g.is_attacking(a), "still attacking until combat ends");
    go_to(&mut t, Step::PostcombatMain);
    assert!(!t.g.is_attacking(a));

    // If paying the cost sacrifices a chosen creature, it doesn't become attacking.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let tax = restriction(Restriction::AttackCost {
        attackers: Filter::creature(),
        defender: PlayerFilter::You,
        planeswalkers: false,
        cost: Cost::default().with(CostPart::Sacrifice {
            filter: Filter::creature(),
            count: Value::c(1),
        }),
    });
    bf(&mut t, P1, custom_with("Grim Toll", "Enchantment", None, vec![tax]));
    declare(&mut t, &[(a, Entity::Player(P1))]);
    t.answer_choose(P0, &[Entity::Object(a)]);
    go_to(&mut t, Step::DeclareAttackers);
    assert!(!t.on_battlefield(a));
    assert!(t.g.attackers().is_empty());
}

#[test]
fn abilities_trigger_on_attackers_being_declared() {
    cr!("508.1m", "508.3a");
    let mut t = TestGame::new(2);
    let a = bf(
        &mut t,
        P0,
        custom_card(
            "Charging Herald",
            "Creature — Human",
            Some((2, 2)),
            "Whenever this creature attacks, you gain 1 life.",
        ),
    );
    // Briar Patch: "Whenever a creature attacks you, it gets -1/-0 until end of turn."
    t.battlefield(P1, "Briar Patch");
    t.battlefield(P0, "Briar Patch");
    declare(&mut t, &[(a, Entity::Player(P1))]);
    go_to(&mut t, Step::DeclareAttackers);
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
    // Only P1's Briar Patch triggered: the creature attacked P1, not P0.
    assert_eq!(t.pt(a), (1, 2));
}

#[test]
fn attacks_triggers_dont_trigger_for_creatures_put_onto_battlefield_attacking() {
    cr!("508.3a", "508.3b");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Briar Patch");
    let watcher = restriction_free_watcher(&mut t);
    to_combat(&mut t, P0);
    let herald = custom_card(
        "Charging Herald",
        "Creature — Human",
        Some((2, 2)),
        "Whenever this creature attacks, you gain 1 life.",
    );
    let a = enter_with(&mut t, P0, herald, Some(Entity::Player(P1)), None);
    assert!(t.g.is_attacking(a));
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.life(P1), 20, "{watcher:?} didn't trigger");
    assert_eq!(t.pt(a), (2, 2));
}

/// P1's permanent: "Whenever you're attacked, you gain 1 life."
fn restriction_free_watcher(t: &mut TestGame) -> ObjectId {
    bf(
        t,
        P1,
        custom_card(
            "Watchtower",
            "Enchantment",
            None,
            "Whenever you're attacked, you gain 1 life.",
        ),
    )
}

#[test]
fn player_is_attacked_triggers_once_per_attacked_player() {
    cr!("508.3b");
    let mut t = TestGame::new(3);
    restriction_free_watcher(&mut t);
    let jace = t.battlefield(P1, "Jace Beleren");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    let c = t.battlefield(P0, "Craw Wurm");
    declare(
        &mut t,
        &[
            (a, Entity::Player(P1)),
            (b, Entity::Player(P1)),
            (c, Entity::Object(jace)),
        ],
    );
    go_to(&mut t, Step::DeclareAttackers);
    t.resolve_all();
    assert_eq!(t.life(P1), 21, "one trigger for two creatures attacking P1");

    // A planeswalker being attacked.
    let mut t = TestGame::new(2);
    let jace = t.battlefield(P1, "Jace Beleren");
    bf(
        &mut t,
        P1,
        custom_with(
            "Walker Warden",
            "Enchantment",
            None,
            vec![triggered(
                TriggerCond::IsAttacked(DamageRecipient::Object(Filter::Type(
                    CardType::Planeswalker,
                ))),
                gain(1),
            )],
        ),
    );
    let a = t.battlefield(P0, "Grizzly Bears");
    declare(&mut t, &[(a, Entity::Object(jace))]);
    go_to(&mut t, Step::DeclareAttackers);
    t.resolve_all();
    assert_eq!(t.life(P1), 21);
}

#[test]
fn player_attacks_with_creatures() {
    cr!("508.3c");
    let mut t = TestGame::new(2);
    bf(
        &mut t,
        P1,
        custom_card(
            "War Chronicle",
            "Enchantment",
            None,
            "Whenever a player attacks with two or more creatures, you gain 1 life.",
        ),
    );
    bf(
        &mut t,
        P1,
        custom_card(
            "Skirmish Chronicle",
            "Enchantment",
            None,
            "Whenever a player attacks with one or more creatures, you gain 10 life.",
        ),
    );
    let a = t.battlefield(P0, "Grizzly Bears");
    declare(&mut t, &[(a, Entity::Player(P1))]);
    go_to(&mut t, Step::DeclareAttackers);
    t.resolve_all();
    assert_eq!(t.life(P1), 30);
    // Next turn P1 attacks with two creatures: both trigger, once each.
    let b = t.battlefield(P1, "Grizzly Bears");
    let c = t.battlefield(P1, "Hill Giant");
    t.advance_to(P1, Step::BeginningOfCombat);
    declare(&mut t, &[(b, Entity::Player(P0)), (c, Entity::Player(P0))]);
    go_to(&mut t, Step::DeclareAttackers);
    t.resolve_all();
    // 30 - 2 combat damage from P0's bears + 11.
    assert_eq!(t.life(P1), 39);
}

#[test]
fn player_attacks_triggers_once_if_any_creatures_attack() {
    cr!("508.3d");
    let mut t = TestGame::new(2);
    bf(
        &mut t,
        P0,
        custom_card(
            "Battle Horn",
            "Artifact",
            None,
            "Whenever you attack, you gain 1 life.",
        ),
    );
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    declare(&mut t, &[(a, Entity::Player(P1)), (b, Entity::Player(P1))]);
    go_to(&mut t, Step::DeclareAttackers);
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
    // No attackers: no trigger.
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::DeclareAttackers);
    t.resolve_all();
    assert!(t.g.attackers().is_empty());
    assert_eq!(t.life(P0), 21);
}

#[test]
fn player_attacks_another_player() {
    cr!("508.3e");
    let mut t = TestGame::new(4);
    // P3: "Whenever an opponent attacks another one of your opponents, you gain 1 life."
    bf(
        &mut t,
        P3,
        custom_card(
            "Instigator's Ledger",
            "Enchantment",
            None,
            "Whenever an opponent attacks another one of your opponents, you gain 1 life.",
        ),
    );
    let jace = t.battlefield(P2, "Jace Beleren");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    let c = t.battlefield(P0, "Craw Wurm");
    // Attacks P1 (twice), P2's planeswalker (doesn't count), and P3 (not "another").
    declare(
        &mut t,
        &[
            (a, Entity::Player(P1)),
            (b, Entity::Player(P1)),
            (c, Entity::Object(jace)),
        ],
    );
    go_to(&mut t, Step::DeclareAttackers);
    t.resolve_all();
    assert_eq!(t.life(P3), 21);
    // A creature put onto the battlefield attacking P2 doesn't trigger it.
    let d = enter_with(&mut t, P0, vanilla("Late", 1, 1), Some(Entity::Player(P2)), None);
    assert!(t.g.is_attacking(d));
    t.resolve_all();
    assert_eq!(t.life(P3), 21);
}

#[test]
fn attacks_and_isnt_blocked_triggers_in_declare_blockers_step() {
    cr!("508.3f");
    let mut t = TestGame::new(2);
    let a = bf(
        &mut t,
        P0,
        custom_card(
            "Sly Raider",
            "Creature — Human Rogue",
            Some((2, 2)),
            "Whenever this creature attacks and isn't blocked, you gain 1 life.",
        ),
    );
    declare(&mut t, &[(a, Entity::Player(P1))]);
    go_to(&mut t, Step::DeclareAttackers);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    go_to(&mut t, Step::DeclareBlockers);
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
}

#[test]
fn active_player_gets_priority_after_declaring_attackers() {
    cr!("508.2");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    declare(&mut t, &[(a, Entity::Player(P1))]);
    let order = priority_order_in(&mut t, Step::DeclareAttackers);
    assert_eq!(order, vec![P0, P1]);
}

#[test]
fn attack_triggers_use_characteristics_at_declaration() {
    cr!("508.2a");
    // "Whenever a green creature attacks, you gain 1 life."
    let watcher = triggered(TriggerCond::Attacks(Filter::Color(Color::Green)), gain(1));
    let mut t = TestGame::new(2);
    bf(&mut t, P0, custom_with("Green Eye", "Enchantment", None, vec![watcher]));
    let blue = t.battlefield(P0, "Coral Merfolk");
    declare(&mut t, &[(blue, Entity::Player(P1))]);
    go_to(&mut t, Step::DeclareAttackers);
    // Turned green after it attacked: no trigger.
    apply(
        &mut t,
        P0,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::SetColors(ColorSet::single(Color::Green))],
            duration: Duration::EndOfTurn,
        },
        &[blue],
    );
    assert!(t.obj_now(blue).chars.colors.contains(Color::Green));
    t.resolve_all();
    go_to(&mut t, Step::EndOfCombat);
    assert_eq!(t.life(P0), 20);
    // A green attacker does trigger it.
    let mut t = TestGame::new(2);
    let watcher = triggered(TriggerCond::Attacks(Filter::Color(Color::Green)), gain(1));
    bf(&mut t, P0, custom_with("Green Eye", "Enchantment", None, vec![watcher]));
    let bears = t.battlefield(P0, "Grizzly Bears");
    declare(&mut t, &[(bears, Entity::Player(P1))]);
    go_to(&mut t, Step::DeclareAttackers);
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
}

#[test]
fn attack_triggers_are_put_on_the_stack_before_the_active_player_gets_priority() {
    cr!("508.2b");
    let mut t = TestGame::new(2);
    let a = bf(
        &mut t,
        P0,
        custom_card(
            "Charging Herald",
            "Creature — Human",
            Some((2, 2)),
            "Whenever this creature attacks, you gain 1 life.",
        ),
    );
    t.battlefield(P1, "Briar Patch");
    declare(&mut t, &[(a, Entity::Player(P1))]);
    go_to(&mut t, Step::DeclareAttackers);
    assert_eq!(t.stack_len(), 0, "triggers are waiting");
    t.script.lock().unwrap().asked.clear();
    // The active player's first priority: both triggers (from different players) are
    // already on the stack.
    t.g.advance();
    let asked = t.asked();
    assert!(matches!(asked.first(), Some((P0, Decision::Priority { .. }))));
    assert_eq!(t.stack_len(), 2);
}
