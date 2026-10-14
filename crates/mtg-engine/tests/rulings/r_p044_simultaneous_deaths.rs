//! Rulings batch P044 — leaves-the-battlefield abilities "look back in time" (CR 603.10a):
//! a death trigger sees creatures that die at the same time as its source, and a source
//! that is itself put into a graveyard.

use crate::r_p044_common::*;
use crate::r_s01_common::*;
use mtg_engine::events::MoveCause;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn anafenza_triggers_for_each_creature_dying_with_it() {
    cr!("603.10a", "603.6c", "701.63a");
    ruling!(
        "Anafenza, Unyielding Lineage",
        "If Anafenza dies at the same time as one or more other nontoken creatures you control, Anafenza’s last ability triggers for each of those other nontoken creatures."
    );
    supported("Anafenza, Unyielding Lineage");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Anafenza, Unyielding Lineage");
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Hill Giant");
    wrath(&mut t, P0);
    // Anafenza is gone, so each endure 2 creates a 2/2 Spirit.
    assert_eq!(with_subtype(&t, P0, "Spirit").len(), 2);
}

#[test]
fn death_triggers_see_creatures_dying_with_their_source() {
    cr!("603.10a", "603.6c", "603.2");
    ruling!(
        "High-Society Hunter",
        "If High-Society Hunter dies at the same time as one or more other nontoken creatures, its last ability will trigger for each of those creatures."
    );
    ruling!(
        "Harvester of Souls",
        "If another nontoken creature dies at the same time as Harvester of Souls, its ability triggers for that creature."
    );
    ruling!(
        "Poison-Tip Archer",
        "If another creature dies at the same time as Poison-Tip Archer does, each opponent loses 1 life."
    );
    ruling!(
        "Gimli, Counter of Kills",
        "If Gimli, Counter of Kills dies at the same time as one or more creatures an opponent controls, the last ability will trigger for each of those other creatures."
    );
    ruling!(
        "Sangromancer",
        "If one or more creatures an opponent controls are put into the graveyard at the same time as Sangromancer, Sangromancer's first ability will trigger that many times."
    );
    // High-Society Hunter: draws for each other nontoken creature (either player's).
    supported("High-Society Hunter");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "High-Society Hunter");
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Hill Giant");
    let hand = t.hand_size(P0);
    wrath(&mut t, P0);
    assert_eq!(t.hand_size(P0), hand + 2);
    // Harvester of Souls ("you may draw").
    supported("Harvester of Souls");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Harvester of Souls");
    t.battlefield(P1, "Grizzly Bears");
    t.answer_yes(P0, true);
    let hand = t.hand_size(P0);
    wrath(&mut t, P0);
    assert_eq!(t.hand_size(P0), hand + 1);
    // Poison-Tip Archer: one other creature, 1 life.
    supported("Poison-Tip Archer");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Poison-Tip Archer");
    t.battlefield(P0, "Grizzly Bears");
    wrath(&mut t, P0);
    assert_eq!(t.life(P1), 19);
    // Gimli: two of P1's creatures, 1 damage each to P1.
    supported("Gimli, Counter of Kills");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Gimli, Counter of Kills");
    t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Hill Giant");
    wrath(&mut t, P0);
    assert_eq!(t.life(P1), 18);
    // Sangromancer: two of P1's creatures, 3 life each.
    supported("Sangromancer");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sangromancer");
    t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Hill Giant");
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    wrath(&mut t, P0);
    assert_eq!(t.life(P0), 26);
}

#[test]
fn enchantments_leaving_with_creatures_still_trigger() {
    cr!("603.10a", "603.6c");
    ruling!(
        "Moldervine Reclamation",
        "If Moldervine Reclamation leaves the battlefield at the same time that one or more creatures you control die, its ability triggers for each of those creatures."
    );
    ruling!(
        "Open the Graves",
        "If another creature you control dies at the same time that Open the Graves leaves the battlefield, you'll get a Zombie token."
    );
    supported("Moldervine Reclamation");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Moldervine Reclamation");
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Hill Giant");
    let hand = t.hand_size(P0);
    cleansing(&mut t, P0);
    assert_eq!(t.life(P0), 22);
    assert_eq!(t.hand_size(P0), hand + 2);
    supported("Open the Graves");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Open the Graves");
    t.battlefield(P0, "Grizzly Bears");
    cleansing(&mut t, P0);
    assert_eq!(with_subtype(&t, P0, "Zombie").len(), 1);
}

#[test]
fn artifact_triggers_see_artifacts_and_themselves() {
    cr!("603.10a", "603.6c", "205.1b");
    ruling!(
        "Disciple of the Vault",
        "If an artifact is put into a graveyard at the same time as Disciple of the Vault, its ability triggers for that artifact."
    );
    ruling!(
        "Fangren Marauder",
        "If Fangren Marauder has itself become an artifact for some reason, you’ll gain 5 life when it is put into a graveyard from the battlefield."
    );
    ruling!(
        "Soul Net",
        "If animated so it is a creature, it can be triggered off its own destruction."
    );
    // Disciple of the Vault and an artifact are destroyed together.
    supported("Disciple of the Vault");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Disciple of the Vault");
    t.battlefield(P0, "Sol Ring");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_yes(P0, true);
    cleansing(&mut t, P0);
    assert_eq!(t.life(P1), 19);
    // Fangren Marauder is an artifact (Mycosynth Lattice) and dies.
    supported("Fangren Marauder");
    supported("Mycosynth Lattice");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mycosynth Lattice");
    t.battlefield(P0, "Fangren Marauder");
    t.answer_yes(P0, true);
    wrath(&mut t, P0);
    assert!(t.in_graveyard(P0, "Fangren Marauder"));
    assert_eq!(t.life(P0), 25);
    // Soul Net is a 1/1 artifact creature (March of the Machines) and is destroyed.
    supported("Soul Net");
    supported("March of the Machines");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "March of the Machines");
    let net = t.battlefield(P0, "Soul Net");
    assert_eq!(t.pt(net), (1, 1));
    t.lands(P0, "Wastes", 1);
    t.answer_yes(P0, true);
    wrath(&mut t, P0);
    assert!(t.in_graveyard(P0, "Soul Net"));
    assert_eq!(t.life(P0), 21);
}

#[test]
fn graha_tia_triggers_once_for_simultaneous_deaths() {
    cr!("603.10a", "603.2c");
    ruling!(
        "G'raha Tia",
        "If G'raha Tia and one or more other creatures and/or artifacts you control die at the same time, its last ability will trigger once (as long as it hasn't already triggered this turn)."
    );
    supported("G'raha Tia");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "G'raha Tia");
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Hill Giant");
    t.battlefield(P0, "Sol Ring");
    let hand = t.hand_size(P0);
    cleansing(&mut t, P0);
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn chevill_leaving_with_the_bounty_creature_still_triggers() {
    cr!("603.10a", "122.1");
    ruling!(
        "Chevill, Bane of Monsters",
        "If Chevill leaves the battlefield at the same time that an opponent’s permanent with a bounty counter on it dies, you gain 3 life and draw a card. If Chevill leaves the battlefield before that permanent dies, Chevill’s last ability won’t be around to trigger."
    );
    supported("Chevill, Bane of Monsters");
    // Together: it triggers.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Chevill, Bane of Monsters");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.add_counters(Entity::Object(bears), "bounty", 1, None);
    let hand = t.hand_size(P0);
    wrath(&mut t, P0);
    assert_eq!(t.life(P0), 23);
    assert_eq!(t.hand_size(P0), hand + 1);
    // Chevill first, then the creature: nothing.
    let mut t = TestGame::new(2);
    let chevill = t.battlefield(P0, "Chevill, Bane of Monsters");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.add_counters(Entity::Object(bears), "bounty", 1, None);
    t.g.destroy(chevill, None);
    t.resolve_all();
    t.g.destroy(bears, None);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
}

#[test]
fn chevill_triggers_for_a_bounty_permanent_that_stopped_being_a_creature() {
    cr!("603.10a", "613.1d");
    ruling!(
        "Chevill, Bane of Monsters",
        "If the creature or planeswalker with a bounty counter on it ceases to be a creature or planeswalker, Chevill’s last ability will still trigger if that permanent is put into a graveyard from the battlefield."
    );
    // P1's Sol Ring is a creature under March of the Machines; it gets the bounty counter,
    // then March leaves and the (noncreature) Sol Ring is destroyed.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Chevill, Bane of Monsters");
    let march = t.battlefield(P1, "March of the Machines");
    let ring = t.battlefield(P1, "Sol Ring");
    t.answer_targets(P0, &[Entity::Object(ring)]);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.counters(ring, "bounty"), 1);
    t.g.destroy(march, None);
    t.resolve_all();
    assert!(!t.obj_now(ring).is(CardType::Creature));
    t.g.destroy(ring, None);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Sol Ring"));
    assert_eq!(t.life(P0), 23);
}

#[test]
fn chevill_looks_for_any_bounty_counter() {
    cr!("603.4", "122.1");
    ruling!(
        "Chevill, Bane of Monsters",
        "Chevill’s middle ability and last ability look for any bounty counter on your opponents’ permanents, not just one from that Chevill."
    );
    // P0 and P1 each control a Chevill; P0 put a bounty counter on P2's Bears.
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Chevill, Bane of Monsters");
    t.battlefield(P1, "Chevill, Bane of Monsters");
    let bears = t.battlefield(P2, "Grizzly Bears");
    let giant = t.battlefield(P2, "Hill Giant");
    t.g.add_counters(Entity::Object(bears), "bounty", 1, None);
    // P1's upkeep: its Chevill's intervening "if" fails; no counter on the Giant.
    t.answer_targets(P1, &[Entity::Object(giant)]);
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.counters(giant, "bounty"), 0);
    // The Bears die: both Chevills trigger.
    t.g.destroy(bears, None);
    t.resolve_all();
    assert_eq!(t.life(P0), 23);
    assert_eq!(t.life(P1), 23);
}

#[test]
fn molder_beast_gets_its_bonus_too_late_for_that_combat() {
    cr!("603.10a", "510.2", "702.19c");
    ruling!(
        "Molder Beast",
        "If Molder Beast and an artifact creature are both involved in combat during the same combat damage step, and that artifact creature is destroyed due to lethal combat damage, Molder Beast's triggered ability will trigger — but it will get the bonus too late to affect that combat."
    );
    supported("Molder Beast");
    let mut t = TestGame::new(2);
    let beast = t.battlefield(P0, "Molder Beast");
    let thopter = t.battlefield(P1, "Ornithopter");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(beast, Entity::Player(P1))], &[(thopter, beast)]);
    assert!(t.in_graveyard(P1, "Ornithopter"));
    // 5 power: 2 to the Ornithopter, 3 tramples over; the +2/+0 came afterwards.
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.pt(beast), (7, 3));
}

#[test]
fn curse_of_clinging_webs_makes_a_spider_even_without_the_card() {
    cr!("608.2c", "400.7");
    ruling!(
        "Curse of Clinging Webs",
        "Exiling the card isn't required to get the Spider token. You just do so if you can."
    );
    supported("Curse of Clinging Webs");
    let mut t = TestGame::new(2);
    let curse = t.battlefield(P0, "Curse of Clinging Webs");
    t.g.attach(curse, Entity::Player(P1));
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.destroy(bears, None);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    // The card leaves the graveyard before the ability resolves.
    let card = t.g.current(bears);
    t.g.move_object(card, Zone::Library(P1), MoveCause::Effect, None);
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Spider").len(), 1);
    assert!(!t.in_exile("Grizzly Bears"));
    // With the card there, it's exiled too.
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.destroy(bears, None);
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    assert_eq!(with_subtype(&t, P0, "Spider").len(), 2);
}
