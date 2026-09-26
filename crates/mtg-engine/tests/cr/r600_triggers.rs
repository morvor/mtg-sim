//! CR 603.1–603.5: triggered abilities — triggering, putting them on the stack, the
//! intervening "if" rule, and optional effects.

use super::r600_common::*;
use mtg_engine::ability::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::events::MoveCause;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::*;
use mtg_engine::replacement::{EtbInfo, MoveEv};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn put_onto_battlefield(t: &mut TestGame, p: PlayerId, def: CardDef) -> ObjectId {
    let id =
        t.g.create_card_object(std::sync::Arc::new(def), p, Zone::Nowhere);
    t.g.move_object_ev(MoveEv {
        obj: id,
        to: Zone::Battlefield,
        pos: LibraryPosition::Top,
        cause: MoveCause::Effect,
        by: Some(p),
        etb: EtbInfo {
            controller: Some(p),
            ..Default::default()
        },
        source: None,
    })
    .unwrap()
}

fn watcher(name: &str, cond: TriggerCond, effect: Effect) -> CardDef {
    CB::new(name)
        .enchantment()
        .ability(trig(cond, Body::effect(effect)))
        .build()
}

fn creature_enters() -> TriggerCond {
    TriggerCond::EntersBattlefield(Filter::creature())
}

#[test]
fn a_triggered_ability_is_a_trigger_condition_and_an_effect() {
    cr!("603.1", "603.6a");
    let def = compile_def(
        "Draw Beast",
        "Creature — Beast",
        "{2}{G}",
        "When this creature enters, draw a card.",
    );
    let AbilityKind::Triggered(tr) = &abilities(&def)[0].kind else {
        panic!("not a triggered ability");
    };
    assert!(matches!(
        tr.trigger,
        TriggerCond::EntersBattlefield(Filter::Source)
    ));
    assert!(matches!(tr.body.effect, Effect::Draw { .. }));
    let mut t = TestGame::new(2);
    let hand = t.hand_size(P0);
    put_onto_battlefield(&mut t, P0, def);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn instructions_after_the_effect_function_while_the_ability_is_on_the_stack() {
    cr!("603.1a");
    let mut t = TestGame::new(2);
    // "Whenever a creature enters, you gain 1 life. This ability can't be countered."
    let mut tr = TriggeredAbility::new(creature_enters(), Body::effect(gain(1)));
    tr.cant_be_countered = true;
    t.custom(
        P0,
        CB::new("Stubborn Watcher")
            .enchantment()
            .ability(trig_from(tr))
            .build(),
        Zone::Battlefield,
    );
    t.custom(
        P0,
        watcher("Plain Watcher", creature_enters(), gain(2)),
        Zone::Battlefield,
    );
    t.enter(P1, "Grizzly Bears");
    t.settle();
    assert_eq!(t.stack.len(), 2);
    // "Counter target activated or triggered ability."
    let stifle = CB::new("Stifle Test")
        .instant()
        .cost("{0}")
        .spell(Body::simple(
            vec![TargetSpec::one(
                TargetKind::Ability(Filter::Any),
                "target ability",
            )],
            Effect::CounterSpell {
                what: Sel::Target(0),
            },
        ))
        .build();
    let (a, b) = (t.stack[0], t.stack[1]);
    for target in [a, b] {
        let s = t.custom(P1, stifle.clone(), Zone::Hand(P1));
        t.g.turn.priority = Some(P1);
        t.cast(P1, s).target(target).go();
        t.resolve();
    }
    // Only the ability that can't be countered is still there.
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
}

#[test]
fn a_trigger_with_several_conditions_can_ask_whether_all_of_them_happened() {
    cr!("603.1b");
    let conds = vec![
        TriggerCond::GainsLife {
            who: PlayerRel::You,
        },
        TriggerCond::Draws {
            who: PlayerRel::You,
        },
    ];
    // "Whenever you gain life or draw a card, put a +1/+1 counter on this creature. Then
    //  if you've both gained life and drawn a card this turn, you gain 10 life."
    let ab = trig(
        TriggerCond::AnyOf(conds.clone()),
        Body::effect(Effect::Seq(vec![
            Effect::AddCounters {
                what: Sel::This,
                kind: counters::PLUS1.into(),
                n: Value::c(1),
            },
            Effect::If {
                cond: Condition::AllTriggerConditionsThisTurn(conds),
                then: Box::new(Effect::LoseLife {
                    who: PlayerRef::EachOpponent,
                    n: Value::c(5),
                }),
                otherwise: Box::new(Effect::Noop),
            },
        ])),
    );
    let mut t = TestGame::new(2);
    // Gaining life before the permanent exists still counts for "all of those conditions".
    t.g.gain_life(P0, 1);
    t.g.flush_events();
    let c = t.custom(
        P0,
        CB::new("Both Watcher").creature(1, 1).ability(ab).build(),
        Zone::Battlefield,
    );
    t.g.draw_cards(P0, 1);
    t.resolve_all();
    assert_eq!(t.counters(c, counters::PLUS1), 1);
    assert_eq!(t.life(P1), 15);
}

#[test]
fn triggering_does_nothing_until_the_ability_resolves() {
    cr!("603.2", "603.3");
    let mut t = TestGame::new(2);
    t.custom(
        P0,
        watcher("Enter Watcher", creature_enters(), gain(3)),
        Zone::Battlefield,
    );
    t.enter(P1, "Grizzly Bears");
    t.g.flush_events();
    // It has triggered, but it isn't on the stack yet and nothing has happened.
    assert_eq!(t.pending_triggers.len(), 1);
    assert!(t.stack.is_empty());
    assert_eq!(t.life(P0), 20);
    // It's put on the stack the next time a player would receive priority.
    t.settle();
    assert_eq!(t.stack.len(), 1);
    let ab = t.stack[0];
    assert_eq!(t.obj(ab).kind, ObjKind::StackAbility);
    assert!(t.obj(ab).chars.card_types.is_empty());
    t.resolve();
    assert_eq!(t.life(P0), 23);
}

#[test]
fn triggered_abilities_trigger_even_when_nothing_could_be_cast() {
    cr!("603.2a");
    let mut t = TestGame::new(2);
    // During combat damage, no player has priority, yet "deals damage" triggers.
    let dealer = CB::new("Damage Watcher")
        .creature(2, 2)
        .ability(trig(
            TriggerCond::DealsDamage {
                source: Filter::Source,
                to: DamageRecipient::Any,
                combat_only: true,
            },
            Body::effect(gain(2)),
        ))
        .build();
    let d = t.custom(P0, dealer, Zone::Battlefield);
    // A "can't cast spells" effect doesn't matter either.
    t.custom(
        P1,
        CB::new("Silence All")
            .enchantment()
            .ability(stat(StaticEffect::Restriction(Restriction::CantCast {
                who: PlayerFilter::Any,
                what: Filter::Any,
            })))
            .build(),
        Zone::Battlefield,
    );
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(d, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.life(P0), 22);
}

#[test]
fn at_the_beginning_of_triggers_when_the_step_begins() {
    cr!("603.2b");
    let mut t = TestGame::new(2);
    t.custom(
        P0,
        watcher(
            "Upkeep Watcher",
            TriggerCond::BeginningOf {
                step: TriggerStep::Upkeep,
                whose: PlayerRel::You,
            },
            gain(1),
        ),
        Zone::Battlefield,
    );
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.life(P0), 20);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    // It triggered as the upkeep began and is on the stack when P0 gets priority.
    assert_eq!(t.stack.len(), 1);
    t.resolve();
    assert_eq!(t.life(P0), 21);
}

#[test]
fn an_ability_triggers_once_per_occurrence_of_its_trigger_event() {
    cr!("603.2c");
    let mut t = TestGame::new(2);
    // "Whenever a land is put into a graveyard from the battlefield, you gain 1 life."
    t.custom(
        P0,
        watcher(
            "Land Mourner",
            TriggerCond::ZoneChange {
                filter: Filter::Type(CardType::Land),
                from: Some(ZoneKind::Battlefield),
                to: Some(ZoneKind::Graveyard),
            },
            gain(1),
        ),
        Zone::Battlefield,
    );
    t.lands(P0, "Plains", 2);
    t.lands(P1, "Island", 3);
    let armageddon = CB::new("Armageddon Test")
        .sorcery()
        .cost("{0}")
        .spell(Body::effect(Effect::Destroy {
            what: Sel::All(Filter::Type(CardType::Land)),
            no_regen: false,
        }))
        .build();
    let a = t.custom(P0, armageddon, Zone::Hand(P0));
    t.cast(P0, a).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 25);
}

fn panharmonicon() -> CardDef {
    // "If an artifact or creature entering causes a triggered ability of a permanent you
    //  control to trigger, that ability triggers an additional time."
    CB::new("Panharmonicon Test")
        .artifact()
        .ability(stat(StaticEffect::AdditionalTrigger {
            sources: Filter::and(vec![
                Filter::Permanent,
                Filter::ControlledBy(PlayerRel::You),
            ]),
            cause: Some(Box::new(TriggerCond::EntersBattlefield(Filter::Or(vec![
                Filter::Type(CardType::Artifact),
                Filter::creature(),
            ])))),
        }))
        .build()
}

#[test]
fn an_ability_can_trigger_additional_times() {
    cr!("603.2d");
    let mut t = TestGame::new(2);
    t.custom(
        P0,
        watcher("Enter Watcher", creature_enters(), gain(1)),
        Zone::Battlefield,
    );
    t.custom(P0, panharmonicon(), Zone::Battlefield);
    t.enter(P1, "Grizzly Bears");
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
    // Two such effects each add one (they don't apply to each other).
    t.custom(P0, panharmonicon(), Zone::Battlefield);
    t.enter(P1, "Grizzly Bears");
    t.resolve_all();
    assert_eq!(t.life(P0), 25);
    // A land entering isn't the stated cause.
    t.custom(
        P0,
        watcher(
            "Land Watcher",
            TriggerCond::EntersBattlefield(Filter::Type(CardType::Land)),
            gain(10),
        ),
        Zone::Battlefield,
    );
    t.enter(P1, "Island");
    t.resolve_all();
    assert_eq!(t.life(P0), 35);
    // Delayed triggered abilities created by abilities of the object aren't affected.
    let delayed = CB::new("Delayed Maker")
        .sorcery()
        .cost("{0}")
        .spell(Body::effect(Effect::DelayedTrigger {
            trigger: creature_enters(),
            body: Box::new(Body::effect(gain(100))),
            once: true,
        }))
        .build();
    let d = t.custom(P0, delayed, Zone::Hand(P0));
    t.cast(P0, d).go();
    t.resolve();
    t.enter(P1, "Grizzly Bears");
    t.resolve_all();
    // 3 from the watcher (1 + 2 additional) plus the delayed trigger once.
    assert_eq!(t.life(P0), 138);
}

#[test]
fn becomes_triggers_trigger_only_when_the_event_happens() {
    cr!("603.2e", "603.6d");
    let mut t = TestGame::new(2);
    t.custom(
        P0,
        watcher(
            "Tap Watcher",
            TriggerCond::BecomesTapped(Filter::Type(CardType::Land)),
            gain(1),
        ),
        Zone::Battlefield,
    );
    // Selesnya Guildgate enters tapped: a static ability (not a trigger), and it doesn't
    // "become tapped".
    let gate = t.hand(P0, "Selesnya Guildgate");
    t.play_land(P0, gate).unwrap();
    t.settle();
    let g = t.named_on_battlefield("Selesnya Guildgate")[0];
    assert!(t.obj(g).tapped);
    assert!(t.stack.is_empty());
    assert_eq!(t.life(P0), 20);
    // Once it's untapped, tapping it is the "becomes tapped" event.
    t.g.untap(g);
    t.g.tap(g);
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
    // Already tapped: nothing more happens.
    t.g.tap(g);
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
}

#[test]
fn objects_never_visible_to_all_players_dont_trigger() {
    cr!("603.2f");
    let mut t = TestGame::new(2);
    let cast_watch = |zone: FunctionZone| {
        let mut tr = TriggeredAbility::new(
            TriggerCond::CastSpell {
                who: PlayerRel::Opponent,
                filter: Filter::Any,
            },
            Body::effect(gain(1)),
        );
        tr.zone = zone;
        CB::new("Hidden Watcher")
            .creature(1, 1)
            .ability(trig_from(tr))
            .build()
    };
    t.custom(P0, cast_watch(FunctionZone::Hand), Zone::Hand(P0));
    t.custom(P0, cast_watch(FunctionZone::Graveyard), Zone::Graveyard(P0));
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.g.turn.priority = Some(P1);
    t.cast(P1, bolt).target(P1).go();
    t.resolve_all();
    // Only the copy in the (public) graveyard triggered.
    assert_eq!(t.life(P0), 21);
}

#[test]
fn prevented_or_replaced_events_dont_trigger_anything() {
    cr!("603.2g");
    let mut t = TestGame::new(2);
    let dealer = CB::new("Damage Watcher")
        .creature(2, 2)
        .ability(trig(
            TriggerCond::DealsDamage {
                source: Filter::Source,
                to: DamageRecipient::Any,
                combat_only: false,
            },
            Body::effect(gain(2)),
        ))
        .ability(act(mana_cost("{0}"), damage_target(2)))
        .build();
    let d = t.custom(P0, dealer, Zone::Battlefield);
    // "Prevent all damage that would be dealt to players."
    t.g.replacements
        .push(mtg_engine::game::ReplacementInstance {
            id: 900,
            source: None,
            controller: P1,
            timestamp: 1,
            duration: Duration::EndOfTurn,
            def: ReplacementDef {
                event: ReplacementEvent::Damage {
                    source: Filter::Any,
                    to_players: Some(PlayerFilter::Any),
                    to_objects: None,
                    combat_only: false,
                },
                action: ReplacementAction::Prevent,
                self_replacement: false,
                optional: false,
            },
            uses: None,
            objects: None,
            remaining: None,
        });
    t.activate(P0, d, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.life(P0), 20);
    // A replaced draw isn't a draw.
    t.custom(
        P0,
        watcher(
            "Draw Watcher",
            TriggerCond::Draws {
                who: PlayerRel::You,
            },
            gain(5),
        ),
        Zone::Battlefield,
    );
    t.g.replacements
        .push(mtg_engine::game::ReplacementInstance {
            id: 901,
            source: None,
            controller: P0,
            timestamp: 2,
            duration: Duration::EndOfTurn,
            def: ReplacementDef {
                event: ReplacementEvent::Draw(PlayerFilter::Any),
                action: ReplacementAction::Instead(Box::new(Effect::Noop)),
                self_replacement: false,
                optional: false,
            },
            uses: Some(1),
            objects: None,
            remaining: None,
        });
    t.g.draw_cards(P0, 1);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
}

#[test]
fn do_this_only_once_each_turn_triggers_only_until_the_action_is_taken() {
    cr!("603.2h");
    let mut t = TestGame::new(2);
    // Nykthos Paragon-like: "Whenever you gain life, you may put a +1/+1 counter on this
    // creature. Do this only once each turn."
    let mut tr = TriggeredAbility::new(
        TriggerCond::GainsLife {
            who: PlayerRel::You,
        },
        Body::effect(Effect::May {
            who: PlayerRef::You,
            effect: Box::new(Effect::AddCounters {
                what: Sel::This,
                kind: counters::PLUS1.into(),
                n: Value::c(1),
            }),
        }),
    );
    tr.do_once_per_turn = true;
    let p = t.custom(
        P0,
        CB::new("Paragon Test")
            .creature(4, 6)
            .ability(trig_from(tr))
            .build(),
        Zone::Battlefield,
    );
    // Declined: the action wasn't taken, so it can trigger again.
    t.answer_yes(P0, false);
    t.g.gain_life(P0, 1);
    t.resolve_all();
    assert_eq!(t.counters(p, counters::PLUS1), 0);
    t.answer_yes(P0, true);
    t.g.gain_life(P0, 1);
    t.resolve_all();
    assert_eq!(t.counters(p, counters::PLUS1), 1);
    // Taken: it doesn't trigger again this turn.
    t.g.gain_life(P0, 1);
    t.g.flush_events();
    assert!(t.pending_triggers.is_empty());
    // Next turn it can.
    t.advance_to(P1, Step::Upkeep);
    t.g.gain_life(P0, 1);
    t.g.flush_events();
    assert_eq!(t.pending_triggers.len(), 1);
}

#[test]
fn a_triggered_ability_is_controlled_by_its_sources_controller() {
    cr!("603.3a");
    let mut t = TestGame::new(2);
    t.custom(
        P1,
        watcher("Enter Watcher", creature_enters(), gain(1)),
        Zone::Battlefield,
    );
    // P0's creature entering triggers P1's ability; P1 controls it.
    t.enter(P0, "Grizzly Bears");
    t.settle();
    assert_eq!(t.obj(t.stack[0]).controller, P1);
    t.resolve();
    assert_eq!(t.life(P1), 21);
}

#[test]
fn triggers_are_put_on_the_stack_in_apnap_order_each_player_choosing_their_order() {
    cr!("603.3b");
    let mut t = TestGame::new(2);
    let p0a = t.custom(
        P0,
        watcher("P0 First", creature_enters(), gain(1)),
        Zone::Battlefield,
    );
    let p0b = t.custom(
        P0,
        watcher("P0 Second", creature_enters(), gain(2)),
        Zone::Battlefield,
    );
    let p1 = t.custom(
        P1,
        watcher("P1 Watcher", creature_enters(), gain(3)),
        Zone::Battlefield,
    );
    // P0 (the active player) puts theirs on the stack first, in the chosen order
    // (first = bottom): Second, then First.
    t.answer(P0, DecisionKind::Order, Answer::Indices(vec![1, 0]));
    t.enter(P0, "Grizzly Bears");
    t.settle();
    let sources: Vec<ObjectId> = t
        .stack
        .iter()
        .map(|s| match &t.obj(*s).stack.as_ref().unwrap().kind {
            StackKind::Triggered { source, .. } => *source,
            _ => panic!(),
        })
        .collect();
    assert_eq!(sources, vec![p0b, p0a, p1]);
}

#[test]
fn abilities_triggering_on_other_abilities_triggering_go_on_the_stack_after_them() {
    cr!("603.3b");
    let mut t = TestGame::new(2);
    // Strict Proctor-like: "Whenever a permanent entering causes a triggered ability to
    // trigger, counter that ability unless its controller pays {2}."
    let proctor = CB::new("Proctor Test")
        .creature(1, 3)
        .ability(trig(
            TriggerCond::AbilityTriggered {
                cause: Box::new(TriggerCond::EntersBattlefield(Filter::Permanent)),
                source: Filter::Any,
            },
            Body::effect(Effect::PayOptional {
                who: PlayerRef::TriggerPlayer,
                cost: mana_cost("{2}"),
                then: Box::new(Effect::Noop),
                otherwise: Box::new(Effect::CounterSpell {
                    what: Sel::TriggerSpell,
                }),
            }),
        ))
        .build();
    t.custom(P1, proctor, Zone::Battlefield);
    t.custom(
        P0,
        watcher("Enter Watcher", creature_enters(), gain(4)),
        Zone::Battlefield,
    );
    // A non-entering trigger isn't affected.
    t.enter(P0, "Grizzly Bears");
    t.settle();
    assert_eq!(t.stack.len(), 2);
    // The Proctor's trigger is above the ability it triggered on.
    assert_eq!(t.obj(t.stack[1]).controller, P1);
    t.resolve_all();
    // P0 couldn't pay {2}, so the watcher's ability was countered.
    assert_eq!(t.life(P0), 20);
}

fn modal_trigger(modes: Vec<Mode>) -> CardDef {
    CB::new("Modal Watcher")
        .enchantment()
        .ability(trig(
            TriggerCond::BeginningOf {
                step: TriggerStep::Upkeep,
                whose: PlayerRel::You,
            },
            Body {
                targets: vec![],
                effect: Effect::Noop,
                modal: Some(Modal {
                    min: Value::c(1),
                    max: Value::c(1),
                    allow_repeat: false,
                    modes,
                    per_mode_cost: false,
                    chooser: ModeChooser::Controller,
                }),
            },
        ))
        .build()
}

#[test]
fn modes_of_a_triggered_ability_are_chosen_as_it_goes_on_the_stack() {
    cr!("603.3c");
    let destroy_mode = Mode {
        text: "destroy target creature".into(),
        targets: vec![target_creature()],
        effect: Effect::Destroy {
            what: Sel::Target(0),
            no_regen: false,
        },
        cost: None,
    };
    let gain_mode = Mode {
        text: "gain 2 life".into(),
        targets: vec![],
        effect: gain(2),
        cost: None,
    };
    let mut t = TestGame::new(2);
    t.custom(
        P0,
        modal_trigger(vec![destroy_mode.clone(), gain_mode]),
        Zone::Battlefield,
    );
    // The destroy mode is illegal (no creatures), so it can't be chosen even if asked for.
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0]));
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    let top = *t.stack.last().unwrap();
    assert_eq!(t.obj(top).stack.as_ref().unwrap().chosen[0].mode, Some(1));
    t.resolve();
    assert_eq!(t.life(P0), 22);
    // If no mode can be chosen, the ability is removed from the stack.
    let mut t = TestGame::new(2);
    t.custom(P0, modal_trigger(vec![destroy_mode]), Zone::Battlefield);
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert!(t.stack.is_empty());
}

#[test]
fn a_triggered_ability_without_legal_targets_is_removed_from_the_stack() {
    cr!("603.3d");
    let mut t = TestGame::new(2);
    // "When this creature enters, it deals 2 damage to target creature an opponent controls."
    let pinger = CB::new("ETB Pinger")
        .creature(1, 1)
        .ability(trig(
            TriggerCond::EntersBattlefield(Filter::Source),
            Body::simple(
                vec![TargetSpec::object(
                    Filter::creature().opp_controls(),
                    "target creature an opponent controls",
                )],
                Effect::DealDamage {
                    source: Sel::This,
                    amount: Value::c(2),
                    to: Sel::Target(0),
                },
            ),
        ))
        .build();
    put_onto_battlefield(&mut t, P0, pinger.clone());
    t.settle();
    assert!(t.stack.is_empty());
    // With a legal target it's put on the stack with that target.
    let bears = t.battlefield(P1, "Grizzly Bears");
    put_onto_battlefield(&mut t, P0, pinger);
    t.settle();
    let top = *t.stack.last().unwrap();
    assert_eq!(
        t.obj(top).stack.as_ref().unwrap().chosen[0].targets,
        vec![vec![Entity::Object(bears)]]
    );
    // Division is announced as it's put on the stack, too.
    let divider = CB::new("ETB Divider")
        .creature(1, 1)
        .ability(trig(
            TriggerCond::EntersBattlefield(Filter::Source),
            Body::simple(
                vec![TargetSpec {
                    min: 1,
                    max: Value::c(2),
                    divide: Some(Value::c(3)),
                    ..TargetSpec::any_target()
                }],
                Effect::DealDividedDamage {
                    source: Sel::This,
                    slot: 0,
                },
            ),
        ))
        .build();
    t.answer_targets(P0, &[Entity::Player(P1), Entity::Object(bears)]);
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![1, 2]));
    put_onto_battlefield(&mut t, P0, divider);
    t.settle();
    let top = *t.stack.last().unwrap();
    assert_eq!(
        t.obj(top).stack.as_ref().unwrap().chosen[0].divided,
        vec![vec![1, 2]]
    );
}

fn sovereign() -> CardDef {
    // Felidar Sovereign: "At the beginning of your upkeep, if you have 40 or more life,
    // you win the game."
    let mut tr = TriggeredAbility::new(
        TriggerCond::BeginningOf {
            step: TriggerStep::Upkeep,
            whose: PlayerRel::You,
        },
        Body::effect(Effect::WinGame {
            who: PlayerRef::You,
        }),
    );
    tr.intervening_if = Some(Condition::Compare(
        Value::LifeTotal(PlayerRef::You),
        Cmp::Ge,
        Value::c(40),
    ));
    CB::new("Sovereign Test")
        .creature(4, 6)
        .ability(trig_from(tr))
        .build()
}

#[test]
fn intervening_if_is_checked_on_triggering_and_on_resolution() {
    cr!("603.4", "608.2a");
    // 39 life: doesn't trigger at all.
    let mut t = TestGame::new(2);
    t.custom(P0, sovereign(), Zone::Battlefield);
    t.g.players[0].life = 39;
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert!(t.stack.is_empty());
    // 40 life: triggers; but if life drops to 39 before it resolves, it does nothing.
    let mut t = TestGame::new(2);
    t.custom(P0, sovereign(), Zone::Battlefield);
    t.g.players[0].life = 40;
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert_eq!(t.stack.len(), 1);
    t.g.players[0].life = 39;
    t.resolve();
    assert!(t.stack.is_empty());
    assert!(t.result.is_none());
    // 40 life on resolution: wins.
    let mut t = TestGame::new(2);
    t.custom(P0, sovereign(), Zone::Battlefield);
    t.g.players[0].life = 40;
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    t.resolve();
    assert_eq!(t.result, Some(GameResult::Win(vec![P0])));
}

#[test]
fn optional_triggers_go_on_the_stack_and_the_choice_is_made_on_resolution() {
    cr!("603.5");
    let mut t = TestGame::new(2);
    t.custom(
        P0,
        watcher(
            "Maybe Drawer",
            TriggerCond::BeginningOf {
                step: TriggerStep::Upkeep,
                whose: PlayerRel::You,
            },
            Effect::May {
                who: PlayerRef::You,
                effect: Box::new(draw(1)),
            },
        ),
        Zone::Battlefield,
    );
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert_eq!(t.stack.len(), 1);
    let asked_yes_no = |t: &TestGame| {
        t.asked()
            .iter()
            .filter(|(_, d)| matches!(d, Decision::YesNo { .. }))
            .count()
    };
    assert_eq!(asked_yes_no(&t), 0);
    let hand = t.hand_size(P0);
    t.answer_yes(P0, false);
    t.resolve();
    assert_eq!(asked_yes_no(&t), 1);
    assert_eq!(t.hand_size(P0), hand);
    // "Unless" is also dealt with on resolution.
    let mut t = TestGame::new(2);
    t.custom(
        P0,
        watcher(
            "Unless Taxer",
            TriggerCond::BeginningOf {
                step: TriggerStep::Upkeep,
                whose: PlayerRel::You,
            },
            Effect::PayOptional {
                who: PlayerRef::You,
                cost: mana_cost("{1}"),
                then: Box::new(Effect::Noop),
                otherwise: Box::new(Effect::LoseLife {
                    who: PlayerRef::You,
                    n: Value::c(3),
                }),
            },
        ),
        Zone::Battlefield,
    );
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert_eq!(t.stack.len(), 1);
    t.resolve();
    assert_eq!(t.life(P0), 17);
    let _ = KeywordKind::Flash;
}
