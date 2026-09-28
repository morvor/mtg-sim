//! Rulings batch S23 — casting a spell "without paying its mana cost" (CR 118.9, 601.2b,
//! 601.2f) and with other alternative costs (flashback, escape): no other alternative cost
//! can be applied, optional additional costs (kicker) may be paid, and mandatory
//! additional costs must be paid; an {X} in the mana cost is 0 (CR 107.3b).

use crate::r_s01_common::*;
use crate::r_s04_common::{asked_of_since, spell_targets, stack_items};
use crate::r_s05_common::enter;
use crate::r_s07_common::{cast_methods, graveyard_n};
use crate::r_s23_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

const FLASHBACK: CastMethod = CastMethod::Keyword(KeywordKind::Flashback);
const ESCAPE: CastMethod = CastMethod::Keyword(KeywordKind::Escape);
const OVERLOAD: CastMethod = CastMethod::Keyword(KeywordKind::Overload);
const DASH: CastMethod = CastMethod::Keyword(KeywordKind::Dash);

/// Whether a "Choose how to cast it" / casting-method decision was asked of `p` since
/// `from` (a choice among alternative costs).
fn casting_way_asked(t: &TestGame, p: PlayerId, from: usize) -> bool {
    asked_of_since(t, p, from, |d| {
        matches!(d, Decision::ChooseCastingMethod { .. })
            || matches!(d, Decision::ChooseOption { prompt, .. } if prompt.contains("how to cast"))
    }) > 0
}

/// Wildfire Eternal attacks P1 and isn't blocked; its trigger resolves (with the answers
/// queued) and combat ends.
fn wildfire_unblocked(t: &mut TestGame) {
    let eternal = t.battlefield(P0, "Wildfire Eternal");
    attack_with(t, &[(eternal, Entity::Player(P1))]);
    block_and_finish(t, P1, &[]);
}

#[test]
fn wildfire_eternal_free_spell_pays_mandatory_and_optional_additional_costs() {
    cr!("118.9", "118.8", "118.8b", "601.2b", "601.2f");
    ruling!(
        "Wildfire Eternal",
        "If you cast a card \"without paying its mana cost,\" you can't pay any alternative costs. You can, however, pay additional costs. If the card has any mandatory additional costs, such as that of Tormenting Voice, those must be paid to cast the card."
    );
    supported("Wildfire Eternal");
    supported("Tormenting Voice");
    // Tormenting Voice: "As an additional cost to cast this spell, discard a card."
    let mut t = TestGame::new(2);
    let voice = t.hand(P0, "Tormenting Voice");
    let forest = t.hand(P0, "Forest");
    t.answer_choose(P0, &[Entity::Object(voice)]);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(forest)]);
    let library = t.library_size(P0);
    wildfire_unblocked(&mut t);
    assert!(t.in_graveyard(P0, "Tormenting Voice"));
    assert!(t.in_graveyard(P0, "Forest"));
    assert_eq!(t.library_size(P0), library - 2);
    // Without another card to discard, it can't be cast.
    let mut t = TestGame::new(2);
    let voice = t.hand(P0, "Tormenting Voice");
    t.answer_choose(P0, &[Entity::Object(voice)]);
    t.answer_yes(P0, true);
    let library = t.library_size(P0);
    wildfire_unblocked(&mut t);
    assert!(t.in_hand(P0, "Tormenting Voice"));
    assert_eq!(t.library_size(P0), library);
    // Kicker, an optional additional cost, may be paid: Burst Lightning deals 4.
    let mut t = TestGame::new(2);
    t.lands(P0, "Wastes", 4);
    let burst = t.hand(P0, "Burst Lightning");
    t.answer_choose(P0, &[Entity::Object(burst)]);
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.answer_targets(P0, &[Entity::Player(P1)]);
    wildfire_unblocked(&mut t);
    assert_eq!(tapped_lands(&t, P0), 4);
    // 4 from Burst Lightning, 1 combat damage.
    assert_eq!(t.life(P1), 15);
}

#[test]
fn torrential_gearhulk_free_spell_cant_be_overloaded_but_must_sacrifice_for_sabotage() {
    cr!("118.9", "118.9a", "118.8", "601.2b", "601.2f");
    ruling!(
        "Torrential Gearhulk",
        "If you cast a card \"without paying its mana cost,\" you can't choose to cast it for any alternative costs, such as emerge costs. You can, however, pay additional costs. If the card has any mandatory additional costs, such as that of Incendiary Sabotage, you must pay those to cast the card."
    );
    supported("Torrential Gearhulk");
    supported("Incendiary Sabotage");
    supported("Cyclonic Rift");
    // Incendiary Sabotage: "As an additional cost to cast this spell, sacrifice an
    // artifact. Incendiary Sabotage deals 3 damage to each creature."
    let mut t = TestGame::new(2);
    let bottle = t.battlefield(P0, "Bottle Gnomes");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let sabotage = t.graveyard(P0, "Incendiary Sabotage");
    t.answer_targets(P0, &[Entity::Object(sabotage)]);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(bottle)]);
    enter(&mut t, P0, "Torrential Gearhulk");
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Bottle Gnomes"));
    assert!(!t.on_battlefield(bears));
    // "If that spell would be put into your graveyard, exile it instead."
    assert!(t.in_exile("Incendiary Sabotage"));
    // With no other artifact, Torrential Gearhulk (an artifact creature) itself must be
    // sacrificed to cast it.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let sabotage = t.graveyard(P0, "Incendiary Sabotage");
    t.answer_targets(P0, &[Entity::Object(sabotage)]);
    t.answer_yes(P0, true);
    let gearhulk = enter(&mut t, P0, "Torrential Gearhulk");
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert!(!t.on_battlefield(gearhulk));
    assert!(t.in_graveyard(P0, "Torrential Gearhulk"));
    // Cyclonic Rift's overload cost is an alternative cost: it's cast for its normal
    // text, returning one target permanent only.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let elves = t.battlefield(P1, "Llanowar Elves");
    t.lands(P0, "Island", 7);
    let rift = t.graveyard(P0, "Cyclonic Rift");
    t.answer_targets(P0, &[Entity::Object(rift)]);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    let from = t.asked().len();
    enter(&mut t, P0, "Torrential Gearhulk");
    t.resolve_all();
    assert!(!casting_way_asked(&t, P0, from));
    assert!(t.in_hand(P1, "Grizzly Bears"));
    assert!(t.on_battlefield(elves));
    assert_eq!(tapped_lands(&t, P0), 0);
}

#[test]
fn yue_free_spell_with_x_has_x_zero() {
    cr!("107.3b", "118.9", "601.2f");
    ruling!(
        "Yue, the Moon Spirit",
        "If a card has {X} in its mana cost, you must choose 0 as the value of X when casting it without paying its mana cost."
    );
    supported("Yue, the Moon Spirit");
    supported("Blaze");
    // "Waterbend {5}, {T}: You may cast a noncreature spell from your hand without paying
    // its mana cost."
    let mut t = TestGame::new(2);
    let yue = t.battlefield(P0, "Yue, the Moon Spirit");
    t.lands(P0, "Wastes", 5);
    let blaze = t.hand(P0, "Blaze");
    // (No creatures are tapped to help pay the waterbend cost.)
    t.answer_choose(P0, &[]);
    t.answer_choose(P0, &[Entity::Object(blaze)]);
    t.answer_yes(P0, true);
    // Even an answer of 5 for X isn't used: X is 0.
    t.answer(P0, DecisionKind::X, Answer::Number(5));
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.activate(P0, yue, 0, &[]).unwrap();
    assert_eq!(t.life(P1), 20);
    t.resolve();
    let items = stack_items(&t);
    assert_eq!(items, vec!["Blaze".to_string()]);
    // No value of X was chosen: it's 0.
    assert_eq!(
        asked_of_since(&t, P0, 0, |d| matches!(d, Decision::ChooseX { .. })),
        0
    );
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    assert!(t.in_graveyard(P0, "Blaze"));
}

#[test]
fn a_spell_cast_with_flashback_cant_be_overloaded_but_can_be_kicked() {
    cr!("702.34a", "118.9a", "118.8", "601.2b", "601.2f");
    ruling!(
        "Slickshot Lockpicker",
        "If you cast a spell with flashback, you can’t pay any alternative costs such as overload costs. You can pay additional costs such as kicker costs. If the spell has any mandatory additional costs, you must pay those to cast the spell with flashback."
    );
    supported("Slickshot Lockpicker");
    // Slickshot Lockpicker: "When this creature enters, target instant or sorcery card in
    // your graveyard gains flashback until end of turn. The flashback cost is equal to
    // its mana cost."
    // Cyclonic Rift with flashback: it can't also be cast for its overload cost.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Grizzly Bears");
    let rift = t.graveyard(P0, "Cyclonic Rift");
    t.answer_targets(P0, &[Entity::Object(rift)]);
    enter(&mut t, P0, "Slickshot Lockpicker");
    t.resolve_all();
    t.lands(P0, "Island", 7);
    let methods = cast_methods(&mut t, P0, rift);
    assert_eq!(methods, vec![FLASHBACK]);
    assert!(!methods.contains(&OVERLOAD));
    // Burst Lightning with flashback may be kicked.
    let mut t = TestGame::new(2);
    let burst = t.graveyard(P0, "Burst Lightning");
    t.answer_targets(P0, &[Entity::Object(burst)]);
    enter(&mut t, P0, "Slickshot Lockpicker");
    t.resolve_all();
    t.lands(P0, "Mountain", 5);
    t.cast(P0, burst)
        .method(FLASHBACK)
        .kicked(true)
        .target(Entity::Player(P1))
        .go();
    assert_eq!(tapped_lands(&t, P0), 5);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    assert!(t.in_exile("Burst Lightning"));
    // Tormenting Voice with flashback: its discard must be paid.
    let mut t = TestGame::new(2);
    let voice = t.graveyard(P0, "Tormenting Voice");
    t.answer_targets(P0, &[Entity::Object(voice)]);
    enter(&mut t, P0, "Slickshot Lockpicker");
    t.resolve_all();
    t.lands(P0, "Mountain", 2);
    assert!(t.cast(P0, voice).method(FLASHBACK).try_go().is_err());
    let island = t.hand(P0, "Island");
    t.answer_choose(P0, &[Entity::Object(island)]);
    t.cast(P0, voice).method(FLASHBACK).go();
    assert!(t.in_graveyard(P0, "Island"));
    t.resolve_all();
    assert!(t.in_exile("Tormenting Voice"));
}

#[test]
fn a_spell_cast_with_escape_from_confession_dial_cant_be_dashed_but_can_be_kicked() {
    cr!("702.138a", "118.9a", "118.8", "601.2b", "601.2f");
    ruling!(
        "Confession Dial",
        "If you cast a spell with its escape permission, you can't choose to apply any other alternative costs or to cast it without paying its mana cost. You may pay any optional additional costs the spell has, such as kicker costs. If it has any mandatory additional costs, you must pay those."
    );
    supported("Confession Dial");
    supported("Zurgo Bellstriker");
    supported("Tourach, Dread Cantor");
    // Confession Dial: "{T}: Target legendary creature card in your graveyard gains
    // escape until end of turn. The escape cost is equal to its mana cost plus exile
    // three other cards from your graveyard."
    fn escape_with_dial(t: &mut TestGame, name: &str) -> ObjectId {
        let dial = t.battlefield(P0, "Confession Dial");
        let card = t.graveyard(P0, name);
        graveyard_n(t, P0, "Grizzly Bears", 3);
        t.activate(P0, dial, 0, &[Entity::Object(card)]).unwrap();
        t.resolve_all();
        card
    }
    // Zurgo Bellstriker (dash {1}{R}) with escape: dash can't be applied.
    let mut t = TestGame::new(2);
    let zurgo = escape_with_dial(&mut t, "Zurgo Bellstriker");
    t.lands(P0, "Mountain", 2);
    let methods = cast_methods(&mut t, P0, zurgo);
    assert_eq!(methods, vec![ESCAPE]);
    assert!(!methods.contains(&DASH));
    // Tourach, Dread Cantor (kicker {B}{B}) with escape may be kicked.
    let mut t = TestGame::new(2);
    let tourach = escape_with_dial(&mut t, "Tourach, Dread Cantor");
    t.lands(P0, "Swamp", 4);
    t.hand(P1, "Forest");
    t.hand(P1, "Island");
    t.cast(P0, tourach).method(ESCAPE).kicked(true).go();
    assert_eq!(tapped_lands(&t, P0), 4);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve_all();
    assert!(t.on_battlefield(tourach));
    // "When Tourach enters, if it was kicked, target opponent discards two cards at
    // random."
    assert!(!t.in_hand(P1, "Forest") && !t.in_hand(P1, "Island"));
    // A mandatory additional cost must be paid: a legendary creature card with "As an
    // additional cost to cast this spell, sacrifice a creature."
    let mut t = TestGame::new(2);
    let dial = t.battlefield(P0, "Confession Dial");
    let lord = t.custom(
        P0,
        custom_card(
            "Test Legendary Lord",
            "Legendary Creature — Demon",
            "{B}",
            Some((5, 5)),
            "As an additional cost to cast this spell, sacrifice a creature.",
        ),
        mtg_engine::object::Zone::Graveyard(P0),
    );
    graveyard_n(&mut t, P0, "Grizzly Bears", 3);
    t.activate(P0, dial, 0, &[Entity::Object(lord)]).unwrap();
    t.resolve_all();
    t.lands(P0, "Swamp", 1);
    assert!(t.cast(P0, lord).method(ESCAPE).try_go().is_err());
    let elves = t.battlefield(P0, "Llanowar Elves");
    t.answer_choose(P0, &[Entity::Object(elves)]);
    t.cast(P0, lord).method(ESCAPE).go();
    assert!(t.in_graveyard(P0, "Llanowar Elves"));
    t.resolve_all();
    assert!(t.on_battlefield(lord));
}

/// P0 casts Vadrok, Apex of Thunder for its mutate cost onto Grizzly Bears, merging on
/// top; its mutate trigger targets `card` in P0's graveyard, P0 casts it (`then` queues
/// the answers for casting it). Everything resolves.
fn vadrok_mutates(t: &mut TestGame, card: ObjectId, then: impl FnOnce(&mut TestGame)) {
    use mtg_engine::mana::ManaType;
    let bears = t.battlefield(P0, "Grizzly Bears");
    let vadrok = t.hand(P0, "Vadrok, Apex of Thunder");
    // Mutate {1}{W/U}{R}{R}.
    crate::r_s04_common::add_mana(t, P0, ManaType::W, 1);
    crate::r_s04_common::add_mana(t, P0, ManaType::R, 2);
    crate::r_s04_common::add_mana(t, P0, ManaType::C, 1);
    // {W/U} paid with {W}; the mutating spell goes on top.
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.cast(P0, vadrok)
        .method(CastMethod::Keyword(KeywordKind::Mutate))
        .target(bears)
        .go();
    t.answer_targets(P0, &[Entity::Object(card)]);
    t.answer_yes(P0, true);
    then(t);
    t.resolve_all();
}

#[test]
fn vadrok_free_spell_can_be_kicked_must_pay_additional_costs_and_cant_be_overloaded() {
    cr!("702.140a", "118.9", "118.9a", "118.8", "601.2b", "601.2f");
    ruling!(
        "Vadrok, Apex of Thunder",
        "If you cast a spell “without paying its mana cost,” you can’t choose to cast it for any alternative costs. You can, however, pay additional costs, such as kicker costs. If the card has any mandatory additional costs, such as that of Swallow Whole, those must be paid to cast the card."
    );
    supported("Vadrok, Apex of Thunder");
    // "Whenever this creature mutates, you may cast target noncreature card with mana
    // value 3 or less from your graveyard without paying its mana cost."
    // Burst Lightning may be kicked.
    let mut t = TestGame::new(2);
    t.lands(P0, "Wastes", 4);
    let burst = t.graveyard(P0, "Burst Lightning");
    vadrok_mutates(&mut t, burst, |t| {
        t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
        t.answer_targets(P0, &[Entity::Player(P1)]);
    });
    assert_eq!(t.life(P1), 16);
    assert_eq!(tapped_lands(&t, P0), 4);
    // Tormenting Voice's discard must be paid.
    let mut t = TestGame::new(2);
    let voice = t.graveyard(P0, "Tormenting Voice");
    let island = t.hand(P0, "Island");
    let library = t.library_size(P0);
    vadrok_mutates(&mut t, voice, |t| {
        t.answer_choose(P0, &[Entity::Object(island)]);
    });
    assert!(t.in_graveyard(P0, "Island"));
    assert_eq!(t.library_size(P0), library - 2);
    // Cyclonic Rift isn't overloaded: only its target returns to its owner's hand.
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 7);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let elves = t.battlefield(P1, "Llanowar Elves");
    let rift = t.graveyard(P0, "Cyclonic Rift");
    let from = t.asked().len();
    vadrok_mutates(&mut t, rift, |t| {
        t.answer_targets(P0, &[Entity::Object(bears)]);
    });
    assert!(!casting_way_asked(&t, P0, from));
    assert!(t.in_hand(P1, "Grizzly Bears"));
    assert!(t.on_battlefield(elves));
    assert_eq!(tapped_lands(&t, P0), 0);
}

#[test]
fn an_adventurer_or_split_card_with_escape_is_cast_as_the_chosen_part_for_its_cost() {
    cr!("702.138a", "715.3", "709.3", "601.2b", "601.2f");
    ruling!(
        "Underworld Breach",
        "If you're casting an adventurer card or split card with escape, you choose how you wish to cast it, then pay the appropriate cost (for the Adventure, the creature, or the half of the split card you chose) plus exiling three cards."
    );
    supported("Underworld Breach");
    supported("Bonecrusher Giant");
    supported("Fire // Ice");
    // Underworld Breach: "Each nonland card in your graveyard has escape. The escape cost
    // is equal to the card's mana cost plus exile three other cards from your graveyard."
    // Bonecrusher Giant ({2}{R}) // Stomp ({1}{R}): cast as Stomp for {1}{R}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Underworld Breach");
    t.lands(P0, "Mountain", 3);
    let giant = t.graveyard(P0, "Bonecrusher Giant");
    graveyard_n(&mut t, P0, "Grizzly Bears", 3);
    assert_eq!(cast_methods(&mut t, P0, giant), vec![ESCAPE, ESCAPE]);
    let from = t.asked().len();
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.cast(P0, giant)
        .method(ESCAPE)
        .target(Entity::Player(P1))
        .go();
    let offered: Vec<Vec<String>> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseCastingMethod { options, .. } => Some(options.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(
        offered,
        vec![vec![
            "escape: Bonecrusher Giant".to_string(),
            "escape: Stomp".to_string()
        ]]
    );
    assert_eq!(tapped_lands(&t, P0), 2);
    assert_eq!(t.g.exile.len(), 3);
    assert_eq!(stack_items(&t), vec!["Stomp".to_string()]);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    // Cast as an Adventure, it's exiled as it resolves.
    assert!(t.in_exile("Bonecrusher Giant"));
    // Cast as the creature: {2}{R} plus exiling three cards.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Underworld Breach");
    t.lands(P0, "Mountain", 3);
    let giant = t.graveyard(P0, "Bonecrusher Giant");
    graveyard_n(&mut t, P0, "Grizzly Bears", 3);
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.cast(P0, giant).method(ESCAPE).go();
    assert_eq!(tapped_lands(&t, P0), 3);
    assert_eq!(t.g.exile.len(), 3);
    t.resolve_all();
    assert!(t.on_battlefield(giant));
    // A split card: Fire // Ice cast as Ice ({1}{U}).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Underworld Breach");
    t.lands(P0, "Island", 2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let fire_ice = t.graveyard(P0, "Fire // Ice");
    graveyard_n(&mut t, P0, "Llanowar Elves", 3);
    let hand = t.hand_size(P0);
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.cast(P0, fire_ice)
        .method(ESCAPE)
        .target(Entity::Object(bears))
        .go();
    assert_eq!(tapped_lands(&t, P0), 2);
    t.resolve_all();
    assert!(t.obj_now(bears).tapped);
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn omniscience_free_spells_have_x_zero_and_pay_additional_costs() {
    cr!("107.3b", "118.9", "118.8", "601.2b", "601.2f");
    ruling!(
        "Omniscience",
        "If a spell has {X} in its mana cost, you must choose 0 as the value of X when casting it without paying its mana cost."
    );
    ruling!(
        "Omniscience",
        "If you cast a spell \"without paying its mana cost,\" you can't choose to cast it for any alternative costs. You can, however, pay additional costs, such as kicker costs. If the card has any mandatory additional costs, such as that of Tormenting Voice, those must be paid to cast the spell."
    );
    let free = CastMethod::Free;
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Omniscience");
    // Blaze ({X}{R}) cast for free: X is 0, whatever is answered.
    let blaze = t.hand(P0, "Blaze");
    t.cast(P0, blaze)
        .method(free.clone())
        .x(3)
        .target(Entity::Player(P1))
        .go();
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    // Tormenting Voice's discard must be paid: with no other card in hand, it can't be
    // cast.
    let voice = t.hand(P0, "Tormenting Voice");
    assert!(t.cast(P0, voice).method(free.clone()).try_go().is_err());
    let forest = t.hand(P0, "Forest");
    t.answer_choose(P0, &[Entity::Object(forest)]);
    t.cast(P0, voice).method(free.clone()).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Forest"));
    // Kicker may be paid: Burst Lightning deals 4.
    t.lands(P0, "Wastes", 4);
    let burst = t.hand(P0, "Burst Lightning");
    t.cast(P0, burst)
        .method(free.clone())
        .kicked(true)
        .target(Entity::Player(P1))
        .go();
    assert_eq!(tapped_lands(&t, P0), 4);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    // An alternative cost can't be combined with it: Fireblast is cast either for free or
    // by sacrificing two Mountains, not both.
    let mountains = t.lands(P0, "Mountain", 2);
    let fireblast = t.hand(P0, "Fireblast");
    t.cast(P0, fireblast)
        .method(free)
        .target(Entity::Player(P1))
        .go();
    t.resolve_all();
    assert!(mountains.iter().all(|m| t.on_battlefield(*m)));
    assert_eq!(t.life(P1), 12);
}
