//! Instructions written together because a later one uses what an earlier one chose:
//! "The next time a red source of your choice would deal damage to you this turn, prevent
//! that damage." (a chosen source, then a prevention shield for it, CR 615.7).

use super::effects::join_words;
use super::players::Case;
use super::*;

impl Renderer<'_> {
    /// At `v[i]` of a sequence: such instructions, how many they are and their text.
    pub(crate) fn tail_seq_part(&mut self, v: &[Effect], i: usize) -> Option<(usize, String)> {
        self.chosen_source_prevention(v, i)
            .map(|s| (2, s))
            .or_else(|| self.does_the_same(v, i).map(|s| (2, s)))
            .or_else(|| self.any_player_may(v, i).map(|s| (2, s)))
    }

    /// "Any player may pay 5 life. If a player does, counter ~.": each player in turn may
    /// pay; the first payment does it (a flag remembers that it's done).
    fn any_player_may(&mut self, v: &[Effect], i: usize) -> Option<String> {
        let (
            Effect::StoreValue {
                var,
                value: Value::Const(0),
            },
            Some(Effect::ForEachPlayer {
                who: PlayerRef::EachPlayer,
                effect,
            }),
        ) = (&v[i], v.get(i + 1))
        else {
            return None;
        };
        let Effect::PayOptional {
            who: PlayerRef::Iterated,
            cost,
            then,
            otherwise,
        } = effect.as_ref()
        else {
            return None;
        };
        let Effect::If {
            cond: Condition::Compare(Value::Var(flag), Cmp::Eq, Value::Const(0)),
            then: inner,
            otherwise: inner_else,
        } = then.as_ref()
        else {
            return None;
        };
        if !matches!(otherwise.as_ref(), Effect::Noop)
            || !matches!(inner_else.as_ref(), Effect::Noop)
            || flag != var
        {
            return None;
        }
        let Effect::Seq(steps) = inner.as_ref() else {
            return None;
        };
        let [Effect::StoreValue {
            var: set,
            value: Value::Const(1),
        }, rest @ ..] = steps.as_slice()
        else {
            return None;
        };
        if set != var || rest.is_empty() {
            return None;
        }
        let c = self.cost_as_payment(cost).replace(" your ", " their ");
        let r = self.seq(rest);
        Some(format!("any player may {c}. If a player does, {r}"))
    }

    /// "Create a Gold token. Each opponent attacking that player does the same.": each of
    /// those players does what you did (as if they were "you").
    fn does_the_same(&mut self, v: &[Effect], i: usize) -> Option<String> {
        let (
            first,
            Some(
                second @ Effect::ForEachPlayer {
                    who: PlayerRef::Each(pf),
                    effect,
                },
            ),
        ) = (&v[i], v.get(i + 1))
        else {
            return None;
        };
        let Effect::AsPlayer {
            who: PlayerRef::Iterated,
            effect: inner,
        } = effect.as_ref()
        else {
            return None;
        };
        if format!("{first:?}") != format!("{inner:?}") || matches!(first, Effect::Seq(_)) {
            return None;
        }
        let a = self.effect(first);
        let b = self.effect(second);
        let attacking = format!(
            "{:?}",
            PlayerFilter::And(vec![
                PlayerFilter::Opponent,
                PlayerFilter::Controls(
                    Box::new(Filter::Custom("attacking the event's player".into())),
                    Cmp::Ge,
                    Box::new(Value::Const(1)),
                ),
            ])
        );
        let subj = if format!("{pf:?}") == attacking {
            "each opponent attacking that player".to_string()
        } else {
            self.player(&PlayerRef::Each(pf.clone()), Case::Subj)
        };
        if a.is_empty() || b.is_empty() || b.contains('\n') {
            return None;
        }
        Some(format!("{a}. {{alt:{b}|{subj} does the same}}"))
    }

    /// "Choose a [quality] source. Prevent the damage it would deal": "the next time a
    /// [quality] source of your choice would deal damage to you this turn, prevent that
    /// damage" (one use, CR 615.7), "prevent all damage a [quality] source of your choice
    /// would deal this turn".
    fn chosen_source_prevention(&mut self, v: &[Effect], i: usize) -> Option<String> {
        let (
            Effect::ChooseSource {
                who: PlayerRef::You,
                filter,
                var,
            },
            Some(Effect::AddReplacement {
                def,
                duration,
                uses,
            }),
        ) = (&v[i], v.get(i + 1))
        else {
            return None;
        };
        let ReplacementEvent::Damage {
            source,
            to_players,
            to_objects,
            combat_only,
        } = &def.event
        else {
            return None;
        };
        if !matches!(def.action, ReplacementAction::Prevent) || def.optional {
            return None;
        }
        // The damage's source is the chosen one (and has the quality it was chosen with).
        let chosen = format!("{:?}", Filter::In(Box::new(Sel::Var(*var))));
        let parts: Vec<&Filter> = match source {
            Filter::And(v) => v.iter().collect(),
            f => vec![f],
        };
        if !parts.iter().any(|f| format!("{f:?}") == chosen) {
            return None;
        }
        let rest: Vec<Filter> = parts
            .into_iter()
            .filter(|f| format!("{f:?}") != chosen)
            .cloned()
            .collect();
        let quality = Filter::and(rest);
        if format!("{quality:?}") != format!("{filter:?}")
            && !(rest_is_any(&quality) && matches!(filter, Filter::Any))
        {
            return None;
        }
        let to = match (to_players, to_objects) {
            (Some(PlayerFilter::You), None) => " to you",
            (Some(PlayerFilter::Any), Some(Filter::Any)) => "",
            _ => return None,
        };
        let src = match filter {
            Filter::Any => "a source of your choice".to_string(),
            f => {
                let n = self.noun(f, Num::One);
                let adj = n.strip_suffix("permanent")?.trim_end();
                format!("{} source of your choice", with_article(adj))
            }
        };
        let when = match duration {
            Duration::EndOfTurn | Duration::ThisTurn => " this turn",
            _ => return None,
        };
        let damage = if *combat_only {
            "combat damage"
        } else {
            "damage"
        };
        Some(match uses {
            Some(1) => {
                format!("the next time {src} would deal {damage}{to}{when}, prevent that damage")
            }
            None => format!("prevent all {damage} {src} would deal{to}{when}"),
            Some(_) => return None,
        })
    }
}

fn rest_is_any(f: &Filter) -> bool {
    match f {
        Filter::Any => true,
        Filter::And(v) => v.is_empty(),
        _ => false,
    }
}

impl Renderer<'_> {
    /// What an earlier instruction did to the objects `s` holds, and whether they're
    /// cards (not permanents): "destroyed", "discarded".
    pub(crate) fn this_way_of(&self, s: &Sel) -> Option<(&'static str, bool, Option<CardType>)> {
        let Sel::Var(v) = s else {
            return None;
        };
        let (_, verb, t) = self.this_way.iter().rev().find(|(x, _, _)| x == v)?;
        let card = matches!(*verb, "discarded" | "milled" | "exiled" | "drawn");
        Some((verb, card, *t))
    }
}

/// The variables `prev` (earlier instructions of a sequence, in order) leave holding the
/// objects one of them acted on, and what it did: a destroy instruction leaves the
/// destroyed permanents in [`vars::IT`] (not those that were regenerated or
/// indestructible), a discard instruction the discarded cards in
/// [`crate::discard_rules::DISCARDED`], a draw instruction the drawn cards in
/// [`vars::REVEALED`].
pub(crate) fn this_way(prev: &[Effect]) -> Vec<(Var, &'static str, Option<CardType>)> {
    let mut out: Vec<(Var, &'static str, Option<CardType>)> = Vec::new();
    for e in prev {
        note_this_way(e, &mut out);
    }
    out
}

fn note_this_way(e: &Effect, out: &mut Vec<(Var, &'static str, Option<CardType>)>) {
    let set = |out: &mut Vec<(Var, &'static str, Option<CardType>)>,
               v: Var,
               verb: Option<(&'static str, Option<CardType>)>| {
        out.retain(|(x, _, _)| *x != v);
        if let Some((verb, t)) = verb {
            out.push((v, verb, t));
        }
    };
    // The one card type the objects acted on all have ("destroy all creatures": the
    // creatures destroyed this way).
    let ty = |s: &Sel| -> Option<CardType> {
        let Sel::All(f) = s else { return None };
        let parts: Vec<&Filter> = match f {
            Filter::And(v) => v.iter().collect(),
            f => vec![f],
        };
        let types: Vec<CardType> = parts
            .iter()
            .filter_map(|f| match f {
                Filter::Type(t) => Some(*t),
                _ => None,
            })
            .collect();
        match types.as_slice() {
            [t] => Some(*t),
            _ => None,
        }
    };
    match e {
        Effect::Destroy { what, .. } => set(out, vars::IT, Some(("destroyed", ty(what)))),
        Effect::Exile { what, .. } => set(out, vars::IT, Some(("exiled", ty(what)))),
        Effect::Sacrifice { .. } | Effect::SacrificeObjects { .. } => {
            set(out, vars::IT, Some(("sacrificed", None)))
        }
        Effect::Move { to, what } if to.zone == ZoneKind::Hand => {
            set(out, vars::IT, Some(("returned", ty(what))))
        }
        Effect::Tap { what } => set(out, vars::TAPPED, Some(("tapped", ty(what)))),
        Effect::Discard { .. } | Effect::DiscardHand { .. } => set(
            out,
            crate::discard_rules::DISCARDED,
            Some(("discarded", None)),
        ),
        Effect::Draw { .. } => set(out, vars::REVEALED, Some(("drawn", None))),
        // "You may discard your hand. If you do, draw that many cards.": what the
        // optional instruction did, if it was done.
        Effect::May { effect, .. } => note_this_way(effect, out),
        Effect::Store {
            var,
            sel: Sel::Var(src),
        } => {
            let verb = out
                .iter()
                .find(|(x, _, _)| x == src)
                .map(|(_, v, t)| (*v, *t));
            set(out, *var, verb);
        }
        Effect::Store { var, .. } => set(out, *var, None),
        // Another instruction acting on objects leaves something else in "it".
        Effect::Move { .. } | Effect::Mill { .. } => set(out, vars::IT, None),
        _ => {}
    }
}

/// Instructions a player chooses among as the effect happens (CR 608.2d), written as one
/// instruction with the alternatives joined by "or": "add {B}{B} or {G}{G}", "~ gets
/// +1/-1 or -1/+1 until end of turn", "~ gains your choice of vigilance, lifelink, or haste
/// until end of turn". The words all of them start and end with are written once.
pub(crate) fn or_form(options: &[String]) -> Option<String> {
    if options.len() < 2
        || options.iter().any(|o| {
            o.is_empty() || o.contains(['\n', '|']) || o.trim_end_matches('.').contains(". ")
        })
    {
        return None;
    }
    let words: Vec<Vec<&str>> = options
        .iter()
        .map(|o| o.trim_end_matches('.').split(' ').collect())
        .collect();
    let same = |a: &str, b: &str| {
        let selfish = |w: &str| matches!(w, "~" | "~it" | "it");
        a == b || (selfish(a) && selfish(b))
    };
    let min = words.iter().map(Vec::len).min()?;
    let mut pre = 0;
    while pre < min && words.iter().all(|w| same(w[pre], words[0][pre])) {
        pre += 1;
    }
    let mut suf = 0;
    while suf < min - pre
        && words
            .iter()
            .all(|w| same(w[w.len() - 1 - suf], words[0][words[0].len() - 1 - suf]))
    {
        suf += 1;
    }
    let middles: Vec<String> = words
        .iter()
        .map(|w| w[pre..w.len() - suf].join(" "))
        .collect();
    if middles.iter().any(String::is_empty) {
        return None;
    }
    let head = words[0][..pre].join(" ");
    let tail = words[0][words[0].len() - suf..].join(" ");
    // "gains your choice of flying or vigilance": a choice among objects of a verb.
    let choice = if pre > 0 { "{opt:your choice of} " } else { "" };
    let list = super::join_list(&middles, "or");
    Some(
        [head, format!("{choice}{list}"), tail]
            .into_iter()
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join(" "),
    )
}

impl Renderer<'_> {
    /// "{3}{W}: Level 2": a Class's level ability (CR 716.2a: activate only as a sorcery
    /// and only if the Class's level is one less).
    pub(crate) fn class_level_up(&mut self, a: &ActivatedAbility) -> Option<String> {
        let Effect::SetClassLevel { level } = &a.body.effect else {
            return None;
        };
        let one_less = matches!(&a.condition,
            Some(Condition::Compare(Value::ClassLevel, Cmp::Eq, Value::Const(n))) if *n + 1 == *level as i32);
        if !matches!(a.timing, ActivationTiming::Sorcery)
            || !one_less
            || a.max_per_turn.is_some()
            || !a.body.targets.is_empty()
        {
            return None;
        }
        let cost = self.cost(&a.cost);
        Some(format!("{cost}: Level {level}"))
    }

    /// A Class's level section: the abilities it has while its level is that level or
    /// greater (CR 716.2a), written after its level ability.
    pub(crate) fn class_level_abilities(&mut self, s: &StaticAbility) -> Option<String> {
        if !matches!(
            &s.condition,
            Some(Condition::Compare(
                Value::ClassLevel,
                Cmp::Ge,
                Value::Const(_)
            ))
        ) {
            return None;
        }
        let StaticEffect::Continuous {
            affected: Filter::Source,
            mods,
        } = &s.effect
        else {
            return None;
        };
        let abilities: Vec<&Ability> = mods
            .iter()
            .map(|m| match m {
                Modification::AddAbility(a) => Some(a),
                _ => None,
            })
            .collect::<Option<_>>()?;
        if abilities.is_empty() {
            return None;
        }
        let texts: Vec<String> = abilities.iter().map(|a| self.nested_ability(a)).collect();
        Some(texts.join("\n"))
    }
}

impl Renderer<'_> {
    /// A leveler's level symbol: "LEVEL 2-3 / 3/3 / Flying" (CR 711.2a: as long as it has
    /// at least 2 and no more than 3 level counters, it has base power and toughness 3/3
    /// and has flying; CR 711.2b: "LEVEL 4+").
    pub(crate) fn level_symbol(&mut self, s: &StaticAbility) -> Option<String> {
        let level = |c: &Condition| match c {
            Condition::Compare(Value::CountersOn(sel, Some(k)), cmp, Value::Const(n))
                if k == "level" && matches!(sel.as_ref(), Sel::This) =>
            {
                Some((*cmp, *n))
            }
            _ => None,
        };
        let range = match s.condition.as_ref()? {
            Condition::And(v) => match v.as_slice() {
                [a, b] => match (level(a)?, level(b)?) {
                    ((Cmp::Ge, lo), (Cmp::Le, hi)) => format!("{lo}-{hi}"),
                    _ => return None,
                },
                [a] => match level(a)? {
                    (Cmp::Ge, lo) => format!("{lo}+"),
                    _ => return None,
                },
                _ => return None,
            },
            c => match level(c)? {
                (Cmp::Ge, lo) => format!("{lo}+"),
                _ => return None,
            },
        };
        let StaticEffect::Continuous {
            affected: Filter::Source,
            mods,
        } = &s.effect
        else {
            return None;
        };
        let mut lines = vec![format!("LEVEL {range}")];
        let mut pt = None;
        let mut abilities = Vec::new();
        for m in mods {
            match m {
                Modification::SetPT(Some(Value::Const(p)), Some(Value::Const(t)))
                    if pt.is_none() =>
                {
                    pt = Some(format!("{p}/{t}"))
                }
                Modification::AddAbility(a) => abilities.push(a.clone()),
                _ => return None,
            }
        }
        lines.push(pt?);
        for a in &abilities {
            lines.push(self.nested_ability(a));
        }
        Some(lines.join("\n"))
    }
}

impl Renderer<'_> {
    /// A station symbol: "8+ | Flying, trample" (CR 721.2a: as long as it has 8 or more
    /// charge counters, it has those abilities; CR 721.2b: with a power/toughness box, it's
    /// also a creature with that base power and toughness, which the box shows).
    pub(crate) fn station_symbol(&mut self, s: &StaticAbility) -> Option<String> {
        if !self
            .info
            .subtypes
            .iter()
            .any(|t| t == "Spacecraft" || t == "Planet")
        {
            return None;
        }
        let Some(Condition::Compare(Value::CountersOn(sel, Some(k)), Cmp::Ge, Value::Const(n))) =
            &s.condition
        else {
            return None;
        };
        if k != "charge" || !matches!(sel.as_ref(), Sel::This) {
            return None;
        }
        let StaticEffect::Continuous {
            affected: Filter::Source,
            mods,
        } = &s.effect
        else {
            return None;
        };
        let mut creature = false;
        let mut pt = false;
        let mut abilities = Vec::new();
        for m in mods {
            match m {
                Modification::AddTypes(t) if t.as_slice() == [CardType::Creature] => {
                    creature = true
                }
                Modification::SetPT(Some(Value::Const(_)), Some(Value::Const(_))) => pt = true,
                Modification::AddAbility(a) => abilities.push(a.clone()),
                _ => return None,
            }
        }
        if creature != pt || abilities.is_empty() {
            return None;
        }
        let texts: Vec<String> = abilities.iter().map(|a| self.nested_ability(a)).collect();
        Some(format!("{n}+ | {}", texts.join("\n")))
    }
}

impl Renderer<'_> {
    /// The instructions after one that sets X ("roll a d20", then "create X Treasure
    /// tokens"), with ", where X is [value]" after the first sentence that uses X.
    pub(crate) fn x_defined_later(&mut self, value: &Value, rest: &[Effect]) -> Option<String> {
        let x = self.value(value);
        if x == "X" || rest.is_empty() {
            return None;
        }
        let text = self.seq(rest);
        let uses_x = |s: &str| {
            s.split(|c: char| !c.is_alphanumeric() && c != '{' && c != '}')
                .any(|w| w == "X" || w == "{X}")
        };
        let sentences: Vec<&str> = text.split(". ").collect();
        let at = sentences.iter().position(|s| uses_x(s))?;
        if sentences[at].contains("where X is") {
            return Some(text);
        }
        let out: Vec<String> = sentences
            .iter()
            .enumerate()
            .map(|(i, s)| {
                if i == at {
                    format!("{}, where X is {x}", s.trim_end_matches('.'))
                } else {
                    s.to_string()
                }
            })
            .collect();
        Some(out.join(". "))
    }
}

impl Renderer<'_> {
    /// "Double the power [and toughness] of [objects] until end of turn": each gets
    /// +X/+0 (+X/+Y), where X is its power (Y its toughness) (CR 701.10b); "triple" for
    /// twice that.
    pub(crate) fn double_pt(&mut self, sel: &Sel, var: Var, effect: &Effect) -> Option<String> {
        let (verb, what, duration) = double_pt_parts(var, effect)?;
        let s = self.sel(sel, Case::Obj);
        let d = self.duration(duration);
        let poss = super::nouns::possessive(&s);
        let obj = if s.contains(['{', '|']) {
            format!("{what} of {s}")
        } else {
            format!(
                "{{alt:{what} of {s}|{poss} {}}}",
                what.trim_start_matches("the ")
            )
        };
        Some(join_words(&[format!("{verb} {obj}"), d]))
    }
}

/// The parts of a "double the power of ..." instruction: the verb, what's doubled and for
/// how long.
pub(crate) fn double_pt_parts(
    var: Var,
    effect: &Effect,
) -> Option<(&'static str, &'static str, &Duration)> {
    {
        let Effect::Modify {
            what: Sel::Var(w),
            mods,
            duration,
        } = effect
        else {
            return None;
        };
        let [Modification::ModifyPT(p, t)] = mods.as_slice() else {
            return None;
        };
        if *w != var {
            return None;
        }
        // The factor of its own power (toughness) it gets: 1 doubles, 2 triples.
        let factor = |v: &Value, power: bool| -> Option<i64> {
            let Value::Mul(a, b) = v else { return None };
            let own = match (a.as_ref(), power) {
                (Value::PowerOf(s), true) | (Value::ToughnessOf(s), false) => {
                    matches!(s.as_ref(), Sel::Var(x) if *x == var)
                }
                _ => false,
            };
            match b.as_ref() {
                Value::Const(k) if own => Some(*k as i64),
                _ => None,
            }
        };
        let fp = factor(p, true);
        let ft = factor(t, false);
        let (k, what) = match (fp, ft, t) {
            (Some(a), Some(b), _) if a == b => (a, "the power and toughness"),
            (Some(a), None, Value::Const(0)) => (a, "the power"),
            (None, Some(b), _) if matches!(p, Value::Const(0)) => (b, "the toughness"),
            _ => return None,
        };
        let verb = match k {
            1 => "double",
            2 => "triple",
            _ => return None,
        };
        Some((verb, what, duration))
    }
}

impl Renderer<'_> {
    /// What an extra turn's start creates, said of "that turn": "at the beginning of that
    /// turn's end step, ...", "during that turn, ...".
    pub(crate) fn extra_turn_effects(&mut self, e: &Effect) -> Option<String> {
        let parts: Vec<&Effect> = match e {
            Effect::Seq(v) => v.iter().collect(),
            e => vec![e],
        };
        let mut out = Vec::new();
        for p in parts {
            match p {
                Effect::AtNext {
                    step: TriggerStep::End,
                    effect,
                } => {
                    let t = self.effect(effect);
                    out.push(format!("at the beginning of that turn's end step, {t}"));
                }
                Effect::AddRestriction {
                    duration: Duration::EndOfTurn,
                    ..
                }
                | Effect::AddPlayerEffect {
                    duration: Duration::EndOfTurn,
                    ..
                } => {
                    let t = self.effect(p);
                    let t = t
                        .strip_suffix(" this turn")
                        .or_else(|| t.strip_suffix(" until end of turn"))?
                        .to_string();
                    out.push(format!("during that turn, {t}"));
                }
                _ => return None,
            }
        }
        Some(out.join(". "))
    }
}

/// "You may pay {0} rather than pay the equip cost of the first equip ability you activate
/// each turn" (an alternative cost for a keyword's activated abilities, CR 118.9).
pub(crate) fn first_ability_alt_cost(
    r: &mut Renderer<'_>,
    cm: &CostModifier,
    your_turns: bool,
) -> Option<String> {
    let CostChange::AlternativeCost(c) = &cm.change else {
        return None;
    };
    let CostTarget::ActivatedAbilities(scope) = &cm.applies_to else {
        return None;
    };
    let AbilityClass::Keyword(k) = scope.class else {
        return None;
    };
    if !scope.first_each_turn
        || !matches!(scope.sources, Filter::Any)
        || scope.targeting.is_some()
        || cm.who != PlayerRel::You
    {
        return None;
    }
    let pay = r.cost_as_payment(c);
    let pay = pay.strip_prefix("pay ").unwrap_or(&pay).to_string();
    let k = k.name().to_lowercase();
    let when = if your_turns {
        "during each of your turns"
    } else {
        "each turn"
    };
    Some(format!(
        "you may pay {pay} rather than pay the {k} cost of the first {k} ability you activate {when}"
    ))
}
