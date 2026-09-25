//! CR 608: resolving spells and abilities.

use super::r600_common::*;
use mtg_engine::ability::*;
use mtg_engine::decision::Decision;
use mtg_engine::events::MoveCause;
use mtg_engine::game::GameResult;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn instant(name: &str, body: Body) -> CardDef {
    CB::new(name).instant().cost("{0}").spell(body).build()
}

#[test]
fn the_top_object_resolves_when_all_players_pass_in_succession() {
    cr!("608.1");
    let mut t = TestGame::new(2);
    let a = t.custom(
        P0,
        instant("First Gain", Body::effect(gain(1))),
        Zone::Hand(P0),
    );
    let b = t.custom(
        P0,
        instant("Second Gain", Body::effect(gain(2))),
        Zone::Hand(P0),
    );
    t.cast(P0, a).go();
    t.cast(P0, b).go();
    t.g.turn.passes = 0;
    t.g.turn.priority = Some(P0);
    t.g.pass_priority(P0);
    // Only one player has passed: nothing resolves.
    assert_eq!(t.stack_len(), 2);
    t.g.pass_priority(P1);
    // All passed in succession: the top object (the last cast) resolved.
    assert_eq!(t.stack_len(), 1);
    assert_eq!(t.life(P0), 22);
    assert!(t.in_graveyard(P0, "Second Gain"));
    t.g.pass_priority(P0);
    t.g.pass_priority(P1);
    assert_eq!(t.life(P0), 23);
}

#[test]
fn instructions_are_followed_in_order_and_later_text_can_modify_earlier_text() {
    cr!("608.2", "608.2c");
    // Careful Study: "Draw two cards, then discard two cards." With an empty hand, the
    // draws come first, so the drawn cards are discarded.
    let mut t = TestGame::new(2);
    let cs = t.hand(P0, "Careful Study");
    t.lands(P0, "Island", 1);
    for id in t.g.player(P0).hand.clone() {
        if id != cs {
            t.g.move_object(id, Zone::Library(P0), MoveCause::Effect, None);
        }
    }
    t.cast(P0, cs).go();
    t.resolve();
    assert_eq!(t.hand_size(P0), 0);
    assert_eq!(t.graveyard_size(P0), 3);
    // Terror: "Destroy target nonartifact, nonblack creature. It can't be regenerated."
    // The second sentence modifies the first: a regeneration shield doesn't help.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let shield = CB::new("Shield Test")
        .instant()
        .cost("{0}")
        .spell(Body::simple(
            vec![target_creature()],
            Effect::Regenerate {
                what: Sel::Target(0),
            },
        ))
        .build();
    let s = t.custom(P1, shield, Zone::Hand(P1));
    t.cast(P1, s).target(bears).go();
    t.resolve();
    let terror = t.hand(P0, "Terror");
    t.lands(P0, "Swamp", 2);
    t.cast(P0, terror).target(bears).go();
    t.resolve();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    // Without "It can't be regenerated", the shield works.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let s = t.custom(
        P1,
        CB::new("Shield Test")
            .instant()
            .cost("{0}")
            .spell(Body::simple(
                vec![target_creature()],
                Effect::Regenerate {
                    what: Sel::Target(0),
                },
            ))
            .build(),
        Zone::Hand(P1),
    );
    t.cast(P1, s).target(bears).go();
    t.resolve();
    let blade = t.hand(P0, "Doom Blade");
    t.lands(P0, "Swamp", 2);
    t.cast(P0, blade).target(bears).go();
    t.resolve();
    assert!(t.on_battlefield(bears));
}

#[test]
fn choices_are_made_while_applying_the_effect_and_cant_be_impossible() {
    cr!("608.2d");
    // "You may sacrifice a creature. If you don't, you lose 4 life."
    let def = instant(
        "Grim Bargain",
        Body::effect(Effect::Seq(vec![
            Effect::May {
                who: PlayerRef::You,
                effect: Box::new(Effect::Sacrifice {
                    who: PlayerRef::You,
                    filter: Filter::creature(),
                    count: Value::c(1),
                }),
            },
            Effect::If {
                cond: Condition::Not(Box::new(Condition::PrevHappened)),
                then: Box::new(Effect::LoseLife {
                    who: PlayerRef::You,
                    n: Value::c(4),
                }),
                otherwise: Box::new(Effect::Noop),
            },
        ])),
    );
    // A player who controls no creatures can't choose the sacrifice option.
    let mut t = TestGame::new(2);
    let g = t.custom(P0, def.clone(), Zone::Hand(P0));
    t.answer_yes(P0, true);
    t.cast(P0, g).go();
    t.resolve();
    assert_eq!(t.life(P0), 16);
    // With a creature, the choice is made as it resolves.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let g = t.custom(P0, def, Zone::Hand(P0));
    t.cast(P0, g).go();
    assert!(t
        .asked()
        .iter()
        .all(|(_, d)| !matches!(d, Decision::YesNo { .. })));
    t.answer_yes(P0, true);
    t.resolve();
    assert!(!t.on_battlefield(bears));
    assert_eq!(t.life(P0), 20);
}

#[test]
fn choices_for_each_action_are_made_in_apnap_order_then_it_happens_at_once() {
    cr!("608.2e");
    // "Each player sacrifices a creature." P1 is the active player, so P1 chooses first;
    // then both creatures are sacrificed simultaneously, so a dying "whenever a creature
    // dies" creature sees both.
    let mut t = TestGame::new(2);
    let watcher = t.custom(
        P0,
        CB::new("Death Counter")
            .creature(1, 1)
            .ability(trig(
                TriggerCond::Dies(Filter::creature()),
                Body::effect(gain(1)),
            ))
            .build(),
        Zone::Battlefield,
    );
    t.battlefield(P0, "Grizzly Bears");
    let b1 = t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::PrecombatMain);
    let edict = t.custom(
        P0,
        instant(
            "Mutual Edict",
            Body::effect(Effect::Sacrifice {
                who: PlayerRef::Each(PlayerFilter::Any),
                filter: Filter::creature(),
                count: Value::c(1),
            }),
        ),
        Zone::Hand(P0),
    );
    t.g.turn.priority = Some(P0);
    t.answer_choose(P1, &[Entity::Object(b1)]);
    t.answer_choose(P0, &[Entity::Object(watcher)]);
    t.cast(P0, edict).go();
    t.resolve();
    let choosers: Vec<PlayerId> = t
        .asked()
        .iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseEntities { .. }))
        .map(|(p, _)| *p)
        .collect();
    assert_eq!(choosers, vec![P1, P0]);
    assert!(!t.is_live(watcher));
    assert!(!t.is_live(b1));
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
}

#[test]
fn actions_on_multiple_objects_happen_simultaneously() {
    cr!("608.2f");
    // Destroy all creatures: a creature with "whenever a creature dies" dying along with the
    // others sees all of them die at once.
    let mut t = TestGame::new(2);
    t.custom(
        P0,
        CB::new("Death Counter")
            .creature(1, 1)
            .ability(trig(
                TriggerCond::Dies(Filter::creature()),
                Body::effect(gain(1)),
            ))
            .build(),
        Zone::Battlefield,
    );
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Grizzly Bears");
    let w = t.hand(P0, "Wrath of God");
    t.lands(P0, "Plains", 4);
    t.cast(P0, w).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 23);
    // Each player losing life at once: both reach 0 together and the game is a draw.
    let mut t = TestGame::new(2);
    t.g.players[P0.idx()].life = 2;
    t.g.players[P1.idx()].life = 2;
    let drain = t.custom(
        P0,
        instant(
            "Mutual Drain",
            Body::effect(Effect::LoseLife {
                who: PlayerRef::Each(PlayerFilter::Any),
                n: Value::c(2),
            }),
        ),
        Zone::Hand(P0),
    );
    t.cast(P0, drain).go();
    t.resolve();
    assert_eq!(t.g.result, Some(GameResult::Draw));
}

#[test]
fn players_may_pay_mana_and_cast_spells_during_resolution() {
    cr!("608.2g");
    // "You may pay {2}. If you do, draw two cards." Mana abilities can be activated to pay.
    let def = instant(
        "Paid Insight",
        Body::effect(Effect::PayOptional {
            who: PlayerRef::You,
            cost: mana_cost("{2}"),
            then: Box::new(draw(2)),
            otherwise: Box::new(Effect::Noop),
        }),
    );
    let mut t = TestGame::new(2);
    let lands = t.lands(P0, "Island", 2);
    let c = t.custom(P0, def, Zone::Hand(P0));
    t.cast(P0, c).go();
    let hand = t.hand_size(P0);
    t.answer_yes(P0, true);
    t.resolve();
    assert!(lands.iter().all(|l| t.obj(*l).tapped));
    assert_eq!(t.hand_size(P0), hand + 2);
    // "Cast target instant card from your graveyard without paying its mana cost. You gain
    // 2 life." The cast spell goes on top of the stack and the resolving spell finishes.
    let recast = CB::new("Recast Test")
        .sorcery()
        .cost("{0}")
        .spell(Body::simple(
            vec![TargetSpec::object(
                Filter::and(vec![
                    Filter::Type(CardType::Instant),
                    Filter::InZone(ZoneKind::Graveyard),
                ]),
                "target instant card in your graveyard",
            )],
            Effect::Seq(vec![
                Effect::CastCard {
                    who: PlayerRef::You,
                    what: Sel::Target(0),
                    free: true,
                    optional: false,
                },
                gain(2),
            ]),
        ))
        .build();
    let mut t = TestGame::new(2);
    let bolt = t.graveyard(P0, "Lightning Bolt");
    let r = t.custom(P0, recast, Zone::Hand(P0));
    t.cast(P0, r).target(bolt).go();
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve();
    assert_eq!(t.life(P0), 22);
    assert_eq!(t.stack_len(), 1);
    assert_eq!(t.life(P1), 20);
    t.resolve();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn information_is_determined_when_the_effect_is_applied_using_lki_if_needed() {
    cr!("608.2h");
    // "{0}: This creature deals damage equal to its power to target player." If it leaves
    // the battlefield before the ability resolves, its last known power is used.
    let def = CB::new("Pumped Shooter")
        .creature(3, 3)
        .ability(act(
            mana_cost("{0}"),
            Body::simple(
                vec![TargetSpec::player(PlayerFilter::Any, "target player")],
                Effect::DealDamage {
                    source: Sel::This,
                    amount: Value::PowerOf(Box::new(Sel::This)),
                    to: Sel::Target(0),
                },
            ),
        ))
        .build();
    let mut t = TestGame::new(2);
    let s = t.custom(P0, def, Zone::Battlefield);
    t.activate(P0, s, 0, &[Entity::Player(P1)]).unwrap();
    let gg = t.hand(P0, "Giant Growth");
    t.lands(P0, "Forest", 1);
    t.cast(P0, gg).target(s).go();
    t.resolve();
    t.g.move_object(s, Zone::Hand(P0), MoveCause::Effect, None);
    t.resolve();
    assert_eq!(t.life(P1), 14);
    // An effect that moved an object to a hidden zone uses its last known information:
    // "Return target creature to its owner's hand. You gain life equal to its toughness."
    let bounce = CB::new("Bounce And Gain")
        .instant()
        .cost("{0}")
        .spell(Body::simple(
            vec![target_creature()],
            Effect::Seq(vec![
                Effect::Move {
                    what: Sel::Target(0),
                    to: Destination::zone(ZoneKind::Hand),
                },
                Effect::GainLife {
                    who: PlayerRef::You,
                    n: Value::ToughnessOf(Box::new(Sel::Target(0))),
                },
            ]),
        ))
        .build();
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let gg = t.hand(P1, "Giant Growth");
    t.lands(P1, "Forest", 1);
    t.cast(P1, gg).target(bears).go();
    t.resolve();
    let b = t.custom(P0, bounce, Zone::Hand(P0));
    t.cast(P0, b).target(bears).go();
    t.resolve();
    assert!(t.in_hand(P1, "Grizzly Bears"));
    assert_eq!(t.life(P0), 25);
    // "The number of creatures" is counted once, as the effect is applied.
    let count = instant(
        "Counted Gain",
        Body::effect(Effect::Seq(vec![
            Effect::GainLife {
                who: PlayerRef::You,
                n: Value::Count(Filter::creature()),
            },
            Effect::CreateToken {
                spec: TokenSpec {
                    name: "Soldier".into(),
                    colors: ColorSet::NONE,
                    supertypes: vec![],
                    card_types: vec![CardType::Creature],
                    subtypes: vec![],
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
        ])),
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    let c = t.custom(P0, count, Zone::Hand(P0));
    t.cast(P0, c).go();
    t.resolve();
    assert_eq!(t.life(P0), 21);
}

#[test]
fn look_back_effects_dont_need_objects_to_still_be_where_they_were() {
    cr!("608.2i");
    // "You gain 1 life for each creature that died this turn."
    let def = instant(
        "Mourning Gain",
        Body::effect(Effect::GainLife {
            who: PlayerRef::You,
            n: Value::CreaturesDiedThisTurn,
        }),
    );
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    t.g.destroy(a, None);
    t.g.destroy(b, None);
    t.settle();
    // One of them has since left the graveyard.
    let gone = t.g.current(a);
    t.g.exile_object(gone, None);
    let c = t.custom(P0, def, Zone::Hand(P0));
    t.cast(P0, c).go();
    t.resolve();
    assert_eq!(t.life(P0), 22);
}

#[test]
fn effects_check_only_the_specified_characteristics() {
    cr!("608.2j");
    let wb = || {
        CB::new("Orzhov Test")
            .creature(2, 2)
            .colors(&[Color::White, Color::Black])
            .build()
    };
    let black = compile_def(
        "Black Sweep",
        "Sorcery",
        "{0}",
        "Destroy all black creatures.",
    );
    let nonblack = compile_def(
        "Nonblack Sweep",
        "Sorcery",
        "{0}",
        "Destroy all nonblack creatures.",
    );
    let mut t = TestGame::new(2);
    let c = t.custom(P1, wb(), Zone::Battlefield);
    let s = t.custom(P0, nonblack, Zone::Hand(P0));
    t.cast(P0, s).go();
    t.resolve();
    assert!(t.on_battlefield(c));
    let s = t.custom(P0, black, Zone::Hand(P0));
    t.cast(P0, s).go();
    t.resolve();
    assert!(!t.on_battlefield(c));
}

#[test]
fn an_object_referred_to_by_the_trigger_condition_is_affected_even_if_it_changed() {
    cr!("608.2k");
    // "Whenever a creature enters, return that creature to its owner's hand." The creature
    // stops being a creature before the ability resolves; it's still returned.
    let mut t = TestGame::new(2);
    t.custom(
        P1,
        CB::new("Bouncing Gate")
            .enchantment()
            .ability(trig(
                TriggerCond::EntersBattlefield(Filter::creature()),
                Body::effect(Effect::Move {
                    what: Sel::TriggerObject,
                    to: Destination::zone(ZoneKind::Hand),
                }),
            ))
            .build(),
        Zone::Battlefield,
    );
    let bears = t.enter(P0, "Grizzly Bears");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    let unmake = instant(
        "Unmake Creature",
        Body::simple(
            vec![target_creature()],
            Effect::Modify {
                what: Sel::Target(0),
                mods: vec![Modification::RemoveTypes(vec![CardType::Creature])],
                duration: Duration::EndOfTurn,
            },
        ),
    );
    let u = t.custom(P0, unmake, Zone::Hand(P0));
    t.cast(P0, u).target(bears).go();
    t.resolve();
    assert!(!t.obj(bears).is_creature());
    t.resolve();
    assert!(t.in_hand(P0, "Grizzly Bears"));
}

#[test]
fn a_spell_that_leaves_the_stack_while_resolving_continues_to_resolve() {
    cr!("608.2m");
    // "Return this spell to its owner's hand. You gain 3 life."
    let def = instant(
        "Recursive Gain",
        Body::effect(Effect::Seq(vec![
            Effect::Move {
                what: Sel::This,
                to: Destination::zone(ZoneKind::Hand),
            },
            gain(3),
        ])),
    );
    let mut t = TestGame::new(2);
    let c = t.custom(P0, def, Zone::Hand(P0));
    t.cast(P0, c).go();
    t.resolve();
    assert_eq!(t.life(P0), 23);
    assert!(t.in_hand(P0, "Recursive Gain"));
    assert!(!t.in_graveyard(P0, "Recursive Gain"));
}

#[test]
fn a_spell_goes_to_its_owners_graveyard_last_and_an_ability_ceases_to_exist() {
    cr!("608.2", "608.2n");
    // "You gain 1 life for each card in your graveyard." The spell isn't in the graveyard
    // while it resolves.
    let def = instant(
        "Graveyard Count",
        Body::effect(Effect::GainLife {
            who: PlayerRef::You,
            n: Value::GraveyardSize(PlayerRef::You),
        }),
    );
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Grizzly Bears");
    let c = t.custom(P0, def.clone(), Zone::Hand(P0));
    t.cast(P0, c).go();
    t.resolve();
    assert_eq!(t.life(P0), 21);
    assert!(t.in_graveyard(P0, "Graveyard Count"));
    // Its owner's graveyard, even if another player controlled it.
    let mut t = TestGame::new(2);
    let c = t.custom(P1, def, Zone::Hand(P1));
    t.g.move_object(c, Zone::Hand(P0), MoveCause::Effect, None);
    let c = t.g.current(c);
    t.cast(P0, c).go();
    t.resolve();
    assert!(t.in_graveyard(P1, "Graveyard Count"));
    assert!(!t.in_graveyard(P0, "Graveyard Count"));
    // An ability is removed from the stack and ceases to exist.
    let mut t = TestGame::new(2);
    let src = t.custom(
        P0,
        CB::new("Gain Rock")
            .artifact()
            .ability(act(mana_cost("{0}"), Body::effect(gain(1))))
            .build(),
        Zone::Battlefield,
    );
    let ab = t.activate(P0, src, 0, &[]).unwrap().unwrap();
    t.resolve();
    assert_eq!(t.stack_len(), 0);
    assert!(!t.is_live(ab));
}

#[test]
fn abilities_that_trigger_on_resolution_trigger_after_it_completes() {
    cr!("608.2p");
    // "Whenever an ability of a creature you control resolves, if you have 25 or more
    // life, draw a card." The ability gains 5 life: the trigger sees the finished
    // resolution.
    let mut t = TestGame::new(2);
    let mut watcher = TriggeredAbility::new(
        TriggerCond::AbilityResolved {
            source: Filter::creature().you_control(),
            final_chapter: false,
        },
        Body::effect(draw(1)),
    );
    watcher.intervening_if = Some(Condition::Compare(
        Value::LifeTotal(PlayerRef::You),
        Cmp::Ge,
        Value::c(25),
    ));
    t.custom(
        P0,
        CB::new("Resolution Watcher")
            .enchantment()
            .ability(trig_from(watcher))
            .build(),
        Zone::Battlefield,
    );
    let c = t.custom(
        P0,
        CB::new("Gain Five")
            .creature(1, 1)
            .ability(act(mana_cost("{0}"), Body::effect(gain(5))))
            .build(),
        Zone::Battlefield,
    );
    let hand = t.hand_size(P0);
    t.activate(P0, c, 0, &[]).unwrap();
    t.resolve();
    assert_eq!(t.life(P0), 25);
    assert_eq!(t.stack_len(), 1);
    t.resolve();
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn permanent_spells_with_targets_check_them_and_auras_attach() {
    cr!("608.3", "608.3b", "608.3c");
    // An Aura spell whose target is gone doesn't resolve.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let aura = t.hand(P0, "Pacifism");
    t.lands(P0, "Plains", 2);
    t.cast(P0, aura).target(bears).go();
    t.g.destroy(bears, None);
    t.resolve();
    assert!(t.in_graveyard(P0, "Pacifism"));
    assert!(t.named_on_battlefield("Pacifism").is_empty());
    // With a legal target it enters attached to it.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let aura = t.hand(P0, "Pacifism");
    t.lands(P0, "Plains", 2);
    t.cast(P0, aura).target(bears).go();
    t.resolve();
    let p = t.named_on_battlefield("Pacifism")[0];
    assert_eq!(t.obj(p).attached_to, Some(Entity::Object(bears)));
    assert_eq!(t.obj(p).controller, P0);
}

#[test]
fn a_permanent_spell_that_cant_enter_goes_to_the_graveyard() {
    cr!("608.3e");
    // Worms of the Earth: "Lands can't enter the battlefield." Here: "Creatures can't enter
    // the battlefield."
    let mut t = TestGame::new(2);
    t.custom(
        P1,
        compile_def(
            "Creature Wall Test",
            "Enchantment",
            "{0}",
            "Creatures can't enter the battlefield.",
        ),
        Zone::Battlefield,
    );
    let bears = t.hand(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 2);
    t.cast(P0, bears).go();
    t.resolve();
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    // Worms of the Earth itself.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Worms of the Earth");
    let f = t.hand(P0, "Forest");
    let _ = t.play_land(P0, f);
    assert!(t.named_on_battlefield("Forest").is_empty());
}

#[test]
fn a_copy_of_a_permanent_spell_becomes_a_token() {
    cr!("608.3f");
    let mut t = TestGame::new(2);
    // "Whenever a token is created, you gain 1 life" doesn't see it: it isn't "created".
    t.custom(
        P0,
        CB::new("Token Watcher")
            .enchantment()
            .ability(trig(
                TriggerCond::TokenCreated(Filter::Any),
                Body::effect(gain(1)),
            ))
            .build(),
        Zone::Battlefield,
    );
    let bears = t.hand(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 2);
    let spell = t.cast(P0, bears).go();
    let copier = instant(
        "Copy Test",
        Body::simple(
            vec![TargetSpec {
                what: TargetKind::Spell(Filter::Any),
                ..TargetSpec::object(Filter::Any, "target spell")
            }],
            Effect::CopySpell {
                what: Sel::Target(0),
                count: Value::c(1),
                new_targets: false,
            },
        ),
    );
    let c = t.custom(P0, copier, Zone::Hand(P0));
    t.cast(P0, c).target(spell).go();
    t.resolve_all();
    let all = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(all.len(), 2);
    assert_eq!(all.iter().filter(|b| t.obj(**b).is_token()).count(), 1);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn a_static_ability_on_the_stack_creates_its_delayed_trigger_as_the_permanent_enters() {
    cr!("608.3g");
    // A dash-like creature: "At the beginning of the next end step, return this permanent
    // to its owner's hand" is created as the spell resolves.
    let mut s = StaticAbility::new(StaticEffect::DelayedTriggerAsEnters {
        condition: None,
        trigger: TriggerCond::BeginningOf {
            step: TriggerStep::End,
            whose: PlayerRel::Any,
        },
        body: Body::effect(Effect::Move {
            what: Sel::Var(vars::IT),
            to: Destination::zone(ZoneKind::Hand),
        }),
    });
    s.zone = FunctionZone::Stack;
    let def = CB::new("Dashing Test")
        .creature(2, 2)
        .cost("{0}")
        .keyword(KeywordKind::Haste)
        .ability(stat_from(s))
        .build();
    let mut t = TestGame::new(2);
    let c = t.custom(P0, def.clone(), Zone::Hand(P0));
    t.cast(P0, c).go();
    t.resolve();
    let perm = t.named_on_battlefield("Dashing Test")[0];
    assert_eq!(t.g.delayed_triggers.len(), 1);
    assert_eq!(t.g.delayed_triggers[0].source, Some(perm));
    t.advance_to(P0, Step::End);
    t.settle();
    t.resolve_all();
    assert!(t.in_hand(P0, "Dashing Test"));
    // Put onto the battlefield without resolving as a spell: no delayed trigger.
    let mut t = TestGame::new(2);
    let c = t.custom(P0, def, Zone::Nowhere);
    t.g.move_object(c, Zone::Battlefield, MoveCause::Effect, Some(P0));
    assert!(t.g.delayed_triggers.is_empty());
}
