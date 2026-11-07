//! Replacement effects the generic replacement wording doesn't cover: "If your life total
//! would be reduced to 0 or less, instead transform ~ and your life total becomes 3"
//! (CR 119.4, 614.1a), "If a source controlled by one of the chosen players would deal
//! damage to the other chosen player or a permanent that player controls, that source
//! deals double that damage ..." (CR 614.1a).

use super::*;

/// The condition "the source is controlled by one of the chosen players and what it
/// would damage is the other chosen player or a permanent that player controls".
fn between_chosen_players() -> Condition {
    let chosen_other = |who: PlayerRef| {
        PlayerFilter::And(vec![
            PlayerFilter::ChosenPlayers,
            PlayerFilter::Not(Box::new(PlayerFilter::Ref(Box::new(who)))),
        ])
    };
    let source_controller = PlayerRef::ControllerOf(Box::new(Sel::TriggerOtherObject));
    let object_controller = PlayerRef::ControllerOf(Box::new(Sel::TriggerObject));
    Condition::Or(vec![
        Condition::And(vec![
            Condition::PlayerMatches(PlayerRef::TriggerPlayer, PlayerFilter::ChosenPlayers),
            Condition::PlayerMatches(
                source_controller.clone(),
                chosen_other(PlayerRef::TriggerPlayer),
            ),
        ]),
        Condition::And(vec![
            Condition::SelNonEmpty(Sel::TriggerObject),
            Condition::PlayerMatches(object_controller.clone(), PlayerFilter::ChosenPlayers),
            Condition::PlayerMatches(source_controller, chosen_other(object_controller)),
        ]),
    ])
}

impl Renderer<'_> {
    pub(crate) fn replacement_named(&mut self, def: &ReplacementDef) -> Option<String> {
        let ReplacementEvent::Where { event, cond } = &def.event else {
            return None;
        };
        match (event.as_ref(), &def.action) {
            // "If your life total would be reduced to 0 or less, instead ...": a life loss
            // that would leave it at 0 or less (CR 119.4).
            (ReplacementEvent::LoseLife(PlayerFilter::You), ReplacementAction::Instead(e)) => {
                let Condition::Compare(Value::Diff(life, amount), Cmp::Le, Value::Const(0)) =
                    cond
                else {
                    return None;
                };
                if !matches!(life.as_ref(), Value::LifeTotal(PlayerRef::You))
                    || !matches!(amount.as_ref(), Value::EventAmount)
                {
                    return None;
                }
                let Effect::Seq(v) = e.as_ref() else {
                    return None;
                };
                let [a, b, rest @ ..] = v.as_slice() else {
                    return None;
                };
                let a = self.effect(a);
                let b = self.effect(b);
                let mut s = format!(
                    "if your life total would be reduced to 0 or less, instead {a} and {b}"
                );
                for r in rest {
                    s.push_str(&format!(". {}", self.then_if_not_transformed(r)?));
                }
                Some(s)
            }
            (
                ReplacementEvent::Damage {
                    source: Filter::Any,
                    to_players: Some(PlayerFilter::Any),
                    to_objects: Some(Filter::Permanent),
                    combat_only: false,
                },
                ReplacementAction::Multiply(2),
            ) if format!("{cond:?}") == format!("{:?}", between_chosen_players()) => Some(
                "if a source controlled by one of the chosen players would deal damage to the other chosen player or a permanent that player controls, that source deals double that damage to that player or permanent instead"
                    .into(),
            ),
            _ => None,
        }
    }

    /// "Then if ~ didn't transform this way, you lose the game."
    fn then_if_not_transformed(&mut self, e: &Effect) -> Option<String> {
        let Effect::If {
            cond: Condition::Not(c),
            then,
            otherwise,
        } = e
        else {
            return None;
        };
        let Condition::SelMatches(Sel::This, Filter::Custom(n)) = c.as_ref() else {
            return None;
        };
        if n != crate::transform_rules::TRANSFORMED || !matches!(otherwise.as_ref(), Effect::Noop) {
            return None;
        }
        let m = self.me();
        let t = self.effect(then);
        Some(format!("then if {m} didn't transform this way, {t}"))
    }
}
