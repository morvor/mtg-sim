//! "also" in an instruction that adds to the rest of the ability: "White creatures you
//! control also gain first strike until end of turn." (pattern in
//! `src/oracle/patterns/also_emphasis.rs`).

use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::*;

fn assert_compiles(names: &[&str]) {
    for n in names {
        let u = card(n).unsupported_text().join(" | ");
        assert!(u.is_empty(), "{n} has unsupported text: {u}");
    }
}

fn has(t: &TestGame, id: ObjectId, k: KeywordKind) -> bool {
    t.obj_now(id).chars.has_keyword(k)
}

#[test]
fn also_instructions_compile() {
    assert_compiles(&[
        "Archon's Glory",
        "Sanctified Charge",
        "Heroic Charge",
        "Liliana's Triumph",
        "Goblin Barrage",
        "Efflorescence",
        "Toolcraft Exemplar",
        "Misthios's Fury",
    ]);
}

#[test]
fn sanctified_charge_also_gives_white_creatures_first_strike() {
    cr!("608.2c", "611.2c");
    // "Creatures you control get +2/+1 until end of turn. White creatures you control also
    // gain first strike until end of turn."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let lions = t.battlefield(P0, "Savannah Lions");
    t.lands(P0, "Plains", 5);
    let c = t.hand(P0, "Sanctified Charge");
    t.cast(P0, c).go();
    t.resolve();
    assert_eq!(t.pt(bears), (4, 3));
    assert_eq!(t.pt(lions), (4, 2));
    assert!(!has(&t, bears, KeywordKind::FirstStrike));
    assert!(has(&t, lions, KeywordKind::FirstStrike));
}

#[test]
fn heroic_charge_also_gives_those_creatures_trample_if_kicked() {
    cr!("608.2c", "702.33d");
    // "Kicker {1}{R}. Creatures you control get +2/+1 until end of turn. If this spell was
    // kicked, those creatures also gain trample until end of turn." ({2}{W}{W})
    for kicked in [false, true] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        t.lands(P0, "Plains", 4);
        t.lands(P0, "Mountain", 2);
        let c = t.hand(P0, "Heroic Charge");
        t.cast(P0, c).kicked(kicked).go();
        t.resolve();
        assert_eq!(t.pt(bears), (4, 3));
        assert_eq!(has(&t, bears, KeywordKind::Trample), kicked);
    }
}

#[test]
fn lilianas_triumph_also_makes_each_opponent_discard_with_a_liliana() {
    cr!("608.2c", "701.9a", "701.21a");
    // "Each opponent sacrifices a creature of their choice. If you control a Liliana
    // planeswalker, each opponent also discards a card."
    for liliana in [false, true] {
        let mut t = TestGame::new(2);
        if liliana {
            t.battlefield(P0, "Liliana of the Veil");
        }
        t.battlefield(P1, "Grizzly Bears");
        t.hand(P1, "Lightning Bolt");
        t.lands(P0, "Swamp", 2);
        let c = t.hand(P0, "Liliana's Triumph");
        t.cast(P0, c).go();
        t.resolve();
        assert!(t.in_graveyard(P1, "Grizzly Bears"));
        assert_eq!(t.in_graveyard(P1, "Lightning Bolt"), liliana);
        assert_eq!(t.hand_size(P1), usize::from(!liliana));
    }
}

#[test]
fn archons_glory_also_gives_flying_and_lifelink_if_bargained() {
    cr!("608.2c", "702.166b");
    // "Bargain. Target creature gets +2/+2 until end of turn. If this spell was bargained,
    // that creature also gains flying and lifelink until end of turn."
    for bargain in [false, true] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let stone = t.battlefield(P0, "Mind Stone");
        t.lands(P0, "Plains", 1);
        let c = t.hand(P0, "Archon's Glory");
        t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(bargain));
        t.answer_choose(P0, &[Entity::Object(stone)]);
        t.cast(P0, c).target(bears).go();
        assert_eq!(t.on_battlefield(stone), !bargain);
        t.resolve();
        assert_eq!(t.pt(bears), (4, 4));
        assert_eq!(has(&t, bears, KeywordKind::Flying), bargain);
        assert_eq!(has(&t, bears, KeywordKind::Lifelink), bargain);
    }
}
