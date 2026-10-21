//! Winter's Chill: "Choose X target attacking creatures. For each of those creatures, its
//! controller may pay {1} or {2}. If that player doesn't, destroy that creature at end of
//! combat. If that player pays only {1}, prevent all combat damage that would be dealt to
//! and dealt by that creature this combat."
//!
//! Three outcomes per creature, chosen as the spell resolves (rulings): pay {2} (nothing
//! happens), pay only {1} (its combat damage is prevented this combat), or pay nothing (it's
//! destroyed at end of combat). The controller is offered {2} first, then {1}.

use super::{map_effect, parse, ManualAbility};
use crate::ability::*;
use crate::mana::ManaCost;

const TEXT: &str = "Choose X target attacking creatures. For each of those creatures, its controller may pay {1} or {2}. If that player doesn't, destroy that creature at end of combat. If that player pays only {1}, prevent all combat damage that would be dealt to and dealt by that creature this combat.";
const IT: Var = vars::USER + 31100;

fn pay(n: &str) -> Cost {
    Cost {
        mana: ManaCost::parse(n),
        parts: vec![],
    }
}

fn prevent(event: ReplacementEvent) -> Effect {
    Effect::AddReplacement {
        def: ReplacementDef {
            event,
            action: ReplacementAction::Prevent,
            self_replacement: false,
            optional: false,
        },
        duration: Duration::EndOfCombat,
        uses: None,
    }
}

fn per_creature() -> Effect {
    let it = || Sel::Var(IT);
    let controller = || PlayerRef::ControllerOf(Box::new(it()));
    let damage_prevented = Effect::Seq(vec![
        prevent(ReplacementEvent::Damage {
            source: Filter::Any,
            to_players: None,
            to_objects: Some(Filter::In(Box::new(it()))),
            combat_only: true,
        }),
        prevent(ReplacementEvent::Damage {
            source: Filter::In(Box::new(it())),
            to_players: Some(PlayerFilter::Any),
            to_objects: Some(Filter::Any),
            combat_only: true,
        }),
    ]);
    let destroyed = Effect::DelayedTrigger {
        trigger: TriggerCond::BeginningOf {
            step: TriggerStep::EndOfCombat,
            whose: PlayerRel::Any,
        },
        body: Box::new(Body::effect(Effect::Destroy {
            what: it(),
            no_regen: false,
        })),
        once: true,
    };
    Effect::PayOptional {
        who: controller(),
        cost: pay("{2}"),
        then: Box::new(Effect::Noop),
        otherwise: Box::new(Effect::PayOptional {
            who: controller(),
            cost: pay("{1}"),
            then: Box::new(damage_prevented),
            otherwise: Box::new(destroyed),
        }),
    }
}

inventory::submit! { ManualAbility {
    card: "Winter's Chill",
    face: 0,
    text: TEXT,
    build: |ctx| {
        // The targets: X attacking creatures.
        parse(ctx, "Tap X target attacking creatures.")
            .iter()
            .map(|a| {
                map_effect(a, TEXT, |_| Effect::ForEach {
                    sel: Sel::Target(0),
                    var: IT,
                    effect: Box::new(per_creature()),
                })
            })
            .collect()
    },
    reason: "a choice of three payments per creature with different outcomes: only this card",
} }
