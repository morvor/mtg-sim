//! CR 701.57: discover.

use crate::a701_028_071_common::*;
use mtg_engine::ability::*;
use mtg_engine::kwa::discover::DISCOVERED_EVENT;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// Puts cards on top of P0's library so that the first one listed is on top.
fn stack_library(t: &mut TestGame, names: &[&str]) -> Vec<ObjectId> {
    let ids: Vec<ObjectId> = names
        .iter()
        .rev()
        .map(|n| t.library_top(P0, n))
        .collect();
    ids.into_iter().rev().collect()
}

#[test]
fn discover_exiles_until_a_nonland_card_with_small_enough_mana_value() {
    cr!("701.57a");
    ruling!(
        "Geological Appraiser",
        "When you discover, you must exile cards. The only optional part of the ability is whether you cast the exiled card or put it into your hand."
    );
    supported("Geological Appraiser");
    // "When this creature enters, if you cast it, discover 3."
    let mut t = TestGame::new(2);
    let ids = stack_library(
        &mut t,
        &["Forest", "Hill Giant", "Lightning Bolt", "Grizzly Bears"],
    );
    let (forest, giant, bolt, bears) = (ids[0], ids[1], ids[2], ids[3]);
    t.lands(P0, "Mountain", 4);
    let appraiser = t.hand(P0, "Geological Appraiser");
    t.cast(P0, appraiser).go();
    t.resolve(); // the creature spell
    // Cast Lightning Bolt without paying its mana cost, targeting P1.
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert!(t.in_graveyard(P0, "Lightning Bolt"));
    let _ = bolt;
    // The Forest and Hill Giant went to the bottom; Grizzly Bears is still on top.
    let lib = &t.g.player(P0).library;
    assert_eq!(t.g.library_top(P0), Some(bears));
    let bottom: Vec<ObjectId> = lib[..2].to_vec();
    let names: Vec<String> = bottom
        .iter()
        .map(|o| t.obj(*o).chars.name.to_string())
        .collect();
    assert!(names.contains(&"Forest".to_string()) && names.contains(&"Hill Giant".to_string()));
    assert!(!t.g.is_live(forest) && !t.g.is_live(giant));
    assert!(t.g.exile.is_empty());
}

#[test]
fn a_discovered_card_that_isnt_cast_goes_to_hand() {
    cr!("701.57a");
    supported("Trumpeting Carnosaur");
    let mut t = TestGame::new(2);
    stack_library(&mut t, &["Island", "Grizzly Bears"]);
    t.answer_yes(P0, false);
    t.enter(P0, "Trumpeting Carnosaur");
    t.resolve_all();
    assert!(t.in_hand(P0, "Grizzly Bears"));
    // The Island went to the bottom.
    assert_eq!(
        t.obj(t.g.player(P0).library[0]).chars.name.as_str(),
        "Island"
    );
}

#[test]
fn a_player_discovers_even_if_nothing_was_found() {
    cr!("701.57b");
    ruling!(
        "Curator of Sun's Creation",
        "even if one or more of those actions were impossible for some reason."
    );
    supported("Curator of Sun's Creation");
    // "Whenever you discover, discover again for the same value. This ability triggers
    // only once each turn."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Curator of Sun's Creation");
    // Only lands and a card with too great a mana value in the library.
    t.g.players[0].library.clear();
    stack_library(&mut t, &["Forest", "Hill Giant", "Island"]);
    run(
        &mut t,
        P0,
        None,
        ka(KeywordAction::Discover, Sel::None, 2),
        &[],
    );
    t.resolve_all();
    // Nothing was discovered, but P0 discovered — twice (the Curator's trigger), with the
    // same value.
    let ev = custom_events(&t, DISCOVERED_EVENT);
    assert_eq!(ev, vec![(Some(P0), None, 2), (Some(P0), None, 2)]);
    assert_eq!(t.library_size(P0), 3);
    assert_eq!(t.hand_size(P0), 0);
}

#[test]
fn the_discovered_card_is_the_last_card_exiled_whether_cast_or_put_into_hand() {
    cr!("701.57c");
    supported("Hit the Mother Lode");
    // "Discover 10. If the discovered card's mana value is less than 10, create a number
    // of tapped Treasure tokens equal to the difference."
    let treasures = |t: &TestGame| {
        t.g.permanents()
            .filter(|o| o.is_token() && o.chars.has_subtype("Treasure"))
            .count()
    };
    // Cast: Hill Giant (mana value 4): six Treasures.
    let mut t = TestGame::new(2);
    stack_library(&mut t, &["Forest", "Hill Giant"]);
    t.lands(P0, "Mountain", 7);
    let spell = t.hand(P0, "Hit the Mother Lode");
    t.answer_yes(P0, true);
    t.cast(P0, spell).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
    assert_eq!(treasures(&t), 6);
    // Put into the hand: Grizzly Bears (mana value 2): eight Treasures.
    let mut t = TestGame::new(2);
    stack_library(&mut t, &["Grizzly Bears"]);
    t.lands(P0, "Mountain", 7);
    let spell = t.hand(P0, "Hit the Mother Lode");
    t.answer_yes(P0, false);
    t.cast(P0, spell).go();
    t.resolve_all();
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert_eq!(treasures(&t), 8);
    assert!(t.g.permanents().filter(|o| o.is_token()).all(|o| o.tapped));
    // No discovered card: no Treasures.
    let mut t = TestGame::new(2);
    t.g.players[0].library.clear();
    stack_library(&mut t, &["Forest", "Island"]);
    t.lands(P0, "Mountain", 7);
    let spell = t.hand(P0, "Hit the Mother Lode");
    t.cast(P0, spell).go();
    t.resolve_all();
    assert_eq!(treasures(&t), 0);
    assert_eq!(t.zone(t.g.player(P0).library[0]), Zone::Library(P0));
}
