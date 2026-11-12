//! Rulings batch P101 — Pacts (a delayed trigger created only as the spell resolves, CR
//! 603.7a, 608.2b) and mana: Moxen (CR 106.1b, 106.5, 605.3), Chrome Mox's colors (105.4,
//! 106.1b), spirit guides during cost payment (601.2g–h), Elemental Resonance's trigger
//! (605.1b), Culling Ritual's colors, Mana Crypt's flip, and paying mana during Tariff's
//! resolution (608.2d, 605.3a).

use crate::r_p017_common::load_coins;
use crate::r_p101_common::*;
use mtg_engine::ability::AbilityKind;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::mana::ManaType;
use mtg_engine::mana_abilities::could_produce;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Whether P0 has lost after their next upkeep (with no mana to pay a Pact's cost).
fn loses_at_next_upkeep(t: &mut TestGame) -> bool {
    next_upkeep(t, P0);
    t.resolve_all();
    t.has_lost(P0)
}

#[test]
fn a_countered_or_fizzled_pact_makes_no_delayed_trigger() {
    cr!("603.7a", "608.2b", "701.6a");
    ruling!(
        "Pact of Negation",
        "If Pact of Negation is countered or otherwise doesn't resolve (perhaps because its target became illegal), the delayed triggered ability that threatens you with a game loss won't trigger at the beginning of your next upkeep."
    );
    ruling!(
        "Pact of the Titan",
        "If Pact of the Titan is countered, the delayed triggered ability that threatens you with a game loss won't trigger at the beginning of your next upkeep."
    );
    ruling!(
        "Slaughter Pact",
        "If Slaughter Pact is countered or otherwise doesn't resolve (perhaps because its target became illegal), the delayed triggered ability that threatens you with a game loss won't trigger at the beginning of your next upkeep."
    );
    ruling!(
        "Summoner's Pact",
        "If Summoner's Pact is countered, the delayed triggered ability that threatens you with a game loss won't trigger at the beginning of your next upkeep."
    );
    for name in [
        "Pact of Negation",
        "Pact of the Titan",
        "Slaughter Pact",
        "Summoner's Pact",
    ] {
        supported(name);
        for fizzle in [false, true] {
            if fizzle && matches!(name, "Pact of the Titan" | "Summoner's Pact") {
                continue; // no targets
            }
            let mut t = TestGame::new(2);
            let bears = t.battlefield(P1, "Grizzly Bears");
            let shock = cast_new(&mut t, P1, "Shock", &[pl(P1)]);
            let targets: Vec<Entity> = match name {
                "Pact of Negation" => vec![obj(shock)],
                "Slaughter Pact" => vec![obj(bears)],
                _ => vec![],
            };
            let pact = t.hand(P0, name);
            t.g.turn.priority = Some(P0);
            let spell = t.cast(P0, pact).targets(&targets).go();
            t.settle();
            if fizzle {
                // The target goes away first: the Shock is countered, or the Bears are
                // destroyed.
                if name == "Pact of Negation" {
                    counter(&mut t, P0, shock);
                } else {
                    destroy(&mut t, bears);
                }
            } else {
                counter(&mut t, P1, spell);
            }
            t.resolve_all();
            assert!(t.g.delayed_triggers.is_empty(), "{name} fizzle={fizzle}");
            assert!(!loses_at_next_upkeep(&mut t), "{name} fizzle={fizzle}");
        }
    }
}

#[test]
fn a_pact_that_resolves_but_does_nothing_still_makes_you_pay() {
    cr!("603.7a", "608.2c", "702.12b", "101.1");
    ruling!(
        "Pact of Negation",
        "If Pact of Negation resolves with a legal target but fails to counter that spell (most likely because that spell can't be countered), the delayed triggered ability will still trigger at the beginning of your next upkeep and you will have to pay. Or else."
    );
    ruling!(
        "Slaughter Pact",
        "If Slaughter Pact resolves with a legal target but the target is not destroyed (perhaps because it has indestructible or regenerated), the delayed triggered ability will still trigger at the beginning of your next upkeep and you will have to pay the cost or lose the game."
    );
    // Pact of Negation on Abrupt Decay (which can't be countered).
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let decay = cast_new(&mut t, P1, "Abrupt Decay", &[obj(bears)]);
    let pact = t.hand(P0, "Pact of Negation");
    t.g.turn.priority = Some(P0);
    t.cast(P0, pact).target(obj(decay)).go();
    t.resolve_all();
    assert!(!t.on_battlefield(bears), "the Decay resolved");
    assert!(loses_at_next_upkeep(&mut t));
    // Slaughter Pact on Darksteel Myr (indestructible).
    let mut t = TestGame::new(2);
    let myr = t.battlefield(P1, "Darksteel Myr");
    let pact = t.hand(P0, "Slaughter Pact");
    t.cast(P0, pact).target(obj(myr)).go();
    t.resolve_all();
    assert!(t.on_battlefield(myr));
    assert!(loses_at_next_upkeep(&mut t));
    // Paying {2}{B} instead.
    let mut t = TestGame::new(2);
    let myr = t.battlefield(P1, "Darksteel Myr");
    t.lands(P0, "Swamp", 3);
    let pact = t.hand(P0, "Slaughter Pact");
    t.cast(P0, pact).target(obj(myr)).go();
    t.resolve_all();
    t.answer_yes(P0, true);
    assert!(!loses_at_next_upkeep(&mut t));
}

/// P0 activates the mana ability of `src` (its first activated ability); whether the
/// activation was legal and the mana P0's pool gained.
fn activate_mana(t: &mut TestGame, src: ObjectId) -> (bool, usize) {
    t.g.turn.priority = Some(P0);
    t.g.recompute();
    let uid =
        t.g.obj(src)
            .chars
            .abilities
            .iter()
            .find(|a| matches!(&a.kind, AbilityKind::Activated(x) if x.is_mana_ability))
            .map(|a| a.uid)
            .expect("a mana ability");
    let before = t.g.player(P0).mana_pool.total();
    let ok = t.g.activate_ability(P0, src, uid).is_ok();
    t.g.flush_events();
    (ok, t.g.player(P0).mana_pool.total() - before)
}

#[test]
fn mox_amber_adds_one_mana_of_a_legends_color_or_nothing() {
    cr!("106.5", "106.1b", "605.3a", "105.2c");
    ruling!(
        "Mox Amber",
        "If you control no legendary creatures or legendary planeswalkers, you can activate Mox Amber's ability, but you won't add any mana."
    );
    ruling!(
        "Mox Amber",
        "If your legendary creatures and legendary planeswalkers are all colorless, you can activate Mox Amber's ability, but you won't add any mana. Colorless is not a color."
    );
    ruling!(
        "Mox Amber",
        "Mox Amber's ability adds one mana of the color of your choice from among the colors of legendary creatures and legendary planeswalkers you control. It doesn't add one mana of each of those colors."
    );
    // No legends; then only a colorless one (Karn, Silver Golem).
    for karn in [false, true] {
        let mut t = TestGame::new(2);
        let mox = t.battlefield(P0, "Mox Amber");
        t.battlefield(P0, "Grizzly Bears");
        if karn {
            t.battlefield(P0, "Karn, Silver Golem");
        }
        assert_eq!(activate_mana(&mut t, mox), (true, 0), "karn = {karn}");
        assert!(t.obj_now(mox).tapped);
    }
    // Adeliz, the Cinder Wind (blue and red): one mana, blue or red.
    let mut t = TestGame::new(2);
    let mox = t.battlefield(P0, "Mox Amber");
    t.battlefield(P0, "Adeliz, the Cinder Wind");
    let mut colors = could_produce(&t.g, mox);
    colors.sort();
    assert_eq!(colors, [ManaType::U, ManaType::R]);
    assert_eq!(activate_mana(&mut t, mox), (true, 1));
}

#[test]
fn chrome_mox_offers_each_color_of_the_imprinted_card_and_never_colorless() {
    cr!("106.1b", "105.4", "106.5", "607.2a");
    ruling!(
        "Chrome Mox",
        "If the exiled card is multicolored, you choose one of that card’s colors each time you tap this artifact for mana."
    );
    ruling!(
        "Chrome Mox",
        "This artifact's activated ability can never add {C}, even if the exiled card is colorless or has a colorless mana symbol in its mana cost."
    );
    for (card, colors) in [
        ("Lightning Helix", vec![ManaType::W, ManaType::R]),
        ("Thought-Knot Seer", vec![]),
    ] {
        let mut t = TestGame::new(2);
        let c = t.hand(P0, card);
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[obj(c)]);
        let mox = t.enter(P0, "Chrome Mox");
        t.resolve_all();
        assert!(t.in_exile(card), "{card}");
        let mut got = could_produce(&t.g, mox);
        got.sort();
        assert_eq!(got, colors, "{card}");
        if colors.is_empty() {
            assert_eq!(activate_mana(&mut t, mox).1, 0, "{card}");
            assert_eq!(t.g.player(P0).mana_pool.count(ManaType::C), 0);
        } else {
            // Each time it's tapped, either color: white now, then red.
            for ty in [ManaType::W, ManaType::R] {
                let i = colors.iter().position(|c| *c == ty).unwrap();
                t.answer(P0, DecisionKind::Option, Answer::Index(i));
                t.g.untap(mox);
                assert_eq!(activate_mana(&mut t, mox).1, 1);
            }
            assert_eq!(t.g.player(P0).mana_pool.count(ManaType::W), 1);
            assert_eq!(t.g.player(P0).mana_pool.count(ManaType::R), 1);
        }
    }
}

#[test]
fn a_second_mox_opal_meets_the_legend_rule_before_either_can_tap() {
    cr!("704.5j", "117.5", "602.5b");
    ruling!(
        "Mox Opal",
        "If you control Mox Opal and one other artifact, then have a second Mox Opal enter the battlefield as your third artifact, you can't activate either one's mana ability before putting one into your graveyard due to the “legend rule.”"
    );
    supported("Mox Opal");
    let mut t = TestGame::new(2);
    let first = t.battlefield(P0, "Mox Opal");
    t.battlefield(P0, "Ornithopter");
    let second = t.hand(P0, "Mox Opal");
    t.cast(P0, second).go();
    t.resolve_all();
    assert_eq!(controls_named(&t, P0, "Mox Opal"), 1);
    assert_eq!(t.graveyard_size(P0), 1);
    let left = if t.on_battlefield(first) {
        first
    } else {
        t.g.current(second)
    };
    assert_eq!(
        activate_mana(&mut t, left),
        (false, 0),
        "only two artifacts now"
    );
}

#[test]
fn a_spirit_guide_returned_to_hand_as_a_cost_cant_pay_for_that_cost() {
    cr!("601.2g", "601.2h", "602.2b", "702.49a");
    ruling!(
        "Elvish Spirit Guide",
        "If a spell or ability's cost includes returning a creature to your hand and paying mana, you can't activate Elvish Spirit Guide's mana ability after returning it to your hand. This is because mana abilities must be activated before you begin paying costs."
    );
    ruling!(
        "Simian Spirit Guide",
        "If a spell or ability's cost includes returning a creature to your hand and paying mana, you can't activate Simian Spirit Guide's mana ability after returning it to your hand. This is because mana abilities must be activated before you begin paying costs."
    );
    // The spirit guide attacks unblocked; ninjutsu of Ninja of the Deep Hours ({1}{U},
    // return an unblocked attacker you control to hand) with only an Island.
    for (guide, land) in [
        ("Elvish Spirit Guide", "Forest"),
        ("Simian Spirit Guide", "Mountain"),
    ] {
        for with_land in [false, true] {
            let mut t = TestGame::new(2);
            let g = t.battlefield(P0, guide);
            t.lands(P0, "Island", 1);
            if with_land {
                t.lands(P0, land, 1);
            }
            let ninja = t.hand(P0, "Ninja of the Deep Hours");
            crate::r_s03_common::to_blockers(&mut t, &[(g, pl(P1))], &[]);
            t.g.recompute();
            let index = t
                .obj(ninja)
                .chars
                .abilities
                .iter()
                .filter(|a| matches!(a.kind, AbilityKind::Activated(_)))
                .position(|a| a.text.starts_with("Ninjutsu"))
                .unwrap();
            t.answer_choose(P0, &[obj(g)]);
            let r = t.activate(P0, ninja, index, &[]);
            assert_eq!(r.is_ok(), with_land, "{guide} with {land}: {r:?}");
            if !with_land {
                assert!(t.on_battlefield(g), "{guide}: the activation was undone");
            }
        }
    }
}

#[test]
fn elemental_resonances_mana_comes_from_a_triggered_ability_on_the_stack() {
    cr!("605.1b", "603.3");
    ruling!(
        "Elemental Resonance",
        "This ability isn't a mana ability. It uses the stack."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    crate::r_s06_common::attach_new(&mut t, P0, "Elemental Resonance", obj(bears));
    t.set_step(P0, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    t.settle();
    assert_eq!(t.stack_len(), 1, "on the stack");
    assert_eq!(t.g.player(P0).mana_pool.total(), 0);
    // P1 may respond (with priority) before it resolves.
    t.resolve();
    assert_eq!(t.g.player(P0).mana_pool.count(ManaType::G), 1);
    assert_eq!(t.g.player(P0).mana_pool.total(), 2);
}

#[test]
fn culling_ritual_adds_black_or_green_for_each_permanent() {
    cr!("106.1a", "608.2c");
    ruling!(
        "Culling Ritual",
        "You can choose to add either {B} or {G} for each permanent destroyed. You aren't limited to one color."
    );
    let mut t = TestGame::new(2);
    for _ in 0..3 {
        t.battlefield(P1, "Grizzly Bears");
    }
    // Black, green, black.
    for i in [0, 1, 0] {
        t.answer(P0, DecisionKind::Option, Answer::Index(i));
    }
    cast_resolve(&mut t, P0, "Culling Ritual", "2BG", &[]);
    assert_eq!(t.g.player(P0).mana_pool.total(), 3, "{}", t.dump_log());
    assert!(t.g.player(P0).mana_pool.count(ManaType::B) >= 1);
    assert!(t.g.player(P0).mana_pool.count(ManaType::G) >= 1);
}

#[test]
fn mana_crypts_damage_follows_the_lost_flip_at_once() {
    cr!("705.2", "608.2c", "117.3b");
    ruling!(
        "Mana Crypt",
        "No player may choose to take actions between determining the result of the flip and damage being dealt if you lost the flip."
    );
    supported("Mana Crypt");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mana Crypt");
    // Whatever P0 calls, one of these two flips loses.
    load_coins(&mut t, &[false, true]);
    next_upkeep(&mut t, P0);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    let from = t.asked().len();
    t.g.resolve_top();
    let lost_first = t.life(P0) == 17;
    assert!(t.asked()[from..]
        .iter()
        .all(|(_, d)| !matches!(d, Decision::Priority { .. })));
    if !lost_first {
        next_upkeep(&mut t, P0);
        t.settle();
        t.g.resolve_top();
        assert_eq!(t.life(P0), 17);
    }
}

#[test]
fn tariff_is_paid_on_resolution_with_mana_abilities() {
    cr!("608.2d", "605.3a", "118.3", "101.4");
    ruling!(
        "Tariff",
        "You can use mana abilities during the resolution of this spell if the player decides to pay the mana cost."
    );
    ruling!(
        "Tariff",
        "You choose whether to pay or not on resolution. If not, then you sacrifice the creature. You can choose to not pay if you control no creatures on resolution."
    );
    supported("Tariff");
    // P1 taps two lands during the resolution to pay for Grizzly Bears ({1}{G}); P2
    // declines for Hill Giant and sacrifices it; P0 controls no creatures.
    let mut t = TestGame::new(3);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P2, "Hill Giant");
    t.lands(P1, "Forest", 2);
    t.lands(P2, "Mountain", 4);
    t.answer_yes(P1, true);
    t.answer_yes(P2, false);
    cast_resolve(&mut t, P0, "Tariff", "1W", &[]);
    assert!(t.on_battlefield(bears));
    assert_eq!(crate::r_s01_common::tapped_lands(&t, P1), 2);
    assert!(!t.on_battlefield(giant));
    assert_eq!(crate::r_s01_common::tapped_lands(&t, P2), 0);
}
