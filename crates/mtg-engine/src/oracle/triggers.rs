//! Parsing triggered abilities: "When/Whenever/At [trigger], [if condition,] [effect]."

use super::effects::parse_trigger_body;
use super::phrases::*;
use super::CompileContext;
use crate::ability::*;

pub fn parse_triggered(text: &str, ctx: &CompileContext) -> Option<Ability> {
    let t = text.trim();
    // Split trigger condition from effect at the first comma outside quotes that
    // follows the trigger phrase.
    let (cond_s, eff_s) = split_trigger(t)?;
    let lower = cond_s.to_lowercase();
    let (trigger, it, it_player) = parse_trigger_condition(&lower)?;
    let mut eff = eff_s.trim();
    // Intervening "if" clause (CR 603.4).
    let mut intervening = None;
    let el = eff.to_lowercase();
    if let Some(r) = el.strip_prefix("if ") {
        if let Some((c, _)) = r.split_once(", ") {
            if let Some(cond) = super::statics::parse_condition(c, ctx) {
                intervening = Some(cond);
                eff = &eff[3 + c.len() + 2..];
            }
        }
    }
    let body = parse_trigger_body(eff, ctx, it, it_player)?;
    let mut tr = TriggeredAbility::new(trigger, body);
    tr.intervening_if = intervening;
    let lower_full = t.to_lowercase();
    if lower_full.ends_with("this ability triggers only once each turn.") {
        tr.once_per_turn = true;
    }
    if let TriggerCond::CastSpell {
        filter: Filter::Source,
        ..
    } = &tr.trigger
    {
        tr.zone = FunctionZone::Stack;
    }
    Some(AbilityDef::new(AbilityKind::Triggered(tr), text))
}

fn split_trigger(t: &str) -> Option<(&str, &str)> {
    let mut in_quote = false;
    for (i, ch) in t.char_indices() {
        match ch {
            '"' => in_quote = !in_quote,
            ',' if !in_quote => return Some((&t[..i], &t[i + 1..])),
            _ => {}
        }
    }
    None
}

/// Returns (trigger, what "it" refers to, what "that player" refers to).
pub fn parse_trigger_condition(l: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    let l = l.trim();
    let obj = || Sel::TriggerObject;
    // "At the beginning of ..."
    if let Some(r) = l.strip_prefix("at the beginning of ") {
        let (step, whose) = match r {
            "your upkeep" => (TriggerStep::Upkeep, PlayerRel::You),
            "each upkeep" | "each player's upkeep" => (TriggerStep::Upkeep, PlayerRel::Any),
            "each opponent's upkeep" => (TriggerStep::Upkeep, PlayerRel::Opponent),
            "your end step" => (TriggerStep::End, PlayerRel::You),
            "each end step" => (TriggerStep::End, PlayerRel::Any),
            "each opponent's end step" => (TriggerStep::End, PlayerRel::Opponent),
            "the next end step" | "the beginning of the next end step" => {
                (TriggerStep::End, PlayerRel::Any)
            }
            "your draw step" => (TriggerStep::Draw, PlayerRel::You),
            "combat on your turn" => (TriggerStep::BeginningOfCombat, PlayerRel::You),
            "each combat" => (TriggerStep::BeginningOfCombat, PlayerRel::Any),
            "your precombat main phase" | "your first main phase" => {
                (TriggerStep::PrecombatMain, PlayerRel::You)
            }
            "your postcombat main phase" | "your second main phase" => {
                (TriggerStep::PostcombatMain, PlayerRel::You)
            }
            "the end of combat" | "end of combat" => (TriggerStep::EndOfCombat, PlayerRel::Any),
            "each player's draw step" => (TriggerStep::Draw, PlayerRel::Any),
            _ => return None,
        };
        return Some((
            TriggerCond::BeginningOf { step, whose },
            Sel::This,
            PlayerRef::ActivePlayer,
        ));
    }
    let r = l
        .strip_prefix("whenever ")
        .or_else(|| l.strip_prefix("when "))?;
    // Self triggers.
    let self_pairs: [(&str, TriggerCond); 12] = [
        ("~ enters", TriggerCond::EntersBattlefield(Filter::Source)),
        (
            "~ enters the battlefield",
            TriggerCond::EntersBattlefield(Filter::Source),
        ),
        ("~ dies", TriggerCond::Dies(Filter::Source)),
        (
            "~ is put into a graveyard from the battlefield",
            TriggerCond::Dies(Filter::Source),
        ),
        (
            "~ leaves the battlefield",
            TriggerCond::LeavesBattlefield(Filter::Source),
        ),
        ("~ attacks", TriggerCond::Attacks(Filter::Source)),
        ("~ blocks", TriggerCond::Blocks(Filter::Source)),
        (
            "~ becomes blocked",
            TriggerCond::BecomesBlocked(Filter::Source),
        ),
        (
            "~ blocks or becomes blocked",
            TriggerCond::BlocksOrBecomesBlocked(Filter::Source),
        ),
        (
            "~ attacks and isn't blocked",
            TriggerCond::AttacksUnblocked(Filter::Source),
        ),
        (
            "~ becomes tapped",
            TriggerCond::BecomesTapped(Filter::Source),
        ),
        (
            "~ becomes the target of a spell or ability an opponent controls",
            TriggerCond::BecomesTarget {
                filter: Filter::Source,
                by: PlayerRel::Opponent,
            },
        ),
    ];
    for (p, t) in self_pairs {
        if r == p {
            return Some((t, Sel::This, PlayerRef::You));
        }
    }
    if r == "you cast this spell" {
        return Some((
            TriggerCond::CastSpell {
                who: PlayerRel::You,
                filter: Filter::Source,
            },
            Sel::This,
            PlayerRef::You,
        ));
    }
    // Damage triggers.
    if let Some(x) = r.strip_prefix("~ deals combat damage to a player") {
        if x.is_empty() {
            return Some((
                TriggerCond::DealsDamage {
                    source: Filter::Source,
                    to: DamageRecipient::Player(PlayerRel::Any),
                    combat_only: true,
                },
                Sel::This,
                PlayerRef::TriggerPlayer,
            ));
        }
        if x == " or planeswalker" {
            return Some((
                TriggerCond::DealsDamage {
                    source: Filter::Source,
                    to: DamageRecipient::PlayerOrPlaneswalker(PlayerRel::Any),
                    combat_only: true,
                },
                Sel::This,
                PlayerRef::TriggerPlayer,
            ));
        }
    }
    if r == "~ deals combat damage to an opponent" {
        return Some((
            TriggerCond::DealsDamage {
                source: Filter::Source,
                to: DamageRecipient::Player(PlayerRel::Opponent),
                combat_only: true,
            },
            Sel::This,
            PlayerRef::TriggerPlayer,
        ));
    }
    if r == "~ deals damage to a player" || r == "~ deals damage to an opponent" {
        let rel = if r.ends_with("opponent") {
            PlayerRel::Opponent
        } else {
            PlayerRel::Any
        };
        return Some((
            TriggerCond::DealsDamage {
                source: Filter::Source,
                to: DamageRecipient::Player(rel),
                combat_only: false,
            },
            Sel::This,
            PlayerRef::TriggerPlayer,
        ));
    }
    if r == "~ deals combat damage" || r == "~ deals damage" {
        return Some((
            TriggerCond::DealsDamage {
                source: Filter::Source,
                to: DamageRecipient::Any,
                combat_only: r.contains("combat"),
            },
            Sel::This,
            PlayerRef::TriggerPlayer,
        ));
    }
    if r == "~ is dealt damage" {
        return Some((
            TriggerCond::IsDealtDamage {
                filter: Filter::Source,
                combat_only: false,
            },
            Sel::This,
            PlayerRef::You,
        ));
    }
    // Cast triggers.
    if let Some((who, rest)) = spell_caster(r) {
        let rest = rest.trim();
        if rest == "a spell" {
            return Some((
                TriggerCond::CastSpell {
                    who,
                    filter: Filter::Any,
                },
                Sel::TriggerSpell,
                PlayerRef::TriggerPlayer,
            ));
        }
        let rest2 = rest
            .strip_prefix("a ")
            .or_else(|| rest.strip_prefix("an "))?;
        let rest2 = rest2
            .strip_suffix(" spell")
            .or_else(|| rest2.strip_suffix(" spells"))?;
        let (f, _, tail) = parse_object_phrase(rest2)?;
        if !end(tail).is_empty() {
            return None;
        }
        return Some((
            TriggerCond::CastSpell { who, filter: f },
            Sel::TriggerSpell,
            PlayerRef::TriggerPlayer,
        ));
    }
    // Player triggers.
    let player_pairs: [(&str, TriggerCond); 10] = [
        (
            "you gain life",
            TriggerCond::GainsLife {
                who: PlayerRel::You,
            },
        ),
        (
            "you lose life",
            TriggerCond::LosesLife {
                who: PlayerRel::You,
            },
        ),
        (
            "an opponent loses life",
            TriggerCond::LosesLife {
                who: PlayerRel::Opponent,
            },
        ),
        (
            "you draw a card",
            TriggerCond::Draws {
                who: PlayerRel::You,
            },
        ),
        (
            "an opponent draws a card",
            TriggerCond::Draws {
                who: PlayerRel::Opponent,
            },
        ),
        (
            "a player draws a card",
            TriggerCond::Draws {
                who: PlayerRel::Any,
            },
        ),
        ("you attack", TriggerCond::PlayerAttacks(PlayerRel::You)),
        (
            "you cycle or discard a card",
            TriggerCond::Discards {
                who: PlayerRel::You,
                filter: Filter::Any,
            },
        ),
        (
            "you discard a card",
            TriggerCond::Discards {
                who: PlayerRel::You,
                filter: Filter::Any,
            },
        ),
        (
            "an opponent discards a card",
            TriggerCond::Discards {
                who: PlayerRel::Opponent,
                filter: Filter::Any,
            },
        ),
    ];
    for (p, t) in player_pairs {
        if r == p {
            return Some((t, obj(), PlayerRef::TriggerPlayer));
        }
    }
    if r == "you sacrifice a permanent" {
        return Some((
            TriggerCond::YouSacrifice(Filter::Any),
            obj(),
            PlayerRef::You,
        ));
    }
    // "[filter] enters [the battlefield] [under your control]"
    for suffix in [
        " enters the battlefield under your control",
        " enters under your control",
        " enters the battlefield",
        " enters",
    ] {
        if let Some(x) = r.strip_suffix(suffix) {
            let x = x
                .strip_prefix("a ")
                .or_else(|| x.strip_prefix("an "))
                .or_else(|| x.strip_prefix("one or more "))
                .unwrap_or(x);
            let (mut f, _, tail) = parse_object_phrase(x)?;
            if !end(tail).is_empty() {
                return None;
            }
            if suffix.contains("under your control") {
                f = Filter::and(vec![f, Filter::ControlledBy(PlayerRel::You)]);
            }
            return Some((
                TriggerCond::EntersBattlefield(f),
                obj(),
                PlayerRef::ControllerOf(Box::new(Sel::TriggerObject)),
            ));
        }
    }
    // "[filter] dies"
    if let Some(x) = r.strip_suffix(" dies").or_else(|| r.strip_suffix(" die")) {
        let x = x
            .strip_prefix("a ")
            .or_else(|| x.strip_prefix("an "))
            .or_else(|| x.strip_prefix("one or more "))
            .unwrap_or(x);
        let (f, _, tail) = parse_object_phrase(x)?;
        if !end(tail).is_empty() {
            return None;
        }
        return Some((
            TriggerCond::Dies(f),
            obj(),
            PlayerRef::ControllerOf(Box::new(Sel::TriggerLki)),
        ));
    }
    // "[filter] attacks"
    if let Some(x) = r.strip_suffix(" attacks") {
        let x = x
            .strip_prefix("a ")
            .or_else(|| x.strip_prefix("an "))
            .unwrap_or(x);
        let (f, _, tail) = parse_object_phrase(x)?;
        if !end(tail).is_empty() {
            return None;
        }
        return Some((TriggerCond::Attacks(f), obj(), PlayerRef::TriggerPlayer));
    }
    // "[filter] deals combat damage to a player"
    if let Some(x) = r.strip_suffix(" deals combat damage to a player") {
        let x = x
            .strip_prefix("a ")
            .or_else(|| x.strip_prefix("an "))
            .unwrap_or(x);
        let (f, _, tail) = parse_object_phrase(x)?;
        if !end(tail).is_empty() {
            return None;
        }
        return Some((
            TriggerCond::DealsDamage {
                source: f,
                to: DamageRecipient::Player(PlayerRel::Any),
                combat_only: true,
            },
            Sel::TriggerOtherObject,
            PlayerRef::TriggerPlayer,
        ));
    }
    // "a +1/+1 counter is put on ~"
    if r == "one or more +1/+1 counters are put on ~" || r == "a +1/+1 counter is put on ~" {
        return Some((
            TriggerCond::CountersPut {
                filter: Filter::Source,
                kind: Some(crate::types::counters::PLUS1.into()),
            },
            Sel::This,
            PlayerRef::You,
        ));
    }
    // "a land enters under your control" handled above; landfall ability word stripped.
    crate::oracle_ext::parse_trigger_ext(r)
}

/// "you cast", "an opponent casts", "a player casts".
fn spell_caster(r: &str) -> Option<(PlayerRel, &str)> {
    if let Some(x) = r.strip_prefix("you cast ") {
        return Some((PlayerRel::You, x));
    }
    if let Some(x) = r.strip_prefix("an opponent casts ") {
        return Some((PlayerRel::Opponent, x));
    }
    if let Some(x) = r.strip_prefix("a player casts ") {
        return Some((PlayerRel::Any, x));
    }
    None
}
