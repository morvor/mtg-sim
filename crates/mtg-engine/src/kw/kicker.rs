//! CR 702.33 Kicker, multikicker, and sticker kicker: optional additional costs
//! (CR 601.2b, 601.2f–h).
//!
//! * "Kicker [cost]" (CR 702.33a); "Kicker [cost 1] and/or [cost 2]" is two kicker
//!   abilities (CR 702.33b; the second cost is kept in [`Keyword::costs`]). Paying a kicker
//!   cost records "kicker" and "kicker [cost]" in the spell's `CastInfo::paid`, which the
//!   linked "if it was kicked (with its [A] kicker)" abilities check (CR 702.33e–f).
//! * "Multikicker [cost]" may be paid any number of times (CR 702.33c).
//! * "Sticker kicker [cost]" means "Kicker [cost]" and "As an additional cost to cast this
//!   spell, if it's kicked, you get a ticket counter and you may put a sticker on this
//!   spell" (CR 702.33h). It's recorded as "sticker kicker": abilities linked to a card's
//!   own printed kicker don't see it (CR 702.33e).
//! * Paying any of these kicks the spell (CR 702.33d): "kicked" is recorded too, which
//!   is what other objects' abilities that care about kicked spells check ("Whenever you
//!   cast a kicked spell" sees a sticker-kicked spell).
//!
//! Targets of a part of a spell that has its effect only if it was kicked are chosen only
//! if it was (CR 702.33g; see `TargetSpec::condition`).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::types::*;
use smol_str::SmolStr;

pub struct Kicker;

/// The optional cost names recorded in `CastInfo::paid`.
pub const KICKER: &str = "kicker";
pub const MULTIKICKER: &str = "multikicker";
pub const STICKER_KICKER: &str = "sticker kicker";
/// Recorded once when any kicker, multikicker, or sticker kicker cost was paid: the spell
/// was kicked (CR 702.33d), as seen by other objects ("a kicked spell").
pub const KICKED: &str = "kicked";

/// Records in `paid` that the spell was kicked if any of its kicker costs was paid
/// (CR 702.33d).
pub fn record_kicked(paid: &mut Vec<SmolStr>) {
    let kicked = paid
        .iter()
        .any(|n| matches!(n.as_str(), KICKER | MULTIKICKER | STICKER_KICKER));
    if kicked && !paid.iter().any(|n| n == KICKED) {
        paid.push(KICKED.into());
    }
}

/// The additional part of a sticker kicker cost: "you get a ticket counter and you may put
/// a sticker on this spell" (CR 702.33h, 123.3).
fn sticker_kicker_part() -> CostPart {
    CostPart::Effect(Box::new(Effect::Seq(vec![
        Effect::AddPlayerCounters {
            who: PlayerRef::You,
            kind: counters::TICKET.into(),
            n: Value::c(1),
        },
        Effect::May {
            who: PlayerRef::You,
            effect: Box::new(Effect::PutSticker {
                who: PlayerRef::You,
                what: Sel::This,
                kind: None,
                max_ticket: None,
                free: false,
            }),
        },
    ])))
}

impl KeywordRules for Kicker {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Kicker]
    }

    fn optional_costs(
        &self,
        _g: &Game,
        _spell: ObjectId,
        kw: &Keyword,
    ) -> Vec<(SmolStr, Cost, bool)> {
        let text = kw.text.as_deref().unwrap_or("").to_lowercase();
        let mut out = Vec::new();
        if let Some(c) = &kw.cost {
            if text.starts_with(MULTIKICKER) {
                out.push((MULTIKICKER.into(), c.clone(), true));
            } else if text.starts_with(STICKER_KICKER) {
                let mut c = c.clone();
                c.parts.push(sticker_kicker_part());
                out.push((STICKER_KICKER.into(), c, false));
            } else {
                out.push((KICKER.into(), c.clone(), false));
            }
        }
        for c in &kw.costs {
            out.push((KICKER.into(), c.clone(), false));
        }
        out
    }
}

inventory::submit! { KeywordRegistration(&Kicker) }
