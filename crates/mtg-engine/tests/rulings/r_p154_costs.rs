//! Rulings batch P154 — spells whose cost is reduced by mana values on the battlefield
//! (CR 601.2f: the total cost is determined once, after which mana abilities are activated
//! and the cost paid, 601.2g–h; 118.7a: only generic mana is reduced), the additional
//! sacrifice cost of Eldritch Evolution (CR 601.2b, 601.2h), and mana values ignoring
//! kicker, cost reductions and what was paid (CR 202.3).

use crate::r_p154_common::*;
use mtg_engine::decision::Decision;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn earthquake_dragon_and_metalwork_colossus_cost_the_colored_or_nothing_when_reduced_enough() {
    cr!("601.2f", "118.7a");
    ruling!(
        "Earthquake Dragon",
        "If the total mana value of Dragons you control is 14 or greater, Earthquake Dragon costs {G} to cast."
    );
    ruling!(
        "Metalwork Colossus",
        "If the total mana value of noncreature artifacts you control is 11 or greater, Metalwork Colossus costs {0} to cast."
    );
    supported("Earthquake Dragon");
    supported("Metalwork Colossus");
    // Three Dragons of mana value 6 (18 >= 14): Earthquake Dragon ({14}{G}) costs {G}.
    let mut t = TestGame::new(2);
    for _ in 0..3 {
        t.battlefield(P0, "Shivan Dragon");
    }
    let dragon = t.hand(P0, "Earthquake Dragon");
    t.lands(P0, "Forest", 1);
    assert!(can_cast(&mut t, P0, dragon, CastMethod::Normal));
    t.cast(P0, dragon).go();
    t.resolve_all();
    assert_eq!(count_named(&t, "Earthquake Dragon"), 1);
    // Two Dragons (12): {2}{G} is needed; one Forest isn't enough.
    let mut t = TestGame::new(2);
    for _ in 0..2 {
        t.battlefield(P0, "Shivan Dragon");
    }
    let dragon = t.hand(P0, "Earthquake Dragon");
    t.lands(P0, "Forest", 1);
    assert!(!can_cast(&mut t, P0, dragon, CastMethod::Normal));
    t.lands(P0, "Forest", 2);
    assert!(can_cast(&mut t, P0, dragon, CastMethod::Normal));
    // Darksteel Forge (9) and Mind Stone (2): Metalwork Colossus ({11}) costs {0}; with
    // the Forge alone it costs {2}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Darksteel Forge");
    let colossus = t.hand(P0, "Metalwork Colossus");
    assert!(!can_cast(&mut t, P0, colossus, CastMethod::Normal));
    t.battlefield(P0, "Mind Stone");
    assert!(can_cast(&mut t, P0, colossus, CastMethod::Normal));
    t.cast(P0, colossus).go();
    t.resolve_all();
    assert_eq!(count_named(&t, "Metalwork Colossus"), 1);
}

#[test]
fn metalwork_colossus_returns_itself_only_from_the_graveyard() {
    cr!("113.6", "602.2");
    ruling!(
        "Metalwork Colossus",
        "Metalwork Colossus's last ability can be activated only if it is in your graveyard."
    );
    supported("Metalwork Colossus");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mind Stone");
    t.battlefield(P0, "Mind Stone");
    let on_bf = t.battlefield(P0, "Metalwork Colossus");
    assert!(!can_activate(&mut t, P0, on_bf), "not from the battlefield");
    let in_hand = t.hand(P0, "Metalwork Colossus");
    assert!(!can_activate(&mut t, P0, in_hand), "not from hand");
    let in_gy = t.graveyard(P0, "Metalwork Colossus");
    assert!(can_activate(&mut t, P0, in_gy));
    let stones: Vec<Entity> = t.named_on_battlefield("Mind Stone").into_iter().map(obj).collect();
    t.answer_choose(P0, &stones);
    t.activate(P0, in_gy, 0, &[]).expect("Colossus");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 2, "the Colossus card joined the one in hand");
}

/// Casts `card` (in P0's hand, with lands or permanents already set up) and returns the
/// decisions other players were asked while it was being cast (none is expected, CR
/// 601.2: no player receives priority until the spell is cast).
fn others_during_cast(t: &mut TestGame, card: ObjectId, answers: impl FnOnce(&mut TestGame)) -> Vec<(PlayerId, Decision)> {
    let from = t.asked().len();
    answers(t);
    t.cast(P0, card).go();
    others_asked_since(t, from, P0)
}

#[test]
fn no_player_may_act_while_a_cost_reducing_spell_is_being_cast() {
    cr!("601.2", "601.2f", "601.2h", "117.3c");
    ruling!(
        "Earthquake Dragon",
        "Once a player has announced that they are casting Earthquake Dragon, no player may take actions to try and change the number of Dragons its controller controls before that spell's cost is locked in."
    );
    ruling!(
        "Metalwork Colossus",
        "Once a player has announced that they are casting Metalwork Colossus, no player may take actions to try to change the number of artifacts its controller controls before that spell's cost is locked in."
    );
    ruling!(
        "Sunderflock",
        "Once you announce you're casting a spell, no player may take actions until the spell has been paid for. Notably, opponents can't try to remove Elementals you control from the battlefield at that time."
    );
    ruling!(
        "Excalibur, Sword of Eden",
        "Once you announce you’re casting a spell, no player may take actions until the spell has been paid for. Notably, opponents can’t try to remove historic permanents you control from the battlefield at that time."
    );
    ruling!(
        "Eldritch Evolution",
        "Once you begin to cast Eldritch Evolution, no player may take actions until you're done. Notably, opponents can't try to remove the creature you wish to sacrifice."
    );
    supported("Sunderflock");
    supported("Excalibur, Sword of Eden");
    // Earthquake Dragon with 18 mana value of Dragons: cast for {G}; P1 (with a Shock in
    // hand and mana) gets no decision in between, and the Dragons are all still there.
    let mut t = TestGame::new(2);
    for _ in 0..3 {
        t.battlefield(P0, "Shivan Dragon");
    }
    t.hand(P1, "Shock");
    mana(&mut t, P1, "R");
    let dragon = t.hand(P0, "Earthquake Dragon");
    t.lands(P0, "Forest", 1);
    assert!(others_during_cast(&mut t, dragon, |_| {}).is_empty());
    assert_eq!(count_named(&t, "Shivan Dragon"), 3);
    // Metalwork Colossus for {0}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Darksteel Forge");
    t.battlefield(P0, "Mind Stone");
    t.hand(P1, "Shock");
    mana(&mut t, P1, "R");
    let colossus = t.hand(P0, "Metalwork Colossus");
    assert!(others_during_cast(&mut t, colossus, |_| {}).is_empty());
    // Sunderflock ({7}{U}{U}) with an Air Elemental (5): {2}{U}{U}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Air Elemental");
    t.hand(P1, "Shock");
    mana(&mut t, P1, "R");
    let flock = t.hand(P0, "Sunderflock");
    t.lands(P0, "Island", 4);
    assert!(others_during_cast(&mut t, flock, |_| {}).is_empty());
    assert_eq!(t.stack_len(), 1);
    // Excalibur ({12}) with Darksteel Forge (9) and Mind Stone (2): {1}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Darksteel Forge");
    t.battlefield(P0, "Mind Stone");
    t.hand(P1, "Shock");
    mana(&mut t, P1, "R");
    let sword = t.hand(P0, "Excalibur, Sword of Eden");
    t.lands(P0, "Wastes", 1);
    assert!(others_during_cast(&mut t, sword, |_| {}).is_empty());
    // Eldritch Evolution sacrificing Grizzly Bears as it's cast: the Bears are gone before
    // anyone could act.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.hand(P1, "Shock");
    mana(&mut t, P1, "R");
    let evo = t.hand(P0, "Eldritch Evolution");
    mana(&mut t, P0, "1GG");
    let others = others_during_cast(&mut t, evo, |t| {
        t.answer_choose(P0, &[obj(bears)]);
    });
    assert!(others.is_empty());
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
}

#[test]
fn a_cost_reduced_by_mana_values_stays_determined_while_mana_abilities_are_activated() {
    cr!("601.2f", "601.2g", "601.2h");
    ruling!(
        "Excalibur, Sword of Eden",
        "Once you determine the cost to cast Excalibur, you may activate mana abilities to pay that cost. If the total mana value of historic permanents you control changes while activating mana abilities (probably because you sacrificed one or more historic permanents), the cost to cast Excalibur remains what you previously determined."
    );
    ruling!(
        "Sunderflock",
        "Once you determine the cost to cast Sunderflock, you may activate mana abilities to pay that cost. If the greatest mana value among Elementals you control changes while activating mana abilities (probably because you sacrificed one or more Elementals), the cost to cast Sunderflock remains what you previously determined."
    );
    supported("Catalyst Elemental");
    supported("Lotus Ring");
    // Sunderflock ({7}{U}{U}) with Catalyst Elemental (3) its only Elemental: {4}{U}{U}.
    // The only way to pay is to sacrifice the Elemental for {R}{R} (two Islands and two
    // Wastes pay the rest): the cost doesn't go back up to {7}{U}{U} when it's gone.
    let mut t = TestGame::new(2);
    let cat = t.battlefield(P0, "Catalyst Elemental");
    let flock = t.hand(P0, "Sunderflock");
    t.lands(P0, "Island", 2);
    t.lands(P0, "Wastes", 2);
    assert!(can_cast(&mut t, P0, flock, CastMethod::Normal));
    t.cast(P0, flock).go();
    assert!(t.in_graveyard(P0, "Catalyst Elemental"), "sacrificed for mana");
    assert!(!t.on_battlefield(cat));
    t.resolve_all();
    assert_eq!(count_named(&t, "Sunderflock"), 1);
    // Excalibur ({12}) with historic permanents Lotus Ring (3), Juggernaut (4),
    // Bonesplitter (1) and Isamaru (1), total 9: {3}. The only mana is Isamaru's, from the
    // Ring ("{T}, Sacrifice this creature: Add three mana of any one color"): the cost
    // stays {3} although Isamaru's mana value is gone.
    let mut t = TestGame::new(2);
    let ring = t.battlefield(P0, "Lotus Ring");
    t.battlefield(P0, "Juggernaut");
    t.battlefield(P0, "Bonesplitter");
    let isamaru = t.battlefield(P0, "Isamaru, Hound of Konda");
    assert!(t.g.attach(ring, obj(isamaru)));
    t.g.recompute();
    let sword = t.hand(P0, "Excalibur, Sword of Eden");
    assert!(can_cast(&mut t, P0, sword, CastMethod::Normal));
    t.cast(P0, sword).go();
    assert!(t.in_graveyard(P0, "Isamaru, Hound of Konda"), "sacrificed for mana");
    t.resolve_all();
    assert_eq!(count_named(&t, "Excalibur, Sword of Eden"), 1);
}

#[test]
fn eldritch_evolutions_sacrifice_is_a_cost() {
    cr!("601.2b", "601.2h", "118.3");
    ruling!(
        "Eldritch Evolution",
        "Sacrificing a creature is part of Eldritch Evolution's cost. You can't sacrifice more creatures to search for more creature cards, and you can't cast Eldritch Evolution at all if you control no creatures."
    );
    supported("Eldritch Evolution");
    // No creatures: it can't be cast.
    let mut t = TestGame::new(2);
    let evo = t.hand(P0, "Eldritch Evolution");
    mana(&mut t, P0, "1GG");
    assert!(!can_cast(&mut t, P0, evo, CastMethod::Normal));
    // Two creatures: exactly one is sacrificed, and one card is searched for.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let wurm = t.library_top(P0, "Craw Wurm");
    t.library_top(P0, "Air Elemental");
    let evo = t.hand(P0, "Eldritch Evolution");
    mana(&mut t, P0, "1GG");
    t.answer_choose(P0, &[obj(giant)]);
    t.answer_choose(P0, &[obj(wurm)]);
    t.cast(P0, evo).go();
    assert!(t.in_graveyard(P0, "Hill Giant"));
    assert!(t.on_battlefield(bears), "only one creature is sacrificed");
    t.resolve_all();
    assert_eq!(count_named(&t, "Craw Wurm"), 1);
    assert_eq!(count_named(&t, "Air Elemental"), 0);
    assert!(t.in_exile("Eldritch Evolution"));
}

#[test]
fn manaplasm_uses_the_mana_value_not_the_cost_paid() {
    cr!("202.3", "601.2f", "118.8d", "118.9c");
    ruling!(
        "Manaplasm",
        "Mana value takes into account only the mana symbols printed in the upper right corner of the card. Manaplasm doesn't care about additional costs, alternative costs, cost-reduction effects, or what you actually paid to cast the spell."
    );
    supported("Manaplasm");
    // Manaplasm is a 1/1. Kicked Burst Lightning ({R}, kicker {4}): +1/+1, not +5/+5.
    let mut t = TestGame::new(2);
    let plasm = t.battlefield(P0, "Manaplasm");
    let burst = t.hand(P0, "Burst Lightning");
    mana(&mut t, P0, "R4");
    t.cast(P0, burst).kicked(true).target(pl(P1)).go();
    t.resolve();
    assert_eq!(t.pt(plasm), (2, 2), "{{R}} is 1");
    // Metalwork Colossus cast for {0}: +11/+11.
    let mut t = TestGame::new(2);
    let plasm = t.battlefield(P0, "Manaplasm");
    t.battlefield(P0, "Darksteel Forge");
    t.battlefield(P0, "Mind Stone");
    let colossus = t.hand(P0, "Metalwork Colossus");
    t.cast(P0, colossus).go();
    t.resolve();
    assert_eq!(t.pt(plasm), (12, 12));
    // Mulldrifter evoked ({2}{U}, an alternative cost): +5/+5.
    let mut t = TestGame::new(2);
    let plasm = t.battlefield(P0, "Manaplasm");
    let drifter = t.hand(P0, "Mulldrifter");
    mana(&mut t, P0, "2U");
    t.cast(P0, drifter)
        .method(CastMethod::Keyword(mtg_engine::keywords::KeywordKind::Evoke))
        .go();
    t.resolve();
    assert_eq!(t.pt(plasm), (6, 6));
}
