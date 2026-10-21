//! Effects ([`Effect`]) → imperative sentences ("Destroy target creature.", "You gain 3
//! life.", "Target player draws two cards.").

use super::nouns::Det;
use super::players::Case;
use super::*;

fn same_player(a: &PlayerRef, b: &PlayerRef) -> bool {
    format!("{a:?}") == format!("{b:?}")
}

fn same_sel(a: &Sel, b: &Sel) -> bool {
    // Two instructions that each choose an object choose separately ("Put a deathtouch
    // counter on either of them. Then put a menace counter on either of them.").
    if matches!(a, Sel::Choose { .. }) || matches!(b, Sel::Choose { .. }) {
        return false;
    }
    format!("{a:?}") == format!("{b:?}")
}

fn same_duration(a: &Duration, b: &Duration) -> bool {
    format!("{a:?}") == format!("{b:?}")
}

/// "~ has base power and toughness 4/2 until end of turn and gains first strike until end
/// of turn": the duration said for each part, as well as once at the end.
fn duration_for_each_part(s: String) -> String {
    let d = " until end of turn";
    match (s.strip_suffix(d), s.find(" and gains ")) {
        (Some(head), Some(k))
            if head.contains(" has base power and toughness ")
                && !s.contains('{')
                && head[..k].matches(" and ").count() == 1 =>
        {
            let (a, b) = head.split_at(k);
            format!("{{alt:{s}|{a}{d}{b}{d}}}")
        }
        _ => s,
    }
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
                        let s = super::graveyard_this_way::chosen_cards(self, sel, s);
                        return Some((who.clone(), format!("discard {s}"), false));
                    }
                }
                // "... for each card discarded this way".
                self.it_done = Some("cards discarded this way".into());
                let noun = self.card_noun(filter);
                // "That player discards all cards with that name": as many as there are
                // in their hand (each that matches).
                if matches!(n, Value::HandSize(p) if same_player(p, who))
                    && !matches!(filter, Filter::Any)
                    && !*random
                {
                    return Some((who.clone(), format!("discard all {}", plural(&noun)), false));
                }
                let (c, w) = self.counted(n, &noun);
                let r = if *random { " at random" } else { "" };
                (
                    who.clone(),
                    format!("discard {c}{r}{}", w.unwrap_or_default()),
                    false,
                )
            }
            Effect::DiscardHand { who } => {
                self.it_done = Some("cards discarded this way".into());
                let p = self.possessive_for(who);
                (who.clone(), format!("discard {p} hand"), false)
            }
            // "Target player mills half their library, rounded up."
            Effect::Mill {
                who,
                n: Value::Div(v, 2, up),
            } if matches!(v.as_ref(), Value::LibrarySize(p) if same_player(p, who)) => {
                let p = self.possessive_for(who);
                let r = if *up { "up" } else { "down" };
                (
                    who.clone(),
                    format!("mill half {p} library, rounded {r}"),
                    false,
                )
            }
            Effect::Mill { who, n } => {
                self.milled = true;
                let (c, w) = self.counted(n, "card");
                (
                    who.clone(),
                    format!("mill {c}{}", w.unwrap_or_default()),
                    false,
                )
            }
            // "You gain life equal to the damage dealt, but not more life than ..." (see
            // `gain_life_equal_to_damage_capped`).
            Effect::GainLife { who, n }
                if crate::oracle::patterns::gain_life_equal_to_damage_capped::capped_text(n)
                    .is_some() =>
            {
                let t = crate::oracle::patterns::gain_life_equal_to_damage_capped::capped_text(n)
                    .unwrap_or_default();
                (who.clone(), t, true)
            }
            Effect::GainLife { who, n } => {
                let (a, w) = self.amount(n);
                (
                    who.clone(),
                    format!("gain {a} life{}", w.unwrap_or_default()),
                    true,
                )
            }
            // "That player loses half their life, rounded up."
            Effect::LoseLife {
                who,
                n: Value::Div(v, 2, up),
            } if matches!(v.as_ref(), Value::LifeTotal(p) if same_player(p, who)) => {
                let p = self.possessive_for(who);
                let r = if *up { "up" } else { "down" };
                (
                    who.clone(),
                    format!("lose half {p} life, rounded {r}"),
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
            // "Each opponent sacrifices a creature for each card discarded this way",
            // "sacrifice half the non-Demon permanents you control, rounded up".
            Effect::Sacrifice { who, filter, count }
                if !matches!(
                    count,
                    Value::Const(_) | Value::X | Value::EventAmount | Value::Prev | Value::Var(_)
                ) && self.stored_x(count).is_none() =>
            {
                let f = strip_controller(filter);
                let v = self.value(count);
                let s = if let Some(rest) = v.strip_prefix("the number of ") {
                    let one = self.noun_det(&f, Det::A);
                    format!("sacrifice {one} for each {rest}")
                } else if let Some(rest) = v.strip_prefix("half the number of ") {
                    format!("sacrifice half the {rest}")
                } else {
                    let many = self.noun(&f, Num::Many);
                    format!("sacrifice X {many}, where X is {v}")
                };
                (who.clone(), s, false)
            }
            Effect::Sacrifice { who, filter, count } => {
                let det = self.det_for(count);
                // "a creature they control with the greatest power among creatures they
                // control": the objects compared are said with whose they are.
                let extreme = matches!(filter, Filter::And(v) if v.iter().any(|x|
                    matches!(x, Filter::ValueCmp(_, _, b) if matches!(b.as_ref(), Value::Extreme(..)))));
                let n = if extreme {
                    self.noun_det(filter, det.clone())
                } else {
                    self.noun_det(&strip_controller(filter), det.clone())
                };
                // "sacrifice a creature other than ~" says "another creature".
                if let (Filter::And(v), Value::Const(1)) = (filter, count) {
                    if v.iter().any(|x| matches!(x, Filter::Other)) && !extreme {
                        let rest = Filter::and(
                            v.iter()
                                .filter(|x| !matches!(x, Filter::Other))
                                .cloned()
                                .collect(),
                        );
                        let r = self.noun_det(&strip_controller(&rest), det);
                        let m = self.me();
                        if !r.contains(['{', '|']) {
                            return Some((
                                who.clone(),
                                format!("sacrifice {{alt:{n}|{r} other than {m}}}"),
                                false,
                            ));
                        }
                    }
                }
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
                        Value::X => format!("X {sym}"),
                        other
                            if Self::is_simple(other)
                                || matches!(other, Value::EventAmount | Value::Prev) =>
                        {
                            let v = self.value(other);
                            format!("{v} {sym}")
                        }
                        // "you get an amount of {E} equal to its mana value".
                        other => {
                            let v = self.value(other);
                            format!(
                                "{{alt:an amount of {sym} equal to {v}|X {sym}, where X is {v}}}"
                            )
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
            Effect::SearchCards(spec) => (spec.who.clone(), self.search_cards(spec), false),
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
    pub(super) fn possessive_for(&mut self, who: &PlayerRef) -> String {
        match who {
            PlayerRef::You => "your".into(),
            _ => "their".into(),
        }
    }

    /// "card", "creature card", "nonland card".
    pub(super) fn card_noun(&mut self, f: &Filter) -> String {
        if matches!(f, Filter::Any) {
            return "card".into();
        }
        let n = self.card_noun_plain(f);
        // "a land or double-faced card": the shared qualities said once, or for each
        // alternative (see `nouns::factor_or`).
        let g = super::nouns::factor_or(f);
        if format!("{g:?}") == format!("{f:?}") {
            return n;
        }
        let m = self.card_noun_plain(&g);
        if m == n {
            n
        } else {
            format!("{{alt:{n}|{m}}}")
        }
    }

    fn card_noun_plain(&mut self, f: &Filter) -> String {
        let n = self.noun(&f.clone().in_zone(ZoneKind::Hand), Num::One);
        // The zone is implied.
        n.replacen(" in a hand", "", 1)
    }

    /// "[subject] [vp]": "you" is dropped for imperative verbs.
    pub(super) fn with_subject(&mut self, who: &PlayerRef, vp: &str, keep_you: bool) -> String {
        if matches!(who, PlayerRef::You) {
            if keep_you {
                return format!("you {vp}");
            }
            return vp.to_string();
        }
        // "You and that player each draw that many cards."
        if let PlayerRef::Each(PlayerFilter::Or(v)) = who {
            if let [PlayerFilter::You, PlayerFilter::Ref(r)] = v.as_slice() {
                let p = self.player(r, Case::Subj);
                return format!("you and {p} each {vp}");
            }
        }
        let s = self.player(who, Case::Subj);
        // "Target opponent reveals their hand. ... They discard that card."
        if s == "that player" && matches!(who, PlayerRef::Target(_)) {
            return format!("{{alt:that player|they}} {}", third_person(vp));
        }
        format!("{s} {}", third_person(vp))
    }

    /// "You may choose new targets for the copy" (CR 707.10c): the player who copies
    /// the spell may choose; inside an instruction another player follows ("that player
    /// may copy ~"), that player, so the clause shares the instruction's subject.
    fn choose_new_targets(&self, copies: &str) -> String {
        if self.in_as_player {
            format!(" and may choose new targets for {copies}")
        } else {
            format!(". You may choose new targets for {copies}")
        }
    }

    /// An effect as text (sentences separated by ". ").
    pub(crate) fn effect(&mut self, e: &Effect) -> String {
        // Who performed the instruction an "if they do" after it refers to.
        let actor_other = match e {
            Effect::May { who, .. }
            | Effect::PayOptional { who, .. }
            | Effect::AsPlayer { who, .. }
            | Effect::Sacrifice { who, .. }
            | Effect::Discard { who, .. } => Some(!matches!(who, PlayerRef::You)),
            Effect::Seq(_) | Effect::If { .. } | Effect::Noop => None,
            _ => Some(false),
        };
        let s = self.effect_inner(e);
        if let Some(x) = actor_other {
            self.last_actor_other = x;
        }
        s
    }

    fn effect_inner(&mut self, e: &Effect) -> String {
        self.new_clause();
        // Instructions worded as a whole (`effect_terms.rs`, `more_terms.rs`).
        if let Some(s) = self.effect_terms(e) {
            return s;
        }
        if let Some(s) = self.effect_more(e) {
            return s;
        }
        // "For each {B}{B} spent to cast it, each opponent discards a card": one action on
        // that many, said as the instruction for each.
        if let Some(each) = per_mana_spent(e) {
            return self.effect(&each);
        }
        // "You may cast that card" (`play_terms.rs`).
        if let Some(s) = self.cast_only_permission(e) {
            return s;
        }
        if let Some(s) = self.free_if_permission(e) {
            return s;
        }
        // "If it's a land card, you may ... If you don't, ..." (`outcomes.rs`).
        if let Some(s) = self.if_or_else_part(e) {
            return s;
        }
        // "Target opponent loses life equal to the number of creatures they control": the
        // target is introduced by the subject, before the amount mentions it.
        if let Effect::LoseLife {
            who: who @ PlayerRef::Target(i),
            n,
        }
        | Effect::GainLife {
            who: who @ PlayerRef::Target(i),
            n,
        } = e
        {
            if !matches!(n, Value::Const(_))
                && !self.introduced.get(*i as usize).copied().unwrap_or(true)
            {
                let subj = self.player(who, Case::Subj);
                let saved = self.they_target.replace(*i);
                let r = self.actor_vp(e);
                self.they_target = saved;
                if let Some((_, vp, _)) = r {
                    return format!("{subj} {}", third_person(&vp));
                }
            }
        }
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
            // "You may play an additional land this turn": the permission is the choice
            // (playing a land is optional anyway, CR 305.1), so the "may" is said once.
            Effect::May { who, effect }
                if same_player(who, &PlayerRef::You)
                    && matches!(
                        effect.as_ref(),
                        Effect::AddPlayerEffect {
                            who: PlayerRef::You,
                            effect: PlayerModification::AdditionalLandPlays(_),
                            duration: Duration::ThisTurn,
                        }
                    ) =>
            {
                self.effect(effect)
            }
            // "When ~ dies, you may cast it from your graveyard ...": a permission to cast
            // is a choice already (CR 601.1a, 608.2d), so the "may" is said once.
            Effect::May { who, effect }
                if same_player(who, &PlayerRef::You)
                    && matches!(
                        effect.as_ref(),
                        Effect::WithPlayTerms { effect: g, .. }
                            if matches!(g.as_ref(), Effect::GrantPlayPermission { who: PlayerRef::You, .. })
                    ) =>
            {
                self.effect(effect)
            }
            // "You may repeat this process any number of times." (CR 608.2c): each time
            // it's done, you may again.
            Effect::May {
                who: PlayerRef::You,
                effect,
            } if matches!(effect.as_ref(), Effect::RepeatThisProcess) => {
                "you may repeat this process {opt:any number of times}".into()
            }
            Effect::May { who, effect } => self.may(who, effect),
            // "Then each player who searched their library this way shuffles."
            Effect::ForEachPlayer {
                who: PlayerRef::Var(v),
                effect,
            } if *v == crate::oracle::patterns::iteration_grammar::SEARCHERS
                && matches!(
                    effect.as_ref(),
                    Effect::Shuffle {
                        who: PlayerRef::Iterated
                    }
                ) =>
            {
                "then each player who searched their library this way shuffles".to_string()
            }
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
            // "Double the power of each creature you control until end of turn" (CR
            // 701.10b).
            Effect::ForEach { sel, var, effect }
                if tail_parts::double_pt_parts(*var, effect).is_some() =>
            {
                self.double_pt(sel, *var, effect).unwrap_or_default()
            }
            // "If it doesn't have suspend, it gains suspend."
            Effect::ForEach { sel, .. }
                if crate::oracle::patterns::r702_062_gains_suspend::is_gains_suspend(e) =>
            {
                let s = self.sel(sel, Case::Subj);
                format!("if {s} doesn't have suspend, it gains suspend")
            }
            // "Put each card exiled with ~ onto the battlefield under its owner's
            // control": each card moved on its own (to its own owner's control).
            Effect::ForEach {
                sel: sel @ Sel::All(_),
                var,
                effect,
            } if matches!(effect.as_ref(), Effect::Move { what: Sel::Var(v), .. } if v == var) => {
                let Effect::Move { to, .. } = effect.as_ref() else {
                    return self.gap("each linked card");
                };
                let mut to = to.clone();
                let owner_each = matches!(&to.controller, Some(PlayerRef::OwnerOf(o)) if matches!(o.as_ref(), Sel::Var(v) if v == var));
                if owner_each {
                    to.controller = None;
                }
                let mut s = self.move_effect(sel, &to);
                if owner_each && to.zone == ZoneKind::Battlefield && !s.contains(" under ") {
                    s.push_str(" under its owner's control");
                }
                s
            }
            // "Return the exiled card to the battlefield": each card linked to this object.
            // "Put each card exiled with ~ into its owner's hand" (an object that exiles
            // one card says "the exiled card").
            Effect::ForEach {
                sel: Sel::Linked,
                var,
                effect,
            } => {
                let all = Sel::All(Filter::And(vec![
                    Filter::In(Box::new(Sel::Linked)),
                    Filter::InZone(ZoneKind::Exile),
                ]));
                self.var_defs.push((*var, all, false));
                self.effect(effect)
            }
            Effect::ForEach {
                sel: Sel::CreatorLinked,
                var,
                effect,
            } => {
                self.var_defs.push((*var, Sel::CreatorLinked, false));
                self.effect(effect)
            }
            // "Choose one of them.": a choice stored for later instructions, nothing done
            // to each chosen object now.
            Effect::ForEach {
                sel: sel @ Sel::Choose { store: Some(_), .. },
                effect,
                ..
            } if matches!(effect.as_ref(), Effect::Noop) => {
                let s = self.sel(sel, Case::Obj);
                format!("choose {s}")
            }
            // "Target unblocked attacking creature becomes blocked": an instruction done
            // for each object of a single target is done to the target.
            Effect::ForEach {
                sel: Sel::Target(i),
                var,
                effect,
            } if self
                .targets
                .get(*i as usize)
                .is_some_and(|t| matches!(t.max, Value::Const(1))) =>
            {
                self.var_defs.push((*var, Sel::Target(*i), false));
                let saved = self.each_target.replace(*i);
                let s = self.effect(effect);
                self.each_target = saved;
                s
            }
            Effect::ForEach { sel, var, effect } => {
                let s = match sel {
                    Sel::All(f) => self.for_each_noun(f),
                    // "for each creature destroyed this way", "for each card exiled this
                    // way".
                    Sel::Matching(inner, f) if self.for_each_this_way(inner).is_some() => {
                        let verb = self.for_each_this_way(inner).unwrap_or_default();
                        let n = self.noun(f, Num::One);
                        format!("{n} {verb} this way")
                    }
                    // "for each of them", "for each of those creatures", "for each of up to
                    // three target creatures".
                    // "Look at the top five cards of your library. For each card, ..."
                    Sel::Var(vars::DUG) => "card".into(),
                    other => {
                        let s = self.sel(other, Case::Obj);
                        if s == "them" {
                            "of {alt:them|those creatures|those permanents|those cards|those tokens}".into()
                        } else if matches!(other, Sel::Target(_)) {
                            format!("of {s}")
                        } else {
                            s
                        }
                    }
                };
                // "If you do, it becomes plotted": done for the one object named "it".
                if matches!(s.as_str(), "it" | "~it" | "~") {
                    self.var_defs.push((*var, sel.clone(), false));
                    let saved = self.for_each_subject.replace(s.clone());
                    let t = self.effect(effect);
                    self.for_each_subject = saved;
                    return t;
                }
                let inner = self.effect(effect);
                // "Each creature deals damage to itself equal to its power": each object
                // is the source, all at once (CR 608.2f).
                if let (Sel::All(f), Effect::DealDamage { source: Sel::Var(v), .. }) =
                    (sel, effect.as_ref())
                {
                    if v == var {
                        if let Some(rest) = inner.strip_prefix("it deals ") {
                            let one = self.noun(f, Num::One);
                            return format!(
                                "{{alt:for each {s}, {inner}|each {one} deals {rest}}}"
                            );
                        }
                    }
                }
                // "Each creature you control attacking a player gets +1/+0 until end of
                // turn for each creature that player controls": each object is modified
                // on its own.
                if let (Sel::All(f), Effect::Modify { what: Sel::Var(v), .. }) =
                    (sel, effect.as_ref())
                {
                    if v == var {
                        if let Some(rest) = inner.strip_prefix("it gets ") {
                            let one = self.noun(f, Num::One);
                            return format!(
                                "{{alt:for each {s}, {inner}|each {one} gets {rest}}}"
                            );
                        }
                    }
                }
                format!("for each {s}, {inner}")
            }
            // "Starting with you, each player chooses a creature."
            Effect::InTurnOrder {
                first,
                who,
                effect,
                direction,
            } => {
                let start = match (first, direction) {
                    (TurnOrderStart::You, SeatSide::Left) => "you",
                    (TurnOrderStart::You, SeatSide::Chosen) => {
                        "you and proceeding in the chosen direction"
                    }
                    (TurnOrderStart::You, SeatSide::Right) => "you and proceeding to the right",
                    (TurnOrderStart::NextOpponent, _) => "the next opponent in turn order",
                };
                let w = self.player_filter_noun(who, Num::One);
                // "Join forces — Starting with you, each player may pay any amount of mana."
                if let Effect::AsPlayer { effect: inner, .. } = effect.as_ref() {
                    if matches!(inner.as_ref(), Effect::Custom(n) if n == crate::kw::join_forces::PAY_ANY_AMOUNT)
                    {
                        return format!("starting with {start}, each {w} may pay any amount of mana");
                    }
                    // "Starting with you, each player may pay any amount of life" (CR
                    // 119.4b: any amount can be 0).
                    if matches!(inner.as_ref(), Effect::Custom(n) if n == crate::kw::loop_totals::PAY_ANY_LIFE_TALLIED)
                    {
                        return format!("starting with {start}, each {w} may pay any amount of life");
                    }
                }
                // "each player chooses a creature" / "each player may choose an artifact".
                if let Effect::Store {
                    var,
                    sel: Sel::Union(v),
                } = effect.as_ref()
                {
                    if let [Sel::Var(a), Sel::Choose {
                        chooser: PlayerRef::Iterated,
                        filter,
                        count: Value::Const(n),
                        up_to,
                        ..
                    }] = v.as_slice()
                    {
                        if a == var {
                            let (verb, det) = match (*n, *up_to) {
                                (1, false) => ("chooses", Det::A),
                                (1, true) => ("may choose", Det::A),
                                (n, true) => ("chooses", Det::UpTo(number_word(n as i32))),
                                (n, false) => ("chooses", Det::Count(number_word(n as i32))),
                            };
                            let saved = std::mem::replace(&mut self.in_as_player, true);
                            let o = self.noun_det(filter, det);
                            self.in_as_player = saved;
                            return format!("starting with {start}, each {w} {verb} {o}");
                        }
                    }
                }
                let inner = self.effect(effect);
                format!("starting with {start}, each {w}: {inner}")
            }
            // "Each player who controls a multicolored creature draws a card."
            Effect::ForEachPlayer { who, effect }
                if matches!(effect.as_ref(), Effect::AsPlayer { who: PlayerRef::Iterated, effect: inner }
                    if matches!(inner.as_ref(), Effect::If { otherwise, cond: Condition::Exists(_) | Condition::Compare(Value::Count(_), _, _), .. }
                        if matches!(otherwise.as_ref(), Effect::Noop))) =>
            {
                let Effect::AsPlayer { effect: inner, .. } = effect.as_ref() else {
                    return self.gap("each player who");
                };
                let Effect::If { cond, then, .. } = inner.as_ref() else {
                    return self.gap("each player who");
                };
                let w = self.player(who, Case::Subj);
                let saved = std::mem::replace(&mut self.in_as_player, true);
                let c = self.condition(cond);
                let t = self.effect(then);
                self.in_as_player = saved;
                match (c.strip_prefix("you "), t.strip_prefix("you ").or(Some(t.as_str()))) {
                    (Some(c), Some(t)) if !c.contains('{') && !t.contains('.') => {
                        let c = format!(" {} ", third_person(c))
                            .replace(" your ", " their ")
                            .replace(" you ", " they ");
                        let t = format!(" {} ", third_person(t))
                            .replace(" your ", " their ")
                            .replace(" you ", " they ");
                        format!("{w} who {} {}", c.trim(), t.trim())
                    }
                    _ => self.gap("each player who"),
                }
            }
            // "Shuffle your graveyard into your library."
            Effect::ForEachPlayer {
                who: PlayerRef::You,
                effect,
            } if matches!(effect.as_ref(), Effect::ShuffleIntoLibrary {
                    what: Sel::All(Filter::And(f)),
                    library: PlayerRef::Iterated,
                } if matches!(f.as_slice(), [Filter::InZone(_), Filter::OwnedBy(PlayerRel::Iterated)])) =>
            {
                let Effect::ShuffleIntoLibrary {
                    what: Sel::All(Filter::And(f)),
                    ..
                } = effect.as_ref()
                else {
                    return self.gap("shuffle your zone");
                };
                let Some(Filter::InZone(z)) = f.first() else {
                    return self.gap("shuffle your zone");
                };
                format!("shuffle your {} into your library", zone_word(*z))
            }
            // "You lose half your life": performed once, by you.
            Effect::ForEachPlayer {
                who: PlayerRef::You,
                effect,
            } if you_alone::iterated_as_you(effect).is_some() => {
                let e = you_alone::iterated_as_you(effect).unwrap_or_default();
                self.effect(&e)
            }
            // "~ deals damage to each opponent equal to the number of lands that player
            // controls": the amount for each player (dealt one player at a time).
            Effect::ForEachPlayer { who, effect }
                if matches!(effect.as_ref(), Effect::DealDamage { source: Sel::This, to: Sel::Players(PlayerRef::Iterated), .. })
                    && matches!(who, PlayerRef::EachOpponent | PlayerRef::EachPlayer) =>
            {
                let inner = self.effect(effect);
                let w = self.player(who, Case::Subj);
                let Effect::DealDamage { source, amount, .. } = effect.as_ref() else {
                    return format!("for {w}, {inner}");
                };
                let mut all = self.effect(&Effect::DealDamage {
                    source: source.clone(),
                    amount: amount.clone(),
                    to: Sel::Players(who.clone()),
                });
                // "the number of tapped creatures that opponent controls".
                if matches!(who, PlayerRef::EachOpponent) {
                    all = all.replace(
                        "{alt:that player controls|they control}",
                        "{alt:that player controls|that opponent controls|they control}",
                    );
                }
                format!("{{alt:for {w}, {inner}|{all}}}")
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
            Effect::AsPlayer { who, effect } if self.choose_and_pay_each(effect).is_some() => {
                let w = self.player(who, Case::Subj);
                let s = self.choose_and_pay_each(effect).unwrap_or_default();
                format!("{w} {s}")
            }
            Effect::AsPlayer { who, effect } => {
                let w = self.player(who, Case::Subj);
                // Performed as that player: "you" in the instruction is that player
                // ("target player loses 4 life").
                let saved = std::mem::replace(&mut self.in_as_player, true);
                let inner = self.effect(effect);
                self.in_as_player = saved;
                let inner = inner.strip_prefix("you ").unwrap_or(&inner);
                // "loses 3 life unless they discard a card" / "... unless that player
                // discards a card": the same player performs the alternative.
                let inner = match inner.split_once(" unless you ") {
                    Some((head, alt)) => {
                        let alt = format!(" {alt} ")
                            .replace(" your ", " their ")
                            .replace(" to you ", " to them ");
                        let alt = alt.trim();
                        let third: Vec<String> = alt
                            .split(" or ")
                            .enumerate()
                            .map(|(i, a)| {
                                let verb = a.split(' ').next().unwrap_or("");
                                let is_verb = matches!(
                                    verb,
                                    "sacrifice"
                                        | "discard"
                                        | "pay"
                                        | "have"
                                        | "exile"
                                        | "return"
                                        | "remove"
                                        | "put"
                                        | "tap"
                                        | "reveal"
                                );
                                if i == 0 || is_verb {
                                    third_person(a)
                                } else {
                                    a.to_string()
                                }
                            })
                            .collect();
                        format!(
                            "{head} unless {{alt:they {alt}|that player {}}}",
                            third.join(" or ")
                        )
                    }
                    None => inner.to_string(),
                };
                let inner = format!(" {inner} ")
                    .replace(" your ", " their ")
                    .replace(" you ", " they ")
                    .replace(" you.", " them.")
                    // "Target players each mill a card and lose 1 life."
                    .replace(". they ", " and ");
                let s = format!("{w} {}", third_person(inner.trim()));
                if self.in_as_player {
                    s
                } else {
                    s.replace(super::KEEP_YOU, "")
                }
            }
            // "... If [condition], repeat this process." (CR 608.2c)
            Effect::RepeatProcess { body } => self.effect(body),
            Effect::RepeatThisProcess => "repeat this process".into(),
            Effect::Repeat { times, effect } => {
                // "For each card drawn this way, discard a card unless you sacrifice a
                // permanent": a choice made once for each (what's counted is named
                // before the instruction that's repeated).
                if matches!(effect.as_ref(), Effect::PayOptional { .. })
                    && matches!(times, Value::Count(_))
                {
                    let v = self.value(times);
                    if let Some(rest) = v.strip_prefix("the number of ") {
                        let inner = self.effect(effect);
                        return format!("for each {rest}, {inner}");
                    }
                }
                let inner = self.effect(effect);
                // "for each {U}{U} spent to cast it, draw a card".
                let spent = |v: &Value| match v {
                    Value::Custom(c) => c.strip_prefix("mana_spent_of:").map(|l| (l.to_string(), 1)),
                    _ => None,
                };
                let per = match times {
                    Value::Div(v, n, false) => spent(v).map(|(l, _)| (l, *n)),
                    v => spent(v),
                };
                if let Some((l, n)) = per {
                    let syms = format!("{{{l}}}").repeat(n.max(1) as usize);
                    return format!("for each {syms} spent to cast {{alt:it|~}}, {inner}");
                }
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
                let list = join_list(&kinds, "or");
                // "a +1/+1, first strike, vigilance, or menace counter": one noun.
                let bare: Vec<String> = kinds
                    .iter()
                    .map(|k| {
                        let k = k.strip_suffix(" counter").unwrap_or(k);
                        k.strip_prefix("a ").or_else(|| k.strip_prefix("an ")).unwrap_or(k).to_string()
                    })
                    .collect();
                let compact = with_article(&format!("{} counter", join_list(&bare, "or")));
                format!("put {{alt:your choice of {list}|your choice of {compact}|{compact}}} on {t}")
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
                let you = matches!(who, PlayerRef::You);
                // "Target creature's owner puts it ...": the player is named first.
                if !you {
                    let subj = self.player(who, Case::Subj);
                    let w = self.sel(what, Case::Obj);
                    return format!(
                        "{subj} puts {w} on their choice of the top or bottom of {{alt:their|its owner's}} library"
                    );
                }
                let w = self.sel(what, Case::Obj);
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
            // "~ gains your choice of flying, deathtouch, or lifelink until end of turn",
            // "~ gets +2/-2 or -2/+2 until end of turn".
            Effect::ChooseOne {
                who: PlayerRef::You,
                options,
            } if options.len() > 1
                && options.iter().all(|(_, e)| {
                    matches!(e, Effect::Modify { mods, .. } if matches!(mods.as_slice(), [Modification::AddKeyword(_)] | [Modification::ModifyPT(..)]))
                })
                && options.windows(2).all(|w| match (&w[0].1, &w[1].1) {
                    (
                        Effect::Modify { what: a, duration: da, mods: ma },
                        Effect::Modify { what: b, duration: db, mods: mb },
                    ) => {
                        same_sel(a, b)
                            && format!("{da:?}") == format!("{db:?}")
                            && std::mem::discriminant(&ma[0]) == std::mem::discriminant(&mb[0])
                    }
                    _ => false,
                }) =>
            {
                let Effect::Modify { what, duration, mods } = &options[0].1 else {
                    return self.gap("choice of modifications");
                };
                let w = self.sel(what, Case::Subj);
                let d = self.duration(duration);
                let keywords = matches!(mods[0], Modification::AddKeyword(_));
                let items: Vec<String> = options
                    .iter()
                    .filter_map(|(_, e)| match e {
                        Effect::Modify { mods, .. } => match &mods[0] {
                            Modification::AddKeyword(k) => Some(self.keyword_lower(k)),
                            Modification::ModifyPT(p, t) => {
                                let p = self.value(p);
                                let t = self.value(t);
                                let sign = |s: String| {
                                    if s.starts_with('-') {
                                        s
                                    } else {
                                        format!("+{s}")
                                    }
                                };
                                Some(format!("{}/{}", sign(p), sign(t)))
                            }
                            _ => None,
                        },
                        _ => None,
                    })
                    .collect();
                let vp = if keywords {
                    format!("gains your choice of {}", join_list(&items, "or"))
                } else {
                    format!("gets {}", join_list(&items, "or"))
                };
                join_words(&[format!("{w} {vp}"), d])
            }
            // "Sacrifice an artifact or discard a card": two simple instructions of yours,
            // one of which you choose.
            Effect::ChooseOne {
                who: PlayerRef::You,
                options,
            } if options.len() == 2
                && options.iter().all(|(_, e)| {
                    matches!(
                        e,
                        Effect::Sacrifice {
                            who: PlayerRef::You,
                            ..
                        } | Effect::Discard {
                            who: PlayerRef::You,
                            ..
                        } | Effect::LoseLife {
                            who: PlayerRef::You,
                            ..
                        }
                    )
                }) =>
            {
                let parts: Vec<String> = options
                    .iter()
                    .map(|(_, e)| {
                        let t = self.effect(e);
                        t.strip_prefix("you ").map(str::to_string).unwrap_or(t)
                    })
                    .collect();
                if parts.iter().any(|p| p.contains(['.', '\n'])) {
                    return self.gap("choose one of two instructions");
                }
                parts.join(" or ")
            }
            Effect::ChooseOne { who, options } => {
                let w = self.player(who, Case::Subj);
                // A choice made as the effect resolves (CR 608.2d), not a modal spell's or
                // ability's choice of modes as it's put on the stack (CR 700.2a-b, which a
                // printed "Choose one —" always is): written so the two can't be confused.
                let head = if w == "you" {
                    "as this resolves, choose one —".to_string()
                } else {
                    format!("as this resolves, {w} chooses one —")
                };
                let mut s = head;
                let mut texts = Vec::new();
                for (_, eff) in options {
                    let t = self.effect_sentences(eff);
                    s.push_str(&format!("\n• {t}"));
                    texts.push(t);
                }
                // A choice made as the effect happens (CR 608.2d): "X or Y".
                match tail_parts::or_form(&texts) {
                    Some(o) if w == "you" => format!("{{alt:{s}|{o}}}"),
                    _ => s,
                }
            }
            Effect::Store { sel, var } if matches!(sel, Sel::All(_)) => {
                self.var_defs.push((*var, sel.clone(), false));
                if !self.in_as_player {
                    self.outer_vars.push(*var);
                }
                String::new()
            }
            // A target remembered for a delayed trigger ("Destroy target blocking creature
            // at end of combat"): its first mention there is the target.
            Effect::Store {
                sel: Sel::Target(i),
                var,
            } if self.introduced.get(*i as usize) == Some(&false) => {
                let t = self.target_phrase(*i);
                self.introduced[*i as usize] = true;
                self.target_vars.push((*var, t, false));
                String::new()
            }
            // The source remembered for a delayed trigger ("Sacrifice ~ at the beginning of
            // the next cleanup step"): its first mention there is the source.
            Effect::Store {
                sel: Sel::This,
                var,
            } => {
                self.var_defs.push((*var, Sel::This, false));
                String::new()
            }
            Effect::Store { sel, .. } => match sel {
                // The player a delayed trigger waits for ("at the beginning of that
                // player's next end step"): named there.
                Sel::Players(_) => String::new(),
                Sel::Choose { chooser, count, .. } => {
                    if let (Value::Const(1), Effect::Store { var, .. }) = (count, e) {
                        self.chosen_one.push(*var);
                    }
                    let c = self.player(chooser, Case::Subj);
                    // "You choose a nonland card from that player's graveyard or hand", "You
                    // choose a nonland card from it or a card from their graveyard".
                    if let (true, "you", Sel::Choose { filter, count: Value::Const(1), up_to: false, .. }) =
                        (self.revealed_hand, c.as_str(), sel)
                    {
                        if let Some(s) = self.hand_or_graveyard_choice(filter) {
                            return format!("{{opt:you}} choose {s}");
                        }
                    }
                    let mut s = self.sel(&strip_chooser(sel), Case::Obj);
                    // "Target opponent reveals their hand. You choose a nonland card from
                    // it."
                    if self.revealed_hand {
                        if let Some(i) = s.find(" in ") {
                            if s.ends_with(" hand") || s.ends_with(" hands") {
                                s = super::graveyard_this_way::from_the_hand(&s[..i]);
                            } else if let Some(j) = s[i..].find(" hand ") {
                                // "a card other than a basic land card from it": the
                                // quality after the hand.
                                let rel = s[i + j + " hand ".len()..].to_string();
                                s = format!("{} {rel} from it", &s[..i]);
                            }
                        }
                    }
                    // "You may choose a nonland card from it. If you do, ...": choosing
                    // up to one is choosing one or none.
                    if let (
                        "you",
                        Some(rest),
                        Effect::Store { var, .. },
                        Sel::Choose {
                            count: Value::Const(1),
                            up_to: true,
                            ..
                        },
                    ) = (c.as_str(), s.strip_prefix("up to one "), e, sel)
                    {
                        self.chosen_up_to_one.push(*var);
                        let a = with_article(rest);
                        return format!("{{alt:choose {s}|you may choose {a}}}");
                    }
                    if c == "you" && self.revealed_hand {
                        // "Target player reveals their hand. You choose a card from it."
                        format!("{{opt:you}} choose {s}")
                    } else if c == "you" {
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
                | Sel::TriggerPlayer
                | Sel::AttachedTo
                | Sel::AttachedToThis
                | Sel::Linked
                | Sel::CreatorLinked => String::new(),
                // A group made of groups already named (or none yet): bookkeeping; later
                // mentions are "them".
                Sel::Union(parts)
                    if parts.iter().all(|p| {
                        matches!(
                            p,
                            Sel::Var(_)
                                | Sel::Target(_)
                                | Sel::This
                                | Sel::TriggerObject
                                | Sel::Players(_)
                        )
                    }) =>
                {
                    if let Effect::Store { var, .. } = e {
                        if !parts.is_empty() {
                            self.plural_vars.push(*var);
                        }
                    }
                    String::new()
                }
                Sel::Union(parts) => {
                    let var = match e {
                        Effect::Store { var, .. } => *var,
                        _ => 0,
                    };
                    match self.remember_union(var, parts) {
                        Some(s) => s,
                        None => self.gap(format!("remembering {sel:?}")),
                    }
                }
                // "Choose a creature at random" (CR 608.2d): the one chosen is "it" later.
                Sel::AtRandom { filter, count, .. } => {
                    let det = self.det_for(count);
                    let n = self.noun_det(filter, det);
                    match n.split_once(" that ") {
                        Some((head, rel)) => format!(
                            "{{alt:choose {head} at random that {rel}|choose {n} at random}}"
                        ),
                        None => format!("choose {n} at random"),
                    }
                }
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
                // "the number of creatures tapped this way", worded later.
                if let Value::CountSel(c) = &value {
                    if let Sel::Var(x) = c.as_ref() {
                        if let Some(e) = self.this_way.iter().rev().find(|e| e.0 == *x) {
                            if !e.1.is_empty() {
                                self.stored_this_way.push(*e);
                            }
                        }
                    }
                }
                // A variable stored again (a count set to 0, then to what an instruction
                // did) holds the later value from here on.
                match self.stored_values.iter_mut().find(|(x, _, _)| x == var) {
                    Some(e) => e.1 = value,
                    None => self.stored_values.push((*var, value, false)),
                }
                String::new()
            }
            Effect::Note { value } => {
                let v = self.value(value);
                format!("note {v}")
            }
            Effect::SetX { .. } => String::new(),
            // Bookkeeping for "... they paid this way" (named by the value that reads it).
            Effect::AddToPlayerTally { .. } => String::new(),
            Effect::Destroy { what, no_regen } => {
                let w = self.sel(what, Case::Obj);
                // "... for each permanent destroyed this way".
                if let Sel::All(f) = what {
                    let n = self.noun(f, Num::Many);
                    self.it_done = Some(format!("{{alt:permanents|{n}}} destroyed this way"));
                }
                if *no_regen {
                    let plural_sel = is_plural_sel(what);
                    let pron = if plural_sel { "They" } else { "It" };
                    // "Creatures destroyed this way can't be regenerated."
                    let filter = match what {
                        Sel::All(f) | Sel::Choose { filter: f, .. } => Some(f.clone()),
                        Sel::Target(i) => {
                            match self.targets.get(*i as usize).map(|t| t.what.clone()) {
                                Some(TargetKind::Object(f)) => Some(f),
                                _ => None,
                            }
                        }
                        _ => None,
                    };
                    let types: Vec<CardType> = match &filter {
                        Some(Filter::And(v)) => v
                            .iter()
                            .filter_map(|x| match x {
                                Filter::Type(t) => Some(*t),
                                _ => None,
                            })
                            .collect(),
                        Some(Filter::Type(t)) => vec![*t],
                        _ => Vec::new(),
                    };
                    let pron = match types.as_slice() {
                        [t] => {
                            let head = t.word().to_string();
                            if plural_sel {
                                format!(
                                    "{{alt:{pron}|{} destroyed this way}}",
                                    capitalize(&plural(&head))
                                )
                            } else {
                                format!(
                                    "{{alt:{pron}|{} destroyed this way}}",
                                    capitalize(&with_article(&head))
                                )
                            }
                        }
                        _ => pron.to_string(),
                    };
                    format!("destroy {w}. {pron} can't be regenerated")
                } else {
                    format!("destroy {w}")
                }
            }
            // "Target opponent exiles a card from their hand.": the player chooses the
            // card they exile (who performs the exile makes no difference, CR 701.13a).
            Effect::Exile {
                what:
                    Sel::Choose {
                        chooser: PlayerRef::Target(t),
                        filter: Filter::And(v),
                        count,
                        up_to: false,
                        store: None,
                    },
                face_down: false,
                ..
            } if v
                .iter()
                .any(|f| matches!(f, Filter::InZone(ZoneKind::Hand | ZoneKind::Graveyard)))
                && v.iter()
                    .any(|f| matches!(f, Filter::OwnedBy(PlayerRel::Target(o)) if o == t)) =>
            {
                let zone = if v
                    .iter()
                    .any(|f| matches!(f, Filter::InZone(ZoneKind::Graveyard)))
                {
                    "graveyard"
                } else {
                    "hand"
                };
                let rest: Vec<Filter> = v
                    .iter()
                    .filter(|f| !matches!(f, Filter::InZone(_) | Filter::OwnedBy(_)))
                    .cloned()
                    .collect();
                let subj = self.player(&PlayerRef::Target(*t), Case::Subj);
                let det = self.det_for(count);
                let n = self.noun_det(&Filter::and(rest), det);
                format!("{subj} exiles {n} from their {zone}")
            }
            // "Target opponent exiles a creature they control.": the player chooses the
            // permanent they exile (CR 701.13a).
            Effect::Exile {
                what:
                    Sel::Choose {
                        chooser: PlayerRef::Target(t),
                        filter: Filter::And(v),
                        count,
                        up_to: false,
                        store: None,
                    },
                face_down: false,
                ..
            } if v
                .iter()
                .any(|f| matches!(f, Filter::ControlledBy(PlayerRel::Target(o)) if o == t))
                && v.iter().all(|f| {
                    !matches!(f, Filter::InZone(_) | Filter::OwnedBy(_) | Filter::ControlledBy(_))
                        || matches!(f, Filter::InZone(ZoneKind::Battlefield))
                        || matches!(f, Filter::ControlledBy(PlayerRel::Target(o)) if o == t)
                }) =>
            {
                let rest: Vec<Filter> = v
                    .iter()
                    .filter(|f| !matches!(f, Filter::ControlledBy(_) | Filter::InZone(_)))
                    .cloned()
                    .collect();
                let subj = self.player(&PlayerRef::Target(*t), Case::Subj);
                let det = self.det_for(count);
                // "a creature or planeswalker they control with the greatest mana value
                // among creatures and planeswalkers they control": who controls it is
                // said with the noun, before the comparison.
                if v.iter().any(|f| matches!(f, Filter::ValueCmp(..))) {
                    let saved = self.they_target.replace(*t);
                    let n = self.noun_det(&Filter::And(v.clone()), det);
                    self.they_target = saved;
                    return format!("{subj} exiles {n}");
                }
                let n = self.noun_det(&Filter::and(rest), det);
                format!("{subj} exiles {n} they control")
            }
            // "Each opponent exiles a card from their hand": each one chooses among their
            // own cards.
            Effect::Exile {
                what:
                    Sel::Choose {
                        chooser: PlayerRef::Iterated,
                        filter: Filter::And(v),
                        count,
                        up_to: false,
                        store: None,
                    },
                face_down: false,
                ..
            } if v
                .iter()
                .any(|f| matches!(f, Filter::InZone(ZoneKind::Hand | ZoneKind::Graveyard)))
                && v.iter().any(|f| matches!(f, Filter::OwnedBy(PlayerRel::Iterated))) =>
            {
                let zone = if v.iter().any(|f| matches!(f, Filter::InZone(ZoneKind::Graveyard))) {
                    "graveyard"
                } else {
                    "hand"
                };
                let rest: Vec<Filter> = v
                    .iter()
                    .filter(|f| !matches!(f, Filter::InZone(_) | Filter::OwnedBy(_)))
                    .cloned()
                    .collect();
                let det = self.det_for(count);
                let n = self.noun_det(&Filter::and(rest), det);
                format!("{{alt:that player|they}} exiles {n} from their {zone}")
            }
            Effect::Exile {
                what, face_down, ..
            } => {
                // "... for each card exiled this way".
                self.it_done = Some("cards exiled this way".into());
                let w = match self.enchanted_card(false) {
                    Some(c) if matches!(what, Sel::AttachedTo) => c,
                    _ => self.sel(what, Case::Obj),
                };
                if *face_down {
                    format!("exile {w} face down")
                } else {
                    format!("exile {w}")
                }
            }
            Effect::Sacrifice { .. } => unreachable_text(),
            // "Target creature's controller sacrifices it": a player sacrifices only a
            // permanent they control (CR 701.21a).
            Effect::SacrificeObjects { what: Sel::Target(i) }
                if self.targets.get(*i as usize).is_some_and(|t| {
                    matches!(&t.what, TargetKind::Object(f) if !format!("{f:?}").contains("ControlledBy(You)"))
                }) && !self.introduced.get(*i as usize).copied().unwrap_or(false) =>
            {
                let w = self.sel(&Sel::Target(*i), Case::Obj);
                format!("{w}'s controller sacrifices it")
            }
            Effect::SacrificeObjects { what } => {
                let w = self.sel(what, Case::Obj);
                // A player sacrifices only permanents they control (CR 701.21a): "you
                // control" restates it ("sacrifice any number of lands").
                let w = if self.in_as_player {
                    w
                } else {
                    w.replacen(" you control", " {opt:you control}", 1)
                };
                format!("sacrifice {w}")
            }
            // "You may put a permanent card from among them into your hand": up to one.
            Effect::Move {
                what:
                    Sel::Choose {
                        chooser: PlayerRef::You,
                        filter,
                        count: Value::Const(1),
                        up_to: true,
                        store: None,
                    },
                to,
            } if !format!("{filter:?}").contains("Target") => {
                let one = Sel::Choose {
                    chooser: PlayerRef::You,
                    filter: filter.clone(),
                    count: Value::Const(1),
                    up_to: false,
                    store: None,
                };
                let a = self.move_effect(&one, to);
                let b = self.move_effect(
                    &Sel::Choose {
                        chooser: PlayerRef::You,
                        filter: filter.clone(),
                        count: Value::Const(1),
                        up_to: true,
                        store: None,
                    },
                    to,
                );
                format!("{{alt:{b}|you may {a}}}")
            }
            // "Return an enchantment card from your graveyard to your hand for each card
            // revealed this way": that many chosen.
            Effect::Move {
                what:
                    what @ Sel::Choose {
                        chooser,
                        filter,
                        count: count @ (Value::Count(_) | Value::CountSel(_)),
                        up_to: false,
                        store,
                    },
                to,
            } => {
                let v = self.value(count);
                match v.strip_prefix("the number of ") {
                    Some(rest) => {
                        let one = Sel::Choose {
                            chooser: chooser.clone(),
                            filter: filter.clone(),
                            count: Value::Const(1),
                            up_to: false,
                            store: *store,
                        };
                        let m = self.move_effect(&one, to);
                        format!("{m} for each {rest}")
                    }
                    None => self.move_effect(what, to),
                }
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
                // The recipient is named before "where X is ...", which may refer back to
                // it ("deals X damage to target player, where X is the number of cards in
                // that player's hand").
                let refers = matches!(to, Sel::Target(i)
                    if format!("{amount:?}").contains(&format!("Target({i})")));
                let (mut t, (a, w)) = if refers {
                    let t = self.sel(to, Case::Obj);
                    (t, self.amount(amount))
                } else {
                    let a = self.amount(amount);
                    (self.sel(to, Case::Obj), a)
                };
                // "Target creature deals damage to itself equal to its power."
                if format!("{source:?}") == format!("{to:?}") && !matches!(to, Sel::This) {
                    t = "itself".into();
                } else if matches!((source, to), (Sel::This, Sel::This)) && !t.contains('{') {
                    // "~ deals 3 damage to itself" (cards also name it again).
                    t = format!("{{alt:{t}|itself}}");
                }
                // "Whenever ~ becomes blocked by a creature, it deals 2 damage to that
                // creature": the object itself and the other object of the event.
                let s = if matches!(source, Sel::This)
                    && matches!(to, Sel::TriggerObject | Sel::TriggerLki)
                    && self.trigger_self_and_other
                    && t == "it"
                {
                    t = "that-object".into();
                    "~it".to_string()
                } else {
                    s
                };
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
                // "~ deals damage equal to its power": the source's own value is "its"
                // even while another object is "it" (Gleeful Arsonist's spell).
                let w = w.unwrap_or_default();
                let w = if matches!(source, Sel::This) && s == "~" {
                    w.replace(" ~'s ", " ~it's ")
                } else {
                    w
                };
                format!("{s} deals {a} damage to {t}{w}")
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
                // "deals X damage divided as you choose among any number of target
                // creatures, where X is ..." (CR 107.3).
                let (a, w) = match amount {
                    Some(v) => self.amount(&v),
                    None => (self.gap("divided damage without amount"), None),
                };
                let t = self.target_mention(*slot, Case::Obj);
                format!(
                    "{s} deals {a} damage divided as you choose among {t}{}",
                    w.unwrap_or_default()
                )
            }
            Effect::Fight { a, b } => {
                let two_targets =
                    matches!((a, b), (Sel::Target(x), Sel::Target(y)) if x != y);
                let a = self.sel(a, Case::Subj);
                let b = self.sel(b, Case::Obj);
                // Two targets both named before: "those creatures fight each other".
                if two_targets && !a.contains("target") && !b.contains("target") {
                    return "{alt:those creatures|they} fight each other".into();
                }
                format!("{a} fights {b}")
            }
            Effect::AddCounters { what, kind, n } => {
                // The object is named before the "where X is" clause about it ("put X
                // +1/+1 counters on target creature, where X is that creature's power").
                let (mut t, (c, w)) = if matches!(what, Sel::Target(_))
                    && format!("{n:?}").contains(&format!("{what:?}"))
                {
                    let t = self.sel(what, Case::Obj);
                    (t, self.counted(n, &counter_name(kind)))
                } else {
                    let cw = self.counted(n, &counter_name(kind));
                    (self.sel(what, Case::Obj), cw)
                };
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
                let has_where = w.is_some();
                let mut s = format!("put {c} on {t}{}", w.unwrap_or_default());
                // "put two +1/+1 counters on ~ for each sprout vote".
                if has_where {
                    let saved = self.gaps.len();
                    match self.counted_for_each(n, &counter_name(kind)) {
                        Some((c1, each)) => s = format!("{{alt:{s}|put {c1} on {t} {each}}}"),
                        None => self.gaps.truncate(saved),
                    }
                }
                match self.support_n(what, kind, n) {
                    Some(k) => format!("{{alt:support {k}|{s}}}"),
                    None => s,
                }
            }
            Effect::RemoveCounters { what, kind, n } => {
                self.removed_kind = Some(match kind {
                    Some(k) => counter_name(k),
                    None => "counter".into(),
                });
                let noun = match kind {
                    Some(k) => counter_name(k),
                    None => "counter".into(),
                };
                let t = self.sel(what, Case::Obj);
                // "Remove all charge counters from ~": as many as are on it.
                let all_on_it = matches!(n, Value::CountersOn(s, k)
                    if same_sel(s, what) && format!("{k:?}") == format!("{kind:?}"));
                match n {
                    Value::Const(i) if *i >= 1000 => {
                        format!("remove all {} from {t}", plural(&noun))
                    }
                    _ if all_on_it => format!("remove all {} from {t}", plural(&noun)),
                    _ => {
                        let (c, w) = self.counted(n, &noun);
                        format!("remove {c} from {t}{}", w.unwrap_or_default())
                    }
                }
            }
            Effect::RemoveCountersUpTo { what, kind, max } => {
                let noun = match kind {
                    Some(k) => counter_name(k),
                    None => "counter".into(),
                };
                let t = self.sel(what, Case::Obj);
                match max {
                    None => format!("remove any number of {} from {t}", plural(&noun)),
                    Some(v) => {
                        let (c, w) = self.counted(v, &noun);
                        format!("remove up to {c} from {t}{}", w.unwrap_or_default())
                    }
                }
            }
            // "Choose a counter on target permanent. Put an additional counter of that kind
            // on that permanent."
            Effect::ChooseCounterKind { from, then } => {
                let f = self.sel(from, Case::Obj);
                let inner = self.effect(then);
                let k = crate::ability::CHOSEN_COUNTER_KIND;
                let inner = inner
                    .replace(&format!("{k} counters"), "counters of that kind")
                    .replace(&format!("{k} counter"), "counter of that kind");
                format!("choose a counter on {f}. {inner}")
            }
            Effect::ForEachCounterKind { from, then } => {
                let f = self.sel(from, Case::Obj);
                // "put another counter of that kind on it or remove one from it".
                if let Effect::ChooseOne { options, .. } = then.as_ref() {
                    if let [(_, Effect::AddCounters { what: a, .. }), (_, Effect::RemoveCounters { what: r, .. })] =
                        options.as_slice()
                    {
                        let (a, r) = (self.sel(a, Case::Obj), self.sel(r, Case::Obj));
                        if a == r {
                            return format!("for each kind of counter on {f}, put another counter of that kind on {a} or remove one from {a}");
                        }
                    }
                }
                let inner = self.effect(then);
                let inner = inner.replace(
                    &format!("put a {} counter on it", crate::ability::CHOSEN_COUNTER_KIND),
                    "{alt:put another counter of that kind on it|give that permanent or player another counter of that kind}",
                );
                let k = crate::ability::CHOSEN_COUNTER_KIND;
                let inner = inner
                    .replace(&format!("{k} counters"), "counters of that kind")
                    .replace(&format!("{k} counter"), "counter of that kind");
                format!("for each kind of counter on {f}, {inner}")
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
            Effect::CountersFromAmong {
                from,
                to,
                kind,
                min,
                max,
                each_missing_kind,
            } => {
                let noun = match kind {
                    Some(k) => counter_name(k),
                    None => "counter".into(),
                };
                let t = to.as_ref().map(|t| self.sel(t, Case::Obj));
                if *each_missing_kind {
                    let f = self.sel(from, Case::Obj);
                    let t = t.unwrap_or_default();
                    return format!("move a {noun} of each kind not on {t} from {f} onto {t}");
                }
                let qty = match (min, max) {
                    (Value::Const(0), None) => format!("any number of {}", plural(&noun)),
                    (Value::Const(1), None) => format!("one or more {}", plural(&noun)),
                    (Value::Const(0), Some(m)) => {
                        let (c, _) = self.counted(m, &noun);
                        format!("up to {c}")
                    }
                    (n, _) => self.counted(n, &noun).0,
                };
                let f = match from {
                    Sel::All(f) => {
                        let f = f.clone();
                        let on_bf = |x: &Filter| matches!(x, Filter::InZone(ZoneKind::Battlefield));
                        let f = match f {
                            Filter::And(v) => Filter::and(v.into_iter().filter(|x| !on_bf(x)).collect()),
                            f => f,
                        };
                        format!("among {}", self.noun(&f, Num::Many))
                    }
                    s => self.sel(s, Case::Obj),
                };
                match t {
                    Some(t) => format!("move {qty} from {f} onto {t}"),
                    None => format!("remove {qty} from {f}"),
                }
            }
            Effect::DistributeCounters {
                n,
                kind,
                among,
                min_objects,
                max_objects,
                up_to,
            } => {
                let noun = counter_name(kind);
                let (c, w) = self.counted(n, &noun);
                let c = if *up_to { format!("up to {c}") } else { c };
                let g = match among {
                    Sel::All(f) => self.noun(f, Num::Many),
                    s => self.sel(s, Case::Obj),
                };
                let q = match (min_objects, max_objects) {
                    (_, Some(3)) => "one, two, or three",
                    (_, Some(2)) => "one or two",
                    _ => "any number of",
                };
                format!("distribute {c} among {q} {g}{}", w.unwrap_or_default())
            }
            Effect::PutEventCounters { to, one_each } => {
                let t = self.sel(to, Case::Obj);
                if *one_each {
                    format!("put one of each of those kinds of counters on {t}")
                } else {
                    format!("put the same number and kind of counters on {t}")
                }
            }
            Effect::PutCountersOf { from, to, kind } => {
                let f = self.sel(from, Case::Poss);
                let t = self.sel(to, Case::Obj);
                match kind {
                    Some(k) => format!("put {f} {k} counters on {t}"),
                    // "If it had counters on it, put those counters on ~."
                    None if matches!(from, Sel::TriggerLki | Sel::TriggerObject | Sel::This) => {
                        format!("put {{alt:{f} counters|those counters}} on {t}")
                    }
                    None => format!("put {f} counters on {t}"),
                }
            }
            // "Exile that card with three time counters on it instead of putting it into
            // your graveyard as it resolves. Then if the exiled card doesn't have suspend,
            // it gains suspend." (see `kw/suspend_as_it_resolves.rs`)
            Effect::Modify {
                what: Sel::TriggerSpell,
                mods,
                ..
            } if matches!(mods.as_slice(), [Modification::AddAbility(a)]
                if crate::kw::suspend_as_it_resolves::parse_marker(&a.text).is_some()) =>
            {
                let [Modification::AddAbility(a)] = mods.as_slice() else {
                    return self.gap("exile as it resolves");
                };
                let Some((kind, n, suspend)) =
                    crate::kw::suspend_as_it_resolves::parse_marker(&a.text)
                else {
                    return self.gap("exile as it resolves");
                };
                let (c, _) = self.counted(&Value::c(n as i32), &counter_name(&kind));
                let mut s = format!(
                    "exile that card with {c} on it instead of putting it into your graveyard as it resolves"
                );
                if suspend {
                    s.push_str(". Then if the exiled card doesn't have suspend, it gains suspend");
                }
                s
            }
            Effect::Modify {
                what,
                mods,
                duration,
            } => {
                let s = self.modify_effect(what, mods, duration);
                duration_for_each_part(s)
            }
            Effect::AddRestriction {
                restriction,
                duration,
            } => {
                if let Restriction::DoesntUntap(f) = restriction {
                    // "Its controller's next untap step" and "your next untap step" are
                    // different durations (they differ once the permanent changes
                    // control), compiled as different `Duration`s.
                    let whose = match duration {
                        Duration::ThroughNextUntapStep => Some("{alt:its controller's|its}"),
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
                // "All creatures able to block target creature this turn do so."
                if d == "this turn" && r.ends_with(" do so") && r.starts_with("all creatures able")
                {
                    return r.replacen(" do so", " this turn do so", 1);
                }
                if d == "this turn" && r.contains(" can't be blocked except by ") {
                    return r.replacen(" except by ", " this turn except by ", 1);
                }
                // "You choose which creatures block this combat and how those creatures
                // block."
                if let Some((a, b)) = r.split_once(" block and how ") {
                    if !d.is_empty() {
                        let d = if d == "until end of combat" {
                            "this combat"
                        } else {
                            &d
                        };
                        return format!("{a} block {d} and how {b}");
                    }
                }
                // "Target creature blocks this turn if able.", "Target creature must be
                // blocked this turn if able.", "... blocks it this combat if able."
                if d == "this turn" {
                    if let Some(x) = r
                        .strip_suffix(" each combat if able")
                        .or_else(|| r.strip_suffix(" if able"))
                    {
                        return format!("{x} this turn if able");
                    }
                    // "Target creature blocks target creature this turn if able."
                    if let Some(x) = r.strip_suffix(" this combat if able") {
                        return format!("{x} this turn if able");
                    }
                }
                // "... blocks it this combat if able."
                if d == "until end of combat" && r.ends_with(" this combat if able") {
                    return r;
                }
                if d == "until end of combat" {
                    if let Some(x) = r.strip_suffix(" each combat if able") {
                        return format!("{x} this combat if able");
                    }
                    // "It can't be blocked this combat."
                    return format!("{r} {{alt:until end of combat|this combat}}");
                }
                join_words(&[r, d])
            }
            // "You may play an additional land this turn."
            Effect::AddPlayerEffect {
                who,
                effect: PlayerModification::AdditionalLandPlays(n),
                duration: Duration::ThisTurn,
            } => {
                let lands = match n {
                    1 => "an additional land".to_string(),
                    n => format!("{} additional lands", number_word(*n as i32)),
                };
                self.with_subject(who, &format!("may play {lands} this turn"), false)
            }
            Effect::AddPlayerEffect {
                who,
                effect,
                duration,
            } => {
                let mut s = self.player_modification(who, effect);
                // A player "gains" hexproof, shroud, or protection for a duration.
                if matches!(
                    effect,
                    PlayerModification::Hexproof
                        | PlayerModification::HexproofFrom(_)
                        | PlayerModification::Shroud
                        | PlayerModification::ProtectionFrom(_)
                ) {
                    s = s.replacen(" have ", if s.starts_with("you ") || s.starts_with("{alt:your opponents") { " gain " } else { " gains " }, 1);
                }
                // A resolving effect with no duration lasts for the rest of the game (CR
                // 611.2a); cards say so or not.
                let d = match duration {
                    Duration::Permanent => "{opt:for the rest of the game}".to_string(),
                    d => self.duration(d),
                };
                join_words(&[s, d])
            }
            Effect::AddReplacement {
                def,
                duration,
                uses,
            } => {
                let s = self.add_replacement(def, duration, uses);
                // A replacement effect that applies only once is always said to ("the
                // next time ... would ...", CR 614.1); a wording that doesn't say it
                // misrepresents it. One about a particular object entering ("that
                // creature enters with an additional +1/+1 counter") can apply only once
                // anyway: an object that enters again is a new object (CR 400.7).
                let one_object = matches!(
                    &def.event,
                    ReplacementEvent::EntersBattlefield(Filter::In(_))
                );
                if uses.is_some() && !one_object && !s.contains("next") {
                    return self.gap("one-use replacement effect");
                }
                s
            }
            Effect::GainControl {
                what,
                who,
                duration,
            } => {
                // "Each player gains control of each creature they own": gaining control
                // of what the player already controls changes nothing, so "that player
                // doesn't control" goes unsaid.
                let what = match what {
                    Sel::All(Filter::And(v)) => {
                        let rel = match who {
                            PlayerRef::Iterated => Some(PlayerRel::Iterated),
                            PlayerRef::You => Some(PlayerRel::You),
                            _ => None,
                        };
                        let kept: Vec<Filter> = v
                            .iter()
                            .filter(|f| !matches!((f, rel), (Filter::Not(n), Some(r)) if matches!(n.as_ref(), Filter::ControlledBy(x) if *x == r)))
                            .cloned()
                            .collect();
                        Sel::All(Filter::and(kept))
                    }
                    other => other.clone(),
                };
                let what = &what;
                let w = self.sel(what, Case::Obj);
                let d = self.duration(duration);
                let vp = join_words(&[format!("gain control of {w}"), d]);
                self.with_subject(who, &vp, false)
            }
            // "Exchange control of two target creatures": the two objects of one target
            // instance (CR 701.12b).
            Effect::ExchangeControl { a, b } if same_sel(a, b) => {
                let a = self.sel(a, Case::Obj);
                format!("exchange control of {a}")
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
                // "For each creature card revealed this way, create a token that's a copy
                // of that card": one copy of each.
                if let (Sel::Matching(u, f), Value::Const(1), false, false, PlayerRef::You) =
                    (of, count, *tapped, *attacking, controller)
                {
                    if super::values::is_revealed_this_way(u) {
                        let one = self.noun(f, Num::One);
                        let ex = if mods.is_empty() {
                            String::new()
                        } else {
                            format!(", except {}", self.exceptions(mods))
                        };
                        return format!(
                            "for each {one} revealed this way, create a token that's a copy of {{alt:that card|it}}{ex}"
                        );
                    }
                }
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
                let ex = if mods.is_empty() {
                    String::new()
                } else {
                    format!(", except {}", self.exceptions(mods))
                };
                vp.push_str(&ex);
                // "create a token that's a copy of another target attacking creature and
                // that's tapped and attacking" / "... The token enters tapped and
                // attacking." / "a token that's a copy of ~ tapped and attacking": the
                // token enters that way (CR 508.4) whichever way it's said.
                if *tapped && *attacking && matches!(count, Value::Const(1)) {
                    let mut alts = vec![
                        vp.clone(),
                        format!("create {head} tapped and attacking{ex}"),
                    ];
                    if ex.is_empty() {
                        alts.push(format!("create {head} and that's tapped and attacking"));
                        alts.push(format!(
                            "create {head}. The token enters tapped and attacking"
                        ));
                    }
                    vp = format!("{{alt:{}}}", alts.join("|"));
                }
                // "For each token you control, create a token that's a copy of that
                // permanent": one copy of each of them.
                if let (Sel::All(f), Value::Const(1), true, true) =
                    (of, count, st.is_empty(), mods.is_empty())
                {
                    if !vp.contains('{') && matches!(controller, PlayerRef::You) {
                        let one = self.noun(f, Num::One);
                        vp = format!(
                            "{{alt:{vp}|for each {one}, create a token that's a copy of {{alt:it|that permanent}}}}"
                        );
                    }
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
                    s.push_str(&self.choose_new_targets(c));
                }
                s
            }
            Effect::TokensJoinCombat {
                effect,
                attacking,
                blocking,
            } => {
                // "create a token that's a copy of ~ tapped and attacking that player,
                // except it isn't legendary": the player it attacks before the exceptions.
                if let (
                    Effect::CreateTokenCopy {
                        of,
                        count: Value::Const(1),
                        controller: PlayerRef::You,
                        tapped: true,
                        attacking: true,
                        mods,
                    },
                    Some(p),
                    None,
                ) = (effect.as_ref(), attacking, blocking)
                {
                    if !mods.is_empty() {
                        let s = self.effect(effect);
                        let o = self.sel(of, Case::Obj);
                        let p = self.player(p, Case::Obj);
                        let ex = self.exceptions(mods);
                        return format!(
                            "{{alt:{s} {p}|create a token that's a copy of {o} tapped and attacking {p}, except {ex}}}"
                        );
                    }
                }
                // "create a 1/1 white Human creature token that's tapped and attacking that
                // player or a planeswalker they control" (`kw/token_creation_rules.rs`).
                if let (Effect::Seq(v), Some(p)) = (effect.as_ref(), attacking) {
                    if let [Effect::Custom(n), rest] = v.as_slice() {
                        if n == crate::kw::token_creation_rules::OR_THEIR_PLANESWALKERS {
                            let s = self.effect(rest);
                            let p = self.player(p, Case::Obj);
                            return format!("{s} {p} or a planeswalker they control");
                        }
                    }
                }
                let mut s = self.effect(effect);
                if let Some(p) = attacking {
                    let p = self.player(p, Case::Obj);
                    s.push_str(&format!(" {p}"));
                }
                if let Some(a) = blocking {
                    let a = self.sel(a, Case::Obj);
                    s.push_str(&format!(" that's blocking {a}"));
                }
                s
            }
            Effect::CopySpellExcept {
                what,
                count,
                new_targets,
                mods,
            } => {
                let w = self.sel(what, Case::Obj);
                let ex = self.exceptions(mods);
                let ex = ex
                    .replacen("it isn't", "the copy isn't", 1)
                    .replacen("it's", "the copy is", 1);
                let mut s = match count {
                    Value::Const(1) => format!("copy {w}, except {ex}"),
                    other => {
                        let t = self.times(other);
                        format!("copy {w} {t}, except {ex}")
                    }
                };
                if *new_targets {
                    let c = if matches!(count, Value::Const(1)) {
                        "the copy"
                    } else {
                        "the copies"
                    };
                    s.push_str(&self.choose_new_targets(c));
                }
                s
            }
            Effect::OfferSpecialAction {
                def,
                duration,
                repeatable,
            } => {
                if matches!(duration, Duration::WhileSourceOnBattlefield) && !*repeatable {
                    if let Some(s) = self.special_action_extra(def) {
                        return s;
                    }
                }
                let d = self.duration(duration);
                let c = self.cost(&def.cost);
                if let SpecialActionEffect::StopDelayedTrigger = &def.action {
                    // "unless they pay {2} before that step": the player, as the subject.
                    let who = match &def.who {
                        PlayerFilter::Ref(r) => self.player(r, Case::Subj),
                        other => self.player_filter_object(other),
                    };
                    return format!("{{alt:unless {who} pays {c} before that step|{who} may pay {c} before then to stop it}}");
                }
                let what = match &def.action {
                    SpecialActionEffect::Effect(e) => self.effect(e),
                    SpecialActionEffect::IgnoreSourceEffects => {
                        "ignore this effect until end of turn".into()
                    }
                    SpecialActionEffect::StopDelayedTrigger => String::new(),
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
                    // "Change a target of target spell or ability to ~": one of its targets
                    // (CR 115.7c).
                    (TargetChange::One, Some(t)) => {
                        let t = self.sel(t, Case::Obj);
                        format!("change {{alt:a|the}} target of {w} to {t}")
                    }
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
                // "becomes a copy of X until end of turn, except ..." / "Until end of
                // turn, ~ becomes a copy of X, except ...".
                if d.is_empty() {
                    format!("{w} becomes a copy of {o}, except {ex}")
                } else {
                    format!(
                        "{{alt:{w} becomes a copy of {o} {d}, except {ex}|{w} becomes a copy of {o}, except {ex} {d}}}"
                    )
                }
            }
            Effect::Transform { what } => {
                let w = self.sel(what, Case::Obj);
                format!("transform {w}")
            }
            Effect::Regenerate { what } => {
                let w = self.sel(what, Case::Obj);
                format!("regenerate {w}")
            }
            Effect::RegenerateWith { what, then } => {
                let w = self.sel(what, Case::Obj);
                match then.as_ref() {
                    Effect::Reflexive { body } => {
                        let e = self.effect(&body.effect);
                        format!("regenerate {w}. When it regenerates this way, {e}")
                    }
                    e => {
                        let e = self.effect(e);
                        format!("regenerate {w}. {e} if it regenerates this way")
                    }
                }
            }
            Effect::Attach { what, to } | Effect::AttachAsCreature { what, to } => {
                // (Naming a target doesn't make "it" in "to it" the target.)
                let salient = self.self_salient;
                let w = self.sel(what, Case::Obj);
                if matches!(to, Sel::This) && !matches!(what, Sel::This) {
                    self.self_salient = salient;
                }
                // "attach ~ to another instant card in a graveyard": other than the card it
                // enchants.
                if let Sel::Choose {
                    filter: Filter::And(v),
                    ..
                } = to
                {
                    if let [Filter::Type(t), Filter::Card, Filter::InZone(ZoneKind::Graveyard), Filter::Not(n)] =
                        v.as_slice()
                    {
                        if matches!(&**n, Filter::In(s) if matches!(**s, Sel::AttachedTo)) {
                            let t = t.word().to_lowercase();
                            return format!("attach {w} to another {t} card in a graveyard");
                        }
                    }
                }
                // "attach target Equipment to it": "it" isn't the object being attached.
                let saved = self.other_salient;
                if matches!(to, Sel::This) && !matches!(what, Sel::This) {
                    self.other_salient = false;
                }
                let t = self.sel(to, Case::Obj);
                self.other_salient = saved;
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
            Effect::BecomeUnblocked { what } => {
                let w = self.sel(what, Case::Subj);
                format!("{w} become unblocked")
            }
            Effect::BlockAttackerOfChoice { blocker } => {
                let w = self.sel(blocker, Case::Subj);
                format!("{w} blocks an attacking creature of your choice")
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
                let x_form = format!("{m} enters with {c} on it{}", w.clone().unwrap_or_default());
                // "enters with a number of +1/+1 counters on it equal to ...".
                match w.as_deref().and_then(|w| w.strip_prefix(", where X is ")) {
                    Some(v) if !v.starts_with("the number of ") => {
                        let cs = plural(&counter_name(kind));
                        either_form(
                            x_form.clone(),
                            format!("{m} enters with a number of {cs} on it equal to {v}"),
                        )
                    }
                    _ => x_form,
                }
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
                // "except it has vanishing 3 if that creature doesn't have vanishing",
                // "except it enters with a shield counter on it if you control that
                // creature": a condition on the object copied (CR 707.9f).
                let copied = match only_if {
                    Some(Filter::Custom(n)) => n
                        .strip_prefix(crate::copy_rules::COPIED_MATCHES)
                        .and_then(|j| serde_json::from_str::<Filter>(j).ok()),
                    _ => None,
                };
                if let Some(g) = copied {
                    let cond = match &g {
                        Filter::ControlledBy(PlayerRel::You) => {
                            "you control that creature".to_string()
                        }
                        Filter::Not(x) => format!("that creature {}", self.is_predicate(x, true)),
                        x => format!("that creature {}", self.is_predicate(x, false)),
                    };
                    if !mods.is_empty() {
                        parts.push(self.exceptions(mods));
                    }
                    let e = self.effect(effect);
                    if !e.is_empty() {
                        parts.push(e);
                    }
                    return format!("except {} if {cond}", parts.join(" "));
                }
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
            | Effect::SearchCards(_)
            | Effect::CreateEmblem { .. } => unreachable_text(),
            // "Each player's life total becomes the lowest life total among all players."
            Effect::SetLife {
                who,
                n: n @ Value::OverPlayers(..),
            } => {
                let p = self.player(who, Case::Poss);
                let v = self.value(n);
                format!("{p} life total becomes {v}")
            }
            Effect::SetLife { who, n } => {
                let p = self.player(who, Case::Poss);
                let (a, w) = self.amount(n);
                // "Your life total becomes equal to your starting life total."
                if let Some(w) = w {
                    let v = self.value(n);
                    return format!("{p} life total becomes {{alt:{a}{w}|equal to {v}}}");
                }
                format!("{p} life total becomes {a}")
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
                if let (ManaProduction::EachColorAmong(f), PlayerRef::You, None) =
                    (mana, who, restriction)
                {
                    let n = self.noun(f, Num::Many);
                    return format!("for each color among {n}, add one mana of that color");
                }
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
                abilities,
                additional,
            } => {
                let a = self.effect(add);
                let f = self.noun_det(spell_filter, Det::A);
                let f = if f.contains("spell") {
                    f
                } else {
                    format!("{f} spell")
                };
                // The spell is the object the rider is about: "it gains haste" (the
                // mana ability's source isn't named in it).
                let saved = self.self_salient;
                self.self_salient = false;
                let b = self.in_event_scope(|r| r.body(body));
                self.self_salient = saved;
                if *abilities {
                    format!("{a}. When you spend this mana to cast {f} or activate an ability, {b}")
                } else if *additional {
                    // Mana spent "on" a spell is spent to cast it (CR 601.2h); the rider
                    // applies if any of that mana is (Arena of Glory's rulings).
                    format!(
                        "{a}. If {{opt:any of}} that mana is spent {{alt:to cast|on}} {f}, {b}"
                    )
                } else {
                    format!(
                        "{a}. When {{alt:that mana is spent|you spend this mana}} to cast {f}, {b}"
                    )
                }
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
                // "Its owner shuffles it into their library" (the object just named).
                if w == "~it" || w == "it" {
                    return format!("{{alt:the owner of {w}|its owner}} shuffles {it} into their library");
                }
                format!("the owner of {w} shuffles {it} into their library")
            }
            // "Each player shuffles their hand and graveyard into their library": each
            // card goes into its owner's library.
            Effect::ShuffleIntoLibrary {
                what: Sel::Union(v),
                library: PlayerRef::EachPlayer,
            } if v.iter().all(|x| {
                matches!(x, Sel::All(Filter::And(f))
                    if matches!(f.as_slice(), [Filter::InZone(_), Filter::OwnedBy(PlayerRel::Any)]))
            }) =>
            {
                let zones: Vec<String> = v
                    .iter()
                    .filter_map(|x| match x {
                        Sel::All(Filter::And(f)) => match f.first() {
                            Some(Filter::InZone(z)) => Some(zone_word(*z).to_string()),
                            _ => None,
                        },
                        _ => None,
                    })
                    .collect();
                format!(
                    "each player shuffles their {} into their library",
                    join_list(&zones, "and")
                )
            }
            // "Target player shuffles up to three target cards from their graveyard into
            // their library": the player is named first.
            Effect::ShuffleIntoLibrary { what, library }
                if matches!(library, PlayerRef::Target(_)) =>
            {
                let subj = self.player(library, Case::Subj);
                let w = self.sel(what, Case::Obj);
                format!("{subj} shuffles {w} into their library")
            }
            Effect::ShuffleIntoLibrary { what, library } => {
                let mut w = self.sel(what, Case::Obj);
                // "Shuffle the cards from your hand into your library" (all of them).
                if matches!(what, Sel::All(Filter::And(f)) if matches!(f.as_slice(), [Filter::InZone(ZoneKind::Hand), Filter::OwnedBy(_)]))
                    && w.ends_with(" hand")
                {
                    w = format!("{{alt:the cards from {w}|{w}}}");
                }
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
            Effect::DigStep(step) => self.dig_step(step),
            Effect::LookAtHand { who } => {
                let p = self.player(who, Case::Poss);
                // A card chosen from the hand looked at is "from it".
                self.revealed_hand = true;
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
                // Every card revealed is exiled, one of them the card found: "exile cards
                // from the top of your library until you exile a nonland card".
                let exiled = |d: &Destination| d.zone == ZoneKind::Exile && !d.face_down;
                if exiled(found_to) && exiled(rest_to) {
                    let vp = format!(
                        "exile cards from the top of {p} library until {} exile {f}",
                        if p == "your" { "you" } else { "they" }
                    );
                    return self.with_subject(who, &vp, false);
                }
                let until = format!(
                    "reveal cards from the top of {p} library until {} reveal {f}",
                    if p == "your" { "you" } else { "they" }
                );
                // Cards left where they are (in place, `FromTop`): the instructions that
                // follow say where "that card" and the rest go.
                let in_place = |d: &Destination| {
                    d.zone == ZoneKind::Library && matches!(d.position, LibraryPosition::FromTop(_))
                };
                if in_place(found_to) && in_place(rest_to) {
                    return self.with_subject(who, &until, false);
                }
                let found = self.destination_phrase(found_to, false, true);
                // "Put that card onto the battlefield and shuffle the rest into your
                // library."
                let you = matches!(who, PlayerRef::You) && !self.in_as_player;
                let rest = if rest_to.zone == ZoneKind::Library
                    && rest_to.position == LibraryPosition::Shuffled
                {
                    format!("{} the rest into {p} library", if you { "shuffle" } else { "shuffles" })
                } else {
                    let r = self.destination_phrase(rest_to, true, true);
                    // The cards revealed from your library are yours (a card goes only to
                    // its owner's library, CR 400.3): "on the bottom of your library".
                    let yours = match rest_to.position {
                        LibraryPosition::Bottom if you && rest_to.position_choice.is_empty() => {
                            Some("on the bottom of your library {opt:in any order}")
                        }
                        LibraryPosition::BottomRandom
                            if you && rest_to.position_choice.is_empty() =>
                        {
                            Some("on the bottom of your library in a random order")
                        }
                        _ => None,
                    }
                    .filter(|_| rest_to.zone == ZoneKind::Library);
                    let r = match yours {
                        Some(y) => format!("{{alt:{r}|{y}}}"),
                        None => r,
                    };
                    format!("the rest {r}")
                };
                // Another player reveals: that player puts the cards (the second sentence
                // names them again, "The player puts that card onto the battlefield").
                let put = if you {
                    "Put".to_string()
                } else {
                    "{alt:The player|That player} puts".to_string()
                };
                let vp = format!("{until}. {put} that card {found} and {rest}");
                self.with_subject(who, &vp, false)
            }
            Effect::ExtraTurnWith { who, at_start } => {
                let base = self.with_subject(who, "take an extra turn after this one", true);
                // Effects created as that turn starts: "During that turn, damage can't be
                // prevented. At the beginning of that turn's end step, you lose the game."
                if let Some(a) = self.extra_turn_effects(at_start) {
                    return format!("{base}. {a}");
                }
                let a = self.effect(at_start);
                format!("{base}. {a}")
            }
            Effect::ExtraCombat { after_this } => {
                if *after_this {
                    "{alt:after this phase, there is an additional combat phase|there is an additional combat phase after this phase}".into()
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
                    // "There is an additional combat phase after this phase" says the same;
                    // "there is an additional combat phase after this phase, followed by
                    // an additional main phase": the first part after this phase.
                    None if t.is_empty() => {
                        let mut forms = vec![
                            format!("after {after}, there is {list}"),
                            format!("there is {list} after {after}"),
                        ];
                        if let [first, rest @ ..] = names.as_slice() {
                            if !rest.is_empty() {
                                let r = join_list(rest, "followed by");
                                forms.push(format!(
                                    "there is {first} after {after}, followed by {r}"
                                ));
                            }
                        }
                        format!("{{alt:{}}}", forms.join("|"))
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
                if let Some(s) = self.next_step_if(trigger, body, *once) {
                    return s;
                }
                // A one-shot delayed trigger at a step: "at the beginning of the next end
                // step" (CR 603.7).
                let delay = if *once { self.delay_text(trigger) } else { None };
                let t = match trigger {
                    _ if delay.is_some() => delay.clone().unwrap_or_default(),
                    // "When you next cast an instant spell this turn, ..."
                    other if *once => {
                        let t = self.trigger_text(other);
                        match t.strip_prefix("whenever you cast ") {
                            // (A one-shot delayed trigger triggers the next time, CR 603.7b.)
                            Some(r) => format!("{{alt:when you next cast {r}|when you cast {r}}}"),
                            None => match t.strip_prefix("whenever you play ") {
                                Some(r) => format!("when you play {r}"),
                                // "When you next attack this turn, ..." (CR 603.7b).
                                None => match t.strip_prefix("whenever you attack") {
                                    Some(r) => format!("when you next attack{r}"),
                                    None => t,
                                },
                            },
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
                // A delayed trigger without targets of its own refers to the targets of the
                // ability that created it ("under that player's control", CR 603.7c).
                let outer = body.targets.is_empty()
                    && body.modal.is_none()
                    && !self.targets.is_empty()
                    && format!("{:?}", body.effect).contains("Target(");
                let b = if outer {
                    self.in_event_scope(|r| r.effect_sentences(&body.effect))
                } else {
                    self.in_event_scope(|r| r.body(body))
                };
                (self.self_salient, self.other_salient, self.trigger_is_self) = saved;
                // "Sacrifice it at end of combat": a time for one instruction may follow it.
                let at_step = delay.is_some();
                let b = lower_first(&b);
                let one = b.trim_end_matches('.');
                if at_step && !one.contains(['.', '"']) {
                    let b = one;
                    format!("{{alt:{t}, {b}|{b} {t}}}")
                } else if let Some((first, rest)) =
                    one.split_once(". ").filter(|(f, _)| at_step && !f.contains(['"', '{']))
                {
                    // "Return it to the battlefield at the beginning of your next upkeep.
                    // It gains haste.": the time may follow the first instruction.
                    format!("{{alt:{t}, {one}|{first} {t}. {rest}}}")
                } else {
                    format!("{t}, {b}")
                }
            }
            // Noting a choice for the linked ability: "Choose a creature card exiled with
            // ~." (Koh, the Face Stealer; the linked ability says "the last chosen card").
            Effect::NoteLinked {
                what: what @ Sel::Choose { chooser, .. },
                ..
            } => {
                let c = self.player(chooser, Case::Subj);
                let s = self.sel(&strip_chooser(what), Case::Obj);
                if c == "you" {
                    format!("choose {s}")
                } else {
                    format!("{c} chooses {s}")
                }
            }
            // Noting what the ability affected has no words of its own: the linked
            // ability says "that player" (CR 607.1).
            Effect::NoteLinked { .. } => String::new(),
            Effect::Reflexive { body } => {
                // A reflexive trigger without targets of its own refers to the targets of
                // the ability that created it ("destroy that permanent").
                let b = if body.targets.is_empty() && body.modal.is_none() && !self.targets.is_empty() {
                    self.in_event_scope(|r| r.effect_sentences(&body.effect))
                } else {
                    self.in_event_scope(|r| r.body(body))
                };
                format!("when you do, {}", lower_first(&b))
            }
            // "At the beginning of that combat, untap all creatures you control": a
            // reflexive trigger as the added combat phase begins (CR 603.7).
            Effect::DuringNext {
                part: NextTurnPart::AddedCombatPhase,
                effect,
                ..
            } if matches!(effect.as_ref(), Effect::Reflexive { .. }) => {
                let Effect::Reflexive { body } = effect.as_ref() else {
                    return String::new();
                };
                let b = if body.targets.is_empty()
                    && body.modal.is_none()
                    && !self.targets.is_empty()
                {
                    self.in_event_scope(|r| r.effect_sentences(&body.effect))
                } else {
                    self.in_event_scope(|r| r.body(body))
                };
                format!("at the beginning of that combat, {}", lower_first(&b))
            }
            Effect::DuringNext { who, part, effect } => {
                let p = self.player(who, super::players::Case::Poss);
                let when = match part {
                    NextTurnPart::Turn => format!("during {p} next turn"),
                    NextTurnPart::CombatPhase => format!("during {p} next combat phase"),
                    NextTurnPart::AddedCombatPhase => "during that combat phase".to_string(),
                };
                // "Target opponent skips all combat phases of their next turn."
                if let Effect::AddReplacement { def, .. } = &**effect {
                    if let ReplacementEvent::SkipStep {
                        step: StepKind::Combat,
                        ..
                    } = def.event
                    {
                        let subj = self.player(who, super::players::Case::Subj);
                        return format!("{subj} skips all combat phases of their next turn");
                    }
                }
                let e = self.effect(effect);
                // "Each opponent can't cast ... during that player's next turn."
                if matches!(who, PlayerRef::EachOpponent | PlayerRef::EachPlayer)
                    && *part == NextTurnPart::Turn
                {
                    if let Some(rest) = e.strip_prefix("{alt:that player|them} ") {
                        let subj = self.player(who, super::players::Case::Subj);
                        let rest = rest.replace(" this turn", "");
                        return format!("{subj} {rest} during that player's next turn");
                    }
                }
                let e = e
                    .replace(" this turn", "")
                    .replace(" this combat", "")
                    .replace(" until end of combat", "");
                // "that creature can't attack during its controller's next turn", "target
                // creature ... attacks during its controller's next combat phase if able".
                if let PlayerRef::ControllerOf(_) = who {
                    let when = when.replace(&format!("during {p} "), "during its controller's ");
                    return match e.strip_suffix(" if able") {
                        Some(r) => format!("{r} {when} if able"),
                        None => format!("{e} {when}"),
                    };
                }
                format!("{{alt:{when}, {e}|{e} {when}}}")
            }
            Effect::AtNext { step, effect } => {
                let s = match step {
                    TriggerStep::EndOfCombat => "at end of combat".to_string(),
                    TriggerStep::Upkeep => "at the beginning of the next turn's upkeep".to_string(),
                    other => format!("at the beginning of the next {}", self.step_name(*other)),
                };
                let e = self.effect(effect);
                if e.contains(['.', '"']) {
                    format!("{s}, {e}")
                } else {
                    format!("{{alt:{s}, {e}|{e} {s}}}")
                }
            }
            Effect::TokensEnterAttached { to, effect } => {
                let e = self.effect(effect);
                let t = self.sel(to, Case::Obj);
                format!("{e} attached to {t}")
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
            Effect::WithPlayTerms { terms, effect } => self.with_play_terms(terms, effect),
            // "Each player chooses up to two creatures they control, then sacrifices the
            // rest."
            Effect::KeepAndSacrificeRest {
                who,
                among,
                keep,
                up_to: true,
            } if keep
                .iter()
                .all(|k| format!("{k:?}") == format!("{among:?}")) =>
            {
                let w = self.player(who, Case::Subj);
                let n = self.noun(among, Num::Many);
                let k = number_word(keep.len() as i32);
                let vp = format!("choose up to {k} {n} they control, then sacrifice the rest");
                if w == "you" {
                    vp.replace(" they control", " you control")
                } else {
                    format!(
                        "{w} {}",
                        third_person(&vp).replacen(", then sacrifice ", ", then sacrifices ", 1)
                    )
                }
            }
            Effect::KeepAndSacrificeRest { up_to: true, .. } => {
                self.gap("keep up to some permanents, sacrifice the rest")
            }
            // "chooses five lands they control and sacrifices the rest".
            Effect::KeepAndSacrificeRest {
                who,
                among,
                keep,
                up_to: false,
            } if keep.len() > 1
                && keep
                    .iter()
                    .all(|k| format!("{k:?}") == format!("{among:?}")) =>
            {
                let w = self.player(who, Case::Subj);
                let n = self.noun(among, Num::Many);
                let k = number_word(keep.len() as i32);
                let vp = format!("choose {k} {n} you control, then sacrifice the rest");
                if w == "you" {
                    vp
                } else {
                    format!(
                        "{w} {}",
                        third_person(&vp.replace(" you control", " they control"))
                            .replacen(", then sacrifice ", ", then sacrifices ", 1)
                    )
                }
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
            } => match self.cast_up_to_one(e) {
                Some(c) => c,
                None => self.play_card("cast", who, what, *free, *optional),
            },
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
                // "You may cast an instant or sorcery spell from among them", "you may
                // play one of those cards": what may be played, and how many.
                let mut w = self.permission_object(&w);
                // "you may cast it from your graveyard as an Adventure" (CR 715.3): the
                // card the triggered ability's source became.
                if let Some(face) = self.play_terms.as_ref().and_then(|t| t.cast_face) {
                    if matches!(what, Sel::This) {
                        w.push_str(" {opt:from your graveyard}");
                    }
                    match face {
                        crate::object::FaceState::Half(1) => w.push_str(" as an Adventure"),
                        _ => w.push_str(&self.gap("permission term: cast face")),
                    }
                }
                let d = self.duration(duration);
                let f = if *free {
                    " without paying its mana cost"
                } else {
                    ""
                };
                let p = self.player(who, Case::Subj);
                let verb = self.permission_verb();
                // A permission for an exiled card lasts as long as it's that object (CR
                // 400.7): "for as long as it remains exiled".
                // "You may play that card until you exile another card with ~" (the
                // permission ends then; as it leaves exile at the latest, CR 400.7).
                let until_another = self.play_terms.as_ref().is_some_and(|t| t.until_another);
                let d = if matches!(duration, Duration::Permanent) && until_another {
                    String::new()
                } else if matches!(duration, Duration::Permanent)
                    && self.after_exile
                    && self.any_turn_permission
                {
                    "{opt:for as long as {alt:it remains|they remain} exiled}".to_string()
                } else if matches!(duration, Duration::Permanent) && self.after_exile {
                    "{alt:for as long as it remains exiled|for as long as they remain exiled}"
                        .to_string()
                } else {
                    d
                };
                // "You may cast that card this turn without paying its mana cost": the
                // duration before or after the way it's cast.
                if !f.is_empty() && !d.is_empty() && !d.contains('{') {
                    let f = f.trim_start();
                    return format!("{p} may {verb} {w} {{alt:{d} {f}|{f} {d}}}");
                }
                join_words(&[format!("{p} may {verb} {w}{f}"), d])
            }
            Effect::PreventDamage {
                to,
                amount,
                duration,
                combat_only,
                then,
            } => {
                // "For each 1 damage prevented this way, [instruction]."
                if let Some(e) = then {
                    let each = match e.as_ref() {
                        Effect::Repeat {
                            times: Value::EventAmount,
                            effect,
                        } => Some((**effect).clone()),
                        // A counted action on that many: once for each 1 damage.
                        e => {
                            let mut one = e.clone();
                            let n = match &mut one {
                                Effect::AddCounters { n, .. }
                                | Effect::Discard { n, .. }
                                | Effect::Mill { n, .. }
                                | Effect::GainLife { n, .. }
                                | Effect::LoseLife { n, .. } => Some(n),
                                Effect::CreateToken { count, .. } => Some(count),
                                _ => None,
                            };
                            match n {
                                Some(n) if matches!(n, Value::EventAmount) => {
                                    *n = Value::c(1);
                                    Some(one)
                                }
                                _ => None,
                            }
                        }
                    };
                    let saved = self.event_scope;
                    self.event_scope = true;
                    let inner = match &each {
                        Some(x) => {
                            let x = self.effect(x).replace(
                                " on it",
                                " on {alt:that creature|it}",
                            );
                            format!("for each 1 damage prevented this way, {x}")
                        }
                        None => self.effect(e),
                    };
                    self.event_scope = saved;
                    let head = self.effect(&Effect::PreventDamage {
                        to: to.clone(),
                        amount: amount.clone(),
                        duration: duration.clone(),
                        combat_only: *combat_only,
                        then: None,
                    });
                    return format!("{head}. {inner}");
                }
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
                // Secret council: "each player secretly votes for ..., then those votes are
                // revealed" (no starting player).
                if spec.secret && spec.action == KeywordAction::Vote {
                    if let Some((_, w)) = s.split_once(" votes for ") {
                        s = format!(
                            "each player secretly votes for {w}, then those votes are revealed"
                        );
                    }
                }
                // "Each suspected creature is no longer suspected" (CR 701.60a).
                if spec.undo && spec.action == KeywordAction::Suspect {
                    let w = self.sel(&spec.what, Case::Subj);
                    s = if w.contains(['{', '|', '}']) {
                        format!("{w} is no longer suspected")
                    } else {
                        format!("{{alt:{w} is no longer suspected|{w} becomes no longer suspected|have {w} become no longer suspected}}")
                    };
                } else if spec.undo && spec.action == KeywordAction::Goad {
                    // "Each creature you control is no longer goaded" (CR 701.15).
                    let w = self.sel(&spec.what, Case::Subj);
                    s = format!("{w} is no longer goaded");
                } else if spec.undo {
                    s = format!("{s} (undo)");
                    s = self.gap(format!("keyword action undo: {s}"));
                }
                // "Populate. The token enters tapped and attacking."
                if spec.action == KeywordAction::Populate && spec.tapped {
                    let how = if spec.attacking { "tapped and attacking" } else { "tapped" };
                    s = format!("{s}. the token enters {how}");
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
                // "one color word with another or one basic land type with another".
                let what = if matches!(words, crate::text_change::TextWords::ColorOrBasicLandType) {
                    "{alt:one color word or basic land type with another|one color word with another or one basic land type with another}".to_string()
                } else {
                    format!("one {kind} with another")
                };
                let mut s = join_words(&[
                    format!(
                        "change the text of {} by replacing all instances of {what}",
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
                let u = match until {
                    UntilEvent::TargetLeavesBattlefield(o) => {
                        let o = self.sel(&Sel::Target(*o), Case::Subj);
                        format!("{o} leaves the battlefield")
                    }
                    _ => until_event(until),
                };
                format!("exile {w} until {u}")
            }
            Effect::PhaseOutUntil { what, until } => {
                let w = self.sel(what, Case::Subj);
                let u = until_event(until);
                format!("{w} phases out until {u}")
            }
            // "Counter target spell. If that spell is countered this way, exile it instead of
            // putting it into its owner's graveyard."
            Effect::SelfReplace {
                replacement:
                    ReplacementDef {
                        event:
                            ReplacementEvent::ZoneChange {
                                filter,
                                from: Some(ZoneKind::Stack),
                                to: Some(ZoneKind::Graveyard),
                            },
                        action: ReplacementAction::MoveInstead(d),
                        ..
                    },
                effect,
            } if countered_this_way(effect, filter).is_some() => {
                let only = countered_this_way(effect, filter).flatten();
                let e = self.effect(effect);
                let which = match only {
                    Some(Filter::PermanentCard) => "a permanent spell".into(),
                    Some(f) => {
                        let n = self.noun_det(f, Det::A);
                        format!("{n} spell")
                    }
                    None => "that spell".into(),
                };
                let instead = if d.zone == ZoneKind::Exile {
                    let mut w = String::new();
                    for (k, n) in &d.with_counters {
                        let (c, _) = self.counted(n, &counter_name(k));
                        w.push_str(&format!(" with {c} on it"));
                    }
                    format!("exile it{w} instead of putting it into its owner's graveyard")
                } else {
                    let dest = self.destination_phrase(d, false, false);
                    format!("put it {dest} instead of into its owner's graveyard")
                };
                format!("{e}. If {which} is countered this way, {instead}")
            }
            // "Return target creature card from your graveyard to the battlefield. If a
            // Hero enters this way, it enters with two additional +1/+1 counters on it.":
            // a replacement of the instruction's own events (CR 614.1c, 614.12).
            Effect::SelfReplace {
                replacement:
                    ReplacementDef {
                        event: ReplacementEvent::EntersBattlefield(f),
                        action: ReplacementAction::EnterWithCounters(kind, n),
                        ..
                    },
                effect,
            } if !matches!(f, Filter::Any | Filter::Source) => {
                let e = self.effect(effect);
                let w = self.noun_det(f, Det::A);
                let (c, x) = self.counted(n, &counter_name(kind));
                format!(
                    "{e}. If {w} enters this way, it enters with {c} on it{}",
                    x.unwrap_or_default()
                )
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
                if let Some(s) = self
                    .each_spell_this_turn(filter, mods, expires)
                    .or_else(|| self.next_spell_marker(filter, mods, expires))
                {
                    return s;
                }
                let f = if matches!(filter, Filter::Any) {
                    "spell".to_string()
                } else {
                    self.noun(filter, Num::One)
                };
                let f = if f.contains("spell") {
                    f
                } else {
                    format!("{f} spell")
                };
                let d = match expires {
                    Duration::EndOfTurn | Duration::ThisTurn => " this turn",
                    _ => "",
                };
                // The spell's own abilities as it's cast: "can't be countered" (CR
                // 101.2), "can be cast as though it had flash" (flash matters only while
                // the card is cast, CR 702.8a).
                let vp = match mods.as_slice() {
                    [Modification::AddKeyword(k)] if k.kind == crate::keywords::KeywordKind::Flash => {
                        "can be cast as though it had flash".to_string()
                    }
                    [Modification::AddAbility(a)]
                        if matches!(
                            &a.kind,
                            AbilityKind::Static(st)
                                if st.condition.is_none()
                                    && matches!(
                                        st.effect,
                                        StaticEffect::Restriction(Restriction::CantBeCountered(Filter::Source))
                                    )
                        ) =>
                    {
                        "can't be countered".to_string()
                    }
                    // "the next spell you cast this turn costs {1} less to cast": the
                    // spell's own cost reduction (CR 601.2f).
                    [Modification::AddAbility(a)]
                        if matches!(
                            &a.kind,
                            AbilityKind::Static(st)
                                if st.condition.is_none()
                                    && matches!(
                                        &st.effect,
                                        StaticEffect::CostModifier(CostModifier {
                                            applies_to: CostTarget::ThisSpell,
                                            who: PlayerRel::You,
                                            change: CostChange::ReduceGeneric(Value::Const(_))
                                                | CostChange::IncreaseGeneric(Value::Const(_)),
                                        })
                                    )
                        ) =>
                    {
                        let AbilityKind::Static(StaticAbility {
                            effect: StaticEffect::CostModifier(cm),
                            ..
                        }) = &a.kind
                        else {
                            return self.gap("next spell cost");
                        };
                        match &cm.change {
                            CostChange::ReduceGeneric(Value::Const(n)) => {
                                format!("costs {{{n}}} less to cast")
                            }
                            CostChange::IncreaseGeneric(Value::Const(n)) => {
                                format!("costs {{{n}}} more to cast")
                            }
                            _ => return self.gap("next spell cost"),
                        }
                    }
                    // "The next noncreature spell you cast this turn has affinity for
                    // artifacts."
                    _ => self.mods_vp(mods, false),
                };
                format!("the next {f} you cast{d} {vp}")
            }
            Effect::CopySpellRetargeted {
                what,
                target,
                among,
            } => {
                let w = self.sel(what, Case::Obj);
                match (target, among) {
                    // "copy that spell for each other creature you control that the spell
                    // could target. Each copy targets a different one of those creatures."
                    // "for each other permanent or player the spell could target. Each
                    // copy targets a different one of those permanents and players."
                    (None, Some(Filter::Or(v)))
                        if v.iter().any(|x| matches!(x, Filter::Custom(n) if n == crate::copy_rules::ANY_PLAYER)) =>
                    {
                        let w = if matches!(what, Sel::TriggerSpell) {
                            "{alt:that spell|that spell or ability}".to_string()
                        } else {
                            w
                        };
                        let objs: Vec<Filter> = v
                            .iter()
                            .filter(|x| !matches!(x, Filter::Custom(n) if n == crate::copy_rules::ANY_PLAYER))
                            .cloned()
                            .collect();
                        let objs = Filter::Or(objs);
                        let n = self.noun(&objs, Num::One);
                        let ns = self.noun(&objs, Num::Many);
                        format!(
                            "copy {w} for each other {n} or player {{alt:the spell|the spell or ability|that spell|that spell or ability}} could target. Each copy targets a different one of those {ns} and players"
                        )
                    }
                    (None, Some(f)) => {
                        let w = if matches!(what, Sel::TriggerSpell) {
                            "that spell".to_string()
                        } else {
                            w
                        };
                        let n = self.noun(f, Num::One);
                        // "those creatures": the kind, without "you control".
                        let ns = self.noun(f, Num::Many);
                        let ns = ns.strip_suffix(" you control").unwrap_or(&ns).to_string();
                        format!(
                            "copy {w} for each other {n} {{alt:that the spell|that spell}} could target. Each copy targets a different one of those {ns}"
                        )
                    }
                    (None, None) => {
                        format!("copy {w} for each other object or player it could target")
                    }
                    (Some(t), _) => {
                        let t = self.sel(t, Case::Obj);
                        format!("copy {w}. The copy targets {t}")
                    }
                }
            }
            Effect::CopyCard { what, named } => match named {
                // "copy the enchanted instant card" (an Aura enchanting a card in a
                // graveyard).
                None if matches!(what, Sel::AttachedTo) && self.enchanted_card(true).is_some() => {
                    let w = self.enchanted_card(true).unwrap_or_default();
                    format!("copy {w}")
                }
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
                    // One player: "an opponent" is the opponent of your choice.
                    let s = self.some_player(separator);
                    let vp = format!("separate {w} into a face-down pile and a face-up pile");
                    if s == "you" {
                        vp
                    } else {
                        format!("{s} {}", third_person(&vp))
                    }
                }
                crate::piles::PileAction::Separate {
                    what,
                    separator,
                    count,
                    face_down,
                    own,
                } => {
                    let w = self.sel(what, Case::Obj);
                    let s = self.some_player(separator);
                    let n = if *count == 3 { "three" } else { "two" };
                    let piles = if *face_down {
                        format!("{n} face-down piles")
                    } else {
                        format!("{n} piles")
                    };
                    let _ = own;
                    if s == "you" {
                        format!("separate {w} into {piles}")
                    } else {
                        format!("{s} separates {w} into {piles}")
                    }
                }
                crate::piles::PileAction::Choose {
                    chooser,
                    picked_by,
                    of,
                    at_random,
                } => {
                    let c = self.some_player(chooser);
                    let piles = if of.is_some() { "their piles" } else { "those piles" };
                    if *at_random {
                        format!("{c} chooses one of {piles} at random")
                    } else if picked_by.is_some() {
                        format!("one of {piles} is chosen by {c} of their choice")
                    } else if c == "you" {
                        format!("choose one of {piles}")
                    } else {
                        format!("{c} chooses one of {{alt:those|the}} piles")
                    }
                }
                crate::piles::PileAction::ExileAsPile {
                    what,
                    pile,
                    face_down,
                    ..
                } => {
                    let w = self.sel(what, Case::Obj);
                    let kind = if *face_down { "face-down" } else { "face-up" };
                    let a = if *pile > 0 { "another" } else { "a" };
                    format!("exile {w} in {a} {kind} pile")
                }
                crate::piles::PileAction::Look { what } => {
                    format!("look at {}", self.sel(what, Case::Obj))
                }
                crate::piles::PileAction::TurnFaceUp { chooser } => {
                    let c = self.some_player(chooser);
                    if c == "you" {
                        "turn a pile of your choice face up".to_string()
                    } else {
                        format!("{c} turns a pile face up")
                    }
                }
                crate::piles::PileAction::Recall { .. } => String::new(),
                crate::piles::PileAction::Shuffle { what } => {
                    format!("shuffle {}", self.sel(what, Case::Obj))
                }
                crate::piles::PileAction::Top { from } => {
                    format!("the top card of {}", self.sel(from, Case::Obj))
                }
                crate::piles::PileAction::PutOnLibrary { what } => {
                    format!(
                        "put {} on top of your library",
                        self.sel(what, Case::Obj)
                    )
                }
                crate::piles::PileAction::SearchPile { what } => format!(
                    "search {} for a card, put it into your hand, then shuffle the rest of that pile into your library",
                    self.sel(what, Case::Obj)
                ),
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
                // "Investigate that many times."
                if s.starts_with("{alt:that much") || s == "that many" {
                    return "that many times".into();
                }
                if Self::is_simple(other) {
                    format!("{s} times")
                } else {
                    format!("X times, where X is {s}")
                }
            }
        }
    }

    /// A sequence of effects, merging clauses with the same subject.
    /// CR 701.41a: "Support N" on a permanent is "put a +1/+1 counter on each of up to N
    /// other target creatures", on an instant or sorcery "... up to N target creatures":
    /// N, when `AddCounters` is exactly that.
    fn support_n(&self, what: &Sel, kind: &str, n: &Value) -> Option<String> {
        let Sel::Target(i) = what else { return None };
        if kind != crate::types::counters::PLUS1 || !matches!(n, Value::Const(1)) {
            return None;
        }
        let t = self.targets.get(*i as usize)?;
        let spell = self.info.card_types.contains(CardType::Instant)
            || self.info.card_types.contains(CardType::Sorcery);
        let want = if spell {
            Filter::creature()
        } else {
            Filter::and(vec![Filter::creature(), Filter::Other])
        };
        let ok = matches!(t.min, Value::Const(0))
            && matches!(&t.what, TargetKind::Object(f) if format!("{f:?}") == format!("{want:?}"));
        match &t.max {
            Value::Const(n) if ok => Some(number_word(*n)),
            Value::X if ok => Some("X".into()),
            _ => None,
        }
    }

    /// "[Replacement effect] this turn" created by a resolving spell or ability; see the
    /// `Effect::AddReplacement` arm of `effect_inner`.
    fn add_replacement(
        &mut self,
        def: &ReplacementDef,
        duration: &Duration,
        uses: &Option<u32>,
    ) -> String {
        // Crafty Cutpurse.
        if let (
            ReplacementEvent::CreateTokens(PlayerFilter::Opponent),
            ReplacementAction::EnterUnderControl(PlayerRef::You),
            Duration::EndOfTurn,
            None,
        ) = (&def.event, &def.action, duration, uses)
        {
            return "each token that would be created under an opponent's control this turn is created under your control instead".into();
        }
        if *uses == Some(1) {
            if let Some(s) = self.attempt(|r| r.next_time_damage(def, duration)) {
                return s;
            }
        }
        let s = self.replacement(def, *uses);
        let d = self.duration(duration);
        // "All damage that would be dealt to target creature this turn is dealt to
        // you instead."
        if (d == "this turn" || d == "until end of turn")
            && (s.starts_with("all ") || s.starts_with("the next "))
            && s.contains(" that would be dealt to ")
            && s.contains(" is dealt to ")
        {
            return s.replacen(" is dealt to ", " this turn is dealt to ", 1);
        }
        // "If it would die this turn, exile it instead."
        if d == "until end of turn" && s.starts_with("if ") && s.contains(" would die,") {
            return s.replacen(" would die,", " would die this turn,", 1);
        }
        // "If any source would deal 1 or more damage to a permanent or player this
        // turn, it deals 2 damage to that permanent or player instead."
        // "If a red source you control would deal damage to a permanent or player
        // this turn, it deals that much damage plus 2 ... instead."
        if d == "until end of turn"
            && uses.is_none()
            && matches!(def.event, ReplacementEvent::Damage { .. })
            && s.starts_with("if ")
            && s.contains(" would deal damage")
            && s.contains(", it deals ")
        {
            if let Some((ev, then)) = s.split_once(", it deals ") {
                return format!("{ev} this turn, it deals {then}");
            }
        }
        if d == "until end of turn" && s.starts_with("if ") && s.contains(" or more damage to ") {
            if let Some((ev, then)) = s.split_once(", ") {
                return format!("{ev} this turn, {then}");
            }
        }
        // "The next time you would draw a card this turn, ... instead": one use.
        if *uses == Some(1) && (d == "until end of turn" || d == "this turn") {
            if let Some(rest) = s.strip_prefix("if ") {
                if let Some((ev, then)) = super::next_time::split_outside_braces(rest, ", ") {
                    let one_alt = ev.starts_with("{alt:")
                        && ev.ends_with('}')
                        && ev.matches('{').count() == 1;
                    // "the next time {alt:that player|they} would draw a card": a player
                    // named first.
                    let alt_subject = ev.starts_with("{alt:")
                        && ev.matches('{').count() == 1
                        && ev
                            .find('}')
                            .is_some_and(|i| ev[i + 1..].starts_with(" would "));
                    if !ev.contains('{') || one_alt || alt_subject {
                        // "..., instead they exile the top card of their library. They may
                        // play it this turn.": "instead" first, before the instructions.
                        if let Some(body) = then.strip_suffix(" instead") {
                            let body = match body.strip_prefix("{opt:") {
                                Some(r) => r.split_once("} ").map_or(body, |(_, b)| b),
                                None => body,
                            };
                            return format!(
                                "the next time {ev} this turn, {{alt:{then}|instead {body}}}"
                            );
                        }
                        return format!("the next time {ev} this turn, {then}");
                    }
                }
            }
        }
        join_words(&[s, d])
    }

    /// "The next time a source of your choice would deal damage to you this turn, prevent
    /// that damage", "Prevent all damage a source of your choice would deal this turn":
    /// a source chosen as the effect resolves (CR 609.7a: any object, wherever it is)
    /// and a damage replacement or prevention effect about it.
    fn chosen_source_damage(&mut self, choose: &Effect, next: &Effect) -> Option<String> {
        let Effect::ChooseSource {
            who: PlayerRef::You,
            filter,
            var,
        } = choose
        else {
            return None;
        };
        let Effect::AddReplacement {
            def,
            duration: Duration::EndOfTurn | Duration::ThisTurn,
            uses,
        } = next
        else {
            return None;
        };
        let ReplacementEvent::Damage {
            source,
            to_players,
            to_objects,
            combat_only: false,
        } = &def.event
        else {
            return None;
        };
        if !matches!(source, Filter::In(s) if matches!(s.as_ref(), Sel::Var(x) if x == var)) {
            return None;
        }
        let src = if matches!(filter, Filter::Any) {
            "a source of your choice".to_string()
        } else {
            let saved = self.default_head;
            self.default_head = Some("source");
            let n = self.noun_det(filter, Det::A);
            self.default_head = saved;
            format!("{n} of your choice")
        };
        let mut to: Vec<String> = Vec::new();
        // "would deal damage to any target": the target, a player or an object.
        let one_target = match (to_players, to_objects) {
            (Some(PlayerFilter::Ref(p)), Some(Filter::In(s))) => match (p.as_ref(), s.as_ref()) {
                (PlayerRef::Target(i), Sel::Target(j)) if i == j => Some(*i),
                _ => None,
            },
            _ => None,
        };
        match to_players {
            None => {}
            _ if one_target.is_some() => {}
            Some(PlayerFilter::You) => to.push("you".into()),
            Some(PlayerFilter::Any) if matches!(to_objects, Some(Filter::Any)) => {}
            Some(_) => return None,
        }
        match to_objects {
            None => {}
            Some(_) if one_target.is_some() => {
                to.push(self.sel(&Sel::Target(one_target.unwrap_or_default()), Case::Obj))
            }
            Some(Filter::Any) if matches!(to_players, Some(PlayerFilter::Any)) => {}
            Some(Filter::Source) => to.push(self.me()),
            Some(Filter::In(s)) => to.push(self.sel(s, Case::Obj)),
            Some(f) => to.push(self.noun_det(f, Det::A)),
        }
        let to_s = if to.is_empty() {
            String::new()
        } else {
            format!(" to {}", join_list(&to, "and"))
        };
        let then = match &def.action {
            ReplacementAction::Prevent => "prevent that damage".to_string(),
            ReplacementAction::PreventAndThen(None, e)
                if matches!(
                    e.as_ref(),
                    Effect::GainLife {
                        who: PlayerRef::You,
                        n: Value::EventAmount
                    }
                ) =>
            {
                "prevent that damage. You gain life equal to the damage prevented this way"
                    .to_string()
            }
            // "prevent half that damage, rounded down" (CR 615.1a).
            ReplacementAction::PreventPortion(Value::Div(n, 2, up))
                if matches!(n.as_ref(), Value::EventAmount) =>
            {
                let r = if *up { "up" } else { "down" };
                format!("prevent half that damage, rounded {r}")
            }
            // "If damage is prevented this way, ~ deals that much damage to that source's
            // controller" (the source of the damage, CR 614.1a).
            ReplacementAction::PreventAndThen(
                None,
                e,
            ) if matches!(e.as_ref(), Effect::DealDamage {
                    source: Sel::This,
                    amount: Value::EventAmount,
                    to: Sel::Players(PlayerRef::ControllerOf(o)),
                } if matches!(o.as_ref(), Sel::TriggerOtherObject)) =>
            {
                let m = self.me();
                format!("prevent that damage. If damage is prevented this way, {m} deals that much damage to that source's controller")
            }
            // "Exile cards from the top of your library equal to the damage prevented this
            // way."
            ReplacementAction::PreventAndThen(None, e)
                if matches!(e.as_ref(), Effect::Exile {
                    what: Sel::TopOfLibrary(PlayerRef::You, Value::EventAmount),
                    face_down: false,
                    ..
                }) =>
            {
                "prevent that damage. Exile cards from the top of your library equal to the damage prevented this way".to_string()
            }
            // "that source's controller": the damage event's source (CR 614.1a).
            ReplacementAction::Redirect(t) => {
                let t = self.in_event_scope(|r| r.sel(t, Case::Obj));
                format!("that damage is dealt to {t} instead")
            }
            a => self.damage_then(a)?,
        };
        Some(match (uses, &def.action) {
            (Some(1), _) => {
                format!("the next time {src} would deal damage{to_s} this turn, {then}")
            }
            (None, ReplacementAction::Prevent) => format!(
                "{{alt:prevent all damage {src} would deal{to_s} this turn|prevent all damage that would be dealt{to_s} this turn by {src}}}"
            ),
            (None, ReplacementAction::Redirect(_)) => {
                format!("all damage that would be dealt this turn{to_s} by {src} is dealt to {} instead", then.trim_start_matches("that damage is dealt to ").trim_end_matches(" instead"))
            }
            _ => return None,
        })
    }

    pub(crate) fn seq(&mut self, v: &[Effect]) -> String {
        let mut parts: Vec<String> = Vec::new();
        let mut outcomes = Vec::new();
        let mut i = 0;
        while i < v.len() {
            if let Some((s, used)) = self.guess_seq(&v[i..]) {
                parts.push(s);
                i += used;
                continue;
            }
            // "... choose one of those creatures. Copy that spell. The copy targets the
            // chosen creature." (the creatures the intervening "if" is about).
            if let (
                Some(Effect::Store {
                    var,
                    sel:
                        Sel::Choose {
                            chooser: PlayerRef::You,
                            filter,
                            count: Value::Const(1),
                            up_to: false,
                            ..
                        },
                }),
                Some(Effect::CopySpellRetargeted {
                    what: Sel::TriggerSpell,
                    target: Some(Sel::Var(t)),
                    among: None,
                }),
            ) = (v.get(i), v.get(i + 1))
            {
                if t == var
                    && matches!(filter, Filter::And(f) if f.iter().any(|x| matches!(x, Filter::Custom(n) if n == crate::copy_rules::TRIGGER_SPELL_COULD_TARGET)))
                    && self
                        .intervening_exists
                        .as_ref()
                        .is_some_and(|g| format!("{g:?}") == format!("{filter:?}"))
                {
                    let types: Vec<Filter> = match filter {
                        Filter::And(f) => f
                            .iter()
                            .filter(|x| matches!(x, Filter::Type(_) | Filter::Permanent))
                            .cloned()
                            .collect(),
                        _ => Vec::new(),
                    };
                    let ty = Filter::and(types);
                    let many = self.noun(&ty, Num::Many);
                    let one = self.noun(&ty, Num::One);
                    parts.push(format!(
                        "choose one of those {many}. Copy that spell. The copy targets the chosen {one}"
                    ));
                    i += 2;
                    continue;
                }
            }
            // "Sacrifice X Foods." with X chosen as they're sacrificed (CR 107.3f,
            // `patterns/sacrifice_x.rs`).
            if let (
                Effect::Store {
                    var,
                    sel:
                        Sel::Choose {
                            chooser: PlayerRef::You,
                            filter,
                            up_to: true,
                            ..
                        },
                },
                Some(Effect::SetX {
                    value: Value::CountSel(x),
                }),
                Some(Effect::SacrificeObjects { what: Sel::Var(w) }),
            ) = (&v[i], v.get(i + 1), v.get(i + 2))
            {
                if matches!(x.as_ref(), Sel::Var(a) if a == var) && w == var {
                    let what = self.noun(filter, Num::Many);
                    let what = what.replacen(" you control", " {opt:you control}", 1);
                    parts.push(format!("sacrifice X {what}"));
                    i += 3;
                    continue;
                }
            }
            if let Some((s, used)) = self.seq_extra(&v[i..]) {
                parts.push(s);
                i += used;
                continue;
            }
            if let Some((s, used)) = self
                .seq_per_owner(&v[i..])
                .or_else(|| self.seq_rest_not_cast(v, i))
                .or_else(|| self.rest_after_cast(v, i))
                .or_else(|| self.same_name_search(v, i))
                .or_else(|| self.named_seq(v, i))
            {
                parts.push(s);
                i += used;
                continue;
            }
            // "Roll a d20. Create X Treasure tokens, where X is the result.": the number
            // X stands for is said where X is first used (CR 107.3).
            if let Effect::SetX { value } = &v[i] {
                if let Some(s) = self.x_defined_later(value, &v[i + 1..]) {
                    parts.push(s);
                    break;
                }
            }
            // "You and target opponent each draw a card."
            if let (Some(first), Some(Effect::AsPlayer { who: other, effect })) =
                (v.get(i), v.get(i + 1))
            {
                if format!("{first:?}") == format!("{:?}", effect.as_ref())
                    && matches!(other, PlayerRef::Target(_))
                    && !format!("{first:?}").contains("Target")
                {
                    let vp = self.effect(first);
                    // (Alternative wordings of one instruction are fine: "each secretly
                    // choose {alt:a number from 1 to 3|1, 2, or 3}".)
                    let alts_only = vp.starts_with("secretly ");
                    if !vp.contains('.')
                        && (!vp.contains('{') || alts_only)
                        && !vp.starts_with("you ")
                    {
                        let p = self.player(other, Case::Subj);
                        parts.push(format!("you and {p} each {vp}"));
                        i += 2;
                        continue;
                    }
                    parts.push(vp);
                    i += 1;
                    continue;
                }
            }
            // "Discard a card. When you discard a card this way, ..." (a reflexive trigger,
            // CR 603.12).
            if let Some(Effect::If {
                cond: Condition::And(cs),
                then,
                otherwise,
            }) = v.get(i)
            {
                // (Drawing a card doesn't change what was discarded.)
                let after_discard = v[..i]
                    .iter()
                    .rev()
                    .find(|e| !matches!(e, Effect::Draw { .. }))
                    .is_some_and(|e| match e {
                        Effect::Discard {
                            who: PlayerRef::You,
                            ..
                        } => true,
                        Effect::May { effect, .. } => {
                            matches!(
                                effect.as_ref(),
                                Effect::Discard {
                                    who: PlayerRef::You,
                                    ..
                                }
                            )
                        }
                        _ => false,
                    });
                let what = match cs.as_slice() {
                    [Condition::PrevHappened, Condition::SelNonEmpty(Sel::Var(x))]
                        if *x == vars::IT =>
                    {
                        Some("a card".to_string())
                    }
                    [Condition::PrevHappened, Condition::SelNonEmpty(Sel::Var(x)), Condition::SelMatches(Sel::Var(y), f)]
                        if *x == vars::IT && *y == vars::IT =>
                    {
                        Some(self.noun_det(f, Det::A))
                    }
                    _ => None,
                };
                if let (true, true, Some(what)) = (
                    after_discard,
                    matches!(otherwise.as_ref(), Effect::Noop),
                    what,
                ) {
                    if let Effect::Reflexive { body } = then.as_ref() {
                        let b = self.in_event_scope(|r| r.body(body));
                        parts.push(format!(
                            "when you discard {what} this way, {}",
                            lower_first(&b)
                        ));
                        i += 1;
                        continue;
                    }
                }
            }
            // "Prevent all combat damage that would be dealt to and dealt by that creature
            // this turn."
            if let (
                Some(Effect::AddReplacement {
                    def: a,
                    duration: da,
                    uses: None,
                }),
                Some(Effect::AddReplacement {
                    def: b,
                    duration: db,
                    uses: None,
                }),
            ) = (v.get(i), v.get(i + 1))
            {
                if let (
                    ReplacementEvent::Damage {
                        source: Filter::Any,
                        to_players: None,
                        to_objects: Some(Filter::In(to)),
                        combat_only: ca,
                    },
                    ReplacementEvent::Damage {
                        source: Filter::In(by),
                        to_players: Some(PlayerFilter::Any),
                        to_objects: Some(Filter::Any),
                        combat_only: cb,
                    },
                ) = (&a.event, &b.event)
                {
                    if same_sel(to, by)
                        && ca == cb
                        && matches!(a.action, ReplacementAction::Prevent)
                        && matches!(b.action, ReplacementAction::Prevent)
                        && format!("{da:?}") == format!("{db:?}")
                    {
                        let kind = if *ca { "combat damage" } else { "damage" };
                        let w = self.sel(to, Case::Obj);
                        let d = self.duration(da);
                        parts.push(join_words(&[
                            format!("prevent all {kind} that would be dealt to and dealt by {w}"),
                            d,
                        ]));
                        i += 2;
                        continue;
                    }
                }
            }
            // "Put that pile into your hand and the other into your graveyard."
            if let (
                Some(Effect::Move {
                    what: Sel::Var(a),
                    to: ta,
                }),
                Some(Effect::Move {
                    what: Sel::Var(b),
                    to: tb,
                }),
            ) = (v.get(i), v.get(i + 1))
            {
                if *a == crate::piles::CHOSEN && *b == crate::piles::OTHER {
                    let da = self.destination_phrase(ta, false, true);
                    let db = self.destination_phrase(tb, false, true);
                    // "Choose one of those piles. Put that pile ..." is "put one pile ..."
                    // when you choose.
                    let you_chose = i > 0
                        && matches!(&v[i - 1], Effect::Piles(p) if matches!(p.as_ref(), crate::piles::PileAction::Choose { chooser: PlayerRef::You, at_random: false, .. }));
                    if you_chose
                        && parts
                            .last()
                            .is_some_and(|p| p == "choose one of those piles")
                    {
                        parts.pop();
                        parts.push(format!("put one pile {da} and the other {db}"));
                    } else {
                        parts.push(format!("put that pile {da} and the other {db}"));
                    }
                    i += 2;
                    continue;
                }
            }
            // "Look at the top four cards ... You may reveal a land card from among them and
            // put it into your hand. ... If you didn't put a card into your hand this way,
            // proliferate.": whether the dig took a card (CR 608.2c).
            if let (
                Some(dig @ Effect::Dig { take, take_to, .. }),
                Some(Effect::If {
                    cond: Condition::Not(c),
                    then,
                    otherwise,
                }),
            ) = (v.get(i), v.get(i + 1))
            {
                if take_to.zone == ZoneKind::Hand
                    && !matches!(take, Value::Const(0))
                    && matches!(c.as_ref(), Condition::PrevHappened)
                    && matches!(otherwise.as_ref(), Effect::Noop)
                {
                    let d = self.effect(dig);
                    let t = self.effect(then);
                    parts.push(d);
                    parts.push(format!(
                        "{{alt:if you don't|if you didn't put a card into your hand this way}}, {t}"
                    ));
                    i += 2;
                    continue;
                }
            }
            // "secretly choose an opponent" (`choices.rs`).
            if let (Some(Effect::Custom(n)), Some(next)) = (v.get(i), v.get(i + 1)) {
                if n == crate::choices::SECRET_NEXT {
                    let e = self.effect(next);
                    parts.push(format!("secretly {e}"));
                    i += 2;
                    continue;
                }
            }
            // "An opponent gains control of ~": an opponent you choose, named once.
            if let (
                Some(Effect::Choose {
                    who: PlayerRef::You,
                    kind: ChoiceKind::Opponent,
                }),
                Some(next),
            ) = (v.get(i), v.get(i + 1))
            {
                if !format!("{:?}", &v[i + 2..]).contains("ChosenOpponent") {
                    let n = self.effect(next);
                    // The one mention of the chosen opponent: "an opponent gains control
                    // of ~", "look at an opponent's hand" (as "clash with an opponent"
                    // means "choose an opponent. You and that opponent each clash", CR
                    // 701.30b).
                    let chosen = [
                        ("the chosen {alt:opponent|player}", "an opponent"),
                        ("the chosen {alt:opponent's|player's}", "an opponent's"),
                        ("the chosen opponent", "an opponent"),
                    ];
                    let mentions: usize = chosen.iter().map(|(c, _)| n.matches(*c).count()).sum();
                    if mentions == 1 && !n.contains("chosen player") {
                        if let Some((c, r)) = chosen.iter().find(|(c, _)| n.contains(*c)) {
                            parts.push(n.replacen(c, r, 1));
                            i += 2;
                            continue;
                        }
                    }
                    parts.push("choose an opponent".into());
                    parts.push(n);
                    i += 2;
                    continue;
                }
            }
            if let (Some(a), Some(b)) = (v.get(i), v.get(i + 1)) {
                if let Some(s) = self.chosen_source_damage(a, b) {
                    parts.push(s);
                    i += 2;
                    continue;
                }
            }
            // "Target land becomes the basic land type of your choice until end of turn."
            if let (
                Some(Effect::Choose {
                    who: PlayerRef::You,
                    kind: ChoiceKind::BasicLandType,
                }),
                Some(Effect::Modify {
                    what,
                    mods,
                    duration,
                }),
            ) = (v.get(i), v.get(i + 1))
            {
                if matches!(mods.as_slice(), [Modification::SetChosenBasicLandType])
                    && !format!("{:?}", &v[i + 2..]).contains("Chosen")
                {
                    let w = self.sel(what, Case::Subj);
                    let d = self.duration(duration);
                    let s = format!("{w} becomes the basic land type of your choice");
                    let b = format!("choose a basic land type. {w} becomes that type");
                    let both = either_form(join_words(&[s, d.clone()]), join_words(&[b, d]));
                    parts.push(both);
                    i += 2;
                    continue;
                }
            }
            // "Target land becomes the basic land type of your choice in addition to its
            // other types until end of turn", "~ becomes the creature type of your choice
            // in addition to its other types".
            if let (
                Some(Effect::Choose {
                    who: PlayerRef::You,
                    kind: kind @ (ChoiceKind::BasicLandType | ChoiceKind::CreatureType),
                }),
                Some(Effect::Modify {
                    what,
                    mods,
                    duration,
                }),
            ) = (v.get(i), v.get(i + 1))
            {
                if matches!(mods.as_slice(), [Modification::AddChosenType])
                    && !format!("{:?}", &v[i + 2..]).contains("Chosen")
                {
                    let w = self.sel(what, Case::Subj);
                    let d = self.duration(duration);
                    let k = if matches!(kind, ChoiceKind::BasicLandType) {
                        "basic land type"
                    } else {
                        "creature type"
                    };
                    let s = format!(
                        "{w} becomes the {k} of your choice in addition to its other types"
                    );
                    parts.push(join_words(&[s, d]));
                    i += 2;
                    continue;
                }
            }
            // "~ becomes the creature type of your choice until end of turn."
            if let (
                Some(Effect::Choose {
                    who: PlayerRef::You,
                    kind: ChoiceKind::CreatureType,
                }),
                Some(Effect::Modify {
                    what,
                    mods,
                    duration,
                }),
            ) = (v.get(i), v.get(i + 1))
            {
                if matches!(
                    mods.as_slice(),
                    [
                        Modification::RemoveAllCreatureTypes,
                        Modification::AddChosenType
                    ]
                ) && !format!("{:?}", &v[i + 2..]).contains("Chosen")
                {
                    let w = self.sel(what, Case::Subj);
                    let d = self.duration(duration);
                    let s = format!("{w} becomes the creature type of your choice");
                    parts.push(join_words(&[s, d]));
                    i += 2;
                    continue;
                }
            }
            // "Reveal a card from your hand", "If a land card was milled this way, ..."
            // (`outcomes.rs`).
            if let Some((n, s)) = self.outcome_seq_part(v, i, &mut outcomes) {
                if !s.is_empty() {
                    parts.push(s);
                }
                i += n;
                continue;
            }
            // "Discard any number of cards.", "Tap any number of untapped Gates you
            // control.": a choice of objects remembered for the one instruction that acts
            // on all of them right away.
            if let (
                Some(Effect::Store {
                    sel:
                        Sel::Choose {
                            chooser: PlayerRef::You,
                            ..
                        },
                    var,
                }),
                Some(next),
            ) = (v.get(i), v.get(i + 1))
            {
                let uses_var = format!("{next:?}").contains(&format!("{:?}", Sel::Var(*var)));
                if uses_var {
                    let saved = (self.var_defs.len(), self.gaps.len());
                    let chosen = self.effect(&v[i]);
                    let act = self.effect(next);
                    let simple = act
                        .split_once(' ')
                        .filter(|(verb, rest)| {
                            matches!(*rest, "it" | "them")
                                && verb.chars().all(|c| c.is_ascii_lowercase())
                        })
                        .map(|(verb, _)| verb.to_string());
                    let what = chosen
                        .strip_prefix("choose ")
                        .filter(|w| w.starts_with("any number of ") || w.starts_with("up to "));
                    match (what, simple) {
                        (Some(what), Some(verb)) => {
                            // A player discards cards from their own hand (CR 701.9a).
                            let mut what = what.to_string();
                            if verb == "discard" || verb == "discards" {
                                for h in [
                                    " in your hand",
                                    " you own in hands",
                                    " in their hand",
                                    " they own in hands",
                                ] {
                                    what = what.replace(h, "");
                                }
                            }
                            // A player sacrifices only permanents they control (CR
                            // 701.21a): "sacrifice any number of lands".
                            if verb == "sacrifice" {
                                what = what.replacen(" you control", " {opt:you control}", 1);
                            }
                            parts.push(format!("{verb} {what}"));
                            i += 2;
                            continue;
                        }
                        _ => {
                            self.var_defs.truncate(saved.0);
                            self.gaps.truncate(saved.1);
                            parts.push(chosen);
                            parts.push(act);
                            i += 2;
                            continue;
                        }
                    }
                }
            }
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
                let var_n = *var;
                let var = format!("{:?}", Sel::Var(*var));
                let names = |n: &Effect| {
                    custom_extra::names_it(n, var_n) || {
                        let n = format!("{n:?}");
                        n.contains(&named) || n.contains(&var)
                    }
                };
                // (A group remembered with others, named by a later instruction.)
                let later = v[i + 1..]
                    .iter()
                    .take_while(|e| matches!(e, Effect::Store { .. }));
                let named_later = v[i + 1..].iter().skip(later.count()).take(1).any(names);
                // The cards in graveyards before an instruction, named by the later
                // "that weren't put there this way" it's remembered for.
                let put_there = var_n == crate::oracle::patterns::referents_values_4::PUT_THERE;
                if !next.is_some_and(names) && !named_later && !put_there {
                    parts.push(self.gap("a remembered group no instruction names"));
                }
            }
            // "Two target players each draw a card": two different target players (CR
            // 115.3), each doing the same.
            if let (
                Some(Effect::Draw {
                    who: PlayerRef::Target(a),
                    n: n1,
                }),
                Some(Effect::Draw {
                    who: PlayerRef::Target(b),
                    n: n2,
                }),
            ) = (v.get(i), v.get(i + 1))
            {
                let one_player = |t: Option<&TargetSpec>| {
                    t.is_some_and(|t| {
                        matches!(t.what, TargetKind::Player(PlayerFilter::Any))
                            && matches!(t.min, Value::Const(1))
                            && matches!(t.max, Value::Const(1))
                    })
                };
                let (ta, tb) = (self.targets.get(*a as usize), self.targets.get(*b as usize));
                let fresh = |r: &Self, x: u8| r.introduced.get(x as usize) == Some(&false);
                if one_player(ta)
                    && one_player(tb)
                    && tb.is_some_and(|t| t.distinct_from == vec![*a])
                    && format!("{n1:?}") == format!("{n2:?}")
                    && fresh(self, *a)
                    && fresh(self, *b)
                {
                    self.introduced[*a as usize] = true;
                    self.introduced[*b as usize] = true;
                    let d = match n1 {
                        Value::Const(1) => "a card".to_string(),
                        Value::Const(k) => format!("{} cards", number_word(*k)),
                        other => {
                            let (c, x) = self.counted(other, "card");
                            format!("{c}{}", x.unwrap_or_default())
                        }
                    };
                    parts.push(format!("two target players each draw {d}"));
                    i += 2;
                    continue;
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
                    kind: ChoiceKind::Color | ChoiceKind::Colors,
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
                    // The next instruction already says "the color of your choice" ("the
                    // color or colors of your choice").
                    if n.contains("the color of your choice")
                        || n.contains("the color or colors of your choice")
                    {
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
            // "Creatures your opponents control block this turn if able, and you choose how
            // those creatures block.": the second restriction is about the blocks the
            // first one requires, for as long.
            if let (
                Some(Effect::AddRestriction {
                    restriction: Restriction::MustBlock(_),
                    duration: d1,
                }),
                Some(Effect::AddRestriction {
                    restriction: r2 @ Restriction::Custom(n),
                    duration: d2,
                }),
            ) = (v.get(i), v.get(i + 1))
            {
                if format!("{d1:?}") == format!("{d2:?}")
                    && n == "combat: controller chooses how opponents' creatures block"
                {
                    let a = self.effect(&v[i]);
                    let b = self.restriction(r2);
                    parts.push(format!("{a}, and {b}"));
                    i += 2;
                    continue;
                }
            }
            // "Target opponent exiles a creature they control and their graveyard.": the
            // player exiles the cards in their graveyard too.
            if let (
                Some(
                    first @ Effect::Exile {
                        what:
                            Sel::Choose {
                                chooser: PlayerRef::Target(t),
                                ..
                            },
                        ..
                    },
                ),
                Some(Effect::Exile {
                    what: Sel::All(Filter::And(g)),
                    face_down: false,
                    ..
                }),
            ) = (v.get(i), v.get(i + 1))
            {
                if let [Filter::InZone(ZoneKind::Graveyard), Filter::OwnedByPlayer(o)] =
                    g.as_slice()
                {
                    let a = self.effect(first);
                    if matches!(o.as_ref(), PlayerRef::Target(x) if x == t)
                        && a.ends_with(" they control")
                    {
                        parts.push(format!("{a} and their graveyard"));
                    } else {
                        parts.push(a);
                        let b = self.effect(&v[i + 1]);
                        parts.push(b);
                    }
                    i += 2;
                    continue;
                }
            }
            // "it enters with an additional +1/+1 counter on it and gains hexproof until
            // your next turn": the spell's permanent, named once.
            if let (
                Some(Effect::AddReplacement { def, .. }),
                Some(Effect::Modify {
                    what: Sel::TriggerSpell,
                    ..
                }),
            ) = (v.get(i), v.get(i + 1))
            {
                if matches!(&def.event, ReplacementEvent::EntersBattlefield(Filter::In(s)) if matches!(s.as_ref(), Sel::TriggerSpell))
                {
                    let a = self.effect(&v[i]);
                    let b = self.effect(&v[i + 1]);
                    match b.strip_prefix("it ") {
                        Some(r) => parts.push(format!("{a} and {{opt:it}} {r}")),
                        None => {
                            parts.push(a);
                            parts.push(b);
                        }
                    }
                    i += 2;
                    continue;
                }
            }
            // "That creature can't be blocked this turn and has base power and toughness
            // 1/1 until end of turn."
            if let (
                Some(Effect::AddRestriction {
                    restriction: Restriction::CantBeBlocked(Filter::In(r)),
                    ..
                }),
                Some(Effect::Modify { what, .. }),
            ) = (v.get(i), v.get(i + 1))
            {
                if same_sel(r, what) {
                    let a = self.effect(&v[i]);
                    let b = self.effect(&v[i + 1]);
                    match ["~it ", "it ", "they "]
                        .iter()
                        .find_map(|p| b.strip_prefix(p))
                    {
                        Some(r) => parts.push(format!("{a} and {{opt:it}} {r}")),
                        None => {
                            parts.push(a);
                            parts.push(b);
                        }
                    }
                    i += 2;
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
                    let (c, x) = self.counted(n, &counter_name(kind));
                    // "with a number of time counters on it equal to its mana value".
                    let s = match x {
                        Some(x) => {
                            let v = self.value(n);
                            let k = plural(&counter_name(kind));
                            format!(
                                "{{alt:exile {w} with {c} on it{x}|exile {w} with a number of {k} on it equal to {v}}}"
                            )
                        }
                        None => format!("exile {w} with {c} on it"),
                    };
                    parts.push(s);
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
                    let mut wheres = String::new();
                    for e in &v[i..j] {
                        if let Effect::AddCounters { kind, n, .. } = e {
                            let (c, x) = self.counted(n, &counter_name(kind));
                            items.push(c);
                            wheres.push_str(&x.unwrap_or_default());
                        }
                    }
                    let t = self.sel(what, Case::Obj);
                    parts.push(format!("put {} on {t}{wheres}", join_list(&items, "and")));
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
                    let s = self.modify_effect(what, &all, duration);
                    parts.push(duration_for_each_part(s));
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
            // Whether an earlier instruction exiled something (`after_exile`).
            let saved_exile = self.after_exile;
            self.after_exile |= v[..i].iter().any(exiles);
            let saved_this_way = self.this_way.clone();
            self.this_way.extend(tail_parts::this_way(&v[..i]));
            let saved_mandatory = std::mem::replace(
                &mut self.prev_mandatory,
                i > 0
                    && matches!(
                        v[i - 1],
                        Effect::Sacrifice {
                            who: PlayerRef::You,
                            ..
                        } | Effect::Discard {
                            who: PlayerRef::You,
                            ..
                        }
                    )
                    // "Pay {E}{E}. If you can't, ..." (see `kw/game_rules.rs`).
                    || i > 0
                        && matches!(&v[i - 1], Effect::Custom(n)
                            if crate::kw::game_rules::is_pay_energy(n)),
            );
            let s = self.effect(&v[i]);
            self.prev_mandatory = saved_mandatory;
            self.this_way = saved_this_way;
            self.after_exile = saved_exile;
            if !s.is_empty() {
                parts.push(s);
            }
            i += 1;
        }
        // "Until end of turn, target creature gets +0/+2 and another target creature gets
        // -2/-0": the duration of consecutive effects said once, after the last.
        for k in 0..parts.len().saturating_sub(1) {
            if parts[k].ends_with(" until end of turn")
                && parts[k + 1].ends_with(" until end of turn")
                && !parts[k].contains('.')
            {
                let n = parts[k].len() - " until end of turn".len();
                parts[k] = format!("{} {{opt:until end of turn}}", &parts[k][..n]);
            }
        }
        // "It connives. When it connives this way, ...": the reflexive trigger refers to
        // the keyword action an object performed (CR 603.12).
        for k in 1..parts.len() {
            if parts[k - 1].ends_with(" connives") {
                if let Some(rest) = parts[k].strip_prefix("when you do, ") {
                    parts[k] = format!("{{alt:when you do|when it connives this way}}, {rest}");
                }
            }
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
        if let Some(s) = self
            .skips_next_this_turn(cond, then, otherwise)
            .or_else(|| self.double_life_total(cond, then, otherwise))
        {
            return s;
        }
        let pushed = super::compared_values(cond)
            .map(|p| self.compared.push(p))
            .is_some();
        let top = std::mem::take(&mut self.trigger_body_if);
        let s = self.if_effect_inner(cond, then, otherwise, top);
        if pushed {
            self.compared.pop();
        }
        s
    }

    fn if_effect_inner(
        &mut self,
        cond: &Condition,
        then: &Effect,
        otherwise: &Effect,
        trigger_body: bool,
    ) -> String {
        let then_empty = matches!(then, Effect::Noop);
        let else_empty = matches!(otherwise, Effect::Noop);
        // "You may choose a card from it. If you do, ... If you don't, ...": whether a
        // card was chosen (see the `Effect::Store` of a choice of up to one).
        if else_empty {
            let chosen = |c: &Condition| matches!(c, Condition::SelNonEmpty(Sel::Var(v)) if self.chosen_up_to_one.contains(v));
            let did = if chosen(cond) {
                Some("do")
            } else if matches!(cond, Condition::Not(c) if chosen(c)) {
                Some("don't")
            } else {
                None
            };
            if let Some(did) = did {
                let t = self.effect(then);
                return format!("if you {did}, {t}");
            }
        }
        // "That player shuffles that card into their library": only if a card was chosen
        // (there's nothing to shuffle otherwise).
        if else_empty {
            if let Condition::SelNonEmpty(sel) = cond {
                let shuffles_it = |e: &Effect| matches!(e, Effect::ShuffleIntoLibrary { what, .. } if same_sel(what, sel));
                if shuffles_it(then)
                    || matches!(then, Effect::AsPlayer { effect, .. } if shuffles_it(effect))
                {
                    return self.effect(then);
                }
            }
        }
        // "Exile that card from your graveyard": only if it's still there (CR 400.7).
        if else_empty {
            if let Condition::SelMatches(sel, Filter::InZone(z)) = cond {
                if matches!(then, Effect::Exile { what, .. } if same_sel(what, sel)) {
                    let t = self.effect(then);
                    if let Some(rest) = t.strip_prefix("exile it") {
                        return format!("exile it from your {}{rest}", zone_word(*z));
                    }
                }
            }
        }
        // "After this main phase, there is an additional combat phase ...": only during
        // a main phase is there a main phase to come after.
        if else_empty
            && matches!(cond, Condition::Phase(PhaseCond::MainPhase))
            && matches!(
                then,
                Effect::AddTurnParts {
                    after_phase: true,
                    ..
                }
            )
        {
            let t = self.effect(then);
            if t.contains("after this phase") {
                return t.replace("after this phase", "after this main phase");
            }
        }
        // "When a creature is put onto the battlefield this way, it deals damage ...": a
        // reflexive trigger for the card the instruction put there (CR 603.12).
        if let (
            true,
            Condition::SelNonEmpty(Sel::Matching(x, Filter::And(f))),
            Effect::Reflexive { body },
        ) = (else_empty, cond, then)
        {
            if matches!(x.as_ref(), Sel::Var(v) if *v == vars::IT)
                && f.iter()
                    .any(|y| matches!(y, Filter::InZone(ZoneKind::Battlefield)))
            {
                let kind: Vec<Filter> = f
                    .iter()
                    .filter(|y| !matches!(y, Filter::InZone(_)))
                    .cloned()
                    .collect();
                let n = self.noun_det(&Filter::and(kind), Det::A);
                let b = self.in_event_scope(|r| r.body(body));
                return format!(
                    "when {n} is put onto the battlefield this way, {}",
                    lower_first(&b)
                );
            }
        }
        match cond {
            Condition::PrevHappened if else_empty && matches!(then, Effect::Reflexive { .. }) => {
                // "When you do, ..." already means "if you do" (CR 603.12).
                self.effect(then)
            }
            // "Clash with an opponent. If you win, ..." (CR 701.30).
            Condition::PrevHappened if self.after_clash => {
                self.after_clash = false;
                let t = self.effect(then);
                if else_empty {
                    return format!("if you win, {t}");
                }
                // "If you win, ... Otherwise, ..." (Pulling Teeth).
                let o = self.effect(otherwise);
                format!("if you win, {t}. otherwise, {o}")
            }
            Condition::PrevHappened => {
                // "Counter target spell unless its controller pays {2}. If they do, ..."
                // "If they don't" / "If that player doesn't".
                let who = if self.last_actor_other {
                    "{alt:they|that player}"
                } else {
                    "you"
                };
                let mut s = String::new();
                if !then_empty {
                    let t = self.effect(then);
                    s = format!("if {who} do, {t}");
                }
                if !else_empty {
                    let o = self.effect(otherwise);
                    // "Sacrifice a creature. If you can't, ...": a mandatory instruction
                    // isn't followed only when it can't be.
                    let dont = if self.prev_mandatory && who == "you" {
                        "{alt:don't|can't}"
                    } else {
                        "don't"
                    };
                    // After "If you do, ...", the other case is "Otherwise, ...".
                    s = if s.is_empty() {
                        format!("if {who} {dont}, {o}")
                    } else {
                        format!("{s}. {{alt:if {who} {dont}|otherwise}}, {o}")
                    };
                }
                s
            }
            Condition::Not(inner) if matches!(inner.as_ref(), Condition::PrevHappened) => {
                self.if_effect(&Condition::PrevHappened, otherwise, then)
            }
            // "Sacrifice it" needs no "if you control it" (CR 701.21a), nor does "you may
            // sacrifice it. If you do, ..." (the check moved before the option).
            Condition::SelMatches(s, Filter::ControlledBy(PlayerRel::You))
                if else_empty && starts_by_sacrificing(then, s) =>
            {
                self.effect(then)
            }
            // "Sacrifice it unless it escaped": nothing happens if C.
            _ if then_empty && !else_empty => {
                let o = self.effect(otherwise);
                let c = self.condition(cond);
                format!("{o} unless {c}")
            }
            // "X. If C, Y instead." (the comparison treats "If C, Y. Otherwise, X." the
            // same): the default effect comes first, so it names the targets.
            // "If ~ is suspected, put a +1/+1 counter on it. Otherwise, suspect it.": the
            // condition names the object first (the comparison reorders either form).
            _ if !else_empty
                && matches!(cond, Condition::SelMatches(Sel::This, _))
                && [then, otherwise].iter().all(|e| {
                    let d = format!("{e:?}");
                    d.contains("This") && !d.contains("Target(")
                }) =>
            {
                let c = self.condition(cond);
                let t = self.effect(then);
                let o = self.effect(otherwise);
                format!("If {c}, {t}. Otherwise, {o}")
            }
            // "You lose life equal to that card's mana value if ~ isn't saddled. Otherwise,
            // each opponent loses that much life.": the same amount either way, said once.
            _ if !else_empty && same_life_amount(then, otherwise) => {
                let c = self.condition(cond);
                let t = self.effect(then);
                let mut o2: Effect = otherwise.clone();
                if let Effect::LoseLife { n, .. } | Effect::GainLife { n, .. } = &mut o2 {
                    *n = Value::Prev;
                }
                let o = self.effect(&o2);
                format!("If {c}, {t}. Otherwise, {o}")
            }
            _ if !else_empty => {
                let o = self.effect(otherwise);
                let c = self.condition(cond);
                let mut t = self.effect(then);
                // "~ deals 3 damage to any target. If ..., it deals 4 damage instead."
                // (Also "If ..., ~ deals damage equal to that card's power instead": the
                // amount set just before.)
                let then_damage = match then {
                    Effect::Seq(v)
                        if matches!(
                            v.as_slice(),
                            [Effect::SetX { .. }, Effect::DealDamage { .. }]
                        ) =>
                    {
                        &v[1]
                    }
                    other => other,
                };
                if let (Effect::DealDamage { to: a, .. }, Effect::DealDamage { to: b, .. }) =
                    (then_damage, otherwise)
                {
                    if same_sel(a, b) && t.contains(" to it, where X is ") {
                        t = t.replacen(" to it, where X is ", " {opt:to it}, where X is ", 1);
                        // "~ deals damage equal to that card's power instead".
                        if let (
                            Effect::Seq(v),
                            Effect::DealDamage {
                                source,
                                amount: Value::X,
                                ..
                            },
                        ) = (then, then_damage)
                        {
                            if let Some(Effect::SetX { value }) = v.first() {
                                let x = self.value(value);
                                let src = self.sel(source, Case::Subj);
                                t = format!("{{alt:{t}|{src} deals damage equal to {x}}}");
                            }
                        }
                    } else if same_sel(a, b) {
                        if let Some(head) = t
                            .strip_suffix(" to it")
                            .or_else(|| t.strip_suffix(" to {alt:it|that permanent or player}"))
                            .or_else(|| t.strip_suffix(" to them"))
                        {
                            t = format!("{head} {{opt:to it}}");
                        }
                    }
                }
                format!("{o}. If {c}, {t} instead")
            }
            // "It deals 2 damage to you unless it came under your control this turn": an
            // event that didn't happen.
            Condition::Not(inner)
                if else_empty
                    && matches!(
                        inner.as_ref(),
                        Condition::Compare(
                            Value::EventsThisTurn(..) | Value::EventsDuring(..),
                            _,
                            _
                        )
                    ) =>
            {
                let Condition::Compare(a, cmp, b) = inner.as_ref() else {
                    return String::new();
                };
                let neg = self.attempt(|r| r.events_compare(a, *cmp, b, true));
                let pos = self.attempt(|r| r.events_compare(a, *cmp, b, false));
                let t = self.effect(then);
                match (neg, pos) {
                    (Some(neg), Some(pos)) => format!("{{alt:{t} unless {pos}|if {neg}, {t}}}"),
                    _ => {
                        let c = self.condition(cond);
                        format!("if {c}, {t}")
                    }
                }
            }
            _ => {
                let form = |c: String, t: String| -> String {
                    // "Discard a card unless you attacked this turn."
                    if let Some(inner) = c.strip_prefix("it's not true that ") {
                        return format!("{t} unless {inner}");
                    }
                    if trigger_body {
                        return format!("{t} if {c}");
                    }
                    format!("if {c}, {t}")
                };
                // A condition about a target the instruction acts on: "Counter target
                // spell if it was kicked" names the target in the instruction, "If target
                // spell was kicked, counter it" in the condition.
                let mut probe = self.clone();
                let _ = probe.condition(cond);
                if probe.introduced != self.introduced {
                    let (a, b) = self.two_ways(
                        |r| {
                            let c = r.condition(cond);
                            let t = r.effect(then);
                            form(c, t)
                        },
                        |r| {
                            let t = r.effect(then);
                            let c = r.condition(cond);
                            form(c, t)
                        },
                    );
                    return if a == b {
                        a
                    } else {
                        format!("{{alt:{a}|{b}}}")
                    };
                }
                let c = self.condition(cond);
                let t = self.effect(then);
                form(c, t)
            }
        }
    }

    /// "You may cast an instant or sorcery spell from your hand without paying its mana
    /// cost": you cast up to one card, chosen as it's cast (choosing none is not casting).
    /// Only a nonland card can be cast (CR 305.9), and the card cast is a spell (CR
    /// 601.2a).
    /// "[That player] may choose any number of tapped blue creatures they control and pay
    /// {4} for each creature chosen this way. If the player does, untap those creatures."
    /// (performed as that player): choosing none and paying nothing is not doing it.
    fn choose_and_pay_each(&mut self, e: &Effect) -> Option<String> {
        let Effect::Seq(v) = e else { return None };
        let [Effect::Store {
            var,
            sel:
                Sel::Choose {
                    chooser: PlayerRef::You,
                    filter,
                    count: Value::CountSel(all),
                    up_to: true,
                    ..
                },
        }, Effect::PayOptional {
            who: PlayerRef::You,
            cost,
            then,
            otherwise,
        }, Effect::If {
            cond: Condition::PrevHappened,
            then: untap,
            otherwise: no,
        }] = v.as_slice()
        else {
            return None;
        };
        if !matches!(all.as_ref(), Sel::All(f) if format!("{f:?}") == format!("{filter:?}"))
            || !matches!(then.as_ref(), Effect::Noop)
            || !matches!(otherwise.as_ref(), Effect::Noop)
            || !matches!(no.as_ref(), Effect::Noop)
            || !matches!(untap.as_ref(), Effect::Untap { what: Sel::Var(x) } if x == var)
            || cost.mana.is_some()
        {
            return None;
        }
        let [CostPart::Repeated { cost: each, times }] = cost.parts.as_slice() else {
            return None;
        };
        if !matches!(times, Value::CountSel(s) if matches!(s.as_ref(), Sel::Var(x) if x == var)) {
            return None;
        }
        let parts: Vec<Filter> = match filter {
            Filter::And(v) => v.clone(),
            other => vec![other.clone()],
        };
        if !parts
            .iter()
            .any(|x| matches!(x, Filter::ControlledBy(PlayerRel::You)))
        {
            return None;
        }
        let kind: Vec<Filter> = parts
            .iter()
            .filter(|x| !matches!(x, Filter::ControlledBy(_) | Filter::InZone(_)))
            .cloned()
            .collect();
        let types: Vec<Filter> = kind
            .iter()
            .filter(|x| matches!(x, Filter::Type(_)))
            .cloned()
            .collect();
        let many = self.noun(&Filter::and(kind), Num::Many);
        let one = self.noun(&Filter::and(types.clone()), Num::One);
        let ones = self.noun(&Filter::and(types), Num::Many);
        let c = self.cost(each);
        Some(format!(
            "may choose any number of {many} they control and pay {c} for each {one} chosen this way. If {{alt:the player|that player|they}} {{alt:does|do}}, untap those {ones}"
        ))
    }

    /// One card chosen from a player's revealed hand or their graveyard: "a nonland card
    /// from that player's graveyard or hand", "a nonland card from it or a card from their
    /// graveyard".
    fn hand_or_graveyard_choice(&mut self, f: &Filter) -> Option<String> {
        // The player whose cards they are, and the qualities other than zone and owner.
        let split = |v: &[Filter]| -> Option<(PlayerRef, Vec<Filter>)> {
            let p = v.iter().find_map(|x| match x {
                Filter::OwnedByPlayer(p) => Some((**p).clone()),
                _ => None,
            })?;
            let rest = v
                .iter()
                .filter(|x| {
                    !matches!(
                        x,
                        Filter::OwnedByPlayer(_) | Filter::InZone(_) | Filter::Or(_)
                    )
                })
                .cloned()
                .collect();
            Some((p, rest))
        };
        match f {
            Filter::And(v)
                if v.iter().any(|x| {
                    matches!(x, Filter::Or(z) if matches!(z.as_slice(),
                    [Filter::InZone(ZoneKind::Hand), Filter::InZone(ZoneKind::Graveyard)]))
                }) =>
            {
                let (p, rest) = split(v)?;
                let n = self.noun_det(&Filter::and(rest), Det::A);
                let alts: Vec<String> = possessives(self, &p)
                    .iter()
                    .flat_map(|poss| {
                        [
                            format!("{n} from {poss} graveyard or hand"),
                            format!("{n} from {poss} hand or graveyard"),
                        ]
                    })
                    .collect();
                Some(format!("{{alt:{}}}", alts.join("|")))
            }
            Filter::Or(alts) => {
                let [Filter::And(a), Filter::And(b)] = alts.as_slice() else {
                    return None;
                };
                let in_zone = |v: &[Filter], z: ZoneKind| {
                    v.iter().any(|x| matches!(x, Filter::InZone(y) if *y == z))
                };
                if !in_zone(a, ZoneKind::Hand) || !in_zone(b, ZoneKind::Graveyard) {
                    return None;
                }
                let (pa, ra) = split(a)?;
                let (pb, rb) = split(b)?;
                if format!("{pa:?}") != format!("{pb:?}") {
                    return None;
                }
                let na = self.noun_det(&Filter::and(ra), Det::A);
                let nb = self.noun_det(&Filter::and(rb), Det::A);
                let alts: Vec<String> = possessives(self, &pb)
                    .iter()
                    .map(|poss| format!("{na} from it or {nb} from {poss} graveyard"))
                    .collect();
                Some(format!("{{alt:{}}}", alts.join("|")))
            }
            _ => None,
        }
    }

    fn cast_up_to_one(&mut self, effect: &Effect) -> Option<String> {
        let Effect::CastCard {
            who: PlayerRef::You,
            what:
                Sel::Choose {
                    chooser: PlayerRef::You,
                    filter,
                    count: Value::Const(count),
                    up_to: true,
                    ..
                },
            free,
            optional: false,
        } = effect
        else {
            return None;
        };
        // "You may cast any number of spells from among them".
        let many = *count >= 99;
        if *count != 1 && !many {
            return None;
        }
        // "from among them": a group an earlier instruction exiled or looked at (a card
        // that left exile or the library is a new object, not one of them, CR 400.7, so
        // "in exile" goes unsaid).
        let parts: Vec<Filter> = match filter {
            Filter::And(v) => v.clone(),
            other => vec![other.clone()],
        };
        let group = parts.iter().find_map(|x| match x {
            Filter::In(s) => match s.as_ref() {
                Sel::Var(v) if *v == vars::IT || *v >= vars::USER => Some(*v),
                _ => None,
            },
            _ => None,
        });
        let group = group.filter(|_| {
            parts
                .iter()
                .any(|x| matches!(x, Filter::InZone(ZoneKind::Exile)))
                || parts.iter().any(|x| matches!(x, Filter::Card))
        });
        let f = Filter::and(
            parts
                .iter()
                .filter(|x| !matches!(x, Filter::Not(n) if matches!(n.as_ref(), Filter::Type(CardType::Land))))
                .filter(|x| {
                    group.is_none()
                        || !matches!(x, Filter::In(_) | Filter::InZone(ZoneKind::Exile | ZoneKind::Library) | Filter::Spell)
                })
                .cloned()
                .collect(),
        );
        let f = if group.is_some() && !format!("{f:?}").contains("Card") {
            Filter::and(vec![f, Filter::Card])
        } else {
            f
        };
        let det = if many {
            super::nouns::Det::Count("any number of".into())
        } else {
            super::nouns::Det::A
        };
        let spell_noun = |r: &mut Self, f: &Filter, det: super::nouns::Det| {
            let n = r.noun_det(f, det);
            match n.find(" card") {
                Some(i) if n[i + 5..].is_empty() || n[i + 5..].starts_with(' ') => {
                    format!("{} spell{}", &n[..i], &n[i + 5..])
                }
                Some(i) if group.is_some() && n[i + 5..].starts_with('s') => {
                    format!("{} spell{}", &n[..i], &n[i + 5..])
                }
                _ => n,
            }
        };
        // "cast a spell with mana value X from among cards exiled with ~", "... from among
        // cards you own exiled with ~": one of the cards an ability of this object exiled
        // (CR 607.2a), cast as a spell.
        let linked = parts
            .iter()
            .any(|x| matches!(x, Filter::In(s) if matches!(s.as_ref(), Sel::Linked)));
        let among_linked = (linked && group.is_none()).then(|| {
            let owned = parts
                .iter()
                .any(|x| matches!(x, Filter::OwnedBy(PlayerRel::You)));
            let kind = Filter::and(
                parts
                    .iter()
                    .filter(|x| {
                        !matches!(x, Filter::In(_) | Filter::InZone(_) | Filter::OwnedBy(PlayerRel::You))
                            && !matches!(x, Filter::Not(n) if matches!(n.as_ref(), Filter::Type(CardType::Land)))
                    })
                    .cloned()
                    .collect(),
            );
            let you_own = if owned { "you own " } else { "" };
            (kind, you_own)
        });
        let n = spell_noun(self, &f, det.clone());
        let n = match among_linked {
            Some((kind, you_own)) => {
                let k = spell_noun(self, &kind, det);
                format!("{{alt:{n}|{k} from among cards {you_own}exiled with ~}}")
            }
            None => n,
        };
        let n = if group.is_some() {
            format!("{n} from among {{alt:them|those cards|the exiled cards}}")
        } else {
            n
        };
        let fr = match (*free, many) {
            (false, _) => "",
            (true, false) => " without paying its mana cost",
            (true, true) => " without paying their mana costs",
        };
        // Any number of them may be none: "Cast any number of cards exiled with ~ ...".
        let may = if many { "{opt:you may}" } else { "you may" };
        Some(format!("{may} cast {n}{fr}"))
    }

    fn may(&mut self, who: &PlayerRef, effect: &Effect) -> String {
        // "Its controller may search ..." recording who searched (for "each player who
        // searched their library this way shuffles"): the record isn't said.
        if let Effect::Seq(v) = effect {
            if let [a, Effect::Store { var, .. }] = v.as_slice() {
                if *var == crate::oracle::patterns::iteration_grammar::SEARCHERS {
                    return self.may(who, a);
                }
            }
        }
        if matches!(who, PlayerRef::You) {
            if let Some(c) = self.cast_up_to_one(effect) {
                return c;
            }
        }
        // "You may exile target creature if it attacked or blocked this turn": the
        // option, with a condition checked as it's taken.
        if let Effect::If {
            cond,
            then,
            otherwise,
        } = effect
        {
            // (Only a condition about a target the instruction names.)
            let mut probe = self.clone();
            let _ = probe.condition(cond);
            if matches!(otherwise.as_ref(), Effect::Noop)
                && !matches!(cond, Condition::PrevHappened)
                && !matches!(then.as_ref(), Effect::Reflexive { .. })
                && probe.introduced != self.introduced
            {
                let p = self.player(who, Case::Subj);
                let t = self.effect(then);
                let c = self.condition(cond);
                return format!("{p} may {t} if {c}");
            }
        }
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
            if matches!(action, KeywordAction::Explore | KeywordAction::Connive | KeywordAction::Endure))
            // "You may have that creature's controller create a token".
            || matches!(effect, Effect::AsPlayer { who: w, .. } if !matches!(w, PlayerRef::You));
        if has_subject
            && !inner.starts_with("gain control")
            && !inner.starts_with("switch")
            && !inner.starts_with("{alt:switch")
        {
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
            // "unless that player sacrifices a nonland permanent or discards a card": the
            // same player's alternatives, nested.
            let mut costs = vec![cost];
            let mut otherwise = otherwise;
            while let Effect::PayOptional {
                who: w,
                cost: c,
                then: t,
                otherwise: o,
            } = otherwise
            {
                if !same_player(w, who)
                    || !matches!(**t, Effect::Noop)
                    || matches!(**o, Effect::Noop)
                {
                    break;
                }
                costs.push(c);
                otherwise = o;
            }
            let o = self.effect(otherwise);
            // Any one of several players may pay ("unless any player pays {1}").
            let p = match who {
                PlayerRef::EachPlayer => "{alt:any player|a player}".to_string(),
                PlayerRef::EachOpponent => "{alt:an opponent|any opponent}".to_string(),
                _ => self.player(who, Case::Subj),
            };
            let mut pays: Vec<String> = costs
                .into_iter()
                .map(|c| {
                    let pays = self.cost_as_payment(c);
                    if p == "you" {
                        pays
                    } else {
                        // The payment is worded for the player who pays. How many times a
                        // repeated payment is made ("{1} for each artifact you control") is
                        // counted for the ability's controller before anyone pays
                        // (`cumulative_upkeep::expand_repeated`), so "you" there stays.
                        let (head, each) = match pays.split_once(" for each ") {
                            Some((h, e)) => (h.to_string(), format!(" for each {e}")),
                            None => match pays.split_once(", where X is ") {
                                Some((h, e)) => (h.to_string(), format!(", where X is {e}")),
                                None => (pays.clone(), String::new()),
                            },
                        };
                        let head = format!(" {head} ")
                            .replace(" your ", " their ")
                            .replace(" you control", " they control")
                            .replace(" to you ", " to them ");
                        format!("{}{each}", third_person(head.trim()))
                    }
                })
                .collect();
            // "pays {1} or 1 life": one verb for payments of the same kind.
            let pay = if p == "you" { "pay " } else { "pays " };
            if pays.len() > 1 && pays.iter().all(|x| x.starts_with(pay)) {
                for x in pays.iter_mut().skip(1) {
                    *x = x[pay.len()..].to_string();
                }
            }
            // "that player sacrifices it unless they pay {X}": the player named before.
            let p = if p == "that player" && o.contains("that player") {
                "{alt:that player|they}".to_string()
            } else {
                p
            };
            return format!("{o} unless {p} {}", pays.join(" or "));
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
        // "cast a spell with mana value X from among cards exiled with ~": one of the
        // cards an ability of this object exiled (CR 607.2a), cast as a spell.
        let w = match w.split_once(" exiled with ~it") {
            Some((head, "")) if !head.contains('{') && head.contains(" spell") => {
                format!("{{alt:{w}|{head} from among cards exiled with ~}}")
            }
            _ => w,
        };
        // An ability that functions in a graveyard casts the card from there ("you may cast
        // ~ from your graveyard", CR 113.6m).
        let w = if matches!(what, Sel::This) && matches!(self.zone, FunctionZone::Graveyard) {
            format!("{w} {{opt:from your graveyard}}")
        } else {
            w
        };
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

    /// "the enchanted instant card" / "the enchanted card", for an Aura that enchants a card
    /// in a graveyard ("Enchant instant card in a graveyard").
    fn enchanted_card(&mut self, with_type: bool) -> Option<String> {
        let e = self.info.enchant.clone()?;
        let head = e.split(" card").next().filter(|_| e.contains(" card"))?;
        Some(if with_type {
            format!("the enchanted {head} card")
        } else {
            "the enchanted card".into()
        })
    }

    /// Moving objects between zones.
    pub(crate) fn move_effect(&mut self, what: &Sel, to: &Destination) -> String {
        let mut w = self.sel(what, Case::Obj);
        // "Target opponent chooses two of those cards. Put the chosen cards into your
        // graveyard" (`patterns::choice_grammar_objects`).
        if matches!(what, Sel::Var(v) if *v == crate::oracle::patterns::choice_grammar_objects::CHOSEN)
            && (w == "it" || w == "them")
        {
            w = format!("{{alt:{w}|the chosen cards|the chosen card}}");
        }
        // An Aura enchanting a card in a graveyard: "return enchanted creature card to the
        // battlefield" (Animate Dead).
        if matches!(what, Sel::AttachedTo)
            && self
                .info
                .enchant
                .as_deref()
                .is_some_and(|e| e.contains(" card"))
        {
            w.push_str(" card");
        }
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
        // "Put ~ from your hand onto the battlefield": the card itself, while in your hand.
        if let Sel::All(Filter::And(f)) = what {
            let source = f.iter().any(|x| {
                matches!(x, Filter::Custom(n) if n == crate::kw::hand_graveyard_actions::SOURCE_OR_NEXT)
            });
            if source
                && f.iter()
                    .any(|x| matches!(x, Filter::InZone(ZoneKind::Hand)))
                && to.zone != ZoneKind::Hand
                && !w.contains(" hand")
            {
                w.push_str(" from your hand");
            }
        }
        // "any number of target artifact cards ... on top of their library in any order".
        let plural = is_plural_sel(what)
            || matches!(what, Sel::Target(i)
                if self.targets.get(*i as usize).is_some_and(|t| !matches!(t.max, Value::Const(1))));
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
        // it there (CR 110.2a), "put" or "return": the instruction is yours.
        let d = if to.zone == ZoneKind::Battlefield {
            // Cards say it before or after "tapped and attacking".
            d.replacen(
                " tapped and attacking under your control",
                " {alt:tapped and attacking {opt:under your control}|under your control tapped and attacking}",
                1,
            )
            .replacen(" under your control", " {opt:under your control}", 1)
        } else {
            d
        };
        // "Put a commander you own from the command zone into your hand".
        if let Some(head) = w.strip_suffix(" in the command zone") {
            let head = if head == "a commander you own" {
                "{alt:a commander you own|your commander}"
            } else {
                head
            };
            return format!(
                "{verb} {head} {{alt:from the command zone {d}|{d} from the command zone}}"
            );
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
            ZoneKind::Library if to.position_choice.len() == 2 => {
                format!("on your choice of the top or bottom of {owner} library")
            }
            // Several cards put on top of or under a library go in the order their owner
            // chooses (CR 401.4): cards say "in any order" or leave it out.
            ZoneKind::Library if plural => match to.position {
                LibraryPosition::Top => format!("on top of {owner} library {{opt:in any order}}"),
                LibraryPosition::Bottom => {
                    format!("on the bottom of {owner} library {{opt:in any order}}")
                }
                LibraryPosition::BottomRandom => {
                    format!("on the bottom of {owner} library in a random order")
                }
                LibraryPosition::FromTop(n) => {
                    format!("into {owner} library {} from the top", ordinal_word(n + 1))
                }
                LibraryPosition::Shuffled => format!("into {owner} library"),
            },
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
                // "onto the battlefield under your control tapped and attacking": the
                // control said before the status.
                Some(PlayerRef::You) if yours && to.tapped && to.attacking && !to.transformed => {
                    let st = " tapped and attacking";
                    if let Some(head) = s.strip_suffix(st) {
                        s = format!(
                            "{head} {{alt:tapped and attacking {{opt:under your control}}|under your control tapped and attacking}}"
                        );
                    } else {
                        s.push_str(" {opt:under your control}");
                    }
                }
                // The player putting it there controls it (CR 110.2a): cards may say so.
                Some(PlayerRef::You) if yours => s.push_str(" {opt:under your control}"),
                // Its owner is you: a card from your graveyard enters under your control
                // (CR 110.2a), said or not.
                Some(PlayerRef::OwnerOf(_)) if yours => {
                    s.push_str(&format!(" {{opt:under {owner} control}}"));
                }
                // "Return all artifact cards from all graveyards to the battlefield under
                // their owners' control": each card's owner.
                Some(PlayerRef::OwnerOf(o)) if matches!(o.as_ref(), Sel::All(_)) => {
                    s.push_str(&format!(" under {owner} control"));
                }
                Some(c) => {
                    let c = self.player(c, Case::Poss);
                    s.push_str(&format!(" under {c} control"));
                }
                // No controller named: the player putting it there (CR 110.2a), whoever
                // owns it ("return a card exiled with ~ to the battlefield").
                None if !yours && !self.in_as_player => s.push_str(" {opt:under your control}"),
                None => {}
            }
            if let Some(a) = &to.attached_to {
                let a = self.sel(a, Case::Obj);
                s.push_str(&format!(" attached to {a}"));
            }
            // "with a hexproof counter and an indestructible counter on it".
            let cs: Vec<String> = to
                .with_counters
                .iter()
                .map(|(k, n)| self.counted(n, &counter_name(k)).0)
                .collect();
            if !cs.is_empty() {
                s.push_str(&format!(" with {} on it", join_list(&cs, "and")));
            }
            if !to.with_mods.is_empty() {
                let vp = self.mods_vp(&to.with_mods, true);
                // It enters that way: "It's a 1/1 Spirit creature with flying in addition
                // to its other types."
                let vp = vp.replace("becomes ", "{alt:becomes|is} ");
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
        // CR 701.23d: a search for a quantity of cards ("three cards") must find that
        // many; CR 701.23b: one for cards with a stated quality needn't find any, so
        // "two basic land cards" and "up to two basic land cards" are the same.
        let quantity = crate::search_rules::quantity_only(filter);
        // "up to X basic land cards ..., where X is five minus the number of lands you
        // control": an arithmetic count is defined after the instruction (CR 107.3).
        let mut x_def = None;
        let det = match count {
            other @ (Value::Diff(..) | Value::Sum(..) | Value::Max(..))
                if self.value(other).contains(" minus ")
                    || self.value(other).contains(" plus ") =>
            {
                x_def = Some(self.value(other));
                if quantity {
                    Det::Count("X".into())
                } else {
                    Det::Count("{opt:up to} X".into())
                }
            }
            Value::Const(1) => Det::A,
            // An unbounded count: "any number of Dragon creature cards".
            Value::Const(n) if *n >= 99 => Det::Count("any number of".into()),
            Value::Const(n) if quantity => Det::Count(number_word(*n)),
            Value::Const(n) => Det::Count(format!("{{opt:up to}} {}", number_word(*n))),
            other => {
                let v = self.value(other);
                if quantity {
                    Det::Count(v)
                } else {
                    Det::Count(format!("{{opt:up to}} {v}"))
                }
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
        if to2.controller.as_ref().is_some_and(|c| same_player(c, who)) && same_player(who, whose) {
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
                if many {
                    "those cards {opt:in any order}"
                } else {
                    "that card"
                }
            ));
            return s;
        }
        if to.zone == ZoneKind::Exile {
            let fd = if to.face_down { " face down" } else { "" };
            s.push_str(&format!(", exile {pron}{fd}"));
        } else {
            s.push_str(&format!(", put {pron} {dest}"));
        }
        if let Some(v) = x_def {
            s.push_str(&format!(", where X is {v}"));
        }
        if *shuffle {
            if same_player(who, whose) {
                s.push_str(", then shuffle");
            } else {
                // "Search target player's library ... Then that player shuffles." (the
                // library's owner shuffles it, CR 701.24a).
                s.push_str(". Then {alt:that player|they} shuffles");
            }
        }
        s
    }

    /// A search of one or more zones for one or more kinds of cards (`SearchSpec`).
    fn search_cards(&mut self, spec: &SearchSpec) -> String {
        let whose =
            if same_player(&spec.who, &spec.whose) || matches!(spec.whose, PlayerRef::Iterated) {
                self.possessive_for(&spec.who)
            } else {
                self.player(&spec.whose, Case::Poss)
            };
        let zone = |z: &ZoneKind| match z {
            ZoneKind::Library => "library",
            ZoneKind::Graveyard => "graveyard",
            ZoneKind::Hand => "hand",
            ZoneKind::Exile => "exile",
            ZoneKind::Battlefield => "battlefield",
            _ => "zone",
        };
        let zones: Vec<&str> = spec.zones.iter().map(zone).collect();
        let joiner = if spec.zones_optional { "and/or" } else { "and" };
        let zones = match zones.as_slice() {
            [] => String::new(),
            [one] => (*one).to_string(),
            [init @ .., last] => format!("{} {joiner} {last}", init.join(", ")),
        };
        let mut many = false;
        let mut where_x: Option<String> = None;
        let parts: Vec<String> = spec
            .parts
            .iter()
            .map(|p| {
                let det = match (&p.count, p.up_to) {
                    _ if p.all => Det::Count("all".into()),
                    (Value::Const(1), false) => Det::A,
                    (Value::Const(n), _) if *n >= 99 => Det::Count("any number of".into()),
                    (Value::Const(n), false) => Det::Count(number_word(*n)),
                    (Value::Const(n), true) => Det::UpTo(number_word(*n)),
                    (other, up_to) => {
                        let v = self.value(other);
                        // "up to X basic land cards, where X is ...".
                        if v != "X" && !v.contains(['{', '|']) {
                            where_x = Some(v);
                            if up_to {
                                Det::UpTo("X".into())
                            } else {
                                Det::Count("X".into())
                            }
                        } else if up_to {
                            Det::UpTo(v)
                        } else {
                            Det::Count(v)
                        }
                    }
                };
                if p.all || p.up_to || !matches!(p.count, Value::Const(1)) {
                    many = true;
                }
                // What's searched for in a library, hand, or graveyard is a card ("a card
                // named The Animus").
                let saved = self.default_head.replace("card");
                let n = self.noun_det(&p.filter, det);
                self.default_head = saved;
                n
            })
            .collect();
        if parts.len() > 1 {
            many = true;
        }
        let parts = match parts.as_slice() {
            [] => String::new(),
            [one] => one.clone(),
            [init @ .., last] => format!("{} and {last}", init.join(", ")),
        };
        let may = if spec.optional { "may " } else { "" };
        let mut s = format!("{may}search {whose} {zones} for {parts}");
        if let Some(v) = where_x {
            s.push_str(&format!(", where X is {v},"));
        }
        if spec.distinct_names {
            s.push_str(" with different names");
        }
        let pron = if many { "them" } else { "it" };
        if spec.reveal {
            s.push_str(&format!(", reveal {pron}"));
        }
        if spec.shuffle == SearchShuffle::Before {
            let order = if many { " {opt:in any order}" } else { "" };
            // Where the cards go once the library is shuffled: on top, or "third from the
            // top".
            let pos = match spec.dests.as_slice() {
                [d] if d.or.is_empty() && d.to.zone == ZoneKind::Library => match d.to.position {
                    LibraryPosition::Top => "on top".to_string(),
                    LibraryPosition::FromTop(n) => format!("{} from the top", ordinal_word(n + 1)),
                    _ => self.gap("a search's destination after shuffling"),
                },
                _ => self.gap("a search's destination after shuffling"),
            };
            let what = if many {
                "{alt:them|those cards}"
            } else {
                "{alt:it|that card}"
            };
            s.push_str(&format!(", then shuffle and put {what} {pos}{order}"));
            return s;
        }
        let own = same_player(&spec.who, &spec.whose);
        let two =
            matches!(spec.parts.as_slice(), [p] if matches!(p.count, Value::Const(2)) && !p.all);
        let dests: Vec<String> = spec
            .dests
            .iter()
            .map(|d| {
                // The searching player puts the card onto the battlefield, under their
                // control (CR 110.2a): "under its controller's control" may go unsaid.
                let to = match &d.to.controller {
                    Some(c)
                        if d.to.zone == ZoneKind::Battlefield
                            && !matches!(c, PlayerRef::You)
                            && same_player(c, &spec.who) =>
                    {
                        let mut t = d.to.clone();
                        t.controller = None;
                        let p = self.player(c, Case::Poss);
                        let base = self.destination_phrase(&t, many, true);
                        Some(format!("{base} {{opt:under {p} control}}"))
                    }
                    _ => None,
                };
                let mut alts = vec![match to {
                    Some(t) => t,
                    None => self.destination_phrase(&d.to, many, own),
                }];
                for o in &d.or {
                    alts.push(self.destination_phrase(o, many, own));
                }
                let phrase = alts.join(" or ");
                // "... and exile them": exile has no "put ... into" phrase.
                let verb = if d.to.zone == ZoneKind::Exile && d.or.is_empty() {
                    "exile"
                } else {
                    "put"
                };
                let what = match &d.count {
                    Some(n) => self.amount(n).0,
                    // "put one onto the battlefield and the other into your hand": of
                    // two cards, the rest is the other one.
                    None if spec.dests.len() > 1 && two => "{alt:the rest|the other}".to_string(),
                    None if spec.dests.len() > 1 => "the rest".to_string(),
                    None => pron.to_string(),
                };
                format!("{verb} {what} {phrase}").trim_end().to_string()
            })
            .collect();
        self.search_verb = Some(match spec.dests.first().map(|d| d.to.zone) {
            Some(ZoneKind::Exile) => "exiled",
            Some(ZoneKind::Graveyard) => "put into a graveyard",
            _ => "put",
        });
        // "put one into your hand and the rest into your graveyard": one verb.
        let mut dests = dests;
        for i in (1..dests.len()).rev() {
            if dests[i].starts_with("put ") && dests[i - 1].starts_with("put ") {
                dests[i] = dests[i]["put ".len()..].to_string();
            }
        }
        // Another player searches ("that player loses 3 life, searches their library for
        // a card, puts it into their hand, then shuffles"): every verb is theirs.
        let third = (!matches!(spec.who, PlayerRef::You) || self.in_as_player) && !spec.optional;
        if third {
            for d in dests.iter_mut() {
                if let Some(r) = d.strip_prefix("put ") {
                    *d = format!("puts {r}");
                } else if let Some(r) = d.strip_prefix("exile ") {
                    *d = format!("exiles {r}");
                }
            }
            s = s.replace(", reveal ", ", reveals ");
        }
        if !dests.is_empty() {
            s.push_str(&format!(", {}", dests.join(" and ")));
        }
        if spec.shuffle == SearchShuffle::After {
            // "Search your library and/or graveyard": the library is shuffled only if it
            // was searched.
            if spec.zones_optional
                && spec.zones.len() > 1
                && spec.zones.contains(&ZoneKind::Library)
            {
                if matches!(spec.who, PlayerRef::You) {
                    s.push_str(". If you search your library this way, shuffle");
                } else {
                    let w = self.player(&spec.who, Case::Subj);
                    s.push_str(&format!(
                        ". If {w} searches {} library this way, {w} shuffles",
                        self.player(&spec.who, Case::Poss)
                    ));
                }
            } else if own || matches!(spec.whose, PlayerRef::Iterated) {
                s.push_str(if third {
                    ", then shuffles"
                } else {
                    ", then shuffle"
                });
            } else {
                // "Search target player's library ... Then that player shuffles."
                s.push_str(". Then {alt:that player|they} shuffles");
            }
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
        // Only looking at another player's cards: nobody chooses anything, so it's what
        // the ability's controller does ("Look at the top card of target player's
        // library.").
        let only_look = matches!(take, Value::Const(0))
            && !reveal
            && !matches!(who, PlayerRef::You)
            && !(rest_to.zone == ZoneKind::Library
                && rest_to.position == LibraryPosition::Top
                && !matches!(n, Value::Const(1)));
        // The cards are another player's, but the ability's controller looks at them and
        // chooses (`library::dig`): "look at the top four cards of target opponent's
        // library, exile one of them".
        let others_cards = !matches!(who, PlayerRef::You) && !self.in_as_player && !only_look;
        let p = if only_look || others_cards {
            self.player(who, Case::Poss)
        } else {
            self.possessive_for(who)
        };
        let top = match n {
            // "that many cards from the top of your library": the amount paid by "pay any
            // amount of ...".
            Value::X if self.extra.x_paid => format!(
                "{{alt:the top X cards of {p} library|that many cards from the top of {p} library}}"
            ),
            // "the top X cards of your library, where X is ...".
            other => self.top_cards(other, &p),
        };
        let look = if reveal { "reveal" } else { "look at" };
        if only_look {
            return format!("look at {top}");
        }
        let mut s = if others_cards {
            format!("{look} {top}")
        } else {
            self.with_subject(who, &format!("{look} {top}"), false)
        };
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
        // Every one of them that matches: "put all land cards revealed this way into your
        // hand"; as many as you like: "you may put any number of ... from among them".
        let every = matches!(take, Value::Const(k) if *k >= 99);
        let take_all = every && !take_up_to;
        let take_s = if every && !matches!(filter, Filter::Any) {
            let what = plural(&noun);
            if take_up_to {
                // "any number of creature and/or land cards".
                let what = if what.contains('{') {
                    what
                } else {
                    what.replacen(" or ", " {alt:or|and/or} ", 1)
                };
                format!("you may put any number of {what} from among them {d}")
            } else {
                let from = if reveal {
                    "{alt:revealed this way|from among them}"
                } else {
                    "from among them"
                };
                format!("put all {what} {from} {d}")
            }
        } else if matches!(filter, Filter::Any) {
            if take_up_to {
                format!("put up to {count} of them {d}")
            } else {
                format!("put {count} of them {d}")
            }
        } else {
            let what = match take {
                Value::Const(1) => with_article(&noun),
                // An unbounded count: "any number of artifact cards".
                // "any number of creature and/or land cards".
                Value::Const(k) if *k >= 99 => {
                    let n = plural(&noun);
                    let n = if n.contains('{') {
                        n
                    } else {
                        n.replacen(" or ", " {alt:or|and/or} ", 1)
                    };
                    format!("any number of {n}")
                }
                _ => format!("{count} {}", plural(&noun)),
            };
            let what = if take_up_to && many && !what.starts_with("any number of ") {
                format!("up to {what}")
            } else {
                what
            };
            // "You may reveal up to two creature cards": "up to" already lets the player
            // take none.
            let may = if take_up_to && !many {
                "you may "
            } else if take_up_to {
                "{opt:you may} "
            } else {
                ""
            };
            if !reveal && take_to.zone == ZoneKind::Hand {
                format!("{may}reveal {what} from among them and put {pron} {d}")
            } else {
                format!("{may}put {what} from among them {d}")
            }
        };
        // "exile one of them face down": an exile destination is the verb.
        let take_s = if take_to.zone == ZoneKind::Exile {
            take_s.replacen("put ", "exile ", 1)
        } else {
            take_s
        };
        let mut rest = self.destination_phrase(rest_to, true, true);
        if rest_to.zone == ZoneKind::Library {
            rest = rest.replace("their owners' library", &format!("{p} library"));
        }
        if rest_to.zone == ZoneKind::Graveyard {
            rest = format!("into {p} graveyard");
        }
        let left = match (n, take) {
            (Value::Const(a), Value::Const(b)) if a - b == 1 => "the other",
            // "Put all cards revealed this way that weren't put onto the battlefield into
            // your graveyard."
            _ if reveal && take_to.zone == ZoneKind::Battlefield => {
                "{alt:the rest|all cards revealed this way that weren't put onto the battlefield}"
            }
            _ => "the rest",
        };
        // "the other back on top of your library": where it came from.
        if left == "the other"
            && rest_to.zone == ZoneKind::Library
            && matches!(rest_to.position, LibraryPosition::Top)
        {
            rest = format!("{{opt:back}} {rest}");
        }
        // The rest stay where they are ("..., then shuffle").
        if rest_to.zone == ZoneKind::Library
            && matches!(rest_to.position, LibraryPosition::FromTop(_))
        {
            s.push_str(&format!(". {}", capitalize(&take_s)));
            return s;
        }
        if (matches!(filter, Filter::Any) || take_all) && take_to.zone != ZoneKind::Exile {
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
        // "create a 1/1 white Soldier creature token that's tapped and attacking": the
        // status said after the token, or before it ("a tapped and attacking token").
        if attacking {
            let saved = self.gaps.len();
            let base = self.create_token(spec, count, false, false);
            self.gaps.truncate(saved);
            let prefix = self.create_token_status(spec, count, tapped, attacking);
            let st = match (tapped, attacking) {
                (true, true) => "tapped and attacking",
                (true, false) => "tapped",
                _ => "attacking",
            };
            let that = if matches!(count, Value::Const(1)) {
                "that's"
            } else {
                "that are"
            };
            let suffix = match base.find(", where X is") {
                Some(i) if !base.contains("{alt:") => {
                    format!("{} {that} {st}{}", &base[..i], &base[i..])
                }
                _ => format!("{base} {that} {st}"),
            };
            return format!("{{alt:{prefix}|{suffix}}}");
        }
        self.create_token_status(spec, count, tapped, false)
    }

    fn create_token_status(
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
                // "create a 2/2 colorless Wizard Soldier creature token named Cadet":
                // the same token, named after its description.
                return format!(
                    "create {{alt:{name}, {} token{with}|{} token named {name}{with}}}",
                    with_article(&head),
                    with_article(&head)
                );
            }
        }
        let noun_one = join_words(&[status.clone(), head.clone(), "token".into()]);
        let noun_many = join_words(&[status, head, "tokens".into()]);
        let (c, w) = match count {
            Value::Const(1) => (with_article(&noun_one), None::<String>),
            Value::Const(n) => (format!("{} {noun_many}", number_word(*n)), None),
            Value::X if self.extra.x_paid => {
                let w = self.extra.x_words.unwrap_or("that many");
                (format!("{w} {noun_many}"), None)
            }
            Value::X => (format!("X {noun_many}"), None),
            // "create that many 1/1 black Insect creature tokens".
            Value::EventAmount | Value::Prev => (format!("that many {noun_many}"), None),
            other => {
                let v = self.value(other);
                // A number already called X ("where X is ..." said elsewhere).
                if v == "X" {
                    return format!("create X {noun_many}{tail}");
                }
                if v.starts_with("the number of ") {
                    return format!("create X {noun_many}{tail}, where X is {v}");
                }
                // "create a 1/1 black Rat creature token for each 1 life they paid this
                // way": one for each 1 life of the amount.
                if let Some(r) = v.strip_prefix("the life ") {
                    let one = with_article(&noun_one);
                    return format!(
                        "{{alt:create X {noun_many}{tail}, where X is {v}|create {one}{tail} for each 1 life {r}}}"
                    );
                }
                // "create a number of ... tokens equal to ...".
                return either_form(
                    format!("create X {noun_many}{tail}, where X is {v}"),
                    format!("create a number of {noun_many}{tail} equal to {v}"),
                );
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
        // An Equipment token's abilities are about the creature it's attached to:
        // "equipped creature".
        let equipment = spec.subtypes.iter().any(|s| s.as_str() == "Equipment");
        let changeling = spec.abilities.iter().any(|a| {
            matches!(&a.kind, AbilityKind::Keyword(k) if k.kind == crate::keywords::KeywordKind::Changeling)
        });
        for a in &spec.abilities {
            match &a.kind {
                AbilityKind::Keyword(k) => kws.push(self.keyword_lower(k)),
                // The characteristic-defining ability the changeling keyword implies.
                _ if changeling && super::is_changeling_cda(a) => {}
                _ => {
                    let saved = std::mem::replace(&mut self.grants_to_equipment, equipment);
                    others.push(self.nested_ability(a));
                    self.grants_to_equipment = saved;
                }
            }
        }
        super::merge_shared_as_though(&mut others);
        let others: Vec<String> = others.iter().map(|o| format!("\"{o}\"")).collect();
        let mut with = Vec::new();
        if !kws.is_empty() {
            with.push(join_list(&kws, "and"));
        }
        with.extend(others.iter().cloned());
        if !with.is_empty() {
            let list = join_list(&with, "and");
            // Keywords and quoted abilities, listed in either order.
            if !kws.is_empty() && !others.is_empty() {
                let mut rev = others.clone();
                rev.push(join_list(&kws, "and"));
                let rev = join_list(&rev, "and");
                tail.push_str(&format!(" with {{alt:{list}|{rev}}}"));
            } else {
                tail.push_str(&format!(" with {list}"));
            }
        }
        tail.push_str(&where_x);
        (words.join(" "), tail)
    }

    fn emblem(&mut self, abilities: &[Ability]) -> String {
        // The emblem's own abilities call it "this emblem".
        let saved = std::mem::replace(&mut self.in_emblem, Some(self.quote_depth + 1));
        let parts: Vec<String> = abilities
            .iter()
            .map(|a| format!("\"{}\"", self.nested_ability(a)))
            .collect();
        self.in_emblem = saved;
        format!("get an emblem with {}", join_list(&parts, "and"))
    }

    /// The modifications of a resolving effect or static ability as a verb phrase.
    pub(crate) fn modify_effect(
        &mut self,
        what: &Sel,
        mods: &[Modification],
        d: &Duration,
    ) -> String {
        if let Some(s) = self.spell_marker_effect(what, mods) {
            return s;
        }
        if mods.len() == 1 {
            if let Modification::SwitchPT = mods[0] {
                // "switch target creature's power and toughness" / "switch the power and
                // toughness of each of up to two target creatures".
                let (a, b) = self.two_ways(
                    |r| {
                        let w = r.sel(what, Case::Poss);
                        format!("switch {w} power and toughness")
                    },
                    |r| {
                        let o = r.sel(what, Case::Obj);
                        let many = o.contains(" creatures") || o.contains(" permanents");
                        let each = if many { "each of " } else { "" };
                        format!("switch the power and toughness of {each}{o}")
                    },
                );
                let d = self.duration(d);
                let s = if a.contains(['{', '|']) || b.contains(['{', '|']) {
                    a
                } else {
                    format!("{{alt:{a}|{b}}}")
                };
                return join_words(&[s, d]);
            }
            if let Modification::SetController(p) = &mods[0] {
                let w = self.sel(what, Case::Obj);
                let d = self.duration(d);
                let vp = join_words(&[format!("gain control of {w}"), d]);
                return self.with_subject(p, &vp, false);
            }
        }
        self.subject_types = self.sel_types(what);
        let group = match what {
            Sel::All(f) => Some(f.clone()),
            Sel::Var(v) => self.var_defs.iter().find_map(|(x, s, used)| match s {
                Sel::All(f) if x == v && !used => Some(f.clone()),
                _ => None,
            }),
            _ => None,
        };
        let mut w = self.sel(what, Case::Subj);
        // "Creatures you control have base power and toughness X/X": all of them, each.
        if group.is_some()
            && !w.contains('{')
            && matches!(mods.first(), Some(Modification::SetPT(..)))
        {
            if let Some(rest) = w.strip_prefix("each ") {
                // (The comparison doesn't tell "creatures" from "creature".)
                w = format!("{{opt:each}} {rest}");
            }
        }
        let (mut vp, tail) = self.mods_vp_split(mods, true);
        self.subject_types.clear();
        let d = self.duration(d);
        // "gets +2/+2 and has fear for as long as ~ remains tapped": an ability an object
        // has for as long as the effect lasts is one it gains (CR 611.2a).
        if d.starts_with("for as long as") && !vp.contains('{') {
            vp = vp.replacen("gains ", "{alt:gains|has} ", 1);
        }
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
            Sel::TopOfLibrary(..) | Sel::BottomOfLibrary(..) => Some(ZoneKind::Library),
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
            Sel::Union(v) => !v.is_empty() && v.iter().all(|x| self.sel_is_yours(x)),
            // A variable holding cards chosen from your own graveyard earlier.
            Sel::Var(var) => self
                .var_defs
                .iter()
                .rev()
                .find(|(w, _, _)| w == var)
                .is_some_and(|(_, d, _)| {
                    !matches!(d, Sel::Var(_) | Sel::This) && self.sel_is_yours(d)
                }),
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
            // Where the "is a ..." part goes when the parts are written in the order of
            // the modifications (see `BECOMES`).
            let contributes = matches!(
                m,
                Modification::SetPT(..)
                    | Modification::CdaPT(..)
                    | Modification::SetName(_)
                    | Modification::AddTypes(_)
                    | Modification::AddSupertypes(_)
                    | Modification::AddSubtypes(_)
                    | Modification::SetTypes { .. }
                    | Modification::SetBasicLandType(_)
                    | Modification::SetColors(_)
            );
            if contributes && !parts.iter().any(|p| p == BECOMES) {
                parts.push(BECOMES.into());
            }
            match m {
                Modification::ModifyPT(p, t) => {
                    let (ps, pw) = self.pt_amount(p);
                    let (mut ts, tw) = self.pt_amount(t);
                    match (pw, tw) {
                        // "+X/+Y, where X is ... and Y is ...": two different numbers.
                        (Some(a), Some(b))
                            if format!("{p:?}") != format!("{t:?}") && ps == "X" && ts == "X" =>
                        {
                            ts = "Y".into();
                            let y = b.trim_start_matches(", where X is ");
                            where_clauses.push(format!("{a}, and Y is {y}"));
                        }
                        (pw, tw) => {
                            if let Some(w) = pw.or(tw) {
                                where_clauses.push(w);
                            }
                        }
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
                    // "has scavenge. The scavenge cost is equal to its mana cost.": the
                    // keyword's cost is paying the object's own mana cost.
                    let own_mana_cost = k.cost.as_ref().is_some_and(|c| {
                        c.mana.is_none()
                            && matches!(c.parts.as_slice(), [CostPart::PayManaCostOf(s)] if matches!(s.as_ref(), Sel::This))
                    });
                    let saved = std::mem::replace(&mut self.granted_keyword, true);
                    let kw = if own_mana_cost {
                        let mut bare = k.clone();
                        bare.cost = None;
                        let plain = self.keyword_lower(&bare);
                        let n = k.kind.name().to_lowercase();
                        where_clauses.push(format!(". The {n} cost is equal to its mana cost"));
                        plain
                    } else {
                        self.keyword_lower(k)
                    };
                    keywords.push(kw);
                    self.granted_keyword = saved;
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
                Modification::AddKeywordOwnCost(k, reduce) => {
                    if keywords.is_empty() && abilities.is_empty() {
                        parts.push(GRANTS.into());
                    }
                    let saved = std::mem::replace(&mut self.granted_keyword, true);
                    keywords.push(self.keyword_lower(&crate::keywords::Keyword::new(k.kind)));
                    self.granted_keyword = saved;
                    let n = k.kind.name().to_lowercase();
                    where_clauses.push(if *reduce == 0 {
                        format!(". The {n} cost is equal to its mana cost")
                    } else {
                        format!(". Its {n} cost is equal to its mana cost reduced by {{{reduce}}}")
                    });
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
                Modification::AddAbilitiesOf { from, which, terms } => {
                    // The activation instructions for the gained abilities (CR 113.10a).
                    if let Some(n) = terms.max_per_turn {
                        let times = match n {
                            1 => "only once".to_string(),
                            2 => "no more than twice".to_string(),
                            n => format!("no more than {} times", number_word(n as i32)),
                        };
                        where_clauses.push(format!(
                            ". You may activate each of those abilities {times} each turn"
                        ));
                    }
                    if let Some(types) = &terms.spend_as_any_color {
                        let kind = match types.as_slice() {
                            [] => "mana".to_string(),
                            [t] => format!(
                                "{} mana",
                                t.color().map_or("colorless", |c| c.word())
                            ),
                            _ => "mana".to_string(),
                        };
                        where_clauses.push(format!(
                            ". You may spend {kind} as though it were mana of any color to activate those abilities"
                        ));
                    }
                    let kinds = match (which.activated, which.triggered, which.only) {
                        (_, _, Some(AbilityClass::Loyalty)) => "loyalty",
                        (_, _, Some(AbilityClass::Mana)) => "mana",
                        (true, true, _) => "activated and triggered",
                        (false, true, _) => "triggered",
                        _ => "activated",
                    };
                    let from = self.sel(from, Case::Obj);
                    let except = match which.except {
                        Some(AbilityClass::Mana) => " except mana abilities",
                        Some(AbilityClass::Loyalty) => " except for loyalty abilities",
                        _ => "",
                    };
                    abilities.push(format!("all {kinds} abilities of {from}{except}"));
                }
                Modification::RemoveKeyword(k) => {
                    parts.push(format!("loses {}", self.keyword_kind_word(*k)))
                }
                Modification::LoseKeywordWithQuality { kind, quality } => {
                    let k = match quality {
                        Some(q) => {
                            let mut kw = crate::keywords::Keyword::new(*kind);
                            kw.filter = Some(q.clone());
                            self.keyword_lower(&kw)
                        }
                        None if *kind == crate::keywords::KeywordKind::Banding => {
                            "all \"bands with other\" abilities".into()
                        }
                        None => self.keyword_kind_word(*kind),
                    };
                    parts.push(format!("loses {k}"))
                }
                Modification::LoseKeyword(k) => {
                    let t = k
                        .text
                        .as_deref()
                        .map(|t| t.to_lowercase())
                        .unwrap_or_else(|| self.keyword_kind_word(k.kind));
                    parts.push(format!("loses \"{t}\""))
                }
                // "has "..." and loses all other abilities": abilities it gains after
                // losing them (in the same effect, applied in order) stay.
                Modification::RemoveAllAbilities => {
                    let pos = mods
                        .iter()
                        .position(|x| matches!(x, Modification::RemoveAllAbilities))
                        .unwrap_or(0);
                    let regains = mods[pos..].iter().any(|x| {
                        matches!(x, Modification::AddAbility(_) | Modification::AddKeyword(_))
                    });
                    let land = if custom_extra::loses_land_types_and_abilities(mods) {
                        "land types and "
                    } else {
                        ""
                    };
                    parts.push(if regains {
                        format!("loses all {land}{{opt:other}} abilities")
                    } else if !land.is_empty() {
                        format!("loses all {land}abilities")
                    } else {
                        "loses all abilities".into()
                    })
                }
                Modification::CantHaveKeyword(k) => {
                    parts.push(format!("can't have or gain {}", self.keyword_kind_word(*k)))
                }
                // "~'s base power and toughness become equal to that creature's power and
                // toughness": the same object's power and toughness.
                Modification::SetPT(Some(Value::PowerOf(a)), Some(Value::ToughnessOf(b)))
                    if mods.len() == 1 && same_sel(a, b) =>
                {
                    let s = self.sel(a, Case::Poss);
                    parts.push(format!(
                        "{{alt:has base power and toughness equal to {s} power and toughness|base power and toughness become equal to {s} power and toughness}}"
                    ));
                }
                Modification::SetPT(p, t) | Modification::CdaPT(p, t) => {
                    becomes.pt = Some((p.clone(), t.clone()));
                }
                Modification::SwitchPT => parts.push("has its power and toughness switched".into()),
                // "loses all land types and abilities" (with `RemoveAllAbilities`).
                Modification::Custom { name, .. }
                    if name.as_str() == crate::kw::grant_filters::LOSE_ALL_LAND_TYPES
                        && custom_extra::loses_land_types_and_abilities(mods) => {}
                // A copy exception's "it's a legendary Alien": these subtypes instead of
                // its others (CR 707.9b, 205.1a), its card types kept.
                Modification::Custom { name, .. }
                    if name
                        .strip_prefix(crate::kw::copy_exception_mods::ONLY_SUBTYPES)
                        .and_then(|j| serde_json::from_str::<Vec<String>>(j).ok())
                        .is_some() =>
                {
                    let j = &name[crate::kw::copy_exception_mods::ONLY_SUBTYPES.len()..];
                    let v: Vec<String> = serde_json::from_str(j).unwrap_or_default();
                    becomes.subtypes.extend(v);
                    becomes.replaces_creature_types = true;
                }
                // "its starting loyalty is 1" (CR 306.5b).
                Modification::Custom { name, .. }
                    if name
                        .strip_prefix(crate::kw::copy_exception_mods::STARTING_LOYALTY)
                        .is_some_and(|n| n.parse::<i32>().is_ok()) =>
                {
                    let n = &name[crate::kw::copy_exception_mods::STARTING_LOYALTY.len()..];
                    parts.push(format!("{{alt:its starting loyalty is {n}|has starting loyalty {n}}}"));
                }
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
                    becomes.sets_types = subtypes.is_empty();
                    becomes.add_types.extend(types.iter().copied());
                    becomes
                        .subtypes
                        .extend(subtypes.iter().map(|x| x.to_string()));
                }
                Modification::AllCreatureTypes => parts.push(
                    "{alt:is every creature type|gains all creature types|all creature types}".into(),
                ),
                // "becomes a 4/4 Dragon artifact creature": the new creature types replace
                // the old ones (CR 205.1b), which the effect records as removing them.
                Modification::RemoveAllCreatureTypes
                    if mods.iter().any(|m| {
                        matches!(m, Modification::AddSubtypes(v) if !v.is_empty())
                            || matches!(m, Modification::SetTypes { subtypes, .. } if !subtypes.is_empty())
                    }) =>
                {
                    becomes.replaces_creature_types = true
                }
                // "becomes that type" (a chosen creature type): the new creature type
                // replaces the old ones (CR 205.1a); worded with the chosen type below.
                Modification::RemoveAllCreatureTypes
                    if mods.iter().any(|m| matches!(m, Modification::AddChosenType)) => {}
                Modification::RemoveAllCreatureTypes => {
                    parts.push("loses all creature types".into())
                }
                Modification::SetBasicLandType(s) => {
                    becomes.subtypes.extend(s.iter().map(|x| x.to_string()));
                    becomes.land_type = true;
                }
                // "This land is the chosen type" on a land with no land type of its own
                // adds it (the land keeps its abilities).
                Modification::AddChosenType
                    if mods.iter().any(|m| matches!(m, Modification::RemoveAllCreatureTypes)) =>
                {
                    // A static ability says what the objects are ("Creatures you control
                    // are the chosen type"); an effect, what they become.
                    if self.in_static {
                        parts.push("{alt:becomes|is} {alt:that type|the chosen type}".into())
                    } else {
                        parts.push("becomes {alt:that type|the chosen type}".into())
                    }
                }
                Modification::AddChosenType => parts.push(
                    "is the chosen type {opt:in addition to its other types}".into(),
                ),
                Modification::SetChosenBasicLandType => parts.push("is the chosen type".into()),
                Modification::SetColors(cs) => {
                    let w: Vec<String> = color_words(*cs);
                    becomes.colors = Some(if w.is_empty() {
                        "colorless".into()
                    } else if w.len() == 5 {
                        // "~ is all colors" (CR 105.1: the five colors).
                        format!("{{alt:{}|all colors}}", join_list(&w, "and"))
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
                Modification::AddChosenColor => {
                    parts.push("is the chosen color in addition to its other colors".into())
                }
            }
        }
        let has = if gains { "gains" } else { "has" };
        // The same modifications written in their order, the granted abilities apart and a
        // set power and toughness as "with base power and toughness": "gets +2/+2, has
        // flying, and is an Angel in addition to its other types", "becomes a blue Dragon
        // with base power and toughness 4/4, loses all abilities, and gains flying".
        // Cards print either form; the AST is the same.
        let in_order = if becomes.is_empty() {
            None
        } else {
            let saved_gaps = self.gaps.len();
            let mut b2 = std::mem::take(&mut becomes);
            b2.base_form = b2.pt.is_some()
                && (!b2.subtypes.is_empty() || !b2.add_types.is_empty() || b2.colors.is_some());
            let r2 = b2.render(self, &[], &[], gains);
            becomes = b2;
            becomes.base_form = false;
            self.gaps.truncate(saved_gaps);
            r2.map(|(b, still)| {
                let mut ps = parts.clone();
                let mut b = b;
                // "is white in addition to its other colors" joins a type change with no
                // color of its own: "is a white Angel in addition to its other colors and
                // types".
                if becomes.colors.is_none() && becomes.pt.is_none() && becomes.additive {
                    if let Some(i) = ps.iter().position(|p| {
                        p.starts_with("is ") && p.ends_with(" in addition to its other colors")
                    }) {
                        let colors = ps[i]
                            .trim_start_matches("is ")
                            .trim_end_matches(" in addition to its other colors")
                            .to_string();
                        if let Some(rest) = b
                            .strip_prefix("is a ")
                            .or_else(|| b.strip_prefix("is an "))
                            .and_then(|r| r.strip_suffix(" in addition to its other types"))
                        {
                            b = format!(
                                "is {} in addition to its other colors and types",
                                with_article(&format!("{colors} {rest}"))
                            );
                            ps.remove(i);
                        }
                    }
                }
                let base_with = b.contains(" with base power and toughness ");
                if let Some(i) = ps.iter().position(|p| p == BECOMES) {
                    ps[i] = b;
                }
                let mut grants = keywords.clone();
                grants.extend(abilities.iter().cloned());
                if !grants.is_empty() {
                    // "becomes a Bird with base power and toughness 1/1 and flying": the
                    // abilities may join the "with" list.
                    let g = if base_with {
                        format!("{{opt:{has}}} {}", join_list(&grants, "and"))
                    } else {
                        format!("{has} {}", join_list(&grants, "and"))
                    };
                    match ps.iter().position(|p| p == GRANTS) {
                        Some(i) => ps[i] = g,
                        None => ps.push(g),
                    }
                }
                ps.retain(|p| p != GRANTS && p != BECOMES);
                ps.sort_by_key(|p| !p.starts_with("isn't"));
                (join_list(&ps, "and"), still)
            })
        };
        parts.retain(|p| p != BECOMES);
        if let Some((b, still)) = becomes.render(self, &keywords, &abilities, gains) {
            // Negations come first ("except it isn't legendary and is a 4/4 Hero").
            // "loses all abilities and has base power and toughness 0/1": as the
            // modifications are listed.
            let loses_first =
                mods.iter()
                    .position(|x| matches!(x, Modification::RemoveAllAbilities))
                    .zip(mods.iter().position(|x| {
                        matches!(x, Modification::SetPT(..) | Modification::CdaPT(..))
                    }))
                    .is_some_and(|(a, b)| a < b);
            let at = parts
                .iter()
                .take_while(|p| {
                    p.starts_with("isn't")
                        || (loses_first && p.as_str() == "loses all abilities")
                        || p.as_str() == custom_extra::LOSES_THIS_ABILITY
                })
                .count();
            parts.insert(at, b);
            if !still.is_empty() {
                where_clauses.push(still);
            }
            parts.retain(|p| p != GRANTS);
            parts.sort_by_key(|p| !p.starts_with("isn't"));
            let s = join_list(&parts, "and");
            return match in_order {
                Some((s2, _)) if s2 != s => (format!("{{alt:{s}|{s2}}}"), where_clauses.concat()),
                _ => (s, where_clauses.concat()),
            };
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
        // "loses hexproof, indestructible, and shroud": one verb for the abilities lost.
        let lost = |p: &String| {
            p.strip_prefix("loses ")
                .filter(|r| !r.contains(['"', '{']) && !r.starts_with("all "))
                .map(str::to_string)
        };
        let mut merged: Vec<String> = Vec::new();
        let mut i = 0;
        while i < parts.len() {
            let mut run = Vec::new();
            while let Some(k) = parts.get(i + run.len()).and_then(lost) {
                run.push(k);
            }
            if run.len() >= 2 {
                merged.push(format!("loses {}", join_list(&run, "and")));
                i += run.len();
            } else {
                merged.push(parts[i].clone());
                i += 1;
            }
        }
        let parts = merged;
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
        let old = self.exceptions_vp(mods);
        // Cards list the exceptions one clause each ("it's 4/3, it's a Vehicle artifact
        // ..., and it has flying"); the AST is the same.
        match self.exception_list(mods) {
            Some(list) => {
                let mut alts = vec![old];
                alts.extend(list);
                alts.dedup();
                if alts.len() == 1 {
                    alts.remove(0)
                } else {
                    format!("{{alt:{}}}", alts.join("|"))
                }
            }
            None => old,
        }
    }

    fn exceptions_vp(&mut self, mods: &[Modification]) -> String {
        let vp = self.mods_vp(mods, false);
        // "except it isn't legendary, it's a 4/4 Hero": each exception has its subject.
        // A copy's "except it's a 4/4 black Zombie" keeps the copied types (the rulings
        // on eternalize and similar copies), with or without "in addition to its other
        // types".
        let vp = vp
            .replace(" and is ", " and it is ")
            .replace(" and has ", " and it has ")
            .replace(
                " in addition to its other types",
                " {opt:in addition to its other types}",
            );
        // "except it's 1/1": its base power and toughness.
        if let Some(pt) = vp.strip_prefix("has base power and toughness ") {
            if !pt.contains(' ') {
                return format!("it {{alt:has base power and toughness {pt}|is {pt}}}");
            }
        }
        format!("it {vp}")
    }

    /// A choice ("a color", "a creature type", "an opponent").
    pub(crate) fn choice(&mut self, k: &ChoiceKind) -> String {
        match k {
            ChoiceKind::Color => "a color".into(),
            ChoiceKind::ColorOtherThan(c) => format!("a color other than {}", c.word()),
            ChoiceKind::Colors => "one or more colors".into(),
            ChoiceKind::ColorsExactly(n) => {
                format!("{} colors", crate::oracle::render::number_word(*n as i32))
            }
            // CR 205.3i: every land type.
            ChoiceKind::OneOf(v) if *v == crate::types::land_types() => "a land type".into(),
            ChoiceKind::OneOf(v) => join_list(v, "or"),
            ChoiceKind::CreatureType => "a creature type".into(),
            ChoiceKind::CreatureTypeOtherThan(v) => {
                format!("a creature type other than {}", join_list(v, "or"))
            }
            // "choose any card name": any card in the Oracle card reference (CR 201.4).
            ChoiceKind::CardName => "{alt:a|any} card name".into(),
            ChoiceKind::CardNameFiltered(f) => format!("a {f} card name"),
            ChoiceKind::Number { min, max } => self.number_choice(*min, *max),
            ChoiceKind::Opponent => "an opponent".into(),
            ChoiceKind::Player => "a player".into(),
            ChoiceKind::BasicLandType => "a basic land type".into(),
            ChoiceKind::CardType => "a card type".into(),
            ChoiceKind::OddOrEven => "odd or even".into(),
            ChoiceKind::Word(v) => join_list(v, "or"),
            ChoiceKind::Distinct(n, v)
                if *v == ["Plains", "Island", "Swamp", "Mountain", "Forest"] =>
            {
                format!(
                    "{} basic land types",
                    crate::oracle::render::number_word(*n as i32)
                )
            }
            ChoiceKind::Players(n) => {
                format!("{} players", crate::oracle::render::number_word(*n as i32))
            }
            // "Choose an order for artifacts, creatures, and lands": all of the card types
            // named, each chosen in turn (the first, second, ... type).
            ChoiceKind::Distinct(n, v)
                if *n as usize == v.len() && v.iter().all(|w| CardType::from_word(w).is_some()) =>
            {
                let ws: Vec<String> = v.iter().map(|w| plural(w)).collect();
                format!("an order for {}", join_list(&ws, "and"))
            }
            ChoiceKind::Distinct(n, v) => format!(
                "{} abilities from among {}",
                crate::oracle::render::number_word(*n as i32),
                join_list(v, "and")
            ),
        }
    }

    /// What mana an effect adds.
    /// The verb of "[objects] [verb] this way" for the objects an earlier instruction
    /// acted on, if `inner` holds them.
    fn for_each_this_way(&self, inner: &Sel) -> Option<&'static str> {
        let v = match inner {
            Sel::Before(b) => b.as_ref(),
            other => other,
        };
        self.this_way_of(v).map(|(verb, _, _)| verb)
    }

    pub(crate) fn mana_production(&mut self, m: &ManaProduction) -> String {
        match m {
            ManaProduction::Fixed(v) => v.iter().map(|t| mana_symbol(*t)).collect(),
            ManaProduction::AnyOneColor(n) => match n {
                Value::Const(1) => "one mana of any color".into(),
                Value::Const(k) => format!("{} mana of any one color", number_word(*k)),
                other => {
                    let (a, w) = self.amount(other);
                    let w = w.unwrap_or_default();
                    // "Add one mana of the color of your choice for each charge counter on
                    // ~": one color chosen for all of it (Selvala, Eager Trailblazer's
                    // rulings), the same as X mana of any one color.
                    match w.strip_prefix(", where X is the number of ") {
                        Some(each) if a == "X" && !each.contains(['{', ',']) => format!(
                            "{{alt:X mana of any one color{w}|one mana of the color of your choice for each {each}}}"
                        ),
                        _ => format!("{a} mana of any one color{w}"),
                    }
                }
            },
            // "Remove all charge counters from ~. Add one mana of any color for each charge
            // counter removed this way."
            ManaProduction::AnyCombination(Value::Var(v))
                if self.removed_kind.is_some()
                    && self
                        .stored_values
                        .iter()
                        .any(|(x, val, _)| x == v && matches!(val, Value::Prev)) =>
            {
                let k = self.removed_kind.clone().unwrap_or_default();
                format!("one mana of any color for each {k} removed this way")
            }
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
                let s = format!(
                    "{a} mana in any combination of {}{}",
                    join_list(&syms, "and/or"),
                    w.unwrap_or_default()
                );
                // "Add {R} or {G} for each Raccoon you control": one of them for each
                // (each chosen separately).
                let counted = match n {
                    Value::Count(_) | Value::CountSel(_) => {
                        let v = self.value(n);
                        v.strip_prefix("the number of ").map(singular_head)
                    }
                    _ => None,
                };
                match counted {
                    Some(c) if !s.contains(['|', '\n']) && !c.contains(['|', '\n']) => {
                        format!("{{alt:{s}|{} for each {c}}}", join_list(&syms, "or"))
                    }
                    _ => s,
                }
            }
            ManaProduction::OneOf(types) => {
                let syms: Vec<String> = types.iter().map(|t| mana_symbol(*t)).collect();
                join_list(&syms, "or")
            }
            ManaProduction::CouldProduce(Filter::In(s))
                if matches!(**s, Sel::Var(crate::ability::vars::SACRIFICED)) =>
            {
                "one mana of any type the sacrificed land could produce".into()
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
                        // "Add that much {R}": an amount an earlier instruction or the
                        // event named.
                        if s.starts_with("{alt:that much") || s == "that many" {
                            return format!("{{alt:that much|that many}} {sym}");
                        }
                        // "Add {B} for each charge counter on ~": a count of things.
                        match s.strip_prefix("the number of ") {
                            Some(rest) => format!(
                                "{{alt:{sym} for each {}|an amount of {sym} equal to {s}}}",
                                singular_head(rest)
                            ),
                            None => format!("an amount of {sym} equal to {s}"),
                        }
                    }
                }
            }
            ManaProduction::AnyColorAmong(Filter::Source) => "one mana of any of ~'s colors".into(),
            ManaProduction::AnyColorAmong(Filter::And(v))
                if v.iter()
                    .any(|f| matches!(f, Filter::In(s) if matches!(**s, Sel::Linked))) =>
            {
                "one mana of any of the exiled {alt:card's|cards'} colors".into()
            }
            ManaProduction::AnyColorAmong(f) => {
                let n = self.noun(f, Num::Many);
                format!("one mana of any color among {n}")
            }
            ManaProduction::DistinctColors(2, each) if *each > 1 => {
                let w = number_word(*each as i32);
                format!("{w} mana of any one color and {w} mana of any other color")
            }
            ManaProduction::DistinctColors(colors, _) => {
                format!("{} mana of different colors", number_word(*colors as i32))
            }
            ManaProduction::EachColorAmong(f) => {
                let n = self.noun(f, Num::Many);
                format!("one mana of each color among {n}")
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
            ManaProduction::DevotionToChosenColor => {
                "an amount of mana of that color equal to your devotion to that color".into()
            }
        }
    }

    pub(crate) fn mana_restriction(&mut self, r: &crate::mana::ManaRestriction) -> String {
        if let crate::mana::ManaRestriction::NotGeneric = r {
            return "This mana can't be spent to pay generic mana costs".into();
        }
        // What the mana can't pay for: a sentence of its own.
        if let crate::mana::ManaRestriction::NotNonartifactSpell
        | crate::mana::ManaRestriction::NotCastSpell(_) = r
        {
            return self.mana_restriction_purpose(r);
        }
        format!("Spend this mana only {}", self.mana_restriction_purpose(r))
    }

    fn mana_restriction_purpose(&mut self, r: &crate::mana::ManaRestriction) -> String {
        use crate::mana::ManaRestriction as M;
        match r {
            M::SpellOfType(t) => format!("to cast {} spells", t.word()),
            M::SpellWithSubtype(s) => format!("to cast {s} spells"),
            M::SpellOfChosenType => "to cast a creature spell of the chosen type".into(),
            M::SpellsOnly => "to cast spells".into(),
            M::AbilitiesOnly => "to activate {alt:abilities|an ability}".into(),
            M::XCostsOnly => "on costs that {alt:include|contain} {X}".into(),
            M::ArtifactSpellOrAbility => {
                "to cast artifact spells or activate abilities of artifacts".into()
            }
            M::InstantOrSorcery => "to cast instant or sorcery spells".into(),
            M::NoncreatureSpell => "to cast noncreature spells".into(),
            M::NotNonartifactSpell => {
                "This mana can't be spent to cast {alt:a nonartifact spell|nonartifact spells}"
                    .into()
            }
            M::NotGeneric => "This mana can't be spent to pay generic mana costs".into(),
            // "to cast an artifact spell or activate an ability": "to" said once.
            M::AnyOf(v) => {
                let parts: Vec<String> = v
                    .iter()
                    .enumerate()
                    .map(|(i, x)| {
                        let p = self.mana_restriction_purpose(x);
                        if i > 0 {
                            format!("{{opt:to}} {}", p.strip_prefix("to ").unwrap_or(&p))
                        } else {
                            p
                        }
                    })
                    .collect();
                join_list(&parts, "or")
            }
            // "to cast Dragon spells" / "to cast a Dragon spell".
            M::CastSpell(f) => {
                let many = self.noun(&f.0, Num::Many);
                let many = if many.contains("spell") {
                    many
                } else {
                    format!("{many} spells")
                };
                let one = self.noun(&f.0, Num::One);
                let one = if one.contains("spell") {
                    one
                } else {
                    format!("{one} spell")
                };
                format!("to cast {{alt:{many}|{}}}", with_article(&one))
            }
            // "to activate abilities of artifacts" / "... an ability of an artifact source".
            M::ActivateAbilityOf(f) => {
                let n = self.noun(&f.0, Num::Many);
                let one = self.noun(&f.0, Num::One);
                let a = with_article(&one);
                format!("to activate {{alt:abilities of {n}|an ability of {a}|an ability of {a} source}}")
            }
            M::ClassLevel => "to gain a Class level".into(),
            M::Purpose(crate::mana::SpendPurpose::UnlockDoor) => "to unlock doors".into(),
            M::Purpose(crate::mana::SpendPurpose::Keyword(k)) => {
                use crate::keywords::KeywordKind as K;
                match k {
                    K::CumulativeUpkeep => "to pay cumulative upkeep costs".into(),
                    K::Foretell => "to foretell {alt:cards|a card from your hand}".into(),
                    k => {
                        let n = k.name().to_lowercase();
                        format!("to activate {{alt:{n} abilities|an {n} ability|a {n} ability}}")
                    }
                }
            }
            M::CostWithColorless => "to pay a cost that contains {C}".into(),
            // "This mana can't be spent to cast spells from your hand."
            M::NotCastSpell(f) => {
                let saved = self.default_head.replace("spell");
                let n = self.noun(&f.0, Num::Many);
                self.default_head = saved;
                let n = n.replace("permanents in your hand", "spells from your hand");
                let n = n.replace(" in your hand", " from your hand");
                format!("This mana can't be spent to cast {n}")
            }
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
        if r.choose_one {
            s.push_str(" and choose one result");
        }
        for row in &r.table {
            let range = match row.hi {
                Some(h) if row.lo == i64::MIN => format!("{h} or less"),
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
        let mut s = if let (true, Some(max)) = (c.until_lose, &c.max_flips) {
            let n = self.value(max);
            format!("flip a coin {n} times or until you lose a flip, whichever comes first")
        } else if c.until_lose {
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
                        let top = self.top_cards(other, &p);
                        format!("{verb} {top}")
                    }
                }
            }
            // CR 701.38a: each player votes, starting with a given player, for one of
            // the words or objects.
            K::Vote => {
                let names: Vec<String> = options.iter().map(|(n, _)| n.clone()).collect();
                // "each player votes for a player" / "for another player".
                let for_player = match what {
                    Sel::Players(PlayerRef::EachPlayer) => Some("a player"),
                    Sel::Players(PlayerRef::EachOtherPlayer) => Some("another player"),
                    _ => None,
                };
                let what_s = if let (Some(p), true) = (for_player, names.is_empty()) {
                    p.to_string()
                } else if names.is_empty() {
                    let f = match what {
                        Sel::All(f) => f.clone(),
                        _ => return self.gap("a vote for objects"),
                    };
                    let head = self.noun(&f, Num::One);
                    self.vote_noun = Some(if format!("{f:?}").contains("Permanent") {
                        "permanent".to_string()
                    } else {
                        head.rsplit(' ').next().unwrap_or("permanent").to_string()
                    });
                    self.noun_det(&f, Det::A)
                } else {
                    self.vote_words = names.clone();
                    join_list(&names, "or")
                };
                let start = self.player(who, Case::Obj);
                format!("starting with {start}, each player votes for {what_s}")
            }
            K::VillainousChoice => {
                let mut s: String = "face a villainous choice —".into();
                for (_, e) in options {
                    let t = self.effect_sentences(e);
                    s.push_str(&format!("\n• {t}"));
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
                if let (K::Behold, Sel::All(f), Value::Const(1), PlayerRef::You) =
                    (other, what, n, who)
                {
                    return self.behold_forms(f);
                }
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
    /// New creature types replace the old ones (CR 205.1a).
    replaces_creature_types: bool,
    land_type: bool,
    /// The card types replace the old ones ("is an enchantment and loses all other card
    /// types").
    sets_types: bool,
    name: Option<String>,
    /// Write a set power and toughness as "with base power and toughness 4/4" after the
    /// types (newer Oracle wording) rather than as "a 4/4 ..." before them.
    base_form: bool,
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
            if self.pt.is_none()
                && self.add_types.is_empty()
                && self.subtypes.is_empty()
                && self.supertypes.is_empty()
                && self.colors.is_none()
            {
                return Some((format!("is named {n}"), String::new()));
            }
        }
        let mut words: Vec<String> = Vec::new();
        let mut base_pt = None;
        if let Some(pt) = pt {
            if self.base_form {
                base_pt = Some(pt);
            } else {
                words.push(pt);
            }
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
        let mut phrase = words.join(" ");
        // "is every basic land type" (CR 305.6: the five basic land types).
        let mut basic = self.subtypes.clone();
        basic.sort();
        if basic == ["Forest", "Island", "Mountain", "Plains", "Swamp"]
            && words.len() == 5
            && self.additive
        {
            phrase = format!("{{alt:{}|every basic land type}}", with_article(&phrase));
        }
        let is_adjective_only =
            self.add_types.is_empty() && self.subtypes.is_empty() && self.pt.is_none();
        let mut s = if is_adjective_only || phrase.starts_with("{alt:") {
            format!("is {phrase}")
        } else {
            format!("is {}", with_article(&phrase))
        };
        // CR 205.1b: an object that becomes an "artifact creature" keeps its other types.
        let artifact_creature = {
            let mut t = self.add_types.clone();
            t.sort_by_key(|x| x.word());
            t == [CardType::Artifact, CardType::Creature]
                && self.subtypes.is_empty()
                && self.supertypes.is_empty()
                && self.colors.is_none()
        };
        // "becomes an artifact creature with base power and toughness 4/5", "... with
        // power and toughness each equal to its mana value".
        if artifact_creature {
            if let Some((Some(p), Some(t))) = &self.pt {
                let ps = r.value(p);
                let ts = r.value(t);
                let with_pt = if format!("{p:?}") == format!("{t:?}") && !Renderer::is_simple(p) {
                    format!("power and toughness each equal to {ps}")
                } else {
                    format!("base power and toughness {ps}/{ts}")
                };
                let a = s.trim_start_matches("is ").to_string();
                s = format!("is {{alt:{a}|an artifact creature with {with_pt}}}");
            }
        }
        // A resolving spell or ability's effect: "It becomes an enchantment" or "It's an
        // enchantment" (the same continuous effect, CR 611.2a).
        if gains {
            s = s.replacen("is ", "{alt:becomes|is} ", 1);
        }
        if let Some(n) = &self.name {
            s.push_str(&format!(" named {n}"));
        }
        if let Some(pt) = base_pt {
            s.push_str(&format!(" with base power and toughness {pt}"));
        }
        s.push_str(&with);
        // CR 205.1a: new card types replace the old ones, which cards may say.
        if self.sets_types && !self.additive {
            s.push_str(" {opt:and loses all other card types}");
        }
        let mut still = String::new();
        // Adding a supertype ("is snow", "is legendary") never removes anything.
        let only_supertypes =
            self.add_types.is_empty() && self.subtypes.is_empty() && !self.supertypes.is_empty();
        // "becomes a Human Warrior": only its creature types change (CR 205.1a); it
        // keeps its card types without adding any, so nothing is "in addition".
        let only_creature_types = self.replaces_creature_types && self.add_types.is_empty();
        // CR 205.1b: "becomes a 4/4 Angel artifact creature" keeps its other card types
        // and subtypes but creature types, which the Angel replaces.
        let typed_artifact_creature = {
            let mut t = self.add_types.clone();
            t.sort_by_key(|x| x.word());
            t == [CardType::Artifact, CardType::Creature]
                && self.replaces_creature_types
                && self.supertypes.is_empty()
        };
        if self.additive && !self.land_type && !only_supertypes && !only_creature_types {
            if artifact_creature || typed_artifact_creature {
                s.push_str(" {opt:in addition to its other types}");
                return Some((s, still));
            }
            // An effect that adds types keeps the old ones: cards say "It's still a land"
            // when they know what the object was, else "in addition to its other types".
            let known: Vec<CardType> = r
                .subject_types
                .iter()
                .copied()
                .filter(|t| !self.add_types.contains(t))
                .collect();
            // "It becomes an Aura with enchant creature": an Aura is an enchantment
            // (CR 303.4), so its card types stay, said or not.
            if self.add_types.is_empty() && self.subtypes.iter().all(|t| t == "Aura") {
                s.push_str(" {opt:in addition to its other types}");
            } else if r.subject_types.is_empty() || self.add_types.is_empty() {
                s.push_str(" in addition to its other types");
            } else if !known.is_empty() {
                let w: Vec<String> = known.iter().map(|t| t.word().to_string()).collect();
                still = format!(". It's still {}", with_article(&w.join(" ")));
            }
        }
        // "have base power and toughness 6/6 and are Oozes in addition to their other
        // types": the same base power and toughness and creature types, said in order.
        if let (Some((Some(p), Some(t))), true) = (
            &self.pt,
            self.add_types.is_empty()
                && !self.subtypes.is_empty()
                && self.supertypes.is_empty()
                && self.colors.is_none()
                && self.name.is_none()
                && self.additive
                && !self.land_type
                && grants.is_empty()
                && !gains
                && s.ends_with(" in addition to its other types"),
        ) {
            let pt = format!("{}/{}", r.value(p), r.value(t));
            let kinds = with_article(&self.subtypes.join(" "));
            s = format!(
                "{{alt:{s}|has base power and toughness {pt} and is {kinds} in addition to its other types}}"
            );
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

/// Where the "is a [types]" part goes among the parts of a verb phrase written in the
/// order of the modifications.
const BECOMES: &str = "\u{1}becomes";

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
        Sel::TopOfLibrary(_, n) | Sel::BottomOfLibrary(_, n) => !matches!(n, Value::Const(1)),
        _ => false,
    }
}

fn until_event(u: &UntilEvent) -> String {
    match u {
        UntilEvent::SourceLeavesBattlefield => "~ leaves the battlefield".into(),
        UntilEvent::OpponentBecomesMonarch => "an opponent becomes the monarch".into(),
        UntilEvent::TargetLeavesBattlefield(_) => "that permanent leaves the battlefield".into(),
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
/// Whether an instruction exiles objects (a card exiled with a permission to play it).
fn exiles(e: &Effect) -> bool {
    let d = format!("{e:?}");
    d.contains("Exile {") || d.contains("zone: Exile") || d.contains("ExileUntil")
}

/// Two wordings of the same instruction ("X tokens, where X is ..." / "a number of tokens
/// equal to ..."), as alternatives when they can be written as such.
pub(crate) fn either_form(a: String, b: String) -> String {
    if a.contains(['{', '|', '}']) || b.contains(['{', '|', '}']) {
        a
    } else {
        format!("{{alt:{a}|{b}}}")
    }
}

pub(crate) fn third_person(vp: &str) -> String {
    // An adverb before the verb: "secretly chooses".
    if let Some(r) = vp.strip_prefix("secretly ") {
        return format!("secretly {}", third_person(r));
    }
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

/// For a counter effect wrapped in a self-replacement effect for the countered spell's move
/// from the stack to a graveyard (see `oracle/patterns/replacements_counter.rs`): `Some`
/// with the kind of spell that moves elsewhere ("if an artifact or creature spell is
/// countered this way"), if only some do.
fn countered_this_way<'a>(effect: &'a Effect, filter: &'a Filter) -> Option<Option<&'a Filter>> {
    let what = match effect {
        Effect::CounterSpell { what } => what,
        Effect::PayOptional {
            then, otherwise, ..
        } if matches!(**then, Effect::Noop) => match &**otherwise {
            Effect::CounterSpell { what } => what,
            _ => return None,
        },
        _ => return None,
    };
    let is_what = |f: &Filter| matches!(f, Filter::In(s) if same_sel(s, what));
    match filter {
        f if is_what(f) => Some(None),
        Filter::And(v) if v.len() == 2 && is_what(&v[0]) => Some(Some(&v[1])),
        _ => None,
    }
}

/// Whether `e` is "sacrifice [s]", or "you may sacrifice [s]. If you do, ...": nothing in
/// it happens unless you sacrifice `s`.
fn starts_by_sacrificing(e: &Effect, s: &Sel) -> bool {
    let sacrifices = |e: &Effect| matches!(e, Effect::SacrificeObjects { what } if format!("{what:?}") == format!("{s:?}"));
    match e {
        Effect::Seq(v) => match v.split_first() {
            Some((
                Effect::May {
                    who: PlayerRef::You,
                    effect,
                },
                rest,
            )) => {
                sacrifices(effect)
                    && rest.iter().all(|x| {
                        matches!(x, Effect::If { cond: Condition::PrevHappened, otherwise, .. }
                            if matches!(**otherwise, Effect::Noop))
                    })
            }
            _ => false,
        },
        e => sacrifices(e),
    }
}

/// A counted noun phrase in the singular: "charge counters on ~" -> "charge counter on
/// ~", "creatures in your party" -> "creature in your party" (the head noun is the word
/// before the first preposition or clause).
pub(crate) fn singular_head(phrase: &str) -> String {
    let words: Vec<&str> = phrase.split(' ').collect();
    let preps = [
        "on", "in", "you", "your", "among", "that", "with", "attached",
    ];
    let head = words
        .iter()
        .position(|w| preps.contains(w))
        .unwrap_or(words.len())
        .saturating_sub(1);
    words
        .iter()
        .enumerate()
        .map(|(i, w)| {
            if i == head && w.ends_with('s') && !w.ends_with("ss") {
                w[..w.len() - 1].to_string()
            } else {
                w.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

impl Renderer<'_> {
    /// The time of a one-shot delayed trigger (CR 603.7): "at the beginning of the next
    /// end step", "at the beginning of your next main phase", "at the beginning of that
    /// player's next end step", "at this turn's next end of combat".
    pub(crate) fn delay_text(&self, t: &TriggerCond) -> Option<String> {
        let (inner, this_turn) = match t {
            TriggerCond::ThisTurn(i) => (&**i, true),
            other => (other, false),
        };
        let parts: Vec<(TriggerStep, PlayerRel)> = match inner {
            TriggerCond::BeginningOf { step, whose } => vec![(*step, *whose)],
            TriggerCond::AnyOf(v) => v
                .iter()
                .map(|c| match c {
                    TriggerCond::BeginningOf { step, whose } => Some((*step, *whose)),
                    _ => None,
                })
                .collect::<Option<Vec<_>>>()?,
            _ => return None,
        };
        let whose = parts.first()?.1;
        if parts.iter().any(|(_, w)| *w != whose) {
            return None;
        }
        let steps: Vec<TriggerStep> = parts.iter().map(|(s, _)| *s).collect();
        let step = match steps[..] {
            [TriggerStep::PrecombatMain, TriggerStep::PostcombatMain] => "main phase",
            [s] => self.step_name(s),
            _ => return None,
        };
        let turn = if this_turn { " this turn" } else { "" };
        Some(match (steps[..].first()?, whose) {
            (TriggerStep::EndOfCombat, PlayerRel::Any) if this_turn => {
                "at this turn's next end of combat".to_string()
            }
            (TriggerStep::EndOfCombat, PlayerRel::Any) => "at end of combat".to_string(),
            (TriggerStep::Upkeep, PlayerRel::Any) if !this_turn => {
                "at the beginning of the next turn's upkeep".to_string()
            }
            (_, PlayerRel::Any) => format!("at the beginning of the next {step}{turn}"),
            (_, PlayerRel::You) => format!("at the beginning of your next {step}{turn}"),
            (_, PlayerRel::Var(_)) => {
                format!("at the beginning of {{alt:that player's|their}} next {step}{turn}")
            }
            _ => return None,
        })
    }
}

/// A counted action whose count is how many times some mana was spent to cast the object
/// ("each opponent discards a card for each {B}{B} spent"), as the [`Effect::Repeat`] of
/// the action on one.
fn per_mana_spent(e: &Effect) -> Option<Effect> {
    let spent = |v: &Value| matches!(v, Value::Custom(c) if c.starts_with("mana_spent_of:"));
    let mut one = e.clone();
    let n = match &mut one {
        Effect::AddCounters { n, .. }
        | Effect::Discard { n, .. }
        | Effect::Mill { n, .. }
        | Effect::GainLife { n, .. }
        | Effect::LoseLife { n, .. } => n,
        Effect::CreateToken { count, .. } => count,
        _ => return None,
    };
    let ok = match &*n {
        Value::Div(v, _, false) => spent(v),
        v => spent(v),
    };
    if !ok {
        return None;
    }
    let times = std::mem::replace(n, Value::c(1));
    Some(Effect::Repeat {
        times,
        effect: Box::new(one),
    })
}

/// Two life instructions with the same amount, worded by a "where X is" clause (not a
/// plain number), for different players.
fn same_life_amount(a: &Effect, b: &Effect) -> bool {
    let amount = |e: &Effect| match e {
        Effect::LoseLife { who, n } | Effect::GainLife { who, n } => Some((who.clone(), n.clone())),
        _ => None,
    };
    let (Some((wa, na)), Some((wb, nb))) = (amount(a), amount(b)) else {
        return false;
    };
    format!("{na:?}") == format!("{nb:?}")
        && format!("{wa:?}") != format!("{wb:?}")
        && !matches!(
            na,
            Value::Const(_) | Value::X | Value::Prev | Value::EventAmount
        )
}

/// The ways to say a player's zone after the player was named ("that player's", "their"),
/// each in full so a phrase built from it is one alternative.
fn possessives(r: &mut Renderer<'_>, p: &PlayerRef) -> Vec<String> {
    let poss = r.player(p, Case::Poss);
    match poss.strip_prefix("{alt:").and_then(|x| x.strip_suffix('}')) {
        Some(inner) if !inner.contains('{') => inner.split('|').map(str::to_string).collect(),
        _ => vec![poss],
    }
}
