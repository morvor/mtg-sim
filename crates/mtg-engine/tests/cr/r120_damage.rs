//! CR 120: damage.

use super::r114_common::*;
use mtg_engine::ability::*;
use mtg_engine::events::Event;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn damage_events(t: &TestGame) -> Vec<(ObjectId, Entity, u32)> {
    events_matching(t, |e| matches!(e, Event::Damage { .. }))
        .into_iter()
        .map(|e| match e {
            Event::Damage {
                source,
                target,
                amount,
                ..
            } => (source, target, amount),
            _ => unreachable!(),
        })
        .collect()
}

fn bolt(t: &mut TestGame, target: Entity) -> ObjectId {
    t.lands(P0, "Mountain", 1);
    let b = t.hand(P0, "Lightning Bolt");
    let s = t.cast(P0, b).target(target).go();
    t.resolve();
    s
}

/// A battle (Siege) with the given defense.
fn siege(defense: i32) -> CardDef {
    let mut cb = CB::new("Test Siege")
        .types(&[CardType::Battle])
        .subtypes(&["Siege"]);
    cb.0.defense = Some(defense);
    cb.build()
}

#[test]
fn objects_deal_damage_to_battles_creatures_planeswalkers_and_players() {
    cr!("120.1");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let jace = t.battlefield(P1, "Jace Beleren");
    let battle = t.custom(P1, siege(5), Zone::Battlefield);
    let s1 = bolt(&mut t, Entity::Object(bears));
    let s2 = bolt(&mut t, Entity::Object(jace));
    let s3 = bolt(&mut t, Entity::Object(battle));
    let s4 = bolt(&mut t, Entity::Player(P1));
    // Each time, the spell that dealt the damage is its source.
    assert_eq!(
        damage_events(&t),
        vec![
            (s1, Entity::Object(bears), 3),
            (s2, Entity::Object(jace), 3),
            (s3, Entity::Object(battle), 3),
            (s4, Entity::Player(P1), 3),
        ]
    );
}

#[test]
fn damage_cant_be_dealt_to_objects_that_arent_battles_creatures_or_planeswalkers() {
    cr!("120.1a");
    let mut t = TestGame::new(2);
    let ring = t.battlefield(P1, "Sol Ring");
    let myr = t.battlefield(P1, "Darksteel Myr");
    let spell = CB::new("Scrap Blast")
        .sorcery()
        .cost("{0}")
        .spell(Body::effect(Effect::DealDamage {
            source: Sel::This,
            amount: Value::c(2),
            to: Sel::All(Filter::Type(CardType::Artifact)),
        }))
        .build();
    let s = t.custom(P0, spell, Zone::Hand(P0));
    t.cast(P0, s).go();
    t.resolve();
    let dealt = damage_events(&t);
    // Only the artifact creature was dealt damage.
    assert_eq!(dealt.len(), 1);
    assert_eq!(dealt[0].1, Entity::Object(myr));
    assert_eq!(t.obj(ring).damage, 0);
    // Directly too: nothing happens.
    t.g.deal_damage(myr, Entity::Object(ring), 3, false);
    assert_eq!(damage_events(&t).len(), 1);
}

#[test]
fn any_object_can_deal_damage() {
    cr!("120.2");
    let mut t = TestGame::new(2);
    // A noncreature enchantment deals damage.
    let pinger = CB::new("Burning Sigil")
        .enchantment()
        .ability(act(Cost::tap(), damage_target(1)))
        .build();
    let sigil = t.custom(P0, pinger, Zone::Battlefield);
    t.activate(P0, sigil, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve();
    assert_eq!(damage_events(&t), vec![(sigil, Entity::Player(P1), 1)]);
    assert_eq!(t.life(P1), 19);
}

#[test]
fn attacking_and_blocking_creatures_deal_combat_damage_equal_to_power() {
    cr!("120.2a");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let wurm = t.battlefield(P0, "Craw Wurm");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(
        &[(giant, Entity::Player(P1)), (wurm, Entity::Player(P1))],
        &[(bears, wurm)],
    );
    assert_eq!(t.life(P1), 17);
    let dealt = damage_events(&t);
    assert!(dealt.contains(&(giant, Entity::Player(P1), 3)));
    assert!(dealt.contains(&(wurm, Entity::Object(bears), 6)));
    assert!(dealt.contains(&(bears, Entity::Object(wurm), 2)));
}

#[test]
fn a_spell_or_ability_specifies_which_object_deals_the_damage() {
    cr!("120.2b");
    let mut t = TestGame::new(2);
    let hawk = t.battlefield(P0, "Vampire Nighthawk");
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Forest", 2);
    let bite = t.hand(P0, "Rabid Bite");
    t.cast(P0, bite)
        .targets(&[Entity::Object(hawk)])
        .target(giant)
        .go();
    t.resolve();
    // The creature, not the spell, dealt the damage: its lifelink and deathtouch apply.
    assert_eq!(damage_events(&t), vec![(hawk, Entity::Object(giant), 2)]);
    assert_eq!(t.life(P0), 22);
    assert!(!t.on_battlefield(giant));
}

#[test]
fn damage_to_a_player_from_a_source_without_infect_is_life_loss() {
    cr!("120.3", "120.3a");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(giant, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.player(P1).poison(), 0);
}

#[test]
fn damage_to_a_player_from_a_source_with_infect_is_poison_counters() {
    cr!("120.3b");
    let mut t = TestGame::new(2);
    let elf = t.battlefield(P0, "Glistener Elf");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(elf, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.player(P1).poison(), 1);
}

#[test]
fn damage_to_a_planeswalker_removes_loyalty_counters() {
    cr!("120.3c");
    let mut t = TestGame::new(2);
    let jace = t.battlefield(P1, "Jace Beleren");
    t.g.obj_mut(jace)
        .counters
        .insert(counters::LOYALTY.into(), 5);
    bolt(&mut t, Entity::Object(jace));
    assert_eq!(t.counters(jace, counters::LOYALTY), 2);
    assert_eq!(t.obj(jace).damage, 0);
}

#[test]
fn damage_to_a_creature_from_wither_or_infect_is_minus_one_counters() {
    cr!("120.3d");
    let mut t = TestGame::new(2);
    let ram_gang = t.battlefield(P0, "Boggart Ram-Gang");
    let mamba = t.battlefield(P0, "Blight Mamba");
    let wurm = t.battlefield(P1, "Craw Wurm");
    let wurm2 = t.battlefield(P1, "Craw Wurm");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(
        &[(ram_gang, Entity::Player(P1)), (mamba, Entity::Player(P1))],
        &[(wurm, ram_gang), (wurm2, mamba)],
    );
    assert_eq!(t.counters(wurm, counters::MINUS1), 3);
    assert_eq!(t.obj(wurm).damage, 0);
    assert_eq!(t.counters(wurm2, counters::MINUS1), 1);
    assert_eq!(t.pt(wurm), (3, 1));
}

#[test]
fn damage_from_a_source_without_wither_or_infect_is_marked() {
    cr!("120.3e");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(bears, Entity::Player(P1))], &[(wurm, bears)]);
    assert_eq!(t.obj(wurm).damage, 2);
    assert!(t.obj(wurm).counters.is_empty());
}

#[test]
fn lifelink_damage_makes_its_controller_gain_that_much_life() {
    cr!("120.3f");
    let mut t = TestGame::new(2);
    let hawk = t.battlefield(P0, "Vampire Nighthawk");
    let angel = t.battlefield(P1, "Serra Angel");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(hawk, Entity::Player(P1))], &[(angel, hawk)]);
    // In addition to the damage's other results (the blocker was destroyed by deathtouch).
    assert_eq!(t.life(P0), 22);
    assert!(!t.on_battlefield(angel));
}

#[test]
fn toxic_combat_damage_to_a_player_also_gives_poison_counters() {
    cr!("120.3g");
    let mut t = TestGame::new(2);
    let blob = t.battlefield(P0, "Bloated Contaminator");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(blob, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 16);
    assert_eq!(t.player(P1).poison(), 1);
}

#[test]
fn damage_to_a_battle_removes_defense_counters() {
    cr!("120.3h");
    let mut t = TestGame::new(2);
    let battle = t.custom(P1, siege(5), Zone::Battlefield);
    bolt(&mut t, Entity::Object(battle));
    assert_eq!(t.counters(battle, counters::DEFENSE), 2);
}

#[test]
fn damage_is_modified_then_processed_into_results_then_the_event_occurs() {
    cr!("120.4", "120.4b", "120.4c", "120.4d");
    // The example of CR 120.4d: a 3/3 with wither and lifelink attacks and is blocked by a
    // 2/2; the next 2 damage to the blocker is prevented; Boon Reflection doubles life
    // gained.
    let mut t = TestGame::new(2);
    let attacker = t.custom(
        P0,
        CB::new("Withering Leech")
            .creature(3, 3)
            .keyword(KeywordKind::Wither)
            .keyword(KeywordKind::Lifelink)
            .build(),
        Zone::Battlefield,
    );
    let boon = CB::new("Boon Reflection")
        .enchantment()
        .ability(stat(StaticEffect::Replacement(ReplacementDef {
            event: ReplacementEvent::GainLife(PlayerFilter::You),
            action: ReplacementAction::Multiply(2),
            self_replacement: false,
            optional: false,
        })))
        .build();
    t.custom(P0, boon, Zone::Battlefield);
    let blocker = t.custom(
        P1,
        CB::new("Stout Guard")
            .creature(2, 2)
            .ability(trig(
                TriggerCond::IsDealtDamage {
                    filter: Filter::Source,
                    combat_only: false,
                },
                Body::effect(Effect::GainLife {
                    who: PlayerRef::You,
                    n: Value::EventAmount,
                }),
            ))
            .build(),
        Zone::Battlefield,
    );
    let shield = CB::new("Guard")
        .instant()
        .cost("{0}")
        .spell(Body::simple(
            vec![target_creature()],
            Effect::PreventDamage {
                to: Sel::Target(0),
                amount: Some(Value::c(2)),
                duration: Duration::EndOfTurn,
                combat_only: false,
            },
        ))
        .build();
    let s = t.custom(P1, shield, Zone::Hand(P1));
    t.cast(P1, s).target(blocker).go();
    t.resolve();
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(attacker, Entity::Player(P1))], &[(blocker, attacker)]);
    // [3 to the 2/2, 2 to the 3/3] → prevention → [1 to the 2/2, 2 to the 3/3] → results
    // [one -1/-1 counter on the 2/2, gain 1, 2 damage marked on the 3/3] → Boon
    // Reflection → gain 2.
    assert_eq!(t.counters(blocker, counters::MINUS1), 1);
    assert_eq!(t.obj(attacker).damage, 2);
    assert_eq!(t.life(P0), 22);
    // Abilities that trigger on the damage being dealt triggered (the 2/2 was dealt 1).
    assert_eq!(t.life(P1), 21);
}

#[test]
fn excess_damage_is_split_off_before_prevention_and_other_replacements() {
    cr!("120.4a");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    // Damage already marked counts toward lethal damage.
    t.g.obj_mut(bears).damage = 1;
    t.lands(P0, "Mountain", 3);
    let spill = t.hand(P0, "Flame Spill");
    t.cast(P0, spill).target(bears).go();
    t.resolve();
    assert_eq!(t.life(P1), 17);
    assert!(!t.on_battlefield(bears));
    // Excess is determined first: a prevention shield on the creature applies to the
    // damage that remains dealt to it, not to the whole 4.
    let bears2 = t.battlefield(P1, "Grizzly Bears");
    let shield = CB::new("Guard")
        .instant()
        .cost("{0}")
        .spell(Body::simple(
            vec![target_creature()],
            Effect::PreventDamage {
                to: Sel::Target(0),
                amount: Some(Value::c(2)),
                duration: Duration::EndOfTurn,
                combat_only: false,
            },
        ))
        .build();
    let s = t.custom(P1, shield, Zone::Hand(P1));
    t.cast(P1, s).target(bears2).go();
    t.resolve();
    let spill2 = t.hand(P0, "Flame Spill");
    t.lands(P0, "Mountain", 3);
    t.cast(P0, spill2).target(bears2).go();
    t.resolve();
    assert_eq!(t.life(P1), 15);
    assert!(t.on_battlefield(bears2));
    assert_eq!(t.obj(bears2).damage, 0);
    // With a deathtouch source any damage beyond 1 is excess.
    let stinger = CB::new("Venom Spitter")
        .creature(1, 1)
        .keyword(KeywordKind::Deathtouch)
        .ability(act(
            Cost::tap(),
            Body::simple(
                vec![target_creature()],
                Effect::DealDamageExcess {
                    source: Sel::This,
                    amount: Value::c(3),
                    to: Sel::Target(0),
                    excess_to: Sel::Players(PlayerRef::ControllerOf(Box::new(Sel::Target(0)))),
                },
            ),
        ))
        .build();
    let spitter = t.custom(P0, stinger, Zone::Battlefield);
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.activate(P0, spitter, 0, &[Entity::Object(wurm)]).unwrap();
    t.resolve();
    assert_eq!(t.life(P1), 13);
    assert!(!t.on_battlefield(wurm));
}

#[test]
fn damage_doesnt_destroy_state_based_actions_do() {
    cr!("120.5");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 1);
    let b = t.hand(P0, "Lightning Bolt");
    t.cast(P0, b).target(bears).go();
    t.g.resolve_top();
    // Right after the damage, the creature is still on the battlefield with 3 damage.
    assert!(t.on_battlefield(bears));
    assert_eq!(t.obj(bears).damage, 3);
    t.settle();
    assert!(!t.on_battlefield(bears));
    // An indestructible creature with lethal damage stays.
    let knight = t.custom(
        P1,
        CB::new("Stalwart")
            .creature(2, 2)
            .keyword(KeywordKind::Indestructible)
            .build(),
        Zone::Battlefield,
    );
    bolt(&mut t, Entity::Object(knight));
    assert!(t.on_battlefield(knight));
    assert_eq!(t.obj(knight).damage, 3);
}

#[test]
fn marked_damage_remains_until_cleanup_even_if_it_stops_being_a_creature() {
    cr!("120.6");
    let mut t = TestGame::new(2);
    let wurm = t.custom(
        P1,
        CB::new("Old Oak").creature(5, 6).build(),
        Zone::Battlefield,
    );
    t.g.deal_damage(wurm, Entity::Object(wurm), 4, false);
    assert_eq!(t.obj(wurm).damage, 4);
    // It stops being a creature until end of combat; the damage stays marked.
    let unanimate = CB::new("Stillness")
        .instant()
        .cost("{0}")
        .spell(Body::simple(
            vec![target_creature()],
            Effect::Modify {
                what: Sel::Target(0),
                mods: vec![Modification::RemoveTypes(vec![CardType::Creature])],
                duration: Duration::EndOfCombat,
            },
        ))
        .build();
    let s = t.custom(P0, unanimate, Zone::Hand(P0));
    t.cast(P0, s).target(wurm).go();
    t.resolve();
    assert!(!t.obj(wurm).is_creature());
    assert_eq!(t.obj(wurm).damage, 4);
    t.advance_to(P0, Step::PostcombatMain);
    assert!(t.obj(wurm).is_creature());
    assert_eq!(t.obj(wurm).damage, 4);
    // Lethal damage in total: 4 + 2 ≥ 6.
    t.g.deal_damage(wurm, Entity::Object(wurm), 2, false);
    t.settle();
    assert!(!t.on_battlefield(wurm));
    // Damage is removed during the cleanup step.
    let wurm2 = t.custom(
        P1,
        CB::new("Old Oak").creature(5, 6).build(),
        Zone::Battlefield,
    );
    t.g.deal_damage(wurm2, Entity::Object(wurm2), 5, false);
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.obj(wurm2).damage, 0);
    // And when the permanent regenerates.
    t.g.deal_damage(wurm2, Entity::Object(wurm2), 3, false);
    let regen = CB::new("Mend")
        .instant()
        .cost("{0}")
        .spell(Body::simple(
            vec![target_creature()],
            Effect::Regenerate {
                what: Sel::Target(0),
            },
        ))
        .build();
    let r = t.custom(P1, regen, Zone::Hand(P1));
    t.cast(P1, r).target(wurm2).go();
    t.resolve();
    t.g.destroy(wurm2, None);
    assert!(t.on_battlefield(wurm2));
    assert_eq!(t.obj(wurm2).damage, 0);
}

#[test]
fn zero_damage_isnt_dealt_and_isnt_replaced() {
    cr!("120.8");
    let mut t = TestGame::new(2);
    let torbran = CB::new("Flame Warden")
        .enchantment()
        .ability(stat(StaticEffect::Replacement(ReplacementDef {
            event: ReplacementEvent::Damage {
                source: Filter::ControlledBy(PlayerRel::You),
                to_players: Some(PlayerFilter::Opponent),
                to_objects: None,
                combat_only: false,
            },
            action: ReplacementAction::Add(Value::c(2)),
            self_replacement: false,
            optional: false,
        })))
        .build();
    t.custom(P0, torbran, Zone::Battlefield);
    let zero = CB::new("Harmless")
        .creature(0, 1)
        .ability(trig(
            TriggerCond::DealsDamage {
                source: Filter::Source,
                to: DamageRecipient::Any,
                combat_only: false,
            },
            Body::effect(draw(1)),
        ))
        .build();
    let z = t.custom(P0, zero, Zone::Battlefield);
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(z, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.hand_size(P0), 0);
    assert!(damage_events(&t).is_empty());
}

#[test]
fn damage_dealt_refers_only_to_the_specified_source() {
    cr!("120.9");
    let mut t = TestGame::new(2);
    let leech = CB::new("Lifedrinker")
        .creature(2, 2)
        .ability(trig(
            TriggerCond::DealsDamage {
                source: Filter::Source,
                to: DamageRecipient::Player(PlayerRel::Any),
                combat_only: true,
            },
            Body::effect(Effect::GainLife {
                who: PlayerRef::You,
                n: Value::EventAmount,
            }),
        ))
        .build();
    let l = t.custom(P0, leech, Zone::Battlefield);
    let wurm = t.battlefield(P0, "Craw Wurm");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(l, Entity::Player(P1)), (wurm, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 12);
    // Only the 2 damage dealt by the specified source, not the wurm's 6.
    assert_eq!(t.life(P0), 22);
}

#[test]
fn excess_damage_triggers_check_damage_from_all_sources_together() {
    cr!("120.10");
    let mut t = TestGame::new(2);
    let watcher = compile_def(
        "Excess Watcher",
        "Enchantment",
        "{1}{R}",
        "Whenever a creature an opponent controls is dealt excess damage, draw a card.",
    );
    t.custom(P0, watcher, Zone::Battlefield);
    // A 3/3 that can block two attackers blocks two 2/2s: 4 damage from two sources
    // together is 1 more than lethal.
    let wall = CB::new("Twin Warden")
        .creature(3, 3)
        .ability(stat(StaticEffect::Restriction(Restriction::ExtraBlocks {
            blocker: Filter::Source,
            n: Some(1),
        })))
        .build();
    let w = t.custom(P1, wall, Zone::Battlefield);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(
        &[(a, Entity::Player(P1)), (b, Entity::Player(P1))],
        &[(w, a), (w, b)],
    );
    t.resolve_all();
    assert!(!t.on_battlefield(w));
    assert_eq!(t.hand_size(P0), 1);
    let excess = events_matching(&t, |e| matches!(e, Event::ExcessDamage { .. }));
    assert!(matches!(
        excess[..],
        [Event::ExcessDamage {
            amount: 1,
            combat: true,
            ..
        }]
    ));
    // A creature dealt exactly lethal damage wasn't dealt excess damage.
    let bears = t.battlefield(P1, "Grizzly Bears");
    let s = t.custom(
        P0,
        CB::new("Singe")
            .instant()
            .cost("{0}")
            .spell(damage_target(2))
            .build(),
        Zone::Hand(P0),
    );
    t.cast(P0, s).target(bears).go();
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert_eq!(t.hand_size(P0), 1);
}
