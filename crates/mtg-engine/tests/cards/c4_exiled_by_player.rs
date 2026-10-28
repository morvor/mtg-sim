//! Cards a player exiled (round-4 card flow): permissions over "cards you exiled" (Haldan,
//! Avid Arcanist), cards "exiled by an ability you controlled" (Evelyn, the Covetous), and
//! "cards they exiled with ~" (Uba Mask, Shared Fate: each player's own share of the cards
//! linked to the source, CR 607.2a), fed by draw replacements that exile cards instead.

use crate::basic_effects_common::*;
use mtg_engine::events::MoveCause;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Puts `card` into exile as if `by` had exiled it, with `n` counters of `kind`.
fn exile_by(t: &mut TestGame, card: ObjectId, by: PlayerId, kind: &str) -> ObjectId {
    let x = t.g.move_object(card, Zone::Exile, MoveCause::Exile, Some(by)).unwrap();
    t.g.obj_mut(x).counters.insert(kind.into(), 1);
    x
}

#[test]
fn haldan_plays_noncreature_cards_pako_exiled_from_any_library() {
    cr!("601.3", "609.4b");
    ruling!(
        "Haldan, Avid Arcanist",
        "You'll be able to play cards you exiled with fetch counters before Haldan entered"
    );
    assert_supported("Haldan, Avid Arcanist");
    assert_supported("Pako, Arcane Retriever");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Haldan, Avid Arcanist");
    let pako = t.battlefield(P0, "Pako, Arcane Retriever");
    t.advance_to(P0, Step::BeginningOfCombat);
    let bolt = t.library_top(P1, "Lightning Bolt");
    let bears = t.library_top(P0, "Grizzly Bears");
    t.attack(&[(pako, Entity::Player(P1))], &[]);
    let bolt = t.g.current(bolt);
    let bears = t.g.current(bears);
    assert_eq!(t.zone(bolt), Zone::Exile);
    assert_eq!(t.counters(bolt, "fetch"), 1);
    // The Bolt P1 owns, with mana of any color (a Forest); not the creature card.
    t.advance_to(P0, Step::PostcombatMain);
    t.lands(P0, "Forest", 3);
    assert!(t.cast_with(P0, bears, &[]).is_err());
    t.cast(P0, bolt).target(Entity::Player(P1)).go();
    t.resolve();
    assert_eq!(t.life(P1), 20 - 4 - 3);
}

#[test]
fn haldan_doesnt_play_cards_another_player_exiled() {
    cr!("601.3");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Haldan, Avid Arcanist");
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    let bolt = exile_by(&mut t, bolt, P1, "fetch");
    assert!(t.cast_with(P0, bolt, &[Entity::Player(P1)]).is_err());
    let mine = t.hand(P1, "Lightning Bolt");
    let mine = exile_by(&mut t, mine, P0, "fetch");
    t.cast(P0, mine).target(Entity::Player(P1)).go();
    t.resolve();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn evelyn_plays_one_card_exiled_by_her_controllers_ability_each_turn() {
    cr!("601.3", "609.4b");
    assert_supported("Evelyn, the Covetous");
    let mut t = TestGame::new(2);
    let bolt = t.library_top(P1, "Lightning Bolt");
    let shock = t.library_top(P0, "Shock");
    t.enter(P0, "Evelyn, the Covetous");
    t.resolve_all();
    let (bolt, shock) = (t.g.current(bolt), t.g.current(shock));
    assert_eq!(t.counters(bolt, "collection"), 1);
    t.lands(P0, "Forest", 2);
    t.cast(P0, bolt).target(Entity::Player(P1)).go();
    t.resolve();
    assert_eq!(t.life(P1), 17);
    // Once each turn.
    assert!(t.cast_with(P0, shock, &[Entity::Player(P1)]).is_err());
    // A card with a collection counter an opponent's ability exiled isn't hers.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Evelyn, the Covetous");
    t.lands(P0, "Mountain", 1);
    let other = t.hand(P1, "Lightning Bolt");
    let other = exile_by(&mut t, other, P1, "collection");
    assert!(t.cast_with(P0, other, &[Entity::Player(P1)]).is_err());
}

#[test]
fn uba_mask_exiles_draws_and_each_player_plays_their_own_this_turn() {
    cr!("614.6", "607.2a", "601.3");
    ruling!("Uba Mask", "You can’t play cards you exiled with Uba Mask on previous turns");
    assert_supported("Uba Mask");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Uba Mask");
    t.library_top(P0, "Lightning Bolt");
    t.library_top(P1, "Shock");
    t.lands(P0, "Mountain", 2);
    let hand = t.hand_size(P0);
    t.g.draw_cards(P0, 1);
    t.g.draw_cards(P1, 1);
    assert_eq!(t.hand_size(P0), hand);
    assert!(t.in_exile("Lightning Bolt") && t.in_exile("Shock"));
    let bolt = t.g.find_in_zone(Zone::Exile, "Lightning Bolt")[0];
    let shock = t.g.find_in_zone(Zone::Exile, "Shock")[0];
    assert!(!t.g.obj(bolt).face_down);
    // P0 plays the card P0 exiled, not the one P1 exiled.
    assert!(t.cast_with(P0, shock, &[Entity::Player(P1)]).is_err());
    t.cast(P0, bolt).target(Entity::Player(P1)).go();
    t.resolve();
    assert_eq!(t.life(P1), 17);
    // On a later turn, the card P1 exiled can't be played any more.
    t.advance_to(P1, Step::PrecombatMain);
    t.lands(P1, "Mountain", 1);
    assert!(t.cast_with(P1, shock, &[Entity::Player(P0)]).is_err());
}

#[test]
fn shared_fate_exiles_an_opponents_top_card_face_down_for_the_drawing_player() {
    cr!("614.6", "406.3", "607.2a");
    ruling!("Shared Fate", "Replacing your draws isn't optional");
    assert_supported("Shared Fate");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Shared Fate");
    t.library_top(P0, "Shock");
    let bolt = t.library_top(P1, "Lightning Bolt");
    let hand = t.hand_size(P0);
    t.g.draw_cards(P0, 1);
    t.settle();
    assert_eq!(t.hand_size(P0), hand);
    let bolt = t.g.current(bolt);
    assert_eq!(t.zone(bolt), Zone::Exile);
    assert!(t.g.obj(bolt).face_down);
    // P0 exiled it: P0 may look at it and cast it; P1 (its owner) may not.
    assert!(mtg_engine::zones::may_look(&t.g, P0, bolt));
    t.lands(P1, "Mountain", 1);
    assert!(t.cast_with(P1, bolt, &[Entity::Player(P0)]).is_err());
    t.lands(P0, "Mountain", 1);
    t.cast(P0, bolt).target(Entity::Player(P1)).go();
    t.resolve();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn gonti_night_minister_the_attacking_creatures_controller_exiles_and_may_play() {
    cr!("406.3", "118.14", "601.3");
    assert_supported("Gonti, Night Minister");
    // P1's creature deals combat damage to P2, an opponent of Gonti's controller P0.
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Gonti, Night Minister");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.advance_to(P1, Step::BeginningOfCombat);
    let bolt = t.library_top(P2, "Lightning Bolt");
    t.attack(&[(bears, Entity::Player(P2))], &[]);
    let bolt = t.g.current(bolt);
    assert_eq!(t.zone(bolt), Zone::Exile);
    assert!(t.g.obj(bolt).face_down);
    // P1 looked at it and may play it; mana of any type (a Forest) can be spent.
    assert!(mtg_engine::zones::may_look(&t.g, P1, bolt));
    assert!(!mtg_engine::zones::may_look(&t.g, P0, bolt));
    t.advance_to(P1, Step::PostcombatMain);
    t.lands(P0, "Mountain", 1);
    assert!(t.cast_with(P0, bolt, &[Entity::Player(P2)]).is_err());
    t.lands(P1, "Forest", 1);
    t.cast(P1, bolt).target(Entity::Player(P2)).go();
    t.resolve_all();
    assert_eq!(t.life(P2), 20 - 2 - 3);
}
