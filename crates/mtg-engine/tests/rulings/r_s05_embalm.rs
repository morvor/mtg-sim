//! Rulings batch S05 — embalm (CR 702.128): "[Cost], Exile this card from your graveyard:
//! Create a token that's a copy of it, except it's a white Zombie [types] with no mana
//! cost. Embalm only as a sorcery."

use crate::r_s01_common::*;
use crate::r_s02_common::*;
use crate::r_s04_common::*;
use crate::r_s05_common::*;
use mtg_engine::decision::Decision;
use mtg_engine::game::Game;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn the_embalmed_token_copies_only_the_printed_card() {
    cr!("702.128a", "707.2");
    ruling!(
        "Sacred Cat",
        "The token copies exactly what was printed on the original card and nothing else. It doesn't copy any information about the object the card was before it was put into your graveyard."
    );
    supported("Sacred Cat");
    supported("Rancor");
    // Sacred Cat (1/1 lifelink) with two +1/+1 counters, Rancor and a Giant Growth dies.
    let mut t = TestGame::new(2);
    let cat = t.battlefield(P0, "Sacred Cat");
    t.g.add_counters(Entity::Object(cat), counters::PLUS1, 2, None);
    t.lands(P0, "Forest", 2);
    let rancor = t.hand(P0, "Rancor");
    t.cast(P0, rancor).target(cat).go();
    t.resolve_all();
    let gg = t.hand(P0, "Giant Growth");
    t.cast(P0, gg).target(cat).go();
    t.resolve_all();
    assert_eq!(t.pt(cat), (8, 6));
    destroy(&mut t, cat);
    t.resolve_all();
    let card = t.g.current(cat);
    assert_eq!(t.zone(card), Zone::Graveyard(P0));
    // Embalm {W}: the token is a plain 1/1 white Zombie Cat with lifelink.
    t.lands(P0, "Plains", 1);
    t.activate(P0, card, 0, &[]).unwrap();
    t.resolve_all();
    let token = tokens_with_subtype(&t, P0, "Cat");
    assert_eq!(token.len(), 1);
    let o = t.obj_now(token[0]);
    assert_eq!(t.pt(token[0]), (1, 1));
    assert_eq!(o.counter(counters::PLUS1), 0);
    assert!(!o.chars.has_keyword(KeywordKind::Trample));
    assert!(o.chars.has_keyword(KeywordKind::Lifelink));
    assert_eq!(o.chars.colors, ColorSet::single(Color::White));
    assert!(o.chars.has_subtype("Zombie"));
}

#[test]
fn the_card_is_exiled_as_embalm_is_activated() {
    cr!("702.128a", "602.2b", "601.2h");
    ruling!(
        "Sacred Cat",
        "Once you've activated an embalm ability, the card is immediately exiled. Opponents can't try to stop the ability by exiling the card with an effect such as that of Crook of Condemnation."
    );
    supported("Crook of Condemnation");
    // Crook of Condemnation: "{1}, {T}: Exile target card from a graveyard."
    let mut t = TestGame::new(2);
    let crook = t.battlefield(P1, "Crook of Condemnation");
    let cat = t.graveyard(P0, "Sacred Cat");
    let other = t.graveyard(P0, "Grizzly Bears");
    assert!(ability_targets(&mut t, crook, 0).contains(&Entity::Object(cat)));
    t.lands(P0, "Plains", 1);
    t.activate(P0, cat, 0, &[]).unwrap();
    // The ability is on the stack and the card is already in exile: it can't be targeted.
    assert_eq!(t.stack_len(), 1);
    assert!(t.in_exile("Sacred Cat"));
    assert_eq!(
        ability_targets(&mut t, crook, 0),
        vec![Entity::Object(other)]
    );
    t.resolve_all();
    assert_eq!(tokens_with_subtype(&t, P0, "Cat").len(), 1);
}

#[test]
fn you_get_priority_to_embalm_right_after_the_card_is_put_into_your_graveyard() {
    cr!("117.3b", "702.128a");
    ruling!(
        "Sacred Cat",
        "If a spell or ability puts a creature card with embalm into your graveyard during your main phase, you'll have priority immediately after that spell or ability resolves."
    );
    supported("Murder");
    // P0 casts Murder on their own Sacred Cat in their main phase. After it resolves, P0
    // gets priority before P1 does, and can embalm the Cat.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Crook of Condemnation");
    t.lands(P1, "Wastes", 1);
    let cat = t.battlefield(P0, "Sacred Cat");
    give_mana_for(&mut t, P0, "Murder");
    let murder = t.hand(P0, "Murder");
    t.cast(P0, murder).target(cat).go();
    let in_graveyard = |g: &Game| {
        g.player(P0)
            .graveyard
            .iter()
            .any(|c| g.obj(*c).chars.name.as_str() == "Sacred Cat")
    };
    let is_priority = |d: &Decision| matches!(d, Decision::Priority { .. });
    let seen1 = watch(&mut t, P1, is_priority, in_graveyard);
    // Run until someone is asked for priority with the Cat in the graveyard.
    let ok = t.g.run_until(1000, |g| {
        in_graveyard(g) && g.stack.is_empty() && g.turn.priority.is_some()
    });
    assert!(ok);
    assert_eq!(t.g.turn.priority, Some(P0));
    assert!(!seen1.lock().unwrap().iter().any(|x| *x));
    let card = t.g.current(cat);
    t.lands(P0, "Plains", 1);
    assert!(can_activate(&mut t, P0, card));
    t.activate(P0, card, 0, &[]).unwrap();
    assert!(t.in_exile("Sacred Cat"));
    t.resolve_all();
    assert_eq!(tokens_with_subtype(&t, P0, "Cat").len(), 1);
}
