//! Rulings batch S23 — casting a spell "without paying its mana cost" (CR 118.9, 601.2b,
//! 601.2f) and with other alternative costs (flashback, escape): no other alternative cost
//! can be applied, optional additional costs (kicker) may be paid, and mandatory
//! additional costs must be paid; an {X} in the mana cost is 0 (CR 107.3b).

use crate::r_s01_common::*;
use crate::r_s04_common::{asked_of_since, stack_items};
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

/// P1 casts the real card `theirs`; P0 counters it with Reinterpret, whose free-cast
/// answers `then` queues. Everything resolves.
fn reinterpret(t: &mut TestGame, theirs: &str, then: impl FnOnce(&mut TestGame)) {
    t.set_step(P1, Step::PrecombatMain);
    give_mana_for(t, P1, theirs);
    let card = t.hand(P1, theirs);
    let spell = t.cast(P1, card).go();
    give_mana_for(t, P0, "Reinterpret");
    let r = t.hand(P0, "Reinterpret");
    t.cast(P0, r).target(Entity::Object(spell)).go();
    then(t);
    t.resolve_all();
    assert!(t.in_graveyard(P1, theirs));
}

#[test]
fn reinterpret_free_spell_can_be_kicked_must_pay_additional_costs_and_cant_be_evoked() {
    cr!("118.9", "118.9a", "118.8", "601.2b", "601.2f", "601.3e");
    ruling!(
        "Reinterpret",
        "If you cast a spell “without paying its mana cost,” you can’t pay any alternative costs. You can, however, pay additional costs, such as kicker costs. If the card has any mandatory additional costs, you must pay those."
    );
    supported("Reinterpret");
    // "Counter target spell. You may cast a spell with equal or lesser mana value from your
    // hand without paying its mana cost." Hill Giant (mana value 4) is countered.
    // Burst Lightning may be kicked.
    let mut t = TestGame::new(2);
    let burst = t.hand(P0, "Burst Lightning");
    t.lands(P0, "Wastes", 4);
    reinterpret(&mut t, "Hill Giant", |t| {
        t.answer_choose(P0, &[Entity::Object(burst)]);
        t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
        t.answer_targets(P0, &[Entity::Player(P1)]);
    });
    assert_eq!(t.life(P1), 16);
    // Tormenting Voice's discard must be paid.
    let mut t = TestGame::new(2);
    let voice = t.hand(P0, "Tormenting Voice");
    let forest = t.hand(P0, "Forest");
    reinterpret(&mut t, "Hill Giant", |t| {
        t.answer_choose(P0, &[Entity::Object(voice)]);
        t.answer_choose(P0, &[Entity::Object(forest)]);
    });
    assert!(t.in_graveyard(P0, "Forest"));
    assert!(t.in_graveyard(P0, "Tormenting Voice"));
    // Mulldrifter (mana value 5) after countering a mana value 5 spell: not for its evoke
    // cost, so it isn't sacrificed.
    let mut t = TestGame::new(2);
    let drifter = t.hand(P0, "Mulldrifter");
    reinterpret(&mut t, "Air Elemental", |t| {
        t.answer_choose(P0, &[Entity::Object(drifter)]);
    });
    assert!(t.on_battlefield(drifter));
    // A spell with greater mana value can't be cast: Mulldrifter after Hill Giant.
    let mut t = TestGame::new(2);
    let drifter = t.hand(P0, "Mulldrifter");
    let from = t.asked().len();
    reinterpret(&mut t, "Hill Giant", |_| {});
    assert!(t.in_hand(P0, "Mulldrifter"));
    assert!(t.asked()[from..].iter().all(|(_, d)| !matches!(
        d,
        mtg_engine::decision::Decision::ChooseEntities { candidates, .. }
            if candidates.contains(&Entity::Object(drifter))
    )));
}

/// Hidetsugu and Kairi (P0's, on the battlefield) dies with `top` on top of P0's library;
/// its trigger targets P1, and `then` queues the answers for casting the card. Everything
/// resolves.
fn hidetsugu_dies(t: &mut TestGame, top: &str, then: impl FnOnce(&mut TestGame)) {
    let hk = t.battlefield(P0, "Hidetsugu and Kairi");
    t.library_top(P0, top);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    then(t);
    crate::r_s02_common::destroy(t, hk);
    t.resolve_all();
}

#[test]
fn hidetsugu_and_kairi_free_card_pays_additional_costs_but_no_alternative_cost() {
    cr!("118.9", "118.9a", "118.8", "601.2b", "601.2f", "608.2g");
    ruling!(
        "Hidetsugu and Kairi",
        "If you cast a card “without paying its mana cost,” you can’t pay any alternative costs. You can, however, pay additional costs. If the card has any mandatory additional costs, those must be paid to cast the card."
    );
    ruling!(
        "Hidetsugu and Kairi",
        "You choose whether or not to cast the instant or sorcery card as the last triggered ability resolves."
    );
    supported("Hidetsugu and Kairi");
    // "When Hidetsugu and Kairi dies, exile the top card of your library. Target opponent
    // loses life equal to its mana value. If it's an instant or sorcery card, you may cast
    // it without paying its mana cost."
    // Burst Lightning (mana value 1) may be kicked.
    let mut t = TestGame::new(2);
    t.lands(P0, "Wastes", 4);
    hidetsugu_dies(&mut t, "Burst Lightning", |t| {
        t.answer_yes(P0, true);
        t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
        t.answer_targets(P0, &[Entity::Player(P1)]);
    });
    assert_eq!(t.life(P1), 20 - 1 - 4);
    assert_eq!(tapped_lands(&t, P0), 4);
    // Tormenting Voice's discard must be paid.
    let mut t = TestGame::new(2);
    let forest = t.hand(P0, "Forest");
    hidetsugu_dies(&mut t, "Tormenting Voice", |t| {
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[Entity::Object(forest)]);
    });
    assert_eq!(t.life(P1), 18);
    assert!(t.in_graveyard(P0, "Forest"));
    assert!(t.in_graveyard(P0, "Tormenting Voice"));
    // Fireblast (mana value 6) is cast for free, not by sacrificing two Mountains.
    let mut t = TestGame::new(2);
    let mountains = t.lands(P0, "Mountain", 2);
    let from = t.asked().len();
    hidetsugu_dies(&mut t, "Fireblast", |t| {
        t.answer_yes(P0, true);
        t.answer_targets(P0, &[Entity::Player(P1)]);
    });
    assert_eq!(t.life(P1), 20 - 6 - 4);
    assert!(mountains.iter().all(|m| t.on_battlefield(*m)));
    assert!(!casting_way_asked(&t, P0, from));
    // Declined as the ability resolves, the card stays in exile: it can't be cast later.
    let mut t = TestGame::new(2);
    hidetsugu_dies(&mut t, "Lightning Bolt", |t| {
        t.answer_yes(P0, false);
    });
    assert!(t.in_exile("Lightning Bolt"));
    let bolt =
        t.g.find_in_zone(mtg_engine::object::Zone::Exile, "Lightning Bolt")[0];
    t.lands(P0, "Mountain", 1);
    assert!(crate::r_s08_common::legal_cast_methods(&mut t, P0, bolt).is_empty());
}

/// P0's Djinn of Wishes (entered with its wish counters) activates with `top` on top of
/// P0's library; `then` queues the answers for playing it. Everything resolves.
fn wish(t: &mut TestGame, top: &str, then: impl FnOnce(&mut TestGame)) {
    let djinn = enter(t, P0, "Djinn of Wishes");
    t.resolve_all();
    assert_eq!(t.counters(djinn, "wish"), 3);
    t.library_top(P0, top);
    t.lands(P0, "Island", 4);
    then(t);
    t.activate(P0, djinn, 0, &[]).unwrap();
    t.resolve_all();
}

#[test]
fn djinn_of_wishes_free_card_pays_additional_costs_but_no_alternative_cost() {
    cr!("118.9", "118.9a", "118.8", "601.2b", "601.2f", "608.2g");
    ruling!(
        "Djinn of Wishes",
        "If you cast a spell \"without paying its mana cost,\" you can't choose to cast it for any alternative costs. You can, however, pay additional costs, such as kicker costs. If the card has any mandatory additional costs, such as that of Tormenting Voice, those must be paid to cast the card."
    );
    supported("Djinn of Wishes");
    // "{2}{U}{U}, Remove a wish counter from this creature: Reveal the top card of your
    // library. You may play that card without paying its mana cost. If you don't, exile
    // it."
    // Burst Lightning may be kicked.
    let mut t = TestGame::new(2);
    t.lands(P0, "Wastes", 4);
    wish(&mut t, "Burst Lightning", |t| {
        t.answer_yes(P0, true);
        t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
        t.answer_targets(P0, &[Entity::Player(P1)]);
    });
    assert_eq!(t.life(P1), 16);
    assert!(t.in_graveyard(P0, "Burst Lightning"));
    // Tormenting Voice's discard must be paid.
    let mut t = TestGame::new(2);
    let forest = t.hand(P0, "Forest");
    wish(&mut t, "Tormenting Voice", |t| {
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[Entity::Object(forest)]);
    });
    assert!(t.in_graveyard(P0, "Forest"));
    assert!(t.in_graveyard(P0, "Tormenting Voice"));
    // Mulldrifter isn't cast for its evoke cost: it stays on the battlefield.
    let mut t = TestGame::new(2);
    let from = t.asked().len();
    wish(&mut t, "Mulldrifter", |t| {
        t.answer_yes(P0, true);
    });
    assert_eq!(t.named_on_battlefield("Mulldrifter").len(), 1);
    assert!(!casting_way_asked(&t, P0, from));
    // Not played: it's exiled.
    let mut t = TestGame::new(2);
    wish(&mut t, "Lightning Bolt", |t| {
        t.answer_yes(P0, false);
    });
    assert!(t.in_exile("Lightning Bolt"));
}

/// P0 casts Stolen Goods targeting P1, whose library has `top` (a nonland card) under a
/// Forest; returns the exiled card.
fn stolen_goods(t: &mut TestGame, top: &str) -> ObjectId {
    stack_library(t, P1, &["Forest", top]);
    give_mana_for(t, P0, "Stolen Goods");
    let goods = t.hand(P0, "Stolen Goods");
    t.cast(P0, goods).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert!(t.in_exile("Forest"));
    t.g.find_in_zone(mtg_engine::object::Zone::Exile, top)[0]
}

#[test]
fn stolen_goods_free_spell_pays_additional_costs_but_no_alternative_cost() {
    cr!("118.9", "118.9a", "118.8", "601.2b", "601.2f");
    ruling!(
        "Stolen Goods",
        "If you cast a card \"without paying its mana cost,\" you can't pay any alternative costs. You can pay additional costs, such as kicker costs. If the card has mandatory additional costs, you must pay those."
    );
    supported("Stolen Goods");
    // "Target opponent exiles cards from the top of their library until they exile a
    // nonland card. Until end of turn, you may cast that card without paying its mana
    // cost."
    // P1's Burst Lightning may be kicked.
    let mut t = TestGame::new(2);
    let burst = stolen_goods(&mut t, "Burst Lightning");
    t.lands(P0, "Wastes", 4);
    t.cast(P0, burst)
        .method(CastMethod::Free)
        .kicked(true)
        .target(Entity::Player(P1))
        .go();
    assert_eq!(tapped_lands(&t, P0), 4 + 4);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    // Tormenting Voice's discard must be paid.
    let mut t = TestGame::new(2);
    let voice = stolen_goods(&mut t, "Tormenting Voice");
    assert!(t.cast(P0, voice).method(CastMethod::Free).try_go().is_err());
    let island = t.hand(P0, "Island");
    t.answer_choose(P0, &[Entity::Object(island)]);
    t.cast(P0, voice).method(CastMethod::Free).go();
    assert!(t.in_graveyard(P0, "Island"));
    // Mulldrifter cast without paying its mana cost isn't evoked: it stays on the
    // battlefield.
    let mut t = TestGame::new(2);
    let drifter = stolen_goods(&mut t, "Mulldrifter");
    assert!(
        crate::r_s08_common::legal_cast_methods(&mut t, P0, drifter).contains(&CastMethod::Free)
    );
    let spell = t.cast(P0, drifter).method(CastMethod::Free).go();
    assert!(!t
        .obj(spell)
        .cast
        .as_deref()
        .is_some_and(|c| c.paid.iter().any(|p| p == "evoke")));
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Mulldrifter").len(), 1);
    // The permission lasts until end of turn.
    let mut t = TestGame::new(2);
    let drifter = stolen_goods(&mut t, "Mulldrifter");
    t.advance_to(P1, Step::Upkeep);
    t.set_step(P0, Step::PrecombatMain);
    assert!(
        !crate::r_s08_common::legal_cast_methods(&mut t, P0, drifter).contains(&CastMethod::Free)
    );
}

/// Narset, Enlightened Master attacks P1 with `top` on top of P0's library (the top four
/// cards are exiled), and combat ends; returns the exiled cards named `top`.
fn narset_attacks(t: &mut TestGame, top: &[&str]) -> Vec<ObjectId> {
    let narset = t.battlefield(P0, "Narset, Enlightened Master");
    stack_library(t, P0, top);
    attack_with(t, &[(narset, Entity::Player(P1))]);
    block_and_finish(t, P1, &[]);
    top.iter()
        .map(|n| t.g.find_in_zone(mtg_engine::object::Zone::Exile, n)[0])
        .collect()
}

#[test]
fn narset_free_spells_have_x_zero_and_follow_timing_rules() {
    cr!("107.3b", "118.9", "601.3", "307.1");
    ruling!(
        "Narset, Enlightened Master",
        "If the card has {X} in its mana cost, you must choose 0 as the value for X when casting it."
    );
    ruling!(
        "Narset, Enlightened Master",
        "You must follow all applicable timing rules. For example, if one of the exiled cards is a sorcery card, you can cast it only during your main phase while the stack is empty."
    );
    ruling!(
        "Narset, Enlightened Master",
        "You can't play any land cards exiled with Narset."
    );
    supported("Narset, Enlightened Master");
    // "Whenever Narset attacks, exile the top four cards of your library. Until end of
    // turn, you may cast noncreature spells from among those cards without paying their
    // mana costs."
    let mut t = TestGame::new(2);
    let cards = narset_attacks(&mut t, &["Blaze", "Grizzly Bears", "Forest", "Opt"]);
    let (blaze, bears, forest, opt) = (cards[0], cards[1], cards[2], cards[3]);
    // During combat, the sorcery can't be cast; the instant can.
    let methods = |t: &mut TestGame, c| crate::r_s08_common::legal_cast_methods(t, P0, c);
    assert!(methods(&mut t, blaze).is_empty());
    assert_eq!(methods(&mut t, opt), vec![CastMethod::Free]);
    // Neither the creature card nor the land can be played.
    assert!(methods(&mut t, bears).is_empty());
    assert!(!crate::r_s02_common::can_play_land(&mut t, P0, forest));
    // In the postcombat main phase, Blaze is cast for free with X = 0.
    t.advance_to(P0, Step::PostcombatMain);
    assert!(!crate::r_s02_common::can_play_land(&mut t, P0, forest));
    let life = t.life(P1);
    t.cast(P0, blaze)
        .method(CastMethod::Free)
        .x(3)
        .target(Entity::Player(P1))
        .go();
    t.resolve_all();
    assert_eq!(t.life(P1), life);
    assert!(t.in_graveyard(P0, "Blaze"));
}

/// P1, the active player, casts `spell` (with X = `x`, targeting `target` if given) while
/// P0 controls Powerbalance with `top` on top of their library; the trigger resolves with
/// the answers queued.
fn powerbalance_trigger(
    t: &mut TestGame,
    spell: &str,
    x: Option<i64>,
    target: Option<Entity>,
    top: &str,
) -> ObjectId {
    t.battlefield(P0, "Powerbalance");
    let top = t.library_top(P0, top);
    t.set_step(P1, Step::PrecombatMain);
    give_mana_for(t, P1, spell);
    t.lands(P1, "Wastes", x.unwrap_or(0) as usize);
    let card = t.hand(P1, spell);
    let mut c = t.cast(P1, card);
    if let Some(x) = x {
        c = c.x(x);
    }
    if let Some(e) = target {
        c = c.target(e);
    }
    c.go();
    // The trigger resolves.
    t.resolve();
    top
}

#[test]
fn powerbalance_free_spell_may_be_kicked_and_pays_mandatory_additional_costs() {
    cr!("118.9", "118.9a", "118.8", "601.2b", "601.2f", "608.2g");
    ruling!(
        "Powerbalance",
        "If you cast a spell \"without paying its mana cost,\" you can't choose to cast it for any alternative costs. You can, however, pay additional costs, such as kicker costs. If the spell has any mandatory additional costs, those must be paid to cast it."
    );
    ruling!(
        "Powerbalance",
        "You choose whether or not to cast the exiled card as Powerbalance's triggered ability resolves. If you do, you do so as part of the resolution of that ability. You can't wait to cast it later in the turn. Timing restrictions based on the card's type are ignored."
    );
    supported("Powerbalance");
    // "Whenever an opponent casts a spell, you may reveal the top card of your library.
    // If you do, you may cast that card without paying its mana cost if the two spells
    // have the same mana value."
    // Burst Lightning (mana value 1) cast kicked for {4} in response to Lightning Bolt.
    let mut t = TestGame::new(2);
    t.lands(P0, "Wastes", 4);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.answer_targets(P0, &[Entity::Player(P1)]);
    powerbalance_trigger(
        &mut t,
        "Lightning Bolt",
        None,
        Some(Entity::Player(P0)),
        "Burst Lightning",
    );
    t.resolve_all();
    assert_eq!(tapped_lands(&t, P0), 4);
    assert_eq!(t.life(P1), 16);
    assert!(t.in_graveyard(P0, "Burst Lightning"));
    // Tormenting Voice (mana value 2), a sorcery, in response to Grizzly Bears during
    // P1's turn: a card is discarded to cast it.
    let mut t = TestGame::new(2);
    let forest = t.hand(P0, "Forest");
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(forest)]);
    powerbalance_trigger(&mut t, "Grizzly Bears", None, None, "Tormenting Voice");
    assert!(t.in_graveyard(P0, "Forest"));
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Tormenting Voice"));
    // Without a card to discard, it can't be cast.
    let mut t = TestGame::new(2);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    powerbalance_trigger(&mut t, "Grizzly Bears", None, None, "Tormenting Voice");
    t.resolve_all();
    assert_eq!(
        t.g.player(P0)
            .library
            .last()
            .map(|c| t.g.obj(*c).chars.name.as_str()),
        Some("Tormenting Voice")
    );
    // Cyclonic Rift (mana value 2) can't be cast for its overload cost.
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 7);
    let elves = t.battlefield(P1, "Llanowar Elves");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(elves)]);
    let from = t.asked().len();
    powerbalance_trigger(&mut t, "Grizzly Bears", None, None, "Cyclonic Rift");
    t.resolve_all();
    assert!(!casting_way_asked(&t, P0, from));
    assert!(t.in_hand(P1, "Llanowar Elves"));
    assert!(t.on_battlefield(bears));
    assert_eq!(tapped_lands(&t, P0), 0);
}

#[test]
fn powerbalance_compares_the_x_chosen_for_the_opponents_spell() {
    cr!("202.3e", "107.3b", "118.9", "608.2g");
    ruling!(
        "Powerbalance",
        "If an opponent casts a spell with {X} in its mana cost, use the value of X that was chosen when it was cast to determine its mana value."
    );
    ruling!(
        "Powerbalance",
        "If the revealed card has {X} in its mana cost, you must choose 0 as the value of X when casting it without paying its mana cost."
    );
    supported("Powerbalance");
    // Blaze with X = 1 has mana value 2: Grizzly Bears can be cast (during P1's turn).
    let mut t = TestGame::new(2);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    powerbalance_trigger(
        &mut t,
        "Blaze",
        Some(1),
        Some(Entity::Player(P0)),
        "Grizzly Bears",
    );
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    // Blaze with X = 2 has mana value 3: it can't.
    let mut t = TestGame::new(2);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    powerbalance_trigger(
        &mut t,
        "Blaze",
        Some(2),
        Some(Entity::Player(P0)),
        "Grizzly Bears",
    );
    t.resolve_all();
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
    // A revealed Blaze has mana value 1 (X is 0) and is cast with X = 0 in response to a
    // mana value 1 spell: it deals no damage.
    let mut t = TestGame::new(2);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    powerbalance_trigger(
        &mut t,
        "Lightning Bolt",
        None,
        Some(Entity::Player(P0)),
        "Blaze",
    );
    assert!(t.g.stack.iter().any(|s| t.g.obj(*s).chars.name == "Blaze"));
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Blaze"));
    assert_eq!(t.life(P1), 20);
}

#[test]
fn powerbalance_compares_the_spell_the_card_would_become() {
    cr!("601.3e", "718.3a", "202.3d", "709.3a");
    ruling!(
        "Powerbalance",
        "In some unusual cases, the card you reveal from the top of your library may not have the same mana value as the spell your opponent cast, but you can still cast that card because the resulting spell does have the same mana value."
    );
    ruling!(
        "Powerbalance",
        "Otherwise, while a split card is on the stack, its mana value is determined by the mana cost of the half that was chosen to be cast."
    );
    supported("Powerbalance");
    supported("Frogmyr Enforcer");
    supported("Fire // Ice");
    // Frogmyr Enforcer (mana value 7, prototype {3}{R}) in response to Hill Giant (mana
    // value 4): it's cast as a prototyped 2/2.
    let mut t = TestGame::new(2);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    powerbalance_trigger(&mut t, "Hill Giant", None, None, "Frogmyr Enforcer");
    t.resolve_all();
    let frog = t.named_on_battlefield("Frogmyr Enforcer");
    assert_eq!(frog.len(), 1);
    assert_eq!(t.pt(frog[0]), (2, 2));
    // Fire // Ice (mana value 4 in the library) in response to Grizzly Bears (mana value
    // 2): either half, a spell with mana value 2, can be cast. Ice taps the Bears.
    let mut t = TestGame::new(2);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    powerbalance_trigger(&mut t, "Grizzly Bears", None, None, "Fire // Ice");
    assert!(t.g.stack.iter().any(|s| t.g.obj(*s).chars.name == "Ice"));
    // In response to a mana value 4 spell (Hill Giant), neither half can be cast.
    let mut t = TestGame::new(2);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    powerbalance_trigger(&mut t, "Hill Giant", None, None, "Fire // Ice");
    assert!(
        !t.g.stack
            .iter()
            .any(|s| t.g.obj(*s).chars.name.contains("Fire")
                || t.g.obj(*s).chars.name.contains("Ice"))
    );
}

/// P0 controls Jodah, the Unifier with `top` on top of their library (top first) and
/// casts Sivitri Scarzam (a legendary spell with mana value 7) from their hand; Jodah's
/// trigger resolves with the answers queued.
fn jodah_trigger(t: &mut TestGame, top: &[&str]) {
    t.battlefield(P0, "Jodah, the Unifier");
    stack_library(t, P0, top);
    give_mana_for(t, P0, "Sivitri Scarzam");
    let sivitri = t.hand(P0, "Sivitri Scarzam");
    t.cast(P0, sivitri).go();
    t.resolve();
}

/// The name of the card on the bottom of `p`'s library.
fn bottom_card(t: &TestGame, p: PlayerId) -> String {
    let c = t.g.player(p).library[0];
    t.g.obj(c).chars.name.to_string()
}

#[test]
fn jodah_free_spell_may_be_kicked_but_not_dashed() {
    cr!("118.9", "118.9a", "118.8", "601.2b", "608.2g", "702.33a", "702.109a");
    ruling!(
        "Jodah, the Unifier",
        "If you cast a spell “without paying its mana cost,” you can’t choose to cast it for any alternative costs. You can, however, pay additional costs. If the card has any mandatory additional costs, you must pay those to cast the spell."
    );
    supported("Jodah, the Unifier");
    supported("Verix Bladewing");
    supported("Kolaghan, the Storm's Fury");
    // "Whenever you cast a legendary spell from your hand, exile cards from the top of
    // your library until you exile a legendary nonland card with lesser mana value. You
    // may cast that card without paying its mana cost. Put the rest on the bottom of your
    // library in a random order."
    // Verix Bladewing (mana value 4) is cast kicked for {3}: Karox Bladewing is created.
    let mut t = TestGame::new(2);
    t.lands(P0, "Wastes", 3);
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    jodah_trigger(&mut t, &["Grizzly Bears", "Verix Bladewing"]);
    assert_eq!(bottom_card(&t, P0), "Grizzly Bears");
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Verix Bladewing").len(), 1);
    assert_eq!(t.named_on_battlefield("Karox Bladewing").len(), 1);
    assert_eq!(tapped_lands(&t, P0), 10);
    // Kolaghan (mana value 5) can't be cast for its dash cost: no way to cast it is
    // offered, and it doesn't have haste.
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 5);
    t.answer_yes(P0, true);
    let from = t.asked().len();
    jodah_trigger(&mut t, &["Kolaghan, the Storm's Fury"]);
    assert!(!casting_way_asked(&t, P0, from));
    t.resolve_all();
    let kolaghan = t.named_on_battlefield("Kolaghan, the Storm's Fury");
    assert_eq!(kolaghan.len(), 1);
    assert!(!t.obj_now(kolaghan[0]).has_keyword(KeywordKind::Haste));
}

#[test]
fn jodah_card_not_cast_stays_exiled_and_the_rest_go_to_the_bottom() {
    cr!("406.1", "608.2g");
    ruling!(
        "Jodah, the Unifier",
        "If you choose not to cast the card, it remains in exile. The rest of the cards will be put on the bottom of your library in a random order."
    );
    ruling!(
        "Jodah, the Unifier",
        "The legendary nonland card exiled may be cast immediately. If you do not cast it immediately, you don’t get to cast it at a later time."
    );
    ruling!(
        "Jodah, the Unifier",
        "If you exile your entire library without exiling a legendary nonland card with lesser mana value, you will randomize the exiled cards and they again become your library, ending the effect."
    );
    supported("Jodah, the Unifier");
    // Declined: Verix Bladewing stays in exile and can't be cast later; Grizzly Bears
    // (not legendary) and a Forest (a land) go to the bottom of the library.
    let mut t = TestGame::new(2);
    t.answer_yes(P0, false);
    jodah_trigger(&mut t, &["Grizzly Bears", "Forest", "Verix Bladewing"]);
    t.resolve_all();
    let verix =
        t.g.find_in_zone(mtg_engine::object::Zone::Exile, "Verix Bladewing");
    assert_eq!(verix.len(), 1);
    assert_eq!(t.g.exile.len(), 1);
    let bottom: Vec<String> = t.g.player(P0).library[..2]
        .iter()
        .map(|c| t.g.obj(*c).chars.name.to_string())
        .collect();
    assert!(bottom.contains(&"Grizzly Bears".to_string()));
    assert!(bottom.contains(&"Forest".to_string()));
    t.lands(P0, "Mountain", 4);
    assert!(crate::r_s08_common::legal_cast_methods(&mut t, P0, verix[0]).is_empty());
    // A legendary card with an equal mana value (Sivitri Scarzam) doesn't stop it; with
    // none to find, the whole library is exiled and becomes the library again.
    let mut t = TestGame::new(2);
    let library = t.library_size(P0);
    jodah_trigger(&mut t, &["Sivitri Scarzam"]);
    t.resolve_all();
    assert!(t.g.exile.is_empty());
    assert_eq!(t.library_size(P0), library + 1);
}

/// P0's Mindleech Mass attacks P1 and isn't blocked; its trigger resolves with the
/// answers queued (P0 looks at P1's hand and chooses `pick` to cast, then the cards in
/// `discard` for its cost) and combat ends.
fn mindleech_unblocked(t: &mut TestGame, pick: Option<ObjectId>, discard: &[ObjectId]) {
    let mass = t.battlefield(P0, "Mindleech Mass");
    t.answer_yes(P0, true);
    if let Some(c) = pick {
        t.answer_choose(P0, &[Entity::Object(c)]);
    }
    if !discard.is_empty() {
        let d: Vec<Entity> = discard.iter().map(|c| Entity::Object(*c)).collect();
        t.answer_choose(P0, &d);
    }
    attack_with(t, &[(mass, Entity::Player(P1))]);
    block_and_finish(t, P1, &[]);
}

#[test]
fn mindleech_mass_free_spell_may_be_kicked_but_not_overloaded() {
    cr!("118.9", "118.9a", "118.8", "601.2b", "601.2f", "608.2g");
    ruling!(
        "Mindleech Mass",
        "If you cast a spell without paying its mana cost, you can't choose to cast it for any alternative costs, such as overload costs. You can pay additional costs, such as kicker costs. If the spell has any mandatory additional costs, you must pay those."
    );
    supported("Mindleech Mass");
    // "Whenever this creature deals combat damage to a player, you may look at that
    // player's hand. If you do, you may cast a spell from among those cards without
    // paying its mana cost."
    // P1's Burst Lightning, kicked for {4}: 6 combat damage and 4 from the spell.
    let mut t = TestGame::new(2);
    t.lands(P0, "Wastes", 4);
    let burst = t.hand(P1, "Burst Lightning");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.answer_targets(P0, &[Entity::Player(P1)]);
    mindleech_unblocked(&mut t, Some(burst), &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 10);
    assert_eq!(tapped_lands(&t, P0), 4);
    // P1's Cyclonic Rift can't be cast for its overload cost.
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 7);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let elves = t.battlefield(P1, "Llanowar Elves");
    let rift = t.hand(P1, "Cyclonic Rift");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    let from = t.asked().len();
    mindleech_unblocked(&mut t, Some(rift), &[]);
    t.resolve_all();
    assert!(!casting_way_asked(&t, P0, from));
    assert!(t.in_hand(P1, "Grizzly Bears"));
    assert!(t.on_battlefield(elves));
    assert_eq!(tapped_lands(&t, P0), 0);
    // P1's Tormenting Voice: P0 discards a card from their own hand to cast it (during
    // combat: its timing is ignored).
    let mut t = TestGame::new(2);
    let forest = t.hand(P0, "Forest");
    let voice = t.hand(P1, "Tormenting Voice");
    mindleech_unblocked(&mut t, Some(voice), &[forest]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Forest"));
    assert!(t.in_graveyard(P1, "Tormenting Voice"));
    assert_eq!(t.hand_size(P0), 2);
    // Without a card to discard, it can't be cast.
    let mut t = TestGame::new(2);
    let voice = t.hand(P1, "Tormenting Voice");
    mindleech_unblocked(&mut t, Some(voice), &[]);
    t.resolve_all();
    assert!(t.in_hand(P1, "Tormenting Voice"));
    assert_eq!(t.hand_size(P0), 0);
}

#[test]
fn mindleech_mass_permanent_spell_enters_under_the_casters_control() {
    cr!("608.2g", "608.3", "608.2n", "110.2");
    ruling!(
        "Mindleech Mass",
        "If you cast a permanent spell this way, it will enter the battlefield under your control when it resolves. If you cast an instant or sorcery spell this way, that card will be put into its owner's graveyard when it resolves."
    );
    supported("Mindleech Mass");
    let mut t = TestGame::new(2);
    let bears = t.hand(P1, "Grizzly Bears");
    mindleech_unblocked(&mut t, Some(bears), &[]);
    t.resolve_all();
    let on = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(on.len(), 1);
    assert_eq!(t.obj_now(on[0]).controller, P0);
    assert_eq!(t.obj_now(on[0]).owner, P1);
    let mut t = TestGame::new(2);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    mindleech_unblocked(&mut t, Some(bolt), &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 20 - 6 - 3);
    assert!(t.in_graveyard(P1, "Lightning Bolt"));
    assert!(!t.in_graveyard(P0, "Lightning Bolt"));
}

/// P0's Anrakyr the Traveller attacks P1 and isn't blocked; its trigger resolves with the
/// answers queued (P0 casts `pick`, choosing the cards in `discard` for its cost) and
/// combat ends.
fn anrakyr_attacks(t: &mut TestGame, pick: ObjectId, discard: &[ObjectId]) {
    let anrakyr = t.battlefield(P0, "Anrakyr the Traveller");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(pick)]);
    if !discard.is_empty() {
        let d: Vec<Entity> = discard.iter().map(|c| Entity::Object(*c)).collect();
        t.answer_choose(P0, &d);
    }
    attack_with(t, &[(anrakyr, Entity::Player(P1))]);
    block_and_finish(t, P1, &[]);
}

#[test]
fn anrakyr_spells_are_cast_only_by_paying_life_plus_additional_costs() {
    cr!("118.9", "118.9a", "118.8", "119.4", "601.2b", "601.2f", "608.2g");
    ruling!(
        "Anrakyr the Traveller",
        "You may only cast a spell this way by paying the appropriate amount of life. You may not pay its normal cost and may not pay any other alternative costs. You may still pay for additional costs, such as kicker costs. If the spell has mandatory additional costs, you must pay those."
    );
    supported("Anrakyr the Traveller");
    supported("Skyclave Sentinel");
    supported("Lesser Masticore");
    supported("Salvage Titan");
    // "Whenever Anrakyr the Traveller attacks, you may cast an artifact spell from your
    // hand or graveyard by paying life equal to its mana value rather than paying its
    // mana cost."
    // Skyclave Sentinel (mana value 3) for 3 life, kicked for {4}: the lands pay only the
    // kicker.
    let mut t = TestGame::new(2);
    t.lands(P0, "Wastes", 7);
    let sentinel = t.hand(P0, "Skyclave Sentinel");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    anrakyr_attacks(&mut t, sentinel, &[]);
    t.resolve_all();
    assert_eq!(t.life(P0), 17);
    assert_eq!(tapped_lands(&t, P0), 4);
    let on = t.named_on_battlefield("Skyclave Sentinel");
    assert_eq!(on.len(), 1);
    assert_eq!(t.counters(on[0], "+1/+1"), 2);
    // Lesser Masticore (mana value 2) from the graveyard for 2 life: a card must be
    // discarded too.
    let mut t = TestGame::new(2);
    let masticore = t.graveyard(P0, "Lesser Masticore");
    let forest = t.hand(P0, "Forest");
    anrakyr_attacks(&mut t, masticore, &[forest]);
    t.resolve_all();
    assert_eq!(t.life(P0), 18);
    assert!(t.in_graveyard(P0, "Forest"));
    assert_eq!(t.named_on_battlefield("Lesser Masticore").len(), 1);
    // Without a card to discard, it can't be cast.
    let mut t = TestGame::new(2);
    let masticore = t.graveyard(P0, "Lesser Masticore");
    anrakyr_attacks(&mut t, masticore, &[]);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    assert!(t.in_graveyard(P0, "Lesser Masticore"));
    // Salvage Titan (mana value 6) costs 6 life: its own alternative cost (sacrificing
    // three artifacts) can't be used instead.
    let mut t = TestGame::new(2);
    let thopters: Vec<ObjectId> = (0..3).map(|_| t.battlefield(P0, "Ornithopter")).collect();
    let titan = t.hand(P0, "Salvage Titan");
    let from = t.asked().len();
    anrakyr_attacks(&mut t, titan, &[]);
    t.resolve_all();
    assert!(!casting_way_asked(&t, P0, from));
    assert_eq!(t.life(P0), 14);
    assert_eq!(t.named_on_battlefield("Salvage Titan").len(), 1);
    assert!(thopters.iter().all(|o| t.on_battlefield(*o)));
    // With less life than its mana value, the life can't be paid.
    let mut t = TestGame::new(2);
    t.g.players[0].life = 5;
    let titan = t.hand(P0, "Salvage Titan");
    anrakyr_attacks(&mut t, titan, &[]);
    t.resolve_all();
    assert_eq!(t.life(P0), 5);
    assert!(t.in_hand(P0, "Salvage Titan"));
}

/// Whether a card named `name` is among the bottom `n` cards of `p`'s library.
fn in_bottom(t: &TestGame, p: PlayerId, n: usize, name: &str) -> bool {
    t.g.player(p).library[..n]
        .iter()
        .any(|c| t.g.obj(*c).chars.name == name)
}

/// P0 controls Kiora, Sovereign of the Deep with `top` on top of their library (top
/// first) and casts Sea Monster (a Serpent spell with mana value 6) from their hand;
/// Kiora's trigger resolves with the answers queued (P0 casts `pick`, choosing the cards
/// in `discard` for its cost).
fn kiora_trigger(t: &mut TestGame, top: &[&str], pick: Option<&str>, discard: &[ObjectId]) {
    t.battlefield(P0, "Kiora, Sovereign of the Deep");
    let cards = stack_library(t, P0, top);
    give_mana_for(t, P0, "Sea Monster");
    let monster = t.hand(P0, "Sea Monster");
    t.answer_yes(P0, true);
    if let Some(name) = pick {
        let i = top.iter().position(|n| *n == name).unwrap();
        t.answer_choose(P0, &[Entity::Object(cards[i])]);
    }
    if !discard.is_empty() {
        let d: Vec<Entity> = discard.iter().map(|c| Entity::Object(*c)).collect();
        t.answer_choose(P0, &d);
    }
    t.cast(P0, monster).go();
    t.resolve();
}

#[test]
fn kiora_free_spell_may_be_kicked_and_pays_mandatory_additional_costs() {
    cr!("118.9", "118.9a", "118.8", "601.2b", "601.2f", "608.2g");
    ruling!(
        "Kiora, Sovereign of the Deep",
        "If you cast a spell \"without paying its mana cost,\" you can't pay any alternative costs. You can, however, pay additional costs. If the spell has any mandatory additional costs, those must be paid to cast it."
    );
    supported("Kiora, Sovereign of the Deep");
    supported("Sea Monster");
    // "Whenever you cast a Kraken, Leviathan, Octopus, or Serpent spell from your hand,
    // look at the top X cards of your library, where X is that spell's mana value. You may
    // cast a spell with mana value less than X from among them without paying its mana
    // cost. Put the rest on the bottom of your library in a random order."
    // Burst Lightning, kicked for {4}; Grizzly Bears goes to the bottom.
    let mut t = TestGame::new(2);
    t.lands(P0, "Wastes", 4);
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.answer_targets(P0, &[Entity::Player(P1)]);
    kiora_trigger(
        &mut t,
        &["Grizzly Bears", "Burst Lightning"],
        Some("Burst Lightning"),
        &[],
    );
    // The other five cards looked at are the bottom five, in a random order.
    assert!(in_bottom(&t, P0, 5, "Grizzly Bears"));
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    assert_eq!(tapped_lands(&t, P0), 10);
    // Tormenting Voice: a card is discarded to cast it.
    let mut t = TestGame::new(2);
    let forest = t.hand(P0, "Forest");
    kiora_trigger(
        &mut t,
        &["Tormenting Voice"],
        Some("Tormenting Voice"),
        &[forest],
    );
    assert!(t.in_graveyard(P0, "Forest"));
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Tormenting Voice"));
    // Cyclonic Rift can't be cast for its overload cost.
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 7);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let elves = t.battlefield(P1, "Llanowar Elves");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    let from = t.asked().len();
    kiora_trigger(&mut t, &["Cyclonic Rift"], Some("Cyclonic Rift"), &[]);
    t.resolve_all();
    assert!(!casting_way_asked(&t, P0, from));
    assert!(t.in_hand(P1, "Grizzly Bears"));
    assert!(t.on_battlefield(elves));
    assert_eq!(tapped_lands(&t, P0), 6);
}

#[test]
fn kiora_free_spell_resolves_first_ignoring_timing_with_x_zero() {
    cr!("608.2g", "107.3b", "405.2");
    ruling!(
        "Kiora, Sovereign of the Deep",
        "The spell you cast without paying its mana cost is cast during the resolution of the triggered ability. Timing restrictions of that spell based on card type are ignored. It will resolve before the spell that caused the ability to trigger."
    );
    ruling!(
        "Kiora, Sovereign of the Deep",
        "If the spell has {X} in its mana cost, you must choose 0 as the value of X when casting it without paying its mana cost."
    );
    supported("Kiora, Sovereign of the Deep");
    // Blaze, a sorcery, is cast with Sea Monster on the stack, with X = 0; it resolves
    // first, dealing no damage.
    let mut t = TestGame::new(2);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    kiora_trigger(&mut t, &["Blaze"], Some("Blaze"), &[]);
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    assert!(t.in_graveyard(P0, "Blaze"));
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.stack_len(), 1);
    assert!(t.named_on_battlefield("Sea Monster").is_empty());
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Sea Monster").len(), 1);
    // A card whose spell would have mana value 6 (another Sea Monster) can't be cast.
    let mut t = TestGame::new(2);
    kiora_trigger(&mut t, &["Sea Monster"], None, &[]);
    assert_eq!(t.stack_len(), 1);
    assert!(in_bottom(&t, P0, 6, "Sea Monster"));
}

/// P0 casts Fevered Suspicion from their hand (with the answers queued, choosing the
/// exiled cards named `cast`, in that order) and it resolves.
fn fevered_suspicion(t: &mut TestGame, cast: &[&str]) {
    give_mana_for(t, P0, "Fevered Suspicion");
    let card = t.hand(P0, "Fevered Suspicion");
    t.answer_yes(P0, true);
    choose_named_once(t, P0, cast);
    t.cast(P0, card).go();
    t.resolve();
}

#[test]
fn fevered_suspicion_free_spells_may_be_kicked_but_not_overloaded() {
    cr!("118.9", "118.9a", "118.8", "601.2b", "601.2f", "608.2g");
    ruling!(
        "Fevered Suspicion",
        "If you cast a spell “without paying its mana cost,” you can't choose to cast it for any alternative costs. You can, however, pay additional costs. If the card has any mandatory additional costs, you must pay those to cast the spell."
    );
    supported("Fevered Suspicion");
    // "Each opponent exiles cards from the top of their library until they exile a
    // nonland card. You may cast any number of spells from among those nonland cards
    // without paying their mana costs."
    // P1's Burst Lightning, kicked for {4}.
    let mut t = TestGame::new(2);
    stack_library(&mut t, P1, &["Forest", "Burst Lightning"]);
    t.lands(P0, "Wastes", 4);
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.answer_targets(P0, &[Entity::Player(P1)]);
    fevered_suspicion(&mut t, &["Burst Lightning"]);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    assert!(t.in_graveyard(P1, "Burst Lightning"));
    // P1's Tormenting Voice: P0 discards a card to cast it.
    let mut t = TestGame::new(2);
    stack_library(&mut t, P1, &["Tormenting Voice"]);
    let forest = t.hand(P0, "Forest");
    t.answer_choose(P0, &[Entity::Object(forest)]);
    fevered_suspicion(&mut t, &["Tormenting Voice"]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Forest"));
    assert!(t.in_graveyard(P1, "Tormenting Voice"));
    // P1's Cyclonic Rift can't be cast for its overload cost.
    let mut t = TestGame::new(2);
    stack_library(&mut t, P1, &["Cyclonic Rift"]);
    t.lands(P0, "Island", 7);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let elves = t.battlefield(P1, "Llanowar Elves");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    let from = t.asked().len();
    fevered_suspicion(&mut t, &["Cyclonic Rift"]);
    t.resolve_all();
    assert!(!casting_way_asked(&t, P0, from));
    assert!(t.in_hand(P1, "Grizzly Bears"));
    assert!(t.on_battlefield(elves));
    assert_eq!(tapped_lands(&t, P0), 8);
}

#[test]
fn fevered_suspicion_cards_are_cast_in_any_order_now_or_never() {
    cr!("608.2g", "406.1");
    ruling!(
        "Fevered Suspicion",
        "You may cast the exiled cards in any order, not just the order they were exiled."
    );
    ruling!(
        "Fevered Suspicion",
        "You cast the cards exiled with Fevered Suspicion as it is resolving. If you choose not to cast them, you cannot cast them later."
    );
    ruling!(
        "Fevered Suspicion",
        "Cards you don't cast this way, including any exiled land cards, will remain in exile indefinitely."
    );
    supported("Fevered Suspicion");
    // P1 exiles a Forest and Burst Lightning, then P2 exiles Lightning Bolt. P0 casts the
    // bolt first, then Burst Lightning (on top of it).
    let mut t = TestGame::new(3);
    stack_library(&mut t, P1, &["Forest", "Burst Lightning"]);
    stack_library(&mut t, P2, &["Lightning Bolt"]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_targets(P0, &[Entity::Player(P2)]);
    fevered_suspicion(&mut t, &["Lightning Bolt", "Burst Lightning"]);
    let names: Vec<String> =
        t.g.stack
            .iter()
            .map(|s| t.g.obj(*s).chars.name.to_string())
            .collect();
    assert_eq!(names, vec!["Lightning Bolt", "Burst Lightning"]);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.life(P2), 18);
    assert_eq!(
        t.g.find_in_zone(mtg_engine::object::Zone::Exile, "Forest")
            .len(),
        1
    );
    // Only the bolt is cast: Burst Lightning stays in exile and can't be cast later.
    let mut t = TestGame::new(3);
    stack_library(&mut t, P1, &["Forest", "Burst Lightning"]);
    stack_library(&mut t, P2, &["Lightning Bolt"]);
    t.answer_targets(P0, &[Entity::Player(P2)]);
    fevered_suspicion(&mut t, &["Lightning Bolt"]);
    t.resolve_all();
    assert_eq!(t.life(P2), 17);
    let burst =
        t.g.find_in_zone(mtg_engine::object::Zone::Exile, "Burst Lightning");
    assert_eq!(burst.len(), 1);
    assert_eq!(
        t.g.find_in_zone(mtg_engine::object::Zone::Exile, "Forest")
            .len(),
        1
    );
    t.lands(P0, "Mountain", 1);
    assert!(crate::r_s08_common::legal_cast_methods(&mut t, P0, burst[0]).is_empty());
}

/// P0 casts Invoke Calamity from their hand (with the answers queued) and it resolves.
fn invoke_calamity(t: &mut TestGame) {
    give_mana_for(t, P0, "Invoke Calamity");
    let card = t.hand(P0, "Invoke Calamity");
    t.answer_yes(P0, true);
    t.cast(P0, card).go();
    t.resolve();
}

#[test]
fn invoke_calamity_free_spells_may_be_kicked_and_pay_mandatory_additional_costs() {
    cr!("118.9", "118.9a", "118.8", "601.2b", "601.2f", "608.2g");
    ruling!(
        "Invoke Calamity",
        "If you cast a spell without paying its mana cost, you can't choose to cast it for any alternative costs. You can, however, pay any additional costs. If the spell has any mandatory additional costs, you must pay those."
    );
    supported("Invoke Calamity");
    // "You may cast up to two instant and/or sorcery spells with total mana value 6 or
    // less from your graveyard and/or hand without paying their mana costs. If those
    // spells would be put into your graveyard, exile them instead. Exile Invoke Calamity."
    // Burst Lightning from the graveyard, kicked for {4}; Tormenting Voice from the hand,
    // discarding a Forest.
    let mut t = TestGame::new(2);
    t.lands(P0, "Wastes", 4);
    let burst = t.graveyard(P0, "Burst Lightning");
    let voice = t.hand(P0, "Tormenting Voice");
    let forest = t.hand(P0, "Forest");
    t.answer_choose(P0, &[Entity::Object(burst)]);
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_choose(P0, &[Entity::Object(voice)]);
    t.answer_choose(P0, &[Entity::Object(forest)]);
    invoke_calamity(&mut t);
    assert!(t.in_graveyard(P0, "Forest"));
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    assert_eq!(tapped_lands(&t, P0), 5 + 4);
    // Those spells and Invoke Calamity are exiled.
    assert!(t.in_exile("Burst Lightning"));
    assert!(t.in_exile("Tormenting Voice"));
    assert!(t.in_exile("Invoke Calamity"));
    // Cyclonic Rift can't be cast for its overload cost.
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 7);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let elves = t.battlefield(P1, "Llanowar Elves");
    let rift = t.graveyard(P0, "Cyclonic Rift");
    t.answer_choose(P0, &[Entity::Object(rift)]);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    let from = t.asked().len();
    invoke_calamity(&mut t);
    t.resolve_all();
    assert!(!casting_way_asked(&t, P0, from));
    assert!(t.in_hand(P1, "Grizzly Bears"));
    assert!(t.on_battlefield(elves));
    assert_eq!(tapped_lands(&t, P0), 5);
}

#[test]
fn invoke_calamity_casts_one_after_the_other_judging_the_spells() {
    cr!("608.2g", "405.2", "601.3e", "709.3a", "202.3d");
    ruling!(
        "Invoke Calamity",
        "The spells are cast one after the other during the resolution of Invoke Calamity. The one you cast second will be the first one to resolve."
    );
    ruling!(
        "Invoke Calamity",
        "Invoke Calamity looks for the mana values and types of the spells on the stack, not the mana values and types of the cards in your graveyard."
    );
    supported("Invoke Calamity");
    supported("Hieroglyphic Illumination");
    // Fire // Ice (mana value 4 in the graveyard) is cast as Ice (mana value 2), then
    // Hieroglyphic Illumination (mana value 4): a total of 6.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let fire_ice = t.graveyard(P0, "Fire // Ice");
    let illumination = t.hand(P0, "Hieroglyphic Illumination");
    t.answer_choose(P0, &[Entity::Object(fire_ice)]);
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.answer_choose(P0, &[Entity::Object(illumination)]);
    invoke_calamity(&mut t);
    let names: Vec<String> =
        t.g.stack
            .iter()
            .map(|s| t.g.obj(*s).chars.name.to_string())
            .collect();
    assert_eq!(names, vec!["Ice", "Hieroglyphic Illumination"]);
    // Hieroglyphic Illumination, cast second, resolves first.
    let hand = t.hand_size(P0);
    t.resolve();
    assert_eq!(t.hand_size(P0), hand + 2);
    assert_eq!(t.stack_len(), 1);
    t.resolve();
    assert!(t.obj_now(bears).tapped);
    assert_eq!(t.hand_size(P0), hand + 3);
    // After Hieroglyphic Illumination (4), a spell with mana value 3 can't be cast too:
    // only spells with mana value 2 or less.
    let mut t = TestGame::new(2);
    let illumination = t.graveyard(P0, "Hieroglyphic Illumination");
    let divination = t.hand(P0, "Divination");
    t.answer_choose(P0, &[Entity::Object(illumination)]);
    t.answer_choose(P0, &[Entity::Object(divination)]);
    invoke_calamity(&mut t);
    assert_eq!(t.stack_len(), 1);
    assert!(t.in_hand(P0, "Divination"));
}
