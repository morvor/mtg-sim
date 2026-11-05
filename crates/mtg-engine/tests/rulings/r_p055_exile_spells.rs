//! Rulings batch P055 — instants and sorceries that exile themselves as they resolve
//! (CR 608.2n): wishes and cards outside the game (CR 400.11), returning cards from a
//! graveyard chosen as targets, at random or on resolution (CR 608.2b), the spell going
//! to the graveyard instead when it doesn't resolve (CR 608.2b, 701.6a), Adventures
//! (CR 715.4), extra turns (CR 500.7), life totals that become a number (CR 119.5) and a
//! token's power fixed as it's created (CR 608.2h).

use crate::r_p054_common::{cast_how, o, pool};
use crate::r_p055_common::*;
use crate::r_s01_common::supported;
use crate::r_s02_common::can_cast;
use mtg_engine::card::card;
use mtg_engine::decision::Decision;
use mtg_engine::events::Event;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::Color;
use mtg_engine::*;

/// `p` casts `name` from hand (from a full mana pool), with one target answer per slot.
fn cast_slots(t: &mut TestGame, p: PlayerId, name: &str, slots: &[&[Entity]]) -> ObjectId {
    cast_how(t, p, name, CastMethod::Normal, slots, None, None)
}

/// P1 exiles the card `card` from a graveyard with Scavenging Ooze ("{G}: Exile target
/// card from a graveyard."), and the ability resolves (above whatever is on the stack).
fn ooze_exiles(t: &mut TestGame, card: ObjectId) {
    let ooze = match t.named_on_battlefield("Scavenging Ooze").first() {
        Some(o) => *o,
        None => t.battlefield(P1, "Scavenging Ooze"),
    };
    t.lands(P1, "Forest", 1);
    t.activate(P1, ooze, 0, &[o(card)])
        .expect("Scavenging Ooze");
    t.resolve();
    assert_eq!(t.zone(card), Zone::Exile);
}

/// The sorted names of the objects `es` (as they were, for objects that have moved).
fn names(t: &TestGame, es: &[Entity]) -> Vec<String> {
    let mut v: Vec<String> = es
        .iter()
        .filter_map(|e| match e {
            Entity::Object(id) => Some(t.g.obj(*id).chars.name.to_string()),
            _ => None,
        })
        .collect();
    v.sort();
    v
}

// ---------------------------------------------------------------------------
// Wishes
// ---------------------------------------------------------------------------

/// Slip Out the Back on `p`'s own creature: "Put a +1/+1 counter on target creature. It
/// phases out."
fn phase_out(t: &mut TestGame, id: ObjectId) {
    let c = cast_slots(t, P0, "Slip Out the Back", &[&[o(id)]]);
    let _ = c;
    t.resolve_all();
    assert!(t.obj_now(id).phased_out);
}

#[test]
fn wishes_cant_get_phased_out_cards() {
    cr!("400.11", "702.26b");
    ruling!("Living Wish", "Can't acquire cards that are phased out.");
    ruling!("Golden Wish", "Can’t acquire cards that are phased out.");
    for (wish, phased, outside) in [
        ("Living Wish", "Grizzly Bears", "Hill Giant"),
        ("Golden Wish", "Ornithopter", "Darksteel Myr"),
        ("Death Wish", "Grizzly Bears", "Hill Giant"),
    ] {
        supported(wish);
        let mut t = TestGame::new(2);
        let ph = t.battlefield(P0, phased);
        phase_out(&mut t, ph);
        let side = t.custom(P0, (*card(outside)).clone(), Zone::Outside(P0));
        let from = t.asked().len();
        t.answer_choose(P0, &[o(ph)]);
        cast_slots(&mut t, P0, wish, &[]);
        t.resolve_all();
        let offered = entity_choices(&t, P0, from);
        assert_eq!(offered, vec![vec![o(side)]], "{wish}");
        // The phased-out permanent stays where it is; the wish got nothing.
        assert_eq!(t.zone(ph), Zone::Battlefield);
        assert!(t.obj_now(ph).phased_out);
        assert!(!t.in_hand(P0, phased), "{wish}");
        // The sideboard card is what a wish can get.
        let mut t = TestGame::new(2);
        let side = t.custom(P0, (*card(outside)).clone(), Zone::Outside(P0));
        t.answer_choose(P0, &[o(side)]);
        cast_slots(&mut t, P0, wish, &[]);
        t.resolve_all();
        assert_eq!(t.zone(side), Zone::Hand(P0), "{wish}");
        assert!(t.in_exile(wish));
    }
}

#[test]
fn cunning_wish_gets_a_card_from_the_sideboard() {
    cr!("400.11a");
    ruling!(
        "Cunning Wish",
        "In a sanctioned event, a card that's \"outside the game\" is one that's in your sideboard. In an unsanctioned event, you may choose any card from your collection."
    );
    supported("Cunning Wish");
    let mut t = TestGame::new(2);
    let side = t.custom(P0, (*card("Lightning Bolt")).clone(), Zone::Outside(P0));
    // A sorcery in the sideboard isn't an instant: not offered.
    t.custom(P0, (*card("Lava Axe")).clone(), Zone::Outside(P0));
    let from = t.asked().len();
    t.answer_choose(P0, &[o(side)]);
    cast_slots(&mut t, P0, "Cunning Wish", &[]);
    t.resolve_all();
    assert_eq!(entity_choices(&t, P0, from), vec![vec![o(side)]]);
    assert_eq!(t.zone(side), Zone::Hand(P0));
    assert!(t.in_exile("Cunning Wish"));
}

// ---------------------------------------------------------------------------
// Returning cards from the graveyard
// ---------------------------------------------------------------------------

#[test]
fn reap_the_past_returns_the_whole_graveyard_but_not_itself() {
    cr!("608.2n", "608.2c");
    ruling!(
        "Reap the Past",
        "If X is greater than or equal to the number of cards in your graveyard, you simply return your entire graveyard to your hand."
    );
    ruling!(
        "Reap the Past",
        "Reap the Past is still on the stack while you return cards to your hand, so it can’t return itself."
    );
    ruling!(
        "Reap the Past",
        "No player may take actions between the time the cards are chosen at random and the time they’re returned to your hand."
    );
    supported("Reap the Past");
    for x in [3, 5] {
        let mut t = TestGame::new(2);
        let gy = [
            t.graveyard(P0, "Opt"),
            t.graveyard(P0, "Grizzly Bears"),
            t.graveyard(P0, "Forest"),
        ];
        cast_how(
            &mut t,
            P0,
            "Reap the Past",
            CastMethod::Normal,
            &[],
            Some(x),
            None,
        );
        t.settle();
        let from = t.asked().len();
        t.g.resolve_top();
        // Nobody received priority while it resolved.
        assert!(!priority_asked(&t, from), "X={x}");
        t.settle();
        for c in gy {
            assert_eq!(t.zone(c), Zone::Hand(P0), "X={x}");
        }
        assert_eq!(t.graveyard_size(P0), 0, "X={x}");
        assert!(t.in_exile("Reap the Past"));
    }
    // Positive control: with X less than the graveyard, only X cards are returned.
    let mut t = TestGame::new(2);
    for n in ["Opt", "Grizzly Bears", "Forest"] {
        t.graveyard(P0, n);
    }
    cast_how(
        &mut t,
        P0,
        "Reap the Past",
        CastMethod::Normal,
        &[],
        Some(2),
        None,
    );
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
    assert_eq!(t.graveyard_size(P0), 1);
}

#[test]
fn bond_of_insight_chooses_on_resolution_including_milled_cards() {
    cr!("608.2c", "701.17a", "115.1");
    ruling!(
        "Bond of Insight",
        "Bond of Insight doesn’t target the cards to return to your hand. You choose which cards to return as it resolves. You can choose up to two cards in your graveyard at that time, including ones just put into your graveyard by Bond of Insight."
    );
    ruling!(
        "Bond of Insight",
        "You can return two instant cards, two sorcery cards, one of each, just one instant or sorcery card, or none at all."
    );
    supported("Bond of Insight");
    // P0's graveyard: Shock and Divination; the top four of P0's library: Opt and Lava
    // Axe and two fillers. (Index: Shock, Divination, Opt, Lava Axe.)
    type Pick = &'static [usize];
    let cases: [(&str, Pick); 5] = [
        ("two instants", &[0, 2]),
        ("two sorceries", &[1, 3]),
        ("one of each", &[0, 3]),
        ("just one", &[2]),
        ("none", &[]),
    ];
    for (case, pick) in cases {
        let mut t = TestGame::new(2);
        let shock = t.graveyard(P0, "Shock");
        let div = t.graveyard(P0, "Divination");
        t.library_top(P0, "Opt");
        t.library_top(P0, "Lava Axe");
        let ids: Vec<ObjectId> = vec![shock, div];
        let from = t.asked().len();
        let bond_card = t.hand(P0, "Bond of Insight");
        pool(&mut t, P0);
        let spell = t.cast(P0, bond_card).go();
        // It has no targets.
        assert!(t.asked()[from..]
            .iter()
            .all(|(_, d)| !matches!(d, Decision::ChooseTargets { .. })));
        let _ = spell;
        // Opt and Lava Axe will be milled before the choice.
        let opt = t.g.find_in_zone(Zone::Library(P0), "Opt")[0];
        let axe = t.g.find_in_zone(Zone::Library(P0), "Lava Axe")[0];
        let all = [ids[0], ids[1], opt, axe];
        let chosen: Vec<Entity> = pick.iter().map(|i| o(all[*i])).collect();
        t.answer_choose_moved(P0, &chosen);
        let from = t.asked().len();
        t.resolve_all();
        // Offered: the four instants and sorceries in the graveyard then, two of them
        // just milled; not Bond of Insight (on the stack) or the milled fillers.
        let offered = entity_choices(&t, P0, from);
        assert_eq!(offered.len(), 1, "{case}");
        assert_eq!(
            names(&t, &offered[0]),
            vec!["Divination", "Lava Axe", "Opt", "Shock"],
            "{case}"
        );
        for (i, c) in all.iter().enumerate() {
            let expect = if pick.contains(&i) {
                Zone::Hand(P0)
            } else {
                Zone::Graveyard(P0)
            };
            assert_eq!(t.zone(*c), expect, "{case}: card {i}");
        }
        assert!(t.in_exile("Bond of Insight"));
    }
}

#[test]
fn all_suns_dawn_has_zero_to_five_targets_one_of_each_color() {
    cr!("115.1d", "601.2c");
    ruling!(
        "All Suns' Dawn",
        "All Suns’ Dawn can have from zero to five targets, one for each color."
    );
    supported("All Suns' Dawn");
    // Five targets, one per color.
    let mut t = TestGame::new(2);
    let cards: Vec<ObjectId> = ["Savannah Lions", "Opt", "Duress", "Shock", "Grizzly Bears"]
        .iter()
        .map(|n| t.graveyard(P0, n))
        .collect();
    let slots: Vec<Vec<Entity>> = cards.iter().map(|c| vec![o(*c)]).collect();
    let slots: Vec<&[Entity]> = slots.iter().map(|s| s.as_slice()).collect();
    cast_slots(&mut t, P0, "All Suns' Dawn", &slots);
    t.resolve_all();
    for c in &cards {
        assert_eq!(t.zone(*c), Zone::Hand(P0));
    }
    assert!(t.in_exile("All Suns' Dawn"));
    // Two cards of the same color can't both be chosen: the red slot takes only red.
    let mut t = TestGame::new(2);
    let shock = t.graveyard(P0, "Shock");
    let bolt = t.graveyard(P0, "Lightning Bolt");
    let from = t.asked().len();
    cast_slots(&mut t, P0, "All Suns' Dawn", &[&[o(shock)]]);
    // Only the red target is asked for (no card of another color), and it is one card.
    let asked: Vec<(u32, Vec<Entity>)> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseTargets {
                candidates, max, ..
            } => Some((*max, candidates.clone())),
            _ => None,
        })
        .collect();
    assert_eq!(asked, vec![(1, vec![o(shock), o(bolt)])]);
    t.resolve_all();
    assert_eq!(t.zone(shock), Zone::Hand(P0));
    assert_eq!(t.zone(bolt), Zone::Graveyard(P0));
    // No targets: it resolves (and is exiled).
    let mut t = TestGame::new(2);
    let lions = t.graveyard(P0, "Savannah Lions");
    cast_slots(&mut t, P0, "All Suns' Dawn", &[&[]]);
    t.resolve_all();
    assert_eq!(t.zone(lions), Zone::Graveyard(P0));
    assert!(t.in_exile("All Suns' Dawn"));
}

#[test]
fn all_suns_dawns_target_must_still_have_the_color_it_was_chosen_for() {
    cr!("608.2b", "115.1d");
    ruling!(
        "All Suns' Dawn",
        "You choose the color each target must be when you choose it as a target, so if the card you targeted as a blue card is red when All Suns’ Dawn resolves, the card is an illegal target, even if the spell doesn’t target another red card."
    );
    supported("Celestial Dawn");
    for dawn in [false, true] {
        let mut t = TestGame::new(2);
        let opt = t.graveyard(P0, "Opt");
        cast_slots(&mut t, P0, "All Suns' Dawn", &[&[o(opt)]]);
        if dawn {
            // "Nonland cards you own that aren't on the battlefield are white": Opt is
            // white, not blue, as All Suns' Dawn resolves (and no card was targeted as
            // the white one).
            t.battlefield(P0, "Celestial Dawn");
            t.g.recompute();
            assert!(!t.obj_now(opt).chars.colors.contains(Color::Blue));
        }
        t.resolve_all();
        if dawn {
            assert_eq!(t.zone(opt), Zone::Graveyard(P0));
            assert!(t.in_graveyard(P0, "All Suns' Dawn"));
        } else {
            assert_eq!(t.zone(opt), Zone::Hand(P0));
            assert!(t.in_exile("All Suns' Dawn"));
        }
    }
}

#[test]
fn wildest_dreams_and_seeds_of_renewal_go_to_the_graveyard_when_every_target_is_illegal() {
    cr!("608.2b", "608.2n");
    ruling!(
        "Wildest Dreams",
        "If each target card is an illegal target as Wildest Dreams resolves, Wildest Dreams doesn't resolve and is put into its owner's graveyard. It's not exiled."
    );
    ruling!(
        "Seeds of Renewal",
        "If each target is illegal as Seeds of Renewal resolves, Seeds of Renewal doesn't resolve and is put into its owner's graveyard. It's not exiled."
    );
    ruling!(
        "Seeds of Renewal",
        "If one target becomes illegal before Seeds of Renewal resolves, you'll still return the legal target from your graveyard to your hand."
    );
    ruling!(
        "Vivid Revival",
        "If some, but not all, targets for Vivid Revival become illegal, the remaining targets are returned to your hand and Vivid Revival is exiled. If all of its targets become illegal, Vivid Revival doesn’t resolve and it’s put into your graveyard."
    );
    for (spell, a, b) in [
        ("Wildest Dreams", "Opt", "Shock"),
        ("Seeds of Renewal", "Opt", "Shock"),
        ("Vivid Revival", "Lightning Helix", "Izzet Charm"),
    ] {
        supported(spell);
        for illegal in [0usize, 1, 2] {
            let mut t = TestGame::new(2);
            let ca = t.graveyard(P0, a);
            let cb = t.graveyard(P0, b);
            let x = (spell == "Wildest Dreams").then_some(2);
            cast_how(
                &mut t,
                P0,
                spell,
                CastMethod::Normal,
                &[&[o(ca), o(cb)]],
                x,
                None,
            );
            for c in [ca, cb].iter().take(illegal) {
                ooze_exiles(&mut t, *c);
            }
            t.resolve_all();
            let case = format!("{spell}, {illegal} illegal");
            match illegal {
                0 => {
                    assert_eq!(t.zone(ca), Zone::Hand(P0), "{case}");
                    assert_eq!(t.zone(cb), Zone::Hand(P0), "{case}");
                    assert!(t.in_exile(spell), "{case}");
                }
                1 => {
                    assert_eq!(t.zone(cb), Zone::Hand(P0), "{case}");
                    assert!(t.in_exile(spell), "{case}");
                    assert!(!t.in_graveyard(P0, spell), "{case}");
                }
                _ => {
                    assert!(t.in_graveyard(P0, spell), "{case}");
                    assert!(!t.in_exile(spell), "{case}");
                }
            }
        }
    }
}

#[test]
fn shreds_of_sanity_targets_and_discard() {
    cr!("608.2b", "601.2c", "701.9a");
    ruling!(
        "Shreds of Sanity",
        "If you target two cards and one of the targeted cards becomes illegal, you’ll still return the other one, discard a card, and exile Shreds of Sanity. If each targeted card becomes illegal, Shreds of Sanity doesn’t resolve and you won’t discard a card or exile Shreds of Sanity."
    );
    ruling!(
        "Shreds of Sanity",
        "You may cast Shreds of Sanity with just one target. You may also cast it without any targets if you just want to discard a card."
    );
    ruling!(
        "Shreds of Sanity",
        "You may discard one of the cards you return to your hand. If you have no other cards in hand, you must discard one of the cards you return to your hand."
    );
    supported("Shreds of Sanity");
    // Two targets, the instant becomes illegal: the sorcery returns, P0 discards, it's
    // exiled. (P0 keeps a Grizzly Bears in hand and discards it.)
    let mut t = TestGame::new(2);
    let opt = t.graveyard(P0, "Opt");
    let div = t.graveyard(P0, "Divination");
    let bears = t.hand(P0, "Grizzly Bears");
    cast_slots(&mut t, P0, "Shreds of Sanity", &[&[o(opt)], &[o(div)]]);
    ooze_exiles(&mut t, opt);
    t.answer_choose(P0, &[o(bears)]);
    t.resolve_all();
    assert_eq!(t.zone(div), Zone::Hand(P0));
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert!(t.in_exile("Shreds of Sanity"));
    // Both illegal: no discard, and it goes to the graveyard.
    let mut t = TestGame::new(2);
    let opt = t.graveyard(P0, "Opt");
    let div = t.graveyard(P0, "Divination");
    let bears = t.hand(P0, "Grizzly Bears");
    cast_slots(&mut t, P0, "Shreds of Sanity", &[&[o(opt)], &[o(div)]]);
    ooze_exiles(&mut t, opt);
    ooze_exiles(&mut t, div);
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Hand(P0));
    assert!(t.in_graveyard(P0, "Shreds of Sanity"));
    assert!(!t.in_exile("Shreds of Sanity"));
    // One target, and no targets at all (just the discard).
    for with_target in [true, false] {
        let mut t = TestGame::new(2);
        let opt = t.graveyard(P0, "Opt");
        let bears = t.hand(P0, "Grizzly Bears");
        let slots: [&[Entity]; 2] = if with_target {
            [&[o(opt)], &[]]
        } else {
            [&[], &[]]
        };
        cast_slots(&mut t, P0, "Shreds of Sanity", &slots);
        t.answer_choose(P0, &[o(bears)]);
        t.resolve_all();
        let expect = if with_target {
            Zone::Hand(P0)
        } else {
            Zone::Graveyard(P0)
        };
        assert_eq!(t.zone(opt), expect, "target: {with_target}");
        assert!(t.in_graveyard(P0, "Grizzly Bears"), "target: {with_target}");
        assert!(t.in_exile("Shreds of Sanity"));
    }
    // A returned card may be the one discarded; with no other card in hand, it must be.
    for other in [true, false] {
        let mut t = TestGame::new(2);
        let opt = t.graveyard(P0, "Opt");
        let div = t.graveyard(P0, "Divination");
        let bears = other.then(|| t.hand(P0, "Grizzly Bears"));
        cast_slots(&mut t, P0, "Shreds of Sanity", &[&[o(opt)], &[o(div)]]);
        t.answer_choose_moved(P0, &[o(opt)]);
        let from = t.asked().len();
        t.resolve_all();
        let offered = entity_choices(&t, P0, from);
        assert_eq!(offered.len(), 1, "other card: {other}");
        let want: &[&str] = if bears.is_some() {
            &["Divination", "Grizzly Bears", "Opt"]
        } else {
            &["Divination", "Opt"]
        };
        assert_eq!(names(&t, &offered[0]), want, "other card: {other}");
        assert!(t.in_graveyard(P0, "Opt"), "other card: {other}");
        assert_eq!(t.zone(div), Zone::Hand(P0));
    }
}

#[test]
fn deliver_unto_evil_an_opponent_chooses_among_the_legal_targets() {
    cr!("608.2b", "608.2c", "115.1");
    ruling!(
        "Deliver Unto Evil",
        "If some of the targets chosen are no longer legal targets as Deliver Unto Evil resolves, an opponent chooses among the targets that remain legal. If no targets remain legal, the spell doesn’t resolve. It’s put into its owner’s graveyard and isn’t exiled."
    );
    ruling!(
        "Deliver Unto Evil",
        "You choose one opponent to make the final choice of which cards remains in your graveyard, but all players may discuss and advise this choice."
    );
    supported("Deliver Unto Evil");
    // Three players: P0 targets four cards; one becomes illegal; P0 chooses P2, who
    // chooses two of the three legal ones to stay.
    let mut t = TestGame::new(3);
    let cards: Vec<ObjectId> = ["Opt", "Shock", "Grizzly Bears", "Hill Giant"]
        .iter()
        .map(|n| t.graveyard(P0, n))
        .collect();
    let all: Vec<Entity> = cards.iter().map(|c| o(*c)).collect();
    cast_slots(&mut t, P0, "Deliver Unto Evil", &[&all]);
    ooze_exiles(&mut t, cards[0]);
    t.answer_choose(P0, &[Entity::Player(P2)]);
    t.answer_choose(P2, &[o(cards[1]), o(cards[2])]);
    let from = t.asked().len();
    t.resolve_all();
    let asked = &t.asked()[from..];
    // P0 chose among the opponents; P2 chose among the legal targets.
    let p0: Vec<&Vec<Entity>> = asked
        .iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseEntities { candidates, .. } if *p == P0 => Some(candidates),
            _ => None,
        })
        .collect();
    assert_eq!(p0, vec![&vec![Entity::Player(P1), Entity::Player(P2)]]);
    let p2 = entity_choices(&t, P2, from);
    assert_eq!(p2, vec![vec![o(cards[1]), o(cards[2]), o(cards[3])]]);
    assert!(entity_choices(&t, P1, from).is_empty());
    assert_eq!(t.zone(cards[1]), Zone::Graveyard(P0));
    assert_eq!(t.zone(cards[2]), Zone::Graveyard(P0));
    assert_eq!(t.zone(cards[3]), Zone::Hand(P0));
    assert!(t.in_exile("Deliver Unto Evil"));
    // Every target illegal: it doesn't resolve and goes to the graveyard.
    let mut t = TestGame::new(2);
    let opt = t.graveyard(P0, "Opt");
    cast_slots(&mut t, P0, "Deliver Unto Evil", &[&[o(opt)]]);
    ooze_exiles(&mut t, opt);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Deliver Unto Evil"));
    assert!(!t.in_exile("Deliver Unto Evil"));
}

#[test]
fn deliver_unto_evil_checks_for_a_bolas_planeswalker_on_resolution() {
    cr!("608.2c", "608.2h");
    ruling!(
        "Deliver Unto Evil",
        "Whether you control a Bolas planeswalker is checked only as this spell resolves."
    );
    ruling!(
        "Deliver Unto Evil",
        "You may cast Deliver Unto Evil with no targets if you wish. It will be exiled, and nothing else happens."
    );
    supported("Nicol Bolas, Planeswalker");
    for destroyed in [false, true] {
        let mut t = TestGame::new(2);
        let bolas = t.battlefield(P0, "Nicol Bolas, Planeswalker");
        let cards: Vec<ObjectId> = ["Opt", "Shock", "Grizzly Bears"]
            .iter()
            .map(|n| t.graveyard(P0, n))
            .collect();
        let all: Vec<Entity> = cards.iter().map(|c| o(*c)).collect();
        cast_slots(&mut t, P0, "Deliver Unto Evil", &[&all]);
        if destroyed {
            // P1 destroys the Bolas planeswalker in response.
            pool(&mut t, P1);
            let downfall = t.hand(P1, "Hero's Downfall");
            t.cast(P1, downfall).target(bolas).go();
            t.resolve();
            assert!(!t.on_battlefield(bolas));
            t.answer_choose(P1, &[o(cards[0]), o(cards[1])]);
        }
        t.resolve_all();
        let in_hand: Vec<bool> = cards.iter().map(|c| t.zone(*c) == Zone::Hand(P0)).collect();
        if destroyed {
            assert_eq!(in_hand, vec![false, false, true]);
        } else {
            assert_eq!(in_hand, vec![true, true, true]);
        }
        assert!(t.in_exile("Deliver Unto Evil"));
    }
    // No targets: exiled, nothing else happens (no opponent is asked anything).
    let mut t = TestGame::new(2);
    let opt = t.graveyard(P0, "Opt");
    cast_slots(&mut t, P0, "Deliver Unto Evil", &[&[]]);
    let from = t.asked().len();
    t.resolve_all();
    assert!(entity_choices(&t, P1, from).is_empty());
    assert_eq!(t.zone(opt), Zone::Graveyard(P0));
    assert!(t.in_exile("Deliver Unto Evil"));
}

// ---------------------------------------------------------------------------
// Exiled as part of the effect; countered: the graveyard
// ---------------------------------------------------------------------------

#[test]
fn a_countered_alchemists_gambit_or_alrunds_epiphany_goes_to_the_graveyard() {
    cr!("608.2n", "701.6a");
    ruling!(
        "Alchemist's Gambit",
        "Exiling Alchemist's Gambit happens as part of its resolution. If the spell is countered, it will go to the graveyard as normal."
    );
    ruling!(
        "Alrund's Epiphany",
        "Exiling Alrund's Epiphany as it resolves is part of its effect. If Alrund's Epiphany doesn't resolve, it will be put into its owner's graveyard. If it's exiled this way, it's exiled face up and doesn't become foretold."
    );
    for name in ["Alchemist's Gambit", "Alrund's Epiphany"] {
        supported(name);
        // Countered.
        let mut t = TestGame::new(2);
        let spell = cast_slots(&mut t, P0, name, &[]);
        t.lands(P1, "Island", 2);
        let cs = t.hand(P1, "Counterspell");
        t.cast(P1, cs).target(spell).go();
        t.resolve_all();
        assert!(t.in_graveyard(P0, name), "{name}");
        assert!(!t.in_exile(name), "{name}");
        assert!(t.g.extra_turns.is_empty(), "{name}");
        // Resolved: exiled face up, with an extra turn.
        let mut t = TestGame::new(2);
        cast_slots(&mut t, P0, name, &[]);
        t.resolve_all();
        assert!(t.in_exile(name), "{name}");
        assert_eq!(t.g.extra_turns.len(), 1, "{name}");
        if name == "Alrund's Epiphany" {
            let card = t.g.find_in_zone(Zone::Exile, name)[0];
            assert!(!t.g.obj(card).face_down);
            // Not foretold: it can't be cast from exile for its foretell cost.
            pool(&mut t, P0);
            assert!(!can_cast(
                &mut t,
                P0,
                card,
                CastMethod::Keyword(KeywordKind::Foretell)
            ));
            assert!(!can_cast(&mut t, P0, card, CastMethod::Normal));
        }
    }
}

#[test]
fn alchemists_gambits_extra_turns_most_recent_first() {
    cr!("500.7");
    ruling!(
        "Alchemist's Gambit",
        "If multiple “extra turn” effects resolve in the same turn, the most recently created turn will be taken first."
    );
    // First an Alchemist's Gambit that makes P0 lose at that turn's end step, then one
    // cast for its cleave cost (no loss): the cleaved one's turn comes first.
    let mut t = TestGame::new(2);
    cast_slots(&mut t, P0, "Alchemist's Gambit", &[]);
    t.resolve_all();
    cast_how(
        &mut t,
        P0,
        "Alchemist's Gambit",
        CastMethod::Keyword(KeywordKind::Cleave),
        &[],
        None,
        None,
    );
    t.resolve_all();
    assert_eq!(t.g.extra_turns.len(), 2);
    let turn = t.g.turn.number;
    // The first extra turn: its end step passes without a loss.
    let ok = t.g.run_until(10_000, |g| {
        g.turn.number == turn + 1 && g.turn.step == Step::Cleanup
    });
    assert!(ok);
    assert_eq!(t.g.turn.active, P0);
    assert!(!t.has_lost(P0));
    // The second: P0 loses at its end step.
    let ok =
        t.g.run_until(10_000, |g| g.turn.number == turn + 2 || g.is_over());
    assert!(ok);
    t.g.run_until(10_000, |g| g.turn.step == Step::Cleanup || g.is_over());
    assert_eq!(t.g.turn.number, turn + 2);
    assert!(t.has_lost(P0));
}

// ---------------------------------------------------------------------------
// Twice Upon a Time // Unlikely Meeting (an Adventure)
// ---------------------------------------------------------------------------

/// A game where P0 controls two Doctors (Cult Healer), with Twice Upon a Time in hand.
fn doctors() -> (TestGame, ObjectId) {
    supported("Twice Upon a Time // Unlikely Meeting");
    supported("Cult Healer");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Cult Healer");
    t.battlefield(P0, "Cult Healer");
    t.library_top(P0, "Cult Healer");
    let card = t.hand(P0, "Twice Upon a Time // Unlikely Meeting");
    pool(&mut t, P0);
    (t, card)
}

#[test]
fn twice_upon_a_time_adventure_exile_and_casting_it_later() {
    cr!("715.3d", "307.1", "117.1a");
    ruling!(
        "Twice Upon a Time // Unlikely Meeting",
        "If a spell is cast as an Adventure, its controller exiles it instead of putting it into its owner's graveyard as it resolves. For as long as it remains exiled, that player may cast it as a non-Adventure spell. If an Adventure spell leaves the stack in any way other than resolving (most likely by being countered or by failing to resolve because its targets have all become illegal), that card won't be exiled and the spell's controller won't be able to cast it as a permanent later."
    );
    ruling!(
        "Twice Upon a Time // Unlikely Meeting",
        "You must still follow any relevant timing rules for the non-Adventure spell you cast from exile. Normally, you'll be able to cast it only during your main phase while the stack is empty."
    );
    // Cast as an Adventure: exiled, then castable as Twice Upon a Time.
    let (mut t, card) = doctors();
    t.cast(P0, card).method(CastMethod::Half(1)).go();
    t.resolve_all();
    assert!(t.in_hand(P0, "Cult Healer"));
    let exiled = t.g.current(card);
    assert_eq!(t.zone(exiled), Zone::Exile);
    // Not while the stack isn't empty, nor outside a main phase.
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    assert!(!can_cast(&mut t, P0, exiled, CastMethod::Normal));
    t.resolve_all();
    t.set_step(P0, Step::End);
    assert!(!can_cast(&mut t, P0, exiled, CastMethod::Normal));
    t.set_step(P0, Step::PostcombatMain);
    pool(&mut t, P0);
    assert!(can_cast(&mut t, P0, exiled, CastMethod::Normal));
    t.cast(P0, exiled).go();
    t.resolve_all();
    assert_eq!(t.g.extra_turns.len(), 1);
    // It exiled itself again as Twice Upon a Time ("Exile Twice Upon a Time"): it can't
    // be cast again from there.
    let again = t.g.current(card);
    assert_eq!(t.zone(again), Zone::Exile);
    pool(&mut t, P0);
    assert!(!can_cast(&mut t, P0, again, CastMethod::Normal));
    // Countered: graveyard, no permission.
    let (mut t, card) = doctors();
    let spell = t.cast(P0, card).method(CastMethod::Half(1)).go();
    t.lands(P1, "Island", 2);
    let cs = t.hand(P1, "Counterspell");
    t.cast(P1, cs).target(spell).go();
    t.resolve_all();
    let gy = t.g.current(card);
    assert_eq!(t.zone(gy), Zone::Graveyard(P0));
    assert!(!can_cast(&mut t, P0, gy, CastMethod::Normal));
}

#[test]
fn twice_upon_a_time_exiled_some_other_way_cant_be_cast() {
    cr!("715.3d");
    ruling!(
        "Twice Upon a Time // Unlikely Meeting",
        "If an adventurer card ends up in exile for any other reason than by exiling itself while resolving, it won't give you permission to cast it as a non-Adventure spell."
    );
    // In the graveyard, then exiled by Scavenging Ooze.
    let (mut t, _) = doctors();
    let gy = t.graveyard(P0, "Twice Upon a Time // Unlikely Meeting");
    let card = gy;
    assert_eq!(t.zone(gy), Zone::Graveyard(P0));
    ooze_exiles(&mut t, gy);
    let exiled = t.g.current(card);
    pool(&mut t, P0);
    assert!(!can_cast(&mut t, P0, exiled, CastMethod::Normal));
    assert!(!can_cast(&mut t, P0, exiled, CastMethod::Half(1)));
}

#[test]
fn twice_upon_a_time_uses_only_its_main_characteristics_outside_the_stack() {
    cr!("715.4", "715.3b", "202.3");
    ruling!(
        "Twice Upon a Time // Unlikely Meeting",
        "In every zone other than the stack, Twice Upon a Time uses only its non-Adventure characteristics, and you ignore its alternative characteristics in those cases. For example, while it's in your graveyard, its mana value is 6."
    );
    let mut t = TestGame::new(2);
    let gy = t.graveyard(P0, "Twice Upon a Time // Unlikely Meeting");
    let hand = t.hand(P0, "Twice Upon a Time // Unlikely Meeting");
    assert_eq!(t.g.mana_value_of(gy), 6);
    assert_eq!(t.g.mana_value_of(hand), 6);
    assert_eq!(t.obj_now(gy).chars.name, "Twice Upon a Time");
    // On the stack as an Adventure, its mana value is the Adventure's (3).
    let (mut t, card) = doctors();
    let spell = t.cast(P0, card).method(CastMethod::Half(1)).go();
    assert_eq!(t.g.mana_value_of(spell), 3);
}

// ---------------------------------------------------------------------------
// Life totals and X
// ---------------------------------------------------------------------------

#[test]
fn long_rest_sets_the_life_total_by_gaining_or_losing_life() {
    cr!("119.5", "119.9");
    ruling!(
        "Long Rest",
        "Setting your life total to a specific value counts as gaining or losing life, as appropriate."
    );
    supported("Long Rest");
    for (start, gained) in [(10, 10), (25, -5)] {
        let mut t = TestGame::new(2);
        // "Whenever you gain life, put a +1/+1 counter on Ajani's Pridemate."
        let pridemate = t.battlefield(P0, "Ajani's Pridemate");
        t.g.players[P0.idx()].life = start;
        // Eight cards with different mana values (0 to 7).
        let names = [
            "Ornithopter",
            "Opt",
            "Grizzly Bears",
            "Divination",
            "Hill Giant",
            "Lava Axe",
            "Craw Wurm",
            "Platinum Angel",
        ];
        let cards: Vec<Entity> = names.iter().map(|n| o(t.graveyard(P0, n))).collect();
        cast_how(
            &mut t,
            P0,
            "Long Rest",
            CastMethod::Normal,
            &[&cards],
            Some(8),
            None,
        );
        let from = t.g.turn_events.len();
        t.resolve_all();
        assert_eq!(t.hand_size(P0), 8);
        assert_eq!(t.life(P0), 20);
        let life: Vec<i64> = t.g.turn_events[from..]
            .iter()
            .filter_map(|e| match e {
                Event::LifeGained { player, amount } if *player == P0 => Some(*amount as i64),
                Event::LifeLost { player, amount } if *player == P0 => Some(-(*amount as i64)),
                _ => None,
            })
            .collect();
        assert_eq!(life, vec![gained]);
        assert_eq!(
            t.counters(pridemate, "+1/+1"),
            (gained > 0) as u32,
            "start {start}"
        );
    }
}

#[test]
fn experimental_overloads_weird_keeps_its_size() {
    cr!("608.2h", "111.4");
    ruling!(
        "Experimental Overload",
        "The value of X is determined only as Experimental Overload resolves. The Weird's power and toughness don't change if the number of instant and sorcery cards in your graveyard changes."
    );
    supported("Experimental Overload");
    let mut t = TestGame::new(2);
    let opt = t.graveyard(P0, "Opt");
    t.graveyard(P0, "Divination");
    t.graveyard(P0, "Grizzly Bears");
    cast_slots(&mut t, P0, "Experimental Overload", &[]);
    // In response, a third instant is put into the graveyard: counted.
    t.lands(P0, "Mountain", 1);
    let shock = t.hand(P0, "Shock");
    t.cast(P0, shock).target(P1).go();
    t.resolve();
    // Opt is returned to hand as it resolves.
    t.answer_choose(P0, &[o(opt)]);
    t.resolve_all();
    assert_eq!(t.zone(opt), Zone::Hand(P0));
    let weird = crate::r_s01_common::tokens(&t, P0)[0];
    assert_eq!(t.pt(weird), (3, 3));
    // More instants later: unchanged.
    t.graveyard(P0, "Lightning Bolt");
    t.g.recompute();
    assert_eq!(t.pt(weird), (3, 3));
}
