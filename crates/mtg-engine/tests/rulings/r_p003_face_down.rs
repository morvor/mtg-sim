//! Rulings batch P003 — Tezzeret, Cruel Machinist's −7 ("Put any number of cards from
//! your hand onto the battlefield face down. They're 5/5 artifact creatures.") and
//! Magar of the Magic Strings: face-down permanents with characteristics listed by the
//! effect (CR 708.2), copies of them (CR 707.2), looking at them (CR 708.5), revealing
//! them (CR 708.9), returning them (CR 400.7) and turning them face up (CR 708.8).
//! Also Depthshaker Titan on Vehicles and Spacecraft.

use crate::r_p003_common::*;
use crate::r_p017_common::enter_copying;
use crate::r_s01_common::supported;
use crate::r_s04_common::crew;
use mtg_engine::events::Event;
use mtg_engine::facedown;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// P0's Tezzeret with seven loyalty uses its −7, putting the named cards from P0's hand
/// onto the battlefield face down. Returns them (followed onto the battlefield).
fn tezzeret_ultimate(t: &mut TestGame, cards: &[&str]) -> Vec<ObjectId> {
    supported("Tezzeret, Cruel Machinist");
    let tez = t.battlefield(P0, "Tezzeret, Cruel Machinist");
    let n = t.counters(tez, counters::LOYALTY);
    put(t, tez, counters::LOYALTY, 7 - n);
    let ids: Vec<ObjectId> = cards.iter().map(|c| t.hand(P0, c)).collect();
    let es: Vec<Entity> = ids.iter().map(|i| obj(*i)).collect();
    t.answer_choose(P0, &es);
    t.activate(P0, tez, 2, &[]).expect("−7");
    t.resolve_all();
    ids.iter()
        .map(|i| {
            let now = t.g.current(*i);
            assert!(t.on_battlefield(now), "card {i:?} entered");
            now
        })
        .collect()
}

fn assert_face_down_5_5(t: &TestGame, id: ObjectId) {
    let o = t.obj_now(id);
    assert!(o.face_down);
    assert_eq!(o.chars.name, "");
    assert_eq!(t.pt(id), (5, 5));
    assert!(o.is(CardType::Artifact) && o.is(CardType::Creature));
    assert!(!o.is(CardType::Land) && !o.is(CardType::Instant));
    assert_eq!(o.chars.colors, ColorSet::default());
    assert!(o.chars.abilities.is_empty());
}

/// How many times the Vehicle became crewed this turn (CR 702.122e: a crew ability of it
/// resolved), and how many of its triggered abilities were put on the stack.
fn crewed_and_triggered(t: &TestGame, id: ObjectId) -> (usize, usize) {
    let crewed = t.g.turn_events.iter().filter(|e| {
        matches!(e, Event::Custom { name, obj: Some(o), .. }
            if name == mtg_engine::kw::crew::CREWED && *o == id)
    });
    let triggered =
        t.g.turn_events.iter().filter(
            |e| matches!(e, Event::AbilityTriggeredOnStack { source, .. } if *source == id),
        );
    (crewed.count(), triggered.count())
}

fn revealed(t: &TestGame, id: ObjectId) -> bool {
    t.g.turn_events.iter().any(|e| {
        matches!(e, Event::Custom { name, obj: Some(o), .. }
            if name == facedown::REVEALED && *o == id)
    })
}

#[test]
fn tezzeret_cards_enter_as_5_5_artifact_creatures() {
    cr!("708.2", "708.3", "603.6a");
    ruling!(
        "Tezzeret, Cruel Machinist",
        "Cards put onto the battlefield using Tezzeret's third ability enter the battlefield as 5/5 artifact creatures. They don't enter the battlefield with their own characteristics and then become 5/5."
    );
    let mut t = TestGame::new(2);
    // "Whenever an artifact you control enters, this creature deals 1 damage to each
    // opponent." / "Whenever another creature you control with power 3 or greater enters,
    // you may draw a card."
    add(&mut t, P0, "Reckless Fireweaver");
    add(&mut t, P0, "Garruk's Packleader");
    let hand = t.hand_size(P0);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    let ids = tezzeret_ultimate(&mut t, &["Grizzly Bears", "Forest", "Lightning Bolt"]);
    for id in &ids {
        assert_face_down_5_5(&t, *id);
    }
    // Three artifacts entered; three creatures with power 5 entered.
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.hand_size(P0), hand + 3);
    // No land was played or entered: P0 may still play a land.
    assert_eq!(t.g.players[0].lands_played_this_turn, 0);
}

#[test]
fn a_copy_of_a_tezzeret_face_down_creature_is_a_face_up_5_5_artifact_creature() {
    cr!("707.2", "708.2");
    ruling!(
        "Tezzeret, Cruel Machinist",
        "If a creature enters the battlefield as a copy of a face-down creature or if a token is created that's a copy of one, that copy has the same characteristics as the face-down creature (in this case, a 5/5 artifact creature with no other characteristics), even though the copy is face up."
    );
    let mut t = TestGame::new(2);
    let ids = tezzeret_ultimate(&mut t, &["Serra Angel"]);
    let clone = enter_copying(&mut t, P0, "Clone", Some(ids[0]));
    let o = t.obj_now(clone);
    assert!(!o.face_down);
    assert_eq!(o.chars.name, "");
    assert_eq!(t.pt(clone), (5, 5));
    assert!(o.is(CardType::Artifact) && o.is(CardType::Creature));
    assert!(o.chars.abilities.is_empty(), "no flying or vigilance");
}

#[test]
fn a_double_faced_card_put_face_down_isnt_revealed_and_has_no_back_face() {
    cr!("708.2", "712.15");
    ruling!(
        "Tezzeret, Cruel Machinist",
        "If a double-faced card (such as Nicol Bolas, the Ravager) is put onto the battlefield face down, you don't reveal it as you do so. The characteristics of its back face don't apply."
    );
    let mut t = TestGame::new(2);
    let ids = tezzeret_ultimate(&mut t, &["Delver of Secrets // Insectile Aberration"]);
    assert_face_down_5_5(&t, ids[0]);
    assert!(!revealed(&t, ids[0]));
    assert!(!t.obj_now(ids[0]).has_keyword(keywords::KeywordKind::Flying));
}

#[test]
fn a_face_down_permanent_leaving_the_battlefield_is_revealed() {
    cr!("708.9");
    ruling!(
        "Tezzeret, Cruel Machinist",
        "If a face-down permanent you control leaves the battlefield, you must reveal it to all players."
    );
    let mut t = TestGame::new(2);
    let ids = tezzeret_ultimate(&mut t, &["Grizzly Bears", "Hill Giant"]);
    assert!(!revealed(&t, ids[0]));
    kill(&mut t, ids[0]);
    assert!(revealed(&t, ids[0]));
    assert!(!revealed(&t, ids[1]), "the one that stayed isn't revealed");
}

#[test]
fn a_returned_face_down_card_returns_face_up_and_an_instant_stays_in_exile() {
    cr!("400.7", "400.4a");
    ruling!(
        "Tezzeret, Cruel Machinist",
        "If an effect tries to return a face-down creature to the battlefield after it leaves (such as Oath of Teferi from the Dominaria set or Abnormal Endurance from this set), that effect returns the card face up. If it tries to put an instant or sorcery card onto the battlefield this way, that card remains in its current zone instead."
    );
    supported("Cloudshift");
    let mut t = TestGame::new(2);
    let ids = tezzeret_ultimate(&mut t, &["Grizzly Bears", "Lightning Bolt"]);
    // Cloudshift: "Exile target creature you control, then return that card to the
    // battlefield under your control."
    cast_resolve(&mut t, P0, "Cloudshift", "W", &[obj(ids[0])]);
    let bears = t.g.current(ids[0]);
    assert!(t.on_battlefield(bears));
    assert!(!t.obj_now(bears).face_down);
    assert_eq!(t.obj_now(bears).chars.name, "Grizzly Bears");
    assert_eq!(t.pt(bears), (2, 2));
    assert!(!is_type(&t, bears, CardType::Artifact));
    cast_resolve(&mut t, P0, "Cloudshift", "W", &[obj(ids[1])]);
    assert_eq!(t.zone(ids[1]), object::Zone::Exile);
    assert!(t.in_exile("Lightning Bolt"));
}

#[test]
fn turning_a_tezzeret_card_face_up_ends_the_5_5_effect_but_an_instant_stays_face_down() {
    cr!("708.2", "708.8", "110.4");
    // The CR says this only for manifested, cloaked and merged permanents (701.40g,
    // 701.58g, 730.2g); Tezzeret's face-down cards follow the same rule.
    interp!("face-down-instant-or-sorcery-stays-face-down");
    ruling!(
        "Tezzeret, Cruel Machinist",
        "If an effect from cards outside of this set does turn one face up, the effect causing it to be a 5/5 artifact creature ends. If such an effect tries to turn an instant or sorcery card face up, reveal that card and it stays face down. In that case, it remains a 5/5 artifact creature."
    );
    supported("Break Open");
    let mut t = TestGame::new(2);
    let ids = tezzeret_ultimate(&mut t, &["Grizzly Bears", "Lightning Bolt"]);
    // P1's Break Open: "Turn target face-down creature an opponent controls face up."
    for id in &ids {
        cast_resolve(&mut t, P1, "Break Open", "1R", &[obj(*id)]);
    }
    let bears = t.g.current(ids[0]);
    assert!(!t.obj_now(bears).face_down);
    assert_eq!(t.obj_now(bears).chars.name, "Grizzly Bears");
    assert_eq!(t.pt(bears), (2, 2));
    assert!(!is_type(&t, bears, CardType::Artifact));
    assert!(revealed(&t, ids[1]));
    assert_face_down_5_5(&t, ids[1]);
}

#[test]
fn you_may_look_at_face_down_permanents_you_control() {
    cr!("708.5");
    ruling!(
        "Tezzeret, Cruel Machinist",
        "You may look at face-down permanents you control at any time."
    );
    ruling!(
        "Magar of the Magic Strings",
        "You may look at face-down permanents you control at any time."
    );
    let mut t = TestGame::new(2);
    let ids = tezzeret_ultimate(&mut t, &["Grizzly Bears"]);
    assert!(facedown::can_look_at(&t.g, P0, ids[0]));
    assert!(!facedown::can_look_at(&t.g, P1, ids[0]));
    // Magar: "{1}{B}{R}: Note the name of target instant or sorcery card in your graveyard
    // and put it onto the battlefield face down. It's a 3/3 creature with ..."
    supported("Magar of the Magic Strings");
    let magar = t.battlefield(P0, "Magar of the Magic Strings");
    let bolt = t.graveyard(P0, "Lightning Bolt");
    activate_resolve(&mut t, P0, magar, "Note the name", "1BR", &[obj(bolt)]);
    let fd = t.g.current(bolt);
    assert!(t.on_battlefield(fd) && t.obj_now(fd).face_down);
    assert_eq!(t.pt(fd), (3, 3));
    assert!(facedown::can_look_at(&t.g, P0, fd));
    assert!(!facedown::can_look_at(&t.g, P1, fd));
}

#[test]
fn depthshaker_titan_doesnt_crew_and_its_3_3_survives_crew_and_station() {
    cr!("702.122a", "702.122e", "721.2b", "613.4b", "613.7a", "613.7b");
    ruling!(
        "Depthshaker Titan",
        "If Depthshaker Titan’s first ability causes a Vehicle to become an artifact creature, it doesn’t count as “crewing” that Vehicle for any ability that would trigger due to a Vehicle becoming crewed."
    );
    ruling!(
        "Depthshaker Titan",
        "If Depthshaker Titan’s first ability causes a Vehicle to become an artifact creature, its base power and toughness will be set to 3/3. Crewing that Vehicle will not restore its power and toughness. Similarly, if the ability causes a permanent with station to become an artifact creature, putting an amount of charge counters on it greater than or equal to the amount required for it to become an artifact creature won’t overwrite the base power and toughness set by Depthshaker Titan’s first ability."
    );
    supported("Mobilizer Mech");
    supported("Uthros Scanship");
    let mut t = TestGame::new(2);
    // Mobilizer Mech (3/4, crew 3): "Whenever this Vehicle becomes crewed, up to one
    // other target Vehicle you control becomes an artifact creature until end of turn."
    // Uthros Scanship: a 4/4 Spacecraft, an artifact creature at 8+ charge counters.
    let mech = t.battlefield(P0, "Mobilizer Mech");
    let ship = t.battlefield(P0, "Uthros Scanship");
    let copter = t.battlefield(P0, "Smuggler's Copter");
    t.answer_targets(P0, &[obj(mech), obj(ship)]);
    enter(&mut t, P0, "Depthshaker Titan");
    t.resolve_all();
    assert_eq!(t.pt(mech), (3, 3));
    assert_eq!(t.pt(ship), (3, 3));
    // The Mech didn't become crewed, so its trigger didn't trigger: the Copter isn't a
    // creature.
    assert_eq!(
        crewed_and_triggered(&t, mech),
        (0, 0),
        "becoming a creature isn't crewing"
    );
    assert!(!is_creature(&t, copter));
    // Crewing it (with Hill Giant): it becomes crewed and animates the Copter, and it's
    // still 3/3.
    let giant = t.battlefield(P0, "Hill Giant");
    assert!(crew(&mut t, P0, mech, &[giant]));
    t.answer_targets(P0, &[obj(copter)]);
    t.resolve_all();
    assert_eq!(crewed_and_triggered(&t, mech), (1, 1));
    assert!(is_creature(&t, copter), "control: crewing triggers it");
    assert_eq!(t.pt(mech), (3, 3));
    // Eight charge counters on the Scanship: its base power and toughness stay 3/3.
    put(&mut t, ship, counters::CHARGE, 8);
    assert!(
        t.obj_now(ship).has_keyword(keywords::KeywordKind::Flying),
        "8+ applies"
    );
    assert_eq!(t.pt(ship), (3, 3));
}
