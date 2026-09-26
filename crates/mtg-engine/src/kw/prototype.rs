//! CR 702.160 Prototype, and prototype cards (CR 718).
//!
//! "Prototype [mana cost] — [power]/[toughness]" gives the card alternative
//! characteristics: a player casting it may cast it as a prototyped spell
//! (`CastMethod::Keyword(Prototype)`, CR 718.3, 702.160a). That isn't casting it for an
//! alternative cost: the spell's mana cost is the prototype's (CR 718.3a).
//!
//! * While casting it, only the prototype's power, toughness and mana cost are evaluated
//!   (CR 718.3a): the characteristics a card would have as a spell cast this way
//!   ([`KeywordRules::spell_text_change`] with the "prototype" payment tag).
//! * The prototyped spell and the permanent it becomes have only those power, toughness
//!   and mana cost characteristics, and the colors of that mana cost (CR 718.3b). They're
//!   its copiable values: a copy of the spell or the permanent has them too (CR 718.2a,
//!   718.3c, 718.3d). Everything else is the card's (CR 718.5).
//! * Elsewhere, or when not cast prototyped, it has only its normal characteristics
//!   (CR 718.4): a new object after a zone change doesn't remember how it was cast.

use super::{KeywordRegistration, KeywordRules};
use crate::casting::CastOption;
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::mana::ManaCost;
use crate::object::*;
use crate::types::*;
use smol_str::SmolStr;

/// The name recorded in `CastInfo::paid` for a spell cast as a prototyped spell.
pub const PROTOTYPE: &str = "prototype";

/// The prototype's mana cost, power and toughness ("Prototype {1}{B} — 1/1").
pub fn prototype_values(kw: &Keyword) -> Option<(ManaCost, i32, i32)> {
    let mana = kw.cost.as_ref()?.mana.clone()?;
    let (p, t) = kw.text.as_deref()?.split_once('/')?;
    Some((mana, p.trim().parse().ok()?, t.trim().parse().ok()?))
}

/// Gives `chars` the prototype's alternative characteristics (CR 718.3b).
fn apply(chars: &mut Characteristics) {
    let Some(values) = chars
        .keywords()
        .find(|k| k.kind == KeywordKind::Prototype)
        .and_then(prototype_values)
    else {
        return;
    };
    let (mana, p, t) = values;
    // Its color is determined by that mana cost (CR 105.2, 718.3b).
    if chars.color_indicator.is_none() {
        chars.colors = mana.colors();
    }
    chars.mana_cost = Some(mana);
    chars.power = Some(p);
    chars.toughness = Some(t);
}

/// Whether the object was cast as a prototyped spell: the spell, a copy of it, or the
/// permanent either became (CR 718.3b–718.3d).
pub fn is_prototyped(o: &GameObject) -> bool {
    let paid = match o.zone {
        Zone::Stack => o.stack.as_deref().map(|s| &s.cast.paid),
        Zone::Battlefield => o.cast.as_deref().map(|c| &c.paid),
        _ => None,
    };
    paid.is_some_and(|v| v.iter().any(|p| p == PROTOTYPE))
}

/// Layer 1: prototyped spells and permanents have the prototype's power, toughness, mana
/// cost and color as their own characteristics, part of the copiable values (CR 718.2a,
/// 718.3b). Called as characteristics are computed, before copy effects.
pub fn prototyped_characteristics(g: &mut Game, live: &[ObjectId]) {
    for id in live {
        let o = &g.objects[id.0 as usize];
        if o.face_down || !is_prototyped(o) {
            continue;
        }
        let mut c = std::mem::take(&mut g.objects[id.0 as usize].chars);
        apply(&mut c);
        g.objects[id.0 as usize].chars = c;
    }
}

pub struct Prototype;

impl KeywordRules for Prototype {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Prototype]
    }

    /// CR 718.3: cast it normally or as a prototyped spell, from any zone it could be
    /// cast from.
    fn cast_options(&self, g: &Game, p: PlayerId, card: ObjectId, kw: &Keyword) -> Vec<CastOption> {
        if prototype_values(kw).is_none() {
            return vec![];
        }
        let o = g.obj(card);
        let mut opt = CastOption::normal(FaceState::Front);
        opt.method = CastMethod::Keyword(KeywordKind::Prototype);
        opt.tag = Some(PROTOTYPE);
        if o.zone != Zone::Hand(p) {
            // A permission to cast it looks at the characteristics it would have
            // (CR 601.3e, 718.3a).
            let chars = g.option_characteristics(card, &opt);
            if !g.permitted_cards(p).contains(&card) || !g.permission_allows(p, card, &chars, false)
            {
                return vec![];
            }
        }
        vec![opt]
    }

    /// CR 718.3a: the characteristics evaluated while casting it this way.
    fn spell_text_change(
        &self,
        _g: &Game,
        _spell: ObjectId,
        _kw: &Keyword,
        paid: &[SmolStr],
        chars: &mut Characteristics,
    ) {
        if paid.iter().any(|p| p == PROTOTYPE) {
            apply(chars);
        }
    }
}

inventory::submit! { KeywordRegistration(&Prototype) }
