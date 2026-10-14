//! CR 701.23i, 608.2f, 614.12: when several players search at once, they all search (in
//! APNAP order), then the cards they found are put onto the battlefield together: a card
//! one player found doesn't see another player's card enter first.

use mtg_engine::decision::Decision;
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
