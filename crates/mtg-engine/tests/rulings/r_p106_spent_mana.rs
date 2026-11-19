//! Rulings batch P106 — what mana with a rider does when it's spent: the Orbs of
//! Dragonkind, Pyromancer's Goggles, Domri's riot mana, Elementalist's Palette's
//! restricted mana, Chandra's emblem counting the mana spent, and {X} costs
//! (CR 106.6, 106.6a, 603.7, 601.2f, 107.3).

use crate::r_p106_common::*;
use mtg_engine::decision::Decision;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

fn has(t: &TestGame, id: ObjectId, k: KeywordKind) -> bool {
    t.obj_now(id).has_keyword(k)
}

/// Taps `source` for its mana ability containing `needle` `times` times (untapping it in
/// between).
fn tap_times(t: &mut TestGame, source: ObjectId, needle: &str, times: usize) {
    for _ in 0..times {
        t.g.untap(source);
        assert!(tap_for_mana(t, P0, source, needle));
    }
}

#[test]
fn two_red_from_pyromancers_goggles_copy_the_spell_twice() {
    cr!("106.6", "603.7", "707.10");
    ruling!(
        "Pyromancer's Goggles",
        "If more than one red mana produced by a Pyromancer's Goggles is spent to cast a single red instant or sorcery spell, the delayed triggered ability associated with each mana spent will trigger. That many copies will be created. It doesn't matter if this red mana was produced by one Pyromancer's Goggles or by multiple Pyromancer's Goggles."
    );
    // One Goggles tapped twice; then two Goggles (Mirror Gallery keeps both) tapped once
    // each. Searing Spear ({1}{R}) is paid with both {R}.
    for two_goggles in [false, true] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Mirror Gallery");
        let g1 = t.battlefield(P0, "Pyromancer's Goggles");
        if two_goggles {
            let g2 = t.battlefield(P0, "Pyromancer's Goggles");
            tap_times(&mut t, g1, "Add {R}", 1);
            tap_times(&mut t, g2, "Add {R}", 1);
        } else {
            tap_times(&mut t, g1, "Add {R}", 2);
        }
        cast_from_hand(&mut t, P0, "Searing Spear", &[Entity::Player(P1)]);
        assert_eq!(pool_total(&t, P0), 0);
        t.settle();
        assert_eq!(t.stack_len(), 3, "two delayed triggers above the spell");
        t.resolve_all();
        assert_eq!(t.life(P1), 11, "the spell and two copies");
    }
}

#[test]
fn two_green_from_jade_orbs_give_two_counters() {
    cr!("106.6", "603.7", "614.1c");
    ruling!(
        "Jade Orb of Dragonkind",
        "If more than one {G} produced by a Jade Orb of Dragonkind is spent to cast a single Dragon creature spell, the delayed triggered ability associated with each mana spent will trigger. That creature will enter the battlefield with that many +1/+1 counters."
    );
    let mut t = TestGame::new(2);
    let o1 = t.battlefield(P0, "Jade Orb of Dragonkind");
    let o2 = t.battlefield(P0, "Jade Orb of Dragonkind");
    tap_times(&mut t, o1, "Add {G}", 1);
    tap_times(&mut t, o2, "Add {G}", 1);
    mana(&mut t, P0, ManaType::R, 2);
    mana(&mut t, P0, ManaType::C, 2);
    cast_from_hand(&mut t, P0, "Shivan Dragon", &[]);
    assert_eq!(pool_total(&t, P0), 0);
    t.resolve_all();
    let d = t.named_on_battlefield("Shivan Dragon")[0];
    assert_eq!(t.counters(d, counters::PLUS1), 2);
}

#[test]
fn jade_orbs_dragon_has_hexproof_as_soon_as_it_enters() {
    cr!("106.6", "603.7", "611.3c");
    ruling!(
        "Jade Orb of Dragonkind",
        "If the mana is spent on a Dragon creature spell, that creature will have hexproof as soon as it enters the battlefield. There is no opportunity to target the creature before it gains hexproof."
    );
    let mut t = TestGame::new(2);
    let orb = t.battlefield(P0, "Jade Orb of Dragonkind");
    tap_times(&mut t, orb, "Add {G}", 1);
    mana(&mut t, P0, ManaType::R, 2);
    mana(&mut t, P0, ManaType::C, 3);
    let spell = cast_from_hand(&mut t, P0, "Shivan Dragon", &[]);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve(); // the delayed trigger
    t.resolve(); // the Dragon
    let d = t.g.current(spell);
    assert_eq!(t.zone(d), Zone::Battlefield);
    assert_eq!(t.stack_len(), 0);
    // Already hexproof the first time anyone could get priority.
    assert!(has(&t, d, KeywordKind::Hexproof));
    assert_eq!(t.counters(d, counters::PLUS1), 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    mana(&mut t, P1, ManaType::R, 1);
    let from = t.asked().len();
    let _ = t.cast_with(P1, bolt, &[obj(d)]);
    let offered = crate::r_s02_common::target_candidates(&t, P1, from);
    assert!(!offered.is_empty());
    assert!(offered.iter().all(|c| !c.contains(&obj(d))));
}

#[test]
fn the_dragonkind_orbs_delayed_triggers_work_after_the_orb_is_gone() {
    cr!("603.7", "106.6");
    ruling!(
        "Jade Orb of Dragonkind",
        "The delayed triggered ability will trigger whether Jade Orb of Dragonkind is still on the battlefield or not."
    );
    ruling!(
        "Lapis Orb of Dragonkind",
        "The delayed triggered ability will trigger whether Lapis Orb of Dragonkind is still on the battlefield or not."
    );
    ruling!(
        "Pyromancer's Goggles",
        "The delayed triggered ability will trigger whether Pyromancer's Goggles is still on the battlefield or not."
    );
    // Jade Orb: destroyed after tapping; the Dragon still gets its counter.
    let mut t = TestGame::new(2);
    let orb = t.battlefield(P0, "Jade Orb of Dragonkind");
    tap_times(&mut t, orb, "Add {G}", 1);
    destroy(&mut t, orb);
    assert!(t.in_graveyard(P0, "Jade Orb of Dragonkind"));
    mana(&mut t, P0, ManaType::R, 2);
    mana(&mut t, P0, ManaType::C, 3);
    cast_from_hand(&mut t, P0, "Shivan Dragon", &[]);
    t.resolve_all();
    let d = t.named_on_battlefield("Shivan Dragon")[0];
    assert_eq!(t.counters(d, counters::PLUS1), 1);
    assert!(has(&t, d, KeywordKind::Hexproof));

    // Lapis Orb: destroyed; casting a Dragon still scries 2.
    let mut t = TestGame::new(2);
    let orb = t.battlefield(P0, "Lapis Orb of Dragonkind");
    tap_times(&mut t, orb, "Add {U}", 1);
    destroy(&mut t, orb);
    mana(&mut t, P0, ManaType::W, 1);
    mana(&mut t, P0, ManaType::C, 3);
    let from = t.asked().len();
    cast_from_hand(&mut t, P0, "Dragonlord Ojutai", &[]);
    t.resolve_all();
    let scries: Vec<usize> = t.asked()[from..]
        .iter()
        .filter_map(|(p, d)| match d {
            Decision::Scry { cards, .. } if *p == P0 => Some(cards.len()),
            _ => None,
        })
        .collect();
    assert_eq!(scries, vec![2]);

    // Pyromancer's Goggles: destroyed; Lightning Bolt is still copied.
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Pyromancer's Goggles");
    tap_times(&mut t, g, "Add {R}", 1);
    destroy(&mut t, g);
    cast_from_hand(&mut t, P0, "Lightning Bolt", &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 14);
}

#[test]
fn the_dragonkind_orbs_mana_can_pay_for_anything() {
    cr!("106.6");
    ruling!(
        "Jade Orb of Dragonkind",
        "The mana created by Jade Orb of Dragonkind can be spent on anything, not just Dragon creature spells."
    );
    ruling!(
        "Lapis Orb of Dragonkind",
        "The mana created by Lapis Orb of Dragonkind can be spent on anything, not just Dragon creature spells."
    );
    // Jade Orb's {G} pays for Grizzly Bears: no counter.
    let mut t = TestGame::new(2);
    let orb = t.battlefield(P0, "Jade Orb of Dragonkind");
    tap_times(&mut t, orb, "Add {G}", 1);
    mana(&mut t, P0, ManaType::C, 1);
    cast_from_hand(&mut t, P0, "Grizzly Bears", &[]);
    assert_eq!(pool_total(&t, P0), 0);
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    let b = t.named_on_battlefield("Grizzly Bears")[0];
    assert_eq!(t.counters(b, counters::PLUS1), 0);
    // Lapis Orb's {U} pays Grizzly Bears' generic mana: no scry.
    let mut t = TestGame::new(2);
    let orb = t.battlefield(P0, "Lapis Orb of Dragonkind");
    tap_times(&mut t, orb, "Add {U}", 1);
    mana(&mut t, P0, ManaType::G, 1);
    let from = t.asked().len();
    cast_from_hand(&mut t, P0, "Grizzly Bears", &[]);
    assert_eq!(pool_total(&t, P0), 0);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert!(!t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::Scry { .. })));
}

#[test]
fn carnelian_orbs_mana_on_any_part_of_a_dragons_cost_gives_haste() {
    cr!("106.6", "601.2f");
    ruling!(
        "Carnelian Orb of Dragonkind",
        "If the mana from Carnelian Orb of Dragonkind is spent to pay any part of the Dragon creature spell's cost, including an alternative or additional cost, the Dragon will gain haste until end of turn."
    );
    // The {R} pays generic mana of the blue Shimmer Dragon.
    let mut t = TestGame::new(2);
    let orb = t.battlefield(P0, "Carnelian Orb of Dragonkind");
    tap_times(&mut t, orb, "Add {R}", 1);
    mana(&mut t, P0, ManaType::U, 2);
    mana(&mut t, P0, ManaType::C, 3);
    cast_from_hand(&mut t, P0, "Shimmer Dragon", &[]);
    assert_eq!(pool_total(&t, P0), 0);
    t.resolve_all();
    let d = t.named_on_battlefield("Shimmer Dragon")[0];
    assert!(has(&t, d, KeywordKind::Haste));
    // Verix Bladewing cast with its kicker (an additional cost): the total cost includes
    // the kicker, and the Orb's {R} spent on it gives haste.
    let mut t = TestGame::new(2);
    let orb = t.battlefield(P0, "Carnelian Orb of Dragonkind");
    tap_times(&mut t, orb, "Add {R}", 1);
    mana(&mut t, P0, ManaType::R, 2);
    mana(&mut t, P0, ManaType::C, 4);
    let v = t.hand(P0, "Verix Bladewing");
    t.cast(P0, v).kicked(true).go();
    assert_eq!(pool_total(&t, P0), 0);
    t.resolve_all();
    let d = t.named_on_battlefield("Verix Bladewing")[0];
    assert!(has(&t, d, KeywordKind::Haste));
}

#[test]
fn carnelian_orbs_mana_on_a_non_dragon_that_becomes_a_dragon_gives_no_haste() {
    cr!("106.6");
    ruling!(
        "Carnelian Orb of Dragonkind",
        "If the mana is spent on a non-Dragon spell that becomes a Dragon later in the turn, that creature won't have haste."
    );
    // Grizzly Bears paid with the Orb's {R}, then Maskwood Nexus makes it a Dragon.
    let mut t = TestGame::new(2);
    let orb = t.battlefield(P0, "Carnelian Orb of Dragonkind");
    tap_times(&mut t, orb, "Add {R}", 1);
    mana(&mut t, P0, ManaType::G, 1);
    cast_from_hand(&mut t, P0, "Grizzly Bears", &[]);
    t.resolve_all();
    let b = t.named_on_battlefield("Grizzly Bears")[0];
    t.battlefield(P0, "Maskwood Nexus");
    t.g.recompute();
    assert!(t.obj_now(b).chars.has_subtype("Dragon"));
    assert!(!has(&t, b, KeywordKind::Haste));
    // With Maskwood Nexus out first, the Bears spell is a Dragon creature spell: haste.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Maskwood Nexus");
    let orb = t.battlefield(P0, "Carnelian Orb of Dragonkind");
    tap_times(&mut t, orb, "Add {R}", 1);
    mana(&mut t, P0, ManaType::G, 1);
    cast_from_hand(&mut t, P0, "Grizzly Bears", &[]);
    t.resolve_all();
    let b = t.named_on_battlefield("Grizzly Bears")[0];
    assert!(has(&t, b, KeywordKind::Haste));
}

#[test]
fn pyromancers_goggles_copy_has_the_same_mode() {
    cr!("707.10", "700.2g");
    ruling!(
        "Pyromancer's Goggles",
        "If the copied spell is modal (that is, it says \"Choose one –\" or the like), the copy will have the same mode or modes. You can't choose a different one."
    );
    // Abrade in its damage mode: the copy also deals damage; P1's Sol Ring survives.
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Pyromancer's Goggles");
    let giant = t.battlefield(P1, "Craw Wurm");
    t.battlefield(P1, "Sol Ring");
    tap_times(&mut t, g, "Add {R}", 1);
    mana(&mut t, P0, ManaType::C, 1);
    let abrade = t.hand(P0, "Abrade");
    t.cast(P0, abrade).modes(&[0]).target(giant).go();
    t.settle();
    t.resolve(); // the Goggles trigger: copies Abrade
    let copies = crate::r_s26_common::spell_copies(&t);
    assert_eq!(copies.len(), 1);
    assert_eq!(crate::r_s26_common::modes_on_stack(&t, copies[0]), vec![0]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Craw Wurm"), "6 damage to the 6/4");
    assert_eq!(t.named_on_battlefield("Sol Ring").len(), 1);
}

#[test]
fn domris_riot_mana_on_a_riot_creature_gives_two_riot_choices() {
    cr!("702.136a", "702.136b", "106.6");
    ruling!(
        "Domri, Chaos Bringer",
        "If a creature enters the battlefield with two instances of riot, you may choose to have it get two +1/+1 counters, one +1/+1 counter and haste, or two instances of haste."
    );
    for (choices, plus1, haste) in [
        ([true, true], 2, false),
        ([true, false], 1, true),
        ([false, false], 0, true),
    ] {
        let mut t = TestGame::new(2);
        let domri = t.battlefield(P0, "Domri, Chaos Bringer");
        t.answer(P0, DecisionKind::Option, Answer::Index(0));
        act(&mut t, P0, domri, "Add {R} or {G}", &[]).unwrap();
        t.resolve_all();
        // Domri's {R}, plus {G} and {C}: all of it pays for Gruul Spellbreaker.
        mana(&mut t, P0, ManaType::G, 1);
        mana(&mut t, P0, ManaType::C, 1);
        cast_from_hand(&mut t, P0, "Gruul Spellbreaker", &[]);
        assert_eq!(pool_total(&t, P0), 0);
        for c in choices {
            t.answer_yes(P0, c);
        }
        t.resolve_all();
        let s = t.named_on_battlefield("Gruul Spellbreaker")[0];
        assert_eq!(t.counters(s, counters::PLUS1), plus1, "{choices:?}\n{}", t.dump_log());
        assert_eq!(has(&t, s, KeywordKind::Haste), haste, "{choices:?}");
    }
}

#[test]
fn elementalists_palette_triggers_once_for_a_spell_with_several_x() {
    cr!("107.3", "603.2");
    ruling!(
        "Elementalist's Palette",
        "If you cast a spell with multiple {X} mana symbols in its cost, the first ability triggers only once."
    );
    // Astral Cornucopia ({X}{X}{X}) with X = 1.
    let mut t = TestGame::new(2);
    let pal = t.battlefield(P0, "Elementalist's Palette");
    mana(&mut t, P0, ManaType::C, 3);
    let ac = t.hand(P0, "Astral Cornucopia");
    t.cast(P0, ac).x(1).go();
    t.resolve_all();
    assert_eq!(t.counters(pal, counters::CHARGE), 2);
}

/// P0's Elementalist's Palette with `n` charge counters, tapped for its restricted {C}.
fn palette_mana(t: &mut TestGame, n: u32) -> ObjectId {
    let pal = t.battlefield(P0, "Elementalist's Palette");
    put_counters(t, pal, counters::CHARGE, n);
    assert!(tap_for_mana(t, P0, pal, "for each charge counter"));
    assert_eq!(pool_total(t, P0), n);
    pal
}

#[test]
fn elementalists_palette_mana_pays_any_part_of_a_cost_with_x() {
    cr!("106.6", "107.3", "118.12");
    ruling!(
        "Elementalist's Palette",
        "The mana added by the last ability can be used only to pay costs with at least one {X}, but it can be used to pay for any part of that cost, not just the {X}. That {X} could be in a spell's total cost, an ability's activation cost, or a cost you're asked to pay as a spell or ability resolves."
    );
    // Not for Grizzly Bears ({1}{G}), which has no {X}.
    let mut t = TestGame::new(2);
    palette_mana(&mut t, 2);
    mana(&mut t, P0, ManaType::G, 1);
    let bears = t.hand(P0, "Grizzly Bears");
    assert!(t.cast_with(P0, bears, &[]).is_err());
    // Stroke of Genius ({X}{2}{U}) with X = 1: the {C} pays the {X} and the {2}.
    let mut t = TestGame::new(2);
    palette_mana(&mut t, 3);
    mana(&mut t, P0, ManaType::U, 1);
    let stroke = t.hand(P0, "Stroke of Genius");
    t.cast(P0, stroke).x(1).target(Entity::Player(P0)).go();
    assert_eq!(pool_total(&t, P0), 0);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    // An activation cost with {X}: Helix Pinnacle's "{X}: Put X tower counters".
    let mut t = TestGame::new(2);
    let helix = t.battlefield(P0, "Helix Pinnacle");
    palette_mana(&mut t, 2);
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    act(&mut t, P0, helix, "tower counters", &[]).unwrap();
    assert_eq!(pool_total(&t, P0), 0);
    t.resolve_all();
    assert_eq!(t.counters(helix, "tower"), 2);
    // A cost asked as a spell resolves: Power Sink's "unless its controller pays {X}".
    let mut t = TestGame::new(2);
    palette_mana(&mut t, 2);
    mana(&mut t, P0, ManaType::G, 1);
    mana(&mut t, P0, ManaType::C, 1);
    let bears = cast_from_hand(&mut t, P0, "Grizzly Bears", &[]);
    assert_eq!(pool_total(&t, P0), 2, "the restricted mana is left");
    mana(&mut t, P1, ManaType::U, 1);
    mana(&mut t, P1, ManaType::C, 2);
    let sink = t.hand(P1, "Power Sink");
    t.g.turn.priority = Some(P1);
    t.cast(P1, sink).x(2).target(bears).go();
    t.answer_yes(P0, true);
    t.resolve();
    assert_eq!(pool_total(&t, P0), 0, "paid with the Palette's mana");
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1, "{}", t.dump_log());
    // Mana Leak's "unless its controller pays {3}" has no {X}: the Palette's mana can't
    // pay it.
    let mut t = TestGame::new(2);
    palette_mana(&mut t, 3);
    mana(&mut t, P0, ManaType::G, 1);
    mana(&mut t, P0, ManaType::C, 1);
    let bears = cast_from_hand(&mut t, P0, "Grizzly Bears", &[]);
    assert_eq!(pool_total(&t, P0), 3);
    mana(&mut t, P1, ManaType::U, 1);
    mana(&mut t, P1, ManaType::C, 1);
    let leak = t.hand(P1, "Mana Leak");
    t.cast_with(P1, leak, &[obj(bears)]).unwrap();
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(pool_total(&t, P0), 3);
}

#[test]
fn elementalists_palette_mana_can_be_split_between_costs() {
    cr!("106.6", "107.3");
    ruling!(
        "Elementalist's Palette",
        "The mana added by the last ability doesn't all have to spent on the same cost."
    );
    let mut t = TestGame::new(2);
    palette_mana(&mut t, 4);
    let a = t.hand(P0, "Endless One");
    t.cast(P0, a).x(2).go();
    assert_eq!(pool_total(&t, P0), 2);
    t.resolve_all();
    let b = t.hand(P0, "Endless One");
    t.cast(P0, b).x(2).go();
    assert_eq!(pool_total(&t, P0), 0);
    t.resolve_all();
    let ones = t.named_on_battlefield("Endless One");
    assert_eq!(ones.len(), 2);
    for o in ones {
        assert_eq!(t.counters(o, counters::PLUS1), 2);
    }
}

#[test]
fn astral_cornucopia_costs_three_mana_per_charge_counter() {
    cr!("107.3", "202.3e", "107.3a");
    ruling!(
        "Astral Cornucopia",
        "If you choose 1 for the value of X, Astral Cornucopia will cost {3} to cast and enter the battlefield with one charge counter. If you choose 2 for the value of X, it will cost {6} to cast and enter the battlefield with two charge counters, and so on."
    );
    for (x, cost) in [(1, 3), (2, 6)] {
        let mut t = TestGame::new(2);
        // One mana short: can't be cast.
        mana(&mut t, P0, ManaType::C, cost - 1);
        let ac = t.hand(P0, "Astral Cornucopia");
        assert!(t.cast(P0, ac).x(x as i64).try_go().is_err(), "X={x}");
        mana(&mut t, P0, ManaType::C, 1);
        t.cast(P0, ac).x(x as i64).go();
        assert_eq!(pool_total(&t, P0), 0, "X={x}");
        t.resolve_all();
        let ac = t.named_on_battlefield("Astral Cornucopia")[0];
        assert_eq!(t.counters(ac, counters::CHARGE), x, "X={x}");
    }
}

/// P0's Chandra, Dressed to Kill emblem (from her −7).
fn chandra_emblem(t: &mut TestGame) {
    let chandra = t.battlefield(P0, "Chandra, Dressed to Kill");
    put_counters(t, chandra, counters::LOYALTY, 4);
    act(t, P0, chandra, "emblem", &[]).unwrap();
    t.resolve_all();
}

#[test]
fn chandras_emblem_counts_the_mana_actually_spent_to_cast() {
    cr!("601.2f", "601.2h", "118.12");
    ruling!(
        "Chandra, Dressed to Kill",
        "The emblem’s triggered ability looks for the actual amount of mana spent to cast the spell. If an effect caused you to pay more or less mana for that spell as you cast it, that will be taken into account when determining the value of X. If an effect would counter that spell unless you pay some amount of mana, that mana doesn’t count as mana spent to cast it."
    );
    // Thalia makes Lightning Bolt cost {1}{R}: X = 2 (the emblem's damage), plus Bolt's 3.
    let mut t = TestGame::new(2);
    chandra_emblem(&mut t);
    t.battlefield(P1, "Thalia, Guardian of Thraben");
    mana(&mut t, P0, ManaType::R, 2);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    cast_from_hand(&mut t, P0, "Lightning Bolt", &[Entity::Player(P1)]);
    assert_eq!(pool_total(&t, P0), 0);
    t.resolve_all();
    assert_eq!(t.life(P1), 15);
    // Mana Leak's {3}, paid after casting, doesn't count: X stays 1.
    let mut t = TestGame::new(2);
    chandra_emblem(&mut t);
    mana(&mut t, P0, ManaType::R, 1);
    let bolt = cast_from_hand(&mut t, P0, "Lightning Bolt", &[Entity::Player(P1)]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.settle();
    mana(&mut t, P1, ManaType::U, 1);
    mana(&mut t, P1, ManaType::C, 1);
    let leak = t.hand(P1, "Mana Leak");
    t.cast_with(P1, leak, &[obj(bolt)]).unwrap();
    mana(&mut t, P0, ManaType::C, 3);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(pool_total(&t, P0), 0, "paid for Mana Leak");
    assert_eq!(t.life(P1), 16, "1 from the emblem and 3 from Bolt\n{}", t.dump_log());
}
