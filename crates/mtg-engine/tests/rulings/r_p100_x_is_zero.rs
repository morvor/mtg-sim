//! Rulings batch P100 — "mana value matters": the mana value of a card or permanent with
//! {X} in its mana cost anywhere but the stack counts X as 0 (CR 202.3e, 107.3g), a
//! token or a card with no mana cost has mana value 0 (CR 202.3a), and a copy has the
//! mana value of what it copies (CR 707.2). Each card's ruling restates this for the zone
//! its ability looks at: a compact case per card.

use crate::r_p100_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn x_is_0_for_a_card_with_x_in_its_cost_in_a_hand() {
    cr!("202.3e", "107.3g");
    ruling!(
        "Mind into Matter",
        "If a card in your hand has {X} in its mana cost, X is 0 for the purpose of determining its mana value."
    );
    ruling!(
        "Anticausal Vestige",
        "If a card in your hand has {X} in its mana cost, X is 0 for the purpose of determining its mana value."
    );
    ruling!(
        "Broodcaller Scourge",
        "If a card in your hand has {X} in its mana cost, X is 0 for the purpose of determining its mana value."
    );
    ruling!(
        "Kinscaer Sentry",
        "If a card in your hand has {X} in its mana cost, X is 0 for the purpose of determining its mana value."
    );
    ruling!(
        "Atemsis, All-Seeing",
        "If a card in your hand has {X} in its mana cost, the value of X is considered to be 0."
    );
    for c in [
        "Mind into Matter",
        "Anticausal Vestige",
        "Broodcaller Scourge",
        "Kinscaer Sentry",
        "Atemsis, All-Seeing",
    ] {
        supported(c);
    }
    // Mind into Matter with X = 0: a Chalice of the Void card ({X}{X}, 0) may be put onto
    // the battlefield; a Grizzly Bears card (2) may not.
    let mut t = TestGame::new(2);
    let chalice = t.hand(P0, "Chalice of the Void");
    let bears = t.hand(P0, "Grizzly Bears");
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(chalice)]);
    cast_x_resolve(&mut t, P0, "Mind into Matter", 0, &[]);
    assert!(was_offered(&t, P0, from, chalice), "{:?}", asked_of(&t, P0, from));
    assert!(!was_offered(&t, P0, from, bears));
    assert!(t.on_battlefield(chalice) && t.obj_now(chalice).tapped, "{:?}", asked_of(&t, P0, from));
    // Anticausal Vestige leaves with no lands: mana value 0 or less, so the Chalice card.
    let mut t = TestGame::new(2);
    let vestige = t.battlefield(P0, "Anticausal Vestige");
    let chalice = t.hand(P0, "Chalice of the Void");
    let bears = t.hand(P0, "Grizzly Bears");
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(chalice)]);
    destroy(&mut t, vestige);
    t.resolve_all();
    assert!(was_offered(&t, P0, from, chalice), "{:?}", asked_of(&t, P0, from));
    assert!(!was_offered(&t, P0, from, bears));
    assert!(t.on_battlefield(chalice));
    // Broodcaller Scourge deals 5 combat damage: the Chalice card (0) and a Hill Giant
    // card (4) may be put onto the battlefield, a Craw Wurm card (6) may not.
    let mut t = TestGame::new(2);
    let scourge = t.battlefield(P0, "Broodcaller Scourge");
    let chalice = t.hand(P0, "Chalice of the Void");
    let giant = t.hand(P0, "Hill Giant");
    let wurm = t.hand(P0, "Craw Wurm");
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(chalice)]);
    t.attack(&[(scourge, pl(P1))], &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 15);
    assert!(was_offered(&t, P0, from, chalice), "{:?}", asked_of(&t, P0, from));
    assert!(was_offered(&t, P0, from, giant));
    assert!(!was_offered(&t, P0, from, wurm));
    assert!(t.on_battlefield(chalice));
    // Kinscaer Sentry attacks alone: X is 1; a Jadelight Spelunker card ({X}{G}, 1) may
    // be put onto the battlefield attacking, a Grizzly Bears card (2) may not.
    let mut t = TestGame::new(2);
    let sentry = t.battlefield(P0, "Kinscaer Sentry");
    let spelunker = t.hand(P0, "Jadelight Spelunker");
    let bears = t.hand(P0, "Grizzly Bears");
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.answer_choose_moved(P0, &[obj(spelunker)]);
    attack_with(&mut t, &[(sentry, pl(P1))]);
    t.resolve_all();
    assert!(was_offered(&t, P0, from, spelunker), "{:?}", asked_of(&t, P0, from));
    assert!(!was_offered(&t, P0, from, bears));
    assert!(t.on_battlefield(spelunker) && t.obj_now(spelunker).tapped);
    // Atemsis: a hand of Endless One (0), Fireball ({X}{R}, 1), Grizzly Bears (2), Hill
    // Giant (4), Air Elemental (5) and Craw Wurm (6) has six different mana values: the
    // opponent loses. With a Walking Ballista (0) in place of the Fireball, only five.
    for (sixth, loses) in [("Fireball", true), ("Walking Ballista", false)] {
        let mut t = TestGame::new(2);
        let atemsis = t.battlefield(P0, "Atemsis, All-Seeing");
        for c in ["Endless One", sixth, "Grizzly Bears", "Hill Giant", "Air Elemental", "Craw Wurm"] {
            t.hand(P0, c);
        }
        t.answer_yes(P0, true);
        crate::r_p151_common::damage(&mut t, atemsis, pl(P1), 4);
        t.resolve_all();
        assert_eq!(t.has_lost(P1), loses, "{sixth}");
    }
}

#[test]
fn x_is_0_for_a_card_with_x_in_its_cost_in_a_graveyard() {
    cr!("202.3e", "107.3g");
    ruling!(
        "Agadeem's Awakening // Agadeem, the Undercrypt",
        "If a card in a player's graveyard has {X} in its mana cost, X is considered to be 0 for that card."
    );
    ruling!(
        "Agadeem's Awakening // Agadeem, the Undercrypt",
        "If X is 0 for Agadeem's Awakening, you can target one creature card with mana value 0."
    );
    ruling!(
        "Sudden Insight",
        "If a card in your graveyard has {X} in its mana cost, X is 0 when determining that card's mana value."
    );
    ruling!(
        "Abiding Grace",
        "If a creature card in your graveyard has {X} in its mana cost, X is 0 when determining its mana value."
    );
    ruling!(
        "Jackdaw Savior",
        "If a creature on the battlefield or a creature card in a graveyard has {X} in its mana cost, X is 0 for the purpose of determining its mana value."
    );
    ruling!(
        "Sidisi, Regent of the Mire",
        "If a creature on the battlefield or a creature card in a graveyard has {X} in its mana cost, X is 0 for the purpose of determining its mana value."
    );
    ruling!(
        "Sphere of Annihilation",
        "If a permanent on the battlefield or a card in a graveyard has {X} in its mana cost, X is 0 when determining that card's mana value."
    );
    ruling!(
        "Desecrate Reality",
        "If a card has {X} in its mana cost, X is zero when it's in any zone but the stack."
    );
    for c in [
        "Agadeem's Awakening // Agadeem, the Undercrypt",
        "Sudden Insight",
        "Abiding Grace",
        "Jackdaw Savior",
        "Sidisi, Regent of the Mire",
        "Sphere of Annihilation",
        "Desecrate Reality",
    ] {
        supported(c);
    }
    // Agadeem's Awakening with X = 0: an Endless One card and a Walking Ballista card (both
    // 0) are legal targets, a Grizzly Bears card (2) isn't; one of them is returned (they
    // don't have different mana values, so not both).
    let mut t = TestGame::new(2);
    let endless = t.graveyard(P0, "Endless One");
    let ballista = t.graveyard(P0, "Walking Ballista");
    let bears = t.graveyard(P0, "Grizzly Bears");
    let from = t.asked().len();
    cast_x_resolve(&mut t, P0, "Agadeem's Awakening // Agadeem, the Undercrypt", 0, &[obj(endless)]);
    let names = target_candidate_names(&t, P0, from);
    assert_eq!(names, ["Endless One", "Walking Ballista"], "{:?}", asked_of(&t, P0, from));
    // (It returned as a 0/0 and died: a new object in the graveyard.)
    assert_ne!(t.g.current(endless), endless, "returned to the battlefield");
    assert!(t.in_graveyard(P0, "Endless One"));
    assert_eq!(t.zone(ballista), Zone::Graveyard(P0));
    assert_eq!(t.zone(bears), Zone::Graveyard(P0));
    // Sudden Insight: Fireball (1), Walking Ballista (0), Endless One (0), Grizzly Bears
    // (2) and a Forest in the graveyard: three different mana values, three cards.
    let mut t = TestGame::new(2);
    for c in ["Fireball", "Walking Ballista", "Endless One", "Grizzly Bears", "Forest"] {
        t.graveyard(P0, c);
    }
    let hand = t.hand_size(P0);
    cast_resolve(&mut t, P0, "Sudden Insight", "4UU", &[]);
    assert_eq!(t.hand_size(P0), hand + 3);
    // Abiding Grace returns a Jadelight Spelunker card ({X}{G}, 1); a Walking Ballista
    // card (0) isn't a legal target.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Abiding Grace");
    let prodigy = t.graveyard(P0, "Jadelight Spelunker");
    let ballista = t.graveyard(P0, "Walking Ballista");
    let from = t.asked().len();
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1]));
    t.answer_targets(P0, &[obj(prodigy)]);
    end_step(&mut t, P0);
    t.resolve_all();
    assert_eq!(target_candidate_names(&t, P0, from), ["Jadelight Spelunker"]);
    assert!(t.on_battlefield(prodigy));
    assert_eq!(t.zone(ballista), Zone::Graveyard(P0));
    // Jackdaw Savior (3) dies: a Jadelight Spelunker card with X in its cost (1) has
    // lesser mana value, a Hill Giant card (4) doesn't.
    let mut t = TestGame::new(2);
    let jackdaw = t.battlefield(P0, "Jackdaw Savior");
    let spelunker = t.graveyard(P0, "Jadelight Spelunker");
    let giant = t.graveyard(P0, "Hill Giant");
    let from = t.asked().len();
    t.answer_targets(P0, &[obj(spelunker)]);
    destroy(&mut t, jackdaw);
    t.resolve_all();
    assert_eq!(target_candidate_names(&t, P0, from), ["Jadelight Spelunker"]);
    assert!(t.on_battlefield(spelunker));
    assert_eq!(t.zone(giant), Zone::Graveyard(P0));
    // Sidisi sacrifices an Endless One with three counters (0 on the battlefield): X is 0,
    // so the target is a creature card with mana value 1 (Jadelight Spelunker), not 4.
    let mut t = TestGame::new(2);
    let sidisi = t.battlefield(P0, "Sidisi, Regent of the Mire");
    let endless = endless_one(&mut t, P0, 3);
    let prodigy = t.graveyard(P0, "Jadelight Spelunker");
    let giant = t.graveyard(P0, "Hill Giant");
    let from = t.asked().len();
    t.answer_choose(P0, &[obj(endless)]);
    act(&mut t, P0, sidisi, "Sacrifice a creature", &[obj(prodigy)]).expect("Sidisi");
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Endless One"));
    assert_eq!(target_candidate_names(&t, P0, from), ["Jadelight Spelunker"]);
    assert!(t.on_battlefield(prodigy));
    assert_eq!(t.zone(giant), Zone::Graveyard(P0));
    // Sphere of Annihilation with no void counters (X was 0): at the upkeep, an Endless One
    // with four counters (0) and a Walking Ballista card in the opponent's graveyard (0)
    // are exiled; Grizzly Bears (2) and a Grizzly Bears card stay.
    let mut t = TestGame::new(2);
    let sphere = t.battlefield(P0, "Sphere of Annihilation");
    let endless = endless_one(&mut t, P1, 4);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let ballista = t.graveyard(P1, "Walking Ballista");
    let bears_card = t.graveyard(P1, "Grizzly Bears");
    next_upkeep(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.zone(sphere), Zone::Exile);
    assert_eq!(t.zone(endless), Zone::Exile);
    assert_eq!(t.zone(ballista), Zone::Exile);
    assert!(t.on_battlefield(bears));
    assert_eq!(t.zone(bears_card), Zone::Graveyard(P1));
    // Desecrate Reality (paid with seven colorless, so adamant): an opponent's Sphere of
    // Annihilation with a void counter ({X}{B}: 1, odd) can't be targeted while an Endless
    // One with a counter (0, even) can; a Sphere card in the graveyard (odd) may be
    // returned, an Endless One card (even) may not.
    let mut t = TestGame::new(2);
    let sphere = with_counters(&mut t, P1, "Sphere of Annihilation", 0);
    t.g.objects[sphere.0 as usize].counters.insert("void".into(), 1);
    t.g.recompute();
    let endless = endless_one(&mut t, P1, 1);
    let sphere_card = t.graveyard(P0, "Sphere of Annihilation");
    let endless_card = t.graveyard(P0, "Endless One");
    let from = t.asked().len();
    t.answer_targets(P0, &[obj(endless)]);
    t.answer_choose(P0, &[obj(sphere_card)]);
    cast_resolve(&mut t, P0, "Desecrate Reality", "7", &[]);
    assert_eq!(target_candidate_names(&t, P0, from), ["Endless One"], "{:?}", asked_of(&t, P0, from));
    assert_eq!(t.zone(endless), Zone::Exile);
    assert!(t.on_battlefield(sphere));
    assert!(was_offered(&t, P0, from, sphere_card));
    assert!(!was_offered(&t, P0, from, endless_card));
    assert!(t.on_battlefield(sphere_card));
}

#[test]
fn x_is_0_for_a_card_with_x_in_its_cost_in_a_library() {
    cr!("202.3e", "107.3g");
    ruling!("Anchor to Reality", "If a card in a player's library has {X} in its mana cost, X is 0.");
    ruling!(
        "Neoform",
        "If a creature or a creature card in your library has {X} in its mana cost, X is considered to be 0."
    );
    ruling!(
        "Prime Speaker Vannifar",
        "If a creature or a creature card in your library has {X} in its mana cost, X is considered to be 0."
    );
    ruling!(
        "Pyre of Heroes",
        "If a permanent or a card in a library has {X} in its mana cost, X is considered to be 0."
    );
    ruling!(
        "Lukka, Coppercoat Outcast",
        "If a creature card in a player's library or a creature on the battlefield has {X} in its mana cost, X is considered to be 0."
    );
    ruling!(
        "Enigmatic Incarnation",
        "If an enchantment on the battlefield or a creature card in a library has {X} in its mana cost, X is considered to be 0."
    );
    ruling!(
        "Repurposing Bay",
        "If there’s an {X} in the sacrificed artifact’s mana cost, X is 0 when determining its mana value."
    );
    for c in [
        "Anchor to Reality",
        "Neoform",
        "Prime Speaker Vannifar",
        "Pyre of Heroes",
        "Lukka, Coppercoat Outcast",
        "Enigmatic Incarnation",
        "Repurposing Bay",
    ] {
        supported(c);
    }
    // Anchor to Reality sacrificing a Mind Stone (2) finds Sigil of Distinction ({X}, 0 in
    // the library): less than 2, so scry 2.
    let mut t = TestGame::new(2);
    let stone = t.battlefield(P0, "Mind Stone");
    let sigil = t.library_top(P0, "Sigil of Distinction");
    let from = t.asked().len();
    t.answer_choose(P0, &[obj(stone)]);
    t.answer_choose(P0, &[obj(sigil)]);
    cast_resolve(&mut t, P0, "Anchor to Reality", "2UU", &[]);
    assert!(t.on_battlefield(sigil));
    assert!(
        asked_of(&t, P0, from).iter().any(|d| matches!(d, Decision::Scry { .. })),
        "scry 2: {:?}",
        asked_of(&t, P0, from)
    );
    // Neoform sacrificing a Walking Ballista with two counters (0): a creature card with
    // mana value 1 — Ingenious Prodigy ({X}{U}, 0 + 1) — is found, not Grizzly Bears (2).
    let mut t = TestGame::new(2);
    let ballista = with_counters(&mut t, P0, "Walking Ballista", 2);
    let prodigy = t.library_top(P0, "Ingenious Prodigy");
    let bears = t.library_top(P0, "Grizzly Bears");
    let from = t.asked().len();
    t.answer_choose(P0, &[obj(ballista)]);
    t.answer_choose(P0, &[obj(prodigy)]);
    cast_resolve(&mut t, P0, "Neoform", "GU", &[]);
    assert!(was_offered(&t, P0, from, prodigy), "{:?}", asked_of(&t, P0, from));
    assert!(!was_offered(&t, P0, from, bears));
    assert!(t.on_battlefield(prodigy));
    assert_eq!(t.counters(prodigy, counters::PLUS1), 1, "an additional counter");
    // Prime Speaker Vannifar: the same through its ability.
    let mut t = TestGame::new(2);
    let vannifar = t.battlefield(P0, "Prime Speaker Vannifar");
    let ballista = with_counters(&mut t, P0, "Walking Ballista", 2);
    let prodigy = t.library_top(P0, "Ingenious Prodigy");
    let bears = t.library_top(P0, "Grizzly Bears");
    let from = t.asked().len();
    t.answer_choose(P0, &[obj(ballista)]);
    t.answer_choose(P0, &[obj(prodigy)]);
    act(&mut t, P0, vannifar, "Sacrifice another creature", &[]).expect("Vannifar");
    t.resolve_all();
    assert!(was_offered(&t, P0, from, prodigy), "{:?}", asked_of(&t, P0, from));
    assert!(!was_offered(&t, P0, from, bears));
    assert!(t.on_battlefield(prodigy));
    // Pyre of Heroes sacrificing an Endless One (Eldrazi, 0) finds Blisterpod (Eldrazi
    // Drone, 1), not Eldrazi Mimic (2).
    let mut t = TestGame::new(2);
    let pyre = t.battlefield(P0, "Pyre of Heroes");
    let endless = endless_one(&mut t, P0, 3);
    let pod = t.library_top(P0, "Blisterpod");
    let mimic = t.library_top(P0, "Eldrazi Mimic");
    mana(&mut t, P0, "2");
    let from = t.asked().len();
    t.answer_choose(P0, &[obj(endless)]);
    t.answer_choose(P0, &[obj(pod)]);
    act(&mut t, P0, pyre, "Sacrifice a creature", &[]).expect("Pyre");
    t.resolve_all();
    assert!(was_offered(&t, P0, from, pod), "{:?}", asked_of(&t, P0, from));
    assert!(!was_offered(&t, P0, from, mimic));
    assert!(t.on_battlefield(pod));
    // Lukka's −2 exiling an Endless One with five counters (0): a Walking Ballista card on
    // top (0, not greater) is revealed and passed over; Grizzly Bears (2) is put onto the
    // battlefield.
    let mut t = TestGame::new(2);
    let lukka = t.battlefield(P0, "Lukka, Coppercoat Outcast");
    let endless = endless_one(&mut t, P0, 5);
    let lib = set_library(&mut t, P0, &["Walking Ballista", "Grizzly Bears", "Forest"]);
    t.activate(P0, lukka, 1, &[obj(endless)]).expect("Lukka −2");
    t.resolve_all();
    assert_eq!(t.zone(endless), Zone::Exile);
    assert!(t.on_battlefield(lib[1]));
    assert_eq!(t.zone(lib[0]), Zone::Library(P0));
    // Enigmatic Incarnation sacrificing a Nyxborn Hydra ({X}{G}, 1 on the battlefield)
    // finds a creature card with mana value 2 (Grizzly Bears), not 4 (Hill Giant).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Enigmatic Incarnation");
    let hydra = with_counters(&mut t, P0, "Nyxborn Hydra", 3);
    let bears = t.library_top(P0, "Grizzly Bears");
    let giant = t.library_top(P0, "Hill Giant");
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(hydra)]);
    t.answer_choose(P0, &[obj(bears)]);
    end_step(&mut t, P0);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Nyxborn Hydra"));
    assert!(was_offered(&t, P0, from, bears), "{:?}", asked_of(&t, P0, from));
    assert!(!was_offered(&t, P0, from, giant));
    assert!(t.on_battlefield(bears));
    // Repurposing Bay sacrificing a Walking Ballista with two counters (0) finds Sol Ring
    // (1), not Mind Stone (2).
    let mut t = TestGame::new(2);
    let bay = t.battlefield(P0, "Repurposing Bay");
    let ballista = with_counters(&mut t, P0, "Walking Ballista", 2);
    let ring = t.library_top(P0, "Sol Ring");
    let stone = t.library_top(P0, "Mind Stone");
    mana(&mut t, P0, "2");
    let from = t.asked().len();
    t.answer_choose(P0, &[obj(ballista)]);
    t.answer_choose(P0, &[obj(ring)]);
    act(&mut t, P0, bay, "Sacrifice another artifact", &[]).expect("Bay");
    t.resolve_all();
    assert!(was_offered(&t, P0, from, ring), "{:?}", asked_of(&t, P0, from));
    assert!(!was_offered(&t, P0, from, stone));
    assert!(t.on_battlefield(ring));
}

#[test]
fn keruga_counts_x_as_0_in_the_starting_deck_and_on_the_battlefield() {
    cr!("202.3e", "107.3g", "702.139a");
    ruling!(
        "Keruga, the Macrosage",
        "If a card in a player's deck or a permanent on the battlefield has {X} in its mana cost, X is considered to be 0."
    );
    supported("Keruga, the Macrosage");
    // A deck of Forests and Hill Giants (4) meets Keruga's condition; one with an Endless
    // One ({X}, 0) doesn't.
    let mut deck: Vec<&str> = vec!["Forest"; 24];
    deck.extend(vec!["Hill Giant"; 36]);
    assert!(companion_revealable(&deck, "Keruga, the Macrosage"));
    deck[59] = "Endless One";
    assert!(!companion_revealable(&deck, "Keruga, the Macrosage"));
    // On the battlefield, an Endless One with five counters (0) isn't a permanent with mana
    // value 3 or greater; a Hill Giant is: Keruga draws one card.
    let mut t = TestGame::new(2);
    endless_one(&mut t, P0, 5);
    t.battlefield(P0, "Hill Giant");
    let hand = t.hand_size(P0);
    t.enter(P0, "Keruga, the Macrosage");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn x_is_0_for_a_permanent_with_x_in_its_cost_on_the_battlefield() {
    cr!("202.3e", "107.3g");
    ruling!(
        "Rebbec, Architect of Ascension",
        "For spells with {X} in their mana costs, use the value chosen for X to determine the spell's mana value. If an artifact you control has {X} in its mana cost, X is considered to be 0."
    );
    ruling!(
        "Favor of the Mighty",
        "If a creature has X in its mana cost, that X is treated as 0 for the purposes of this effect. It doesn't matter what the value of X was while the creature was on the stack."
    );
    ruling!(
        "Hidetsugu Consumes All // Vessel of the All-Consuming",
        "If a permanent on the battlefield has {X} in its mana value, X is 0."
    );
    ruling!(
        "Padeem, Consul of Innovation",
        "If an artifact has {X} in its mana cost, X is considered to be 0."
    );
    ruling!(
        "Simulacrum Synthesizer",
        "If the mana cost of an artifact on the battlefield includes {X}, X is 0 for the purpose of determining its mana value."
    );
    for c in [
        "Rebbec, Architect of Ascension",
        "Favor of the Mighty",
        "Hidetsugu Consumes All // Vessel of the All-Consuming",
        "Padeem, Consul of Innovation",
        "Simulacrum Synthesizer",
    ] {
        supported(c);
    }
    // Rebbec with a Walking Ballista cast with X = 2: artifacts P0 controls have protection
    // from mana value 0 (an opponent's Memnite), not from 2 (Grizzly Bears). The Ballista
    // spell itself had mana value 4.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rebbec, Architect of Ascension");
    let memnite = t.battlefield(P1, "Memnite");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let spell = cast_x(&mut t, P0, "Walking Ballista", 2, &[]);
    assert_eq!(mana_value(&t, spell), 4);
    t.resolve_all();
    assert_eq!(t.counters(spell, counters::PLUS1), 2);
    assert!(protected_from(&t, spell, memnite));
    assert!(!protected_from(&t, spell, bears));
    // Favor of the Mighty: an Endless One cast with X = 5 has mana value 0 on the
    // battlefield; Grizzly Bears (2) has the greatest mana value and protection from each
    // color.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Favor of the Mighty");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let goblin = t.battlefield(P1, "Raging Goblin");
    let endless = cast_x(&mut t, P0, "Endless One", 5, &[]);
    assert_eq!(mana_value(&t, endless), 5);
    t.resolve_all();
    assert_eq!(t.pt(endless), (5, 5));
    assert!(protected_from(&t, bears, goblin));
    assert!(!protected_from(&t, endless, goblin));
    // Hidetsugu Consumes All, chapter I: an Endless One with three counters (0) and a
    // Memnite are destroyed, Grizzly Bears (2) isn't.
    let mut t = TestGame::new(2);
    let endless = endless_one(&mut t, P1, 3);
    let memnite = t.battlefield(P1, "Memnite");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.enter(P0, "Hidetsugu Consumes All // Vessel of the All-Consuming");
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Endless One"), "{}", t.dump_log());
    assert_eq!(t.zone(memnite), Zone::Graveyard(P1));
    assert!(t.on_battlefield(bears));
    assert_eq!(t.zone(endless), Zone::Graveyard(P1));
    // Padeem with a Walking Ballista with four counters (0) against an opponent's Mind
    // Stone (2): P0 doesn't control the artifact with the greatest mana value, no card.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Padeem, Consul of Innovation");
    ballista(&mut t, P0, 4);
    t.battlefield(P1, "Mind Stone");
    let hand = t.hand_size(P0);
    next_upkeep(&mut t, P0);
    assert_eq!(triggers_on_stack(&t, "draw a card"), 0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
    // Simulacrum Synthesizer: a Walking Ballista cast with X = 4 enters with mana value 0,
    // so no Construct; a Juggernaut (4) makes one.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Simulacrum Synthesizer");
    cast_x_resolve(&mut t, P0, "Walking Ballista", 4, &[]);
    assert!(tokens(&t, P0).is_empty());
    t.enter(P0, "Juggernaut");
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 1, "{:?}", battlefield_names(&t));
}

#[test]
fn kozilek_discards_with_x_as_0_and_counters_a_spell_with_its_chosen_x() {
    cr!("202.3e", "107.3g", "107.3a");
    ruling!(
        "Kozilek, the Great Distortion",
        "If there's an {X} in the mana cost of the card you discard, that X is 0. Any {X} in the mana cost of the target spell will have whatever value was determined for it as the spell was cast."
    );
    supported("Kozilek, the Great Distortion");
    // P1 casts Endless One with X = 2 (mana value 2): P0 announces X = 2 (CR 107.3a) and
    // may discard Grizzly Bears (2) to counter it, not a Walking Ballista card (0).
    let mut t = TestGame::new(2);
    let kozilek = t.battlefield(P0, "Kozilek, the Great Distortion");
    let bears = t.hand(P0, "Grizzly Bears");
    let ballista = t.hand(P0, "Walking Ballista");
    t.set_step(P1, Step::PrecombatMain);
    let spell = cast_x(&mut t, P1, "Endless One", 2, &[]);
    let from = t.asked().len();
    answer_x(&mut t, P0, 2);
    t.answer_choose(P0, &[obj(bears)]);
    act(&mut t, P0, kozilek, "Discard a card", &[obj(spell)]).expect("Kozilek");
    assert!(was_offered(&t, P0, from, bears), "{:?}", asked_of(&t, P0, from));
    assert!(!was_offered(&t, P0, from, ballista));
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Endless One"), "countered");
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    // P1 casts Ornithopter (0): the Walking Ballista card (0) is discarded to counter it.
    let mut t = TestGame::new(2);
    let kozilek = t.battlefield(P0, "Kozilek, the Great Distortion");
    let bears = t.hand(P0, "Grizzly Bears");
    let ballista = t.hand(P0, "Walking Ballista");
    t.set_step(P1, Step::PrecombatMain);
    let thopter = t.hand(P1, "Ornithopter");
    let spell = t.cast(P1, thopter).go();
    let from = t.asked().len();
    answer_x(&mut t, P0, 0);
    t.answer_choose(P0, &[obj(ballista)]);
    act(&mut t, P0, kozilek, "Discard a card", &[obj(spell)]).expect("Kozilek");
    assert!(was_offered(&t, P0, from, ballista), "{:?}", asked_of(&t, P0, from));
    assert!(!was_offered(&t, P0, from, bears));
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Ornithopter"), "countered");
}
