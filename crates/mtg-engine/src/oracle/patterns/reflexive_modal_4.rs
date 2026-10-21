//! Modal headers with extra structure (CR 700.2):
//!
//! - A header joined with an instruction: "choose one and ~ gets +1/+1 until end of turn
//!   —" (Glorfindel, Dauntless Rescuer), "choose one that hasn't been chosen and you lose
//!   2 life —" (Zuko, Conflicted). The modes are chosen as the ability is put on the
//!   stack; as it resolves, the chosen mode is performed, then the instruction (CR
//!   608.2c). With no mode chosen, the ability is removed from the stack (CR 700.2b) and
//!   the instruction isn't performed.

use crate::ability::*;
use crate::oracle::effects::{parse_effect_text, parse_modal, Builder};
use crate::oracle::CompileContext;

/// "[header] and [instruction] —\n• mode\n• mode": the modal body, with the instruction
/// as its effect (performed after the chosen modes, see `Game::exec_chosen`).
pub fn modal_with_effect(
    t: &str,
    ctx: &CompileContext,
    it: &Sel,
    it_player: &PlayerRef,
) -> Option<Body> {
    let (head, rest) = t.split_once('\n')?;
    let h = head.trim().trim_end_matches(['—', ':', ' ']);
    let lower = h.to_lowercase();
    if !lower.starts_with("choose ") {
        return None;
    }
    // The header is the words before the first " and " that leave a header.
    let mut from = 0;
    while let Some(i) = lower[from..].find(" and ") {
        let at = from + i;
        from = at + " and ".len();
        let header = &h[..at];
        let instruction = &h[at + " and ".len()..];
        let Some(modal) = parse_modal(&format!("{header}\n{rest}"), ctx, Some((it, it_player)))
        else {
            continue;
        };
        let mut b = Builder::new(ctx);
        b.in_trigger = true;
        b.it = it.clone();
        b.it_player = it_player.clone();
        let Some(effect) = parse_effect_text(instruction, &mut b) else {
            continue;
        };
        // Targets of the instruction would have to be chosen with the modes' (none here).
        if !b.targets.is_empty() || matches!(effect, Effect::Noop) {
            continue;
        }
        return Some(Body {
            targets: vec![],
            effect,
            modal: Some(modal),
        });
    }
    None
}

#[cfg(test)]
mod tests {
    use super::super::followup_grammar::tests::compiled;

    #[test]
    fn modal_headers_with_an_instruction() {
        for (ty, text) in [
            ("Legendary Creature", "Whenever you scry, choose one and ~ gets +1/+1 until end of turn.\n• ~ must be blocked this turn if able.\n• ~ can't be blocked by more than one creature each combat this turn."),
            ("Legendary Creature", "At the beginning of your first main phase, choose one that hasn't been chosen and you lose 2 life —\n• Draw a card.\n• Put a +1/+1 counter on ~.\n• Add {R}.\n• Exile ~, then return him to the battlefield under an opponent's control."),
        ] {
            assert!(compiled(ty, text).is_some(), "{text}");
        }
    }
}
