//! CR 701.58: cloak.

use crate::a701_028_071_common::*;
use mtg_engine::ability::*;
use mtg_engine::decision::{Action, SpecialAction};
use mtg_engine::events::{Event, MoveCause};
use mtg_engine::facedown::REVEALED;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::kwa::manifest::CLOAKED;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// Cloaks the top card of P0's library (after putting `top` there).
fn cloak(t: &mut TestGame, top: &str) -> ObjectId {
    let card = t.library_top(P0, top);
    run(t, P0, None, ka(KeywordAction::Cloak, Sel::None, 1), &[]);
    t.g.current(card)
}

fn turn_up(t: &mut TestGame, p: PlayerId, obj: ObjectId) -> bool {
    t.g.turn.priority = Some(p);
    t.g.perform_action(p, Action::Special(SpecialAction::TurnFaceUp { obj }))
        .is_ok()
}

#[test]
fn a_cloaked_card_is_a_face_down_2_2_creature_with_ward_2() {
    cr!("701.58a");
    ruling!(
        "Cryptic Coat",
        "It becomes a 2/2 face-down creature card with ward {2} and no name, mana cost, or creature types."
    );
    supported("Cryptic Coat");
    // "When this Equipment enters, cloak the top card of your library, then attach this
    // Equipment to it."
    let mut t = TestGame::new(2);
    let card = t.library_top(P0, "Hill Giant");
    let coat = t.enter(P0, "Cryptic Coat");
    t.resolve_all();
    let c = t.g.current(card);
    let o = t.obj(c);
    assert!(o.face_down && o.is_card() && o.zone == Zone::Battlefield);
    assert_eq!(o.choices.text.as_deref(), Some(CLOAKED));
    assert!(o.chars.name.is_empty() && o.chars.subtypes.is_empty());
    assert!(o.chars.mana_cost.is_none());
    assert!(o.has_keyword(KeywordKind::Ward));
    let ward = o.chars.keyword(KeywordKind::Ward).unwrap();
    assert_eq!(format!("{:?}", ward.cost.as_ref().unwrap().mana), format!("{:?}", mtg_engine::mana::ManaCost::parse("{2}")));
    // Equipped: +1/+0.
    assert_eq!(t.obj(coat).attached_to, Some(Entity::Object(c)));
    assert_eq!(t.pt(c), (3, 2));
}

#[test]
fn a_cloaked_creature_card_turns_face_up_for_its_mana_cost() {
    cr!("701.58b");
    ruling!(
        "Cryptic Coat",
        "Any time you have priority, you can turn a cloaked permanent you control face-up by revealing that it's a creature card"
    );
    let mut t = TestGame::new(2);
    let c = cloak(&mut t, "Hill Giant");
    t.lands(P0, "Mountain", 4);
    assert!(turn_up(&mut t, P0, c));
    assert_eq!(t.obj(c).chars.name.as_str(), "Hill Giant");
    // Face up, it no longer has ward.
    assert!(!t.obj(c).has_keyword(KeywordKind::Ward));
    // A noncreature card can't be turned face up this way.
    let mut t = TestGame::new(2);
    let c = cloak(&mut t, "Lightning Bolt");
    t.lands(P0, "Mountain", 4);
    assert!(!turn_up(&mut t, P0, c));
    assert!(t.obj(c).face_down);
}

#[test]
fn a_cloaked_card_with_morph_turns_face_up_either_way() {
    cr!("701.58c");
    // Sagu Mauler: morph {3}{G}{U}.
    let mut t = TestGame::new(2);
    let c = cloak(&mut t, "Sagu Mauler");
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Island", 1);
    t.lands(P0, "Plains", 3);
    // Only the morph cost ({3}{G}{U}) can be paid, not the mana cost ({4}{G}{U}).
    assert!(turn_up(&mut t, P0, c));
    assert_eq!(t.obj(c).chars.name.as_str(), "Sagu Mauler");
    assert!(!t.obj(c).has_keyword(KeywordKind::Ward));
}

#[test]
fn a_cloaked_card_with_disguise_turns_face_up_either_way() {
    cr!("701.58d");
    ruling!(
        "Cryptic Coat",
        "If a cloaked creature would have disguise (or morph) if it were face up, you may also turn it face up by paying its disguise (or morph) cost."
    );
    // Nightdrinker Moroii: mana cost {3}{B}, disguise {B}{B}.
    // With four Swamps, either cost can be paid: the player chooses the disguise cost ...
    let mut t = TestGame::new(2);
    let c = cloak(&mut t, "Nightdrinker Moroii");
    let swamps = t.lands(P0, "Swamp", 4);
    option(&mut t, P0, 1);
    assert!(turn_up(&mut t, P0, c));
    assert_eq!(t.obj(c).chars.name.as_str(), "Nightdrinker Moroii");
    assert_eq!(swamps.iter().filter(|s| t.obj(**s).tapped).count(), 2);
    // ... or its mana cost.
    let mut t = TestGame::new(2);
    let c = cloak(&mut t, "Nightdrinker Moroii");
    let swamps = t.lands(P0, "Swamp", 4);
    option(&mut t, P0, 0);
    assert!(turn_up(&mut t, P0, c));
    assert!(swamps.iter().all(|s| t.obj(*s).tapped));
}

#[test]
fn several_cards_are_cloaked_one_at_a_time() {
    cr!("701.58e");
    let mut t = TestGame::new(2);
    let second = t.library_top(P0, "Grizzly Bears");
    let first = t.library_top(P0, "Hill Giant");
    run(&mut t, P0, None, ka(KeywordAction::Cloak, Sel::None, 2), &[]);
    let (a, b) = (t.g.current(first), t.g.current(second));
    assert!(t.obj(a).face_down && t.obj(b).face_down);
    let moves: Vec<ObjectId> = t
        .turn_events
        .iter()
        .chain(t.g.events.iter())
        .filter_map(|e| match e {
            Event::ZoneChange {
                new,
                to: Zone::Battlefield,
                ..
            } => Some(*new),
            _ => None,
        })
        .collect();
    assert_eq!(moves, vec![a, b]);
}

#[test]
fn a_card_that_cant_enter_the_battlefield_isnt_cloaked() {
    cr!("701.58f");
    let mut t = TestGame::new(2);
    run(
        &mut t,
        P1,
        None,
        Effect::AddRestriction {
            restriction: Restriction::CantEnter(Filter::creature()),
            duration: Duration::EndOfTurn,
        },
        &[],
    );
    let card = t.library_top(P0, "Hill Giant");
    run(&mut t, P0, None, ka(KeywordAction::Cloak, Sel::None, 1), &[]);
    assert!(t.g.is_live(card));
    assert_eq!(t.zone(card), Zone::Library(P0));
    assert!(!t.obj(card).face_down);
    assert_eq!(t.obj(card).chars.name.as_str(), "Hill Giant");
}

#[test]
fn a_cloaked_sorcery_that_would_turn_face_up_is_revealed_and_stays_face_down() {
    cr!("701.58g");
    ruling!(
        "Cryptic Coat",
        "If something tries to turn a face-down instant or sorcery card on the battlefield face up, reveal that card to show all players it's an instant or sorcery card."
    );
    let mut t = TestGame::new(2);
    let c = cloak(&mut t, "Divination");
    run(
        &mut t,
        P0,
        None,
        Effect::TurnFaceUp {
            what: Sel::Target(0),
        },
        &[Entity::Object(c)],
    );
    assert!(t.obj(c).face_down);
    assert!(t.obj(c).has_keyword(KeywordKind::Ward));
    assert!(t.turn_events.iter().any(
        |e| matches!(e, Event::Custom { name, obj: Some(o), .. } if name == REVEALED && *o == c)
    ));
    assert!(t
        .turn_events
        .iter()
        .all(|e| !matches!(e, Event::TurnedFaceUp { .. })));
}

#[test]
fn cloaked_permanents_follow_the_face_down_rules() {
    cr!("701.58h", "708.9");
    ruling!(
        "Cryptic Coat",
        "Similarly, if a face-down permanent leaves the battlefield, you must reveal it."
    );
    let mut t = TestGame::new(2);
    let c = cloak(&mut t, "Hill Giant");
    t.g.move_object(c, Zone::Hand(P0), MoveCause::Effect, None);
    t.g.flush_events();
    assert!(t.turn_events.iter().any(
        |e| matches!(e, Event::Custom { name, obj: Some(o), .. } if name == REVEALED && *o == c)
    ));
    assert!(t.in_hand(P0, "Hill Giant"));
}
