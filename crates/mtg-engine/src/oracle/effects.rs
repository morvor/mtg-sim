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
    /// The creature target named by a "Choose target [creature]." sentence (slot and
    /// target text): "that creature" keeps referring to it after "it" has come to mean
    /// something else ("Choose target attacking or blocking creature. Scry 3, then reveal
    /// the top card of your library. ~ deals damage equal to that card's mana value to
    /// that creature.").
    pub chosen_creature: Option<(u8, String)>,
    /// The group ("all creatures you control") an earlier instruction affected, which
    /// "they" and "those creatures" refer to (see `patterns::pronoun_groups`).
    pub group: Option<super::patterns::pronoun_groups::GroupRef>,
    /// Phrases that name objects the text chose earlier, and what they select ("Choose
    /// target creature you control and target creature you don't control. Put a +1/+1
    /// counter on the creature you control."; see `patterns::choose_two_targets`).
    pub named: Vec<(String, Sel)>,
    pub ctx: &'c CompileContext<'c>,
}

impl<'c> Builder<'c> {
    pub fn new(ctx: &'c CompileContext<'c>) -> Self {
        // An instant's or sorcery's pronouns never mean the spell itself, and "that player"
        // never means the controller: until the text gives them an antecedent they have
        // none (see `patterns::oracle_hardening_referents`). A trigger's body gets them
        // from the trigger condition.
        use super::patterns::oracle_hardening_referents as r;
        Builder {
            targets: vec![],
            it: if ctx.is_spell() {
                r::no_referent()
            } else {
                Sel::This
            },
            it_player: r::no_player_referent(),
            in_trigger: false,
            sentences: 0,
            chosen_creature: None,
            group: None,
            named: vec![],
            ctx,
        }
    }
    pub fn add_target(&mut self, mut spec: TargetSpec, text: &str) -> u8 {
        spec.text = text.to_string();
        // "target creature card with lesser mana value": the object "it" means as the
        // target is described (see `patterns::filters_relational`); left unresolved
        // (and so not understood) if "it" has no antecedent.
        if super::patterns::filters_relational::mentions_referent(&spec) {
            let it = super::patterns::pronoun_groups::singular_it(self);
            if super::patterns::filters_relational::names_one_object(&it) {
                if let Some(s) = super::patterns::filters_relational::substitute(&spec, &it) {
                    spec = s;
                }
            }
        }
        // "another target creature" / "up to one other target creature" after earlier
        // targets: different objects from those (CR 115.3 allows the same object for
        // different instances of "target" unless the text says otherwise).
        let other = text.starts_with("another target") || text.contains("other target");
        if other && spec.distinct_from.is_empty() {
            spec.distinct_from = (0..self.targets.len() as u8).collect();
        }
        // A target player doesn't become "it" ("target opponent loses life equal to its
        // power" — "its" is still the object from before).
        let is_player = matches!(spec.what, TargetKind::Player(_));
        // "Destroy target creature an opponent controls. That player loses 3 life.": the
        // opponent mentioned is that object's controller.
        let opponents = matches!(&spec.what, TargetKind::Object(f) if controlled_by_opponent(f));
        self.targets.push(spec);
        let slot = (self.targets.len() - 1) as u8;
        if !is_player {
            self.it = Sel::Target(slot);
        } else if super::patterns::oracle_hardening_referents::is_no_player_referent(
            &self.it_player,
        ) {
            // "~ deals 3 damage to target opponent. That player discards two cards."
            self.it_player = PlayerRef::Target(slot);
        }
        if opponents {
            self.it_player = PlayerRef::ControllerOf(Box::new(Sel::Target(slot)));
        }
        slot
    }
}

fn controlled_by_opponent(f: &Filter) -> bool {
    match f {
        Filter::ControlledBy(PlayerRel::Opponent) => true,
        Filter::And(v) => v.iter().any(controlled_by_opponent),
        _ => false,
    }
}

/// Parses a full body (possibly modal).
pub fn parse_body(text: &str, ctx: &CompileContext) -> Option<Body> {
    let t = text.trim();
    if let Some(modal) = parse_modal(t, ctx, None) {
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

/// Parses a body in which "it"/"that card" initially refers to `it` (e.g. the card an
/// activated ability's cost exiled, CR 400.7j).
pub fn parse_body_with_it(text: &str, ctx: &CompileContext, it: Sel) -> Option<Body> {
    let mut b = Builder::new(ctx);
    b.it = it;
    let effect = parse_effect_text(text.trim(), &mut b)?;
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
    // A trigger whose "that player" would be you has no "that player": oracle text says
    // "you" for you.
    let it_player = match it_player {
        PlayerRef::You => super::patterns::oracle_hardening_referents::no_player_referent(),
        p => p,
    };
    if let Some(modal) = parse_modal(t, ctx, Some((&it, &it_player))) {
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

/// "Choose one —\n• mode\n• mode" (CR 700.2). In a triggered ability (`trigger`: what
/// "it" and "that player" refer to), the modes' pronouns refer to what the trigger's do.
fn parse_modal(
    t: &str,
    ctx: &CompileContext,
    trigger: Option<(&Sel, &PlayerRef)>,
) -> Option<Modal> {
    let (head, rest) = t.split_once('\n')?;
    let hl = head.to_lowercase();
    let hl = hl.trim().trim_end_matches(['—', ':', '.', ' ']);
    let fixed = match hl {
        "choose one" => Some((1, 1)),
        "choose two" => Some((2, 2)),
        "choose three" => Some((3, 3)),
        "choose one or both" => Some((1, 2)),
        "choose one or more" => Some((1, 99)),
        "choose any number" => Some((0, 99)),
        "choose one or two" => Some((1, 2)),
        _ => None,
    };
    let header = match fixed {
        Some((min, max)) => super::patterns::ModalHeader {
            min: Value::Const(min),
            max: Value::Const(max),
            allow_repeat: false,
            chooser: ModeChooser::Controller,
        },
        None => super::patterns::modal_header_patterns()
            .iter()
            .find_map(|p| (p.parse)(hl, ctx))?,
    };
    let mut modes = Vec::new();
    for line in rest.lines() {
        let l = line.trim().trim_start_matches('•').trim();
        if l.is_empty() {
            continue;
        }
        let mut b = Builder::new(ctx);
        if let Some((it, it_player)) = trigger {
            b.in_trigger = true;
            b.it = it.clone();
            b.it_player = it_player.clone();
        }
        let effect = parse_effect_text(strip_flavor_word(l), &mut b)?;
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
    let n = modes.len() as i32;
    let max = match header.max {
        // Without repeats, at most every mode (CR 700.2d).
        Value::Const(m) if !header.allow_repeat => Value::Const(m.min(n)),
        other => other,
    };
    Some(Modal {
        min: header.min,
        max,
        allow_repeat: header.allow_repeat,
        modes,
        per_mode_cost: false,
        chooser: header.chooser,
    })
}

/// A mode's text without its flavor word ("Cure Wounds — You gain 2 life."): flavor words
/// have no rules meaning (CR 207.2d).
pub fn strip_flavor_word(mode: &str) -> &str {
    match mode.split_once(" — ") {
        Some((head, rest))
            if !head.is_empty()
                && head.split_whitespace().count() <= 6
                && !head.contains(['{', ':', '"', '.', ','])
                && head
                    .chars()
                    .next()
                    .is_some_and(|c| c.is_uppercase() || c == '~') =>
        {
            rest.trim()
        }
        _ => mode,
    }
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
    use super::patterns::pronoun_groups as groups;
    // No instructions at all ("Whenever you attack with two or more creatures," followed
    // by a line the compiler can't join to it) isn't an effect that does nothing.
    if t.trim().trim_end_matches('.').trim().is_empty() {
        return None;
    }
    let outer_group = b.group.take();
    let mut introduced = super::patterns::oracle_hardening_referents::Introduced::default();
    let mut effects = Vec::new();
    // An X an earlier sentence defined ("..., where X is ..."): where that sentence's
    // effect is in `effects`, and its value, if it could be read.
    let mut x_defined: Option<(usize, Option<Value>)> = None;
    let mut x_stored = false;
    for s in split_sentences(t) {
        // "~ deals 1 damage to each creature. If it was kicked, it deals 2 damage to each
        // creature instead.": a spell that is the subject of an instruction is what a
        // later "it" refers to, until something else is mentioned.
        super::patterns::oracle_hardening_referents::note_subject(&s, b);
        let defines_x = s.to_lowercase().contains(", where x is ");
        // Read before the sentence is parsed, with pronouns as the sentence reads them.
        let defined_value = if defines_x {
            super::patterns::r107_numbers::defined_x(&s, b)
        } else {
            None
        };
        // "If it was a creature card, ... If it was a land card, ... Otherwise, ...": the
        // alternative to the whole run of conditions.
        if x_defined.is_none() && s.to_lowercase().starts_with("otherwise, ") {
            super::patterns::conditional_followups::group_condition_run(&mut effects);
        }
        // Sentences that modify the previous one ("It can't be regenerated.").
        let before_followup = (b.it.clone(), b.targets.len());
        let followed_up = match effects.last_mut() {
            Some(prev) => crate::oracle_ext::apply_followup_ext(&s, prev, b),
            None => false,
        };
        // "You may reveal a card that shares a creature type with that creature from among
        // them ...": a qualifier's "it" in the sentence (see `patterns::filters_relational`).
        if followed_up {
            if let Some(prev) = effects.pop() {
                let e = super::patterns::filters_relational::resolve_in_sentence(
                    prev,
                    b,
                    before_followup,
                )?;
                effects.push(e);
            }
        }
        if followed_up {
            b.sentences += 1;
        } else {
            let before = (b.it.clone(), b.targets.len());
            let parsed = parse_sentence(&s, b).and_then(|e| {
                super::patterns::filters_relational::resolve_in_sentence(e, b, before)
            });
            let Some(mut e) = parsed else {
                super::patterns::oracle_hardening_referents::abandon_introduced(b, introduced);
                groups::abandon(b, outer_group);
                return None;
            };
            // "Untap all creatures you control. They gain haste until end of turn."
            effects.extend(groups::note(&mut e, b));
            super::patterns::oracle_hardening_referents::note_introduced(
                &mut e,
                b,
                &mut introduced,
            );
            super::patterns::oracle_hardening_referents::note_player_mention(&s, b);
            effects.push(e);
            b.sentences += 1;
        }
        // "Create an X/X ... token, where X is half your life total, rounded up. It deals X
        // damage to you.": an X the text defined in an earlier sentence is that value, not
        // the X chosen for the spell or ability. It's determined once (CR 608.2h), as the
        // defining instruction is performed, and kept for the later ones.
        if let Some((at, value)) = &x_defined {
            if !defines_x
                && effects
                    .last()
                    .is_some_and(super::patterns::r107_numbers::uses_x)
            {
                let stored = Value::Var(super::patterns::r107_numbers::DEFINED_X);
                let substituted = match (value, effects.last()) {
                    (Some(_), Some(last)) => {
                        super::patterns::r107_numbers::substitute_x(last, &stored)
                    }
                    _ => None,
                };
                let Some(e) = substituted else {
                    super::patterns::oracle_hardening_referents::abandon_introduced(b, introduced);
                    groups::abandon(b, outer_group);
                    return None;
                };
                *effects.last_mut().expect("checked above") = e;
                if !x_stored {
                    let at = (*at).min(effects.len() - 1);
                    effects.insert(
                        at,
                        Effect::StoreValue {
                            var: super::patterns::r107_numbers::DEFINED_X,
                            value: value.clone().expect("checked above"),
                        },
                    );
                    x_stored = true;
                }
            }
        }
        if defines_x {
            x_defined = Some((effects.len().saturating_sub(1), defined_value));
            x_stored = false;
        }
        // "If you do, repeat this process.": the instructions so far are the process.
        if effects
            .last()
            .is_some_and(crate::repeat_process::has_open_repeat)
        {
            let body = Effect::seq(std::mem::take(&mut effects));
            effects.push(Effect::RepeatProcess {
                body: Box::new(body),
            });
        }
    }
    let e = super::patterns::oracle_hardening_referents::finish_introduced(
        Effect::seq(effects),
        b,
        introduced,
    );
    Some(groups::finish(e, b, outer_group))
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
    // "You may pay [cost]" is an optional cost as a whole (a pattern), not "you may" + "pay";
    // so is "you may [effect] unless that player pays [cost]", where that player decides
    // whether to pay first (CR 118.12a); "you may cast [spells] this turn as though they
    // had flash" is a permission.
    if l.starts_with("you may pay ")
        || (l.starts_with("you may ") && l.contains(" unless that player pays "))
        || (l.starts_with("you may cast ") && l.ends_with(" as though they had flash"))
    {
        if let Some(e) = parse_simple(l, b) {
            return Some(e);
        }
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
    let saved = (
        b.targets.clone(),
        b.it.clone(),
        b.it_player.clone(),
        b.group.clone(),
    );
    if let Some(e) = parse_clause(l, b) {
        return Some(e);
    }
    (b.targets, b.it, b.it_player, b.group) = saved;
    // "If it isn't a creature, it becomes ...", "If that creature was a Human, ...": a
    // condition about what an earlier part of the text refers to, when the patterns
    // didn't understand the sentence as a whole (see `patterns::conditions_referents`).
    super::patterns::conditions_referents::leading_if(l, b)
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
            // "That player" may name a player target of the first half (see
            // `Builder::add_target`), which is dropped below if the split fails.
            let saved_player = b.it_player.clone();
            let saved_group = b.group.clone();
            if let Some(mut ea) = parse_simple(a, b) {
                // "untap all creatures and gain control of them": the group the first
                // half affected.
                let store = super::patterns::pronoun_groups::note(&mut ea, b);
                // "return target permanent to its owner's hand, then that player ..."
                super::patterns::oracle_hardening_referents::note_player_mention(a, b);
                // The second half may modify the first ("exile it, then return it").
                if matches!(sep, ", then " | " and then ")
                    && crate::oracle_ext::apply_followup_ext(c, &mut ea, b)
                {
                    return Some(Effect::seq(store.into_iter().chain([ea]).collect()));
                }
                // Second half may omit the subject: "draw a card and lose 1 life"; or be
                // optional: "draw a card, then you may cast a spell ...".
                let optional = matches!(sep, ", then " | " and then ") && c.starts_with("you may ");
                if let Some(ec) = parse_simple(c, b)
                    .or_else(|| parse_clause(c, b))
                    .or_else(|| optional.then(|| parse_sentence(c, b)).flatten())
                {
                    return Some(Effect::seq(store.into_iter().chain([ea, ec]).collect()));
                }
            }
            b.targets.truncate(saved_targets);
            b.it = saved_it;
            b.it_player = saved_player;
            b.group = saved_group;
        }
    }
    None
}

/// Resolves pronoun/self references to a selection.
pub fn object_ref(s: &str, b: &mut Builder) -> Option<(Sel, String)> {
    let s = s.trim();
    let pairs: [(&str, Sel); 7] = [
        ("~", Sel::This),
        ("enchanted creature", Sel::AttachedTo),
        ("equipped creature", Sel::AttachedTo),
        // Auras with "enchant permanent/land/artifact/...": the object it's attached to
        // (CR 303.4).
        ("enchanted permanent", Sel::AttachedTo),
        ("enchanted land", Sel::AttachedTo),
        ("enchanted artifact", Sel::AttachedTo),
        ("enchanted planeswalker", Sel::AttachedTo),
    ];
    for (p, sel) in pairs {
        if let Some(rest) = s.strip_prefix(p) {
            // "Gain control of enchanted permanent. Untap that permanent.": the object
            // just named is what a later pronoun refers to (unless something more
            // specific already is).
            if matches!(sel, Sel::AttachedTo)
                && (matches!(b.it, Sel::This)
                    || super::patterns::oracle_hardening_referents::is_no_referent(&b.it))
            {
                b.it = Sel::AttachedTo;
            }
            return Some((sel, rest.to_string()));
        }
    }
    // The longest phrase that names the object ("the creature an opponent controls"
    // before "the creature").
    let named = b
        .named
        .iter()
        .filter_map(|(p, sel)| {
            let rest = s.strip_prefix(p.as_str())?;
            (rest.is_empty() || rest.starts_with(' ') || rest.starts_with('\''))
                .then(|| (p.len(), sel.clone(), rest.to_string()))
        })
        .max_by_key(|(len, _, _)| *len);
    if let Some((_, sel, rest)) = named {
        if matches!(sel, Sel::Target(_)) {
            b.it = sel.clone();
        }
        return Some((sel, rest));
    }
    // "the creature with the least power" (see `patterns::filters_relational`).
    if let Some(r) = super::patterns::filters_relational::definite_extreme(s, b) {
        return Some(r);
    }
    if let Some((slot, text)) = &b.chosen_creature {
        let still_there = b
            .targets
            .get(*slot as usize)
            .is_some_and(|t| &t.text == text);
        if let Some(rest) = s.strip_prefix("that creature") {
            if still_there && (rest.is_empty() || rest.starts_with(' ') || rest.starts_with('\'')) {
                return Some((Sel::Target(*slot), rest.to_string()));
            }
        }
    }
    for p in [
        "it",
        "that creature",
        "that permanent",
        "that card",
        "that spell",
        "the creature",
        "that token",
        "this token",
        // "Whenever ~ crews a Vehicle, that Vehicle ..." (CR 702.122b); "whenever ~
        // saddles a Mount or crews a Vehicle, that Mount or Vehicle ..." (CR 702.171c).
        "that vehicle",
        "that mount or vehicle",
        "that mount",
        "that artifact",
        "that land",
        // "Choose target creature ... Exile the chosen creature." (Turncoat Kunoichi).
        "the chosen creature",
        "the chosen card",
        // A named character's personal pronouns mean what "it" would ("Put a +1/+1
        // counter on ~. He gains vigilance ...", "put a +1/+1 counter on him").
        "he",
        "she",
        "him",
        "her",
    ] {
        if let Some(rest) = s.strip_prefix(p) {
            let personal = matches!(p, "he" | "she" | "him" | "her");
            // "her" is also possessive ("her power"): only an object at the end of the
            // phrase or before a conjunction or preposition.
            if p == "her"
                && !(rest.is_empty()
                    || [" and ", " until ", " to ", " from ", " on "]
                        .iter()
                        .any(|x| rest.starts_with(x)))
            {
                continue;
            }
            if rest.is_empty() || rest.starts_with(' ') || (rest.starts_with('\'') && !personal) {
                // "This token" is always the source (normally normalized to "~").
                if p == "this token" {
                    return Some((Sel::This, rest.to_string()));
                }
                // Not a group an earlier instruction affected: that's "they".
                let it = super::patterns::pronoun_groups::singular_it(b);
                // No antecedent (a spell's first mention), or "that creature" meaning the
                // source, which oracle text calls "~" (except "that card" for what the
                // source became in its own trigger: "When ~ dies, return that card ..."):
                // not understood.
                let own_card = p == "that card" && b.in_trigger && b.sentences == 0;
                if super::patterns::oracle_hardening_referents::is_no_referent(&it)
                    || (p != "it" && !personal && !own_card && matches!(it, Sel::This))
                {
                    return None;
                }
                return Some((it, rest.to_string()));
            }
        }
    }
    // Plural pronouns: the targets, group, or cards an earlier instruction was about
    // ("Untap all creatures you control. They gain hexproof until end of turn.").
    if let Some(r) = super::patterns::pronoun_groups::plural_object_ref(s, b) {
        return r;
    }
    if let Some((mut spec, rest)) = parse_any_target(s) {
        // "target creature blocking it"
        let mut rest = rest.to_string();
        if let TargetKind::Object(f) = &spec.what {
            if let Some((f, r)) = super::patterns::pronoun_groups::blocking_it(f.clone(), &rest, b)
            {
                spec.what = TargetKind::Object(f);
                rest = r;
            }
        }
        let text = s[..s.len() - rest.len()].trim().to_string();
        let slot = b.add_target(spec, &text);
        return Some((Sel::Target(slot), rest));
    }
    if let Some(r) = s.strip_prefix("each ").or_else(|| s.strip_prefix("all ")) {
        let (f, _, rest) = parse_object_phrase(r)?;
        let f = super::patterns::filters_relational::resolve_referent(f, b)?;
        // "each creature blocking it"
        let (f, rest) = match super::patterns::pronoun_groups::blocking_it(f.clone(), rest, b) {
            Some((f, r)) => (f, r),
            None => (f, rest.to_string()),
        };
        let (f, rest) = bind_target_player(f, &rest, b);
        return Some((Sel::All(f), rest));
    }
    // Bare plural noun phrases ("creatures you control") mean all such objects.
    if let Some((f, plural, rest)) = parse_object_phrase(s) {
        if plural {
            let f = super::patterns::filters_relational::resolve_referent(f, b)?;
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
    use super::patterns::oracle_hardening_referents::{is_no_player_referent, is_no_referent};
    let s = s.trim();
    if let Some(r) = s.strip_prefix("that player") {
        if is_no_player_referent(&b.it_player) {
            return None;
        }
        return Some((b.it_player.clone(), r.to_string()));
    }
    if let Some(r) = s
        .strip_prefix("its controller")
        .or_else(|| s.strip_prefix("their controller"))
    {
        if is_no_referent(&b.it) {
            return None;
        }
        return Some((
            PlayerRef::ControllerOf(Box::new(b.it.clone())),
            r.to_string(),
        ));
    }
    if let Some(r) = s
        .strip_prefix("its owner")
        .or_else(|| s.strip_prefix("their owner"))
    {
        if is_no_referent(&b.it) {
            return None;
        }
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
        // CR 901.11.
        (
            " until a player planeswalks away from a plane",
            Duration::UntilPlaneswalk {
                away_from_plane: true,
            },
        ),
        (
            " until a player planeswalks",
            Duration::UntilPlaneswalk {
                away_from_plane: false,
            },
        ),
        // CR 500.4: until that step next begins.
        (
            " until your next upkeep",
            Duration::UntilYourNextStep(TriggerStep::Upkeep),
        ),
        (
            " until your next end step",
            Duration::UntilYourNextStep(TriggerStep::End),
        ),
        (" until end of combat", Duration::EndOfCombat),
        (
            " for as long as ~ remains on the battlefield",
            Duration::WhileSourceOnBattlefield,
        ),
        (
            " for as long as you control ~",
            Duration::WhileYouControlSource,
        ),
        // CR 611.2b.
        (
            " for as long as ~ remains tapped",
            crate::untap_choice::remains_tapped(false),
        ),
        (
            " for as long as you control ~ and ~ remains tapped",
            crate::untap_choice::remains_tapped(true),
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
    // "it" in an object qualifier ("search for a creature card with lesser mana value")
    // means what "it" meant as the clause began (see `patterns::filters_relational`).
    let it = super::patterns::pronoun_groups::singular_it(b);
    let e = parse_simple_clause(l, b)?;
    let e = super::patterns::filters_relational::resolve_clause(e, &it);
    super::patterns::filters_relational::note_sacrificed(&e, b);
    Some(e)
}

fn parse_simple_clause(l: &str, b: &mut Builder) -> Option<Effect> {
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
    } else if super::patterns::pronoun_groups::plural_pronoun(l).is_some() {
        // "They each deal damage equal to their power to ...": several sources, each
        // dealing its own damage, which a single damage event can't express.
        return None;
    } else if let Some((sel, r)) = object_ref(l, b) {
        (sel, r.trim_start().strip_prefix("deals ")?.to_string())
    } else {
        return None;
    };
    let rest = rest.as_str();
    let owned_tail: String;
    let (amount, rest) = if let Some(r) = rest.strip_prefix("damage equal to ") {
        // "Each creature you control deals damage equal to its power": "its" is each of
        // the sources in turn (the executor binds `vars::AFFECTED` to each).
        let multi = matches!(src, Sel::All(_) | Sel::Union(_));
        let saved_it = multi.then(|| std::mem::replace(&mut b.it, Sel::Var(vars::AFFECTED)));
        let parsed = super::statics::parse_value_phrase(r, b);
        if let Some(it) = saved_it {
            b.it = it;
        }
        let (v, r2) = parsed?;
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
    if let Sel::Target(n) = to {
        let spec = &mut b.targets[n as usize];
        if spec.text == "any other target" {
            super::patterns::damage_removal::other_than_damage_source(spec, &src);
        }
    }
    let to = other_than_subject(to, &src);
    Some(Effect::DealDamage {
        source: src,
        amount,
        to,
    })
}

/// In an instant or sorcery with a single object target, "other creatures" (Intimidation
/// Bolt: "~ deals 3 damage to target creature. Other creatures can't attack this turn")
/// means other than that target: the spell itself is never among the objects described.
pub(crate) fn other_than_sole_target(mut body: Body) -> Body {
    if body.modal.is_some()
        || body.targets.len() != 1
        || !matches!(body.targets[0].what, TargetKind::Object(_))
        || !matches!(body.targets[0].max, Value::Const(1))
    {
        return body;
    }
    // `Filter::Other` inside a filter conjunction ({"And": [..., "Other", ...]}).
    fn fix(v: serde_json::Value, in_and: bool) -> serde_json::Value {
        use serde_json::Value as J;
        match v {
            J::String(s) if in_and && s == "Other" => {
                serde_json::json!({"Not": {"In": {"Target": 0}}})
            }
            J::Object(m) => J::Object(
                m.into_iter()
                    .map(|(k, v)| {
                        let and = k == "And";
                        (k, fix(v, and))
                    })
                    .collect(),
            ),
            J::Array(a) => J::Array(a.into_iter().map(|x| fix(x, in_and)).collect()),
            other => other,
        }
    }
    let Ok(json) = serde_json::to_value(&body.effect) else {
        return body;
    };
    if let Ok(e) = serde_json::from_value(fix(json, false)) {
        body.effect = e;
    }
    body
}

/// "That creature deals damage ... to each other creature": "other" means other than the
/// subject of the sentence, which isn't necessarily the ability's source.
fn other_than_subject(to: Sel, subject: &Sel) -> Sel {
    // "Each creature deals damage to each other creature" means other than each one in
    // turn, which a single filter can't say; leave such sentences as they were.
    if matches!(
        subject,
        Sel::This | Sel::All(_) | Sel::Union(_) | Sel::Players(_) | Sel::None
    ) {
        return to;
    }
    let fix = |f: Filter| match f {
        Filter::Other => Filter::not(Filter::In(Box::new(subject.clone()))),
        Filter::And(v) => Filter::And(
            v.into_iter()
                .map(|x| match x {
                    Filter::Other => Filter::not(Filter::In(Box::new(subject.clone()))),
                    x => x,
                })
                .collect(),
        ),
        f => f,
    };
    match to {
        Sel::All(f) => Sel::All(fix(f)),
        Sel::Union(v) => Sel::Union(
            v.into_iter()
                .map(|s| match s {
                    Sel::All(f) => Sel::All(fix(f)),
                    s => s,
                })
                .collect(),
        ),
        to => to,
    }
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
    // "Whenever an opponent draws a card, ~ deals 1 damage to them.": a player.
    if let Some(who) = super::patterns::pronoun_groups::them_player(s, b) {
        return Some((Sel::Players(who), s["them".len()..].to_string()));
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
/// graveyard to your hand", "... to the battlefield [tapped [and attacking]]".
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
        // "two target cards from an opponent's graveyard to their hand": a card goes to
        // its owner's hand (CR 400.3).
        || t == "to their hand"
    {
        Destination::zone(ZoneKind::Hand)
    } else if t == "to the battlefield" || t == "to the battlefield under your control" {
        Destination::battlefield().under_your_control()
    } else if t == "to the battlefield tapped"
        || t == "to the battlefield tapped under your control"
    {
        Destination::battlefield().under_your_control().tapped()
    } else if t == "to the battlefield tapped and attacking" {
        // CR 508.4: it's attacking without having been declared as an attacker.
        let mut d = Destination::battlefield().under_your_control().tapped();
        d.attacking = true;
        d
    } else if t == "to the battlefield under its owner's control" {
        let mut d = Destination::battlefield();
        d.controller = Some(PlayerRef::OwnerOf(Box::new(what.clone())));
        d
    } else if t == "to the top of its owner's library" || t == "on top of its owner's library" {
        Destination::library_top()
    } else {
        return None;
    };
    if to.zone == ZoneKind::Battlefield {
        // "It gains haste": the permanent the card became (CR 400.7), which the move
        // records.
        b.it = Sel::Var(vars::IT);
    }
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
    } else if r.contains(" or ") && r.contains("}{") {
        // "{W}{W}, {W}{B}, or {B}{B}" (CR 106.1): one of several combinations of mana.
        let options: Option<Vec<(String, Effect)>> = r
            .split([',', ' '])
            .filter(|w| w.starts_with('{'))
            .map(|w| {
                let w = w.trim_end_matches('.');
                match p_add_mana(&format!("add {w}"), _b)? {
                    e @ Effect::AddMana {
                        mana: ManaProduction::Fixed(_),
                        ..
                    } => Some((format!("Add {}", w.to_uppercase()), e)),
                    _ => None,
                }
            })
            .collect();
        let options = options.filter(|o| o.len() >= 2)?;
        return Some(Effect::ChooseOne {
            who: PlayerRef::You,
            options,
        });
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
        pt_values: None,
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
    // "Put a +1/+1 counter on each creature you control. Those creatures gain vigilance
    // until end of turn.": "those creatures" are the group that got the counters (see
    // `patterns::pronoun_groups`).
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
    // "search your library for any card" (Demonic Counsel) is "a card".
    let any = r.strip_prefix("any card").map(|x| format!("a card{x}"));
    let r = any.as_deref().unwrap_or(r);
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
    if let Some(r) = l
        .strip_prefix("mill ")
        .or_else(|| l.strip_prefix("you mill "))
    {
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
    let (mut dur, mut l) = duration_suffix(l);
    // "That creature can't block this combat" (Forgestoker Dragon): until the combat
    // phase ends.
    if matches!(dur, Duration::Permanent) {
        if let Some(r) = l.trim().strip_suffix(" this combat") {
            (dur, l) = (Duration::EndOfCombat, r);
        }
    }
    // "Until your next turn, creatures can't attack you" (Chronomantic Escape).
    if matches!(dur, Duration::Permanent) {
        if let Some(r) = l.trim().strip_prefix("until your next turn, ") {
            (dur, l) = (Duration::UntilYourNextTurn, r);
        }
    }
    if matches!(dur, Duration::Permanent) {
        return None;
    }
    let (what, rest) = object_ref(l, b)?;
    let rest = end(&rest);
    // CR 611.2c: these effects modify the rules, not characteristics, so one naming a
    // class of objects ("creatures can't be blocked this turn") also applies to objects
    // that join the class later (Veiling Oddity ruling). Specific objects (targets, "those
    // creatures") are locked in as the effect begins.
    // "Other creatures can't attack this turn" is a class too: other than the source, or
    // (in an instant or sorcery) other than its target, see `other_than_sole_target`.
    let f = match &what {
        Sel::All(f) if is_class_filter(&without_other(f)) => f.clone(),
        _ => Filter::In(Box::new(what)),
    };
    let r = match rest {
        "can't block" => Restriction::CantBlock(f),
        "can't attack" => Restriction::CantAttack(f),
        "can't attack or block" => Restriction::CantAttackOrBlock(f),
        "can't attack you" | "can't attack you or planeswalkers you control" => {
            Restriction::CantAttackPlayer {
                attackers: f,
                defender: PlayerFilter::You,
                planeswalkers: rest.ends_with("planeswalkers you control"),
                battles: false,
            }
        }
        "can't be blocked" => Restriction::CantBeBlocked(f),
        "attacks this combat if able" | "attacks if able" => Restriction::MustAttack(f),
        _ => return None,
    };
    Some(Effect::AddRestriction {
        restriction: r,
        duration: dur,
    })
}

/// The filter with its top-level "other" parts removed.
fn without_other(f: &Filter) -> Filter {
    match f {
        Filter::Other => Filter::Any,
        Filter::And(v) => Filter::And(v.iter().map(without_other).collect()),
        f => f.clone(),
    }
}

/// Whether a filter describes a class of objects by their current qualities only, without
/// referring to the resolving ability's targets, choices, source, or referenced objects,
/// so it can be evaluated again later in the effect's duration.
pub(crate) fn is_class_filter(f: &Filter) -> bool {
    match f {
        Filter::And(v) | Filter::Or(v) => v.iter().all(is_class_filter),
        Filter::Not(x) => is_class_filter(x),
        Filter::Power(_, v) | Filter::Toughness(_, v) | Filter::ManaValue(_, v) => {
            matches!(**v, Value::Const(_))
        }
        // Relative to the effect's controller, which doesn't change.
        Filter::ControlledBy(r) | Filter::OwnedBy(r) => matches!(
            r,
            PlayerRel::You | PlayerRel::Opponent | PlayerRel::Any | PlayerRel::NotYou
        ),
        Filter::Any
        | Filter::Type(_)
        | Filter::Supertype(_)
        | Filter::Subtype(_)
        | Filter::Color(_)
        | Filter::ExactColors(_)
        | Filter::Colorless
        | Filter::Multicolored
        | Filter::Monocolored
        | Filter::Permanent
        | Filter::Token
        | Filter::Tapped
        | Filter::Untapped
        | Filter::HasKeyword(_) => true,
        _ => false,
    }
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
        Effect::PersistentMana(inner) => is_mana_effect(inner),
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
