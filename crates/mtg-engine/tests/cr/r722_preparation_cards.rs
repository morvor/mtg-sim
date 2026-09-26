//! CR 722: preparation cards.

use super::r709_common::*;
use mtg_engine::ability::*;
use mtg_engine::card::{card, Layout};
use mtg_engine::designations;
use mtg_engine::eval::Ctx;
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Goblin Glasswright ({1}{R} 2/2 Goblin Sorcerer, "This creature enters prepared.") //
/// Craft with Pride ({R} sorcery, "Create a Treasure token.").
const GLASSWRIGHT: &str = "Goblin Glasswright // Craft with Pride";

fn treasures(t: &TestGame) -> usize {
    t.g.battlefield
        .iter()
        .filter(|id| t.obj(**id).chars.has_subtype("Treasure"))
        .count()
}

#[test]
fn a_preparation_card_has_an_inset_prepare_spell() {
    cr!("722.1", "722.2");
    supported(GLASSWRIGHT);
    let def = card(GLASSWRIGHT);
    assert_eq!(def.layout, Layout::Prepare);
    assert_eq!(def.faces.len(), 2);
    let spell = &def.faces[1].chars;
    assert_eq!(spell.name, "Craft with Pride");
    assert!(spell.is(CardType::Sorcery));
    // Its normal characteristics appear as usual.
    let mut t = TestGame::new(2);
    let g = t.hand(P0, GLASSWRIGHT);
    t.g.recompute();
    assert_eq!(t.obj(g).chars.name, "Goblin Glasswright");
    assert!(t.obj(g).chars.is_creature());
}

#[test]
fn only_an_object_with_a_prepare_spell_can_become_prepared() {
    cr!("722.2a");
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, GLASSWRIGHT);
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert!(designations::has_prepare_spell(&t.g, g));
    assert!(!designations::has_prepare_spell(&t.g, bears));
    designations::become_prepared(&mut t.g, g);
    designations::become_prepared(&mut t.g, bears);
    assert!(t.obj(g).prepared.is_some());
    assert!(t.obj(bears).prepared.is_none());
}

#[test]
fn the_prepare_spell_is_part_of_the_copiable_values() {
    cr!("722.2b", "722.3c");
    ruling!(
        "Goblin Glasswright // Craft with Pride",
        "the copy of Craft with Pride in exile will still be a red sorcery"
    );
    ruling!(
        "Encouraging Aviator // Jump",
        "Being prepared isn't a copiable value"
    );
    supported("Croaking Counterpart");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let g = t.enter(P0, GLASSWRIGHT);
    let g = t.g.current(g);
    assert!(t.obj(g).prepared.is_some());
    // Croaking Counterpart: "Create a token that's a copy of target non-Frog creature,
    // except it's a 1/1 green Frog." The token enters prepared (its copiable values
    // include the prepare spell) ...
    t.lands(P0, "Forest", 2);
    t.lands(P0, "Island", 1);
    let cc = t.hand(P0, "Croaking Counterpart");
    t.cast(P0, cc).target(g).go();
    t.resolve_all();
    let token = t
        .g
        .battlefield
        .iter()
        .copied()
        .find(|id| t.obj(*id).kind == ObjKind::Token)
        .expect("token");
    assert_eq!(t.obj(token).chars.name, "Goblin Glasswright");
    assert_eq!(t.pt(token), (1, 1));
    assert!(designations::has_prepare_spell(&t.g, token));
    let copy = t.obj(token).prepared.expect("the token entered prepared");
    // ... and the copy of Craft with Pride in exile ignores the copy exception: a red
    // sorcery, not a green 1/1 Frog.
    let c = &t.obj(copy).chars;
    assert_eq!(c.name, "Craft with Pride");
    assert!(c.is(CardType::Sorcery) && !c.is_creature());
    assert_eq!(c.colors, ColorSet::single(Color::Red));
    assert!(!c.has_subtype("Frog"));
    // A permanent that becomes a copy of a prepared creature isn't prepared, but has the
    // prepare spell.
    let bears = t.battlefield(P0, "Grizzly Bears");
    run_effect(
        &mut t,
        P0,
        None,
        &[Entity::Object(bears)],
        Effect::BecomeCopy {
            what: Sel::Target(0),
            of: Sel::All(Filter::Objects(vec![g])),
            duration: Duration::Permanent,
        },
    );
    assert!(t.obj(bears).prepared.is_none());
    assert!(designations::has_prepare_spell(&t.g, bears));
    designations::become_prepared(&mut t.g, bears);
    assert!(t.obj(bears).prepared.is_some());
}

#[test]
fn a_preparation_card_is_one_card() {
    cr!("722.2c");
    let mut t = TestGame::new(2);
    t.library_top(P0, GLASSWRIGHT);
    run_effect(
        &mut t,
        P0,
        None,
        &[],
        Effect::Draw {
            who: PlayerRef::You,
            n: Value::c(1),
        },
    );
    assert_eq!(t.hand_size(P0), 1);
    let ctx = Ctx::new(None, P0);
    assert_eq!(t.g.eval_value(&Value::CardsDrawnThisTurn(PlayerRef::You), &ctx), 1);
}

#[test]
fn a_copy_of_a_prepare_spell_is_a_prepare_spell() {
    cr!("722.3d");
    supported("Codie, Ravenous Codex");
    // Codie, Ravenous Codex: "Whenever you cast a prepared spell, copy it."
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Codie, Ravenous Codex");
    let g = t.enter(P0, GLASSWRIGHT);
    let copy = t.obj_now(g).prepared.expect("prepared");
    t.lands(P0, "Mountain", 1);
    let spell = t.cast(P0, copy).go();
    let ctx = Ctx::new(None, P0);
    assert!(t.g.matches(spell, &Filter::Prepared, &ctx));
    t.settle();
    // Codie's trigger copies it; the copy is a prepare spell too.
    t.resolve();
    assert_eq!(t.stack_len(), 2);
    let spell_copy = t.g.stack[1];
    assert_eq!(t.obj(spell_copy).kind, ObjKind::SpellCopy);
    assert!(t.g.matches(spell_copy, &Filter::Prepared, &ctx));
    t.resolve_all();
    assert_eq!(treasures(&t), 2);
    // A spell that wasn't cast as a prepare spell isn't one.
    let mut t2 = TestGame::new(2);
    t2.set_step(P0, Step::PrecombatMain);
    t2.lands(P0, "Mountain", 1);
    let bolt = t2.hand(P0, "Lightning Bolt");
    let s = t2.cast(P0, bolt).target(P1).go();
    assert!(!t2.g.matches(s, &Filter::Prepared, &Ctx::new(None, P0)));
}

#[test]
fn a_preparation_card_has_only_its_normal_characteristics() {
    cr!("722.4");
    ruling!(
        "Adventurous Eater // Have a Bite",
        "A preparation card is a creature card in every zone"
    );
    let mut t = TestGame::new(2);
    let gy = t.graveyard(P0, GLASSWRIGHT);
    let ex = t.exile(P0, GLASSWRIGHT);
    t.g.recompute();
    for id in [gy, ex] {
        let c = &t.obj(id).chars;
        assert!(c.is_creature() && !c.is(CardType::Sorcery));
        assert_eq!(c.name, "Goblin Glasswright");
    }
    assert_eq!(mv(&mut t, gy), 2);
    // Prepared on the battlefield, too.
    let g = t.enter(P0, GLASSWRIGHT);
    let g = t.g.current(g);
    assert!(t.obj(g).prepared.is_some());
    assert_eq!(t.obj(g).chars.name, "Goblin Glasswright");
    assert!(t.obj(g).chars.is_creature());
}
