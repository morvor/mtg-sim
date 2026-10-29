//! Instructions implemented in code (`Effect::Custom`) and the instruction sequences they
//! begin, put into words from what they do: bidding life (`bidding.rs`), reordering a
//! graveyard and putting a card beneath the top cards of a library
//! (`kw/zones_dig_effects.rs`), revealing cards at random (`kw/reveal_at_random.rs`),
//! revealing a library (`reveal_library.rs`), a hand and the top card of a library, and
//! "Repeat this process until no one pays life" (`kw/loop_totals.rs`).

use super::players::Case;
use super::*;

impl Renderer<'_> {
    /// The words of a custom instruction this module knows, as performed by "you".
    pub(crate) fn named_effect(&mut self, name: &str) -> Option<String> {
        use crate::kw::zones_dig_effects as z;
        if let Some(b) = z::parse_beneath(name) {
            // "Put target creature into its owner's library just beneath the top X cards
            // of that library, where X is ..." (with fewer cards, on the bottom).
            let what = self.sel(&b.what, Case::Obj);
            let (n, w) = match &b.n {
                Value::X => ("X".to_string(), String::new()),
                Value::Const(k) => (number_word(*k), String::new()),
                other => {
                    let v = self.value(other);
                    ("X".to_string(), format!(", where X is {v}"))
                }
            };
            return Some(format!(
                "put {what} into its owner's library just beneath the top {n} cards of that library{w}"
            ));
        }
        if let Some(r) = name.strip_prefix("reveal cards at random from hand:") {
            let (n, who) = r.split_once(':')?;
            // "reveals X cards at random" (Nebuchadnezzar): the ability's X.
            let n: i32 = if n == "x" { -1 } else { n.parse().ok()? };
            let cards = if n == -1 {
                "X cards".to_string()
            } else if n == 1 {
                "a card".to_string()
            } else {
                format!("{} cards", number_word(n))
            };
            let who = if who == "you" {
                PlayerRef::You
            } else if let Some(s) = who.strip_prefix("controller of target ") {
                PlayerRef::ControllerOf(Box::new(Sel::Target(s.parse().ok()?)))
            } else {
                PlayerRef::Target(who.strip_prefix("target ")?.parse().ok()?)
            };
            if matches!(who, PlayerRef::You) {
                return Some(format!("reveal {cards} at random from your hand"));
            }
            // "Target creature's controller reveals ...": the target named first here.
            if let PlayerRef::ControllerOf(s) = &who {
                if let Sel::Target(t) = s.as_ref() {
                    if self.introduced.get(*t as usize) == Some(&false) {
                        let tp = self.target_phrase(*t);
                        self.introduced[*t as usize] = true;
                        let tp = tp.replace(" {opt:on the battlefield}", "");
                        return Some(format!(
                            "{tp}'s controller reveals {cards} at random from their hand"
                        ));
                    }
                }
            }
            let p = self.player(&who, Case::Subj);
            return Some(format!("{p} reveals {cards} at random from their hand"));
        }
        Some(match name {
            crate::choices::FOR_EACH_DIFFERENT_POWER => {
                "for each different power among creatures on the battlefield, choose a creature with that power".into()
            }
            // Bookkeeping: the object beheld is remembered for the spell (`choices.rs`),
            // named by the value that reads it.
            crate::choices::REMEMBER_CHOSEN_OBJECTS => String::new(),
            z::REORDER_GRAVEYARD_CHOOSE => "reorder your graveyard as you choose".into(),
            z::REORDER_GRAVEYARD_RANDOM => "reorder your graveyard at random".into(),
            // The player performing it reveals their own library.
            crate::reveal_library::REVEAL_OWN_LIBRARY => {
                if self.in_as_player {
                    "reveal their library".into()
                } else {
                    "reveal your library".into()
                }
            }
            _ => return None,
        })
    }

    /// Instruction sequences this module words as a whole: (text, instructions used).
    pub(crate) fn named_seq(&mut self, v: &[Effect], i: usize) -> Option<(String, usize)> {
        self.two_choices(&v[i..])
            .or_else(|| self.rite_of_types(&v[i..]))
            .or_else(|| self.chosen_number_seq(&v[i..]))
            .or_else(|| self.balance_seq(&v[i..]))
            .or_else(|| self.choose_kind_then_reveal(&v[i..]))
            .or_else(|| self.revealed_at_random(&v[i..]))
            .or_else(|| self.bidding(&v[i..]))
            .or_else(|| self.hand_and_top_revealed(&v[i..]))
            .or_else(|| self.reveal_hand_or_top(&v[i..]))
            .or_else(|| self.repeat_until_no_one(&v[i..]))
            .or_else(|| self.delayed_unless_paid(&v[i..]))
    }

    /// "Target player reveals two cards at random from their hand, then discards each
    /// nonland card revealed this way."; "Target creature's controller reveals a card at
    /// random from their hand. ~ deals damage to that creature and that player equal to
    /// the revealed card's mana value."
    fn revealed_at_random(&mut self, v: &[Effect]) -> Option<(String, usize)> {
        let [r @ Effect::Custom(n), next, ..] = v else {
            return None;
        };
        let (_, who) = n
            .strip_prefix("reveal cards at random from hand:")?
            .split_once(':')?;
        match next {
            Effect::Discard {
                who: PlayerRef::Target(t),
                n: Value::CountSel(c),
                random: false,
                filter: f @ Filter::And(parts),
            } if who == format!("target {t}")
                && matches!(c.as_ref(), Sel::All(g) if format!("{g:?}") == format!("{f:?}")) =>
            {
                let [Filter::In(x), rest @ ..] = parts.as_slice() else {
                    return None;
                };
                if !matches!(x.as_ref(), Sel::Var(vars::IT)) {
                    return None;
                }
                let mut q = rest.to_vec();
                q.push(Filter::Card);
                let noun = self.noun(&Filter::and(q), Num::One);
                let s = self.effect(r);
                Some((
                    format!("{s}, then discards each {noun} revealed this way"),
                    2,
                ))
            }
            Effect::DealDamage {
                source: Sel::This,
                amount: Value::ManaValueOf(m),
                to: Sel::Union(to),
            } if matches!(m.as_ref(), Sel::Var(vars::IT)) => {
                let [Sel::Target(t), Sel::Players(PlayerRef::ControllerOf(c))] = to.as_slice()
                else {
                    return None;
                };
                if who != format!("controller of target {t}")
                    || !matches!(c.as_ref(), Sel::Target(u) if u == t)
                {
                    return None;
                }
                let s = self.effect(r);
                let noun = match self.targets.get(*t as usize).map(|x| &x.what) {
                    Some(TargetKind::Object(f)) if format!("{f:?}").contains("Creature") => {
                        "creature"
                    }
                    _ => "permanent",
                };
                let m = self.me();
                Some((
                    format!("{s}. {m} deals damage to that {noun} and that player equal to the revealed card's mana value"),
                    2,
                ))
            }
            _ => None,
        }
    }

    /// "Each player may bid life for control of target creature. You start the bidding
    /// with a bid of 0. In turn order, each player may top the high bid. The bidding ends
    /// if the high bid stands. The high bidder loses life equal to the high bid and gains
    /// control of the creature." (CR 101.4: in turn order).
    fn bidding(&mut self, v: &[Effect]) -> Option<(String, usize)> {
        let [Effect::Custom(n), Effect::AsPlayer { who, effect }, rest @ ..] = v else {
            return None;
        };
        let (each, start) = if let Some(s) = n.strip_prefix(crate::bidding::EACH_PLAYER_BIDS) {
            (true, s)
        } else {
            (
                false,
                n.strip_prefix(crate::bidding::YOU_AND_TARGET_CONTROLLER_BID)?,
            )
        };
        if !matches!(who, PlayerRef::Var(x) if *x == crate::bidding::HIGH_BIDDER) {
            return None;
        }
        let Effect::Seq(steps) = effect.as_ref() else {
            return None;
        };
        let [Effect::SetX {
            value: Value::Var(bid),
        }, Effect::LoseLife {
            who: PlayerRef::You,
            n: Value::X,
        }, then @ ..] = steps.as_slice()
        else {
            return None;
        };
        if *bid != crate::bidding::HIGH_BID {
            return None;
        }
        // What the high bidder gets: "and gains control of the creature", "and draws
        // four cards".
        let mut prize = String::new();
        let mut control_of = String::new();
        for e in then {
            match e {
                Effect::GainControl {
                    what: Sel::Target(t),
                    who: PlayerRef::You,
                    duration: Duration::Permanent,
                } => {
                    let tn = self.target_phrase(*t);
                    if let Some(x) = self.introduced.get_mut(*t as usize) {
                        *x = true;
                    }
                    control_of = format!(" for control of {tn}");
                    prize.push_str(" and gains control of the creature");
                }
                other => {
                    let s = self.effect(other);
                    if s.is_empty() || s.contains('.') {
                        return None;
                    }
                    prize.push_str(&format!(" and {}", effects::third_person(&s)));
                }
            }
        }
        let bidders = if each {
            format!("each player may bid life{control_of}")
        } else {
            let t = self.target_mention(0, Case::Poss);
            format!("you and {t} controller bid life{control_of}")
        };
        let opening = match start {
            "any" => "any number".to_string(),
            n => n.to_string(),
        };
        let mut out = format!(
            "{bidders}. you start the bidding with a bid of {opening}. in turn order, each player may top the high bid. the bidding ends if the high bid stands. the high bidder loses life equal to the high bid{prize}"
        );
        let mut used = 2;
        // "If you win the bidding, counter that spell."
        if let [Effect::If {
            cond: Condition::PlayerMatches(PlayerRef::You, PlayerFilter::Ref(r)),
            then,
            otherwise,
        }, ..] = rest
        {
            if matches!(r.as_ref(), PlayerRef::Var(x) if *x == crate::bidding::HIGH_BIDDER)
                && matches!(otherwise.as_ref(), Effect::Noop)
            {
                let t = self.effect(then);
                out.push_str(&format!(". if you win the bidding, {t}"));
                used = 3;
            }
        }
        Some((out, used))
    }

    /// "Target player reveals their hand and the top card of their library. You choose a
    /// card revealed this way." (the card chosen is "the chosen card" after).
    fn hand_and_top_revealed(&mut self, v: &[Effect]) -> Option<(String, usize)> {
        let [Effect::RevealHand { who }, Effect::Custom(n), Effect::Store {
            var: group,
            sel: Sel::Union(parts),
        }, Effect::Store {
            var: chosen,
            sel:
                Sel::Choose {
                    chooser: PlayerRef::You,
                    filter: Filter::In(from),
                    count: Value::Const(1),
                    up_to: false,
                    ..
                },
        }, ..] = v
        else {
            return None;
        };
        let top = Sel::TopOfLibrary(who.clone(), Value::c(1));
        let revealed: Sel = serde_json::from_str(n.strip_prefix("zones: reveal:")?).ok()?;
        if format!("{revealed:?}") != format!("{top:?}")
            || !matches!(from.as_ref(), Sel::Var(g) if g == group)
            || parts.len() != 2
            || format!("{:?}", parts[1]) != format!("{top:?}")
        {
            return None;
        }
        let p = self.player(who, Case::Subj);
        self.target_vars
            .push((*chosen, "{alt:the chosen card|it}".into(), false));
        Some((
            format!(
                "{p} reveals their hand and the top card of their library. you choose a card revealed this way"
            ),
            4,
        ))
    }

    /// "Reveal a card from your hand or the top card of your library. If you reveal a
    /// creature card this way, put it onto the battlefield."
    fn reveal_hand_or_top(&mut self, v: &[Effect]) -> Option<(String, usize)> {
        let [Effect::Store {
            var: group,
            sel: Sel::Union(parts),
        }, Effect::Store {
            var: chosen,
            sel:
                Sel::Choose {
                    chooser: PlayerRef::You,
                    filter: Filter::In(from),
                    count: Value::Const(1),
                    up_to: false,
                    ..
                },
        }, Effect::Custom(n), rest @ ..] = v
        else {
            return None;
        };
        let [Sel::All(Filter::And(hand)), Sel::TopOfLibrary(PlayerRef::You, Value::Const(1))] =
            parts.as_slice()
        else {
            return None;
        };
        let in_hand = hand
            .iter()
            .any(|f| matches!(f, Filter::InZone(ZoneKind::Hand)))
            && hand
                .iter()
                .any(|f| matches!(f, Filter::OwnedBy(PlayerRel::You)));
        let revealed: Sel = serde_json::from_str(n.strip_prefix("zones: reveal:")?).ok()?;
        if !in_hand
            || !matches!(from.as_ref(), Sel::Var(g) if g == group)
            || !matches!(revealed, Sel::Var(x) if x == *chosen)
        {
            return None;
        }
        let mut out = "reveal a card from your hand or the top card of your library".to_string();
        let mut used = 3;
        if let [Effect::If {
            cond: Condition::SelMatches(Sel::Var(x), f),
            then,
            otherwise,
        }, ..] = rest
        {
            if x == chosen && matches!(otherwise.as_ref(), Effect::Noop) {
                let c = self.noun_det(f, super::nouns::Det::A);
                let t = self.effect(then);
                out.push_str(&format!(". if you reveal {c} this way, {t}"));
                used = 4;
            }
        }
        Some((out, used))
    }

    /// "That player loses 2 life at the beginning of their next draw step unless they pay
    /// {2} before that step." (a delayed trigger the player may stop by paying, CR 116.2).
    fn delayed_unless_paid(&mut self, v: &[Effect]) -> Option<(String, usize)> {
        let [d @ Effect::DelayedTrigger { .. }, o @ Effect::OfferSpecialAction { def, .. }, ..] = v
        else {
            return None;
        };
        if !matches!(def.action, SpecialActionEffect::StopDelayedTrigger) {
            return None;
        }
        let a = self.effect(d);
        let b = self.effect(o);
        // "before that draw step": the step it waits for, named.
        let step = match d {
            Effect::DelayedTrigger {
                trigger: TriggerCond::BeginningOf { step, .. },
                ..
            } => match step {
                TriggerStep::Draw => Some("draw step"),
                TriggerStep::Upkeep => Some("upkeep"),
                _ => None,
            },
            _ => None,
        };
        let b = match step {
            Some(w) if b.contains(" before that step") => {
                let named = b.replacen(" before that step", &format!(" before that {w}"), 1);
                format!("{{alt:{b}|{named}}}")
            }
            _ => b,
        };
        Some((format!("{a} {b}"), 2))
    }

    /// "Starting with you, each player may pay any amount of life. Repeat this process
    /// until no one pays life." (`oracle/patterns/loop_totals_grammar.rs`).
    fn repeat_until_no_one(&mut self, v: &[Effect]) -> Option<(String, usize)> {
        let [Effect::RepeatProcess { body }, ..] = v else {
            return None;
        };
        let Effect::Seq(b) = body.as_ref() else {
            return None;
        };
        let [Effect::Seq(pass), Effect::If {
            cond: Condition::Compare(Value::Var(did), Cmp::Gt, Value::Const(0)),
            then,
            otherwise,
        }] = b.as_slice()
        else {
            return None;
        };
        if *did != crate::tallies::SOMEONE_DID
            || !matches!(then.as_ref(), Effect::RepeatThisProcess)
            || !matches!(otherwise.as_ref(), Effect::Noop)
        {
            return None;
        }
        let [Effect::StoreValue { var, .. }, order @ Effect::InTurnOrder { effect, .. }] =
            pass.as_slice()
        else {
            return None;
        };
        if *var != *did {
            return None;
        }
        let Effect::AsPlayer { effect: each, .. } = effect.as_ref() else {
            return None;
        };
        let until = match each.as_ref() {
            Effect::Custom(n) if n == crate::kw::loop_totals::PAY_ANY_LIFE_TALLIED => {
                "no one pays life"
            }
            _ => return None,
        };
        let s = self.effect(order);
        Some((format!("{s}. repeat this process until {until}"), 1))
    }
}

impl Renderer<'_> {
    /// "Target player skips their next combat phase this turn." (only the active player
    /// has a combat phase this turn, CR 506.1; a one-use skip, CR 614.10).
    pub(crate) fn skips_next_this_turn(
        &mut self,
        cond: &Condition,
        then: &Effect,
        otherwise: &Effect,
    ) -> Option<String> {
        let Condition::PlayerMatches(p @ PlayerRef::Target(_), PlayerFilter::Active) = cond else {
            return None;
        };
        let Effect::AddReplacement {
            def,
            duration: Duration::EndOfTurn | Duration::ThisTurn,
            uses: Some(1),
        } = then
        else {
            return None;
        };
        if !matches!(otherwise, Effect::Noop) {
            return None;
        }
        let (
            ReplacementEvent::SkipStep {
                step,
                whose: PlayerRel::Any,
            },
            ReplacementAction::Prevent,
        ) = (&def.event, &def.action)
        else {
            return None;
        };
        let what = match step {
            StepKind::Combat => "combat phase",
            StepKind::Upkeep => "upkeep step",
            StepKind::Draw => "draw step",
            StepKind::End => "end step",
            _ => return None,
        };
        let w = self.player(p, Case::Subj);
        Some(format!("{w} skips their next {what} this turn"))
    }
}

impl Renderer<'_> {
    /// "Choose a land type and a basic land type." (two choices, one sentence).
    fn two_choices(&mut self, v: &[Effect]) -> Option<(String, usize)> {
        let [Effect::Choose {
            who: PlayerRef::You,
            kind: a,
        }, Effect::Choose {
            who: PlayerRef::You,
            kind: b,
        }, ..] = v
        else {
            return None;
        };
        let player = |k: &ChoiceKind| matches!(k, ChoiceKind::Opponent | ChoiceKind::Player);
        if player(a) || player(b) {
            return None;
        }
        let x = self.choice(a);
        let y = self.choice(b);
        Some((
            format!("{{alt:choose {x}. choose {y}|choose {x} and {y}}}"),
            2,
        ))
    }

    /// "Each player sacrifices one permanent of their choice of the first type, sacrifices
    /// two of their choice of the second type, then sacrifices three of their choice of
    /// the third type." (the types in the order chosen).
    fn rite_of_types(&mut self, v: &[Effect]) -> Option<(String, usize)> {
        let [Effect::ForEachPlayer {
            who: PlayerRef::EachPlayer,
            effect,
        }, ..] = v
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
        let Effect::Seq(steps) = inner.as_ref() else {
            return None;
        };
        let mut parts = Vec::new();
        for (i, s) in steps.iter().enumerate() {
            let Effect::Sacrifice {
                who: PlayerRef::You,
                filter: Filter::And(f),
                count: Value::Const(n),
            } = s
            else {
                return None;
            };
            let [Filter::Permanent, Filter::Custom(c)] = f.as_slice() else {
                return None;
            };
            let k: usize = c
                .strip_prefix(crate::choices::NTH_CHOSEN_CARD_TYPE)?
                .parse()
                .ok()?;
            let nth = ordinal_word(k as u32 + 1);
            let what = if i == 0 {
                format!("{} permanent of their choice", number_word(*n))
            } else {
                format!("{} of their choice", number_word(*n))
            };
            parts.push(format!("sacrifices {what} of the {nth} type"));
        }
        if parts.len() < 2 {
            return None;
        }
        let last = parts.pop()?;
        Some((format!("each player {}, then {last}", parts.join(", ")), 1))
    }
}

impl Renderer<'_> {
    /// "Double your life total" (CR 701.10d: the player gains or loses the life that
    /// makes it twice what it was).
    pub(crate) fn double_life_total(
        &mut self,
        cond: &Condition,
        then: &Effect,
        otherwise: &Effect,
    ) -> Option<String> {
        let Condition::Compare(Value::LifeTotal(p), Cmp::Ge, Value::Const(0)) = cond else {
            return None;
        };
        let Effect::GainLife {
            who: g,
            n: Value::Mul(a, k),
        } = then
        else {
            return None;
        };
        let Effect::LoseLife { who: l, n } = otherwise else {
            return None;
        };
        let same = |x: &PlayerRef| format!("{x:?}") == format!("{p:?}");
        if !same(g)
            || !same(l)
            || !matches!(a.as_ref(), Value::LifeTotal(x) if same(x))
            || !matches!(k.as_ref(), Value::Const(1))
            // It loses the same amount when its life total is negative.
            || !matches!(n, Value::Mul(c, m)
                if matches!(m.as_ref(), Value::Const(-1))
                    && matches!(c.as_ref(), Value::Mul(a2, k2)
                        if matches!(a2.as_ref(), Value::LifeTotal(x) if same(x))
                            && matches!(k2.as_ref(), Value::Const(1))))
        {
            return None;
        }
        let w = self.player(p, Case::Poss);
        Some(format!("double {w} life total"))
    }
}
