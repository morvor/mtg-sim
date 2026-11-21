//! Item `rulings-residue-misc-1`, pass B — combat rulings (batches P074, P076, P085, P094):
//! evasion gained or lost after blockers are declared (CR 509.1b, 702.14), first strike
//! lost between the combat damage steps (CR 510.4), "can't be blocked by creatures that
//! player controls" chosen as an ability resolves, additional combat phases (CR 500.8),
//! attacking creatures dying together, and Giant Oyster's draw-step counters.

use crate::r_residue_misc_1_common::*;
use mtg_engine::ability::*;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// P0 attacks P1 with `attacker`; P1 answers the block with `blocks`. If `before` is given,
/// P1 activates that land's ability (its second, "{T}: Target creature loses ...")
/// targeting the attacker in the declare attackers step; `after`, in the declare blockers
/// step. Combat is run to its end.
fn landwalk_attack(
    t: &mut TestGame,
    attacker: ObjectId,
    blocks: &[(ObjectId, ObjectId)],
    before: Option<ObjectId>,
    after: Option<ObjectId>,
) {
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(attacker, P1.into())]),
    );
    if !blocks.is_empty() {
        t.answer(
            P1,
            DecisionKind::Blockers,
            Answer::Blockers(blocks.to_vec()),
        );
    }
    t.advance_to(P0, Step::DeclareAttackers);
    if let Some(land) = before {
        t.activate(P1, land, 1, &[obj(attacker)]).unwrap();
        t.resolve_all();
    }
    t.advance_to(P0, Step::DeclareBlockers);
    if let Some(land) = after {
        t.activate(P1, land, 1, &[obj(attacker)]).unwrap();
        t.resolve_all();
    }
    t.advance_to(P0, Step::EndOfCombat);
}

#[test]
fn losing_landwalk_helps_only_before_blockers_are_declared() {
    cr!("509.1b", "702.14c", "506.4");
    ruling!(
        "Urborg",
        "Causing a creature to lose swampwalk is useful only during the declare attackers step or earlier. Once the declare blockers step begins, it’s too late to block the creature."
    );
    ruling!(
        "Hammerheim",
        "Causing a creature to lose landwalk abilities is useful only during the declare attackers step or earlier. Once the declare blockers step begins, it’s too late to block the creature."
    );
    for land in ["Urborg", "Hammerheim"] {
        for early in [true, false] {
            let mut t = TestGame::new(2);
            // Bog Wraith (3/3 swampwalk) against P1's Swamp and Hill Giant.
            let wraith = t.battlefield(P0, "Bog Wraith");
            t.battlefield(P1, "Swamp");
            let giant = t.battlefield(P1, "Hill Giant");
            let l = t.battlefield(P1, land);
            // Urborg: "loses first strike or swampwalk": swampwalk.
            t.answer(P1, DecisionKind::Option, Answer::Index(1));
            let (before, after) = if early {
                (Some(l), None)
            } else {
                (None, Some(l))
            };
            landwalk_attack(&mut t, wraith, &[(giant, wraith)], before, after);
            if early {
                assert_eq!(t.life(P1), 20, "{land}: blocked");
            } else {
                assert_eq!(t.life(P1), 17, "{land}: unblocked");
            }
        }
    }
}

#[test]
fn gaining_landwalk_after_being_blocked_doesnt_unblock() {
    cr!("509.1h", "702.14c");
    ruling!(
        "Wormwood Treefolk",
        "Causing Wormwood Treefolk to gain forestwalk or swampwalk is useful only during the declare attackers step or earlier. Once it becomes blocked, giving it a landwalk ability won't change that."
    );
    let mut t = TestGame::new(2);
    let tree = t.battlefield(P0, "Wormwood Treefolk");
    t.battlefield(P1, "Forest");
    let wall = t.battlefield(P1, "Wall of Air");
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(tree, P1.into())]),
    );
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(wall, tree)]),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    t.g.players[0]
        .mana_pool
        .add_type(mtg_engine::mana::ManaType::G, 2);
    activate_containing(&mut t, P0, tree, "forestwalk").unwrap();
    t.resolve_all();
    assert!(has(&mut t, tree, KeywordKind::Landwalk));
    assert_eq!(t.life(P0), 18);
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20, "still blocked");
}

#[test]
fn urborg_removing_first_strike_after_first_strike_damage() {
    cr!("510.4", "702.7c");
    ruling!(
        "Urborg",
        "If a creature loses first strike after first strike combat damage has been assigned, it won’t deal damage during the second combat damage step."
    );
    let mut t = TestGame::new(2);
    // White Knight: 2/2 first strike.
    let knight = t.battlefield(P0, "White Knight");
    let urborg = t.battlefield(P1, "Urborg");
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(knight, P1.into())]),
    );
    t.advance_to(P0, Step::FirstStrikeDamage);
    assert_eq!(t.life(P1), 18);
    t.activate(P1, urborg, 1, &[obj(knight)]).unwrap();
    t.resolve_all();
    assert!(!has(&mut t, knight, KeywordKind::FirstStrike));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 18);
}

#[test]
fn urborg_and_hammerheim_target_creatures_without_those_abilities() {
    cr!("115.1", "609.3");
    ruling!(
        "Urborg",
        "The ability can be used on a creature without First Strike or Swampwalk but has no effect."
    );
    ruling!(
        "Hammerheim",
        "This can be used on a creature without landwalking but has no effect."
    );
    for land in ["Urborg", "Hammerheim"] {
        let mut t = TestGame::new(2);
        let l = t.battlefield(P0, land);
        let bears = t.battlefield(P1, "Grizzly Bears");
        assert!(t.activate(P0, l, 1, &[obj(bears)]).is_ok(), "{land}");
        t.resolve_all();
        assert_eq!(t.pt(bears), (2, 2));
    }
}

#[test]
fn urborg_doesnt_take_away_double_strike() {
    cr!("702.4", "702.7");
    ruling!(
        "Urborg",
        "Urborg’s second ability affects only the first strike ability, not the part of the double strike ability that acts like first strike."
    );
    let mut t = TestGame::new(2);
    // Fencing Ace: 1/1 double strike.
    let ace = t.battlefield(P0, "Fencing Ace");
    let urborg = t.battlefield(P1, "Urborg");
    t.activate(P1, urborg, 1, &[obj(ace)]).unwrap();
    t.resolve_all();
    assert!(has(&mut t, ace, KeywordKind::DoubleStrike));
    t.attack(&[(ace, P1.into())], &[]);
    assert_eq!(t.life(P1), 18);
}

#[test]
fn the_black_gate_chooses_the_player_once() {
    cr!("608.2c", "611.2c", "509.1b");
    ruling!(
        "The Black Gate",
        "The chosen player needs only to have the most life or tied for most life as The Black Gate's last ability resolves. After the ability resolves, any changes in life total won't affect which creatures can block or be blocked that turn."
    );
    ruling!(
        "The Black Gate",
        "The creature that was targeted by The Black Gate's last ability can't be blocked by any creature the chosen player controls, including creatures that weren't on the battlefield when The Black Gate's ability resolved."
    );
    let mut t = TestGame::new(2);
    let gate = t.battlefield(P0, "The Black Gate");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.players[0].life = 15;
    fill_pool(&mut t, P0, 1);
    t.answer_choose(P0, &[P1.into()]);
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    activate_containing(&mut t, P0, gate, "most life").unwrap();
    t.resolve_all();
    // P1 now has less life than P0, and a creature that entered later.
    t.g.players[1].life = 10;
    let wall = t.enter(P1, "Wall of Air");
    t.resolve_all();
    t.g.objects[wall.0 as usize].summoning_sick = false;
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(bears, P1.into())]),
    );
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(wall, bears)]),
    );
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 8, "the Bears couldn't be blocked");
}

#[test]
fn balthier_and_frans_extra_combat_has_no_main_phase() {
    cr!("500.8", "505.1a");
    ruling!(
        "Balthier and Fran",
        "Balthier and Fran's last ability doesn't give you any additional main phases. This means that you will move directly from the end of combat step of one combat phase to the beginning of combat step of the next one."
    );
    let mut t = TestGame::new(2);
    let bf = t.battlefield(P0, "Balthier and Fran");
    let copter = t.battlefield(P0, "Smuggler's Copter");
    t.answer_choose(P0, &[obj(bf)]);
    activate_containing(&mut t, P0, copter, "Crew").unwrap();
    t.resolve_all();
    assert!(t.obj_now(copter).is(CardType::Creature));
    t.lands(P0, "Plateau", 1);
    t.lands(P0, "Forest", 2);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.attack(&[(copter, P1.into())], &[]);
    t.resolve_all();
    // From this end of combat step straight to the next beginning of combat step.
    let steps: Vec<Step> = t.g.turn.schedule.iter().take(2).copied().collect();
    assert_eq!(steps.first(), Some(&Step::BeginningOfCombat), "{steps:?}");
}

#[test]
fn rinoa_sees_all_attackers_that_died_together_including_itself() {
    cr!("603.10a", "603.2c");
    ruling!(
        "Rinoa, Angel Wing",
        "If you're attacking with Rinoa and one or more other attacking creatures you control die at the same time, Rinoa's last ability will still trigger and see all of those creatures, including Rinoa."
    );
    let mut t = TestGame::new(2);
    let rinoa = t.battlefield(P0, "Rinoa, Angel Wing");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let a = t.battlefield(P1, "Craw Wurm");
    let b = t.battlefield(P1, "Hill Giant");
    t.answer_yes(P0, true);
    t.answer_choose_moved(P0, &[obj(rinoa)]);
    t.attack(
        &[(rinoa, P1.into()), (bears, P1.into())],
        &[(a, rinoa), (b, bears)],
    );
    t.resolve_all();
    let rinoa = t.g.current(rinoa);
    assert!(t.on_battlefield(rinoa));
    assert!(t.obj_now(rinoa).tapped);
    assert_eq!(t.counters(rinoa, "flying"), 1);
    assert_eq!(t.zone(bears), Zone::Graveyard(P0));
}

#[test]
fn giant_oyster_counters_from_the_next_draw_step() {
    cr!("603.7", "611.2b");
    ruling!(
        "Giant Oyster",
        "Giant Oyster’s ability can be activated during your upkeep, and the creature will get a -1/-1 counter during the draw step that follows it."
    );
    ruling!(
        "Giant Oyster",
        "The activated ability can target only a tapped creature."
    );
    let mut t = TestGame::new(2);
    let oyster = t.battlefield(P0, "Giant Oyster");
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.objects[giant.0 as usize].tapped = true;
    t.set_step(P0, Step::Upkeep);
    let from = t.asked().len();
    t.activate(P0, oyster, 0, &[obj(giant)]).unwrap();
    let offered = crate::r_s02_common::target_candidates(&t, P0, from);
    assert!(offered
        .iter()
        .all(|c| c.contains(&obj(giant)) && !c.contains(&obj(bears))));
    t.resolve_all();
    t.advance_to(P0, Step::Draw);
    t.resolve_all();
    assert_eq!(t.counters(giant, counters::MINUS1), 1);
}

#[test]
fn giant_oyster_untapped_with_its_draw_step_trigger_waiting() {
    cr!("603.7c", "603.4");
    ruling!(
        "Giant Oyster",
        "If the Oyster untaps during the draw step before its ability to add a -1/-1 counter resolves, the -1/-1 counter effect resolves and adds a -1/-1 counter after the -1/-1 counters are removed due to being untapped."
    );
    let mut t = TestGame::new(2);
    let oyster = t.battlefield(P0, "Giant Oyster");
    let giant = t.battlefield(P1, "Craw Wurm");
    t.g.objects[giant.0 as usize].tapped = true;
    t.set_step(P0, Step::Upkeep);
    t.activate(P0, oyster, 0, &[obj(giant)]).unwrap();
    t.resolve_all();
    // A first draw step: one counter.
    t.advance_to(P0, Step::Draw);
    t.resolve_all();
    assert_eq!(t.counters(giant, counters::MINUS1), 1);
    // The next draw step: its trigger waits on the stack, and the Oyster untaps.
    t.set_step(P1, Step::End);
    t.answer_yes(P0, false);
    t.advance_to(P0, Step::Draw);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    run_from(
        &mut t,
        P0,
        None,
        Effect::Untap {
            what: Sel::Target(0),
        },
        &[obj(oyster)],
    );
    t.resolve_all();
    assert!(!t.obj_now(oyster).tapped);
    assert_eq!(
        t.counters(giant, counters::MINUS1),
        1,
        "removed, then one added"
    );
}
