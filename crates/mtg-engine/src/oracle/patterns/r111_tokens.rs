//! Token creation text (CR 111): tokens created by another player ("its controller
//! creates ...", CR 111.2), legendary named tokens ("create Boo, a legendary ...",
//! CR 111.9), token copies ("create a token that's a copy of target creature", CR 111.4,
//! 707), tokens by card name ("create a Tarmogoyf token", CR 111.11), and Role tokens
//! ("create a Monster Role token attached to it", CR 111.10j-r).

use crate::ability::*;
use crate::oracle::effects::{parse_token_description, Builder};
use crate::oracle::patterns::EffectPattern;
use crate::oracle::phrases::*;
use crate::types::*;
use smol_str::SmolStr;

/// "[count] [tapped] <token description>" → (spec, count, tapped).
fn token_phrase(r: &str) -> Option<(TokenSpec, Value, bool)> {
    let (count, r) = parse_number(r)?;
    let r = r.trim();
    let (r, tapped) = match r.strip_prefix("tapped ") {
        Some(x) => (x, true),
        None => (r, false),
    };
    let (w, rest) = split_word(r);
    if let Some(spec) = crate::tokens::predefined(w) {
        let rest = end(rest);
        if rest == "token" || rest == "tokens" {
            return Some((spec, count, tapped));
        }
    }
    Some((parse_token_description(r)?, count, tapped))
}

/// "its controller creates a 3/3 green Beast creature token", "target opponent creates
/// four 1/1 blue Faerie creature tokens with flying", "each player creates a Food token":
/// the player who creates a token is its owner and controller (CR 111.2).
fn another_player_creates(l: &str, b: &mut Builder) -> Option<Effect> {
    let (subject, rest) = l.split_once(" creates ")?;
    let who = match subject {
        "its controller" | "their controller" => PlayerRef::ControllerOf(Box::new(b.it.clone())),
        "its owner" => PlayerRef::OwnerOf(Box::new(b.it.clone())),
        "that player" => b.it_player.clone(),
        "each player" => PlayerRef::EachPlayer,
        "each opponent" => PlayerRef::EachOpponent,
        "target opponent" | "target player" => {
            let (filter, text) = if subject == "target opponent" {
                (PlayerFilter::Opponent, "target opponent")
            } else {
                (PlayerFilter::Any, "target player")
            };
            let it = b.it.clone();
            let slot = b.add_target(TargetSpec::player(filter, text), text);
            b.it = it;
            b.it_player = PlayerRef::Target(slot);
            PlayerRef::Target(slot)
        }
        _ => return None,
    };
    let (spec, count, tapped) = token_phrase(rest)?;
    Some(Effect::CreateToken {
        spec,
        count,
        controller: who,
        tapped,
        attacking: false,
    })
}

/// "Voja fenstalker" → "Voja Fenstalker" (the effect text is lowercased before parsing).
fn title_case(name: &str) -> String {
    name.split(' ')
        .enumerate()
        .map(|(i, w)| {
            if i > 0 && matches!(w, "of" | "the" | "and" | "a" | "an" | "in" | "to") {
                return w.to_string();
            }
            let mut c = w.chars();
            match c.next() {
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// "create Boo, a legendary 1/1 red Hamster creature token with trample and haste": a
/// token with the listed characteristics that has the given name (CR 111.9).
fn legendary_named_token(l: &str, _b: &mut Builder) -> Option<Effect> {
    let r = l.strip_prefix("create ")?;
    let (name, desc) = r.split_once(", a ")?;
    if name.is_empty() || name.contains('~') || name.split(' ').count() > 5 {
        return None;
    }
    let desc = desc.strip_prefix("legendary ")?;
    let mut spec = parse_token_description(desc)?;
    spec.name = SmolStr::new(title_case(name));
    if !spec.supertypes.contains(&Supertype::Legendary) {
        spec.supertypes.push(Supertype::Legendary);
    }
    Some(Effect::CreateToken {
        spec,
        count: Value::c(1),
        controller: PlayerRef::You,
        tapped: false,
        attacking: false,
    })
}

/// "create a token that's a copy of target creature", "create a token that's a copy of
/// it": the token's characteristics are the copiable values of the object, and its name
/// is that object's name (CR 111.4, 707.2).
fn token_copy(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = l.strip_prefix("create ")?;
    let (count, r) = parse_number(r)?;
    let r = r.trim_start();
    let (r, tapped) = match r.strip_prefix("tapped ") {
        Some(x) => (x, true),
        None => (r, false),
    };
    let r = r
        .strip_prefix("token that's a copy of ")
        .or_else(|| r.strip_prefix("tokens that are copies of "))?;
    let (of, rest) = if let Some(rest) = r.strip_prefix('~') {
        (Sel::This, rest)
    } else if let Some(rest) = ["it", "that creature", "that permanent", "that card"]
        .iter()
        .find_map(|p| r.strip_prefix(p))
    {
        (b.it.clone(), rest)
    } else {
        let (spec, rest) = parse_target(r)?;
        let text = r[..r.len() - rest.len()].trim().to_string();
        let slot = b.add_target(spec, &text);
        (Sel::Target(slot), rest)
    };
    if !end(rest).is_empty() {
        return None;
    }
    Some(Effect::CreateTokenCopy {
        of,
        count,
        controller: PlayerRef::You,
        tapped,
        attacking: false,
        mods: vec![],
    })
}

/// "create a Tarmogoyf token": a token by name that isn't a predefined token uses the
/// characteristics of the card with that name in the Oracle card reference (CR 111.11).
fn token_by_card_name(l: &str, _b: &mut Builder) -> Option<Effect> {
    let r = l.strip_prefix("create ")?;
    let (count, r) = parse_number(r)?;
    let Value::Const(n) = count else {
        return None;
    };
    let r = end(r);
    let name = r
        .strip_suffix(" tokens")
        .or_else(|| r.strip_suffix(" token"))?
        .trim();
    if name.is_empty() || crate::tokens::predefined(name).is_some() {
        return None;
    }
    let sc = mtg_data::cards().by_name(name)?;
    if sc.layout.contains("token") {
        return None;
    }
    Some(Effect::Custom(SmolStr::new(format!(
        "named-token:{n}:{}",
        sc.name
    ))))
}

/// "create a Monster Role token attached to it", "create a Cursed Role token attached to
/// target creature an opponent controls" (CR 111.10j-r, 303.7).
fn role_token(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = l.strip_prefix("create a ")?;
    let (role, rest) = r.split_once(" role token attached to ")?;
    let spec = crate::tokens::predefined(role)?;
    if !spec.subtypes.iter().any(|s| s.as_str() == "Role") {
        return None;
    }
    let (to, tail) = if let Some(rest) = ["it", "that creature", "that permanent"]
        .iter()
        .find_map(|p| rest.strip_prefix(p))
    {
        (b.it.clone(), rest)
    } else if let Some(rest) = rest.strip_prefix('~') {
        (Sel::This, rest)
    } else {
        let (spec, tail) = parse_target(rest)?;
        let text = rest[..rest.len() - tail.len()].trim().to_string();
        let slot = b.add_target(spec, &text);
        (Sel::Target(slot), tail)
    };
    if !end(tail).is_empty() {
        return None;
    }
    Some(Effect::seq(vec![
        Effect::CreateToken {
            spec,
            count: Value::c(1),
            controller: PlayerRef::You,
            tapped: false,
            attacking: false,
        },
        Effect::Attach {
            what: Sel::Var(vars::CREATED),
            to,
        },
    ]))
}

/// "create two Food tokens named Hot Dog": the effect that creates a predefined token may
/// modify its predefined characteristics (CR 111.10), here its name.
fn named_predefined_tokens(l: &str, _b: &mut Builder) -> Option<Effect> {
    let r = l.strip_prefix("create ")?;
    let (count, r) = parse_number(r)?;
    let r = r.trim();
    let (r, tapped) = match r.strip_prefix("tapped ") {
        Some(x) => (x, true),
        None => (r, false),
    };
    let (kind, rest) = split_word(r);
    let mut spec = crate::tokens::predefined(kind)?;
    let name = rest
        .trim()
        .strip_prefix("tokens named ")
        .or_else(|| rest.trim().strip_prefix("token named "))?;
    let name = end(name);
    if name.is_empty() || name.contains(' ') && name.split(' ').count() > 4 {
        return None;
    }
    spec.name = SmolStr::new(title_case(name));
    Some(Effect::CreateToken {
        spec,
        count,
        controller: PlayerRef::You,
        tapped,
        attacking: false,
    })
}

inventory::submit! { EffectPattern { name: "r111 named predefined tokens", priority: 100, parse: named_predefined_tokens } }
inventory::submit! { EffectPattern { name: "r111 another player creates tokens", priority: 100, parse: another_player_creates } }
inventory::submit! { EffectPattern { name: "r111 legendary named token", priority: 100, parse: legendary_named_token } }
inventory::submit! { EffectPattern { name: "r111 token copy", priority: 100, parse: token_copy } }
inventory::submit! { EffectPattern { name: "r111 token by card name", priority: 110, parse: token_by_card_name } }
inventory::submit! { EffectPattern { name: "r111 role token attached", priority: 100, parse: role_token } }
