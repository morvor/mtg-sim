//! Numbers (CR 107.1–107.3): "half ..., rounded up/down" (CR 107.1a), "where X is ..."
//! (CR 107.3c) with negative results treated as 0 (CR 107.1b), and "+X/+Y" with two
//! defined variables (CR 107.3p).

use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::patterns::{AbilityPattern, EffectPattern};
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;

/// "half the number of forests you control, rounded down", "half their life, rounded up",
/// or a value phrase understood by the core compiler. Returns the value and the rest.
pub fn value_phrase(s: &str, b: &mut Builder) -> Option<(Value, String)> {
    let s = s.trim();
    // "the number of cards in their hand minus 4" (Black Vise).
    if let Some((a, r)) = s.rsplit_once(" minus ") {
        if let Some((n @ Value::Const(_), tail)) = parse_number(r) {
            if end(tail).is_empty() {
                if let Some((v, _)) = value_phrase(a, b).filter(|(_, r)| end(r).is_empty()) {
                    return Some((Value::Diff(Box::new(v), Box::new(n)), tail.to_string()));
                }
            }
        }
    }
    // "the number of times this ability has resolved this turn" (this resolution
    // included; Bronze Cudgels' ruling).
    if let Some(r) = s.strip_prefix("the number of times this ability has resolved this turn") {
        return Some((Value::TimesResolvedThisTurn, r.to_string()));
    }
    // "the number of cards in their hand": "their" is "that player" (Black Vise).
    for p in [
        "the number of cards in their hand",
        "the number of cards in that player's hand",
    ] {
        if let Some(r) = s.strip_prefix(p) {
            // With no player mentioned before, "their" has no antecedent here.
            if super::oracle_hardening_referents::is_no_player_referent(&b.it_player) {
                return None;
            }
            return Some((Value::HandSize(b.it_player.clone()), r.to_string()));
        }
    }
    // "3 minus the number of cards in their hand" (The Rack).
    if let Some((n, r)) = parse_number(s) {
        if let (Value::Const(_), Some(r)) = (&n, r.trim_start().strip_prefix("minus ")) {
            let (v, rest) = value_phrase(r, b)?;
            return Some((Value::Diff(Box::new(n), Box::new(v)), rest));
        }
    }
    // "the revealed card's mana value", after an instruction revealing a card (which "it"
    // then names, e.g. "Target opponent reveals a card at random from their hand.").
    if let Some(r) = s.strip_prefix("the revealed card's mana value") {
        if matches!(b.it, Sel::Var(vars::IT)) && (r.is_empty() || r.starts_with([' ', ','])) {
            return Some((Value::ManaValueOf(Box::new(b.it.clone())), r.to_string()));
        }
    }
    super::value_grammar::parse_value(s, b)
}

/// CR 107.1b: a calculation that determines the result of an effect uses 0 instead of a
/// negative number.
pub(crate) fn nonnegative(v: Value) -> Value {
    Value::Max(Box::new(v), Box::new(Value::c(0)))
}

/// Where an X a sentence defined ("..., where X is ...") is kept for later sentences of
/// the same ability that use it (CR 107.3c, 608.2h).
pub const DEFINED_X: Var = u16::MAX - 1073;

/// The value a sentence "..., where X is [value]." defines, as a calculation that uses 0
/// instead of a negative number (CR 107.1b). Reading it leaves `b` as it was.
pub(crate) fn defined_x(sentence: &str, b: &mut Builder) -> Option<Value> {
    let lower = sentence.to_lowercase();
    let (_, value_s) = end(&lower).rsplit_once(", where x is ")?;
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let read = value_phrase(value_s, b);
    b.targets.truncate(saved.0);
    b.it = saved.1;
    b.it_player = saved.2;
    let (v, tail) = read?;
    end(&tail).is_empty().then(|| nonnegative(v))
}

/// Whether `e` uses the X chosen for its spell or ability (`Value::X`, or `{X}` in a cost).
pub(crate) fn uses_x(e: &Effect) -> bool {
    fn walk(v: &serde_json::Value) -> bool {
        match v {
            serde_json::Value::String(s) => s == "X",
            serde_json::Value::Object(m) => m.values().any(walk),
            serde_json::Value::Array(a) => a.iter().any(walk),
            _ => false,
        }
    }
    serde_json::to_value(e).is_ok_and(|v| walk(&v))
}

/// Replaces `Value::X` (and optionally `Value::Y` written as the variable `y`) in an
/// effect.
pub(crate) fn substitute_x(e: &Effect, x: &Value) -> Option<Effect> {
    substitute_x_in(e, x)
}

/// The numeric variable holding a defined X used by several instructions of one sentence.
const SENTENCE_X: Var = u16::MAX - 1074;

/// Replaces X in an effect with its defined value. An X several instructions use ("you
/// gain X life and draw X cards") is determined once, before the first of them (CR
/// 608.2h), unless it depends on each player or object in turn.
fn bind_x(e: &Effect, x: &Value) -> Option<Effect> {
    let uses = serde_json::to_string(e)
        .map(|j| j.matches("\"X\"").count())
        .unwrap_or(0);
    let xj = serde_json::to_string(x).unwrap_or_default();
    let per_each = xj.contains("Iterated") || xj.contains(&format!("{{\"Var\":{}}}", vars::AFFECTED));
    if uses < 2 || x.as_const().is_some() || per_each {
        return substitute_x(e, x);
    }
    let store = Effect::StoreValue {
        var: SENTENCE_X,
        value: x.clone(),
    };
    let e = substitute_x(e, &Value::Var(SENTENCE_X))?;
    Some(store_before_first_use(e, store))
}

/// Puts `store` right before the first instruction of `e` that uses [`SENTENCE_X`] (the
/// value is determined as that instruction is performed: "mill two cards, then ~ gets
/// +X/+X ..., where X is the number of creature cards in your graveyard").
fn store_before_first_use(e: Effect, store: Effect) -> Effect {
    let uses = |e: &Effect| {
        serde_json::to_string(e).is_ok_and(|j| j.contains(&format!("{{\"Var\":{SENTENCE_X}}}")))
    };
    match e {
        Effect::Seq(mut v) => match v.iter().position(uses) {
            Some(i) => {
                let first = std::mem::take(&mut v[i]);
                v[i] = store_before_first_use(first, store);
                Effect::Seq(v)
            }
            None => Effect::Seq(v),
        },
        e => Effect::Seq(vec![store, e]),
    }
}

/// Replaces `Value::X` in any part of an ability (an effect, a target's number or
/// division).
fn substitute_x_in<T: serde::Serialize + serde::de::DeserializeOwned>(
    t: &T,
    x: &Value,
) -> Option<T> {
    let json = serde_json::to_value(t).ok()?;
    let xv = serde_json::to_value(x).ok()?;
    let out = subst(json, &serde_json::Value::String("X".into()), &xv);
    serde_json::from_value(out).ok()
}

fn subst(
    v: serde_json::Value,
    from: &serde_json::Value,
    to: &serde_json::Value,
) -> serde_json::Value {
    use serde_json::Value as J;
    if &v == from {
        return to.clone();
    }
    match v {
        J::Object(m) => J::Object(
            m.into_iter()
                .map(|(k, x)| (k, subst(x, from, to)))
                .collect(),
        ),
        J::Array(a) => J::Array(a.into_iter().map(|x| subst(x, from, to)).collect()),
        other => other,
    }
}

/// "[effect], where X is [value]" (CR 107.3c: the text defines X, so the controller
/// doesn't choose it; the value is determined as the effect is performed).
fn where_x_is(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    // The sentence's comma may sit inside a closing quote: "create X ... tokens with
    // "This token can't block," where X is ...".
    let (clause, value_s) = match l.rsplit_once(", where x is ") {
        Some((c, v)) => (c.to_string(), v),
        None => {
            let (c, v) = l.rsplit_once(",\" where x is ")?;
            (format!("{c}\""), v)
        }
    };
    let it = b.it.clone();
    // "..., where X is the number of creature cards in your graveyard as you cast this
    // spell" (Undercity Upheaval): X is read as the spell is cast. That's when X is used
    // if it's only the number of targets or the amount divided among them (CR 601.2c-d),
    // so nothing else may use it.
    if let Some(v) = value_s
        .strip_suffix(" as you cast ~")
        .or_else(|| value_s.strip_suffix(" as you cast this spell"))
    {
        let (x, tail) = value_phrase(v, b)?;
        if !end(&tail).is_empty() {
            return None;
        }
        let e = where_x_is_parts(&clause, v, b, it)?;
        return (!format!("{e:?}").contains(&format!("{:?}", nonnegative(x)))).then_some(e);
    }
    where_x_is_parts(&clause, value_s, b, it)
}

/// "[clause], where X is [value]": the value is read first, with pronouns as they are
/// ("that spell's mana value"); then "it" in the clause names `it` (e.g. a token the
/// previous instruction created: "Put X +1/+1 counters on it, where X is ...").
pub fn where_x_is_parts(clause: &str, value_s: &str, b: &mut Builder, it: Sel) -> Option<Effect> {
    // "where X is the number of creatures on the battlefield as you cast ~", "... you
    // controlled as you cast ~", "... as you activate this ability": the value as the
    // spell or ability is put on the stack. That's when the number of targets and the
    // division of damage or counters among them are chosen (CR 601.2c-d, 602.2b); the
    // value isn't remembered for its resolution, so the X may only be used for those.
    let value_s = end(value_s);
    let as_cast = [" as you cast ~", " as you cast this spell", " as you activate this ability"]
        .iter()
        .find_map(|p| value_s.strip_suffix(p));
    let owned;
    let value_s = match as_cast {
        Some(v) => {
            owned = v.replace(" you controlled", " you control");
            owned.as_str()
        }
        None => value_s,
    };
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    if let Some((v, tail)) = value_phrase(value_s, b) {
        if end(&tail).is_empty() {
            return where_x_is_value_inner(clause, v, b, it, as_cast.is_some());
        }
    }
    b.targets.truncate(saved.0);
    b.it = saved.1;
    b.it_player = saved.2;
    // The value may refer to what the instruction names ("Target player draws X cards,
    // where X is the number of cards in their graveyard"): read the instruction first.
    where_x_is_clause_first(clause, value_s, b, it, as_cast.is_some())
}

/// "[clause], where X is [value]" with the value read after the clause, so that its
/// pronouns can refer to the clause's targets.
fn where_x_is_clause_first(
    clause: &str,
    value_s: &str,
    b: &mut Builder,
    it: Sel,
    targets_only: bool,
) -> Option<Effect> {
    let first_target = b.targets.len();
    b.it = it;
    let e = super::value_grammar::with_x_defined(true, || {
        crate::oracle::effects::parse_clause(clause, b)
    })?;
    let (v, tail) = value_phrase(value_s, b)?;
    if !end(&tail).is_empty() {
        return None;
    }
    let x = nonnegative(v);
    let mentions_x =
        |j: Result<String, serde_json::Error>| j.is_ok_and(|j| j.contains("\"X\""));
    let clause_targets = first_target..b.targets.len();
    if targets_only
        && (mentions_x(serde_json::to_string(&e))
            || !b.targets[clause_targets.clone()]
                .iter()
                .any(|t| mentions_x(serde_json::to_string(t))))
    {
        return None;
    }
    for i in clause_targets {
        b.targets[i] = substitute_x_in(&b.targets[i], &x)?;
    }
    bind_x(&e, &x)
}

/// "[clause], where X is [value]" with the value already read.
pub fn where_x_is_value(clause: &str, v: Value, b: &mut Builder, it: Sel) -> Option<Effect> {
    where_x_is_value_inner(clause, v, b, it, false)
}

/// [`where_x_is_value`]; with `targets_only`, X may only be the number of targets or the
/// amount divided among them (a value determined as the spell or ability is put on the
/// stack).
fn where_x_is_value_inner(
    clause: &str,
    v: Value,
    b: &mut Builder,
    it: Sel,
    targets_only: bool,
) -> Option<Effect> {
    // An object the value named ("cards equal to the sacrificed creature's power") is
    // what a later "its" refers to, unless the clause names another.
    let value_it = std::mem::replace(&mut b.it, it.clone());
    let first_target = b.targets.len();
    // The clause's X has a value (e.g. "create an X/X token, where X is ...").
    let marker = super::tokens_x_x::X_DEFINED;
    let marked = !b.named.iter().any(|(n, _)| n == marker);
    if marked {
        b.named.push((marker.to_string(), Sel::None));
    }
    let e = super::value_grammar::with_x_defined(true, || {
        crate::oracle::effects::parse_clause(clause, b)
    });
    if marked {
        b.named.retain(|(n, _)| n != marker);
    }
    let e = e?;
    if format!("{:?}", b.it) == format!("{it:?}") {
        b.it = value_it;
    }
    let x = nonnegative(v);
    if targets_only {
        let mentions_x = |j: Result<String, serde_json::Error>| j.is_ok_and(|j| j.contains("\"X\""));
        if mentions_x(serde_json::to_string(&e))
            || !b.targets[first_target..]
                .iter()
                .any(|t| mentions_x(serde_json::to_string(t)))
        {
            return None;
        }
    }
    // "Return up to X target permanents ..., where X is ...": the defined X is also the
    // number of targets (or the amount divided among them).
    for i in first_target..b.targets.len() {
        b.targets[i] = substitute_x_in(&b.targets[i], &x)?;
    }
    // "Create an X/X ... token, where X is .... It deals X damage to you.": the token's X
    // is the X of the following instructions too.
    if matches!(e, Effect::CreateTokenWithPT { .. }) {
        let e = substitute_x(&e, &x)?;
        b.named
            .push((super::tokens_x_x::X_DEFINED.to_string(), Sel::None));
        return Some(Effect::seq(vec![Effect::SetX { value: x }, e]));
    }
    bind_x(&e, &x)
}

inventory::submit! { EffectPattern { name: "r107 where x is", priority: 70, parse: where_x_is } }

/// "Whenever you cast a spell with {X} in its mana cost, [effect with X]" (Zaxara): the X
/// in the effect is that spell's X (CR 107.3e).
fn cast_x_spell_trigger(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let lower = block.trim().to_lowercase();
    let eff = lower.strip_prefix("whenever you cast a spell with {x} in its mana cost, ")?;
    let x = Value::XOf(Box::new(Sel::TriggerSpell));
    // "create [token], then put X +1/+1 counters on it": "it" is the created token.
    let body = if let Some((create, counters)) = end(eff).split_once(", then put x ") {
        let first = crate::oracle::effects::parse_trigger_body(
            create,
            ctx,
            Sel::TriggerSpell,
            PlayerRef::You,
        )?;
        if !matches!(first.effect, Effect::CreateToken { .. }) {
            return None;
        }
        let (kind, rest) = crate::oracle::costs::counter_kind(counters)?;
        if end(rest) != "counters on it" {
            return None;
        }
        Body {
            effect: Effect::seq(vec![
                first.effect,
                Effect::AddCounters {
                    what: Sel::Var(vars::CREATED),
                    kind,
                    n: x.clone(),
                },
            ]),
            ..first
        }
    } else {
        let body = super::value_grammar::with_x_defined(true, || {
            crate::oracle::effects::parse_trigger_body(eff, ctx, Sel::TriggerSpell, PlayerRef::You)
        })?;
        let effect = substitute_x(&body.effect, &x)?;
        Body { effect, ..body }
    };
    let tr = TriggeredAbility::new(
        TriggerCond::CastSpell {
            who: PlayerRel::You,
            filter: Filter::and(vec![Filter::Spell, Filter::HasX]),
        },
        body,
    );
    Some(vec![AbilityDef::new(AbilityKind::Triggered(tr), block)])
}

inventory::submit! { AbilityPattern { name: "r107 cast spell with x", priority: 70, parse: cast_x_spell_trigger } }

/// "−X: [effect]" loyalty abilities (CR 107.7: [−X] means "Remove X loyalty counters from
/// this permanent"; X is announced as it's activated, CR 107.3a).
fn minus_x_loyalty(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let t = block.trim().replace('\u{2212}', "-");
    let eff = t.strip_prefix("-X: ")?;
    let body = crate::oracle::effects::parse_body(eff, ctx)?;
    let mut act = ActivatedAbility::new(
        Cost {
            mana: None,
            parts: vec![CostPart::RemoveCounters {
                kind: crate::types::counters::LOYALTY.into(),
                count: Value::X,
            }],
        },
        body,
    );
    act.is_loyalty = true;
    Some(vec![AbilityDef::new(AbilityKind::Activated(act), block)])
}

inventory::submit! { AbilityPattern { name: "r107 minus x loyalty", priority: 70, parse: minus_x_loyalty } }

/// "[trigger], you may pay [cost]. If you do, [effect]. [If you don't, [effect].]" (e.g.
/// Flameblast Dragon: "you may pay {X}{R}. If you do, it deals X damage to any target").
/// An X in the cost isn't defined by the ability, so the controller chooses it as they pay
/// (CR 107.3f).
fn may_pay_trigger(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let lower = block.trim().to_lowercase();
    if !(lower.starts_with("when ") || lower.starts_with("whenever ") || lower.starts_with("at ")) {
        return None;
    }
    let idx = lower.find(", you may pay ")?;
    let cond_s = &lower[..idx];
    let rest = &lower[idx + ", you may pay ".len()..];
    let (cost_s, then_s) = rest.split_once(". if you do, ")?;
    // "If you don't, ..." is what happens when the cost isn't paid. An "otherwise" may
    // refer to another condition in the text, so leave such text to the core compiler.
    let (then_s, else_s) = match then_s.split_once(". if you don't, ") {
        Some((a, b)) => (a, Some(b)),
        None => (then_s, None),
    };
    if then_s.contains("otherwise") || else_s.is_some_and(|e| e.contains("otherwise")) {
        return None;
    }
    let (trigger, it, it_player) = crate::oracle::triggers::parse_trigger_condition(cond_s)?;
    let (cost, loyalty) = crate::oracle::costs::parse_cost(cost_s)?;
    if loyalty {
        return None;
    }
    let mut b = Builder::new(ctx);
    b.in_trigger = true;
    b.it = it.clone();
    b.it_player = it_player.clone();
    let then = crate::oracle::effects::parse_effect_text(then_s, &mut b)?;
    let otherwise = match else_s {
        Some(e) => {
            b.it = it;
            b.it_player = it_player;
            crate::oracle::effects::parse_effect_text(e, &mut b)?
        }
        None => Effect::Noop,
    };
    let body = Body {
        targets: b.targets,
        effect: Effect::PayOptional {
            who: PlayerRef::You,
            cost,
            then: Box::new(then),
            otherwise: Box::new(otherwise),
        },
        modal: None,
    };
    let mut tr = TriggeredAbility::new(trigger, body);
    // "If you do, return ~ from your graveyard to the battlefield": it functions from the
    // graveyard (CR 113.6m).
    tr.zone = crate::oracle::triggers::trigger_zone(&tr.trigger, rest);
    Some(vec![AbilityDef::new(AbilityKind::Triggered(tr), block)])
}

inventory::submit! { AbilityPattern { name: "r107 may pay trigger", priority: 70, parse: may_pay_trigger } }

/// "sacrifice ~ unless you pay its mana cost" (CR 107.3h: an {X} in it is 0 for a
/// permanent).
fn sacrifice_unless_mana_cost(l: &str, _b: &mut Builder) -> Option<Effect> {
    if end(l) != "sacrifice ~ unless you pay its mana cost" {
        return None;
    }
    Some(Effect::PayOptional {
        who: PlayerRef::You,
        cost: Cost {
            mana: None,
            parts: vec![CostPart::PayManaCostOf(Box::new(Sel::This))],
        },
        then: Box::new(Effect::Noop),
        otherwise: Box::new(Effect::SacrificeObjects { what: Sel::This }),
    })
}

inventory::submit! { EffectPattern { name: "r107 sacrifice unless mana cost", priority: 70, parse: sacrifice_unless_mana_cost } }

/// "[player] loses half their life, rounded up" (CR 107.1a).
fn loses_half_life(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (subject, up) = if let Some(s) = l.strip_suffix(" loses half their life, rounded up") {
        (s, true)
    } else if let Some(s) = l.strip_suffix(" loses half their life, rounded down") {
        (s, false)
    } else if let Some(s) = l.strip_suffix(" lose half your life, rounded up") {
        (s, true)
    } else if let Some(s) = l.strip_suffix(" lose half your life, rounded down") {
        (s, false)
    } else if let (Some(s), Some(up)) = (
        l.strip_suffix(" loses half their life")
            .or_else(|| l.strip_suffix(" lose half your life")),
        super::value_grammar::half_rounding(),
    ) {
        // "Round up each time." after the instructions (CR 107.1a).
        (s, up)
    } else {
        return None;
    };
    let who = match subject {
        "that player" => b.it_player.clone(),
        "you" => PlayerRef::You,
        "each player" => PlayerRef::EachPlayer,
        "each opponent" => PlayerRef::EachOpponent,
        "target player" | "target opponent" => {
            let filter = if subject == "target player" {
                PlayerFilter::Any
            } else {
                PlayerFilter::Opponent
            };
            let slot = b.add_target(TargetSpec::player(filter, subject), subject);
            b.it_player = PlayerRef::Target(slot);
            PlayerRef::Target(slot)
        }
        _ => return None,
    };
    Some(Effect::ForEachPlayer {
        who,
        effect: Box::new(Effect::LoseLife {
            who: PlayerRef::Iterated,
            n: Value::Div(Box::new(Value::LifeTotal(PlayerRef::Iterated)), 2, up),
        }),
    })
}

inventory::submit! { EffectPattern { name: "r107 half life", priority: 70, parse: loses_half_life } }

/// "Enchanted creature gets +X/+Y, where X is [value], and Y is [value]." (CR 107.3p:
/// Y follows the same rules as X.)
fn enchanted_gets_xy(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let (subject, affected) = if let Some(r) = l.strip_prefix("enchanted creature gets ") {
        (r, Filter::AttachedToSource)
    } else if let Some(r) = l.strip_prefix("equipped creature gets ") {
        (r, Filter::AttachedToSource)
    } else {
        return None;
    };
    let (pt, rest) = subject.split_once(", where x is ")?;
    let tl = crate::types::TypeLine::default();
    let cctx = CompileContext {
        card_name: "",
        full_name: "",
        type_line: &tl,
        layout: crate::card::Layout::Normal,
        face_index: 0,
        keywords: &[],
        power: None,
        toughness: None,
    };
    let mut b = Builder::new(&cctx);
    let (x, tail) = value_phrase(rest, &mut b)?;
    let (p, t) = match pt {
        "+x/+y" => {
            let y_s = tail.trim_start().strip_prefix(", and y is ")?;
            let (y, tail2) = value_phrase(y_s, &mut b)?;
            if !end(&tail2).is_empty() {
                return None;
            }
            (nonnegative(x), nonnegative(y))
        }
        "+x/+x" => {
            if !end(&tail).is_empty() {
                return None;
            }
            (nonnegative(x.clone()), nonnegative(x))
        }
        _ => return None,
    };
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Continuous {
            affected,
            mods: vec![Modification::ModifyPT(p, t)],
        })),
        text,
    )])
}

fn enchanted_gets_xy_block(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let lower = block.trim().to_lowercase();
    enchanted_gets_xy(end(&lower), block, ctx)
}

inventory::submit! { AbilityPattern { name: "r107 gets +x/+y", priority: 70, parse: enchanted_gets_xy_block } }

/// "[instructions with "half ..."]. Round up each time." / "Round down each time." (CR
/// 107.1a: the text says how to round, once for every "half" in the ability).
fn round_each_time(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let t = block.trim();
    let (body, up) = if let Some(b) = t.strip_suffix(" Round up each time.") {
        (b, true)
    } else if let Some(b) = t.strip_suffix(" Round down each time.") {
        (b, false)
    } else {
        return None;
    };
    if !body.to_lowercase().contains("half ") {
        return None;
    }
    let mut abilities = super::value_grammar::with_half_rounding(up, || {
        crate::oracle::parse_ability(body, ctx)
    })?;
    for a in &mut abilities {
        std::sync::Arc::make_mut(a).text = block.to_string();
    }
    Some(abilities)
}

inventory::submit! { AbilityPattern { name: "r107 round up each time", priority: 70, parse: round_each_time } }
