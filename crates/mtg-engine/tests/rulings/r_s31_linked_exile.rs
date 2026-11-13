//! Rulings batch S31 — linked abilities and "the exiled card" when the exiling triggered
//! ability triggers an additional time (CR 603.2d): every instance is linked to the other
//! ability, which then refers to all the cards they exiled (CR 607.2a, 607.3).

use crate::r_s01_common::{attack_with, block_and_finish, supported, tokens, with_subtype};
use crate::r_s02_common::destroy;
use crate::r_s04_common::add_mana;
use crate::r_s06_common::activate_containing;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

/// With `doubler` on P0's battlefield, Phantom Steed ("When this creature enters, exile
/// another target creature you control until this creature leaves the battlefield.
/// Whenever this creature attacks, create a tapped and attacking token that's a copy of
/// the exiled card, except it's an Illusion in addition to its other types. Sacrifice that
/// token at end of combat.") enters, its enters ability triggers twice and exiles
/// Grizzly Bears and Hill Giant; it attacks and makes a token copy of each.
fn doubled_phantom_steed(doubler: &str) {
    supported(doubler);
    supported("Phantom Steed");
    let mut t = TestGame::new(2);
    t.battlefield(P0, doubler);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.answer_targets(P0, &[Entity::Object(giant)]);
    let steed = t.enter(P0, "Phantom Steed");
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Exile);
    assert_eq!(t.zone(giant), Zone::Exile);
    t.g.objects[steed.0 as usize].summoning_sick = false;
    attack_with(&mut t, &[(steed, Entity::Player(P1))]);
    t.resolve_all();
    // "The exiled card" is both cards: a token copy of each.
    let mut names: Vec<String> = with_subtype(&t, P0, "Illusion")
        .into_iter()
        .filter(|i| t.obj_now(*i).is_token())
        .map(|i| t.obj_now(i).chars.name.to_string())
        .collect();
    names.sort();
    assert_eq!(names, vec!["Grizzly Bears", "Hill Giant"]);
    block_and_finish(&mut t, P1, &[]);
    t.resolve_all();
    assert!(tokens(&t, P0).is_empty());
    // Both return when Phantom Steed leaves.
    destroy(&mut t, steed);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
}

#[test]
fn panharmonicon_links_both_instances_to_the_exiled_card() {
    cr!("603.2d", "607.2a", "607.3");
    ruling!(
        "Panharmonicon",
        "If a triggered ability is linked to a second ability, additional instances of that triggered ability are also linked to that second ability. If the second ability refers to \"the exiled card,\" it refers to all cards exiled by instances of the triggered ability."
    );
    doubled_phantom_steed("Panharmonicon");
}

#[test]
fn starfield_vocalist_links_both_instances_to_the_exiled_card() {
    cr!("603.2d", "607.2a", "607.3");
    ruling!(
        "Starfield Vocalist",
        "If a triggered ability is linked to a second ability, additional instances of that triggered ability are also linked to that second ability. If the second ability refers to “the exiled card,” it refers to all cards exiled by instances of the triggered ability."
    );
    doubled_phantom_steed("Starfield Vocalist");
}

#[test]
fn isochron_scepter_copies_both_cards_its_doubled_imprint_exiled() {
    cr!("603.2d", "607.2a", "607.3", "707.12");
    ruling!(
        "Panharmonicon",
        "In some cases involving linked abilities, an ability requires information about \"the exiled card.\" When this happens, the ability gets multiple answers."
    );
    supported("Panharmonicon");
    supported("Isochron Scepter");
    // Isochron Scepter: "Imprint — When this artifact enters, you may exile an instant
    // card with mana value 2 or less from your hand. {2}, {T}: You may copy the exiled
    // card. If you do, you may cast the copy without paying its mana cost."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Panharmonicon");
    let bolt = t.hand(P0, "Lightning Bolt");
    let shock = t.hand(P0, "Shock");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(bolt)]);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(shock)]);
    let scepter = t.enter(P0, "Isochron Scepter");
    t.resolve_all();
    assert_eq!(t.zone(bolt), Zone::Exile);
    assert_eq!(t.zone(shock), Zone::Exile);
    t.clear_answers();
    // Activating it copies both cards; P0 casts both copies (at P1).
    add_mana(&mut t, P0, ManaType::C, 2);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    activate_containing(&mut t, P0, scepter, "copy").expect("activate");
    t.resolve_all();
    assert_eq!(t.life(P1), 15);
    // The cards stay exiled.
    assert_eq!(t.zone(bolt), Zone::Exile);
    assert_eq!(t.zone(shock), Zone::Exile);
}

#[test]
fn a_doubled_champion_ability_returns_every_card_it_exiled() {
    cr!("603.2d", "607.2a", "702.72a");
    ruling!(
        "Chief of the Wilds",
        "In some cases involving linked abilities, an ability requires information about \"the exiled card.\" When this happens, the ability gets multiple answers."
    );
    supported("Chief of the Wilds");
    supported("Changeling Hero");
    // Chief of the Wilds: "If a triggered ability of another Wolf or battle you control
    // triggers, that ability triggers an additional time." Changeling Hero (every creature
    // type, so a Wolf): "Champion a creature (When this enters, sacrifice it unless you
    // exile another creature you control. When this leaves the battlefield, that card
    // returns to the battlefield.)" (CR 702.72a: "... return the exiled card to the
    // battlefield under its owner's control.")
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Chief of the Wilds");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(giant)]);
    let hero = t.enter(P0, "Changeling Hero");
    t.resolve_all();
    assert!(t.on_battlefield(hero));
    assert_eq!(t.zone(bears), Zone::Exile);
    assert_eq!(t.zone(giant), Zone::Exile);
    // When it leaves, "the exiled card" is both cards: both return.
    destroy(&mut t, hero);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
}
