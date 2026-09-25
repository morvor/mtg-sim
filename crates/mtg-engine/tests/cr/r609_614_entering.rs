//! CR 614.12–614.14: replacement effects that modify how permanents enter the
//! battlefield, objects that change zones while applying them, and linked exile.

use crate::r609_common::*;
use mtg_engine::ability::*;
use mtg_engine::events::MoveCause;
use mtg_engine::object::*;
use mtg_engine::replacement::{EtbInfo, MoveEv};
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

fn to_battlefield(obj: ObjectId, p: PlayerId) -> MoveEv {
    MoveEv {
        obj,
        to: Zone::Battlefield,
        pos: LibraryPosition::Top,
        cause: MoveCause::Effect,
        by: Some(p),
        etb: EtbInfo {
            controller: Some(p),
            ..Default::default()
        },
        source: None,
    }
}

fn voice_of_all() -> CardDef {
    let mut prot =
        mtg_engine::keywords::Keyword::new(mtg_engine::keywords::KeywordKind::Protection);
    prot.filter = Some(Filter::Color(Color::Red));
    creature_with(
        "Voice of All",
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
    )
}

#[test]
fn a_token_copy_makes_its_own_as_enters_choice() {
    // CR 614.12 example: a token that's a copy of Voice of All ("As this creature enters,
    // choose a color") has that ability as it would exist on the battlefield, so its
    // controller chooses a color for it as it's created.
    cr!("614.12", "614.12a");
    let mut t = TestGame::new(2);
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    let voice = t.custom(P0, voice_of_all(), Zone::Hand(P0));
    t.cast_with(P0, voice, &[]).unwrap();
    t.resolve();
    let voice = t.named_on_battlefield("Voice of All")[0];
    assert_eq!(t.obj_now(voice).choices.color, Some(Color::White));
    t.answer(P0, DecisionKind::Option, Answer::Index(4));
    resolve_effect(
        &mut t,
        P0,
        vec![target_creature()],
        &[Entity::Object(voice)],
        Effect::CreateTokenCopy {
            of: Sel::Target(0),
            count: Value::c(1),
            controller: PlayerRef::You,
            tapped: false,
            attacking: false,
            mods: vec![],
        },
    );
    let voices = t.named_on_battlefield("Voice of All");
    assert_eq!(voices.len(), 2);
    let token = voices
        .into_iter()
        .find(|v| t.obj_now(*v).is_token())
        .unwrap();
    assert_eq!(t.obj_now(token).choices.color, Some(Color::Green));
}

#[test]
fn abilities_are_checked_as_the_permanent_would_exist_on_the_battlefield() {
    // CR 614.12 example: Yixlid Jailer ("Cards in graveyards lose all abilities") doesn't
    // stop a Scarwood Treefolk put onto the battlefield from a graveyard from entering
    // tapped.
    cr!("614.12");
    let mut t = TestGame::new(2);
    t.custom(
        P1,
        permanent(
            "Yixlid Jailer",
            &[CardType::Creature],
            vec![continuous(
                Filter::and(vec![Filter::InZone(ZoneKind::Graveyard), Filter::Card]),
                vec![Modification::RemoveAllAbilities],
            )],
        ),
        Zone::Battlefield,
    );
    let tf = t.graveyard(P0, "Scarwood Treefolk");
    assert!(t.obj_now(tf).chars.abilities.is_empty());
    let new = t.g.move_object_ev(to_battlefield(tf, P0)).unwrap();
    assert!(t.obj_now(new).tapped);
}

#[test]
fn a_permanent_doesnt_apply_its_general_etb_effect_to_itself() {
    // CR 614.12 example: Orb of Dreams ("Permanents enter tapped") won't affect itself.
    cr!("614.12", "614.1d");
    let mut t = TestGame::new(2);
    let orb = || {
        permanent(
            "Orb of Dreams",
            &[CardType::Artifact],
            vec![replacement(
                ReplacementEvent::EntersBattlefield(Filter::Permanent),
                ReplacementAction::EnterTapped,
            )],
        )
    };
    let o = t.custom(P0, orb(), Zone::Hand(P0));
    let o = t.g.move_object_ev(to_battlefield(o, P0)).unwrap();
    assert!(!t.obj_now(o).tapped);
    let bears = t.enter(P1, "Grizzly Bears");
    assert!(t.obj_now(bears).tapped);
}

#[test]
fn clone_chooses_before_entering_and_enters_as_the_copy() {
    // CR 614.12a: a choice required by an effect that modifies how a permanent enters is
    // made before it enters: Clone chooses what to copy (it can't choose itself) and
    // enters as the copy, so the copied creature's "When this creature enters" ability
    // triggers.
    cr!("614.12a", "614.1c");
    let mut t = TestGame::new(2);
    let drawer = t.custom(
        P1,
        creature_with(
            "Seer",
            1,
            1,
            &[Color::Blue],
            vec![triggered(
                TriggerCond::EntersBattlefield(Filter::Source),
                Effect::Draw {
                    who: PlayerRef::You,
                    n: Value::c(1),
                },
            )],
        ),
        Zone::Battlefield,
    );
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
    t.answer_choose(P0, &[Entity::Object(drawer)]);
    let c = t.custom(P0, clone, Zone::Hand(P0));
    let h = t.hand_size(P0);
    t.cast_with(P0, c, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Seer").len(), 2);
    assert_eq!(t.hand_size(P0), h);
    // The choice was offered before it entered: it wasn't among the candidates.
    let asked = t.asked();
    let (_, d) = asked
        .iter()
        .find(|(_, d)| matches!(d, Decision::ChooseEntities { .. }))
        .unwrap();
    if let Decision::ChooseEntities { candidates, .. } = d {
        assert_eq!(candidates, &vec![Entity::Object(drawer)]);
    }
}

fn shock_land() -> CardDef {
    let mut c = chars("Breeding Pool");
    c.card_types = CardTypeSet::single(CardType::Land);
    c.abilities = vec![replacement(
        ReplacementEvent::EntersBattlefield(Filter::Source),
        ReplacementAction::AsEnters(Box::new(Effect::PayOptional {
            who: PlayerRef::You,
            cost: Cost::free().with(CostPart::PayLife(Value::c(2))),
            then: Box::new(Effect::Noop),
            otherwise: Box::new(Effect::Tap { what: Sel::This }),
        })),
    )];
    CardDef::custom(c)
}

#[test]
fn combined_costs_of_simultaneous_entries_must_be_payable() {
    // CR 614.12b: two lands that "may pay 2 life" as they enter simultaneously, with only
    // 3 life: the player can't choose to pay for both.
    cr!("614.12b");
    let mut t = TestGame::new(2);
    t.g.players[0].life = 3;
    let a = t.custom(P0, shock_land(), Zone::Hand(P0));
    let b = t.custom(P0, shock_land(), Zone::Hand(P0));
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    let res =
        t.g.move_objects(vec![to_battlefield(a, P0), to_battlefield(b, P0)]);
    let tapped: Vec<bool> = res.iter().map(|o| t.obj_now(o.unwrap()).tapped).collect();
    assert_eq!(t.life(P0), 1);
    assert_eq!(tapped.iter().filter(|x| **x).count(), 1);
}

#[test]
fn anchor_word_abilities_depend_on_the_choice() {
    // CR 614.12c: "[Anchor word] — [ability]" means "As long as [anchor word] was chosen as
    // this permanent entered, it has [ability]." (Choices "Khans" = 0, "Dragons" = 1.)
    cr!("614.12c");
    for choice in [0i64, 1] {
        let mut t = TestGame::new(2);
        let chose = |n: i32| Condition::Compare(Value::Chosen, Cmp::Eq, Value::c(n));
        let siege = permanent(
            "Citadel Siege",
            &[CardType::Enchantment],
            vec![
                replacement(
                    ReplacementEvent::EntersBattlefield(Filter::Source),
                    ReplacementAction::AsEnters(Box::new(Effect::Choose {
                        who: PlayerRef::You,
                        kind: ChoiceKind::Number { min: 0, max: 1 },
                    })),
                ),
                continuous_if(chose(0), Filter::creature().you_control(), vec![pt(1, 1)]),
                continuous_if(chose(1), Filter::creature().opp_controls(), vec![pt(-1, 0)]),
            ],
        );
        let mine = t.battlefield(P0, "Grizzly Bears");
        let theirs = t.battlefield(P1, "Grizzly Bears");
        t.answer(P0, DecisionKind::Number, Answer::Number(choice));
        let s = t.custom(P0, siege, Zone::Hand(P0));
        t.g.move_object_ev(to_battlefield(s, P0)).unwrap();
        t.recompute();
        if choice == 0 {
            assert_eq!(t.pt(mine), (3, 3));
            assert_eq!(t.pt(theirs), (2, 2));
        } else {
            assert_eq!(t.pt(mine), (2, 2));
            assert_eq!(t.pt(theirs), (1, 2));
        }
    }
}

fn sutured_ghoul() -> CardDef {
    creature_with(
        "Sutured Ghoul",
        0,
        0,
        &[Color::Black],
        vec![replacement(
            ReplacementEvent::EntersBattlefield(Filter::Source),
            ReplacementAction::AsEnters(Box::new(Effect::Exile {
                what: Sel::Choose {
                    chooser: PlayerRef::You,
                    filter: Filter::and(vec![
                        Filter::creature(),
                        Filter::InZone(ZoneKind::Graveyard),
                        Filter::OwnedBy(PlayerRel::You),
                    ]),
                    count: Value::c(99),
                    up_to: true,
                    store: None,
                },
                face_down: false,
                link: true,
            })),
        )],
    )
}

#[test]
fn objects_entering_at_the_same_time_cant_be_chosen() {
    // CR 614.13, 614.13a example: Sutured Ghoul and Runeclaw Bear enter from the graveyard
    // at the same time; the Ghoul's "exile any number of creature cards from your
    // graveyard" can't choose either of them.
    cr!("614.13", "614.13a");
    let mut t = TestGame::new(2);
    let bear = t.graveyard(P0, "Runeclaw Bear");
    let ghoul = t.custom(P0, sutured_ghoul(), Zone::Graveyard(P0));
    let other = t.graveyard(P0, "Grizzly Bears");
    t.answer_choose(P0, &[Entity::Object(other)]);
    t.g.move_objects(vec![to_battlefield(ghoul, P0), to_battlefield(bear, P0)]);
    let asked = t.asked();
    let cands = asked
        .iter()
        .find_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some(candidates.clone()),
            _ => None,
        })
        .unwrap();
    assert_eq!(cands, vec![Entity::Object(other)]);
    assert_eq!(t.named_on_battlefield("Runeclaw Bear").len(), 1);
    assert!(t.in_exile("Grizzly Bears"));
}

#[test]
fn the_same_object_cant_be_chosen_twice() {
    // CR 614.13b: when applying two effects that each have a player sacrifice creatures as
    // a permanent enters (devour-like), the same creature can't be chosen for both.
    cr!("614.13b");
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P0, "Runeclaw Bear");
    let devour = |n: i32| {
        replacement(
            ReplacementEvent::EntersBattlefield(Filter::Source),
            ReplacementAction::AsEnters(Box::new(Effect::seq(vec![
                Effect::Store {
                    var: 20,
                    sel: Sel::Choose {
                        chooser: PlayerRef::You,
                        filter: Filter::creature().you_control().other(),
                        count: Value::c(99),
                        up_to: true,
                        store: None,
                    },
                },
                Effect::SacrificeObjects { what: Sel::Var(20) },
                Effect::AddCounters {
                    what: Sel::This,
                    kind: counters::PLUS1.into(),
                    n: Value::Mul(
                        Box::new(Value::CountSel(Box::new(Sel::Var(20)))),
                        Box::new(Value::c(n)),
                    ),
                },
            ]))),
        )
    };
    let elder = creature_with("Elder", 1, 1, &[Color::Red], vec![devour(3), devour(5)]);
    t.answer_choose(P0, &[Entity::Object(bear)]);
    t.answer_choose(P0, &[Entity::Object(bear)]);
    let e = t.custom(P0, elder, Zone::Hand(P0));
    let e = t.g.move_object_ev(to_battlefield(e, P0)).unwrap();
    let n = t.counters(e, counters::PLUS1);
    assert!(n == 3 || n == 5, "got {n} counters");
    assert!(!t.on_battlefield(bear));
}

#[test]
fn cards_entering_from_a_library_arent_milled_by_their_own_entry() {
    // CR 614.13c: while applying an effect that modifies how a permanent enters, another
    // effect mills cards; cards entering the battlefield from that library at the same
    // time aren't milled.
    cr!("614.13c");
    let mut t = TestGame::new(2);
    let miller = creature_with(
        "Mill Beast",
        2,
        2,
        &[],
        vec![replacement(
            ReplacementEvent::EntersBattlefield(Filter::Source),
            ReplacementAction::AsEnters(Box::new(Effect::Mill {
                who: PlayerRef::You,
                n: Value::c(1),
            })),
        )],
    );
    let under = t.custom(P0, creature("Under", 1, 1, &[]), Zone::Library(P0));
    let second = t.custom(P0, creature("Second", 1, 1, &[]), Zone::Library(P0));
    let top = t.custom(P0, miller, Zone::Library(P0));
    // Put the top two cards onto the battlefield simultaneously.
    t.g.move_objects(vec![to_battlefield(top, P0), to_battlefield(second, P0)]);
    assert_eq!(t.named_on_battlefield("Second").len(), 1);
    assert!(t.in_graveyard(P0, "Under"));
    let _ = under;
}

#[test]
fn replacement_exile_is_linked_to_its_source() {
    // CR 614.14: Void Maw's "If another creature would die, exile it instead" and "Put a
    // card exiled with Void Maw into its owner's graveyard: ..." are linked; the second
    // refers only to cards exiled by the first.
    cr!("614.14");
    let mut t = TestGame::new(2);
    let exile_instead = AbilityDef::with_link(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Replacement(
            ReplacementDef {
                event: ReplacementEvent::Dies(Filter::creature().other()),
                action: ReplacementAction::MoveInstead(Destination::zone(ZoneKind::Exile)),
                self_replacement: false,
                optional: false,
            },
        ))),
        "If another creature would die, exile it instead.",
        1,
    );
    let pump = AbilityDef::with_link(
        AbilityKind::Activated(ActivatedAbility::new(
            Cost::free(),
            Body::effect(Effect::seq(vec![
                Effect::Store {
                    var: 20,
                    sel: Sel::Choose {
                        chooser: PlayerRef::You,
                        filter: Filter::and(vec![
                            Filter::In(Box::new(Sel::Linked)),
                            Filter::InZone(ZoneKind::Exile),
                        ]),
                        count: Value::c(1),
                        up_to: false,
                        store: None,
                    },
                },
                Effect::Move {
                    what: Sel::Var(20),
                    to: Destination::zone(ZoneKind::Graveyard),
                },
                Effect::If {
                    cond: Condition::PrevAffectedAny,
                    then: Box::new(Effect::Modify {
                        what: Sel::This,
                        mods: vec![pt(2, 2)],
                        duration: Duration::EndOfTurn,
                    }),
                    otherwise: Box::new(Effect::Noop),
                },
            ])),
        )),
        "Put a card exiled with Void Maw into its owner's graveyard: +2/+2.",
        1,
    );
    let maw = t.custom(
        P0,
        creature_with("Void Maw", 4, 5, &[Color::Black], vec![exile_instead, pump]),
        Zone::Battlefield,
    );
    // A card exiled some other way isn't linked.
    let other = t.graveyard(P1, "Hill Giant");
    t.g.exile_object(other, None);
    let bear = t.battlefield(P1, "Grizzly Bears");
    resolve_effect(
        &mut t,
        P0,
        vec![target_creature()],
        &[Entity::Object(bear)],
        Effect::Destroy {
            what: Sel::Target(0),
            no_regen: false,
        },
    );
    assert!(t.in_exile("Grizzly Bears"));
    let linked = t.obj_now(maw).linked.get(&1).cloned().unwrap_or_default();
    assert_eq!(linked.len(), 1);
    assert_eq!(t.obj_now(linked[0]).chars.name, "Grizzly Bears");
    t.activate(P0, maw, 0, &[]).unwrap();
    t.resolve();
    assert_eq!(t.pt(maw), (6, 7));
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(t.in_exile("Hill Giant"));
}
