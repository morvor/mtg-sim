//! Rulings batch S30 — combat damage (CR 510): assigning it as though a creature weren't
//! blocked, blocked creatures whose blockers are gone, creatures with no power, creatures
//! that stay in combat with their damage prevented, damage dealt during the declare
//! blockers step, sacrificing combatants, and "destroy at end of combat".

use crate::r_s01_common::supported;
use crate::r_s03_common::to_blockers;
use crate::r_s25_common::cast_new;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Whether `id` is an attacking creature in the current combat.
fn attacking(t: &TestGame, id: ObjectId) -> bool {
    let id = t.g.current(id);
    t.g.combat
        .as_ref()
        .is_some_and(|c| c.attackers.iter().any(|a| a.id == id))
}

#[test]
fn damage_can_go_to_the_player_even_if_the_blocker_has_protection() {
    cr!("510.1b");
    ruling!(
        "Thorn Elemental",
        "You can decide to assign damage to the defending player or planeswalker even if the blocking creature has protection from green or damage preventing effects on it."
    );
    supported("Thorn Elemental");
    let mut t = TestGame::new(2);
    let thorn = t.battlefield(P0, "Thorn Elemental");
    // Mirran Crusader: 2/2 double strike, protection from black and from green.
    let crusader = t.battlefield(P1, "Mirran Crusader");
    t.answer_yes(P0, true);
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(thorn, Entity::Player(P1))], &[(crusader, thorn)]);
    // All 7 damage was assigned to P1 as though Thorn Elemental weren't blocked.
    assert_eq!(t.life(P1), 13);
    assert_eq!(t.obj_now(crusader).damage, 0);
    assert_eq!(t.obj_now(thorn).damage, 4);
}

#[test]
fn destroying_the_blocker_doesnt_make_the_attacker_unblocked() {
    cr!("509.1h", "506.4", "510.1c", "702.19d");
    ruling!(
        "Divine Verdict",
        "Destroying a blocking creature won't cause any of the creatures it was blocking to become unblocked. They won't deal combat damage to the defending player or planeswalker (unless they have trample)."
    );
    supported("Divine Verdict");
    for trample in [false, true] {
        let mut t = TestGame::new(2);
        let attacker = if trample {
            t.battlefield(P0, "Colossal Dreadmaw")
        } else {
            t.battlefield(P0, "Hill Giant")
        };
        let blocker = t.battlefield(P1, "Grizzly Bears");
        to_blockers(&mut t, &[(attacker, Entity::Player(P1))], &[(blocker, attacker)]);
        // "Destroy target attacking or blocking creature."
        cast_new(&mut t, P0, "Divine Verdict", &[Entity::Object(blocker)]);
        t.resolve_all();
        assert!(t.in_graveyard(P1, "Grizzly Bears"));
        t.advance_to(P0, Step::EndOfCombat);
        if trample {
            // All 6 of the trampler's damage is assigned to the player.
            assert_eq!(t.life(P1), 14);
        } else {
            assert_eq!(t.life(P1), 20);
        }
    }
}

#[test]
fn a_creature_with_power_0_or_less_assigns_no_combat_damage() {
    cr!("510.1a");
    ruling!(
        "Chant of the Skifsang",
        "A creature with power 0 or less assigns no combat damage. (It doesn’t assign a negative amount of combat damage.)"
    );
    supported("Chant of the Skifsang");
    let mut t = TestGame::new(2);
    // Hill Giant with Chant of the Skifsang ("Enchanted creature gets -13/-0"): -10/3.
    let giant = t.battlefield(P1, "Hill Giant");
    let chant = t.battlefield(P0, "Chant of the Skifsang");
    t.g.attach(chant, Entity::Object(giant));
    t.g.recompute();
    assert_eq!(t.pt(giant), (-10, 3));
    t.set_step(P1, Step::BeginningOfCombat);
    t.attack(&[(giant, Entity::Player(P0))], &[]);
    // Unblocked: no damage to P0, and no negative damage (P0 gains no life).
    assert_eq!(t.life(P0), 20);
    // Blocked: the blocker isn't dealt damage either.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let chant = t.battlefield(P0, "Chant of the Skifsang");
    t.g.attach(chant, Entity::Object(giant));
    t.g.recompute();
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P1, Step::BeginningOfCombat);
    t.attack(&[(giant, Entity::Player(P0))], &[(bears, giant)]);
    assert_eq!(t.obj_now(bears).damage, 0);
    assert_eq!(t.obj_now(giant).damage, 2);
}

#[test]
fn maze_of_ith_leaves_the_creature_attacking_with_its_damage_prevented() {
    cr!("506.4", "615.1a");
    ruling!(
        "Maze of Ith",
        "The creature isn't removed from combat; it just has its damage prevented. It's still an attacking creature until the combat phase is complete."
    );
    supported("Maze of Ith");
    let mut t = TestGame::new(2);
    let maze = t.battlefield(P1, "Maze of Ith");
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    to_blockers(&mut t, &[(giant, Entity::Player(P1))], &[(bears, giant)]);
    // "{T}: Untap target attacking creature. Prevent all combat damage that would be dealt
    // to and dealt by that creature this turn."
    t.activate(P1, maze, 0, &[Entity::Object(giant)]).unwrap();
    t.resolve_all();
    assert!(!t.obj_now(giant).tapped);
    assert!(attacking(&t, giant));
    t.advance_to(P0, Step::CombatDamage);
    t.settle();
    // No damage to or from it; it's still attacking, so it's still a legal target for
    // "destroy target attacking or blocking creature".
    assert_eq!(t.obj_now(bears).damage, 0);
    assert_eq!(t.obj_now(giant).damage, 0);
    assert!(attacking(&t, giant));
    cast_new(&mut t, P1, "Divine Verdict", &[Entity::Object(giant)]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Hill Giant"));
}

#[test]
fn laccolith_damage_is_optional_and_dealt_in_the_declare_blockers_step() {
    cr!("509.2", "510.4", "603.2");
    ruling!(
        "Laccolith Grunt",
        "The ability is optional. You can decide to allow it to deal combat damage as normal."
    );
    ruling!(
        "Laccolith Grunt",
        "The ability allows you to deal damage during the declare blockers step of combat, which is well before even first strike creatures deal damage."
    );
    supported("Laccolith Grunt");
    // Blocked by White Knight (2/2 first strike): the Grunt deals 2 damage to it as the
    // trigger resolves in the declare blockers step, before first-strike damage.
    let mut t = TestGame::new(2);
    let grunt = t.battlefield(P0, "Laccolith Grunt");
    let knight = t.battlefield(P1, "White Knight");
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(knight)]);
    to_blockers(&mut t, &[(grunt, Entity::Player(P1))], &[(knight, grunt)]);
    assert_eq!(t.g.turn.step, Step::DeclareBlockers);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "White Knight"));
    t.advance_to(P0, Step::EndOfCombat);
    assert!(t.on_battlefield(grunt));
    assert_eq!(t.obj_now(grunt).damage, 0);
    assert_eq!(t.life(P1), 20);
    // Declining: it deals its combat damage as normal.
    let mut t = TestGame::new(2);
    let grunt = t.battlefield(P0, "Laccolith Grunt");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_yes(P0, false);
    to_blockers(&mut t, &[(grunt, Entity::Player(P1))], &[(bears, grunt)]);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    t.advance_to(P0, Step::EndOfCombat);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(t.in_graveyard(P0, "Laccolith Grunt"));
}

#[test]
fn farrels_zealots_damage_in_declare_blockers_removes_a_blocker_first() {
    cr!("509.2", "510.1c");
    ruling!(
        "Farrel's Zealot",
        "This ability resolves during the declare blockers step. If the damage dealt this way is enough to destroy an attacking or blocking creature, that other creature won’t be around to deal its combat damage."
    );
    supported("Farrel's Zealot");
    let mut t = TestGame::new(2);
    let zealot = t.battlefield(P0, "Farrel's Zealot");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    // "Whenever this creature attacks and isn't blocked, you may have it deal 3 damage to
    // target creature. If you do, this creature assigns no combat damage this turn."
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(giant)]);
    to_blockers(
        &mut t,
        &[(zealot, Entity::Player(P1)), (bears, Entity::Player(P1))],
        &[(giant, bears)],
    );
    assert_eq!(t.g.turn.step, Step::DeclareBlockers);
    t.resolve_all();
    // The Giant blocking the Bears is destroyed before combat damage: it deals none, and
    // the Bears stay blocked.
    assert!(t.in_graveyard(P1, "Hill Giant"));
    t.advance_to(P0, Step::EndOfCombat);
    assert!(t.on_battlefield(bears));
    assert_eq!(t.obj_now(bears).damage, 0);
    // The Zealot assigned no combat damage; the Bears were blocked.
    assert_eq!(t.life(P1), 20);
}

#[test]
fn a_creature_sacrificed_in_the_declare_blockers_step_deals_no_combat_damage() {
    cr!("510.1", "704.5g", "510.2");
    ruling!(
        "Collateral Damage",
        "If you sacrifice an attacking or blocking creature during the declare blockers step, it won’t deal combat damage. If you wait until the combat damage step, but that creature is dealt lethal damage, it’ll be destroyed before you get a chance to sacrifice it."
    );
    supported("Collateral Damage");
    // Sacrificed during the declare blockers step: the Bears deal no combat damage.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let lions = t.battlefield(P1, "Savannah Lions");
    to_blockers(&mut t, &[(bears, Entity::Player(P1))], &[(lions, bears)]);
    t.answer_choose(P0, &[Entity::Object(bears)]);
    cast_new(&mut t, P0, "Collateral Damage", &[Entity::Player(P1)]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(t.life(P1), 17);
    t.advance_to(P0, Step::EndOfCombat);
    assert!(t.on_battlefield(lions));
    assert_eq!(t.obj_now(lions).damage, 0);
    // Waiting until the combat damage step: the Bears were dealt lethal damage and were
    // destroyed before anyone gets priority, so they can't be sacrificed.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    to_blockers(&mut t, &[(bears, Entity::Player(P1))], &[(giant, bears)]);
    t.advance_to(P0, Step::CombatDamage);
    t.settle();
    assert!(!t.on_battlefield(bears));
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(t.obj_now(giant).damage, 2);
}

#[test]
fn destroy_at_end_of_combat_can_require_a_second_regeneration() {
    cr!("511.2", "701.19a", "510.2");
    ruling!(
        "Thicket Basilisk",
        "The ability destroys the creature at the end of the combat, which is after all first strike and normal damage dealing is done. This means that a creature may have to regenerate twice to survive the combat, once from damage and once again at end of combat."
    );
    supported("Thicket Basilisk");
    for shields in [1, 2] {
        let mut t = TestGame::new(2);
        // River Boa (2/1, "{G}: Regenerate River Boa.") attacks; Thicket Basilisk (2/4)
        // blocks it: "Whenever this creature blocks or becomes blocked by a non-Wall
        // creature, destroy that creature at end of combat."
        let boa = t.battlefield(P0, "River Boa");
        let basilisk = t.battlefield(P1, "Thicket Basilisk");
        t.lands(P0, "Forest", shields);
        to_blockers(&mut t, &[(boa, Entity::Player(P1))], &[(basilisk, boa)]);
        t.resolve_all();
        for _ in 0..shields {
            t.activate(P0, boa, 0, &[]).unwrap();
            t.resolve_all();
        }
        // Combat damage: the Basilisk's 2 damage is lethal; the first shield is used.
        t.advance_to(P0, Step::CombatDamage);
        t.settle();
        assert!(t.on_battlefield(boa));
        assert_eq!(t.obj_now(boa).damage, 0);
        // End of combat: the delayed trigger destroys it again.
        t.advance_to(P0, Step::EndOfCombat);
        t.resolve_all();
        assert_eq!(t.on_battlefield(boa), shields == 2);
    }
}

#[test]
fn the_source_of_combat_damage_is_the_creature_that_dealt_it() {
    cr!("510.2", "120.3");
    ruling!(
        "Retaliator Griffin",
        "The source of combat damage is the creature that dealt it."
    );
    supported("Retaliator Griffin");
    let mut t = TestGame::new(2);
    // "Whenever a source an opponent controls deals damage to you, you may put that many
    // +1/+1 counters on Retaliator Griffin."
    let griffin = t.battlefield(P0, "Retaliator Griffin");
    // P1's Bears wear P0's Rancor (+2/+0 and trample): the creature, which P1 controls, is
    // the source of its combat damage, not the Aura P0 controls.
    let bears = t.battlefield(P1, "Grizzly Bears");
    let rancor = t.battlefield(P0, "Rancor");
    t.g.attach(rancor, Entity::Object(bears));
    t.g.recompute();
    t.answer_yes(P0, true);
    t.set_step(P1, Step::BeginningOfCombat);
    t.attack(&[(bears, Entity::Player(P0))], &[]);
    t.resolve_all();
    assert_eq!(t.life(P0), 16);
    assert_eq!(t.counters(griffin, "+1/+1"), 4);
}
