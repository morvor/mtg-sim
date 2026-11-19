//! Item `rulings-residue-keywords` — rulings that restate what the card's own text says
//! (devour land, heroic-adjacent counter triggers, hexproof from a color, repeated
//! imprint, Thought Prison's single trigger, kicker basics, Baron Sengir's Vampire
//! targets, Malcolm with one opponent): one compact table, each case with its own setup,
//! assertion and ruling citation.

use crate::r_residue_keywords_common::*;
use mtg_engine::decision::Decision;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

/// A case: the card, and a test of the ruling's claim on a fresh game.
type Case = (&'static str, fn());

fn famished_worldsire_devours_lands() {
    ruling!(
        "Famished Worldsire",
        "Devour land is a variant of the devour ability. It allows you to sacrifice lands rather than creatures, but otherwise functions identically to devour."
    );
    ruling!(
        "Famished Worldsire",
        "You may choose not to sacrifice any lands for the devour land ability."
    );
    for n in [2usize, 0] {
        let mut t = TestGame::new(2);
        let lands = t.lands(P0, "Forest", 3);
        t.battlefield(P0, "Grizzly Bears");
        t.answer_choose(P0, &lands[..n].iter().map(|l| obj(*l)).collect::<Vec<_>>());
        let w = t.enter(P0, "Famished Worldsire");
        assert_eq!(p1(&t, w), 3 * n as u32);
        assert_eq!(
            t.g.permanents()
                .filter(|o| o.chars.name == "Forest")
                .count(),
            3 - n
        );
        assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    }
}

fn bloodcrazed_hoplite_any_counter() {
    ruling!(
        "Bloodcrazed Hoplite",
        "Bloodcrazed Hoplite's last ability will trigger whenever any +1/+1 counter is placed on it, not just ones due to its heroic ability."
    );
    let mut t = TestGame::new(2);
    let h = t.battlefield(P0, "Bloodcrazed Hoplite");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    crate::r_p120_common::give_plus1(&mut t, theirs, 1);
    t.answer_targets(P0, &[obj(theirs)]);
    crate::r_p108_common::put_counters(&mut t, h, mtg_engine::types::counters::PLUS1, 1);
    t.resolve_all();
    assert_eq!(p1(&t, theirs), 0);
}

fn garruks_harbinger_hexproof_from_black() {
    ruling!(
        "Garruk's Harbinger",
        "Hexproof from black means that Garruk's Harbinger can't be the target of black spells your opponents control or abilities your opponents control of black sources."
    );
    let candidates = |caster: PlayerId, spell: &str| {
        let mut t = TestGame::new(2);
        let h = t.battlefield(P0, "Garruk's Harbinger");
        t.battlefield(P0, "Grizzly Bears");
        let from = t.asked().len();
        crate::r_s25_common::lands_for_cost(&mut t, caster, spell);
        let c = t.hand(caster, spell);
        let _ = t.cast(caster, c).try_go();
        crate::r_s02_common::target_candidates(&t, caster, from)
            .first()
            .is_some_and(|c| c.contains(&obj(h)))
    };
    // An opponent's black spell can't target it; a red one can, and so can its
    // controller's black spell.
    assert!(!candidates(P1, "Murder"));
    assert!(candidates(P1, "Lightning Bolt"));
    assert!(candidates(P0, "Murder"));
}

fn death_mask_duplicant_many_imprints() {
    ruling!(
        "Death-Mask Duplicant",
        "Death-Mask Duplicant has an activated imprint ability. This ability can be activated any number of times, so multiple creature cards can become imprinted on this creature."
    );
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Death-Mask Duplicant");
    let a = t.graveyard(P0, "Wind Drake");
    let b = t.graveyard(P0, "Youthful Knight");
    t.lands(P0, "Wastes", 2);
    t.activate(P0, d, 0, &[obj(a)]).unwrap();
    t.activate(P0, d, 0, &[obj(b)]).unwrap();
    t.resolve_all();
    assert_eq!(t.zone(a), Zone::Exile);
    assert_eq!(t.zone(b), Zone::Exile);
    assert!(has(&mut t, d, mtg_engine::keywords::KeywordKind::Flying));
    assert!(has(
        &mut t,
        d,
        mtg_engine::keywords::KeywordKind::FirstStrike
    ));
}

fn thought_prison_one_trigger_per_spell() {
    ruling!(
        "Thought Prison",
        "The second ability triggers only once per spell. If the spell shares both a color and a mana value with the imprinted card, Thought Prison deals 2 damage, not 4."
    );
    let mut t = TestGame::new(2);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(bolt)]);
    t.enter(P0, "Thought Prison");
    t.resolve_all();
    assert_eq!(t.zone(bolt), Zone::Exile);
    // P1's Shock: red and mana value 1, like the Bolt.
    cast_new(&mut t, P1, "Shock", &[Entity::Player(P0)]);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

/// Casts Consult the Star Charts (kicked or not) with four lands and returns how many
/// cards it put into P0's hand, the spell's mana value, and how many optional-cost
/// questions were asked.
fn consult(kicked: bool) -> (usize, u32, usize) {
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 4);
    let c = t.hand(P0, "Consult the Star Charts");
    let from = t.asked().len();
    let s = t.cast(P0, c).kicked(kicked).go();
    let mv = t.g.mana_value_of(s);
    let asked = count_asked(&t, from, |d| matches!(d, Decision::OptionalCost { .. }));
    let hand = t.hand_size(P0);
    t.resolve_all();
    (t.hand_size(P0) - hand, mv, asked)
}

fn consult_the_star_charts_kicker() {
    ruling!(
        "Consult the Star Charts",
        "If a spell’s kicker cost was paid, the spell is “kicked.”"
    );
    ruling!(
        "Consult the Star Charts",
        "The kicker ability doesn’t let you pay a kicker cost more than once."
    );
    ruling!(
        "Consult the Star Charts",
        "To determine a spell’s total cost, start with the mana cost (or an alternative cost if another card’s effect allows you to pay one instead), add any cost increases (such as kicker), then apply any cost reductions. The spell’s mana value remains unchanged, no matter what the total cost to cast it was."
    );
    // Kicked: two cards; one yes/no question about the kicker; mana value 2 although {3}{U}
    // ... {1}{U}{1}{U} was paid.
    assert_eq!(consult(true), (2, 2, 1));
    assert_eq!(consult(false), (1, 2, 1));
}

fn baron_sengir_regenerates_vampires_by_type() {
    ruling!(
        "Baron Sengir",
        "The Baron’s ability will only regenerate creatures with type “Vampire,” regardless of whether the creature has “Vampire” in its name."
    );
    let mut t = TestGame::new(2);
    let baron = t.battlefield(P0, "Baron Sengir");
    let bats = t.battlefield(P0, "Vampire Bats");
    let vamp = t.battlefield(P0, "Vampire Interloper");
    let from = t.asked().len();
    t.activate(P0, baron, 0, &[obj(vamp)]).unwrap();
    let offered = crate::r_s02_common::target_candidates(&t, P0, from);
    assert!(offered[0].contains(&obj(vamp)));
    assert!(!offered[0].contains(&obj(bats)));
}

fn malcolm_one_opponent_one_treasure() {
    ruling!(
        "Malcolm, Keen-Eyed Navigator",
        "Malcolm's triggered ability will trigger even if Pirates you control deal damage to only a single opponent. A single Treasure for you. Yarrr."
    );
    let mut t = TestGame::new(3);
    let m = t.battlefield(P0, "Malcolm, Keen-Eyed Navigator");
    attack_with(&mut t, &[(m, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[]);
    t.resolve_all();
    assert_eq!(tokens_of(&t, P0, "Treasure"), 1);
}

#[test]
fn keyword_card_restatements() {
    cr!(
        "702.82a", "702.82b", "702.82c", "122.6", "702.11d", "607.2a", "603.2", "702.33a",
        "702.33d", "202.3", "601.2f", "115.1", "205.3a", "603.2c"
    );
    let cases: &[Case] = &[
        ("Famished Worldsire", famished_worldsire_devours_lands),
        ("Bloodcrazed Hoplite", bloodcrazed_hoplite_any_counter),
        ("Garruk's Harbinger", garruks_harbinger_hexproof_from_black),
        ("Death-Mask Duplicant", death_mask_duplicant_many_imprints),
        ("Thought Prison", thought_prison_one_trigger_per_spell),
        ("Consult the Star Charts", consult_the_star_charts_kicker),
        ("Baron Sengir", baron_sengir_regenerates_vampires_by_type),
        (
            "Malcolm, Keen-Eyed Navigator",
            malcolm_one_opponent_one_treasure,
        ),
    ];
    for (card, case) in cases {
        supported(card);
        case();
    }
}
