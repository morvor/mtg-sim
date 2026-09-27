//! Rulings batch S05 — devoid (CR 702.114) and the Eldrazi Scion tokens and ingest
//! (CR 702.115) of the same sets.
//!
//! The curly-apostrophe versions of the devoid rulings are cited with Forerunner of
//! Slaughter by the keyword tests; these are the straight-apostrophe versions.

use crate::r_s01_common::*;
use crate::r_s04_common::*;
use crate::r_s05_common::*;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn a_card_with_devoid_is_only_colorless() {
    cr!("702.114a", "105.2c");
    ruling!(
        "Brood Butcher",
        "A card with devoid is just colorless. It's not colorless and the colors of mana in its mana cost."
    );
    supported("Brood Butcher");
    supported("Doom Blade");
    // Brood Butcher ({3}{B}{G}) is colorless in every zone, not black and green.
    let mut t = TestGame::new(2);
    for zone in [Zone::Hand(P0), Zone::Graveyard(P0), Zone::Library(P0)] {
        let id = place(&mut t, P0, "Brood Butcher", zone);
        assert_eq!(colors(&t, id), ColorSet::NONE, "in {zone:?}");
    }
    let butcher = t.battlefield(P0, "Brood Butcher");
    assert!(colorless(&t, butcher));
    assert!(!colors(&t, butcher).contains(Color::Black));
    // So "destroy target nonblack creature" can destroy it.
    t.lands(P1, "Swamp", 2);
    let blade = t.hand(P1, "Doom Blade");
    t.cast(P1, blade).target(butcher).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Brood Butcher"));
}

#[test]
fn a_card_that_loses_devoid_is_still_colorless() {
    cr!("702.114a", "613.1e", "613.1f");
    ruling!(
        "Eldrazi Skyspawner",
        "If a card loses devoid, it will still be colorless. This is because effects that change an object's color (like the one created by devoid) are considered before the object loses devoid."
    );
    supported("Eldrazi Skyspawner");
    supported("Humility");
    let mut t = TestGame::new(2);
    let skyspawner = t.battlefield(P0, "Eldrazi Skyspawner");
    t.battlefield(P1, "Humility");
    t.g.recompute();
    // It lost devoid (and flying): a 1/1 with no abilities, still colorless.
    assert!(t.obj_now(skyspawner).chars.abilities.is_empty());
    assert_eq!(t.pt(skyspawner), (1, 1));
    assert!(colorless(&t, skyspawner));
}

#[test]
fn a_card_with_devoid_given_a_color_is_just_that_color() {
    cr!("702.114a", "613.1e", "613.3");
    ruling!(
        "Emrakul's Messenger",
        "Other cards and abilities can give a card with devoid a color. If that happens, it's just the new color, not that color and colorless."
    );
    ruling!(
        "Blisterpod",
        "Other cards and abilities can give a card with devoid color. If that happens, it's just the new color, not that color and colorless."
    );
    supported("Cerulean Wisps");
    supported("Deathlace");
    let mut t = TestGame::new(2);
    // Cerulean Wisps: "Target creature becomes blue until end of turn."
    let messenger = t.battlefield(P0, "Emrakul's Messenger");
    t.lands(P0, "Island", 1);
    let wisps = t.hand(P0, "Cerulean Wisps");
    t.cast(P0, wisps).target(messenger).go();
    t.resolve_all();
    assert_eq!(colors(&t, messenger), ColorSet::single(Color::Blue));
    assert!(!colorless(&t, messenger));
    // Deathlace: "Target spell or permanent becomes black."
    let pod = t.battlefield(P0, "Blisterpod");
    t.lands(P0, "Swamp", 1);
    let lace = t.hand(P0, "Deathlace");
    t.cast(P0, lace).target(pod).go();
    t.resolve_all();
    assert_eq!(colors(&t, pod), ColorSet::single(Color::Black));
    // Once the Wisps' effect ends, the Messenger is colorless again.
    t.advance_to(P1, Step::Upkeep);
    assert!(colorless(&t, messenger));
    assert_eq!(colors(&t, pod), ColorSet::single(Color::Black));
}

/// Eldrazi Skyspawner enters under `p`'s control and its enters trigger resolves: the
/// Eldrazi Scion token it created.
fn skyspawner_scion(t: &mut TestGame, p: PlayerId) -> ObjectId {
    enter(t, p, "Eldrazi Skyspawner");
    t.resolve_all();
    let scions = tokens_with_subtype(t, p, "Scion");
    assert_eq!(scions.len(), 1);
    scions[0]
}

#[test]
fn eldrazi_scions_are_1_1() {
    cr!("111.1");
    ruling!(
        "Eldrazi Skyspawner",
        "Eldrazi Scions are similar to Eldrazi Spawn, seen in the Zendikar block. Note that Eldrazi Scions are 1/1, not 0/1."
    );
    let mut t = TestGame::new(2);
    let scion = skyspawner_scion(&mut t, P0);
    assert_eq!(t.pt(scion), (1, 1));
    let o = t.obj_now(scion);
    assert!(o.is_token() && o.chars.is(CardType::Creature));
    assert!(o.chars.colors.is_colorless());
}

#[test]
fn eldrazi_and_scion_are_separate_creature_types() {
    cr!("205.3m", "111.4");
    ruling!(
        "Eldrazi Skyspawner",
        "Eldrazi and Scion are each separate creature types. Anything that affects Eldrazi will affect these tokens, for example."
    );
    supported("Eldrazi Linebreaker");
    let mut t = TestGame::new(2);
    let scion = skyspawner_scion(&mut t, P0);
    let o = t.obj_now(scion);
    assert!(o.chars.has_subtype("Eldrazi") && o.chars.has_subtype("Scion"));
    assert_eq!(o.chars.name.as_str(), "Eldrazi Scion Token");
    // Eldrazi Linebreaker: "At the beginning of combat on your turn, target creature you
    // control gains haste and gets +X/+0 until end of turn, where X is the number of
    // Eldrazi you control." The Scion and the Skyspawner count, with the Linebreaker.
    let linebreaker = t.battlefield(P0, "Eldrazi Linebreaker");
    t.answer_targets(P0, &[Entity::Object(linebreaker)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert_eq!(t.pt(linebreaker), (3 + 3, 3));
}

#[test]
fn sacrificing_a_scion_for_mana_is_a_mana_ability() {
    cr!("605.1a", "605.3b");
    ruling!(
        "Eldrazi Skyspawner",
        "Sacrificing an Eldrazi Scion creature token to add {C} is a mana ability. It doesn't use the stack and can't be responded to."
    );
    let mut t = TestGame::new(2);
    let scion = skyspawner_scion(&mut t, P0);
    let o = t.obj_now(scion);
    assert!(o.chars.abilities.iter().any(|a| a.is_mana_ability()));
    // Activating it adds {C} at once: nothing goes on the stack, and no player gets
    // priority in between.
    let from = t.asked().len();
    let r = t.activate(P0, scion, 0, &[]);
    assert!(matches!(r, Ok(None)), "{r:?}");
    assert_eq!(t.stack_len(), 0);
    assert!(!t.on_battlefield(scion));
    assert_eq!(t.g.player(P0).mana_pool.count(ManaType::C), 1);
    assert_eq!(
        asked_of_since(&t, P1, from, |d| matches!(
            d,
            mtg_engine::decision::Decision::Priority { .. }
        )),
        0
    );
}

#[test]
fn a_scion_making_spell_whose_targets_are_all_illegal_makes_no_scions() {
    cr!("608.2b");
    ruling!(
        "Adverse Conditions",
        "Some instants and sorceries that create Eldrazi Scions require targets. If all targets for such a spell have become illegal by the time that spell tries to resolve, the spell won't resolve and none of its effects will happen. You won't get any Eldrazi Scions."
    );
    supported("Adverse Conditions");
    // Adverse Conditions: "Tap up to two target creatures. ... Create a 1/1 colorless
    // Eldrazi Scion creature token."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    give_mana_for(&mut t, P0, "Adverse Conditions");
    let spell = t.hand(P0, "Adverse Conditions");
    t.cast(P0, spell).targets(&[Entity::Object(bears)]).go();
    // The only target leaves the battlefield before it resolves.
    move_to(&mut t, bears, Zone::Hand(P1));
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Adverse Conditions"));
    assert!(tokens_with_subtype(&t, P0, "Scion").is_empty());
    // With its target still there, it makes a Scion.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    give_mana_for(&mut t, P0, "Adverse Conditions");
    let spell = t.hand(P0, "Adverse Conditions");
    t.cast(P0, spell).targets(&[Entity::Object(bears)]).go();
    t.resolve_all();
    assert!(t.obj_now(bears).tapped);
    assert_eq!(tokens_with_subtype(&t, P0, "Scion").len(), 1);
}

#[test]
fn ingest_with_an_empty_library_does_nothing() {
    cr!("702.115a", "704.5b");
    ruling!(
        "Sludge Crawler",
        "If the player has no cards in their library when the ingest ability resolves, nothing happens. That player won't lose the game (until they have to draw a card from an empty library)."
    );
    supported("Sludge Crawler");
    let mut t = TestGame::new(2);
    let crawler = t.battlefield(P0, "Sludge Crawler");
    t.g.players[1].library.clear();
    attack_with(&mut t, &[(crawler, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.life(P1), 19);
    assert!(t.g.exile.is_empty());
    assert!(!t.has_lost(P1));
    // They lose when they would draw from the empty library.
    t.advance_to(P1, Step::Upkeep);
    assert!(!t.has_lost(P1));
    t.g.run_until(1000, |g| g.result.is_some());
    assert!(t.has_lost(P1));
}
