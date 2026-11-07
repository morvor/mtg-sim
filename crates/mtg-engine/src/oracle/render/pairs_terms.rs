//! Wordings from roundtrip-pairs-4 (docs/agents/items/roundtrip-pairs-4.md), each put into
//! words from the AST it is compiled to.

use super::effects::lower_first;
use super::players::Case;
use super::*;

impl Renderer<'_> {
    /// "If ~ would enter, you may discard a land card instead. If you do, put ~ onto the
    /// battlefield. If you don't, put it into its owner's graveyard." (Mox Diamond, a
    /// choice) and "If ~ would enter, sacrifice a Forest instead. If you do, ..." (Lake of
    /// the Dead: the instruction, then whether all of it was done).
    pub(crate) fn enter_only_if_done(&mut self, def: &ReplacementDef) -> Option<String> {
        let (ReplacementEvent::EntersBattlefield(Filter::Source), ReplacementAction::Instead(e)) =
            (&def.event, &def.action)
        else {
            return None;
        };
        let Effect::Seq(v) = e.as_ref() else {
            return None;
        };
        let [first, Effect::If {
            cond,
            then,
            otherwise,
        }] = v.as_slice()
        else {
            return None;
        };
        let same = |a: &Destination, b: Destination| format!("{a:?}") == format!("{b:?}");
        let enters = matches!(then.as_ref(), Effect::Move { what: Sel::This, to }
            if same(to, Destination::battlefield()));
        let to_graveyard = matches!(otherwise.as_ref(), Effect::Move { what: Sel::This, to }
            if same(to, Destination::zone(ZoneKind::Graveyard)));
        if !enters || !to_graveyard {
            return None;
        }
        let instruction = match (first, cond) {
            (
                Effect::PayOptional {
                    who: PlayerRef::You,
                    cost,
                    then,
                    otherwise,
                },
                Condition::PrevHappened,
            ) if matches!(then.as_ref(), Effect::Noop)
                && matches!(otherwise.as_ref(), Effect::Noop) =>
            {
                let c = self.cost(cost);
                format!("you may {}", lower_first(&c))
            }
            (
                Effect::Sacrifice {
                    who: PlayerRef::You,
                    filter,
                    count,
                },
                Condition::Compare(Value::Prev, Cmp::Ge, n),
            ) if format!("{n:?}") == format!("{count:?}") => {
                let c = self.cost(&Cost {
                    mana: None,
                    parts: vec![CostPart::Sacrifice {
                        filter: filter.clone(),
                        count: count.clone(),
                    }],
                });
                lower_first(&c)
            }
            _ => return None,
        };
        let m = self.me();
        Some(format!(
            "If {m} would enter, {instruction} instead. If you do, put {m} onto the battlefield. If you don't, put it into its owner's graveyard"
        ))
    }

    /// "if you gained or lost life this turn": either one (one player, one verb phrase).
    pub(crate) fn gained_or_lost_life(&mut self, v: &[Condition]) -> Option<String> {
        let some = |c: &Condition| -> Option<(bool, PlayerRef)> {
            let Condition::Compare(a, cmp, Value::Const(n)) = c else {
                return None;
            };
            if !matches!((cmp, n), (Cmp::Gt, 0) | (Cmp::Ge, 1)) {
                return None;
            }
            match a {
                Value::LifeGainedThisTurn(p) => Some((true, p.clone())),
                Value::LifeLostThisTurn(p) => Some((false, p.clone())),
                _ => None,
            }
        };
        let [a, b] = v else { return None };
        let ((true, p), (false, q)) = (some(a)?, some(b)?) else {
            return None;
        };
        if format!("{p:?}") != format!("{q:?}") {
            return None;
        }
        let w = self.player(&p, Case::Subj);
        Some(format!("{w} gained or lost life this turn"))
    }

    /// "Enchanted creature gets +1/+2 as long as it's white. Otherwise, it gets -2/-1."
    /// (Serra's Boon): two static abilities, one while the object has the quality and one
    /// while it doesn't, printed as one. (How many abilities, the line.)
    pub(crate) fn otherwise_group(&mut self, abs: &[Ability]) -> Option<(usize, String)> {
        let [a, b, ..] = abs else { return None };
        let (AbilityKind::Static(x), AbilityKind::Static(y)) = (&a.kind, &b.kind) else {
            return None;
        };
        let (Some(Condition::SelMatches(sel, f)), Some(Condition::Not(n))) =
            (&x.condition, &y.condition)
        else {
            return None;
        };
        if format!("{:?}", Condition::SelMatches(sel.clone(), f.clone())) != format!("{n:?}") {
            return None;
        }
        let (
            StaticEffect::Continuous { affected, mods: m1 },
            StaticEffect::Continuous {
                affected: affected2,
                mods: m2,
            },
        ) = (&x.effect, &y.effect)
        else {
            return None;
        };
        // The quality is the affected object's own.
        let same_object = matches!(
            (sel, affected),
            (Sel::AttachedTo, Filter::AttachedToSource) | (Sel::This, Filter::Source)
        );
        if !same_object
            || format!("{affected:?}") != format!("{affected2:?}")
            || x.zone != y.zone
            || x.zone != FunctionZone::Battlefield
        {
            return None;
        }
        let subj = self.static_subject(affected);
        let vp1 = self.mods_vp(m1, false);
        let vp2 = self.mods_vp(m2, false);
        let pred = self.is_predicate(f, false);
        Some((
            2,
            format!(
                "{} {vp1} as long as it {pred}. Otherwise, it {vp2}.",
                capitalize(&subj)
            ),
        ))
    }

    /// "As long as you control exactly one creature, that creature gets +3/+1 and has
    /// lifelink.": the creatures you control are then that one creature.
    pub(crate) fn exactly_one_static(&mut self, abs: &[Ability]) -> Option<(usize, String)> {
        let AbilityKind::Static(x) = &abs.first()?.kind else {
            return None;
        };
        let Some(Condition::Compare(Value::Count(f), Cmp::Eq, Value::Const(1))) = &x.condition
        else {
            return None;
        };
        let StaticEffect::Continuous { affected, mods } = &x.effect else {
            return None;
        };
        if format!("{affected:?}") != format!("{f:?}")
            || x.zone != FunctionZone::Battlefield
            || !matches!(super::values::split_controller(f).0, Some(PlayerRel::You))
        {
            return None;
        }
        let one = self.noun(f, Num::One);
        let n = one.strip_suffix(" you control")?.to_string();
        let vp = self.mods_vp(mods, false);
        Some((
            1,
            format!("As long as you control exactly one {n}, that {n} {vp}."),
        ))
    }

    /// "Each spell you cast has split second if mana from an artifact was spent to cast
    /// it": the spells some of whose mana was of that kind
    /// (`kw::cast_history::SPENT_SOME`).
    pub(crate) fn spent_some_static(
        &mut self,
        affected: &Filter,
        mods: &[Modification],
    ) -> Option<String> {
        let Filter::And(v) = affected else {
            return None;
        };
        let [Filter::Spell, Filter::ControlledBy(PlayerRel::You), Filter::Custom(n)] = v.as_slice()
        else {
            return None;
        };
        let clause = self.spent_some_clause(n)?;
        let vp = self.mods_vp(mods, false);
        Some(format!("each spell you cast {vp} if {clause}"))
    }

    /// "If a spell or ability would cause its controller to gain life, that player loses
    /// that much life instead." (Rain of Gore, `kw::life_gain_cause`).
    pub(crate) fn caused_life_gain(&mut self, def: &ReplacementDef) -> Option<String> {
        let ReplacementEvent::Where { event, cond } = &def.event else {
            return None;
        };
        let (ReplacementEvent::GainLife(PlayerFilter::Any), Condition::Custom(c)) =
            (event.as_ref(), cond)
        else {
            return None;
        };
        if c != crate::kw::life_gain_cause::RESOLVING_CONTROLLER {
            return None;
        }
        let then = match &def.action {
            ReplacementAction::Instead(e)
                if matches!(
                    e.as_ref(),
                    Effect::LoseLife {
                        who: PlayerRef::TriggerPlayer,
                        n: Value::EventAmount
                    }
                ) =>
            {
                "that player loses that much life instead".to_string()
            }
            _ => return None,
        };
        Some(format!(
            "if a spell or ability would cause its controller to gain life, {then}"
        ))
    }

    /// "Whenever another artifact is put into your graveyard from the battlefield or an
    /// artifact card is put into your graveyard from anywhere other than the battlefield"
    /// (Ultron): one trigger condition for each zone a card can be put into a graveyard
    /// from (CR 400.1).
    pub(crate) fn into_graveyard_from_anywhere(&mut self, t: &TriggerCond) -> Option<String> {
        let TriggerCond::AnyOf(v) = t else {
            return None;
        };
        let (TriggerCond::Dies(Filter::And(d)), rest) = v.split_first()? else {
            return None;
        };
        let [Filter::Other, kind, Filter::OwnedBy(PlayerRel::You)] = d.as_slice() else {
            return None;
        };
        let card = Filter::And(vec![
            kind.clone(),
            Filter::Card,
            Filter::OwnedBy(PlayerRel::You),
        ]);
        let mut zones: Vec<ZoneKind> = Vec::new();
        for x in rest {
            let TriggerCond::ZoneChange {
                filter,
                from: Some(z),
                to: Some(ZoneKind::Graveyard),
            } = x
            else {
                return None;
            };
            if format!("{filter:?}") != format!("{card:?}") {
                return None;
            }
            zones.push(*z);
        }
        use ZoneKind::*;
        let all = [Library, Hand, Stack, Exile, Command];
        if zones.len() != all.len() || !all.iter().all(|z| zones.contains(z)) {
            return None;
        }
        let one = self.noun(kind, Num::One);
        let a = with_article(&one);
        Some(format!(
            "whenever another {one} is put into your graveyard from the battlefield or {a} card is put into your graveyard from anywhere other than the battlefield"
        ))
    }

    /// "During your turn, your opponents can't cast spells or activate abilities of
    /// artifacts, creatures, or enchantments." (Grand Abolisher): two restrictions on the
    /// same players with the same condition, printed as one sentence. (How many
    /// abilities, the line.)
    pub(crate) fn cast_or_activate_group(&mut self, abs: &[Ability]) -> Option<(usize, String)> {
        let [a, b, ..] = abs else { return None };
        let (AbilityKind::Static(x), AbilityKind::Static(y)) = (&a.kind, &b.kind) else {
            return None;
        };
        let (
            StaticEffect::Restriction(Restriction::CantCast {
                who,
                what: Filter::Any,
            }),
            StaticEffect::Restriction(Restriction::CantActivate {
                who: who2,
                sources,
                include_mana: true,
            }),
        ) = (&x.effect, &y.effect)
        else {
            return None;
        };
        if format!("{who:?}") != format!("{who2:?}")
            || !matches!(x.condition, Some(Condition::YourTurn))
            || !matches!(y.condition, Some(Condition::YourTurn))
            || x.zone != FunctionZone::Battlefield
            || y.zone != FunctionZone::Battlefield
        {
            return None;
        }
        let w = self.player_filter_subject(who);
        let n = self.noun(sources, Num::Many);
        Some((
            2,
            format!("During your turn, {w} can't cast spells or activate abilities of {n}."),
        ))
    }

    /// Instructions printed as one ("Target creature gains protection from the color of
    /// its controller's choice until end of turn"): (the text, how many instructions).
    pub(crate) fn pairs_seq(&mut self, v: &[Effect], i: usize) -> Option<(String, usize)> {
        // A color chosen by the target's controller, then protection from it (CR 702.16a).
        if let [Effect::Choose {
            who: PlayerRef::ControllerOf(c),
            kind: ChoiceKind::Color,
        }, Effect::Modify {
            what,
            mods,
            duration,
        }, ..] = &v[i..]
        {
            let protection_from_chosen = matches!(mods.as_slice(), [Modification::AddKeyword(k)]
                if k.kind == crate::keywords::KeywordKind::Protection
                    && matches!(k.filter, Some(Filter::ChosenColor)));
            if protection_from_chosen
                && matches!(what, Sel::Target(_))
                && format!("{:?}", c.as_ref()) == format!("{what:?}")
            {
                let t = self.sel(what, Case::Subj);
                let d = self.duration(duration);
                let d = if d.is_empty() { d } else { format!(" {d}") };
                return Some((
                    format!("{t} gains protection from the color of its controller's choice{d}"),
                    2,
                ));
            }
        }
        None
    }

    /// Conditions from this item's cards that the general condition renderer words
    /// differently.
    pub(crate) fn pairs_condition(&mut self, c: &Condition) -> Option<String> {
        // "if that player controls more lands than each other player": no other player
        // controls at least as many.
        if let Condition::Not(n) = c {
            if let Condition::PlayerMatches(PlayerRef::Each(PlayerFilter::Not(other)), pf) =
                n.as_ref()
            {
                let PlayerFilter::Ref(p) = other.as_ref() else {
                    return None;
                };
                let PlayerFilter::Controls(f, Cmp::Ge, v) = pf else {
                    return None;
                };
                let Value::Count(g) = v.as_ref() else {
                    return None;
                };
                let theirs = Filter::and(vec![
                    f.as_ref().clone(),
                    Filter::ControllerMatches(Box::new(PlayerFilter::Ref(p.clone()))),
                ]);
                if format!("{theirs:?}") != format!("{:?}", g) {
                    return None;
                }
                let w = self.player(p, Case::Subj);
                let verb = if w == "you" { "control" } else { "controls" };
                let n = self.noun(f, Num::Many);
                return Some(format!("{w} {verb} more {n} than each other player"));
            }
        }
        // "if a player other than you lost life this turn".
        if let Condition::Compare(
            Value::CountPlayers(PlayerFilter::And(v)),
            Cmp::Ge,
            Value::Const(1),
        ) = c
        {
            if let [PlayerFilter::NotYou, PlayerFilter::History(h)] = v.as_slice() {
                if let Condition::Compare(
                    Value::EventsThisTurn(t, Tally::Events),
                    Cmp::Ge,
                    Value::Const(1),
                ) = h.as_ref()
                {
                    if matches!(
                        t.as_ref(),
                        TriggerCond::LosesLife {
                            who: PlayerRel::Iterated
                        }
                    ) {
                        return Some("a player other than you lost life this turn".into());
                    }
                }
            }
        }
        None
    }
}
