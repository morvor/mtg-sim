//! Characteristic-changing grammar (CR 613.1d-f, 613.4b): one subject with a list of
//! predicates that each change what the subject is, applied together as one continuous
//! effect for one duration (CR 611.2):
//!
//! * "Until end of turn, ~ has base power and toughness 5/3, gains trample, and isn't a
//!   Human." (Werewolf Pack Leader)
//! * "Target creature gets +1/+0, becomes black, and gains shadow until end of turn."
//!   (Traitor's Clutch)
//! * "each creature target opponent controls loses all abilities, becomes a Coward in
//!   addition to its other types, and has base power and toughness 1/1" (Curious Colossus)
//! * "Vehicles you control become artifact creatures until end of turn." (Vehicles have a
//!   printed power and toughness, CR 301.7b)
//!
//! Each predicate is one of: "gets +N/+N" (7c), "gains/has [keywords]" (6), "loses [keyword
//! | all abilities]" (6), "becomes [colors]" (5), "becomes [type words]" (4, and 7b for a
//! "with base power and toughness N/N" in them), "has base power [and toughness] N[/N]"
//! (7b), "isn't a [type]" / "isn't [supertype]" (4), "loses all creature types" (4). The
//! modifications are applied in layer order; within a layer, in the order written
//! (CR 613.7, 613.1f).

use super::planeswalkers_modal_becomes::{makes_creature, subject};
use super::statics::{type_predicate_mods, Subject};
use super::{EffectPattern, FollowupPattern, StaticPattern};
use crate::oracle::CompileContext;
use crate::ability::*;
use crate::keywords::Keyword;
use crate::oracle::effects::{keyword_mods, object_ref, parse_pt_mod, Builder};
use crate::oracle::phrases::end;
use crate::types::*;

thread_local! {
    /// Set while the predicates of a static ability are parsed (see
    /// [`special_quality_keyword`]).
    static STATIC_CONTEXT: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// The verbs that start a predicate, in their singular and plural forms.
const VERBS: &[&str] = &[
    "gets ", "get ", "gains ", "gain ", "has ", "have ", "loses ", "lose ", "becomes ",
    "become ", "isn't ", "aren't ", "is no longer ", "are no longer ",
];

fn starts_with_verb(s: &str) -> bool {
    VERBS.iter().any(|v| s.starts_with(v))
}

/// Splits "[a], [b], and [c]" / "[a] and [b]" at the separators that start a predicate.
/// "[subject] gets +X/+Y [duration], where X is [value] and Y is its toughness"
/// (Phyrexian Ingester, Bioplasm): "its" is the object of the first value.
fn gets_x_y(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (head, def) = l.split_once(", where x is ")?;
    let (xs, ys) = def.split_once(" and y is ")?;
    let (subj, rest) = head
        .split_once(" gets +x/+y")
        .or_else(|| head.split_once(" get +x/+y"))?;
    let duration = only_duration(rest)?;
    // "Y is its toughness" after "X is the exiled creature card's power".
    let ys = match (ys.strip_prefix("its "), xs.split_once("'s ")) {
        (Some(stat), Some((obj, _))) => format!("{obj}'s {stat}"),
        _ => ys.to_string(),
    };
    let saved = (b.targets.len(), b.it.clone());
    let result = (|| {
        let (what, r) = object_ref(subj, b)?;
        if !r.trim().is_empty() || matches!(what, Sel::None) {
            return None;
        }
        let (x, r1) = crate::oracle::statics::parse_value_phrase(xs, b)?;
        let (y, r2) = crate::oracle::statics::parse_value_phrase(&ys, b)?;
        if !r1.trim().is_empty() || !r2.trim().is_empty() || b.targets.len() != saved.0 {
            return None;
        }
        Some(Effect::Modify {
            what,
            mods: vec![Modification::ModifyPT(x, y)],
            duration,
        })
    })();
    if result.is_none() {
        b.targets.truncate(saved.0);
        b.it = saved.1;
    }
    result
}

inventory::submit! { EffectPattern { name: "becomes grammar: gets +X/+Y, where X is ... and Y is ...", priority: 1100, parse: gets_x_y } }

/// "It deals X plus 1 damage instead if that target is a creature or planeswalker.":
/// "that target" is what "it" names in the condition.
fn that_target_instead(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    if !l.contains(" instead if that target is ") {
        return false;
    }
    let mut t = l.replacen(" instead if that target is ", " instead if it's ", 1);
    // "if it's white and/or blue": of either color.
    if let Some((a, c)) = t.split_once(" instead if it's ") {
        if let Some((x, y)) = c.split_once(" and/or ") {
            if Color::from_word(x).is_some() && Color::from_word(y).is_some() {
                t = format!("{a} instead if it's {x} or {y}");
            }
        }
    }
    let saved = b.it.clone();
    if let Some(i) = last_damage_target(prev) {
        b.it = Sel::Target(i);
    }
    let ok = crate::oracle_ext::apply_followup_ext(&t, prev, b);
    if !ok {
        b.it = saved;
    }
    ok
}

fn last_damage_target(e: &Effect) -> Option<u8> {
    match e {
        Effect::DealDamage {
            to: Sel::Target(i), ..
        } => Some(*i),
        Effect::Seq(v) => v.last().and_then(last_damage_target),
        _ => None,
    }
}

inventory::submit! { FollowupPattern { name: "becomes grammar: ... instead if that target is ...", priority: 1100, apply: that_target_instead } }

/// "creatures that are green and/or white": of any of those colors.
fn that_are_colors<'a>(s: &'a str, _f: &Filter) -> Option<(Filter, &'a str)> {
    let r = s.strip_prefix("that are ")?;
    let (a, r) = r.split_once(' ')?;
    let r = r.strip_prefix("and/or ")?;
    let (c, rest) = match r.split_once(' ') {
        Some((c, rest)) => (c, rest),
        None => (r, ""),
    };
    let a = Color::from_word(a)?;
    let c = Color::from_word(c)?;
    let rest_start = s.len() - rest.len();
    Some((
        Filter::Or(vec![Filter::Color(a), Filter::Color(c)]),
        &s[rest_start..],
    ))
}

inventory::submit! { super::FilterSuffixPattern { name: "becomes grammar: that are [color] and/or [color]", priority: 1100, parse: that_are_colors } }

/// "~ gets +X/+Y, where X is the exiled creature card's power and Y is its toughness."
/// (Phyrexian Ingester): values that follow the game as the static ability applies.
fn static_gets_x_y(l: &str, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let (head, def) = l.split_once(", where x is ")?;
    let (xs, ys) = def.split_once(" and y is ")?;
    let subj_text = head.strip_suffix(" gets +x/+y")?;
    let ys = match (ys.strip_prefix("its "), xs.split_once("'s ")) {
        (Some(stat), Some((obj, _))) => format!("{obj}'s {stat}"),
        _ => ys.to_string(),
    };
    let subj = super::statics::parse_subject(subj_text, Some(&Sel::This), ctx)?;
    let mut b = Builder::new(ctx);
    b.it = Sel::This;
    let (x, r1) = crate::oracle::statics::parse_value_phrase(xs, &mut b)?;
    let (y, r2) = crate::oracle::statics::parse_value_phrase(&ys, &mut b)?;
    if !r1.trim().is_empty() || !r2.trim().is_empty() || !b.targets.is_empty() {
        return None;
    }
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Continuous {
            affected: subj.filter,
            mods: vec![Modification::ModifyPT(x, y)],
        })),
        text,
    )])
}

/// "As a historic permanent you control enters, it becomes a 7/7 Dinosaur creature in
/// addition to its other types." (CR 614.1c: a replacement effect that modifies how the
/// permanent enters; it's that from then on.)
fn as_objects_enter_becomes(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let r = end(l).strip_prefix("as ")?;
    let (obj, pred) = r.split_once(" enters, it becomes ")?;
    let obj = obj
        .strip_prefix("a ")
        .or_else(|| obj.strip_prefix("an "))
        .or_else(|| obj.strip_prefix("another "))?;
    let (f, _, tail) = crate::oracle::phrases::parse_object_phrase(obj)?;
    if !end(tail).trim().is_empty() {
        return None;
    }
    let f = if r.starts_with("another ") {
        Filter::and(vec![f, Filter::Other])
    } else {
        f
    };
    let subj = super::statics::Subject {
        filter: Filter::Any,
        it: None,
        hint: CardType::Creature,
        lands: false,
        creatures: super::statics::filter_mentions(&f, &|x| {
            matches!(x, Filter::Type(CardType::Creature))
        }),
    };
    let mods = type_predicate_mods(pred, &subj)?;
    if makes_creature(&mods)
        && !mods
            .iter()
            .any(|m| matches!(m, Modification::SetPT(Some(_), Some(_))))
    {
        return None;
    }
    let e = Effect::OnEntry(Box::new(Effect::Modify {
        what: Sel::This,
        mods,
        duration: Duration::Permanent,
    }));
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Replacement(
            ReplacementDef {
                event: ReplacementEvent::EntersBattlefield(f),
                action: ReplacementAction::AsEnters(Box::new(e)),
                self_replacement: false,
                optional: false,
            },
        ))),
        text,
    )])
}

inventory::submit! { StaticPattern { name: "becomes grammar: as [objects] enter, it becomes ...", priority: 1100, parse: as_objects_enter_becomes } }

/// "Until end of turn, that permanent becomes saddled if it's a Mount and becomes an
/// artifact creature if it's a Vehicle." (Alacrian Armory): each predicate applies if the
/// object is of its kind.
fn predicates_if_its(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (lead, core) = match l.strip_prefix("until end of turn, ") {
        Some(r) => (" until end of turn", r),
        None => ("", l),
    };
    if !core.contains(" if it's ") {
        return None;
    }
    let i = core.find(" becomes ")?;
    let subj_text = &core[..i];
    let preds = split_predicates(&core[i + 1..]);
    if preds.len() < 2 {
        return None;
    }
    let saved = (b.targets.len(), b.it.clone());
    let result = (|| {
        let (what, r) = object_ref(subj_text, b)?;
        if !r.trim().is_empty() || matches!(what, Sel::None | Sel::All(_)) {
            return None;
        }
        let mut out = Vec::new();
        for p in preds {
            let (pred, kind) = p.split_once(" if it's ")?;
            let kind = kind
                .strip_prefix("a ")
                .or_else(|| kind.strip_prefix("an "))?;
            let (f, plural, tail) = crate::oracle::phrases::parse_object_phrase(kind)?;
            if plural || !tail.trim().is_empty() {
                return None;
            }
            b.it = what.clone();
            let e = crate::oracle::effects::parse_clause(&format!("it {pred}{lead}"), b)?;
            out.push(Effect::If {
                cond: Condition::SelMatches(what.clone(), f),
                then: Box::new(e),
                otherwise: Box::new(Effect::Noop),
            });
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

inventory::submit! { EffectPattern { name: "becomes grammar: [predicate] if it's a [kind] and [predicate] if it's a [kind]", priority: 1100, parse: predicates_if_its } }


fn split_predicates(s: &str) -> Vec<&str> {
    split_predicates_with(s, &[])
}

/// [`split_predicates`], also splitting before the `more` verbs.
fn split_predicates_with<'a>(s: &'a str, more: &[&str]) -> Vec<&'a str> {
    let verb = |r: &str| starts_with_verb(r) || more.iter().any(|v| r.starts_with(v));
    let mut out = Vec::new();
    let mut from = 0;
    let mut i = 0;
    while i < s.len() {
        let rest = &s[i..];
        let sep = [", and ", ", ", " and "]
            .into_iter()
            .find(|sep| rest.starts_with(sep) && verb(&rest[sep.len()..]));
        match sep {
            Some(sep) => {
                out.push(&s[from..i]);
                i += sep.len();
                from = i;
            }
            None => i += rest.chars().next().map_or(1, char::len_utf8),
        }
    }
    out.push(&s[from..]);
    out
}

/// A leading or trailing duration of the whole sentence.
fn split_duration(l: &str) -> (Duration, &str) {
    for (p, d) in [
        ("until end of turn, ", Duration::EndOfTurn),
        ("until your next turn, ", Duration::UntilYourNextTurn),
    ] {
        if let Some(r) = l.strip_prefix(p) {
            return (d, r);
        }
    }
    match crate::oracle::effects::duration_suffix(l) {
        // "this turn" belongs to a restriction predicate ("can't block this turn").
        (Duration::EndOfTurn, _) if l.ends_with(" this turn") => (Duration::Permanent, l),
        (d, r) => (d, r),
    }
}

/// The duration `s` (" until end of turn", or nothing) consists of.
fn only_duration(s: &str) -> Option<Duration> {
    let t = format!("x{s}");
    let (d, rest) = crate::oracle::effects::duration_suffix(&t);
    (rest == "x").then_some(d)
}

/// "N/N" or "X/X" (X defined by the ability).
fn pt_values(s: &str) -> Option<(Value, Value)> {
    let (p, t) = s.trim().split_once('/')?;
    Some((pt_value(p)?, pt_value(t)?))
}

fn pt_value(s: &str) -> Option<Value> {
    match s {
        "x" if super::value_grammar::x_defined() => Some(Value::X),
        s => s.parse::<i32>().ok().map(Value::c),
    }
}

/// "base power and toughness N/N", "base power N" (layer 7b).
fn base_pt_mods(s: &str) -> Option<Vec<Modification>> {
    if let Some(r) = s.strip_prefix("base power and toughness ") {
        let (p, t) = pt_values(r)?;
        return Some(vec![Modification::SetPT(Some(p), Some(t))]);
    }
    let r = s.strip_prefix("base power ")?;
    Some(vec![Modification::SetPT(Some(pt_value(r)?), None)])
}

/// A keyword list ("flying and trample", "toxic 1 and protection from each of that
/// permanent's colors"): "protection/hexproof from each of [object]'s colors" is one
/// ability per color that object has as the effect begins (CR 702.16g, 608.2h; locked
/// in by `Game::fix_mods`).
fn kw_list_mods(s: &str, subj: &Subject, b: &mut Builder) -> Option<Vec<Modification>> {
    let mut out = Vec::new();
    for part in crate::oracle::keywords::split_keyword_phrases(s) {
        let mut done = false;
        let own = matches!(subj.it, Some(Sel::This));
        if let Some(m) = special_quality_keyword(&part, own, b) {
            out.push(m);
            continue;
        }
        for (p, kind) in [
            ("protection from each of ", crate::keywords::KeywordKind::Protection),
            ("hexproof from each of ", crate::keywords::KeywordKind::Hexproof),
        ] {
            let Some(obj) = part
                .strip_prefix(p)
                .and_then(|r| r.strip_suffix("'s colors"))
            else {
                continue;
            };
            let saved = b.targets.len();
            let (sel, rest) = object_ref(obj, b)?;
            if !rest.is_empty() || b.targets.len() != saved || matches!(sel, Sel::None) {
                return None;
            }
            let mut k = Keyword::new(kind);
            k.filter = Some(Filter::SharesColor(Box::new(sel)));
            k.text = Some(part.as_str().into());
            out.push(Modification::AddKeyword(k));
            done = true;
        }
        if !done {
            out.extend(keyword_mods(&part)?);
        }
    }
    (!out.is_empty()).then_some(out)
}

/// Keyword abilities whose quality is defined by the text around them: "hexproof from
/// that color" / "protection from that color" (the color chosen as the effect began,
/// CR 608.2h), and, for the source's own abilities (`own`), "protection from each color
/// among permanents you control" and "protection from each of the exiled card's card
/// types" (qualities that follow the game as the static ability applies).
fn special_quality_keyword(part: &str, own: bool, b: &mut Builder) -> Option<Modification> {
    use crate::keywords::KeywordKind::{Hexproof, Protection};
    let (kind, q) = if let Some(q) = part.strip_prefix("protection from ") {
        (Protection, q)
    } else {
        (Hexproof, part.strip_prefix("hexproof from ")?)
    };
    let filter = match q {
        "that color" => Filter::ChosenColor,
        // "Each creature has protection from each of its colors": one ability per color
        // the object that has it has (CR 702.16g), i.e. from each object that shares a
        // color with it. In a resolving effect, "its" is what the sentence is about, and
        // its colors are those it has as the effect begins (CR 608.2h).
        "each of its colors" => {
            let sel = if STATIC_CONTEXT.with(|c| c.get()) {
                Sel::This
            } else {
                b.it.clone()
            };
            if matches!(sel, Sel::All(_) | Sel::None) {
                return None;
            }
            Filter::SharesColor(Box::new(sel))
        }
        "each color among permanents you control" if own && kind == Protection => {
            Filter::SharesColor(Box::new(Sel::All(Filter::and(vec![
                Filter::Permanent,
                Filter::ControlledBy(PlayerRel::You),
            ]))))
        }
        _ => {
            let obj = q
                .strip_prefix("each of ")?
                .strip_suffix("'s card types")?;
            if !own || kind != Protection {
                return None;
            }
            let saved = b.targets.len();
            let (sel, rest) = object_ref(obj, b)?;
            if !rest.is_empty() || b.targets.len() != saved || !names_linked_card(&sel) {
                b.targets.truncate(saved);
                return None;
            }
            Filter::SharesCardType(Box::new(sel))
        }
    };
    let mut k = Keyword::new(kind);
    k.filter = Some(filter);
    k.text = Some(part.into());
    Some(Modification::AddKeyword(k))
}

/// A selection of cards the source's linked ability exiled ("the exiled card").
fn names_linked_card(sel: &Sel) -> bool {
    !matches!(
        sel,
        Sel::This | Sel::None | Sel::Target(_) | Sel::TriggerObject | Sel::TriggerSpell
    )
}

/// "base power and toughness each equal to [value]", "base power and base toughness each
/// equal to [value]" (layer 7b; the value is determined as the effect begins, CR 608.2h).
fn base_pt_each_equal(s: &str, b: &mut Builder) -> Option<Value> {
    let r = s
        .strip_prefix("base power and toughness each equal to ")
        .or_else(|| s.strip_prefix("base power and base toughness each equal to "))?;
    let (v, rest) = crate::oracle::statics::parse_value_phrase(r, b)?;
    rest.trim().is_empty().then_some(v)
}

/// "loses [keyword]": that keyword ability (CR 613.1f).
fn lose_keyword_mods(s: &str) -> Option<Vec<Modification>> {
    keyword_mods(s)?
        .into_iter()
        .map(|m| match m {
            Modification::AddKeyword(k) => Some(lose_keyword(k)),
            _ => None,
        })
        .collect()
}

fn lose_keyword(k: Keyword) -> Modification {
    if k.text.is_none() && k.n.is_none() && k.filter.is_none() && k.cost.is_none() {
        Modification::RemoveKeyword(k.kind)
    } else {
        Modification::LoseKeyword(k)
    }
}

/// "isn't a Human", "isn't an artifact", "isn't snow", "is no longer an Equipment".
fn isnt_mods(s: &str) -> Option<Vec<Modification>> {
    let w = s
        .strip_prefix("a ")
        .or_else(|| s.strip_prefix("an "))
        .unwrap_or(s);
    let w = w.trim();
    if w.contains(' ') {
        return None;
    }
    let sing = |w: &str| -> String {
        w.strip_suffix('s')
            .filter(|_| s == w)
            .unwrap_or(w)
            .to_string()
    };
    if let Some(t) = CardType::from_word(w).or_else(|| CardType::from_word(&sing(w))) {
        return Some(vec![Modification::RemoveTypes(vec![t])]);
    }
    if let Some(st) = Supertype::from_word(w) {
        return Some(vec![Modification::RemoveSupertypes(vec![st])]);
    }
    let st = crate::oracle::phrases::subtype_word(w)
        .or_else(|| crate::oracle::phrases::subtype_word(&sing(w)))?;
    Some(vec![Modification::RemoveSubtypes(vec![st])])
}

/// "becomes [colors]" / "becomes [colors] in addition to its other colors" (CR 105.3).
fn color_mods(s: &str) -> Option<Vec<Modification>> {
    if s == "colorless" {
        return Some(vec![Modification::SetColors(ColorSet::NONE)]);
    }
    let (cs, rest) = super::r105_colors::parse_color_list(s)?;
    match rest.trim() {
        "" => Some(vec![Modification::SetColors(cs)]),
        "in addition to its other colors" | "in addition to their other colors" => {
            Some(vec![Modification::AddColors(cs)])
        }
        _ => None,
    }
}

/// "~ becomes a ~ in addition to its other types" said by a card whose name is a creature
/// type (Coward // Killer: "becomes a Coward"): the normalizer replaced the type by "~".
fn name_as_type(s: &str, b: &Builder) -> Option<String> {
    if !s.contains('~') {
        return Some(s.to_string());
    }
    // "Lizard, Connors's Curse" is "Lizard" in its text.
    let name = b.ctx.card_name.split(',').next()?.to_lowercase();
    let st = crate::oracle::phrases::subtype_word(&name)?;
    is_creature_type(st.as_str()).then(|| s.replace('~', &name))
}

/// The modifications of one predicate.
pub(crate) fn predicate_mods(
    p: &str,
    subj: &Subject,
    b: &mut Builder,
) -> Option<Vec<Modification>> {
    let p = p.trim();
    if let Some(r) = p.strip_prefix("gets ").or_else(|| p.strip_prefix("get ")) {
        let (pw, t, rest) = parse_pt_mod(r)?;
        if !rest.trim().is_empty() {
            return None;
        }
        return Some(vec![Modification::ModifyPT(pw, t)]);
    }
    if let Some(r) = p.strip_prefix("has ").or_else(|| p.strip_prefix("have ")) {
        if let Some(m) = base_pt_mods(r) {
            return Some(m);
        }
        if let Some(v) = base_pt_each_equal(r, b) {
            return Some(vec![Modification::SetPT(Some(v.clone()), Some(v))]);
        }
        // "has base power and toughness 4/4, vigilance, and trample"
        if let Some(rest) = r.strip_prefix("base power and toughness ") {
            let (pt, kws) = rest.split_once(", ")?;
            let mut m = base_pt_mods(&format!("base power and toughness {pt}"))?;
            m.extend(keyword_mods(kws.strip_prefix("and ").unwrap_or(kws))?);
            return Some(m);
        }
        return kw_list_mods(r, subj, b);
    }
    if let Some(r) = p.strip_prefix("gains ").or_else(|| p.strip_prefix("gain ")) {
        // "Lands you control gain all basic land types" (CR 305.6: the abilities come
        // with the types).
        if r == "all basic land types" && subj.lands {
            return Some(vec![Modification::AddSubtypes(
                ["Plains", "Island", "Swamp", "Mountain", "Forest"]
                    .into_iter()
                    .map(Subtype::from)
                    .collect(),
            )]);
        }
        return kw_list_mods(r, subj, b);
    }
    if let Some(r) = p.strip_prefix("loses ").or_else(|| p.strip_prefix("lose ")) {
        return match r {
            "all abilities" => Some(vec![Modification::RemoveAllAbilities]),
            // The ability whose text this is (the Licids).
            "this ability" => Some(vec![Modification::Custom {
                name: crate::kw::end_this_effect::LOSE_THIS_ABILITY.into(),
                layer: Layer::L6Ability,
            }]),
            "all creature types" => Some(vec![Modification::RemoveAllCreatureTypes]),
            _ => lose_keyword_mods(r),
        };
    }
    for v in ["isn't ", "aren't ", "is no longer ", "are no longer "] {
        if let Some(r) = p.strip_prefix(v) {
            return isnt_mods(r);
        }
    }
    let r = p
        .strip_prefix("becomes ")
        .or_else(|| p.strip_prefix("become "))?;
    if let Some(m) = color_mods(r) {
        return Some(m);
    }
    let r = name_as_type(r, b)?;
    // "a 4/4 Spirit artifact creature that's no longer an Equipment"
    for sep in [" that's no longer ", " that are no longer "] {
        if let Some((head, x)) = r.split_once(sep) {
            let mut m = predicate_mods(&format!("becomes {head}"), subj, b)?;
            m.extend(isnt_mods(x)?);
            return Some(m);
        }
    }
    // "a green and blue Fractal with base power and toughness each equal to X plus 1"
    if let Some((head, v)) = r.split_once(" with ") {
        if let Some(v) = base_pt_each_equal(v, b) {
            let mut m = type_predicate_mods(head, subj)?;
            m.push(Modification::SetPT(Some(v.clone()), Some(v)));
            return Some(m);
        }
    }
    // "a 4/4 Giant creature with protection from each of that spell's colors"
    if let Some((head, kws)) = r.split_once(" with ") {
        if kws.contains("'s colors") {
            let mut m = type_predicate_mods(head, subj)?;
            m.extend(kw_list_mods(kws, subj, b)?);
            return Some(m);
        }
    }
    // "a Dragon with base power and toughness 4/4, flying, and haste": the keywords after
    // the power and toughness.
    if let Some((head, kws)) = split_pt_keywords(&r) {
        let mut m = type_predicate_mods(head, subj)?;
        m.extend(keyword_mods(kws)?);
        return Some(m);
    }
    type_predicate_mods(&r, subj)
}

/// "[type words] with base power and toughness N/N, [keywords]" → the type words with the
/// power and toughness, and the keyword list.
fn split_pt_keywords(s: &str) -> Option<(&str, &str)> {
    const BASE: &str = " with base power and toughness ";
    let i = s.find(BASE)?;
    let after = &s[i + BASE.len()..];
    let (pt, rest) = after.split_once(", ")?;
    pt_values(pt)?;
    let kws = rest.strip_prefix("and ").unwrap_or(rest);
    Some((&s[..i + BASE.len() + pt.len()], kws))
}

/// Whether the affected objects have a printed power and toughness, so that becoming a
/// creature without a set power and toughness gives it a usable one (CR 208.3): the
/// source when it has them, or Vehicles (CR 301.7b).
fn has_printed_pt(what: &Sel, subj_text: &str, b: &Builder) -> bool {
    match what {
        Sel::This => b.ctx.power.is_some() && b.ctx.toughness.is_some(),
        _ => {
            let w: Vec<&str> = subj_text.split(' ').collect();
            w.iter().any(|w| matches!(*w, "vehicle" | "vehicles"))
                && !w.iter().any(|w| matches!(*w, "or" | "and"))
        }
    }
}

/// Whether a creature made by `mods` has a power and toughness: one they set, or one the
/// objects already have (creatures, CR 208.3; printed ones).
fn creature_has_pt(mods: &[Modification], what: &Sel, subj: &Subject, text: &str, b: &Builder) -> bool {
    !makes_creature(mods)
        || subj.creatures
        || mods
            .iter()
            .any(|m| matches!(m, Modification::SetPT(Some(_), Some(_))))
        || has_printed_pt(what, text, b)
}

/// Whether `mods` change more than abilities and P/T modifications: types, colors, base
/// power and toughness, or the loss of abilities.
fn changes_characteristics(mods: &[Modification]) -> bool {
    mods.iter().any(|m| match m {
        // Qualities this grammar reads from the game ("protection from each of that
        // permanent's colors").
        Modification::AddKeyword(k) => matches!(
            k.filter,
            Some(Filter::SharesColor(_) | Filter::SharesCardType(_) | Filter::ChosenColor)
        ),
        Modification::ModifyPT(..) | Modification::AddAbility(_) => false,
        _ => true,
    })
}

/// Where the chosen option's modifications go among the others while parsing.
const CHOICE_SLOT: &str = "becomes grammar: chosen option";

/// "becomes your choice of [A] or [B]" / "becomes [A] or [B]": each option's
/// modifications, named by its text.
fn becomes_one_of(
    p: &str,
    subj: &Subject,
    b: &mut Builder,
) -> Option<Vec<(String, Vec<Modification>)>> {
    let r = p
        .strip_prefix("becomes ")
        .or_else(|| p.strip_prefix("become "))?;
    let r = r.strip_prefix("your choice of ").unwrap_or(r);
    let (x, y) = r.split_once(" or ")?;
    let mut out = Vec::new();
    for o in [x, y] {
        let m = predicate_mods(&format!("becomes {o}"), subj, b)?;
        out.push((o.trim_start_matches("a ").trim_start_matches("an ").to_string(), m));
    }
    Some(out)
}

/// Whether the object of the triggered ability whose text has `pred` is a creature: its
/// trigger condition is "whenever a [...] creature [...] [verb]" ("Whenever a creature you
/// control with flying attacks, you may have it become ...").
fn trigger_object_is_creature(b: &Builder, pred: &str) -> bool {
    if !b.in_trigger {
        return false;
    }
    let raw = crate::oracle::raw_text().to_lowercase();
    let Some(line) = raw.lines().find(|l| l.contains(pred)) else {
        return false;
    };
    let Some((cond, _)) = line.split_once(", ") else {
        return false;
    };
    let Some(r) = cond
        .strip_prefix("whenever ")
        .or_else(|| cond.strip_prefix("when "))
    else {
        return false;
    };
    let mut words = vec![];
    for w in r.split(' ') {
        if matches!(w, "attacks" | "blocks" | "enters" | "dies" | "deals" | "becomes") {
            return matches!(words.first(), Some(&"a" | &"an" | &"another"))
                && words.contains(&"creature")
                && !words.iter().any(|w| matches!(*w, "spell" | "or" | "and"));
        }
        words.push(w);
    }
    false
}

/// Verbs that start a predicate that isn't a characteristic change ("can't block this
/// turn"): such a predicate is its own instruction about the same object.
const OTHER_VERBS: &[&str] = &["can't ", "can "];

/// "[subject] [predicate], [predicate], and [predicate] [duration]".
fn subject_predicates(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l)
        .replace(" his other ", " its other ")
        .replace(" her other ", " its other ");
    let (duration, core) = split_duration(&l);
    // "you may have it become ..."
    let core = core.strip_prefix("have ").unwrap_or(core);
    // The subject ends where the first predicate begins.
    let i = VERBS
        .iter()
        .chain(OTHER_VERBS)
        .filter_map(|v| core.find(&format!(" {v}")))
        .min()?;
    let (subj_text, rest) = (&core[..i], &core[i + 1..]);
    // "up to two target creatures each have ..."
    let subj_text = subj_text.strip_suffix(" each").unwrap_or(subj_text);
    let preds = split_predicates_with(rest, OTHER_VERBS);
    let saved = (b.targets.clone(), b.it.clone());
    let mut listed_subject = false;
    let result = (|| {
        let (what, r) = match subj_text {
            // A card named for two characters ("Moon Girl and Devil Dinosaur") is "they".
            "they" if b.ctx.card_name.contains(" and ") => (Sel::This, String::new()),
            // "each Advisor, Artificer, and Monk you control"
            s if s.starts_with("each ") && s.contains(", and ") => {
                listed_subject = true;
                object_ref(&super::statics::union_nouns(s), b)?
            }
            _ => object_ref(subj_text, b)?,
        };
        if !r.trim().is_empty() || matches!(what, Sel::None) {
            return None;
        }
        let mut subj = subject(subj_text, b);
        // A Vehicle that attacks or is crewed is a creature (CR 301.7): "~ becomes an
        // Assassin in addition to its other types until end of turn".
        if matches!(what, Sel::This)
            && b.ctx.type_line.subtypes.iter().any(|s| s == "Vehicle")
            && !matches!(duration, Duration::Permanent)
        {
            subj.creatures = true;
        }
        // "Whenever a creature you control with flying attacks, you may have it become
        // ...": the trigger's object is a creature.
        if matches!(what, Sel::TriggerObject) && trigger_object_is_creature(b, rest) {
            subj.creatures = true;
        }
        // "its" in the predicates is what the sentence is about (unless the text
        // rewrote "that permanent's" as "its", which keeps meaning that object).
        if !b.its_is_it {
            b.it = what.clone();
        }
        let mut mods = Vec::new();
        let mut others = Vec::new();
        let mut choice: Option<Vec<(String, Vec<Modification>)>> = None;
        let mut pending = Vec::new();
        for p in &preds {
            if let Some(m) = predicate_mods(p, &subj, b) {
                mods.extend(m);
                continue;
            }
            // "becomes your choice of a blue Frog creature with base power and toughness
            // 1/1 or a blue Octopus creature with base power and toughness 4/4", "becomes
            // a Plains or an Island": one of them, chosen as the effect begins.
            if choice.is_none() {
                if let Some(opts) = becomes_one_of(p, &subj, b) {
                    choice = Some(opts);
                    mods.push(Modification::Custom {
                        name: CHOICE_SLOT.into(),
                        layer: Layer::L4Type,
                    });
                    continue;
                }
            }
            // Another instruction about the same single object, with its own duration
            // (parsed below, once a characteristic predicate made this sentence ours).
            if !OTHER_VERBS.iter().any(|v| p.starts_with(v)) || matches!(what, Sel::All(_)) {
                return None;
            }
            pending.push(*p);
        }
        if mods.is_empty() || !creature_has_pt(&mods, &what, &subj, subj_text, b) {
            return None;
        }
        // Only sentences that change what the object is: plain grants and pumps ("the
        // token has enchant creature", "it gets +1/+1 and gains flying") are other
        // patterns'.
        if !changes_characteristics(&mods) && !listed_subject {
            return None;
        }
        for p in pending {
            b.it = what.clone();
            others.push(crate::oracle::effects::parse_clause(&format!("it {p}"), b)?);
        }
        b.it = what.clone();
        match choice {
            None => others.push(Effect::Modify {
                what,
                mods,
                duration,
            }),
            Some(opts) => {
                let options = opts
                    .into_iter()
                    .map(|(name, opt)| {
                        let mods = mods
                            .iter()
                            .flat_map(|m| match m {
                                Modification::Custom { name, .. } if name == CHOICE_SLOT => {
                                    opt.clone()
                                }
                                m => vec![m.clone()],
                            })
                            .collect();
                        let e = Effect::Modify {
                            what: what.clone(),
                            mods,
                            duration: duration.clone(),
                        };
                        (name, e)
                    })
                    .collect();
                others.push(Effect::ChooseOne {
                    who: PlayerRef::You,
                    options,
                });
            }
        }
        Some(Effect::seq(others))
    })();
    if result.is_none() {
        b.targets = saved.0;
        b.it = saved.1;
    }
    result
}

inventory::submit! { EffectPattern { name: "becomes grammar: [subject] [characteristic predicates]", priority: 1100, parse: subject_predicates } }

// ---------------------------------------------------------------------------
// "is [characteristic]": static abilities, and the objects an effect puts onto the
// battlefield
// ---------------------------------------------------------------------------

/// The static form of a predicate: "has [...]", "gets [...]", "is/are [type words or
/// colors]", "isn't [...]", "loses/lose all creature types|abilities".
fn static_predicate_mods(p: &str, subj: &Subject, b: &mut Builder) -> Option<Vec<Modification>> {
    for v in ["is ", "are "] {
        if let Some(r) = p.strip_prefix(v) {
            if r.starts_with("no longer ") {
                break;
            }
            return color_mods(r).or_else(|| type_predicate_mods(r, subj));
        }
    }
    if p.starts_with("become") {
        return None;
    }
    predicate_mods(p, subj, b)
}

const STATIC_VERBS: &[&str] = &["is ", "are "];

/// Splits a static line into its subject and predicates.
fn static_parts(l: &str) -> Option<(&str, Vec<&str>)> {
    let i = VERBS
        .iter()
        .chain(STATIC_VERBS)
        .filter_map(|v| l.find(&format!(" {v}")))
        .min()?;
    Some((&l[..i], split_predicates_with(&l[i + 1..], STATIC_VERBS)))
}

/// "other snow and Zombie creatures you control": the creatures that are snow or Zombies
/// (a supertype and a subtype joined by "and" before the noun).
fn either_adjective(s: &str, ctx: &CompileContext) -> Option<Subject> {
    let words: Vec<&str> = s.split(' ').collect();
    let i = words.iter().position(|w| *w == "and")?;
    let (a, c) = (*words.get(i.checked_sub(1)?)?, *words.get(i + 1)?);
    let sup = |w: &str| Supertype::from_word(w);
    let sub = |w: &str| crate::oracle::phrases::subtype_word(w);
    let either = match (sup(a), sub(c), sub(a), sup(c)) {
        (Some(x), Some(y), _, _) => Filter::Or(vec![Filter::Supertype(x), Filter::Subtype(y)]),
        (_, _, Some(y), Some(x)) => Filter::Or(vec![Filter::Subtype(y), Filter::Supertype(x)]),
        _ => return None,
    };
    let rest: Vec<&str> = words
        .iter()
        .enumerate()
        .filter(|(j, _)| *j + 1 != i && *j != i && *j != i + 1)
        .map(|(_, w)| *w)
        .collect();
    let mut subj = super::statics::parse_subject(&rest.join(" "), None, ctx)?;
    subj.filter = Filter::and(vec![subj.filter, either]);
    Some(subj)
}

/// "[objects] have base power and toughness 3/3 and lose all creature types" (Curse of
/// Conformity), "Each nonland permanent you control is all colors." (Leyline of the
/// Guildpact), "All lands are no longer snow." (Melting).
fn static_predicates(l: &str, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if ctx.is_spell() {
        return None;
    }
    let l = end(l);
    if let Some(a) = static_gets_x_y(l, text, ctx) {
        return Some(a);
    }
    // "During your turn, ~ is a 4/4 ... creature ...": a condition on the whole ability.
    let mut cond = None;
    let mut line = l.to_string();
    if let Some((c, r)) = l.split_once(", ") {
        if let Some(t) = super::statics::turn_condition(c) {
            cond = Some(t);
            line = r.to_string();
        } else if c == "as long as ~ is on the battlefield" {
            // A static ability of a permanent applies only on the battlefield anyway.
            line = r.replacen("it's ", "~ is ", 1);
        }
    }
    // "... that's still a planeswalker" (CR 205.1b).
    let mut still = false;
    for t in [" that's still a planeswalker", " that's still a land"] {
        if let Some(r) = line.strip_suffix(t) {
            line = r.to_string();
            still = true;
            break;
        }
    }
    let l = line.as_str();
    let (subj_text, preds) = static_parts(l)?;
    let subj = super::statics::parse_subject(subj_text, Some(&Sel::This), ctx)
        .or_else(|| either_adjective(subj_text, ctx))?;
    let mut b = Builder::new(ctx);
    let mut mods = Vec::new();
    STATIC_CONTEXT.with(|c| c.set(true));
    let parsed: Option<Vec<Vec<Modification>>> = preds
        .iter()
        .map(|p| static_predicate_mods(p, &subj, &mut b))
        .collect();
    STATIC_CONTEXT.with(|c| c.set(false));
    for m in parsed? {
        mods.extend(m);
    }
    if mods.is_empty() || !b.targets.is_empty() {
        return None;
    }
    if still {
        mods = super::planeswalkers_modal_becomes::retain_prior_types(mods)?;
    }
    if makes_creature(&mods)
        && !subj.creatures
        && !mods
            .iter()
            .any(|m| matches!(m, Modification::SetPT(Some(_), Some(_))))
    {
        return None;
    }
    let mut st = StaticAbility::new(StaticEffect::Continuous {
        affected: subj.filter,
        mods,
    });
    st.condition = cond;
    Some(vec![AbilityDef::new(AbilityKind::Static(st), text)])
}

inventory::submit! { StaticPattern { name: "becomes grammar: [objects] [characteristic predicates]", priority: 1100, parse: static_predicates } }

/// The zones of cards that aren't on the battlefield (or the stack).
const OFF_BATTLEFIELD: [ZoneKind; 5] = [
    ZoneKind::Hand,
    ZoneKind::Library,
    ZoneKind::Graveyard,
    ZoneKind::Exile,
    ZoneKind::Command,
];

/// Objects in several zones, one alternative per zone (a static ability affects the
/// objects it describes wherever they are, CR 611.3): "lands you control and land cards
/// you own that aren't on the battlefield", "nonland permanents you control and permanent
/// spells you control", "all cards that aren't on the battlefield, spells, and permanents".
fn zone_subject(s: &str) -> Option<(Vec<Filter>, Subject)> {
    let s = s.trim();
    let mut alts = Vec::new();
    let mut lands = true;
    let mut creatures = true;
    if s == "all cards that aren't on the battlefield, spells, and permanents" {
        for z in OFF_BATTLEFIELD {
            alts.push(Filter::and(vec![Filter::Card, Filter::InZone(z)]));
        }
        alts.push(Filter::InZone(ZoneKind::Stack));
        alts.push(Filter::InZone(ZoneKind::Battlefield));
        lands = false;
        creatures = false;
    } else {
        let parts: Vec<&str> = s.split(" and ").collect();
        if parts.len() < 2 {
            return None;
        }
        for part in parts {
            let part = part.trim();
            let (base, off) = match part.strip_suffix(" that aren't on the battlefield") {
                Some(r) => (r, true),
                None => (part, false),
            };
            let (f, _, tail) = crate::oracle::phrases::parse_object_phrase(base)?;
            if !end(tail).trim().is_empty() {
                return None;
            }
            lands &= super::statics::filter_mentions(&f, &|x| {
                matches!(x, Filter::Type(CardType::Land))
            });
            creatures &= super::statics::filter_mentions(&f, &|x| {
                matches!(x, Filter::Type(CardType::Creature))
            });
            if off {
                for z in OFF_BATTLEFIELD {
                    alts.push(Filter::and(vec![f.clone(), Filter::InZone(z)]));
                }
            } else if f.zone().is_some() {
                alts.push(f);
            } else {
                alts.push(Filter::and(vec![f, Filter::InZone(ZoneKind::Battlefield)]));
            }
        }
    }
    let subj = Subject {
        filter: Filter::Or(alts.clone()),
        it: None,
        hint: if lands {
            CardType::Land
        } else {
            CardType::Creature
        },
        lands,
        creatures,
    };
    Some((alts, subj))
}

/// "[objects in several zones] are [type words | colorless] [in addition to their other
/// types]" (Dune Chanter, Secret Arcade, Mycosynth Lattice).
fn zone_static(l: &str, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if ctx.is_spell() {
        return None;
    }
    let (subj_text, pred) = end(l).split_once(" are ")?;
    let (_, subj) = zone_subject(subj_text)?;
    let mods = color_mods(pred).or_else(|| type_predicate_mods(pred, &subj))?;
    // Only additions and colors: card types or subtypes set on cards would need their
    // kinds checked in each zone.
    if !mods.iter().all(|m| {
        matches!(
            m,
            Modification::AddTypes(_)
                | Modification::AddSubtypes(_)
                | Modification::AddSupertypes(_)
                | Modification::SetColors(_)
                | Modification::AddColors(_)
        )
    }) {
        return None;
    }
    let mut s = StaticAbility::new(StaticEffect::Continuous {
        affected: subj.filter,
        mods,
    });
    s.zone = FunctionZone::Battlefield;
    Some(vec![AbilityDef::new(AbilityKind::Static(s), text)])
}

inventory::submit! { StaticPattern { name: "becomes grammar: [objects in several zones] are [type words]", priority: 1100, parse: zone_static } }

/// The subjects that name what an earlier instruction of the effect is about.
fn is_referent_subject(s: &str) -> Option<&str> {
    for p in [
        "it's ",
        "it is ",
        "that creature is ",
        "that permanent is ",
        "each of those creatures is ",
        "those creatures are ",
        "they're ",
        "they are ",
    ] {
        if let Some(r) = s.strip_prefix(p) {
            return Some(r);
        }
    }
    None
}

/// "That creature is black and is a Nightmare in addition to its other creature types."
/// (Chainer): the predicates' modifications.
fn is_predicates(pred: &str, subj: &Subject, b: &mut Builder) -> Option<Vec<Modification>> {
    let pred = pred.replace(" and is ", " and is ");
    let mut mods = Vec::new();
    for p in split_predicates_with(&format!("is {pred}"), STATIC_VERBS) {
        mods.extend(static_predicate_mods(p, subj, b)?);
    }
    let ok = !mods.is_empty()
        && mods.iter().all(|m| {
            matches!(
                m,
                Modification::AddTypes(_)
                    | Modification::AddSubtypes(_)
                    | Modification::AddSupertypes(_)
                    | Modification::SetColors(_)
                    | Modification::AddColors(_)
            )
        });
    ok.then_some(mods)
}

/// The last instruction of `e`, if it puts objects onto the battlefield.
fn last_battlefield_move(e: &mut Effect) -> Option<&mut Destination> {
    match e {
        Effect::Move { to, .. } if to.zone == ZoneKind::Battlefield => Some(to),
        Effect::Seq(v) => v.last_mut().and_then(last_battlefield_move),
        Effect::If {
            then, otherwise, ..
        } if matches!(**otherwise, Effect::Noop) => last_battlefield_move(then),
        Effect::May { effect, .. } => last_battlefield_move(effect),
        // "When you do, return target creature card ... to the battlefield."
        Effect::Reflexive { body } => last_battlefield_move(&mut body.effect),
        _ => None,
    }
}

/// "Return target creature card from your graveyard to the battlefield. It's a Phyrexian
/// in addition to its other types." (CR 611.2e: the effect applies as the permanent
/// enters).
fn its_a_followup(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    let Some(pred) = is_referent_subject(l) else {
        return false;
    };
    if pred.contains('"') {
        return false;
    }
    let subj = Subject {
        filter: Filter::Any,
        it: None,
        hint: CardType::Creature,
        lands: false,
        creatures: true,
    };
    let Some(to) = last_battlefield_move(prev) else {
        return false;
    };
    if !to.with_mods.is_empty() {
        return false;
    }
    let Some(mods) = is_predicates(pred, &subj, b) else {
        return false;
    };
    to.with_mods = mods;
    true
}

inventory::submit! { FollowupPattern { name: "becomes grammar: it's [type words] (as it enters)", priority: 1100, apply: its_a_followup } }

/// "It's a Spirit in addition to its other types." after an instruction that put the
/// object onto the battlefield and another one about it ("Put a flying counter on it."):
/// the object is that from then on.
fn its_a_effect(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let pred = is_referent_subject(l)?;
    let subj_text = &l[..l.len() - pred.len()];
    let subj_text = subj_text
        .trim_end()
        .trim_end_matches("'s")
        .trim_end_matches(" is")
        .trim_end_matches(" are")
        .trim_end_matches("'re");
    let subj_text = match subj_text {
        "each of those creatures" => "those creatures",
        s => s,
    };
    let saved = (b.targets.len(), b.it.clone());
    let (what, rest) = object_ref(subj_text, b)?;
    if !rest.trim().is_empty()
        || b.targets.len() != saved.0
        || matches!(what, Sel::None | Sel::This)
        || b.sentences == 0
    {
        b.targets.truncate(saved.0);
        b.it = saved.1;
        return None;
    }
    let subj = Subject {
        filter: Filter::Any,
        it: None,
        hint: CardType::Creature,
        lands: false,
        creatures: true,
    };
    let mods = is_predicates(pred, &subj, b)?;
    Some(Effect::Modify {
        what,
        mods,
        duration: Duration::Permanent,
    })
}

inventory::submit! { EffectPattern { name: "becomes grammar: it's [type words]", priority: 1100, parse: its_a_effect } }

/// "Attacking creatures become blocked.", "X target attacking creatures become blocked."
/// (CR 509.1h).
fn objects_become_blocked(l: &str, b: &mut Builder) -> Option<Effect> {
    let subj = end(l).strip_suffix(" become blocked")?;
    let saved = b.targets.len();
    let (sel, rest) = object_ref(subj, b)?;
    if !rest.trim().is_empty() || matches!(sel, Sel::None | Sel::This) {
        b.targets.truncate(saved);
        return None;
    }
    Some(Effect::ForEach {
        sel,
        var: vars::AFFECTED,
        effect: Box::new(Effect::Custom(
            super::k702_banding::TARGET_BECOMES_BLOCKED.into(),
        )),
    })
}

inventory::submit! { EffectPattern { name: "becomes grammar: [objects] become blocked", priority: 1100, parse: objects_become_blocked } }

/// "switch its power and toughness until end of turn" (CR 613.4d).
fn switch_its_pt(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("switch its power and toughness")?;
    let duration = only_duration(r)?;
    let (what, _) = object_ref("it", b)?;
    Some(Effect::Modify {
        what,
        mods: vec![Modification::SwitchPT],
        duration,
    })
}

inventory::submit! { EffectPattern { name: "becomes grammar: switch its power and toughness", priority: 1100, parse: switch_its_pt } }

/// "Each player's life total becomes the number of creatures they control." (CR 119.5):
/// each player's own number.
fn each_life_becomes(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("each player's life total becomes ")?;
    if !r.contains("they ") && !r.contains("their ") {
        return None;
    }
    let saved = std::mem::replace(&mut b.it_player, PlayerRef::Iterated);
    let parsed = crate::oracle::statics::parse_value_phrase(r, b);
    b.it_player = saved;
    let (n, rest) = parsed?;
    if !rest.trim().is_empty() || !mentions_iterated(&n) {
        return None;
    }
    Some(Effect::ForEachPlayer {
        who: PlayerRef::EachPlayer,
        effect: Box::new(Effect::SetLife {
            who: PlayerRef::Iterated,
            n,
        }),
    })
}

fn mentions_iterated(v: &Value) -> bool {
    format!("{v:?}").contains("Iterated")
}

inventory::submit! { EffectPattern { name: "becomes grammar: each player's life total becomes [their number]", priority: 1100, parse: each_life_becomes } }

/// "you may have the base power and toughness of [object] become [N/N | X/X | equal to
/// ~'s power [and toughness]] until end of turn[, where X is ~'s power]", "the base power
/// and toughness of [objects] become 0/2" (layer 7b; values determined as the effect
/// begins, CR 608.2h).
fn base_pt_of_become(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (l, x_def) = match l.split_once(", where x is ") {
        Some((a, x)) => (a, Some(x)),
        None => (l, None),
    };
    let r = l.strip_prefix("have ").unwrap_or(l);
    let r = r.strip_prefix("the base power and toughness of ")?;
    let (duration, r) = crate::oracle::effects::duration_suffix(r);
    let (subj, value) = r.rsplit_once(" become ")?;
    let saved = (b.targets.len(), b.it.clone());
    let result = (|| {
        let (what, rest) = object_ref(subj, b)?;
        if !rest.trim().is_empty() || matches!(what, Sel::None) {
            return None;
        }
        let (p, t) = if let Some(v) = value.strip_prefix("equal to ") {
            let (p, t) = if let Some(o) = v.strip_suffix("'s power and toughness") {
                (o, true)
            } else {
                (v.strip_suffix("'s power")?, false)
            };
            let (src, rest) = object_ref(p, b)?;
            if !rest.is_empty() {
                return None;
            }
            let pw = Value::PowerOf(Box::new(src.clone()));
            let tg = if t {
                Value::ToughnessOf(Box::new(src))
            } else {
                pw.clone()
            };
            (pw, tg)
        } else if value == "x/x" {
            let (x, rest) = crate::oracle::statics::parse_value_phrase(x_def?, b)?;
            if !rest.trim().is_empty() {
                return None;
            }
            (x.clone(), x)
        } else {
            if x_def.is_some() {
                return None;
            }
            pt_values(value)?
        };
        b.it = what.clone();
        Some(Effect::Modify {
            what,
            mods: vec![Modification::SetPT(Some(p), Some(t))],
            duration,
        })
    })();
    if result.is_none() {
        b.targets.truncate(saved.0);
        b.it = saved.1;
    }
    result
}

inventory::submit! { EffectPattern { name: "becomes grammar: the base power and toughness of [objects] become ...", priority: 1100, parse: base_pt_of_become } }

/// "Until your next turn, ~'s base power becomes twice that card's power and its base
/// toughness becomes twice that card's toughness." (Amplifire)
fn base_power_and_its_toughness(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (duration, core) = split_duration(l);
    let (a, c) = core.split_once(" and its base toughness becomes ")?;
    let (subj, pv) = a.split_once("'s base power becomes ")?;
    let saved = (b.targets.len(), b.it.clone());
    let result = (|| {
        let (what, rest) = object_ref(subj, b)?;
        if !rest.is_empty() || matches!(what, Sel::None) {
            return None;
        }
        let (p, r1) = crate::oracle::statics::parse_value_phrase(pv, b)?;
        let (t, r2) = crate::oracle::statics::parse_value_phrase(c, b)?;
        if !r1.trim().is_empty() || !r2.trim().is_empty() || b.targets.len() != saved.0 {
            return None;
        }
        Some(Effect::Modify {
            what,
            mods: vec![Modification::SetPT(Some(p), Some(t))],
            duration,
        })
    })();
    if result.is_none() {
        b.targets.truncate(saved.0);
        b.it = saved.1;
    }
    result
}

inventory::submit! { EffectPattern { name: "becomes grammar: base power becomes ... and its base toughness becomes ...", priority: 1100, parse: base_power_and_its_toughness } }

/// "[A] and they gain trample", "[A] and it gains flying", where [A] names power and
/// toughness ("~'s base power and toughness become 6/6"): split at the last "and".
fn and_pronoun_predicate(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (lead, core) = match l.strip_prefix("until end of turn, ") {
        Some(r) => (" until end of turn", r),
        None => ("", l),
    };
    let (a, c) = core.rsplit_once(" and ")?;
    if !a.contains("power and toughness") || !matches!(c.split(' ').next(), Some("they" | "it")) {
        return None;
    }
    let saved = (b.targets.clone(), b.it.clone());
    let first = crate::oracle::effects::parse_simple(&format!("{a}{lead}"), b);
    let second = first
        .as_ref()
        .and_then(|_| crate::oracle::effects::parse_simple(&format!("{c}{lead}"), b));
    match (first, second) {
        (Some(x), Some(y)) => Some(Effect::seq(vec![x, y])),
        _ => {
            b.targets = saved.0;
            b.it = saved.1;
            None
        }
    }
}

inventory::submit! { EffectPattern { name: "becomes grammar: [power and toughness] and they gain ...", priority: 1100, parse: and_pronoun_predicate } }

/// "Creatures that are green and/or white get an additional -2/-2 until end of turn.":
/// "additional" only says it's another effect.
fn get_an_additional(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    if !l.contains(" get an additional ") && !l.contains(" gets an additional ") {
        return None;
    }
    let t = l
        .replacen(" get an additional ", " get ", 1)
        .replacen(" gets an additional ", " gets ", 1);
    crate::oracle::effects::parse_clause(&t, b)
}

inventory::submit! { EffectPattern { name: "becomes grammar: get an additional +N/+N", priority: 1100, parse: get_an_additional } }

/// "Target creature gains protection from the color of its controller's choice until end
/// of turn." (CR 608.2d: that player chooses as the ability resolves).
fn color_of_controllers_choice(l: &str, b: &mut Builder) -> Option<Effect> {
    const PHRASE: &str = "the color of its controller's choice";
    let l = end(l);
    if l.matches(PHRASE).count() != 1 {
        return None;
    }
    let t = l.replacen(PHRASE, "the chosen color", 1);
    let saved = (b.targets.len(), b.it.clone());
    let e = crate::oracle::effects::parse_sentence(&t, b)?;
    let who = match &b.it {
        Sel::Target(i) if b.targets.len() == saved.0 + 1 => {
            PlayerRef::ControllerOf(Box::new(Sel::Target(*i)))
        }
        _ => {
            b.targets.truncate(saved.0);
            b.it = saved.1;
            return None;
        }
    };
    Some(Effect::seq(vec![
        Effect::Choose {
            who,
            kind: ChoiceKind::Color,
        },
        e,
    ]))
}

inventory::submit! { EffectPattern { name: "becomes grammar: the color of its controller's choice", priority: 1100, parse: color_of_controllers_choice } }

/// "It's still an enchantment." after a sentence that made the object a creature
/// (CR 205.1b): it keeps its other types.
fn still_a_card_type(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    let Some(t) = end(l)
        .strip_prefix("it's still an ")
        .or_else(|| end(l).strip_prefix("it's still a "))
    else {
        return false;
    };
    if !matches!(t, "enchantment" | "artifact") {
        return false;
    }
    let Some(mods) = last_modify(prev) else {
        return false;
    };
    if !makes_creature(mods) {
        return false;
    }
    match super::planeswalkers_modal_becomes::retain_prior_types(std::mem::take(mods)) {
        Some(m) => {
            *mods = m;
            true
        }
        None => false,
    }
}

fn last_modify(e: &mut Effect) -> Option<&mut Vec<Modification>> {
    match e {
        Effect::Modify { mods, .. } => Some(mods),
        Effect::Seq(v) => v.last_mut().and_then(last_modify),
        Effect::May { effect, .. } => last_modify(effect),
        Effect::If {
            then, otherwise, ..
        } if matches!(**otherwise, Effect::Noop) => last_modify(then),
        _ => None,
    }
}

inventory::submit! { FollowupPattern { name: "becomes grammar: it's still an enchantment", priority: 1100, apply: still_a_card_type } }

/// "You may pay {W} to end this effect." after an instruction that created a continuous
/// effect (the Licids): its controller may take a special action any time they have
/// priority, for as long as the effect lasts, that ends it (CR 116.2c).
fn pay_to_end_this_effect(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    let Some(cost) = end(l)
        .strip_prefix("you may pay ")
        .and_then(|r| r.strip_suffix(" to end this effect"))
    else {
        return false;
    };
    let Some(mana) = crate::mana::ManaCost::parse(cost) else {
        return false;
    };
    // The effect it ends: the earlier instruction of this ability that made the source
    // lose this ability (which marks the effect, see `kw::end_this_effect`).
    let raw = crate::oracle::raw_text().to_lowercase();
    if !raw
        .lines()
        .any(|x| x.contains("loses this ability and becomes ") && x.contains(" to end this effect"))
    {
        return false;
    }
    let offer = Effect::OfferSpecialAction {
        def: Box::new(SpecialActionDef {
            who: PlayerFilter::You,
            cost: Cost::mana(mana),
            action: SpecialActionEffect::Effect(Effect::Custom(
                crate::kw::end_this_effect::END_THIS_EFFECT.into(),
            )),
            mana_timing: false,
        }),
        duration: Duration::WhileSourceOnBattlefield,
        repeatable: false,
    };
    let old = std::mem::replace(prev, Effect::Noop);
    *prev = Effect::seq(vec![old, offer]);
    true
}

inventory::submit! { FollowupPattern { name: "becomes grammar: you may pay [cost] to end this effect", priority: 1100, apply: pay_to_end_this_effect } }

#[cfg(test)]
mod tests {
    use super::split_predicates;

    #[test]
    fn splits_at_verbs_only() {
        assert_eq!(
            split_predicates(
                "has base power and toughness 5/3, gains trample, and isn't a human"
            ),
            vec!["has base power and toughness 5/3", "gains trample", "isn't a human"]
        );
        assert_eq!(
            split_predicates("gets +1/+0, becomes black, and gains shadow"),
            vec!["gets +1/+0", "becomes black", "gains shadow"]
        );
        assert_eq!(
            split_predicates("becomes a dragon with base power and toughness 4/4, flying, and haste"),
            vec!["becomes a dragon with base power and toughness 4/4, flying, and haste"]
        );
    }
}
