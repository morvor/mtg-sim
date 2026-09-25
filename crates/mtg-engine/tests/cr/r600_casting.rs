//! CR 601: casting spells.

use super::r600_common::*;
use mtg_engine::ability::*;
use mtg_engine::decision::{Action, Answer, Decision};
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn exile_permission_sorcery() -> CardDef {
    // "You may play cards you own in exile this turn."
    CB::new("Exile Permission")
        .sorcery()
        .cost("{0}")
        .spell(Body::effect(Effect::GrantPlayPermission {
            who: PlayerRef::You,
            what: Sel::All(Filter::and(vec![
                Filter::InZone(ZoneKind::Exile),
                Filter::OwnedBy(PlayerRel::You),
            ])),
            duration: Duration::EndOfTurn,
            free: false,
        }))
        .build()
}

#[test]
fn playing_a_card_means_playing_a_land_or_casting_a_spell() {
    cr!("601.1a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 2);
    let land = t.exile(P0, "Mountain");
    let bears = t.exile(P0, "Grizzly Bears");
    // Without a permission neither card can be played from exile.
    assert!(t.play_land(P0, land).is_err());
    assert!(t.cast(P0, bears).try_go().is_err());
    let s = t.custom(P0, exile_permission_sorcery(), Zone::Hand(P0));
    t.cast(P0, s).go();
    t.resolve();
    // "Play" permits both playing the land and casting the spell.
    t.play_land(P0, land).expect("the land can be played");
    assert_eq!(t.named_on_battlefield("Mountain").len(), 1);
    t.cast(P0, bears).go();
    t.resolve();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

#[test]
fn casting_moves_the_card_to_the_top_of_the_stack_under_the_casters_control() {
    cr!("601.2", "601.2a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 2);
    // A static effect that modifies the characteristics of spells applies as soon as
    // the spell is put on the stack (CR 601.2a / 611.2f).
    let painter = CB::new("Spell Painter")
        .enchantment()
        .ability(stat(StaticEffect::Continuous {
            affected: Filter::and(vec![Filter::Spell, Filter::ControlledBy(PlayerRel::You)]),
            mods: vec![Modification::AddColors(ColorSet::single(Color::Blue))],
        }))
        .build();
    t.custom(P0, painter, Zone::Battlefield);
    let bolt = t.hand(P0, "Lightning Bolt");
    let shock = t.hand(P0, "Shock");
    let s1 = t.cast(P0, bolt).target(P1).go();
    // A new object (CR 400.7) in the stack zone, controlled by the caster.
    assert_ne!(s1, bolt);
    assert!(!t.is_live(bolt));
    let o = t.obj(s1);
    assert_eq!(o.zone, Zone::Stack);
    assert_eq!(o.controller, P0);
    assert_eq!(o.chars.name, "Lightning Bolt");
    assert!(o.chars.is(CardType::Instant));
    assert!(o.chars.colors.contains(Color::Red) && o.chars.colors.contains(Color::Blue));
    // The next spell becomes the topmost object.
    let s2 = t.cast(P0, shock).target(P1).go();
    assert_eq!(t.stack, vec![s1, s2]);
}

#[test]
fn casting_an_opponents_card_makes_the_caster_its_controller() {
    cr!("601.2a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 2);
    let bears = t.exile(P1, "Grizzly Bears");
    // Grant P0 permission to cast P1's exiled card.
    t.g.play_grants.push(mtg_engine::casting::PlayGrant {
        player: P0,
        object: bears,
        duration: Duration::EndOfTurn,
        free: false,
        source: None,
        turn: 1,
    });
    let s = t.cast(P0, bears).go();
    assert_eq!(t.obj(s).controller, P0);
    assert_eq!(t.obj(s).owner, P1);
    t.resolve();
    let b = t.named_on_battlefield("Grizzly Bears")[0];
    assert_eq!(t.obj(b).controller, P0);
}

#[test]
fn x_is_announced_before_targets_and_modes_and_costs_are_announced() {
    cr!("601.2b", "601.2d");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 5);
    let a = t.battlefield(P1, "Grizzly Bears");
    let thunder = t.hand(P0, "Rolling Thunder");
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![2, 1]));
    t.cast(P0, thunder)
        .x(3)
        .targets(&[Entity::Object(a), Entity::Player(P1)])
        .go();
    let asked = t.asked();
    let pos = |f: &dyn Fn(&Decision) -> bool| asked.iter().position(|(_, d)| f(d)).unwrap();
    let x_at = pos(&|d| matches!(d, Decision::ChooseX { .. }));
    let tgt_at = pos(&|d| matches!(d, Decision::ChooseTargets { .. }));
    let div_at = pos(&|d| matches!(d, Decision::Divide { .. }));
    assert!(x_at < tgt_at && tgt_at < div_at);
    // X=3 was locked in and paid: RR + 3.
    assert_eq!(t.permanents().filter(|o| o.tapped).count(), 5);
    t.resolve();
    assert!(!t.on_battlefield(a));
    assert_eq!(t.life(P1), 19);
}

#[test]
fn each_target_of_a_divided_effect_receives_at_least_one() {
    cr!("601.2d");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 5);
    let a = t.battlefield(P1, "Grizzly Bears");
    let thunder = t.hand(P0, "Rolling Thunder");
    // An illegal division (0 to one target) is rejected; the default division is used.
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![0, 3]));
    t.cast(P0, thunder)
        .x(3)
        .targets(&[Entity::Object(a), Entity::Player(P1)])
        .go();
    let top = *t.stack.last().unwrap();
    let div = t.obj(top).stack.as_ref().unwrap().chosen[0].divided[0].clone();
    assert_eq!(div.iter().sum::<u32>(), 3);
    assert!(div.iter().all(|n| *n >= 1));
}

#[test]
fn modes_are_announced_while_casting() {
    cr!("601.2b");
    let mut t = TestGame::new(2);
    let charm = CB::new("Choice Charm")
        .instant()
        .cost("{0}")
        .spell(Body {
            targets: vec![],
            effect: Effect::Noop,
            modal: Some(Modal {
                min: Value::c(1),
                max: Value::c(1),
                allow_repeat: false,
                modes: vec![
                    Mode {
                        text: "gain 3 life".into(),
                        targets: vec![],
                        effect: gain(3),
                        cost: None,
                    },
                    Mode {
                        text: "draw a card".into(),
                        targets: vec![],
                        effect: draw(1),
                        cost: None,
                    },
                ],
                per_mode_cost: false,
                chooser: ModeChooser::Controller,
            }),
        })
        .build();
    let c = t.custom(P0, charm, Zone::Hand(P0));
    let hand = t.hand_size(P0);
    let s = t.cast(P0, c).modes(&[1]).go();
    assert_eq!(t.obj(s).stack.as_ref().unwrap().chosen[0].mode, Some(1));
    t.resolve();
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.hand_size(P0), hand); // cast one, drew one
}

#[test]
fn additional_costs_are_announced_and_paid() {
    cr!("601.2b", "601.2f");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 2);
    // "Kicker {1}. Deal 1 damage to any target; if kicked, 3 instead."
    let kicked_bolt = CB::new("Kicked Bolt")
        .instant()
        .cost("{R}")
        .ability(AbilityDef::new(
            AbilityKind::Keyword(mtg_engine::keywords::Keyword::with_cost(
                mtg_engine::keywords::KeywordKind::Kicker,
                mana_cost("{1}"),
            )),
            "Kicker {1}",
        ))
        .spell(Body::simple(
            vec![TargetSpec::any_target()],
            Effect::If {
                cond: Condition::CostPaid("kicker".into()),
                then: Box::new(Effect::DealDamage {
                    source: Sel::This,
                    amount: Value::c(3),
                    to: Sel::Target(0),
                }),
                otherwise: Box::new(Effect::DealDamage {
                    source: Sel::This,
                    amount: Value::c(1),
                    to: Sel::Target(0),
                }),
            },
        ))
        .build();
    let c = t.custom(P0, kicked_bolt, Zone::Hand(P0));
    t.cast(P0, c).kicked(true).target(P1).go();
    assert_eq!(t.permanents().filter(|o| o.tapped).count(), 2);
    t.resolve();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn phyrexian_mana_can_be_paid_with_life() {
    cr!("601.2b", "601.2h");
    let mut t = TestGame::new(2);
    let probe = CB::new("Phyrexian Probe")
        .sorcery()
        .cost("{U/P}")
        .spell(Body::effect(draw(1)))
        .build();
    let c = t.custom(P0, probe, Zone::Hand(P0));
    t.cast(P0, c).go();
    assert_eq!(t.life(P0), 18);
}

#[test]
fn the_same_object_can_be_chosen_once_for_each_instance_of_target() {
    cr!("601.2c");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 2);
    // "Destroy target nonblack creature and target land."
    let spores = CB::new("Plague Spores Test")
        .sorcery()
        .cost("{B}{B}")
        .spell(Body::simple(
            vec![
                TargetSpec::object(
                    Filter::and(vec![Filter::creature(), Filter::not(Filter::Color(Color::Black))]),
                    "target nonblack creature",
                ),
                TargetSpec::object(Filter::Type(CardType::Land), "target land"),
            ],
            Effect::Seq(vec![
                Effect::Destroy {
                    what: Sel::Target(0),
                    no_regen: true,
                },
                Effect::Destroy {
                    what: Sel::Target(1),
                    no_regen: true,
                },
            ]),
        ))
        .build();
    let arbor = t.battlefield(P1, "Dryad Arbor");
    let c = t.custom(P0, spores, Zone::Hand(P0));
    let s = t
        .cast(P0, c)
        .target(arbor)
        .target(arbor)
        .go();
    let chosen = &t.obj(s).stack.as_ref().unwrap().chosen[0].targets;
    assert_eq!(chosen, &vec![vec![Entity::Object(arbor)], vec![Entity::Object(arbor)]]);
    t.resolve();
    assert!(!t.on_battlefield(arbor));
}

fn tap_two() -> CardDef {
    CB::new("Tap Two")
        .instant()
        .cost("{0}")
        .spell(Body::simple(
            vec![TargetSpec {
                min: 2,
                max: Value::c(2),
                ..TargetSpec::object(Filter::creature(), "two target creatures")
            }],
            Effect::Tap {
                what: Sel::Target(0),
            },
        ))
        .build()
}

#[test]
fn the_same_target_cant_be_chosen_twice_for_one_instance_of_target() {
    cr!("601.2c");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let c = t.custom(P0, tap_two(), Zone::Hand(P0));
    // Only one creature: "two target creatures" can't be satisfied.
    assert!(t.cast(P0, c).try_go().is_err());
    assert!(t.in_hand(P0, "Tap Two"));
    let b = t.battlefield(P1, "Hill Giant");
    // Choosing the same creature twice is not a legal choice.
    t.answer_targets(P0, &[Entity::Object(a), Entity::Object(a)]);
    let s = t.cast(P0, c).go();
    let chosen = &t.obj(s).stack.as_ref().unwrap().chosen[0].targets[0];
    assert_eq!(chosen.len(), 2);
    assert!(chosen.contains(&Entity::Object(a)) && chosen.contains(&Entity::Object(b)));
}

#[test]
fn becomes_the_target_triggers_wait_until_the_spell_is_cast() {
    cr!("601.2c", "601.2i");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 1);
    let watcher = CB::new("Target Watcher")
        .creature(2, 2)
        .ability(trig(
            TriggerCond::BecomesTarget {
                filter: Filter::Source,
                by: PlayerRel::Any,
            },
            Body::effect(gain(1)),
        ))
        .build();
    let w = t.custom(P1, watcher, Zone::Battlefield);
    let bolt = t.hand(P0, "Lightning Bolt");
    let s = t.cast(P0, bolt).target(w).go();
    // The spell is fully cast before the trigger goes on the stack, above it.
    assert_eq!(t.stack, vec![s]);
    t.settle();
    assert_eq!(t.stack.len(), 2);
    assert_eq!(t.stack[0], s);
    t.resolve();
    assert_eq!(t.life(P1), 21);
}

#[test]
fn a_mode_without_targets_can_be_chosen_when_another_needs_unavailable_targets() {
    cr!("601.2c");
    let mut t = TestGame::new(2);
    let charm = CB::new("Kill or Gain")
        .instant()
        .cost("{0}")
        .spell(Body {
            targets: vec![],
            effect: Effect::Noop,
            modal: Some(Modal {
                min: Value::c(1),
                max: Value::c(1),
                allow_repeat: false,
                modes: vec![
                    Mode {
                        text: "destroy target creature".into(),
                        targets: vec![target_creature()],
                        effect: Effect::Destroy {
                            what: Sel::Target(0),
                            no_regen: false,
                        },
                        cost: None,
                    },
                    Mode {
                        text: "gain 2 life".into(),
                        targets: vec![],
                        effect: gain(2),
                        cost: None,
                    },
                ],
                per_mode_cost: false,
                chooser: ModeChooser::Controller,
            }),
        })
        .build();
    let c = t.custom(P0, charm, Zone::Hand(P0));
    // No creatures: the first mode can't be chosen, but the spell is castable with the second.
    t.cast(P0, c).modes(&[1]).go();
    t.resolve();
    assert_eq!(t.life(P0), 22);
}

#[test]
fn the_number_of_targets_is_announced_for_a_variable_number_of_targets() {
    cr!("601.2c");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let _b = t.battlefield(P1, "Hill Giant");
    let up_to_two = CB::new("Up To Two Taps")
        .instant()
        .cost("{0}")
        .spell(Body::simple(
            vec![TargetSpec::up_to(2, TargetKind::Object(Filter::creature()), "up to two")],
            Effect::Tap {
                what: Sel::Target(0),
            },
        ))
        .build();
    let c = t.custom(P0, up_to_two, Zone::Hand(P0));
    let s = t.cast(P0, c).targets(&[Entity::Object(a)]).go();
    assert_eq!(t.obj(s).stack.as_ref().unwrap().chosen[0].targets[0].len(), 1);
    t.resolve();
    assert!(t.obj_now(a).tapped);
    assert_eq!(t.permanents().filter(|o| o.tapped).count(), 1);
}

fn cost_reducer(name: &str, filter: Filter, n: i32) -> CardDef {
    CB::new(name)
        .enchantment()
        .ability(stat(StaticEffect::CostModifier(CostModifier {
            applies_to: CostTarget::Spells(filter),
            who: PlayerRel::You,
            change: CostChange::ReduceGeneric(Value::c(n)),
        })))
        .build()
}

#[test]
fn total_cost_includes_increases_and_reductions_but_not_below_zero() {
    cr!("601.2f");
    let mut t = TestGame::new(2);
    // Thalia: noncreature spells cost {1} more.
    t.battlefield(P1, "Thalia, Guardian of Thraben");
    t.custom(
        P0,
        cost_reducer("Instant Discount", Filter::Type(CardType::Instant), 1),
        Zone::Battlefield,
    );
    t.custom(
        P0,
        cost_reducer("Instant Discount 2", Filter::Type(CardType::Instant), 1),
        Zone::Battlefield,
    );
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    // {R} + {1} - {1} - {1}: the generic part can't go below zero, so it costs {R}.
    t.cast(P0, bolt).target(P1).go();
    t.resolve();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn total_cost_is_locked_in_before_costs_are_paid() {
    cr!("601.2f", "601.2h");
    // CR 601.2h example: Altar's Reap with Thunderscape Familiar.
    let mut t = TestGame::new(2);
    let familiar = t.custom(
        P0,
        CB::new("Familiar Test")
            .creature(1, 1)
            .ability(stat(StaticEffect::CostModifier(CostModifier {
                applies_to: CostTarget::Spells(Filter::Color(Color::Black)),
                who: PlayerRel::You,
                change: CostChange::ReduceGeneric(Value::c(1)),
            })))
            .build(),
        Zone::Battlefield,
    );
    t.lands(P0, "Swamp", 1);
    let reap = t.hand(P0, "Altar's Reap");
    t.answer_choose(P0, &[Entity::Object(familiar)]);
    t.cast(P0, reap).go();
    // Paid {B} even though the Familiar was sacrificed during payment.
    assert!(!t.on_battlefield(familiar));
    t.resolve();
    assert_eq!(t.hand_size(P0), 2);
}

#[test]
fn mana_abilities_are_activated_before_costs_are_paid() {
    cr!("601.2g", "601.2h");
    let mut t = TestGame::new(2);
    let swamp = t.battlefield(P0, "Swamp");
    let elves = t.battlefield(P0, "Llanowar Elves");
    let reap = t.hand(P0, "Altar's Reap");
    // The Elves can produce mana for the spell and then be sacrificed as its cost.
    t.answer_choose(P0, &[Entity::Object(elves)]);
    t.cast(P0, reap).go();
    assert!(t.obj(swamp).tapped);
    assert!(!t.on_battlefield(elves));
    assert!(t.in_graveyard(P0, "Llanowar Elves"));
    assert!(t.player(P0).mana_pool.is_empty());
}

#[test]
fn partial_payments_are_not_allowed() {
    cr!("601.2", "601.2h");
    let mut t = TestGame::new(2);
    let swamps = t.lands(P0, "Swamp", 2);
    let reap = t.hand(P0, "Altar's Reap");
    // No creature to sacrifice: the whole casting is reversed, including mana payment.
    assert!(t.cast(P0, reap).try_go().is_err());
    assert!(t.in_hand(P0, "Altar's Reap"));
    assert!(swamps.iter().all(|s| !t.obj(*s).tapped));
    assert!(t.stack.is_empty());
}

#[test]
fn cast_triggers_trigger_after_casting_and_the_caster_keeps_priority() {
    cr!("601.2i");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 1);
    let watcher = CB::new("Cast Watcher")
        .enchantment()
        .ability(trig(
            TriggerCond::CastSpell {
                who: PlayerRel::You,
                filter: Filter::Any,
            },
            Body::effect(gain(1)),
        ))
        .build();
    t.custom(P0, watcher, Zone::Battlefield);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.g.turn.priority = Some(P0);
    t.g.take_action(
        P0,
        Action::Cast {
            card: bolt,
            method: CastMethod::Normal,
        },
    );
    assert_eq!(t.g.turn.priority, Some(P0));
    t.settle();
    // The cast trigger is above the spell.
    assert_eq!(t.stack.len(), 2);
    assert!(t.obj(t.stack[1]).is_stack_ability());
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
    assert_eq!(t.life(P1), 17);
}

#[test]
fn a_spell_can_be_cast_only_if_allowed_and_not_prohibited() {
    cr!("601.3");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 2);
    // Not allowed: no rule lets a player cast a card from their graveyard.
    let gy_bolt = t.graveyard(P0, "Lightning Bolt");
    assert!(t.cast(P0, gy_bolt).target(P1).try_go().is_err());
    // Prohibited: "Your opponents can't cast spells."
    let silence = CB::new("Silence Test")
        .enchantment()
        .ability(stat(StaticEffect::Restriction(Restriction::CantCast {
            who: PlayerFilter::Opponent,
            what: Filter::Any,
        })))
        .build();
    let sil = t.custom(P1, silence, Zone::Battlefield);
    let bolt = t.hand(P0, "Lightning Bolt");
    assert!(t.cast(P0, bolt).target(P1).try_go().is_err());
    assert!(!t
        .legal_actions(P0)
        .iter()
        .any(|a| matches!(a, Action::Cast { card, .. } if *card == bolt)));
    // Once the prohibition is gone the spell can be cast.
    let owner = t.obj(sil).owner;
    t.g.move_object(sil, Zone::Graveyard(owner), mtg_engine::events::MoveCause::Effect, None);
    t.cast(P0, bolt).target(P1).go();
}

#[test]
fn spells_already_on_the_stack_are_unaffected_by_cost_changes() {
    cr!("601.8");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 1);
    t.lands(P1, "Plains", 2);
    let bolt = t.hand(P0, "Lightning Bolt");
    let s = t.cast(P0, bolt).target(P1).go();
    // P1 responds with a flash spell whose permanent makes noncreature spells cost more.
    let taxer = CB::new("Flash Taxer")
        .creature(1, 1)
        .cost("{1}{W}")
        .keyword(mtg_engine::keywords::KeywordKind::Flash)
        .ability(stat(StaticEffect::CostModifier(CostModifier {
            applies_to: CostTarget::Spells(Filter::not(Filter::creature())),
            who: PlayerRel::Any,
            change: CostChange::IncreaseGeneric(Value::c(2)),
        })))
        .build();
    let tx = t.custom(P1, taxer, Zone::Hand(P1));
    t.cast(P1, tx).go();
    t.resolve(); // the taxer resolves
    assert_eq!(t.stack, vec![s]);
    t.resolve(); // the bolt resolves normally without further payment
    assert_eq!(t.life(P1), 17);
}

#[test]
fn casting_during_combat_uses_normal_timing() {
    // Sanity: sorcery-speed spells can't be cast during combat; instants can.
    cr!("601.3");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 2);
    t.lands(P0, "Mountain", 1);
    t.set_step(P0, Step::BeginningOfCombat);
    let bears = t.hand(P0, "Grizzly Bears");
    assert!(t.cast(P0, bears).try_go().is_err());
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
}
