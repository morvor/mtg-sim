//! More instructions and instruction sequences worded as a whole.

use super::players::Case;
use super::*;

/// The amount of a trigger's event, in words: the damage of a "deals damage to a player"
/// trigger, the cards looked at of a "whenever you scry" trigger (CR 701.22a).
pub(super) fn event_amount_words(t: &TriggerCond) -> Option<&'static str> {
    match t {
        TriggerCond::PlayerAction { name, .. } if name == "scry" => {
            Some("the number of cards looked at while scrying this way")
        }
        TriggerCond::DealsDamage {
            source: Filter::Source | Filter::AttachedToSource,
            to: DamageRecipient::Player(_),
            ..
        } => Some("the amount of damage it dealt to that player"),
        TriggerCond::Batched {
            trigger,
            per: BatchPer::Player,
        } => match trigger.as_ref() {
            TriggerCond::DealsDamage {
                to: DamageRecipient::Player(_),
                ..
            } => Some("the amount of damage those creatures dealt to that player"),
            _ => None,
        },
        _ => None,
    }
}

/// The kind of card of "Whenever you reveal a [card] this way" after "Reveal the first
/// card you draw each turn" (or "on each of your turns"): the condition on a draw trigger
/// that it's the first card drawn this turn and of that kind.
pub(super) fn first_draw_revealed(c: &Condition) -> Option<&Filter> {
    let Condition::And(v) = c else {
        return None;
    };
    match v.as_slice() {
        [Condition::Compare(Value::EventAmount, Cmp::Eq, Value::Const(1)), Condition::SelMatches(Sel::TriggerObject, f)]
        | [Condition::Compare(Value::EventAmount, Cmp::Eq, Value::Const(1)), Condition::SelMatches(Sel::TriggerObject, f), Condition::PlayerMatches(PlayerRef::You, PlayerFilter::Active)] => {
            Some(f)
        }
        _ => None,
    }
}

/// "Choose a number from 0 to N" stored for a following instruction that uses it.
fn chosen_number(e: &Effect) -> bool {
    matches!(
        e,
        Effect::Choose {
            who: PlayerRef::You,
            kind: ChoiceKind::Number { min: 0, .. },
        }
    )
}

impl Renderer<'_> {
    /// An instruction worded as a whole, before the general wording.
    pub(crate) fn effect_more(&mut self, e: &Effect) -> Option<String> {
        match e {
            // "Move any number of +1/+1 counters from other permanents you control onto
            // ~": for each of them, a number chosen and that many moved (moving more
            // counters than it has moves all of them).
            Effect::ForEach {
                sel: Sel::All(f),
                var,
                effect,
            } => {
                let Effect::Seq(v) = effect.as_ref() else {
                    return None;
                };
                let [c, Effect::MoveCounters {
                    from: Sel::Var(x),
                    to,
                    kind: Some(k),
                    n: Some(Value::Chosen),
                }] = v.as_slice()
                else {
                    return None;
                };
                if !chosen_number(c) || x != var || matches!(to, Sel::Var(t) if t == var) {
                    return None;
                }
                let from = self.noun(f, Num::Many);
                let t = self.sel(to, Case::Obj);
                Some(format!(
                    "move any number of {} from {from} onto {t}",
                    plural(&counter_name(k))
                ))
            }
            // "The owner of target nonland permanent puts it into their library second from
            // the top or on the bottom": the owner chooses where.
            Effect::ChooseOne {
                who: PlayerRef::OwnerOf(o),
                options,
            } => {
                let [(_, Effect::Move { what: a, to: x }), (_, Effect::Move { what: b, to: y })] =
                    options.as_slice()
                else {
                    return None;
                };
                let same = |s: &Sel| format!("{s:?}") == format!("{o:?}");
                let lib = |d: &Destination, p: LibraryPosition| {
                    d.zone == ZoneKind::Library
                        && format!("{:?}", d.position) == format!("{p:?}")
                        && d.controller.is_none()
                        && !d.face_down
                };
                if !(same(a)
                    && same(b)
                    && lib(x, LibraryPosition::FromTop(1))
                    && lib(y, LibraryPosition::Bottom))
                {
                    return None;
                }
                let t = self.sel(o, Case::Obj);
                let poss = super::nouns::possessive(&t);
                Some(format!(
                    "{{alt:the owner of {t}|{poss} owner}} puts it into their library second from the top or on the bottom"
                ))
            }
            // "Each opponent who has three or more poison counters loses 3 life."
            Effect::ForEachPlayer { who, effect } => {
                let Effect::AsPlayer {
                    who: PlayerRef::Iterated,
                    effect: inner,
                } = effect.as_ref()
                else {
                    return None;
                };
                let Effect::If {
                    cond: Condition::Compare(Value::PlayerCounters(PlayerRef::You, k), c, n),
                    then,
                    otherwise,
                } = inner.as_ref()
                else {
                    return None;
                };
                if !matches!(otherwise.as_ref(), Effect::Noop) {
                    return None;
                }
                let n = match n {
                    Value::Const(k) => number_word(*k),
                    other => self.value(other),
                };
                let count = match c {
                    Cmp::Ge => format!("{n} or more"),
                    Cmp::Le => format!("{n} or fewer"),
                    Cmp::Eq => n,
                    _ => return None,
                };
                let w = self.player(who, Case::Subj);
                let saved = std::mem::replace(&mut self.in_as_player, true);
                let t = self.effect(then);
                self.in_as_player = saved;
                let t = t.strip_prefix("you ").unwrap_or(&t);
                if t.contains(['.', '{']) {
                    return None;
                }
                let t = format!(" {} ", super::effects::third_person(t))
                    .replace(" your ", " their ")
                    .replace(" you ", " they ");
                Some(format!(
                    "{w} who has {count} {} counters {}",
                    k.as_str(),
                    t.trim()
                ))
            }
            _ => None,
        }
    }

    /// At the start of `v`: a sequence printed as one sentence, as (text, how many
    /// instructions).
    pub(crate) fn seq_more(&mut self, v: &[Effect]) -> Option<(String, usize)> {
        if let Some(s) = self.shuffle_hands_then_draw(v) {
            return Some((s, 3));
        }
        if let Some(r) = self.another_one(v) {
            return Some(r);
        }
        if let Some(s) = self.may_reveal_top(v) {
            return Some((s, 3));
        }
        // "Move any number of +1/+1 counters from target creature onto another target
        // creature": a number chosen, then that many moved.
        if let [c, Effect::MoveCounters {
            kind: Some(k),
            n: Some(Value::Chosen),
            from,
            to,
        }, ..] = v
        {
            if chosen_number(c) {
                let f = self.sel(from, Case::Obj);
                let t = self.sel(to, Case::Obj);
                return Some((
                    format!(
                        "move any number of {} from {f} onto {t}",
                        plural(&counter_name(k))
                    ),
                    2,
                ));
            }
        }
        None
    }

    /// "Each player shuffles the cards from their hand into their library, then draws
    /// that many cards": the cards shuffled in are remembered, and each player draws as
    /// many as they own among them (CR 401.2: a card goes to its owner's library).
    fn shuffle_hands_then_draw(&mut self, v: &[Effect]) -> Option<String> {
        let [Effect::ShuffleIntoLibrary {
            what: Sel::All(Filter::And(f)),
            library: PlayerRef::EachPlayer,
        }, Effect::Store {
            var,
            sel: Sel::Var(vars::IT),
        }, Effect::ForEachPlayer {
            who: PlayerRef::EachPlayer,
            effect,
        }, ..] = v
        else {
            return None;
        };
        if !matches!(
            f.as_slice(),
            [
                Filter::InZone(ZoneKind::Hand),
                Filter::OwnedBy(PlayerRel::Any)
            ]
        ) {
            return None;
        }
        let Effect::Draw {
            who: PlayerRef::Iterated,
            n: Value::Count(Filter::And(c)),
        } = effect.as_ref()
        else {
            return None;
        };
        let counted = matches!(c.as_slice(), [Filter::InZone(ZoneKind::Library), Filter::In(s), Filter::OwnedBy(PlayerRel::Iterated)]
            if matches!(s.as_ref(), Sel::Var(x) if x == var));
        counted.then(|| {
            "each player shuffles the cards from their hand into their library, then draws that many cards"
                .to_string()
        })
    }

    /// "Put a +1/+1 counter on ~. Then if ..., put another +1/+1 counter on ~", "Draw a
    /// card. Draw another card if ...": the same instruction again, under a condition.
    fn another_one(&mut self, v: &[Effect]) -> Option<(String, usize)> {
        let [first, Effect::If {
            then, otherwise, ..
        }, ..] = v
        else {
            return None;
        };
        if !matches!(otherwise.as_ref(), Effect::Noop) {
            return None;
        }
        let again = match (first, then.as_ref()) {
            (
                Effect::AddCounters {
                    what: a,
                    kind: k,
                    n: Value::Const(1),
                },
                Effect::AddCounters {
                    what: b,
                    kind: l,
                    n: Value::Const(1),
                },
            ) if k == l && format!("{a:?}") == format!("{b:?}") => {
                Some(format!("put a {} on", counter_name(k)))
            }
            (
                Effect::Draw {
                    who: a,
                    n: Value::Const(1),
                },
                Effect::Draw {
                    who: b,
                    n: Value::Const(1),
                },
            ) if format!("{a:?}") == format!("{b:?}") => Some("draw a card".to_string()),
            _ => None,
        }?;
        let one = self.effect(first);
        let two = self.effect(&v[1]);
        let another = again.replacen(" a ", " {alt:another|a} ", 1);
        if !two.contains(&again) {
            return None;
        }
        let two = two.replacen(&again, &another, 1);
        Some((format!("{one}. {two}"), 2))
    }

    /// A permanent you control tapped for mana, as "you tap [noun]" (only a permanent's
    /// controller can activate its mana abilities, CR 602.2): the noun without "you
    /// control".
    pub(crate) fn yours_tapped(&mut self, filter: &Filter) -> Option<String> {
        let Filter::And(v) = filter else {
            return None;
        };
        if !v
            .iter()
            .any(|x| matches!(x, Filter::ControlledBy(PlayerRel::You)))
        {
            return None;
        }
        let rest = Filter::and(
            v.iter()
                .filter(|x| !matches!(x, Filter::ControlledBy(PlayerRel::You)))
                .cloned()
                .collect(),
        );
        Some(self.noun_det(&rest, super::nouns::Det::A))
    }

    /// A condition worded as a whole, before the general wording.
    pub(crate) fn condition_more(&mut self, c: &Condition) -> Option<String> {
        // "if you chose a creature other than ~ as your Ring-bearer": the creature chosen
        // as the Ring tempted you is the event's object (CR 701.54a).
        if self.extra.ring_tempts {
            if let Condition::And(v) = c {
                if let [Condition::SelNonEmpty(Sel::TriggerObject), Condition::Not(n)] =
                    v.as_slice()
                {
                    if matches!(
                        n.as_ref(),
                        Condition::SelMatches(Sel::TriggerObject, Filter::Source)
                    ) {
                        let m = self.me();
                        return Some(format!(
                            "you chose a creature other than {m} as your Ring-bearer"
                        ));
                    }
                }
            }
        }
        None
    }

    /// "You may reveal the top card of your library. If a land card is revealed this way,
    /// put it onto the battlefield tapped.": "it" is the card only if it was revealed.
    fn may_reveal_top(&mut self, v: &[Effect]) -> Option<String> {
        let [Effect::Store {
            var: vars::IT,
            sel: Sel::None,
        }, may @ Effect::May {
            who: PlayerRef::You,
            effect,
        }, Effect::If {
            cond: Condition::SelMatches(Sel::Var(vars::IT), f),
            then,
            otherwise,
        }, ..] = v
        else {
            return None;
        };
        let Effect::Dig {
            n: Value::Const(1),
            reveal: true,
            take: Value::Const(0),
            ..
        } = effect.as_ref()
        else {
            return None;
        };
        if !matches!(otherwise.as_ref(), Effect::Noop) {
            return None;
        }
        let m = self.effect(may);
        let n = self.noun_det(f, super::nouns::Det::A);
        let t = self.effect(then);
        Some(format!("{m}. If {n} is revealed this way, {t}"))
    }
}
