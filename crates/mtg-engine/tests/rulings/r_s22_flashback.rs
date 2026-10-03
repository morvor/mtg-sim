//! Rulings batch S22 — flashback granted by an effect (Snapcaster Mage, Past in Flames:
//! "The flashback cost is equal to its mana cost", CR 702.34a): X is still chosen and
//! paid (CR 107.3), a card with no mana cost can't be cast this way (CR 118.6), a split
//! card's flashback cost is the mana cost of the half cast (CR 709.3), and flashback is an
//! alternative cost: no other alternative cost can be paid, but additional costs can or
//! must be (CR 118.9a, 118.8).

use crate::r_s01_common::*;
use crate::r_s07_common::cast_methods;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::*;

const FLASHBACK: CastMethod = CastMethod::Keyword(KeywordKind::Flashback);
const OVERLOAD: CastMethod = CastMethod::Keyword(KeywordKind::Overload);

/// Snapcaster Mage enters under P0's control, its ability targeting `card` in P0's
/// graveyard ("target instant or sorcery card in your graveyard gains flashback until end
/// of turn. The flashback cost is equal to its mana cost.").
fn snapcaster(t: &mut TestGame, card: ObjectId) {
    t.answer_targets(P0, &[Entity::Object(card)]);
    t.enter(P0, "Snapcaster Mage");
    t.resolve_all();
    t.clear_answers();
}

#[test]
fn snapcaster_mage_x_is_still_chosen_and_paid() {
    cr!("702.34a", "107.3", "601.2b", "601.2f");
    ruling!(
        "Snapcaster Mage",
        "If you cast an instant or sorcery with {X} in its mana cost this way, you still choose the value of X as part of casting the spell and pay that cost."
    );
    supported("Snapcaster Mage");
    let mut t = TestGame::new(2);
    let blaze = t.graveyard(P0, "Blaze");
    snapcaster(&mut t, blaze);
    t.lands(P0, "Mountain", 5);
    // Flashback {X}{R} with X = 3: four Mountains; 3 damage.
    t.cast(P0, blaze)
        .method(FLASHBACK)
        .x(3)
        .target(Entity::Player(P1))
        .go();
    assert_eq!(tapped_lands(&t, P0), 4);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert!(t.in_exile("Blaze"));
}

#[test]
fn snapcaster_mage_a_card_with_no_mana_cost_cant_be_flashed_back() {
    cr!("702.34a", "118.6", "202.1b");
    ruling!(
        "Snapcaster Mage",
        "If a card with no mana cost gains flashback, it has no flashback cost. It can't be cast this way."
    );
    supported("Ancestral Vision");
    // Ancestral Vision (a sorcery with no mana cost, suspend 4) gains flashback: it can't
    // be cast with it.
    let mut t = TestGame::new(2);
    let vision = t.graveyard(P0, "Ancestral Vision");
    snapcaster(&mut t, vision);
    t.lands(P0, "Island", 5);
    assert!(t.obj_now(vision).chars.has_keyword(KeywordKind::Flashback));
    assert!(!cast_methods(&mut t, P0, vision).contains(&FLASHBACK));
    assert!(t.cast(P0, vision).method(FLASHBACK).try_go().is_err());
    assert_eq!(t.zone(vision), Zone::Graveyard(P0));
}

#[test]
fn snapcaster_mage_a_split_card_is_flashed_back_for_the_half_cast() {
    cr!("702.34a", "709.3", "709.3a");
    ruling!(
        "Snapcaster Mage",
        "If a split card gains flashback, you pay only the cost of the half you're casting."
    );
    supported("Fire // Ice");
    // Fire // Ice: Fire {1}{R} ("Fire deals 2 damage divided as you choose among one or two
    // targets.") // Ice {1}{U} ("Tap target permanent. Draw a card."). Flashback Fire: two
    // mana, not four.
    let mut t = TestGame::new(2);
    let fi = t.graveyard(P0, "Fire // Ice");
    snapcaster(&mut t, fi);
    t.lands(P0, "Mountain", 2);
    let opts: Vec<_> = t
        .g
        .cast_options(P0, t.g.current(fi))
        .into_iter()
        .filter(|o| o.method == FLASHBACK)
        .collect();
    assert_eq!(opts.len(), 2, "one way per half");
    // The first is Fire.
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    let spell = t
        .cast(P0, fi)
        .method(FLASHBACK)
        .target(Entity::Player(P1))
        .go();
    assert_eq!(t.obj(spell).chars.name, "Fire");
    assert_eq!(tapped_lands(&t, P0), 2);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert!(t.in_exile("Fire // Ice"));
    // Ice, for {1}{U}.
    let mut t = TestGame::new(2);
    let fi = t.graveyard(P0, "Fire // Ice");
    snapcaster(&mut t, fi);
    t.lands(P0, "Island", 2);
    let hand = t.hand_size(P0);
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    let bears = t.battlefield(P1, "Grizzly Bears");
    let spell = t
        .cast(P0, fi)
        .method(FLASHBACK)
        .target(Entity::Object(bears))
        .go();
    assert_eq!(t.obj(spell).chars.name, "Ice");
    t.resolve_all();
    assert!(t.obj_now(bears).tapped);
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn past_in_flames_flashback_excludes_alternative_costs_but_not_additional_costs() {
    cr!("702.34a", "118.9a", "118.8", "601.2b");
    ruling!(
        "Past in Flames",
        "If you cast a spell with flashback, you can't pay any alternative costs such as overload costs. You can pay additional costs such as kicker costs. If the spell has any mandatory additional costs, you must pay those to cast the spell with flashback."
    );
    supported("Past in Flames");
    // "Each instant and sorcery card in your graveyard gains flashback until end of turn.
    // The flashback cost is equal to its mana cost."
    let past = |t: &mut TestGame| {
        t.lands(P0, "Mountain", 4);
        let p = t.hand(P0, "Past in Flames");
        t.cast(P0, p).go();
        t.resolve_all();
    };
    // Cyclonic Rift can be flashed back only for {1}{U}, not overloaded.
    let mut t = TestGame::new(2);
    let rift = t.graveyard(P0, "Cyclonic Rift");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    past(&mut t);
    t.lands(P0, "Island", 7);
    let methods = cast_methods(&mut t, P0, rift);
    assert!(methods.contains(&FLASHBACK));
    assert!(!methods.contains(&OVERLOAD));
    t.cast(P0, rift)
        .method(FLASHBACK)
        .target(Entity::Object(bears))
        .go();
    t.resolve_all();
    assert!(t.in_hand(P1, "Grizzly Bears"));
    assert!(t.on_battlefield(giant));
    assert!(t.in_exile("Cyclonic Rift"));
    // Kicker can be paid: kicked Burst Lightning deals 4.
    let mut t = TestGame::new(2);
    let bolt = t.graveyard(P0, "Burst Lightning");
    past(&mut t);
    t.lands(P0, "Mountain", 5);
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.cast(P0, bolt)
        .method(FLASHBACK)
        .target(Entity::Player(P1))
        .go();
    assert_eq!(tapped_lands(&t, P0), 4 + 5);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    // Village Rites' sacrifice must be paid: without a creature it can't be cast.
    let mut t = TestGame::new(2);
    let rites = t.graveyard(P0, "Village Rites");
    past(&mut t);
    t.lands(P0, "Swamp", 1);
    assert!(t.cast(P0, rites).method(FLASHBACK).try_go().is_err());
    t.battlefield(P0, "Grizzly Bears");
    t.cast(P0, rites).method(FLASHBACK).go();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
}
