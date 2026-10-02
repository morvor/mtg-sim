//! "Repeat this process" as an instruction of its own (CR 608.2c): "If it's a permanent
//! card, you may put it onto the battlefield. If you do, repeat this process." (Primal
//! Surge), "put it into your graveyard and repeat this process" (Countryside Crusher),
//! "You may repeat this process any number of times." (Ad Nauseam). The instructions
//! before it in the ability are the process, which is performed again after each pass
//! that reached the instruction (see `crate::repeat_process`; the compiler wraps them in
//! `Effect::RepeatProcess`). "Repeat this process once", "... X more times", "... for
//! [other things]" and "... until [condition]" are other instructions.

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;

/// "repeat this process"; after "may", "repeat this process any number of times" / "as
/// many times as they choose" (the choice is made again after each pass).
fn repeat_this_process(l: &str, _b: &mut Builder) -> Option<Effect> {
    match end(l) {
        "repeat this process"
        | "repeat this process any number of times"
        | "repeat this process as many times as you choose"
        | "repeat this process as many times as they choose" => Some(Effect::RepeatThisProcess),
        _ => None,
    }
}

inventory::submit! { EffectPattern { name: "repeat this process", priority: 50, parse: repeat_this_process } }
