//! Rulings batch P224 — surveil (CR 701.25): spells that surveil after another effect,
//! illegal targets, reflexive triggers, and what's on the battlefield while you surveil.

use crate::r_p224_common::*;
use crate::r_s01_common::*;
use crate::r_s02_common::{create_token, destroy};
use crate::r_s07_common::resolved;
use crate::r_s11_common::{empty_library, triggered_from};
use crate::r_s12_common::morph;
use crate::r_s15_common::library_top_n;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::*;

fn is_surveil(d: &Decision) -> bool {
    matches!(d, Decision::Surveil { .. })
}

/// Number of surveil decisions asked since `from`.
fn surveils_since(t: &TestGame, from: usize) -> usize {
    t.asked()[from..]
        .iter()
        .filter(|(_, d)| is_surveil(d))
        .count()
}

#[test]
fn sinister_sabotage_targets_an_uncounterable_spell_and_still_surveils() {
    cr!("701.6b", "608.2c", "701.25a");
    ruling!(
        "Sinister Sabotage",
        "A spell that can't be countered is a legal target for Sinister Sabotage. The spell won't be countered when Sinister Sabotage resolves, but you'll still surveil 1."
    );
    supported("Sinister Sabotage");
    supported("Abrupt Decay");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    // Abrupt Decay: "This spell can't be countered. Destroy target nonland permanent with
    // mana value 3 or less."
    t.lands(P1, "Swamp", 1);
    t.lands(P1, "Forest", 1);
    let decay = t.hand(P1, "Abrupt Decay");
    let decay = t.cast(P1, decay).target(bears).go();
    // Sinister Sabotage: "Counter target spell. Surveil 1."
    let top = t.library_top(P0, "Mountain");
    t.answer(P0, DecisionKind::Surveil, Answer::Split(vec![], vec![top]));
    t.lands(P0, "Island", 3);
    let ss = t.hand(P0, "Sinister Sabotage");
    let from = t.asked().len();
    let ss = t.cast(P0, ss).target(decay).go();
    t.resolve();
    assert!(resolved(&t, ss));
    assert_eq!(surveils_since(&t, from), 1);
    assert!(t.in_graveyard(P0, "Mountain"));
    // Abrupt Decay wasn't countered.
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert!(t.in_graveyard(P1, "Abrupt Decay"));
}

#[test]
fn creatures_mephitic_vapors_kills_are_still_there_while_you_surveil() {
    cr!("608.2c", "704.3", "603.3", "701.25a");
    ruling!(
        "Mephitic Vapors",
        "Creatures that are going to die after getting -1/-1 will still be on the battlefield while you surveil."
    );
    supported("Mephitic Vapors");
    supported("Dimir Spybug");
    // Dimir Spybug (1/1): "Whenever you surveil, put a +1/+1 counter on this creature."
    let mut t = TestGame::new(2);
    let spybug = t.battlefield(P0, "Dimir Spybug");
    let seen = watch(&mut t, P0, is_surveil, |g| {
        g.permanents()
            .filter(|o| o.chars.name == "Dimir Spybug")
            .map(|o| (o.power(), o.toughness()))
            .collect::<Vec<_>>()
    });
    // Mephitic Vapors: "All creatures get -1/-1 until end of turn. Surveil 2."
    t.lands(P0, "Swamp", 3);
    let mv = t.hand(P0, "Mephitic Vapors");
    t.cast(P0, mv).go();
    t.resolve();
    // While P0 surveilled, the 0/0 Spybug was still on the battlefield.
    assert_eq!(*seen.lock().unwrap(), vec![vec![(0, 0)]]);
    // Its ability triggered, but it died before the ability resolved.
    assert_eq!(triggered_from(&t, spybug), 1);
    assert!(!t.on_battlefield(spybug));
    assert!(t.in_graveyard(P0, "Dimir Spybug"));
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Dimir Spybug"));
}

#[test]
fn destroy_and_surveil_spells_can_target_an_indestructible_creature() {
    cr!("702.12b", "608.2c", "701.25a");
    ruling!(
        "Deadly Visit",
        "Deadly Visit can target a creature that's indestructible. It won't be destroyed, but you'll surveil."
    );
    ruling!(
        "Price of Fame",
        "Price of Fame can target a creature that's indestructible. It won't be destroyed, but you'll surveil."
    );
    supported("Deadly Visit");
    supported("Price of Fame");
    supported("Darksteel Myr");
    for (name, swamps) in [("Deadly Visit", 5), ("Price of Fame", 4)] {
        let mut t = TestGame::new(2);
        let myr = t.battlefield(P1, "Darksteel Myr");
        t.lands(P0, "Swamp", swamps);
        let c = t.hand(P0, name);
        let from = t.asked().len();
        t.cast(P0, c).target(myr).go();
        t.resolve_all();
        assert!(t.on_battlefield(myr), "{name}");
        assert_eq!(surveils_since(&t, from), 1, "{name}");
    }
}

#[test]
fn dream_eaters_reflexive_trigger_targets_after_surveilling_even_with_a_short_library() {
    cr!("603.12", "701.25a", "701.25c");
    ruling!(
        "Dream Eater",
        "Dream Eater's reflexive triggered ability triggers even if you have fewer than four cards in your library to surveil."
    );
    ruling!(
        "Dream Eater",
        "Dream Eater's triggered ability goes on the stack without a target. While that ability is resolving, after you've surveilled, the reflexive triggered ability triggers and you pick a target nonland permanent to be returned to its owner's hand."
    );
    supported("Dream Eater");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    empty_library(&mut t, P0);
    stack_library(&mut t, P0, &["Swamp", "Island"]);
    // Dream Eater: "When this creature enters, surveil 4. When you do, you may return
    // target nonland permanent an opponent controls to its owner's hand."
    let from = t.asked().len();
    t.enter(P0, "Dream Eater");
    t.settle();
    // The enters trigger is on the stack, with no target chosen yet.
    assert_eq!(t.stack_len(), 1);
    let top = *t.g.stack.last().unwrap();
    assert!(t
        .obj(top)
        .stack
        .as_ref()
        .unwrap()
        .chosen
        .iter()
        .all(|c| c.targets.iter().all(|s| s.is_empty())));
    assert!(!t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseTargets { .. })));
    t.answer_yes(P0, true);
    t.resolve_all();
    // P0 surveilled (two cards), then chose the reflexive trigger's target.
    let asked = &t.asked()[from..];
    let surveil = asked.iter().position(|(_, d)| is_surveil(d)).unwrap();
    let target = asked
        .iter()
        .position(|(_, d)| matches!(d, Decision::ChooseTargets { .. }))
        .expect("no target chosen");
    assert!(surveil < target);
    assert!(t.in_hand(P1, "Grizzly Bears"));
    assert!(!t.on_battlefield(bears));
}

/// P0 casts `name` at an Ornithopter P1 controls (an artifact creature with flying, a
/// nonland permanent); if `illegal`, the Ornithopter is destroyed before the spell
/// resolves. Returns whether P0 surveilled and whether the spell resolved.
fn surveil_spell_at_ornithopter(name: &str, illegal: bool) -> (bool, bool) {
    let mut t = TestGame::new(2);
    // Failed Fording surveils only if P0 controls a Desert.
    t.battlefield(P0, "Desert of the True");
    let thopter = t.battlefield(P1, "Ornithopter");
    give_mana_for(&mut t, P0, name);
    let c = t.hand(P0, name);
    let from = t.asked().len();
    let spell = t.cast(P0, c).target(thopter).go();
    if illegal {
        destroy(&mut t, thopter);
    }
    t.resolve_all();
    (surveils_since(&t, from) > 0, resolved(&t, spell))
}

#[test]
fn a_targeted_surveil_spell_with_an_illegal_target_doesnt_surveil() {
    cr!("608.2b", "701.25a");
    ruling!(
        "Consuming Ashes",
        "If the target creature is an illegal target as Consuming Ashes tries to resolve, it won’t resolve and none of its effects will happen. You won’t surveil."
    );
    ruling!(
        "Unsubtle Mockery",
        "If the target creature is an illegal target as Unsubtle Mockery tries to resolve, it won't resolve and none of its effects will happen. You won't surveil."
    );
    ruling!(
        "Unauthorized Exit",
        "If the target nonland permanent is an illegal target as Unauthorized Exit tries to resolve, it won't resolve and none of its effects will happen. You won't surveil 1."
    );
    ruling!(
        "Vanish from Sight",
        "If the target nonland permanent is an illegal target when Vanish from Sight tries to resolve, it won't resolve and none of its effects will happen. You won't surveil."
    );
    ruling!(
        "Failed Fording",
        "If the target permanent is an illegal target as Failed Fording tries to resolve, it won’t resolve and none of its effects will happen. You won’t surveil."
    );
    ruling!(
        "Lost in Space",
        "If the target permanent is an illegal target as Lost in Space tries to resolve, it won’t resolve and none of its effects will happen. You won’t surveil."
    );
    ruling!(
        "Shattered Wings",
        "If the target permanent is an illegal target as Shattered Wings tries to resolve, it won’t resolve and none of its effects will happen. You won’t surveil."
    );
    ruling!(
        "Banishing Betrayal",
        "If the target permanent is an illegal target when Banishing Betrayal tries to resolve, it won't resolve and none of its effects will happen. You won't surveil."
    );
    for name in [
        "Consuming Ashes",
        "Unsubtle Mockery",
        "Unauthorized Exit",
        "Vanish from Sight",
        "Failed Fording",
        "Lost in Space",
        "Shattered Wings",
        "Banishing Betrayal",
    ] {
        supported(name);
        assert_eq!(
            surveil_spell_at_ornithopter(name, false),
            (true, true),
            "{name}"
        );
        assert_eq!(
            surveil_spell_at_ornithopter(name, true),
            (false, false),
            "{name}"
        );
    }
}

#[test]
fn eloises_surveil_trigger_resolves_before_the_clues_draw() {
    cr!("602.2b", "603.3", "701.25a");
    ruling!(
        "Eloise, Nephalia Sleuth",
        "If you sacrifice a Clue token, Eloise's last ability will trigger in response. You'll surveil before drawing the card."
    );
    supported("Eloise, Nephalia Sleuth");
    let mut t = TestGame::new(2);
    // Eloise: "Whenever you sacrifice a token, surveil 1."
    t.battlefield(P0, "Eloise, Nephalia Sleuth");
    let clue = create_token(&mut t, P0, "Clue");
    let cards = stack_library(&mut t, P0, &["Swamp", "Island"]);
    t.answer(
        P0,
        DecisionKind::Surveil,
        Answer::Split(vec![], vec![cards[0]]),
    );
    t.lands(P0, "Island", 2);
    t.activate(P0, clue, 0, &[]).unwrap();
    t.settle();
    // The surveil trigger is above the Clue's ability.
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    // The Swamp was surveilled into the graveyard, then the Island drawn.
    assert!(t.in_graveyard(P0, "Swamp"));
    assert!(t.in_hand(P0, "Island"));
}

#[test]
fn concoct_returns_a_creature_card_just_surveilled_with_no_actions_in_between() {
    cr!("608.2c", "117.1", "701.25a");
    ruling!(
        "Connive // Concoct",
        "No player may take actions between the time you surveil and the time you return a creature card to the battlefield."
    );
    ruling!(
        "Connive // Concoct",
        "The creature card you return with Concoct may be one that you just surveilled into your graveyard."
    );
    supported("Connive // Concoct");
    let mut t = TestGame::new(2);
    let cards = stack_library(&mut t, P0, &["Hill Giant", "Swamp", "Island"]);
    // Concoct ({3}{U}{B}): "Surveil 3, then return a creature card from your graveyard to
    // the battlefield." The Hill Giant goes into the graveyard and comes back.
    t.answer(
        P0,
        DecisionKind::Surveil,
        Answer::Split(vec![cards[1], cards[2]], vec![cards[0]]),
    );
    t.answer_choose(P0, &[Entity::Object(cards[0])]);
    t.lands(P0, "Island", 4);
    t.lands(P0, "Swamp", 1);
    let c = t.hand(P0, "Connive // Concoct");
    t.cast(P0, c).method(CastMethod::Half(1)).go();
    let from = t.asked().len();
    t.resolve();
    let asked = &t.asked()[from..];
    let surveil = asked.iter().position(|(_, d)| is_surveil(d)).unwrap();
    // No player got priority between the surveil and the end of the resolution.
    assert!(!asked[surveil..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::Priority { .. })));
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
    assert_eq!(library_top_n(&t, P0, 2), vec![cards[1], cards[2]]);
}

#[test]
fn snarling_gorehound_checks_power_only_as_the_creature_enters() {
    cr!("603.10a", "603.6a", "701.25a");
    ruling!(
        "Snarling Gorehound",
        "Snarling Gorehound's ability checks the power of a creature only at the moment it enters the battlefield. If it enters with counters, those counters are included. If that creature's power is 2 or less when it enters the battlefield but becomes greater than 2 after the ability triggers, you'll still surveil 1."
    );
    supported("Snarling Gorehound");
    supported("Hangarback Walker");
    // Snarling Gorehound: "Whenever another creature you control with power 2 or less
    // enters, surveil 1." Hangarback Walker (0/0) enters with X +1/+1 counters.
    let hangarback = |x: i64| {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Snarling Gorehound");
        t.lands(P0, "Wastes", 2 * x as usize);
        let c = t.hand(P0, "Hangarback Walker");
        let from = t.asked().len();
        t.cast(P0, c).x(x).go();
        t.resolve_all();
        surveils_since(&t, from)
    };
    assert_eq!(hangarback(2), 1);
    assert_eq!(hangarback(3), 0);
    // A 2/2 enters; with the ability on the stack, it gets +3/+3: P0 still surveils.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Snarling Gorehound");
    let from = t.asked().len();
    let bears = t.enter(P0, "Grizzly Bears");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.lands(P0, "Forest", 1);
    let gg = t.hand(P0, "Giant Growth");
    t.cast(P0, gg).target(bears).go();
    t.resolve();
    assert_eq!(t.pt(bears), (5, 5));
    t.resolve_all();
    assert_eq!(surveils_since(&t, from), 1);
}

#[test]
fn consuming_ashes_uses_the_exiled_creatures_last_known_mana_value() {
    cr!("608.2h", "708.2", "701.25a");
    ruling!(
        "Consuming Ashes",
        "Use the mana value of the creature as it last existed on the battlefield to determine whether or not you surveil."
    );
    supported("Exalted Angel");
    // Consuming Ashes: "Exile target creature. If it had mana value 3 or less, surveil 2."
    // A face-down Exalted Angel has mana value 0 on the battlefield (6 in exile).
    let ashes = |face_down: bool| {
        let mut t = TestGame::new(2);
        let angel = if face_down {
            morph(&mut t, P0, "Exalted Angel")
        } else {
            t.battlefield(P0, "Exalted Angel")
        };
        t.lands(P1, "Swamp", 4);
        let c = t.hand(P1, "Consuming Ashes");
        let from = t.asked().len();
        t.cast(P1, c).target(angel).go();
        t.resolve_all();
        assert!(t.in_exile("Exalted Angel"));
        surveils_since(&t, from)
    };
    assert_eq!(ashes(true), 1);
    assert_eq!(ashes(false), 0);
}

#[test]
fn thought_erasure_surveils_even_if_nothing_is_discarded() {
    cr!("608.2c", "701.25a");
    ruling!(
        "Thought Erasure",
        "You surveil 1 even if the opponent doesn't discard a card, perhaps because they had no cards in hand at all."
    );
    supported("Thought Erasure");
    let mut t = TestGame::new(2);
    assert_eq!(t.hand_size(P1), 0);
    t.lands(P0, "Island", 1);
    t.lands(P0, "Swamp", 1);
    let c = t.hand(P0, "Thought Erasure");
    let from = t.asked().len();
    t.cast(P0, c).target(P1).go();
    t.resolve_all();
    assert_eq!(surveils_since(&t, from), 1);
}

#[test]
fn plan_the_heist_surveils_only_with_an_empty_hand_but_always_draws() {
    cr!("608.2c", "701.25a");
    ruling!(
        "Plan the Heist",
        "You’ll only surveil 3 if you have no cards in hand when Plan the Heist resolves, but you’ll still draw three cards either way."
    );
    supported("Plan the Heist");
    // Plan the Heist: "Surveil 3 if you have no cards in hand. Then draw three cards."
    let heist = |extra_cards: usize| {
        let mut t = TestGame::new(2);
        for _ in 0..extra_cards {
            t.hand(P0, "Forest");
        }
        t.lands(P0, "Island", 4);
        let c = t.hand(P0, "Plan the Heist");
        let from = t.asked().len();
        t.cast(P0, c).go();
        t.resolve_all();
        (surveils_since(&t, from), t.hand_size(P0) - extra_cards)
    };
    assert_eq!(heist(0), (1, 3));
    assert_eq!(heist(1), (0, 3));
}
