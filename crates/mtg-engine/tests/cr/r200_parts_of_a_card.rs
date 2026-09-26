//! CR 200: the parts of a card, which of them are characteristics, and the parts objects
//! that aren't cards have.

use crate::r105_util::{colors, cs, matches};
use crate::r703_common::run_effect;
use mtg_engine::ability::*;
use mtg_engine::card::card;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{Characteristics, ObjKind, Zone};
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn the_parts_of_a_card() {
    // CR 200.1: the game-relevant parts of a card come from its Oracle data: name, mana
    // cost, color indicator, type line, text box, power and toughness, loyalty, defense,
    // hand and life modifiers. Some cards have more than one of them.
    cr!("200.1");
    let angel = card("Serra Angel");
    let c = &angel.front().chars;
    assert_eq!(c.name, "Serra Angel");
    assert_eq!(c.mana_cost.as_ref().unwrap().to_string(), "{3}{W}{W}");
    assert!(c.is(CardType::Creature) && c.has_subtype("Angel"));
    assert!(c.rules_text.contains("Flying"));
    assert_eq!((c.power, c.toughness), (Some(4), Some(4)));
    assert_eq!(card("Jace Beleren").front().chars.loyalty, Some(3));
    assert_eq!(card("Invasion of Segovia").front().chars.defense, Some(4));
    let titania = card("Titania");
    assert_eq!(titania.front().chars.hand_modifier, Some(2));
    assert_eq!(titania.front().chars.life_modifier, Some(-5));
    assert_eq!(card("Dryad Arbor").front().chars.color_indicator, Some(cs("G")));
    // More than one of a part: a split card has two names, mana costs, type lines and
    // text boxes; a double-faced card has two faces.
    let fi = card("Fire // Ice");
    assert_eq!(fi.faces.len(), 2);
    let names: Vec<&str> = fi.faces.iter().map(|f| f.chars.name.as_str()).collect();
    assert_eq!(names, vec!["Fire", "Ice"]);
    assert!(fi.faces.iter().all(|f| f.chars.mana_cost.is_some()));
    assert_ne!(fi.faces[0].chars.rules_text, fi.faces[1].chars.rules_text);
    let delver = card("Delver of Secrets");
    assert_eq!(delver.faces[1].chars.name, "Insectile Aberration");
    assert_eq!(delver.faces[1].chars.color_indicator, Some(cs("U")));
    assert_eq!(
        (delver.faces[1].chars.power, delver.faces[1].chars.toughness),
        (Some(3), Some(2))
    );
}

#[test]
fn some_parts_of_a_card_are_characteristics() {
    // CR 200.2 (with 109.3): the name, mana cost, color indicator, type line, rules text,
    // power and toughness, loyalty, ... are characteristics — what effects look at and
    // change, and what copy effects copy. The illustration, expansion symbol and other
    // printed information aren't.
    cr!("200.2");
    let mut t = TestGame::new(2);
    let angel = t.battlefield(P0, "Serra Angel");
    let named = Filter::Named("Serra Angel".into());
    assert!(matches(&t, angel, &named, P0));
    assert!(matches(
        &t,
        angel,
        &Filter::ManaValue(Cmp::Eq, Box::new(Value::c(5))),
        P0
    ));
    assert!(matches(
        &t,
        angel,
        &Filter::Power(Cmp::Eq, Box::new(Value::c(4))),
        P0
    ));
    // A copy of Serra Angel has all of those characteristics.
    let bears = t.battlefield(P0, "Grizzly Bears");
    let mut ctx = mtg_engine::eval::Ctx::new(None, P0);
    ctx.targets = vec![vec![Entity::Object(bears)], vec![Entity::Object(angel)]];
    t.g.exec(
        &Effect::BecomeCopy {
            what: Sel::Target(0),
            of: Sel::Target(1),
            duration: Duration::Permanent,
        },
        &mut ctx,
    );
    t.g.recompute();
    let c: &Characteristics = &t.obj_now(bears).chars;
    assert_eq!(c.name, "Serra Angel");
    assert_eq!(c.mana_cost.as_ref().unwrap().to_string(), "{3}{W}{W}");
    assert_eq!(colors(&t, bears), cs("W"));
    assert!(t.obj_now(bears).has_keyword(KeywordKind::Flying));
    assert_eq!(t.pt(bears), (4, 4));
    // Effects change characteristics: its name can be changed.
    run_effect(
        &mut t,
        P0,
        None,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::SetName("Renamed Angel".into())],
            duration: Duration::EndOfTurn,
        },
        &[Entity::Object(angel)],
    );
    assert!(!matches(&t, angel, &named, P0));
    // Printed information that isn't a characteristic, like an alternate name
    // (CR 201.6), isn't part of the object at all.
    let kibo = t.battlefield(P0, "Kibo, Uktabi Prince");
    assert_eq!(t.obj_now(kibo).chars.name, "Kibo, Uktabi Prince");
    assert!(!t.obj_now(kibo).chars.has_name("Monkey, Awakened to Emptiness"));
}

#[test]
fn tokens_and_copies_have_only_the_parts_that_are_characteristics() {
    cr!("200.3");
    // A token: characteristics given by the effect that created it, and no mana cost.
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 3);
    let sp = t.hand(P0, "Spectral Procession");
    t.cast(P0, sp).go();
    t.resolve();
    let spirit = t.named_on_battlefield("Spirit Token")[0];
    let o = t.obj_now(spirit);
    assert_eq!(o.kind, ObjKind::Token);
    assert!(o.card.is_none() || o.is_token());
    assert!(o.chars.mana_cost.is_none());
    assert_eq!(o.chars.colors, cs("W"));
    assert!(o.chars.is(CardType::Creature) && o.chars.has_subtype("Spirit"));
    assert_eq!(t.pt(spirit), (1, 1));
    assert!(o.has_keyword(KeywordKind::Flying));
    // A copy of a spell: the copied characteristics (name, mana cost, color, type, text).
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    let spell = t.cast(P0, bolt).target(P1).go();
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
    let copy = *t.g.stack.last().unwrap();
    let o = t.obj_now(copy);
    assert_eq!(o.kind, ObjKind::SpellCopy);
    assert_eq!(o.chars.name, "Lightning Bolt");
    assert_eq!(o.chars.mana_cost.as_ref().unwrap().to_string(), "{R}");
    assert!(o.chars.is(CardType::Instant));
    t.resolve_all();
    assert_eq!(t.life(P1), 14);
    // A copy of a card: its copiable characteristics.
    let mut t = TestGame::new(2);
    let exiled = t.exile(P0, "Serra Angel");
    run_effect(
        &mut t,
        P0,
        None,
        Effect::CopyCard {
            what: Sel::Target(0),
            named: None,
        },
        &[Entity::Object(exiled)],
    );
    let copy = t
        .g
        .exile
        .iter()
        .copied()
        .find(|i| *i != exiled)
        .expect("a copy of the card");
    let o = t.obj_now(copy);
    assert_eq!(o.kind, ObjKind::CardCopy);
    assert_eq!(o.zone, Zone::Exile);
    assert_eq!(o.chars.name, "Serra Angel");
    assert_eq!(o.chars.mana_cost.as_ref().unwrap().to_string(), "{3}{W}{W}");
    assert_eq!((o.chars.power, o.chars.toughness), (Some(4), Some(4)));
}
