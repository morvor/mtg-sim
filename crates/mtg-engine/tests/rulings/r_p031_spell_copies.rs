//! Rulings batch P031 — spells that copy themselves: a copy is created on the stack, not
//! cast (CR 707.10), keeps the original's targets unless the effect lets its controller
//! choose new ones (CR 707.10c), and a spell or copy whose targets are all illegal doesn't
//! resolve (CR 608.2b); and spells that let a player play an exiled card.

use crate::r_p030_common::elementals;
use crate::r_p031_common::*;
use crate::r_s01_common::supported;
use crate::r_s02_common::can_cast;
use crate::r_s06_common::give_control;
use crate::r_s11_common::empty_library;
use crate::r_s25_common::{
    cast_new, change_copy_targets, keep_copy_targets, lands_for_cost, spell_copies, targets_of,
};
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Moves the card or permanent to exile (as a resolving effect would) and settles.
fn exile(t: &mut TestGame, id: ObjectId) {
    let id = t.g.current(id);
    t.g.exile_object(id, None);
    t.g.flush_events();
    t.settle();
}

// --- Sevinne's Reclamation -----------------------------------------------------------------

/// P0 (with Young Pyromancer) flashbacks Sevinne's Reclamation from the graveyard
/// targeting `target`. Returns the spell.
fn flashback_reclamation(t: &mut TestGame, target: ObjectId) -> ObjectId {
    supported("Sevinne's Reclamation");
    t.battlefield(P0, "Young Pyromancer");
    let r = t.graveyard(P0, "Sevinne's Reclamation");
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Wastes", 4);
    let s = t
        .cast(P0, r)
        .method(CastMethod::Keyword(KeywordKind::Flashback))
        .target(obj(target))
        .go();
    t.settle();
    assert_eq!(elementals(t, P0), 0);
    s
}

#[test]
fn sevinne_s_reclamation_copy_isn_t_cast_and_makes_no_copy() {
    cr!("707.10", "601.2i");
    ruling!(
        "Sevinne's Reclamation",
        "The copy that Sevinne's Reclamation creates is created on the stack, so it's not \"cast.\" Abilities that trigger when a player casts a spell won't trigger. The copy wasn't cast from a graveyard, so it won't make another copy of itself."
    );
    ruling!(
        "Sevinne's Reclamation",
        "If you cast Sevinne's Reclamation from your graveyard, any abilities that trigger as the permanent card returns to the battlefield will resolve before the copy of Sevinne's Reclamation resolves but after new targets for the copy have been chosen."
    );
    let mut t = TestGame::new(2);
    let visionary = t.graveyard(P0, "Elvish Visionary");
    let bears = t.graveyard(P0, "Grizzly Bears");
    flashback_reclamation(&mut t, visionary);
    // Young Pyromancer's trigger resolves first.
    t.resolve();
    assert_eq!(elementals(&t, P0), 1);
    // The Reclamation resolves: Elvish Visionary returns; the copy targets the Bears.
    t.answer_yes(P0, true);
    change_copy_targets(&mut t, P0, &[Some(obj(bears))]);
    t.resolve();
    assert_eq!(named(&t, P0, "Elvish Visionary").len(), 1);
    let copies = spell_copies(&t);
    assert_eq!(copies.len(), 1);
    assert_eq!(targets_of(&t, copies[0]), vec![obj(bears)]);
    // Visionary's enters trigger is on the stack above the copy.
    assert_eq!(t.stack_len(), 2);
    assert_eq!(*t.g.stack.first().unwrap(), copies[0]);
    let hand = t.hand_size(P0);
    t.resolve();
    assert_eq!(t.hand_size(P0), hand + 1, "Visionary's trigger resolved first");
    assert!(!t.on_battlefield(bears) && t.named_on_battlefield("Grizzly Bears").is_empty());
    // The copy resolves: no cast trigger, and no further copy.
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(named(&t, P0, "Grizzly Bears").len(), 1);
    assert_eq!(elementals(&t, P0), 1);
    assert!(spell_copies(&t).is_empty());
    assert_eq!(t.g.history.spells_cast.len(), 1);
}

#[test]
fn sevinne_s_reclamation_with_an_illegal_target_makes_no_copy() {
    cr!("608.2b");
    ruling!(
        "Sevinne's Reclamation",
        "If the target card is an illegal target by the time Sevinne's Reclamation tries to resolve, the spell doesn't resolve. You won't copy it if you cast it from a graveyard."
    );
    let mut t = TestGame::new(2);
    let bears = t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P0, "Hill Giant");
    flashback_reclamation(&mut t, bears);
    t.resolve();
    exile(&mut t, bears);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.resolve();
    assert!(spell_copies(&t).is_empty());
    assert!(t.g.stack.is_empty());
    assert!(t.named_on_battlefield("Hill Giant").is_empty());
}

// --- Banish into Fable ----------------------------------------------------------------------

#[test]
fn banish_into_fable_copies_twice_for_an_artifact_enchantment() {
    cr!("707.10", "608.2b");
    ruling!(
        "Banish into Fable",
        "If you control one permanent that's an artifact and an enchantment, you'll copy Banish into Fable twice."
    );
    ruling!(
        "Banish into Fable",
        "If the target nonland permanent is an illegal target by the time Banish into Fable tries to resolve, the spell won't resolve. You won't create a Knight token."
    );
    supported("Banish into Fable");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Spear of Heliod");
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_new(&mut t, P0, "Banish into Fable", &[obj(bears)]);
    keep_copy_targets(&mut t, P0);
    keep_copy_targets(&mut t, P0);
    t.settle();
    t.resolve();
    assert_eq!(spell_copies(&t).len(), 2);
    t.resolve_all();
    assert!(t.in_hand(P1, "Grizzly Bears"));
    // The first copy returned the Bears; the second copy and the original had an illegal
    // target and made no Knight.
    assert_eq!(named(&t, P0, "Knight Token").len(), 1);
}

// --- Super Combo ------------------------------------------------------------------------------

#[test]
fn super_combo_copy_with_an_illegal_target_deals_no_damage() {
    cr!("702.56a", "608.2b", "707.10c");
    ruling!(
        "Super Combo",
        "If either target is an illegal target as Super Combo resolves, the creature you control won't deal damage due to the effect of that copy of Super Combo."
    );
    supported("Super Combo");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    lands_for_cost(&mut t, P0, "Super Combo");
    t.lands(P0, "Wastes", 2);
    t.answer(P0, DecisionKind::OptionalCost, Answer::Number(1));
    let sc = t.hand(P0, "Super Combo");
    t.cast_with(P0, sc, &[obj(giant), obj(bears)]).unwrap();
    keep_copy_targets(&mut t, P0);
    t.settle();
    t.resolve();
    let copies = spell_copies(&t);
    assert_eq!(copies.len(), 1);
    assert_eq!(targets_of(&t, copies[0]), vec![obj(giant), obj(bears)]);
    // The Hill Giant is no longer a creature P0 controls.
    give_control(&mut t, giant, P1);
    t.resolve_all();
    assert!(t.on_battlefield(bears), "the Giant dealt no damage");
    assert_eq!(t.obj_now(bears).damage, 0);
}

// --- Show of Confidence -----------------------------------------------------------------------

/// P0 casts Shock and Opt (two instants), then Show of Confidence targeting `target`; the
/// cast trigger is on the stack.
fn show_of_confidence(t: &mut TestGame, target: ObjectId) {
    supported("Show of Confidence");
    cast_new(t, P0, "Shock", &[Entity::Player(P1)]);
    t.resolve_all();
    cast_new(t, P0, "Opt", &[]);
    t.resolve_all();
    cast_new(t, P0, "Show of Confidence", &[obj(target)]);
    t.settle();
}

#[test]
fn show_of_confidence_copies_keep_an_illegal_target() {
    cr!("707.10c", "608.2b");
    ruling!(
        "Show of Confidence",
        "If the target of the original Show of Confidence has become an illegal target when that spell is copied, the copies will initially still target the same creature. If you don’t choose new targets for the copies, they each will fail to resolve."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let other = t.battlefield(P0, "Hill Giant");
    show_of_confidence(&mut t, bears);
    kill(&mut t, bears);
    keep_copy_targets(&mut t, P0);
    keep_copy_targets(&mut t, P0);
    t.resolve();
    let copies = spell_copies(&t);
    assert_eq!(copies.len(), 2);
    for c in &copies {
        assert_eq!(targets_of(&t, *c), vec![obj(bears)]);
    }
    t.resolve_all();
    assert_eq!(t.counters(other, counters::PLUS1), 0);
}

#[test]
fn magecraft_triggers_for_each_show_of_confidence_copy() {
    cr!("707.10");
    ruling!(
        "Show of Confidence",
        "Magecraft abilities trigger once for each copy created as Show of Confidence’s triggered ability resolves."
    );
    supported("Archmage Emeritus");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Archmage Emeritus");
    show_of_confidence(&mut t, bears);
    let hand = t.hand_size(P0);
    keep_copy_targets(&mut t, P0);
    keep_copy_targets(&mut t, P0);
    t.resolve_all();
    // One draw for casting Show of Confidence, and one for each of its two copies.
    assert_eq!(t.hand_size(P0), hand + 3);
    assert_eq!(t.counters(bears, counters::PLUS1), 3);
}

// --- Malicious Affliction ----------------------------------------------------------------------

#[test]
fn malicious_affliction_s_copy_keeps_the_target_unless_changed() {
    cr!("707.10c");
    ruling!(
        "Malicious Affliction",
        "The copy will have the same target as the original spell unless you choose a new one."
    );
    supported("Malicious Affliction");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    let dies = t.battlefield(P1, "Hill Giant");
    kill(&mut t, dies);
    cast_new(&mut t, P0, "Malicious Affliction", &[obj(a)]);
    t.answer_yes(P0, true);
    keep_copy_targets(&mut t, P0);
    t.settle();
    t.resolve();
    let copies = spell_copies(&t);
    assert_eq!(copies.len(), 1);
    assert_eq!(targets_of(&t, copies[0]), vec![obj(a)]);
    t.resolve_all();
    assert!(!t.on_battlefield(a));
    assert!(t.on_battlefield(b));
}

// --- Mind's Desire, Galvanic Relay -------------------------------------------------------------

#[test]
fn mind_s_desire_s_card_is_face_up_and_stays_exiled() {
    cr!("406.3");
    ruling!("Mind's Desire", "The card is face-up when exiled.");
    ruling!(
        "Mind's Desire",
        "If the card is not played by end of turn, it remains exiled until end of game."
    );
    supported("Mind's Desire");
    let mut t = TestGame::new(2);
    empty_library(&mut t, P0);
    let bears = t.library_top(P0, "Grizzly Bears");
    cast_new(&mut t, P0, "Mind's Desire", &[]);
    t.resolve_all();
    let bears = t.g.current(bears);
    assert_eq!(t.zone(bears), Zone::Exile);
    assert!(!t.obj(bears).face_down);
    assert!(can_cast(&mut t, P0, bears, CastMethod::Free) || can_cast(&mut t, P0, bears, CastMethod::Normal));
    t.library_top(P0, "Island");
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    assert_eq!(t.zone(bears), Zone::Exile);
    lands_for_cost(&mut t, P0, "Grizzly Bears");
    assert!(!can_cast(&mut t, P0, bears, CastMethod::Free));
    assert!(!can_cast(&mut t, P0, bears, CastMethod::Normal));
}

#[test]
fn galvanic_relay_s_card_is_played_next_turn_normally() {
    cr!("601.3", "117.1a");
    ruling!(
        "Galvanic Relay",
        "Playing the card has the same timing restrictions and requires the same payments as normal, but it can only be played next turn."
    );
    supported("Galvanic Relay");
    let mut t = TestGame::new(2);
    let bears = t.library_top(P0, "Grizzly Bears");
    cast_new(&mut t, P0, "Galvanic Relay", &[]);
    t.resolve_all();
    let bears = t.g.current(bears);
    assert_eq!(t.zone(bears), Zone::Exile);
    lands_for_cost(&mut t, P0, "Grizzly Bears");
    assert!(!can_cast(&mut t, P0, bears, CastMethod::Normal), "not this turn");
    // Next turn: not during the upkeep (sorcery timing), not without mana.
    t.advance_to(P0, Step::Upkeep);
    assert!(!can_cast(&mut t, P0, bears, CastMethod::Normal), "timing");
    t.advance_to(P0, Step::PrecombatMain);
    for l in t.g.battlefield.clone() {
        if t.obj(l).is(CardType::Land) {
            t.g.tap(l);
        }
    }
    assert!(!can_cast(&mut t, P0, bears, CastMethod::Normal), "payment");
    lands_for_cost(&mut t, P0, "Grizzly Bears");
    assert!(can_cast(&mut t, P0, bears, CastMethod::Normal));
    t.cast_with(P0, bears, &[]).unwrap();
    t.resolve_all();
    assert_eq!(named(&t, P0, "Grizzly Bears").len(), 1);
}

// --- Impulsivity, Stream of Thought, A Little Chat ---------------------------------------------

#[test]
fn impulsivity_casts_the_card_while_resolving_ignoring_timing() {
    cr!("608.2g", "601.2");
    ruling!(
        "Impulsivity",
        "If you choose to cast the instant or sorcery card, you do so while Impulsivity's first ability is resolving and still on the stack. You can't wait to cast it later in the turn. Timing restrictions based on the card's type are ignored."
    );
    supported("Impulsivity");
    // During P1's turn, P0's Impulsivity enters: P0 casts Divination (a sorcery) from P1's
    // graveyard.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let div = t.graveyard(P1, "Divination");
    t.answer_targets(P0, &[obj(div)]);
    t.answer_yes(P0, true);
    t.enter(P0, "Impulsivity");
    t.g.flush_events();
    t.settle();
    let hand = t.hand_size(P0);
    t.resolve();
    // Divination is on the stack, cast by P0.
    assert_eq!(t.stack_len(), 1);
    assert_eq!(t.g.history.spells_cast.len(), 1);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
    assert!(t.in_exile("Divination"));
    // Declining: the card can't be cast later.
    let mut t = TestGame::new(2);
    let div = t.graveyard(P0, "Divination");
    t.answer_targets(P0, &[obj(div)]);
    t.answer_yes(P0, false);
    t.enter(P0, "Impulsivity");
    t.g.flush_events();
    t.resolve_all();
    lands_for_cost(&mut t, P0, "Divination");
    assert!(!can_cast(&mut t, P0, div, CastMethod::Normal));
    assert!(!can_cast(&mut t, P0, div, CastMethod::Free));
}

#[test]
fn stream_of_thought_can_t_shuffle_itself() {
    cr!("608.2", "608.2n");
    ruling!(
        "Stream of Thought",
        "Stream of Thought is still on the stack while you choose cards in your graveyard to shuffle into your library. It can’t shuffle itself into your library."
    );
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Grizzly Bears");
    let from = t.asked().len();
    cast_new(&mut t, P0, "Stream of Thought", &[Entity::Player(P1)]);
    t.resolve_all();
    let offered: Vec<Entity> = crate::r_s03_common::choice_candidates(&t, from, "")
        .into_iter()
        .flatten()
        .collect();
    assert!(!offered.is_empty());
    for e in offered {
        if let Entity::Object(id) = e {
            assert_ne!(t.obj(id).chars.name, "Stream of Thought");
        }
    }
    assert!(t.in_graveyard(P0, "Stream of Thought"));
}

#[test]
fn a_little_chat_with_one_card_in_the_library() {
    ruling!(
        "A Little Chat",
        "If there's only one card in your library as A Little Chat resolves, you'll put it into your hand."
    );
    supported("A Little Chat");
    let mut t = TestGame::new(2);
    empty_library(&mut t, P0);
    t.library_top(P0, "Grizzly Bears");
    cast_new(&mut t, P0, "A Little Chat", &[]);
    t.resolve_all();
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert_eq!(t.library_size(P0), 0);
}

// --- Plumb the Forbidden -------------------------------------------------------------------

#[test]
fn plumb_the_forbidden_copies_for_each_creature_sacrificed() {
    cr!("601.2b", "601.2i", "707.10");
    interp!("cost-paid-reflexive-trigger");
    ruling!(
        "Plumb the Forbidden",
        "If you copy Plumb the Forbidden multiple times, each magecraft ability of a creature you control will trigger that many times."
    );
    ruling!(
        "Plumb the Forbidden",
        "If you sacrifice a creature with a magecraft ability to cast Plumb the Forbidden, that creature's ability will not trigger."
    );
    supported("Plumb the Forbidden");
    supported("Archmage Emeritus");
    // (creatures sacrificed, Emeritus sacrificed too?) → (cards drawn, life lost)
    for (n, emeritus_too, draws, loss) in [
        (0, false, 2, 1),
        (2, false, 6, 3),
        (1, true, 2, 2),
    ] {
        let mut t = TestGame::new(2);
        let em = t.battlefield(P0, "Archmage Emeritus");
        let bears: Vec<ObjectId> = (0..2).map(|_| t.battlefield(P0, "Grizzly Bears")).collect();
        let sacs: Vec<Entity> = if emeritus_too {
            vec![obj(em)]
        } else {
            bears.iter().take(n).map(|b| obj(*b)).collect()
        };
        t.answer(P0, DecisionKind::OptionalCost, Answer::Number(n as i64));
        t.answer_choose(P0, &sacs);
        lands_for_cost(&mut t, P0, "Plumb the Forbidden");
        let plumb = t.hand(P0, "Plumb the Forbidden");
        let hand = t.hand_size(P0) - 1;
        t.cast_with(P0, plumb, &[]).unwrap();
        t.settle();
        assert_eq!(t.g.battlefield.iter().filter(|o| t.obj(**o).is(CardType::Creature)).count(), 3 - n);
        // The spell, with above it the copy trigger (if any creature was sacrificed) and
        // Emeritus's magecraft trigger (if it's still there), all controlled by P0.
        let want = 1 + usize::from(n > 0) + usize::from(!emeritus_too);
        assert_eq!(t.stack_len(), want, "sacrificed {n}, Emeritus: {emeritus_too}");
        assert_eq!(t.obj(t.g.stack[0]).chars.name, "Plumb the Forbidden");
        assert!(t.g.stack.iter().all(|s| t.obj(*s).controller == P0));
        t.resolve_all();
        assert!(spell_copies(&t).is_empty());
        assert_eq!(t.hand_size(P0), hand + draws, "sacrificed {n}, Emeritus: {emeritus_too}");
        assert_eq!(t.life(P0), 20 - loss);
    }
}
