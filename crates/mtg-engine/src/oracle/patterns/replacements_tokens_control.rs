//! "When ~ enters, each token that would be created under an opponent's control this
//! turn is created under your control instead." (Crafty Cutpurse): a replacement effect
//! for the rest of the turn that changes under whose control tokens are created
//! (CR 614.1a, 111.2). It applies before effects that modify how the tokens are created
//! (Divine Visitation's rulings; see `replacement.rs`).

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;

fn parse(l: &str, _b: &mut Builder) -> Option<Effect> {
    (end(l)
        == "each token that would be created under an opponent's control this turn is created under your control instead")
        .then(|| Effect::AddReplacement {
            def: ReplacementDef {
                event: ReplacementEvent::CreateTokens(PlayerFilter::Opponent),
                action: ReplacementAction::EnterUnderControl(PlayerRef::You),
                self_replacement: false,
                optional: false,
            },
            duration: Duration::EndOfTurn,
            uses: None,
        })
}

inventory::submit! { EffectPattern { name: "replacements: tokens created under an opponent's control are created under yours", priority: 150, parse } }
