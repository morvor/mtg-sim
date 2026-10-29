//! Vexing Arcanix: "{3}, {T}: Target player chooses a card name, then reveals the top card
//! of their library. If that card has the chosen name, that player puts it into their
//! hand. Otherwise, they put it into their graveyard and this artifact deals 2 damage to
//! them." "Them" is the target player (`player_subjects.rs`), not the revealed card.

use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

fn activate(t: &mut TestGame, named: &str) -> ObjectId {
    t.lands(P0, "Island", 3);
    let arcanix = t.battlefield(P0, "Vexing Arcanix");
    t.answer(P1, DecisionKind::Name, Answer::Text(named.into()));
    t.activate(P0, arcanix, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    arcanix
}

#[test]
fn a_wrong_guess_puts_the_card_into_the_graveyard_and_deals_2_damage_to_the_player() {
    cr!("201.4", "120.3a");
    let c = card("Vexing Arcanix");
    assert!(c.unsupported_text().is_empty(), "{:?}", c.unsupported_text());
    ruling!("Vexing Arcanix", "The target player names a card on resolution.");
    let mut t = TestGame::new(2);
    let bears = t.library_top(P1, "Grizzly Bears");
    activate(&mut t, "Lightning Bolt");
    assert!(t
        .asked()
        .into_iter()
        .any(|(p, d)| p == P1 && matches!(d, Decision::NameCard { .. })));
    assert_eq!(t.zone(t.g.current(bears)), Zone::Graveyard(P1));
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn a_right_guess_puts_the_card_into_their_hand() {
    cr!("201.4");
    let mut t = TestGame::new(2);
    let bears = t.library_top(P1, "Grizzly Bears");
    activate(&mut t, "Grizzly Bears");
    assert_eq!(t.zone(t.g.current(bears)), Zone::Hand(P1));
    assert_eq!(t.life(P1), 20);
}

#[test]
fn an_empty_library_deals_no_damage() {
    cr!("201.4");
    ruling!(
        "Vexing Arcanix",
        "If the player has no cards in their library, the effect does nothing. It does not cause any damage."
    );
    let mut t = TestGame::new(2);
    t.g.players[1].library.clear();
    activate(&mut t, "Lightning Bolt");
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.life(P0), 20);
}
