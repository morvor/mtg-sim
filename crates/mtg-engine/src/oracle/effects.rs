//! Parsing effect text (the part of a spell or ability that does something) into
//! [`Effect`]s with target slots.
//!
//! Sentences are parsed independently by a list of pattern functions; each pattern
//! either fully understands its sentence or returns None. A [`Builder`] accumulates
//! target slots and tracks what pronouns ("it", "that creature") refer to.

use super::phrases::*;
use super::CompileContext;
use crate::ability::*;
use crate::keywords::KeywordKind;
use crate::mana::ManaType;
use crate::types::*;
use smol_str::SmolStr;

/// Accumulates targets while parsing a body.
pub struct Builder<'c> {
    pub targets: Vec<TargetSpec>,
    /// What "it"/"that creature" refers to.
    pub it: Sel,
    /// What "that player"/"its controller" refers to.
    pub it_player: PlayerRef,
    /// Whether we're inside a triggered ability (pronouns default to the trigger object).
    pub in_trigger: bool,
    /// Number of sentences of the current effect text parsed so far (a pronoun in the
    /// first sentence can only refer to the source, trigger object, or a target).
    pub sentences: usize,
    pub ctx: &'c CompileContext<'c>,
}

impl<'c> Builder<'c> {
    pub fn new(ctx: &'c CompileContext<'c>) -> Self {
        Builder {
            targets: vec![],
            it: Sel::This,
            it_player: PlayerRef::You,
            in_trigger: false,
            sentences: 0,
            ctx,
        }
    }
    pub fn add_target(&mut self, mut spec: TargetSpec, text: &str) -> u8 {
        spec.text = text.to_string();
        // A target player doesn't become "it" ("target opponent loses life equal to its
        // power" — "its" is still the object from before).
        let is_player = matches!(spec.what, TargetKind::Player(_));
        self.targets.push(spec);
        let slot = (self.targets.len() - 1) as u8;
        if !is_player {
            self.it = Sel::Target(slot);
        }
        slot
    }
}

/// Parses a full body (possibly modal).
pub fn parse_body(text: &str, ctx: &CompileContext) -> Option<Body> {
    let t = text.trim();
    if let Some(modal) = parse_modal(t, ctx) {
        return Some(Body {
            targets: vec![],
            effect: Effect::Noop,
            modal: Some(modal),
        });
    }
    let mut b = Builder::new(ctx);
    let effect = parse_effect_text(t, &mut b)?;
    Some(Body {
        targets: b.targets,
        effect,
        modal: None,
    })
}

/// Parses a body inside a triggered ability (pronouns refer to the trigger object).
pub fn parse_trigger_body(
    text: &str,
    ctx: &CompileContext,
    it: Sel,
    it_player: PlayerRef,
) -> Option<Body> {
    let t = text.trim();
    if let Some(modal) = parse_modal(t, ctx) {
        return Some(Body {
            targets: vec![],
            effect: Effect::Noop,
            modal: Some(modal),
        });
    }
    let mut b = Builder::new(ctx);
    b.in_trigger = true;
    b.it = it;
    b.it_player = it_player;
    let effect = parse_effect_text(t, &mut b)?;
    Some(Body {
        targets: b.targets,
        effect,
        modal: None,
    })
}

/// "Choose one —\n• mode\n• mode" (CR 700.2).
fn parse_modal(t: &str, ctx: &CompileContext) -> Option<Modal> {
    let (head, rest) = t.split_once('\n')?;
    let hl = head.to_lowercase();
    let hl = hl.trim().trim_end_matches(['—', ':', ' ']);
    let (min, max) = match hl {
        "choose one" => (1, 1),
        "choose two" => (2, 2),
        "choose three" => (3, 3),
        "choose one or both" => (1, 2),
        "choose one or more" => (1, 99),
        "choose any number" => (0, 99),
        "choose one or two" => (1, 2),
        _ => return None,
    };
    let mut modes = Vec::new();
    for line in rest.lines() {
        let l = line.trim().trim_start_matches('•').trim();
        if l.is_empty() {
            continue;
        }
        let mut b = Builder::new(ctx);
        let effect = parse_effect_text(l, &mut b)?;
        modes.push(Mode {
            text: l.to_string(),
            targets: b.targets,
            effect,
            cost: None,
        });
    }
    if modes.is_empty() {
        return None;
    }
    let max = max.min(modes.len() as i32);
    Some(Modal {
        min: Value::Const(min),
        max: Value::Const(max),
        allow_repeat: false,
        modes,
        per_mode_cost: false,
        chooser: ModeChooser::Controller,
    })
}

/// Splits text into sentences at ". " boundaries outside quotes.
pub fn split_sentences(t: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut in_quote = false;
    let chars: Vec<char> = t.chars().collect();
    for (i, ch) in chars.iter().enumerate() {
        cur.push(*ch);
        if *ch == '"' {
            in_quote = !in_quote;
        }
        if *ch == '.' && !in_quote && (i + 1 == chars.len() || chars[i + 1] == ' ') {
            let s = cur.trim().to_string();
            if !s.is_empty() {
                out.push(s);
            }
            cur.clear();
        }
    }
    let s = cur.trim().to_string();
    if !s.is_empty() {
        out.push(s);
    }
    out
}

/// Parses effect text (one or more sentences).
pub fn parse_effect_text(t: &str, b: &mut Builder) -> Option<Effect> {
    let mut effects = Vec::new();
    for s in split_sentences(t) {
        // Sentences that modify the previous one ("It can't be regenerated.").
        if let Some(prev) = effects.last_mut() {
            if crate::oracle_ext::apply_followup_ext(&s, prev, b) {
                b.sentences += 1;
                continue;
            }
        }
        effects.push(parse_sentence(&s, b)?);
        b.sentences += 1;
    }
    Some(Effect::seq(effects))
}

/// Parses one sentence.
pub fn parse_sentence(s: &str, b: &mut Builder) -> Option<Effect> {
    let lower = s.to_lowercase();
    let l = end(&lower);
    let l = l.strip_prefix("then ").unwrap_or(l);
    // Conditionals.
    if let Some(r) = l.strip_prefix("if you do, ") {
        let e = parse_clause(r, b)?;
        return Some(Effect::If {
            cond: Condition::PrevHappened,
            then: Box::new(e),
            otherwise: Box::new(Effect::Noop),
        });
    }
    if let Some(r) = l.strip_prefix("if you don't, ") {
        let e = parse_clause(r, b)?;
        return Some(Effect::If {
            cond: Condition::Not(Box::new(Condition::PrevHappened)),
            then: Box::new(e),
            otherwise: Box::new(Effect::Noop),
        });
    }
    if let Some(r) = l.strip_prefix("you may ") {
        let e = parse_clause(r, b)?;
        return Some(Effect::May {
            who: PlayerRef::You,
            effect: Box::new(e),
        });
    }
    if let Some((cond, rest)) = parse_leading_if(l, b) {
        // "If ~ was kicked, it deals 2 damage ...": the subject "it" is the condition's.
        let subject_is_source;
        let rest = match rest.strip_prefix("it ") {
            Some(r) if l.starts_with("if ~ ") => {
                subject_is_source = format!("~ {r}");
                subject_is_source.as_str()
            }
            _ => rest,
        };
        let first_new_target = b.targets.len();
        let e = parse_clause(rest, b)?;
        // CR 601.2c, 702.33g: targets of a part that has its effect only if an optional
        // cost was paid as the spell was cast are chosen only if it was paid.
        if matches!(cond, Condition::CostPaid(_)) {
            for spec in &mut b.targets[first_new_target..] {
                spec.condition = Some(cond.clone());
            }
        }
        return Some(Effect::If {
            cond,
            then: Box::new(e),
            otherwise: Box::new(Effect::Noop),
        });
    }
    parse_clause(l, b)
}

/// "if [condition], [effect]"
fn parse_leading_if<'a>(l: &'a str, b: &mut Builder) -> Option<(Condition, &'a str)> {
    let r = l.strip_prefix("if ")?;
    let (c, rest) = r.split_once(", ")?;
    let cond = super::statics::parse_condition(c, b.ctx)?;
    Some((cond, rest))
}

/// Parses a clause, splitting on ", then " and " and " joins where both halves parse.
pub fn parse_clause(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    if let Some(e) = parse_simple(l, b) {
        return Some(e);
    }
    for sep in [", then ", " and then ", ". then ", ", and ", " and "] {
        if let Some((a, c)) = l.split_once(sep) {
            let saved_targets = b.targets.len();
            let saved_it = b.it.clone();
            if let Some(mut ea) = parse_simple(a, b) {
                // The second half may modify the first ("exile it, then return it").
                if matches!(sep, ", then " | " and then ")
                    && crate::oracle_ext::apply_followup_ext(c, &mut ea, b)
                {
                    return Some(ea);
                }
                // Second half may omit the subject: "draw a card and lose 1 life".
                if let Some(ec) = parse_simple(c, b).or_else(|| parse_clause(c, b)) {
                    return Some(Effect::seq(vec![ea, ec]));
                }
            }
            b.targets.truncate(saved_targets);
            b.it = saved_it;
        }
    }
    None
}

/// Resolves pronoun/self references to a selection.
pub fn object_ref(s: &str, b: &mut Builder) -> Option<(Sel, String)> {
    let s = s.trim();
    let pairs: [(&str, Sel); 3] = [
        ("~", Sel::This),
        ("enchanted creature", Sel::AttachedTo),
        ("equipped creature", Sel::AttachedTo),
    ];
    for (p, sel) in pairs {
        if let Some(rest) = s.strip_prefix(p) {
            return Some((sel, rest.to_string()));
        }
    }
    for p in [
        "it",
        "that creature",
        "that permanent",
        "that card",
        "them",
        "those creatures",
        "that spell",
        "the creature",
        "that token",
        "those cards",
        "this token",
    ] {
        if let Some(rest) = s.strip_prefix(p) {
            if rest.is_empty() || rest.starts_with(' ') || rest.starts_with('\'') {
                return Some((b.it.clone(), rest.to_string()));
            }
        }
    }
    if let Some((spec, rest)) = parse_any_target(s) {
        let text = s[..s.len() - rest.len()].trim().to_string();
        let slot = b.add_target(spec, &text);
        return Some((Sel::Target(slot), rest.to_string()));
    }
    if let Some(r) = s.strip_prefix("each ").or_else(|| s.strip_prefix("all ")) {
        let (f, _, rest) = parse_object_phrase(r)?;
        let (f, rest) = bind_target_player(f, rest, b);
        return Some((Sel::All(f), rest));
    }
    // Bare plural noun phrases ("creatures you control") mean all such objects.
    if let Some((f, plural, rest)) = parse_object_phrase(s) {
        if plural {
            let (f, rest) = bind_target_player(f, rest, b);
            return Some((Sel::All(f), rest));
        }
    }
    None
}

/// "[objects] target player controls": adds the player target and restricts the filter
/// to objects that player controls. Returns the filter and the rest of the text.
pub fn bind_target_player(f: Filter, rest: &str, b: &mut Builder) -> (Filter, String) {
    match target_player_controls(rest) {
        Some((pf, text, r)) => {
            let it = b.it.clone();
            let slot = b.add_target(TargetSpec::player(pf, text), text);
            b.it = it;
            b.it_player = PlayerRef::Target(slot);
            (
                Filter::and(vec![f, Filter::ControlledBy(PlayerRel::Target(slot))]),
                r.to_string(),
            )
        }
        None => (f, rest.to_string()),
    }
}

pub fn player_ref(s: &str, b: &mut Builder) -> Option<(PlayerRef, String)> {
    let s = s.trim();
    if let Some(r) = s.strip_prefix("that player") {
        return Some((b.it_player.clone(), r.to_string()));
    }
    if let Some(r) = s
        .strip_prefix("its controller")
        .or_else(|| s.strip_prefix("their controller"))
    {
        return Some((
            PlayerRef::ControllerOf(Box::new(b.it.clone())),
            r.to_string(),
        ));
    }
    if let Some(r) = s
        .strip_prefix("its owner")
        .or_else(|| s.strip_prefix("their owner"))
    {
        return Some((PlayerRef::OwnerOf(Box::new(b.it.clone())), r.to_string()));
    }
    let with_space = format!("{s} ");
    if let Some((r, spec, rest)) = parse_player(&with_space) {
        let rest = rest.to_string();
        if let Some(spec) = spec {
            let slot = b.add_target(spec, "target player");
            b.it_player = PlayerRef::Target(slot);
            return Some((PlayerRef::Target(slot), rest));
        }
        return Some((r, rest));
    }
    None
}

/// Duration suffix: "until end of turn", "this turn", "until your next turn".
pub fn duration_suffix(s: &str) -> (Duration, &str) {
    let t = s.trim();
    for (p, d) in [
        (" until end of turn", Duration::EndOfTurn),
        (" this turn", Duration::EndOfTurn),
        (" until your next turn", Duration::UntilYourNextTurn),
        (" until end of combat", Duration::EndOfCombat),
        (
            " for as long as ~ remains on the battlefield",
            Duration::WhileSourceOnBattlefield,
        ),
        (
            " for as long as you control ~",
            Duration::WhileYouControlSource,
        ),
    ] {
        if let Some(r) = t.strip_suffix(p) {
            return (d, r);
        }
    }
    (Duration::Permanent, t)
}

pub fn keyword_mods(s: &str) -> Option<Vec<Modification>> {
    // "flying", "flying and trample", "first strike, vigilance, and lifelink",
    // "hexproof and indestructible", "protection from red"
    let parts = super::keywords::split_keyword_phrases(s);
    let mut out = Vec::new();
    for p in &parts {
        let lines = super::keywords::parse_keyword_line(p, &dummy_ctx())?;
        for a in lines {
            if let AbilityKind::Keyword(k) = &a.kind {
                out.push(Modification::AddKeyword(k.clone()));
            }
        }
    }
    if out.is_empty() {
        None
    } else {
        Some(out)
    }
}

fn dummy_ctx() -> CompileContext<'static> {
    use std::sync::OnceLock;
    static TL: OnceLock<TypeLine> = OnceLock::new();
    let tl = TL.get_or_init(TypeLine::default);
    CompileContext {
        card_name: "",
        full_name: "",
        type_line: tl,
        layout: crate::card::Layout::Normal,
        face_index: 0,
        keywords: &[],
        power: None,
        toughness: None,
    }
}

/// Parses "+N/+N", "-N/-N", "+X/+0".
pub fn parse_pt_mod(s: &str) -> Option<(Value, Value, &str)> {
    let s = s.trim_start();
    let (w, rest) = split_word(s);
    let (p, t) = w.split_once('/')?;
    let pv = signed_value(p)?;
    let tv = signed_value(t)?;
    Some((pv, tv, rest))
}

fn signed_value(s: &str) -> Option<Value> {
    let neg = s.starts_with('-');
    let body = s.trim_start_matches(['+', '-']);
    let v = if body == "x" {
        Value::X
    } else {
        Value::Const(body.parse().ok()?)
    };
    Some(if neg {
        match v {
            Value::Const(n) => Value::Const(-n),
            other => Value::Diff(Box::new(Value::Const(0)), Box::new(other)),
        }
    } else {
        v
    })
}

/// The main pattern list. Each returns Some if it fully understands the clause.
pub fn parse_simple(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    type P = fn(&str, &mut Builder) -> Option<Effect>;
    const PATTERNS: &[P] = &[
        p_damage,
        p_draw,
        p_life,
        p_destroy,
        p_exile,
        p_return,
        p_counter_spell,
        p_pump,
        p_gains,
        p_add_mana,
        p_create_token,
        p_counters,
        p_tap_untap,
        p_search,
        p_scry_surveil_mill,
        p_cant,
        p_sacrifice,
        p_discard,
        p_fight,
        p_gain_control,
        p_simple_actions,
        p_shuffle,
    ];
    for p in PATTERNS {
        let saved_targets = b.targets.len();
        let saved_it = b.it.clone();
        let saved_player = b.it_player.clone();
        if let Some(e) = p(l, b) {
            return Some(e);
        }
        b.targets.truncate(saved_targets);
        b.it = saved_it;
        b.it_player = saved_player;
    }
    crate::oracle_ext::parse_effect_ext(l, b)
}

// ---------------------------------------------------------------------------
// Patterns
// ---------------------------------------------------------------------------

/// "~ deals N damage to X", "~ deals damage equal to ... to X", divided damage.
fn p_damage(l: &str, b: &mut Builder) -> Option<Effect> {
    let (src, rest): (Sel, String) = if let Some(r) = l.strip_prefix("~ deals ") {
        (Sel::This, r.to_string())
    } else if let Some((sel, r)) = object_ref(l, b) {
        (sel, r.trim_start().strip_prefix("deals ")?.to_string())
    } else {
        return None;
    };
    let rest = rest.as_str();
    let owned_tail: String;
    let (amount, rest) = if let Some(r) = rest.strip_prefix("damage equal to ") {
        let (v, r2) = super::statics::parse_value_phrase(r, b)?;
        owned_tail = r2.trim_start().strip_prefix("to ")?.to_string();
        (v, owned_tail.as_str())
    } else {
        let (n, r) = parse_number(rest)?;
        let r = strip(r, "damage")?;
        // "divided as you choose among any number of targets"
        if let Some(r2) = strip(r, "divided as you choose among ") {
            let replaced = r2.replace("any number of targets", "any target");
            let (mut spec, tail) =
                parse_any_target(replaced.as_str()).map(|(s, t)| (s, t.to_string()))?;
            if !end(&tail).is_empty() {
                return None;
            }
            spec.min = 1;
            spec.max = n.clone();
            spec.divide = Some(n);
            let slot = b.add_target(spec, "targets (divided)");
            return Some(Effect::DealDividedDamage { source: src, slot });
        }
        (n, strip(r, "to")?)
    };
    // Recipients.
    let (to, tail) = damage_recipients(rest, b)?;
    if !end(&tail).is_empty() {
        return None;
    }
    Some(Effect::DealDamage {
        source: src,
        amount,
        to,
    })
}

fn damage_recipients(s: &str, b: &mut Builder) -> Option<(Sel, String)> {
    let s = s.trim();
    let fixed: [(&str, Sel); 8] = [
        ("each opponent", Sel::Players(PlayerRef::EachOpponent)),
        ("each player", Sel::Players(PlayerRef::EachPlayer)),
        ("you", Sel::Players(PlayerRef::You)),
        ("that player", Sel::Players(b.it_player.clone())),
        (
            "its controller",
            Sel::Players(PlayerRef::ControllerOf(Box::new(b.it.clone()))),
        ),
        ("defending player", Sel::Players(PlayerRef::DefendingPlayer)),
        (
            "each other player",
            Sel::Players(PlayerRef::EachOtherPlayer),
        ),
        ("target player", Sel::None),
    ];
    // "each creature and each player", "each creature without flying and each player"
    if let Some(r) = s.strip_prefix("each ") {
        if let Some((obj, rest)) = r.split_once(" and each player") {
            let (f, _, t) = parse_object_phrase(obj)?;
            if !end(t).is_empty() {
                return None;
            }
            return Some((
                Sel::Union(vec![Sel::All(f), Sel::Players(PlayerRef::EachPlayer)]),
                rest.to_string(),
            ));
        }
    }
    for (p, sel) in fixed {
        if let Some(rest) = s.strip_prefix(p) {
            if matches!(sel, Sel::None) {
                break;
            }
            if rest.is_empty() || rest.starts_with(' ') || rest.starts_with(',') {
                return Some((sel, rest.to_string()));
            }
        }
    }
    let (sel, rest) = object_ref(s, b)?;
    Some((sel, rest))
}

/// "draw N cards", "target player draws N cards", "each player draws a card".
fn p_draw(l: &str, b: &mut Builder) -> Option<Effect> {
    if let Some(r) = l.strip_prefix("draw ") {
        let (n, rest) = parse_card_count(r)?;
        if !end(rest).is_empty() {
            return None;
        }
        return Some(Effect::Draw {
            who: PlayerRef::You,
            n,
        });
    }
    let (who, rest) = player_ref(l, b)?;
    let r = rest.trim().strip_prefix("draws ")?;
    let (n, tail) = parse_card_count(r)?;
    if !end(tail).is_empty() {
        return None;
    }
    Some(Effect::Draw { who, n })
}

/// "you gain N life", "target player loses N life", "each opponent loses N life".
fn p_life(l: &str, b: &mut Builder) -> Option<Effect> {
    let (who, rest) = if let Some(r) = l.strip_prefix("gain ") {
        (PlayerRef::You, format!("gains {r}"))
    } else if let Some(r) = l.strip_prefix("lose ") {
        (PlayerRef::You, format!("loses {r}"))
    } else {
        let (p, r) = player_ref(l, b)?;
        (p, r.trim().to_string())
    };
    let rest = rest.trim();
    let (gain, r) = if let Some(r) = rest
        .strip_prefix("gains ")
        .or_else(|| rest.strip_prefix("gain "))
    {
        (true, r)
    } else if let Some(r) = rest
        .strip_prefix("loses ")
        .or_else(|| rest.strip_prefix("lose "))
    {
        (false, r)
    } else {
        return None;
    };
    let (n, tail) = if let Some(r2) = r.strip_prefix("life equal to ") {
        super::statics::parse_value_phrase(r2, b)?
    } else {
        let (n, r2) = parse_number(r)?;
        (n, strip(r2, "life")?.to_string())
    };
    if !end(&tail).is_empty() {
        return None;
    }
    Some(if gain {
        Effect::GainLife { who, n }
    } else {
        Effect::LoseLife { who, n }
    })
}

/// "destroy target creature", "destroy all creatures", "... it can't be regenerated".
fn p_destroy(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = l.strip_prefix("destroy ")?;
    let (r, no_regen) = match r
        .strip_suffix(". it can't be regenerated")
        .or_else(|| r.strip_suffix(". they can't be regenerated"))
    {
        Some(x) => (x, true),
        None => (r, false),
    };
    let (what, tail) = object_ref(r, b)?;
    if !end(&tail).is_empty() {
        return None;
    }
    Some(Effect::Destroy { what, no_regen })
}

/// "exile target creature", "exile all graveyards"...
fn p_exile(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = l.strip_prefix("exile ")?;
    let (what, tail) = object_ref(r, b)?;
    if !end(&tail).is_empty() {
        return None;
    }
    Some(Effect::Exile {
        what,
        face_down: false,
        link: false,
    })
}

/// "return target creature to its owner's hand", "return target creature card from your
/// graveyard to your hand", "... to the battlefield [tapped]".
fn p_return(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = l.strip_prefix("return ")?;
    let (what, tail) = object_ref(r, b)?;
    let t = tail.trim();
    let t = t
        .strip_prefix("from your graveyard")
        .map(str::trim)
        .unwrap_or(t);
    let t = t
        .strip_prefix("from a graveyard")
        .map(str::trim)
        .unwrap_or(t);
    let to = if t == "to its owner's hand"
        || t == "to their owners' hands"
        || t == "to your hand"
        || t == "to their owner's hand"
    {
        Destination::zone(ZoneKind::Hand)
    } else if t == "to the battlefield" || t == "to the battlefield under your control" {
        Destination::battlefield().under_your_control()
    } else if t == "to the battlefield tapped"
        || t == "to the battlefield tapped under your control"
    {
        Destination::battlefield().under_your_control().tapped()
    } else if t == "to the battlefield under its owner's control" {
        let mut d = Destination::battlefield();
        d.controller = Some(PlayerRef::OwnerOf(Box::new(what.clone())));
        d
    } else if t == "to the top of its owner's library" || t == "on top of its owner's library" {
        Destination::library_top()
    } else {
        return None;
    };
    Some(Effect::Move { what, to })
}

/// "counter target spell", "counter target creature spell", "counter target spell
/// unless its controller pays {N}".
fn p_counter_spell(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = l.strip_prefix("counter ")?;
    if let Some((spell, pays)) = r.split_once(" unless its controller pays ") {
        let (what, tail) = object_ref(spell, b)?;
        if !end(&tail).is_empty() {
            return None;
        }
        let cost = super::keywords::parse_keyword_cost(pays)?;
        return Some(Effect::PayOptional {
            who: PlayerRef::ControllerOf(Box::new(what.clone())),
            cost,
            then: Box::new(Effect::Noop),
            otherwise: Box::new(Effect::CounterSpell { what }),
        });
    }
    let (what, tail) = object_ref(r, b)?;
    if !end(&tail).is_empty() {
        return None;
    }
    Some(Effect::CounterSpell { what })
}

/// "target creature gets +3/+3 until end of turn", "creatures you control get +1/+1
/// until end of turn", "~ gets +2/+0 and gains flying until end of turn".
fn p_pump(l: &str, b: &mut Builder) -> Option<Effect> {
    let (dur, l) = duration_suffix(l);
    let (what, rest) = object_ref(l, b)?;
    let rest = rest.trim();
    let r = rest
        .strip_prefix("gets ")
        .or_else(|| rest.strip_prefix("get "))?;
    let (p, t, tail) = parse_pt_mod(r)?;
    let mut mods = vec![Modification::ModifyPT(p, t)];
    let tail = tail.trim();
    if !tail.is_empty() {
        let k = tail
            .strip_prefix("and gains ")
            .or_else(|| tail.strip_prefix("and gain "))?;
        mods.extend(keyword_mods(k)?);
    }
    Some(Effect::Modify {
        what,
        mods,
        duration: dur,
    })
}

/// "target creature gains flying until end of turn", "creatures you control gain
/// indestructible until end of turn".
fn p_gains(l: &str, b: &mut Builder) -> Option<Effect> {
    let (dur, l) = duration_suffix(l);
    let (what, rest) = object_ref(l, b)?;
    let rest = rest.trim();
    let r = rest
        .strip_prefix("gains ")
        .or_else(|| rest.strip_prefix("gain "))?;
    let mods = keyword_mods(r)?;
    Some(Effect::Modify {
        what,
        mods,
        duration: dur,
    })
}

/// "add {G}", "add {G}{G}", "add one mana of any color", "add {R} or {G}", "add {C}{C}".
fn p_add_mana(l: &str, _b: &mut Builder) -> Option<Effect> {
    let r = l.strip_prefix("add ")?;
    let mana = if let Some(x) = r.strip_prefix("one mana of any color") {
        if !end(x).is_empty() {
            return None;
        }
        ManaProduction::AnyOneColor(Value::c(1))
    } else if let Some(x) = r.strip_prefix("two mana of any one color") {
        if !end(x).is_empty() {
            return None;
        }
        ManaProduction::AnyOneColor(Value::c(2))
    } else if let Some(x) = r.strip_prefix("three mana of any one color") {
        if !end(x).is_empty() {
            return None;
        }
        ManaProduction::AnyOneColor(Value::c(3))
    } else if let Some(x) = r.strip_prefix("one mana of the chosen color") {
        if !end(x).is_empty() {
            return None;
        }
        ManaProduction::ChosenColor(Value::c(1))
    } else if r.contains(" or ") {
        // "{R} or {G}", "{W}, {U}, or {B}"
        let opts: Vec<ManaType> = r
            .split(|c| c == ',' || c == ' ')
            .filter(|w| w.starts_with('{'))
            .map(|w| {
                w.trim_matches(|c| c == '{' || c == '}' || c == '.')
                    .to_uppercase()
            })
            .filter_map(|w| w.chars().next().and_then(ManaType::from_letter))
            .collect();
        if opts.len() < 2 {
            return None;
        }
        ManaProduction::OneOf(opts)
    } else {
        let syms: Vec<ManaType> = r
            .split('}')
            .filter_map(|x| x.trim().strip_prefix('{'))
            .map(|w| w.to_uppercase())
            .filter_map(|w| {
                if w.len() == 1 {
                    w.chars().next().and_then(ManaType::from_letter)
                } else {
                    None
                }
            })
            .collect();
        let rebuilt: String = syms
            .iter()
            .map(|t| format!("{{{t:?}}}").to_lowercase())
            .collect();
        if syms.is_empty() || end(r).replace(' ', "") != rebuilt {
            return None;
        }
        ManaProduction::Fixed(syms)
    };
    Some(Effect::AddMana {
        who: PlayerRef::You,
        mana,
        restriction: None,
    })
}

/// "create a 1/1 white Soldier creature token", "create two Treasure tokens",
/// "create a 2/2 black Zombie creature token with deathtouch".
fn p_create_token(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = l.strip_prefix("create ")?;
    let (count, r) = parse_number(r)?;
    let r = r.trim();
    let (r, tapped) = match r.strip_prefix("tapped ") {
        Some(x) => (x, true),
        None => (r, false),
    };
    // Predefined tokens: "Treasure token(s)", "Food token", "Clue token".
    let (w, rest) = split_word(r);
    if let Some(spec) = crate::tokens::predefined(w) {
        let rest = rest.trim();
        if rest == "token" || rest == "tokens" {
            return Some(Effect::CreateToken {
                spec,
                count,
                controller: PlayerRef::You,
                tapped,
                attacking: false,
            });
        }
    }
    let spec = parse_token_description(r)?;
    let _ = b;
    Some(Effect::CreateToken {
        spec,
        count,
        controller: PlayerRef::You,
        tapped,
        attacking: false,
    })
}

/// "1/1 white Soldier creature token with flying" → TokenSpec.
pub fn parse_token_description(s: &str) -> Option<TokenSpec> {
    let s = s.trim();
    let (pt, mut rest) = split_word(s);
    let (p, t) = pt.split_once('/')?;
    let power: i32 = p.parse().ok()?;
    let toughness: i32 = t.parse().ok()?;
    let mut colors = ColorSet::NONE;
    let mut types = Vec::new();
    let mut subtypes = Vec::new();
    let mut supertypes = Vec::new();
    let mut abilities: Vec<Ability> = Vec::new();
    let name = SmolStr::default();
    loop {
        let (w, r) = split_word(rest);
        let w = w.trim_end_matches(',');
        if w.is_empty() {
            break;
        }
        if w == "and" {
            rest = r;
            continue;
        }
        if let Some(c) = Color::from_word(w) {
            colors.insert(c);
        } else if w == "colorless" {
        } else if let Some(st) = Supertype::from_word(w) {
            supertypes.push(st);
        } else if w == "token" || w == "tokens" {
            rest = r;
            break;
        } else if let Some(ct) = CardType::from_word(w) {
            types.push(ct);
        } else if let Some(sub) = subtype_word(w) {
            subtypes.push(sub);
        } else {
            return None;
        }
        rest = r;
    }
    let rest = rest.trim();
    if let Some(r) = rest.strip_prefix("with ") {
        let r = end(r);
        let mods = keyword_mods(r)?;
        for m in mods {
            if let Modification::AddKeyword(k) = m {
                abilities.push(AbilityDef::new(
                    AbilityKind::Keyword(k.clone()),
                    k.kind.name(),
                ));
            }
        }
    } else if !end(rest).is_empty() {
        // Named tokens ("named Wasp") are left to the `tokens_copies_create` pattern,
        // which keeps the name's original case and stops at the clause after the name
        // ("named Cadet, where X is ...", "named Blue Horror with "..."").
        return None;
    }
    if !types.contains(&CardType::Creature) {
        return None;
    }
    Some(TokenSpec {
        name,
        colors,
        supertypes,
        card_types: types,
        subtypes,
        power: Some(power),
        toughness: Some(toughness),
        abilities,
        scryfall_name: None,
    })
}

/// "put a +1/+1 counter on target creature", "put two +1/+1 counters on ~",
/// "put a +1/+1 counter on each creature you control".
fn p_counters(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = l.strip_prefix("put ")?;
    let (n, r) = parse_number(r)?;
    let (kind, r) = super::costs::counter_kind(r)?;
    let r = strip(r, "counters").or_else(|| strip(r, "counter"))?;
    let r = strip(r, "on")?;
    let (what, tail) = object_ref(r, b)?;
    if !end(&tail).is_empty() {
        return None;
    }
    Some(Effect::AddCounters { what, kind, n })
}

/// "tap target creature", "untap target land", "tap all creatures your opponents control".
fn p_tap_untap(l: &str, b: &mut Builder) -> Option<Effect> {
    if let Some(r) = l.strip_prefix("tap ") {
        let (what, tail) = object_ref(r, b)?;
        if !end(&tail).is_empty() {
            return None;
        }
        return Some(Effect::Tap { what });
    }
    if let Some(r) = l.strip_prefix("untap ") {
        let (what, tail) = object_ref(r, b)?;
        if !end(&tail).is_empty() {
            return None;
        }
        return Some(Effect::Untap { what });
    }
    None
}

/// "search your library for a basic land card, put it onto the battlefield tapped, then shuffle".
fn p_search(l: &str, _b: &mut Builder) -> Option<Effect> {
    let r = l.strip_prefix("search your library for ")?;
    let (count, r) = if let Some(r2) = r.strip_prefix("up to ") {
        let (n, r3) = parse_number(r2)?;
        (n, r3)
    } else {
        parse_number(r)?
    };
    let (filter, _, rest) = parse_object_phrase(r)?;
    let rest = rest.trim().trim_start_matches(',').trim();
    let (dest_s, shuffle) = if let Some(x) = rest
        .strip_suffix(", then shuffle")
        .or_else(|| rest.strip_suffix(" then shuffle"))
    {
        (x, true)
    } else {
        (rest, false)
    };
    let dest_s = dest_s
        .trim_start_matches("reveal it, ")
        .trim_start_matches("reveal them, ");
    let to = match dest_s {
        "put it into your hand" | "put that card into your hand" | "put them into your hand" => {
            Destination::zone(ZoneKind::Hand)
        }
        "put it onto the battlefield" | "put them onto the battlefield" => {
            Destination::battlefield().under_your_control()
        }
        "put it onto the battlefield tapped" | "put them onto the battlefield tapped" => {
            Destination::battlefield().under_your_control().tapped()
        }
        "put it into your graveyard" => Destination::zone(ZoneKind::Graveyard),
        "put that card on top" | "put it on top of your library" => Destination::library_top(),
        _ => return None,
    };
    let _ = count.clone();
    let filter = Filter::and(vec![filter, Filter::InZone(ZoneKind::Library)]);
    Some(Effect::Search {
        who: PlayerRef::You,
        whose: PlayerRef::You,
        filter,
        count,
        to,
        reveal: rest.contains("reveal"),
        shuffle,
    })
}

/// "scry 2", "surveil 1", "mill three cards", "target player mills two cards".
fn p_scry_surveil_mill(l: &str, b: &mut Builder) -> Option<Effect> {
    if let Some(r) = l.strip_prefix("scry ") {
        let (n, t) = parse_number(r)?;
        return end(t).is_empty().then_some(Effect::Scry {
            who: PlayerRef::You,
            n,
        });
    }
    if let Some(r) = l.strip_prefix("surveil ") {
        let (n, t) = parse_number(r)?;
        return end(t).is_empty().then_some(Effect::Surveil {
            who: PlayerRef::You,
            n,
        });
    }
    if let Some(r) = l.strip_prefix("mill ") {
        let (n, t) = parse_card_count(r)?;
        return end(t).is_empty().then_some(Effect::Mill {
            who: PlayerRef::You,
            n,
        });
    }
    let (who, rest) = player_ref(l, b)?;
    let r = rest.trim().strip_prefix("mills ")?;
    let (n, t) = parse_card_count(r)?;
    end(t).is_empty().then_some(Effect::Mill { who, n })
}

/// "target creature can't block this turn", "~ can't be blocked this turn".
fn p_cant(l: &str, b: &mut Builder) -> Option<Effect> {
    let (dur, l) = duration_suffix(l);
    if matches!(dur, Duration::Permanent) {
        return None;
    }
    let (what, rest) = object_ref(l, b)?;
    let rest = end(&rest);
    let f = Filter::In(Box::new(what));
    let r = match rest {
        "can't block" => Restriction::CantBlock(f),
        "can't attack" => Restriction::CantAttack(f),
        "can't attack or block" => Restriction::CantAttackOrBlock(f),
        "can't be blocked" => Restriction::CantBeBlocked(f),
        "attacks this combat if able" | "attacks if able" => Restriction::MustAttack(f),
        _ => return None,
    };
    Some(Effect::AddRestriction {
        restriction: r,
        duration: dur,
    })
}

/// "each player sacrifices a creature", "target player sacrifices an artifact",
/// "sacrifice ~", "sacrifice a creature".
fn p_sacrifice(l: &str, b: &mut Builder) -> Option<Effect> {
    if l == "sacrifice ~" {
        return Some(Effect::SacrificeObjects { what: Sel::This });
    }
    let (who, r) = if let Some(r) = l.strip_prefix("sacrifice ") {
        (PlayerRef::You, r.to_string())
    } else {
        let (p, rest) = player_ref(l, b)?;
        (p, rest.trim().strip_prefix("sacrifices ")?.to_string())
    };
    let (n, r2) = parse_number(&r)?;
    let (f, _, tail) = parse_object_phrase(r2)?;
    if !end(tail).is_empty() {
        return None;
    }
    Some(Effect::Sacrifice {
        who,
        filter: f,
        count: n,
    })
}

/// "discard a card", "target player discards two cards", "each opponent discards a card",
/// "discard two cards at random", "discard your hand".
fn p_discard(l: &str, b: &mut Builder) -> Option<Effect> {
    if l == "discard your hand" {
        return Some(Effect::DiscardHand {
            who: PlayerRef::You,
        });
    }
    let (who, r) = if let Some(r) = l.strip_prefix("discard ") {
        (PlayerRef::You, r.to_string())
    } else {
        let (p, rest) = player_ref(l, b)?;
        let rest = rest.trim().to_string();
        if rest == "discards their hand" {
            return Some(Effect::DiscardHand { who: p });
        }
        (p, rest.strip_prefix("discards ")?.to_string())
    };
    let random = r.ends_with(" at random");
    let r = r.trim_end_matches(" at random");
    let (n, t) = parse_card_count(r)?;
    if !end(t).is_empty() {
        return None;
    }
    Some(Effect::Discard {
        who,
        n,
        random,
        filter: Filter::Any,
    })
}

/// "~ fights target creature", "target creature you control fights target creature you don't control".
fn p_fight(l: &str, b: &mut Builder) -> Option<Effect> {
    let (a, rest) = object_ref(l, b)?;
    let r = rest.trim().strip_prefix("fights ")?;
    let (c, tail) = if let Some(r2) = r.strip_prefix("another target ") {
        let (mut spec, t) =
            parse_target(&format!("target {r2}")).map(|(s, t)| (s, t.to_string()))?;
        spec.distinct_from = vec![0];
        let slot = b.add_target(spec, "another target");
        (Sel::Target(slot), t)
    } else {
        object_ref(r, b)?
    };
    if !end(&tail).is_empty() {
        return None;
    }
    Some(Effect::Fight { a, b: c })
}

/// "gain control of target creature [until end of turn]".
fn p_gain_control(l: &str, b: &mut Builder) -> Option<Effect> {
    let (dur, l) = duration_suffix(l);
    let r = l.strip_prefix("gain control of ")?;
    let (what, tail) = object_ref(r, b)?;
    if !end(&tail).is_empty() {
        return None;
    }
    Some(Effect::GainControl {
        what,
        who: PlayerRef::You,
        duration: dur,
    })
}

/// Short fixed phrases.
fn p_simple_actions(l: &str, b: &mut Builder) -> Option<Effect> {
    Some(match l {
        "investigate" => Effect::KeywordAction {
            action: KeywordAction::Investigate,
            who: PlayerRef::You,
            what: Sel::None,
            n: Value::c(1),
        },
        "proliferate" => Effect::KeywordAction {
            action: KeywordAction::Proliferate,
            who: PlayerRef::You,
            what: Sel::None,
            n: Value::c(1),
        },
        "regenerate ~" => Effect::Regenerate { what: Sel::This },
        "transform ~" => Effect::Transform { what: Sel::This },
        "you become the monarch" => Effect::BecomeMonarch {
            who: PlayerRef::You,
        },
        "you take the initiative" => Effect::TakeInitiative {
            who: PlayerRef::You,
        },
        "take an extra turn after this one" => Effect::ExtraTurn {
            who: PlayerRef::You,
        },
        "prevent all combat damage that would be dealt this turn" => Effect::AddReplacement {
            def: ReplacementDef {
                event: ReplacementEvent::Damage {
                    source: Filter::Any,
                    to_players: Some(PlayerFilter::Any),
                    to_objects: Some(Filter::Any),
                    combat_only: true,
                },
                action: ReplacementAction::Prevent,
                self_replacement: false,
                optional: false,
            },
            duration: Duration::EndOfTurn,
            uses: None,
        },
        _ => {
            if let Some(r) = l.strip_prefix("regenerate ") {
                let (what, tail) = object_ref(r, b)?;
                if !end(&tail).is_empty() {
                    return None;
                }
                return Some(Effect::Regenerate { what });
            }
            return None;
        }
    })
}

fn p_shuffle(l: &str, _b: &mut Builder) -> Option<Effect> {
    (l == "shuffle" || l == "shuffle your library").then_some(Effect::Shuffle {
        who: PlayerRef::You,
    })
}

/// Whether an effect produces mana (for mana ability detection, CR 605.1a).
pub fn is_mana_effect(e: &Effect) -> bool {
    match e {
        Effect::AddMana { .. } => true,
        Effect::Seq(v) => {
            v.iter().any(is_mana_effect) && v.iter().all(|x| !matches!(x, Effect::Draw { .. }))
        }
        Effect::ChooseOne { options, .. } => options.iter().all(|(_, e)| is_mana_effect(e)),
        // "Add {G}. If you control four or more creatures, add {G}{G} instead."
        Effect::If {
            then, otherwise, ..
        } => is_mana_effect(then) && is_mana_effect(otherwise),
        _ => false,
    }
}

#[allow(dead_code)]
fn _kw(_: KeywordKind) {}
