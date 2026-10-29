//! Choices made as an Equipment becomes attached (cards5-choices-players), beyond a
//! color (`choices::AS_ATTACHED_CHOOSE_COLOR`):
//!
//! - "As this Equipment becomes attached to a creature, choose a creature card name and a
//!   creature type." (Psychic Paper): the choices are made each time it becomes attached
//!   to a creature, as that happens (not by a triggered ability), and replace the ones
//!   made before ("the last chosen name and creature type", CR 607.2d);
//! - "its name and creature type are the last chosen name and creature type": the
//!   equipped creature's name becomes the chosen name (layer 3, with the other name and
//!   text changes; see the interpretation register, `docs/interpretations/`) and its
//!   creature types become the chosen one (layer 4, CR 613.1d, 205.1b). Each does
//!   nothing until a choice of its kind was made (CR 607.5a).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::{AbilityKind, ChoiceKind, StaticEffect};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::types::*;

/// `StaticEffect::Custom` name prefix: "As ~ becomes attached to a creature, choose ...";
/// the choice kinds follow as JSON.
pub const AS_ATTACHED_CHOOSE: &str = "attach5: as attached to a creature, choose:";
/// `Modification::Custom` (layer 3): its name is the last chosen card name.
pub const LAST_CHOSEN_NAME: &str = "attach5: name is the last chosen name";
/// `Modification::Custom` (layer 4): its creature type is the last chosen creature type.
pub const LAST_CHOSEN_CREATURE_TYPE: &str = "attach5: creature type is the last chosen type";

/// The static ability making these choices.
pub fn as_attached_choose(kinds: &[ChoiceKind]) -> Option<StaticEffect> {
    let json = serde_json::to_string(kinds).ok()?;
    Some(StaticEffect::Custom(format!("{AS_ATTACHED_CHOOSE}{json}").into()))
}

/// The choice kinds of an [`AS_ATTACHED_CHOOSE`] static ability.
pub fn attached_choice_kinds(name: &str) -> Option<Vec<ChoiceKind>> {
    serde_json::from_str(name.strip_prefix(AS_ATTACHED_CHOOSE)?).ok()
}

pub struct AttachChoices5;

impl KeywordRules for AttachChoices5 {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn on_event(&self, g: &mut Game, ev: &crate::events::Event) {
        let crate::events::Event::Attached {
            obj,
            to: Entity::Object(host),
        } = ev
        else {
            return;
        };
        let (obj, host) = (*obj, *host);
        if !g.is_live(obj) || !g.is_live(host) || !g.obj(host).chars.is_creature() {
            return;
        }
        let kinds: Vec<Vec<ChoiceKind>> = g
            .obj(obj)
            .chars
            .abilities
            .iter()
            .filter_map(|a| match &a.kind {
                AbilityKind::Static(s) => match &s.effect {
                    StaticEffect::Custom(n) => attached_choice_kinds(n),
                    _ => None,
                },
                _ => None,
            })
            .collect();
        let p = g.obj(obj).controller;
        for kind in kinds.iter().flatten() {
            let mut ctx = Ctx::new(Some(obj), p);
            crate::choices::make_choice(g, p, kind, &mut ctx);
        }
    }

    fn custom_modification(
        &self,
        g: &Game,
        name: &str,
        chars: &mut crate::object::Characteristics,
        ctx: &Ctx,
        _target: ObjectId,
    ) -> bool {
        match name {
            LAST_CHOSEN_NAME => {
                if let Some(n) = g
                    .source_choices(ctx)
                    .and_then(|c| c.card_name.clone())
                    .filter(|n| !n.is_empty())
                {
                    // As a name-setting effect does (CR 612.8): no other name.
                    chars.name = n;
                    chars.all_creature_names = false;
                    chars.interchangeable_names.clear();
                }
                true
            }
            LAST_CHOSEN_CREATURE_TYPE => {
                if let Some(t) = g.source_choices(ctx).and_then(|c| c.creature_type.clone()) {
                    chars.subtypes.retain(|s| !is_creature_type(s));
                    chars.subtypes.push(t);
                }
                true
            }
            _ => false,
        }
    }
}

inventory::submit! { KeywordRegistration(&AttachChoices5) }
