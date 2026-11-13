//! CR 702.35 Madness: "If a player would discard this card, that player discards it, but
//! exiles it instead of putting it into their graveyard" (a static ability that functions
//! while the card is in a hand) and "When this card is exiled this way, its owner may cast
//! it by paying [cost] rather than paying its mana cost. If that player doesn't, they put
//! this card into their graveyard." (CR 702.35a). Casting it follows the rules for
//! alternative costs (CR 702.35b) and ignores timing restrictions based on its card type.
//! If it isn't cast and goes to the graveyard, effects referencing the discarded card can
//! find it there (CR 702.35c, 400.7k).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::casting::CastOption;
use crate::eval::Ctx;
use crate::events::{Event, MoveCause};
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::object::*;
use crate::replacement::{EtbInfo, MoveEv};
use crate::types::*;

/// The custom effect of the madness triggered ability (followed by `:` and the key of the
/// madness ability's cost, see [`cost_key`]).
pub const CAST_MADNESS: &str = "madness_cast";
/// The custom instruction a madness replacement effect performs right after the card is
/// discarded into exile, and the custom trigger of its linked triggered ability ("When
/// this card is exiled this way"), each followed by `:` and the key of its cost.
pub const EXILED_THIS_WAY: &str = "madness_exiled";
/// The name recorded in `CastInfo::paid` when a spell is cast for its madness cost.
pub const MADNESS: &str = "madness";
/// The link under which the exiled card records which madness ability exiled it (the
/// key of its cost, in `Choices::text`): its own two abilities are linked (CR 607.2),
/// and of several madness abilities only the one whose replacement exiled the card
/// triggers ("exiled this way"; Falkenrath Gorger's ruling: "you'll choose which madness
/// ability exiles it"). A reserved link, below the links of copied abilities (`0x4000`
/// on, `layers::copied_link`) like the other reserved ones.
const EXILED_BY: u16 = 0x3fb3;

pub struct Madness;

/// The key of a madness ability: its cost itself, written out (`null`: "The madness cost
/// is equal to its mana cost"), so that its triggered ability casts the card for that
/// ability's own cost even if the card has lost the ability since (Falkenrath Gorger
/// leaving: "the madness ability will still let you cast that Vampire card for the
/// appropriate cost even though it no longer has madness").
fn cost_key(kw: &Keyword) -> String {
    serde_json::to_string(&kw.cost).unwrap_or_default()
}

/// The madness cost the ability with cost key `key` casts `card` for. A madness ability
/// granted with "The madness cost is equal to its mana cost" has no cost of its own.
fn madness_cost(g: &Game, card: ObjectId, key: &str) -> Option<Cost> {
    let cost: Option<Cost> = serde_json::from_str(key).ok()?;
    Some(match cost {
        Some(c) => c,
        // A card with no mana cost has an unpayable madness cost (CR 118.6).
        None => Cost::mana(
            g.obj(card)
                .chars
                .mana_cost
                .clone()
                .unwrap_or_else(crate::cost_rules::unpayable),
        ),
    })
}

impl KeywordRules for Madness {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Madness]
    }

    fn derived(&self, kw: &Keyword) -> Option<Vec<Ability>> {
        let text = KeywordKind::Madness.name();
        let key = cost_key(kw);
        // The discard into exile, after which the card notes that this ability exiled it.
        // When a card has several madness abilities, its owner chooses which one to apply
        // (CR 616.1; Falkenrath Gorger's ruling): that one exiles it and is noted; the
        // others, applied after it, change nothing.
        let mut exile = StaticAbility::new(StaticEffect::Replacement(ReplacementDef {
            event: ReplacementEvent::Discard(PlayerFilter::Any, Filter::Source),
            action: ReplacementAction::Then(
                Box::new(ReplacementAction::MoveInstead(Destination::zone(
                    ZoneKind::Exile,
                ))),
                Box::new(Effect::Custom(format!("{EXILED_THIS_WAY}:{key}").into())),
            ),
            self_replacement: false,
            optional: false,
        }));
        exile.zone = FunctionZone::Hand;
        // "When this card is exiled this way": the discard of this card into exile by the
        // replacement effect above (which applies to a discard of this card even along
        // with other effects that exile it, as with Rest in Peace).
        let mut cast = TriggeredAbility::new(
            TriggerCond::Custom(format!("{EXILED_THIS_WAY}:{key}").into()),
            Body::effect(Effect::Custom(format!("{CAST_MADNESS}:{key}").into())),
        );
        cast.zone = FunctionZone::Exile;
        // The replacement effect names its cost, so that a player choosing which of two
        // madness abilities exiles the card can tell them apart.
        let exile_text = match &kw.cost {
            Some(c) => format!("{text} {}", crate::resolve::describe_cost(c)),
            None => format!("{text} (cost equal to its mana cost)"),
        };
        Some(vec![
            AbilityDef::new(AbilityKind::Static(exile), exile_text),
            AbilityDef::new(AbilityKind::Triggered(cast), text),
        ])
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if let Some(key) = name.strip_prefix(CAST_MADNESS).and_then(|k| k.strip_prefix(':')) {
            cast_madness(g, ctx, key);
            return true;
        }
        if let Some(key) = name
            .strip_prefix(EXILED_THIS_WAY)
            .and_then(|k| k.strip_prefix(':'))
        {
            // The card discarded into exile notes the madness ability that exiled it,
            // unless another one already did.
            let it = ctx.vars.get(&vars::IT).and_then(|v| v.first().copied());
            if let Some(Entity::Object(card)) = it {
                let card = g.current(card);
                if g.is_live(card) && g.obj(card).zone == Zone::Exile {
                    let c = g.objects[card.0 as usize]
                        .linked_choices
                        .entry(EXILED_BY)
                        .or_default();
                    if c.text.is_none() {
                        c.text = Some(key.into());
                    }
                }
            }
            return true;
        }
        false
    }

    fn custom_trigger(
        &self,
        g: &Game,
        name: &str,
        src: ObjectId,
        _ctl: PlayerId,
        ev: &Event,
    ) -> Option<Vec<EventInfo>> {
        let key = name
            .strip_prefix(EXILED_THIS_WAY)
            .and_then(|k| k.strip_prefix(':'))?;
        let Event::Discarded {
            player, card, by, ..
        } = ev
        else {
            return Some(vec![]);
        };
        let exiled_this_way = *card == src
            && g.try_obj(src).is_some_and(|o| {
                o.zone == Zone::Exile
                    && o.linked_choices
                        .get(&EXILED_BY)
                        .and_then(|c| c.text.as_deref())
                        == Some(key)
            });
        Some(if exiled_this_way {
            vec![EventInfo {
                player: Some(*player),
                object: Some(*card),
                caused_by: *by,
                ..Default::default()
            }]
        } else {
            vec![]
        })
    }
}

/// The madness triggered ability resolving: the card's owner may cast it from exile by
/// paying its madness cost rather than its mana cost; if they don't, they put it into
/// their graveyard (CR 702.35a).
pub fn cast_madness(g: &mut Game, ctx: &mut Ctx, key: &str) {
    let Some(card) = ctx.source else {
        return;
    };
    // The ability can't find the card if it has left exile (CR 400.7).
    if !g.is_live(card) || g.obj(card).zone != Zone::Exile {
        return;
    }
    let owner = g.obj(card).owner;
    if let Some(cost) = madness_cost(g, card, key) {
        let name = g.obj(card).chars.name.clone();
        if g.ask_yes_no(owner, Some(card), &format!("Cast {name} for its madness cost?"), true) {
            let mut opt = CastOption::normal(FaceState::Front);
            opt.method = CastMethod::Keyword(KeywordKind::Madness);
            opt.alt_cost = Some(cost);
            // Timing permissions based on card type don't apply (CR 702.35 rulings).
            opt.any_time = true;
            opt.tag = Some(MADNESS);
            if g.cast_with_option(owner, card, opt).is_ok() {
                return;
            }
        }
    }
    if g.is_live(card) && g.obj(card).zone == Zone::Exile {
        g.move_object_ev(MoveEv {
            obj: card,
            to: Zone::Graveyard(owner),
            pos: LibraryPosition::Top,
            cause: MoveCause::Effect,
            by: Some(owner),
            etb: EtbInfo::default(),
            source: Some(card),
        });
    }
}

/// CR 702.35c, 400.7k: after a madness triggered ability resolved, if the exiled card
/// wasn't cast and was moved to a public zone, effects referencing the discarded card
/// (the object it was in exile) can find the object it became.
pub fn found_after_madness(g: &Game, discarded: ObjectId) -> Option<ObjectId> {
    let o = g.try_obj(discarded)?;
    // A madness ability exiled it: the card noted which one (it may have lost the ability
    // since, as when Falkenrath Gorger leaves the battlefield).
    if o.zone != Zone::Exile || !o.linked_choices.contains_key(&EXILED_BY) {
        return None;
    }
    // It was discarded into exile (not exiled from a hand some other way).
    let prev = g.try_obj(o.prev?)?;
    let discarded_this_way = g.turn_events.iter().any(
        |e| matches!(e, crate::events::Event::Discarded { card, .. } if *card == discarded),
    );
    if !matches!(prev.zone, Zone::Hand(_)) || !discarded_this_way {
        return None;
    }
    let next = o.next?;
    let public = !matches!(
        g.obj(next).zone,
        Zone::Hand(_) | Zone::Library(_) | Zone::Outside(_) | Zone::Nowhere | Zone::Stack
    );
    (public && g.is_live(next)).then_some(next)
}

inventory::submit! { KeywordRegistration(&Madness) }
