//! CR 614.15–614.17 (self-replacement effects, token and counter replacements, "can't"
//! effects) and CR 616 (interaction of replacement and prevention effects).

use crate::r609_614_replacement::rest_in_peace;
use crate::r609_common::*;
use mtg_engine::ability::*;
use mtg_engine::card::{FaceDef, Layout};
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

fn memory_lapse() -> Body {
    Body::simple(
        vec![TargetSpec {
            what: TargetKind::Spell(Filter::Any),
            ..TargetSpec::object(Filter::Any, "target spell")
        }],
        Effect::SelfReplace {
            replacement: ReplacementDef {
                event: ReplacementEvent::ZoneChange {
                    filter: Filter::In(Box::new(Sel::Target(0))),
                    from: Some(ZoneKind::Stack),
                    to: Some(ZoneKind::Graveyard),
                },
                action: ReplacementAction::MoveInstead(Destination::library_top()),
                self_replacement: true,
                optional: false,
            },
            effect: Box::new(Effect::CounterSpell {
                what: Sel::Target(0),
            }),
        },
    )
}

fn replacement_decisions(t: &TestGame) -> usize {
    t.asked()
        .iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseReplacement { .. }))
        .count()
}

#[test]
fn self_replacement_effects_apply_first() {
    // CR 614.15, 616.1a: Memory Lapse's "put it on top of its owner's library instead of
    // into that player's graveyard" is a self-replacement effect: it must be applied before
    // Rest in Peace, which then no longer applies.
    cr!("614.15", "616.1a");
    let mut t = TestGame::new(2);
    t.custom(P1, rest_in_peace(), Zone::Battlefield);
    t.lands(P0, "Forest", 2);
    let bears = t.hand(P0, "Grizzly Bears");
    let spell = t.cast(P0, bears).go();
    let lapse = t.custom(P1, instant("Memory Lapse", memory_lapse()), Zone::Hand(P1));
    t.cast_with(P1, lapse, &[Entity::Object(spell)]).unwrap();
    t.resolve();
    let top = t.g.library_top(P0).unwrap();
    assert_eq!(t.obj_now(top).chars.name, "Grizzly Bears");
    assert!(!t.in_exile("Grizzly Bears"));
    assert_eq!(replacement_decisions(&t), 0);
    // Memory Lapse itself is exiled by Rest in Peace.
    assert!(t.in_exile("Memory Lapse"));
}

fn colossus() -> CardDef {
    creature_with(
        "Darksteel Colossus",
        11,
        11,
        &[],
        vec![replacement(
            ReplacementEvent::ZoneChange {
                filter: Filter::Source,
                from: None,
                to: Some(ZoneKind::Graveyard),
            },
            ReplacementAction::MoveInstead(Destination {
                position: LibraryPosition::Shuffled,
                ..Destination::zone(ZoneKind::Library)
            }),
        )],
    )
}

fn destroy(t: &mut TestGame, p: PlayerId, what: ObjectId) {
    resolve_effect(
        t,
        p,
        vec![target_creature()],
        &[Entity::Object(what)],
        Effect::Destroy {
            what: Sel::Target(0),
            no_regen: false,
        },
    );
}

#[test]
fn the_affected_objects_controller_chooses_and_the_other_does_nothing() {
    // CR 616.1, 616.1e, 616.1f, first example: Rest in Peace and "If this creature would be
    // put into a graveyard, shuffle it into its owner's library instead". The creature's
    // controller chooses which applies first; the other then does nothing.
    cr!("616.1", "616.1e", "616.1f");
    for pick in [0usize, 1] {
        let mut t = TestGame::new(2);
        t.custom(P1, rest_in_peace(), Zone::Battlefield);
        let c = t.custom(P0, colossus(), Zone::Battlefield);
        t.answer(P0, DecisionKind::Replacement, Answer::Index(pick));
        destroy(&mut t, P1, c);
        let asked: Vec<PlayerId> = t
            .asked()
            .iter()
            .filter(|(_, d)| matches!(d, Decision::ChooseReplacement { .. }))
            .map(|(p, _)| *p)
            .collect();
        assert_eq!(asked, vec![P0]);
        // Candidates are listed with the static from the permanents first (Rest in
        // Peace), then the creature's own.
        let exiled = t.in_exile("Darksteel Colossus");
        let in_library =
            !t.g.find_in_zone(Zone::Library(P0), "Darksteel Colossus")
                .is_empty();
        assert_eq!(exiled, pick == 0);
        assert_eq!(in_library, pick == 1);
        assert!(!t.in_graveyard(P0, "Darksteel Colossus"));
    }
}

#[test]
fn simultaneous_choices_are_made_in_apnap_order() {
    // CR 616.1: if two or more players have to make these choices at the same time, they
    // do so in APNAP order.
    cr!("616.1");
    for active in [P0, P1] {
        let mut t = TestGame::new(2);
        t.set_step(active, mtg_engine::turn::Step::PrecombatMain);
        t.custom(P0, rest_in_peace(), Zone::Battlefield);
        let a = t.custom(P0, colossus(), Zone::Battlefield);
        let b = t.custom(P1, colossus(), Zone::Battlefield);
        let objs = if active == P0 { vec![b, a] } else { vec![a, b] };
        t.g.destroy_all(objs, None, false);
        let asked: Vec<PlayerId> = t
            .asked()
            .iter()
            .filter(|(_, d)| matches!(d, Decision::ChooseReplacement { .. }))
            .map(|(p, _)| *p)
            .collect();
        let other = if active == P0 { P1 } else { P0 };
        assert_eq!(asked, vec![active, other]);
    }
}

#[test]
fn control_changing_entry_effects_apply_first() {
    // CR 616.1b: an effect modifying under whose control an object would enter must be
    // chosen first. Then the opponent's "creatures you control enter with a +1/+1
    // counter" no longer applies.
    cr!("616.1b");
    let mut t = TestGame::new(2);
    t.custom(
        P1,
        permanent(
            "Counter Lord",
            &[CardType::Enchantment],
            vec![replacement(
                ReplacementEvent::EntersBattlefield(Filter::creature().you_control()),
                ReplacementAction::EnterWithCounters(counters::PLUS1.into(), Value::c(1)),
            )],
        ),
        Zone::Battlefield,
    );
    // "If a creature would enter under an opponent's control this turn, it enters under
    // your control instead."
    resolve_effect(
        &mut t,
        P0,
        vec![],
        &[],
        Effect::AddReplacement {
            def: ReplacementDef {
                event: ReplacementEvent::EntersBattlefield(Filter::and(vec![
                    Filter::creature(),
                    Filter::ControlledBy(PlayerRel::Opponent),
                ])),
                action: ReplacementAction::EnterUnderControl(PlayerRef::You),
                self_replacement: false,
                optional: false,
            },
            duration: Duration::EndOfTurn,
            uses: None,
        },
    );
    let bear = t.enter(P1, "Grizzly Bears");
    assert_eq!(t.obj_now(bear).controller, P0);
    assert_eq!(t.counters(bear, counters::PLUS1), 0);
    assert_eq!(replacement_decisions(&t), 0);
}

#[test]
fn copy_entry_effects_apply_before_others() {
    // CR 616.1c; 616.1f, second example: Essence of the Wild ("Creatures you control enter
    // as a copy of this creature") is applied first to Rusted Sentinel, which then no
    // longer has "enters tapped": it's an untapped copy of Essence of the Wild.
    cr!("616.1c", "616.1f");
    let mut t = TestGame::new(2);
    t.custom(
        P0,
        creature_with(
            "Essence of the Wild",
            6,
            6,
            &[Color::Green],
            vec![replacement(
                ReplacementEvent::EntersBattlefield(Filter::creature().you_control().other()),
                ReplacementAction::EnterAsCopy {
                    filter: Filter::Source,
                    optional: false,
                },
            )],
        ),
        Zone::Battlefield,
    );
    let s = t.enter(P0, "Rusted Sentinel");
    assert_eq!(t.obj_now(s).chars.name, "Essence of the Wild");
    assert!(!t.obj_now(s).tapped);
    assert_eq!(replacement_decisions(&t), 0);
}

#[test]
fn back_face_entry_effects_apply_before_others() {
    // CR 616.1d: an effect that makes a card enter with its back face up must be chosen
    // before other effects; the front face's "enters tapped" then doesn't apply.
    cr!("616.1d");
    let mut t = TestGame::new(2);
    let mut front = chars("Day Face");
    front.card_types = CardTypeSet::single(CardType::Creature);
    front.power = Some(2);
    front.toughness = Some(2);
    front.abilities = vec![replacement(
        ReplacementEvent::EntersBattlefield(Filter::Source),
        ReplacementAction::EnterTapped,
    )];
    let mut back = chars("Night Face");
    back.card_types = CardTypeSet::single(CardType::Creature);
    back.power = Some(4);
    back.toughness = Some(4);
    let mut def = CardDef::custom(front);
    def.layout = Layout::Transform;
    def.faces.push(FaceDef {
        chars: back,
        unsupported: vec![],
        star_power: false,
        star_toughness: false,
    });
    t.custom(
        P0,
        permanent(
            "Moonlight",
            &[CardType::Enchantment],
            vec![replacement(
                ReplacementEvent::EntersBattlefield(Filter::Named("Day Face".into())),
                ReplacementAction::EnterTransformed,
            )],
        ),
        Zone::Battlefield,
    );
    let c = t.custom(P0, def, Zone::Hand(P0));
    let c = t.g.move_object_ev(to_battlefield(c, P0)).unwrap();
    assert_eq!(t.obj_now(c).chars.name, "Night Face");
    assert!(!t.obj_now(c).tapped);
    assert_eq!(replacement_decisions(&t), 0);
}

#[test]
fn the_affected_player_chooses_the_order() {
    // CR 616.1e: any applicable effect may be chosen: "+1 damage" then "double" gives 6,
    // "double" then "+1" gives 5 (the damaged player chooses).
    cr!("616.1e", "616.1f");
    for (pick, expect) in [(0usize, 6), (1, 5)] {
        let mut t = TestGame::new(2);
        let dmg = |a: ReplacementAction| {
            replacement(
                ReplacementEvent::Damage {
                    source: Filter::Any,
                    to_players: Some(PlayerFilter::Any),
                    to_objects: None,
                    combat_only: false,
                },
                a,
            )
        };
        t.custom(
            P0,
            permanent(
                "Plus One",
                &[CardType::Enchantment],
                vec![dmg(ReplacementAction::Add(Value::c(1)))],
            ),
            Zone::Battlefield,
        );
        t.custom(
            P0,
            permanent(
                "Doubler",
                &[CardType::Enchantment],
                vec![dmg(ReplacementAction::Multiply(2))],
            ),
            Zone::Battlefield,
        );
        t.answer(P1, DecisionKind::Replacement, Answer::Index(pick));
        resolve_effect(
            &mut t,
            P0,
            vec![target_any()],
            &[Entity::Player(P1)],
            Effect::DealDamage {
                source: Sel::This,
                amount: Value::c(2),
                to: Sel::Target(0),
            },
        );
        assert_eq!(t.life(P1), 20 - expect);
    }
}

#[test]
fn replacement_of_a_contained_event_is_chosen_after() {
    // CR 616.1g example: a token doubler applies to creating the tokens first; then each
    // token copy of Voice of All applies its own "as this enters, choose a color".
    cr!("616.1g");
    let mut t = TestGame::new(2);
    t.custom(
        P0,
        permanent(
            "Doubling Season",
            &[CardType::Enchantment],
            vec![replacement(
                ReplacementEvent::CreateTokens(PlayerFilter::You),
                ReplacementAction::Multiply(2),
            )],
        ),
        Zone::Battlefield,
    );
    let voice = t.custom(
        P0,
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
        ),
        Zone::Battlefield,
    );
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.answer(P0, DecisionKind::Option, Answer::Index(2));
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
    let tokens: Vec<ObjectId> = t
        .named_on_battlefield("Voice of All")
        .into_iter()
        .filter(|v| t.obj_now(*v).is_token())
        .collect();
    assert_eq!(tokens.len(), 2);
    let mut colors: Vec<Option<Color>> =
        tokens.iter().map(|v| t.obj_now(*v).choices.color).collect();
    colors.sort();
    assert_eq!(colors, vec![Some(Color::Blue), Some(Color::Black)]);
}

#[test]
fn a_modified_event_can_become_subject_to_other_effects() {
    // CR 616.2 example: "If you would gain life, draw that many cards instead" and "If you
    // would draw a card, return a card from your graveyard to your hand instead": gaining
    // 1 life returns a card from the graveyard to hand.
    cr!("616.2");
    let mut t = TestGame::new(2);
    t.custom(
        P0,
        permanent(
            "Life to Cards",
            &[CardType::Enchantment],
            vec![replacement(
                ReplacementEvent::GainLife(PlayerFilter::You),
                ReplacementAction::Instead(Box::new(Effect::Draw {
                    who: PlayerRef::You,
                    n: Value::EventAmount,
                })),
            )],
        ),
        Zone::Battlefield,
    );
    t.custom(
        P0,
        permanent(
            "Cards to Recursion",
            &[CardType::Enchantment],
            vec![replacement(
                ReplacementEvent::Draw(PlayerFilter::You),
                ReplacementAction::Instead(Box::new(Effect::Move {
                    what: Sel::Choose {
                        chooser: PlayerRef::You,
                        filter: Filter::and(vec![
                            Filter::InZone(ZoneKind::Graveyard),
                            Filter::OwnedBy(PlayerRel::You),
                        ]),
                        count: Value::c(1),
                        up_to: false,
                        store: None,
                    },
                    to: Destination::zone(ZoneKind::Hand),
                })),
            )],
        ),
        Zone::Battlefield,
    );
    t.graveyard(P0, "Lightning Bolt");
    let lib = t.library_size(P0);
    t.g.gain_life(P0, 1);
    assert_eq!(t.life(P0), 20);
    assert!(t.in_hand(P0, "Lightning Bolt"));
    assert_eq!(t.library_size(P0), lib);
}

#[test]
fn counter_and_token_replacements_apply_to_replacement_effects() {
    // CR 614.16: "if one or more +1/+1 counters would be put on a creature" applies to
    // counters a permanent enters with (a replacement effect), and a token doubler applies
    // to tokens created by a replacement effect even though the original event (a
    // creature dying as a state-based action) wasn't an effect.
    cr!("614.16");
    let mut t = TestGame::new(2);
    t.custom(
        P0,
        permanent(
            "Hardened Scales",
            &[CardType::Enchantment],
            vec![replacement(
                ReplacementEvent::PutCounters {
                    on_objects: Some(Filter::creature().you_control()),
                    on_players: None,
                    kind: Some(counters::PLUS1.into()),
                },
                ReplacementAction::Add(Value::c(1)),
            )],
        ),
        Zone::Battlefield,
    );
    let hydra = t.custom(
        P0,
        creature_with(
            "Hydra",
            0,
            0,
            &[Color::Green],
            vec![replacement(
                ReplacementEvent::EntersBattlefield(Filter::Source),
                ReplacementAction::EnterWithCounters(counters::PLUS1.into(), Value::c(2)),
            )],
        ),
        Zone::Hand(P0),
    );
    let hydra = t.g.move_object_ev(to_battlefield(hydra, P0)).unwrap();
    assert_eq!(t.counters(hydra, counters::PLUS1), 3);

    t.custom(
        P0,
        permanent(
            "Doubling Season",
            &[CardType::Enchantment],
            vec![replacement(
                ReplacementEvent::CreateTokens(PlayerFilter::You),
                ReplacementAction::Multiply(2),
            )],
        ),
        Zone::Battlefield,
    );
    let spirit = TokenSpec {
        name: "Spirit".into(),
        colors: colors(&[Color::White]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: vec!["Spirit".into()],
        power: Some(1),
        toughness: Some(1),
        abilities: vec![],
        scryfall_name: None,
    };
    t.custom(
        P0,
        permanent(
            "Haunting",
            &[CardType::Enchantment],
            vec![replacement(
                ReplacementEvent::Dies(Filter::creature().you_control()),
                ReplacementAction::Also(Box::new(Effect::CreateToken {
                    spec: spirit,
                    count: Value::c(1),
                    controller: PlayerRef::You,
                    tapped: false,
                    attacking: false,
                })),
            )],
        ),
        Zone::Battlefield,
    );
    let frail = t.custom(P0, creature("Frail", 1, 1, &[]), Zone::Battlefield);
    t.g.objects[frail.0 as usize].damage = 1;
    t.settle();
    assert!(!t.on_battlefield(frail));
    assert_eq!(t.named_on_battlefield("Spirit").len(), 2);
}

#[test]
fn cant_effects_dont_go_back_in_time() {
    // CR 614.17, 614.17a: "can't" effects aren't replacement effects; they must exist
    // before the event and don't change what already happened.
    cr!("614.17", "614.17a");
    let mut t = TestGame::new(2);
    t.g.gain_life(P0, 3);
    resolve_effect(
        &mut t,
        P1,
        vec![],
        &[],
        Effect::AddRestriction {
            restriction: Restriction::CantGainLife(PlayerFilter::Any),
            duration: Duration::EndOfTurn,
        },
    );
    assert_eq!(t.life(P0), 23);
    t.g.gain_life(P0, 3);
    assert_eq!(t.life(P0), 23);
}

#[test]
fn costs_including_events_that_cant_happen_cant_be_paid() {
    // CR 614.17b: a creature that can't be sacrificed can't be sacrificed to pay a cost;
    // a player whose life total can't change can't pay life.
    cr!("614.17b");
    let mut t = TestGame::new(2);
    let altar = t.custom(
        P0,
        permanent(
            "Altar",
            &[CardType::Artifact],
            vec![activated(
                Cost::free().with(CostPart::Sacrifice {
                    filter: Filter::creature(),
                    count: Value::c(1),
                }),
                Body::effect(Effect::GainLife {
                    who: PlayerRef::You,
                    n: Value::c(1),
                }),
            )],
        ),
        Zone::Battlefield,
    );
    let c = t.battlefield(P0, "Grizzly Bears");
    assert!(t.activate(P0, altar, 0, &[]).is_ok());
    t.resolve();
    assert_eq!(t.life(P0), 21);
    t.battlefield(P0, "Grizzly Bears");
    t.custom(
        P0,
        permanent(
            "Sanctuary",
            &[CardType::Enchantment],
            vec![restriction(Restriction::CantBeSacrificed(
                Filter::creature(),
            ))],
        ),
        Zone::Battlefield,
    );
    t.recompute();
    assert!(t.activate(P0, altar, 0, &[]).is_err());
    let _ = c;
    // Life can't be paid while it can't change.
    t.custom(
        P0,
        permanent(
            "Emperion",
            &[CardType::Enchantment],
            vec![restriction(Restriction::CantLoseLife(PlayerFilter::You))],
        ),
        Zone::Battlefield,
    );
    t.recompute();
    assert!(!t.g.can_pay_life(P0, 2));
    assert!(t.g.can_pay_life(P0, 0));
}

#[test]
fn events_that_cant_happen_are_only_replaced_by_self_replacement() {
    // CR 614.17c: Tainted Remedy ("If an opponent would gain life, that player loses that
    // much life instead") doesn't apply to life gain that can't happen. A self-replacement
    // effect still can replace it.
    cr!("614.17c");
    let mut t = TestGame::new(2);
    t.custom(
        P0,
        permanent(
            "Tainted Remedy",
            &[CardType::Enchantment],
            vec![replacement(
                ReplacementEvent::GainLife(PlayerFilter::Opponent),
                ReplacementAction::Instead(Box::new(Effect::LoseLife {
                    who: PlayerRef::TriggerPlayer,
                    n: Value::EventAmount,
                })),
            )],
        ),
        Zone::Battlefield,
    );
    t.g.gain_life(P1, 3);
    assert_eq!(t.life(P1), 17);
    t.custom(
        P0,
        permanent(
            "Erebos",
            &[CardType::Enchantment],
            vec![restriction(Restriction::CantGainLife(
                PlayerFilter::Opponent,
            ))],
        ),
        Zone::Battlefield,
    );
    t.recompute();
    t.g.gain_life(P1, 3);
    assert_eq!(t.life(P1), 17);
    // "You gain 3 life. If you would gain life this way, draw that many cards instead."
    let h = t.hand_size(P1);
    cast_resolve(
        &mut t,
        P1,
        Body::effect(Effect::SelfReplace {
            replacement: ReplacementDef {
                event: ReplacementEvent::GainLife(PlayerFilter::You),
                action: ReplacementAction::Instead(Box::new(Effect::Draw {
                    who: PlayerRef::You,
                    n: Value::EventAmount,
                })),
                self_replacement: true,
                optional: false,
            },
            effect: Box::new(Effect::GainLife {
                who: PlayerRef::You,
                n: Value::c(3),
            }),
        }),
        &[],
    );
    assert_eq!(t.hand_size(P1), h + 3);
    assert_eq!(t.life(P1), 17);
}

#[test]
fn cant_enter_checks_the_object_as_it_would_exist_on_the_battlefield() {
    // CR 614.17d: "Blue permanents can't enter the battlefield" stops a red creature card
    // that would be blue on the battlefield (because of "Creatures are blue").
    cr!("614.17d");
    let mut t = TestGame::new(2);
    t.custom(
        P1,
        permanent(
            "Blue Ban",
            &[CardType::Enchantment],
            vec![restriction(Restriction::CantEnter(Filter::and(vec![
                Filter::Permanent,
                Filter::Color(Color::Blue),
            ])))],
        ),
        Zone::Battlefield,
    );
    let goblin = t.graveyard(P0, "Raging Goblin");
    let g2 = t.g.move_object_ev(to_battlefield(goblin, P0)).unwrap();
    assert!(t.on_battlefield(g2));
    t.custom(
        P1,
        permanent(
            "Blue Wash",
            &[CardType::Enchantment],
            vec![continuous(
                Filter::creature(),
                vec![Modification::SetColors(colors(&[Color::Blue]))],
            )],
        ),
        Zone::Battlefield,
    );
    t.recompute();
    let goblin2 = t.graveyard(P0, "Raging Goblin");
    let res = t.g.move_object_ev(to_battlefield(goblin2, P0));
    assert!(res.is_none());
    assert!(t.in_graveyard(P0, "Raging Goblin"));
}
