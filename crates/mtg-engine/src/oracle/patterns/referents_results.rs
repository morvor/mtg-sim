//! Referent and value grammar III: amounts and objects that point back to something an
//! earlier part of the ability named (CR 107, 603.4, 608.2c, 608.2h).
//!
//! - "that many" / "that much" bound to the count an intervening "if" clause names ("At
//!   the beginning of combat on your turn, if you control five or more Deserts, create
//!   that many ... tokens", CR 603.4: the count as the ability resolves), to the counters
//!   or mana value it names about the object the trigger is about ("When ~ dies, if it had
//!   one or more -1/-1 counters on it, its owner draws that many cards"), or to the
//!   quantity a trigger's "one or more" qualifier names ("Whenever a creature you control
//!   with one or more counters on it dies, exile that many cards", "Whenever you cast a
//!   noncreature spell with one or more blue mana symbols in its mana cost, create that
//!   many ... tokens");
//! - "remove all of them" / "remove them" after a condition about counters ("Then if it
//!   has eight or more night counters on it, remove all of them"): those counters.

use super::AbilityPattern;
use crate::ability::*;
use crate::oracle::CompileContext;

thread_local! {
    static IN_STANDARD: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Whether the standard classifier (and the other patterns) already read the block, so
/// that the patterns here leave it alone. Patterns here call this before doing anything
/// else; while it runs, they decline.
pub(crate) fn standard_ok(block: &str, ctx: &CompileContext) -> bool {
    if IN_STANDARD.with(|c| c.get()) {
        return true;
    }
    IN_STANDARD.with(|c| c.set(true));
    let parsed = crate::oracle::parse_ability(block, ctx);
    IN_STANDARD.with(|c| c.set(false));
    parsed.is_some_and(|v| {
        !v.iter().any(|a| {
            super::oracle_hardening_referents::has_no_referent(a)
                || crate::repeat_process::has_stray_repeat(a)
                || super::filters_relational::unresolved(a)
        })
    })
}

/// Whether the patterns here are being asked from [`standard_ok`] (so they decline).
pub(crate) fn in_standard() -> bool {
    IN_STANDARD.with(|c| c.get())
}

/// Whether `s` has the word `w`.
pub(crate) fn has_word(s: &str, w: &str) -> bool {
    s.split(|c: char| !c.is_alphanumeric() && c != '\'' && c != '/' && c != '+' && c != '-')
        .any(|x| x == w)
}

/// "N or more [kind] counters on it": the counter kind a condition names (`None` for
/// "counters" of any kind).
fn condition_counter_kind(cond: &str) -> Option<Option<String>> {
    let i = cond.find(" counters on ")?;
    let head = &cond[..i];
    let kind = head.rsplit(' ').next()?;
    if kind == "more" {
        return Some(None);
    }
    Some(Some(kind.to_string()))
}

/// "remove all of them from it", "remove all of them", "remove them" after a condition
/// about counters on `holder`: those counters ("remove all night counters from it").
fn rewrite_remove_them(cond: &str, eff: &str) -> Option<String> {
    let kind = condition_counter_kind(cond)?;
    let holder = {
        let i = cond.find(" counters on ")?;
        let h = cond[i + " counters on ".len()..].trim();
        // "if ~ has seven or more ember counters on it": "it" is the condition's subject.
        let subject = cond.split_once(" has ").map(|(s, _)| s);
        match (h, subject) {
            ("it", Some("~")) => "~".to_string(),
            ("it", Some("it")) | ("it", None) => "it".to_string(),
            ("~", _) => "~".to_string(),
            _ => return None,
        }
    };
    let what = match &kind {
        Some(k) => format!("remove all {k} counters from {holder}"),
        None => format!("remove all counters from {holder}"),
    };
    let lower = eff.to_lowercase();
    for p in [
        "remove all of them from it",
        "remove all of them from ~",
        "remove all of them",
        "remove them",
    ] {
        if let Some(i) = lower.find(p) {
            let after = &eff[i + p.len()..];
            if !(after.is_empty() || after.starts_with([' ', ',', '.'])) {
                continue;
            }
            return Some(format!("{}{what}{after}", &eff[..i]));
        }
    }
    None
}

/// The selection `Var(TESTED)` stands for in a filter's values replaced by `sel`.
fn tested_as(v: &Value, sel: &Sel) -> Option<Value> {
    use serde_json::Value as J;
    fn walk(v: J, tested: &J, sel: &J) -> J {
        match v {
            J::Object(m) => {
                if J::Object(m.clone()) == *tested {
                    return sel.clone();
                }
                J::Object(m.into_iter().map(|(k, v)| (k, walk(v, tested, sel))).collect())
            }
            J::Array(a) => J::Array(a.into_iter().map(|x| walk(x, tested, sel)).collect()),
            other => other,
        }
    }
    let tested = serde_json::to_value(Sel::Var(vars::TESTED)).ok()?;
    let sel = serde_json::to_value(sel).ok()?;
    let j = walk(serde_json::to_value(v).ok()?, &tested, &sel);
    serde_json::from_value(j).ok()
}

/// The quantity a filter's "one or more" / "N or more" qualifier counts for the object
/// `sel` ("with one or more counters on it", "with one or more blue mana symbols in its
/// mana cost", "with mana value 1 or greater"): exactly one such qualifier.
fn amount_in_filter(f: &Filter, sel: &Sel) -> Option<Value> {
    let mut found: Vec<Value> = Vec::new();
    collect_filter_amounts(f, sel, &mut found);
    (found.len() == 1).then(|| found.remove(0))
}

fn collect_filter_amounts(f: &Filter, sel: &Sel, out: &mut Vec<Value>) {
    let at_least = |c: &Cmp| matches!(c, Cmp::Ge | Cmp::Gt);
    match f {
        Filter::And(v) => v.iter().for_each(|x| collect_filter_amounts(x, sel, out)),
        Filter::CounterCount(k, c, _) if at_least(c) => {
            out.push(Value::CountersOn(Box::new(sel.clone()), k.clone()))
        }
        // "with one or more counters on it".
        Filter::HasCounter(k) => out.push(Value::CountersOn(Box::new(sel.clone()), k.clone())),
        Filter::ManaValue(c, _) if at_least(c) => {
            out.push(Value::ManaValueOf(Box::new(sel.clone())))
        }
        Filter::ValueCmp(v, c, _) if at_least(c) => {
            if let Some(v) = tested_as(v, sel) {
                out.push(v);
            }
        }
        // "a spell that targets one or more creatures or Vehicles you control": how many of
        // its targets are such objects.
        Filter::Targets(inner) if matches!(sel, Sel::TriggerSpell) => out.push(Value::CountSel(
            Box::new(Sel::All(Filter::and(vec![
                (**inner).clone(),
                Filter::TargetOf(Box::new(Sel::TriggerSpell)),
            ]))),
        )),
        _ => {}
    }
}

/// The count an intervening "if" condition names ("if you control five or more Deserts":
/// the number of Deserts you control; "if it had one or more +1/+1 counters on it": the
/// number of +1/+1 counters it had; "if its mana value was 1 or greater": its mana value).
pub(crate) fn amount_in_condition(c: &Condition) -> Option<Value> {
    match c {
        Condition::Compare(v, Cmp::Ge | Cmp::Gt, k) if k.as_const().is_some() => Some(v.clone()),
        Condition::SelMatches(sel, f) => amount_in_filter(f, sel),
        Condition::And(v) => {
            let found: Vec<Value> = v.iter().filter_map(amount_in_condition).collect();
            (found.len() == 1).then(|| found.into_iter().next())?
        }
        _ => None,
    }
}

/// The object a trigger event is about, as the ability sees it: the spell cast, or the
/// object as it last existed (for a permanent that left the battlefield, CR 603.10a).
fn trigger_subject(t: &TriggerCond) -> Option<(Sel, &Filter)> {
    match t {
        TriggerCond::CastSpell { filter, .. } => Some((Sel::TriggerSpell, filter)),
        TriggerCond::Dies(f) | TriggerCond::LeavesBattlefield(f) => Some((Sel::TriggerLki, f)),
        TriggerCond::ZoneChange { filter, .. } => Some((Sel::TriggerLki, filter)),
        TriggerCond::EntersBattlefield(f)
        | TriggerCond::Attacks(f)
        | TriggerCond::Blocks(f)
        | TriggerCond::BecomesBlocked(f) => Some((Sel::TriggerObject, f)),
        TriggerCond::Where { trigger, .. } => trigger_subject(trigger),
        _ => None,
    }
}

/// The quantity a trigger condition's qualifier names ("Whenever a creature you control
/// with one or more counters on it dies": the number of counters it had).
fn amount_in_trigger(t: &TriggerCond) -> Option<Value> {
    let (sel, f) = trigger_subject(t)?;
    amount_in_filter(f, &sel)
}

/// Whether "that many" can also mean the trigger event's own amount (damage dealt, life
/// gained, counters put, creatures attacking).
fn event_has_amount(t: &TriggerCond) -> bool {
    match t {
        TriggerCond::DealsDamage { .. }
        | TriggerCond::IsDealtDamage { .. }
        | TriggerCond::PlayerDealtDamage { .. }
        | TriggerCond::GainsLife { .. }
        | TriggerCond::LosesLife { .. }
        | TriggerCond::CountersPut { .. }
        | TriggerCond::CountersPutBy { .. }
        | TriggerCond::CountersRemoved { .. }
        | TriggerCond::Mills(_)
        | TriggerCond::PlayerAttacks(_)
        | TriggerCond::PlayerAttacksWith { .. }
        | TriggerCond::Batched { .. } => true,
        TriggerCond::Where { trigger, .. }
        | TriggerCond::FirstTimeEachTurn(trigger)
        | TriggerCond::ThisTurn(trigger)
        | TriggerCond::UntilYourNextTurn(trigger)
        | TriggerCond::Noncombat(trigger) => event_has_amount(trigger),
        _ => false,
    }
}

/// Replaces "that many" / "that much" by X (for [`crate::oracle::patterns::r107_numbers`]'s
/// substitution), unless the text has an X of its own.
fn that_many_as_x(eff: &str) -> Option<String> {
    let lower = eff.to_lowercase();
    if has_word(&lower, "x") {
        return None;
    }
    let mut s = eff.to_string();
    for (p, r) in [
        ("that amount of damage", "X damage"),
        ("that amount of life", "X life"),
        ("that much damage", "X damage"),
        ("that much life", "X life"),
        ("that many", "X"),
        ("that much", "X"),
    ] {
        s = s.replace(p, r);
    }
    Some(s)
}

/// "[trigger], if [condition], [instructions with "that many", "that much" or "remove all
/// of them"]" and "[trigger with a "one or more" qualifier], [instructions with "that
/// many"]".
fn trigger_condition_amount(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if in_standard() {
        return None;
    }
    let text = crate::oracle::strip_ability_word(block.trim());
    let lower = text.to_lowercase();
    if !(lower.starts_with("when ") || lower.starts_with("whenever ") || lower.starts_with("at "))
    {
        return None;
    }
    let mentions = lower.contains("that many")
        || lower.contains("that much")
        || lower.contains("that amount of ")
        || lower.contains("remove all of them")
        || lower.contains("remove them");
    if !mentions || standard_ok(block, ctx) {
        return None;
    }
    // The intervening condition, if any (its counters are what "them" refers to).
    let mut rewritten = text.to_string();
    if let Some(i) = lower.find(", if ") {
        let rest = &text[i + 5..];
        if let Some(j) = rest.find(", ") {
            let cond = rest[..j].to_lowercase();
            let eff = &rest[j + 2..];
            if let Some(e) = rewrite_remove_them(&cond, eff) {
                rewritten = format!("{}{}", &text[..i + 5 + j + 2], e);
            }
        }
    }
    // "remove all of them from it and create that many ...": the number removed (the
    // counter grammar's "remove ... and [that many]").
    let removed_first = {
        let r = rewritten.to_lowercase();
        match (r.find("remove all "), r.find("that many")) {
            (Some(i), Some(j)) => i < j,
            _ => false,
        }
    };
    let uses_amount = !removed_first
        && (rewritten.contains("that many")
            || rewritten.contains("that much")
            || rewritten.contains("that amount of "));
    if uses_amount {
        rewritten = that_many_as_x(&rewritten)?;
    }
    let a = super::value_grammar::with_x_defined(true, || {
        crate::oracle::triggers::parse_triggered(&rewritten, ctx)
    })?;
    let AbilityKind::Triggered(mut tr) = a.kind.clone() else {
        return None;
    };
    if uses_amount {
        let from_cond = tr.intervening_if.as_ref().and_then(amount_in_condition);
        let from_trigger = amount_in_trigger(&tr.trigger);
        let event = event_has_amount(&tr.trigger).then_some(Value::EventAmount);
        // One meaning only: a condition's count and the event's own amount would both be
        // "that many".
        // "that much damage" / "that amount of damage" is an amount, which only the event
        // has (a condition counts objects).
        let low = lower.as_str();
        let amount_only = !low.contains("that many")
            && (low.contains("that much") || low.contains("that amount of "));
        let amount = match (from_cond, from_trigger, event) {
            (_, _, Some(v)) if amount_only => v,
            (Some(v), None, None) | (None, Some(v), None) | (None, None, Some(v)) => v,
            _ => return None,
        };
        tr.body.effect = super::r107_numbers::substitute_x(&tr.body.effect, &amount)?;
        for t in tr.body.targets.iter_mut() {
            *t = super::r107_numbers::substitute_x_in_target(t, &amount)?;
        }
    }
    let def = AbilityDef::new(AbilityKind::Triggered(tr), text);
    if super::oracle_hardening_referents::has_no_referent(&def)
        || crate::repeat_process::has_stray_repeat(&def)
        || super::filters_relational::unresolved(&def)
    {
        return None;
    }
    Some(vec![def])
}

inventory::submit! { AbilityPattern { name: "referents: that many (the condition's or qualifier's count)", priority: 50, parse: trigger_condition_amount } }

/// "Then if it has eight or more night counters on it, remove all of them and create ...",
/// "if ~ has seven or more ember counters on it, remove them and transform ~": the
/// counters the condition names.
fn if_counters_remove_them(l: &str, b: &mut crate::oracle::effects::Builder) -> Option<Effect> {
    let l = crate::oracle::phrases::end(l);
    let l = l.strip_prefix("then ").unwrap_or(l);
    let r = l.strip_prefix("if ")?;
    let (cond, eff) = r.split_once(", ")?;
    let eff = rewrite_remove_them(cond, eff)?;
    crate::oracle::effects::parse_sentence(&format!("if {cond}, {eff}"), b)
}

inventory::submit! { super::EffectPattern { name: "referents: if [it] has N or more [kind] counters, remove them", priority: 1500, parse: if_counters_remove_them } }

/// The same sentence after an earlier one ("Put a night counter on ~. Then if it has eight
/// or more night counters on it, remove all of them and ..."): a leading "if" whose
/// instruction refers to the condition's counters isn't read by the sentence parser.
fn if_counters_remove_them_followup(
    l: &str,
    prev: &mut Effect,
    b: &mut crate::oracle::effects::Builder,
) -> bool {
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let Some(e) = if_counters_remove_them(l, b) else {
        b.targets.truncate(saved.0);
        (b.it, b.it_player) = (saved.1, saved.2);
        return false;
    };
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![old, e]);
    true
}

inventory::submit! { super::FollowupPattern { name: "referents: then if [it] has N or more [kind] counters, remove them", priority: 1500, apply: if_counters_remove_them_followup } }

/// "exile that many cards from the top of your library", "exile two cards from the top of
/// their library": the top cards of that library ("exile the top two cards of ...").
fn exile_cards_from_top(l: &str, b: &mut crate::oracle::effects::Builder) -> Option<Effect> {
    let l = crate::oracle::phrases::end(l);
    let (verb, i) = [" exiles ", "exile "]
        .iter()
        .find_map(|v| l.find(v).map(|i| (*v, i)))?;
    if verb == "exile " && i != 0 {
        return None;
    }
    let head = &l[..i + verb.len()];
    let r = &l[i + verb.len()..];
    let (n, rest) = r.split_once(" cards from the top of ")?;
    if n.contains(' ') && !n.starts_with("that many") || n.contains(',') {
        return None;
    }
    if !(rest.ends_with(" library") || rest.contains(" library ")) {
        return None;
    }
    crate::oracle::effects::parse_clause(&format!("{head}the top {n} cards of {rest}"), b)
}

inventory::submit! { super::EffectPattern { name: "referents: exile N cards from the top of a library", priority: 1500, parse: exile_cards_from_top } }

/// "the number of counters removed this way" after an instruction (not a cost) removed
/// them ("you may remove all of them. If you do, exile each other nonland permanent with
/// mana value less than or equal to the number of counters removed this way"): the number
/// that instruction removed (its result, CR 608.2c). Called from the value grammar's
/// counts (`value_results::count_ext`).
pub(crate) fn count_ext(
    r: &str,
    b: &mut crate::oracle::effects::Builder,
) -> Option<(Value, String)> {
    // "Auras you controlled that were attached to it" after it left the battlefield: the
    // permanents attached to it as it last existed there, each as it last existed (CR
    // 608.2h; Hateful Eidolon rulings).
    if let Some(v) = attached_as_it_left(r, b) {
        return Some(v);
    }
    // "nontoken creatures you controlled that died this turn": those that died under your
    // control this turn (CR 700.4).
    for (p, q) in [(
        " you controlled that died this turn",
        " that died under your control this turn",
    )] {
        if let Some(i) = r.find(p) {
            if !r[..i].contains(',') {
                let rewritten = format!("the number of {}{q}{}", &r[..i], &r[i + p.len()..]);
                return super::value_grammar::parse_value(&rewritten, b);
            }
        }
    }
    let i = r.find(" removed this way")?;
    let head = &r[..i];
    let rest = &r[i + " removed this way".len()..];
    let counters = head == "counters"
        || head == "counter"
        || head.ends_with(" counters") && !head.contains(" of ");
    if !counters {
        return None;
    }
    let raw = crate::oracle::raw_text().to_lowercase();
    let j = raw.find("removed this way")?;
    let before = &raw[..j];
    let k = before.rfind("remove ")?;
    let between = &before[k..];
    // The same ability, a separate instruction, not a cost ("Remove ...: ...").
    if between.contains('\n') || between.contains(':') || !(between.contains(". ") || between.contains(", ")) {
        return None;
    }
    Some((Value::Prev, rest.to_string()))
}

/// "Whenever one or more nontoken creatures attack this turn, create that many 1/1 white
/// Soldier creature tokens that are tapped and attacking." (Basri Ket): a delayed
/// triggered ability (CR 603.7b) whose "that many" is its own trigger event's amount.
fn delayed_that_many(l: &str, b: &mut crate::oracle::effects::Builder) -> Option<Effect> {
    let l = crate::oracle::phrases::end(l);
    if !(l.starts_with("whenever ")
        || l.starts_with("until end of turn, whenever ")
        || l.starts_with("until your next turn, whenever "))
    {
        return None;
    }
    if !(l.contains("that many") || l.contains("that much")) {
        return None;
    }
    let rewritten = that_many_as_x(l)?;
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let e = super::value_grammar::with_x_defined(true, || {
        crate::oracle::effects::parse_clause(&rewritten, b)
    });
    let restore = |b: &mut crate::oracle::effects::Builder| {
        b.targets.truncate(saved.0);
        (b.it, b.it_player) = (saved.1.clone(), saved.2.clone());
    };
    let Some(mut e) = e else {
        restore(b);
        return None;
    };
    fn last_delayed(e: &mut Effect) -> Option<(&mut TriggerCond, &mut Body)> {
        match e {
            Effect::Seq(v) => v.last_mut().and_then(last_delayed),
            Effect::DelayedTrigger { trigger, body, .. } => Some((trigger, &mut **body)),
            _ => None,
        }
    }
    let ok = (|| {
        let (trigger, body) = last_delayed(&mut e)?;
        // "Whenever one or more [creatures] attack this turn": one event per declaration of
        // attackers, its amount the number of such attackers (CR 508.3c), as for "whenever
        // a player attacks with one or more [creatures]".
        if let TriggerCond::ThisTurn(inner) | TriggerCond::UntilYourNextTurn(inner) = trigger {
            if let TriggerCond::Batched { trigger: t, .. } = &**inner {
                if let TriggerCond::Attacks(f) = &**t {
                    **inner = TriggerCond::PlayerAttacksWith {
                        who: PlayerRel::Any,
                        filter: f.clone(),
                        min: 1,
                    };
                }
            }
        }
        let trigger = &*trigger;
        if !event_has_amount(trigger) || !body.targets.is_empty() {
            return None;
        }
        body.effect = super::r107_numbers::substitute_x(&body.effect, &Value::EventAmount)?;
        Some(())
    })();
    if ok.is_none() || super::r107_numbers::uses_x(&e) {
        restore(b);
        return None;
    }
    Some(e)
}

inventory::submit! { super::EffectPattern { name: "referents: whenever ... this turn, ... that many (its event's amount)", priority: 1500, parse: delayed_that_many } }

/// "For each 1 life you lost, sacrifice a permanent other than ~ unless you discard a
/// card." (Oath of Lim-Dûl), "for each nontoken creature you controlled that died this
/// turn, create a 2/2 black Zombie creature token": an amount before the instruction,
/// which is performed that many times (or on that many, see
/// `iteration_grammar::times_over`). Only instructions that don't refer to what's
/// counted (those are `iteration_grammar`'s "for each [objects], [instruction about it]").
fn for_each_amount_leading(l: &str, b: &mut crate::oracle::effects::Builder) -> Option<Effect> {
    let l = crate::oracle::phrases::end(l);
    let r = l.strip_prefix("for each ")?;
    let (x, y) = r.split_once(", ")?;
    if x.contains('"') || y.contains(" for each ") {
        return None;
    }
    // "{1}, Remove X ki counters from ~: For each counter removed, ~ gets +2/+0 until end
    // of turn.": X, the number the cost removed.
    if x == "counter removed" {
        let raw = crate::oracle::raw_text().to_lowercase();
        let cost = raw.split(':').next().unwrap_or("");
        if !(cost.contains("remove x ") && super::value_grammar::x_defined()) {
            return None;
        }
        let e = crate::oracle::effects::parse_sentence(y, b)?;
        return Some(super::iteration_grammar::times_over(e, Value::X));
    }
    // "For each five counters removed this way": in groups of five, rounded down (an
    // amount of life or damage in groups is the value grammar's own).
    let (group, x) = match crate::oracle::phrases::parse_number(x) {
        Some((n, rest))
            if n.as_const().is_some_and(|k| k > 1)
                && rest.starts_with(' ')
                && !rest.contains(" life ")
                && !rest.contains("damage") =>
        {
            (n.as_const(), rest.trim_start())
        }
        _ => (None, x),
    };
    let refers_back = [
        "it", "its", "they", "them", "their", "that", "those", "he", "she", "his", "her",
    ]
    .iter()
    .any(|w| has_word(y, w));
    if refers_back {
        return None;
    }
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let restore = |b: &mut crate::oracle::effects::Builder| {
        b.targets.truncate(saved.0);
        (b.it, b.it_player) = (saved.1.clone(), saved.2.clone());
    };
    // "for each nontoken creature you controlled that died this turn": the number of
    // nontoken creatures ... (the counted noun in the plural).
    let plural = plural_head(x);
    let parsed = super::value_grammar::parse_value(&format!("the number of {x}"), b)
        .filter(|(_, rest)| rest.trim().is_empty())
        .or_else(|| {
            restore(b);
            let p = plural.as_ref()?;
            super::value_grammar::parse_value(&format!("the number of {p}"), b)
        });
    let Some((times, rest)) = parsed else {
        restore(b);
        return None;
    };
    if !rest.trim().is_empty() || b.targets.len() != saved.0 {
        restore(b);
        return None;
    }
    let Some(e) = crate::oracle::effects::parse_sentence(y, b) else {
        restore(b);
        return None;
    };
    let times = match group {
        Some(k) => Value::Div(Box::new(times), k, false),
        None => times,
    };
    Some(super::iteration_grammar::times_over(e, times))
}

inventory::submit! { super::EffectPattern { name: "referents: for each [amount], [instruction]", priority: 990, parse: for_each_amount_leading } }

/// The phrase with its first counted noun in the plural ("nontoken creature you
/// controlled" → "nontoken creatures you controlled").
fn plural_head(x: &str) -> Option<String> {
    const NOUNS: [&str; 10] = [
        "creature", "card", "spell", "permanent", "land", "artifact", "enchantment", "token",
        "opponent", "player",
    ];
    let mut words: Vec<String> = x.split(' ').map(str::to_string).collect();
    let i = words.iter().position(|w| NOUNS.contains(&w.as_str()))?;
    words[i].push('s');
    Some(words.join(" "))
}

/// The objects a mass instruction acted on, recorded as it begins ("their total power").
pub const ACTED_ON: Var = vars::USER + 12_002;

/// "Destroy all creatures target opponent controls. ~ deals damage to that player equal
/// to their total power.": the objects the instruction acted on, those destroyed as they
/// last existed on the battlefield and any that weren't with their current power (CR
/// 608.2h; Angrath, Minotaur Pirate ruling).
fn their_total_after_mass(s: &str, prev: &mut Effect, b: &mut crate::oracle::effects::Builder) -> bool {
    let lower = s.to_lowercase();
    let (phrase, stat) = match [
        ("their total power", Stat::Power),
        ("their total toughness", Stat::Toughness),
    ]
    .into_iter()
    .find(|(p, _)| lower.contains(p))
    {
        Some(x) => x,
        None => return false,
    };
    if has_word(&lower, "x") || lower.matches(phrase).count() != 1 {
        return false;
    }
    fn last_mass(e: &mut Effect) -> Option<&mut Effect> {
        match e {
            Effect::Seq(v) => v.last_mut().and_then(last_mass),
            Effect::Destroy {
                what: Sel::All(_), ..
            }
            | Effect::Exile {
                what: Sel::All(_), ..
            } => Some(e),
            _ => None,
        }
    }
    if last_mass(prev).is_none() {
        return false;
    }
    let rewritten = lower.replace(phrase, "x");
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let parsed = super::value_grammar::with_x_defined(true, || {
        crate::oracle::effects::parse_sentence(&rewritten, b)
    });
    let amount = Value::Aggregate(AggOp::Sum, stat, Box::new(Sel::Var(ACTED_ON)));
    let Some(e) = parsed.and_then(|e| super::r107_numbers::substitute_x(&e, &amount)) else {
        b.targets.truncate(saved.0);
        (b.it, b.it_player) = (saved.1, saved.2);
        return false;
    };
    if super::r107_numbers::uses_x(&e) {
        b.targets.truncate(saved.0);
        (b.it, b.it_player) = (saved.1, saved.2);
        return false;
    }
    let Some(mass) = last_mass(prev) else {
        return false;
    };
    let what = match mass {
        Effect::Destroy { what, .. } | Effect::Exile { what, .. } => what,
        _ => return false,
    };
    let group = std::mem::replace(what, Sel::Var(ACTED_ON));
    let acted = std::mem::take(mass);
    *mass = Effect::seq(vec![
        Effect::Store {
            var: ACTED_ON,
            sel: group,
        },
        acted,
    ]);
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![old, e]);
    true
}

inventory::submit! { super::FollowupPattern { name: "referents: their total power (of the objects a mass instruction acted on)", priority: 1500, apply: their_total_after_mass } }

/// "Aura you controlled that was attached to it", "Auras that were attached to it",
/// "Equipment that was attached to it": the permanents attached to "it" as it last
/// existed on the battlefield (for an object that has left it).
fn attached_as_it_left(
    r: &str,
    b: &mut crate::oracle::effects::Builder,
) -> Option<(Value, String)> {
    let (noun, x) = r.split_once(' ')?;
    let sub = match noun {
        "aura" | "auras" => "Aura",
        "equipment" => "Equipment",
        _ => return None,
    };
    let (yours, x) = match x.strip_prefix("you controlled ") {
        Some(x) => (true, x),
        None => (false, x),
    };
    let x = x
        .strip_prefix("that was attached to ")
        .or_else(|| x.strip_prefix("that were attached to "))?;
    let (who, rest) = crate::oracle::effects::object_ref(x, b)?;
    // Only an object that has left the battlefield (the trigger's object as it last
    // existed): what's attached to a permanent still there is "attached to it".
    if !matches!(who, Sel::TriggerLki) {
        return None;
    }
    let mut f = vec![Filter::Subtype(sub.into())];
    if yours {
        f.push(Filter::ControlledBy(PlayerRel::You));
    }
    Some((
        Value::CountSel(Box::new(Sel::Matching(
            Box::new(Sel::AttachedAsItLeft(Box::new(who))),
            Filter::and(f),
        ))),
        rest,
    ))
}

/// "Put a +1/+1 counter on the creature tapped to pay ~'s additional cost." (Swallow
/// Whole): the permanent the spell's "tap an untapped [permanent]" additional cost tapped
/// (recorded as the cost is paid, `zones::record_cost_moved`).
fn tapped_to_pay_additional_cost(
    l: &str,
    b: &mut crate::oracle::effects::Builder,
) -> Option<Effect> {
    const PHRASES: [&str; 2] = [
        "the creature tapped to pay ~'s additional cost",
        "the creature tapped to pay this spell's additional cost",
    ];
    let lower = l.to_lowercase();
    let p = PHRASES.iter().find(|p| lower.contains(*p))?;
    let raw = crate::oracle::raw_text().to_lowercase();
    if !raw.contains("as an additional cost to cast this spell, tap an untapped ") {
        return None;
    }
    let key = p.to_string();
    b.named.push((key.clone(), Sel::Var(vars::TAPPED)));
    let e = crate::oracle::effects::parse_clause(&lower, b);
    b.named.retain(|(n, _)| *n != key);
    e
}

inventory::submit! { super::EffectPattern { name: "referents: the creature tapped to pay ~'s additional cost", priority: 1500, parse: tapped_to_pay_additional_cost } }

/// "the life lost this way" / "the amount of life lost this way" after a life-loss
/// instruction ("Each opponent loses two times X life. You gain life equal to the life
/// lost this way."): the total that instruction made players lose (CR 119.3, 608.2c).
pub(crate) fn atom_ext(
    s: &str,
    _b: &mut crate::oracle::effects::Builder,
) -> Option<(Value, String)> {
    for p in ["the life lost this way", "the amount of life lost this way"] {
        if let Some(rest) = s.strip_prefix(p) {
            if rest.is_empty() || rest.starts_with([' ', ',', '.', ';', '"']) {
                // The instruction before is a life loss.
                let raw = crate::oracle::raw_text().to_lowercase();
                let i = raw.find("life lost this way")?;
                let before = &raw[..i];
                let last = before.rfind(". ").map_or(before, |j| &before[..j]);
                if !(last.contains(" lose ") || last.contains(" loses ")) {
                    return None;
                }
                return Some((Value::Prev, rest.to_string()));
            }
        }
    }
    None
}
