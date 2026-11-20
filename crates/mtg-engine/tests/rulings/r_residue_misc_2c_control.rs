//! Item `rulings-residue-misc-2`, pass C — control changes, multiplayer and leaving the
//! game, votes, goad: Sigurd's boast under another player's control, Intellect Devourer's
//! exiled card that left exile, council's dilemma, Hot Pursuit, Debt of Loyalty on an
//! indestructible creature, Etrata's cloaked cards, Razorfield Ripper, Kaima, Linessa.

use crate::r_residue_misc_2c_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

const SIGURD: &str = "Sigurd, Jarl of Ravensthorpe";

// --- Sigurd, Jarl of Ravensthorpe -------------------------------------------------------------

#[test]
fn sigurd_put_onto_the_battlefield_attacking_cant_boast() {
    cr!("702.142a", "508.4");
    ruling!(
        "Sigurd, Jarl of Ravensthorpe",
        "If a creature with a boast ability is put onto the battlefield attacking, it was never declared as an attacker. Its boast ability can’t be activated that turn."
    );
    let mut t = TestGame::new(2);
    let saga = t.battlefield(P0, "History of Benalia");
    t.g.objects[saga.0 as usize]
        .counters
        .insert(counters::LORE.into(), 1);
    t.lands(P0, "Wastes", 2);
    t.set_step(P0, Step::DeclareAttackers);
    let sig = crate::r_p079_common::enter_attacking(&mut t, P0, SIGURD, pl(P1));
    assert!(t
        .g
        .combat
        .as_ref()
        .unwrap()
        .attackers
        .iter()
        .any(|a| a.id == sig));
    assert!(!can_activate(&mut t, P0, sig), "{}", t.dump_log());
}

#[test]
fn sigurd_gained_after_it_attacked_on_another_turn_can_boast() {
    cr!("702.142a", "506.4");
    ruling!(
        "Sigurd, Jarl of Ravensthorpe",
        "If it’s not your turn and you gain control of a creature with a boast ability after that creature attacked, you can activate that creature’s boast ability if it hasn’t been activated yet that turn."
    );
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let sig = t.battlefield(P1, SIGURD);
    let saga = t.battlefield(P0, "History of Benalia");
    t.g.objects[saga.0 as usize]
        .counters
        .insert(counters::LORE.into(), 1);
    attack_with(&mut t, &[(sig, pl(P0))]);
    give_control(&mut t, sig, P0);
    assert_eq!(controller(&t, sig), P0);
    t.lands(P0, "Wastes", 1);
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0]));
    t.answer_targets(P0, &[obj(saga)]);
    activate_containing(&mut t, P0, sig, "lore counter").expect("boast");
    t.resolve();
    assert_eq!(t.counters(saga, counters::LORE), 2, "{}", t.dump_log());
    // Once each turn.
    t.lands(P0, "Wastes", 1);
    assert!(!can_activate(&mut t, P0, sig));
}

// --- Intellect Devourer -----------------------------------------------------------------------

#[test]
fn intellect_devourer_doesnt_return_a_card_that_left_exile() {
    cr!("610.3", "610.3c", "400.7");
    ruling!(
        "Intellect Devourer",
        "If a card exiled with the Devour Intellect leaves exile for any reason (most likely because it was played using the Body Thief ability), it won't be returned to the opponent's hand if Intellect Devourer leaves the battlefield."
    );
    let mut t = TestGame::new(2);
    let forest = t.hand(P1, "Forest");
    let d = t.enter(P0, "Intellect Devourer");
    t.resolve_all();
    let exiled = t.g.current(forest);
    assert_eq!(t.zone(exiled), Zone::Exile);
    // P0 plays the Forest with Body Thief.
    t.play_land(P0, exiled).expect("Body Thief");
    let mine = t.g.current(exiled);
    assert_eq!(t.zone(mine), Zone::Battlefield);
    kill(&mut t, d);
    t.resolve_all();
    assert_eq!(t.hand_size(P1), 0, "{}", t.dump_log());
    assert!(t.on_battlefield(mine));
    assert_eq!(controller(&t, mine), P0);
}

// --- Council's dilemma (Expropriate's general rulings) ----------------------------------------

#[test]
fn councils_dilemma_votes_for_a_gone_creatures_counters_and_happens_in_order() {
    cr!("701.38a", "608.2c", "608.2b");
    ruling!(
        "Expropriate",
        "If a creature with an enters-the-battlefield council's dilemma ability leaves the battlefield before that ability resolves, players can still vote for any option that would put +1/+1 counters on that creature, even though—or perhaps especially because—those votes won't generate an effect."
    );
    ruling!(
        "Expropriate",
        "The effects of each council's dilemma ability happen in the stated order. First the vote occurs, then the first effect, and finally the second effect."
    );
    supported("Orchard Elemental");
    // The general council's dilemma ruling, on Orchard Elemental ("Council's dilemma —
    // When this creature enters, starting with you, each player votes for sprout or
    // harvest. Put two +1/+1 counters on this creature for each sprout vote. You gain 3
    // life for each harvest vote."): it's bounced in response; both may still vote sprout.
    let mut t = TestGame::new(2);
    let e = t.enter(P0, "Orchard Elemental");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    mv(&mut t, e, Zone::Hand(P0));
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.answer(P1, DecisionKind::Option, Answer::Index(0));
    let from = t.asked().len();
    t.resolve_all();
    let votes: Vec<(PlayerId, Vec<String>)> = asked_since(&t, from)
        .into_iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseOption { options, .. } => Some((p, options)),
            _ => None,
        })
        .collect();
    assert_eq!(votes.len(), 2, "{votes:?}");
    assert!(votes
        .iter()
        .all(|(_, o)| o.iter().any(|s| s.contains("sprout"))));
    assert_eq!(t.life(P0), 20);
    // Expropriate: the extra turn (time) is taken before the control change (money): it's
    // already queued as P0 chooses the permanent.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.players[0]
        .mana_pool
        .add_type(mtg_engine::mana::ManaType::U, 9);
    t.answer(P0, DecisionKind::Option, Answer::Index(0)); // time
    t.answer(P1, DecisionKind::Option, Answer::Index(1)); // money
    t.answer_choose(P0, &[obj(bears)]);
    let queued = watch(
        &mut t,
        P0,
        |d| matches!(d, Decision::ChooseEntities { .. }),
        |g| g.extra_turns.len(),
    );
    let c = t.hand(P0, "Expropriate");
    t.cast(P0, c).go();
    t.resolve_all();
    assert_eq!(controller(&t, bears), P0, "{}", t.dump_log());
    assert_eq!(*queued.lock().unwrap(), vec![1]);
}

// --- Hot Pursuit ------------------------------------------------------------------------------

#[test]
fn hot_pursuit_takes_every_goaded_or_suspected_creature_without_choices() {
    cr!("701.15a", "701.60a", "800.4a", "603.4");
    ruling!(
        "Hot Pursuit",
        "Hot Pursuit’s second triggered ability causes you to gain control of all creatures that are goaded, suspected, or both. You don’t have to make any choices."
    );
    supported("Hot Pursuit");
    let mut t = TestGame::new(4);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let other = t.battlefield(P1, "Savannah Lions");
    t.answer_targets(P0, &[obj(bears)]);
    t.enter(P0, "Hot Pursuit");
    t.resolve_all();
    assert!(goaded(&t, bears));
    // The Giant is goaded by another effect (P0's), tapped.
    mtg_engine::kwa::goad::goad(&mut t.g, giant, P0);
    t.g.objects[giant.0 as usize].tapped = true;
    concede(&mut t, P2);
    concede(&mut t, P3);
    let from = t.asked().len();
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    for c in [bears, giant] {
        assert_eq!(controller(&t, c), P0, "{}", t.dump_log());
        assert!(!t.obj_now(c).tapped);
        assert!(t.obj_now(c).has_keyword(KeywordKind::Haste));
    }
    assert_eq!(controller(&t, other), P1);
    assert!(!asked_since(&t, from).iter().any(|(_, d)| matches!(
        d,
        Decision::ChooseEntities { .. } | Decision::ChooseTargets { .. }
    )));
}

// --- Debt of Loyalty --------------------------------------------------------------------------

#[test]
fn debt_of_loyalty_on_an_indestructible_creature_does_nothing() {
    cr!("701.19a", "702.12b");
    ruling!(
        "Debt of Loyalty",
        "A creature with indestructible never needs to regenerate, so Debt of Loyalty will have no effect on it."
    );
    let mut t = TestGame::new(2);
    let myr = t.battlefield(P1, "Darksteel Myr");
    cast_new(&mut t, P0, "Debt of Loyalty", &[obj(myr)]);
    t.resolve_all();
    destroy(&mut t, myr);
    t.resolve_all();
    assert!(t.on_battlefield(myr));
    assert_eq!(controller(&t, myr), P1, "{}", t.dump_log());
    // Not regenerated (it's still untapped and in no combat), and still P1's next turn.
    assert!(!t.obj_now(myr).tapped);
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(controller(&t, myr), P1);
}

// --- Etrata, Deadly Fugitive --------------------------------------------------------------------

/// A three-player game: P0's Etrata deals combat damage to P1 and cloaks the top card of
/// P1's library (a Hill Giant). Returns the cloaked permanent.
fn etrata_cloaks() -> (TestGame, ObjectId) {
    let mut t = TestGame::new(3);
    let etrata = t.battlefield(P0, "Etrata, Deadly Fugitive");
    let giant = t.library_top(P1, "Hill Giant");
    attack_with(&mut t, &[(etrata, pl(P1))]);
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    let cloaked = t.g.current(giant);
    assert!(t.obj_now(cloaked).face_down, "{}", t.dump_log());
    assert_eq!(controller(&t, cloaked), P0);
    (t, cloaked)
}

#[test]
fn etratas_cloaked_cards_leave_with_their_owner_or_are_exiled_with_their_controller() {
    cr!("800.4a", "701.58a");
    ruling!(
        "Etrata, Deadly Fugitive",
        "In a multiplayer game, if an opponent leaves the game, all of the cards they own that you cloaked leave as well. If you leave the game, the creatures you cloaked with Etrata, Deadly Fugitive's triggered ability are exiled."
    );
    supported("Etrata, Deadly Fugitive");
    let (mut t, cloaked) = etrata_cloaks();
    concede(&mut t, P1);
    assert!(!t.g.is_live(cloaked) || t.zone(cloaked) != Zone::Battlefield);
    assert!(t.g.permanents().all(|o| !o.face_down), "{}", t.dump_log());
    let (mut t, cloaked) = etrata_cloaks();
    concede(&mut t, P0);
    let now = t.g.current(cloaked);
    assert_eq!(t.zone(now), Zone::Exile, "{}", t.dump_log());
}

#[test]
fn etratas_cloaked_cards_cant_be_looked_at_by_their_owners() {
    cr!("701.58a", "708.5");
    ruling!(
        "Etrata, Deadly Fugitive",
        "Your opponents can't look at cards they own that you cloaked."
    );
    let (t, cloaked) = etrata_cloaks();
    assert!(mtg_engine::facedown::can_look_at(&t.g, P0, cloaked));
    assert!(!mtg_engine::facedown::can_look_at(&t.g, P1, cloaked));
    assert!(!mtg_engine::facedown::can_look_at(&t.g, P2, cloaked));
}

// --- Razorfield Ripper ------------------------------------------------------------------------

#[test]
fn razorfield_ripper_attached_doesnt_tap_with_the_creature() {
    cr!("702.151b", "301.5c", "701.26a");
    ruling!(
        "Razorfield Ripper",
        "An Equipment doesn't become tapped when the permanent it's attached to becomes tapped. For example, if you attack with a creature that is equipped with Razorfield Ripper, then use reconfigure to unattach Razorfield Ripper after combat, Razorfield Ripper will be untapped and could be used to block during your opponent's turn."
    );
    let mut t = TestGame::new(2);
    let ripper = t.battlefield(P0, "Razorfield Ripper");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Wastes", 4);
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    activate_containing(&mut t, P0, ripper, "Reconfigure").expect("attach");
    t.answer_targets(P0, &[]);
    t.resolve_all();
    assert_eq!(
        t.obj_now(ripper).attached_to,
        Some(obj(bears)),
        "{}",
        t.dump_log()
    );
    attack_with(&mut t, &[(bears, pl(P1))]);
    t.advance_to(P0, Step::PostcombatMain);
    t.resolve_all();
    assert!(t.obj_now(bears).tapped);
    assert!(!t.obj_now(ripper).tapped);
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    // Its second reconfigure ability: unattach.
    t.activate(P0, ripper, 1, &[]).expect("unattach");
    t.resolve_all();
    assert_eq!(t.obj_now(ripper).attached_to, None);
    assert!(is_creature(&mut t, ripper));
    assert!(!t.obj_now(ripper).tapped);
    assert!(t.g.can_block_at_all(t.g.current(ripper)));
}

// --- Kaima, the Fractured Calm ----------------------------------------------------------------

#[test]
fn kaima_goads_the_creatures_enchanted_as_it_resolves_and_only_them() {
    cr!("701.15a", "608.2h", "611.2c");
    ruling!(
        "Kaima, the Fractured Calm",
        "The set of creatures that are goaded by this ability are locked in when it resolves. A creature becoming enchanted later or an Aura being removed from a creature later won't change whether it is goaded."
    );
    supported("Kaima, the Fractured Calm");
    let mut t = TestGame::new(2);
    let kaima = t.battlefield(P0, "Kaima, the Fractured Calm");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let aura = crate::r_s06_common::attach_new(&mut t, P0, "Holy Strength", bears);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(goaded(&t, bears), "{}", t.dump_log());
    assert!(!goaded(&t, giant));
    assert_eq!(t.counters(kaima, counters::PLUS1), 1);
    // The Aura leaves; another one enchants the Giant: no change.
    destroy(&mut t, aura);
    crate::r_s06_common::attach_new(&mut t, P0, "Holy Strength", giant);
    assert!(goaded(&t, bears));
    assert!(!goaded(&t, giant));
}

// --- Linessa, Zephyr Mage ---------------------------------------------------------------------

#[test]
fn linessa_grandeur_the_player_returns_one_permanent_per_type_in_sequence() {
    cr!("702.71a", "115.1", "608.2c");
    ruling!(
        "Linessa, Zephyr Mage",
        "Linessa’s grandeur ability targets only a player. That player chooses which permanents to return as the ability resolves. The permanents are returned one at a time, in a specific sequence. The player can’t return an artifact creature and have that count as both the artifact and the creature, for example."
    );
    supported("Linessa, Zephyr Mage");
    let mut t = TestGame::new(2);
    let linessa = t.battlefield(P0, "Linessa, Zephyr Mage");
    t.hand(P0, "Linessa, Zephyr Mage");
    // P1: an artifact creature and a creature, an enchantment, a land.
    let thopter = t.battlefield(P1, "Ornithopter");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Solemnity");
    t.battlefield(P1, "Forest");
    // P1 returns the Ornithopter as the creature: no artifact is left for the artifact.
    t.answer_choose(P1, &[obj(thopter)]);
    let from = t.asked().len();
    t.answer_targets(P0, &[pl(P1)]);
    activate_containing(&mut t, P0, linessa, "Discard another").expect("grandeur");
    t.resolve_all();
    let targets: Vec<Vec<Entity>> = asked_since(&t, from)
        .into_iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseTargets { candidates, .. } => Some(candidates),
            _ => None,
        })
        .collect();
    assert_eq!(targets.len(), 1);
    assert!(targets[0].iter().all(|e| matches!(e, Entity::Player(_))));
    let choosers: Vec<PlayerId> = asked_since(&t, from)
        .into_iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseEntities { candidates, .. } if candidates.len() > 1))
        .map(|(p, _)| p)
        .collect();
    assert!(choosers.iter().all(|p| *p == P1), "{choosers:?}");
    assert!(t.in_hand(P1, "Ornithopter"), "{}", t.dump_log());
    assert!(t.on_battlefield(bears));
    assert!(t.in_hand(P1, "Solemnity"));
    assert!(t.in_hand(P1, "Forest"));
}
