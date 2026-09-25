//! CR 615: prevention effects, and CR 609.7: effects that apply to damage from a source.

use crate::r609_common::*;
use mtg_engine::ability::*;
use mtg_engine::events::Event;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn deal_from(t: &mut TestGame, src: ObjectId, n: u32, to: Entity) {
    t.g.deal_damage(src, to, n, false);
    t.settle();
}

fn prevent_def(
    source: Filter,
    to_players: Option<PlayerFilter>,
    to_objects: Option<Filter>,
    action: ReplacementAction,
) -> ReplacementDef {
    ReplacementDef {
        event: ReplacementEvent::Damage {
            source,
            to_players,
            to_objects,
            combat_only: false,
        },
        action,
        self_replacement: false,
        optional: false,
    }
}

/// Resolves "prevent ..." as a spell that creates the given replacement effect.
fn add_prevention(
    t: &mut TestGame,
    p: PlayerId,
    specs: Vec<TargetSpec>,
    targets: &[Entity],
    def: ReplacementDef,
    uses: Option<u32>,
) {
    resolve_effect(
        t,
        p,
        specs,
        targets,
        Effect::AddReplacement {
            def,
            duration: Duration::EndOfTurn,
            uses,
        },
    );
}

#[test]
fn prevent_effects_are_prevention_effects() {
    // CR 615.1, 615.1a: Fog ("Prevent all combat damage that would be dealt this turn").
    cr!("615.1", "615.1a");
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    t.lands(P1, "Forest", 1);
    let fog = t.hand(P1, "Fog");
    t.cast(P1, fog).go();
    t.resolve();
    t.attack(&[(bear, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn prevention_of_damage_from_a_source() {
    // CR 615.2 (see 609.7): "Prevent all damage target creature would deal this turn."
    // It also lasts as a continuous effect through the turn (CR 609.6).
    cr!("615.2", "609.6", "609.7");
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P0, "Grizzly Bears");
    let other = t.battlefield(P0, "Hill Giant");
    add_prevention(
        &mut t,
        P1,
        vec![target_creature()],
        &[Entity::Object(bear)],
        prevent_def(
            Filter::In(Box::new(Sel::Target(0))),
            Some(PlayerFilter::Any),
            Some(Filter::Any),
            ReplacementAction::Prevent,
        ),
        None,
    );
    deal_from(&mut t, bear, 2, Entity::Player(P1));
    deal_from(&mut t, bear, 2, Entity::Player(P1));
    assert_eq!(t.life(P1), 20);
    deal_from(&mut t, other, 3, Entity::Player(P1));
    assert_eq!(t.life(P1), 17);
    // It ends at end of turn.
    t.advance_to(P1, Step::Upkeep);
    deal_from(&mut t, bear, 2, Entity::Player(P1));
    assert_eq!(t.life(P1), 15);
}

#[test]
fn shields_last_until_used_up_or_expired() {
    // CR 615.3: no special timing restrictions; the effect lasts until used up or its
    // duration ends. CR 615.7: a shield of N prevents 1 damage at a time; the number of
    // events doesn't matter; once it's 0, damage is dealt normally.
    cr!("615.3", "615.7");
    let mut t = TestGame::new(2);
    let src = t.battlefield(P1, "Hill Giant");
    let c = t.custom(P0, creature("Wall", 0, 10, &[]), Zone::Battlefield);
    t.set_step(P1, Step::End);
    resolve_effect(
        &mut t,
        P0,
        vec![target_creature()],
        &[Entity::Object(c)],
        Effect::PreventDamage {
            to: Sel::Target(0),
            amount: Some(Value::c(3)),
            duration: Duration::EndOfTurn,
            combat_only: false,
        },
    );
    deal_from(&mut t, src, 1, Entity::Object(c));
    assert_eq!(t.obj_now(c).damage, 0);
    deal_from(&mut t, src, 3, Entity::Object(c));
    assert_eq!(t.obj_now(c).damage, 1);
    deal_from(&mut t, src, 2, Entity::Object(c));
    assert_eq!(t.obj_now(c).damage, 3);
    // An unused shield expires at end of turn.
    resolve_effect(
        &mut t,
        P0,
        vec![target_creature()],
        &[Entity::Object(c)],
        Effect::PreventDamage {
            to: Sel::Target(0),
            amount: Some(Value::c(3)),
            duration: Duration::EndOfTurn,
            combat_only: false,
        },
    );
    assert_eq!(t.g.replacements.len(), 1);
    t.advance_to(P0, Step::Upkeep);
    assert!(t.g.replacements.is_empty());
}

#[test]
fn prevention_must_exist_before_the_damage() {
    // CR 615.4: once damage has been dealt it's too late to prevent it.
    cr!("615.4");
    let mut t = TestGame::new(2);
    let src = t.battlefield(P1, "Hill Giant");
    let c = t.custom(P0, creature("Wall", 0, 10, &[]), Zone::Battlefield);
    deal_from(&mut t, src, 3, Entity::Object(c));
    resolve_effect(
        &mut t,
        P0,
        vec![target_creature()],
        &[Entity::Object(c)],
        Effect::PreventDamage {
            to: Sel::Target(0),
            amount: Some(Value::c(3)),
            duration: Duration::EndOfTurn,
            combat_only: false,
        },
    );
    assert_eq!(t.obj_now(c).damage, 3);
}

#[test]
fn additional_effects_happen_right_after_prevention() {
    // CR 615.5: "Prevent the next 3 damage ... You gain life equal to the damage prevented
    // this way." The prevention happens when the damage would be dealt; the life gain
    // immediately afterward — a player at 1 life taking 5 survives at 2.
    cr!("615.5");
    let mut t = TestGame::new(2);
    t.g.players[0].life = 1;
    let src = t.battlefield(P1, "Hill Giant");
    add_prevention(
        &mut t,
        P0,
        vec![],
        &[],
        prevent_def(
            Filter::Any,
            Some(PlayerFilter::You),
            None,
            ReplacementAction::PreventAndThen(
                Some(Value::c(3)),
                Box::new(Effect::GainLife {
                    who: PlayerRef::You,
                    n: Value::EventAmount,
                }),
            ),
        ),
        None,
    );
    deal_from(&mut t, src, 5, Entity::Player(P0));
    assert_eq!(t.life(P0), 2);
    assert!(!t.has_lost(P0));
}

#[test]
fn prevented_damage_never_happens() {
    // CR 615.6: prevented damage is never dealt: "is dealt damage" abilities don't trigger
    // and lifelink doesn't apply.
    cr!("615.6");
    let mut t = TestGame::new(2);
    let src = t.custom(
        P1,
        creature_with(
            "Leech",
            2,
            2,
            &[Color::Black],
            vec![keyword(KeywordKind::Lifelink)],
        ),
        Zone::Battlefield,
    );
    let c = t.custom(
        P0,
        creature_with(
            "Sensitive",
            1,
            5,
            &[],
            vec![triggered(
                TriggerCond::IsDealtDamage {
                    filter: Filter::Source,
                    combat_only: false,
                },
                Effect::GainLife {
                    who: PlayerRef::You,
                    n: Value::c(10),
                },
            )],
        ),
        Zone::Battlefield,
    );
    add_prevention(
        &mut t,
        P0,
        vec![target_creature()],
        &[Entity::Object(c)],
        prevent_def(
            Filter::Any,
            None,
            Some(Filter::In(Box::new(Sel::Target(0)))),
            ReplacementAction::Prevent,
        ),
        None,
    );
    deal_from(&mut t, src, 2, Entity::Object(c));
    t.resolve_all();
    assert_eq!(t.obj_now(c).damage, 0);
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn the_shielded_player_chooses_which_simultaneous_damage_is_prevented() {
    // CR 615.7: damage from two sources at the same time to a shielded permanent: its
    // controller chooses which damage the shield prevents.
    cr!("615.7");
    for (order, gain) in [(vec![0usize, 1], 0), (vec![1, 0], 1)] {
        let mut t = TestGame::new(2);
        let leech = t.custom(
            P1,
            creature_with(
                "Leech",
                2,
                2,
                &[Color::Black],
                vec![keyword(KeywordKind::Lifelink)],
            ),
            Zone::Battlefield,
        );
        let brute = t.custom(
            P1,
            creature("Brute", 2, 2, &[Color::Red]),
            Zone::Battlefield,
        );
        let c = t.custom(P0, creature("Wall", 0, 10, &[]), Zone::Battlefield);
        resolve_effect(
            &mut t,
            P0,
            vec![target_creature()],
            &[Entity::Object(c)],
            Effect::PreventDamage {
                to: Sel::Target(0),
                amount: Some(Value::c(3)),
                duration: Duration::EndOfTurn,
                combat_only: false,
            },
        );
        t.answer(P0, DecisionKind::Order, Answer::Indices(order));
        t.g.deal_damage_batch(
            vec![(leech, Entity::Object(c), 2), (brute, Entity::Object(c), 2)],
            true,
        );
        assert_eq!(t.obj_now(c).damage, 1);
        assert_eq!(t.life(P1), 20 + gain);
    }
}

#[test]
fn next_time_a_source_would_deal_damage() {
    // CR 615.8: "the next time [source] would deal damage" prevents the next instance of
    // damage from it, however much; later instances are dealt normally.
    cr!("615.8");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    add_prevention(
        &mut t,
        P0,
        vec![target_creature()],
        &[Entity::Object(giant)],
        prevent_def(
            Filter::In(Box::new(Sel::Target(0))),
            Some(PlayerFilter::Any),
            Some(Filter::Any),
            ReplacementAction::Prevent,
        ),
        Some(1),
    );
    deal_from(&mut t, giant, 5, Entity::Player(P0));
    assert_eq!(t.life(P0), 20);
    deal_from(&mut t, giant, 3, Entity::Player(P0));
    assert_eq!(t.life(P0), 17);
}

/// Circle of Protection: Red: "The next time a red source of your choice would deal
/// damage to you this turn, prevent that damage."
fn cop_red(t: &mut TestGame, p: PlayerId) {
    let red = Filter::Color(Color::Red);
    resolve_effect(
        t,
        p,
        vec![],
        &[],
        Effect::seq(vec![
            Effect::ChooseSource {
                who: PlayerRef::You,
                filter: red.clone(),
                var: 20,
            },
            Effect::AddReplacement {
                def: prevent_def(
                    Filter::and(vec![Filter::In(Box::new(Sel::Var(20))), red]),
                    Some(PlayerFilter::You),
                    None,
                    ReplacementAction::Prevent,
                ),
                duration: Duration::EndOfTurn,
                uses: Some(1),
            },
        ]),
    );
}

#[test]
fn shields_recheck_the_chosen_sources_properties() {
    // CR 609.7b, 615.9: the shield rechecks the source's properties when it would deal
    // damage; if they no longer match, the damage isn't prevented and the shield isn't
    // used up.
    cr!("609.7b", "615.9");
    let mut t = TestGame::new(2);
    let goblin = t.custom(
        P1,
        creature("Red Goblin", 2, 2, &[Color::Red]),
        Zone::Battlefield,
    );
    t.answer_choose(P0, &[Entity::Object(goblin)]);
    cop_red(&mut t, P0);
    modify_target(
        &mut t,
        P1,
        goblin,
        vec![Modification::SetColors(colors(&[Color::Blue]))],
    );
    deal_from(&mut t, goblin, 2, Entity::Player(P0));
    assert_eq!(t.life(P0), 18);
    modify_target(
        &mut t,
        P1,
        goblin,
        vec![Modification::SetColors(colors(&[Color::Red]))],
    );
    deal_from(&mut t, goblin, 2, Entity::Player(P0));
    assert_eq!(t.life(P0), 18);
    deal_from(&mut t, goblin, 2, Entity::Player(P0));
    assert_eq!(t.life(P0), 16);
}

#[test]
fn choosing_a_source_of_damage() {
    // CR 609.7a: the chosen source can be a permanent, a spell on the stack (a permanent
    // spell's choice also covers the permanent it becomes), an object referred to by an
    // object on the stack (even if it's left its zone), or a face-up object in the command
    // zone. It needn't be able to deal damage, and it's chosen as the effect is created.
    cr!("609.7a");
    let mut t = TestGame::new(2);
    let wall = t.custom(
        P1,
        creature("Harmless", 0, 3, &[Color::Red]),
        Zone::Battlefield,
    );
    let pinger = creature_with(
        "Pinger",
        1,
        1,
        &[Color::Red],
        vec![
            keyword(KeywordKind::Flash),
            triggered(
                TriggerCond::EntersBattlefield(Filter::Source),
                Effect::DealDamage {
                    source: Sel::This,
                    amount: Value::c(2),
                    to: Sel::Players(PlayerRef::EachOpponent),
                },
            ),
        ],
    );
    let emblem = t.custom(P1, permanent("Red Emblem", &[], vec![]), Zone::Command);
    t.g.objects[emblem.0 as usize].chars.colors = colors(&[Color::Red]);
    t.g.objects[emblem.0 as usize].base.colors = colors(&[Color::Red]);
    let dies_ping = creature_with(
        "Dying Pinger",
        1,
        1,
        &[Color::Red],
        vec![triggered(
            TriggerCond::Dies(Filter::Source),
            Effect::DealDamage {
                source: Sel::This,
                amount: Value::c(3),
                to: Sel::Players(PlayerRef::EachOpponent),
            },
        )],
    );
    let dp = t.custom(P1, dies_ping, Zone::Battlefield);
    // Put a dies trigger on the stack: the creature that died is referred to by it.
    t.g.objects[dp.0 as usize].damage = 1;
    t.settle();
    assert_eq!(t.stack_len(), 1);
    let p = t.custom(P1, pinger, Zone::Hand(P1));
    let spell = t.cast_with(P1, p, &[]).unwrap();
    let cands = mtg_engine::prevention::source_candidates(&t.g);
    assert!(cands.contains(&wall));
    assert!(cands.contains(&spell));
    assert!(cands.contains(&dp));
    assert!(cands.contains(&emblem));
    // Choose the creature spell: its permanent's damage is prevented.
    t.answer_choose(P0, &[Entity::Object(spell)]);
    cop_red(&mut t, P0);
    t.resolve_all();
    // The Pinger's ETB damage (2) is prevented; the dying Pinger's 3 damage isn't.
    assert_eq!(t.life(P0), 17);
}

#[test]
fn static_prevention_applies_to_sources_off_the_battlefield() {
    // CR 609.7c: a static "prevent all damage red sources would deal to you" applies to red
    // permanents and to red sources that aren't on the battlefield (such as a red spell).
    cr!("609.7c");
    let mut t = TestGame::new(2);
    t.custom(
        P0,
        permanent(
            "Red Ward",
            &[CardType::Enchantment],
            vec![replacement(
                ReplacementEvent::Damage {
                    source: Filter::Color(Color::Red),
                    to_players: Some(PlayerFilter::You),
                    to_objects: None,
                    combat_only: false,
                },
                ReplacementAction::Prevent,
            )],
        ),
        Zone::Battlefield,
    );
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(P0).go();
    t.resolve();
    assert_eq!(t.life(P0), 20);
    let goblin = t.battlefield(P1, "Raging Goblin");
    deal_from(&mut t, goblin, 1, Entity::Player(P0));
    assert_eq!(t.life(P0), 20);
    let bear = t.battlefield(P1, "Grizzly Bears");
    deal_from(&mut t, bear, 2, Entity::Player(P0));
    assert_eq!(t.life(P0), 18);
}

#[test]
fn static_prevent_one_applies_to_each_event() {
    // CR 615.10 example: Daunting Defender ("If a source would deal damage to a Cleric
    // creature you control, prevent 1 of that damage") and Pyroclasm (2 damage to each
    // creature): each Cleric is dealt 1, others 2.
    cr!("615.10");
    let mut t = TestGame::new(2);
    t.custom(
        P0,
        creature_with(
            "Daunting Defender",
            3,
            3,
            &[Color::White],
            vec![replacement(
                ReplacementEvent::Damage {
                    source: Filter::Any,
                    to_players: None,
                    to_objects: Some(Filter::and(vec![
                        Filter::Subtype("Cleric".into()),
                        Filter::creature(),
                        Filter::ControlledBy(PlayerRel::You),
                    ])),
                    combat_only: false,
                },
                ReplacementAction::PreventAmount(Value::c(1)),
            )],
        ),
        Zone::Battlefield,
    );
    let mut cleric = creature("Cleric A", 1, 5, &[Color::White]);
    cleric.faces[0].chars.subtypes.push("Cleric".into());
    let a = t.custom(P0, cleric.clone(), Zone::Battlefield);
    let b = t.custom(P0, cleric, Zone::Battlefield);
    let other = t.custom(P0, creature("Other", 1, 5, &[]), Zone::Battlefield);
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Mountain", 2);
    let pyro = t.hand(P1, "Pyroclasm");
    t.cast(P1, pyro).go();
    t.resolve();
    assert_eq!(t.obj_now(a).damage, 1);
    assert_eq!(t.obj_now(b).damage, 1);
    assert_eq!(t.obj_now(other).damage, 2);
}

#[test]
fn shields_for_each_of_a_set_of_creatures_are_locked_in() {
    // CR 615.11 example: Wojek Apothecary — "Prevent the next 1 damage that would be dealt
    // to target creature and each other creature that shares a color with it this turn":
    // shields are created for the creatures sharing a color at resolution; later color
    // changes and new creatures don't matter.
    cr!("615.11");
    let mut t = TestGame::new(2);
    let target = t.custom(
        P0,
        creature("White A", 1, 5, &[Color::White]),
        Zone::Battlefield,
    );
    let friend = t.custom(
        P0,
        creature("White B", 1, 5, &[Color::White]),
        Zone::Battlefield,
    );
    let stranger = t.custom(
        P0,
        creature("Green C", 1, 5, &[Color::Green]),
        Zone::Battlefield,
    );
    resolve_effect(
        &mut t,
        P0,
        vec![target_creature()],
        &[Entity::Object(target)],
        Effect::PreventDamage {
            to: Sel::Union(vec![
                Sel::Target(0),
                Sel::All(Filter::and(vec![
                    Filter::creature(),
                    Filter::SharesColor(Box::new(Sel::Target(0))),
                    Filter::not(Filter::In(Box::new(Sel::Target(0)))),
                ])),
            ]),
            amount: Some(Value::c(1)),
            duration: Duration::EndOfTurn,
            combat_only: false,
        },
    );
    modify_target(
        &mut t,
        P0,
        stranger,
        vec![Modification::SetColors(colors(&[Color::White]))],
    );
    modify_target(
        &mut t,
        P0,
        friend,
        vec![Modification::SetColors(colors(&[Color::Green]))],
    );
    let late = t.custom(
        P0,
        creature("White D", 1, 5, &[Color::White]),
        Zone::Battlefield,
    );
    let src = t.battlefield(P1, "Hill Giant");
    t.g.deal_damage_batch(
        vec![
            (src, Entity::Object(target), 2),
            (src, Entity::Object(friend), 2),
            (src, Entity::Object(stranger), 2),
            (src, Entity::Object(late), 2),
        ],
        false,
    );
    assert_eq!(t.obj_now(target).damage, 1);
    assert_eq!(t.obj_now(friend).damage, 1);
    assert_eq!(t.obj_now(stranger).damage, 2);
    assert_eq!(t.obj_now(late).damage, 2);
}

#[test]
fn damage_that_cant_be_prevented() {
    // CR 615.12: prevention effects still apply to unpreventable damage: they prevent
    // nothing, their additional effects happen, and shields aren't reduced. CR 615.12a:
    // each applies just once to the event.
    cr!("615.12", "615.12a");
    let mut t = TestGame::new(2);
    let src = t.battlefield(P1, "Hill Giant");
    let c = t.custom(P0, creature("Wall", 0, 10, &[]), Zone::Battlefield);
    resolve_effect(
        &mut t,
        P0,
        vec![target_creature()],
        &[Entity::Object(c)],
        Effect::PreventDamage {
            to: Sel::Target(0),
            amount: Some(Value::c(2)),
            duration: Duration::EndOfTurn,
            combat_only: false,
        },
    );
    add_prevention(
        &mut t,
        P0,
        vec![],
        &[],
        prevent_def(
            Filter::Any,
            None,
            Some(Filter::creature().you_control()),
            ReplacementAction::PreventAndThen(
                None,
                Box::new(Effect::GainLife {
                    who: PlayerRef::You,
                    n: Value::c(1),
                }),
            ),
        ),
        None,
    );
    // Skullcrack: "Damage can't be prevented this turn."
    resolve_effect(
        &mut t,
        P1,
        vec![],
        &[],
        Effect::AddRestriction {
            restriction: Restriction::DamageCantBePrevented,
            duration: Duration::EndOfTurn,
        },
    );
    deal_from(&mut t, src, 3, Entity::Object(c));
    assert_eq!(t.obj_now(c).damage, 3);
    assert_eq!(t.life(P0), 21);
    let shield =
        t.g.replacements
            .iter()
            .find(|r| r.remaining.is_some())
            .unwrap();
    assert_eq!(shield.remaining, Some(2));
}

#[test]
fn damage_prevented_triggers_once_per_application() {
    // CR 615.13: "Whenever damage that would be dealt to this creature is prevented"
    // triggers each time a prevention effect is applied to one or more simultaneous damage
    // events and prevents some or all of it.
    cr!("615.13");
    let mut t = TestGame::new(2);
    let c = t.custom(
        P0,
        creature_with(
            "Stonehoof",
            1,
            5,
            &[],
            vec![
                replacement(
                    ReplacementEvent::Damage {
                        source: Filter::Any,
                        to_players: None,
                        to_objects: Some(Filter::Source),
                        combat_only: false,
                    },
                    ReplacementAction::Prevent,
                ),
                triggered(
                    TriggerCond::Custom("damage prevented:self".into()),
                    Effect::GainLife {
                        who: PlayerRef::You,
                        n: Value::EventAmount,
                    },
                ),
            ],
        ),
        Zone::Battlefield,
    );
    let a = t.battlefield(P1, "Hill Giant");
    let b = t.battlefield(P1, "Grizzly Bears");
    t.g.deal_damage_batch(
        vec![(a, Entity::Object(c), 3), (b, Entity::Object(c), 2)],
        true,
    );
    t.resolve_all();
    let prevented: Vec<u32> =
        t.g.turn_events
            .iter()
            .filter_map(|e| match e {
                Event::DamagePrevented { amount, .. } => Some(*amount),
                _ => None,
            })
            .collect();
    assert_eq!(prevented, vec![5]);
    assert_eq!(t.life(P0), 25);
    assert_eq!(t.obj_now(c).damage, 0);
}
