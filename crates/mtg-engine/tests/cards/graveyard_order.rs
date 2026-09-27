//! Abilities of cards in a graveyard: triggered abilities with an intervening "if this
//! card is in your graveyard" clause function from the graveyard (CR 113.6, 603.4), and
//! conditions about the cards above a card in its graveyard (CR 404.1).

use mtg_engine::ability::AbilityKind;
use mtg_engine::card::card;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn assert_supported(name: &str) {
    let c = card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        c.unsupported_text()
    );
}

/// Puts real cards into `p`'s graveyard in order (the last one on top).
fn bury(t: &mut TestGame, p: PlayerId, names: &[&str]) -> Vec<ObjectId> {
    names.iter().map(|n| t.graveyard(p, n)).collect()
}

/// Moves to `p`'s next upkeep (from P0's first main phase) and puts its triggers on the
/// stack.
fn to_upkeep(t: &mut TestGame, p: PlayerId) {
    if p == P0 {
        t.advance_to(P1, Step::Upkeep);
    }
    t.advance_to(p, Step::Upkeep);
    t.settle();
}

#[test]
fn pyre_zombie_returns_from_the_graveyard_at_upkeep() {
    cr!("113.6", "603.4");
    assert_supported("Pyre Zombie");
    // "At the beginning of your upkeep, if this card is in your graveyard, you may pay
    // {1}{B}{B}. If you do, return it to your hand."
    let mut t = TestGame::new(2);
    let z = t.graveyard(P0, "Pyre Zombie");
    t.lands(P0, "Swamp", 3);
    t.answer_yes(P0, true);
    to_upkeep(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.zone(z), Zone::Hand(P0));
    // On the battlefield it doesn't trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Pyre Zombie");
    t.lands(P0, "Swamp", 3);
    to_upkeep(&mut t, P0);
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn the_intervening_if_is_checked_again_on_resolution() {
    cr!("603.4");
    let mut t = TestGame::new(2);
    let z = t.graveyard(P0, "Pyre Zombie");
    t.lands(P0, "Swamp", 3);
    to_upkeep(&mut t, P0);
    assert_eq!(t.stack_len(), 1);
    // The card leaves the graveyard with the ability on the stack: it does nothing.
    let z = t.g.current(z);
    t.g.move_object(
        z,
        Zone::Exile,
        mtg_engine::events::MoveCause::Effect,
        None,
    );
    t.g.flush_events();
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(t.in_exile("Pyre Zombie"));
    assert_eq!(untapped(&t, P0), 3);
}

fn untapped(t: &TestGame, p: PlayerId) -> usize {
    t.g.permanents()
        .filter(|o| o.controller == p && o.chars.is_land() && !o.tapped)
        .count()
}

#[test]
fn bridge_from_below_triggers_from_the_graveyard() {
    cr!("113.6", "603.4");
    assert_supported("Bridge from Below");
    let mut t = TestGame::new(2);
    let bridge = t.graveyard(P0, "Bridge from Below");
    // A nontoken creature of P0's dies: a Zombie token.
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.destroy(bears, None);
    t.resolve_all();
    let zombies = t
        .g
        .permanents()
        .filter(|o| o.is_token() && o.chars.has_subtype("Zombie"))
        .count();
    assert_eq!(zombies, 1);
    // A creature of P1's dies: Bridge from Below is exiled.
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.g.destroy(theirs, None);
    t.resolve_all();
    assert_eq!(t.zone(bridge), Zone::Exile);
}

#[test]
fn thunderblade_charge_is_cast_from_the_graveyard_after_combat_damage() {
    cr!("113.6", "603.4", "118.9");
    assert_supported("Thunderblade Charge");
    // "Whenever one or more creatures you control deal combat damage to a player, if this
    // card is in your graveyard, you may pay {2}{R}{R}{R}. If you do, you may cast it
    // without paying its mana cost."
    let mut t = TestGame::new(2);
    let charge = t.graveyard(P0, "Thunderblade Charge");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Mountain", 5);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(bears, Entity::Player(P1))], &[]);
    t.resolve_all();
    // 2 combat damage and 3 from the Charge.
    assert_eq!(t.life(P1), 15);
    assert_eq!(t.g.history.spells_cast.len(), 1);
    assert_eq!(t.zone(charge), Zone::Graveyard(P0));
    assert_eq!(untapped(&t, P0), 0);
}

#[test]
fn blood_operative_returns_when_you_surveil() {
    cr!("113.6", "603.4");
    assert_supported("Blood Operative");
    // "Whenever you surveil, if this card is in your graveyard, you may pay 3 life. If you
    // do, return this card to your hand."
    let mut t = TestGame::new(2);
    let op = t.graveyard(P0, "Blood Operative");
    t.lands(P0, "Island", 1);
    let opt = t.hand(P0, "Consider");
    t.answer_yes(P0, true);
    t.cast(P0, opt).go();
    t.resolve_all();
    assert_eq!(t.zone(op), Zone::Hand(P0));
    assert_eq!(t.life(P0), 17);
}

#[test]
fn glory_is_activated_from_the_graveyard() {
    cr!("113.6", "602.5b");
    assert_supported("Glory");
    let mut t = TestGame::new(2);
    let on_bf = t.battlefield(P0, "Glory");
    t.lands(P0, "Plains", 3);
    t.g.recompute();
    t.g.turn.priority = Some(P0);
    let can = |t: &mut TestGame, id: ObjectId| {
        t.g.legal_actions(P0).iter().any(|a| {
            matches!(a, mtg_engine::decision::Action::Activate { source, .. } if *source == id)
        })
    };
    assert!(!can(&mut t, on_bf));
    let in_gy = t.graveyard(P0, "Glory");
    t.g.recompute();
    assert!(can(&mut t, in_gy));
    assert!(t
        .g
        .obj(in_gy)
        .chars
        .abilities
        .iter()
        .any(|a| matches!(a.kind, AbilityKind::Activated(_))));
}

#[test]
fn nether_shadow_needs_three_creature_cards_above_it() {
    cr!("404.1", "603.4");
    assert_supported("Nether Shadow");
    // "At the beginning of your upkeep, if this card is in your graveyard with three or
    // more creature cards above it, you may put this card onto the battlefield."
    let mut t = TestGame::new(2);
    bury(&mut t, P0, &["Hill Giant", "Hill Giant"]);
    let shadow = t.graveyard(P0, "Nether Shadow");
    bury(&mut t, P0, &["Grizzly Bears", "Lightning Bolt", "Llanowar Elves"]);
    // Only two creature cards are above it.
    to_upkeep(&mut t, P0);
    assert_eq!(t.stack_len(), 0);
    // A third one is put into the graveyard later.
    t.graveyard(P0, "Serra Angel");
    t.answer_yes(P0, true);
    to_upkeep(&mut t, P0);
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert!(t.on_battlefield(shadow));
}

#[test]
fn ashen_ghoul_is_activated_during_upkeep_with_three_creature_cards_above_it() {
    cr!("404.1", "602.5b");
    assert_supported("Ashen Ghoul");
    // "{B}: Return this card from your graveyard to the battlefield. Activate only during
    // your upkeep and only if three or more creature cards are above this card."
    let mut t = TestGame::new(2);
    let ghoul = t.graveyard(P0, "Ashen Ghoul");
    bury(&mut t, P0, &["Grizzly Bears", "Hill Giant", "Llanowar Elves"]);
    t.lands(P0, "Swamp", 1);
    let can = |t: &mut TestGame| {
        t.g.recompute();
        t.g.turn.priority = Some(P0);
        let g = t.g.current(ghoul);
        t.g.legal_actions(P0).iter().any(|a| {
            matches!(a, mtg_engine::decision::Action::Activate { source, .. } if *source == g)
        })
    };
    // Not in the main phase.
    assert!(!can(&mut t));
    to_upkeep(&mut t, P0);
    assert!(can(&mut t));
    t.g.recompute();
    let uid = t
        .g
        .obj(ghoul)
        .chars
        .abilities
        .iter()
        .find(|a| matches!(a.kind, AbilityKind::Activated(_)))
        .map(|a| a.uid)
        .unwrap();
    t.g.activate_ability(P0, ghoul, uid).expect("activate");
    t.resolve_all();
    assert!(t.on_battlefield(ghoul));
    // With only two creature cards above it, it can't be activated.
    let mut t = TestGame::new(2);
    let ghoul2 = t.graveyard(P0, "Ashen Ghoul");
    bury(&mut t, P0, &["Grizzly Bears", "Lightning Bolt", "Hill Giant"]);
    t.graveyard(P0, "Llanowar Elves");
    // Three creature cards, but one is below it now: move the Ghoul up.
    let gy = &mut t.g.players[0].graveyard;
    gy.retain(|o| *o != ghoul2);
    gy.insert(1, ghoul2);
    t.lands(P0, "Swamp", 1);
    to_upkeep(&mut t, P0);
    t.g.recompute();
    t.g.turn.priority = Some(P0);
    assert!(!t.g.legal_actions(P0).iter().any(|a| {
        matches!(a, mtg_engine::decision::Action::Activate { source, .. } if *source == ghoul2)
    }));
}

#[test]
fn krovikan_horror_needs_a_creature_card_directly_above_it() {
    cr!("404.1", "603.4");
    assert_supported("Krovikan Horror");
    assert_supported("Death Spark");
    // "At the beginning of the end step, if this card is in your graveyard with a creature
    // card directly above it, you may return this card to your hand."
    let mut t = TestGame::new(2);
    let horror = t.graveyard(P0, "Krovikan Horror");
    t.graveyard(P0, "Lightning Bolt");
    t.graveyard(P0, "Grizzly Bears");
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 0);
    let mut t = TestGame::new(2);
    let horror2 = t.graveyard(P0, "Krovikan Horror");
    t.graveyard(P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.zone(horror2), Zone::Hand(P0));
    let _ = horror;
}
