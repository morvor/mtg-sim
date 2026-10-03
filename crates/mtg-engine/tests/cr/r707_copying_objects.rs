//! CR 707.1–707.9: copying objects (permanents and cards).

use crate::r703_common::*;
use mtg_engine::ability::*;
use mtg_engine::keywords::{Keyword, KeywordKind};
use mtg_engine::object::{Characteristics, Zone};
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;
use smol_str::SmolStr;
use std::sync::Arc;

/// `P0`'s Clone enters as a copy of `what`.
fn clone_of(t: &mut TestGame, what: ObjectId) -> ObjectId {
    t.answer_choose(P0, &[Entity::Object(what)]);
    let c = t.enter(P0, "Clone");
    t.g.current(c)
}

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

/// `obj` becomes a copy of `of` (a copy effect from a resolving ability).
fn become_copy(t: &mut TestGame, obj: ObjectId, of: ObjectId) {
    let mut ctx = mtg_engine::eval::Ctx::new(None, P0);
    ctx.targets = vec![vec![Entity::Object(obj)], vec![Entity::Object(of)]];
    t.g.exec(
        &Effect::BecomeCopy {
            what: Sel::Target(0),
            of: Sel::Target(1),
            duration: Duration::Permanent,
        },
        &mut ctx,
    );
    t.g.recompute();
    t.g.flush_events();
}

/// A token copy of `of` created by an effect `p` controls, with `mods` as exceptions.
fn token_copy(t: &mut TestGame, p: PlayerId, of: ObjectId, mods: Vec<Modification>) -> ObjectId {
    let before = t.g.objects.len();
    run_effect(
        t,
        p,
        None,
        Effect::CreateTokenCopy {
            of: Sel::Target(0),
            count: Value::c(1),
            controller: PlayerRef::You,
            tapped: false,
            attacking: false,
            mods,
        },
        &[Entity::Object(of)],
    );
    (before..t.g.objects.len())
        .map(|i| ObjectId(i as u32))
        .find(|id| t.g.is_live(*id) && t.obj(*id).zone == Zone::Battlefield)
        .expect("token created")
}

fn replacement(
    event: ReplacementEvent,
    action: ReplacementAction,
    self_replacement: bool,
) -> Ability {
    AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Replacement(
            ReplacementDef {
                event,
                action,
                self_replacement,
                optional: false,
            },
        ))),
        "replacement",
    )
}

/// A Clone-like creature card: "You may have this creature enter as a copy of any
/// creature on the battlefield[, except ...]".
fn cloner(name: &str, p: i32, t: i32, colors: ColorSet, except: Vec<Effect>) -> CardDef {
    let mut abilities = vec![replacement(
        ReplacementEvent::EntersBattlefield(Filter::Source),
        ReplacementAction::EnterAsCopy {
            filter: Filter::creature(),
            optional: true,
        },
        false,
    )];
    for e in except {
        abilities.push(replacement(
            ReplacementEvent::EntersBattlefield(Filter::Source),
            ReplacementAction::AsEnters(Box::new(e)),
            true,
        ));
    }
    CardDef::custom(Characteristics {
        name: SmolStr::new(name),
        rules_text: Arc::from(""),
        mana_cost: mtg_engine::mana::ManaCost::parse("{0}"),
        colors,
        card_types: CardTypeSet::single(CardType::Creature),
        subtypes: [SmolStr::new("Shapeshifter")].into_iter().collect(),
        power: Some(p),
        toughness: Some(t),
        abilities,
        ..Default::default()
    })
}

fn enter_custom(t: &mut TestGame, p: PlayerId, def: CardDef, copy: Option<ObjectId>) -> ObjectId {
    if let Some(c) = copy {
        t.answer_choose(p, &[Entity::Object(c)]);
    } else {
        t.answer_choose(p, &[]);
    }
    let id = t.custom(p, def, Zone::Hand(p));
    let new =
        t.g.move_object_ev(mtg_engine::replacement::MoveEv {
            obj: id,
            to: Zone::Battlefield,
            pos: LibraryPosition::Top,
            cause: mtg_engine::events::MoveCause::Effect,
            by: Some(p),
            etb: mtg_engine::replacement::EtbInfo {
                controller: Some(p),
                ..Default::default()
            },
            source: None,
        })
        .expect("entered");
    t.g.flush_events();
    new
}

#[test]
fn permanents_can_become_or_create_copies_of_objects() {
    cr!("707.1");
    supported("Cackling Counterpart");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    // A permanent that enters as a copy.
    let c = clone_of(&mut t, giant);
    assert_eq!(t.obj(c).chars.name.as_str(), "Hill Giant");
    // A token that's a copy.
    t.lands(P0, "Island", 3);
    let cc = t.hand(P0, "Cackling Counterpart");
    t.cast(P0, cc).target(giant).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 3);
    // A copy of a spell.
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    let spell = t.cast(P0, bolt).target(Entity::Player(P1)).go();
    run_effect(
        &mut t,
        P0,
        None,
        Effect::CopySpell {
            what: Sel::Target(0),
            count: Value::c(1),
            new_targets: false,
        },
        &[Entity::Object(spell)],
    );
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(t.life(P1), 14);
}

#[test]
fn a_copy_gets_color_and_abilities_from_the_copiable_values_once() {
    cr!("707.2a");
    supported("Wall of Omens");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    // Effects make the Bears blue and give it flying: not copiable values.
    modify(
        &mut t,
        bears,
        vec![
            Modification::SetColors(ColorSet::single(Color::Blue)),
            Modification::AddKeyword(Keyword::new(KeywordKind::Flying)),
        ],
    );
    let c = clone_of(&mut t, bears);
    assert_eq!(t.obj(c).chars.colors, ColorSet::single(Color::Green));
    assert!(!t.obj(c).has_keyword(KeywordKind::Flying));
    // Its abilities come from the copied rules text once: one "When this creature
    // enters, draw a card".
    let wall = t.battlefield(P1, "Wall of Omens");
    let hand = t.hand_size(P0);
    let w = clone_of(&mut t, wall);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    let triggers = t
        .obj(w)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, AbilityKind::Triggered(_)))
        .count();
    assert_eq!(triggers, 1);
}

#[test]
fn a_copy_doesnt_change_when_the_original_does() {
    cr!("707.2b");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let c = clone_of(&mut t, giant);
    // The Giant's copiable values change: it becomes a copy of the Bears.
    become_copy(&mut t, giant, bears);
    assert_eq!(t.obj(giant).chars.name.as_str(), "Grizzly Bears");
    assert_eq!(t.obj(c).chars.name.as_str(), "Hill Giant");
    assert_eq!(t.pt(c), (3, 3));
}

#[test]
fn a_static_copy_effect_copies_values_as_it_starts_to_apply() {
    cr!("707.2c");
    let mut t = TestGame::new(2);
    // "Creatures you control enter as a copy of this creature." (Essence of the Wild)
    let mut essence = oracle_card(
        "Essence of Sameness",
        "Creature — Avatar",
        "{0}",
        Some((6, 6)),
        "",
    );
    essence.faces[0].chars.abilities.push(replacement(
        ReplacementEvent::EntersBattlefield(Filter::And(vec![
            Filter::creature(),
            Filter::ControlledBy(PlayerRel::You),
            Filter::Not(Box::new(Filter::Source)),
        ])),
        ReplacementAction::EnterAsCopy {
            filter: Filter::Source,
            optional: false,
        },
        false,
    ));
    let e = t.custom(P0, essence, Zone::Battlefield);
    let bears = t.enter(P0, "Grizzly Bears");
    assert_eq!(t.obj(bears).chars.name.as_str(), "Essence of Sameness");
    assert_eq!(t.pt(bears), (6, 6));
    // Later changes to the Essence's copiable values don't change what the Bears copied.
    let giant = t.battlefield(P1, "Hill Giant");
    become_copy(&mut t, e, giant);
    assert_eq!(t.obj(e).chars.name.as_str(), "Hill Giant");
    assert_eq!(t.obj(bears).chars.name.as_str(), "Essence of Sameness");
    assert_eq!(t.pt(bears), (6, 6));
}

#[test]
fn copying_a_copy_uses_its_new_copiable_values() {
    cr!("707.3");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let first = clone_of(&mut t, bears);
    let second = clone_of(&mut t, first);
    assert_eq!(t.obj(second).chars.name.as_str(), "Grizzly Bears");
    assert_eq!(t.pt(second), (2, 2));
    // A copy exception is part of the copy's copiable values too.
    let blue = cloner(
        "Blue Mimic",
        0,
        0,
        ColorSet::single(Color::Blue),
        vec![Effect::EnterCopyExceptions(vec![Modification::SetColors(
            ColorSet::single(Color::Blue),
        )])],
    );
    let m = enter_custom(&mut t, P0, blue, Some(bears));
    let third = clone_of(&mut t, m);
    assert_eq!(t.obj(third).chars.name.as_str(), "Grizzly Bears");
    assert_eq!(t.obj(third).chars.colors, ColorSet::single(Color::Blue));
}

#[test]
fn becoming_a_copy_of_something_else_isnt_entering_or_leaving() {
    cr!("707.4");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let wall = t.battlefield(P1, "Wall of Omens");
    let c = clone_of(&mut t, bears);
    t.lands(P0, "Forest", 1);
    let gg = t.hand(P0, "Giant Growth");
    t.cast(P0, gg).target(c).go();
    t.resolve_all();
    assert_eq!(t.pt(c), (5, 5));
    // It becomes a copy of Wall of Omens: its "When this creature enters, draw a card"
    // doesn't trigger, and Giant Growth still applies.
    let hand = t.hand_size(P0);
    become_copy(&mut t, c, wall);
    t.resolve_all();
    assert_eq!(t.obj(c).chars.name.as_str(), "Wall of Omens");
    assert!(t.on_battlefield(c));
    assert_eq!(t.hand_size(P0), hand);
    assert_eq!(t.pt(c), (3, 7));
}

#[test]
fn entering_as_a_copy_includes_the_copied_entry_abilities() {
    cr!("707.5");
    let mut t = TestGame::new(2);
    let slow = oracle_card(
        "Slow Beast",
        "Creature — Beast",
        "{0}",
        Some((3, 3)),
        "This creature enters tapped.\nThis creature enters with two +1/+1 counters on it.",
    );
    let beast = t.custom(P1, slow, Zone::Battlefield);
    let wall = t.battlefield(P1, "Wall of Omens");
    // The copied "enters tapped" and "enters with counters" abilities apply to the Clone.
    let c = clone_of(&mut t, beast);
    assert!(t.obj(c).tapped);
    assert_eq!(t.counters(c, counters::PLUS1), 2);
    // The copied "When this creature enters" ability triggers.
    let hand = t.hand_size(P0);
    clone_of(&mut t, wall);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn choices_made_for_the_original_arent_copied() {
    cr!("707.6");
    supported("Adaptive Automaton");
    let pick = |t: &mut TestGame, p: PlayerId, ty: &str| {
        let i = mtg_engine::types::subtype_lists()
            .creature
            .iter()
            .position(|s| s == ty)
            .unwrap();
        t.answer(p, DecisionKind::Option, Answer::Index(i));
    };
    let mut t = TestGame::new(2);
    let elf = t.battlefield(P0, "Llanowar Elves");
    let goblin = t.battlefield(P0, "Raging Goblin");
    pick(&mut t, P0, "Elf");
    let a = t.enter(P0, "Adaptive Automaton");
    // The Clone's controller makes a new choice as it enters.
    pick(&mut t, P0, "Goblin");
    let c = clone_of(&mut t, a);
    assert_eq!(t.obj(c).choices.creature_type.as_deref(), Some("Goblin"));
    assert_eq!(t.obj(a).choices.creature_type.as_deref(), Some("Elf"));
    assert_eq!(t.pt(elf), (2, 2));
    assert_eq!(t.pt(goblin), (2, 2));
}

#[test]
fn copied_linked_abilities_stay_linked_to_each_other() {
    cr!("707.7");
    supported("Fiend Hunter");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[Entity::Object(a)]);
    t.answer_yes(P0, true);
    let hunter = t.enter(P0, "Fiend Hunter");
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    // A Clone copying Fiend Hunter exiles the Giant with its copy of the first ability.
    t.answer_targets(P0, &[Entity::Object(b)]);
    t.answer_yes(P0, true);
    let c = clone_of(&mut t, hunter);
    t.resolve_all();
    assert!(t.in_exile("Hill Giant"));
    // When the Clone leaves, its second ability returns the card its first exiled, not
    // the one the original exiled.
    t.g.destroy(c, None);
    t.settle();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
    assert!(t.in_exile("Grizzly Bears"));
}

#[test]
fn copying_a_double_faced_permanent_copies_the_face_up() {
    cr!("707.8");
    let mut t = TestGame::new(2);
    let delver = t.battlefield(P1, "Delver of Secrets");
    assert!(mtg_engine::dfc::transform(&mut t.g, delver));
    t.g.recompute();
    assert_eq!(t.obj(delver).chars.name.as_str(), "Insectile Aberration");
    let c = clone_of(&mut t, delver);
    assert_eq!(t.obj(c).chars.name.as_str(), "Insectile Aberration");
    assert_eq!(t.pt(c), (3, 2));
    assert!(t.obj(c).has_keyword(KeywordKind::Flying));
    // The Clone isn't a double-faced card: it can't transform.
    assert!(!mtg_engine::dfc::transform(&mut t.g, c));
}

#[test]
fn a_token_copy_of_a_double_faced_object_has_both_faces() {
    cr!("707.8a");
    let mut t = TestGame::new(2);
    let delver = t.battlefield(P1, "Delver of Secrets");
    assert!(mtg_engine::dfc::transform(&mut t.g, delver));
    t.g.recompute();
    // It enters with the same face up as the permanent it copies, and can transform.
    let tok = token_copy(&mut t, P0, delver, vec![]);
    assert_eq!(t.obj(tok).chars.name.as_str(), "Insectile Aberration");
    assert!(mtg_engine::dfc::transform(&mut t.g, tok));
    t.g.recompute();
    assert_eq!(t.obj(tok).chars.name.as_str(), "Delver of Secrets");
    assert_eq!(t.pt(tok), (1, 1));
    // A token copy of a double-faced card that isn't on the battlefield has its front
    // face up.
    let card = t.graveyard(P1, "Delver of Secrets");
    let tok2 = token_copy(&mut t, P0, card, vec![]);
    assert_eq!(t.obj(tok2).chars.name.as_str(), "Delver of Secrets");
    assert!(mtg_engine::dfc::transform(&mut t.g, tok2));
    // A token copy of a Clone card entering as a copy of a double-faced permanent (by
    // Clone's replacement effect) isn't a double-faced token.
    let clone_card = t.graveyard(P0, "Clone");
    t.answer_choose(P0, &[Entity::Object(delver)]);
    let tok3 = token_copy(&mut t, P0, clone_card, vec![]);
    assert_eq!(t.obj(tok3).chars.name.as_str(), "Insectile Aberration");
    assert!(!mtg_engine::dfc::transform(&mut t.g, tok3));
}

#[test]
fn an_ability_gained_in_copying_is_part_of_the_copiable_values() {
    cr!("707.9a");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    // "Create a token that's a copy of it, except it has haste."
    let tok = token_copy(
        &mut t,
        P0,
        bears,
        vec![Modification::AddKeyword(Keyword::new(KeywordKind::Haste))],
    );
    assert!(t.obj(tok).has_keyword(KeywordKind::Haste));
    // A Clone copying the token has haste too.
    let c = clone_of(&mut t, tok);
    assert!(t.obj(c).has_keyword(KeywordKind::Haste));
    assert!(!t.obj(bears).has_keyword(KeywordKind::Haste));
}

#[test]
fn a_copy_effect_can_retain_the_original_value_of_a_characteristic() {
    cr!("707.9c");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    // "... except it doesn't copy that creature's color": it keeps its own (blue).
    let doppel = cloner(
        "Doppel",
        0,
        0,
        ColorSet::single(Color::Blue),
        vec![Effect::EnterCopyExceptions(vec![Modification::SetColors(
            ColorSet::single(Color::Blue),
        )])],
    );
    let d = enter_custom(&mut t, P0, doppel, Some(bears));
    assert_eq!(t.obj(d).chars.name.as_str(), "Grizzly Bears");
    assert_eq!(t.obj(d).chars.colors, ColorSet::single(Color::Blue));
}

/// A Clone-like 0/0 whose exception is "it enters with two additional +1/+1 counters on
/// it" (Altered Ego).
fn altered_ego() -> CardDef {
    cloner(
        "Altered Egg",
        0,
        0,
        ColorSet::NONE,
        vec![Effect::EnterCopyExtra {
            only_if: None,
            mods: vec![],
            effect: Box::new(Effect::EnterWithCounters {
                kind: counters::PLUS1.into(),
                n: Value::c(2),
            }),
        }],
    )
}

#[test]
fn an_additional_effect_exception_doesnt_happen_if_another_copy_effect_applies_later() {
    cr!("707.9e");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    // Copying the Giant: it enters with two additional +1/+1 counters.
    let e = enter_custom(&mut t, P0, altered_ego(), Some(giant));
    assert_eq!(t.obj(e).chars.name.as_str(), "Hill Giant");
    assert_eq!(t.counters(e, counters::PLUS1), 2);
    // Copying a Clone-like creature (a 2/2 that copied nothing) whose own replacement
    // effect then chooses the Giant: that later copy effect wins, and the counters
    // aren't put on it.
    let copycat = t.custom(
        P1,
        cloner("Copycat", 2, 2, ColorSet::NONE, vec![]),
        Zone::Battlefield,
    );
    t.answer_choose(P0, &[Entity::Object(copycat)]);
    let e2 = enter_custom(&mut t, P0, altered_ego(), Some(giant));
    assert_eq!(t.obj(e2).chars.name.as_str(), "Hill Giant");
    assert_eq!(t.counters(e2, counters::PLUS1), 0);
}

#[test]
fn a_conditional_exception_looks_at_the_copy_without_it() {
    cr!("707.9f");
    // "... except, if it's a creature, it enters with two additional +1/+1 counters on it
    // and has flying." (Moritte of the Frost)
    let moritte = || {
        cloner(
            "Frosty Mimic",
            2,
            2,
            ColorSet::NONE,
            vec![Effect::EnterCopyExtra {
                only_if: Some(Filter::creature()),
                mods: vec![Modification::AddKeyword(Keyword::new(KeywordKind::Flying))],
                effect: Box::new(Effect::EnterWithCounters {
                    kind: counters::PLUS1.into(),
                    n: Value::c(2),
                }),
            }],
        )
    };
    let mut t = TestGame::new(2);
    // Copying a land that's a creature until end of turn: the copy isn't a creature.
    let mut def = moritte();
    if let ReplacementAction::EnterAsCopy { filter, .. } = replacement_action_mut(&mut def) {
        *filter = Filter::Permanent;
    }
    let land = t.battlefield(P1, "Forest");
    modify(
        &mut t,
        land,
        vec![
            Modification::AddTypes(vec![CardType::Creature]),
            Modification::SetPT(Some(Value::c(1)), Some(Value::c(1))),
        ],
    );
    let m = enter_custom(&mut t, P0, def, Some(land));
    assert_eq!(t.obj(m).chars.name.as_str(), "Forest");
    assert!(!t.obj(m).is_creature());
    assert_eq!(t.counters(m, counters::PLUS1), 0);
    assert!(!t.obj(m).has_keyword(KeywordKind::Flying));
    // Copying a creature: it has flying (part of its copiable values) and the counters.
    let bears = t.battlefield(P1, "Grizzly Bears");
    let m2 = enter_custom(&mut t, P0, moritte(), Some(bears));
    assert_eq!(t.counters(m2, counters::PLUS1), 2);
    assert!(t.obj(m2).has_keyword(KeywordKind::Flying));
    assert!(t.obj(m2).copiable.abilities.iter().any(|a| matches!(
        &a.kind,
        AbilityKind::Keyword(k) if k.kind == KeywordKind::Flying
    )));
}

/// The EnterAsCopy action of a `cloner` definition.
fn replacement_action_mut(def: &mut CardDef) -> &mut ReplacementAction {
    for a in def.faces[0].chars.abilities.iter_mut() {
        if let AbilityKind::Static(s) = &mut Arc::make_mut(a).kind {
            if let StaticEffect::Replacement(r) = &mut s.effect {
                if matches!(r.action, ReplacementAction::EnterAsCopy { .. }) {
                    return &mut r.action;
                }
            }
        }
    }
    panic!("no copy replacement")
}

#[test]
fn a_linked_trigger_doesnt_trigger_if_another_copy_effect_applies_later() {
    cr!("707.9g");
    // "... except it's a Wall in addition to its other types. When you do, you gain 5
    // life." (Wall of Stolen Identity's shape)
    let wall = || {
        cloner(
            "Stolen Face",
            0,
            0,
            ColorSet::NONE,
            vec![Effect::EnterCopyExtra {
                only_if: None,
                mods: vec![Modification::AddSubtypes(vec!["Wall".into()])],
                effect: Box::new(Effect::Reflexive {
                    body: Box::new(Body::effect(Effect::GainLife {
                        who: PlayerRef::You,
                        n: Value::c(5),
                    })),
                }),
            }],
        )
    };
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let w = enter_custom(&mut t, P0, wall(), Some(giant));
    t.resolve_all();
    assert!(t.obj(w).chars.has_subtype("Wall"));
    assert_eq!(t.life(P0), 25);
    // Copying a Clone-like creature that then copies the Giant: no trigger.
    let copycat = t.custom(
        P1,
        cloner("Copycat", 2, 2, ColorSet::NONE, vec![]),
        Zone::Battlefield,
    );
    t.answer_choose(P0, &[Entity::Object(copycat)]);
    let w2 = enter_custom(&mut t, P0, wall(), Some(giant));
    t.resolve_all();
    assert_eq!(t.obj(w2).chars.name.as_str(), "Hill Giant");
    assert!(!t.obj(w2).chars.has_subtype("Wall"));
    assert_eq!(t.life(P0), 25);
}
