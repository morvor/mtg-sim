//! "As ~ enters, choose a creature you control it could be attached to. If you do, it
//! enters attached to that creature." (Grifter's Blade): an Equipment put onto the
//! battlefield attached to an object chosen as it enters (CR 301.5b, 614.12). If the
//! chosen object can't be equipped by it when it enters, it enters unattached (CR 301.5e,
//! `attach::entry_attachment`). The text is parsed in
//! `oracle/patterns/as_enters_actions.rs`.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::{vars, Var};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;

/// The object or player chosen as it enters.
pub const ENTRY_HOST: Var = vars::USER + 52010;

/// `Effect::Custom`: the entering permanent enters attached to [`ENTRY_HOST`], if one was
/// chosen.
pub const ENTER_ATTACHED_TO_CHOSEN: &str = "enter attached to the chosen object";

/// `Effect::Custom`: the entering permanent is put into its owner's graveyard instead of
/// onto the battlefield (Frankenstein's Monster: "If you can't, put ~ into its owner's
/// graveyard instead of onto the battlefield").
pub const ENTER_GRAVEYARD_INSTEAD: &str = "put the entering permanent into its owner's graveyard instead";

pub struct EntersAttached;

impl KeywordRules for EntersAttached {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_effect(&self, _g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name == ENTER_GRAVEYARD_INSTEAD {
            if let Some(em) = ctx.entering.as_mut() {
                em.graveyard_instead = true;
            }
            return true;
        }
        if name != ENTER_ATTACHED_TO_CHOSEN {
            return false;
        }
        let host = ctx.vars.get(&ENTRY_HOST).and_then(|v| v.first().copied());
        if let (Some(h), Some(em)) = (host, ctx.entering.as_mut()) {
            em.attach_to = Some(h);
        }
        true
    }
}

inventory::submit! { KeywordRegistration(&EntersAttached) }
