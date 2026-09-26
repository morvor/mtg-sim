//! CR 715: adventurer cards.

use super::r709_common::*;
use mtg_engine::ability::*;
use mtg_engine::adventure::{self, Inset};
use mtg_engine::card::{card, Layout};
use mtg_engine::eval::Ctx;
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Lovestruck Beast ({2}{G} 5/5 Beast Noble, "This creature can't attack unless you
/// control a 1/1 creature.") // Heart's Desire ({G} Sorcery — Adventure, "Create a 1/1
/// white Human creature token.").
const BEAST: &str = "Lovestruck Beast // Heart's Desire";

fn humans(t: &TestGame) -> usize {
    t.named_on_battlefield("Human Token").len()
}

fn has_adventure(t: &mut TestGame, id: ObjectId) -> bool {
    t.g.recompute();
    t.g.matches(
        id,
        &Filter::Custom(adventure::HAS_ADVENTURE.into()),
        &Ctx::new(None, P0),
    )
}

#[test]
fn an_adventurer_card_has_an_inset_with_alternative_characteristics() {
    cr!("715.1", "715.2");
    supported(BEAST);
    let def = card(BEAST);
    assert_eq!(def.layout, Layout::Adventure);
    assert_eq!(adventure::inset_of(&def), Some(Inset::Adventure));
    let inset = &def.faces[1].chars;
    assert_eq!(inset.name, "Heart's Desire");
    assert!(inset.is(CardType::Sorcery) && inset.has_subtype("Adventure"));
    // Its normal characteristics appear as usual.
    let mut t = TestGame::new(2);
    let beast = t.hand(P0, BEAST);
    t.g.recompute();
    let c = &t.obj(beast).chars;
    assert_eq!(c.name, "Lovestruck Beast");
    assert!(c.is_creature() && !c.is(CardType::Sorcery));
}

#[test]
fn a_card_that_has_an_adventure_even_when_not_cast_as_one() {
    cr!("715.2a");
    supported("Edgewall Innkeeper");
    // Edgewall Innkeeper: "Whenever you cast a creature spell that has an Adventure, draw
    // a card. (It doesn't need to have gone on the adventure first.)"
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Edgewall Innkeeper");
    t.lands(P0, "Forest", 5);
    let beast = t.hand(P0, BEAST);
    assert!(has_adventure(&mut t, beast));
    t.cast(P0, beast).go();
    t.settle();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 1);
    // A creature spell without an Adventure doesn't draw.
    let bears = t.hand(P0, "Grizzly Bears");
    assert!(!has_adventure(&mut t, bears));
    t.cast(P0, bears).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 1);
}

#[test]
fn a_copy_of_an_adventurer_has_an_adventure() {
    cr!("715.2b");
    ruling!(
        "Lovestruck Beast // Heart's Desire",
        "If an object becomes a copy of an object that has an Adventure, the copy also has an Adventure"
    );
    let mut t = TestGame::new(2);
    let beast = t.battlefield(P1, BEAST);
    t.answer_choose(P0, &[Entity::Object(beast)]);
    t.answer_yes(P0, true);
    let clone = t.enter(P0, "Clone");
    let clone = t.g.current(clone);
    assert_eq!(t.obj(clone).chars.name, "Lovestruck Beast");
    assert!(has_adventure(&mut t, clone));
    // Once it leaves the battlefield it's a Clone card again.
    let gy = t
        .g
        .move_object(clone, Zone::Graveyard(P0), mtg_engine::events::MoveCause::Destroy, None)
        .unwrap();
    assert!(!has_adventure(&mut t, gy));
}


#[test]
fn an_adventurer_card_is_one_card() {
    cr!("715.2c");
    let mut t = TestGame::new(2);
    t.library_top(P0, BEAST);
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
fn a_player_chooses_to_cast_it_normally_or_as_an_adventure() {
    cr!("715.3", "715.3a", "715.3b");
    ruling!(
        "Lovestruck Beast // Heart's Desire",
        "When casting a spell as an Adventure, use the alternative characteristics and ignore all of the card's normal characteristics"
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let beast = t.hand(P0, BEAST);
    let faces: Vec<FaceState> = t
        .g
        .cast_options(P0, beast)
        .into_iter()
        .map(|o| o.face)
        .collect();
    assert_eq!(faces, vec![FaceState::Front, FaceState::Half(1)]);
    // With one Forest, only Heart's Desire ({G}) can be cast.
    t.lands(P0, "Forest", 1);
    assert!(!can_cast_face(&mut t, P0, beast, FaceState::Front));
    assert!(can_cast_face(&mut t, P0, beast, FaceState::Half(1)));
    let spell = cast_half(&mut t, P0, beast, 1);
    let c = t.obj(spell).chars.clone();
    assert_eq!(c.name, "Heart's Desire");
    assert!(c.is(CardType::Sorcery) && !c.is_creature());
    assert_eq!((c.power, c.toughness), (None, None));
    assert_eq!(mv(&mut t, spell), 1);
    // Timing is the Adventure's: Stomp is an instant, Bonecrusher Giant a creature.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P0, "Mountain", 3);
    let giant = t.hand(P0, "Bonecrusher Giant // Stomp");
    assert!(!can_cast_face(&mut t, P0, giant, FaceState::Front));
    // Priority during an opponent's turn: only the instant.
    t.g.turn.priority = Some(P0);
    assert!(t
        .cast(P0, giant)
        .method(CastMethod::Half(1))
        .target(P1)
        .try_go()
        .is_ok());
}

#[test]
fn a_copy_of_an_adventure_spell_is_an_adventure() {
    cr!("715.3c");
    ruling!(
        "Lovestruck Beast // Heart's Desire",
        "If an effect copies an Adventure spell, that copy is exiled as it resolves"
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Forest", 1);
    let beast = t.hand(P0, BEAST);
    let spell = cast_half(&mut t, P0, beast, 1);
    run_effect(
        &mut t,
        P0,
        None,
        &[Entity::Object(spell)],
        Effect::CopySpell {
            what: Sel::Target(0),
            count: Value::c(1),
            new_targets: false,
        },
    );
    let copy = t.g.stack[1];
    assert_eq!(t.obj(copy).kind, ObjKind::SpellCopy);
    assert_eq!(t.obj(copy).chars.name, "Heart's Desire");
    assert!(t.obj(copy).chars.has_subtype("Adventure"));
    assert_eq!(adventure::on_stack_as(&t.g, copy), Some(Inset::Adventure));
    t.resolve();
    // The copy was exiled as it resolved, then ceased to exist; the card is still on the
    // stack.
    t.settle();
    assert_eq!(humans(&t), 1);
    assert!(t
        .g
        .exile
        .iter()
        .all(|id| t.obj(*id).kind != ObjKind::SpellCopy));
    t.resolve_all();
    assert_eq!(humans(&t), 2);
    // Lucky Clover: "Whenever you cast an Adventure instant or sorcery spell, copy it."
    supported("Lucky Clover");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Lucky Clover");
    t.lands(P0, "Forest", 1);
    let beast = t.hand(P0, BEAST);
    cast_half(&mut t, P0, beast, 1);
    t.settle();
    t.resolve();
    let copy = t.g.stack[1];
    assert_eq!(adventure::on_stack_as(&t.g, copy), Some(Inset::Adventure));
    t.resolve_all();
    assert_eq!(humans(&t), 2);
    // Only the card went on an adventure (the copy ceased to exist).
    assert_eq!(t.g.find_in_zone(Zone::Exile, "Lovestruck Beast").len(), 1);
}

#[test]
fn a_resolved_adventure_is_exiled_and_may_be_cast_later_but_not_as_an_adventure() {
    cr!("715.3d", "715.4");
    ruling!(
        "Lovestruck Beast // Heart's Desire",
        "For as long as it remains exiled, that player may cast it as a permanent spell"
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Forest", 4);
    let beast = t.hand(P0, BEAST);
    cast_half(&mut t, P0, beast, 1);
    t.resolve_all();
    assert_eq!(humans(&t), 1);
    assert!(t.graveyard_size(P0) == 0);
    let exiled = t.g.find_in_zone(Zone::Exile, "Lovestruck Beast");
    assert_eq!(exiled.len(), 1);
    let ex = exiled[0];
    // In exile it has only its normal characteristics.
    let c = &t.obj(ex).chars;
    assert!(c.is_creature() && c.name == "Lovestruck Beast");
    // Its owner may cast it — only as the creature.
    let methods: Vec<FaceState> = t
        .g
        .cast_options(P0, ex)
        .into_iter()
        .map(|o| o.face)
        .collect();
    assert_eq!(methods, vec![FaceState::Front]);
    // The opponent can't.
    assert!(t.g.cast_options(P1, ex).is_empty());
    let spell = t.cast(P0, ex).go();
    assert_eq!(t.obj(spell).chars.name, "Lovestruck Beast");
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Lovestruck Beast").len(), 1);
}

#[test]
fn a_countered_adventure_goes_to_the_graveyard() {
    cr!("715.3d");
    ruling!(
        "Lovestruck Beast // Heart's Desire",
        "that card won't be exiled and the spell's controller won't be able to cast it as a permanent later"
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Forest", 1);
    let beast = t.hand(P0, BEAST);
    let spell = cast_half(&mut t, P0, beast, 1);
    t.g.counter(spell, None);
    assert!(t.in_graveyard(P0, "Lovestruck Beast"));
    assert!(t.g.exile.is_empty());
}

#[test]
fn outside_the_stack_it_has_only_its_normal_characteristics() {
    cr!("715.4");
    ruling!(
        "Lovestruck Beast // Heart's Desire",
        "An adventurer card is a permanent card in every zone except the stack"
    );
    let mut t = TestGame::new(2);
    let gy = t.graveyard(P0, BEAST);
    t.g.recompute();
    let ctx = Ctx::new(None, P0);
    assert!(t.g.matches(gy, &Filter::Type(CardType::Creature), &ctx));
    assert!(!t.g.matches(gy, &Filter::Type(CardType::Sorcery), &ctx));
    assert_eq!(mv(&mut t, gy), 3);
    // Cast normally, on the stack it's the creature spell too.
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Forest", 3);
    let beast = t.hand(P0, BEAST);
    let spell = t.cast(P0, beast).go();
    assert_eq!(t.obj(spell).chars.name, "Lovestruck Beast");
    assert_eq!(adventure::on_stack_as(&t.g, spell), None);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Lovestruck Beast").len(), 1);
    assert!(t.g.exile.is_empty());
}

#[test]
fn the_adventures_name_may_be_chosen() {
    cr!("715.5");
    ruling!(
        "Lovestruck Beast // Heart's Desire",
        "If an effect instructs you to choose a card name, you may choose the alternative Adventure name"
    );
    let mut t = TestGame::new(2);
    name_card(&mut t, P1, "Heart's Desire");
    let mage = t.enter(P1, "Meddling Mage");
    assert_eq!(t.obj_now(mage).choices.card_name.as_deref(), Some("Heart's Desire"));
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Forest", 3);
    let beast = t.hand(P0, BEAST);
    assert!(t.cast(P0, beast).method(CastMethod::Half(1)).try_go().is_err());
    let beast = t.g.current(beast);
    assert!(t.cast(P0, beast).try_go().is_ok());
}
