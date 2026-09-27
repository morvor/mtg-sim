//! Rulings batch S11 — mill (CR 701.17): "you may mill N cards" (Deathcap Marionette,
//! Another Chance, Summon Undead), Millstone, Coral Colony, Madame Web, Wasteful Harvest.

use crate::r_s01_common::*;
use crate::r_s02_common::destroy;
use crate::r_s03_common::in_hand_with_mana;
use crate::r_s05_common::enter;
use crate::r_s07_common::cast_methods;
use crate::r_s11_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Puts `n` Hill Giants into `p`'s otherwise empty library.
fn library_of(t: &mut TestGame, p: PlayerId, n: usize) -> Vec<ObjectId> {
    empty_library(t, p);
    (0..n).map(|_| t.library_top(p, "Hill Giant")).collect()
}

#[test]
fn you_cant_choose_to_mill_two_cards_with_fewer_than_two_in_your_library() {
    cr!("701.17b");
    ruling!(
        "Deathcap Marionette",
        "If you don't have at least two cards in your library, you can't choose to mill two cards."
    );
    ruling!(
        "Another Chance",
        "If you don't have at least two cards in your library, you can't choose to mill two cards."
    );
    supported("Deathcap Marionette");
    supported("Another Chance");
    // Deathcap Marionette: "When this creature enters, you may mill two cards."
    let mut t = TestGame::new(2);
    let lib = library_of(&mut t, P0, 1);
    t.answer_yes(P0, true);
    enter(&mut t, P0, "Deathcap Marionette");
    t.resolve_all();
    assert_eq!(t.library_size(P0), 1);
    assert_eq!(t.zone(t.g.current(lib[0])), Zone::Library(P0));
    assert_eq!(t.graveyard_size(P0), 0);
    // With two, the player may.
    let mut t = TestGame::new(2);
    library_of(&mut t, P0, 2);
    t.answer_yes(P0, true);
    enter(&mut t, P0, "Deathcap Marionette");
    t.resolve_all();
    assert_eq!(t.library_size(P0), 0);
    assert_eq!(t.graveyard_size(P0), 2);
    // Another Chance: "You may mill two cards. Then return up to two creature cards from
    // your graveyard to your hand." With one card in the library, nothing is milled; the
    // creature card already in the graveyard is still returned.
    let mut t = TestGame::new(2);
    library_of(&mut t, P0, 1);
    let bears = t.graveyard(P0, "Grizzly Bears");
    let chance = in_hand_with_mana(&mut t, P0, "Another Chance");
    t.cast(P0, chance).go();
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.resolve_all();
    assert_eq!(t.library_size(P0), 1);
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert!(!t.in_hand(P0, "Hill Giant"));
}

#[test]
fn you_cant_choose_to_mill_three_cards_with_fewer_than_three_in_your_library() {
    cr!("701.17b", "608.2c");
    ruling!(
        "Summon Undead",
        "If you have fewer than three cards in your library, you can't choose to mill any cards this way."
    );
    ruling!(
        "Druidic Ritual",
        "If you have fewer than three cards in your library, you can't choose to mill any cards this way."
    );
    supported("Summon Undead");
    supported("Druidic Ritual");
    // Summon Undead: "You may mill three cards. Then return a creature card from your
    // graveyard to the battlefield." With two cards, nothing is milled: the Bears already
    // in the graveyard return.
    let mut t = TestGame::new(2);
    let lib = library_of(&mut t, P0, 2);
    let bears = t.graveyard(P0, "Grizzly Bears");
    let spell = in_hand_with_mana(&mut t, P0, "Summon Undead");
    t.cast(P0, spell).go();
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.library_size(P0), 2);
    assert!(lib.iter().all(|c| t.zone(t.g.current(*c)) == Zone::Library(P0)));
    assert!(t.on_battlefield(t.g.current(bears)));
    // With three, the player mills them and may return one of them.
    let mut t = TestGame::new(2);
    library_of(&mut t, P0, 3);
    let spell = in_hand_with_mana(&mut t, P0, "Summon Undead");
    t.cast(P0, spell).go();
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.library_size(P0), 0);
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
    assert_eq!(t.graveyard_size(P0), 3);
    // Druidic Ritual: "You may mill three cards. Then return up to one creature card and
    // up to one land card from your graveyard to your hand."
    let mut t = TestGame::new(2);
    library_of(&mut t, P0, 2);
    let bears = t.graveyard(P0, "Grizzly Bears");
    let forest = t.graveyard(P0, "Forest");
    let ritual = in_hand_with_mana(&mut t, P0, "Druidic Ritual");
    t.cast(P0, ritual).go();
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.answer_choose(P0, &[Entity::Object(forest)]);
    t.resolve_all();
    assert_eq!(t.library_size(P0), 2);
    assert!(t.in_hand(P0, "Grizzly Bears") && t.in_hand(P0, "Forest"));
}

#[test]
fn millstone_isnt_a_draw_so_a_short_library_doesnt_lose() {
    cr!("701.17a", "701.17b", "704.5b", "121.4");
    ruling!(
        "Millstone",
        "It is not a draw effect so it will not cause a player with less than 2 cards in their library to lose."
    );
    supported("Millstone");
    // Millstone: "{2}, {T}: Target player mills two cards."
    let mut t = TestGame::new(2);
    library_of(&mut t, P1, 1);
    let stone = t.battlefield(P0, "Millstone");
    t.lands(P0, "Wastes", 2);
    t.activate(P0, stone, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    t.settle();
    assert_eq!(t.library_size(P1), 0);
    assert_eq!(t.graveyard_size(P1), 1);
    assert!(!t.has_lost(P1));
    // Only drawing from the empty library loses.
    t.advance_to(P1, Step::Draw);
    t.settle();
    assert!(t.has_lost(P1));
}

#[test]
fn coral_colony_counts_defenders_as_its_ability_resolves() {
    cr!("608.2h", "701.17a");
    ruling!(
        "Coral Colony",
        "The number of creatures with defender you control is counted when the ability resolves."
    );
    supported("Coral Colony");
    supported("Wall of Omens");
    // Coral Colony: "{1}{U}, {T}: Target player mills X cards, where X is the number of
    // creatures you control with defender." Activated with two defenders; one leaves and
    // two more enter before it resolves.
    let mut t = TestGame::new(2);
    let colony = t.battlefield(P0, "Coral Colony");
    let wall = t.battlefield(P0, "Wall of Omens");
    t.lands(P0, "Island", 2);
    t.activate(P0, colony, 0, &[Entity::Player(P1)]).unwrap();
    destroy(&mut t, wall);
    t.battlefield(P0, "Coral Colony");
    t.battlefield(P0, "Wall of Omens");
    let lib = t.library_size(P1);
    t.resolve_all();
    assert_eq!(t.library_size(P1), lib - 3);
    assert_eq!(t.graveyard_size(P1), 3);
}

#[test]
fn casting_from_the_top_of_the_library_follows_normal_timing() {
    cr!("601.3", "307.1", "117.1a");
    ruling!(
        "Madame Web, Clairvoyant",
        "You must follow all normal timing rules when casting a spell this way."
    );
    supported("Madame Web, Clairvoyant");
    // Madame Web: "You may cast Spider spells and noncreature spells from the top of your
    // library."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Madame Web, Clairvoyant");
    let div = t.library_top(P0, "Divination");
    t.lands(P0, "Island", 3);
    // A sorcery: in P0's main phase with an empty stack.
    assert!(cast_methods(&mut t, P0, div).contains(&CastMethod::Normal));
    // Not while a spell is on the stack.
    let bolt = in_hand_with_mana(&mut t, P1, "Lightning Bolt");
    t.cast(P1, bolt).target(Entity::Player(P1)).go();
    assert!(cast_methods(&mut t, P0, div).is_empty());
    t.resolve_all();
    // Nor during the opponent's turn.
    t.advance_to(P1, Step::PrecombatMain);
    assert!(cast_methods(&mut t, P0, div).is_empty());
    // An instant on top can be cast then.
    let top = t.library_top(P0, "Lightning Bolt");
    t.lands(P0, "Mountain", 1);
    assert!(cast_methods(&mut t, P0, top).contains(&CastMethod::Normal));
    // A (non-Spider) creature card can't be cast from the top at all.
    t.advance_to(P0, Step::PrecombatMain);
    let giant = t.library_top(P0, "Hill Giant");
    t.lands(P0, "Mountain", 4);
    assert!(cast_methods(&mut t, P0, giant).is_empty());
}

#[test]
fn cards_milled_into_exile_instead_are_still_milled_this_way() {
    cr!("701.17c", "614.1a");
    ruling!(
        "Wasteful Harvest",
        "If a replacement effect causes the milled cards to be exiled instead of going to the graveyard, this effect can still apply to the cards \"milled this way\" in exile."
    );
    supported("Wasteful Harvest");
    supported("Rest in Peace");
    // Wasteful Harvest: "Mill five cards. You may put a permanent card from among the cards
    // milled this way into your hand." Rest in Peace: "If a card or token would be put
    // into a graveyard from anywhere, exile it instead."
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Rest in Peace");
    let cards = stack_library(
        &mut t,
        P0,
        &["Lightning Bolt", "Hill Giant", "Opt", "Forest", "Divination"],
    );
    let harvest = in_hand_with_mana(&mut t, P0, "Wasteful Harvest");
    // P0 chooses the Hill Giant among the exiled cards, having seen only the permanent
    // cards (Hill Giant and Forest) offered.
    fn choose_giant(g: &mtg_engine::game::Game, d: &Decision) -> Option<Answer> {
        let Decision::ChooseEntities { candidates, .. } = d else {
            return None;
        };
        let names: Vec<&str> = candidates
            .iter()
            .filter_map(|e| e.object())
            .map(|o| g.obj(o).chars.name.as_str())
            .collect();
        assert_eq!(names.len(), 2, "offered {names:?}");
        assert!(names.contains(&"Forest"));
        assert!(candidates
            .iter()
            .filter_map(|e| e.object())
            .all(|o| g.obj(o).zone == Zone::Exile));
        let giant = candidates
            .iter()
            .find(|e| e.object().is_some_and(|o| g.obj(o).chars.name == "Hill Giant"))?;
        Some(Answer::Entities(vec![*giant]))
    }
    crate::r_s03_common::respond(&mut t, P0, choose_giant);
    t.cast(P0, harvest).go();
    t.resolve_all();
    assert!(t.in_hand(P0, "Hill Giant"));
    for c in [cards[0], cards[2], cards[3], cards[4]] {
        assert_eq!(t.zone(t.g.current(c)), Zone::Exile);
    }
    assert_eq!(t.graveyard_size(P0), 0);
}
