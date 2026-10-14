//! Keywords granted to the spells a player casts and to cards in a hand, as one grammar
//! (CR 610.5, 611.3a, 613.1f):
//!
//! * SUBJECT HAS/HAVE KEYWORDS, where the subject is a class of spells you cast ("Each
//!   creature spell you cast", "Colorless spells you cast from your hand with mana value 7
//!   or greater", "The first spell you cast from exile each turn", "The first spell you
//!   cast from anywhere other than your hand each turn"; `spell_cost_grammar::
//!   parse_subject`), or "each [quality] card in your hand [without KEYWORD]";
//! * the keywords: a keyword line, or a quoted one ("\"Cascade, cascade.\"": two instances,
//!   each triggering, CR 702.85c);
//! * an optional sentence about a cost keyword's cost: "The emerge cost is equal to its
//!   mana cost.", "Its foretell cost is equal to its mana cost reduced by {2}."
//!   (`Modification::AddKeywordOwnCost`: the cost of each spell or card's own mana cost,
//!   CR 702.119a, 702.152a, 702.143a, 702.94a);
//! * an optional leading "During your turn, ".
//!
//! Spells have the keywords as they're cast (CR 601.2a; a keyword that's an alternative
//! cost is offered as a way to cast them, `next_spell::cast_grant_keywords`).

use super::StaticPattern;
use crate::ability::*;
use crate::keywords::{Keyword, KeywordKind};
use crate::oracle::phrases::{end, parse_object_phrase};
use crate::oracle::CompileContext;

/// The keywords of "has [keywords]": a keyword line, or one quoted ability that's a
/// keyword line.
fn keywords(s: &str, ctx: &CompileContext) -> Option<Vec<Keyword>> {
    let s = s.trim();
    let s = match s.strip_prefix('"').and_then(|r| r.strip_suffix('"')) {
        Some(q) => q.trim_end_matches('.'),
        None => s,
    };
    let mut out = Vec::new();
    for a in crate::oracle::keywords::parse_keyword_line(s, ctx)? {
        match &a.kind {
            AbilityKind::Keyword(k) => out.push(k.clone()),
            _ => return None,
        }
    }
    (!out.is_empty()).then_some(out)
}

/// Cost keywords whose cost a grant may define as the spell's or card's mana cost.
fn own_cost_kind(k: KeywordKind) -> bool {
    matches!(
        k,
        KeywordKind::Emerge
            | KeywordKind::Blitz
            | KeywordKind::Foretell
            | KeywordKind::Miracle
            | KeywordKind::Evoke
            | KeywordKind::Dash
    )
}

/// "The emerge cost is equal to its mana cost.", "Its foretell cost is equal to its mana
/// cost reduced by {2}.": the reduction, for the keyword `k`.
fn own_cost_sentence(s: &str, k: KeywordKind) -> Option<u32> {
    let name = k.name().to_lowercase();
    let s = end(s.trim());
    let r = s
        .strip_prefix(&format!("the {name} cost is equal to "))
        .or_else(|| s.strip_prefix(&format!("its {name} cost is equal to ")))?;
    let r = r
        .strip_prefix("its mana cost")
        .or_else(|| r.strip_prefix("that spell's mana cost"))
        .or_else(|| r.strip_prefix("that card's mana cost"))?;
    if r.is_empty() {
        return Some(0);
    }
    let n = r.strip_prefix(" reduced by {")?.strip_suffix('}')?;
    n.parse().ok()
}

/// The modifications for the keywords, with the cost sentence if there's one.
fn mods(kws: Vec<Keyword>, cost_sentence: Option<&str>) -> Option<Vec<Modification>> {
    match cost_sentence {
        None => {
            // A cost keyword needs its cost.
            if kws.iter().any(|k| own_cost_kind(k.kind) && k.cost.is_none()) {
                return None;
            }
            Some(kws.into_iter().map(Modification::AddKeyword).collect())
        }
        Some(c) => {
            let [k] = &kws[..] else { return None };
            if k.cost.is_some() || !own_cost_kind(k.kind) {
                return None;
            }
            let reduce = own_cost_sentence(c, k.kind)?;
            Some(vec![Modification::AddKeywordOwnCost(k.clone(), reduce)])
        }
    }
}

/// "each nonland card in your hand [without foretell]": the cards.
fn hand_cards(subject: &str) -> Option<Filter> {
    let r = subject.strip_prefix("each ")?;
    let (head, without) = match r.split_once(" without ") {
        Some((h, w)) => (h.to_string(), Some(w)),
        None => (r.to_string(), None),
    };
    let head = match without {
        // "nonland card in your hand without foretell": the zone after the noun.
        Some(_) => head,
        None => head,
    };
    let (f, plural, tail) = parse_object_phrase(&head)?;
    if plural || !end(tail).trim().is_empty() || f.zone() != Some(ZoneKind::Hand) {
        return None;
    }
    let mut parts = vec![f];
    if let Some(w) = without {
        let kw = Keyword::new(KeywordKind::from_name(w)?);
        parts.push(Filter::not(Filter::HasKeyword(kw.kind)));
    }
    Some(Filter::and(parts))
}

/// `f` with X (`Value::X`) replaced by `v`.
fn with_x(f: &Filter, v: &Value) -> Option<Filter> {
    let json = serde_json::to_string(f).ok()?;
    if !json.contains("\"X\"") {
        return None;
    }
    let v = serde_json::to_string(v).ok()?;
    serde_json::from_str(&json.replace("\"X\"", &v)).ok()
}

fn s_spell_keyword_grant(l: &str, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let l = end(l.trim());
    let (lead_cond, l) = match l.strip_prefix("during your turn, ") {
        Some(r) => (Some(Condition::YourTurn), r),
        None => (None, l),
    };
    let (first, cost_sentence) = match l.split_once(". ") {
        Some((a, b)) => (a, Some(b)),
        None => (l, None),
    };
    // "..., where X is the total amount of life your opponents have lost this turn": X in
    // the subject, determined continuously.
    let (first, x_value) = match first.split_once(", where x is ") {
        Some((a, x)) => {
            let mut b = crate::oracle::effects::Builder::new(ctx);
            let (v, rest) = super::value_grammar::parse_value(x, &mut b)?;
            if !end(&rest).trim().is_empty() || !b.targets.is_empty() {
                return None;
            }
            (a, Some(v))
        }
        None => (first, None),
    };
    // "Elemental permanent spells you cast from your hand gain evoke {4} as you cast
    // them." (they have it as they're cast).
    let gain;
    let first = match first.strip_suffix(" as you cast them") {
        Some(r) => {
            let (subject, granted) = r.split_once(" gain ")?;
            gain = format!("{subject} have {granted}");
            gain.as_str()
        }
        None => first,
    };
    // Cards in a hand.
    if let Some((subject, granted)) = first.split_once(" has ") {
        if let Some(f) = hand_cards(subject) {
            let kws = keywords(granted, ctx)?;
            let mods = mods(kws, cost_sentence)?;
            let mut s = StaticAbility::new(StaticEffect::Continuous { affected: f, mods });
            s.condition = lead_cond;
            return Some(vec![AbilityDef::new(AbilityKind::Static(s), text)]);
        }
    }
    // Spells you cast: the verb after the caster.
    let at = first.find(" you cast")?;
    let after = &first[at..];
    let (verb_at, verb) = [" has ", " have "]
        .iter()
        .filter_map(|v| after.find(v).map(|i| (i, *v)))
        .min_by_key(|x| x.0)?;
    let subject = &first[..at + verb_at];
    let granted = &after[verb_at + verb.len()..];
    let subj = super::spell_cost_grammar::parse_subject(subject)?;
    if subj.who != PlayerRel::You
        || subj.this_turn
        || !matches!(subj.ordinal, None | Some(super::spell_cost_grammar::Ordinal::First))
        || (verb == " has ") != subj.singular
    {
        return None;
    }
    let kws = keywords(granted, ctx)?;
    let mods = mods(kws, cost_sentence)?;
    let mut parts = subj.parts;
    if !parts.iter().any(|f| matches!(f, Filter::Spell)) {
        parts.push(Filter::Spell);
    }
    parts.push(Filter::ControlledBy(PlayerRel::You));
    if subj.ordinal.is_some() {
        parts.push(Filter::Custom(
            crate::kw::first_spell_each_turn::FIRST_THIS_TURN.into(),
        ));
    }
    let mut affected = Filter::and(parts);
    if let Some(v) = &x_value {
        affected = with_x(&affected, v)?;
    }
    if serde_json::to_string(&affected).is_ok_and(|s| s.contains("\"X\"")) {
        return None;
    }
    let mut conds: Vec<Condition> = subj.conds;
    conds.extend(lead_cond);
    let mut s = StaticAbility::new(StaticEffect::Continuous { affected, mods });
    s.condition = match conds.len() {
        0 => None,
        1 => conds.pop(),
        _ => Some(Condition::And(conds)),
    };
    Some(vec![AbilityDef::new(AbilityKind::Static(s), text)])
}

inventory::submit! { StaticPattern { name: "spell keyword grants: [spells you cast] have [keywords]", priority: 130, parse: s_spell_keyword_grant } }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn own_cost_sentences() {
        assert_eq!(
            own_cost_sentence("The emerge cost is equal to its mana cost.", KeywordKind::Emerge),
            Some(0)
        );
        assert_eq!(
            own_cost_sentence(
                "Its foretell cost is equal to its mana cost reduced by {2}.",
                KeywordKind::Foretell
            ),
            Some(2)
        );
        assert_eq!(
            own_cost_sentence("The blitz cost is equal to its mana cost.", KeywordKind::Emerge),
            None
        );
    }
}
