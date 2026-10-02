//! The counter grammar (CR 122): putting and removing counters, read compositionally.
//!
//! ```text
//! remove  := "remove" quantity counters "from" holder
//! put     := "put" quantity counters "on" holder [("for each" | "equal to") value]
//! quantity:= "all" | "up to" number | "any number of" | number | "twice" value
//!          | "a number of" (with "equal to" after the holder) | "another" | "an additional"
//! counters:= ["kind"] "counter" | ["kind"] "counters"   (no kind: counters of any kind)
//! holder  := an object reference (`object_ref`: "~", "it", "target creature", "each
//!            creature you control", "creatures your opponents control", ...)
//!          | "target" objects "or opponent" | "target" objects "or player"
//!          | ("a" | "an") objects                (chosen as the instruction is performed)
//!          | "each of" ("any number of" | "up to" N | N) objects     (chosen likewise)
//!          | holder "and each" holder
//! ```
//!
//! Removing N counters with no kind named removes N counters in all, of the kinds the
//! player removing them chooses ([`crate::counter_rules::remove_chosen_counters`]); "up to
//! N" and "any number of" let that player choose how many (`Effect::RemoveCountersUpTo`).

use super::{EffectPattern, FollowupPattern, StaticPattern, TriggerPattern};
use crate::ability::*;
use crate::oracle::costs::counter_kind;
use crate::oracle::effects::{object_ref, Builder};
use crate::oracle::phrases::*;
use crate::types::CounterKind;

/// How many counters.
#[derive(Clone, Debug)]
pub enum Qty {
    Exact(Value),
    UpTo(Value),
    AnyNumber,
    All,
}

/// A quantity at the start of `s`.
pub fn quantity<'a>(s: &'a str, b: &mut Builder) -> Option<(Qty, String)> {
    let s = s.trim_start();
    if let Some(r) = s.strip_prefix("all ") {
        return Some((Qty::All, r.to_string()));
    }
    if let Some(r) = s.strip_prefix("any number of ") {
        return Some((Qty::AnyNumber, r.to_string()));
    }
    if let Some(r) = s.strip_prefix("up to ") {
        let (n, r) = number(r, b)?;
        return Some((Qty::UpTo(n), r));
    }
    let (n, r) = number(s, b)?;
    Some((Qty::Exact(n), r))
}

/// "a", "two", "x", "twice x", "that many": a number of counters.
fn number(s: &str, b: &mut Builder) -> Option<(Value, String)> {
    if s.starts_with("twice ") {
        let (v, r) = super::value_grammar::parse_value(s, b)?;
        return Some((v, r));
    }
    let (n, r) = parse_number(s)?;
    if matches!(n, Value::X) && !super::value_grammar::x_defined() {
        // X defined later in the sentence ("..., where X is ...") or by the cost.
        return Some((n, r.to_string()));
    }
    Some((n, r.to_string()))
}

/// "counters", "counter", "+1/+1 counters", "a time counter": the kind (None: any).
pub fn counter_noun(s: &str) -> Option<(Option<CounterKind>, &str)> {
    if let Some(r) = strip(s, "counters").or_else(|| strip(s, "counter")) {
        return Some((None, r));
    }
    let (k, r) = counter_kind(s)?;
    let r = strip(r, "counters").or_else(|| strip(r, "counter"))?;
    Some((Some(k), r))
}

/// "target permanent or opponent", "target artifact, creature, planeswalker, or
/// opponent", "target permanent or player": an object or a player.
fn object_or_player_target(s: &str, b: &mut Builder) -> Option<Sel> {
    let r = s.trim().strip_prefix("up to one ").unwrap_or(s.trim());
    let up_to = r.len() != s.trim().len();
    let r = r.strip_prefix("target ")?;
    let (objs, pf) = if let Some(o) = r.strip_suffix(", or opponent") {
        (o, PlayerFilter::Opponent)
    } else if let Some(o) = r.strip_suffix(" or opponent") {
        (o, PlayerFilter::Opponent)
    } else if let Some(o) = r.strip_suffix(", or player") {
        (o, PlayerFilter::Any)
    } else if let Some(o) = r.strip_suffix(" or player") {
        (o, PlayerFilter::Any)
    } else {
        return None;
    };
    // "artifact, creature, planeswalker": the list without its last item's "or".
    let listed = match objs.rsplit_once(", ") {
        Some((a, z)) => format!("{a}, or {z}"),
        None => objs.to_string(),
    };
    let (f, _, tail) = parse_object_phrase(&listed)?;
    if !end(tail).is_empty() {
        return None;
    }
    let mut spec = TargetSpec::any_target();
    spec.what = TargetKind::ObjectOrPlayer(f, pf);
    if up_to {
        spec.min = 0;
    }
    let slot = b.add_target(spec, s.trim());
    Some(Sel::Target(slot))
}

/// The holder of the counters, with nothing after it.
pub fn holder(s: &str, b: &mut Builder) -> Option<Sel> {
    let s = end(s.trim());
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let restore = |b: &mut Builder| {
        b.targets.truncate(saved.0);
        b.it = saved.1.clone();
        b.it_player = saved.2.clone();
    };
    if let Some(sel) = object_or_player_target(s, b) {
        return Some(sel);
    }
    restore(b);
    // "target permanent or suspended card", "target permanent with a time counter on it
    // or suspended card": a permanent or a suspended card in exile (CR 702.62b).
    if let Some(perm) = s
        .strip_prefix("target ")
        .and_then(|r| r.strip_suffix(" or suspended card"))
    {
        let (f, plural, tail) = parse_object_phrase(perm)?;
        if plural || !end(tail).is_empty() {
            return None;
        }
        let suspended = Filter::and(vec![
            Filter::Card,
            Filter::InZone(ZoneKind::Exile),
            Filter::HasKeyword(crate::keywords::KeywordKind::Suspend),
            Filter::HasCounter(Some(crate::types::counters::TIME.into())),
        ]);
        let spec = TargetSpec::one(
            TargetKind::Object(Filter::Or(vec![
                Filter::and(vec![f, Filter::InZone(ZoneKind::Battlefield)]),
                suspended,
            ])),
            s,
        );
        let slot = b.add_target(spec, s);
        return Some(Sel::Target(slot));
    }
    // "each permanent and each suspended card".
    if let Some((x, y)) = s.split_once(" and each ") {
        if let (Some(a), Some(c)) = (holder(x, b), holder(&format!("each {y}"), b)) {
            // Objects in different zones: each group found in its own zone.
            if let (Sel::All(_), Sel::All(_)) = (&a, &c) {
                return Some(Sel::Union(vec![a, c]));
            }
        }
        restore(b);
    }
    // "one of them", "either of them": one of the objects the pronoun refers to, chosen
    // as the instruction is performed.
    if let Some(r) = s
        .strip_prefix("one of ")
        .or_else(|| s.strip_prefix("either of "))
    {
        if matches!(r, "them" | "those creatures" | "those permanents") {
            if let Some((sel, rest)) = object_ref(r, b) {
                if end(&rest).is_empty() && !matches!(sel, Sel::None) {
                    return Some(Sel::Choose {
                        chooser: PlayerRef::You,
                        filter: Filter::In(Box::new(sel)),
                        count: Value::c(1),
                        up_to: false,
                        store: None,
                    });
                }
            }
        }
        restore(b);
    }
    // "each of up to one target land and up to one target creature": both targets.
    if let Some(r) = s.strip_prefix("each of ") {
        if let Some((x, y)) = r.split_once(" and ") {
            if x.contains("target ") && y.contains("target ") {
                if let (Some((s1, r1)), Some((s2, r2))) = (object_ref(x, b), object_ref(y, b)) {
                    if end(&r1).is_empty() && end(&r2).is_empty() {
                        if let (Sel::Target(_), Sel::Target(_)) = (&s1, &s2) {
                            return Some(Sel::Union(vec![s1, s2]));
                        }
                    }
                }
                restore(b);
            }
        }
    }
    // "Put a loyalty counter on Ajani." (Ajani Resolute), "Put a +1/+1 counter on
    // Liliana." (Liliana the Repentant): a legendary card named by its first name, which
    // is also a planeswalker type (so the compiler doesn't read it as a name).
    if !s.contains(' ') && !s.is_empty() {
        let name = b.ctx.card_name.to_lowercase();
        let first = b.ctx.card_name.split(' ').next().unwrap_or_default();
        let pw_type = crate::types::subtype_kinds(first)
            .contains(&crate::types::SubtypeKind::Planeswalker);
        if b.ctx.type_line.supertypes.contains(crate::types::Supertype::Legendary)
            && pw_type
            && name.strip_prefix(s).is_some_and(|r| r.starts_with(' '))
        {
            return Some(Sel::This);
        }
    }
    // "create Zabu, a legendary 2/2 green Cat creature token with "... put a +1/+1 counter
    // on Zabu."": in the token's own ability, the token's name is the token.
    if !s.contains(' ')
        && !s.is_empty()
        && crate::oracle::raw_text()
            .to_lowercase()
            .contains(&format!("create {s}, a "))
    {
        return Some(Sel::This);
    }
    // "that Hero", "that Saga": the object the text is about, named by its type.
    if let Some(noun) = s.strip_prefix("that ") {
        if !noun.contains(' ')
            && matches!(b.it, Sel::TriggerObject | Sel::Target(_))
            && parse_object_phrase(noun).is_some_and(|(_, plural, tail)| !plural && tail.is_empty())
        {
            return Some(b.it.clone());
        }
    }
    // "a creature they control" in a trigger about a player: that player.
    let they;
    let s = match s.strip_suffix(" they control") {
        Some(head) if b.in_trigger && matches!(b.it_player, PlayerRef::TriggerPlayer) => {
            they = format!("{head} the triggering player controls");
            &they[..]
        }
        _ => s,
    };
    // "each creature each opponent controls": the creatures opponents control.
    let each_opp;
    let s = if s.starts_with("each ") && s.ends_with(" each opponent controls") {
        each_opp = s.replace(" each opponent controls", " your opponents control");
        &each_opp[..]
    } else {
        s
    };
    if let Some((sel, rest)) = object_ref(s, b) {
        // A spell that was cast isn't what counters are put on or removed from ("Whenever
        // an opponent casts a spell, ... remove a time counter from it": not understood).
        if end(&rest).is_empty() && !matches!(sel, Sel::TriggerSpell) {
            return Some(sel);
        }
    }
    restore(b);
    // "each suspended card", "permanents that player controls".
    if let Some(r) = s.strip_prefix("each ").or_else(|| s.strip_prefix("all ")) {
        if let Some((f, rest)) = super::value_grammar::objects(r, b) {
            if end(&rest).is_empty() {
                return Some(Sel::All(on_battlefield_unless_zoned(f)));
            }
        }
        restore(b);
    }
    // "a creature you control", "a creature or planeswalker you control": chosen as the
    // instruction is performed (CR 608.2c), not targeted.
    if let Some(r) = s.strip_prefix("a ").or_else(|| s.strip_prefix("an ")) {
        if let Some(sel) = chosen(r, Value::c(1), false, b) {
            return Some(sel);
        }
        restore(b);
    }
    // "up to one creature": chosen likewise.
    if let Some(r) = s.strip_prefix("up to ") {
        if let Some((n, r)) = parse_number(r) {
            if !r.starts_with("target") && !r.contains(" target ") {
                if let Some(sel) = chosen(r, n, true, b) {
                    return Some(sel);
                }
            }
        }
        restore(b);
    }
    // "each of any number of Sagas you control", "each of up to two Soldiers you
    // control", "each of two creatures you control".
    if let Some(r) = s.strip_prefix("each of ") {
        let parsed = if let Some(r) = r.strip_prefix("any number of ") {
            Some((Value::c(1000), true, r.to_string()))
        } else if let Some(r) = r.strip_prefix("up to ") {
            parse_number(r).map(|(n, r)| (n, true, r.to_string()))
        } else {
            parse_number(r).map(|(n, r)| (n, false, r.to_string()))
        };
        if let Some((n, up_to, r)) = parsed {
            if !r.starts_with("target") {
                if let Some(sel) = chosen(&r, n, up_to, b) {
                    return Some(sel);
                }
            }
        }
        restore(b);
    }
    None
}

/// Objects on the battlefield unless the phrase names another zone.
fn on_battlefield_unless_zoned(f: Filter) -> Filter {
    if mentions_zone(&f) {
        f
    } else {
        Filter::and(vec![f, Filter::InZone(ZoneKind::Battlefield)])
    }
}

fn mentions_zone(f: &Filter) -> bool {
    match f {
        Filter::InZone(_) => true,
        Filter::And(v) | Filter::Or(v) => v.iter().any(mentions_zone),
        _ => false,
    }
}

/// `n` objects the controller chooses as the instruction is performed.
fn chosen(r: &str, n: Value, up_to: bool, b: &mut Builder) -> Option<Sel> {
    let (f, rest) = super::value_grammar::objects(r, b)?;
    if !end(&rest).is_empty() {
        return None;
    }
    Some(Sel::Choose {
        chooser: PlayerRef::You,
        filter: on_battlefield_unless_zoned(f),
        count: n,
        up_to,
        store: None,
    })
}

/// "remove [quantity] [kind] counter(s) from [holder]".
fn remove_counters(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("remove ")?;
    let (q, r) = quantity(r, b)?;
    let (kind, r) = counter_noun(&r)?;
    let r = r.strip_prefix("from ")?;
    // "Remove a +1/+1 counter from each of two creatures you control. If you do, ...":
    // done only if there are two such creatures (each then has one removed).
    let mut needs: Option<(Filter, i32)> = None;
    let what = match holder(r, b)? {
        // "Remove a counter from a creature you control": one with such a counter.
        Sel::Choose {
            chooser,
            filter,
            count,
            up_to,
            store,
        } => {
            let filter = Filter::and(vec![filter, Filter::HasCounter(kind.clone())]);
            if !up_to && !matches!(count, Value::Const(1)) {
                let Value::Const(n) = count else {
                    return None;
                };
                if !matches!(q, Qty::Exact(_)) {
                    return None;
                }
                needs = Some((filter.clone(), n));
            }
            Sel::Choose {
                chooser,
                filter,
                count,
                up_to,
                store,
            }
        }
        other => other,
    };
    let e = remove_effect(q, what, kind);
    Some(match needs {
        Some((f, n)) => Effect::If {
            cond: Condition::Compare(Value::Count(f), Cmp::Ge, Value::c(n)),
            then: Box::new(e),
            otherwise: Box::new(Effect::Noop),
        },
        None => e,
    })
}

/// The removal of `q` counters of `kind` (any kind: None) from `what`.
fn remove_effect(q: Qty, what: Sel, kind: Option<CounterKind>) -> Effect {
    match q {
        Qty::Exact(n) => Effect::RemoveCounters { what, kind, n },
        Qty::All => Effect::RemoveCounters {
            what,
            kind,
            n: Value::Const(i32::MAX),
        },
        Qty::UpTo(n) => Effect::RemoveCountersUpTo {
            what,
            kind,
            max: Some(n),
        },
        Qty::AnyNumber => Effect::RemoveCountersUpTo {
            what,
            kind,
            max: None,
        },
    }
}

inventory::submit! { EffectPattern { name: "counter grammar: remove [quantity] counters from [holder]", priority: 400, parse: remove_counters } }


/// The variable a chosen holder is stored in ("Put a +1/+1 counter on a creature you
/// control. It gains double strike until end of turn.").
pub const CHOSEN: Var = vars::USER + 4123;

/// One counter item of a put: (quantity, kind).
fn put_item(s: &str, b: &mut Builder) -> Option<(Qty, CounterKind)> {
    let s = s.trim();
    // "another +1/+1 counter", "an additional +1/+1 counter": one more.
    let s2;
    let s = match s
        .strip_prefix("another ")
        .or_else(|| s.strip_prefix("an additional "))
    {
        Some(r) => {
            s2 = format!("a {r}");
            s2.as_str()
        }
        None => s,
    };
    let (q, r) = quantity(s, b)?;
    if matches!(q, Qty::All | Qty::AnyNumber) {
        return None;
    }
    let (kind, r) = counter_noun(&r)?;
    if !r.trim().is_empty() {
        return None;
    }
    Some((q, kind?))
}

/// "a +1/+1 counter or a loyalty counter", "a flying, lifelink, or +1/+1 counter", "two
/// +1/+1 counters or two charge counters": the options of a choice among counters
/// (CR 122.1b keyword counters among them), at least two.
fn put_options(s: &str, b: &mut Builder) -> Option<Vec<(Qty, CounterKind)>> {
    let s = s.trim();
    let parts: Vec<&str> = s
        .split(", or ")
        .flat_map(|p| p.split(" or "))
        .flat_map(|p| p.split(", "))
        .map(str::trim)
        .collect();
    if parts.len() < 2 {
        return None;
    }
    // Each option in full ("a +1/+1 counter or a loyalty counter") ...
    let full: Option<Vec<_>> = parts.iter().map(|p| put_item(p, b)).collect();
    if let Some(v) = full {
        return Some(v);
    }
    // ... or the quantity on the first and the noun on the last ("a flying, lifelink,
    // or +1/+1 counter", "that many +1/+1 counters or charge counters").
    let (q, first) = quantity(parts[0], b)?;
    let last = parts[parts.len() - 1];
    let noun = if last.ends_with(" counters") {
        " counters"
    } else if last.ends_with(" counter") {
        " counter"
    } else {
        return None;
    };
    let mut out = Vec::new();
    for (i, p) in parts.iter().enumerate() {
        let name = if i == 0 { first.as_str() } else { p };
        let name = name.strip_suffix(noun).unwrap_or(name).trim();
        if name.is_empty() || name.contains(' ') && crate::layers::keyword_counter(name).is_none() {
            return None;
        }
        let named = format!("{name}{noun}");
        let (k, rest) = counter_noun(&named)?;
        if !rest.trim().is_empty() {
            return None;
        }
        out.push((q.clone(), k?));
    }
    Some(out)
}

/// The effect putting `q` counters of `kind` on `what`.
fn add(what: &Sel, q: &Qty, kind: CounterKind) -> Option<Effect> {
    Some(match q {
        Qty::Exact(n) => Effect::AddCounters {
            what: what.clone(),
            kind,
            n: n.clone(),
        },
        // "put up to three lore counters on it": how many is chosen as it's put.
        Qty::UpTo(Value::Const(n)) => Effect::seq(vec![
            Effect::Choose {
                who: PlayerRef::You,
                kind: ChoiceKind::Number { min: 0, max: *n },
            },
            Effect::AddCounters {
                what: what.clone(),
                kind,
                n: Value::Chosen,
            },
        ]),
        _ => return None,
    })
}

/// A value phrase with nothing after it.
fn amount_value(a: &str, b: &mut Builder) -> Option<Value> {
    // "the amount of life you gained this turn or the amount of life you lost this turn,
    // whichever is greater".
    if let Some(r) = end(a).strip_suffix(", whichever is greater") {
        let (x, y) = r.split_once(" or ")?;
        return Some(Value::Max(
            Box::new(amount_value(x, b)?),
            Box::new(amount_value(y, b)?),
        ));
    }
    if end(a) == "the amount of life you lost this turn" {
        return Some(Value::LifeLostThisTurn(PlayerRef::You));
    }
    let (v, rest) = super::value_grammar::parse_value(a, b)?;
    end(&rest).is_empty().then_some(v)
}

/// "put that many [counters] on ..." / "put that number of [counters] on ..." after "if
/// it had counters on it" / "if that artifact had counters on it": how many counters the
/// object had as it last existed (CR 603.10, 608.2h).
fn counters_it_had(b: &Builder) -> Option<Value> {
    let raw = crate::oracle::raw_text().to_lowercase();
    let lki = if b.in_trigger && (raw.contains(", if it had one or more counters on it,") || raw.contains(", if it had counters on it,")) {
        Sel::TriggerLki
    } else if raw.contains("if that artifact had counters on it")
        || raw.contains("if that creature had counters on it")
        || raw.contains("if that permanent had counters on it")
    {
        match &b.it {
            Sel::Target(_) => b.it.clone(),
            _ => return None,
        }
    } else {
        return None;
    };
    Some(Value::CountersOn(Box::new(lki), None))
}

/// "put [items] on [holder] [equal to V]": one put instruction.
fn put_one(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("put ")?;
    // "put your choice of a +1/+1 counter or two charge counters on ..."
    let (choice, r) = match r.strip_prefix("your choice of ") {
        Some(r) => (true, r),
        None => (false, r),
    };
    // "put that many +1/+1 counters or charge counters on ..." after "if that artifact had
    // counters on it": as many as it had.
    let had = match r
        .strip_prefix("that many ")
        .or_else(|| r.strip_prefix("that number of "))
    {
        Some(rest) => Some((counters_it_had(b)?, rest)),
        None => None,
    };
    let x_items;
    let r = match &had {
        Some((_, rest)) => {
            x_items = format!("x {rest}");
            x_items.as_str()
        }
        None => r,
    };
    // "put a number of +1/+1 counters equal to ~'s power on ...", "put a number of
    // +1/+1 counters on ~ equal to ...".
    let (equal, r) = match r.strip_prefix("a number of ") {
        Some(r) => (true, r),
        None => (false, r),
    };
    let (items, holder_s) = r.split_once(" on ")?;
    let (items, amount_before) = match items.split_once(" equal to ") {
        Some((i, a)) if equal => (format!("x {i}"), Some(a.to_string())),
        _ if equal => (format!("x {items}"), None),
        _ => (items.to_string(), None),
    };
    let (holder_s, amount_after) = match holder_s.rsplit_once(" equal to ") {
        Some((h, a)) if equal && amount_before.is_none() => (h, Some(a.to_string())),
        _ => (holder_s, None),
    };
    let before = amount_before.is_some();
    let amount = amount_before.or(amount_after);
    if equal != amount.is_some() {
        return None;
    }
    let options = if choice {
        put_options(&items, b)?
    } else {
        match put_item(&items, b) {
            Some(i) => vec![i],
            None => put_options(&items, b)?,
        }
    };
    // The amount and the holder are read in text order: "equal to its power on up to one
    // target creature" ("its" is what the text was about before the target).
    let mut value = None;
    if before {
        value = Some(amount_value(amount.as_deref()?, b)?);
    }
    let mut what = holder(holder_s, b)?;
    if let Sel::Choose {
        count: Value::Const(1),
        store,
        filter,
        ..
    } = &mut what
    {
        // Not "one of them": "them" are still the objects they were.
        if !matches!(filter, Filter::In(_)) {
            *store = Some(CHOSEN);
            b.it = Sel::Var(CHOSEN);
        }
    }
    if !before {
        if let Some(a) = &amount {
            value = Some(amount_value(a, b)?);
        }
    }
    let amount = match had {
        Some((v, _)) => Some(v),
        None => value,
    };
    let mut effects = Vec::new();
    for (q, kind) in options {
        let mut e = add(&what, &q, kind.clone())?;
        if let Some(v) = &amount {
            e = super::r107_numbers::substitute_x(&e, v)?;
        }
        let label = match &q {
            Qty::Exact(Value::Const(1)) => format!("{kind} counter"),
            Qty::Exact(Value::Const(n)) => format!("{n} {kind} counters"),
            _ => format!("{kind} counters"),
        };
        effects.push((label, e));
    }
    if effects.len() == 1 {
        return effects.pop().map(|(_, e)| e);
    }
    // A choice among counters: the choice is made as the instruction is carried out; a
    // chosen holder is chosen first.
    Some(Effect::ChooseOne {
        who: PlayerRef::You,
        options: effects,
    })
}

/// "put [counters] on [holder], [counters] on [holder], and [counters] on [holder]":
/// several puts in one instruction ("Put a +1/+1 counter on target creature, two +1/+1
/// counters on another target creature, and three +1/+1 counters on a third target
/// creature.", "Put a +1/+1 counter on that Hero and a +1/+1 counter on ~.").
fn put_counters(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    // "they lose 1 life and you put a -1/-1 counter on ...".
    let l = l.strip_prefix("you ").unwrap_or(l);
    // "put a +1/+1 counter on each of them for every three cards in your graveyard": that
    // many for each full group of three (rounded down).
    if let Some((head, every)) = l.rsplit_once(" for every ") {
        let (k, r) = parse_number(every)?;
        let Value::Const(k) = k else {
            return None;
        };
        if k < 2 {
            return None;
        }
        let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
        let count = super::value_grammar::parse_value(&format!("the number of {r}"), b)
            .filter(|(_, rest)| end(rest).is_empty())
            .map(|(v, _)| v);
        let e = count.and_then(|count| {
            let e = put_counters(head, b)?;
            super::damage_removal_foreach::multiply(e, Value::Div(Box::new(count), k, false))
        });
        if e.is_none() {
            b.targets.truncate(saved.0);
            b.it = saved.1;
            b.it_player = saved.2;
        }
        return e;
    }
    let r = l.strip_prefix("put ")?;
    let r = r.replace(" a second target ", " another target ");
    let r = r.replace(" a third target ", " another target ");
    if let Some(e) = put_one(&format!("put {r}"), b) {
        return Some(e);
    }
    // Split before a later "[counters] on": each part is a put of its own.
    for sep in [", and ", " and ", ", "] {
        let mut from = 0;
        while let Some(i) = r[from..].find(sep).map(|i| i + from) {
            from = i + sep.len();
            let (a, c) = (&r[..i], &r[i + sep.len()..]);
            if !a.contains(" on ") || !c.contains(" on ") {
                continue;
            }
            let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
            if let Some(ea) = crate::oracle::effects::parse_clause(&format!("put {a}"), b) {
                if let Some(ec) = put_counters(&format!("put {c}"), b)
                    .or_else(|| crate::oracle::effects::parse_clause(&format!("put {c}"), b))
                {
                    return Some(Effect::seq(vec![ea, ec]));
                }
            }
            b.targets.truncate(saved.0);
            b.it = saved.1;
            b.it_player = saved.2;
        }
    }
    None
}

inventory::submit! { EffectPattern { name: "counter grammar: put [counters] on [holder]", priority: 400, parse: put_counters } }

/// How many counters the latest removal removed ("for each counter removed this way").
pub const REMOVED: Var = vars::USER + 4122;

/// The last instruction of `e` if it removes counters.
fn last_removal(e: &mut Effect) -> Option<&mut Effect> {
    if matches!(
        e,
        Effect::RemoveCounters { .. }
            | Effect::RemoveCountersUpTo { .. }
            | Effect::MoveCounters { .. }
    ) {
        return Some(e);
    }
    match e {
        Effect::Seq(v) => v.last_mut().and_then(last_removal),
        Effect::May { effect, .. } => last_removal(effect),
        Effect::If {
            then, otherwise, ..
        } if matches!(**otherwise, Effect::Noop) => last_removal(then),
        _ => None,
    }
}

/// "[kind] counter(s) removed this way", "counters were removed this way".
fn removed_this_way(thing: &str) -> bool {
    let Some(t) = [
        " were removed this way",
        " was removed this way",
        " removed this way",
        " are moved this way",
        " were moved this way",
        " moved this way",
    ]
    .iter()
    .find_map(|p| thing.strip_suffix(p)) else {
        return false;
    };
    counter_noun(t).is_some_and(|(_, r)| r.trim().is_empty())
}

/// After an instruction that removed counters: "[instruction] for each counter removed
/// this way", "For each [kind] counter removed this way, [instruction]", "[instruction]
/// that many [...]", "If no counters were removed this way, [instruction]": the number
/// of counters it removed (CR 608.2c).
fn after_removal(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    if last_removal(prev).is_none() {
        return false;
    }
    let l = end(l);
    // "that much"/"that many" become X below: not where the text has an X of its own.
    if l.split(|c: char| !c.is_alphanumeric()).any(|w| w == "x") {
        return false;
    }
    let count = Value::Var(REMOVED);
    let cond_form = [("if no ", Cmp::Eq, 0), ("if one or more ", Cmp::Ge, 1)]
        .into_iter()
        .find_map(|(p, cmp, n)| l.strip_prefix(p).map(|r| (r, cmp, n)));
    let e = if let Some((r, cmp, n)) = cond_form {
        let Some((thing, clause)) = r.split_once(", ") else {
            return false;
        };
        if !removed_this_way(thing) {
            return false;
        }
        // "you gain that much life": the number of counters.
        let Some(e) = crate::oracle::effects::parse_clause(&clause.replace("that much", "x"), b)
            .and_then(|e| super::r107_numbers::substitute_x(&e, &count))
        else {
            return false;
        };
        Effect::If {
            cond: Condition::Compare(count, cmp, Value::c(n)),
            then: Box::new(e),
            otherwise: Box::new(Effect::Noop),
        }
    } else if let Some(r) = l.strip_prefix("for each ") {
        let Some((thing, clause)) = r.split_once(", ") else {
            return false;
        };
        if !removed_this_way(thing) {
            return false;
        }
        let Some(e) = crate::oracle::effects::parse_clause(clause, b)
            .and_then(|e| super::damage_removal_foreach::multiply(e, count))
        else {
            return false;
        };
        e
    } else if let Some((clause, thing)) = l.rsplit_once(" for each ") {
        if !removed_this_way(thing) {
            return false;
        }
        let Some(e) = crate::oracle::effects::parse_clause(clause, b).and_then(|e| match e {
            // "Add one mana of any color for each charge counter removed this way": each
            // of any color.
            Effect::AddMana {
                who,
                mana: ManaProduction::AnyOneColor(Value::Const(1)),
                restriction,
            } => Some(Effect::AddMana {
                who,
                mana: ManaProduction::AnyCombination(count),
                restriction,
            }),
            e => super::damage_removal_foreach::multiply(e, count),
        }) else {
            return false;
        };
        e
    } else if l.contains(" that many ") {
        // "Draw that many cards."
        let Some(e) = crate::oracle::effects::parse_clause(&l.replace(" that many ", " x "), b)
            .and_then(|e| super::r107_numbers::substitute_x(&e, &count))
        else {
            return false;
        };
        e
    } else {
        return false;
    };
    let Some(removal) = last_removal(prev) else {
        return false;
    };
    let r = std::mem::take(removal);
    *removal = Effect::Seq(vec![
        r,
        Effect::StoreValue {
            var: REMOVED,
            value: Value::Prev,
        },
    ]);
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![
        Effect::StoreValue {
            var: REMOVED,
            value: Value::c(0),
        },
        old,
        e,
    ]);
    true
}

inventory::submit! { FollowupPattern { name: "counter grammar: ... counters removed this way", priority: 50, apply: after_removal } }

/// "remove all +1/+1 counters from ~, and it deals that much damage to each creature",
/// "remove [counters] from [holder] and [instruction with "that many"]": the number of
/// counters removed.
fn remove_and_that_much(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    if !l.starts_with("remove ") {
        return None;
    }
    for sep in [", and ", " and ", ", then "] {
        let Some((a, c)) = l.split_once(sep) else {
            continue;
        };
        if !(c.contains("that much") || c.contains("that many"))
            || c.split(|ch: char| !ch.is_alphanumeric()).any(|w| w == "x")
        {
            continue;
        }
        let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
        let parsed = (|| {
            let removal = remove_counters(a, b)?;
            let c = c.replace("that much", "x").replace("that many", "x");
            let e = crate::oracle::effects::parse_clause(&c, b)?;
            let e = super::r107_numbers::substitute_x(&e, &Value::Var(REMOVED))?;
            Some(Effect::Seq(vec![
                removal,
                Effect::StoreValue {
                    var: REMOVED,
                    value: Value::Prev,
                },
                e,
            ]))
        })();
        if parsed.is_some() {
            return parsed;
        }
        b.targets.truncate(saved.0);
        b.it = saved.1;
        b.it_player = saved.2;
    }
    None
}

inventory::submit! { EffectPattern { name: "counter grammar: remove counters and [that much]", priority: 90, parse: remove_and_that_much } }

/// "three or more", "two or fewer", "exactly one", "one or more", "a", "an", "no": how
/// many counters an object has, as a comparison.
fn counter_amount(s: &str) -> Option<(Cmp, Value, &str)> {
    if let Some(r) = s.strip_prefix("no ") {
        return Some((Cmp::Eq, Value::c(0), r));
    }
    if let Some(r) = s.strip_prefix("exactly ") {
        let (n, r) = parse_number(r)?;
        return Some((Cmp::Eq, n, r));
    }
    let (n, r) = parse_number(s)?;
    if let Some(r) = r.strip_prefix("or more ") {
        return Some((Cmp::Ge, n, r));
    }
    if let Some(r) = r
        .strip_prefix("or fewer ")
        .or_else(|| r.strip_prefix("or less "))
    {
        return Some((Cmp::Le, n, r));
    }
    // "a +1/+1 counter on it": one or more.
    if matches!(s.split(' ').next(), Some("a" | "an")) {
        return Some((Cmp::Ge, Value::c(1), r));
    }
    None
}

/// After "with" / "without": "[amount] [kind] counter(s) on it/them" (CR 122). Returns
/// the filter and the rest.
pub fn counters_on(rest: &str, negate: bool) -> Option<(Filter, &str)> {
    let (cmp, n, r) = match counter_amount(rest) {
        Some(x) => x,
        // "with counters on them", "with +1/+1 counters on them": one or more.
        None => (Cmp::Ge, Value::c(1), rest),
    };
    let (kind, r) = counter_noun(r)?;
    let tail = ["on it", "on them", "on him", "on her"]
        .iter()
        .find_map(|p| r.strip_prefix(p))?;
    if !(tail.is_empty() || tail.starts_with(' ') || tail.starts_with(',')) {
        return None;
    }
    let f = match (cmp, &n) {
        (Cmp::Ge, Value::Const(1)) => Filter::HasCounter(kind),
        (Cmp::Eq, Value::Const(0)) => Filter::not(Filter::HasCounter(kind)),
        _ => Filter::CounterCount(kind, cmp, Box::new(n)),
    };
    Some((if negate { Filter::not(f) } else { f }, tail))
}

/// "... for as long as it has a [kind] counter on it": the duration of an effect on an
/// object (CR 611.2b). Returns the duration and the text before it.
pub fn counter_duration(t: &str) -> Option<(Duration, &str)> {
    let (head, tail) = t.rsplit_once(" for as long as ")?;
    let r = tail
        .strip_prefix("it has ")
        .or_else(|| tail.strip_prefix("they have "))?;
    let r = r
        .strip_prefix("a ")
        .or_else(|| r.strip_prefix("an "))
        .unwrap_or(r);
    let (kind, r) = counter_noun(r)?;
    if !matches!(end(r), "on it" | "on them") {
        return None;
    }
    Some((Duration::WhileAffectedHasCounter(kind?), head))
}

/// The counter kind of "[it/that creature/...] has a [kind] counter on it".
fn has_counter_on_it(c: &str) -> Option<CounterKind> {
    let (_, r) = c.split_once(" has ")?;
    let r = r.strip_prefix("a ").or_else(|| r.strip_prefix("an "))?;
    let (kind, r) = counter_noun(r)?;
    (end(r) == "on it").then_some(kind?)
}

/// Rewrites every "until end of turn" duration of continuous effects in `e` to `d`.
fn with_duration(e: &mut Effect, d: &Duration) -> usize {
    match e {
        Effect::Seq(v) => v.iter_mut().map(|x| with_duration(x, d)).sum(),
        Effect::Modify { duration, .. } if matches!(duration, Duration::EndOfTurn) => {
            *duration = d.clone();
            1
        }
        _ => 0,
    }
}

/// "That land is an Island in addition to its other types for as long as it has a flood
/// counter on it.", "For as long as that creature has a shadow counter on it, it's a
/// Wraith in addition to its other types.", "that creature has base power and toughness
/// 3/1 and has flying for as long as it has a feather counter on it": a continuous
/// effect on an object for as long as it has a counter of that kind (CR 611.2b).
fn while_it_has_counter(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (clause, kind) = if let Some(r) = l.strip_prefix("for as long as ") {
        let (c, clause) = r.split_once(", ")?;
        (clause.to_string(), has_counter_on_it(c)?)
    } else {
        let (clause, c) = l.rsplit_once(" for as long as ")?;
        if !c.starts_with("it has ") {
            return None;
        }
        (clause.to_string(), has_counter_on_it(c)?)
    };
    // The state the object is in ("is", "'s", "has") as an effect that begins now.
    let clause = if let Some(r) = clause.strip_prefix("it's ") {
        format!("it becomes {r}")
    } else {
        let (subject, rest) = clause
            .split_once(" is ")
            .map(|(s, r)| (s.to_string(), format!("becomes {r}")))
            .unwrap_or_else(|| (String::new(), clause.clone()));
        if subject.is_empty() {
            clause.clone()
        } else {
            format!("{subject} {rest}")
        }
    };
    let clause = clause.replace(" and has ", " and gains ");
    let mut e = crate::oracle::effects::parse_clause(&format!("{clause} until end of turn"), b)?;
    if with_duration(&mut e, &Duration::WhileAffectedHasCounter(kind)) == 0 {
        return None;
    }
    Some(e)
}

inventory::submit! { EffectPattern { name: "counter grammar: [effect] for as long as it has a [kind] counter on it", priority: 50, parse: while_it_has_counter } }

/// "As long as there is exactly one tide counter on ~, it gets -1/-1." (Homarid), "As
/// long as ~ has seven or more loyalty counters on him, he's ...": after a condition on
/// the counters on ~, the pronoun is ~.
fn as_long_as_counters_it(l: &str, text: &str, ctx: &crate::oracle::CompileContext) -> Option<Vec<Ability>> {
    let r = end(l).strip_prefix("as long as ")?;
    let (c, rest) = r.split_once(", ")?;
    if !c.contains(" counter") || !(c.ends_with(" on ~") || c.starts_with("~ has ")) {
        return None;
    }
    let rest = ["it ", "he ", "she "]
        .iter()
        .find_map(|p| rest.strip_prefix(p))?;
    crate::oracle::statics::parse_static(&format!("as long as {c}, ~ {rest}."), ctx).map(|v| {
        v.into_iter()
            .map(|a| AbilityDef::new(a.kind.clone(), text))
            .collect()
    })
}

inventory::submit! { StaticPattern { name: "counter grammar: as long as [counters on ~], it ...", priority: 50, parse: as_long_as_counters_it } }

/// "When there are four or more depletion counters on ~", "Whenever there are four or
/// more tide counters on ~", "When ~ has three or more plague counters on it": a state
/// trigger (CR 603.8) on the counters on the source.
fn counters_state_trigger(r: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    let r = end(r);
    let (c, on) = if let Some(c) = r
        .strip_prefix("there are ")
        .or_else(|| r.strip_prefix("there is "))
    {
        (c, "~")
    } else {
        (r.strip_prefix("~ has ")?, "it")
    };
    let (cmp, n, c) = super::counters_resources_counters::amount_cmp(c)?;
    let (kind, c) = super::counters_resources_counters::kind_then_on(c)?;
    if end(c) != on {
        return None;
    }
    Some((
        TriggerCond::State(Condition::Compare(
            Value::CountersOn(Box::new(Sel::This), kind),
            cmp,
            n,
        )),
        Sel::This,
        PlayerRef::You,
    ))
}

inventory::submit! { TriggerPattern { name: "counter grammar: when there are N counters on ~ (state trigger)", priority: 50, parse: counters_state_trigger } }

/// "Put a charge counter on ~ or remove one from it.", "Put a plague counter on ~ or
/// remove a plague counter from it.", "Put a lore counter on target Saga you control or
/// remove one from it.": the controller chooses which as the ability resolves (Jinxed
/// Choker's ruling), and can choose to remove one only if there's one there (Plague
/// Boiler's ruling).
fn put_or_remove(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let r = l.strip_prefix("put ")?;
    let (put_s, remove_s) = r.split_once(" or remove ")?;
    let put = put_one(&format!("put {put_s}"), b)?;
    let Effect::AddCounters { what, kind, n } = &put else {
        return None;
    };
    let removed = remove_s
        .strip_suffix(" from it")
        .or_else(|| remove_s.strip_suffix(" from them"))?;
    let (rn, rkind) = if removed == "one" {
        (n.clone(), kind.clone())
    } else {
        let (q, rest) = quantity(removed, b)?;
        let (k, rest) = counter_noun(&rest)?;
        let Qty::Exact(rn) = q else {
            return None;
        };
        if !rest.trim().is_empty() || k.as_ref() != Some(kind) {
            return None;
        }
        (rn, kind.clone())
    };
    let remove = Effect::RemoveCounters {
        what: what.clone(),
        kind: Some(rkind.clone()),
        n: rn,
    };
    // Removing a counter can be chosen only if there's one to remove (Plague Boiler's
    // ruling).
    Some(Effect::If {
        cond: Condition::Compare(
            Value::CountersOn(Box::new(what.clone()), Some(rkind.clone())),
            Cmp::Ge,
            Value::c(1),
        ),
        then: Box::new(Effect::ChooseOne {
            who: PlayerRef::You,
            options: vec![
                (format!("Put a {kind} counter"), put.clone()),
                (format!("Remove a {rkind} counter"), remove),
            ],
        }),
        otherwise: Box::new(put.clone()),
    })
}

inventory::submit! { EffectPattern { name: "counter grammar: put a counter on [holder] or remove one from it", priority: 50, parse: put_or_remove } }

/// "sacrifice it, draw a card, and put a +1/+1 counter on each creature you control": a
/// list of three or more instructions, one of them about counters, each understood on
/// its own and performed in order.
fn instruction_list(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (head, last) = l.rsplit_once(", and ")?;
    let mut parts: Vec<&str> = head.split(", ").collect();
    parts.push(last);
    if parts.len() < 3
        || !parts
            .iter()
            .any(|p| (p.starts_with("put ") || p.starts_with("remove ")) && p.contains(" counter"))
    {
        return None;
    }
    let mut out = Vec::new();
    for p in parts {
        out.push(crate::oracle::effects::parse_clause(p, b)?);
    }
    Some(Effect::Seq(out))
}

inventory::submit! { EffectPattern { name: "counter grammar: A, B, and C with a counter instruction", priority: 450, parse: instruction_list } }

/// "If a creature would deal combat damage to ~, prevent that damage and put a +1/+1
/// counter on ~." (Ironscale Hydra), "If a source would deal damage to you, prevent that
/// damage and put an incarnation counter on ~." (Nine Lives): a prevention effect whose
/// instruction is performed for each damage event it prevents (CR 615.5).
fn prevent_and_put_counter(l: &str, text: &str, _ctx: &crate::oracle::CompileContext) -> Option<Vec<Ability>> {
    let r = end(l).strip_prefix("if ")?;
    let (source, r) = if let Some(r) = r.strip_prefix("a creature would deal ") {
        (Filter::creature(), r)
    } else {
        (Filter::Any, r.strip_prefix("a source would deal ")?)
    };
    let (combat_only, r) = match r.strip_prefix("combat damage to ") {
        Some(r) => (true, r),
        None => (false, r.strip_prefix("damage to ")?),
    };
    let (to_players, to_objects, r) = if let Some(r) = r.strip_prefix("~, ") {
        (None, Some(Filter::Source), r)
    } else {
        (Some(PlayerFilter::You), None, r.strip_prefix("you, ")?)
    };
    let r = r.strip_prefix("prevent that damage and put ")?;
    let (n, r) = parse_number(r)?;
    let (kind, r) = counter_noun(r)?;
    if end(r) != "on ~" {
        return None;
    }
    let def = ReplacementDef {
        event: ReplacementEvent::Damage {
            source,
            to_players,
            to_objects,
            combat_only,
        },
        action: ReplacementAction::PreventAndThen(
            None,
            Box::new(Effect::AddCounters {
                what: Sel::This,
                kind: kind?,
                n,
            }),
        ),
        self_replacement: false,
        optional: false,
    };
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Replacement(def))),
        text,
    )])
}

inventory::submit! { StaticPattern { name: "counter grammar: prevent that damage and put a counter on ~", priority: 50, parse: prevent_and_put_counter } }

/// "Exile it with a hit counter on it.", "exile target nonland card from your graveyard
/// with two time counters on it", "exile a card from your hand with a number of time
/// counters on it equal to its mana value": the exiled card gets the counters as it's
/// exiled (CR 122.1, 406.3).
fn exile_with_counters(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    if !l.starts_with("exile ") {
        return None;
    }
    // "exile a creature you control and put a takeover counter on it": the same as
    // exiling it with the counter on it.
    let rewritten;
    let l = match l.rsplit_once(" and put ") {
        Some((head, put)) if put.ends_with(" on it") && !head.contains(" with ") => {
            rewritten = format!("{head} with {put}");
            rewritten.as_str()
        }
        _ => l,
    };
    // "... and it gains suspend" is another instruction.
    let (l, and_then) = match l.split_once(" on it and ") {
        Some((a, c)) => (format!("{a} on it"), Some(c.to_string())),
        None => (l.to_string(), None),
    };
    let (head, tail) = l.rsplit_once(" with ")?;
    let (counters, amount) = match tail.split_once(" on it equal to ") {
        Some((c, a)) => (c.to_string(), Some(a.to_string())),
        None => (tail.strip_suffix(" on it")?.to_string(), None),
    };
    let (n, r) = if let Some(r) = counters.strip_prefix("a number of ") {
        (Value::X, r.to_string())
    } else {
        let (n, r) = parse_number(&counters)?;
        (n, r.to_string())
    };
    if amount.is_some() != counters.starts_with("a number of ") {
        return None;
    }
    let (kind, r) = counter_noun(&r)?;
    if !r.trim().is_empty() {
        return None;
    }
    let exile = crate::oracle::effects::parse_clause(head, b)?;
    if !matches!(exile, Effect::Exile { face_down: false, .. }) {
        return None;
    }
    let exiled = Sel::Var(vars::IT);
    b.it = exiled.clone();
    let n = match amount {
        // "equal to its mana value": the exiled card's.
        Some(a) => amount_value(&a, b)?,
        None => n,
    };
    let mut out = vec![
        exile,
        Effect::AddCounters {
            what: exiled,
            kind: kind?,
            n,
        },
    ];
    if let Some(c) = and_then {
        out.push(crate::oracle::effects::parse_clause(&c, b)?);
    }
    Some(Effect::Seq(out))
}

inventory::submit! { EffectPattern { name: "counter grammar: exile [object] with N counters on it", priority: 50, parse: exile_with_counters } }

/// "Choose a counter on target permanent.", "Choose a kind of counter on a creature you
/// control.", "Choose a counter on target permanent or player.": the kind is chosen as
/// the ability resolves; the next sentence says what's done with it (see
/// [`with_that_kind`]).
fn choose_a_counter(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l)
        .strip_prefix("choose a counter on ")
        .or_else(|| end(l).strip_prefix("choose a kind of counter on "))?;
    let from = match holder(r, b)? {
        Sel::Choose {
            chooser,
            filter,
            count,
            up_to,
            ..
        } => {
            let s = Sel::Choose {
                chooser,
                filter: Filter::and(vec![filter, Filter::HasCounter(None)]),
                count,
                up_to,
                store: Some(CHOSEN),
            };
            b.it = Sel::Var(CHOSEN);
            s
        }
        other => other,
    };
    Some(Effect::ChooseCounterKind {
        from,
        then: Box::new(Effect::Noop),
    })
}

inventory::submit! { EffectPattern { name: "counter grammar: choose a counter on [holder]", priority: 50, parse: choose_a_counter } }

/// After "Choose a counter on [holder].": "Put an additional counter of that kind on that
/// permanent.", "Give that permanent or player another counter of that kind.", "Put a
/// counter of that kind on each other creature you control.", "Put a counter of that kind
/// on target permanent you control if it doesn't have a counter of that kind on it.",
/// "Remove that counter from that permanent or card or put another of those counters on
/// it."
fn with_that_kind(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let slot = match prev {
        Effect::ChooseCounterKind { then, .. } if matches!(**then, Effect::Noop) => then,
        _ => return false,
    };
    let k = CHOSEN_COUNTER_KIND;
    let mut t = end(l).to_string();
    for (from, to) in [
        ("an additional counter of that kind", format!("an additional {k} counter")),
        ("another counter of that kind", format!("another {k} counter")),
        ("a counter of that kind", format!("a {k} counter")),
        ("another of those counters", format!("another {k} counter")),
        ("that counter", format!("a {k} counter")),
    ] {
        t = t.replace(from, &to);
    }
    if let Some(r) = t.strip_prefix("give that permanent or player ") {
        t = format!("put {r} on it");
    }
    // "... if it doesn't have a counter of that kind on it": only then.
    let (t, unless_has) = match t.strip_suffix(&format!(" if it doesn't have a {k} counter on it")) {
        Some(head) => (head.to_string(), true),
        None => (t, false),
    };
    // "remove [a counter] from that permanent or card or put another [one] on it".
    let e = if let Some(r) = t.strip_prefix(&format!("remove a {k} counter from ")) {
        let Some((_, put)) = r.split_once(" or put ") else {
            return false;
        };
        if put != format!("another {k} counter on it") {
            return false;
        }
        let it = b.it.clone();
        Effect::ChooseOne {
            who: PlayerRef::You,
            options: vec![
                (
                    "remove that counter".into(),
                    Effect::RemoveCounters {
                        what: it.clone(),
                        kind: Some(k.into()),
                        n: Value::c(1),
                    },
                ),
                (
                    "put another of those counters".into(),
                    Effect::AddCounters {
                        what: it,
                        kind: k.into(),
                        n: Value::c(1),
                    },
                ),
            ],
        }
    } else {
        let Some(mut e) = crate::oracle::effects::parse_clause(&t, b) else {
            return false;
        };
        // "each other creature you control": other than the one with the chosen counter.
        if t.contains(" each other ") {
            match &mut e {
                Effect::AddCounters {
                    what: Sel::All(f), ..
                } => {
                    *f = Filter::and(vec![
                        f.clone(),
                        Filter::not(Filter::In(Box::new(Sel::Var(CHOSEN)))),
                    ]);
                }
                _ => return false,
            }
        }
        if unless_has {
            let Effect::AddCounters { what, .. } = &e else {
                return false;
            };
            e = Effect::If {
                cond: Condition::Not(Box::new(Condition::SelMatches(
                    what.clone(),
                    Filter::HasCounter(Some(k.into())),
                ))),
                then: Box::new(e),
                otherwise: Box::new(Effect::Noop),
            };
        }
        e
    };
    // Only instructions about the chosen kind.
    if !serde_json::to_string(&e).is_ok_and(|s| s.contains(k)) {
        return false;
    }
    **slot = e;
    true
}

inventory::submit! { FollowupPattern { name: "counter grammar: [instruction] with a counter of that kind", priority: 50, apply: with_that_kind } }

/// "they lose 1 life and you put a -1/-1 counter on up to one target creature they
/// control": a put with "you" as its explicit subject, tried before patterns that read
/// "you [verb]" generally.
fn you_put(l: &str, b: &mut Builder) -> Option<Effect> {
    if !end(l).starts_with("you put ") {
        return None;
    }
    put_counters(l, b)
}

inventory::submit! { EffectPattern { name: "counter grammar: you put [counters] on [holder]", priority: 5, parse: you_put } }

/// "put the same number of each kind of counter on that creature" (as ~ has, Denry
/// Klin): for each kind of counter on ~, that many of that kind (CR 122.1).
fn same_counters_as_this(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("put the same number of each kind of counter on ")?;
    // "Whenever a nontoken creature you control enters, if ~ has counters on it, put ...
    // on that creature": the creature that entered (the condition made "it" ~).
    let to = if r == "that creature" && b.in_trigger {
        Sel::TriggerObject
    } else {
        holder(r, b)?
    };
    if matches!(to, Sel::This) {
        return None;
    }
    // Only after a condition that names ~'s counters ("if ~ has counters on it").
    let raw = crate::oracle::raw_text().to_lowercase();
    if !raw.contains(" has counters on it") {
        return None;
    }
    Some(Effect::PutCountersOf {
        from: Sel::This,
        to,
        kind: None,
    })
}

inventory::submit! { EffectPattern { name: "counter grammar: put the same number of each kind of counter on [holder]", priority: 5, parse: same_counters_as_this } }

/// "Then sacrifice it if it has five or more bloodstain counters on it." after an
/// instruction that changed the counters on ~: the condition first (see
/// `counters_then_if_on_it`).
fn then_x_if_it_has(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    let Some(r) = l.strip_prefix("then ") else {
        return false;
    };
    let Some((clause, cond)) = r.rsplit_once(" if it has ") else {
        return false;
    };
    if !cond.contains(" counter") {
        return false;
    }
    super::counters_then_if_on_it::then_if_counters_on_it(
        &format!("then if it has {cond}, {clause}"),
        prev,
        b,
    )
}

inventory::submit! { FollowupPattern { name: "counter grammar: then [instruction] if it has N counters on it", priority: 55, apply: then_x_if_it_has } }

/// "~ has trample as long as it has two or fewer oil counters on it. Otherwise, it has
/// hexproof." (Evolved Spinoderm): one static ability while the counters say so, another
/// otherwise.
fn has_x_otherwise_y(l: &str, text: &str, ctx: &crate::oracle::CompileContext) -> Option<Vec<Ability>> {
    let l = end(l);
    let (first, second) = l.split_once(". otherwise, it has ")?;
    let (grant, cond) = first.split_once(" as long as it has ")?;
    let grant = grant.strip_prefix("~ has ")?;
    if !cond.contains(" counter") {
        return None;
    }
    let cond = crate::oracle::statics::parse_condition(&format!("~ has {cond}"), ctx)?;
    let a = crate::oracle::statics::parse_static(&format!("~ has {grant}."), ctx)?;
    let b2 = crate::oracle::statics::parse_static(&format!("~ has {second}."), ctx)?;
    let with = |v: Vec<Ability>, c: Condition| -> Option<Vec<Ability>> {
        v.into_iter()
            .map(|ab| match &ab.kind {
                AbilityKind::Static(st) if st.condition.is_none() => {
                    let mut st = st.clone();
                    st.condition = Some(c.clone());
                    Some(AbilityDef::new(AbilityKind::Static(st), text))
                }
                _ => None,
            })
            .collect()
    };
    let mut out = with(a, cond.clone())?;
    out.extend(with(b2, Condition::Not(Box::new(cond)))?);
    Some(out)
}

inventory::submit! { StaticPattern { name: "counter grammar: ~ has X as long as it has N counters on it. Otherwise, it has Y.", priority: 50, parse: has_x_otherwise_y } }
