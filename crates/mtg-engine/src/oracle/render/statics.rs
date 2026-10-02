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

fn filter_has(f: &Filter, p: &dyn Fn(&Filter) -> bool) -> bool {
    p(f) || match f {
        Filter::And(v) | Filter::Or(v) => v.iter().any(|x| filter_has(x, p)),
        Filter::Not(x) => filter_has(x, p),
        _ => false,
    }
}

impl Renderer<'_> {
    pub(crate) fn static_ability(&mut self, s: &StaticAbility) -> String {
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
            StaticEffect::FlashPermission { who, what } => {
                let w = self.rel_subject(*who);
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
                let b = self.body(body);
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
        }
    }

    fn cast_only_condition(&mut self, c: &Condition) -> String {
        match c {
            Condition::CombatTiming(ct) => self.combat_timing(ct),
            Condition::YourTurn => "during your turn".into(),
            Condition::NotYourTurn => "during an opponent's turn".into(),
            Condition::Phase(PhaseCond::Combat) => "during combat".into(),
            Condition::Phase(PhaseCond::Upkeep) => "during an opponent's upkeep".into(),
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
            PlayerModification::Custom(name) => self.custom_player_mod(name, &subj, &poss),
        }
    }

    fn spell_noun_plural(&mut self, f: &Filter) -> String {
        if matches!(f, Filter::Source) {
            return self.me();
        }
        // "Red spells and white spells you cast cost {1} less".
        let saved = self.alt_and;
        self.alt_and = true;
        let n = self.noun(f, Num::Many);
        self.alt_and = saved;
        if n.contains("spell") {
            n
        } else if n == "permanents" || n == "cards" {
            "spells".into()
        } else {
            format!("{} spells", n.trim_end_matches('s'))
        }
    }

    fn play_permission(&mut self, pp: &PlayPermission) -> String {
        let who = self.rel_subject(pp.who);
        let p = if who == "you" { "your" } else { "their" };
        let verb = match (pp.lands, pp.spells) {
            (true, true) => {
                if matches!(pp.what, Filter::Any) {
                    "play lands and cast spells".to_string()
                } else {
                    let n = self.noun(&pp.what, Num::Many);
                    format!("play {n}")
                }
            }
            (true, false) => "play lands".to_string(),
            (false, _) => {
                let s = self.spell_noun_plural(&pp.what);
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
        s
    }

    /// What a cost modifier applies to ("spells you cast", "activated abilities of
    /// creatures").
    fn cost_target(&mut self, t: &CostTarget) -> String {
        match t {
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
        }
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
        let target = self.cost_target(&cm.applies_to);
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
                let mut s = format!("{target}{who} {costs} {mana} less {act}");
                if *colored_only {
                    s.push_str(". This effect reduces only the amount of colored mana you pay");
                }
                s
            }
            CostChange::AdditionalCost(c) => {
                let c = self.cost_as_payment(c);
                if matches!(cm.applies_to, CostTarget::ThisSpell) {
                    format!("as an additional cost to cast ~, {c}")
                } else {
                    format!("as an additional cost to cast {target}, {c}")
                }
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
                format!("as an additional cost to cast ~, you may {c}")
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
                let b = subj(self, blocker);
                let a = self.noun_det(attacker, Det::A);
                format!("{b} blocks {a} this combat if able")
            }
            Restriction::AttackCost {
                attackers,
                defender,
                planeswalkers,
                cost,
            } => {
                let a = subj(self, attackers);
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
                    return format!(
                        "{} activated abilities can't be activated{m}",
                        nouns::possessive(&s)
                    );
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
            Restriction::CantEnterBattlefield(f) | Restriction::CantEnter(f) => {
                format!("{} can't enter the battlefield", subj(self, f))
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
                let x = self.noun(what, Num::Many);
                format!(
                    "{w} can't untap more than {} {x} during their untap steps",
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
            Restriction::SourceDamageCantBePrevented(f) => {
                if matches!(f, Filter::Source) {
                    "damage that would be dealt by ~ can't be prevented".into()
                } else {
                    let n = self.noun(f, Num::Many);
                    format!("damage that would be dealt by {n} can't be prevented")
                }
            }
            Restriction::CantTransform(f) => format!("{} can't transform", subj(self, f)),
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
        let s = self.replacement_inner(def, uses);
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
                let it = if subj == "~" { "it" } else { "it" };
                match action {
                    A::EnterTapped if subj != "~" && subj != "~it" => {
                        format!("{subj} enter tapped")
                    }
                    A::EnterTapped => format!("{subj} enters tapped"),
                    A::EnterWithCounters(k, n) => {
                        let (c, w) = self.counted(n, &counter_name(k));
                        format!("{subj} enters with {c} on {it}{}", w.unwrap_or_default())
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
                    other => self.gap(format!("enters replacement {other:?}")),
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
                    (None, Some(t)) => {
                        format!("be put into {} from anywhere", self.zone_any(*t))
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
                let w = self.player_filter_subject(p);
                let w = if w == "players" { "a player".into() } else { w };
                let then = self.replacement_then(action, "");
                format!("if {w} would draw a card, {then}")
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
                let then = match action {
                    A::Multiply(2) => {
                        format!("twice that many {} are put on it instead", plural(&k))
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
            (E::PutCountersBy { by, kind }, action) => {
                let w = self.rel_subject(*by);
                let k = match kind {
                    Some(k) => counter_name(k),
                    None => "counter".into(),
                };
                let then = match action {
                    A::Multiply(2) => {
                        format!("{w} put twice that many {} on it instead", plural(&k))
                    }
                    A::Add(v) => {
                        let v = match v {
                            Value::Const(1) => "one".to_string(),
                            other => self.value(other),
                        };
                        format!("{w} put that many plus {v} {} on it instead", plural(&k))
                    }
                    other => self.replacement_then(other, "it"),
                };
                format!(
                    "if {w} would put one or more {} on a permanent, {then}",
                    plural(&k)
                )
            }
            (E::CreateTokens(p), action) => {
                let w = self.player_filter_subject(p);
                let w = if w == "players" { "a player".into() } else { w };
                let poss = nouns::possessive(&w);
                match action {
                    A::Multiply(2) => format!(
                        "if an effect would create one or more tokens under {poss} control, it creates twice that many of those tokens instead"
                    ),
                    A::PlusTokens { spec, count } => {
                        let (d, tail) = self.token_desc(spec);
                        let c = match count {
                            Value::EventAmount => "that many".to_string(),
                            other => self.value(other),
                        };
                        format!("if one or more tokens would be created under {poss} control, those tokens plus {c} {d} tokens{tail} are created instead")
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
        match e {
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
            Effect::Seq(v) => {
                let parts: Vec<String> = v.iter().map(|x| self.as_enters_vp(x)).collect();
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
            A::PreventAmount(v) => {
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
            other => self.gap(format!(
                "replacement action {:?}",
                std::mem::discriminant(other)
            )),
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
        // Damage to anything: no recipient phrase.
        let (to_objects, to_players) = match (to_objects, to_players) {
            (Some(Filter::Any), Some(PlayerFilter::Any)) => (&None, &None),
            other => other,
        };
        if let Some(o) = to_objects {
            to.push(match o {
                Filter::Source => self.me(),
                Filter::AttachedToSource => self.attached_noun(),
                other => self.noun_det(other, Det::A),
            });
        }
        if let Some(p) = to_players
            .as_ref()
            .filter(|p| !matches!(p, PlayerFilter::You))
        {
            to.push(self.player_filter_object(p));
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
        let to_s = if to.is_empty() {
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
            A::PreventAmount(v) => {
                let v = self.value(v);
                match &src {
                    Some(s) => {
                        format!("if {s} would deal {kind}{to_s}, prevent {v} of that damage")
                    }
                    None => format!("if {kind} would be dealt{to_s}, prevent {v} of that damage"),
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
                    None => format!("if {kind} would be dealt{to_s}, {p}. {e}"),
                }
            }
            other => {
                let s = src.unwrap_or_else(|| "a source".into());
                let then = match other {
                    A::Multiply(2) => "it deals double that damage instead".to_string(),
                    A::Multiply(3) => "it deals triple that damage instead".to_string(),
                    A::Add(v) => {
                        let v = self.value(v);
                        format!("it deals that much damage plus {v} instead")
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
                let e = self.noun(each, Num::One);
                let m = self.is_predicate(must, false);
                let m = m.strip_prefix("is ").map(|x| x.to_string()).unwrap_or(m);
                format!("each {e} in your starting deck has {m}")
            }
            D::DifferentNames { each } => {
                let e = self.noun(each, Num::One);
                format!("each {e} in your starting deck has a different name")
            }
            D::ShareACardType { each } => {
                let e = self.noun(each, Num::One);
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
