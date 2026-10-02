//! Rulings batch P223 — spell mastery (an ability word, CR 207.2c) and splice
//! (CR 702.47): Exquisite Firecraft, Dark Dabbling, Send to Sleep, Gideon's Phalanx,
//! Ravaging Blaze, Desperate Ritual, Splicer's Skill, Everdream.

use crate::r_p223_common::*;
use crate::r_s01_common::*;
use crate::r_s02_common::destroy;
use crate::r_s03_common::{in_hand_with_mana, respond};
use crate::r_s06_common::give_control;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn instants_in_graveyard(t: &mut TestGame, p: PlayerId, n: usize) {
    for _ in 0..n {
        t.graveyard(p, "Shock");
    }
}

#[test]
fn an_uncounterable_firecraft_still_gets_the_counterspells_other_effects() {
    cr!("101.2", "608.2c", "701.6a");
    ruling!("Exquisite Firecraft", "A spell or ability that counters spells can still target Exquisite Firecraft. When that spell or ability resolves, Exquisite Firecraft won't be countered if its spell mastery ability applies, but any additional effects of the countering spell or ability will still happen.");
    supported("Exquisite Firecraft");
    supported("Dissolve");
    // Dissolve: "Counter target spell. Scry 1."
    for mastery in [true, false] {
        let mut t = TestGame::new(2);
        for _ in 0..3 {
            t.library_top(P1, "Hill Giant");
        }
        instants_in_graveyard(&mut t, P0, if mastery { 2 } else { 0 });
        let firecraft = in_hand_with_mana(&mut t, P0, "Exquisite Firecraft");
        let spell = t.cast(P0, firecraft).target(P1).go();
        let dissolve = in_hand_with_mana(&mut t, P1, "Dissolve");
        t.cast(P1, dissolve).target(spell).go();
        let from = t.asked().len();
        t.resolve_all();
        assert_eq!(t.life(P1), if mastery { 16 } else { 20 });
        assert_eq!(scry_sizes(&t, P1, from), vec![1]);
    }
}

#[test]
fn dark_dabblings_spell_mastery_sees_cards_dredged_by_its_draw() {
    cr!("608.2c", "702.52a", "701.19a");
    ruling!("Dark Dabbling", "Dark Dabbling’s spell mastery ability is a normal spell effect, not a replacement effect. This means that if you replace drawing a card for Dark Dabbling’s effect with dredging a card from your graveyard, the cards put into your graveyard this way may enable the additional regeneration.");
    supported("Dark Dabbling");
    supported("Golgari Thug");
    // Two Shocks in the top four cards: dredging Golgari Thug (dredge 4) mills them.
    for dredge in [true, false] {
        let mut t = TestGame::new(2);
        stack_library(&mut t, P0, &["Shock", "Shock", "Hill Giant", "Hill Giant"]);
        t.graveyard(P0, "Golgari Thug");
        let target = t.battlefield(P0, "Grizzly Bears");
        let other = t.battlefield(P0, "Savannah Lions");
        let dd = in_hand_with_mana(&mut t, P0, "Dark Dabbling");
        t.answer_yes(P0, dredge);
        t.cast(P0, dd).target(target).go();
        t.resolve_all();
        assert_eq!(t.in_hand(P0, "Golgari Thug"), dredge);
        // Both regenerated (or only the target).
        destroy(&mut t, target);
        destroy(&mut t, other);
        assert!(t.on_battlefield(target));
        assert_eq!(t.on_battlefield(other), dredge);
    }
}

/// P0 casts Send to Sleep on `targets` (P1's creatures) with spell mastery if `mastery`.
fn send_to_sleep(t: &mut TestGame, targets: &[ObjectId], mastery: bool) -> ObjectId {
    supported("Send to Sleep");
    instants_in_graveyard(t, P0, if mastery { 2 } else { 0 });
    let s = in_hand_with_mana(t, P0, "Send to Sleep");
    let es: Vec<Entity> = targets.iter().map(|x| Entity::Object(*x)).collect();
    t.cast(P0, s).targets(&es).go()
}

#[test]
fn send_to_sleep_an_already_tapped_creature_stays_tapped() {
    cr!("608.2c", "502.3");
    ruling!("Send to Sleep", "Send to Sleep can target tapped creatures. If a targeted creature is already tapped when the spell resolves (and the spell mastery ability applies), that creature remains tapped and doesn’t untap during its controller’s next untap step.");
    for mastery in [true, false] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.g.tap(bears);
        send_to_sleep(&mut t, &[bears], mastery);
        t.resolve_all();
        assert!(t.obj_now(bears).tapped);
        t.advance_to(P1, Step::Upkeep);
        assert_eq!(t.obj_now(bears).tapped, mastery);
    }
}

#[test]
fn send_to_sleep_doesnt_affect_an_illegal_target_at_all() {
    cr!("608.2b", "702.16b");
    ruling!("Send to Sleep", "If you chose two targets and one is an illegal target when Send to Sleep resolves, that creature won’t become tapped and it won’t be stopped from untapping during its controller’s next untap step (if the spell mastery ability applies). It won’t be affected by Send to Sleep in any way.");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    send_to_sleep(&mut t, &[a, b], true);
    // In response, B gains protection from blue.
    let gw = in_hand_with_mana(&mut t, P1, "Gods Willing");
    let blue = Color::ALL.iter().position(|c| *c == Color::Blue).unwrap();
    t.answer(P1, DecisionKind::Option, Answer::Index(blue));
    t.cast(P1, gw).target(b).go();
    t.resolve_all();
    assert!(t.obj_now(a).tapped);
    assert!(!t.obj_now(b).tapped);
    // B becomes tapped some other way: it still untaps normally.
    t.g.tap(b);
    t.advance_to(P1, Step::Upkeep);
    assert!(t.obj_now(a).tapped);
    assert!(!t.obj_now(b).tapped);
}

#[test]
fn send_to_sleep_follows_the_creature_to_its_new_controller() {
    cr!("608.2c", "502.3", "611.2c");
    ruling!("Send to Sleep", "If the spell mastery ability applies and a creature affected by Send to Sleep changes controllers before its old controller’s next untap step, Send to Sleep will prevent it from becoming untapped during its new controller’s next untap step.");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    send_to_sleep(&mut t, &[bears], true);
    t.resolve_all();
    give_control(&mut t, bears, P0);
    assert_eq!(t.obj_now(bears).controller, P0);
    // P1's untap step doesn't matter; P0's next one doesn't untap it.
    t.advance_to(P1, Step::Upkeep);
    assert!(t.obj_now(bears).tapped);
    t.advance_to(P0, Step::Upkeep);
    assert!(t.obj_now(bears).tapped);
    // The one after that does.
    t.advance_to(P0, Step::End);
    t.advance_to(P0, Step::Upkeep);
    assert!(!t.obj_now(bears).tapped);
}

#[test]
fn gideons_phalanx_knights_gain_indestructible_too() {
    cr!("608.2c", "611.2c", "702.12b");
    ruling!("Gideon's Phalanx", "If the spell mastery ability applies, the four Knight tokens will also gain indestructible until end of turn.");
    supported("Gideon's Phalanx");
    for mastery in [true, false] {
        let mut t = TestGame::new(2);
        instants_in_graveyard(&mut t, P0, if mastery { 2 } else { 0 });
        let bears = t.battlefield(P0, "Grizzly Bears");
        let gp = in_hand_with_mana(&mut t, P0, "Gideon's Phalanx");
        t.cast(P0, gp).go();
        t.resolve_all();
        let knights = with_subtype(&t, P0, "Knight");
        assert_eq!(knights.len(), 4);
        for k in knights.iter().chain([bears].iter()) {
            assert_eq!(
                t.obj_now(*k).has_keyword(KeywordKind::Indestructible),
                mastery
            );
        }
    }
}

#[test]
fn ravaging_blaze_targets_only_the_creature() {
    cr!("115.10a", "702.11b", "608.2c");
    ruling!("Ravaging Blaze", "Ravaging Blaze targets only the creature. It doesn’t target any player, even if the spell mastery ability applies.");
    supported("Ravaging Blaze");
    // P1 has hexproof (Leyline of Sanctity), yet is dealt damage.
    let mut t = TestGame::new(2);
    instants_in_graveyard(&mut t, P0, 2);
    t.battlefield(P1, "Leyline of Sanctity");
    let giant = t.battlefield(P1, "Hill Giant");
    let blaze = in_hand_with_mana(&mut t, P0, "Ravaging Blaze");
    t.lands(P0, "Wastes", 3);
    let from = t.asked().len();
    t.cast(P0, blaze).target(giant).x(3).go();
    let target_choices = t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseTargets { .. }))
        .count();
    assert_eq!(target_choices, 1);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Hill Giant"));
    assert_eq!(t.life(P1), 17);
}

#[test]
fn desperate_ritual_uses_the_stack() {
    cr!("605.1a", "405.1");
    ruling!("Desperate Ritual", "Desperate Ritual isn’t a mana ability. When cast, it (or the spell it’s spliced onto) goes on the stack like any spell and can be responded to.");
    supported("Desperate Ritual");
    let mut t = TestGame::new(2);
    let ritual = in_hand_with_mana(&mut t, P0, "Desperate Ritual");
    let spell = t.cast(P0, ritual).go();
    // On the stack; no mana yet, and the opponent may respond.
    assert!(t.g.stack.contains(&spell));
    assert_eq!(t.g.player(P0).mana_pool.count(ManaType::R), 0);
    let shock = in_hand_with_mana(&mut t, P1, "Lightning Bolt");
    t.cast(P1, shock).target(P0).go();
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    assert_eq!(t.life(P0), 17);
    t.resolve();
    assert_eq!(t.g.player(P0).mana_pool.count(ManaType::R), 3);
}

fn splice(t: &mut TestGame, p: PlayerId, yes: bool) {
    t.answer(p, DecisionKind::OptionalCost, Answer::Bool(yes));
}

#[test]
fn splicers_skill_still_creates_a_golem_when_the_spell_makes_its_target_illegal() {
    cr!("608.2b", "702.47a");
    ruling!("Splicer's Skill", "The legality of a spell’s targets is checked only as that spell begins to resolve. If the spell this card is spliced onto causes its targets to become illegal while it’s resolving (for example, by removing them from the battlefield) you’ll still create a Golem.");
    supported("Splicer's Skill");
    supported("Unsummon");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let unsummon = in_hand_with_mana(&mut t, P0, "Unsummon");
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Wastes", 3);
    t.hand(P0, "Splicer's Skill");
    splice(&mut t, P0, true);
    t.cast(P0, unsummon).target(bears).go();
    t.resolve_all();
    assert!(t.in_hand(P1, "Grizzly Bears"));
    assert_eq!(with_subtype(&t, P0, "Golem").len(), 1);
    assert!(t.in_hand(P0, "Splicer's Skill"));
}

#[test]
fn spliced_effects_happen_last() {
    cr!("702.47a", "608.2c");
    ruling!("Everdream", "The abilities spliced onto the spell happen last, after all of that spell’s other effects.");
    supported("Everdream");
    supported("Faithless Looting");
    // Faithless Looting: "Draw two cards, then discard two cards." With Everdream ("Draw a
    // card.") spliced on, you discard before drawing Everdream's card.
    let mut t = TestGame::new(2);
    stack_library(&mut t, P0, &["Hill Giant", "Hill Giant", "Grizzly Bears"]);
    let looting = in_hand_with_mana(&mut t, P0, "Faithless Looting");
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 2);
    t.hand(P0, "Everdream");
    splice(&mut t, P0, true);
    t.cast(P0, looting).go();
    // Only Everdream is in hand now.
    assert_eq!(t.hand_size(P0), 1);
    // Discard the two drawn cards (not Everdream).
    respond(&mut t, P0, |g, d| match d {
        Decision::ChooseEntities { candidates, .. } => Some(Answer::Entities(
            candidates
                .iter()
                .filter(|e| e.object().is_some_and(|o| g.obj(o).chars.name != "Everdream"))
                .copied()
                .collect(),
        )),
        _ => None,
    });
    let seen = watch(
        &mut t,
        P0,
        |d| matches!(d, Decision::ChooseEntities { prompt, .. } if prompt.to_lowercase().contains("discard")),
        |g| g.player(P0).hand.len(),
    );
    t.resolve_all();
    // At the discard: Everdream plus two drawn cards. Afterwards: Everdream plus the
    // card drawn last.
    assert_eq!(*seen.lock().unwrap(), vec![3]);
    assert_eq!(t.hand_size(P0), 2);
    assert!(t.in_hand(P0, "Everdream"));
    assert!(t.in_hand(P0, "Grizzly Bears"));
}
