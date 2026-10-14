//! Replacement effects whose event is qualified by a condition or a cause (CR 614.1):
//! "If a spell or ability an opponent controls causes you to discard ~, put it onto the
//! battlefield instead of putting it into your graveyard", "If a source would deal 4 or
//! more damage to a permanent or player, that source deals 3 damage to that permanent or
//! player instead", "If a nontoken creature would enter and it wasn't cast, exile it
//! instead", "Creatures played by your opponents enter tapped", "If you would begin your
//! draw step, you may skip that step instead", "The first time you would draw a card each
//! turn, instead ...".

use super::nouns::Det;
use super::*;

/// "it wasn't cast": the object entering isn't a spell that was cast (a permanent spell
/// resolving enters from the stack, CR 608.3).
fn is_not_cast(c: &Condition) -> bool {
    matches!(c, Condition::Not(inner) if is_cast(inner))
}

/// "it was cast": the object entering is a spell on the stack that was cast.
fn is_cast(c: &Condition) -> bool {
    matches!(c, Condition::SelMatches(Sel::TriggerObject, Filter::And(v))
        if matches!(v.as_slice(), [Filter::InZone(ZoneKind::Stack), Filter::Not(n)]
            if matches!(n.as_ref(), Filter::Custom(x) if x == crate::kw::spell_not_cast::NOT_CAST)))
}

/// "it was played": cast, or played as a land (CR 305.1).
fn is_played(c: &Condition) -> bool {
    matches!(c, Condition::Or(v) if v.len() == 2 && is_cast(&v[0])
        && matches!(&v[1], Condition::Not(n)
            if matches!(n.as_ref(), Condition::Custom(x) if x == crate::kw::trigger_event_causes::NOT_PLAYED)))
}

impl Renderer<'_> {
    /// The replacement effect, if it's one this module words.
    pub(crate) fn qualified_replacement(&mut self, def: &ReplacementDef) -> Option<String> {
        use ReplacementAction as A;
        use ReplacementEvent as E;
        match (&def.event, &def.action) {
            (
                E::DiscardCausedBy { who, filter, by },
                act @ (A::MoveInstead(d) | A::RevealAndMoveInstead(d)),
            ) => {
                let by = match by {
                    PlayerRel::Any => String::new(),
                    PlayerRel::Opponent => " an opponent controls".into(),
                    PlayerRel::You => " you control".into(),
                    r => format!(" {}", self.controls_phrase(*r, Num::One)),
                };
                let w = self.player_filter_object(who);
                let c = match filter {
                    Filter::Source => self.me(),
                    Filter::Any => "a card".to_string(),
                    f => self.noun_det(f, Det::A),
                };
                // The card is put there by its owner, who discarded it (CR 110.2a).
                let dest = self
                    .destination_phrase(d, false, false)
                    .replace(" under its owner's control", "");
                let may = if def.optional { "you may " } else { "" };
                // "reveal that card and put it on top of your library" (CR 701.20a).
                let reveal = if matches!(act, A::RevealAndMoveInstead(_)) {
                    "reveal that card and "
                } else {
                    ""
                };
                Some(format!(
                    "if a spell or ability{by} causes {w} to discard {c}, {may}{reveal}put it {dest} {{alt:instead of putting it into your graveyard|instead of putting it anywhere else|instead}}"
                ))
            }
            (E::Where { event, cond }, action) => {
                self.qualified_where_replacement(event, cond, action)
            }
            // "Prevent all combat damage that would be dealt by creatures other than target
            // creature this turn": the target is named here first.
            (
                E::Damage {
                    source: Filter::And(v),
                    to_players: Some(PlayerFilter::Any),
                    to_objects: Some(Filter::Any),
                    combat_only,
                },
                A::Prevent,
            ) => {
                let new_target = |x: &Filter| -> Option<u8> {
                    match x {
                        Filter::Not(n) => match n.as_ref() {
                            Filter::In(s) => match s.as_ref() {
                                Sel::Target(i) => Some(*i),
                                _ => None,
                            },
                            _ => None,
                        },
                        _ => None,
                    }
                };
                let i = v.iter().find_map(new_target)?;
                if self.introduced.get(i as usize).copied().unwrap_or(true) {
                    return None;
                }
                let rest = Filter::and(
                    v.iter()
                        .filter(|x| new_target(x).is_none())
                        .cloned()
                        .collect(),
                );
                let n = self.noun(&rest, Num::Many);
                let t = self.target_mention(i, super::players::Case::Obj);
                let kind = if *combat_only {
                    "combat damage"
                } else {
                    "damage"
                };
                Some(format!(
                    "prevent all {kind} that would be dealt by {n} other than {t}"
                ))
            }
            // "If you would begin your draw step, you may skip that step instead. If you
            // do, you gain 2 life."
            (E::SkipStep { step, whose }, A::Instead(e)) if def.optional => {
                let s = match step {
                    StepKind::Untap => "untap step",
                    StepKind::Upkeep => "upkeep step",
                    StepKind::Draw => "draw step",
                    StepKind::Main => "main phase",
                    StepKind::Combat => "combat phase",
                    StepKind::End => "end step",
                    StepKind::Turn => "turn",
                };
                let w = self.rel_subject(*whose);
                let p = self.rel_possessive(*whose, Num::One);
                let e = self.effect(e);
                let that = if *step == StepKind::Turn {
                    "that turn"
                } else if *step == StepKind::Main || *step == StepKind::Combat {
                    "that phase"
                } else {
                    "that step"
                };
                Some(format!(
                    "if {w} would begin {p} {s}, {w} may skip {that} instead. if {w} do, {e}"
                ))
            }
            _ => None,
        }
    }

    fn qualified_where_replacement(
        &mut self,
        event: &ReplacementEvent,
        cond: &Condition,
        action: &ReplacementAction,
    ) -> Option<String> {
        use ReplacementAction as A;
        use ReplacementEvent as E;
        match (event, cond, action) {
            // "If a source would deal 3 or more damage to you, it deals 2 damage to you
            // instead": the damage becomes that amount (the event's amount minus the
            // difference).
            (
                E::Damage {
                    source,
                    to_players,
                    to_objects,
                    combat_only: false,
                },
                Condition::Compare(Value::EventAmount, cmp @ (Cmp::Ge | Cmp::Le), Value::Const(n)),
                act,
            ) => {
                let src = match source {
                    Filter::Any => "{alt:a source|any source}".to_string(),
                    Filter::Source => self.me(),
                    other => {
                        let saved = self.default_head;
                        self.default_head = Some("source");
                        let s = self.noun_det(other, Det::A);
                        self.default_head = saved;
                        // "an instant or sorcery source".
                        if s.split(' ').any(|w| w == "source" || w == "sources") {
                            s
                        } else {
                            format!("{s} source")
                        }
                    }
                };
                let (to, to_that) = match (to_players, to_objects) {
                    (Some(PlayerFilter::Any), Some(Filter::Any)) => (
                        "a permanent or player".to_string(),
                        "that permanent or player".to_string(),
                    ),
                    (Some(PlayerFilter::You), None) => ("you".to_string(), "you".to_string()),
                    (None, Some(Filter::Source)) => {
                        let m = self.me();
                        (m.clone(), m)
                    }
                    _ => return None,
                };
                let amount = if *cmp == Cmp::Ge {
                    format!("{} or more", number_word(*n))
                } else {
                    format!("{} or less", number_word(*n))
                };
                let then = match act {
                    A::Subtract(Value::Diff(a, b))
                        if matches!(a.as_ref(), Value::EventAmount)
                            && matches!(b.as_ref(), Value::Const(_)) =>
                    {
                        let Value::Const(k) = b.as_ref() else {
                            return None;
                        };
                        format!(
                            "{{alt:it|that source}} deals {} damage to {to_that} instead",
                            number_word(*k)
                        )
                    }
                    A::Prevent => "prevent that damage".to_string(),
                    _ => return None,
                };
                Some(format!(
                    "if {src} would deal {amount} damage to {to}, {then}"
                ))
            }
            // "If a nontoken creature would enter and it wasn't cast, exile it instead."
            (E::EntersBattlefield(f), c, A::MoveInstead(d)) if is_not_cast(c) => {
                let n = self.noun_det(f, Det::A);
                let then = self.replacement_then_move(d);
                Some(format!("if {n} would enter and it wasn't cast, {then}"))
            }
            // "Creatures played by your opponents enter tapped."
            (E::EntersBattlefield(f), c, A::EnterTapped) if is_played(c) => {
                let Filter::And(v) = f else {
                    return None;
                };
                let by = v.iter().find_map(|x| match x {
                    Filter::ControlledBy(r) => Some(*r),
                    _ => None,
                })?;
                let rest = Filter::and(
                    v.iter()
                        .filter(|x| !matches!(x, Filter::ControlledBy(_)))
                        .cloned()
                        .collect(),
                );
                let n = self.noun(&rest, Num::Many);
                let p = match by {
                    PlayerRel::Opponent => "your opponents".to_string(),
                    PlayerRel::You => "you".to_string(),
                    r => self.rel_object(r),
                };
                Some(format!("{n} played by {p} enter tapped"))
            }
            // "The first time you would draw a card each turn, instead ...".
            (E::Draw(p), Condition::Custom(n), A::Instead(e))
                if n == crate::kw::first_draw_each_turn::FIRST_DRAW =>
            {
                let w = match p {
                    PlayerFilter::You => "you".to_string(),
                    other => self.player_filter_object(other),
                };
                let e = self.effect(e);
                Some(format!(
                    "the first time {w} would draw a card each turn, instead {e}"
                ))
            }
            _ => None,
        }
    }

    /// "exile it instead", "put it into its owner's graveyard instead".
    fn replacement_then_move(&mut self, d: &Destination) -> String {
        if d.zone == ZoneKind::Exile {
            return "exile it instead".into();
        }
        let dest = self.destination_phrase(d, false, false);
        format!("put it {dest} instead")
    }
}
