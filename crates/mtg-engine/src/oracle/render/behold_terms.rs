//! Behold (CR 701.4): "Behold a [quality]" means "Reveal a [quality] card from your hand
//! or choose a [quality] permanent you control on the battlefield." Older cards print the
//! long form ("As an additional cost to cast this spell, you may reveal a Dragon card
//! from your hand or choose a Dragon you control.").

use super::nouns::Det;
use super::*;

/// The quality an additional cost of this face beholds, if it has one.
pub(crate) fn beheld(abilities: &[Ability]) -> Option<Filter> {
    abilities.iter().find_map(|a| {
        let AbilityKind::Static(s) = &a.kind else {
            return None;
        };
        let StaticEffect::CostModifier(CostModifier {
            applies_to: CostTarget::ThisSpell,
            change:
                CostChange::OptionalAdditionalCost { cost, .. } | CostChange::AdditionalCost(cost),
            ..
        }) = &s.effect
        else {
            return None;
        };
        cost.parts.iter().find_map(|p| match p {
            CostPart::Effect(e) => beheld_in(e),
            _ => None,
        })
    })
}

fn beheld_in(e: &Effect) -> Option<Filter> {
    match e {
        Effect::KeywordAction {
            action: KeywordAction::Behold,
            what: Sel::All(f),
            n: Value::Const(1),
            ..
        } => Some(f.clone()),
        Effect::Seq(v) => v.iter().find_map(beheld_in),
        _ => None,
    }
}

impl Renderer<'_> {
    /// "a Dragon card" and "a Dragon" for what is beheld.
    fn beheld_nouns(&mut self, f: &Filter) -> (String, String) {
        let card = self.noun_det(&Filter::and(vec![f.clone(), Filter::Card]), Det::A);
        let perm = self.noun_det(f, Det::A);
        (card, perm)
    }

    /// "behold a Dragon", in either long form (CR 701.4a).
    pub(crate) fn behold_forms(&mut self, f: &Filter) -> String {
        let (card, perm) = self.beheld_nouns(f);
        format!(
            "{{alt:behold {perm}|reveal {card} from your hand or choose {perm} you control|choose {perm} you control or reveal {card} from your hand}}"
        )
    }

    /// "the behold cost was paid": "you revealed a Dragon card or chose a Dragon as you
    /// cast ~".
    pub(crate) fn behold_paid(&mut self) -> Option<String> {
        let f = self.info.behold.clone()?;
        let (card, perm) = self.beheld_nouns(&f);
        let m = self.me();
        Some(format!(
            "{{alt:the behold cost was paid|you revealed {card} or chose {perm} as you cast {m}|{perm} was beheld}}"
        ))
    }

    /// The power of the object beheld as the spell was cast (`choices.rs`).
    pub(crate) fn beheld_power(&mut self) -> String {
        let q = match self.info.behold.clone() {
            Some(f) => self.noun(&f, Num::One),
            None => "creature".into(),
        };
        format!(
            "{{alt:the power of that card or creature|the power of the {q} you chose or the card you revealed|the power of the chosen {q} or card|the power of the chosen {q}}}"
        )
    }
}
