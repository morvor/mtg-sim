//! Item `rulings-residue-misc-1`, pass B — casting rulings (batches P066, P085): spells
//! cast while an ability resolves (CR 608.2g), cards cast from exile and the permissions
//! that end, copies cast without paying their mana costs (CR 118.9, 707.12), cost
//! increases (CR 601.2f) and "can't cast ... during that player's next turn" effects.

use crate::r_p053_common::concede;
use crate::r_residue_misc_1_common::*;
use mtg_engine::ability::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

const NORMAL: CastMethod = CastMethod::Normal;

/// Whether `p` may cast `card` now, by any method.
fn castable(t: &mut TestGame, p: PlayerId, card: ObjectId) -> bool {
    t.g.turn.priority = Some(p);
    t.g.recompute();
    let card = t.g.current(card);
    t.g.legal_actions(p)
        .iter()
        .any(|a| matches!(a, Action::Cast { card: c, .. } if *c == card))
}

/// The priority decisions anyone was given since decision `from`.
fn priorities_since(t: &TestGame, from: usize) -> usize {
    t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::Priority { .. }))
        .count()
}

/// Moves every card of `p`'s library to exile, so that the cards put on top afterwards are
/// the whole library.
fn empty_library(t: &mut TestGame, p: PlayerId) {
    for c in t.g.players[p.idx()].library.clone() {
        exile_card(t, c);
    }
}

#[test]
fn emergent_ultimatum_finds_only_monocolored_cards() {
    cr!("105.4", "105.2c", "701.23a");
    ruling!(
        "Emergent Ultimatum",
        "A monocolored card is exactly one of white, blue, black, red, or green and no other colors. A colorless card isn’t monocolored."
    );
    let mut t = TestGame::new(2);
    let bolt = t.library_top(P0, "Lightning Bolt");
    let orni = t.library_top(P0, "Ornithopter");
    let charm = t.library_top(P0, "Boros Charm");
    let from = t.asked().len();
    cast_resolve(&mut t, P0, "Emergent Ultimatum", &[]);
    let offered: Vec<Entity> = t.asked()[from..]
        .iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseEntities { candidates, .. } if *p == P0 => Some(candidates.clone()),
            _ => None,
        })
        .next()
        .unwrap();
    assert!(offered.contains(&obj(bolt)));
    assert!(!offered.contains(&obj(orni)));
    assert!(!offered.contains(&obj(charm)));
}

#[test]
fn uvildas_card_cast_or_left_in_exile() {
    cr!("608.2g", "601.2", "122.1");
    ruling!(
        "Uvilda, Dean of Perfection // Nassari, Dean of Expression",
        "After removing the last hone counter, casting the spell is optional. Timing permissions based on the card’s type are ignored. If an effect prohibits you from casting the card or you choose not to cast it, it remains exiled with no hone counters on it, and you won’t be able to cast it later."
    );
    // The ruling's "hone counter" is the refine counter of today's Oracle text (renamed
    // when hone counters got a meaning of their own, CR 122.1j; logged as outdated in
    // docs/RULING_DISCREPANCIES.md).
    for cast in [true, false] {
        let mut t = TestGame::new(2);
        let uvilda = t.battlefield(
            P0,
            "Uvilda, Dean of Perfection // Nassari, Dean of Expression",
        );
        let div = t.hand(P0, "Divination");
        t.answer_choose(P0, &[obj(div)]);
        t.answer_yes(P0, true);
        t.activate(P0, uvilda, 0, &[]).unwrap();
        t.resolve_all();
        let div = t.g.current(div);
        assert_eq!(t.zone(div), Zone::Exile);
        assert_eq!(t.counters(div, "refine"), 3);
        // Three upkeeps: the last counter comes off in the third, and P0 may cast the
        // sorcery then, in the upkeep, for {U} ({2}{U} less {4} of generic mana).
        t.lands(P0, "Island", 1);
        let mut hand = 0;
        for k in 0..3 {
            t.set_step(P1, Step::End);
            if k == 2 {
                t.answer_yes(P0, cast);
            }
            t.advance_to(P0, Step::Upkeep);
            hand = t.hand_size(P0);
            t.resolve_all();
        }
        if cast {
            assert_eq!(
                t.hand_size(P0),
                hand + 2,
                "Divination resolved in the upkeep"
            );
        } else {
            let div = t.g.current(div);
            assert_eq!(t.zone(div), Zone::Exile);
            assert_eq!(t.counters(div, "refine"), 0);
            t.set_step(P0, Step::PrecombatMain);
            fill_pool(&mut t, P0, 3);
            assert!(!castable(&mut t, P0, div));
        }
    }
}

#[test]
fn nassari_cards_not_cast_stay_exiled() {
    cr!("611.2a", "406.3");
    ruling!(
        "Uvilda, Dean of Perfection // Nassari, Dean of Expression",
        "Any cards exiled with Nassari’s ability that you don’t cast will remain exiled. You won’t be able to cast those cards on future turns"
    );
    let mut t = TestGame::new(2);
    let nassari = t.battlefield(
        P0,
        "Uvilda, Dean of Perfection // Nassari, Dean of Expression",
    );
    // As Nassari, the back face.
    mtg_engine::dfc::transform(&mut t.g, nassari);
    t.g.recompute();
    assert!(t.obj_now(nassari).chars.name.starts_with("Nassari"));
    let bolt = t.library_top(P1, "Lightning Bolt");
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    let bolt = t.g.current(bolt);
    assert_eq!(t.zone(bolt), Zone::Exile);
    t.set_step(P0, Step::PrecombatMain);
    fill_pool(&mut t, P0, 1);
    assert!(castable(&mut t, P0, bolt), "this turn");
    // P0's next turn: no longer.
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::PrecombatMain);
    t.resolve_all();
    fill_pool(&mut t, P0, 1);
    assert_eq!(t.zone(bolt), Zone::Exile);
    assert!(!castable(&mut t, P0, bolt));
}

#[test]
fn dance_with_calamity_cards_not_cast_stay_exiled() {
    cr!("608.2g", "611.2a");
    ruling!(
        "Dance with Calamity",
        "Any cards not cast, including land cards, remain in exile. They can't be cast on later turns."
    );
    let mut t = TestGame::new(2);
    empty_library(&mut t, P0);
    let forest = t.library_top(P0, "Forest");
    let bolt = t.library_top(P0, "Lightning Bolt");
    // Exile both (two "yes"), then cast none.
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_yes(P0, false);
    // "Cast this card?" for the Bolt: no.
    t.answer_yes(P0, false);
    t.answer_yes(P0, false);
    cast_resolve(&mut t, P0, "Dance with Calamity", &[]);
    let (forest, bolt) = (t.g.current(forest), t.g.current(bolt));
    assert_eq!(t.zone(forest), Zone::Exile);
    assert_eq!(t.zone(bolt), Zone::Exile);
    fill_pool(&mut t, P0, 1);
    assert!(!castable(&mut t, P0, bolt));
    assert!(t.play_land(P0, forest).is_err());
}

#[test]
fn unpredictable_cyclone_shares_a_card_type_not_a_supertype() {
    cr!("205.2a", "205.4a", "702.29a");
    ruling!(
        "Unpredictable Cyclone",
        "Artifact, creature, enchantment, instant, planeswalker, and sorcery are card types. Supertypes (such as legendary) and subtypes (such as Giant) are not."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Unpredictable Cyclone");
    // Cycling Street Wraith (a creature): a legendary land and a Giant sorcery-less pile
    // are passed over; the creature card is found.
    let bears = t.library_top(P0, "Grizzly Bears");
    let urborg = t.library_top(P0, "Urborg, Tomb of Yawgmoth");
    let wraith = t.hand(P0, "Street Wraith");
    t.answer_yes(P0, true);
    activate_containing(&mut t, P0, wraith, "Cycling").unwrap();
    t.resolve_all();
    assert!(t.on_battlefield(t.g.current(bears)));
    assert_eq!(t.zone(t.g.current(urborg)), Zone::Library(P0));
}

#[test]
fn summon_esper_valigarmanda_casts_as_the_chapter_resolves() {
    cr!("608.2g", "714.2b");
    ruling!(
        "Summon: Esper Valigarmanda",
        "You cast the spell while the ability is resolving and still on the stack. You can't wait to cast it later in the turn."
    );
    for cast in [true, false] {
        let mut t = TestGame::new(2);
        t.graveyard(P1, "Lightning Bolt");
        let summon = t.enter(P0, "Summon: Esper Valigarmanda");
        t.resolve_all();
        let bolt =
            t.g.exile
                .iter()
                .copied()
                .find(|c| t.obj_now(*c).chars.name == "Lightning Bolt");
        let bolt = bolt.expect("chapter I exiled the Bolt");
        t.set_step(P0, Step::Draw);
        if cast {
            t.answer_choose(P0, &[obj(bolt)]);
            t.answer_yes(P0, true);
            t.answer_targets(P0, &[P1.into()]);
        } else {
            t.answer_choose(P0, &[]);
            t.answer_yes(P0, false);
        }
        t.advance_to(P0, Step::PrecombatMain);
        t.resolve_all();
        assert_eq!(t.counters(summon, counters::LORE), 2);
        if cast {
            assert_eq!(t.life(P1), 17);
        } else {
            assert_eq!(t.life(P1), 20);
            fill_pool(&mut t, P0, 1);
            assert!(!castable(&mut t, P0, bolt), "too late");
        }
    }
}

#[test]
fn wildfire_devils_no_actions_between_exiling_and_casting() {
    cr!("608.2g", "707.12", "706.2");
    ruling!(
        "Wildfire Devils",
        "You choose a player at random and they exile a card all while the triggered ability is resolving. No player may take any actions between the two or between exiling the card and you copying and casting it."
    );
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Lightning Bolt");
    t.graveyard(P1, "Lightning Bolt");
    t.enter(P0, "Wildfire Devils");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[P1.into()]);
    t.resolve();
    assert_eq!(priorities_since(&t, from), 0);
    assert_eq!(t.stack_len(), 1, "the copy, cast");
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn tinybones_casts_during_resolution_ignoring_timing() {
    cr!("608.2g", "307.1");
    ruling!(
        "Tinybones, the Pickpocket",
        "You choose whether to cast the target nonland permanent card as Tinybones's triggered ability resolves. If you do, you do so as part of the resolution of that ability. You can't wait to cast it later in the turn. Timing restrictions based on the card's types are ignored."
    );
    for cast in [true, false] {
        let mut t = TestGame::new(2);
        let tiny = t.battlefield(P0, "Tinybones, the Pickpocket");
        let bears = t.graveyard(P1, "Grizzly Bears");
        t.lands(P0, "Swamp", 2);
        t.answer_targets(P0, &[obj(bears)]);
        t.answer_yes(P0, cast);
        t.attack(&[(tiny, P1.into())], &[]);
        let bears = t.g.current(bears);
        if cast {
            // Cast in the combat damage step, paid with {B}{B} (any type of mana).
            assert!(t.on_battlefield(bears));
            assert_eq!(controller(&mut t, bears), P0);
        } else {
            assert_eq!(t.zone(bears), Zone::Graveyard(P1));
            t.set_step(P0, Step::PostcombatMain);
            assert!(!castable(&mut t, P0, bears));
        }
    }
}

#[test]
fn spellweaver_helix_copies_pay_no_mana_cost_but_additional_costs() {
    cr!("118.9a", "107.3b", "707.12", "601.2b");
    ruling!(
        "Spellweaver Helix",
        "You don’t pay the spell’s mana cost. If a spell has X in its mana cost, X is 0. You do pay any additional costs for that spell. You can’t use any alternative costs."
    );
    /// P0's Spellweaver Helix with the two sorceries exiled from P0's graveyard.
    fn helix(t: &mut TestGame, a: &str, b: &str) {
        let a = t.graveyard(P0, a);
        let b = t.graveyard(P0, b);
        t.answer_targets(P0, &[obj(a), obj(b)]);
        t.enter(P0, "Spellweaver Helix");
        t.resolve_all();
        assert!(t.in_exile(&t.obj_now(a).chars.name.clone()));
    }
    // Blaze ({X}{R}: "deals X damage to any target"): X is 0.
    let mut t = TestGame::new(2);
    helix(&mut t, "Lava Axe", "Blaze");
    lands_for_cost(&mut t, P0, "Lava Axe");
    let axe = t.hand(P0, "Lava Axe");
    t.answer_targets(P0, &[P1.into()]);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[P1.into()]);
    t.cast(P0, axe).go();
    t.resolve_all();
    let cast: Vec<String> =
        t.g.history
            .spells_cast
            .iter()
            .map(|(_, s)| t.obj(*s).chars.name.to_string())
            .collect();
    assert_eq!(cast, vec!["Lava Axe".to_string(), "Blaze".to_string()]);
    assert_eq!(t.life(P1), 15, "Lava Axe's 5 and Blaze's 0");
    // Bone Splinters ("As an additional cost to cast this spell, sacrifice a creature.
    // Destroy target creature."): the copy's sacrifice is paid.
    let mut t = TestGame::new(2);
    helix(&mut t, "Lava Axe", "Bone Splinters");
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Hill Giant");
    lands_for_cost(&mut t, P0, "Lava Axe");
    let axe = t.hand(P0, "Lava Axe");
    t.answer_targets(P0, &[P1.into()]);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(theirs)]);
    t.answer_choose(P0, &[obj(mine)]);
    t.cast(P0, axe).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 15);
    assert!(!t.on_battlefield(mine));
    assert!(!t.on_battlefield(theirs));
}

#[test]
fn intrepid_paleontologist_dinosaurs_follow_the_normal_rules() {
    cr!("601.2", "307.1", "601.3");
    ruling!(
        "Intrepid Paleontologist",
        "You must still pay all costs and follow any timing restrictions and permissions for Dinosaur creature spells you cast from exile this way."
    );
    let mut t = TestGame::new(2);
    let paleo = t.battlefield(P0, "Intrepid Paleontologist");
    let dino = t.graveyard(P0, "Colossal Dreadmaw");
    fill_pool(&mut t, P0, 1);
    t.activate(P0, paleo, 1, &[obj(dino)]).unwrap();
    t.resolve_all();
    let dino = t.g.current(dino);
    assert_eq!(t.zone(dino), Zone::Exile);
    // No mana: no.
    tap_lands(&mut t, P0);
    t.g.players[0].mana_pool.mana.clear();
    assert!(!castable(&mut t, P0, dino));
    // Mana, but on the opponent's turn: no.
    t.set_step(P1, Step::PrecombatMain);
    fill_pool(&mut t, P0, 2);
    assert!(!castable(&mut t, P0, dino));
    // Its own main phase: yes.
    t.set_step(P0, Step::PrecombatMain);
    fill_pool(&mut t, P0, 2);
    assert!(castable(&mut t, P0, dino));
}

#[test]
fn soundwave_zero_is_even() {
    cr!("202.3a", "107.1");
    ruling!(
        "Soundwave, Sonic Spy // Soundwave, Superior Captain",
        "Zero is even."
    );
    let mut t = TestGame::new(2);
    let sw = t.battlefield(P0, "Soundwave, Sonic Spy // Soundwave, Superior Captain");
    mtg_engine::dfc::transform(&mut t.g, sw);
    t.g.recompute();
    assert_eq!(t.obj_now(sw).chars.name, "Soundwave, Superior Captain");
    t.answer_yes(P0, true);
    cast_resolve(&mut t, P0, "Ornithopter", &[]);
    assert_eq!(t.named_on_battlefield("Laserbeak").len(), 1);
    assert!(t.named_on_battlefield("Ravage").is_empty());
}

/// P0's Elspeth Conquers Death's second chapter has resolved.
fn ecd_chapter_two(t: &mut TestGame) -> ObjectId {
    let ecd = t.battlefield(P0, "Elspeth Conquers Death");
    t.g.objects[ecd.0 as usize]
        .counters
        .insert(counters::LORE.into(), 1);
    t.set_step(P0, Step::Draw);
    t.advance_to(P0, Step::PrecombatMain);
    t.resolve_all();
    assert_eq!(t.counters(ecd, counters::LORE), 2);
    ecd
}

/// What P1 pays for Shock now (with plenty of mana); `None`: can't cast it.
fn shock_cost(t: &mut TestGame, p: PlayerId) -> Option<usize> {
    let shock = t.hand(p, "Shock");
    let r = cost_to_cast(t, p, shock, &[P0.into()]);
    t.g.players[p.idx()].mana_pool.mana.clear();
    r
}

#[test]
fn elspeth_conquers_death_tax_lasts_until_your_next_turn() {
    cr!("611.2a", "601.2f", "610.3");
    ruling!(
        "Elspeth Conquers Death",
        "The effect of the second chapter lasts until your next turn even if this Saga leaves the battlefield before then."
    );
    let mut t = TestGame::new(2);
    let ecd = ecd_chapter_two(&mut t);
    destroy(&mut t, ecd);
    t.set_step(P1, Step::PrecombatMain);
    assert_eq!(shock_cost(&mut t, P1), Some(3));
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.g.turn.priority = Some(P1);
    assert_eq!(shock_cost(&mut t, P1), Some(1));
}

#[test]
fn elspeth_conquers_death_tax_after_its_controller_leaves() {
    cr!("800.4m");
    ruling!(
        "Elspeth Conquers Death",
        "In a multiplayer game, if you leave the game after this Saga's second chapter ability resolves but before your next turn begins, its effect lasts until your next turn would have begun. It neither expires immediately nor lasts indefinitely."
    );
    let mut t = TestGame::new(3);
    ecd_chapter_two(&mut t);
    t.set_step(P1, Step::PrecombatMain);
    concede(&mut t, P0);
    assert_eq!(shock_cost(&mut t, P1), Some(3), "it doesn't expire at once");
    t.set_step(P2, Step::PrecombatMain);
    assert_eq!(shock_cost(&mut t, P2), Some(3));
    // P2's turn ends; P0's would have begun: it's over.
    t.set_step(P2, Step::Cleanup);
    t.advance_to(P1, Step::PrecombatMain);
    assert_eq!(shock_cost(&mut t, P1), Some(1));
}

#[test]
fn elspeth_conquers_death_total_cost_order_and_mana_value() {
    cr!("601.2f", "202.3b");
    ruling!(
        "Elspeth Conquers Death",
        "To determine the total cost of a spell, start with the mana cost or alternative cost you're paying, add any cost increases (such as that of this Saga's second chapter), then apply any cost reductions. The mana value of the spell remains unchanged, no matter what the total cost to cast it was."
    );
    let mut t = TestGame::new(2);
    ecd_chapter_two(&mut t);
    t.set_step(P1, Step::PrecombatMain);
    // A reduction applied after the increase: Ruby Medallion's {1} off red spells.
    t.battlefield(P1, "Ruby Medallion");
    assert_eq!(shock_cost(&mut t, P1), Some(2));
    let shock = t.hand(P1, "Shock");
    fill_pool(&mut t, P1, 3);
    let s = t.cast(P1, shock).target(P0).go();
    assert_eq!(t.g.mana_value_of(s), 1);
}

#[test]
fn elspeth_conquers_death_chooses_the_counter_after_it_enters() {
    cr!("608.2c", "122.1");
    ruling!(
        "Elspeth Conquers Death",
        "You choose which kind of counter to put on the returned permanent after it has entered the battlefield. You may put a +1/+1 counter on a planeswalker or a loyalty counter on a creature"
    );
    let mut t = TestGame::new(2);
    let ecd = t.battlefield(P0, "Elspeth Conquers Death");
    t.g.objects[ecd.0 as usize]
        .counters
        .insert(counters::LORE.into(), 2);
    let bears = t.graveyard(P0, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.set_step(P0, Step::Draw);
    t.advance_to(P0, Step::PrecombatMain);
    t.resolve_all();
    let bears = t.g.current(bears);
    assert!(t.on_battlefield(bears));
    assert_eq!(t.counters(bears, counters::LOYALTY), 1);
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn two_cant_cast_next_turn_effects_apply_to_the_same_turn() {
    cr!("611.2a", "101.2");
    ruling!(
        "Sphinx's Decree",
        "If multiple effects say that an opponent can’t cast instant or sorcery spells during that player’s next turn, they all apply to the same turn."
    );
    ruling!(
        "Azor, the Lawbringer",
        "If multiple effects say that an opponent can't cast instant or sorcery spells during that player's next turn, they all apply to the same turn."
    );
    let mut t = TestGame::new(2);
    cast_resolve(&mut t, P0, "Sphinx's Decree", &[]);
    t.enter(P0, "Azor, the Lawbringer");
    t.resolve_all();
    t.set_step(P0, Step::End);
    t.advance_to(P1, Step::PrecombatMain);
    let bolt = t.hand(P1, "Lightning Bolt");
    fill_pool(&mut t, P1, 1);
    assert!(!castable(&mut t, P1, bolt), "P1's next turn");
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::PrecombatMain);
    t.set_step(P0, Step::End);
    t.advance_to(P1, Step::PrecombatMain);
    fill_pool(&mut t, P1, 1);
    assert!(castable(&mut t, P1, bolt), "the turn after");
    let _ = NORMAL;
}
