//! Rulings batch P124 — damage and combat: assigning combat damage by toughness (CR
//! 510.1a), prevention ending with its source (CR 615), simultaneous damage and triggers
//! (CR 603.10a, 510.2), double strike with trample (CR 702.19c), fights with an illegal
//! target (CR 701.14b), ward triggers (CR 702.21a), loyalty and damage to planeswalkers
//! (CR 120.3c, 306.8).

use crate::r_p124_common::*;
use crate::r_p130_common::loyalty;
use mtg_engine::ability::Modification;
use mtg_engine::keywords::{Keyword, KeywordKind};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters::LOYALTY;
use mtg_engine::*;

fn give_keywords(t: &mut TestGame, id: ObjectId, kws: &[KeywordKind]) {
    modify_until_eot(
        t,
        id,
        kws.iter()
            .map(|k| Modification::AddKeyword(Keyword::new(*k)))
            .collect(),
    );
}

#[test]
fn bedrock_tortoise_changes_only_the_combat_damage_assigned() {
    cr!("510.1a", "701.14a", "208.3");
    ruling!(
        "Bedrock Tortoise",
        "Bedrock Tortoise's ability doesn't actually change any creature's power. It changes only the amount of combat damage the creature assigns."
    );
    supported("Bedrock Tortoise");
    // A 0/6 Tortoise attacks unblocked: 6 damage, though its power is still 0.
    let mut t = TestGame::new(2);
    let tortoise = t.battlefield(P0, "Bedrock Tortoise");
    t.attack(&[(tortoise, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 14);
    assert_eq!(t.pt(tortoise), (0, 6));
    // Prey Upon: it fights with its power, 0.
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P0, Step::PostcombatMain);
    cast_resolve(&mut t, P0, "Prey Upon", &[obj(tortoise), obj(bears)]);
    assert_eq!(t.obj_now(bears).damage, 0);
    assert_eq!(t.obj_now(tortoise).damage, 2);
}

#[test]
fn rescue_retriever_leaving_before_damage_ends_the_prevention() {
    cr!("615.1", "510.1", "611.3a");
    ruling!(
        "Rescue Retriever",
        "If Rescue Retriever leaves the battlefield before combat damage is dealt to your other attacking Soldiers, they will take damage as normal."
    );
    supported("Rescue Retriever");
    for leaves in [false, true] {
        let mut t = TestGame::new(2);
        let retriever = t.battlefield(P0, "Rescue Retriever");
        let vanguard = t.battlefield(P0, "Elite Vanguard");
        let bears = t.battlefield(P1, "Grizzly Bears");
        to_blockers(&mut t, &[(vanguard, Entity::Player(P1))], &[(bears, vanguard)]);
        if leaves {
            exile_now(&mut t, retriever);
        }
        t.advance_to(P0, Step::EndOfCombat);
        assert_eq!(t.on_battlefield(vanguard), !leaves, "Retriever left: {leaves}");
        assert!(!t.on_battlefield(bears));
    }
}

#[test]
fn mikaeus_destroys_any_human_that_deals_damage_to_you() {
    cr!("603.10a", "603.2", "120.3");
    ruling!(
        "Mikaeus, the Unhallowed",
        "Any damage dealt to you by a Human will cause Mikaeus’s ability to trigger, including noncombat damage dealt to you by a Human and damage dealt to you by a Human you control."
    );
    ruling!(
        "Mikaeus, the Unhallowed",
        "If a Human deals damage to you at the same time that Mikaeus is dealt lethal damage, Mikaeus’s triggered ability triggers. That Human will be destroyed even though Mikaeus has left the battlefield."
    );
    supported("Mikaeus, the Unhallowed");
    // P0's own Prodigal Sorcerer (a Human Wizard) pings P0.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mikaeus, the Unhallowed");
    let sorcerer = t.battlefield(P0, "Prodigal Sorcerer");
    t.activate(P0, sorcerer, 0, &[Entity::Player(P0)]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P0), 19);
    assert!(!t.on_battlefield(sorcerer));
    // P1's Human deals damage to P0 and lethal damage to Mikaeus at once.
    let mut t = TestGame::new(2);
    let mikaeus = t.battlefield(P0, "Mikaeus, the Unhallowed");
    let human = t.battlefield(P1, "Elite Vanguard");
    t.g.deal_damage_batch(
        vec![(human, Entity::Player(P0), 1), (human, Entity::Object(mikaeus), 5)],
        false,
    );
    t.g.flush_events();
    t.settle();
    assert!(!t.on_battlefield(mikaeus));
    t.resolve_all();
    assert!(!t.on_battlefield(human));
}

#[test]
fn domri_emblem_double_strike_and_trample() {
    cr!("702.19b", "702.4b", "510.1c", "510.4");
    ruling!(
        "Domri Rade",
        "If a creature has both double strike and trample, the combat damage it assigned during the first combat damage step will be considered when determining how much damage can trample through in the second combat damage step."
    );
    supported("Domri Rade");
    // (blocker, P1's life after): a 1/1 blocker dies in the first step, so all 3 of the
    // second step trample over (2 + 3); a 0/5 Wall of Roots takes 3 in the first step, and
    // needs only 2 more in the second (0 + 1).
    for (blocker, life) in [("Llanowar Elves", 15), ("Wall of Roots", 19)] {
        let mut t = TestGame::new(2);
        let domri = t.battlefield(P0, "Domri Rade");
        loyalty(&mut t, domri, 7);
        t.activate(P0, domri, 2, &[]).unwrap();
        t.resolve_all();
        let giant = t.battlefield(P0, "Hill Giant");
        let b = t.battlefield(P1, blocker);
        t.attack(&[(giant, Entity::Player(P1))], &[(b, giant)]);
        assert!(!t.on_battlefield(b), "{blocker}");
        assert_eq!(t.life(P1), life, "{blocker}");
    }
}

#[test]
fn domri_fight_with_an_illegal_target_deals_no_damage() {
    cr!("701.14b", "608.2b");
    ruling!(
        "Domri Rade",
        "If either target of Domri's second ability is an illegal target when the ability tries to resolve, neither creature will deal or be dealt damage."
    );
    for gone in [0, 1] {
        let mut t = TestGame::new(2);
        let domri = t.battlefield(P0, "Domri Rade");
        let giant = t.battlefield(P0, "Hill Giant");
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.activate(P0, domri, 1, &[obj(giant), obj(bears)]).unwrap();
        let (leaving, staying) = if gone == 0 { (giant, bears) } else { (bears, giant) };
        bounce(&mut t, leaving);
        t.resolve_all();
        assert_eq!(t.obj_now(staying).damage, 0, "target {gone} left");
        assert!(t.on_battlefield(staying));
    }
}

#[test]
fn djeru_prevents_one_damage_from_each_source() {
    cr!("615.1", "510.2", "306.8");
    ruling!(
        "Djeru, With Eyes Open",
        "If multiple sources would deal damage to one or more planeswalkers you control at once (for example, several attacking creatures), 1 damage from each of those sources to each of those planeswalkers is prevented."
    );
    supported("Djeru, With Eyes Open");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Djeru, With Eyes Open");
    let liliana = t.battlefield(P0, "Liliana Vess");
    assert_eq!(t.counters(liliana, LOYALTY), 5);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::BeginningOfCombat);
    t.attack(&[(a, obj(liliana)), (b, obj(liliana))], &[]);
    assert_eq!(t.counters(liliana, LOYALTY), 3);
}

#[test]
fn deification_applies_when_the_last_creature_dies_in_the_same_damage_step() {
    cr!("120.3c", "510.2", "704.3");
    ruling!(
        "Deification",
        "Deification's last ability will apply to a planeswalker dealt combat damage in the same combat damage step that its controller's last creature is destroyed."
    );
    supported("Deification");
    let mut t = TestGame::new(2);
    let shrine = t.battlefield(P0, "Deification");
    t.g.objects[shrine.0 as usize].choices.creature_type = Some("Jace".into());
    let jace = t.battlefield(P0, "Jace Beleren");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let wurm = t.battlefield(P1, "Craw Wurm");
    let giant = t.battlefield(P1, "Hill Giant");
    t.set_step(P1, Step::BeginningOfCombat);
    t.attack(
        &[(wurm, obj(jace)), (giant, Entity::Player(P0))],
        &[(bears, giant)],
    );
    assert!(!t.on_battlefield(bears));
    assert!(t.on_battlefield(jace));
    assert_eq!(t.counters(jace, LOYALTY), 1);
}

#[test]
fn ready_willing_multiple_lifelink_is_redundant() {
    cr!("702.15f", "702.2f");
    ruling!(
        "Ready // Willing",
        "Multiple instances of deathtouch or lifelink are redundant."
    );
    // A creature with lifelink and deathtouch gains both again: 2 damage gains 2 life.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    give_keywords(&mut t, bears, &[KeywordKind::Lifelink, KeywordKind::Deathtouch]);
    give_keywords(&mut t, bears, &[KeywordKind::Lifelink, KeywordKind::Deathtouch]);
    let giant = t.battlefield(P1, "Hill Giant");
    t.attack(&[(bears, Entity::Player(P1))], &[(giant, bears)]);
    assert_eq!(t.life(P0), 22);
    assert!(!t.on_battlefield(giant));
}

#[test]
fn sparkshaper_visionary_planeswalkers_become_birds() {
    cr!("306.8", "120.3c", "606.3");
    ruling!(
        "Sparkshaper Visionary",
        "They don't lose any loyalty counters or abilities, and you can still activate their loyalty abilities if you haven't done so yet this turn. They don't lose loyalty if they're dealt damage while they're not planeswalkers."
    );
    supported("Sparkshaper Visionary");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sparkshaper Visionary");
    let jace = t.battlefield(P0, "Jace Beleren");
    t.answer_targets(P0, &[obj(jace)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    t.g.recompute();
    assert!(t.obj_now(jace).is(mtg_engine::types::CardType::Creature));
    assert!(!t.obj_now(jace).is(mtg_engine::types::CardType::Planeswalker));
    assert_eq!(t.pt(jace), (3, 3));
    assert_eq!(t.counters(jace, LOYALTY), 3);
    // Damage doesn't remove loyalty.
    damage(&mut t, jace, 1);
    assert_eq!(t.counters(jace, LOYALTY), 3);
    // Its loyalty abilities can still be activated (+2: each player draws a card).
    t.advance_to(P0, Step::PostcombatMain);
    let hand = t.hand_size(P0);
    t.activate(P0, jace, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.counters(jace, LOYALTY), 5);
}

#[test]
fn ward_already_triggered_still_resolves_when_its_source_leaves() {
    cr!("702.21a", "113.7a", "603.4");
    ruling!(
        "Coppercoat Vanguard",
        "Once a ward ability has triggered, causing that Human to lose ward by removing Coppercoat Vanguard won't affect that ability. The appropriate player will still have to pay {1} or see their spell or ability countered."
    );
    ruling!(
        "Gold-Forged Thopteryx",
        "Once a ward ability has triggered, causing that legendary permanent to lose ward by removing Gold-Forged Thopteryx won't affect that ability. The appropriate player will still have to pay {2} or see their spell or ability countered."
    );
    for (lord, target) in [
        ("Coppercoat Vanguard", "Elite Vanguard"),
        ("Gold-Forged Thopteryx", "Isamaru, Hound of Konda"),
    ] {
        supported(lord);
        let mut t = TestGame::new(2);
        let lord_id = t.battlefield(P0, lord);
        let creature = t.battlefield(P0, target);
        t.set_step(P1, Step::PrecombatMain);
        t.lands(P1, "Mountain", 1);
        let shock = t.hand(P1, "Shock");
        t.cast(P1, shock).target(creature).go();
        t.settle();
        assert_eq!(t.stack_len(), 2, "{lord}: ward triggered");
        exile_now(&mut t, lord_id);
        t.resolve_all();
        assert!(t.on_battlefield(creature), "{lord}: Shock was countered");
        assert!(t.in_graveyard(P1, "Shock"));
    }
}
