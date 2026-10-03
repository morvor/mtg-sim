//! CR 121: drawing a card.

use super::r114_common::*;
use mtg_engine::ability::*;
use mtg_engine::decision::Decision;
use mtg_engine::events::Event;
use mtg_engine::game::{GameConfig, Variant};
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn draws(t: &TestGame) -> Vec<(PlayerId, u32)> {
    events_matching(t, |e| matches!(e, Event::Drew { .. }))
        .into_iter()
        .map(|e| match e {
            Event::Drew { player, nth, .. } => (player, nth),
            _ => unreachable!(),
        })
        .collect()
}

fn empty_library(t: &mut TestGame, p: PlayerId) {
    t.g.players[p.idx()].library.clear();
}

fn spell(name: &str, e: Effect) -> CardDef {
    CB::new(name)
        .sorcery()
        .cost("{0}")
        .spell(Body::effect(e))
        .build()
}

fn cast_and_resolve(t: &mut TestGame, p: PlayerId, def: CardDef) {
    let s = t.custom(p, def, Zone::Hand(p));
    t.cast(p, s).go();
    t.resolve();
}

fn static_card(name: &str, text: &str) -> CardDef {
    compile_def(name, "Enchantment", "{2}", text)
}

fn yes_no_asked(t: &TestGame) -> bool {
    t.asked()
        .iter()
        .any(|(_, d)| matches!(d, Decision::YesNo { .. }))
}

#[test]
fn drawing_puts_the_top_card_of_the_library_into_the_hand() {
    cr!("121.1");
    let mut t = TestGame::new(2);
    // As a turn-based action during the draw step.
    let top = t.library_top(P1, "Grizzly Bears");
    t.advance_to(P1, Step::PrecombatMain);
    assert_eq!(t.zone(top), Zone::Hand(P1));
    // As an effect.
    let top0 = t.library_top(P0, "Hill Giant");
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Island", 3);
    let div = t.hand(P0, "Divination");
    t.cast(P0, div).go();
    t.resolve();
    assert_eq!(t.zone(top0), Zone::Hand(P0));
    assert_eq!(t.hand_size(P0), 2);
}

#[test]
fn multiple_draws_are_performed_one_at_a_time() {
    cr!("121.2");
    let mut t = TestGame::new(2);
    let watcher = CB::new("Draw Watcher")
        .enchantment()
        .ability(trig(
            TriggerCond::Draws {
                who: PlayerRel::You,
            },
            Body::effect(gain(1)),
        ))
        .build();
    t.custom(P0, watcher, Zone::Battlefield);
    t.lands(P0, "Island", 3);
    let div = t.hand(P0, "Divination");
    t.cast(P0, div).go();
    t.resolve();
    // Two individual draws: the first and second card drawn this turn, each an event.
    assert_eq!(draws(&t), vec![(P0, 1), (P0, 2)]);
    assert_eq!(t.stack_len(), 2);
}

#[test]
fn replacement_effects_referring_to_the_number_of_cards_drawn_apply_first() {
    cr!("121.2a");
    let mut t = TestGame::new(2);
    t.custom(
        P1,
        static_card(
            "Alms Keeper",
            "If an opponent would draw two or more cards, instead you and that player each draw a card.",
        ),
        Zone::Battlefield,
    );
    t.lands(P0, "Island", 3);
    let div = t.hand(P0, "Divination");
    t.cast(P0, div).go();
    t.resolve();
    assert_eq!(t.hand_size(P0), 1);
    assert_eq!(t.hand_size(P1), 1);
    // A single draw isn't affected.
    cast_and_resolve(&mut t, P0, spell("Peek", draw(1)));
    assert_eq!(t.hand_size(P0), 2);
    assert_eq!(t.hand_size(P1), 1);
}

#[test]
fn cant_draw_more_than_one_card_each_turn_applies_to_individual_draws() {
    cr!("121.2b");
    let mut t = TestGame::new(2);
    t.custom(
        P1,
        static_card(
            "Labyrinth",
            "Each player can't draw more than one card each turn.",
        ),
        Zone::Battlefield,
    );
    // An instruction to draw two cards is partially carried out.
    t.lands(P0, "Island", 3);
    let div = t.hand(P0, "Divination");
    t.cast(P0, div).go();
    t.resolve();
    assert_eq!(t.hand_size(P0), 1);
    // P1 hasn't drawn this turn, but can't choose to draw two cards...
    t.set_step(P1, Step::PrecombatMain);
    cast_and_resolve(
        &mut t,
        P1,
        spell(
            "Offer",
            Effect::May {
                who: PlayerRef::You,
                effect: Box::new(draw(2)),
            },
        ),
    );
    assert_eq!(t.hand_size(P1), 0);
    assert!(!yes_no_asked(&t));
    // ...nor pay a cost that includes drawing two cards.
    let costly = CB::new("Library Tap")
        .artifact()
        .ability(act(
            Cost::free().with(CostPart::Effect(Box::new(draw(2)))),
            Body::effect(gain(1)),
        ))
        .build();
    let c = t.custom(P1, costly, Zone::Battlefield);
    assert!(t.activate(P1, c, 0, &[]).is_err());
}

#[test]
fn when_several_players_draw_the_active_player_draws_first() {
    cr!("121.2c");
    let mut t = TestGame::new(3);
    t.set_step(P1, Step::PrecombatMain);
    cast_and_resolve(
        &mut t,
        P1,
        spell(
            "Howling Wisdom",
            Effect::Draw {
                who: PlayerRef::EachPlayer,
                n: Value::c(2),
            },
        ),
    );
    assert_eq!(
        draws(&t),
        vec![(P1, 1), (P1, 2), (P2, 1), (P2, 2), (P0, 1), (P0, 2)]
    );
}

#[test]
fn with_shared_team_turns_the_active_team_draws_first() {
    cr!("121.2d");
    let mut t = TestGame::with_config(
        4,
        GameConfig {
            variant: Variant::TwoHeadedGiant,
            teams: Some(vec![0, 0, 1, 1]),
            ..Default::default()
        },
    );
    t.set_step(P1, Step::PrecombatMain);
    cast_and_resolve(
        &mut t,
        P1,
        spell(
            "Howling Wisdom",
            Effect::Draw {
                who: PlayerRef::EachPlayer,
                n: Value::c(1),
            },
        ),
    );
    let order: Vec<PlayerId> = draws(&t).into_iter().map(|(p, _)| p).collect();
    // Both players on the active team (P1 and P0), then the nonactive team.
    assert_eq!(order, vec![P1, P0, P2, P3]);
}

#[test]
fn a_player_with_an_empty_library_may_choose_to_draw_but_not_one_who_cant_draw() {
    cr!("121.3");
    let mut t = TestGame::new(2);
    empty_library(&mut t, P0);
    let offer = || {
        spell(
            "Offer",
            Effect::May {
                who: PlayerRef::You,
                effect: Box::new(draw(1)),
            },
        )
    };
    t.answer_yes(P0, true);
    cast_and_resolve(&mut t, P0, offer());
    assert!(yes_no_asked(&t));
    assert!(t.has_lost(P0));
    // With "players can't draw cards", the choice can't be taken.
    let mut t = TestGame::new(2);
    t.custom(
        P1,
        static_card("Mornsong", "Players can't draw cards."),
        Zone::Battlefield,
    );
    t.answer_yes(P0, true);
    cast_and_resolve(&mut t, P0, offer());
    assert!(!yes_no_asked(&t));
    assert_eq!(t.hand_size(P0), 0);
}

#[test]
fn the_same_applies_when_another_player_makes_the_choice() {
    cr!("121.3a");
    let offer = || {
        CB::new("Generous Offer")
            .sorcery()
            .cost("{0}")
            .spell(Body::simple(
                vec![TargetSpec::player(
                    PlayerFilter::Opponent,
                    "target opponent",
                )],
                // "Target opponent may have you draw a card."
                Effect::May {
                    who: PlayerRef::Target(0),
                    effect: Box::new(draw(1)),
                },
            ))
            .build()
    };
    let mut t = TestGame::new(2);
    empty_library(&mut t, P0);
    t.answer_yes(P1, true);
    let s = t.custom(P0, offer(), Zone::Hand(P0));
    t.cast(P0, s).target(P1).go();
    t.resolve();
    assert!(t.has_lost(P0));
    // If P0 can't draw cards, P1 can't choose it.
    let mut t = TestGame::new(2);
    t.custom(
        P0,
        static_card("Quiet Mind", "You can't draw cards."),
        Zone::Battlefield,
    );
    t.answer_yes(P1, true);
    let s = t.custom(P0, offer(), Zone::Hand(P0));
    t.cast(P0, s).target(P1).go();
    t.resolve();
    assert!(!yes_no_asked(&t));
    assert_eq!(t.hand_size(P0), 0);
}

#[test]
fn drawing_from_an_empty_library_loses_the_next_time_state_based_actions_are_checked() {
    cr!("121.4");
    let mut t = TestGame::new(2);
    empty_library(&mut t, P0);
    t.library_top(P0, "Grizzly Bears");
    t.lands(P0, "Island", 3);
    let div = t.hand(P0, "Divination");
    t.cast(P0, div).go();
    t.g.resolve_top();
    // The first draw succeeded; the second was attempted from an empty library.
    assert_eq!(t.hand_size(P0), 1);
    assert!(!t.has_lost(P0));
    t.settle();
    assert!(t.has_lost(P0));
}

#[test]
fn putting_cards_into_the_hand_without_the_word_draw_isnt_drawing() {
    cr!("121.5");
    let mut t = TestGame::new(2);
    let watcher = CB::new("Draw Watcher")
        .enchantment()
        .ability(trig(
            TriggerCond::Draws {
                who: PlayerRel::You,
            },
            Body::effect(gain(1)),
        ))
        .build();
    t.custom(P0, watcher, Zone::Battlefield);
    let take = || {
        spell(
            "Take the Top",
            Effect::Dig {
                who: PlayerRef::You,
                n: Value::c(1),
                reveal: false,
                filter: Filter::Any,
                take: Value::c(1),
                take_up_to: false,
                take_to: Destination::zone(ZoneKind::Hand),
                rest_to: Destination::library_bottom(),
            },
        )
    };
    cast_and_resolve(&mut t, P0, take());
    assert_eq!(t.hand_size(P0), 1);
    assert!(draws(&t).is_empty());
    assert_eq!(t.stack_len(), 0);
    // With an empty library, nothing happens: it isn't an attempt to draw.
    empty_library(&mut t, P0);
    cast_and_resolve(&mut t, P0, take());
    t.settle();
    assert!(!t.has_lost(P0));
}

#[test]
fn draw_replacements_apply_even_with_an_empty_library() {
    cr!("121.6", "121.6a");
    let mut t = TestGame::new(2);
    t.custom(
        P0,
        static_card(
            "Maniac's Notes",
            "If you would draw a card while your library has no cards in it, you win the game instead.",
        ),
        Zone::Battlefield,
    );
    empty_library(&mut t, P0);
    cast_and_resolve(&mut t, P0, spell("Peek", draw(1)));
    assert!(t.player(P0).has_won);
    assert!(!t.has_lost(P0));
}

#[test]
fn a_replaced_draw_in_a_sequence_is_completed_before_the_next_draw() {
    cr!("121.6b");
    let mut t = TestGame::new(2);
    let cup = CB::new("Toasting Cup")
        .artifact()
        .ability(stat(StaticEffect::Replacement(ReplacementDef {
            event: ReplacementEvent::Draw(PlayerFilter::You),
            action: ReplacementAction::Instead(Box::new(Effect::seq(vec![gain(1), draw(1)]))),
            self_replacement: false,
            optional: false,
        })))
        .build();
    t.custom(P0, cup, Zone::Battlefield);
    cast_and_resolve(&mut t, P0, spell("Study", draw(2)));
    let seq: Vec<&str> = events_matching(&t, |e| {
        matches!(e, Event::Drew { .. } | Event::LifeGained { .. })
    })
    .iter()
    .map(|e| match e {
        Event::Drew { .. } => "draw",
        _ => "gain",
    })
    .collect();
    assert_eq!(seq, vec!["gain", "draw", "gain", "draw"]);
}

#[test]
fn additional_actions_on_a_drawn_card_arent_performed_if_the_draw_was_replaced() {
    cr!("121.6c");
    // "Draw a card, then you gain 1 life for each card drawn this way."
    let study = || {
        spell(
            "Careful Study",
            Effect::seq(vec![
                draw(1),
                Effect::GainLife {
                    who: PlayerRef::You,
                    n: Value::CountSel(Box::new(Sel::Var(vars::REVEALED))),
                },
            ]),
        )
    };
    let mut t = TestGame::new(2);
    cast_and_resolve(&mut t, P0, study());
    assert_eq!(t.life(P0), 21);
    // "If you would draw a card, draw two cards instead."
    let double = CB::new("Twofold Tome")
        .artifact()
        .ability(stat(StaticEffect::Replacement(ReplacementDef {
            event: ReplacementEvent::Draw(PlayerFilter::You),
            action: ReplacementAction::Instead(Box::new(draw(2))),
            self_replacement: false,
            optional: false,
        })))
        .build();
    t.custom(P0, double, Zone::Battlefield);
    cast_and_resolve(&mut t, P0, study());
    assert_eq!(t.hand_size(P0), 3);
    assert_eq!(t.life(P0), 21);
}

#[test]
fn draws_from_replacement_effects_happen_after_the_unreplaced_parts_of_the_event() {
    cr!("121.7");
    let mut t = TestGame::new(2);
    // "If a source would deal damage to this creature, prevent that damage. The source's
    // controller draws cards equal to the damage prevented this way."
    let swan = CB::new("Swanlike")
        .creature(1, 4)
        .ability(stat(StaticEffect::Replacement(ReplacementDef {
            event: ReplacementEvent::Damage {
                source: Filter::Any,
                to_players: None,
                to_objects: Some(Filter::Source),
                combat_only: false,
            },
            action: ReplacementAction::Instead(Box::new(Effect::Draw {
                who: PlayerRef::ControllerOf(Box::new(Sel::TriggerOtherObject)),
                n: Value::EventAmount,
            })),
            self_replacement: false,
            optional: false,
        })))
        .build();
    let s = t.custom(P1, swan, Zone::Battlefield);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(
        &[(bears, Entity::Player(P1)), (giant, Entity::Player(P1))],
        &[(s, bears)],
    );
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.hand_size(P0), 2);
    assert_eq!(t.obj(s).damage, 0);
    // The damage to the player happened first, then the draws, one at a time.
    let order: Vec<&str> = events_matching(&t, |e| {
        matches!(e, Event::Drew { .. })
            || matches!(
                e,
                Event::Damage {
                    target: Entity::Player(_),
                    ..
                }
            )
    })
    .iter()
    .map(|e| match e {
        Event::Drew { .. } => "draw",
        _ => "damage",
    })
    .collect();
    assert_eq!(order, vec!["damage", "draw", "draw"]);
}

#[test]
fn a_player_looks_at_a_card_as_they_draw_it_before_choosing_to_reveal_it() {
    cr!("121.9", "702.94a");
    let mut t = TestGame::new(2);
    // Thunderous Wrath: "~ deals 5 damage to any target. Miracle {R}".
    t.library_top(P0, "Thunderous Wrath");
    t.library_top(P0, "Island");
    // The first card P0 draws this turn is the Island: no miracle. Then Thunderous Wrath
    // is the second card drawn: no reveal either.
    let log = spy(&mut t, P0, move |g, _p, d| match d {
        mtg_engine::decision::Decision::YesNo { source, prompt } if prompt.contains("Reveal") => {
            Some(format!(
                "{:?} {:?}",
                g.obj(source.unwrap()).zone,
                g.obj(source.unwrap()).chars.name
            ))
        }
        _ => None,
    });
    t.g.draw_cards(P0, 2);
    assert!(probe_lines(&log).is_empty());
    // Next turn it's the first card drawn: P0 is asked whether to reveal it while it's
    // already in their hand, so they know what it is.
    let mut t = TestGame::new(2);
    t.library_top(P0, "Thunderous Wrath");
    let log = spy(&mut t, P0, move |g, _p, d| match d {
        mtg_engine::decision::Decision::YesNo { source, prompt } if prompt.contains("Reveal") => {
            Some(format!(
                "{:?} {}",
                g.obj(source.unwrap()).zone,
                g.obj(source.unwrap()).chars.name
            ))
        }
        _ => None,
    });
    t.answer_yes(P0, true); // reveal
    t.answer_yes(P0, true); // cast it for its miracle cost
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.lands(P0, "Mountain", 1);
    t.g.draw_cards(P0, 1);
    assert_eq!(
        probe_lines(&log),
        vec![format!("{:?} Thunderous Wrath", Zone::Hand(P0))]
    );
    t.settle();
    t.resolve_all();
    assert_eq!(t.life(P1), 15);
    assert!(t.in_graveyard(P0, "Thunderous Wrath"));
}

/// "Whenever you tap a land for mana, add {G} and draw a card." — a triggered mana
/// ability (CR 605.1b), so it draws while a spell is being cast.
fn drawing_well() -> CardDef {
    compile_def(
        "Drawing Well",
        "Enchantment",
        "{2}{G}",
        "Whenever you tap a land for mana, add {G} and draw a card.",
    )
}

/// A {G}{G} instant with an additional cost of discarding a card matching `filter`.
fn offering(name: &str, filter: Filter) -> CardDef {
    let mut s = StaticAbility::new(StaticEffect::CostModifier(CostModifier {
        applies_to: CostTarget::ThisSpell,
        who: PlayerRel::You,
        change: CostChange::AdditionalCost(Cost::free().with(CostPart::Discard {
            filter,
            count: Value::c(1),
            random: false,
        })),
    }));
    s.zone = FunctionZone::Anywhere;
    CB::new(name)
        .instant()
        .cost("{G}{G}")
        .ability(AbilityDef::new(AbilityKind::Static(s), "additional cost"))
        .spell(Body::effect(gain(3)))
        .build()
}

#[test]
fn a_card_drawn_while_a_spell_is_cast_stays_face_down_until_it_becomes_cast() {
    cr!("121.8");
    // The card drawn by the mana ability has no characteristics while the spell is being
    // cast: it can't be discarded as "a creature card", so the spell can't be cast.
    let setup = |filter: Filter, bears_in_hand: bool| {
        let mut t = TestGame::new(2);
        t.custom(P0, drawing_well(), Zone::Battlefield);
        t.lands(P0, "Forest", 1);
        t.library_top(P0, "Island");
        if bears_in_hand {
            t.hand(P0, "Grizzly Bears");
        } else {
            t.library_top(P0, "Grizzly Bears");
        }
        let s = t.custom(P0, offering("Offering", filter), Zone::Hand(P0));
        (t, s)
    };
    let (mut t, s) = setup(Filter::creature(), false);
    assert!(t.cast(P0, s).try_go().is_err());
    assert!(t.in_hand(P0, "Offering"));
    assert_eq!(t.hand_size(P0), 1);
    // With the creature card already in hand, the same spell can be cast.
    let (mut t, s) = setup(Filter::creature(), true);
    t.cast(P0, s).go();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    // A cost that doesn't need specific characteristics ("discard a card") can use the
    // card drawn while casting.
    let (mut t, s) = setup(Filter::Any, false);
    t.cast(P0, s).go();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    t.resolve_all();
    assert_eq!(t.life(P0), 23);
    // Revealing a card as it's drawn waits until the spell has become cast: a miracle
    // card drawn while casting is revealed afterward.
    let mut t = TestGame::new(2);
    t.custom(P0, drawing_well(), Zone::Battlefield);
    t.lands(P0, "Forest", 1);
    t.library_top(P0, "Thunderous Wrath");
    let gift = t.custom(
        P0,
        CB::new("Green Gift")
            .instant()
            .cost("{G}{G}")
            .spell(Body::effect(gain(1)))
            .build(),
        Zone::Hand(P0),
    );
    let log = spy(&mut t, P0, |g, _p, d| match d {
        Decision::YesNo { prompt, .. } if prompt.contains("Reveal") => Some(format!(
            "stack={} spell_cast={}",
            g.stack.len(),
            g.history.spells_cast.len()
        )),
        _ => None,
    });
    t.answer_yes(P0, false);
    t.cast(P0, gift).go();
    assert_eq!(probe_lines(&log), vec!["stack=1 spell_cast=1".to_string()]);
}
