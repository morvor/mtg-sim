//! "If you would lose the game, instead [effect]." (Lich's Mirror, The Golden Throne): a
//! replacement effect on the game-loss event (CR 104.3, 614.1a). The effect must compile
//! (Exquisite Archangel's "becomes equal to your starting life total" doesn't yet).
//! It replaces losing for any reason except conceding (CR 104.3a); a player who wins
//! ends the game without anyone "losing" through this event (CR 104.2).

use super::StaticPattern;
use crate::ability::*;
use crate::oracle::CompileContext;

fn lose_game_instead(l: &str, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let clause = l.strip_prefix("if you would lose the game, instead ")?;
    let clause = clause.strip_suffix('.').unwrap_or(clause);
    if clause.contains('.') {
        return None;
    }
    let mut b = crate::oracle::effects::Builder::new(ctx);
    let effect = crate::oracle::effects::parse_effect_text(clause, &mut b)?;
    if !b.targets.is_empty() {
        return None;
    }
    let s = StaticAbility::new(StaticEffect::Replacement(ReplacementDef {
        event: ReplacementEvent::LoseGame(PlayerFilter::You),
        action: ReplacementAction::Instead(Box::new(effect)),
        self_replacement: false,
        optional: false,
    }));
    Some(vec![AbilityDef::new(AbilityKind::Static(s), text)])
}

inventory::submit! { StaticPattern { name: "would lose the game, instead", priority: 0, parse: lose_game_instead } }
