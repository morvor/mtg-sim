//! Object phrases over several zones (round-4 card flow): "~ from your graveyard or from
//! exile", "a card from your hand or graveyard", graveyard order ("the top two cards of
//! your graveyard", "the top three black cards of your graveyard"), a target list across
//! the stack, the battlefield and graveyards, a three-part exile list, and "which have"
//! qualifiers.

use crate::basic_effects_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn mister_immortal_returns_from_the_graveyard_or_from_exile() {
    cr!("113.6m");
    assert_supported("Mister Immortal");
    for from_exile in [false, true] {
        let mut t = TestGame::new(2);
        let m = if from_exile {
            t.exile(P0, "Mister Immortal")
        } else {
            t.graveyard(P0, "Mister Immortal")
        };
        t.lands(P0, "Forest", 3);
        t.activate(P0, m, 0, &[]).unwrap();
        t.resolve();
        let on = t.named_on_battlefield("Mister Immortal");
        assert_eq!(on.len(), 1, "from exile: {from_exile}");
        assert!(t.obj_now(on[0]).tapped);
    }
    // Not from the hand or the battlefield.
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 3);
    let m = t.hand(P0, "Mister Immortal");
    assert!(t.activate(P0, m, 0, &[]).is_err());
    let m = t.battlefield(P0, "Mister Immortal");
    assert!(t.activate(P0, m, 0, &[]).is_err());
}

#[test]
fn mairsil_exiles_a_card_from_hand_or_graveyard_with_a_cage_counter() {
    cr!("122.1");
    assert_supported("Mairsil, the Pretender");
    let mut t = TestGame::new(2);
    let in_hand = t.hand(P0, "Grizzly Bears");
    t.graveyard(P0, "Sol Ring");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(in_hand)]);
    t.enter(P0, "Mairsil, the Pretender");
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    assert!(t.in_graveyard(P0, "Sol Ring"));
    let bears = t.g.current(in_hand);
    assert_eq!(t.counters(bears, "cage"), 1);
    // The choice offered the cards in both zones.
    let offered = t.asked().into_iter().any(|(_, d)| {
        matches!(d, decision::Decision::ChooseEntities { candidates, .. } if candidates.len() == 2)
    });
    assert!(offered);
}

#[test]
fn moratorium_stone_exiles_the_card_its_namesakes_and_permanents_with_that_name() {
    cr!("201.2", "406.2");
    assert_supported("Moratorium Stone");
    let mut t = TestGame::new(2);
    let stone = t.battlefield(P0, "Moratorium Stone");
    let target = t.graveyard(P1, "Grizzly Bears");
    t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P0, "Hill Giant");
    t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Plains", 2);
    t.lands(P0, "Swamp", 2);
    t.activate(P0, stone, 1, &[Entity::Object(target)]).unwrap();
    t.resolve();
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
    assert!(!t.in_graveyard(P0, "Grizzly Bears") && !t.in_graveyard(P1, "Grizzly Bears"));
    assert!(t.in_graveyard(P0, "Hill Giant"));
}

#[test]
fn endless_detour_puts_a_spell_on_the_bottom_of_its_owners_library() {
    cr!("115.1", "401.4");
    ruling!("Endless Detour", "The card's owner chooses whether to put it on the top or bottom");
    ruling!("Endless Detour", "doesn't counter spells");
    assert_supported("Endless Detour");
    let mut t = TestGame::new(2);
    for _ in 0..3 {
        t.library_top(P1, "Island");
    }
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(Entity::Player(P0)).go();
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Island", 1);
    let d = t.hand(P0, "Endless Detour");
    // Bolt's owner (not the caster of Endless Detour) chooses: the bottom.
    t.answer(P1, DecisionKind::Option, Answer::Index(1));
    t.cast(P0, d).target(Entity::Object(bolt)).go();
    t.resolve();
    assert_eq!(t.stack_len(), 1 - 1);
    let lib = &t.g.player(P1).library;
    assert_eq!(t.g.obj(lib[0]).name(), "Lightning Bolt");
    assert_eq!(t.life(P0), 20);
    // A card in a graveyard and a nonland permanent are legal targets too; a land isn't.
    let mut t = TestGame::new(2);
    let gy = t.graveyard(P1, "Forest");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let land = t.battlefield(P1, "Mountain");
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Island", 1);
    let d = t.hand(P0, "Endless Detour");
    t.cast(P0, d).target(Entity::Object(gy)).go();
    let candidates = last_target_candidates(&t, P0);
    assert!(candidates.contains(&Entity::Object(bears)));
    assert!(!candidates.contains(&Entity::Object(land)));
    t.resolve();
    // The default choice is the top (the library's last card).
    assert_eq!(t.g.obj(*t.g.player(P1).library.last().unwrap()).name(), "Forest");
}

#[test]
fn spinning_darkness_exiles_the_top_three_black_cards_of_your_graveyard() {
    cr!("404.2", "118.9");
    ruling!("Spinning Darkness", "The “top” card of your graveyard is the card that was put there most recently");
    assert_supported("Spinning Darkness");
    let mut t = TestGame::new(2);
    // Bottom to top: a black card, then black, red, black, black.
    let deep = t.graveyard(P0, "Dark Ritual");
    for n in ["Duress", "Lightning Bolt", "Duress", "Dark Ritual"] {
        t.graveyard(P0, n);
    }
    let bears = t.battlefield(P1, "Grizzly Bears");
    let sd = t.hand(P0, "Spinning Darkness");
    let alt = t
        .cast_options(P0, sd)
        .into_iter()
        .map(|o| o.method)
        .find(|m| matches!(m, CastMethod::Alternative(_)))
        .expect("the alternative cost is offered");
    t.cast(P0, sd).method(alt).target(Entity::Object(bears)).go();
    t.resolve();
    assert!(!t.on_battlefield(bears));
    assert_eq!(t.life(P0), 23);
    // The three black cards on top went; the deepest black card and the red one stayed.
    assert!(t.in_graveyard(P0, "Lightning Bolt"));
    assert_eq!(t.zone(deep), Zone::Graveyard(P0));
    assert_eq!(t.graveyard_size(P0), 3);
}

#[test]
fn spinning_darkness_needs_three_black_cards() {
    cr!("404.2", "118.9");
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Duress");
    t.graveyard(P0, "Dark Ritual");
    t.graveyard(P0, "Lightning Bolt");
    let sd = t.hand(P0, "Spinning Darkness");
    let alt = t
        .cast_options(P0, sd)
        .into_iter()
        .map(|o| o.method)
        .find(|m| matches!(m, CastMethod::Alternative(_)));
    if let Some(alt) = alt {
        let bears = t.battlefield(P1, "Grizzly Bears");
        assert!(t.cast(P0, sd).method(alt).target(Entity::Object(bears)).try_go().is_err());
    }
    assert_eq!(t.graveyard_size(P0), 3);
}

#[test]
fn phyrexian_grimoire_opponent_chooses_one_of_the_top_two_cards() {
    cr!("404.2");
    ruling!("Phyrexian Grimoire", "It does not target the cards, but it targets the opponent");
    assert_supported("Phyrexian Grimoire");
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Hill Giant");
    let second = t.graveyard(P0, "Lightning Bolt");
    let top = t.graveyard(P0, "Grizzly Bears");
    let g = t.battlefield(P0, "Phyrexian Grimoire");
    t.lands(P0, "Swamp", 4);
    t.answer_choose(P1, &[Entity::Object(top)]);
    t.activate(P0, g, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve();
    assert!(t.in_exile("Grizzly Bears"));
    assert!(t.in_hand(P0, "Lightning Bolt"));
    assert!(t.in_graveyard(P0, "Hill Giant"));
    // The opponent chose between the top two only.
    let offered = t.asked().into_iter().find_map(|(p, d)| match d {
        decision::Decision::ChooseEntities { candidates, .. } if p == P1 => Some(candidates),
        _ => None,
    });
    let offered = offered.expect("the opponent chose");
    assert_eq!(offered.len(), 2);
    assert!(offered.contains(&Entity::Object(second)));
}

#[test]
fn phyrexian_grimoire_with_one_card_exiles_it() {
    cr!("404.2");
    ruling!("Phyrexian Grimoire", "If there is only one card in your graveyard");
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Grizzly Bears");
    let g = t.battlefield(P0, "Phyrexian Grimoire");
    t.lands(P0, "Swamp", 4);
    t.activate(P0, g, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve();
    assert!(t.in_exile("Grizzly Bears"));
    assert_eq!(t.hand_size(P0), 0);
}

#[test]
fn canoptek_wraith_finds_basic_lands_which_have_the_chosen_lands_name() {
    cr!("201.2", "701.23a");
    ruling!("Canoptek Wraith", "the two lands must have the same name as each other");
    assert_supported("Canoptek Wraith");
    let mut t = TestGame::new(2);
    let w = t.battlefield(P0, "Canoptek Wraith");
    let island = t.battlefield(P0, "Island");
    t.lands(P0, "Plains", 3);
    for n in ["Island", "Plains", "Island", "Swamp"] {
        t.library_top(P0, n);
    }
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(island)]);
    t.advance_to(P0, Step::DeclareAttackers);
    t.attack(&[(w, Entity::Player(P1))], &[]);
    t.advance_to_step(Step::CombatDamage);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert!(t.in_graveyard(P0, "Canoptek Wraith"));
    // The two Islands came in tapped (the Plains stayed in the library).
    let islands = t.named_on_battlefield("Island");
    assert_eq!(islands.len(), 3);
    let new: Vec<_> = islands.iter().filter(|i| **i != island).collect();
    assert!(new.iter().all(|i| t.obj_now(**i).tapped));
    let names: Vec<&str> = t.g.player(P0).library.iter().map(|i| t.g.obj(*i).name()).collect();
    assert!(!names.contains(&"Island"));
    assert!(names.contains(&"Plains"));
}
