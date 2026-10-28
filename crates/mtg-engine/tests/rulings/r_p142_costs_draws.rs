//! Rulings batch P142 — cost reducers (the total cost adds increases before applying
//! reductions, only generic mana is reduced, the mana value doesn't change), restricted
//! mana, and draw replacements and draw-step engines (Abundance, Necropotence, Jace,
//! Balin).

use crate::r_p142_common::*;
use crate::r_p194_common::{assert_total_cost, casts_for};
use crate::r_s01_common::{custom_card, watch};
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const W: ManaType = ManaType::W;
const U: ManaType = ManaType::U;
const B: ManaType = ManaType::B;
const G: ManaType = ManaType::G;

#[test]
fn gorex_adds_cost_increases_before_its_reduction() {
    cr!("601.2f", "118.7");
    ruling!(
        "Gorex, the Tombshell",
        "Include any additional costs to cast Gorex before applying its own cost reduction. For example, if Gorex is your commander and an additional cost of {6} applies from the \"commander tax,\" you may cast Gorex by exiling four creature cards from your graveyard and paying {4}{B}{B}."
    );
    supported("Lodestone Golem");
    // With Lodestone Golem ({1} more), exiling three cards: {6} + {1} − {6} = {1}{B}{B}.
    for (generic, castable) in [(1, true), (0, false)] {
        let mut t = TestGame::new(2);
        t.battlefield(P1, "Lodestone Golem");
        let cards: Vec<ObjectId> = (0..3).map(|_| t.graveyard(P0, "Grizzly Bears")).collect();
        let g = t.hand(P0, "Gorex, the Tombshell");
        lots_of(&mut t, B, 2);
        lots_of(&mut t, ManaType::C, generic);
        t.answer(P0, DecisionKind::OptionalCost, Answer::Number(3));
        let es: Vec<Entity> = cards.iter().map(|c| obj(*c)).collect();
        t.answer_choose(P0, &es);
        let ok = t.cast(P0, g).try_go().is_ok() && t.g.player(P0).mana_pool.total() == 0;
        assert_eq!(ok, castable, "{generic}");
    }
}

fn lots_of(t: &mut TestGame, ty: ManaType, n: u32) {
    t.g.players[0].mana_pool.add_type(ty, n);
}

#[test]
fn cid_reduces_generic_mana_after_increases() {
    cr!("601.2f", "202.3");
    ruling!(
        "Cid, Freeflier Pilot",
        "The cost reduction applies only to generic mana in the total cost of Equipment and Vehicle spells you cast."
    );
    ruling!(
        "Cid, Freeflier Pilot",
        "To determine the total cost of a spell, start with the mana cost or alternative cost you're paying (such as a flashback cost), add any cost increases (such as kicker costs), then apply any cost reductions (such as that of Cid's first ability). The mana value of the spell is determined by only its mana cost, no matter what the total cost to cast that spell was."
    );
    supported("Cid, Freeflier Pilot");
    let cid = |t: &mut TestGame| {
        t.battlefield(P0, "Cid, Freeflier Pilot");
    };
    // Kor Halberd ({W}): no generic mana to reduce.
    assert_total_cost(&cid, P0, "Kor Halberd", &[(W, 1)], 0);
    assert!(!casts_for(&cid, P0, "Kor Halberd", &[], 0));
    // Bonesplitter ({1}) and Smuggler's Copter ({2}).
    assert_total_cost(&cid, P0, "Bonesplitter", &[], 0);
    assert_total_cost(&cid, P0, "Smuggler's Copter", &[], 1);
    // With Thalia's {1} more first: Bonesplitter costs {1}, Kor Halberd {W}.
    let both = |t: &mut TestGame| {
        t.battlefield(P0, "Cid, Freeflier Pilot");
        t.battlefield(P1, "Thalia, Guardian of Thraben");
    };
    assert_total_cost(&both, P0, "Bonesplitter", &[], 1);
    assert_total_cost(&both, P0, "Kor Halberd", &[(W, 1)], 0);
    // The mana value of Bonesplitter cast for {0} is still 1.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Cid, Freeflier Pilot");
    let b = t.hand(P0, "Bonesplitter");
    let spell = t.cast(P0, b).go();
    assert_eq!(t.g.mana_value_of(spell), 1);
}

#[test]
fn unesh_reduces_other_sphinxes_after_increases_but_not_itself() {
    cr!("601.2f", "202.3", "113.6");
    ruling!(
        "Unesh, Criosphinx Sovereign",
        "To determine the total cost of a Sphinx spell, start with the mana cost or alternative cost you're paying, add any cost increases, then apply any cost reductions. The mana value of the creature remains unchanged, no matter what the total cost to cast it was."
    );
    ruling!(
        "Unesh, Criosphinx Sovereign",
        "Unesh's cost-reduction ability doesn't reduce the cost to cast itself."
    );
    supported("Unesh, Criosphinx Sovereign");
    supported("Sphinx of Magosi");
    let unesh = |t: &mut TestGame| {
        t.battlefield(P0, "Unesh, Criosphinx Sovereign");
    };
    // Sphinx of Magosi ({3}{U}{U}{U}) costs {1}{U}{U}{U}; with Lodestone Golem {2}{U}{U}{U}.
    assert_total_cost(&unesh, P0, "Sphinx of Magosi", &[(U, 3)], 1);
    let both = |t: &mut TestGame| {
        t.battlefield(P0, "Unesh, Criosphinx Sovereign");
        t.battlefield(P1, "Lodestone Golem");
    };
    assert_total_cost(&both, P0, "Sphinx of Magosi", &[(U, 3)], 2);
    let mut t = TestGame::new(2);
    unesh(&mut t);
    let s = t.hand(P0, "Sphinx of Magosi");
    lots_of(&mut t, U, 3);
    lots_of(&mut t, ManaType::C, 1);
    let spell = t.cast(P0, s).go();
    assert_eq!(t.g.mana_value_of(spell), 6);
    // Unesh itself, from the hand: the full {4}{U}{U}.
    assert_total_cost(&|_: &mut TestGame| {}, P0, "Unesh, Criosphinx Sovereign", &[(U, 2)], 4);
}

/// A Myr with "Kicker {2}" or "Cycling {2}".
fn test_myr(text: &str) -> mtg_engine::card::CardDef {
    custom_card("Test Myr", "Artifact Creature — Myr", "{1}", Some((1, 1)), text)
}

#[test]
fn myr_reservoir_mana_pays_myr_spell_extras_but_not_myr_cards_elsewhere() {
    cr!("106.6", "601.2f", "602.2b");
    ruling!(
        "Myr Reservoir",
        "You can use the mana produced by Myr Reservoir’s first ability to pay an alternative cost or additional cost incurred while casting a Myr spell. It’s not limited to just that spell’s mana cost."
    );
    ruling!(
        "Myr Reservoir",
        "The mana can’t be spent to activate activated abilities of Myr sources that aren’t on the battlefield."
    );
    supported("Myr Reservoir");
    // Kicked: {1} + {2}, paid with the Reservoir's {C}{C} and a Wastes.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Myr Reservoir");
    let w = t.battlefield(P0, "Wastes");
    let m = t.custom(P0, test_myr("Kicker {2}"), Zone::Hand(P0));
    let spell = t.cast(P0, m).kicked(true).try_go();
    assert!(spell.is_ok(), "{spell:?}");
    assert!(t.obj_now(w).tapped);
    t.resolve_all();
    let paid = t.obj_now(m).cast.clone().expect("cast").times_kicked;
    assert_eq!(paid, 1);
    // Cycling a Myr card in the hand: the Reservoir's mana can't pay for it.
    for wastes in [0, 2] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Myr Reservoir");
        t.lands(P0, "Wastes", wastes);
        let m = t.custom(P0, test_myr("Cycling {2}"), Zone::Hand(P0));
        let hand = t.hand_size(P0);
        let r = t.activate(P0, m, 0, &[]);
        assert_eq!(r.is_ok(), wastes == 2, "{wastes}: {r:?}");
        if wastes == 2 {
            t.resolve_all();
            assert_eq!(t.hand_size(P0), hand, "cycled: discarded and drew");
            assert!(t.in_graveyard(P0, "Test Myr"));
        }
    }
}

/// A creature card with no creature type.
fn typeless() -> mtg_engine::card::CardDef {
    custom_card("Typeless", "Creature", "{1}{G}", Some((2, 2)), "")
}

#[test]
fn heralds_horn_reduces_only_generic_mana_and_does_nothing_without_a_type() {
    cr!("601.2f", "205.3m");
    ruling!(
        "Herald's Horn",
        "The effect of Herald's Horn reduces only generic mana in a spell's cost. If that cost has no generic mana, the cost isn't reduced."
    );
    ruling!(
        "Herald's Horn",
        "If you somehow control a Herald's Horn with no chosen creature type, no spells will cost less to cast, not even creature spells with no creature type. You'll be able to look at the top card of your library at the beginning of each of your upkeeps, but you can never put it into your hand this way, even if it's a creature card with no creature type."
    );
    supported("Herald's Horn");
    let elf = |t: &mut TestGame| {
        crate::r_p125_common::choose_creature_type(t, P0, "Elf");
        t.enter(P0, "Herald's Horn");
        t.clear_answers();
    };
    assert_total_cost(&elf, P0, "Llanowar Elves", &[(G, 1)], 0);
    assert!(!casts_for(&elf, P0, "Llanowar Elves", &[], 0));
    assert_total_cost(&elf, P0, "Elvish Visionary", &[(G, 1)], 0);
    // No chosen type (put onto the battlefield without the choice).
    let none = |t: &mut TestGame| {
        t.battlefield(P0, "Herald's Horn");
    };
    assert_total_cost(&none, P0, "Elvish Visionary", &[(G, 1)], 1);
    let mut t = TestGame::new(2);
    none(&mut t);
    let c = t.custom(P0, typeless(), Zone::Hand(P0));
    lots_of(&mut t, G, 1);
    assert!(t.cast(P0, c).try_go().is_err(), "no reduction for a typeless creature");
    // The upkeep look can't take a typeless creature card.
    let mut t = TestGame::new(2);
    none(&mut t);
    t.set_step(P1, Step::End);
    let top = t.custom(P0, typeless(), Zone::Library(P0));
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert!(!t.in_hand(P0, "Typeless"));
    assert_eq!(t.zone(top), Zone::Library(P0));
}

/// P0 controls Abundance; the answers to replace the next draw choosing `land`.
fn abundance_replaces(t: &mut TestGame, land: bool) {
    yes(t, P0);
    t.answer(P0, DecisionKind::Option, Answer::Index(if land { 0 } else { 1 }));
}

#[test]
fn abundance_replaces_each_draw_separately() {
    cr!("614.11", "121.2");
    ruling!(
        "Abundance",
        "If you use this on a multi-card draw, each replaced draw is handled separately. In other words, you reveal and then put on the bottom of the library for the first card, then do the same for the second, and so on. In a multi-card draw you do not have to choose how many of those draws will be replaced before you do any drawing or use of this card."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Abundance");
    stack_library(&mut t, P0, &["Shock", "Forest", "Island", "Opt"]);
    // First draw replaced (land): Shock to the bottom, Forest to the hand. Second draw
    // not replaced: Island.
    abundance_replaces(&mut t, true);
    t.answer_yes(P0, false);
    t.g.draw_cards(P0, 2);
    t.settle();
    let mut h = hand_names(&t, P0);
    h.sort();
    assert_eq!(h, vec!["Forest", "Island"]);
    assert_eq!(top_name(&t, P0), "Opt");
    assert_eq!(library_names(&t, P0).last().unwrap(), "Shock");
}

#[test]
fn abundance_saves_you_from_an_empty_library_and_doesnt_draw() {
    cr!("614.11", "704.5b", "121.3");
    ruling!(
        "Abundance",
        "If your library is empty, Abundance can prevent you from losing the game for being unable to draw a card. If an effect or turn-based action would cause you to draw a card, you can replace that draw with Abundance's replacement effect. (It doesn't matter that you'd be unable to actually draw a card.) Since Abundance's effect has you put a card into your hand instead of drawing a card, you'll never be forced to draw a card with an empty library."
    );
    ruling!(
        "Abundance",
        "This replacement effect replaces the draw, so nothing that triggers on a draw will trigger."
    );
    // An empty library.
    for replace in [true, false] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Abundance");
        t.g.players[0].library.clear();
        if replace {
            abundance_replaces(&mut t, true);
        } else {
            t.answer_yes(P0, false);
        }
        t.g.draw_cards(P0, 1);
        t.settle();
        assert_eq!(t.has_lost(P0), !replace, "{replace}");
    }
    // Psychosis Crawler doesn't trigger on a replaced draw.
    for replace in [true, false] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Abundance");
        t.battlefield(P0, "Psychosis Crawler");
        t.library_top(P0, "Forest");
        if replace {
            abundance_replaces(&mut t, true);
        } else {
            t.answer_yes(P0, false);
        }
        t.g.draw_cards(P0, 1);
        t.settle();
        t.resolve_all();
        assert!(t.in_hand(P0, "Forest"));
        assert_eq!(t.life(P1), if replace { 20 } else { 19 }, "{replace}");
    }
}

#[test]
fn necropotence_cards_come_back_even_without_it_and_stay_hidden() {
    cr!("603.7c", "406.3", "708.2");
    ruling!(
        "Necropotence",
        "Necropotence's last ability creates a delayed triggered ability that will put the exiled card into your hand. That ability still triggers even if Necropotence is removed from the battlefield before your end step."
    );
    ruling!(
        "Necropotence",
        "You can't look at the cards exiled by the last ability while they're face down."
    );
    supported("Necropotence");
    let mut t = TestGame::new(2);
    let n = t.battlefield(P0, "Necropotence");
    let c = t.library_top(P0, "Grizzly Bears");
    t.activate(P0, n, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P0), 19);
    let exiled = now(&t, c);
    assert_eq!(t.zone(exiled), Zone::Exile);
    assert!(t.obj_now(exiled).face_down);
    assert!(!mtg_engine::player_control::can_see(&t.g, P0, exiled));
    destroy(&mut t, n);
    end_step(&mut t, P0);
    assert!(t.in_hand(P0, "Grizzly Bears"));
}

#[test]
fn necropotence_and_madness() {
    cr!("702.35a", "603.3b");
    ruling!(
        "Necropotence",
        "If you discard a card with madness and wish to cast it, Necropotence's ability won't exile that card. If you don't wish to cast it, you choose whether it ends up exiled or in your graveyard."
    );
    supported("Fiery Temper");
    // Cast with madness: it resolves and stays in the graveyard.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Necropotence");
    let f = t.hand(P0, "Fiery Temper");
    lots_of(&mut t, ManaType::R, 1);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    discard(&mut t, P0, f);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert!(t.in_graveyard(P0, "Fiery Temper"));
    // Not cast: the order of the two triggers decides.
    let mut ends = Vec::new();
    for order in [vec![0, 1], vec![1, 0]] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Necropotence");
        let f = t.hand(P0, "Fiery Temper");
        t.answer(P0, DecisionKind::Order, Answer::Indices(order));
        t.answer_yes(P0, false);
        discard(&mut t, P0, f);
        assert_eq!(stacked_triggers(&t), 2);
        t.resolve_all();
        assert_eq!(t.life(P1), 20);
        ends.push(t.zone(f));
    }
    ends.sort_by_key(|z| format!("{z:?}"));
    assert_eq!(ends, vec![Zone::Exile, Zone::Graveyard(P0)]);
}

#[test]
fn jace_draws_and_puts_back_in_one_resolution() {
    cr!("608.2c", "117.1");
    ruling!(
        "Jace, the Mind Sculptor",
        "You draw three cards and put two cards back all while Jace's second ability is resolving. Nothing can happen between the two, and no player may choose to take actions."
    );
    supported("Jace, the Mind Sculptor");
    let mut t = TestGame::new(2);
    let j = t.battlefield(P0, "Jace, the Mind Sculptor");
    let cards = stack_library(&mut t, P0, &["Shock", "Opt", "Forest"]);
    let seen = watch(
        &mut t,
        P0,
        |d| matches!(d, Decision::ChooseEntities { .. }),
        |g| (g.player(PlayerId(0)).hand.len(), g.stack.len()),
    );
    t.answer_choose_moved(P0, &[obj(cards[0]), obj(cards[1])]);
    t.activate(P0, j, 1, &[]).unwrap();
    let from = n_asked(&t);
    // Resolve by passing priority: nobody gets priority during the resolution.
    let ok = t.g.run_until(100, |g| g.stack.is_empty());
    assert!(ok);
    let priority_during = t.asked()[from..]
        .iter()
        .take_while(|(_, d)| !matches!(d, Decision::ChooseEntities { .. }))
        .filter(|(_, d)| matches!(d, Decision::Priority { .. }))
        .count();
    // Only the passes before the ability began resolving.
    assert!(priority_during <= 2, "{priority_during}");
    assert_eq!(seen.lock().unwrap()[0], (3, 1), "drew three, still resolving");
    assert_eq!(hand_names(&t, P0), vec!["Forest"]);
    let top: Vec<String> = library_names(&t, P0).into_iter().take(2).collect();
    assert!(top.contains(&"Shock".to_string()) && top.contains(&"Opt".to_string()));
}

#[test]
fn balin_discard_decision_is_made_while_resolving() {
    cr!("608.2c", "603.3");
    ruling!(
        "Balin, Loremaster",
        "The decision to discard your hand is made during the resolution of the ability, so there is no opportunity for an opponent to respond between choosing to discard your hand and drawing cards."
    );
    supported("Balin, Loremaster");
    let mut t = TestGame::new(2);
    t.hand(P0, "Shock");
    t.hand(P0, "Opt");
    let seen = watch(
        &mut t,
        P0,
        |d| matches!(d, Decision::YesNo { .. }),
        |g| (g.player(PlayerId(0)).hand.len(), g.stack.len()),
    );
    yes(&mut t, P0);
    enter(&mut t, P0, "Balin, Loremaster");
    assert_eq!(stacked_triggers(&t), 1);
    assert!(seen.lock().unwrap().is_empty(), "not asked when it triggers");
    t.resolve_all();
    assert_eq!(seen.lock().unwrap()[0], (2, 1), "asked while the trigger resolves");
    assert_eq!(t.hand_size(P0), 2);
    assert!(t.in_graveyard(P0, "Shock") && t.in_graveyard(P0, "Opt"));
}
