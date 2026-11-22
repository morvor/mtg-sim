//! Rulings batch P111 — "whenever you cast a spell during an opponent's turn" and "your
//! first spell during each opponent's turn": the trigger resolves before the spell, even
//! if it's countered (CR 603.3, 601.2i); with several opponents it can trigger once in
//! each opponent's turn, and once in an opposing team's shared turn (CR 805.4); "each
//! opponent loses 1 life" is one choice (CR 603.5).

use crate::r_p044_common::two_headed_giant;
use crate::r_p119_common::counter_spell;
use crate::r_s01_common::supported;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::mana::ManaType;
use mtg_engine::object::ObjKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// `p` casts Lightning Bolt at `at` from their hand during whatever turn it is.
fn bolt(t: &mut TestGame, p: PlayerId, at: Entity) -> ObjectId {
    bolt_then(t, p, at, &[])
}

/// `p` casts Lightning Bolt at `at`; the abilities that trigger target `then` (in order).
fn bolt_then(t: &mut TestGame, p: PlayerId, at: Entity, then: &[Entity]) -> ObjectId {
    let b = t.hand(p, "Lightning Bolt");
    t.g.players[p.idx()].mana_pool.add_type(ManaType::R, 1);
    t.answer_targets(p, &[at]);
    for e in then {
        t.answer_targets(p, &[*e]);
    }
    t.cast(p, b).go();
    t.settle();
    t.g.current(b)
}

#[test]
fn the_trigger_resolves_first_even_if_the_spell_is_countered() {
    cr!("603.3", "601.2i", "405.5");
    ruling!(
        "Brineborn Cutthroat",
        "Brineborn Cutthroat's ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered."
    );
    supported("Brineborn Cutthroat");
    let mut t = TestGame::new(2);
    let cutthroat = t.battlefield(P0, "Brineborn Cutthroat");
    t.set_step(P1, Step::PrecombatMain);
    let spell = bolt(&mut t, P0, Entity::Player(P1));
    assert_eq!(t.stack_len(), 2);
    let top = *t.g.stack.last().unwrap();
    assert!(!matches!(t.g.obj(top).kind, ObjKind::Card), "the trigger is on top");
    counter_spell(&mut t, spell);
    t.resolve_all();
    assert_eq!(t.counters(cutthroat, "+1/+1"), 1);
    assert_eq!(t.life(P1), 20, "the spell was countered");
    // On your own turn, nothing.
    let mut t = TestGame::new(2);
    let cutthroat = t.battlefield(P0, "Brineborn Cutthroat");
    bolt(&mut t, P0, Entity::Player(P1));
    t.resolve_all();
    assert_eq!(t.counters(cutthroat, "+1/+1"), 0);
}

/// How many times `name`'s first-spell ability has resolved so far, as shown by its
/// effect: Wavebreak Hippocamp draws, Arena Trickster gets counters, Dreamstalker
/// Manticore (aimed at P3) and Mischievous Chimera (each opponent) deal damage to P3, who
/// is never bolted.
fn times_triggered(t: &TestGame, name: &str, library_before: usize, it: ObjectId) -> u32 {
    match name {
        "Wavebreak Hippocamp" => (library_before - t.library_size(P0)) as u32,
        "Arena Trickster" => t.counters(it, "+1/+1"),
        _ => (20 - t.life(P3)) as u32,
    }
}

#[test]
fn a_first_spell_trigger_can_trigger_in_each_opponents_turn() {
    cr!("603.2", "603.4", "102.2");
    ruling!(
        "Wavebreak Hippocamp",
        "If you have more than one opponent, this ability can trigger once during each of those opponents' turns."
    );
    ruling!(
        "Stinging Lionfish",
        "If you have more than one opponent, this ability can trigger once during each of those opponents' turns."
    );
    ruling!(
        "Arena Trickster",
        "If you have more than one opponent, this ability can trigger once during each of those opponents’ turns."
    );
    ruling!(
        "Dreamstalker Manticore",
        "If you have more than one opponent, this ability can trigger once during each of those opponents’ turns."
    );
    ruling!(
        "Mischievous Chimera",
        "If you have more than one opponent, this ability can trigger once during each of those opponents’ turns."
    );
    for name in [
        "Wavebreak Hippocamp",
        "Arena Trickster",
        "Dreamstalker Manticore",
        "Mischievous Chimera",
    ] {
        supported(name);
        // Four players: P1 and P2 are opponents whose turns we cast in; P3 takes the
        // damage of the damage triggers (any target / each opponent).
        let mut t = TestGame::new(4);
        let it = t.battlefield(P0, name);
        let library = t.library_size(P0);
        for (n, opp) in [P1, P2].into_iter().enumerate() {
            t.advance_to(opp, Step::PrecombatMain);
            for _ in 0..2 {
                bolt_then(&mut t, P0, Entity::Player(opp), &[Entity::Player(P3)]);
                t.resolve_all();
                t.clear_answers();
            }
            let got = times_triggered(&t, name, library, it);
            assert_eq!(got, n as u32 + 1, "{name} in {opp}'s turn");
        }
    }
    // Stinging Lionfish: "you may tap or untap target nonland permanent".
    supported("Stinging Lionfish");
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Stinging Lionfish");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let ox = t.battlefield(P2, "Grizzly Bears");
    let taps = |t: &TestGame| [t.obj_now(bears).tapped, t.obj_now(ox).tapped];
    for (opp, target) in [(P1, bears), (P2, ox)] {
        t.advance_to(opp, Step::PrecombatMain);
        for _ in 0..2 {
            // Each trigger: yes, tap (option 0).
            t.answer_yes(P0, true);
            t.answer(P0, DecisionKind::Option, Answer::Index(0));
            bolt_then(&mut t, P0, Entity::Player(opp), &[Entity::Object(target)]);
            t.resolve_all();
        }
        assert!(t.obj_now(target).tapped, "in {opp}'s turn");
        // The second spell didn't trigger: its queued answers are unused.
        let unused = t.script.lock().unwrap().queues[0].len();
        assert_eq!(unused, 3, "in {opp}'s turn");
        t.clear_answers();
    }
    assert_eq!(taps(&t), [true, true]);
}

#[test]
fn in_two_headed_giant_a_first_spell_trigger_triggers_once_per_opposing_team_turn() {
    cr!("805.4a", "603.2");
    ruling!(
        "Wavebreak Hippocamp",
        "In a Two-Headed Giant game, this ability triggers no more than once during each opposing team's turn."
    );
    ruling!(
        "Stinging Lionfish",
        "In a Two-Headed Giant game, this ability triggers no more than once during each opposing team's turn."
    );
    ruling!(
        "Arena Trickster",
        "In a Two-Headed Giant game, this ability triggers no more than once during each opposing team’s turn."
    );
    ruling!(
        "Dreamstalker Manticore",
        "In a Two-Headed Giant game, this ability triggers no more than once during each opposing team’s turn."
    );
    ruling!(
        "Mischievous Chimera",
        "In a Two-Headed Giant game, this ability triggers no more than once during each opposing team’s turn."
    );
    // Teams: P0 and P1 against P2 and P3. Arena Trickster gets a counter per trigger.
    let mut t = two_headed_giant();
    let trickster = t.battlefield(P0, "Arena Trickster");
    let hippo = t.battlefield(P0, "Wavebreak Hippocamp");
    t.set_step(P2, Step::PrecombatMain);
    let hand = t.hand_size(P0);
    for _ in 0..3 {
        bolt(&mut t, P0, Entity::Player(P2));
        t.resolve_all();
    }
    assert_eq!(t.counters(trickster, "+1/+1"), 1);
    // Three bolts left the hand; one card was drawn.
    assert_eq!(t.hand_size(P0), hand + 1);
    assert!(t.on_battlefield(hippo));
}

#[test]
fn each_opponent_loses_life_or_none_does() {
    cr!("603.5", "102.2");
    ruling!(
        "Faerie Tauntings",
        "You choose either to have each opponent lose 1 life or to have no opponent lose any life. You don't choose for each opponent individually."
    );
    supported("Faerie Tauntings");
    for yes in [true, false] {
        let mut t = TestGame::new(3);
        t.battlefield(P0, "Faerie Tauntings");
        t.set_step(P1, Step::PrecombatMain);
        let from = t.asked().len();
        t.answer_yes(P0, yes);
        bolt(&mut t, P0, Entity::Player(P1));
        t.resolve_all();
        let asked = t.asked()[from..]
            .iter()
            .filter(|(p, d)| *p == P0 && matches!(d, Decision::YesNo { .. }))
            .count();
        assert_eq!(asked, 1);
        let loss = if yes { 1 } else { 0 };
        assert_eq!(t.life(P1), 20 - 3 - loss);
        assert_eq!(t.life(P2), 20 - loss);
    }
}
