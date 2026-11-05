//! Rulings batch P179 — "tome" artifacts: activated abilities that draw cards (Halo
//! Fountain, Liar's Pendulum, Relic Vial, Temporal Aperture, ...), Druidic Satchel and the
//! discard-a-card additional cost of Tormenting Voice and friends.

use crate::r_p179_common::*;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn halo_fountain_creature_tapped_for_mana_can_be_untapped_for_the_cost() {
    cr!("601.2g", "602.2b");
    ruling!(
        "Halo Fountain",
        "you may both tap it for mana and untap it to pay costs for Halo Fountain's activated abilities."
    );
    supported("Halo Fountain");
    supported("Avacyn's Pilgrim");
    // Halo Fountain and an untapped Avacyn's Pilgrim, no other mana and no tapped
    // creature: the Pilgrim pays {W} and is then untapped for the cost.
    let mut t = TestGame::new(2);
    let fountain = t.battlefield(P0, "Halo Fountain");
    let pilgrim = t.battlefield(P0, "Avacyn's Pilgrim");
    assert!(can_activate(&mut t, P0, fountain));
    activate_containing(&mut t, P0, fountain, "Citizen").expect("activate");
    assert!(!t.obj_now(pilgrim).tapped);
    assert!(t.obj_now(fountain).tapped);
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 1);
    // With a Plains for the mana too, the Pilgrim still ends up tapped for mana and then
    // untapped.
    let mut t = TestGame::new(2);
    let fountain = t.battlefield(P0, "Halo Fountain");
    let pilgrim = t.battlefield(P0, "Avacyn's Pilgrim");
    t.battlefield(P0, "Plains");
    activate_containing(&mut t, P0, fountain, "Citizen").expect("activate");
    assert!(!t.obj_now(pilgrim).tapped);
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 1);
    // Control: a creature with no mana ability, untapped, can't be untapped for the cost.
    let mut t = TestGame::new(2);
    let fountain = t.battlefield(P0, "Halo Fountain");
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Plains");
    assert!(!can_activate(&mut t, P0, fountain));
    assert!(activate_containing(&mut t, P0, fountain, "Citizen").is_err());
}

#[test]
fn halo_fountain_untapping_is_part_of_the_cost() {
    cr!("602.2b", "601.2h");
    ruling!(
        "Halo Fountain",
        "Untapping creatures is part of the cost of all of Halo Fountain's abilities."
    );
    let mut t = TestGame::new(2);
    let fountain = t.battlefield(P0, "Halo Fountain");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let ogre = t.battlefield(P0, "Gray Ogre");
    t.lands(P0, "Plains", 2);
    t.g.objects[bears.0 as usize].tapped = true;
    t.g.objects[ogre.0 as usize].tapped = true;
    let hand = t.hand_size(P0);
    activate_containing(&mut t, P0, fountain, "Untap two").expect("activate");
    // Already untapped while the ability is on the stack, before anyone could respond.
    assert_eq!(t.g.stack.len(), 1);
    assert!(!t.obj_now(bears).tapped && !t.obj_now(ogre).tapped);
    // Removing a creature now doesn't matter: the ability resolves.
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}

/// P0 activates Liar's Pendulum targeting P1, naming `name`; P1 guesses `in_hand`
/// ("yes"/"no"); P0 reveals if `reveal`. Returns the game, P0's hand growth, and the
/// order of the decisions (who and what) after targets.
fn pendulum(in_hand: bool, guess_yes: bool, reveal: bool) -> (usize, Vec<String>) {
    supported("Liar's Pendulum");
    let mut t = TestGame::new(2);
    let lp = t.battlefield(P0, "Liar's Pendulum");
    t.lands(P0, "Wastes", 2);
    if in_hand {
        t.hand(P0, "Grizzly Bears");
    }
    let from = t.asked().len();
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer(
        P0,
        DecisionKind::Name,
        mtg_engine::decision::Answer::Text("Grizzly Bears".into()),
    );
    t.answer(
        P1,
        DecisionKind::Option,
        mtg_engine::decision::Answer::Index(if guess_yes { 0 } else { 1 }),
    );
    t.answer_yes(P0, reveal);
    activate_containing(&mut t, P0, lp, "Choose a card name").unwrap();
    let hand = t.hand_size(P0);
    t.resolve_all();
    let order = t.asked()[from..]
        .iter()
        .filter(|(_, d)| !matches!(d, mtg_engine::decision::Decision::ChooseTargets { .. }))
        .map(|(p, d)| format!("{p:?} {}", format!("{d:?}").split(' ').next().unwrap()))
        .collect();
    (t.hand_size(P0) - hand, order)
}

#[test]
fn liars_pendulum_name_then_guess_and_reveal_is_optional() {
    cr!("608.2c");
    ruling!(
        "Liar's Pendulum",
        "First you name a card. Then the targeted opponent guesses whether the card is in your hand."
    );
    ruling!(
        "Liar's Pendulum",
        "You reveal your hand only if you choose to, regardless of whether your opponent guessed right or wrong."
    );
    let (drawn, order) = pendulum(true, false, true);
    assert_eq!(
        order,
        vec!["PlayerId(0) NameCard", "PlayerId(1) ChooseOption", "PlayerId(0) YesNo"]
    );
    // Guessed wrong, revealed: draw. Guessed wrong, not revealed: no draw. Guessed right
    // and revealed: no draw.
    assert_eq!(drawn, 1);
    assert_eq!(pendulum(true, false, false).0, 0);
    assert_eq!(pendulum(true, true, true).0, 0);
    assert_eq!(pendulum(false, true, true).0, 1);
    assert_eq!(pendulum(false, false, true).0, 0);
}

#[test]
fn scrying_glass_offers_only_the_five_colors() {
    cr!("105.1");
    ruling!(
        "Scrying Glass",
        "The only valid colors are white, black, blue, red, and green."
    );
    supported("Scrying Glass");
    let mut t = TestGame::new(2);
    let sg = t.battlefield(P0, "Scrying Glass");
    t.lands(P0, "Wastes", 3);
    t.hand(P1, "Lightning Bolt");
    t.hand(P1, "Shock");
    let from = t.asked().len();
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer(P0, DecisionKind::Number, mtg_engine::decision::Answer::Number(2));
    // "red" is the fourth option.
    t.answer(P0, DecisionKind::Option, mtg_engine::decision::Answer::Index(3));
    activate_containing(&mut t, P0, sg, "Choose a number").unwrap();
    let hand = t.hand_size(P0);
    t.resolve_all();
    let options = options_asked_since(&t, P0, from);
    assert_eq!(
        options,
        vec![vec!["white", "blue", "black", "red", "green"]]
    );
    // Exactly two red cards: P0 draws.
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn relic_vial_sacrificing_the_only_cleric_still_triggers() {
    cr!("603.10a", "603.6c");
    ruling!(
        "Relic Vial",
        "If you sacrifice the only Cleric you control to activate Relic Vial’s first ability, the triggered ability it gives itself triggers."
    );
    supported("Relic Vial");
    let mut t = TestGame::new(2);
    let vial = t.battlefield(P0, "Relic Vial");
    let cleric = t.battlefield(P0, "Shrine Keeper");
    t.lands(P0, "Wastes", 2);
    t.answer_choose(P0, &[obj(cleric)]);
    activate_containing(&mut t, P0, vial, "Draw a card").unwrap();
    t.settle();
    assert!(t.in_graveyard(P0, "Shrine Keeper"));
    assert_eq!(triggers_from(&t, vial), 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    assert_eq!(t.life(P0), 21);
}

#[test]
fn relic_vial_leaving_with_the_clerics_triggers_for_each_creature() {
    cr!("603.10a");
    ruling!(
        "Relic Vial",
        "If Relic Vial leaves the battlefield at the same time as each Cleric you control, the triggered ability triggers for any creatures you control that die at the same time"
    );
    supported("Akroma's Vengeance");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Relic Vial");
    t.battlefield(P0, "Shrine Keeper");
    t.battlefield(P0, "Grizzly Bears");
    cast_resolve(&mut t, P0, "Akroma's Vengeance", &[]);
    assert!(t.in_graveyard(P0, "Relic Vial"));
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.life(P0), 22);
}

#[test]
fn relic_vial_trigger_resolves_without_a_cleric() {
    cr!("603.4", "113.7a");
    ruling!(
        "Relic Vial",
        "Once Relic Vial’s ability has triggered, that ability resolves even if you no longer control a Cleric."
    );
    let mut t = TestGame::new(2);
    let vial = t.battlefield(P0, "Relic Vial");
    let cleric = t.battlefield(P0, "Shrine Keeper");
    let bears = t.battlefield(P0, "Grizzly Bears");
    destroy(&mut t, bears);
    assert_eq!(triggers_from(&t, vial), 1);
    // The Cleric is exiled in response (no death, so no second trigger).
    crate::r_s05_common::move_to(&mut t, cleric, Zone::Exile);
    assert_eq!(triggers_from(&t, vial), 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    assert_eq!(t.life(P0), 21);
}

#[test]
fn relic_vial_in_two_headed_giant() {
    cr!("810.9");
    ruling!(
        "Relic Vial",
        "In a Two-Headed Giant game, Relic Vial’s ability causes the opposing team to lose 2 life and you to gain 1 life."
    );
    let mut t = two_headed_giant();
    t.battlefield(P0, "Relic Vial");
    t.battlefield(P0, "Shrine Keeper");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let (us, them) = (t.life(P0), t.life(P2));
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.life(P2), them - 2);
    assert_eq!(t.life(P3), them - 2);
    assert_eq!(t.life(P0), us + 1);
}

#[test]
fn izzet_generatoria_each_add_one() {
    cr!("616.1", "614.1a");
    ruling!(
        "Izzet Generatorium",
        "If you control two Izzet Generatoriums (Generatoria?), the number of {E} you get is the original number plus two."
    );
    supported("Izzet Generatorium");
    supported("Aether Hub");
    for (n, expected) in [(0, 1), (1, 2), (2, 3), (3, 4)] {
        let mut t = TestGame::new(2);
        t.lands(P0, "Izzet Generatorium", n);
        t.enter(P0, "Aether Hub");
        t.g.flush_events();
        t.resolve_all();
        assert_eq!(crate::r_s28_common::energy(&t, P0), expected, "{n}");
    }
}

#[test]
fn angelheart_vial_lethal_damage_loses_before_counters() {
    cr!("704.5a", "117.5");
    ruling!(
        "Angelheart Vial",
        "If you’re dealt enough damage to reduce your life total to 0 or less, you’ll lose the game before you can put charge counters on Angelheart Vial."
    );
    supported("Angelheart Vial");
    for life in [3, 4] {
        let mut t = TestGame::new(2);
        let vial = t.battlefield(P0, "Angelheart Vial");
        set_life(&mut t, P0, life);
        cast_targeting(&mut t, P1, "Lightning Bolt", &[Entity::Player(P0)]);
        t.answer_yes(P0, true);
        t.resolve_all();
        if life == 3 {
            assert!(t.has_lost(P0));
            assert_eq!(t.counters(vial, "charge"), 0);
        } else {
            assert!(!t.has_lost(P0));
            assert_eq!(t.counters(vial, "charge"), 3);
        }
    }
}

/// A game where P0's library is only the given cards (the first on top) and P0 controls
/// Temporal Aperture with mana to activate it.
fn aperture(library: &[&str]) -> (TestGame, ObjectId) {
    supported("Temporal Aperture");
    let mut t = TestGame::new(2);
    let lib = t.g.players[0].library.clone();
    for c in lib {
        t.g.objects[c.0 as usize].zone = Zone::Nowhere;
    }
    t.g.players[0].library.clear();
    crate::r_s01_common::stack_library(&mut t, P0, library);
    let ap = t.battlefield(P0, "Temporal Aperture");
    t.lands(P0, "Wastes", 5);
    activate_containing(&mut t, P0, ap, "Shuffle your library").unwrap();
    t.resolve_all();
    (t, ap)
}

/// The first way a permission allows `p` to cast `card`, if any.
fn free_method(t: &mut TestGame, card: ObjectId) -> Option<CastMethod> {
    t.g.recompute();
    t.g.turn.priority = Some(P0);
    t.g.cast_options(P0, card)
        .into_iter()
        .find(|o| t.g.can_begin_cast(P0, card, o))
        .map(|o| o.method)
}

#[test]
fn temporal_aperture_card_leaves_the_library_when_played() {
    cr!("601.2a");
    ruling!(
        "Temporal Aperture",
        "The card is removed from the top of the library when you play it, just like a card is removed from your hand when you play it."
    );
    let (mut t, _) = aperture(&["Grizzly Bears", "Grizzly Bears"]);
    let top = *t.g.player(P0).library.last().unwrap();
    let m = free_method(&mut t, top).expect("castable from the top");
    t.cast(P0, top).method(m).go();
    assert_eq!(t.library_size(P0), 1);
    assert_eq!(t.zone(top), Zone::Stack);
    // The new top card isn't the one revealed: it can't be cast for free.
    let next = *t.g.player(P0).library.last().unwrap();
    assert!(free_method(&mut t, next).is_none());
    t.resolve_all();
    assert!(t.on_battlefield(top));
}

#[test]
fn temporal_aperture_still_requires_additional_costs() {
    cr!("118.9d", "601.2b");
    ruling!(
        "Temporal Aperture",
        "While you do not have to pay the mana cost, you do have to pay any other costs described in the text that are paid when playing the card. If the cost in the card text is optional, such as with Buyback, you can optionally pay that cost."
    );
    // Tormenting Voice on top: it can be cast for free only by discarding a card.
    let (mut t, _) = aperture(&["Tormenting Voice"; 3]);
    let top = *t.g.player(P0).library.last().unwrap();
    assert_eq!(t.obj_now(top).chars.name.as_str(), "Tormenting Voice");
    assert_eq!(t.hand_size(P0), 0);
    assert!(free_method(&mut t, top).is_none());
    let opt = t.hand(P0, "Opt");
    let m = free_method(&mut t, top).expect("castable by discarding Opt");
    t.answer_choose(P0, &[obj(opt)]);
    t.cast(P0, top).method(m).go();
    assert!(t.in_graveyard(P0, "Opt"));
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 2);
    // An optional cost such as buyback may be paid: Whispers of the Muse returns to hand.
    let (mut t, _) = aperture(&["Whispers of the Muse"; 3]);
    let top = *t.g.player(P0).library.last().unwrap();
    t.lands(P0, "Wastes", 5);
    let m = free_method(&mut t, top).expect("castable");
    t.answer(P0, DecisionKind::OptionalCost, mtg_engine::decision::Answer::Bool(true));
    t.cast(P0, top).method(m).go();
    t.resolve_all();
    assert!(t.in_hand(P0, "Whispers of the Muse"));
    assert_eq!(t.hand_size(P0), 2);
    assert_eq!(crate::r_s04_common::untapped_lands(&t, P0), 0);
}

#[test]
fn tamiyos_logbook_reduction_never_reduces_the_blue() {
    cr!("601.2f", "118.7");
    ruling!(
        "Tamiyo's Logbook",
        "Even if you have five or more artifacts, it still costs {U} to activate."
    );
    supported("Tamiyo's Logbook");
    for (artifacts, islands, wastes, ok) in [
        (6, 1, 0, true),
        (6, 0, 1, false),
        (6, 0, 0, false),
        (2, 1, 3, true),
        (2, 1, 2, false),
    ] {
        let mut t = TestGame::new(2);
        let book = t.battlefield(P0, "Tamiyo's Logbook");
        t.lands(P0, "Ornithopter", artifacts);
        t.lands(P0, "Island", islands);
        t.lands(P0, "Wastes", wastes);
        assert_eq!(
            can_activate(&mut t, P0, book),
            ok,
            "{artifacts} {islands} {wastes}"
        );
    }
}

#[test]
fn illuminated_folio_resolves_even_if_a_revealed_card_leaves_the_hand() {
    cr!("701.20a");
    ruling!(
        "Illuminated Folio",
        "If one of them is played, is discarded, or otherwise leaves your hand, the ability will still resolve as normal."
    );
    supported("Illuminated Folio");
    let mut t = TestGame::new(2);
    let folio = t.battlefield(P0, "Illuminated Folio");
    t.lands(P0, "Wastes", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    let shock = t.hand(P0, "Shock");
    t.answer_choose(P0, &[obj(bolt), obj(shock)]);
    activate_containing(&mut t, P0, folio, "Draw a card").unwrap();
    // P0 casts the revealed Lightning Bolt in response.
    t.lands(P0, "Mountain", 1);
    t.cast_with(P0, bolt, &[Entity::Player(P1)]).unwrap();
    t.resolve();
    assert_eq!(t.life(P1), 17);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn loreseekers_stone_counts_the_hand_on_activation() {
    cr!("601.2f", "602.2b");
    ruling!(
        "Loreseeker's Stone",
        "Use the number of cards in your hand at the time you activate the ability (before drawing three cards)"
    );
    supported("Loreseeker's Stone");
    for (hand, lands, ok) in [(2, 5, true), (2, 4, false), (0, 3, true)] {
        let mut t = TestGame::new(2);
        let stone = t.battlefield(P0, "Loreseeker's Stone");
        fill_hand(&mut t, P0, hand);
        t.lands(P0, "Wastes", lands);
        assert_eq!(can_activate(&mut t, P0, stone), ok, "{hand} {lands}");
        if ok {
            activate_containing(&mut t, P0, stone, "Draw three").unwrap();
            t.resolve_all();
            assert_eq!(t.hand_size(P0), hand + 3);
        }
    }
}

#[test]
fn book_of_rass_cant_pay_life_you_dont_have() {
    cr!("119.4", "118.3");
    ruling!(
        "Book of Rass",
        "You can't spend yourself to below zero life. You can't spend life you don't have."
    );
    supported("Book of Rass");
    for (life, ok) in [(1, false), (2, true)] {
        let mut t = TestGame::new(2);
        let book = t.battlefield(P0, "Book of Rass");
        t.lands(P0, "Wastes", 2);
        set_life(&mut t, P0, life);
        assert_eq!(can_activate(&mut t, P0, book), ok, "{life}");
        assert_eq!(
            activate_containing(&mut t, P0, book, "Draw a card").is_ok(),
            ok
        );
        assert_eq!(t.life(P0), if ok { 0 } else { 1 });
    }
}

/// Activates Druidic Satchel with `top` on top of P0's library.
fn satchel(top: &str) -> (TestGame, ObjectId) {
    supported("Druidic Satchel");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Druidic Satchel");
    t.lands(P0, "Wastes", 2);
    let card = t.library_top(P0, top);
    activate_containing(&mut t, P0, s, "Reveal the top card").unwrap();
    t.resolve_all();
    (t, card)
}

#[test]
fn druidic_satchel_outcomes() {
    cr!("701.20a");
    ruling!(
        "Druidic Satchel",
        "If the card is not a land card, it will remain on top of your library."
    );
    ruling!(
        "Druidic Satchel",
        "If the revealed card is both a creature and a land (such as Dryad Arbor), you’ll put a Saproling creature token and that card onto the battlefield."
    );
    let (t, card) = satchel("Grizzly Bears");
    assert_eq!(*t.g.player(P0).library.last().unwrap(), card);
    assert_eq!(tokens(&t, P0).len(), 1);
    let (t, card) = satchel("Lightning Bolt");
    assert_eq!(*t.g.player(P0).library.last().unwrap(), card);
    assert_eq!(t.life(P0), 22);
    let (t, card) = satchel("Dryad Arbor");
    assert!(t.on_battlefield(card));
    assert_eq!(tokens(&t, P0).len(), 1);
    let (t, card) = satchel("Forest");
    assert!(t.on_battlefield(card));
    assert_eq!(tokens(&t, P0).len(), 0);
}

#[test]
fn discard_a_card_additional_cost_is_exactly_one_card() {
    cr!("601.2b", "118.3");
    ruling!(
        "Wild Guess",
        "Because discarding a card is an additional cost, you can’t cast Wild Guess if you have no other cards in hand."
    );
    ruling!(
        "Thrill of Possibility",
        "You must discard exactly one card to cast Thrill of Possibility; you can't cast it without discarding a card, and you can't discard additional cards."
    );
    ruling!(
        "Tormenting Voice",
        "You must discard exactly one card to cast Tormenting Voice; you can't cast it without discarding a card, and you can't discard additional cards."
    );
    for name in ["Wild Guess", "Thrill of Possibility", "Tormenting Voice"] {
        supported(name);
        let mut t = TestGame::new(2);
        give_mana_for(&mut t, P0, name);
        let spell = t.hand(P0, name);
        assert!(!can_cast(&mut t, P0, spell, CastMethod::Normal), "{name}");
        let a = t.hand(P0, "Opt");
        let b = t.hand(P0, "Shock");
        assert!(can_cast(&mut t, P0, spell, CastMethod::Normal), "{name}");
        let from = t.asked().len();
        // Asking to discard both: only one may be discarded.
        t.answer_choose(P0, &[obj(a), obj(b)]);
        let _ = t.cast(P0, spell).try_go();
        let asked: Vec<(u32, u32)> = t.asked()[from..]
            .iter()
            .filter_map(|(_, d)| match d {
                mtg_engine::decision::Decision::ChooseEntities { min, max, .. } => {
                    Some((*min, *max))
                }
                _ => None,
            })
            .collect();
        assert_eq!(asked, vec![(1, 1)], "{name}");
        t.resolve_all();
        assert_eq!(t.graveyard_size(P0), 2, "{name}: the spell and one card");
        assert_eq!(t.hand_size(P0), 3, "{name}");
    }
}
