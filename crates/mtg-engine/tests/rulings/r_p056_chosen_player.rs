//! Rulings batch P056 — permanents that choose a player as they enter (CR 614.12, 607.2d):
//! Stuffy Doll, Sower of Discord, Saskia the Unyielding, Nyxathid, Booby Trap. What's
//! chosen is linked to the object (a new object chooses again, CR 400.7; a copy has no
//! choice, CR 607.2d), simultaneous damage and losing the game (CR 704.3, 800.4a),
//! state-based actions waiting for a resolution to finish (CR 704.3), and triggers that
//! wait for the drawing to finish (CR 603.3).

use crate::r_p056_common::*;
use crate::r_s01_common::{supported, triggers_on_stack, watch};
use crate::r_s06_common::give_control;
use crate::r_s29_common::cast_and_resolve;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::events::Event;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// `p`'s permanent `name` enters choosing `who` (one or two players).
fn enter_choosing(t: &mut TestGame, p: PlayerId, name: &str, who: &[PlayerId]) -> ObjectId {
    let players: Vec<Entity> = who.iter().map(|q| Entity::Player(*q)).collect();
    t.answer_choose(p, &players);
    let id = t.enter(p, name);
    t.resolve_all();
    id
}

/// `caster` casts a "deals X damage to each creature [with flying] and each player" spell
/// with lands for it.
fn cast_x(t: &mut TestGame, caster: PlayerId, name: &str, x: i64) {
    t.lands(caster, "Forest", 1 + x as usize);
    t.lands(caster, "Mountain", 1);
    let c = t.hand(caster, name);
    t.cast(caster, c).x(x).go();
    t.resolve_all();
}

#[test]
fn stuffy_doll_deals_all_the_damage_it_was_dealt() {
    cr!("120.3e", "510.1");
    ruling!(
        "Stuffy Doll",
        "A creature can be dealt an amount of damage greater than its toughness. For example, if Stuffy Doll is dealt 3 damage, its triggered ability deals 3 damage"
    );
    supported("Stuffy Doll");
    let mut t = TestGame::new(2);
    let doll = enter_choosing(&mut t, P0, "Stuffy Doll", &[P1]);
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(doll).go();
    t.resolve_all();
    assert!(t.on_battlefield(doll));
    assert_eq!(t.life(P1), 17);
}

#[test]
fn stuffy_doll_controller_loses_before_the_trigger_goes_on_the_stack() {
    cr!("704.3", "704.5a", "800.4a");
    ruling!(
        "Stuffy Doll",
        "If your life total is brought to 0 or less at the same time that Stuffy Doll is dealt damage, you lose the game before its triggered ability goes on the stack."
    );
    supported("Earthquake");
    let mut t = TestGame::new(3);
    enter_choosing(&mut t, P0, "Stuffy Doll", &[P1]);
    t.g.players[0].life = 3;
    t.set_step(P2, Step::PrecombatMain);
    cast_x(&mut t, P2, "Earthquake", 3);
    assert!(t.has_lost(P0));
    assert_eq!(t.life(P1), 17, "only Earthquake's damage");
}

#[test]
fn sower_dealt_lethal_damage_at_the_same_time_still_triggers() {
    cr!("603.10a", "603.2");
    ruling!(
        "Sower of Discord",
        "If Sower of Discord is dealt lethal damage at the same time that one of the chosen players is dealt damage, its ability will cause the second player to lose life."
    );
    supported("Sower of Discord");
    supported("Hurricane");
    let mut t = TestGame::new(3);
    let sower = enter_choosing(&mut t, P0, "Sower of Discord", &[P1, P2]);
    cast_x(&mut t, P0, "Hurricane", 6);
    assert!(!t.on_battlefield(sower));
    // Each chosen player was dealt 6 and lost 6 for the other.
    assert_eq!((t.life(P0), t.life(P1), t.life(P2)), (14, 8, 8));
}

#[test]
fn sower_controller_losing_at_the_same_time_stops_the_trigger() {
    cr!("704.3", "800.4a");
    ruling!(
        "Sower of Discord",
        "If Sower of Discord’s controller is dealt damage that causes them to lose the game at the same time that either of the chosen players is dealt damage, Sower of Discord’s triggered ability won’t resolve"
    );
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Island");
    enter_choosing(&mut t, P0, "Sower of Discord", &[P1, P2]);
    t.g.players[0].life = 6;
    t.set_step(P1, Step::PrecombatMain);
    cast_x(&mut t, P1, "Hurricane", 6);
    assert!(t.has_lost(P0));
    assert_eq!((t.life(P1), t.life(P2)), (14, 14));
}

#[test]
fn saskia_deals_that_much_damage_again_and_it_isnt_combat_damage() {
    cr!("510.2", "120.2b");
    ruling!(
        "Saskia the Unyielding",
        "If a creature you control deals combat damage to the chosen player, it deals that much damage to that player again."
    );
    ruling!(
        "Saskia the Unyielding",
        "The damage that Saskia's ability causes a creature to deal isn't combat damage."
    );
    supported("Saskia the Unyielding");
    let mut t = TestGame::new(3);
    enter_choosing(&mut t, P0, "Saskia the Unyielding", &[P1]);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.attack(&[(bears, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    let to_p1: Vec<(ObjectId, bool)> =
        t.g.turn_events
            .iter()
            .filter_map(|e| match e {
                Event::Damage {
                    source,
                    target: Entity::Player(P1),
                    combat,
                    ..
                } => Some((*source, *combat)),
                _ => None,
            })
            .collect();
    assert_eq!(
        to_p1,
        vec![(bears, true), (bears, false)],
        "{}",
        t.dump_log()
    );
}

#[test]
fn saskia_returning_chooses_a_player_again() {
    cr!("400.7", "614.12");
    ruling!(
        "Saskia the Unyielding",
        "If Saskia leaves the battlefield and returns, you choose a player again. It doesn't remember the previously chosen player."
    );
    supported("Flicker of Fate");
    let mut t = TestGame::new(3);
    let saskia = enter_choosing(&mut t, P0, "Saskia the Unyielding", &[P1]);
    assert_eq!(t.obj_now(saskia).choices.player, Some(P1));
    t.answer_choose(P0, &[Entity::Player(P2)]);
    cast_and_resolve(&mut t, P0, "Flicker of Fate", &[obj(saskia)]);
    let now = t.g.current(saskia);
    assert_ne!(now, saskia);
    assert_eq!(t.obj_now(saskia).choices.player, Some(P2));
}

#[test]
fn saskia_does_nothing_once_the_chosen_player_left_the_game() {
    cr!("800.4a", "800.4h");
    ruling!(
        "Saskia the Unyielding",
        "Once the chosen player has left the game, Saskia's triggered ability will have no effect."
    );
    let mut t = TestGame::new(3);
    enter_choosing(&mut t, P0, "Saskia the Unyielding", &[P2]);
    t.g.lose_game(P2);
    t.settle();
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.attack(&[(bears, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

#[test]
fn nyxathid_changes_as_cards_are_drawn_and_discarded_without_dying() {
    cr!("704.3", "121.2");
    ruling!(
        "Nyxathid",
        "Although Nyxathid momentarily had 0 toughness while that spell was resolving, it isn't put into its owner's graveyard"
    );
    supported("Nyxathid");
    supported("Frantic Search");
    let mut t = TestGame::new(2);
    let nyx = enter_choosing(&mut t, P0, "Nyxathid", &[P1]);
    for _ in 0..5 {
        t.hand(P1, "Grizzly Bears");
    }
    let search = t.hand(P1, "Frantic Search");
    assert_eq!(t.pt(nyx), (1, 1));
    t.lands(P1, "Island", 3);
    t.set_step(P1, Step::PrecombatMain);
    let seen = watch(
        &mut t,
        P1,
        |d| matches!(d, Decision::ChooseEntities { .. }),
        |g| {
            let n = g
                .permanents()
                .find(|o| o.chars.name == "Nyxathid")
                .expect("Nyxathid");
            n.toughness()
        },
    );
    t.cast(P1, search).go();
    t.resolve_all();
    assert_eq!(seen.lock().unwrap()[0], 0, "0/0 while discarding");
    assert!(t.on_battlefield(nyx));
    assert_eq!(t.pt(nyx), (2, 2));
}

#[test]
fn nyxathid_does_nothing_once_the_chosen_player_left_the_game() {
    cr!("800.4a");
    ruling!(
        "Nyxathid",
        "In a multiplayer game, if the chosen player leaves the game, Nyxathid's second ability simply won't do anything anymore."
    );
    let mut t = TestGame::new(3);
    let nyx = enter_choosing(&mut t, P0, "Nyxathid", &[P1]);
    t.hand(P1, "Grizzly Bears");
    assert_eq!(t.pt(nyx), (6, 6));
    t.g.lose_game(P1);
    t.settle();
    t.g.recompute();
    assert_eq!(t.pt(nyx), (7, 7));
}

#[test]
fn nyxathid_tracks_the_chosen_player_after_a_control_change() {
    cr!("607.2d", "614.12");
    ruling!(
        "Nyxathid",
        "Nyxathid tracks the hand size of the chosen player even if that player ceases to be an opponent of its controller."
    );
    let mut t = TestGame::new(2);
    let nyx = enter_choosing(&mut t, P0, "Nyxathid", &[P1]);
    for _ in 0..2 {
        t.hand(P1, "Grizzly Bears");
    }
    t.hand(P0, "Grizzly Bears");
    give_control(&mut t, nyx, P1);
    assert_eq!(t.obj_now(nyx).controller, P1);
    assert_eq!(t.pt(nyx), (5, 5));
}

#[test]
fn a_copy_of_nyxathid_has_no_chosen_player() {
    cr!("607.2d", "707.2");
    ruling!(
        "Nyxathid",
        "If another creature becomes a copy of Nyxathid (due to Mirrorweave, for example), the second ability won't do anything"
    );
    supported("Mirrorweave");
    let mut t = TestGame::new(2);
    let nyx = enter_choosing(&mut t, P0, "Nyxathid", &[P1]);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.hand(P1, "Grizzly Bears");
    t.lands(P0, "Plains", 6);
    let weave = t.hand(P0, "Mirrorweave");
    t.cast(P0, weave).target(nyx).go();
    t.resolve_all();
    assert_eq!(t.obj_now(bears).chars.name, "Nyxathid");
    assert_eq!(t.pt(bears), (7, 7));
    assert_eq!(t.pt(nyx), (6, 6));
}

// ---------------------------------------------------------------------------
// Booby Trap
// ---------------------------------------------------------------------------

/// P0's Booby Trap is cast and resolves, choosing P1 and Lightning Bolt.
fn trap(t: &mut TestGame) -> ObjectId {
    t.answer_choose(P0, &[Entity::Player(P1)]);
    t.answer(
        P0,
        DecisionKind::Name,
        Answer::Text("Lightning Bolt".into()),
    );
    t.lands(P0, "Wastes", 6);
    let trap = t.hand(P0, "Booby Trap");
    t.cast(P0, trap).go();
    t.settle();
    t.resolve();
    assert!(t.on_battlefield(trap));
    assert_eq!(
        t.obj_now(trap).choices.card_name.as_deref(),
        Some("Lightning Bolt")
    );
    trap
}

#[test]
fn booby_trap_naming_is_part_of_its_resolution() {
    cr!("614.12", "608.3");
    ruling!(
        "Booby Trap",
        "The choice of which card to name is part of the resolution of the spell or ability which puts Booby Trap onto the battlefield. It can’t be countered."
    );
    supported("Booby Trap");
    let mut t = TestGame::new(2);
    trap(&mut t);
    // Nothing used the stack for the choice.
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn booby_trap_stays_if_its_ability_is_countered() {
    cr!("701.6b");
    ruling!(
        "Booby Trap",
        "If Booby Trap’s last ability is countered, Booby Trap remains on the battlefield"
    );
    supported("Stifle");
    let mut t = TestGame::new(2);
    let trap = trap(&mut t);
    t.library_top(P1, "Lightning Bolt");
    t.g.draw_cards(P1, 1);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    let ability = *t.g.stack.last().unwrap();
    t.lands(P1, "Island", 1);
    let stifle = t.hand(P1, "Stifle");
    t.cast(P1, stifle).target(ability).go();
    t.resolve_all();
    assert!(t.on_battlefield(trap));
    assert_eq!(t.life(P1), 20);
}

#[test]
fn booby_trap_only_the_first_trigger_deals_damage() {
    cr!("603.2", "701.21a");
    ruling!(
        "Booby Trap",
        "If a single Booby Trap’s ability triggers multiple times, then only the first ability to resolve causes Booby Trap to be sacrificed and damage to be dealt."
    );
    ruling!(
        "Booby Trap",
        "If an effect causes the player to draw multiple cards, each card is revealed as it’s drawn. If Booby Trap’s ability triggers, it will go on the stack once the card-drawing effect has concluded."
    );
    let mut t = TestGame::new(2);
    let trap = trap(&mut t);
    t.library_top(P1, "Lightning Bolt");
    t.library_top(P1, "Lightning Bolt");
    t.lands(P1, "Island", 3);
    t.set_step(P1, Step::PrecombatMain);
    let div = t.hand(P1, "Divination");
    t.cast(P1, div).go();
    t.g.resolve_top();
    // Both cards were revealed as they were drawn; the two triggers wait for the
    // drawing to finish.
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.dump_log().matches("reveals Lightning Bolt").count(), 2);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "chosen name"), 2);
    t.resolve_all();
    assert!(!t.on_battlefield(trap));
    assert_eq!(t.life(P1), 10);
}
