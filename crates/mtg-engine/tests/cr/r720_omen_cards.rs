//! CR 720: omen cards.

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

/// Dirgur Island Dragon ({5}{U} 4/4 flying, ward {2}) // Skimming Strike ({1}{U} Instant —
/// Omen, "Tap up to one target creature. Draw a card.").
const DRAGON: &str = "Dirgur Island Dragon // Skimming Strike";

fn has_omen(t: &mut TestGame, id: ObjectId) -> bool {
    t.g.recompute();
    t.g.matches(
        id,
        &Filter::Custom(adventure::HAS_OMEN.into()),
        &Ctx::new(None, P0),
    )
}

/// Casts Skimming Strike (tapping `target`) with {1}{U} from two Islands.
fn cast_strike(t: &mut TestGame, target: ObjectId) -> ObjectId {
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Island", 2);
    let dragon = t.hand(P0, DRAGON);
    t.cast(P0, dragon)
        .method(CastMethod::Half(1))
        .target(target)
        .go()
}

#[test]
fn an_omen_card_has_an_inset_with_alternative_characteristics() {
    cr!("720.1", "720.2");
    supported(DRAGON);
    let def = card(DRAGON);
    assert_eq!(def.layout, Layout::Adventure);
    assert_eq!(adventure::inset_of(&def), Some(Inset::Omen));
    let inset = &def.faces[1].chars;
    assert_eq!(inset.name, "Skimming Strike");
    assert!(inset.is(CardType::Instant) && inset.has_subtype("Omen"));
    let mut t = TestGame::new(2);
    let d = t.hand(P0, DRAGON);
    t.g.recompute();
    assert_eq!(t.obj(d).chars.name, "Dirgur Island Dragon");
    assert!(t.obj(d).chars.is_creature());
}

#[test]
fn a_card_that_has_an_omen() {
    cr!("720.2a", "720.2b");
    let mut t = TestGame::new(2);
    let d = t.hand(P0, DRAGON);
    let gy = t.graveyard(P0, DRAGON);
    assert!(has_omen(&mut t, d) && has_omen(&mut t, gy));
    let beast = t.hand(P0, "Lovestruck Beast // Heart's Desire");
    assert!(!has_omen(&mut t, beast));
    // A copy of an omen card's permanent has an Omen (part of its copiable values).
    let dragon = t.battlefield(P1, DRAGON);
    t.answer_choose(P0, &[Entity::Object(dragon)]);
    t.answer_yes(P0, true);
    let clone = t.enter(P0, "Clone");
    let clone = t.g.current(clone);
    assert_eq!(t.obj(clone).chars.name, "Dirgur Island Dragon");
    assert!(has_omen(&mut t, clone));
}

#[test]
fn an_omen_card_is_one_card() {
    cr!("720.2c");
    let mut t = TestGame::new(2);
    t.library_top(P0, DRAGON);
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
    assert_eq!(
        t.g.eval_value(&Value::CardsDrawnThisTurn(PlayerRef::You), &ctx),
        1
    );
}

#[test]
fn a_player_chooses_to_cast_it_normally_or_as_an_omen() {
    cr!("720.3", "720.3a", "720.3b");
    ruling!(
        "Dirgur Island Dragon // Skimming Strike",
        "As a player casts an omen card, the player chooses whether they cast the card normally or as an Omen"
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let d = t.hand(P0, DRAGON);
    let faces: Vec<FaceState> =
        t.g.cast_options(P0, d)
            .into_iter()
            .map(|o| o.face)
            .collect();
    assert_eq!(faces, vec![FaceState::Front, FaceState::Half(1)]);
    t.lands(P0, "Island", 2);
    assert!(!can_cast_face(&mut t, P0, d, FaceState::Front));
    assert!(can_cast_face(&mut t, P0, d, FaceState::Half(1)));
    let bears = t.battlefield(P1, "Grizzly Bears");
    let spell = t.cast(P0, d).method(CastMethod::Half(1)).target(bears).go();
    let c = t.obj(spell).chars.clone();
    assert_eq!(c.name, "Skimming Strike");
    assert!(c.is(CardType::Instant) && !c.is_creature());
    assert_eq!(mv(&mut t, spell), 2);
    assert_eq!(adventure::on_stack_as(&t.g, spell), Some(Inset::Omen));
}

#[test]
fn a_copy_of_an_omen_spell_is_an_omen() {
    cr!("720.3c");
    ruling!(
        "Dirgur Island Dragon // Skimming Strike",
        "If an Omen spell is copied, that copy is also an Omen"
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let spell = cast_strike(&mut t, bears);
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
    assert_eq!(t.obj(copy).chars.name, "Skimming Strike");
    assert!(t.obj(copy).chars.has_subtype("Omen"));
    assert_eq!(adventure::on_stack_as(&t.g, copy), Some(Inset::Omen));
    let lib = t.library_size(P0);
    t.resolve();
    t.settle();
    // The copy was put into the library and ceased to exist; a card was drawn.
    assert_eq!(t.library_size(P0), lib - 1);
    assert!(t
        .g
        .player(P0)
        .library
        .iter()
        .all(|id| t.obj(*id).kind == ObjKind::Card));
}

#[test]
fn a_resolving_omen_is_shuffled_into_its_owners_library() {
    cr!("720.3d", "720.4");
    ruling!(
        "Dirgur Island Dragon // Skimming Strike",
        "its controller shuffles it into its owner’s library instead of putting it into its owner’s graveyard"
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let lib = t.library_size(P0);
    cast_strike(&mut t, bears);
    let shuffles = |t: &TestGame| {
        t.g.turn_events
            .iter()
            .chain(t.g.events.iter())
            .filter(
                |e| matches!(e, mtg_engine::events::Event::Shuffled { player } if *player == P0),
            )
            .count()
    };
    assert_eq!(shuffles(&t), 0);
    t.resolve_all();
    assert!(t.obj(bears).tapped);
    // Shuffled in, not just put into the library.
    assert_eq!(shuffles(&t), 1);
    // It drew a card and went into the library: the same count, plus the dragon.
    assert_eq!(t.library_size(P0), lib);
    assert_eq!(t.graveyard_size(P0), 0);
    let found = t.g.find_in_zone(Zone::Library(P0), "Dirgur Island Dragon");
    assert_eq!(found.len(), 1);
    // In the library it's the creature card.
    assert!(t.obj(found[0]).chars.is_creature());
    // Countered, it goes to the graveyard instead.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let spell = cast_strike(&mut t, bears);
    t.g.counter(spell, None);
    assert!(t.in_graveyard(P0, "Dirgur Island Dragon"));
}

#[test]
fn outside_the_stack_it_has_only_its_normal_characteristics() {
    cr!("720.4");
    ruling!(
        "Dirgur Island Dragon // Skimming Strike",
        "An omen card is a creature card in every zone except the stack"
    );
    let mut t = TestGame::new(2);
    let gy = t.graveyard(P0, DRAGON);
    t.g.recompute();
    let c = &t.obj(gy).chars;
    assert!(c.is_creature() && !c.is(CardType::Instant));
    assert_eq!(mv(&mut t, gy), 6);
    // Cast normally, it's the creature spell.
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Island", 6);
    let d = t.hand(P0, DRAGON);
    let spell = t.cast(P0, d).go();
    assert_eq!(t.obj(spell).chars.name, "Dirgur Island Dragon");
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Dirgur Island Dragon").len(), 1);
}

#[test]
fn the_omens_name_may_be_chosen() {
    cr!("720.5");
    ruling!(
        "Dirgur Island Dragon // Skimming Strike",
        "If an effect instructs you to choose a card name, you may choose the alternative Omen name"
    );
    let mut t = TestGame::new(2);
    name_card(&mut t, P1, "Skimming Strike");
    let mage = t.enter(P1, "Meddling Mage");
    assert_eq!(
        t.obj_now(mage).choices.card_name.as_deref(),
        Some("Skimming Strike")
    );
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Island", 6);
    let d = t.hand(P0, DRAGON);
    let bears = t.battlefield(P1, "Grizzly Bears");
    assert!(t
        .cast(P0, d)
        .method(CastMethod::Half(1))
        .target(bears)
        .try_go()
        .is_err());
    let d = t.g.current(d);
    assert!(t.cast(P0, d).try_go().is_ok());
}
