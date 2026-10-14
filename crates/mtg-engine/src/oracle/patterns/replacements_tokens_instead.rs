//! "If one or more creature tokens would be created under your control, that many 4/4
//! white Angel creature tokens with flying and vigilance are created instead." (Divine
//! Visitation, Draconic Visitor): the tokens the event creates have the described
//! characteristics instead of their own (CR 614.1a; `ReplacementAction::TokensInstead`).
//! Whether the tokens match is decided by the characteristics they'd be created with,
//! before continuous effects apply to them (CR 701.7b; Divine Visitation's rulings).

use super::StaticPattern;
use crate::ability::*;
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;

fn tokens_instead(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let l = end(l);
    let r = l.strip_prefix("if one or more ")?;
    let (kind, r) = r.split_once(" would be created under your control, that many ")?;
    // "4/4 white Angel creature tokens with flying and vigilance": one token's
    // description.
    let desc = r.strip_suffix(" are created instead")?;
    if !desc.contains(" tokens") {
        return None;
    }
    let desc = desc.replacen(" tokens", " token", 1);
    let event = if kind == "tokens" {
        ReplacementEvent::CreateTokens(PlayerFilter::You)
    } else {
        let (tokens, plural, tail) = parse_object_phrase(kind)?;
        if !plural || !end(tail).is_empty() || matches!(tokens, Filter::Any) {
            return None;
        }
        ReplacementEvent::CreateTokensMatching {
            who: PlayerFilter::You,
            tokens,
        }
    };
    let spec = crate::oracle::effects::parse_token_description(&desc)?;
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Replacement(
            ReplacementDef {
                event,
                action: ReplacementAction::TokensInstead { spec },
                self_replacement: false,
                optional: false,
            },
        ))),
        text,
    )])
}

inventory::submit! { StaticPattern { name: "replacements: that many other tokens are created instead", priority: 80, parse: tokens_instead } }
