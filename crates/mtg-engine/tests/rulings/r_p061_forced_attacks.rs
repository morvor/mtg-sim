//! Rulings batch P061 — creatures that attack each combat if able (CR 508.1d): one that's
//! tapped, can't attack, or came under its controller's control this turn doesn't attack,
//! and one whose attack has a cost isn't forced to pay it (CR 508.1c–d); its controller
//! still chooses what it attacks (CR 508.1a) unless it must attack a chosen player; a
//! requirement with an "unless" condition is checked as attackers are declared; Cogwork
//! Tracker's noted players (CR 905.2b).

use crate::r_p061_common::*;
use mtg_engine::card::card;
use mtg_engine::draft::{DraftInfo, DraftNote};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[derive(Clone, Copy, Debug, PartialEq)]
enum Unable {
    /// Nothing stops it.
    No,
    Tapped,
    /// Enchanted by the opponent's Pacifism ("Enchanted creature can't attack or block.").
    CantAttack,
    /// It came under its controller's control this turn.
    Sick,
    /// The opponent controls Ghostly Prison ("Creatures can't attack you unless their
    /// controller pays {2} for each creature they control that's attacking you."); its
    /// controller has the mana.
    Cost,
}
use Unable::*;

/// P0's creature `name` (P0's own requirement), `why` unable; P0 declares no attackers.
/// Returns whether it attacked anyway (the engine enforces the requirement, CR 508.1d).
fn attacks(name: &str, why: Unable) -> bool {
    supported(name);
    let mut t = TestGame::new(2);
    let c = if why == Sick {
        t.battlefield_sick(P0, name)
    } else {
        t.battlefield(P0, name)
    };
    t.lands(P0, "Plains", 4);
    match why {
        Tapped => {
            t.g.tap(c);
        }
        CantAttack => {
            attach_new(&mut t, P1, "Pacifism", c);
        }
        Cost => {
            t.battlefield(P1, "Ghostly Prison");
        }
        _ => {}
    }
    t.g.recompute();
    let attacks = declare_from(&mut t, P0, &[]);
    let attacked = attacks.iter().any(|(x, _)| *x == c);
    if attacked {
        assert_eq!(attacks, vec![(c, pl(P1))], "{name}");
    }
    attacked
}

/// It attacks when nothing stops it, and not when it's one of `cases`.
fn check(name: &str, cases: &[Unable]) {
    assert!(attacks(name, No), "{name}: attacks when able");
    for why in cases {
        assert!(!attacks(name, *why), "{name}: {why:?}");
    }
}

#[test]
fn forced_attackers_dont_attack_when_unable_or_when_it_costs() {
    cr!("508.1d", "508.1c", "302.6");
    ruling!(
        "Amarant Coral",
        "If Amarant Coral can't attack for any reason (such as being tapped or having come under your control that turn), then it doesn't attack. If there's a cost associated with having it attack, you're not forced to pay that cost, so it doesn't have to attack in that case either."
    );
    check("Amarant Coral", &[Tapped, CantAttack, Sick, Cost]);
    ruling!(
        "Haktos the Unscarred",
        "If Haktos can't attack for any reason (such as being tapped or having come under that player's control that turn), then it doesn't attack. If there's a cost associated with having it attack, you aren't forced to pay that cost, so it doesn't have to attack in that case either."
    );
    // (Put onto the battlefield without its "as enters" choice, it has no protection, so
    // Pacifism can enchant it.)
    check("Haktos the Unscarred", &[Tapped, CantAttack, Sick, Cost]);
    ruling!(
        "Rubblebelt Recluse",
        "If Rubblebelt Recluse can’t attack for any reason (such as being tapped or having come under that player’s control that turn), then it doesn’t attack. If there’s a cost associated with having it attack, its controller isn’t forced to pay that cost, so it doesn’t have to attack in that case either."
    );
    check("Rubblebelt Recluse", &[Tapped, CantAttack, Sick, Cost]);
    ruling!(
        "Ramroller",
        "If, during its controller's declare attackers step, Ramroller is tapped or is affected by a spell or ability that says it can't attack, then Ramroller doesn't attack. If there's a cost associated with having it attack, its controller isn't forced to pay that cost. If they don't, Ramroller doesn't have to attack."
    );
    check("Ramroller", &[Tapped, CantAttack, Cost]);
    ruling!(
        "Reckless Cohort",
        "If, during its controller’s declare attackers step, that player doesn’t control another Ally but Reckless Cohort is tapped, is affected by a spell or ability that says it can’t attack, or hasn’t been under that player’s control continuously since the turn began (and doesn’t have haste), then it doesn’t attack. If there’s a cost associated with having Reckless Cohort attack, its controller isn’t forced to pay that cost. If they don’t, Reckless Cohort doesn’t have to attack."
    );
    check("Reckless Cohort", &[Tapped, CantAttack, Sick, Cost]);
    ruling!(
        "Berserkers of Blood Ridge",
        "If, during your declare attackers step, Berserkers of Blood Ridge is tapped, is affected by a spell or ability that says it can’t attack, or is affected by “summoning sickness,” then it doesn’t attack. If there’s a cost associated with having Berserkers of Blood Ridge attack, you aren’t forced to pay that cost, so it doesn’t have to attack in that case either."
    );
    check(
        "Berserkers of Blood Ridge",
        &[Tapped, CantAttack, Sick, Cost],
    );
    ruling!(
        "Cogwork Tracker",
        "If, during your declare attackers step, Cogwork Tracker is tapped, is affected by a spell or ability that says it can't attack, or hasn't been under your control continuously since the turn began (and doesn't have haste), then it doesn't attack. If there's a cost associated with having a creature attack, you're not forced to pay that cost, so it doesn't have to attack in that case either."
    );
    check("Cogwork Tracker", &[Tapped, CantAttack, Sick, Cost]);
    ruling!(
        "Flameborn Hellion",
        "If, during your declare attackers step, Flameborn Hellion is tapped, is affected by a spell or ability that says it can't attack, or is affected by \"summoning sickness,\" then it doesn't attack. If there's a cost associated with having Flameborn Hellion attack, you aren't forced to pay that cost, so it doesn't have to attack in that case either."
    );
    // It has haste, so it's never affected by "summoning sickness": it attacks the turn
    // it arrives.
    check("Flameborn Hellion", &[Tapped, CantAttack, Cost]);
    assert!(attacks("Flameborn Hellion", Sick));
    ruling!(
        "Frontline Rebel",
        "If, during your declare attackers step, Frontline Rebel is tapped or is affected by a spell or ability that says it can’t attack, then it doesn’t attack. If there’s a cost associated with having it attack, you aren’t forced to pay that cost, so it doesn’t have to attack in that case either."
    );
    check("Frontline Rebel", &[Tapped, CantAttack, Cost]);
    ruling!(
        "Manticore Eternal",
        "If, during your declare attackers step, Manticore Eternal is tapped or is affected by a spell or ability that says it can't attack, then it doesn't attack. If there's a cost associated with having it attack, you aren't forced to pay that cost, so it doesn't have to attack in that case either."
    );
    check("Manticore Eternal", &[Tapped, CantAttack, Cost]);
    ruling!(
        "Marauding Maulhorn",
        "If, during your declare attackers step, Marauding Maulhorn is tapped, is affected by a spell or ability that says it can’t attack, or hasn’t been under your control continuously since the turn began (and doesn’t have haste), then it doesn’t attack. If there’s a cost associated with having Marauding Maulhorn attack, you aren’t forced to pay that cost, so it doesn’t have to attack in that case either."
    );
    check("Marauding Maulhorn", &[Tapped, CantAttack, Sick, Cost]);
    ruling!(
        "Valley Dasher",
        "If, during your declare attackers step, Valley Dasher is tapped or is affected by a spell or ability that says it can't attack, then it doesn't attack. If there's a cost associated with having Valley Dasher attack, you're not forced to pay that cost, so it doesn't have to attack in that case either."
    );
    check("Valley Dasher", &[Tapped, CantAttack, Cost]);
    ruling!(
        "Impetuous Sunchaser",
        "Impetuous Sunchaser attacks only if it's able to do so as the declare attackers step begins. If, at that time, it's tapped or affected by a spell or ability that says it can't attack, then it doesn't attack. If there's a cost associated with having it attack, you're not forced to pay that cost. If that cost isn't paid, Impetuous Sunchaser won't attack."
    );
    check("Impetuous Sunchaser", &[Tapped, CantAttack, Cost]);
    ruling!(
        "Reckless Brute",
        "Reckless Brute attacks only if it's able to do so as the declare attackers step begins. If, at that time, it's tapped, it's affected by a spell or ability that says it can't attack, then it doesn't attack. If there's a cost associated with having it attack, you're not forced to pay that cost, so it doesn't attack in that case either."
    );
    check("Reckless Brute", &[Tapped, CantAttack, Cost]);
    // The chosen-player requirements: with one opponent, the random choice is P1.
    ruling!(
        "Raving Dead",
        "If, as attackers are declared, Raving Dead is tapped, is affected by a spell or ability that says it can't attack, or hasn't been under your control continuously since your turn began (and doesn't have haste), then it doesn't attack. If there's a cost associated with having Raving Dead attack the chosen player, you aren't forced to pay that cost, so it doesn't have to attack that player in that case either."
    );
    check("Raving Dead", &[Tapped, CantAttack, Sick, Cost]);
    ruling!(
        "Ruhan of the Fomori",
        "If, during your declare attackers step, Ruhan is tapped, is affected by a spell or ability that says it can't attack, or hasn't been under your control continuously since your turn began (and doesn't have haste), then it doesn't attack. If there's a cost associated with having Ruhan attack the chosen player, you aren't forced to pay that cost, so it doesn't have to attack that player in that case either."
    );
    check("Ruhan of the Fomori", &[Tapped, CantAttack, Sick, Cost]);
}

#[test]
fn the_controller_chooses_what_a_forced_attacker_attacks() {
    cr!("508.1a", "508.1d");
    ruling!(
        "Impetuous Sunchaser",
        "You choose which player or planeswalker Impetuous Sunchaser attacks."
    );
    ruling!(
        "Reckless Brute",
        "You choose which player or planeswalker Reckless Brute attacks."
    );
    ruling!(
        "Valley Dasher",
        "You still choose which player or planeswalker Valley Dasher attacks."
    );
    for name in ["Impetuous Sunchaser", "Reckless Brute", "Valley Dasher"] {
        supported(name);
        for at_walker in [false, true] {
            let mut t = TestGame::new(2);
            let walker = t.battlefield(P1, "Garruk Wildspeaker");
            let c = t.battlefield(P0, name);
            let target = if at_walker { o(walker) } else { pl(P1) };
            assert_eq!(
                declare_from(&mut t, P0, &[(c, target)]),
                vec![(c, target)],
                "{name}: {at_walker}"
            );
        }
    }
}

#[test]
fn bloodrock_cyclops_has_no_penalty_if_it_cant_attack() {
    cr!("508.1d");
    ruling!(
        "Bloodrock Cyclops",
        "There is no penalty if it can’t attack."
    );
    assert!(attacks("Bloodrock Cyclops", No));
    let mut t = TestGame::new(2);
    let cyclops = t.battlefield(P0, "Bloodrock Cyclops");
    t.g.tap(cyclops);
    assert!(declare_from(&mut t, P0, &[]).is_empty());
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert!(t.on_battlefield(cyclops));
    assert_eq!(t.life(P0), 20);
}

#[test]
fn valley_dasher_attacks_the_turn_it_arrives_before_combat_only() {
    cr!("702.10b", "302.6", "508.1d");
    ruling!(
        "Valley Dasher",
        "If Valley Dasher enters the battlefield before the combat phase, it will attack that turn if able. If it enters the battlefield after combat, it won't attack that turn and will usually be available to block on the following turn."
    );
    // Before combat (haste): it attacks.
    let mut t = TestGame::new(2);
    let dasher = t.enter(P0, "Valley Dasher");
    t.resolve_all();
    assert_eq!(declare_from(&mut t, P0, &[]), vec![(dasher, pl(P1))]);
    // After combat: no more combat this turn; it's untapped to block on P1's turn.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PostcombatMain);
    let dasher = t.enter(P0, "Valley Dasher");
    t.resolve_all();
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.advance_to(P1, Step::PrecombatMain);
    assert!(!t.obj_now(dasher).tapped);
    declare_from(&mut t, P1, &[(bears, pl(P0))]);
    block(&mut t, P0, &[(dasher, bears)]);
    assert!(t.g.is_blocking(dasher), "Valley Dasher blocks on P1's turn");
}

#[test]
fn an_unless_condition_is_checked_as_attackers_are_declared() {
    cr!("508.1d");
    ruling!(
        "Marauding Maulhorn",
        "Marauding Maulhorn can attack if you control a creature named Advocate of the Beast. It just isn’t forced to."
    );
    ruling!(
        "Reckless Cohort",
        "Reckless Cohort can attack if you control another Ally. It just isn’t forced to."
    );
    ruling!(
        "Marauding Maulhorn",
        "Marauding Maulhorn checks whether you control a creature named Advocate of the Beast as attacking creatures are declared. At that time, if you don’t, Marauding Maulhorn attacks if able."
    );
    ruling!(
        "Reckless Cohort",
        "Reckless Cohort checks whether you control another Ally as attacking creatures are declared. At that time, if you don’t, Reckless Cohort attacks if able."
    );
    for (name, other) in [
        ("Marauding Maulhorn", "Advocate of the Beast"),
        ("Reckless Cohort", "Hada Freeblade"),
    ] {
        supported(name);
        // With the other creature: it isn't forced to attack, but it may.
        for attack in [false, true] {
            let mut t = TestGame::new(2);
            let c = t.battlefield(P0, name);
            t.battlefield(P0, other);
            let decl = if attack { vec![(c, pl(P1))] } else { vec![] };
            assert_eq!(declare_from(&mut t, P0, &decl), decl, "{name}: {attack}");
        }
        // The other creature is gone by the time attackers are declared (destroyed in the
        // beginning of combat step): it attacks.
        let mut t = TestGame::new(2);
        let c = t.battlefield(P0, name);
        let x = t.battlefield(P0, other);
        t.advance_to(P0, Step::BeginningOfCombat);
        crate::r_s02_common::destroy(&mut t, x);
        assert!(t.named_on_battlefield(other).is_empty());
        assert_eq!(declare_from(&mut t, P0, &[]), vec![(c, pl(P1))], "{name}");
        // And the other way: it arrives in the beginning of combat step.
        let mut t = TestGame::new(2);
        let c = t.battlefield(P0, name);
        t.advance_to(P0, Step::BeginningOfCombat);
        t.battlefield(P0, other);
        assert!(declare_from(&mut t, P0, &[]).is_empty(), "{name}");
        let _ = c;
    }
}

#[test]
fn chosen_player_requirements_mean_the_player_not_a_planeswalker() {
    cr!("508.1d", "508.1a");
    ruling!(
        "Raving Dead",
        "Raving Dead must attack the chosen player if able, not a planeswalker controlled by that player."
    );
    ruling!(
        "Ruhan of the Fomori",
        "Ruhan must attack the chosen player if able, not a planeswalker controlled by the chosen player."
    );
    for name in ["Raving Dead", "Ruhan of the Fomori"] {
        supported(name);
        let mut t = TestGame::new(2);
        let walker = t.battlefield(P1, "Garruk Wildspeaker");
        let c = t.battlefield(P0, name);
        assert_eq!(
            declare_from(&mut t, P0, &[(c, o(walker))]),
            vec![(c, pl(P1))],
            "{name}"
        );
    }
}

/// The opponent `name` (Raving Dead or Ruhan) chooses at random with `seed` in a
/// three-player game, found by letting it attack.
fn chosen_with_seed(name: &str, seed: u64) -> PlayerId {
    let mut t = seeded(3, seed);
    let c = t.battlefield(P0, name);
    let attacks = declare_from(&mut t, P0, &[]);
    assert_eq!(attacks.len(), 1, "{name}");
    assert_eq!(attacks[0].0, c);
    match attacks[0].1 {
        Entity::Player(p) => p,
        e => panic!("{name} attacked {e:?}"),
    }
}

#[test]
fn when_the_chosen_player_cant_be_attacked_it_may_attack_elsewhere() {
    cr!("508.1d", "508.1a", "508.1c");
    ruling!(
        "Raving Dead",
        "If Raving Dead can't attack the chosen player for one of the above reasons but it can still attack elsewhere, you may choose to have it attack another player, attack a planeswalker, or not attack at all."
    );
    ruling!(
        "Ruhan of the Fomori",
        "If Ruhan isn't forced to attack the chosen player for one of the above reasons but can still attack, you may choose to have Ruhan attack another player, attack a planeswalker, or not attack at all."
    );
    for name in ["Raving Dead", "Ruhan of the Fomori"] {
        for seed in 0..2u64 {
            let chosen = chosen_with_seed(name, seed);
            let other = if chosen == P1 { P2 } else { P1 };
            // The chosen player has Ghostly Prison (P0 has the mana but doesn't pay) and a
            // planeswalker: no attack, an attack on the other player, or on the
            // planeswalker are each legal.
            for choice in 0..3 {
                let mut t = seeded(3, seed);
                t.battlefield(chosen, "Ghostly Prison");
                let walker = t.battlefield(chosen, "Garruk Wildspeaker");
                let c = t.battlefield(P0, name);
                t.lands(P0, "Plains", 2);
                let decl = match choice {
                    0 => vec![],
                    1 => vec![(c, pl(other))],
                    _ => vec![(c, o(walker))],
                };
                assert_eq!(
                    declare_from(&mut t, P0, &decl),
                    decl,
                    "{name}, seed {seed}: {choice}"
                );
            }
        }
    }
}

#[test]
fn each_combat_phase_has_its_own_random_choice() {
    cr!("508.1d", "500.8", "603.2");
    ruling!(
        "Raving Dead",
        "If your turn has multiple combat phases, Raving Dead's ability triggers at the beginning of each of them. Ignore any choices made during previous combat phases that turn."
    );
    ruling!(
        "Ruhan of the Fomori",
        "If your turn has multiple combat phases, Ruhan's ability triggers at the beginning of each of them. Ignore any choices made during previous combat phases."
    );
    for name in ["Raving Dead", "Ruhan of the Fomori"] {
        // Over several seeds: in the second combat (from Relentless Assault), P0 declares
        // an attack on the player chosen for the first combat. When the new choice is the
        // other player, that declaration isn't legal and the creature attacks the other
        // player: the first choice doesn't count any more.
        let mut moved = false;
        for seed in 0..10u64 {
            let mut t = seeded(3, seed);
            t.g.players[1].life = 40;
            t.g.players[2].life = 40;
            let c = t.battlefield(P0, name);
            let first = declare_from(&mut t, P0, &[]);
            assert_eq!(first.len(), 1);
            t.advance_to(P0, Step::PostcombatMain);
            t.resolve_all();
            cast_resolve(&mut t, P0, "Relentless Assault", &[]);
            assert!(!t.obj_now(c).tapped, "untapped");
            let second = declare_from(&mut t, P0, &[(c, first[0].1)]);
            assert_eq!(second.len(), 1, "{name}, seed {seed}");
            if second[0].1 != first[0].1 {
                moved = true;
            }
        }
        assert!(moved, "{name}: a new choice is made for the second combat");
    }
}

#[test]
fn raving_dead_halves_life_after_combat_damage() {
    cr!("510.2", "603.2");
    ruling!(
        "Raving Dead",
        "The last ability triggers and resolves after combat damage has been dealt. For example, if an opponent has 17 life and Raving Dead deals 2 combat damage to them, they will end up at 8 life."
    );
    let mut t = TestGame::new(2);
    t.g.players[1].life = 17;
    t.battlefield(P0, "Raving Dead");
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    assert_eq!(t.life(P1), 8);
}

#[test]
fn amarant_corals_damage_to_the_other_opponents_isnt_combat_damage() {
    cr!("510.2", "120.2a", "603.2");
    ruling!(
        "Amarant Coral",
        "The damage dealt by Amarant Coral as a result of its last ability is not combat damage and therefore doesn't cause the ability to trigger again."
    );
    let mut t = TestGame::new(3);
    let coral = t.battlefield(P0, "Amarant Coral");
    declare_from(&mut t, P0, &[(coral, pl(P1))]);
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    assert_eq!((t.life(P1), t.life(P2)), (15, 15));
}

#[test]
fn relentless_raptor_blocks_for_a_defending_player_who_gains_control_of_it() {
    cr!("506.4", "509.1c", "508.1d");
    ruling!(
        "Relentless Raptor",
        "If the defending player somehow gains control of Relentless Raptor after it attacks, it must also block if able."
    );
    let mut t = TestGame::new(2);
    let raptor = t.battlefield(P0, "Relentless Raptor");
    let bears = t.battlefield(P0, "Grizzly Bears");
    declare_from(&mut t, P0, &[(raptor, pl(P1)), (bears, pl(P1))]);
    // P1 casts Ray of Command ("Untap target creature an opponent controls and gain
    // control of it until end of turn. ...") on the Raptor: it's removed from combat.
    cast_resolve(&mut t, P1, "Ray of Command", &[o(raptor)]);
    assert_eq!(t.obj_now(raptor).controller, P1);
    assert!(!t.g.is_attacking(raptor));
    // P1 declares no blockers: the Raptor blocks anyway.
    block(&mut t, P1, &[]);
    assert!(t.g.is_blocking(raptor), "{}", t.dump_log());
}

#[test]
fn haktos_protection_from_mana_values() {
    cr!("702.16a", "702.16b", "702.16c", "202.3e", "107.3g");
    ruling!(
        "Haktos the Unscarred",
        "If Haktos somehow has no choice made for its ability, its last ability grants it no protection abilities."
    );
    // Put onto the battlefield without its "as enters" replacement: nothing was chosen.
    let mut t = TestGame::new(2);
    let haktos = t.battlefield(P0, "Haktos the Unscarred");
    let others: Vec<ObjectId> = [
        "Memnite",
        "Savannah Lions",
        "Grizzly Bears",
        "Gray Ogre",
        "Hill Giant",
        "Craw Wurm",
    ]
    .iter()
    .map(|n| t.battlefield(P1, n))
    .collect();
    t.g.recompute();
    for x in &others {
        assert!(!t.g.protected_from(haktos, *x));
    }

    ruling!(
        "Haktos the Unscarred",
        "A number is chosen randomly for Haktos before it enters the battlefield. There's no point at which players can target it before it has gained the appropriate protection abilities."
    );
    // Cast and resolved: the moment it's on the battlefield, before anyone gets priority,
    // it has protection from mana value 1 (the number is 2, 3 or 4).
    let mut t = TestGame::new(2);
    let lions = t.battlefield(P1, "Savannah Lions");
    let h = cast(&mut t, P0, "Haktos the Unscarred", &[]);
    t.g.resolve_top();
    let haktos = t.g.current(h);
    assert!(t.on_battlefield(haktos));
    assert!(t.g.protected_from(haktos, lions));

    ruling!(
        "Haktos the Unscarred",
        "For spells with {X} in their mana costs, use the value chosen for X to determine the spell's mana value. If a card or permanent in any other zone has {X} in its mana cost, X is considered to be 0."
    );
    // Find a seed where the number is 2 (it isn't protected from Grizzly Bears).
    let mut found = false;
    for seed in 0..30u64 {
        let mut t = seeded(2, seed);
        let bears = t.battlefield(P1, "Grizzly Bears");
        let haktos = t.enter(P0, "Haktos the Unscarred");
        t.resolve_all();
        if t.g.protected_from(haktos, bears) {
            continue;
        }
        found = true;
        // P0's own Blaze ({X}{R}) with X = 1 has mana value 2 on the stack: it can target
        // Haktos; with X = 2 (mana value 3) it can't.
        for (x, legal) in [(1, true), (2, false)] {
            let mut t2 = seeded(2, seed);
            let haktos = t2.enter(P0, "Haktos the Unscarred");
            t2.resolve_all();
            t2.lands(P0, "Mountain", 3);
            let blaze = t2.hand(P0, "Blaze");
            t2.answer(P0, DecisionKind::X, mtg_engine::decision::Answer::Number(x));
            t2.answer_targets(P0, &[o(haktos)]);
            let r = t2.cast(P0, blaze).try_go();
            if legal {
                assert!(r.is_ok(), "{r:?} {}", t2.dump_log());
                t2.resolve_all();
                assert!(!t2.on_battlefield(haktos), "1 damage kills Haktos");
            } else {
                // The engine may have chosen another target or failed to cast.
                if r.is_ok() {
                    t2.resolve_all();
                }
                assert!(t2.on_battlefield(haktos), "X = {x}");
            }
        }
        // P1's Endless One ({X}) cast for X = 2 is a mana value 0 permanent: Haktos has
        // protection from it (it can't block Haktos).
        let mut t3 = seeded(2, seed);
        let haktos = t3.enter(P0, "Haktos the Unscarred");
        t3.resolve_all();
        let one = t3.battlefield(P1, "Endless One");
        put(&mut t3, one, "+1/+1", 2);
        assert_eq!(t3.g.mana_value_of(one), 0);
        assert!(t3.g.protected_from(haktos, one));
        break;
    }
    assert!(found, "no seed chose 2");
}

/// The draft info in which P0 noted `noted` for cards named Cogwork Tracker (one note
/// each).
fn tracker_notes(noted: &[Option<PlayerId>]) -> DraftInfo {
    let mut info = DraftInfo::default();
    for n in noted {
        info.notes.push(DraftNote {
            player: P0,
            card: "Cogwork Tracker".into(),
            number: None,
            noted_player: *n,
            noted_name: None,
            noted_types: vec![],
            colors: vec![],
            guessed_correctly: None,
        });
    }
    info
}

#[test]
fn cogwork_tracker_without_a_noted_player_in_the_game_still_must_attack() {
    cr!("905.2b", "508.1d");
    ruling!(
        "Cogwork Tracker",
        "If the noted player isn't in the game, Cogwork Tracker's last ability won't do anything, although it must still attack each turn if able. This is also true if there is no noted player because Cogwork Tracker was the first card drafted from a booster pack."
    );
    ruling!(
        "Cogwork Tracker",
        "If your draft is breaking up into multiple games, you and the noted player(s) aren't required to be in the same game."
    );
    let _ = card("Cogwork Tracker");
    // No noted player (first pick), or the noted player (P2 of the draft) is in another
    // game: P0 may choose what it attacks, but it attacks.
    for noted in [None, Some(P2)] {
        for at_walker in [None, Some(false), Some(true)] {
            let mut t = TestGame::new(2);
            t.g.start.draft = tracker_notes(&[noted]);
            let walker = t.battlefield(P1, "Garruk Wildspeaker");
            let tracker = t.battlefield(P0, "Cogwork Tracker");
            let decl = match at_walker {
                None => vec![],
                Some(false) => vec![(tracker, pl(P1))],
                Some(true) => vec![(tracker, o(walker))],
            };
            let attacks = declare_from(&mut t, P0, &decl);
            assert_eq!(attacks.len(), 1, "{noted:?}: it attacks");
            if !decl.is_empty() {
                assert_eq!(attacks, decl, "{noted:?}");
            }
        }
    }
}

#[test]
fn cogwork_trackers_with_several_noted_players() {
    cr!("905.2b", "508.1d", "508.1a");
    ruling!(
        "Cogwork Tracker",
        "If there are multiple noted players, perhaps because the players on both your left and right passed a Cogwork Tracker to you, you choose which of those players each Cogwork Tracker attacks. If you control more than one, they can each attack a different noted player."
    );
    // Four players; P0 noted P1 and P3 (left and right).
    let decls = |a: ObjectId, b: ObjectId| {
        vec![
            (vec![(a, pl(P1)), (b, pl(P3))], true),
            (vec![(a, pl(P3)), (b, pl(P1))], true),
            (vec![(a, pl(P1)), (b, pl(P1))], true),
            (vec![(a, pl(P2)), (b, pl(P3))], false),
        ]
    };
    for i in 0..4 {
        let mut t = TestGame::new(4);
        t.g.start.draft = tracker_notes(&[Some(P1), Some(P3)]);
        let a = t.battlefield(P0, "Cogwork Tracker");
        let b = t.battlefield(P0, "Cogwork Tracker");
        let (decl, legal) = decls(a, b)[i].clone();
        let attacks = declare_from(&mut t, P0, &decl);
        if legal {
            assert_eq!(attacks, decl, "{i}");
        } else {
            assert!(
                attacks.iter().all(|(_, d)| *d == pl(P1) || *d == pl(P3)),
                "{i}: {attacks:?}"
            );
            assert_eq!(attacks.len(), 2);
        }
    }
}
