//! Prevention grammar IV (cards3-costs-mana-replacements).
//!
//! - "that player may pay any amount of mana. ~ deals 2 damage to that player. Prevent X
//!   of that damage, where X is the amount of mana that player paid this way." (Errant
//!   Minion, Power Leak): the player pays (`kw/spell_cost_grammar.rs`), then a prevention
//!   shield for X of the damage the source deals to them is set up before the damage is
//!   dealt (CR 615.7), so X of it is prevented.
//! - "Prevent all damage that would be dealt to a creature by another creature if they
//!   share a color." (Well-Laid Plans): a prevention effect on damage between two
//!   creatures with a color in common (`kw/damage_shares_a_color.rs`).

use super::{FollowupPattern, StaticPattern};
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;
use crate::oracle::CompileContext;

/// "Prevent X of that damage, where X is the amount of mana that player paid this way."
/// after "[that player] may pay any amount of mana. ~ deals N damage to that player." (the
/// payment's X is the amount paid, `kw/spell_cost_grammar.rs`).
fn prevent_x_of_that_damage(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    if end(l) != "prevent x of that damage, where x is the amount of mana that player paid this way"
    {
        return false;
    }
    let Effect::DealDamage {
        source: Sel::This,
        to: Sel::Players(to),
        ..
    } = &*prev
    else {
        return false;
    };
    // The sentence before is that player's payment, and the damage is to them.
    let raw = crate::oracle::raw_text().to_lowercase();
    if !raw.contains(", that player may pay any amount of mana. ~ deals ")
        && !raw.contains(", that player may pay any amount of mana. this aura deals ")
    {
        return false;
    }
    if !matches!(to, PlayerRef::ActivePlayer | PlayerRef::TriggerPlayer) {
        return false;
    }
    let shield = Effect::AddReplacement {
        def: ReplacementDef {
            event: ReplacementEvent::Damage {
                source: Filter::Source,
                to_players: Some(PlayerFilter::Ref(Box::new(to.clone()))),
                to_objects: None,
                combat_only: false,
            },
            action: ReplacementAction::PreventAmount(Value::X),
            self_replacement: false,
            optional: false,
        },
        // Only "that damage": the one damage event that follows.
        duration: Duration::EndOfTurn,
        uses: Some(1),
    };
    let damage = prev.clone();
    *prev = Effect::Seq(vec![shield, damage]);
    true
}

inventory::submit! { FollowupPattern { name: "prevention 4: prevent X of that damage (the mana paid)", priority: 100, apply: prevent_x_of_that_damage } }

/// "prevent all damage that would be dealt to a creature by another creature if they share
/// a color".
fn prevent_if_they_share_a_color(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    if end(l)
        != "prevent all damage that would be dealt to a creature by another creature if they share a color"
    {
        return None;
    }
    let def = ReplacementDef {
        event: ReplacementEvent::Where {
            event: Box::new(ReplacementEvent::Damage {
                source: Filter::creature(),
                to_players: None,
                to_objects: Some(Filter::creature()),
                combat_only: false,
            }),
            cond: Condition::Custom(
                crate::kw::damage_shares_a_color::SOURCE_AND_RECIPIENT_SHARE_A_COLOR.into(),
            ),
        },
        action: ReplacementAction::Prevent,
        self_replacement: false,
        optional: false,
    };
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Replacement(def))),
        text,
    )])
}

inventory::submit! { StaticPattern { name: "prevention 4: prevent damage between creatures that share a color", priority: 50, parse: prevent_if_they_share_a_color } }
