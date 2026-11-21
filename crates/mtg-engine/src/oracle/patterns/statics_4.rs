//! Characteristic statics IV (cards3-combat-statics):
//!
//! * "Until your next turn, you and planeswalkers you control gain protection from that
//!   player." (Eon Frolicker): protection from a player (CR 702.16k) for the player and
//!   for permanents, the player bound as the effect begins.
//! * "Target creature gains landwalk of each of the land types of the sacrificed land
//!   until end of turn." (Excavator): one landwalk ability per land type (CR 702.14a),
//!   determined from the sacrificed land as it last existed (CR 608.2h).
//! * "Target creature gains snow landwalk of the chosen type until end of turn." (Barbarian
//!   Guides): snow [type]walk (CR 702.14c).
//! * "Each creature target opponent controls phases out. Until the end of your next turn,
//!   they can't phase in." (Teferi, Timeless Voyager): "they" are the permanents phased
//!   out this way.

use super::{EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::keywords::{Keyword, KeywordKind};
use crate::oracle::effects::{player_ref, Builder};
use crate::oracle::phrases::{end, parse_object_phrase};

/// `Filter::Custom` marker in a landwalk keyword: "landwalk of each of the land types of
/// the sacrificed land", expanded into one landwalk per land type as the effect is created
/// (`Game::fix_mods`).
pub const LAND_TYPES_OF_SACRIFICED: &str = "statics4:land types of the sacrificed land";

fn keyword(kind: KeywordKind, filter: Option<Filter>, text: &str) -> Keyword {
    Keyword {
        kind,
        n: None,
        cost: None,
        costs: vec![],
        filter,
        text: Some(text.into()),
        x: None,
    }
}

/// A leading or trailing duration of a "gain" instruction.
fn split_duration(l: &str) -> (Duration, String) {
    for (p, d) in [
        ("until your next turn, ", Duration::UntilYourNextTurn),
        ("until end of turn, ", Duration::EndOfTurn),
        ("until the end of your next turn, ", Duration::UntilEndOfYourNextTurn),
    ] {
        if let Some(r) = l.strip_prefix(p) {
            return (d, r.to_string());
        }
    }
    for (s, d) in [
        (" until your next turn", Duration::UntilYourNextTurn),
        (" until end of turn", Duration::EndOfTurn),
        (" until the end of your next turn", Duration::UntilEndOfYourNextTurn),
    ] {
        if let Some(r) = l.strip_suffix(s) {
            return (d, r.to_string());
        }
    }
    (Duration::Permanent, l.to_string())
}

/// "[you and] [permanents] gain protection from [player]".
fn p_protection_from_player(l: &str, b: &mut Builder) -> Option<Effect> {
    let (duration, l) = split_duration(end(l.trim()));
    if matches!(duration, Duration::Permanent) {
        return None;
    }
    let (subject, who) = l.split_once(" gain protection from ")?;
    let n = b.targets.len();
    let (p, rest) = player_ref(who, b)?;
    if !rest.trim().is_empty()
        || b.targets.len() != n
        || crate::oracle::patterns::oracle_hardening_referents::is_no_player_referent(&p)
        || matches!(p, PlayerRef::You)
    {
        b.targets.truncate(n);
        return None;
    }
    let quality = Filter::ControlledByPlayer(Box::new(p));
    let (you, objects) = if subject == "you" {
        (true, None)
    } else if let Some(o) = subject.strip_prefix("you and ") {
        (true, Some(o))
    } else {
        (false, Some(subject))
    };
    let mut out = Vec::new();
    if you {
        out.push(Effect::AddPlayerEffect {
            who: PlayerRef::You,
            effect: PlayerModification::ProtectionFrom(quality.clone()),
            duration: duration.clone(),
        });
    }
    if let Some(o) = objects {
        let (f, plural, tail) = parse_object_phrase(o)?;
        if !plural || !tail.trim().is_empty() {
            return None;
        }
        out.push(Effect::Modify {
            what: Sel::All(f),
            mods: vec![Modification::AddKeyword(keyword(
                KeywordKind::Protection,
                Some(quality),
                "protection from that player",
            ))],
            duration,
        });
    }
    Some(Effect::seq(out))
}

inventory::submit! { EffectPattern { name: "statics 4: you and [permanents] gain protection from that player", priority: 90, parse: p_protection_from_player } }

/// "[target] gains landwalk of each of the land types of the sacrificed land [duration]",
/// "[target] gains snow landwalk of the chosen type [duration]".
fn p_landwalk_of(l: &str, b: &mut Builder) -> Option<Effect> {
    let (duration, l) = split_duration(end(l.trim()));
    let (subject, kw) = l
        .split_once(" gains ")
        .or_else(|| l.split_once(" gain "))?;
    let k = match kw {
        "landwalk of each of the land types of the sacrificed land" => keyword(
            KeywordKind::Landwalk,
            Some(Filter::Custom(LAND_TYPES_OF_SACRIFICED.into())),
            kw,
        ),
        "snow landwalk of the chosen type" => keyword(
            KeywordKind::Landwalk,
            Some(Filter::and(vec![
                Filter::Supertype(crate::types::Supertype::Snow),
                Filter::ChosenType,
            ])),
            kw,
        ),
        _ => return None,
    };
    let n = b.targets.len();
    let (what, rest) = crate::oracle::effects::object_ref(subject, b)?;
    if !rest.trim().is_empty() || matches!(what, Sel::None) {
        b.targets.truncate(n);
        return None;
    }
    Some(Effect::Modify {
        what,
        mods: vec![Modification::AddKeyword(k)],
        duration,
    })
}

inventory::submit! { EffectPattern { name: "statics 4: gains landwalk of the sacrificed land's types / snow landwalk", priority: 90, parse: p_landwalk_of } }

/// The last instruction of `e`, if it phases permanents out.
fn phases_out(e: &Effect) -> bool {
    match e {
        Effect::Seq(v) => v.last().is_some_and(phases_out),
        Effect::PhaseOut { what } => matches!(what, Sel::All(_)),
        _ => false,
    }
}

/// "Until the end of your next turn, they can't phase in." after "Each creature target
/// opponent controls phases out.": the permanents phased out this way.
fn f_they_cant_phase_in(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l.trim());
    if !phases_out(prev) {
        return false;
    }
    let rewritten = ["they can't phase in", "those creatures can't phase in", "those permanents can't phase in"]
        .iter()
        .find(|p| l.contains(*p))
        .map(|p| l.replacen(p, "it can't phase in", 1));
    let Some(rewritten) = rewritten else {
        return false;
    };
    let saved = b.it.clone();
    b.it = Sel::Var(vars::PHASED_OUT);
    let n = b.targets.len();
    let e = crate::oracle::effects::parse_sentence(&rewritten, b);
    b.it = saved;
    let Some(e) = e.filter(|_| b.targets.len() == n) else {
        b.targets.truncate(n);
        return false;
    };
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![old, e]);
    true
}

inventory::submit! { FollowupPattern { name: "statics 4: they can't phase in (the permanents phased out)", priority: 60, apply: f_they_cant_phase_in } }

/// "As long as ~ is in your graveyard, it has flashback {2}{G}.", "Corrupted — As long as an
/// opponent has three or more poison counters and ~ is in your graveyard, it has flashback
/// {N}{G}." (Viral Spawning): a static ability that functions in the graveyard (CR 113.6b)
/// granting the card a keyword that works there.
fn s_in_graveyard_has_keyword(
    text: &str,
    ctx: &crate::oracle::CompileContext,
) -> Option<Vec<Ability>> {
    // A permanent card's statics are read by the static grammar; an instant's or
    // sorcery's lines are spell abilities unless a pattern says otherwise.
    if !ctx.is_spell() {
        return None;
    }
    let lower = text.trim().to_lowercase();
    let l = end(&lower);
    let r = l.strip_prefix("as long as ")?;
    let (cond, granted) = r.split_once(", it has ")?;
    let other = if cond == "~ is in your graveyard" {
        None
    } else {
        Some(cond.strip_suffix(" and ~ is in your graveyard")?)
    };
    // The keyword as written (its cost's mana symbols are case-sensitive).
    let at = text.to_lowercase().rfind(", it has ")?;
    let granted_text = text[at + ", it has ".len()..].trim().trim_end_matches('.');
    if granted_text.to_lowercase() != granted {
        return None;
    }
    let kws: Vec<Keyword> = crate::oracle::keywords::parse_keyword_line(granted_text, ctx)?
        .into_iter()
        .map(|a| match &a.kind {
            AbilityKind::Keyword(k) => Some(k.clone()),
            _ => None,
        })
        .collect::<Option<_>>()?;
    let [kw] = &kws[..] else { return None };
    // Keywords that work from the graveyard and need their cost written out.
    if !matches!(
        kw.kind,
        KeywordKind::Flashback | KeywordKind::Unearth | KeywordKind::Embalm | KeywordKind::Eternalize
    ) || kw.cost.is_none()
    {
        return None;
    }
    let in_graveyard = Condition::SelMatches(
        Sel::This,
        Filter::and(vec![
            Filter::InZone(ZoneKind::Graveyard),
            Filter::OwnedBy(PlayerRel::You),
            Filter::Custom(crate::zones::STILL_THERE.into()),
        ]),
    );
    let condition = match other {
        None => in_graveyard,
        Some(c) => Condition::And(vec![
            crate::oracle::statics::parse_condition(c, ctx)?,
            in_graveyard,
        ]),
    };
    let mut st = StaticAbility::new(StaticEffect::Continuous {
        affected: Filter::Source,
        mods: vec![Modification::AddKeyword(kw.clone())],
    });
    st.condition = Some(condition);
    st.zone = FunctionZone::Graveyard;
    Some(vec![AbilityDef::new(AbilityKind::Static(st), text)])
}

inventory::submit! { super::AbilityPattern { name: "statics 4: as long as ~ is in your graveyard, it has [keyword] (instants and sorceries)", priority: 90, parse: s_in_graveyard_has_keyword } }

/// "Return it to the battlefield. It's an Aura enchantment with ... and \"...\". ~ loses
/// all other abilities." (Harold and Bob, First Numens): the permanent it returns as
/// loses the abilities other than those the rider gives it (layer 6, in order:
/// CR 613.1f, 613.7).
fn f_loses_all_other_abilities(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l.trim());
    // The sentence after a quote ending in a period is read with it: "It's an Aura
    // enchantment with ... and \"...\" ~ loses all other abilities."
    if let Some(r) = l
        .strip_suffix("\" ~ loses all other abilities")
        .filter(|r| r.starts_with("it's ") || r.starts_with("it is "))
    {
        return super::characteristic_riders::rider_followup(
            &format!("{r}\" and loses all other abilities"),
            prev,
            b,
        );
    }
    if !matches!(l, "~ loses all other abilities" | "it loses all other abilities") {
        return false;
    }
    let Some(to) = super::characteristic_riders::last_battlefield_move(prev) else {
        return false;
    };
    let gives = to
        .with_mods
        .iter()
        .any(|m| matches!(m, Modification::AddAbility(_) | Modification::AddKeyword(_)));
    if !gives
        || to
            .with_mods
            .iter()
            .any(|m| matches!(m, Modification::RemoveAllAbilities))
    {
        return false;
    }
    to.with_mods.insert(0, Modification::RemoveAllAbilities);
    true
}

inventory::submit! { FollowupPattern { name: "statics 4: ~ loses all other abilities (after a characteristic rider)", priority: 1140, apply: f_loses_all_other_abilities } }

/// The creature an "as enters" text-box exchange is with.
const EXCHANGE_WITH: Var = vars::USER + 13374;

/// "As ~ enters, you may exchange his text box and another creature's." (Deadpool, Trading
/// Card): as it enters, its controller may choose another creature; the two exchange text
/// boxes (CR 612.5) from then on.
fn as_enters_exchange_text_box(block: &str, ctx: &crate::oracle::CompileContext) -> Option<Vec<Ability>> {
    if !ctx.is_permanent() || block.contains('\n') {
        return None;
    }
    let text = crate::oracle::strip_ability_word(block.trim());
    let lower = text.to_lowercase();
    let l = end(&lower);
    let r = l.strip_prefix("as ~ enters, you may exchange ")?;
    if !matches!(
        r,
        "his text box and another creature's"
            | "her text box and another creature's"
            | "its text box and another creature's"
            | "~'s text box and another creature's"
    ) || !ctx.type_line.card_types.contains(crate::types::CardType::Creature)
    {
        return None;
    }
    let other = Sel::Choose {
        chooser: PlayerRef::You,
        filter: Filter::and(vec![
            Filter::Other,
            Filter::Type(crate::types::CardType::Creature),
            Filter::InZone(ZoneKind::Battlefield),
        ]),
        count: Value::c(1),
        up_to: false,
        store: None,
    };
    let body = Effect::May {
        who: PlayerRef::You,
        effect: Box::new(Effect::seq(vec![
            Effect::Store {
                var: EXCHANGE_WITH,
                sel: other,
            },
            // Performed on the permanent as it's put onto the battlefield (CR 614.1c).
            Effect::OnEntry(Box::new(Effect::Modify {
                what: Sel::Union(vec![Sel::This, Sel::Var(EXCHANGE_WITH)]),
                mods: vec![Modification::ExchangeText],
                duration: Duration::Permanent,
            })),
        ])),
    };
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Replacement(
            ReplacementDef {
                event: ReplacementEvent::EntersBattlefield(Filter::Source),
                action: ReplacementAction::AsEnters(Box::new(body)),
                self_replacement: false,
                optional: false,
            },
        ))),
        block,
    )])
}

inventory::submit! { super::AbilityPattern { name: "statics 4: as ~ enters, you may exchange its text box and another creature's", priority: 90, parse: as_enters_exchange_text_box } }

/// "Draw two cards, then you may exile a nonland card from your hand with a number of time
/// counters on it equal to its mana value. Then target opponent does the same. Cards
/// exiled this way that don't have suspend gain suspend." (The Wedding of River Song): the
/// cards the spell's exiles linked to it — this resolution's, as a spell's object is new
/// each time it's cast (CR 607.2a, 702.62a).
fn f_cards_exiled_this_way_gain_suspend(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    if end(l.trim()) != "cards exiled this way that don't have suspend gain suspend" {
        return false;
    }
    let s = format!("{prev:?}");
    if !s.contains("\"time\"") || !s.contains("link: true") || !_b.ctx.is_spell() {
        return false;
    }
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![
        old,
        super::r702_062_gains_suspend::gains_suspend(Sel::Linked),
    ]);
    true
}

inventory::submit! { FollowupPattern { name: "statics 4: cards exiled this way that don't have suspend gain suspend", priority: 60, apply: f_cards_exiled_this_way_gain_suspend } }

/// "~ can attack players who attacked you during their last turn as though it didn't have
/// defender" (Weathered Sentinels): CR 702.3b doesn't stop it from attacking those
/// players. "players who [did it]" is read as the condition "a player [did it]" about each
/// player in turn.
fn s_attack_players_despite_defender(
    block: &str,
    _norm: &str,
    ctx: &crate::oracle::CompileContext,
) -> Option<Vec<Ability>> {
    let text = block.trim();
    let lower = text.to_lowercase();
    let who = end(&lower)
        .strip_prefix("~ can attack players who ")?
        .strip_suffix(" as though it didn't have defender")?;
    let cond = crate::oracle::statics::parse_condition(&format!("a player {who}"), ctx)?;
    // "a player [did it]" over all players: the comparison of each one's own value.
    let Condition::Compare(Value::OverPlayers(AggOp::Max, PlayerFilter::Any, v), cmp, n) = cond
    else {
        return None;
    };
    if !matches!(cmp, Cmp::Ge | Cmp::Gt) {
        return None;
    }
    let st = StaticAbility::new(StaticEffect::Restriction(
        Restriction::AttackPlayersDespiteDefender {
            attackers: Filter::Source,
            players: Box::new(Condition::Compare(*v, cmp, n)),
        },
    ));
    Some(vec![AbilityDef::new(AbilityKind::Static(st), text)])
}

inventory::submit! { super::StaticPattern { name: "statics 4: ~ can attack players who [did it] as though it didn't have defender", priority: 90, parse: s_attack_players_despite_defender } }
