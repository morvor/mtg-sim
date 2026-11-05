//! CR 104.4b, 732.5: a loop of mandatory triggered abilities is still a loop of mandatory
//! actions when each time around a player chooses the target the looping ability calls
//! for: the game is a draw. Unless another target, or another order of the triggered
//! abilities, would end the loop: then the loop has a way to stop, and isn't a draw.

use crate::r105_util::card_from_text;
use mtg_engine::decision::PassiveAgent;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn a_loop_of_targeted_triggered_abilities_is_a_draw() {
    cr!("104.4b", "732.5");
    interp!("mandatory-loop-detection");
    // Faceless Devourer: "When ~ enters, exile another target creature with shadow.
    // When ~ leaves the battlefield, return the exiled card to the battlefield under its
    // owner's control."
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 6);
    t.battlefield(P0, "Faceless Devourer");
    let second = t.hand(P0, "Faceless Devourer");
    t.cast(P0, second).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Faceless Devourer").len(), 1);
    // A third one exiles the second, which returns the first, which exiles the third,
    // which returns the second, ... — each time with only one creature to target.
    let third = t.hand(P0, "Faceless Devourer");
    t.cast(P0, third).go();
    t.g.run_until(4000, |g| g.is_over());
    assert_eq!(t.g.result, Some(GameResult::Draw));
}

#[test]
fn a_loop_whose_trigger_has_several_targets_is_a_draw_after_a_forced_choice() {
    cr!("104.4b", "732.4", "732.5");
    interp!("mandatory-loop-detection");
    // Five Faceless Devourers. Each one on the battlefield holds another exiled, so
    // whichever the entering one exiles returns its own, which must exile another: the
    // chain can't stop, and each time around there are two targets to choose from.
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 9);
    let a = t.battlefield(P0, "Faceless Devourer");
    // B's trigger has a single possible target (A): a choice with one answer, which the
    // loop detection counts apart from the other decisions.
    let b = t.hand(P0, "Faceless Devourer");
    t.cast(P0, b).go();
    t.resolve_all();
    assert!(!t.on_battlefield(a));
    let c = t.battlefield(P0, "Faceless Devourer");
    let d = t.hand(P0, "Faceless Devourer");
    t.cast(P0, d).go();
    // D exiles C (B, holding A, is the other candidate).
    t.answer_targets(P0, &[Entity::Object(c)]);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Faceless Devourer").len(), 2);
    // E's trigger exiles B or D; either returns the card it holds, whose trigger has two
    // targets again, and so on.
    let e = t.hand(P0, "Faceless Devourer");
    let before = t.asked().len();
    t.cast(P0, e).go();
    t.g.run_until(4000, |g| g.is_over());
    assert_eq!(t.g.result, Some(GameResult::Draw));
    let two_targets = t.asked()[before..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseTargets { candidates, .. } if candidates.len() == 2))
        .count();
    assert!(
        two_targets >= 3,
        "the looping trigger had two targets each time"
    );
}

#[test]
fn a_hostage_taker_chain_with_two_targets_each_time_is_a_draw() {
    cr!("104.4b", "732.4", "732.5");
    interp!("mandatory-loop-detection");
    // The every-card sweep's slow Hostage Taker game: "When Hostage Taker enters, exile
    // another target artifact or creature until Hostage Taker leaves the battlefield."
    // Every Hostage Taker on the battlefield holds another, so whichever one the entering
    // one exiles returns its own, which must exile another.
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 9);
    t.lands(P0, "Swamp", 9);
    let a = t.battlefield(P0, "Hostage Taker");
    // A single possible target (A) first.
    let b = t.hand(P0, "Hostage Taker");
    t.cast(P0, b).go();
    t.resolve_all();
    assert!(!t.on_battlefield(a));
    let c = t.battlefield(P0, "Hostage Taker");
    let d = t.hand(P0, "Hostage Taker");
    t.cast(P0, d).go();
    t.answer_targets(P0, &[Entity::Object(c)]);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Hostage Taker").len(), 2);
    let e = t.hand(P0, "Hostage Taker");
    let before = t.asked().len();
    t.cast(P0, e).go();
    t.g.run_until(4000, |g| g.is_over());
    assert_eq!(t.g.result, Some(GameResult::Draw));
    let two_targets = t.asked()[before..]
        .iter()
        .filter(|(_, d)| {
            matches!(d, Decision::ChooseTargets { candidates, .. } if candidates.len() == 2)
        })
        .count();
    assert!(
        two_targets >= 3,
        "the looping trigger had two targets each time"
    );
}

/// An agent that chooses targets with the given name when there are enough of them, and
/// orders the triggered abilities of the source with that name last (on top of the stack,
/// so they resolve first); otherwise it passes and lets the engine choose.
struct Prefers(&'static str);

impl Agent for Prefers {
    fn name(&self) -> &str {
        "prefers"
    }
    fn decide(&mut self, g: &Game, p: PlayerId, d: &Decision) -> Answer {
        match d {
            Decision::ChooseTargets {
                candidates, min, ..
            } => {
                let v: Vec<Entity> = candidates
                    .iter()
                    .copied()
                    .filter(|e| matches!(e, Entity::Object(o) if g.obj(*o).chars.name == self.0))
                    .take(*min as usize)
                    .collect();
                if v.len() == *min as usize {
                    Answer::Entities(v)
                } else {
                    Answer::Default
                }
            }
            Decision::Order { items, .. } => {
                let mine = |i: &usize| items[*i].starts_with(self.0);
                let mut v: Vec<usize> = (0..items.len()).filter(|i| !mine(i)).collect();
                v.extend((0..items.len()).filter(mine));
                Answer::Indices(v)
            }
            _ => PassiveAgent.decide(g, p, d),
        }
    }
}

fn agents(p0: &'static str) -> Vec<Box<dyn Agent>> {
    vec![Box::new(Prefers(p0)), Box::new(PassiveAgent)]
}

/// Runs the game until it's over or leaves P0's first precombat main phase, where the loop
/// keeps it while it goes on.
fn run_while_looping(t: &mut TestGame, max: usize) {
    t.g.run_until(max, |g| {
        g.is_over() || g.turn.number != 1 || g.turn.step != Step::PrecombatMain
    });
}

fn still_looping(t: &TestGame) -> bool {
    t.g.result.is_none() && t.g.turn.number == 1 && t.g.turn.step == Step::PrecombatMain
}

#[test]
fn a_chain_of_single_target_choices_that_changes_the_game_state_is_not_a_draw() {
    cr!("104.4b");
    interp!("mandatory-loop-detection");
    interp!("growing-loops-run-to-the-limit");
    // The three-Devourer chain of the first test, each time with one target to choose,
    // while P1's Soul Warden ("Whenever another creature enters, you gain 1 life.") gains
    // life each time around: the game state never repeats, so the chain isn't recognized
    // as a loop.
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 6);
    t.battlefield(P1, "Soul Warden");
    t.battlefield(P0, "Faceless Devourer");
    let second = t.hand(P0, "Faceless Devourer");
    t.cast(P0, second).go();
    t.resolve_all();
    let third = t.hand(P0, "Faceless Devourer");
    t.cast(P0, third).go();
    run_while_looping(&mut t, 3000);
    assert!(still_looping(&t), "not a draw: {:?}", t.g.result);
    assert!(t.life(P1) > 100, "P1 gained life each time around");
    let single = t
        .asked()
        .iter()
        .filter(|(_, d)| {
            matches!(d, Decision::ChooseTargets { candidates, .. } if candidates.len() == 1)
        })
        .count();
    assert!(single > 50, "one target to choose each time around");
}

#[test]
fn a_loop_another_target_would_end_is_not_a_draw() {
    cr!("104.4b", "732.5");
    interp!("mandatory-loop-detection");
    // The Faceless Devourer chain above, and a creature with shadow that isn't a Devourer:
    // exiling it instead ends the chain. P0 keeps choosing Devourers, but could stop the
    // loop any time around, so it isn't a loop of mandatory actions "with no way to stop"
    // (a player can be made to take an action an object involved in the loop calls for,
    // such as choosing another target, CR 732.5).
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 9);
    let a = t.battlefield(P0, "Faceless Devourer");
    // A single possible target first.
    let b = t.hand(P0, "Faceless Devourer");
    t.cast(P0, b).go();
    t.resolve_all();
    assert!(!t.on_battlefield(a));
    let c = t.battlefield(P0, "Faceless Devourer");
    let d = t.hand(P0, "Faceless Devourer");
    t.cast(P0, d).go();
    t.answer_targets(P0, &[Entity::Object(c)]);
    t.resolve_all();
    let soltari = t.battlefield(P1, "Soltari Foot Soldier");
    let e = t.hand(P0, "Faceless Devourer");
    t.g.set_agents(agents("Faceless Devourer"));
    t.cast(P0, e).go();
    run_while_looping(&mut t, 3000);
    assert!(still_looping(&t), "not a draw: {:?}", t.g.result);
    assert!(t.on_battlefield(soltari));
    // Choosing the Soltari Foot Soldier ends the chain, and the game goes on.
    t.g.set_agents(agents("Soltari Foot Soldier"));
    run_while_looping(&mut t, 200);
    assert_eq!(t.g.result, None);
    assert!(!still_looping(&t));
    assert!(!t.on_battlefield(soltari));
    assert_eq!(t.named_on_battlefield("Faceless Devourer").len(), 3);
}

#[test]
fn a_loop_another_order_of_its_triggers_would_end_is_not_a_draw() {
    cr!("104.4b", "732.5", "603.3b");
    interp!("mandatory-loop-detection");
    // Each time Hollow Husk dies, both enchantments trigger. With the return on top of the
    // stack, the Husk comes back and then gets a -1/-1 counter and dies again; with the
    // counters on top, they find no creature and the Husk comes back to stay. P0 keeps
    // choosing the first order, but could stop the loop any time around.
    let mut t = TestGame::new(2);
    let ret = card_from_text(
        "Endless Return",
        "",
        "Enchantment",
        None,
        "Whenever a creature dies, return that card to the battlefield under its owner's control.",
    );
    let wither = card_from_text(
        "Withering Field",
        "",
        "Enchantment",
        None,
        "Whenever a creature dies, put a -1/-1 counter on each creature.",
    );
    let husk = card_from_text("Hollow Husk", "", "Creature — Construct", Some((1, 1)), "");
    t.custom(P0, ret, Zone::Battlefield);
    t.custom(P0, wither, Zone::Battlefield);
    let h = t.custom(P0, husk, Zone::Battlefield);
    t.lands(P0, "Mountain", 1);
    let shock = t.hand(P0, "Shock");
    t.cast(P0, shock).target(Entity::Object(h)).go();
    t.g.set_agents(agents("Endless Return"));
    run_while_looping(&mut t, 3000);
    assert!(still_looping(&t), "not a draw: {:?}", t.g.result);
    // The other order ends it: the Husk comes back to stay, and the game goes on.
    t.g.set_agents(agents("Withering Field"));
    run_while_looping(&mut t, 200);
    assert_eq!(t.g.result, None);
    assert!(!still_looping(&t));
    assert_eq!(t.named_on_battlefield("Hollow Husk").len(), 1);
}
