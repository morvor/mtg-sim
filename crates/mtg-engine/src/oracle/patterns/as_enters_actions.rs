//! As-enters actions that aren't choices or counters (CR 614.1c, 614.12): replacement
//! effects that do something as the permanent enters.
//!
//! * "As ~ enters, you get an emblem with "..."" (Tibalt, Cosmic Impostor): the emblem is
//!   created as it enters; "cards exiled with ~" on the emblem are those exiled with the
//!   permanent (see `Sel::Linked` for emblems in `eval.rs`).
//! * "As ~ enters, choose a creature you control it could be attached to. If you do, it
//!   enters attached to that creature." (Grifter's Blade; `kw/enters_attached.rs`).
//! * "As ~ enters, exile X creature cards from your graveyard. If you can't, put ~ into its
//!   owner's graveyard instead ..." (Frankenstein's Monster).

use super::AbilityPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::CompileContext;

fn etb_replacement(action: ReplacementAction, text: &str) -> Ability {
    AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Replacement(
            ReplacementDef {
                event: ReplacementEvent::EntersBattlefield(Filter::Source),
                action,
                self_replacement: false,
                optional: false,
            },
        ))),
        text,
    )
}

/// "As ~ enters, you get an emblem with "..."".
fn as_enters_emblem(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if !ctx.is_permanent() || block.contains('\n') {
        return None;
    }
    let text = crate::oracle::strip_ability_word(block.trim());
    let rest = text.strip_prefix("As ~ enters, ")?;
    if !rest.starts_with("you get an emblem with \"") {
        return None;
    }
    let mut b = Builder::new(ctx);
    b.it = Sel::This;
    let e = crate::oracle::effects::parse_effect_text(rest, &mut b)?;
    if !b.targets.is_empty() || !matches!(e, Effect::CreateEmblem { .. }) {
        return None;
    }
    Some(vec![etb_replacement(
        ReplacementAction::AsEnters(Box::new(e)),
        block,
    )])
}

inventory::submit! { AbilityPattern { name: "as ~ enters, you get an emblem", priority: 45, parse: as_enters_emblem } }

/// "As ~ enters, choose a creature you control it could be attached to. If you do, it
/// enters attached to that creature." (Grifter's Blade): an Equipment's host chosen as it
/// enters (CR 301.5b); without one it enters unattached.
fn as_enters_attached(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    use crate::kw::enters_attached::{ENTER_ATTACHED_TO_CHOSEN, ENTRY_HOST};
    use crate::oracle::phrases::{end, parse_object_phrase};
    if !ctx.is_permanent() || block.contains('\n') {
        return None;
    }
    let lower = crate::oracle::strip_ability_word(block.trim()).to_lowercase();
    let r = end(&lower).strip_prefix("as ~ enters, choose ")?;
    let (choice, then) = r.split_once(". ")?;
    let noun = then
        .strip_prefix("if you do, it enters attached to that ")
        .or_else(|| then.strip_prefix("if you do, ~ enters attached to that "))?;
    let r = choice
        .strip_prefix("a ")
        .or_else(|| choice.strip_prefix("an "))?;
    let (f, plural, tail) = parse_object_phrase(r)?;
    if plural || !r.starts_with(noun) {
        return None;
    }
    let can = match tail.trim() {
        "it could be attached to" | "it can be attached to" => {
            Filter::Custom(crate::attach::SOURCE_CAN_ATTACH.into())
        }
        _ => return None,
    };
    if f.zone().is_some_and(|z| z != ZoneKind::Battlefield) {
        return None;
    }
    let body = Effect::seq(vec![
        Effect::Store {
            var: ENTRY_HOST,
            sel: Sel::Choose {
                chooser: PlayerRef::You,
                filter: Filter::and(vec![f, can]),
                count: Value::c(1),
                up_to: false,
                store: None,
            },
        },
        Effect::Custom(ENTER_ATTACHED_TO_CHOSEN.into()),
    ]);
    Some(vec![etb_replacement(
        ReplacementAction::AsEnters(Box::new(body)),
        block,
    )])
}

inventory::submit! { AbilityPattern { name: "as ~ enters, choose [a host]. it enters attached to it", priority: 45, parse: as_enters_attached } }

/// The cards exiled as it entered (Frankenstein's Monster).
const EXILED_AS_ENTERED: Var = vars::USER + 52011;

/// "As ~ enters, exile X creature cards from your graveyard. If you can't, put ~ into its
/// owner's graveyard instead of onto the battlefield. For each creature card exiled this
/// way, ~ enters with a +2/+0, +1/+1, or +0/+2 counter on it." (Frankenstein's Monster):
/// exactly that many cards (other than itself) or it doesn't enter; a counter of a kind
/// chosen for each card exiled (CR 614.1c, 614.12).
fn as_enters_exile_or_graveyard(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    use crate::kw::enters_attached::ENTER_GRAVEYARD_INSTEAD;
    use crate::oracle::phrases::{end, parse_number, parse_object_phrase};
    if !ctx.is_permanent() || block.contains('\n') {
        return None;
    }
    let lower = crate::oracle::strip_ability_word(block.trim()).to_lowercase();
    let r = end(&lower).strip_prefix("as ~ enters, exile ")?;
    let sentences: Vec<&str> = r.split(". ").collect();
    let [cards, cant, each] = sentences.as_slice() else {
        return None;
    };
    if *cant != "if you can't, put ~ into its owner's graveyard instead of onto the battlefield"
    {
        return None;
    }
    let (n, r) = parse_number(cards)?;
    let (f, _, tail) = parse_object_phrase(r)?;
    let yours = format!("{f:?}").contains("OwnedBy(You)");
    match (end(tail), f.zone()) {
        ("from your graveyard", None) => {}
        ("", Some(ZoneKind::Graveyard)) if yours => {}
        _ => return None,
    }
    let filter = Filter::and(vec![
        f,
        Filter::InZone(ZoneKind::Graveyard),
        Filter::OwnedBy(PlayerRel::You),
        Filter::Other,
    ]);
    // "for each creature card exiled this way, ~ enters with a +2/+0, +1/+1, or +0/+2
    // counter on it".
    let r = each.strip_prefix("for each ")?;
    let (what, r) = r.split_once(" exiled this way, ")?;
    let (_, _, t) = parse_object_phrase(what)?;
    if !end(t).is_empty() {
        return None;
    }
    let kinds = r
        .strip_prefix("~ enters with a ")
        .or_else(|| r.strip_prefix("it enters with a "))?
        .strip_suffix(" counter on it")?;
    let mut options = Vec::new();
    for k in kinds.split(", or ").flat_map(|p| p.split(", ")) {
        let (kind, rest) = crate::oracle::costs::counter_kind(&format!("{k} counter"))
            .map(|(kind, rest)| (kind, rest.to_string()))?;
        if rest.trim() != "counter" {
            return None;
        }
        options.push((
            k.to_string(),
            Effect::EnterWithCounters {
                kind,
                n: Value::c(1),
            },
        ));
    }
    if options.len() < 2 {
        return None;
    }
    let body = Effect::If {
        cond: Condition::Compare(
            Value::Count(filter.clone()),
            Cmp::Ge,
            n.clone(),
        ),
        then: Box::new(Effect::seq(vec![
            Effect::Exile {
                what: Sel::Choose {
                    chooser: PlayerRef::You,
                    filter,
                    count: n,
                    up_to: false,
                    store: Some(EXILED_AS_ENTERED),
                },
                face_down: false,
                link: true,
            },
            Effect::Repeat {
                times: Value::CountSel(Box::new(Sel::Var(EXILED_AS_ENTERED))),
                effect: Box::new(Effect::ChooseOne {
                    who: PlayerRef::You,
                    options,
                }),
            },
        ])),
        otherwise: Box::new(Effect::Custom(ENTER_GRAVEYARD_INSTEAD.into())),
    };
    Some(vec![etb_replacement(
        ReplacementAction::AsEnters(Box::new(body)),
        block,
    )])
}

inventory::submit! { AbilityPattern { name: "as ~ enters, exile N cards; if you can't, it goes to the graveyard instead", priority: 44, parse: as_enters_exile_or_graveyard } }
