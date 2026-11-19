//! Rulings batch P084 — values graveyard hate reads from the cards it exiles: power and
//! toughness defined by characteristic-defining abilities in every zone and as the card
//! last existed in the graveyard (CR 604.3, 608.2h), {X} in mana costs off the stack
//! (CR 107.3, 202.3), card types (CR 205.2a), names as the creature leaves the
//! battlefield, and the colors of exiled cards.

use crate::r_p084_common::*;
use crate::r_s06_common::activate_containing;
use mtg_engine::decision::Decision;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::*;

/// Tarmogoyf ("Tarmogoyf's power is equal to the number of card types among cards in all
/// graveyards and its toughness is equal to that number plus 1.") in P1's graveyard with
/// cards of `others`.
fn goyf_in_graveyard(t: &mut TestGame, others: &[&str]) -> ObjectId {
    let g = t.graveyard(P1, "Tarmogoyf");
    for o in others {
        t.graveyard(P1, o);
    }
    t.g.recompute();
    g
}

#[test]
fn morbid_bloom_uses_the_star_toughness_as_the_card_last_existed() {
    cr!("604.3", "608.2h", "113.6a");
    interp!("exiled-card-power-as-it-last-existed-in-the-graveyard");
    ruling!(
        "Morbid Bloom",
        "If a creature card has “*” in its toughness, the ability that defines “*” works in all zones. The value of X is equal to the toughness of the targeted creature card as it last existed in the graveyard."
    );
    let mut t = TestGame::new(2);
    // Creature, instant, land: Tarmogoyf is 3/4 in the graveyard; once it's exiled, only
    // two card types are left (it would be 2/3).
    let goyf = goyf_in_graveyard(&mut t, &["Lightning Bolt", "Forest"]);
    assert_eq!(t.pt(goyf), (3, 4));
    cast_modes(&mut t, P0, "Morbid Bloom", "4BG", &[], &[&[obj(goyf)]]);
    t.resolve_all();
    assert_eq!(tokens_of(&t, P0, "Saproling"), 4);
}

#[test]
fn tymarets_toughness_works_in_the_graveyard() {
    cr!("604.3", "700.5", "608.2h");
    ruling!(
        "Tymaret, Chosen from Death",
        "The ability that defines Tymaret's toughness works in all zones, not just the battlefield."
    );
    // P0's devotion to black is 2 (Grave Robbers, {1}{B}{B}); Tymaret in P0's graveyard is
    // 2/2, and Rotfeaster Maggot gains P0 life equal to its toughness.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grave Robbers");
    let ty = t.graveyard(P0, "Tymaret, Chosen from Death");
    t.g.recompute();
    assert_eq!(t.pt(ty), (2, 2));
    t.answer_targets(P0, &[obj(ty)]);
    t.enter(P0, "Rotfeaster Maggot");
    t.resolve_all();
    assert_eq!(t.zone(ty), Zone::Exile);
    // With Rotfeaster Maggot ({4}{B}) on the battlefield too, the devotion (and Tymaret's
    // toughness in the graveyard as the ability resolves) is 3.
    assert_eq!(t.life(P0), 23);
}

#[test]
fn selfless_exorcist_star_power_applies_in_the_graveyard() {
    cr!("604.3", "608.2h", "120.3");
    interp!("exiled-card-power-as-it-last-existed-in-the-graveyard");
    ruling!(
        "Selfless Exorcist",
        "The reminder text printed on Selfless Exorcist is no longer correct. Abilities that define a creature’s power (and toughness) apply in every zone. The value of * is defined by such an ability."
    );
    supported("Selfless Exorcist");
    // Creature, instant, land, sorcery: Tarmogoyf is 4/5 in the graveyard (3/4 once
    // exiled); it deals 4 damage to the 3/4 Exorcist.
    let mut t = TestGame::new(2);
    let ex = t.battlefield(P0, "Selfless Exorcist");
    let goyf = goyf_in_graveyard(&mut t, &["Lightning Bolt", "Forest", "Divination"]);
    assert_eq!(t.pt(goyf), (4, 5));
    activate(&mut t, P0, ex, "Exile target creature", "", &[obj(goyf)]);
    t.resolve_all();
    assert_eq!(t.zone(goyf), Zone::Exile);
    assert!(t.in_graveyard(P0, "Selfless Exorcist"), "{}", t.dump_log());
}

#[test]
fn lobelia_uses_the_power_as_the_card_last_existed_in_the_graveyard() {
    cr!("608.2h", "604.3");
    interp!("exiled-card-power-as-it-last-existed-in-the-graveyard");
    ruling!(
        "Lobelia Sackville-Baggins",
        "Use the card's power as it last existed in the graveyard to determine the value of X."
    );
    supported("Lobelia Sackville-Baggins");
    let mut t = TestGame::new(2);
    t.graveyard(P1, "Lightning Bolt");
    let goyf = t.battlefield(P1, "Tarmogoyf");
    t.g.destroy(goyf, None);
    t.g.flush_events();
    t.settle();
    let card = t.g.current(goyf);
    // Creature and instant: 2 power in the graveyard (1 once exiled).
    assert_eq!(t.pt(card), (2, 3));
    t.answer_targets(P0, &[obj(card)]);
    t.enter(P0, "Lobelia Sackville-Baggins");
    t.resolve_all();
    assert_eq!(t.zone(card), Zone::Exile);
    assert_eq!(t.named_on_battlefield("Treasure Token").len(), 2);
}

#[test]
fn flesh_counters_use_last_known_power_and_blood_the_power_on_resolution() {
    cr!("608.2h", "702.102c", "709.3");
    interp!("exiled-card-power-as-it-last-existed-in-the-graveyard");
    ruling!(
        "Flesh // Blood",
        "The number of +1/+1 counters is based on the power of the creature card as it last existed in the graveyard."
    );
    ruling!(
        "Flesh // Blood",
        "The amount of damage dealt is based on the first target creature's power as Blood resolves."
    );
    const FB: &str = "Flesh // Blood";
    supported(FB);
    // Flesh alone: Tarmogoyf (creature, instant: 2 power in the graveyard) is exiled;
    // Grizzly Bears gets two counters.
    let mut t = TestGame::new(2);
    let goyf = goyf_in_graveyard(&mut t, &["Lightning Bolt"]);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let fb = t.hand(P0, FB);
    mana(&mut t, P0, "3BG");
    t.answer_targets(P0, &[obj(goyf)]);
    t.answer_targets(P0, &[obj(bears)]);
    t.cast(P0, fb).method(CastMethod::Half(0)).go();
    t.resolve_all();
    assert_eq!(t.counters(bears, "+1/+1"), 2, "{}", t.dump_log());
    // Fused: Flesh's counters are on the Bears by the time Blood deals damage (Hill Giant
    // has 3 power: the Bears deal 5).
    let mut t = TestGame::new(2);
    let giant = t.graveyard(P1, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let fb = t.hand(P0, FB);
    mana(&mut t, P0, "3BGRG");
    t.answer_targets(P0, &[obj(giant)]);
    t.answer_targets(P0, &[obj(bears)]);
    t.answer_targets(P0, &[obj(bears)]);
    t.answer_targets(P0, &[pl(P1)]);
    t.cast(P0, fb)
        .method(CastMethod::Keyword(KeywordKind::Fuse))
        .go();
    t.resolve_all();
    assert_eq!(t.pt(bears), (5, 5));
    assert_eq!(t.life(P1), 15, "{}", t.dump_log());
    // Blood alone, the creature pumped in response: Giant Growth makes it deal 5.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let fb = t.hand(P0, FB);
    mana(&mut t, P0, "RG");
    t.answer_targets(P0, &[obj(bears)]);
    t.answer_targets(P0, &[pl(P1)]);
    t.cast(P0, fb).method(CastMethod::Half(1)).go();
    cast_modes(&mut t, P0, "Giant Growth", "G", &[], &[&[obj(bears)]]);
    t.resolve_all();
    assert_eq!(t.life(P1), 15);
}

#[test]
fn shadows_verdict_treats_x_as_zero_off_the_stack() {
    cr!("107.3", "202.3e");
    ruling!(
        "Shadows' Verdict",
        "If a permanent on the battlefield or a card in a graveyard has {X} in its mana cost, X is considered to be 0."
    );
    supported("Shadows' Verdict");
    let mut t = TestGame::new(2);
    // Endless One ({X}) with four counters: mana value 0. Hangarback Walker ({X}{X}) in a
    // graveyard: mana value 0. Hill Giant: mana value 4.
    let one = t.battlefield(P1, "Endless One");
    t.g.add_counters(obj(one), "+1/+1", 4, None);
    let walker = t.graveyard(P1, "Hangarback Walker");
    let giant = t.battlefield(P1, "Hill Giant");
    cast_modes(&mut t, P0, "Shadows' Verdict", "3BB", &[], &[]);
    t.resolve_all();
    assert_eq!(t.zone(one), Zone::Exile);
    assert_eq!(t.zone(walker), Zone::Exile);
    assert!(t.on_battlefield(giant));
}

#[test]
fn exile_effects_that_read_card_types() {
    cr!("205.2a", "205.4a", "205.3");
    ruling!(
        "Cling to Dust",
        "If the exiled card is a creature card that’s also another card type (such as an enchantment creature), you gain 3 life. You don’t also draw a card."
    );
    ruling!(
        "Deathgorge Scavenger",
        "There's no way to get both bonuses at once while Deathgorge Scavenger's ability is resolving. If an artifact creature card is exiled this way, it's a creature card and not a noncreature card."
    );
    // Cling to Dust on Tymaret (an enchantment creature card): 3 life, no card.
    let mut t = TestGame::new(2);
    let ty = t.graveyard(P1, "Tymaret, Chosen from Death");
    let hand = t.hand_size(P0);
    cast_modes(&mut t, P0, "Cling to Dust", "B", &[], &[&[obj(ty)]]);
    t.resolve_all();
    assert_eq!(t.life(P0), 23);
    assert_eq!(t.hand_size(P0), hand);
    // Deathgorge Scavenger exiling Ornithopter (an artifact creature card): 2 life, no
    // +1/+1.
    supported("Deathgorge Scavenger");
    let mut t = TestGame::new(2);
    let orni = t.graveyard(P1, "Ornithopter");
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(orni)]);
    let ds = t.enter(P0, "Deathgorge Scavenger");
    t.resolve_all();
    assert_eq!(t.zone(orni), Zone::Exile);
    assert_eq!(t.life(P0), 22);
    assert_eq!(t.pt(ds), (3, 2));
}

#[test]
fn keen_eyed_curator_counts_card_types_only() {
    cr!("205.2a", "205.4a", "205.3", "613.4c");
    ruling!(
        "Keen-Eyed Curator",
        "The card types that can appear on cards exiled with Keen-Eyed Curator are artifact, battle, creature, enchantment, instant, kindred, land, planeswalker, and sorcery. Legendary, basic, and snow are supertypes, not card types. Raccoon and Scout are subtypes, not card types."
    );
    supported("Keen-Eyed Curator");
    let mut t = TestGame::new(2);
    let kc = t.battlefield(P0, "Keen-Eyed Curator");
    let exile_with = |t: &mut TestGame, name: &str| {
        let c = t.graveyard(P1, name);
        activate(t, P0, kc, "Exile target card", "1", &[obj(c)]);
        t.resolve_all();
        assert_eq!(t.zone(c), Zone::Exile, "{name}");
    };
    // Forest (basic land), Snow-Covered Forest, a legendary creature, a Raccoon Scout:
    // two card types (land, creature).
    for name in [
        "Forest",
        "Snow-Covered Forest",
        "Isamaru, Hound of Konda",
        "Keen-Eyed Curator",
    ] {
        exile_with(&mut t, name);
    }
    assert_eq!(t.pt(kc), (3, 3));
    // Bitterblossom (Kindred Enchantment — Faerie): four.
    exile_with(&mut t, "Bitterblossom");
    assert_eq!(t.pt(kc), (7, 7));
    assert!(t.obj_now(kc).has_keyword(KeywordKind::Trample));
}

#[test]
fn grime_gorger_exiles_one_card_of_each_card_type() {
    cr!("205.2a", "205.4a", "205.3");
    ruling!(
        "Grime Gorger",
        "The card types that might appear in a player's graveyard are artifact, battle, creature, enchantment, instant, kindred, land, planeswalker, and sorcery. Supertypes (such as legendary and basic) and subtypes (such as Human and Equipment) are not card types."
    );
    supported("Grime Gorger");
    // Two lands (basic and snow), two creatures (one legendary), two Equipment: three
    // card types, so at most three cards, one of each.
    for all in [false, true] {
        let mut t = TestGame::new(2);
        let gg = t.battlefield(P0, "Grime Gorger");
        let cards: Vec<ObjectId> = [
            "Forest",
            "Snow-Covered Forest",
            "Grizzly Bears",
            "Isamaru, Hound of Konda",
            "Bone Saw",
            "Short Sword",
        ]
        .iter()
        .map(|n| t.graveyard(P1, n))
        .collect();
        // Choosing all six (two of each card type) isn't a legal choice; the snow land,
        // the legendary creature and one Equipment are.
        let chosen: Vec<ObjectId> = if all {
            cards.clone()
        } else {
            vec![cards[1], cards[3], cards[5]]
        };
        let es: Vec<Entity> = chosen.iter().map(|c| obj(*c)).collect();
        t.answer_choose(P0, &es);
        t.attack(&[(gg, pl(P1))], &[]);
        t.resolve_all();
        let exiled: Vec<ObjectId> = cards
            .iter()
            .copied()
            .filter(|c| t.zone(*c) == Zone::Exile)
            .collect();
        assert!(exiled.len() <= 3, "all={all}: {exiled:?}");
        // At most one card of each pair (each pair shares its only card type).
        for pair in cards.chunks(2) {
            assert!(
                !(exiled.contains(&pair[0]) && exiled.contains(&pair[1])),
                "all={all}"
            );
        }
        if !all {
            assert_eq!(exiled, chosen);
        }
        assert_eq!(t.counters(gg, "+1/+1") as usize, exiled.len(), "all={all}");
    }
}

#[test]
fn eradicate_splinter_and_scour_target_by_type_then_search_by_name() {
    cr!("201.2a", "608.2h", "613.1d", "701.23a");
    ruling!(
        "Eradicate",
        "If you manage to turn a land (such as a Forest or Stalking Stones) into a creature, you can indeed use this effect on that basic land type. Eradicate’s exile effect only looks for cards by name, not type."
    );
    ruling!(
        "Splinter",
        "Splinter can target and exile cards that aren’t always artifacts, but are artifacts when Splinter is cast and resolves (such as an animated Stalking Stones). Splinter’s exile effect only looks for cards by name, not type."
    );
    ruling!(
        "Scour",
        "Scour can target and exile cards that aren’t always enchantments, but are enchantments when Scour is cast and resolves (such as creatures turned into enchantments by Soul Sculptor). Scour’s exile effect only looks for cards by name, not type."
    );
    supported("Stalking Stones");
    // Stalking Stones animated ("{6}: This land becomes a 3/3 Elemental artifact creature
    // that's still a land."): a nonblack creature and an artifact.
    for spell in [("Eradicate", "2BB"), ("Splinter", "2GG")] {
        let mut t = TestGame::new(2);
        let ss = t.battlefield(P1, "Stalking Stones");
        mana(&mut t, P1, "6");
        activate_containing(&mut t, P1, ss, "becomes").unwrap();
        t.resolve_all();
        let gy = t.graveyard(P1, "Stalking Stones");
        let hand = t.hand(P1, "Stalking Stones");
        let lib = t.library_top(P1, "Stalking Stones");
        let forest = t.graveyard(P1, "Forest");
        cast_modes(&mut t, P0, spell.0, spell.1, &[], &[&[obj(ss)]]);
        t.resolve_all();
        for c in [ss, gy, hand, lib] {
            assert_eq!(t.zone(c), Zone::Exile, "{}", spell.0);
        }
        assert_eq!(t.zone(forest), Zone::Graveyard(P1));
    }
    // Grizzly Bears turned into an enchantment by Soul Sculptor ("{1}{W}, {T}: Target
    // creature becomes an enchantment and loses all abilities until a player casts a
    // creature spell."; "becomes an enchantment" replaces its card types, CR 205.1a).
    supported("Soul Sculptor");
    let mut t = TestGame::new(2);
    let sculptor = t.battlefield(P1, "Soul Sculptor");
    let bears = t.battlefield(P1, "Grizzly Bears");
    activate(
        &mut t,
        P1,
        sculptor,
        "becomes an enchantment",
        "1W",
        &[obj(bears)],
    );
    t.resolve_all();
    let gy = t.graveyard(P1, "Grizzly Bears");
    let hand = t.hand(P1, "Grizzly Bears");
    cast_modes(&mut t, P0, "Scour", "2WW", &[], &[&[obj(bears)]]);
    t.resolve_all();
    for c in [bears, gy, hand] {
        assert_eq!(t.zone(c), Zone::Exile, "Scour");
    }
}

#[test]
fn eradicate_uses_the_name_as_the_creature_leaves_the_battlefield() {
    cr!("608.2h", "707.2", "201.2a");
    ruling!(
        "Eradicate",
        "Use the name of the card as it leaves the battlefield."
    );
    // P1's Clone copying a Grizzly Bears: Eradicate exiles P1's Grizzly Bears cards, not
    // the Clone in P1's hand.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_yes(P1, true);
    t.answer_choose(P1, &[obj(bears)]);
    let clone = t.enter(P1, "Clone");
    t.resolve_all();
    assert_eq!(t.obj_now(clone).chars.name, "Grizzly Bears");
    let their_bears = t.graveyard(P1, "Grizzly Bears");
    let other_clone = t.hand(P1, "Clone");
    cast_modes(&mut t, P0, "Eradicate", "2BB", &[], &[&[obj(clone)]]);
    t.resolve_all();
    assert_eq!(t.zone(clone), Zone::Exile);
    assert_eq!(t.zone(their_bears), Zone::Exile);
    assert_eq!(t.zone(other_clone), Zone::Hand(P1));
    // P0's own Grizzly Bears isn't searched for (only its controller's zones).
    assert!(t.on_battlefield(bears));
}

#[test]
fn pit_of_offerings_makes_only_the_exiled_cards_colors() {
    cr!("106.1a", "106.5", "105.1");
    ruling!(
        "Pit of Offerings",
        "If no cards are exiled with Pit of Offerings, its last ability can't add mana."
    );
    ruling!(
        "Pit of Offerings",
        "The five colors are white, blue, black, red, and green. The last ability of Pit of Offerings can't produce {C}."
    );
    supported("Pit of Offerings");
    // (exiled cards, colors the last ability can make)
    for (exiled, colors) in [
        (&[][..], 0usize),
        (&["Ornithopter"][..], 0),
        (&["Ornithopter", "Lightning Bolt"][..], 1),
        (&["Lightning Bolt", "Watchwolf"][..], 3),
    ] {
        let mut t = TestGame::new(2);
        let cards: Vec<Entity> = exiled.iter().map(|n| obj(t.graveyard(P1, n))).collect();
        t.answer_targets(P0, &cards);
        let pit = t.enter(P0, "Pit of Offerings");
        t.resolve_all();
        assert_eq!(t.g.exile.len(), exiled.len());
        // The colors offered (or the mana made, when there's no choice).
        let from = t.asked().len();
        t.g.players[0].mana_pool.empty();
        // It entered tapped.
        let pit = t.g.current(pit);
        t.g.obj_mut(pit).tapped = false;
        // (With no colors to choose from, it can be activated but adds no mana: 106.5.)
        let _ = t.activate(P0, pit, 1, &[]);
        let offered: Vec<String> = t.asked()[from..]
            .iter()
            .filter_map(|(_, d)| match d {
                Decision::ChooseOption {
                    prompt, options, ..
                } if prompt.contains("mana type") => Some(options.clone()),
                _ => None,
            })
            .flatten()
            .collect();
        let pool = &t.g.players[0].mana_pool;
        let made: std::collections::BTreeSet<String> = if offered.is_empty() {
            [
                ManaType::W,
                ManaType::U,
                ManaType::B,
                ManaType::R,
                ManaType::G,
                ManaType::C,
            ]
            .into_iter()
            .filter(|m| pool.count(*m) > 0)
            .map(|m| format!("{m:?}"))
            .collect()
        } else {
            offered.into_iter().collect()
        };
        assert!(pool.total() <= 1);
        assert!(!made.contains("C"), "{exiled:?}: {made:?}");
        assert_eq!(made.len(), colors, "{exiled:?}: {made:?}");
    }
}
