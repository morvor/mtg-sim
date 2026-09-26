//! CR 701.40: manifest.

use crate::a701_028_071_common::*;
use mtg_engine::ability::*;
use mtg_engine::decision::{Action, SpecialAction};
use mtg_engine::events::{Event, MoveCause};
use mtg_engine::facedown::REVEALED;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::kwa::manifest::MANIFESTED;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// Casts Soul Summons ("Manifest the top card of your library.") with `top` on top of
/// P0's library. Returns the manifested permanent.
fn summon(t: &mut TestGame, top: &str) -> ObjectId {
    supported("Soul Summons");
    let card = t.library_top(P0, top);
    t.lands(P0, "Plains", 2);
    let spell = t.hand(P0, "Soul Summons");
    t.cast(P0, spell).go();
    t.resolve_all();
    t.g.current(card)
}

fn can_turn_up(t: &mut TestGame, p: PlayerId, obj: ObjectId) -> bool {
    t.g.turn.priority = Some(p);
    t.g.legal_actions(p)
        .contains(&Action::Special(SpecialAction::TurnFaceUp { obj }))
}

fn turn_up(t: &mut TestGame, p: PlayerId, obj: ObjectId) -> bool {
    t.g.turn.priority = Some(p);
    t.g.perform_action(p, Action::Special(SpecialAction::TurnFaceUp { obj }))
        .is_ok()
}

#[test]
fn a_manifested_card_is_a_face_down_2_2_creature() {
    cr!("701.40a");
    ruling!(
        "Soul Summons",
        "The face-down permanent is a 2/2 creature with no name, mana cost, creature types, or abilities. It's colorless and has a mana value of 0."
    );
    let mut t = TestGame::new(2);
    let m = summon(&mut t, "Hill Giant");
    let o = t.obj(m);
    assert_eq!(o.zone, Zone::Battlefield);
    assert!(o.face_down && o.is_card());
    assert_eq!(o.choices.text.as_deref(), Some(MANIFESTED));
    assert!(o.chars.name.is_empty());
    assert_eq!((o.chars.power, o.chars.toughness), (Some(2), Some(2)));
    assert_eq!(o.chars.card_types, CardTypeSet::single(CardType::Creature));
    assert!(o.chars.subtypes.is_empty() && o.chars.mana_cost.is_none());
    assert!(o.chars.abilities.is_empty());
    assert_eq!(o.chars.colors, ColorSet::NONE);
    assert_eq!(t.g.mana_value_of(m), 0);
    assert_eq!(o.controller, P0);
    // Any card can be manifested, even a noncreature one.
    let mut t = TestGame::new(2);
    let m = summon(&mut t, "Lightning Bolt");
    assert!(t.obj(m).face_down && t.obj(m).is_creature());
}

#[test]
fn a_manifested_creature_card_turns_face_up_for_its_mana_cost() {
    cr!("701.40b");
    ruling!(
        "Soul Summons",
        "Any time you have priority, you may turn a manifested creature face up by revealing that it's a creature card (ignoring any copy effects or type-changing effects that might be applying to it) and paying its mana cost."
    );
    ruling!(
        "Soul Summons",
        "Unlike a face-down creature that was cast using the morph ability, a manifested creature may still be turned face up after it loses its abilities if it's a creature card."
    );
    let mut t = TestGame::new(2);
    let m = summon(&mut t, "Hill Giant");
    // It loses all abilities: it can still be turned face up.
    run(
        &mut t,
        P1,
        None,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::RemoveAllAbilities],
            duration: Duration::EndOfTurn,
        },
        &[Entity::Object(m)],
    );
    assert!(!can_turn_up(&mut t, P0, m));
    t.lands(P0, "Mountain", 4);
    assert!(can_turn_up(&mut t, P0, m));
    assert!(turn_up(&mut t, P0, m));
    assert!(!t.obj(m).face_down);
    assert_eq!(t.obj(m).chars.name.as_str(), "Hill Giant");
    assert_eq!(t.pt(m), (3, 3));
    // A special action: nothing went on the stack.
    assert_eq!(t.stack_len(), 0);
    // A noncreature card, or a creature card without a mana cost, can't be turned face up
    // this way.
    for name in ["Lightning Bolt", "Dryad Arbor"] {
        let mut t = TestGame::new(2);
        let m = summon(&mut t, name);
        t.lands(P0, "Mountain", 4);
        assert!(!can_turn_up(&mut t, P0, m), "{name}");
        assert!(!turn_up(&mut t, P0, m), "{name}");
        assert!(t.obj(m).face_down);
    }
}

#[test]
fn a_manifested_card_with_morph_turns_face_up_either_way() {
    cr!("701.40c");
    ruling!(
        "Soul Summons",
        "If a manifested creature would have morph if it were face up, you may also turn it face up by paying its morph cost."
    );
    // Sagu Mauler: mana cost {4}{G}{U}, morph {3}{G}{U}.
    let mut t = TestGame::new(2);
    let m = summon(&mut t, "Sagu Mauler");
    let lands = [
        t.lands(P0, "Forest", 1),
        t.lands(P0, "Island", 1),
        t.lands(P0, "Plains", 3),
    ]
    .concat();
    // Five lands: only the morph cost can be paid.
    assert!(can_turn_up(&mut t, P0, m));
    option(&mut t, P0, 1);
    assert!(turn_up(&mut t, P0, m));
    assert!(!t.obj(m).face_down);
    assert_eq!(t.obj(m).chars.name.as_str(), "Sagu Mauler");
    assert!(lands.iter().all(|l| t.obj(*l).tapped));
    // Or its mana cost.
    let mut t = TestGame::new(2);
    let m = summon(&mut t, "Sagu Mauler");
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Island", 1);
    let plains = t.lands(P0, "Plains", 5);
    option(&mut t, P0, 0);
    assert!(turn_up(&mut t, P0, m));
    assert!(!t.obj(m).face_down);
    assert_eq!(plains.iter().filter(|l| t.obj(**l).tapped).count(), 4);
}

#[test]
fn a_manifested_card_with_disguise_turns_face_up_either_way() {
    cr!("701.40d");
    // Nightdrinker Moroii: mana cost {3}{B}, disguise {B}{B}.
    let mut t = TestGame::new(2);
    let m = summon(&mut t, "Nightdrinker Moroii");
    // Manifested, not disguised: no ward {2}.
    assert!(!t.obj(m).has_keyword(KeywordKind::Ward));
    t.lands(P0, "Swamp", 2);
    assert!(can_turn_up(&mut t, P0, m));
    option(&mut t, P0, 1);
    assert!(turn_up(&mut t, P0, m));
    assert_eq!(t.obj(m).chars.name.as_str(), "Nightdrinker Moroii");
}

#[test]
fn several_cards_are_manifested_one_at_a_time() {
    cr!("701.40e");
    let mut t = TestGame::new(2);
    let second = t.library_top(P0, "Grizzly Bears");
    let first = t.library_top(P0, "Hill Giant");
    let text = text_card(
        "Two Manifests",
        "Sorcery",
        "{0}",
        None,
        "Manifest the top two cards of your library.",
    );
    let spell = t.custom(P0, text, Zone::Hand(P0));
    t.cast(P0, spell).go();
    t.resolve_all();
    let (a, b) = (t.g.current(first), t.g.current(second));
    assert!(t.obj(a).face_down && t.obj(b).face_down);
    // The top card first, then the new top card: two separate zone changes, in that order.
    let moves: Vec<ObjectId> = t
        .turn_events
        .iter()
        .filter_map(|e| match e {
            Event::ZoneChange {
                new,
                to: Zone::Battlefield,
                from: Zone::Library(_),
                ..
            } => Some(*new),
            _ => None,
        })
        .collect();
    assert_eq!(moves, vec![a, b]);
    assert!(t.obj(a).timestamp < t.obj(b).timestamp);
}

#[test]
fn a_card_that_cant_enter_the_battlefield_isnt_manifested() {
    cr!("701.40f");
    let mut t = TestGame::new(2);
    // "Creatures can't enter the battlefield": the face-down 2/2 creature can't.
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
    let card = t.library_top(P0, "Lightning Bolt");
    run(&mut t, P0, None, ka(KeywordAction::Manifest, Sel::None, 1), &[]);
    assert!(t.g.is_live(card));
    assert_eq!(t.zone(card), Zone::Library(P0));
    assert_eq!(t.g.library_top(P0), Some(card));
    let o = t.obj(card);
    assert!(!o.face_down);
    assert_eq!(o.chars.name.as_str(), "Lightning Bolt");
    assert!(o.choices.text.is_none());
}

#[test]
fn a_manifested_instant_that_would_turn_face_up_is_revealed_and_stays_face_down() {
    cr!("701.40g");
    ruling!(
        "Soul Summons",
        "If something tries to turn a face-down instant or sorcery card on the battlefield face up, reveal that card to show all players it's an instant or sorcery card. The permanent remains on the battlefield face down."
    );
    supported("Mastery of the Unseen");
    let mut t = TestGame::new(2);
    // "Whenever a permanent you control is turned face up, you gain 1 life for each
    // creature you control."
    t.battlefield(P0, "Mastery of the Unseen");
    let m = summon(&mut t, "Lightning Bolt");
    run(
        &mut t,
        P0,
        None,
        Effect::TurnFaceUp {
            what: Sel::Target(0),
        },
        &[Entity::Object(m)],
    );
    t.resolve_all();
    assert!(t.obj(m).face_down);
    assert!(t.on_battlefield(m));
    let revealed: Vec<ObjectId> = t
        .turn_events
        .iter()
        .filter_map(|e| match e {
            Event::Custom { name, obj, .. } if name == REVEALED => *obj,
            _ => None,
        })
        .collect();
    assert_eq!(revealed, vec![m]);
    assert!(t
        .turn_events
        .iter()
        .all(|e| !matches!(e, Event::TurnedFaceUp { .. })));
    assert_eq!(t.life(P0), 20);
}

#[test]
fn manifested_permanents_follow_the_face_down_rules() {
    cr!("701.40h", "708.9");
    ruling!(
        "Soul Summons",
        "If a face-down permanent you control leaves the battlefield, you must reveal it."
    );
    let mut t = TestGame::new(2);
    let m = summon(&mut t, "Hill Giant");
    t.g.move_object(m, Zone::Graveyard(P0), MoveCause::Destroy, None);
    t.g.flush_events();
    let revealed = t.turn_events.iter().any(
        |e| matches!(e, Event::Custom { name, obj: Some(o), .. } if name == REVEALED && *o == m),
    );
    assert!(revealed);
    assert!(t.in_graveyard(P0, "Hill Giant"));
}
