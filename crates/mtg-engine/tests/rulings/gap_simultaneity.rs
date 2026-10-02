//! Simultaneity (gap-simultaneity): when several players perform an instruction at the
//! same time, each makes their choices in APNAP order knowing the earlier ones, then the
//! actions happen at once (CR 101.4, 608.2e, 608.2f): the cards they put onto the
//! battlefield enter together (a Clone can't copy a creature entering with it), the
//! players' discards happen once everyone has chosen, and the events are one batch.

use crate::r_s01_common::*;
use mtg_engine::decision::Decision;
use mtg_engine::events::Event;
use mtg_engine::game::Game;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn is_choose(d: &Decision) -> bool {
    matches!(d, Decision::ChooseEntities { .. })
}

fn is_yes_no(d: &Decision) -> bool {
    matches!(d, Decision::YesNo { .. })
}

/// The objects a player was offered to choose among, for each such decision since `from`.
fn offered(t: &TestGame, p: PlayerId, from: usize) -> Vec<Vec<ObjectId>> {
    t.asked()[from..]
        .iter()
        .filter(|(q, _)| *q == p)
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => {
                Some(candidates.iter().filter_map(|e| e.object()).collect())
            }
            _ => None,
        })
        .collect()
}

/// The choices offered to `p` since `from` among permanents on the battlefield (a Clone's
/// "copy any creature on the battlefield").
fn offered_permanents(t: &TestGame, p: PlayerId, from: usize) -> Vec<Vec<ObjectId>> {
    offered(t, p, from)
        .into_iter()
        .filter(|c| {
            !c.is_empty()
                && c.iter()
                    .all(|o| t.g.obj(*o).zone == Zone::Battlefield)
        })
        .collect()
}

/// The public record of the players who accepted so far.
fn acceptances(g: &Game) -> Vec<String> {
    g.log
        .iter()
        .filter(|l| l.text.ends_with("chooses to take part"))
        .map(|l| l.text.clone())
        .collect()
}

/// The positions in this turn's events of each discard and each draw.
fn discards_and_draws(t: &TestGame, from: usize) -> (Vec<usize>, Vec<usize>) {
    let mut discards = Vec::new();
    let mut draws = Vec::new();
    for (i, e) in t.g.turn_events.iter().enumerate().skip(from) {
        match e {
            Event::Discarded { .. } => discards.push(i),
            Event::Drew { .. } => draws.push(i),
            _ => {}
        }
    }
    (discards, draws)
}

#[test]
fn exhume_puts_each_players_creature_onto_the_battlefield_at_the_same_time() {
    cr!("101.4", "101.4b");
    ruling!(
        "Clone",
        "If Clone somehow enters at the same time as another creature, Clone can't become a copy of that creature. You may choose only a creature that's already on the battlefield."
    );
    supported("Exhume");
    supported("Clone");
    let mut t = TestGame::new(2);
    let bears = t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P0, "Hill Giant");
    let clone = t.graveyard(P1, "Clone");
    t.graveyard(P1, "Hill Giant");
    // When P1 chooses, P0 has chosen but nothing has moved yet.
    let seen = watch(&mut t, P1, is_choose, |g| {
        g.find_in_zone(Zone::Graveyard(P0), "Grizzly Bears").len()
    });
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.answer_choose(P1, &[Entity::Object(clone)]);
    t.answer_yes(P1, true);
    t.lands(P0, "Swamp", 2);
    let spell = t.hand(P0, "Exhume");
    let from = t.asked().len();
    t.cast(P0, spell).go();
    t.resolve_all();
    assert_eq!(seen.lock().unwrap().first(), Some(&1));
    let b = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(b.len(), 1, "only P0's Grizzly Bears");
    assert_eq!(t.obj_now(b[0]).controller, P0);
    // The Clone entered with nothing to copy: Grizzly Bears entered at the same time.
    assert!(offered_permanents(&t, P1, from)
        .iter()
        .all(|c| !c.contains(&b[0])));
    assert!(t.in_graveyard(P1, "Clone"));
}

#[test]
fn exhume_is_one_event_for_one_or_more_triggers() {
    cr!("101.4", "603.2c");
    let mut t = TestGame::new(2);
    t.custom(
        P0,
        custom_card(
            "Arrival Bell",
            "Enchantment",
            "{1}",
            None,
            "Whenever one or more creatures enter, you gain 1 life.",
        ),
        Zone::Battlefield,
    );
    t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P1, "Hill Giant");
    t.lands(P0, "Swamp", 2);
    let spell = t.hand(P0, "Exhume");
    t.cast(P0, spell).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
    // Both creatures entered in one event: the ability triggered once.
    assert_eq!(t.life(P0), 21);
}

#[test]
fn mind_swords_both_players_choose_then_the_cards_are_exiled_together() {
    cr!("101.4", "101.4a");
    ruling!(
        "Mind Swords",
        "The active player chooses two cards, then the other player chooses two cards, then all the chosen cards are exiled simultaneously."
    );
    supported("Mind Swords");
    let mut t = TestGame::new(2);
    let a = t.hand(P0, "Grizzly Bears");
    let b = t.hand(P0, "Hill Giant");
    let keep0 = t.hand(P0, "Forest");
    let c = t.hand(P1, "Lightning Bolt");
    let d = t.hand(P1, "Shock");
    let keep1 = t.hand(P1, "Island");
    // What P1 sees as they choose: P0's chosen cards are still in P0's hand, face down
    // (CR 101.4a: only the fact that P0 chose is known).
    let seen = watch(&mut t, P1, is_choose, |g| {
        (
            g.player(P0).hand.len(),
            g.known_apnap_choices(P1)
                .iter()
                .map(|(p, c)| (*p, c.is_some()))
                .collect::<Vec<_>>(),
        )
    });
    t.answer_choose(P0, &[Entity::Object(a), Entity::Object(b)]);
    t.answer_choose(P1, &[Entity::Object(c), Entity::Object(d)]);
    t.lands(P0, "Swamp", 2);
    let spell = t.hand(P0, "Mind Swords");
    let order_from = t.asked().len();
    t.cast(P0, spell).go();
    t.resolve_all();
    let order: Vec<PlayerId> = t.asked()[order_from..]
        .iter()
        .filter(|(_, d)| is_choose(d))
        .map(|(p, _)| *p)
        .collect();
    assert_eq!(order, vec![P0, P1]);
    assert_eq!(*seen.lock().unwrap(), vec![(3, vec![(P0, false)])]);
    for x in [a, b, c, d] {
        assert_eq!(t.zone(x), Zone::Exile);
    }
    assert_eq!(t.zone(keep0), Zone::Hand(P0));
    assert_eq!(t.zone(keep1), Zone::Hand(P1));
}

#[test]
fn show_and_tell_entry_choices_are_made_after_everyone_chose_a_card() {
    cr!("101.4", "616.1");
    ruling!(
        "Show and Tell",
        "If the cards being put onto the battlefield also require choices, those choices are made after all players choose their card. The active player makes choices for their card (if any), then the other players (if any) in turn order."
    );
    ruling!("Show and Tell", "Players choose cards during resolution, not announcement.");
    supported("Show and Tell");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let clone0 = t.hand(P0, "Clone");
    let clone1 = t.hand(P1, "Clone");
    t.lands(P0, "Island", 3);
    let spell = t.hand(P0, "Show and Tell");
    t.answer_yes(P0, true).answer_yes(P1, true);
    t.answer_choose(P0, &[Entity::Object(clone0)]);
    t.answer_choose(P1, &[Entity::Object(clone1)]);
    let from = t.asked().len();
    t.cast(P0, spell).go();
    // Nothing is chosen as the spell is cast.
    assert!(t.asked()[from..].iter().all(|(_, d)| !is_choose(d)));
    t.answer_choose(P0, &[Entity::Object(giant)]);
    t.answer_choose(P1, &[Entity::Object(bears)]);
    t.resolve_all();
    // The decisions: each player's card, then each Clone's copy choice, P0's first.
    let chooses: Vec<(PlayerId, bool)> = t.asked()[from..]
        .iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some((
                *p,
                candidates
                    .iter()
                    .filter_map(|e| e.object())
                    .any(|o| o == bears || o == giant),
            )),
            _ => None,
        })
        .collect();
    let cards: Vec<usize> = (0..chooses.len()).filter(|i| !chooses[*i].1).collect();
    let copies: Vec<usize> = (0..chooses.len()).filter(|i| chooses[*i].1).collect();
    assert_eq!(cards.len(), 2);
    assert_eq!(copies.len(), 2);
    assert!(cards.iter().max() < copies.iter().min());
    assert_eq!(chooses[copies[0]].0, P0);
    assert_eq!(chooses[copies[1]].0, P1);
    // Each Clone could copy only the creatures already on the battlefield.
    for p in [P0, P1] {
        for c in offered_permanents(&t, p, from) {
            assert!(c.iter().all(|o| *o == bears || *o == giant), "{c:?}");
        }
    }
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 2);
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 2);
}

#[test]
fn tempting_wurm_opponents_put_their_cards_onto_the_battlefield_together() {
    cr!("101.4", "608.2f");
    supported("Tempting Wurm");
    let mut t = TestGame::new(3);
    let bears = t.hand(P1, "Grizzly Bears");
    let giant = t.hand(P1, "Hill Giant");
    let forest = t.hand(P1, "Forest");
    let clone = t.hand(P2, "Clone");
    let bolt = t.hand(P2, "Lightning Bolt");
    t.answer_yes(P1, true).answer_yes(P2, true);
    t.answer_choose(P1, &[Entity::Object(bears), Entity::Object(giant)]);
    t.answer_choose(P2, &[Entity::Object(clone)]);
    let from = t.asked().len();
    let wurm = t.enter(P0, "Tempting Wurm");
    t.settle();
    t.answer_choose(P2, &[Entity::Object(wurm)]);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
    // P2's Clone could copy only Tempting Wurm: P1's creatures entered with it.
    assert_eq!(offered_permanents(&t, P2, from), vec![vec![wurm]]);
    assert_eq!(t.named_on_battlefield("Tempting Wurm").len(), 2);
    // Only artifact, creature, enchantment, and land cards could be chosen.
    assert!(offered(&t, P2, from).iter().all(|c| !c.contains(&bolt)));
    assert_eq!(t.zone(forest), Zone::Hand(P1));
    assert_eq!(t.zone(bolt), Zone::Hand(P2));
}

/// Kynaios and Tiro of Meletis's end step ability for P0 in a three-player game.
fn kynaios_end_step(t: &mut TestGame) {
    t.battlefield(P0, "Kynaios and Tiro of Meletis");
    t.set_step(P0, Step::PostcombatMain);
    t.advance_to(P0, Step::End);
    t.resolve_all();
}

#[test]
fn kynaios_and_tiro_players_choose_lands_in_turn_order_then_they_enter_together() {
    cr!("101.4", "101.4b");
    ruling!(
        "Kynaios and Tiro of Meletis",
        "Starting with you and proceeding in turn order, each player chooses a land card from their hand or chooses not to choose one, but doesn’t reveal any chosen lands yet. Once all players have made this choice, all chosen lands are revealed and enter the battlefield simultaneously."
    );
    supported("Kynaios and Tiro of Meletis");
    let mut t = TestGame::new(3);
    let forest = t.hand(P0, "Forest");
    let island = t.hand(P1, "Island");
    let mountain = t.hand(P2, "Mountain");
    let hands: Vec<usize> = [P0, P1, P2].iter().map(|p| t.hand_size(*p)).collect();
    t.answer_yes(P0, true)
        .answer_yes(P1, false)
        .answer_yes(P2, true);
    t.answer_choose(P0, &[Entity::Object(forest)]);
    t.answer_choose(P2, &[Entity::Object(mountain)]);
    // When P2 chooses, P0's land hasn't entered yet.
    let seen = watch(&mut t, P2, is_choose, |g| {
        g.find_in_zone(Zone::Hand(P0), "Forest").len()
    });
    let from = t.asked().len();
    kynaios_end_step(&mut t);
    let order: Vec<PlayerId> = t.asked()[from..]
        .iter()
        .filter(|(_, d)| is_yes_no(d))
        .map(|(p, _)| *p)
        .collect();
    assert_eq!(order, vec![P0, P1, P2]);
    assert_eq!(*seen.lock().unwrap(), vec![1]);
    let f = t.g.current(forest);
    let m = t.g.current(mountain);
    assert!(t.on_battlefield(f) && t.on_battlefield(m));
    assert_eq!(t.obj_now(f).controller, P0);
    assert_eq!(t.obj_now(m).controller, P2);
    assert_eq!(t.zone(island), Zone::Hand(P1));
    // The opponent who didn't put a land drew a card; the one who did didn't.
    assert_eq!(t.hand_size(P1), hands[1] + 1);
    assert_eq!(t.hand_size(P2), hands[2] - 1);
    // P0 drew a card and put the Forest onto the battlefield.
    assert_eq!(t.hand_size(P0), hands[0]);
}

#[test]
fn kynaios_and_tiro_the_card_drawn_can_be_the_land_put_onto_the_battlefield() {
    cr!("101.4");
    ruling!(
        "Kynaios and Tiro of Meletis",
        "If the card you draw is a land card, you may choose it to put onto the battlefield."
    );
    let mut t = TestGame::new(2);
    let top = t.library_top(P0, "Plains");
    t.answer_yes(P0, true).answer_yes(P1, false);
    t.answer_choose(P0, &[Entity::Object(top)]);
    kynaios_end_step(&mut t);
    let p = t.g.current(top);
    assert!(t.on_battlefield(p));
    assert_eq!(t.obj_now(p).controller, P0);
}

#[test]
fn tempt_with_immortality_returns_cards_without_targeting_them() {
    cr!("101.4", "115.1");
    ruling!(
        "Tempt with Immortality",
        "None of the creature cards in graveyards are targeted. The cards are chosen as Tempt with Immortality resolves."
    );
    ruling!(
        "Tempt with Immortality",
        "After each opponent has decided, the effect happens simultaneously for each one who accepted the offer."
    );
    supported("Tempt with Immortality");
    supported("Ground Seal");
    let mut t = TestGame::new(3);
    // "Cards in graveyards can't be the targets of spells or abilities."
    t.battlefield(P2, "Ground Seal");
    let bears = t.graveyard(P0, "Grizzly Bears");
    let giant = t.graveyard(P1, "Hill Giant");
    t.graveyard(P1, "Elvish Mystic");
    let clone = t.graveyard(P2, "Clone");
    t.answer_yes(P1, true).answer_yes(P2, true);
    t.answer_choose(P1, &[Entity::Object(giant)]);
    t.answer_choose(P2, &[Entity::Object(clone)]);
    t.lands(P0, "Swamp", 5);
    let spell = t.hand(P0, "Tempt with Immortality");
    let from = t.asked().len();
    t.cast(P0, spell).go();
    t.resolve_all();
    let b = t.g.current(bears);
    assert!(t.on_battlefield(b), "P0's creature returned first");
    // P2's Clone entered with P1's Hill Giant: it could copy only what was already there
    // (P0's Grizzly Bears).
    let copy_choices = offered_permanents(&t, P2, from);
    assert_eq!(copy_choices, vec![vec![b]]);
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
    assert!(t
        .asked()
        .iter()
        .all(|(_, d)| !matches!(d, Decision::ChooseTargets { .. })));
}

#[test]
fn flux_everyone_discards_then_each_draws_that_many() {
    cr!("608.2e", "101.4");
    ruling!(
        "Flux",
        "Each player chooses how many cards they would like to discard. That number may be 0."
    );
    supported("Flux");
    let mut t = TestGame::new(3);
    let a = t.hand(P0, "Forest");
    t.hand(P0, "Island");
    t.hand(P1, "Swamp");
    let b = t.hand(P2, "Grizzly Bears");
    let c = t.hand(P2, "Hill Giant");
    t.hand(P2, "Shock");
    // P0 discards one card, P1 none, P2 two.
    t.answer_choose(P0, &[Entity::Object(a)]);
    t.answer_choose(P1, &[]);
    t.answer_choose(P2, &[Entity::Object(b), Entity::Object(c)]);
    t.lands(P0, "Island", 3);
    let spell = t.hand(P0, "Flux");
    t.cast(P0, spell).go();
    let events = t.g.turn_events.len();
    let hands: Vec<usize> = [P0, P1, P2].iter().map(|p| t.hand_size(*p)).collect();
    t.resolve_all();
    let (discards, draws) = discards_and_draws(&t, events);
    assert_eq!(discards.len(), 3);
    // P0 draws one and P2 two, then P0 draws a card for "Draw a card."
    assert_eq!(draws.len(), 4);
    // CR 608.2e: everyone discards before anyone draws.
    assert!(discards.iter().max() < draws.iter().min());
    assert_eq!(t.hand_size(P0), hands[0] + 1);
    assert_eq!(t.hand_size(P1), hands[1]);
    assert_eq!(t.hand_size(P2), hands[2]);
}

#[test]
fn ruin_grinder_players_decide_in_turn_order_then_all_discard_and_draw() {
    cr!("101.4", "101.4b", "608.2e");
    ruling!(
        "Ruin Grinder",
        "Players decide and announce in turn order, starting with the active player, whether or not they want to discard their hand. Then, all players who chose to do so discard their hands and draw seven cards."
    );
    supported("Ruin Grinder");
    let mut t = TestGame::new(2);
    let grinder = t.battlefield(P0, "Ruin Grinder");
    t.hand(P0, "Forest");
    t.hand(P1, "Island");
    t.hand(P1, "Swamp");
    t.answer_yes(P0, true).answer_yes(P1, true);
    // P1 knows P0's decision when deciding.
    let seen = watch(&mut t, P1, is_yes_no, acceptances);
    let events = t.g.turn_events.len();
    t.g.destroy(grinder, None);
    t.settle();
    t.resolve_all();
    assert_eq!(seen.lock().unwrap().len(), 1);
    assert_eq!(seen.lock().unwrap()[0].len(), 1, "P0's decision is known");
    let (discards, draws) = discards_and_draws(&t, events);
    assert_eq!(discards.len(), 3);
    assert_eq!(draws.len(), 14);
    assert!(discards.iter().max() < draws.iter().min());
    assert_eq!(t.hand_size(P0), 7);
    assert_eq!(t.hand_size(P1), 7);
}

#[test]
fn worldgorger_dragon_returns_the_exiled_cards_together() {
    cr!("608.2f", "303.4f");
    ruling!(
        "Worldgorger Dragon",
        "It can't enter the battlefield enchanting a permanent that enters the battlefield at the same time."
    );
    supported("Worldgorger Dragon");
    supported("Holy Strength");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let aura = t.battlefield(P0, "Holy Strength");
    assert!(t.g.attach(aura, Entity::Object(bears)));
    let dragon = t.enter(P0, "Worldgorger Dragon");
    t.settle();
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears") && t.in_exile("Holy Strength"));
    // It leaves: the exiled cards return at the same time, so the Aura has nothing it
    // could enchant (the Grizzly Bears enters with it) and stays in exile.
    let d = t.g.current(dragon);
    t.g.destroy(d, None);
    t.settle();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert!(t.in_exile("Holy Strength"));
}

#[test]
fn damage_dealt_to_each_opponent_at_once_is_one_life_gain_event() {
    cr!("608.2f", "120.3f", "702.15b");
    supported("Ajani's Pridemate");
    let mut t = TestGame::new(3);
    // "Whenever you gain life, put a +1/+1 counter on Ajani's Pridemate."
    let mate = t.battlefield(P0, "Ajani's Pridemate");
    t.lands(P1, "Forest", 2);
    t.lands(P2, "Forest", 3);
    let def = custom_card(
        "Draining Wurm",
        "Creature — Wurm",
        "{4}",
        Some((3, 3)),
        "Lifelink\nWhen this creature enters, it deals damage to each opponent equal to the number of lands that player controls.",
    );
    t.custom(P0, def, Zone::Hand(P0));
    let wurm = t.g.find_in_zone(Zone::Hand(P0), "Draining Wurm")[0];
    let w = t.g.move_object(wurm, Zone::Battlefield, mtg_engine::events::MoveCause::Effect, Some(P0));
    assert!(w.is_some());
    t.settle();
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.life(P2), 17);
    assert_eq!(t.life(P0), 25);
    // The damage was dealt at once: one life gain event.
    assert_eq!(t.counters(mate, "+1/+1"), 1);
}

#[test]
fn timetwister_shuffles_every_library_even_with_nothing_to_shuffle_in() {
    cr!("701.24d");
    supported("Timetwister");
    supported("Cosi's Trickster");
    let mut t = TestGame::new(2);
    // "Whenever an opponent shuffles their library, you may put a +1/+1 counter on this
    // creature."
    let trickster = t.battlefield(P1, "Cosi's Trickster");
    t.answer_yes(P1, true);
    t.lands(P0, "Island", 3);
    let spell = t.hand(P0, "Timetwister");
    // P0's hand and graveyard are empty once Timetwister is cast.
    assert_eq!(t.graveyard_size(P0), 0);
    t.cast(P0, spell).go();
    assert_eq!(t.hand_size(P0), 0);
    t.resolve_all();
    let shuffles = |p: PlayerId| {
        t.g.turn_events
            .iter()
            .filter(|e| matches!(e, Event::Shuffled { player } if *player == p))
            .count()
    };
    assert_eq!(shuffles(P0), 1);
    assert_eq!(shuffles(P1), 1);
    assert_eq!(t.counters(trickster, "+1/+1"), 1);
    assert_eq!(t.hand_size(P0), 7);
    assert_eq!(t.hand_size(P1), 7);
}

#[test]
fn tempt_with_discovery_the_opponents_lands_enter_in_one_event() {
    cr!("101.4", "603.2c");
    ruling!(
        "Tempt with Discovery",
        "After each opponent has decided, the effect happens simultaneously for each one who accepted the offer."
    );
    supported("Tempt with Discovery");
    let mut t = TestGame::new(3);
    t.custom(
        P0,
        custom_card(
            "Border Watch",
            "Enchantment",
            "{1}",
            None,
            "Whenever one or more lands you don't control enter, you gain 1 life.",
        ),
        Zone::Battlefield,
    );
    let f1 = t.library_top(P1, "Forest");
    let f2 = t.library_top(P2, "Forest");
    t.answer_yes(P1, true).answer_yes(P2, true);
    t.answer_choose(P1, &[Entity::Object(f1)]);
    t.answer_choose(P2, &[Entity::Object(f2)]);
    t.lands(P0, "Forest", 4);
    let spell = t.hand(P0, "Tempt with Discovery");
    t.cast(P0, spell).go();
    t.resolve_all();
    assert!(t.on_battlefield(t.g.current(f1)) && t.on_battlefield(t.g.current(f2)));
    // Both opponents' lands entered at the same time: one trigger.
    assert_eq!(t.life(P0), 21);
}

#[test]
fn twilights_call_all_the_creature_cards_enter_together() {
    cr!("101.4", "608.2f");
    ruling!("Twilight's Call", "All the creature cards enter simultaneously.");
    supported("Twilight's Call");
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Clone");
    t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P1, "Hill Giant");
    t.answer_yes(P0, true);
    t.lands(P0, "Swamp", 6);
    let spell = t.hand(P0, "Twilight's Call");
    let from = t.asked().len();
    t.cast(P0, spell).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
    // Nothing was on the battlefield for the Clone to copy as it entered.
    assert!(offered_permanents(&t, P0, from).is_empty());
    assert!(t.in_graveyard(P0, "Clone"));
}

/// P0's creature `card` enters with "each player sacrifices [n] creature(s)": P0 and P1
/// each control Grizzly Bears, Hill Giant and Elvish Mystic; P0 chooses Grizzly Bears
/// (and Elvish Mystic), P1 Hill Giant (and Elvish Mystic). Checks that P1 chose knowing
/// P0's choice, before anything was sacrificed, and that the chosen creatures died.
fn each_player_sacrifices(card: &str, n: usize) {
    supported(card);
    let mut t = TestGame::new(2);
    let mut p0 = vec![t.battlefield(P0, "Grizzly Bears")];
    t.battlefield(P0, "Hill Giant");
    let mystic0 = t.battlefield(P0, "Elvish Mystic");
    t.battlefield(P1, "Grizzly Bears");
    let mut p1 = vec![t.battlefield(P1, "Hill Giant")];
    let mystic1 = t.battlefield(P1, "Elvish Mystic");
    if n == 2 {
        p0.push(mystic0);
        p1.push(mystic1);
    }
    let ents = |v: &[ObjectId]| v.iter().map(|o| Entity::Object(*o)).collect::<Vec<_>>();
    t.answer_choose(P0, &ents(&p0));
    t.answer_choose(P1, &ents(&p1));
    let seen = watch(&mut t, P1, is_choose, |g| {
        (
            g.find_in_zone(Zone::Battlefield, "Grizzly Bears").len(),
            g.known_apnap_choices(PlayerId(1))
                .iter()
                .map(|(p, c)| (*p, c.as_ref().map(|c| c.len())))
                .collect::<Vec<_>>(),
        )
    });
    t.enter(P0, card);
    t.settle();
    t.resolve_all();
    assert_eq!(
        *seen.lock().unwrap(),
        vec![(2, vec![(P0, Some(n))])],
        "{card}: P1 knew P0's choice, and nothing had been sacrificed yet"
    );
    for o in p0.iter().chain(&p1) {
        assert!(!t.g.is_live(*o) || t.zone(*o) != Zone::Battlefield, "{card}");
    }
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1, "{card}");
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1, "{card}");
}

#[test]
fn each_player_sacrifices_choices_in_turn_order_then_all_at_once() {
    cr!("101.4", "101.4b", "701.21a");
    ruling!(
        "Fleshbag Marauder",
        "first the player whose turn it is chooses a creature to sacrifice, then each other player in turn order does the same knowing the choices made before them. Then all those creatures are sacrificed simultaneously."
    );
    ruling!(
        "Lokhust Heavy Destroyer",
        "first the player whose turn it is chooses a creature to sacrifice, then each other player in turn order does the same knowing the choices made before them. Then all those creatures are sacrificed simultaneously."
    );
    ruling!(
        "Accursed Marauder",
        "Players will know the choices of previous players when making their choice. Then all of the chosen creatures are sacrificed by their controllers simultaneously."
    );
    ruling!(
        "Abyssal Gorestalker",
        "first the player whose turn it is chooses which two creatures they are going to sacrifice, then each other player in turn order does the same. Players will know the choices of previous players when making their choices. Then all creatures are sacrificed by their controllers simultaneously."
    );
    each_player_sacrifices("Fleshbag Marauder", 1);
    each_player_sacrifices("Lokhust Heavy Destroyer", 1);
    each_player_sacrifices("Accursed Marauder", 1);
    each_player_sacrifices("Abyssal Gorestalker", 2);
}

#[test]
fn each_opponent_discards_cards_chosen_face_down_then_discarded_together() {
    cr!("101.4", "101.4a");
    ruling!(
        "Elvish Doomsayer",
        "first the next opponent in turn order chooses a card without revealing it, then each other opponent in turn order does the same. Then each chosen card is discarded simultaneously."
    );
    supported("Elvish Doomsayer");
    let mut t = TestGame::new(3);
    let doomsayer = t.battlefield(P0, "Elvish Doomsayer");
    let a = t.hand(P1, "Grizzly Bears");
    t.hand(P1, "Hill Giant");
    t.hand(P2, "Grizzly Bears");
    let b = t.hand(P2, "Hill Giant");
    t.answer_choose(P1, &[Entity::Object(a)]);
    t.answer_choose(P2, &[Entity::Object(b)]);
    // When P2 chooses, P1's card is still in P1's hand, and only the fact that P1 chose
    // is known.
    let seen = watch(&mut t, P2, is_choose, |g| {
        (
            g.player(PlayerId(1)).hand.len(),
            g.known_apnap_choices(PlayerId(2))
                .iter()
                .map(|(p, c)| (*p, c.is_some()))
                .collect::<Vec<_>>(),
        )
    });
    let from = t.asked().len();
    t.g.destroy(doomsayer, None);
    t.settle();
    t.resolve_all();
    let order: Vec<PlayerId> = t.asked()[from..]
        .iter()
        .filter(|(_, d)| is_choose(d))
        .map(|(p, _)| *p)
        .collect();
    assert_eq!(order, vec![P1, P2]);
    assert_eq!(*seen.lock().unwrap(), vec![(2, vec![(P1, false)])]);
    assert!(t.in_graveyard(P1, "Grizzly Bears") && t.in_graveyard(P2, "Hill Giant"));
}

#[test]
fn each_player_discards_at_the_beginning_of_upkeep_all_at_once() {
    cr!("101.4", "101.4a");
    ruling!(
        "Cunning Lethemancer",
        "First you choose a card to discard, then each other player in turn order chooses a card to discard, then all those cards are discarded simultaneously. No one sees what the other players are discarding before deciding which cards to discard."
    );
    supported("Cunning Lethemancer");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Cunning Lethemancer");
    let a = t.hand(P0, "Grizzly Bears");
    t.hand(P0, "Hill Giant");
    let b = t.hand(P1, "Lightning Bolt");
    t.hand(P1, "Shock");
    t.answer_choose(P0, &[Entity::Object(a)]);
    t.answer_choose(P1, &[Entity::Object(b)]);
    let seen = watch(&mut t, P1, is_choose, |g| {
        (
            g.player(PlayerId(0)).hand.len(),
            g.known_apnap_choices(PlayerId(1))
                .iter()
                .map(|(p, c)| (*p, c.is_some()))
                .collect::<Vec<_>>(),
        )
    });
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(*seen.lock().unwrap(), vec![(2, vec![(P0, false)])]);
    assert!(t.in_graveyard(P0, "Grizzly Bears") && t.in_graveyard(P1, "Lightning Bolt"));
}

#[test]
fn malboro_each_opponent_discards_then_loses_life_then_exiles() {
    cr!("101.4", "608.2e");
    ruling!(
        "Malboro",
        "Then the chosen cards are discarded at the same time. Next, each opponent loses 2 life at the same time. Finally, each opponent exiles the top three cards of their library simultaneously."
    );
    supported("Malboro");
    let mut t = TestGame::new(3);
    let a = t.hand(P1, "Grizzly Bears");
    t.hand(P1, "Hill Giant");
    t.hand(P2, "Grizzly Bears");
    let b = t.hand(P2, "Hill Giant");
    t.answer_choose(P1, &[Entity::Object(a)]);
    t.answer_choose(P2, &[Entity::Object(b)]);
    let seen = watch(&mut t, P2, is_choose, |g| g.player(PlayerId(1)).hand.len());
    let from = t.g.turn_events.len();
    t.enter(P0, "Malboro");
    t.settle();
    t.resolve_all();
    assert_eq!(*seen.lock().unwrap(), vec![2]);
    let mut discards = Vec::new();
    let mut losses = Vec::new();
    let mut exiles = Vec::new();
    for (i, e) in t.g.turn_events.iter().enumerate().skip(from) {
        match e {
            Event::Discarded { .. } => discards.push(i),
            Event::LifeLost { .. } => losses.push(i),
            Event::ZoneChange { to: Zone::Exile, .. } => exiles.push(i),
            _ => {}
        }
    }
    assert_eq!((discards.len(), losses.len(), exiles.len()), (2, 2, 6));
    assert!(discards.iter().max() < losses.iter().min());
    assert!(losses.iter().max() < exiles.iter().min());
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.life(P2), 18);
}

#[test]
fn each_player_spares_creatures_in_turn_order_then_the_rest_are_sacrificed() {
    cr!("101.4", "101.4b");
    ruling!(
        "Slaughter the Strong",
        "first the player whose turn it is chooses which creatures will be spared, then each other player in turn order does the same knowing the choices made before them. Then all the creatures not chosen are sacrificed simultaneously."
    );
    ruling!(
        "Destined Confrontation",
        "first the player whose turn it is chooses which creatures will be spared, then each other player in turn order does the same knowing the choices made before them. Then all the creatures not chosen are sacrificed simultaneously."
    );
    for (card, cost) in [("Slaughter the Strong", 3), ("Destined Confrontation", 4)] {
        supported(card);
        let mut t = TestGame::new(2);
        let bears0 = t.battlefield(P0, "Grizzly Bears");
        let giant0 = t.battlefield(P0, "Hill Giant");
        let bears1 = t.battlefield(P1, "Grizzly Bears");
        let giant1 = t.battlefield(P1, "Hill Giant");
        t.answer_choose(P0, &[Entity::Object(bears0)]);
        t.answer_choose(P1, &[Entity::Object(giant1)]);
        // When P1 chooses, P0's Hill Giant (not spared) is still on the battlefield.
        let seen = watch(&mut t, P1, is_choose, |g| {
            g.find_in_zone(Zone::Battlefield, "Hill Giant").len()
        });
        t.lands(P0, "Plains", cost);
        let spell = t.hand(P0, card);
        let from = t.asked().len();
        t.cast(P0, spell).go();
        t.resolve_all();
        let order: Vec<PlayerId> = t.asked()[from..]
            .iter()
            .filter(|(_, d)| is_choose(d))
            .map(|(p, _)| *p)
            .collect();
        assert_eq!(order, vec![P0, P1], "{card}");
        assert_eq!(*seen.lock().unwrap(), vec![2], "{card}");
        assert!(t.on_battlefield(bears0) && t.on_battlefield(giant1), "{card}");
        assert!(!t.on_battlefield(giant0) && !t.on_battlefield(bears1), "{card}");
    }
}
