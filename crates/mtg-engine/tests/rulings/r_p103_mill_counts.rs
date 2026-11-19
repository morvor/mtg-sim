//! Rulings batch P103 — how many cards are milled: a player mills as many as they can
//! when their library is short (CR 701.17b), "reveal until" effects that run out of
//! library (CR 701.20), the owner arranging milled cards in their graveyard (CR 404.3),
//! and values counted as the effect resolves (CR 608.2h).

use crate::r_p103_common::*;
use crate::r_s01_common::{stack_library, supported, with_subtype};
use crate::r_s06_common::attach_new;
use crate::r_s25_common::cast_new;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

/// `p0` attacks `p1` with `attacker`, answering `targets` for its triggers; combat ends.
fn attack_player(t: &mut TestGame, attacker: ObjectId, targets: &[Entity]) {
    for e in targets {
        t.answer_targets(P0, &[*e]);
    }
    t.attack(&[(attacker, pl(P1))], &[]);
    t.resolve_all();
}

#[test]
fn a_short_library_is_milled_as_far_as_it_goes() {
    cr!("701.17b", "121.4");
    ruling!(
        "Millstone",
        "Can be used on a player with less than 2 cards in their library. It will remove 0 or 1 cards if that is all that is available."
    );
    ruling!(
        "Whetstone",
        "Can be used if a player has less than 2 cards in their library. It will remove 0 or 1 cards if that is all that is available."
    );
    ruling!(
        "Sands of Delirium",
        "If the player has fewer than X cards in their library, they put all of them into their graveyard."
    );
    ruling!(
        "Tome Scour",
        "If there are fewer than five cards in the targeted player’s library, that player puts all the cards from their library into their graveyard."
    );
    ruling!(
        "Nemesis of Reason",
        "If the defending player has fewer than ten cards in their library, that player's entire library is put into their graveyard."
    );
    ruling!(
        "Sword of Body and Mind",
        "If there are fewer than ten cards in that player's library, that player puts all the cards from their library into their graveyard."
    );
    // Each case: the card, P1's library size, the action, P1's graveyard after it.
    type Act = fn(&mut TestGame);
    let cases: Vec<(&str, usize, Act)> = vec![
        ("Millstone", 1, |t| {
            let m = t.battlefield(P0, "Millstone");
            t.lands(P0, "Wastes", 2);
            t.activate(P0, m, 0, &[pl(P1)]).unwrap();
            t.resolve_all();
        }),
        ("Whetstone", 1, |t| {
            library_of(t, P0, "Island", 0);
            let w = t.battlefield(P0, "Whetstone");
            t.lands(P0, "Wastes", 3);
            t.activate(P0, w, 0, &[]).unwrap();
            t.resolve_all();
            // P0's empty library: nothing milled, and nobody lost.
            assert_eq!(t.graveyard_size(P0), 0);
        }),
        ("Sands of Delirium", 3, |t| {
            let s = t.battlefield(P0, "Sands of Delirium");
            t.lands(P0, "Wastes", 5);
            t.answer(P0, DecisionKind::X, Answer::Number(5));
            t.activate(P0, s, 0, &[pl(P1)]).unwrap();
            t.resolve_all();
            assert_eq!(t.g.permanents().filter(|o| o.tapped).count(), 6, "X was 5");
        }),
        ("Tome Scour", 3, |t| {
            cast_new(t, P0, "Tome Scour", &[pl(P1)]);
            t.resolve_all();
        }),
        ("Nemesis of Reason", 4, |t| {
            let n = t.battlefield(P0, "Nemesis of Reason");
            attack_player(t, n, &[]);
        }),
        ("Sword of Body and Mind", 6, |t| {
            let bears = t.battlefield(P0, "Grizzly Bears");
            attach_new(t, P0, "Sword of Body and Mind", obj(bears));
            attack_player(t, bears, &[]);
            assert_eq!(with_subtype(t, P0, "Wolf").len(), 1);
        }),
    ];
    for (name, n, act) in cases {
        supported(name);
        let mut t = TestGame::new(2);
        library_of(&mut t, P1, "Grizzly Bears", n);
        act(&mut t);
        assert_eq!(t.library_size(P1), 0, "{name}");
        assert_eq!(t.graveyard_size(P1), n, "{name}");
        t.settle();
        assert!(!t.has_lost(P0) && !t.has_lost(P1), "{name}");
    }
}

#[test]
fn reveal_until_lands_takes_the_whole_library_when_it_runs_out() {
    cr!("701.20a", "701.17b");
    ruling!(
        "Mind Grind",
        "If an opponent’s library contains fewer than X land cards, all cards from that library are revealed and put into their graveyard."
    );
    ruling!(
        "Mirko Vosk, Mind Drinker",
        "If the player has fewer than four land cards in their library, all cards from that library will be revealed and put into that player's graveyard."
    );
    supported("Mind Grind");
    supported("Mirko Vosk, Mind Drinker");
    let lib = ["Grizzly Bears", "Island", "Hill Giant", "Forest", "Shock"];
    // Mind Grind with X = 3: only two lands.
    let mut t = TestGame::new(2);
    library_of(&mut t, P1, "Island", 0);
    stack_library(&mut t, P1, &lib);
    t.lands(P0, "Wastes", 3);
    t.answer(P0, DecisionKind::X, Answer::Number(3));
    cast_new(&mut t, P0, "Mind Grind", &[]);
    t.resolve_all();
    assert_eq!(t.library_size(P1), 0);
    assert_eq!(t.graveyard_size(P1), 5);
    // Mirko Vosk's combat damage trigger: two lands of four.
    let mut t = TestGame::new(2);
    library_of(&mut t, P1, "Island", 0);
    stack_library(&mut t, P1, &lib);
    let mirko = t.battlefield(P0, "Mirko Vosk, Mind Drinker");
    attack_player(&mut t, mirko, &[]);
    assert_eq!(t.library_size(P1), 0);
    assert_eq!(t.graveyard_size(P1), 5);
}

#[test]
fn balustrade_spy_mills_the_revealed_land_too() {
    cr!("701.20a");
    ruling!(
        "Balustrade Spy",
        "The land card they reveal, if any, is included in the cards put into their graveyard."
    );
    supported("Balustrade Spy");
    let mut t = TestGame::new(2);
    let cards = stack_library(
        &mut t,
        P1,
        &["Grizzly Bears", "Hill Giant", "Island", "Shock"],
    );
    let spy = cast_new(&mut t, P0, "Balustrade Spy", &[]);
    t.answer_targets(P0, &[pl(P1)]);
    t.resolve_all();
    assert!(t.on_battlefield(spy));
    for c in &cards[..3] {
        assert_eq!(t.zone(*c), Zone::Graveyard(P1));
    }
    assert_eq!(t.zone(cards[3]), Zone::Library(P1));
}

#[test]
fn sire_of_stagnation_with_one_or_no_cards_in_the_library() {
    cr!("701.17b", "704.5b");
    ruling!(
        "Sire of Stagnation",
        "If that player has one card in their library when Sire of Stagnation’s triggered ability resolves, that card will be exiled. If their library has zero cards, no cards are exiled. In both cases, you’ll still draw two cards."
    );
    supported("Sire of Stagnation");
    for n in [1, 0] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Sire of Stagnation");
        let lib = library_of(&mut t, P1, "Grizzly Bears", n);
        let exiled = t.g.exile.len();
        let hand = t.hand_size(P0);
        t.enter(P1, "Forest");
        t.g.flush_events();
        t.resolve_all();
        assert_eq!(t.g.exile.len(), exiled + n, "{n}");
        for c in lib {
            assert_eq!(t.zone(c), Zone::Exile);
        }
        assert_eq!(t.hand_size(P0), hand + 2, "{n}");
        // P1 hasn't drawn from the empty library, so hasn't lost.
        assert!(!t.has_lost(P1), "{n}");
    }
}

/// The names in P1's graveyard after `act` mills Grizzly Bears then Hill Giant (the top
/// two cards of P1's library), P1 answering the order with `order`; and whether P1 (and
/// only P1, among the asked) chose the order.
fn milled_in_order(
    act: fn(&mut TestGame),
    order: Vec<usize>,
) -> (Vec<String>, Vec<PlayerId>) {
    let mut t = TestGame::new(2);
    library_of(&mut t, P1, "Island", 2);
    stack_library(&mut t, P1, &["Grizzly Bears", "Hill Giant"]);
    t.answer(P1, DecisionKind::Order, Answer::Indices(order));
    act(&mut t);
    let gy: Vec<String> = graveyard_names(&t, P1)
        .into_iter()
        .filter(|n| n != "Island")
        .collect();
    let askers = t
        .asked()
        .iter()
        .filter(|(_, d)| matches!(d, Decision::Order { .. }))
        .map(|(p, _)| *p)
        .collect();
    (gy, askers)
}

#[test]
fn the_milling_player_arranges_the_milled_cards() {
    cr!("404.3", "701.17a");
    ruling!(
        "Millstone",
        "Since you are putting more than one card in the graveyard at one time, the affected player can choose the order the 2 cards go in."
    );
    ruling!(
        "Whetstone",
        "Since you are putting more than one card in the graveyard at one time, each affected player can choose the order the 2 cards go in."
    );
    ruling!(
        "Traumatize",
        "The player can put the cards in their graveyard in any order they choose."
    );
    let millstone: fn(&mut TestGame) = |t| {
        let m = t.battlefield(P0, "Millstone");
        t.lands(P0, "Wastes", 2);
        t.activate(P0, m, 0, &[pl(P1)]).unwrap();
        t.resolve_all();
    };
    let whetstone: fn(&mut TestGame) = |t| {
        let w = t.battlefield(P0, "Whetstone");
        t.lands(P0, "Wastes", 3);
        t.activate(P0, w, 0, &[]).unwrap();
        t.resolve_all();
    };
    let traumatize: fn(&mut TestGame) = |t| {
        // P1's library is four cards: half of it is the top two.
        cast_new(t, P0, "Traumatize", &[pl(P1)]);
        t.resolve_all();
    };
    for (name, act) in [
        ("Millstone", millstone),
        ("Whetstone", whetstone),
        ("Traumatize", traumatize),
    ] {
        supported(name);
        let (a, asked_a) = milled_in_order(act, vec![0, 1]);
        let (b, asked_b) = milled_in_order(act, vec![1, 0]);
        assert_eq!(a.len(), 2, "{name}");
        assert_ne!(a, b, "{name}: P1's order is followed");
        assert!(asked_a.contains(&P1) && asked_b.contains(&P1), "{name}");
        if name == "Whetstone" {
            // Each affected player arranges their own two cards.
            assert!(asked_a.contains(&P0), "{name}");
        } else {
            assert!(!asked_a.contains(&P0), "{name}");
        }
    }
}

#[test]
fn horrifying_revelation_does_each_part_it_can() {
    cr!("608.2c", "701.17b");
    ruling!(
        "Horrifying Revelation",
        "If the player has no cards in their hand or no cards in their library, the other effect still happens."
    );
    supported("Horrifying Revelation");
    // No cards in hand: the mill still happens.
    let mut t = TestGame::new(2);
    cast_new(&mut t, P0, "Horrifying Revelation", &[pl(P1)]);
    t.resolve_all();
    assert_eq!(t.graveyard_size(P1), 1);
    // No cards in library: the discard still happens.
    let mut t = TestGame::new(2);
    empty_library(&mut t, P1);
    let card = t.hand(P1, "Grizzly Bears");
    cast_new(&mut t, P0, "Horrifying Revelation", &[pl(P1)]);
    t.resolve_all();
    assert_eq!(t.zone(card), Zone::Graveyard(P1));
    assert_eq!(t.graveyard_size(P1), 1);
}

#[test]
fn dreadwaters_counts_lands_as_it_resolves() {
    cr!("608.2h");
    ruling!(
        "Dreadwaters",
        "The number of cards put into the graveyard is based on the number of lands you control when Dreadwaters resolves."
    );
    supported("Dreadwaters");
    let mut t = TestGame::new(2);
    // Four lands to cast it, then two more enter before it resolves.
    cast_new(&mut t, P0, "Dreadwaters", &[pl(P1)]);
    t.lands(P0, "Island", 2);
    t.resolve_all();
    assert_eq!(t.graveyard_size(P1), 6);
}

#[test]
fn lich_lord_of_unx_counts_every_zombie_including_itself() {
    cr!("608.2h");
    ruling!(
        "Lich Lord of Unx",
        "The second ability counts all Zombies you control, not just the tokens put onto the battlefield with the first ability. It will count Lich Lord of Unx itself if it’s still under your control and still a Zombie when the ability resolves."
    );
    supported("Lich Lord of Unx");
    let mut t = TestGame::new(2);
    let lord = t.battlefield(P0, "Lich Lord of Unx");
    // A nontoken Zombie, and a Zombie an opponent controls (not counted).
    t.battlefield(P0, "Walking Corpse");
    t.battlefield(P1, "Walking Corpse");
    t.lands(P0, "Island", 2);
    t.lands(P0, "Swamp", 2);
    t.activate(P0, lord, 1, &[pl(P1)]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.graveyard_size(P1), 2);
}

#[test]
fn grindstone_repeats_only_for_a_shared_color() {
    cr!("105.2c", "608.2c");
    ruling!(
        "Grindstone",
        "If the two cards are both colorless, then the effect does not repeat. It only repeats if both cards have at least one color and at least one color is shared between them."
    );
    supported("Grindstone");
    // (top two cards, how many are milled)
    for (top, milled) in [
        (["Ornithopter", "Memnite"], 2),
        (["Island", "Forest"], 2),
        (["Grizzly Bears", "Llanowar Elves"], 4),
        (["Grizzly Bears", "Shock"], 2),
    ] {
        let mut t = TestGame::new(2);
        library_of(&mut t, P1, "Ornithopter", 4);
        stack_library(&mut t, P1, &top);
        let g = t.battlefield(P0, "Grindstone");
        t.lands(P0, "Wastes", 3);
        t.activate(P0, g, 0, &[pl(P1)]).unwrap();
        t.resolve_all();
        assert_eq!(t.graveyard_size(P1), milled, "{top:?}");
    }
}

#[test]
fn two_fleet_swallowers_halve_the_library_one_after_the_other() {
    cr!("603.3b", "608.2h", "107.1a");
    ruling!(
        "Fleet Swallower",
        "If two Fleet Swallowers attack, each trigger determines the top half of that player’s library separately."
    );
    ruling!(
        "Fleet Swallower",
        "Fleet Swallower’s triggered ability can target any player, not just the player it’s attacking."
    );
    supported("Fleet Swallower");
    let mut t = TestGame::new(3);
    let a = t.battlefield(P0, "Fleet Swallower");
    let b = t.battlefield(P0, "Fleet Swallower");
    library_of(&mut t, P2, "Grizzly Bears", 15);
    let lib1 = t.library_size(P1);
    let from = t.asked().len();
    // Both attack P1; both triggers target P2.
    t.answer_targets(P0, &[pl(P2)]);
    t.answer_targets(P0, &[pl(P2)]);
    t.attack(&[(a, pl(P1)), (b, pl(P1))], &[]);
    t.resolve_all();
    let offered = crate::r_s02_common::target_candidates(&t, P0, from);
    assert!(offered.iter().all(|c| c.contains(&pl(P0)) && c.contains(&pl(P2))));
    // 15 -> mill 8 -> 7 -> mill 4 -> 3.
    assert_eq!(t.graveyard_size(P2), 12);
    assert_eq!(t.library_size(P2), 3);
    assert_eq!(t.library_size(P1), lib1);
}

#[test]
fn psychic_spiral_shuffles_and_doesnt_count_itself() {
    cr!("608.2k", "701.24a");
    ruling!(
        "Psychic Spiral",
        "If you cast Psychic Spiral when there are no cards in your graveyard, you’ll still shuffle your library."
    );
    ruling!(
        "Psychic Spiral",
        "Psychic Spiral won’t be shuffled into your library, and it won’t be counted when determining how many cards the target player puts into their graveyard. It will be put into your graveyard after you follow its instructions."
    );
    supported("Psychic Spiral");
    // An empty graveyard: still shuffled, nothing milled.
    let mut t = TestGame::new(2);
    let spiral = cast_new(&mut t, P0, "Psychic Spiral", &[pl(P1)]);
    t.resolve_all();
    assert!(t.g.turn_events.iter().any(|e| matches!(
        e,
        mtg_engine::events::Event::Shuffled { player } if *player == P0
    )));
    assert_eq!(t.graveyard_size(P1), 0);
    assert_eq!(t.zone(spiral), Zone::Graveyard(P0));
    // Three cards in the graveyard: three milled; the Spiral ends up in the graveyard.
    let mut t = TestGame::new(2);
    for _ in 0..3 {
        t.graveyard(P0, "Grizzly Bears");
    }
    let lib = t.library_size(P0);
    let spiral = cast_new(&mut t, P0, "Psychic Spiral", &[pl(P1)]);
    t.resolve_all();
    assert_eq!(t.graveyard_size(P1), 3);
    assert_eq!(t.library_size(P0), lib + 3);
    assert_eq!(graveyard_names(&t, P0), vec!["Psychic Spiral".to_string()]);
    assert_eq!(t.zone(spiral), Zone::Graveyard(P0));
}

#[test]
fn enter_the_god_eternals_deals_all_4_and_gains_4() {
    cr!("120.3", "120.4a");
    ruling!(
        "Enter the God-Eternals",
        "If the target creature has less than 4 toughness, Enter the God-Eternals still deals 4 damage to it and you'll gain 4 life."
    );
    ruling!(
        "Enter the God-Eternals",
        "You can't cast Enter the God-Eternals without targeting a creature and a player."
    );
    supported("Enter the God-Eternals");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_new(&mut t, P0, "Enter the God-Eternals", &[obj(bears), pl(P1)]);
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Graveyard(P1));
    assert_eq!(t.life(P0), 24);
    // No creature to target: it can't be cast.
    let mut t = TestGame::new(2);
    crate::r_s25_common::lands_for_cost(&mut t, P0, "Enter the God-Eternals");
    let card = t.hand(P0, "Enter the God-Eternals");
    assert!(!crate::r_s02_common::can_cast(
        &mut t,
        P0,
        card,
        mtg_engine::object::CastMethod::Normal
    ));
    assert!(t.cast_with(P0, card, &[pl(P1)]).is_err());
}
