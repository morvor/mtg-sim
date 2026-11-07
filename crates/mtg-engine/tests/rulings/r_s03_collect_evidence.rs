//! Rulings batch S03 — collect evidence (CR 701.59): "To collect evidence N, exile cards
//! with total mana value N or greater from your graveyard."

use crate::r_s01_common::*;
use mtg_engine::decision::{Action, Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn opponents_cant_act_while_a_spell_that_collects_evidence_is_being_cast() {
    cr!("601.2", "601.2h", "601.2i", "117.3c", "701.59a");
    ruling!(
        "Detective's Phoenix",
        "Once you've announced that you're casting a spell, players can't take actions until you've finished doing so. Notably, opponents can't try to remove cards from your graveyard to stop you from collecting evidence."
    );
    supported("Detective's Phoenix");
    // Detective's Phoenix: "Bestow—{R}, Collect evidence 6." P0 casts it bestowed from
    // hand, in the game's priority loop, exiling Hill Giant (4) and Grizzly Bears (2) from
    // the graveyard.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Mountain", 1);
    let giant = t.graveyard(P0, "Hill Giant");
    let dead_bears = t.graveyard(P0, "Grizzly Bears");
    let phoenix = t.hand(P0, "Detective's Phoenix");
    // P1 holds priority the first time it gets it; watch the game state at each of P1's
    // decisions.
    let seen = watch(
        &mut t,
        P1,
        |_| true,
        |g| (g.player(P0).graveyard.len(), g.stack.len(), g.exile.len()),
    );
    let from = t.asked().len();
    t.answer(
        P0,
        DecisionKind::Priority,
        Answer::Action(Action::Cast {
            card: phoenix,
            method: CastMethod::Keyword(KeywordKind::Bestow),
        }),
    );
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.answer_choose(P0, &[Entity::Object(giant), Entity::Object(dead_bears)]);
    let ok = t.g.run_until(1_000, |g| {
        !g.find_in_zone(Zone::Battlefield, "Detective's Phoenix")
            .is_empty()
    });
    assert!(ok);
    // From the moment P0 announced the spell until it was cast, only P0 was asked
    // anything; P0 got priority first afterward (CR 117.3c), then P1.
    let asked = t.asked()[from..].to_vec();
    let first_p1 = asked.iter().position(|(p, _)| *p == P1).unwrap();
    assert!(first_p1 > 1);
    assert!(asked[..first_p1].iter().all(|(p, _)| *p == P0));
    assert!(matches!(
        asked[first_p1 - 1],
        (_, Decision::Priority { .. })
    ));
    // The first time P1 could act, the evidence was already exiled and the Phoenix spell
    // was on the stack.
    let seen = seen.lock().unwrap().clone();
    assert_eq!(seen[0], (0, 1, 2));
    assert_eq!(t.pt(bears), (4, 4));
}
