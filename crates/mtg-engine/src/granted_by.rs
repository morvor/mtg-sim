//! Abilities that name the object granting them: "Equipped creature has \"{T}, Sacrifice
//! Blazing Torch: Blazing Torch deals 2 damage to any target.\"" (CR 113.7: the
//! equipped creature's ability, whose source is the creature; the name still means
//! Blazing Torch, CR 201.5). The compiler writes the card as
//! `Filter::Custom(GRANTER)`; when an object acquires the ability from another object
//! (`layers::acquired_ability`), that becomes the granting object itself. "Sacrifice
//! [it]" in the cost sacrifices that object (which its controller must control), and the
//! sacrificed object, as it last existed, is what "[it] deals damage" refers to
//! (CR 608.2h).

use crate::ability::*;
use crate::types::ObjectId;

/// `Filter::Custom` standing for the object that grants the ability.
pub const GRANTER: &str = "granted_by:source";

/// Rewrites an ability compiled from a quote whose every self-reference is the granting
/// card: a cost "sacrifice ~" sacrifices the granter, and "~" in the effect is the
/// sacrificed granter (or the granter, for an ability that doesn't sacrifice it). Only
/// activated abilities, whose source is otherwise never referred to.
pub fn refer_to_granter(a: &Ability) -> Option<Ability> {
    refer_to_granter_in(a, true)
}

/// [`refer_to_granter`] for an ability that names the granting card only in its cost
/// ("{T}, Unattach Surestrike Trident: This creature deals ..."): "~" in its effect is
/// the object that has the ability.
pub fn refer_cost_to_granter(a: &Ability) -> Option<Ability> {
    refer_to_granter_in(a, false)
}

fn refer_to_granter_in(a: &Ability, in_body: bool) -> Option<Ability> {
    rewrite(
        a,
        if in_body {
            Rewrite::Names
        } else {
            Rewrite::CostOnly
        },
    )
}

/// [`refer_to_granter`] for an ability compiled from the granting object's point of view
/// (see `oracle::patterns::statics`): a quote that names the granting card and also says
/// "this creature" ("Whenever this creature attacks, you may sacrifice Spare Dagger. When
/// you do, this creature deals 1 damage to any target.") is compiled with "this creature"
/// read as "enchanted creature"; here "~" becomes the granter and "enchanted creature"
/// the object that has the ability (CR 113.7, 201.5a).
pub fn refer_mixed_to_granter(a: &Ability) -> Option<Ability> {
    rewrite(a, Rewrite::Mixed)
}

#[derive(Clone, Copy, PartialEq)]
enum Rewrite {
    /// Every "~" is the granter.
    Names,
    /// Only "~" in the cost is the granter.
    CostOnly,
    /// "~" is the granter and "enchanted creature" the object that has the ability.
    Mixed,
}

/// Constructs that refer to the object an ability is on in ways this rewriting doesn't
/// translate: such quotes are left unsupported.
const UNTRANSLATED: [&str; 8] = [
    "\"AttachedToThis\"",
    "\"BlockingSource\"",
    "\"BlockedBySource\"",
    "\"WhileSourceOnBattlefield\"",
    "\"WhileYouControlSource\"",
    "\"ExertSelf\"",
    "\"AddThisAbility\"",
    "attached_to_host",
];

fn rewrite(a: &Ability, how: Rewrite) -> Option<Ability> {
    let granter_all = Sel::All(Filter::Custom(GRANTER.into()));
    let this = json(&Sel::This)?;
    let source = json(&Filter::Source)?;
    let all_granter = json(&granter_all)?;
    let granter_filter = json(&Filter::Custom(GRANTER.into()))?;
    let attached_to = json(&Sel::AttachedTo)?;
    let attached_to_source = json(&Filter::AttachedToSource)?;
    // "other than ~": other than the granter (`Filter::Other` is other than the object
    // that has the ability).
    let other = json(&Filter::Other)?;
    let not_granter = json(&Filter::Not(Box::new(Filter::Custom(GRANTER.into()))))?;
    let granterize = |v: serde_json::Value, sel: &serde_json::Value| -> serde_json::Value {
        let v = replace(v, &this, sel);
        let v = replace(v, &source, &granter_filter);
        replace(v, &other, &not_granter)
    };
    // After "~" became the granter: "enchanted creature" is the object with the ability.
    let host_to_self = |v: serde_json::Value| -> serde_json::Value {
        if how != Rewrite::Mixed {
            return v;
        }
        let v = replace(v, &attached_to, &this);
        replace(v, &attached_to_source, &source)
    };
    let kind = match &a.kind {
        AbilityKind::Activated(act) => {
            let mut act = act.clone();
            let mut sacrifices = false;
            let mut removed_kinds = Vec::new();
            for p in act.cost.parts.iter_mut() {
                match p {
                    CostPart::SacrificeSelf => {
                        *p = CostPart::Sacrifice {
                            filter: Filter::Custom(GRANTER.into()),
                            count: Value::c(1),
                        };
                        sacrifices = true;
                    }
                    // "{15}, Exile The Dominion Bracelet: ..." (the granter, which its
                    // controller must control, like a sacrificed one).
                    CostPart::ExileSelf => {
                        *p = CostPart::Exile {
                            filter: Filter::Custom(GRANTER.into()),
                            zone: ZoneKind::Battlefield,
                            count: Value::c(1),
                        };
                    }
                    // "{T}, Unattach Heartseeker: ..." (CR 701.3d): the granting Equipment
                    // is unattached, not the creature.
                    CostPart::Effect(_) => {
                        let v = serde_json::to_value(&*p).ok()?;
                        let v = replace(v, &this, &all_granter);
                        *p = serde_json::from_value(host_to_self(v)).ok()?;
                    }
                    // Other costs naming "~" (discarding, exerting or returning it) aren't
                    // supported here. ({T} is the creature's own: it's the source of the
                    // ability.)
                    // "Remove all aim counters from Hankyu": from the granter.
                    CostPart::RemoveCounters { kind, count } => {
                        let count = serde_json::to_value(&*count).ok()?;
                        let count = replace(count, &this, &all_granter);
                        removed_kinds.push(kind.clone());
                        *p = CostPart::RemoveCountersFromAmong {
                            kind: Some(kind.clone()),
                            filter: Filter::Custom(GRANTER.into()),
                            count: serde_json::from_value(host_to_self(count)).ok()?,
                        };
                    }
                    CostPart::DiscardSelf
                    | CostPart::ReturnSelfToHand
                    | CostPart::ExertSelf
                    | CostPart::AddCounters { .. } => return None,
                    _ => {}
                }
            }
            if how == Rewrite::CostOnly {
                // "Remove all aim counters from Hankyu": all of the granter's.
                for k in &removed_kinds {
                    let from = json(&Value::CountersOn(Box::new(Sel::This), Some(k.clone())))?;
                    let to = json(&Value::CountersOn(
                        Box::new(granter_all.clone()),
                        Some(k.clone()),
                    ))?;
                    let cond = replace(serde_json::to_value(&act.condition).ok()?, &from, &to);
                    act.condition = serde_json::from_value(cond).ok()?;
                }
                AbilityKind::Activated(act)
            } else {
                let granter = if sacrifices {
                    Sel::Var(vars::SACRIFICED)
                } else {
                    granter_all.clone()
                };
                let to = serde_json::to_value(&granter).ok()?;
                let body = serde_json::to_value(&act.body).ok()?;
                let body = granterize(body, &to);
                act.body = serde_json::from_value(host_to_self(body)).ok()?;
                let cond = serde_json::to_value(&act.condition).ok()?;
                let cond = granterize(cond, &all_granter);
                act.condition = serde_json::from_value(host_to_self(cond)).ok()?;
                AbilityKind::Activated(act)
            }
        }
        AbilityKind::Triggered(_) | AbilityKind::Static(_) if how != Rewrite::CostOnly => {
            let v = serde_json::to_value(&a.kind).ok()?;
            serde_json::from_value(host_to_self(granterize(v, &all_granter))).ok()?
        }
        _ => return None,
    };
    let s = serde_json::to_string(&kind).ok()?;
    if UNTRANSLATED.iter().any(|m| s.contains(m)) {
        return None;
    }
    // Compiled from the granter's point of view, "enchanted creature" must have meant the
    // object it's attached to, not any enchanted or equipped creature.
    if how == Rewrite::Mixed && (s.contains("\"Enchanted\"") || s.contains("\"Equipped\"")) {
        return None;
    }
    Some(AbilityDef::with_link(kind, a.text.clone(), a.link))
}

fn json<T: serde::Serialize>(x: &T) -> Option<serde_json::Value> {
    serde_json::to_value(x).ok()
}

fn replace(
    v: serde_json::Value,
    from: &serde_json::Value,
    to: &serde_json::Value,
) -> serde_json::Value {
    use serde_json::Value as J;
    if &v == from {
        return to.clone();
    }
    match v {
        J::Array(a) => J::Array(a.into_iter().map(|x| replace(x, from, to)).collect()),
        J::Object(m) => J::Object(
            m.into_iter()
                .map(|(k, x)| (k, replace(x, from, to)))
                .collect(),
        ),
        other => other,
    }
}

/// The ability `a` as acquired from `granter`: its granter is that object.
pub fn bind_granter(a: &Ability, granter: ObjectId) -> Ability {
    let Ok(json) = serde_json::to_value(&a.kind) else {
        return a.clone();
    };
    let placeholder = serde_json::to_value(Filter::Custom(GRANTER.into())).ok();
    let concrete = serde_json::to_value(Filter::Objects(vec![granter])).ok();
    let (Some(placeholder), Some(concrete)) = (placeholder, concrete) else {
        return a.clone();
    };
    match serde_json::from_value::<AbilityKind>(replace(json, &placeholder, &concrete)) {
        Ok(kind) => AbilityDef::with_link(kind, a.text.clone(), a.link),
        Err(_) => a.clone(),
    }
}

/// Whether `sel` is the granter of an acquired ability, as [`bind_granter`] wrote it.
pub fn is_bound_granter(sel: &Sel) -> bool {
    matches!(sel, Sel::All(Filter::Objects(v)) if v.len() == 1)
}

/// Whether an ability refers to its granter.
pub fn refers_to_granter(a: &Ability) -> bool {
    serde_json::to_string(&a.kind).is_ok_and(|s| s.contains(GRANTER))
}

/// How a quoted ability of a token names the card that creates it.
pub enum TokenQuote {
    /// Only in "card named [card name]": the name (CR 201.2; the quote as it is, whose
    /// "card named ~" is compiled as the name, see `card_flow_search`).
    Named(String),
    /// "This token's power and toughness are each equal to the number of slime counters
    /// on Gutter Grime.": the characteristic-defining ability refers to the object that
    /// created the token (CR 201.5a: "each Ooze token remembers which one created it").
    Creator,
}

/// How the quoted ability `normalized` (of a token the card being compiled creates) names
/// that card, if it's a way this module handles.
pub fn token_quote_naming(
    normalized: &str,
    ctx: &crate::oracle::CompileContext,
) -> Option<TokenQuote> {
    let raw = crate::oracle::raw_text();
    let bare = |s: &str| s.trim().trim_end_matches(',').to_string();
    let q = quoted(&raw)
        .into_iter()
        .find(|q| bare(&crate::oracle::normalize(q, ctx)) == bare(normalized))?;
    let tl = crate::types::TypeLine::default();
    let anonymous = crate::oracle::CompileContext {
        card_name: "\u{1}",
        full_name: "\u{1}",
        type_line: &tl,
        layout: crate::card::Layout::Normal,
        face_index: 0,
        keywords: &[],
        power: None,
        toughness: None,
    };
    let names_card =
        |s: &str| crate::oracle::normalize(s, ctx) != crate::oracle::normalize(s, &anonymous);
    // "named [card name]" (CR 201.2): kept as the name.
    let named = format!("named {}", ctx.card_name);
    if q.contains(&named) {
        let marked = q.replace(&named, "named \u{2}");
        if names_card(&marked) {
            return None;
        }
        return Some(TokenQuote::Named(bare(normalized)));
    }
    // A characteristic-defining ability whose value names the creating card.
    for subject in ["This token's", "This creature's"] {
        let Some(rest) = q.strip_prefix(subject) else {
            continue;
        };
        let rest = rest.strip_prefix(" power and toughness are each equal to ")?;
        if crate::oracle::normalize(rest, &anonymous).contains('~') {
            return None;
        }
        return Some(TokenQuote::Creator);
    }
    None
}

/// The quoted segments of a text.
fn quoted(text: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut rest = text;
    while let Some(i) = rest.find('"') {
        let after = &rest[i + 1..];
        let Some(j) = after.find('"') else { break };
        out.push(&after[..j]);
        rest = &after[j + 1..];
    }
    out
}

/// A token's characteristic-defining ability compiled from a quote of
/// [`TokenQuote::Creator`] form: "~" in its values is the creating object (bound as the
/// token's characteristics are built, see `tokens::token_characteristics_in`).
pub fn refer_cda_to_creator(a: &Ability) -> Option<Ability> {
    let AbilityKind::Static(st) = &a.kind else {
        return None;
    };
    if !st.is_cda {
        return None;
    }
    let StaticEffect::Continuous { affected, mods } = &st.effect else {
        return None;
    };
    if !matches!(affected, Filter::Source) {
        return None;
    }
    let this = json(&Sel::This)?;
    let to = json(&Sel::All(Filter::Custom(GRANTER.into())))?;
    let v = replace(serde_json::to_value(mods).ok()?, &this, &to);
    let mut st = st.clone();
    st.effect = StaticEffect::Continuous {
        affected: Filter::Source,
        mods: serde_json::from_value(v).ok()?,
    };
    Some(AbilityDef::with_link(
        AbilityKind::Static(st),
        a.text.clone(),
        a.link,
    ))
}
