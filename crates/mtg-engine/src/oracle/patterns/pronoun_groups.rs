//! Plural pronouns after an instruction that affected a group of objects: "Untap all
//! creatures you control. They gain hexproof and indestructible until end of turn.",
//! "Gain control of all creatures target opponent controls until end of turn. Untap those
//! creatures.", "Put a +1/+1 counter on each creature you control. They gain vigilance
//! until end of turn."
//!
//! "They" / "those creatures" / "each of them" are the objects the earlier instruction
//! affected, not whatever matches its description later: after gaining control of the
//! creatures target opponent controls, that player controls none of them. The members
//! are recorded as the instruction begins (the same set the instruction itself locks in,
//! CR 608.2c, 611.2c) into [`GROUP`], and the pronouns read that variable.
//!
//! The effect parser calls [`note`] after each instruction (a sentence, or either half
//! of "X and Y"): if the instruction affected a group, it puts an [`Effect::Store`] just
//! before it and makes the builder's `it` the variable. An instruction that might not
//! happen ("If this spell was kicked, untap all Forests ... They become 3/3 ...", "you
//! may ...") records the group only if it does. [`finish`] drops the stores again if
//! nothing referred to the group, so the compiled ability is unchanged for texts without
//! such a pronoun.
//!
//! Also here, for the effect parser's pronoun handling:
//! - plural pronouns never refer to the source itself or to a single object a trigger is
//!   about ([`plural_referent`]); singular "it" after a group keeps its earlier referent
//!   ([`singular_it`]);
//! - "those creatures" after "whenever you cast a spell that targets one or more
//!   creatures" are the spell's targets ([`Filter::TargetOf`]);
//! - "they" in a "one or more (objects) attack/become blocked/..." trigger are the
//!   batch's objects ([`batch_referent`], [`Sel::TriggerObjects`]);
//! - "~ deals 1 damage to them" in a trigger about a player is that player
//!   ([`them_player`]);
//! - "creatures blocking it" where "it" isn't the source ([`blocking_it`]).

use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::CompileContext;

/// The members of the group the latest group instruction affected.
pub const GROUP: Var = vars::USER + 1100;

/// The group a builder's pronouns currently refer to.
#[derive(Clone, Debug)]
pub struct GroupRef {
    /// The group as the instruction described it ("all creatures you control").
    pub sel: Sel,
    /// What "it" referred to before the group instruction.
    pub it_before: Sel,
}

/// The group an instruction affected without moving it to another zone: the objects of
/// a tap, untap, continuous effect, control change, counters, goad, or removal from
/// combat described as "all/each (objects)" or "(objects) you control". (Zone changes
/// make new objects, CR 400.7; the parsers for those record the new objects themselves.)
pub fn affected_group(e: &Effect) -> Option<&Sel> {
    let what = match e {
        Effect::Seq(v) => return v.last().and_then(affected_group),
        // "If this spell was kicked, untap all Forests put onto the battlefield this way.
        // They become 3/3 green creatures with haste that are still lands.": the group,
        // if the instruction happened (see [`place_store`]).
        Effect::If {
            then, otherwise, ..
        } if matches!(**otherwise, Effect::Noop) => return affected_group(then),
        Effect::May { effect, .. } => return affected_group(effect),
        Effect::Tap { what }
        | Effect::Untap { what }
        | Effect::Modify { what, .. }
        | Effect::GainControl { what, .. }
        | Effect::AddCounters { what, .. }
        | Effect::RemoveCounters { what, .. }
        | Effect::RemoveFromCombat { what }
        | Effect::KeywordAction {
            action: KeywordAction::Goad,
            what,
            ..
        } => what,
        // "~ deals 1 damage to each creature with flying your opponents control. Tap
        // those creatures."
        Effect::DealDamage { to, .. } => to,
        // "Double the power of each other creature you control until end of turn. Those
        // creatures gain vigilance until end of turn."
        Effect::ForEach { sel, var, effect } if acts_on_var(effect, *var) => sel,
        _ => return None,
    };
    matches!(what, Sel::All(_)).then_some(what)
}

/// Whether a "for each" body just changes the object it's about (without moving it).
fn acts_on_var(e: &Effect, var: Var) -> bool {
    let what = match e {
        Effect::Tap { what }
        | Effect::Untap { what }
        | Effect::Modify { what, .. }
        | Effect::AddCounters { what, .. }
        | Effect::RemoveCounters { what, .. } => what,
        _ => return false,
    };
    matches!(what, Sel::Var(v) if *v == var)
}

fn is_group_store(e: &Effect) -> bool {
    matches!(e, Effect::Store { var, .. } if *var == GROUP)
}

/// After an instruction has been parsed: if it affected a group, later plural pronouns
/// refer to the group's members. Records the members just before the group instruction
/// (inside `e` if that's part of it) and returns what to execute before `e`, if anything.
pub fn note(e: &mut Effect, b: &mut Builder) -> Option<Effect> {
    // Already recorded by the clause parser ("untap all creatures and gain control of
    // them").
    if let Effect::Seq(v) = &*e {
        if v.iter().any(is_group_store) {
            return None;
        }
    }
    let sel = affected_group(e)?.clone();
    let it_before = std::mem::replace(&mut b.it, Sel::Var(GROUP));
    // A second group in the same text: "it" before it is still what it was before the
    // first one.
    let it_before = match &b.group {
        Some(g) if matches!(it_before, Sel::Var(GROUP)) => g.it_before.clone(),
        _ => it_before,
    };
    b.group = Some(GroupRef {
        sel: sel.clone(),
        it_before,
    });
    let store = Effect::Store { var: GROUP, sel };
    match place_store(e, &store) {
        None => Some(store),
        Some(false) => None,
        // The instruction might not happen: until it does, "they" are nothing (not the
        // members of an earlier group).
        Some(true) => Some(Effect::Store {
            var: GROUP,
            sel: Sel::None,
        }),
    }
}

/// Puts `store` just before the group instruction [`affected_group`] found in `e`, when
/// that's part of `e`: `None` if `e` is the instruction itself (the store goes before
/// it), otherwise whether the instruction is conditional. "If this spell was kicked,
/// untap all Forests put onto the battlefield this way. They become 3/3 ...": they become
/// creatures only if the spell was kicked, so the group is recorded only then.
fn place_store(e: &mut Effect, store: &Effect) -> Option<bool> {
    let branch = match e {
        Effect::Seq(v) => {
            let n = v.len();
            return match place_store(v.last_mut()?, store) {
                None => {
                    v.insert(n - 1, store.clone());
                    Some(false)
                }
                r => r,
            };
        }
        Effect::If {
            then, otherwise, ..
        } if matches!(**otherwise, Effect::Noop) => then,
        Effect::May { effect, .. } => effect,
        _ => return None,
    };
    if place_store(branch, store).is_none() {
        let inner = std::mem::replace(&mut **branch, Effect::Noop);
        **branch = Effect::seq(vec![store.clone(), inner]);
    }
    Some(true)
}

/// A text failed to parse: forgets the groups it recorded, as [`finish`] does.
pub fn abandon(b: &mut Builder, outer: Option<GroupRef>) {
    if let Some(g) = b.group.take() {
        if matches!(b.it, Sel::Var(GROUP)) {
            b.it = g.it_before;
        }
    }
    b.group = outer;
}

/// Whether an effect reads the group variable.
pub fn mentions_group(e: &Effect) -> bool {
    serde_json::to_string(e).is_ok_and(|s| s.contains(&format!("{{\"Var\":{GROUP}}}")))
}

/// At the end of a text: keeps the group stores if something refers to the group, or
/// removes them and restores "it" otherwise. `outer` is the group of an enclosing text
/// being parsed with the same builder.
pub fn finish(e: Effect, b: &mut Builder, outer: Option<GroupRef>) -> Effect {
    let Some(g) = b.group.take() else {
        b.group = outer;
        return e;
    };
    if mentions_group(&e) {
        b.group = Some(g);
        return e;
    }
    if matches!(b.it, Sel::Var(GROUP)) {
        b.it = g.it_before;
    }
    b.group = outer;
    strip_group_stores(e)
}

/// Removes the group stores from an effect.
fn strip_group_stores(e: Effect) -> Effect {
    match e {
        Effect::Seq(v) if v.iter().any(is_group_store) => Effect::seq(
            v.into_iter()
                .filter(|x| !is_group_store(x))
                .map(strip_group_stores)
                .collect(),
        ),
        Effect::Seq(v) => Effect::Seq(v.into_iter().map(strip_group_stores).collect()),
        Effect::If {
            cond,
            then,
            otherwise,
        } => Effect::If {
            cond,
            then: Box::new(strip_group_stores(*then)),
            otherwise: Box::new(strip_group_stores(*otherwise)),
        },
        Effect::May { who, effect } => Effect::May {
            who,
            effect: Box::new(strip_group_stores(*effect)),
        },
        Effect::PayOptional {
            who,
            cost,
            then,
            otherwise,
        } => Effect::PayOptional {
            who,
            cost,
            then: Box::new(strip_group_stores(*then)),
            otherwise: Box::new(strip_group_stores(*otherwise)),
        },
        e if is_group_store(&e) => Effect::Noop,
        e => e,
    }
}

/// Whether a plural pronoun ("they", "them", "those creatures") can refer to `sel`: not
/// to the source itself (unless the card is a pair, "Whenever Tui and La become untapped,
/// put a +1/+1 counter on them"), the permanent it's attached to, a single object or spell
/// a trigger is about, or nothing.
pub fn plural_referent(sel: &Sel, ctx: &CompileContext) -> bool {
    match sel {
        Sel::This => plural_self(ctx),
        Sel::None
        | Sel::AttachedTo
        | Sel::TriggerObject
        | Sel::TriggerLki
        | Sel::TriggerOtherObject
        | Sel::TriggerPlayer
        | Sel::TriggerSpell => false,
        _ => true,
    }
}

/// Whether the card names a pair of characters that its text treats as plural ("Tui and
/// La become untapped", "Cloak and Dagger enter").
fn plural_self(ctx: &CompileContext) -> bool {
    let short = ctx.card_name.split(',').next().unwrap_or_default();
    if !short.contains(" and ") {
        return false;
    }
    let text = crate::oracle::raw_text();
    [
        "become", "enter", "attack", "block", "deal", "leave", "die", "have", "are", "get",
        "gain",
    ]
    .iter()
    .any(|v| text.contains(&format!("{short} {v} ")))
}

/// A plural object pronoun at the start of `s` ("they", "they each", "them", "those
/// creatures", "each of them", "each of those permanents"); returns the rest.
pub fn plural_pronoun(s: &str) -> Option<&str> {
    const NOUNS: [&str; 6] = [
        "creatures",
        "permanents",
        "lands",
        "artifacts",
        "tokens",
        "cards",
    ];
    let word_end = |r: &str| r.is_empty() || r.starts_with(' ') || r.starts_with('\'');
    for p in ["they each", "each of them", "them", "they"] {
        if let Some(r) = s.strip_prefix(p) {
            if word_end(r) {
                return Some(r);
            }
        }
    }
    let r = s.strip_prefix("each of those ").or_else(|| s.strip_prefix("those "))?;
    NOUNS
        .iter()
        .find_map(|n| r.strip_prefix(n).filter(|r| word_end(r)))
}

/// What a singular pronoun ("it", "that creature") refers to: what it referred to before
/// a group instruction ("~ deals 1 damage to each creature. If it was kicked, it deals 2
/// damage to each creature instead." — "it" is still ~).
pub fn singular_it(b: &Builder) -> Sel {
    match (&b.it, &b.group) {
        (Sel::Var(v), Some(g)) if *v == GROUP => g.it_before.clone(),
        // The objects of a "one or more" trigger are "they", never "it".
        (Sel::TriggerObjects, _) => Sel::None,
        (it, _) => it.clone(),
    }
}

/// What "they" refers to in a "one or more (objects) (event)" trigger: the objects of the
/// batch, for events after which they're still the same permanents (attacking, becoming
/// blocked, tapped or untapped, entering). Objects that left the battlefield would need
/// their last known information, so those batches have no such referent.
pub fn batch_referent(c: &TriggerCond) -> Sel {
    match c {
        TriggerCond::Attacks(_)
        | TriggerCond::BecomesBlocked(_)
        | TriggerCond::BecomesTapped(_)
        | TriggerCond::BecomesUntapped(_)
        | TriggerCond::EntersBattlefield(_) => Sel::TriggerObjects,
        TriggerCond::Where { trigger, .. } | TriggerCond::FirstTimeEachTurn(trigger) => {
            batch_referent(trigger)
        }
        _ => Sel::None,
    }
}

/// A plural object pronoun at the start of `s`: `None` if there's none, `Some(None)` if
/// it has nothing it could refer to, otherwise its referent and the rest of the text.
#[allow(clippy::option_option)]
pub fn plural_object_ref(s: &str, b: &Builder) -> Option<Option<(Sel, String)>> {
    let rest = plural_pronoun(s)?;
    // "Whenever you cast a spell that targets one or more creatures, those creatures gain
    // flying until end of turn.": the objects the spell targets.
    if matches!(b.it, Sel::TriggerSpell) {
        let Some(noun) = s.strip_prefix("those ") else {
            return Some(None);
        };
        let noun = &noun[..noun.len() - rest.len()];
        // Only the objects the trigger names without qualification: "those creatures"
        // after "targets one or more creatures you control" would be only the targets
        // you control.
        let named = format!("targets one or more {noun},");
        if !crate::oracle::raw_text().to_lowercase().contains(&named) {
            return Some(None);
        }
        let f = match noun {
            "creatures" => Filter::creature(),
            "permanents" => Filter::Permanent,
            _ => return Some(None),
        };
        return Some(Some((
            Sel::All(Filter::and(vec![
                f,
                Filter::TargetOf(Box::new(Sel::TriggerSpell)),
            ])),
            rest.to_string(),
        )));
    }
    if !plural_referent(&b.it, b.ctx) {
        return Some(None);
    }
    Some(Some((b.it.clone(), rest.to_string())))
}

/// "them" as a damage recipient that is a player: "Whenever an opponent draws a card, ~
/// deals 1 damage to them." (singular "they" for the player the ability is about, when
/// no group or objects were mentioned).
pub fn them_player(s: &str, b: &Builder) -> Option<PlayerRef> {
    let r = s.strip_prefix("them")?;
    if !(r.is_empty() || r.starts_with(' ') || r.starts_with(',')) {
        return None;
    }
    if plural_referent(&b.it, b.ctx) || matches!(b.it, Sel::TriggerSpell) {
        return None;
    }
    match &b.it_player {
        PlayerRef::You => None,
        p => Some(p.clone()),
    }
}

/// "(creature) blocking it" / "(creature) that's blocking it" after an object phrase,
/// where "it" is the object the ability is about: the source ("Whenever ~ becomes
/// blocked, it deals 1 damage to each creature blocking it"), the creature a trigger is
/// about ("Whenever a Beast becomes blocked, it gets +1/+1 until end of turn for each
/// creature blocking it"), the permanent the source is attached to, or a single target.
/// Returns the filter narrowed to such blockers and the rest of the text.
pub fn blocking_it(f: Filter, rest: &str, b: &Builder) -> Option<(Filter, String)> {
    let r = rest.trim_start();
    let r = r.strip_prefix("that's ").unwrap_or(r);
    let r = r.strip_prefix("blocking it")?;
    if !(r.is_empty() || r.starts_with(' ') || r.starts_with(',')) {
        return None;
    }
    // Once "it" has left the battlefield (sacrificed as a cost or by an earlier
    // instruction: "{B}, Sacrifice ~: Destroy target creature blocking it."), the creatures
    // that were blocking it are known only from its last known information, which combat
    // doesn't keep.
    if crate::oracle::raw_text().to_lowercase().contains("sacrifice") {
        return None;
    }
    let blocking = match singular_it(b) {
        Sel::This => Filter::BlockingSource,
        sel @ (Sel::TriggerObject | Sel::AttachedTo) => Filter::BlockingAnyOf(Box::new(sel)),
        Sel::Target(slot)
            if b.targets.get(slot as usize).is_some_and(|t| {
                t.max.as_const() == Some(1) && matches!(t.what, TargetKind::Object(_))
            }) =>
        {
            Filter::BlockingAnyOf(Box::new(Sel::Target(slot)))
        }
        _ => return None,
    };
    Some((Filter::and(vec![f, blocking]), r.to_string()))
}
