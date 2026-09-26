//! Oracle patterns for the keyword actions of CR 701.29–701.71 (implemented in `kwa/`):
//! explore, connive, endure, adapt, monstrosity, bolster, support, populate, amass,
//! incubate, recruit, empower Jace, learn, discover, fateseal, clash, time travel, blight,
//! harness, suspect, and detain.

use super::{AbilityPattern, CostPattern, EffectPattern};
use crate::ability::*;
use crate::kwa::{kvars, Spec};
use crate::oracle::effects::{object_ref, player_ref, Builder};
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;
use smol_str::SmolStr;

/// An `Effect::KeywordAction`.
pub fn keyword_action(action: KeywordAction, who: PlayerRef, what: Sel, n: Value) -> Effect {
    Effect::KeywordAction {
        action,
        who,
        what,
        n,
    }
}

/// "he"/"she" as the subject of a sentence mean "it" (a legendary creature's pronoun).
fn pronoun_subject(l: &str) -> String {
    for p in ["he ", "she ", "they "] {
        if let Some(r) = l.strip_prefix(p) {
            return format!("it {r}");
        }
    }
    l.to_string()
}

/// "[subject] [verb](s)[ rest]": the subject (an object reference) and what follows the
/// verb.
fn object_verb(l: &str, verb: &str, b: &mut Builder) -> Option<(Sel, String)> {
    let l = pronoun_subject(end(l));
    let (sel, rest) = object_ref(&l, b)?;
    let rest = rest.trim_start();
    let after = rest
        .strip_prefix(&format!("{verb}s"))
        .or_else(|| rest.strip_prefix(verb))?;
    if !(after.is_empty() || after.starts_with(' ') || after.starts_with(',')) {
        return None;
    }
    Some((sel, after.trim().to_string()))
}

/// "[player] [verb](s)[ rest]" with an optional player subject ("you recruit", "each
/// opponent blights 1", "blight 1").
fn player_verb(l: &str, verb: &str, b: &mut Builder) -> Option<(PlayerRef, String)> {
    let l = end(l);
    if let Some(r) = l.strip_prefix(verb) {
        if r.is_empty() || r.starts_with(' ') {
            return Some((PlayerRef::You, r.trim().to_string()));
        }
    }
    let (who, rest) = player_ref(l, b)?;
    let rest = rest.trim_start();
    let after = rest
        .strip_prefix(&format!("{verb}s"))
        .or_else(|| rest.strip_prefix(&format!("{verb}es")))
        .or_else(|| rest.strip_prefix(verb))?;
    if !(after.is_empty() || after.starts_with(' ')) {
        return None;
    }
    Some((who, after.trim().to_string()))
}

/// A number and nothing else.
fn just_number(s: &str) -> Option<Value> {
    let (n, rest) = parse_number(s)?;
    end(rest).is_empty().then_some(n)
}

/// "N times", "twice", "three times", or nothing (once).
fn times(s: &str) -> Option<Value> {
    let s = end(s);
    if s.is_empty() || s == "again" {
        return Some(Value::c(1));
    }
    if s == "twice" {
        return Some(Value::c(2));
    }
    let (n, rest) = parse_number(s)?;
    (end(rest) == "times").then_some(n)
}

/// "[permanent] explores[ N times][, then it explores again]" (CR 701.44).
fn explore(l: &str, b: &mut Builder) -> Option<Effect> {
    let (what, rest) = object_verb(l, "explore", b)?;
    let n = match rest.as_str() {
        ", then it explores again" => Value::c(2),
        r => times(r)?,
    };
    Some(keyword_action(KeywordAction::Explore, PlayerRef::You, what, n))
}

/// "[permanent] connives[ N]", "you may have [it] connive" (CR 701.50).
fn connive(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    if let Some(r) = l.strip_prefix("have ") {
        let (what, rest) = object_ref(r, b)?;
        let n = match end(&rest) {
            "connive" => Value::c(1),
            r => just_number(r.strip_prefix("connive ")?)?,
        };
        return Some(keyword_action(KeywordAction::Connive, PlayerRef::You, what, n));
    }
    let (what, rest) = object_verb(l, "connive", b)?;
    let n = if rest.is_empty() {
        Value::c(1)
    } else {
        just_number(&rest)?
    };
    Some(keyword_action(KeywordAction::Connive, PlayerRef::You, what, n))
}

/// "[permanent] endures N" (CR 701.63).
fn endure(l: &str, b: &mut Builder) -> Option<Effect> {
    let (what, rest) = object_verb(l, "endure", b)?;
    let n = just_number(&rest)?;
    Some(keyword_action(KeywordAction::Endure, PlayerRef::You, what, n))
}

/// "[player] [verb] N" / "[verb]" for actions performed by a player with a number (or
/// none): bolster, amass, incubate, empower Jace, recruit, learn, discover, fateseal,
/// populate, time travel, blight, clash.
fn player_actions(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let numbered: [(&str, KeywordAction); 6] = [
        ("bolster", KeywordAction::Bolster),
        ("incubate", KeywordAction::Incubate),
        ("empower jace", KeywordAction::EmpowerJace),
        ("discover", KeywordAction::Discover),
        ("fateseal", KeywordAction::Fateseal),
        ("blight", KeywordAction::Blight),
    ];
    for (verb, action) in numbered {
        if let Some((who, rest)) = player_verb(l, verb, b) {
            let n = just_number(&rest)?;
            return Some(keyword_action(action, who, Sel::None, n));
        }
    }
    let repeated: [(&str, KeywordAction); 4] = [
        ("populate", KeywordAction::Populate),
        ("time travel", KeywordAction::TimeTravel),
        ("recruit", KeywordAction::Recruit),
        ("learn", KeywordAction::Learn),
    ];
    for (verb, action) in repeated {
        if let Some((who, rest)) = player_verb(l, verb, b) {
            let n = match rest.as_str() {
                ", then time travel" if action == KeywordAction::TimeTravel => Value::c(2),
                r => times(r)?,
            };
            return Some(keyword_action(action, who, Sel::None, n));
        }
    }
    if let Some((who, rest)) = player_verb(l, "clash", b) {
        if rest == "with an opponent" {
            return Some(keyword_action(
                KeywordAction::Clash,
                who,
                Sel::None,
                Value::c(1),
            ));
        }
    }
    None
}

/// "adapt N" and "monstrosity N" (the permanent itself, CR 701.46, 701.37).
fn self_counters(l: &str, _b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    for (verb, action) in [
        ("adapt ", KeywordAction::Adapt),
        ("monstrosity ", KeywordAction::Monstrosity),
    ] {
        if let Some(r) = l.strip_prefix(verb) {
            let n = just_number(r)?;
            return Some(keyword_action(action, PlayerRef::You, Sel::This, n));
        }
    }
    None
}

/// "amass [subtype] N" (CR 701.47). Older cards' "amass N" reads "amass Zombies N" in the
/// Oracle text (CR 701.47d).
fn amass(l: &str, b: &mut Builder) -> Option<Effect> {
    let (who, rest) = player_verb(l, "amass", b)?;
    let (w, r) = split_word(&rest);
    let subtype = subtype_word(w)?;
    let n = just_number(r)?;
    let mut spec = Spec::new(KeywordAction::Amass, who, Sel::None, n);
    spec.subtype = Some(subtype);
    Some(spec.effect())
}

/// "amass [subtype] N, then [effect with 'the Army you amassed' / 'the amassed Army']"
/// (CR 701.47c).
fn amass_then(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (first, rest) = l.split_once(", then ")?;
    let a = amass(first, b)?;
    let rest = replace_amassed(rest)?;
    let saved = b.it.clone();
    b.it = Sel::Var(kvars::AMASSED);
    let then = crate::oracle::effects::parse_clause(&rest, b);
    b.it = saved;
    Some(Effect::seq(vec![a, then?]))
}

/// Replaces "the Army you amassed" / "the amassed Army" by pronouns (the builder's "it"
/// is then the Army). None if the text doesn't mention it.
fn replace_amassed(s: &str) -> Option<String> {
    let mut out = s.to_string();
    for (from, to) in [
        ("the army you amassed's", "its"),
        ("the amassed army's", "its"),
        ("the army you amassed", "it"),
        ("the amassed army", "it"),
    ] {
        out = out.replace(from, to);
    }
    (out != s).then_some(out)
}

/// "harness ~" (CR 701.64).
fn harness(l: &str, _b: &mut Builder) -> Option<Effect> {
    (end(l) == "harness ~").then(|| {
        keyword_action(
            KeywordAction::Harness,
            PlayerRef::You,
            Sel::This,
            Value::c(1),
        )
    })
}

/// "suspect [objects]" and "detain [objects]" (CR 701.60, 701.35).
fn suspect_detain(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (action, r) = if let Some(r) = l.strip_prefix("suspect ") {
        (KeywordAction::Suspect, r)
    } else if let Some(r) = l.strip_prefix("detain ") {
        (KeywordAction::Detain, r)
    } else {
        return None;
    };
    let (what, rest) = object_ref(r, b)?;
    if !end(&rest).is_empty() {
        return None;
    }
    Some(keyword_action(action, PlayerRef::You, what, Value::c(1)))
}

/// "forage" and "collect evidence N" as instructions (CR 701.61, 701.59).
fn forage_evidence(l: &str, _b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    if l == "forage" {
        return Some(Effect::KeywordAction {
            action: KeywordAction::Forage,
            who: PlayerRef::You,
            what: Sel::None,
            n: Value::c(1),
        });
    }
    let n = just_number(l.strip_prefix("collect evidence ")?)?;
    Some(keyword_action(
        KeywordAction::CollectEvidence,
        PlayerRef::You,
        Sel::None,
        n,
    ))
}

inventory::submit! { EffectPattern { name: "a701 explore", priority: 60, parse: explore } }
inventory::submit! { EffectPattern { name: "a701 connive", priority: 60, parse: connive } }
inventory::submit! { EffectPattern { name: "a701 endure", priority: 60, parse: endure } }
inventory::submit! { EffectPattern { name: "a701 player actions", priority: 60, parse: player_actions } }
inventory::submit! { EffectPattern { name: "a701 adapt / monstrosity", priority: 60, parse: self_counters } }
inventory::submit! { EffectPattern { name: "a701 amass", priority: 60, parse: amass } }
inventory::submit! { EffectPattern { name: "a701 amass then", priority: 59, parse: amass_then } }
inventory::submit! { EffectPattern { name: "a701 harness", priority: 60, parse: harness } }
inventory::submit! { EffectPattern { name: "a701 suspect / detain", priority: 60, parse: suspect_detain } }
inventory::submit! { EffectPattern { name: "a701 forage / collect evidence", priority: 60, parse: forage_evidence } }

/// "Support N." (CR 701.41a): on a permanent, "When this enters, put a +1/+1 counter on
/// each of up to N other target creatures"; on an instant or sorcery, "Put a +1/+1 counter
/// on each of up to N target creatures".
fn support(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let t = block.trim();
    let lower = t.to_lowercase();
    let n = just_number(end(lower.strip_prefix("support ")?))?;
    let spell = ctx.is_spell();
    let mut what = Filter::creature();
    if !spell {
        what = Filter::and(vec![what, Filter::Other]);
    }
    let spec = TargetSpec {
        what: TargetKind::Object(what),
        min: 0,
        max: n,
        distinct_from: vec![],
        divide: None,
        chosen_by_opponent: false,
        text: if spell {
            "up to N target creatures".into()
        } else {
            "up to N other target creatures".into()
        },
        condition: None,
    };
    let body = Body {
        targets: vec![spec],
        effect: Effect::AddCounters {
            what: Sel::Target(0),
            kind: SmolStr::new(crate::types::counters::PLUS1),
            n: Value::c(1),
        },
        modal: None,
    };
    let kind = if spell {
        AbilityKind::Spell(SpellAbility { body })
    } else {
        AbilityKind::Triggered(TriggeredAbility::new(
            TriggerCond::EntersBattlefield(Filter::Source),
            body,
        ))
    };
    Some(vec![AbilityDef::new(kind, t)])
}

inventory::submit! { AbilityPattern { name: "a701 support", priority: 60, parse: support } }

/// "blight N" as a cost (CR 701.68): putting N -1/-1 counters on a creature you control.
fn blight_cost(p: &str) -> Option<CostPart> {
    let n = just_number(end(p).strip_prefix("blight ")?)?;
    Some(CostPart::Effect(Box::new(keyword_action(
        KeywordAction::Blight,
        PlayerRef::You,
        Sel::None,
        n,
    ))))
}

inventory::submit! { CostPattern { name: "a701 blight cost", priority: 60, parse: blight_cost } }
