//! Rulings batch S33 — the top card of a library: a player allowed to look at it may do
//! so any time, without priority (CR 401.5); a card played from the top of a library
//! follows its normal timing (CR 601.3, 305.2) and a land played that way uses a land
//! play (CR 305.2, 305.3); the top card isn't in its owner's hand, so it can't be cycled,
//! suspended, discarded or have its other hand abilities activated (CR 602.2, 702.29a,
//! 702.62a, 701.9a).

use crate::r_s01_common::{stack_library, supported};
use crate::r_s02_common::{can_cast, can_play_land};
use crate::r_s08_common::actions_of;
use crate::r_s29_common::cast_and_resolve;
use mtg_engine::decision::{Action, SpecialAction};
use mtg_engine::facedown::can_look_at;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// The legal actions of `p` now (with priority) that use `card`: casting or playing it,
/// activating its abilities, or a special action with it.
fn actions_with(t: &mut TestGame, p: PlayerId, card: ObjectId) -> Vec<Action> {
    let card = t.g.current(card);
    actions_of(t, p)
        .into_iter()
        .filter(|a| match a {
            Action::PlayLand { card: c } | Action::Cast { card: c, .. } => *c == card,
            Action::Activate { source, .. } => *source == card,
            Action::Special(SpecialAction::Suspend { card: c })
            | Action::Special(SpecialAction::Foretell { card: c })
            | Action::Special(SpecialAction::Plot { card: c }) => *c == card,
            _ => false,
        })
        .collect()
}

/// Whether some action in `actions` activates an ability or is a special action.
fn activates_or_special(actions: &[Action]) -> bool {
    actions
        .iter()
        .any(|a| matches!(a, Action::Activate { .. } | Action::Special(_)))
}

/// P1 makes P0 discard two cards (Mind Rot) while P0's hand is empty: the top card of
/// P0's library isn't discarded.
fn opponent_mind_rots_p0(t: &mut TestGame, top: ObjectId) {
    assert_eq!(t.hand_size(P0), 0);
    t.set_step(P1, Step::PrecombatMain);
    cast_and_resolve(t, P1, "Mind Rot", &[Entity::Player(P0)]);
    assert_eq!(t.zone(top), Zone::Library(P0));
    assert_eq!(t.g.library_top(P0), Some(top));
    assert_eq!(t.graveyard_size(P0), 0);
}

#[test]
fn korlessa_the_top_card_can_be_looked_at_any_time_without_priority() {
    cr!("401.5", "116.1");
    ruling!(
        "Korlessa, Scale Singer",
        "You can look at the top card of your library whenever you want (with one restriction; see below), even if you don't have priority. This action doesn't use the stack. Knowing what that card is becomes part of the information you have access to, just like you can look at the cards in your hand."
    );
    supported("Korlessa, Scale Singer");
    // "You may look at the top card of your library any time."
    let mut t = TestGame::new(2);
    let top = t.library_top(P0, "Grizzly Bears");
    assert!(!can_look_at(&t.g, P0, top));
    t.battlefield(P0, "Korlessa, Scale Singer");
    t.g.recompute();
    // During P1's turn, while P1 has priority: P0 may look at it, as at the cards in
    // their hand; P1 may not. Nothing goes on the stack.
    t.set_step(P1, Step::DeclareAttackers);
    assert_eq!(t.g.turn.priority, Some(P1));
    assert!(can_look_at(&t.g, P0, top));
    assert!(!can_look_at(&t.g, P1, top));
    assert_eq!(t.stack_len(), 0);
    let hand = t.hand(P0, "Hill Giant");
    assert!(can_look_at(&t.g, P0, hand));
    // A new top card can be looked at too.
    let next = t.library_top(P0, "Hill Giant");
    assert!(can_look_at(&t.g, P0, next));
    assert!(!can_look_at(&t.g, P0, top));
}

#[test]
fn augur_of_autumn_the_top_card_cant_be_suspended_cycled_discarded_or_activated() {
    cr!("702.62a", "702.29a", "602.2", "701.9a");
    ruling!(
        "Augur of Autumn",
        "The top card of your library isn't in your hand, so you can't suspend it, cycle it, discard it, or activate any of its activated abilities."
    );
    supported("Augur of Autumn");
    // "You may look at the top card of your library any time. You may play lands from the
    // top of your library."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Augur of Autumn");
    t.lands(P0, "Mountain", 3);
    t.lands(P0, "Swamp", 2);
    t.lands(P0, "Forest", 2);
    // Suspend (Rift Bolt): only from a hand.
    let bolt = t.library_top(P0, "Rift Bolt");
    assert!(actions_with(&mut t, P0, bolt).is_empty());
    let bolt_in_hand = t.hand(P0, "Rift Bolt");
    assert!(activates_or_special(&actions_with(&mut t, P0, bolt_in_hand)));
    // Cycling (Horror of the Broken Lands).
    let horror = t.library_top(P0, "Horror of the Broken Lands");
    assert!(actions_with(&mut t, P0, horror).is_empty());
    // Channel (Boseiju, Who Endures): the land can be played from the top, but its hand
    // ability can't be activated.
    let boseiju = t.library_top(P0, "Boseiju, Who Endures");
    t.battlefield(P1, "Mind Stone");
    let acts = actions_with(&mut t, P0, boseiju);
    assert!(acts
        .iter()
        .any(|a| matches!(a, Action::PlayLand { .. })));
    assert!(!activates_or_special(&acts));
    // Discarding: an effect making P0 discard doesn't reach it (P0's hand has only the
    // other Rift Bolt; move it away first).
    t.g.move_object(
        bolt_in_hand,
        Zone::Exile,
        mtg_engine::events::MoveCause::Effect,
        None,
    );
    opponent_mind_rots_p0(&mut t, boseiju);
}

#[test]
fn garruks_horde_the_top_card_cant_be_suspended_or_cycled() {
    cr!("702.62a", "702.29a", "602.2", "701.9a");
    ruling!(
        "Garruk's Horde",
        "The top card of your library isn’t in your hand, so you can’t suspend it, cycle it, discard it, or activate any of its activated abilities."
    );
    supported("Garruk's Horde");
    // "Play with the top card of your library revealed. You may cast creature spells from
    // the top of your library."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Garruk's Horde");
    t.lands(P0, "Mountain", 5);
    t.lands(P0, "Swamp", 1);
    // Keldon Halberdier (suspend 4): it can be cast from the top, not suspended.
    let halberdier = t.library_top(P0, "Keldon Halberdier");
    let acts = actions_with(&mut t, P0, halberdier);
    assert!(acts.iter().any(|a| matches!(a, Action::Cast { .. })));
    assert!(!activates_or_special(&acts));
    // Monstrous Carabid (cycling {B/R}): cast, not cycled.
    let carabid = t.library_top(P0, "Monstrous Carabid");
    let acts = actions_with(&mut t, P0, carabid);
    assert!(acts.iter().any(|a| matches!(a, Action::Cast { .. })));
    assert!(!activates_or_special(&acts));
    opponent_mind_rots_p0(&mut t, carabid);
}

#[test]
fn traveling_chocobo_the_top_card_cant_be_cycled() {
    cr!("702.29a", "602.2", "305.2");
    ruling!(
        "Traveling Chocobo",
        "The top card of your library isn't in your hand, so you can't cycle it, discard it, or activate any of its abilities that could be activated from your hand."
    );
    supported("Traveling Chocobo");
    // "You may look at the top card of your library any time. You may play lands and cast
    // Bird spells from the top of your library."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Traveling Chocobo");
    t.lands(P0, "Island", 2);
    // Desert of the Mindful (cycling {1}{U}): it can be played, not cycled.
    let desert = t.library_top(P0, "Desert of the Mindful");
    let acts = actions_with(&mut t, P0, desert);
    assert_eq!(acts, vec![Action::PlayLand { card: desert }]);
    // In hand, it could be cycled.
    let in_hand = t.hand(P0, "Desert of the Mindful");
    assert!(activates_or_special(&actions_with(&mut t, P0, in_hand)));
    t.g.move_object(
        in_hand,
        Zone::Exile,
        mtg_engine::events::MoveCause::Effect,
        None,
    );
    opponent_mind_rots_p0(&mut t, desert);
}

#[test]
fn elven_chorus_the_top_card_cant_be_cycled_or_discarded() {
    cr!("702.29a", "701.9a", "602.2");
    ruling!(
        "Elven Chorus",
        "The top card of your library isn't in your hand, so you can't take other actions that would normally be allowed from your hand, such as discarding it due to an effect or activating a cycling ability."
    );
    supported("Elven Chorus");
    // "You may look at the top card of your library any time. You may cast creature spells
    // from the top of your library."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Elven Chorus");
    t.lands(P0, "Mountain", 4);
    t.lands(P0, "Swamp", 1);
    let carabid = t.library_top(P0, "Monstrous Carabid");
    let acts = actions_with(&mut t, P0, carabid);
    assert!(acts.iter().any(|a| matches!(a, Action::Cast { .. })));
    assert!(!activates_or_special(&acts));
    opponent_mind_rots_p0(&mut t, carabid);
}

#[test]
fn precognition_field_spells_from_the_top_follow_their_normal_timing() {
    cr!("601.3", "307.1", "304.1");
    ruling!(
        "Precognition Field",
        "You must follow the normal timing permissions and restrictions of the cards you play from your library."
    );
    supported("Precognition Field");
    // "You may cast instant and sorcery spells from the top of your library."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Precognition Field");
    t.lands(P0, "Island", 3);
    let divination = t.library_top(P0, "Divination");
    assert!(can_cast(&mut t, P0, divination, CastMethod::Normal));
    // In combat, or during P1's turn: the sorcery can't be cast; an instant can.
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!can_cast(&mut t, P0, divination, CastMethod::Normal));
    t.set_step(P1, Step::PrecombatMain);
    assert!(!can_cast(&mut t, P0, divination, CastMethod::Normal));
    let opt = t.library_top(P0, "Opt");
    assert!(can_cast(&mut t, P0, opt, CastMethod::Normal));
}

#[test]
fn emperor_mihail_merfolk_creature_spells_from_the_top_follow_sorcery_timing() {
    cr!("601.3", "302.1");
    ruling!(
        "Emperor Mihail II",
        "You must follow the normal timing permissions and restrictions of the cards you play from your library."
    );
    supported("Emperor Mihail II");
    // "You may cast Merfolk spells from the top of your library."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Emperor Mihail II");
    t.lands(P0, "Island", 2);
    let lord = t.library_top(P0, "Lord of Atlantis");
    assert!(can_cast(&mut t, P0, lord, CastMethod::Normal));
    // Not while a spell is on the stack, not in combat, not in P1's turn.
    let bolt = t.hand(P0, "Lightning Bolt");
    t.lands(P0, "Mountain", 1);
    t.cast(P0, bolt).target(Entity::Player(P1)).go();
    assert!(!can_cast(&mut t, P0, lord, CastMethod::Normal));
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    t.set_step(P0, Step::DeclareAttackers);
    assert!(!can_cast(&mut t, P0, lord, CastMethod::Normal));
    t.set_step(P1, Step::PrecombatMain);
    assert!(!can_cast(&mut t, P0, lord, CastMethod::Normal));
    t.set_step(P0, Step::PostcombatMain);
    assert!(can_cast(&mut t, P0, lord, CastMethod::Normal));
}

#[test]
fn courser_of_kruphix_a_land_from_the_top_uses_the_land_play() {
    cr!("305.2", "305.3", "305.2a");
    ruling!(
        "Courser of Kruphix",
        "Playing a land from the top of your library counts as your land play for the turn. Once you play a land during your turn, you won't be able to play an additional land from the top of your library unless another effect (such as that of Azusa, Lost but Seeking) allows you to."
    );
    supported("Courser of Kruphix");
    // "Play with the top card of your library revealed. You may play lands from the top of
    // your library."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Courser of Kruphix");
    // (Explore draws the Mountain.)
    let tops = stack_library(&mut t, P0, &["Forest", "Mountain", "Plains"]);
    let island = t.hand(P0, "Island");
    assert!(can_play_land(&mut t, P0, tops[0]));
    t.play_land(P0, tops[0]).unwrap();
    t.settle();
    // The land play is used: neither the next land on top nor one in hand can be played.
    assert!(!can_play_land(&mut t, P0, tops[1]));
    assert!(!can_play_land(&mut t, P0, island));
    // An effect allowing an additional land play (Explore) lets P0 play one from the top.
    t.lands(P0, "Forest", 2);
    t.resolve_all();
    let explore = t.hand(P0, "Explore");
    t.cast(P0, explore).go();
    t.resolve_all();
    let top = t.g.library_top(P0).unwrap();
    assert_eq!(top, t.g.current(tops[2]));
    assert!(can_play_land(&mut t, P0, top));
    t.play_land(P0, top).unwrap();
    t.settle();
    assert!(!can_play_land(&mut t, P0, island));
}

#[test]
fn magus_of_the_future_a_land_from_the_top_uses_the_land_play() {
    cr!("305.2", "305.3");
    ruling!(
        "Magus of the Future",
        "Playing a land from the top of your library counts as your land play for the turn. Once you play a land during your turn, you won't be able to play an additional land from the top of your library unless another effect (such as that of Azusa, Lost but Seeking) allows you to."
    );
    supported("Magus of the Future");
    // Playing a land from hand first uses the land play too.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Magus of the Future");
    let top = t.library_top(P0, "Forest");
    let island = t.hand(P0, "Island");
    assert!(can_play_land(&mut t, P0, top));
    t.play_land(P0, island).unwrap();
    t.settle();
    assert!(!can_play_land(&mut t, P0, top));
    // Azusa, Lost but Seeking: "You may play two additional lands on each of your turns."
    supported("Azusa, Lost but Seeking");
    t.battlefield(P0, "Azusa, Lost but Seeking");
    assert!(can_play_land(&mut t, P0, top));
    t.play_land(P0, top).unwrap();
    t.settle();
    let next = t.library_top(P0, "Mountain");
    assert!(can_play_land(&mut t, P0, next));
    t.play_land(P0, next).unwrap();
    t.settle();
    let last = t.library_top(P0, "Plains");
    assert!(!can_play_land(&mut t, P0, last));
}
