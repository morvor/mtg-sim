//! Parsing triggered abilities: "When/Whenever/At [trigger], [if condition,] [effect]."

use super::effects::parse_trigger_body;
use super::phrases::*;
use super::CompileContext;
use crate::ability::*;

pub fn parse_triggered(text: &str, ctx: &CompileContext) -> Option<Ability> {
    let t = text.trim();
    // Split trigger condition from effect at the first comma outside quotes that
    // follows the trigger phrase. A condition with a list in it ("Whenever you cast an
    // instant, sorcery, or Wizard spell, ...") ends at a later comma: the first one at
    // which the whole ability can be read.
    trigger_splits(t)
        .into_iter()
        .find_map(|(c, e)| parse_triggered_at(text, c, e, ctx))
}

/// The triggered ability `text`, read as the trigger condition `cond_s` followed by
/// `eff_s`.
fn parse_triggered_at(
    text: &str,
    cond_s: &str,
    eff_s: &str,
    ctx: &CompileContext,
) -> Option<Ability> {
    let lower = cond_s.to_lowercase();
    let (trigger, it, it_player) = parse_trigger_condition(&lower)?;
    let mut eff = eff_s.trim();
    // "This ability triggers only once each turn." is a rule about the ability, not part
    // of its effect.
    let mut once_per_turn = false;
    for suffix in [
        "this ability triggers only once each turn.",
        "this ability triggers only once each turn",
    ] {
        let el = eff.to_lowercase();
        if el.ends_with(suffix) {
            eff = eff[..eff.len() - suffix.len()].trim_end();
            once_per_turn = true;
            break;
        }
    }
    // Intervening "if" clause (CR 603.4).
    let mut intervening = None;
    let mut body_it: Option<Sel> = None;
    let subject_is_source;
    let el = eff.to_lowercase();
    if let Some(r) = el.strip_prefix("if ") {
        // The condition ends at the first comma that doesn't continue a list of adjectives
        // ("if that player controls a nonblack, nonland permanent, ...").
        let continues_list = |before: &str, rest: &str| {
            let last = before.rsplit(' ').next().unwrap_or("");
            let next = split_word(rest).0.trim_end_matches(',');
            matches!(next, "or" | "and" | "and/or")
                || (adjective(last).is_some()
                    && head_noun(last).is_none()
                    && (adjective(next).is_some() || head_noun(next).is_some()))
        };
        let split = r
            .match_indices(", ")
            .map(|(i, _)| i)
            .find(|&i| !continues_list(&r[..i], &r[i + 2..]))
            .map(|i| (&r[..i], &r[i + 2..]));
        if let Some((c, _)) = split {
            // Conditions are parsed without a referent: "it" in them means the source
            // ("When ~ enters, if you cast it"), so it can't refer to another object
            // ("Whenever a creature enters, if you cast it" is about that creature).
            let mentions_it = c
                .split(|ch: char| !ch.is_alphanumeric() && ch != '\'')
                .any(|w| matches!(w, "it" | "its" | "it's"));
            let parsed = super::statics::parse_condition(c, ctx).or_else(|| {
                super::patterns::that_player_conditions::that_player_condition(c, &it_player)
            });
            // A condition read as being about the triggering object ("if it had counters
            // on it", CR 603.10a; "if it doesn't have the same name as another creature
            // you control") has the trigger's referent.
            let about_trigger_object = matches!(
                parsed,
                Some(Condition::SelMatches(
                    Sel::TriggerLki | Sel::TriggerObject,
                    _
                ))
            ) || parsed.as_ref().is_some_and(|p| {
                let d = format!("{p:?}");
                d.contains("TriggerObject") || d.contains("TriggerLki")
            });
            // Otherwise the condition's pronouns refer to the trigger's object and player
            // ("if it had a +1/+1 counter on it", "if that player has no cards in hand").
            // "If enchanted creature is untapped, tap it": the condition's object is what
            // the effect's "it" refers to.
            let parsed = if (mentions_it && !matches!(it, Sel::This) && !about_trigger_object)
                || parsed.is_none()
            {
                match super::patterns::conditions_referents::intervening(c, ctx, &it, &it_player) {
                    Some((cond, subject)) => {
                        match (&subject, &it) {
                            (Some(sel @ Sel::AttachedTo), Sel::This) => body_it = Some(sel.clone()),
                            // "Whenever you cast an instant or sorcery spell, if ~ has fewer
                            // than three charge counters on it, put a charge counter on it."
                            (Some(Sel::This), _) => body_it = Some(Sel::This),
                            _ => {}
                        }
                        Some(cond)
                    }
                    None if mentions_it && !matches!(it, Sel::This) && !about_trigger_object => {
                        return None
                    }
                    None => parsed,
                }
            } else {
                parsed
            };
            if let Some(cond) = parsed {
                // "Whenever you scry, if ~ is tapped, you may untap it": with no object of
                // the trigger event's own, "it" is the condition's subject.
                if matches!(it, Sel::None) && c.starts_with("~ ") && body_it.is_none() {
                    body_it = Some(Sel::This);
                }
                intervening = Some(cond);
                eff = &eff[3 + c.len() + 2..];
                // "..., if ~ is an enchantment, it becomes a 3/3 Knight creature": the
                // subject "it" is the condition's.
                if c.starts_with("~ ") {
                    if let Some(r) = eff.strip_prefix("it ") {
                        subject_is_source = format!("~ {r}");
                        eff = &subject_is_source;
                    }
                }
            }
        }
    }
    // "..., if this card is in your graveyard, you may pay {3}. If you do, you may cast
    // it ...": "it" is the card the condition is about.
    let it = if intervening
        .as_ref()
        .is_some_and(super::patterns::graveyard_order::requires_source_in_graveyard)
    {
        Sel::This
    } else {
        body_it.unwrap_or(it)
    };
    // A trigger condition with no single referent for "it" (e.g. several conditions
    // joined by "and whenever", or "whenever chaos ensues") gives the body's pronouns no
    // antecedent, as in a spell's text, until an instruction introduces one ("choose
    // target creature. ... that creature"); a pronoun left without one isn't understood.
    let it = if matches!(it, Sel::None) && mentions_object_pronoun(eff) {
        super::patterns::oracle_hardening_referents::no_referent()
    } else {
        it
    };
    if matches!(it_player, PlayerRef::Iterated) && eff.to_lowercase().contains("that player") {
        return None;
    }
    let mut body = parse_trigger_body(eff, ctx, it, it_player)?;
    // "Whenever you cast your first spell with {X} in its mana cost each turn, put X +1/+1
    // counters on ~": a triggered ability has no X of its own; X is the spell's (CR 107.3e).
    if casts_spell_with_x(&trigger) {
        let x = Value::XOf(Box::new(Sel::TriggerSpell));
        body.effect = super::patterns::r107_numbers::substitute_x(&body.effect, &x)?;
    }
    // CR 605.1b: a triggered ability without targets that triggers from resolving a mana
    // ability and could add mana is a mana ability (it resolves immediately, CR 605.4a).
    let is_mana_ability = is_triggered_mana_ability(&trigger, &body);
    let mut tr = TriggeredAbility::new(trigger, body);
    tr.is_mana_ability = is_mana_ability;
    tr.intervening_if = intervening;
    tr.once_per_turn = once_per_turn;
    tr.zone = trigger_zone(&tr.trigger, &eff.to_lowercase());
    // "At the beginning of your upkeep, if ~ is in your graveyard, ...": it functions from
    // the graveyard (CR 113.6).
    if tr
        .intervening_if
        .as_ref()
        .is_some_and(super::patterns::graveyard_order::requires_source_in_graveyard)
    {
        tr.zone = FunctionZone::Graveyard;
    }
    // "if ~ is in the command zone" / "if ~ is exiled": it functions there (CR 113.6b).
    if let Some(z) = tr
        .intervening_if
        .as_ref()
        .and_then(super::patterns::conditions_state::required_source_zone)
    {
        tr.zone = z;
    }
    // CR 113.6: an instant or sorcery is never on the battlefield, so a triggered ability
    // that would only function there can't be what the text means.
    if ctx.is_spell() && tr.zone == FunctionZone::Battlefield {
        return None;
    }
    Some(AbilityDef::new(AbilityKind::Triggered(tr), text))
}

/// The zone a triggered ability functions from (CR 113.6): the battlefield, unless the
/// trigger can't trigger from there — "when you cast ~" (the stack) and "when ~ is put
/// into a graveyard from anywhere" (the graveyard; from the battlefield it triggers by
/// looking back in time), CR 113.6k — or its effect moves ~ out of the graveyard ("return
/// ~ from your graveyard to your hand") and the trigger condition doesn't put it there
/// (CR 113.6m).
pub(crate) fn trigger_zone(trigger: &TriggerCond, eff: &str) -> FunctionZone {
    // "Whenever a creature you control dies while ~ is in your graveyard" (CR 113.6).
    if super::patterns::trigger_grammar_events::requires_source_in_graveyard(trigger) {
        return FunctionZone::Graveyard;
    }
    match trigger {
        TriggerCond::CastSpell {
            filter: Filter::Source,
            ..
        } => return FunctionZone::Stack,
        TriggerCond::ZoneChange {
            filter,
            from,
            to: Some(ZoneKind::Graveyard),
        } if mentions_source(filter) && *from != Some(ZoneKind::Battlefield) => {
            return FunctionZone::Graveyard
        }
        // CR 702.29c: "when you cycle ~" triggers from whatever zone the card winds up in;
        // so does "when you discard ~" (the graveyard, or exile with madness).
        TriggerCond::Cycled {
            filter: Filter::Source,
            ..
        }
        | TriggerCond::Discards {
            filter: Filter::Source,
            ..
        } => return FunctionZone::Anywhere,
        TriggerCond::Dies(f)
        | TriggerCond::LeavesBattlefield(f)
        | TriggerCond::ZoneChange { filter: f, .. }
            if mentions_source(f) || matches!(f, Filter::AttachedToSource) =>
        {
            // (An Aura's "when enchanted creature dies, return ~ from your graveyard"
            // triggers while the Aura is on the battlefield, CR 603.10a.)
            return FunctionZone::Battlefield;
        }
        // Several trigger conditions (CR 603.1b), such as "When you cycle this card and
        // when this creature dies": one that triggers from wherever the card is combined
        // with the object's own leaves-the-battlefield conditions, which trigger by
        // looking back at it on the battlefield (CR 603.10a), functions from anywhere.
        TriggerCond::AnyOf(conds) => {
            let zones: Vec<FunctionZone> = conds.iter().map(|c| trigger_zone(c, eff)).collect();
            let own_ltb = |c: &TriggerCond| {
                matches!(c, TriggerCond::Dies(f) | TriggerCond::LeavesBattlefield(f)
                    if mentions_source(f))
            };
            if zones.contains(&FunctionZone::Anywhere)
                && conds
                    .iter()
                    .zip(&zones)
                    .all(|(c, z)| *z == FunctionZone::Anywhere || own_ltb(c))
            {
                return FunctionZone::Anywhere;
            }
            // "When you cast this spell and whenever this creature attacks" / "... and when
            // this creature dies": the cast condition triggers from the stack, the others
            // only for the permanent itself (attacking, or leaving the battlefield, which
            // looks back at it): from anywhere.
            let own_permanent =
                |c: &TriggerCond| own_ltb(c) || matches!(c, TriggerCond::Attacks(Filter::Source));
            if zones.contains(&FunctionZone::Stack)
                && conds
                    .iter()
                    .zip(&zones)
                    .all(|(c, z)| *z == FunctionZone::Stack || own_permanent(c))
            {
                return FunctionZone::Anywhere;
            }
        }
        _ => {}
    }
    if super::without_quotes(eff).contains("~ from your graveyard") {
        return FunctionZone::Graveyard;
    }
    FunctionZone::Battlefield
}

fn mentions_source(f: &Filter) -> bool {
    match f {
        Filter::Source => true,
        Filter::And(v) | Filter::Or(v) => v.iter().any(mentions_source),
        _ => false,
    }
}

/// Whether effect text uses a pronoun that refers back to the trigger's object.
fn mentions_object_pronoun(eff: &str) -> bool {
    let l = format!(" {} ", eff.to_lowercase().replace(['.', ','], " "));
    [
        " it ",
        " its ",
        " it's ",
        " them ",
        " that creature",
        " that card",
        " that permanent",
        " that spell",
        " that token",
        " those ",
    ]
    .iter()
    .any(|p| l.contains(p))
}

/// Every way to split `t` at a comma outside quotes, first comma first.
fn trigger_splits(t: &str) -> Vec<(&str, &str)> {
    let mut in_quote = false;
    let mut out = Vec::new();
    for (i, ch) in t.char_indices() {
        match ch {
            '"' => in_quote = !in_quote,
            ',' if !in_quote => out.push((&t[..i], &t[i + 1..])),
            _ => {}
        }
    }
    out
}

/// Returns (trigger, what "it" refers to, what "that player" refers to).
///
/// The built-in forms below are tried first, then the patterns registered in
/// `oracle/patterns/` (which receive the text after "when"/"whenever", or the whole text
/// for "at ..." conditions).
pub fn parse_trigger_condition(l: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    let l = l.trim();
    if let Some(x) = core_trigger_condition(l) {
        return Some(x);
    }
    let r = l
        .strip_prefix("whenever ")
        .or_else(|| l.strip_prefix("when "))
        .unwrap_or(l);
    crate::oracle_ext::parse_trigger_ext(r)
}

fn core_trigger_condition(l: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    let obj = || Sel::TriggerObject;
    // CR 511.2: "at end of combat" triggers as the end of combat step begins.
    if l == "at end of combat" {
        return Some((
            TriggerCond::BeginningOf {
                step: TriggerStep::EndOfCombat,
                whose: PlayerRel::Any,
            },
            Sel::This,
            PlayerRef::ActivePlayer,
        ));
    }
    // "At the beginning of ..."
    if let Some(r) = l.strip_prefix("at the beginning of ") {
        let (step, whose) = match r {
            "your upkeep" => (TriggerStep::Upkeep, PlayerRel::You),
            "each upkeep" | "each player's upkeep" => (TriggerStep::Upkeep, PlayerRel::Any),
            "each opponent's upkeep" => (TriggerStep::Upkeep, PlayerRel::Opponent),
            "your end step" => (TriggerStep::End, PlayerRel::You),
            // CR 513.1a: "at end of turn" was errata'd to "at the beginning of the end step".
            "each end step" | "the end step" => (TriggerStep::End, PlayerRel::Any),
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
            // "your second main phase" counts main phases (CR 505.1b): see
            // patterns/r500_turn_structure.rs.
            "your postcombat main phase" => (TriggerStep::PostcombatMain, PlayerRel::You),
            "the end of combat" | "end of combat" => (TriggerStep::EndOfCombat, PlayerRel::Any),
            "each player's draw step" => (TriggerStep::Draw, PlayerRel::Any),
            _ => return None,
        };
        // "That player" is the player whose step it is: the active player, or with shared
        // team turns each player on the active team the ability triggers for (CR 805.4d).
        // On "your [step]" that's you, whom the text calls "you": "that player" then
        // names a player the instructions mention ("target opponent").
        let it_player = if matches!(whose, PlayerRel::You) {
            PlayerRef::You
        } else {
            PlayerRef::TriggerPlayer
        };
        return Some((
            TriggerCond::BeginningOf { step, whose },
            Sel::This,
            it_player,
        ));
    }
    let r = l
        .strip_prefix("whenever ")
        .or_else(|| l.strip_prefix("when "))?;
    // "enchanted creature"/"equipped creature" without an article is the object this
    // permanent is attached to, not any enchanted/equipped creature: see
    // patterns/triggers.rs.
    for p in ["enchanted ", "equipped ", "fortified "] {
        if r.starts_with(p) {
            return None;
        }
    }
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
            // "..., that player loses 4 life": the player who controls the spell or
            // ability that targeted it.
            let who = match t {
                TriggerCond::BecomesTarget { .. } => PlayerRef::TriggerPlayer,
                _ => PlayerRef::You,
            };
            return Some((t, Sel::This, who));
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
    // "~ deals damage" and "~ is dealt damage" trigger once per batch of simultaneous
    // damage: see patterns/triggers.rs.
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
        // "another [quality] spell": a spell other than this object (e.g. a commander's
        // eminence ability, which can trigger while it's in the command zone).
        let (another, rest2) = match rest.strip_prefix("another ") {
            Some(x) => (true, x),
            None => (
                false,
                rest.strip_prefix("a ")
                    .or_else(|| rest.strip_prefix("an "))?,
            ),
        };
        let rest2 = rest2
            .strip_suffix(" spell")
            .or_else(|| rest2.strip_suffix(" spells"))?;
        let (f, _, tail) = parse_object_phrase(rest2)?;
        if !end(tail).is_empty() {
            return None;
        }
        let f = permanent_spell(f);
        let f = if another {
            Filter::And(vec![Filter::Other, f])
        } else {
            f
        };
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
            // "Whenever you ...": the text calls that player "you", so "they" and "that
            // player" name someone else.
            let it_player = if p.starts_with("you ") {
                PlayerRef::You
            } else {
                PlayerRef::TriggerPlayer
            };
            return Some((t, obj(), it_player));
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
            // "one or more ..." triggers once per batch (CR 603.2c): see patterns/triggers.rs.
            if x.starts_with("one or more ") {
                return None;
            }
            let x = x
                .strip_prefix("a ")
                .or_else(|| x.strip_prefix("an "))
                .unwrap_or(x);
            let (mut f, _, tail) = parse_object_phrase(x)?;
            if !end(tail).is_empty() {
                return None;
            }
            // "Whenever a nonland permanent an opponent owns enters under your control,
            // they lose life ...": "they" is the owner the subject names.
            let who = if suffix.contains("under your control")
                && super::patterns::trigger_grammar_filters::names_other_owner(&f)
            {
                PlayerRef::OwnerOf(Box::new(Sel::TriggerObject))
            } else {
                PlayerRef::ControllerOf(Box::new(Sel::TriggerObject))
            };
            if suffix.contains("under your control") {
                f = Filter::and(vec![f, Filter::ControlledBy(PlayerRel::You)]);
            }
            return Some((TriggerCond::EntersBattlefield(f), obj(), who));
        }
    }
    // "[filter] dies"
    if let Some(x) = r.strip_suffix(" dies").or_else(|| r.strip_suffix(" die")) {
        if x.starts_with("one or more ") {
            return None;
        }
        let x = x
            .strip_prefix("a ")
            .or_else(|| x.strip_prefix("an "))
            .unwrap_or(x);
        let (f, _, tail) = parse_object_phrase(x)?;
        if !end(tail).is_empty() {
            return None;
        }
        // "it" is the creature that died: its last known information for values ("its
        // power"), and the card it became for actions ("return it", CR 400.7e).
        return Some((
            TriggerCond::Dies(f),
            Sel::TriggerLki,
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
                each: r.starts_with("a "),
            },
            Sel::This,
            PlayerRef::You,
        ));
    }
    // "a land enters under your control" handled above; landfall ability word stripped.
    None
}

/// In a description of a spell ("a Faerie or Wizard permanent spell", "a green permanent
/// spell"), "permanent" means a spell with a permanent type (CR 110.4b), not an object on
/// the battlefield.
fn permanent_spell(f: Filter) -> Filter {
    match f {
        Filter::Permanent => Filter::PermanentCard,
        Filter::And(v) => Filter::And(v.into_iter().map(permanent_spell).collect()),
        Filter::Or(v) => Filter::Or(v.into_iter().map(permanent_spell).collect()),
        Filter::Not(x) => Filter::not(permanent_spell(*x)),
        other => other,
    }
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

/// A trigger on casting a spell with {X} in its mana cost.
fn casts_spell_with_x(t: &TriggerCond) -> bool {
    fn has_x(f: &Filter) -> bool {
        match f {
            Filter::HasX => true,
            Filter::And(v) => v.iter().any(has_x),
            _ => false,
        }
    }
    match t {
        TriggerCond::CastSpell { filter, .. } => has_x(filter),
        TriggerCond::FirstTimeEachTurn(t) => casts_spell_with_x(t),
        _ => false,
    }
}
