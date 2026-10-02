//! CR 702.192 Paradigm: "If this is the first time a spell you control with this spell's
//! name has resolved this game, at the beginning of each of your precombat main phases for
//! the rest of the game, create a copy of this object in exile. You may cast the copy
//! without paying its mana cost" and "Exile this spell." (CR 702.192a, 707.10).
//!
//! * Both are spell abilities performed as the spell resolves, after its other text: the
//!   spell is put into exile rather than its owner's graveyard
//!   ([`KeywordRules::resolved_destination`]). A countered spell doesn't resolve.
//! * The names of the spells each player controlled that have resolved are recorded as
//!   they resolve (`KeywordState::resolved_spell_names`), whether or not they have
//!   paradigm, copies included.
//! * The delayed triggered ability copies the spell as it last existed on the stack
//!   (the card leaving exile later doesn't matter); a copy that isn't cast ceases to exist
//!   (CR 707.10a).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::events::Event;
use crate::game::{DelayedTrigger, Game};
use crate::keywords::{Keyword, KeywordKind};
use crate::object::*;
use crate::types::*;
use smol_str::SmolStr;

/// `Effect::Custom`: the delayed triggered ability's effect.
const PARADIGM_COPY: &str = "paradigm:create a copy in exile and cast it";
/// The variable of the delayed triggered ability's context holding the resolved spell.
const PARADIGM_SPELL: Var = vars::USER + 192;

/// Whether a spell `p` controlled named `name` has resolved this game.
pub fn resolved_before(g: &Game, p: PlayerId, name: &str) -> bool {
    g.kw_state
        .resolved_spell_names
        .get(&p)
        .is_some_and(|s| s.contains(name))
}

pub struct Paradigm;

impl KeywordRules for Paradigm {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Paradigm]
    }

    /// "Exile this spell."
    fn resolved_destination(
        &self,
        _g: &Game,
        _spell: ObjectId,
        _kw: &Keyword,
    ) -> Option<(Zone, LibraryPosition)> {
        Some((Zone::Exile, LibraryPosition::Top))
    }

    /// The spell's own instruction, not a replacement effect.
    fn resolved_destination_replaces(&self) -> bool {
        false
    }

    /// "If this is the first time a spell you control with this spell's name has resolved
    /// this game, at the beginning of each of your precombat main phases for the rest of
    /// the game, create a copy of this object in exile."
    fn after_spell_resolved(&self, g: &mut Game, spell: ObjectId, _kw: &Keyword, _new: ObjectId) {
        let o = g.obj(spell);
        let (p, name) = (o.controller, o.chars.name.clone());
        if name.is_empty() || resolved_before(g, p, &name) {
            return;
        }
        g.kw_state
            .resolved_spell_names
            .entry(p)
            .or_default()
            .insert(name);
        let mut dctx = Ctx::new(Some(spell), p);
        dctx.set_var(PARADIGM_SPELL, vec![Entity::Object(spell)]);
        let id = g.new_effect_id();
        g.delayed_triggers.push(DelayedTrigger {
            id,
            source: Some(spell),
            controller: p,
            trigger: TriggerCond::BeginningOf {
                step: TriggerStep::PrecombatMain,
                whose: PlayerRel::You,
            },
            body: Body::effect(Effect::Custom(SmolStr::new(PARADIGM_COPY))),
            once: false,
            ctx: dctx,
            created_turn: g.turn.number,
            created_step: Some(g.turn.step),
            for_rest_of_game: true,
            performer: None,
        });
        g.log(|g| format!("{p}'s paradigm: {}", g.describe(spell)));
    }

    fn on_event(&self, g: &mut Game, ev: &Event) {
        if let Event::SpellResolved { spell } = ev {
            let o = g.obj(*spell);
            let (p, name) = (o.controller, o.chars.name.clone());
            if !name.is_empty() {
                g.kw_state
                    .resolved_spell_names
                    .entry(p)
                    .or_default()
                    .insert(name);
            }
        }
    }

    /// "Create a copy of this object in exile. You may cast the copy without paying its
    /// mana cost."
    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != PARADIGM_COPY {
            return false;
        }
        let Some(spell) = ctx.var_objects(PARADIGM_SPELL).first().copied() else {
            return true;
        };
        let o = g.obj(spell).clone();
        let Some(copy) = crate::copy_rules::new_card_copy(
            g,
            o.card.clone(),
            o.copiable.clone(),
            ctx.controller,
            Zone::Exile,
        ) else {
            return true;
        };
        g.recompute();
        ctx.set_var(vars::CREATED, vec![Entity::Object(copy)]);
        g.exec(
            &Effect::CastCard {
                who: PlayerRef::You,
                what: Sel::Var(vars::CREATED),
                free: true,
                optional: true,
            },
            ctx,
        );
        true
    }
}

inventory::submit! { KeywordRegistration(&Paradigm) }
