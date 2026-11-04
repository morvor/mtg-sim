//! roundtrip-pairs-5 (docs/agents/items/roundtrip-pairs-5.md): untriaged round-trip
//! mismatches (the 3+-card `gap:` clusters, two-card clusters and the single-card tail).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn the_deck_of_many_things_creature_owner_loses_when_it_dies() {
    cr!("603.7a", "104.3e", "108.3");
    // "20 | Put a creature card from any graveyard onto the battlefield under your control.
    // When that creature dies, its owner loses the game." "Its owner" is the owner of the
    // creature that died (the delayed ability has no targets): P1's Craw Wurm, so P1 loses.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let deck = t.battlefield(P0, "The Deck of Many Things");
    t.lands(P0, "Plains", 2);
    let wurm = t.graveyard(P1, "Craw Wurm");
    t.g.dice.loaded.push_back(20);
    t.answer_choose(P0, &[Entity::Object(wurm)]);
    t.activate(P0, deck, 0, &[]).unwrap();
    t.resolve_all();
    let wurms = t.named_on_battlefield("Craw Wurm");
    assert_eq!(wurms.len(), 1);
    t.g.destroy(wurms[0], None);
    t.settle();
    t.resolve_all();
    assert!(t.has_lost(P1), "the Wurm's owner loses");
    assert!(!t.has_lost(P0));
}

/// Puts the real Aura or Equipment `name` onto the battlefield under `p`'s control attached
/// to `to`.
fn attach_new(t: &mut TestGame, p: PlayerId, name: &str, to: ObjectId) -> ObjectId {
    let id = t.battlefield(p, name);
    assert!(t.g.attach(id, Entity::Object(to)), "{name} couldn't be attached");
    t.g.recompute();
    id
}

#[test]
fn pact_weapon_reveals_the_drawn_card_and_uses_its_mana_value() {
    cr!("508.1m", "603.2");
    // "Whenever equipped creature attacks, draw a card and reveal it. The creature gets
    // +X/+X until end of turn and you lose X life, where X is that card's mana value."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Pact Weapon", bears);
    t.library_top(P0, "Craw Wurm");
    t.set_step(P0, Step::PrecombatMain);
    let life = t.life(P0);
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(bears, Entity::Player(P1))]),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    t.resolve_all();
    assert!(t.in_hand(P0, "Craw Wurm"));
    assert_eq!(t.pt(bears), (8, 8), "Craw Wurm's mana value is 6");
    assert_eq!(t.life(P0), life - 6);
}

#[test]
fn pact_weapon_gives_no_bonus_when_the_draw_is_replaced_by_draws() {
    cr!("614.1a", "614.11");
    ruling!(
        "Pact Weapon",
        "This is true even if the card draw is replaced by one or more card draws."
    );
    // Thought Reflection: "If you would draw a card, draw two cards instead." The card
    // draw was replaced, so no card is "that card": no bonus, no life lost.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Pact Weapon", bears);
    t.battlefield(P0, "Thought Reflection");
    t.library_top(P0, "Craw Wurm");
    t.library_top(P0, "Craw Wurm");
    t.set_step(P0, Step::PrecombatMain);
    let life = t.life(P0);
    let hand = t.hand_size(P0);
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(bears, Entity::Player(P1))]),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2, "two cards drawn instead");
    assert_eq!(t.pt(bears), (2, 2));
    assert_eq!(t.life(P0), life);
}
