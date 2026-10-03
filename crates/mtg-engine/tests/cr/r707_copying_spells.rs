//! CR 707.10–707.14: copying spells and abilities, and copies of cards.

use crate::r703_common::*;
use mtg_engine::ability::*;
use mtg_engine::copy_rules::NamedCopy;
use mtg_engine::decision::Decision;
use mtg_engine::keywords::{Keyword, KeywordKind};
use mtg_engine::object::{ObjKind, Zone};
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// Resolves an effect controlled by `p` with `targets` in slot 0 and `more` in slot 1.
fn run_with(t: &mut TestGame, p: PlayerId, effect: Effect, targets: &[Entity], more: &[Entity]) {
    let mut ctx = mtg_engine::eval::Ctx::new(None, p);
    ctx.targets = vec![targets.to_vec(), more.to_vec()];
    t.g.exec(&effect, &mut ctx);
    t.g.recompute();
    t.g.flush_events();
}

fn copy_spell(t: &mut TestGame, p: PlayerId, spell: ObjectId, new_targets: bool) {
    run_with(
        t,
        p,
        Effect::CopySpell {
            what: Sel::Target(0),
            count: Value::c(1),
            new_targets,
        },
        &[Entity::Object(spell)],
        &[],
    );
}

/// "Ping deals 1 damage to target creature."
fn ping() -> CardDef {
    oracle_card(
        "Ping",
        "Instant",
        "{0}",
        None,
        "Ping deals 1 damage to target creature.",
    )
}

fn targets_of(t: &TestGame, id: ObjectId) -> Vec<Entity> {
    t.obj(id)
        .stack
        .as_deref()
        .map(|si| {
            si.chosen
                .iter()
                .flat_map(|m| m.targets.iter().flatten().copied())
                .collect()
        })
        .unwrap_or_default()
}

#[test]
fn a_copy_of_a_spell_or_card_ceases_to_exist_outside_its_zones() {
    cr!("707.10a", "704.5e");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    let spell = t.cast(P0, bolt).target(Entity::Player(P1)).go();
    copy_spell(&mut t, P0, spell, false);
    let copy = *t.g.stack.last().unwrap();
    assert_eq!(t.obj(copy).kind, ObjKind::SpellCopy);
    t.resolve_all();
    // The resolved copy went to the graveyard and then ceased to exist.
    assert_eq!(t.life(P1), 14);
    assert_eq!(t.graveyard_size(P0), 1);
    assert_eq!(t.obj(t.g.current(copy)).zone, Zone::Nowhere);
}

#[test]
fn a_copy_of_an_ability_has_the_same_source_and_is_the_same_ability() {
    cr!("707.10b");
    let mut t = TestGame::new(2);
    let adept = t.custom(
        P0,
        oracle_card(
            "Echo Adept",
            "Creature — Human Wizard",
            "{0}",
            Some((1, 1)),
            "{0}: Put a +1/+1 counter on ~. When this ability resolves for the second time this turn, draw a card.",
        ),
        Zone::Battlefield,
    );
    let other = t.custom(
        P0,
        oracle_card(
            "Echo Adept",
            "Creature — Human Wizard",
            "{0}",
            Some((1, 1)),
            "",
        ),
        Zone::Battlefield,
    );
    let ability = t.activate(P0, adept, 0, &[]).unwrap().unwrap();
    copy_spell(&mut t, P0, ability, false);
    assert_eq!(t.stack_len(), 2);
    let hand = t.hand_size(P0);
    t.resolve_all();
    // Both put their counter on the source (not another object with the same name), and
    // the copy counts as the same ability resolving: the second resolution draws.
    assert_eq!(t.counters(adept, counters::PLUS1), 2);
    assert_eq!(t.counters(other, counters::PLUS1), 0);
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn new_targets_for_a_copy_are_optional_but_must_be_legal() {
    cr!("707.10c");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    let spell = cast_and_keep(&mut t, P0, ping(), a);
    // The Bears gains shroud: the original target is now illegal.
    run_with(
        &mut t,
        P1,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::AddKeyword(Keyword::new(KeywordKind::Shroud))],
            duration: Duration::EndOfTurn,
        },
        &[Entity::Object(a)],
        &[],
    );
    // P0 may leave the target unchanged even though it's illegal.
    t.answer_yes(P0, false);
    copy_spell(&mut t, P0, spell, true);
    let copy = *t.g.stack.last().unwrap();
    assert_eq!(targets_of(&t, copy), vec![Entity::Object(a)]);
    // Changing it: only legal targets can be chosen (not the Bears).
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(b)]);
    copy_spell(&mut t, P0, spell, true);
    let copy2 = *t.g.stack.last().unwrap();
    assert_eq!(targets_of(&t, copy2), vec![Entity::Object(b)]);
    let offered: Vec<Entity> = t
        .asked()
        .into_iter()
        .rev()
        .find_map(|(_, d)| match d {
            Decision::ChooseTargets { candidates, .. } => Some(candidates),
            _ => None,
        })
        .unwrap();
    assert!(!offered.contains(&Entity::Object(a)));
    t.resolve_all();
    assert_eq!(t.obj(b).damage, 1);
    assert_eq!(t.obj(a).damage, 0);
}

fn cast_and_keep(t: &mut TestGame, p: PlayerId, def: CardDef, target: ObjectId) -> ObjectId {
    let card = t.custom(p, def, Zone::Hand(p));
    t.cast(p, card).target(target).go()
}

#[test]
fn a_spell_copied_for_each_other_object_it_could_target() {
    cr!("707.10d");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    let c = t.battlefield(P0, "Hill Giant");
    // One with hexproof can't be targeted by P0's copies.
    let d = t.battlefield(P1, "Grizzly Bears");
    run_with(
        &mut t,
        P1,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::AddKeyword(Keyword::new(
                KeywordKind::Hexproof,
            ))],
            duration: Duration::EndOfTurn,
        },
        &[Entity::Object(d)],
        &[],
    );
    let spell = cast_and_keep(&mut t, P0, ping(), a);
    run_with(
        &mut t,
        P0,
        Effect::CopySpellRetargeted {
            what: Sel::Target(0),
            target: None,
        },
        &[Entity::Object(spell)],
        &[],
    );
    assert_eq!(t.stack_len(), 3);
    let mut copied: Vec<Entity> = t.g.stack[1..]
        .iter()
        .flat_map(|s| targets_of(&t, *s))
        .collect();
    copied.sort();
    let mut expect = vec![Entity::Object(b), Entity::Object(c)];
    expect.sort();
    assert_eq!(copied, expect);
    t.resolve_all();
    for x in [a, b, c] {
        assert_eq!(t.obj_now(x).damage, 1);
    }
    assert_eq!(t.obj(d).damage, 0);
}

#[test]
fn a_spell_copied_with_a_specified_new_target() {
    cr!("707.10e");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let token_like = t.battlefield(P0, "Llanowar Elves");
    let spell = cast_and_keep(&mut t, P0, ping(), a);
    // "Copy that spell. The copy targets [it]."
    run_with(
        &mut t,
        P0,
        Effect::CopySpellRetargeted {
            what: Sel::Target(0),
            target: Some(Sel::Target(1)),
        },
        &[Entity::Object(spell)],
        &[Entity::Object(token_like)],
    );
    assert_eq!(t.stack_len(), 2);
    let copy = *t.g.stack.last().unwrap();
    assert_eq!(targets_of(&t, copy), vec![Entity::Object(token_like)]);
    // A specified target that isn't legal for the spell: no copy.
    run_with(
        &mut t,
        P0,
        Effect::CopySpellRetargeted {
            what: Sel::Target(0),
            target: Some(Sel::Target(1)),
        },
        &[Entity::Object(spell)],
        &[Entity::Player(P1)],
    );
    assert_eq!(t.stack_len(), 2);
    // Several specified: the copy's controller chooses one.
    let b = t.battlefield(P0, "Hill Giant");
    t.answer_choose(P0, &[Entity::Object(b)]);
    run_with(
        &mut t,
        P0,
        Effect::CopySpellRetargeted {
            what: Sel::Target(0),
            target: Some(Sel::Target(1)),
        },
        &[Entity::Object(spell)],
        &[Entity::Object(token_like), Entity::Object(b)],
    );
    let copy = *t.g.stack.last().unwrap();
    assert_eq!(targets_of(&t, copy), vec![Entity::Object(b)]);
}

#[test]
fn a_copy_of_a_permanent_spell_becomes_a_token() {
    cr!("707.10f");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 2);
    let bears = t.hand(P0, "Grizzly Bears");
    let spell = t.cast(P0, bears).go();
    copy_spell(&mut t, P0, spell, false);
    t.resolve_all();
    let all = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(all.len(), 2);
    let kinds: Vec<ObjKind> = all.iter().map(|id| t.obj(*id).kind).collect();
    assert!(kinds.contains(&ObjKind::Token));
    assert!(kinds.contains(&ObjKind::Card));
}

#[test]
fn a_copy_of_a_double_faced_permanent_spell_is_double_faced() {
    cr!("707.10g");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 1);
    let delver = t.hand(P0, "Delver of Secrets");
    let spell = t.cast(P0, delver).go();
    copy_spell(&mut t, P0, spell, false);
    t.resolve_all();
    let token = t
        .named_on_battlefield("Delver of Secrets")
        .into_iter()
        .find(|id| t.obj(*id).kind == ObjKind::Token)
        .expect("a token");
    // The token has both faces: it can transform.
    assert!(mtg_engine::dfc::transform(&mut t.g, token));
    t.g.recompute();
    assert_eq!(t.obj(token).chars.name.as_str(), "Insectile Aberration");
}

#[test]
fn an_effect_referring_to_a_permanent_by_name_still_tracks_it() {
    cr!("707.11");
    let mut t = TestGame::new(2);
    let olivia = t.custom(
        P0,
        oracle_card(
            "Olive Voldaren",
            "Creature — Vampire",
            "{0}",
            Some((3, 3)),
            "{0}: Put a +1/+1 counter on Olive Voldaren.",
        ),
        Zone::Battlefield,
    );
    t.activate(P0, olivia, 0, &[]).unwrap();
    // Before the ability resolves, it becomes a copy of Hill Giant (a different name).
    let giant = t.battlefield(P1, "Hill Giant");
    run_with(
        &mut t,
        P0,
        Effect::BecomeCopy {
            what: Sel::Target(0),
            of: Sel::Target(1),
            duration: Duration::Permanent,
        },
        &[Entity::Object(olivia)],
        &[Entity::Object(giant)],
    );
    assert_eq!(t.obj(olivia).chars.name.as_str(), "Hill Giant");
    t.resolve_all();
    assert_eq!(t.counters(olivia, counters::PLUS1), 1);
    assert_eq!(t.counters(giant, counters::PLUS1), 0);
}

/// "Copy target instant card in your graveyard[s]. You may cast the copy without paying
/// its mana cost." as an effect controlled by `p`.
fn copy_and_cast(t: &mut TestGame, p: PlayerId, cards: &[ObjectId]) {
    let targets: Vec<Entity> = cards.iter().map(|c| Entity::Object(*c)).collect();
    run_with(
        t,
        p,
        Effect::Seq(vec![
            Effect::CopyCard {
                what: Sel::Target(0),
                named: None,
            },
            Effect::CastCard {
                who: PlayerRef::You,
                what: Sel::Var(vars::CREATED),
                free: true,
                optional: true,
            },
        ]),
        &targets,
        &[],
    );
}

#[test]
fn casting_a_copy_of_a_card() {
    cr!("707.12");
    let mut t = TestGame::new(2);
    let bolt = t.graveyard(P0, "Lightning Bolt");
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    copy_and_cast(&mut t, P0, &[bolt]);
    // The copy was created in the graveyard and cast from there: it's a spell on the
    // stack; the card itself stayed in the graveyard.
    assert_eq!(t.stack_len(), 1);
    let copy = t.g.stack[0];
    assert_eq!(t.obj(copy).kind, ObjKind::CardCopy);
    assert_eq!(t.obj(copy).chars.name.as_str(), "Lightning Bolt");
    assert!(t.in_graveyard(P0, "Lightning Bolt"));
    assert!(t
        .g
        .history
        .spells_cast
        .iter()
        .any(|(p, s)| *p == P0 && *s == copy));
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.graveyard_size(P0), 1);
}

#[test]
fn each_copy_may_be_cast_or_not() {
    cr!("707.12a");
    let mut t = TestGame::new(2);
    let bolt = t.graveyard(P0, "Lightning Bolt");
    let shock = t.graveyard(P0, "Shock");
    // Cast the first copy, not the second.
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_yes(P0, false);
    copy_and_cast(&mut t, P0, &[bolt, shock]);
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    // The copy that wasn't cast ceased to exist.
    let copies =
        t.g.objects
            .iter()
            .filter(|o| o.kind == ObjKind::CardCopy && o.zone != Zone::Nowhere && o.next.is_none())
            .count();
    assert_eq!(copies, 0);
    assert_eq!(t.graveyard_size(P0), 2);
}

#[test]
fn a_copy_of_a_card_defined_by_name_comes_from_the_oracle_reference() {
    cr!("707.13");
    ruling!(
        "Garth One-Eye",
        "You may choose not to cast the copy created, but you won't be able choose that card name again later."
    );
    ruling!(
        "Garth One-Eye",
        "Resolving copies of permanent spells become tokens as they enter the battlefield."
    );
    supported("Garth One-Eye");
    let mut t = TestGame::new(2);
    let garth = t.battlefield(P0, "Garth One-Eye");
    // Choose Black Lotus (the last name) and cast the copy.
    t.answer(P0, DecisionKind::Option, Answer::Index(5));
    t.answer_yes(P0, true);
    t.activate(P0, garth, 0, &[]).unwrap();
    t.resolve_all();
    let lotus = t.named_on_battlefield("Black Lotus");
    assert_eq!(lotus.len(), 1);
    assert_eq!(t.obj(lotus[0]).kind, ObjKind::Token);
    // Next time, Black Lotus can't be chosen again; choose Regrowth and don't cast it.
    t.g.untap(garth);
    t.g.objects[garth.0 as usize].activations_this_turn.clear();
    t.answer(P0, DecisionKind::Option, Answer::Index(4));
    t.answer_yes(P0, false);
    t.activate(P0, garth, 0, &[]).unwrap();
    t.resolve_all();
    let options: Vec<String> = t
        .asked()
        .into_iter()
        .rev()
        .find_map(|(_, d)| match d {
            Decision::ChooseOption {
                prompt, options, ..
            } if prompt == "Choose a card name" => Some(options),
            _ => None,
        })
        .unwrap();
    assert_eq!(options.len(), 5);
    assert!(!options.iter().any(|o| o == "black lotus"));
    // The uncast copy ceased to exist.
    assert!(t
        .g
        .objects
        .iter()
        .all(|o| !(o.kind == ObjKind::CardCopy && o.next.is_none() && o.zone != Zone::Nowhere)));
}

#[test]
fn a_copy_of_a_noted_card_uses_its_last_known_information() {
    cr!("707.14");
    let mut t = TestGame::new(2);
    const NOTED: Var = vars::USER + 5;
    let bolt = t.graveyard(P0, "Lightning Bolt");
    // "Note the name of target instant card in your graveyard and put it onto the
    // battlefield face down. ... create a copy of the card with the noted name. You may
    // cast the copy without paying its mana cost."
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    run_with(
        &mut t,
        P0,
        Effect::Seq(vec![
            Effect::Store {
                var: NOTED,
                sel: Sel::Target(0),
            },
            Effect::Move {
                what: Sel::Target(0),
                to: Destination {
                    face_down: true,
                    ..Destination::battlefield()
                },
            },
            Effect::CopyCard {
                what: Sel::Var(NOTED),
                named: Some(NamedCopy::LastKnown),
            },
            Effect::CastCard {
                who: PlayerRef::You,
                what: Sel::Var(vars::CREATED),
                free: true,
                optional: true,
            },
        ]),
        &[Entity::Object(bolt)],
        &[],
    );
    // The card is now a face-down 2/2 on the battlefield, but the copy has the
    // characteristics the card had in the graveyard.
    let fd = t.g.current(bolt);
    assert!(t.obj(fd).face_down && t.on_battlefield(fd));
    assert_eq!(t.stack_len(), 1);
    let copy = t.g.stack[0];
    assert_eq!(t.obj(copy).chars.name.as_str(), "Lightning Bolt");
    assert!(t.obj(copy).chars.is(CardType::Instant));
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}
