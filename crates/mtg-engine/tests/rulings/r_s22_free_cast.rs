//! Rulings batch S22 — casting a spell without paying its mana cost (CR 118.9): no
//! alternative cost can be paid, but optional additional costs (kicker) may be and
//! mandatory additional costs must be (CR 118.9a–b); a spell with {X} in its mana cost
//! cast this way has X = 0 (CR 107.3b). Checked for each effect that casts a spell this
//! way: from the hand (Sram's Expertise, Surtland Elementalist), from a graveyard
//! (Impulsivity, Seifer Almasy, Efreet Flamepainter, Wishing Well), from exile (Collector's
//! Cage, Evercoat Ursine), and a copy of a card (Isochron Scepter).

use crate::r_s01_common::*;
use crate::r_s22_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::*;

fn sram(t: &mut TestGame, name: &str) -> ObjectId {
    t.hand(P0, name)
}

fn arcane_heist(t: &mut TestGame, name: &str) -> ObjectId {
    t.graveyard(P1, name)
}

fn run_arcane_heist(t: &mut TestGame, card: ObjectId, answers: &dyn Fn(&mut TestGame)) {
    crate::r_s04_common::add_mana(t, P0, ManaType::U, 4);
    let heist = t.hand(P0, "Arcane Heist");
    t.answer_targets(P0, &[Entity::Object(card)]);
    answers(t);
    t.cast(P0, heist).go();
    t.resolve_all();
    t.clear_answers();
}

#[test]
fn arcane_heist_casts_without_paying_but_additional_costs_are_paid() {
    cr!("118.9a", "118.9b", "118.8a", "601.2b", "608.2g");
    ruling!(
        "Arcane Heist",
        "If you cast a spell “without paying its mana cost,” you can’t choose to cast it for any alternative costs. You can, however, pay additional costs, such as kicker costs. If the spell has any mandatory additional costs, those must be paid to cast the spell."
    );
    supported("Arcane Heist");
    // "You may cast target instant or sorcery card from an opponent's graveyard without
    // paying its mana cost. If that spell would be put into their graveyard, exile it
    // instead."
    check_free_cast_costs(&FreeCaster {
        instants_only: false,
        place: arcane_heist,
        run: run_arcane_heist,
    });
    // The spell cast this way is exiled rather than put into its owner's graveyard.
    let mut t = TestGame::new(2);
    let bolt = arcane_heist(&mut t, "Burst Lightning");
    run_arcane_heist(&mut t, bolt, &|t| {
        t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(false));
        t.answer_targets(P0, &[Entity::Player(P1)]);
    });
    assert_eq!(t.life(P1), 18);
    assert!(t.in_exile("Burst Lightning"));
    assert!(!t.in_graveyard(P1, "Burst Lightning"));
}

fn glamdring(t: &mut TestGame, name: &str) -> ObjectId {
    // Grizzly Bears equipped with Glamdring (first strike): 2 combat damage.
    let bears = t.battlefield(P0, "Grizzly Bears");
    crate::r_s06_common::attach_new(t, P0, "Glamdring", bears);
    t.hand(P0, name)
}

fn run_glamdring(t: &mut TestGame, card: ObjectId, answers: &dyn Fn(&mut TestGame)) {
    let bears = crate::r_s20_common::controlled_named(t, P0, "Grizzly Bears")[0];
    t.answer_choose(P0, &[Entity::Object(card)]);
    answers(t);
    attack_p1_unblocked(t, bears);
    t.clear_answers();
}

#[test]
fn glamdring_casts_without_paying_but_additional_costs_are_paid() {
    cr!("118.9a", "118.9b", "118.8a", "601.2b", "608.2g");
    ruling!(
        "Glamdring",
        "If you cast a spell \"without paying its mana cost\", you can't choose to cast it for any alternative costs. You can, however, pay additional costs, such as kicker costs. If the card has any mandatory additional costs, those must be paid to cast the spell."
    );
    supported("Glamdring");
    // "Whenever equipped creature deals combat damage to a player, you may cast an instant
    // or sorcery spell from your hand with mana value less than or equal to that damage
    // without paying its mana cost."
    check_free_cast_costs(&FreeCaster {
        instants_only: false,
        place: glamdring,
        run: run_glamdring,
    });
    // A spell with a greater mana value than the damage can't be cast this way.
    let mut t = TestGame::new(2);
    let wrath = glamdring(&mut t, "Act on Impulse");
    run_glamdring(&mut t, wrath, &|_| {});
    assert!(t.in_hand(P0, "Act on Impulse"));
    assert_eq!(t.stack_len(), 0);
}

/// Grizzly Bears equipped with Buster Sword ("Equipped creature gets +3/+2. Whenever
/// equipped creature deals combat damage to a player, draw a card, then you may cast a
/// spell from your hand with mana value less than or equal to that damage without paying
/// its mana cost.") attacks P1: 5 damage. P0 casts `card` from their hand, `answers`
/// queued for casting it.
fn buster_sword_attack(t: &mut TestGame, card: ObjectId, answers: &dyn Fn(&mut TestGame)) {
    let bears = t.battlefield(P0, "Grizzly Bears");
    crate::r_s06_common::attach_new(t, P0, "Buster Sword", bears);
    t.answer_choose(P0, &[Entity::Object(card)]);
    answers(t);
    attack_p1_unblocked(t, bears);
    t.clear_answers();
}

#[test]
fn buster_sword_casts_without_paying_but_additional_costs_are_paid() {
    cr!("118.9a", "118.9b", "118.8a", "601.2b", "608.2g");
    ruling!(
        "Buster Sword",
        "If you cast a spell \"without paying its mana cost,\" you can't choose to cast it for any alternative costs. You can, however, pay additional costs, such as kicker costs. If the card has any mandatory additional costs, those must be paid to cast the spell."
    );
    supported("Buster Sword");
    // Kicker may be paid: kicked Burst Lightning kills Hill Giant.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let bolt = t.hand(P0, "Burst Lightning");
    t.lands(P0, "Wastes", 4);
    buster_sword_attack(&mut t, bolt, &|t| {
        t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
        t.answer_targets(P0, &[Entity::Object(giant)]);
    });
    assert_eq!(tapped_named(&t, P0, "Wastes"), 4);
    assert!(t.in_graveyard(P1, "Hill Giant"));
    // Tormenting Voice's discard must be paid: P0 discards the Forest (the card drawn by
    // Buster Sword stays).
    let mut t = TestGame::new(2);
    let voice = t.hand(P0, "Tormenting Voice");
    let forest = t.hand(P0, "Forest");
    let lib = t.library_size(P0);
    buster_sword_attack(&mut t, voice, &|t| {
        t.answer_choose(P0, &[Entity::Object(forest)]);
    });
    assert!(t.in_graveyard(P0, "Forest"));
    assert!(t.in_graveyard(P0, "Tormenting Voice"));
    assert_eq!(t.library_size(P0), lib - 1 - 2);
    // Cyclonic Rift isn't overloaded.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let other = t.battlefield(P1, "Llanowar Elves");
    let rift = t.hand(P0, "Cyclonic Rift");
    t.lands(P0, "Island", 7);
    buster_sword_attack(&mut t, rift, &|t| {
        t.answer_targets(P0, &[Entity::Object(giant)]);
    });
    assert!(t.in_hand(P1, "Hill Giant"));
    assert!(t.on_battlefield(other));
    assert_eq!(tapped_named(&t, P0, "Island"), 0);
}

#[test]
fn a_spell_with_x_cast_by_buster_sword_has_x_zero() {
    cr!("107.3b", "118.9");
    ruling!(
        "Buster Sword",
        "If a spell you cast has {X} in its mana cost, you must choose 0 as the value of X when casting it without paying its mana cost."
    );
    supported("Buster Sword");
    // "Equipped creature gets +3/+2. Whenever equipped creature deals combat damage to a
    // player, draw a card, then you may cast a spell from your hand with mana value less
    // than or equal to that damage without paying its mana cost." Grizzly Bears deals 5;
    // Blaze (mana value 1) is cast with X = 0 at P1's Hill Giant.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    crate::r_s06_common::attach_new(&mut t, P0, "Buster Sword", bears);
    let giant = t.battlefield(P1, "Hill Giant");
    let blaze = t.hand(P0, "Blaze");
    t.lands(P0, "Mountain", 6);
    let hand = t.hand_size(P0);
    t.answer_choose(P0, &[Entity::Object(blaze)]);
    t.answer(P0, DecisionKind::X, Answer::Number(5));
    t.answer_targets(P0, &[Entity::Object(giant)]);
    attack_p1_unblocked(&mut t, bears);
    assert_eq!(t.life(P1), 15);
    assert!(t.in_graveyard(P0, "Blaze"));
    assert_eq!(t.hand_size(P0), hand - 1 + 1);
    assert_eq!(tapped_lands(&t, P0), 0);
    assert!(t.on_battlefield(giant));
    assert_eq!(crate::r_s07_common::damage_on(&t, giant), 0);
}

fn run_sram(t: &mut TestGame, card: ObjectId, answers: &dyn Fn(&mut TestGame)) {
    crate::r_s04_common::add_mana(t, P0, ManaType::W, 4);
    let expertise = t.hand(P0, "Sram's Expertise");
    t.answer_choose(P0, &[Entity::Object(card)]);
    answers(t);
    t.cast(P0, expertise).go();
    t.resolve_all();
    t.clear_answers();
}

#[test]
fn sram_expertise_casts_without_paying_but_additional_costs_are_paid() {
    cr!("118.9", "118.9a", "118.9b", "118.8a", "601.2b", "608.2g");
    ruling!(
        "Sram's Expertise",
        "If you cast a card \"without paying its mana cost,\" you can't choose to cast it for any alternative costs, such as emerge costs. You can, however, pay additional costs. If the card has any mandatory additional costs, such as that of Cathartic Reunion, you must pay those to cast the card."
    );
    supported("Sram's Expertise");
    check_free_cast_costs(&FreeCaster {
        instants_only: false,
        place: sram,
        run: run_sram,
    });
}

fn surtland(t: &mut TestGame, name: &str) -> ObjectId {
    t.battlefield(P0, "Surtland Elementalist");
    t.hand(P0, name)
}

fn run_surtland(t: &mut TestGame, card: ObjectId, answers: &dyn Fn(&mut TestGame)) {
    let s = t.named_on_battlefield("Surtland Elementalist")[0];
    t.answer_choose(P0, &[Entity::Object(card)]);
    answers(t);
    attack_p1_unblocked(t, s);
    t.clear_answers();
}

#[test]
fn surtland_elementalist_casts_without_paying_but_additional_costs_are_paid() {
    cr!("118.9a", "118.9b", "118.8a", "601.2b", "608.2g");
    ruling!(
        "Surtland Elementalist",
        "If you cast a spell “without paying its mana cost,” you can’t choose to cast it for any alternative costs. You can, however, pay additional costs, such as kicker costs. If the card has any mandatory additional costs, those must be paid to cast the spell."
    );
    supported("Surtland Elementalist");
    check_free_cast_costs(&FreeCaster {
        instants_only: false,
        place: surtland,
        run: run_surtland,
    });
}

fn impulsivity(t: &mut TestGame, name: &str) -> ObjectId {
    t.graveyard(P0, name)
}

fn run_impulsivity(t: &mut TestGame, card: ObjectId, answers: &dyn Fn(&mut TestGame)) {
    t.answer_targets(P0, &[Entity::Object(card)]);
    answers(t);
    t.enter(P0, "Impulsivity");
    t.resolve_all();
    t.clear_answers();
}

#[test]
fn impulsivity_casts_for_an_alternative_cost_but_additional_costs_are_paid() {
    cr!("118.9", "118.9a", "118.9b", "118.8a", "601.2b");
    ruling!(
        "Impulsivity",
        "Since you are using an alternative cost to cast the spell, you can't pay any other alternative costs. You can, however, pay additional costs, such as kicker costs. If the card has any mandatory additional costs, you must pay those."
    );
    supported("Impulsivity");
    check_free_cast_costs(&FreeCaster {
        instants_only: false,
        place: impulsivity,
        run: run_impulsivity,
    });
}

fn seifer(t: &mut TestGame, name: &str) -> ObjectId {
    t.battlefield(P0, "Seifer Almasy");
    // A second attacker: Seifer doesn't attack alone (and doesn't gain double strike).
    t.battlefield(P0, "Llanowar Elves");
    t.graveyard(P0, name)
}

fn run_seifer(t: &mut TestGame, card: ObjectId, answers: &dyn Fn(&mut TestGame)) {
    let s = t.named_on_battlefield("Seifer Almasy")[0];
    let elves = t.named_on_battlefield("Llanowar Elves")[0];
    t.answer_targets(P0, &[Entity::Object(card)]);
    answers(t);
    attack_with(
        t,
        &[(s, Entity::Player(P1)), (elves, Entity::Player(P1))],
    );
    block_and_finish(t, P1, &[]);
    t.resolve_all();
    t.clear_answers();
}

#[test]
fn seifer_almasy_casts_without_paying_but_additional_costs_are_paid() {
    cr!("118.9a", "118.9b", "118.8a", "601.2b");
    ruling!(
        "Seifer Almasy",
        "If you cast a spell \"without paying its mana cost,\" you can't choose to cast it for any alternative costs. You can, however, pay additional costs. If the card has any mandatory additional costs, you must pay those to cast the spell."
    );
    supported("Seifer Almasy");
    check_free_cast_costs(&FreeCaster {
        instants_only: false,
        place: seifer,
        run: run_seifer,
    });
}

fn efreet(t: &mut TestGame, name: &str) -> ObjectId {
    t.battlefield(P0, "Efreet Flamepainter");
    t.graveyard(P0, name)
}

fn run_efreet(t: &mut TestGame, card: ObjectId, answers: &dyn Fn(&mut TestGame)) {
    let e = t.named_on_battlefield("Efreet Flamepainter")[0];
    t.answer_targets(P0, &[Entity::Object(card)]);
    answers(t);
    attack_p1_unblocked(t, e);
    t.clear_answers();
}

#[test]
fn efreet_flamepainter_casts_without_paying_but_additional_costs_are_paid() {
    cr!("118.9a", "118.9b", "118.8a", "601.2b");
    ruling!(
        "Efreet Flamepainter",
        "If you cast a spell “without paying its mana cost,” you can’t choose to cast it for any alternative costs. You can, however, pay additional costs. If the spell has any mandatory additional costs, those must be paid to cast the spell."
    );
    supported("Efreet Flamepainter");
    check_free_cast_costs(&FreeCaster {
        instants_only: false,
        place: efreet,
        run: run_efreet,
    });
}

fn wishing_well(t: &mut TestGame, name: &str) -> ObjectId {
    let card = t.graveyard(P0, name);
    let well = t.battlefield(P0, "Wishing Well");
    // The ability casts a card with mana value equal to the coin counters after one more
    // is put on it.
    let mv = crate::r_s08_common::mana_value(t, card);
    crate::r_s13_common::add(t, well, "coin", (mv - 1) as u32);
    card
}

fn run_wishing_well(t: &mut TestGame, card: ObjectId, answers: &dyn Fn(&mut TestGame)) {
    let well = t.named_on_battlefield("Wishing Well")[0];
    t.answer_targets(P0, &[Entity::Object(card)]);
    answers(t);
    t.activate(P0, well, 0, &[]).expect("activate Wishing Well");
    t.resolve_all();
    t.clear_answers();
}

#[test]
fn wishing_well_casts_for_an_alternative_cost_but_additional_costs_are_paid() {
    cr!("118.9", "118.9a", "118.9b", "118.8a", "601.2b", "603.12");
    ruling!(
        "Wishing Well",
        "Since you are using an alternative cost to cast the spell, you can’t pay any other alternative costs. You can, however, pay additional costs, such as kicker costs. If the card has any mandatory additional costs, you must pay those."
    );
    supported("Wishing Well");
    check_free_cast_costs(&FreeCaster {
        instants_only: false,
        place: wishing_well,
        run: run_wishing_well,
    });
}

fn collectors_cage(t: &mut TestGame, name: &str) -> ObjectId {
    // Creatures with powers 1, 3, and 6: three different powers.
    t.battlefield(P0, "Llanowar Elves");
    t.battlefield(P0, "Hill Giant");
    t.battlefield(P0, "Craw Wurm");
    let card = t.library_top(P0, name);
    t.answer_choose(P0, &[Entity::Object(card)]);
    t.enter(P0, "Collector's Cage");
    t.resolve_all();
    t.clear_answers();
    assert!(t.obj_now(card).face_down);
    card
}

fn run_collectors_cage(t: &mut TestGame, card: ObjectId, answers: &dyn Fn(&mut TestGame)) {
    let cage = t.named_on_battlefield("Collector's Cage")[0];
    let elves = t.named_on_battlefield("Llanowar Elves")[0];
    let _ = card;
    pool(t, 1);
    // The +1/+1 counter goes on Llanowar Elves: powers 2, 3, and 6.
    t.activate(P0, cage, 0, &[Entity::Object(elves)])
        .expect("activate Collector's Cage");
    answers(t);
    t.resolve_all();
    t.clear_answers();
}

#[test]
fn collectors_cage_plays_without_paying_but_additional_costs_are_paid() {
    cr!("118.9a", "118.9b", "118.8a", "601.2b", "702.75a");
    ruling!(
        "Collector's Cage",
        "If you cast a spell \"without paying its mana cost,\" you can't choose to cast it for any alternative costs. You can, however, pay additional costs, such as kicker costs. If the spell has any mandatory additional costs, those must be paid to cast the spell."
    );
    supported("Collector's Cage");
    check_free_cast_costs(&FreeCaster {
        instants_only: false,
        place: collectors_cage,
        run: run_collectors_cage,
    });
}

fn evercoat_ursine(t: &mut TestGame, name: &str) -> ObjectId {
    let cards = stack_library(t, P0, &[name, "Forest", "Forest", "Island"]);
    // Hideaway 3 twice: the card, then one of the next three.
    t.answer_choose(P0, &[Entity::Object(cards[0])]);
    let u = t.enter(P0, "Evercoat Ursine");
    t.resolve_all();
    t.clear_answers();
    // It has been under P0's control since the turn began (it can attack).
    t.g.objects[u.0 as usize].summoning_sick = false;
    assert!(t.obj_now(cards[0]).face_down);
    cards[0]
}

fn run_evercoat_ursine(t: &mut TestGame, card: ObjectId, answers: &dyn Fn(&mut TestGame)) {
    let u = t.named_on_battlefield("Evercoat Ursine")[0];
    t.answer_choose(P0, &[Entity::Object(t.g.current(card))]);
    answers(t);
    attack_p1_unblocked(t, u);
    t.clear_answers();
}

#[test]
fn evercoat_ursine_plays_without_paying_but_additional_costs_are_paid() {
    cr!("118.9a", "118.9b", "118.8a", "601.2b");
    ruling!(
        "Evercoat Ursine",
        "If you cast a spell \"without paying its mana cost,\" you can't choose to cast it for any alternative costs. You can, however, pay additional costs. If the spell has any mandatory additional costs, those must be paid to cast the spell."
    );
    supported("Evercoat Ursine");
    check_free_cast_costs(&FreeCaster {
        instants_only: false,
        place: evercoat_ursine,
        run: run_evercoat_ursine,
    });
}

fn isochron_scepter(t: &mut TestGame, name: &str) -> ObjectId {
    let card = t.hand(P0, name);
    t.answer_choose(P0, &[Entity::Object(card)]);
    t.enter(P0, "Isochron Scepter");
    t.resolve_all();
    t.clear_answers();
    assert_eq!(t.zone(card), mtg_engine::object::Zone::Exile);
    card
}

fn run_isochron_scepter(t: &mut TestGame, _card: ObjectId, answers: &dyn Fn(&mut TestGame)) {
    let s = t.named_on_battlefield("Isochron Scepter")[0];
    pool(t, 2);
    t.activate(P0, s, 0, &[]).expect("activate Isochron Scepter");
    answers(t);
    t.resolve_all();
    t.clear_answers();
}

#[test]
fn isochron_scepter_casts_the_copy_without_paying_but_additional_costs_are_paid() {
    cr!("118.9a", "118.9b", "118.8a", "601.2b", "707.12");
    ruling!(
        "Isochron Scepter",
        "If you cast a spell \"without paying its mana cost,\" you can't choose to cast it for any alternative costs. You can, however, pay additional costs. If the card has any mandatory additional costs, those must be paid to cast the spell."
    );
    supported("Isochron Scepter");
    check_free_cast_costs(&FreeCaster {
        instants_only: true,
        place: isochron_scepter,
        run: run_isochron_scepter,
    });
}

#[test]
fn a_spell_with_x_cast_by_electrodominance_has_x_zero() {
    cr!("107.3b", "118.9");
    ruling!(
        "Electrodominance",
        "If a spell has {X} in its mana cost, you must choose 0 as the value of X when casting it without paying its mana cost."
    );
    supported("Electrodominance");
    supported("Blaze");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let blaze = t.hand(P0, "Blaze");
    let dom = t.hand(P0, "Electrodominance");
    t.lands(P0, "Mountain", 3);
    // Electrodominance with X = 1 (1 damage to P1); then Blaze (mana value 1) is cast
    // from P0's hand without paying its mana cost. P0 would choose X = 5 for Blaze if
    // the choice were P0's; enough Mountains remain for that.
    t.lands(P0, "Mountain", 6);
    t.answer(P0, DecisionKind::X, Answer::Number(1));
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_choose(P0, &[Entity::Object(blaze)]);
    t.answer(P0, DecisionKind::X, Answer::Number(5));
    t.answer_targets(P0, &[Entity::Object(giant)]);
    t.cast(P0, dom).go();
    t.resolve();
    assert_eq!(t.life(P1), 19);
    // Blaze is on the stack with X = 0 and no mana was paid for it.
    let spell = t.g.stack[0];
    assert_eq!(t.obj(spell).chars.name, "Blaze");
    let si = t.obj(spell).stack.as_ref().expect("a spell");
    assert_eq!(si.cast.x.unwrap_or(0), 0);
    assert_eq!(tapped_lands(&t, P0), 3);
    t.resolve_all();
    assert!(t.on_battlefield(giant));
    assert_eq!(crate::r_s07_common::damage_on(&t, giant), 0);
}

#[test]
fn a_spell_with_x_cast_by_impulsivity_has_x_zero() {
    cr!("107.3b", "118.9");
    ruling!(
        "Impulsivity",
        "If the spell you cast has {X} in its mana cost, you must choose 0 as the value of X when casting it without paying its mana cost."
    );
    supported("Impulsivity");
    let mut t = TestGame::new(2);
    let blaze = t.graveyard(P1, "Blaze");
    t.lands(P0, "Mountain", 6);
    t.answer_targets(P0, &[Entity::Object(blaze)]);
    t.answer(P0, DecisionKind::X, Answer::Number(5));
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.enter(P0, "Impulsivity");
    t.settle();
    t.resolve();
    // Blaze (from P1's graveyard) is on the stack with X = 0.
    let spell = *t.g.stack.last().expect("Blaze was cast");
    assert_eq!(t.obj(spell).chars.name, "Blaze");
    let si = t.obj(spell).stack.as_ref().expect("a spell");
    assert_eq!(si.cast.x.unwrap_or(0), 0);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    assert_eq!(tapped_lands(&t, P0), 0);
    // It's exiled rather than put into a graveyard.
    assert!(t.in_exile("Blaze"));
}
