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
fn cast_only_of<'a>(e: &'a Effect, x: &Sel) -> Option<(&'a PlayTerms, &'a Effect, bool)> {
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
    let Effect::WithPlayTerms { terms, effect } = then.as_ref() else {
        return None;
    };
    let Effect::GrantPlayPermission { what, .. } = effect.as_ref() else {
        return None;
    };
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
                let g = Effect::GrantPlayPermission {
                    who: who.clone(),
                    what: sel.clone(),
                    duration: duration.clone(),
                    free: *free,
                };
                let saved = self.after_exile;
                self.after_exile |= in_exile;
                let s = self.with_play_terms(terms, &g);
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
                let s = self.with_play_terms(terms, grant);
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
        let saved = self.play_terms.replace(terms.clone());
        let mut s = self.effect(effect);
        self.play_terms = saved;
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
                _ => self.gap("an alternative cost for spells cast with a permission"),
            };
            s.push_str(&format!(
                ". If you cast a spell this way, {pay} rather than pay its mana cost"
            ));
        }
        if let Some(c) = &terms.extra_cost {
            let pay = self.cost_as_payment(c);
            s.push_str(&format!(
                " by {} in addition to paying its other costs",
                pay.replacen("pay ", "paying ", 1)
            ));
        }
        if terms.spend_as_any_color {
            s.push_str(
                " {alt:. If you cast a spell this way, you may spend mana as though it were mana of any color to cast it|and you may spend mana as though it were mana of any color to cast that spell|. You may spend mana as though it were mana of any color to cast that spell}",
            );
        }
        if terms.spend_any_type {
            s.push_str(
                " {alt:and mana of any type can be spent to cast it|and mana of any type can be spent to cast that spell|. You may spend mana as though it were mana of any type to cast that spell}",
            );
        }
        if terms.cost_increase > 0 {
            s.push_str(&format!(
                ". {{alt:A|Each}} spell cast this way costs {{{}}} more to cast",
                terms.cost_increase
            ));
        }
        if terms.lands_enter_tapped {
            s.push_str(". Each land played this way enters tapped");
        }
        // "you may play that card if you control a Kavu".
        if let Some(c) = &terms.condition {
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
                ". If {alt:that spell|that card|it} would be put into {alt:a|your} graveyard, exile it instead",
            );
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
