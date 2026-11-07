//! Rulings batch S04 — detain (CR 701.35): "Until your next turn, that permanent can't
//! attack or block and its activated abilities can't be activated."
//!
//! Several of these rulings are printed twice, with curly and with straight apostrophes;
//! the curly ones are cited with New Prahv Guildmage and the creatures and spells of
//! Return to Ravnica, the straight ones with Lyev Skyknight (made instant-speed with
//! Leyline of Anticipation) and Lavinia of the Tenth.

use crate::r_s01_common::*;
use crate::r_s04_common::*;
use mtg_engine::decision::{Action, Answer};
use mtg_engine::game::GameConfig;
use mtg_engine::testing::*;
use mtg_engine::turn::{Stage, Step};
use mtg_engine::*;

/// Lands for New Prahv Guildmage's "{3}{W}{U}: Detain target nonland permanent an
/// opponent controls."
fn guildmage(t: &mut TestGame, p: PlayerId) -> ObjectId {
    supported("New Prahv Guildmage");
    let g = t.battlefield(p, "New Prahv Guildmage");
    t.lands(p, "Plains", 2);
    t.lands(p, "Island", 1);
    t.lands(p, "Wastes", 2);
    g
}

/// P0 detains `target` with New Prahv Guildmage's second ability, which resolves.
fn detain_with_guildmage(t: &mut TestGame, g: ObjectId, target: ObjectId) {
    t.activate(P0, g, 1, &[Entity::Object(target)]).unwrap();
    t.resolve();
}

/// P0 casts Lyev Skyknight at instant speed (Leyline of Anticipation) and its enters
/// trigger detains `target`.
fn flash_skyknight(t: &mut TestGame, target: ObjectId) {
    supported("Lyev Skyknight");
    t.battlefield(P0, "Leyline of Anticipation");
    give_mana_for(t, P0, "Lyev Skyknight");
    let knight = t.hand(P0, "Lyev Skyknight");
    t.cast(P0, knight).go();
    t.answer_targets(P0, &[Entity::Object(target)]);
    t.resolve();
    t.resolve();
    assert!(t.on_battlefield(knight));
}

#[test]
fn a_detained_permanents_static_and_triggered_abilities_still_work() {
    cr!("701.35a");
    ruling!(
        "Lavinia of the Tenth",
        "The static abilities of a detained permanent still apply. The triggered abilities of a detained permanent can still trigger."
    );
    supported("Lavinia of the Tenth");
    // Lavinia of the Tenth: "When Lavinia enters, detain each nonland permanent your
    // opponents control with mana value 4 or less."
    let mut t = TestGame::new(2);
    let marshal = t.battlefield(P1, "Benalish Marshal");
    let warden = t.battlefield(P1, "Soul Warden");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.enter(P0, "Lavinia of the Tenth");
    t.resolve_all();
    t.advance_to(P1, Step::PrecombatMain);
    for c in [marshal, warden, bears] {
        assert!(!t.g.can_attack(c));
    }
    // Benalish Marshal's static ability ("Other creatures you control get +1/+1").
    assert_eq!(t.pt(bears), (3, 3));
    assert_eq!(t.pt(warden), (2, 2));
    // Soul Warden's triggered ability ("Whenever another creature enters, you gain 1
    // life").
    let life = t.life(P1);
    t.enter(P1, "Hill Giant");
    t.resolve_all();
    assert_eq!(t.life(P1), life + 1);
}

#[test]
fn no_one_can_activate_a_detained_permanents_abilities() {
    cr!("701.35a", "602.5", "605.3a");
    ruling!(
        "Azorius Justiciar",
        "Activated abilities include a colon and are written in the form “[cost]: [effect].” No one can activate any activated abilities, including mana abilities, of a detained permanent."
    );
    supported("Azorius Justiciar");
    // Azorius Justiciar: "When this creature enters, detain up to two target creatures
    // your opponents control."
    let mut t = TestGame::new(2);
    let elves = t.battlefield(P1, "Llanowar Elves");
    let pyro = t.battlefield(P1, "Prodigal Pyromancer");
    t.answer_targets(P0, &[Entity::Object(elves), Entity::Object(pyro)]);
    t.enter(P0, "Azorius Justiciar");
    t.resolve_all();
    t.advance_to(P1, Step::PrecombatMain);
    // Neither the mana ability nor the other ability can be activated.
    assert!(t.activate(P1, elves, 0, &[]).is_err());
    assert_eq!(t.g.player(P1).mana_pool.total(), 0);
    assert!(t.activate(P1, pyro, 0, &[Entity::Player(P0)]).is_err());
    assert_eq!(t.life(P0), 20);
    assert!(!t.obj(elves).tapped && !t.obj(pyro).tapped);
    // They can after P0's next turn has begun.
    t.advance_to(P0, Step::Upkeep);
    t.advance_to(P1, Step::PrecombatMain);
    t.activate(P1, pyro, 0, &[Entity::Player(P0)]).unwrap();
    t.resolve();
    assert_eq!(t.life(P0), 19);
}

#[test]
fn detaining_an_attacking_or_blocking_creature_doesnt_remove_it_from_combat() {
    cr!("701.35a", "506.4", "508.1", "509.1");
    ruling!(
        "New Prahv Guildmage",
        "If a creature is already attacking or blocking when it’s detained, it won’t be removed from combat. It will continue to attack or block."
    );
    // Attacking: P1's Hill Giant is detained during the declare attackers step.
    let mut t = TestGame::new(2);
    let g = guildmage(&mut t, P0);
    let giant = t.battlefield(P1, "Hill Giant");
    t.set_step(P1, Step::BeginningOfCombat);
    attack_with(&mut t, &[(giant, Entity::Player(P0))]);
    detain_with_guildmage(&mut t, g, giant);
    assert!(t.g.is_attacking(giant));
    block_and_finish(&mut t, P0, &[]);
    assert_eq!(t.life(P0), 17);

    // Blocking: P1's Hill Giant blocks and is detained during the declare blockers step.
    let mut t = TestGame::new(2);
    let g = guildmage(&mut t, P0);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![(giant, bears)]));
    t.advance_to(P0, Step::DeclareBlockers);
    assert!(t.g.is_blocking(giant));
    detain_with_guildmage(&mut t, g, giant);
    assert!(t.g.is_blocking(giant));
    t.advance_to(P0, Step::EndOfCombat);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(t.life(P1), 20);
}

#[test]
fn a_creature_detained_by_lyev_skyknight_while_attacking_keeps_attacking() {
    cr!("701.35a", "506.4");
    ruling!(
        "Lyev Skyknight",
        "If a creature is already attacking or blocking when it's detained, it won't be removed from combat. It will continue to attack or block."
    );
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.set_step(P1, Step::BeginningOfCombat);
    attack_with(&mut t, &[(giant, Entity::Player(P0))]);
    flash_skyknight(&mut t, giant);
    assert!(t.g.is_attacking(giant));
    block_and_finish(&mut t, P0, &[]);
    assert_eq!(t.life(P0), 17);
}

#[test]
fn an_activated_ability_already_on_the_stack_is_unaffected_by_detain() {
    cr!("701.35a", "113.7a");
    ruling!(
        "New Prahv Guildmage",
        "If a permanent’s activated ability is on the stack when that permanent is detained, the ability will be unaffected."
    );
    ruling!(
        "Lyev Skyknight",
        "If a permanent's activated ability is on the stack when that permanent is detained, the ability will be unaffected."
    );
    for skyknight in [false, true] {
        let mut t = TestGame::new(2);
        let g = if skyknight {
            None
        } else {
            Some(guildmage(&mut t, P0))
        };
        let pyro = t.battlefield(P1, "Prodigal Pyromancer");
        t.set_step(P1, Step::PrecombatMain);
        t.activate(P1, pyro, 0, &[Entity::Player(P0)]).unwrap();
        // In response, P0 detains the Pyromancer.
        match g {
            Some(g) => detain_with_guildmage(&mut t, g, pyro),
            None => flash_skyknight(&mut t, pyro),
        }
        assert_eq!(on_stack(&t, "damage"), 1, "{:?}", stack_items(&t));
        t.resolve_all();
        assert_eq!(t.life(P0), 19);
        // It's detained: it can't be activated again this turn (it's untapped by then in
        // this test only to show the restriction).
        t.g.untap(pyro);
        assert!(t.activate(P1, pyro, 0, &[Entity::Player(P0)]).is_err());
    }
}

#[test]
fn a_detained_noncreature_permanent_that_becomes_a_creature_cant_attack_or_block() {
    cr!("701.35a", "508.1a", "509.1a");
    ruling!(
        "New Prahv Guildmage",
        "If a noncreature permanent is detained and later turns into a creature, it won’t be able to attack or block."
    );
    let mut t = TestGame::new(2);
    let g = guildmage(&mut t, P0);
    let stone = t.battlefield(P1, "Mind Stone");
    detain_with_guildmage(&mut t, g, stone);
    // On P1's turn, Ensoul Artifact makes it a 5/5 artifact creature.
    t.advance_to(P1, Step::PrecombatMain);
    t.lands(P1, "Island", 2);
    let ensoul = t.hand(P1, "Ensoul Artifact");
    t.cast(P1, ensoul).target(stone).go();
    t.resolve_all();
    assert!(t.obj_now(stone).is_creature());
    assert_eq!(t.pt(stone), (5, 5));
    assert!(!t.g.can_attack(stone));
    assert!(!t.g.can_block_at_all(stone));
    // Once P0's next turn begins, it can block.
    t.advance_to(P0, Step::Upkeep);
    assert!(t.g.can_block_at_all(stone));
}

/// A four-player free-for-all game.
fn ffa4() -> TestGame {
    TestGame::with_config(4, GameConfig::free_for_all())
}

/// Advances until `active`'s next turn has begun, with priority.
fn to_turn_of(t: &mut TestGame, active: PlayerId) {
    let turn = t.g.turn.number;
    let ok = t.g.run_until(20_000, |g| {
        g.turn.number != turn && g.turn.active == active && g.turn.stage == Stage::Priority
    });
    assert!(ok, "{active}'s turn didn't begin");
}

#[test]
fn detain_by_a_player_who_left_lasts_until_their_turn_would_have_begun() {
    cr!("701.35a", "800.4m", "800.4k");
    ruling!(
        "Azorius Arrester",
        "When a player leaves a multiplayer game, any continuous effects with durations that last until that player’s next turn or until a specific point in that turn will last until that turn would have begun. They neither expire immediately nor last indefinitely."
    );
    supported("Azorius Arrester");
    // Azorius Arrester: "When this creature enters, detain target creature an opponent
    // controls." P0 detains P2's Hill Giant, then leaves the game during P1's turn.
    let mut t = ffa4();
    let giant = t.battlefield(P2, "Hill Giant");
    t.answer_targets(P0, &[Entity::Object(giant)]);
    t.enter(P0, "Azorius Arrester");
    t.resolve_all();
    to_turn_of(&mut t, P1);
    t.g.take_action(P0, Action::Concede);
    t.g.flush_events();
    assert!(t.has_lost(P0));
    // It doesn't expire immediately.
    t.g.recompute();
    assert!(!t.g.can_block_at_all(giant));
    to_turn_of(&mut t, P2);
    assert!(!t.g.can_attack(giant));
    to_turn_of(&mut t, P3);
    assert!(!t.g.can_block_at_all(giant));
    // It ends when P0's turn would have begun: in P1's next turn, it can block again.
    to_turn_of(&mut t, P1);
    assert!(t.g.can_block_at_all(giant));
    to_turn_of(&mut t, P2);
    assert!(t.g.can_attack(giant));
}

#[test]
fn lyev_skyknights_detain_lasts_until_the_leaving_players_turn_would_have_begun() {
    cr!("701.35a", "800.4m");
    ruling!(
        "Lyev Skyknight",
        "When a player leaves a multiplayer game, any continuous effects with durations that last until that player's next turn or until a specific point in that turn will last until that turn would have begun. They neither expire immediately nor last indefinitely."
    );
    // P0's Lyev Skyknight detains P2's Llanowar Elves; P0 leaves during P1's turn.
    let mut t = ffa4();
    let elves = t.battlefield(P2, "Llanowar Elves");
    t.answer_targets(P0, &[Entity::Object(elves)]);
    t.enter(P0, "Lyev Skyknight");
    t.resolve_all();
    to_turn_of(&mut t, P1);
    t.g.take_action(P0, Action::Concede);
    t.g.flush_events();
    // In P2's turn it still can't be tapped for mana.
    to_turn_of(&mut t, P2);
    t.g.recompute();
    assert!(t.activate(P2, elves, 0, &[]).is_err());
    // After the turn P0 would have had, it can.
    to_turn_of(&mut t, P2);
    assert!(t.activate(P2, elves, 0, &[]).is_ok());
}
