//! CR 608.2c: an ability that exiles a card and then refers to "the exiled card" means the
//! card it exiled as it resolved (Nexus of Becoming), unlike an ability linked to another
//! ability that exiled cards, which refers to every card that one exiled (CR 607.2a,
//! 607.3; Phantom Steed).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// The names of the tokens P0 controls, sorted.
fn p0_token_names(t: &TestGame) -> Vec<String> {
    let mut v: Vec<String> = t
        .g
        .permanents()
        .filter(|o| o.controller == P0 && o.is_token())
        .map(|o| o.chars.name.to_string())
        .collect();
    v.sort();
    v
}

/// P0 exiles `name` from their hand to Nexus of Becoming's trigger at the beginning of
/// combat on their turn.
fn nexus_exiles(t: &mut TestGame, name: &str) -> ObjectId {
    let card = t.hand(P0, name);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(card)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    card
}

#[test]
fn nexus_of_becoming_copies_only_the_card_it_just_exiled() {
    cr!("608.2c");
    // "At the beginning of combat on your turn, draw a card. Then you may exile an artifact
    // or creature card from your hand. If you do, create a token that's a copy of the
    // exiled card, except it's a 3/3 Golem artifact creature in addition to its other
    // types."
    let def = mtg_engine::card::card("Nexus of Becoming");
    assert!(def.unsupported_text().is_empty());
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Nexus of Becoming");
    let bears = nexus_exiles(&mut t, "Grizzly Bears");
    assert_eq!(t.zone(bears), mtg_engine::object::Zone::Exile);
    assert_eq!(p0_token_names(&t), vec!["Grizzly Bears"]);
    let token = t
        .g
        .permanents()
        .find(|o| o.controller == P0 && o.is_token())
        .map(|o| o.id)
        .unwrap();
    assert_eq!(t.pt(token), (3, 3));
    assert!(t.obj_now(token).is(CardType::Artifact));
    assert!(t.obj_now(token).chars.has_subtype("Golem"));
    assert!(t.obj_now(token).chars.has_subtype("Bear"));
    // On P0's next turn, the Bears are still in exile: only the Hill Giant exiled then is
    // copied.
    t.advance_to(P1, Step::Upkeep);
    nexus_exiles(&mut t, "Hill Giant");
    assert_eq!(p0_token_names(&t), vec!["Grizzly Bears", "Hill Giant"]);
}
