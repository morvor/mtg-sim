//! Item `rulings-residue-misc-2`, pass B — control: control-changing effects and their
//! durations (CR 611.2b, 613.1b), exchanges (701.12), "an opponent's choice" targets,
//! and casting or activating with another player's cards (Sen Triplets, Knowledge Pool).

use crate::r_residue_misc_2b_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::CardType;
use mtg_engine::*;

// --- Seasinger ---------------------------------------------------------------------------------

/// P0's Seasinger (and an Island) and P1's Grizzly Bears (and an Island); Seasinger's
/// ability targets the Bears and waits on the stack.
fn seasinger() -> (TestGame, ObjectId, ObjectId) {
    supported("Seasinger");
    let mut t = TestGame::new(3);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Island");
    let s = t.battlefield(P0, "Seasinger");
    t.battlefield(P1, "Island");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.activate(P0, s, 0, &[obj(bears)]).expect("Seasinger");
    (t, s, bears)
}

#[test]
fn seasinger_target_needs_an_island_on_activation_and_resolution_only() {
    cr!("115.1", "608.2b");
    ruling!(
        "Seasinger",
        "If a creature’s controller doesn’t control an Island, that creature is an illegal target. Whether that player controls an Island is checked both when Seasinger’s ability is activated and when it resolves. However, targeting restrictions aren’t checked continually while a spell or ability is on the stack."
    );
    // No Island: not a legal target.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Island");
    let s = t.battlefield(P0, "Seasinger");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let from = t.asked().len();
    t.activate(P0, s, 0, &[obj(bears)])
        .expect("it can target itself");
    let offered: Vec<Entity> = asked_since(&t, from)
        .into_iter()
        .find_map(|(_, d)| match d {
            Decision::ChooseTargets { candidates, .. } => Some(candidates),
            _ => None,
        })
        .unwrap();
    assert!(!offered.contains(&obj(bears)) && offered.contains(&obj(s)));
    // The Island leaves while the ability waits, and comes back: still legal as it resolves.
    let (mut t, _, bears) = seasinger();
    let island = t.g.permanents_controlled_by(P1)[0];
    let in_hand = mv(&mut t, island, Zone::Hand(P1));
    mv(&mut t, in_hand, Zone::Battlefield);
    t.resolve_all();
    assert_eq!(controller(&t, bears), P0);
    // Gone as it resolves: the target is illegal and nothing happens.
    let (mut t, _, bears) = seasinger();
    let island = t.g.permanents_controlled_by(P1)[0];
    mv(&mut t, island, Zone::Hand(P1));
    t.resolve_all();
    assert_eq!(controller(&t, bears), P1);
}

#[test]
fn seasinger_duration_starts_when_activated_and_ignores_the_creature_stopping_being_one() {
    cr!("611.2b", "613.1b");
    ruling!(
        "Seasinger",
        "Whether you control Seasinger is checked continually, starting when the ability is activated. If, before the ability resolves, there’s any point at which you don’t control Seasinger, the ability has no effect — even if you control Seasinger again by the time the ability resolves. The same is true regarding whether Seasinger remains tapped."
    );
    ruling!(
        "Seasinger",
        "You do not lose control of the stolen creature if it stops being a creature. You only lose control if Seasinger leaves your control or becomes untapped."
    );
    // Control of Seasinger lost and regained before the ability resolves: no effect.
    let (mut t, s, bears) = seasinger();
    give_control(&mut t, s, P2);
    give_control(&mut t, s, P0);
    t.resolve_all();
    assert_eq!(controller(&t, bears), P1, "{}", t.dump_log());
    // Untapped and tapped again: no effect.
    let (mut t, s, bears) = seasinger();
    t.g.untap(s);
    t.g.tap(s);
    t.resolve_all();
    assert_eq!(controller(&t, bears), P1);
    // A Mishra's Factory, animated, stolen, then no longer a creature: still P0's.
    supported("Mishra's Factory");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Island");
    let s = t.battlefield(P0, "Seasinger");
    t.battlefield(P1, "Island");
    let factory = t.battlefield(P1, "Mishra's Factory");
    t.lands(P1, "Wastes", 1);
    activate_containing(&mut t, P1, factory, "becomes").expect("animate");
    t.resolve_all();
    assert!(t.obj_now(factory).is(CardType::Creature));
    t.activate(P0, s, 0, &[obj(factory)]).expect("Seasinger");
    t.resolve_all();
    assert_eq!(controller(&t, factory), P0);
    t.advance_to(P1, Step::Upkeep);
    t.g.recompute();
    assert!(!t.obj_now(factory).is(CardType::Creature));
    assert_eq!(controller(&t, factory), P0, "{}", t.dump_log());
    // Seasinger untaps: P1 gets it back.
    t.g.untap(s);
    t.g.recompute();
    assert_eq!(controller(&t, factory), P1);
}

// --- Thrull Champion -------------------------------------------------------------------------------

#[test]
fn thrull_champion_pumps_itself_steals_another_champion_and_untaps_freely() {
    cr!("613.4c", "611.2b", "502.3");
    ruling!("Thrull Champion", "Gives himself +1/+1.");
    ruling!(
        "Thrull Champion",
        "Can be used to steal another Thrull Champion."
    );
    ruling!(
        "Thrull Champion",
        "Unlike many similar creatures, Thrull Champion untaps as normal during your untap step. Whether it’s tapped or untapped has no bearing on the control-change effect."
    );
    supported("Thrull Champion");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let mine = t.battlefield(P0, "Thrull Champion");
    let theirs = t.battlefield(P1, "Thrull Champion");
    // Each pumps every Thrull, itself included: 2/2 base, +1/+1 twice.
    assert_eq!(t.pt(mine), (4, 4));
    t.activate(P0, mine, 0, &[obj(theirs)]).expect("steal");
    t.resolve_all();
    assert_eq!(controller(&t, theirs), P0);
    // P0's untap step untaps it, and P0 keeps the stolen Champion.
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    assert!(!t.obj_now(mine).tapped);
    assert_eq!(controller(&t, theirs), P0);
    // Losing control of the Champion ends it.
    give_control(&mut t, mine, P1);
    assert_eq!(controller(&t, theirs), P1);
}

// --- Confusion in the Ranks -----------------------------------------------------------------------

fn confusion() -> TestGame {
    TestGame::new(2)
}

#[test]
fn confusion_in_the_ranks_triggers_on_itself_and_must_exchange_if_it_can() {
    cr!("603.6a", "701.12a", "603.3d");
    ruling!(
        "Confusion in the Ranks",
        "Confusion in the Ranks triggers on itself entering. If an opponent controls an enchantment, exchange Confusion in the Ranks for that enchantment."
    );
    ruling!(
        "Confusion in the Ranks",
        "The exchange isn't optional. If there is a legal choice, you must make one."
    );
    let mut t = confusion();
    let pac = t.battlefield(P1, "Glorious Anthem");
    // P0 answers nothing: the only legal target is chosen anyway.
    let cir = t.enter(P0, "Confusion in the Ranks");
    t.settle();
    assert_eq!(triggers_of(&t, "Confusion in the Ranks").len(), 1);
    t.resolve_all();
    assert_eq!(controller(&t, cir), P1, "{}", t.dump_log());
    assert_eq!(controller(&t, pac), P0);
}

#[test]
fn confusion_in_the_ranks_an_artifact_land_can_be_exchanged_for_a_land() {
    cr!("115.1", "701.12a");
    ruling!(
        "Confusion in the Ranks",
        "Even though Confusion in the Ranks triggers only when an artifact, creature, or enchantment enters, the ability can target a land if the permanent entering is an artifact land."
    );
    let mut t = confusion();
    t.battlefield(P0, "Confusion in the Ranks");
    let island = t.battlefield(P1, "Island");
    t.answer_targets(P0, &[obj(island)]);
    let seat = t.enter(P0, "Seat of the Synod");
    t.resolve_all();
    assert_eq!(controller(&t, island), P0, "{}", t.dump_log());
    assert_eq!(controller(&t, seat), P1);
}

#[test]
fn confusion_in_the_ranks_exchanges_only_if_both_are_there_and_still_share_a_type() {
    cr!("701.12b", "608.2b");
    ruling!(
        "Confusion in the Ranks",
        "The permanents are exchanged only if they're both on the battlefield when the ability resolves."
    );
    ruling!(
        "Confusion in the Ranks",
        "The permanents have to share a permanent type both when the target is chosen and when the ability resolves."
    );
    // The entering creature leaves: no exchange.
    let mut t = confusion();
    t.battlefield(P0, "Confusion in the Ranks");
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[obj(giant)]);
    let bears = t.enter(P0, "Grizzly Bears");
    t.settle();
    kill(&mut t, bears);
    t.resolve_all();
    assert_eq!(controller(&t, giant), P1);
    // The target stops being a creature (Mishra's Factory, animated, until it isn't): no
    // exchange.
    let mut t = confusion();
    t.battlefield(P0, "Confusion in the Ranks");
    let factory = t.battlefield(P1, "Mishra's Factory");
    t.lands(P1, "Wastes", 1);
    activate_containing(&mut t, P1, factory, "becomes").expect("animate");
    t.resolve_all();
    t.answer_targets(P0, &[obj(factory)]);
    let bears = t.enter(P0, "Grizzly Bears");
    t.settle();
    assert_eq!(triggers_of(&t, "Confusion in the Ranks").len(), 1);
    // End the animation (as the cleanup step would).
    t.g.expire_effects(|d| matches!(d, mtg_engine::ability::Duration::EndOfTurn));
    t.g.recompute();
    assert!(!t.obj_now(factory).is(CardType::Creature));
    t.resolve_all();
    assert_eq!(controller(&t, factory), P1, "{}", t.dump_log());
    assert_eq!(controller(&t, bears), P0);
}

// --- Preacher ----------------------------------------------------------------------------------

#[test]
fn preacher_target_must_stay_controlled_by_the_chosen_opponent() {
    cr!("608.2b", "115.7");
    ruling!(
        "Preacher",
        "The target of the ability is a creature the chosen opponent controls. As the ability resolves, if the target isn’t a creature controlled by the chosen opponent, the ability won’t resolve. If the ability’s target is changed somehow, the new target must be a creature controlled by the chosen opponent."
    );
    supported("Preacher");
    supported("Deflecting Swat");
    // The target changes control: the ability doesn't resolve.
    let mut t = TestGame::new(3);
    t.set_step(P0, Step::PrecombatMain);
    let preacher = t.battlefield(P0, "Preacher");
    let elf = t.battlefield(P1, "Llanowar Elves");
    t.battlefield(P2, "Hill Giant");
    t.answer_choose(P0, &[pl(P1)]);
    t.answer_targets(P1, &[obj(elf)]);
    t.activate(P0, preacher, 0, &[]).unwrap();
    give_control(&mut t, elf, P2);
    t.resolve_all();
    assert_eq!(controller(&t, elf), P2, "{}", t.dump_log());
    // A new target (Deflecting Swat): only P1's creatures are offered.
    let mut t = TestGame::new(3);
    t.set_step(P0, Step::PrecombatMain);
    let preacher = t.battlefield(P0, "Preacher");
    let elf = t.battlefield(P1, "Llanowar Elves");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P2, "Hill Giant");
    t.answer_choose(P0, &[pl(P1)]);
    t.answer_targets(P1, &[obj(elf)]);
    let ability = t.activate(P0, preacher, 0, &[]).unwrap();
    let ability = ability.or_else(|| t.g.stack.last().copied()).unwrap();
    t.lands(P2, "Mountain", 3);
    let swat = t.hand(P2, "Deflecting Swat");
    let from = t.asked().len();
    t.answer_targets(P2, &[obj(ability)]);
    t.answer_yes(P2, true);
    t.answer_targets(P2, &[obj(bears)]);
    t.cast(P2, swat).go();
    t.resolve();
    let offered: Vec<Vec<Entity>> = asked_since(&t, from)
        .into_iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseTargets { candidates, .. } if p == P2 => Some(candidates),
            _ => None,
        })
        .collect();
    let new_choice = offered.last().expect("new targets asked");
    assert!(new_choice.contains(&obj(bears)), "{new_choice:?}");
    assert!(!new_choice.contains(&obj(giant)));
    t.resolve_all();
    assert_eq!(controller(&t, bears), P0, "{}", t.dump_log());
    assert_eq!(controller(&t, elf), P1);
}

// --- Orcish Squatters, Farrel's Zealot, Mindstab Thrull ----------------------------------------

#[test]
fn orcish_squatters_triggers_at_blocks_and_needs_its_controller_to_keep_it() {
    cr!("509.1h", "611.2b");
    ruling!(
        "Orcish Squatters",
        "The ability triggers on the declaration of blockers if no blockers are declared for it."
    );
    ruling!(
        "Orcish Squatters",
        "If you lose control of Orcish Squatters before its ability resolves, you won’t gain control of the targeted land at all."
    );
    supported("Orcish Squatters");
    for lose in [false, true] {
        let mut t = TestGame::new(3);
        t.set_step(P0, Step::PrecombatMain);
        let sq = t.battlefield(P0, "Orcish Squatters");
        let land = t.battlefield(P1, "Forest");
        attack_with(&mut t, &[(sq, pl(P1))]);
        assert!(triggers_of(&t, "Orcish Squatters").is_empty());
        t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![]));
        t.answer_yes(P0, true);
        t.answer_targets(P0, &[obj(land)]);
        t.advance_to(P0, Step::DeclareBlockers);
        t.settle();
        assert_eq!(triggers_of(&t, "Orcish Squatters").len(), 1);
        if lose {
            give_control(&mut t, sq, P2);
        }
        t.resolve_all();
        let want = if lose { P1 } else { P0 };
        assert_eq!(controller(&t, land), want, "{}", t.dump_log());
    }
}

#[test]
fn farrels_zealot_targets_any_creature_even_itself() {
    cr!("603.3d", "115.1", "802.1");
    ruling!(
        "Farrel's Zealot",
        "If there are no other creatures on the battlefield, Farrel’s Zealot will have to target itself. Of course, you can choose not to use the ability when it resolves."
    );
    ruling!(
        "Farrel's Zealot",
        "In a multiplayer game, the target creature does not need to be one of the ones controlled by the player you actually attacked."
    );
    supported("Farrel's Zealot");
    // Alone: it targets itself, and P0 declines; it deals its combat damage.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let z = t.battlefield(P0, "Farrel's Zealot");
    t.answer_yes(P0, false);
    attack_with(&mut t, &[(z, pl(P1))]);
    t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![]));
    t.advance_to(P0, Step::DeclareBlockers);
    t.settle();
    let trig = triggers_of(&t, "Farrel's Zealot");
    assert_eq!(trig.len(), 1);
    let targets = t.g.obj(trig[0]).stack.as_ref().unwrap().chosen[0].targets[0].clone();
    assert_eq!(targets, vec![obj(z)]);
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.obj_now(z).damage, 0);
    assert_eq!(t.life(P1), 18);
    // Three players: attacking P1, it can hit P2's creature.
    let mut t = TestGame::new(3);
    t.set_step(P0, Step::PrecombatMain);
    let z = t.battlefield(P0, "Farrel's Zealot");
    let giant = t.battlefield(P2, "Hill Giant");
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(giant)]);
    attack_with(&mut t, &[(z, pl(P1))]);
    t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![]));
    t.advance_to(P0, Step::EndOfCombat);
    assert!(t.in_graveyard(P2, "Hill Giant"), "{}", t.dump_log());
    assert_eq!(t.life(P1), 20);
}

#[test]
fn mindstab_thrull_defending_player_chooses_the_discards() {
    cr!("701.9a");
    ruling!(
        "Mindstab Thrull",
        "The defending player gets to choose which cards they discard."
    );
    supported("Mindstab Thrull");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let m = t.battlefield(P0, "Mindstab Thrull");
    let cards: Vec<ObjectId> = ["Island", "Forest", "Mountain", "Plains"]
        .iter()
        .map(|n| t.hand(P1, n))
        .collect();
    t.answer_yes(P0, true);
    let from = t.asked().len();
    t.answer_choose(P1, &[obj(cards[0]), obj(cards[1]), obj(cards[3])]);
    attack_with(&mut t, &[(m, pl(P1))]);
    t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![]));
    t.advance_to(P0, Step::EndOfCombat);
    assert!(t.in_graveyard(P0, "Mindstab Thrull"));
    assert_eq!(t.hand_size(P1), 1);
    assert!(t.in_hand(P1, "Mountain"), "{}", t.dump_log());
    assert!(asked_since(&t, from)
        .iter()
        .any(|(p, d)| *p == P1 && matches!(d, Decision::ChooseEntities { .. })));
}

// --- Magus of the Unseen, Reins of Power, Stolen Uniform -------------------------------------

#[test]
fn magus_of_the_unseen_artifact_gains_haste_and_taps_as_it_returns() {
    cr!("603.2", "611.2a", "702.10b");
    ruling!(
        "Magus of the Unseen",
        "The artifact will gain haste whether or not it’s a creature. Of course, if it’s not a creature, haste will have no particular effect."
    );
    ruling!(
        "Magus of the Unseen",
        "The artifact taps after returning to the opponent (if it is not already tapped), so any abilities triggered off it tapping happen at that time."
    );
    supported("Magus of the Unseen");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let magus = t.battlefield(P0, "Magus of the Unseen");
    let stone = t.battlefield(P1, "Mind Stone");
    t.g.tap(stone);
    t.lands(P0, "Island", 2);
    t.activate(P0, magus, 0, &[obj(stone)]).expect("Magus");
    t.resolve_all();
    assert_eq!(controller(&t, stone), P0);
    assert!(!t.obj_now(stone).tapped);
    assert!(t.obj_now(stone).has_keyword(KeywordKind::Haste));
    // P1 regains control: the "when you lose control" trigger taps it then, under P1's
    // control.
    give_control(&mut t, stone, P1);
    assert!(!t.obj_now(stone).tapped);
    assert_eq!(t.stack_len(), 1, "{}", t.dump_log());
    t.resolve_all();
    assert_eq!(controller(&t, stone), P1);
    assert!(t.obj_now(stone).tapped);
}

#[test]
fn reins_of_power_untaps_only_the_two_players_creatures() {
    cr!("608.2c");
    ruling!(
        "Reins of Power",
        "You only untap creatures controlled by you and the targeted opponent, not all creatures."
    );
    supported("Reins of Power");
    let mut t = TestGame::new(3);
    t.lands(P0, "Island", 4);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let angel = t.battlefield(P2, "Serra Angel");
    for c in [bears, giant, angel] {
        t.g.tap(c);
    }
    let spell = t.hand(P0, "Reins of Power");
    t.cast(P0, spell).target(pl(P1)).go();
    t.resolve_all();
    assert!(!t.obj_now(bears).tapped && !t.obj_now(giant).tapped);
    assert!(t.obj_now(angel).tapped);
}

#[test]
fn stolen_uniform_with_an_illegal_creature_still_takes_the_equipment() {
    cr!("608.2b");
    ruling!(
        "Stolen Uniform",
        "If the target creature is an illegal target, you'll still gain control of the target Equipment until end of turn, but you won't attach it to the chosen creature."
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let saw = t.battlefield(P1, "Bone Saw");
    t.lands(P0, "Island", 1);
    let su = t.hand(P0, "Stolen Uniform");
    t.cast(P0, su).targets(&[obj(bears), obj(saw)]).go();
    kill(&mut t, bears);
    t.resolve_all();
    assert_eq!(controller(&t, saw), P0, "{}", t.dump_log());
    assert!(t.obj_now(saw).attached_to.is_none());
}

// --- Become the Pilot --------------------------------------------------------------------------

#[test]
fn become_the_pilot_only_planeswalkers_and_battles_can_be_attacked() {
    cr!("506.3", "508.1b", "509.1b");
    ruling!(
        "Become the Pilot",
        "Only planeswalker and battle permanents can be attacked. Other permanents, such as creatures, can't be attacked."
    );
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P1, "Grizzly Bears");
    let pilot = t.battlefield(P0, "Become the Pilot");
    t.g.attach(pilot, Entity::Object(bear));
    t.g.recompute();
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    let walker = t.battlefield(P1, "Jace Beleren");
    let wall = t.battlefield(P1, "Wall of Stone");
    let from = t.asked().len();
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(bear, obj(walker))]),
    );
    t.advance_to(P0, Step::DeclareAttackers);
    let options = asked_since(&t, from)
        .into_iter()
        .find_map(|(_, d)| match d {
            Decision::DeclareAttackers { options } => Some(options),
            _ => None,
        })
        .expect("asked");
    let (_, targets) = options.iter().find(|(c, _)| *c == bear).unwrap();
    assert!(targets.contains(&obj(walker)) && targets.contains(&pl(P1)));
    assert!(!targets.contains(&obj(wall)));
    // Attacking a permanent its owner controls, it can be blocked.
    assert!(legal_blocks(&mut t, P1, &[(wall, bear)]));
}

// --- Danse Macabre -------------------------------------------------------------------------------

#[test]
fn danse_macabre_everyone_chooses_in_turn_order_then_sacrifices_before_the_roll() {
    cr!("101.4", "701.21a", "706.2");
    ruling!(
        "Danse Macabre",
        "All creatures are chosen and sacrificed before the die is rolled. You don't get to see what the result of the die roll is before choosing which creature to sacrifice."
    );
    ruling!(
        "Danse Macabre",
        "Starting with you and continuing in turn order, each player will choose which creature they will sacrifice. They will know the choices made by players before them in this order when they make that choice. The creatures are all sacrificed at the same time."
    );
    supported("Danse Macabre");
    let mut t = TestGame::new(3);
    let mine = [
        t.battlefield(P0, "Grizzly Bears"),
        t.battlefield(P0, "Hill Giant"),
    ];
    let p1 = [
        t.battlefield(P1, "Grizzly Bears"),
        t.battlefield(P1, "Hill Giant"),
    ];
    let p2 = [
        t.battlefield(P2, "Grizzly Bears"),
        t.battlefield(P2, "Hill Giant"),
    ];
    t.lands(P0, "Swamp", 5);
    t.g.dice.loaded.push_back(1);
    let creatures_now = |g: &mtg_engine::game::Game| {
        (
            g.permanents().filter(|o| o.is(CardType::Creature)).count(),
            g.dice.loaded.len(),
        )
    };
    let when = |d: &Decision| matches!(d, Decision::ChooseEntities { .. });
    let seen0 = watch(&mut t, P0, when, creatures_now);
    let seen1 = watch(&mut t, P1, when, creatures_now);
    let seen2 = watch(&mut t, P2, when, creatures_now);
    t.answer_choose(P0, &[obj(mine[0])]);
    t.answer_choose(P1, &[obj(p1[1])]);
    t.answer_choose(P2, &[obj(p2[0])]);
    let from = t.asked().len();
    let spell = t.hand(P0, "Danse Macabre");
    t.cast(P0, spell).go();
    t.resolve_all();
    // The sacrifice choices came in turn order, all with six creatures still there and the
    // die not rolled yet.
    let order: Vec<PlayerId> = asked_since(&t, from)
        .into_iter()
        .filter(|(_, d)| when(d))
        .map(|(p, _)| p)
        .take(3)
        .collect();
    assert_eq!(order, vec![P0, P1, P2]);
    for seen in [&seen0, &seen1, &seen2] {
        assert_eq!(seen.lock().unwrap().first().copied(), Some((6, 1)));
    }
    assert!(t.g.dice.loaded.is_empty(), "the die was rolled");
    // Three sacrificed, one returned.
    assert_eq!(
        creatures(&t, P0).len() + creatures(&t, P1).len() + creatures(&t, P2).len(),
        4
    );
    assert!(t.in_graveyard(P1, "Hill Giant") && t.in_graveyard(P2, "Grizzly Bears"));
}

#[test]
fn danse_macabre_can_return_a_sacrificed_commander() {
    cr!("903.9a", "704.6d");
    ruling!(
        "Danse Macabre",
        "If a player sacrifices their commander this way, they won't be able to choose whether to put it into the command zone until after Danse Macabre has finished resolving. Notably, this means that the sacrificed commander is a legal choice for the spell's controller to return to the battlefield. If they choose it this way, it will stay on the battlefield under their control."
    );
    let mut t = commander_game(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let cmdr = t.battlefield(P1, "Hill Giant");
    make_commander(&mut t, cmdr);
    t.lands(P0, "Swamp", 5);
    t.g.dice.loaded.push_back(12);
    t.answer_choose(P0, &[obj(bears)]);
    t.answer_choose(P1, &[obj(cmdr)]);
    t.answer_choose_moved(P0, &[obj(cmdr)]);
    t.answer_yes(P1, true);
    let spell = t.hand(P0, "Danse Macabre");
    t.cast(P0, spell).go();
    t.resolve_all();
    let giant = t.named_on_battlefield("Hill Giant");
    assert_eq!(giant.len(), 1, "{}", t.dump_log());
    assert_eq!(controller(&t, giant[0]), P0);
    assert!(t.g.find_in_zone(Zone::Command, "Hill Giant").is_empty());
}

// --- Sen Triplets, Green Sun's Zenith ------------------------------------------------------

/// P0's upkeep; P0's Sen Triplets has targeted P1 this turn.
fn triplets() -> TestGame {
    supported("Sen Triplets");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sen Triplets");
    t.answer_targets(P0, &[pl(P1)]);
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    t
}

#[test]
fn sen_triplets_leaves_triggers_alone_and_gives_no_activations_from_their_hand() {
    cr!("603.1", "602.1", "601.2a");
    ruling!(
        "Sen Triplets",
        "Triggered abilities use the word “when,” “whenever,” or “at.” They're often written as “[Trigger condition], [effect].” Triggered abilities are unaffected by Sen Triplets."
    );
    ruling!(
        "Sen Triplets",
        "You can't activate abilities of cards in the target opponent's hand."
    );
    let mut t = triplets();
    // P1's Soul Warden still triggers.
    t.battlefield(P1, "Soul Warden");
    t.enter(P0, "Grizzly Bears");
    t.resolve_all();
    assert_eq!(t.life(P1), 21);
    // P0 can't cycle P1's Desert Cerodon (P0 could cast it).
    t.advance_to(P0, Step::PrecombatMain);
    let cerodon = t.hand(P1, "Desert Cerodon");
    t.lands(P0, "Mountain", 6);
    t.g.turn.priority = Some(P0);
    t.g.recompute();
    let acts = t.g.activatable_abilities(P0);
    assert!(!acts.iter().any(|(id, _)| *id == cerodon));
    assert!(can_cast(
        &mut t,
        P0,
        cerodon,
        mtg_engine::object::CastMethod::Normal
    ));
}

#[test]
fn green_suns_zenith_cast_by_an_opponent_searches_their_library_and_shuffles_into_yours() {
    cr!("108.3", "400.3", "701.23a");
    ruling!(
        "Green Sun's Zenith",
        "If you own Green Sun's Zenith, but an opponent casts it (due to Knowledge Pool's effect, for example), that opponent searches their library for an appropriate creature card, then shuffles that library. That opponent then shuffles Green Sun's Zenith into your library. You won't shuffle any library in this case."
    );
    supported("Green Sun's Zenith");
    // P0 casts P1's Green Sun's Zenith with Sen Triplets.
    let mut t = triplets();
    t.advance_to(P0, Step::PrecombatMain);
    let gsz = t.hand(P1, "Green Sun's Zenith");
    let bears = t.library_top(P0, "Grizzly Bears");
    t.library_top(P1, "Llanowar Elves");
    t.lands(P0, "Forest", 3);
    t.answer_choose(P0, &[obj(bears)]);
    let from = t.g.turn_events.len();
    t.cast(P0, gsz).x(2).go();
    t.resolve_all();
    assert_eq!(n_named(&t, P0, "Grizzly Bears"), 1, "{}", t.dump_log());
    assert!(t.named_on_battlefield("Llanowar Elves").is_empty());
    let in_lib = |t: &TestGame, p: PlayerId| {
        t.g.player(p)
            .library
            .iter()
            .any(|c| t.g.obj(*c).chars.name == "Green Sun's Zenith")
    };
    assert!(in_lib(&t, P1) && !in_lib(&t, P0));
    assert!(shuffles_since(&t, P0, from) >= 1);
    // The owner's library is shuffled only as Zenith goes into it (CR 701.24): one shuffle.
    assert_eq!(shuffles_since(&t, P1, from), 1);
}
