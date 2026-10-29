//! Rulings batch P147 — putting cards from a graveyard back into the library: "dies" triggers
//! that put the card on top only if it's still in the graveyard (CR 400.7, 603.10a), spells
//! with "any number of targets" that can be cast with none but don't resolve when all
//! their targets became illegal (CR 608.2b), Dead Reckoning's two targets and last known
//! power, and who controls a dies trigger of a stolen creature.

use crate::r_p147_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

// --- Any number of targets ----------------------------------------------------------------

/// The "put any number of target <kind> cards from your graveyard on top of your library.
/// Draw a card." spells, with a card of the right kind for each.
const RESTOCK_DRAW: &[(&str, &str)] = &[
    ("Gravepurge", "Grizzly Bears"),
    ("Forever Young", "Grizzly Bears"),
    ("Footbottom Feast", "Grizzly Bears"),
    ("Frantic Salvage", "Ornithopter"),
];

#[test]
fn restock_spells_with_zero_targets_just_draw() {
    cr!("601.2c", "115.1d");
    ruling!(
        "Gravepurge",
        "You can cast Gravepurge with zero targets. If you do, you’ll draw a card as the spell resolves."
    );
    ruling!(
        "Gravepurge",
        "You can cast Gravepurge with no targets. If you do, you’ll just draw a card."
    );
    ruling!(
        "Forever Young",
        "You can cast Forever Young with no targets if you just want to draw a card."
    );
    ruling!(
        "Frantic Salvage",
        "You can cast this with zero targets. If you do, you’ll get to draw a card. However, if you cast this with at least one target and all of those targets become illegal (because another effect has removed them from the graveyard, perhaps)"
    );
    ruling!(
        "Footbottom Feast",
        "You can cast this with zero targets. If you do, you’ll get to draw a card. However, if you cast this with at least one target and all of those targets become illegal, the spell won’t resolve"
    );
    for (name, _) in RESTOCK_DRAW {
        supported(name);
        let mut t = TestGame::new(2);
        let hand = t.hand_size(P0);
        let spell = cast_new(&mut t, P0, name, &[]);
        t.resolve_all();
        assert!(resolved(&t, spell), "{name}");
        assert_eq!(t.hand_size(P0), hand + 1, "{name}");
    }
}

#[test]
fn restock_spells_whose_targets_all_became_illegal_dont_resolve() {
    cr!("608.2b");
    ruling!(
        "Gravepurge",
        "If you cast Gravepurge with at least one target and all of those targets become illegal, the spell won’t resolve and you won’t get to draw a card."
    );
    ruling!(
        "Gravepurge",
        "However, if you cast it with at least one target and all of those targets become illegal, the spell won’t resolve and you won’t draw a card."
    );
    ruling!(
        "Forever Young",
        "If you choose any target creature cards and they are all illegal targets by the time Forever Young tries to resolve, the spell won't resolve. You won't draw a card."
    );
    for (name, card) in RESTOCK_DRAW {
        // Positive control: with a legal target it's put on top and a card is drawn (the
        // target itself).
        let mut t = TestGame::new(2);
        let c = t.graveyard(P0, card);
        cast_new(&mut t, P0, name, &[obj(c)]);
        t.resolve_all();
        assert!(t.in_hand(P0, card), "{name}");
        // The target leaves the graveyard: no draw.
        let mut t = TestGame::new(2);
        let c = t.graveyard(P0, card);
        let hand = t.hand_size(P0);
        let spell = cast_new(&mut t, P0, name, &[obj(c)]);
        move_to(&mut t, c, Zone::Exile);
        t.resolve_all();
        assert!(!resolved(&t, spell), "{name}");
        assert_eq!(t.hand_size(P0), hand, "{name}");
    }
}

/// P0 casts Forever Young targeting Grizzly Bears and Hill Giant in P0's graveyard and
/// orders them with the permutation `order`; returns the card drawn (the one put on top).
fn forever_young_draw(order: [usize; 2]) -> String {
    let cards = ["Grizzly Bears", "Hill Giant"];
    let mut t = TestGame::new(2);
    let a = t.graveyard(P0, cards[0]);
    let b = t.graveyard(P0, cards[1]);
    t.answer(P0, DecisionKind::Order, Answer::Indices(order.to_vec()));
    let spell = cast_multi(&mut t, P0, "Forever Young", &[obj(a), obj(b)]);
    t.resolve_all();
    assert!(resolved(&t, spell));
    let drawn: Vec<String> = hand_names(&t, P0)
        .into_iter()
        .filter(|n| cards.contains(&n.as_str()))
        .collect();
    assert_eq!(drawn.len(), 1, "{drawn:?}");
    assert!(library_names(&t, P0).iter().any(|n| cards.contains(&n.as_str())));
    drawn[0].clone()
}

#[test]
fn forever_young_order_is_chosen_and_the_top_one_is_drawn() {
    cr!("401.4", "608.2c");
    ruling!(
        "Forever Young",
        "You choose the order that the creature cards are put on top of your library. You don't have to reveal this order to other players. Whichever ends up on top will be the card you draw."
    );
    assert_ne!(forever_young_draw([0, 1]), forever_young_draw([1, 0]));
}

#[test]
fn misinformation_targets_cards_not_the_opponent() {
    cr!("115.1", "702.11b");
    ruling!(
        "Misinformation",
        "You target the cards, but not the opponent."
    );
    supported("Leyline of Sanctity");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Leyline of Sanctity");
    let a = t.graveyard(P1, "Grizzly Bears");
    let b = t.graveyard(P1, "Hill Giant");
    supported("Misinformation");
    let spell = cast_multi(&mut t, P0, "Misinformation", &[obj(a), obj(b)]);
    t.resolve_all();
    assert!(resolved(&t, spell));
    let top2: Vec<String> = library_names(&t, P1).into_iter().take(2).collect();
    assert!(top2.contains(&"Grizzly Bears".to_string()) && top2.contains(&"Hill Giant".to_string()));
}

#[test]
fn misinformation_the_cards_owner_arranges_them() {
    cr!("401.4");
    interp!("library-arranged-by-owner-not-instructing-player");
    // Current behavior (provisional): the owner of the cards (P1) orders them, not the
    // caster, although Misinformation's ruling says the caster puts them in any order.
    let cards = ["Grizzly Bears", "Hill Giant"];
    let mut tops = vec![];
    for order in [[0usize, 1], [1, 0]] {
        let mut t = TestGame::new(2);
        let a = t.graveyard(P1, cards[0]);
        let b = t.graveyard(P1, cards[1]);
        let from = n_asked(&t);
        t.answer(P1, DecisionKind::Order, Answer::Indices(order.to_vec()));
        let spell = cast_multi(&mut t, P0, "Misinformation", &[obj(a), obj(b)]);
        t.resolve_all();
        assert!(resolved(&t, spell));
        let orders: Vec<PlayerId> = t.asked()[from..]
            .iter()
            .filter(|(_, d)| matches!(d, mtg_engine::decision::Decision::Order { .. }))
            .map(|(p, _)| *p)
            .collect();
        assert_eq!(orders, vec![P1]);
        tops.push(top_name(&t, P1));
    }
    assert_ne!(tops[0], tops[1]);
}

// --- Dead Reckoning ------------------------------------------------------------------------

#[test]
fn dead_reckoning_needs_both_targets() {
    cr!("601.2c", "115.1");
    ruling!(
        "Dead Reckoning",
        "You must choose two targets as you cast Dead Reckoning: a creature card in your graveyard and a creature on the battlefield. If you can’t, then you can’t cast the spell."
    );
    supported("Dead Reckoning");
    use mtg_engine::object::CastMethod;
    // No creature card in P0's graveyard.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Hill Giant");
    lands_for_cost(&mut t, P0, "Dead Reckoning");
    let dr = t.hand(P0, "Dead Reckoning");
    assert!(!can_cast(&mut t, P0, dr, CastMethod::Normal));
    // No creature on the battlefield.
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Hill Giant");
    lands_for_cost(&mut t, P0, "Dead Reckoning");
    let dr = t.hand(P0, "Dead Reckoning");
    assert!(!can_cast(&mut t, P0, dr, CastMethod::Normal));
    // Both: castable.
    t.battlefield(P1, "Grizzly Bears");
    assert!(can_cast(&mut t, P0, dr, CastMethod::Normal));
}

#[test]
fn dead_reckoning_uses_the_power_the_card_had_in_the_graveyard() {
    cr!("608.2h", "604.3");
    ruling!(
        "Dead Reckoning",
        "Dead Reckoning deals damage equal to the power the creature card had while it was in your graveyard."
    );
    supported("Tarmogoyf");
    let mut t = TestGame::new(2);
    let goyf = t.graveyard(P0, "Tarmogoyf");
    t.graveyard(P0, "Lightning Bolt");
    t.graveyard(P0, "Island");
    let giant = t.battlefield(P1, "Hill Giant");
    t.g.recompute();
    // Creature, instant and land in graveyards: 3/4 there; once it's on top of the
    // library, only instant and land remain (2).
    assert_eq!(t.obj_now(goyf).power(), 3);
    yes(&mut t, P0);
    cast_new(&mut t, P0, "Dead Reckoning", &[obj(goyf), obj(giant)]);
    t.resolve_all();
    assert_eq!(top_name(&t, P0), "Tarmogoyf");
    assert!(t.in_graveyard(P1, "Hill Giant"), "3 damage wasn't dealt");
}

#[test]
fn dead_reckoning_with_an_illegal_target() {
    cr!("608.2b", "608.2c");
    ruling!(
        "Dead Reckoning",
        "If the targeted creature card is an illegal target as Dead Reckoning resolves (because it’s no longer in your graveyard, perhaps), you can’t choose to put it on top of your library. No damage will be dealt."
    );
    ruling!(
        "Dead Reckoning",
        "If the targeted creature is an illegal target as Dead Reckoning resolves (because it’s no longer on the battlefield, perhaps), you may still put the targeted creature card from your graveyard on top of your library. No damage will be dealt."
    );
    // The card left the graveyard.
    let mut t = TestGame::new(2);
    let card = t.graveyard(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    yes(&mut t, P0);
    cast_new(&mut t, P0, "Dead Reckoning", &[obj(card), obj(bears)]);
    move_to(&mut t, card, Zone::Exile);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert_eq!(t.obj_now(bears).damage, 0);
    assert!(t.in_exile("Hill Giant"));
    // The creature left the battlefield: the card is still put on top.
    let mut t = TestGame::new(2);
    let card = t.graveyard(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let other = t.battlefield(P1, "Grizzly Bears");
    yes(&mut t, P0);
    cast_new(&mut t, P0, "Dead Reckoning", &[obj(card), obj(bears)]);
    move_to(&mut t, bears, Zone::Hand(P1));
    t.resolve_all();
    assert_eq!(top_name(&t, P0), "Hill Giant");
    assert_eq!(t.obj_now(other).damage, 0);
    assert_eq!(t.life(P1), 20);
}

// --- Dies triggers that put the card into the library ---------------------------------------

#[test]
fn dies_triggers_do_nothing_once_the_card_left_the_graveyard() {
    cr!("400.7", "603.10a", "608.2b");
    ruling!(
        "Arashin Sovereign",
        "Arashin Sovereign will be put on the top or bottom of that library only if it’s still in the graveyard when its ability resolves. If it leaves the graveyard before that point, it will stay in whatever zone it’s in, even if it’s returned to the graveyard before the ability resolves."
    );
    ruling!(
        "Dark Revenant",
        "Dark Revenant’s ability isn’t optional. However, if Dark Revenant leaves the graveyard before its ability resolves, it won’t be put on top of its owner’s library."
    );
    supported("Arashin Sovereign");
    supported("Dark Revenant");
    // Dark Revenant: put on top without being asked.
    let mut t = TestGame::new(2);
    let r = t.battlefield(P0, "Dark Revenant");
    destroy(&mut t, r);
    let from = n_asked(&t);
    t.resolve_all();
    assert_eq!(top_name(&t, P0), "Dark Revenant");
    assert!(!t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, mtg_engine::decision::Decision::YesNo { .. })));
    // Leaves the graveyard first: stays in exile.
    let mut t = TestGame::new(2);
    let r = t.battlefield(P0, "Dark Revenant");
    destroy(&mut t, r);
    move_to(&mut t, r, Zone::Exile);
    t.resolve_all();
    assert!(t.in_exile("Dark Revenant"));
    // Arashin Sovereign: exiled and returned to the graveyard; it's a new object.
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Arashin Sovereign");
    destroy(&mut t, s);
    move_to(&mut t, s, Zone::Exile);
    move_to(&mut t, s, Zone::Graveyard(P0));
    yes(&mut t, P0);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Arashin Sovereign"));
}

#[test]
fn arashin_sovereign_its_last_controller_chooses_top_bottom_or_neither() {
    cr!("603.3a", "608.2d");
    ruling!(
        "Arashin Sovereign",
        "The player who controlled Arashin Sovereign when it died chooses whether to put it on the top or bottom of its owner’s library. The player may also choose to leave Arashin Sovereign in the graveyard."
    );
    // P0 controls P1's Sovereign when it dies. Each answer: (yes?, option index).
    let mut results = vec![];
    for (put, option) in [(false, 0), (true, 0), (true, 1)] {
        let mut t = TestGame::new(2);
        stack_library(&mut t, P1, &["Island", "Island"]);
        let s = t.battlefield(P1, "Arashin Sovereign");
        take_control(&mut t, P0, s);
        destroy(&mut t, s);
        let from = n_asked(&t);
        t.answer_yes(P0, put);
        t.answer(P0, DecisionKind::Option, Answer::Index(option));
        t.resolve_all();
        // Only P0 was asked.
        assert!(t.asked()[from..].iter().all(|(p, _)| *p == P0));
        let lib = library_names(&t, P1);
        results.push((
            t.in_graveyard(P1, "Arashin Sovereign"),
            lib.first().map(|n| n == "Arashin Sovereign").unwrap_or(false),
            lib.last().map(|n| n == "Arashin Sovereign").unwrap_or(false),
        ));
    }
    assert_eq!(results[0], (true, false, false));
    assert!(!results[1].0 && !results[2].0);
    // The two options put it on top and on the bottom.
    assert_ne!(results[1], results[2]);
    assert!(results[1].1 || results[1].2);
    assert!(results[2].1 || results[2].2);
}

#[test]
fn avenging_angel_goes_to_the_graveyard_and_its_death_triggers_other_abilities() {
    cr!("603.6c", "700.4");
    ruling!(
        "Avenging Angel",
        "The Angel does go to the graveyard, and will trigger any other abilities that look for this event."
    );
    supported("Avenging Angel");
    supported("Blood Artist");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Blood Artist");
    let a = t.battlefield(P0, "Avenging Angel");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    destroy(&mut t, a);
    assert!(t.in_graveyard(P0, "Avenging Angel"));
    assert_eq!(stacked_triggers(&t), 2);
    yes(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    assert_eq!(top_name(&t, P0), "Avenging Angel");
}

#[test]
fn mortuary_is_not_optional() {
    cr!("603.5");
    ruling!("Mortuary", "This is not an optional ability.");
    supported("Mortuary");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mortuary");
    let b = t.battlefield(P0, "Grizzly Bears");
    t.answer_yes(P0, false);
    destroy(&mut t, b);
    t.resolve_all();
    assert_eq!(top_name(&t, P0), "Grizzly Bears");
}

#[test]
fn bubble_up_cant_target_an_adventurer_card() {
    cr!("715.4", "115.1");
    ruling!(
        "Aquatic Alchemist // Bubble Up",
        "Adventurer cards aren't instant or sorcery cards while they're in your graveyard. You can't use Bubble Up to put an adventurer card from your graveyard on top of your library."
    );
    supported("Aquatic Alchemist // Bubble Up");
    let mut t = TestGame::new(2);
    let adv = t.graveyard(P0, "Brazen Borrower // Petty Theft");
    let bolt = t.graveyard(P0, "Lightning Bolt");
    t.lands(P0, "Island", 3);
    let card = t.hand(P0, "Aquatic Alchemist // Bubble Up");
    let from = n_asked(&t);
    t.answer_targets(P0, &[obj(bolt)]);
    t.cast(P0, card)
        .method(crate::r_s18_common::ADVENTURE)
        .go();
    assert!(offered_target(&t, P0, from, bolt));
    assert!(!offered_target(&t, P0, from, adv));
    t.resolve_all();
    assert_eq!(top_name(&t, P0), "Lightning Bolt");
}

// --- Barishi, Enhanced Surveillance -------------------------------------------------------

#[test]
fn barishi_shuffles_even_if_it_left_the_graveyard() {
    cr!("608.2b", "603.10a");
    ruling!(
        "Barishi",
        "If Barishi isn't in your graveyard by the time the ability resolves, you still shuffle all creature cards from your graveyard into your library."
    );
    supported("Barishi");
    let mut t = TestGame::new(2);
    let b = t.battlefield(P0, "Barishi");
    t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P0, "Lightning Bolt");
    destroy(&mut t, b);
    move_to(&mut t, b, Zone::Hand(P0));
    t.resolve_all();
    assert!(t.in_hand(P0, "Barishi"));
    assert!(!t.in_graveyard(P0, "Grizzly Bears"));
    assert!(t.in_graveyard(P0, "Lightning Bolt"));
    assert!(library_names(&t, P0).iter().any(|n| n == "Grizzly Bears"));
}

#[test]
fn barishi_controlled_by_its_last_controller_whose_graveyard_matters() {
    cr!("603.3a", "109.5");
    ruling!(
        "Barishi",
        "The player who controlled Barishi when it left the battlefield is the player who controls the ability. It doesn't matter whose graveyard Barishi is put into."
    );
    let mut t = TestGame::new(2);
    let b = t.battlefield(P1, "Barishi");
    take_control(&mut t, P0, b);
    t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P1, "Hill Giant");
    destroy(&mut t, b);
    assert!(t.in_graveyard(P1, "Barishi"));
    t.resolve_all();
    // P0's graveyard was shuffled into P0's library; P1's Hill Giant stays.
    assert!(!t.in_graveyard(P0, "Grizzly Bears"));
    assert!(library_names(&t, P0).iter().any(|n| n == "Grizzly Bears"));
    assert!(t.in_graveyard(P1, "Hill Giant"));
    // Barishi itself was exiled.
    assert!(t.in_exile("Barishi"));
}

#[test]
fn enhanced_surveillance_last_ability_only_on_the_battlefield() {
    cr!("602.1", "113.6");
    ruling!(
        "Enhanced Surveillance",
        "Enhanced Surveillance's last ability can be activated only while it's on the battlefield."
    );
    supported("Enhanced Surveillance");
    let mut t = TestGame::new(2);
    let h = t.hand(P0, "Enhanced Surveillance");
    assert!(!can_activate(&mut t, P0, h));
    let gy = t.graveyard(P0, "Enhanced Surveillance");
    assert!(!can_activate(&mut t, P0, gy));
    let e = t.battlefield(P0, "Enhanced Surveillance");
    t.graveyard(P0, "Grizzly Bears");
    assert!(can_activate(&mut t, P0, e));
    activate_resolve(&mut t, P0, e, 0, &[]);
    assert!(t.in_exile("Enhanced Surveillance"));
    assert!(!t.in_graveyard(P0, "Grizzly Bears"));
}

// --- Life totals and spells that cause triggers --------------------------------------------

#[test]
fn beacon_of_immortality_doubling_is_life_gain() {
    cr!("119.5", "119.9");
    ruling!(
        "Beacon of Immortality",
        "Beacon of Immortality’s effect counts as life gain (or life loss, if the life total was negative) for effects that trigger on or replace life gain"
    );
    supported("Beacon of Immortality");
    supported("Ajani's Pridemate");
    let mut t = TestGame::new(2);
    let pm = t.battlefield(P0, "Ajani's Pridemate");
    t.g.players[P0.idx()].life = 7;
    cast_new(&mut t, P0, "Beacon of Immortality", &[Entity::Player(P0)]);
    t.resolve_all();
    assert_eq!(t.life(P0), 14);
    assert_eq!(t.counters(pm, "+1/+1"), 1);
    // Shuffled into the library.
    assert!(library_names(&t, P0).iter().any(|n| n == "Beacon of Immortality"));
}

#[test]
fn oketra_trigger_resolves_first_even_if_the_spell_is_countered() {
    cr!("603.3", "405.5", "601.2i");
    ruling!(
        "God-Eternal Oketra",
        "Oketra's first triggered ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered."
    );
    supported("God-Eternal Oketra");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "God-Eternal Oketra");
    let spell = cast_new(&mut t, P0, "Grizzly Bears", &[]);
    t.settle();
    assert_eq!(stacked_triggers(&t), 1);
    // The trigger is above the spell.
    t.resolve();
    assert_eq!(tokens(&t, P0).len(), 1);
    assert_eq!(t.zone(spell), Zone::Stack);
    // Countered spell, trigger still resolves.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "God-Eternal Oketra");
    let spell = cast_new(&mut t, P0, "Grizzly Bears", &[]);
    t.settle();
    counter(&mut t, spell);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(tokens(&t, P0).len(), 1);
}
