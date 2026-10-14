//! Rulings batch P184 — Cleric and Dinosaur typal cards: Edgewalker's colored cost
//! reduction (CR 118.7, 601.2f), "as long as you control a Dinosaur" statics (CR 611.3a),
//! intervening "if" clauses checked on trigger and resolution (CR 603.4), and an
//! "attacks while" condition checked only as the creature attacks.

use crate::r_s01_common::*;
use crate::r_s02_common::destroy;
use crate::r_s03_common::to_blockers;
use crate::r_s25_common::cast_new;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

/// Casts `card` for P0 (paying automatically with P0's lands), kicked if `kicked`;
/// whether it was cast.
fn try_cast(t: &mut TestGame, card: ObjectId, kicked: bool) -> bool {
    let ok = t.cast(P0, card).kicked(kicked).try_go().is_ok();
    t.clear_answers();
    ok
}

#[test]
fn edgewalker_reduces_a_hybrid_symbol_paid_with_white_or_black() {
    cr!("601.2f", "118.7", "107.4e");
    ruling!(
        "Edgewalker",
        "If a spell has hybrid mana symbols in its mana cost, you choose which half you will be paying before determining the total cost. If you choose to pay such a cost with {W} or {B}, Edgewalker can reduce that part of the cost."
    );
    supported("Edgewalker");
    supported("Moonrise Cleric");
    supported("Kirol, Attentive First-Year");
    // Moonrise Cleric ({1}{W/B}{W/B}) paid as {1}{W}{B} costs {1}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Edgewalker");
    let cleric = t.hand(P0, "Moonrise Cleric");
    t.lands(P0, "Wastes", 1);
    assert!(try_cast(&mut t, cleric, false));
    t.resolve_all();
    assert!(t.on_battlefield(cleric));
    // Without Edgewalker, one land isn't enough.
    let mut t = TestGame::new(2);
    let cleric = t.hand(P0, "Moonrise Cleric");
    t.lands(P0, "Wastes", 1);
    assert!(!try_cast(&mut t, cleric, false));
    // Kirol ({1}{R/W}{R/W}): only a half paid with {W} is reduced; the other is {R} or
    // {W} and still has to be paid.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Edgewalker");
    let kirol = t.hand(P0, "Kirol, Attentive First-Year");
    t.lands(P0, "Wastes", 2);
    assert!(!try_cast(&mut t, kirol, false));
    let mut t2 = TestGame::new(2);
    t2.battlefield(P0, "Edgewalker");
    let kirol = t2.hand(P0, "Kirol, Attentive First-Year");
    t2.lands(P0, "Wastes", 1);
    t2.lands(P0, "Mountain", 1);
    assert!(try_cast(&mut t2, kirol, false));
    assert_eq!(tapped_lands(&t2, P0), 2);
}

#[test]
fn edgewalker_reductions_are_cumulative_and_apply_to_additional_costs() {
    cr!("601.2f", "118.7", "702.33a");
    ruling!(
        "Edgewalker",
        "If you have more than one of these on the battlefield, the cost reduction is cumulative."
    );
    ruling!(
        "Edgewalker",
        "You apply cost reduction effects after other cost modifiers, so it can reduce additional costs or alternative costs if they include {W} and/or {B}."
    );
    supported("Tourach, Dread Cantor");
    supported("Phyrexian Missionary");
    // Tourach ({1}{B}, kicker {B}{B}) kicked costs {1}{B}{B}{B}: with two Edgewalkers,
    // {1}{B}.
    let tourach_kicked = |edgewalkers: usize| {
        let mut t = TestGame::new(2);
        for _ in 0..edgewalkers {
            t.battlefield(P0, "Edgewalker");
        }
        t.hand(P1, "Grizzly Bears");
        t.hand(P1, "Grizzly Bears");
        let tourach = t.hand(P0, "Tourach, Dread Cantor");
        t.lands(P0, "Wastes", 1);
        t.lands(P0, "Swamp", 1);
        // (The kicker is offered only if the total cost with it can be paid.)
        assert!(try_cast(&mut t, tourach, true));
        t.answer_targets(P0, &[Entity::Player(P1)]);
        t.resolve_all();
        // Kicked: P1 discarded two cards.
        t.hand_size(P1) == 0
    };
    assert!(!tourach_kicked(1));
    assert!(tourach_kicked(2));
    // Phyrexian Missionary ({1}{W}, kicker {1}{B}) kicked costs {2}{W}{B}: Edgewalker
    // reduces the kicker's {B} too, so it costs {2}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Edgewalker");
    let bears = t.graveyard(P0, "Grizzly Bears");
    let missionary = t.hand(P0, "Phyrexian Missionary");
    t.lands(P0, "Wastes", 2);
    assert!(try_cast(&mut t, missionary, true));
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.resolve_all();
    assert!(t.in_hand(P0, "Grizzly Bears"), "it was kicked");
}

#[test]
fn drover_of_the_mighty_marked_damage_becomes_lethal_without_a_dinosaur() {
    cr!("611.3a", "120.6", "704.5g", "514.2");
    ruling!(
        "Drover of the Mighty",
        "Because damage remains marked on a creature until it's removed as the turn ends, the damage Drover of the Mighty takes during combat may become lethal if you no longer control a Dinosaur later in the turn."
    );
    supported("Drover of the Mighty");
    let mut t = TestGame::new(2);
    let drover = t.battlefield(P0, "Drover of the Mighty");
    let dino = t.battlefield(P0, "Colossal Dreadmaw");
    let bears = t.battlefield(P1, "Grizzly Bears");
    assert_eq!(t.pt(drover), (3, 3));
    to_blockers(&mut t, &[(drover, Entity::Player(P1))], &[(bears, drover)]);
    t.advance_to(P0, Step::EndOfCombat);
    // Drover survived 2 damage as a 3/3.
    assert!(t.on_battlefield(drover));
    assert_eq!(t.obj_now(drover).damage, 2);
    t.advance_to(P0, Step::PostcombatMain);
    destroy(&mut t, dino);
    assert!(t.in_graveyard(P0, "Drover of the Mighty"));
}

#[test]
fn pugnacious_hammerskull_checks_only_as_it_attacks() {
    cr!("508.1m", "603.2", "122.1g");
    ruling!(
        "Pugnacious Hammerskull",
        "If you didn't control another Dinosaur when you declared Pugnacious Hammerskull as an attacker, it doesn't matter whether or not you control one as its ability resolves. You'll still put a stun counter on Pugnacious Hammerskull."
    );
    supported("Pugnacious Hammerskull");
    let mut t = TestGame::new(2);
    let hammer = t.battlefield(P0, "Pugnacious Hammerskull");
    attack_with(&mut t, &[(hammer, Entity::Player(P1))]);
    assert_eq!(t.stack_len(), 1);
    // A Dinosaur arrives before the ability resolves.
    t.battlefield(P0, "Colossal Dreadmaw");
    t.resolve_all();
    assert_eq!(t.counters(hammer, counters::STUN), 1);
    // Attacking while controlling another Dinosaur: nothing happens.
    let mut t = TestGame::new(2);
    let hammer = t.battlefield(P0, "Pugnacious Hammerskull");
    t.battlefield(P0, "Colossal Dreadmaw");
    attack_with(&mut t, &[(hammer, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(t.counters(hammer, counters::STUN), 0);
}

#[test]
fn tilonallis_knight_checks_for_a_dinosaur_on_trigger_and_resolution() {
    cr!("603.4", "611.2a");
    ruling!(
        "Tilonalli's Knight",
        "If you don’t control a Dinosaur as Tilonalli’s Knight attacks, its ability won’t trigger at all. If you don’t control a Dinosaur as the ability of Tilonalli’s Knight resolves, that ability has no effect."
    );
    ruling!(
        "Tilonalli's Knight",
        "Once the ability of Tilonalli’s Knight resolves while you control one or more Dinosaurs, Tilonalli’s Knight gets +1/+1 for the rest of the turn even if you no longer control a Dinosaur later in the turn."
    );
    supported("Tilonalli's Knight");
    // No Dinosaur: no trigger (even if one arrives right after).
    let mut t = TestGame::new(2);
    let knight = t.battlefield(P0, "Tilonalli's Knight");
    attack_with(&mut t, &[(knight, Entity::Player(P1))]);
    assert_eq!(t.stack_len(), 0);
    // A Dinosaur when it attacks, gone before resolution: no effect.
    let mut t = TestGame::new(2);
    let knight = t.battlefield(P0, "Tilonalli's Knight");
    let dino = t.battlefield(P0, "Colossal Dreadmaw");
    attack_with(&mut t, &[(knight, Entity::Player(P1))]);
    assert_eq!(t.stack_len(), 1);
    destroy(&mut t, dino);
    t.resolve_all();
    assert_eq!(t.pt(knight), (2, 2));
    // A Dinosaur on resolution: +1/+1 for the turn, even after the Dinosaur leaves.
    let mut t = TestGame::new(2);
    let knight = t.battlefield(P0, "Tilonalli's Knight");
    let dino = t.battlefield(P0, "Colossal Dreadmaw");
    attack_with(&mut t, &[(knight, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(t.pt(knight), (3, 3));
    destroy(&mut t, dino);
    assert_eq!(t.pt(knight), (3, 3));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.pt(knight), (3, 3));
}

#[test]
fn imperial_lancer_loses_double_strike_after_first_strike_damage() {
    cr!("611.3a", "702.4b", "510.4");
    ruling!(
        "Imperial Lancer",
        "If you no longer control a Dinosaur after the first-strike combat damage step, Imperial Lancer won’t have double strike, and so it won’t deal regular combat damage."
    );
    supported("Imperial Lancer");
    let lancer_damage = |lose_dino: bool| {
        let mut t = TestGame::new(2);
        let lancer = t.battlefield(P0, "Imperial Lancer");
        let dino = t.battlefield(P0, "Colossal Dreadmaw");
        attack_with(&mut t, &[(lancer, Entity::Player(P1))]);
        t.advance_to(P0, Step::FirstStrikeDamage);
        assert_eq!(t.life(P1), 19);
        if lose_dino {
            destroy(&mut t, dino);
        }
        t.advance_to(P0, Step::EndOfCombat);
        20 - t.life(P1)
    };
    assert_eq!(lancer_damage(false), 2);
    assert_eq!(lancer_damage(true), 1);
}

#[test]
fn palanis_hatcher_counts_any_egg() {
    cr!("205.3m", "603.4", "507.1");
    ruling!(
        "Palani's Hatcher",
        "Palani's Hatcher's last ability counts any creature you control with the creature type Egg, not just the tokens Palani's Hatcher creates."
    );
    supported("Palani's Hatcher");
    supported("Roc Egg");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Palani's Hatcher");
    // Roc Egg is a Bird Egg; it isn't a token Palani's Hatcher created.
    let egg = t.battlefield(P0, "Roc Egg");
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert!(!t.on_battlefield(egg), "the Roc Egg was sacrificed");
    let dinos: Vec<_> = tokens(&t, P0)
        .into_iter()
        .filter(|id| t.pt(*id) == (3, 3) && t.obj(*id).chars.has_subtype("Dinosaur"))
        .collect();
    assert_eq!(dinos.len(), 1);
}

