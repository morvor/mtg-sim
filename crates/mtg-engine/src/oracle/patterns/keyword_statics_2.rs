//! Keyword statics II (CR 702.1c, 613.1f):
//!
//! - "~ has flying as long as a card exiled with it has flying. The same is true for first
//!   strike, ..." (Urborg Scavengers), "As long as a card exiled with ~ has flying, equipped
//!   creature has flying. The same is true for ..." (Eater of Virtue, Death-Mask
//!   Duplicant), "As long as an exiled creature card with a blood counter on it has flying,
//!   ..." (Rayami), "... an exiled card used to craft it has flying" (Altar of the
//!   Wretched): each listed keyword, in every variant such a card has.
//! - "Each creature you control has vigilance if it's white, hexproof if it's blue, ..."
//!   (Scion of Draco): one keyword per quality of the affected object.
//! - "For each basic land type among lands you control, ~ has landwalk of that type."
//!   (Magnigoth Treefolk): one landwalk per basic land type you control.

use super::AbilityPattern;
use crate::ability::*;
use crate::keywords::{Keyword, KeywordKind};
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;
use crate::oracle::effects::Builder;
use crate::types::*;

/// The keyword kinds of "flying" plus "first strike, double strike, ..., and trample".
fn kind_list(first: &str, list: &str) -> Option<Vec<KeywordKind>> {
    let mut kinds = vec![KeywordKind::from_name(first)?];
    for item in list
        .split(", and ")
        .flat_map(|p| p.split(", "))
        .flat_map(|p| p.split(" and "))
        .map(str::trim)
        .filter(|p| !p.is_empty())
    {
        kinds.push(KeywordKind::from_name(item)?);
    }
    Some(kinds)
}

/// The cards a keyword is copied from: "a card exiled with ~" / "a card exiled with it"
/// (cards exiled by the source's linked abilities, CR 607.2a), "an exiled creature card
/// with a blood counter on it", "an exiled card used to craft it" (CR 702.167c).
fn exiled_holders(s: &str) -> Option<Filter> {
    let s = s
        .strip_prefix("a ")
        .or_else(|| s.strip_prefix("an "))
        .unwrap_or(s);
    let linked = |f: Filter| {
        Filter::and(vec![
            f,
            Filter::InZone(ZoneKind::Exile),
            Filter::In(Box::new(Sel::Linked)),
        ])
    };
    for suffix in [" exiled with ~", " exiled with it"] {
        if let Some(r) = s.strip_suffix(suffix) {
            let (f, _, tail) = parse_object_phrase(r)?;
            return end(tail).is_empty().then(|| linked(f));
        }
    }
    let r = s.strip_prefix("exiled ")?;
    if let Some(r) = r.strip_suffix(" used to craft it") {
        let (f, _, tail) = parse_object_phrase(r)?;
        return end(tail)
            .is_empty()
            .then(|| Filter::and(vec![f, crate::kw::craft::used_to_craft_filter()]));
    }
    let (f, _, tail) = parse_object_phrase(r)?;
    end(tail)
        .is_empty()
        .then(|| Filter::and(vec![f, Filter::InZone(ZoneKind::Exile)]))
}

/// "[subject] has [kw] as long as [cards] has [kw]" / "As long as [cards] has [kw],
/// [subject] has [kw]", followed by ". The same is true for [keywords]".
fn same_is_true_for_exiled(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if ctx.is_spell() || block.contains('\n') || block.contains(':') {
        return None;
    }
    let lower = block.to_lowercase();
    let l = end(&lower);
    let (first, list) = l.split_once(". the same is true for ")?;
    let (subject, kw, holder) = match first.strip_prefix("as long as ") {
        Some(r) => {
            let (cond, main) = r.split_once(", ")?;
            let (subject, kw) = main.split_once(" has ")?;
            let holder = cond
                .strip_suffix(&format!(" has {kw}"))
                .unwrap_or(cond);
            (subject, kw, holder)
        }
        None => {
            let (main, cond) = first.split_once(" as long as ")?;
            let (subject, kw) = main.split_once(" has ")?;
            let holder = cond.strip_suffix(&format!(" has {kw}"))?;
            (subject, kw, holder)
        }
    };
    let affected = super::k702_001_010::static_subject(subject)?;
    // "protection from any color": each color's protection, separately.
    let (list, any_color) = match list
        .strip_suffix(", and protection from any color")
        .or_else(|| list.strip_suffix(" and protection from any color"))
    {
        Some(r) => (r, true),
        None => (list, false),
    };
    let kinds = kind_list(kw, list)?;
    let from = exiled_holders(holder).or_else(|| opponents_creatures_with(holder, kw))?;
    let mut out = vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Continuous {
            affected: affected.clone(),
            mods: vec![Modification::AddKeywordsOf {
                kinds,
                from: from.clone(),
            }],
        })),
        block,
    )];
    if any_color {
        for c in Color::ALL {
            let mut k = Keyword::new(KeywordKind::Protection);
            k.filter = Some(Filter::Color(c));
            k.text = Some(format!("protection from {}", c.word()).into());
            let mut st = StaticAbility::new(StaticEffect::Continuous {
                affected: affected.clone(),
                mods: vec![Modification::AddKeyword(k)],
            });
            st.condition = Some(Condition::Exists(Filter::and(vec![
                from.clone(),
                crate::kw::protection_from_color_filter::filter(c),
            ])));
            out.push(AbilityDef::new(AbilityKind::Static(st), block));
        }
    }
    Some(out)
}

/// "an opponent controls a creature with flying not named ~" (Escaped Shapeshifter): the
/// creatures whose keywords count, without the keyword itself.
fn opponents_creatures_with(holder: &str, kw: &str) -> Option<Filter> {
    let r = holder.strip_prefix("an opponent controls a ")?;
    let r = r.replace(&format!(" with {kw}"), "");
    let (r, not_named) = match r.strip_suffix(" not named ~") {
        Some(x) => (x.to_string(), true),
        None => (r, false),
    };
    let (f, plural, tail) = parse_object_phrase(&r)?;
    if plural || !end(tail).is_empty() {
        return None;
    }
    let mut parts = vec![
        f,
        Filter::ControlledBy(PlayerRel::Opponent),
        Filter::InZone(ZoneKind::Battlefield),
    ];
    if not_named {
        parts.push(Filter::not(Filter::SameNameAs(Box::new(Sel::This))));
    }
    Some(Filter::and(parts))
}

inventory::submit! { AbilityPattern { name: "keyword statics: [subject] has [kw] as long as [an exiled card] has [kw]. the same is true for ...", priority: 49, parse: same_is_true_for_exiled } }

/// A keyword without parameters ("vigilance", "first strike").
fn plain_keyword(s: &str) -> Option<Keyword> {
    let tl = TypeLine::default();
    let ctx = CompileContext {
        card_name: "",
        full_name: "",
        type_line: &tl,
        layout: crate::card::Layout::Normal,
        face_index: 0,
        keywords: &[],
        power: None,
        toughness: None,
    };
    let abilities = crate::oracle::keywords::parse_keyword_line(s, &ctx)?;
    match abilities.as_slice() {
        [a] => match &a.kind {
            AbilityKind::Keyword(k) => Some(k.clone()),
            _ => None,
        },
        _ => None,
    }
}

/// "Each creature you control has vigilance if it's white, hexproof if it's blue, ...,
/// and trample if it's green." (Scion of Draco): for each color, the objects of that
/// color have that keyword.
fn keyword_per_color(l: &str, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if ctx.is_spell() {
        return None;
    }
    let l = end(l);
    let (subject, list) = l.split_once(" has ").or_else(|| l.split_once(" have "))?;
    let subject = subject.strip_prefix("each ").unwrap_or(subject);
    let (f, _, tail) = parse_object_phrase(subject)?;
    if !end(tail).is_empty() {
        return None;
    }
    let mut out = Vec::new();
    for item in list
        .split(", and ")
        .flat_map(|p| p.split(", "))
        .map(str::trim)
    {
        let (kw, color) = item
            .split_once(" if it's ")
            .or_else(|| item.split_once(" if they're "))?;
        let color = Color::from_word(color)?;
        let k = plain_keyword(kw)?;
        out.push(AbilityDef::new(
            AbilityKind::Static(StaticAbility::new(StaticEffect::Continuous {
                affected: Filter::and(vec![f.clone(), Filter::Color(color)]),
                mods: vec![Modification::AddKeyword(k)],
            })),
            text,
        ));
    }
    (out.len() >= 2).then_some(out)
}

inventory::submit! { super::StaticPattern { name: "keyword statics: [objects] have [kw] if it's [color], [kw] if it's [color], ...", priority: 1090, parse: keyword_per_color } }

/// "For each basic land type among lands you control, ~ has landwalk of that type."
/// (Magnigoth Treefolk): plainswalk as long as you control a Plains, and so on.
fn landwalk_per_basic_type(l: &str, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if ctx.is_spell() {
        return None;
    }
    let rest = end(l).strip_prefix("for each basic land type among lands you control, ")?;
    let subject = rest.strip_suffix(" has landwalk of that type")?;
    let affected = super::k702_001_010::static_subject(subject)?;
    let mut out = Vec::new();
    for t in ["Plains", "Island", "Swamp", "Mountain", "Forest"] {
        let walk = format!("{}walk", t.to_lowercase());
        let k = plain_keyword(&walk)?;
        let mut st = StaticAbility::new(StaticEffect::Continuous {
            affected: affected.clone(),
            mods: vec![Modification::AddKeyword(k)],
        });
        st.condition = Some(Condition::Exists(Filter::and(vec![
            Filter::Type(CardType::Land),
            Filter::Subtype(Subtype::from(t)),
            Filter::ControlledBy(PlayerRel::You),
            Filter::InZone(ZoneKind::Battlefield),
        ])));
        out.push(AbilityDef::new(AbilityKind::Static(st), text));
    }
    Some(out)
}

inventory::submit! { super::StaticPattern { name: "keyword statics: for each basic land type among lands you control, ~ has landwalk of that type", priority: 1090, parse: landwalk_per_basic_type } }

/// One condition of a conditional predicate: "if it's red" (the object the sentence is
/// about, `what`), "if you control a Dwarf".
fn predicate_condition(c: &str, what: &Sel, b: &Builder) -> Option<Condition> {
    if let Some(k) = c.strip_prefix("it's ") {
        if let Some(col) = Color::from_word(k) {
            return Some(Condition::SelMatches(what.clone(), Filter::Color(col)));
        }
        let k = k.strip_prefix("a ").or_else(|| k.strip_prefix("an "))?;
        let (f, plural, tail) = parse_object_phrase(k)?;
        return (!plural && end(tail).trim().is_empty())
            .then(|| Condition::SelMatches(what.clone(), f));
    }
    if c.starts_with("you control ") {
        return crate::oracle::statics::parse_condition(c, b.ctx);
    }
    None
}

/// Splits "trample if you control a dwarf and hexproof if you control an elf" / "from
/// white if you control a plains, from blue if ..., and from green if ..." into its
/// conditional items.
fn conditional_items(s: &str) -> Option<Vec<(&str, &str)>> {
    let mut out = Vec::new();
    let mut rest = s;
    loop {
        let (item, after) = rest.split_once(" if ")?;
        // The condition runs to the next ", and ", ", " or " and " that starts an item
        // with its own condition.
        let mut cut = None;
        for sep in [", and ", ", ", " and "] {
            for (i, _) in after.match_indices(sep) {
                let next = &after[i + sep.len()..];
                if next.contains(" if ") && cut.is_none_or(|(j, _): (usize, usize)| i < j) {
                    cut = Some((i, sep.len()));
                }
            }
        }
        match cut {
            Some((i, n)) => {
                out.push((item, &after[..i]));
                rest = &after[i + n..];
            }
            None => {
                out.push((item, after));
                break;
            }
        }
    }
    (out.len() >= 2 || !out.is_empty()).then_some(out)
}

/// "Until end of turn, that creature gets +2/+0, gains trample if it's red, and gains
/// vigilance if it's white." (Aurelia), "Until your next turn, it gains trample if you
/// control a Dwarf and hexproof if you control an Elf." (Doors of Durin), "Until end of
/// turn, creatures you control gain protection from white if you control a Plains, from
/// blue if you control an Island, ..." (Dominaria's Judgment): each conditional predicate
/// applies if its condition holds as the effect is created.
fn conditional_predicates(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    if !l.contains(" if ") || l.contains('"') {
        return None;
    }
    let (lead, core) = if let Some(r) = l.strip_prefix("until end of turn, ") {
        ("until end of turn, ", r)
    } else if let Some(r) = l.strip_prefix("until your next turn, ") {
        ("until your next turn, ", r)
    } else {
        ("", l)
    };
    let i = [" gets ", " get ", " gains ", " gain "]
        .iter()
        .filter_map(|v| core.find(v))
        .min()?;
    let subj_text = &core[..i];
    let preds = super::becomes_grammar::split_predicates(&core[i + 1..]);
    let saved = (b.targets.len(), b.it.clone());
    let result = (|| {
        let (what, r) = crate::oracle::effects::object_ref(subj_text, b)?;
        if !r.trim().is_empty() || matches!(what, Sel::None) {
            return None;
        }
        let after_subject = b.targets.len();
        let it = if matches!(what, Sel::All(_)) {
            subj_text.to_string()
        } else {
            "it".to_string()
        };
        let mut out = Vec::new();
        let mut conditional = 0;
        for p in preds {
            let (verb, body) = p.split_once(' ')?;
            if !p.contains(" if ") {
                b.it = what.clone();
                out.push(crate::oracle::effects::parse_clause(
                    &format!("{lead}{it} {p}"),
                    b,
                )?);
                continue;
            }
            if !matches!(verb, "gains" | "gain") {
                return None;
            }
            let items = conditional_items(body)?;
            let mut prefix = "";
            for (item, c) in items {
                // "protection from white if ..., from blue if ...": the later items
                // continue the first one's keyword.
                let item = if item.starts_with("protection from ") {
                    prefix = "protection ";
                    item.to_string()
                } else if item.starts_with("from ") && !prefix.is_empty() {
                    format!("{prefix}{item}")
                } else {
                    item.to_string()
                };
                let cond = predicate_condition(c, &what, b)?;
                b.it = what.clone();
                let e = crate::oracle::effects::parse_clause(
                    &format!("{lead}{it} {verb} {item}"),
                    b,
                )?;
                if !matches!(e, Effect::Modify { .. }) {
                    return None;
                }
                conditional += 1;
                out.push(Effect::If {
                    cond,
                    then: Box::new(e),
                    otherwise: Box::new(Effect::Noop),
                });
            }
        }
        if conditional == 0 || b.targets.len() != after_subject {
            return None;
        }
        b.it = what;
        Some(Effect::seq(out))
    })();
    if result.is_none() {
        b.targets.truncate(saved.0);
        b.it = saved.1;
    }
    result
}

inventory::submit! { super::EffectPattern { name: "keyword statics: [subject] gains [keyword] if [condition] and [keyword] if [condition]", priority: 1090, parse: conditional_predicates } }

/// "[static ability with a condition]. Otherwise, [static ability]." (Dog Umbra), "[anthem].
/// It gets [P/T] instead as long as [condition]." (So Tiny), "Equipped creature gets
/// +1/+1. If it's a Warrior, it gets +2/+1 instead." (Relic Axe): the second ability
/// applies when the first one's condition doesn't hold, or replaces it when its own does.
fn otherwise_statics(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if ctx.is_spell() || block.contains('\n') || block.contains(':') {
        return None;
    }
    let lower = block.to_lowercase();
    let l = end(&lower);
    let single = |s: &str| -> Option<StaticAbility> {
        let v = crate::oracle::statics::parse_static(s, ctx)?;
        match v.as_slice() {
            [a] => match &a.kind {
                AbilityKind::Static(st) => Some(st.clone()),
                _ => None,
            },
            _ => None,
        }
    };
    let ability = |st: StaticAbility| AbilityDef::new(AbilityKind::Static(st), block);
    if let Some(i) = l.find(". otherwise, ") {
        let first = &block[..i + 1];
        let second = &block[i + ". otherwise, ".len()..];
        // A pronoun refers to the first sentence's subject: left to `statics_sentences`.
        if ["it ", "they ", "its "].iter().any(|p| second.to_lowercase().starts_with(p)) {
            return None;
        }
        let a = single(first)?;
        let cond = a.condition.clone()?;
        let mut b = single(&capitalize(second))?;
        if b.condition.is_some() {
            return None;
        }
        b.condition = Some(Condition::Not(Box::new(cond)));
        return Some(vec![ability(a), ability(b)]);
    }
    // "[anthem]. It gets [P/T] instead as long as [condition]." / "[anthem]. If it's a
    // [kind], it gets [P/T] instead."
    let (first, second) = l.split_once(". ")?;
    let (pt, cond_text, kind) = if let Some(r) = second.strip_prefix("it gets ") {
        let (pt, c) = r.split_once(" instead as long as ")?;
        (pt, c, None)
    } else {
        let r = second.strip_prefix("if it's ")?;
        let (k, r) = r.split_once(", it gets ")?;
        let pt = r.strip_suffix(" instead")?;
        (pt, "", Some(k))
    };
    let mut a = single(&block[..first.len() + 1])?;
    if a.condition.is_some() {
        return None;
    }
    let StaticEffect::Continuous { affected, mods } = &a.effect else {
        return None;
    };
    if !matches!(mods.as_slice(), [Modification::ModifyPT(..)]) || !matches!(affected, Filter::AttachedToSource) {
        return None;
    }
    let (p, t, rest) = crate::oracle::effects::parse_pt_mod(pt)?;
    if !rest.trim().is_empty() {
        return None;
    }
    let mut b = StaticAbility::new(StaticEffect::Continuous {
        affected: affected.clone(),
        mods: vec![Modification::ModifyPT(p, t)],
    });
    match kind {
        Some(k) => {
            let k = k.strip_prefix("a ").or_else(|| k.strip_prefix("an "))?;
            let (f, plural, tail) = parse_object_phrase(k)?;
            if plural || !end(tail).trim().is_empty() {
                return None;
            }
            let base = affected.clone();
            a.effect = StaticEffect::Continuous {
                affected: Filter::and(vec![base.clone(), Filter::not(f.clone())]),
                mods: mods.clone(),
            };
            b.effect = StaticEffect::Continuous {
                affected: Filter::and(vec![base, f]),
                mods: match b.effect {
                    StaticEffect::Continuous { mods, .. } => mods,
                    _ => return None,
                },
            };
        }
        None => {
            // "its controller has seven or more cards in their graveyard": the controller
            // of the object it's attached to.
            let cond = match cond_text
                .strip_prefix("its controller has ")
                .and_then(|r| r.strip_suffix(" or more cards in their graveyard"))
            {
                Some(n) => {
                    let (n, rest) = parse_number(n)?;
                    if !rest.trim().is_empty() {
                        return None;
                    }
                    Condition::Compare(
                        Value::GraveyardSize(PlayerRef::ControllerOf(Box::new(Sel::AttachedTo))),
                        Cmp::Ge,
                        n,
                    )
                }
                None => crate::oracle::statics::parse_condition(cond_text, ctx)?,
            };
            a.condition = Some(Condition::Not(Box::new(cond.clone())));
            b.condition = Some(cond);
        }
    }
    Some(vec![ability(a), ability(b)])
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

inventory::submit! { AbilityPattern { name: "keyword statics: [static]. otherwise, [static] / [anthem]. it gets [p/t] instead ...", priority: 49, parse: otherwise_statics } }

/// "As long as the top card of your library is a creature card, creatures you control
/// that share a color with that card get +1/+1." (Crown of Convergence): the top card of
/// the library as the referent of a static ability, checked continuously.
fn top_card_static(text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if ctx.is_spell() || text.contains('\n') {
        return None;
    }
    let lower = text.to_lowercase();
    let r = end(&lower).strip_prefix("as long as the top card of your library is ")?;
    let (kind, rest) = r.split_once(", ")?;
    let kind = kind.strip_prefix("a ").or_else(|| kind.strip_prefix("an "))?;
    let (kf, plural, tail) = parse_object_phrase(kind)?;
    if plural || !end(tail).trim().is_empty() {
        return None;
    }
    let (subj, pt) = rest.split_once(" that share a color with that card get ")?;
    let (f, _, tail) = parse_object_phrase(subj)?;
    if !end(tail).trim().is_empty() {
        return None;
    }
    let (p, t, tail) = crate::oracle::effects::parse_pt_mod(pt)?;
    if !tail.trim().is_empty() {
        return None;
    }
    let top = Sel::TopOfLibrary(PlayerRef::You, Value::c(1));
    let mut st = StaticAbility::new(StaticEffect::Continuous {
        affected: Filter::and(vec![f, Filter::SharesColor(Box::new(top.clone()))]),
        mods: vec![Modification::ModifyPT(p, t)],
    });
    st.condition = Some(Condition::SelMatches(top, kf));
    Some(vec![AbilityDef::new(AbilityKind::Static(st), text)])
}

inventory::submit! { AbilityPattern { name: "keyword statics: as long as the top card of your library is [a card], [objects] that share a color with that card get +N/+N", priority: 40, parse: top_card_static } }

/// "Until end of turn, target creature with defender gains haste, can attack as though it
/// didn't have defender, and assigns combat damage equal to its toughness rather than its
/// power." (Walking Bulwark), "... gets +1/+0, gains \"[ability]\", and can attack as
/// though it didn't have defender." (The Pride of Hull Clade): predicates that change the
/// object (layers) joined with ones that change rules for it (CR 613.11), each read as the
/// sentence it would be about the same object, for the same duration.
fn mixed_predicates(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    if !(l.contains(" can attack as though it didn't have defender")
        || l.contains(" assigns combat damage equal to its toughness rather than its power"))
    {
        return None;
    }
    let (masked, quotes) = super::statics::mask_quotes(l)?;
    let (lead, core) = match masked.strip_prefix("until end of turn, ") {
        Some(r) => ("until end of turn, ", r.to_string()),
        None => ("", masked.clone()),
    };
    let verbs = [" gets ", " gains ", " can attack ", " assigns "];
    let i = verbs.iter().filter_map(|v| core.find(v)).min()?;
    let subj = core[..i].to_string();
    // Split the predicates at the commas and "and"s before a verb.
    let rest = &core[i + 1..];
    let mut preds: Vec<String> = Vec::new();
    let mut from = 0;
    let mut k = 0;
    while k < rest.len() {
        let tail = &rest[k..];
        let sep = [", and ", ", ", " and "].into_iter().find(|s| {
            tail.starts_with(s)
                && ["gets ", "gains ", "can attack ", "assigns "]
                    .iter()
                    .any(|v| tail[s.len()..].starts_with(v))
        });
        match sep {
            Some(s) => {
                preds.push(rest[from..k].to_string());
                k += s.len();
                from = k;
            }
            None => k += tail.chars().next().map_or(1, char::len_utf8),
        }
    }
    preds.push(rest[from..].to_string());
    if preds.len() < 2 {
        return None;
    }
    // Put the quotes back.
    let unmask = |s: &str| -> String {
        let mut out = s.to_string();
        for (n, q) in quotes.iter().enumerate() {
            out = out.replace(&format!("\"#{n}\""), &format!("\"{q}\""));
        }
        out
    };
    let saved = (b.targets.len(), b.it.clone());
    let result = (|| {
        let (what, r) = crate::oracle::effects::object_ref(&unmask(&subj), b)?;
        if !r.trim().is_empty() || matches!(what, Sel::None | Sel::All(_)) {
            return None;
        }
        let after_subject = b.targets.len();
        let mut out = Vec::new();
        for p in &preds {
            b.it = what.clone();
            let clause = if lead.is_empty() {
                format!("it {}", unmask(p))
            } else {
                format!("{lead}it {}", unmask(p))
            };
            out.push(crate::oracle::effects::parse_clause(&clause, b)?);
        }
        if b.targets.len() != after_subject {
            return None;
        }
        b.it = what;
        Some(Effect::seq(out))
    })();
    if result.is_none() {
        b.targets.truncate(saved.0);
        b.it = saved.1;
    }
    result
}

inventory::submit! { super::EffectPattern { name: "keyword statics: [subject] gains [kw], can attack as though it didn't have defender, and assigns combat damage ...", priority: 1090, parse: mixed_predicates } }

/// "That enchantment gains \"Cumulative upkeep {1}.\"" (Balduvian Shaman): a keyword
/// ability granted in quotes is that keyword (CR 702.24 for cumulative upkeep).
fn gains_quoted_keyword(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (head, q) = l.split_once(" gains \"").or_else(|| l.split_once(" gain \""))?;
    let (inner, tail) = q.split_once('"')?;
    let inner = inner.trim_end_matches('.');
    if inner.contains('"') || inner.contains(':') {
        return None;
    }
    // Only a keyword line.
    let kws = crate::oracle::effects::keyword_mods(inner)?;
    if kws.is_empty() {
        return None;
    }
    let verb = if l.contains(" gains \"") { "gains" } else { "gain" };
    crate::oracle::effects::parse_clause(&format!("{head} {verb} {inner}{tail}"), b)
}

inventory::submit! { super::EffectPattern { name: "keyword statics: [object] gains \"[keyword]\"", priority: 1090, parse: gains_quoted_keyword } }

/// "That enchantment gains ..." after an instruction about a target enchantment: "that
/// enchantment" is what "it" refers to.
fn that_noun_is_it(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let rest = ["that enchantment ", "that artifact ", "that land "]
        .iter()
        .find_map(|p| l.strip_prefix(p))?;
    if matches!(b.it, Sel::None | Sel::This) || b.sentences == 0 {
        return None;
    }
    crate::oracle::effects::parse_clause(&format!("it {rest}"), b)
}

inventory::submit! { super::EffectPattern { name: "keyword statics: that enchantment [predicate] (it)", priority: 1095, parse: that_noun_is_it } }
