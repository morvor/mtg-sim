//! Celebr-8000: "At the beginning of combat on your turn, roll two six-sided dice. For
//! each result of 1, this creature gets +1/+1 until end of turn. For each other result,
//! it gains the indicated ability until end of turn. If you rolled doubles, it also gains
//! double strike until end of turn." with the table 2 — menace, 3 — vigilance,
//! 4 — lifelink, 5 — flying, 6 — indestructible.
//!
//! The trigger, the roll and the doubles clause are compiled; the per-result bonuses are
//! hand-written as the roll's results table (CR 706.3a): each result of the two dice
//! picks its row, so two 1s give +2/+2. A result outside 1–6 (a modified roll) picks no
//! row (ruling).

use super::{parse, ManualAbility};
use crate::ability::*;
use crate::dice::ResultRow;
use crate::keywords::{Keyword, KeywordKind};

const TEXT: &str = "At the beginning of combat on your turn, roll two six-sided dice. For each result of 1, ~ gets +1/+1 until end of turn. For each other result, it gains the indicated ability until end of turn. If you rolled doubles, it also gains double strike until end of turn.\n• 2 — menace\n• 3 — vigilance\n• 4 — lifelink\n• 5 — flying\n• 6 — indestructible";

/// The compiled parts: the trigger, the roll and "if you rolled doubles".
const COMPILED: &str = "At the beginning of combat on your turn, roll two six-sided dice. If you rolled doubles, ~ gains double strike until end of turn.";

/// The ability each other result indicates.
const TABLE: [(i64, KeywordKind); 5] = [
    (2, KeywordKind::Menace),
    (3, KeywordKind::Vigilance),
    (4, KeywordKind::Lifelink),
    (5, KeywordKind::Flying),
    (6, KeywordKind::Indestructible),
];

fn until_eot(m: Modification) -> Effect {
    Effect::Modify {
        what: Sel::This,
        mods: vec![m],
        duration: Duration::EndOfTurn,
    }
}

fn rows() -> Vec<ResultRow> {
    let mut rows = vec![ResultRow {
        lo: 1,
        hi: Some(1),
        effect: until_eot(Modification::ModifyPT(Value::c(1), Value::c(1))),
    }];
    rows.extend(TABLE.iter().map(|(n, k)| ResultRow {
        lo: *n,
        hi: Some(*n),
        effect: until_eot(Modification::AddKeyword(Keyword::new(*k))),
    }));
    rows
}

/// Gives the (first) die roll in `e` the results table; whether it found one.
fn add_table(e: &mut Effect) -> bool {
    match e {
        Effect::RollDice(r) if r.table.is_empty() => {
            r.table = rows();
            true
        }
        Effect::Seq(v) => v.iter_mut().any(add_table),
        _ => false,
    }
}

fn build(ctx: &crate::oracle::CompileContext) -> Vec<Ability> {
    let mut found = true;
    let out: Vec<Ability> = parse(ctx, COMPILED)
        .iter()
        .map(|a| {
            super::map_effect(a, TEXT, |mut e| {
                found &= add_table(&mut e);
                e
            })
        })
        .collect();
    if !found || out.len() != 1 {
        // Not the compiled shape this definition expects: unsupported, not approximated.
        return vec![AbilityDef::new(AbilityKind::Unsupported(TEXT.into()), TEXT)];
    }
    out
}

inventory::submit! { ManualAbility {
    card: "Celebr-8000",
    face: 0,
    text: TEXT,
    build,
    reason: "a +1/+1 or keyword for each die result from a bulleted table (CR 706.5 names it as the only such card): unique",
} }
