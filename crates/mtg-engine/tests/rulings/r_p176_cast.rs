//! Rulings batch P176 — casting other players' cards: during the resolution of the effect
//! that allows it (Silent-Blade Oni, Spectral Arcanist, Jace's Mindseeker, Mysterious
//! Stranger, Reversal of Fortune, Knowledge Exploitation, Quistis Trepe), without paying
//! the mana cost (CR 118.9), or later with a permission (Whispersteel Dagger, Lidless
//! Gaze).

use crate::r_p176_common::*;
use crate::r_s01_common::watch;
use crate::r_s02_common::can_cast;
use crate::r_s04_common::add_mana;
use crate::r_s22_common::{
    attack_p1_unblocked, check_free_cast_costs, choose_named_when_offered, FreeCaster,
};
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// CR 107.3b, 118.9: Blaze (`fc` places it) cast "without paying its mana cost" with X
/// announced as 5 at P1's Hill Giant: X is 0, so the Giant is undamaged.
fn check_x_is_zero(fc: &FreeCaster) {
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let blaze = (fc.place)(&mut t, "Blaze");
    t.lands(P0, "Mountain", 6);
    (fc.run)(&mut t, blaze, &|t| {
        t.answer(P0, DecisionKind::X, Answer::Number(5));
        t.answer_targets(P0, &[obj(giant)]);
    });
    assert!(t.on_battlefield(giant));
    assert_eq!(damage_on(&t, giant), 0, "X was 0");
    assert_eq!(crate::r_s01_common::tapped_lands(&t, P0), 0);
    assert!(
        t.g.history
            .spells_cast
            .iter()
            .any(|(p, s)| *p == P0 && t.g.obj(*s).chars.name == "Blaze"),
        "Blaze was cast"
    );
}

fn damage_on(t: &TestGame, id: ObjectId) -> u32 {
    t.obj_now(id).damage
}

// ---------------------------------------------------------------------------------
// Spectral Arcanist: "you may cast an instant or sorcery spell with mana value less than
// or equal to the number of Spirits you control from a graveyard without paying its mana
// cost."
// ---------------------------------------------------------------------------------

fn arcanist_place(t: &mut TestGame, name: &str) -> ObjectId {
    t.graveyard(P1, name)
}

fn arcanist_run(t: &mut TestGame, card: ObjectId, answers: &dyn Fn(&mut TestGame)) {
    // A second Spirit: P0 may cast a spell with mana value 2 or less.
    t.battlefield(P0, "Spectral Arcanist");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(card)]);
    answers(t);
    t.enter(P0, "Spectral Arcanist");
    t.resolve_all();
    t.clear_answers();
}

const ARCANIST: FreeCaster = FreeCaster {
    instants_only: false,
    place: arcanist_place,
    run: arcanist_run,
};

#[test]
fn spectral_arcanist_free_cast_costs_and_x() {
    cr!("118.9a", "118.9b", "107.3b");
    ruling!(
        "Spectral Arcanist",
        "If you cast a spell without paying its mana cost, you may choose to pay additional costs (such as kicker) but not alternative costs (such as flashback)."
    );
    ruling!(
        "Spectral Arcanist",
        "If you cast a spell with X in its mana cost this way, you must choose 0 as the value for X."
    );
    supported("Spectral Arcanist");
    check_free_cast_costs(&ARCANIST);
    check_x_is_zero(&ARCANIST);
}

#[test]
fn spectral_arcanist_cast_now_or_never() {
    cr!("608.2g", "601.2");
    ruling!(
        "Spectral Arcanist",
        "If you choose to cast the spell, you must do so as Spectral Arcanist's ability is resolving. You can't wait and cast it later."
    );
    let mut t = TestGame::new(2);
    let bolt = t.graveyard(P1, "Lightning Bolt");
    t.answer_yes(P0, false);
    t.answer_choose(P0, &[]);
    t.enter(P0, "Spectral Arcanist");
    t.resolve_all();
    assert_eq!(t.zone(bolt), Zone::Graveyard(P1));
    t.lands(P0, "Mountain", 1);
    t.set_step(P0, Step::PrecombatMain);
    assert!(!can_cast(&mut t, P0, bolt, CastMethod::Free));
    assert!(!can_cast(&mut t, P0, bolt, CastMethod::Normal));
    // Positive control: chosen as it resolves, it's cast.
    let mut t = TestGame::new(2);
    let bolt = t.graveyard(P1, "Lightning Bolt");
    arcanist_run(&mut t, bolt, &|t| {
        t.answer_targets(P0, &[Entity::Player(P1)]);
    });
    assert_eq!(t.life(P1), 17);
}

// ---------------------------------------------------------------------------------
// Mysterious Stranger: two graveyards' instants or sorceries are exiled; one is copied at
// random, and the copy may be cast without paying its mana cost.
// ---------------------------------------------------------------------------------

/// Puts `name` into both players' graveyards (the random choice copies the same card
/// either way); returns P1's.
fn stranger_place(t: &mut TestGame, name: &str) -> ObjectId {
    t.graveyard(P0, name);
    t.graveyard(P1, name)
}

fn stranger_run(t: &mut TestGame, card: ObjectId, answers: &dyn Fn(&mut TestGame)) {
    let mine = *t.g.player(P0).graveyard.last().unwrap();
    t.answer_targets(P0, &[obj(mine)]);
    t.answer_targets(P0, &[obj(card)]);
    t.answer_yes(P0, true);
    answers(t);
    t.enter(P0, "Mysterious Stranger");
    t.resolve_all();
    t.clear_answers();
}

#[test]
fn mysterious_stranger_free_cast_costs() {
    cr!("118.9a", "118.9b", "707.12");
    ruling!(
        "Mysterious Stranger",
        "If you cast a spell “without paying its mana cost,” you can’t choose to cast it for any alternative costs. You can, however, pay additional costs. If the spell has any mandatory additional costs, those must be paid to cast the card."
    );
    supported("Mysterious Stranger");
    check_free_cast_costs(&FreeCaster {
        instants_only: false,
        place: stranger_place,
        run: stranger_run,
    });
}

/// Records the names of the objects on the stack whenever P0 is asked to choose targets.
fn stack_at_targeting(t: &mut TestGame) -> crate::r_s01_common::Seen<Vec<String>> {
    watch(
        t,
        P0,
        |d| matches!(d, Decision::ChooseTargets { .. }),
        |g| {
            g.stack
                .iter()
                .map(|id| {
                    let o = g.obj(*id);
                    match &o.stack {
                        Some(si) => match &si.kind {
                            mtg_engine::object::StackKind::Triggered { ability, .. } => {
                                format!("trigger: {}", ability.text)
                            }
                            _ => o.chars.name.to_string(),
                        },
                        None => o.chars.name.to_string(),
                    }
                })
                .collect()
        },
    )
}

#[test]
fn mysterious_stranger_casts_the_copy_while_its_ability_is_on_the_stack() {
    cr!("608.2g", "707.12");
    ruling!(
        "Mysterious Stranger",
        "You cast the copy while Mysterious Stranger’s last ability is resolving and still on the stack. You can’t wait to cast it later in the turn."
    );
    let mut t = TestGame::new(2);
    let bolt = stranger_place(&mut t, "Lightning Bolt");
    let mine = *t.g.player(P0).graveyard.last().unwrap();
    t.answer_targets(P0, &[obj(mine)]);
    t.answer_targets(P0, &[obj(bolt)]);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    let seen = stack_at_targeting(&mut t);
    t.enter(P0, "Mysterious Stranger");
    t.resolve_all();
    assert_eq!(t.life(P1), 17, "the copy was cast and resolved");
    let seen = seen.lock().unwrap();
    let last = seen.last().expect("the copy's target was chosen");
    assert!(
        last.iter()
            .any(|s| s.contains("trigger") && s.contains("copy")),
        "the ability is still on the stack: {last:?}"
    );
}

// ---------------------------------------------------------------------------------
// Reversal of Fortune: "Target opponent reveals their hand. You may copy an instant or
// sorcery card in it. If you do, you may cast the copy without paying its mana cost."
// ---------------------------------------------------------------------------------

fn reversal_place(t: &mut TestGame, name: &str) -> ObjectId {
    t.hand(P1, name)
}

fn reversal_run(t: &mut TestGame, card: ObjectId, answers: &dyn Fn(&mut TestGame)) {
    add_mana(t, P0, ManaType::R, 2);
    add_mana(t, P0, ManaType::C, 4);
    let rev = t.hand(P0, "Reversal of Fortune");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(card)]);
    t.answer_yes(P0, true);
    answers(t);
    t.set_step(P0, Step::PrecombatMain);
    t.g.turn.priority = Some(P0);
    t.g.cast_spell(P0, rev, CastMethod::Normal)
        .expect("Reversal of Fortune");
    t.g.flush_events();
    t.resolve_all();
    t.clear_answers();
}

const REVERSAL: FreeCaster = FreeCaster {
    instants_only: false,
    place: reversal_place,
    run: reversal_run,
};

#[test]
fn reversal_of_fortune_free_cast_costs_and_x() {
    cr!("118.9a", "118.9b", "107.3b", "707.12");
    ruling!(
        "Reversal of Fortune",
        "You don’t pay the spell’s mana cost. If the spell has X in its mana cost, X is 0. You do pay any additional costs for that spell. You can’t use any alternative costs."
    );
    ruling!(
        "Reversal of Fortune",
        "Reversal of Fortune creates a copy of the card in the other player’s hand, then allows you to cast the copy from that player’s hand without paying its mana cost."
    );
    supported("Reversal of Fortune");
    check_x_is_zero(&REVERSAL);
    // Kicker may be paid: kicked, Burst Lightning deals 4 damage to Hill Giant, not 2.
    for kick in [true, false] {
        let mut t = TestGame::new(2);
        let giant = t.battlefield(P1, "Hill Giant");
        let bolt = reversal_place(&mut t, "Burst Lightning");
        t.lands(P0, "Wastes", 4);
        reversal_run(&mut t, bolt, &|t| {
            t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(kick));
            t.answer_targets(P0, &[obj(giant)]);
        });
        if kick {
            assert!(t.in_graveyard(P1, "Hill Giant"), "kicked");
            assert_eq!(crate::r_s01_common::tapped_lands(&t, P0), 4);
        } else {
            assert_eq!(damage_on(&t, giant), 2, "cast, not kicked");
            assert_eq!(crate::r_s01_common::tapped_lands(&t, P0), 0);
        }
    }
    // A mandatory additional cost must be paid: Tormenting Voice ("As an additional cost
    // to cast this spell, discard a card. Draw two cards.").
    for fodder in [true, false] {
        let mut t = TestGame::new(2);
        let voice = reversal_place(&mut t, "Tormenting Voice");
        if fodder {
            t.hand(P0, "Forest");
        }
        let lib = t.library_size(P0);
        reversal_run(&mut t, voice, &|_| {});
        if fodder {
            assert!(t.in_graveyard(P0, "Forest"), "discarded");
            assert_eq!(t.library_size(P0), lib - 2, "cast and resolved");
        } else {
            assert_eq!(t.library_size(P0), lib, "couldn't be cast");
        }
    }
    // No alternative cost: Cyclonic Rift isn't cast for its overload cost.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let rift = reversal_place(&mut t, "Cyclonic Rift");
    t.lands(P0, "Island", 7);
    reversal_run(&mut t, rift, &|t| {
        t.answer_targets(P0, &[obj(giant)]);
    });
    assert!(t.in_hand(P1, "Hill Giant"));
    assert!(t.on_battlefield(bears), "not overloaded");
    assert_eq!(crate::r_s01_common::tapped_lands(&t, P0), 0);
}

#[test]
fn reversal_of_fortune_casts_the_copy_while_resolving() {
    cr!("608.2g", "707.12");
    ruling!(
        "Reversal of Fortune",
        "You cast the copy while this ability is resolving and still on the stack. Normally, you’re not allowed to cast spells or activate abilities at this time. Reversal of Fortune provides an exception."
    );
    let mut t = TestGame::new(2);
    let bolt = reversal_place(&mut t, "Lightning Bolt");
    let seen = stack_at_targeting(&mut t);
    reversal_run(&mut t, bolt, &|t| {
        t.answer_targets(P0, &[Entity::Player(P1)]);
    });
    assert_eq!(t.life(P1), 17);
    assert!(
        t.in_hand(P1, "Lightning Bolt"),
        "the card stays in P1's hand"
    );
    let seen = seen.lock().unwrap();
    let last = seen.last().expect("the copy's target was chosen");
    assert!(
        last.iter().any(|s| s == "Reversal of Fortune"),
        "Reversal is still on the stack: {last:?}"
    );
}

#[test]
fn reversal_of_fortune_the_copy_needs_legal_targets() {
    cr!("601.2c", "707.12");
    ruling!(
        "Reversal of Fortune",
        "You can’t cast the copy unless all of its targets, if any, can be chosen."
    );
    // Murder ("Destroy target creature.") with no creatures: not cast.
    let mut t = TestGame::new(2);
    let murder = reversal_place(&mut t, "Murder");
    reversal_run(&mut t, murder, &|_| {});
    assert_eq!(t.stack_len(), 0);
    assert!(t.in_graveyard(P0, "Reversal of Fortune"));
    assert!(!t.in_graveyard(P1, "Murder"));
    // Positive control: with a creature, the copy is cast.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let murder = reversal_place(&mut t, "Murder");
    reversal_run(&mut t, murder, &|t| {
        t.answer_targets(P0, &[obj(bears)]);
    });
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn reversal_of_fortune_only_whether_you_can_cast_it_matters() {
    cr!("101.2", "601.2");
    ruling!(
        "Reversal of Fortune",
        "You can’t cast the copy if an effect prevents you from casting instants or sorceries, or from casting that particular instant or sorcery. It doesn’t matter whether your opponent can cast the card at this time; the only thing that matters is whether you can cast it."
    );
    supported("Silence");
    // P1 is under P0's Silence: P0 may still cast the copy.
    let mut t = TestGame::new(2);
    cast_resolve(&mut t, P0, "Silence", &[]);
    let bolt = reversal_place(&mut t, "Lightning Bolt");
    reversal_run(&mut t, bolt, &|t| {
        t.answer_targets(P0, &[Entity::Player(P1)]);
    });
    assert_eq!(t.life(P1), 17, "P0 cast the copy");
    // P1 responds to Reversal of Fortune with Silence: P0 can't cast the copy.
    let mut t = TestGame::new(2);
    reversal_place(&mut t, "Lightning Bolt");
    add_mana(&mut t, P0, ManaType::R, 2);
    add_mana(&mut t, P0, ManaType::C, 4);
    let rev = t.hand(P0, "Reversal of Fortune");
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.cast(P0, rev).target(Entity::Player(P1)).go();
    cast_only(&mut t, P1, "Silence", &[]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Reversal of Fortune"));
    assert_eq!(t.life(P1), 20, "P0 can't cast spells");
}

#[test]
fn reversal_of_fortune_the_copy_is_cast_from_the_opponents_hand() {
    cr!("601.2a", "101.2");
    ruling!(
        "Reversal of Fortune",
        "The spell is cast from your opponent’s hand, not yours, which can be important for some cards."
    );
    supported("Drannith Magistrate");
    // P1's Drannith Magistrate: "Your opponents can't cast spells from anywhere other than
    // their hands." The copy is in P1's hand, not P0's: P0 can't cast it.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Drannith Magistrate");
    let bolt = reversal_place(&mut t, "Lightning Bolt");
    reversal_run(&mut t, bolt, &|t| {
        t.answer_targets(P0, &[Entity::Player(P1)]);
    });
    assert_eq!(t.life(P1), 20, "not cast from P0's own hand");
    // Positive control: without the Magistrate, it's cast.
    let mut t = TestGame::new(2);
    let bolt = reversal_place(&mut t, "Lightning Bolt");
    reversal_run(&mut t, bolt, &|t| {
        t.answer_targets(P0, &[Entity::Player(P1)]);
    });
    assert_eq!(t.life(P1), 17);
}

// ---------------------------------------------------------------------------------
// Knowledge Exploitation: "Search target opponent's library for an instant or sorcery
// card. You may cast that card without paying its mana cost."
// ---------------------------------------------------------------------------------

fn exploitation_place(t: &mut TestGame, name: &str) -> ObjectId {
    t.library_top(P1, name)
}

fn exploitation_run(t: &mut TestGame, card: ObjectId, answers: &dyn Fn(&mut TestGame)) {
    add_mana(t, P0, ManaType::U, 2);
    add_mana(t, P0, ManaType::C, 5);
    let ke = t.hand(P0, "Knowledge Exploitation");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    let name = t.obj_now(card).chars.name.to_string();
    choose_named_when_offered(t, P0, &name);
    t.answer_yes(P0, true);
    answers(t);
    t.set_step(P0, Step::PrecombatMain);
    t.g.turn.priority = Some(P0);
    t.g.cast_spell(P0, ke, CastMethod::Normal)
        .expect("Knowledge Exploitation");
    t.g.flush_events();
    t.resolve_all();
    t.clear_answers();
}

const EXPLOITATION: FreeCaster = FreeCaster {
    instants_only: false,
    place: exploitation_place,
    run: exploitation_run,
};

#[test]
fn knowledge_exploitation_casts_from_the_opponents_library() {
    cr!("118.9a", "118.9b", "107.3b", "608.2g");
    ruling!(
        "Knowledge Exploitation",
        "You cast the instant or sorcery card as part of the resolution of this spell. It's cast from your opponent's library, not your hand."
    );
    supported("Knowledge Exploitation");
    check_free_cast_costs(&EXPLOITATION);
    check_x_is_zero(&EXPLOITATION);
    // Modes are chosen as normal: Funeral Charm's "Target player discards a card."
    let mut t = TestGame::new(2);
    let charm = exploitation_place(&mut t, "Funeral Charm");
    t.hand(P1, "Forest");
    exploitation_run(&mut t, charm, &|t| {
        t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0]));
        t.answer_targets(P0, &[Entity::Player(P1)]);
    });
    assert!(t.in_graveyard(P1, "Forest"), "the chosen mode happened");
    assert!(t.in_graveyard(P1, "Funeral Charm"));
    // It was cast from P1's library, not P0's hand.
    let (_, spell) = *t
        .g
        .history
        .spells_cast
        .iter()
        .find(|(p, s)| *p == P0 && t.g.obj(*s).chars.name == "Funeral Charm")
        .expect("Funeral Charm was cast");
    let from = t.g.obj(spell).stack.as_ref().and_then(|si| si.cast.from);
    assert_eq!(from, Some(mtg_engine::ability::ZoneKind::Library));
}

// ---------------------------------------------------------------------------------
// Jace's Mindseeker: "target opponent mills five cards. You may cast an instant or
// sorcery spell from among them without paying its mana cost."
// ---------------------------------------------------------------------------------

/// P1's library has Divination on top of four Forests; P1's graveyard already has a
/// Lightning Bolt. Mindseeker enters for P0 (on `turn`'s precombat main phase) and its
/// ability resolves with P0 choosing to cast the top card if `cast`, or declining.
fn mindseeker(turn: PlayerId, top: &str, cast: bool) -> (TestGame, usize) {
    supported("Jace's Mindseeker");
    let mut t = TestGame::new(2);
    t.graveyard(P1, "Lightning Bolt");
    for _ in 0..4 {
        t.library_top(P1, "Forest");
    }
    let top = t.library_top(P1, top);
    t.set_step(turn, Step::PrecombatMain);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    if cast {
        t.answer_choose_moved(P0, &[obj(top)]);
        t.answer_yes(P0, true);
    } else {
        t.answer_yes(P0, false);
        t.answer_choose(P0, &[]);
    }
    let from = t.asked().len();
    t.enter(P0, "Jace's Mindseeker");
    t.resolve_all();
    (t, from)
}

#[test]
fn jaces_mindseeker_from_among_the_milled_cards_only() {
    cr!("608.2g", "701.17a");
    ruling!(
        "Jace's Mindseeker",
        "“From among them” means the five cards put into the graveyard, not all cards in that graveyard."
    );
    let (t, from) = mindseeker(P0, "Divination", true);
    let offered: Vec<String> = entity_choices_since(&t, P0, from)
        .into_iter()
        .flatten()
        .filter_map(|e| e.object())
        .map(|o| t.g.obj(o).chars.name.to_string())
        .collect();
    assert!(offered.iter().any(|n| n == "Divination"), "{offered:?}");
    assert!(
        !offered.iter().any(|n| n == "Lightning Bolt"),
        "{offered:?}"
    );
    assert_eq!(t.hand_size(P0), 2, "Divination was cast");
}

#[test]
fn jaces_mindseeker_ignores_type_timing_but_not_other_restrictions_and_is_now_or_never() {
    cr!("608.2g", "307.1", "601.3");
    ruling!(
        "Jace's Mindseeker",
        "When casting an instant or sorcery card this way, ignore timing restrictions based on the card’s type. Other timing restrictions, such as “Cast [this spell] only during combat,” must be followed."
    );
    ruling!(
        "Jace's Mindseeker",
        "If you cast an instant or sorcery card this way, you do so while the ability of Jace’s Mindseeker is resolving. If you choose not to (or can’t), you won’t get a chance to cast one later."
    );
    // A sorcery during P1's turn.
    let (t, _) = mindseeker(P1, "Divination", true);
    assert_eq!(t.hand_size(P0), 2, "Divination was cast on P1's turn");
    // Chaotic Strike: "Cast this spell only during combat after blockers are declared."
    let (t, _) = mindseeker(P0, "Chaotic Strike", true);
    assert!(t.in_graveyard(P1, "Chaotic Strike"), "couldn't be cast");
    assert_eq!(t.hand_size(P0), 0);
    // Declined: it can't be cast later.
    let (mut t, _) = mindseeker(P0, "Divination", false);
    assert_eq!(t.hand_size(P0), 0);
    let div =
        t.g.player(P1)
            .graveyard
            .iter()
            .copied()
            .find(|c| t.g.obj(*c).chars.name == "Divination");
    let div = div.expect("milled");
    t.lands(P0, "Island", 3);
    assert!(!can_cast(&mut t, P0, div, CastMethod::Free));
    assert!(!can_cast(&mut t, P0, div, CastMethod::Normal));
}

// ---------------------------------------------------------------------------------
// Quistis Trepe: "you may cast target instant or sorcery card from a graveyard, and mana
// of any type can be spent to cast that spell."
// ---------------------------------------------------------------------------------

#[test]
fn quistis_trepe_cast_as_it_resolves_ignoring_type_timing() {
    cr!("608.2g", "307.1");
    ruling!(
        "Quistis Trepe",
        "You choose whether to cast the target instant or sorcery card as Quistis Trepe's triggered ability resolves. If you do, you do so as part of the resolution of that ability. You can't wait to cast it later in the turn. Timing restrictions based on the card's types are ignored."
    );
    supported("Quistis Trepe");
    // A sorcery on P1's turn.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let div = t.graveyard(P1, "Divination");
    t.lands(P0, "Island", 3);
    t.answer_targets(P0, &[obj(div)]);
    t.answer_yes(P0, true);
    t.enter(P0, "Quistis Trepe");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 2, "cast on P1's turn");
    // Declined: no later chance.
    let mut t = TestGame::new(2);
    let div = t.graveyard(P1, "Divination");
    t.lands(P0, "Island", 3);
    t.answer_targets(P0, &[obj(div)]);
    t.answer_yes(P0, false);
    t.enter(P0, "Quistis Trepe");
    t.resolve_all();
    t.set_step(P0, Step::PrecombatMain);
    assert!(!can_cast(&mut t, P0, div, CastMethod::Normal));
}

// ---------------------------------------------------------------------------------
// Silent-Blade Oni: "look at that player's hand. You may cast a spell from among those
// cards without paying its mana cost."
// ---------------------------------------------------------------------------------

/// P0's Silent-Blade Oni deals combat damage to P1, who holds `card`; P0 chooses to cast
/// it (with `answers` queued for casting it).
fn oni(card: &str, answers: &dyn Fn(&mut TestGame)) -> TestGame {
    supported("Silent-Blade Oni");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let oni = t.battlefield(P0, "Silent-Blade Oni");
    let c = t.hand(P1, card);
    t.answer_choose(P0, &[obj(c)]);
    t.answer_yes(P0, true);
    answers(&mut t);
    attack_p1_unblocked(&mut t, oni);
    t
}

#[test]
fn silent_blade_oni_permanents_are_yours_spells_go_to_their_owner() {
    cr!("608.3a", "608.2n", "404.1");
    ruling!(
        "Silent-Blade Oni",
        "If you cast a permanent spell this way, it will enter under your control when it resolves. If you cast an instant or sorcery spell this way, that card will be put into its owner's graveyard when it resolves."
    );
    let mut t = oni("Grizzly Bears", &|_| {});
    let bears = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(bears.len(), 1);
    assert_eq!(controller(&mut t, bears[0]), P0);
    let t = oni("Divination", &|_| {});
    assert_eq!(t.hand_size(P0), 2);
    assert!(t.in_graveyard(P1, "Divination"));
    assert!(!t.in_graveyard(P0, "Divination"));
}

#[test]
fn silent_blade_oni_ignores_type_timing_but_not_other_restrictions() {
    cr!("608.2g", "307.1", "601.3");
    ruling!(
        "Silent-Blade Oni",
        "The nonland card you cast via the triggered ability is cast as part of the resolution of that ability. Timing restrictions based on the card's type are ignored. Other restrictions, such as \"Cast [this card] only during an opponent's turn,\" are not."
    );
    // A sorcery in the combat damage step.
    let t = oni("Divination", &|_| {});
    assert_eq!(t.hand_size(P0), 2);
    // Festival: "Cast this spell only during an opponent's upkeep."
    let t = oni("Festival", &|_| {});
    assert!(t.in_hand(P1, "Festival"), "not cast");
}

// ---------------------------------------------------------------------------------
// Whispersteel Dagger: "you may cast a creature spell from that player's graveyard this
// turn"
// ---------------------------------------------------------------------------------

/// P0's Grizzly Bears equipped with Whispersteel Dagger deals combat damage to P1, whose
/// graveyard holds Hill Giant and Serra Angel. Returns (game, Giant, Angel, decisions
/// asked of P0 as the trigger resolved).
fn dagger() -> (TestGame, ObjectId, ObjectId, Vec<Decision>) {
    supported("Whispersteel Dagger");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let dagger = t.battlefield(P0, "Whispersteel Dagger");
    t.g.attach(dagger, obj(bears));
    let giant = t.graveyard(P1, "Hill Giant");
    let angel = t.graveyard(P1, "Serra Angel");
    let from = t.asked().len();
    attack_p1_unblocked(&mut t, bears);
    assert_eq!(t.life(P1), 16);
    let asked = t.asked()[from..]
        .iter()
        .filter(|(p, _)| *p == P0)
        .map(|(_, d)| d.clone())
        .collect();
    (t, giant, angel, asked)
}

#[test]
fn whispersteel_dagger_grants_a_permission_with_normal_timing() {
    cr!("601.3", "307.1", "608.2g");
    ruling!(
        "Whispersteel Dagger",
        "You don’t choose a card to cast as Whispersteel Dagger’s ability resolves. You simply get a permission to cast one later."
    );
    ruling!(
        "Whispersteel Dagger",
        "You must follow the normal timing permissions and restrictions for the spell you wish to cast."
    );
    let (mut t, giant, angel, asked) = dagger();
    assert!(
        !asked
            .iter()
            .any(|d| matches!(d, Decision::ChooseEntities { .. })),
        "no card chosen as it resolves"
    );
    t.lands(P0, "Swamp", 5);
    // Not during combat (creature spells: main phase, empty stack).
    assert!(!can_cast(&mut t, P0, giant, CastMethod::Normal), "combat");
    t.advance_to(P0, Step::PostcombatMain);
    assert!(can_cast(&mut t, P0, giant, CastMethod::Normal));
    assert!(can_cast(&mut t, P0, angel, CastMethod::Normal));
    t.cast(P0, angel).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Serra Angel").len(), 1);
}

// ---------------------------------------------------------------------------------
// Lidless Gaze: "you may play those cards, and mana of any type can be spent to cast
// those spells."
// ---------------------------------------------------------------------------------

/// P0 casts Lidless Gaze with P1's top card `top`; returns (game, that card).
fn gaze(top: &str) -> (TestGame, ObjectId) {
    supported("Lidless Gaze");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.library_top(P0, "Forest");
    let c = t.library_top(P1, top);
    cast_resolve(&mut t, P0, "Lidless Gaze", &[]);
    let c = t.g.current(c);
    assert_eq!(t.zone(c), Zone::Exile);
    (t, c)
}

#[test]
fn lidless_gaze_any_type_of_mana_including_colorless() {
    cr!("609.4b", "106.1");
    ruling!(
        "Lidless Gaze",
        "Lidless Gaze uses a new template indicating that you may spend mana as though it were mana of any type to cast the exiled cards. The six types of mana are white, blue, black, red, green, and colorless."
    );
    supported("Thought-Knot Seer");
    // Thought-Knot Seer ({3}{C}) paid with red mana only.
    let (mut t, seer) = gaze("Thought-Knot Seer");
    add_mana(&mut t, P0, ManaType::R, 4);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.cast(P0, seer).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Thought-Knot Seer").len(), 1);
    // Lightning Bolt ({R}) paid with colorless mana.
    let (mut t, bolt) = gaze("Lightning Bolt");
    add_mana(&mut t, P0, ManaType::C, 1);
    t.cast(P0, bolt).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn lidless_gaze_doesnt_pay_snow_costs() {
    cr!("107.4h", "106.1b", "609.4b");
    ruling!(
        "Lidless Gaze",
        "Snow mana is not a type of mana. Lidless Gaze won't let you pay a snow cost using mana produced by a nonsnow source."
    );
    supported("Icehide Golem");
    let (mut t, golem) = gaze("Icehide Golem");
    t.lands(P0, "Mountain", 1);
    assert!(t.cast(P0, golem).try_go().is_err(), "nonsnow mana");
    // Positive control: a snow source pays it.
    t.lands(P0, "Snow-Covered Mountain", 1);
    t.cast(P0, golem).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Icehide Golem").len(), 1);
}
