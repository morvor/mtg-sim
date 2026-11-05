//! Rulings batch P195 — permanents that untap themselves: by an activated ability
//! (Basalt Monolith, Pili-Pala, Devoted Druid, Chained Brute, Seeker of Skybreak,
//! Torchling), by a triggered ability with an intervening "if" (Sabertooth Mauler,
//! Ragost, Betor, Foreboding Statue, Hydro-Man), in combat (Lattice-Blade Mantis,
//! Mage-Ring Responder), or in other players' untap steps (Thousand Moons Infantry); and
//! "you may tap or untap" chosen on resolution (Elder Druid, Puppeteer).

use crate::r_p195_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn prize_pig_removes_the_counters_even_if_already_untapped() {
    cr!("122.8", "701.26b");
    ruling!(
        "Prize Pig",
        "If there are three or more ribbon counters on Prize Pig after you put ribbon counters on it, all of the counters will be removed. It does not matter whether or not Prize Pig is already untapped."
    );
    supported("Prize Pig");
    for start_tapped in [true, false] {
        let mut t = TestGame::new(2);
        let p = t.battlefield(P0, "Prize Pig");
        if start_tapped {
            tap(&mut t, p);
        }
        t.g.gain_life(P0, 2);
        t.g.flush_events();
        t.resolve_all();
        assert_eq!(t.counters(p, "ribbon"), 2);
        t.g.gain_life(P0, 2);
        t.g.flush_events();
        t.resolve_all();
        assert_eq!(t.counters(p, "ribbon"), 0, "tapped={start_tapped}");
        assert!(!tapped(&t, p));
    }
}

/// P0's end step, with whatever triggers there resolved.
fn to_end_step(t: &mut TestGame) {
    t.advance_to(P0, Step::End);
    t.resolve_all();
}

#[test]
fn hydro_man_becomes_a_land_until_your_next_turn() {
    cr!("302.6", "305.1", "611.2b", "514.3");
    ruling!(
        "Hydro-Man, Fluid Felon",
        "After the triggered ability resolves during your end step, however, you may tap him for {U}, even if he entered the battlefield this turn."
    );
    ruling!(
        "Hydro-Man, Fluid Felon",
        "The effect of Hydro-Man's last ability ends as your turn begins. There is no opportunity to tap Hydro-Man for {U} once your turn has started."
    );
    supported("Hydro-Man, Fluid Felon");
    let mut t = TestGame::new(2);
    let h = t.battlefield_sick(P0, "Hydro-Man, Fluid Felon");
    tap(&mut t, h);
    to_end_step(&mut t);
    assert!(!tapped(&t, h));
    assert!(!t.obj_now(h).is(CardType::Creature));
    assert!(t.obj_now(h).is(CardType::Land));
    assert!(t.obj_now(h).summoning_sick);
    let before = t.g.player(P0).mana_pool.total();
    t.activate(P0, h, 0, &[]).expect("tap for {U}");
    assert_eq!(t.g.player(P0).mana_pool.total(), before + 1);
    // Still a land during the opponent's turn; a creature again as your turn begins.
    t.advance_to(P1, Step::PrecombatMain);
    assert!(t.obj_now(h).is(CardType::Land));
    t.advance_to(P0, Step::Upkeep);
    assert!(t.obj_now(h).is(CardType::Creature));
    assert!(!t.obj_now(h).is(CardType::Land));
    let mana_abilities = t
        .obj_now(h)
        .chars
        .abilities
        .iter()
        .filter(|a| a.is_mana_ability())
        .count();
    assert_eq!(mana_abilities, 0);
}

#[test]
fn foreboding_statue_transforms_as_it_untaps() {
    cr!("701.27a", "712.8e");
    ruling!(
        "Foreboding Statue // Forsaken Thresher",
        "You can't tap Foreboding Statue to get one more mana while resolving its triggered ability in your end step."
    );
    supported("Foreboding Statue // Forsaken Thresher");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Foreboding Statue // Forsaken Thresher");
    t.g.objects[s.0 as usize]
        .counters
        .insert("omen".into(), 2);
    t.activate(P0, s, 0, &[]).unwrap();
    assert_eq!(t.counters(s, "omen"), 3);
    t.advance_to(P0, Step::End);
    t.settle();
    assert!(top_is_trigger_of(&t, s));
    t.resolve();
    // Untapped and transformed in one resolution: the back face has no mana ability.
    assert!(!tapped(&t, s));
    assert_eq!(t.obj_now(s).chars.name, "Forsaken Thresher");
    assert!(!t
        .obj_now(s)
        .chars
        .abilities
        .iter()
        .any(|a| a.is_mana_ability()));
}

#[test]
fn sabertooth_mauler_counts_deaths_before_it_entered() {
    cr!("603.4", "700.4");
    ruling!(
        "Sabertooth Mauler",
        "If a creature didn't die before your end step begins, Sabertooth Mauler's ability doesn't trigger at all. The creature may have died before Sabertooth Mauler entered the battlefield, however."
    );
    supported("Sabertooth Mauler");
    // No death: no trigger.
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Sabertooth Mauler");
    tap(&mut t, m);
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 0);
    // A death before it entered: triggers.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    kill(&mut t, bears);
    let m = t.battlefield(P0, "Sabertooth Mauler");
    tap(&mut t, m);
    to_end_step(&mut t);
    assert!(!tapped(&t, m));
    assert_eq!(t.counters(m, "+1/+1"), 1);
}

#[test]
fn ragost_checks_life_gained_as_the_end_step_starts() {
    cr!("603.4", "513.1");
    ruling!(
        "Ragost, Deft Gastronaut",
        "Ragost’s last ability will check as the end step starts to see if you gained life this turn. If you haven’t, the ability won’t trigger at all. You won’t be able to gain life during your end step in time to have the ability trigger."
    );
    supported("Ragost, Deft Gastronaut");
    let mut t = TestGame::new(2);
    let r = t.battlefield(P0, "Ragost, Deft Gastronaut");
    tap(&mut t, r);
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 0);
    t.g.gain_life(P0, 3);
    t.g.flush_events();
    t.resolve_all();
    assert!(tapped(&t, r));
    // Life gained earlier in the turn: untaps.
    let mut t = TestGame::new(2);
    let r = t.battlefield(P0, "Ragost, Deft Gastronaut");
    tap(&mut t, r);
    t.g.gain_life(P0, 1);
    t.g.flush_events();
    to_end_step(&mut t);
    assert!(!tapped(&t, r));
}

#[test]
fn ragost_makes_every_artifact_a_food() {
    cr!("205.3g", "613.8", "602.2b");
    ruling!(
        "Ragost, Deft Gastronaut",
        "If Ragost somehow becomes an artifact, Ragost will also be a Food."
    );
    ruling!(
        "Ragost, Deft Gastronaut",
        "If an effect refers to a Food, it means any Food artifact, not just a Food token. For example, when you activate Ragost’s second ability, you can sacrifice any Food you control to pay its cost."
    );
    ruling!(
        "Ragost, Deft Gastronaut",
        "You can’t sacrifice a Food to pay multiple costs. For example, you can’t sacrifice a single Food to pay the cost of its own ability and also to pay the cost of Ragost’s second ability."
    );
    supported("Mycosynth Lattice");
    // Mycosynth Lattice makes Ragost an artifact, so it's a Food too.
    let mut t = TestGame::new(2);
    let r = t.battlefield(P0, "Ragost, Deft Gastronaut");
    assert!(!t.obj_now(r).chars.has_subtype("Food"));
    t.battlefield(P0, "Mycosynth Lattice");
    assert!(t.obj_now(r).is(CardType::Artifact));
    assert!(t.obj_now(r).chars.has_subtype("Food"));
    // A nontoken artifact (Ornithopter) is a Food and pays Ragost's cost.
    let mut t = TestGame::new(2);
    let r = t.battlefield(P0, "Ragost, Deft Gastronaut");
    let o = t.battlefield(P0, "Ornithopter");
    assert!(t.obj_now(o).chars.has_subtype("Food"));
    t.answer_choose(P0, &[obj(o)]);
    act(&mut t, P0, r, 0, &[]);
    assert_eq!(t.life(P1), 17);
    assert!(t.in_graveyard(P0, "Ornithopter"));
    // One Food can't pay two costs: after it's sacrificed for its own ability, Ragost's
    // ability can't be activated.
    let mut t = TestGame::new(2);
    let r = t.battlefield(P0, "Ragost, Deft Gastronaut");
    let o = t.battlefield(P0, "Ornithopter");
    let food = t
        .obj_now(o)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, mtg_engine::ability::AbilityKind::Activated(_)))
        .count();
    assert_eq!(food, 1);
    act(&mut t, P0, o, 0, &[]);
    assert_eq!(t.life(P0), 23);
    lots(&mut t, P0);
    assert!(t.activate(P0, r, 0, &[]).is_err());
}

#[test]
fn betor_checks_ten_on_trigger_and_twenty_on_resolution() {
    cr!("603.4", "608.2c");
    ruling!(
        "Betor, Kin to All",
        "Betor’s last ability will check as the end step starts to see if creatures you control have total toughness 10 or greater. If they don’t, the ability won’t trigger at all. If it does trigger, the ability will check again as it tries to resolve. If creatures you control don’t have total toughness 10 or more at that time, the ability won’t resolve and none of its effects will happen. The ability won’t check to see if their total toughness is 20 or greater (or 40 or greater) until the ability resolves."
    );
    supported("Betor, Kin to All");
    // 7 toughness: no trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Betor, Kin to All");
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 0);
    // 10 toughness, then below 10 before it resolves: nothing happens.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Betor, Kin to All");
    let giant = t.battlefield(P0, "Hill Giant");
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    kill(&mut t, giant);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
    // 10 toughness on trigger, 20+ on resolution: draws and untaps.
    let mut t = TestGame::new(2);
    let b = t.battlefield(P0, "Betor, Kin to All");
    t.battlefield(P0, "Hill Giant");
    tap(&mut t, b);
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.battlefield(P0, "Tree of Redemption");
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert!(!tapped(&t, b));
}

#[test]
fn untapping_an_attacker_keeps_it_attacking() {
    cr!("506.4", "508.1f");
    ruling!(
        "Lattice-Blade Mantis",
        "Untapping Lattice-Blade Mantis as its last ability resolves doesn't remove it from combat."
    );
    supported("Lattice-Blade Mantis");
    let mut t = TestGame::new(2);
    let m = t.enter(P0, "Lattice-Blade Mantis");
    t.g.objects[m.0 as usize].summoning_sick = false;
    assert_eq!(t.counters(m, "oil"), 2);
    t.answer(P0, DecisionKind::Attackers, Answer::Attackers(vec![(m, pl(P1))]));
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::DeclareBlockers);
    t.resolve_all();
    assert!(!tapped(&t, m));
    assert!(attacking(&t, m));
    assert_eq!(t.counters(m, "oil"), 1);
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 15);
}

#[test]
fn mage_ring_responder_kills_a_blocker_before_blocks() {
    cr!("508.2", "509.1");
    ruling!(
        "Mage-Ring Responder",
        "The last ability triggers and resolves before blockers are declared. If the target creature is destroyed by the damage, it won’t be on the battlefield to block."
    );
    supported("Mage-Ring Responder");
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Mage-Ring Responder");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![(bears, m)]));
    t.attack(&[(m, pl(P1))], &[]);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(t.life(P1), 13);
}

#[test]
fn thousand_moons_infantry_untaps_in_every_untap_step() {
    cr!("502.3", "502.2");
    ruling!(
        "Thousand Moons Infantry",
        "Thousand Moons Infantry still untaps during your untap step as normal."
    );
    ruling!(
        "Thousand Moons Infantry",
        "Thousand Moons Infantry untaps at the same time as the active player's permanents. You can't choose not to untap it at that time."
    );
    ruling!(
        "Thousand Moons Infantry",
        "If an effect states that Thousand Moons Infantry doesn't untap during your untap step, that effect won't apply during another player's untap step."
    );
    supported("Thousand Moons Infantry");
    supported("Frost Breath");
    let mut t = TestGame::new(2);
    let i = t.battlefield(P0, "Thousand Moons Infantry");
    tap(&mut t, i);
    let asked = t.asked().len();
    t.advance_to(P1, Step::Upkeep);
    assert!(!tapped(&t, i));
    assert!(!t.asked()[asked..]
        .iter()
        .any(|(p, d)| *p == P0 && matches!(d, Decision::YesNo { .. })));
    tap(&mut t, i);
    t.advance_to(P0, Step::Upkeep);
    assert!(!tapped(&t, i));
    // Frost Breath: it doesn't untap during its controller's next untap step, but it
    // still untaps during the other player's.
    let mut t = TestGame::new(2);
    let i = t.battlefield(P0, "Thousand Moons Infantry");
    cast_at(&mut t, P0, "Frost Breath", &[obj(i)]);
    t.resolve_all();
    assert!(tapped(&t, i));
    t.advance_to(P1, Step::Upkeep);
    assert!(!tapped(&t, i));
}

#[test]
fn self_untapping_activated_abilities() {
    cr!("602.2", "601.2h", "605.1a", "106.12");
    ruling!(
        "Basalt Monolith",
        "Basalt Monolith's last ability can untap it as often as you can pay for it. If you believe you've found a way to generate an unbounded amount of mana with it, you're probably right."
    );
    ruling!(
        "Chained Brute",
        "You can activate Chained Brute’s last ability even if it’s already untapped."
    );
    ruling!("Seeker of Skybreak", "It can use its tap ability to untap itself.");
    ruling!(
        "Pili-Pala",
        "If Pili-Pala is already untapped, you can’t activate its {Q} ability. That’s because you can’t pay the “Untap this permanent” cost."
    );
    ruling!(
        "Pili-Pala",
        "Pili-Pala’s ability is a mana ability. It doesn’t use the stack and can’t be responded to."
    );
    ruling!(
        "Pili-Pala",
        "Using this creature’s ability means you’re untapping it for mana, not tapping it for mana. Mana Reflection won’t cause it to produce extra mana."
    );
    // Basalt Monolith: tap for {C}{C}{C}, untap for {3}, again.
    let mut t = TestGame::new(2);
    let b = t.battlefield(P0, "Basalt Monolith");
    for _ in 0..2 {
        t.activate(P0, b, 0, &[]).unwrap();
        assert!(tapped(&t, b));
        t.activate(P0, b, 1, &[]).unwrap();
        t.resolve();
        assert!(!tapped(&t, b));
    }
    // Chained Brute: activatable while untapped.
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "Chained Brute");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_choose(P0, &[obj(bears)]);
    lots(&mut t, P0);
    assert!(t.activate(P0, c, 0, &[]).is_ok());
    // Seeker of Skybreak untaps itself.
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Seeker of Skybreak");
    t.activate(P0, s, 0, &[obj(s)]).unwrap();
    assert!(tapped(&t, s));
    t.resolve();
    assert!(!tapped(&t, s));
    // Pili-Pala: not while untapped; a mana ability once tapped; Mana Reflection doesn't
    // double it.
    for reflection in [false, true] {
        let mut t = TestGame::new(2);
        let p = t.battlefield(P0, "Pili-Pala");
        if reflection {
            t.battlefield(P0, "Mana Reflection");
        }
        mana(&mut t, P0, ManaType::C, 2);
        assert!(t.activate(P0, p, 0, &[]).is_err());
        tap(&mut t, p);
        t.activate(P0, p, 0, &[]).unwrap();
        assert_eq!(t.stack_len(), 0);
        assert!(!tapped(&t, p));
        assert_eq!(t.g.player(P0).mana_pool.total(), 1, "reflection={reflection}");
    }
}

#[test]
fn devoted_druid_counter_is_a_cost() {
    cr!("602.2b", "118.3", "704.5f");
    ruling!(
        "Devoted Druid",
        "If you can't put -1/-1 counters on Devoted Druid (due to an effect such as that of Solemnity), you can't activate its second ability. If you can put counters on it, but that is modified by an effect (such as that of Vizier of Remedies), you can activate the ability even if paying the cost causes no counters to be put on Devoted Druid."
    );
    ruling!(
        "Devoted Druid",
        "You put the -1/-1 counter on Devoted Druid as a cost to activate its ability, not when it resolves. If paying the cost causes Devoted Druid to have 0 toughness, it's put into your graveyard before you can untap it and before you can even pay the cost again."
    );
    supported("Solemnity");
    supported("Vizier of Remedies");
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Devoted Druid");
    t.battlefield(P0, "Solemnity");
    tap(&mut t, d);
    assert!(t.activate(P0, d, 1, &[]).is_err());
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Devoted Druid");
    t.battlefield(P0, "Vizier of Remedies");
    tap(&mut t, d);
    t.activate(P0, d, 1, &[]).unwrap();
    assert_eq!(t.counters(d, "-1/-1"), 0);
    t.resolve();
    assert!(!tapped(&t, d));
    // The counter is put as the cost: the second activation kills it before the
    // ability resolves.
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Devoted Druid");
    tap(&mut t, d);
    t.activate(P0, d, 1, &[]).unwrap();
    assert_eq!(t.counters(d, "-1/-1"), 1);
    assert_eq!(t.stack_len(), 1);
    t.activate(P0, d, 1, &[]).unwrap();
    t.settle();
    assert_eq!(t.zone(d), Zone::Graveyard(P0));
    assert_eq!(t.stack_len(), 2);
}

/// P0's Torchling and Grizzly Bears, P1's Hill Giant.
fn torchling(t: &mut TestGame) -> (ObjectId, ObjectId, ObjectId) {
    let tl = t.battlefield(P0, "Torchling");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    (tl, bears, giant)
}

#[test]
fn torchling_redirects_spells_that_target_only_it() {
    cr!("115.7a", "115.10a", "608.2b");
    ruling!(
        "Torchling",
        "If a spell has multiple targets, but it's targeting Torchling with all of them (such as Seeds of Strength targeting Torchling three times), you can target that spell with Torchling's third ability. In that case, you change all of those targets to the same new target, as long as it meets each of the spell's targeting conditions."
    );
    ruling!(
        "Torchling",
        "If a spell targets multiple things, you can't target it with Torchling's third ability, even if all but one of those targets has become illegal."
    );
    ruling!(
        "Torchling",
        "When the third ability resolves, you must change the target if able. If you can't choose a legal target for the spell when this ability resolves, that spell's target doesn't change."
    );
    supported("Torchling");
    supported("Seeds of Strength");
    // Seeds of Strength targeting Torchling three times: all three change to the Bears.
    let mut t = TestGame::new(2);
    let (tl, bears, _) = torchling(&mut t);
    let s = cast_at(&mut t, P0, "Seeds of Strength", &[obj(tl), obj(tl), obj(tl)]);
    t.answer_targets(P0, &[obj(bears)]);
    act_only(&mut t, P0, tl, 2, &[obj(s)]);
    t.resolve();
    assert_eq!(stack_targets(&t, s), vec![obj(bears); 3]);
    t.resolve_all();
    assert_eq!(t.pt(bears), (5, 5));
    // Targeting Torchling and the Bears: can't be targeted, even after the Bears die.
    let mut t = TestGame::new(2);
    let (tl, bears, _) = torchling(&mut t);
    let s = cast_at(&mut t, P0, "Seeds of Strength", &[obj(tl), obj(bears), obj(tl)]);
    kill(&mut t, bears);
    lots(&mut t, P0);
    assert!(t.activate(P0, tl, 2, &[obj(s)]).is_err());
    // Shock at Torchling, no other legal target... a Shock can always target a player:
    // use Giant Growth with only Torchling as a creature: the target doesn't change.
    let mut t = TestGame::new(2);
    let tl = t.battlefield(P0, "Torchling");
    let s = cast_at(&mut t, P0, "Giant Growth", &[obj(tl)]);
    act_only(&mut t, P0, tl, 2, &[obj(s)]);
    t.resolve();
    assert_eq!(stack_targets(&t, s), vec![obj(tl)]);
    // With another creature, the target must change (to the only other choice).
    let mut t = TestGame::new(2);
    let (tl, _, giant) = torchling(&mut t);
    let bears = t.named_on_battlefield("Grizzly Bears")[0];
    kill(&mut t, bears);
    let s = cast_at(&mut t, P0, "Giant Growth", &[obj(tl)]);
    act_only(&mut t, P0, tl, 2, &[obj(s)]);
    t.resolve();
    assert_eq!(stack_targets(&t, s), vec![obj(giant)]);
}

#[test]
fn tap_or_untap_is_chosen_on_resolution() {
    cr!("608.2d", "701.26a");
    ruling!(
        "Elder Druid",
        "The decision of whether to tap or untap is made on resolution."
    );
    ruling!(
        "Puppeteer",
        "You decide on resolution whether or not to tap or untap the creature."
    );
    for name in ["Elder Druid", "Puppeteer"] {
        supported(name);
        for untap in [false, true] {
            let mut t = TestGame::new(2);
            let d = t.battlefield(P0, name);
            let bears = t.battlefield(P1, "Grizzly Bears");
            if untap {
                tap(&mut t, bears);
            }
            act_only(&mut t, P0, d, 0, &[obj(bears)]);
            let asked = t.asked().len();
            assert!(!t
                .asked()
                .iter()
                .any(|(_, d)| matches!(d, Decision::ChooseOption { .. })));
            t.answer_yes(P0, true);
            t.answer(P0, DecisionKind::Option, Answer::Index(untap as usize));
            t.resolve();
            assert!(t.asked()[asked..]
                .iter()
                .any(|(_, d)| matches!(d, Decision::ChooseOption { .. })));
            assert_eq!(tapped(&t, bears), !untap, "{name} untap={untap}");
        }
    }
}
