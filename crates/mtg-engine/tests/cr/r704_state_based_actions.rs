//! CR 704: state-based actions.

use crate::r114_common::{probe_lines, spy};
use crate::r703_common::*;
use mtg_engine::ability::*;
use mtg_engine::decision::Decision;
use mtg_engine::events::Event;
use mtg_engine::game::ReplacementInstance;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::{Stage, Step};
use mtg_engine::types::*;
use mtg_engine::*;

fn modify(t: &mut TestGame, id: ObjectId, mods: Vec<Modification>) {
    run_effect(
        t,
        P0,
        None,
        Effect::Modify {
            what: Sel::Target(0),
            mods,
            duration: Duration::EndOfTurn,
        },
        &[Entity::Object(id)],
    );
}

fn shrink(t: &mut TestGame, id: ObjectId, n: i32) {
    modify(
        t,
        id,
        vec![Modification::ModifyPT(Value::c(-n), Value::c(-n))],
    );
}

#[test]
fn state_based_actions_happen_automatically_without_the_stack_or_a_player() {
    cr!("704.1", "704.2");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let asked = t.asked().len();
    shrink(&mut t, bears, 2);
    // Nothing has checked state-based actions yet.
    assert!(t.on_battlefield(bears));
    t.settle();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(t.stack_len(), 0);
    // No player controlled it: nobody was asked anything.
    assert_eq!(t.asked().len(), asked);
    // They're checked throughout the game, e.g. during an opponent's turn when a player
    // would receive priority.
    let giant = t.battlefield(P0, "Hill Giant");
    t.set_step(P1, Step::Upkeep);
    shrink(&mut t, giant, 3);
    to_step_start(&mut t, P1, Step::Draw);
    t.g.advance();
    assert!(t.in_graveyard(P0, "Hill Giant"));
}

#[test]
fn abilities_that_watch_for_a_game_state_are_triggered_abilities() {
    cr!("704.1a");
    supported("Sea Serpent");
    let mut t = TestGame::new(2);
    let serpent = t.battlefield(P0, "Sea Serpent");
    t.settle();
    // "When you control no Islands, sacrifice this creature." triggers and uses the stack;
    // a player can respond before it resolves.
    assert_eq!(t.stack_len(), 1);
    assert!(t.on_battlefield(serpent));
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Sea Serpent"));
}

#[test]
fn checks_repeat_until_nothing_happens_then_triggers_go_on_the_stack() {
    cr!("704.3");
    supported("Blood Artist");
    supported("Pacifism");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Blood Artist");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let pacifism = t.battlefield(P0, "Pacifism");
    t.g.attach(pacifism, Entity::Object(bears));
    shrink(&mut t, bears, 2);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.settle();
    // First check: the Bears died. That check performed an action, so the game checked
    // again: now the Aura is attached to nothing and goes to the graveyard. Only then was
    // Blood Artist's trigger put on the stack.
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(t.in_graveyard(P0, "Pacifism"));
    let died = event_index(
        &t,
        |e| matches!(e, Event::ZoneChange { old, .. } if *old == bears),
    )
    .unwrap();
    let aura = event_index(
        &t,
        |e| matches!(e, Event::ZoneChange { old, .. } if *old == pacifism),
    )
    .unwrap();
    let trig = event_index(&t, |e| matches!(e, Event::AbilityTriggeredOnStack { .. })).unwrap();
    assert!(died < aura && aura < trig);
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
}

#[test]
fn state_based_actions_in_a_check_are_performed_simultaneously() {
    cr!("704.3");
    let mut t = TestGame::new(2);
    let artist = t.battlefield(P0, "Blood Artist");
    let bears = t.battlefield(P0, "Grizzly Bears");
    shrink(&mut t, artist, 1);
    shrink(&mut t, bears, 2);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.settle();
    // Both left in one event, so Blood Artist saw both deaths (itself and the Bears).
    assert!(!t.on_battlefield(artist) && !t.on_battlefield(bears));
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

#[test]
fn cleanup_step_gives_priority_only_if_something_happened() {
    cr!("704.3");
    for with_trigger in [false, true] {
        let mut t = TestGame::new(2);
        if with_trigger {
            // Discarding to hand size in the cleanup step triggers this.
            let watcher = oracle_card(
                "Discard Watcher",
                "Enchantment",
                "{0}",
                None,
                "Whenever you discard a card, you gain 1 life.",
            );
            t.custom(P0, watcher, Zone::Battlefield);
        }
        for _ in 0..8 {
            t.hand(P0, "Grizzly Bears");
        }
        let seen = spy(&mut t, P0, |g, _, d| match d {
            Decision::Priority { .. } if g.turn.step == Step::Cleanup => {
                Some("cleanup priority".into())
            }
            _ => None,
        });
        t.set_step(P0, Step::End);
        let mut cleanups = 0;
        run_until(&mut t, "P1's turn", |g| {
            if g.turn.step == Step::Cleanup && g.turn.stage == Stage::Begin {
                cleanups += 1;
            }
            g.turn.active == P1
        });
        assert_eq!(t.hand_size(P0), 7);
        if with_trigger {
            // The trigger was put on the stack and players received priority; then
            // another cleanup step began.
            assert!(!probe_lines(&seen).is_empty());
            assert_eq!(cleanups, 2);
            assert_eq!(t.life(P0), 21);
        } else {
            // Nothing happened: the step ended without anyone receiving priority.
            assert!(probe_lines(&seen).is_empty());
            assert_eq!(cleanups, 1);
        }
    }
}

#[test]
fn state_based_actions_ignore_what_happens_during_resolution() {
    cr!("704.4");
    supported("Maro");
    let mut t = TestGame::new(2);
    let maro = t.battlefield(P0, "Maro");
    for _ in 0..3 {
        t.hand(P0, "Grizzly Bears");
    }
    t.g.recompute();
    assert_eq!(t.pt(maro), (3, 3));
    // "Discard your hand, then draw seven cards." Maro is 0/0 in the middle of the
    // resolution, but 7/7 when state-based actions are next checked.
    let wheel = instant_doing(
        "Private Wheel",
        Effect::Seq(vec![
            Effect::DiscardHand {
                who: PlayerRef::You,
            },
            Effect::Draw {
                who: PlayerRef::You,
                n: Value::c(7),
            },
        ]),
    );
    cast_and_resolve(&mut t, P0, wheel, &[]);
    assert!(t.on_battlefield(maro));
    assert_eq!(t.pt(maro), (7, 7));
}

#[test]
fn ten_poison_counters_lose_the_game() {
    cr!("704.5", "704.5c");
    let mut t = TestGame::new(2);
    t.g.add_counters(Entity::Player(P1), counters::POISON, 9, None);
    t.settle();
    assert!(!t.has_lost(P1));
    t.g.add_counters(Entity::Player(P1), counters::POISON, 1, None);
    t.settle();
    assert!(t.has_lost(P1));
}

#[test]
fn tokens_outside_the_battlefield_cease_to_exist() {
    cr!("704.5d");
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 2);
    t.lands(P0, "Island", 1);
    let alarm = t.hand(P0, "Raise the Alarm");
    t.cast(P0, alarm).go();
    t.resolve();
    let soldiers = t.named_on_battlefield("Soldier Token");
    assert_eq!(soldiers.len(), 2);
    let unsummon = t.hand(P0, "Unsummon");
    t.cast(P0, unsummon).target(soldiers[0]).go();
    t.g.resolve_top();
    // The token is in its owner's hand until state-based actions are checked...
    let in_hand = t.g.current(soldiers[0]);
    assert_eq!(t.obj(in_hand).zone, Zone::Hand(P0));
    t.settle();
    // ...then it ceases to exist.
    assert!(!t.player(P0).hand.contains(&in_hand));
    assert_eq!(t.obj(in_hand).zone, Zone::Nowhere);
    assert!(t.on_battlefield(soldiers[1]));
}

#[test]
fn world_rule_keeps_only_the_newest_world_permanent() {
    cr!("704.5k");
    supported("Concordant Crossroads");
    let mut t = TestGame::new(2);
    let old = t.battlefield(P0, "Concordant Crossroads");
    t.settle();
    let new = t.enter(P1, "Concordant Crossroads");
    t.settle();
    assert!(!t.on_battlefield(old));
    assert!(t.on_battlefield(new));
    // A tie for the shortest time: all of them go.
    let mut t = TestGame::new(2);
    let a = t.graveyard(P0, "Concordant Crossroads");
    let b = t.graveyard(P1, "Concordant Crossroads");
    let moves = [(a, P0), (b, P1)]
        .iter()
        .map(|(id, p)| mtg_engine::replacement::MoveEv {
            obj: *id,
            to: Zone::Battlefield,
            pos: LibraryPosition::Top,
            cause: mtg_engine::events::MoveCause::Effect,
            by: Some(*p),
            etb: mtg_engine::replacement::EtbInfo {
                controller: Some(*p),
                ..Default::default()
            },
            source: None,
        })
        .collect();
    t.g.move_objects(moves);
    t.g.recompute();
    assert_eq!(t.named_on_battlefield("Concordant Crossroads").len(), 2);
    t.settle();
    assert!(t.named_on_battlefield("Concordant Crossroads").is_empty());
    assert!(t.in_graveyard(P0, "Concordant Crossroads"));
    assert!(t.in_graveyard(P1, "Concordant Crossroads"));
}

#[test]
fn attached_creatures_and_other_permanents_become_unattached() {
    cr!("704.5p");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let splitter = t.battlefield(P0, "Bonesplitter");
    t.g.attach(splitter, Entity::Object(bears));
    t.settle();
    assert_eq!(t.obj(splitter).attached_to, Some(Entity::Object(bears)));
    // It becomes an artifact creature: an attached creature becomes unattached and stays
    // on the battlefield.
    modify(
        &mut t,
        splitter,
        vec![
            Modification::AddTypes(vec![CardType::Creature]),
            Modification::SetPT(Some(Value::c(1)), Some(Value::c(1))),
        ],
    );
    t.settle();
    assert!(t.on_battlefield(splitter));
    assert_eq!(t.obj(splitter).attached_to, None);
    // A noncreature permanent that's no longer an Equipment (or an Aura or a
    // Fortification) becomes unattached too.
    let other = t.battlefield(P0, "Bonesplitter");
    t.g.attach(other, Entity::Object(bears));
    t.settle();
    assert_eq!(t.obj(other).attached_to, Some(Entity::Object(bears)));
    modify(
        &mut t,
        other,
        vec![Modification::RemoveSubtypes(vec!["Equipment".into()])],
    );
    t.settle();
    assert!(t.on_battlefield(other));
    assert_eq!(t.obj(other).attached_to, None);
}

#[test]
fn plus_one_and_minus_one_counters_annihilate() {
    cr!("704.5q");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.add_counters(Entity::Object(bears), counters::PLUS1, 3, None);
    t.g.add_counters(Entity::Object(bears), counters::MINUS1, 2, None);
    t.g.recompute();
    assert_eq!(t.pt(bears), (3, 3));
    t.settle();
    assert_eq!(t.counters(bears, counters::PLUS1), 1);
    assert_eq!(t.counters(bears, counters::MINUS1), 0);
    assert_eq!(t.pt(bears), (3, 3));
}

#[test]
fn a_permanent_loses_counters_beyond_its_limit() {
    cr!("704.5r");
    let mut t = TestGame::new(2);
    let dreamer = oracle_card(
        "Capped Dreamer",
        "Creature — Human Wizard",
        "{0}",
        Some((4, 1)),
        "~ can't have more than seven dream counters on it.",
    );
    let d = t.custom(P0, dreamer, Zone::Battlefield);
    t.g.add_counters(Entity::Object(d), "dream", 9, None);
    assert_eq!(t.counters(d, "dream"), 9);
    t.settle();
    assert_eq!(t.counters(d, "dream"), 7);
}

fn test_saga(name: &str) -> CardDef {
    oracle_card(
        name,
        "Enchantment — Saga",
        "{0}",
        None,
        "I — You gain 1 life.\nII — You gain 2 life.\nIII — You gain 3 life.",
    )
}

#[test]
fn a_saga_is_sacrificed_after_its_final_chapter_leaves_the_stack() {
    cr!("704.5s");
    let mut t = TestGame::new(2);
    let saga = t.custom(P0, test_saga("Short Saga"), Zone::Battlefield);
    t.g.add_counters(Entity::Object(saga), counters::LORE, 2, None);
    t.resolve_all();
    let life = t.life(P0);
    t.g.add_counters(Entity::Object(saga), counters::LORE, 1, None);
    t.settle();
    // Chapter III has triggered but not left the stack: the Saga stays.
    assert_eq!(t.stack_len(), 1);
    assert!(t.on_battlefield(saga));
    t.resolve();
    assert_eq!(t.life(P0), life + 3);
    assert!(t.in_graveyard(P0, "Short Saga"));
    // A Saga with enough lore counters and no chapter ability waiting is sacrificed.
    let other = t.custom(P0, test_saga("Other Saga"), Zone::Battlefield);
    t.g.objects[other.0 as usize]
        .counters
        .insert(counters::LORE.into(), 4);
    t.settle();
    assert!(t.in_graveyard(P0, "Other Saga"));
    let sacrificed = t
        .turn_events
        .iter()
        .any(|e| matches!(e, Event::Sacrificed { obj, player } if *obj == other && *player == P0));
    assert!(sacrificed);
}

fn battle(name: &str, siege: bool, defense: i32) -> CardDef {
    let line = if siege { "Battle — Siege" } else { "Battle" };
    let mut d = oracle_card(
        name,
        line,
        "{0}",
        None,
        "Whenever ~ is dealt damage, you gain 1 life.",
    );
    d.faces[0].chars.defense = Some(defense);
    d
}

fn damage(t: &mut TestGame, target: ObjectId, n: i32) {
    let src = t.battlefield(P1, "Grizzly Bears");
    run_effect(
        t,
        P1,
        Some(src),
        Effect::DealDamage {
            source: Sel::This,
            amount: Value::c(n),
            to: Sel::Target(0),
        },
        &[Entity::Object(target)],
    );
}

#[test]
fn battles_with_zero_defense_are_put_into_the_graveyard() {
    cr!("704.5v", "704.5w");
    let mut t = TestGame::new(2);
    let siege = t.custom(P0, battle("Test Siege", true, 3), Zone::Battlefield);
    let plain = t.custom(P0, battle("Test Battle", false, 3), Zone::Battlefield);
    t.g.objects[siege.0 as usize].choices.player = Some(P1);
    t.g.objects[plain.0 as usize].choices.player = Some(P0);
    t.settle();
    damage(&mut t, siege, 3);
    damage(&mut t, plain, 3);
    t.settle();
    // Both have defense 0 and each is the source of a trigger on the stack: the Siege
    // waits for it; the other battle doesn't.
    assert!(t.on_battlefield(siege));
    assert!(!t.on_battlefield(plain));
    assert!(t.in_graveyard(P0, "Test Battle"));
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Test Siege"));
}

#[test]
fn a_battle_without_a_protector_gets_one() {
    cr!("704.5x");
    let mut t = TestGame::new(3);
    t.answer(
        P0,
        DecisionKind::Entities,
        Answer::Entities(vec![Entity::Player(P2)]),
    );
    let siege = t.custom(P0, battle("Test Siege", true, 3), Zone::Battlefield);
    t.settle();
    assert_eq!(t.obj(siege).choices.player, Some(P2));
    // The choice was among the controller's opponents.
    let asked = t.asked();
    let (_, d) = asked
        .iter()
        .rev()
        .find(|(p, d)| *p == P0 && matches!(d, Decision::ChooseEntities { .. }))
        .unwrap();
    let Decision::ChooseEntities { candidates, .. } = d else {
        unreachable!()
    };
    assert_eq!(candidates, &vec![Entity::Player(P1), Entity::Player(P2)]);
    // A battle with no battle type: its controller must be chosen.
    let plain = t.custom(P1, battle("Test Battle", false, 3), Zone::Battlefield);
    t.settle();
    assert_eq!(t.obj(plain).choices.player, Some(P1));
}

#[test]
fn a_battle_whose_protector_cant_protect_it_gets_a_new_one() {
    cr!("704.5y");
    let mut t = TestGame::new(2);
    let siege = t.custom(P0, battle("Test Siege", true, 3), Zone::Battlefield);
    t.settle();
    assert_eq!(t.obj(siege).choices.player, Some(P1));
    // P1 gains control of the Siege: P1 can't protect a Siege they control, so P1
    // chooses a new protector from among their opponents.
    run_effect(
        &mut t,
        P1,
        None,
        Effect::GainControl {
            what: Sel::Target(0),
            who: PlayerRef::You,
            duration: Duration::Permanent,
        },
        &[Entity::Object(siege)],
    );
    assert_eq!(t.obj(siege).controller, P1);
    t.settle();
    assert_eq!(t.obj(siege).choices.player, Some(P0));
}

#[test]
fn only_the_newest_role_a_player_controls_on_a_permanent_stays() {
    cr!("704.5z");
    supported("Monstrous Rage");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let rage = |t: &mut TestGame, p: PlayerId| {
        t.lands(p, "Mountain", 1);
        let r = t.hand(p, "Monstrous Rage");
        t.cast(p, r).target(bears).go();
        t.resolve_all();
    };
    rage(&mut t, P0);
    let first = t.named_on_battlefield("Monster");
    assert_eq!(first.len(), 1);
    rage(&mut t, P0);
    let roles = t.named_on_battlefield("Monster");
    assert_eq!(roles.len(), 1);
    assert_ne!(roles[0], first[0]);
    // A Role controlled by another player on the same permanent doesn't count.
    t.set_step(P1, Step::PrecombatMain);
    rage(&mut t, P1);
    assert_eq!(t.named_on_battlefield("Monster").len(), 2);
}

#[test]
fn start_your_engines_gives_a_player_without_speed_speed_one() {
    cr!("704.5aa");
    supported("Burnout Bashtronaut");
    let mut t = TestGame::new(2);
    assert_eq!(t.player(P0).speed, None);
    t.battlefield(P0, "Burnout Bashtronaut");
    t.settle();
    assert_eq!(t.player(P0).speed, Some(1));
    assert_eq!(t.player(P1).speed, None);
    // A player who already has speed keeps it.
    t.g.players[0].speed = Some(3);
    t.settle();
    assert_eq!(t.player(P0).speed, Some(3));
}

#[test]
fn one_replacement_effect_replaces_several_identical_state_based_actions() {
    cr!("704.7");
    let mut t = TestGame::new(2);
    // Lich's Mirror's effect: "If you would lose the game, instead shuffle your hand,
    // your graveyard, and all permanents you own into your library, then draw seven
    // cards and your life total becomes 20."
    let owned_here =
        |z: ZoneKind| Filter::And(vec![Filter::OwnedBy(PlayerRel::You), Filter::InZone(z)]);
    let mirror = t.battlefield(P0, "Grizzly Bears");
    let id = t.g.new_effect_id();
    let ts = t.g.new_timestamp();
    t.g.replacements.push(ReplacementInstance {
        id,
        source: Some(mirror),
        controller: P0,
        timestamp: ts,
        duration: Duration::Permanent,
        def: ReplacementDef {
            event: ReplacementEvent::LoseGame(PlayerFilter::You),
            action: ReplacementAction::Instead(Box::new(Effect::Seq(vec![
                Effect::ShuffleInto {
                    what: Sel::Union(vec![
                        Sel::All(owned_here(ZoneKind::Hand)),
                        Sel::All(owned_here(ZoneKind::Graveyard)),
                        Sel::All(Filter::And(vec![
                            Filter::OwnedBy(PlayerRel::You),
                            Filter::Permanent,
                        ])),
                    ]),
                },
                Effect::Draw {
                    who: PlayerRef::You,
                    n: Value::c(7),
                },
                Effect::SetLife {
                    who: PlayerRef::You,
                    n: Value::c(20),
                },
            ]))),
            self_replacement: false,
            optional: false,
        },
        uses: None,
        objects: None,
        remaining: None,
    });
    // One card in the library, 1 life; a spell makes P0 draw two cards and lose 2 life.
    let lib = t.player(P0).library.clone();
    for c in &lib[1..] {
        t.g.move_object(
            *c,
            Zone::Graveyard(P0),
            mtg_engine::events::MoveCause::Effect,
            None,
        );
    }
    t.g.players[0].life = 1;
    let spell = instant_doing(
        "Painful Study",
        Effect::Seq(vec![
            Effect::Draw {
                who: PlayerRef::You,
                n: Value::c(2),
            },
            Effect::LoseLife {
                who: PlayerRef::You,
                n: Value::c(2),
            },
        ]),
    );
    cast_and_resolve(&mut t, P0, spell, &[]);
    // Both 704.5a and 704.5b would make P0 lose; the single game loss was replaced once:
    // seven cards drawn (not fourteen), life 20.
    assert!(!t.has_lost(P0));
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.hand_size(P0), 7);
    assert!(t.g.result.is_none());
}

#[test]
fn last_known_information_is_from_before_any_state_based_actions() {
    cr!("704.8");
    // Young Wolf: a 1/1 with undying, spelled out.
    let young_wolf = || {
        let mut d = oracle_card(
            "Undying Pup",
            "Creature — Wolf",
            "{G}",
            Some((1, 1)),
            "When this creature dies, return that card to the battlefield under its owner's control.",
        );
        // "... if it had no +1/+1 counters on it, ..." (its last known information).
        for a in d.faces[0].chars.abilities.iter_mut() {
            if let AbilityKind::Triggered(t) = &mut std::sync::Arc::make_mut(a).kind {
                t.intervening_if = Some(Condition::Compare(
                    Value::CountersOn(Box::new(Sel::TriggerLki), Some(counters::PLUS1.into())),
                    Cmp::Eq,
                    Value::c(0),
                ));
            }
        }
        d
    };
    let mut t = TestGame::new(2);
    let wolf = t.custom(P0, young_wolf(), Zone::Battlefield);
    t.g.add_counters(Entity::Object(wolf), counters::PLUS1, 1, None);
    t.settle();
    // Three -1/-1 counters: it dies (toughness -1) at the same time as one +1/+1 and one
    // -1/-1 counter would be removed. It had a +1/+1 counter when it was last on the
    // battlefield, so undying doesn't trigger.
    t.g.add_counters(Entity::Object(wolf), counters::MINUS1, 3, None);
    t.settle();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Undying Pup"));
    assert!(t.named_on_battlefield("Undying Pup").is_empty());
    // Without the +1/+1 counter, undying would have returned it.
    let other = t.custom(P1, young_wolf(), Zone::Battlefield);
    t.g.add_counters(Entity::Object(other), counters::MINUS1, 1, None);
    t.settle();
    t.resolve_all();
    let back = t.named_on_battlefield("Undying Pup");
    assert_eq!(back.len(), 1);
}
