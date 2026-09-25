//! CR 708: face-down spells and permanents.

use crate::r703_common::*;
use mtg_engine::ability::*;
use mtg_engine::decision::{Action, SpecialAction};
use mtg_engine::events::Event;
use mtg_engine::facedown;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// Casts `name` face down with its disguise ability (for {3}). Returns the spell.
fn cast_disguised(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    t.lands(p, "Wastes", 3);
    let card = t.hand(p, name);
    t.cast(p, card)
        .method(CastMethod::FaceDown(KeywordKind::Disguise))
        .go()
}

/// A face-down permanent: `name` cast face down with disguise and resolved.
fn disguised(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    let s = cast_disguised(t, p, name);
    t.resolve_all();
    let id = t.g.current(s);
    assert!(t.obj(id).face_down && t.on_battlefield(id));
    id
}

/// Puts `name` from `p`'s hand onto the battlefield face down. Returns the permanent.
fn put_face_down(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    let card = t.hand(p, name);
    run_effect(
        t,
        p,
        None,
        Effect::Move {
            what: Sel::Target(0),
            to: Destination {
                face_down: true,
                ..Destination::battlefield()
            },
        },
        &[Entity::Object(card)],
    );
    let id = t.g.current(card);
    assert!(t.obj(id).face_down && t.on_battlefield(id));
    id
}

fn turn_up_action(t: &mut TestGame, p: PlayerId, obj: ObjectId) -> bool {
    t.g.turn.priority = Some(p);
    t.g.legal_actions(p)
        .contains(&Action::Special(SpecialAction::TurnFaceUp { obj }))
}

fn turn_up(t: &mut TestGame, p: PlayerId, obj: ObjectId) -> bool {
    t.g.turn.priority = Some(p);
    t.g.perform_action(p, Action::Special(SpecialAction::TurnFaceUp { obj }))
        .is_ok()
}

fn turn_down(t: &mut TestGame, id: ObjectId) {
    run_effect(
        t,
        P0,
        None,
        Effect::TurnFaceDown {
            what: Sel::Target(0),
        },
        &[Entity::Object(id)],
    );
}

/// `obj` becomes a copy of `of`.
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
}

fn revealed(t: &TestGame) -> Vec<ObjectId> {
    t.turn_events
        .iter()
        .chain(t.g.events.iter())
        .filter_map(|e| match e {
            Event::Custom { name, obj, .. } if name == facedown::REVEALED => *obj,
            _ => None,
        })
        .collect()
}

fn assert_blank_2_2(t: &TestGame, id: ObjectId) {
    let o = t.obj_now(id);
    assert!(o.chars.name.is_empty(), "{:?}", o.chars.name);
    assert_eq!((o.chars.power, o.chars.toughness), (Some(2), Some(2)));
    assert!(o.chars.is(CardType::Creature));
    assert!(o.chars.subtypes.is_empty());
    assert!(o.chars.mana_cost.is_none());
    assert_eq!(o.chars.colors, ColorSet::NONE);
}

#[test]
fn face_down_spells_and_permanents_have_only_the_listed_characteristics() {
    cr!("708.1", "708.2", "708.4");
    ruling!(
        "Bubble Smuggler",
        "The creature spell is a 2/2 creature spell with ward {2} that has no name, mana cost, or creature types."
    );
    supported("Nightdrinker Moroii");
    let mut t = TestGame::new(2);
    // "Whenever a player casts a black spell" sees only the face-down spell: colorless.
    let watcher = oracle_card(
        "Black Watcher",
        "Enchantment",
        "{0}",
        None,
        "Whenever a player casts a black spell, you gain 1 life.",
    );
    t.custom(P1, watcher, Zone::Battlefield);
    let spell = cast_disguised(&mut t, P0, "Nightdrinker Moroii");
    assert!(t.obj(spell).face_down);
    assert_blank_2_2(&t, spell);
    assert_eq!(t.g.mana_value_of(spell), 0);
    // Disguise lists ward {2}.
    assert!(t.obj(spell).has_keyword(KeywordKind::Ward));
    assert!(!t.obj(spell).has_keyword(KeywordKind::Flying));
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    // The permanent it becomes is face down, with the same characteristics (and the
    // Moroii's "When this creature enters, you lose 3 life" never existed).
    let moroii = t.g.current(spell);
    assert!(t.obj(moroii).face_down);
    assert_blank_2_2(&t, moroii);
    assert!(t.obj(moroii).has_keyword(KeywordKind::Ward));
    assert_eq!(t.life(P0), 20);
    // Those are its copiable values: a Clone copying it is a face-up 2/2 with no name.
    t.answer_choose(P1, &[Entity::Object(moroii)]);
    let clone = t.enter(P1, "Clone");
    assert!(!t.obj_now(clone).face_down);
    assert_blank_2_2(&t, clone);
}

#[test]
fn a_permanent_turned_face_down_without_listed_characteristics_is_a_blank_2_2() {
    cr!("708.2a");
    supported("Wall of Deceit");
    let mut t = TestGame::new(2);
    let wall = t.battlefield(P0, "Wall of Deceit");
    t.g.tap(wall);
    t.lands(P0, "Island", 3);
    t.activate(P0, wall, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.obj(wall).face_down);
    assert_blank_2_2(&t, wall);
    // No text: not even defender.
    assert!(!t.obj(wall).has_keyword(KeywordKind::Defender));
    assert!(t.obj(wall).chars.abilities.is_empty());
    // It's the same permanent: its status didn't change.
    assert!(t.obj(wall).tapped);
    // A permanent that was cast with disguise and turned face up, then turned face down
    // by an effect that lists no characteristics, doesn't get ward {2} back.
    let moroii = disguised(&mut t, P0, "Nightdrinker Moroii");
    t.lands(P0, "Swamp", 2);
    assert!(turn_up(&mut t, P0, moroii));
    assert_eq!(t.obj(moroii).chars.name.as_str(), "Nightdrinker Moroii");
    turn_down(&mut t, moroii);
    assert_blank_2_2(&t, moroii);
    assert!(!t.obj(moroii).has_keyword(KeywordKind::Ward));
}

#[test]
fn a_face_down_permanent_cant_be_turned_face_down() {
    cr!("708.2b");
    let mut t = TestGame::new(2);
    let moroii = disguised(&mut t, P0, "Nightdrinker Moroii");
    let ts = t.obj(moroii).timestamp;
    let events = t.turn_events.len();
    turn_down(&mut t, moroii);
    // Nothing happened: no new status, timestamp, or characteristics.
    assert!(t.obj(moroii).has_keyword(KeywordKind::Ward));
    assert_eq!(t.obj(moroii).timestamp, ts);
    assert!(t.turn_events[events..]
        .iter()
        .all(|e| !matches!(e, Event::TurnedFaceDown { .. })));
}

#[test]
fn permanents_put_onto_the_battlefield_face_down_have_no_enters_abilities() {
    cr!("708.3");
    supported("Wall of Omens");
    supported("Skyshroud Behemoth");
    let mut t = TestGame::new(2);
    let hand = t.hand_size(P0);
    put_face_down(&mut t, P0, "Wall of Omens");
    let behemoth = put_face_down(&mut t, P0, "Skyshroud Behemoth");
    t.resolve_all();
    // Wall of Omens' "When this creature enters, draw a card" didn't trigger.
    assert_eq!(t.hand_size(P0), hand);
    // Skyshroud Behemoth's fading and "This creature enters tapped" didn't apply.
    assert!(!t.obj(behemoth).tapped);
    assert_eq!(t.counters(behemoth, "fade"), 0);
    // Cast face down, the same (Alley Assailant: "This creature enters tapped.").
    let a = disguised(&mut t, P0, "Alley Assailant");
    assert!(!t.obj(a).tapped);
}

#[test]
fn you_may_look_at_your_face_down_spells_and_permanents_only() {
    cr!("708.5");
    ruling!(
        "Bubble Smuggler",
        "At any time, you can look at a face-down spell or permanent you control. You can't look at face-down permanents or spells you don't control unless an effect instructs or allows you to do so."
    );
    let mut t = TestGame::new(2);
    let spell = cast_disguised(&mut t, P0, "Nightdrinker Moroii");
    assert!(facedown::can_look_at(&t.g, P0, spell));
    assert!(!facedown::can_look_at(&t.g, P1, spell));
    t.resolve_all();
    let moroii = t.g.current(spell);
    assert!(facedown::can_look_at(&t.g, P0, moroii));
    assert!(!facedown::can_look_at(&t.g, P1, moroii));
    // Even while it's phased out.
    mtg_engine::kw::phasing::phase_out(&mut t.g, vec![moroii]);
    assert!(facedown::can_look_at(&t.g, P0, moroii));
    // Not a face-down card in another zone, even one you own.
    let card = t.hand(P0, "Grizzly Bears");
    run_effect(
        &mut t,
        P0,
        None,
        Effect::Exile {
            what: Sel::Target(0),
            face_down: true,
            link: false,
        },
        &[Entity::Object(card)],
    );
    let exiled = t.g.current(card);
    assert!(t.obj(exiled).face_down);
    assert!(!facedown::can_look_at(&t.g, P0, exiled));
    // Face-up objects in public zones can be seen by anyone.
    let bears = t.battlefield(P1, "Grizzly Bears");
    assert!(facedown::can_look_at(&t.g, P0, bears));
}

#[test]
fn the_rules_that_made_it_face_down_can_let_it_be_turned_face_up() {
    cr!("708.7");
    ruling!(
        "Bubble Smuggler",
        "Only a face-down permanent can be turned face up this way; a face-down spell cannot."
    );
    let mut t = TestGame::new(2);
    let spell = cast_disguised(&mut t, P0, "Nightdrinker Moroii");
    t.lands(P0, "Swamp", 2);
    // A spell can't be turned face up.
    assert!(!turn_up_action(&mut t, P0, spell));
    assert!(!turn_up(&mut t, P0, spell));
    t.resolve_all();
    let moroii = t.g.current(spell);
    // Disguise lets its controller turn the permanent face up by paying its disguise
    // cost; morph likewise.
    assert!(turn_up_action(&mut t, P0, moroii));
    assert!(!turn_up_action(&mut t, P1, moroii));
    assert!(turn_up(&mut t, P0, moroii));
    assert!(!t.obj(moroii).face_down);
    assert_eq!(t.obj(moroii).chars.name.as_str(), "Nightdrinker Moroii");
    let demon = put_face_down(&mut t, P0, "Grinning Demon");
    t.lands(P0, "Swamp", 4);
    assert!(turn_up(&mut t, P0, demon));
    assert_eq!(t.pt(demon), (6, 6));
    // A card without such an ability put onto the battlefield face down can't be.
    let b = put_face_down(&mut t, P0, "Grizzly Bears");
    t.lands(P0, "Forest", 4);
    assert!(!turn_up_action(&mut t, P0, b));
    assert!(!turn_up(&mut t, P0, b));
}

#[test]
fn turning_face_up_reverts_copiable_values_and_keeps_effects() {
    cr!("708.8");
    ruling!(
        "Bubble Smuggler",
        "Because the permanent is on the battlefield both before and after it's turned face up, turning a permanent face up doesn't cause any enters-the-battlefield abilities to trigger."
    );
    let mut t = TestGame::new(2);
    let moroii = disguised(&mut t, P0, "Nightdrinker Moroii");
    t.lands(P0, "Forest", 1);
    let gg = t.hand(P0, "Giant Growth");
    t.cast(P0, gg).target(moroii).go();
    t.resolve_all();
    assert_eq!(t.pt(moroii), (5, 5));
    t.lands(P0, "Swamp", 2);
    assert!(turn_up(&mut t, P0, moroii));
    t.resolve_all();
    // Its copiable values are the Moroii's again; Giant Growth still applies.
    assert_eq!(t.obj(moroii).copiable.name.as_str(), "Nightdrinker Moroii");
    assert_eq!(t.pt(moroii), (7, 5));
    // "When this creature enters, you lose 3 life" doesn't trigger.
    assert_eq!(t.life(P0), 20);
    // Neither does a static "enters tapped" ability apply.
    let a = disguised(&mut t, P0, "Alley Assailant");
    t.lands(P0, "Swamp", 6);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    assert!(turn_up(&mut t, P0, a));
    t.resolve_all();
    assert!(!t.obj(a).tapped);
    assert_eq!(t.obj(a).chars.name.as_str(), "Alley Assailant");
}

#[test]
fn face_down_objects_are_revealed_when_they_leave_and_at_the_end() {
    cr!("708.9");
    ruling!(
        "Bubble Smuggler",
        "If a face-down spell leaves the stack and goes to any zone other than the battlefield (if it was countered, for example), you must reveal it."
    );
    let mut t = TestGame::new(3);
    // A face-down permanent leaving the battlefield.
    let moroii = disguised(&mut t, P0, "Nightdrinker Moroii");
    t.g.destroy(moroii, None);
    t.settle();
    assert_eq!(revealed(&t), vec![moroii]);
    // A face-down spell countered.
    let spell = cast_disguised(&mut t, P0, "Bubble Smuggler");
    run_effect(
        &mut t,
        P1,
        None,
        Effect::CounterSpell {
            what: Sel::Target(0),
        },
        &[Entity::Object(spell)],
    );
    assert_eq!(revealed(&t), vec![moroii, spell]);
    // A face-down permanent that stays put isn't revealed; when its owner leaves the
    // game, it is.
    let p2s = put_face_down(&mut t, P2, "Grinning Demon");
    assert_eq!(revealed(&t).len(), 2);
    t.take_action(P2, Action::Concede);
    assert!(revealed(&t).contains(&p2s));
    // At the end of the game, every face-down permanent is revealed.
    let last = put_face_down(&mut t, P0, "Grinning Demon");
    t.take_action(P1, Action::Concede);
    assert!(t.g.result.is_some());
    assert!(revealed(&t).contains(&last));
}

#[test]
fn a_face_down_permanent_that_becomes_a_copy_stays_face_down() {
    cr!("708.10", "707.3");
    let mut t = TestGame::new(2);
    let lorian = t.battlefield(P1, "Branchsnap Lorian");
    let demon = put_face_down(&mut t, P0, "Grinning Demon");
    become_copy(&mut t, demon, lorian);
    // Its characteristics stay those of a face-down 2/2.
    assert_blank_2_2(&t, demon);
    // It can be turned face up for the copied creature's morph cost, {G} (not Grinning
    // Demon's); then it has the copied characteristics.
    t.lands(P0, "Forest", 1);
    assert!(turn_up(&mut t, P0, demon));
    assert_eq!(t.obj(demon).chars.name.as_str(), "Branchsnap Lorian");
    assert_eq!(t.pt(demon), (4, 1));
    assert!(t.obj(demon).has_keyword(KeywordKind::Trample));
    // A face-down permanent copying a creature without morph can't be turned face up as
    // a special action.
    let ones = t.battlefield(P1, "Wandering Ones");
    let other = put_face_down(&mut t, P0, "Grinning Demon");
    become_copy(&mut t, other, ones);
    assert_blank_2_2(&t, other);
    t.lands(P0, "Swamp", 4);
    assert!(!turn_up_action(&mut t, P0, other));
    // An effect that turns it face up makes it a Wandering Ones.
    run_effect(
        &mut t,
        P0,
        None,
        Effect::TurnFaceUp {
            what: Sel::Target(0),
        },
        &[Entity::Object(other)],
    );
    assert_eq!(t.obj(other).chars.name.as_str(), "Wandering Ones");
    assert_eq!(t.pt(other), (1, 1));
}

#[test]
fn as_turned_face_up_abilities_apply_while_it_is_turned_face_up() {
    cr!("708.11");
    supported("Bubble Smuggler");
    let mut t = TestGame::new(2);
    let smuggler = disguised(&mut t, P0, "Bubble Smuggler");
    t.lands(P0, "Island", 6);
    assert!(turn_up(&mut t, P0, smuggler));
    // "As this creature is turned face up, put four +1/+1 counters on it": the counters
    // were put on it before it was face up, not afterward.
    assert_eq!(t.counters(smuggler, counters::PLUS1), 4);
    assert_eq!(t.pt(smuggler), (6, 5));
    let events: Vec<&Event> = t.turn_events.iter().chain(t.g.events.iter()).collect();
    let face_up = events
        .iter()
        .position(|e| matches!(e, Event::TurnedFaceUp { .. }))
        .unwrap();
    let counters_added = events
        .iter()
        .position(|e| matches!(e, Event::CountersAdded { .. }))
        .unwrap();
    assert!(counters_added < face_up);
}

#[test]
fn a_revealed_face_down_permanent_is_judged_ignoring_continuous_effects() {
    cr!("708.12");
    ruling!(
        "Hauntwoods Shrieker",
        "Hauntwoods Shrieker's last ability considers only the characteristics of the printed card."
    );
    let mut t = TestGame::new(2);
    let shrieker = t.custom(
        P0,
        oracle_card(
            "Face Revealer",
            "Creature — Beast Mutant",
            "{1}{G}{G}",
            Some((3, 3)),
            "{1}{G}: Reveal target face-down permanent. If it's a creature card, you may turn it face up.",
        ),
        Zone::Battlefield,
    );
    // A face-down creature card that an effect has made a noncreature land is still a
    // creature card when revealed.
    let bears = put_face_down(&mut t, P0, "Grizzly Bears");
    run_effect(
        &mut t,
        P0,
        None,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![
                Modification::RemoveTypes(vec![CardType::Creature]),
                Modification::AddTypes(vec![CardType::Land]),
            ],
            duration: Duration::EndOfTurn,
        },
        &[Entity::Object(bears)],
    );
    assert!(!t.obj(bears).is_creature());
    // A face-down noncreature card is a creature (a 2/2) on the battlefield, but it isn't
    // a creature card.
    let ring = put_face_down(&mut t, P0, "Sol Ring");
    assert!(t.obj(ring).is_creature());
    t.lands(P0, "Forest", 4);
    t.answer_yes(P0, true);
    t.activate(P0, shrieker, 0, &[Entity::Object(bears)])
        .unwrap();
    t.resolve_all();
    assert!(revealed(&t).contains(&bears));
    assert!(!t.obj(bears).face_down);
    assert_eq!(t.obj(bears).chars.name.as_str(), "Grizzly Bears");
    t.answer_yes(P0, true);
    t.g.objects[shrieker.0 as usize]
        .activations_this_turn
        .clear();
    t.activate(P0, shrieker, 0, &[Entity::Object(ring)])
        .unwrap();
    t.resolve_all();
    assert!(revealed(&t).contains(&ring));
    assert!(t.obj(ring).face_down);
}
