//! Pronouns with no antecedent.
//!
//! An instant's or sorcery's text is its spell ability (CR 113.3a): "it", "that
//! creature", "that player", "its controller" in it refer to an object or player an
//! earlier instruction mentioned (CR 608.2c), never to the spell itself, which oracle
//! text calls "~" ("this spell"). A permanent's ability may say "it" for the permanent
//! itself ("{1}: Put a +1/+1 counter on it" is rare, but "~ gets +1/+1. It gains
//! flying" is common), so there the effect parser's default referent stays the source.
//!
//! For spells, the effect parser's builder starts out with [`NO_REFERENT`] as what "it"
//! and "that player" mean. The pronoun resolvers ([`crate::oracle::effects::object_ref`],
//! [`crate::oracle::effects::player_ref`]) refuse to resolve a pronoun to it, and any
//! compiled ability that still contains it (a pattern that copied the builder's referent
//! directly) is rejected as unsupported by [`has_no_referent`]: a paragraph such as
//! "Search your library for a Dinosaur creature card, put it onto the battlefield, then
//! shuffle. It gains indestructible until your next turn." must not compile to the spell
//! gaining indestructible.
//!
//! "That creature", "that card", "that permanent" and the like are never the source even
//! for a permanent (oracle text calls the source "this creature"/"~"), so they don't
//! resolve to the source's default referent either (see
//! [`crate::oracle::effects::object_ref`]).

use crate::ability::*;
use crate::oracle::effects::Builder;

/// The variable standing for "no antecedent" (never stored by any instruction).
pub const NO_REFERENT: Var = 0x7ffe;

/// What "it" means before anything gave it an antecedent in a spell's text.
pub fn no_referent() -> Sel {
    Sel::Var(NO_REFERENT)
}

/// What "that player" means before anything gave it an antecedent in a spell's text.
pub fn no_player_referent() -> PlayerRef {
    PlayerRef::Var(NO_REFERENT)
}

/// Whether `s` is the "no antecedent" placeholder.
pub fn is_no_referent(s: &Sel) -> bool {
    matches!(s, Sel::Var(v) if *v == NO_REFERENT)
}

/// Whether `p` is the "no antecedent" placeholder, or a player defined by it ("its
/// controller" with no antecedent for "it").
pub fn is_no_player_referent(p: &PlayerRef) -> bool {
    match p {
        PlayerRef::Var(v) => *v == NO_REFERENT,
        PlayerRef::ControllerOf(s) | PlayerRef::OwnerOf(s) => is_no_referent(s),
        _ => false,
    }
}

/// Before a sentence of a spell's text is parsed: if nothing gave "it" an antecedent yet
/// and the spell itself is the sentence's subject ("~ deals 1 damage to each creature"),
/// "it" in later sentences refers to the spell ("If it was kicked, it deals 2 damage to
/// each creature instead.").
pub fn note_subject(sentence: &str, b: &mut Builder) {
    if is_no_referent(&b.it) && sentence.starts_with("~ ") {
        b.it = Sel::This;
    }
}

/// After a sentence has been parsed: if it ended by bringing new objects into play —
/// creating tokens ("Create a 1/1 white Ally creature token. Put a +1/+1 counter on it
/// for each ...") or finding cards in a library ("Search your library for a Dinosaur
/// creature card, put it onto the battlefield, then shuffle. It gains indestructible ...")
/// — and "it" had no more specific antecedent than the source, later pronouns refer to
/// those objects (the new objects the instruction recorded, CR 400.7).
pub fn note_introduced(e: &Effect, b: &mut Builder) {
    if !(matches!(b.it, Sel::This) || is_no_referent(&b.it)) {
        return;
    }
    let mut creates = 0;
    count_creates(e, &mut creates);
    match last_instruction(e) {
        Effect::CreateToken { .. } | Effect::CreateTokenCopy { .. } if creates == 1 => {
            b.it = Sel::Var(vars::CREATED);
        }
        Effect::Search { .. } => b.it = Sel::Var(vars::IT),
        _ => {}
    }
}

/// The instruction an effect ends with (looking into sequences and optional parts).
fn last_instruction(e: &Effect) -> &Effect {
    match e {
        Effect::Seq(v) => v.last().map_or(e, last_instruction),
        Effect::May { effect, .. } => last_instruction(effect),
        Effect::If {
            then, otherwise, ..
        } if matches!(**otherwise, Effect::Noop) => last_instruction(then),
        _ => e,
    }
}

fn count_creates(e: &Effect, n: &mut usize) {
    match e {
        Effect::Seq(v) => v.iter().for_each(|x| count_creates(x, n)),
        Effect::May { effect, .. } => count_creates(effect, n),
        Effect::If {
            then, otherwise, ..
        } => {
            count_creates(then, n);
            count_creates(otherwise, n);
        }
        Effect::CreateToken { .. } | Effect::CreateTokenCopy { .. } => *n += 1,
        _ => {}
    }
}

/// Whether a compiled ability refers to an object or player through a pronoun that had
/// no antecedent (so the compiler didn't understand what it refers to).
pub fn has_no_referent(a: &AbilityDef) -> bool {
    let needle = format!("\"Var\":{NO_REFERENT}");
    serde_json::to_string(&a.kind).is_ok_and(|s| s.contains(&needle))
}
