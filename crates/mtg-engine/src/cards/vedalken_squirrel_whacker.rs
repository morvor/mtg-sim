//! Vedalken Squirrel-Whacker: "If you would roll one or more six-sided dice, instead roll
//! them and you may exchange one result with this creature's base power or base
//! toughness."
//!
//! Its as-enters rolls setting its base power and toughness are compiled (dice patterns);
//! this ability is the marker [`EXCHANGE`] with the `dice_rolled` hook. When its
//! controller rolls six-sided dice, once the results are known (after the modifiers, CR
//! 706.2; ruling: "After rolling one or more six-sided dice and seeing the results, you
//! decide"), they may exchange one die's result with the creature's base power or base
//! toughness (CR 208.4b: the values after layers 7a and 7b): that die's result becomes the
//! old value, and an effect that sets the base power or toughness to the old result
//! applies in layer 7b (CR 613.4b), with a new timestamp. The exchanged result is the
//! roll's result for everything that uses it, its triggers included; its natural result
//! doesn't change (interpretation `die-result-exchange-after-modifiers`). It functions
//! only on the battlefield, so its own as-enters rolls aren't affected (ruling), and the
//! planar die has no numerical result to exchange (CR 706.7).

use super::{active, marker, ManualAbility};
use crate::ability::*;
use crate::dice::RolledDie;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::types::{CardType, ObjectId, PlayerId};

/// The marker of the ability.
pub const EXCHANGE: &str =
    "card:Vedalken Squirrel-Whacker:exchange a six-sided die result with base power or toughness";

const TEXT: &str = "If you would roll one or more six-sided dice, instead roll them and you may exchange one result with ~'s base power or base toughness.";

inventory::submit! { ManualAbility {
    card: "Vedalken Squirrel-Whacker",
    face: 0,
    text: TEXT,
    build: |_| vec![marker(EXCHANGE, TEXT)],
    reason: "exchanging a die result with a base power or toughness: the only such card",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn dice_rolled(
        &self,
        g: &mut Game,
        p: PlayerId,
        dice: &mut [RolledDie],
        bonus: i64,
        _source: Option<ObjectId>,
    ) {
        if !dice.iter().any(|d| d.sides == 6) {
            return;
        }
        // Each such permanent its controller controls offers one exchange (each is a
        // separate replacement, CR 614.5).
        let sources: Vec<ObjectId> = active(g, EXCHANGE)
            .into_iter()
            .filter(|(_, ctl)| *ctl == p)
            .map(|(src, _)| src)
            .collect();
        for src in sources {
            exchange(g, p, src, dice, bonus);
        }
    }
}

/// Offers `p` the exchange of one six-sided die result with `src`'s base power or base
/// toughness.
fn exchange(g: &mut Game, p: PlayerId, src: ObjectId, dice: &mut [RolledDie], bonus: i64) {
    if !g.is_live(src) {
        return;
    }
    g.recompute();
    let o = g.obj(src);
    if !o.is(CardType::Creature) {
        return;
    }
    let (power, toughness) = o.base_pt;
    let mut opts = vec!["Don't exchange".to_string()];
    // (die index, power?, the base value)
    let mut picks: Vec<(usize, bool, i64)> = Vec::new();
    for (i, d) in dice.iter().enumerate().filter(|(_, d)| d.sides == 6) {
        let r = d.result() + bonus;
        for (is_power, v) in [(true, power), (false, toughness)] {
            let Some(v) = v else { continue };
            let what = if is_power { "power" } else { "toughness" };
            opts.push(format!(
                "Exchange die {} (result {r}) with its base {what} {v}",
                i + 1
            ));
            picks.push((i, is_power, i64::from(v)));
        }
    }
    let k = g.ask_option(
        p,
        Some(src),
        "Exchange one result with its base power or base toughness?",
        opts,
    );
    let Some(&(i, is_power, v)) = k.checked_sub(1).and_then(|k| picks.get(k)) else {
        return;
    };
    let d = &mut dice[i];
    let result = d.result() + bonus;
    // The die's result becomes the base value ...
    d.adjust = v - bonus - d.natural;
    // ... and the base value becomes the result (layer 7b, CR 613.4b).
    let n = Some(Value::c(result.clamp(i32::MIN as i64, i32::MAX as i64) as i32));
    let set = if is_power {
        Modification::SetPT(n, None)
    } else {
        Modification::SetPT(None, n)
    };
    let mut ctx = Ctx::new(Some(src), g.obj(src).controller);
    g.exec(
        &Effect::Modify {
            what: Sel::This,
            mods: vec![set],
            duration: Duration::Permanent,
        },
        &mut ctx,
    );
    g.log(|_| {
        format!(
            "{p} exchanges a die result of {result} with base {} {v}",
            if is_power { "power" } else { "toughness" }
        )
    });
}

inventory::submit! { KeywordRegistration(&Rules) }
