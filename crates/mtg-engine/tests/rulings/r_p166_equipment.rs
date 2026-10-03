//! Rulings batch P166 — Equipment synergies (CR 301.5, 702.6): attaching Equipment
//! another player controls, "equipped creature" triggers and statics, cost reductions for
//! equip abilities, abilities that put Equipment onto the battlefield attached, and
//! Equipment falling off mid-combat.

use crate::r_p125_common::at_p1;
use crate::r_s01_common::*;
use crate::r_s02_common::{can_activate, target_candidates};
use crate::r_s03_common::in_hand_with_mana;
use crate::r_s04_common::*;
use crate::r_s05_common::move_to;
use crate::r_s06_common::*;
use crate::r_s20_common::to_beginning_of_combat;
use crate::r_s21_common::legal_blocks;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

/// `p` activates the equip ability of `equipment` targeting `target` (with whatever mana
/// is available).
fn equip(
    t: &mut TestGame,
    p: PlayerId,
    equipment: ObjectId,
    target: ObjectId,
) -> Result<Option<ObjectId>, mtg_engine::casting::Illegal> {
    t.answer_targets(p, &[Entity::Object(target)]);
    activate_containing(t, p, equipment, "Equip")
}

fn controller(t: &TestGame, id: ObjectId) -> PlayerId {
    t.obj_now(id).controller
}

/// Advances the current combat to the step after first-strike combat damage, the active
/// player holding priority.
fn after_first_strike(t: &mut TestGame) {
    let ap = t.g.turn.active;
    t.advance_to(ap, Step::FirstStrikeDamage);
    t.settle();
}

// ---------------------------------------------------------------------------------------
// Armory Automaton
// ---------------------------------------------------------------------------------------

#[test]
fn armory_automaton_attaches_an_opponents_equipment() {
    cr!("301.5c", "301.5d", "602.5d", "702.6a");
    ruling!(
        "Armory Automaton",
        "Armory Automaton's ability can cause an Equipment one player controls to become attached to a creature another player controls. The controller of the Equipment can pay the equip cost to attach that Equipment to a creature they control, but only any time that player could cast a sorcery. The controller of Armory Automaton can't activate equip abilities of Equipment they don't control."
    );
    supported("Armory Automaton");
    let mut t = TestGame::new(2);
    let splitter = t.battlefield(P1, "Bonesplitter");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(splitter)]);
    t.answer_yes(P0, true);
    let auto = t.enter(P0, "Armory Automaton");
    t.settle();
    t.resolve_all();
    assert_eq!(attached_to(&t, splitter), Some(Entity::Object(auto)));
    assert_eq!(controller(&t, splitter), P1);
    assert_eq!(t.pt(auto), (4, 2));
    // P0 can't equip it.
    t.lands(P0, "Wastes", 1);
    assert!(!can_activate(&mut t, P0, splitter));
    // P1 can, only as a sorcery: not on P0's turn.
    t.lands(P1, "Wastes", 1);
    assert!(!can_activate(&mut t, P1, splitter));
    t.set_step(P1, Step::PrecombatMain);
    equip(&mut t, P1, splitter, bears).unwrap();
    t.resolve_all();
    assert_eq!(attached_to(&t, splitter), Some(Entity::Object(bears)));
}

#[test]
fn armory_automaton_leaving_first_does_nothing() {
    cr!("608.2b", "301.5c");
    ruling!(
        "Armory Automaton",
        "If Armory Automaton leaves the battlefield before its ability resolves, nothing happens to any of the Equipment it targeted. If they were already attached to other creatures, they remain attached to those creatures."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let splitter = attach_new(&mut t, P0, "Bonesplitter", bears);
    let free = t.battlefield(P0, "Short Sword");
    t.answer_targets(P0, &[Entity::Object(splitter), Entity::Object(free)]);
    t.answer_yes(P0, true);
    let auto = t.enter(P0, "Armory Automaton");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    move_to(&mut t, auto, Zone::Exile);
    t.resolve_all();
    assert_eq!(attached_to(&t, splitter), Some(Entity::Object(bears)));
    assert_eq!(attached_to(&t, free), None);
}

#[test]
fn equipment_attack_triggers_need_the_equipment_attached_as_attackers_are_declared() {
    cr!("508.1m", "603.2", "608.2h");
    ruling!(
        "Armory Automaton",
        "An ability of an Equipment that triggers \"whenever equipped creature attacks\" triggers only if the Equipment was attached to a creature at the moment that creature was declared as an attacker, and any references to \"that creature\" in the effect refer to the creature the Equipment was attached to when the ability triggered."
    );
    supported("Greatsword of Tyr");
    // Greatsword of Tyr: "Whenever equipped creature attacks, put a +1/+1 counter on it
    // and tap up to one target creature defending player controls."
    // Attached to a creature that doesn't attack, then moved by the Automaton's attack
    // trigger: no Greatsword trigger.
    let mut t = TestGame::new(2);
    let auto = t.battlefield(P0, "Armory Automaton");
    let thopter = t.battlefield(P0, "Ornithopter");
    let sword = attach_new(&mut t, P0, "Greatsword of Tyr", thopter);
    t.answer_targets(P0, &[Entity::Object(sword)]);
    t.answer_yes(P0, true);
    t.set_step(P0, Step::PrecombatMain);
    attack_with(&mut t, &at_p1(&[auto]));
    t.resolve_all();
    assert_eq!(attached_to(&t, sword), Some(Entity::Object(auto)));
    assert_eq!(t.counters(auto, counters::PLUS1), 0);
    // Attached to an attacker and moved to another creature before the trigger resolves:
    // "it" is still the creature it was attached to.
    supported("Brass Squire");
    let mut t = TestGame::new(2);
    let squire = t.battlefield(P0, "Brass Squire");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let sword = attach_new(&mut t, P0, "Greatsword of Tyr", bears);
    t.answer_targets(P0, &[]);
    t.set_step(P0, Step::PrecombatMain);
    attack_with(&mut t, &at_p1(&[bears]));
    assert_eq!(triggers_on_stack(&t, "+1/+1 counter"), 1);
    t.answer_targets(P0, &[Entity::Object(sword)]);
    t.answer_targets(P0, &[Entity::Object(squire)]);
    activate_containing(&mut t, P0, squire, "Attach").unwrap();
    t.resolve();
    assert_eq!(attached_to(&t, sword), Some(Entity::Object(squire)));
    t.resolve_all();
    assert_eq!(t.counters(bears, counters::PLUS1), 1);
    assert_eq!(t.counters(squire, counters::PLUS1), 0);
}

#[test]
fn you_on_an_opponents_equipment_is_its_controller() {
    cr!("109.5", "113.8", "702.6a");
    ruling!(
        "Armory Automaton",
        "If an Equipment an opponent controls is attached to a creature you control, any ability of that Equipment that says \"you\" refers to that opponent. However, if the Equipment says that the equipped creature has an ability, the word \"you\" in that ability refers to you, the controller of the creature."
    );
    supported("Sword of Fire and Ice");
    supported("Diviner's Wand");
    // P1's Sword of Fire and Ice on P0's Grizzly Bears: "Whenever equipped creature deals
    // combat damage to a player, this Equipment deals 2 damage to any target and you draw
    // a card." P1 draws.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let sword = t.battlefield(P1, "Sword of Fire and Ice");
    assert!(t.g.attach(sword, Entity::Object(bears)));
    stack_library(&mut t, P0, &["Opt"]);
    stack_library(&mut t, P1, &["Opt"]);
    let (h0, h1) = (t.hand_size(P0), t.hand_size(P1));
    t.answer_targets(P1, &[Entity::Player(P0)]);
    t.set_step(P0, Step::PrecombatMain);
    attack_with(&mut t, &at_p1(&[bears]));
    t.resolve_all();
    block_and_finish(&mut t, P1, &[]);
    t.resolve_all();
    assert_eq!(t.hand_size(P1), h1 + 1);
    assert_eq!(t.hand_size(P0), h0);
    // P1's Diviner's Wand on P0's creature: "Equipped creature has ... '{4}: Draw a
    // card.'" P0 activates it and draws.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let wand = t.battlefield(P1, "Diviner's Wand");
    assert!(t.g.attach(wand, Entity::Object(bears)));
    stack_library(&mut t, P0, &["Opt"]);
    let (h0, h1) = (t.hand_size(P0), t.hand_size(P1));
    t.lands(P0, "Wastes", 4);
    activate_containing(&mut t, P0, bears, "Draw a card").unwrap();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), h0 + 1);
    assert_eq!(t.hand_size(P1), h1);
}

// ---------------------------------------------------------------------------------------
// Balan, Wandering Knight
// ---------------------------------------------------------------------------------------

#[test]
fn balan_can_attach_at_instant_speed() {
    cr!("602.5d", "117.1b");
    ruling!(
        "Balan, Wandering Knight",
        "Balan's activated ability has no timing restriction. You can activate it any time you have priority."
    );
    supported("Balan, Wandering Knight");
    let mut t = TestGame::new(2);
    let balan = t.battlefield(P0, "Balan, Wandering Knight");
    let a = t.battlefield(P0, "Short Sword");
    let b = t.battlefield(P0, "Bonesplitter");
    t.lands(P0, "Plains", 2);
    // On the opponent's turn, with a spell on the stack.
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Island", 1);
    let opt = t.hand(P1, "Opt");
    t.cast(P1, opt).go();
    activate_containing(&mut t, P0, balan, "Attach all").unwrap();
    t.resolve();
    assert_eq!(attached_to(&t, a), Some(Entity::Object(balan)));
    assert_eq!(attached_to(&t, b), Some(Entity::Object(balan)));
    assert!(has_kw(&t, balan, KeywordKind::DoubleStrike));
}

#[test]
fn balan_gaining_double_strike_after_first_strike_damage_deals_regular_damage() {
    cr!("702.4c", "702.4d", "510.4");
    ruling!(
        "Balan, Wandering Knight",
        "If Balan deals first-strike damage and then gains double strike (most likely because it picked up some Equipment with its activated ability after first-strike damage was dealt), it will also deal regular combat damage."
    );
    let mut t = TestGame::new(2);
    let balan = t.battlefield(P0, "Balan, Wandering Knight");
    t.battlefield(P0, "Short Sword");
    t.battlefield(P0, "Short Sword");
    t.lands(P0, "Plains", 2);
    t.set_step(P0, Step::PrecombatMain);
    attack_with(&mut t, &at_p1(&[balan]));
    t.answer(
        P1,
        DecisionKind::Blockers,
        mtg_engine::decision::Answer::Blockers(vec![]),
    );
    after_first_strike(&mut t);
    assert_eq!(t.life(P1), 17);
    activate_containing(&mut t, P0, balan, "Attach all").unwrap();
    t.resolve();
    assert!(has_kw(&t, balan, KeywordKind::DoubleStrike));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 12);
}

// ---------------------------------------------------------------------------------------
// Static bonuses
// ---------------------------------------------------------------------------------------

#[test]
fn bearded_axe_counts_itself() {
    cr!("301.5", "611.3a");
    ruling!(
        "Bearded Axe",
        "Bearded Axe will count itself, so it will give at least +1/+1 in most cases."
    );
    supported("Bearded Axe");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Bearded Axe", bears);
    t.settle();
    assert_eq!(t.pt(bears), (3, 3));
}

#[test]
fn stone_haven_outfitter_bonus_and_trigger_are_once_per_creature() {
    cr!("613.4c", "603.2");
    ruling!(
        "Stone Haven Outfitter",
        "Equipped creatures will get just +1/+1 from Stone Haven Outfitter, no matter how many Equipment are attached to them. Similarly, the last ability will trigger just once per equipped creature."
    );
    ruling!(
        "Stone Haven Outfitter",
        "In this context, \"equipped creatures you control\" and \"equipped creature you control\" refer to any creatures you control with Equipment attached to them."
    );
    supported("Stone Haven Outfitter");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Stone Haven Outfitter");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let elves = t.battlefield(P0, "Llanowar Elves");
    attach_new(&mut t, P0, "Short Sword", bears);
    attach_new(&mut t, P0, "Short Sword", bears);
    attach_new(&mut t, P0, "Bonesplitter", giant);
    t.settle();
    assert_eq!(t.pt(bears), (5, 5), "2 Short Swords and one +1/+1");
    assert_eq!(t.pt(giant), (6, 4));
    assert_eq!(t.pt(elves), (1, 1));
    stack_library(&mut t, P0, &["Opt", "Opt", "Opt"]);
    let hand = t.hand_size(P0);
    crate::r_p146_common::destroy_together(&mut t, &[bears]);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn militant_inquisitor_counts_all_your_equipment_on_top_of_their_bonuses() {
    cr!("613.4c", "301.5");
    ruling!(
        "Militant Inquisitor",
        "Militant Inquisitor's ability applies in addition to any effects from those Equipment that are attached to it."
    );
    ruling!(
        "Militant Inquisitor",
        "Militant Inquisitor's ability counts all Equipment you control, regardless of whether they're attached to a creature."
    );
    supported("Militant Inquisitor");
    let mut t = TestGame::new(2);
    let mi = t.battlefield(P0, "Militant Inquisitor");
    let base = t.pt(mi);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Short Sword", mi);
    attach_new(&mut t, P0, "Bonesplitter", bears);
    t.battlefield(P0, "Leonin Scimitar");
    // P1's Equipment doesn't count.
    t.battlefield(P1, "Short Sword");
    t.settle();
    // +3/+0 for three Equipment, +1/+1 from the Short Sword.
    assert_eq!(t.pt(mi), (base.0 + 4, base.1 + 1));
}

#[test]
fn blacksmiths_talent_equipped_doesnt_need_your_equipment() {
    cr!("301.5a", "716.2a");
    ruling!(
        "Blacksmith's Talent",
        "A creature you control is equipped if there's an Equipment attached to it. You don't have to control that Equipment."
    );
    supported("Blacksmith's Talent");
    let mut t = TestGame::new(2);
    let talent = t.battlefield(P0, "Blacksmith's Talent");
    mtg_engine::classes::set_level(&mut t.g, talent, 3);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let other = t.battlefield(P0, "Hill Giant");
    let splitter = t.battlefield(P1, "Bonesplitter");
    assert!(t.g.attach(splitter, Entity::Object(bears)));
    t.settle();
    assert!(has_kw(&t, bears, KeywordKind::DoubleStrike));
    assert!(has_kw(&t, bears, KeywordKind::Haste));
    assert!(!has_kw(&t, other, KeywordKind::DoubleStrike));
}

#[test]
fn blacksmiths_talent_level_two_with_illegal_targets() {
    cr!("608.2b", "701.3b");
    ruling!(
        "Blacksmith's Talent",
        "If either target of the Level 2 class ability is an illegal target as the ability resolves, the ability won't do anything. If both targets are illegal, the ability won't resolve. If the Equipment is already attached to the target creature, nothing happens."
    );
    let setup = || {
        let mut t = TestGame::new(2);
        let talent = t.battlefield(P0, "Blacksmith's Talent");
        mtg_engine::classes::set_level(&mut t.g, talent, 2);
        let thopter = t.battlefield(P0, "Ornithopter");
        let bears = t.battlefield(P0, "Grizzly Bears");
        let splitter = attach_new(&mut t, P0, "Bonesplitter", thopter);
        (t, thopter, bears, splitter)
    };
    // The creature becomes illegal.
    let (mut t, thopter, bears, splitter) = setup();
    t.answer_targets(P0, &[Entity::Object(splitter)]);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.set_step(P0, Step::PrecombatMain);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "attach target Equipment"), 1);
    move_to(&mut t, bears, Zone::Exile);
    t.resolve_all();
    assert_eq!(attached_to(&t, splitter), Some(Entity::Object(thopter)));
    // Already attached to the target: it stays.
    let (mut t, thopter, _, splitter) = setup();
    t.answer_targets(P0, &[Entity::Object(splitter)]);
    t.answer_targets(P0, &[Entity::Object(thopter)]);
    t.set_step(P0, Step::PrecombatMain);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.settle();
    let ts = t.obj_now(splitter).timestamp;
    t.resolve_all();
    assert_eq!(attached_to(&t, splitter), Some(Entity::Object(thopter)));
    assert_eq!(t.obj_now(splitter).timestamp, ts);
    // Both legal: it moves.
    let (mut t, _, bears, splitter) = setup();
    t.answer_targets(P0, &[Entity::Object(splitter)]);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.set_step(P0, Step::PrecombatMain);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.settle();
    t.resolve_all();
    assert_eq!(attached_to(&t, splitter), Some(Entity::Object(bears)));
}

// ---------------------------------------------------------------------------------------
// Equip cost reductions
// ---------------------------------------------------------------------------------------

#[test]
fn cloud_reduces_only_generic_equip_costs_targeting_it() {
    cr!("601.2f", "602.2b", "702.6a");
    ruling!(
        "Cloud, Planet's Champion",
        "Cloud's last ability reduces only the amount of generic mana you pay for equip abilities that target Cloud. For example, it will reduce {2} to {0}, but it will have no effect on an equip cost of {G}."
    );
    supported("Cloud, Planet's Champion");
    let mut t = TestGame::new(2);
    let cloud = t.battlefield(P0, "Cloud, Planet's Champion");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let splitter = t.battlefield(P0, "Bonesplitter");
    let sword = t.battlefield(P0, "Greatsword of Tyr");
    // Equip {1} targeting the Bears: not reduced.
    assert!(equip(&mut t, P0, splitter, bears).is_err());
    t.clear_answers();
    // Equip {W} targeting Cloud: not reduced.
    assert!(equip(&mut t, P0, sword, cloud).is_err());
    t.clear_answers();
    // Equip {1} targeting Cloud: free.
    equip(&mut t, P0, splitter, cloud).unwrap();
    t.resolve_all();
    assert_eq!(attached_to(&t, splitter), Some(Entity::Object(cloud)));
}

#[test]
fn fervent_champion_makes_generic_equip_costs_free() {
    cr!("601.2f", "602.2b", "702.6a");
    ruling!(
        "Fervent Champion",
        "Fervent Champion's last ability reduces only the generic mana in equip abilities. If those costs are only {3}, {2}, or {1}, that equip ability will be free to activate if it targets Fervent Champion."
    );
    supported("Fervent Champion");
    supported("Loxodon Warhammer");
    let mut t = TestGame::new(2);
    let champ = t.battlefield(P0, "Fervent Champion");
    let hammer = t.battlefield(P0, "Loxodon Warhammer");
    let sword = t.battlefield(P0, "Greatsword of Tyr");
    assert!(equip(&mut t, P0, sword, champ).is_err());
    t.clear_answers();
    equip(&mut t, P0, hammer, champ).unwrap();
    t.resolve_all();
    assert_eq!(attached_to(&t, hammer), Some(Entity::Object(champ)));
}

// ---------------------------------------------------------------------------------------
// Fighter Class
// ---------------------------------------------------------------------------------------

#[test]
fn fighter_class_target_that_cant_block_doesnt() {
    cr!("509.1c", "716.2a");
    ruling!(
        "Fighter Class",
        "For the last triggered ability, if the target creature is tapped or is affected by a spell or ability that says it can't block, then it doesn't block."
    );
    supported("Fighter Class");
    supported("Hulking Goblin");
    for tapped in [true, false] {
        let mut t = TestGame::new(2);
        let class = t.battlefield(P0, "Fighter Class");
        mtg_engine::classes::set_level(&mut t.g, class, 3);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let target = if tapped {
            let ogre = t.battlefield(P1, "Gray Ogre");
            t.g.tap(ogre);
            ogre
        } else {
            t.battlefield(P1, "Hulking Goblin")
        };
        to_beginning_of_combat(&mut t, P0);
        t.answer_targets(P0, &[Entity::Object(target)]);
        attack_with(&mut t, &at_p1(&[bears]));
        t.resolve_all();
        assert!(legal_blocks(&mut t, P1, &[]));
        assert!(!legal_blocks(&mut t, P1, &[(target, bears)]));
    }
}

// ---------------------------------------------------------------------------------------
// Warchanter Skald
// ---------------------------------------------------------------------------------------

fn dwarves(t: &TestGame) -> usize {
    t.named_on_battlefield("Dwarf Berserker Token").len()
}

#[test]
fn warchanter_skald_must_become_tapped() {
    cr!("603.2e", "701.26a");
    ruling!(
        "Warchanter Skald",
        "For the triggered ability to trigger, Warchanter Skald has to actually change from untapped to tapped. If an effect attempts to tap while it is already tapped, the ability won’t trigger."
    );
    supported("Warchanter Skald");
    let mut t = TestGame::new(2);
    let skald = t.battlefield(P0, "Warchanter Skald");
    attach_new(&mut t, P0, "Short Sword", skald);
    t.g.tap(skald);
    t.g.flush_events();
    t.settle();
    t.resolve_all();
    assert_eq!(dwarves(&t), 1);
    t.g.tap(skald);
    t.g.flush_events();
    t.settle();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(dwarves(&t), 1);
}

#[test]
fn warchanter_skald_checks_equipped_twice() {
    cr!("603.4");
    ruling!(
        "Warchanter Skald",
        "If Warchanter Skald isn’t enchanted or equipped at the moment it becomes tapped, the ability won’t trigger at all. If the ability does trigger, it will check again as it tries to resolve to make sure Warchanter Skald is still enchanted or equipped: if not, the ability won’t do anything. (Warchanter Skald could have different Auras and/or Equipment attached to it by that point, though.)"
    );
    let tap = |t: &mut TestGame, id: ObjectId| {
        t.g.tap(id);
        t.g.flush_events();
        t.settle();
    };
    // Not equipped: no trigger.
    let mut t = TestGame::new(2);
    let skald = t.battlefield(P0, "Warchanter Skald");
    tap(&mut t, skald);
    assert_eq!(t.stack_len(), 0);
    // Unequipped before it resolves: nothing.
    let mut t = TestGame::new(2);
    let skald = t.battlefield(P0, "Warchanter Skald");
    let sword = attach_new(&mut t, P0, "Short Sword", skald);
    tap(&mut t, skald);
    assert_eq!(t.stack_len(), 1);
    move_to(&mut t, sword, Zone::Exile);
    t.resolve_all();
    assert_eq!(dwarves(&t), 0);
    // A different Aura by then: a token.
    let mut t = TestGame::new(2);
    let skald = t.battlefield(P0, "Warchanter Skald");
    let sword = attach_new(&mut t, P0, "Short Sword", skald);
    tap(&mut t, skald);
    move_to(&mut t, sword, Zone::Exile);
    attach_new(&mut t, P0, "Holy Strength", skald);
    t.resolve_all();
    assert_eq!(dwarves(&t), 1);
}

// ---------------------------------------------------------------------------------------
// Leaving the battlefield before an ability resolves
// ---------------------------------------------------------------------------------------

#[test]
fn auriok_survivors_gone_returns_the_equipment_unattached() {
    cr!("608.2b", "301.5c");
    ruling!(
        "Auriok Survivors",
        "If Auriok Survivors is no longer on the battlefield when its \"enters\" ability resolves, the Equipment will return to the battlefield and remain unattached."
    );
    supported("Auriok Survivors");
    let mut t = TestGame::new(2);
    let splitter = t.graveyard(P0, "Bonesplitter");
    t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(splitter)]);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    let survivors = t.enter(P0, "Auriok Survivors");
    t.settle();
    move_to(&mut t, survivors, Zone::Exile);
    t.resolve_all();
    let s = t.g.current(splitter);
    assert!(t.on_battlefield(s));
    assert_eq!(attached_to(&t, s), None);
}

#[test]
fn lunarch_inquisitors_gone_exiles_nothing() {
    cr!("610.3c", "608.2b");
    ruling!(
        "Avacynian Missionaries // Lunarch Inquisitors",
        "If Lunarch Inquisitors leaves the battlefield before its triggered ability resolves, the target creature won't be exiled."
    );
    supported("Avacynian Missionaries // Lunarch Inquisitors");
    let mut t = TestGame::new(2);
    let missionaries = t.battlefield(P0, "Avacynian Missionaries // Lunarch Inquisitors");
    attach_new(&mut t, P0, "Short Sword", missionaries);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::End);
    t.settle();
    t.resolve();
    t.settle();
    assert_eq!(triggers_on_stack(&t, "exile another target creature"), 1);
    move_to(&mut t, missionaries, Zone::Exile);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
}

#[test]
fn stonehewer_giant_with_no_creature_leaves_the_equipment_unattached() {
    cr!("701.3b", "301.5c");
    ruling!(
        "Stonehewer Giant",
        "If there is no legal creature for you to attach the Equipment to, it remains on the battlefield unattached."
    );
    supported("Stonehewer Giant");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Stonehewer Giant");
    t.library_top(P0, "Bonesplitter");
    t.lands(P0, "Plains", 2);
    activate_containing(&mut t, P0, giant, "Search").unwrap();
    move_to(&mut t, giant, Zone::Exile);
    t.resolve_all();
    let s = t.named_on_battlefield("Bonesplitter");
    assert_eq!(s.len(), 1);
    assert_eq!(attached_to(&t, s[0]), None);
}

// ---------------------------------------------------------------------------------------
// Combat
// ---------------------------------------------------------------------------------------

#[test]
fn training_drone_keeps_attacking_when_unequipped() {
    cr!("506.4", "508.1c");
    ruling!(
        "Training Drone",
        "If Training Drone has been declared as an attacking or blocking creature, and it stops being equipped later in combat, it won’t stop attacking or blocking."
    );
    supported("Training Drone");
    let mut t = TestGame::new(2);
    let drone = t.battlefield(P0, "Training Drone");
    let sword = attach_new(&mut t, P0, "Short Sword", drone);
    t.set_step(P0, Step::PrecombatMain);
    attack_with(&mut t, &at_p1(&[drone]));
    assert!(t.g.is_attacking(drone));
    move_to(&mut t, sword, Zone::Exile);
    assert!(t.g.is_attacking(t.g.current(drone)));
    block_and_finish(&mut t, P1, &[]);
    assert!(t.life(P1) < 20);
}

#[test]
fn sunspear_shikari_unequipped_after_first_strike_deals_no_regular_damage() {
    cr!("702.7c", "510.4");
    ruling!(
        "Sunspear Shikari",
        "If all Equipment attached to Sunspear Shikari somehow becomes unequipped after it deals combat damage in the first combat damage step, it won't assign combat damage in the second combat damage step."
    );
    supported("Sunspear Shikari");
    let mut t = TestGame::new(2);
    let shikari = t.battlefield(P0, "Sunspear Shikari");
    let base = t.pt(shikari);
    let sword = attach_new(&mut t, P0, "Short Sword", shikari);
    t.set_step(P0, Step::PrecombatMain);
    attack_with(&mut t, &at_p1(&[shikari]));
    t.answer(
        P1,
        DecisionKind::Blockers,
        mtg_engine::decision::Answer::Blockers(vec![]),
    );
    after_first_strike(&mut t);
    assert_eq!(t.life(P1), 20 - (base.0 + 1));
    move_to(&mut t, sword, Zone::Exile);
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20 - (base.0 + 1));
}

#[test]
fn nazahn_defending_player_of_a_planeswalker_attacker() {
    cr!("506.2", "508.5");
    ruling!(
        "Nazahn, Revered Bladesmith",
        "If a creature is attacking a planeswalker, its controller is that creature's defending player."
    );
    supported("Nazahn, Revered Bladesmith");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Nazahn, Revered Bladesmith");
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Short Sword", bears);
    let jace = t.battlefield(P1, "Jace Beleren");
    let giant = t.battlefield(P1, "Hill Giant");
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(giant)]);
    t.set_step(P0, Step::PrecombatMain);
    attack_with(&mut t, &[(bears, Entity::Object(jace))]);
    t.resolve_all();
    assert!(target_candidates(&t, P0, from)
        .concat()
        .contains(&Entity::Object(giant)));
    assert!(t.obj_now(giant).tapped);
}

#[test]
fn nazahn_triggers_once_per_equipped_attacker() {
    cr!("603.2", "508.1m");
    ruling!(
        "Nazahn, Revered Bladesmith",
        "If more than one equipped creature you control attacks, Nazahn's last ability triggers once for each of those creatures. It doesn't trigger additional times for additional Equipment attached to an attacking creature."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Nazahn, Revered Bladesmith");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let elves = t.battlefield(P0, "Llanowar Elves");
    attach_new(&mut t, P0, "Short Sword", bears);
    attach_new(&mut t, P0, "Bonesplitter", bears);
    attach_new(&mut t, P0, "Short Sword", giant);
    t.battlefield(P1, "Gray Ogre");
    t.set_step(P0, Step::PrecombatMain);
    attack_with(&mut t, &at_p1(&[bears, giant, elves]));
    assert_eq!(triggers_on_stack(&t, "tap target creature"), 2);
}

#[test]
fn nazahn_targets_only_the_attacked_players_creatures() {
    cr!("506.2", "802.2", "115.1");
    ruling!(
        "Nazahn, Revered Bladesmith",
        "Nazahn's last ability can target a creature controlled only by the defending player who is being attacked by the attacking creature that caused the ability to trigger."
    );
    let mut t = TestGame::new(3);
    let nazahn = t.battlefield(P0, "Nazahn, Revered Bladesmith");
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Short Sword", bears);
    let a = t.battlefield(P1, "Hill Giant");
    let b = t.battlefield(P2, "Gray Ogre");
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(b)]);
    t.set_step(P0, Step::PrecombatMain);
    attack_with(
        &mut t,
        &[(nazahn, Entity::Player(P1)), (bears, Entity::Player(P2))],
    );
    t.resolve_all();
    let offered = target_candidates(&t, P0, from).concat();
    assert!(offered.contains(&Entity::Object(b)));
    assert!(!offered.contains(&Entity::Object(a)));
}

#[test]
fn reyav_triggers_once_for_an_enchanted_and_equipped_attacker() {
    cr!("603.2", "702.4a");
    ruling!(
        "Reyav, Master Smith",
        "If a creature you control that's enchanted and equipped attacks, Reyav's ability will trigger only once for that creature."
    );
    supported("Reyav, Master Smith");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Reyav, Master Smith");
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Short Sword", bears);
    attach_new(&mut t, P0, "Holy Strength", bears);
    t.set_step(P0, Step::PrecombatMain);
    attack_with(&mut t, &at_p1(&[bears]));
    assert_eq!(triggers_on_stack(&t, "double strike"), 1);
    t.resolve_all();
    assert!(has_kw(&t, bears, KeywordKind::DoubleStrike));
}

#[test]
fn lunarch_inquisitors_exiled_card_returns_when_its_owner_leaves() {
    cr!("800.4a", "610.3a");
    ruling!(
        "Avacynian Missionaries // Lunarch Inquisitors",
        "In a multiplayer game, if Lunarch Inquisitors's owner leaves the game, the exiled card will return to the battlefield. Because the one-shot effect that returns the card isn't an ability that goes on the stack, it won't cease to exist along with the leaving player's spells and abilities on the stack."
    );
    let mut t = TestGame::new(3);
    let missionaries = t.battlefield(P0, "Avacynian Missionaries // Lunarch Inquisitors");
    attach_new(&mut t, P0, "Short Sword", missionaries);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::End);
    t.settle();
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert!(t.in_exile("Grizzly Bears"));
    t.g.perform_action(P0, mtg_engine::decision::Action::Concede)
        .expect("concede");
    t.settle();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}
