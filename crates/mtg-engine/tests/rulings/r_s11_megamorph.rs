//! Rulings batch S11 — megamorph (CR 702.37b): a variant of morph whose cost, paid to turn
//! the permanent face up, also puts a +1/+1 counter on it.

use crate::r_s01_common::*;
use crate::r_s04_common::untapped_lands;
use crate::r_s11_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

const FACE_DOWN: CastMethod = CastMethod::FaceDown(KeywordKind::Morph);

/// `p` casts `name` face down (paying {3} with Wastes) and it resolves.
fn cast_face_down(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    t.lands(p, "Wastes", 3);
    let card = t.hand(p, name);
    let spell = t.cast(p, card).method(FACE_DOWN).go();
    assert!(t.obj(spell).face_down);
    t.resolve_all();
    let id = t.g.current(spell);
    assert!(t.obj(id).face_down && t.on_battlefield(id));
    id
}

#[test]
fn megamorph_is_a_variant_of_morph() {
    cr!("702.37a", "702.37b", "702.37e");
    ruling!(
        "Gudul Lurker",
        "Megamorph is a variant of the morph ability."
    );
    supported("Gudul Lurker");
    // Gudul Lurker: {U} 1/1 "can't be blocked", megamorph {U}. Cast face down for {3}
    // like a morph: a 2/2 with no abilities.
    let mut t = TestGame::new(2);
    let lurker = cast_face_down(&mut t, P0, "Gudul Lurker");
    assert!(is_plain_face_down_2_2(&t, lurker));
    // Turned face up for its megamorph cost, with a +1/+1 counter.
    t.lands(P0, "Island", 1);
    assert!(turn_face_up(&mut t, P0, lurker));
    assert_eq!(t.obj(lurker).chars.name.as_str(), "Gudul Lurker");
    assert_eq!(t.counters(lurker, counters::PLUS1), 1);
    assert_eq!(t.pt(lurker), (2, 2));
}

#[test]
fn a_manifested_megamorph_card_turned_up_for_its_mana_cost_gets_no_counter() {
    cr!("701.40b", "701.40c", "702.37b");
    ruling!(
        "Aven Sunstriker",
        "If a face-down creature with megamorph is turned face up some other way (for example, if you manifest a card with megamorph and then pay its mana cost to turn it face up), you won’t put a +1/+1 counter on it."
    );
    supported("Aven Sunstriker");
    // Aven Sunstriker: {1}{W}{W} 1/1 flying, double strike, megamorph {4}{W}.
    let mut t = TestGame::new(2);
    let m = manifest_card(&mut t, P0, "Aven Sunstriker");
    t.lands(P0, "Plains", 3);
    assert!(turn_face_up(&mut t, P0, m));
    assert_eq!(t.obj(m).chars.name.as_str(), "Aven Sunstriker");
    assert_eq!(t.counters(m, counters::PLUS1), 0);
    assert_eq!(t.pt(m), (1, 1));
    // Turned up for its megamorph cost instead, it gets the counter.
    let mut t = TestGame::new(2);
    let m = manifest_card(&mut t, P0, "Aven Sunstriker");
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Wastes", 4);
    assert!(turn_face_up(&mut t, P0, m));
    assert_eq!(t.counters(m, counters::PLUS1), 1);
    assert_eq!(t.pt(m), (2, 2));
}

#[test]
fn a_manifested_den_protector_turned_up_for_its_mana_cost_gets_no_counter() {
    cr!("701.40c", "702.37b");
    ruling!(
        "Den Protector",
        "If a face-down creature with megamorph is turned face up some other way (for example, if you manifest a card with megamorph and then pay its mana cost to turn it face up), you won't put a +1/+1 counter on it."
    );
    supported("Den Protector");
    // Den Protector: {1}{G} 2/1, megamorph {1}{G}: its controller chooses which cost.
    for (choice, counter) in [(0, 0), (1, 1)] {
        let mut t = TestGame::new(2);
        let m = manifest_card(&mut t, P0, "Den Protector");
        t.lands(P0, "Forest", 2);
        t.answer(P0, DecisionKind::Option, Answer::Index(choice));
        assert!(turn_face_up(&mut t, P0, m));
        assert_eq!(t.obj(m).chars.name.as_str(), "Den Protector");
        assert_eq!(t.counters(m, counters::PLUS1), counter);
    }
}

#[test]
fn turning_a_megamorph_face_up_with_its_counter_doesnt_use_the_stack() {
    cr!("702.37b", "702.37e", "116.2b");
    ruling!(
        "Den Protector",
        "Turning a face-down creature with megamorph face up and putting a +1/+1 counter on it is a special action. It doesn't use the stack and can't be responded to."
    );
    let mut t = TestGame::new(2);
    let den = cast_face_down(&mut t, P0, "Den Protector");
    let bolt = t.graveyard(P0, "Lightning Bolt");
    t.lands(P0, "Forest", 2);
    let from = t.asked().len();
    assert!(turn_face_up(&mut t, P0, den));
    // Face up with its counter at once: no one got priority in between.
    assert!(!t.obj(den).face_down);
    assert_eq!(t.counters(den, counters::PLUS1), 1);
    assert_eq!(t.pt(den), (3, 2));
    assert!(asked_since(&t, from)
        .iter()
        .all(|(_, d)| !matches!(d, mtg_engine::decision::Decision::Priority { .. })));
    assert_eq!(untapped_lands(&t, P0), 0);
    // Only its trigger ("When this creature is turned face up, return target card from
    // your graveyard to your hand.") uses the stack.
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert!(t.in_hand(P0, "Lightning Bolt"));
    assert!(!t.g.is_live(bolt));
}
