//! Rulings batch P167 — cards that care about Forests: reveal lands and check lands that
//! look for land types (CR 205.3i, 614.12), Shifting Woodland's copy effect (CR 707.2,
//! 707.5), Quirion Ranger's cost, and counts of Forests.

use crate::r_p194_common::mana;
use crate::r_s01_common::*;
use crate::r_s02_common::{destroy, target_candidates};
use crate::r_s04_common::{activate_named, is_creature};
use crate::r_s05_common::tokens_with_subtype;
use crate::r_s31_common::{entered_tapped, put_together, revealed_cards};
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

/// (reveal land, a basic land card with one of the types it asks for).
const REVEAL_LANDS: [(&str, &str); 2] = [("Fortified Village", "Forest"), ("Game Trail", "Mountain")];

#[test]
fn a_land_entering_from_hand_at_the_same_time_can_be_revealed() {
    cr!("614.12", "614.12a", "614.13a");
    ruling!(
        "Fortified Village",
        "If a Forest or Plains is entering the battlefield from your hand at the same time as Fortified Village, you may reveal the other land to have Fortified Village enter untapped."
    );
    ruling!(
        "Game Trail",
        "If a Mountain or Forest is entering the battlefield from your hand at the same time as Game Trail, you may reveal the other land to have Game Trail enter untapped."
    );
    for (name, basic) in REVEAL_LANDS {
        supported(name);
        let mut t = TestGame::new(2);
        let land = t.hand(P0, name);
        let other = t.hand(P0, basic);
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[obj(other)]);
        put_together(&mut t, &[land, other]);
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
        "Fortified Village",
        "If an effect instructs you to put Fortified Village onto the battlefield tapped, it will still enter the battlefield tapped even if you reveal a land card from your hand."
    );
    ruling!(
        "Game Trail",
        "If an effect instructs you to put Game Trail onto the battlefield tapped, it will still enter the battlefield tapped even if you reveal a land card from your hand."
    );
    supported("Hour of Promise");
    for (name, basic) in REVEAL_LANDS {
        // Hour of Promise: "Search your library for up to two land cards, put them onto
        // the battlefield tapped, then shuffle. ..."
        let mut t = TestGame::new(2);
        let land = t.library_top(P0, name);
        let other = t.hand(P0, basic);
        mana(&mut t, P0, ManaType::G, 1);
        mana(&mut t, P0, ManaType::C, 4);
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
        "Fortified Village",
        "Lands don't have a subtype just because they can produce mana of the corresponding color. Fortified Village itself is neither a Forest nor a Plains, even though it produces green and white mana, so you can't reveal one to satisfy the ability of another."
    );
    ruling!(
        "Game Trail",
        "Lands don't have a subtype just because they can produce mana of the corresponding color. Game Trail itself is neither a Mountain nor a Forest, even though it produces red and green mana, so you can't reveal one to satisfy the ability of another."
    );
    for (name, _) in REVEAL_LANDS {
        let mut t = TestGame::new(2);
        let land = t.hand(P0, name);
        let copy = t.hand(P0, name);
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[obj(copy)]);
        put_together(&mut t, &[land]);
        assert!(!revealed_cards(&t).contains(&copy), "{name}");
        assert!(entered_tapped(&t, land), "{name}");
        assert!(!t.obj_now(land).chars.has_subtype("Forest"));
    }
}

#[test]
fn any_land_card_with_the_subtype_can_be_revealed() {
    cr!("614.12", "205.3i");
    ruling!(
        "Game Trail",
        "You may reveal any land card with either or both of the appropriate subtypes. It doesn't have to be a basic land. For example, you could reveal Canopy Vista from the Battle for Zendikar set to satisfy the ability of Game Trail."
    );
    ruling!(
        "Fortified Village",
        "You may reveal any land card with either or both of the appropriate subtypes. It doesn't have to be a basic land. For example, you could reveal Prairie Stream from the Battle for Zendikar set to satisfy the ability of Fortified Village."
    );
    // Canopy Vista (Forest Plains); Prairie Stream (Plains Island).
    for (name, nonbasic) in [
        ("Game Trail", "Canopy Vista"),
        ("Fortified Village", "Prairie Stream"),
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
        "Sunpetal Grove",
        "This checks for lands you control with the land type Forest or Plains, not for lands named Forest or Plains. The lands it checks for don't have to be basic lands. For example, if you control Hallowed Fountain (a nonbasic land with the land types Plains and Island), Sunpetal Grove will enter untapped."
    );
    ruling!(
        "Rootbound Crag",
        "This checks for lands you control with the land type Mountain or Forest, not for lands named Mountain or Forest. The lands it checks for don't have to be basic lands. For example, if you control Temple Garden (a nonbasic land with the land types Forest and Plains), Rootbound Crag will enter untapped."
    );
    supported("Rootbound Crag");
    for (name, typed, other) in [
        ("Sunpetal Grove", "Hallowed Fountain", "Island"),
        ("Rootbound Crag", "Temple Garden", "Plains"),
    ] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, typed);
        let land = t.enter(P0, name);
        assert!(!entered_tapped(&t, land), "{name}");
        // A land without those types doesn't count.
        let mut t = TestGame::new(2);
        t.battlefield(P0, other);
        let land = t.enter(P0, name);
        assert!(entered_tapped(&t, land), "{name}");
    }
}

// ---------------------------------------------------------------------------------------
// Shifting Woodland
// ---------------------------------------------------------------------------------------

#[test]
fn shifting_woodland_needs_a_forest_already_there() {
    cr!("614.12", "614.1d", "603.6a");
    ruling!(
        "Shifting Woodland",
        "You must already control a Forest as Shifting Woodland enters the battlefield for it to enter untapped. If it enters the battlefield at the same time as a Forest when you control no other Forests, it will enter tapped."
    );
    let mut t = TestGame::new(2);
    let w = t.hand(P0, "Shifting Woodland");
    let f = t.hand(P0, "Forest");
    put_together(&mut t, &[w, f]);
    assert!(entered_tapped(&t, w));
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Forest");
    let w = t.enter(P0, "Shifting Woodland");
    assert!(!entered_tapped(&t, w));
}

/// Shifting Woodland on the battlefield with delirium (an instant, a sorcery and a land
/// card in P0's graveyard plus `card`), and the mana for its ability. Returns the
/// Woodland and the card.
fn woodland_with(t: &mut TestGame, card: &str) -> (ObjectId, ObjectId) {
    let w = t.battlefield(P0, "Shifting Woodland");
    t.graveyard(P0, "Shock");
    t.graveyard(P0, "Divination");
    t.graveyard(P0, "Mountain");
    let c = t.graveyard(P0, card);
    mana(t, P0, ManaType::G, 2);
    mana(t, P0, ManaType::C, 2);
    (w, c)
}

fn shift(t: &mut TestGame, w: ObjectId, target: ObjectId) {
    t.answer_targets(P0, &[obj(target)]);
    crate::r_s06_common::activate_containing(t, P0, w, "copy")
        .expect("Shifting Woodland's ability");
    t.resolve_all();
}

#[test]
fn shifting_woodland_keeps_its_effects_and_counters() {
    cr!("707.2", "613.1a", "122.1");
    ruling!(
        "Shifting Woodland",
        "Any effects that applied to Shifting Woodland before it becomes a copy of another card will continue to apply after it becomes a copy. The same is true of any counters that are on Shifting Woodland."
    );
    let mut t = TestGame::new(2);
    let (w, bears) = woodland_with(&mut t, "Grizzly Bears");
    t.g.add_counters(obj(w), counters::PLUS1, 1, None);
    // Heroic Intervention: "Permanents you control gain hexproof and indestructible
    // until end of turn."
    let hi = t.hand(P0, "Heroic Intervention");
    mana(&mut t, P0, ManaType::G, 1);
    mana(&mut t, P0, ManaType::C, 1);
    t.cast(P0, hi).go();
    t.resolve_all();
    shift(&mut t, w, bears);
    assert_eq!(t.obj_now(w).chars.name, "Grizzly Bears");
    assert!(is_creature(&t, w));
    assert_eq!(t.pt(w), (3, 3));
    assert!(t
        .obj_now(w)
        .has_keyword(mtg_engine::keywords::KeywordKind::Indestructible));
}

#[test]
fn shifting_woodland_doesnt_enter_so_enters_abilities_dont_apply() {
    cr!("707.5", "603.6a", "614.1c");
    ruling!(
        "Shifting Woodland",
        "Because Shifting Woodland isn't entering the battlefield when it becomes a copy of a card, any \"When [this creature] enters the battlefield\" or \"[This creature] enters the battlefield with\" abilities of the copied card won't apply."
    );
    // Elvish Visionary: "When this creature enters, draw a card."
    let mut t = TestGame::new(2);
    let (w, elf) = woodland_with(&mut t, "Elvish Visionary");
    let hand = t.hand_size(P0);
    shift(&mut t, w, elf);
    assert_eq!(t.obj_now(w).chars.name, "Elvish Visionary");
    assert_eq!(t.hand_size(P0), hand);
    // Yorvo: "enters with four +1/+1 counters": a 0/0 that dies.
    let mut t = TestGame::new(2);
    let (w, yorvo) = woodland_with(&mut t, "Yorvo, Lord of Garenbrig");
    shift(&mut t, w, yorvo);
    assert!(!t.on_battlefield(w));
    assert!(t.in_graveyard(P0, "Shifting Woodland"));
}

#[test]
fn shifting_woodland_copies_only_the_printed_card() {
    cr!("707.2", "400.7");
    ruling!(
        "Shifting Woodland",
        "Shifting Woodland copies exactly what was printed on the original card and nothing else. It doesn't copy any information about the object the card was before it was put into your graveyard."
    );
    // A Clone that was a copy of a Hill Giant: the card is a Clone (a 0/0 that dies).
    let mut t = TestGame::new(2);
    let w = t.battlefield(P0, "Shifting Woodland");
    t.graveyard(P0, "Shock");
    t.graveyard(P0, "Divination");
    t.graveyard(P0, "Mountain");
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(giant)]);
    let clone = crate::r_s05_common::enter(&mut t, P0, "Clone");
    t.resolve_all();
    assert_eq!(t.obj_now(clone).chars.name, "Hill Giant");
    destroy(&mut t, clone);
    let card = t.g.current(clone);
    assert_eq!(t.zone(card), Zone::Graveyard(P0));
    // A Grizzly Bears that had a +1/+1 counter: a 2/2.
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.add_counters(obj(bears), counters::PLUS1, 1, None);
    destroy(&mut t, bears);
    let bears_card = t.g.current(bears);
    mana(&mut t, P0, ManaType::G, 2);
    mana(&mut t, P0, ManaType::C, 2);
    shift(&mut t, w, bears_card);
    assert_eq!(t.pt(w), (2, 2));
    // Another Woodland becoming the Clone card: a 0/0 Clone, put into the graveyard.
    let w2 = t.battlefield(P0, "Shifting Woodland");
    mana(&mut t, P0, ManaType::G, 2);
    mana(&mut t, P0, ManaType::C, 2);
    shift(&mut t, w2, card);
    assert!(!t.on_battlefield(w2));
}

// ---------------------------------------------------------------------------------------
// Forests counted and returned
// ---------------------------------------------------------------------------------------

#[test]
fn howl_of_the_night_pack_counts_forests_as_it_resolves() {
    cr!("205.3i", "608.2h");
    ruling!(
        "Howl of the Night Pack",
        "Howl of the Night Pack counts any land you control with the subtype Forest, not just ones named Forest."
    );
    ruling!(
        "Howl of the Night Pack",
        "The number of Wolves you put onto the battlefield is based on the number of Forests you control when Howl of the Night Pack resolves."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Forest");
    t.battlefield(P0, "Temple Garden");
    t.battlefield(P0, "Plains");
    t.battlefield(P1, "Forest");
    let howl = t.hand(P0, "Howl of the Night Pack");
    mana(&mut t, P0, ManaType::G, 1);
    mana(&mut t, P0, ManaType::C, 6);
    t.cast(P0, howl).go();
    // A third Forest arrives before it resolves.
    t.battlefield(P0, "Canopy Vista");
    t.resolve_all();
    assert_eq!(tokens_with_subtype(&t, P0, "Wolf").len(), 3);
}

#[test]
fn llanowar_druid_untaps_every_forest() {
    cr!("205.3i", "701.26b");
    ruling!(
        "Llanowar Druid",
        "Llanowar Druid’s ability untaps all Forests on the battlefield, not just the ones you control."
    );
    let mut t = TestGame::new(2);
    let druid = t.battlefield(P0, "Llanowar Druid");
    let mine = t.battlefield(P0, "Forest");
    let theirs = t.battlefield(P1, "Forest");
    let garden = t.battlefield(P1, "Temple Garden");
    let mountain = t.battlefield(P1, "Mountain");
    for l in [mine, theirs, garden, mountain] {
        t.g.tap(l);
    }
    t.activate(P0, druid, 0, &[]).unwrap();
    t.resolve_all();
    for l in [mine, theirs, garden] {
        assert!(!t.obj_now(l).tapped);
    }
    assert!(t.obj_now(mountain).tapped);
}

#[test]
fn loam_lion_gets_the_bonus_once() {
    cr!("611.3a", "613.4c");
    ruling!(
        "Loam Lion",
        "Loam Lion either gets +1/+2 or it doesn’t. It doesn’t get the bonus for each Forest you control."
    );
    for (forests, pt) in [(0, (1, 1)), (1, (2, 3)), (3, (2, 3))] {
        let mut t = TestGame::new(2);
        let lion = t.battlefield(P0, "Loam Lion");
        t.lands(P0, "Forest", forests);
        assert_eq!(t.pt(lion), pt, "{forests}");
    }
}

#[test]
fn quirion_ranger_returns_any_forest_as_a_cost_to_untap_any_creature() {
    cr!("602.2b", "601.2h", "205.3i", "115.1");
    ruling!(
        "Quirion Ranger",
        "Returning the Forest you control to its owner's hand is the cost to activate the ability. Once you activate the ability, no one can try to do anything to the Forest to stop you from activating the ability."
    );
    ruling!(
        "Quirion Ranger",
        "You can target any creature with Quirion Ranger's ability, not just a tapped creature."
    );
    ruling!(
        "Quirion Ranger",
        "You may return any land you control with the subtype Forest. It doesn't have to be one named Forest."
    );
    let mut t = TestGame::new(2);
    let ranger = t.battlefield(P0, "Quirion Ranger");
    let garden = t.battlefield(P0, "Temple Garden");
    let bears = t.battlefield(P1, "Grizzly Bears");
    // An untapped opponent's creature is a legal target.
    let from = t.asked().len();
    t.answer_choose(P0, &[obj(garden)]);
    t.activate(P0, ranger, 0, &[obj(bears)]).unwrap();
    assert!(target_candidates(&t, P0, from)
        .iter()
        .any(|c| c.contains(&obj(bears)) && c.contains(&obj(ranger))));
    // The Forest is back in its owner's hand before anyone can respond.
    assert!(t.in_hand(P0, "Temple Garden"));
    assert!(t.asked()[from..].iter().all(|(p, _)| *p == P0));
    t.resolve_all();
    assert!(!t.obj_now(bears).tapped);
}
