//! Rulings batch S02 — casualty (CR 702.153): "Casualty N" means "As an additional cost to
//! cast this spell, you may sacrifice a creature with power N or greater," and "When you
//! cast this spell, if a casualty cost was paid for it, copy it. If the spell has any
//! targets, you may choose new targets for the copy."

use crate::r_s01_common::*;
use crate::r_s02_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::ObjKind;
use mtg_engine::testing::*;
use mtg_engine::*;

/// The spells on the stack, bottom first, as (name, is a copy).
fn spells(t: &TestGame) -> Vec<(String, bool)> {
    t.g.stack
        .iter()
        .filter(|s| t.g.obj(**s).is_spell())
        .map(|s| {
            let o = t.g.obj(*s);
            (o.chars.name.to_string(), o.kind == ObjKind::SpellCopy)
        })
        .collect()
}

/// The optional additional costs P0 was offered since decision `from`.
fn offered_costs(t: &TestGame, from: usize) -> Vec<String> {
    t.asked()[from..]
        .iter()
        .filter_map(|(p, d)| match d {
            Decision::OptionalCost { name, .. } if *p == P0 => Some(name.clone()),
            _ => None,
        })
        .collect()
}

/// P0 casts the real card `name` from hand (with the mana for it), paying its casualty
/// cost by sacrificing `sacrifice` if given.
fn cast_with_casualty(
    t: &mut TestGame,
    name: &str,
    sacrifice: Option<ObjectId>,
    targets: &[Entity],
) -> ObjectId {
    supported(name);
    give_mana_for(t, P0, name);
    let c = t.hand(P0, name);
    t.answer(
        P0,
        DecisionKind::OptionalCost,
        Answer::Bool(sacrifice.is_some()),
    );
    if let Some(s) = sacrifice {
        t.answer_choose(P0, &[Entity::Object(s)]);
    }
    let mut b = t.cast(P0, c);
    for e in targets {
        b = b.target(*e);
    }
    let id = b.go();
    t.settle();
    id
}

#[test]
fn the_copy_resolves_before_the_original_spell() {
    cr!("702.153a", "707.10", "707.10c", "405.5", "608.1");
    ruling!(
        "Light 'Em Up",
        "If you pay the casualty cost of a spell, the copy will resolve before the original spell."
    );
    // "Casualty 2. Light 'Em Up deals 2 damage to target creature or planeswalker."
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let first = t.battlefield(P1, "Grizzly Bears");
    let second = t.battlefield(P1, "Llanowar Elves");
    cast_with_casualty(
        &mut t,
        "Light 'Em Up",
        Some(giant),
        &[Entity::Object(first)],
    );
    assert!(t.in_graveyard(P0, "Hill Giant"));
    assert_eq!(triggers_on_stack(&t, "Casualty"), 1);
    // The casualty ability resolves: the copy goes on the stack above the original, with
    // a new target.
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(second)]);
    t.resolve();
    assert_eq!(
        spells(&t),
        vec![
            ("Light 'Em Up".to_string(), false),
            ("Light 'Em Up".to_string(), true)
        ]
    );
    // The copy resolves first: its target is dealt damage while the original waits.
    t.resolve();
    assert!(t.in_graveyard(P1, "Llanowar Elves"));
    assert!(t.on_battlefield(first));
    assert_eq!(spells(&t), vec![("Light 'Em Up".to_string(), false)]);
    t.resolve();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn only_one_creature_is_sacrificed_and_the_spell_is_copied_once() {
    cr!("702.153a", "601.2b", "601.2h", "118.8a");
    ruling!(
        "Join the Maestros",
        "You may sacrifice only one creature to pay a spell's casualty cost, and you copy the spell only once."
    );
    // "Casualty 2. Create a 4/3 black Ogre Warrior creature token."
    let mut t = TestGame::new(2);
    let giants = [
        t.battlefield(P0, "Hill Giant"),
        t.battlefield(P0, "Hill Giant"),
    ];
    let from = t.asked().len();
    cast_with_casualty(&mut t, "Join the Maestros", Some(giants[0]), &[]);
    // One casualty cost, paid by sacrificing one creature.
    assert_eq!(offered_costs(&t, from), vec!["casualty#1".to_string()]);
    let sacrifice: Vec<(u32, u32)> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities { min, max, .. } => Some((*min, *max)),
            _ => None,
        })
        .collect();
    assert_eq!(sacrifice, vec![(1, 1)]);
    assert!(!t.on_battlefield(giants[0]));
    assert!(t.on_battlefield(giants[1]));
    assert_eq!(triggers_on_stack(&t, "Casualty"), 1);
    t.resolve_all();
    // The spell and one copy: two Ogres.
    let ogres = with_subtype(&t, P0, "Ogre");
    assert_eq!(ogres.len(), 2);
    assert!(ogres.iter().all(|o| t.pt(*o) == (4, 3)));
}

#[test]
fn casualty_is_an_optional_cost_sacrificing_a_creature_with_enough_power() {
    cr!("702.153a", "118.8b", "601.2b", "603.4");
    ruling!(
        "Join the Maestros",
        "Casualty N means \"As an additional cost to cast this spell, you may sacrifice a creature with power N or greater.\" and \"When you cast this spell, if a casualty cost was paid for it, copy it. If the spell has any targets, you may choose new targets for the copy.\""
    );
    // A creature with power less than 2 can't be sacrificed: the cost isn't offered, and
    // the spell isn't copied.
    let mut t = TestGame::new(2);
    let elves = t.battlefield(P0, "Llanowar Elves");
    let from = t.asked().len();
    cast_with_casualty(&mut t, "Join the Maestros", None, &[]);
    assert!(offered_costs(&t, from).is_empty());
    assert_eq!(triggers_on_stack(&t, "Casualty"), 0);
    t.resolve_all();
    assert!(t.on_battlefield(elves));
    assert_eq!(with_subtype(&t, P0, "Ogre").len(), 1);

    // With a creature with power 2, paying is optional: declined, no copy.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let from = t.asked().len();
    cast_with_casualty(&mut t, "Join the Maestros", None, &[]);
    assert_eq!(offered_costs(&t, from), vec!["casualty#1".to_string()]);
    assert_eq!(triggers_on_stack(&t, "Casualty"), 0);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert_eq!(with_subtype(&t, P0, "Ogre").len(), 1);

    // Paid: the Bears are sacrificed and the spell is copied.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_with_casualty(&mut t, "Join the Maestros", Some(bears), &[]);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Ogre").len(), 2);

    // A copy of a spell with targets may keep them: two copies of Light 'Em Up's damage
    // to the same creature.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let wurm = t.battlefield(P1, "Craw Wurm");
    cast_with_casualty(
        &mut t,
        "Light 'Em Up",
        Some(bears),
        &[Entity::Object(wurm)],
    );
    t.answer_yes(P0, false);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Craw Wurm"));
}

#[test]
fn spells_given_casualty_by_silverquill_can_be_copied() {
    cr!("702.153a", "610.5", "601.2b", "113.7a");
    ruling!(
        "Silverquill, the Disputant",
        "Casualty N means \"As an additional cost to cast this spell, you may sacrifice a creature with power N or greater.\" and \"When you cast this spell, if a casualty cost was paid for it, copy it."
    );
    supported("Silverquill, the Disputant");
    // "Each instant and sorcery spell you cast has casualty 1."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Silverquill, the Disputant");
    let elves = t.battlefield(P0, "Llanowar Elves");
    let from = t.asked().len();
    cast_with_casualty(
        &mut t,
        "Shock",
        Some(elves),
        &[Entity::Player(P1)],
    );
    assert_eq!(offered_costs(&t, from), vec!["casualty#1".to_string()]);
    assert!(t.in_graveyard(P0, "Llanowar Elves"));
    t.answer_yes(P0, false);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    // A creature spell doesn't have casualty.
    let from = t.asked().len();
    t.battlefield(P0, "Grizzly Bears");
    give_mana_for(&mut t, P0, "Hill Giant");
    let giant = t.hand(P0, "Hill Giant");
    t.cast(P0, giant).go();
    assert!(offered_costs(&t, from).is_empty());
    // Without Silverquill, an instant has no casualty.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Llanowar Elves");
    let from = t.asked().len();
    cast_with_casualty(&mut t, "Shock", None, &[Entity::Player(P1)]);
    assert!(offered_costs(&t, from).is_empty());
    // The spell gained casualty as it was cast: Silverquill leaving the battlefield in
    // response to the casualty ability doesn't stop the copy.
    let mut t = TestGame::new(2);
    let silverquill = t.battlefield(P0, "Silverquill, the Disputant");
    let elves = t.battlefield(P0, "Llanowar Elves");
    cast_with_casualty(&mut t, "Shock", Some(elves), &[Entity::Player(P1)]);
    assert_eq!(triggers_on_stack(&t, "Casualty"), 1);
    destroy(&mut t, silverquill);
    assert!(t.in_graveyard(P0, "Silverquill, the Disputant"));
    t.answer_yes(P0, false);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
}

#[test]
fn each_instance_of_casualty_is_paid_separately_and_copies_separately() {
    cr!("702.153b");
    // Light 'Em Up has casualty 2, and Silverquill gives it casualty 1. It deals 2 damage
    // to a 6/6: once for the spell and once for each copy.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Silverquill, the Disputant");
    let giant = t.battlefield(P0, "Hill Giant");
    let elves = t.battlefield(P0, "Llanowar Elves");
    let dreadmaw = t.battlefield(P1, "Colossal Dreadmaw");
    supported("Light 'Em Up");
    give_mana_for(&mut t, P0, "Light 'Em Up");
    let c = t.hand(P0, "Light 'Em Up");
    let from = t.asked().len();
    // Pay both: the Giant for casualty 2, the Elves for casualty 1.
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.answer_choose(P0, &[Entity::Object(giant)]);
    t.answer_choose(P0, &[Entity::Object(elves)]);
    t.cast(P0, c).target(dreadmaw).go();
    t.settle();
    assert_eq!(
        offered_costs(&t, from),
        vec!["casualty#1".to_string(), "casualty#2".to_string()]
    );
    assert!(t.in_graveyard(P0, "Hill Giant"));
    assert!(t.in_graveyard(P0, "Llanowar Elves"));
    // Each instance triggers based on its own payment.
    assert_eq!(triggers_on_stack(&t, "Casualty"), 2);
    t.answer_yes(P0, false);
    t.answer_yes(P0, false);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Colossal Dreadmaw"));

    // Paying only one of them copies the spell once.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Silverquill, the Disputant");
    let giant = t.battlefield(P0, "Hill Giant");
    let elves = t.battlefield(P0, "Llanowar Elves");
    let dreadmaw = t.battlefield(P1, "Colossal Dreadmaw");
    give_mana_for(&mut t, P0, "Light 'Em Up");
    let c = t.hand(P0, "Light 'Em Up");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(false));
    t.answer_choose(P0, &[Entity::Object(giant)]);
    t.cast(P0, c).target(dreadmaw).go();
    t.settle();
    assert!(t.on_battlefield(elves));
    assert_eq!(triggers_on_stack(&t, "Casualty"), 1);
    t.answer_yes(P0, false);
    t.resolve_all();
    assert!(t.on_battlefield(dreadmaw));
    assert_eq!(t.obj_now(dreadmaw).damage, 4);

    // Paying only the granted one (casualty 1, with the Elves) copies it once too.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Silverquill, the Disputant");
    let giant = t.battlefield(P0, "Hill Giant");
    let elves = t.battlefield(P0, "Llanowar Elves");
    let dreadmaw = t.battlefield(P1, "Colossal Dreadmaw");
    give_mana_for(&mut t, P0, "Light 'Em Up");
    let c = t.hand(P0, "Light 'Em Up");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(false));
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.answer_choose(P0, &[Entity::Object(elves)]);
    t.cast(P0, c).target(dreadmaw).go();
    t.settle();
    assert!(t.on_battlefield(giant));
    assert!(t.in_graveyard(P0, "Llanowar Elves"));
    assert_eq!(triggers_on_stack(&t, "Casualty"), 1);
    t.answer_yes(P0, false);
    t.resolve_all();
    assert_eq!(t.obj_now(dreadmaw).damage, 4);
}

#[test]
fn the_copy_is_not_cast() {
    cr!("707.10", "603.2", "702.108a");
    ruling!(
        "Rob the Archives",
        "The copy of the spell is created on the stack, so it's not “cast.” Abilities that trigger when a player casts a spell won't trigger."
    );
    // Young Pyromancer: "Whenever you cast an instant or sorcery spell, create a 1/1 red
    // Elemental creature token." Monastery Swiftspear has prowess.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Young Pyromancer");
    let swiftspear = t.battlefield(P0, "Monastery Swiftspear");
    let giant = t.battlefield(P0, "Hill Giant");
    t.library_top(P0, "Forest");
    t.library_top(P0, "Island");
    t.library_top(P0, "Swamp");
    t.library_top(P0, "Plains");
    cast_with_casualty(&mut t, "Rob the Archives", Some(giant), &[]);
    t.resolve_all();
    // Rob the Archives and its copy each exiled two cards.
    assert_eq!(t.g.exile.len(), 4);
    // Only the spell that was cast triggered them.
    assert_eq!(with_subtype(&t, P0, "Elemental").len(), 1);
    assert_eq!(t.pt(swiftspear), (2, 3));
}

#[test]
fn a_casualty_trigger_copies_the_spell_even_if_the_spell_lost_casualty() {
    cr!("702.153a", "113.7a", "603.4");
    // Light 'Em Up is cast with its casualty cost paid. In response to its casualty
    // ability, the spell loses all abilities: the ability exists independently of the
    // spell, and the cost was paid, so it still copies the spell.
    use mtg_engine::ability::{Duration, Effect, Modification, Sel};
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let dreadmaw = t.battlefield(P1, "Colossal Dreadmaw");
    let spell = cast_with_casualty(
        &mut t,
        "Light 'Em Up",
        Some(giant),
        &[Entity::Object(dreadmaw)],
    );
    assert_eq!(triggers_on_stack(&t, "Casualty"), 1);
    let mut ctx = mtg_engine::eval::Ctx::new(None, P1);
    ctx.targets = vec![vec![Entity::Object(spell)]];
    t.g.exec(
        &Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::RemoveAllAbilities],
            duration: Duration::EndOfTurn,
        },
        &mut ctx,
    );
    t.settle();
    assert!(!t
        .obj(spell)
        .chars
        .keywords()
        .any(|k| k.kind == mtg_engine::keywords::KeywordKind::Casualty));
    t.answer_yes(P0, false);
    t.resolve();
    assert_eq!(
        spells(&t),
        vec![
            ("Light 'Em Up".to_string(), false),
            ("Light 'Em Up".to_string(), true)
        ]
    );
}
