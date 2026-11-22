//! Rulings batch P142 — "reveal cards until" with no card found (the library is revealed
//! and put back in a random order, which isn't a shuffle), "look at the top card; you may
//! reveal it / put it into your graveyard" (the card stays on top otherwise), and the
//! creature that enters attacking from Fireflux Squad.

use crate::r_p142_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::CardType;
use mtg_engine::*;

/// A library of fillers plus `extra` (top first), with Cosi's Trickster on P1's side to
/// detect shuffles. Returns the library contents before the ability.
fn library_with(t: &mut TestGame, p: PlayerId, extra: &[&str]) -> Vec<String> {
    t.battlefield(P1, "Cosi's Trickster");
    stack_library(t, p, extra);
    library_contents(t, p)
}

/// Asserts that nothing was found: the library holds the same cards, nothing went to the
/// hand, and no library was shuffled (Cosi's Trickster has no counter).
fn nothing_found(t: &TestGame, p: PlayerId, before: &[String], hand: usize) {
    assert_eq!(library_contents(t, p), before);
    assert_eq!(t.hand_size(p), hand);
    assert_eq!(shuffles(t), 0, "putting the library back isn't shuffling");
    let trickster = t.named_on_battlefield("Cosi's Trickster")[0];
    assert_eq!(t.counters(trickster, "+1/+1"), 0);
}

#[test]
fn reveal_until_with_no_match_puts_the_library_back_without_shuffling() {
    cr!("701.20a", "701.24a", "608.2c");
    ruling!(
        "Bag of Tricks",
        "If you do not have a creature card in your library with mana value equal to the result, you will reveal your entire library and then put the entire library back in a random order."
    );
    ruling!(
        "Ajani, Valiant Protector",
        "If you don’t have a creature card in your library while resolving Ajani’s second ability, you’ll reveal your library and then randomize it."
    );
    ruling!(
        "Tezzeret, Master of Metal",
        "If you don’t have an artifact card in your library while resolving Tezzeret’s first ability, you’ll reveal your library and then randomize it."
    );
    ruling!(
        "Jalira, Master Polymorphist",
        "If your library has no nonlegendary creature cards in it, you'll reveal all the cards in your library, then put them in a random order."
    );
    ruling!(
        "House Cartographer",
        "If you reveal your entire library and don't reveal a land card, you'll put all revealed cards back in a random order. Since you were never instructed to shuffle your library, abilities that trigger whenever a player shuffles their library won't trigger."
    );
    for c in [
        "Bag of Tricks",
        "Ajani, Valiant Protector",
        "Tezzeret, Master of Metal",
        "Jalira, Master Polymorphist",
        "House Cartographer",
    ] {
        supported(c);
    }

    // Bag of Tricks: only noncreature cards.
    let mut t = TestGame::new(2);
    let bag = t.battlefield(P0, "Bag of Tricks");
    t.lands(P0, "Forest", 5);
    let before = library_with(&mut t, P0, &["Shock", "Forest"]);
    let hand = t.hand_size(P0);
    t.activate(P0, bag, 0, &[]).unwrap();
    t.resolve_all();
    nothing_found(&t, P0, &before, hand);
    assert!(creatures(&t, P0).is_empty());
    // Positive control: with a creature of each mana value 1–8, exactly one enters.
    let mut t = TestGame::new(2);
    let bag = t.battlefield(P0, "Bag of Tricks");
    t.lands(P0, "Forest", 5);
    stack_library(
        &mut t,
        P0,
        &[
            "Llanowar Elves",
            "Grizzly Bears",
            "Centaur Courser",
            "Hill Giant",
            "Serra Angel",
            "Craw Wurm",
            "Krosan Tusker",
            "Ancient Brontodon",
        ],
    );
    t.activate(P0, bag, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(creatures(&t, P0).len(), 1);

    // Ajani's +1 and Tezzeret's +1.
    for (name, ability, find) in [
        ("Ajani, Valiant Protector", 1, "Grizzly Bears"),
        ("Tezzeret, Master of Metal", 0, "Ornithopter"),
    ] {
        let mut t = TestGame::new(2);
        let pw = t.battlefield(P0, name);
        let before = library_with(&mut t, P0, &["Shock", "Forest"]);
        let hand = t.hand_size(P0);
        t.activate(P0, pw, ability, &[]).unwrap();
        t.resolve_all();
        nothing_found(&t, P0, &before, hand);
        // Positive control: the card is found under the other two.
        let mut t = TestGame::new(2);
        let pw = t.battlefield(P0, name);
        stack_library(&mut t, P0, &["Shock", "Forest", find]);
        t.activate(P0, pw, ability, &[]).unwrap();
        t.resolve_all();
        assert!(t.in_hand(P0, find), "{name}");
        assert_eq!(library_names(&t, P0).len(), 32);
    }

    // Jalira: only a legendary creature.
    let mut t = TestGame::new(2);
    let j = t.battlefield(P0, "Jalira, Master Polymorphist");
    let fodder = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Island", 3);
    let before = library_with(&mut t, P0, &["Ravos, Soultender", "Forest"]);
    let hand = t.hand_size(P0);
    t.answer_choose(P0, &[obj(fodder)]);
    t.activate(P0, j, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    nothing_found(&t, P0, &before, hand);
    assert_eq!(creatures(&t, P0), vec![j]);
    // Positive control: a nonlegendary creature under the legendary one.
    let mut t = TestGame::new(2);
    let j = t.battlefield(P0, "Jalira, Master Polymorphist");
    let fodder = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Island", 3);
    stack_library(&mut t, P0, &["Ravos, Soultender", "Hill Giant"]);
    t.answer_choose(P0, &[obj(fodder)]);
    t.activate(P0, j, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);

    // House Cartographer, tapped as the second main phase begins.
    for land in [false, true] {
        let mut t = TestGame::new(2);
        let hc = t.battlefield(P0, "House Cartographer");
        t.g.tap(hc);
        let top: &[&str] = if land {
            &["Shock", "Forest"]
        } else {
            &["Shock", "Grizzly Bears"]
        };
        let before = library_with(&mut t, P0, top);
        let hand = t.hand_size(P0);
        t.set_step(P0, Step::EndOfCombat);
        t.advance_to(P0, Step::PostcombatMain);
        t.resolve_all();
        if land {
            assert!(t.in_hand(P0, "Forest"));
        } else {
            nothing_found(&t, P0, &before, hand);
        }
    }
}

#[test]
fn nissa_second_landfall_with_no_elf_or_elemental_puts_the_library_back() {
    cr!("701.20a", "701.24a");
    ruling!(
        "Nissa, Resurgent Animist",
        "If you reveal your entire library and don't reveal an Elf or Elemental card, you'll put all revealed cards back in a random order. This uses the same physical action as shuffling, although it isn't technically a \"shuffle.\""
    );
    supported("Nissa, Resurgent Animist");
    for elf in [false, true] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Nissa, Resurgent Animist");
        let top: &[&str] = if elf {
            &["Shock", "Llanowar Elves"]
        } else {
            &["Shock", "Grizzly Bears"]
        };
        let before = library_with(&mut t, P0, top);
        let hand = t.hand_size(P0);
        for _ in 0..2 {
            enter(&mut t, P0, "Forest");
            t.resolve_all();
        }
        if elf {
            assert!(t.in_hand(P0, "Llanowar Elves"));
        } else {
            nothing_found(&t, P0, &before, hand);
        }
    }
}

/// P0 controls The Prismatic Bridge (Esika's back face) with `top` on top of the library
/// (filler cards below); the upkeep trigger resolves. Returns the game, the library
/// contents and the hand size before the trigger.
fn prismatic_bridge(top: &[&str]) -> (TestGame, Vec<String>, usize) {
    let mut t = TestGame::new(2);
    back_face(&mut t, P0, "Esika, God of the Tree // The Prismatic Bridge");
    t.set_step(P1, Step::End);
    let before = library_with(&mut t, P0, top);
    t.advance_to(P0, Step::Upkeep);
    let hand = t.hand_size(P0);
    t.resolve_all();
    (t, before, hand)
}

#[test]
fn prismatic_bridge_finds_front_faces_and_otherwise_puts_the_library_back() {
    cr!("701.20a", "712.14", "711.2");
    ruling!(
        "Esika, God of the Tree // The Prismatic Bridge",
        "If you reveal a double-faced card whose front face is a creature or planeswalker, you'll put it onto the battlefield with its front face up."
    );
    ruling!(
        "Esika, God of the Tree // The Prismatic Bridge",
        "While resolving the triggered ability of The Prismatic Bridge, if there are no creature or planeswalker cards in your library, you'll simply reveal your entire library, randomize it, and continue playing."
    );
    supported("Esika, God of the Tree // The Prismatic Bridge");
    // A transforming DFC whose front face is a creature enters front face up.
    let (t, ..) = prismatic_bridge(&["Shock", "Delver of Secrets"]);
    let d = t
        .g
        .permanents()
        .find(|o| o.controller == P0 && o.chars.name.starts_with("Delver"))
        .map(|o| o.id)
        .expect("Delver entered");
    assert_eq!(t.obj_now(d).chars.name.as_str(), "Delver of Secrets");
    assert!(t.obj_now(d).is(CardType::Creature));
    // The revealed Shock went to the bottom.
    assert_eq!(library_names(&t, P0).last().unwrap(), "Shock");
    // No creature or planeswalker: the library is put back.
    let (t, before, hand) = prismatic_bridge(&["Shock", "Forest"]);
    assert_eq!(before.len(), 32);
    nothing_found(&t, P0, &before, hand);
    assert!(creatures(&t, P0).is_empty());
}

/// Sacrifices a creature through Descendants' Fury (or has it die with Heirloom Blade)
/// with `lib` on top of the library; returns the game.
fn sharing_a_type(card: &str, lib: &[&str]) -> (TestGame, Vec<String>, usize) {
    let mut t = TestGame::new(2);
    let b = t.battlefield(P0, "Grizzly Bears");
    let before = library_with(&mut t, P0, lib);
    let hand = t.hand_size(P0);
    if card == "Descendants' Fury" {
        t.battlefield(P0, "Descendants' Fury");
        yes(&mut t, P0);
        t.answer_choose(P0, &[obj(b)]);
        t.attack(&[(b, Entity::Player(P1))], &[]);
    } else {
        attach_new(&mut t, P0, "Heirloom Blade", b);
        yes(&mut t, P0);
        destroy(&mut t, b);
    }
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"), "{card}");
    (t, before, hand)
}

#[test]
fn sharing_a_creature_type_with_none_found_reveals_and_randomizes() {
    cr!("701.20a", "205.3m");
    ruling!(
        "Descendants' Fury",
        "If you don't reveal a creature card that shares a creature type with the creature you sacrificed, you'll just reveal and randomize your library."
    );
    ruling!(
        "Heirloom Blade",
        "If you don't reveal a creature card that shares a type with the creature that died, you'll just reveal and randomize your library."
    );
    supported("Descendants' Fury");
    supported("Heirloom Blade");
    for card in ["Descendants' Fury", "Heirloom Blade"] {
        let (t, before, hand) = sharing_a_type(card, &["Llanowar Elves", "Forest"]);
        nothing_found(&t, P0, &before, hand);
        assert!(creatures(&t, P0).is_empty(), "{card}");
        // Positive control: a Bear under the Elf is found.
        let (t, ..) = sharing_a_type(card, &["Llanowar Elves", "Forest", "Runeclaw Bear"]);
        let found = if card == "Descendants' Fury" {
            !t.named_on_battlefield("Runeclaw Bear").is_empty()
        } else {
            t.in_hand(P0, "Runeclaw Bear")
        };
        assert!(found, "{card}");
    }
}

/// Fireflux Squad attacks P1 alongside Grizzly Bears, exiling the Bears.
fn fireflux_attack(t: &mut TestGame) -> (ObjectId, ObjectId) {
    let f = t.battlefield(P0, "Fireflux Squad");
    let b = t.battlefield(P0, "Grizzly Bears");
    yes(t, P0);
    t.answer_targets(P0, &[obj(b)]);
    attack_with(t, &[(f, Entity::Player(P1)), (b, Entity::Player(P1))]);
    (f, b)
}

#[test]
fn fireflux_squad_with_no_creature_puts_the_library_back_and_loses_the_attacker() {
    cr!("701.20a", "608.2c");
    ruling!(
        "Fireflux Squad",
        "If you exile your entire library without exiling a creature card, you put the library back in a random order and continue. You won’t return the exiled attacking creature. Oops."
    );
    supported("Fireflux Squad");
    let mut t = TestGame::new(2);
    let lib = t.library_size(P0);
    fireflux_attack(&mut t);
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    assert_eq!(t.library_size(P0), lib);
    assert_eq!(creatures(&t, P0).len(), 1);
    // Positive control: a creature in the library enters attacking.
    let mut t = TestGame::new(2);
    t.library_top(P0, "Hill Giant");
    fireflux_attack(&mut t);
    t.resolve_all();
    let g = t.named_on_battlefield("Hill Giant");
    assert_eq!(g.len(), 1);
    assert!(t.g.is_attacking(g[0]));
}

#[test]
fn fireflux_squad_new_creature_can_attack_a_different_player() {
    cr!("508.4", "506.2");
    ruling!(
        "Fireflux Squad",
        "You choose which player or planeswalker the new creature is attacking. It doesn’t have to be attacking the same player or planeswalker that Fireflux Squad or the exiled creature was attacking."
    );
    for defender in [P2, P1] {
        let mut t = TestGame::new(3);
        t.library_top(P0, "Hill Giant");
        t.answer(
            P0,
            DecisionKind::Entities,
            Answer::Entities(vec![pl(defender)]),
        );
        fireflux_attack(&mut t);
        t.resolve_all();
        let g = t.named_on_battlefield("Hill Giant")[0];
        assert!(t.g.is_attacking(g));
        t.advance_to(P0, Step::EndOfCombat);
        assert_eq!(t.life(defender), 20 - if defender == P1 { 4 + 3 } else { 3 });
        if defender == P2 {
            assert_eq!(t.life(P1), 20 - 4);
        }
    }
}

fn pl(p: PlayerId) -> Entity {
    Entity::Player(p)
}

/// Asserts the outcome of a "look at / reveal the top card, you may put it into your
/// graveyard" ability: with `to_graveyard` it's in the graveyard, otherwise on top.
fn stayed_or_milled(t: &TestGame, name: &str, to_graveyard: bool) {
    assert_eq!(t.in_graveyard(P0, name), to_graveyard, "{name}");
    if !to_graveyard {
        assert_eq!(top_name(t, P0), name);
    }
}

#[test]
fn declined_top_cards_stay_on_top_of_the_library() {
    cr!("608.2c", "401.4");
    ruling!(
        "Kellan, Daring Traveler // Journey On",
        "If you don't put the card you reveal due to Kellan's triggered ability into your hand and you choose not to put it into your graveyard, it stays on top of your library."
    );
    ruling!(
        "The Biblioplex",
        "If you don’t choose to either reveal the card or put it into your graveyard, the card stays on top of your library."
    );
    ruling!(
        "Nylea, Keen-Eyed",
        "If you don’t put the revealed noncreature card into your graveyard, it remains on top of your library."
    );
    ruling!(
        "Territory Culler",
        "You don't have to reveal the card even if it's a creature card. You may leave it on top of your library or put it into your graveyard."
    );
    ruling!(
        "Gathering Stone",
        "You don't have to reveal the card you look at with Gathering Stone's third ability even if it's a card of the chosen type. If you choose not to reveal it this way, you can still put it into your graveyard."
    );
    for c in [
        "Kellan, Daring Traveler // Journey On",
        "The Biblioplex",
        "Nylea, Keen-Eyed",
        "Territory Culler",
        "Gathering Stone",
    ] {
        supported(c);
    }
    for mill in [false, true] {
        // Kellan attacks; the top card is a land.
        let mut t = TestGame::new(2);
        let k = t.battlefield(P0, "Kellan, Daring Traveler // Journey On");
        t.library_top(P0, "Forest");
        t.answer_yes(P0, mill);
        t.attack(&[(k, Entity::Player(P1))], &[]);
        stayed_or_milled(&t, "Forest", mill);

        // The Biblioplex, with an empty hand, looking at an instant: don't reveal it.
        let mut t = TestGame::new(2);
        let b = t.battlefield(P0, "The Biblioplex");
        t.lands(P0, "Wastes", 2);
        t.library_top(P0, "Shock");
        assert_eq!(t.hand_size(P0), 0);
        t.answer_yes(P0, false);
        t.answer_yes(P0, mill);
        t.activate(P0, b, 1, &[]).unwrap();
        t.resolve_all();
        assert!(!t.in_hand(P0, "Shock"));
        stayed_or_milled(&t, "Shock", mill);

        // Nylea reveals a noncreature card.
        let mut t = TestGame::new(2);
        let n = t.battlefield(P0, "Nylea, Keen-Eyed");
        t.lands(P0, "Forest", 3);
        t.library_top(P0, "Shock");
        t.answer_yes(P0, mill);
        t.activate(P0, n, 0, &[]).unwrap();
        t.resolve_all();
        stayed_or_milled(&t, "Shock", mill);

        // Territory Culler looks at a creature card and doesn't reveal it.
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Territory Culler");
        t.library_top(P0, "Grizzly Bears");
        t.answer_yes(P0, false);
        t.answer_yes(P0, mill);
        enter(&mut t, P0, "Forest");
        t.resolve_all();
        assert!(!t.in_hand(P0, "Grizzly Bears"));
        stayed_or_milled(&t, "Grizzly Bears", mill);

        // Gathering Stone (Elf) looks at an Elf card at the beginning of the upkeep.
        let mut t = TestGame::new(2);
        crate::r_p125_common::choose_creature_type(&mut t, P0, "Elf");
        t.enter(P0, "Gathering Stone");
        t.clear_answers();
        t.resolve_all();
        t.set_step(P1, Step::End);
        t.library_top(P0, "Llanowar Elves");
        t.answer_yes(P0, false);
        t.answer_yes(P0, mill);
        t.advance_to(P0, Step::Upkeep);
        t.resolve_all();
        assert!(!t.in_hand(P0, "Llanowar Elves"));
        stayed_or_milled(&t, "Llanowar Elves", mill);
    }
    // Positive controls: Kellan takes a small creature; Territory Culler's card can be
    // revealed and taken.
    let mut t = TestGame::new(2);
    let k = t.battlefield(P0, "Kellan, Daring Traveler // Journey On");
    t.library_top(P0, "Grizzly Bears");
    t.attack(&[(k, Entity::Player(P1))], &[]);
    assert!(t.in_hand(P0, "Grizzly Bears"));
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Territory Culler");
    t.library_top(P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    enter(&mut t, P0, "Forest");
    t.resolve_all();
    assert!(t.in_hand(P0, "Grizzly Bears"));
}

#[test]
fn kenessos_may_leave_the_card_on_top() {
    cr!("608.2c", "401.4");
    ruling!(
        "Kenessos, Priest of Thassa",
        "While resolving Kenessos, Priest of Thassa's last ability, you may choose to leave the card on top of your library."
    );
    supported("Kenessos, Priest of Thassa");
    // (top card, put onto the battlefield?, put on the bottom?)
    for (top, enter_it, bottom) in [
        ("Sea Serpent", false, false),
        ("Sea Serpent", false, true),
        ("Sea Serpent", true, false),
        ("Grizzly Bears", false, false),
    ] {
        let mut t = TestGame::new(2);
        let k = t.battlefield(P0, "Kenessos, Priest of Thassa");
        t.lands(P0, "Island", 4);
        let c = t.library_top(P0, top);
        if top == "Sea Serpent" {
            t.answer_yes(P0, enter_it);
        }
        if !enter_it {
            t.answer_yes(P0, bottom);
        }
        t.activate(P0, k, 0, &[]).unwrap();
        t.resolve_all();
        let names = library_names(&t, P0);
        if enter_it {
            assert!(t.on_battlefield(c));
        } else if bottom {
            assert_eq!(names.last().unwrap(), top, "{top} on the bottom");
        } else {
            assert_eq!(names[0], top, "{top} stays on top");
        }
    }
}

#[test]
fn sapling_of_colfenor_survives_damage_and_destroy_but_not_other_sbas() {
    cr!("702.12b", "704.5g", "704.5f", "704.5j");
    ruling!(
        "Sapling of Colfenor",
        "Lethal damage and effects that say “destroy” won’t cause Sapling of Colfenor to be put into the graveyard. However, Sapling of Colfenor could be put into the graveyard if its toughness becomes 0 or less, if it’s sacrificed, or if you control another Sapling of Colfenor (due to the “legend rule”)."
    );
    // Damage and destroy: it stays.
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Sapling of Colfenor");
    t.g.deal_damage(s, Entity::Object(s), 6, false);
    t.g.flush_events();
    t.settle();
    destroy(&mut t, s);
    assert!(t.on_battlefield(s));
    // Toughness 0.
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Sapling of Colfenor");
    pump(&mut t, s, 0, -5);
    assert!(t.in_graveyard(P0, "Sapling of Colfenor"));
    // Sacrificed (to Fell Shepherd's cost).
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Sapling of Colfenor");
    let fs = t.battlefield(P0, "Fell Shepherd");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Swamp", 1);
    t.answer_choose(P0, &[obj(s)]);
    t.activate(P0, fs, 0, &[obj(bears)]).unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Sapling of Colfenor"));
    // The legend rule.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Sapling of Colfenor");
    let b = t.battlefield(P0, "Sapling of Colfenor");
    t.answer_choose(P0, &[obj(b)]);
    t.settle();
    assert!(t.on_battlefield(a) ^ t.on_battlefield(b));
    assert!(t.in_graveyard(P0, "Sapling of Colfenor"));
}

#[test]
fn sapling_of_colfenor_gains_then_loses_then_takes_the_card() {
    cr!("608.2c", "119.10");
    ruling!(
        "Sapling of Colfenor",
        "The triggered ability results in three sequential events: you gain some life (subject to effects that might alter life gain), then lose some life (subject to other effects that might alter life loss), and then put the card into your hand."
    );
    // Rhox Faithmender doubles the gain only: Hill Giant (3/3): +6, then -3.
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Sapling of Colfenor");
    t.battlefield(P0, "Rhox Faithmender");
    t.library_top(P0, "Hill Giant");
    t.attack(&[(s, Entity::Player(P1))], &[]);
    assert!(t.in_hand(P0, "Hill Giant"));
    assert_eq!(t.life(P0), 23);
    // Positive control without the Faithmender: net zero.
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Sapling of Colfenor");
    t.library_top(P0, "Hill Giant");
    t.attack(&[(s, Entity::Player(P1))], &[]);
    assert!(t.in_hand(P0, "Hill Giant"));
    assert_eq!(t.life(P0), 20);
}
