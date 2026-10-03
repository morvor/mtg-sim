//! Rulings batch P201 — amass (CR 701.47) in spells and abilities with targets, reflexive
//! triggers, "Amass N" alongside other effects, and choosing among several Armies.

use crate::r_p108_common::resolved;
use crate::r_s01_common::{give_mana_for, supported, with_subtype};
use mtg_engine::decision::Decision;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn armies(t: &TestGame, p: PlayerId) -> Vec<ObjectId> {
    with_subtype(t, p, "Army")
}

/// P0 begins casting the real spell `name` (with the mana for it), answering one target
/// choice per entry of `slots` (an empty entry chooses no target), and returns the spell.
fn cast(t: &mut TestGame, name: &str, slots: &[&[Entity]]) -> ObjectId {
    supported(name);
    give_mana_for(t, P0, name);
    let c = t.hand(P0, name);
    for s in slots {
        t.answer_targets(P0, s);
    }
    t.cast(P0, c).go()
}

/// Makes the target illegal: it leaves the battlefield.
fn remove(t: &mut TestGame, id: ObjectId) {
    let id = t.g.current(id);
    t.g.move_object(
        id,
        mtg_engine::object::Zone::Exile,
        mtg_engine::events::MoveCause::Effect,
        None,
    );
}

#[test]
fn a_spell_with_all_targets_illegal_doesnt_amass() {
    cr!("608.2b", "701.47a");
    ruling!(
        "Enter the God-Eternals",
        "If both targets are no longer legal targets as the spell tries to resolve, it doesn't resolve."
    );
    ruling!(
        "Treason of Isengard",
        "You can cast Treason of Isengard without a target just to amass Orcs."
    );
    ruling!(
        "Bleeding Edge",
        "You may cast Bleeding Edge without choosing a target creature. You'll just amass 2."
    );
    // Enter the God-Eternals: the creature leaves, the player gains hexproof (Lazotep
    // Plating, "You and permanents you control gain hexproof until end of turn.").
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let spell = cast(
        &mut t,
        "Enter the God-Eternals",
        &[&[Entity::Object(bears)], &[Entity::Player(P1)]],
    );
    remove(&mut t, bears);
    give_mana_for(&mut t, P1, "Lazotep Plating");
    let plating = t.hand(P1, "Lazotep Plating");
    t.cast(P1, plating).go();
    t.resolve_all();
    assert!(!resolved(&t, spell));
    assert!(armies(&t, P0).is_empty());
    assert_eq!(t.life(P0), 20);
    // Treason of Isengard and Bleeding Edge: no target chosen, or a target that becomes
    // illegal.
    for (name, target_name) in [
        ("Treason of Isengard", "Lightning Bolt"),
        ("Bleeding Edge", "Grizzly Bears"),
    ] {
        let mut t = TestGame::new(2);
        cast(&mut t, name, &[&[]]);
        t.resolve_all();
        let a = armies(&t, P0);
        assert_eq!(a.len(), 1, "{name}");
        assert_eq!(t.pt(a[0]), (2, 2));
        let mut t = TestGame::new(2);
        let target = if name == "Bleeding Edge" {
            t.battlefield(P1, target_name)
        } else {
            t.graveyard(P0, target_name)
        };
        let spell = cast(&mut t, name, &[&[Entity::Object(target)]]);
        remove(&mut t, target);
        t.resolve_all();
        assert!(!resolved(&t, spell), "{name}");
        assert!(armies(&t, P0).is_empty(), "{name}");
    }
}

#[test]
fn a_spell_with_one_illegal_target_does_the_rest() {
    cr!("608.2b", "608.2c", "701.47a");
    ruling!(
        "Enter the God-Eternals",
        "If the target creature is no longer a legal target as the spell resolves but the player is a legal target, no creature is dealt damage and you gain no life, but the target player still moves the top cards of their library and you still amass 4."
    );
    ruling!(
        "Enter the God-Eternals",
        "If the target player is no longer a legal target as the spell resolves but the creature is a legal target, the creature is dealt damage, you gain that much life, and you amass 4. No player moves cards from their library."
    );
    // The creature becomes illegal.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast(
        &mut t,
        "Enter the God-Eternals",
        &[&[Entity::Object(bears)], &[Entity::Player(P1)]],
    );
    remove(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.graveyard_size(P1), 4);
    let a = armies(&t, P0);
    assert_eq!(a.len(), 1);
    assert_eq!(t.pt(a[0]), (4, 4));
    // The player becomes illegal (hexproof from Lazotep Plating); the creature is P0's
    // own Hill Giant.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let lib1 = t.library_size(P1);
    cast(
        &mut t,
        "Enter the God-Eternals",
        &[&[Entity::Object(giant)], &[Entity::Player(P1)]],
    );
    give_mana_for(&mut t, P1, "Lazotep Plating");
    let plating = t.hand(P1, "Lazotep Plating");
    t.cast(P1, plating).go();
    t.resolve_all();
    assert_eq!(t.library_size(P1), lib1);
    assert_eq!(t.graveyard_size(P1), 1); // just the Plating
    assert!(t.in_graveyard(P0, "Hill Giant"));
    assert_eq!(t.life(P0), 24);
    let a = armies(&t, P0);
    assert_eq!(a.len(), 1);
    assert_eq!(t.pt(a[0]), (4, 4));
}

#[test]
fn a_creature_dealt_lethal_damage_is_still_there_for_the_rest_of_the_spell() {
    cr!("608.2c", "704.3", "603.2");
    ruling!(
        "Enter the God-Eternals",
        "If the target creature is dealt lethal damage by Enter the God-Eternals, it'll still be on the battlefield while you gain life, the target player moves cards, and you amass 4."
    );
    supported("Soul Warden");
    // P1's Soul Warden ("Whenever another creature enters, you gain 1 life.") is dealt 4
    // damage, but sees the Zombie Army enter.
    let mut t = TestGame::new(2);
    let warden = t.battlefield(P1, "Soul Warden");
    cast(
        &mut t,
        "Enter the God-Eternals",
        &[&[Entity::Object(warden)], &[Entity::Player(P1)]],
    );
    t.resolve_all();
    assert!(!t.on_battlefield(warden));
    assert_eq!(t.life(P0), 24);
    assert_eq!(t.life(P1), 21);
}

#[test]
fn amass_zero_without_an_army_makes_a_0_0_that_dies() {
    cr!("701.47a", "704.5f");
    ruling!(
        "Invade the City",
        "If there are no instant or sorcery cards in your graveyard, you'll amass 0."
    );
    let mut t = TestGame::new(2);
    let spell = cast(&mut t, "Invade the City", &[]);
    t.g.resolve_top();
    // The 0/0 token exists until state-based actions are checked.
    let a = armies(&t, P0);
    assert_eq!(a.len(), 1);
    assert_eq!(t.pt(a[0]), (0, 0));
    assert!(resolved(&t, spell));
    t.settle();
    assert!(armies(&t, P0).is_empty());
}

#[test]
fn lazotep_plating_gives_the_new_army_hexproof() {
    cr!("701.47a", "608.2c", "611.2c", "702.11b");
    ruling!(
        "Lazotep Plating",
        "If you create a Zombie Army token when Lazotep Plating instructs you to amass 1, that token will gain hexproof until end of turn."
    );
    ruling!(
        "Lazotep Plating",
        "You amass 1 and grant hexproof all while Lazotep Plating is resolving."
    );
    let mut t = TestGame::new(2);
    cast(&mut t, "Lazotep Plating", &[]);
    t.resolve();
    let a = armies(&t, P0);
    assert_eq!(a.len(), 1);
    assert!(t
        .obj_now(a[0])
        .chars
        .has_keyword(mtg_engine::keywords::KeywordKind::Hexproof));
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn several_armies_you_choose_which_one_becomes_a_sliver_or_orc() {
    cr!("701.47a", "205.3d");
    ruling!(
        "Lazotep Sliver",
        "In the rare case that you control multiple Army creatures (perhaps because you played a creature with changeling) while you amass Slivers, you choose which of your Army creatures to put the +1/+1 counters on."
    );
    ruling!(
        "Lazotep Sliver",
        "To amass Slivers 2, if you don't control an Army creature, create a 0/0 black Sliver Army creature token."
    );
    ruling!(
        "Warg Rider",
        "In the rare case that you control multiple Army creatures (perhaps because you cast a creature with changeling) while you amass Orcs, you choose which of your Army creatures to put the +1/+1 counters on."
    );
    supported("Lazotep Sliver");
    supported("Warg Rider");
    // Lazotep Sliver: "Whenever a nontoken Sliver you control dies, amass Slivers 2."
    // Without an Army: a 0/0 black Sliver Army gets two counters.
    let mut t = TestGame::new(2);
    let sliver = t.battlefield(P0, "Lazotep Sliver");
    t.g.destroy(sliver, None);
    t.resolve_all();
    let a = armies(&t, P0);
    assert_eq!(a.len(), 1);
    let o = t.obj_now(a[0]);
    assert!(o.is_token() && o.chars.has_subtype("Sliver"));
    assert_eq!(t.pt(a[0]), (2, 2));
    // With a Zombie Army and a changeling (also an Army), choose the Zombie Army: it
    // becomes a Sliver too.
    let mut t = TestGame::new(2);
    cast(&mut t, "Lazotep Plating", &[]);
    t.resolve_all();
    let zombie = armies(&t, P0)[0];
    let changeling = t.battlefield(P0, "Changeling Outcast");
    let sliver = t.battlefield(P0, "Lazotep Sliver");
    assert_eq!(armies(&t, P0).len(), 2);
    t.answer_choose(P0, &[Entity::Object(zombie)]);
    t.g.destroy(sliver, None);
    t.resolve_all();
    assert_eq!(t.pt(zombie), (3, 3));
    assert!(t.obj_now(zombie).chars.has_subtype("Sliver"));
    assert!(t.obj_now(zombie).chars.has_subtype("Zombie"));
    assert_eq!(t.counters(changeling, "+1/+1"), 0);
    // Warg Rider: "At the beginning of combat on your turn, amass Orcs 2." Choose the
    // changeling this time.
    let mut t = TestGame::new(2);
    cast(&mut t, "Lazotep Plating", &[]);
    t.resolve_all();
    let zombie = armies(&t, P0)[0];
    let changeling = t.battlefield(P0, "Changeling Outcast");
    t.battlefield(P0, "Warg Rider");
    t.answer_choose(P0, &[Entity::Object(changeling)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert_eq!(t.counters(changeling, "+1/+1"), 2);
    assert_eq!(t.counters(zombie, "+1/+1"), 1);
    assert!(!t.obj_now(zombie).chars.has_subtype("Orc"));
}

#[test]
fn amass_happens_whatever_else_the_spell_did() {
    cr!("701.47a", "608.2c");
    ruling!(
        "Crush Dissent",
        "You amass 2 even if the controller of the spell pays {2}."
    );
    ruling!(
        "Toll of the Invasion",
        "You still amass 1 even if that player has no nonland cards to discard."
    );
    // Crush Dissent: "Counter target spell unless its controller pays {2}. Amass Zombies
    // 2." P1 pays.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let div = t.hand(P1, "Divination");
    give_mana_for(&mut t, P1, "Divination");
    let div = t.cast(P1, div).go();
    t.lands(P1, "Island", 2);
    t.answer_yes(P1, true);
    cast(&mut t, "Crush Dissent", &[&[Entity::Object(div)]]);
    t.resolve();
    assert!(t.g.stack.contains(&div));
    let a = armies(&t, P0);
    assert_eq!(a.len(), 1);
    assert_eq!(t.pt(a[0]), (2, 2));
    // Toll of the Invasion: P1's hand has only lands.
    let mut t = TestGame::new(2);
    t.hand(P1, "Forest");
    cast(&mut t, "Toll of the Invasion", &[&[Entity::Player(P1)]]);
    t.resolve_all();
    assert!(t.in_hand(P1, "Forest"));
    let a = armies(&t, P0);
    assert_eq!(a.len(), 1);
    assert_eq!(t.pt(a[0]), (1, 1));
}

#[test]
fn commence_the_endgame_counts_the_hand_after_drawing() {
    cr!("608.2c", "701.47a");
    ruling!(
        "Commence the Endgame",
        "You draw two cards and amass X all while Commence the Endgame is resolving."
    );
    let mut t = TestGame::new(2);
    t.hand(P0, "Forest");
    cast(&mut t, "Commence the Endgame", &[]);
    t.resolve_all();
    // One card in hand, then two drawn: amass 3.
    let a = armies(&t, P0);
    assert_eq!(t.hand_size(P0), 3);
    assert_eq!(t.pt(a[0]), (3, 3));
}

#[test]
fn foray_of_orcs_targets_with_a_reflexive_trigger() {
    cr!("603.12", "701.47a");
    ruling!(
        "Foray of Orcs",
        "You don't choose a target for Foray of Orcs at the time you cast it."
    );
    supported("Foray of Orcs");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let from = t.asked().len();
    cast(&mut t, "Foray of Orcs", &[]);
    assert!(!t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseTargets { .. })));
    t.answer_targets(P0, &[Entity::Object(giant)]);
    t.resolve();
    // The spell finished; its reflexive trigger is on the stack with its target, and
    // players may respond to it.
    assert!(t.in_graveyard(P0, "Foray of Orcs"));
    assert_eq!(t.stack_len(), 1);
    assert!(t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseTargets { .. })));
    assert!(t.on_battlefield(giant));
    t.resolve();
    // 2 damage (the Army's power) to the 3/3.
    assert!(t.on_battlefield(giant));
    assert_eq!(t.obj_now(giant).damage, 2);
}

#[test]
fn shagrat_amasses_without_a_target_but_not_with_an_illegal_one() {
    cr!("603.3d", "608.2b", "701.47a");
    ruling!(
        "Shagrat, Loot Bearer",
        "When Shagrat, Loot Bearer's ability triggers, you can choose not to target an Equipment just to amass Orcs."
    );
    supported("Shagrat, Loot Bearer");
    // Shagrat already wears a Bonesplitter: no target, amass Orcs 1.
    let mut t = TestGame::new(2);
    let shagrat = t.battlefield(P0, "Shagrat, Loot Bearer");
    let bs = t.battlefield(P0, "Bonesplitter");
    t.g.attach(bs, Entity::Object(shagrat));
    t.answer_targets(P0, &[]);
    t.attack(&[(shagrat, Entity::Player(P1))], &[]);
    let a = armies(&t, P0);
    assert_eq!(a.len(), 1);
    assert_eq!(t.counters(a[0], "+1/+1"), 1);
    // A second Bonesplitter is targeted, then leaves: nothing happens.
    let mut t = TestGame::new(2);
    let shagrat = t.battlefield(P0, "Shagrat, Loot Bearer");
    let bs = t.battlefield(P0, "Bonesplitter");
    t.g.attach(bs, Entity::Object(shagrat));
    let other = t.battlefield(P0, "Bonesplitter");
    t.answer_targets(P0, &[Entity::Object(other)]);
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(shagrat, Entity::Player(P1))]),
    );
    t.advance_to(P0, Step::DeclareAttackers);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    remove(&mut t, other);
    t.resolve_all();
    assert!(armies(&t, P0).is_empty());
}

#[test]
fn azog_with_no_target_amasses_nothing() {
    cr!("701.47a", "115.10");
    ruling!(
        "Azog, Moria's Ruin",
        "If no target is chosen for Azog's ability, \"its controller\" is undefined and no player amasses Goblins."
    );
    supported("Azog, Moria's Ruin");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[]);
    t.enter(P0, "Azog, Moria's Ruin");
    t.resolve_all();
    assert!(armies(&t, P0).is_empty());
    assert!(armies(&t, P1).is_empty());
    // With a target, its controller amasses.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.enter(P0, "Azog, Moria's Ruin");
    t.resolve_all();
    assert_eq!(armies(&t, P1).len(), 1);
}
