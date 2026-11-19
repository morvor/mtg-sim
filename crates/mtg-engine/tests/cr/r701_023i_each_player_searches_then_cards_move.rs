//! CR 701.23i, 608.2f, 614.12: when several players search at once, they all search (in
//! APNAP order), then the cards they found are put onto the battlefield together: a card
//! one player found doesn't see another player's card enter first.

use mtg_engine::decision::Decision;
use mtg_engine::events::Event;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn incoming_a_clone_found_by_one_player_cant_copy_a_creature_another_found() {
    cr!("701.23i", "608.2f", "614.12");
    ruling!(
        "Clone",
        "If Clone somehow enters at the same time as another creature, Clone can't become a copy of that creature."
    );
    // "Each player searches their library for any number of artifact, creature,
    // enchantment, and/or land cards, puts them onto the battlefield, then shuffles."
    let mut t = TestGame::new(2);
    let bears = t.library_top(P0, "Grizzly Bears");
    let clone = t.library_top(P1, "Clone");
    t.lands(P0, "Forest", 8);
    let spell = t.hand(P0, "Incoming!");
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.answer_choose(P1, &[Entity::Object(clone)]);
    let from = t.asked().len();
    t.cast(P0, spell).go();
    t.resolve_all();
    let bears_now = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(bears_now.len(), 1, "P0's Grizzly Bears entered; nothing copied it");
    // P1 was never offered the Bears to copy: it entered together with the Clone.
    let offered_bears = t.asked()[from..].iter().any(|(p, d)| match d {
        Decision::ChooseEntities { candidates, .. } => {
            *p == P1 && candidates.contains(&Entity::Object(bears_now[0]))
        }
        _ => false,
    });
    assert!(!offered_bears);
    // The Clone entered as a 0/0 and died.
    assert_eq!(t.obj_now(clone).zone, Zone::Graveyard(P1));
}

#[test]
fn boldwyr_heavyweights_a_clone_one_opponent_finds_cant_copy_what_another_finds() {
    cr!("701.23i", "608.2f", "614.12");
    ruling!(
        "Clone",
        "If Clone somehow enters at the same time as another creature, Clone can't become a copy of that creature."
    );
    assert!(card("Boldwyr Heavyweights").is_fully_supported());
    // "When this creature enters, each opponent may search their library for a creature
    // card and put it onto the battlefield. Then each player who searched their library
    // this way shuffles." P1 finds Grizzly Bears, P2 finds Clone: they enter together.
    let mut t = TestGame::new(3);
    let bears = t.library_top(P1, "Grizzly Bears");
    let clone = t.library_top(P2, "Clone");
    t.answer_yes(P1, true);
    t.answer_yes(P2, true);
    t.answer_choose(P1, &[Entity::Object(bears)]);
    t.answer_choose(P2, &[Entity::Object(clone)]);
    let from = t.asked().len();
    t.enter(P0, "Boldwyr Heavyweights");
    t.resolve_all();
    let bears_now = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(bears_now.len(), 1);
    let offered_bears = t.asked()[from..].iter().any(|(p, d)| match d {
        Decision::ChooseEntities { candidates, .. } => {
            *p == P2 && candidates.contains(&Entity::Object(bears_now[0]))
        }
        _ => false,
    });
    assert!(!offered_bears);
}

#[test]
fn krenkos_buzzcrusher_the_controllers_search_then_shuffle_in_turn_order() {
    cr!("701.23i", "608.2f", "101.4");
    assert!(card("Krenko's Buzzcrusher").is_fully_supported());
    // "When this creature enters, for each player, destroy up to one nonbasic land that
    // player controls. For each land destroyed this way, its controller may search their
    // library for a basic land card, put it onto the battlefield tapped, then shuffle."
    // Both players' lands are destroyed and both search: they search, then the lands enter
    // together and they shuffle, in turn order.
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Ancient Tomb");
    let theirs = t.battlefield(P1, "Ancient Tomb");
    t.library_top(P0, "Forest");
    t.library_top(P1, "Forest");
    t.answer_choose(P0, &[Entity::Object(mine)]);
    t.answer_choose(P0, &[Entity::Object(theirs)]);
    t.answer_yes(P0, true);
    t.answer_yes(P1, true);
    let from = t.g.turn_events.len();
    t.enter(P0, "Krenko's Buzzcrusher");
    t.resolve_all();
    let order: Vec<String> = t.g.turn_events[from..]
        .iter()
        .filter_map(|e| match e {
            Event::Searched { player, .. } => Some(format!("search {player}")),
            Event::Shuffled { player } => Some(format!("shuffle {player}")),
            _ => None,
        })
        .collect();
    assert_eq!(
        order,
        vec!["search P0", "search P1", "shuffle P0", "shuffle P1"]
    );
    for p in [P0, P1] {
        assert!(!t.in_graveyard(p, "Forest") && t.g.permanents().any(|o| o.controller == p
            && o.chars.name == "Forest"));
    }
}
