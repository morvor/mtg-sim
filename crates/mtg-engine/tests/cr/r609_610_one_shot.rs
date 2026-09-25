//! CR 610: one-shot effects — delayed triggers, "until" effects that change zones or phase
//! permanents out, and abilities spells gain as they're cast.

use crate::r609_common::*;
use mtg_engine::ability::*;
use mtg_engine::events::MoveCause;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn opp_creature() -> TargetSpec {
    TargetSpec::object(
        Filter::creature().opp_controls(),
        "target creature an opponent controls",
    )
}

/// Banisher Priest: "When this creature enters, exile target creature an opponent
/// controls until this creature leaves the battlefield."
fn banisher() -> CardDef {
    creature_with(
        "Banisher Priest",
        2,
        2,
        &[Color::White],
        vec![AbilityDef::new(
            AbilityKind::Triggered(TriggeredAbility::new(
                TriggerCond::EntersBattlefield(Filter::Source),
                Body::simple(
                    vec![opp_creature()],
                    Effect::ExileUntil {
                        what: Sel::Target(0),
                        until: UntilEvent::SourceLeavesBattlefield,
                    },
                ),
            )),
            "When this creature enters, exile target creature an opponent controls until this creature leaves the battlefield.",
        )],
    )
}

/// Oubliette: "When this enters, target creature phases out until this leaves the
/// battlefield."
fn oubliette() -> CardDef {
    permanent(
        "Oubliette",
        &[CardType::Enchantment],
        vec![AbilityDef::new(
            AbilityKind::Triggered(TriggeredAbility::new(
                TriggerCond::EntersBattlefield(Filter::Source),
                Body::simple(
                    vec![target_creature()],
                    Effect::PhaseOutUntil {
                        what: Sel::Target(0),
                        until: UntilEvent::SourceLeavesBattlefield,
                    },
                ),
            )),
            "When this enters, target creature phases out until this leaves the battlefield.",
        )],
    )
}

fn cast_permanent(t: &mut TestGame, p: PlayerId, def: CardDef, targets: &[Entity]) -> ObjectId {
    let c = t.custom(p, def, Zone::Hand(p));
    t.cast_with(p, c, &[]).unwrap();
    t.resolve(); // the permanent spell; its ETB trigger goes on the stack
    for e in targets {
        let _ = e;
    }
    t.g.current(c)
}

fn kill(t: &mut TestGame, id: ObjectId) {
    let id = t.g.current(id);
    t.g.destroy(id, None);
    t.settle();
}

#[test]
fn one_shot_effects_happen_once() {
    // CR 610.1: a one-shot effect does something just once and has no duration: the
    // damage stays marked, the token stays, a creature that enters later isn't destroyed.
    cr!("610.1", "609.1");
    let mut t = TestGame::new(2);
    let wall = t.custom(P1, creature("Wall", 0, 5, &[]), Zone::Battlefield);
    resolve_effect(
        &mut t,
        P0,
        vec![target_creature()],
        &[Entity::Object(wall)],
        Effect::seq(vec![
            Effect::DealDamage {
                source: Sel::This,
                amount: Value::c(2),
                to: Sel::Target(0),
            },
            Effect::CreateToken {
                spec: TokenSpec {
                    name: "Soldier".into(),
                    colors: colors(&[Color::White]),
                    supertypes: vec![],
                    card_types: vec![CardType::Creature],
                    subtypes: vec!["Soldier".into()],
                    power: Some(1),
                    toughness: Some(1),
                    abilities: vec![],
                    scryfall_name: None,
                },
                count: Value::c(1),
                controller: PlayerRef::You,
                tapped: false,
                attacking: false,
            },
            Effect::Destroy {
                what: Sel::All(Filter::Named("Bear".into())),
                no_regen: false,
            },
        ]),
    );
    assert_eq!(t.obj_now(wall).damage, 2);
    assert_eq!(t.named_on_battlefield("Soldier").len(), 1);
    let bear = t.custom(P1, creature("Bear", 2, 2, &[]), Zone::Battlefield);
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.named_on_battlefield("Soldier").len(), 1);
    assert!(t.on_battlefield(bear));
}

#[test]
fn one_shot_effects_can_create_delayed_triggers() {
    // CR 610.2 (Otherworldly Journey): "Exile target creature. At the beginning of the next
    // end step, return that card to the battlefield under its owner's control with a +1/+1
    // counter on it."
    cr!("610.2");
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P1, "Grizzly Bears");
    resolve_effect(
        &mut t,
        P0,
        vec![target_creature()],
        &[Entity::Object(bear)],
        Effect::seq(vec![
            Effect::Exile {
                what: Sel::Target(0),
                face_down: false,
                link: false,
            },
            Effect::AtNext {
                step: TriggerStep::End,
                effect: Box::new(Effect::Move {
                    what: Sel::Var(vars::IT),
                    to: Destination {
                        controller: Some(PlayerRef::OwnerOf(Box::new(Sel::Var(vars::IT)))),
                        with_counters: vec![(counters::PLUS1.into(), Value::c(1))],
                        ..Destination::battlefield()
                    },
                }),
            },
        ]),
    );
    assert!(t.in_exile("Grizzly Bears"));
    t.advance_to(P0, Step::End);
    t.resolve_all();
    let b = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(b.len(), 1);
    assert_eq!(t.obj_now(b[0]).controller, P1);
    assert_eq!(t.pt(b[0]), (3, 3));
}

#[test]
fn exile_until_the_source_leaves() {
    // CR 610.3: the card returns immediately after the priest leaves the battlefield (not
    // via the stack). CR 610.3c: under its owner's control.
    cr!("610.3", "610.3c");
    ruling!("Banisher Priest", "When the card returns to the battlefield, it will be a new object");
    ruling!("Oubliette", "won't happen when a permanent phases out");
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P1, "Grizzly Bears");
    // P0 has stolen P1's other creature; it returns under its owner's control.
    let giant = t.battlefield(P1, "Hill Giant");
    resolve_effect(
        &mut t,
        P0,
        vec![target_creature()],
        &[Entity::Object(giant)],
        Effect::GainControl {
            what: Sel::Target(0),
            who: PlayerRef::You,
            duration: Duration::Permanent,
        },
    );
    assert_eq!(t.obj_now(giant).controller, P0);
    // P1's Banisher Priest exiles the stolen Hill Giant.
    t.set_step(P1, Step::PrecombatMain);
    t.answer_targets(P1, &[Entity::Object(giant)]);
    let priest = cast_permanent(&mut t, P1, banisher(), &[]);
    t.resolve();
    assert!(t.in_exile("Hill Giant"));
    let _ = bear;
    // Phasing out isn't leaving the battlefield.
    let pr = t.g.current(priest);
    mtg_engine::keyword_impls::phase_out(&mut t.g, vec![pr]);
    t.g.flush_events();
    assert!(t.in_exile("Hill Giant"));
    mtg_engine::keyword_impls::phase_in(&mut t.g, pr);
    kill(&mut t, priest);
    let g = t.named_on_battlefield("Hill Giant");
    assert_eq!(g.len(), 1);
    assert_eq!(t.obj_now(g[0]).controller, P1);
    // It happened right away, not as an ability on the stack.
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn until_event_before_a_spell_or_activated_ability_resolves() {
    // CR 610.3a: an activated ability "exile target creature until this artifact leaves
    // the battlefield": if the artifact leaves before the ability resolves (after it was
    // activated), the creature isn't exiled.
    cr!("610.3a", "610.4b");
    for phase in [false, true] {
        let mut t = TestGame::new(2);
        let bear = t.battlefield(P1, "Grizzly Bears");
        let effect = if phase {
            Effect::PhaseOutUntil {
                what: Sel::Target(0),
                until: UntilEvent::SourceLeavesBattlefield,
            }
        } else {
            Effect::ExileUntil {
                what: Sel::Target(0),
                until: UntilEvent::SourceLeavesBattlefield,
            }
        };
        let prison = t.custom(
            P0,
            permanent(
                "Prison",
                &[CardType::Artifact],
                vec![activated(
                    Cost::free(),
                    Body::simple(vec![target_creature()], effect),
                )],
            ),
            Zone::Battlefield,
        );
        t.activate(P0, prison, 0, &[Entity::Object(bear)]).unwrap();
        kill(&mut t, prison);
        t.resolve();
        assert!(t.on_battlefield(bear));
        assert!(!t.obj_now(bear).phased_out);
        assert!(t.g.untils.is_empty());
    }
}

#[test]
fn until_event_before_a_triggered_ability_resolves() {
    // CR 610.3b / 610.4c: Banisher Priest leaves before its ability resolves: the creature
    // isn't exiled (or phased out).
    cr!("610.3b", "610.4c");
    ruling!("Banisher Priest", "If Banisher Priest leaves the battlefield before its triggered ability resolves");
    ruling!("Oubliette", "If Oubliette leaves the battlefield before its triggered ability resolves");
    for phase in [false, true] {
        let mut t = TestGame::new(2);
        let bear = t.battlefield(P1, "Grizzly Bears");
        t.answer_targets(P0, &[Entity::Object(bear)]);
        let src = if phase {
            cast_permanent(&mut t, P0, oubliette(), &[])
        } else {
            cast_permanent(&mut t, P0, banisher(), &[])
        };
        assert_eq!(t.stack_len(), 1);
        kill(&mut t, src);
        t.resolve();
        assert!(t.on_battlefield(bear));
        assert!(!t.obj_now(bear).phased_out);
    }
}

#[test]
fn simultaneous_returns_are_simultaneous() {
    // CR 610.3d example: two Banisher Priests have each exiled a card; Day of Judgment
    // destroys both; the two exiled cards return at the same time — each sees the other
    // enter.
    cr!("610.3d");
    let mut t = TestGame::new(2);
    let watcher = |name: &str| {
        creature_with(
            name,
            1,
            1,
            &[],
            vec![triggered(
                TriggerCond::EntersBattlefield(Filter::creature().other()),
                Effect::GainLife {
                    who: PlayerRef::You,
                    n: Value::c(1),
                },
            )],
        )
    };
    let a = t.custom(P1, watcher("Watcher A"), Zone::Battlefield);
    let b = t.custom(P1, watcher("Watcher B"), Zone::Battlefield);
    t.answer_targets(P0, &[Entity::Object(a)]);
    let p1 = cast_permanent(&mut t, P0, banisher(), &[]);
    t.resolve_all();
    t.answer_targets(P0, &[Entity::Object(b)]);
    let p2 = cast_permanent(&mut t, P0, banisher(), &[]);
    t.resolve_all();
    assert!(t.in_exile("Watcher A") && t.in_exile("Watcher B"));
    let life = t.life(P1);
    t.lands(P0, "Plains", 4);
    t.g.destroy_all(vec![t.g.current(p1), t.g.current(p2)], None, false);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Watcher A").len(), 1);
    assert_eq!(t.named_on_battlefield("Watcher B").len(), 1);
    assert_eq!(t.life(P1), life + 2);
}

#[test]
fn phase_out_until_the_source_leaves() {
    // CR 610.4: the creature phases in right after Oubliette leaves the battlefield.
    // CR 610.4a: it doesn't phase in during its controller's untap step; if another effect
    // phases it in, the second one-shot effect doesn't happen even if it phases out again.
    cr!("610.4", "610.4a");
    ruling!("Oubliette", "it phases in immediately after Oubliette leaves the battlefield");
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bear)]);
    let oub = cast_permanent(&mut t, P0, oubliette(), &[]);
    t.resolve();
    assert!(t.obj_now(bear).phased_out);
    t.advance_to(P1, Step::Upkeep);
    assert!(t.obj_now(bear).phased_out);
    t.advance_to(P0, Step::PrecombatMain);
    kill(&mut t, oub);
    assert!(!t.obj_now(bear).phased_out);

    // Phased in by another effect, then out again: Oubliette leaving doesn't phase it in.
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bear)]);
    let oub = cast_permanent(&mut t, P0, oubliette(), &[]);
    t.resolve();
    mtg_engine::keyword_impls::phase_in(&mut t.g, bear);
    mtg_engine::keyword_impls::phase_out(&mut t.g, vec![bear]);
    kill(&mut t, oub);
    assert!(t.obj_now(bear).phased_out);
    // Now it phases in normally during its controller's untap step.
    t.advance_to(P1, Step::Upkeep);
    assert!(!t.obj_now(bear).phased_out);
}

#[test]
fn simultaneous_phase_ins_are_simultaneous() {
    // CR 610.4d: two Oubliettes destroyed at the same time phase their creatures in at the
    // same time.
    cr!("610.4d");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[Entity::Object(a)]);
    let o1 = cast_permanent(&mut t, P0, oubliette(), &[]);
    t.resolve();
    t.answer_targets(P0, &[Entity::Object(b)]);
    let o2 = cast_permanent(&mut t, P0, oubliette(), &[]);
    t.resolve();
    let before = t.g.events.len();
    t.g.destroy_all(vec![t.g.current(o1), t.g.current(o2)], None, false);
    t.g.flush_events();
    assert!(!t.obj_now(a).phased_out && !t.obj_now(b).phased_out);
    let _ = before;
    assert!(t.g.untils.is_empty());
}

#[test]
fn spells_gain_abilities_as_theyre_cast() {
    // CR 610.5: "Red instant spells you cast have lifelink" makes a spell gain lifelink as
    // it's cast (a one-shot effect): it keeps it after the source leaves. A spell cast
    // before the source existed doesn't gain it.
    cr!("610.5");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 2);
    let early = t.hand(P0, "Lightning Bolt");
    t.cast(P0, early).target(P1).go();
    let source = t.custom(
        P0,
        permanent(
            "Lifelink Lens",
            &[CardType::Enchantment],
            vec![continuous(
                Filter::and(vec![
                    Filter::Spell,
                    Filter::Type(CardType::Instant),
                    Filter::Color(Color::Red),
                    Filter::ControlledBy(PlayerRel::You),
                ]),
                vec![Modification::AddKeyword(mtg_engine::keywords::Keyword::new(
                    KeywordKind::Lifelink,
                ))],
            )],
        ),
        Zone::Battlefield,
    );
    let bolt = t.hand(P0, "Lightning Bolt");
    let spell = t.cast(P0, bolt).target(P1).go();
    assert!(t.obj_now(spell).has_keyword(KeywordKind::Lifelink));
    t.g.move_object(source, Zone::Graveyard(P0), MoveCause::Effect, None);
    t.recompute();
    assert!(t.obj_now(spell).has_keyword(KeywordKind::Lifelink));
    t.resolve();
    assert_eq!(t.life(P0), 23);
    t.resolve();
    assert_eq!(t.life(P0), 23);
    assert_eq!(t.life(P1), 14);
}
