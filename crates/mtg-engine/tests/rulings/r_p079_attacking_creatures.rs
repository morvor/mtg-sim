//! Rulings batch P079 — what an "attacking creature" or "blocking creature" is: declared
//! or put onto the battlefield attacking/blocking this combat, staying so through the end
//! of combat step (CR 508.1k, 508.4, 509.1g, 511.3), even after the attacked player
//! leaves the game (CR 506.4, 800.4a), and none outside combat.

use crate::r_p079_common::*;
use crate::r_s01_common::supported;
use crate::r_s03_common::to_blockers;
use crate::r_s10_common::{attacking, blocking};
use crate::r_s21_common::go_to;
use crate::r_s25_common::creature_tokens;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// P0 attacks P1 with a Grizzly Bears (unblocked) and the game moves on to `step` of the
/// combat (damage dealt if past it). Returns the attacker.
fn bears_attack_until(t: &mut TestGame, step: Step) -> ObjectId {
    let bears = t.battlefield(P0, "Grizzly Bears");
    to_blockers(t, &[(bears, Entity::Player(P1))], &[]);
    go_to(t, step);
    bears
}

/// One case: a spell P1 casts at an attacking creature in the end of combat step, after
/// combat damage, and whether its targets are the creature or its controller.
struct Case {
    card: &'static str,
    targets_player: bool,
    /// Where the attacker ends up.
    gone: fn(&TestGame, ObjectId) -> bool,
}

fn left_battlefield(t: &TestGame, id: ObjectId) -> bool {
    !t.on_battlefield(id)
}

#[test]
fn spells_for_attacking_creatures_work_in_the_end_of_combat_step() {
    cr!("508.1k", "511.3", "511.1");
    ruling!(
        "Entrapment Maneuver",
        "An \"attacking creature\" is one that has been declared as an attacker or put onto the battlefield attacking during this combat."
    );
    ruling!(
        "Aetherize",
        "Unless that creature leaves combat, it continues to be an attacking creature through the end of combat step, even if the player it was attacking has left the game, or the planeswalker it was attacking has left combat. There's no such thing"
    );
    ruling!(
        "Puncturing Light",
        "An “attacking creature” is one that has been declared as an attacker this combat, or one that was put onto the battlefield attacking this combat. Unless that creature leaves combat, it continues to be an attacking creature through the end of combat step."
    );
    ruling!(
        "Angel of the Dire Hour",
        "It’s possible to exile all attacking creatures after they’ve dealt combat damage if you cast Angel of the Dire Hour during the combat damage step or end of combat step."
    );
    let cases = [
        Case {
            card: "Entrapment Maneuver",
            targets_player: true,
            gone: left_battlefield,
        },
        Case {
            card: "Aetherize",
            targets_player: false,
            gone: |t, id| t.in_hand(P0, "Grizzly Bears") && !t.on_battlefield(id),
        },
        Case {
            card: "Puncturing Light",
            targets_player: false,
            gone: |t, id| t.in_graveyard(P0, "Grizzly Bears") && !t.on_battlefield(id),
        },
        Case {
            card: "Angel of the Dire Hour",
            targets_player: false,
            gone: |t, id| t.in_exile("Grizzly Bears") && !t.on_battlefield(id),
        },
        Case {
            card: "Wing Shards",
            targets_player: true,
            gone: left_battlefield,
        },
        Case {
            card: "Celestial Flare",
            targets_player: true,
            gone: left_battlefield,
        },
        Case {
            card: "Arrows of Justice",
            targets_player: false,
            gone: left_battlefield,
        },
        Case {
            card: "Warrant // Warden",
            targets_player: false,
            gone: |t, id| t.library_size(P0) == 31 && !t.on_battlefield(id),
        },
    ];
    for c in cases {
        supported(c.card);
        for step in [Step::CombatDamage, Step::EndOfCombat] {
            let mut t = TestGame::new(2);
            let bears = bears_attack_until(&mut t, step);
            assert_eq!(t.life(P1), 18, "{}: combat damage was dealt", c.card);
            assert!(attacking(&t, bears));
            let target = if c.targets_player {
                vec![Entity::Player(P0)]
            } else {
                vec![obj(bears)]
            };
            let targets: &[Entity] = if c.card == "Aetherize" || c.card == "Angel of the Dire Hour"
            {
                &[]
            } else {
                &target
            };
            cast_card(&mut t, P1, c.card, targets).unwrap_or_else(|e| panic!("{}: {e:?}", c.card));
            t.resolve_all();
            assert!((c.gone)(&t, bears), "{} in {step:?}", c.card);
        }
    }
}

#[test]
fn creatures_put_onto_the_battlefield_attacking_are_attacking_creatures() {
    cr!("508.4", "508.1k");
    ruling!(
        "Wing Shards",
        "An “attacking creature” is one that has been declared as an attacker or put onto the battlefield attacking during a turn’s combat."
    );
    // The only attacking creature P0 controls entered attacking: Wing Shards makes P0
    // sacrifice it (and not the non-attacking Hill Giant).
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    to_blockers(&mut t, &[(bears, Entity::Player(P1))], &[]);
    let soldier = enter_attacking(&mut t, P0, "Savannah Lions", Entity::Player(P1));
    assert!(attacking(&t, soldier));
    crate::r_s02_common::destroy(&mut t, bears);
    cast_card(&mut t, P1, "Wing Shards", &[Entity::Player(P0)]).unwrap();
    t.resolve_all();
    assert!(
        !t.on_battlefield(soldier),
        "the creature that entered attacking"
    );
    assert!(t.on_battlefield(giant));
}

#[test]
fn creature_stays_attacking_after_the_attacked_player_leaves_the_game() {
    cr!("506.4", "800.4a", "508.1k");
    ruling!(
        "Wing Shards",
        "Unless that creature leaves combat, it continues to be an attacking creature through the end of combat step, even if the player it was attacking has left the game or the planeswalker it was attacking has left combat."
    );
    ruling!(
        "Warrant // Warden",
        "Unless that creature leaves combat, it continues to be an attacking creature through the end of combat step, even if the player it was attacking has left the game or the planeswalker it was attacking has left combat. Similarly, a \"blocking creature\""
    );
    // Three players: P0 attacks P2 with Bears and P1's planeswalker with a Giant; P2
    // leaves the game and the planeswalker leaves combat (dies).
    let mut t = TestGame::new(3);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let pw = t.battlefield(P1, "Jace Beleren");
    to_blockers(
        &mut t,
        &[(bears, Entity::Player(P2)), (giant, obj(pw))],
        &[],
    );
    concede(&mut t, P2);
    crate::r_s02_common::destroy(&mut t, pw);
    assert!(attacking(&t, bears) && attacking(&t, giant));
    // Wing Shards: P0 sacrifices one of them (the Bears).
    t.answer_choose(P0, &[obj(bears)]);
    cast_card(&mut t, P1, "Wing Shards", &[Entity::Player(P0)]).unwrap();
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    // Warrant puts the Giant (attacking a planeswalker that left) on top of its library.
    cast_card(&mut t, P1, "Warrant // Warden", &[obj(giant)]).unwrap();
    t.resolve_all();
    assert!(!t.on_battlefield(giant));
    assert_eq!(
        t.g.player(P0)
            .library
            .last()
            .map(|c| t.g.obj(*c).chars.name.to_string()),
        Some("Hill Giant".into())
    );
}

#[test]
fn blocking_creature_stays_blocking_after_its_attacker_leaves() {
    cr!("509.1g", "506.4", "511.3");
    ruling!(
        "Arrows of Justice",
        "even if the player it was attacking has left the game, the planeswalker it was attacking has left combat, or the creature or creatures it was blocking have left combat, as appropriate."
    );
    ruling!(
        "Warrant // Warden",
        "Similarly, a \"blocking creature\" is one that has been declared as a blocker or put onto the battlefield blocking during this combat."
    );
    ruling!(
        "Arrows of Justice",
        "Arrows of Justice may be cast during the end of combat step, after combat damage has been dealt."
    );
    // P1's Hill Giant blocks and kills P0's Bears; in the end of combat step it's still a
    // blocking creature: Arrows of Justice (cast by P0) can target it.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    to_blockers(&mut t, &[(bears, Entity::Player(P1))], &[(giant, bears)]);
    go_to(&mut t, Step::EndOfCombat);
    assert!(!t.on_battlefield(bears));
    assert!(blocking(&t, giant));
    cast_card(&mut t, P0, "Arrows of Justice", &[obj(giant)]).unwrap();
    t.resolve_all();
    assert!(!t.on_battlefield(giant), "4 damage to a 3/3 with 2 damage");
    // A creature that entered blocking is a blocking creature: Warrant.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    to_blockers(&mut t, &[(bears, Entity::Player(P1))], &[]);
    let wall = crate::r_s21_common::enter_blocking(&mut t, P1, "Wall of Stone", bears);
    assert!(blocking(&t, wall));
    cast_card(&mut t, P0, "Warrant // Warden", &[obj(wall)]).unwrap();
    t.resolve_all();
    assert!(!t.on_battlefield(wall));
    assert_eq!(t.library_size(P1), 31);
}

#[test]
fn no_attacking_creatures_outside_combat() {
    cr!("511.3", "506.4");
    ruling!(
        "Aetherize",
        "There's no such thing as an attacking creature outside of the combat phase."
    );
    let mut t = TestGame::new(2);
    let bears = bears_attack_until(&mut t, Step::PostcombatMain);
    assert!(!attacking(&t, bears));
    cast_card(&mut t, P1, "Aetherize", &[]).unwrap();
    t.resolve_all();
    assert!(
        t.on_battlefield(bears),
        "not returned: not attacking after combat"
    );
    assert_eq!(creature_tokens(&t, P1), 0);
}
