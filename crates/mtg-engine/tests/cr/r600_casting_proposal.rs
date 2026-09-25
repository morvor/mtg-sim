//! CR 601.2c–e, 601.3–601.7: proposing a spell, permissions and prohibitions, and choices
//! made by opponents.

use super::r600_common::*;
use mtg_engine::ability::*;
use mtg_engine::decision::{Action, Answer, Decision};
use mtg_engine::keywords::{Keyword, KeywordKind};
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn kicker(cost: &str) -> Ability {
    AbilityDef::new(
        AbilityKind::Keyword(Keyword::with_cost(KeywordKind::Kicker, mana_cost(cost))),
        format!("Kicker {cost}"),
    )
}

/// "Kicker {1}. Deal 4 damage to target creature. If this spell was kicked, it also deals
/// 4 damage to target player." (Goblin Barrage-like.)
fn barrage() -> CardDef {
    CB::new("Barrage Test")
        .sorcery()
        .cost("{R}")
        .ability(kicker("{1}"))
        .spell(Body::simple(
            vec![
                target_creature(),
                TargetSpec {
                    condition: Some(Condition::CostPaid("kicker".into())),
                    ..TargetSpec::player(PlayerFilter::Any, "target player")
                },
            ],
            Effect::Seq(vec![
                Effect::DealDamage {
                    source: Sel::This,
                    amount: Value::c(4),
                    to: Sel::Target(0),
                },
                Effect::DealDamage {
                    source: Sel::This,
                    amount: Value::c(4),
                    to: Sel::Target(1),
                },
            ]),
        ))
        .build()
}

#[test]
fn a_target_required_only_if_kicked_is_chosen_only_if_kicked() {
    cr!("601.2c");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 4);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    // Not kicked: the spell requires only the creature target.
    let c = t.custom(P0, barrage(), Zone::Hand(P0));
    let s = t.cast(P0, c).kicked(false).target(bears).go();
    let chosen = t.obj(s).stack.as_ref().unwrap().chosen[0].targets.clone();
    assert_eq!(chosen, vec![vec![Entity::Object(bears)], vec![]]);
    t.resolve();
    assert_eq!(t.life(P1), 20);
    // Kicked: it also requires the player target.
    let c2 = t.custom(P0, barrage(), Zone::Hand(P0));
    t.cast(P0, c2).kicked(true).target(giant).target(P1).go();
    t.resolve();
    assert!(!t.on_battlefield(giant));
    assert_eq!(t.life(P1), 16);
}

fn flagbearer() -> CardDef {
    CB::new("Flagbearer Test")
        .creature(1, 1)
        .subtypes(&["Human", "Flagbearer"])
        .ability(stat(StaticEffect::Restriction(Restriction::MustTarget {
            chooser: PlayerFilter::Opponent,
            what: Filter::Subtype("Flagbearer".into()),
        })))
        .build()
}

#[test]
fn targets_must_obey_effects_that_say_an_object_must_be_chosen() {
    cr!("601.2c", "602.2b");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 2);
    let fb = t.custom(P1, flagbearer(), Zone::Battlefield);
    let bolt = t.hand(P0, "Lightning Bolt");
    // P0 wants to target P1, but must choose a Flagbearer if able.
    let s = t.cast(P0, bolt).target(P1).go();
    let chosen = t.obj(s).stack.as_ref().unwrap().chosen[0].targets.clone();
    assert_eq!(chosen, vec![vec![Entity::Object(fb)]]);
    t.resolve();
    assert!(!t.on_battlefield(fb));
    assert_eq!(t.life(P1), 20);
    // The same applies to activated abilities.
    let fb2 = t.custom(P1, flagbearer(), Zone::Battlefield);
    let pyro = t.battlefield(P0, "Prodigal Pyromancer");
    t.activate(P0, pyro, 0, &[Entity::Player(P1)]).unwrap();
    let top = *t.stack.last().unwrap();
    assert_eq!(
        t.obj(top).stack.as_ref().unwrap().chosen[0].targets,
        vec![vec![Entity::Object(fb2)]]
    );
}

#[test]
fn must_target_effects_dont_apply_to_triggered_abilities_or_impossible_choices() {
    cr!("601.2c");
    let mut t = TestGame::new(2);
    t.custom(P1, flagbearer(), Zone::Battlefield);
    t.lands(P0, "Island", 1);
    // "Target player draws a card" can't target a Flagbearer, so it is unaffected.
    let draw_spell = CB::new("Draw Target")
        .instant()
        .cost("{U}")
        .spell(Body::simple(
            vec![TargetSpec::player(PlayerFilter::Any, "target player")],
            Effect::Draw {
                who: PlayerRef::Target(0),
                n: Value::c(1),
            },
        ))
        .build();
    let d = t.custom(P0, draw_spell, Zone::Hand(P0));
    let s = t.cast(P0, d).target(P0).go();
    assert_eq!(
        t.obj(s).stack.as_ref().unwrap().chosen[0].targets,
        vec![vec![Entity::Player(P0)]]
    );
    t.resolve();
    // A triggered ability isn't cast or activated.
    let pinger = CB::new("ETB Pinger")
        .creature(1, 1)
        .ability(trig(
            TriggerCond::EntersBattlefield(Filter::Source),
            damage_target(1),
        ))
        .build();
    t.answer_targets(P0, &[Entity::Player(P1)]);
    let def = std::sync::Arc::new(pinger);
    let p = t.g.create_card_object(def, P0, Zone::Nowhere);
    t.g.move_object(
        p,
        Zone::Battlefield,
        mtg_engine::events::MoveCause::Effect,
        Some(P0),
    );
    t.settle();
    let top = *t.stack.last().unwrap();
    assert_eq!(
        t.obj(top).stack.as_ref().unwrap().chosen[0].targets,
        vec![vec![Entity::Player(P1)]]
    );
}

fn even_mana_values() -> Filter {
    Filter::Or(
        (0..=10)
            .map(|n| Filter::ManaValue(Cmp::Eq, Box::new(Value::c(2 * n))))
            .collect(),
    )
}

fn winnower() -> CardDef {
    // "Your opponents can't cast spells with even mana values."
    CB::new("Winnower Test")
        .creature(11, 9)
        .ability(stat(StaticEffect::Restriction(Restriction::CantCast {
            who: PlayerFilter::Opponent,
            what: even_mana_values(),
        })))
        .build()
}

#[test]
fn a_prohibited_spell_may_be_begun_if_proposal_choices_could_change_that() {
    cr!("601.2e", "601.3a", "601.6");
    let mut t = TestGame::new(2);
    t.custom(P1, winnower(), Zone::Battlefield);
    let mountains = t.lands(P0, "Mountain", 4);
    // Grizzly Bears (mana value 2) can't be begun at all.
    let bears = t.hand(P0, "Grizzly Bears");
    assert!(t.cast(P0, bears).try_go().is_err());
    // Rolling Thunder ({X}{R}{R}) may be begun because X could make its mana value odd.
    let thunder = t.hand(P0, "Rolling Thunder");
    assert!(t
        .legal_actions(P0)
        .iter()
        .any(|a| matches!(a, Action::Cast { card, .. } if *card == thunder)));
    // X = 2 makes the proposed spell's mana value 4: the casting is illegal and reversed.
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![2]));
    assert!(t.cast(P0, thunder).x(2).target(P1).try_go().is_err());
    assert!(t.in_hand(P0, "Rolling Thunder"));
    assert!(mountains.iter().all(|m| !t.obj(*m).tapped));
    // X = 1 (mana value 3) is fine.
    t.clear_answers();
    t.cast(P0, thunder).x(1).target(P1).go();
    t.resolve();
    assert_eq!(t.life(P1), 19);
}

#[test]
fn prohibitions_that_begin_while_paying_costs_dont_matter() {
    cr!("601.6");
    let mut t = TestGame::new(2);
    // "{T}: Add {R}. Players can't cast spells this turn." (a mana ability)
    let mut mana = ActivatedAbility::new(
        Cost::tap(),
        Body::effect(Effect::Seq(vec![
            Effect::AddMana {
                who: PlayerRef::You,
                mana: ManaProduction::Fixed(vec![mtg_engine::mana::ManaType::R]),
                restriction: None,
            },
            Effect::AddRestriction {
                restriction: Restriction::CantCast {
                    who: PlayerFilter::Any,
                    what: Filter::Any,
                },
                duration: Duration::EndOfTurn,
            },
        ])),
    );
    mana.is_mana_ability = true;
    let land = CB::new("Silencing Land")
        .land()
        .ability(act_from(mana))
        .build();
    t.custom(P0, land, Zone::Battlefield);
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    let shock = t.hand(P0, "Shock");
    // Paying for the Bolt with the Silencing Land prohibits casting, but only after the
    // proposal is complete; the Bolt is still cast.
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    let s = t.cast(P0, bolt).target(P1).go();
    assert_eq!(t.stack, vec![s]);
    // Any time after it's been cast, the prohibition doesn't affect it.
    t.resolve();
    assert_eq!(t.life(P1), 17);
    // But no further spells can be cast this turn.
    assert!(t.cast(P0, shock).target(P1).try_go().is_err());
}

#[test]
fn costs_with_random_elements_are_paid_after_other_costs() {
    cr!("601.2h");
    let mut t = TestGame::new(2);
    // "As an additional cost, discard a card at random and discard a card."
    let spell = CB::new("Double Discard")
        .sorcery()
        .cost("{0}")
        .ability(stat_from(StaticAbility {
            zone: FunctionZone::Anywhere,
            ..StaticAbility::new(StaticEffect::CostModifier(CostModifier {
                applies_to: CostTarget::ThisSpell,
                who: PlayerRel::You,
                change: CostChange::AdditionalCost(Cost {
                    mana: None,
                    parts: vec![
                        CostPart::Discard {
                            filter: Filter::Any,
                            count: Value::c(1),
                            random: true,
                        },
                        CostPart::Discard {
                            filter: Filter::Any,
                            count: Value::c(1),
                            random: false,
                        },
                    ],
                }),
            }))
        }))
        .spell(Body::effect(gain(1)))
        .build();
    let c = t.custom(P0, spell, Zone::Hand(P0));
    let a = t.hand(P0, "Grizzly Bears");
    t.hand(P0, "Hill Giant");
    t.hand(P0, "Shock");
    t.answer_choose(P0, &[Entity::Object(a)]);
    t.cast(P0, c).go();
    // The chosen discard happened while all three cards were still in hand.
    let asked = t.asked();
    let d = asked
        .iter()
        .find_map(|(_, d)| match d {
            Decision::ChooseEntities {
                prompt, candidates, ..
            } if prompt.contains("discard") => Some(candidates.clone()),
            _ => None,
        })
        .unwrap();
    assert_eq!(d.len(), 3);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(t.hand_size(P0), 1);
}

fn sorcery_flash_enabler() -> CardDef {
    // "You may cast sorcery spells as though they had flash."
    CB::new("Sorcery Flash")
        .enchantment()
        .ability(stat(StaticEffect::FlashPermission {
            who: PlayerRel::You,
            what: Filter::Type(CardType::Sorcery),
        }))
        .build()
}

#[test]
fn flash_permissions_consider_the_qualities_chosen_in_the_proposal() {
    cr!("601.3b", "601.3e");
    let mut t = TestGame::new(2);
    t.custom(P0, sorcery_flash_enabler(), Zone::Battlefield);
    t.lands(P0, "Forest", 6);
    t.library_top(P0, "Forest");
    let giant = t.hand(P0, "Beanstalk Giant");
    t.set_step(P1, Step::End);
    t.g.turn.priority = Some(P0);
    // Casting the creature isn't allowed at instant speed...
    assert!(t.cast(P0, giant).try_go().is_err());
    // ... but its sorcery Adventure is, because choosing it makes the spell a sorcery.
    let s = t.cast(P0, giant).method(CastMethod::Half(1)).go();
    assert!(t.obj(s).chars.is(CardType::Sorcery));
    t.resolve();
    assert_eq!(
        t.permanents().filter(|o| o.chars.name == "Forest").count(),
        7
    );
}

fn top_of_library_permission(name: &str, what: Filter) -> CardDef {
    CB::new(name)
        .enchantment()
        .ability(stat(StaticEffect::PlayPermission(PlayPermission {
            who: PlayerRel::You,
            zone: ZoneKind::Library,
            top_only: true,
            what,
            lands: false,
            spells: true,
            cost: None,
        })))
        .build()
}

#[test]
fn permissions_check_the_alternative_characteristics_a_card_is_cast_with() {
    cr!("601.3e", "601.3");
    // CR 601.3e example: Melek with Giant Killer on top of the library.
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 4);
    t.custom(
        P0,
        top_of_library_permission(
            "Melek Test",
            Filter::Or(vec![
                Filter::Type(CardType::Instant),
                Filter::Type(CardType::Sorcery),
            ]),
        ),
        Zone::Battlefield,
    );
    let big = t.battlefield(P1, "Craw Wurm");
    let killer = t.library_top(P0, "Giant Killer");
    assert!(t.cast(P0, killer).try_go().is_err());
    t.cast(P0, killer)
        .method(CastMethod::Half(1))
        .target(big)
        .go();
    t.resolve();
    assert!(!t.on_battlefield(big));

    // With Garruk's Horde instead, Giant Killer may be cast but Chop Down may not.
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 4);
    t.custom(
        P0,
        top_of_library_permission("Horde Test", Filter::creature()),
        Zone::Battlefield,
    );
    let big = t.battlefield(P1, "Craw Wurm");
    let killer = t.library_top(P0, "Giant Killer");
    assert!(t
        .cast(P0, killer)
        .method(CastMethod::Half(1))
        .target(big)
        .try_go()
        .is_err());
    t.cast(P0, killer).go();
    t.resolve();
    assert_eq!(t.named_on_battlefield("Giant Killer").len(), 1);
}

#[test]
fn face_down_exiled_cards_can_be_cast_only_by_a_player_who_can_look_at_them() {
    cr!("601.3f");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 4);
    // "You may cast spells from among cards in exile."
    t.custom(
        P0,
        CB::new("Exile Caster")
            .enchantment()
            .ability(stat(StaticEffect::PlayPermission(PlayPermission {
                who: PlayerRel::You,
                zone: ZoneKind::Exile,
                top_only: false,
                what: Filter::Any,
                lands: false,
                spells: true,
                cost: None,
            })))
            .build(),
        Zone::Battlefield,
    );
    let up = t.exile(P0, "Grizzly Bears");
    let down = t.exile(P0, "Grizzly Bears");
    t.g.objects[down.0 as usize].face_down = true;
    t.g.recompute();
    // Nobody may look at the face-down card, so it can't be cast.
    assert!(t.cast(P0, down).try_go().is_err());
    t.cast(P0, up).go();
    t.resolve();
    // An effect that lets its player look at and cast the card makes it castable.
    t.g.play_grants.push(mtg_engine::casting::PlayGrant {
        player: P0,
        object: down,
        duration: Duration::EndOfTurn,
        free: false,
        source: None,
        turn: 1,
    });
    t.cast(P0, down).go();
    t.resolve();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 2);
}

#[test]
fn a_flash_permission_that_requires_an_additional_cost() {
    cr!("601.3c");
    let mut t = TestGame::new(2);
    // "You may cast this spell as though it had flash if you pay {2} more to cast it.
    //  Destroy all creatures."
    let rout = CB::new("Rout Test")
        .sorcery()
        .cost("{3}{W}{W}")
        .ability(stat_from(StaticAbility {
            zone: FunctionZone::Anywhere,
            ..StaticAbility::new(StaticEffect::CostModifier(CostModifier {
                applies_to: CostTarget::ThisSpell,
                who: PlayerRel::You,
                change: CostChange::FlashForAdditionalCost(mana_cost("{2}")),
            }))
        }))
        .spell(Body::effect(Effect::Destroy {
            what: Sel::All(Filter::creature()),
            no_regen: true,
        }))
        .build();
    let r = t.custom(P0, rout, Zone::Hand(P0));
    let bears = t.battlefield(P1, "Grizzly Bears");
    let plains = t.lands(P0, "Plains", 7);
    t.set_step(P1, Step::End);
    t.g.turn.priority = Some(P0);
    // Normally it can't be cast at instant speed.
    assert!(t.cast(P0, r).try_go().is_err());
    let uid = t.obj(r).chars.abilities[0].uid;
    t.cast(P0, r).method(CastMethod::Alternative(uid)).go();
    // It cost {2} more.
    assert!(plains.iter().all(|p| t.obj(*p).tapped));
    t.resolve();
    assert!(!t.on_battlefield(bears));
}

/// A creature that has flash as long as you control an artifact, and whose additional
/// cost is sacrificing an artifact.
fn conditional_flash_creature() -> CardDef {
    CB::new("Conditional Flasher")
        .creature(2, 2)
        .cost("{1}")
        .ability(stat_from(StaticAbility {
            condition: Some(Condition::Exists(
                Filter::Type(CardType::Artifact).you_control(),
            )),
            zone: FunctionZone::Anywhere,
            ..StaticAbility::new(StaticEffect::Continuous {
                affected: Filter::Source,
                mods: vec![Modification::AddKeyword(Keyword::new(KeywordKind::Flash))],
            })
        }))
        .ability(stat_from(StaticAbility {
            zone: FunctionZone::Anywhere,
            ..StaticAbility::new(StaticEffect::CostModifier(CostModifier {
                applies_to: CostTarget::ThisSpell,
                who: PlayerRel::You,
                change: CostChange::AdditionalCost(Cost {
                    mana: None,
                    parts: vec![CostPart::Sacrifice {
                        filter: Filter::Type(CardType::Artifact).you_control(),
                        count: Value::c(1),
                    }],
                }),
            }))
        }))
        .build()
}

#[test]
fn a_spell_with_conditional_flash_can_be_cast_as_though_it_had_flash_while_the_condition_holds() {
    cr!("601.3d", "601.6a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 1);
    let c = t.custom(P0, conditional_flash_creature(), Zone::Hand(P0));
    t.set_step(P1, Step::End);
    t.g.turn.priority = Some(P0);
    // No artifact: no flash (and the additional cost couldn't be paid anyway).
    assert!(!t.obj(c).has_keyword(KeywordKind::Flash));
    assert!(t.cast(P0, c).try_go().is_err());
    let stone = t.battlefield(P0, "Mind Stone");
    t.g.recompute();
    assert!(t.obj(c).has_keyword(KeywordKind::Flash));
    // Sacrificing the artifact while paying ends the condition, but the casting continues.
    t.answer_choose(P0, &[Entity::Object(stone)]);
    let s = t.cast(P0, c).go();
    assert!(!t.on_battlefield(stone));
    assert_eq!(t.stack, vec![s]);
    t.resolve();
    assert_eq!(t.named_on_battlefield("Conditional Flasher").len(), 1);
}

#[test]
fn modes_may_depend_on_additional_costs_chosen_later_in_the_announcement() {
    cr!("601.4");
    // "Kicker {1}. Choose one. If this spell was kicked, choose any number instead."
    let inscription = || {
        CB::new("Inscription Test")
            .instant()
            .cost("{G}")
            .ability(kicker("{1}"))
            .spell(Body {
                targets: vec![],
                effect: Effect::Noop,
                modal: Some(Modal {
                    min: Value::c(1),
                    max: Value::Sum(vec![
                        Value::c(1),
                        Value::Mul(Box::new(Value::TimesKicked), Box::new(Value::c(2))),
                    ]),
                    allow_repeat: false,
                    modes: vec![
                        Mode {
                            text: "gain 2 life".into(),
                            targets: vec![],
                            effect: gain(2),
                            cost: None,
                        },
                        Mode {
                            text: "draw a card".into(),
                            targets: vec![],
                            effect: draw(1),
                            cost: None,
                        },
                        Mode {
                            text: "gain 3 life".into(),
                            targets: vec![],
                            effect: gain(3),
                            cost: None,
                        },
                    ],
                    per_mode_cost: false,
                    chooser: ModeChooser::Controller,
                }),
            })
            .build()
    };
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 2);
    let c = t.custom(P0, inscription(), Zone::Hand(P0));
    t.cast(P0, c).kicked(true).modes(&[0, 2]).go();
    t.resolve();
    assert_eq!(t.life(P0), 25);
    // Unkicked, only one mode may be chosen.
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 1);
    let c = t.custom(P0, inscription(), Zone::Hand(P0));
    let s = t.cast(P0, c).kicked(false).modes(&[0, 2]).go();
    assert_eq!(t.obj(s).stack.as_ref().unwrap().chosen.len(), 1);
}

#[test]
fn targets_may_be_chosen_considering_objects_used_to_pay_costs() {
    cr!("601.5");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 1);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let splinters = t.hand(P0, "Bone Splinters");
    // The creature that will be sacrificed to pay the cost is a legal target when chosen.
    t.answer_choose(P0, &[Entity::Object(bears)]);
    let s = t.cast(P0, splinters).target(bears).go();
    assert_eq!(
        t.obj(s).stack.as_ref().unwrap().chosen[0].targets,
        vec![vec![Entity::Object(bears)]]
    );
    assert!(!t.on_battlefield(bears));
    // Its target is gone on resolution, so the spell doesn't resolve.
    t.resolve();
    assert!(t.in_graveyard(P0, "Bone Splinters"));
}

fn opponent_modal() -> CardDef {
    // "An opponent chooses one — • You gain 5 life. • That player draws a card."
    CB::new("Opponent Choice")
        .sorcery()
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
                        text: "you gain 5 life".into(),
                        targets: vec![],
                        effect: gain(5),
                        cost: None,
                    },
                    Mode {
                        text: "that player draws a card".into(),
                        targets: vec![],
                        effect: Effect::Draw {
                            who: PlayerRef::ChosenOpponent,
                            n: Value::c(1),
                        },
                        cost: None,
                    },
                ],
                per_mode_cost: false,
                chooser: ModeChooser::Opponent,
            }),
        })
        .build()
}

#[test]
fn an_opponent_makes_a_choice_the_controller_would_normally_make() {
    cr!("601.7", "601.7a");
    let mut t = TestGame::new(3);
    let c = t.custom(P0, opponent_modal(), Zone::Hand(P0));
    // The controller decides which opponent chooses: P2. P2 chooses the draw mode.
    t.answer(
        P0,
        DecisionKind::Entities,
        Answer::Entities(vec![Entity::Player(P2)]),
    );
    t.answer(P2, DecisionKind::Modes, Answer::Indices(vec![1]));
    let lib2 = t.library_size(P2);
    t.cast(P0, c).go();
    let asked = t.asked();
    assert!(asked
        .iter()
        .any(|(p, d)| *p == P2 && matches!(d, Decision::ChooseModes { .. })));
    assert!(!asked
        .iter()
        .any(|(p, d)| *p != P2 && matches!(d, Decision::ChooseModes { .. })));
    t.resolve();
    assert_eq!(t.library_size(P2), lib2 - 1);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn the_controller_chooses_before_the_opponent_when_both_choose_while_casting() {
    cr!("601.7b");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    // Slot 0 is chosen by an opponent, slot 1 by the controller: the controller goes first.
    let spell = CB::new("Both Choose")
        .instant()
        .cost("{0}")
        .spell(Body::simple(
            vec![
                TargetSpec {
                    chosen_by_opponent: true,
                    ..TargetSpec::object(Filter::creature(), "target creature an opponent chooses")
                },
                target_creature(),
            ],
            Effect::Tap {
                what: Sel::AllTargets,
            },
        ))
        .build();
    let c = t.custom(P0, spell, Zone::Hand(P0));
    t.answer_targets(P1, &[Entity::Object(a)]);
    t.answer_targets(P0, &[Entity::Object(b)]);
    t.cast(P0, c).go();
    let order: Vec<PlayerId> = t
        .asked()
        .iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseTargets { .. }))
        .map(|(p, _)| *p)
        .collect();
    assert_eq!(order, vec![P0, P1]);
    t.resolve();
    assert!(t.obj(a).tapped && t.obj(b).tapped);
}
