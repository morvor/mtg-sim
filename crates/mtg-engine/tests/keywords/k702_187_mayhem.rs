//! CR 702.187 Mayhem (`src/kw/mayhem.rs`).

use crate::common_k702_178_195::*;
use mtg_engine::ability::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const MAYHEM: CastMethod = CastMethod::Keyword(KeywordKind::Mayhem);

fn untapped_lands(t: &TestGame) -> usize {
    t.g.permanents()
        .filter(|o| o.is(mtg_engine::types::CardType::Land) && !o.tapped)
        .count()
}

/// `p` discards `card` from their hand; returns the card in the graveyard.
fn discard(t: &mut TestGame, p: PlayerId, card: ObjectId) -> ObjectId {
    let new = t.g.discard(p, card, None).expect("discarded");
    t.g.flush_events();
    new
}

#[test]
fn mayhem_cards_compile() {
    assert_supported(&[
        "Electro's Bolt",
        "Spider-Islanders",
        "Swarm, Being of Bees",
        "Sandman's Quicksand",
        "Oscorp Industries",
        "Raging Goblinoids",
    ]);
}

#[test]
fn cast_from_the_graveyard_for_the_mayhem_cost_if_discarded_this_turn() {
    cr!("702.187a", "702.187b");
    ruling!(
        "Electro's Bolt",
        "Mayhem represents an alternative cost that can be paid to cast a spell from your graveyard if you discarded the card with mayhem that turn."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 3);
    let wurm = t.battlefield(P1, "Craw Wurm");
    let bolt = t.hand(P0, "Electro's Bolt");
    // In hand, mayhem doesn't function.
    assert!(t.cast(P0, bolt).method(MAYHEM).target(wurm).try_go().is_err());
    t.clear_answers();
    let bolt = discard(&mut t, P0, bolt);
    let spell = t.cast(P0, bolt).method(MAYHEM).target(wurm).go();
    // {1}{R} rather than {2}{R}; its mana value is unchanged.
    assert_eq!(untapped_lands(&t), 1);
    assert_eq!(t.g.mana_value_of(spell), 3);
    t.resolve_all();
    assert!(!t.on_battlefield(wurm));
    // Like any instant or sorcery, it goes to the graveyard.
    assert!(t.in_graveyard(P0, "Electro's Bolt"));
}

#[test]
fn only_a_card_discarded_this_turn() {
    cr!("702.187b");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 6);
    let wurm = t.battlefield(P1, "Craw Wurm");
    // Put into the graveyard some other way: not discarded.
    let milled = t.graveyard(P0, "Electro's Bolt");
    assert!(t
        .cast(P0, milled)
        .method(MAYHEM)
        .target(wurm)
        .try_go()
        .is_err());
    t.clear_answers();
    // Discarded on an earlier turn.
    let bolt = t.hand(P0, "Electro's Bolt");
    let bolt = discard(&mut t, P0, bolt);
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    assert!(t.cast(P0, bolt).method(MAYHEM).target(wurm).try_go().is_err());
    t.clear_answers();
    // Discarded this turn, but it left the graveyard and came back (not discarded): a
    // new object.
    let bolt2 = t.hand(P0, "Electro's Bolt");
    let bolt2 = discard(&mut t, P0, bolt2);
    for to in [ZoneKind::Hand, ZoneKind::Graveyard] {
        let cur = now(&t, bolt2);
        run(
            &mut t,
            P0,
            None,
            Effect::Move {
                what: Sel::Target(0),
                to: Destination::zone(to),
            },
            &[Entity::Object(cur)],
        );
    }
    let again = now(&t, bolt2);
    assert_eq!(t.zone(again), Zone::Graveyard(P0));
    assert!(t.cast(P0, again).method(MAYHEM).target(wurm).try_go().is_err());
}

#[test]
fn timing_rules_still_apply() {
    cr!("702.187b");
    ruling!(
        "Spider-Islanders",
        "you can't cast a creature card with mayhem you couldn't normally cast during an opponent's turn, even if you discard that card during an opponent's turn"
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 2);
    t.lands(P0, "Swamp", 1);
    t.set_step(P1, Step::PrecombatMain);
    let islanders = t.hand(P0, "Spider-Islanders");
    let islanders = discard(&mut t, P0, islanders);
    assert!(t.cast(P0, islanders).method(MAYHEM).try_go().is_err());
    // Swarm, Being of Bees has flash: it can be cast then.
    let swarm = t.hand(P0, "Swarm, Being of Bees");
    let swarm = discard(&mut t, P0, swarm);
    t.cast(P0, swarm).method(MAYHEM).go();
    t.resolve_all();
    assert_eq!(named(&t, "Swarm, Being of Bees").len(), 1);
    // On its own turn, in a main phase, the Islanders could be cast; it's a new turn now,
    // so it wasn't discarded this turn.
    t.advance_to(P0, Step::PrecombatMain);
    assert!(t.cast(P0, islanders).method(MAYHEM).try_go().is_err());
}

#[test]
fn additional_costs_are_still_paid() {
    cr!("702.187b");
    ruling!(
        "Spider-Islanders",
        "You must pay any additional costs the spell has."
    );
    let def = custom_card(
        "Costly Mayhem",
        "{3}{R}",
        "Sorcery",
        None,
        "As an additional cost to cast this spell, pay 3 life.\nYou draw two cards.\nMayhem {R}",
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 1);
    let c = put(&mut t, P0, def, Zone::Hand(P0));
    let c = discard(&mut t, P0, c);
    let hand = t.hand_size(P0);
    t.cast(P0, c).method(MAYHEM).go();
    assert_eq!(t.life(P0), 17);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
}

#[test]
fn if_the_mayhem_cost_was_paid() {
    cr!("702.187b");
    // Sandman's Quicksand: "All creatures get -2/-2 until end of turn. If this spell's
    // mayhem cost was paid, creatures your opponents control get -2/-2 until end of turn
    // instead."
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 4);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let c = t.hand(P0, "Sandman's Quicksand");
    let c = discard(&mut t, P0, c);
    t.cast(P0, c).method(MAYHEM).go();
    t.resolve_all();
    assert!(t.on_battlefield(mine));
    assert!(!t.on_battlefield(theirs));
    // Cast for its mana cost, it affects all creatures.
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 3);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let c = t.hand(P0, "Sandman's Quicksand");
    t.cast(P0, c).go();
    t.resolve_all();
    assert!(!t.on_battlefield(mine));
    assert!(!t.on_battlefield(theirs));
}

#[test]
fn mayhem_without_a_cost_lets_a_discarded_land_be_played() {
    cr!("702.187c");
    ruling!(
        "Oscorp Industries",
        "You can play Oscorp Industries using its mayhem ability only if you have available land plays remaining."
    );
    let mut t = TestGame::new(2);
    let land = t.hand(P0, "Oscorp Industries");
    let land = discard(&mut t, P0, land);
    assert!(t
        .g
        .legal_actions(P0)
        .iter()
        .any(|a| matches!(a, mtg_engine::decision::Action::PlayLand { card } if *card == land)));
    t.play_land(P0, land).unwrap();
    t.resolve_all();
    let oscorp = named(&t, "Oscorp Industries")[0];
    assert!(t.obj(oscorp).tapped);
    // "When this land enters from a graveyard, you lose 2 life."
    assert_eq!(t.life(P0), 18);
    // Without a land play left, another discarded one can't be played.
    let land2 = t.hand(P0, "Oscorp Industries");
    let land2 = discard(&mut t, P0, land2);
    assert!(t.play_land(P0, land2).is_err());
    // Nor one that wasn't discarded.
    let mut t = TestGame::new(2);
    let milled = t.graveyard(P0, "Oscorp Industries");
    assert!(t.play_land(P0, milled).is_err());
}
