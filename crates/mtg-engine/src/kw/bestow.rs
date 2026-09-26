//! CR 702.103 Bestow: "Bestow [cost]" means "As you cast this spell, you may choose to cast
//! it bestowed. If you do, you pay [cost] rather than its mana cost." (CR 702.103a), an
//! alternative cost (CR 601.2b, 601.2f–h) from any zone the card could be cast from.
//!
//! * As a spell cast bestowed is put onto the stack it becomes an Aura enchantment with
//!   enchant creature, a bestowed Aura spell (CR 702.103b): the change is made to the
//!   spell's characteristics while its cast information says it was cast bestowed (see
//!   [`KeywordRules::spell_text_change`]), which is also how legality is checked as it's
//!   cast (only the characteristics as modified by bestow count, CR 702.103d). A copy of
//!   the spell copies how it was cast, so it's bestowed too (CR 702.103c).
//! * As it begins resolving, a bestowed Aura spell whose target is illegal ceases to be
//!   bestowed and resolves as a creature spell (CR 702.103e, [`KeywordRules::unbestow`]).
//! * The permanent it becomes is a bestowed Aura: an effect ([`is_bestowed`]) makes it an
//!   Aura enchantment with enchant creature until it ceases to be bestowed. It does when it
//!   becomes unattached, is attached to an illegal object or player (it becomes unattached
//!   instead of being put into its owner's graveyard, an exception to CR 704.5m), or phases
//!   in unattached (CR 702.103f–g); it's then the creature it would otherwise be.
//!
//! A permanent with bestow put onto the battlefield other than by resolving as a bestowed
//! Aura spell is simply a creature.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::casting::CastOption;
use crate::game::{Affected, ContinuousEffect, Game};
use crate::keywords::{Keyword, KeywordKind};
use crate::object::*;
use crate::types::*;
use smol_str::SmolStr;

/// The name recorded in `CastInfo::paid` when a spell is cast bestowed (also read by
/// `stack.rs` for CR 702.103e).
pub const BESTOW: &str = "bestow";
/// `StaticEffect::Custom`, functioning in a graveyard: "You may cast this card from your
/// graveyard using its bestow ability." (Detective's Phoenix).
pub const CAST_BESTOWED_FROM_GRAVEYARD: &str =
    "bestow:you may cast this card from your graveyard using its bestow ability";

/// Whether `card` in its owner's graveyard lets its owner cast it bestowed from there.
fn castable_bestowed_from_graveyard(g: &Game, p: PlayerId, card: ObjectId) -> bool {
    let o = g.obj(card);
    o.zone == Zone::Graveyard(p)
        && o.owner == p
        && o.chars.abilities.iter().any(|a| {
            matches!(&a.kind, AbilityKind::Static(s)
                if s.zone == FunctionZone::Graveyard
                    && matches!(&s.effect, StaticEffect::Custom(n)
                        if n.as_str() == CAST_BESTOWED_FROM_GRAVEYARD))
        })
}
/// The text of the enchant creature ability a bestowed Aura gains, marking the effect that
/// makes a permanent a bestowed Aura.
const BESTOWED: &str = "enchant creature (bestowed)";

/// The enchant creature ability bestow gives.
fn enchant_creature() -> Keyword {
    Keyword::with_filter(KeywordKind::Enchant, Filter::creature()).text(BESTOWED)
}

/// How being bestowed modifies an object: it becomes an Aura enchantment (and nothing
/// else, CR 205.1b) and gains enchant creature.
fn bestowed_mods() -> Vec<Modification> {
    vec![
        Modification::SetTypes {
            types: vec![CardType::Enchantment],
            subtypes: vec![SmolStr::new("Aura")],
        },
        Modification::AddKeyword(enchant_creature()),
    ]
}

fn is_bestow_effect(e: &ContinuousEffect) -> bool {
    e.mods.iter().any(|m| {
        matches!(m, Modification::AddKeyword(k)
            if k.kind == KeywordKind::Enchant && k.text.as_deref() == Some(BESTOWED))
    })
}

/// Whether the permanent is a bestowed Aura (CR 702.103b).
pub fn is_bestowed(g: &Game, id: ObjectId) -> bool {
    g.effects
        .iter()
        .any(|e| is_bestow_effect(e) && matches!(&e.affected, Affected::Objects(v) if v.contains(&id)))
}

/// The permanent ceases to be bestowed: the effect making it an Aura ends.
pub fn cease_to_be_bestowed(g: &mut Game, id: ObjectId) {
    let before = g.effects.len();
    g.effects.retain(|e| {
        !(is_bestow_effect(e) && matches!(&e.affected, Affected::Objects(v) if v.contains(&id)))
    });
    if g.effects.len() != before {
        g.dirty = true;
        g.log(|g| format!("{} is no longer bestowed", g.describe(id)));
    }
}

pub struct Bestow;

impl KeywordRules for Bestow {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Bestow]
    }

    fn cast_options(&self, g: &Game, p: PlayerId, card: ObjectId, kw: &Keyword) -> Vec<CastOption> {
        let o = g.obj(card);
        let Some(cost) = kw.cost.clone() else {
            return vec![];
        };
        let mut opt = CastOption::normal(FaceState::Front);
        opt.method = CastMethod::Keyword(KeywordKind::Bestow);
        opt.tag = Some(BESTOW);
        // Only from a zone the card could be cast from as the Aura spell it would be
        // (CR 702.103d): an effect letting its owner cast creature spells from elsewhere
        // doesn't let them cast it bestowed.
        if o.zone != Zone::Hand(p) && !castable_bestowed_from_graveyard(g, p, card) {
            let chars = g.option_characteristics(card, &opt);
            if !g.permitted_cards(p).contains(&card) || !g.permission_allows(p, card, &chars, false)
            {
                return vec![];
            }
        }
        opt.alt_cost = Some(super::modified_keyword_cost(
            g,
            p,
            KeywordKind::Bestow,
            &cost,
        ));
        vec![opt]
    }

    fn spell_text_change(
        &self,
        _g: &Game,
        _spell: ObjectId,
        _kw: &Keyword,
        paid: &[SmolStr],
        chars: &mut Characteristics,
    ) {
        if !paid.iter().any(|p| p == BESTOW) {
            return;
        }
        chars.card_types = [CardType::Enchantment].into_iter().collect();
        chars.subtypes = [SmolStr::new("Aura")].into_iter().collect();
        chars.all_creature_types = false;
        let kw = enchant_creature();
        chars.abilities.push(AbilityDef::new(
            AbilityKind::Keyword(kw.clone()),
            kw.kind.name(),
        ));
    }

    /// CR 702.103e: the spell ceases to be bestowed and resolves as a creature spell.
    fn unbestow(&self, g: &mut Game, spell: ObjectId) -> bool {
        let Some(si) = g.objects[spell.0 as usize].stack.as_deref_mut() else {
            return false;
        };
        if !si.cast.paid.iter().any(|p| p == BESTOW) {
            return false;
        }
        si.cast.paid.retain(|p| p != BESTOW);
        g.dirty = true;
        g.recompute();
        g.log(|g| format!("{} resolves as a creature spell", g.describe(spell)));
        true
    }

    /// The permanent a bestowed Aura spell becomes is a bestowed Aura.
    fn after_permanent_resolves(&self, g: &mut Game, spell: ObjectId, new: ObjectId, _kw: &Keyword) {
        let bestowed = g
            .obj(spell)
            .stack
            .as_deref()
            .is_some_and(|si| si.cast.paid.iter().any(|p| p == BESTOW));
        if !bestowed || g.obj(new).attached_to.is_none() || is_bestowed(g, new) {
            return;
        }
        let id = g.new_effect_id();
        let timestamp = g.obj(new).timestamp;
        let controller = g.obj(new).controller;
        let turn = g.turn.number;
        g.effects.push(ContinuousEffect {
            id,
            source: Some(new),
            controller,
            timestamp,
            duration: Duration::Permanent,
            affected: Affected::Objects(vec![new]),
            mods: bestowed_mods(),
            layer1: None,
            created_turn: turn,
        });
        g.dirty = true;
        g.recompute();
    }

    fn keeps_unattached_aura(&self, g: &Game, aura: ObjectId) -> bool {
        is_bestowed(g, aura)
    }

    /// CR 702.103f–g: a bestowed Aura that's unattached, or attached to an illegal object
    /// or player, becomes unattached and ceases to be bestowed.
    fn state_based_actions(&self, g: &mut Game) -> bool {
        let bestowed: Vec<ObjectId> = g
            .battlefield
            .iter()
            .copied()
            .filter(|id| !g.obj(*id).phased_out && is_bestowed(g, *id))
            .collect();
        let mut performed = false;
        for id in bestowed {
            let o = g.obj(id);
            let legal = match o.attached_to {
                None => false,
                Some(t) => {
                    !o.is_creature()
                        && !o.is(CardType::Battle)
                        && crate::attach::legal_attachment(g, id, t)
                }
            };
            if legal {
                continue;
            }
            if o.attached_to.is_some() {
                g.unattach(id);
            }
            cease_to_be_bestowed(g, id);
            performed = true;
        }
        if performed {
            g.recompute();
        }
        performed
    }
}

inventory::submit! { KeywordRegistration(&Bestow) }
