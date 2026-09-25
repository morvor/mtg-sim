//! CR 611: continuous effects — from resolving spells and abilities (durations, locked
//! sets, variables, effects applying as permanents enter, "next spell" effects) and from
//! static abilities.

use crate::r609_common::*;
use mtg_engine::ability::*;
use mtg_engine::events::MoveCause;
use mtg_engine::keywords::{Keyword, KeywordKind};
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn continuous_effects_modify_objects_control_players_and_rules() {
    // CR 611.1: continuous effects modify characteristics, control, players, or the rules,
    // for a period. CR 611.2, 611.2a: from a resolving spell they last as stated ("until
    // end of turn"), or until the end of the game if no duration is stated.
    cr!("611.1", "611.2", "611.2a");
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    modify_target(&mut t, P0, bear, vec![pt(2, 2)]);
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
    resolve_effect(
        &mut t,
        P0,
        vec![],
        &[],
        Effect::AddPlayerEffect {
            who: PlayerRef::You,
            effect: PlayerModification::MaxHandSize(None),
            duration: Duration::EndOfTurn,
        },
    );
    resolve_effect(
        &mut t,
        P0,
        vec![],
        &[],
        Effect::AddRestriction {
            restriction: Restriction::CantGainLife(PlayerFilter::Opponent),
            duration: Duration::EndOfTurn,
        },
    );
    assert_eq!(t.pt(bear), (4, 4));
    assert_eq!(t.obj_now(giant).controller, P0);
    assert_eq!(t.player(P0).max_hand_size, None);
    t.g.gain_life(P1, 2);
    assert_eq!(t.life(P1), 20);
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.pt(bear), (2, 2));
    assert_eq!(t.player(P0).max_hand_size, Some(7));
    t.g.gain_life(P1, 2);
    assert_eq!(t.life(P1), 22);
    // No duration: the control change lasts.
    assert_eq!(t.obj_now(giant).controller, P0);
}

#[test]
fn for_as_long_as_that_never_starts_does_nothing() {
    // CR 611.2b example: Master Thief ("gain control of target artifact for as long as you
    // control this creature") leaves before its ability resolves: nothing happens.
    cr!("611.2b");
    ruling!(
        "Master Thief",
        "If Master Thief ceases to be under your control before its ability resolves"
    );
    let mut t = TestGame::new(2);
    let relic = t.custom(
        P1,
        permanent("Relic", &[CardType::Artifact], vec![]),
        Zone::Battlefield,
    );
    t.lands(P0, "Island", 4);
    let thief = t.hand(P0, "Master Thief");
    t.answer_targets(P0, &[Entity::Object(relic)]);
    t.cast(P0, thief).go();
    t.resolve();
    assert_eq!(t.stack_len(), 1);
    let thief = t.named_on_battlefield("Master Thief")[0];
    t.g.move_object(thief, Zone::Hand(P0), MoveCause::Effect, None);
    t.resolve();
    assert_eq!(t.obj_now(relic).controller, P1);
    assert!(t.g.effects.is_empty());
    // With Master Thief staying, the artifact is stolen until it leaves.
    let thief = t.hand(P0, "Master Thief");
    t.lands(P0, "Island", 4);
    t.answer_targets(P0, &[Entity::Object(relic)]);
    t.cast(P0, thief).go();
    t.resolve_all();
    assert_eq!(t.obj_now(relic).controller, P0);
}

#[test]
fn affected_set_locked_for_characteristics_but_not_rules() {
    // CR 611.2c: "All white creatures get +1/+1 until end of turn" affects the white
    // creatures at resolution, even if they change color, and not ones that become white
    // or enter later. "Prevent all damage creatures would deal this turn" modifies the
    // rules and applies to creatures that enter later and permanents that become creatures.
    cr!("611.2c");
    let mut t = TestGame::new(2);
    let w = t.custom(
        P0,
        creature("White A", 2, 2, &[Color::White]),
        Zone::Battlefield,
    );
    let g = t.custom(
        P0,
        creature("Green B", 2, 2, &[Color::Green]),
        Zone::Battlefield,
    );
    resolve_effect(
        &mut t,
        P0,
        vec![],
        &[],
        Effect::Modify {
            what: Sel::All(Filter::and(vec![
                Filter::creature(),
                Filter::Color(Color::White),
            ])),
            mods: vec![pt(1, 1)],
            duration: Duration::EndOfTurn,
        },
    );
    modify_target(
        &mut t,
        P0,
        w,
        vec![Modification::SetColors(colors(&[Color::Blue]))],
    );
    modify_target(
        &mut t,
        P0,
        g,
        vec![Modification::SetColors(colors(&[Color::White]))],
    );
    let late = t.custom(
        P0,
        creature("White C", 2, 2, &[Color::White]),
        Zone::Battlefield,
    );
    t.recompute();
    assert_eq!(t.pt(w), (3, 3));
    assert_eq!(t.pt(g), (2, 2));
    assert_eq!(t.pt(late), (2, 2));

    resolve_effect(
        &mut t,
        P1,
        vec![],
        &[],
        Effect::AddReplacement {
            def: ReplacementDef {
                event: ReplacementEvent::Damage {
                    source: Filter::creature(),
                    to_players: Some(PlayerFilter::Any),
                    to_objects: Some(Filter::Any),
                    combat_only: false,
                },
                action: ReplacementAction::Prevent,
                self_replacement: false,
                optional: false,
            },
            duration: Duration::EndOfTurn,
            uses: None,
        },
    );
    let newcomer = t.battlefield(P0, "Hill Giant");
    t.g.deal_damage(newcomer, Entity::Player(P1), 3, false);
    assert_eq!(t.life(P1), 20);
    let relic = t.custom(
        P0,
        permanent("Relic", &[CardType::Artifact], vec![]),
        Zone::Battlefield,
    );
    t.g.deal_damage(relic, Entity::Player(P1), 1, false);
    assert_eq!(t.life(P1), 19);
    resolve_effect(
        &mut t,
        P0,
        vec![TargetSpec::object(
            Filter::Type(CardType::Artifact),
            "target artifact",
        )],
        &[Entity::Object(relic)],
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![
                Modification::AddTypes(vec![CardType::Creature]),
                set_pt(1, 1),
            ],
            duration: Duration::EndOfTurn,
        },
    );
    t.g.deal_damage(relic, Entity::Player(P1), 1, false);
    assert_eq!(t.life(P1), 19);
}

#[test]
fn parts_of_one_effect_lock_their_sets_independently() {
    // CR 611.2c: "Creatures you control get +1/+1 and can't be blocked this turn": the P/T
    // part is locked in; the "can't be blocked" part modifies the rules and applies to a
    // creature that enters later.
    cr!("611.2c");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    resolve_effect(
        &mut t,
        P0,
        vec![],
        &[],
        Effect::seq(vec![
            Effect::Modify {
                what: Sel::All(Filter::creature().you_control()),
                mods: vec![pt(1, 1)],
                duration: Duration::EndOfTurn,
            },
            Effect::AddRestriction {
                restriction: Restriction::CantBeBlocked(Filter::creature().you_control()),
                duration: Duration::EndOfTurn,
            },
        ]),
    );
    let b = t.battlefield(P0, "Hill Giant");
    t.recompute();
    assert_eq!(t.pt(a), (3, 3));
    assert_eq!(t.pt(b), (3, 3));
    let blocker = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(b, Entity::Player(P1))], &[(blocker, b)]);
    assert_eq!(t.life(P1), 17);
}

#[test]
fn variables_are_fixed_on_resolution() {
    // CR 611.2d: "Target creature gets +X/+X until end of turn, where X is the number of
    // creatures you control" — X is determined once, as it resolves.
    cr!("611.2d");
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Hill Giant");
    let x = Value::Count(Filter::creature().you_control());
    modify_target(&mut t, P0, bear, vec![Modification::ModifyPT(x.clone(), x)]);
    assert_eq!(t.pt(bear), (4, 4));
    t.battlefield(P0, "Hill Giant");
    t.recompute();
    assert_eq!(t.pt(bear), (4, 4));
}

fn zombie_watcher() -> CardDef {
    permanent(
        "Zombie Watcher",
        &[CardType::Enchantment],
        vec![triggered(
            TriggerCond::EntersBattlefield(Filter::Subtype("Zombie".into())),
            Effect::GainLife {
                who: PlayerRef::You,
                n: Value::c(1),
            },
        )],
    )
}

#[test]
fn is_characteristics_apply_as_the_permanent_enters() {
    // CR 611.2e: "Return target creature card to the battlefield. That creature is a black
    // Zombie in addition to its other colors and types." applies as it enters — a "Whenever
    // a Zombie enters" ability triggers. A "becomes" effect applies only afterward.
    cr!("611.2e");
    let mut t = TestGame::new(2);
    t.custom(P0, zombie_watcher(), Zone::Battlefield);
    let bear = t.graveyard(P0, "Grizzly Bears");
    let target_card = TargetSpec::object(
        Filter::and(vec![
            Filter::creature(),
            Filter::InZone(ZoneKind::Graveyard),
        ]),
        "target creature card",
    );
    resolve_effect(
        &mut t,
        P0,
        vec![target_card.clone()],
        &[Entity::Object(bear)],
        Effect::Move {
            what: Sel::Target(0),
            to: Destination {
                with_mods: vec![
                    Modification::AddSubtypes(vec!["Zombie".into()]),
                    Modification::AddColors(colors(&[Color::Black])),
                ],
                ..Destination::battlefield()
            },
        },
    );
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
    let b = t.named_on_battlefield("Grizzly Bears")[0];
    assert!(t.obj_now(b).chars.has_subtype("Zombie"));
    assert!(t.obj_now(b).chars.colors.contains(Color::Black));

    // "becomes a Zombie" applies after the permanent is on the battlefield: it didn't
    // enter as a Zombie.
    let giant = t.graveyard(P0, "Hill Giant");
    resolve_effect(
        &mut t,
        P0,
        vec![target_card],
        &[Entity::Object(giant)],
        Effect::Move {
            what: Sel::Target(0),
            to: Destination::battlefield(),
        },
    );
    t.resolve_all();
    let g = t.named_on_battlefield("Hill Giant")[0];
    modify_target(
        &mut t,
        P0,
        g,
        vec![Modification::AddSubtypes(vec!["Zombie".into()])],
    );
    t.resolve_all();
    assert!(t.obj_now(g).chars.has_subtype("Zombie"));
    assert_eq!(t.life(P0), 21);
}

#[test]
fn own_static_abilities_are_older_than_the_effect_that_puts_it_onto_the_battlefield() {
    // CR 613.7n: an object's own static ability and a resolving effect that puts it onto
    // the battlefield and sets a characteristic get simultaneous timestamps; the static
    // ability's is earlier. "Creatures you control are black" (its own) and "That creature
    // is white" (the resolving effect): it's white.
    cr!("613.7n", "611.2e");
    let mut t = TestGame::new(2);
    let c = t.custom(
        P0,
        creature_with(
            "Shade Lord",
            2,
            2,
            &[Color::Black],
            vec![continuous(
                Filter::creature().you_control(),
                vec![Modification::SetColors(colors(&[Color::Black]))],
            )],
        ),
        Zone::Graveyard(P0),
    );
    resolve_effect(
        &mut t,
        P0,
        vec![TargetSpec::object(
            Filter::and(vec![
                Filter::creature(),
                Filter::InZone(ZoneKind::Graveyard),
            ]),
            "target creature card",
        )],
        &[Entity::Object(c)],
        Effect::Move {
            what: Sel::Target(0),
            to: Destination {
                with_mods: vec![Modification::SetColors(colors(&[Color::White]))],
                ..Destination::battlefield()
            },
        },
    );
    let s = t.named_on_battlefield("Shade Lord")[0];
    assert_eq!(color_of(&t, s), colors(&[Color::White]));
    // Other creatures still get its effect.
    let bear = t.battlefield(P0, "Grizzly Bears");
    t.recompute();
    assert_eq!(color_of(&t, bear), colors(&[Color::Black]));
}

#[test]
fn next_spell_effects_begin_when_that_spell_is_cast() {
    // CR 611.2f: "The next instant spell you cast this turn has lifelink" doesn't apply to
    // a spell already on the stack; it applies to the next matching spell as it's put on
    // the stack, and only that one.
    cr!("611.2f");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 3);
    let first = t.hand(P0, "Lightning Bolt");
    let s1 = t.cast(P0, first).target(P1).go();
    let mut ctx = mtg_engine::eval::Ctx::new(None, P0);
    t.g.exec(
        &Effect::NextSpell {
            filter: Filter::Type(CardType::Instant),
            mods: vec![Modification::AddKeyword(Keyword::new(
                KeywordKind::Lifelink,
            ))],
            expires: Duration::EndOfTurn,
        },
        &mut ctx,
    );
    t.recompute();
    assert!(!t.obj_now(s1).has_keyword(KeywordKind::Lifelink));
    let second = t.hand(P0, "Lightning Bolt");
    let s2 = t.cast(P0, second).target(P1).go();
    assert!(t.obj_now(s2).has_keyword(KeywordKind::Lifelink));
    let third = t.hand(P0, "Lightning Bolt");
    let s3 = t.cast(P0, third).target(P1).go();
    assert!(!t.obj_now(s3).has_keyword(KeywordKind::Lifelink));
    t.resolve_all();
    assert_eq!(t.life(P0), 23);
    assert_eq!(t.life(P1), 11);
}

#[test]
fn static_effects_apply_to_permanents_as_they_enter() {
    // CR 611.3, 611.3c: static effects modifying characteristics apply as a permanent
    // enters: with an anthem, a 1/1 enters as a 2/2, and "whenever a creature with power 2
    // or greater enters" triggers.
    cr!("611.3", "611.3c");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Glorious Anthem");
    t.custom(
        P0,
        permanent(
            "Big Watch",
            &[CardType::Enchantment],
            vec![triggered(
                TriggerCond::EntersBattlefield(Filter::and(vec![
                    Filter::creature(),
                    Filter::Power(Cmp::Ge, Box::new(Value::c(2))),
                ])),
                Effect::GainLife {
                    who: PlayerRef::You,
                    n: Value::c(1),
                },
            )],
        ),
        Zone::Battlefield,
    );
    let g = t.enter(P0, "Raging Goblin");
    t.resolve_all();
    assert_eq!(t.pt(g), (2, 2));
    assert_eq!(t.life(P0), 21);
}

#[test]
fn static_effects_function_only_where_their_object_is() {
    // CR 611.3b: the effect applies while the object generating it is on the battlefield
    // (or in the zone where its ability functions); it isn't locked in (CR 611.3a).
    cr!("611.3b", "611.3a");
    let mut t = TestGame::new(2);
    let anthem = t.battlefield(P0, "Glorious Anthem");
    let bear = t.battlefield(P0, "Grizzly Bears");
    assert_eq!(t.pt(bear), (3, 3));
    t.g.move_object(anthem, Zone::Graveyard(P0), MoveCause::Effect, None);
    t.recompute();
    assert_eq!(t.pt(bear), (2, 2));
    // An ability that functions from the graveyard applies while its card is there.
    let mut s = StaticAbility::new(StaticEffect::Continuous {
        affected: Filter::creature().you_control(),
        mods: vec![pt(0, 3)],
    });
    s.zone = FunctionZone::Graveyard;
    let spirit = t.custom(
        P0,
        permanent(
            "Ancestral Spirit",
            &[CardType::Creature],
            vec![AbilityDef::new(AbilityKind::Static(s), "static")],
        ),
        Zone::Graveyard(P0),
    );
    t.recompute();
    assert_eq!(t.pt(bear), (2, 5));
    t.g.move_object(spirit, Zone::Exile, MoveCause::Effect, None);
    t.recompute();
    assert_eq!(t.pt(bear), (2, 2));
}

#[test]
fn abilities_granted_with_a_casting_permission_last() {
    // CR 611.3d: a static ability lets you cast creature spells from your graveyard, and a
    // spell cast this way gains haste. The granted ability lasts until the end of the game
    // (no duration stated), even after the source leaves the battlefield.
    cr!("611.3d");
    let mut t = TestGame::new(2);
    let paragon = t.custom(
        P0,
        creature_with(
            "Graveyard Paragon",
            3,
            3,
            &[Color::White],
            vec![
                static_ab(StaticEffect::PlayPermission(PlayPermission {
                    who: PlayerRel::You,
                    zone: ZoneKind::Graveyard,
                    top_only: false,
                    what: Filter::creature(),
                    lands: false,
                    spells: true,
                    cost: None,
                })),
                static_ab(StaticEffect::CastGrant {
                    zone: ZoneKind::Graveyard,
                    what: Filter::creature(),
                    mods: vec![Modification::AddKeyword(Keyword::new(KeywordKind::Haste))],
                }),
            ],
        ),
        Zone::Battlefield,
    );
    t.lands(P0, "Forest", 2);
    let bear = t.graveyard(P0, "Grizzly Bears");
    t.cast(P0, bear).go();
    t.resolve();
    let b = t.named_on_battlefield("Grizzly Bears")[0];
    assert!(has_kw(&t, b, KeywordKind::Haste));
    t.g.move_object(paragon, Zone::Graveyard(P0), MoveCause::Effect, None);
    t.recompute();
    assert!(has_kw(&t, b, KeywordKind::Haste));
    t.advance_to(P1, Step::Upkeep);
    assert!(has_kw(&t, b, KeywordKind::Haste));
}
