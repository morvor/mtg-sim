//! Rulings batch P133 — triggered abilities of bounce cards: what triggers them (Fallowsage,
//! Storm Cauldron, Mana Breach, Dissipation Field, Equilibrium, Jace's emblem, Scalding
//! Viper, Ugin's Binding, Living Breakthrough) and how they resolve (CR 603).

use crate::r_p130_common::loyalty;
use crate::r_p133_common::*;
use crate::r_s01_common::supported;
use crate::r_s02_common::can_cast;
use crate::r_s11_common::triggered_from;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn becomes_tapped_triggers_need_an_untapped_creature() {
    cr!("603.2", "701.26a");
    ruling!(
        "Fallowsage",
        "For the ability to trigger, the creature has to actually change from untapped to tapped. If an effect attempts to tap the creature, but it was already tapped at the time, this ability won't trigger."
    );
    ruling!(
        "Surgespanner",
        "For the ability to trigger, the creature has to actually change from untapped to tapped."
    );
    for name in ["Fallowsage", "Surgespanner"] {
        supported(name);
        for already_tapped in [true, false] {
            let mut t = TestGame::new(2);
            let it = t.battlefield(P0, name);
            t.g.objects[it.0 as usize].tapped = already_tapped;
            let pp = spell(&mut t, P1, "Pressure Point");
            t.cast(P1, pp).target(it).go();
            t.resolve();
            assert!(t.obj(it).tapped);
            assert_eq!(
                triggered_from(&t, it),
                usize::from(!already_tapped),
                "{name}, already tapped: {already_tapped}"
            );
        }
    }
}

#[test]
fn storm_cauldron_lands_tapped_for_mana() {
    cr!("603.2", "605.1a", "305.2");
    ruling!(
        "Storm Cauldron",
        "If a land is tapped for mana, it is returned to its owner’s hand as a triggered ability."
    );
    ruling!(
        "Storm Cauldron",
        "If a land is tapped for mana and sacrificed all in one action, it goes to the graveyard before the Cauldron can return it to the player’s hand."
    );
    ruling!(
        "Storm Cauldron",
        "Having multiples of these on the battlefield means you can play an additional land for each one."
    );
    supported("Storm Cauldron");
    // A Forest tapped for mana: the trigger goes on the stack, then returns it.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Storm Cauldron");
    let forest = t.battlefield(P0, "Forest");
    t.activate(P0, forest, 0, &[]).unwrap();
    assert_eq!(t.g.player(P0).mana_pool.count(ManaType::G), 1);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    assert!(t.on_battlefield(forest));
    t.resolve_all();
    assert!(in_hand_of(&t, forest, P0));
    // Crystal Vein tapped and sacrificed for mana: it stays in the graveyard.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Storm Cauldron");
    let vein = t.battlefield(P0, "Crystal Vein");
    t.activate(P0, vein, 1, &[]).unwrap();
    assert_eq!(t.g.player(P0).mana_pool.count(ManaType::C), 2);
    t.resolve_all();
    assert_eq!(t.zone(vein), Zone::Graveyard(P0));
    // Two Cauldrons: three land plays this turn.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Storm Cauldron");
    t.battlefield(P0, "Storm Cauldron");
    let lands: Vec<_> = (0..4).map(|_| t.hand(P0, "Forest")).collect();
    for l in &lands[..3] {
        t.play_land(P0, *l).unwrap();
    }
    assert!(t.play_land(P0, lands[3]).is_err());
}

#[test]
fn mana_breach_trigger_controller_and_choice() {
    cr!("603.3a", "113.8");
    ruling!(
        "Mana Breach",
        "The ability itself is controlled by Mana Breach’s controller."
    );
    ruling!(
        "Mana Breach",
        "The person who cast the spell chooses which of their lands is affected."
    );
    supported("Mana Breach");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mana Breach");
    let mountain = t.battlefield(P1, "Mountain");
    let forest = t.battlefield(P1, "Forest");
    let shock = t.hand(P1, "Shock");
    t.cast(P1, shock).target(Entity::Player(P0)).go();
    t.settle();
    let trig = *t.g.stack.last().unwrap();
    assert_eq!(t.obj(trig).controller, P0);
    t.answer_choose(P1, &[Entity::Object(forest)]);
    let from = t.asked().len();
    t.resolve();
    assert!(in_hand_of(&t, forest, P1));
    assert!(t.on_battlefield(mountain));
    assert!(t.asked()[from..]
        .iter()
        .any(|(p, d)| *p == P1 && matches!(d, decision::Decision::ChooseEntities { .. })));
}

#[test]
fn dissipation_field_any_damage_from_any_permanent() {
    cr!("603.2", "120.3");
    ruling!(
        "Dissipation Field",
        "The ability triggers whenever you're dealt any damage (not just combat damage) by any permanent (not just creatures). This includes permanents you control."
    );
    supported("Dissipation Field");
    // P0's own land deals noncombat damage to P0.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Dissipation Field");
    let springs = t.battlefield(P0, "Sulfurous Springs");
    t.activate(P0, springs, 1, &[]).unwrap();
    assert_eq!(t.life(P0), 19);
    t.resolve_all();
    assert!(in_hand_of(&t, springs, P0));
    // An opponent's creature's activated ability.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Dissipation Field");
    let pyro = t.battlefield(P1, "Prodigal Sorcerer");
    t.activate(P1, pyro, 0, &[Entity::Player(P0)]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P0), 19);
    assert!(in_hand_of(&t, pyro, P1));
}

#[test]
fn equilibrium_targets_and_countered_spell() {
    cr!("603.3d", "601.2i");
    ruling!(
        "Equilibrium",
        "The ability is put on the stack immediately after you cast a creature spell, which means the creature that spell will become won't be a legal target."
    );
    ruling!(
        "Equilibrium",
        "The ability will still resolve even if the spell that triggered it gets countered."
    );
    supported("Equilibrium");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Equilibrium");
    let memnite = t.battlefield(P1, "Memnite");
    let bears = spell(&mut t, P0, "Grizzly Bears");
    t.lands(P0, "Wastes", 1);
    t.answer_targets(P0, &[Entity::Object(memnite)]);
    t.answer_yes(P0, true);
    let from = t.asked().len();
    let s = t.cast(P0, bears).go();
    t.settle();
    let cands = crate::r_s02_common::target_candidates(&t, P0, from);
    assert!(!cands.is_empty());
    for c in &cands {
        assert!(!c.contains(&Entity::Object(s)));
        assert!(!c.contains(&Entity::Object(bears)));
    }
    // The creature spell is countered in response.
    let s_now = t.g.current(s);
    t.g.counter(s_now, None);
    t.resolve_all();
    assert!(in_hand_of(&t, memnite, P1));
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
}

/// P0 gets Jace, Unraveler of Secrets' emblem.
fn jace_emblem(t: &mut TestGame) {
    supported("Jace, Unraveler of Secrets");
    let jace = t.battlefield(P0, "Jace, Unraveler of Secrets");
    loyalty(t, jace, 8);
    t.activate(P0, jace, 2, &[]).unwrap();
    t.resolve_all();
}

#[test]
fn jace_emblem_first_spell_each_turn() {
    cr!("114.4", "603.2");
    ruling!(
        "Jace, Unraveler of Secrets",
        "If Jace’s emblem’s triggered ability doesn’t counter the first spell an opponent casts (perhaps because that spell can’t be countered), it won’t trigger again in the same turn to try to counter that player’s second spell."
    );
    ruling!(
        "Jace, Unraveler of Secrets",
        "The emblem’s triggered ability counters the first spell an opponent casts on each turn, not just that opponent’s turn."
    );
    let mut t = TestGame::new(2);
    jace_emblem(&mut t);
    let bears = t.battlefield(P0, "Grizzly Bears");
    // On P0's turn, P1's first spell can't be countered: it resolves.
    let eject = spell(&mut t, P1, "Eject");
    t.cast(P1, eject).target(bears).go();
    t.resolve_all();
    assert!(in_hand_of(&t, bears, P0));
    // The second spell isn't countered either.
    let shock = spell(&mut t, P1, "Shock");
    t.cast(P1, shock).target(Entity::Player(P0)).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 18);
    // A fresh game: P1's first spell on P0's turn is countered.
    let mut t = TestGame::new(2);
    jace_emblem(&mut t);
    let shock = spell(&mut t, P1, "Shock");
    t.cast(P1, shock).target(Entity::Player(P0)).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    assert!(t.in_graveyard(P1, "Shock"));
}

#[test]
fn jace_emblem_once_per_opponent() {
    cr!("114.4", "603.2");
    ruling!(
        "Jace, Unraveler of Secrets",
        "If you have multiple opponents, Jace’s emblem can trigger once each turn for each opponent."
    );
    let mut t = TestGame::new(3);
    jace_emblem(&mut t);
    for p in [P1, P2] {
        let shock = spell(&mut t, p, "Shock");
        t.cast(p, shock).target(Entity::Player(P0)).go();
        t.resolve_all();
    }
    assert_eq!(t.life(P0), 20);
    // P1's second spell isn't countered.
    let shock = spell(&mut t, P1, "Shock");
    t.cast(P1, shock).target(Entity::Player(P0)).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 18);
}

#[test]
fn scalding_viper_x_spells_use_the_chosen_x() {
    cr!("202.3e", "107.3a");
    ruling!(
        "Scalding Viper // Steam Clean",
        "If a spell has {X} in its mana cost, use the value chosen for X to determine the mana value of that spell."
    );
    supported("Scalding Viper // Steam Clean");
    for (x, dmg) in [(1, 1), (2, 1), (3, 0)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Scalding Viper // Steam Clean");
        t.set_step(P1, Step::PrecombatMain);
        let blaze = t.hand(P1, "Blaze");
        t.lands(P1, "Mountain", 1);
        t.lands(P1, "Wastes", x as usize);
        t.cast(P1, blaze).x(x).target(Entity::Player(P0)).go();
        t.resolve_all();
        assert_eq!(t.life(P1), 20 - dmg, "X={x}");
        assert_eq!(t.life(P0), 20 - x as i32, "X={x}");
    }
}

#[test]
fn ugins_binding_graveyard_trigger() {
    cr!("603.2", "603.12", "113.6m");
    ruling!(
        "Ugin's Binding",
        "The last ability triggers only if Ugin's Binding is in your graveyard as you cast the colorless spell. That ability will resolve before the colorless spell does. If the colorless spell is countered or otherwise leaves the stack in response to Ugin's Binding's triggered ability, the ability will still resolve as normal."
    );
    supported("Ugin's Binding");
    // In the graveyard: the trigger resolves even though the spell is countered.
    let mut t = TestGame::new(2);
    let binding = t.graveyard(P0, "Ugin's Binding");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let colossus = spell(&mut t, P0, "Darksteel Colossus");
    t.answer_yes(P0, true);
    let s = t.cast(P0, colossus).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    assert_ne!(*t.g.stack.last().unwrap(), t.g.current(s));
    let s_now = t.g.current(s);
    t.g.counter(s_now, None);
    t.resolve_all();
    assert!(in_hand_of(&t, bears, P1));
    assert_eq!(t.zone(binding), Zone::Exile);
    // In hand: no trigger.
    let mut t = TestGame::new(2);
    t.hand(P0, "Ugin's Binding");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let colossus = spell(&mut t, P0, "Darksteel Colossus");
    t.cast(P0, colossus).go();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
}

#[test]
fn living_breakthrough_restriction_timing() {
    cr!("603.3", "601.2", "608.2");
    ruling!(
        "Inventive Iteration // Living Breakthrough",
        "Living Breakthrough's triggered ability will resolve before the spell that caused it to trigger. Notably, the ability has no effect until it resolves, so your opponents can respond to it by casting instant spells with the same mana value as the original spell."
    );
    ruling!(
        "Inventive Iteration // Living Breakthrough",
        "Living Breakthrough's triggered ability won't affect spells that a player has already cast, even if those spells haven't resolved yet."
    );
    supported("Inventive Iteration // Living Breakthrough");
    let mut t = TestGame::new(2);
    crate::r_s17_common::enter_transformed(&mut t, P0, "Inventive Iteration // Living Breakthrough");
    // P1 has already cast a Shock (mana value 1) when P0 casts one.
    let p1_first = spell(&mut t, P1, "Shock");
    t.cast(P1, p1_first).target(Entity::Player(P0)).go();
    let mine = spell(&mut t, P0, "Shock");
    t.cast(P0, mine).target(Entity::Player(P1)).go();
    t.settle();
    // The trigger is on top; P1 can still respond with another mana value 1 instant.
    assert_eq!(t.stack_len(), 3);
    let p1_second = spell(&mut t, P1, "Shock");
    assert!(can_cast(&mut t, P1, p1_second, CastMethod::Normal));
    t.resolve();
    // After the trigger resolved: no more mana value 1 spells, but others are fine.
    assert!(!can_cast(&mut t, P1, p1_second, CastMethod::Normal));
    let other = spell(&mut t, P1, "Lightning Strike");
    assert!(can_cast(&mut t, P1, other, CastMethod::Normal));
    t.resolve_all();
    // P1's first Shock (already cast) still resolved.
    assert_eq!(t.life(P0), 18);
    assert_eq!(t.life(P1), 18);
}

#[test]
fn mu_yanling_ultimate_affects_flyers_at_resolution() {
    cr!("611.2c");
    ruling!(
        "Mu Yanling, Celestial Wind",
        "Mu Yanling’s last ability affects only creatures you control with flying at the time it resolves. Creatures you begin to control later in the turn (or creatures that that gain flying later in the turn) won’t get +5/+5."
    );
    supported("Mu Yanling, Celestial Wind");
    let mut t = TestGame::new(2);
    let mu = t.battlefield(P0, "Mu Yanling, Celestial Wind");
    loyalty(&mut t, mu, 7);
    let crow = t.battlefield(P0, "Storm Crow");
    t.activate(P0, mu, 2, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(crow), (6, 7));
    let later = t.battlefield(P0, "Storm Crow");
    t.g.recompute();
    assert_eq!(t.pt(later), (1, 2));
}

#[test]
fn only_creatures_you_control_at_resolution_are_affected() {
    cr!("611.2c");
    ruling!(
        "Simic Charm",
        "If you choose the second mode, only permanents you control when Simic Charm resolves will gain hexproof. Permanents that come under your control later that turn will not."
    );
    ruling!(
        "Arm with Aether",
        "Only creatures you control on the battlefield when Arm with Aether resolves gain the ability. Creatures you gain control of later in the turn won’t be affected."
    );
    ruling!(
        "Sunscape Master",
        "The first ability only affects creatures you control at the time the ability resolves."
    );
    use mtg_engine::keywords::KeywordKind;
    // Simic Charm: a creature P0 gains control of afterwards has no hexproof.
    supported("Simic Charm");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let charm = spell(&mut t, P0, "Simic Charm");
    t.cast(P0, charm).modes(&[1]).go();
    t.resolve_all();
    crate::r_s06_common::give_control(&mut t, theirs, P0);
    assert!(t.obj_now(mine).chars.has_keyword(KeywordKind::Hexproof));
    assert!(!t.obj_now(theirs).chars.has_keyword(KeywordKind::Hexproof));
    // Arm with Aether: same.
    supported("Arm with Aether");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let arm = spell(&mut t, P0, "Arm with Aether");
    t.cast(P0, arm).go();
    t.resolve_all();
    crate::r_s06_common::give_control(&mut t, theirs, P0);
    let n_mine = t.obj_now(mine).chars.abilities.len();
    let n_theirs = t.obj_now(theirs).chars.abilities.len();
    assert_eq!(n_mine, n_theirs + 1);
    // Sunscape Master: a creature that enters afterwards isn't pumped.
    supported("Sunscape Master");
    let mut t = TestGame::new(2);
    let master = t.battlefield(P0, "Sunscape Master");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 2);
    t.activate(P0, master, 0, &[]).unwrap();
    t.resolve_all();
    let later = t.battlefield(P0, "Grizzly Bears");
    t.g.recompute();
    assert_eq!(t.pt(bears), (4, 4));
    assert_eq!(t.pt(later), (2, 2));
}

#[test]
fn arm_with_aether_granted_trigger() {
    cr!("603.2", "120.1", "603.3a");
    ruling!(
        "Arm with Aether",
        "The granted ability triggers on all damage dealt by the creature to one of your opponents, not just combat damage."
    );
    ruling!(
        "Arm with Aether",
        "In a multiplayer game, if an affected creature deals damage to two of your opponents simultaneously, the ability will trigger twice."
    );
    ruling!(
        "Arm with Aether",
        "The affected creatures keep the ability even if another player gains control of them that turn. If such a creature deals damage to you, the ability will trigger, targeting a creature you control."
    );
    supported("Arm with Aether");
    // Noncombat damage to two opponents at once: two triggers.
    let mut t = TestGame::new(3);
    let alch = t.battlefield(P0, "Thermo-Alchemist");
    let b1 = t.battlefield(P1, "Grizzly Bears");
    let b2 = t.battlefield(P2, "Grizzly Bears");
    let arm = spell(&mut t, P0, "Arm with Aether");
    t.cast(P0, arm).go();
    t.resolve_all();
    t.answer_targets(P0, &[Entity::Object(b1)]);
    t.answer_targets(P0, &[Entity::Object(b2)]);
    // (Thermo-Alchemist's own untap ability triggered on casting Arm with Aether.)
    let before = triggered_from(&t, alch);
    t.activate(P0, alch, 0, &[]).unwrap();
    t.resolve();
    assert_eq!((t.life(P1), t.life(P2)), (19, 19));
    assert_eq!(triggered_from(&t, alch) - before, 2);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(in_hand_of(&t, b1, P1) && in_hand_of(&t, b2, P2));
    // P1 gains control of the creature; it damages P0, P1's opponent: P1's trigger
    // targets a creature P0 controls.
    let mut t = TestGame::new(2);
    let pyro = t.battlefield(P0, "Prodigal Sorcerer");
    let mine = t.battlefield(P0, "Grizzly Bears");
    let arm = spell(&mut t, P0, "Arm with Aether");
    t.cast(P0, arm).go();
    t.resolve_all();
    crate::r_s06_common::give_control(&mut t, pyro, P1);
    // (As if P1 had controlled it continuously since their most recent turn began.)
    let pyro = t.g.current(pyro);
    t.g.objects[pyro.0 as usize].summoning_sick = false;
    t.activate(P1, pyro, 0, &[Entity::Player(P0)]).unwrap();
    t.answer_targets(P1, &[Entity::Object(mine)]);
    t.answer_yes(P1, true);
    t.resolve_all();
    assert_eq!(t.life(P0), 19);
    assert_eq!(triggered_from(&t, pyro), 1);
    assert!(in_hand_of(&t, mine, P0));
}
