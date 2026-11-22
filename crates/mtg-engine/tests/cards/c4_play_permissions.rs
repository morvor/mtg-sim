//! Permissions with new parts (round-4 card flow): a permission used once over all the
//! cards an ability exiled (Kotose, the Silent Spider), "other cards exiled with ~"
//! (Knowledge Pool), ...

use crate::basic_effects_common::*;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn kotose_plays_one_of_all_the_cards_it_exiled() {
    cr!("601.3", "607.2a", "609.4b");
    ruling!("Kotose, the Silent Spider", "You may play exactly one of the exiled cards");
    assert_supported("Kotose, the Silent Spider");
    let mut t = TestGame::new(2);
    let target = t.graveyard(P1, "Lightning Bolt");
    let basic = t.graveyard(P1, "Forest");
    let own = t.graveyard(P0, "Shock");
    let in_hand = t.hand(P1, "Lightning Bolt");
    let in_lib = t.library_top(P1, "Lightning Bolt");
    t.answer_targets(P0, &[Entity::Object(target)]);
    t.answer_choose(P0, &[Entity::Object(in_hand), Entity::Object(in_lib)]);
    t.enter(P0, "Kotose, the Silent Spider");
    t.resolve_all();
    let exiled: Vec<ObjectId> = [target, in_hand, in_lib]
        .iter()
        .map(|o| t.g.current(*o))
        .collect();
    assert!(exiled.iter().all(|o| t.zone(*o) == Zone::Exile));
    // Only a card other than a basic land card in an opponent's graveyard was a target.
    assert_eq!(last_target_candidates(&t, P0), vec![Entity::Object(target)]);
    assert_eq!((t.zone(basic), t.zone(own)), (Zone::Graveyard(P1), Zone::Graveyard(P0)));
    // The target card (exiled by the first instruction) can be cast, with any mana.
    t.lands(P0, "Forest", 2);
    t.cast(P0, exiled[0]).target(Entity::Player(P1)).go();
    t.resolve();
    assert_eq!(t.life(P1), 17);
    // Only one of them.
    assert!(t.cast_with(P0, exiled[1], &[Entity::Player(P1)]).is_err());
}

#[test]
fn kotose_permission_covers_the_searched_cards_too() {
    cr!("601.3", "607.2a");
    let mut t = TestGame::new(2);
    let target = t.graveyard(P1, "Lightning Bolt");
    let in_hand = t.hand(P1, "Lightning Bolt");
    t.answer_targets(P0, &[Entity::Object(target)]);
    t.answer_choose(P0, &[Entity::Object(in_hand)]);
    t.enter(P0, "Kotose, the Silent Spider");
    t.resolve_all();
    let in_hand = t.g.current(in_hand);
    t.lands(P0, "Mountain", 1);
    t.cast(P0, in_hand).target(Entity::Player(P1)).go();
    t.resolve();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn knowledge_pool_casts_another_card_not_the_one_just_exiled() {
    cr!("607.2a", "601.3");
    ruling!("Knowledge Pool", "The spell a player casts from their hand won't resolve if it's exiled");
    assert_supported("Knowledge Pool");
    let mut t = TestGame::new(2);
    for _ in 0..3 {
        t.library_top(P0, "Grizzly Bears");
        t.library_top(P1, "Lightning Bolt");
    }
    t.enter(P0, "Knowledge Pool");
    t.resolve_all();
    assert_eq!(t.g.exile.len(), 6);
    let shock = t.hand(P0, "Shock");
    t.lands(P0, "Mountain", 1);
    t.cast(P0, shock).target(Entity::Player(P1)).go();
    // The cast trigger exiles Shock; the choice of a card to cast offers the others only.
    t.resolve();
    let shock_x = t.g.current(shock);
    assert_eq!(t.zone(shock_x), Zone::Exile);
    let offered = t.asked().into_iter().rev().find_map(|(p, d)| match d {
        decision::Decision::ChooseEntities { candidates, .. } if p == P0 => Some(candidates),
        _ => None,
    });
    let offered = offered.expect("a card to cast was offered");
    assert!(!offered.contains(&Entity::Object(shock_x)));
    assert_eq!(offered.len(), 6);
}

#[test]
fn share_the_spoils_playing_a_card_exiles_the_players_top_card() {
    cr!("601.3", "603.2", "607.2a");
    ruling!("Share the Spoils", "it's one land or one spell, not one of each");
    assert_supported("Share the Spoils");
    let mut t = TestGame::new(2);
    let bolt = t.library_top(P1, "Lightning Bolt");
    t.library_top(P0, "Forest");
    t.enter(P0, "Share the Spoils");
    t.resolve_all();
    let bolt = t.g.current(bolt);
    assert_eq!(t.zone(bolt), Zone::Exile);
    let next = t.library_top(P0, "Shock");
    t.lands(P0, "Forest", 1);
    t.cast(P0, bolt).target(Entity::Player(P1)).go();
    // "When they do": the player who cast it exiles their top card.
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.zone(t.g.current(next)), Zone::Exile);
    // One card a turn: the exiled Forest can't be played now.
    let forest = t.g.find_in_zone(Zone::Exile, "Forest")[0];
    assert!(t.play_land(P0, forest).is_err());
}

#[test]
fn aminatous_augury_one_free_spell_of_each_nonland_card_type() {
    cr!("601.3", "118.9");
    ruling!("Aminatou's Augury", "you may cast it as either of those types");
    assert_supported("Aminatou's Augury");
    let mut t = TestGame::new(2);
    let names = [
        "Ornithopter",
        "Lightning Bolt",
        "Shock",
        "Grizzly Bears",
        "Hill Giant",
        "Divination",
        "Forest",
        "Island",
    ];
    let ids: Vec<ObjectId> = names.iter().map(|n| t.library_top(P0, n)).collect();
    t.lands(P0, "Island", 8);
    let aug = t.hand(P0, "Aminatou's Augury");
    // Put the Forest onto the battlefield.
    t.answer_choose_moved(P0, &[Entity::Object(ids[6])]);
    t.cast(P0, aug).go();
    t.resolve();
    assert_eq!(t.named_on_battlefield("Forest").len(), 1);
    let cur = |t: &TestGame, i: usize| t.g.current(ids[i]);
    // An instant (Bolt), then not a second instant (Shock).
    let bolt = cur(&t, 1);
    t.cast(P0, bolt).method(CastMethod::Free).target(Entity::Player(P1)).go();
    t.resolve();
    assert_eq!(t.life(P1), 17);
    let shock = cur(&t, 2);
    assert!(t.cast_with(P0, shock, &[Entity::Player(P1)]).is_err());
    // A creature (Bears), then not another creature (Hill Giant).
    let bears = cur(&t, 3);
    t.cast(P0, bears).method(CastMethod::Free).go();
    t.resolve();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    let giant = cur(&t, 4);
    assert!(t.cast_with(P0, giant, &[]).is_err());
    // An artifact creature cast as the artifact (the creature's permission is used up).
    let thopter = cur(&t, 0);
    t.cast(P0, thopter).method(CastMethod::Free).go();
    t.resolve();
    assert_eq!(t.named_on_battlefield("Ornithopter").len(), 1);
    // A sorcery.
    let div = cur(&t, 5);
    t.cast(P0, div).method(CastMethod::Free).go();
    t.resolve();
    assert!(t.in_graveyard(P0, "Divination"));
}

fn talent_game(mastery: bool) -> (TestGame, Vec<ObjectId>) {
    let mut t = TestGame::new(2);
    if mastery {
        t.graveyard(P0, "Lightning Bolt");
        t.graveyard(P0, "Divination");
    }
    let mut ids = Vec::new();
    for n in ["Island", "Island", "Island", "Island", "Island"] {
        t.library_top(P1, n);
    }
    ids.push(t.library_top(P1, "Lightning Bolt"));
    ids.push(t.library_top(P1, "Lightning Bolt"));
    t.lands(P0, "Island", 4);
    (t, ids)
}

#[test]
fn talent_of_the_telepath_casts_two_with_spell_mastery() {
    cr!("608.2c", "118.9");
    ruling!("Talent of the Telepath", "you'll cast the two cards in order");
    assert_supported("Talent of the Telepath");
    let (mut t, ids) = talent_game(true);
    let talent = t.hand(P0, "Talent of the Telepath");
    t.answer_choose_moved(P0, &[Entity::Object(ids[0]), Entity::Object(ids[1])]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.cast(P0, talent).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 14);
    let max = t.asked().into_iter().find_map(|(_, d)| match d {
        decision::Decision::ChooseEntities { max, candidates, .. } if candidates.len() == 2 => {
            Some(max)
        }
        _ => None,
    });
    assert_eq!(max, Some(2));
}

#[test]
fn talent_of_the_telepath_casts_one_without_spell_mastery() {
    cr!("608.2c", "118.9");
    let (mut t, ids) = talent_game(false);
    let talent = t.hand(P0, "Talent of the Telepath");
    t.answer_choose_moved(P0, &[Entity::Object(ids[0])]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.cast(P0, talent).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    // Up to one card was offered.
    let max = t.asked().into_iter().find_map(|(_, d)| match d {
        decision::Decision::ChooseEntities { max, candidates, .. } if candidates.len() == 2 => {
            Some(max)
        }
        _ => None,
    });
    assert_eq!(max, Some(1));
}
