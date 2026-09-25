//! CR 122: counters.

use super::r114_common::*;
use mtg_engine::ability::*;
use mtg_engine::events::Event;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn put(t: &mut TestGame, obj: ObjectId, kind: &str, n: u32) {
    t.g.add_counters(Entity::Object(obj), kind, n, None);
    t.g.recompute();
}

fn destroy(t: &mut TestGame, obj: ObjectId) {
    let s = t.custom(P0, destroy_spell(), Zone::Hand(P0));
    t.cast(P0, s).target(obj).go();
}

fn spell(name: &str, targets: Vec<TargetSpec>, e: Effect) -> CardDef {
    CB::new(name)
        .instant()
        .cost("{0}")
        .spell(Body::simple(targets, e))
        .build()
}

fn cast_at(t: &mut TestGame, p: PlayerId, def: CardDef, targets: &[Entity]) {
    let s = t.custom(p, def, Zone::Hand(p));
    t.cast(p, s).targets(targets).go();
    t.resolve();
}

fn destroy_spell() -> CardDef {
    spell(
        "Unmake",
        vec![target_creature()],
        Effect::Destroy {
            what: Sel::Target(0),
            no_regen: false,
        },
    )
}

#[test]
fn counters_are_interchangeable_markers_on_objects_and_players() {
    cr!("122.1");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    // Counters aren't objects: putting them on something creates no object.
    let objects = t.objects.len();
    put(&mut t, bears, counters::PLUS1, 1);
    assert_eq!(t.objects.len(), objects);
    // Counters from two different sources are the same kind of counter.
    cast_at(
        &mut t,
        P0,
        spell(
            "Grow",
            vec![target_creature()],
            Effect::AddCounters {
                what: Sel::Target(0),
                kind: counters::PLUS1.into(),
                n: Value::c(2),
            },
        ),
        &[Entity::Object(bears)],
    );
    assert_eq!(t.counters(bears, counters::PLUS1), 3);
    t.g.remove_counters(Entity::Object(bears), counters::PLUS1, 1);
    assert_eq!(t.counters(bears, counters::PLUS1), 2);
    // Players can have counters too.
    t.g.add_counters(Entity::Player(P1), counters::ENERGY, 2, None);
    assert_eq!(t.player(P1).counter(counters::ENERGY), 2);
}

#[test]
fn pt_counters_modify_power_and_toughness_of_creatures_and_creature_cards() {
    cr!("122.1a");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    put(&mut t, bears, counters::PLUS1, 2);
    put(&mut t, bears, "+1/+0", 1);
    put(&mut t, bears, "-0/-1", 1);
    assert_eq!(t.pt(bears), (5, 3));
    // A creature card in a graveyard with a +1/+1 counter on it.
    let giant = t.graveyard(P0, "Hill Giant");
    t.g.obj_mut(giant)
        .counters
        .insert(counters::PLUS1.into(), 2);
    t.g.recompute();
    assert_eq!(t.pt(giant), (5, 5));
}

#[test]
fn keyword_counters_grant_keywords_to_permanents_and_cards_elsewhere() {
    cr!("122.1b");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    put(&mut t, bears, "flying", 1);
    put(&mut t, bears, "deathtouch", 1);
    assert!(t.obj(bears).has_keyword(KeywordKind::Flying));
    assert!(t.obj(bears).has_keyword(KeywordKind::Deathtouch));
    let exiled = t.exile(P0, "Hill Giant");
    t.g.obj_mut(exiled).counters.insert("vigilance".into(), 1);
    t.g.recompute();
    assert!(t.obj(exiled).has_keyword(KeywordKind::Vigilance));
    // A counter with any other name isn't a keyword counter.
    put(&mut t, bears, "fear", 1);
    assert!(!t.obj(bears).has_keyword(KeywordKind::Fear));
}

#[test]
fn shield_counters_protect_from_destruction_by_effects_and_from_damage() {
    cr!("122.1c");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    put(&mut t, bears, counters::SHIELD, 2);
    // Destroyed as the result of an effect: a shield counter is removed instead.
    cast_at(&mut t, P0, destroy_spell(), &[Entity::Object(bears)]);
    assert!(t.on_battlefield(bears));
    assert_eq!(t.counters(bears, counters::SHIELD), 1);
    // Damage is prevented and one shield counter is removed (the counters create a
    // single prevention effect: only one is removed per damage event).
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(bears).go();
    t.resolve();
    assert!(t.on_battlefield(bears));
    assert_eq!(t.obj(bears).damage, 0);
    assert_eq!(t.counters(bears, counters::SHIELD), 0);
    // Without shield counters, the damage is dealt.
    t.lands(P0, "Mountain", 1);
    let bolt2 = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt2).target(bears).go();
    t.resolve();
    assert!(!t.on_battlefield(bears));
    // Destruction by a state-based action isn't the result of an effect: a creature with
    // a shield counter and lethal damage marked on it is destroyed.
    let wurm = t.battlefield(P1, "Craw Wurm");
    put(&mut t, wurm, counters::SHIELD, 1);
    t.g.obj_mut(wurm).damage = 4;
    t.settle();
    assert!(!t.on_battlefield(wurm));
}

#[test]
fn stun_counters_replace_untapping() {
    cr!("122.1d");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.tap(bears);
    put(&mut t, bears, counters::STUN, 2);
    // An untap effect: a stun counter is removed instead.
    cast_at(
        &mut t,
        P0,
        spell(
            "Wake",
            vec![target_creature()],
            Effect::Untap {
                what: Sel::Target(0),
            },
        ),
        &[Entity::Object(bears)],
    );
    assert!(t.obj(bears).tapped);
    assert_eq!(t.counters(bears, counters::STUN), 1);
    // The untap step: the other stun counter is removed instead.
    t.advance_to(P1, Step::Upkeep);
    assert!(t.obj(bears).tapped);
    assert_eq!(t.counters(bears, counters::STUN), 0);
    t.advance_to(P0, Step::Upkeep);
    t.advance_to(P1, Step::Upkeep);
    assert!(!t.obj(bears).tapped);
}

#[test]
fn loyalty_counters_are_a_planeswalkers_loyalty() {
    cr!("122.1e");
    let mut t = TestGame::new(2);
    let jace = t.battlefield(P0, "Jace Beleren");
    assert_eq!(t.obj(jace).loyalty(), 3);
    put(&mut t, jace, counters::LOYALTY, 2);
    assert_eq!(t.obj(jace).loyalty(), 5);
    t.g.remove_counters(Entity::Object(jace), counters::LOYALTY, 5);
    t.settle();
    assert!(t.in_graveyard(P0, "Jace Beleren"));
}

#[test]
fn ten_poison_counters_lose_the_game_and_poisoned_players() {
    cr!("122.1f");
    let mut t = TestGame::new(3);
    t.g.add_counters(Entity::Player(P1), counters::POISON, 1, None);
    // "Each poisoned opponent loses 2 life."
    cast_at(
        &mut t,
        P0,
        spell(
            "Venom Pulse",
            vec![],
            Effect::LoseLife {
                who: PlayerRef::Each(PlayerFilter::And(vec![
                    PlayerFilter::Opponent,
                    PlayerFilter::Poisoned,
                ])),
                n: Value::c(2),
            },
        ),
        &[],
    );
    assert_eq!((t.life(P1), t.life(P2)), (18, 20));
    t.g.add_counters(Entity::Player(P1), counters::POISON, 8, None);
    t.settle();
    assert!(!t.has_lost(P1));
    t.g.add_counters(Entity::Player(P1), counters::POISON, 1, None);
    t.settle();
    assert!(t.has_lost(P1));
}

#[test]
fn defense_counters_and_a_defeated_battle_waiting_for_its_trigger() {
    cr!("122.1g");
    let mut t = TestGame::new(2);
    let mut cb = CB::new("Test Siege")
        .types(&[CardType::Battle])
        .subtypes(&["Siege"])
        .ability(trig(
            TriggerCond::IsDealtDamage {
                filter: Filter::Source,
                combat_only: false,
            },
            Body::effect(gain(1)),
        ));
    cb.0.defense = Some(3);
    let battle = t.custom(P1, cb.build(), Zone::Battlefield);
    assert_eq!(t.obj(battle).defense(), 3);
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(battle).go();
    t.resolve();
    // Defense 0, but its triggered ability is on the stack: it stays for now.
    assert_eq!(t.obj(battle).defense(), 0);
    assert_eq!(t.stack_len(), 1);
    assert!(t.on_battlefield(battle));
    t.resolve();
    assert!(!t.on_battlefield(battle));
}

#[test]
fn finality_counters_exile_instead_of_graveyard() {
    cr!("122.1h");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    put(&mut t, bears, counters::FINALITY, 1);
    cast_at(&mut t, P0, destroy_spell(), &[Entity::Object(bears)]);
    assert!(t.in_exile("Grizzly Bears"));
    assert!(!t.in_graveyard(P1, "Grizzly Bears"));
    // Also when it dies from a state-based action.
    let giant = t.battlefield(P1, "Hill Giant");
    put(&mut t, giant, counters::FINALITY, 2);
    t.g.obj_mut(giant).damage = 3;
    t.settle();
    assert!(t.in_exile("Hill Giant"));
}

#[test]
fn rad_counters_trigger_at_the_beginning_of_the_precombat_main_phase() {
    cr!("122.1i");
    let mut t = TestGame::new(2);
    t.library_top(P0, "Grizzly Bears");
    t.library_top(P0, "Forest");
    t.library_top(P0, "Hill Giant");
    // (Drawn in P0's next draw step.)
    t.library_top(P0, "Lightning Bolt");
    t.g.add_counters(Entity::Player(P0), counters::RAD, 3, None);
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    // The ability triggered; it's put on the stack before P0 receives priority.
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve();
    // Three cards milled: two nonland cards, so 2 life lost and 2 rad counters removed.
    assert_eq!(t.graveyard_size(P0), 3);
    assert_eq!(t.life(P0), 18);
    assert_eq!(t.player(P0).counter(counters::RAD), 1);
}

#[test]
fn hone_counters_on_equipment_give_the_equipped_creature_plus_one_power() {
    cr!("122.1j");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let sword = t.battlefield(P0, "Bonesplitter");
    t.g.attach(sword, Entity::Object(bears));
    t.g.recompute();
    assert_eq!(t.pt(bears), (4, 2));
    put(&mut t, sword, "hone", 2);
    assert_eq!(t.pt(bears), (6, 2));
    t.g.unattach(sword);
    t.g.recompute();
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn counters_arent_retained_when_an_object_changes_zones() {
    cr!("122.2");
    let mut t = TestGame::new(2);
    let watcher = CB::new("Counter Watcher")
        .enchantment()
        .ability(trig(
            TriggerCond::CountersRemoved {
                filter: Filter::Any,
                kind: None,
            },
            Body::effect(gain(1)),
        ))
        .build();
    t.custom(P0, watcher, Zone::Battlefield);
    let bears = t.battlefield(P0, "Grizzly Bears");
    put(&mut t, bears, counters::PLUS1, 2);
    cast_at(
        &mut t,
        P0,
        spell(
            "Unsummon",
            vec![target_creature()],
            Effect::Move {
                what: Sel::Target(0),
                to: Destination::zone(ZoneKind::Hand),
            },
        ),
        &[Entity::Object(bears)],
    );
    let card = t.g.current(bears);
    assert_eq!(t.zone(card), Zone::Hand(P0));
    assert!(t.obj(card).counters.is_empty());
    // The counters weren't "removed".
    assert!(events_matching(&t, |e| matches!(e, Event::CountersRemoved { .. })).is_empty());
    assert_eq!(t.life(P0), 20);
}

#[test]
fn plus_one_and_minus_one_counters_annihilate() {
    cr!("122.3");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    put(&mut t, bears, counters::PLUS1, 3);
    put(&mut t, bears, counters::MINUS1, 2);
    t.settle();
    assert_eq!(t.counters(bears, counters::PLUS1), 1);
    assert_eq!(t.counters(bears, counters::MINUS1), 0);
    assert_eq!(t.pt(bears), (3, 3));
}

#[test]
fn a_permanent_that_cant_have_more_than_n_counters_loses_the_extra() {
    cr!("122.4");
    let mut t = TestGame::new(2);
    let def = compile_def(
        "Capped Battery",
        "Artifact",
        "{2}",
        "~ can't have more than two charge counters on it.",
    );
    let battery = t.custom(P0, def, Zone::Battlefield);
    put(&mut t, battery, counters::CHARGE, 5);
    t.settle();
    assert_eq!(t.counters(battery, counters::CHARGE), 2);
}

#[test]
fn moving_a_counter_removes_it_from_one_object_and_puts_it_on_another() {
    cr!("122.5");
    let mover = CB::new("Nesting Place")
        .artifact()
        .ability(act(
            Cost::free(),
            Body::simple(
                vec![
                    TargetSpec::object(Filter::Permanent, "target permanent"),
                    TargetSpec::object(Filter::Permanent, "target permanent"),
                ],
                Effect::MoveCounters {
                    from: Sel::Target(0),
                    to: Sel::Target(1),
                    kind: Some(counters::PLUS1.into()),
                    n: Some(Value::c(1)),
                },
            ),
        ))
        .build();
    let mut t = TestGame::new(2);
    let m = t.custom(P0, mover, Zone::Battlefield);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    put(&mut t, a, counters::PLUS1, 2);
    t.activate(P0, m, 0, &[Entity::Object(a), Entity::Object(b)])
        .unwrap();
    t.resolve();
    assert_eq!(t.counters(a, counters::PLUS1), 1);
    assert_eq!(t.counters(b, counters::PLUS1), 1);
    // The same object as both: no counter is removed or put on anything.
    t.activate(P0, m, 0, &[Entity::Object(a), Entity::Object(a)])
        .unwrap();
    t.resolve();
    assert_eq!(t.counters(a, counters::PLUS1), 1);
    // The first object doesn't have that kind of counter.
    t.activate(P0, m, 0, &[Entity::Object(b), Entity::Object(a)])
        .unwrap();
    t.resolve();
    assert_eq!(t.counters(b, counters::PLUS1), 0);
    assert_eq!(t.counters(a, counters::PLUS1), 2);
    // The second object is no longer in the correct zone.
    let gone = t.graveyard(P0, "Craw Wurm");
    let moved = mtg_engine::counter_rules::move_counters(
        &mut t.g,
        a,
        Entity::Object(gone),
        counters::PLUS1,
        1,
    );
    assert_eq!(moved, 0);
    assert_eq!(t.counters(a, counters::PLUS1), 2);
}

#[test]
fn counters_an_object_enters_with_are_counters_put_on_it() {
    cr!("122.6");
    let mut t = TestGame::new(2);
    let watcher = CB::new("Growth Watcher")
        .enchantment()
        .ability(trig(
            TriggerCond::CountersPut {
                filter: Filter::creature().you_control(),
                kind: Some(counters::PLUS1.into()),
            },
            Body::effect(gain(1)),
        ))
        .build();
    t.custom(P0, watcher, Zone::Battlefield);
    // "If one or more +1/+1 counters would be put on a creature you control, that many
    // plus one are put on it instead."
    let scales = CB::new("Scales")
        .enchantment()
        .ability(stat(StaticEffect::Replacement(ReplacementDef {
            event: ReplacementEvent::PutCounters {
                on_objects: Some(Filter::creature().you_control()),
                on_players: None,
                kind: Some(counters::PLUS1.into()),
            },
            action: ReplacementAction::Add(Value::c(1)),
            self_replacement: false,
            optional: false,
        })))
        .build();
    t.custom(P0, scales, Zone::Battlefield);
    t.lands(P0, "Plains", 4);
    let ballista = t.hand(P0, "Walking Ballista");
    t.cast(P0, ballista).x(2).go();
    t.resolve();
    let b = t.named_on_battlefield("Walking Ballista")[0];
    assert_eq!(t.counters(b, counters::PLUS1), 3);
    // The counters it entered with were put on it: the ability triggered.
    assert_eq!(t.stack_len(), 1);
    t.resolve();
    assert_eq!(t.life(P0), 21);
}

#[test]
fn the_controller_of_an_entering_object_puts_its_counters_on_it() {
    cr!("122.6a");
    let mut t = TestGame::new(2);
    // "If you would put one or more counters on a permanent, put twice that many instead."
    let doubler = CB::new("Counter Doubler")
        .enchantment()
        .ability(stat(StaticEffect::Replacement(ReplacementDef {
            event: ReplacementEvent::PutCountersBy {
                by: PlayerRel::You,
                kind: None,
            },
            action: ReplacementAction::Multiply(2),
            self_replacement: false,
            optional: false,
        })))
        .build();
    t.custom(P0, doubler, Zone::Battlefield);
    t.lands(P0, "Plains", 2);
    let mine = t.hand(P0, "Star Pupil");
    t.cast(P0, mine).go();
    t.resolve();
    let pupil = t.named_on_battlefield("Star Pupil")[0];
    // P0 controls the entering Star Pupil, so P0 put its counter on it.
    assert_eq!(t.counters(pupil, counters::PLUS1), 2);
    // An opponent's Star Pupil: its controller (P1) puts the counter on it.
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Plains", 2);
    let theirs = t.hand(P1, "Star Pupil");
    t.cast(P1, theirs).go();
    t.resolve();
    let pupils = t.named_on_battlefield("Star Pupil");
    let their_pupil = *pupils.iter().find(|p| t.obj(**p).controller == P1).unwrap();
    assert_eq!(t.counters(their_pupil, counters::PLUS1), 1);
}

#[test]
fn nth_counter_triggers_when_the_count_reaches_n() {
    cr!("122.7");
    let mut t = TestGame::new(2);
    let def = compile_def(
        "Charging Idol",
        "Artifact",
        "{2}",
        "When the third charge counter is put on ~, draw a card.",
    );
    let idol = t.custom(P0, def, Zone::Battlefield);
    put(&mut t, idol, counters::CHARGE, 2);
    t.settle();
    assert_eq!(t.stack_len(), 0);
    // From 2 to 4: it had fewer than three before and three or more after.
    put(&mut t, idol, counters::CHARGE, 2);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve();
    assert_eq!(t.hand_size(P0), 1);
    put(&mut t, idol, counters::CHARGE, 1);
    t.settle();
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn a_trigger_puts_the_same_counters_a_departed_object_had_on_another() {
    cr!("122.8");
    let mut t = TestGame::new(2);
    let pupil = t.battlefield(P0, "Star Pupil");
    put(&mut t, pupil, counters::PLUS1, 2);
    put(&mut t, pupil, "flying", 1);
    let bears = t.battlefield(P0, "Grizzly Bears");
    destroy(&mut t, pupil);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.resolve_all();
    // The same number of each kind of counter it had.
    assert_eq!(t.counters(bears, counters::PLUS1), 2);
    assert_eq!(t.counters(bears, "flying"), 1);
    // Also with "if it had counters on it, put those counters on ...".
    let apprentice = t.battlefield(P0, "Iron Apprentice");
    put(&mut t, apprentice, counters::PLUS1, 3);
    destroy(&mut t, apprentice);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.resolve_all();
    assert_eq!(t.counters(bears, counters::PLUS1), 5);
}

#[test]
fn a_sacrificed_source_puts_the_same_counters_it_had_on_another_object() {
    cr!("122.9");
    let mut t = TestGame::new(2);
    let def = compile_def(
        "Sapling Peach",
        "Artifact Creature — Plant",
        "{1}",
        "Sacrifice ~: Put its +1/+1 counters onto target creature.",
    );
    let peach = t.custom(P0, def, Zone::Battlefield);
    put(&mut t, peach, counters::PLUS1, 2);
    put(&mut t, peach, counters::CHARGE, 1);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.activate(P0, peach, 0, &[Entity::Object(bears)]).unwrap();
    t.resolve();
    // Only the specified kind; the counters weren't moved (the sacrificed object is gone).
    assert_eq!(t.counters(bears, counters::PLUS1), 2);
    assert_eq!(t.counters(bears, counters::CHARGE), 0);
}
