//! One-use damage replacement and prevention effects (CR 614.1, 615.1, 615.3): "the next
//! time that creature would deal damage this turn, prevent that damage", "the next time
//! damage would be dealt to ~ this turn, that damage is dealt to any target instead",
//! "the next time that creature would deal combat damage this turn, it deals double that
//! damage instead", with what follows the prevention ("You gain life equal to the damage
//! prevented this way", "If damage from a red source is prevented this way, ...").

use super::nouns::Det;
use super::players::Case;
use super::*;

impl Renderer<'_> {
    /// "to you", "to ~ and/or you", "to any target", "to one or more players": what the
    /// damage would be dealt to; `None` when it isn't one of these.
    pub(crate) fn damage_to(
        &mut self,
        to_players: &Option<PlayerFilter>,
        to_objects: &Option<Filter>,
    ) -> Option<String> {
        // "would deal damage to any target": one target, a player or an object.
        if let (Some(PlayerFilter::Ref(p)), Some(Filter::In(s))) = (to_players, to_objects) {
            if let (PlayerRef::Target(i), Sel::Target(j)) = (p.as_ref(), s.as_ref()) {
                if i == j {
                    let t = self.sel(&Sel::Target(*i), Case::Obj);
                    return Some(format!(" to {t}"));
                }
            }
        }
        let objects = match to_objects {
            None => None,
            Some(Filter::Any) => {
                return match to_players {
                    // Damage to anything.
                    Some(PlayerFilter::Any) => Some(String::new()),
                    _ => None,
                };
            }
            Some(Filter::Source) => Some(self.me()),
            Some(Filter::In(s)) => Some(self.sel(s, Case::Obj)),
            Some(f) => Some(self.noun_det(f, Det::Plural)),
        };
        let players = match to_players {
            None => None,
            Some(PlayerFilter::You) => Some("you".to_string()),
            // "to one or more players": a combat damage event can be dealt to several.
            Some(PlayerFilter::Any) if objects.is_none() => {
                Some("{alt:a player|one or more players}".to_string())
            }
            Some(_) => return None,
        };
        Some(match (objects, players) {
            (None, None) => String::new(),
            (Some(o), None) => format!(" to {o}"),
            (None, Some(p)) => format!(" to {p}"),
            // Either or both of them (CR 615.1a: the damage of one event).
            (Some(o), Some(p)) if p == "you" && (o == "~" || o == "~it") => {
                format!(" to {o} and/or you")
            }
            (Some(o), Some(p)) => format!(" to {p} and/or {o}"),
        })
    }

    /// What's done after damage is prevented, about that damage: "You gain life equal to
    /// the damage prevented this way", "If damage is prevented this way, create that many
    /// ... tokens", "If damage from a red source is prevented this way, ~ deals that much
    /// damage to the source's controller".
    fn prevented_this_way(&mut self, e: &Effect) -> Option<String> {
        // "from a red source": the source of the prevented damage (CR 614.1a).
        if let Effect::If {
            cond: Condition::SelMatches(Sel::TriggerOtherObject, f),
            then,
            otherwise,
        } = e
        {
            if !matches!(otherwise.as_ref(), Effect::Noop) {
                return None;
            }
            let saved = self.default_head;
            self.default_head = Some("source");
            let src = self.noun_det(f, Det::A);
            self.default_head = saved;
            let src = if src.contains("source") {
                src
            } else {
                format!("{src} source")
            };
            let t = self.prevented_result(then)?;
            return Some(format!("If damage from {src} is prevented this way, {t}"));
        }
        if let Effect::GainLife {
            who: PlayerRef::You,
            n: Value::EventAmount,
        } = e
        {
            return Some("you gain life equal to the damage prevented this way".into());
        }
        let t = self.prevented_result(e)?;
        Some(format!("If damage is prevented this way, {t}"))
    }

    /// An instruction about the damage just prevented: "you gain that much life", "~
    /// deals that much damage to the source's controller", "create that many ... tokens".
    fn prevented_result(&mut self, e: &Effect) -> Option<String> {
        Some(match e {
            Effect::GainLife {
                who: PlayerRef::You,
                n: Value::EventAmount,
            } => "you gain that much life".into(),
            Effect::DealDamage {
                source: Sel::This,
                amount: Value::EventAmount,
                to: Sel::Players(PlayerRef::ControllerOf(o)),
            } if matches!(o.as_ref(), Sel::TriggerOtherObject) => {
                let m = self.me();
                format!(
                    "{m} deals that much damage to {{alt:the source's|that source's}} controller"
                )
            }
            Effect::CreateToken {
                count: Value::EventAmount,
                ..
            } => self.effect(e),
            _ => return None,
        })
    }

    /// The instruction of a damage replacement: "prevent that damage", "it deals double
    /// that damage instead", "that damage is dealt to any target instead".
    pub(crate) fn damage_then(&mut self, a: &ReplacementAction) -> Option<String> {
        Some(match a {
            ReplacementAction::Prevent => "prevent that damage".into(),
            ReplacementAction::Multiply(2) => "it deals double that damage instead".into(),
            ReplacementAction::PreventAndThen(None, e) => {
                let t = self.prevented_this_way(e)?;
                format!("prevent that damage. {t}")
            }
            ReplacementAction::Redirect(t) => {
                let t = self.in_event_scope(|r| r.sel(t, Case::Obj));
                format!("that damage is dealt to {t} instead")
            }
            _ => return None,
        })
    }

    /// "The next time [source] would deal damage [to ...] this turn, [instruction]": a
    /// damage replacement that applies once (CR 614.1).
    pub(crate) fn next_time_damage(
        &mut self,
        def: &ReplacementDef,
        duration: &Duration,
    ) -> Option<String> {
        let ReplacementEvent::Damage {
            source,
            to_players,
            to_objects,
            combat_only,
        } = &def.event
        else {
            return None;
        };
        let when = match duration {
            Duration::EndOfTurn | Duration::ThisTurn => "this turn",
            Duration::EndOfCombat => "this combat",
            _ => return None,
        };
        let damage = if *combat_only {
            "combat damage"
        } else {
            "damage"
        };
        let src = match source {
            Filter::Any => None,
            Filter::Source => Some(self.me()),
            Filter::In(s) => Some(self.sel(s, Case::Subj)),
            f => Some(self.noun_det(f, Det::A)),
        };
        let to = self.damage_to(to_players, to_objects)?;
        let then = self.damage_then(&def.action)?;
        Some(match src {
            Some(s) => format!("the next time {s} would deal {damage}{to} {when}, {then}"),
            // "the next time damage would be dealt to ~ this turn".
            None => {
                let to = to.strip_prefix(" to ")?;
                format!("the next time {damage} would be dealt to {to} {when}, {then}")
            }
        })
    }
}

/// `s` split at the first `sep` outside `{...}`.
pub(crate) fn split_outside_braces<'a>(s: &'a str, sep: &str) -> Option<(&'a str, &'a str)> {
    let mut depth = 0i32;
    for (i, c) in s.char_indices() {
        match c {
            '{' => depth += 1,
            '}' => depth -= 1,
            _ if depth == 0 && s[i..].starts_with(sep) => {
                return Some((&s[..i], &s[i + sep.len()..]));
            }
            _ => {}
        }
    }
    None
}
