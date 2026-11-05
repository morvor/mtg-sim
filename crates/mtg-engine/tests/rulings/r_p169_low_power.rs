//! Rulings batch P169 — "power 2 or less" abilities: casting from the top of the library
//! (Assemble the Players), searching (Imperial Recruiter), and "whenever a creature with
//! power 2 or less enters" triggers (CR 603.2, 603.6a, 603.4).

use crate::r_p169_common::*;
use mtg_engine::decision::{Action, Decision};
use mtg_engine::events::MoveCause;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::types::counters;
use mtg_engine::*;

/// The ways `p` may cast `card` now (and whether it may play it as a land).
fn cast_methods(t: &mut TestGame, p: PlayerId, card: ObjectId) -> (Vec<CastMethod>, bool) {
    t.g.turn.priority = Some(p);
    t.g.recompute();
    let card = t.g.current(card);
    let actions = t.g.legal_actions(p);
    let methods = actions
        .iter()
        .filter_map(|a| match a {
            Action::Cast { card: c, method } if *c == card => Some(method.clone()),
            _ => None,
        })
        .collect();
    let land = actions
        .iter()
        .any(|a| matches!(a, Action::PlayLand { card: c } if *c == card));
    (methods, land)
}

#[test]
fn assemble_the_players_cant_play_a_land_creature_from_the_top() {
    cr!("305.9", "601.3");
    ruling!(
        "Assemble the Players",
        "Because you never \"cast\" a land card, Assemble the Players doesn't allow you to play a land creature (such as Dryad Arbor) from the top of your library."
    );
    supported("Assemble the Players");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Assemble the Players");
    t.lands(P0, "Forest", 2);
    let arbor = t.library_top(P0, "Dryad Arbor");
    assert_eq!(cast_methods(&mut t, P0, arbor), (vec![], false));
    // Positive control: a 2/2 creature card on top can be cast.
    let bears = t.library_top(P0, "Grizzly Bears");
    assert!(castable(&mut t, P0, bears));
}

/// Casts the top card of P0's library (a real creature card put there) and resolves it.
fn cast_from_top(t: &mut TestGame, name: &str) -> ObjectId {
    let c = t.library_top(P0, name);
    lands_for_cost(t, P0, name);
    assert!(castable(t, P0, c), "{name} isn't castable from the top");
    t.cast(P0, c).go();
    t.resolve_all();
    c
}

#[test]
fn each_assemble_the_players_grants_its_own_once_each_turn_permission() {
    cr!("400.7", "601.3");
    ruling!(
        "Assemble the Players",
        "If Assemble the Players leaves the battlefield and returns to the battlefield in the same turn, the casting permission granted by the old one is different than the casting permission granted by the new one. Similarly, if you control multiple Assemble the Players, each one has a different casting permission that may be used once each turn."
    );
    // One Assemble the Players: once each turn.
    let mut t = TestGame::new(2);
    let assemble = t.battlefield(P0, "Assemble the Players");
    cast_from_top(&mut t, "Grizzly Bears");
    let lions = t.library_top(P0, "Savannah Lions");
    t.lands(P0, "Plains", 1);
    assert!(!castable(&mut t, P0, lions));
    // It leaves and returns: the new object grants a new permission.
    t.g.move_object(assemble, Zone::Exile, MoveCause::Effect, Some(P0));
    let back = t.g.current(assemble);
    t.g.move_object(back, Zone::Battlefield, MoveCause::Effect, Some(P0));
    t.g.recompute();
    assert!(castable(&mut t, P0, lions));
    t.cast(P0, lions).go();
    t.resolve_all();
    assert!(t.on_battlefield(lions));

    // Two of them: two creature spells from the top this turn, but not a third.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Assemble the Players");
    t.battlefield(P0, "Assemble the Players");
    cast_from_top(&mut t, "Grizzly Bears");
    cast_from_top(&mut t, "Savannah Lions");
    let third = t.library_top(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 2);
    assert!(!castable(&mut t, P0, third));
}

#[test]
fn assemble_the_players_casts_a_morph_face_down_from_the_top() {
    cr!("708.4", "702.37c");
    ruling!(
        "Assemble the Players",
        "If the top card of your library has a disguise or morph ability, you can cast it face down from the top of your library, even if it doesn't normally have power 2 or less."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Assemble the Players");
    t.lands(P0, "Wastes", 3);
    let abom = t.library_top(P0, "Abomination of Gudul");
    let (methods, _) = cast_methods(&mut t, P0, abom);
    // Only face down (as a 2/2); a 3/4 creature spell can't be cast this way.
    assert_eq!(methods, vec![CastMethod::FaceDown(KeywordKind::Morph)]);
    t.cast(P0, abom)
        .method(CastMethod::FaceDown(KeywordKind::Morph))
        .go();
    t.resolve_all();
    assert!(t.on_battlefield(abom));
    assert!(t.obj_now(abom).face_down);
    assert_eq!(t.pt(abom), (2, 2));
}

#[test]
fn imperial_recruiter_sees_a_star_power_in_the_library() {
    cr!("604.3", "604.3a");
    ruling!(
        "Imperial Recruiter",
        "If a creature card has \"*\" in its power, the ability that defines its power works in all zones."
    );
    supported("Imperial Recruiter");
    // With three card types in graveyards, Tarmogoyf in the library is 3/4: not findable.
    let offered = |gy: &[&str]| {
        let mut t = TestGame::new(2);
        for c in gy {
            t.graveyard(P1, c);
        }
        let goyf = t.library_top(P0, "Tarmogoyf");
        let from = t.asked().len();
        t.enter(P0, "Imperial Recruiter");
        t.resolve_all();
        let found = t.asked()[from..].iter().any(|(_, d)| {
            matches!(d, Decision::ChooseEntities { candidates, .. }
                if candidates.contains(&obj(goyf)))
        });
        (found, t.zone(goyf))
    };
    let (found, zone) = offered(&["Lightning Bolt", "Forest", "Grizzly Bears"]);
    assert!(!found);
    assert!(matches!(zone, Zone::Library(_)));
    // With empty graveyards it's 0/1, and found.
    let (found, zone) = offered(&[]);
    assert!(found);
    assert_eq!(zone, Zone::Hand(P0));
}

/// Draws by P0 this turn.
fn hand(t: &TestGame) -> usize {
    t.hand_size(P0)
}

#[test]
fn welcoming_vampire_sees_counters_and_static_effects_as_creatures_enter() {
    cr!("603.6a", "613.1");
    ruling!(
        "Welcoming Vampire",
        "If creatures enter the battlefield with +1/+1 counters or a continuous effect such as that of Wedding Festivity will apply to the creatures on the battlefield, those effects apply when checking to see if Welcoming Vampire's ability will trigger."
    );
    supported("Welcoming Vampire");
    // Positive control: a 2/2 entering triggers it.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Welcoming Vampire");
    t.enter(P0, "Grizzly Bears");
    t.resolve_all();
    assert_eq!(hand(&t), 1);
    // Entering with a +1/+1 counter: 3/3, no trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Welcoming Vampire");
    enter_with(&mut t, P0, "Grizzly Bears", 1, false);
    t.resolve_all();
    assert_eq!(hand(&t), 0);
    // An anthem ("Creatures you control get +1/+1", as Wedding Festivity) makes it 3/3
    // as it enters: no trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Welcoming Vampire");
    t.battlefield(P0, "Glorious Anthem");
    t.enter(P0, "Grizzly Bears");
    t.resolve_all();
    assert_eq!(hand(&t), 0);
}

#[test]
fn welcoming_vampire_checks_power_only_as_creatures_enter() {
    cr!("603.2", "603.4");
    ruling!(
        "Welcoming Vampire",
        "Welcoming Vampire's ability checks the power of the other creatures only as they enter the battlefield. If one of those creatures has power 2 or less, the ability will trigger. Once the ability triggers, raising that creature's power above 2 won't affect that ability. Similarly, reducing a creature's power to 2 or less after it enters the battlefield won't cause the ability to trigger."
    );
    // Raised above 2 while the ability is on the stack: still draws.
    let mut t = TestGame::new(2);
    let wv = t.battlefield(P0, "Welcoming Vampire");
    let bears = t.enter(P0, "Grizzly Bears");
    t.settle();
    assert_eq!(stack_triggers_from(&t, wv).len(), 1);
    cast_new(&mut t, P0, "Giant Growth", &[obj(bears)]);
    t.resolve();
    assert_eq!(t.pt(bears), (5, 5));
    t.resolve_all();
    assert_eq!(hand(&t), 1);
    // A 3/3 entering, then reduced to 2 power: no trigger.
    let mut t = TestGame::new(2);
    let wv = t.battlefield(P0, "Welcoming Vampire");
    let giant = t.enter(P0, "Hill Giant");
    t.settle();
    assert_eq!(triggers_of(&t, wv), 0);
    crate::r_p117_common::minus_power(&mut t, giant, 1);
    assert_eq!(t.pt(giant).0, 2);
    t.resolve_all();
    assert_eq!(triggers_of(&t, wv), 0);
    assert_eq!(hand(&t), 0);
}

/// The effect of a "whenever another creature you control with power 2 or less enters"
/// trigger has happened (`bears` is the entering creature, `src` the card).
type Check = fn(&TestGame, ObjectId, ObjectId) -> bool;

#[test]
fn power_two_or_less_enter_triggers_check_only_as_the_creature_enters() {
    cr!("603.2", "603.6a", "603.4");
    ruling!(
        "Neighborhood Guardian",
        "Neighborhood Guardian's ability checks the power of a creature only at the moment it enters the battlefield. If it enters with counters, those counters are included. If that creature's power is 2 or less when it enters the battlefield but becomes greater than 2 after the ability triggers, the target creature will still get +1/+1 until end of turn."
    );
    ruling!(
        "Serra Redeemer",
        "Serra Redeemer's second ability checks the power of a creature only at the moment it enters the battlefield. If it enters with counters, those counters are included. If that creature's power is less than 2 when it enters the battlefield but becomes greater than 2 after the ability triggers, you will still put two counters on it."
    );
    ruling!(
        "Vicious Clown",
        "Vicious Clown's ability checks the power of the other creature only at the moment it enters. If a creature enters with counters, those counters are included. If a creature's power is 2 or less when it enters but becomes greater than 2 while Vicious Clown's ability is still on the stack, Vicious Clown will still get +2/+0 until end of turn when its ability resolves."
    );
    // Each card: the effect seen once Giant Growth (+3/+3) and the trigger resolved.
    let cases: [(&str, Check); 3] = [
        ("Neighborhood Guardian", |t, _, b| t.pt(b) == (6, 6)),
        ("Serra Redeemer", |t, _, b| {
            t.counters(b, counters::PLUS1) == 2 && t.pt(b) == (7, 7)
        }),
        ("Vicious Clown", |t, s, _| t.pt(s) == (4, 3)),
    ];
    for (card, check) in cases {
        supported(card);
        // Entering with a +1/+1 counter (3/3): no trigger.
        let mut t = TestGame::new(2);
        let src = t.battlefield(P0, card);
        enter_with(&mut t, P0, "Grizzly Bears", 1, false);
        t.resolve_all();
        assert_eq!(triggers_of(&t, src), 0, "{card}");
        // A 2/2 enters; its power becomes 5 while the trigger waits: it still resolves.
        let mut t = TestGame::new(2);
        let src = t.battlefield(P0, card);
        let bears = t.enter(P0, "Grizzly Bears");
        t.answer_targets(P0, &[obj(bears)]);
        t.settle();
        assert_eq!(stack_triggers_from(&t, src).len(), 1, "{card}");
        cast_new(&mut t, P0, "Giant Growth", &[obj(bears)]);
        t.resolve();
        assert_eq!(t.pt(bears).0, 5);
        t.resolve_all();
        assert!(check(&t, src, bears), "{card}: {:?} {:?}", t.pt(src), t.pt(bears));
    }
}

