//! Rulings batch P149 — triggered abilities: on damage (Farsight Mask, Nafs Asp, Sabertooth
//! Cobra), on draws (The Unagi of Kyoshi Island, Smothering Tithe, Phyrexian Tyranny), on
//! the second spell (Saruman), intervening "if" clauses (Cackling Prowler, Imoen), extra
//! triggers (Felix Five-Boots), and ward (Iymrith, Codsworth's commanders).

use crate::r_p149_common::*;
use mtg_engine::decision::Decision;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

// --- Damage --------------------------------------------------------------------------------

#[test]
fn farsight_mask_draws_one_card_per_damage_event() {
    cr!("603.2c", "120.3");
    ruling!(
        "Farsight Mask",
        "You draw no more than one card each time a source an opponent controls damages you, no matter how much damage the source deals."
    );
    supported("Farsight Mask");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Farsight Mask");
    let giant = t.battlefield(P1, "Craw Wurm");
    let hand = t.hand_size(P0);
    t.answer_yes(P0, true);
    t.g.deal_damage(giant, pl(P0), 6, false);
    t.g.flush_events();
    t.settle();
    assert_eq!(triggers_on_stack(&t, "you may draw a card"), 1);
    t.resolve_all();
    assert_eq!(t.life(P0), 14);
    assert_eq!(t.hand_size(P0), hand + 1);
}

/// P0's `name` ("Whenever this creature deals damage to a player, ...") deals 1 damage to
/// P1, and the trigger resolves.
fn hits(t: &mut TestGame, name: &str) -> ObjectId {
    supported(name);
    let id = t.battlefield(P0, name);
    t.g.deal_damage(id, pl(P1), 1, false);
    t.g.flush_events();
    t.resolve_all();
    id
}

#[test]
fn nafs_asp_can_be_paid_any_time_before_that_players_draw_step() {
    cr!("116.2c", "603.7a");
    ruling!(
        "Nafs Asp",
        "The player can pay the {1} mana at any time after damage is done before the draw step of that player’s turn. People commonly pay during upkeep."
    );
    // Right after the damage, in P0's turn: paid, no life lost.
    let mut t = TestGame::new(2);
    hits(&mut t, "Nafs Asp");
    lots(&mut t, P0);
    lots(&mut t, P1);
    assert_eq!(offers(&mut t, P1).len(), 1);
    assert!(offers(&mut t, P0).is_empty(), "only that player pays");
    pay_offer(&mut t, P1);
    t.advance_to(P1, Step::Draw);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    // In P1's upkeep: also in time.
    let mut t = TestGame::new(2);
    hits(&mut t, "Nafs Asp");
    t.advance_to(P1, Step::Upkeep);
    lots(&mut t, P1);
    assert_eq!(offers(&mut t, P1).len(), 1);
    pay_offer(&mut t, P1);
    t.advance_to(P1, Step::Draw);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    // Not paid: 1 life at the beginning of the draw step.
    let mut t = TestGame::new(2);
    hits(&mut t, "Nafs Asp");
    t.advance_to(P1, Step::Draw);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

#[test]
fn nafs_asp_life_loss_isnt_damage_and_doesnt_retrigger() {
    cr!("119.3", "120.1");
    ruling!(
        "Nafs Asp",
        "The ability does not cause itself to trigger again. It cause loss of life, and not damage."
    );
    let mut t = TestGame::new(2);
    hits(&mut t, "Nafs Asp");
    t.advance_to(P1, Step::Draw);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.stack_len(), 0);
    // No new pay-or-lose offer, and nothing more at P1's next draw step.
    lots(&mut t, P1);
    assert!(offers(&mut t, P1).is_empty());
    t.advance_to(P0, Step::Draw);
    t.advance_to(P1, Step::Draw);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

#[test]
fn nafs_asp_damage_redirected_to_its_controller_still_triggers() {
    cr!("614.9", "603.2");
    ruling!(
        "Nafs Asp",
        "If its damage gets redirected to its controller, it will still trigger the ability."
    );
    supported("Harm's Way");
    let mut t = TestGame::new(2);
    let asp = t.battlefield(P0, "Nafs Asp");
    // P1's Harm's Way: the Asp's damage to P1 is dealt to P0 instead.
    t.set_step(P1, Step::PrecombatMain);
    t.answer_choose(P1, &[obj(asp)]);
    cast_at(&mut t, P1, "Harm's Way", &[pl(P0)]);
    t.resolve_all();
    t.set_step(P0, Step::PrecombatMain);
    t.g.deal_damage(asp, pl(P1), 1, false);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.life(P0), 19);
    // The trigger saw the damage to P0: P0 pays or loses 1 life in their next draw step.
    lots(&mut t, P0);
    lots(&mut t, P1);
    assert_eq!(offers(&mut t, P0).len(), 1);
    assert!(offers(&mut t, P1).is_empty());
    t.advance_to(P1, Step::Draw);
    t.advance_to(P0, Step::Draw);
    t.resolve_all();
    assert_eq!(t.life(P0), 18);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn sabertooth_cobra_second_poison_counter_without_the_cobra() {
    cr!("603.7a", "603.7b");
    ruling!(
        "Sabertooth Cobra",
        "The ability still gives a poison counter even if Sabertooth Cobra leaves the battlefield before the next upkeep."
    );
    let mut t = TestGame::new(2);
    let cobra = hits(&mut t, "Sabertooth Cobra");
    assert_eq!(poison(&t, P1), 1);
    kill(&mut t, cobra);
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert_eq!(poison(&t, P1), 2);
}

// --- Draws ---------------------------------------------------------------------------------

#[test]
fn unagi_triggers_after_the_draw_effect_finishes_resolving() {
    cr!("603.3", "121.2");
    ruling!(
        "The Unagi of Kyoshi Island",
        "If an effect instructs your opponent to draw multiple cards, the ability triggers after they draw whichever is the second one for the turn. It goes on the stack after they've drawn and looked at all the cards and finished resolving the spell or ability that caused them to draw cards."
    );
    supported("The Unagi of Kyoshi Island");
    supported("Divination");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "The Unagi of Kyoshi Island");
    t.set_step(P1, Step::PrecombatMain);
    let hand = t.hand_size(P0);
    let p1_hand = t.hand_size(P1);
    cast_at(&mut t, P1, "Divination", &[]);
    t.resolve();
    assert!(t.in_graveyard(P1, "Divination"));
    assert_eq!(t.hand_size(P1), p1_hand + 2);
    assert_eq!(triggers_on_stack(&t, "second card"), 1);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
}

#[test]
fn unagi_triggers_once_per_opponent_each_turn_on_the_second_draw() {
    cr!("603.2", "603.10");
    ruling!(
        "The Unagi of Kyoshi Island",
        "The last ability can trigger only once each turn for each opponent. It doesn't matter whether The Unagi of Kyoshi Island was on the battlefield when the first card was drawn. If it's not on the battlefield when the second card is drawn, the ability can't trigger at all that turn. It won't trigger when the third or fourth card is drawn."
    );
    // The first draw before the Unagi arrives: the second still triggers it.
    let mut t = TestGame::new(2);
    draw(&mut t, P1, 1);
    t.battlefield(P0, "The Unagi of Kyoshi Island");
    draw(&mut t, P1, 1);
    assert_eq!(triggers_on_stack(&t, "second card"), 1);
    t.resolve_all();
    draw(&mut t, P1, 2);
    assert_eq!(t.stack_len(), 0, "not on the third or fourth");
    // Not there for the second: never this turn.
    let mut t = TestGame::new(2);
    draw(&mut t, P1, 2);
    t.battlefield(P0, "The Unagi of Kyoshi Island");
    draw(&mut t, P1, 1);
    draw(&mut t, P1, 1);
    assert_eq!(t.stack_len(), 0);
    // Each opponent: P1's and P2's second cards each trigger it; P0's own don't.
    let mut t = TestGame::new(3);
    t.battlefield(P0, "The Unagi of Kyoshi Island");
    draw(&mut t, P0, 2);
    draw(&mut t, P1, 2);
    draw(&mut t, P2, 1);
    assert_eq!(triggers_on_stack(&t, "second card"), 1);
    draw(&mut t, P2, 1);
    assert_eq!(triggers_on_stack(&t, "second card"), 2);
}

#[test]
fn unagi_cards_put_into_hand_arent_drawn() {
    cr!("121.1", "121.2");
    ruling!(
        "The Unagi of Kyoshi Island",
        "If a card or ability instructs your opponent to put cards into their hand without specifically using the word \"draw,\" it's not a card drawn."
    );
    supported("Regrowth");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "The Unagi of Kyoshi Island");
    t.set_step(P1, Step::PrecombatMain);
    draw(&mut t, P1, 1);
    // Regrowth ("Return target card from your graveyard to your hand."): not a draw.
    let bears = t.graveyard(P1, "Grizzly Bears");
    cast_at(&mut t, P1, "Regrowth", &[obj(bears)]);
    t.resolve_all();
    assert!(t.in_hand(P1, "Grizzly Bears"));
    assert_eq!(triggers_on_stack(&t, "second card"), 0);
    let hand = t.hand_size(P0);
    draw(&mut t, P1, 1);
    assert_eq!(triggers_on_stack(&t, "second card"), 1);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
}

#[test]
fn smothering_tithe_both_cards_are_drawn_before_paying() {
    cr!("603.3", "121.2");
    ruling!(
        "Smothering Tithe",
        "If an opponent is instructed to draw multiple cards, that player draws all of them before deciding how many times to pay as the multiple triggered abilities from Smothering Tithe resolve."
    );
    supported("Smothering Tithe");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Smothering Tithe");
    t.set_step(P1, Step::PrecombatMain);
    let p1_hand = t.hand_size(P1);
    cast_at(&mut t, P1, "Divination", &[]);
    let seen = crate::r_s01_common::watch(
        &mut t,
        P1,
        |d| matches!(d, Decision::YesNo { .. }),
        |g| g.player(P1).hand.len(),
    );
    t.resolve();
    assert_eq!(t.hand_size(P1), p1_hand + 2);
    assert_eq!(triggers_on_stack(&t, "may pay {2}"), 2);
    // Pays for one, not the other.
    t.answer_yes(P1, true);
    t.answer_yes(P1, false);
    t.resolve_all();
    assert_eq!(*seen.lock().unwrap(), vec![p1_hand + 2, p1_hand + 2]);
    assert_eq!(
        crate::r_s01_common::with_subtype(&t, P0, "Treasure").len(),
        1
    );
}

#[test]
fn phyrexian_tyranny_triggers_for_each_card_drawn() {
    cr!("121.2", "603.2c");
    ruling!(
        "Phyrexian Tyranny",
        "If more than one card is drawn at a time, this triggers for each card drawn."
    );
    supported("Phyrexian Tyranny");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Phyrexian Tyranny");
    draw(&mut t, P1, 2);
    assert_eq!(triggers_on_stack(&t, "loses 2 life"), 2);
    t.answer_yes(P1, false);
    t.answer_yes(P1, false);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
}

// --- The second spell ------------------------------------------------------------------------

#[test]
fn saruman_counts_spells_cast_before_it_including_itself() {
    cr!("603.2", "601.2i");
    ruling!(
        "Saruman the White",
        "Spells that were cast before Saruman the White count. If Saruman the White was the first spell you cast this turn, the next spell you cast this turn is your second spell."
    );
    ruling!(
        "Storm of Saruman",
        "Spells that were cast before Storm of Saruman count. If Storm of Saruman was the first spell you cast this turn, the next spell you cast this turn is your second spell."
    );
    supported("Saruman the White");
    supported("Storm of Saruman");
    // Saruman the White, the turn's first spell; Shock, the second: amass Orcs 2.
    let mut t = TestGame::new(2);
    cast_at(&mut t, P0, "Saruman the White", &[]);
    t.resolve_all();
    cast_at(&mut t, P0, "Shock", &[pl(P1)]);
    t.resolve_all();
    let army = crate::r_s01_common::with_subtype(&t, P0, "Army");
    assert_eq!(army.len(), 1);
    assert_eq!(t.counters(army[0], counters::PLUS1), 2);
    // Storm of Saruman, the first spell; Shock, the second: copied (the copy keeping its
    // target).
    let mut t = TestGame::new(2);
    cast_at(&mut t, P0, "Storm of Saruman", &[]);
    t.resolve_all();
    t.answer_yes(P0, false);
    let from = t.asked().len();
    cast_at(&mut t, P0, "Shock", &[pl(P1)]);
    t.resolve_all();
    fn new_targets(d: &Decision) -> bool {
        let prompt = "Choose new targets for the copy?";
        matches!(d, Decision::YesNo { prompt: p, .. } if p == prompt)
    }
    assert_eq!(asked_who(&t, from, new_targets), vec![P0]);
    assert_eq!(t.life(P1), 16);
    // A spell cast before it counts: Shock first, Saruman second (its ability isn't on
    // the battlefield to see that), then a third spell doesn't trigger it.
    let mut t = TestGame::new(2);
    cast_at(&mut t, P0, "Shock", &[pl(P1)]);
    t.resolve_all();
    cast_at(&mut t, P0, "Saruman the White", &[]);
    t.resolve_all();
    cast_at(&mut t, P0, "Shock", &[pl(P1)]);
    t.resolve_all();
    assert!(crate::r_s01_common::with_subtype(&t, P0, "Army").is_empty());
}

// --- Intervening "if" clauses ----------------------------------------------------------------

#[test]
fn cackling_prowler_doesnt_trigger_if_no_creature_died() {
    cr!("603.4");
    ruling!(
        "Cackling Prowler",
        "Cackling Prowler's last ability will check as the end step starts to see if a creature died this turn. If none did, the ability won't trigger at all."
    );
    supported("Cackling Prowler");
    let mut t = TestGame::new(2);
    let prowler = t.battlefield(P0, "Cackling Prowler");
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 0);
    t.resolve_all();
    assert_eq!(t.counters(prowler, counters::PLUS1), 0);
    // A creature died: it triggers.
    let mut t = TestGame::new(2);
    let prowler = t.battlefield(P0, "Cackling Prowler");
    let bears = t.battlefield(P1, "Grizzly Bears");
    kill(&mut t, bears);
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.counters(prowler, counters::PLUS1), 1);
}

#[test]
fn imoen_checks_the_initiative_as_the_end_step_begins_and_on_resolution() {
    cr!("603.4", "726.1");
    ruling!(
        "Imoen, Mystic Trickster",
        "Imoen's second ability triggers only if you have the initiative as your end step begins. If the ability does trigger, it will check if you have the initiative again as it tries to resolve. If you don't still have the initiative at that time, the ability won't resolve and none of its effects will happen. Notably, if you don't have the initiative as your end step begins but you have completed a dungeon, the ability won't trigger and you won't draw any cards."
    );
    supported("Imoen, Mystic Trickster");
    // A completed dungeon, no initiative: no trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Imoen, Mystic Trickster");
    t.g.players[0].dungeons_completed = 1;
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 0);
    // The initiative as the step begins, lost before it resolves: nothing.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Imoen, Mystic Trickster");
    t.g.players[0].dungeons_completed = 1;
    t.g.initiative = Some(P0);
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.g.initiative = Some(P1);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
    // Kept: one card, and another for the completed dungeon.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Imoen, Mystic Trickster");
    t.g.players[0].dungeons_completed = 1;
    t.g.initiative = Some(P0);
    t.advance_to(P0, Step::End);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
}

// --- Felix Five-Boots ------------------------------------------------------------------------

/// P0's `attacker` attacks P1 unblocked; the game stops in the combat damage step with
/// the triggers on the stack.
fn hit_with(t: &mut TestGame, attacker: ObjectId) {
    attack_with(t, &[(attacker, pl(P1))]);
    t.answer(
        P1,
        DecisionKind::Blockers,
        mtg_engine::decision::Answer::Blockers(vec![]),
    );
    t.advance_to(P0, Step::CombatDamage);
    t.settle();
}

#[test]
fn felix_doesnt_double_triggers_on_the_results_of_damage() {
    cr!("603.2", "702.15b");
    ruling!(
        "Felix Five-Boots",
        "Felix’s last ability affects only triggered abilities whose trigger conditions refer specifically to damage being dealt. It does not affect triggered abilities that would trigger because of the results of that damage. For example, if you control Felix as well as an Ajani’s Pridemate (“Whenever you gain life, put a +1/+1 counter on Ajani’s Pridemate”) and a creature you control with lifelink deals combat damage to a player, the triggered ability of Ajani’s Pridemate will still trigger only once."
    );
    supported("Felix Five-Boots");
    supported("Coastal Piracy");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Felix Five-Boots");
    let pridemate = t.battlefield(P0, "Ajani's Pridemate");
    t.battlefield(P0, "Coastal Piracy");
    let nighthawk = t.battlefield(P0, "Vampire Nighthawk");
    hit_with(&mut t, nighthawk);
    assert_eq!(triggers_on_stack(&t, "gain life"), 1);
    // Coastal Piracy's damage trigger, twice.
    assert_eq!(triggers_on_stack(&t, "deals combat damage"), 2);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.counters(pridemate, counters::PLUS1), 1);
    assert_eq!(t.life(P0), 22);
}

#[test]
fn felix_extra_instances_choose_targets_and_options_separately() {
    cr!("603.2c", "603.3d", "608.2d");
    ruling!(
        "Felix Five-Boots",
        "Felix’s last ability doesn’t copy the triggered ability; it just causes the ability to trigger an additional time. Any choices made as you put the ability onto the stack, such as modes and targets, are made separately for each instance of the ability. Any choices made on resolution, such as whether to put counters on a permanent, are also made individually."
    );
    supported("Sword of Fire and Ice");
    // Sword of Fire and Ice ("Whenever equipped creature deals combat damage to a player,
    // this Equipment deals 2 damage to any target and you draw a card."): one instance at
    // P1's Hill Giant, the other at P1.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Felix Five-Boots");
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Sword of Fire and Ice", obj(bears));
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[obj(giant)]);
    t.answer_targets(P0, &[pl(P1)]);
    let hand = t.hand_size(P0);
    hit_with(&mut t, bears);
    assert_eq!(triggers_on_stack(&t, "deals combat damage"), 2);
    let targets: Vec<Vec<Entity>> =
        t.g.stack
            .iter()
            .map(|id| crate::r_p016_common::stack_targets(&t, *id))
            .collect();
    assert!(targets.contains(&vec![obj(giant)]) && targets.contains(&vec![pl(P1)]));
    t.resolve_all();
    assert_eq!(crate::r_p003_common::dmg(&t, giant), 2);
    assert_eq!(t.life(P1), 20 - 4 - 2);
    assert_eq!(t.hand_size(P0), hand + 2);
    // Coastal Piracy's "you may draw a card", answered for each instance.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Felix Five-Boots");
    t.battlefield(P0, "Coastal Piracy");
    let bears = t.battlefield(P0, "Grizzly Bears");
    hit_with(&mut t, bears);
    let hand = t.hand_size(P0);
    t.answer_yes(P0, true);
    t.answer_yes(P0, false);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn two_felixes_make_damage_triggers_trigger_three_times() {
    cr!("603.2c", "704.5j");
    ruling!(
        "Felix Five-Boots",
        "If you somehow control two Felixes, a creature you control dealing combat damage to a player causes abilities of permanents you control to trigger three times. A third Felix causes such abilities to trigger four times, a fourth causes such abilities to trigger five times, and so on."
    );
    supported("Mirror Gallery");
    for felixes in [2, 3] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Mirror Gallery");
        for _ in 0..felixes {
            t.battlefield(P0, "Felix Five-Boots");
        }
        t.battlefield(P0, "Coastal Piracy");
        let bears = t.battlefield(P0, "Grizzly Bears");
        hit_with(&mut t, bears);
        assert_eq!(
            triggers_on_stack(&t, "deals combat damage"),
            felixes + 1,
            "{felixes} Felixes"
        );
    }
}

#[test]
fn felix_doesnt_apply_replacement_effects_again() {
    cr!("614.1a", "603.2c");
    ruling!(
        "Felix Five-Boots",
        "Replacement effects are unaffected by Felix’s last ability. For example, if Charging Tuskodon (“If Charging Tuskodon would deal combat damage to a player, it deals double that damage to that player instead”) deals combat damage to a player, Felix’s ability won’t cause the damage it deals to be doubled an additional time."
    );
    supported("Charging Tuskodon");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Felix Five-Boots");
    let tusk = t.battlefield(P0, "Charging Tuskodon");
    let power = t.pt(tusk).0;
    hit_with(&mut t, tusk);
    t.resolve_all();
    assert_eq!(t.life(P1), 20 - 2 * power);
}

// --- Ward ------------------------------------------------------------------------------------

#[test]
fn iymrith_untapped_after_being_targeted_doesnt_trigger_ward() {
    cr!("702.21a", "603.2");
    ruling!(
        "Iymrith, Desert Doom",
        "If a spell or ability an opponent controls targets Iymrith, Desert Doom while it is tapped, finding a way to untap it in response will not cause its ward ability to trigger."
    );
    supported("Iymrith, Desert Doom");
    let mut t = TestGame::new(2);
    let iymrith = t.battlefield(P0, "Iymrith, Desert Doom");
    t.g.tap(iymrith);
    t.set_step(P1, Step::PrecombatMain);
    cast_at(&mut t, P1, "Shock", &[obj(iymrith)]);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.g.untap(iymrith);
    t.g.flush_events();
    t.settle();
    assert_eq!(t.stack_len(), 1, "no ward trigger");
    t.resolve_all();
    assert_eq!(crate::r_p003_common::dmg(&t, iymrith), 2);
    // Untapped when targeted: ward triggers.
    let mut t = TestGame::new(2);
    let iymrith = t.battlefield(P0, "Iymrith, Desert Doom");
    t.set_step(P1, Step::PrecombatMain);
    cast_at(&mut t, P1, "Shock", &[obj(iymrith)]);
    t.settle();
    assert_eq!(t.stack_len(), 2);
}

#[test]
fn codsworth_leaving_doesnt_stop_a_commanders_ward_trigger() {
    cr!("702.21a", "113.7a");
    ruling!(
        "Codsworth, Handy Helper",
        "Once a commander’s ward ability has triggered, causing that commander to lose ward by removing Codsworth won’t affect the ability. The appropriate player will still have to pay {2} or see their spell or ability countered."
    );
    supported("Codsworth, Handy Helper");
    for pays in [false, true] {
        let mut t = crate::r_p114_common::commander_game_n(2);
        let codsworth = t.battlefield(P0, "Codsworth, Handy Helper");
        let commander = crate::r_p114_common::commander_on_battlefield(&mut t, P0, "Hill Giant");
        t.set_step(P1, Step::PrecombatMain);
        cast_at(&mut t, P1, "Shock", &[obj(commander)]);
        t.settle();
        assert_eq!(t.stack_len(), 2, "ward triggered");
        kill(&mut t, codsworth);
        t.answer_yes(P1, pays);
        t.resolve_all();
        assert_eq!(
            crate::r_p003_common::dmg(&t, commander),
            if pays { 2 } else { 0 },
            "pays: {pays}"
        );
        assert!(t.in_graveyard(P1, "Shock"));
    }
}

#[test]
fn aboleth_spawn_copies_triggered_abilities_not_replacement_or_static_ones() {
    cr!("113.3c", "603.1", "614.1c");
    ruling!(
        "Aboleth Spawn",
        "Triggered abilities use the words \"when,\" \"whenever,\" or \"at.\" They're often written as \"[Trigger condition],[effect].\""
    );
    supported("Aboleth Spawn");
    supported("Elvish Visionary");
    supported("Adaptive Automaton");
    // Elvish Visionary ("When this creature enters, draw a card."): copied.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Aboleth Spawn");
    let hand = t.hand_size(P0);
    t.answer_yes(P0, true);
    t.enter(P1, "Elvish Visionary");
    t.g.flush_events();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    // Adaptive Automaton ("As this creature enters, choose a creature type."): not a
    // triggered ability; nothing to copy.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Aboleth Spawn");
    let from = t.asked().len();
    let auto = t.enter(P1, "Adaptive Automaton");
    t.g.flush_events();
    t.settle();
    assert!(t.on_battlefield(auto));
    assert_eq!(t.stack_len(), 0);
    assert!(asked_who(&t, from, |d| matches!(d, Decision::YesNo { .. }))
        .iter()
        .all(|p| *p != P0));
}
