//! Celebr-8000 (hand-written, `src/cards/celebr_8000.rs`).

use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::decision::Answer;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

const KEYWORDS: [KeywordKind; 6] = [
    KeywordKind::Menace,
    KeywordKind::Vigilance,
    KeywordKind::Lifelink,
    KeywordKind::Flying,
    KeywordKind::Indestructible,
    KeywordKind::DoubleStrike,
];

/// Celebr-8000 on P0's battlefield after its beginning-of-combat trigger resolved with
/// the dice showing `rolls`.
fn roll(rolls: &[u32]) -> (TestGame, ObjectId) {
    let mut t = TestGame::new(2);
    let id = t.battlefield(P0, "Celebr-8000");
    t.set_step(P0, Step::PrecombatMain);
    t.g.dice.loaded.extend(rolls.iter().copied());
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    (t, id)
}

fn keywords(t: &TestGame, id: ObjectId) -> Vec<KeywordKind> {
    KEYWORDS
        .into_iter()
        .filter(|k| t.obj_now(id).has_keyword(*k))
        .collect()
}

#[test]
fn celebr_8000_compiles() {
    let def = mtg_engine::card::card("Celebr-8000");
    assert!(def.unsupported_text().is_empty(), "{:?}", def.unsupported_text());
}

#[test]
fn each_other_result_grants_its_ability_until_end_of_turn() {
    cr!("706.3a");
    let (mut t, id) = roll(&[2, 5]);
    assert_eq!(t.pt(id), (3, 3));
    assert_eq!(
        keywords(&t, id),
        [KeywordKind::Menace, KeywordKind::Flying]
    );
    let (t2, id2) = roll(&[3, 6]);
    assert_eq!(
        keywords(&t2, id2),
        [KeywordKind::Vigilance, KeywordKind::Indestructible]
    );
    let (t3, id3) = roll(&[4, 1]);
    assert_eq!(keywords(&t3, id3), [KeywordKind::Lifelink]);
    assert_eq!(t3.pt(id3), (4, 4), "the 1 gives +1/+1");
    // Until end of turn.
    t.advance_to(P1, Step::Upkeep);
    assert!(keywords(&t, id).is_empty());
    assert_eq!(t.pt(id), (3, 3));
}

#[test]
fn doubles_also_grant_double_strike() {
    cr!("706.5");
    let (t, id) = roll(&[1, 1]);
    assert_eq!(t.pt(id), (5, 5), "+1/+1 for each result of 1");
    assert_eq!(keywords(&t, id), [KeywordKind::DoubleStrike]);
    let (t, id) = roll(&[5, 5]);
    assert_eq!(t.pt(id), (3, 3));
    assert_eq!(
        keywords(&t, id),
        [KeywordKind::Flying, KeywordKind::DoubleStrike]
    );
}

#[test]
fn a_result_above_six_grants_nothing_but_doubles_still_count() {
    cr!("706.2", "706.5");
    ruling!(
        "Celebr-8000",
        "If you roll a result higher than 6 because of modifications to the results, Celebr-8000 won't gain any ability from that roll. If you roll doubles of such a number, it will still gain double strike"
    );
    // Two Xenosquirrels: "After you roll a die, you may remove a +1/+1 counter from this
    // creature. If you do, increase or decrease the result by 1." Each raises a 6 to 7.
    let mut t = TestGame::new(2);
    let id = t.battlefield(P0, "Celebr-8000");
    for _ in 0..2 {
        let s = t.battlefield(P0, "Xenosquirrels");
        t.g.objects[s.0 as usize]
            .counters
            .insert(counters::PLUS1.into(), 2);
    }
    t.set_step(P0, Step::PrecombatMain);
    t.g.dice.loaded.extend([6, 6]);
    // The first Xenosquirrels: on the first die, increase. The second: not on the first
    // die (now 7), on the second, increase.
    for a in [true, false, true] {
        t.answer_yes(P0, a);
    }
    for _ in 0..3 {
        t.answer(P0, DecisionKind::Option, Answer::Index(0));
    }
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert_eq!(t.pt(id), (3, 3));
    assert_eq!(keywords(&t, id), [KeywordKind::DoubleStrike], "7 and 7: doubles only");
}
