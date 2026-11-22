//! Rulings batch P203 — clash (CR 701.30): who you clash with, when the clash happens
//! relative to the rest of the effect, and what "if you win" does.

use crate::r_s01_common::*;
use crate::r_s02_common::destroy;
use crate::r_s03_common::*;
use crate::r_s06_common::attach_new;
use mtg_engine::ability::{Effect, Filter, Sel};
use mtg_engine::decision::Decision;
use mtg_engine::events::Event;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Who clashed (each clashing player reports a "clash" event) since the start of the turn.
fn clashers(t: &TestGame) -> Vec<PlayerId> {
    t.g.turn_events
        .iter()
        .filter_map(|e| match e {
            Event::Custom { name, player, .. } if name == "clash" => *player,
            _ => None,
        })
        .collect()
}

#[test]
fn ringskipper_still_clashes_if_it_left_the_graveyard_but_doesnt_return() {
    cr!("701.30a", "701.30b", "603.6c", "400.7");
    ruling!(
        "Ringskipper",
        "If Ringskipper is removed from the graveyard before the ability resolves, you still clash, but nothing will happen if you win."
    );
    supported("Ringskipper");
    let mut t = TestGame::new(2);
    stack_library(&mut t, P0, &["Colossal Dreadmaw"]);
    let rs = t.battlefield(P0, "Ringskipper");
    destroy(&mut t, rs);
    assert_eq!(triggers_on_stack(&t, "clash"), 1);
    // In response, the card is exiled from the graveyard.
    let dead = t.g.find_in_zone(Zone::Graveyard(P0), "Ringskipper")[0];
    t.g.exile_object(dead, None);
    t.resolve_all();
    assert_eq!(clashers(&t), vec![P0, P1]);
    assert!(t.in_exile("Ringskipper"));
    assert!(!t.in_hand(P0, "Ringskipper"));

    // Left in the graveyard, a win returns it.
    let mut t = TestGame::new(2);
    stack_library(&mut t, P0, &["Colossal Dreadmaw"]);
    let rs = t.battlefield(P0, "Ringskipper");
    destroy(&mut t, rs);
    t.resolve_all();
    assert!(t.in_hand(P0, "Ringskipper"));
}

#[test]
fn recross_the_paths_puts_the_revealed_nonland_cards_on_the_bottom_before_clashing() {
    cr!("701.30a", "701.20a", "608.2c");
    ruling!(
        "Recross the Paths",
        "You put the revealed nonland cards on the bottom of your library before you clash."
    );
    supported("Recross the Paths");
    // Library: Hill Giant (MV 4), Plains, Grizzly Bears (MV 2), ... The Giant goes to the
    // bottom before the clash, so P0 reveals the Bears (2 > P1's filler, MV 0) and wins.
    let mut t = TestGame::new(2);
    stack_library(&mut t, P0, &["Hill Giant", "Plains", "Grizzly Bears"]);
    t.library_top(P1, "Llanowar Elves"); // MV 1
    let c = in_hand_with_mana(&mut t, P0, "Recross the Paths");
    t.cast(P0, c).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Plains").len(), 1);
    let lib = &t.g.player(P0).library;
    assert_eq!(t.obj(lib[0]).chars.name, "Hill Giant");
    assert_eq!(t.obj(*lib.last().unwrap()).chars.name, "Grizzly Bears");
    // P0 revealed the Bears, not the Giant: 2 > 1, a win.
    assert!(t.in_hand(P0, "Recross the Paths"));

    // With the Giant still on top P0 would have won too, so check a loss: Elves (1) vs
    // Grizzly Bears (2) for P1.
    let mut t = TestGame::new(2);
    stack_library(&mut t, P0, &["Hill Giant", "Plains", "Llanowar Elves"]);
    t.library_top(P1, "Grizzly Bears");
    let c = in_hand_with_mana(&mut t, P0, "Recross the Paths");
    t.cast(P0, c).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Recross the Paths"));
}

#[test]
fn recross_the_paths_with_no_land_reveals_the_whole_library_then_clashes() {
    cr!("701.30a", "701.20a");
    ruling!(
        "Recross the Paths",
        "If you have only nonland cards in your library, you reveal your entire library (one card at a time), rearrange it however you like, and then clash."
    );
    let mut t = TestGame::new(2);
    // Only nonland cards: the 30 filler cards and a Colossal Dreadmaw on top.
    stack_library(&mut t, P0, &["Colossal Dreadmaw"]);
    let size = t.library_size(P0);
    let c = in_hand_with_mana(&mut t, P0, "Recross the Paths");
    t.cast(P0, c).go();
    t.resolve_all();
    // Nothing entered, the whole library is still there, and P0 still clashed.
    assert!(t.named_on_battlefield("Colossal Dreadmaw").is_empty());
    assert_eq!(t.library_size(P0), size);
    assert_eq!(clashers(&t), vec![P0, P1]);
}

#[test]
fn spring_cleaning_destroys_every_opponents_enchantments_and_the_target_again() {
    cr!("701.30b", "701.8a", "701.19a", "608.2c");
    ruling!(
        "Spring Cleaning",
        "If you win the clash, you destroy all enchantments all of your opponents control, not just enchantments controlled by the opponent you clashed with."
    );
    ruling!(
        "Spring Cleaning",
        "If you win the clash and the targeted enchantment is controlled by an opponent, it will be destroyed a second time if it's still on the battlefield (for instance, if it somehow regenerated)."
    );
    supported("Spring Cleaning");
    // Three players: P0 clashes with P1 and wins; P2's enchantments are destroyed too, and
    // the targeted Anthem, which regenerated, is destroyed again. P0's own survive.
    let mut t = TestGame::new(3);
    stack_library(&mut t, P0, &["Colossal Dreadmaw"]);
    let target = t.battlefield(P1, "Glorious Anthem");
    let p1_other = t.battlefield(P1, "Glorious Anthem");
    let p2s = t.battlefield(P2, "Glorious Anthem");
    let mine = t.battlefield(P0, "Glorious Anthem");
    run_effect(
        &mut t,
        None,
        P1,
        Effect::Regenerate {
            what: Sel::All(Filter::Objects(vec![target])),
        },
        &[],
    );
    let c = in_hand_with_mana(&mut t, P0, "Spring Cleaning");
    t.cast(P0, c).target(target).go();
    t.answer_choose(P0, &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(clashers(&t), vec![P0, P1]);
    for x in [target, p1_other, p2s] {
        assert!(!t.on_battlefield(x), "{}", t.dump_log());
    }
    assert!(t.on_battlefield(mine));
    assert!(t.in_graveyard(P1, "Glorious Anthem"));

    // Losing the clash: the Anthem regenerated (it survives), and the rest stay.
    let mut t = TestGame::new(2);
    stack_library(&mut t, P1, &["Colossal Dreadmaw"]);
    let target = t.battlefield(P1, "Glorious Anthem");
    let other = t.battlefield(P1, "Glorious Anthem");
    run_effect(
        &mut t,
        None,
        P1,
        Effect::Regenerate {
            what: Sel::All(Filter::Objects(vec![target])),
        },
        &[],
    );
    let c = in_hand_with_mana(&mut t, P0, "Spring Cleaning");
    t.cast(P0, c).target(target).go();
    t.resolve_all();
    assert!(t.on_battlefield(target));
    assert!(t.on_battlefield(other));
}

#[test]
fn captivating_glance_gives_the_creature_to_the_clash_winner_or_the_other_player() {
    cr!("701.30b", "701.30d", "613.1b");
    ruling!(
        "Captivating Glance",
        "If you win the clash, you gain control of the enchanted creature. If you don't win the clash, the other player gains control of the enchanted creature (even if that player didn't win the clash either)."
    );
    ruling!(
        "Captivating Glance",
        "The opponent you clash with doesn't have to be the controller of the enchanted creature."
    );
    supported("Captivating Glance");
    // Win: P0 (clashing with P1) gains control of P2's creature.
    let mut t = TestGame::new(3);
    stack_library(&mut t, P0, &["Colossal Dreadmaw"]);
    let giant = t.battlefield(P2, "Hill Giant");
    attach_new(&mut t, P0, "Captivating Glance", giant);
    t.answer_choose(P0, &[Entity::Player(P1)]);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(clashers(&t), vec![P0, P1]);
    assert_eq!(t.obj_now(giant).controller, P0);

    // A tie (nobody wins): the opponent P0 clashed with, P1, gains control of P2's
    // creature.
    let mut t = TestGame::new(3);
    let giant = t.battlefield(P2, "Hill Giant");
    attach_new(&mut t, P0, "Captivating Glance", giant);
    t.answer_choose(P0, &[Entity::Player(P1)]);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(clashers(&t), vec![P0, P1]);
    assert_eq!(t.obj_now(giant).controller, P1);
}

#[test]
fn broken_ambitions_clashes_whether_or_not_the_spell_was_countered() {
    cr!("701.30b", "608.2c", "701.17a");
    ruling!(
        "Broken Ambitions",
        "The clash (and the result of the clash if you win) happens regardless of whether the targeted spell was countered or {X} was paid."
    );
    ruling!(
        "Broken Ambitions",
        "The opponent you clash with doesn't have to be the controller of the targeted spell."
    );
    supported("Broken Ambitions");
    // X = 0: P2 "pays" {0}, so their spell isn't countered (X = 5: P2 can't pay); P0 clashes with P1, wins, and
    // the spell's controller, P2, mills four.
    for x in [0i64, 5] {
        let mut t = TestGame::new(3);
        stack_library(&mut t, P0, &["Colossal Dreadmaw"]);
        let opt = in_hand_with_mana(&mut t, P2, "Opt");
        let spell = t.cast(P2, opt).go();
        let c = in_hand_with_mana(&mut t, P0, "Broken Ambitions");
        t.lands(P0, "Wastes", x as usize);
        t.cast(P0, c).x(x).target(spell).go();
        t.answer_choose(P0, &[Entity::Player(P1)]);
        t.answer_yes(P2, true);
        let (l1, l2) = (t.library_size(P1), t.library_size(P2));
        t.resolve();
        assert_eq!(clashers(&t), vec![P0, P1], "x = {x}");
        // Countered only when P2 couldn't pay X.
        let countered = t.in_graveyard(P2, "Opt");
        assert_eq!(countered, x == 5, "x = {x}");
        assert_eq!(t.library_size(P2), l2 - 4);
        assert_eq!(t.library_size(P1), l1);
    }
}

#[test]
fn hoarders_greed_repeats_until_you_dont_win() {
    cr!("701.30a", "701.30d", "608.2c");
    ruling!(
        "Hoarder's Greed",
        "The effect will automatically repeat itself until its controller doesn't win the clash. There's no other way to stop it."
    );
    supported("Hoarder's Greed");
    // P0 draws two (Bears, Bears), clashes with Hill Giant (4) on top, wins and keeps it
    // there; draws Giant and Elves, then clashes with Dreadmaw (6) and wins; draws Dreadmaw
    // and a filler, then reveals a filler (0 vs P1's 0): no win, done.
    let mut t = TestGame::new(2);
    stack_library(
        &mut t,
        P0,
        &[
            "Grizzly Bears",
            "Grizzly Bears",
            "Hill Giant",
            "Llanowar Elves",
            "Colossal Dreadmaw",
        ],
    );
    let c = in_hand_with_mana(&mut t, P0, "Hoarder's Greed");
    t.cast(P0, c).go();
    let hand = t.hand_size(P0);
    let from = t.asked().len();
    t.resolve_all();
    assert_eq!(t.life(P0), 14);
    assert_eq!(t.hand_size(P0), hand + 6);
    assert_eq!(clashers(&t).len(), 6);
    // There was never a choice to stop.
    assert!(!t.asked()[from..]
        .iter()
        .any(|(p, d)| *p == P0 && matches!(d, Decision::YesNo { prompt, .. } if !prompt.contains("Clash"))));
}

#[test]
fn pulling_teeth_can_clash_with_a_player_it_doesnt_target() {
    cr!("701.30b", "701.9a");
    ruling!(
        "Pulling Teeth",
        "The opponent you clash with doesn't have to be the player targeted by Pulling Teeth."
    );
    supported("Pulling Teeth");
    let mut t = TestGame::new(3);
    stack_library(&mut t, P0, &["Colossal Dreadmaw"]);
    for _ in 0..3 {
        t.hand(P2, "Grizzly Bears");
        t.hand(P1, "Grizzly Bears");
    }
    let c = in_hand_with_mana(&mut t, P0, "Pulling Teeth");
    t.cast(P0, c).target(P2).go();
    t.answer_choose(P0, &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(clashers(&t), vec![P0, P1]);
    // P0 won: the target, P2, discards two; P1 discards nothing.
    assert_eq!(t.hand_size(P2), 1);
    assert_eq!(t.hand_size(P1), 3);
}
