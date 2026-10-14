//! Rulings batch P116 — "You may play an additional land" effects (CR 305.2, 305.2a:
//! cumulative extra land plays; CR 305.1, 505.6b: a land is played only in its
//! controller's main phase with an empty stack, never as a spell resolves) and playing
//! lands from the top of the library (CR 401.5, 305.2).

use crate::r_p116_common::*;
use mtg_engine::facedown::can_look_at;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn explorations_are_cumulative() {
    cr!("305.2", "305.2a");
    ruling!(
        "Exploration",
        "Exploration’s ability is cumulative with other effects that allow you to play additional lands, including ones created by other Explorations you control."
    );
    supported("Exploration");
    supported("Explore");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Exploration");
    t.battlefield(P0, "Exploration");
    assert_eq!(play_forests(&mut t, P0, 5), 3);
    // With Explore too: four.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Exploration");
    t.battlefield(P0, "Exploration");
    cast_new(&mut t, P0, "Explore", &[]);
    t.resolve_all();
    assert_eq!(play_forests(&mut t, P0, 6), 4);
}

/// P0 casts `name` (lands for its mana cost are put onto the battlefield untapped and
/// tapped when paying; they aren't land plays). While it's on the stack no land can be
/// played. Returns the spell.
fn cast_with_land_on_top(t: &mut TestGame, name: &str, targets: &[Entity]) -> ObjectId {
    t.library_top(P0, "Forest");
    let spell = cast_new(t, P0, name, targets);
    // Not as it resolves: the stack isn't empty now and the spell gives no chance to
    // play a land while resolving.
    let land = t.hand(P0, "Forest");
    assert!(!can_play_land(t, P0, land));
    t.g.move_object(
        land,
        Zone::Exile,
        mtg_engine::events::MoveCause::Effect,
        None,
    );
    spell
}

#[test]
fn explore_resolves_fully_then_lands_follow_normal_timing() {
    cr!("305.2", "305.1", "608.2");
    ruling!(
        "Explore",
        "Explore's effect allows you to play an additional land during your main phase. Doing so follows the normal timing rules for playing lands."
    );
    let mut t = TestGame::new(2);
    cast_with_land_on_top(&mut t, "Explore", &[]);
    let hand = t.hand_size(P0);
    t.resolve();
    // It drew the Forest; that Forest is one of the two lands P0 may play.
    assert_eq!(t.hand_size(P0), hand + 1);
    let drawn = t.g.find_in_zone(Zone::Hand(P0), "Forest");
    assert_eq!(drawn.len(), 1);
    // Not during combat.
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!can_play_land(&mut t, P0, drawn[0]));
    t.set_step(P0, Step::PostcombatMain);
    t.play_land(P0, drawn[0]).unwrap();
    assert_eq!(play_forests(&mut t, P0, 3), 1);
}

#[test]
fn urban_evolution_resolves_fully_then_lands_follow_normal_timing() {
    cr!("305.2", "305.1", "608.2");
    ruling!(
        "Urban Evolution",
        "Urban Evolution's effect allows you to play an additional land during your main phase. Doing so follows the normal timing rules for playing lands. In particular, you don't get to play a land as Urban Evolution resolves"
    );
    supported("Urban Evolution");
    let mut t = TestGame::new(2);
    cast_with_land_on_top(&mut t, "Urban Evolution", &[]);
    let hand = t.hand_size(P0);
    t.resolve();
    assert_eq!(t.hand_size(P0), hand + 3);
    let drawn = t.g.find_in_zone(Zone::Hand(P0), "Forest");
    t.play_land(P0, drawn[0]).unwrap();
    assert_eq!(play_forests(&mut t, P0, 3), 1);
}

#[test]
fn urban_evolutions_are_cumulative_with_each_other_and_explore() {
    cr!("305.2", "305.2a");
    ruling!(
        "Urban Evolution",
        "The effects of multiple Urban Evolutions in the same turn are cumulative. They're also cumulative with other effects that let you play additional lands, such as the one from Explore."
    );
    let mut t = TestGame::new(2);
    cast_new(&mut t, P0, "Urban Evolution", &[]);
    t.resolve_all();
    cast_new(&mut t, P0, "Urban Evolution", &[]);
    t.resolve_all();
    cast_new(&mut t, P0, "Explore", &[]);
    t.resolve_all();
    assert_eq!(play_forests(&mut t, P0, 6), 4);
}

/// P0 resolves `name` during P1's main phase (cast as though it had flash), then can't
/// play a land that turn; on P0's next turn only one land can be played.
fn on_opponents_turn(name: &str) {
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Vedalken Orrery");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P1, Step::PrecombatMain);
    let targets: Vec<Entity> = if name == "Scale the Heights" {
        vec![obj(bears)]
    } else {
        vec![]
    };
    let hand = t.hand_size(P0);
    let spell = cast_new(&mut t, P0, name, &targets);
    t.resolve_all();
    assert!(resolved(&t, spell));
    assert!(t.hand_size(P0) > hand);
    let land = t.hand(P0, "Forest");
    assert!(!can_play_land(&mut t, P0, land), "{name}");
    t.advance_to(P0, Step::PrecombatMain);
    assert_eq!(play_forests(&mut t, P0, 3), 1, "{name}");
}

#[test]
fn urban_evolution_on_another_players_turn_gives_no_land_play() {
    cr!("305.2", "305.1", "514.2");
    ruling!(
        "Urban Evolution",
        "If you somehow manage to cast Urban Evolution when it's not your turn, you'll draw three cards when it resolves, but you won't be able to play a land that turn."
    );
    supported("Vedalken Orrery");
    on_opponents_turn("Urban Evolution");
}

#[test]
fn scale_the_heights_resolves_fully_and_needs_your_turn() {
    cr!("305.2", "305.1", "608.2");
    ruling!(
        "Scale the Heights",
        "You don’t play a land as Scale the Heights resolves; Scale the Heights fully resolves first and you draw a card, perhaps including a land you’ll play later. If it’s not your turn, you won’t be able to play a land this turn at all."
    );
    supported("Scale the Heights");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_with_land_on_top(&mut t, "Scale the Heights", &[obj(bears)]);
    t.resolve();
    assert_eq!(t.life(P0), 22);
    assert_eq!(t.counters(bears, "+1/+1"), 1);
    let drawn = t.g.find_in_zone(Zone::Hand(P0), "Forest");
    t.play_land(P0, drawn[0]).unwrap();
    assert_eq!(play_forests(&mut t, P0, 3), 1);
    on_opponents_turn("Scale the Heights");
}

#[test]
fn scale_the_heights_with_an_illegal_target_does_nothing() {
    cr!("608.2b");
    ruling!(
        "Scale the Heights",
        "If you choose a target creature and it’s an illegal target by the time Scale the Heights tries to resolve, the spell doesn’t resolve. You don’t gain life, draw a card, or get to play an additional land."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_new(&mut t, P0, "Scale the Heights", &[obj(bears)]);
    destroy(&mut t, bears);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.hand_size(P0), hand);
    assert_eq!(play_forests(&mut t, P0, 3), 1);
}

#[test]
fn scale_the_heights_is_cumulative_with_other_extra_land_plays() {
    cr!("305.2", "305.2a");
    ruling!(
        "Scale the Heights",
        "The permission to play lands is cumulative with other effects that allow you to play additional lands"
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Exploration");
    cast_new(&mut t, P0, "Scale the Heights", &[]);
    t.resolve_all();
    cast_new(&mut t, P0, "Explore", &[]);
    t.resolve_all();
    assert_eq!(play_forests(&mut t, P0, 6), 4);
}

#[test]
fn oracles_of_mul_daya_are_cumulative() {
    cr!("305.2", "305.2a");
    ruling!(
        "Oracle of Mul Daya",
        "If you control more than one Oracle of Mul Daya, the effects of their first abilities are cumulative. If you control two, for example, you can play three lands on your turn."
    );
    supported("Oracle of Mul Daya");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Oracle of Mul Daya");
    t.battlefield(P0, "Oracle of Mul Daya");
    assert_eq!(play_forests(&mut t, P0, 5), 3);
}

#[test]
fn oracle_of_mul_daya_plays_the_next_land_from_the_top_too() {
    cr!("305.2", "401.5");
    ruling!(
        "Oracle of Mul Daya",
        "If you play your first land of the turn from the top of your library, and the new top card is another land card, you can play that one too."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Oracle of Mul Daya");
    let second = t.library_top(P0, "Forest");
    let first = t.library_top(P0, "Forest");
    t.play_land(P0, first).unwrap();
    assert_eq!(t.g.library_top(P0), Some(t.g.current(second)));
    t.play_land(P0, t.g.current(second)).unwrap();
    assert!(t.on_battlefield(first) && t.on_battlefield(second));
    // Both land plays are used.
    let third = t.library_top(P0, "Forest");
    assert!(!can_play_land(&mut t, P0, third));
}

/// With `name` on the battlefield, a land on top of P0's library can be played only in
/// P0's main phase with an empty stack, and it uses P0's land play.
fn top_land_timing(name: &str) {
    let mut t = TestGame::new(2);
    t.battlefield(P0, name);
    let top = t.library_top(P0, "Forest");
    // Not during combat, not on the opponent's turn, not with a spell on the stack.
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!can_play_land(&mut t, P0, top));
    t.set_step(P1, Step::PrecombatMain);
    assert!(!can_play_land(&mut t, P0, top));
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Mountain", 1);
    let shock = t.hand(P0, "Shock");
    t.cast(P0, shock).target(Entity::Player(P1)).go();
    assert!(!can_play_land(&mut t, P0, top));
    t.resolve_all();
    assert!(can_play_land(&mut t, P0, top));
    t.play_land(P0, top).unwrap();
    // That was a land play: Augur allows one, Oracle two in total.
    let extra = if name == "Oracle of Mul Daya" { 1 } else { 0 };
    assert_eq!(play_forests(&mut t, P0, 3), extra, "{name}");
}

#[test]
fn oracle_of_mul_daya_doesnt_change_when_lands_can_be_played() {
    cr!("305.1", "305.2", "505.6b");
    ruling!(
        "Oracle of Mul Daya",
        "Oracle of Mul Daya doesn't change the times when you can play a land card from the top of your library. You can play a land only during your main phase when you have priority and the stack is empty. Doing so counts as one of your land plays for the turn."
    );
    top_land_timing("Oracle of Mul Daya");
}

#[test]
fn augur_of_autumn_doesnt_change_when_lands_can_be_played() {
    cr!("305.1", "305.2", "505.6b");
    ruling!(
        "Augur of Autumn",
        "Augur of Autumn doesn't change the times when you can play a land card from the top of your library. You can play a land only during your main phase when you have priority and the stack is empty. Doing so counts as your land play for the turn."
    );
    supported("Augur of Autumn");
    top_land_timing("Augur of Autumn");
}

#[test]
fn augur_of_autumn_lets_you_look_at_the_top_card_any_time() {
    cr!("401.5");
    ruling!(
        "Augur of Autumn",
        "Essentially, Augur of Autumn lets you play with the top card of your library revealed only to you."
    );
    let mut t = TestGame::new(2);
    let top = t.library_top(P0, "Grizzly Bears");
    assert!(!can_look_at(&t.g, P0, top));
    t.battlefield(P0, "Augur of Autumn");
    t.g.recompute();
    // On the opponent's turn, while they have priority: P0 can look; P1 can't.
    t.set_step(P1, Step::DeclareAttackers);
    assert!(can_look_at(&t.g, P0, top));
    assert!(!can_look_at(&t.g, P1, top));
    assert_eq!(t.stack_len(), 0);
}

