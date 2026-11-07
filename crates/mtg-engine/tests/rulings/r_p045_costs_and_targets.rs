//! Rulings batch P045 — discard costs (CR 118, 601.2h, 602.2b) and the targets of spells
//! and abilities that make players discard (CR 115, 601.2c, 608.2b).

use crate::r_p045_common::*;
use crate::r_s01_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

// ---------------------------------------------------------------------------------------
// Assassin's Strike: "Destroy target creature. Its controller discards a card."
// ---------------------------------------------------------------------------------------

fn assassins_strike(t: &mut TestGame, target: ObjectId) {
    let s = t.hand(P0, "Assassin's Strike");
    give_mana_for(t, P0, "Assassin's Strike");
    t.cast(P0, s).target(target).go();
}

#[test]
fn assassins_strike_discard_happens_even_if_the_creature_survives() {
    cr!("608.2c", "702.12b");
    ruling!(
        "Assassin's Strike",
        "If Assassin’s Strike resolves but the creature isn’t destroyed (perhaps because it regenerated or has indestructible), its controller will discard a card."
    );
    supported("Assassin's Strike");
    let mut t = TestGame::new(2);
    // Darksteel Myr: indestructible.
    let myr = t.battlefield(P1, "Darksteel Myr");
    bears_in_hand(&mut t, P1, 1);
    assassins_strike(&mut t, myr);
    t.resolve_all();
    assert!(t.on_battlefield(myr));
    assert_eq!(t.hand_size(P1), 0);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn assassins_strike_with_an_illegal_target_does_nothing() {
    cr!("608.2b", "701.6a");
    ruling!(
        "Assassin's Strike",
        "If the creature is an illegal target when Assassin’s Strike tries to resolve, it won’t resolve and none of its effects will happen."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    bears_in_hand(&mut t, P1, 1);
    assassins_strike(&mut t, bears);
    // In response, the Bears are returned to hand.
    let unsummon = t.hand(P1, "Unsummon");
    t.lands(P1, "Island", 1);
    t.cast(P1, unsummon).target(bears).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P1), 2);
    assert!(t.in_graveyard(P0, "Assassin's Strike"));
    assert_eq!(t.graveyard_size(P1), 1); // Unsummon only.
}

#[test]
fn assassins_strike_needs_a_creature_to_target() {
    cr!("601.2c", "115.1");
    ruling!(
        "Assassin's Strike",
        "You must be able to target a creature to cast Assassin’s Strike."
    );
    let mut t = TestGame::new(2);
    bears_in_hand(&mut t, P1, 1);
    let s = t.hand(P0, "Assassin's Strike");
    give_mana_for(&mut t, P0, "Assassin's Strike");
    assert!(t.cast(P0, s).try_go().is_err());
    assert_eq!(t.zone(s), Zone::Hand(P0));
}

#[test]
fn mindculling_with_an_illegal_target_doesnt_draw() {
    cr!("608.2b", "702.18a");
    ruling!(
        "Mindculling",
        "If that opponent is an illegal target when Mindculling tries to resolve, it’ll be countered and will have no effect. You won’t draw two cards."
    );
    supported("Mindculling");
    supported("Gilded Light");
    let mut t = TestGame::new(2);
    bears_in_hand(&mut t, P1, 2);
    let m = t.hand(P0, "Mindculling");
    give_mana_for(&mut t, P0, "Mindculling");
    t.cast(P0, m).target(Entity::Player(P1)).go();
    // Gilded Light: "You gain shroud until end of turn."
    let light = t.hand(P1, "Gilded Light");
    give_mana_for(&mut t, P1, "Gilded Light");
    t.cast(P1, light).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 0);
    assert_eq!(t.hand_size(P1), 2);
}

// ---------------------------------------------------------------------------------------
// Ruthless Disposal: "As an additional cost to cast this spell, discard a card and
// sacrifice a creature. Two target creatures each get -13/-13 until end of turn."
// ---------------------------------------------------------------------------------------

/// P0 holds Ruthless Disposal and its mana.
fn disposal(t: &mut TestGame) -> ObjectId {
    supported("Ruthless Disposal");
    let d = t.hand(P0, "Ruthless Disposal");
    give_mana_for(t, P0, "Ruthless Disposal");
    d
}

#[test]
fn ruthless_disposal_costs_are_exactly_one_discard_and_one_sacrifice() {
    cr!("601.2b", "601.2h", "118.8");
    ruling!(
        "Ruthless Disposal",
        "You can’t discard or sacrifice more to target more creatures, and you can’t cast Ruthless Disposal at all if you don’t have any other cards in hand or if you control no creatures."
    );
    // No other card in hand: can't cast.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    let d = disposal(&mut t);
    assert!(t
        .cast(P0, d)
        .targets(&[Entity::Object(a), Entity::Object(b)])
        .try_go()
        .is_err());
    assert_eq!(t.zone(d), Zone::Hand(P0));
    // No creature to sacrifice (two opposing creatures to target): can't cast.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    t.hand(P0, "Island");
    let d = disposal(&mut t);
    assert!(t
        .cast(P0, d)
        .targets(&[Entity::Object(a), Entity::Object(b)])
        .try_go()
        .is_err());
    // With three cards in hand and three creatures: exactly one discarded, exactly one
    // sacrificed, exactly two targets.
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let mine2 = t.battlefield(P0, "Hill Giant");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Craw Wurm");
    bears_in_hand(&mut t, P0, 3);
    let d = disposal(&mut t);
    let from = t.asked().len();
    t.cast(P0, d)
        .targets(&[Entity::Object(a), Entity::Object(b)])
        .go();
    let max_targets: Vec<u32> = asked_since(&t, from)
        .into_iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseTargets { min, max, .. } => Some(min * 100 + max),
            _ => None,
        })
        .collect();
    assert_eq!(max_targets, vec![202]);
    assert_eq!(t.hand_size(P0), 2);
    assert_eq!(
        [mine, mine2].iter().filter(|c| t.on_battlefield(**c)).count(),
        1
    );
    t.resolve_all();
    assert!(!t.on_battlefield(a));
    assert!(!t.on_battlefield(b));
}

#[test]
fn ruthless_disposal_needs_two_distinct_targets_one_may_be_the_sacrifice() {
    cr!("601.2c", "601.2h", "115.3", "608.2b");
    ruling!(
        "Ruthless Disposal",
        "You must target exactly two creatures to cast Ruthless Disposal. However, one of the targets can be the creature you intend to sacrifice"
    );
    ruling!(
        "Ruthless Disposal",
        "You can’t target one creature twice to give it -26/-26."
    );
    ruling!(
        "Ruthless Disposal",
        "If one targeted creature becomes an illegal target, the other will still get -13/-13 until end of turn."
    );
    // Only one creature on the battlefield: there aren't two targets (targets are chosen
    // before costs are paid), so it can't be cast.
    let mut t = TestGame::new(2);
    let b = t.battlefield(P1, "Craw Wurm");
    t.hand(P0, "Island");
    let d = disposal(&mut t);
    assert!(t.cast(P0, d).targets(&[Entity::Object(b)]).try_go().is_err());
    assert!(t
        .cast(P0, d)
        .targets(&[Entity::Object(b), Entity::Object(b)])
        .try_go()
        .is_err());
    assert_eq!(t.zone(d), Zone::Hand(P0));
    // P0's only creature is a target and is sacrificed to pay the cost: the spell
    // resolves for the other target (the sacrificed one is now an illegal target).
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.hand(P0, "Island");
    let d = disposal(&mut t);
    t.cast(P0, d)
        .targets(&[Entity::Object(mine), Entity::Object(wurm)])
        .go();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert!(t.in_graveyard(P0, "Island"));
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Craw Wurm"));
    assert!(t.in_graveyard(P0, "Ruthless Disposal"));
}

#[test]
fn ruthless_disposal_casting_is_uninterrupted() {
    cr!("601.2", "601.2i", "117.1");
    ruling!(
        "Ruthless Disposal",
        "Once you begin to cast Ruthless Disposal, no player may take actions until you’re done."
    );
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Craw Wurm");
    t.hand(P0, "Island");
    let d = disposal(&mut t);
    let from = t.asked().len();
    t.cast(P0, d)
        .targets(&[Entity::Object(a), Entity::Object(b)])
        .go();
    // Only P0 was asked anything while casting; P1 got no priority before the Bears were
    // sacrificed.
    assert!(asked_since(&t, from).iter().all(|(p, _)| *p == P0));
    assert!(!t.on_battlefield(mine));
    assert_eq!(t.stack_len(), 1);
}

#[test]
fn ruthless_disposal_cant_use_a_madness_creature_for_both_costs() {
    cr!("601.2h", "702.35a", "702.35c");
    ruling!(
        "Ruthless Disposal",
        "There’s no way to discard a creature card with madness, cast it, and sacrifice it to pay for both of Ruthless Disposal’s additional costs."
    );
    supported("Basking Rootwalla");
    // P0 controls no creature and holds a madness card: the sacrifice can't be paid as
    // Ruthless Disposal is cast, before the madness card could be cast.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Craw Wurm");
    // Basking Rootwalla: a creature with madness {0}.
    t.hand(P0, "Basking Rootwalla");
    let d = disposal(&mut t);
    assert!(t
        .cast(P0, d)
        .targets(&[Entity::Object(a), Entity::Object(b)])
        .try_go()
        .is_err());
    assert_eq!(t.zone(d), Zone::Hand(P0));
    assert!(t.in_hand(P0, "Basking Rootwalla"));
}

// ---------------------------------------------------------------------------------------
// Other discard costs.
// ---------------------------------------------------------------------------------------

#[test]
fn magmatic_insight_needs_exactly_one_land_card_to_discard() {
    cr!("601.2b", "601.2h", "118.8");
    ruling!(
        "Magmatic Insight",
        "You must discard exactly one land card to cast Magmatic Insight. You can't cast it without discarding a land card, and you can't discard additional cards."
    );
    supported("Magmatic Insight");
    // Only nonland cards in hand: can't cast.
    let mut t = TestGame::new(2);
    bears_in_hand(&mut t, P0, 2);
    let m = t.hand(P0, "Magmatic Insight");
    give_mana_for(&mut t, P0, "Magmatic Insight");
    assert!(t.cast(P0, m).try_go().is_err());
    // Two land cards in hand: exactly one is discarded.
    let mut t = TestGame::new(2);
    give_hand(&mut t, P0, &["Island", "Swamp", "Grizzly Bears"]);
    let m = t.hand(P0, "Magmatic Insight");
    give_mana_for(&mut t, P0, "Magmatic Insight");
    t.cast(P0, m).go();
    assert_eq!(t.graveyard_size(P0), 1);
    assert_eq!(t.hand_size(P0), 2);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 4);
}

/// Sonic Burst: "As an additional cost to cast this spell, discard a card at random.
/// Sonic Burst deals 4 damage to any target."
#[test]
fn sonic_burst_target_is_chosen_before_the_random_discard_and_only_one_card() {
    cr!("601.2c", "601.2h", "118.8");
    ruling!(
        "Sonic Burst",
        "You pick the target before you know which card is going to be discarded."
    );
    ruling!(
        "Pyromancy",
        "You pick the target before you know which card is going to be discarded."
    );
    ruling!(
        "Sonic Burst",
        "You can’t discard more than once to target more than one creature (or player) or to do multiple amounts of damage"
    );
    supported("Sonic Burst");
    let mut t = TestGame::new(2);
    bears_in_hand(&mut t, P0, 3);
    let s = t.hand(P0, "Sonic Burst");
    give_mana_for(&mut t, P0, "Sonic Burst");
    let seen = watch(
        &mut t,
        P0,
        |d| matches!(d, Decision::ChooseTargets { .. }),
        |g| (g.players[0].hand.len(), g.players[0].graveyard.len()),
    );
    t.cast(P0, s).target(Entity::Player(P1)).go();
    // When the target was chosen, nothing had been discarded yet (the 3 Bears; Sonic
    // Burst itself had moved to the stack).
    assert_eq!(seen.lock().unwrap().clone(), vec![(3, 0)]);
    assert_eq!(t.hand_size(P0), 2);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
}

#[test]
fn stormbind_discard_is_a_cost_so_library_of_leng_doesnt_apply() {
    cr!("701.9a", "602.2b");
    ruling!(
        "Stormbind",
        "Since the discard is a cost, it can't be used with Library of Leng."
    );
    supported("Stormbind");
    let mut t = TestGame::new(2);
    let storm = t.battlefield(P0, "Stormbind");
    t.battlefield(P0, "Library of Leng");
    t.lands(P0, "Wastes", 2);
    bears_in_hand(&mut t, P0, 1);
    t.answer_yes(P0, true);
    t.activate(P0, storm, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(t.life(P1), 18);
}

#[test]
fn arc_mage_divides_among_one_or_two_targets() {
    cr!("601.2c", "601.2d", "115.1d");
    ruling!(
        "Arc Mage",
        "You can't choose zero targets. You must choose between 1 and 2 targets."
    );
    supported("Arc Mage");
    let mut t = TestGame::new(2);
    let mage = t.battlefield(P0, "Arc Mage");
    t.lands(P0, "Mountain", 3);
    bears_in_hand(&mut t, P0, 1);
    let from = t.asked().len();
    t.activate(P0, mage, 0, &[Entity::Player(P1)]).unwrap();
    let bounds: Vec<(u32, u32)> = asked_since(&t, from)
        .into_iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseTargets { min, max, .. } => Some((min, max)),
            _ => None,
        })
        .collect();
    assert_eq!(bounds, vec![(1, 2)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

#[test]
fn call_the_bloodline_once_on_each_players_turn() {
    cr!("602.5b", "113.6");
    ruling!(
        "Call the Bloodline",
        "You can activate Call the Bloodline’s ability once on each player’s turn, not just your own."
    );
    supported("Call the Bloodline");
    let mut t = TestGame::new(2);
    let call = t.battlefield(P0, "Call the Bloodline");
    t.lands(P0, "Wastes", 2);
    bears_in_hand(&mut t, P0, 3);
    t.activate(P0, call, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 1);
    assert!(t.activate(P0, call, 0, &[]).is_err());
    // P1's turn: again.
    t.advance_to(P1, Step::Upkeep);
    t.activate(P0, call, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 2);
}

#[test]
fn skirsdag_supplicant_can_end_the_game_in_a_draw() {
    cr!("104.4a", "119.3");
    ruling!(
        "Skirsdag Supplicant",
        "If Skirsdag Supplicant’s ability causes each player’s life total to become 0 or less, the game ends in a draw."
    );
    supported("Skirsdag Supplicant");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Skirsdag Supplicant");
    t.lands(P0, "Swamp", 1);
    bears_in_hand(&mut t, P0, 1);
    t.g.players[0].life = 2;
    t.g.players[1].life = 1;
    t.activate(P0, s, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.has_lost(P0));
    assert!(t.has_lost(P1));
    assert!(t.g.result.is_some());
}

#[test]
fn skirsdag_supplicant_in_two_headed_giant_each_team_loses_four() {
    cr!("810.9", "119.3");
    ruling!(
        "Skirsdag Supplicant",
        "In a Two-Headed Giant game, Skirsdag Supplicant’s ability causes each player to lose 2 life, so each team loses a total of 4 life."
    );
    let mut t = crate::r_p050_common::two_headed_giant();
    let s = t.battlefield(P0, "Skirsdag Supplicant");
    t.lands(P0, "Swamp", 1);
    bears_in_hand(&mut t, P0, 1);
    let before = crate::r_p050_common::team_lives(&t);
    t.activate(P0, s, 0, &[]).unwrap();
    t.resolve_all();
    let now = crate::r_p050_common::team_lives(&t);
    assert_eq!((before.0 - now.0, before.1 - now.1), (4, 4));
}

#[test]
fn disrupting_scepter_can_target_yourself() {
    cr!("115.1", "701.9a");
    ruling!("Disrupting Scepter", "You can use it on yourself.");
    supported("Disrupting Scepter");
    let mut t = TestGame::new(2);
    let sc = t.battlefield(P0, "Disrupting Scepter");
    t.lands(P0, "Wastes", 3);
    bears_in_hand(&mut t, P0, 1);
    t.activate(P0, sc, 0, &[Entity::Player(P0)]).unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
}

#[test]
fn cinderhaze_wretch_counter_is_a_cost_and_can_kill_it_before_it_untaps() {
    cr!("602.2b", "118.3", "704.5f");
    ruling!(
        "Cinderhaze Wretch",
        "You put the -1/-1 counter on Cinderhaze Wretch as a cost. That means it happens when you activate the ability, not when it resolves."
    );
    supported("Cinderhaze Wretch");
    let mut t = TestGame::new(2);
    let w = t.battlefield(P0, "Cinderhaze Wretch");
    t.g.tap(w);
    // "Put a -1/-1 counter on this creature: Untap this creature."
    t.activate(P0, w, 1, &[]).unwrap();
    assert_eq!(t.counters(w, counters::MINUS1), 1);
    assert_eq!(t.pt(w), (2, 1));
    assert!(t.obj_now(w).tapped);
    // Activating again in response: it becomes 1/0 and dies before either resolves.
    t.activate(P0, w, 1, &[]).unwrap();
    t.settle();
    assert!(t.in_graveyard(P0, "Cinderhaze Wretch"));
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Cinderhaze Wretch"));
}

#[test]
fn drainpipe_vermin_pays_once_and_discards_one() {
    cr!("603.3", "118.12");
    ruling!(
        "Drainpipe Vermin",
        "You may pay {B} only once. The target player will discard a maximum of one card"
    );
    supported("Drainpipe Vermin");
    let mut t = TestGame::new(2);
    let v = t.battlefield(P0, "Drainpipe Vermin");
    t.lands(P0, "Swamp", 3);
    bears_in_hand(&mut t, P1, 3);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    let shock = t.hand(P1, "Shock");
    t.lands(P1, "Mountain", 1);
    t.cast(P1, shock).target(v).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Drainpipe Vermin"));
    assert_eq!(t.hand_size(P1), 2);
    assert_eq!(tapped_lands(&t, P0), 1);
}

#[test]
fn furyblade_vampire_discards_only_one_card() {
    cr!("603.3", "118.12");
    ruling!(
        "Furyblade Vampire",
        "While resolving Furyblade Vampire’s triggered ability, you can’t discard multiple cards to multiply the bonus."
    );
    supported("Furyblade Vampire");
    let mut t = TestGame::new(2);
    let v = t.battlefield(P0, "Furyblade Vampire");
    bears_in_hand(&mut t, P0, 3);
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 2);
    assert_eq!(t.pt(v), (4, 2));
}

#[test]
fn mind_maggots_can_discard_zero_creature_cards() {
    cr!("107.1c", "701.9a");
    ruling!(
        "Mind Maggots",
        "You can choose to discard zero creature cards."
    );
    supported("Mind Maggots");
    let mut t = TestGame::new(2);
    bears_in_hand(&mut t, P0, 2);
    t.answer_choose(P0, &[]);
    let m = t.enter(P0, "Mind Maggots");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 2);
    assert_eq!(t.pt(m), (2, 2));
    // Discarding both: four +1/+1 counters.
    let mut t = TestGame::new(2);
    let bears = bears_in_hand(&mut t, P0, 2);
    t.answer_choose(
        P0,
        &bears.iter().map(|b| Entity::Object(*b)).collect::<Vec<_>>(),
    );
    let m = t.enter(P0, "Mind Maggots");
    t.resolve_all();
    assert_eq!(t.pt(m), (6, 6));
}

#[test]
fn ana_battlemage_must_target_itself_and_fizzles_if_it_becomes_tapped() {
    cr!("603.3d", "608.2b", "115.1");
    ruling!(
        "Ana Battlemage",
        "If there are no other untapped creatures when Ana Battlemage enters, it must target Ana Battlemage. If the target becomes tapped by the time the ability tries to resolve, the ability won't resolve."
    );
    supported("Ana Battlemage");
    let mut t = TestGame::new(2);
    // An opposing tapped creature isn't a legal target.
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.tap(bears);
    let ana = t.hand(P0, "Ana Battlemage");
    give_mana_for(&mut t, P0, "Ana Battlemage");
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Wastes", 1);
    // Kicker {1}{B} (the {2}{U} kicker can't be paid: P0 has no blue mana).
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.cast(P0, ana).go();
    t.resolve();
    t.settle();
    let cands: Vec<Vec<Entity>> = t
        .asked()
        .into_iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseTargets {
                candidates, text, ..
            } if text.contains("untapped") => Some(candidates),
            _ => None,
        })
        .collect();
    let ana = t.g.current(ana);
    assert_eq!(cands, vec![vec![Entity::Object(ana)]]);
    assert_eq!(t.stack_len(), 1);
    // Ana becomes tapped before the trigger resolves: it doesn't resolve (no damage).
    t.g.tap(ana);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
}
