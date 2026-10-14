//! Searching (CR 701.23): "If an opponent would search a library, that player searches
//! the top four cards of that library instead." (Aven Mindcensor, CR 701.23f).

use super::StaticPattern;
use crate::ability::*;
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;
use crate::search_rules::SEARCH_PORTION;
use smol_str::SmolStr;

fn search_portion(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let l = end(l);
    let (who, r) = if let Some(r) = l.strip_prefix(
        "if an opponent would search a library, that player searches the top ",
    ) {
        ("opponents", r)
    } else if let Some(r) =
        l.strip_prefix("if a player would search a library, that player searches the top ")
    {
        ("each", r)
    } else {
        return None;
    };
    let (n, r) = parse_number(r)?;
    let n = n.as_const()?;
    if r.trim() != "cards of that library instead" {
        return None;
    }
    let name = SmolStr::new(format!("{SEARCH_PORTION}{n}:{who}"));
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Custom(name))),
        text,
    )])
}

inventory::submit! { StaticPattern { name: "a701 search a portion", priority: 100, parse: search_portion } }
