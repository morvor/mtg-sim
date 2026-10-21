//! Rulings batch P145 — cards "exiled with" a permanent and permissions to cast cards from
//! a zone: Bane Alley Broker, Hauken's Insight, Jace, Telepath Unbound's −3, Jace, Vryn's
//! Prodigy and Arcade Gannon.

use crate::r_p079_common::back_face;
use crate::r_p145_common::*;
use mtg_engine::facedown::can_look_at;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

const BROKER: &str = "Bane Alley Broker";

/// `p` activates `broker`'s first ability (untapping it first), exiling `card` from hand.
fn broker_exiles(t: &mut TestGame, p: PlayerId, broker: ObjectId, card: ObjectId) {
    let b = t.g.current(broker);
    t.g.objects[b.0 as usize].tapped = false;
    t.library_top(p, "Island");
    t.answer_choose(p, &[obj(card)]);
    t.activate(p, b, 0, &[]).expect("exile ability");
    t.resolve_all();
    assert_eq!(t.zone(card), Zone::Exile);
}

/// The cards `p` may choose from as `broker`'s last ability resolves.
fn broker_return_choices(t: &mut TestGame, p: PlayerId, broker: ObjectId) -> Vec<Entity> {
    let b = t.g.current(broker);
    t.g.objects[b.0 as usize].tapped = false;
    t.lands(p, "Island", 1);
    t.lands(p, "Swamp", 1);
    let from = n_asked(t);
    t.activate(p, b, 1, &[]).expect("return ability");
    t.resolve_all();
    entity_choices(t, p, from)
        .into_iter()
        .flat_map(|(_, _, c)| c)
        .collect()
}

#[test]
fn bane_alley_broker_tracks_its_own_exiled_cards() {
    cr!("607.2a", "406.6");
    ruling!(
        "Bane Alley Broker",
        "Bane Alley Broker’s second and third abilities apply to cards exiled with that specific Bane Alley Broker, not any other creature named Bane Alley Broker."
    );
    supported(BROKER);
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, BROKER);
    let b = t.battlefield(P0, BROKER);
    let bears = t.hand(P0, "Grizzly Bears");
    let elves = t.hand(P0, "Llanowar Elves");
    broker_exiles(&mut t, P0, a, bears);
    broker_exiles(&mut t, P0, b, elves);
    let exiled_elves = t.g.current(elves);
    assert_eq!(
        broker_return_choices(&mut t, P0, b),
        vec![obj(exiled_elves)]
    );
    assert!(t.in_hand(P0, "Llanowar Elves"));
    assert_eq!(t.zone(bears), Zone::Exile);
}

#[test]
fn bane_alley_broker_draws_then_exiles_and_a_new_broker_has_no_cards() {
    cr!("608.2c", "400.7", "607.2a");
    ruling!(
        "Bane Alley Broker",
        "If Bane Alley Broker’s first ability resolves when you have no cards in your hand, you’ll draw a card and then exile it."
    );
    ruling!(
        "Bane Alley Broker",
        "If Bane Alley Broker leaves the battlefield, the cards exiled with it will be exiled indefinitely."
    );
    let mut t = TestGame::new(2);
    let broker = t.battlefield(P0, BROKER);
    let memnite = t.library_top(P0, "Memnite");
    assert_eq!(t.hand_size(P0), 0);
    t.activate(P0, broker, 0, &[]).expect("exile ability");
    t.resolve_all();
    assert_eq!(t.zone(memnite), Zone::Exile);
    assert!(t.obj_now(memnite).face_down);
    assert!(can_look_at(&t.g, P0, t.g.current(memnite)));
    // The Broker leaves and comes back: the new object has no exiled cards.
    move_to(&mut t, broker, Zone::Hand(P0));
    let new = t.battlefield(P0, BROKER);
    assert!(broker_return_choices(&mut t, P0, new).is_empty());
    assert_eq!(t.zone(memnite), Zone::Exile);
}

#[test]
fn bane_alley_broker_owners_are_known_under_another_controller() {
    cr!("607.2a", "108.3", "400.3");
    ruling!(
        "Bane Alley Broker",
        "Even if not all players can look at the exiled cards, each card’s owner is still known."
    );
    ruling!(
        "Bane Alley Broker",
        "If you no longer control Bane Alley Broker when its last ability resolves, you can continue to look at the relevant cards in exile to choose one to return."
    );
    let mut t = TestGame::new(2);
    let broker = t.battlefield(P0, BROKER);
    let bears = t.hand(P0, "Grizzly Bears");
    broker_exiles(&mut t, P0, broker, bears);
    // P0 activates the return ability; P1 gains control of the Broker in response. P0
    // still chooses among the cards exiled with it and gets the Bears back.
    t.lands(P0, "Island", 1);
    t.lands(P0, "Swamp", 1);
    t.g.objects[broker.0 as usize].tapped = false;
    t.activate(P0, broker, 1, &[]).expect("return ability");
    t.set_step(P1, Step::PrecombatMain);
    let exiled = t.g.current(bears);
    cast_new(&mut t, P1, "Ray of Command", &[obj(broker)]);
    t.resolve();
    assert_eq!(t.obj_now(broker).controller, P1);
    let from = n_asked(&t);
    t.resolve_all();
    assert_eq!(entity_choices(&t, P0, from)[0].2, vec![obj(exiled)]);
    assert!(t.in_hand(P0, "Grizzly Bears"));
    // Under P1's control it exiles P1's card; P1 can return either card, each to its
    // owner's hand.
    let mut t = TestGame::new(2);
    let broker = t.battlefield(P0, BROKER);
    let elves = t.hand(P0, "Llanowar Elves");
    broker_exiles(&mut t, P0, broker, elves);
    t.set_step(P1, Step::PrecombatMain);
    cast_new(&mut t, P1, "Act of Treason", &[obj(broker)]);
    t.resolve_all();
    let bears = t.hand(P1, "Grizzly Bears");
    broker_exiles(&mut t, P1, broker, bears);
    let (exiled_elves, exiled_bears) = (t.g.current(elves), t.g.current(bears));
    t.answer_choose(P1, &[obj(exiled_elves)]);
    let choices = broker_return_choices(&mut t, P1, broker);
    assert_eq!(choices.len(), 2, "both owners' cards");
    assert!(choices.contains(&obj(exiled_elves)) && choices.contains(&obj(exiled_bears)));
    assert!(
        t.in_hand(P0, "Llanowar Elves"),
        "returned to its owner's hand"
    );
    assert_eq!(t.zone(bears), Zone::Exile);
}

const JACOB: &str = "Jacob Hauken, Inspector // Hauken's Insight";

/// Hauken's Insight on P0's battlefield, having exiled `top` (the top card of P0's
/// library) in P0's upkeep; P0 is in the upkeep with an empty stack.
fn insight_exiled(t: &mut TestGame, top: &str) -> (ObjectId, ObjectId) {
    supported(JACOB);
    let insight = back_face(t, P0, JACOB);
    let card = t.library_top(P0, top);
    into_upkeep(t, P0);
    t.resolve_all();
    assert_eq!(t.zone(card), Zone::Exile);
    (insight, card)
}

#[test]
fn haukens_insight_permission_ends_when_it_leaves() {
    cr!("607.2a", "400.7");
    ruling!(
        "Jacob Hauken, Inspector // Hauken's Insight",
        "Once Hauken's Insight leaves the battlefield, the player that controlled it may no longer play the exiled cards."
    );
    let mut t = TestGame::new(2);
    let (insight, memnite) = insight_exiled(&mut t, "Memnite");
    t.set_step(P0, Step::PrecombatMain);
    assert!(!legal_cast_methods(&mut t, P0, memnite).is_empty());
    move_to(&mut t, insight, Zone::Hand(P0));
    assert!(legal_cast_methods(&mut t, P0, memnite).is_empty());
    // A new Hauken's Insight has no memory of it.
    back_face(&mut t, P0, JACOB);
    assert!(legal_cast_methods(&mut t, P0, memnite).is_empty());
}

#[test]
fn haukens_insight_lands_follow_the_timing_rules() {
    cr!("305.1", "305.2", "116.2a");
    ruling!(
        "Jacob Hauken, Inspector // Hauken's Insight",
        "Playing a card using the last ability of Hauken's Insight follows all the normal timing rules for that card."
    );
    let mut t = TestGame::new(2);
    let (_, forest) = insight_exiled(&mut t, "Forest");
    // In the upkeep: no.
    assert!(!can_play_land(&mut t, P0, forest));
    t.set_step(P0, Step::PrecombatMain);
    // After the turn's land play: no.
    let island = t.hand(P0, "Island");
    t.play_land(P0, island).expect("land");
    assert!(!can_play_land(&mut t, P0, forest));
    // In a main phase with a land play left and an empty stack: yes.
    let mut t = TestGame::new(2);
    let (_, forest) = insight_exiled(&mut t, "Forest");
    t.set_step(P0, Step::PrecombatMain);
    assert!(can_play_land(&mut t, P0, forest));
}

const JACE: &str = "Jace, Vryn's Prodigy // Jace, Telepath Unbound";

/// Jace, Telepath Unbound's −3 targeting `card` in P0's graveyard has resolved.
fn jace_minus_three(t: &mut TestGame, card: ObjectId) {
    supported(JACE);
    let jace = back_face(t, P0, JACE);
    let jace = t.g.current(jace);
    t.g.objects[jace.0 as usize]
        .counters
        .insert(counters::LOYALTY.into(), 5);
    t.answer_targets(P0, &[obj(card)]);
    activate_containing(t, P0, jace, "You may cast target").expect("−3");
    t.resolve_all();
}

#[test]
fn jace_minus_three_follows_timing_and_costs() {
    cr!("601.2", "307.1", "608.2b");
    ruling!(
        "Jace, Vryn's Prodigy // Jace, Telepath Unbound",
        "If you activate the second ability of Jace, Telepath Unbound, you must follow the timing rules for the card's types."
    );
    ruling!(
        "Jace, Vryn's Prodigy // Jace, Telepath Unbound",
        "If you don't cast the card that turn, nothing happens. It remains in your graveyard."
    );
    let mut t = TestGame::new(2);
    let divination = t.graveyard(P0, "Divination");
    jace_minus_three(&mut t, divination);
    // No mana: it can't be cast; outside a main phase neither.
    assert!(legal_cast_methods(&mut t, P0, divination).is_empty());
    t.lands(P0, "Island", 3);
    assert!(!legal_cast_methods(&mut t, P0, divination).is_empty());
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(legal_cast_methods(&mut t, P0, divination).is_empty());
    // Not cast: next turn it's still in the graveyard and can't be cast.
    t.advance_to(P0, Step::PrecombatMain);
    assert_eq!(t.zone(divination), Zone::Graveyard(P0));
    t.lands(P0, "Island", 3);
    assert!(legal_cast_methods(&mut t, P0, divination).is_empty());
}

#[test]
fn jace_minus_three_exiles_only_the_spell_cast_from_the_graveyard() {
    cr!("614.1a", "400.7");
    ruling!(
        "Jace, Vryn's Prodigy // Jace, Telepath Unbound",
        "The card is exiled only if it's cast from the graveyard and put back into the graveyard (either by resolving or being countered)."
    );
    // Cast and resolved: exiled.
    let mut t = TestGame::new(2);
    let divination = t.graveyard(P0, "Divination");
    jace_minus_three(&mut t, divination);
    t.lands(P0, "Island", 3);
    t.cast(P0, divination).go();
    t.resolve_all();
    assert_eq!(t.zone(divination), Zone::Exile);
    // Returned to hand by Remand, then discarded: the effect lost track of it.
    let mut t = TestGame::new(2);
    let divination = t.graveyard(P0, "Divination");
    jace_minus_three(&mut t, divination);
    t.lands(P0, "Island", 3);
    let spell = t.cast(P0, divination).go();
    cast_new(&mut t, P1, "Remand", &[obj(spell)]);
    t.resolve();
    assert_eq!(t.zone(divination), Zone::Hand(P0));
    let in_hand = t.g.current(divination);
    discard(&mut t, P0, in_hand);
    assert_eq!(t.zone(divination), Zone::Graveyard(P0));
}

#[test]
fn jace_vryns_prodigy_checks_after_the_discard() {
    cr!("608.2c", "702.35a");
    ruling!(
        "Jace, Vryn's Prodigy // Jace, Telepath Unbound",
        "If you discard a card with madness while resolving the ability of Jace, Vryn's Prodigy, you'll need to already have five other cards in your graveyard to satisfy that ability's condition."
    );
    ruling!(
        "Jace, Vryn's Prodigy // Jace, Telepath Unbound",
        "The activated ability of Jace, Vryn's Prodigy checks to see if there are five or more cards in your graveyard after you discard a card."
    );
    // Five cards in the graveyard as Jace enters: he doesn't transform.
    let mut t = TestGame::new(2);
    fill_graveyard(&mut t, P0, 5);
    let jace = enter(&mut t, P0, JACE);
    t.resolve_all();
    assert_eq!(name_of(&t, jace), "Jace, Vryn's Prodigy");
    // Four cards, discarding a madness card (exiled by madness): no.
    let mut t = TestGame::new(2);
    fill_graveyard(&mut t, P0, 4);
    let jace = t.battlefield(P0, JACE);
    let temper = t.hand(P0, "Fiery Temper");
    t.library_top(P0, "Island");
    discard_these(&mut t, P0, &[temper]);
    t.activate(P0, jace, 0, &[]).expect("loot");
    t.resolve();
    assert_eq!(t.zone(temper), Zone::Exile);
    assert_eq!(t.graveyard_size(P0), 4);
    assert_eq!(name_of(&t, jace), "Jace, Vryn's Prodigy");
    // Four cards, discarding another card: the fifth; he transforms.
    let mut t = TestGame::new(2);
    fill_graveyard(&mut t, P0, 4);
    let jace = t.battlefield(P0, JACE);
    t.library_top(P0, "Island");
    t.activate(P0, jace, 0, &[]).expect("loot");
    t.resolve_all();
    assert_eq!(name_of(&t, jace), "Jace, Telepath Unbound");
}

const ARCADE: &str = "Arcade Gannon";

/// Arcade Gannon with `n` quest counters.
fn arcade(t: &mut TestGame, n: u32) -> ObjectId {
    supported(ARCADE);
    let a = t.battlefield(P0, ARCADE);
    if n > 0 {
        put_counters(t, a, "quest", n);
    }
    a
}

#[test]
fn arcade_gannon_discarded_artifact_can_be_cast_right_away() {
    cr!("117.3b", "601.2");
    ruling!(
        "Arcade Gannon",
        "If an artifact or Human card is put into your graveyard during your main phase and the stack is empty, you have a chance to cast it before any player may attempt to remove that card from your graveyard."
    );
    let mut t = TestGame::new(2);
    let a = arcade(&mut t, 0);
    let memnite = t.hand(P0, "Memnite");
    t.library_top(P0, "Island");
    discard_these(&mut t, P0, &[memnite]);
    t.activate(P0, a, 0, &[]).expect("loot");
    // Run the game: after the ability resolves, P0 (the active player) gets priority.
    let ok = t.g.run_until(1000, |g| {
        g.stack.is_empty() && g.turn.priority.is_some() && g.player(P0).graveyard.len() == 1
    });
    assert!(ok);
    assert_eq!(t.g.turn.priority, Some(P0));
    assert_eq!(t.counters(a, "quest"), 1);
    assert!(!legal_cast_methods(&mut t, P0, memnite).is_empty());
}

#[test]
fn arcade_gannon_other_permissions_and_new_arcades() {
    cr!("601.2", "601.3", "400.7");
    ruling!(
        "Arcade Gannon",
        "If you cast an artifact or Human spell from your graveyard using another permission, Arcade Gannon’s effect doesn’t apply."
    );
    ruling!(
        "Arcade Gannon",
        "If you cast one artifact or Human spell from your graveyard and then have a new Arcade Gannon come under your control in the same turn, you may cast another artifact or Human spell from your graveyard that turn"
    );
    supported("Emry, Lurker of the Loch");
    let mut t = TestGame::new(2);
    let old = arcade(&mut t, 1);
    let emry = t.battlefield(P0, "Emry, Lurker of the Loch");
    let thopter = t.graveyard(P0, "Ornithopter");
    let memnite = t.graveyard(P0, "Memnite");
    let vanguard = t.graveyard(P0, "Elite Vanguard");
    let bears = t.graveyard(P0, "Grizzly Bears");
    t.activate(P0, emry, 0, &[obj(thopter)]).expect("Emry");
    t.resolve_all();
    t.answer_choose(P0, &[obj(emry)]);
    t.cast(P0, thopter).go();
    t.resolve_all();
    // Arcade's permission is unused: Memnite (an artifact, mana value 0).
    t.cast(P0, memnite).go();
    t.resolve_all();
    assert!(t.on_battlefield(t.g.current(memnite)));
    // Used: Elite Vanguard (a Human, mana value 1) can't be cast too...
    t.lands(P0, "Plains", 1);
    assert!(legal_cast_methods(&mut t, P0, vanguard).is_empty());
    // ...until a new Arcade Gannon with a quest counter (the old one has died: the legend
    // rule).
    destroy(&mut t, old);
    arcade(&mut t, 1);
    assert!(!legal_cast_methods(&mut t, P0, vanguard).is_empty());
    // Grizzly Bears is neither an artifact nor a Human.
    assert!(legal_cast_methods(&mut t, P0, bears).is_empty());
}

#[test]
fn arcade_gannon_losing_it_while_casting_doesnt_matter() {
    cr!("601.2h", "601.3");
    ruling!(
        "Arcade Gannon",
        "Once you begin to cast the spell, losing control of Arcade Gannon won’t affect the spell."
    );
    supported("Dusk Rose Reliquary");
    let mut t = TestGame::new(2);
    let a = arcade(&mut t, 2);
    let reliquary = t.graveyard(P0, "Dusk Rose Reliquary");
    t.lands(P0, "Plains", 1);
    t.answer_choose(P0, &[obj(a)]);
    let spell = t.cast(P0, reliquary).method(CastMethod::Normal).go();
    assert!(t.in_graveyard(P0, ARCADE), "sacrificed as a cost");
    t.resolve_all();
    assert!(t.on_battlefield(spell));
}
