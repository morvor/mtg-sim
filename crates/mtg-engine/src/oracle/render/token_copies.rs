//! "The first time you would create one or more tokens each turn, you may instead create
//! that many tokens that are copies of enchanted permanent." (Moonlit Meditation, Mirrormind
//! Crown, Esix, Fractal Bloom; `patterns/token_replacements_2.rs`): a replacement effect
//! for the first token creation of a turn (CR 614.1a), the tokens created instead being
//! copies of one object (CR 707.2).

use super::*;
use crate::oracle::patterns::token_replacements_2::FIRST_TOKENS;

impl Renderer<'_> {
    /// The whole replacement, or None if it isn't one of these.
    pub(crate) fn first_tokens_as_copies(&mut self, def: &ReplacementDef) -> Option<String> {
        let ReplacementEvent::Where { event, cond } = &def.event else {
            return None;
        };
        let ReplacementEvent::CreateTokens(PlayerFilter::You) = event.as_ref() else {
            return None;
        };
        let first = |c: &Condition| matches!(c, Condition::Custom(n) if n == FIRST_TOKENS);
        let when = match cond {
            c if first(c) => "each turn",
            Condition::And(v) if matches!(v.as_slice(), [a, Condition::YourTurn] if first(a)) => {
                "during each of your turns"
            }
            _ => return None,
        };
        let ReplacementAction::TokenCopiesInstead { of, choose } = &def.action else {
            return None;
        };
        let copies = match (of, choose) {
            (Some(s), None) => {
                let o = self.sel(s, super::players::Case::Obj);
                format!("create that many tokens that are copies of {o}")
            }
            // "choose a creature other than ~ and create that many tokens that are copies
            // of that creature".
            (None, Some(f)) => {
                let (kind, other) = match f {
                    Filter::And(v) if v.iter().any(|x| matches!(x, Filter::Other)) => (
                        Filter::and(
                            v.iter()
                                .filter(|x| !matches!(x, Filter::Other))
                                .cloned()
                                .collect(),
                        ),
                        format!(" other than {}", self.me()),
                    ),
                    other => (other.clone(), String::new()),
                };
                let n = self.noun_det(&kind, super::nouns::Det::A);
                format!(
                    "choose {n}{other} and create that many tokens that are copies of that creature"
                )
            }
            _ => return None,
        };
        let may = if def.optional {
            "you may instead"
        } else {
            "instead"
        };
        Some(format!(
            "the first time you would create one or more tokens {when}, {may} {copies}"
        ))
    }
}
