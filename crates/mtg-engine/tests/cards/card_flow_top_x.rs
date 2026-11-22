//! "Reveal the top X cards of your library" on instants and sorceries with {X} in their
//! mana cost (or an X defined by the text), and "Then put all cards revealed this way that
//! weren't put onto the battlefield into your graveyard" (Genesis Wave, Saheeli's
//! Directive).

use mtg_engine::card::card;
use mtg_engine::decision::Answer;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

fn assert_supported(name: &str) {
    let c = card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        c.unsupported_text()
    );
}

/// Stacks named cards on top of P0's library; the last one named ends up on top.
fn stack(t: &mut TestGame, names: &[&str]) -> Vec<ObjectId> {
    names.iter().map(|n| t.library_top(P0, n)).collect()
}

#[test]
fn genesis_wave_puts_permanents_with_mana_value_x_or_less_onto_the_battlefield() {
    cr!("107.3a", "701.20a");
    assert_supported("Genesis Wave");
    // "Reveal the top X cards of your library. You may put any number of permanent cards
    // with mana value X or less from among them onto the battlefield. Then put all cards
    // revealed this way that weren't put onto the battlefield into your graveyard."
    let mut t = TestGame::new(2);
    let below = t.library_top(P0, "Llanowar Elves");
    let [giant, bolt, bears, forest] = stack(
        &mut t,
        &["Hill Giant", "Lightning Bolt", "Grizzly Bears", "Forest"],
    )[..] else {
        unreachable!()
    };
    t.lands(P0, "Forest", 7);
    let wave = t.hand(P0, "Genesis Wave");
    // X = 4: the top four cards. The Bears and the Forest are chosen; the Hill Giant (mana
    // value 4) could have been.
    t.answer(P0, DecisionKind::X, Answer::Number(4));
    t.answer_choose(P0, &[Entity::Object(bears), Entity::Object(forest)]);
    t.cast(P0, wave).go();
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert!(t.on_battlefield(forest));
    assert_eq!(t.zone(giant), Zone::Graveyard(P0));
    assert_eq!(t.zone(bolt), Zone::Graveyard(P0));
    // The fifth card wasn't revealed.
    assert_eq!(t.zone(below), Zone::Library(P0));
    assert!(t.in_graveyard(P0, "Genesis Wave"));
}

#[test]
fn genesis_wave_cant_put_a_card_with_greater_mana_value_onto_the_battlefield() {
    cr!("107.3a");
    let mut t = TestGame::new(2);
    let [giant, bears] = stack(&mut t, &["Hill Giant", "Grizzly Bears"])[..] else {
        unreachable!()
    };
    t.lands(P0, "Forest", 5);
    let wave = t.hand(P0, "Genesis Wave");
    // X = 2: the Hill Giant (mana value 4) can't be put onto the battlefield.
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.answer_choose(P0, &[Entity::Object(giant), Entity::Object(bears)]);
    t.cast(P0, wave).go();
    t.resolve_all();
    assert!(!t.on_battlefield(giant));
    assert_eq!(t.zone(giant), Zone::Graveyard(P0));
}

#[test]
fn machinate_looks_at_x_cards_where_x_is_the_number_of_artifacts() {
    cr!("107.3c");
    assert_supported("Machinate");
    // "Look at the top X cards of your library, where X is the number of artifacts you
    // control. Put one of those cards into your hand and the rest on the bottom of your
    // library in any order."
    let mut t = TestGame::new(2);
    let deep = t.library_top(P0, "Hill Giant");
    let [bolt, bears] = stack(&mut t, &["Lightning Bolt", "Grizzly Bears"])[..] else {
        unreachable!()
    };
    t.battlefield(P0, "Ornithopter");
    t.battlefield(P0, "Ornithopter");
    t.lands(P0, "Island", 3);
    let m = t.hand(P0, "Machinate");
    t.answer_choose(P0, &[Entity::Object(bolt)]);
    t.cast(P0, m).go();
    t.resolve_all();
    assert_eq!(t.zone(bolt), Zone::Hand(P0));
    // The other one went to the bottom; the third card wasn't looked at.
    assert_eq!(t.g.player(P0).library[0], bears);
    assert_eq!(*t.g.player(P0).library.last().unwrap(), deep);
}

#[test]
fn flash_of_insight_looks_at_x_cards() {
    cr!("107.3a");
    assert_supported("Flash of Insight");
    // "Look at the top X cards of your library. Put one of them into your hand and the
    // rest on the bottom of your library in any order. Flashback—{1}{U}, Exile X blue
    // cards from your graveyard."
    let mut t = TestGame::new(2);
    let deep = t.library_top(P0, "Hill Giant");
    let [bolt, bears] = stack(&mut t, &["Lightning Bolt", "Grizzly Bears"])[..] else {
        unreachable!()
    };
    t.lands(P0, "Island", 4);
    let f = t.hand(P0, "Flash of Insight");
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.answer_choose(P0, &[Entity::Object(bolt)]);
    t.cast(P0, f).go();
    t.resolve_all();
    assert_eq!(t.zone(bolt), Zone::Hand(P0));
    assert_eq!(t.g.player(P0).library[0], bears);
    assert_eq!(*t.g.player(P0).library.last().unwrap(), deep);
    // Flashback: X appears only in the alternative cost ("Exile X blue cards"), and is
    // chosen as it's cast (CR 107.3a). Exiling one blue card: X is 1.
    let mut t = TestGame::new(2);
    let deep = t.library_top(P0, "Hill Giant");
    let top = t.library_top(P0, "Grizzly Bears");
    t.lands(P0, "Island", 2);
    t.graveyard(P0, "Counterspell");
    let f = t.graveyard(P0, "Flash of Insight");
    t.answer(P0, DecisionKind::X, Answer::Number(1));
    t.answer_choose(P0, &[Entity::Object(top)]);
    t.cast(P0, f)
        .method(mtg_engine::object::CastMethod::Keyword(
            mtg_engine::keywords::KeywordKind::Flashback,
        ))
        .go();
    t.resolve_all();
    assert_eq!(t.zone(top), Zone::Hand(P0));
    assert_eq!(t.zone(deep), Zone::Library(P0));
    assert!(t.in_exile("Counterspell"));
}
