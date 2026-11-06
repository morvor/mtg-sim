//! Rulings batch P059 — "attacks this turn (or each combat) if able" requirements (CR
//! 508.1d): a creature that's tapped, can't attack, or is affected by "summoning sickness"
//! (CR 302.6) doesn't attack, and one whose attack has a cost isn't forced to pay it; the
//! controller still chooses what it attacks (CR 508.1a); the requirement applies to
//! creatures that come later (CR 611.2c doesn't lock "each creature" requirements of
//! rules-changing effects), and to a creature with haste the turn it arrives.

use crate::r_p058_common::mana;
use crate::r_p059_common::*;
use crate::r_s01_common::supported;
use crate::r_s06_common::attach_new;
use crate::r_s09_common::{attacks_now, declare, legal_attack, to_combat};
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[derive(Clone, Copy, Debug, PartialEq)]
enum Unable {
    /// Nothing stops it.
    No,
    Tapped,
    /// Enchanted by Pacifism ("Enchanted creature can't attack or block.").
    CantAttack,
    /// It came under its controller's control this turn.
    Sick,
    /// The defending player controls Ghostly Prison ("Creatures can't attack you unless
    /// their controller pays {2} for each creature they control that's attacking you.");
    /// its controller has the mana.
    Cost,
}

/// Whether the requirement of `name` is the creature's own (or its controller's
/// permanent's): P0's creature attacks P1. Otherwise P0 applies it to P1's creature on
/// P1's turn, and it would attack P0 (P2 for Dulcet Sirens, which targets P2).
fn own_requirement(name: &str) -> bool {
    matches!(
        name,
        "Zurgo Helmsmasher" | "Goblin Rabblemaster" | "Hellraiser Goblin" | "Goblin Assault"
    )
}

/// P0 applies the requirement of the card `name` to the creature `c` (P0 has the mana;
/// the game is in the precombat main phase of `c`'s controller's turn).
fn apply(t: &mut TestGame, name: &str, c: ObjectId) {
    supported(name);
    mana(t, P0, 2);
    match name {
        "Zurgo Helmsmasher" => {}
        "Goblin Rabblemaster" | "Hellraiser Goblin" | "Goblin Assault" => {
            t.battlefield(P0, name);
        }
        "Alluring Siren" | "Courtly Provocateur" | "Rage Nimbus" | "Basandra, Battle Seraph"
        | "Heckling Fiends" => {
            let s = t.battlefield(P0, name);
            t.activate(P0, s, 0, &[o(c)]).unwrap();
        }
        "Dulcet Sirens" => {
            let s = t.battlefield(P0, name);
            t.activate(P0, s, 0, &[o(c), pl(P2)]).unwrap();
        }
        "Goblin Diplomats" => {
            let s = t.battlefield(P0, name);
            t.activate(P0, s, 0, &[]).unwrap();
        }
        "Bullwhip" => {
            let s = t.battlefield(P0, name);
            t.activate(P0, s, 0, &[o(c)]).unwrap();
        }
        "Instigator" => {
            let s = t.battlefield(P0, name);
            t.hand(P0, "Grizzly Bears");
            let p = t.obj_now(c).controller;
            t.activate(P0, s, 0, &[pl(p)]).unwrap();
        }
        "Hustle // Bustle" => {
            let s = t.hand(P0, name);
            t.cast(P0, s).method(CastMethod::Half(0)).target(c).go();
        }
        _ => {
            cast(t, P0, name, &[o(c)]);
        }
    }
    t.resolve_all();
}

/// Sets up `name`'s requirement on a creature that's `why` unable, has its controller
/// declare no attackers, and returns whether the creature attacked anyway (the engine
/// enforces the requirements, CR 508.1d).
fn attacks(name: &str, why: Unable) -> bool {
    let n = if name == "Dulcet Sirens" { 3 } else { 2 };
    let mut t = TestGame::new(n);
    let own = own_requirement(name);
    let (a, d) = if own {
        (P0, P1)
    } else if name == "Dulcet Sirens" {
        (P1, P2)
    } else {
        (P1, P0)
    };
    let cname = match name {
        "Zurgo Helmsmasher" => "Zurgo Helmsmasher",
        "Goblin Rabblemaster" | "Goblin Assault" => "Goblin Piker",
        _ => "Grizzly Bears",
    };
    t.set_step(a, Step::PrecombatMain);
    let c = if why == Unable::Sick {
        t.battlefield_sick(a, cname)
    } else {
        t.battlefield(a, cname)
    };
    t.lands(a, "Plains", 4);
    match why {
        Unable::Tapped => {
            t.g.tap(c);
        }
        Unable::CantAttack => {
            attach_new(&mut t, d, "Pacifism", c);
        }
        Unable::Cost => {
            t.battlefield(d, "Ghostly Prison");
        }
        _ => {}
    }
    t.g.recompute();
    apply(&mut t, name, c);
    let attacks = declare(&mut t, a, &[]);
    let attacked = attacks.iter().any(|(x, _)| *x == t.g.current(c));
    if attacked {
        assert_eq!(
            attacks.iter().find(|(x, _)| *x == c).map(|x| x.1),
            Some(pl(d)),
            "{name}"
        );
    }
    attacked
}

/// Checks the cases: it attacks when nothing stops it, and not when it's one of `cases`.
fn check(name: &str, cases: &[Unable]) {
    assert!(attacks(name, Unable::No), "{name}: attacks when able");
    for why in cases {
        assert!(!attacks(name, *why), "{name}: {why:?}");
    }
}

use Unable::*;

#[test]
fn forced_attackers_dont_attack_when_unable_or_when_it_costs() {
    cr!("508.1d", "508.1c", "302.6");
    ruling!(
        "Alluring Siren",
        "After the ability resolves, the targeted creature attacks you only if it’s able to do so as that turn’s declare attackers step begins. If, at that time, the creature is tapped, is affected by a spell or ability that says it can’t attack, or is affected by “summoning sickness,” it can’t attack. If there’s a cost associated with having that creature attack, its controller isn’t forced to pay that cost, so the creature doesn’t have to attack in that case either."
    );
    check("Alluring Siren", &[Tapped, CantAttack, Sick, Cost]);
    ruling!(
        "Courtly Provocateur",
        "After the first ability resolves, the creature attacks only if it’s able to do so as the declare attackers step begins. If, at that time, the creature is tapped, is affected by a spell or ability that says it can’t attack, or hasn’t been under that player’s control continuously since the turn began (and doesn’t have haste), then it doesn’t attack. If there’s a cost associated with having the creature attack, the player isn’t forced to pay that cost, so it doesn’t attack in that case either."
    );
    check("Courtly Provocateur", &[Tapped, CantAttack, Sick, Cost]);
    ruling!(
        "Dulcet Sirens",
        "If, as attackers are declared, the target creature is tapped, is affected by a spell or ability that says it can’t attack, or hasn’t been under its controller’s control continuously since their last turn began (and doesn’t have haste), then it doesn’t attack. If there’s a cost associated with having the creature attack the opponent, its controller isn’t forced to pay that cost, so it doesn’t have to attack that player in that case either."
    );
    check("Dulcet Sirens", &[Tapped, CantAttack, Sick, Cost]);
    ruling!(
        "Chemister's Trick",
        "If, during a player's declare attacker's step, a creature is tapped, is affected by a spell or ability that says it can't attack, or hasn't been under that player's control continuously since the turn began (and doesn't have haste), then it doesn't attack. If there's a cost associated with having a creature attack, the player isn't forced to pay that cost, so it doesn't have to attack in that case either."
    );
    check("Chemister's Trick", &[Tapped, CantAttack, Sick, Cost]);
    ruling!(
        "Incite",
        "If, during its controller’s declare attackers step, a creature affected by Incite is tapped, is affected by a spell or ability that says it can’t attack, or is affected by “summoning sickness,” then that creature doesn’t attack. If there’s a cost associated with having that creature attack, its controller isn’t forced to pay that cost, so the creature doesn’t have to attack in that case either."
    );
    check("Incite", &[Tapped, CantAttack, Sick, Cost]);
    ruling!(
        "Rage Nimbus",
        "If, during its controller’s declare attackers step, a creature affected by Rage Nimbus’s activated ability is tapped, is affected by a spell or ability that says it can’t attack, or is affected by “summoning sickness,” then that creature doesn’t attack. If there’s a cost associated with having that creature attack, its controller isn’t forced to pay that cost, so the creature doesn’t have to attack in that case either."
    );
    check("Rage Nimbus", &[Tapped, CantAttack, Sick, Cost]);
    ruling!(
        "Hustle // Bustle",
        "If the creature targeted by Hustle is tapped or is affected by a spell or ability that says it can't attack or block, then it won't attack or block this turn. If there's a cost associated with having that creature attack or block, its controller isn't forced to pay that cost, so it doesn't have to attack or block in that case either."
    );
    check("Hustle // Bustle", &[Tapped, CantAttack, Cost]);
    ruling!(
        "Goblin Rabblemaster",
        "If, during your declare attackers step, a creature that must attack if able is tapped, is affected by a spell or ability that says it can't attack, or hasn't been under your control continuously since the turn began (and doesn't have haste), then it doesn't attack. If there's a cost associated with having a creature attack, you're not forced to pay that cost, so it doesn't have to attack in that case either."
    );
    check("Goblin Rabblemaster", &[Tapped, CantAttack, Sick, Cost]);
    ruling!(
        "Zurgo Helmsmasher",
        "If, during your declare attackers step, Zurgo is tapped or is affected by a spell or ability that says it can't attack, then it doesn't attack. If there's a cost associated with having Zurgo attack, you aren't forced to pay that cost, so it doesn't have to attack in that case either."
    );
    check("Zurgo Helmsmasher", &[Tapped, CantAttack, Cost]);
    ruling!(
        "Hellraiser Goblin",
        "If, during your declare attackers step, a creature you control is tapped or is affected by a spell or ability that says it can't attack, then it doesn't attack. If there's a cost associated with having a creature attack, you aren't forced to pay that cost, so it doesn't have to attack in that case either."
    );
    check("Hellraiser Goblin", &[Tapped, CantAttack, Cost]);
}

#[test]
fn haste_creatures_that_arrive_this_turn_still_must_attack() {
    cr!("702.10b", "302.6", "508.1d");
    ruling!(
        "Zurgo Helmsmasher",
        "If Zurgo enters the battlefield before the combat phase, it will attack that turn if able. If it enters the battlefield after combat, it won't attack that turn and will usually be available to block on the following turn."
    );
    // Haste (Zurgo's own; Hellraiser Goblin's for every creature P0 controls).
    assert!(attacks("Zurgo Helmsmasher", Sick));
    assert!(attacks("Hellraiser Goblin", Sick));
    // After combat it doesn't attack (there's no more combat), and it's untapped to block
    // on P1's turn: the requirement is only about P0's combats.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PostcombatMain);
    let zurgo = t.enter(P0, "Zurgo Helmsmasher");
    t.resolve_all();
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.advance_to(P1, Step::BeginningOfCombat);
    assert!(!t.obj_now(zurgo).tapped);
    t.answer(
        P1,
        DecisionKind::Attackers,
        mtg_engine::decision::Answer::Attackers(vec![(bears, pl(P0))]),
    );
    run_to(&mut t, Step::DeclareAttackers);
    block(&mut t, P0, &[(zurgo, bears)]);
    assert!(t.g.is_blocking(zurgo), "Zurgo blocks on P1's turn");

    ruling!(
        "Hellraiser Goblin",
        "Hellraiser Goblin's ability also affects itself. If it enters before combat, it will have to attack that combat if able."
    );
    let mut t = TestGame::new(2);
    let hg = t.enter(P0, "Hellraiser Goblin");
    t.resolve_all();
    let attacks = declare(&mut t, P0, &[]);
    assert_eq!(attacks, vec![(hg, pl(P1))]);

    ruling!(
        "Siren's Call",
        "It will require creatures with Haste to attack since they are able, but it won’t destroy them if they don’t for some reason."
    );
    // "Cast this spell only during an opponent's turn, before attackers are declared.
    // Creatures the active player controls attack this turn if able. At the beginning of
    // the next end step, destroy all non-Wall creatures that player controls that didn't
    // attack this turn. Ignore this effect for each creature the player didn't control
    // continuously since the beginning of the turn."
    supported("Siren's Call");
    // A Raging Goblin (haste) that entered this turn must attack...
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let goblin = t.battlefield_sick(P1, "Raging Goblin");
    cast_resolve(&mut t, P0, "Siren's Call", &[]);
    let attacks = declare(&mut t, P1, &[]);
    assert_eq!(attacks, vec![(goblin, pl(P0))]);
    // ...but if it doesn't (here it's tapped), it isn't destroyed.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let goblin = t.battlefield_sick(P1, "Raging Goblin");
    cast_resolve(&mut t, P0, "Siren's Call", &[]);
    t.g.tap(goblin);
    assert!(declare(&mut t, P1, &[]).is_empty());
    end_step_of(&mut t, P1);
    assert!(t.on_battlefield(goblin), "not destroyed");
}

#[test]
fn creatures_that_cant_attack_are_destroyed_by_the_delayed_trigger() {
    cr!("508.1d", "603.7");
    ruling!(
        "Nettling Imp",
        "The creature is destroyed if it does not attack because it simply can’t do so legally."
    );
    ruling!(
        "Norritt",
        "The creature is destroyed if it does not attack because it simply can’t do so legally."
    );
    ruling!(
        "Siren's Call",
        "The creature is destroyed if it does not attack because it simply can’t do so legally."
    );
    ruling!(
        "Siren's Call",
        "This will destroy creatures that weren’t able to attack because they had been previously tapped."
    );
    for name in ["Nettling Imp", "Norritt", "Siren's Call"] {
        supported(name);
        // A tapped Grizzly Bears and a pacified Hill Giant can't attack; an untapped Gray
        // Ogre attacks.
        let mut t = TestGame::new(2);
        mana(&mut t, P0, 2);
        t.set_step(P1, Step::PrecombatMain);
        let bears = t.battlefield(P1, "Grizzly Bears");
        let giant = t.battlefield(P1, "Hill Giant");
        let ogre = t.battlefield(P1, "Gray Ogre");
        t.g.tap(bears);
        attach_new(&mut t, P0, "Pacifism", giant);
        if name == "Siren's Call" {
            cast_resolve(&mut t, P0, name, &[]);
        } else {
            let s = t.battlefield(P0, name);
            let i = crate::r_p040_common::ability(&mut t, s, "{T}: Choose");
            for c in [bears, giant, ogre] {
                t.activate(P0, s, i, &[o(c)]).unwrap();
                t.resolve_all();
                t.g.untap(s);
            }
        }
        let attacks = declare(&mut t, P1, &[]);
        assert_eq!(attacks, vec![(ogre, pl(P0))], "{name}");
        end_step_of(&mut t, P1);
        assert!(t.in_graveyard(P1, "Grizzly Bears"), "{name}: tapped");
        assert!(t.in_graveyard(P1, "Hill Giant"), "{name}: can't attack");
        assert!(t.on_battlefield(ogre), "{name}: attacked");
    }
}

#[test]
fn instigator_has_no_penalty_for_creatures_that_cant_attack() {
    cr!("508.1d");
    ruling!("Instigator", "There is no penalty if the creatures are unable to attack.");
    // "{1}{B}{B}, {T}, Discard a card: Creatures target player controls attack this turn
    // if able."
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let ogre = t.battlefield(P1, "Gray Ogre");
    t.g.tap(bears);
    apply(&mut t, "Instigator", ogre);
    let attacks = declare(&mut t, P1, &[]);
    assert_eq!(attacks, vec![(ogre, pl(P0))]);
    end_step_of(&mut t, P1);
    assert!(t.on_battlefield(bears), "nothing happens to it");
    t.advance_to(P0, Step::Upkeep);
    assert!(t.on_battlefield(bears));
}

#[test]
fn bullwhip_forces_an_attack_only_if_the_creature_can() {
    cr!("508.1d");
    ruling!("Bullwhip", "If the creature can attack, it must. If it cannot, then nothing happens.");
    // "{2}, {T}: This artifact deals 1 damage to target creature. That creature attacks
    // this turn if able."
    assert!(attacks("Bullwhip", No));
    assert!(!attacks("Bullwhip", Tapped));
    // Nothing else happens to it.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let giant = t.battlefield(P1, "Hill Giant");
    t.g.tap(giant);
    apply(&mut t, "Bullwhip", giant);
    assert!(declare(&mut t, P1, &[]).is_empty());
    end_step_of(&mut t, P1);
    assert!(t.on_battlefield(giant));
    assert_eq!(t.obj_now(giant).damage, 1);
}

#[test]
fn the_controller_chooses_what_a_forced_attacker_attacks() {
    cr!("508.1a", "508.1d");
    ruling!(
        "Chemister's Trick",
        "The controller of a creature that attacks if able still chooses which player or planeswalker it attacks."
    );
    ruling!(
        "Goblin Diplomats",
        "The controller of each creature still decides which player or planeswalker that creature attacks."
    );
    ruling!(
        "Courtly Provocateur",
        "The controller of the creature targeted by the first ability chooses which player or planeswalker that creature attacks."
    );
    ruling!(
        "Heckling Fiends",
        "The creature’s controller still decides which player or planeswalker the creature attacks."
    );
    ruling!(
        "Zurgo Helmsmasher",
        "You still choose which player or planeswalker Zurgo Helmsmasher attacks."
    );
    for name in [
        "Chemister's Trick",
        "Goblin Diplomats",
        "Courtly Provocateur",
        "Heckling Fiends",
        "Zurgo Helmsmasher",
    ] {
        for at_walker in [false, true] {
            let mut t = TestGame::new(2);
            let own = own_requirement(name);
            let (a, d) = if own { (P0, P1) } else { (P1, P0) };
            t.set_step(a, Step::PrecombatMain);
            let walker = t.battlefield(d, "Garruk Wildspeaker");
            let c = if name == "Zurgo Helmsmasher" {
                t.battlefield(P0, name)
            } else {
                t.battlefield(a, "Grizzly Bears")
            };
            apply(&mut t, name, c);
            let target = if at_walker { o(walker) } else { pl(d) };
            let attacks = declare(&mut t, a, &[(c, target)]);
            assert_eq!(attacks, vec![(c, target)], "{name}: {at_walker}");
        }
    }
}

#[test]
fn alluring_sirens_creature_must_attack_you_not_your_planeswalker() {
    cr!("508.1d", "508.1a");
    ruling!(
        "Alluring Siren",
        "If the targeted creature would be able to attack either you or a planeswalker you control, it must attack you, not the planeswalker."
    );
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let walker = t.battlefield(P0, "Garruk Wildspeaker");
    let bears = t.battlefield(P1, "Grizzly Bears");
    apply(&mut t, "Alluring Siren", bears);
    to_combat(&mut t, P1);
    assert!(!legal_attack(&mut t, &[(bears, o(walker))]));
    assert!(legal_attack(&mut t, &[(bears, pl(P0))]));
    let attacks = declare(&mut t, P1, &[(bears, o(walker))]);
    assert_eq!(attacks, vec![(bears, pl(P0))]);
}

#[test]
fn alluring_siren_does_nothing_after_attackers_or_on_your_own_turn() {
    cr!("508.1d", "611.2a");
    ruling!(
        "Alluring Siren",
        "Activating the ability during your opponent’s turn after attackers have been declared will have no effect. The same is true if you activate the ability during your own turn."
    );
    // On P1's turn, after attackers were declared (none): the Bears don't join combat, and
    // the requirement is gone on P1's next turn.
    let mut t = TestGame::new(2);
    let siren = t.battlefield(P0, "Alluring Siren");
    let bears = t.battlefield(P1, "Grizzly Bears");
    assert!(declare(&mut t, P1, &[]).is_empty());
    t.activate(P0, siren, 0, &[o(bears)]).unwrap();
    t.resolve_all();
    assert!(!t.g.is_attacking(bears));
    end_step_of(&mut t, P1);
    t.advance_to(P1, Step::PrecombatMain);
    assert!(declare(&mut t, P1, &[]).is_empty(), "not on a later turn");
    // On P0's own turn: P1's creature can't attack on P0's turn, and isn't forced on P1's.
    let mut t = TestGame::new(2);
    let siren = t.battlefield(P0, "Alluring Siren");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.activate(P0, siren, 0, &[o(bears)]).unwrap();
    t.resolve_all();
    t.advance_to(P1, Step::PrecombatMain);
    assert!(declare(&mut t, P1, &[]).is_empty());
}

#[test]
fn dulcet_sirens_creature_must_attack_the_target_opponent() {
    cr!("508.1d", "508.1a", "115.1");
    ruling!(
        "Dulcet Sirens",
        "The target creature must attack the target opponent if able, not a planeswalker controlled by that player."
    );
    let mut t = TestGame::new(3);
    t.set_step(P1, Step::PrecombatMain);
    let walker = t.battlefield(P2, "Garruk Wildspeaker");
    let bears = t.battlefield(P1, "Grizzly Bears");
    apply(&mut t, "Dulcet Sirens", bears);
    to_combat(&mut t, P1);
    assert!(!legal_attack(&mut t, &[(bears, o(walker))]));
    assert!(!legal_attack(&mut t, &[(bears, pl(P0))]));
    assert_eq!(declare(&mut t, P1, &[(bears, o(walker))]), vec![(bears, pl(P2))]);

    ruling!(
        "Dulcet Sirens",
        "“Target opponent” means one of your opponents. In other words, Dulcet Sirens’s ability can’t make a creature attack you."
    );
    let mut t = TestGame::new(3);
    let sirens = t.battlefield(P0, "Dulcet Sirens");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Island", 1);
    let players = ability_target_choices(&mut t, sirens, 0, 1);
    assert!(players.contains(&pl(P2)) && players.contains(&pl(P1)));
    assert!(!players.contains(&pl(P0)), "{players:?}");
    let _ = bears;

    ruling!(
        "Dulcet Sirens",
        "If the creature can’t attack the player for one of the above reasons but it can still attack elsewhere, its controller may choose to have it attack another player, attack a planeswalker, or not attack at all."
    );
    // P2 has Ghostly Prison: P1's Bears may attack P0, P2's planeswalker, or nothing.
    for choice in 0..3 {
        let mut t = TestGame::new(3);
        t.set_step(P1, Step::PrecombatMain);
        t.battlefield(P2, "Ghostly Prison");
        let walker = t.battlefield(P2, "Garruk Wildspeaker");
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.lands(P1, "Plains", 2);
        apply(&mut t, "Dulcet Sirens", bears);
        let decl = match choice {
            0 => vec![],
            1 => vec![(bears, pl(P0))],
            _ => vec![(bears, o(walker))],
        };
        assert_eq!(declare(&mut t, P1, &decl), decl, "{choice}");
    }

    ruling!(
        "Dulcet Sirens",
        "If either the target creature or the target player is an illegal target when the activated ability tries to resolve, the ability will have no effect. The creature won’t be forced to attack that opponent, but it still can if able."
    );
    // The creature gains hexproof in response (Ranger's Guile), or the player gains
    // shroud (Gilded Light, cast by P2).
    for illegal_player in [false, true] {
        let mut t = TestGame::new(3);
        t.set_step(P1, Step::PrecombatMain);
        let sirens = t.battlefield(P0, "Dulcet Sirens");
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.lands(P0, "Island", 1);
        t.activate(P0, sirens, 0, &[o(bears), pl(P2)]).unwrap();
        if illegal_player {
            cast(&mut t, P2, "Gilded Light", &[]);
        } else {
            cast(&mut t, P1, "Ranger's Guile", &[o(bears)]);
        }
        t.resolve_all();
        assert!(t.g.stack.is_empty());
        assert!(declare(&mut t, P1, &[]).is_empty(), "not forced");
        // It still can attack P2.
        let mut t2 = TestGame::new(3);
        t2.set_step(P1, Step::PrecombatMain);
        let sirens = t2.battlefield(P0, "Dulcet Sirens");
        let bears = t2.battlefield(P1, "Grizzly Bears");
        t2.lands(P0, "Island", 1);
        t2.activate(P0, sirens, 0, &[o(bears), pl(P2)]).unwrap();
        if illegal_player {
            cast(&mut t2, P2, "Gilded Light", &[]);
        } else {
            cast(&mut t2, P1, "Ranger's Guile", &[o(bears)]);
        }
        t2.resolve_all();
        assert_eq!(
            declare(&mut t2, P1, &[(bears, pl(P2))]),
            vec![(bears, pl(P2))]
        );
    }
}

#[test]
fn requirements_for_each_creature_apply_to_creatures_that_come_later() {
    cr!("508.1d", "611.2c");
    ruling!(
        "Goblin Diplomats",
        "Creatures that enter the battlefield (or noncreature permanents that become creatures) after the ability resolves are affected by the ability and must attack if able."
    );
    // "{T}: Each creature attacks this turn if able."
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let diplomats = t.battlefield(P0, "Goblin Diplomats");
    t.activate(P0, diplomats, 0, &[]).unwrap();
    t.resolve_all();
    // A creature that arrives later (with haste) and an artifact that becomes a creature.
    let goblin = t.enter(P1, "Raging Goblin");
    t.resolve_all();
    let stone = t.battlefield(P1, "Millstone");
    t.battlefield(P1, "March of the Machines");
    t.g.recompute();
    let attacks = declare(&mut t, P1, &[]);
    assert!(attacks.contains(&(goblin, pl(P0))), "{attacks:?}");
    assert!(attacks.contains(&(stone, pl(P0))), "{attacks:?}");

    ruling!(
        "Goblin Assault",
        "Goblin Assault’s second ability affects all Goblin creatures controlled by all players. It’s not limited to the tokens created with the first ability."
    );
    // "Goblin creatures attack each combat if able."
    supported("Goblin Assault");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Goblin Assault");
    let mine = t.battlefield(P0, "Goblin Piker");
    let theirs = t.battlefield(P1, "Goblin Piker");
    let bears = t.battlefield(P1, "Grizzly Bears");
    assert_eq!(declare(&mut t, P0, &[]), vec![(mine, pl(P1))]);
    t.g.combat = None;
    let attacks = declare(&mut t, P1, &[]);
    assert_eq!(attacks, vec![(theirs, pl(P0))]);
    assert!(!attacks.iter().any(|(c, _)| *c == bears), "not a Goblin");
}

#[test]
fn two_goblin_rabblemasters_force_each_other_to_attack() {
    cr!("508.1d");
    ruling!(
        "Goblin Rabblemaster",
        "Although Goblin Rabblemaster doesn't force itself to attack, if you control two of them, they'll force each other to attack if able."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Goblin Rabblemaster");
    assert!(declare(&mut t, P0, &[]).is_empty(), "one doesn't force itself");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Goblin Rabblemaster");
    let b = t.battlefield(P0, "Goblin Rabblemaster");
    let attacks = declare(&mut t, P0, &[]);
    assert_eq!(attacks.len(), 2);
    assert!(attacks.contains(&(a, pl(P1))) && attacks.contains(&(b, pl(P1))));
}

#[test]
fn goblin_rabblemasters_bonus_is_locked_in_as_it_resolves() {
    cr!("608.2h", "611.2c");
    ruling!(
        "Goblin Rabblemaster",
        "The number of attacking Goblins is counted as the last ability resolves, and the bonus is locked in at that time."
    );
    // "Whenever this creature attacks, it gets +1/+0 until end of turn for each other
    // attacking Goblin." With two other attacking Goblins: +2/+0, even after one leaves.
    let mut t = TestGame::new(2);
    let rabble = t.battlefield(P0, "Goblin Rabblemaster");
    let g1 = t.battlefield(P0, "Goblin Piker");
    let g2 = t.battlefield(P0, "Goblin Piker");
    t.set_step(P0, Step::BeginningOfCombat);
    crate::r_s01_common::attack_with(
        &mut t,
        &[(rabble, pl(P1)), (g1, pl(P1)), (g2, pl(P1))],
    );
    t.resolve_all();
    assert_eq!(t.pt(rabble), (4, 2));
    crate::r_s02_common::destroy(&mut t, g1);
    assert_eq!(t.pt(rabble), (4, 2), "locked in");
}

#[test]
fn incite_targets_and_colors() {
    cr!("115.1", "105.3", "613.1e");
    ruling!(
        "Incite",
        "You may target any creature with Incite. It’s okay if that creature is already red. It’s also okay if it would be unable to attack that turn."
    );
    ruling!(
        "Incite",
        "Incite overwrites all of the targeted creature’s old colors, leaving it just red. It doesn’t matter what colors it used to be (even if, for example, it used to be red and green)."
    );
    use mtg_engine::types::Color;
    // "Target creature becomes red until end of turn and attacks this turn if able."
    let mut t = TestGame::new(2);
    let kird = t.battlefield(P1, "Kird Ape");
    let wall = t.battlefield(P1, "Wall of Stone");
    let mine = t.battlefield_sick(P0, "Grizzly Bears");
    let candidates = crate::r_s04_common::spell_targets(&mut t, P0, "Incite");
    for c in [kird, wall, mine] {
        assert!(candidates.contains(&o(c)), "{candidates:?}");
    }
    // A red and green creature (Boggart Ram-Gang).
    let ram = t.battlefield(P1, "Boggart Ram-Gang");
    assert_eq!(
        crate::r_s05_common::colors(&t, ram),
        crate::r_s17_common::color_set(&[Color::Red, Color::Green])
    );
    cast_resolve(&mut t, P0, "Incite", &[o(ram)]);
    assert_eq!(
        crate::r_s05_common::colors(&t, ram),
        crate::r_s17_common::color_set(&[Color::Red])
    );
    cast_resolve(&mut t, P0, "Incite", &[o(wall)]);
    assert_eq!(
        crate::r_s05_common::colors(&t, wall),
        crate::r_s17_common::color_set(&[Color::Red])
    );
}

#[test]
fn besmirch_targets_any_creature_and_its_creature_must_attack() {
    cr!("701.15b", "508.1d", "115.1");
    ruling!(
        "Besmirch",
        "Besmirch can target any creature, including one you already control or one that's untapped."
    );
    ruling!(
        "Besmirch",
        "The creature you gain control of must attack during your turn if able."
    );
    // "Until end of turn, gain control of target creature and it gains haste. Untap and
    // goad that creature."
    supported("Besmirch");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Hill Giant");
    let candidates = crate::r_s04_common::spell_targets(&mut t, P0, "Besmirch");
    assert!(candidates.contains(&o(mine)) && candidates.contains(&o(theirs)));
    cast_resolve(&mut t, P0, "Besmirch", &[o(theirs)]);
    assert_eq!(t.obj_now(theirs).controller, P0);
    assert_eq!(declare(&mut t, P0, &[]), vec![(theirs, pl(P1))]);
    // P0's own untapped creature.
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    cast_resolve(&mut t, P0, "Besmirch", &[o(mine)]);
    assert_eq!(declare(&mut t, P0, &[]), vec![(mine, pl(P1))]);
}

#[test]
fn illusionists_gambit_additional_combat() {
    cr!("508.1d", "506.4", "500.8", "508.1c");
    ruling!(
        "Illusionist's Gambit",
        "Creatures that didn't attack during the combat phase when Illusionist's Gambit resolved aren't required to attack in the additional combat phase, although they may. Those creatures can attack you or a planeswalker you control."
    );
    ruling!(
        "Illusionist's Gambit",
        "If, during a player's declare attackers step, a creature is tapped, is affected by a spell or ability that says it can't attack, or hasn't been under that player's control continuously since the turn began (and doesn't have haste), then it doesn't attack."
    );
    ruling!(
        "Illusionist's Gambit",
        "If there's a cost associated with having a creature attack, the player isn't forced to pay that cost, so it doesn't have to attack."
    );
    // "Cast this spell only during the declare blockers step on an opponent's turn.
    // Remove all attacking creatures from combat and untap them. After this phase, there
    // is an additional combat phase. Each of those creatures attacks that combat if able.
    // They can't attack you or planeswalkers you control that combat."
    supported("Illusionist's Gambit");
    // P1 attacks P0 with Bears, Ogre and Giant; Elves stays home. In the additional
    // combat, with P2 as the other opponent: the Bears must attack P2; the Ogre is tapped
    // and the Giant pacified, so they don't; the Elves needn't attack but may attack P0.
    for elves_attack in [false, true] {
        let mut t = TestGame::new(3);
        let bears = t.battlefield(P1, "Grizzly Bears");
        let ogre = t.battlefield(P1, "Gray Ogre");
        let giant = t.battlefield(P1, "Hill Giant");
        let elves = t.battlefield(P1, "Llanowar Elves");
        t.set_step(P1, Step::BeginningOfCombat);
        crate::r_s01_common::attack_with(
            &mut t,
            &[(bears, pl(P0)), (ogre, pl(P0)), (giant, pl(P0))],
        );
        block(&mut t, P0, &[]);
        cast_resolve(&mut t, P0, "Illusionist's Gambit", &[]);
        assert!(!t.g.is_attacking(bears), "removed from combat");
        assert!(!t.obj_now(bears).tapped, "untapped");
        t.g.tap(ogre);
        attach_new(&mut t, P2, "Pacifism", giant);
        // (Declaring the Bears' attack too: a declaration that breaks a requirement is
        // replaced by a legal one.)
        let decl = if elves_attack {
            vec![(bears, pl(P2)), (elves, pl(P0))]
        } else {
            vec![]
        };
        t.answer(
            P1,
            DecisionKind::Attackers,
            mtg_engine::decision::Answer::Attackers(decl.clone()),
        );
        let turn = t.g.turn.number;
        let ok = t.g.run_until(10_000, |g| {
            g.turn.step == Step::DeclareAttackers
                && g.turn.stage == mtg_engine::turn::Stage::Priority
                && g.turn.combat_phases >= 2
        });
        assert!(ok && t.g.turn.number == turn, "an additional combat");
        let attacks = attacks_now(&t);
        assert!(attacks.contains(&(bears, pl(P2))), "{attacks:?}");
        assert!(!attacks.iter().any(|(c, _)| *c == ogre || *c == giant));
        assert_eq!(
            attacks.iter().any(|a| *a == (elves, pl(P0))),
            elves_attack,
            "{attacks:?}"
        );
    }
    // A cost: P2 has Ghostly Prison; the Bears aren't forced to pay to attack P2.
    let mut t = TestGame::new(3);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P1, "Plains", 2);
    t.battlefield(P2, "Ghostly Prison");
    t.set_step(P1, Step::BeginningOfCombat);
    crate::r_s01_common::attack_with(&mut t, &[(bears, pl(P0))]);
    block(&mut t, P0, &[]);
    cast_resolve(&mut t, P0, "Illusionist's Gambit", &[]);
    t.answer(
        P1,
        DecisionKind::Attackers,
        mtg_engine::decision::Answer::Attackers(vec![]),
    );
    let ok = t.g.run_until(10_000, |g| {
        g.turn.step == Step::DeclareAttackers
            && g.turn.stage == mtg_engine::turn::Stage::Priority
            && g.turn.combat_phases >= 2
    });
    assert!(ok);
    assert!(attacks_now(&t).is_empty());
}
