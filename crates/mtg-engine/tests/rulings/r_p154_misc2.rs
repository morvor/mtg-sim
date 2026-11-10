//! Rulings batch P154 — abilities that scale with a mana value, miscellany (second half):
//! a Saga whose chapter was copied (CR 707.10, 714.2), activated abilities in other zones
//! and face-down exiled cards (CR 602.2, 406.3), mana added by a triggered ability (CR
//! 605.1a), "as enters" replacements (CR 614.1c), reflexive triggers (CR 603.12),
//! combat damage steps (CR 510.4), triggers with no legal target (CR 603.3d), intervening
//! ifs (CR 603.4), no priority during resolution (CR 608.2, 117.3c), "do this only once
//! each turn", simultaneous deaths (CR 603.10a), state triggers (CR 603.8), creatures put
//! onto the battlefield attacking (CR 508.4), Phyrexian mana (CR 107.4f), "can't gain
//! life" (CR 119.8), and Captain Rex Nebula's Vehicles.

use crate::r_p154_common::*;
use mtg_engine::decision::Decision;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// The Creation of Avacyn enters for P0 with its chapter I ability copied by Strionic
/// Resonator, exiling `first` and `second` from P0's library face down; then the game
/// reaches P0's next precombat main phase with chapter II resolved. Returns the Saga.
fn avacyn_with_two_exiled(t: &mut TestGame, first: &str, second: &str) -> (ObjectId, ObjectId) {
    let res = t.battlefield(P0, "Strionic Resonator");
    let a = t.library_top(P0, first);
    let b = t.library_top(P0, second);
    for _ in 0..3 {
        t.library_top(P0, "Forest");
    }
    t.enter(P0, "The Creation of Avacyn");
    t.settle();
    assert_eq!(t.stack_len(), 1, "chapter I");
    let top = *t.g.stack.last().unwrap();
    mana(t, P0, "2");
    t.answer_targets(P0, &[obj(top)]);
    crate::r_s06_common::activate_containing(t, P0, res, "Copy target").expect("Resonator");
    t.answer_choose(P0, &[obj(a)]);
    t.answer_choose(P0, &[obj(b)]);
    t.resolve_all();
    assert!(t.obj_now(a).face_down && t.obj_now(b).face_down);
    assert_eq!(t.zone(a), Zone::Exile);
    assert_eq!(t.zone(b), Zone::Exile);
    (a, b)
}

#[test]
fn the_creation_of_avacyn_with_two_exiled_cards() {
    cr!("707.10", "714.2b", "608.2c");
    ruling!(
        "The Creation of Avacyn",
        "In the unusual case where two or more cards are exiled face down with The Creation of Avacyn's first chapter ability (likely because the triggered ability was copied or the ability triggered a second time), the second chapter ability will turn all of the exiled cards face up. If at least one of them is a creature card, you'll lose life equal to the combined mana value of all the exiled cards."
    );
    ruling!(
        "The Creation of Avacyn",
        "In the unusual case where two or more cards are exiled face down with The Creation of Avacyn's first ability when the third chapter ability resolves, if at least one of the exiled cards is a creature card, you may choose to put all or none of the exiled cards that are permanent cards onto the battlefield. Regardless of what you choose, any remaining exiled cards will be put into their owners' hands."
    );
    supported("The Creation of Avacyn");
    supported("Strionic Resonator");
    for put in [true, false] {
        let mut t = TestGame::new(2);
        let (a, b) = avacyn_with_two_exiled(&mut t, "Hill Giant", "Mind Stone");
        // Chapter II: both face up; a creature among them, so 4 + 2 life is lost.
        t.advance_to(P1, Step::PrecombatMain);
        t.advance_to(P0, Step::PrecombatMain);
        t.resolve_all();
        assert!(!t.obj_now(a).face_down && !t.obj_now(b).face_down);
        assert_eq!(t.life(P0), 14, "put: {put}");
        // Chapter III: all permanent cards onto the battlefield, or none (then to hand).
        t.answer_yes(P0, put);
        t.advance_to(P1, Step::PrecombatMain);
        t.advance_to(P0, Step::PrecombatMain);
        t.resolve_all();
        if put {
            assert_eq!(count_named(&t, "Hill Giant"), 1, "put: {put}");
            assert_eq!(count_named(&t, "Mind Stone"), 1, "put: {put}");
        } else {
            assert!(t.in_hand(P0, "Hill Giant") && t.in_hand(P0, "Mind Stone"), "put: {put}");
        }
    }
    // Two noncreature cards: no life lost, and both go to hand at chapter III.
    let mut t = TestGame::new(2);
    avacyn_with_two_exiled(&mut t, "Mind Stone", "Bonesplitter");
    t.advance_to(P1, Step::PrecombatMain);
    t.advance_to(P0, Step::PrecombatMain);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    t.advance_to(P1, Step::PrecombatMain);
    t.advance_to(P0, Step::PrecombatMain);
    t.resolve_all();
    assert!(t.in_hand(P0, "Mind Stone") && t.in_hand(P0, "Bonesplitter"));
}

#[test]
fn karn_the_great_creator_stops_only_battlefield_abilities_and_takes_only_face_up_cards() {
    cr!("602.2", "113.6", "406.3");
    ruling!(
        "Karn, the Great Creator",
        "Karn's first ability affects only artifacts on the battlefield. Activated abilities that work in other zones can still be activated."
    );
    ruling!(
        "Karn, the Great Creator",
        "Karn's last ability can't get a face-down card in exile, even if you know it's an artifact card."
    );
    supported("Karn, the Great Creator");
    // P1's Metalwork Colossus in their graveyard can be activated under P0's Karn; P1's
    // Mind Stone on the battlefield can't.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Karn, the Great Creator");
    let stone = t.battlefield(P1, "Mind Stone");
    t.battlefield(P1, "Mind Stone");
    let colossus = t.graveyard(P1, "Metalwork Colossus");
    assert!(!can_activate(&mut t, P1, stone));
    assert!(can_activate(&mut t, P1, colossus));
    // Karn's -2: a face-down exiled Mind Stone isn't offered; a face-up one is.
    let mut t = TestGame::new(2);
    let karn = t.battlefield(P0, "Karn, the Great Creator");
    let hidden = t.exile(P0, "Mind Stone");
    t.g.objects[hidden.0 as usize].face_down = true;
    let shown = t.exile(P0, "Bonesplitter");
    t.g.recompute();
    let from = t.asked().len();
    t.answer_choose(P0, &[obj(shown)]);
    act(&mut t, P0, karn, "from outside the game", &[]).expect("-2");
    t.resolve_all();
    assert!(!was_offered(&t, P0, from, hidden));
    assert!(t.in_hand(P0, "Bonesplitter"));
    assert_eq!(t.zone(hidden), Zone::Exile);
}

#[test]
fn karn_legacy_reforged_adds_mana_with_a_trigger_counted_on_resolution() {
    cr!("605.1a", "603.3", "608.2h");
    ruling!(
        "Karn, Legacy Reforged",
        "Karn's last ability isn't a mana ability, even though it adds mana. It uses the stack and it can be responded to. Use the number of artifacts you control as the ability resolves to determine how much mana to add."
    );
    supported("Karn, Legacy Reforged");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Karn, Legacy Reforged");
    crate::r_s04_common::next_upkeep(&mut t, P0);
    t.settle();
    assert_eq!(t.stack_len(), 1, "on the stack");
    assert_eq!(pool(&t, P0), 0);
    // In response, a second artifact enters: {C}{C}.
    t.enter(P0, "Mind Stone");
    t.resolve_all();
    assert_eq!(pool(&t, P0), 2);
}

#[test]
fn monstrous_war_leechs_mill_is_a_replacement_effect() {
    cr!("614.1c", "614.12", "603.6a");
    ruling!(
        "Monstrous War-Leech",
        "Monstrous War-Leech's second ability is a replacement effect, not a triggered ability. It happens at the same time that Monstrous War-Leech enters the battlefield, it doesn't use the stack, and it can't be responded to."
    );
    supported("Monstrous War-Leech");
    let mut t = TestGame::new(2);
    stack_library(&mut t, P0, &["Forest", "Island", "Swamp", "Hill Giant"]);
    let leech = t.hand(P0, "Monstrous War-Leech");
    mana(&mut t, P0, "3BU");
    t.cast(P0, leech).kicked(true).go();
    t.g.settle();
    t.g.resolve_top();
    // Right after the spell resolved, before anything could be put on the stack: the
    // cards are already milled and nothing is waiting on the stack.
    assert_eq!(t.graveyard_size(P0), 4);
    assert_eq!(t.stack_len(), 0);
    t.settle();
    assert_eq!(t.stack_len(), 0, "no trigger");
    let leech = t.named_on_battlefield("Monstrous War-Leech")[0];
    assert_eq!(t.pt(leech), (4, 4));
}

#[test]
fn narsets_minus_two_targets_only_with_its_reflexive_trigger() {
    cr!("603.12", "115.1", "602.2b");
    ruling!(
        "Narset of the Ancient Way",
        "Narset's second ability goes on the stack without a target. While that ability is resolving, you draw a card and may choose to discard a card. When you discard a nonland card this way, the reflexive triggered ability triggers and you pick a target to be dealt damage. This is different from effects that say \"If you do . . .\" in that you choose a target after looking at the card you draw and players can respond to the reflexive triggered ability knowing how much damage will be dealt."
    );
    supported("Narset of the Ancient Way");
    let mut t = TestGame::new(2);
    let narset = t.battlefield(P0, "Narset of the Ancient Way");
    let giant_card = t.library_top(P0, "Hill Giant");
    let victim = t.battlefield(P1, "Indomitable Ancients");
    let from = t.asked().len();
    act(&mut t, P0, narset, "Draw a card", &[]).expect("-2");
    assert!(target_candidates(&t, P0, from).is_empty(), "no target on activation");
    t.answer_yes(P0, true);
    t.answer_choose_moved(P0, &[obj(giant_card)]);
    t.answer_targets(P0, &[obj(victim)]);
    t.resolve();
    assert!(t.in_graveyard(P0, "Hill Giant"));
    assert_eq!(t.stack_len(), 1, "the reflexive trigger, with its target chosen now");
    let cands = target_candidates(&t, P0, from);
    assert!(cands.iter().any(|c| c.contains(&obj(victim))));
    assert!(t.on_battlefield(victim), "players could respond here");
    t.resolve_all();
    assert_eq!(dmg(&t, victim), 4);
}

#[test]
fn stronghold_arena_triggers_once_per_opponent_in_each_combat_damage_step() {
    cr!("510.4", "603.2c", "702.4b");
    ruling!(
        "Stronghold Arena",
        "Normally, combat damage is dealt all at the same time. In that case, Stronghold Arena's last ability triggers once for each opponent that creatures you control dealt combat damage to, regardless of how many creatures were dealing that damage. If any of those creatures have double strike or first strike, the ability will trigger once for each opponent dealt damage in each combat damage step."
    );
    supported("Stronghold Arena");
    supported("Fencing Ace");
    // Two creatures without first strike: one trigger (lands on top: no life lost).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Stronghold Arena");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    stack_library(&mut t, P0, &["Forest", "Island", "Swamp"]);
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.attack(&[(bears, pl(P1)), (giant, pl(P1))], &[]);
    t.resolve_all();
    let asked: Vec<Decision> = asked_of(&t, P0, from)
        .into_iter()
        .filter(|d| matches!(d, Decision::YesNo { .. }))
        .collect();
    assert_eq!(asked.len(), 1, "one trigger: {asked:?}");
    // A double striker and a Bear: one trigger in the first-strike step, one in the
    // regular step.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Stronghold Arena");
    let ace = t.battlefield(P0, "Fencing Ace");
    let bears = t.battlefield(P0, "Grizzly Bears");
    stack_library(&mut t, P0, &["Forest", "Island", "Swamp"]);
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.attack(&[(ace, pl(P1)), (bears, pl(P1))], &[]);
    t.resolve_all();
    let asked: Vec<Decision> = asked_of(&t, P0, from)
        .into_iter()
        .filter(|d| matches!(d, Decision::YesNo { .. }))
        .collect();
    assert_eq!(asked.len(), 2, "{asked:?}");
    assert_eq!(t.hand_size(P0), 2, "two cards put into hand");
}

#[test]
fn argentum_masticore_stays_when_the_discard_has_no_legal_target() {
    cr!("603.3d", "603.12", "701.21a");
    ruling!(
        "Argentum Masticore",
        "Notably, this means that you may discard a card to keep Argentum Masticore on the battlefield even if there isn't a legal target for that second triggered ability. In that case, no players may respond to the second triggered ability and it does not resolve."
    );
    supported("Argentum Masticore");
    // P0 discards a Forest (mana value 0); P1's only nonland permanent is Grizzly Bears
    // (2): the reflexive trigger has no legal target and never goes on the stack.
    let mut t = TestGame::new(2);
    let masticore = t.battlefield(P0, "Argentum Masticore");
    let forest = t.hand(P0, "Forest");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(forest)]);
    crate::r_s04_common::next_upkeep(&mut t, P0);
    t.resolve();
    assert!(t.in_graveyard(P0, "Forest"));
    assert!(t.on_battlefield(masticore));
    assert_eq!(t.stack_len(), 0, "no reflexive trigger on the stack");
    assert!(target_candidates(&t, P0, from).is_empty());
    assert!(t.on_battlefield(bears));
    // Discarding a Hill Giant (4): the Bears are destroyed.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Argentum Masticore");
    let giant = t.hand(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(giant)]);
    t.answer_targets(P0, &[obj(bears)]);
    crate::r_s04_common::next_upkeep(&mut t, P0);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn nylas_end_step_trigger_has_an_intervening_if() {
    cr!("603.4");
    ruling!(
        "Nyla, Shirshu Sleuth",
        "Nyla, Shirshu Sleuth's second ability has an \"intervening 'if' clause.\" That means the ability won't trigger at all unless you control no Clues at the beginning of your end step, and the ability will do nothing if you control a Clue by the time it resolves."
    );
    supported("Nyla, Shirshu Sleuth");
    // Nyla exiles a Hill Giant (4 Clues): no trigger at the end step.
    let mut t = TestGame::new(2);
    let giant = t.graveyard(P0, "Hill Giant");
    t.answer_targets(P0, &[obj(giant)]);
    t.enter(P0, "Nyla, Shirshu Sleuth");
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 4);
    assert_eq!(t.life(P0), 16);
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 0, "no trigger with Clues");
    assert!(t.in_exile("Hill Giant"));
    // Nyla exiles an Ornithopter (0 Clues): the trigger targets it; a Clue created in
    // response makes it do nothing. Without a response it returns the card to hand.
    for respond in [true, false] {
        let mut t = TestGame::new(2);
        let thopter = t.graveyard(P0, "Ornithopter");
        t.answer_targets(P0, &[obj(thopter)]);
        t.enter(P0, "Nyla, Shirshu Sleuth");
        t.resolve_all();
        assert!(tokens(&t, P0).is_empty());
        t.answer_targets(P0, &[obj(thopter)]);
        t.advance_to(P0, Step::End);
        t.settle();
        assert_eq!(t.stack_len(), 1, "respond: {respond}");
        if respond {
            create_token(&mut t, P0, "Clue");
        }
        t.resolve_all();
        assert_eq!(t.in_hand(P0, "Ornithopter"), !respond, "respond: {respond}");
        assert_eq!(t.in_exile("Ornithopter"), respond, "respond: {respond}");
    }
}

#[test]
fn nobody_can_respond_once_a_reveal_and_damage_spell_starts_resolving() {
    cr!("608.2", "117.3c", "608.2c");
    ruling!(
        "Cerebral Eruption",
        "Once the card is revealed, it’s too late for players to respond. The targeted opponent can’t, for example, see how much damage will be dealt, then cast a damage prevention spell or activate a regeneration ability. All such actions must be done before Cerebral Eruption starts to resolve."
    );
    ruling!(
        "Explosive Revelation",
        "Once you start revealing cards from your library, it’s too late for players to respond. If you target a creature, for example, its controller can’t wait to see how much damage will be dealt to it before casting a damage prevention spell, activating a regeneration ability, sacrificing it for some benefit, or anything else. Any such actions must be done before Explosive Revelation starts to resolve."
    );
    supported("Cerebral Eruption");
    supported("Explosive Revelation");
    // Cerebral Eruption at P1, whose top card is a Hill Giant and who holds Healing
    // Salve with mana: P1 gets priority with the spell on the stack (before the reveal),
    // and no decision at all while it resolves; the Bears are dealt 4 and die.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Grizzly Bears");
    stack_library(&mut t, P1, &["Hill Giant", "Forest"]);
    t.hand(P1, "Healing Salve");
    mana(&mut t, P1, "W");
    cast(&mut t, P0, "Cerebral Eruption", "2RR", &[pl(P1)]);
    let from = t.asked().len();
    t.resolve();
    assert!(asked_of(&t, P1, from).is_empty(), "no decision of P1's during resolution");
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(t.life(P1), 16);
    // Explosive Revelation at P1's Grizzly Bears with a Craw Wurm (6) under a Forest.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    stack_library(&mut t, P0, &["Forest", "Craw Wurm", "Island"]);
    t.hand(P1, "Healing Salve");
    mana(&mut t, P1, "W");
    cast(&mut t, P0, "Explosive Revelation", "3RR", &[obj(bears)]);
    let from = t.asked().len();
    t.resolve();
    assert!(asked_of(&t, P1, from).is_empty());
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(t.in_hand(P0, "Craw Wurm"));
}

#[test]
fn riveteers_ascendancy_returns_a_creature_only_once_each_turn() {
    cr!("603.2", "603.12");
    ruling!(
        "Riveteers Ascendancy",
        "Once you have chosen to return a creature to the battlefield, further instances of sacrificing creatures the same turn will not cause the ability to trigger."
    );
    supported("Riveteers Ascendancy");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Riveteers Ascendancy");
    let giant = t.battlefield(P0, "Hill Giant");
    let wurm = t.battlefield(P0, "Craw Wurm");
    let bears = t.graveyard(P0, "Grizzly Bears");
    let thopter = t.graveyard(P0, "Ornithopter");
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(bears)]);
    sacrifice(&mut t, P0, giant);
    t.resolve_all();
    let back = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(back.len(), 1);
    assert!(t.obj_now(back[0]).tapped);
    // A second sacrifice this turn: no trigger.
    sacrifice(&mut t, P0, wurm);
    assert_eq!(t.stack_len(), 0);
    t.resolve_all();
    assert_eq!(t.zone(thopter), Zone::Graveyard(P0));
    // Next turn it works again.
    t.advance_to(P1, Step::PrecombatMain);
    t.advance_to(P0, Step::PrecombatMain);
    let giant = t.battlefield(P0, "Hill Giant");
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(thopter)]);
    sacrifice(&mut t, P0, giant);
    t.resolve_all();
    assert_eq!(count_named(&t, "Ornithopter"), 1);
}

#[test]
fn orah_can_target_a_cleric_that_died_at_the_same_time() {
    cr!("603.10a", "603.6c", "400.7");
    ruling!(
        "Orah, Skyclave Hierophant",
        "Orah's ability can target a Cleric card that was put into your graveyard at the same time as the Cleric whose death caused the ability to trigger."
    );
    supported("Orah, Skyclave Hierophant");
    supported("Benevolent Bodyguard");
    supported("Samite Healer");
    // Samite Healer (2) and Benevolent Bodyguard (1) die together: the Healer's trigger
    // can return the Bodyguard card.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Orah, Skyclave Hierophant");
    let healer = t.battlefield(P0, "Samite Healer");
    let guard = t.battlefield(P0, "Benevolent Bodyguard");
    let from = t.asked().len();
    t.answer_targets(P0, &[obj(guard)]);
    crate::r_p075_common::leave_together(&mut t, &[healer, guard]);
    t.resolve_all();
    assert_eq!(target_candidate_names(&t, P0, from), vec!["Benevolent Bodyguard".to_string()]);
    assert_eq!(count_named(&t, "Benevolent Bodyguard"), 1);
}

#[test]
fn phyrexian_devourers_state_trigger() {
    cr!("603.8", "603.3", "608.2h");
    ruling!(
        "Phyrexian Devourer",
        "Phyrexian Devourer’s first ability continually checks its power as a state trigger. Once it triggers, it won’t trigger again as long as the ability is on the stack. If the ability is countered and the trigger condition is still true, it will immediately trigger again. If the ability resolves, you’ll have to sacrifice Phyrexian Devourer even if its power has been reduced to 6 or less by then."
    );
    supported("Phyrexian Devourer");
    supported("Stifle");
    let mut t = TestGame::new(2);
    // A 1/1: six counters make its power 7.
    let dev = t.battlefield(P0, "Phyrexian Devourer");
    put(&mut t, dev, counters::PLUS1, 6);
    assert_eq!(t.pt(dev).0, 7);
    assert_eq!(t.stack_len(), 1, "triggered");
    put(&mut t, dev, counters::PLUS1, 1);
    assert_eq!(t.stack_len(), 1, "not again while on the stack");
    // Stifled: it triggers again at once (Stifle resolves; the Devourer is still there).
    let trigger = *t.g.stack.last().unwrap();
    cast(&mut t, P0, "Stifle", "U", &[obj(trigger)]);
    t.resolve();
    assert!(t.on_battlefield(dev));
    assert_eq!(t.stack_len(), 1, "triggered again");
    assert_ne!(*t.g.stack.last().unwrap(), trigger);
    // Its power drops to 5 before the ability resolves: sacrificed all the same.
    crate::r_p117_common::minus_power(&mut t, dev, 3);
    assert_eq!(t.pt(dev).0, 5);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Phyrexian Devourer"));
}

#[test]
fn arni_metalbrow_triggers_again_for_a_creature_put_onto_the_battlefield_attacking() {
    cr!("508.4", "603.2", "202.3");
    ruling!(
        "Arni Metalbrow",
        "Putting a creature onto the battlefield attacking as the ability resolves will cause the ability to trigger again. This instance of the ability will consider the mana value of the new creature that entered the battlefield attacking."
    );
    supported("Arni Metalbrow");
    // Hill Giant (4) attacks: pay {1}{R}, Grizzly Bears (2) enters attacking; its
    // trigger: pay again, Ornithopter (0) enters attacking; the Ornithopter's trigger has
    // nothing with a lesser mana value.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Arni Metalbrow");
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.hand(P0, "Grizzly Bears");
    let thopter = t.hand(P0, "Ornithopter");
    t.lands(P0, "Mountain", 6);
    attack_with(&mut t, &[(giant, pl(P1))]);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(bears)]);
    t.resolve();
    assert_eq!(t.stack_len(), 1, "the Bears' trigger");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(thopter)]);
    t.resolve();
    assert_eq!(t.stack_len(), 1, "the Ornithopter's trigger");
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(entity_candidates(&t, P0, from).iter().all(|c| c.is_empty()));
    assert_eq!(t.g.combat.as_ref().map(|c| c.attackers.len()), Some(3));
    assert!(t.obj_now(bears).tapped && t.obj_now(thopter).tapped);
}

#[test]
fn rage_extractor_triggers_for_any_phyrexian_mana_symbol_however_paid() {
    cr!("107.4f", "202.3g", "603.2");
    ruling!(
        "Rage Extractor",
        "Rage Extractor's ability checks for any spell you cast with any Phyrexian mana symbol in its mana cost, regardless of that symbol's color or how the cost was paid."
    );
    supported("Rage Extractor");
    supported("Gut Shot");
    // Gut Shot ({R/P}) paid with 2 life, then with {R}: 1 damage from the Extractor each
    // time; Vault Skirge ({1}{B/P}, mana value 2) paid with mana: 2.
    for (card, cost, life_paid, dealt) in [
        ("Gut Shot", "", 2, 1),
        ("Gut Shot", "R", 0, 1),
        ("Vault Skirge", "1B", 0, 2),
    ] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Rage Extractor");
        let c = t.hand(P0, card);
        mana(&mut t, P0, cost);
        let targets: &[Entity] = if card == "Gut Shot" { &[pl(P0)] } else { &[] };
        t.answer_targets(P0, &[pl(P1)]);
        let mut b = t.cast(P0, c);
        for e in targets {
            b = b.target(*e);
        }
        b.go();
        t.resolve_all();
        assert_eq!(t.life(P1), 20 - dealt, "{card} for {cost:?}");
        assert_eq!(t.life(P0), 20 - life_paid - if card == "Gut Shot" { 1 } else { 0 }, "{card} for {cost:?}");
    }
}

#[test]
fn the_lord_of_pain_lets_life_gain_spells_resolve_without_the_life() {
    cr!("119.8", "608.2c");
    ruling!(
        "The Lord of Pain",
        "Spells and abilities that cause opponents to gain life still resolve while The Lord of Pain is on the battlefield. They won't gain life, but any other effects of that spell or ability will still happen."
    );
    supported("The Lord of Pain");
    supported("Revitalize");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "The Lord of Pain");
    t.set_step(P1, Step::PrecombatMain);
    let hand = t.hand_size(P1);
    t.answer_targets(P0, &[pl(P0)]);
    cast_resolve(&mut t, P1, "Revitalize", "1W", &[]);
    assert_eq!(t.life(P1), 20, "no life gained");
    assert_eq!(t.life(P0), 18, "The Lord of Pain's trigger hits the other player");
    assert_eq!(t.hand_size(P1), hand + 1, "the card is still drawn");
}

/// Captain Rex Nebula's beginning-of-combat trigger makes `target` (P0's) a Vehicle;
/// the game is in P0's beginning of combat step with the trigger resolved.
fn rex_makes_a_vehicle(t: &mut TestGame, target: ObjectId) {
    t.battlefield(P0, "Captain Rex Nebula");
    t.answer_targets(P0, &[obj(target)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    let o = t.obj_now(target);
    assert!(o.is(CardType::Artifact) && o.chars.has_subtype("Vehicle"));
}

#[test]
fn captain_rex_nebulas_vehicle_keeps_its_abilities_and_crashes_on_any_damage_once_per_instance() {
    cr!("613.1d", "613.1f", "603.2c", "702.122a");
    ruling!(
        "Captain Rex Nebula",
        "The Vehicle will retain any abilities it previously had, as well as gaining crew 2 and the ability to crash into stuff."
    );
    ruling!(
        "Captain Rex Nebula",
        "The Crash Land ability will trigger whenever the Vehicle deals any damage, not only combat damage."
    );
    ruling!(
        "Captain Rex Nebula",
        "The Crash Land ability will trigger once for each instance of damage, no matter how many players or permanents the Vehicle deals damage to. For example, if the Vehicle is blocked by two creatures and it deals damage to each of them at the same time, the Crash Land ability will trigger only once."
    );
    supported("Captain Rex Nebula");
    supported("Prodigal Sorcerer");
    // Prodigal Sorcerer (3) becomes a 3/3 Vehicle with crew 2 that still has its ping
    // ability; the ping is noncombat damage: with a 3 rolled, it crashes (sacrificed, 3
    // damage to P1).
    let mut t = TestGame::new(2);
    let tim = t.battlefield(P0, "Prodigal Sorcerer");
    rex_makes_a_vehicle(&mut t, tim);
    assert!(t.obj_now(tim).chars.has_keyword(KeywordKind::Crew));
    assert!(!t.obj_now(tim).is(CardType::Creature), "not crewed");
    t.g.dice.loaded.push_back(3);
    t.answer_targets(P0, &[pl(P1)]);
    act(&mut t, P0, tim, "deals 1 damage", &[pl(P1)]).expect("still has its ability");
    t.resolve_all();
    assert_eq!(t.life(P1), 20 - 1 - 3);
    assert!(t.in_graveyard(P0, "Prodigal Sorcerer"));
    // Crewed by a Hill Giant and blocked by two Llanowar Elves: combat damage to both at
    // once is one instance, so Crash Land triggers once (one die is rolled).
    let mut t = TestGame::new(2);
    let tim = t.battlefield(P0, "Prodigal Sorcerer");
    let giant = t.battlefield(P0, "Hill Giant");
    let e1 = t.battlefield(P1, "Llanowar Elves");
    let e2 = t.battlefield(P1, "Llanowar Elves");
    rex_makes_a_vehicle(&mut t, tim);
    t.answer_choose(P0, &[obj(giant)]);
    crate::r_s06_common::activate_containing(&mut t, P0, tim, "Crew").expect("crew 2");
    t.resolve_all();
    assert!(t.obj_now(tim).is(CardType::Creature));
    assert_eq!(t.pt(tim), (3, 3));
    t.g.dice.loaded.push_back(3);
    t.g.dice.loaded.push_back(3);
    t.answer_targets(P0, &[pl(P1)]);
    attack_with(&mut t, &[(tim, pl(P1))]);
    block_and_finish(&mut t, P1, &[(e1, tim), (e2, tim)]);
    assert!(!t.on_battlefield(e1) && !t.on_battlefield(e2));
    assert_eq!(t.g.dice.loaded.len(), 1, "one roll");
    assert_eq!(t.life(P1), 17, "one crash");
}
