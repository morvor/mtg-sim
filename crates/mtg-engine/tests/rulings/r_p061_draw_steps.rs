//! Rulings batch P061 — draws in the beginning phase: "additional" draws from triggered
//! abilities at the beginning of the draw step (CR 504.1, 603.2), the upkeep before the
//! draw step (CR 503, 504), a Curse's targets chosen by its controller (CR 603.3d), and
//! Happily Ever After's intervening "if" clause (CR 603.4) and card types (CR 205.2a).

use crate::r_p061_common::*;
use crate::r_s02_common::{can_cast, destroy};
use mtg_engine::decision::Decision;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn additional_draws_happen_after_the_draw_step_draw_when_the_trigger_resolves() {
    cr!("504.1", "504.2", "603.3");
    ruling!(
        "Howling Mine",
        "The additional draw is separate from any other draw during your draw step. It happens when the triggered ability resolves."
    );
    ruling!(
        "Kami of the Crescent Moon",
        "The additional draw triggers just after the normal draw for the turn happens."
    );
    for name in ["Howling Mine", "Kami of the Crescent Moon"] {
        supported(name);
        let mut t = TestGame::new(2);
        t.battlefield(P0, name);
        t.set_step(P1, Step::End);
        t.advance_to(P0, Step::Draw);
        // The turn-based draw has happened; the trigger isn't on the stack yet.
        assert_eq!(t.hand_size(P0), 1, "{name}");
        assert_eq!(crate::r_p051_common::drawn_this_turn(&t, P0), 1, "{name}");
        t.settle();
        assert_eq!(t.stack_len(), 1, "{name}");
        assert_eq!(t.hand_size(P0), 1, "{name}");
        t.resolve();
        assert_eq!(t.hand_size(P0), 2, "{name}");
        assert_eq!(
            crate::r_p051_common::drawn_this_turn(&t, P0),
            2,
            "{name}: two separate draws"
        );
    }
}

#[test]
fn ghirapur_orrery_checks_the_hand_in_upkeep_before_the_draw() {
    cr!("503.1a", "504.1", "603.4");
    ruling!(
        "Ghirapur Orrery",
        "The draw step is after the upkeep step, so drawing a card as a turn-based action won't affect whether Ghirapur Orrery's second ability triggers."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ghirapur Orrery");
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    assert_eq!(t.hand_size(P0), 0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 3);
    t.advance_to(P0, Step::Draw);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 4);
}

#[test]
fn curse_of_surveillances_controller_chooses_the_targets() {
    cr!("603.3d", "115.1");
    ruling!(
        "Curse of Surveillance",
        "The controller of Curse of Surveillance chooses the targets for its triggered ability, no matter which player it is attached to. If you control Curse of Surveillance and it's attached to another player, you can (and probably should) choose yourself as a target."
    );
    let mut t = TestGame::new(3);
    attach_new(&mut t, P0, "Curse of Surveillance", pl(P1));
    t.answer_targets(P0, &[pl(P0)]);
    let from = t.asked().len();
    t.set_step(P0, Step::End);
    t.advance_to(P1, Step::Upkeep);
    let before = libraries(&t);
    t.resolve_all();
    assert_eq!(drawn_since(&t, &before), vec![1, 0, 0]);
    let targets_asked: Vec<PlayerId> = t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseTargets { .. }))
        .map(|(p, _)| *p)
        .collect();
    assert_eq!(targets_asked, vec![P0]);
}

/// Sets up P0 for Happily Ever After: the enchantment (white, an enchantment), a Forest
/// (a land), and creatures of the other four colors; in the graveyard a Lightning Bolt
/// (an instant), and — if `kindred` — Bitterblossom (a kindred enchantment) and — if
/// `battle` — Invasion of Azgol (a battle): six card types with both.
fn happily_setup(t: &mut TestGame, kindred: bool, battle: bool) {
    t.battlefield(P0, "Happily Ever After");
    t.battlefield(P0, "Forest");
    for c in [
        "Wind Drake",
        "Typhoid Rats",
        "Goblin Piker",
        "Grizzly Bears",
    ] {
        t.battlefield(P0, c);
    }
    t.graveyard(P0, "Lightning Bolt");
    if kindred {
        t.graveyard(P0, "Bitterblossom");
    }
    if battle {
        t.graveyard(P0, "Invasion of Azgol // Ashen Reaper");
    }
}

/// Runs P0's upkeep (from P1's end step) and returns whether P0 won.
fn upkeep_wins(t: &mut TestGame) -> bool {
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    t.has_lost(P1)
}

#[test]
fn happily_ever_after_counts_kindred_and_battle() {
    cr!("205.2a", "603.4", "104.2b");
    ruling!(
        "Happily Ever After",
        "The card types that could appear among your permanents and cards in your graveyard are artifact, battle, creature, enchantment, instant, kindred, land, planeswalker, and sorcery. Supertypes, such as legendary and basic, aren't card types."
    );
    for (kindred, battle, wins) in [
        (true, true, true),
        (false, true, false),
        (true, false, false),
    ] {
        let mut t = TestGame::new(2);
        happily_setup(&mut t, kindred, battle);
        assert_eq!(
            upkeep_wins(&mut t),
            wins,
            "kindred {kindred}, battle {battle}"
        );
    }
    // Supertypes don't count: a legendary basic snow land adds nothing (Snow-Covered
    // Forest and a legendary creature instead of the battle).
    let mut t = TestGame::new(2);
    happily_setup(&mut t, true, false);
    t.battlefield(P0, "Snow-Covered Forest");
    t.battlefield(P0, "Isamaru, Hound of Konda");
    assert!(!upkeep_wins(&mut t));

    ruling!(
        "Happily Ever After",
        "You could have one permanent that's all five colors, or the five colors could be spread out among any number of permanents."
    );
    // Transguild Courier (all five colors, an artifact creature) alone.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Happily Ever After");
    t.battlefield(P0, "Forest");
    t.battlefield(P0, "Transguild Courier");
    t.graveyard(P0, "Lightning Bolt");
    t.graveyard(P0, "Divination");
    assert!(upkeep_wins(&mut t));
}

#[test]
fn happily_ever_after_checks_everything_again_on_resolution() {
    cr!("603.4");
    ruling!(
        "Happily Ever After",
        "The second triggered ability of Happily Ever After has one very long intervening \"if\" clause. This means that all of the conditions must be true as your upkeep begins—you must control permanents with all five colors, your permanents and graveyard must contain six or more card types, and your life total must be high enough—or the ability won't trigger at all. If they're not all true as the ability resolves, you won't win the game."
    );
    // Life below the starting total as the upkeep begins: it doesn't trigger.
    let mut t = TestGame::new(2);
    happily_setup(&mut t, true, true);
    t.g.players[0].life = 19;
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert_eq!(t.stack_len(), 0);
    // No longer true on resolution (a color gone, or life lost): no win.
    for lose_life in [false, true] {
        let mut t = TestGame::new(2);
        happily_setup(&mut t, true, true);
        t.set_step(P1, Step::End);
        t.advance_to(P0, Step::Upkeep);
        t.settle();
        assert_eq!(t.stack_len(), 1);
        if lose_life {
            cast_resolve(&mut t, P1, "Lightning Bolt", &[pl(P0)]);
        } else {
            let drake = t.named_on_battlefield("Wind Drake")[0];
            destroy(&mut t, drake);
        }
        t.resolve_all();
        assert!(!t.has_lost(P1), "life lost: {lose_life}");
    }
}

#[test]
fn innocuous_researchers_restriction_outlasts_it() {
    cr!("608.2", "611.2a", "117.1");
    ruling!(
        "Innocuous Researcher",
        "There’s no window for you to cast spells after untapping all lands you control but before the “you can’t cast spells” portion of Innocuous Researcher’s last ability takes effect. Once that ability resolves, it doesn’t matter whether Innocuous Researcher stays on the battlefield; you’ll still be unable to cast spells until your next turn begins."
    );
    let mut t = TestGame::new(2);
    let researcher = t.battlefield(P0, "Innocuous Researcher");
    let lands = t.lands(P0, "Mountain", 2);
    for l in &lands {
        t.g.tap(*l);
    }
    let bolt = t.hand(P0, "Lightning Bolt");
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::End);
    t.settle();
    let from = t.asked().len();
    t.resolve();
    assert!(lands.iter().all(|l| !t.obj_now(*l).tapped), "untapped");
    // No priority during the resolution.
    assert!(!t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::Priority { .. })));
    assert!(!can_cast(&mut t, P0, bolt, CastMethod::Normal));
    destroy(&mut t, researcher);
    assert!(!can_cast(&mut t, P0, bolt, CastMethod::Normal));
    t.advance_to(P1, Step::Upkeep);
    assert!(
        !can_cast(&mut t, P0, bolt, CastMethod::Normal),
        "on P1's turn"
    );
    t.advance_to(P0, Step::Upkeep);
    assert!(
        can_cast(&mut t, P0, bolt, CastMethod::Normal),
        "P0's next turn"
    );
}
