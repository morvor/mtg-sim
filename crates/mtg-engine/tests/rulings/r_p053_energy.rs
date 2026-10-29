//! Rulings batch P053 — energy generators: energy as a player resource (CR 107.14, 122.1),
//! "may pay {E}" and targets (CR 608.2b), "if you do" and "when you do" (CR 603.12),
//! energy replacement effects (CR 616.1), Deadlock Trap, Demon of Dark Schemes, Peema
//! Aether-Seer and other energy cards.

use crate::r_p053_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

#[test]
fn aetherstorm_roc_entering_with_another_creature_and_its_attack_trigger() {
    cr!("603.6a", "608.2b", "509.1a");
    ruling!(
        "Aetherstorm Roc",
        "A creature entering the battlefield at the same time as Aetherstorm Roc causes its first triggered ability to trigger."
    );
    ruling!(
        "Aetherstorm Roc",
        "Aetherstorm Roc's second triggered ability resolves before blockers are chosen. A creature tapped this way won't be able to block."
    );
    ruling!(
        "Aetherstorm Roc",
        "You don't have to target a creature with Aetherstorm Roc's last ability. If you do choose a target, and that target becomes illegal, the ability doesn't resolve and you won't be able to pay {E}{E} to put a counter on Aetherstorm Roc."
    );
    supported("Aetherstorm Roc");
    let mut t = TestGame::new(2);
    let ids = enter_together(&mut t, &[(P0, "Aetherstorm Roc"), (P0, "Grizzly Bears")]);
    t.resolve_all();
    assert_eq!(energy(&t, P0), 2);
    assert_eq!(ids.len(), 2);
    // Attack, pay {E}{E}: the target is tapped before blockers and can't block.
    let mut t = TestGame::new(2);
    let roc = t.battlefield(P0, "Aetherstorm Roc");
    give_energy(&mut t, P0, 2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    t.answer_yes(P0, true);
    attack_with(&mut t, &[(roc, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(energy(&t, P0), 0);
    assert_eq!(t.counters(roc, counters::PLUS1), 1);
    assert!(t.obj_now(bears).tapped);
    assert!(!legal_blocks(&mut t, P1, &[(bears, roc)]));
    // No target: the payment still puts the counter on it.
    let mut t = TestGame::new(2);
    let roc = t.battlefield(P0, "Aetherstorm Roc");
    give_energy(&mut t, P0, 2);
    t.answer_targets(P0, &[]);
    t.answer_yes(P0, true);
    attack_with(&mut t, &[(roc, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(energy(&t, P0), 0);
    assert_eq!(t.counters(roc, counters::PLUS1), 1);
    // The target becomes illegal: the ability doesn't resolve; no payment, no counter.
    let mut t = TestGame::new(2);
    let roc = t.battlefield(P0, "Aetherstorm Roc");
    give_energy(&mut t, P0, 2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    t.answer_yes(P0, true);
    attack_with(&mut t, &[(roc, Entity::Player(P1))]);
    assert_eq!(t.stack_len(), 1);
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(energy(&t, P0), 2);
    assert_eq!(t.counters(roc, counters::PLUS1), 0);
}

#[test]
fn energy_spells_whose_targets_are_illegal_dont_resolve() {
    cr!("608.2b", "107.14");
    for name in ["Harnessed Lightning", "Aether Chaser", "Die Young", "Aether Meltdown"] {
        supported(name);
    }
    ruling!(
        "Harnessed Lightning",
        "If a spell or ability with one or more targets states that you \"may pay\" some amount of {E}, and each permanent that it targets has become an illegal target, the spell or ability won't resolve. You can't pay any {E} even if you want to."
    );
    ruling!(
        "Harnessed Lightning",
        "Some spells and abilities that give you {E} may require targets. If each target chosen is an illegal target as that spell or ability tries to resolve, it won't resolve. You won't get any {E}."
    );
    ruling!(
        "Die Young",
        "If the target creature becomes an illegal target, Die Young doesn't resolve and none of its effects happen. You won't get {E}{E} or be able to pay any {E}."
    );
    ruling!(
        "Aether Meltdown",
        "If the target creature or Vehicle becomes an illegal target while Aether Meltdown is on the stack, Aether Meltdown won't resolve. It won't enter the battlefield and you won't get {E}{E}."
    );
    ruling!(
        "Territorial Aetherkite",
        "Some spells and abilities which give you {E} or say you “may pay” some amount of {E} require one or more targets. If each target chosen is an illegal target as that spell or ability tries to resolve, it won’t resolve. None of its effects will happen, you won’t get any {E} from it, and you won’t get the chance to pay any {E} for its effects even if you want to."
    );
    supported("Territorial Aetherkite");
    for name in ["Harnessed Lightning", "Die Young", "Aether Meltdown"] {
        let mut t = TestGame::new(2);
        // One {E} from Territorial Aetherkite (paying 1 of its 2 to deal 1 damage).
        t.answer(P0, DecisionKind::X, Answer::Number(1));
        t.enter(P0, "Territorial Aetherkite");
        t.resolve_all();
        assert_eq!(energy(&t, P0), 1, "{name}");
        let bears = t.battlefield(P1, "Grizzly Bears");
        crate::r_s25_common::lands_for_cost(&mut t, P0, name);
        let c = t.hand(P0, name);
        let spell = t.cast(P0, c).target(obj(bears)).go();
        destroy(&mut t, bears);
        t.resolve_all();
        assert_eq!(energy(&t, P0), 1, "{name}");
        assert_eq!(t.zone(spell), Zone::Graveyard(P0), "{name}");
        assert!(!t.on_battlefield(spell), "{name}");
    }
}

#[test]
fn die_young_chooses_the_energy_as_it_resolves() {
    cr!("601.2c", "608.2c", "107.14");
    ruling!(
        "Die Young",
        "You choose the target creature as you cast Die Young. You don't choose how much {E} to pay until Die Young is resolving."
    );
    supported("Die Young");
    let mut t = TestGame::new(2);
    give_energy(&mut t, P0, 1);
    let ogre = t.battlefield(P1, "Gray Ogre");
    crate::r_s25_common::lands_for_cost(&mut t, P0, "Die Young");
    let c = t.hand(P0, "Die Young");
    let from = t.asked().len();
    t.cast(P0, c).target(obj(ogre)).go();
    assert!(
        !t.asked()[from..]
            .iter()
            .any(|(_, d)| matches!(d, Decision::ChooseNumber { .. })),
        "the amount was asked while casting"
    );
    // Pay all three {E} (one before, two from the spell).
    t.answer(P0, DecisionKind::X, Answer::Number(3));
    t.resolve_all();
    assert_eq!(energy(&t, P0), 0);
    assert_eq!(t.zone(ogre), Zone::Graveyard(P1));
}

/// P0's Deadlock Trap (untapped) with `e` energy.
fn trap(t: &mut TestGame, e: u32) -> ObjectId {
    supported("Deadlock Trap");
    let trap = t.battlefield(P0, "Deadlock Trap");
    t.g.untap(trap);
    give_energy(t, P0, e);
    trap
}

#[test]
fn deadlock_trap_tapped_planeswalker_can_still_be_attacked_and_damaged() {
    cr!("508.1b");
    ruling!(
        "Deadlock Trap",
        "A tapped planeswalker can be attacked or dealt damage as normal."
    );
    let mut t = TestGame::new(2);
    let trap = trap(&mut t, 1);
    let pw = t.battlefield(P1, "Ajani Goldmane");
    let bears = t.battlefield(P0, "Grizzly Bears");
    activate_containing(&mut t, P0, trap, "Tap target").unwrap();
    t.answer_targets(P0, &[obj(pw)]);
    t.resolve_all();
    assert!(t.obj_now(pw).tapped);
    let loyalty = t.counters(pw, counters::LOYALTY);
    attack_with(&mut t, &[(bears, obj(pw))]);
    assert!(attacking(&t, bears));
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.counters(pw, counters::LOYALTY), loyalty - 2);
}

#[test]
fn deadlock_trap_untapped_creature_can_fight_but_not_activate() {
    cr!("602.5");
    ruling!(
        "Deadlock Trap",
        "If the tapped creature somehow becomes untapped, it can attack and block as normal, but its abilities still can't be activated this turn."
    );
    let mut t = TestGame::new(2);
    let trap = trap(&mut t, 1);
    let sorcerer = t.battlefield(P1, "Prodigal Sorcerer");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[obj(sorcerer)]);
    activate_containing(&mut t, P0, trap, "Tap target").unwrap();
    t.resolve_all();
    assert!(t.obj_now(sorcerer).tapped);
    t.g.untap(sorcerer);
    assert!(!can_activate(&mut t, P1, sorcerer));
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    assert!(legal_blocks(&mut t, P1, &[(sorcerer, bears)]));
    assert!(!can_activate(&mut t, P1, sorcerer));
}

#[test]
fn deadlock_trap_and_abilities_already_activated_or_activated_in_response() {
    cr!("602.5");
    ruling!(
        "Deadlock Trap",
        "Once a player has announced an ability, Deadlock Trap can't be used to undo it. The last ability must be activated before that player activates that ability."
    );
    ruling!(
        "Deadlock Trap",
        "Players may respond to Deadlock Trap's last ability by activating an ability of the target permanent if that ability's timing permissions allow it."
    );
    // The Sorcerer's ability is already on the stack: tapping it doesn't undo it.
    let mut t = TestGame::new(2);
    let tr = trap(&mut t, 1);
    let sorcerer = t.battlefield(P1, "Prodigal Sorcerer");
    t.activate(P1, sorcerer, 0, &[Entity::Player(P0)]).unwrap();
    t.answer_targets(P0, &[obj(sorcerer)]);
    activate_containing(&mut t, P0, tr, "Tap target").unwrap();
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(t.life(P0), 19);
    // In response to the Trap's ability, P1 activates the Sorcerer's ability.
    let mut t = TestGame::new(2);
    let trap = trap(&mut t, 1);
    let sorcerer = t.battlefield(P1, "Prodigal Sorcerer");
    t.answer_targets(P0, &[obj(sorcerer)]);
    activate_containing(&mut t, P0, trap, "Tap target").unwrap();
    assert!(can_activate(&mut t, P1, sorcerer));
    t.activate(P1, sorcerer, 0, &[Entity::Player(P0)]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P0), 19);
    assert!(t.obj_now(sorcerer).tapped);
}

#[test]
fn deadlock_trap_active_player_gets_priority_after_a_planeswalker_enters() {
    cr!("117.3b");
    ruling!(
        "Deadlock Trap",
        "After a planeswalker enters the battlefield, the active player receives priority and may activate an ability of that planeswalker before any player can activate Deadlock Trap's last ability."
    );
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    supported("Deadlock Trap");
    let trap1 = t.battlefield(P0, "Deadlock Trap");
    t.g.untap(trap1);
    give_energy(&mut t, P0, 1);
    t.lands(P1, "Plains", 4);
    let ajani = t.hand(P1, "Ajani Goldmane");
    t.cast(P1, ajani).go();
    let pw_enters = |g: &mtg_engine::game::Game| {
        g.stack.is_empty()
            && g.turn.priority.is_some()
            && g.permanents().any(|o| o.chars.name == "Ajani Goldmane")
    };
    assert!(t.g.run_until(1000, pw_enters));
    assert_eq!(t.g.turn.priority, Some(P1));
    let pw = t.named_on_battlefield("Ajani Goldmane")[0];
    assert!(can_activate(&mut t, P1, pw));
}

#[test]
fn demon_of_dark_schemes_energy_from_deaths_and_its_minus_two() {
    cr!("603.10a", "611.2c");
    ruling!(
        "Demon of Dark Schemes",
        "Demon of Dark Schemes's second triggered ability triggers whenever another creature dies for any reason, not just due to the first triggered ability."
    );
    ruling!(
        "Demon of Dark Schemes",
        "If Demon of Dark Schemes dies at the same time as another creature, its second triggered ability triggers."
    );
    ruling!(
        "Demon of Dark Schemes",
        "The set of creatures affected by Demon of Dark Schemes's first triggered ability is determined as the ability resolves. Creatures that enter the battlefield later in the turn and noncreature permanents that become creatures later in the turn won't get -2/-2."
    );
    supported("Demon of Dark Schemes");
    // Another creature dies to a Shock.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Demon of Dark Schemes");
    let bears = t.battlefield(P1, "Grizzly Bears");
    damage(&mut t, bears, 2, bears);
    t.resolve_all();
    assert_eq!(energy(&t, P0), 1);
    // The Demon and another creature die at the same time.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Demon of Dark Schemes");
    t.battlefield(P1, "Gray Ogre");
    t.lands(P0, "Plains", 4);
    let wrath = t.hand(P0, "Wrath of God");
    t.cast(P0, wrath).go();
    t.resolve_all();
    assert!(t.named_on_battlefield("Demon of Dark Schemes").is_empty());
    assert_eq!(energy(&t, P0), 1);
    // -2/-2 for the creatures there as it resolves (including a Gray Ogre that entered
    // after it triggered: it and the Bears die); a creature entering later isn't affected.
    let mut t = TestGame::new(2);
    let ogre = t.battlefield(P1, "Hill Giant");
    t.battlefield(P1, "Grizzly Bears");
    let demon = t.enter(P0, "Demon of Dark Schemes");
    t.settle();
    let later_before = t.battlefield(P1, "Gray Ogre");
    t.resolve_all();
    assert_eq!(t.pt(ogre), (1, 1));
    assert!(!t.on_battlefield(later_before));
    assert_eq!(energy(&t, P0), 2);
    assert_eq!(t.pt(demon), (5, 5));
    let later = t.battlefield(P1, "Gray Ogre");
    assert_eq!(t.pt(later), (2, 2));
}

#[test]
fn demon_of_dark_schemes_creature_leaves_with_its_owner_or_is_exiled() {
    cr!("800.4a");
    ruling!(
        "Demon of Dark Schemes",
        "If a creature's owner leaves the game after you've put that creature onto the battlefield with Demon of Dark Schemes's last ability, the creature leaves with that player. If you leave the game before that player, the creature is exiled."
    );
    let setup = || {
        let mut t = TestGame::new(3);
        let demon = t.battlefield(P0, "Demon of Dark Schemes");
        give_energy(&mut t, P0, 4);
        t.lands(P0, "Swamp", 3);
        let bears = t.graveyard(P1, "Grizzly Bears");
        t.answer_targets(P0, &[obj(bears)]);
        activate_containing(&mut t, P0, demon, "Put target creature card").unwrap();
        t.resolve_all();
        let b = t.named_on_battlefield("Grizzly Bears");
        assert_eq!(b.len(), 1);
        assert_eq!(t.obj_now(b[0]).controller, P0);
        assert!(t.obj_now(b[0]).tapped);
        (t, b[0])
    };
    let (mut t, bears) = setup();
    concede(&mut t, P1);
    assert!(!t.on_battlefield(bears));
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
    let (mut t, bears) = setup();
    concede(&mut t, P0);
    assert_eq!(t.zone(bears), Zone::Exile);
}

#[test]
fn aether_refineries_multiply_and_their_controller_orders_energy_replacements() {
    cr!("616.1", "616.1e", "107.14");
    ruling!(
        "Aether Refinery",
        "If you control two Aether Refineries, the amount of {E} you get is four times the original number."
    );
    ruling!(
        "Aether Refinery",
        "If two or more effects attempt to modify how many {E} you get (or how many counters you would get), you choose the order to apply those effects, no matter who controls the sources of those effects."
    );
    supported("Aether Refinery");
    supported("Vorinclex, Monstrous Raider");
    let mut t = TestGame::new(2);
    let r = t.battlefield(P0, "Aether Refinery");
    t.battlefield(P0, "Aether Refinery");
    t.answer(P0, DecisionKind::X, Answer::Number(0));
    t.activate(P0, r, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(energy(&t, P0), 4);
    assert!(crate::r_s01_common::tokens(&t, P0).is_empty());
    // An opponent's Vorinclex halves it; P0 (the player getting {E}) orders the effects:
    // doubling first gives 1, halving first gives 0.
    fn half_first(g: &mtg_engine::game::Game, d: &Decision) -> Option<Answer> {
        crate::r_s30_common::pick_replacement(g, d, "half")
    }
    type Responder = crate::r_s29_common::Responder;
    for (refinery_first, order, expect) in [
        (true, crate::r_s29_common::twice_first as Responder, 1),
        (false, half_first, 0),
    ] {
        let mut t = TestGame::new(2);
        let r = t.battlefield(P0, "Aether Refinery");
        t.battlefield(P1, "Vorinclex, Monstrous Raider");
        crate::r_s03_common::respond(&mut t, P0, order);
        t.answer(P0, DecisionKind::X, Answer::Number(0));
        let from = t.asked().len();
        t.activate(P0, r, 0, &[]).unwrap();
        t.resolve_all();
        assert_eq!(energy(&t, P0), expect, "refinery first: {refinery_first}");
        assert_eq!(
            crate::r_s29_common::replacement_choosers(&t, from),
            vec![P0]
        );
    }
}

#[test]
fn fabrication_module_triggers_once_per_energy_event() {
    cr!("603.2");
    ruling!(
        "Fabrication Module",
        "If you get multiple {E} at once, you only put one +1/+1 counter on a creature, not one counter for each {E}. If you get multiple {E} at different times, Fabrication Module's ability triggers once each time."
    );
    supported("Fabrication Module");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Fabrication Module");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    // Aether Chaser: "When this creature enters, you get {E}{E}".
    t.answer_targets(P0, &[obj(bears)]);
    t.enter(P0, "Aether Chaser");
    t.resolve_all();
    assert_eq!(energy(&t, P0), 2);
    assert_eq!(t.counters(bears, counters::PLUS1), 1);
    t.answer_targets(P0, &[obj(bears)]);
    t.enter(P0, "Aether Chaser");
    t.resolve_all();
    assert_eq!(energy(&t, P0), 4);
    assert_eq!(t.counters(bears, counters::PLUS1), 2);
}

#[test]
fn peema_aether_seer_energy_from_greatest_power_as_it_resolves() {
    cr!("608.2h", "107.1b");
    ruling!(
        "Peema Aether-Seer",
        "The amount of {E} you get is determined as Peema Aether-Seer's first ability resolves. If the greatest power among creatures you control is somehow negative, you don't get or lose any {E}."
    );
    supported("Peema Aether-Seer");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.enter(P0, "Peema Aether-Seer");
    t.settle();
    crate::r_p021_common::giant_growth(&mut t, P0, bears);
    t.resolve_all();
    assert_eq!(energy(&t, P0), 5);
    // Its power is negative (-4/-0 until end of turn): no energy gained or lost.
    let mut t = TestGame::new(2);
    give_energy(&mut t, P0, 2);
    let peema = t.enter(P0, "Peema Aether-Seer");
    t.settle();
    crate::r_s26_common::modify_until_eot(
        &mut t,
        peema,
        vec![mtg_engine::ability::Modification::ModifyPT(
            mtg_engine::ability::Value::c(-4),
            mtg_engine::ability::Value::c(0),
        )],
    );
    assert_eq!(t.pt(peema).0, -1);
    t.resolve_all();
    assert_eq!(energy(&t, P0), 2);
}

#[test]
fn peema_aether_seer_target_blocks_if_able_and_its_controller_picks_the_attacker() {
    cr!("509.1c");
    ruling!(
        "Peema Aether-Seer",
        "The controller of the target creature chooses which attacking creature that creature blocks."
    );
    ruling!(
        "Peema Aether-Seer",
        "The target creature blocks only if it's able to do so as the declare blockers step begins. If, at that time, the creature is tapped, it's affected by a spell or ability that says it can't block"
    );
    let setup = || {
        let mut t = TestGame::new(2);
        let peema = t.battlefield(P0, "Peema Aether-Seer");
        let a = t.battlefield(P0, "Grizzly Bears");
        let b = t.battlefield(P0, "Gray Ogre");
        let wall = t.battlefield(P1, "Hill Giant");
        give_energy(&mut t, P0, 3);
        t.answer_targets(P0, &[obj(wall)]);
        activate_containing(&mut t, P0, peema, "blocks this turn").unwrap();
        t.resolve_all();
        (t, a, b, wall)
    };
    let (mut t, a, b, wall) = setup();
    attack_with(&mut t, &[(a, Entity::Player(P1)), (b, Entity::Player(P1))]);
    assert!(!legal_blocks(&mut t, P1, &[]));
    assert!(legal_blocks(&mut t, P1, &[(wall, a)]));
    assert!(legal_blocks(&mut t, P1, &[(wall, b)]));
    // Tapped as blockers are declared: it doesn't block.
    let (mut t, a, b, wall) = setup();
    attack_with(&mut t, &[(a, Entity::Player(P1)), (b, Entity::Player(P1))]);
    t.g.tap(wall);
    assert!(legal_blocks(&mut t, P1, &[]));
}

#[test]
fn razorfield_ripper_x_is_locked_in_as_the_trigger_resolves() {
    cr!("608.2h");
    ruling!(
        "Razorfield Ripper",
        "The value of X is calculated only once, as Razorfield Ripper's triggered ability resolves."
    );
    supported("Razorfield Ripper");
    let mut t = TestGame::new(2);
    let ripper = t.battlefield(P0, "Razorfield Ripper");
    give_energy(&mut t, P0, 2);
    attack_with(&mut t, &[(ripper, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(energy(&t, P0), 3);
    assert_eq!(t.pt(ripper), (6, 6));
    give_energy(&mut t, P0, 5);
    t.g.recompute();
    assert_eq!(t.pt(ripper), (6, 6));
}

#[test]
fn lightning_runner_untaps_creatures_that_arent_attacking() {
    ruling!(
        "Lightning Runner",
        "You untap all creatures you control, including ones that aren't attacking."
    );
    supported("Lightning Runner");
    let mut t = TestGame::new(2);
    let runner = t.battlefield(P0, "Lightning Runner");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.tap(bears);
    give_energy(&mut t, P0, 6);
    t.answer_yes(P0, true);
    attack_with(&mut t, &[(runner, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(energy(&t, P0), 0);
    assert!(!t.obj_now(bears).tapped);
    assert!(!t.obj_now(runner).tapped);
    // And after this combat phase there's another one: the untapped Bears can attack.
    block_and_finish(&mut t, P1, &[]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.answer_yes(P0, false);
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.g.turn.active, P0);
    assert_eq!(t.life(P1), 20 - 2 * 2 - 2);
}

#[test]
fn aethergeode_miner_returns_as_a_new_object() {
    cr!("400.7", "506.4");
    ruling!(
        "Aethergeode Miner",
        "After Aethergeode Miner returns to the battlefield, it will be a new object with no connection to the creature that was exiled. It won't be in combat or have any additional abilities it may have had when it was exiled. Any +1/+1 counters on it or Auras attached to it are removed, and any Equipment will no longer be attached."
    );
    supported("Aethergeode Miner");
    let mut t = TestGame::new(2);
    let miner = t.battlefield(P0, "Aethergeode Miner");
    put_counters(&mut t, miner, counters::PLUS1, 1);
    let aura = attach_new(&mut t, P0, "Holy Strength", obj(miner));
    let axe = attach_new(&mut t, P0, "Bonesplitter", obj(miner));
    attack_with(&mut t, &[(miner, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(energy(&t, P0), 2);
    activate_containing(&mut t, P0, miner, "Exile").unwrap();
    t.resolve_all();
    let now = t.g.current(miner);
    assert_ne!(now, miner);
    assert!(t.on_battlefield(now));
    assert!(!attacking(&t, now));
    assert_eq!(t.counters(now, counters::PLUS1), 0);
    assert_eq!(t.pt(now), (3, 1));
    assert!(!t.on_battlefield(aura));
    assert!(t.on_battlefield(axe));
    assert_eq!(attached_to(&t, axe), None);
}

#[test]
fn may_pay_energy_once_if_you_do_and_when_you_do() {
    cr!("603.12", "608.2c", "107.14");
    ruling!(
        "Aether Chaser",
        "Some triggered abilities state that you \"may pay\" a certain amount of {E}. You can't pay that amount multiple times to multiply the effect. You simply choose whether or not to pay that amount of {E} as the ability resolves."
    );
    ruling!(
        "Aether Chaser",
        "Some triggered abilities that state that you \"may pay\" a certain amount of {E} describe an effect that happens \"If you do.\" In that case, no player may take actions to try to stop the ability's effect after you make your choice. If the payment is followed by the phrase \"When you do,\" then you'll choose any targets for that reflexive triggered ability and put it on the stack before players can take actions."
    );
    ruling!(
        "Territorial Aetherkite",
        "Some spells and abilities say that you “may pay” a certain amount of {E}, then describes an effect that happens “if you do.” In that case, no player may take actions to try to stop the ability’s effect after you make your choice. If the payment is instead followed by the phrase “when you do” followed by an effect, then after you choose to pay {E}, a new triggered ability called a reflexive trigger is put on the stack"
    );
    supported("Aether Chaser");
    supported("Territorial Aetherkite");
    // "If you do": paying {E}{E} once makes one Servo within the same resolution.
    let mut t = TestGame::new(2);
    let chaser = t.battlefield(P0, "Aether Chaser");
    give_energy(&mut t, P0, 6);
    t.answer_yes(P0, true);
    attack_with(&mut t, &[(chaser, Entity::Player(P1))]);
    assert_eq!(t.stack_len(), 1);
    let from = t.asked().len();
    t.resolve();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(energy(&t, P0), 4);
    assert_eq!(crate::r_s01_common::tokens(&t, P0).len(), 1);
    let pays = t.asked()[from..]
        .iter()
        .filter(|(_, d)| !matches!(d, Decision::Priority { .. }))
        .count();
    assert_eq!(pays, 1, "{:?}", asked_debug(&t, from));
    // "When you do": the reflexive trigger goes on the stack; players may respond.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    let kite = t.enter(P0, "Territorial Aetherkite");
    t.settle();
    t.resolve();
    assert_eq!(energy(&t, P0), 0);
    assert_eq!(t.stack_len(), 1);
    assert!(is_trigger(&t, top(&t)));
    assert!(t.on_battlefield(bears));
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert!(t.on_battlefield(kite));
}

#[test]
fn inventors_axe_enters_unattached_and_attaches_with_its_trigger() {
    cr!("608.2b");
    ruling!(
        "Inventor's Axe",
        "Inventor's Axe doesn't enter the battlefield attached to a creature. Instead, Inventor's Axe enters the battlefield and then its triggered ability attaches it to a creature. You may cast Inventor's Axe even if you don't control any creatures."
    );
    ruling!(
        "Inventor's Axe",
        "If the target creature becomes an illegal target, Inventor's Axe remains on the battlefield unattached."
    );
    supported("Inventor's Axe");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 1);
    let axe = t.hand(P0, "Inventor's Axe");
    t.cast(P0, axe).go();
    t.resolve_all();
    let axe = t.named_on_battlefield("Inventor's Axe")[0];
    assert_eq!(energy(&t, P0), 2);
    assert_eq!(attached_to(&t, axe), None);
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Mountain", 1);
    let axe = t.hand(P0, "Inventor's Axe");
    t.answer_targets(P0, &[obj(bears)]);
    t.cast(P0, axe).go();
    t.resolve();
    let axe = t.named_on_battlefield("Inventor's Axe")[0];
    assert_eq!(attached_to(&t, axe), None);
    assert_eq!(t.stack_len(), 2);
    destroy(&mut t, bears);
    t.resolve_all();
    assert!(t.on_battlefield(axe));
    assert_eq!(attached_to(&t, axe), None);
    assert_eq!(energy(&t, P0), 2);
}
