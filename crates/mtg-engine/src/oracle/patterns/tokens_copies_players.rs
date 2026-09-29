//! Other players creating token copies: "Each player other than its controller creates a
//! token that's a copy of it." (Fractured Identity), "each opponent creates a token
//! that's a copy of ...". Each of those players creates the token, and so controls it
//! (CR 111.2).

use super::oracle_hardening_referents::is_no_referent;
use super::tokens_copies_copy::token_copy_with_exceptions;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::patterns::EffectPattern;
use crate::oracle::phrases::*;

fn players_create_token_copy(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (who, rest) = if let Some(r) = l.strip_prefix("each player other than its controller ") {
        if is_no_referent(&b.it) {
            return None;
        }
        let controller = PlayerRef::ControllerOf(Box::new(b.it.clone()));
        (
            PlayerRef::Each(PlayerFilter::Not(Box::new(PlayerFilter::Ref(Box::new(
                controller,
            ))))),
            r,
        )
    } else if let Some(r) = l.strip_prefix("each opponent ") {
        (PlayerRef::EachOpponent, r)
    } else if let Some(r) = l.strip_prefix("each other player ") {
        (PlayerRef::EachOtherPlayer, r)
    } else {
        return None;
    };
    let r = rest.strip_prefix("creates ")?;
    let Effect::CreateTokenCopy {
        of,
        count,
        tapped,
        attacking,
        mods,
        ..
    } = token_copy_with_exceptions(&format!("create {r}"), b)?
    else {
        return None;
    };
    Some(Effect::ForEachPlayer {
        who,
        effect: Box::new(Effect::CreateTokenCopy {
            of,
            count,
            controller: PlayerRef::Iterated,
            tapped,
            attacking,
            mods,
        }),
    })
}

inventory::submit! { EffectPattern { name: "tokens_copies: other players create token copies", priority: 105, parse: players_create_token_copy } }
