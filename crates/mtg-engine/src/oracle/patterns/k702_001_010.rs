//! Oracle patterns for the keyword abilities of CR 702.1–702.10 and the effects that
//! grant, modify, or refer to them:
//!
//! * "[This] has double strike as long as you have no cards in hand" — a static ability
//!   with a trailing condition, the same as "As long as [condition], [this] has ...".
//! * "Instant and sorcery spells you control have deathtouch" (keywords on spells,
//!   CR 702.2d).
//! * "Equip costs you pay cost {1} less" / "Equip abilities you activate cost {1} less to
//!   activate" (CR 702.1a: a "[keyword] cost" is the keyword's own cost).
//! * Self-state conditions: "it's attacking", "~ is equipped", ...
//! * "[Creatures] can attack as though they didn't have defender" (CR 702.3b), as a static
//!   ability or for a turn.

use super::{AbilityPattern, ConditionPattern, EffectPattern, StaticPattern};
use crate::oracle::effects::Builder;
use crate::ability::*;
use crate::keywords::KeywordKind;
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;
use crate::types::CardType;

fn with_text(v: Vec<Ability>, text: &str) -> Vec<Ability> {
    v.into_iter()
        .map(|a| AbilityDef::with_link(a.kind.clone(), text, a.link))
        .collect()
}

/// "[subject] has [keywords] as long as [condition]" → "As long as [condition],
/// [subject] has [keywords]". (An ability pattern, since the static parser commits to
/// "~ has ..." lines before trying pluggable static patterns.)
fn trailing_as_long_as(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if block.contains('\n') || block.contains(':') || block.contains('"') {
        return None;
    }
    let lower = block.to_lowercase();
    let l = end(&lower);
    if l.starts_with("as long as ")
        || l.starts_with("when")
        || l.starts_with("at ")
        || l.contains(". ")
    {
        return None;
    }
    let (head, cond) = l.split_once(" as long as ")?;
    if cond.contains(" as long as ")
        || !(head.contains(" has ")
            || head.contains(" have ")
            || head.contains(" gets ")
            || head.contains(" get ")
            || head.contains(" can "))
    {
        return None;
    }
    let reordered = format!("as long as {cond}, {head}");
    let v = crate::oracle::statics::parse_static(&reordered, ctx)?;
    // Only accept it if the condition was attached to every ability.
    if v.is_empty()
        || !v
            .iter()
            .all(|a| matches!(&a.kind, AbilityKind::Static(s) if s.condition.is_some()))
    {
        return None;
    }
    let v: Vec<Ability> = v.into_iter().map(|a| conditional_flash_zone(&a)).collect();
    // On instants and sorceries, only "this spell has flash as long as ..." is a static.
    if ctx.is_spell()
        && !v.iter().all(
            |a| matches!(&a.kind, AbilityKind::Static(s) if s.zone == FunctionZone::Anywhere),
        )
    {
        return None;
    }
    Some(with_text(v, block))
}

/// "[This] has flash as long as ..." modifies how the object can be cast, so it functions
/// in every zone it could be cast from and on the stack (CR 113.6e, 601.3d, 702.8a).
fn conditional_flash_zone(a: &Ability) -> Ability {
    if let AbilityKind::Static(s) = &a.kind {
        if let StaticEffect::Continuous { affected, mods } = &s.effect {
            if matches!(affected, Filter::Source)
                && !mods.is_empty()
                && mods.iter().all(
                    |m| matches!(m, Modification::AddKeyword(k) if k.kind == KeywordKind::Flash),
                )
            {
                let mut s = s.clone();
                s.zone = FunctionZone::Anywhere;
                return AbilityDef::with_link(AbilityKind::Static(s), a.text.clone(), a.link);
            }
        }
    }
    a.clone()
}

inventory::submit! {
    AbilityPattern { name: "k702: trailing as long as", priority: 50, parse: trailing_as_long_as }
}

/// Parses a list of keywords ("deathtouch", "flying and first strike") into keyword
/// modifications.
pub fn keyword_mods(s: &str) -> Option<Vec<Modification>> {
    let s = end(s);
    let tl = crate::types::TypeLine::default();
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
    let mut out = Vec::new();
    for p in s
        .split(", and ")
        .flat_map(|p| p.split(" and "))
        .flat_map(|p| p.split(", "))
        .map(str::trim)
        .filter(|p| !p.is_empty())
    {
        for a in crate::oracle::keywords::parse_keyword_line(p, &ctx)? {
            if let AbilityKind::Keyword(k) = &a.kind {
                out.push(Modification::AddKeyword(k.clone()));
            }
        }
    }
    (!out.is_empty()).then_some(out)
}

/// "Instant and sorcery spells you control have deathtouch." — a continuous effect on
/// spells on the stack (CR 611.3, 702.2d).
fn spells_have_keywords(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let (subject, kws) = l.split_once(" spells you control have ")?;
    let mut types = Vec::new();
    for w in subject
        .split(" and ")
        .flat_map(|p| p.split(" or "))
        .flat_map(|p| p.split(", "))
        .map(str::trim)
        .filter(|w| !w.is_empty() && *w != "and/or")
    {
        types.push(Filter::Type(CardType::from_word(w)?));
    }
    if types.is_empty() {
        return None;
    }
    let mods = keyword_mods(kws)?;
    let affected = Filter::and(vec![
        Filter::Or(types),
        Filter::Spell,
        Filter::ControlledBy(PlayerRel::You),
    ]);
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Continuous {
            affected,
            mods,
        })),
        text,
    )])
}

inventory::submit! {
    StaticPattern { name: "k702: spells you control have keywords", priority: 50, parse: spells_have_keywords }
}

/// "Equip costs you pay cost {1} less." / "Equip abilities you activate cost {1} less to
/// activate." (CR 702.1a).
fn keyword_cost_modifier(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let (kw, rest) = if let Some(r) = l.strip_prefix("equip costs you pay cost ") {
        (KeywordKind::Equip, r)
    } else if let Some(r) = l.strip_prefix("equip abilities you activate cost ") {
        (KeywordKind::Equip, r.strip_suffix(" to activate").unwrap_or(r))
    } else {
        return None;
    };
    let (amount, tail) = rest.split_once('}')?;
    let n: i32 = amount.strip_prefix('{')?.parse().ok()?;
    let change = match end(tail).trim() {
        "less" => CostChange::ReduceGeneric(Value::c(n)),
        "more" => CostChange::IncreaseGeneric(Value::c(n)),
        _ => return None,
    };
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::CostModifier(
            CostModifier {
                applies_to: CostTarget::Keyword(kw),
                who: PlayerRel::You,
                change,
            },
        ))),
        text,
    )])
}

inventory::submit! {
    StaticPattern { name: "k702: keyword cost modifiers", priority: 50, parse: keyword_cost_modifier }
}

/// Conditions about the object itself: "it's attacking", "~ is equipped", "it's
/// enchanted or equipped".
fn self_state_condition(c: &str) -> Option<Condition> {
    let c = end(c);
    let state = c
        .strip_prefix("it's ")
        .or_else(|| c.strip_prefix("it is "))
        .or_else(|| c.strip_prefix("~ is "))?;
    let f = match state {
        "attacking" => Filter::Attacking,
        "blocking" => Filter::Blocking,
        "attacking or blocking" => Filter::Or(vec![Filter::Attacking, Filter::Blocking]),
        "enchanted" => Filter::Enchanted,
        "equipped" => Filter::Equipped,
        "enchanted or equipped" | "equipped or enchanted" => {
            Filter::Or(vec![Filter::Enchanted, Filter::Equipped])
        }
        "modified" => Filter::Modified,
        "untapped" => Filter::Untapped,
        "tapped" => Filter::Tapped,
        _ => return None,
    };
    Some(Condition::SelMatches(Sel::This, f))
}

inventory::submit! {
    ConditionPattern { name: "k702: self state", priority: 150, parse: self_state_condition }
}

const DESPITE_DEFENDER: [&str; 2] = [
    " as though it didn't have defender",
    " as though they didn't have defender",
];

/// Strips "as though it/they didn't have defender" from the end of a clause.
fn strip_despite_defender(l: &str) -> Option<&str> {
    DESPITE_DEFENDER.iter().find_map(|s| l.strip_suffix(s))
}

/// The objects a static ability's subject refers to: "~", "it", "enchanted Wall",
/// "creatures you control", "Wall creatures".
fn static_subject(s: &str) -> Option<Filter> {
    let s = s.trim();
    match s {
        "~" | "it" => return Some(Filter::Source),
        "enchanted creature" | "equipped creature" | "enchanted wall" => {
            return Some(Filter::AttachedToSource)
        }
        _ => {}
    }
    let s = s.strip_prefix("each ").unwrap_or(s);
    let (f, _, tail) = parse_object_phrase(s)?;
    end(tail).is_empty().then_some(f)
}

/// "~ can attack as though it didn't have defender", "Creatures you control can attack
/// as though they didn't have defender", "~ gets +2/+2 and can attack as though it didn't
/// have defender".
fn attack_despite_defender_static(
    l: &str,
    text: &str,
    ctx: &CompileContext,
) -> Option<Vec<Ability>> {
    let l = end(l);
    let head = strip_despite_defender(l)?;
    let restriction = |f: Filter| {
        AbilityDef::new(
            AbilityKind::Static(StaticAbility::new(StaticEffect::Restriction(
                Restriction::AttackDespiteDefender(f),
            ))),
            text,
        )
    };
    if let Some(subject) = head.strip_suffix(" can attack") {
        return Some(vec![restriction(static_subject(subject)?)]);
    }
    // "[subject] gets +2/+2 and can attack ...": the rest of the ability, then this.
    let rest = head.strip_suffix(" and can attack")?;
    let subject = [" gets ", " has ", " get ", " have "]
        .iter()
        .find_map(|v| rest.split_once(v).map(|(s, _)| s))?;
    let f = static_subject(subject)?;
    let mut v = crate::oracle::statics::parse_static(rest, ctx)?;
    if v.is_empty() || !v.iter().all(|a| matches!(a.kind, AbilityKind::Static(_))) {
        return None;
    }
    v = with_text(v, text);
    v.push(restriction(f));
    Some(v)
}

inventory::submit! {
    StaticPattern { name: "k702: attack despite defender", priority: 50, parse: attack_despite_defender_static }
}

/// "~ can attack this turn as though it didn't have defender", "target creature with
/// defender can attack this turn as though it didn't have defender", "creatures you
/// control with defender can attack this turn as though they didn't have defender".
fn attack_despite_defender_effect(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (l, until_eot) = match l.strip_prefix("until end of turn, ") {
        Some(r) => (r, true),
        None => (l, false),
    };
    let head = strip_despite_defender(l)?;
    let subject = head
        .strip_suffix(" can attack this turn")
        .or_else(|| head.strip_suffix(" can attack").filter(|_| until_eot))
        .or_else(|| {
            // Subjectless second half: "~ gets +3/-1 until end of turn and can attack
            // this turn as though it didn't have defender".
            (head == "can attack this turn").then_some("")
        })?
        .trim();
    let sel = match subject {
        "" | "it" | "that creature" => b.it.clone(),
        "~" => Sel::This,
        s if s.contains("target ") => {
            let (spec, tail) = parse_target(s)?;
            if !end(tail).is_empty() {
                return None;
            }
            let slot = b.add_target(spec, s);
            Sel::Target(slot)
        }
        s => {
            let s = s.strip_prefix("each ").unwrap_or(s);
            let (f, _, tail) = parse_object_phrase(s)?;
            if !end(tail).is_empty() {
                return None;
            }
            Sel::All(f)
        }
    };
    Some(Effect::AddRestriction {
        restriction: Restriction::AttackDespiteDefender(Filter::In(Box::new(sel))),
        duration: Duration::EndOfTurn,
    })
}

inventory::submit! {
    EffectPattern { name: "k702: attack despite defender this turn", priority: 50, parse: attack_despite_defender_effect }
}

/// "You may cast creature spells from the top of your library." / "You may play lands and
/// cast spells from the top of your library." — permissions to play cards from another
/// zone; the cards' own timing rules (including flash, CR 702.8a) still apply.
fn play_from_top_of_library(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let r = l
        .strip_prefix("you may ")?
        .strip_suffix(" from the top of your library")?;
    let (lands, spells_part) = if let Some(s) = r.strip_prefix("play lands and cast ") {
        (true, Some(s))
    } else if r == "play lands" {
        (true, None)
    } else {
        (false, Some(r.strip_prefix("cast ")?))
    };
    let what = match spells_part {
        None => Filter::Type(CardType::Land),
        Some("spells") => Filter::Any,
        Some(s) => {
            let mut fs = Vec::new();
            for part in s.split(" and ").flat_map(|p| p.split(" or ")) {
                let (f, _, tail) = parse_object_phrase(part.trim())?;
                if !end(tail).is_empty() {
                    return None;
                }
                fs.push(f);
            }
            if fs.len() == 1 {
                fs.pop()?
            } else {
                Filter::Or(fs)
            }
        }
    };
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::PlayPermission(
            PlayPermission {
                who: PlayerRel::You,
                zone: ZoneKind::Library,
                top_only: true,
                what,
                lands,
                spells: spells_part.is_some(),
                cost: None,
            },
        ))),
        text,
    )])
}

inventory::submit! {
    StaticPattern { name: "k702: play from the top of your library", priority: 50, parse: play_from_top_of_library }
}

/// "As long as [condition], ~ gets +2/+2 and can attack as though it didn't have
/// defender": the two halves as separate conditional statics.
fn conditional_and_attack_despite_defender(
    block: &str,
    ctx: &CompileContext,
) -> Option<Vec<Ability>> {
    if ctx.is_spell() || block.contains('\n') || block.contains(':') {
        return None;
    }
    let lower = block.to_lowercase();
    let l = end(&lower);
    let r = l.strip_prefix("as long as ")?;
    let (cond, rest) = r.split_once(", ")?;
    let head = strip_despite_defender(rest)?;
    let first = head.strip_suffix(" and can attack")?;
    let subject = [" gets ", " has ", " get ", " have "]
        .iter()
        .find_map(|v| first.split_once(v).map(|(s, _)| s))?;
    let mut v =
        crate::oracle::statics::parse_static(&format!("as long as {cond}, {first}"), ctx)?;
    v.extend(crate::oracle::statics::parse_static(
        &format!("as long as {cond}, {subject} can attack as though it didn't have defender"),
        ctx,
    )?);
    if !v
        .iter()
        .all(|a| matches!(&a.kind, AbilityKind::Static(s) if s.condition.is_some()))
    {
        return None;
    }
    Some(with_text(v, block))
}

inventory::submit! {
    AbilityPattern { name: "k702: conditional and attack despite defender", priority: 50, parse: conditional_and_attack_despite_defender }
}
