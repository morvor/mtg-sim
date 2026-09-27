//! Rulings batch S07 — eternalize (CR 702.129): "[Cost], Exile this card from your
//! graveyard: Create a token that's a copy of it, except it's a 4/4 black Zombie [types]
//! with no mana cost. Eternalize only as a sorcery."

use crate::r_s01_common::*;
use crate::r_s02_common::*;
use crate::r_s04_common::*;
use crate::r_s05_common::*;
use crate::r_s07_common::*;
use mtg_engine::game::Game;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// Activates the eternalize ability of the card `card` in `p`'s graveyard.
fn eternalize(t: &mut TestGame, p: PlayerId, card: ObjectId) {
    activate_named(t, p, card, "Eternalize", 0).expect("eternalize");
}

#[test]
fn the_eternalized_token_copies_only_the_printed_card() {
    cr!("702.129a", "707.2", "707.9b");
    ruling!(
        "Adorned Pouncer",
        "The token copies exactly what was printed on the original card and nothing else, except the characteristics specifically modified by eternalize. It doesn't copy any information about the object the card was before it was put into your graveyard."
    );
    supported("Adorned Pouncer");
    supported("Rancor");
    // Adorned Pouncer (1/1 double strike Cat) with two +1/+1 counters, Rancor and a Giant
    // Growth dies.
    let mut t = TestGame::new(2);
    let cat = t.battlefield(P0, "Adorned Pouncer");
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
    // Eternalize {3}{W}{W}: the token is a plain 4/4 black Zombie Cat with double strike.
    t.lands(P0, "Plains", 5);
    eternalize(&mut t, P0, card);
    t.resolve_all();
    let token = tokens_with_subtype(&t, P0, "Cat");
    assert_eq!(token.len(), 1);
    let o = t.obj_now(token[0]);
    assert_eq!(t.pt(token[0]), (4, 4));
    assert_eq!(o.counter(counters::PLUS1), 0);
    assert!(!o.chars.has_keyword(KeywordKind::Trample));
    assert!(o.chars.has_keyword(KeywordKind::DoubleStrike));
    assert_eq!(o.chars.colors, ColorSet::single(Color::Black));
}

#[test]
fn the_token_is_a_black_4_4_zombie_with_no_mana_cost_and_those_are_copiable_values() {
    cr!("702.129a", "707.9b", "202.3a");
    ruling!(
        "Timeless Dragon",
        "The token is a Zombie in addition to its other types and is black instead of its other colors. Its base power and toughness are 4/4. It has no mana cost, and thus its mana value is 0. These are copiable values of the token that other effects may copy."
    );
    supported("Timeless Dragon");
    supported("Clone");
    // Timeless Dragon: {3}{W}{W} 5/5 white Dragon with flying; eternalize {2}{W}{W}.
    let mut t = TestGame::new(2);
    let dragon = t.graveyard(P0, "Timeless Dragon");
    t.lands(P0, "Plains", 4);
    eternalize(&mut t, P0, dragon);
    t.resolve_all();
    let token = tokens_with_subtype(&t, P0, "Dragon")[0];
    let o = t.obj_now(token);
    assert_eq!(t.pt(token), (4, 4));
    assert!(o.chars.has_subtype("Zombie") && o.chars.has_subtype("Dragon"));
    assert_eq!(o.chars.colors, ColorSet::single(Color::Black));
    assert!(o.chars.mana_cost.is_none());
    assert_eq!(t.g.mana_value_of(token), 0);
    assert!(o.chars.has_keyword(KeywordKind::Flying));
    // Clone copies the token: a 4/4 black Zombie Dragon with no mana cost.
    t.lands(P0, "Island", 4);
    let clone = t.hand(P0, "Clone");
    t.cast(P0, clone).go();
    t.answer_choose(P0, &[Entity::Object(token)]);
    t.resolve_all();
    let copy = t.g.current(clone);
    assert!(t.on_battlefield(copy));
    let c = t.obj_now(copy);
    assert_eq!(c.chars.name, "Timeless Dragon");
    assert_eq!(t.pt(copy), (4, 4));
    assert!(c.chars.has_subtype("Zombie"));
    assert_eq!(c.chars.colors, ColorSet::single(Color::Black));
    assert!(c.chars.mana_cost.is_none());
    assert_eq!(t.g.mana_value_of(copy), 0);
}

#[test]
fn the_card_is_exiled_as_eternalize_is_activated() {
    cr!("702.129a", "602.2b", "601.2h");
    ruling!(
        "Steadfast Sentinel",
        "Once you've activated an eternalize ability, the card is immediately exiled. Opponents can't try to stop the ability by exiling the card with an effect such as that of Crook of Condemnation."
    );
    supported("Steadfast Sentinel");
    supported("Crook of Condemnation");
    // Crook of Condemnation: "{1}, {T}: Exile target card from a graveyard."
    let mut t = TestGame::new(2);
    let crook = t.battlefield(P1, "Crook of Condemnation");
    let sentinel = t.graveyard(P0, "Steadfast Sentinel");
    let other = t.graveyard(P0, "Grizzly Bears");
    assert!(ability_targets(&mut t, crook, 0).contains(&Entity::Object(sentinel)));
    t.lands(P0, "Plains", 6);
    eternalize(&mut t, P0, sentinel);
    // The ability is on the stack and the card is already in exile: it can't be targeted.
    assert_eq!(t.stack_len(), 1);
    assert!(t.in_exile("Steadfast Sentinel"));
    assert_eq!(
        ability_targets(&mut t, crook, 0),
        vec![Entity::Object(other)]
    );
    t.resolve_all();
    let token = tokens_with_subtype(&t, P0, "Zombie");
    assert_eq!(token.len(), 1);
    assert_eq!(t.pt(token[0]), (4, 4));
}

#[test]
fn the_card_is_exiled_as_eternalize_is_activated_timeless_dragon() {
    cr!("702.129a", "602.2b");
    ruling!(
        "Timeless Dragon",
        "Once you've activated an eternalize ability, the card is immediately exiled. Opponents can't try to stop the ability by exiling the card."
    );
    let mut t = TestGame::new(2);
    let crook = t.battlefield(P1, "Crook of Condemnation");
    let dragon = t.graveyard(P0, "Timeless Dragon");
    assert_eq!(
        ability_targets(&mut t, crook, 0),
        vec![Entity::Object(dragon)]
    );
    t.lands(P0, "Plains", 4);
    eternalize(&mut t, P0, dragon);
    assert!(t.in_exile("Timeless Dragon"));
    assert!(ability_targets(&mut t, crook, 0).is_empty());
    t.resolve_all();
    assert_eq!(tokens_with_subtype(&t, P0, "Dragon").len(), 1);
}

/// P0 casts Murder on their own creature `name` in their main phase; the game runs until
/// someone has priority with the card in P0's graveyard. Returns the card, and whether
/// P1 was asked for priority with the card in the graveyard before that.
fn murder_own(t: &mut TestGame, name: &'static str) -> (ObjectId, bool) {
    t.battlefield(P1, "Crook of Condemnation");
    t.lands(P1, "Wastes", 1);
    let c = t.battlefield(P0, name);
    give_mana_for(t, P0, "Murder");
    let murder = t.hand(P0, "Murder");
    t.cast(P0, murder).target(c).go();
    fn in_graveyard(g: &Game) -> bool {
        let p = g.player(P0);
        p.graveyard
            .iter()
            .any(|c| g.obj(*c).chars.is(CardType::Creature))
    }
    let seen1 = watch(t, P1, is_priority, in_graveyard);
    let ok = t.g.run_until(1000, |g| {
        in_graveyard(g) && g.stack.is_empty() && g.turn.priority.is_some()
    });
    assert!(ok);
    let p1_first = seen1.lock().unwrap().iter().any(|x| *x);
    (t.g.current(c), p1_first)
}

#[test]
fn you_get_priority_to_eternalize_right_after_the_card_is_put_into_your_graveyard() {
    cr!("117.3b", "702.129a");
    ruling!(
        "Proven Combatant",
        "If a creature card with eternalize is put into your graveyard during your main phase, you'll have priority immediately afterward. You can activate its eternalize ability before any player can try to exile it, such as with Crook of Condemnation, if it's legal for you to do so."
    );
    supported("Proven Combatant");
    supported("Murder");
    let mut t = TestGame::new(2);
    let (card, p1_first) = murder_own(&mut t, "Proven Combatant");
    assert_eq!(t.g.turn.priority, Some(P0));
    assert!(!p1_first);
    t.lands(P0, "Island", 6);
    assert!(can_activate(&mut t, P0, card));
    eternalize(&mut t, P0, card);
    assert!(t.in_exile("Proven Combatant"));
    t.resolve_all();
    assert_eq!(tokens_with_subtype(&t, P0, "Zombie").len(), 1);
}

#[test]
fn you_get_priority_to_eternalize_right_after_the_card_is_put_into_your_graveyard_fanatic() {
    cr!("117.3b", "702.129a");
    ruling!(
        "Fanatic of Rhonas",
        "If a creature card with eternalize is put into your graveyard during your main phase, you'll have priority immediately afterward. You can activate its eternalize ability before any player can try to exile it."
    );
    supported("Fanatic of Rhonas");
    let mut t = TestGame::new(2);
    let (card, p1_first) = murder_own(&mut t, "Fanatic of Rhonas");
    assert_eq!(t.g.turn.priority, Some(P0));
    assert!(!p1_first);
    t.lands(P0, "Forest", 4);
    eternalize(&mut t, P0, card);
    assert!(t.in_exile("Fanatic of Rhonas"));
    t.resolve_all();
    let token = tokens_with_subtype(&t, P0, "Snake");
    assert_eq!(token.len(), 1);
    assert_eq!(t.pt(token[0]), (4, 4));
}

#[test]
fn the_card_with_eternalize_cant_be_discarded_to_pay_its_own_cost() {
    cr!("702.129a", "602.2a", "602.2b");
    ruling!(
        "Sunscourge Champion",
        "You can't discard the card with eternalize to pay its own cost because the card has to be in your graveyard to begin activating its eternalize ability."
    );
    supported("Sunscourge Champion");
    // Sunscourge Champion: eternalize—{2}{W}{W}, discard a card.
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 4);
    // In the hand, its eternalize ability can't be activated.
    let champ = t.hand(P0, "Sunscourge Champion");
    t.hand(P0, "Grizzly Bears");
    assert!(!can_activate(&mut t, P0, champ));
    // In the graveyard with no other card in hand: no card to discard.
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 4);
    let champ = t.graveyard(P0, "Sunscourge Champion");
    assert!(!can_activate(&mut t, P0, champ));
    // With another card in hand: that card is discarded.
    t.hand(P0, "Grizzly Bears");
    assert!(can_activate(&mut t, P0, champ));
    eternalize(&mut t, P0, champ);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert!(t.in_exile("Sunscourge Champion"));
    assert_eq!(t.hand_size(P0), 0);
    t.resolve_all();
    assert_eq!(tokens_with_subtype(&t, P0, "Wizard").len(), 1);
}
