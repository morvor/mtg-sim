//! CR 112: spells — what a spell is, copies of spells and cards, owners and controllers
//! of spells, and characteristics of spells.

use super::r105_util::*;
use mtg_engine::ability::*;
use mtg_engine::object::{ObjKind, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// "Copy target spell." (no new targets).
fn copier() -> mtg_engine::card::CardDef {
    card_with(
        "Copy Test",
        "{0}",
        "Instant",
        None,
        vec![spell_ab(
            vec![TargetSpec {
                what: TargetKind::Spell(Filter::Any),
                ..TargetSpec::object(Filter::Any, "target spell")
            }],
            Effect::CopySpell {
                what: Sel::Target(0),
                count: Value::c(1),
                new_targets: false,
            },
        )],
    )
}

#[test]
fn a_spell_is_a_card_on_the_stack() {
    cr!("112.1");
    let mut t = TestGame::new(2);
    let bears = t.hand(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 2);
    let spell = t.cast(P0, bears).go();
    // The card moved from its owner's hand to the top of the stack.
    assert!(!t.g.player(P0).hand.contains(&bears));
    assert_eq!(*t.g.stack.last().unwrap(), spell);
    assert_eq!(t.obj(spell).kind, ObjKind::Card);
    assert_eq!(t.obj(spell).zone, Zone::Stack);
    assert!(matches(&t, spell, &Filter::Spell, P0));
    // It remains a spell until it resolves (or is countered).
    t.resolve();
    assert!(!matches(&t, spell, &Filter::Spell, P0));
    assert!(t.on_battlefield(spell));
    // A countered spell leaves the stack.
    let giant = t.hand(P0, "Hill Giant");
    t.lands(P0, "Mountain", 4);
    let s2 = t.cast(P0, giant).go();
    let cs = t.hand(P1, "Counterspell");
    t.lands(P1, "Island", 2);
    t.g.turn.priority = Some(P1);
    t.cast(P1, cs).target(s2).go();
    t.resolve();
    assert_eq!(t.stack_len(), 0);
    assert!(t.in_graveyard(P0, "Hill Giant"));
}

#[test]
fn a_copy_of_a_spell_is_a_spell() {
    cr!("112.1a");
    let mut t = TestGame::new(2);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.lands(P0, "Mountain", 1);
    let spell = t.cast(P0, bolt).target(P1).go();
    let c = put_in_hand(&mut t, P0, copier());
    t.cast(P0, c).target(spell).go();
    t.resolve();
    let copy = *t.g.stack.last().unwrap();
    assert_eq!(t.obj(copy).kind, ObjKind::SpellCopy);
    // It isn't a card, but it's a spell.
    assert!(!matches(&t, copy, &Filter::Card, P0));
    assert!(matches(&t, copy, &Filter::Spell, P0));
    // It can be countered by Counterspell like any spell.
    let cs = t.hand(P1, "Counterspell");
    t.lands(P1, "Island", 2);
    assert!(spell_target_candidates(&t, P1, cs, 0).contains(&Entity::Object(copy)));
    t.g.turn.priority = Some(P1);
    t.cast(P1, cs).target(copy).go();
    t.resolve();
    t.resolve_all();
    // Only the original Bolt dealt damage.
    assert_eq!(t.life(P1), 17);
}

#[test]
fn a_cast_copy_of_a_card_is_a_spell() {
    cr!("112.1b");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 4);
    let eater = t.hand(P0, "Adventurous Eater // Have a Bite");
    t.cast(P0, eater).go();
    t.resolve();
    let copy = t.obj_now(eater).prepared.expect("prepared");
    assert_eq!(t.obj(copy).kind, ObjKind::CardCopy);
    assert!(!matches(&t, copy, &Filter::Spell, P0));
    let now = t.g.current(eater);
    let spell = t.cast(P0, copy).target(now).go();
    assert_eq!(t.obj(spell).zone, Zone::Stack);
    assert!(matches(&t, spell, &Filter::Spell, P0));
    // It's a spell that can be countered.
    let cs = t.hand(P1, "Counterspell");
    t.lands(P1, "Island", 2);
    assert!(spell_target_candidates(&t, P1, cs, 0).contains(&Entity::Object(spell)));
}

#[test]
fn a_spells_owner_is_its_cards_owner_and_its_controller_who_cast_it() {
    cr!("112.2");
    ruling!(
        "Aethersnatch",
        "If you gain control of an instant or sorcery spell, it will be put into its owner’s graveyard as it resolves."
    );
    let mut t = TestGame::new(2);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.lands(P1, "Mountain", 1);
    t.set_step(P1, Step::PrecombatMain);
    let spell = t.cast(P1, bolt).target(P0).go();
    assert_eq!(t.obj(spell).owner, P1);
    assert_eq!(t.obj(spell).controller, P1);
    // P0 gains control of it: P0 controls it, P1 still owns it.
    let thief = put_in_hand(
        &mut t,
        P0,
        card_from_text(
            "Spell Thief",
            "{0}",
            "Instant",
            None,
            "Gain control of target spell.",
        ),
    );
    t.g.turn.priority = Some(P0);
    t.cast(P0, thief).target(spell).go();
    t.resolve();
    assert_eq!(t.obj_now(spell).controller, P0);
    assert_eq!(t.obj_now(spell).owner, P1);
    t.resolve();
    // It goes to its owner's graveyard.
    assert!(t.in_graveyard(P1, "Lightning Bolt"));
    assert!(!t.in_graveyard(P0, "Lightning Bolt"));
}

#[test]
fn a_copy_of_a_spell_is_owned_by_the_player_who_put_it_on_the_stack() {
    cr!("112.2");
    let mut t = TestGame::new(2);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.lands(P0, "Mountain", 1);
    let spell = t.cast(P0, bolt).target(P1).go();
    // P1 copies P0's spell: the copy is P1's.
    let c = put_in_hand(&mut t, P1, copier());
    t.g.turn.priority = Some(P1);
    t.cast(P1, c).target(spell).go();
    t.resolve();
    let copy = *t.g.stack.last().unwrap();
    assert_ne!(copy, spell);
    assert_eq!(t.obj(copy).owner, P1);
    assert_eq!(t.obj(copy).controller, P1);
    assert_eq!(t.obj(spell).owner, P0);
}

#[test]
fn a_copy_of_a_card_created_to_be_cast_is_owned_by_the_player_who_created_it() {
    cr!("112.2a");
    let mut t = TestGame::new(2);
    // P0 controls P1's Eiganjo Dynastorian when it becomes prepared: P0 creates the
    // copy of its prepare spell and may cast it, so P0 owns that copy.
    let dyn_ = t.battlefield(P1, "Eiganjo Dynastorian // Replenish");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let treason = t.hand(P0, "Act of Treason");
    t.lands(P0, "Mountain", 3);
    t.cast(P0, treason).target(dyn_).go();
    t.resolve();
    assert_eq!(t.obj_now(dyn_).controller, P0);
    // "Whenever you attack with two or more creatures, this creature becomes prepared."
    t.attack(
        &[
            (t.g.current(dyn_), Entity::Player(P1)),
            (bears, Entity::Player(P1)),
        ],
        &[],
    );
    t.resolve_all();
    let copy = t.obj_now(dyn_).prepared.expect("prepared");
    assert_eq!(t.obj(copy).owner, P0);
    assert_eq!(t.obj(t.g.current(dyn_)).owner, P1);
    assert!(t.g.permitted_cards(P0).contains(&copy));
    assert!(!t.g.permitted_cards(P1).contains(&copy));
}

#[test]
fn a_spells_characteristics_are_its_cards_as_modified_by_effects() {
    cr!("112.3");
    let mut t = TestGame::new(2);
    let bears = t.hand(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 2);
    let spell = t.cast(P0, bears).go();
    let c = &t.obj(spell).chars;
    assert_eq!(c.name.as_str(), "Grizzly Bears");
    assert_eq!(c.colors, cs("G"));
    assert_eq!(c.power, Some(2));
    // Chaoslace: "Target spell or permanent becomes red."
    let lace = t.hand(P1, "Chaoslace");
    t.lands(P1, "Mountain", 1);
    t.g.turn.priority = Some(P1);
    t.cast(P1, lace).target(spell).go();
    t.resolve();
    assert_eq!(t.obj(spell).chars.colors, cs("R"));
}

#[test]
fn an_effect_on_a_permanent_spell_continues_to_apply_to_the_permanent() {
    cr!("112.4");
    let mut t = TestGame::new(2);
    let bears = t.hand(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 2);
    let spell = t.cast(P0, bears).go();
    let lace = t.hand(P1, "Chaoslace");
    t.lands(P1, "Mountain", 1);
    t.g.turn.priority = Some(P1);
    t.cast(P1, lace).target(spell).go();
    t.resolve();
    t.resolve();
    let perm = t.named_on_battlefield("Grizzly Bears")[0];
    assert_eq!(t.obj(perm).chars.colors, cs("R"));
    // It remains red for the duration of the effect.
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.obj(perm).chars.colors, cs("R"));
}
