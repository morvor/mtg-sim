//! Static abilities ([`StaticAbility`], CR 604), restrictions, replacement effects
//! (CR 614–616), cost modifiers, and named custom behaviors.

use super::effects::{join_words, lower_first, third_person};
use super::nouns::Det;
use super::players::Case;
use super::values::split_controller;
use super::*;

/// A verb phrase with plural agreement: "gets +1/+1 and has flying" → "get +1/+1 and
/// have flying" (the comparison ignores agreement; this keeps renderings readable).
pub(crate) fn plural_vp(vp: &str) -> String {
    let fix = |w: &str| -> String {
        match w {
            "gets" => "get".into(),
            "has" => "have".into(),
            "is" => "are".into(),
            "gains" => "gain".into(),
            "loses" => "lose".into(),
            "can't" | "can" => w.into(),
            other => other.into(),
        }
    };
    let mut out = Vec::new();
    let mut first = true;
    for part in vp.split(" and ") {
        let (w, rest) = match part.split_once(' ') {
            Some((w, r)) => (w, format!(" {r}")),
            None => (part, String::new()),
        };
        if first || ["gets", "has", "is", "gains", "loses"].contains(&w) {
            out.push(format!("{}{rest}", fix(w)));
        } else {
            out.push(part.to_string());
        }
        first = false;
    }
    out.join(" and ")
}

/// A cost's instructions with the first verb as a gerund ("by sacrificing a land", "by
/// removing a counter").
pub(crate) fn gerund_first(s: &str) -> String {
    let (verb, rest) = s.split_once(' ').unwrap_or((s, ""));
    let g = match verb {
        "pay" => "paying",
        "sacrifice" => "sacrificing",
        "remove" => "removing",
        "discard" => "discarding",
        "exile" => "exiling",
        "tap" => "tapping",
        "untap" => "untapping",
        "return" => "returning",
        "put" => "putting",
        "reveal" => "revealing",
        "collect" => "collecting",
        other => return format!("{other} {rest}").trim_end().to_string(),
    };
    format!("{g} {rest}").trim_end().to_string()
}

/// `[common] and (a land or a nonland card)`: the cards a permission to "play lands and
/// cast spells from among [cards]" is for (a land is played, a nonland card cast, CR
/// 305.9): the common part.
fn lands_or_spells(f: &Filter) -> Option<Filter> {
    let Filter::And(v) = f else {
        return None;
    };
    let is_land = |x: &Filter| match x {
        Filter::Type(CardType::Land) => true,
        Filter::And(w) => w.iter().all(|y| matches!(y, Filter::Type(CardType::Land))),
        _ => false,
    };
    let is_nonland_card = |x: &Filter| match x {
        Filter::And(w) => w.iter().all(|y| {
            matches!(y, Filter::Card)
                || matches!(y, Filter::Not(l) if matches!(l.as_ref(), Filter::Type(CardType::Land)))
        }),
        _ => false,
    };
    let i = v.iter().position(|x| {
        matches!(x, Filter::Or(alts) if alts.len() == 2
            && alts.iter().any(is_land) && alts.iter().any(is_nonland_card))
    })?;
    let common: Vec<Filter> = v
        .iter()
        .enumerate()
        .filter(|(j, x)| *j != i && !matches!(x, Filter::Card))
        .map(|(_, x)| x.clone())
        .collect();
    Some(Filter::and(common))
}

fn filter_has(f: &Filter, p: &dyn Fn(&Filter) -> bool) -> bool {
    p(f) || match f {
        Filter::And(v) | Filter::Or(v) => v.iter().any(|x| filter_has(x, p)),
        Filter::Not(x) => filter_has(x, p),
        _ => false,
    }
}

/// Whether a static ability grants abilities to Equipment: the objects it affects are
/// Equipment, or it affects the permanent this one is attached to only as long as that
/// permanent is an Equipment. An ability granted to an Equipment calls the creature it's
/// attached to "equipped creature" (CR 301.5).
pub(crate) fn grants_to_equipment(s: &StaticAbility) -> bool {
    let StaticEffect::Continuous { affected, mods } = &s.effect else {
        return false;
    };
    if !mods
        .iter()
        .any(|m| matches!(m, Modification::AddAbility(_)))
    {
        return false;
    }
    let equipment = |f: &Filter| match f {
        Filter::Subtype(t) => t == "Equipment",
        Filter::And(v) => v
            .iter()
            .any(|x| matches!(x, Filter::Subtype(t) if t == "Equipment")),
        _ => false,
    };
    equipment(affected)
        || (matches!(affected, Filter::AttachedToSource)
            && matches!(&s.condition, Some(Condition::SelMatches(Sel::AttachedTo, f)) if equipment(f)))
}

impl Renderer<'_> {
    pub(crate) fn static_ability(&mut self, s: &StaticAbility) -> String {
        // CR 702.178a: "Max speed — [ability]" means "As long as your speed is 4, this
        // object has [ability]": a static ability that applies as long as you have max
        // speed. Cards may also say the condition.
        if matches!(s.condition, Some(Condition::MaxSpeed)) {
            let mut inner = s.clone();
            inner.condition = None;
            let (i, c) = self.two_ways(
                |r| r.static_ability(&inner),
                |r| r.static_ability_conditioned(s),
            );
            return format!("{{alt:Max speed — {i}|{c}}}");
        }
        self.static_ability_conditioned(s)
    }

    fn static_ability_conditioned(&mut self, s: &StaticAbility) -> String {
        // "Cast this spell only during combat": no player can cast it unless (CR 601.3).
        if let (
            Some(Condition::Not(c)),
            StaticEffect::Restriction(Restriction::CantCast {
                who: PlayerFilter::Any,
                what: Filter::Source,
            }),
        ) = (&s.condition, &s.effect)
        {
            let m = self.me();
            let when = match c.as_ref() {
                Condition::YourTurn => "during your turn".to_string(),
                Condition::Phase(PhaseCond::Combat) => "during combat".to_string(),
                other => format!("if {}", self.condition(other)),
            };
            return format!("Cast {m} only {when}.");
        }
        // CR 716.2a: the abilities printed with a class level bar: "As long as this Class
        // is level N or greater, it has [abilities]" (the bar is the activated ability).
        if let (
            Some(Condition::Compare(Value::ClassLevel, Cmp::Ge, Value::Const(_))),
            StaticEffect::Continuous {
                affected: Filter::Source,
                mods,
            },
        ) = (&s.condition, &s.effect)
        {
            if !mods.is_empty()
                && mods
                    .iter()
                    .all(|m| matches!(m, Modification::AddAbility(_)))
            {
                let lines: Vec<String> = mods
                    .iter()
                    .filter_map(|m| match m {
                        Modification::AddAbility(a) => Some(self.nested_ability(a)),
                        _ => None,
                    })
                    .collect();
                return lines.join("\n");
            }
        }
        if let Some(t) = self.level_symbol(s).or_else(|| self.station_symbol(s)) {
            return t;
        }
        // "Solved — [ability]" (CR 719.3b), compiled as the ability granted while solved.
        if s.condition.as_ref().is_some_and(super::is_solved) {
            if let StaticEffect::Continuous {
                affected: Filter::Source,
                mods,
            } = &s.effect
            {
                if let [Modification::AddAbility(a)] = mods.as_slice() {
                    let inner = self.nested_ability(a);
                    return format!("Solved — {inner}");
                }
            }
            let mut s2 = s.clone();
            s2.condition = None;
            let inner = self.static_ability(&s2);
            return format!("Solved — {inner}");
        }
        let saved = self.self_salient;
        self.self_salient = false;
        if let StaticEffect::Continuous {
            affected: Filter::Source,
            ..
        } = &s.effect
        {
            self.subject_types = self.info.card_types.iter().collect();
        }
        let e = match &s.condition {
            // "Once during each of your turns, you may cast a Zombie creature spell from
            // your graveyard" (`once_each_turn.rs`).
            Some(Condition::And(v)) if self.once_each_turn_permission(v, &s.effect).is_some() => {
                self.once_each_turn_permission(v, &s.effect)
                    .unwrap_or_default()
            }
            Some(c @ Condition::Custom(_))
                if self
                    .once_each_turn_permission(std::slice::from_ref(c), &s.effect)
                    .is_some() =>
            {
                self.once_each_turn_permission(std::slice::from_ref(c), &s.effect)
                    .unwrap_or_default()
            }
            // A once-each-turn permission or alternative cost ("Once during each of your
            // turns, you may cast a creature spell from your graveyard", see
            // `kw/once_each_turn_cast.rs`).
            Some(c) if crate::kw::once_each_turn_cast::condition_slot(c).is_some() => {
                let yours = matches!(c, Condition::And(v) if v.iter().any(|x| matches!(x, Condition::YourTurn)));
                let e = self.static_effect(&s.effect);
                let when = if yours {
                    "once during each of your turns"
                } else {
                    "once each turn"
                };
                format!("{when}, {}", lower_first(&e))
            }
            // "You may pay {0} rather than pay the power-up cost of the first power-up
            // ability you activate during each of your turns."
            Some(Condition::YourTurn)
                if matches!(&s.effect, StaticEffect::CostModifier(cm)
                    if super::tail_parts::first_ability_alt_cost(self, cm, true).is_some()) =>
            {
                let StaticEffect::CostModifier(cm) = &s.effect else {
                    return self.gap("cost modifier");
                };
                super::tail_parts::first_ability_alt_cost(self, cm, true).unwrap_or_default()
            }
            // Cost modifiers state their condition with "if" ("This spell costs {2} less
            // to cast if ...").
            Some(c) if matches!(s.effect, StaticEffect::CostModifier(_)) => {
                let e = self.static_effect(&s.effect);
                let c = self.condition(c);
                format!("{} if {c}", e.trim_end_matches('.'))
            }
            Some(Condition::YourTurn) => {
                let e = self.static_effect(&s.effect);
                format!("during your turn, {}", lower_first(&e))
            }
            Some(Condition::NotYourTurn) => {
                let e = self.static_effect(&s.effect);
                format!("during turns other than yours, {}", lower_first(&e))
            }
            // "Players can't cast spells during combat."
            Some(Condition::Phase(PhaseCond::Combat)) => {
                let e = self.static_effect(&s.effect);
                format!("{} during combat", e.trim_end_matches('.'))
            }
            // "~ can't attack or block unless an opponent has eight or more cards in their
            // graveyard."
            Some(Condition::Not(c)) if matches!(s.effect, StaticEffect::Restriction(_)) => {
                let e = self.static_effect(&s.effect);
                let c = self.condition(c);
                format!("{} unless {c}", e.trim_end_matches('.'))
            }
            Some(c) => {
                // "As long as ~ is enchanted, it has ..." / "~ has ... as long as it's
                // enchanted": the condition comes first or last, so the object itself is
                // named by "~" or "it" in either place.
                self.self_salient = true;
                let mut c = self.condition(c);
                // "As long as ~ is in your graveyard and you control a Forest": an ability
                // that functions only from the graveyard (CR 113.6).
                if s.zone == FunctionZone::Graveyard
                    && matches!(&s.effect, StaticEffect::Continuous { affected, .. }
                        if !matches!(affected, Filter::Source))
                    && !c.contains("graveyard")
                {
                    c = format!("{} is in your graveyard and {c}", self.me());
                }
                let e = self.static_effect(&s.effect);
                format!("as long as {c}, {}", lower_first(&e))
            }
            // "As long as ~ isn't on the battlefield, it's a 1/1 Insect creature in
            // addition to its other types." (CR 113.6c)
            None if s.zone == FunctionZone::AnywhereExcept(ZoneKind::Battlefield) => {
                // What it is elsewhere isn't known: "in addition to its other types".
                self.subject_types.clear();
                let me = self.me();
                let e = self.static_effect(&s.effect);
                let e = match e
                    .strip_prefix(&format!("{me} is "))
                    .or_else(|| e.strip_prefix("~it is "))
                {
                    Some(rest) => format!("it's {rest}"),
                    None => lower_first(&e),
                };
                format!("as long as {me} isn't on the battlefield, {e}")
            }
            None => self.static_effect(&s.effect),
        };
        self.subject_types.clear();
        self.self_salient = saved;
        let mut e = e;
        // CR 702.16n, 702.16p: protection that doesn't make some permanents fall off.
        if let StaticEffect::Continuous { mods, .. } = &s.effect {
            for m in mods {
                let Modification::AddKeyword(k) = m else {
                    continue;
                };
                if k.kind != crate::keywords::KeywordKind::Protection {
                    continue;
                }
                let what = match k.text.as_deref() {
                    Some(crate::choices::DOESNT_REMOVE_SOURCE) => self.me(),
                    Some(crate::kw::protection::DOESNT_REMOVE_AURAS) => "Auras".into(),
                    Some(crate::kw::protection::DOESNT_REMOVE_ATTACHED) => {
                        "Auras and Equipment you control that are already attached to it".into()
                    }
                    _ => continue,
                };
                e = format!(
                    "{}. This effect doesn't remove {what}",
                    e.trim_end_matches('.')
                );
                break;
            }
        }
        let e = capitalize(e.trim());
        if e.ends_with('.') || e.ends_with('"') {
            e
        } else {
            format!("{e}.")
        }
    }

    pub(crate) fn restriction_subject(&mut self, f: &Filter) -> String {
        self.affected_subject(f)
    }

    /// The subject for a static effect on objects: "~", "enchanted creature", "creatures
    /// you control".
    /// `f` without the "first ... each turn" quality (`kw::first_spell_each_turn`), when
    /// it has it.
    fn first_each_turn(f: &Filter) -> Option<Filter> {
        let first = crate::kw::first_spell_each_turn::FIRST_THIS_TURN;
        let Filter::And(v) = f else {
            return None;
        };
        if !v
            .iter()
            .any(|x| matches!(x, Filter::Custom(n) if n == first))
        {
            return None;
        }
        let rest: Vec<Filter> = v
            .iter()
            .filter(|x| !matches!(x, Filter::Custom(n) if n == first))
            .cloned()
            .collect();
        Some(if rest.len() == 1 {
            rest.into_iter().next().unwrap_or(Filter::Any)
        } else {
            Filter::And(rest)
        })
    }

    /// "instant or sorcery spell" (singular, no article).
    fn spell_noun_one(&mut self, f: &Filter) -> String {
        let n = self.noun(f, Num::One);
        let n = n.trim_end_matches(" you control").to_string();
        if n.contains("spell") {
            n
        } else if n == "permanent" || n == "card" {
            "spell".into()
        } else {
            format!("{n} spell")
        }
    }

    /// The subject of a restriction: spells are the ones on the stack ("spells you control
    /// can't be countered").
    fn restricted_subject(&mut self, f: &Filter) -> String {
        if filter_has(f, &|x| matches!(x, Filter::Spell)) && Self::first_each_turn(f).is_none() {
            return self.noun_det(f, Det::Plural);
        }
        self.affected_subject(f)
    }

    fn affected_subject(&mut self, f: &Filter) -> String {
        if let Some(base) = Self::first_each_turn(f) {
            // "The first historic spell you cast each turn has convoke."
            let n = self.spell_noun_one(&base);
            return format!("the first {n} you cast each turn");
        }
        match f {
            Filter::Source => self.me(),
            Filter::AttachedToSource => self.attached_noun(),
            Filter::In(sel) => self.sel(sel, Case::Subj),
            other => {
                let s = self.noun_det(other, Det::Plural);
                // Abilities granted to spells are gained as they're cast (CR 610.5, see
                // `next_spell::is_cast_grant`): "spells you cast have convoke".
                if filter_has(other, &|x| matches!(x, Filter::Spell)) {
                    s.replace("spells you control", "spells you cast")
                        .replace("spell you control", "spell you cast")
                        .replace("spells} you control", "spells} you cast")
                } else {
                    s
                }
            }
        }
    }

    pub(crate) fn static_effect(&mut self, e: &StaticEffect) -> String {
        match e {
            StaticEffect::Continuous { affected, mods } => {
                if mods.len() == 1 {
                    if let Modification::CdaPT(p, t) = &mods[0] {
                        return self.cda_pt(affected, p, t);
                    }
                    // "You control enchanted creature."
                    if let Modification::SetController(p) = &mods[0] {
                        let obj = self.affected_subject(affected);
                        let w = self.player(p, Case::Subj);
                        let verb = if w == "you" { "control" } else { "controls" };
                        return format!("{w} {verb} {obj}");
                    }
                }
                let subj = self.affected_subject(affected);
                let vp = self.mods_vp(mods, false);
                let plural = !matches!(
                    affected,
                    Filter::Source | Filter::AttachedToSource | Filter::In(_)
                );
                if plural {
                    format!("{subj} {}", plural_vp(&vp))
                } else {
                    format!("{subj} {vp}")
                }
            }
            StaticEffect::PlayerEffect { affected, effect } => {
                let who = self.player_filter_as_ref(affected);
                self.player_modification(&who, effect)
            }
            StaticEffect::Restriction(r) => self.restriction(r),
            StaticEffect::CostModifier(cm) => self.cost_modifier(cm),
            StaticEffect::Replacement(def) => self.replacement(def, None),
            StaticEffect::PlayPermission(pp) => self.play_permission(pp),
            StaticEffect::ActivationPermission(ap) => self.activation_permission("you", ap),
            StaticEffect::FlashPermission { who, what } => {
                // "Any player may cast Sliver spells as though they had flash."
                let w = if matches!(who, PlayerRel::Any) {
                    "any player".to_string()
                } else {
                    self.rel_subject(*who)
                };
                let s = self.spell_noun_plural(what);
                format!("{w} may cast {s} as though they had flash")
            }
            StaticEffect::AdditionalTrigger { sources, cause } => {
                let s = self.noun_det(sources, Det::A);
                match cause {
                    None => format!(
                        "if a triggered ability of {s} triggers, that ability triggers an additional time"
                    ),
                    Some(c) => {
                        let t = self.trigger_text(c);
                        let t = t.trim_start_matches("whenever ").to_string();
                        // "If an artifact or creature entering causes ...": the event as a
                        // gerund.
                        let t = if let Some(x) = t.strip_suffix(" enters") {
                            format!("{x} entering {{opt:the battlefield}}")
                        } else if let Some(x) = t.strip_suffix(" dies") {
                            format!("{x} dying")
                        } else if let Some(x) = t.strip_suffix(" attacks") {
                            format!("{x} attacking")
                        } else if let Some((x, y)) = t.split_once(" deals ") {
                            format!("{x} dealing {y}")
                        } else if let Some((x, y)) = t.split_once(" draws ") {
                            format!("{x} drawing {y}")
                        } else {
                            t
                        };
                        format!(
                            "if {t} causes a triggered ability of {s} to trigger, that ability triggers an additional time"
                        )
                    }
                }
            }
            StaticEffect::SpendAsAnyColor { applies_to, types } => {
                let what = self.cost_target(applies_to);
                let t = if types.is_empty() {
                    "mana".to_string()
                } else {
                    let w: Vec<String> = types.iter().map(|t| mana_symbol(*t)).collect();
                    format!("{} mana", join_list(&w, "or"))
                };
                format!("you may spend {t} as though it were mana of any color to pay {what}")
            }
            StaticEffect::OpeningHand { delayed } => match delayed {
                None => "if ~ is in your opening hand, you may begin the game with it on the battlefield".into(),
                Some(d) => {
                    let (t, b) = d.as_ref();
                    // CR 103.? leylines and chancellors: the first upkeep of the game.
                    let t = match t {
                        TriggerCond::BeginningOf {
                            step: TriggerStep::Upkeep,
                            whose: PlayerRel::Any,
                        } => "at the beginning of the first upkeep".to_string(),
                        t => self.trigger_text(t),
                    };
                    let b = self.body(b);
                    format!(
                        "you may reveal ~ from your opening hand. If you do, {t}, {}",
                        lower_first(&b)
                    )
                }
            },
            StaticEffect::PregameChoice {
                kind,
                only_if_commander,
            } => {
                let c = self.choice(kind);
                if *only_if_commander {
                    format!("if ~ is your commander, choose {c} before the game begins")
                } else {
                    format!("choose {c} before the game begins")
                }
            }
            StaticEffect::BeforeShuffleExile { what } => {
                let n = self.noun_det(what, Det::A);
                format!(
                    "before you shuffle your deck to start the game, you may reveal ~ from your deck and exile {n} you drafted that isn't in your deck"
                )
            }
            StaticEffect::Companion(dc) => {
                let c = self.deck_condition(dc);
                format!("companion — {c}")
            }
            StaticEffect::AnyTimeCouldMulligan(e) => {
                let e = self.effect(e);
                format!("any time you could mulligan and ~ is in your hand, you may {e}")
            }
            StaticEffect::DelayedTriggerAsEnters {
                condition,
                trigger,
                body,
            } => {
                let t = self.trigger_text(trigger);
                let b = self.in_event_scope(|r| r.body(body));
                match condition {
                    Some(c) => {
                        let c = self.condition(c);
                        format!("if {c}, {t}, {}", lower_first(&b))
                    }
                    None => format!("{t}, {}", lower_first(&b)),
                }
            }
            StaticEffect::SpecialAction(def) => {
                let who = self.player_filter_object(&def.who);
                let c = self.cost_as_payment(&def.cost);
                let what = match &def.action {
                    SpecialActionEffect::Effect(e) => self.effect(e),
                    SpecialActionEffect::IgnoreSourceEffects => {
                        "that player ignores this effect until end of turn".into()
                    }
                };
                format!("{who} may {c} any time they could cast an instant. If they do, {what}")
            }
            StaticEffect::LookAtTopCard(r) => {
                let w = self.rel_subject(*r);
                let p = if w == "you" { "your" } else { "their" };
                format!("{w} may look at the top card of {p} library any time")
            }
            StaticEffect::RevealTopCard(r) => {
                let p = self.rel_possessive(*r, Num::One);
                format!("play with the top card of {p} library revealed")
            }
            StaticEffect::AdditionalLandPlays(r, n) => {
                let w = self.rel_subject(*r);
                let p = if w == "you" { "your" } else { "their" };
                match n {
                    1 => format!("{w} may play an additional land on each of {p} turns"),
                    n => format!(
                        "{w} may play {} additional lands on each of {p} turns",
                        number_word(*n as i32)
                    ),
                }
            }
            StaticEffect::Dice(d) => self.dice_static(d),
            StaticEffect::AttachOnlyTo(f) => {
                let n = self.noun_det(f, super::nouns::Det::A);
                format!("~ can be attached only to {n}")
            }
            // "You can't cast ~ during your first, second, or third turns of the game."
            StaticEffect::CastOnlyIf(Condition::Not(inner))
                if crate::rule_statics::turns_taken::early_turns_n(inner).is_some() =>
            {
                let n = crate::rule_statics::turns_taken::early_turns_n(inner).unwrap_or(1);
                let l = crate::rule_statics::turns_taken::ordinal_list(n);
                format!("you can't cast ~ during your {l} turns of the game")
            }
            StaticEffect::CastOnlyIf(c) => {
                let c = self.cast_only_condition(c);
                format!("cast ~ only {c}")
            }
            StaticEffect::OptionalAttackCost { cost, then } => {
                let c = self.cost_as_payment(cost);
                let mut s = format!("you may {c} as ~ attacks");
                if let Some(b) = then {
                    let b = self.body(b);
                    s.push_str(&format!(". When you do, {}", lower_first(&b)));
                }
                s
            }
            StaticEffect::Custom(name) => self.custom_static(name),
            StaticEffect::CastGrant { zone, what, mods } => {
                let w = self.noun(what, Num::One);
                let vp = self.mods_vp(mods, true);
                format!(
                    "if you cast {} from your {} this way, it {vp}",
                    with_article(&w),
                    zone_word(*zone)
                )
            }
            StaticEffect::LegendRuleExempt(f) => {
                let n = self.noun(f, Num::Many);
                format!("the \"legend rule\" doesn't apply to {n}")
            }
            StaticEffect::DamageNotRemoved(f) => {
                let n = self.affected_subject(f);
                format!("damage isn't removed from {n} during cleanup steps")
            }
            StaticEffect::CountersRemain => {
                let me = self.me();
                format!(
                    "counters remain on {me} as it moves to any zone other than a player's hand or library"
                )
            }
        }
    }

    fn cast_only_condition(&mut self, c: &Condition) -> String {
        match c {
            Condition::CombatTiming(ct) => self.combat_timing(ct),
            Condition::YourTurn => "during your turn".into(),
            Condition::NotYourTurn => "during an opponent's turn".into(),
            Condition::Phase(PhaseCond::Combat) => "during combat".into(),
            Condition::Phase(PhaseCond::Upkeep) => "during any upkeep step".into(),
            Condition::And(v)
                if matches!(
                    v.as_slice(),
                    [Condition::NotYourTurn, Condition::Phase(PhaseCond::Upkeep)]
                ) =>
            {
                "during an opponent's upkeep".into()
            }
            Condition::And(v)
                if matches!(
                    v.as_slice(),
                    [Condition::YourTurn, Condition::Phase(PhaseCond::Upkeep)]
                ) =>
            {
                "during your upkeep".into()
            }
            Condition::Phase(PhaseCond::DeclareAttackers) => {
                "during the declare attackers step".into()
            }
            Condition::Phase(PhaseCond::EndStep) => "during the end step".into(),
            Condition::Custom(n) if n == "restrictions:declare_blockers_step" => {
                "during the declare blockers step".into()
            }
            // "Cast ~ only during the declare attackers step and only if you've been
            // attacked this step."
            Condition::And(v) if v.len() == 2 => {
                let a = self.cast_only_condition(&v[0]);
                let b = self.cast_only_condition(&v[1]);
                format!("{a} and only {b}")
            }
            other => {
                let c = self.condition(other);
                format!("if {c}")
            }
        }
    }

    fn cda_pt(&mut self, affected: &Filter, p: &Option<Value>, t: &Option<Value>) -> String {
        let subj = self.affected_subject(affected);
        let subj = nouns::possessive(&subj);
        match (p, t) {
            (Some(a), Some(b)) if format!("{a:?}") == format!("{b:?}") => {
                let v = self.value(a);
                format!("{subj} power and toughness are each equal to {v}")
            }
            // "its toughness is equal to that number plus 1".
            (Some(a), Some(Value::Sum(v)))
                if v.len() == 2 && format!("{:?}", v[0]) == format!("{a:?}") =>
            {
                let a = self.value(a);
                let k = self.value(&v[1]);
                format!("{subj} power is equal to {a} and its toughness is equal to that number plus {k}")
            }
            (Some(a), Some(b)) => {
                let a = self.value(a);
                let b = self.value(b);
                format!("{subj} power is equal to {a} and its toughness is equal to {b}")
            }
            (Some(a), None) => {
                let a = self.value(a);
                format!("{subj} power is equal to {a}")
            }
            (None, Some(b)) => {
                let b = self.value(b);
                format!("{subj} toughness is equal to {b}")
            }
            (None, None) => self.gap("CdaPT without values"),
        }
    }

    fn player_filter_as_ref(&mut self, pf: &PlayerFilter) -> PlayerRef {
        match pf {
            PlayerFilter::You | PlayerFilter::Controller => PlayerRef::You,
            PlayerFilter::Opponent => PlayerRef::Each(PlayerFilter::Opponent),
            PlayerFilter::Any => PlayerRef::EachPlayer,
            PlayerFilter::Ref(r) => (**r).clone(),
            other => PlayerRef::Each(other.clone()),
        }
    }

    /// Effects on players: "you have hexproof", "your maximum hand size is reduced by
    /// two".
    pub(crate) fn player_modification(
        &mut self,
        who: &PlayerRef,
        m: &PlayerModification,
    ) -> String {
        let subj = match who {
            PlayerRef::Each(PlayerFilter::Opponent) | PlayerRef::EachOpponent => {
                "{alt:your opponents|each opponent}".to_string()
            }
            PlayerRef::EachPlayer => "each player".into(),
            other => self.player(other, Case::Subj),
        };
        let poss = match subj.as_str() {
            "you" => "your".to_string(),
            "your opponents" => "your opponents'".into(),
            "{alt:your opponents|each opponent}" => "{alt:your opponents'|each opponent's}".into(),
            "each player" => "each player's".into(),
            s => nouns::possessive(s),
        };
        match m {
            PlayerModification::Hexproof => format!("{subj} have hexproof"),
            PlayerModification::ActivationPermission(ap) => {
                let who = subj.clone();
                self.activation_permission(&who, ap)
            }
            PlayerModification::Shroud => format!("{subj} have shroud"),
            PlayerModification::ProtectionFrom(f) => {
                let q = self.quality(f);
                format!("{subj} have protection from {q}")
            }
            PlayerModification::MaxHandSize(None) => {
                format!("{subj} have no maximum hand size")
            }
            PlayerModification::MaxHandSize(Some(v)) => {
                let v = self.value(v);
                format!("{poss} maximum hand size is {v}")
            }
            PlayerModification::HandSizeDelta(n) if *n > 0 => {
                format!(
                    "{poss} maximum hand size is increased by {}",
                    number_word(*n)
                )
            }
            PlayerModification::HandSizeDelta(n) => {
                format!(
                    "{poss} maximum hand size is reduced by {}",
                    number_word(-*n)
                )
            }
            PlayerModification::AdditionalLandPlays(n) => {
                let p = if subj == "you" { "your" } else { "their" };
                match n {
                    1 => format!("{subj} may play an additional land on each of {p} turns"),
                    n => format!(
                        "{subj} may play {} additional lands on each of {p} turns",
                        number_word(*n as i32)
                    ),
                }
            }
            PlayerModification::CantLoseGame => format!("{subj} can't lose the game"),
            PlayerModification::CostModifier(cm) => {
                let s = self.cost_modifier(cm);
                s.replacen("you cast", &format!("{subj} cast"), 1)
            }
            PlayerModification::FlashPermission(f) => {
                let s = self.spell_noun_plural(f);
                format!("{subj} may cast {s} as though they had flash")
            }
            PlayerModification::PlayPermission(pp) => {
                let s = self.play_permission(pp);
                if subj != "you" {
                    s.replacen("you may", &format!("{subj} may"), 1)
                } else {
                    s
                }
            }
            PlayerModification::PayLifeForMana { color, life } => {
                let sym = mana_symbol(crate::mana::ManaType::from_color(*color));
                format!(
                    "for each {sym} in a cost, {subj} may pay {} life rather than pay that mana",
                    number_word(*life as i32)
                )
            }
            PlayerModification::Custom(name) => self.custom_player_mod(name, &subj, &poss),
        }
    }

    fn spell_noun_plural(&mut self, f: &Filter) -> String {
        if matches!(f, Filter::Source) {
            return self.me();
        }
        // "Red spells and white spells you cast cost {1} less".
        let saved = (self.alt_and, self.plural_alts, self.default_head);
        (self.alt_and, self.plural_alts) = (true, true);
        // "noncreature spells": what's cast is a spell.
        self.default_head = Some("spell");
        let n = self.noun(f, Num::Many);
        (self.alt_and, self.plural_alts, self.default_head) = saved;
        if n.contains("spell") {
            n
        } else if n == "permanents" || n == "cards" {
            "spells".into()
        } else if let Some(r) = n
            .strip_prefix("permanents ")
            .filter(|_| !format!("{f:?}").contains("Permanent"))
        {
            // "spells from anywhere other than your hand" (no type named).
            format!("spells {r}")
        } else {
            // "sorcery spells" (not "sorcerie spells").
            let one = super::singular(&n);
            format!("{one} spells")
        }
    }

    fn play_permission(&mut self, pp: &PlayPermission) -> String {
        let who = self.rel_subject(pp.who);
        let p = if who == "you" { "your" } else { "their" };
        // A spell isn't a land, and it's the card that's cast (CR 305.9): "nonland" and
        // "card" in what may be cast say nothing more.
        let spell_what = |f: &Filter| {
            match f {
            Filter::And(v) => Filter::and(
                v.iter()
                    .filter(|x| {
                        !matches!(x, Filter::Not(l) if matches!(l.as_ref(), Filter::Type(CardType::Land)))
                    })
                    .map(|x| match x {
                        Filter::Card => Filter::Spell,
                        x => x.clone(),
                    })
                    .collect(),
            ),
            Filter::Not(l) if matches!(l.as_ref(), Filter::Type(CardType::Land)) => Filter::Any,
            other => other.clone(),
        }
        };
        let verb = match (pp.lands, pp.spells) {
            (true, true) => {
                if matches!(pp.what, Filter::Any) {
                    "play lands and cast spells".to_string()
                } else if let Some(common) = lands_or_spells(&pp.what) {
                    // "play lands and cast spells from among cards you own with croak
                    // counters on them": cards of the group, as lands or as spells.
                    let n = self.noun(&Filter::and(vec![Filter::Card, common]), Num::Many);
                    format!("play lands and cast spells from among {n}")
                } else {
                    let n = self.noun(&pp.what, Num::Many);
                    format!("play {n}")
                }
            }
            (true, false) => {
                if matches!(pp.what, Filter::Type(CardType::Land)) {
                    "play lands".to_string()
                } else {
                    let n = self.noun(&pp.what, Num::Many);
                    format!("play {n}")
                }
            }
            (false, _) => {
                let w = spell_what(&pp.what);
                let s = if matches!(w, Filter::Any | Filter::Spell) {
                    "spells".to_string()
                } else {
                    self.spell_noun_plural(&w)
                };
                format!("cast {s}")
            }
        };
        let zone = match pp.zone {
            ZoneKind::Library if pp.top_only => format!("from the top of {p} library"),
            ZoneKind::Exile => "from exile".into(),
            z => format!("from {p} {}", zone_word(z)),
        };
        let mut s = format!("{who} may {verb} {zone}");
        if let Some(c) = &pp.cost {
            if c.is_free() {
                s.push_str(" without paying their mana costs");
            } else {
                let c = self.cost_as_payment(c);
                s.push_str(&format!(" by {}", c.replacen("pay", "paying", 1)));
            }
        }
        let tail = self.static_permission_terms(pp);
        s.push_str(&tail);
        s
    }

    /// The terms a static ability's permission to play cards comes with: "If you cast a
    /// spell this way, you may cast it as though it had flash", "by removing a counter
    /// from a creature you control in addition to paying their other costs", "If a spell
    /// cast this way would be put into your graveyard, exile it instead" (see
    /// [`PlayTerms`]).
    pub(crate) fn static_permission_terms(&mut self, pp: &PlayPermission) -> String {
        let t = &pp.terms;
        let mut s = String::new();
        if let Some(c) = &t.extra_cost {
            let pay = self.cost_as_payment(c);
            s.push_str(&format!(
                " by {} in addition to paying their other costs",
                gerund_first(&pay)
            ));
        }
        if pp.flash || t.flash {
            s.push_str(
                " {alt:. If you cast a spell this way, you may cast it as though it had flash|as though they had flash|as though it had flash}",
            );
        }
        if t.alt_cost.is_some() {
            s.push_str(&self.gap("an alternative cost for spells cast with a static permission"));
        }
        if t.spend_as_any_color {
            s.push_str(
                " {alt:. If you cast a spell this way, you may spend mana as though it were mana of any color to cast it|and you may spend mana as though it were mana of any color to cast those spells}",
            );
        }
        if t.spend_any_type {
            s.push_str(" {alt:and mana of any type can be spent to cast those spells|. Mana of any type can be spent to cast those spells}");
        }
        if t.cost_increase > 0 {
            s.push_str(&format!(
                ". {{alt:A|Each}} spell cast this way costs {{{}}} more to cast",
                t.cost_increase
            ));
        }
        if t.lands_enter_tapped {
            s.push_str(". Each land played this way enters tapped");
        }
        if let Some(c) = &t.condition {
            let c = self.condition(c);
            s.push_str(&format!(" {{alt:if|as long as}} {c}"));
        }
        if t.what.is_some() || t.limit.is_some() || t.until_another || t.later_turn {
            s.push_str(&self.gap("terms of a static permission to play cards"));
        }
        if t.exile_instead {
            s.push_str(
                ". If a spell cast this way would be put into its owner's graveyard, exile it instead",
            );
        }
        s
    }

    /// What a cost modifier applies to ("spells you cast", "activated abilities of
    /// creatures").
    fn cost_target(&mut self, t: &CostTarget) -> String {
        match t {
            // A card cast from a zone is judged while it's still there or by the zone it
            // was cast from (CR 601.2a): "spells you cast from your graveyard".
            CostTarget::Spells(f) => self.spell_noun_plural(f),
            CostTarget::Abilities(f) => {
                if matches!(f, Filter::Source) {
                    "activated abilities of ~".into()
                } else {
                    let n = self.noun(f, Num::Many);
                    format!("activated abilities of {n}")
                }
            }
            CostTarget::ThisSpell => self.me(),
            CostTarget::Keyword(k) => format!("{} abilities", k.name().to_lowercase()),
            CostTarget::KeywordAbilitiesOf(k, f) => {
                let n = self.noun_det(f, Det::Plural);
                format!("{} abilities of {n}", k.name().to_lowercase())
            }
            CostTarget::LoyaltyAbilities(f) => {
                let n = self.noun_det(f, Det::Plural);
                format!("loyalty abilities of {n}")
            }
            CostTarget::ActivatedAbilities(scope) => self.ability_scope(scope),
        }
    }

    /// Activated abilities by kind, source, and targets: "the first equip ability you
    /// activate each turn", "abilities of creatures you control that target a Merfolk".
    pub(crate) fn ability_scope(&mut self, scope: &AbilityScope) -> String {
        let kind = match scope.class {
            AbilityClass::Any if scope.nonmana => {
                "activated abilities that aren't mana abilities".to_string()
            }
            AbilityClass::Any => "activated abilities".to_string(),
            AbilityClass::Loyalty => "loyalty abilities".to_string(),
            AbilityClass::Mana => "mana abilities".to_string(),
            AbilityClass::Keyword(k) => format!("{} abilities", k.name().to_lowercase()),
        };
        let mut s = if scope.first_each_turn {
            format!("the first {}", kind.replacen("abilities", "ability", 1))
        } else {
            kind
        };
        if !matches!(scope.sources, Filter::Any) {
            let n = if matches!(scope.sources, Filter::Source) {
                self.me()
            } else {
                self.noun_det(&scope.sources, Det::Plural)
            };
            s = format!("{s} of {n}");
        }
        if let Some(t) = &scope.targeting {
            let n = self.noun_det(t, Det::A);
            s = format!("{s} that target {n}");
        }
        if scope.first_each_turn {
            s.push_str(" you activate each turn");
        }
        s
    }

    /// "You may activate loyalty abilities of planeswalkers you control any time you could
    /// cast an instant", "... twice each turn rather than only once", "... as though those
    /// creatures had haste" (CR 602.5d, 606.3, 302.6).
    pub(crate) fn activation_permission(&mut self, who: &str, ap: &ActivationPermission) -> String {
        let what = self.ability_scope(&ap.scope);
        let mut parts = Vec::new();
        if ap.instant_timing {
            parts.push("any time you could cast an instant".to_string());
        }
        if let Some(n) = ap.loyalty_per_turn {
            let t = match n {
                1 => "once".to_string(),
                2 => "twice".to_string(),
                n => format!("{} times", number_word(n as i32)),
            };
            parts.push(format!("{t} each turn rather than only once"));
        }
        if ap.as_though_haste {
            parts.push("as though those creatures had haste".to_string());
        }
        let tail = if parts.is_empty() {
            String::new()
        } else {
            format!(" {}", join_list(&parts, "and"))
        };
        format!("{who} may activate {what}{tail}")
    }

    pub(crate) fn cost_modifier(&mut self, cm: &CostModifier) -> String {
        // "The first creature spell you cast each turn costs {2} less to cast."
        let first = match &cm.applies_to {
            CostTarget::Spells(f) => Self::first_each_turn(f),
            _ => None,
        };
        if let Some(base) = first {
            let n = self.spell_noun_one(&base);
            let mut cm2 = cm.clone();
            cm2.applies_to = CostTarget::Spells(base);
            let s = self.cost_modifier(&cm2);
            let who = if cm.who == PlayerRel::You {
                "you".to_string()
            } else {
                self.rel_subject(cm.who)
            };
            if let Some(i) = s.find(&format!(" {who} cast cost")) {
                let rest = &s[i + format!(" {who} cast").len()..];
                return format!("the first {n} {who} cast each turn{rest}");
            }
            return s;
        }
        // A card cast from a zone is judged while it's still there or by the zone it was
        // cast from (CR 601.2a): "spells you cast from your graveyard".
        let target = match &cm.applies_to {
            CostTarget::Spells(f) => self.cost_target(&CostTarget::Spells(cast_from_zone_only(f))),
            other => self.cost_target(other),
        };
        let is_spell = matches!(cm.applies_to, CostTarget::Spells(_) | CostTarget::ThisSpell);
        let who = match (&cm.applies_to, cm.who) {
            (CostTarget::ThisSpell, _) => String::new(),
            (_, PlayerRel::Any) => String::new(),
            (CostTarget::Spells(_), r) => {
                let w = self.rel_subject(r);
                let w = if w == "an opponent" {
                    "your opponents".to_string()
                } else {
                    w
                };
                format!(" {w} cast")
            }
            (_, PlayerRel::You) => " you activate".into(),
            (_, r) => {
                let w = self.rel_subject(r);
                format!(" {w} activate")
            }
        };
        // "Spells your opponents cast that target ~": the caster before a relative
        // clause.
        // "Spells you cast from your graveyard": the caster before where they're cast
        // from.
        let from = [
            " from your ",
            " from anywhere ",
            " cast from ",
            " {opt:cast} from ",
        ]
        .iter()
        .find_map(|p| target.find(p))
        .filter(|_| {
            matches!(
                cm.change,
                CostChange::ReduceGeneric(_) | CostChange::IncreaseGeneric(_)
            )
        });
        let (target, who) = match target.find(" that ").or(from) {
            Some(i) if !who.is_empty() && !target[..i].contains('{') => (
                format!("{}{who}{}", &target[..i], &target[i..]),
                String::new(),
            ),
            _ => (target, who),
        };
        let act = if is_spell { "to cast" } else { "to activate" };
        let costs = if matches!(cm.applies_to, CostTarget::ThisSpell) {
            "costs"
        } else {
            "cost"
        };
        match &cm.change {
            CostChange::IncreaseGeneric(v) => {
                let (amt, tail) = self.cost_amount(v);
                format!("{target}{who} {costs} {amt} more {act}{tail}")
            }
            CostChange::ReduceGeneric(v) => {
                let (amt, tail) = self.cost_amount(v);
                format!("{target}{who} {costs} {amt} less {act}{tail}")
            }
            CostChange::ReduceGenericMinOne(v) => {
                let (amt, tail) = self.cost_amount(v);
                format!(
                    "{target}{who} {costs} {amt} less {act}{tail}. This effect can't reduce the mana in that cost to less than one mana"
                )
            }
            CostChange::SpendAnyType => {
                let t = target.clone();
                format!("you can spend mana of any type to cast {t}")
            }
            CostChange::Rule(r) => cost_rule_text(r),
            CostChange::IncreaseMana(m) => format!("{target}{who} {costs} {m} more {act}"),
            CostChange::ReduceColored(c, v) => {
                let sym = mana_symbol(crate::mana::ManaType::from_color(*c));
                let s = match v {
                    Value::Const(n) if *n > 0 => sym.repeat(*n as usize),
                    other => {
                        let v = self.value(other);
                        format!("{sym} for each {v}")
                    }
                };
                format!("{target}{who} {costs} {s} less {act}")
            }
            CostChange::ReduceMana { mana, colored_only } => {
                // "As an additional cost to cast green permanent spells, you may pay 2 life.
                // Those spells cost {G} less to cast if you paid life this way." (see
                // `kw/offered_costs.rs`).
                let paid_life = match &cm.applies_to {
                    CostTarget::Spells(Filter::And(v)) => v.iter().any(|f| {
                        matches!(f, Filter::Custom(n)
                            if n.strip_prefix(crate::kw::offered_costs::PAID_OFFERED_COST)
                                .is_some_and(|c| c.starts_with("pay ") && c.ends_with(" life")))
                    }),
                    _ => false,
                };
                let mut s = if paid_life && cm.who == PlayerRel::You {
                    format!("those spells cost {mana} less {act} if you paid life this way")
                } else {
                    format!("{target}{who} {costs} {mana} less {act}")
                };
                if *colored_only {
                    // A reduction of mana of one color reduces only that color's mana.
                    let syms = mana.to_string();
                    let colors: Vec<&str> = syms
                        .split('}')
                        .filter_map(|x| x.strip_prefix('{'))
                        .collect();
                    let one = match colors.first() {
                        Some(c) if colors.iter().all(|x| x == c) => match *c {
                            "W" => Some("white"),
                            "U" => Some("blue"),
                            "B" => Some("black"),
                            "R" => Some("red"),
                            "G" => Some("green"),
                            _ => None,
                        },
                        _ => None,
                    };
                    let which = match one {
                        Some(c) => format!("{{alt:colored|{c}}}"),
                        None => "colored".into(),
                    };
                    s.push_str(&format!(
                        ". This effect reduces only the amount of {which} mana you pay"
                    ));
                }
                s
            }
            // "This spell costs {R} more to cast for each target beyond the first": paid
            // with the spell's total cost like an additional cost (CR 601.2f).
            CostChange::AdditionalCost(Cost { mana: None, parts })
                if matches!(cm.applies_to, CostTarget::ThisSpell)
                    && matches!(parts.as_slice(), [CostPart::Repeated { cost, times: Value::Custom(t) }]
                        if t == "spell_targets_beyond_first" && cost.parts.is_empty() && cost.mana.is_some()) =>
            {
                let [CostPart::Repeated { cost, .. }] = parts.as_slice() else {
                    return self.gap("cost per target");
                };
                let mana = cost
                    .mana
                    .as_ref()
                    .map(|m| m.to_string())
                    .unwrap_or_default();
                format!("~ costs {mana} more to cast for each target beyond the first")
            }
            // "Planeswalkers' loyalty abilities you activate cost an additional +1 to
            // activate" (CR 606.4: the loyalty cost changes).
            CostChange::AdditionalCost(Cost { mana: None, parts })
                if matches!(cm.applies_to, CostTarget::LoyaltyAbilities(_))
                    && matches!(parts.as_slice(), [CostPart::Loyalty(_)]) =>
            {
                let [CostPart::Loyalty(n)] = parts.as_slice() else {
                    return self.gap("loyalty cost");
                };
                let n = if *n > 0 {
                    format!("+{n}")
                } else {
                    n.to_string()
                };
                let CostTarget::LoyaltyAbilities(f) = &cm.applies_to else {
                    return self.gap("loyalty cost");
                };
                let o = self.noun(f, Num::Many);
                let who = if cm.who == PlayerRel::You {
                    " you activate"
                } else {
                    ""
                };
                format!(
                    "{} loyalty abilities{who} cost an additional {n} to activate",
                    super::nouns::possessive(&o)
                )
            }
            CostChange::AdditionalCost(c) => {
                let c = self.cost_as_payment(c);
                if matches!(cm.applies_to, CostTarget::ThisSpell) {
                    format!("as an additional cost to cast ~, {c}")
                } else {
                    format!("as an additional cost to cast {target}, {c}")
                }
            }
            // An alternative cost offered for the spells a player casts (CR 118.9; see
            // `kw/offered_costs.rs`).
            CostChange::AlternativeCost(c) if matches!(cm.applies_to, CostTarget::Spells(_)) => {
                self.offered_alternative_cost(&cm.applies_to, c, &target, &who)
            }
            // "You may cast creature spells with mana value 3 or less by paying {E} rather
            // than paying their mana costs. If you cast a spell this way, you may cast it as
            // though it had flash."
            // "Any player may cast creature spells with mana value 3 or less without paying
            // their mana costs and as though they had flash." (Aluren)
            CostChange::AlternativeCostWithFlash(c)
                if c.is_free() && matches!(cm.applies_to, CostTarget::Spells(_)) =>
            {
                let subject = if cm.who == PlayerRel::Any {
                    "any player"
                } else {
                    "you"
                };
                format!(
                    "{subject} may cast {target} without paying their mana costs and as though they had flash"
                )
            }
            CostChange::AlternativeCostWithFlash(c) => {
                let c = self.cost_as_payment(c);
                let c = c.strip_prefix("pay ").unwrap_or(&c);
                let flash = "If you cast a spell this way, you may cast it as though it had flash";
                if matches!(cm.applies_to, CostTarget::Spells(_)) {
                    format!(
                        "you may cast {target} by paying {c} rather than paying their mana costs. {flash}"
                    )
                } else {
                    format!(
                        "you may cast ~ by paying {c} rather than paying its mana cost. {flash}"
                    )
                }
            }
            CostChange::AlternativeCost(_)
                if super::tail_parts::first_ability_alt_cost(self, cm, false).is_some() =>
            {
                super::tail_parts::first_ability_alt_cost(self, cm, false).unwrap_or_default()
            }
            CostChange::AlternativeCost(c) if c.is_free() => {
                let m = self.me();
                format!("you may cast {m} without paying its mana cost")
            }
            CostChange::AlternativeCost(c) => {
                let c = self.cost_as_payment(c);
                format!("you may {c} rather than pay ~'s mana cost")
            }
            CostChange::FlashForAdditionalCost(c) if c.is_free() => {
                format!("you may cast {} as though it had flash", self.me())
            }
            CostChange::FlashForAdditionalCost(c) => {
                let c = self.cost(c);
                format!("you may cast ~ as though it had flash if you pay {c} more to cast it")
            }
            CostChange::OptionalAdditionalCost { cost, .. } => {
                let c = self.cost_as_payment(cost);
                if matches!(cm.applies_to, CostTarget::ThisSpell) {
                    format!("as an additional cost to cast ~, you may {c}")
                } else {
                    format!("as an additional cost to cast {target}, you may {c}")
                }
            }
            CostChange::AdditionalCostChoice(v) => {
                let parts: Vec<String> = v.iter().map(|(_, c)| self.cost_as_payment(c)).collect();
                format!(
                    "as an additional cost to cast ~, {}",
                    join_list(&parts, "or")
                )
            }
        }
    }

    /// "This ability costs {1} less to activate for each legendary creature you control",
    /// "... during your turn", "... if you control an artifact" (an activated ability's own
    /// cost change, CR 601.2f).
    pub(crate) fn own_cost_change(&mut self, oc: &OwnCostChange) -> String {
        let (dir, amt, tail) = match &oc.change {
            CostChange::IncreaseGeneric(v) => {
                let (a, t) = self.cost_amount(v);
                ("more", a, t)
            }
            CostChange::ReduceGeneric(v) => {
                let (a, t) = self.cost_amount(v);
                ("less", a, t)
            }
            CostChange::IncreaseMana(m) => ("more", m.to_string(), String::new()),
            CostChange::ReduceMana { mana, .. } => ("less", mana.to_string(), String::new()),
            CostChange::ReduceColored(c, v) => {
                let sym = mana_symbol(crate::mana::ManaType::from_color(*c));
                match v {
                    Value::Const(n) if *n > 0 => ("less", sym.repeat(*n as usize), String::new()),
                    other => {
                        let v = self.value(other);
                        ("less", sym.to_string(), format!(" for each {v}"))
                    }
                }
            }
            other => {
                return self.gap(format!("own cost change {other:?}"));
            }
        };
        let cond = match &oc.condition {
            None => String::new(),
            Some(Condition::YourTurn) => " during your turn".into(),
            Some(c) => format!(" if {}", self.condition(c)),
        };
        format!("This ability costs {amt} {dir} to activate{tail}{cond}.")
    }

    /// An alternative cost offered for the spells `t` describes (CR 118.9): "You may cast
    /// Dragon spells without paying their mana costs", "You may pay {W}{U}{B}{R}{G} rather
    /// than pay the mana cost for spells you cast", "... for a creature spell you cast from
    /// exile", "you may cast an enchantment spell by paying life equal to its mana value
    /// rather than paying its mana cost" (see `kw/offered_costs.rs`).
    fn offered_alternative_cost(
        &mut self,
        t: &CostTarget,
        c: &Cost,
        target: &str,
        who: &str,
    ) -> String {
        let CostTarget::Spells(f) = t else {
            return String::new();
        };
        if c.mana.is_none() && c.parts.is_empty() {
            return format!("you may cast {target} without paying their mana costs");
        }
        let (kind, quals) = self.offered_spells(f);
        if let [CostPart::PayLife(Value::ManaValueOf(s))] = c.parts.as_slice() {
            if c.mana.is_none() && matches!(**s, Sel::This) {
                let a = if kind.starts_with(['a', 'e', 'i', 'o', 'u']) {
                    "an"
                } else {
                    "a"
                };
                return format!(
                    "you may cast {a} {kind}{quals} by paying life equal to its mana value rather than paying its mana cost"
                );
            }
        }
        // "{X}, where X is that spell's mana value" (Kentaro).
        if let [CostPart::Repeated {
            cost,
            times: Value::ManaValueOf(s),
        }] = c.parts.as_slice()
        {
            if c.mana.is_none() && matches!(**s, Sel::This) && cost.parts.is_empty() {
                return format!(
                    "you may pay {{X}} rather than pay the mana cost for {kind}s{who}{quals}, where X is that spell's mana value"
                );
            }
        }
        let c = self.cost_as_payment(c);
        format!("you may {c} rather than pay the mana cost for {kind}s{who}{quals}")
    }

    /// The spells an offered alternative cost is for, as a kind of spell ("Zombie creature
    /// spell") and the qualifiers that follow "you cast" ("from exile", "that you don't
    /// own", "with mana value 3 or less").
    fn offered_spells(&mut self, f: &Filter) -> (String, String) {
        let parts: Vec<Filter> = match f {
            Filter::And(v) => v
                .iter()
                .flat_map(|x| match x {
                    Filter::And(w) => w.clone(),
                    other => vec![other.clone()],
                })
                .collect(),
            Filter::Any => vec![],
            other => vec![other.clone()],
        };
        let mut kind = Vec::new();
        let mut quals = String::new();
        for p in parts {
            match &p {
                Filter::Or(v) if matches!(v.as_slice(), [Filter::InZone(a), Filter::CastFrom(b)] if a == b) =>
                {
                    let Filter::InZone(z) = v[0] else {
                        continue;
                    };
                    quals.push_str(match z {
                        ZoneKind::Exile => " from exile",
                        ZoneKind::Hand => " from your hand",
                        ZoneKind::Graveyard => " from your graveyard",
                        _ => " from somewhere",
                    });
                }
                Filter::Not(x) if matches!(**x, Filter::OwnedBy(PlayerRel::You)) => {
                    quals.push_str(" that you don't own")
                }
                Filter::ManaValue(Cmp::Le, v) => match &**v {
                    Value::Const(n) => quals.push_str(&format!(" with mana value {n} or less")),
                    other => {
                        let v = self.value(other);
                        quals.push_str(&format!(" with mana value X or less, where X is {v}"));
                    }
                },
                Filter::Spell => {}
                _ => kind.push(p),
            }
        }
        let kind = if kind.is_empty() {
            "spell".to_string()
        } else {
            kind.push(Filter::Spell);
            let n = self.spell_noun_plural(&Filter::and(kind));
            n.strip_suffix('s').unwrap_or(&n).to_string()
        };
        (kind, quals)
    }

    /// "{1}" / "{1} for each artifact you control" / "{X}, where X is ...".
    fn cost_amount(&mut self, v: &Value) -> (String, String) {
        match v {
            Value::Const(n) => (format!("{{{n}}}"), String::new()),
            Value::X => ("{X}".into(), String::new()),
            Value::Count(f) => {
                let n = self.for_each_noun(f);
                ("{1}".into(), format!(" for each {n}"))
            }
            Value::Custom(n) if n == "party_size" => {
                ("{1}".into(), " for each creature in your party".into())
            }
            Value::Custom(n) if n == "spell_targets_beyond_first" => {
                ("{1}".into(), " for each target beyond the first".into())
            }
            Value::Sum(v) if v.iter().all(|x| matches!(x, Value::Count(_))) => {
                let parts: Vec<String> = v
                    .iter()
                    .map(|x| match x {
                        Value::Count(f) => format!("each {}", self.for_each_noun(f)),
                        _ => String::new(),
                    })
                    .collect();
                ("{1}".into(), format!(" for {}", join_list(&parts, "and")))
            }
            other => {
                let s = self.value(other);
                ("{X}".into(), format!(", where X is {s}"))
            }
        }
    }

    /// The text of a restriction (a rule-modifying effect, CR 613.11).
    pub(crate) fn restriction(&mut self, r: &Restriction) -> String {
        let subj = |me: &mut Self, f: &Filter| me.restricted_subject(f);
        match r {
            Restriction::CantAttack(f) => format!("{} can't attack", subj(self, f)),
            Restriction::CantBlock(f) => format!("{} can't block", subj(self, f)),
            Restriction::CantAttackOrBlock(f) => {
                format!("{} can't attack or block", subj(self, f))
            }
            Restriction::CantAttackPlayer {
                attackers,
                defender,
                planeswalkers,
                battles,
            } => {
                let a = subj(self, attackers);
                // "~ can't attack unless defending player controls an Island".
                if let (PlayerFilter::Not(inner), true) = (defender, *planeswalkers) {
                    let c = self.condition(&Condition::PlayerMatches(
                        PlayerRef::DefendingPlayer,
                        (**inner).clone(),
                    ));
                    return format!("{a} can't attack unless {c}");
                }
                let d = self.player_filter_object(defender);
                let mut s = format!("{a} can't attack {d}");
                if *planeswalkers {
                    let p = if d == "you" {
                        "you control"
                    } else {
                        "they control"
                    };
                    s.push_str(&format!(" or planeswalkers {p}"));
                }
                if *battles {
                    s.push_str(" or battles they protect");
                }
                s
            }
            Restriction::Goaded(f) => format!("{} is goaded", subj(self, f)),
            Restriction::DamageByToughness(f) => format!(
                "{} assigns combat damage equal to its toughness rather than its power",
                subj(self, f)
            ),
            Restriction::AssignsNoCombatDamage(f) => {
                format!("{} assigns no combat damage", subj(self, f))
            }
            Restriction::MustAttack(f) => {
                format!("{} attacks each combat if able", subj(self, f))
            }
            Restriction::MustAttackPlayer {
                attackers,
                defender,
            } => {
                let a = subj(self, attackers);
                let d = self.player_filter_object(defender);
                format!("{a} attacks {d} each combat if able")
            }
            Restriction::MustBlock(f) => format!("{} blocks each combat if able", subj(self, f)),
            Restriction::MustBeBlocked(f) => {
                format!("{} must be blocked if able", subj(self, f))
            }
            Restriction::CantBeBlocked(f) => format!("{} can't be blocked", subj(self, f)),
            // "~ can't block creatures with power greater than ~'s power."
            Restriction::CantBeBlockedBy {
                attacker,
                blocker: Filter::Source,
            } => {
                let m = self.me();
                let a = self.noun(attacker, Num::Many);
                format!("{m} can't block {a}")
            }
            // "Target creature can't block ~ this turn": a blocking restriction between
            // two objects (CR 509.1b).
            Restriction::CantBeBlockedBy {
                attacker: Filter::Source,
                blocker: Filter::In(b),
            } if matches!(b.as_ref(), Sel::Target(_)) => {
                let b = self.sel(b, Case::Subj);
                let m = self.me();
                format!("{b} can't block {m}")
            }
            // "~ can't be blocked except by creatures with flying."
            Restriction::CantBeBlockedBy {
                attacker,
                blocker: Filter::Not(allowed),
            } => {
                let a = subj(self, attacker);
                let b = self.noun_det(allowed, Det::Plural);
                format!("{a} can't be blocked except by {b}")
            }
            Restriction::CantBeBlockedBy { attacker, blocker } => {
                let a = subj(self, attacker);
                let b = self.noun(blocker, Num::Many);
                format!("{a} can't be blocked by {b}")
            }
            Restriction::MinBlockers { attacker, n } => {
                let a = subj(self, attacker);
                format!(
                    "{a} can't be blocked except by {} or more creatures",
                    number_word(*n as i32)
                )
            }
            Restriction::MaxBlockedBy { attacker, n } => {
                let a = subj(self, attacker);
                let c = if *n == 1 {
                    "one creature".to_string()
                } else {
                    format!("{} creatures", number_word(*n as i32))
                };
                format!("{a} can't be blocked by more than {c}")
            }
            Restriction::ExtraBlocks { blocker, n } => {
                let b = subj(self, blocker);
                match n {
                    None => format!("{b} can block any number of creatures"),
                    Some(1) => format!("{b} can block an additional creature each combat"),
                    Some(n) => format!(
                        "{b} can block an additional {} creatures each combat",
                        number_word(*n as i32)
                    ),
                }
            }
            Restriction::CanBlockOnly { blocker, attackers } => {
                let b = subj(self, blocker);
                let a = self.noun(attackers, Num::Many);
                format!("{b} can block only {a}")
            }
            Restriction::AttackDespiteDefender(f) => format!(
                "{} can attack as though it didn't have defender",
                subj(self, f)
            ),
            Restriction::AttackAsThoughHaste {
                attackers,
                defender,
            } => {
                let a = if matches!(attackers, Filter::Type(CardType::Creature)) {
                    "all creatures".to_string()
                } else {
                    subj(self, attackers)
                };
                match defender {
                    None => format!("{a} can attack as though it had haste"),
                    Some(d) => {
                        let d = match d {
                            PlayerFilter::Opponent => "your opponents".to_string(),
                            d => self.player_filter_object(d),
                        };
                        format!(
                            "{a} can attack {d} and planeswalkers {d} control as though those creatures had haste"
                        )
                    }
                }
            }
            Restriction::BlockAsThoughUntapped(f) => {
                format!("{} can block as though it were untapped", subj(self, f))
            }
            Restriction::CantAttackAlone(f) => format!("{} can't attack alone", subj(self, f)),
            Restriction::CantBlockAlone(f) => format!("{} can't block alone", subj(self, f)),
            Restriction::MaxAttackers(n) => format!(
                "no more than {} creatures can attack each combat",
                number_word(*n as i32)
            ),
            Restriction::MaxBlockers(n) => format!(
                "no more than {} creatures can block each combat",
                number_word(*n as i32)
            ),
            Restriction::MustBeBlockedByAll(f) => {
                let a = self.noun_det(f, Det::A);
                format!("all creatures able to block {a} do so")
            }
            Restriction::MustBlockAttacker { blocker, attacker } => {
                let named = self.self_salient;
                let b = subj(self, blocker);
                // "target creature blocks it": the object of "blocks" isn't its subject
                // (that would be "itself"), so "it" is ~ when ~ was named before.
                let a = if named && matches!(attacker, Filter::Source) {
                    "~it".to_string()
                } else {
                    self.noun_det(attacker, Det::A)
                };
                format!("{b} blocks {a} each combat if able")
            }
            Restriction::AttackCost {
                attackers,
                defender,
                planeswalkers,
                cost,
            } => {
                let a = subj(self, attackers);
                // Attacking anything (a player, a planeswalker, or a battle, `combat.rs`):
                // "Leviathan can't attack unless you sacrifice two Islands", "Green
                // creatures can't attack unless their controller sacrifices a land for
                // each green creature they control that's attacking".
                if matches!(defender, PlayerFilter::Any) && *planeswalkers {
                    let pay = self.cost_as_payment(cost);
                    if matches!(attackers, Filter::Source) {
                        return format!("{a} can't attack unless you {pay}");
                    }
                    let pays = super::effects::third_person(&pay);
                    let n = self.noun(&super::effects::strip_controller(attackers), Num::One);
                    return format!(
                        "{a} can't attack unless their controller {pays} for each {{alt:{n} they control that's attacking|of those creatures}}"
                    );
                }
                let d = self.player_filter_object(defender);
                let pw = if *planeswalkers {
                    " or planeswalkers you control"
                } else {
                    ""
                };
                let c = self.cost(cost);
                format!(
                    "{a} can't attack {d}{pw} unless their controller pays {c} for each {{alt:creature they control that's attacking {d}|of those creatures}}"
                )
            }
            Restriction::BlockCost { blockers, cost } => {
                let b = subj(self, blockers);
                let c = self.cost(cost);
                format!("{b} can't block unless their controller pays {c} for each blocking creature they control")
            }
            Restriction::CantBeTargeted { what, by } => {
                let w = subj(self, what);
                let by = self.target_restriction(by);
                format!("{w} can't be the target of {by}")
            }
            Restriction::PlayerCantBeTargeted { who, by } => {
                let w = self.player_filter_object(who);
                let by = self.target_restriction(by);
                format!("{w} can't be the target of {by}")
            }
            Restriction::CantCast { who, what } => {
                let w = self.player_filter_subject(who);
                let s = self.spell_noun_plural(what);
                format!("{w} can't cast {s}")
            }
            Restriction::CantActivate {
                who,
                sources,
                include_mana,
            } => {
                // "Activated abilities of sources with the chosen name can't be activated"
                // (a source of an ability can be any object, CR 113.7).
                if matches!(who, PlayerFilter::Any) && matches!(sources, Filter::ChosenName) {
                    let m = if *include_mana {
                        ""
                    } else {
                        " unless they're mana abilities"
                    };
                    return format!(
                        "activated abilities of sources with the chosen name can't be activated{m}"
                    );
                }
                // "Enchanted creature's activated abilities can't be activated."
                if matches!(who, PlayerFilter::Any) && !matches!(sources, Filter::Any) {
                    let s = match sources {
                        Filter::AttachedToSource => self.attached_noun(),
                        Filter::Source => self.me(),
                        other => self.noun_det(other, Det::Plural),
                    };
                    let m = if *include_mana {
                        ""
                    } else {
                        " unless they're mana abilities"
                    };
                    let poss = nouns::possessive(&s);
                    // "Enchanted creature can't attack or block, and its activated
                    // abilities can't be activated." (one line, one subject)
                    let poss = if matches!(sources, Filter::AttachedToSource | Filter::Source)
                        && !poss.contains('{')
                    {
                        format!("{{alt:{poss}|its}}")
                    } else if !poss.contains(['{', '|']) && !s.contains(['{', '|']) {
                        // "Activated abilities of artifacts can't be activated."
                        return format!(
                            "{{alt:{poss} activated abilities|activated abilities of {s}}} can't be activated{m}"
                        );
                    } else {
                        // "Activated abilities of artifacts and creatures" (a list with
                        // its own alternatives).
                        return format!("activated abilities of {s} can't be activated{m}");
                    };
                    return format!("{poss} activated abilities can't be activated{m}");
                }
                let w = self.player_filter_subject(who);
                let s = if matches!(sources, Filter::Any) {
                    "abilities".to_string()
                } else {
                    let n = self.noun(sources, Num::Many);
                    format!("abilities of {n}")
                };
                let m = if *include_mana {
                    ""
                } else {
                    " that aren't mana abilities"
                };
                format!("{w} can't activate {s}{m}")
            }
            Restriction::CantBeCountered(f) => format!("{} can't be countered", subj(self, f)),
            Restriction::CantBeCopied(f) => format!("{} can't be copied", subj(self, f)),
            Restriction::CantCauseSacrifice { what, by, exile } => {
                let who = match by {
                    SacrificeCauses::OpponentsSpellsAndAbilities => {
                        "spells and abilities your opponents control"
                    }
                    SacrificeCauses::YourTriggeredAbilities => "triggered abilities you control",
                };
                let verb = if *exile {
                    "sacrifice or exile"
                } else {
                    "sacrifice"
                };
                let n = self.noun(what, Num::Many);
                format!("{who} can't cause you to {verb} {n}")
            }
            Restriction::CantPayToCastOrActivate {
                who,
                life,
                sacrifice,
                mana_abilities,
            } => {
                let w = self.player_filter_subject(who);
                let mut what = Vec::new();
                if *life {
                    what.push("pay life".to_string());
                }
                if let Some(f) = sacrifice {
                    what.push(format!("sacrifice {}", self.noun(f, Num::Many)));
                }
                let purpose = if *mana_abilities {
                    "to cast spells or activate abilities"
                } else {
                    "to cast spells or to activate abilities that aren't mana abilities"
                };
                format!("{w} can't {} {purpose}", what.join(" or "))
            }
            Restriction::CantEnterBattlefield(f) | Restriction::CantEnter(f) => {
                format!("{} can't enter the battlefield", subj(self, f))
            }
            Restriction::CantEnterFrom { what, zones } => {
                let zones: Vec<String> = zones
                    .iter()
                    .map(|z| match z {
                        ZoneKind::Library => "libraries".to_string(),
                        ZoneKind::Graveyard => "graveyards".to_string(),
                        ZoneKind::Hand => "hands".to_string(),
                        z => format!("{z:?}").to_lowercase(),
                    })
                    .collect();
                format!(
                    "{} in {} can't enter the battlefield",
                    subj(self, what),
                    join_list(&zones, "and")
                )
            }
            Restriction::DoesntUntap(f) => {
                let s = subj(self, f);
                let whose = if matches!(f, Filter::Source)
                    || super::values::split_controller(f).0 == Some(PlayerRel::You)
                {
                    "your"
                } else {
                    "its controller's"
                };
                format!("{s} doesn't untap during {whose} untap step")
            }
            Restriction::MaxUntaps { who, what, n } => {
                let w = self.player_filter_subject(who);
                let x = if *n == 1 {
                    self.noun(what, Num::One)
                } else {
                    self.noun(what, Num::Many)
                };
                let steps = if w == "you" {
                    "your untap step"
                } else {
                    "their untap steps"
                };
                format!(
                    "{w} can't untap more than {} {x} during {steps}",
                    number_word(*n as i32)
                )
            }
            Restriction::UntapDuringOthersUntapSteps(f) => {
                let s = subj(self, f);
                format!("untap {s} during each other player's untap step")
            }
            Restriction::CantGainLife(p) => {
                format!("{} can't gain life", self.player_filter_subject(p))
            }
            Restriction::CantLoseLife(p) => {
                format!("{} can't lose life", self.player_filter_subject(p))
            }
            Restriction::CantLoseGame(p) => {
                format!("{} can't lose the game", self.player_filter_subject(p))
            }
            Restriction::CantWinGame(p) => {
                format!("{} can't win the game", self.player_filter_subject(p))
            }
            Restriction::MaxDrawsPerTurn(p, n) => {
                let w = self.player_filter_subject(p);
                let c = if *n == 1 {
                    "one card".to_string()
                } else {
                    format!("{} cards", number_word(*n as i32))
                };
                format!("{w} can't draw more than {c} each turn")
            }
            Restriction::MaxSpellsOfKindPerTurn { who, what, n } => {
                let w = self.player_filter_subject(who);
                let s = self.spell_noun_plural(what);
                let s = s.trim_end_matches('s');
                let c = number_word(*n as i32);
                format!("{w} can't cast more than {c} {s} each turn")
            }
            Restriction::MaxSpellsPerTurn(p, n) => {
                let w = self.player_filter_subject(p);
                let c = if *n == 1 {
                    "one spell".to_string()
                } else {
                    format!("{} spells", number_word(*n as i32))
                };
                format!("{w} can't cast more than {c} each turn")
            }
            Restriction::CantBeSacrificed(f) => {
                format!("{} can't be sacrificed", subj(self, f))
            }
            Restriction::CantBeRegenerated(f) => {
                format!("{} can't be regenerated", subj(self, f))
            }
            Restriction::DamageCantBePrevented => "damage can't be prevented".into(),
            Restriction::CombatDamageCantBePrevented(f) => {
                if matches!(f, Filter::Any) {
                    "combat damage can't be prevented".into()
                } else {
                    let n = self.noun(f, Num::Many);
                    format!("combat damage that would be dealt by {n} can't be prevented")
                }
            }
            Restriction::SourceDamageCantBePrevented(f) => {
                if matches!(f, Filter::Source) {
                    "damage that would be dealt by ~ can't be prevented".into()
                } else {
                    let n = self.noun(f, Num::Many);
                    format!("damage that would be dealt by {n} can't be prevented")
                }
            }
            Restriction::CantTransform(f) => format!("{} can't transform", subj(self, f)),
            // "As long as enchanted creature is face down, it can't be turned face up."
            Restriction::CantTurnFaceUp(Filter::And(v))
                if v.len() >= 2 && matches!(v.last(), Some(Filter::FaceDown)) =>
            {
                let rest = Filter::and(v[..v.len() - 1].to_vec());
                format!(
                    "as long as {} is face down, it can't be turned face up",
                    subj(self, &rest)
                )
            }
            Restriction::CantTurnFaceUp(f) => {
                format!("{} can't be turned face up", subj(self, f))
            }
            Restriction::CantSearch(p) => {
                format!("{} can't search libraries", self.player_filter_subject(p))
            }
            Restriction::SorcerySpeedOnly(p) => {
                let w = self.player_filter_subject(p);
                format!("{w} can cast spells only any time they could cast a sorcery")
            }
            Restriction::CantPlayLands(p) => {
                format!("{} can't play lands", self.player_filter_subject(p))
            }
            Restriction::CantPlayLandCards { who, what } => {
                let w = self.player_filter_subject(who);
                let n = self.noun(what, Num::Many);
                format!("{w} can't play {n}")
            }
            Restriction::MustTarget { chooser, what } => {
                let c = self.player_filter_object(chooser);
                let n = self.noun_det(what, Det::A);
                format!(
                    "while {c} is choosing targets as part of casting a spell or activating an ability, that player must choose at least {n} if able"
                )
            }
            Restriction::CantBe { what, action } => {
                let w = subj(self, what);
                let a = match action {
                    ObjectAction::Untapped => "can't become untapped",
                    ObjectAction::PhasedIn => "can't phase in",
                    ObjectAction::Equipped => "can't be equipped",
                    ObjectAction::EnchantedByOtherAuras => "can't be enchanted by other Auras",
                    ObjectAction::Suspected => "can't become suspected",
                };
                format!("{w} {a}")
            }
            Restriction::AttackTogether {
                attackers,
                triggers,
                ..
            } => {
                let t = self.noun_det(triggers, Det::A);
                let a = subj(self, attackers);
                format!("if {t} attacks, {a} attacks if able")
            }
            Restriction::MustAttackOtherThan { attackers, players } => {
                let a = subj(self, attackers);
                let p = self.player_filter_object(players);
                format!("{a} attacks a player other than {p} if able")
            }
            Restriction::AttackOnlyAlone(f) => format!("{} can only attack alone", subj(self, f)),
            Restriction::MaxAttackersAgainst { player, object, n } => {
                let d = match (player, object) {
                    (Some(p), _) => self.player_filter_object(p),
                    (None, Some(o)) => self.noun_det(o, Det::A),
                    (None, None) => "anything".into(),
                };
                let (n, noun) = if *n == 1 {
                    ("one".to_string(), "creature")
                } else {
                    (number_word(*n as i32), "creatures")
                };
                format!("no more than {n} {noun} can attack {d} each combat")
            }
            Restriction::MustBeBlockedBy { attacker, blocker } => {
                let a = subj(self, attacker);
                let b = self.noun_det(blocker, Det::A);
                format!("{a} must be blocked by {b} if able")
            }
            Restriction::BlockerCountRequirement { attacker, min, max } => {
                let a = subj(self, attacker);
                let n = match max {
                    Some(m) if m == min => format!("exactly {}", number_word(*m as i32)),
                    _ => format!("{} or more", number_word(*min as i32)),
                };
                let noun = if *min == 1 && *max == Some(1) {
                    "creature"
                } else {
                    "creatures"
                };
                format!("{a} must be blocked by {n} {noun} if able")
            }
            Restriction::MaxBlockersOf { who, n } => {
                let w = self.player_filter_subject(who);
                let noun = if *n == 1 { "creature" } else { "creatures" };
                format!(
                    "{w} can't block with more than {} {noun}",
                    number_word(*n as i32)
                )
            }
            Restriction::Custom(name) => self.custom_restriction(name),
        }
    }

    /// "you", "your opponents", "each player" as the subject of a restriction.
    fn player_filter_subject(&mut self, pf: &PlayerFilter) -> String {
        match pf {
            PlayerFilter::You | PlayerFilter::Controller => "you".into(),
            PlayerFilter::Opponent => "your opponents".into(),
            PlayerFilter::Any => "players".into(),
            PlayerFilter::NotYou => "other players".into(),
            other => self.player_filter_object(other),
        }
    }

    fn target_restriction(&mut self, by: &TargetRestriction) -> String {
        match by {
            TargetRestriction::Opponents => "spells or abilities your opponents control".into(),
            TargetRestriction::Any => "spells or abilities".into(),
            TargetRestriction::Sources(f) => {
                let n = self.noun(f, Num::Many);
                if n.contains("spell") || n.contains("abilit") {
                    n
                } else {
                    format!("{n} spells or abilities from {n} sources")
                }
            }
            TargetRestriction::OpponentsSources(f) => {
                let n = self.noun(f, Num::Many);
                if n.contains("spell") || n.contains("abilit") {
                    format!("{n} your opponents control")
                } else {
                    format!("{n} spells your opponents control or abilities from {n} sources your opponents control")
                }
            }
        }
    }

    /// The duration of a restriction created by a resolving effect: "this turn" is how
    /// cards say "until end of turn" for rule-modifying effects.
    pub(crate) fn restriction_duration(&mut self, d: &Duration) -> String {
        match d {
            Duration::EndOfTurn | Duration::ThisTurn => "this turn".into(),
            other => self.duration(other),
        }
    }

    /// A replacement or prevention effect.
    pub(crate) fn replacement(&mut self, def: &ReplacementDef, uses: Option<u32>) -> String {
        let s = self.in_event_scope(|r| r.replacement_inner(def, uses));
        if def.optional && !s.contains(" may ") {
            format!("optionally, {s}")
        } else {
            s
        }
    }

    fn replacement_inner(&mut self, def: &ReplacementDef, uses: Option<u32>) -> String {
        use ReplacementAction as A;
        use ReplacementEvent as E;
        let _ = uses;
        match (&def.event, &def.action) {
            // --- Entering the battlefield (CR 614.1c).
            (E::EntersBattlefield(f), action) => {
                let subj = self.enters_subject(f);
                let it = "it";
                match action {
                    A::EnterTapped if subj != "~" && subj != "~it" => {
                        format!("{subj} enter tapped")
                    }
                    A::EnterTapped => format!("{subj} enters tapped"),
                    A::EnterUntapped if subj != "~" && subj != "~it" => {
                        format!("{subj} enter untapped")
                    }
                    A::EnterUntapped => format!("{subj} enters untapped"),
                    A::EnterWithCounters(k, n) => {
                        let (c, w) = self.counted(n, &counter_name(k));
                        format!("{subj} enters with {c} on {it}{}", w.unwrap_or_default())
                    }
                    // "~ escapes with a +1/+1 counter on it" (CR 702.138c).
                    A::AsEnters(e)
                        if matches!(e.as_ref(), Effect::If {
                        cond: Condition::CostPaid(c),
                        then,
                        otherwise,
                    } if c == "escape"
                        && matches!(otherwise.as_ref(), Effect::Noop)
                        && matches!(then.as_ref(), Effect::EnterWithCounters { .. })
                        && (subj == "~" || subj == "~it")) =>
                    {
                        let Effect::If { then, .. } = e.as_ref() else {
                            return self.gap("escapes with");
                        };
                        let Effect::EnterWithCounters { kind, n } = then.as_ref() else {
                            return self.gap("escapes with");
                        };
                        let (c, w) = self.counted(n, &counter_name(kind));
                        format!("~ escapes with {c} on {it}{}", w.unwrap_or_default())
                    }
                    A::AsEnters(e) => self.as_enters(&subj, e),
                    A::EnterAsCopy { filter, optional } => {
                        let n = format!("any {}", self.noun(filter, Num::One));
                        if *optional {
                            format!("you may have {subj} enter as a copy of {n} on the battlefield")
                        } else {
                            format!("{subj} enters as a copy of {n} on the battlefield")
                        }
                    }
                    A::EnterUnderControl(p) => {
                        let p = self.player(p, Case::Obj);
                        format!("{subj} enters under the control of {p}")
                    }
                    A::EnterTransformed => format!("{subj} enters transformed"),
                    A::MoveInstead(d) => {
                        let d = self.destination_phrase(d, false, false);
                        format!("if {subj} would enter, put it {d} instead")
                    }
                    A::Instead(e) => {
                        let e = self.effect(e);
                        format!("if {subj} would enter, {e} instead")
                    }
                    A::Also(e) => {
                        let e = self.effect(e);
                        format!("as {subj} enters, {e}")
                    }
                    A::Prevent => format!("{subj} can't enter the battlefield"),
                    other @ (A::PreventAmount(_)
                    | A::PreventPortion(_)
                    | A::Multiply(_)
                    | A::Add(_)
                    | A::PlusTokens { .. }
                    | A::Subtract(_)
                    | A::Redirect(_)
                    | A::RedirectNext(..)
                    | A::Regenerate
                    | A::PreventAndThen(..)
                    | A::LifeFloor(_)
                    | A::ManaTypeInstead(_)) => {
                        let then = self.replacement_then(other, "it");
                        format!("if {subj} would enter, {then}")
                    }
                }
            }
            (E::TurnedFaceUp, A::AsEnters(e)) => {
                let m = self.me();
                let e = self.effect(e);
                format!("as {m} is turned face up, {e}")
            }
            (E::Transforms, A::AsEnters(e)) => {
                let m = self.me();
                let e = self.effect(e);
                format!("as {m} transforms, {e}")
            }
            // --- Zone changes.
            (E::Dies(f), action) => {
                let subj = self.noun_det(f, Det::A);
                let then = self.replacement_then(action, "it");
                format!("if {subj} would die, {then}")
            }
            (E::ZoneChange { filter, from, to }, action) => {
                let subj = self.noun_det(filter, Det::A);
                let where_ = match (from, to) {
                    (Some(ZoneKind::Battlefield), Some(ZoneKind::Graveyard)) => "die".to_string(),
                    (Some(f), Some(t)) => format!(
                        "be put into {} from {}",
                        self.zone_any(*t),
                        self.zone_src(*f)
                    ),
                    // A card is put into its owner's graveyard (CR 400.3); a zone change
                    // with no origin is one from anywhere.
                    (None, Some(ZoneKind::Graveyard)) => {
                        "be put {alt:into a graveyard|into its owner's graveyard} {opt:from anywhere}"
                            .to_string()
                    }
                    (None, Some(t)) => {
                        format!("be put into {} {{opt:from anywhere}}", self.zone_any(*t))
                    }
                    (Some(f), None) => format!("leave {}", self.zone_src(*f)),
                    (None, None) => "change zones".into(),
                };
                let then = self.replacement_then(action, "it");
                format!("if {subj} would {where_}, {then}")
            }
            (E::Destroy(f), action) => {
                let subj = self.noun_det(f, Det::A);
                match action {
                    A::Regenerate => format!("regenerate {subj}"),
                    other => {
                        let then = self.replacement_then(other, "it");
                        format!("if {subj} would be destroyed, {then}")
                    }
                }
            }
            // --- Damage (CR 615 prevention, 614.1a replacement).
            (
                E::Damage {
                    source,
                    to_players,
                    to_objects,
                    combat_only,
                },
                action,
            ) => {
                self.damage_replacement(source, to_players, to_objects, *combat_only, false, action)
            }
            (
                E::NoncombatDamage {
                    source,
                    to_players,
                    to_objects,
                },
                action,
            ) => self.damage_replacement(source, to_players, to_objects, false, true, action),
            // --- Players.
            (E::Draw(p), action) => {
                // "If you would draw a card except the first one you draw in each of your
                // draw steps".
                let (p, except) = match p {
                    PlayerFilter::And(v)
                        if v.iter().any(|x| matches!(x, PlayerFilter::Not(f) if matches!(f.as_ref(), PlayerFilter::FirstDrawInDrawStep))) =>
                    {
                        let rest: Vec<PlayerFilter> = v
                            .iter()
                            .filter(|x| !matches!(x, PlayerFilter::Not(f) if matches!(f.as_ref(), PlayerFilter::FirstDrawInDrawStep)))
                            .cloned()
                            .collect();
                        let rest = match rest.as_slice() {
                            [one] => one.clone(),
                            _ => PlayerFilter::And(rest),
                        };
                        (rest, true)
                    }
                    other => (other.clone(), false),
                };
                let w = self.player_filter_subject(&p);
                let w = match w.as_str() {
                    "players" => "a player".into(),
                    "your opponents" if except => "an opponent".into(),
                    _ => w,
                };
                let then = self.replacement_then(action, "");
                let except = if !except {
                    String::new()
                } else if w == "you" {
                    " except the first one you draw in each of your draw steps".into()
                } else {
                    " except the first one they draw in each of their draw steps".into()
                };
                format!("if {w} would draw a card{except}, {then}")
            }
            // Keyword actions (CR 701): "If an opponent would mill one or more cards, they
            // mill twice that many cards instead.", "If you would proliferate, ...", "If a
            // creature you control would explore, ...".
            (E::Action { kind, who, objects }, action) => {
                use crate::ability::ReplaceableAction as K;
                let (verb, counted) = match kind {
                    K::Mill => ("mill", Some("one or more cards")),
                    K::Scry => ("scry", Some("a number of cards")),
                    K::Proliferate => ("proliferate", None),
                    K::Explore => ("explore", None),
                    K::Connive => ("connive", None),
                    K::Learn => ("learn", None),
                };
                let w = match objects {
                    Some(f) => self.noun_det(f, Det::A),
                    None => {
                        let w = self.player_filter_subject(who);
                        match w.as_str() {
                            "players" => "a player".into(),
                            "your opponents" => "an opponent".into(),
                            _ => w,
                        }
                    }
                };
                let they = if w == "you" { "you" } else { "they" };
                let then = match (action, counted) {
                    (A::Multiply(k), Some(_)) => {
                        let k = match k {
                            2 => "twice".to_string(),
                            3 => "three times".to_string(),
                            k => format!("{k} times"),
                        };
                        format!("{they} {verb} {k} that many cards instead")
                    }
                    (A::Add(v), Some(_)) => {
                        let v = self.value(v);
                        format!("{they} {verb} that many cards plus {v} instead")
                    }
                    (other, _) => self.replacement_then(other, ""),
                };
                let what = counted.map(|c| format!(" {c}")).unwrap_or_default();
                format!("if {w} would {verb}{what}, {then}")
            }
            (E::DrawCards { who, min }, action) => {
                let w = self.player_filter_subject(who);
                let then = self.replacement_then(action, "");
                format!(
                    "if {w} would draw {} or more cards, {then}",
                    number_word(*min as i32)
                )
            }
            (E::GainLife(p), action) => {
                let w = self.player_filter_subject(p);
                let w = if w == "players" { "a player".into() } else { w };
                let they = if w == "you" { "you" } else { "that player" };
                let then = match action {
                    A::Multiply(2) => format!("{they} gain twice that much life instead"),
                    A::Add(v) => {
                        let v = self.value(v);
                        format!("{they} gain that much life plus {v} instead")
                    }
                    A::Prevent => "they gain no life instead".into(),
                    other => self.replacement_then(other, ""),
                };
                format!("if {w} would gain life, {then}")
            }
            (E::LoseLife(p), action) => {
                let w = self.player_filter_subject(p);
                let w = if w == "players" { "a player".into() } else { w };
                let they = if w == "you" { "you" } else { "that player" };
                let then = match action {
                    A::Multiply(2) => format!("{they} lose twice that much life instead"),
                    A::Add(v) => {
                        let v = self.value(v);
                        format!("{they} lose that much life plus {v} instead")
                    }
                    other => self.replacement_then(other, ""),
                };
                format!("if {w} would lose life, {then}")
            }
            (E::LifeLossFromDamage(p), A::LifeFloor(v)) => {
                let w = self.player_filter_subject(p);
                let v = self.value(v);
                format!("damage that would reduce {} life total to less than {v} reduces it to {v} instead", nouns::possessive(&w))
            }
            // "Creatures your opponents control can't have +1/+1 counters put on them."
            (
                E::PutCounters {
                    on_objects: Some(f),
                    on_players: None,
                    kind,
                },
                A::Prevent,
            ) if !matches!(f, Filter::Source) => {
                let k = match kind {
                    Some(k) => plural(&counter_name(k)),
                    None => "counters".into(),
                };
                let subj = self.affected_subject(f);
                format!("{subj} can't have {k} put on them")
            }
            (
                E::PutCounters {
                    on_objects,
                    on_players,
                    kind,
                },
                action,
            ) => {
                let k = match kind {
                    Some(k) => counter_name(k),
                    None => "counter".into(),
                };
                let mut on = Vec::new();
                if let Some(o) = on_objects {
                    on.push(self.noun_det(o, Det::A));
                }
                if let Some(p) = on_players {
                    on.push(self.player_filter_object(p));
                }
                let on = join_list(&on, "or");
                // Counters of any kind: "twice that many of each of those kinds of
                // counters".
                let many = if kind.is_none() {
                    "{alt:of each of those kinds of counters|counters}".to_string()
                } else {
                    plural(&k)
                };
                let then = match action {
                    A::Multiply(2) => {
                        format!("twice that many {many} are put on it instead")
                    }
                    A::Add(v) => {
                        let v = match v {
                            Value::Const(1) => "one".to_string(),
                            other => self.value(other),
                        };
                        format!("that many plus {v} {} are put on it instead", plural(&k))
                    }
                    A::Prevent => "they aren't put on it instead".into(),
                    other => self.replacement_then(other, "it"),
                };
                format!("if one or more {} would be put on {on}, {then}", plural(&k))
            }
            // The event is any counters a matching player puts, on a permanent or on a
            // player (CR 122.6a).
            (E::PutCountersBy { by, kind }, action) => {
                let w = self.rel_subject(*by);
                let (k, each) = match kind {
                    Some(k) => (plural(&counter_name(k)), plural(&counter_name(k))),
                    None => (
                        "counters".to_string(),
                        "{alt:of each of those kinds of counters|counters}".to_string(),
                    ),
                };
                let they = if w == "you" {
                    "you".to_string()
                } else {
                    "they".to_string()
                };
                let on = "on {alt:that permanent or player|it}";
                let then = match action {
                    A::Multiply(2) => {
                        format!("{{opt:{they}}} put twice that many {each} {on} instead")
                    }
                    A::Multiply(3) => {
                        format!("{{opt:{they}}} put three times that many {each} {on} instead")
                    }
                    A::Add(v) => {
                        let v = match v {
                            Value::Const(1) => "one".to_string(),
                            other => self.value(other),
                        };
                        format!("{{opt:{they}}} put that many plus {v} {each} {on} instead")
                    }
                    // Half rounded down remains when half rounded up is taken away (see
                    // `r122_counters_put_by.rs`).
                    A::Subtract(Value::Div(a, 2, up))
                        if matches!(a.as_ref(), Value::EventAmount) =>
                    {
                        let r = if *up { "down" } else { "up" };
                        format!("{they} put half that many {each} {on} instead, rounded {r}")
                    }
                    other => self.replacement_then(other, "it"),
                };
                format!("if {w} would put one or more {k} on a permanent or player, {then}")
            }
            (E::CreateTokens(p), action) => {
                let w = self.player_filter_subject(p);
                let w = if w == "players" { "a player".into() } else { w };
                let poss = nouns::possessive(&w);
                match action {
                    // Current Oracle wording says what's created, older wording what an
                    // effect creates; tokens created under any player's control need no
                    // "under ... control".
                    A::Multiply(2) => {
                        let c = if w == "a player" {
                            String::new()
                        } else {
                            format!(" under {poss} control")
                        };
                        format!(
                            "if {{alt:one or more tokens would be created{c}, twice that many of those tokens are created|an effect would create one or more tokens{c}, it creates twice that many of those tokens}} instead"
                        )
                    }
                    A::PlusTokens { spec, count } => {
                        let (d, tail) = self.token_desc(spec);
                        let plus = match count {
                            // "those tokens plus a Food token".
                            Value::Const(1) => with_article(&format!("{d} token")),
                            Value::EventAmount => format!("that many {d} tokens"),
                            other => format!("{} {d} tokens", self.value(other)),
                        };
                        format!("if one or more tokens would be created under {poss} control, those tokens plus {plus}{tail} are created instead")
                    }
                    other => {
                        let then = self.replacement_then(other, "");
                        format!("if {w} would create one or more tokens, {then}")
                    }
                }
            }
            (E::CreateTokensMatching { who, tokens }, action) => {
                let w = self.player_filter_subject(who);
                let t = self.noun(tokens, Num::Many);
                let then = match action {
                    A::Multiply(2) => {
                        "twice that many of those tokens are created instead".to_string()
                    }
                    other => self.replacement_then(other, ""),
                };
                format!(
                    "if one or more {t} would be created under {} control, {then}",
                    nouns::possessive(&w)
                )
            }
            (E::LoseGame(p), action) => {
                let w = self.player_filter_subject(p);
                let then = self.replacement_then(action, "");
                format!("if {w} would lose the game, {then}")
            }
            (E::SkipStep { step, whose }, A::Prevent) => {
                let s = match step {
                    StepKind::Untap => "untap step",
                    StepKind::Upkeep => "upkeep step",
                    StepKind::Draw => "draw step",
                    StepKind::Main => "main phase",
                    StepKind::Combat => "combat phase",
                    StepKind::End => "end step",
                    StepKind::Turn => "turn",
                };
                let p = self.rel_possessive(*whose, Num::One);
                format!("skip {p} {s}")
            }
            (E::ProduceMana(f), action) => {
                let n = self.noun_det(f, Det::A);
                let then = match action {
                    A::Multiply(2) => "it produces twice as much of that mana instead".to_string(),
                    A::Multiply(3) => {
                        "it produces three times as much of that mana instead".to_string()
                    }
                    A::ManaTypeInstead(t) => {
                        format!("it produces {} instead of any other type", mana_symbol(*t))
                    }
                    A::Also(e) => {
                        let e = self.effect(e);
                        format!("{e} in addition")
                    }
                    other => self.replacement_then(other, "it"),
                };
                format!("if {n} is tapped for mana, {then}")
            }
            (E::UntapDuringUntapStep(f), A::Prevent) => {
                let s = self.affected_subject(f);
                format!("{s} doesn't untap during its controller's untap step")
            }
            (E::Countered(f), action) => {
                let n = self.noun_det(f, Det::A);
                let then = self.replacement_then(action, "it");
                format!("if {n} would be countered, {then}")
            }
            (E::Discard(p, f), action) => {
                let w = self.player_filter_subject(p);
                let n = self.noun_det(f, Det::A);
                let then = self.replacement_then(action, "it");
                format!("if {w} would discard {n}, {then}")
            }
            (E::Mill(p), action) => {
                let w = self.player_filter_subject(p);
                let then = self.replacement_then(action, "");
                format!("if {w} would mill one or more cards, {then}")
            }
            (E::Search(p), action) => {
                let w = self.player_filter_subject(p);
                let then = self.replacement_then(action, "");
                format!("if {w} would search a library, {then}")
            }
            // "If an effect would put one or more counters on a permanent you control, it
            // puts twice that many of those counters on that permanent instead."
            (
                E::PutCountersMatching {
                    on_objects,
                    on_players,
                    kind,
                    by,
                    effect_only,
                },
                action @ (A::Multiply(_) | A::Add(_)),
            ) => self.counters_matching_replacement(
                on_objects.as_ref(),
                on_players.as_ref(),
                kind.as_ref(),
                *by,
                *effect_only,
                action,
            ),
            (event, action) => self.gap(format!(
                "replacement {:?} / {:?}",
                std::mem::discriminant(event),
                std::mem::discriminant(action)
            )),
        }
    }

    /// The subject of an "enters" replacement: "~", "creatures your opponents control".
    fn enters_subject(&mut self, f: &Filter) -> String {
        match f {
            Filter::Source => self.me(),
            other => self.noun_det(other, Det::Plural),
        }
    }

    fn as_enters(&mut self, subj: &str, e: &Effect) -> String {
        // "~ enters with your choice of a reach counter or a vigilance counter on it."
        if let Effect::ChooseOne {
            who: PlayerRef::You,
            options,
        } = e
        {
            let one = |x: &Effect| match x {
                Effect::EnterWithCounters {
                    kind,
                    n: Value::Const(1),
                } => Some(kind.to_string()),
                _ => None,
            };
            let singles: Option<Vec<String>> = options.iter().map(|(_, x)| one(x)).collect();
            if let Some(kinds) = singles.filter(|k| k.len() >= 2) {
                let full: Vec<String> = kinds
                    .iter()
                    .map(|k| with_article(&counter_name(k)))
                    .collect();
                let short: Vec<String> = kinds.iter().map(|k| k.to_string()).collect();
                let short = with_article(&format!("{} counter", join_list(&short, "or")));
                return format!(
                    "{subj} enters with your choice of {{alt:{}|{short}}} on it",
                    join_list(&full, "or")
                );
            }
            // "your choice of two different counters on it from among menace, deathtouch,
            // and lifelink": every pair of different kinds.
            let pairs: Option<Vec<(String, String)>> = options
                .iter()
                .map(|(_, x)| match x {
                    Effect::Seq(v) if v.len() == 2 => {
                        Some((one(&v[0])?, one(&v[1])?)).filter(|(a, b)| a != b)
                    }
                    _ => None,
                })
                .collect();
            if let Some(pairs) = pairs {
                let mut kinds: Vec<String> = Vec::new();
                for (a, b) in &pairs {
                    for k in [a, b] {
                        if !kinds.contains(k) {
                            kinds.push(k.clone());
                        }
                    }
                }
                let n = kinds.len();
                let all_pairs = pairs.len() == n * (n - 1) / 2
                    && kinds.iter().enumerate().all(|(i, a)| {
                        kinds[i + 1..].iter().all(|b| {
                            pairs
                                .iter()
                                .any(|(x, y)| (x == a && y == b) || (x == b && y == a))
                        })
                    });
                if all_pairs && n >= 3 {
                    return format!(
                        "{subj} enters with your choice of two different counters on it from among {}",
                        join_list(&kinds, "and")
                    );
                }
            }
        }
        match e {
            // CR 702.138c: "~ escapes with [counters]" means "If this permanent escaped, it
            // enters with [those counters]" (it escaped if its escape cost was paid).
            Effect::If {
                cond: Condition::CostPaid(name),
                then,
                otherwise,
            } if name == "escape"
                && matches!(otherwise.as_ref(), Effect::Noop)
                && matches!(then.as_ref(), Effect::EnterWithCounters { .. }) =>
            {
                let Effect::EnterWithCounters { kind, n } = then.as_ref() else {
                    return self.gap("escapes with");
                };
                let (c, w) = self.counted(n, &counter_name(kind));
                format!("{subj} escapes with {c} on it{}", w.unwrap_or_default())
            }
            // CR 307.5a: "If you cast it any time a sorcery couldn't have been cast, the
            // controller of the permanent it becomes sacrifices it at the beginning of the
            // next cleanup step."
            Effect::If {
                cond: Condition::Custom(c),
                then,
                otherwise,
            } if c
                == crate::oracle::patterns::r307_sorcery_timing::CAST_BY_OWN_FLASH_AT_INSTANT_TIMING
                && matches!(otherwise.as_ref(), Effect::Noop)
                && matches!(then.as_ref(), Effect::OnEntry(x)
                    if matches!(x.as_ref(), Effect::AtNext { step: TriggerStep::Cleanup, effect }
                        if matches!(effect.as_ref(), Effect::SacrificeObjects { what: Sel::This }))) =>
            {
                "if you cast it any time a sorcery couldn't have been cast, the controller of \
                 the permanent it becomes sacrifices it at the beginning of the next cleanup step"
                    .into()
            }
            Effect::If {
                cond,
                then,
                otherwise,
            } if matches!(then.as_ref(), Effect::Noop)
                && !matches!(otherwise.as_ref(), Effect::Noop) =>
            {
                let t = self.as_enters_vp(otherwise);
                let c = self.condition(cond);
                format!("{subj} {t} unless {c}")
            }
            Effect::If {
                cond,
                then,
                otherwise,
            } if matches!(otherwise.as_ref(), Effect::Noop)
                && !matches!(cond, Condition::Not(_))
                && !mentions_entry_modification(then) =>
            {
                // "If it's neither day nor night, it becomes day as ~ enters."
                let c = self.condition(cond);
                let s = self.effect(then);
                format!("if {c}, {s} as {subj} enters")
            }
            Effect::If {
                cond,
                then,
                otherwise,
            } if matches!(otherwise.as_ref(), Effect::Noop) => {
                let t = self.as_enters_vp(then);
                match cond {
                    Condition::Not(inner) => {
                        let c = self.condition(inner);
                        format!("{subj} {t} unless {c}")
                    }
                    other => {
                        // "If this spell was kicked, it enters with ...": the condition
                        // comes first, so the object is named there.
                        let was_self = subj == "~";
                        if was_self {
                            self.self_salient = false;
                        }
                        let c = self.condition(other);
                        let subj = if was_self {
                            self.me()
                        } else {
                            subj.to_string()
                        };
                        format!("if {c}, {subj} {t}")
                    }
                }
            }
            Effect::EnterTapped | Effect::EnterWithCounters { .. } | Effect::EnterPrepared => {
                let t = self.as_enters_vp(e);
                format!("{subj} {t}")
            }
            Effect::Seq(v)
                if v.iter().all(|x| {
                    matches!(x, Effect::EnterTapped | Effect::EnterWithCounters { .. })
                }) =>
            {
                let parts: Vec<String> = v.iter().map(|x| self.as_enters_vp(x)).collect();
                format!("{subj} {}", join_list(&parts, "and"))
            }
            other => {
                let s = self.effect(other);
                // "~ enters tapped. As it enters, choose a color.": the object itself is
                // named again by "it" when another ability on the line named it.
                let subj = if subj == "~" { "~it" } else { subj };
                format!("as {subj} enters, {s}")
            }
        }
    }

    /// "enters tapped", "enters with two +1/+1 counters on it".
    fn as_enters_vp(&mut self, e: &Effect) -> String {
        match e {
            Effect::EnterTapped => "enters tapped".into(),
            Effect::EnterPrepared => "enters prepared".into(),
            Effect::EnterWithCounters { kind, n } => {
                let (c, w) = self.counted(n, &counter_name(kind));
                format!("enters with {c} on it{}", w.unwrap_or_default())
            }
            // "it enters with two +1/+1 counters on it and with haste": the keywords it
            // has from the moment it enters.
            Effect::OnEntry(inner)
                if matches!(inner.as_ref(), Effect::Modify { what: Sel::This, mods, duration: Duration::Permanent }
                    if !mods.is_empty() && mods.iter().all(|m| matches!(m, Modification::AddKeyword(_)))) =>
            {
                let Effect::Modify { mods, .. } = inner.as_ref() else {
                    return self.gap("enters with keywords");
                };
                let k: Vec<String> = mods
                    .iter()
                    .filter_map(|m| match m {
                        Modification::AddKeyword(k) => Some(self.keyword_lower(k)),
                        _ => None,
                    })
                    .collect();
                format!("enters with {}", join_list(&k, "and"))
            }
            Effect::Seq(v) => {
                let mut parts: Vec<String> = v.iter().map(|x| self.as_enters_vp(x)).collect();
                // "enters with two +1/+1 counters on it and with haste": one verb.
                for i in (1..parts.len()).rev() {
                    if parts[i].starts_with("enters with ") && parts[i - 1].starts_with("enters ") {
                        parts[i] = parts[i]["enters ".len()..].to_string();
                    }
                }
                join_list(&parts, "and")
            }
            other => {
                let s = self.effect(other);
                format!("enters and {s}")
            }
        }
    }

    /// "exile it instead", "prevent that damage", ...
    fn replacement_then(&mut self, a: &ReplacementAction, it: &str) -> String {
        use ReplacementAction as A;
        let it = if it.is_empty() { "it" } else { it };
        match a {
            A::MoveInstead(d) => {
                if d.zone == ZoneKind::Exile {
                    return format!("exile {it} instead");
                }
                if d.zone == ZoneKind::Library && d.position == LibraryPosition::Shuffled {
                    return format!("shuffle {it} into its owner's library instead");
                }
                let dest = self.destination_phrase(d, false, false);
                let verb = if d.zone == ZoneKind::Hand {
                    "return"
                } else {
                    "put"
                };
                format!("{verb} {it} {dest} instead")
            }
            A::Instead(e) => {
                let e = self.effect(e);
                format!("{e} instead")
            }
            A::Also(e) => {
                let e = self.effect(e);
                format!("{e} as well")
            }
            A::Prevent => "prevent that event".into(),
            A::PreventAmount(v) | A::PreventPortion(v) => {
                let v = self.value(v);
                format!("prevent {v} of that damage")
            }
            A::Multiply(2) => "it's doubled instead".into(),
            A::Multiply(n) => format!("it's multiplied by {n} instead"),
            A::Add(v) => {
                let v = self.value(v);
                format!("that much plus {v} instead")
            }
            A::Subtract(v) => {
                let v = self.value(v);
                format!("that much minus {v} instead")
            }
            A::Regenerate => format!("regenerate {it}"),
            A::PlusTokens { spec, count } => {
                let (d, tail) = self.token_desc(spec);
                let c = match count {
                    Value::EventAmount => "that many".to_string(),
                    Value::Const(1) => with_article(&d),
                    other => self.value(other),
                };
                let tokens = if matches!(count, Value::Const(1)) {
                    format!("{c} token{tail}")
                } else {
                    format!("{c} {d} tokens{tail}")
                };
                format!("those tokens plus {tokens} are created instead")
            }
            A::Redirect(sel) => {
                let t = self.sel(sel, Case::Obj);
                format!("that damage is dealt to {t} instead")
            }
            A::RedirectNext(sel, v) => {
                let t = self.sel(sel, Case::Obj);
                let v = self.value(v);
                format!("{v} of that damage is dealt to {t} instead")
            }
            A::PreventAndThen(amount, e) => {
                let p = match amount {
                    None => "prevent that damage".to_string(),
                    Some(v) => {
                        let v = self.value(v);
                        format!("prevent {v} of that damage")
                    }
                };
                let e = self.effect(e);
                format!("{p}. {e}")
            }
            A::LifeFloor(v) => {
                let v = self.value(v);
                format!("{it} loses life only down to {v}")
            }
            A::ManaTypeInstead(t) => format!("it produces {} instead", mana_symbol(*t)),
            // Enters-the-battlefield replacements are worded with their event
            // (`as_enters`); here they have no event to go with.
            A::EnterTapped
            | A::EnterUntapped
            | A::EnterWithCounters(..)
            | A::AsEnters(_)
            | A::EnterAsCopy { .. }
            | A::EnterUnderControl(_)
            | A::EnterTransformed => self.gap("enters replacement without an enters event"),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn damage_replacement(
        &mut self,
        source: &Filter,
        to_players: &Option<PlayerFilter>,
        to_objects: &Option<Filter>,
        combat_only: bool,
        noncombat: bool,
        action: &ReplacementAction,
    ) -> String {
        use ReplacementAction as A;
        let kind = if combat_only {
            "combat damage"
        } else if noncombat {
            "noncombat damage"
        } else {
            "damage"
        };
        let mut to: Vec<String> = Vec::new();
        // "to you and creatures you control": you first.
        if let Some(PlayerFilter::You) = to_players {
            to.push("you".into());
        }
        // Damage to anything: no recipient phrase, or "to a permanent or player" (what
        // damage can be dealt to, CR 120.1 — a battle is a permanent).
        let anything = matches!(
            (to_objects, to_players),
            (Some(Filter::Any), Some(PlayerFilter::Any))
        );
        let (to_objects, to_players) = match (to_objects, to_players) {
            (Some(Filter::Any), Some(PlayerFilter::Any)) => (&None, &None),
            other => other,
        };
        // "to an opponent or a permanent an opponent controls": players, then objects.
        if let Some(p) = to_players
            .as_ref()
            .filter(|p| !matches!(p, PlayerFilter::You))
        {
            to.push(self.player_filter_object(p));
        }
        if let Some(o) = to_objects {
            to.push(match o {
                Filter::Source => self.me(),
                Filter::AttachedToSource => self.attached_noun(),
                other => self.noun_det(other, Det::A),
            });
        }
        let src = match source {
            Filter::Any => None,
            Filter::Source => Some(self.me()),
            other => {
                let saved = self.default_head;
                self.default_head = Some("source");
                let s = self.noun_det(other, Det::A);
                self.default_head = saved;
                Some(s)
            }
        };
        let to_s = if anything {
            " {opt:to a permanent or player}".to_string()
        } else if to.is_empty() {
            String::new()
        } else {
            // "to you and creatures you control" / "to you or a creature you control".
            format!(" to {}", join_list(&to, "{alt:or|and}"))
        };
        match action {
            A::Prevent => {
                let by = match &src {
                    Some(s) => format!(" by {s}"),
                    None => String::new(),
                };
                format!("prevent all {kind} that would be dealt{to_s}{by}")
            }
            // Damage is always dealt by a source (CR 120.1): "if a source would deal
            // damage to you" is "if damage would be dealt to you".
            A::PreventAmount(v) => {
                let v = self.value(v);
                match &src {
                    Some(s) => {
                        format!("if {s} would deal {kind}{to_s}, prevent {v} of that damage")
                    }
                    None => format!("if {{alt:{kind} would be dealt{to_s}|a source would deal {kind}{to_s}}}, prevent {v} of that damage"),
                }
            }
            A::PreventAndThen(amount, e) => {
                let e = self.effect(e);
                let p = match amount {
                    None => "prevent that damage".to_string(),
                    Some(v) => {
                        let v = self.value(v);
                        format!("prevent {v} of that damage")
                    }
                };
                match &src {
                    Some(s) => format!("if {s} would deal {kind}{to_s}, {p}. {e}"),
                    None => format!("if {{alt:{kind} would be dealt{to_s}|a source would deal {kind}{to_s}}}, {p}. {e}"),
                }
            }
            // "The next 1 damage that would be dealt to target creature this turn is dealt
            // to ~ instead."
            A::RedirectNext(sel, v) if src.is_none() => {
                let t = self.sel(sel, Case::Obj);
                let v = self.value(v);
                format!("the next {v} {kind} that would be dealt{to_s} is dealt to {t} instead")
            }
            // "All damage that would be dealt to you is dealt to ~ instead."
            A::Redirect(sel) if src.is_none() => {
                let t = self.sel(sel, Case::Obj);
                format!("all {kind} that would be dealt{to_s} is dealt to {t} instead")
            }
            other => {
                let s = src.unwrap_or_else(|| "a source".into());
                // "it deals double that damage to that player or permanent instead".
                let both = to_players.is_some() && to_objects.is_some();
                let to_that = if anything {
                    " {opt:to that permanent or player}"
                } else if both {
                    " {opt:to that player or permanent}"
                } else if to_players.is_some() {
                    " {opt:to that player}"
                } else {
                    ""
                };
                let then = match other {
                    A::Multiply(2) => format!("it deals double that damage{to_that} instead"),
                    A::Multiply(3) => format!("it deals triple that damage{to_that} instead"),
                    A::Add(v) => {
                        let v = self.value(v);
                        let to_that = if anything {
                            " {opt:to that permanent or player}"
                        } else {
                            to_that
                        };
                        format!("it deals that much damage plus {v}{to_that} instead")
                    }
                    A::Subtract(v) => {
                        let v = self.value(v);
                        format!("it deals that much damage minus {v} instead")
                    }
                    A::Redirect(sel) => {
                        let t = self.sel(sel, Case::Obj);
                        format!("that damage is dealt to {t} instead")
                    }
                    A::RedirectNext(sel, v) => {
                        let t = self.sel(sel, Case::Obj);
                        let v = self.value(v);
                        format!("the next {v} of that damage is dealt to {t} instead")
                    }
                    other => self.replacement_then(other, "it"),
                };
                format!("if {s} would deal {kind}{to_s}, {then}")
            }
        }
    }

    fn zone_any(&self, z: ZoneKind) -> String {
        match z {
            ZoneKind::Exile => "exile".into(),
            ZoneKind::Battlefield => "the battlefield".into(),
            other => with_article(zone_word(other)),
        }
    }

    fn zone_src(&self, z: ZoneKind) -> String {
        match z {
            ZoneKind::Battlefield => "the battlefield".into(),
            ZoneKind::Exile => "exile".into(),
            ZoneKind::Stack => "the stack".into(),
            other => with_article(zone_word(other)),
        }
    }

    fn deck_condition(&mut self, dc: &crate::start::DeckCondition) -> String {
        use crate::start::DeckCondition as D;
        match dc {
            D::Each { each, must } => {
                let e = self.card_noun(each);
                // What the card must be beyond being one of `each`.
                let parts = |f: &Filter| match f {
                    Filter::And(v) => v.clone(),
                    other => vec![other.clone()],
                };
                let have = parts(each);
                let rest: Vec<Filter> = parts(must)
                    .into_iter()
                    .filter(|x| {
                        !matches!(x, Filter::Card)
                            && !have.iter().any(|h| format!("{h:?}") == format!("{x:?}"))
                    })
                    .collect();
                let m = match rest.as_slice() {
                    // "has mana value 2 or less".
                    [Filter::ManaValue(c, v)] => {
                        let v = self.value(v);
                        format!("has mana value {}", super::nouns::cmp_phrase(*c, &v))
                    }
                    // "is a Cat, Elemental, Nightmare, Dinosaur, or Beast card".
                    _ => {
                        let n = self.card_noun(&Filter::and(rest.clone()));
                        format!("is {}", with_article(&n))
                    }
                };
                format!("each {e} in your starting deck {m}")
            }
            D::DifferentNames { each } => {
                let e = self.card_noun(each);
                format!("each {e} in your starting deck has a different name")
            }
            D::ShareACardType { each } => {
                let e = self.card_noun(each);
                format!("each {e} in your starting deck shares a card type")
            }
            D::MoreThanMinimumSize(n) => format!(
                "your starting deck contains at least {} cards more than the minimum deck size",
                number_word(*n as i32)
            ),
            D::NoRepeatedManaSymbol => "no card in your starting deck has more than one of the same mana symbol in its mana cost".into(),
        }
    }

    fn dice_static(&mut self, d: &crate::dice::DiceStatic) -> String {
        use crate::dice::DiceStatic as D;
        match d {
            D::ExtraDie { who } => {
                let w = self.rel_subject(*who);
                format!("if {w} would roll one or more dice, instead {w} roll that many dice plus one and ignore the lowest roll")
            }
            D::ExtraCoin { who } => {
                let w = self.rel_subject(*who);
                format!("if {w} would flip a coin, instead flip two coins and ignore one")
            }
            D::FirstFlipsWin { who } => {
                let w = self.rel_subject(*who);
                format!("the first time {w} flip one or more coins each turn, those coins come up heads and {w} win those flips")
            }
            D::Modifier(m) => {
                let c = self.cost_as_payment(&m.cost);
                let what = match m.kind {
                    crate::dice::ModifierKind::Reroll => "reroll it".to_string(),
                    crate::dice::ModifierKind::Add(n) if n >= 0 => format!("add {n} to the result"),
                    crate::dice::ModifierKind::Add(n) => format!("subtract {} from the result", -n),
                };
                format!("whenever you roll a die, you may {c}. If you do, {what}")
            }
        }
    }

    #[allow(dead_code)]
    fn unused(&mut self) {
        let _ = split_controller;
        let _ = join_words;
        let _ = third_person;
    }
}

/// The sentence of a rule a spell or ability states about its own cost (see
/// `payment_rules.rs`).
pub(crate) fn cost_rule_text(r: &CostRule) -> String {
    match r {
        CostRule::XAtLeast(1) => "X can't be 0".into(),
        CostRule::XAtLeast(n) => format!("X can't be less than {n}"),
        CostRule::XOnlyColors { colors, distinct } => {
            let words: Vec<&str> = Color::ALL
                .iter()
                .filter(|c| colors.contains(**c))
                .map(|c| c.word())
                .collect();
            let mut s = if *colors == ColorSet::ALL {
                "spend only colored mana on X".to_string()
            } else {
                format!("spend only {} mana on X", words.join(" and/or "))
            };
            if *distinct {
                s.push_str(". No more than one mana of each color may be spent this way");
            }
            s
        }
        CostRule::NoMana => "you can't spend mana to cast ~".into(),
    }
}

/// Whether `e` modifies how the object enters anywhere ("enters tapped", "enters with ...
/// counters"), or is a choice of how it enters.
fn mentions_entry_modification(e: &Effect) -> bool {
    let d = format!("{e:?}");
    d.contains("Enter") || d.contains("Choose") || d.contains("Custom")
}

/// `InZone(z) or CastFrom(z)` (a card about to be cast from a zone, or a spell cast from
/// it) as `CastFrom(z)`.
fn cast_from_zone_only(f: &Filter) -> Filter {
    match f {
        Filter::Or(v) => match v.as_slice() {
            [Filter::InZone(a), Filter::CastFrom(b)] if a == b => Filter::CastFrom(*a),
            _ => Filter::Or(v.iter().map(cast_from_zone_only).collect()),
        },
        Filter::And(v) => Filter::And(v.iter().map(cast_from_zone_only).collect()),
        Filter::Not(x) => Filter::Not(Box::new(cast_from_zone_only(x))),
        other => other.clone(),
    }
}
