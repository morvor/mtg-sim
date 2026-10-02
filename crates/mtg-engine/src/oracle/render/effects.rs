//! Effects ([`Effect`]) → imperative sentences ("Destroy target creature.", "You gain 3
//! life.", "Target player draws two cards.").

use super::nouns::Det;
use super::players::Case;
use super::*;

fn same_player(a: &PlayerRef, b: &PlayerRef) -> bool {
    format!("{a:?}") == format!("{b:?}")
}

fn same_sel(a: &Sel, b: &Sel) -> bool {
    format!("{a:?}") == format!("{b:?}")
}

fn same_duration(a: &Duration, b: &Duration) -> bool {
    format!("{a:?}") == format!("{b:?}")
}

/// Signed P/T modifier: "+2", "-1", "+X".
pub(crate) fn signed(s: &str) -> String {
    if s.starts_with('-') || s.starts_with('+') {
        s.to_string()
    } else {
        format!("+{s}")
    }
}

impl Renderer<'_> {
    /// An effect as one or more sentences (lowercase start; the comparison ignores case).
    pub(crate) fn effect_sentences(&mut self, e: &Effect) -> String {
        let s = self.effect(e);
        let s = s.trim().to_string();
        if s.is_empty() {
            return s;
        }
        let s = capitalize(&s);
        if s.ends_with('.') || s.ends_with('"') || s.ends_with('—') {
            s
        } else {
            format!("{s}.")
        }
    }

    /// A player-performed action: (who, verb phrase in base form, whether "you" stays as
    /// the subject when it's the controller, e.g. "you gain 3 life").
    fn actor_vp(&mut self, e: &Effect) -> Option<(PlayerRef, String, bool)> {
        Some(match e {
            Effect::Draw { who, n } => {
                let (c, w) = self.counted(n, "card");
                (
                    who.clone(),
                    format!("draw {c}{}", w.unwrap_or_default()),
                    false,
                )
            }
            Effect::Discard {
                who,
                n,
                random,
                filter,
            } => {
                // "That player discards that card": the cards chosen earlier.
                if let (Filter::In(sel), Value::CountSel(sel2)) = (filter, n) {
                    if format!("{sel:?}") == format!("{sel2:?}") {
                        let s = self.sel(sel, Case::Obj);
                        return Some((who.clone(), format!("discard {s}"), false));
                    }
                }
                let noun = self.card_noun(filter);
                let (c, w) = self.counted(n, &noun);
                let r = if *random { " at random" } else { "" };
                (
                    who.clone(),
                    format!("discard {c}{r}{}", w.unwrap_or_default()),
                    false,
                )
            }
            Effect::DiscardHand { who } => {
                let p = self.possessive_for(who);
                (who.clone(), format!("discard {p} hand"), false)
            }
            Effect::Mill { who, n } => {
                let (c, w) = self.counted(n, "card");
                (
                    who.clone(),
                    format!("mill {c}{}", w.unwrap_or_default()),
                    false,
                )
            }
            Effect::GainLife { who, n } => {
                let (a, w) = self.amount(n);
                (
                    who.clone(),
                    format!("gain {a} life{}", w.unwrap_or_default()),
                    true,
                )
            }
            Effect::LoseLife { who, n } => {
                let (a, w) = self.amount(n);
                (
                    who.clone(),
                    format!("lose {a} life{}", w.unwrap_or_default()),
                    true,
                )
            }
            Effect::Scry { who, n } => {
                let (a, w) = self.amount(n);
                (
                    who.clone(),
                    format!("scry {a}{}", w.unwrap_or_default()),
                    false,
                )
            }
            Effect::Surveil { who, n } => {
                let (a, w) = self.amount(n);
                (
                    who.clone(),
                    format!("surveil {a}{}", w.unwrap_or_default()),
                    false,
                )
            }
            // "Shuffle." / "Shuffle your library." / "Target player shuffles their
            // library." (a player shuffles their own library, CR 701.24a).
            Effect::Shuffle { who } => (
                who.clone(),
                "shuffle {alt:|your library|their library}".into(),
                false,
            ),
            Effect::RevealHand { who } => {
                let p = self.possessive_for(who);
                self.revealed_hand = true;
                (who.clone(), format!("reveal {p} hand"), false)
            }
            Effect::Sacrifice { who, filter, count } => {
                let det = self.det_for(count);
                let n = self.noun_det(&strip_controller(filter), det);
                (who.clone(), format!("sacrifice {n}"), false)
            }
            Effect::AddPlayerCounters { who, kind, n } => {
                let symbol = match kind.as_str() {
                    "energy" => Some("{E}"),
                    "ticket" => Some("{TK}"),
                    _ => None,
                };
                let s = if let Some(sym) = symbol {
                    match n {
                        Value::Const(k) if *k > 0 => sym.repeat(*k as usize),
                        other => {
                            let v = self.value(other);
                            format!("{v} {sym}")
                        }
                    }
                } else {
                    let (c, w) = self.counted(n, &counter_name(kind));
                    format!("{c}{}", w.unwrap_or_default())
                };
                (who.clone(), format!("get {s}"), true)
            }
            Effect::ExtraTurn { who } => (
                who.clone(),
                "take an extra turn after this one".into(),
                true,
            ),
            Effect::WinGame { who } => (who.clone(), "win the game".into(), true),
            Effect::LoseGame { who } => (who.clone(), "lose the game".into(), true),
            Effect::BecomeMonarch { who } => (who.clone(), "become the monarch".into(), true),
            Effect::TakeInitiative { who } => (who.clone(), "take the initiative".into(), true),
            Effect::SetLife { .. } => return None,
            Effect::Search { .. } => {
                let who = match e {
                    Effect::Search { who, .. } => who.clone(),
                    _ => unreachable_player(),
                };
                let s = self.search(e);
                (who, s, false)
            }
            Effect::CreateToken {
                controller,
                spec,
                count,
                tapped,
                attacking,
            } => {
                let s = self.create_token(spec, count, *tapped, *attacking);
                (controller.clone(), s, false)
            }
            Effect::CreateTokenWithPT {
                spec,
                power,
                toughness,
                count,
                controller,
                tapped,
                attacking,
            } => {
                let mut spec = spec.clone();
                spec.pt_values = Some(Box::new((power.clone(), toughness.clone())));
                let s = self.create_token(&spec, count, *tapped, *attacking);
                (controller.clone(), s, false)
            }
            Effect::CreateEmblem { who, abilities } => {
                let s = self.emblem(abilities);
                (who.clone(), s, true)
            }
            _ => return None,
        })
    }

    /// "your"/"their" for a player's own zone.
    fn possessive_for(&mut self, who: &PlayerRef) -> String {
        match who {
            PlayerRef::You => "your".into(),
            _ => "their".into(),
        }
    }

    /// "card", "creature card", "nonland card".
    fn card_noun(&mut self, f: &Filter) -> String {
        if matches!(f, Filter::Any) {
            return "card".into();
        }
        let n = self.noun(&f.clone().in_zone(ZoneKind::Hand), Num::One);
        // The zone is implied.
        n.trim_end_matches(" in a hand").to_string()
    }

    /// "[subject] [vp]": "you" is dropped for imperative verbs.
    fn with_subject(&mut self, who: &PlayerRef, vp: &str, keep_you: bool) -> String {
        if matches!(who, PlayerRef::You) {
            if keep_you {
                return format!("you {vp}");
            }
            return vp.to_string();
        }
        let s = self.player(who, Case::Subj);
        format!("{s} {}", third_person(vp))
    }

    /// An effect as text (sentences separated by ". ").
    pub(crate) fn effect(&mut self, e: &Effect) -> String {
        self.new_clause();
        if let Some((who, vp, keep)) = self.actor_vp(e) {
            return self.with_subject(&who, &vp, keep);
        }
        match e {
            Effect::Noop => String::new(),
            Effect::Seq(v) => self.seq(v),
            Effect::If {
                cond,
                then,
                otherwise,
            } => self.if_effect(cond, then, otherwise),
            Effect::May { who, effect } => self.may(who, effect),
            Effect::PayOptional {
                who,
                cost,
                then,
                otherwise,
            } => self.pay_optional(who, cost, then, otherwise),
            // "Double the number of +1/+1 counters on each of those creatures" (CR 701.10e):
            // put on each the counters it has.
            Effect::ForEach { sel, var, effect } if matches!(effect.as_ref(), Effect::PutCountersOf { from: Sel::Var(a), to: Sel::Var(b), .. } if a == var && b == var) =>
            {
                let Effect::PutCountersOf { kind, .. } = effect.as_ref() else {
                    return String::new();
                };
                let k = match kind {
                    Some(k) => format!("{k} counters"),
                    None => "each kind of counter".into(),
                };
                let s = self.sel(sel, Case::Obj);
                format!("double the number of {k} on {s}")
            }
            // "Return the exiled card to the battlefield": each card linked to this object.
            Effect::ForEach {
                sel: Sel::Linked | Sel::CreatorLinked,
                var,
                effect,
            } => {
                self.var_defs.push((*var, Sel::CreatorLinked, false));
                self.effect(effect)
            }
            Effect::ForEach { sel, effect, .. } => {
                let s = match sel {
                    Sel::All(f) => self.for_each_noun(f),
                    other => self.sel(other, Case::Obj),
                };
                let inner = self.effect(effect);
                format!("for each {s}, {inner}")
            }
            Effect::ForEachPlayer { who, effect } => {
                let inner = self.effect(effect);
                let w = self.player(who, Case::Subj);
                // "Each opponent loses 2 life": each player does it in turn.
                let rest = inner.strip_prefix("that player ").or_else(|| {
                    inner
                        .strip_prefix("{alt:that player|")
                        .and_then(|r| r.split_once("} "))
                        .map(|(_, r)| r)
                });
                if let Some(rest) = rest {
                    format!("{w} {rest}")
                } else {
                    format!("for {w}, {inner}")
                }
            }
            Effect::AsPlayer { who, effect } => {
                let w = self.player(who, Case::Subj);
                // Performed as that player: "you" in the instruction is that player
                // ("target player loses 4 life").
                let inner = self.effect(effect);
                let inner = inner.strip_prefix("you ").unwrap_or(&inner);
                let inner = format!(" {inner} ").replace(" your ", " their ");
                format!("{w} {}", third_person(inner.trim()))
            }
            // "... If [condition], repeat this process." (CR 608.2c)
            Effect::RepeatProcess { body } => self.effect(body),
            Effect::RepeatThisProcess => "repeat this process".into(),
            Effect::Repeat { times, effect } => {
                let inner = self.effect(effect);
                let t = self.times(times);
                format!("{inner} {t}")
            }
            // "Put your choice of a flying counter or a first strike counter on it."
            Effect::ChooseOne {
                who: PlayerRef::You,
                options,
            } if options.len() > 1
                && options.iter().all(|(_, e)| {
                    matches!(
                        e,
                        Effect::AddCounters {
                            n: Value::Const(1),
                            ..
                        }
                    )
                })
                && options.windows(2).all(|w| match (&w[0].1, &w[1].1) {
                    (Effect::AddCounters { what: a, .. }, Effect::AddCounters { what: b, .. }) => {
                        same_sel(a, b)
                    }
                    _ => false,
                }) =>
            {
                let kinds: Vec<String> = options
                    .iter()
                    .filter_map(|(_, e)| match e {
                        Effect::AddCounters { kind, .. } => Some(with_article(&counter_name(kind))),
                        _ => None,
                    })
                    .collect();
                let what = match &options[0].1 {
                    Effect::AddCounters { what, .. } => what.clone(),
                    _ => Sel::None,
                };
                let t = self.sel(&what, Case::Obj);
                format!("put your choice of {} on {t}", join_list(&kinds, "or"))
            }
            // "Target creature's owner puts it on their choice of the top or bottom of their
            // library."
            Effect::ChooseOne { who, options }
                if matches!(options.as_slice(),
                    [(_, Effect::Move { what: a, to: ta }), (_, Effect::Move { what: b, to: tb })]
                    if same_sel(a, b)
                        && ta.zone == ZoneKind::Library && tb.zone == ZoneKind::Library
                        && ta.position == LibraryPosition::Top
                        && tb.position == LibraryPosition::Bottom) =>
            {
                let Effect::Move { what, .. } = &options[0].1 else {
                    return self.gap("top or bottom");
                };
                let w = self.sel(what, Case::Obj);
                let you = matches!(who, PlayerRef::You);
                let choice = if you { "your" } else { "their" };
                let vp = format!(
                    "put {w} on {choice} choice of the top or bottom of {{alt:their|its owner's|your}} library"
                );
                return self.with_subject(who, &vp, false);
            }
            // "Tap or untap target creature."
            Effect::ChooseOne {
                who: PlayerRef::You,
                options,
            } if matches!(options.as_slice(),
                [(_, Effect::Tap { what: a }), (_, Effect::Untap { what: b })] if same_sel(a, b)) =>
            {
                let Effect::Tap { what } = &options[0].1 else {
                    return self.gap("tap or untap");
                };
                let w = self.sel(what, Case::Obj);
                format!("tap or untap {w}")
            }
            Effect::ChooseOne { who, options } => {
                let w = self.player(who, Case::Subj);
                let head = if w == "you" {
                    "choose one —".to_string()
                } else {
                    format!("{w} chooses one —")
                };
                let mut s = head;
                for (_, eff) in options {
                    let t = self.effect_sentences(eff);
                    s.push_str(&format!("\n• {t}"));
                }
                s
            }
            Effect::Store { sel, var } if matches!(sel, Sel::All(_)) => {
                self.var_defs.push((*var, sel.clone(), false));
                String::new()
            }
            Effect::Store { sel, .. } => match sel {
                Sel::Choose { chooser, .. } => {
                    let c = self.player(chooser, Case::Subj);
                    let mut s = self.sel(&strip_chooser(sel), Case::Obj);
                    // "Target opponent reveals their hand. You choose a nonland card from
                    // it."
                    if self.revealed_hand {
                        if let Some(i) = s.find(" in ") {
                            if s.ends_with(" hand") {
                                s = format!("{} from it", &s[..i]);
                            }
                        }
                    }
                    if c == "you" {
                        format!("choose {s}")
                    } else {
                        format!("{c} chooses {s}")
                    }
                }
                // Remembering something already named (a target, the object itself, an
                // earlier remembered group): bookkeeping that later mentions refer to as
                // "it"/"them".
                Sel::None
                | Sel::This
                | Sel::Target(_)
                | Sel::AllTargets
                | Sel::Var(_)
                | Sel::TriggerObject
                | Sel::TriggerLki
                | Sel::TriggerOtherObject
                | Sel::TriggerObjects
                | Sel::TriggerSpell
                | Sel::AttachedTo
                | Sel::AttachedToThis
                | Sel::Linked
                | Sel::CreatorLinked => String::new(),
                other => self.gap(format!("remembering {other:?}")),
            },
            Effect::StoreValue { var, value } => {
                // Another name for a number already remembered.
                let value = match value {
                    Value::Var(n) => self
                        .stored_values
                        .iter()
                        .find(|(x, _, _)| x == n)
                        .map_or_else(|| value.clone(), |(_, v, _)| v.clone()),
                    other => other.clone(),
                };
                self.stored_values.push((*var, value, false));
                String::new()
            }
            Effect::Note { value } => {
                let v = self.value(value);
                format!("note {v}")
            }
            Effect::SetX { .. } => String::new(),
            Effect::Destroy { what, no_regen } => {
                let w = self.sel(what, Case::Obj);
                if *no_regen {
                    let pron = if is_plural_sel(what) { "They" } else { "It" };
                    format!("destroy {w}. {pron} can't be regenerated")
                } else {
                    format!("destroy {w}")
                }
            }
            Effect::Exile {
                what, face_down, ..
            } => {
                let w = self.sel(what, Case::Obj);
                if *face_down {
                    format!("exile {w} face down")
                } else {
                    format!("exile {w}")
                }
            }
            Effect::Sacrifice { .. } => unreachable_text(),
            Effect::SacrificeObjects { what } => {
                let w = self.sel(what, Case::Obj);
                format!("sacrifice {w}")
            }
            Effect::Move { what, to } => self.move_effect(what, to),
            Effect::Tap { what } => {
                let w = self.sel(what, Case::Obj);
                format!("tap {w}")
            }
            Effect::Untap { what } => {
                let w = self.sel(what, Case::Obj);
                format!("untap {w}")
            }
            Effect::DealDamage { source, amount, to } => {
                let s = self.sel(source, Case::Subj);
                let (a, w) = self.amount(amount);
                let mut t = self.sel(to, Case::Obj);
                // "deals 3 damage to each of up to two target creatures".
                if let Sel::Target(i) = to {
                    let many = self
                        .targets
                        .get(*i as usize)
                        .is_some_and(|t| t.max.as_const() != Some(1) && t.divide.is_none());
                    if many && t.contains("target") {
                        t = format!("each of {t}");
                    }
                }
                format!("{s} deals {a} damage to {t}{}", w.unwrap_or_default())
            }
            Effect::DealDamageExcess {
                source,
                amount,
                to,
                excess_to,
            } => {
                let s = self.sel(source, Case::Subj);
                let (a, w) = self.amount(amount);
                let t = self.sel(to, Case::Obj);
                let x = self.sel(excess_to, Case::Obj);
                format!(
                    "{s} deals {a} damage to {t}{}. Excess damage is dealt to {x} instead",
                    w.unwrap_or_default()
                )
            }
            Effect::DealDividedDamage { source, slot } => {
                let s = self.sel(source, Case::Subj);
                let amount = self
                    .targets
                    .get(*slot as usize)
                    .and_then(|t| t.divide.clone());
                let a = match amount {
                    Some(v) => self.value(&v),
                    None => self.gap("divided damage without amount"),
                };
                let t = self.target_mention(*slot, Case::Obj);
                format!("{s} deals {a} damage divided as you choose among {t}")
            }
            Effect::Fight { a, b } => {
                let a = self.sel(a, Case::Subj);
                let b = self.sel(b, Case::Obj);
                format!("{a} fights {b}")
            }
            Effect::AddCounters { what, kind, n } => {
                let (c, w) = self.counted(n, &counter_name(kind));
                let mut t = self.sel(what, Case::Obj);
                // "put a +1/+1 counter on each of up to two target creatures".
                if let Sel::Target(i) = what {
                    let many = self
                        .targets
                        .get(*i as usize)
                        .is_some_and(|t| t.max.as_const() != Some(1));
                    if many && (t.contains("target") || t == "them") {
                        t = format!("each of {t}");
                    }
                }
                format!("put {c} on {t}{}", w.unwrap_or_default())
            }
            Effect::RemoveCounters { what, kind, n } => {
                let noun = match kind {
                    Some(k) => counter_name(k),
                    None => "counter".into(),
                };
                let t = self.sel(what, Case::Obj);
                match n {
                    Value::Const(i) if *i >= 1000 => {
                        format!("remove all {} from {t}", plural(&noun))
                    }
                    _ => {
                        let (c, w) = self.counted(n, &noun);
                        format!("remove {c} from {t}{}", w.unwrap_or_default())
                    }
                }
            }
            Effect::MoveCounters { from, to, kind, n } => {
                let noun = match kind {
                    Some(k) => counter_name(k),
                    None => "counter".into(),
                };
                let c = match n {
                    None => format!("all {} from", plural(&noun)),
                    Some(v) => {
                        let (c, _) = self.counted(v, &noun);
                        format!("{c} from")
                    }
                };
                let f = self.sel(from, Case::Obj);
                let t = self.sel(to, Case::Obj);
                format!("move {c} {f} onto {t}")
            }
            Effect::PutCountersOf { from, to, kind } => {
                let f = self.sel(from, Case::Poss);
                let t = self.sel(to, Case::Obj);
                match kind {
                    Some(k) => format!("put {f} {k} counters on {t}"),
                    None => format!("put {f} counters on {t}"),
                }
            }
            Effect::Modify {
                what,
                mods,
                duration,
            } => self.modify_effect(what, mods, duration),
            Effect::AddRestriction {
                restriction,
                duration,
            } => {
                if let Restriction::DoesntUntap(f) = restriction {
                    // "Its controller's next untap step" and "your next untap step" are
                    // different durations (they differ once the permanent changes
                    // control), compiled as different `Duration`s.
                    let whose = match duration {
                        Duration::ThroughNextUntapStep => Some("its controller's"),
                        Duration::ThroughYourNextUntapStep => Some("your"),
                        _ => None,
                    };
                    if let Some(whose) = whose {
                        let s = self.restriction_subject(f);
                        return format!("{s} doesn't untap during {whose} next untap step");
                    }
                }
                let r = self.restriction(restriction);
                let d = self.restriction_duration(duration);
                // "~ can attack this turn as though it didn't have defender."
                if d == "this turn" && r.contains(" as though ") {
                    return r.replacen(" as though ", " this turn as though ", 1);
                }
                if d == "this turn" && r.contains(" can't be blocked except by ") {
                    return r.replacen(" except by ", " this turn except by ", 1);
                }
                // "Target creature blocks this turn if able."
                if d == "this turn" {
                    if let Some(x) = r.strip_suffix(" each combat if able") {
                        return format!("{x} this turn if able");
                    }
                }
                join_words(&[r, d])
            }
            Effect::AddPlayerEffect {
                who,
                effect,
                duration,
            } => {
                let s = self.player_modification(who, effect);
                let d = self.duration(duration);
                join_words(&[s, d])
            }
            Effect::AddReplacement {
                def,
                duration,
                uses,
            } => {
                let s = self.replacement(def, *uses);
                let d = self.duration(duration);
                // "If it would die this turn, exile it instead."
                if d == "until end of turn" && s.starts_with("if ") && s.contains(" would die,") {
                    return s.replacen(" would die,", " would die this turn,", 1);
                }
                join_words(&[s, d])
            }
            Effect::GainControl {
                what,
                who,
                duration,
            } => {
                let w = self.sel(what, Case::Obj);
                let d = self.duration(duration);
                let vp = join_words(&[format!("gain control of {w}"), d]);
                self.with_subject(who, &vp, false)
            }
            Effect::ExchangeControl { a, b } => {
                let a = self.sel(a, Case::Obj);
                let b = self.sel(b, Case::Obj);
                format!("exchange control of {a} and {b}")
            }
            Effect::CreateToken { .. } | Effect::CreateTokenWithPT { .. } => unreachable_text(),
            Effect::CreateTokenAttached {
                spec,
                count,
                controller,
                to,
            } => {
                let s = self.create_token(spec, count, false, false);
                let t = self.sel(to, Case::Obj);
                let vp = format!("{s} attached to {t}");
                self.with_subject(controller, &vp, false)
            }
            Effect::CreateTokenCopy {
                of,
                count,
                controller,
                tapped,
                attacking,
                mods,
            } => {
                let o = self.sel(of, Case::Obj);
                let mut status = Vec::new();
                if *tapped {
                    status.push("tapped");
                }
                if *attacking {
                    status.push("attacking");
                }
                let st = if status.is_empty() {
                    String::new()
                } else {
                    format!(" {}", status.join(" and "))
                };
                let head = match count {
                    Value::Const(1) => format!("a token that's a copy of {o}"),
                    other => {
                        let (c, w) = self.counted(other, "token");
                        format!("{c} that are copies of {o}{}", w.unwrap_or_default())
                    }
                };
                let mut vp = if st.is_empty() {
                    format!("create {head}")
                } else {
                    // "create a tapped and attacking token that's a copy of ..."
                    format!(
                        "create {}",
                        head.replacen("token", &format!("{} token", st.trim()), 1)
                    )
                };
                if !mods.is_empty() {
                    let ex = self.exceptions(mods);
                    vp.push_str(&format!(", except {ex}"));
                }
                self.with_subject(controller, &vp, false)
            }
            Effect::CounterSpell { what } => {
                let w = self.sel(what, Case::Obj);
                format!("counter {w}")
            }
            Effect::CopySpell {
                what,
                count,
                new_targets,
            } => {
                let w = self.sel(what, Case::Obj);
                let mut s = match count {
                    Value::Const(1) => format!("copy {w}"),
                    other => {
                        let t = self.times(other);
                        format!("copy {w} {t}")
                    }
                };
                if *new_targets {
                    let c = if matches!(count, Value::Const(1)) {
                        "the copy"
                    } else {
                        "the copies"
                    };
                    s.push_str(&format!(". You may choose new targets for {c}"));
                }
                s
            }
            Effect::OfferSpecialAction {
                def,
                duration,
                repeatable,
            } => {
                let d = self.duration(duration);
                let c = self.cost(&def.cost);
                let what = match &def.action {
                    SpecialActionEffect::Effect(e) => self.effect(e),
                    SpecialActionEffect::IgnoreSourceEffects => {
                        "ignore this effect until end of turn".into()
                    }
                };
                let once = if *repeatable {
                    ""
                } else {
                    " Do this only once"
                };
                format!(
                    "{d}, you may pay {c} any time you could cast an instant. If you do, {what}.{once}"
                )
            }
            Effect::PutSticker {
                who,
                what,
                kind,
                max_ticket,
                free,
            } => {
                let k = match kind {
                    None => "sticker",
                    Some(StickerType::Name) => "name sticker",
                    Some(StickerType::Ability) => "ability sticker",
                    Some(StickerType::PowerToughness) => "power and toughness sticker",
                    Some(StickerType::Art) => "art sticker",
                };
                let w = self.sel(what, Case::Obj);
                let mut vp = format!("put {} on {w}", with_article(k));
                if let Some(m) = max_ticket {
                    let m = self.value(m);
                    vp.push_str(&format!(" with ticket cost {m} or less"));
                }
                if *free {
                    vp.push_str(" without paying its ticket cost");
                }
                self.with_subject(who, &vp, false)
            }
            Effect::SpendAnyTypeMana {
                who,
                what,
                duration,
            } => {
                let w = self.sel(what, Case::Obj);
                let p = self.player(who, Case::Subj);
                let d = self.duration(duration);
                join_words(&[
                    format!("{p} may spend mana as though it were mana of any type to cast {w}"),
                    d,
                ])
            }
            Effect::ChangeTargets { what, who, how, to } => {
                let w = self.sel(what, Case::Obj);
                let vp = match (how, to) {
                    (_, Some(t)) => {
                        let t = self.sel(t, Case::Obj);
                        format!("change the target of {w} to {t}")
                    }
                    (TargetChange::All, None) => format!("change the target of {w}"),
                    (TargetChange::One, None) => format!("change a target of {w}"),
                    (TargetChange::Any, None) => format!("change any targets of {w}"),
                    (TargetChange::ChooseNew, None) => format!("choose new targets for {w}"),
                };
                // "Change the target of ..." is mandatory if a legal new target exists
                // (CR 115.7a; the rulings on Willbender and Ricochet Trap); "you may
                // change ..." is `Effect::May` around it.
                if matches!(who, PlayerRef::You) {
                    return vp;
                }
                self.gap("targets changed by a player other than you")
            }
            Effect::BecomeCopy { what, of, duration } => {
                let w = self.sel(what, Case::Subj);
                let o = self.sel(of, Case::Obj);
                let d = self.duration(duration);
                join_words(&[format!("{w} becomes a copy of {o}"), d])
            }
            Effect::BecomeCopyExcept {
                what,
                of,
                duration,
                exceptions,
            } => {
                let w = self.sel(what, Case::Subj);
                let o = self.sel(of, Case::Obj);
                let d = self.duration(duration);
                let ex = self.exceptions(exceptions);
                join_words(&[format!("{w} becomes a copy of {o}"), d]) + &format!(", except {ex}")
            }
            Effect::Transform { what } => {
                let w = self.sel(what, Case::Obj);
                format!("transform {w}")
            }
            Effect::Regenerate { what } => {
                let w = self.sel(what, Case::Obj);
                format!("regenerate {w}")
            }
            Effect::Attach { what, to } | Effect::AttachAsCreature { what, to } => {
                let w = self.sel(what, Case::Obj);
                let t = self.sel(to, Case::Obj);
                format!("attach {w} to {t}")
            }
            Effect::Unattach { what } => {
                let w = self.sel(what, Case::Obj);
                format!("unattach {w}")
            }
            Effect::PhaseOut { what } => {
                let w = self.sel(what, Case::Subj);
                format!("{w} phases out")
            }
            Effect::TurnFaceUp { what } => {
                let w = self.sel(what, Case::Obj);
                format!("turn {w} face up")
            }
            Effect::TurnFaceDown { what } => {
                let w = self.sel(what, Case::Obj);
                format!("turn {w} face down")
            }
            Effect::RemoveFromCombat { what } => {
                let w = self.sel(what, Case::Obj);
                format!("remove {w} from combat")
            }
            Effect::Choose { who, kind } => {
                let c = self.choice(kind);
                self.with_subject(who, &format!("choose {c}"), false)
            }
            Effect::EnterTapped => {
                let m = self.me();
                format!("{m} enters tapped")
            }
            Effect::EnterWithCounters { kind, n } => {
                let (c, w) = self.counted(n, &counter_name(kind));
                let m = self.me();
                format!("{m} enters with {c} on it{}", w.unwrap_or_default())
            }
            Effect::EnterPrepared => {
                let m = self.me();
                format!("{m} enters prepared")
            }
            Effect::EnterCopyExceptions(mods) => {
                let ex = self.exceptions(mods);
                format!("except {ex}")
            }
            Effect::EnterCopyExtra {
                only_if,
                mods,
                effect,
            } => {
                let mut parts = Vec::new();
                if let Some(f) = only_if {
                    let n = self.noun_det(f, Det::A);
                    parts.push(format!("if it's {n},"));
                }
                if !mods.is_empty() {
                    parts.push(self.exceptions(mods));
                }
                let e = self.effect(effect);
                if !e.is_empty() {
                    parts.push(e);
                }
                format!("except {}", parts.join(" "))
            }
            Effect::OnEntry(e) => self.effect(e),
            Effect::EnterAs(mods) => {
                let vp = self.mods_vp(mods, false);
                format!("it {vp}")
            }
            Effect::SetDayNight { day } => {
                if *day {
                    "it becomes day".into()
                } else {
                    "it becomes night".into()
                }
            }
            Effect::SetPrepared { what, prepared } => {
                let w = self.sel(what, Case::Subj);
                if *prepared {
                    format!("{w} becomes prepared")
                } else {
                    format!("{w} becomes unprepared")
                }
            }
            Effect::Draw { .. }
            | Effect::Discard { .. }
            | Effect::DiscardHand { .. }
            | Effect::Mill { .. }
            | Effect::GainLife { .. }
            | Effect::LoseLife { .. }
            | Effect::Scry { .. }
            | Effect::Surveil { .. }
            | Effect::Shuffle { .. }
            | Effect::RevealHand { .. }
            | Effect::AddPlayerCounters { .. }
            | Effect::ExtraTurn { .. }
            | Effect::WinGame { .. }
            | Effect::LoseGame { .. }
            | Effect::BecomeMonarch { .. }
            | Effect::TakeInitiative { .. }
            | Effect::Search { .. }
            | Effect::CreateEmblem { .. } => unreachable_text(),
            Effect::SetLife { who, n } => {
                let p = self.player(who, Case::Poss);
                let (a, w) = self.amount(n);
                format!("{p} life total becomes {a}{}", w.unwrap_or_default())
            }
            Effect::ExchangeLifeTotals { a, b } => {
                let a = self.player(a, Case::Subj);
                let b = self.player(b, Case::Obj);
                if a == "you" {
                    format!("exchange life totals with {b}")
                } else {
                    format!("{a} and {b} exchange life totals")
                }
            }
            Effect::AddMana {
                who,
                mana,
                restriction,
            } => {
                let m = self.mana_production(mana);
                let mut s = self.with_subject(who, &format!("add {m}"), false);
                if let Some(r) = restriction {
                    let r = self.mana_restriction(r);
                    s.push_str(&format!(". {r}"));
                }
                s
            }
            Effect::AddManaWithSpentTrigger {
                add,
                spell_filter,
                body,
            } => {
                let a = self.effect(add);
                let f = self.noun_det(spell_filter, Det::A);
                let f = if f.contains("spell") {
                    f
                } else {
                    format!("{f} spell")
                };
                let b = self.in_event_scope(|r| r.body(body));
                format!("{a}. When that mana is spent to cast {f}, {b}")
            }
            Effect::PersistentMana(e) => {
                let a = self.effect(e);
                format!("{a}. Until end of turn, you don't lose this mana as steps and phases end")
            }
            Effect::SetClassLevel { level } => format!("~'s level becomes {level}"),
            Effect::ActivateManaAbilities { who, filter } => {
                let w = self.player(who, Case::Subj);
                let f = self.noun_det(filter, Det::Each);
                format!("{w} activates a mana ability of {f}")
            }
            Effect::LoseUnspentMana { who, to } => {
                let w = self.player(who, Case::Subj);
                let mut s = format!("{w} loses all unspent mana");
                if to.is_some() {
                    s.push_str(" and you add the mana lost this way");
                }
                s
            }
            Effect::ShuffleInto { what } => {
                let w = self.sel(what, Case::Obj);
                let p = if is_plural_sel(what) {
                    "their owners' libraries"
                } else {
                    "its owner's library"
                };
                format!("shuffle {w} into {p}")
            }
            // "The owner of target artifact shuffles it into their library."
            Effect::ShuffleIntoLibrary {
                what,
                library: PlayerRef::OwnerOf(s),
            } if format!("{s:?}") == format!("{what:?}") => {
                let w = self.sel(what, Case::Obj);
                let it = if is_plural_sel(what) { "them" } else { "it" };
                format!("the owner of {w} shuffles {it} into their library")
            }
            Effect::ShuffleIntoLibrary { what, library } => {
                let w = self.sel(what, Case::Obj);
                let p = self.possessive_for(library);
                let vp = format!("shuffle {w} into {p} library");
                self.with_subject(library, &vp, false)
            }
            Effect::Dig {
                who,
                n,
                reveal,
                filter,
                take,
                take_up_to,
                take_to,
                rest_to,
            } => self.dig(who, n, *reveal, filter, take, *take_up_to, take_to, rest_to),
            Effect::LookAtHand { who } => {
                let p = self.player(who, Case::Poss);
                format!("look at {p} hand")
            }
            Effect::RevealUntil {
                who,
                filter,
                found_to,
                rest_to,
            } => {
                let p = self.possessive_for(who);
                let f = self.card_noun(filter);
                let f = with_article(&f);
                let found = self.destination_phrase(found_to, false, true);
                let rest = self.destination_phrase(rest_to, true, true);
                let vp = format!(
                    "reveal cards from the top of {p} library until {} reveal {f}. Put that card {found} and the rest {rest}",
                    if p == "your" { "you" } else { "they" }
                );
                self.with_subject(who, &vp, false)
            }
            Effect::ExtraTurnWith { who, at_start } => {
                let base = self.with_subject(who, "take an extra turn after this one", true);
                let a = self.effect(at_start);
                format!("{base}. {a}")
            }
            Effect::ExtraCombat { after_this } => {
                if *after_this {
                    "after this phase, there is an additional combat phase".into()
                } else {
                    "there is an additional combat phase".into()
                }
            }
            Effect::AddTurnParts {
                parts,
                after_phase,
                n,
                who,
            } => {
                let names: Vec<String> = parts
                    .iter()
                    .map(|p| match p {
                        TurnPart::BeginningPhase => "an additional beginning phase".to_string(),
                        TurnPart::CombatPhase => "an additional combat phase".into(),
                        TurnPart::MainPhase => "an additional main phase".into(),
                        TurnPart::Step(s) => format!("an additional {}", self.step_name(*s)),
                    })
                    .collect();
                let list = join_list(&names, "followed by");
                let after = if *after_phase {
                    "this phase"
                } else {
                    "this step"
                };
                let t = match n {
                    Value::Const(1) => String::new(),
                    other => format!(" {}", self.times(other)),
                };
                match who {
                    Some(p) => {
                        let p = self.player(p, Case::Subj);
                        format!("{p} get {list} after {after}{t}")
                    }
                    None => format!("after {after}, there is {list}{t}"),
                }
            }
            Effect::Skip { who, step } => {
                let p = self.possessive_for(who);
                let s = match step {
                    StepKind::Untap => "untap step",
                    StepKind::Upkeep => "upkeep",
                    StepKind::Draw => "draw step",
                    StepKind::Main => "main phase",
                    StepKind::Combat => "combat phase",
                    StepKind::End => "end step",
                    StepKind::Turn => "turn",
                };
                let vp = format!("skip {p} next {s}");
                self.with_subject(who, &vp, false)
            }
            Effect::DelayedTrigger {
                trigger,
                body,
                once,
            } => {
                // A one-shot delayed trigger at a step: "at the beginning of the next end
                // step" (CR 603.7).
                let t = match trigger {
                    TriggerCond::BeginningOf { step, whose } if *once => match (step, whose) {
                        (TriggerStep::EndOfCombat, _) => "at end of combat".to_string(),
                        (TriggerStep::Upkeep, PlayerRel::You) => {
                            "at the beginning of your next upkeep".to_string()
                        }
                        (TriggerStep::Upkeep, _) => {
                            "at the beginning of the next turn's upkeep".to_string()
                        }
                        (s, PlayerRel::You) if *s != TriggerStep::End => {
                            format!("at the beginning of your next {}", self.step_name(*s))
                        }
                        (s, _) => format!("at the beginning of the next {}", self.step_name(*s)),
                    },
                    // "When you next cast an instant spell this turn, ..."
                    other if *once => {
                        let t = self.trigger_text(other);
                        match t.strip_prefix("whenever you cast ") {
                            Some(r) => format!("when you next cast {r}"),
                            None => t,
                        }
                    }
                    other => self.trigger_text(other),
                };
                // Its own trigger condition: "When you next cast an instant spell this
                // turn, copy it" (it: that spell).
                let saved = (self.self_salient, self.other_salient, self.trigger_is_self);
                self.self_salient = false;
                self.other_salient = false;
                self.trigger_is_self = false;
                let b = self.in_event_scope(|r| r.body(body));
                (self.self_salient, self.other_salient, self.trigger_is_self) = saved;
                format!("{t}, {}", lower_first(&b))
            }
            Effect::Reflexive { body } => {
                let b = self.in_event_scope(|r| r.body(body));
                format!("when you do, {}", lower_first(&b))
            }
            Effect::AtNext { step, effect } => {
                let s = match step {
                    TriggerStep::EndOfCombat => "at end of combat".to_string(),
                    TriggerStep::Upkeep => "at the beginning of the next turn's upkeep".to_string(),
                    other => format!("at the beginning of the next {}", self.step_name(*other)),
                };
                let e = self.effect(effect);
                format!("{s}, {e}")
            }
            Effect::TokensEnterWithCounters { counters, effect } => {
                let e = self.effect(effect);
                let mut parts = Vec::new();
                for (k, n) in counters {
                    let (c, w) = self.counted(n, &counter_name(k));
                    parts.push(format!("{c}{}", w.unwrap_or_default()));
                }
                format!(
                    "{e}. {{alt:it|the token|they|the tokens}} enters with {} on it",
                    join_list(&parts, "and")
                )
            }
            Effect::WithPlayTerms { .. } => self.gap("permission to play with terms"),
            Effect::KeepAndSacrificeRest { up_to: true, .. } => {
                self.gap("keep up to some permanents, sacrifice the rest")
            }
            Effect::KeepAndSacrificeRest {
                who, among, keep, ..
            } => {
                let w = self.player(who, Case::Subj);
                let a = self.noun(among, Num::Many);
                let k: Vec<String> = keep.iter().map(|f| self.noun_det(f, Det::A)).collect();
                format!(
                    "{w} chooses from among the {a} they control {}, then sacrifices the rest",
                    join_list(&k, "and")
                )
            }
            Effect::RestartGame { keep } => match keep {
                None => "restart the game".into(),
                Some(s) => {
                    let s = self.sel(s, Case::Obj);
                    format!("restart the game, leaving in exile {s}")
                }
            },
            Effect::CastCard {
                who,
                what,
                free,
                optional,
            } => self.play_card("cast", who, what, *free, *optional),
            Effect::PlayCard {
                who,
                what,
                free,
                optional,
            } => self.play_card("play", who, what, *free, *optional),
            Effect::GrantPlayPermission {
                who,
                what,
                duration,
                free,
            } => {
                let w = self.sel(what, Case::Obj);
                let d = self.duration(duration);
                let f = if *free {
                    " without paying its mana cost"
                } else {
                    ""
                };
                let p = self.player(who, Case::Subj);
                join_words(&[format!("{p} may play {w}{f}"), d])
            }
            Effect::PreventDamage {
                to,
                amount,
                duration,
                combat_only,
            } => {
                let c = if *combat_only { "combat " } else { "" };
                let t = match to {
                    Sel::None => String::new(),
                    other => format!(" to {}", self.sel(other, Case::Obj)),
                };
                let d = self.duration(duration);
                let d = if d == "until end of turn" {
                    "this turn".to_string()
                } else {
                    d
                };
                match amount {
                    None => {
                        join_words(&[format!("prevent all {c}damage that would be dealt{t}"), d])
                    }
                    Some(v) => {
                        let (a, w) = self.amount(v);
                        join_words(&[
                            format!("prevent the next {a} {c}damage that would be dealt{t}"),
                            d,
                        ]) + &w.unwrap_or_default()
                    }
                }
            }
            Effect::PreventDividedDamage { slot, duration } => {
                let amount = self
                    .targets
                    .get(*slot as usize)
                    .and_then(|t| t.divide.clone());
                let a = match amount {
                    Some(v) => self.value(&v),
                    None => self.gap("divided prevention without amount"),
                };
                let d = self.duration(duration);
                let d = if d == "until end of turn" {
                    "this turn".to_string()
                } else {
                    d
                };
                let t = self.target_mention(*slot, Case::Obj);
                format!("prevent the next {a} damage that would be dealt {d} to {t}, divided as you choose")
            }
            Effect::KeywordAction {
                action,
                who,
                what,
                n,
            } => self.keyword_action(*action, who, what, n, None, &[]),
            Effect::KeywordActionEx(spec) => {
                let sub = spec.subtype.clone();
                let mut s = self.keyword_action(
                    spec.action,
                    &spec.who,
                    &spec.what,
                    &spec.n,
                    sub.as_deref(),
                    &spec.options,
                );
                if spec.undo {
                    s = format!("{s} (undo)");
                    s = self.gap(format!("keyword action undo: {s}"));
                }
                s
            }
            Effect::ChangeText {
                what,
                words,
                exclude,
                duration,
            } => {
                let w = self.sel(what, Case::Poss);
                let kind = match words {
                    crate::text_change::TextWords::Color => "color word",
                    crate::text_change::TextWords::BasicLandType => "basic land type",
                    crate::text_change::TextWords::ColorOrBasicLandType => {
                        "color word or basic land type"
                    }
                    crate::text_change::TextWords::CreatureType => "creature type",
                };
                let d = self.duration(duration);
                let mut s = join_words(&[
                    format!(
                        "change the text of {} by replacing all instances of one {kind} with another",
                        w.trim_end_matches("'s").trim_end_matches('\'')
                    ),
                    d,
                ]);
                if !exclude.is_empty() {
                    s.push_str(&format!(
                        ". The new {kind} can't be {}",
                        exclude.join(" or ")
                    ));
                }
                s
            }
            Effect::ExileUntil { what, until } => {
                let w = self.sel(what, Case::Obj);
                let u = until_event(until);
                format!("exile {w} until {u}")
            }
            Effect::PhaseOutUntil { what, until } => {
                let w = self.sel(what, Case::Subj);
                let u = until_event(until);
                format!("{w} phases out until {u}")
            }
            Effect::SelfReplace {
                replacement,
                effect,
            } => {
                let e = self.effect(effect);
                let r = self.replacement(replacement, None);
                format!("{e}. {r}")
            }
            Effect::ChooseSource { who, filter, .. } => {
                let w = self.player(who, Case::Subj);
                let f = self.noun_det(filter, Det::A);
                format!("{w} choose {f} source")
            }
            Effect::NextSpell {
                filter,
                mods,
                expires,
            } => {
                let f = self.noun(filter, Num::One);
                let f = if f.contains("spell") {
                    f
                } else {
                    format!("{f} spell")
                };
                let d = match expires {
                    Duration::EndOfTurn | Duration::ThisTurn => " this turn",
                    _ => "",
                };
                // "The next noncreature spell you cast this turn has affinity for artifacts."
                let vp = self.mods_vp(mods, false);
                format!("the next {f} you cast{d} {vp}")
            }
            Effect::CopySpellRetargeted { what, target } => {
                let w = self.sel(what, Case::Obj);
                match target {
                    None => format!("copy {w} for each other object or player it could target"),
                    Some(t) => {
                        let t = self.sel(t, Case::Obj);
                        format!("copy {w}. The copy targets {t}")
                    }
                }
            }
            Effect::CopyCard { what, named } => match named {
                None => {
                    let w = self.sel(what, Case::Obj);
                    format!("copy {w}")
                }
                Some(crate::copy_rules::NamedCopy::OneOf { names, unchosen }) => {
                    let u = if *unchosen {
                        " that hasn't been chosen"
                    } else {
                        ""
                    };
                    format!(
                        "choose a card name{u} from among {}. Create a copy of the card with the chosen name",
                        join_list(&names.iter().map(|n| n.to_string()).collect::<Vec<_>>(), "and")
                    )
                }
                Some(crate::copy_rules::NamedCopy::LastKnown) => {
                    "create a copy of the card with the noted name".into()
                }
            },
            Effect::RollDice(r) => self.roll_dice(r),
            Effect::FlipCoins(c) => self.flip_coins(c),
            Effect::Piles(p) => match p.as_ref() {
                crate::piles::PileAction::SeparateFaceDown { what, separator } => {
                    let w = self.sel(what, Case::Obj);
                    let s = self.player(separator, Case::Subj);
                    let vp = format!("separate {w} into a face-down pile and a face-up pile");
                    if s == "you" {
                        vp
                    } else {
                        format!("{s} {}", third_person(&vp))
                    }
                }
                crate::piles::PileAction::Separate { what, separator } => {
                    let w = self.sel(what, Case::Obj);
                    let s = self.player(separator, Case::Subj);
                    let vp = format!("separate {w} into two piles");
                    if s == "you" {
                        vp
                    } else {
                        format!("{s} separates {w} into two piles")
                    }
                }
                crate::piles::PileAction::Choose { chooser } => {
                    let c = self.player(chooser, Case::Subj);
                    format!("{c} chooses one of those piles")
                }
            },
            Effect::Exchange(x) => self.exchange(x),
            Effect::Custom(name) => self.custom_effect(name),
        }
    }

    fn times(&mut self, v: &Value) -> String {
        match v {
            Value::Const(1) => "once".into(),
            Value::Const(2) => "twice".into(),
            Value::Const(n) => format!("{} times", number_word(*n)),
            other => {
                let s = self.value(other);
                if Self::is_simple(other) {
                    format!("{s} times")
                } else {
                    format!("X times, where X is {s}")
                }
            }
        }
    }

    /// A sequence of effects, merging clauses with the same subject.
    fn seq(&mut self, v: &[Effect]) -> String {
        let mut parts: Vec<String> = Vec::new();
        let mut i = 0;
        while i < v.len() {
            // "Put a +1/+1 counter on each other creature you control. You gain 1 life for
            // each of those creatures.": the group is remembered silently, so the next
            // instruction must name it, either as the same group or as the remembered one
            // (whose first mention then spells it out).
            if let (
                Some(Effect::Store {
                    sel: all @ Sel::All(_),
                    var,
                }),
                next,
            ) = (v.get(i), v.get(i + 1))
            {
                let named = format!("{all:?}");
                let var = format!("{:?}", Sel::Var(*var));
                if !next.is_some_and(|n| {
                    let n = format!("{n:?}");
                    n.contains(&named) || n.contains(&var)
                }) {
                    parts.push(self.gap("a remembered group no instruction names"));
                }
            }
            // "Create a Treasure token and a 2/2 blue Bird creature token with flying."
            if let (
                Some(Effect::CreateToken { controller: c1, .. }),
                Some(Effect::CreateToken { controller: c2, .. }),
            ) = (v.get(i), v.get(i + 1))
            {
                if format!("{c1:?}") == format!("{c2:?}") {
                    let a = self.effect(&v[i]);
                    let b = self.effect(&v[i + 1]);
                    match b.strip_prefix("create ") {
                        Some(rest) if a.starts_with("create ") => {
                            parts.push(format!("{a} and {{opt:create}} {rest}"))
                        }
                        _ => {
                            parts.push(a);
                            parts.push(b);
                        }
                    }
                    i += 2;
                    continue;
                }
            }
            // "Target creature gains protection from the color of your choice": a color
            // chosen for the next instruction only.
            if let (
                Some(Effect::Choose {
                    who: PlayerRef::You,
                    kind: ChoiceKind::Color,
                }),
                Some(next),
            ) = (v.get(i), v.get(i + 1))
            {
                let later = format!("{:?}", &v[i + 2..]);
                if !later.contains("Chosen") {
                    let n = self.effect(next);
                    if n.matches("the chosen color").count() == 1 {
                        parts.push(format!(
                            "{{opt:choose a color}} {}",
                            n.replace(
                                "the chosen color",
                                "{alt:the chosen color|the color of your choice}"
                            )
                        ));
                        i += 2;
                        continue;
                    }
                    // The next instruction already says "the color of your choice".
                    if n.contains("the color of your choice") {
                        parts.push(n);
                        i += 2;
                        continue;
                    }
                    let c = self.effect(&v[i]);
                    parts.push(c);
                    parts.push(n);
                    i += 2;
                    continue;
                }
            }
            // "Each opponent may scry 1": each player chooses whether to take part
            // (`scry_rules::OPT_IN`), then those who did act.
            if let (
                Some(Effect::ForEachPlayer { who, effect: opt }),
                Some(Effect::Store {
                    var,
                    sel: Sel::Var(opted),
                }),
                Some(Effect::ForEachPlayer {
                    who: PlayerRef::Var(v2),
                    effect: act,
                }),
            ) = (v.get(i), v.get(i + 1), v.get(i + 2))
            {
                let is_opt_in = matches!(opt.as_ref(), Effect::May { who: PlayerRef::Iterated, effect }
                    if matches!(effect.as_ref(), Effect::Custom(n) if n == crate::scry_rules::OPT_IN));
                if is_opt_in && *opted == crate::scry_rules::OPTED && v2 == var {
                    let inner = match act.as_ref() {
                        Effect::AsPlayer {
                            who: PlayerRef::Iterated,
                            effect,
                        } => self.effect(effect),
                        other => {
                            let saved = self.trigger_player.take();
                            let s = self.effect(other);
                            self.trigger_player = saved;
                            s
                        }
                    };
                    let inner = inner
                        .replace("that player ", "")
                        .replace(" your ", " their ");
                    let w = self.player(who, Case::Subj);
                    parts.push(format!("{w} may {inner}"));
                    i += 3;
                    continue;
                }
            }
            // "~ gets +1/+0 until end of turn and can't be blocked this turn."
            if let (Some(Effect::Modify { .. }), Some(Effect::AddRestriction { .. })) =
                (v.get(i), v.get(i + 1))
            {
                let a = self.effect(&v[i]);
                let b = self.effect(&v[i + 1]);
                let rest = ["~it ", "it ", "they ", "them "]
                    .iter()
                    .find_map(|p| b.strip_prefix(p));
                match rest {
                    // Cards say both "... and can't be blocked" and "... It can't be
                    // blocked".
                    Some(r) => parts.push(format!("{a} and {{opt:it}} {r}")),
                    None => {
                        parts.push(a);
                        parts.push(b);
                    }
                }
                i += 2;
                continue;
            }
            // "Exile ~ with three time counters on it."
            if let (
                Some(Effect::Exile {
                    what,
                    face_down: false,
                    ..
                }),
                Some(Effect::AddCounters { what: w2, kind, n }),
            ) = (v.get(i), v.get(i + 1))
            {
                let same = same_sel(what, w2) || matches!(w2, Sel::Var(x) if *x == vars::IT);
                if same {
                    let w = self.sel(what, Case::Obj);
                    let (c, _) = self.counted(n, &counter_name(kind));
                    parts.push(format!("exile {w} with {c} on it"));
                    i += 2;
                    continue;
                }
            }
            // "Put two +1/+1 counters and a flying counter on ~."
            if let Effect::AddCounters { what, .. } = &v[i] {
                let mut j = i + 1;
                while let Some(Effect::AddCounters { what: w2, .. }) = v.get(j) {
                    if same_sel(what, w2) {
                        j += 1;
                    } else {
                        break;
                    }
                }
                if j > i + 1 {
                    let mut items = Vec::new();
                    for e in &v[i..j] {
                        if let Effect::AddCounters { kind, n, .. } = e {
                            let (c, _) = self.counted(n, &counter_name(kind));
                            items.push(c);
                        }
                    }
                    let t = self.sel(what, Case::Obj);
                    parts.push(format!("put {} on {t}", join_list(&items, "and")));
                    i = j;
                    continue;
                }
                // "Put a +1/+1 counter on it and a +1/+1 counter on ~."
                if let Some(Effect::AddCounters { .. }) = v.get(i + 1) {
                    let a = self.effect(&v[i]);
                    let b = self.effect(&v[i + 1]);
                    match b.strip_prefix("put ") {
                        Some(rest) => parts.push(format!("{a} and {{opt:put}} {rest}")),
                        None => {
                            parts.push(a);
                            parts.push(b);
                        }
                    }
                    i += 2;
                    continue;
                }
            }
            // "~ deals 1 damage to any target and 1 damage to you."
            if let Effect::DealDamage { source, .. } = &v[i] {
                let mut j = i + 1;
                while let Some(Effect::DealDamage { source: s2, .. }) = v.get(j) {
                    if same_sel(source, s2) {
                        j += 1;
                    } else {
                        break;
                    }
                }
                if j > i + 1 {
                    let s = self.sel(source, Case::Subj);
                    let mut items = Vec::new();
                    let mut wheres = String::new();
                    for e in &v[i..j] {
                        if let Effect::DealDamage { amount, to, .. } = e {
                            let (a, w) = self.amount(amount);
                            let t = self.sel(to, Case::Obj);
                            items.push(format!("{a} damage to {t}"));
                            wheres.push_str(&w.unwrap_or_default());
                        }
                    }
                    parts.push(format!("{s} deals {}{wheres}", join_list(&items, "and")));
                    i = j;
                    continue;
                }
            }
            // "Target creature gets +2/+2 and gains flying until end of turn."
            if let Effect::Modify {
                what,
                mods,
                duration,
            } = &v[i]
            {
                let mut all = mods.clone();
                let mut j = i + 1;
                while let Some(Effect::Modify {
                    what: w2,
                    mods: m2,
                    duration: d2,
                }) = v.get(j)
                {
                    if same_sel(what, w2) && same_duration(duration, d2) {
                        all.extend(m2.iter().cloned());
                        j += 1;
                    } else {
                        break;
                    }
                }
                if j > i + 1 {
                    parts.push(self.modify_effect(what, &all, duration));
                    i = j;
                    continue;
                }
            }
            // "Target player draws two cards and loses 2 life."
            if let Some((who, vp, keep)) = self.actor_vp_peek(&v[i]) {
                if !matches!(who, PlayerRef::You) {
                    let mut vps = vec![vp];
                    let mut j = i + 1;
                    while let Some(e2) = v.get(j) {
                        match self.actor_vp_peek(e2) {
                            Some((w2, vp2, _)) if same_player(&who, &w2) => {
                                vps.push(vp2);
                                j += 1;
                            }
                            _ => break,
                        }
                    }
                    if j > i + 1 {
                        // Render for real (introduces targets in order).
                        let mut vps = Vec::new();
                        let mut subj = String::new();
                        for (k, e) in v[i..j].iter().enumerate() {
                            let Some((w, vp, _)) = self.actor_vp(e) else {
                                vps.push(self.effect(e));
                                continue;
                            };
                            if k == 0 {
                                subj = self.player(&w, Case::Subj);
                            }
                            vps.push(third_person(&vp));
                        }
                        // One "where X is ..." for the clauses that share it.
                        let mut wheres: Vec<String> = Vec::new();
                        let vps: Vec<String> = vps
                            .into_iter()
                            .map(|vp| match vp.split_once(", where X is ") {
                                Some((a, w)) => {
                                    let w = format!(", where X is {w}");
                                    if !wheres.contains(&w) {
                                        wheres.push(w);
                                    }
                                    a.to_string()
                                }
                                None => vp,
                            })
                            .collect();
                        parts.push(format!(
                            "{subj} {}{}",
                            join_list(&vps, "and"),
                            wheres.concat()
                        ));
                        i = j;
                        let _ = keep;
                        continue;
                    }
                }
            }
            let s = self.effect(&v[i]);
            if !s.is_empty() {
                parts.push(s);
            }
            i += 1;
        }
        dedupe_where(&parts.join(". "))
    }

    /// `actor_vp` without side effects on target introduction or gaps.
    fn actor_vp_peek(&mut self, e: &Effect) -> Option<(PlayerRef, String, bool)> {
        let saved_i = self.introduced.clone();
        let saved_g = self.gaps.len();
        let r = self.actor_vp(e);
        self.introduced = saved_i;
        self.gaps.truncate(saved_g);
        r
    }

    fn if_effect(&mut self, cond: &Condition, then: &Effect, otherwise: &Effect) -> String {
        let then_empty = matches!(then, Effect::Noop);
        let else_empty = matches!(otherwise, Effect::Noop);
        match cond {
            Condition::PrevHappened if else_empty && matches!(then, Effect::Reflexive { .. }) => {
                // "When you do, ..." already means "if you do" (CR 603.12).
                self.effect(then)
            }
            // "Clash with an opponent. If you win, ..." (CR 701.30).
            Condition::PrevHappened if self.after_clash => {
                self.after_clash = false;
                let t = self.effect(then);
                format!("if you win, {t}")
            }
            Condition::PrevHappened => {
                let mut s = String::new();
                if !then_empty {
                    let t = self.effect(then);
                    s = format!("if you do, {t}");
                }
                if !else_empty {
                    let o = self.effect(otherwise);
                    let o = format!("if you don't, {o}");
                    s = if s.is_empty() { o } else { format!("{s}. {o}") };
                }
                s
            }
            Condition::Not(inner) if matches!(inner.as_ref(), Condition::PrevHappened) => {
                self.if_effect(&Condition::PrevHappened, otherwise, then)
            }
            // "Sacrifice it" needs no "if you control it" (CR 701.21a).
            Condition::SelMatches(s, Filter::ControlledBy(PlayerRel::You))
                if else_empty
                    && matches!(then, Effect::SacrificeObjects { what } if format!("{what:?}") == format!("{s:?}")) =>
            {
                self.effect(then)
            }
            // "X. If C, Y instead." (the comparison treats "If C, Y. Otherwise, X." the
            // same): the default effect comes first, so it names the targets.
            _ if !else_empty => {
                let o = self.effect(otherwise);
                let c = self.condition(cond);
                let mut t = self.effect(then);
                // "~ deals 3 damage to any target. If ..., it deals 4 damage instead."
                if let (Effect::DealDamage { to: a, .. }, Effect::DealDamage { to: b, .. }) =
                    (then, otherwise)
                {
                    if same_sel(a, b) {
                        if let Some(head) = t
                            .strip_suffix(" to it")
                            .or_else(|| t.strip_suffix(" to them"))
                        {
                            t = format!("{head} {{opt:to it}}");
                        }
                    }
                }
                format!("{o}. If {c}, {t} instead")
            }
            _ => {
                let c = self.condition(cond);
                let t = self.effect(then);
                // "Discard a card unless you attacked this turn."
                if let Some(inner) = c.strip_prefix("it's not true that ") {
                    return format!("{t} unless {inner}");
                }
                format!("if {c}, {t}")
            }
        }
    }

    fn may(&mut self, who: &PlayerRef, effect: &Effect) -> String {
        if let Some((w, vp, _)) = self.actor_vp(effect) {
            if same_player(&w, who) {
                let p = self.player(who, Case::Subj);
                return format!("{p} may {vp}");
            }
            let p = self.player(who, Case::Subj);
            let inner = self.with_subject(&w, &vp, true);
            return format!("{p} may have {inner}");
        }
        let p = self.player(who, Case::Subj);
        let inner = self.effect(effect);
        // An effect with its own subject: "you may have ~ deal 3 damage to ...".
        let has_subject = matches!(
            effect,
            Effect::DealDamage { .. }
                | Effect::DealDamageExcess { .. }
                | Effect::DealDividedDamage { .. }
                | Effect::Fight { .. }
                | Effect::BecomeCopy { .. }
                | Effect::BecomeCopyExcept { .. }
                | Effect::Modify { .. }
                | Effect::PhaseOut { .. }
                | Effect::AddRestriction { .. }
        ) || matches!(effect, Effect::KeywordAction { action, .. }
            if matches!(action, KeywordAction::Explore | KeywordAction::Connive | KeywordAction::Endure));
        if has_subject && !inner.starts_with("gain control") && !inner.starts_with("switch") {
            return format!("{p} may have {inner}");
        }
        format!("{p} may {inner}")
    }

    fn pay_optional(
        &mut self,
        who: &PlayerRef,
        cost: &Cost,
        then: &Effect,
        otherwise: &Effect,
    ) -> String {
        if matches!(then, Effect::Noop) && !matches!(otherwise, Effect::Noop) {
            let o = self.effect(otherwise);
            let p = self.player(who, Case::Subj);
            let pays = self.cost_as_payment(cost);
            if p == "you" {
                return format!("{o} unless you {pays}");
            }
            return format!("{o} unless {p} {}", third_person(&pays));
        }
        let p = self.player(who, Case::Subj);
        let pays = self.cost_as_payment(cost);
        let mut s = format!("{p} may {pays}");
        let they = if p == "you" { "you" } else { "they" };
        if !matches!(then, Effect::Noop) {
            let t = self.effect(then);
            s.push_str(&format!(". If {they} do, {t}"));
        }
        if !matches!(otherwise, Effect::Noop) {
            let o = self.effect(otherwise);
            s.push_str(&format!(". If {they} don't, {o}"));
        }
        s
    }

    fn play_card(
        &mut self,
        verb: &str,
        who: &PlayerRef,
        what: &Sel,
        free: bool,
        optional: bool,
    ) -> String {
        let w = self.sel(what, Case::Obj);
        let f = if free {
            " without paying its mana cost"
        } else {
            ""
        };
        let p = self.player(who, Case::Subj);
        if optional {
            format!("{p} may {verb} {w}{f}")
        } else if p == "you" {
            format!("{verb} {w}{f}")
        } else {
            format!("{p} {verb}s {w}{f}")
        }
    }

    /// Moving objects between zones.
    fn move_effect(&mut self, what: &Sel, to: &Destination) -> String {
        let mut w = self.sel(what, Case::Obj);
        // An ability that functions in a hidden or public zone moves the card from there
        // ("Return ~ from your graveyard to your hand", CR 113.6m).
        if matches!(what, Sel::This) {
            match self.zone {
                // Left out when the ability already said where it is ("if ~ is in your
                // graveyard, ... return it to your hand").
                FunctionZone::Graveyard => w.push_str(" {opt:from your graveyard}"),
                FunctionZone::Battlefield if self.attached_left => {
                    w.push_str(" {opt:from your graveyard}")
                }
                FunctionZone::Hand if to.zone != ZoneKind::Hand => w.push_str(" from your hand"),
                FunctionZone::Exile => w.push_str(" from exile"),
                _ => {}
            }
        }
        let plural = is_plural_sel(what);
        let yours = self.sel_is_yours(what);
        let d = self.destination_phrase(to, plural, yours);
        // Cards "return" what comes back from a graveyard, exile, or the battlefield,
        // and "put" what comes from a hand or library.
        let from_hidden = matches!(
            self.sel_zone(what),
            Some(ZoneKind::Hand | ZoneKind::Library)
        );
        let verb = match to.zone {
            ZoneKind::Hand | ZoneKind::Battlefield if !from_hidden => "return",
            ZoneKind::Exile => "exile",
            _ => "put",
        };
        let d = if verb == "put" && to.zone == ZoneKind::Battlefield {
            d.replacen("to the battlefield", "onto the battlefield", 1)
        } else if verb == "put" && to.zone == ZoneKind::Hand {
            d.replacen("to ", "into ", 1)
        } else {
            d
        };
        if to.zone == ZoneKind::Exile {
            return format!(
                "exile {w}{}",
                if d.is_empty() { "" } else { " " }.to_string() + &d
            );
        }
        if to.zone == ZoneKind::Library && to.position == LibraryPosition::Shuffled {
            let p = if plural {
                "their owners' libraries"
            } else {
                "its owner's library"
            };
            return format!("shuffle {w} into {p}");
        }
        // A permanent put onto the battlefield is under the control of the player who put
        // it there (CR 110.2a); "return" puts it under its owner's control instead.
        let d = if verb == "put" {
            d.replacen(" under your control", " {opt:under your control}", 1)
        } else {
            d
        };
        // Several cards put on top of or under a library go in the order their owner
        // chooses (CR 401.4): cards say "in any order" or leave it out.
        if plural
            && to.zone == ZoneKind::Library
            && matches!(to.position, LibraryPosition::Top | LibraryPosition::Bottom)
        {
            return format!("{verb} {w} {d} {{opt:in any order}}");
        }
        format!("{verb} {w} {d}")
    }

    /// "to its owner's hand", "onto the battlefield tapped under your control", "on top of
    /// its owner's library".
    pub(crate) fn destination_phrase(
        &mut self,
        to: &Destination,
        plural: bool,
        yours: bool,
    ) -> String {
        let owner = if plural {
            "their owners'"
        } else {
            "its owner's"
        };
        let mut s = match to.zone {
            ZoneKind::Hand => format!("to {owner} hand"),
            ZoneKind::Graveyard => format!("into {owner} graveyard"),
            ZoneKind::Exile => String::new(),
            ZoneKind::Library => match to.position {
                LibraryPosition::Top => format!("on top of {owner} library"),
                LibraryPosition::Bottom => format!("on the bottom of {owner} library"),
                LibraryPosition::BottomRandom => {
                    format!("on the bottom of {owner} library in a random order")
                }
                LibraryPosition::FromTop(n) => {
                    format!("into {owner} library {} from the top", ordinal_word(n + 1))
                }
                LibraryPosition::Shuffled => format!("into {owner} library"),
            },
            ZoneKind::Battlefield => "to the battlefield".into(),
            ZoneKind::Stack => "onto the stack".into(),
            ZoneKind::Command => "into the command zone".into(),
            ZoneKind::Ante => "into the ante".into(),
            ZoneKind::Outside => "outside the game".into(),
        };
        if to.face_down {
            s.push_str(" face down");
        }
        if to.zone == ZoneKind::Battlefield {
            let mut st = Vec::new();
            if to.tapped {
                st.push("tapped");
            }
            if to.attacking {
                st.push("attacking");
            }
            if !st.is_empty() {
                s.push_str(&format!(" {}", st.join(" and ")));
            }
            if to.transformed {
                s.push_str(" transformed");
            }
            // CR 110.2a: an object put onto the battlefield enters under the control of
            // the player putting it there; cards still say "under your control" when the
            // object isn't theirs, and "under its owner's control" when it goes back.
            match &to.controller {
                Some(PlayerRef::You) if yours => {}
                Some(c) => {
                    let c = self.player(c, Case::Poss);
                    s.push_str(&format!(" under {c} control"));
                }
                None if !yours => {
                    s.push_str(&format!(" under {owner} control"));
                }
                None => {}
            }
            if let Some(a) = &to.attached_to {
                let a = self.sel(a, Case::Obj);
                s.push_str(&format!(" attached to {a}"));
            }
            for (k, n) in &to.with_counters {
                let (c, _) = self.counted(n, &counter_name(k));
                s.push_str(&format!(" with {c} on it"));
            }
            if !to.with_mods.is_empty() {
                let vp = self.mods_vp(&to.with_mods, true);
                s.push_str(&format!(". It {vp}"));
            }
        }
        s
    }

    /// Search a library.
    fn search(&mut self, e: &Effect) -> String {
        let Effect::Search {
            who,
            whose,
            filter,
            count,
            to,
            reveal,
            shuffle,
        } = e
        else {
            return String::new();
        };
        let whose_s = if same_player(who, whose) {
            self.possessive_for(who)
        } else {
            self.player(whose, Case::Poss)
        };
        let det = match count {
            Value::Const(1) => Det::A,
            // An unbounded count: "any number of Dragon creature cards".
            Value::Const(n) if *n >= 99 => Det::Count("any number of".into()),
            Value::Const(n) => Det::UpTo(number_word(*n)),
            other => {
                let v = self.value(other);
                Det::UpTo(v)
            }
        };
        let f = self.noun_det(&filter.clone().in_zone(ZoneKind::Library), det);
        let f = f.replace(" in a library", "").replace(" in libraries", "");
        let many = !matches!(count, Value::Const(1));
        let pron = if many { "them" } else { "it" };
        let mut s = format!("search {whose_s} library for {f}");
        if *reveal {
            s.push_str(&format!(", reveal {pron}"));
        }
        // The searching player puts the card onto the battlefield under their own control
        // (CR 110.2a).
        let mut to2 = to.clone();
        if to2.controller.as_ref().is_some_and(|c| same_player(c, who)) {
            to2.controller = None;
        }
        let mut dest = self.destination_phrase(&to2, many, same_player(who, whose));
        if to.zone == ZoneKind::Battlefield {
            dest = dest.replacen("to the battlefield", "onto the battlefield", 1);
        }
        if to.zone == ZoneKind::Hand {
            dest = format!("into {} hand", self.possessive_for(who));
        }
        let top = to.zone == ZoneKind::Library && to.position == LibraryPosition::Top;
        if top && *shuffle {
            s.push_str(&format!(
                ", then shuffle and put {} on top",
                if many { "those cards" } else { "that card" }
            ));
            return s;
        }
        if to.zone == ZoneKind::Exile {
            s.push_str(&format!(", exile {pron}"));
        } else {
            s.push_str(&format!(", put {pron} {dest}"));
        }
        if *shuffle {
            s.push_str(", then shuffle");
        }
        s
    }

    #[allow(clippy::too_many_arguments)]
    fn dig(
        &mut self,
        who: &PlayerRef,
        n: &Value,
        reveal: bool,
        filter: &Filter,
        take: &Value,
        take_up_to: bool,
        take_to: &Destination,
        rest_to: &Destination,
    ) -> String {
        let p = self.possessive_for(who);
        let top = match n {
            Value::Const(1) => format!("the top card of {p} library"),
            Value::Const(k) => format!("the top {} cards of {p} library", number_word(*k)),
            other => {
                let v = self.value(other);
                format!("the top {v} cards of {p} library")
            }
        };
        let look = if reveal { "reveal" } else { "look at" };
        let mut s = self.with_subject(who, &format!("{look} {top}"), false);
        // Only looking ("Look at the top card of your library."), or looking and
        // putting them back ("then put them back in any order").
        if matches!(take, Value::Const(0)) {
            if rest_to.zone == ZoneKind::Library
                && rest_to.position == LibraryPosition::Top
                && !matches!(n, Value::Const(1))
            {
                s.push_str(". Put them back in any order");
            }
            return s;
        }
        // Taking every card looked at: "reveal the top card of your library and put that
        // card into your hand".
        if matches!(filter, Filter::Any) && !take_up_to && format!("{take:?}") == format!("{n:?}") {
            let many = !matches!(take, Value::Const(1));
            let mut d = self.destination_phrase(take_to, many, true);
            if take_to.zone == ZoneKind::Hand {
                d = format!("into {p} hand");
            }
            let pron = if many { "them" } else { "it" };
            s.push_str(&format!(". Put {pron} {d}"));
            return s;
        }
        let noun = self.card_noun(filter);
        let many = !matches!(take, Value::Const(1));
        let mut d = self.destination_phrase(take_to, many, true);
        if take_to.zone == ZoneKind::Hand {
            d = format!("into {p} hand");
        }
        if take_to.zone == ZoneKind::Battlefield {
            d = d.replacen("to the battlefield", "onto the battlefield", 1);
        }
        let count = match take {
            Value::Const(k) => number_word(*k),
            other => self.value(other),
        };
        let pron = if many { "them" } else { "it" };
        let take_s = if matches!(filter, Filter::Any) {
            if take_up_to {
                format!("put up to {count} of them {d}")
            } else {
                format!("put {count} of them {d}")
            }
        } else {
            let what = match take {
                Value::Const(1) => with_article(&noun),
                _ => format!("{count} {}", plural(&noun)),
            };
            let what = if take_up_to && many {
                format!("up to {what}")
            } else {
                what
            };
            let may = if take_up_to && !many { "you may " } else { "" };
            if !reveal && take_to.zone == ZoneKind::Hand {
                format!("{may}reveal {what} from among them and put {pron} {d}")
            } else {
                format!("{may}put {what} from among them {d}")
            }
        };
        let mut rest = self.destination_phrase(rest_to, true, true);
        if rest_to.zone == ZoneKind::Library {
            rest = rest.replace("their owners' library", &format!("{p} library"));
            if rest_to.position == LibraryPosition::Bottom {
                rest.push_str(" in any order");
            }
        }
        if rest_to.zone == ZoneKind::Graveyard {
            rest = format!("into {p} graveyard");
        }
        let left = match (n, take) {
            (Value::Const(a), Value::Const(b)) if a - b == 1 => "the other",
            _ => "the rest",
        };
        if matches!(filter, Filter::Any) {
            s.push_str(&format!(". {} and {left} {rest}", capitalize(&take_s)));
        } else {
            s.push_str(&format!(". {}. Put {left} {rest}", capitalize(&take_s)));
        }
        s
    }

    fn create_token(
        &mut self,
        spec: &TokenSpec,
        count: &Value,
        tapped: bool,
        attacking: bool,
    ) -> String {
        let desc = self.token_desc(spec);
        let mut st = Vec::new();
        if tapped {
            st.push("tapped");
        }
        if attacking {
            st.push("attacking");
        }
        let status = st.join(" and ");
        let (head, tail) = desc;
        // A named token: "create Scion of the Deep, a legendary 8/8 blue Octopus creature
        // token".
        if let Some(rest) = tail.strip_prefix(" named ") {
            if matches!(count, Value::Const(1)) && status.is_empty() {
                let (name, with) = match rest.split_once(" with ") {
                    Some((n, w)) => (n.to_string(), format!(" with {w}")),
                    None => (rest.to_string(), String::new()),
                };
                return format!("create {name}, {} token{with}", with_article(&head));
            }
        }
        let noun_one = join_words(&[status.clone(), head.clone(), "token".into()]);
        let noun_many = join_words(&[status, head, "tokens".into()]);
        let (c, w) = match count {
            Value::Const(1) => (with_article(&noun_one), None),
            Value::Const(n) => (format!("{} {noun_many}", number_word(*n)), None),
            Value::X => (format!("X {noun_many}"), None),
            other => {
                let v = self.value(other);
                (format!("X {noun_many}"), Some(format!(", where X is {v}")))
            }
        };
        format!("create {c}{tail}{}", w.unwrap_or_default())
    }

    /// ("1/1 white Soldier creature", " with flying") for a token.
    pub(crate) fn token_desc(&mut self, spec: &TokenSpec) -> (String, String) {
        // Predefined tokens (CR 111.10) are named by their name ("a Treasure token", "a
        // Monster Role token"), when the spec is exactly that token.
        let candidates: Vec<String> = if spec.name.is_empty() {
            spec.subtypes.iter().map(|s| s.to_string()).collect()
        } else {
            vec![spec.name.to_string()]
        };
        for cand in candidates {
            if let Some(mut p) = crate::tokens_predefined::predefined(&cand) {
                p.name = spec.name.clone();
                p.scryfall_name = spec.scryfall_name.clone();
                let saved = self.gaps.len();
                let a = self.token_desc_full(&p);
                let b = self.token_desc_full(spec);
                self.gaps.truncate(saved);
                if a == b {
                    let is_role = spec.subtypes.iter().any(|s| s == "Role");
                    let n = if is_role && cand != "Role" {
                        format!("{cand} Role")
                    } else {
                        cand
                    };
                    return (n, String::new());
                }
            }
        }
        self.token_desc_full(spec)
    }

    fn token_desc_full(&mut self, spec: &TokenSpec) -> (String, String) {
        let mut words: Vec<String> = Vec::new();
        // "a legendary 8/8 blue Octopus creature token".
        for s in &spec.supertypes {
            words.push(nouns::supertype_word(*s).to_string());
        }
        let mut where_x = String::new();
        if let Some(pt) = &spec.pt_values {
            // "an X/X green Ooze creature token, where X is ..." (CR 107.3).
            let (p, t) = pt.as_ref();
            let ps = self.value(p);
            if format!("{p:?}") == format!("{t:?}") {
                words.push("X/X".into());
                if ps != "X" {
                    where_x = format!(", where X is {ps}");
                }
            } else {
                let ts = self.value(t);
                words.push(format!("{ps}/{ts}"));
            }
        } else if let (Some(p), Some(t)) = (spec.power, spec.toughness) {
            words.push(format!("{p}/{t}"));
        }
        let colors: Vec<String> = color_words(spec.colors);
        if colors.is_empty() {
            words.push("colorless".into());
        } else {
            words.push(join_list(&colors, "and"));
        }
        for s in &spec.subtypes {
            words.push(s.to_string());
        }
        let mut types = spec.card_types.clone();
        types.sort_by_key(|t| match t {
            CardType::Enchantment => 0,
            CardType::Artifact => 1,
            CardType::Land => 2,
            CardType::Creature => 3,
            _ => 4,
        });
        for t in &types {
            words.push(t.word().to_string());
        }
        let mut tail = String::new();
        if !spec.name.is_empty()
            && !spec
                .subtypes
                .iter()
                .any(|s| s.as_str() == spec.name.as_str())
        {
            tail.push_str(&format!(" named {}", spec.name));
        }
        let mut kws = Vec::new();
        let mut others = Vec::new();
        for a in &spec.abilities {
            match &a.kind {
                AbilityKind::Keyword(k) => kws.push(self.keyword_lower(k)),
                _ => others.push(format!("\"{}\"", self.nested_ability(a))),
            }
        }
        let mut with = Vec::new();
        if !kws.is_empty() {
            with.push(join_list(&kws, "and"));
        }
        with.extend(others);
        if !with.is_empty() {
            tail.push_str(&format!(" with {}", join_list(&with, "and")));
        }
        tail.push_str(&where_x);
        (words.join(" "), tail)
    }

    fn emblem(&mut self, abilities: &[Ability]) -> String {
        let parts: Vec<String> = abilities
            .iter()
            .map(|a| format!("\"{}\"", self.nested_ability(a)))
            .collect();
        format!("get an emblem with {}", join_list(&parts, "and"))
    }

    /// The modifications of a resolving effect or static ability as a verb phrase.
    pub(crate) fn modify_effect(
        &mut self,
        what: &Sel,
        mods: &[Modification],
        d: &Duration,
    ) -> String {
        if mods.len() == 1 {
            if let Modification::SwitchPT = mods[0] {
                let w = self.sel(what, Case::Poss);
                let d = self.duration(d);
                return join_words(&[format!("switch {w} power and toughness"), d]);
            }
            if let Modification::SetController(p) = &mods[0] {
                let w = self.sel(what, Case::Obj);
                let d = self.duration(d);
                let vp = join_words(&[format!("gain control of {w}"), d]);
                return self.with_subject(p, &vp, false);
            }
        }
        self.subject_types = self.sel_types(what);
        let w = self.sel(what, Case::Subj);
        let (vp, tail) = self.mods_vp_split(mods, true);
        self.subject_types.clear();
        let d = self.duration(d);
        join_words(&[format!("{w} {vp}"), d]) + &tail
    }

    /// The zone the selected objects are known to be in.
    pub(crate) fn sel_zone(&self, s: &Sel) -> Option<ZoneKind> {
        match s {
            Sel::This => match self.zone {
                FunctionZone::Graveyard => Some(ZoneKind::Graveyard),
                FunctionZone::Hand => Some(ZoneKind::Hand),
                FunctionZone::Exile => Some(ZoneKind::Exile),
                FunctionZone::Library => Some(ZoneKind::Library),
                _ => Some(ZoneKind::Battlefield),
            },
            Sel::Target(i) => match self.targets.get(*i as usize) {
                Some(TargetSpec {
                    what: TargetKind::Object(f),
                    ..
                }) => f.zone(),
                _ => None,
            },
            Sel::All(f) | Sel::Choose { filter: f, .. } => f.zone(),
            Sel::TopOfLibrary(..) => Some(ZoneKind::Library),
            _ => None,
        }
    }

    /// Whether the selected objects are known to be the controller's own (their own
    /// graveyard's cards, the source).
    pub(crate) fn sel_is_yours(&self, s: &Sel) -> bool {
        fn owned(f: &Filter) -> bool {
            match f {
                Filter::OwnedBy(PlayerRel::You) | Filter::Source => true,
                Filter::And(v) => v.iter().any(owned),
                _ => false,
            }
        }
        match s {
            Sel::This => true,
            Sel::Target(i) => match self.targets.get(*i as usize) {
                Some(TargetSpec {
                    what: TargetKind::Object(f),
                    ..
                }) => owned(f),
                _ => false,
            },
            Sel::All(f) | Sel::Choose { filter: f, .. } => owned(f),
            _ => false,
        }
    }

    /// The card types a selection is known to have (from a target's or the source's
    /// description), for "It's still a land".
    pub(crate) fn sel_types(&self, s: &Sel) -> Vec<CardType> {
        fn types_of(f: &Filter, out: &mut Vec<CardType>) {
            match f {
                Filter::Type(t) => out.push(*t),
                Filter::And(v) => v.iter().for_each(|x| types_of(x, out)),
                _ => {}
            }
        }
        let mut out = Vec::new();
        match s {
            Sel::This => out.extend(self.info.card_types.iter()),
            Sel::Target(i) => {
                if let Some(TargetSpec {
                    what: TargetKind::Object(f),
                    ..
                }) = self.targets.get(*i as usize)
                {
                    types_of(f, &mut out);
                }
            }
            Sel::All(f) => types_of(f, &mut out),
            _ => {}
        }
        out
    }

    /// Modifications as a verb phrase: "gets +1/+1 and has flying" (static) / "gains"
    /// (resolving, `gains = true`).
    pub(crate) fn mods_vp(&mut self, mods: &[Modification], gains: bool) -> String {
        let (vp, tail) = self.mods_vp_split(mods, gains);
        format!("{vp}{tail}")
    }

    /// [`Self::mods_vp`] with the trailing clauses ("where X is ...", ". It's still a
    /// land") separate, so that a duration can go between.
    pub(crate) fn mods_vp_split(&mut self, mods: &[Modification], gains: bool) -> (String, String) {
        let mut parts: Vec<String> = Vec::new();
        let mut keywords: Vec<String> = Vec::new();
        let mut abilities: Vec<String> = Vec::new();
        // "becomes a 3/3 Elemental creature with flying" collects these.
        let mut becomes = Becomes::default();
        let mut where_clauses: Vec<String> = Vec::new();
        for m in mods {
            match m {
                Modification::ModifyPT(p, t) => {
                    let (ps, pw) = self.pt_amount(p);
                    let (ts, tw) = self.pt_amount(t);
                    if let Some(w) = pw.or(tw) {
                        where_clauses.push(w);
                    }
                    let (mut a, mut b) = (signed(&ps), signed(&ts));
                    // "-2/-0": a zero next to a negative modifier is printed "-0".
                    if a == "+0" && b.starts_with('-') {
                        a = "-0".into();
                    }
                    if b == "+0" && a.starts_with('-') {
                        b = "-0".into();
                    }
                    parts.push(format!("gets {a}/{b}"));
                }
                Modification::AddKeyword(k) => {
                    if keywords.is_empty() && abilities.is_empty() {
                        parts.push(GRANTS.into());
                    }
                    keywords.push(self.keyword_lower(k));
                    // A granted cost keyword without a cost uses the mana cost:
                    // "gains flashback until end of turn. The flashback cost is equal to
                    // its mana cost."
                    use crate::keywords::KeywordKind as K;
                    if k.cost.is_none()
                        && matches!(k.kind, K::Flashback | K::Madness | K::Replicate | K::Escape | K::Disturb | K::Warp)
                    {
                        let n = k.kind.name().to_lowercase();
                        where_clauses.push(format!(". The {n} cost is equal to its mana cost"));
                    }
                }
                Modification::AddKeywordX(k, v) => {
                    let s = self.keyword_lower(k);
                    let v = self.value(v);
                    keywords.push(s);
                    where_clauses.push(format!(", where X is {v}"));
                }
                Modification::AddAbility(a) => {
                    let s = self.nested_ability(a);
                    abilities.push(format!("\"{s}\""));
                }
                Modification::AddThisAbility => abilities.push("this ability".into()),
                Modification::AddKeywordsOf { kinds, from } => {
                    let f = self.noun_det(from, Det::A);
                    let k: Vec<String> = kinds.iter().map(|k| self.keyword_kind_word(*k)).collect();
                    parts.push(format!(
                        "has {} as long as {f} has that ability",
                        join_list(&k, "and")
                    ));
                }
                Modification::RemoveKeyword(k) => {
                    parts.push(format!("loses {}", self.keyword_kind_word(*k)))
                }
                Modification::RemoveAllAbilities => parts.push("loses all abilities".into()),
                Modification::CantHaveKeyword(k) => {
                    parts.push(format!("can't have or gain {}", self.keyword_kind_word(*k)))
                }
                Modification::SetPT(p, t) | Modification::CdaPT(p, t) => {
                    becomes.pt = Some((p.clone(), t.clone()));
                }
                Modification::SwitchPT => parts.push("has its power and toughness switched".into()),
                Modification::Custom { name, .. } => {
                    let t = self.custom_modification(name);
                    parts.push(t);
                }
                Modification::SetController(p) => {
                    let p = self.player(p, Case::Subj);
                    parts.push(format!("is controlled by {p}"));
                }
                Modification::ChangeText { from, to } => parts.push(format!(
                    "has its text changed by replacing all instances of {from} with {to}"
                )),
                Modification::SetName(n) => becomes.name = Some(n.to_string()),
                Modification::ExchangeText => parts.push("exchanges text boxes".into()),
                Modification::SetText { .. } => {
                    let g = self.gap("Modification::SetText");
                    parts.push(g)
                }
                Modification::FullTextOf(s) => {
                    let s = self.sel(s, Case::Poss);
                    parts.push(format!("has the full text of {s}"));
                }
                Modification::AddText { .. } => {
                    let g = self.gap("Modification::AddText");
                    parts.push(g)
                }
                Modification::AllCreatureNames => parts.push(
                    "has all names of nonlegendary creature cards in addition to its name".into(),
                ),
                Modification::NameSticker { .. } => {
                    let g = self.gap("Modification::NameSticker");
                    parts.push(g)
                }
                Modification::NoManaCost => parts.push("has no mana cost".into()),
                Modification::AddTypes(t) => {
                    becomes.add_types.extend(t.iter().copied());
                    becomes.additive = true;
                }
                Modification::RemoveTypes(t) => {
                    let w: Vec<String> = t.iter().map(|x| x.word().to_string()).collect();
                    parts.push(format!("isn't {}", with_article(&join_list(&w, "or"))));
                }
                Modification::AddSupertypes(s) => {
                    becomes
                        .supertypes
                        .extend(s.iter().map(|x| nouns::supertype_word(*x).to_string()));
                    becomes.additive = true;
                }
                Modification::RemoveSupertypes(s) => {
                    let w: Vec<String> = s
                        .iter()
                        .map(|x| nouns::supertype_word(*x).to_string())
                        .collect();
                    parts.push(format!("isn't {}", join_list(&w, "or")));
                }
                Modification::AddSubtypes(s) => {
                    becomes.subtypes.extend(s.iter().map(|x| x.to_string()));
                    becomes.additive = true;
                }
                Modification::RemoveSubtypes(s) => {
                    let w: Vec<String> = s.iter().map(|x| x.to_string()).collect();
                    parts.push(format!("isn't {}", with_article(&join_list(&w, "or"))));
                }
                Modification::SetTypes { types, subtypes } => {
                    becomes.add_types.extend(types.iter().copied());
                    becomes
                        .subtypes
                        .extend(subtypes.iter().map(|x| x.to_string()));
                }
                Modification::AllCreatureTypes => parts.push("is every creature type".into()),
                // "becomes a 4/4 Dragon artifact creature": the new creature types replace
                // the old ones (CR 205.1b), which the effect records as removing them.
                Modification::RemoveAllCreatureTypes
                    if mods.iter().any(|m| {
                        matches!(m, Modification::AddSubtypes(v) if !v.is_empty())
                            || matches!(m, Modification::SetTypes { subtypes, .. } if !subtypes.is_empty())
                    }) => {}
                Modification::RemoveAllCreatureTypes => {
                    parts.push("loses all creature types".into())
                }
                Modification::SetBasicLandType(s) => {
                    becomes.subtypes.extend(s.iter().map(|x| x.to_string()));
                    becomes.land_type = true;
                }
                // "This land is the chosen type" on a land with no land type of its own
                // adds it (the land keeps its abilities).
                Modification::AddChosenType => parts.push(
                    "is the chosen type {opt:in addition to its other types}".into(),
                ),
                Modification::SetChosenBasicLandType => parts.push("is the chosen type".into()),
                Modification::SetColors(cs) => {
                    let w: Vec<String> = color_words(*cs);
                    becomes.colors = Some(if w.is_empty() {
                        "colorless".into()
                    } else {
                        join_list(&w, "and")
                    });
                }
                Modification::SetLinkedChosenColor => parts.push("is the chosen color".into()),
                Modification::AddColors(cs) => {
                    let w: Vec<String> = cs.iter().map(|c| c.word().to_string()).collect();
                    parts.push(format!(
                        "is {} in addition to its other colors",
                        join_list(&w, "and")
                    ));
                }
                Modification::SetChosenColor => {
                    parts.push("becomes the color of your choice".into())
                }
                Modification::SetChosenColors => {
                    parts.push("becomes the color or colors of your choice".into())
                }
            }
        }
        let has = if gains { "gains" } else { "has" };
        if let Some((b, still)) = becomes.render(self, &keywords, &abilities, gains) {
            // Negations come first ("except it isn't legendary and is a 4/4 Hero").
            let at = parts.iter().take_while(|p| p.starts_with("isn't")).count();
            parts.insert(at, b);
            if !still.is_empty() {
                where_clauses.push(still);
            }
        } else {
            // "protection from each color" (CR 702.16h).
            let all_colors = ["white", "blue", "black", "red", "green"]
                .iter()
                .all(|c| keywords.contains(&format!("protection from {c}")));
            if all_colors {
                keywords.retain(|k| {
                    !["white", "blue", "black", "red", "green"]
                        .iter()
                        .any(|c| *k == format!("protection from {c}"))
                });
                keywords.push("protection from each color".into());
            }
            // "protection from green and from blue" (CR 702.16a: one quality each).
            let prot: Vec<String> = keywords
                .iter()
                .filter_map(|k| k.strip_prefix("protection from ").map(str::to_string))
                .collect();
            if prot.len() >= 2 {
                if let Some(first) = keywords
                    .iter()
                    .position(|k| k.starts_with("protection from "))
                {
                    keywords.retain(|k| !k.starts_with("protection from "));
                    let froms: Vec<String> = prot.iter().map(|q| format!("from {q}")).collect();
                    let merged = format!("protection {}", join_list(&froms, "and"));
                    keywords.insert(first.min(keywords.len()), merged);
                }
            }
            let mut grants = keywords.clone();
            grants.extend(abilities.iter().cloned());
            if !grants.is_empty() {
                let g = format!("{has} {}", join_list(&grants, "and"));
                match parts.iter().position(|p| p == GRANTS) {
                    Some(i) => parts[i] = g,
                    None => parts.push(g),
                }
            }
        }
        parts.retain(|p| p != GRANTS);
        // An "isn't" part moves before the rest: cards list it first.
        parts.sort_by_key(|p| !p.starts_with("isn't"));
        let s = join_list(&parts, "and");
        (s, where_clauses.concat())
    }

    fn pt_amount(&mut self, v: &Value) -> (String, Option<String>) {
        match v {
            Value::Const(n) => (n.to_string(), None),
            Value::X => ("X".into(), None),
            // "+1/+0 for each ...": no change to the other number.
            Value::Mul(a, b)
                if matches!(a.as_ref(), Value::Const(0))
                    || matches!(b.as_ref(), Value::Const(0)) =>
            {
                ("0".into(), None)
            }
            Value::Mul(a, b) if matches!(a.as_ref(), Value::Const(-1)) => {
                let (s, w) = self.pt_amount(b);
                (format!("-{s}"), w)
            }
            Value::Diff(a, b) if matches!(a.as_ref(), Value::Const(0)) => {
                let (s, w) = self.pt_amount(b);
                (format!("-{s}"), w)
            }
            other => {
                let s = self.value(other);
                ("X".into(), Some(format!(", where X is {s}")))
            }
        }
    }

    /// Copy exceptions ("it's 1/1", "it has haste", "it isn't legendary").
    pub(crate) fn exceptions(&mut self, mods: &[Modification]) -> String {
        let vp = self.mods_vp(mods, false);
        // "except it isn't legendary, it's a 4/4 Hero": each exception has its subject.
        // A copy's "except it's a 4/4 black Zombie" keeps the copied types (the rulings
        // on eternalize and similar copies), with or without "in addition to its other
        // types".
        format!(
            "it {}",
            vp.replace(" and is ", " and it is ")
                .replace(" and has ", " and it has ")
                .replace(
                    " in addition to its other types",
                    " {opt:in addition to its other types}"
                )
        )
    }

    /// A choice ("a color", "a creature type", "an opponent").
    pub(crate) fn choice(&mut self, k: &ChoiceKind) -> String {
        match k {
            ChoiceKind::Color => "a color".into(),
            ChoiceKind::ColorOtherThan(c) => format!("a color other than {}", c.word()),
            ChoiceKind::Colors => "one or more colors".into(),
            ChoiceKind::OneOf(v) => join_list(v, "or"),
            ChoiceKind::CreatureType => "a creature type".into(),
            ChoiceKind::CardName => "a card name".into(),
            ChoiceKind::CardNameFiltered(f) => format!("a {f} card name"),
            ChoiceKind::Number { min, max } => format!("a number from {min} to {max}"),
            ChoiceKind::Opponent => "an opponent".into(),
            ChoiceKind::Player => "a player".into(),
            ChoiceKind::BasicLandType => "a basic land type".into(),
            ChoiceKind::CardType => "a card type".into(),
            ChoiceKind::OddOrEven => "odd or even".into(),
            ChoiceKind::Word(v) => join_list(v, "or"),
        }
    }

    /// What mana an effect adds.
    pub(crate) fn mana_production(&mut self, m: &ManaProduction) -> String {
        match m {
            ManaProduction::Fixed(v) => v.iter().map(|t| mana_symbol(*t)).collect(),
            ManaProduction::AnyOneColor(n) => match n {
                Value::Const(1) => "one mana of any color".into(),
                Value::Const(k) => format!("{} mana of any one color", number_word(*k)),
                other => {
                    let (a, w) = self.amount(other);
                    format!("{a} mana of any one color{}", w.unwrap_or_default())
                }
            },
            ManaProduction::AnyCombination(n) => {
                let (a, w) = match n {
                    Value::Const(k) => (number_word(*k), None),
                    other => self.amount(other),
                };
                format!(
                    "{a} mana in any combination of colors{}",
                    w.unwrap_or_default()
                )
            }
            ManaProduction::CombinationOf(types, n) => {
                let (a, w) = match n {
                    Value::Const(k) => (number_word(*k), None),
                    other => self.amount(other),
                };
                let syms: Vec<String> = types.iter().map(|t| mana_symbol(*t)).collect();
                format!(
                    "{a} mana in any combination of {}{}",
                    join_list(&syms, "and/or"),
                    w.unwrap_or_default()
                )
            }
            ManaProduction::OneOf(types) => {
                let syms: Vec<String> = types.iter().map(|t| mana_symbol(*t)).collect();
                join_list(&syms, "or")
            }
            ManaProduction::CouldProduce(f) => {
                let n = self.noun_det(f, Det::A);
                format!("one mana of any type that {n} could produce")
            }
            ManaProduction::CouldProduceColor(f) => {
                let n = self.noun_det(f, Det::A);
                format!("one mana of any color that {n} could produce")
            }
            ManaProduction::ChosenColor(n) => match n {
                Value::Const(1) => "one mana of the chosen color".into(),
                Value::Const(k) => format!("{} mana of the chosen color", number_word(*k)),
                other => {
                    let (a, w) = self.amount(other);
                    format!("{a} mana of the chosen color{}", w.unwrap_or_default())
                }
            },
            ManaProduction::OneOfOrChosenColor(types) => {
                let syms: Vec<String> = types.iter().map(|t| mana_symbol(*t)).collect();
                format!("{} or one mana of the chosen color", join_list(&syms, "or"))
            }
            ManaProduction::Amount(t, v) => {
                let sym = mana_symbol(*t);
                match v {
                    Value::Const(k) if *k > 0 => sym.repeat(*k as usize),
                    Value::Count(f) => {
                        let n = self.for_each_noun(f);
                        format!("{sym} for each {}", n)
                    }
                    Value::X if self.x_for_each.is_some() => {
                        let fe = self.x_for_each.clone().unwrap_or_default();
                        format!("{sym} {fe}")
                    }
                    other => {
                        let s = self.value(other);
                        format!("an amount of {sym} equal to {s}")
                    }
                }
            }
            ManaProduction::AnyColorAmong(f) => {
                let n = self.noun(f, Num::Many);
                format!("one mana of any color among {n}")
            }
            ManaProduction::AnyTypeProduced => "one mana of any type that land produced".into(),
            ManaProduction::ManaCostOf(s) => {
                let s = self.sel(s, Case::Poss);
                format!("mana equal to {s} mana cost")
            }
            ManaProduction::DoubleUnspent => {
                "mana equal to the amount of each type of unspent mana you have".into()
            }
            ManaProduction::TypeProduced => {
                "one additional mana of any type that permanent produced".into()
            }
            ManaProduction::CommanderIdentity => {
                "one mana of any color in your commander's color identity".into()
            }
        }
    }

    pub(crate) fn mana_restriction(&mut self, r: &crate::mana::ManaRestriction) -> String {
        format!("Spend this mana only {}", self.mana_restriction_purpose(r))
    }

    fn mana_restriction_purpose(&mut self, r: &crate::mana::ManaRestriction) -> String {
        use crate::mana::ManaRestriction as M;
        match r {
            M::SpellOfType(t) => format!("to cast {} spells", t.word()),
            M::SpellWithSubtype(s) => format!("to cast {s} spells"),
            M::SpellOfChosenType => "to cast a creature spell of the chosen type".into(),
            M::SpellsOnly => "to cast spells".into(),
            M::AbilitiesOnly => "to activate abilities".into(),
            M::XCostsOnly => "on costs that include {X}".into(),
            M::ArtifactSpellOrAbility => {
                "to cast artifact spells or activate abilities of artifacts".into()
            }
            M::InstantOrSorcery => "to cast instant or sorcery spells".into(),
            M::NoncreatureSpell => "to cast noncreature spells".into(),
            M::NotNonartifactSpell => "This mana can't be spent to cast a nonartifact spell".into(),
            M::AnyOf(v) => {
                let parts: Vec<String> =
                    v.iter().map(|x| self.mana_restriction_purpose(x)).collect();
                join_list(&parts, "or")
            }
            M::CastSpell(f) => {
                let n = self.noun(&f.0, Num::Many);
                let n = if n.contains("spell") {
                    n
                } else {
                    format!("{n} spells")
                };
                format!("to cast {n}")
            }
            M::ActivateAbilityOf(f) => {
                let n = self.noun(&f.0, Num::Many);
                format!("to activate abilities of {n}")
            }
            M::ClassLevel => "to gain a Class level".into(),
        }
    }

    fn roll_dice(&mut self, r: &crate::dice::DieRoll) -> String {
        let p = self.player(&r.who, Case::Subj);
        let die = format!("d{}", r.sides);
        let mut s = match &r.count {
            Value::Const(1) => format!("roll {}", with_article(&die)),
            other => {
                let (c, w) = self.counted(other, &die);
                format!("roll {c}{}", w.unwrap_or_default())
            }
        };
        if p != "you" {
            s = format!("{p} {}", third_person(&s));
        }
        if let Some(b) = &r.bonus {
            let b = self.value(b);
            s.push_str(&format!(" and add {b} to the result"));
        }
        match r.ignore {
            crate::dice::DiceIgnore::None => {}
            crate::dice::DiceIgnore::Lowest(1) => s.push_str(" and ignore the lower roll"),
            crate::dice::DiceIgnore::Lowest(n) => s.push_str(&format!(
                " and ignore the lowest {} rolls",
                number_word(n as i32)
            )),
            crate::dice::DiceIgnore::AllButHighest => {
                s.push_str(" and ignore all but the highest roll")
            }
        }
        for row in &r.table {
            let range = match row.hi {
                None => format!("{}+", row.lo),
                Some(h) if h == row.lo => format!("{h}"),
                Some(h) => format!("{}—{h}", row.lo),
            };
            let e = self.effect_sentences(&row.effect);
            s.push_str(&format!("\n{range} | {e}"));
        }
        if let Some(o) = &r.store_on {
            let o = self.sel(o, Case::Obj);
            s.push_str(&format!(". Store those results on {o}"));
        }
        s
    }

    fn flip_coins(&mut self, c: &crate::dice::CoinFlip) -> String {
        let p = self.player(&c.who, Case::Subj);
        let mut s = if c.until_lose {
            "flip a coin until you lose a flip".to_string()
        } else {
            match &c.count {
                Value::Const(1) => "flip a coin".into(),
                other => {
                    let (n, w) = self.counted(other, "coin");
                    format!("flip {n}{}", w.unwrap_or_default())
                }
            }
        };
        if p != "you" {
            s = format!("{p} {}", third_person(&s));
        }
        let mut add = |r: &mut Self, cond: &str, e: &Effect| {
            if !matches!(e, Effect::Noop) {
                let t = r.effect(e);
                s.push_str(&format!(". If {cond}, {t}"));
            }
        };
        add(self, "you win the flip", &c.on_win);
        add(self, "you lose the flip", &c.on_lose);
        add(self, "it comes up heads", &c.on_heads);
        add(self, "it comes up tails", &c.on_tails);
        s
    }

    fn exchange(&mut self, x: &crate::exchange::ExchangeSpec) -> String {
        use crate::exchange::ExchangeSpec as X;
        match x {
            X::LifeAndStat {
                player,
                what,
                power,
            } => {
                let p = self.player(player, Case::Poss);
                let w = self.sel(what, Case::Poss);
                let stat = if *power { "power" } else { "toughness" };
                format!("exchange {p} life total with {w} {stat}")
            }
            X::Stats {
                a,
                b,
                power,
                duration,
            } => {
                let a = self.sel(a, Case::Poss);
                let b = self.sel(b, Case::Poss);
                let stat = if *power { "power" } else { "toughness" };
                let d = self.duration(duration);
                join_words(&[format!("exchange {a} {stat} and {b} {stat}"), d])
            }
            X::Zones { player, a, b } => {
                let p = self.player(player, Case::Poss);
                format!("exchange {p} {} and {}", zone_word(*a), zone_word(*b))
            }
        }
    }

    /// Keyword actions (CR 701).
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn keyword_action(
        &mut self,
        action: KeywordAction,
        who: &PlayerRef,
        what: &Sel,
        n: &Value,
        subtype: Option<&str>,
        options: &[(String, Effect)],
    ) -> String {
        use KeywordAction as K;
        let has_what = !matches!(what, Sel::None);
        let num = |r: &mut Self| -> String {
            let (a, w) = r.amount(n);
            format!("{a}{}", w.unwrap_or_default())
        };
        // Subject-verb actions: "[permanent] explores".
        let subject_verb = |r: &mut Self, verb: &str, with_n: bool| -> String {
            let w = r.sel(what, Case::Subj);
            let mut s = format!("{w} {verb}");
            if with_n {
                s.push(' ');
                s.push_str(&num(r));
            }
            s
        };
        let vp = match action {
            K::Proliferate => self.repeated("proliferate", n),
            K::Investigate => self.repeated("investigate", n),
            K::Populate => self.repeated("populate", n),
            K::TimeTravel => self.repeated("time travel", n),
            K::Learn => self.repeated("learn", n),
            K::Recruit => self.repeated("recruit", n),
            K::Venture => self.repeated("venture into the dungeon", n),
            K::Forage => self.repeated("forage", n),
            K::Planeswalk => self.repeated("planeswalk", n),
            K::RollAttractions => "roll to visit your Attractions".into(),
            K::TheRingTemptsYou => return "the Ring tempts you".into(),
            K::OpenAttraction => self.repeated("open an Attraction", n),
            K::Assemble => "assemble a Contraption".into(),
            K::ManifestDread => self.repeated("manifest dread", n),
            // "It explores, then it explores again." / "It explores X times."
            K::Explore => {
                let base = subject_verb(self, "explores", false);
                return match n {
                    Value::Const(0) | Value::Const(1) => base,
                    Value::Const(2) => format!("{base}, then {{alt:it|~it}} explores again"),
                    other => {
                        let t = self.times(other);
                        format!("{base} {t}")
                    }
                };
            }
            K::Connive => {
                return subject_verb(self, "connives", !matches!(n, Value::Const(1)));
            }
            K::Endure => return subject_verb(self, "endures", true),
            K::Amass => match subtype {
                Some(st) => format!("amass {} {}", plural(st), num(self)),
                None => format!("amass {}", num(self)),
            },
            K::Bolster => format!("bolster {}", num(self)),
            K::Adapt => format!("adapt {}", num(self)),
            K::Monstrosity => format!("monstrosity {}", num(self)),
            K::Support => format!("support {}", num(self)),
            K::Fateseal => format!("fateseal {}", num(self)),
            K::Incubate => format!("incubate {}", num(self)),
            K::Discover => format!("discover {}", num(self)),
            K::CollectEvidence => format!("collect evidence {}", num(self)),
            K::Blight => format!("blight {}", num(self)),
            K::EmpowerJace => format!("empower Jace {}", num(self)),
            K::Earthbend if !has_what => format!("earthbend {}", num(self)),
            // CR 701.66a: "Earthbend N" targets a land you control by definition.
            K::Earthbend if matches!(what, Sel::Target(_)) => {
                let w = self.sel(what, Case::Obj);
                if w == "target land you control" {
                    format!("earthbend {}", num(self))
                } else {
                    format!("earthbend {w} {}", num(self))
                }
            }
            // CR 701.67a: "Waterbend [cost]", a generic mana cost.
            K::Waterbend if !has_what => match n {
                Value::Const(k) => format!("waterbend {{{k}}}"),
                _ => format!("waterbend {{{}}}", num(self)),
            },
            K::Clash => {
                self.after_clash = true;
                "clash with an opponent".into()
            }
            K::Manifest | K::Cloak if !has_what => {
                let p = self.possessive_for(who);
                let verb = if action == K::Manifest {
                    "manifest"
                } else {
                    "cloak"
                };
                match n {
                    Value::Const(1) => format!("{verb} the top card of {p} library"),
                    other => {
                        let (c, w) = self.counted(other, "card");
                        let c = c.split_once(' ').map(|x| x.0.to_string()).unwrap_or(c);
                        format!(
                            "{verb} the top {c} cards of {p} library{}",
                            w.unwrap_or_default()
                        )
                    }
                }
            }
            K::Vote | K::VillainousChoice => {
                let names: Vec<String> = options.iter().map(|(n, _)| n.clone()).collect();
                let mut s = if action == K::Vote {
                    format!("vote for {}", join_list(&names, "or"))
                } else {
                    "face a villainous choice —".into()
                };
                if action == K::VillainousChoice {
                    for (_, e) in options {
                        let t = self.effect_sentences(e);
                        s.push_str(&format!("\n• {t}"));
                    }
                }
                s
            }
            other => {
                let verb = match other {
                    K::Goad => "goad",
                    K::Manifest => "manifest",
                    K::Cloak => "cloak",
                    K::Detain => "detain",
                    K::Suspect => "suspect",
                    K::Exert => "exert",
                    K::Convert => "convert",
                    K::Harness => "harness",
                    K::Airbend => "airbend",
                    K::Earthbend => "earthbend",
                    K::Waterbend => "waterbend",
                    K::Behold => "behold",
                    K::Meld => "meld",
                    K::Abandon => "abandon",
                    K::SetInMotion => "set in motion",
                    K::Double => "double",
                    K::Triple => "triple",
                    K::Heal => "heal",
                    K::Exchange => "exchange",
                    K::Seek => "seek",
                    K::Conjure => "conjure",
                    K::Specialize => "specialize",
                    K::Perpetually => "perpetually",
                    K::Mutate => "mutate",
                    _ => {
                        return self.gap(format!("keyword action {other:?}"));
                    }
                };
                // "Behold a Kithkin", "behold two Elves": chosen from those the filter
                // describes.
                if let (K::Behold, Sel::All(f), Value::Const(c)) = (other, what, n) {
                    let w = if *c == 1 {
                        self.noun_det(f, Det::A)
                    } else {
                        format!("{} {}", number_word(*c), self.noun(f, Num::Many))
                    };
                    return self.with_subject(who, &format!("behold {w}"), false);
                }
                if has_what {
                    let w = self.sel(what, Case::Obj);
                    if matches!(n, Value::Const(1)) || matches!(n, Value::Const(0)) {
                        format!("{verb} {w}")
                    } else {
                        let a = num(self);
                        format!("{verb} {w} {a}")
                    }
                } else {
                    let a = num(self);
                    format!("{verb} {a}")
                }
            }
        };
        self.with_subject(who, &vp, false)
    }

    fn repeated(&mut self, verb: &str, n: &Value) -> String {
        match n {
            Value::Const(1) | Value::Const(0) => verb.to_string(),
            other => {
                let t = self.times(other);
                format!("{verb} {t}")
            }
        }
    }
}

#[derive(Default)]
struct Becomes {
    pt: Option<(Option<Value>, Option<Value>)>,
    colors: Option<String>,
    supertypes: Vec<String>,
    subtypes: Vec<String>,
    add_types: Vec<CardType>,
    additive: bool,
    land_type: bool,
    name: Option<String>,
}

impl Becomes {
    fn is_empty(&self) -> bool {
        self.pt.is_none()
            && self.colors.is_none()
            && self.supertypes.is_empty()
            && self.subtypes.is_empty()
            && self.add_types.is_empty()
            && self.name.is_none()
    }

    fn render(
        &self,
        r: &mut Renderer,
        keywords: &[String],
        abilities: &[String],
        gains: bool,
    ) -> Option<(String, String)> {
        if self.is_empty() {
            return None;
        }
        // Base P/T only: "has base power and toughness 1/1".
        let only_pt = self.colors.is_none()
            && self.supertypes.is_empty()
            && self.subtypes.is_empty()
            && self.add_types.is_empty()
            && self.name.is_none();
        let pt = self.pt.as_ref().map(|(p, t)| {
            let ps = p.as_ref().map(|v| r.value(v)).unwrap_or_else(|| "*".into());
            let ts = t.as_ref().map(|v| r.value(v)).unwrap_or_else(|| "*".into());
            format!("{ps}/{ts}")
        });
        let mut grants: Vec<String> = keywords.to_vec();
        grants.extend(abilities.iter().cloned());
        let with = if grants.is_empty() {
            String::new()
        } else {
            format!(" with {}", join_list(&grants, "and"))
        };
        if only_pt {
            let has = if gains { "has" } else { "has" };
            let mut s = match self.pt.as_ref() {
                Some((None, Some(t))) => format!("{has} base toughness {}", r.value(t)),
                Some((Some(p), None)) => format!("{has} base power {}", r.value(p)),
                _ => format!("{has} base power and toughness {}", pt.unwrap_or_default()),
            };
            if !grants.is_empty() {
                let g = if gains { "gains" } else { "has" };
                s.push_str(&format!(" and {g} {}", join_list(&grants, "and")));
            }
            return Some((s, String::new()));
        }
        if let Some(n) = &self.name {
            if self.pt.is_none() && self.add_types.is_empty() && self.subtypes.is_empty() {
                return Some((format!("is named {n}"), String::new()));
            }
        }
        let mut words: Vec<String> = Vec::new();
        if let Some(pt) = pt {
            words.push(pt);
        }
        words.extend(self.supertypes.iter().cloned());
        if let Some(c) = &self.colors {
            words.push(c.clone());
        }
        words.extend(self.subtypes.iter().cloned());
        let mut types = self.add_types.clone();
        types.sort_by_key(|t| match t {
            CardType::Enchantment => 0,
            CardType::Artifact => 1,
            CardType::Land => 2,
            CardType::Creature => 3,
            _ => 4,
        });
        words.extend(types.iter().map(|t| t.word().to_string()));
        let phrase = words.join(" ");
        let is_adjective_only =
            self.add_types.is_empty() && self.subtypes.is_empty() && self.pt.is_none();
        let mut s = if is_adjective_only {
            format!("is {phrase}")
        } else {
            format!("is {}", with_article(&phrase))
        };
        if gains {
            s = s.replacen("is ", "becomes ", 1);
        }
        if let Some(n) = &self.name {
            s.push_str(&format!(" named {n}"));
        }
        s.push_str(&with);
        let mut still = String::new();
        // Adding a supertype ("is snow", "is legendary") never removes anything.
        let only_supertypes =
            self.add_types.is_empty() && self.subtypes.is_empty() && !self.supertypes.is_empty();
        if self.additive && !self.land_type && !only_supertypes {
            // An effect that adds types keeps the old ones: cards say "It's still a land"
            // when they know what the object was, else "in addition to its other types".
            let known: Vec<CardType> = r
                .subject_types
                .iter()
                .copied()
                .filter(|t| !self.add_types.contains(t))
                .collect();
            if r.subject_types.is_empty() || self.add_types.is_empty() {
                s.push_str(" in addition to its other types");
            } else if !known.is_empty() {
                let w: Vec<String> = known.iter().map(|t| t.word().to_string()).collect();
                still = format!(". It's still {}", with_article(&w.join(" ")));
            }
        }
        Some((s, still))
    }
}

/// A filter without its "you control" part: a player can sacrifice only permanents they
/// control (CR 701.21a), so "sacrifice a creature" needs no "you control".
pub(crate) fn strip_controller(f: &Filter) -> Filter {
    match f {
        Filter::ControlledBy(_) => Filter::Any,
        Filter::And(v) => Filter::and(
            v.iter()
                .filter(|x| !matches!(x, Filter::ControlledBy(_)))
                .cloned()
                .collect(),
        ),
        other => other.clone(),
    }
}

/// Where the granted keywords and abilities go among the parts of a verb phrase.
const GRANTS: &str = "\u{1}grants";

fn unreachable_player() -> PlayerRef {
    PlayerRef::You
}

fn unreachable_text() -> String {
    String::new()
}

fn strip_chooser(s: &Sel) -> Sel {
    match s {
        Sel::Choose {
            filter,
            count,
            up_to,
            store,
            ..
        } => Sel::Choose {
            chooser: PlayerRef::You,
            filter: filter.clone(),
            count: count.clone(),
            up_to: *up_to,
            store: *store,
        },
        other => other.clone(),
    }
}

/// Whether a selection is plural ("them", "their owners' hands").
pub(crate) fn is_plural_sel(s: &Sel) -> bool {
    match s {
        Sel::All(_) | Sel::TriggerObjects | Sel::Linked | Sel::Union(_) | Sel::AllTargets => true,
        Sel::Choose { count, .. } => !matches!(count, Value::Const(1)),
        Sel::TopOfLibrary(_, n) => !matches!(n, Value::Const(1)),
        _ => false,
    }
}

fn until_event(u: &UntilEvent) -> String {
    match u {
        UntilEvent::SourceLeavesBattlefield => "~ leaves the battlefield".into(),
        UntilEvent::OpponentBecomesMonarch => "an opponent becomes the monarch".into(),
    }
}

/// Keeps only the last of repeated identical "where X is ..." clauses in a sentence
/// sequence: "You gain X life and each opponent loses X life, where X is ...".
pub(crate) fn dedupe_where(s: &str) -> String {
    let mut out = s.to_string();
    loop {
        let mut changed = false;
        let mut start = 0;
        while let Some(i) = out[start..].find(", where X is ") {
            let i = start + i;
            let end = out[i..].find(". ").map(|e| i + e).unwrap_or(out.len());
            let clause = out[i..end].to_string();
            if out[end..].contains(&clause) {
                out.replace_range(i..end, "");
                changed = true;
                break;
            }
            start = end;
        }
        if !changed {
            return out;
        }
    }
}

/// Joins non-empty phrases with spaces.
pub(crate) fn join_words(parts: &[String]) -> String {
    parts
        .iter()
        .filter(|p| !p.is_empty())
        .cloned()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Lowercases the first letter.
pub(crate) fn lower_first(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_lowercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

/// Third-person singular of a verb phrase in base form ("draw a card" → "draws a card").
pub(crate) fn third_person(vp: &str) -> String {
    let (verb, rest) = match vp.split_once(' ') {
        Some((v, r)) => (v, format!(" {r}")),
        None => (vp, String::new()),
    };
    let v = match verb {
        "have" => "has".to_string(),
        "do" => "does".to_string(),
        "may" | "can't" | "can" => verb.to_string(),
        v if v.ends_with('s') || v.ends_with("sh") || v.ends_with("ch") || v.ends_with('x') => {
            format!("{v}es")
        }
        v if v.ends_with('y') && !v.ends_with("ay") && !v.ends_with("ey") => {
            format!("{}ies", &v[..v.len() - 1])
        }
        v => format!("{v}s"),
    };
    format!("{v}{rest}")
}
