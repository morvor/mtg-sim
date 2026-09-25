//! CR 602: activating activated abilities.

use super::r600_common::*;
use mtg_engine::ability::*;
use mtg_engine::decision::{Action, Answer, Decision};
use mtg_engine::events::MoveCause;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn an_activated_ability_is_a_cost_and_an_effect() {
    cr!("602.1", "602.1a");
    let mut t = TestGame::new(2);
    // Fountain of Youth: "{2}, {T}: You gain 1 life."
    let fountain = t.battlefield(P0, "Fountain of Youth");
    let AbilityKind::Activated(a) = &t.obj(fountain).chars.abilities[0].kind else {
        panic!("not an activated ability");
    };
    // The activation cost is everything before the colon: two mana and tapping it.
    assert_eq!(a.cost.mana.as_ref().unwrap().mana_value(), 2);
    assert!(a.cost.has_tap());
    assert!(matches!(a.body.effect, Effect::GainLife { .. }));
    let lands = t.lands(P0, "Plains", 2);
    t.activate(P0, fountain, 0, &[]).unwrap();
    assert!(t.obj(fountain).tapped);
    assert!(lands.iter().all(|l| t.obj(*l).tapped));
    t.resolve();
    assert_eq!(t.life(P0), 21);
}

#[test]
fn the_activation_cost_is_paid_by_the_player_activating_the_ability() {
    cr!("602.1a", "602.1b", "602.2");
    let mut t = TestGame::new(2);
    // "{1}: You gain 2 life. Any player may activate this ability."
    let mut a = ActivatedAbility::new(mana_cost("{1}"), Body::effect(gain(2)));
    a.any_player = true;
    let shrine = t.custom(
        P0,
        CB::new("Public Shrine")
            .artifact()
            .ability(act_from(a))
            .build(),
        Zone::Battlefield,
    );
    let p0_land = t.battlefield(P0, "Plains");
    let p1_land = t.battlefield(P1, "Plains");
    t.activate(P1, shrine, 0, &[]).unwrap();
    // P1 activated it, so P1 paid and P1 controls the ability.
    assert!(t.obj(p1_land).tapped);
    assert!(!t.obj(p0_land).tapped);
    let top = *t.stack.last().unwrap();
    assert_eq!(t.obj(top).controller, P1);
    t.resolve();
    assert_eq!(t.life(P1), 22);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn only_the_controller_can_activate_an_objects_abilities_unless_it_says_otherwise() {
    cr!("602.2");
    let mut t = TestGame::new(2);
    let pyro = t.battlefield(P0, "Prodigal Pyromancer");
    assert!(t.activate(P1, pyro, 0, &[Entity::Player(P0)]).is_err());
    assert!(t.stack.is_empty());
    t.activate(P0, pyro, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve();
    assert_eq!(t.life(P1), 19);
}

#[test]
fn activation_instructions_are_not_part_of_the_effect() {
    cr!("602.1b", "602.5d");
    // "{1}: Draw a card. Activate only as a sorcery."
    let def = compile_def(
        "Sorcery Lamp",
        "Artifact",
        "{2}",
        "{1}: Draw a card. Activate only as a sorcery.",
    );
    let AbilityKind::Activated(a) = &abilities(&def)[0].kind else {
        panic!("not activated");
    };
    assert_eq!(a.timing, ActivationTiming::Sorcery);
    assert!(matches!(a.body.effect, Effect::Draw { .. }));
    let mut t = TestGame::new(2);
    let lamp = t.custom(P0, def, Zone::Battlefield);
    t.lands(P0, "Island", 3);
    // Sorcery timing: not during combat, not on an opponent's turn, not with a non-empty stack.
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(t.activate(P0, lamp, 0, &[]).is_err());
    t.set_step(P1, Step::PrecombatMain);
    t.g.turn.priority = Some(P0);
    assert!(t.activate(P0, lamp, 0, &[]).is_err());
    t.set_step(P0, Step::PrecombatMain);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.lands(P0, "Mountain", 1);
    t.cast(P0, bolt).target(P1).go();
    assert!(t.activate(P0, lamp, 0, &[]).is_err());
    t.resolve();
    // Main phase, own turn, empty stack: OK. The player needn't have a sorcery to cast.
    t.activate(P0, lamp, 0, &[]).unwrap();
}

#[test]
fn activate_only_as_an_instant_means_instant_timing() {
    cr!("602.5e", "605.1a", "605.5");
    // Chromatic Sphere: "{1}, {T}, Sacrifice this artifact: Add one mana of any color.
    // Draw a card. (Activate only as an instant.)" — it draws, so it isn't a mana ability.
    let mut t = TestGame::new(2);
    let sphere = t.battlefield(P0, "Chromatic Sphere");
    assert!(!t.obj(sphere).chars.abilities[0].is_mana_ability());
    t.lands(P0, "Mountain", 2);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.set_step(P1, Step::End);
    t.g.turn.priority = Some(P0);
    t.cast(P0, bolt).target(P1).go();
    // Any time the player has priority, even with a spell on the stack on another turn.
    let hand = t.hand_size(P0);
    t.activate(P0, sphere, 0, &[]).unwrap();
    assert_eq!(t.stack.len(), 2);
    t.resolve();
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn only_activated_abilities_can_be_activated() {
    cr!("602.1c", "603.2a");
    let mut t = TestGame::new(2);
    let both = CB::new("Two Abilities")
        .creature(2, 2)
        .ability(act(mana_cost("{0}"), Body::effect(gain(1))))
        .ability(trig(
            TriggerCond::CastSpell {
                who: PlayerRel::Any,
                filter: Filter::Any,
            },
            Body::effect(gain(2)),
        ))
        .build();
    let obj = t.custom(P0, both, Zone::Battlefield);
    let trig_uid = t.obj(obj).chars.abilities[1].uid;
    t.g.turn.priority = Some(P0);
    assert!(t.g.activate_ability(P0, obj, trig_uid).is_err());
    // An effect that stops activating abilities doesn't stop triggered abilities.
    let needle = CB::new("Needle Test")
        .artifact()
        .ability(stat(StaticEffect::Restriction(Restriction::CantActivate {
            who: PlayerFilter::Any,
            sources: Filter::Named("Two Abilities".into()),
            include_mana: false,
        })))
        .build();
    t.custom(P1, needle, Zone::Battlefield);
    assert!(t.activate(P0, obj, 0, &[]).is_err());
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.g.turn.priority = Some(P1);
    t.cast(P1, bolt).target(P0).go();
    t.settle();
    assert_eq!(t.stack.len(), 2);
    t.resolve_all();
    assert_eq!(t.life(P0), 19);
}

#[test]
fn the_ability_on_the_stack_is_not_a_card_and_has_only_its_text() {
    cr!("602.2a");
    let mut t = TestGame::new(2);
    let pyro = t.battlefield(P0, "Prodigal Pyromancer");
    let bolt = t.hand(P0, "Lightning Bolt");
    t.lands(P0, "Mountain", 1);
    let s = t.cast(P0, bolt).target(P1).go();
    let ab = t
        .activate(P0, pyro, 0, &[Entity::Player(P1)])
        .unwrap()
        .unwrap();
    // It becomes the topmost object, controlled by the activating player.
    assert_eq!(t.stack, vec![s, ab]);
    let o = t.obj(ab);
    assert_eq!(o.kind, ObjKind::StackAbility);
    assert!(!o.is_card());
    assert_eq!(o.controller, P0);
    assert!(o.chars.card_types.is_empty());
    assert!(o.chars.colors.is_colorless());
    assert!(o.chars.mana_cost.is_none());
    assert!(o.chars.rules_text.contains("deals 1 damage"));
    t.resolve();
    assert_eq!(t.stack, vec![s]);
    assert_eq!(t.obj(ab).zone, Zone::Nowhere);
}

#[test]
fn activating_from_a_hidden_zone_reveals_the_card() {
    cr!("602.2a");
    let mut t = TestGame::new(2);
    // A card with cycling in hand.
    let cycler = CB::new("Cycler Test")
        .creature(1, 1)
        .ability(AbilityDef::new(
            AbilityKind::Keyword(mtg_engine::keywords::Keyword::with_cost(
                KeywordKind::Cycling,
                mana_cost("{0}"),
            )),
            "Cycling {0}",
        ))
        .build();
    let c = t.custom(P0, cycler, Zone::Hand(P0));
    t.g.recompute();
    let uid = activated_uid(&t, c, 0);
    t.g.turn.priority = Some(P0);
    t.g.activate_ability(P0, c, uid).unwrap();
    assert!(t.g.turn_events.iter().chain(t.g.events.iter()).any(|e| matches!(
        e,
        mtg_engine::events::Event::Custom { name, obj, .. } if name == "revealed" && *obj == Some(c)
    )));
}

#[test]
fn activation_follows_the_casting_steps() {
    cr!("602.2b");
    let mut t = TestGame::new(2);
    // "{X}, {T}: This creature deals X damage to any target." and a static that makes
    // activated abilities cost {1} more: the total cost is determined as for spells.
    let mut a = ActivatedAbility::new(
        Cost {
            mana: Some(mana("{X}")),
            parts: vec![CostPart::Tap],
        },
        Body::simple(
            vec![TargetSpec::any_target()],
            Effect::DealDamage {
                source: Sel::This,
                amount: Value::X,
                to: Sel::Target(0),
            },
        ),
    );
    a.timing = ActivationTiming::Instant;
    let blaster = t.custom(
        P0,
        CB::new("X Blaster")
            .creature(1, 1)
            .ability(act_from(a))
            .build(),
        Zone::Battlefield,
    );
    t.custom(
        P1,
        CB::new("Ability Tax")
            .enchantment()
            .ability(stat(StaticEffect::CostModifier(CostModifier {
                applies_to: CostTarget::Abilities(Filter::creature()),
                who: PlayerRel::Any,
                change: CostChange::IncreaseGeneric(Value::c(1)),
            })))
            .build(),
        Zone::Battlefield,
    );
    let watcher = CB::new("Activation Watcher")
        .enchantment()
        .ability(trig(
            TriggerCond::AbilityActivated {
                who: PlayerRel::Any,
                source: Filter::Any,
                include_mana: false,
            },
            Body::effect(gain(1)),
        ))
        .build();
    t.custom(P1, watcher, Zone::Battlefield);
    let lands = t.lands(P0, "Mountain", 3);
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.activate(P0, blaster, 0, &[Entity::Player(P1)]).unwrap();
    // X=2 plus {1} more, paid with mana abilities activated during payment.
    assert!(lands.iter().all(|l| t.obj(*l).tapped));
    // The "becomes activated" trigger goes on the stack above it.
    t.settle();
    assert_eq!(t.stack.len(), 2);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
}

#[test]
fn an_opponent_chooses_targets_for_an_ability_when_the_ability_says_so() {
    cr!("602.3", "602.3a");
    let mut t = TestGame::new(3);
    // Echo Chamber-like: "{0}: An opponent chooses target creature they control. Tap it."
    let a = ActivatedAbility::new(
        mana_cost("{0}"),
        Body::simple(
            vec![TargetSpec {
                chosen_by_opponent: true,
                ..TargetSpec::object(Filter::creature(), "target creature they control")
            }],
            Effect::Tap {
                what: Sel::Target(0),
            },
        ),
    );
    let chamber = t.custom(
        P0,
        CB::new("Chamber Test")
            .artifact()
            .ability(act_from(a))
            .build(),
        Zone::Battlefield,
    );
    let p2_bear = t.battlefield(P2, "Grizzly Bears");
    t.battlefield(P1, "Hill Giant");
    // P0 decides that P2 makes the choice; P2 chooses its own creature.
    t.answer(
        P0,
        DecisionKind::Entities,
        Answer::Entities(vec![Entity::Player(P2)]),
    );
    t.answer_targets(P2, &[Entity::Object(p2_bear)]);
    t.g.turn.priority = Some(P0);
    let uid = activated_uid(&t, chamber, 0);
    t.g.activate_ability(P0, chamber, uid).unwrap();
    assert!(t
        .asked()
        .iter()
        .any(|(p, d)| *p == P2 && matches!(d, Decision::ChooseTargets { .. })));
    t.resolve();
    assert!(t.obj(p2_bear).tapped);
}

#[test]
fn the_abilitys_controller_chooses_before_the_other_player() {
    cr!("602.3b");
    let mut t = TestGame::new(2);
    let a = ActivatedAbility::new(
        mana_cost("{0}"),
        Body::simple(
            vec![
                TargetSpec {
                    chosen_by_opponent: true,
                    ..TargetSpec::object(Filter::creature(), "opponent's pick")
                },
                target_creature(),
            ],
            Effect::Tap {
                what: Sel::AllTargets,
            },
        ),
    );
    let src = t.custom(
        P0,
        CB::new("Both Pick").artifact().ability(act_from(a)).build(),
        Zone::Battlefield,
    );
    let x = t.battlefield(P0, "Grizzly Bears");
    let y = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[Entity::Object(y)]);
    t.answer_targets(P1, &[Entity::Object(x)]);
    t.activate(P0, src, 0, &[]).unwrap();
    let order: Vec<PlayerId> = t
        .asked()
        .iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseTargets { .. }))
        .map(|(p, _)| *p)
        .collect();
    assert_eq!(order, vec![P0, P1]);
}

#[test]
fn activating_an_ability_that_alters_costs_doesnt_affect_the_stack() {
    cr!("602.4");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    let s = t.cast(P0, bolt).target(P1).go();
    // P1 activates an ability whose effect makes spells cost {2} more.
    let tax_token = TokenSpec {
        name: "Tax Token".into(),
        colors: ColorSet::NONE,
        supertypes: vec![],
        card_types: vec![CardType::Artifact],
        subtypes: vec![],
        power: None,
        toughness: None,
        abilities: vec![stat(StaticEffect::CostModifier(CostModifier {
            applies_to: CostTarget::Spells(Filter::Any),
            who: PlayerRel::Any,
            change: CostChange::IncreaseGeneric(Value::c(2)),
        }))],
        scryfall_name: None,
    };
    let taxer = t.custom(
        P1,
        CB::new("Taxer")
            .artifact()
            .ability(act(
                mana_cost("{0}"),
                Body::effect(Effect::CreateToken {
                    spec: tax_token,
                    count: Value::c(1),
                    controller: PlayerRef::You,
                    tapped: false,
                    attacking: false,
                }),
            ))
            .build(),
        Zone::Battlefield,
    );
    t.activate(P1, taxer, 0, &[]).unwrap();
    t.resolve();
    assert_eq!(t.named_on_battlefield("Tax Token").len(), 1);
    // The Bolt already on the stack resolves without any further payment.
    assert_eq!(t.stack, vec![s]);
    t.resolve();
    assert_eq!(t.life(P1), 17);
    // New spells do cost more.
    t.lands(P0, "Mountain", 1);
    let shock = t.hand(P0, "Shock");
    assert!(t.cast(P0, shock).target(P1).try_go().is_err());
}

#[test]
fn a_prohibited_ability_cant_be_activated() {
    cr!("602.5");
    let mut t = TestGame::new(2);
    let pyro = t.battlefield(P0, "Prodigal Pyromancer");
    t.custom(
        P1,
        CB::new("Pithing Test")
            .artifact()
            .ability(stat(StaticEffect::Restriction(Restriction::CantActivate {
                who: PlayerFilter::Any,
                sources: Filter::Named("Prodigal Pyromancer".into()),
                include_mana: false,
            })))
            .build(),
        Zone::Battlefield,
    );
    assert!(t.activate(P0, pyro, 0, &[Entity::Player(P1)]).is_err());
    assert!(!t
        .legal_actions(P0)
        .iter()
        .any(|a| matches!(a, Action::Activate { source, .. } if *source == pyro)));
}

#[test]
fn tap_abilities_of_creatures_need_control_since_the_most_recent_turn_began() {
    cr!("602.5a");
    let mut t = TestGame::new(2);
    let sick = t.battlefield_sick(P0, "Prodigal Pyromancer");
    assert!(t.activate(P0, sick, 0, &[Entity::Player(P1)]).is_err());
    // A noncreature artifact's {T} ability can be activated right away.
    let stone = t.battlefield_sick(P0, "Mind Stone");
    t.activate(P0, stone, 0, &[]).unwrap();
    assert_eq!(t.player(P0).mana_pool.total(), 1);
    // A creature with haste ignores the rule.
    let hasty = t.battlefield_sick(P0, "Prodigal Pyromancer");
    t.g.effects.push(mtg_engine::game::ContinuousEffect {
        id: 999,
        source: None,
        controller: P0,
        timestamp: t.g.next_timestamp,
        duration: Duration::EndOfTurn,
        affected: mtg_engine::game::Affected::Objects(vec![hasty]),
        mods: vec![Modification::AddKeyword(
            mtg_engine::keywords::Keyword::new(KeywordKind::Haste),
        )],
        layer1: None,
        created_turn: 1,
    });
    t.g.recompute();
    t.activate(P0, hasty, 0, &[Entity::Player(P1)]).unwrap();
    // {Q} abilities are covered as well.
    let untapper = CB::new("Untapper")
        .creature(1, 1)
        .ability(act(
            Cost {
                mana: None,
                parts: vec![CostPart::Untap],
            },
            Body::effect(gain(1)),
        ))
        .build();
    let u = t.custom(P0, untapper, Zone::Battlefield);
    t.g.objects[u.0 as usize].tapped = true;
    t.g.objects[u.0 as usize].summoning_sick = true;
    assert!(t.activate(P0, u, 0, &[]).is_err());
    // Next turn the creatures have been under P0's control since the turn began.
    t.advance_to(P0, Step::Upkeep);
    t.clear_answers();
    t.advance_to(P0, Step::PrecombatMain);
    let sick_now = t.g.current(sick);
    assert!(t.activate(P0, sick_now, 0, &[Entity::Player(P1)]).is_ok());
}

fn once_each_turn_pumper() -> CardDef {
    let mut a = ActivatedAbility::new(
        mana_cost("{0}"),
        Body::effect(Effect::Modify {
            what: Sel::This,
            mods: vec![Modification::ModifyPT(Value::c(1), Value::c(0))],
            duration: Duration::EndOfTurn,
        }),
    );
    a.max_per_turn = Some(1);
    CB::new("Once Pumper")
        .creature(1, 1)
        .ability(act_from(a))
        .build()
}

#[test]
fn activation_restrictions_follow_the_object_through_control_changes() {
    cr!("602.5b");
    let mut t = TestGame::new(2);
    let p = t.custom(P0, once_each_turn_pumper(), Zone::Battlefield);
    t.activate(P0, p, 0, &[]).unwrap();
    t.resolve();
    assert!(t.activate(P0, p, 0, &[]).is_err());
    // P1 gains control of it this turn: the restriction still applies.
    let steal = CB::new("Steal Test")
        .sorcery()
        .cost("{0}")
        .spell(Body::simple(
            vec![target_creature()],
            Effect::GainControl {
                what: Sel::Target(0),
                who: PlayerRef::You,
                duration: Duration::EndOfTurn,
            },
        ))
        .build();
    let s = t.custom(P1, steal, Zone::Hand(P1));
    t.set_step(P1, Step::PrecombatMain);
    t.cast(P1, s).target(p).go();
    t.resolve();
    assert_eq!(t.obj(p).controller, P1);
    assert!(t.activate(P1, p, 0, &[]).is_err());
}

#[test]
fn restrictions_on_acquired_abilities_apply_per_granting_object() {
    cr!("602.5c");
    let mut t = TestGame::new(2);
    // "Creatures you control have '{0}: You gain 1 life. Activate only once each turn.'"
    let mut granted = ActivatedAbility::new(mana_cost("{0}"), Body::effect(gain(1)));
    granted.max_per_turn = Some(1);
    let granted = act_from(granted);
    let banner = CB::new("Life Banner")
        .enchantment()
        .ability(stat(StaticEffect::Continuous {
            affected: Filter::creature().you_control(),
            mods: vec![Modification::AddAbility(granted)],
        }))
        .build();
    let banner = std::sync::Arc::new(banner);
    for _ in 0..2 {
        let b =
            t.g.create_card_object(banner.clone(), P0, Zone::Battlefield);
        t.g.battlefield.push(b);
    }
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.recompute();
    // Two identically worded abilities, acquired from two different objects.
    assert_eq!(t.obj(bears).chars.abilities.len(), 2);
    t.activate(P0, bears, 0, &[]).unwrap();
    t.resolve();
    assert!(t.activate(P0, bears, 0, &[]).is_err());
    t.activate(P0, bears, 1, &[]).unwrap();
    t.resolve();
    assert!(t.activate(P0, bears, 1, &[]).is_err());
    assert_eq!(t.life(P0), 22);
}
