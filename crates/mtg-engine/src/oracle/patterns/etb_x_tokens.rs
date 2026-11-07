//! "When this enchantment enters, create an X/X green Treefolk creature token." (Arboreal
//! Alliance): a permanent's own enters-the-battlefield triggered ability that refers to X
//! with nothing defining it uses the value of X chosen for the spell that became the
//! permanent (CR 107.3m). The trigger is compiled with X defined; its `Value::X` reads the
//! spell's cast information as the ability resolves.

use super::AbilityPattern;
use crate::ability::*;
use crate::oracle::CompileContext;

fn etb_x_x_token(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    // Already compiling with X defined (this pattern's own recursion, or a "where X is").
    if ctx.is_spell() || !ctx.is_permanent() || super::value_grammar::x_defined() {
        return None;
    }
    let text = crate::oracle::strip_ability_word(block.trim());
    let lower = text.to_lowercase();
    let (head, body) = lower.split_once(", ")?;
    if !head.starts_with("when ")
        || !(head.ends_with(" enters") && (head.contains(" this ") || head.contains('~')))
        || head.contains(" or ")
        || !(body.starts_with("create an x/x ") || body.starts_with("create a x/x "))
        || body.contains("where x")
        || body.contains('"')
    {
        return None;
    }
    let mut abilities = super::value_grammar::with_x_defined(true, || {
        crate::oracle::parse_ability(&text, ctx)
    })?;
    for a in &mut abilities {
        std::sync::Arc::make_mut(a).text = block.to_string();
    }
    Some(abilities)
}

inventory::submit! { AbilityPattern { name: "etb x/x token: the spell's X (CR 107.3m)", priority: 104, parse: etb_x_x_token } }
