//! Rulings batch P063 — "Play with the top card of your library revealed. You may play
//! lands and cast spells from the top of your library." (Future Sight, Magus of the
//! Future, Garruk's Horde): the card stays in the library (CR 401), the normal timing
//! rules and costs apply (CR 601.2, 305.2, 307.1), and the new top card isn't revealed
//! until the spell or land play is finished (CR 401.5).

use crate::r_p062_common::is_tapped;
use crate::r_p063_common::*;
use crate::r_s01_common::{supported, watch};
use crate::r_s02_common::can_play_land;
use crate::r_s04_common::can_cycle;
use crate::r_s08_common::legal_cast_methods;
use crate::r_s21_common::castable;
use mtg_engine::ability::{Effect, Filter, PlayerRef, Value};
use mtg_engine::decision::{Action, Decision, SpecialAction};
use mtg_engine::facedown::can_look_at;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// The cards in P0's library P1 can see.
fn visible_to_p1(g: &mtg_engine::game::Game) -> Vec<ObjectId> {
    g.player(P0)
        .library
        .iter()
        .copied()
        .filter(|c| can_look_at(g, P1, *c))
        .collect()
}

#[test]
fn magus_of_the_future_reveals_the_new_top_card_only_after_casting_or_playing() {
    cr!("401.5", "601.2", "305.1");
    ruling!(
        "Magus of the Future",
        "If the top card of your library changes while you're casting a spell, playing a land, or activating an ability, you don't reveal the new top card until you finish doing so."
    );
    supported("Magus of the Future");
    // Casting Lightning Bolt from the top: while its target is chosen, the card below
    // (Hill Giant) isn't revealed to P1.
    let mut t = TestGame::new(2);
    let next = t.library_top(P0, "Hill Giant");
    let bolt = t.library_top(P0, "Lightning Bolt");
    t.battlefield(P0, "Magus of the Future");
    t.lands(P0, "Mountain", 1);
    let seen = watch(
        &mut t,
        P0,
        |d| matches!(d, Decision::ChooseTargets { .. }),
        visible_to_p1,
    );
    t.cast(P0, bolt).target(Entity::Player(P1)).go();
    assert_eq!(seen.lock().unwrap().len(), 1);
    assert!(!seen.lock().unwrap()[0].contains(&next));
    assert!(can_look_at(&t.g, P1, next), "revealed once the spell is cast");
    t.resolve();
    assert_eq!(t.life(P1), 17);
    // Playing Steam Vents from the top ("As this land enters, you may pay 2 life"): the
    // next card isn't revealed while that replacement effect is handled.
    let mut t = TestGame::new(2);
    let next = t.library_top(P0, "Hill Giant");
    let vents = t.library_top(P0, "Steam Vents");
    t.battlefield(P0, "Magus of the Future");
    let seen = watch(
        &mut t,
        P0,
        |d| !matches!(d, Decision::Priority { .. }),
        visible_to_p1,
    );
    t.play_land(P0, vents).unwrap();
    assert!(!seen.lock().unwrap().is_empty());
    assert!(seen.lock().unwrap().iter().all(|v| !v.contains(&next)));
    assert!(t.on_battlefield(vents));
    assert!(can_look_at(&t.g, P1, next));
}

#[test]
fn garruks_horde_cant_play_dryad_arbor_from_the_top() {
    cr!("305.9", "601.3");
    ruling!(
        "Garruk's Horde",
        "If the top card of your library is Dryad Arbor (the only card that’s both a creature and a land), you can’t play it this way. Dryad Arbor can’t be cast as a spell."
    );
    supported("Garruk's Horde");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Garruk's Horde");
    let arbor = t.library_top(P0, "Dryad Arbor");
    assert!(!can_play_land(&mut t, P0, arbor));
    assert!(!castable(&mut t, P0, arbor));
    // Control: a creature card on top can be cast.
    let bears = t.library_top(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 2);
    assert!(castable(&mut t, P0, bears));
}

#[test]
fn future_sight_land_on_top_needs_an_available_land_play() {
    cr!("305.2", "305.1");
    ruling!(
        "Future Sight",
        "If the top card of your library is a land, you may play that land only if you have any available land plays."
    );
    supported("Future Sight");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Future Sight");
    let forest = t.hand(P0, "Forest");
    let top = t.library_top(P0, "Mountain");
    assert!(can_play_land(&mut t, P0, top));
    t.play_land(P0, forest).unwrap();
    assert!(!can_play_land(&mut t, P0, top), "no land plays left");
}

#[test]
fn magus_of_the_future_lands_only_in_a_main_phase_with_an_empty_stack() {
    cr!("305.1", "116.2a");
    ruling!(
        "Magus of the Future",
        "Magus of the Future doesn't change when you can play lands. You can do so only during your main phase when you have priority and the stack is empty."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Magus of the Future");
    let top = t.library_top(P0, "Mountain");
    assert!(can_play_land(&mut t, P0, top));
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!can_play_land(&mut t, P0, top), "not in combat");
    t.set_step(P0, Step::PostcombatMain);
    // A spell on the stack.
    pool(&mut t, P0, ManaType::R, 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(Entity::Player(P1)).go();
    assert!(!can_play_land(&mut t, P0, top), "not with a spell on the stack");
    t.resolve_all();
    assert!(can_play_land(&mut t, P0, top));
    // Not on an opponent's turn.
    t.set_step(P1, Step::PrecombatMain);
    assert!(!can_play_land(&mut t, P0, top));
}

#[test]
fn garruks_horde_flash_creature_on_top_can_be_cast_at_instant_speed() {
    cr!("702.8a", "601.3");
    ruling!(
        "Garruk's Horde",
        "Normally, Garruk’s Horde allows you to cast the top card of your library if it’s a creature card, it’s your main phase, and the stack is empty. If that creature card has flash, you’ll be able to cast it at the time you could cast an instant, even on an opponent’s turn."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Garruk's Horde");
    let viper = t.library_top(P0, "Ambush Viper");
    t.lands(P0, "Forest", 2);
    t.set_step(P1, Step::BeginningOfCombat);
    assert!(castable(&mut t, P0, viper));
    t.g.turn.priority = Some(P0);
    t.cast(P0, viper).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Ambush Viper").len(), 1);
    // Control: a creature card without flash can't be cast then.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Garruk's Horde");
    let bears = t.library_top(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 2);
    t.set_step(P1, Step::BeginningOfCombat);
    assert!(!castable(&mut t, P0, bears));
}

#[test]
fn magus_of_the_future_top_card_cant_be_discarded_or_suspended() {
    cr!("401.1", "702.62a", "701.9a");
    ruling!(
        "Magus of the Future",
        "The top card of your library is still in your library and not in your hand. You can't discard or suspend cards from the top of your library, for example."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Magus of the Future");
    let bolt = t.library_top(P0, "Rift Bolt");
    t.lands(P0, "Mountain", 3);
    let methods = legal_cast_methods(&mut t, P0, bolt);
    assert!(methods.contains(&CastMethod::Normal), "it can be cast");
    let actions = crate::r_s08_common::actions_of(&mut t, P0);
    assert!(
        !actions.iter().any(|a| matches!(
            a,
            Action::Special(SpecialAction::Suspend { card }) if *card == t.g.current(bolt)
        )),
        "but not suspended"
    );
    // Control: from hand it can be suspended.
    let in_hand = t.hand(P0, "Rift Bolt");
    let actions = crate::r_s08_common::actions_of(&mut t, P0);
    assert!(actions.iter().any(|a| matches!(
        a,
        Action::Special(SpecialAction::Suspend { card }) if *card == in_hand
    )));
    crate::r_s05_common::move_to(&mut t, in_hand, Zone::Graveyard(P0));
    // A discard with an empty hand doesn't take it.
    let lib = t.library_size(P0);
    crate::r_s03_common::run_effect(
        &mut t,
        None,
        P0,
        Effect::Discard {
            who: PlayerRef::You,
            n: Value::Const(1),
            random: false,
            filter: Filter::Any,
        },
        &[],
    );
    assert_eq!(t.library_size(P0), lib);
    assert_eq!(t.zone(bolt), Zone::Library(P0));
}

#[test]
fn future_sight_top_card_cant_be_cycled() {
    cr!("401.1", "702.29a", "602.1");
    ruling!(
        "Future Sight",
        "The top card of your library isn't in your hand, so you can't suspend it, cycle it, or activate any of its abilities."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Future Sight");
    let misc = t.library_top(P0, "Miscalculation");
    t.lands(P0, "Island", 3);
    assert!(!can_cycle(&mut t, P0, misc));
    // Control: in hand it can be cycled.
    let in_hand = t.hand(P0, "Miscalculation");
    assert!(can_cycle(&mut t, P0, in_hand));
}

#[test]
fn future_sight_mandatory_additional_costs_must_be_paid() {
    cr!("601.2b", "601.2f", "118.8");
    ruling!(
        "Future Sight",
        "You must follow the normal timing permissions and restrictions for the top card of your library and pay its costs. If the card has alternative or additional costs, you may pay those. If the card has any mandatory additional costs, you must pay those."
    );
    // Village Rites: "As an additional cost to cast this spell, sacrifice a creature."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Future Sight");
    let rites = t.library_top(P0, "Village Rites");
    t.lands(P0, "Swamp", 1);
    assert!(!castable(&mut t, P0, rites), "no creature to sacrifice");
    t.battlefield(P0, "Grizzly Bears");
    assert!(castable(&mut t, P0, rites));
    let hand = t.hand_size(P0);
    t.cast(P0, rites).go();
    assert!(t.in_graveyard(P0, "Grizzly Bears"), "sacrificed as it was cast");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
    // Timing: a sorcery-speed card on top can't be cast in combat.
    t.set_step(P0, Step::BeginningOfCombat);
    let bears2 = t.library_top(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 2);
    assert!(!castable(&mut t, P0, bears2));
}

#[test]
fn garruks_horde_additional_and_alternative_costs() {
    cr!("601.2b", "118.9", "702.33a", "702.74a");
    ruling!(
        "Garruk's Horde",
        "You’ll still pay all costs for that spell, including additional costs. You may also pay alternative costs."
    );
    // Kicker (an additional cost): Kavu Titan kicked from the top.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Garruk's Horde");
    let titan = t.library_top(P0, "Kavu Titan");
    let lands = t.lands(P0, "Forest", 5);
    t.cast(P0, titan).kicked(true).go();
    assert!(lands.iter().all(|l| is_tapped(&t, *l)), "{{1}}{{G}} plus {{2}}{{G}}");
    t.resolve_all();
    assert_eq!(t.counters(titan, "+1/+1"), 3);
    // Evoke (an alternative cost): Mulldrifter for {2}{U}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Garruk's Horde");
    let drifter = t.library_top(P0, "Mulldrifter");
    t.lands(P0, "Island", 3);
    let evoke = legal_cast_methods(&mut t, P0, drifter)
        .into_iter()
        .find(|m| *m != CastMethod::Normal)
        .expect("evoke from the top");
    let hand = t.hand_size(P0);
    t.cast(P0, drifter).method(evoke).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Mulldrifter"), "sacrificed (evoked)");
    assert_eq!(t.hand_size(P0), hand + 2);
}
