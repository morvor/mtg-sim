//! CR 607: linked abilities.

use super::r600_common::*;
use mtg_engine::ability::*;
use mtg_engine::card::{FaceDef, Layout};
use mtg_engine::events::MoveCause;
use mtg_engine::keywords::{Keyword, KeywordKind};
use mtg_engine::mana::ManaType;
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

fn graveyard_card() -> TargetSpec {
    TargetSpec::object(
        Filter::InZone(ZoneKind::Graveyard),
        "target card from a graveyard",
    )
}

/// "{0}: Exile target card from a graveyard."
fn exile_from_graveyard() -> Ability {
    act(
        mana_cost("{0}"),
        Body::simple(
            vec![graveyard_card()],
            Effect::Exile {
                what: Sel::Target(0),
                face_down: false,
                link: true,
            },
        ),
    )
}

/// "{0}: Return all cards exiled with this to their owners' hands."
fn return_exiled_to_hand() -> Ability {
    act(
        mana_cost("{0}"),
        Body::effect(Effect::Move {
            what: Sel::Linked,
            to: Destination::zone(ZoneKind::Hand),
        }),
    )
}

fn exile_spell() -> CardDef {
    CB::new("Exile Test")
        .instant()
        .cost("{0}")
        .spell(Body::simple(
            vec![target_creature()],
            Effect::Exile {
                what: Sel::Target(0),
                face_down: false,
                link: false,
            },
        ))
        .build()
}

fn choose_color(t: &mut TestGame, p: PlayerId, c: Color) {
    let i = Color::ALL.iter().position(|x| *x == c).unwrap();
    t.answer(p, DecisionKind::Option, Answer::Index(i));
}

fn pool(t: &TestGame, p: PlayerId) -> Vec<ManaType> {
    t.g.player(p).mana_pool.mana.iter().map(|m| m.ty).collect()
}

#[test]
fn an_exiled_card_ability_refers_only_to_what_its_linked_ability_exiled() {
    cr!("607.1", "607.2", "607.2a");
    // Journey to Nowhere: "When this enchantment enters, exile target creature." / "When
    // this enchantment leaves the battlefield, return the exiled card to the battlefield
    // under its owner's control."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    let j = t.enter(P0, "Journey to Nowhere");
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    // Another effect exiles the Wurm.
    let e = t.custom(P0, exile_spell(), Zone::Hand(P0));
    t.cast(P0, e).target(wurm).go();
    t.resolve_all();
    assert!(t.in_exile("Craw Wurm"));
    t.g.destroy(j, None);
    t.resolve_all();
    let back = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(back.len(), 1);
    assert_eq!(t.obj(back[0]).controller, P1);
    assert!(t.in_exile("Craw Wurm"));
}

#[test]
fn an_ability_an_object_grants_itself_counts_as_printed_on_it() {
    cr!("607.1a");
    // "This artifact has '{0}: Exile target card from a graveyard.'" and "{0}: Return all
    // cards exiled with this artifact to their owners' hands."
    let def = CB::new("Self Granter")
        .artifact()
        .ability(stat(StaticEffect::Continuous {
            affected: Filter::Source,
            mods: vec![Modification::AddAbility(exile_from_graveyard())],
        }))
        .ability(return_exiled_to_hand())
        .build();
    let mut t = TestGame::new(2);
    let a = t.custom(P0, def, Zone::Battlefield);
    let b = t.graveyard(P1, "Grizzly Bears");
    // The granted ability is the second activated ability it has.
    t.activate(P0, a, 1, &[Entity::Object(b)]).unwrap();
    t.resolve();
    assert!(t.in_exile("Grizzly Bears"));
    t.activate(P0, a, 0, &[]).unwrap();
    t.resolve();
    assert!(t.in_hand(P1, "Grizzly Bears"));
}

#[test]
fn abilities_on_both_faces_of_a_transforming_card_are_linked() {
    cr!("607.1b");
    let front = CB::new("Two Faced Warden")
        .artifact()
        .ability(exile_from_graveyard())
        .build();
    let back = CB::new("Warden Released")
        .artifact()
        .ability(return_exiled_to_hand())
        .build();
    let mut def = front;
    def.layout = Layout::Transform;
    def.faces.push(FaceDef {
        chars: back.faces[0].chars.clone(),
        unsupported: vec![],
        star_power: false,
        star_toughness: false,
    });
    let mut t = TestGame::new(2);
    let w = t.custom(P0, def, Zone::Battlefield);
    let b = t.graveyard(P1, "Grizzly Bears");
    t.activate(P0, w, 0, &[Entity::Object(b)]).unwrap();
    t.resolve();
    assert!(t.in_exile("Grizzly Bears"));
    assert!(mtg_engine::dfc::transform(&mut t.g, w));
    t.g.recompute();
    assert_eq!(t.obj(w).chars.name.as_str(), "Warden Released");
    t.activate(P0, w, 0, &[]).unwrap();
    t.resolve();
    assert!(t.in_hand(P1, "Grizzly Bears"));
}

#[test]
fn an_ability_can_be_linked_to_itself() {
    cr!("607.1c");
    // "{0}: Exile target card from a graveyard. You gain life equal to the number of cards
    // exiled with this artifact."
    let def = CB::new("Self Linked")
        .artifact()
        .ability(act(
            mana_cost("{0}"),
            Body::simple(
                vec![graveyard_card()],
                Effect::Seq(vec![
                    Effect::Exile {
                        what: Sel::Target(0),
                        face_down: false,
                        link: true,
                    },
                    Effect::GainLife {
                        who: PlayerRef::You,
                        n: Value::CountSel(Box::new(Sel::Linked)),
                    },
                ]),
            ),
        ))
        .build();
    let mut t = TestGame::new(2);
    let a = t.custom(P0, def, Zone::Battlefield);
    let b1 = t.graveyard(P1, "Grizzly Bears");
    let b2 = t.graveyard(P1, "Craw Wurm");
    t.activate(P0, a, 0, &[Entity::Object(b1)]).unwrap();
    t.resolve();
    assert_eq!(t.life(P0), 21);
    t.activate(P0, a, 0, &[Entity::Object(b2)]).unwrap();
    t.resolve();
    assert_eq!(t.life(P0), 23);
}

#[test]
fn a_token_can_have_an_ability_linked_to_its_creator() {
    cr!("607.1d", "607.2c");
    // "{0}: Exile target creature. Create a token with 'Sacrifice this token: Return the
    // card exiled with the object that created this token to the battlefield under its
    // owner's control.'"
    let jailer_token = TokenSpec {
        name: "Key".into(),
        colors: ColorSet::NONE,
        supertypes: vec![],
        card_types: vec![CardType::Artifact],
        subtypes: vec![],
        power: None,
        toughness: None,
        abilities: vec![act(
            Cost::default().with(CostPart::SacrificeSelf),
            Body::effect(Effect::Move {
                what: Sel::CreatorLinked,
                to: {
                    let mut d = Destination::battlefield();
                    d.controller = Some(PlayerRef::OwnerOf(Box::new(Sel::CreatorLinked)));
                    d
                },
            }),
        )],
        scryfall_name: None,
    };
    let def = CB::new("Jailer")
        .artifact()
        .ability(act(
            mana_cost("{0}"),
            Body::simple(
                vec![target_creature()],
                Effect::Seq(vec![
                    Effect::Exile {
                        what: Sel::Target(0),
                        face_down: false,
                        link: true,
                    },
                    Effect::CreateToken {
                        spec: jailer_token,
                        count: Value::c(1),
                        controller: PlayerRef::You,
                        tapped: false,
                        attacking: false,
                    },
                ]),
            ),
        ))
        // "{0}: Destroy all tokens created with this artifact."
        .ability(act(
            mana_cost("{0}"),
            Body::effect(Effect::Destroy {
                what: Sel::All(Filter::And(vec![
                    Filter::Token,
                    Filter::In(Box::new(Sel::Linked)),
                ])),
                no_regen: false,
            }),
        ))
        .build();
    let mut t = TestGame::new(2);
    let j = t.custom(P0, def.clone(), Zone::Battlefield);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.activate(P0, j, 0, &[Entity::Object(bears)]).unwrap();
    t.resolve();
    let key = t.named_on_battlefield("Key")[0];
    assert_eq!(t.obj(key).created_by.map(|(c, _)| c), Some(j));
    t.activate(P0, key, 0, &[]).unwrap();
    t.resolve();
    let back = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(back.len(), 1);
    assert_eq!(t.obj(back[0]).controller, P1);
    // "Tokens created with this artifact": only its own tokens, not another object's.
    let mut t = TestGame::new(2);
    let j1 = t.custom(P0, def.clone(), Zone::Battlefield);
    let j2 = t.custom(P0, def, Zone::Battlefield);
    let b1 = t.battlefield(P1, "Grizzly Bears");
    let b2 = t.battlefield(P1, "Grizzly Bears");
    t.activate(P0, j1, 0, &[Entity::Object(b1)]).unwrap();
    t.resolve();
    t.activate(P0, j2, 0, &[Entity::Object(b2)]).unwrap();
    t.resolve();
    assert_eq!(t.named_on_battlefield("Key").len(), 2);
    t.activate(P0, j1, 1, &[]).unwrap();
    t.resolve();
    let keys = t.named_on_battlefield("Key");
    assert_eq!(keys.len(), 1);
    assert_eq!(t.obj(keys[0]).created_by.map(|(c, _)| c), Some(j2));
}

#[test]
fn cards_exiled_by_a_replacement_effect_are_linked_to_it() {
    cr!("607.2b");
    // "If a creature an opponent controls would die, exile it instead." / "{0}: Put a
    // creature card exiled with this enchantment onto the battlefield under your control."
    let def = CB::new("Soul Warden Test")
        .enchantment()
        .ability(stat(StaticEffect::Replacement(ReplacementDef {
            event: ReplacementEvent::ZoneChange {
                filter: Filter::creature().opp_controls(),
                from: Some(ZoneKind::Battlefield),
                to: Some(ZoneKind::Graveyard),
            },
            action: ReplacementAction::MoveInstead(Destination::zone(ZoneKind::Exile)),
            self_replacement: false,
            optional: false,
        })))
        .ability(act(
            mana_cost("{0}"),
            Body::effect(Effect::Move {
                what: Sel::Choose {
                    chooser: PlayerRef::You,
                    filter: Filter::And(vec![
                        Filter::In(Box::new(Sel::Linked)),
                        Filter::InZone(ZoneKind::Exile),
                        Filter::creature(),
                    ]),
                    count: Value::c(1),
                    up_to: false,
                    store: None,
                },
                to: Destination::battlefield().under_your_control(),
            }),
        ))
        .build();
    let mut t = TestGame::new(2);
    let w = t.custom(P0, def, Zone::Battlefield);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.g.destroy(bears, None);
    t.settle();
    assert!(t.in_exile("Grizzly Bears"));
    let e = t.custom(P0, exile_spell(), Zone::Hand(P0));
    t.cast(P0, e).target(wurm).go();
    t.resolve_all();
    assert!(t.in_exile("Craw Wurm"));
    t.activate(P0, w, 0, &[]).unwrap();
    t.resolve();
    let b = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(b.len(), 1);
    assert_eq!(t.obj(b[0]).controller, P0);
    assert!(t.in_exile("Craw Wurm"));
    // Nothing else is linked to it.
    t.activate(P0, w, 0, &[]).unwrap();
    t.resolve();
    assert!(t.named_on_battlefield("Craw Wurm").is_empty());
}

#[test]
fn a_choice_is_linked_to_the_abilities_that_refer_to_it() {
    cr!("607.2d", "607.4");
    // Paradise Plume: "As this artifact enters, choose a color." / "Whenever a player casts
    // a spell of the chosen color, you may gain 1 life." / "{T}: Add one mana of the
    // chosen color."
    let mut t = TestGame::new(2);
    choose_color(&mut t, P0, Color::Red);
    let plume = t.enter(P0, "Paradise Plume");
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.answer_yes(P0, true);
    t.cast(P1, bolt).target(P1).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
    // A green spell doesn't trigger it.
    t.lands(P1, "Forest", 1);
    let gg = t.hand(P1, "Giant Growth");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.cast(P1, gg).target(bears).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
    t.activate(P0, plume, 0, &[]).unwrap();
    assert_eq!(pool(&t, P0), vec![ManaType::R]);
    // Voice of All: "As this creature enters, choose a color." / "This creature has
    // protection from the chosen color."
    let mut t = TestGame::new(2);
    choose_color(&mut t, P0, Color::Red);
    let v = t.enter(P0, "Voice of All");
    t.g.recompute();
    let bolt = t.hand(P1, "Lightning Bolt");
    let gg = t.hand(P1, "Giant Growth");
    assert!(t.g.protected_from(v, bolt));
    assert!(!t.g.protected_from(v, gg));
}

#[test]
fn noted_information_is_linked_to_the_ability_that_noted_it() {
    cr!("607.2e");
    // "As this artifact enters, note the number of creatures you control." / "{0}: You gain
    // life equal to the noted number."
    let def = CB::new("Census Taker")
        .artifact()
        .ability(stat(StaticEffect::Replacement(ReplacementDef {
            event: ReplacementEvent::EntersBattlefield(Filter::Source),
            action: ReplacementAction::AsEnters(Box::new(Effect::Note {
                value: Value::Count(Filter::creature().you_control()),
            })),
            self_replacement: true,
            optional: false,
        })))
        .ability(act(
            mana_cost("{0}"),
            Body::effect(Effect::GainLife {
                who: PlayerRef::You,
                n: Value::Chosen,
            }),
        ))
        .build();
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Grizzly Bears");
    let c = t.custom(P0, def, Zone::Nowhere);
    t.g.move_object(c, Zone::Battlefield, MoveCause::Effect, Some(P0));
    let c = t.g.current(c);
    // More creatures later don't change the noted number.
    t.battlefield(P0, "Grizzly Bears");
    t.activate(P0, c, 0, &[]).unwrap();
    t.resolve();
    assert_eq!(t.life(P0), 22);
}

#[test]
fn a_choice_between_words_is_linked() {
    cr!("607.2f");
    // "As this enchantment enters, choose odd or even." / "Creatures you control with mana
    // value of the chosen quality get +1/+1."
    let def = CB::new("Parity Banner")
        .enchantment()
        .ability(stat(StaticEffect::Replacement(ReplacementDef {
            event: ReplacementEvent::EntersBattlefield(Filter::Source),
            action: ReplacementAction::AsEnters(Box::new(Effect::Choose {
                who: PlayerRef::You,
                kind: ChoiceKind::OddOrEven,
            })),
            self_replacement: true,
            optional: false,
        })))
        .ability(stat(StaticEffect::Continuous {
            affected: Filter::And(vec![
                Filter::creature().you_control(),
                Filter::ManaValueOfChosenQuality,
            ]),
            mods: vec![Modification::ModifyPT(Value::c(1), Value::c(1))],
        }))
        .build();
    let mut t = TestGame::new(2);
    let elves = t.battlefield(P0, "Llanowar Elves");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    let c = t.custom(P0, def, Zone::Nowhere);
    t.g.move_object(c, Zone::Battlefield, MoveCause::Effect, Some(P0));
    t.g.recompute();
    assert_eq!(t.pt(elves), (2, 2));
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn a_cost_paid_as_it_entered_is_linked() {
    cr!("607.2g");
    // "As this artifact enters, pay any amount of life." / "{0}: Put X +1/+1 counters on
    // target creature, where X is the life paid as this artifact entered."
    let def = CB::new("Life Processor")
        .artifact()
        .ability(stat(StaticEffect::Replacement(ReplacementDef {
            event: ReplacementEvent::EntersBattlefield(Filter::Source),
            action: ReplacementAction::AsEnters(Box::new(Effect::Seq(vec![
                Effect::Choose {
                    who: PlayerRef::You,
                    kind: ChoiceKind::Number { min: 0, max: 20 },
                },
                Effect::LoseLife {
                    who: PlayerRef::You,
                    n: Value::Chosen,
                },
            ]))),
            self_replacement: true,
            optional: false,
        })))
        .ability(act(
            mana_cost("{0}"),
            Body::simple(
                vec![target_creature()],
                Effect::AddCounters {
                    what: Sel::Target(0),
                    kind: "+1/+1".into(),
                    n: Value::Chosen,
                },
            ),
        ))
        .build();
    let mut t = TestGame::new(2);
    t.answer(P0, DecisionKind::Number, Answer::Number(4));
    let c = t.custom(P0, def, Zone::Nowhere);
    t.g.move_object(c, Zone::Battlefield, MoveCause::Effect, Some(P0));
    let c = t.g.current(c);
    assert_eq!(t.life(P0), 16);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.activate(P0, c, 0, &[Entity::Object(bears)]).unwrap();
    t.resolve();
    assert_eq!(t.pt(bears), (6, 6));
}

#[test]
fn each_kicker_is_linked_to_the_ability_that_refers_to_it() {
    cr!("607.2i");
    // Stormscape Battlemage-like: "Kicker {W} and/or {2}{B}" / "When this creature enters,
    // if it was kicked with its {W} kicker, you gain 3 life." / "When this creature
    // enters, if it was kicked with its {2}{B} kicker, each opponent loses 2 life."
    let mut kicker = Keyword::new(KeywordKind::Kicker);
    kicker.cost = Some(mana_cost("{W}"));
    kicker.costs = vec![mana_cost("{2}{B}")];
    let etb = |cost: &str, effect: Effect| {
        let mut tr = TriggeredAbility::new(
            TriggerCond::EntersBattlefield(Filter::Source),
            Body::effect(effect),
        );
        tr.intervening_if = Some(Condition::CostPaid(format!("kicker {cost}").into()));
        trig_from(tr)
    };
    let def = CB::new("Twin Kicker Mage")
        .creature(2, 2)
        .cost("{2}{U}")
        .ability(AbilityDef::new(AbilityKind::Keyword(kicker), "Kicker"))
        .ability(etb("{W}", gain(3)))
        .ability(etb(
            "{2}{B}",
            Effect::LoseLife {
                who: PlayerRef::EachOpponent,
                n: Value::c(2),
            },
        ))
        .build();
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 3);
    t.lands(P0, "Plains", 1);
    let c = t.custom(P0, def.clone(), Zone::Hand(P0));
    // Pay the {W} kicker only.
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(false));
    t.cast(P0, c).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 23);
    assert_eq!(t.life(P1), 20);
    // Pay the {2}{B} kicker only.
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 3);
    t.lands(P0, "Swamp", 3);
    let c = t.custom(P0, def, Zone::Hand(P0));
    // (The {W} kicker can't be paid with these lands, so only {2}{B} is offered.)
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.cast(P0, c).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.life(P1), 18);
}

#[test]
fn a_variable_additional_cost_is_linked_to_the_ability_that_refers_to_it() {
    cr!("607.2j");
    // Hatred: "As an additional cost to cast this spell, pay X life." / "Target creature
    // gets +X/+0 until end of turn."
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 5);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let h = t.hand(P0, "Hatred");
    t.cast(P0, h).target(bears).x(3).go();
    assert_eq!(t.life(P0), 17);
    t.resolve();
    assert_eq!(t.pt(bears), (5, 2));
}

#[test]
fn anchor_words_are_linked_to_the_choice() {
    cr!("607.2m");
    let def = compile_def(
        "Siege Test",
        "Enchantment",
        "{3}",
        "As Siege Test enters, choose Khans or Dragons.\n• Khans — At the beginning of your upkeep, you gain 2 life.\n• Dragons — At the beginning of your upkeep, each opponent loses 1 life.",
    );
    assert!(abilities(&def)
        .iter()
        .all(|a| !matches!(a.kind, AbilityKind::Unsupported(_))));
    let mut t = TestGame::new(2);
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    let c = t.custom(P0, def, Zone::Nowhere);
    t.g.move_object(c, Zone::Battlefield, MoveCause::Effect, Some(P0));
    t.advance_to(P0, mtg_engine::turn::Step::Upkeep);
    t.settle();
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.life(P1), 19);
}

#[test]
fn cards_exiled_to_pay_a_permanent_spells_cost_are_linked_to_the_permanent() {
    cr!("607.2q");
    // Fear of Abduction: "As an additional cost to cast this spell, exile a creature you
    // control." / "When this creature enters, exile target creature an opponent controls."
    // / "When this creature leaves the battlefield, put each card exiled with it into its
    // owner's hand."
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 6);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Craw Wurm");
    let f = t.hand(P0, "Fear of Abduction");
    t.answer_choose(P0, &[Entity::Object(mine)]);
    t.cast(P0, f).go();
    assert!(t.in_exile("Grizzly Bears"));
    t.answer_targets(P0, &[Entity::Object(theirs)]);
    t.resolve_all();
    assert!(t.in_exile("Craw Wurm"));
    let fear = t.named_on_battlefield("Fear of Abduction")[0];
    t.g.destroy(fear, None);
    t.resolve_all();
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert!(t.in_hand(P1, "Craw Wurm"));
}

#[test]
fn the_exiled_card_means_each_exiled_card_and_values_are_summed() {
    cr!("607.3");
    // "{0}: Exile target card from a graveyard." / "{0}: You gain life equal to the exiled
    // card's mana value." / "{0}: Return the exiled card to its owner's hand."
    let def = CB::new("Double Exiler")
        .artifact()
        .ability(exile_from_graveyard())
        .ability(act(
            mana_cost("{0}"),
            Body::effect(Effect::GainLife {
                who: PlayerRef::You,
                n: Value::ManaValueOf(Box::new(Sel::Linked)),
            }),
        ))
        .ability(act(
            mana_cost("{0}"),
            Body::effect(Effect::Move {
                what: Sel::Linked,
                to: Destination::zone(ZoneKind::Hand),
            }),
        ))
        .build();
    let mut t = TestGame::new(2);
    let a = t.custom(P0, def, Zone::Battlefield);
    let b = t.graveyard(P1, "Grizzly Bears");
    let w = t.graveyard(P1, "Craw Wurm");
    t.activate(P0, a, 0, &[Entity::Object(b)]).unwrap();
    t.resolve();
    t.activate(P0, a, 0, &[Entity::Object(w)]).unwrap();
    t.resolve();
    t.activate(P0, a, 1, &[]).unwrap();
    t.resolve();
    // Grizzly Bears (2) + Craw Wurm (6).
    assert_eq!(t.life(P0), 28);
    t.activate(P0, a, 2, &[]).unwrap();
    t.resolve();
    assert!(t.in_hand(P1, "Grizzly Bears"));
    assert!(t.in_hand(P1, "Craw Wurm"));
}

#[test]
fn linked_abilities_acquired_together_are_linked_only_to_each_other() {
    cr!("607.5");
    // Two artifacts each grant the same creature a pair of linked abilities: "{0}: Exile
    // target card from a graveyard" and "{0}: Return all cards exiled with this creature to
    // their owners' hands".
    let granter = |name: &str| {
        CB::new(name)
            .artifact()
            .ability(stat(StaticEffect::Continuous {
                affected: Filter::Named("Grizzly Bears".into()),
                mods: vec![
                    Modification::AddAbility(exile_from_graveyard()),
                    Modification::AddAbility(return_exiled_to_hand()),
                ],
            }))
            .build()
    };
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.custom(P0, granter("Granter A"), Zone::Battlefield);
    t.custom(P0, granter("Granter B"), Zone::Battlefield);
    t.g.recompute();
    let c1 = t.graveyard(P1, "Craw Wurm");
    let c2 = t.graveyard(P1, "Llanowar Elves");
    // Activated abilities in order: A's exile, A's return, B's exile, B's return.
    t.activate(P0, bears, 0, &[Entity::Object(c1)]).unwrap();
    t.resolve();
    t.activate(P0, bears, 2, &[Entity::Object(c2)]).unwrap();
    t.resolve();
    assert!(t.in_exile("Craw Wurm") && t.in_exile("Llanowar Elves"));
    // B's return ability returns only the card B's exile ability exiled.
    t.activate(P0, bears, 3, &[]).unwrap();
    t.resolve();
    assert!(t.in_hand(P1, "Llanowar Elves"));
    assert!(t.in_exile("Craw Wurm"));
}

#[test]
fn a_choice_that_wasnt_made_for_an_ability_is_undefined() {
    cr!("607.5a");
    // (b) A creature that becomes a copy of Voice of All after it's on the battlefield
    // never chose a color for it: it has no protection.
    let mut t = TestGame::new(2);
    choose_color(&mut t, P0, Color::Red);
    let voice = t.enter(P0, "Voice of All");
    let shifter = t.battlefield(P0, "Grizzly Bears");
    // The shifter becomes a copy of Voice of All.
    let mut ctx = mtg_engine::eval::Ctx::new(Some(shifter), P0);
    ctx.set_var(vars::USER, vec![Entity::Object(voice)]);
    t.g.exec(
        &Effect::BecomeCopy {
            what: Sel::This,
            of: Sel::Var(vars::USER),
            duration: Duration::Permanent,
        },
        &mut ctx,
    );
    t.g.recompute();
    assert_eq!(t.obj(shifter).chars.name.as_str(), "Voice of All");
    let bolt = t.hand(P1, "Lightning Bolt");
    assert!(t.g.protected_from(voice, bolt));
    assert!(!t.g.protected_from(shifter, bolt));
    t.lands(P1, "Mountain", 1);
    t.cast(P1, bolt).target(shifter).go();
    t.resolve_all();
    assert!(!t.on_battlefield(shifter));
    // (a) A permanent with a chosen color gains "{T}: Add one mana of the chosen color"
    // from another object: that ability's choice is undefined, so it adds nothing.
    let mut t = TestGame::new(2);
    choose_color(&mut t, P0, Color::Green);
    let plume = t.enter(P0, "Paradise Plume");
    let grant = CB::new("Mana Granter")
        .artifact()
        .ability(stat(StaticEffect::Continuous {
            affected: Filter::Named("Paradise Plume".into()),
            mods: vec![Modification::AddAbility({
                let mut a = ActivatedAbility::new(
                    Cost::tap(),
                    Body::effect(Effect::AddMana {
                        who: PlayerRef::You,
                        mana: ManaProduction::ChosenColor(Value::c(1)),
                        restriction: None,
                    }),
                );
                a.is_mana_ability = true;
                act_from(a)
            })],
        }))
        .build();
    t.custom(P0, grant, Zone::Battlefield);
    t.g.recompute();
    t.activate(P0, plume, 1, &[]).unwrap();
    assert!(pool(&t, P0).is_empty());
    // Its own linked mana ability still works (on another turn it untaps).
    let mut t = TestGame::new(2);
    choose_color(&mut t, P0, Color::Green);
    let plume = t.enter(P0, "Paradise Plume");
    t.activate(P0, plume, 0, &[]).unwrap();
    assert_eq!(pool(&t, P0), vec![ManaType::G]);
}

#[test]
fn the_two_champion_abilities_are_linked() {
    cr!("607.2k");
    // Changeling Hero: "Champion a creature".
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let wurm = t.battlefield(P0, "Craw Wurm");
    t.answer_choose(P0, &[Entity::Object(bears)]);
    let hero = t.enter(P0, "Changeling Hero");
    t.resolve_all();
    assert!(t.on_battlefield(hero));
    assert!(t.in_exile("Grizzly Bears"));
    // Another card exiled by something else isn't returned by the champion ability.
    let e = t.custom(P0, exile_spell(), Zone::Hand(P0));
    t.cast(P0, e).target(wurm).go();
    t.resolve_all();
    t.g.destroy(hero, None);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert!(t.in_exile("Craw Wurm"));
    // With nothing to champion, it's sacrificed.
    let mut t = TestGame::new(2);
    let hero = t.enter(P0, "Changeling Hero");
    t.resolve_all();
    assert!(!t.on_battlefield(hero));
    assert!(t.in_graveyard(P0, "Changeling Hero"));
}

#[test]
fn cards_exiled_before_the_game_are_linked_to_cards_with_that_name() {
    cr!("607.2n");
    // Arcane Savant: "Before you shuffle your deck to start the game, you may reveal this
    // card from your deck and exile an instant or sorcery card you drafted that isn't in
    // your deck." A card named Arcane Savant refers to "a card you exiled with cards named
    // Arcane Savant" — including cards exiled by another Arcane Savant.
    let fetcher = |name: &str| {
        CB::new(name)
            .creature(3, 3)
            .ability(trig(
                TriggerCond::EntersBattlefield(Filter::Source),
                Body::effect(Effect::Move {
                    what: Sel::ExiledWithCardsNamed("Arcane Savant".into()),
                    to: Destination::zone(ZoneKind::Hand),
                }),
            ))
            .build()
    };
    let mut t = TestGame::new(2);
    t.library_top(P0, "Arcane Savant");
    // The sideboard (cards drafted that aren't in the deck).
    let bolt = t.custom(P0, (*card("Lightning Bolt")).clone(), Zone::Outside(P0));
    let bears = t.custom(P0, (*card("Grizzly Bears")).clone(), Zone::Outside(P0));
    t.g.players[P0.idx()].sideboard.extend([bolt, bears]);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(bolt)]);
    mtg_engine::opening_hand::before_shuffle_actions(&mut t.g);
    assert!(t.in_exile("Lightning Bolt"));
    // A card with another name doesn't refer to it.
    let mut def = fetcher("Other Savant");
    if let Some(a) = def.faces[0].chars.abilities.first().cloned() {
        let mut k = a.kind.clone();
        if let AbilityKind::Triggered(tr) = &mut k {
            tr.body = Body::effect(Effect::Move {
                what: Sel::ExiledWithCardsNamed("Other Savant".into()),
                to: Destination::zone(ZoneKind::Hand),
            });
        }
        def.faces[0].chars.abilities = vec![AbilityDef::new(k, "t")];
    }
    let o = t.custom(P0, def, Zone::Nowhere);
    t.g.move_object(o, Zone::Battlefield, MoveCause::Effect, Some(P0));
    t.resolve_all();
    assert!(t.in_exile("Lightning Bolt"));
    // Another object named Arcane Savant does.
    let s2 = t.custom(P0, fetcher("Arcane Savant"), Zone::Nowhere);
    t.g.move_object(s2, Zone::Battlefield, MoveCause::Effect, Some(P0));
    t.resolve_all();
    assert!(t.in_hand(P0, "Lightning Bolt"));
    assert!(!t.in_exile("Grizzly Bears"));
}

#[test]
fn a_pregame_choice_for_a_cda_is_linked_and_follows_the_card() {
    cr!("607.2p");
    // The Prismatic Piper: "If The Prismatic Piper is your commander, choose a color before
    // the game begins. The Prismatic Piper is the chosen color."
    let mut t = TestGame::new(2);
    let piper = t.command(P0, "The Prismatic Piper");
    t.g.objects[piper.0 as usize].is_commander = true;
    let other = t.hand(P0, "The Prismatic Piper");
    choose_color(&mut t, P0, Color::Blue);
    mtg_engine::opening_hand::before_shuffle_actions(&mut t.g);
    t.g.recompute();
    let blue = {
        let mut c = ColorSet::NONE;
        c.insert(Color::Blue);
        c
    };
    assert_eq!(t.obj(piper).chars.colors, blue);
    // It keeps referring to that choice as it changes zones.
    let on_bf =
        t.g.move_object(piper, Zone::Battlefield, MoveCause::Effect, Some(P0))
            .unwrap();
    t.g.recompute();
    assert_eq!(t.obj(on_bf).chars.colors, blue);
    let in_gy =
        t.g.move_object(on_bf, Zone::Graveyard(P0), MoveCause::Effect, None)
            .unwrap();
    t.g.recompute();
    assert_eq!(t.obj(in_gy).chars.colors, blue);
    // A Piper that isn't a commander had no choice made: it's colorless.
    assert_eq!(t.obj(other).chars.colors, ColorSet::NONE);
}
