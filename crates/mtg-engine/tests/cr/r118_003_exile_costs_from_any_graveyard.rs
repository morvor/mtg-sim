//! CR 118.3, 602.2b, 601.2h, 404.1: a cost to exile cards "from a graveyard" may use any
//! player's graveyard; "from a single graveyard" needs all of them from the same one; "from
//! your graveyard" only the payer's.

use mtg_engine::card::card;
use mtg_engine::mana::ManaType;
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

fn supported(name: &str) {
    let c = card(name);
    assert!(
        c.is_fully_supported(),
        "{name}: unsupported {:?}",
        c.unsupported_text()
    );
}

fn add_mana(t: &mut TestGame, p: PlayerId, ty: ManaType, n: u32) {
    t.g.players[p.idx()].mana_pool.add_type(ty, n);
}

#[test]
fn a_card_may_be_exiled_from_any_graveyard() {
    cr!("118.3", "602.2b", "404.1");
    supported("Thelon of Havenwood");
    let mut t = TestGame::new(2);
    // Thelon of Havenwood: "{B}{G}, Exile a Fungus card from a graveyard: Put a spore
    // counter on each Fungus on the battlefield."
    let thelon = t.battlefield(P0, "Thelon of Havenwood");
    let thallid = t.battlefield(P0, "Thallid");
    // The only Fungus card is in the opponent's graveyard.
    let theirs = t.graveyard(P1, "Thallid");
    add_mana(&mut t, P0, ManaType::B, 1);
    add_mana(&mut t, P0, ManaType::G, 1);
    t.answer_choose(P0, &[Entity::Object(theirs)]);
    t.activate(P0, thelon, 0, &[]).expect("activate");
    assert_eq!(t.zone(t.g.current(theirs)), Zone::Exile);
    t.resolve_all();
    assert_eq!(t.counters(thallid, "spore"), 1);
    // No Fungus card in any graveyard: the cost can't be paid.
    add_mana(&mut t, P0, ManaType::B, 1);
    add_mana(&mut t, P0, ManaType::G, 1);
    t.graveyard(P1, "Grizzly Bears");
    assert!(t.activate(P0, thelon, 0, &[]).is_err());
}

#[test]
fn cards_exiled_from_a_single_graveyard_come_from_the_same_one() {
    cr!("118.3", "602.2b", "601.2h");
    ruling!(
        "Night Soil",
        "If no graveyard has two creature cards in it, the cost can't be paid."
    );
    supported("Night Soil");
    // Night Soil: "{1}, Exile two creature cards from a single graveyard: Create a 1/1
    // green Saproling creature token."
    let mut t = TestGame::new(2);
    let soil = t.battlefield(P0, "Night Soil");
    t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P1, "Gray Ogre");
    add_mana(&mut t, P0, ManaType::C, 1);
    // One creature card in each graveyard: no graveyard has two.
    assert!(t.activate(P0, soil, 0, &[]).is_err());
    // Two in the opponent's graveyard: those two are exiled.
    let a = t.graveyard(P1, "Hill Giant");
    t.activate(P0, soil, 0, &[]).expect("activate");
    assert_eq!(t.zone(t.g.current(a)), Zone::Exile);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert!(!t.in_graveyard(P1, "Gray Ogre"));
    t.resolve_all();
    let saprolings = t
        .g
        .permanents()
        .filter(|o| o.is_token() && o.chars.has_subtype("Saproling"))
        .count();
    assert_eq!(saprolings, 1);
}

#[test]
fn a_cost_from_your_graveyard_uses_only_yours() {
    cr!("118.3", "602.2b");
    // Makeshift Mauler {3}{U}: "As an additional cost to cast this spell, exile a creature
    // card from your graveyard." The opponent's graveyard doesn't count.
    let mut t = TestGame::new(2);
    let mauler = t.hand(P0, "Makeshift Mauler");
    t.graveyard(P1, "Grizzly Bears");
    add_mana(&mut t, P0, ManaType::U, 1);
    add_mana(&mut t, P0, ManaType::C, 3);
    assert!(t.cast(P0, mauler).try_go().is_err());
    t.graveyard(P0, "Gray Ogre");
    t.cast(P0, mauler).go();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(!t.in_graveyard(P0, "Gray Ogre"));
}
