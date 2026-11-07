//! "Transform it" (CR 701.27a) where "it" names an object the clause before introduced:
//! "unattach Elbrus, then transform it" (Elbrus, the Binding Blade).

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;

fn transform_it(l: &str, b: &mut Builder) -> Option<Effect> {
    if end(l) != "transform it" || !matches!(b.it, Sel::This) {
        return None;
    }
    Some(Effect::Transform { what: Sel::This })
}

inventory::submit! { EffectPattern { name: "transform it (~)", priority: 100, parse: transform_it } }
