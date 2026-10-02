//! Rulings batch P050 — activated and loyalty abilities and spells around drain-life
//! cards: sacrificing a permanent to its own ability (CR 602.2b, 118.3), modes that
//! haven't been chosen (CR 700.2), activation restrictions checked only on activation
//! (CR 602.5), dice-roll triggers (CR 706), power/toughness-setting effects (CR 613.4b),
//! redundant lifelink and deathtouch (CR 702.2f, 702.15f) and controlling a player
//! (CR 723).

use crate::r_p057_common::into_upkeep;
use crate::r_s01_common::{attack_with, supported, triggers_on_stack};
use crate::r_s05_common::{move_to, run_from};
use mtg_engine::ability::*;
use mtg_engine::decision::{Action, Answer, Decision};
use mtg_engine::dice::DieRoll;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::player_control::decider;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const BEARS: &str = "Grizzly Bears";

#[test]
fn cabal_archon_can_sacrifice_itself() {
    cr!("602.2b", "701.21a");
    ruling!("Cabal Archon", "It can sacrifice itself.");
    supported("Cabal Archon");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 1);
    let a = t.battlefield(P0, "Cabal Archon");
    t.answer_choose(P0, &[Entity::Object(a)]);
    t.activate(P0, a, 0, &[Entity::Player(P1)]).unwrap();
    assert!(t.in_graveyard(P0, "Cabal Archon"));
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (22, 18));
}

#[test]
fn lampad_can_be_sacrificed_to_its_own_ability() {
    cr!("602.2b", "701.21a");
    ruling!(
        "Lampad of Death's Vigil",
        "Lampad of Death’s Vigil can be sacrificed to pay the cost of its own ability."
    );
    supported("Lampad of Death's Vigil");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 1);
    let l = t.battlefield(P0, "Lampad of Death's Vigil");
    t.answer_choose(P0, &[Entity::Object(l)]);
    t.activate(P0, l, 0, &[]).unwrap();
    assert!(t.in_graveyard(P0, "Lampad of Death's Vigil"));
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (21, 19));
}

#[test]
fn ghost_council_sacrificed_to_its_own_ability_stays_in_the_graveyard() {
    cr!("602.2b", "400.7");
    ruling!(
        "Ghost Council of Orzhova",
        "Sacrificing Ghost Council of Orzhova to its own ability means it will wind up in the graveyard, not exiled."
    );
    supported("Ghost Council of Orzhova");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 1);
    let g = t.battlefield(P0, "Ghost Council of Orzhova");
    t.answer_choose(P0, &[Entity::Object(g)]);
    t.activate(P0, g, 0, &[]).unwrap();
    t.resolve_all();
    t.advance_to(P1, Step::Upkeep);
    assert!(t.in_graveyard(P0, "Ghost Council of Orzhova"));
    assert!(!t.in_exile("Ghost Council of Orzhova"));
    assert!(t.named_on_battlefield("Ghost Council of Orzhova").is_empty());
}

#[test]
fn ghost_council_exiled_during_the_end_step_waits_for_the_next_one() {
    cr!("603.7a");
    ruling!(
        "Ghost Council of Orzhova",
        "But if it's exiled during the End step, it's too late to return it this turn. It has to wait to return to the battlefield until the next End step."
    );
    supported("Ghost Council of Orzhova");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 1);
    let g = t.battlefield(P0, "Ghost Council of Orzhova");
    let bears = t.battlefield(P0, BEARS);
    t.set_step(P0, Step::End);
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.activate(P0, g, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.in_exile("Ghost Council of Orzhova"));
    // Not this turn: it's still exiled during P1's turn until P1's end step begins.
    t.advance_to(P1, Step::PostcombatMain);
    assert!(t.in_exile("Ghost Council of Orzhova"));
    t.advance_to(P1, Step::End);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Ghost Council of Orzhova").len(), 1);
    // Exiled before the end step: it returns at the end step of the same turn.
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 1);
    let g = t.battlefield(P0, "Ghost Council of Orzhova");
    let bears = t.battlefield(P0, BEARS);
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.activate(P0, g, 0, &[]).unwrap();
    t.resolve_all();
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Ghost Council of Orzhova").len(), 1);
}

#[test]
fn shadows_of_the_past_checks_the_graveyard_only_on_activation() {
    cr!("602.5", "602.2");
    ruling!(
        "Shadows of the Past",
        "Once you legally activate the last ability, it doesn’t matter how many creature cards are in your graveyard as it resolves."
    );
    supported("Shadows of the Past");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 5);
    let s = t.battlefield(P0, "Shadows of the Past");
    let cards: Vec<ObjectId> = (0..4).map(|_| t.graveyard(P0, BEARS)).collect();
    t.activate(P0, s, 0, &[]).unwrap();
    // In response, three of them are exiled.
    for c in &cards[..3] {
        move_to(&mut t, *c, Zone::Exile);
    }
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (22, 18));
    // With only three creature cards, it can't be activated.
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 5);
    let s = t.battlefield(P0, "Shadows of the Past");
    for _ in 0..3 {
        t.graveyard(P0, BEARS);
    }
    assert!(t.activate(P0, s, 0, &[]).is_err());
}

#[test]
fn ebony_charm_targets_cards_as_it_is_cast() {
    cr!("601.2c", "115.1");
    ruling!(
        "Ebony Charm",
        "It targets the cards in the graveyard. These targets are selected when casting the spell."
    );
    supported("Ebony Charm");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 1);
    let a = t.graveyard(P1, BEARS);
    let b = t.graveyard(P1, "Hill Giant");
    let charm = t.hand(P0, "Ebony Charm");
    t.cast(P0, charm)
        .modes(&[1])
        .targets(&[Entity::Object(a), Entity::Object(b)])
        .go();
    // A card put into the graveyard after the spell was cast isn't one of its targets.
    let late = t.graveyard(P1, "Gray Ogre");
    t.resolve_all();
    assert!(t.in_exile(BEARS) && t.in_exile("Hill Giant"));
    assert_eq!(t.zone(late), Zone::Graveyard(P1));
}

#[test]
fn tezzeret_agent_of_bolas_sets_base_power_and_toughness() {
    cr!("613.4b", "613.7");
    ruling!(
        "Tezzeret, Agent of Bolas",
        "If the target of the second ability is already an artifact creature, its power and toughness will each become 5. This overwrites all previous effects"
    );
    supported("Tezzeret, Agent of Bolas");
    let set = |t: &mut TestGame, id: ObjectId, n: i32| {
        run_from(
            t,
            P0,
            None,
            Effect::Modify {
                what: Sel::All(Filter::Objects(vec![id])),
                mods: vec![Modification::SetPT(Some(Value::c(n)), Some(Value::c(n)))],
                duration: Duration::EndOfTurn,
            },
            &[],
        )
    };
    let mut t = TestGame::new(2);
    let tez = t.battlefield(P0, "Tezzeret, Agent of Bolas");
    let thopter = t.battlefield(P0, "Ornithopter");
    // An earlier effect made it 1/1.
    set(&mut t, thopter, 1);
    assert_eq!(t.pt(thopter), (1, 1));
    t.activate(P0, tez, 1, &[Entity::Object(thopter)]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(thopter), (5, 5));
    // A later setting effect overwrites it.
    set(&mut t, thopter, 3);
    assert_eq!(t.pt(thopter), (3, 3));
}

#[test]
fn sorin_imperious_bloodlord_redundant_deathtouch_and_lifelink() {
    cr!("702.2f", "702.15f");
    ruling!(
        "Sorin, Imperious Bloodlord",
        "Multiple instances of deathtouch and/or lifelink on the same creature are redundant."
    );
    supported("Sorin, Imperious Bloodlord");
    supported("Vampire Nighthawk");
    // Vampire Nighthawk (2/3 flying, deathtouch, lifelink) gains deathtouch and lifelink
    // and, as a Vampire, a +1/+1 counter: it deals 3 damage and P0 gains 3, not 6.
    let mut t = TestGame::new(2);
    let sorin = t.battlefield(P0, "Sorin, Imperious Bloodlord");
    let hawk = t.battlefield(P0, "Vampire Nighthawk");
    t.activate(P0, sorin, 0, &[Entity::Object(hawk)]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(hawk), (3, 4));
    t.attack(&[(hawk, Entity::Player(P1))], &[]);
    assert_eq!((t.life(P0), t.life(P1)), (23, 17));
}

#[test]
fn sorin_vengeful_bloodlord_redundant_lifelink() {
    cr!("702.15f");
    ruling!(
        "Sorin, Vengeful Bloodlord",
        "Multiple instances of lifelink on the same creature or planeswalker are redundant."
    );
    supported("Sorin, Vengeful Bloodlord");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sorin, Vengeful Bloodlord");
    let hawk = t.battlefield(P0, "Vampire Nighthawk");
    t.attack(&[(hawk, Entity::Player(P1))], &[]);
    assert_eq!((t.life(P0), t.life(P1)), (22, 18));
}

#[test]
fn needlebite_trap_checks_only_that_life_was_gained() {
    cr!("118.9");
    ruling!(
        "Needlebite Trap",
        "Needlebite Trap's alternative cost condition checks only whether life was gained. It doesn't care whether life was also lost."
    );
    supported("Needlebite Trap");
    let alt_methods = |t: &mut TestGame, card: ObjectId| -> Vec<CastMethod> {
        t.g.turn.priority = Some(P0);
        t.g.recompute();
        t.g.legal_actions(P0)
            .into_iter()
            .filter_map(|a| match a {
                Action::Cast { card: c, method } if c == card && method != CastMethod::Normal => {
                    Some(method)
                }
                _ => None,
            })
            .collect()
    };
    // No life gained: only {5}{B}{B}, which P0 can't pay with one Swamp.
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 1);
    let trap = t.hand(P0, "Needlebite Trap");
    assert!(alt_methods(&mut t, trap).is_empty());
    // P1 gained 4 and lost 6: still castable for {B}.
    t.g.gain_life(P1, 4);
    t.g.lose_life(P1, 6);
    t.g.flush_events();
    assert_eq!(t.life(P1), 18);
    let methods = alt_methods(&mut t, trap);
    assert_eq!(methods.len(), 1);
    t.cast(P0, trap)
        .method(methods[0].clone())
        .target(P1)
        .go();
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (25, 13));
}

/// P0's Demonic Pact's upkeep trigger, choosing `mode` (targets answered by the caller).
fn pact_upkeep(t: &mut TestGame, mode: usize) {
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![mode]));
    into_upkeep(t, P0);
    t.resolve_all();
}

#[test]
fn demonic_pact_with_no_mode_left_is_removed_from_the_stack() {
    cr!("700.2b");
    ruling!(
        "Demonic Pact",
        "In some very unusual situations, you may not be able to choose a mode, either because all modes have previously been chosen or the only remaining modes require targets and there are no legal targets available. In this case, the ability is simply removed from the stack with no effect."
    );
    supported("Demonic Pact");
    supported("Platinum Angel");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Demonic Pact");
    t.battlefield(P0, "Platinum Angel");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    pact_upkeep(&mut t, 0);
    assert_eq!(t.life(P1), 16);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    pact_upkeep(&mut t, 1);
    pact_upkeep(&mut t, 2);
    pact_upkeep(&mut t, 3);
    assert!(!t.has_lost(P0), "Platinum Angel");
    // Every mode has been chosen: the next trigger does nothing.
    let (life, hand) = (t.life(P1), t.hand_size(P0));
    let from = t.asked().len();
    into_upkeep(&mut t, P0);
    assert_eq!(t.stack_len(), 0, "removed from the stack");
    assert!(!t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseModes { .. })));
    assert_eq!((t.life(P1), t.hand_size(P0)), (life, hand));
    assert!(!t.has_lost(P0));
}

#[test]
fn demonic_pact_modes_chosen_by_anyone_count() {
    cr!("700.2b");
    ruling!(
        "Demonic Pact",
        "It doesn't matter who has chosen any particular mode. For example, say you control Demonic Pact and have chosen the first two modes."
    );
    supported("Demonic Pact");
    let mut t = TestGame::new(2);
    let pact = t.battlefield(P0, "Demonic Pact");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    pact_upkeep(&mut t, 0);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    pact_upkeep(&mut t, 1);
    // P1 gains control of it; at P1's upkeep only the third and fourth modes are left.
    run_from(
        &mut t,
        P1,
        None,
        Effect::GainControl {
            what: Sel::All(Filter::Objects(vec![pact])),
            who: PlayerRef::Player(P1),
            duration: Duration::Permanent,
        },
        &[],
    );
    let from = t.asked().len();
    t.answer(P1, DecisionKind::Modes, Answer::Indices(vec![2]));
    into_upkeep(&mut t, P1);
    let offered: Vec<Vec<usize>> = t.asked()[from..]
        .iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseModes { available, .. } if *p == P1 => Some(available.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(offered, vec![vec![2, 3]]);
    t.resolve_all();
    assert_eq!(t.hand_size(P1), 2, "P1 drew two");
}

/// Dee Kay, Finder of the Lost on P0's battlefield: "Whenever you roll a 2, each opponent
/// loses 1 life and you gain 1 life. / Whenever you roll a 4, you may tap or untap target
/// artifact or creature. / Whenever you roll a 6, return target creature card from your
/// graveyard to your hand."
fn dee_kay() -> TestGame {
    supported("Dee Kay, Finder of the Lost");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Dee Kay, Finder of the Lost");
    t
}

/// P0 rolls `count` `sides`-sided dice with the given natural results.
fn roll(t: &mut TestGame, count: i32, sides: u32, naturals: &[u32]) {
    t.g.dice.loaded.extend(naturals.iter().copied());
    let mut d = DieRoll::new(sides);
    d.count = Value::c(count);
    run_from(t, P0, None, Effect::RollDice(Box::new(d)), &[]);
}

#[test]
fn dee_kay_triggers_after_the_rolling_spell_finishes() {
    cr!("603.3", "706.3a");
    ruling!(
        "Dee Kay, Finder of the Lost",
        "If the spell or ability that caused you to roll a die includes any effects due to the result, resolve those effects before putting Dee Kay's triggered abilities on the stack."
    );
    supported("Diviner's Portent");
    // Diviner's Portent: "Roll a d20 and add the number of cards in your hand. 1—14 | Draw
    // X cards. ..." X is the result: with an empty hand, a natural 2 draws two cards.
    let mut t = dee_kay();
    t.lands(P0, "Island", 5);
    let p = t.hand(P0, "Diviner's Portent");
    t.cast(P0, p).x(2).go();
    t.g.dice.loaded.push_back(2);
    t.resolve();
    assert_eq!(t.hand_size(P0), 2, "the result's effect happened");
    assert_eq!(triggers_on_stack(&t, "loses 1 life"), 1, "then Dee Kay triggered");
    assert_eq!(t.life(P0), 20);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (21, 19));
}

#[test]
fn dee_kay_triggers_for_each_die() {
    cr!("603.2c", "706.1");
    ruling!(
        "Dee Kay, Finder of the Lost",
        "If you roll more than one die at a time, Dee Kay's abilities will trigger for each corresponding result."
    );
    let mut t = dee_kay();
    let bears = t.battlefield(P1, BEARS);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    roll(&mut t, 3, 6, &[2, 2, 4]);
    assert_eq!(triggers_on_stack(&t, "loses 1 life"), 2);
    assert_eq!(triggers_on_stack(&t, "tap or untap"), 1);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (22, 18));
}

#[test]
fn dee_kay_any_die_counts() {
    cr!("706.1", "603.2");
    ruling!(
        "Dee Kay, Finder of the Lost",
        "It doesn't matter why you rolled a die or how many sides that die has: if you roll a 2, a 4, or a 6, the corresponding ability will trigger."
    );
    // A d4 showing 2 and a d20 showing 6.
    let mut t = dee_kay();
    let card = t.graveyard(P0, BEARS);
    roll(&mut t, 1, 4, &[2]);
    assert_eq!(triggers_on_stack(&t, "loses 1 life"), 1);
    t.resolve_all();
    t.answer_targets(P0, &[Entity::Object(card)]);
    roll(&mut t, 1, 20, &[6]);
    assert_eq!(triggers_on_stack(&t, "return target creature card"), 1);
    t.resolve_all();
    assert!(t.in_hand(P0, BEARS));
    assert_eq!(t.life(P1), 19);
}

#[test]
fn dee_kay_and_visit_abilities_go_on_the_stack_in_any_order() {
    cr!("603.3b", "702.159a");
    ruling!(
        "Dee Kay, Finder of the Lost",
        "If you roll a 2, a 4, or a 6 while rolling to visit your attractions, and one or more visit abilities also trigger from that roll, you choose the order in which those abilities and Dee Kay's ability go on the stack."
    );
    // Information Booth (2 and 6 lit up): "Visit — Draw a card."
    let mut tops = vec![];
    for order in [vec![0, 1], vec![1, 0]] {
        let mut t = dee_kay();
        t.battlefield(P0, "Information Booth");
        t.answer(P0, DecisionKind::Order, Answer::Indices(order));
        t.g.dice.loaded.push_back(2);
        mtg_engine::variants::roll_to_visit(&mut t.g, P0);
        t.g.flush_events();
        t.settle();
        assert_eq!(t.stack_len(), 2);
        t.resolve();
        let drained_first = t.life(P1) == 19;
        tops.push(drained_first);
        assert_eq!(t.hand_size(P0), if drained_first { 0 } else { 1 });
        t.resolve_all();
        assert_eq!((t.life(P1), t.hand_size(P0)), (19, 1));
    }
    assert_ne!(tops[0], tops[1], "either can resolve first");
}

#[test]
fn sorin_markov_controls_the_next_turn_the_player_actually_takes() {
    cr!("723.1");
    ruling!(
        "Sorin Markov",
        "Sorin’s third ability allows you to control another player. This effect applies to the next turn that the affected player actually takes."
    );
    ruling!(
        "Sorin Markov",
        "If the player affected by Sorin’s third ability skips their next turn, the ability will wait. You’ll control the next turn the affected player actually takes."
    );
    supported("Sorin Markov");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Sorin Markov");
    t.g.objects[s.0 as usize]
        .counters
        .insert("loyalty".into(), 7);
    t.activate(P0, s, 2, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    t.g.players[1].skips.push(StepKind::Turn);
    let turn = t.g.turn.number;
    assert!(t.g.run_until(500, |g| g.turn.number == turn + 1));
    assert_eq!(t.g.turn.active, P0, "P1's turn was skipped");
    assert_eq!(decider(&t.g, P1), P1);
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(decider(&t.g, P1), P0, "P0 controls P1's turn");
    t.advance_to(P0, Step::Upkeep);
    assert_eq!(decider(&t.g, P1), P1, "only for that turn");
}

#[test]
fn brutal_hordechief_block_costs_may_be_declined() {
    cr!("509.1d");
    ruling!(
        "Brutal Hordechief",
        "If there’s a cost associated with having a creature block and you choose for that creature to block, its controller can choose to pay that cost or not."
    );
    supported("Brutal Hordechief");
    supported("Archangel of Tithes");
    for pays in [false, true] {
        let mut t = TestGame::new(2);
        let chief = t.battlefield(P0, "Brutal Hordechief");
        // "As long as this creature is attacking, creatures can't block unless their
        // controller pays {1} for each of those creatures."
        let archangel = t.battlefield(P0, "Archangel of Tithes");
        let bears = t.battlefield(P0, BEARS);
        let ogre = t.battlefield(P1, "Gray Ogre");
        t.lands(P1, "Wastes", 1);
        t.lands(P0, "Mountain", 5);
        crate::r_s06_common::activate_containing(
            &mut t,
            P0,
            chief,
            "you choose how those creatures block",
        )
        .unwrap();
        t.resolve_all();
        attack_with(
            &mut t,
            &[
                (archangel, Entity::Player(P1)),
                (bears, Entity::Player(P1)),
            ],
        );
        t.resolve_all();
        // P0 has the Ogre block the Bears; P1 decides whether to pay {1}. If P1 doesn't,
        // P0 proposes again: no blocks.
        t.answer(
            P0,
            DecisionKind::Blockers,
            Answer::Blockers(vec![(ogre, bears)]),
        );
        t.answer_yes(P1, pays);
        t.answer(P0, DecisionKind::Blockers, Answer::Blockers(vec![]));
        crate::r_s21_common::go_to(&mut t, Step::DeclareBlockers);
        let blocks = crate::r_s21_common::blocks_now(&t);
        if pays {
            assert_eq!(blocks, vec![(ogre, bears)]);
        } else {
            assert!(blocks.is_empty(), "a new set of blocks was proposed");
        }
    }
}
