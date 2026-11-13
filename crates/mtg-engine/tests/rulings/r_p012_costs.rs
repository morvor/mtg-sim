//! Rulings batch P012 — costs that depend on graveyards and other conditions: cost
//! reductions are fixed amounts applied after increases (CR 601.2f), reduce only generic
//! mana, don't change the mana value (CR 202.3), don't count the spell itself; "mana
//! cost" means the printed cost (CR 202.1); restricted mana (CR 106.6); mana abilities
//! (CR 605.3); flash conditions checked before costs (CR 601.3).

use crate::r_p012_common::*;
use crate::r_s01_common::{asked_since, supported};
use crate::r_s04_common::untapped_lands;
use crate::r_s08_common::{legal_cast_methods, mana_value};
use crate::r_s20_common::tap_for_mana;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Whether `p` can cast the card `card` now with the lands they control.
fn can_cast(t: &mut TestGame, p: PlayerId, card: ObjectId) -> bool {
    !legal_cast_methods(t, p, card).is_empty()
}

#[test]
fn frostpyre_arcanist_costs_only_1_less_with_several_giants_and_wizards() {
    cr!("601.2f");
    ruling!(
        "Frostpyre Arcanist",
        "If you control more than one Giant and/or Wizard, Frostpyre Arcanist still costs only {1} less to cast."
    );
    supported("Frostpyre Arcanist");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Omnispell Adept");
    t.battlefield(P0, "Hill Giant");
    t.battlefield(P0, "Magmatic Channeler");
    t.lands(P0, "Island", 3);
    let fa = t.hand(P0, "Frostpyre Arcanist");
    assert!(!can_cast(&mut t, P0, fa), "{{4}}{{U}} - {{1}} needs 4 mana");
    t.lands(P0, "Island", 1);
    t.cast(P0, fa).go();
    assert_eq!(untapped_lands(&t, P0), 0);
}

#[test]
fn no_player_acts_while_frostpyre_arcanist_is_being_cast() {
    cr!("601.2", "117.1");
    ruling!(
        "Frostpyre Arcanist",
        "Once you announce that you’re casting Frostpyre Arcanist, no player may take other actions until the spell’s been paid for. Notably, opponents can’t try to change the cost by removing your Giants and Wizards."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Omnispell Adept");
    t.lands(P0, "Island", 4);
    let fa = t.hand(P0, "Frostpyre Arcanist");
    let from = t.asked().len();
    t.cast(P0, fa).go();
    assert!(asked_since(&t, from)
        .iter()
        .all(|(p, d)| *p == P0 && !matches!(d, Decision::Priority { .. })));
    assert_eq!(t.zone(fa), Zone::Stack);
}

#[test]
fn locket_of_yesterdays_reduces_the_cost_not_the_mana_value() {
    cr!("601.2f", "202.3");
    ruling!(
        "Locket of Yesterdays",
        "Locket of Yesterdays reduces only the amount of mana you need to pay to cast a spell. The spell’s mana cost and mana value are not affected."
    );
    supported("Locket of Yesterdays");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Locket of Yesterdays");
    t.graveyard(P0, "Divination");
    t.lands(P0, "Island", 2);
    let div = t.hand(P0, "Divination");
    let div = t.cast(P0, div).go();
    assert_eq!(untapped_lands(&t, P0), 0);
    assert_eq!(mana_value(&t, div), 3);
}

#[test]
fn locket_of_yesterdays_doesnt_count_the_spell_being_cast() {
    cr!("601.2a", "601.2f");
    ruling!(
        "Locket of Yesterdays",
        "The spell being cast doesn’t reduce its own cost, even if you’re casting it from a graveyard. At the time the cost is reduced, the spell is on the stack."
    );
    // Think Twice's flashback {2}{U} from the graveyard: no reduction unless another Think
    // Twice is there.
    for (others, cost) in [(0usize, 3usize), (1, 2)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Locket of Yesterdays");
        t.lands(P0, "Island", 3);
        let tt = t.graveyard(P0, "Think Twice");
        fill_graveyard(&mut t, P0, "Think Twice", others);
        t.cast(P0, tt)
            .method(CastMethod::Keyword(KeywordKind::Flashback))
            .go();
        assert_eq!(3 - untapped_lands(&t, P0), cost, "{others}");
    }
}

#[test]
fn pteramander_cant_reduce_below_its_blue_mana() {
    cr!("601.2f", "602.2b");
    ruling!(
        "Pteramander",
        "Pteramander's activated ability can't reduce itself to less than {U} to activate."
    );
    supported("Pteramander");
    let mut t = TestGame::new(2);
    let pt = t.battlefield(P0, "Pteramander");
    fill_graveyard(&mut t, P0, "Opt", 10);
    t.lands(P0, "Island", 1);
    t.activate(P0, pt, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(pt, "+1/+1"), 4);
    // With no mana it can't be activated.
    let mut t = TestGame::new(2);
    let pt = t.battlefield(P0, "Pteramander");
    fill_graveyard(&mut t, P0, "Opt", 10);
    assert!(t.activate(P0, pt, 0, &[]).is_err());
}

#[test]
fn into_the_story_reduction_comes_after_increases_and_keeps_the_mana_value() {
    cr!("601.2f", "202.3");
    ruling!(
        "Into the Story",
        "To determine the total cost of a spell, start with the mana cost or alternative cost you're paying, add any cost increases, then apply any cost reductions (such as that of Into the Story). The mana value of the spell remains unchanged, no matter what the total cost to cast it was."
    );
    supported("Into the Story");
    // {5}{U}{U} + {1} (Sphere of Resistance) - {3} = 5 mana.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sphere of Resistance");
    fill_graveyard(&mut t, P1, "Grizzly Bears", 7);
    t.lands(P0, "Island", 4);
    let its = t.hand(P0, "Into the Story");
    assert!(!can_cast(&mut t, P0, its));
    t.lands(P0, "Island", 1);
    let its = t.cast(P0, its).go();
    assert_eq!(untapped_lands(&t, P0), 0);
    assert_eq!(mana_value(&t, its), 7);
}

#[test]
fn into_the_story_reduces_its_cost_by_only_3() {
    cr!("601.2f");
    ruling!(
        "Into the Story",
        "Into the Story's cost reduction can only reduce its cost by {3}. It doesn't reduce the cost further if more than one opponent has seven cards in their graveyard or if an opponent has fourteen cards in their graveyard."
    );
    for (p1, p2) in [(7usize, 7usize), (14, 0)] {
        let mut t = TestGame::new(3);
        fill_graveyard(&mut t, P1, "Grizzly Bears", p1);
        fill_graveyard(&mut t, P2, "Grizzly Bears", p2);
        t.lands(P0, "Island", 3);
        let its = t.hand(P0, "Into the Story");
        assert!(!can_cast(&mut t, P0, its), "{p1} {p2}");
        t.lands(P0, "Island", 1);
        assert!(can_cast(&mut t, P0, its), "{p1} {p2}");
    }
}

#[test]
fn ice_cave_asks_for_the_printed_mana_cost() {
    cr!("202.1", "118.12a");
    ruling!(
        "Ice Cave",
        "The mana cost does not include cost increases or reductions."
    );
    supported("Ice Cave");
    // P1's Divination costs {3}{U} with Sphere of Resistance; P0 counters it by paying its
    // mana cost {2}{U} with three Islands.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ice Cave");
    t.battlefield(P0, "Sphere of Resistance");
    t.lands(P0, "Island", 3);
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Island", 4);
    let div = t.hand(P1, "Divination");
    t.answer_yes(P0, true);
    let div = t.cast(P1, div).go();
    t.resolve_all();
    assert_eq!(t.zone(div), Zone::Graveyard(P1));
    assert_eq!(t.hand_size(P1), 0);
    assert_eq!(untapped_lands(&t, P0), 0);
}

#[test]
fn curious_homunculus_mana_pays_additional_costs_but_not_costs_during_resolution() {
    cr!("106.6", "601.2f", "702.120a", "118.12a");
    ruling!(
        "Curious Homunculus // Voracious Reader",
        "Mana from Curious Homunculus's first ability can be spent to cast a spell that has generic mana in its additional cost, such as Borrowed Malevolence with both modes chosen. It can't be spent to pay costs during a spell's resolution, such as the cost Convolute prompts a player to pay."
    );
    // Borrowed Malevolence ({B}, escalate {2}) with both modes: {2}{B}, paid with the
    // Homunculus's {C} and two Swamps.
    let mut t = TestGame::new(2);
    let ch = t.battlefield(P0, "Curious Homunculus // Voracious Reader");
    t.lands(P0, "Swamp", 2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    assert!(tap_for_mana(&mut t, P0, ch, "Add {C}"));
    let bm = t.hand(P0, "Borrowed Malevolence");
    t.cast(P0, bm)
        .modes(&[0, 1])
        .target(bears)
        .target(giant)
        .go();
    assert_eq!(t.g.player(P0).mana_pool.total(), 0, "the {{C}} was spent");
    assert_eq!(untapped_lands(&t, P0), 0);
    t.resolve_all();
    assert_eq!(t.pt(bears), (3, 3));
    assert_eq!(t.pt(giant), (2, 2));
    // P1's Convolute: P0 can't pay {4} with four Homunculus mana; Opt is countered.
    let mut t = TestGame::new(2);
    let hs: Vec<ObjectId> = (0..4)
        .map(|_| t.battlefield(P0, "Curious Homunculus // Voracious Reader"))
        .collect();
    let opt = cast_card(&mut t, P0, "Opt", &[]);
    for h in hs {
        assert!(tap_for_mana(&mut t, P0, h, "Add {C}"));
    }
    t.answer_yes(P0, true);
    cast_card(&mut t, P1, "Convolute", &[obj(opt)]);
    t.resolve_all();
    assert_eq!(t.zone(opt), Zone::Graveyard(P0));
    assert_eq!(t.hand_size(P0), 0);
}

#[test]
fn corrupted_grafstone_adds_a_color_of_a_graveyard_card() {
    cr!("605.3b", "106.1");
    ruling!(
        "Corrupted Grafstone",
        "Players can't respond to mana abilities, so no player can take any action to stop you from getting the color of mana you expect to produce once you announce the ability."
    );
    supported("Corrupted Grafstone");
    let mut t = TestGame::new(2);
    let cg = t.battlefield(P0, "Corrupted Grafstone");
    t.graveyard(P0, "Lightning Bolt");
    let stack = t.stack_len();
    assert!(tap_for_mana(&mut t, P0, cg, "Add"));
    assert_eq!(t.stack_len(), stack, "a mana ability doesn't use the stack");
    assert_eq!(
        t.g.player(P0)
            .mana_pool
            .count(mtg_engine::mana::ManaType::R),
        1
    );
}

#[test]
fn graveyard_shift_has_flash_with_five_mana_values() {
    cr!("702.8a", "601.3");
    ruling!(
        "Graveyard Shift",
        "“Five or more mana values among cards in your graveyard” means there are at least five mana values among those cards."
    );
    supported("Graveyard Shift");
    // Five cards with only four mana values: no flash. A fifth value: flash.
    let mut t = TestGame::new(2);
    for n in ["Ornithopter", "Memnite", "Llanowar Elves", "Grizzly Bears", "Divination"] {
        t.graveyard(P0, n);
    }
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P0, "Swamp", 5);
    let gs = t.hand(P0, "Graveyard Shift");
    assert!(!can_cast(&mut t, P0, gs));
    t.graveyard(P0, "Hill Giant");
    t.g.recompute();
    assert!(can_cast(&mut t, P0, gs));
}
