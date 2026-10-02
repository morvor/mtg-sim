//! Rulings batch P171 — cards that look for land types: Choked Estuary and Foreboding
//! Ruins (reveal a land card with a subtype), Drowned Catacomb and Dragonskull Summit
//! (control a land with a subtype), Bubbling Muck (Swamps tapped for mana) and Infernal
//! Harvest (return X Swamps as a cost).

use crate::r_p171_common::*;
use crate::r_s20_common::tap_for_mana;
use crate::r_s24_common::pool;
use crate::r_s25_common::lands_for_cost;
use crate::r_s31_common::{entered_tapped, put_together, revealed_cards};
use mtg_engine::decision::Answer;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

/// (reveal land, the land types it asks for, a land card with one of those types).
const REVEAL_LANDS: [(&str, &str); 2] = [
    ("Choked Estuary", "Island"),
    ("Foreboding Ruins", "Swamp"),
];

#[test]
fn a_land_entering_from_hand_at_the_same_time_can_be_revealed() {
    cr!("614.12", "614.12a", "614.13a");
    ruling!(
        "Choked Estuary",
        "If an Island or Swamp is entering the battlefield from your hand at the same time as Choked Estuary, you may reveal the other land to have Choked Estuary enter untapped."
    );
    ruling!(
        "Foreboding Ruins",
        "If a Swamp or Mountain is entering the battlefield from your hand at the same time as Foreboding Ruins, you may reveal the other land to have Foreboding Ruins enter untapped."
    );
    for (name, basic) in REVEAL_LANDS {
        supported(name);
        let mut t = TestGame::new(2);
        let land = t.hand(P0, name);
        let other = t.hand(P0, basic);
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[obj(other)]);
        let ids = put_together(&mut t, &[land, other]);
        assert!(ids.iter().all(|i| i.is_some()));
        assert_eq!(t.zone(other), Zone::Battlefield);
        assert!(revealed_cards(&t).contains(&other), "{name}");
        assert!(!entered_tapped(&t, land), "{name}");
        // Without one, it enters tapped.
        let lone = t.hand(P0, name);
        put_together(&mut t, &[lone]);
        assert!(entered_tapped(&t, lone), "{name}");
    }
}

#[test]
fn put_onto_the_battlefield_tapped_wins_even_if_you_reveal() {
    cr!("614.12", "614.1c");
    ruling!(
        "Choked Estuary",
        "If an effect instructs you to put Choked Estuary onto the battlefield tapped, it will still enter the battlefield tapped even if you reveal a land card from your hand."
    );
    ruling!(
        "Foreboding Ruins",
        "If an effect instructs you to put Foreboding Ruins onto the battlefield tapped, it will still enter the battlefield tapped even if you reveal a land card from your hand."
    );
    supported("Hour of Promise");
    for (name, basic) in REVEAL_LANDS {
        // Hour of Promise: "Search your library for up to two land cards, put them onto
        // the battlefield tapped, then shuffle. ..."
        let mut t = TestGame::new(2);
        let land = t.library_top(P0, name);
        let other = t.hand(P0, basic);
        lands_for_cost(&mut t, P0, "Hour of Promise");
        let hour = t.hand(P0, "Hour of Promise");
        t.answer_choose(P0, &[obj(land)]);
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[obj(other)]);
        t.cast(P0, hour).go();
        t.resolve_all();
        assert!(t.in_hand(P0, basic), "{name}");
        assert!(revealed_cards(&t).contains(&other), "{name}");
        assert!(entered_tapped(&t, land), "{name}");
    }
}

#[test]
fn the_land_itself_has_no_land_types() {
    cr!("205.3i", "614.12");
    ruling!(
        "Choked Estuary",
        "Lands don't have a subtype just because they can produce mana of the corresponding color. Choked Estuary itself is neither an Island nor a Swamp"
    );
    ruling!(
        "Foreboding Ruins",
        "Lands don't have a subtype just because they can produce mana of the corresponding color. Foreboding Ruins itself is neither a Swamp nor a Mountain"
    );
    for (name, _) in REVEAL_LANDS {
        // The only land card in hand is another copy of the same land: nothing can be
        // revealed, and it enters tapped.
        let mut t = TestGame::new(2);
        let land = t.hand(P0, name);
        let copy = t.hand(P0, name);
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[obj(copy)]);
        put_together(&mut t, &[land]);
        assert!(!revealed_cards(&t).contains(&copy), "{name}");
        assert!(entered_tapped(&t, land), "{name}");
    }
}

#[test]
fn any_land_card_with_the_subtype_can_be_revealed() {
    cr!("614.12", "205.3i");
    ruling!(
        "Choked Estuary",
        "You may reveal any land card with either or both of the appropriate subtypes. It doesn't have to be a basic land. For example, you could reveal Prairie Stream"
    );
    ruling!(
        "Foreboding Ruins",
        "You may reveal any land card with either or both of the appropriate subtypes. It doesn't have to be a basic land. For example, you could reveal Sunken Hollow"
    );
    // Prairie Stream (Plains Island); Sunken Hollow (Island Swamp).
    for (name, nonbasic) in [
        ("Choked Estuary", "Prairie Stream"),
        ("Foreboding Ruins", "Sunken Hollow"),
    ] {
        let mut t = TestGame::new(2);
        let land = t.hand(P0, name);
        let other = t.hand(P0, nonbasic);
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[obj(other)]);
        put_together(&mut t, &[land]);
        assert!(revealed_cards(&t).contains(&other), "{name}");
        assert!(!entered_tapped(&t, land), "{name}");
    }
}

#[test]
fn check_lands_look_for_land_types_not_names() {
    cr!("614.12", "614.1d", "205.3i");
    ruling!(
        "Drowned Catacomb",
        "This checks for lands you control with the land type Island or Swamp, not for lands named Island or Swamp. The lands it checks for don't have to be basic lands. For example, if you control Blood Crypt"
    );
    ruling!(
        "Dragonskull Summit",
        "This checks for lands you control with the land type Swamp or Mountain, not for lands named Swamp or Mountain. The lands it checks for don't have to be basic lands. For example, if you control Stomping Ground"
    );
    supported("Drowned Catacomb");
    supported("Dragonskull Summit");
    for (name, typed) in [
        ("Drowned Catacomb", "Blood Crypt"),
        ("Dragonskull Summit", "Stomping Ground"),
    ] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, typed);
        let land = t.enter(P0, name);
        assert!(!entered_tapped(&t, land), "{name}");
        // A land without those types (Plains) doesn't count.
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Plains");
        let land = t.enter(P0, name);
        assert!(entered_tapped(&t, land), "{name}");
    }
}

#[test]
fn bubbling_muck_affects_every_swamp_tapped_this_turn() {
    cr!("605.1b", "106.12a");
    ruling!(
        "Bubbling Muck",
        "Affects lands tapped for rest of turn, not just swamps on the battlefield at the time it resolves."
    );
    ruling!(
        "Bubbling Muck",
        "Affects lands with type Swamp, not lands that are named “Swamp.”"
    );
    supported("Bubbling Muck");
    let mut t = TestGame::new(2);
    cast_card(&mut t, P0, "Bubbling Muck");
    t.resolve_all();
    // A Swamp that arrives later, and a Swamp of P1's.
    let swamp = t.battlefield(P0, "Swamp");
    assert!(tap_for_mana(&mut t, P0, swamp, "Add"));
    assert_eq!(pool(&t, P0, ManaType::B), 2);
    let theirs = t.battlefield(P1, "Swamp");
    assert!(tap_for_mana(&mut t, P1, theirs, "Add"));
    assert_eq!(pool(&t, P1, ManaType::B), 2);
    // Blood Crypt (Swamp Mountain) is a Swamp: it adds an additional {B}.
    let crypt = t.battlefield(P0, "Blood Crypt");
    t.g.players[P0.idx()].mana_pool.empty();
    assert!(tap_for_mana(&mut t, P0, crypt, "Add"));
    assert_eq!(t.g.player(P0).mana_pool.total(), 2);
    assert!(pool(&t, P0, ManaType::B) >= 1);
    // A land that isn't a Swamp gets nothing extra.
    let mountain = t.battlefield(P0, "Mountain");
    t.g.players[P0.idx()].mana_pool.empty();
    assert!(tap_for_mana(&mut t, P0, mountain, "Add"));
    assert_eq!(t.g.player(P0).mana_pool.total(), 1);
}

#[test]
fn infernal_harvest_returns_x_swamps_once_as_it_is_cast() {
    cr!("601.2b", "601.2h", "107.3a");
    ruling!(
        "Infernal Harvest",
        "The return of X of your Swamps to your hand is part of the cost and is paid on casting."
    );
    ruling!("Infernal Harvest", "You can use X as being zero.");
    supported("Infernal Harvest");
    // X = 2: two Swamps return as the spell is cast; 2 damage divided between two
    // creatures.
    let mut t = TestGame::new(2);
    let swamps = t.lands(P0, "Swamp", 3);
    t.lands(P0, "Mountain", 1);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    let card = t.hand(P0, "Infernal Harvest");
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.answer_choose(P0, &[obj(swamps[1]), obj(swamps[2])]);
    t.answer_targets(P0, &[obj(a), obj(b)]);
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![1, 1]));
    t.cast_with(P0, card, &[]).expect("cast Infernal Harvest");
    // Paid on casting: the Swamps are back in hand while the spell is on the stack.
    assert_eq!(t.stack_len(), 1);
    assert_eq!(t.hand_size(P0), 2);
    assert!(t.on_battlefield(swamps[0]));
    t.resolve_all();
    assert_eq!(damage_on(&t, a), 1);
    assert_eq!(damage_on(&t, b), 1);
    // X = 0: nothing is returned, and the spell deals no damage.
    let mut t = TestGame::new(2);
    let swamps = t.lands(P0, "Swamp", 2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let card = t.hand(P0, "Infernal Harvest");
    t.answer(P0, DecisionKind::X, Answer::Number(0));
    t.cast_with(P0, card, &[]).expect("cast Infernal Harvest for X = 0");
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Infernal Harvest"));
    assert!(swamps.iter().all(|s| t.on_battlefield(*s)));
    assert_eq!(t.hand_size(P0), 0);
    assert_eq!(damage_on(&t, bears), 0);
}
