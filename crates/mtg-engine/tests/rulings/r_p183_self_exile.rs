//! Rulings batch P183 — "When ~ dies, [you may] exile it, then/if you do, return target
//! ... card from your graveyard": the dying card may be one of the targets, but "exile it"
//! makes it a new object, which the target no longer refers to (CR 400.7, 608.2b). A
//! regression test for the engine following a target to the exiled card.

use crate::r_p009_common::obj;
use crate::r_s01_common::supported;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

/// P0's `name` dies with `others` (real cards) in P0's graveyard; its trigger targets its
/// own card and the others, and P0 agrees to exile it. Returns (the dead card, the
/// others).
fn dies_targeting_itself(name: &str, others: &[&str]) -> (TestGame, ObjectId, Vec<ObjectId>) {
    supported(name);
    let mut t = TestGame::new(2);
    let src = t.battlefield(P0, name);
    let others: Vec<ObjectId> = others.iter().map(|n| t.graveyard(P0, n)).collect();
    t.g.destroy(src, None);
    let card = t.g.current(src);
    assert_eq!(t.g.obj(card).zone, Zone::Graveyard(P0));
    let mut targets = vec![obj(card)];
    targets.extend(others.iter().map(|o| obj(*o)));
    t.answer_targets(P0, &targets);
    t.answer_yes(P0, true);
    t.g.flush_events();
    t.settle();
    assert_eq!(t.stack_len(), 1, "{name}");
    t.resolve_all();
    (t, card, others)
}

#[test]
fn a_card_that_exiles_itself_isnt_returned_as_its_own_target() {
    cr!("400.7", "608.2b");
    ruling!(
        "Iname as One",
        "Iname can target itself with the graveyard-triggered ability, but it will be exiled and the ability won't be able to return Iname."
    );
    ruling!(
        "Iname, Life Aspect",
        "You may target Iname, Life Aspect. However, this doesn't do anything since the card will no longer be in your graveyard when the creature cards are returned to your hand."
    );
    ruling!(
        "Body Snatcher",
        "Although you can choose Body Snatcher itself as the target of its last ability, it will be exiled before it can be returned to the battlefield."
    );
    // (the card, other targets, where the others go)
    let cases: &[(&str, &[&str], Option<Zone>)] = &[
        ("Iname as One", &[], None),
        ("Iname, Life Aspect", &["Spectral Sailor"], Some(Zone::Hand(P0))),
        ("Body Snatcher", &[], None),
        ("Greenwarden of Murasa", &[], None),
        ("Celestial Gatekeeper", &["Bishop of Wings"], Some(Zone::Battlefield)),
    ];
    for (name, others, to) in cases {
        let (t, card, others) = dies_targeting_itself(name, others);
        assert_eq!(t.zone(card), Zone::Exile, "{name}");
        for o in others {
            assert_eq!(Some(t.zone(o)), *to, "{name}");
        }
    }
}
