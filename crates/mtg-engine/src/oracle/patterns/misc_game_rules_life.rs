//! "You don't lose the game for having 0 or less life." (Lich, Phyrexian Unlife, Soul
//! Echo): the state-based action of CR 704.5a doesn't apply to the player (a player
//! modification read by `sba.rs` through `life_totals::ignores_zero_life`), and "You
//! have no maximum hand size and don't lose the game for having 0 or less life."

use super::StaticPattern;
use crate::ability::*;
use crate::life_totals::NO_LOSS_FOR_ZERO_LIFE;
use crate::oracle::phrases::end;
use crate::oracle::CompileContext;

fn player_static(affected: PlayerFilter, effect: PlayerModification, text: &str) -> Ability {
    AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::PlayerEffect {
            affected,
            effect,
        })),
        text,
    )
}

fn no_loss() -> PlayerModification {
    PlayerModification::Custom(NO_LOSS_FOR_ZERO_LIFE.into())
}

/// "[players] don't lose the game for having 0 or less life".
fn dont_lose_for_zero_life(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let l = end(l);
    let (who, rest) = [
        ("you ", PlayerFilter::You),
        ("players ", PlayerFilter::Any),
        ("each player ", PlayerFilter::Any),
    ]
    .into_iter()
    .find_map(|(p, f)| l.strip_prefix(p).map(|r| (f, r)))?;
    let mut out = Vec::new();
    let rest = match rest.strip_prefix("have no maximum hand size and ") {
        // Marina Vendrell's Grimoire: two player effects in one sentence (CR 402.2).
        Some(r) if matches!(who, PlayerFilter::You) => {
            out.push(player_static(
                who.clone(),
                PlayerModification::MaxHandSize(None),
                text,
            ));
            r
        }
        Some(_) => return None,
        None => rest,
    };
    if !matches!(
        rest,
        "don't lose the game for having 0 or less life"
            | "doesn't lose the game for having 0 or less life"
    ) {
        return None;
    }
    out.push(player_static(who, no_loss(), text));
    Some(out)
}

inventory::submit! { StaticPattern { name: "misc: don't lose the game for having 0 or less life", priority: 100, parse: dont_lose_for_zero_life } }
