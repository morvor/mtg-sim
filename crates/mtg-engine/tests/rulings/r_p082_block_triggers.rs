//! Rulings batch P082 — "blocks or becomes blocked" triggers: how many times they trigger
//! (once per blocking creature for "by a creature", once otherwise; CR 509.3c, 509.3d,
//! 509.3e), and that they resolve in the declare blockers step, before combat damage, without
//! making the attacker unblocked (CR 509.1h, 506.4).

use crate::r_p082_common::*;
use crate::r_s01_common::supported;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn carnage_gladiator_triggers_once_for_a_creature_blocking_several() {
    cr!("509.3a", "509.1a");
    ruling!(
        "Carnage Gladiator",
        "If a creature can block multiple creatures and does so, Carnage Gladiator’s ability triggers only once."
    );
    supported("Carnage Gladiator");
    supported("Palace Guard");
    let mut t = TestGame::new(2);
    let glad = t.battlefield(P0, "Carnage Gladiator");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    // Palace Guard: "This creature can block any number of creatures."
    let guard = t.battlefield(P1, "Palace Guard");
    blocked(&mut t, &[a, b], &[(guard, a), (guard, b)]);
    assert_eq!(triggers_from(&t, glad), 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
}

#[test]
fn per_blocker_triggers_trigger_once_for_each_blocking_creature() {
    cr!("509.3d", "603.2c");
    ruling!(
        "Grasping Giant",
        "If more than one creature blocks Grasping Giant, its ability triggers once for each of those creatures."
    );
    ruling!(
        "Somberwald Vigilante",
        "Somberwald Vigilante’s ability will trigger for each creature it becomes blocked by."
    );
    // Grasping Giant: each blocker is exiled.
    let mut t = TestGame::new(2);
    let (giant, bs) = block_with(&mut t, "Grasping Giant", &["Grizzly Bears", "Hill Giant"]);
    assert_eq!(triggers_from(&t, giant), 2);
    t.resolve_all();
    assert!(bs.iter().all(|b| t.zone(*b) == mtg_engine::object::Zone::Exile));
    // Somberwald Vigilante: 1 damage to each of two 1/1 blockers.
    let mut t = TestGame::new(2);
    let (v, bs) = block_with(
        &mut t,
        "Somberwald Vigilante",
        &["Llanowar Elves", "Llanowar Elves"],
    );
    assert_eq!(triggers_from(&t, v), 2);
    t.resolve_all();
    assert!(bs.iter().all(|b| !t.on_battlefield(*b)));
}

#[test]
fn engulfing_slagwurm_triggers_for_each_creature_and_each_trigger_has_its_creature() {
    cr!("509.3d", "509.3f", "603.3b");
    ruling!(
        "Engulfing Slagwurm",
        "If your Engulfing Slagwurm blocks or becomes blocked by multiple creatures, its ability triggers that many times. Each trigger will be associated with a specific creature. You choose which order to have the abilities resolve."
    );
    let mut t = TestGame::new(2);
    let from = t.asked().len();
    let (w, bs) = block_with(&mut t, "Engulfing Slagwurm", &["Grizzly Bears", "Hill Giant"]);
    assert_eq!(triggers_from(&t, w), 2);
    // P0 was asked to order the two triggers.
    assert!(t.asked()[from..].iter().any(|(p, d)| *p == P0
        && matches!(d, mtg_engine::decision::Decision::Order { items, .. } if items.len() == 2)));
    // The top trigger destroys one creature and P0 gains that creature's toughness.
    t.resolve();
    let gone: Vec<ObjectId> = bs.iter().copied().filter(|b| !t.on_battlefield(*b)).collect();
    assert_eq!(gone.len(), 1);
    let gained = t.life(P0) - 20;
    assert!(gained == 2 || gained == 3);
    t.resolve_all();
    assert!(bs.iter().all(|b| !t.on_battlefield(*b)));
    assert_eq!(t.life(P0), 25);
}

#[test]
fn becomes_blocked_triggers_trigger_only_once() {
    cr!("509.3c");
    ruling!(
        "Anzrag, the Quake-Mole",
        "If multiple creatures block Anzrag, the Quake-Mole, its first ability still triggers only once."
    );
    ruling!(
        "Rust Scarab",
        "Rust Scarab’s ability normally triggers only once per combat, no matter how many creatures are assigned to block it."
    );
    ruling!(
        "Bill Ferny, Bree Swindler",
        "The ability triggers only once, no matter how many creatures are blocking it."
    );
    ruling!(
        "Ichorclaw Myr",
        "Whenever Ichorclaw Myr becomes blocked, its last ability triggers just once, no matter how many creatures it became blocked by. It will get just +2/+2."
    );
    for name in [
        "Anzrag, the Quake-Mole",
        "Rust Scarab",
        "Bill Ferny, Bree Swindler",
        "Ichorclaw Myr",
    ] {
        let mut t = TestGame::new(2);
        // (A target for Rust Scarab's "destroy target artifact or enchantment".)
        t.battlefield(P1, "Mind Stone");
        let (a, _) = block_with(&mut t, name, &["Grizzly Bears", "Grizzly Bears"]);
        assert_eq!(triggers_from(&t, a), 1, "{name}");
        if name == "Ichorclaw Myr" {
            t.resolve_all();
            assert_eq!(t.pt(a), (3, 3));
        }
    }
}

#[test]
fn battle_scarred_goblin_triggers_once_and_stays_blocked() {
    cr!("509.3c", "509.1h", "506.4");
    ruling!(
        "Battle-Scarred Goblin",
        "The ability triggers only once, no matter how many creatures are blocking it. The ability resolves and deals damage to those creatures before combat damage is dealt. Even if that damage destroys all creatures blocking Battle-Scarred Goblin, Battle-Scarred Goblin doesn't become unblocked."
    );
    let mut t = TestGame::new(2);
    let (g, bs) = block_with(
        &mut t,
        "Battle-Scarred Goblin",
        &["Llanowar Elves", "Llanowar Elves"],
    );
    assert_eq!(triggers_from(&t, g), 1);
    t.resolve_all();
    assert_eq!(t.g.turn.step, Step::DeclareBlockers);
    assert!(bs.iter().all(|b| !t.on_battlefield(*b)));
    assert!(is_blocked(&t, g));
    finish_combat(&mut t);
    assert_eq!(t.life(P1), 20);
}

/// "Whenever this creature blocks or becomes blocked by a creature, this creature deals 1
/// damage to that creature": once per creature; the damage is dealt before combat damage
/// and a blocked attacker stays blocked.
fn pings_each_blocker_before_damage(name: &str) {
    let mut t = TestGame::new(2);
    let (a, bs) = block_with(&mut t, name, &["Llanowar Elves", "Llanowar Elves"]);
    assert_eq!(triggers_from(&t, a), 2, "{name}");
    t.resolve_all();
    assert_eq!(t.g.turn.step, Step::DeclareBlockers);
    assert!(bs.iter().all(|b| !t.on_battlefield(*b)), "{name}");
    assert!(is_blocked(&t, a), "{name}");
    finish_combat(&mut t);
    assert_eq!(t.life(P1), 20, "{name}");
    assert!(t.on_battlefield(a));
}

#[test]
fn kessig_forgemaster_pings_each_blocker() {
    cr!("509.3d", "509.1h", "506.4");
    ruling!(
        "Kessig Forgemaster // Flameheart Werewolf",
        "Kessig Forgemaster's first ability triggers once for each creature blocking or blocked by it. The ability resolves and deals damage to that creature before combat damage is dealt. If that damage destroys a creature blocking Kessig Forgemaster, Kessig Forgemaster doesn't become unblocked."
    );
    pings_each_blocker_before_damage("Kessig Forgemaster // Flameheart Werewolf");
}

#[test]
fn ornery_goblin_pings_each_blocker() {
    cr!("509.3d", "509.1h", "506.4");
    ruling!(
        "Ornery Goblin",
        "The triggered ability triggers once for each creature blocking or blocked by Ornery Goblin. The ability resolves and deals damage to that creature before combat damage is dealt. If that damage destroys all creatures blocking Ornery Goblin, Ornery Goblin doesn’t become unblocked."
    );
    pings_each_blocker_before_damage("Ornery Goblin");
}

#[test]
fn skewer_slinger_pings_each_blocker() {
    cr!("509.3d", "509.1h", "506.4");
    ruling!(
        "Skewer Slinger",
        "The triggered ability triggers once for each creature blocking or blocked by Skewer Slinger. The ability resolves and deals damage to that creature before combat damage is dealt. If that damage destroys all creatures blocking Skewer Slinger, it doesn't become unblocked."
    );
    pings_each_blocker_before_damage("Skewer Slinger");
}

#[test]
fn per_blocker_damage_triggers_resolve_before_combat_damage() {
    cr!("509.3d", "510.1", "509.1h");
    ruling!(
        "Somberwald Vigilante",
        "The ability resolves before combat damage is dealt, so it could cause blocking creatures to be destroyed and thus not deal combat damage."
    );
    ruling!(
        "Kolaghan Aspirant",
        "The damage Kolaghan Aspirant deals because of its triggered ability isn’t combat damage. If that damage causes each creature blocking it to leave combat, Kolaghan Aspirant will remain blocked. It won’t be dealt combat damage in this case."
    );
    // Somberwald Vigilante (1/1) blocked by Llanowar Elves (1/1): the Elves die first and
    // deal no combat damage.
    let mut t = TestGame::new(2);
    let (v, _) = block_with(&mut t, "Somberwald Vigilante", &["Llanowar Elves"]);
    finish_combat(&mut t);
    assert!(t.on_battlefield(v));
    assert_eq!(t.obj_now(v).damage, 0);
    // Kolaghan Aspirant (2/1): the 1 damage isn't combat damage (no lifelink-like combat
    // effects; the Elves just die), and the Aspirant stays blocked and isn't dealt damage.
    let mut t = TestGame::new(2);
    let (a, bs) = block_with(&mut t, "Kolaghan Aspirant", &["Llanowar Elves"]);
    t.resolve_all();
    assert!(!t.on_battlefield(bs[0]));
    let combat_damage = t.g.turn_events.iter().any(|e| {
        matches!(e, mtg_engine::events::Event::Damage { combat: true, .. })
    });
    assert!(!combat_damage, "the trigger's damage isn't combat damage");
    assert!(is_blocked(&t, a));
    finish_combat(&mut t);
    assert!(t.on_battlefield(a));
    assert_eq!(t.obj_now(a).damage, 0);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn plague_wight_stays_blocked_when_its_blockers_die() {
    cr!("509.1h", "506.4", "510.1c");
    ruling!(
        "Plague Wight",
        "If all creatures blocking Plague Wight die, Plague Wight remains blocked. It won’t assign any combat damage."
    );
    let mut t = TestGame::new(2);
    let (w, bs) = block_with(&mut t, "Plague Wight", &["Llanowar Elves"]);
    t.resolve_all();
    assert!(!t.on_battlefield(bs[0]));
    assert!(is_blocked(&t, w));
    finish_combat(&mut t);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn somberwald_alpha_gives_one_bonus_however_many_blockers() {
    cr!("509.3c");
    ruling!(
        "Somberwald Alpha",
        "Somberwald Alpha’s first ability will give each creature you control that becomes blocked +1/+1 until end of turn. It doesn’t matter how many of your opponent’s creatures are blocking each of your creatures."
    );
    supported("Somberwald Alpha");
    let mut t = TestGame::new(2);
    let alpha = t.battlefield(P0, "Somberwald Alpha");
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let b1 = t.battlefield(P1, "Llanowar Elves");
    let b2 = t.battlefield(P1, "Llanowar Elves");
    let b3 = t.battlefield(P1, "Llanowar Elves");
    blocked(&mut t, &[giant, bears], &[(b1, giant), (b2, giant), (b3, bears)]);
    assert_eq!(triggers_from(&t, alpha), 2);
    t.resolve_all();
    assert_eq!(t.pt(giant), (4, 4));
    assert_eq!(t.pt(bears), (3, 3));
}

#[test]
fn multiple_afflict_instances_trigger_separately() {
    cr!("702.130a", "702.130b");
    ruling!(
        "Dagger of the Worthy",
        "If a creature has multiple instances of afflict, each triggers separately."
    );
    ruling!(
        "Lazotep Sliver",
        "If a creature has multiple instances of afflict, each triggers separately."
    );
    supported("Dagger of the Worthy");
    supported("Lazotep Sliver");
    let mut t = TestGame::new(2);
    // Lazotep Sliver gives itself afflict 2; the Dagger gives it afflict 1 as well.
    let sliver = t.battlefield(P0, "Lazotep Sliver");
    let dagger = t.battlefield(P0, "Dagger of the Worthy");
    t.g.attach(dagger, Entity::Object(sliver));
    t.settle();
    let wall = t.battlefield(P1, "Grizzly Bears");
    blocked(&mut t, &[sliver], &[(wall, sliver)]);
    assert_eq!(triggers_from(&t, sliver), 2);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn sporeweb_weaver_triggers_once_per_damage_event_even_if_it_dies() {
    cr!("603.2c", "603.10a", "510.2");
    ruling!(
        "Sporeweb Weaver",
        "Sporeweb Weaver's last ability triggers just once, no matter how many things deal how much damage to it at once (most likely because two or more creatures blocked it)."
    );
    ruling!(
        "Sporeweb Weaver",
        "Sporeweb Weaver's last ability triggers even if the damage causes it to die."
    );
    // Weaver (1/4) attacks and is blocked by two Hill Giants: 6 damage at once, it dies.
    let mut t = TestGame::new(2);
    let (w, _) = block_with(&mut t, "Sporeweb Weaver", &["Hill Giant", "Hill Giant"]);
    t.advance_to(P0, Step::CombatDamage);
    t.settle();
    assert!(!t.on_battlefield(w));
    assert_eq!(triggers_from(&t, w), 1);
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
    assert_eq!(crate::r_s25_common::creature_tokens(&t, P0), 1);
}

#[test]
fn sporeweb_weavers_controller_loses_before_the_trigger_is_put_on_the_stack() {
    cr!("704.3", "704.5a", "117.5");
    ruling!(
        "Sporeweb Weaver",
        "If your life total is brought to 0 or less at the same time that Sporeweb Weaver is dealt damage, you lose the game before the ability goes on the stack."
    );
    supported("Sporeweb Weaver");
    let mut t = TestGame::new(2);
    let weaver = t.battlefield(P1, "Sporeweb Weaver");
    t.g.players[1].life = 2;
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    // The Weaver blocks the Giant; the unblocked Bears deal P1's last 2 life.
    blocked(&mut t, &[giant, bears], &[(weaver, giant)]);
    t.advance_to_step(Step::CombatDamage);
    t.settle();
    assert!(t.has_lost(P1));
    assert_eq!(triggers_from(&t, weaver), 0);
    assert_eq!(t.life(P1), 0, "the Weaver's lifegain never happened");
}

#[test]
fn saprazzan_raider_and_cunning_evasion_return_before_damage() {
    cr!("509.3c", "510.1", "506.4");
    ruling!(
        "Saprazzan Raider",
        "It returns as soon as it is blocked. It does not remain on the battlefield long enough to deal damage."
    );
    ruling!(
        "Cunning Evasion",
        "You return the attacking creature to its owner’s hand before the combat damage step. A creature returned this way neither deals nor is dealt combat damage."
    );
    let mut t = TestGame::new(2);
    let (r, bs) = block_with(&mut t, "Saprazzan Raider", &["Grizzly Bears"]);
    t.resolve_all();
    assert_eq!(t.g.turn.step, Step::DeclareBlockers);
    assert!(t.in_hand(P0, "Saprazzan Raider"));
    finish_combat(&mut t);
    assert_eq!(t.obj_now(bs[0]).damage, 0);
    let _ = r;
    // Cunning Evasion: P0 chooses to return the blocked Hill Giant.
    supported("Cunning Evasion");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Cunning Evasion");
    t.answer_yes(P0, true);
    let (g, bs) = block_with(&mut t, "Hill Giant", &["Grizzly Bears"]);
    t.resolve_all();
    assert_eq!(t.g.turn.step, Step::DeclareBlockers);
    assert!(t.in_hand(P0, "Hill Giant"));
    finish_combat(&mut t);
    assert!(t.on_battlefield(bs[0]));
    assert_eq!(t.obj_now(bs[0]).damage, 0);
    let _ = g;
}

#[test]
fn wooden_stake_destroys_the_vampire_before_combat_damage() {
    cr!("509.3f", "510.1");
    ruling!(
        "Wooden Stake",
        "The Vampire is destroyed before any combat damage is dealt."
    );
    supported("Wooden Stake");
    supported("Sengir Vampire");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let stake = t.battlefield(P0, "Wooden Stake");
    t.g.attach(stake, Entity::Object(bears));
    t.settle();
    let vamp = t.battlefield(P1, "Sengir Vampire");
    blocked(&mut t, &[bears], &[(vamp, bears)]);
    t.resolve_all();
    assert!(!t.on_battlefield(vamp));
    finish_combat(&mut t);
    assert!(t.on_battlefield(bears), "the Vampire dealt no combat damage");
}

#[test]
fn rust_scarab_destroys_after_blocks_before_damage() {
    cr!("509.3c", "510.1");
    ruling!(
        "Rust Scarab",
        "The artifact or enchantment will be destroyed before combat damage is dealt but after blockers are chosen."
    );
    let mut t = TestGame::new(2);
    let relic = t.battlefield(P1, "Mind Stone");
    t.answer_targets(P0, &[Entity::Object(relic)]);
    t.answer_yes(P0, true);
    let (s, _) = block_with(&mut t, "Rust Scarab", &["Grizzly Bears"]);
    // Blockers were chosen with the artifact still there; it's destroyed in the declare
    // blockers step.
    assert!(t.on_battlefield(relic));
    t.resolve_all();
    assert_eq!(t.g.turn.step, Step::DeclareBlockers);
    assert!(!t.on_battlefield(relic));
    let _ = s;
}
