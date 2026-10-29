//! Akroma, Vision of Ixidor: "At the beginning of each combat, until end of turn, each
//! other creature you control gets +1/+1 if it has flying, +1/+1 if it has first strike,
//! and so on for double strike, deathtouch, haste, hexproof, indestructible, lifelink,
//! menace, protection, reach, trample, vigilance, and partner."
//!
//! The bonus of each creature is determined as the ability resolves (CR 608.2h).

use super::{map_effect, parse, ManualAbility};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::partner::{partner_keyword, PartnerAbility};
use crate::kw::{KeywordRegistration, KeywordRules};

const BONUS: &str = "card:Akroma, Vision of Ixidor:one for each listed keyword it has";
const TEXT: &str = "At the beginning of each combat, until end of turn, each other creature you control gets +1/+1 if it has flying, +1/+1 if it has first strike, and so on for double strike, deathtouch, haste, hexproof, indestructible, lifelink, menace, protection, reach, trample, vigilance, and partner.";

const LISTED: [KeywordKind; 13] = [
    KeywordKind::Flying,
    KeywordKind::FirstStrike,
    KeywordKind::DoubleStrike,
    KeywordKind::Deathtouch,
    KeywordKind::Haste,
    KeywordKind::Hexproof,
    KeywordKind::Indestructible,
    KeywordKind::Lifelink,
    KeywordKind::Menace,
    KeywordKind::Protection,
    KeywordKind::Reach,
    KeywordKind::Trample,
    KeywordKind::Vigilance,
];

inventory::submit! { ManualAbility {
    card: "Akroma, Vision of Ixidor",
    face: 0,
    text: TEXT,
    build: |ctx| {
        parse(ctx, "At the beginning of each combat, until end of turn, each other creature you control gets +1/+1.")
            .iter()
            .map(|a| {
                map_effect(a, TEXT, |e| match e {
                    Effect::Modify { what, duration, .. } => Effect::Modify {
                        what,
                        mods: vec![Modification::ModifyPT(
                            Value::Custom(BONUS.into()),
                            Value::Custom(BONUS.into()),
                        )],
                        duration,
                    },
                    e => e,
                })
            })
            .collect()
    },
    reason: "+1/+1 for each keyword from a list ('and so on'): unique",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    /// One for each listed keyword the affected creature has, however many instances or
    /// variants of it; partner counts only "partner" and "partner with".
    fn custom_value(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<i64> {
        if name != BONUS {
            return None;
        }
        let it = ctx.var_objects(vars::AFFECTED).first().copied()?;
        let o = g.obj(it);
        let listed = LISTED.iter().filter(|k| o.has_keyword(**k)).count() as i64;
        let partner = o.chars.keywords().any(|k| {
            k.kind == KeywordKind::Partner
                && matches!(
                    partner_keyword(k),
                    PartnerAbility::Partner | PartnerAbility::With(_)
                )
        });
        Some(listed + partner as i64)
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
