//! CR 702.99 Cipher, on instants and sorceries. It represents a spell ability, "If this
//! spell is represented by a card, you may exile this card encoded on a creature you
//! control," and a static ability of the card in exile, "For as long as this card is
//! encoded on that creature, that creature has 'Whenever this creature deals combat damage
//! to a player, you may copy the encoded card and you may cast the copy without paying its
//! mana cost.'" (CR 702.99a).
//!
//! The spell ability is the last instruction of the spell: the card goes directly from
//! the stack to exile, encoded on the creature chosen as it resolves (a creature isn't
//! targeted). The relation (CR 702.99b) is kept on the card in exile, linked under
//! [`CIPHER_LINK`] to the creature, and lasts as long as the card stays exiled and the
//! creature stays on the battlefield, even if it changes controller or stops being a
//! creature (CR 702.99c). Meanwhile the creature has the triggered ability, granted by an
//! ability-adding effect of the card: a creature that loses it doesn't trigger, but the
//! card stays encoded. Whoever controls the creature controls the ability, and casts the
//! copy from exile during its resolution. The effect is created as the card is encoded,
//! with a timestamp from when it's exiled (as the card's static ability would have,
//! CR 613.7a, 613.7d), rather than being a static ability the keyword stands for, because
//! the granted ability refers to the one card that granted it.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::events::MoveCause;
use crate::game::{Affected, ContinuousEffect, Game};
use crate::keywords::{Keyword, KeywordKind};
use crate::object::*;
use crate::replacement::{EtbInfo, MoveEv};
use crate::types::*;
use smol_str::SmolStr;

/// The link (in `GameObject::linked`) of an exiled card with cipher to the creature it's
/// encoded on.
pub const CIPHER_LINK: u16 = 0x7ffb;
/// `Effect::Custom`: "you may exile this card encoded on a creature you control".
const ENCODE: &str = "cipher:you may exile this card encoded on a creature you control";
/// `Effect::Custom` prefix, followed by the encoded card's id: "you may copy the encoded
/// card and you may cast the copy without paying its mana cost".
const CAST_COPY: &str = "cipher:copy and cast #";
/// `Condition::Custom`: the source of the context (an exiled card) is still encoded on a
/// creature.
const STILL_ENCODED: &str = "cipher:this card is encoded on a creature";

pub struct Cipher;

/// The permanent the exiled card `card` is encoded on, if it's still encoded
/// (CR 702.99c).
pub fn encoded_on(g: &Game, card: ObjectId) -> Option<ObjectId> {
    if !g.is_live(card) || g.obj(card).zone != Zone::Exile {
        return None;
    }
    g.obj(card)
        .linked
        .get(&CIPHER_LINK)?
        .iter()
        .copied()
        .find(|c| g.is_live(*c) && g.obj(*c).zone == Zone::Battlefield)
}

/// The cards encoded on `creature`.
pub fn encoded_cards(g: &Game, creature: ObjectId) -> Vec<ObjectId> {
    g.exile
        .iter()
        .copied()
        .filter(|c| encoded_on(g, *c) == Some(creature))
        .collect()
}

impl KeywordRules for Cipher {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Cipher]
    }

    fn derived(&self, _kw: &Keyword) -> Option<Vec<Ability>> {
        // Spell abilities follow the spell's other instructions (see `Game::spell_body`).
        Some(vec![AbilityDef::new(
            AbilityKind::Spell(SpellAbility {
                body: Body::effect(Effect::Custom(ENCODE.into())),
            }),
            KeywordKind::Cipher.name(),
        )])
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name == ENCODE {
            encode(g, ctx);
            return true;
        }
        let Some(id) = name.strip_prefix(CAST_COPY) else {
            return false;
        };
        if let Ok(n) = id.parse::<u32>() {
            cast_copy(g, ctx, ObjectId(n));
        }
        true
    }

    fn custom_condition(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<bool> {
        (name == STILL_ENCODED).then(|| ctx.source.is_some_and(|c| encoded_on(g, c).is_some()))
    }
}

/// The spell ability: the spell (`ctx.source`), if it's a card, may be exiled encoded on
/// a creature its controller controls.
fn encode(g: &mut Game, ctx: &mut Ctx) {
    let Some(spell) = ctx.source else {
        return;
    };
    let o = g.obj(spell);
    if o.kind != ObjKind::Card || o.zone != Zone::Stack || !g.is_live(spell) {
        return;
    }
    let p = ctx.controller;
    let cands: Vec<ObjectId> = g
        .battlefield
        .iter()
        .copied()
        .filter(|c| {
            let o = g.obj(*c);
            o.controller == p && o.chars.is(CardType::Creature) && !o.phased_out
        })
        .collect();
    let Some(creature) = g
        .ask_objects(
            p,
            Some(spell),
            "Exile this card encoded on a creature you control? (cipher)",
            cands,
            0,
            1,
        )
        .first()
        .copied()
    else {
        return;
    };
    let Some(card) = g.move_object_ev(MoveEv {
        obj: spell,
        to: Zone::Exile,
        pos: LibraryPosition::Top,
        cause: MoveCause::Resolve,
        by: Some(p),
        etb: EtbInfo::default(),
        source: None,
    }) else {
        return;
    };
    if g.obj(card).zone != Zone::Exile {
        return;
    }
    g.objects[card.0 as usize]
        .linked
        .insert(CIPHER_LINK, vec![creature]);
    // "For as long as this card is encoded on that creature, that creature has ...".
    let trigger = TriggeredAbility::new(
        TriggerCond::DealsDamage {
            source: Filter::Source,
            to: DamageRecipient::Player(PlayerRel::Any),
            combat_only: true,
        },
        Body::effect(Effect::Custom(SmolStr::new(format!("{CAST_COPY}{}", card.0)))),
    );
    let id = g.new_effect_id();
    let timestamp = g.new_timestamp();
    let turn = g.turn.number;
    let owner = g.obj(card).owner;
    g.effects.push(ContinuousEffect {
        id,
        source: Some(card),
        controller: owner,
        timestamp,
        duration: Duration::WhileCondition(Condition::Custom(STILL_ENCODED.into())),
        affected: Affected::Objects(vec![creature]),
        mods: vec![Modification::AddAbility(AbilityDef::new(
            AbilityKind::Triggered(trigger),
            KeywordKind::Cipher.name(),
        ))],
        layer1: None,
        created_turn: turn,
    });
    g.dirty = true;
    g.log(|g| format!("{} is encoded on {}", g.describe(card), g.describe(creature)));
}

/// The granted triggered ability: its controller may copy the encoded card (in exile) and
/// may cast the copy without paying its mana cost, during its resolution.
fn cast_copy(g: &mut Game, ctx: &mut Ctx, card: ObjectId) {
    if !g.is_live(card) || g.obj(card).zone != Zone::Exile {
        return;
    }
    const CARD: Var = vars::USER + 71;
    ctx.set_var(CARD, vec![Entity::Object(card)]);
    g.exec(
        &Effect::Seq(vec![
            Effect::CopyCard {
                what: Sel::Var(CARD),
                named: None,
            },
            Effect::CastCard {
                who: PlayerRef::You,
                what: Sel::Var(vars::CREATED),
                free: true,
                optional: true,
            },
        ]),
        ctx,
    );
}

inventory::submit! { KeywordRegistration(&Cipher) }
