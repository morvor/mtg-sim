//! Rulings batch P155 — more interactions: changing a spell's target (CR 115.7a), an
//! animation with no duration (CR 611.2a), "do this only once each turn" (CR 603.2h),
//! the legend rule with Brothers Yamazaki (CR 704.5j), an intervening-if upkeep trigger
//! (CR 603.4), lethal damage by power (Zilortha, CR 704.5f, 704.5g), a repeatable
//! process (CR 608.2c), a cost locked in before mana abilities (CR 601.2f, 601.2g),
//! kicker "and/or" (CR 702.33d), delve after the total cost (CR 702.66b), restricted
//! mana (CR 106.6a) and casting with no window for other players (CR 601.2).

use crate::r_p155_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn imps_mischief_changes_the_target_on_resolution_to_a_legal_one_but_never_to_the_spell_itself() {
    cr!("115.7a", "115.5", "608.2b");
    ruling!("Imp's Mischief", "You can't make a spell which is on the stack target itself.");
    ruling!(
        "Imp's Mischief",
        "You don't choose the new target for the spell until Imp's Mischief resolves. You must change the target if possible. However, you can't change the target to an illegal target. If there are no legal targets to choose from, the target isn't changed. It doesn't matter if the original target has somehow become illegal itself."
    );
    supported("Imp's Mischief");
    // P1's Shock at P0's Bears; the Bears die in response: the target must still change
    // (to Hill Giant or a player: Hill Giant is chosen); the new target is asked only as
    // Imp's Mischief resolves; P0 loses 1 (Shock's mana value).
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let shock = cast_at(&mut t, P1, "Shock", &[obj(bears)]);
    let mischief = cast_at(&mut t, P0, "Imp's Mischief", &[obj(shock)]);
    let from = t.asked().len();
    assert!(target_choices(&t, P0, 0, "New target").is_empty());
    kill(&mut t, bears);
    t.answer_targets(P0, &[obj(giant)]);
    t.resolve();
    assert!(!on_stack(&t, mischief));
    assert_eq!(target_choices(&t, P0, from, "New target").len(), 1);
    assert_eq!(stack_targets(&t, shock), vec![obj(giant)]);
    assert_eq!(t.life(P0), 19);
    t.resolve();
    assert_eq!(dmg(&t, giant), 2);
    // P1's Cancel at P0's Shock: the Cancel can be made to target Imp's Mischief, not
    // itself.
    let mut t = TestGame::new(2);
    let shock = cast_at(&mut t, P0, "Shock", &[pl(P1)]);
    let cancel = cast_at(&mut t, P1, "Cancel", &[obj(shock)]);
    let mischief = cast_at(&mut t, P0, "Imp's Mischief", &[obj(cancel)]);
    let from = t.asked().len();
    t.answer_targets(P0, &[obj(mischief)]);
    t.resolve();
    let offered = &target_choices(&t, P0, from, "New target")[0];
    assert!(offered.contains(&obj(mischief)) && !offered.contains(&obj(cancel)), "{offered:?}");
    assert_eq!(stack_targets(&t, cancel), vec![obj(mischief)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 18, "Shock resolved; Cancel had no target left");
    // P1's Naturalize at P0's only artifact: no other legal target, so it's unchanged.
    let mut t = TestGame::new(2);
    let ring = t.battlefield(P0, "Sol Ring");
    let nat = cast_at(&mut t, P1, "Naturalize", &[obj(ring)]);
    cast_at(&mut t, P0, "Imp's Mischief", &[obj(nat)]);
    let from = t.asked().len();
    t.resolve();
    assert!(target_choices(&t, P0, from, "New target").iter().all(|c| c.is_empty()), "{:?}", target_choices(&t, P0, from, "New target"));
    assert_eq!(stack_targets(&t, nat), vec![obj(ring)]);
    assert_eq!(t.life(P0), 18, "Naturalize's mana value");
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Sol Ring"));
}

#[test]
fn zurs_animation_has_no_duration() {
    cr!("611.2a", "611.2b");
    ruling!(
        "Zur, Eternal Schemer",
        "Zur, Eternal Schemer’s last ability does not have a duration. Once it resolves the target enchantment will be a creature until it leaves the battlefield or another effect causes it to stop being a creature."
    );
    supported("Zur, Eternal Schemer");
    let mut t = TestGame::new(2);
    let zur = t.battlefield(P0, "Zur, Eternal Schemer");
    let arena = t.battlefield(P0, "Stronghold Arena");
    activate_resolve(&mut t, P0, zur, "becomes a creature", "1W", &[obj(arena)]);
    assert!(crate::r_s04_common::is_creature(&t, arena));
    assert_eq!(t.pt(arena), (2, 2));
    // Still a creature on later turns.
    t.advance_to(P1, Step::PrecombatMain);
    t.advance_to(P0, Step::PrecombatMain);
    assert!(crate::r_s04_common::is_creature(&t, arena));
    assert_eq!(t.pt(arena), (2, 2));
    // Even after Zur has left.
    kill(&mut t, zur);
    assert!(crate::r_s04_common::is_creature(&t, arena));
}

#[test]
fn riveteers_ascendancy_does_it_once_each_turn_per_ascendancy_but_triggers_each_time() {
    cr!("603.2h", "603.2");
    ruling!(
        "Riveteers Ascendancy",
        "“Do this only once each turn” refers only to the ability it is attached to, not other instances of the same ability. In other words, if you control multiple copies of Riveteers Ascendancy, you will be able to do this once for each of them."
    );
    ruling!(
        "Riveteers Ascendancy",
        "“Do this only once each turn” refers to returning a creature card from your graveyard to the battlefield using the triggered ability. As long as you haven't yet chosen to do that this turn, all instances of sacrificing a creature will cause Riveteers Ascendancy's ability to trigger."
    );
    supported("Riveteers Ascendancy");
    // Two creatures sacrificed before either trigger resolves: both triggers go on the
    // stack (nothing has been returned yet); only the first to resolve returns a card.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Riveteers Ascendancy");
    let g1 = t.battlefield(P0, "Hill Giant");
    let g2 = t.battlefield(P0, "Hill Giant");
    let bears = t.graveyard(P0, "Grizzly Bears");
    let courser = t.graveyard(P0, "Centaur Courser");
    t.answer_targets(P0, &[obj(bears)]);
    sacrifice(&mut t, P0, g1);
    assert_eq!(t.stack_len(), 1, "{}", t.dump_log());
    t.answer_targets(P0, &[obj(courser)]);
    sacrifice(&mut t, P0, g2);
    assert_eq!(t.stack_len(), 2, "still triggers: {}", t.dump_log());
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(count_named(&t, "Centaur Courser"), 1, "{}", t.dump_log());
    assert_eq!(count_named(&t, "Grizzly Bears"), 0, "only once each turn: {}", t.dump_log());
    assert_eq!(t.zone(bears), Zone::Graveyard(P0));
    // Once a card was returned this turn, a later sacrifice doesn't trigger it again
    // (CR 603.2h).
    let g3 = t.battlefield(P0, "Hill Giant");
    sacrifice(&mut t, P0, g3);
    assert_eq!(t.stack_len(), 0);
    // Two Ascendancies: one creature sacrificed returns two cards (one per Ascendancy).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Riveteers Ascendancy");
    t.battlefield(P0, "Riveteers Ascendancy");
    let g1 = t.battlefield(P0, "Hill Giant");
    let bears = t.graveyard(P0, "Grizzly Bears");
    let courser = t.graveyard(P0, "Centaur Courser");
    t.answer_targets(P0, &[obj(bears)]);
    t.answer_targets(P0, &[obj(courser)]);
    sacrifice(&mut t, P0, g1);
    assert_eq!(t.stack_len(), 2);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(count_named(&t, "Grizzly Bears"), 1, "{}", t.dump_log());
    assert_eq!(count_named(&t, "Centaur Courser"), 1, "{}", t.dump_log());
}

#[test]
fn brothers_yamazaki_two_under_different_players_and_the_legend_rule_with_a_third() {
    cr!("704.5j", "613.1f");
    ruling!(
        "Brothers Yamazaki",
        "If a third Brothers Yamazaki enters the battlefield, the legend rule will apply. If one player controls more than one Brothers Yamazaki at that time, that player will have to choose one and put the rest into their graveyard."
    );
    ruling!("Brothers Yamazaki", "The two Brothers could be controlled by different players.");
    supported("Brothers Yamazaki");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Brothers Yamazaki");
    let b = t.battlefield(P1, "Brothers Yamazaki");
    t.settle();
    assert!(t.on_battlefield(a) && t.on_battlefield(b));
    assert_eq!(t.pt(a), (4, 3));
    assert_eq!(t.pt(b), (4, 3));
    assert!(t.obj_now(a).chars.has_keyword(KeywordKind::Haste));
    // A third, under P0: P0 keeps one of theirs; P1's stays.
    let c = t.battlefield(P0, "Brothers Yamazaki");
    t.answer_choose(P0, &[obj(c)]);
    t.settle();
    assert!(!t.on_battlefield(a), "{}", t.dump_log());
    assert!(t.on_battlefield(b) && t.on_battlefield(c));
    assert_eq!(t.pt(b), (4, 3));
}

#[test]
fn myr_servitor_must_still_be_on_the_battlefield_as_its_ability_resolves() {
    cr!("603.4");
    ruling!(
        "Myr Servitor",
        "The Myr Servitor that's the source of the triggered ability must still be on the battlefield when the ability resolves. If it is, all other Myr Servitor cards are returned from all graveyards to the battlefield under their owner's control. If it's not, no Myr Servitors are returned to the battlefield."
    );
    supported("Myr Servitor");
    for dies in [false, true] {
        let mut t = TestGame::new(2);
        let myr = t.battlefield(P0, "Myr Servitor");
        let mine = t.graveyard(P0, "Myr Servitor");
        let theirs = t.graveyard(P1, "Myr Servitor");
        crate::r_s04_common::next_upkeep(&mut t, P0);
        assert_eq!(t.stack_len(), 1);
        if dies {
            kill(&mut t, myr);
        }
        t.resolve_all();
        assert_eq!(t.on_battlefield(mine), !dies, "dies: {dies}: {}", t.dump_log());
        assert_eq!(t.on_battlefield(theirs), !dies, "dies: {dies}");
        if !dies {
            assert_eq!(t.obj_now(theirs).controller, P1, "under its owner's control");
        }
    }
}

#[test]
fn zilortha_doesnt_change_the_toughness_0_action_and_marked_damage_is_rechecked() {
    cr!("704.5f", "704.5g", "120.6");
    ruling!(
        "Zilortha, Strength Incarnate",
        "A creature with 0 toughness is put into its owner’s graveyard. This state-based action isn’t a consequence of damage, so Zilortha doesn’t affect it."
    );
    ruling!(
        "Zilortha, Strength Incarnate",
        "Because damage remains marked on a creature until the damage is removed as the turn ends, nonlethal damage dealt to a creature you control may become lethal if Zilortha enters or leaves the battlefield during that turn."
    );
    supported("Dead Weight");
    // Craw Wurm (6/4) with two Dead Weights: 2/0, it dies with no damage on it.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Zilortha, Strength Incarnate");
    let wurm = t.battlefield(P0, "Craw Wurm");
    crate::r_s06_common::attach_new(&mut t, P1, "Dead Weight", wurm);
    t.settle();
    assert_eq!(t.pt(wurm), (4, 2));
    assert!(t.on_battlefield(wurm));
    crate::r_s06_common::attach_new(&mut t, P1, "Dead Weight", wurm);
    t.settle();
    assert!(t.in_graveyard(P0, "Craw Wurm"), "{}", t.dump_log());
    // Horned Turtle (1/4) with 2 damage survives; Zilortha enters: lethal by power.
    let mut t = TestGame::new(2);
    let turtle = t.battlefield(P0, "Horned Turtle");
    let src = t.battlefield(P1, "Grizzly Bears");
    damage(&mut t, src, turtle, 2);
    assert!(t.on_battlefield(turtle));
    t.battlefield(P0, "Zilortha, Strength Incarnate");
    t.settle();
    assert!(t.in_graveyard(P0, "Horned Turtle"), "{}", t.dump_log());
    // Craw Wurm (6/4) with 5 damage survives under Zilortha; Zilortha leaves: lethal.
    let mut t = TestGame::new(2);
    let z = t.battlefield(P0, "Zilortha, Strength Incarnate");
    let wurm = t.battlefield(P0, "Craw Wurm");
    let src = t.battlefield(P1, "Grizzly Bears");
    damage(&mut t, src, wurm, 5);
    assert!(t.on_battlefield(wurm), "{}", t.dump_log());
    kill(&mut t, z);
    assert!(t.in_graveyard(P0, "Craw Wurm"), "{}", t.dump_log());
}

#[test]
fn kindle_the_carnage_asks_after_each_discard_and_its_damage() {
    cr!("608.2c");
    ruling!(
        "Kindle the Carnage",
        "You decide whether to repeat the process after each iteration. In other words, you discard a card, see what it is, Kindle the Carnage deals damage, then you decide whether to continue. If so, you discard a card, see what it is, Kindle the Carnage deals damage, then you decide whether to continue, and so on."
    );
    supported("Kindle the Carnage");
    let mut t = TestGame::new(2);
    t.hand(P0, "Hill Giant");
    t.hand(P0, "Hill Giant");
    let wurm = t.battlefield(P1, "Craw Wurm");
    let maw = t.battlefield(P1, "Colossal Dreadmaw");
    let from = t.asked().len();
    // Continue after the first discard (4 damage: the Wurm dies), stop after the second
    // (8: the Dreadmaw dies).
    t.answer_yes(P0, true);
    t.answer_yes(P0, false);
    cast_resolve(&mut t, P0, "Kindle the Carnage", "1RR", &[]);
    assert!(t.in_graveyard(P1, "Craw Wurm") && t.in_graveyard(P1, "Colossal Dreadmaw"), "{}", t.dump_log());
    assert_eq!(t.hand_size(P0), 0);
    assert_eq!(yes_nos_asked(&t, P0, from), 2, "{:?}", asked_of(&t, P0, from));
    // Stopping after the first: the Dreadmaw lives, a Giant stays in hand.
    let mut t = TestGame::new(2);
    t.hand(P0, "Hill Giant");
    t.hand(P0, "Hill Giant");
    t.battlefield(P1, "Craw Wurm");
    t.battlefield(P1, "Colossal Dreadmaw");
    t.answer_yes(P0, false);
    cast_resolve(&mut t, P0, "Kindle the Carnage", "1RR", &[]);
    assert!(t.in_graveyard(P1, "Craw Wurm"));
    assert_eq!(count_named(&t, "Colossal Dreadmaw"), 1);
    assert_eq!(t.hand_size(P0), 1);
    let _ = (wurm, maw);
}

#[test]
fn cost_reductions_are_locked_in_before_mana_abilities_sacrifice_what_they_counted() {
    cr!("601.2f", "601.2g", "601.2h");
    ruling!(
        "Earthquake Dragon",
        "The total cost to cast Earthquake Dragon is locked in before you pay that cost. For example, if you control three Dragons, each with mana value 4, including one you can sacrifice to add {C}, the total cost of Earthquake Dragon is {2}{G}. Then you can sacrifice the Dragon as you activate mana abilities just before paying the cost."
    );
    ruling!(
        "Metalwork Colossus",
        "The total cost to cast Metalwork Colossus is locked in before you pay that cost. For example, if you control three noncreature artifacts each with mana value 2, including one you can sacrifice to add {C} , the total cost of Metalwork Colossus is {5}. Then you can sacrifice the artifact when you activate mana abilities just before paying the cost."
    );
    supported("Earthquake Dragon");
    supported("Furnace Whelp");
    supported("Ashnod's Altar");
    // Three Furnace Whelps ({2}{R}{R}): Earthquake Dragon ({14}{G}) costs {2}{G}, paid by
    // sacrificing a Whelp to Ashnod's Altar ({C}{C}) and a Forest.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ashnod's Altar");
    let whelps: Vec<ObjectId> = (0..3).map(|_| t.battlefield(P0, "Furnace Whelp")).collect();
    t.lands(P0, "Forest", 1);
    let dragon = t.hand(P0, "Earthquake Dragon");
    t.cast(P0, dragon).go();
    assert_eq!(t.zone(dragon), Zone::Stack, "{}", t.dump_log());
    assert_eq!(whelps.iter().filter(|w| t.on_battlefield(**w)).count(), 2);
    t.resolve_all();
    assert_eq!(count_named(&t, "Earthquake Dragon"), 1);
    // Krark-Clan Ironworks ({4}) and two Mind Stones ({2}): Metalwork Colossus ({11})
    // costs {3}, paid by sacrificing a Mind Stone to the Ironworks ({C}{C}) and tapping
    // the other.
    supported("Metalwork Colossus");
    supported("Krark-Clan Ironworks");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Krark-Clan Ironworks");
    let stones: Vec<ObjectId> = (0..2).map(|_| t.battlefield(P0, "Mind Stone")).collect();
    let colossus = t.hand(P0, "Metalwork Colossus");
    t.cast(P0, colossus).go();
    assert_eq!(t.zone(colossus), Zone::Stack, "{}", t.dump_log());
    // A counted artifact (a Mind Stone, or the Ironworks itself) was sacrificed to pay.
    let counted: u32 = t
        .g
        .permanents()
        .filter(|o| o.controller == P0 && o.is(CardType::Artifact) && !o.is(CardType::Creature))
        .map(|o| t.g.mana_value_of(o.id))
        .sum();
    assert!(counted < 8, "{counted}: {}", t.dump_log());
    let _ = stones;
    t.resolve_all();
    assert_eq!(count_named(&t, "Metalwork Colossus"), 1);
}

#[test]
fn stronghold_arena_is_kicked_once_for_each_of_its_two_kicker_costs() {
    cr!("702.33d");
    ruling!(
        "Stronghold Arena",
        "You may kick Stronghold Arena only once for its {G} cost and only once for its {W} cost. You can kick it twice by paying {G}{W}, but you can't kick it twice by paying {G}{G} or {W}{W}. You can't kick it more than twice."
    );
    supported("Stronghold Arena");
    for (kicks, life) in [([true, true], 26), ([true, false], 23), ([false, false], 20)] {
        let mut t = TestGame::new(2);
        let arena = t.hand(P0, "Stronghold Arena");
        mana(&mut t, P0, "1BGW");
        let from = t.asked().len();
        for k in kicks {
            t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(k));
        }
        t.cast(P0, arena).go();
        t.resolve_all();
        assert_eq!(t.life(P0), life, "{kicks:?}: {}", t.dump_log());
        let asked = asked_of(&t, P0, from)
            .iter()
            .filter(|d| matches!(d, Decision::OptionalCost { .. }))
            .count();
        assert!(asked <= 2, "{kicks:?}: each kicker once: {:?}", asked_of(&t, P0, from));
        assert_eq!(t.obj_now(arena).cast.as_ref().map(|c| c.times_kicked), Some(kicks.iter().filter(|k| **k).count() as u32));
    }
}

#[test]
fn tevals_delve_applies_after_the_total_cost_is_determined() {
    cr!("702.66a", "702.66b", "601.2f");
    ruling!(
        "Teval, Arbiter of Virtue",
        "“Delve” means “For each generic mana in this spell’s total cost, you may exile a card from your graveyard rather than pay that mana.” The delve ability isn’t an additional or alternative cost and applies only after the total cost of the spell with delve is determined."
    );
    supported("Teval, Arbiter of Virtue");
    supported("Thalia, Guardian of Thraben");
    // Lava Spike ({R}) costs {1}{R} with Thalia; the {1} of the total cost is paid by
    // exiling the Forest from the graveyard; it's cast normally, and Teval has P0 lose 1
    // (its mana value).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Teval, Arbiter of Virtue");
    t.battlefield(P0, "Thalia, Guardian of Thraben");
    let forest = t.graveyard(P0, "Forest");
    mana(&mut t, P0, "R");
    let spike = t.hand(P0, "Lava Spike");
    assert!(can_cast(&mut t, P0, spike, CastMethod::Normal));
    t.answer_choose(P0, &[obj(forest)]);
    t.cast(P0, spike).target(pl(P1)).go();
    assert_eq!(t.zone(forest), Zone::Exile, "{}", t.dump_log());
    assert_eq!(pool(&t, P0), 0);
    t.resolve_all();
    assert_eq!(t.life(P0), 19);
    assert_eq!(t.life(P1), 17);
    // Without a card to exile, {1}{R} can't be paid from {R}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Teval, Arbiter of Virtue");
    t.battlefield(P0, "Thalia, Guardian of Thraben");
    mana(&mut t, P0, "R");
    let spike = t.hand(P0, "Lava Spike");
    assert!(!can_cast(&mut t, P0, spike, CastMethod::Normal));
}

#[test]
fn karns_mana_stays_through_the_turn_and_pays_for_anything_but_nonartifact_spells() {
    cr!("106.6", "106.4");
    ruling!(
        "Karn, Legacy Reforged",
        "The mana produced by Karn's last ability can be spent on anything that isn't a nonartifact spell. This includes casting artifact spells, paying costs to activate abilities of both artifact and nonartifact permanents, paying ward costs, and so on."
    );
    supported("Karn, Legacy Reforged");
    supported("Mishra's Factory");
    supported("Kappa Cannoneer");
    // Karn and three Memnites: {C}{C}{C}{C} at upkeep, still there in the main phase.
    fn karn_mana(t: &mut TestGame) {
        t.battlefield(P0, "Karn, Legacy Reforged");
        for _ in 0..3 {
            t.battlefield(P0, "Memnite");
        }
        t.set_step(P1, Step::End);
        t.advance_to(P0, Step::Upkeep);
        t.resolve_all();
        assert_eq!(pool(t, P0), 4);
        t.advance_to(P0, Step::PrecombatMain);
        assert_eq!(pool(t, P0), 4, "not lost as the upkeep ended");
    }
    // An artifact spell (Juggernaut, {4}) can be cast; a creature spell can't, even
    // with a Forest for its {G}.
    let mut t = TestGame::new(2);
    karn_mana(&mut t);
    t.lands(P0, "Forest", 1);
    let jugg = t.hand(P0, "Juggernaut");
    let bears = t.hand(P0, "Grizzly Bears");
    assert!(t.cast(P0, bears).try_go().is_err(), "{{1}}{{G}} can't use Karn's mana");
    assert_eq!(pool(&t, P0), 4);
    assert!(can_cast(&mut t, P0, jugg, CastMethod::Normal));
    t.cast(P0, jugg).go();
    assert_eq!(pool(&t, P0), 0);
    // The ability of a nonartifact permanent (Mishra's Factory's {1}) and of an artifact
    // (Walking Ballista's {4}).
    let mut t = TestGame::new(2);
    karn_mana(&mut t);
    let factory = t.battlefield(P0, "Mishra's Factory");
    act(&mut t, P0, factory, "becomes a", &[]).unwrap();
    t.resolve_all();
    assert!(crate::r_s04_common::is_creature(&t, factory));
    assert_eq!(pool(&t, P0), 3);
    let mut t = TestGame::new(2);
    karn_mana(&mut t);
    let ballista = t.battlefield(P0, "Walking Ballista");
    put(&mut t, ballista, counters::PLUS1, 1);
    act(&mut t, P0, ballista, "Put a +1/+1 counter", &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(ballista, counters::PLUS1), 2);
    assert_eq!(pool(&t, P0), 0);
    // A ward cost: Kappa Cannoneer's ward {4}, paid for Walking Ballista's ping.
    let mut t = TestGame::new(2);
    karn_mana(&mut t);
    let ballista = t.battlefield(P0, "Walking Ballista");
    put(&mut t, ballista, counters::PLUS1, 1);
    let kappa = t.battlefield(P1, "Kappa Cannoneer");
    t.answer_yes(P0, true);
    act(&mut t, P0, ballista, "deals 1 damage", &[obj(kappa)]).unwrap();
    t.resolve_all();
    assert_eq!(pool(&t, P0), 0, "{}", t.dump_log());
    assert_eq!(dmg(&t, kappa), 1, "the ward cost was paid: {}", t.dump_log());
}

#[test]
fn karns_power_and_toughness_are_defined_in_all_zones() {
    cr!("604.3", "604.3a", "208.2");
    ruling!(
        "Karn, Legacy Reforged",
        "The ability that defines Karn's power and toughness works in all zones, not just the battlefield. As long as Karn is under your control and still an artifact, its own mana value will count. In most cases, it'll be at least 5/5."
    );
    supported("Karn, Legacy Reforged");
    let mut t = TestGame::new(2);
    let in_hand = t.hand(P0, "Karn, Legacy Reforged");
    assert_eq!(t.pt(in_hand), (0, 0), "no artifacts");
    t.battlefield(P0, "Mind Stone");
    assert_eq!(t.pt(in_hand), (2, 2));
    let karn = t.battlefield(P0, "Karn, Legacy Reforged");
    assert_eq!(t.pt(karn), (5, 5), "its own mana value");
    assert_eq!(t.pt(in_hand), (5, 5));
    t.battlefield(P0, "Darksteel Forge");
    assert_eq!(t.pt(karn), (9, 9));
}

#[test]
fn tormented_thoughts_gives_no_player_a_window_before_the_sacrifice() {
    cr!("601.2", "601.2h", "117.3c");
    ruling!(
        "Tormented Thoughts",
        "Players can only respond once this spell has been cast and all its costs have been paid. No one can try to destroy the creature you sacrifice to stop you from casting this spell."
    );
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    t.hand(P1, "Shock");
    mana(&mut t, P1, "R");
    mana(&mut t, P0, "2B");
    let tt = t.hand(P0, "Tormented Thoughts");
    let from = t.asked().len();
    t.answer_choose(P0, &[obj(giant)]);
    t.answer_targets(P0, &[pl(P1)]);
    t.cast(P0, tt).go();
    assert!(others_asked_since(&t, from, P0).is_empty());
    assert!(t.in_graveyard(P0, "Hill Giant"), "sacrificed as the spell was cast");
    assert_eq!(t.zone(tt), Zone::Stack);
}
