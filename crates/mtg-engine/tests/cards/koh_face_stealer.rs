//! Koh, the Face Stealer: "Pay 1 life: Choose a creature card exiled with Koh." and "Koh
//! has all activated and triggered abilities of the last chosen card." The choice is
//! noted for the linked ability (`Effect::NoteLinked`), which the Oracle round-trip
//! renderer used to put into no words at all ("Pay 1 life: ."); in game Koh uses the
//! chosen card's abilities.

use mtg_engine::ability::AbilityKind;
use mtg_engine::card::card;
use mtg_engine::object::Zone;
use mtg_engine::oracle::render::compare::check_card;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn koh_round_trips_with_its_choice_worded() {
    cr!("607.2e");
    let c = card("Koh, the Face Stealer");
    assert!(c.unsupported_text().is_empty(), "{:?}", c.unsupported_text());
    let r = check_card(&c);
    assert!(
        r.pass,
        "oracle: {:?}\nrendered: {:?}\ngaps: {:?}",
        r.unmatched_oracle, r.unmatched_rendered, r.gaps
    );
}

/// The position of Koh's activated ability whose text contains `text`.
fn activated_index(t: &mut TestGame, koh: ObjectId, text: &str) -> Option<usize> {
    t.g.recompute();
    t.g.obj(koh)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, AbilityKind::Activated(_)))
        .position(|a| a.text.contains(text))
}

#[test]
fn koh_uses_the_activated_and_triggered_abilities_of_the_chosen_card() {
    cr!("607.2e", "613.1f", "602.2");
    let mut t = TestGame::new(2);
    let koh = t.battlefield(P0, "Koh, the Face Stealer");
    let sorcerer = t.battlefield(P1, "Prodigal Sorcerer");
    let warden = t.battlefield(P1, "Soul Warden");
    for c in [sorcerer, warden] {
        t.g.destroy(c, None);
        t.answer_yes(P0, true);
        t.resolve_all();
    }
    let exiled = |t: &TestGame, name: &str| t.g.find_in_zone(Zone::Exile, name)[0];
    // Nothing chosen yet: no "deals 1 damage" ability.
    assert_eq!(activated_index(&mut t, koh, "deals 1 damage"), None);

    // Choose the Sorcerer (paying 1 life), then use its ability: Koh deals the damage.
    let s = exiled(&t, "Prodigal Sorcerer");
    t.answer_choose(P0, &[Entity::Object(s)]);
    t.activate(P0, koh, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P0), 19);
    let i = activated_index(&mut t, koh, "deals 1 damage").expect("the Sorcerer's ability");
    t.activate(P0, koh, i, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    assert!(t.g.obj(koh).tapped);

    // Choose the Warden instead: Koh triggers when another creature enters, and no longer
    // has the Sorcerer's ability.
    let w = exiled(&t, "Soul Warden");
    t.answer_choose(P0, &[Entity::Object(w)]);
    t.activate(P0, koh, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P0), 18);
    assert_eq!(activated_index(&mut t, koh, "deals 1 damage"), None);
    t.enter(P1, "Grizzly Bears");
    t.resolve_all();
    assert_eq!(t.life(P0), 19, "Soul Warden's trigger, on Koh");
}
