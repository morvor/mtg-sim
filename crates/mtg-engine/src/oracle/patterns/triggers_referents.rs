//! Effects that refer back to what a triggered ability triggered on (CR 603.2, 608.2h):
//! "that creature's controller", "attach it to target creature you control" (the source,
//! for a self trigger), "you may attach ~ to it" (the triggering creature), "shuffle it
//! into its owner's library", "sacrifice ~ unless you pay {1}", "that player draws an
//! additional card", and single-sentence trigger bodies whose referents need care:
//! "destroy that creature at end of combat" (a delayed trigger that must remember the
//! creature, CR 603.7c) and "put that many +1/+1 counters on it" ("that many" is the
//! trigger event's amount).

use crate::ability::*;
use crate::oracle::effects::{parse_clause, parse_trigger_body, split_sentences, Builder};
use crate::oracle::patterns::{AbilityPattern, EffectPattern};
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;

inventory::submit! {
    EffectPattern { name: "that creature's → its", priority: 150, parse: that_creatures }
}
inventory::submit! {
    EffectPattern { name: "attach it/~ (trigger referent)", priority: 100, parse: attach_referent }
}
inventory::submit! {
    EffectPattern { name: "shuffle/put it into its owner's library", priority: 100, parse: to_owners_library }
}
inventory::submit! {
    EffectPattern { name: "sacrifice ~ unless you pay", priority: 100, parse: sacrifice_unless }
}
inventory::submit! {
    EffectPattern { name: "draws an additional card", priority: 100, parse: additional_card }
}
inventory::submit! {
    EffectPattern { name: "(you may) have it deal ...", priority: 100, parse: have_it_deal }
}

/// "[you may] have it deal 2 damage to any target", "have ~ deal damage equal to its
/// power to target creature": the object named deals the damage.
fn have_it_deal(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("have ")?;
    let (subject, rest) = ["it ", "~ ", "that creature "]
        .into_iter()
        .find_map(|s| r.strip_prefix(s).map(|x| (s.trim_end(), x)))?;
    let rest = rest.strip_prefix("deal ")?;
    if subject != "~" && matches!(b.it, Sel::None) {
        return None;
    }
    parse_clause(&format!("{subject} deals {rest}"), b)
}
inventory::submit! {
    AbilityPattern { name: "trigger with delayed or 'that many' body", priority: 100, parse: trigger_with_event_body }
}

/// "that creature's controller", "that creature's power", "that permanent's owner":
/// possessives of the trigger's object read like "its". Oracle text calls the source
/// itself "this creature"/"~", never "that creature", so when "it" still means the
/// source the antecedent wasn't tracked (a discarded card, a sacrificed permanent, the
/// spell that targeted the source) and the phrase isn't rewritten.
fn that_creatures(l: &str, b: &mut Builder) -> Option<Effect> {
    if matches!(b.it, Sel::None | Sel::This) {
        return None;
    }
    let mut s = end(l).to_string();
    let before = s.clone();
    for p in [
        "that creature's ",
        "that permanent's ",
        "that card's ",
        "that spell's ",
        "that land's ",
        "that artifact's ",
        "that enchantment's ",
        "that token's ",
    ] {
        s = s.replace(p, "its ");
    }
    if s == before {
        return None;
    }
    parse_clause(&s, b)
}

/// "attach it to target creature you control" (it = the source), "attach ~ to target
/// creature you control", "attach ~ to it" (it = the triggering creature).
fn attach_referent(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let r = l.strip_prefix("attach ")?;
    let (what, rest) = if let Some(x) = r.strip_prefix("~ to ") {
        (Sel::This, x)
    } else if let Some(x) = r.strip_prefix("it to ") {
        if !matches!(b.it, Sel::This) {
            return None;
        }
        (Sel::This, x)
    } else {
        return None;
    };
    let to = if rest == "it" || rest == "that creature" {
        // Only a creature still on the battlefield (not one that died and may have been
        // returned by an earlier instruction).
        if !b.in_trigger || !matches!(b.it, Sel::TriggerObject | Sel::TriggerOtherObject) {
            return None;
        }
        b.it.clone()
    } else {
        let (spec, tail) = parse_target(rest)?;
        if !end(tail).is_empty() || !matches!(spec.what, TargetKind::Object(_)) {
            return None;
        }
        let slot = b.add_target(spec, rest);
        // "That creature gains first strike": the creature it was attached to.
        Sel::Target(slot)
    };
    Some(Effect::Attach { what, to })
}

/// "shuffle it into its owner's library", "put it on the bottom of its owner's library",
/// "put ~ on top of its owner's library".
fn to_owners_library(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let pronoun = |w: &str, b: &Builder| -> Option<Sel> {
        match w {
            "~" => Some(Sel::This),
            "it" | "that card" | "that creature" if !matches!(b.it, Sel::None) => {
                Some(b.it.clone())
            }
            _ => None,
        }
    };
    if let Some(r) = l.strip_prefix("shuffle ") {
        let w = r.strip_suffix(" into its owner's library")?;
        return Some(Effect::ShuffleInto {
            what: pronoun(w, b)?,
        });
    }
    let r = l.strip_prefix("put ")?;
    // "put it onto the battlefield [tapped] [under your control]": the card the trigger is
    // about (e.g. "when ~ is put into your graveyard from your library").
    for (suffix, tapped) in [
        (" onto the battlefield under your control", false),
        (" onto the battlefield", false),
        (" onto the battlefield tapped under your control", true),
        (" onto the battlefield tapped", true),
    ] {
        if let Some(w) = r.strip_suffix(suffix) {
            let what = pronoun(w, b)?;
            let mut to = Destination::battlefield().under_your_control();
            if tapped {
                to = to.tapped();
            }
            return Some(Effect::Move { what, to });
        }
    }
    for (suffix, to) in [
        (
            " on the bottom of its owner's library",
            Destination::library_bottom(),
        ),
        (" on top of its owner's library", Destination::library_top()),
    ] {
        if let Some(w) = r.strip_suffix(suffix) {
            return Some(Effect::Move {
                what: pronoun(w, b)?,
                to,
            });
        }
    }
    None
}

/// "sacrifice ~ unless you pay {u}", "sacrifice it unless you discard a card".
fn sacrifice_unless(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let r = l.strip_prefix("sacrifice ~ unless you ").or_else(|| {
        if matches!(b.it, Sel::This) {
            l.strip_prefix("sacrifice it unless you ")
        } else {
            None
        }
    })?;
    let cost = if let Some(m) = r.strip_prefix("pay ") {
        if m.starts_with('{') {
            crate::oracle::keywords::parse_keyword_cost(m)?
        } else {
            crate::oracle::costs::parse_cost(r)?.0
        }
    } else {
        crate::oracle::costs::parse_cost(r)?.0
    };
    // CR 118.12a: "[Do something] unless [a player pays a cost]".
    Some(Effect::PayOptional {
        who: PlayerRef::You,
        cost,
        then: Box::new(Effect::Noop),
        otherwise: Box::new(Effect::SacrificeObjects { what: Sel::This }),
    })
}

/// "that player draws an additional card", "draw an additional card", "you draw two
/// additional cards".
fn additional_card(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (who, r) = if let Some(r) = l.strip_prefix("that player draws ") {
        (b.it_player.clone(), r)
    } else if let Some(r) = l
        .strip_prefix("you draw ")
        .or_else(|| l.strip_prefix("draw "))
    {
        (PlayerRef::You, r)
    } else {
        return None;
    };
    let (n, r) = parse_number(r)?;
    if !matches!(end(r), "additional card" | "additional cards") {
        return None;
    }
    Some(Effect::Draw { who, n })
}

// ---------------------------------------------------------------------------
// Whole triggered abilities with a single-sentence body.
// ---------------------------------------------------------------------------

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

/// "When [trigger], [effect] at end of combat." / "... at the beginning of the next end
/// step." / "... that many ...": the body is one sentence, so its pronouns and "that
/// many" refer to the trigger (not to something an earlier sentence created).
fn trigger_with_event_body(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let text = crate::oracle::strip_ability_word(block.trim());
    let lower = text.to_lowercase();
    if !(lower.starts_with("when ") || lower.starts_with("whenever ") || lower.starts_with("at ")) {
        return None;
    }
    // Triggered abilities of instants and sorceries need zone handling (see
    // `oracle::triggers::parse_triggered`); these forms are for permanents.
    if ctx.is_spell() {
        return None;
    }
    let (cond_s, eff_s) = split_trigger(text)?;
    let eff = eff_s.trim();
    if split_sentences(eff).len() != 1 {
        return None;
    }
    let el = end(&eff.to_lowercase()).to_string();
    if el.starts_with("if ") || el.contains("triggers only once") {
        return None;
    }
    let (trigger, it, it_player) =
        crate::oracle::triggers::parse_trigger_condition(&cond_s.to_lowercase())?;
    let words: Vec<&str> = el
        .split(|c: char| !c.is_alphanumeric() && c != '\'')
        .collect();
    let has = |w: &str| words.contains(&w);
    if matches!(it, Sel::None)
        && (has("it") || has("its") || has("them") || el.contains("that creature"))
    {
        return None;
    }
    if matches!(it_player, PlayerRef::Iterated) && el.contains("that player") {
        return None;
    }
    let body = if let Some((delay, inner)) = super::triggers_delayed::split_delay(&el) {
        let mut body = parse_trigger_body(inner, ctx, it, it_player)?;
        if body.modal.is_some() {
            return None;
        }
        let (mut seq, effect) = super::triggers_delayed::capture(&body.effect)?;
        seq.push(Effect::DelayedTrigger {
            trigger: delay,
            body: Box::new(Body::effect(effect)),
            once: true,
        });
        body.effect = Effect::seq(seq);
        body
    } else if let Some(i) = el.find("that many") {
        if !has_event_amount(&trigger) {
            return None;
        }
        // Only when "that many" is in the first clause: "discard up to two cards, then
        // draw that many cards" refers to the discard instead.
        let head = &el[..i];
        if head.contains(',') || head.contains(" and ") || head.contains(" then ") {
            return None;
        }
        if el.split(|c: char| !c.is_alphanumeric()).any(|w| w == "x") {
            return None;
        }
        let with_x = el.replacen("that many", "x", 1);
        let mut body = parse_trigger_body(&with_x, ctx, it, it_player)?;
        if body.modal.is_some() || !body.targets.is_empty() {
            return None;
        }
        body.effect = replace_x_with_event_amount(&body.effect)?;
        body
    } else {
        return None;
    };
    let mut tr = TriggeredAbility::new(trigger, body);
    if let TriggerCond::CastSpell {
        filter: Filter::Source,
        ..
    } = &tr.trigger
    {
        tr.zone = FunctionZone::Stack;
    }
    Some(vec![AbilityDef::new(AbilityKind::Triggered(tr), text)])
}

/// Whether the trigger event carries a meaningful amount for "that many"/"that much":
/// damage, life gained or lost, counters put, cards milled, attackers, or the number of
/// objects in a batch.
fn has_event_amount(t: &TriggerCond) -> bool {
    match t {
        TriggerCond::DealsDamage { .. }
        | TriggerCond::IsDealtDamage { .. }
        | TriggerCond::PlayerDealtDamage { .. }
        | TriggerCond::GainsLife { .. }
        | TriggerCond::LosesLife { .. }
        | TriggerCond::CountersPut { .. }
        | TriggerCond::CountersRemoved { .. }
        | TriggerCond::Mills(_)
        | TriggerCond::PlayerAttacks(_)
        | TriggerCond::Batched { .. } => true,
        TriggerCond::Where { trigger, .. } | TriggerCond::FirstTimeEachTurn(trigger) => {
            has_event_amount(trigger)
        }
        _ => false,
    }
}

fn replace_x_with_event_amount(e: &Effect) -> Option<Effect> {
    use serde_json::Value as J;
    fn walk(v: J) -> J {
        match v {
            J::String(s) if s == "X" => J::String("EventAmount".into()),
            J::Object(m) => J::Object(m.into_iter().map(|(k, v)| (k, walk(v))).collect()),
            J::Array(a) => J::Array(a.into_iter().map(walk).collect()),
            other => other,
        }
    }
    let json = serde_json::to_value(e).ok()?;
    serde_json::from_value(walk(json)).ok()
}
