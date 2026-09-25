//! CR 305: lands — playing lands (a special action), the number of land plays, "putting"
//! lands onto the battlefield, land types, and basic lands.

use crate::r300_common::*;
use mtg_engine::ability::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn playing_a_land_is_a_special_action_that_doesnt_use_the_stack() {
    cr!("305.1");
    let mut t = TestGame::new(2);
    let forest = t.hand(P0, "Forest");
    // Only in a main phase of the player's turn with an empty stack.
    t.set_step(P1, Step::PrecombatMain);
    assert!(!can_play_land(&mut t, P0, forest), "opponent's turn");
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!can_play_land(&mut t, P0, forest), "combat");
    assert!(t.play_land(P0, forest).is_err());
    t.set_step(P0, Step::PrecombatMain);
    hold_stack(&mut t, P0);
    assert!(!can_play_land(&mut t, P0, forest), "nonempty stack");
    t.resolve_all();
    // It's simply put onto the battlefield: it's never a spell, and no one gets priority
    // to respond to it.
    let events = t.turn_events.len();
    t.play_land(P0, forest).unwrap();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.zone(forest), Zone::Battlefield);
    assert_eq!(t.g.turn.priority, Some(P0));
    assert!(!t.turn_events[events..]
        .iter()
        .any(|e| matches!(e, mtg_engine::events::Event::SpellCast { .. })));
}

#[test]
fn a_player_can_normally_play_one_land_and_effects_may_add_more() {
    cr!("305.2");
    ruling!(
        "Explore",
        "The effects of multiple Explores in the same turn are cumulative"
    );
    let mut t = TestGame::new(2);
    let lands: Vec<ObjectId> = (0..4).map(|_| t.hand(P0, "Forest")).collect();
    t.play_land(P0, lands[0]).unwrap();
    assert!(t.play_land(P0, lands[1]).is_err());
    // Two Explores: two more land plays this turn.
    for _ in 0..2 {
        let e = t.hand(P0, "Explore");
        t.lands(P0, "Forest", 2);
        t.cast(P0, e).go();
        t.resolve();
    }
    t.play_land(P0, lands[1]).unwrap();
    t.play_land(P0, lands[2]).unwrap();
    assert!(t.play_land(P0, lands[3]).is_err());
    assert_eq!(t.player(P0).lands_played_this_turn, 3);
}

fn play_by_effect(t: &mut TestGame, p: PlayerId, card: ObjectId) {
    run_effect(
        t,
        p,
        None,
        Effect::PlayCard {
            who: PlayerRef::You,
            what: Sel::Target(0),
            free: true,
            optional: false,
        },
        &[Entity::Object(card)],
    );
}

#[test]
fn lands_played_during_resolution_count_toward_the_land_plays() {
    cr!("305.2a");
    let mut t = TestGame::new(2);
    // A land played as an effect resolves (outside the normal timing: here during
    // combat)...
    let a = t.hand(P0, "Forest");
    let b = t.hand(P0, "Forest");
    t.set_step(P0, Step::BeginningOfCombat);
    play_by_effect(&mut t, P0, a);
    assert_eq!(t.zone(a), Zone::Battlefield);
    assert_eq!(t.player(P0).lands_played_this_turn, 1);
    // ...was the land play for the turn: the number played isn't less than the number
    // allowed any more.
    t.set_step(P0, Step::PostcombatMain);
    assert!(!can_play_land(&mut t, P0, b));
    assert!(t.play_land(P0, b).is_err());
}

#[test]
fn an_instruction_to_play_a_land_without_a_land_play_left_is_ignored() {
    cr!("305.2b");
    let mut t = TestGame::new(2);
    let a = t.hand(P0, "Forest");
    let b = t.hand(P0, "Forest");
    t.play_land(P0, a).unwrap();
    play_by_effect(&mut t, P0, b);
    assert_eq!(t.zone(b), Zone::Hand(P0));
    assert_eq!(t.player(P0).lands_played_this_turn, 1);
}

#[test]
fn a_player_cant_play_a_land_during_another_players_turn() {
    cr!("305.3");
    ruling!(
        "Explore",
        "you'll draw a card when it resolves, but you won't be able to play a land that turn"
    );
    let mut t = TestGame::new(2);
    let forest = t.hand(P0, "Forest");
    // Explore cast during P1's turn (as though it had flash).
    t.battlefield(P0, "Leyline of Anticipation");
    t.lands(P0, "Forest", 2);
    let explore = t.hand(P0, "Explore");
    t.set_step(P1, Step::PrecombatMain);
    let hand = t.hand_size(P0);
    t.cast(P0, explore).go();
    t.resolve();
    assert_eq!(t.hand_size(P0), hand, "Explore left, a card was drawn");
    // Neither the land play nor an effect instructing P0 to play a land works.
    assert!(t.play_land(P0, forest).is_err());
    play_by_effect(&mut t, P0, forest);
    assert_eq!(t.zone(forest), Zone::Hand(P0));
    assert_eq!(t.player(P0).lands_played_this_turn, 0);
}

#[test]
fn putting_a_land_onto_the_battlefield_isnt_playing_a_land() {
    cr!("305.4");
    ruling!("Rampant Growth", "does not count toward your one per turn limit");
    let mut t = TestGame::new(2);
    let in_library = t.library_top(P0, "Forest");
    let in_hand = t.hand(P0, "Forest");
    t.lands(P0, "Forest", 2);
    let growth = t.hand(P0, "Rampant Growth");
    t.answer_choose(P0, &[Entity::Object(in_library)]);
    t.cast(P0, growth).go();
    t.resolve();
    assert_eq!(t.zone(in_library), Zone::Battlefield);
    assert_eq!(t.player(P0).lands_played_this_turn, 0);
    // The land play is still available.
    t.play_land(P0, in_hand).unwrap();
}

#[test]
fn land_subtypes_are_single_words_and_a_land_may_have_several() {
    cr!("305.5");
    ruling!("Underground Sea", "mana abilities associated with both of its basic land types");
    assert_eq!(subtypes_of("Mountain"), vec!["Mountain"]);
    assert_eq!(subtypes_of("Underground Sea"), vec!["Island", "Swamp"]);
    // "Urza's" is a land type of its own.
    assert_eq!(subtypes_of("Urza's Tower"), vec!["Urza's", "Tower"]);
    for s in ["Mountain", "Island", "Swamp", "Urza's", "Tower", "Gate", "Cave", "Desert"] {
        assert_eq!(subtype_kind(s), Some(SubtypeKind::Land), "{s}");
    }
    // Both of Underground Sea's types give it their mana abilities (CR 305.6).
    let mut t = TestGame::new(2);
    let sea = t.battlefield(P0, "Underground Sea");
    assert!(t.obj(sea).chars.has_subtype("Island") && t.obj(sea).chars.has_subtype("Swamp"));
}

#[test]
fn a_land_is_basic_only_if_it_has_the_basic_supertype() {
    cr!("305.8");
    ruling!("Underground Sea", "Things that affect basic land types do");
    // Blood Moon: "Nonbasic lands are Mountains." Underground Sea has basic land types
    // but not the basic supertype: it's nonbasic.
    let mut t = TestGame::new(2);
    let sea = t.battlefield(P0, "Underground Sea");
    let island = t.battlefield(P0, "Island");
    assert!(t.obj(island).chars.supertypes.contains(Supertype::Basic));
    assert!(!t.obj(sea).chars.supertypes.contains(Supertype::Basic));
    t.battlefield(P1, "Blood Moon");
    let s = t.obj(sea);
    assert!(s.chars.has_subtype("Mountain"));
    assert!(!s.chars.has_subtype("Island") && !s.chars.has_subtype("Swamp"));
    let i = t.obj(island);
    assert!(i.chars.has_subtype("Island") && !i.chars.has_subtype("Mountain"));
}
