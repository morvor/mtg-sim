//! "That many" after a card action performed by each of several players ("Each player
//! discards all the cards in their hand, then draws that many cards"): each player's
//! later instruction uses the number of cards *that player* acted on. The grammar
//! (`oracle/patterns/hand_graveyard_grammar.rs`) records the number per iterated player
//! with [`RECORD_THAT_MANY`], and reads it back with the value [`THAT_MANY`].

use super::{KeywordRegistration, KeywordRules};
use crate::ability::{vars, Var};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;

/// The number of cards the most recent card action affected (any player).
pub const THAT_MANY_VAR: Var = vars::USER + 6102;
/// Per-player numbers: `THAT_MANY_BY_PLAYER + player index`.
const THAT_MANY_BY_PLAYER: Var = vars::USER + 6120;

/// `Effect::Custom`: copies [`THAT_MANY_VAR`] into the iterated player's slot (when the
/// action is performed by each of several players in turn).
pub const RECORD_THAT_MANY: &str = "record that many for the iterated player";
/// `Value::Custom`: the iterated player's number if they performed the action, otherwise
/// [`THAT_MANY_VAR`].
pub const THAT_MANY: &str = "that many";

/// `Filter::Custom` name prefix: a card that was put into its graveyard this turn
/// ("target creature card in a graveyard that was put there this turn"), followed by
/// the zone it came from ("battlefield", "library") or nothing for anywhere.
pub const PUT_THERE_THIS_TURN: &str = "put into its graveyard this turn from:";

/// `Modification::Custom` name prefix (layer 6): "has all activated abilities of all
/// [kind] cards in [all graveyards | your graveyard]" — followed by "all:" or "your:" and
/// the card type or subtype word.
pub const ACTIVATED_ABILITIES_OF_GRAVEYARD: &str = "has all activated abilities of cards in graveyards:";

/// `Filter::Custom`: the bottom card of its graveyard (the one put there earliest, CR
/// 404.2).
pub const BOTTOM_OF_GRAVEYARD: &str = "the bottom card of its graveyard";

/// `Filter::Custom`: the source itself, or the new object it became with its latest zone
/// change ("exile ~ from your graveyard" after it died, CR 400.7).
pub const SOURCE_OR_NEXT: &str = "the source or the object it became";

/// `Filter::Custom`: the top card of its graveyard (the one put there latest, CR 404.2).
pub const TOP_OF_GRAVEYARD: &str = "the top card of its graveyard";

pub struct HandGraveyardActions;

impl KeywordRules for HandGraveyardActions {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_effect(&self, _g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != RECORD_THAT_MANY {
            return false;
        }
        if let Some(p) = ctx.iter_player {
            let n = ctx.nums.get(&THAT_MANY_VAR).copied().unwrap_or(0);
            ctx.nums.insert(THAT_MANY_BY_PLAYER + p.0 as Var, n);
        }
        true
    }

    fn custom_filter(&self, g: &Game, name: &str, id: crate::types::ObjectId, ctx: &Ctx) -> Option<bool> {
        if name == SOURCE_OR_NEXT {
            return Some(ctx.source.is_some_and(|s| s == id || g.obj(s).next == Some(id)));
        }
        if name == TOP_OF_GRAVEYARD {
            let o = g.obj(id);
            let crate::object::Zone::Graveyard(p) = o.zone else {
                return Some(false);
            };
            return Some(g.player(p).graveyard.last() == Some(&id));
        }
        if name == BOTTOM_OF_GRAVEYARD {
            let o = g.obj(id);
            let crate::object::Zone::Graveyard(p) = o.zone else {
                return Some(false);
            };
            return Some(g.player(p).graveyard.first() == Some(&id));
        }
        let from = name.strip_prefix(PUT_THERE_THIS_TURN)?;
        let o = g.obj(id);
        if !matches!(o.zone, crate::object::Zone::Graveyard(_)) || o.entered_turn != g.turn.number {
            return Some(false);
        }
        if from.is_empty() {
            return Some(true);
        }
        // The zone it was in before (its previous incarnation, CR 400.7).
        let prev_zone = o.prev.map(|p| g.obj(p).zone);
        Some(match (from, prev_zone) {
            ("battlefield", Some(crate::object::Zone::Battlefield)) => true,
            ("library", Some(crate::object::Zone::Library(_))) => true,
            _ => false,
        })
    }

    fn custom_modification(
        &self,
        g: &Game,
        name: &str,
        chars: &mut crate::object::Characteristics,
        ctx: &Ctx,
        _target: crate::types::ObjectId,
    ) -> bool {
        let Some(r) = name.strip_prefix(ACTIVATED_ABILITIES_OF_GRAVEYARD) else {
            return false;
        };
        let Some((scope, kind)) = r.split_once(':') else {
            return true;
        };
        let card_type = crate::types::CardType::from_word(kind);
        let players: Vec<crate::types::PlayerId> = match scope {
            "your" => vec![ctx.controller],
            _ => g.players_in_game(),
        };
        let mut gained = Vec::new();
        for p in players {
            for c in &g.player(p).graveyard {
                let o = g.obj(*c);
                if !o.is_card() {
                    continue;
                }
                let ok = match card_type {
                    Some(t) => o.chars.card_types.contains(t),
                    None => o.chars.has_subtype(kind),
                };
                if !ok {
                    continue;
                }
                for a in &o.chars.abilities {
                    match &a.kind {
                        crate::ability::AbilityKind::Activated(_) => gained.push(a.clone()),
                        // A keyword that is an activated ability (outlast, reconfigure,
                        // ...): its activated ability (Necrotic Ooze rulings).
                        crate::ability::AbilityKind::Keyword(k) => gained.extend(
                            crate::keyword_impls::derived_abilities(k)
                                .into_iter()
                                .filter(|d| {
                                    matches!(d.kind, crate::ability::AbilityKind::Activated(_))
                                }),
                        ),
                        _ => {}
                    }
                }
            }
        }
        chars.abilities.extend(gained);
        true
    }

    fn custom_value(&self, _g: &Game, name: &str, ctx: &Ctx) -> Option<i64> {
        if name != THAT_MANY {
            return None;
        }
        let per = ctx
            .iter_player
            .and_then(|p| ctx.nums.get(&(THAT_MANY_BY_PLAYER + p.0 as Var)).copied());
        Some(per.unwrap_or_else(|| ctx.nums.get(&THAT_MANY_VAR).copied().unwrap_or(0)))
    }
}

inventory::submit! { KeywordRegistration(&HandGraveyardActions) }
