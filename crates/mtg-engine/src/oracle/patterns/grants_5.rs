//! Grant grammar V (cards4-referents-statics, family `granted-abilities`).
//!
//! - "At the beginning of your upkeep, exile the top card of your library. For as long as
//!   it remains exiled, it has "You may cast this card from exile as long as you've cast
//!   another spell this turn."" (Monk Class): a permission to cast the exiled card that
//!   lasts as long as it stays exiled (CR 113.6b: the granted static ability functions in
//!   exile; CR 400.7: once it leaves exile it's a new object), with the condition of the
//!   granted ability, read as "For as long as it remains exiled, you may cast it as long as
//!   you've cast another spell this turn."

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_sentence, Builder};
use crate::oracle::phrases::end;

/// "for as long as it remains exiled, it has "you may cast ~ from exile [condition].""
fn exiled_card_has_cast_permission(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let r = l.strip_prefix("for as long as it remains exiled, it has \"you may cast ~ from exile")?;
    let cond = r.strip_suffix(".\"").or_else(|| r.strip_suffix('"'))?;
    if cond.contains('"') {
        return None;
    }
    parse_sentence(
        &format!("for as long as it remains exiled, you may cast it{cond}"),
        b,
    )
}

inventory::submit! { EffectPattern { name: "grants 5: for as long as it remains exiled, it has \"you may cast ~ from exile ...\"", priority: 100, parse: exiled_card_has_cast_permission } }

/// The last token creation of an effect (looking into sequences, conditions and delayed
/// triggered abilities).
fn last_token_spec(e: &mut Effect) -> Option<&mut TokenSpec> {
    match e {
        Effect::CreateToken { spec, .. } => Some(spec),
        Effect::Seq(v) => v.iter_mut().rev().find_map(|x| {
            if matches!(x, Effect::Store { .. } | Effect::StoreValue { .. }) {
                None
            } else {
                Some(last_token_spec(x))
            }
        })?,
        Effect::If {
            then, otherwise, ..
        } if matches!(**otherwise, Effect::Noop) => last_token_spec(then),
        Effect::DelayedTrigger { body, .. } => last_token_spec(&mut body.effect),
        _ => None,
    }
}

/// "... then create a 0/1 colorless Eldrazi Spawn creature token for each card put into
/// your graveyard this way. Those tokens have "Sacrifice this token: Add {C}."" (Glimpse
/// the Impossible): abilities the tokens are created with, read as part of the token's
/// description ("create ... tokens with "..."", CR 111.4).
fn those_tokens_have(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let Some(q) = end(l)
        .strip_prefix("those tokens have ")
        .or_else(|| end(l).strip_prefix("that token has "))
    else {
        return false;
    };
    if !q.starts_with('"') || !q.ends_with('"') {
        return false;
    }
    let Some(spec) = last_token_spec(prev) else {
        return false;
    };
    let mut sub = Builder::new(b.ctx);
    let probe = format!("create a 1/1 colorless ooze creature token with {q}");
    let Some(Effect::CreateToken { spec: probe_spec, .. }) =
        crate::oracle::effects::parse_clause(&probe, &mut sub)
    else {
        return false;
    };
    if probe_spec.abilities.is_empty() {
        return false;
    }
    spec.abilities.extend(probe_spec.abilities);
    true
}

inventory::submit! { super::FollowupPattern { name: "grants 5: those tokens have \"...\"", priority: 100, apply: those_tokens_have } }

std::thread_local! {
    static READING_OTHERS: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// "{2}{U/B}, Exile this card from your graveyard: Put a +1/+1 counter on target creature
/// you control. It gains this card's other abilities." (Symbiote Spider-Man): the card's
/// printed abilities other than this one, which the creature has from then on (CR 613.1f,
/// 113.10). In an ability an object gains, "this creature" means the object that has it
/// (CR 113.7a's referent: the object the ability is on).
fn gains_this_cards_other_abilities(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let who = l
        .strip_suffix(" gains ~'s other abilities")
        .or_else(|| l.strip_suffix(" gains this card's other abilities"))?;
    if READING_OTHERS.with(|r| r.get()) {
        return None;
    }
    let what = match who {
        "it" => super::pronoun_groups::singular_it(b),
        _ => crate::oracle::effects::object_ref(who, b).map(|(s, _)| s)?,
    };
    if super::oracle_hardening_referents::is_no_referent(&what) || matches!(what, Sel::This) {
        return None;
    }
    let raw = crate::oracle::raw_text();
    let norm = crate::oracle::normalize(&raw, b.ctx);
    let blocks: Vec<String> = crate::oracle::split_abilities(&norm)
        .into_iter()
        .filter(|blk| !blk.to_lowercase().contains("other abilities"))
        .collect();
    if blocks.is_empty() {
        return None;
    }
    READING_OTHERS.with(|r| r.set(true));
    let parsed: Option<Vec<Ability>> = blocks
        .iter()
        .map(|blk| crate::oracle::parse_ability(blk, b.ctx))
        .collect::<Option<Vec<Vec<Ability>>>>()
        .map(|v| v.into_iter().flatten().collect());
    READING_OTHERS.with(|r| r.set(false));
    let abilities = parsed?;
    if abilities
        .iter()
        .any(|a| matches!(a.kind, AbilityKind::Unsupported(_)))
    {
        return None;
    }
    Some(Effect::Modify {
        what,
        mods: abilities.into_iter().map(Modification::AddAbility).collect(),
        duration: Duration::Permanent,
    })
}

inventory::submit! { EffectPattern { name: "grants 5: it gains ~'s other abilities", priority: 100, parse: gains_this_cards_other_abilities } }

/// "Until end of turn, permanents your opponents control gain "When this permanent deals
/// damage to the player who cast Hellish Rebuke, sacrifice this permanent. You lose 2
/// life."" (Hellish Rebuke): inside the granted ability, the player who cast the granting
/// spell, fixed as the spell grants it (`granted_captures.rs`; each grant refers to its
/// own caster, Hellish Rebuke rulings).
fn deals_damage_to_the_caster(t: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    let t = end(t);
    let r = t.strip_suffix(&format!(" deals damage to {CASTER_PHRASE}"))?;
    // The object that has the ability: "~", or "enchanted permanent" as the granting
    // card's own point of view reads it when the quote also names that card
    // (`statics::mixed_granted_abilities`, `granted_by`).
    let source = match r {
        "~" => Filter::Source,
        "enchanted permanent" | "enchanted creature" => Filter::AttachedToSource,
        _ => return None,
    };
    let raw = crate::oracle::raw_text().to_lowercase();
    // Only inside an ability a spell grants ("gain \"when this permanent deals damage to
    // the player who cast ...").
    if !raw.contains(" gain \"when ") || !raw.contains(" to the player who cast ") {
        return None;
    }
    Some((
        TriggerCond::Where {
            trigger: Box::new(TriggerCond::DealsDamage {
                source,
                to: DamageRecipient::Player(PlayerRel::Any),
                combat_only: false,
            }),
            cond: Condition::PlayerMatches(
                PlayerRef::TriggerPlayer,
                PlayerFilter::Ref(Box::new(PlayerRef::Var(
                    crate::granted_captures::CAPTURED_CASTER,
                ))),
            ),
        },
        Sel::This,
        PlayerRef::TriggerPlayer,
    ))
}

inventory::submit! { super::TriggerPattern { name: "grants 5: ~ deals damage to the player who cast ~ (in a granted ability)", priority: 100, parse: deals_damage_to_the_caster } }

/// Whether `l` (a sentence, lowercase) is part of a quoted ability of the card's text.
fn inside_quotes(l: &str) -> bool {
    let raw = crate::oracle::raw_text().to_lowercase();
    raw.split('"')
        .enumerate()
        .any(|(i, part)| i % 2 == 1 && part.contains(l))
}

/// "the exiled card" in an ability granted to a token by the effect that exiled it ("It
/// gains "When this token deals combat damage to a player, sacrifice it and return the
/// exiled card to the battlefield tapped."", Timothar, Baron of Bats; "... it has "When
/// this token leaves the battlefield, return the exiled card to its owner's graveyard.""
/// Hofri Ghostforge): the card that effect exiled (CR 607.2a), fixed as it grants the
/// ability (`granted_captures.rs`), while it remains in exile (CR 400.7).
fn the_exiled_card_in_granted(l: &str, b: &mut Builder) -> Option<Effect> {
    const P: &str = "the exiled card";
    let l = end(l);
    if !l.contains(P) || l.contains('"') || b.named.iter().any(|(n, _)| n == P) || !inside_quotes(l) {
        return None;
    }
    b.named.push((
        P.into(),
        Sel::All(Filter::and(vec![
            Filter::In(Box::new(Sel::Var(crate::granted_captures::CAPTURED_EXILED))),
            Filter::InZone(ZoneKind::Exile),
        ])),
    ));
    let e = parse_sentence(l, b);
    b.named.retain(|(n, _)| n != P);
    e
}

inventory::submit! { EffectPattern { name: "grants 5: the exiled card (in an ability the exiling effect grants)", priority: 100, parse: the_exiled_card_in_granted } }

/// The objects the last exile instruction of `e` exiles (as an instruction or as a cost
/// it pays).
fn last_exiled(e: &Effect) -> Option<Sel> {
    let json = serde_json::to_value(e).ok()?;
    fn walk(v: &serde_json::Value, out: &mut Option<serde_json::Value>) {
        match v {
            serde_json::Value::Object(m) => {
                if let Some(serde_json::Value::Object(x)) = m.get("Exile") {
                    if let Some(w) = x.get("what") {
                        *out = Some(w.clone());
                    }
                }
                m.values().for_each(|x| walk(x, out));
            }
            serde_json::Value::Array(a) => a.iter().for_each(|x| walk(x, out)),
            _ => {}
        }
    }
    let mut out = None;
    walk(&json, &mut out);
    serde_json::from_value(out?).ok()
}

std::thread_local! {
    static CAPTURING: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// The sentence that grants an ability referring to "the exiled card": the card the
/// previous instructions exiled is stored for the grant (`granted_captures.rs`).
fn grants_ability_about_the_exiled_card(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    let Some(q) = l.find('"') else {
        return false;
    };
    if !l[q..].contains("the exiled card") || CAPTURING.with(|c| c.get()) {
        return false;
    }
    let Some(exiled) = last_exiled(prev) else {
        return false;
    };
    CAPTURING.with(|c| c.set(true));
    let e = parse_sentence(l, b);
    CAPTURING.with(|c| c.set(false));
    let Some(e) = e else {
        return false;
    };
    if !serde_json::to_string(&e)
        .unwrap_or_default()
        .contains(&format!("{{\"Var\":{}}}", crate::granted_captures::CAPTURED_EXILED))
    {
        return false;
    }
    let store = Effect::Store {
        var: crate::granted_captures::CAPTURED_EXILED,
        sel: exiled,
    };
    let e = match e {
        Effect::If {
            cond,
            then,
            otherwise,
        } => Effect::If {
            cond,
            then: Box::new(Effect::seq(vec![store, *then])),
            otherwise,
        },
        e => Effect::seq(vec![store, e]),
    };
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![old, e]);
    true
}

inventory::submit! { super::FollowupPattern { name: "grants 5: [grant an ability about the exiled card]", priority: 100, apply: grants_ability_about_the_exiled_card } }

/// The stand-in [`granting_spell_caster`] writes for "the player who cast [this card]"
/// inside a quoted ability the card grants.
const CASTER_PHRASE: &str = "the player who cast the granting spell";

std::thread_local! {
    static NAMING_CASTER: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// A spell whose granted ability says "the player who cast [this spell's name]" (Hellish
/// Rebuke): the name there is the spell's caster, not a reference to the object the ability
/// is on or to the card as an object, so the quote is read with a stand-in for that phrase
/// (see [`deals_damage_to_the_caster`]).
fn granting_spell_caster(block: &str, ctx: &crate::oracle::CompileContext) -> Option<Vec<Ability>> {
    if !ctx.is_spell() || NAMING_CASTER.with(|n| n.get()) {
        return None;
    }
    let name = crate::oracle::card_name();
    if name.is_empty() {
        return None;
    }
    // (The card's name is "~" in the normalized block.)
    let phrase = if block.contains(&format!("the player who cast {name}")) {
        format!("the player who cast {name}")
    } else {
        "the player who cast ~".to_string()
    };
    let q = block.find('"')?;
    if !block[q..].contains(&phrase) {
        return None;
    }
    let text = block.replace(&phrase, CASTER_PHRASE);
    let raw = crate::oracle::raw_text().replace(&format!("the player who cast {name}"), CASTER_PHRASE);
    NAMING_CASTER.with(|n| n.set(true));
    let out = crate::oracle::with_raw_text(&raw, || crate::oracle::parse_ability(&text, ctx));
    NAMING_CASTER.with(|n| n.set(false));
    Some(
        out?.into_iter()
            .map(|a| AbilityDef::with_link(a.kind.clone(), block, a.link))
            .collect(),
    )
}

inventory::submit! { super::AbilityPattern { name: "grants 5: the player who cast [this spell] in a granted ability", priority: 100, parse: granting_spell_caster } }
