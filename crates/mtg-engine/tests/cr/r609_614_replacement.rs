//! CR 614.1–614.11: replacement effects ("instead", "skip", entering the battlefield,
//! damage, regeneration, redirection, skipping, and card draws).

use crate::r609_common::*;
use mtg_engine::ability::*;
use mtg_engine::events::Event;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// "If a card or token would be put into a graveyard from anywhere, exile it instead."
pub fn rest_in_peace() -> CardDef {
    permanent(
        "Rest in Peace",
        &[CardType::Enchantment],
        vec![replacement(
            ReplacementEvent::ZoneChange {
                filter: Filter::Any,
                from: None,
                to: Some(ZoneKind::Graveyard),
            },
            ReplacementAction::MoveInstead(Destination::zone(ZoneKind::Exile)),
        )],
    )
}

/// "If a source would deal damage to a permanent or player, it deals double that damage
/// to that permanent or player instead."
pub fn furnace() -> CardDef {
    permanent(
        "Furnace of Rath",
        &[CardType::Enchantment],
        vec![replacement(
            ReplacementEvent::Damage {
                source: Filter::Any,
                to_players: Some(PlayerFilter::Any),
                to_objects: Some(Filter::Any),
                combat_only: false,
            },
            ReplacementAction::Multiply(2),
        )],
    )
}

fn deal(t: &mut TestGame, p: PlayerId, n: i32, to: Entity) {
    resolve_effect(
        t,
        p,
        vec![target_any()],
        &[to],
        Effect::DealDamage {
            source: Sel::This,
            amount: Value::c(n),
            to: Sel::Target(0),
        },
    );
}

#[test]
fn instead_effects_replace_events() {
    // CR 614.1a: an effect with "instead" is a replacement effect. CR 614.6: the replaced
    // event never happens — the creature doesn't die, so its "dies" trigger doesn't
    // trigger — and the modified event happens instead.
    cr!("614.1a", "614.6", "614.1");
    let mut t = TestGame::new(2);
    t.custom(P0, rest_in_peace(), Zone::Battlefield);
    let c = t.custom(
        P1,
        creature_with(
            "Martyr",
            1,
            1,
            &[],
            vec![triggered(
                TriggerCond::Dies(Filter::Source),
                Effect::GainLife {
                    who: PlayerRef::You,
                    n: Value::c(5),
                },
            )],
        ),
        Zone::Battlefield,
    );
    deal(&mut t, P0, 3, Entity::Object(c));
    t.resolve_all();
    assert!(t.in_exile("Martyr"));
    assert!(!t.in_graveyard(P1, "Martyr"));
    assert_eq!(t.life(P1), 20);
    // The spell itself is exiled instead of going to its owner's graveyard.
    assert!(t.in_exile("Test Spell"));
}

#[test]
fn impossible_parts_of_a_modified_event_are_ignored() {
    // CR 614.6: a modified event may contain instructions that can't be carried out; the
    // impossible instruction is simply ignored.
    cr!("614.6");
    let mut t = TestGame::new(2);
    t.custom(
        P0,
        permanent(
            "Odd Bargain",
            &[CardType::Enchantment],
            vec![replacement(
                ReplacementEvent::Draw(PlayerFilter::You),
                ReplacementAction::Instead(Box::new(Effect::seq(vec![
                    Effect::Sacrifice {
                        who: PlayerRef::You,
                        filter: Filter::creature(),
                        count: Value::c(1),
                    },
                    Effect::GainLife {
                        who: PlayerRef::You,
                        n: Value::c(3),
                    },
                ]))),
            )],
        ),
        Zone::Battlefield,
    );
    let hand = t.hand_size(P0);
    t.g.draw_cards(P0, 1);
    assert_eq!(t.hand_size(P0), hand);
    assert_eq!(t.life(P0), 23);
}

#[test]
fn skip_effects_are_replacement_effects() {
    // CR 614.1b: "skip" effects are replacement effects: "Skip your draw step" replaces the
    // draw step with nothing.
    cr!("614.1b", "614.10");
    let mut t = TestGame::new(2);
    t.custom(
        P0,
        permanent(
            "Necropotence",
            &[CardType::Enchantment],
            vec![replacement(
                ReplacementEvent::SkipStep {
                    step: StepKind::Draw,
                    whose: PlayerRel::You,
                },
                ReplacementAction::Prevent,
            )],
        ),
        Zone::Battlefield,
    );
    let (h0, h1) = (t.hand_size(P0), t.hand_size(P1));
    t.advance_to(P1, Step::PrecombatMain);
    assert_eq!(t.hand_size(P1), h1 + 1);
    t.advance_to(P0, Step::PrecombatMain);
    assert_eq!(t.hand_size(P0), h0);
    assert!(!t.g.turn_events.iter().any(|e| matches!(
        e,
        Event::StepBegan {
            step: Step::Draw,
            ..
        }
    )));
}

#[test]
fn entering_with_counters_as_enters_and_enters_as_copy() {
    // CR 614.1c: "enters with", "As [this] enters", and "enters as" are replacement effects.
    cr!("614.1c");
    let mut t = TestGame::new(2);
    let hydra = creature_with(
        "Counter Hydra",
        0,
        0,
        &[Color::Green],
        vec![replacement(
            ReplacementEvent::EntersBattlefield(Filter::Source),
            ReplacementAction::EnterWithCounters(counters::PLUS1.into(), Value::c(3)),
        )],
    );
    let h = t.custom(P0, hydra, Zone::Hand(P0));
    t.cast_with(P0, h, &[]).unwrap();
    t.resolve();
    let h = t.named_on_battlefield("Counter Hydra")[0];
    assert_eq!(t.counters(h, counters::PLUS1), 3);
    assert_eq!(t.pt(h), (3, 3));

    let voice = creature_with(
        "Voice",
        2,
        2,
        &[Color::White],
        vec![replacement(
            ReplacementEvent::EntersBattlefield(Filter::Source),
            ReplacementAction::AsEnters(Box::new(Effect::Choose {
                who: PlayerRef::You,
                kind: ChoiceKind::Color,
            })),
        )],
    );
    t.answer(P0, DecisionKind::Option, Answer::Index(3));
    let v = t.custom(P0, voice, Zone::Hand(P0));
    t.cast_with(P0, v, &[]).unwrap();
    t.resolve();
    let v = t.named_on_battlefield("Voice")[0];
    assert_eq!(t.obj_now(v).choices.color, Some(Color::Red));

    let bears = t.battlefield(P1, "Grizzly Bears");
    let clone = creature_with(
        "Clone",
        0,
        0,
        &[Color::Blue],
        vec![replacement(
            ReplacementEvent::EntersBattlefield(Filter::Source),
            ReplacementAction::EnterAsCopy {
                filter: Filter::creature(),
                optional: true,
            },
        )],
    );
    t.answer_choose(P0, &[Entity::Object(bears)]);
    let c = t.custom(P0, clone, Zone::Hand(P0));
    t.cast_with(P0, c, &[]).unwrap();
    t.resolve();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 2);
}

#[test]
fn enters_tapped_effects() {
    // CR 614.1d: "[This permanent] enters tapped" and "[Objects] enter tapped" are
    // replacement effects.
    cr!("614.1d");
    let mut t = TestGame::new(2);
    let treefolk = t.enter(P0, "Scarwood Treefolk");
    assert!(t.obj_now(treefolk).tapped);
    t.custom(
        P1,
        permanent(
            "Blind Obedience",
            &[CardType::Enchantment],
            vec![replacement(
                ReplacementEvent::EntersBattlefield(Filter::and(vec![
                    Filter::creature(),
                    Filter::ControlledBy(PlayerRel::Opponent),
                ])),
                ReplacementAction::EnterTapped,
            )],
        ),
        Zone::Battlefield,
    );
    let bears = t.enter(P0, "Grizzly Bears");
    assert!(t.obj_now(bears).tapped);
    let own = t.enter(P1, "Grizzly Bears");
    assert!(!t.obj_now(own).tapped);
}

#[test]
fn as_turned_face_up_effects() {
    // CR 614.1e: "As [this permanent] is turned face up, ..." is a replacement effect: it
    // applies as the permanent turns face up.
    cr!("614.1e");
    ruling!(
        "Hooded Hydra",
        "It's a replacement ability that modifies how Hooded Hydra is turned face up"
    );
    let mut t = TestGame::new(2);
    let smuggler = t.custom(
        P0,
        creature_with(
            "Hooded Hydra",
            1,
            1,
            &[Color::Green],
            vec![replacement(
                ReplacementEvent::TurnedFaceUp,
                ReplacementAction::AsEnters(Box::new(Effect::AddCounters {
                    what: Sel::This,
                    kind: counters::PLUS1.into(),
                    n: Value::c(5),
                })),
            )],
        ),
        Zone::Battlefield,
    );
    t.g.objects[smuggler.0 as usize].face_down = true;
    t.recompute();
    assert_eq!(t.pt(smuggler), (2, 2));
    assert!(mtg_engine::facedown::turn_face_up(&mut t.g, smuggler, true));
    t.recompute();
    assert_eq!(t.pt(smuggler), (6, 6));
    // The counters were there before anything saw it face up.
    let up =
        t.g.events
            .iter()
            .position(|e| matches!(e, Event::TurnedFaceUp { .. }))
            .unwrap();
    let counters =
        t.g.events
            .iter()
            .position(|e| matches!(e, Event::CountersAdded { .. }))
            .unwrap();
    assert!(counters < up);
}

#[test]
fn damage_from_a_source_can_be_replaced() {
    // CR 614.2 (and 609.7): replacement effects that apply to damage from a source.
    cr!("614.2", "609.7");
    let mut t = TestGame::new(2);
    t.custom(P0, furnace(), Zone::Battlefield);
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    t.resolve();
    assert_eq!(t.life(P1), 14);
}

#[test]
fn regeneration_shields_have_no_timing_restrictions_and_last_until_used() {
    // CR 614.3: a spell or ability that generates a replacement effect has no special
    // timing restrictions; the effect lasts until it's used up or its duration ends.
    // CR 614.7: a replacement effect whose event never happens does nothing.
    cr!("614.3", "614.7");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 2);
    let skel = t.battlefield(P0, "Drudge Skeletons");
    // Activated during the opponent's turn with nothing to replace.
    t.set_step(P1, Step::Upkeep);
    t.activate(P0, skel, 0, &[]).unwrap();
    t.resolve();
    assert_eq!(t.g.replacements.len(), 1);
    t.advance_to(P1, Step::End);
    assert_eq!(t.g.replacements.len(), 1);
    assert!(!t.obj_now(skel).tapped);
    // The shield ends at end of turn without having done anything.
    t.advance_to(P0, Step::Upkeep);
    assert!(t.g.replacements.is_empty());
    assert!(t.on_battlefield(skel));
}

#[test]
fn replacement_effects_must_exist_before_the_event() {
    // CR 614.4: regeneration in response to a spell that would destroy the creature saves
    // it; afterward it's too late. CR 614.8: regeneration removes damage, taps it, and
    // removes it from combat, and abilities that trigger on damage still trigger.
    cr!("614.4", "614.8");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 1);
    t.lands(P1, "Mountain", 2);
    let skel = t.battlefield(P0, "Drudge Skeletons");
    let shock = t.hand(P1, "Shock");
    t.cast(P1, shock).target(skel).go();
    t.activate(P0, skel, 0, &[]).unwrap();
    t.resolve(); // regeneration shield
    t.resolve(); // Shock
    assert!(t.on_battlefield(skel));
    assert!(t.obj_now(skel).tapped);
    assert_eq!(t.obj_now(skel).damage, 0);
    // Without a shield in place first, it's destroyed.
    let shock2 = t.hand(P1, "Shock");
    t.cast(P1, shock2).target(skel).go();
    t.resolve();
    assert!(!t.on_battlefield(skel));
}

#[test]
fn regeneration_removes_from_combat_and_damage_triggers_still_trigger() {
    cr!("614.8");
    let mut t = TestGame::new(2);
    let regen = activated(
        Cost::free(),
        Body::effect(Effect::Regenerate { what: Sel::This }),
    );
    let pinger = triggered(
        TriggerCond::IsDealtDamage {
            filter: Filter::Source,
            combat_only: false,
        },
        Effect::GainLife {
            who: PlayerRef::You,
            n: Value::c(1),
        },
    );
    let knight = t.custom(
        P0,
        creature_with(
            "Vigilant Troll",
            2,
            2,
            &[Color::Green],
            vec![keyword(KeywordKind::Vigilance), regen, pinger],
        ),
        Zone::Battlefield,
    );
    t.set_step(P0, Step::BeginningOfCombat);
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(knight, Entity::Player(P1))]),
    );
    t.advance_to(P0, Step::DeclareAttackers);
    assert!(t.g.is_attacking(knight));
    assert!(!t.obj_now(knight).tapped);
    t.activate(P0, knight, 0, &[]).unwrap();
    t.resolve();
    deal(&mut t, P1, 5, Entity::Object(knight));
    t.resolve_all();
    assert!(t.on_battlefield(knight));
    assert!(t.obj_now(knight).tapped);
    assert!(!t.g.is_attacking(knight));
    assert_eq!(t.life(P0), 21);
}

#[test]
fn replacement_effects_apply_once_to_an_event() {
    // CR 614.5 example: two "deals double that damage" effects: 2 damage becomes 8, not 4
    // and not an infinite amount.
    cr!("614.5");
    let mut t = TestGame::new(2);
    let doubler = || {
        permanent(
            "Doubler",
            &[CardType::Enchantment],
            vec![replacement(
                ReplacementEvent::Damage {
                    source: Filter::creature().you_control(),
                    to_players: Some(PlayerFilter::Any),
                    to_objects: Some(Filter::Any),
                    combat_only: false,
                },
                ReplacementAction::Multiply(2),
            )],
        )
    };
    t.custom(P0, doubler(), Zone::Battlefield);
    t.custom(P0, doubler(), Zone::Battlefield);
    let bear = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(bear, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 12);
}

#[test]
fn zero_damage_is_not_dealt_and_cant_be_replaced() {
    // CR 614.7a: a source that would deal 0 damage deals no damage at all; effects that
    // would increase or redirect it have no event to replace.
    cr!("614.7a", "614.7");
    let mut t = TestGame::new(2);
    t.custom(P0, furnace(), Zone::Battlefield);
    let wall = t.custom(P0, creature("Wall", 0, 4, &[]), Zone::Battlefield);
    let before = t.g.turn_events.len();
    resolve_effect(
        &mut t,
        P0,
        vec![],
        &[],
        Effect::DealDamage {
            source: Sel::All(Filter::Named("Wall".into())),
            amount: Value::PowerOf(Box::new(Sel::All(Filter::Named("Wall".into())))),
            to: Sel::Players(PlayerRef::EachOpponent),
        },
    );
    let _ = wall;
    t.settle();
    assert_eq!(t.life(P1), 20);
    assert!(!t.g.turn_events[before..]
        .iter()
        .chain(t.g.events.iter())
        .any(|e| matches!(e, Event::Damage { .. })));
}

#[test]
fn redirection_to_something_no_longer_valid_does_nothing() {
    // CR 614.9: damage redirected to a creature that's no longer on the battlefield (or no
    // longer a creature) isn't redirected.
    cr!("614.9");
    for remove in [false, true] {
        let mut t = TestGame::new(2);
        let bear = t.battlefield(P1, "Grizzly Bears");
        resolve_effect(
            &mut t,
            P0,
            vec![target_creature()],
            &[Entity::Object(bear)],
            Effect::AddReplacement {
                def: ReplacementDef {
                    event: ReplacementEvent::Damage {
                        source: Filter::Any,
                        to_players: Some(PlayerFilter::You),
                        to_objects: None,
                        combat_only: false,
                    },
                    action: ReplacementAction::Redirect(Sel::Target(0)),
                    self_replacement: false,
                    optional: false,
                },
                duration: Duration::EndOfTurn,
                uses: Some(1),
            },
        );
        if remove {
            modify_target(
                &mut t,
                P1,
                bear,
                vec![Modification::RemoveTypes(vec![CardType::Creature])],
            );
        }
        deal(&mut t, P1, 1, Entity::Player(P0));
        if remove {
            assert_eq!(t.life(P0), 19);
            assert_eq!(t.obj_now(bear).damage, 0);
        } else {
            assert_eq!(t.life(P0), 20);
            assert_eq!(t.obj_now(bear).damage, 1);
        }
    }
}

#[test]
fn redirection_to_a_player_who_left_does_nothing() {
    cr!("614.9");
    let mut t = TestGame::new(3);
    resolve_effect(
        &mut t,
        P0,
        vec![TargetSpec::player(PlayerFilter::Any, "target player")],
        &[Entity::Player(P2)],
        Effect::AddReplacement {
            def: ReplacementDef {
                event: ReplacementEvent::Damage {
                    source: Filter::Any,
                    to_players: Some(PlayerFilter::You),
                    to_objects: None,
                    combat_only: false,
                },
                action: ReplacementAction::Redirect(Sel::Target(0)),
                self_replacement: false,
                optional: false,
            },
            duration: Duration::EndOfTurn,
            uses: None,
        },
    );
    deal(&mut t, P1, 2, Entity::Player(P0));
    assert_eq!(t.life(P2), 18);
    t.g.player_loses(P2);
    deal(&mut t, P1, 2, Entity::Player(P0));
    assert_eq!(t.life(P0), 18);
}

fn fatigue(t: &mut TestGame, caster: PlayerId, who: PlayerId) {
    resolve_effect(
        t,
        caster,
        vec![TargetSpec::player(PlayerFilter::Any, "target player")],
        &[Entity::Player(who)],
        Effect::Skip {
            who: PlayerRef::Target(0),
            step: StepKind::Draw,
        },
    );
}

#[test]
fn a_step_that_started_cant_be_skipped() {
    // CR 614.10: once a step has started it can't be skipped; the skip waits for the next
    // occurrence.
    cr!("614.10");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::Draw);
    let h = t.hand_size(P0);
    fatigue(&mut t, P1, P0);
    // This draw step continues; the next one is skipped.
    t.advance_to(P1, Step::PrecombatMain);
    t.advance_to(P0, Step::PrecombatMain);
    assert_eq!(t.hand_size(P0), h);
    t.advance_to(P1, Step::PrecombatMain);
    t.advance_to(P0, Step::PrecombatMain);
    assert_eq!(t.hand_size(P0), h + 1);
}

#[test]
fn two_skip_effects_skip_two_occurrences() {
    // CR 614.10a: two "skip your next draw step" effects skip the next two; anything
    // scheduled for the "next" occurrence waits for the first one that isn't skipped.
    cr!("614.10a");
    let mut t = TestGame::new(2);
    fatigue(&mut t, P1, P0);
    fatigue(&mut t, P1, P0);
    let h = t.hand_size(P0);
    // "At the beginning of your next draw step, gain 3 life."
    let mut ctx = mtg_engine::eval::Ctx::new(None, P0);
    t.g.exec(
        &Effect::DelayedTrigger {
            trigger: TriggerCond::BeginningOf {
                step: TriggerStep::Draw,
                whose: PlayerRel::You,
            },
            body: Box::new(Body::effect(Effect::GainLife {
                who: PlayerRef::You,
                n: Value::c(3),
            })),
            once: true,
        },
        &mut ctx,
    );
    for _ in 0..2 {
        t.advance_to(P1, Step::PrecombatMain);
        t.advance_to(P0, Step::PrecombatMain);
        assert_eq!(t.hand_size(P0), h);
        assert_eq!(t.life(P0), 20);
    }
    t.advance_to(P1, Step::PrecombatMain);
    t.advance_to(P0, Step::PrecombatMain);
    assert_eq!(t.hand_size(P0), h + 1);
    assert_eq!(t.life(P0), 23);
}

#[test]
fn skip_then_act_happens_first_in_the_next_step() {
    // CR 614.10b: "If you would begin your draw step, you may skip that step instead. If
    // you do, you gain 2 life." The life gain is the first thing that happens in the next
    // step that actually occurs.
    cr!("614.10b");
    let mut t = TestGame::new(2);
    t.custom(
        P0,
        permanent(
            "Fasting",
            &[CardType::Enchantment],
            vec![static_ab(StaticEffect::Replacement(ReplacementDef {
                event: ReplacementEvent::SkipStep {
                    step: StepKind::Draw,
                    whose: PlayerRel::You,
                },
                action: ReplacementAction::Instead(Box::new(Effect::GainLife {
                    who: PlayerRef::You,
                    n: Value::c(2),
                })),
                self_replacement: false,
                optional: true,
            }))],
        ),
        Zone::Battlefield,
    );
    t.set_step(P1, Step::End);
    let h = t.hand_size(P0);
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::PrecombatMain);
    assert_eq!(t.hand_size(P0), h);
    assert_eq!(t.life(P0), 22);
    let ev = &t.g.turn_events;
    let main = ev
        .iter()
        .position(|e| {
            matches!(
                e,
                Event::StepBegan {
                    step: Step::PrecombatMain,
                    ..
                }
            )
        })
        .unwrap();
    let gain = ev
        .iter()
        .position(|e| matches!(e, Event::LifeGained { .. }))
        .unwrap();
    assert!(gain > main);
}

#[test]
fn draw_replacements_apply_with_an_empty_library() {
    // CR 614.11: effects that replace card draws apply even if no cards could be drawn.
    cr!("614.11");
    let mut t = TestGame::new(2);
    t.g.players[0].library.clear();
    t.custom(
        P0,
        permanent(
            "Lab",
            &[CardType::Enchantment],
            vec![replacement(
                ReplacementEvent::Draw(PlayerFilter::You),
                ReplacementAction::Instead(Box::new(Effect::GainLife {
                    who: PlayerRef::You,
                    n: Value::c(1),
                })),
            )],
        ),
        Zone::Battlefield,
    );
    t.g.draw_cards(P0, 1);
    t.settle();
    assert_eq!(t.life(P0), 21);
    assert!(!t.has_lost(P0));
}

#[test]
fn a_replaced_draw_in_a_sequence_is_finished_first() {
    // CR 614.11a: if a draw within a sequence of draws is replaced, the replacement's
    // actions are completed before the sequence resumes.
    cr!("614.11a");
    let mut t = TestGame::new(2);
    let names = ["E", "D", "C", "B", "A"];
    for n in names {
        t.custom(P0, creature(n, 1, 1, &[]), Zone::Library(P0));
    }
    // "The next time you would draw a card this turn, mill two cards instead."
    resolve_effect(
        &mut t,
        P0,
        vec![],
        &[],
        Effect::AddReplacement {
            def: ReplacementDef {
                event: ReplacementEvent::Draw(PlayerFilter::You),
                action: ReplacementAction::Instead(Box::new(Effect::Mill {
                    who: PlayerRef::You,
                    n: Value::c(2),
                })),
                self_replacement: false,
                optional: false,
            },
            duration: Duration::EndOfTurn,
            uses: Some(1),
        },
    );
    t.g.draw_cards(P0, 3);
    assert!(t.in_graveyard(P0, "A") && t.in_graveyard(P0, "B"));
    assert!(t.in_hand(P0, "C") && t.in_hand(P0, "D"));
    assert!(!t.in_hand(P0, "E"));
}

#[test]
fn additional_actions_arent_performed_on_replacement_draws() {
    // CR 614.11b: "Draw a card. If it's a land card, you gain 3 life." If the draw is
    // replaced, the cards drawn by the replacement aren't "it".
    cr!("614.11b");
    let body = Body::effect(Effect::seq(vec![
        Effect::Draw {
            who: PlayerRef::You,
            n: Value::c(1),
        },
        Effect::If {
            cond: Condition::SelMatches(Sel::Var(vars::REVEALED), Filter::Type(CardType::Land)),
            then: Box::new(Effect::GainLife {
                who: PlayerRef::You,
                n: Value::c(3),
            }),
            otherwise: Box::new(Effect::Noop),
        },
    ]));
    for replaced in [false, true] {
        let mut t = TestGame::new(2);
        t.library_top(P0, "Forest");
        t.library_top(P0, "Forest");
        if replaced {
            t.custom(
                P0,
                permanent(
                    "Double Draw",
                    &[CardType::Enchantment],
                    vec![replacement(
                        ReplacementEvent::Draw(PlayerFilter::You),
                        ReplacementAction::Instead(Box::new(Effect::Draw {
                            who: PlayerRef::You,
                            n: Value::c(2),
                        })),
                    )],
                ),
                Zone::Battlefield,
            );
        }
        let h = t.hand_size(P0);
        cast_resolve(&mut t, P0, body.clone(), &[]);
        if replaced {
            assert_eq!(t.hand_size(P0), h + 2);
            assert_eq!(t.life(P0), 20);
        } else {
            assert_eq!(t.hand_size(P0), h + 1);
            assert_eq!(t.life(P0), 23);
        }
    }
}
