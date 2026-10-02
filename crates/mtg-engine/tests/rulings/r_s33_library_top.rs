//! Rulings batch S33 — the top card of a library: a player allowed to look at it may do
//! so any time, without priority (CR 401.5); a card played from the top of a library
//! follows its normal timing (CR 601.3) and a land played that way uses a land
//! play (CR 305.2); the top card isn't in its owner's hand, so it can't be cycled,
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
    cr!("401.5");
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
    cr!("305.2", "305.2a", "305.2b");
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
    cr!("305.2", "305.2b");
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

#[test]
fn vizier_of_the_menagerie_the_top_card_cant_be_cycled_or_discarded() {
    cr!("702.29a", "701.9a", "602.2");
    ruling!(
        "Vizier of the Menagerie",
        "The top card of your library isn't in your hand, so you can't cycle it, discard it, or activate any of its activated abilities."
    );
    supported("Vizier of the Menagerie");
    // "You may look at the top card of your library any time. You may cast creature
    // spells from the top of your library. You can spend mana of any type to cast creature
    // spells."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Vizier of the Menagerie");
    t.lands(P0, "Island", 5);
    // Monstrous Carabid ({3}{B}{R}, cycling {B/R}): cast with Islands, never cycled.
    let carabid = t.library_top(P0, "Monstrous Carabid");
    let acts = actions_with(&mut t, P0, carabid);
    assert!(acts.iter().any(|a| matches!(a, Action::Cast { .. })));
    assert!(!activates_or_special(&acts));
    opponent_mind_rots_p0(&mut t, carabid);
}

#[test]
fn vizier_of_the_menagerie_any_type_of_mana_for_any_creature_spell() {
    cr!("609.4b", "118.14");
    ruling!(
        "Vizier of the Menagerie",
        "You may spend mana as though it were mana of any type to cast any creature spell, not just creature spells that you cast from the top of your library."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 2);
    let bears = t.hand(P0, "Grizzly Bears");
    let growth = t.hand(P0, "Giant Growth");
    t.battlefield(P0, "Grizzly Bears");
    assert!(!can_cast(&mut t, P0, bears, CastMethod::Normal));
    t.battlefield(P0, "Vizier of the Menagerie");
    t.g.recompute();
    // A creature spell from the hand: {1}{G} paid with two Islands.
    assert!(can_cast(&mut t, P0, bears, CastMethod::Normal));
    // Not a noncreature spell.
    assert!(!can_cast(&mut t, P0, growth, CastMethod::Normal));
    t.cast(P0, bears).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 2);
}

/// The ways P0 may cast `card` now.
fn methods(t: &mut TestGame, card: ObjectId) -> Vec<CastMethod> {
    crate::r_s08_common::legal_cast_methods(t, P0, card)
}

#[test]
fn bolass_citadel_a_land_from_the_top_needs_an_available_land_play() {
    cr!("305.2", "305.1");
    ruling!(
        "Bolas's Citadel",
        "You can play a land card from the top of your library only if you have available land plays remaining."
    );
    supported("Bolas's Citadel");
    // "You may play lands and cast spells from the top of your library. If you cast a
    // spell this way, pay life equal to its mana value rather than pay its mana cost."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Bolas's Citadel");
    let top = t.library_top(P0, "Forest");
    assert!(can_play_land(&mut t, P0, top));
    let island = t.hand(P0, "Island");
    t.play_land(P0, island).unwrap();
    t.settle();
    assert!(!can_play_land(&mut t, P0, top));
    // With the land play available, it can be played.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Bolas's Citadel");
    let top = t.library_top(P0, "Forest");
    t.play_land(P0, top).unwrap();
    t.settle();
    assert!(t.on_battlefield(top));
    // That was the turn's land play.
    let next = t.library_top(P0, "Mountain");
    assert!(!can_play_land(&mut t, P0, next));
}

#[test]
fn bolass_citadel_spells_from_the_top_cost_life_and_no_other_alternative_cost() {
    cr!("118.9", "118.9a", "118.8", "601.2f", "601.2h");
    ruling!(
        "Bolas's Citadel",
        "If you cast a spell for another cost \"rather than pay its mana cost,\" you can't choose to cast it for any alternative costs. You can, however, pay additional costs. If the card has any mandatory additional costs, such as that of Spark Harvest, those must be paid to cast the card."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Bolas's Citadel");
    t.lands(P0, "Swamp", 2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    // Snuff Out ("If you control a Swamp, you may pay 4 life rather than pay this spell's
    // mana cost. Destroy target nonblack creature."): from the top, only for Bolas's
    // Citadel's cost.
    let snuff = t.library_top(P0, "Snuff Out");
    assert_eq!(
        methods(&mut t, snuff),
        vec![CastMethod::Alternative(mtg_engine::casting::PERMISSION_COST)]
    );
    // Bone Splinters ({B}; "As an additional cost to cast this spell, sacrifice a
    // creature."): P0 pays 1 life and sacrifices a creature; no mana is spent.
    let splinters = t.library_top(P0, "Bone Splinters");
    let m = methods(&mut t, splinters);
    assert_eq!(m.len(), 1);
    t.cast(P0, splinters).method(m[0].clone()).target(giant).go();
    assert_eq!(t.life(P0), 19);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert!(!t.on_battlefield(bears));
    assert_eq!(crate::r_s04_common::untapped_lands(&t, P0), 2);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Hill Giant"));
    // Without a creature to sacrifice, it can't be cast.
    let again = t.library_top(P0, "Bone Splinters");
    assert!(methods(&mut t, again).is_empty());
}

#[test]
fn gwenom_a_land_from_the_top_needs_an_available_land_play() {
    cr!("305.2", "611.2a");
    ruling!(
        "Gwenom, Remorseless",
        "You can play a land card from the top of your library only if you have available land plays remaining."
    );
    supported("Gwenom, Remorseless");
    // "Whenever Gwenom attacks, until end of turn, you may look at the top card of your
    // library any time and you may play cards from the top of your library. If you cast a
    // spell this way, pay life equal to its mana value rather than pay its mana cost."
    let mut t = TestGame::new(2);
    let gwenom = t.battlefield(P0, "Gwenom, Remorseless");
    let island = t.hand(P0, "Island");
    let forest = t.library_top(P0, "Forest");
    // Before Gwenom attacks, there's no permission.
    assert!(!can_play_land(&mut t, P0, forest));
    t.set_step(P0, Step::BeginningOfCombat);
    crate::r_s01_common::attack_with(&mut t, &[(gwenom, Entity::Player(P1))]);
    t.resolve_all();
    t.advance_to(P0, Step::PostcombatMain);
    // With the turn's land play available, the land on top can be played; once P0 has
    // played a land this turn, it can't.
    assert!(can_play_land(&mut t, P0, forest));
    t.play_land(P0, island).unwrap();
    t.settle();
    assert!(!can_play_land(&mut t, P0, forest));
    // A spell from the top costs life equal to its mana value.
    let bears = t.library_top(P0, "Grizzly Bears");
    let m = methods(&mut t, bears);
    assert_eq!(m.len(), 1);
    let life = t.life(P0);
    t.cast(P0, bears).method(m[0].clone()).go();
    assert_eq!(t.life(P0), life - 2);
    t.resolve_all();
    // The permission lasts until end of turn: an instant on top can be cast in P0's end
    // step, but not once the turn is over.
    let bolt = t.library_top(P0, "Lightning Bolt");
    t.advance_to(P0, Step::End);
    assert_eq!(
        methods(&mut t, bolt),
        vec![CastMethod::Alternative(mtg_engine::casting::PERMISSION_COST)]
    );
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.g.library_top(P0), Some(t.g.current(bolt)));
    assert!(methods(&mut t, bolt).is_empty());
}

#[test]
fn bolass_citadel_x_is_0_for_a_spell_cast_for_life() {
    cr!("107.3b", "202.3e");
    ruling!(
        "Bolas's Citadel",
        "If a spell has {X} in its mana cost, you must choose 0 as the value of X when casting it without paying its mana cost."
    );
    // Fireball ({X}{R}: "This spell costs {1} more to cast for each target beyond the
    // first. Fireball deals X damage divided as you choose among any number of targets.")
    // from the top: P0 isn't asked for X, pays 1 life (its mana value with X = 0), and it
    // deals 0 damage.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Bolas's Citadel");
    let fireball = t.library_top(P0, "Fireball");
    let m = methods(&mut t, fireball);
    assert_eq!(
        m,
        vec![CastMethod::Alternative(mtg_engine::casting::PERMISSION_COST)]
    );
    t.cast(P0, fireball)
        .method(m[0].clone())
        .x(5)
        .target(Entity::Player(P1))
        .go();
    assert_eq!(t.life(P0), 19);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    assert!(t.in_graveyard(P0, "Fireball"));
}
