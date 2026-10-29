//! Type choices (cards5-choices-players): "each player chooses artifact, creature, or land
//! during their untap step. That player can untap only permanents of the chosen type this
//! step." (Storage Matrix; see `untap_limits.rs`, CR 502.3).

use super::StaticPattern;
use crate::ability::*;
use crate::oracle::phrases::end;
use crate::oracle::CompileContext;
use crate::types::CardType;

/// "artifact, creature, or land": two or more card type words.
fn card_type_list(s: &str) -> Option<Vec<CardType>> {
    let s = s.replace(", or ", ", ").replace(" or ", ", ");
    let v: Vec<CardType> = s
        .split(", ")
        .map(|w| CardType::from_word(w.trim()))
        .collect::<Option<_>>()?;
    let mut d = v.clone();
    d.dedup();
    (v.len() >= 2 && d.len() == v.len()).then_some(v)
}

fn untap_only_chosen_type(l: &str, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let l = end(l);
    let (cond, rest) = match l.strip_prefix("as long as ") {
        Some(r) => {
            let (c, rest) = r.split_once(", ")?;
            let (cond, _) =
                super::statics_conditions::parse_static_condition(c, Some(&Sel::This), ctx)?;
            (Some(cond), rest)
        }
        None => (None, l),
    };
    let list = rest
        .strip_prefix("each player chooses ")?
        .strip_suffix(
            " during their untap step. that player can untap only permanents of the chosen type this step",
        )?;
    let types = card_type_list(list)?;
    let name = format!(
        "{}{}",
        crate::untap_limits::UNTAP_ONLY_CHOSEN_TYPE,
        types.iter().map(|t| t.word()).collect::<Vec<_>>().join(",")
    );
    let mut s = StaticAbility::new(StaticEffect::Restriction(Restriction::Custom(name.into())));
    s.condition = cond;
    Some(vec![AbilityDef::new(AbilityKind::Static(s), text)])
}

inventory::submit! { StaticPattern { name: "each player chooses [card types] during their untap step; untap only the chosen type", priority: 100, parse: untap_only_chosen_type } }
