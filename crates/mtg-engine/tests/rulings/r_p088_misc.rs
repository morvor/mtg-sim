//! Rulings batch P088 — the other cards of the batch: costs paid on casting (Kaervek's
//! Spite, CR 601.2h), switching power and toughness (Aeromoeba, CR 613.4d), mana abilities
//! activated only as an instant (Diamond Lion, CR 605.3a, 602.5e), dividing damage
//! (Conflagrate, CR 601.2d, 608.2b), extra land plays and cast triggers (Song of
//! Creation), shadow (CR 702.28), "only if you have no cards in hand" activation
//! restrictions and resolution checks, an intervening "if" (Quest for the Nihil Stone,
//! CR 603.4), an "enters with" condition (Ascendant Packleader, CR 614.12) and mana value
//! (Emrakul's Influence, Spectacle Mage, CR 202.3).

use crate::r_p088_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn kaervek_s_spite_costs_are_paid_on_casting() {
    cr!("601.2h", "701.21a", "118.8");
    ruling!(
        "Kaervek's Spite",
        "As always, you can’t sacrifice things you do not control."
    );
    ruling!(
        "Kaervek's Spite",
        "The sacrifice of all your permanents and discarding of your hand is part of the cost and is paid on casting. You do not have a choice to pay this cost zero times or more than one time in order to multiply the effect."
    );
    supported("Kaervek's Spite");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let stolen = t.battlefield(P1, "Hill Giant");
    crate::r_s06_common::give_control(&mut t, stolen, P0);
    t.hand(P0, "Lightning Bolt");
    t.hand(P0, "Forest");
    add_mana(&mut t, P0, ManaType::B, 3);
    let spite = t.hand(P0, "Kaervek's Spite");
    t.cast(P0, spite).target(P1).go();
    // Paid as it's cast: P0's permanents (including the one P0 controls but doesn't own)
    // are sacrificed and P0's hand is discarded; P1's permanents stay.
    assert_eq!(t.zone(spite), Zone::Stack);
    assert!(!t.on_battlefield(mine));
    assert!(!t.on_battlefield(stolen));
    assert!(t.on_battlefield(theirs));
    assert_eq!(t.hand_size(P0), 0);
    assert_eq!(t.graveyard_size(P0), 3);
    t.resolve_all();
    assert_eq!(t.life(P1), 15);
}

#[test]
fn aeromoeba_switch_applies_after_other_effects() {
    cr!("613.4d", "613.4c");
    ruling!(
        "Aeromoeba",
        "Effects that switch a creature's power and toughness apply after all other effects, regardless of when those effects began to apply. For instance, if you switch Aeromoeba's power and toughness and then give it +2/+0 later in the turn, it's a 4/4 creature, not a 6/2 creature."
    );
    supported("Aeromoeba");
    supported("Brute Strength");
    // Aeromoeba 2/4: switched (4/2), then Brute Strength (+3/+1): 5/5 (2+3 / 4+1,
    // switched), not 7/3.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Aeromoeba");
    let fodder = t.hand(P0, "Island");
    t.answer_choose(P0, &[obj(fodder)]);
    activate_containing(&mut t, P0, a, "Switch").expect("switch");
    t.resolve_all();
    assert_eq!(t.pt(a), (4, 2));
    cast_new(&mut t, P0, "Brute Strength", &[obj(a)]);
    t.resolve_all();
    assert_eq!(t.pt(a), (5, 5));
}

#[test]
fn diamond_lion_is_a_mana_ability_but_cant_pay_for_a_spell_being_cast() {
    cr!("605.1a", "605.3a", "605.3b", "602.5e", "601.2g");
    ruling!(
        "Diamond Lion",
        "Even though you can only activate this as an instant, it is still a mana ability. It doesn't use the stack and cannot be responded to."
    );
    ruling!(
        "Diamond Lion",
        "You can't cast an instant (or activate an ability as an instant) while casting a spell. Therefore, much like Lion's Eye Diamond, you can't activate the ability intending to use the mana to cast a spell from your hand."
    );
    supported("Diamond Lion");
    // While casting Hill Giant ({3}{R}) with only Diamond Lion: no.
    let mut t = TestGame::new(2);
    let lion = t.battlefield(P0, "Diamond Lion");
    let giant = t.hand(P0, "Hill Giant");
    assert!(!can_cast(&mut t, P0, giant, CastMethod::Normal));
    assert!(t.cast(P0, giant).try_go().is_err());
    assert!(t.on_battlefield(lion));
    // With priority: three mana at once, without using the stack.
    let red = Color::ALL.iter().position(|c| *c == Color::Red).unwrap();
    t.answer(P0, DecisionKind::Option, Answer::Index(red));
    t.activate(P0, lion, 0, &[]).expect("activate");
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.player(P0).mana_pool.total(), 3);
    assert!(!t.on_battlefield(lion));
    assert_eq!(t.zone(giant), Zone::Graveyard(P0));
}

/// Casts Conflagrate from P0's hand with X = `x`, the targets and the division.
fn conflagrate(
    t: &mut TestGame,
    x: i64,
    targets: &[Entity],
    split: &[i64],
) -> Result<ObjectId, mtg_engine::casting::Illegal> {
    add_mana(t, P0, ManaType::R, 1);
    add_mana(t, P0, ManaType::C, 2 * x as u32);
    let c = t.hand(P0, "Conflagrate");
    if !split.is_empty() {
        t.answer(P0, DecisionKind::Divide, Answer::Numbers(split.to_vec()));
    }
    t.cast(P0, c).x(x).targets(targets).try_go()
}

#[test]
fn conflagrate_divides_its_damage_as_its_cast() {
    cr!("601.2d", "608.2b", "115.1a", "107.3a");
    ruling!(
        "Conflagrate",
        "If X is 0, Conflagrate has no targets and deals no damage."
    );
    ruling!(
        "Conflagrate",
        "You divide the damage as you cast Conflagrate, not as it resolves. Each target must be assigned at least 1 damage. You can't choose more than X targets and assign 0 damage to a target."
    );
    ruling!(
        "Conflagrate",
        "If some of the targets become illegal for Conflagrate, the original division of damage still applies, but the damage that would have been dealt to illegal targets isn't dealt at all."
    );
    supported("Conflagrate");
    // X = 0: no targets, no damage.
    let mut t = TestGame::new(2);
    let c = conflagrate(&mut t, 0, &[], &[]).expect("cast with X = 0");
    t.resolve_all();
    assert_eq!(t.zone(c), Zone::Graveyard(P0));
    assert_eq!((t.life(P0), t.life(P1)), (20, 20));
    // X = 2: at most two targets are offered, and each gets at least 1 (a division
    // giving one of them 0 isn't accepted).
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    let from = t.asked().len();
    let c = conflagrate(&mut t, 2, &[obj(a), obj(b)], &[2, 0]).expect("cast");
    let max: Vec<u32> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            mtg_engine::decision::Decision::ChooseTargets { max, .. } => Some(*max),
            _ => None,
        })
        .collect();
    assert_eq!(max, vec![2]);
    let divided = t.g.obj(c).stack.as_ref().unwrap().chosen[0].divided.clone();
    assert_eq!(divided, vec![vec![1, 1]]);
    t.resolve_all();
    assert_eq!(t.obj_now(a).damage, 1);
    assert_eq!(t.obj_now(b).damage, 1);
    // X = 3: two to a Bears, one to P1, divided as it's cast; the Bears leaves, P1 still
    // takes only 1.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let from = t.asked().len();
    let c = conflagrate(&mut t, 3, &[obj(a), Entity::Player(P1)], &[2, 1]).expect("cast");
    assert!(t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, mtg_engine::decision::Decision::Divide { .. })));
    assert_eq!(t.zone(c), Zone::Stack);
    t.g.exile_object(a, None);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
}

#[test]
fn song_of_creation_land_plays_and_cast_trigger() {
    cr!("305.2", "305.2a", "603.3", "117.3b");
    ruling!(
        "Song of Creation",
        "Song of Creation’s first ability is cumulative if you control more than one. It’s also cumulative with other effects that let you play additional lands, such as the one from Dryad of the Ilysian Grove."
    );
    ruling!(
        "Song of Creation",
        "Song of Creation’s second ability doesn’t trigger when you cast it because it’s not on the battlefield yet."
    );
    ruling!(
        "Song of Creation",
        "Players can cast spells and activate abilities after the first triggered ability resolves but before the spell that caused it to trigger does."
    );
    supported("Song of Creation");
    supported("Dryad of the Ilysian Grove");
    for second in ["Song of Creation", "Dryad of the Ilysian Grove"] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Song of Creation");
        t.battlefield(P0, second);
        for i in 0..3 {
            let f = t.hand(P0, "Forest");
            assert!(t.play_land(P0, f).is_ok(), "{second}: land {i}");
        }
        let f = t.hand(P0, "Forest");
        assert!(t.play_land(P0, f).is_err(), "{second}: a fourth");
    }
    // Casting Song of Creation itself: no draw.
    let mut t = TestGame::new(2);
    let hand = t.hand_size(P0);
    cast_new(&mut t, P0, "Song of Creation", &[]);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
    // Its trigger for Grizzly Bears resolves first; then P1 may respond to the Bears.
    let bears = cast_new(&mut t, P0, "Grizzly Bears", &[]);
    t.settle();
    t.resolve();
    assert_eq!(t.hand_size(P0), hand + 2);
    assert_eq!(t.g.stack, vec![bears]);
    cast_new(&mut t, P1, "Lightning Bolt", &[Entity::Player(P0)]);
    t.resolve();
    assert_eq!(t.life(P0), 17);
    assert_eq!(t.zone(bears), Zone::Stack);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
}

#[test]
fn shadow_granted_by_discarding() {
    cr!("702.28a", "702.28b", "702.28c");
    ruling!(
        "Thalakos Drifters",
        "The reminder text is only accurate when this card has Shadow."
    );
    ruling!(
        "Trespasser il-Vec",
        "You can activate Trespasser il-Vec's ability as many times as you can afford to pay the cost, even if it already has shadow. Multiple instances of shadow on the same creature are redundant."
    );
    supported("Thalakos Drifters");
    supported("Trespasser il-Vec");
    // Thalakos Drifters without shadow can block a creature without shadow; with it, it
    // can't.
    let blocks = |shadow: bool| {
        let mut t = TestGame::new(2);
        let d = t.battlefield(P1, "Thalakos Drifters");
        if shadow {
            let c = t.hand(P1, "Island");
            t.answer_choose(P1, &[obj(c)]);
            activate_containing(&mut t, P1, d, "shadow").expect("activate");
            t.resolve_all();
            assert!(t.obj_now(d).has_keyword(KeywordKind::Shadow));
        }
        let bears = t.battlefield(P0, "Grizzly Bears");
        crate::r_s01_common::attack_with(&mut t, &[(bears, Entity::Player(P1))]);
        crate::r_s01_common::block_and_finish(&mut t, P1, &[(d, bears)]);
        t.life(P1) == 20
    };
    assert!(blocks(false));
    assert!(!blocks(true));
    // Trespasser il-Vec: twice, with shadow already.
    let mut t = TestGame::new(2);
    let tr = t.battlefield(P0, "Trespasser il-Vec");
    for name in ["Island", "Forest"] {
        let c = t.hand(P0, name);
        t.answer_choose(P0, &[obj(c)]);
        activate_containing(&mut t, P0, tr, "shadow").expect("activate");
        t.resolve_all();
    }
    assert_eq!(t.graveyard_size(P0), 2);
    assert!(t.obj_now(tr).has_keyword(KeywordKind::Shadow));
}

#[test]
fn hellbent_activation_doesnt_check_the_hand_on_resolution() {
    cr!("602.5b", "608.2");
    ruling!(
        "Keldon Megaliths",
        "It doesn't matter how many cards you have in your hand when the ability resolves."
    );
    ruling!(
        "Sea Gate Wreckage",
        "It doesn’t matter how many cards are in your hand as the last ability resolves. For example, if you have no cards in hand and control two Sea Gate Wreckages, you can activate the last ability of each of them. You’ll draw a card as each ability resolves."
    );
    supported("Keldon Megaliths");
    supported("Sea Gate Wreckage");
    // Keldon Megaliths: "{1}{R}, {T}: This land deals 1 damage to any target. Activate only
    // if you have no cards in hand." A card comes into P0's hand before it resolves.
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Keldon Megaliths");
    add_mana(&mut t, P0, ManaType::R, 1);
    colorless(&mut t, P0, 1);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    activate_containing(&mut t, P0, m, "1 damage").expect("activate");
    t.hand(P0, "Forest");
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    // Two Sea Gate Wreckages: "{2}{C}, {T}: Draw a card. Activate only if you have no
    // cards in hand."
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Sea Gate Wreckage");
    let b = t.battlefield(P0, "Sea Gate Wreckage");
    colorless(&mut t, P0, 6);
    activate_containing(&mut t, P0, a, "Draw").expect("first");
    activate_containing(&mut t, P0, b, "Draw").expect("second");
    t.resolve();
    assert_eq!(t.hand_size(P0), 1);
    t.resolve();
    assert_eq!(t.hand_size(P0), 2);
}

#[test]
fn idle_thoughts_checks_the_hand_on_resolution() {
    cr!("608.2", "602.1");
    ruling!(
        "Idle Thoughts",
        "You can activate the ability whether or not you have any cards in your hand. However, you’ll draw a card only if your hand is empty when the ability resolves."
    );
    supported("Idle Thoughts");
    // "{2}: Draw a card if you have no cards in hand."
    let mut t = TestGame::new(2);
    let it = t.battlefield(P0, "Idle Thoughts");
    t.hand(P0, "Forest");
    colorless(&mut t, P0, 2);
    t.activate(P0, it, 0, &[]).expect("with a card in hand");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 1);
    // Empty hand: draws.
    let mut t = TestGame::new(2);
    let it = t.battlefield(P0, "Idle Thoughts");
    colorless(&mut t, P0, 4);
    t.activate(P0, it, 0, &[]).expect("empty hand");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 1);
    // Empty as it's activated, not as it resolves: nothing.
    let card = t.g.players[0].hand[0];
    t.g.discard(P0, card, None);
    t.activate(P0, it, 0, &[]).expect("empty hand");
    t.hand(P0, "Forest");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 1);
}

#[test]
fn quest_for_the_nihil_stone_intervening_if() {
    cr!("603.4", "608.2h", "113.7a");
    ruling!(
        "Quest for the Nihil Stone",
        "The second ability has an “intervening ‘if’ clause.” That means (1) the ability won’t trigger at all unless, as an opponent’s upkeep starts, that player has no cards in hand and Quest for the Nihil Stone has two or more quest counters on it, and (2) the ability will do nothing if that player has at least one card in hand and/or Quest for the Nihil Stone has fewer than two quest counters on it by the time it resolves. (If Quest for the Nihil Stone is no longer on the battlefield by that point, its last existence on the battlefield is checked to see how many quest counters were on it.)"
    );
    supported("Quest for the Nihil Stone");
    // "At the beginning of each opponent's upkeep, if that player has no cards in hand and
    // this enchantment has two or more quest counters on it, you may have that player lose
    // 5 life."
    let setup = |counters: u32, card: bool| {
        let mut t = TestGame::new(2);
        let q = t.battlefield(P0, "Quest for the Nihil Stone");
        t.g.obj_mut(q).counters.insert("quest".into(), counters);
        if card {
            t.hand(P1, "Forest");
        }
        yes(&mut t, P0, 3);
        t.advance_to(P1, Step::Upkeep);
        t.settle();
        (t, q)
    };
    // A card in hand, or one counter: no trigger.
    let (t, _) = setup(2, true);
    assert_eq!(t.stack_len(), 0);
    let (t, _) = setup(1, false);
    assert_eq!(t.stack_len(), 0);
    // It triggers; P1 gets a card before it resolves: nothing.
    let (mut t, _) = setup(2, false);
    assert_eq!(t.stack_len(), 1);
    t.hand(P1, "Forest");
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    // It triggers; a counter is removed: nothing.
    let (mut t, q) = setup(2, false);
    t.g.remove_counters(obj(q), "quest", 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    // It triggers; the Quest is destroyed: its last two counters count.
    let (mut t, q) = setup(2, false);
    destroy(&mut t, q);
    t.resolve_all();
    assert_eq!(t.life(P1), 15);
}

#[test]
fn ascendant_packleader_doesnt_see_permanents_entering_with_it() {
    cr!("614.12", "614.1c");
    ruling!(
        "Ascendant Packleader",
        "Ascendant Packleader's first ability considers only permanents already on the battlefield when determining if it should apply. If Ascendant Packleader is entering the battlefield at the same time as another permanent, the replacement effect won't see that permanent."
    );
    supported("Ascendant Packleader");
    // "This creature enters with a +1/+1 counter on it if you control a permanent with
    // mana value 4 or greater." Hill Giant: mana value 4.
    let mut t = TestGame::new(2);
    let ids = enter_together(&mut t, &[("Ascendant Packleader", P0), ("Hill Giant", P0)]);
    assert_eq!(t.counters(ids[0], counters::PLUS1), 0);
    let p = t.enter(P0, "Ascendant Packleader");
    assert_eq!(t.counters(p, counters::PLUS1), 1);
}

#[test]
fn emrakuls_influence_looks_at_mana_value_not_the_cost_paid() {
    cr!("202.3", "702.119a", "603.2");
    ruling!(
        "Emrakul's Influence",
        "Emrakul's Influence doesn't care about how much mana you paid for an Eldrazi creature spell, only what its mana value is. Cost reductions, such as that of emerge, won't change whether or not it triggers."
    );
    supported("Emrakul's Influence");
    supported("Wretched Gryff");
    // Wretched Gryff ({7}{U}, an Eldrazi; emerge {5}{U}) cast by sacrificing Hill Giant
    // (mana value 4) for {1}{U}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Emrakul's Influence");
    let giant = t.battlefield(P0, "Hill Giant");
    t.lands(P0, "Island", 2);
    t.answer_choose(P0, &[obj(giant)]);
    let gryff = t.hand(P0, "Wretched Gryff");
    t.cast(P0, gryff)
        .method(CastMethod::Keyword(KeywordKind::Emerge))
        .go();
    assert_eq!(t.zone(giant), Zone::Graveyard(P0));
    t.settle();
    assert_eq!(triggers_on_stack(&t, "draw two"), 1);
}

#[test]
fn spectacle_mage_reduces_by_mana_value_and_only_generic() {
    cr!("202.3e", "601.2f", "118.7a");
    ruling!(
        "Spectacle Mage",
        "If there is an {X} in the spell’s mana cost, use the chosen value for X to determine its mana value. For example, if you cast a sorcery with mana cost {X}{U} and choose 4 for X, its mana value is 5."
    );
    ruling!(
        "Spectacle Mage",
        "Spectacle Mage’s second ability only looks at the mana value of a spell, not the cost you’re paying for it. For example, if you control two Spectacle Mages and cast a sorcery with mana cost {4}{U}, the cost will be reduced to {2}{U}."
    );
    ruling!(
        "Spectacle Mage",
        "Spectacle Mage’s second ability can’t reduce the colored mana requirements in spells you cast."
    );
    supported("Spectacle Mage");
    supported("Prosperity");
    supported("Brilliant Plan");
    // Prosperity ({X}{U}) with X = 4: mana value 5, {3}{U}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Spectacle Mage");
    add_mana(&mut t, P0, ManaType::U, 1);
    colorless(&mut t, P0, 3);
    let p = t.hand(P0, "Prosperity");
    t.cast(P0, p).x(4).go();
    assert_eq!(t.zone(p), Zone::Stack);
    assert_eq!(t.player(P0).mana_pool.total(), 0);
    // With X = 3 (mana value 4): no reduction; {2}{U} isn't enough for {3}{U}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Spectacle Mage");
    add_mana(&mut t, P0, ManaType::U, 1);
    colorless(&mut t, P0, 2);
    let p = t.hand(P0, "Prosperity");
    assert!(t.cast(P0, p).x(3).try_go().is_err());
    // Two Mages, Brilliant Plan ({4}{U}): {2}{U}, not less although the cost paid is
    // under 5.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Spectacle Mage");
    t.battlefield(P0, "Spectacle Mage");
    add_mana(&mut t, P0, ManaType::U, 1);
    colorless(&mut t, P0, 1);
    let bp = t.hand(P0, "Brilliant Plan");
    assert!(t.cast(P0, bp).try_go().is_err());
    colorless(&mut t, P0, 1);
    let bp = t.hand(P0, "Brilliant Plan");
    t.cast(P0, bp).go();
    assert_eq!(t.zone(bp), Zone::Stack);
    assert_eq!(t.player(P0).mana_pool.total(), 0);
    // Brilliant Plan with five Mages: {U}, never less than its colored part ({5} of
    // colorless mana can't pay it).
    let mut t = TestGame::new(2);
    for _ in 0..5 {
        t.battlefield(P0, "Spectacle Mage");
    }
    colorless(&mut t, P0, 5);
    let bp = t.hand(P0, "Brilliant Plan");
    assert!(t.cast(P0, bp).try_go().is_err());
    add_mana(&mut t, P0, ManaType::U, 1);
    t.cast(P0, bp).go();
    assert_eq!(t.zone(bp), Zone::Stack);
    assert_eq!(t.player(P0).mana_pool.total(), 5);
}
