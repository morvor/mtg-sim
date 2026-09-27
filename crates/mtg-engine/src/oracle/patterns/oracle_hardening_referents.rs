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

/// The cards the latest search found (see [`note_introduced`]).
pub const INTRODUCED: Var = vars::USER + 1101;

/// What "it" referred to before a text being parsed recorded found cards, so that it can
/// be restored if nothing referred to them (see [`finish_introduced`]).
#[derive(Default)]
pub struct Introduced(Option<Sel>);

/// After a sentence has been parsed: if it ended by bringing new objects into play —
/// creating tokens ("Create a 1/1 white Ally creature token. Put a +1/+1 counter on it
/// for each ...") or finding cards in a library ("Search your library for a Dinosaur
/// creature card, put it onto the battlefield, then shuffle. It gains indestructible ...")
/// — and "it" had no more specific antecedent than the source (or referred to objects an
/// earlier sentence brought into play: the latest ones are the antecedent), later
/// pronouns refer to those objects (the new objects the instruction recorded, CR 400.7).
///
/// Tokens are the objects the latest token-creating instruction created
/// ([`vars::CREATED`], which only such instructions change). The cards a search found are
/// recorded into [`INTRODUCED`] right after the search (inside the same optional or
/// conditional part): the variable the search itself sets ("the objects affected by the
/// most recent effect", [`vars::IT`]) is overwritten by later instructions ("..., then
/// shuffle. You may behold an Elf. If you do, untap that land.": beholding affects the
/// Elf).
pub fn note_introduced(e: &mut Effect, b: &mut Builder, intro: &mut Introduced) {
    let introduced = matches!(b.it, Sel::Var(v) if v == vars::CREATED || v == INTRODUCED);
    if !(matches!(b.it, Sel::This) || is_no_referent(&b.it) || introduced) {
        return;
    }
    let mut creates = 0;
    count_creates(e, &mut creates);
    let last = last_instruction_mut(e);
    match last {
        Effect::CreateToken { .. } | Effect::CreateTokenCopy { .. } if creates == 1 => {
            b.it = Sel::Var(vars::CREATED);
        }
        Effect::Search { .. } => {
            let search = std::mem::replace(last, Effect::Noop);
            *last = Effect::Seq(vec![
                search,
                Effect::Store {
                    var: INTRODUCED,
                    sel: Sel::Var(vars::IT),
                },
            ]);
            intro.0.get_or_insert_with(|| b.it.clone());
            b.it = Sel::Var(INTRODUCED);
        }
        _ => {}
    }
}

/// At the end of a text: keeps the [`INTRODUCED`] stores if something refers to the
/// found cards, or removes them and restores "it" otherwise (so the compiled
/// ability is unchanged for texts without such a pronoun).
pub fn finish_introduced(e: Effect, b: &mut Builder, intro: Introduced) -> Effect {
    let Some(before) = intro.0 else {
        return e;
    };
    let mentioned = serde_json::to_string(&e)
        .is_ok_and(|s| s.contains(&format!("{{\"Var\":{INTRODUCED}}}")));
    if mentioned {
        return e;
    }
    abandon_introduced(b, Introduced(Some(before)));
    strip_introduced(e)
}

/// A text failed to parse: "it" no longer refers to cards it found.
pub fn abandon_introduced(b: &mut Builder, intro: Introduced) {
    if let Some(before) = intro.0 {
        if matches!(b.it, Sel::Var(v) if v == INTRODUCED) {
            b.it = before;
        }
    }
}

fn is_introduced_store(e: &Effect) -> bool {
    matches!(e, Effect::Store { var, .. } if *var == INTRODUCED)
}

/// Removes the [`INTRODUCED`] stores from an effect.
fn strip_introduced(e: Effect) -> Effect {
    match e {
        Effect::Seq(v) => Effect::seq(
            v.into_iter()
                .filter(|x| !is_introduced_store(x))
                .map(strip_introduced)
                .collect(),
        ),
        Effect::If {
            cond,
            then,
            otherwise,
        } => Effect::If {
            cond,
            then: Box::new(strip_introduced(*then)),
            otherwise: Box::new(strip_introduced(*otherwise)),
        },
        Effect::May { who, effect } => Effect::May {
            who,
            effect: Box::new(strip_introduced(*effect)),
        },
        e => e,
    }
}

/// After an instruction that named a player as "its owner" or "its controller" ("Return
/// target permanent to its owner's hand, then that player discards a card.", "Counter
/// target spell unless its controller pays {1}. That player discards a card."): if
/// "that player" had no antecedent yet, it's that player now.
pub fn note_player_mention(text: &str, b: &mut Builder) {
    if !is_no_player_referent(&b.it_player) || is_no_referent(&b.it) {
        return;
    }
    let l = text.to_lowercase();
    let owner = l.contains("its owner");
    let controller = l.contains("its controller");
    // Neither, or both (ambiguous).
    if owner == controller {
        return;
    }
    let it = Box::new(b.it.clone());
    b.it_player = if owner {
        PlayerRef::OwnerOf(it)
    } else {
        PlayerRef::ControllerOf(it)
    };
}

/// The instruction an effect ends with (looking into sequences and optional parts).
fn last_instruction_mut(e: &mut Effect) -> &mut Effect {
    let descend = match &*e {
        Effect::Seq(v) => !v.is_empty(),
        Effect::May { .. } => true,
        Effect::If { otherwise, .. } => matches!(**otherwise, Effect::Noop),
        _ => false,
    };
    if !descend {
        return e;
    }
    match e {
        Effect::Seq(v) => {
            let n = v.len();
            last_instruction_mut(&mut v[n - 1])
        }
        Effect::May { effect, .. } => last_instruction_mut(effect),
        Effect::If { then, .. } => last_instruction_mut(then),
        other => other,
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
