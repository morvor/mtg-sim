//! Rulings batch P220 — partner (CR 702.124): Amy Pond's two partner abilities.

use mtg_engine::card::card;
use mtg_engine::kw::partner::commanders_problem;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn amy_pond_uses_one_partner_ability_but_still_searches_for_rory() {
    cr!("702.124g", "702.124j");
    ruling!(
        "Amy Pond",
        "Amy must choose one of the two. Even if your other commander is one of the Doctor cards, however, the \"Partner with Rory Williams\" ability can still search for Rory in a library."
    );
    let amy = card("Amy Pond");
    let rory = card("Rory Williams");
    let doctor = card("The Tenth Doctor");
    // Either partner ability on its own...
    assert!(commanders_problem(&[&amy, &rory]).is_none());
    assert!(commanders_problem(&[&amy, &doctor]).is_none());
    // ...but not both at once.
    assert!(commanders_problem(&[&amy, &rory, &doctor]).is_some());

    // With the Doctor as the other commander, Amy's "partner with" trigger still finds
    // Rory in a library.
    let mut t = TestGame::new(2);
    for name in ["Amy Pond", "The Tenth Doctor"] {
        t.g.players[P0.idx()].commander_names.push(name.into());
    }
    let doc = t.command(P0, "The Tenth Doctor");
    t.g.objects[doc.0 as usize].is_commander = true;
    t.library_top(P0, "Rory Williams");
    for _ in 0..3 {
        t.library_top(P0, "Island");
    }
    t.lands(P0, "Mountain", 3);
    let amy = t.hand(P0, "Amy Pond");
    t.g.objects[amy.0 as usize].is_commander = true;
    t.cast(P0, amy).go();
    t.answer_targets(P0, &[Entity::Player(P0)]);
    t.resolve_all();
    assert!(t.in_hand(P0, "Rory Williams"));
}
