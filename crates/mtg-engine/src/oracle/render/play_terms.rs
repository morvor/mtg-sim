//! Permissions to play cards that come with terms ([`Effect::WithPlayTerms`]): "you may
//! cast that card" (not play it as a land, CR 305.9), "If you cast a spell this way, pay
//! life equal to its mana value rather than pay its mana cost" (CR 118.9), "you may spend
//! mana as though it were mana of any color to cast that spell" (CR 609.4b), "A spell cast
//! this way costs {2} more to cast" (CR 601.2f), "Each land played this way enters tapped".

use super::nouns::Det;
use super::*;

/// `If { [x] isn't a land, [cast-only permission for x] }`: a land can't be cast (CR
/// 305.9), so the check says nothing a permission to cast doesn't say.
/// Also `[x] is in exile and isn't a land`: the permission is for the card as long as
/// it's exiled (the `bool`).
fn cast_only_of<'a>(e: &'a Effect, x: &Sel) -> Option<(PlayTerms, &'a Effect, bool)> {
    let Effect::If {
        cond: Condition::SelMatches(s, cond),
        then,
        otherwise,
    } = e
    else {
        return None;
    };
    let not_land = |f: &Filter| matches!(f, Filter::Not(l) if matches!(l.as_ref(), Filter::Type(CardType::Land)));
    let in_exile = match cond {
        f if not_land(f) => false,
        // "You may cast ~ for as long as it remains exiled": a permission to cast.
        Filter::InZone(ZoneKind::Exile) => true,
        Filter::And(v)
            if v.len() == 2
                && v.iter().any(not_land)
                && v.iter()
                    .any(|f| matches!(f, Filter::InZone(ZoneKind::Exile))) =>
        {
            true
        }
        _ => return None,
    };
    if !matches!(otherwise.as_ref(), Effect::Noop) || format!("{s:?}") != format!("{x:?}") {
        return None;
    }
    // "..., and mana of any type can be spent to cast it": the permission and a rule for
    // spending mana on that card, the terms said together.
    let (with_terms, any_type) = match then.as_ref() {
        Effect::Seq(v) => match v.as_slice() {
            [w, Effect::SpendAnyTypeMana {
                who: PlayerRef::You,
                what,
                duration: Duration::Permanent,
            }] if format!("{what:?}") == format!("{x:?}") => (w, true),
            _ => return None,
        },
        other => (other, false),
    };
    let Effect::WithPlayTerms { terms, effect } = with_terms else {
        return None;
    };
    let Effect::GrantPlayPermission { what, duration, .. } = effect.as_ref() else {
        return None;
    };
    if any_type && !matches!(duration, Duration::Permanent) {
        return None;
    }
    let mut terms = terms.clone();
    terms.spend_any_type |= any_type;
    (terms.spells_only && format!("{what:?}") == format!("{x:?}"))
        .then_some((terms, effect, in_exile))
}

impl Renderer<'_> {
    /// "You may cast that card": a permission to cast compiled with a check that the card
    /// isn't a land, for each card it's about.
    pub(crate) fn cast_only_permission(&mut self, e: &Effect) -> Option<String> {
        match e {
            Effect::ForEach { sel, var, effect } => {
                let (terms, grant, in_exile) = cast_only_of(effect, &Sel::Var(*var))?;
                let Effect::GrantPlayPermission {
                    who,
                    duration,
                    free,
                    ..
                } = grant
                else {
                    return None;
                };
                // "Exile ~ from your hand: ... You may cast ~ for as long as it remains
                // exiled": the card the cost exiled is the object itself.
                let what = match sel {
                    Sel::Var(v) if *v == crate::zones::COST_MOVED && self.cost_exiles_self => {
                        Sel::This
                    }
                    other => other.clone(),
                };
                let g = Effect::GrantPlayPermission {
                    who: who.clone(),
                    what,
                    duration: duration.clone(),
                    free: *free,
                };
                let saved = self.after_exile;
                self.after_exile |= in_exile;
                let s = self.with_play_terms(&terms, &g);
                self.after_exile = saved;
                Some(s)
            }
            Effect::If {
                cond: Condition::SelMatches(x, _),
                ..
            } => {
                let (terms, grant, in_exile) = cast_only_of(e, x)?;
                // "Choose target artifact card in your graveyard. You may cast that card
                // this turn."
                let intro = match x {
                    Sel::Target(i)
                        if !self.introduced.get(*i as usize).copied().unwrap_or(true) =>
                    {
                        let t = self.target_mention(*i, super::players::Case::Obj);
                        format!("choose {t}. ")
                    }
                    _ => String::new(),
                };
                let saved = self.after_exile;
                self.after_exile |= in_exile;
                let s = self.with_play_terms(&terms, grant);
                self.after_exile = saved;
                // Cards may say the check anyway: "If it's a nonland card, you may cast
                // that card this turn."
                let check = if intro.is_empty() && matches!(x, Sel::Var(_)) {
                    "{opt:if it's a nonland card,} "
                } else {
                    ""
                };
                Some(format!("{intro}{check}{s}"))
            }
            _ => None,
        }
    }

    /// The verb of a permission given under the current terms: "cast" or "play".
    pub(crate) fn permission_verb(&self) -> &'static str {
        if self.play_terms.as_ref().is_some_and(|t| t.spells_only) {
            "cast"
        } else {
            "play"
        }
    }

    pub(crate) fn with_play_terms(&mut self, terms: &PlayTerms, effect: &Effect) -> String {
        // "During any turn you attacked with a token, you may play that card": as long as
        // it happened this turn.
        let any_turn = terms
            .condition
            .as_ref()
            .and_then(|c| self.attempt(|r| r.during_any_turn(c)));
        let saved = self.play_terms.replace(terms.clone());
        let saved_any = std::mem::replace(&mut self.any_turn_permission, any_turn.is_some());
        let mut s = self.effect(effect);
        self.any_turn_permission = saved_any;
        self.play_terms = saved;
        // Terms this wording doesn't say: a gap, never silently dropped.
        if let Some(what) = unworded_effect_terms(terms) {
            s.push_str(&self.gap(format!("permission term: {what}")));
        }
        if let Some(u) = &any_turn {
            s = format!("during any turn {u}, {s}");
        }
        if terms.flash {
            s.push_str(" as though {alt:it|they} had flash");
        }
        if let Some(c) = &terms.alt_cost {
            let pay = match c.parts.as_slice() {
                [CostPart::PayLife(Value::ManaValueOf(s))]
                    if c.mana.is_none() && matches!(s.as_ref(), Sel::This) =>
                {
                    "pay life equal to {alt:its|that spell's} mana value".to_string()
                }
                _ => self.cost_as_payment(c),
            };
            // "You may cast it by paying {1} rather than paying its mana cost" (an
            // alternative cost, CR 118.9).
            let by = super::statics::gerund_first(&pay);
            s = format!(
                "{{alt:{s}. If you cast a spell this way, {pay} rather than pay its mana cost|{s} by {by} rather than paying its mana cost}}"
            );
        }
        if let Some(c) = &terms.extra_cost {
            let pay = self.cost_as_payment(c);
            s.push_str(&format!(
                " by {} in addition to paying its other costs",
                super::statics::gerund_first(&pay)
            ));
        }
        if terms.spend_as_any_color {
            s.push_str(
                " {alt:. If you cast a spell this way, you may spend mana as though it were mana of any color to cast it|and you may spend mana as though it were mana of any color to cast that spell|. You may spend mana as though it were mana of any color to cast that spell}",
            );
        }
        if terms.spend_any_type {
            s.push_str(
                " {alt:and mana of any type can be spent to cast it|and mana of any type can be spent to cast that spell|. You may spend mana as though it were mana of any type to cast that spell|. If you cast a spell this way, you may spend mana as though it were mana of any type to cast it|. If you cast a spell this way, mana of any type can be spent to cast it}",
            );
        }
        if terms.cost_increase > 0 {
            s.push_str(&format!(
                ". {{alt:A|Each}} spell cast this way costs {{{}}} more to cast",
                terms.cost_increase
            ));
        }
        // "If you do, it enters tapped": a land played this way, and a permanent spell
        // cast this way, enters tapped (CR 614.1d, 614.12).
        let enters_tapped_on_cast = terms.on_cast.as_ref().is_some_and(|e| enters_tapped(e));
        if terms.lands_enter_tapped && enters_tapped_on_cast {
            s.push_str(". If you do, it enters tapped");
        } else if terms.lands_enter_tapped {
            s.push_str(". Each land played this way enters tapped");
        }
        // What casting a spell this way does to it: "That spell has undaunted", "If you
        // do, it has affinity for artifacts".
        if let (Some(e), false) = (&terms.on_cast, enters_tapped_on_cast) {
            match self.on_cast_words(e) {
                Some(w) => s.push_str(&w),
                None => s.push_str(&self.gap("permission term: on cast")),
            }
        }
        // "Spells you cast this way cost {2} less to cast" (CR 601.2f).
        if terms.cost_reduction > 0 {
            s.push_str(&format!(
                ". {{alt:Spells you cast this way cost|A spell cast this way costs|Each spell cast this way costs}} {{{}}} less to cast",
                terms.cost_reduction
            ));
        }
        // "You may cast Equipment spells this way without paying their mana costs" (CR
        // 118.9).
        if let Some(f) = &terms.free_option {
            let what = match f {
                Filter::Any => "{alt:cards|spells}".to_string(),
                f => {
                    let f = match f {
                        Filter::And(v) => Filter::and(
                            v.iter()
                                .filter(|x| !matches!(x, Filter::Card))
                                .cloned()
                                .collect(),
                        ),
                        other => other.clone(),
                    };
                    self.spell_noun(&f, Det::Plural)
                }
            };
            let verb = if matches!(f, Filter::Any) && !terms.spells_only {
                "play"
            } else {
                "cast"
            };
            s.push_str(&format!(
                ". You may {verb} {what} this way without paying their mana costs"
            ));
        }
        // "and you may spend colorless mana as though it were mana of any color to cast
        // that spell" (CR 609.4b).
        if let Some(t) = &terms.spend_type_as_any_color {
            let m = mana_type_words(*t);
            s.push_str(&format!(
                " {{alt:and you may spend {m} mana as though it were mana of any color to cast that spell|. You may spend {m} mana as though it were mana of any color to cast {{alt:that spell|it}}}}"
            ));
        }
        // "you may play that card if you control a Kavu".
        if let Some(c) = terms.condition.as_ref().filter(|_| any_turn.is_none()) {
            let c = self.condition(c);
            s.push_str(&format!(" {{alt:if|as long as|during any turn}} {c}"));
        }
        if terms.later_turn {
            s = format!("during your next turn, {s}");
        }
        if terms.until_another {
            s.push_str(&format!(" until you exile another card with {}", self.me()));
        }
        // "If that spell would be put into a graveyard, exile it instead."
        if terms.exile_instead {
            s.push_str(
                ". If {alt:that spell|that card|it|a spell cast this way} would be put {alt:into a graveyard|into your graveyard}, exile it instead",
            );
        }
        // "If that spell would be put into a graveyard, put it on the bottom of its
        // owner's library instead."
        if let Some(d) = &terms.graveyard_instead {
            let dest = self.destination_phrase(d, false, false);
            s.push_str(&format!(
                ". If {{alt:that spell|a spell cast this way}} would be put into a graveyard, put it {dest} instead"
            ));
        }
        s
    }

    /// What a permission is for, given the cards it's about (`w`): "an instant or
    /// sorcery spell from among them", "one of them", or the cards themselves.
    pub(crate) fn permission_object(&mut self, w: &str) -> String {
        let Some(terms) = self.play_terms.clone() else {
            return w.to_string();
        };
        let among = if w.starts_with("them") || w == "it" {
            "{alt:them|those cards|those exiled cards}".to_string()
        } else {
            w.to_string()
        };
        match (&terms.what, terms.limit) {
            (Some(f), limit) => {
                // A spell isn't a land, and it's the card that's cast (CR 305.9).
                let f = match f {
                    Filter::And(v) => Filter::and(
                        v.iter()
                            .filter(|x| {
                                !matches!(x, Filter::Card)
                                    && !matches!(x, Filter::Not(l) if matches!(l.as_ref(), Filter::Type(CardType::Land)))
                            })
                            .cloned()
                            .collect(),
                    ),
                    other => other.clone(),
                };
                let one = self.spell_noun(&f, Det::A);
                match limit {
                    Some(1) => format!("{one} from among {among}"),
                    Some(n) => {
                        let many = self.spell_noun(&f, Det::Plural);
                        format!("up to {} {many} from among {among}", number_word(n as i32))
                    }
                    None => {
                        let many = self.spell_noun(&f, Det::Plural);
                        format!("{many} from among {among}")
                    }
                }
            }
            (None, Some(1)) => format!("one of {among}"),
            (None, Some(n)) => format!("up to {} of {among}", number_word(n as i32)),
            (None, None) => w.to_string(),
        }
    }

    /// The terms recorded on permissions an effect just gave
    /// (`kw::play_permission_terms`).
    pub(crate) fn permission_terms(&mut self, name: &str) -> Option<String> {
        let (_, t) = crate::kw::play_permission_terms::parse_terms(name)?;
        let mut parts = Vec::new();
        if t.cost_increase > 0 {
            let by = if t.opponents_only {
                " by an opponent"
            } else {
                ""
            };
            parts.push(format!(
                "{{alt:a|each}} spell cast{by} this way costs {{{}}} more to cast",
                t.cost_increase
            ));
        }
        if t.lands_enter_tapped {
            parts.push("each land played this way enters tapped".into());
        }
        if parts.is_empty() {
            return None;
        }
        Some(parts.join(". "))
    }
}

/// Whether a spell's on-cast effect is "it enters tapped" (a one-use replacement for the
/// spell, `Var(IT)`).
fn enters_tapped(e: &Effect) -> bool {
    matches!(e, Effect::AddReplacement {
        def: ReplacementDef {
            event: ReplacementEvent::EntersBattlefield(Filter::In(s)),
            action: ReplacementAction::AsEnters(a),
            ..
        },
        ..
    } if matches!(s.as_ref(), Sel::Var(vars::IT))
        && matches!(a.as_ref(), Effect::EnterTapped))
}

/// "colorless", "red": a type of mana.
fn mana_type_words(t: crate::mana::ManaType) -> String {
    use crate::mana::ManaType as M;
    match t {
        M::C => "colorless".into(),
        M::W => "white".into(),
        M::U => "blue".into(),
        M::B => "black".into(),
        M::R => "red".into(),
        M::G => "green".into(),
    }
}

impl Renderer<'_> {
    /// "Until the end of your next turn, you may play those cards. If ~ was cast from a
    /// graveyard, you may play cards this way without paying their mana costs.": the same
    /// permission, free if the condition holds.
    pub(crate) fn free_if_permission(&mut self, e: &Effect) -> Option<String> {
        let Effect::If {
            cond,
            then,
            otherwise,
        } = e
        else {
            return None;
        };
        let Effect::WithPlayTerms { terms, effect } = then.as_ref() else {
            return None;
        };
        let free = terms.free_option.clone()?;
        let only_free = PlayTerms {
            free_option: Some(free.clone()),
            ..terms.clone()
        };
        let plain = PlayTerms {
            free_option: Some(free.clone()),
            ..Default::default()
        };
        if format!("{only_free:?}") != format!("{plain:?}")
            || format!("{effect:?}") != format!("{otherwise:?}")
            || !matches!(
                effect.as_ref(),
                Effect::GrantPlayPermission {
                    who: PlayerRef::You,
                    ..
                }
            )
            || !matches!(free, Filter::Any)
        {
            return None;
        }
        let o = self.effect(otherwise);
        let c = self.condition(cond);
        Some(format!(
            "{o}. If {c}, you may play cards this way without paying their mana costs"
        ))
    }

    /// What casting a spell with a permission does to it ("That spell has undaunted",
    /// "If you do, it has affinity for artifacts"), if it's a keyword it gains.
    fn on_cast_words(&mut self, e: &Effect) -> Option<String> {
        let Effect::Modify {
            what: Sel::Var(vars::IT),
            mods,
            duration: Duration::Permanent,
        } = e
        else {
            return None;
        };
        let mut kws = Vec::new();
        for m in mods {
            let Modification::AddKeyword(k) = m else {
                return None;
            };
            kws.push(self.keyword(k));
        }
        let k = join_list(&kws, "and");
        Some(format!(
            ". {{alt:That spell has {k}|If you do, it has {k}|If you cast a spell this way, it has {k}}}"
        ))
    }
}

/// The first term of an effect's permission that [`Renderer::with_play_terms`] doesn't
/// word (`what`, `limit` and `until_another` are worded with the permission's object).
fn unworded_effect_terms(t: &PlayTerms) -> Option<&'static str> {
    [
        (t.on_land.is_some(), "on land"),
        (t.any_owner, "any owner"),
        (t.cost_increase_each.is_some(), "cost increase for each"),
        (t.snow_as_any_color, "snow as any color"),
    ]
    .into_iter()
    .find_map(|(set, w)| set.then_some(w))
}

/// The first term of a static permission that [`Renderer::static_permission_terms`]
/// doesn't word.
pub(crate) fn unworded_static_terms(t: &PlayTerms) -> Option<&'static str> {
    [
        (t.on_land.is_some(), "on land"),
        (t.cost_reduction > 0, "cost reduction"),
        (t.free_option.is_some(), "free option"),
        (t.cast_face.is_some(), "cast face"),
        (t.cost_increase_each.is_some(), "cost increase for each"),
        (
            t.spend_type_as_any_color.is_some(),
            "spend a type as any color",
        ),
        (t.snow_as_any_color, "snow as any color"),
        (t.graveyard_instead.is_some(), "graveyard instead"),
        (t.any_owner, "any owner"),
    ]
    .into_iter()
    .find_map(|(set, w)| set.then_some(w))
}
