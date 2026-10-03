//! CR 603.7: delayed triggered abilities; CR 603.8: state triggers.

use super::r600_common::*;
use mtg_engine::ability::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::events::MoveCause;
use mtg_engine::keywords::{Keyword, KeywordKind};
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn sorcery(name: &str, body: Body) -> CardDef {
    CB::new(name).sorcery().cost("{0}").spell(body).build()
}

fn delayed(trigger: TriggerCond, effect: Effect, once: bool) -> Effect {
    Effect::DelayedTrigger {
        trigger,
        body: Box::new(Body::effect(effect)),
        once,
    }
}

#[test]
fn a_delayed_triggered_ability_does_something_later() {
    cr!("603.7");
    let mut t = TestGame::new(2);
    let s = t.custom(
        P0,
        sorcery(
            "Later Gain",
            Body::effect(Effect::AtNext {
                step: TriggerStep::End,
                effect: Box::new(gain(3)),
            }),
        ),
        Zone::Hand(P0),
    );
    t.cast(P0, s).go();
    t.resolve();
    assert_eq!(t.life(P0), 20);
    t.advance_to(P0, Step::End);
    t.settle();
    t.resolve_all();
    assert_eq!(t.life(P0), 23);
    // It triggers only once.
    t.advance_to(P1, Step::End);
    t.settle();
    t.resolve_all();
    assert_eq!(t.life(P0), 23);
}

#[test]
fn a_delayed_trigger_wont_trigger_on_events_before_it_was_created() {
    cr!("603.7a");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    // "Exile target creature. When that creature leaves the battlefield, you gain 5 life."
    let s = t.custom(
        P0,
        sorcery(
            "Too Late",
            Body::simple(
                vec![target_creature()],
                Effect::Seq(vec![
                    Effect::Exile {
                        what: Sel::Target(0),
                        face_down: false,
                        link: false,
                    },
                    delayed(
                        TriggerCond::LeavesBattlefield(Filter::In(Box::new(Sel::Target(0)))),
                        gain(5),
                        true,
                    ),
                ]),
            ),
        ),
        Zone::Hand(P0),
    );
    t.cast(P0, s).target(a).go();
    t.resolve_all();
    // The creature left before the delayed ability existed; another creature leaving
    // isn't "that creature".
    t.g.destroy(b, None);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    // "Untap target creature. When it becomes untapped, you gain 5 life": it untapped
    // before the delayed ability was created, so it waits for the next time.
    let c = t.battlefield(P1, "Grizzly Bears");
    t.g.tap(c);
    let s2 = t.custom(
        P0,
        sorcery(
            "Wait For It",
            Body::simple(
                vec![target_creature()],
                Effect::Seq(vec![
                    Effect::Untap {
                        what: Sel::Target(0),
                    },
                    delayed(
                        TriggerCond::BecomesUntapped(Filter::In(Box::new(Sel::Target(0)))),
                        gain(5),
                        true,
                    ),
                ]),
            ),
        ),
        Zone::Hand(P0),
    );
    t.cast(P0, s2).target(c).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    t.g.tap(c);
    t.g.untap(c);
    t.resolve_all();
    assert_eq!(t.life(P0), 25);
}

#[test]
fn a_delayed_trigger_triggers_once_unless_it_has_a_duration() {
    cr!("603.7b");
    let mut t = TestGame::new(2);
    let dies = TriggerCond::Dies(Filter::creature());
    let s1 = t.custom(
        P0,
        sorcery("Once", Body::effect(delayed(dies.clone(), gain(1), true))),
        Zone::Hand(P0),
    );
    let s2 = t.custom(
        P0,
        sorcery("This Turn", Body::effect(delayed(dies, gain(10), false))),
        Zone::Hand(P0),
    );
    t.cast(P0, s1).go();
    t.resolve();
    t.cast(P0, s2).go();
    t.resolve();
    let x = t.battlefield(P1, "Grizzly Bears");
    let y = t.battlefield(P1, "Hill Giant");
    t.g.destroy(x, None);
    t.resolve_all();
    t.g.destroy(y, None);
    t.resolve_all();
    // "Once" triggered for the first death only; "this turn" for both.
    assert_eq!(t.life(P0), 41);
    // The "this turn" delayed trigger ends with the turn.
    t.advance_to(P1, Step::PrecombatMain);
    let z = t.battlefield(P1, "Grizzly Bears");
    t.g.destroy(z, None);
    t.resolve_all();
    assert_eq!(t.life(P0), 41);
}

#[test]
fn simultaneous_events_let_the_controller_choose_which_causes_the_delayed_trigger() {
    cr!("603.7b");
    let mut t = TestGame::new(2);
    // "When a creature dies this turn, return that card to the battlefield." (once)
    let s = t.custom(
        P0,
        sorcery(
            "Pick One",
            Body::effect(delayed(
                TriggerCond::Dies(Filter::creature()),
                Effect::Move {
                    what: Sel::TriggerObject,
                    to: Destination::battlefield(),
                },
                true,
            )),
        ),
        Zone::Hand(P0),
    );
    t.cast(P0, s).go();
    t.resolve();
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    let wrath = t.custom(
        P0,
        sorcery(
            "Wrath",
            Body::effect(Effect::Destroy {
                what: Sel::All(Filter::creature()),
                no_regen: true,
            }),
        ),
        Zone::Hand(P0),
    );
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.cast(P0, wrath).go();
    t.resolve_all();
    assert!(t.asked().iter().any(|(p, d)| *p == P0
        && matches!(d, Decision::ChooseOption { prompt, .. } if prompt.contains("delayed"))));
    // Exactly one of them came back: the chosen one.
    let back: Vec<ObjectId> = t
        .permanents()
        .filter(|o| o.is_creature())
        .map(|o| o.id)
        .collect();
    assert_eq!(back.len(), 1);
    assert_eq!(t.obj(back[0]).chars.name, "Hill Giant");
    let _ = (a, b);
}

#[test]
fn a_delayed_trigger_affects_its_object_despite_changes_but_not_a_new_object() {
    cr!("603.7c");
    // "{0}: Exile this creature at the beginning of the next end step."
    let doomed = || {
        CB::new("Doomed")
            .creature(2, 2)
            .ability(act(
                mana_cost("{0}"),
                Body::effect(Effect::AtNext {
                    step: TriggerStep::End,
                    effect: Box::new(Effect::Exile {
                        what: Sel::This,
                        face_down: false,
                        link: false,
                    }),
                }),
            ))
            .build()
    };
    let mut t = TestGame::new(2);
    let d = t.custom(P0, doomed(), Zone::Battlefield);
    t.activate(P0, d, 0, &[]).unwrap();
    t.resolve();
    // It stops being a creature; it's still exiled.
    t.g.effects.push(mtg_engine::game::ContinuousEffect {
        id: 950,
        source: None,
        controller: P0,
        timestamp: t.g.next_timestamp,
        duration: Duration::Permanent,
        affected: mtg_engine::game::Affected::Objects(vec![d]),
        mods: vec![Modification::RemoveTypes(vec![CardType::Creature])],
        layer1: None,
        created_turn: 1,
    });
    t.g.recompute();
    assert!(!t.obj(d).is_creature());
    t.advance_to(P0, Step::End);
    t.settle();
    t.resolve_all();
    assert!(t.in_exile("Doomed"));
    // If it left the battlefield and came back, it's a new object and stays.
    let mut t = TestGame::new(2);
    let d = t.custom(P0, doomed(), Zone::Battlefield);
    t.activate(P0, d, 0, &[]).unwrap();
    t.resolve();
    let ex = t.g.exile_object(d, None).unwrap();
    t.g.move_object(ex, Zone::Battlefield, MoveCause::Effect, Some(P0));
    t.advance_to(P0, Step::End);
    t.settle();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Doomed").len(), 1);
}

#[test]
fn a_spells_delayed_trigger_has_the_spell_as_its_source() {
    cr!("603.7d");
    let mut t = TestGame::new(2);
    let plain = t.battlefield(P1, "Grizzly Bears");
    // A creature with protection from red can't be targeted by the red spell's ability.
    let warded = t.custom(
        P1,
        CB::new("Red Warded")
            .creature(2, 2)
            .ability(AbilityDef::new(
                AbilityKind::Keyword(Keyword::with_filter(
                    KeywordKind::Protection,
                    Filter::Color(Color::Red),
                )),
                "Protection from red",
            ))
            .build(),
        Zone::Battlefield,
    );
    let red = CB::new("Delayed Burn")
        .sorcery()
        .cost("{R}")
        .spell(Body::effect(Effect::AtNext {
            step: TriggerStep::End,
            effect: Box::new(Effect::Noop),
        }))
        .build();
    // Replace the body: at the next end step, "this spell" deals 2 damage to target creature.
    let red = {
        let mut c = red.faces[0].chars.clone();
        c.abilities = vec![spell(Body::effect(Effect::DelayedTrigger {
            trigger: TriggerCond::BeginningOf {
                step: TriggerStep::End,
                whose: PlayerRel::Any,
            },
            body: Box::new(Body::simple(
                vec![target_creature()],
                Effect::DealDamage {
                    source: Sel::This,
                    amount: Value::c(2),
                    to: Sel::Target(0),
                },
            )),
            once: true,
        }))];
        CardDef::custom(c)
    };
    t.lands(P0, "Mountain", 1);
    let s = t.custom(P0, red, Zone::Hand(P0));
    t.cast(P0, s).go();
    t.resolve();
    t.advance_to(P0, Step::End);
    t.settle();
    let top = *t.stack.last().unwrap();
    // Controlled by the spell's controller; its source is the (red) spell.
    assert_eq!(t.obj(top).controller, P0);
    assert_eq!(
        t.obj(top).stack.as_ref().unwrap().chosen[0].targets,
        vec![vec![Entity::Object(plain)]]
    );
    t.resolve_all();
    assert!(!t.on_battlefield(plain));
    assert!(t.on_battlefield(warded));
}

#[test]
fn an_abilitys_delayed_trigger_is_controlled_by_the_abilitys_controller() {
    cr!("603.7e");
    let mut t = TestGame::new(2);
    // "{0}: At the beginning of the next end step, you gain 3 life."
    let src = t.custom(
        P0,
        CB::new("Delayer")
            .creature(1, 1)
            .ability(act(
                mana_cost("{0}"),
                Body::effect(Effect::AtNext {
                    step: TriggerStep::End,
                    effect: Box::new(gain(3)),
                }),
            ))
            .build(),
        Zone::Battlefield,
    );
    t.activate(P0, src, 0, &[]).unwrap();
    t.resolve();
    // P1 gains control of the source; the delayed trigger is still P0's.
    let steal = t.custom(
        P1,
        sorcery(
            "Steal",
            Body::simple(
                vec![target_creature()],
                Effect::GainControl {
                    what: Sel::Target(0),
                    who: PlayerRef::You,
                    duration: Duration::Permanent,
                },
            ),
        ),
        Zone::Hand(P1),
    );
    t.g.turn.priority = Some(P1);
    t.set_step(P1, Step::PrecombatMain);
    t.cast(P1, steal).target(src).go();
    t.resolve();
    assert_eq!(t.obj(src).controller, P1);
    t.advance_to(P1, Step::End);
    t.settle();
    let top = *t.stack.last().unwrap();
    assert_eq!(t.obj(top).controller, P0);
    match &t.obj(top).stack.as_ref().unwrap().kind {
        StackKind::Triggered { source, .. } => assert_eq!(*source, src),
        _ => panic!(),
    }
    t.resolve_all();
    assert_eq!(t.life(P0), 23);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn a_replacement_effects_delayed_trigger_belongs_to_the_static_abilitys_object() {
    cr!("603.7f");
    let mut t = TestGame::new(2);
    // "If a creature would die, exile it instead. At the beginning of the next end step,
    // return it to the battlefield under its owner's control."
    let exiled_return = Effect::Seq(vec![
        Effect::Exile {
            what: Sel::TriggerObject,
            face_down: false,
            link: false,
        },
        Effect::AtNext {
            step: TriggerStep::End,
            effect: Box::new(Effect::Move {
                what: Sel::Var(vars::IT),
                to: Destination {
                    controller: Some(PlayerRef::OwnerOf(Box::new(Sel::Var(vars::IT)))),
                    ..Destination::battlefield()
                },
            }),
        },
    ]);
    let keeper = t.custom(
        P0,
        CB::new("Spirit Keeper")
            .enchantment()
            .ability(stat(StaticEffect::Replacement(ReplacementDef {
                event: ReplacementEvent::Dies(Filter::creature()),
                action: ReplacementAction::Instead(Box::new(exiled_return)),
                self_replacement: false,
                optional: false,
            })))
            .build(),
        Zone::Battlefield,
    );
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.destroy(bears, None);
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    assert_eq!(t.g.delayed_triggers.len(), 1);
    assert_eq!(t.g.delayed_triggers[0].controller, P0);
    assert_eq!(t.g.delayed_triggers[0].source, Some(keeper));
    t.advance_to(P0, Step::End);
    t.settle();
    let top = *t.stack.last().unwrap();
    assert_eq!(t.obj(top).controller, P0);
    t.resolve_all();
    let b = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(b.len(), 1);
    assert_eq!(t.obj(b[0]).controller, P1);
}

#[test]
fn a_delayed_trigger_created_on_a_particular_resolution_is_created_only_once() {
    cr!("603.7h");
    // Gimli, Mournful Avenger: "Whenever another creature you control dies, put a +1/+1
    // counter on Gimli. When this ability resolves for the third time this turn, Gimli
    // fights up to one target creature you don't control."
    let mut t = TestGame::new(2);
    let gimli = t.battlefield(P0, "Gimli, Mournful Avenger");
    let victims: Vec<ObjectId> = (0..4).map(|_| t.battlefield(P0, "Grizzly Bears")).collect();
    let foe = t.battlefield(P1, "Hill Giant");
    let foe2 = t.battlefield(P1, "Hill Giant");
    for (i, v) in victims.iter().enumerate() {
        if i == 2 {
            t.answer_targets(P0, &[Entity::Object(foe)]);
        }
        t.g.destroy(*v, None);
        t.settle();
        t.resolve();
        // The fight trigger exists only after the third resolution.
        let fights = t.stack.len();
        assert_eq!(
            fights,
            usize::from(i == 2),
            "after death {}: {:?} {}",
            i + 1,
            t.stack
                .iter()
                .map(|s| t.obj(*s).chars.rules_text.to_string())
                .collect::<Vec<_>>(),
            t.dump_log()
        );
        t.resolve_all();
    }
    assert_eq!(t.counters(gimli, counters::PLUS1), 4);
    // The chosen Hill Giant fought once (and died: Gimli is 6/5 after three counters).
    assert!(!t.on_battlefield(foe));
    assert!(t.on_battlefield(foe2));
}

#[test]
fn state_triggers_trigger_when_the_game_state_matches() {
    cr!("603.8");
    // "Whenever you have no cards in hand, draw a card."
    let st = CB::new("Empty Hand Muse")
        .enchantment()
        .ability(trig(
            TriggerCond::State(Condition::Compare(
                Value::HandSize(PlayerRef::You),
                Cmp::Eq,
                Value::c(0),
            )),
            Body::effect(draw(1)),
        ))
        .build();
    let mut t = TestGame::new(2);
    t.custom(P0, st, Zone::Battlefield);
    assert_eq!(t.hand_size(P0), 0);
    t.settle();
    // It triggers once and doesn't trigger again while on the stack.
    assert_eq!(t.stack.len(), 1);
    t.settle();
    assert_eq!(t.stack.len(), 1);
    t.resolve();
    assert_eq!(t.hand_size(P0), 1);
    assert!(t.stack.is_empty());
    // "Discard your hand, then draw that many cards": the hand is momentarily empty
    // during the resolution, so it triggers then.
    t.hand(P0, "Grizzly Bears");
    let wheel = t.custom(
        P0,
        sorcery(
            "Hand Wheel",
            Body::effect(Effect::Seq(vec![
                Effect::DiscardHand {
                    who: PlayerRef::You,
                },
                Effect::Draw {
                    who: PlayerRef::You,
                    n: Value::Prev,
                },
            ])),
        ),
        Zone::Hand(P0),
    );
    t.cast(P0, wheel).go();
    t.resolve();
    assert_eq!(t.hand_size(P0), 2);
    assert_eq!(t.stack.len(), 1);
    t.resolve();
    assert_eq!(t.hand_size(P0), 3);
}

#[test]
fn a_countered_state_trigger_can_trigger_again() {
    cr!("603.8");
    let st = CB::new("Low Life Watcher")
        .enchantment()
        .ability(trig(
            TriggerCond::State(Condition::Compare(
                Value::LifeTotal(PlayerRef::You),
                Cmp::Le,
                Value::c(5),
            )),
            Body::effect(gain(1)),
        ))
        .build();
    let mut t = TestGame::new(2);
    t.custom(P0, st, Zone::Battlefield);
    t.g.players[0].life = 3;
    t.settle();
    assert_eq!(t.stack.len(), 1);
    let ab = t.stack[0];
    assert!(t.g.counter(ab, None));
    // It left the stack and the state still matches: it triggers again.
    t.settle();
    assert_eq!(t.stack.len(), 1);
    t.resolve();
    assert_eq!(t.life(P0), 4);
}
