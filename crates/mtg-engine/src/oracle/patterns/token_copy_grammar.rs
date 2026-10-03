//! Token creation and token copies, the parts the other token patterns don't read
//! (CR 111, 707.2, 707.9):
//!
//! ```text
//! if COND, instead create N of those tokens [that are tapped and attacking]
//!     [, where X is VALUE] [and CLAUSE]                  (more of the same tokens)
//! if COND, create N of those tokens instead
//! ```

use super::{EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::oracle::effects::{parse_clause, Builder};
use crate::oracle::phrases::*;

// ---------------------------------------------------------------------------
// "If [condition], instead create N of those tokens"
// ---------------------------------------------------------------------------

fn is_create(e: &Effect) -> bool {
    matches!(
        e,
        Effect::CreateToken { .. }
            | Effect::CreateTokenWithPT { .. }
            | Effect::CreateTokenCopy { .. }
            | Effect::CreateTokenAttached { .. }
    )
}

/// The same creation with another count (and maybe tapped and attacking).
fn with_count(e: &Effect, n: Value, tapped_attacking: bool) -> Option<Effect> {
    let mut e = e.clone();
    match &mut e {
        Effect::CreateToken {
            count,
            tapped,
            attacking,
            ..
        }
        | Effect::CreateTokenWithPT {
            count,
            tapped,
            attacking,
            ..
        }
        | Effect::CreateTokenCopy {
            count,
            tapped,
            attacking,
            ..
        } => {
            *count = n;
            if tapped_attacking {
                *tapped = true;
                *attacking = true;
            }
        }
        Effect::CreateTokenAttached { count, .. } if !tapped_attacking => *count = n,
        _ => return None,
    }
    Some(e)
}

/// "If this spell's madness cost was paid, instead create X of those tokens and you gain X
/// life." (From Under the Floorboards), "If this spell was cast from a graveyard, instead
/// create X of those tokens, where X is ..." (The Final Days), "If that creature is
/// legendary, instead create two of those tokens that are tapped and attacking" (Andúril),
/// "If that creature is a Kraken, Leviathan, Octopus, or Serpent, create two of those
/// tokens instead." (Krothuss): the previous sentence's tokens, more of them (or created
/// tapped and attacking), when the condition holds as the effect happens.
fn f_instead_more_tokens(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    let Some(r) = l.strip_prefix("if ") else {
        return false;
    };
    let (c, x) = if let Some(i) = r.find(", instead create ") {
        (&r[..i], &r[i + ", instead ".len()..])
    } else if let Some(i) = r.rfind(", create ") {
        let Some(x) = r[i + 2..].strip_suffix(" instead") else {
            return false;
        };
        (&r[..i], x)
    } else {
        return false;
    };
    let Some(x) = x.strip_prefix("create ") else {
        return false;
    };
    let (x, where_x) = match x.split_once(", where x is ") {
        Some((a, v)) => (a, Some(v)),
        None => (x, None),
    };
    let Some((n, rest)) = parse_number(x) else {
        return false;
    };
    let Some(rest) = rest.trim().strip_prefix("of those tokens") else {
        return false;
    };
    let (rest, tapped_attacking) = match rest.strip_prefix(" that are tapped and attacking") {
        Some(r) => (r, true),
        None => (rest, false),
    };
    let and_clause = match rest.trim() {
        "" => None,
        r => match r.strip_prefix("and ") {
            Some(c) => Some(c),
            None => return false,
        },
    };
    // The creation the previous sentence ends with (alone, or with one more instruction
    // that " and [clause]" restates).
    let (create, rest_count) = match &*prev {
        e if is_create(e) => (e.clone(), 0),
        Effect::Seq(v) if v.len() == 2 && is_create(&v[0]) => (v[0].clone(), 1),
        _ => return false,
    };
    if rest_count != usize::from(and_clause.is_some()) {
        return false;
    }
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let restore = |b: &mut Builder| {
        b.targets.truncate(saved.0);
        b.it = saved.1.clone();
        b.it_player = saved.2.clone();
    };
    let cond = crate::oracle::statics::parse_condition(c, b.ctx)
        .or_else(|| super::conditions_referents::parse_condition_with(c, b));
    let Some(cond) = cond else {
        restore(b);
        return false;
    };
    let mut parts = Vec::new();
    if let Some(v) = where_x {
        if !matches!(n, Value::X) {
            restore(b);
            return false;
        }
        let Some((value, tail)) = crate::oracle::statics::parse_value_phrase(v, b) else {
            restore(b);
            return false;
        };
        if !end(&tail).is_empty() {
            restore(b);
            return false;
        }
        parts.push(Effect::SetX { value });
    }
    let Some(more) = with_count(&create, n, tapped_attacking) else {
        restore(b);
        return false;
    };
    parts.push(more);
    if let Some(c2) = and_clause {
        let targets = b.targets.len();
        match parse_clause(c2, b) {
            Some(e) if b.targets.len() == targets => parts.push(e),
            _ => {
                restore(b);
                return false;
            }
        }
    }
    if b.targets.len() != saved.0 {
        restore(b);
        return false;
    }
    let old = std::mem::replace(prev, Effect::Noop);
    *prev = Effect::If {
        cond,
        then: Box::new(Effect::seq(parts)),
        otherwise: Box::new(old),
    };
    true
}

inventory::submit! { FollowupPattern { name: "token grammar: if [condition], instead create N of those tokens", priority: 59, apply: f_instead_more_tokens } }

// ---------------------------------------------------------------------------
// Creature tokens whose power and toughness a later sentence gives
// ---------------------------------------------------------------------------

/// The placeholder P/T of a creature token described without one ("Create a white Avatar
/// creature token."): it refers to nothing, so the ability isn't understood (see
/// `oracle_hardening_referents::has_no_referent`) unless a following sentence defines the
/// token's power and toughness ("It has "This token's power and toughness are each equal
/// to your life total."", "Its power is equal to that card's power ...").
fn pending_pt() -> Box<(Value, Value)> {
    let v = Value::CountSel(Box::new(super::oracle_hardening_referents::no_referent()));
    Box::new((v.clone(), v))
}

/// Whether a token's power and toughness are still to be defined by a later sentence.
pub(crate) fn is_pt_pending(spec: &TokenSpec) -> bool {
    // (An Aura token's pending enchant ability uses the same placeholder.)
    spec.pt_values
        .as_ref()
        .is_some_and(|pt| format!("{:?}", pt.0) == format!("{:?}", pending_pt().0))
}

/// Gives the one 0/0 creature token `e` creates the pending P/T; false unless there's
/// exactly one.
fn mark_pending(e: &mut Effect) -> bool {
    fn specs<'a>(e: &'a mut Effect, out: &mut Vec<&'a mut TokenSpec>) {
        match e {
            Effect::CreateToken { spec, .. } | Effect::CreateTokenAttached { spec, .. } => {
                if spec.power == Some(0)
                    && spec.toughness == Some(0)
                    && spec.pt_values.is_none()
                    && spec.card_types.contains(&crate::types::CardType::Creature)
                {
                    out.push(spec)
                }
            }
            Effect::Seq(v) => v.iter_mut().for_each(|x| specs(x, out)),
            Effect::May { effect, .. }
            | Effect::AsPlayer { effect, .. }
            | Effect::ForEachPlayer { effect, .. } => specs(effect, out),
            _ => {}
        }
    }
    let mut v = Vec::new();
    specs(e, &mut v);
    if v.len() != 1 {
        return false;
    }
    let spec = v.pop().expect("one");
    spec.power = None;
    spec.toughness = None;
    spec.pt_values = Some(pending_pt());
    true
}

/// "create a white Avatar creature token", "each player creates a green Elephant
/// creature token", "create a colorless Construct artifact creature token named Twin
/// that's attacking": a creature token without a printed power and toughness, which the
/// next sentence gives it.
fn create_pt_pending(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    if !l.contains(" creature token") || l.contains("0/0") {
        return None;
    }
    // Where the token's description starts: after "create(s) [count] [tapped]".
    let at = if l.starts_with("create ") {
        "create ".len()
    } else {
        l.find(" creates ")
            .map(|i| i + " creates ".len())
            .or_else(|| l.find(" create ").map(|i| i + " create ".len()))?
    };
    let (_, r) = parse_number(&l[at..])?;
    let r = r.trim_start();
    let r = r.strip_prefix("tapped ").unwrap_or(r);
    let desc_at = l.len() - r.len();
    let (first, _) = split_word(r);
    if first.contains('/') || l[..desc_at].contains('"') {
        return None;
    }
    let rewritten = format!("{}0/0 {}", &l[..desc_at], &l[desc_at..]);
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let parsed = crate::oracle::effects::parse_clause(&rewritten, b);
    let ok = parsed.map(|mut e| mark_pending(&mut e).then_some(e)).flatten();
    if ok.is_none() {
        b.targets.truncate(saved.0);
        b.it = saved.1;
        b.it_player = saved.2;
    }
    ok
}

inventory::submit! { EffectPattern { name: "token grammar: creature token whose P/T comes later", priority: 96, parse: create_pt_pending } }

/// The pending token spec `e` ends with.
fn pending_spec(e: &mut Effect) -> Option<&mut TokenSpec> {
    match super::tokens_copies_create::last_create(e)? {
        Effect::CreateToken { spec, .. } | Effect::CreateTokenAttached { spec, .. }
            if is_pt_pending(spec) =>
        {
            Some(spec)
        }
        _ => None,
    }
}

/// "Its power is equal to that card's power and its toughness is equal to that card's
/// toughness." (Ritual of the Returned), "The token's power and toughness are each equal
/// to ...": the power and toughness of the token just described, determined as it's
/// created and part of its copiable values (CR 111.3, 608.2h).
fn f_token_pt(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    if pending_spec(prev).is_none() {
        return false;
    }
    let r = ["its ", "the token's ", "that token's "]
        .iter()
        .find_map(|p| l.strip_prefix(p));
    let Some(r) = r else {
        return false;
    };
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let restore = |b: &mut Builder| {
        b.targets.truncate(saved.0);
        b.it = saved.1.clone();
        b.it_player = saved.2.clone();
    };
    let value = |s: &str, b: &mut Builder| -> Option<Value> {
        let (v, tail) = crate::oracle::statics::parse_value_phrase(s, b)?;
        end(&tail).is_empty().then_some(v)
    };
    let pt = if let Some(v) = r
        .strip_prefix("power and toughness are each equal to ")
        .or_else(|| r.strip_prefix("power and toughness are equal to "))
    {
        value(v, b).map(|v| (v.clone(), v))
    } else if let Some(x) = r.strip_prefix("power is equal to ") {
        let (p, t) = match x.split_once(" and its toughness is equal to ") {
            Some(pt) => pt,
            None => return false,
        };
        match (value(p, b), value(t, b)) {
            (Some(p), Some(t)) => Some((p, t)),
            _ => None,
        }
    } else {
        None
    };
    let Some(pt) = pt else {
        restore(b);
        return false;
    };
    if b.targets.len() != saved.0 {
        restore(b);
        return false;
    }
    let Some(spec) = pending_spec(prev) else {
        return false;
    };
    spec.pt_values = Some(Box::new(pt));
    b.it = Sel::Var(vars::CREATED);
    true
}

inventory::submit! { FollowupPattern { name: "token grammar: the token's power and toughness", priority: 35, apply: f_token_pt } }

// ---------------------------------------------------------------------------
// Several creations, choices between tokens, counters on the tokens
// ---------------------------------------------------------------------------

/// One creation ("create a Food token", "create X 1/1 white Halfling creature tokens"),
/// read by the token patterns; nothing else.
fn creation(clause: &str, b: &mut Builder) -> Option<Effect> {
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let e = super::tokens_copies_create::create_described(clause, b)
        .or_else(|| crate::oracle::effects::parse_simple(clause, b))
        .filter(|e| is_create(e));
    if e.is_none() {
        b.targets.truncate(saved.0);
        b.it = saved.1;
        b.it_player = saved.2;
    }
    e
}

/// "create a Food token or a Treasure token" (Tireless Provisioner), "create your choice
/// of a Blood token, a Clue token, or a Food token" (Transmutation Font): the controller
/// chooses one as the effect happens.
fn create_choice(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let r = l
        .strip_prefix("create your choice of ")
        .or_else(|| l.strip_prefix("create "))?;
    if !r.contains(" or ") || r.contains('"') {
        return None;
    }
    let norm = r.replace(", or ", ", ").replace(" or ", ", ");
    let alts: Vec<&str> = norm.split(", ").collect();
    if alts.len() < 2 {
        return None;
    }
    let mut options = Vec::new();
    for alt in alts {
        if !(alt.starts_with("a ") || alt.starts_with("an ")) || !alt.ends_with(" token") {
            return None;
        }
        let e = creation(&format!("create {alt}"), b)?;
        options.push((alt.to_string(), e));
    }
    Some(Effect::ChooseOne {
        who: PlayerRef::You,
        options,
    })
}

inventory::submit! { EffectPattern { name: "token grammar: create a token or another token", priority: 91, parse: create_choice } }

/// "create X 1/1 white Halfling creature tokens and X Food tokens" (Farmer Cotton): two
/// creations, each with its own count.
fn create_two_counted(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let r = l.strip_prefix("create ")?;
    if r.contains('"') {
        return None;
    }
    let mut from = 0;
    while let Some(i) = r[from..].find(" and ") {
        let at = from + i;
        from = at + 5;
        let (x, y) = (&r[..at], &r[at + 5..]);
        if !x.ends_with(" tokens") && !x.ends_with(" token") {
            continue;
        }
        let saved = b.targets.len();
        if let Some(ex) = creation(&format!("create {x}"), b) {
            if let Some(ey) = creation(&format!("create {y}"), b) {
                return Some(Effect::seq(vec![ex, ey]));
            }
        }
        b.targets.truncate(saved);
    }
    None
}

inventory::submit! { EffectPattern { name: "token grammar: create tokens and other tokens", priority: 92, parse: create_two_counted } }

/// "create a 0/0 green and blue Fractal creature token and put X +1/+1 counters on it,
/// where X is ..." (Fractal Anomaly), "create a 1/1 ... token with lifelink and put a
/// bloodline counter on ~" (Edgar Markov's Coffin): the creation, then the counters ("it"
/// is the token just created).
fn create_and_put_counters(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (body, where_x) = match l.rsplit_once(", where x is ") {
        Some((c, v)) => (c, Some(v)),
        None => (l, None),
    };
    let (x, y) = body.rsplit_once(" and put ")?;
    if !(x.starts_with("create ") || x.starts_with("you create ")) {
        return None;
    }
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let restore = |b: &mut Builder| {
        b.targets.truncate(saved.0);
        b.it = saved.1.clone();
        b.it_player = saved.2.clone();
    };
    let Some(ex) = creation(x, b) else {
        restore(b);
        return None;
    };
    let it_before = b.it.clone();
    b.it = Sel::Var(vars::CREATED);
    let ey = crate::oracle::effects::parse_simple(&format!("put {y}"), b);
    let Some(ey) = ey.filter(|e| matches!(e, Effect::AddCounters { .. })) else {
        restore(b);
        return None;
    };
    // "on it": the token; "on ~": the source (the pronoun keeps its old meaning after).
    if !y.ends_with(" on it") {
        b.it = it_before;
    }
    let e = Effect::seq(vec![ex, ey]);
    match where_x {
        Some(v) => {
            let Some((v, tail)) = super::r107_numbers::value_phrase(v, b)
                .or_else(|| super::statics::parse_amount(v, None).map(|x| (x, String::new())))
            else {
                restore(b);
                return None;
            };
            if !end(&tail).is_empty() {
                restore(b);
                return None;
            }
            let out = super::r107_numbers::substitute_x(&e, &super::r107_numbers::nonnegative(v));
            if out.is_none() {
                restore(b);
            }
            out
        }
        None => {
            // X must be the spell's or ability's own (CR 107.3a).
            if b.in_trigger && serde_json::to_string(&e).is_ok_and(|s| s.contains("\"X\"")) {
                restore(b);
                return None;
            }
            Some(e)
        }
    }
}

inventory::submit! { EffectPattern { name: "token grammar: create a token and put counters", priority: 93, parse: create_and_put_counters } }

/// "Sacrifice all creatures you control, then create that many 4/4 red Hellion creature
/// tokens." (Hellion Eruption): as many tokens as permanents were sacrificed this way.
fn f_create_that_many(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    let Some(r) = l.strip_prefix("create that many ") else {
        return false;
    };
    if !matches!(prev, Effect::SacrificeObjects { .. }) {
        return false;
    }
    let Some(e) = creation(&format!("create two {r}"), b) else {
        return false;
    };
    let Some(e) = with_count(&e, Value::CountSel(Box::new(Sel::Var(vars::IT))), false) else {
        return false;
    };
    let old = std::mem::replace(prev, Effect::Noop);
    *prev = Effect::seq(vec![old, e]);
    true
}

inventory::submit! { FollowupPattern { name: "token grammar: then create that many tokens", priority: 50, apply: f_create_that_many } }

/// Whether `abilities`, given to a token whose definition is pending, complete it: a
/// creature token's power and toughness (a characteristic-defining ability), an Aura
/// token's enchant ability (CR 303.4a, 702.5).
pub(crate) fn completes_pending(
    spec: &TokenSpec,
    abilities: &[Ability],
    sets_pt: fn(&Ability) -> bool,
) -> bool {
    if spec.card_types.contains(&crate::types::CardType::Creature) {
        abilities.iter().any(sets_pt)
    } else {
        abilities.iter().any(|a| {
            matches!(&a.kind, AbilityKind::Keyword(k)
                if k.kind == crate::keywords::KeywordKind::Enchant)
        })
    }
}

// ---------------------------------------------------------------------------
// Aura tokens created attached
// ---------------------------------------------------------------------------

/// "create a white Aura enchantment token named Mask attached to another target
/// permanent. The token has enchant permanent and umbra armor." (Estrid, the Masked), "...
/// named Mark of the Rani attached to another target creature. That token has enchant
/// creature and "..."" (The Rani): an Aura token entering attached (CR 303.4f-i). Its
/// enchant ability is in the next sentence; until then the definition is pending.
fn create_aura_attached(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let r = l.strip_prefix("create a ").or_else(|| l.strip_prefix("create an "))?;
    let (desc, to_s) = r.split_once(" attached to ")?;
    if !desc.contains(" aura enchantment token") || desc.contains('"') {
        return None;
    }
    let d = super::tokens_copies_create::token_desc(desc, b.ctx)?;
    if d.attacking
        || d.tapped
        || !d.spec.subtypes.iter().any(|s| s.as_str() == "Aura")
        || d.spec
            .abilities
            .iter()
            .any(|a| matches!(&a.kind, AbilityKind::Keyword(k) if k.kind == crate::keywords::KeywordKind::Enchant))
    {
        return None;
    }
    let (to, tail) = if let Some(t) = to_s.strip_prefix('~') {
        (Sel::This, t)
    } else {
        let (spec, tail) = parse_target(to_s)?;
        let text = to_s[..to_s.len() - tail.len()].trim().to_string();
        let slot = b.add_target(spec, &text);
        (Sel::Target(slot), tail)
    };
    if !end(tail).is_empty() {
        return None;
    }
    let mut spec = d.spec;
    spec.pt_values = Some(pending_pt());
    Some(Effect::CreateTokenAttached {
        spec,
        count: Value::c(1),
        controller: PlayerRef::You,
        to,
    })
}

inventory::submit! { EffectPattern { name: "token grammar: Aura token created attached", priority: 94, parse: create_aura_attached } }

// ---------------------------------------------------------------------------
// Delayed triggers about the tokens just created
// ---------------------------------------------------------------------------

/// The tokens a delayed trigger below is about, captured as it's created.
const DELAYED_TOKENS: Var = vars::USER + 7350;

/// "Exile that token when ~ leaves the battlefield." (Stangg), "Exile those tokens when ~
/// leaves the battlefield." (Mysterio, Master of Illusion), "Sacrifice ~ when that token
/// leaves the battlefield." (Stangg): a delayed triggered ability created as the ability
/// resolves, which triggers once (CR 603.7a, 603.7c), about the tokens just created. One
/// about the source leaving never triggers if the source has already left.
fn f_delayed_about_tokens(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    fn has_create(e: &Effect) -> bool {
        match e {
            Effect::Seq(v) => v.iter().any(has_create),
            e => is_create(e),
        }
    }
    if !has_create(prev) || !b.in_trigger {
        return false;
    }
    const TOKEN_WORDS: [&str; 6] = [
        "that token",
        "those tokens",
        "the token",
        "the tokens",
        "it",
        "them",
    ];
    let (instr, trigger) = if let Some(x) = l.strip_suffix(" when ~ leaves the battlefield") {
        // The instruction is about the tokens.
        let Some(w) = TOKEN_WORDS.iter().find(|w| x.ends_with(&format!(" {w}"))) else {
            return false;
        };
        let head = &x[..x.len() - w.len()];
        (
            format!("{head}{}", if w.ends_with('s') || *w == "them" { "them" } else { "it" }),
            TriggerCond::LeavesBattlefield(Filter::Source),
        )
    } else if let Some(x) = ["that token", "the token", "those tokens"]
        .iter()
        .find_map(|w| l.strip_suffix(&format!(" when {w} leaves the battlefield")))
    {
        (
            x.to_string(),
            TriggerCond::LeavesBattlefield(Filter::In(Box::new(Sel::Var(DELAYED_TOKENS)))),
        )
    } else {
        return false;
    };
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    b.it = Sel::Var(DELAYED_TOKENS);
    let parsed = crate::oracle::effects::parse_simple(&instr, b);
    let ok = match parsed {
        Some(e) if b.targets.len() == saved.0 => Some(e),
        _ => None,
    };
    b.it = saved.1.clone();
    let Some(effect) = ok else {
        b.targets.truncate(saved.0);
        b.it_player = saved.2;
        return false;
    };
    let Some((mut stores, effect)) = super::triggers_delayed::capture(&effect) else {
        return false;
    };
    stores.insert(
        0,
        Effect::Store {
            var: DELAYED_TOKENS,
            sel: Sel::Var(vars::CREATED),
        },
    );
    stores.push(Effect::DelayedTrigger {
        trigger,
        body: Box::new(Body::effect(effect)),
        once: true,
    });
    super::tokens_copies_create::append_after_create(prev, Effect::seq(stores))
}

inventory::submit! { FollowupPattern { name: "token grammar: [instruction] when ~ / that token leaves the battlefield", priority: 79, apply: f_delayed_about_tokens } }

/// "create X minus one 2/2 white Samurai creature tokens with vigilance" (Eiganjo
/// Uprising), "create X plus one ...": a count that's X changed by a number (CR 107.1b:
/// not less than 0).
fn create_x_plus_minus(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let r = l.strip_prefix("create x ")?;
    let (minus, r) = if let Some(x) = r.strip_prefix("minus ") {
        (true, x)
    } else {
        (false, r.strip_prefix("plus ")?)
    };
    let (n, rest) = parse_number(r)?;
    let Value::Const(k) = n else {
        return None;
    };
    let e = creation(&format!("create x {}", rest.trim_start()), b)?;
    let count = if minus {
        super::r107_numbers::nonnegative(Value::Diff(Box::new(Value::X), Box::new(Value::c(k))))
    } else {
        Value::Sum(vec![Value::X, Value::c(k)])
    };
    with_count(&e, count, false)
}

inventory::submit! { EffectPattern { name: "token grammar: create X minus N tokens", priority: 97, parse: create_x_plus_minus } }

// ---------------------------------------------------------------------------
// "tokens created with ~"
// ---------------------------------------------------------------------------

/// "[tokens] created with ~" (Tombstone Stairwell, Dual Nature): the tokens the source's
/// abilities created, while they remain on the battlefield (CR 607.1d; the objects a
/// resolving ability creates are linked to its source, `Game::link_to_creator`).
fn created_with_this<'a>(t: &'a str, f: &Filter) -> Option<(Filter, &'a str)> {
    let r = t.strip_prefix("created with ~")?;
    if !(r.is_empty() || r.starts_with([' ', ',', '.'])) {
        return None;
    }
    // Only tokens are created.
    if !serde_json::to_string(f).is_ok_and(|s| s.contains("\"Token\"")) {
        return None;
    }
    Some((Filter::In(Box::new(Sel::Linked)), r))
}

inventory::submit! { super::FilterSuffixPattern { name: "token grammar: created with ~", priority: 100, parse: created_with_this } }

/// "Then exile all other tokens created with ~." (Faerie Artisans): other than the tokens
/// the previous instruction just created.
fn f_other_tokens_created_with(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    let l = end(l);
    let l = l.strip_prefix("then ").unwrap_or(l);
    let (verb, r) = match l.split_once(' ') {
        Some(x) => x,
        None => return false,
    };
    if r != "all other tokens created with ~" || !matches!(verb, "exile" | "destroy" | "sacrifice") {
        return false;
    }
    if super::tokens_copies_create::last_create(prev).is_none() {
        return false;
    }
    let f = Filter::and(vec![
        Filter::Token,
        Filter::In(Box::new(Sel::Linked)),
        Filter::not(Filter::In(Box::new(Sel::Var(vars::CREATED)))),
    ]);
    let what = Sel::All(f);
    let e = match verb {
        "exile" => Effect::Exile {
            what,
            face_down: false,
            link: false,
        },
        "destroy" => Effect::Destroy {
            what,
            no_regen: false,
        },
        _ => Effect::SacrificeObjects { what },
    };
    let old = std::mem::replace(prev, Effect::Noop);
    *prev = Effect::seq(vec![old, e]);
    true
}

inventory::submit! { FollowupPattern { name: "token grammar: all other tokens created with ~", priority: 50, apply: f_other_tokens_created_with } }

// ---------------------------------------------------------------------------
// Incubate repeatedly; transforming an Incubator token
// ---------------------------------------------------------------------------

/// "Incubate 2 twice", "incubate 3 X times", "Incubate X twice, where X is the number of
/// lands you control" (CR 701.53a): the action that many times, each creating its own
/// Incubator token.
fn incubate_times(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (body, where_x) = match l.split_once(", where x is ") {
        Some((c, v)) => (c, Some(v)),
        None => (l, None),
    };
    let r = body.strip_prefix("incubate ")?;
    let (n_s, times) = if let Some(x) = r.strip_suffix(" twice") {
        (x, Value::c(2))
    } else {
        let x = r.strip_suffix(" times")?;
        let (n_s, t) = x.rsplit_once(' ')?;
        let (t, tail) = parse_number(t)?;
        if !tail.trim().is_empty() {
            return None;
        }
        (n_s, t)
    };
    let (n, tail) = parse_number(n_s)?;
    if !tail.trim().is_empty() {
        return None;
    }
    let mut parts = Vec::new();
    if let Some(v) = where_x {
        let (v, tail) = super::r107_numbers::value_phrase(v, b)
            .or_else(|| super::statics::parse_amount(v, None).map(|x| (x, String::new())))?;
        if !end(&tail).is_empty() || !matches!(n, Value::X) {
            return None;
        }
        parts.push(Effect::SetX { value: v });
    }
    parts.push(Effect::Repeat {
        times,
        effect: Box::new(super::a701_actions::keyword_action(
            KeywordAction::Incubate,
            PlayerRef::You,
            Sel::None,
            n,
        )),
    });
    Some(Effect::seq(parts))
}

inventory::submit! { EffectPattern { name: "token grammar: incubate N times", priority: 95, parse: incubate_times } }

/// "transform an Incubator token you control" (Sunder the Gateway): one chosen as the
/// effect happens.
fn transform_one_chosen(l: &str, _b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let r = l
        .strip_prefix("transform an ")
        .or_else(|| l.strip_prefix("transform a "))?;
    let (f, plural, tail) = parse_object_phrase(r)?;
    if plural || !end(tail).is_empty() || matches!(f, Filter::Any) {
        return None;
    }
    Some(Effect::Transform {
        what: Sel::Choose {
            chooser: PlayerRef::You,
            filter: Filter::and(vec![f, Filter::InZone(ZoneKind::Battlefield)]),
            count: Value::c(1),
            up_to: false,
            store: None,
        },
    })
}

inventory::submit! { EffectPattern { name: "token grammar: transform a chosen permanent", priority: 100, parse: transform_one_chosen } }

// ---------------------------------------------------------------------------
// Tokens created attacking a given player, or blocking
// ---------------------------------------------------------------------------

/// Whether `e` creates tokens attacking (`attacking`) or not attacking.
fn creates_attacking(e: &Effect, attacking: bool) -> bool {
    match e {
        Effect::CreateToken { attacking: a, .. }
        | Effect::CreateTokenWithPT { attacking: a, .. }
        | Effect::CreateTokenCopy { attacking: a, .. } => *a == attacking,
        Effect::Seq(v) => v.iter().any(|x| creates_attacking(x, attacking)),
        Effect::AsPlayer { effect, .. }
        | Effect::ForEachPlayer { effect, .. }
        | Effect::May { effect, .. } => creates_attacking(effect, attacking),
        _ => false,
    }
}

/// "create a 1/1 white Glimmer enchantment creature token that's tapped and attacking that
/// player" (Soaring Lightbringer), "that attacking player creates a tapped 2/1 ... token
/// with flying that's attacking that opponent" (Combat Calligrapher): the tokens attack
/// that player (CR 508.4). "create a 1/1 ... token with vigilance that's blocking that
/// creature" (Brimaz), "... that's blocking target creature attacking you" (Flash
/// Foliage): the tokens block that creature (CR 509.4).
fn create_join_combat(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    if !l.contains("create") {
        return None;
    }
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let restore = |b: &mut Builder| {
        b.targets.truncate(saved.0);
        b.it = saved.1.clone();
        b.it_player = saved.2.clone();
    };
    if let Some((base, who)) = [" attacking that player", " attacking that opponent"]
        .iter()
        .find_map(|p| l.strip_suffix(p).map(|x| (x, &p[" attacking ".len()..])))
    {
        let defender = if who == "that opponent"
            && b.in_trigger
            && crate::oracle::raw_text()
                .to_lowercase()
                .contains("attacks one of your opponents")
        {
            PlayerRef::TriggerPlayer
        } else {
            let p = b.it_player.clone();
            if super::oracle_hardening_referents::is_no_player_referent(&p)
                || matches!(p, PlayerRef::You)
            {
                return None;
            }
            p
        };
        let base = format!("{base} attacking");
        let e = crate::oracle::effects::parse_clause(&base, b);
        let Some(e) = e.filter(|e| creates_attacking(e, true)) else {
            restore(b);
            return None;
        };
        return Some(Effect::TokensJoinCombat {
            effect: Box::new(e),
            attacking: Some(defender),
            blocking: None,
        });
    }
    let (base, blocked) = l.split_once(" that's blocking ")?;
    let e = crate::oracle::effects::parse_clause(base, b);
    let Some(e) = e.filter(|e| creates_attacking(e, false)) else {
        restore(b);
        return None;
    };
    let (attacker, tail) = if let Some(t) = blocked.strip_prefix("that creature") {
        if !b.in_trigger || super::oracle_hardening_referents::is_no_referent(&b.it) {
            restore(b);
            return None;
        }
        (b.it.clone(), t)
    } else {
        let Some((spec, tail)) = parse_target(blocked)
            .filter(|(_, t)| end(t).is_empty())
            .or_else(|| super::basic_effects_targets::target_alternatives(blocked))
        else {
            restore(b);
            return None;
        };
        let text = blocked[..blocked.len() - tail.len()].trim().to_string();
        let slot = b.add_target(spec, &text);
        (Sel::Target(slot), tail)
    };
    if !end(tail).is_empty() {
        restore(b);
        return None;
    }
    Some(Effect::TokensJoinCombat {
        effect: Box::new(e),
        attacking: None,
        blocking: Some(attacker),
    })
}

inventory::submit! { EffectPattern { name: "token grammar: tokens attacking that player / blocking", priority: 89, parse: create_join_combat } }

/// "[creature] attacking you" (Flash Foliage: "target creature attacking you"):
/// attacking you rather than a planeswalker or battle (CR 506.3, 508.1b).
fn attacking_you<'a>(t: &'a str, f: &Filter) -> Option<(Filter, &'a str)> {
    let r = t.strip_prefix("attacking you")?;
    if !(r.is_empty() || r.starts_with([' ', ',', '.'])) || r.starts_with(" or ") {
        return None;
    }
    if !serde_json::to_string(f).is_ok_and(|s| s.contains("\"Creature\"")) {
        return None;
    }
    Some((Filter::AttackingPlayer(PlayerRel::You), r))
}

inventory::submit! { super::FilterSuffixPattern { name: "token grammar: attacking you", priority: 100, parse: attacking_you } }

// ---------------------------------------------------------------------------
// More subjects and follow-ups
// ---------------------------------------------------------------------------

/// The creations in `e` made by `who` (each creation's controller).
fn set_creator(e: &mut Effect, who: &PlayerRef) -> bool {
    match e {
        Effect::CreateToken { controller, .. }
        | Effect::CreateTokenWithPT { controller, .. }
        | Effect::CreateTokenCopy { controller, .. } => {
            if !matches!(controller, PlayerRef::You) {
                return false;
            }
            *controller = who.clone();
            true
        }
        Effect::Seq(v) => !v.is_empty() && v.iter_mut().all(|x| set_creator(x, who)),
        Effect::SetX { .. } => true,
        _ => false,
    }
}

/// "the exiled card's owner creates an X/X blue Illusion creature token, where X is the
/// mana value of the exiled card" (Skyclave Apparition): the owner of the card the
/// source's linked ability exiled creates the token (CR 111.2, 607.2a).
fn other_creator(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (who, rest) = if let Some(r) = l.strip_prefix("the exiled card's owner creates ") {
        if b.ctx.is_spell() {
            return None;
        }
        let exiled = super::imprint::exiled_card_ref("the exiled card")?;
        (PlayerRef::OwnerOf(Box::new(exiled)), r)
    } else {
        return None;
    };
    // "an X/X ... token, where X is the mana value of the exiled card" (determined as the
    // token is created, CR 608.2h).
    let (rest, mv) = match rest.strip_suffix(", where x is the mana value of the exiled card") {
        Some(r) if r.contains(" x/x ") => (r.replacen(" x/x ", " 0/0 ", 1), true),
        Some(_) => return None,
        None => (rest.to_string(), false),
    };
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let mut e = match crate::oracle::effects::parse_clause(&format!("create {rest}"), b) {
        Some(e) if b.targets.len() == saved.0 => e,
        _ => {
            b.targets.truncate(saved.0);
            b.it = saved.1;
            b.it_player = saved.2;
            return None;
        }
    };
    let mut ok = set_creator(&mut e, &who);
    if mv {
        let exiled = super::imprint::exiled_card_ref("the exiled card")?;
        let v = Value::ManaValueOf(Box::new(exiled));
        ok &= match &mut e {
            Effect::CreateToken { spec, .. }
                if spec.power == Some(0) && spec.toughness == Some(0) && spec.pt_values.is_none() =>
            {
                spec.pt_values = Some(Box::new((v.clone(), v)));
                true
            }
            _ => false,
        };
    }
    if !ok {
        b.targets.truncate(saved.0);
        b.it = saved.1;
        b.it_player = saved.2;
        return None;
    }
    Some(e)
}

inventory::submit! { EffectPattern { name: "token grammar: the exiled card's owner creates", priority: 98, parse: other_creator } }

/// "The token created this way gains haste." after populating (Determined Iteration):
/// the token the previous instruction created.
fn f_token_created_this_way(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    let Some(r) = l
        .strip_prefix("the token created this way ")
        .or_else(|| l.strip_prefix("the tokens created this way "))
    else {
        return false;
    };
    if super::tokens_copies_create::last_create(prev).is_none() {
        return false;
    }
    let saved = (b.targets.len(), b.it.clone());
    b.it = Sel::Var(vars::CREATED);
    let e = crate::oracle::effects::parse_simple(&format!("it {r}"), b);
    match e {
        Some(e @ Effect::Modify { .. }) if b.targets.len() == saved.0 => {
            super::tokens_copies_create::append_after_create(prev, e)
        }
        _ => {
            b.targets.truncate(saved.0);
            b.it = saved.1;
            false
        }
    }
}

inventory::submit! { FollowupPattern { name: "token grammar: the token created this way", priority: 40, apply: f_token_created_this_way } }

/// "If that enchantment is an Aura, you may attach it to the token." (Ajani's Chosen):
/// the object the trigger names, attached to the token just created.
fn f_attach_it_to_the_token(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    let Some(r) = l.strip_prefix("if ") else {
        return false;
    };
    let Some((c, x)) = r.split_once(", ") else {
        return false;
    };
    let (optional, x) = match x.strip_prefix("you may ") {
        Some(y) => (true, y),
        None => (false, x),
    };
    if !matches!(x, "attach it to the token" | "attach it to that token") {
        return false;
    }
    if super::tokens_copies_create::last_create(prev).is_none() || !b.in_trigger {
        return false;
    }
    // "That enchantment": the object the trigger names.
    let what = Sel::TriggerObject;
    let saved = b.it.clone();
    b.it = what.clone();
    // "that enchantment is an Aura": the trigger's object, "it".
    let c = match c.strip_prefix("that ").and_then(|x| x.split_once(" is ")) {
        Some((noun, rest)) if !noun.contains(' ') => format!("it's {rest}"),
        _ => c.to_string(),
    };
    let cond = super::conditions_referents::parse_condition_with(&c, b);
    b.it = saved;
    let Some(cond) = cond else {
        return false;
    };
    let mut e = Effect::Attach {
        what,
        to: Sel::Var(vars::CREATED),
    };
    if optional {
        e = Effect::May {
            who: PlayerRef::You,
            effect: Box::new(e),
        };
    }
    super::tokens_copies_create::append_after_create(
        prev,
        Effect::If {
            cond,
            then: Box::new(e),
            otherwise: Box::new(Effect::Noop),
        },
    )
}

inventory::submit! { FollowupPattern { name: "token grammar: attach it to the token", priority: 40, apply: f_attach_it_to_the_token } }

/// "You create a Treasure token for each opponent who was dealt damage this turn." (You've
/// Been Caught Stealing).
fn create_for_each_damaged_opponent(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let body = l.strip_suffix(" for each opponent who was dealt damage this turn")?;
    let body = body.strip_prefix("you ").unwrap_or(body);
    let e = creation(body, b)?;
    with_count(
        &e,
        Value::CountPlayers(PlayerFilter::And(vec![
            PlayerFilter::Opponent,
            PlayerFilter::DealtDamageThisTurn,
        ])),
        false,
    )
}

inventory::submit! { EffectPattern { name: "token grammar: create a token for each opponent dealt damage", priority: 97, parse: create_for_each_damaged_opponent } }

/// "Put a deathtouch counter on the token if a black card was milled this way." (Grist,
/// the Plague Swarm: "Create a 1/1 ... Insect creature token, then mill two cards."): the
/// token just created gets the counters if one of the cards the previous instruction
/// milled matches.
fn f_counter_on_token_if_milled(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    let Some((put, cond)) = l.split_once(" on the token if ") else {
        return false;
    };
    let Some(phrase) = cond
        .strip_prefix("a ")
        .or_else(|| cond.strip_prefix("an "))
        .and_then(|c| c.strip_suffix(" was milled this way"))
    else {
        return false;
    };
    // The mill is the last instruction; the token was created before it.
    let Effect::Seq(v) = &*prev else {
        return false;
    };
    let milled_last = matches!(v.last(), Some(Effect::Mill { .. }));
    let created = v.iter().any(is_create);
    if !milled_last || !created {
        return false;
    }
    let Some((f, plural, tail)) = parse_object_phrase(phrase) else {
        return false;
    };
    if plural || !end(tail).is_empty() {
        return false;
    }
    let saved = (b.targets.len(), b.it.clone());
    b.it = Sel::Var(vars::CREATED);
    let e = crate::oracle::effects::parse_simple(&format!("{put} on it"), b);
    let Some(e @ Effect::AddCounters { .. }) = e else {
        b.targets.truncate(saved.0);
        b.it = saved.1;
        return false;
    };
    let cond = Condition::SelNonEmpty(Sel::All(Filter::and(vec![
        f,
        Filter::In(Box::new(Sel::Var(vars::IT))),
    ])));
    let Effect::Seq(v) = prev else {
        return false;
    };
    v.push(Effect::If {
        cond,
        then: Box::new(e),
        otherwise: Box::new(Effect::Noop),
    });
    true
}

inventory::submit! { FollowupPattern { name: "token grammar: counter on the token if a card was milled this way", priority: 40, apply: f_counter_on_token_if_milled } }

/// "for each artifact destroyed this way, create a token that's a copy of it" (Red Sun's
/// Twilight): a token copy of each object the previous instruction destroyed.
fn for_each_destroyed_copy(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let r = l.strip_prefix("for each ")?;
    let (phrase, rest) = r.split_once(" destroyed this way, ")?;
    if rest != "create a token that's a copy of it" {
        return None;
    }
    let (f, plural, tail) = parse_object_phrase(phrase)?;
    if plural || !end(tail).is_empty() || b.ctx.is_spell() && b.targets.is_empty() {
        return None;
    }
    Some(Effect::CreateTokenCopy {
        of: Sel::All(Filter::and(vec![f, Filter::In(Box::new(Sel::Var(vars::IT)))])),
        count: Value::c(1),
        controller: PlayerRef::You,
        tapped: false,
        attacking: false,
        mods: vec![],
    })
}

inventory::submit! { EffectPattern { name: "token grammar: for each destroyed this way, a token copy of it", priority: 99, parse: for_each_destroyed_copy } }

/// "Put each card exiled with ~ into its owner's graveyard, then create a 2/2 colorless
/// Robot artifact creature token for each card put into a graveyard this way." (Pinnacle
/// Starcage).
fn f_create_for_each_put_this_way(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    let Some(body) = l.strip_suffix(" for each card put into a graveyard this way") else {
        return false;
    };
    let is_move = |e: &Effect| matches!(e, Effect::Move { .. });
    let last_move = match &*prev {
        Effect::Seq(v) => v.last().is_some_and(is_move),
        e => is_move(e),
    };
    if !last_move || !body.starts_with("create ") {
        return false;
    }
    let Some(e) = creation(body, b) else {
        return false;
    };
    let count = Value::Count(Filter::and(vec![
        Filter::Card,
        Filter::InZone(ZoneKind::Graveyard),
        Filter::In(Box::new(Sel::Var(vars::IT))),
    ]));
    let Some(e) = with_count(&e, count, false) else {
        return false;
    };
    let old = std::mem::replace(prev, Effect::Noop);
    *prev = Effect::seq(vec![old, e]);
    true
}

inventory::submit! { FollowupPattern { name: "token grammar: create a token for each card put into a graveyard this way", priority: 50, apply: f_create_for_each_put_this_way } }

/// "Target creature's controller sacrifices it, then creates X 1/1 ... tokens" (Mercy
/// Killing): the second instruction's subject is the first one's player, left out.
fn then_same_player_creates(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (a, c) = l.split_once(", then creates ")?;
    // One player named by the first instruction ("each player discards ..., then creates
    // that many ..." is each of them in turn, `hand_graveyard_grammar`).
    if a.starts_with("you ") || a.starts_with("~ ") || a.starts_with("each ") || !a.contains(' ')
    {
        return None;
    }
    let rewritten = format!("{a}, then that player creates {c}");
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let e = crate::oracle::effects::parse_clause(&rewritten, b);
    if e.is_none() {
        b.targets.truncate(saved.0);
        b.it = saved.1;
        b.it_player = saved.2;
    }
    e
}

inventory::submit! { EffectPattern { name: "token grammar: [player] ..., then creates", priority: 990, parse: then_same_player_creates } }
