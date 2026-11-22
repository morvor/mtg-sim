//! "the chosen color is the most common color among nontoken permanents the chosen player
//! controls but isn't tied for most common" (Call to Arms): a `Condition::Custom` built by
//! [`name`] and compiled by `oracle/patterns/paper_triggers.rs`.
//!
//! Call to Arms' rulings: each nontoken permanent the chosen player controls counts once for
//! each of its colors and colorless ones are ignored; the color is the most common only
//! while more of those permanents have it than have any other color (a tie isn't enough).
//! The color and the player are those chosen for the source as it entered (CR 607.2d),
//! even if its controller changes.

use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::types::*;

const PREFIX: &str = "most common color:";

/// Whose permanents are counted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Among {
    /// "nontoken permanents the chosen player controls".
    ChosenPlayersNontoken,
}

/// The condition "[the chosen color / a color] is the most common color among [permanents]
/// [and isn't tied]": `color` is a color letter or `None` for the chosen color; with
/// `strict`, a tie doesn't count.
pub fn name(color: Option<Color>, among: Among, strict: bool) -> String {
    let c = color.map_or("chosen".to_string(), |c| c.letter().to_string());
    let a = match among {
        Among::ChosenPlayersNontoken => "chosen player's nontoken",
    };
    format!("{PREFIX}{c}:{a}:{}", if strict { "strict" } else { "or tied" })
}

/// The parts of a name built by [`name`].
pub fn parse_name(n: &str) -> Option<(Option<Color>, Among, bool)> {
    let mut it = n.strip_prefix(PREFIX)?.split(':');
    let c = it.next()?;
    let color = if c == "chosen" {
        None
    } else {
        Some(Color::from_letter(c.chars().next()?)?)
    };
    let among = match it.next()? {
        "chosen player's nontoken" => Among::ChosenPlayersNontoken,
        _ => return None,
    };
    let strict = match it.next()? {
        "strict" => true,
        "or tied" => false,
        _ => return None,
    };
    Some((color, among, strict))
}

fn holds(g: &Game, name: &str, ctx: &Ctx) -> Option<bool> {
    let (color, among, strict) = parse_name(name)?;
    let color = match color {
        Some(c) => Some(c),
        None => g.source_choices(ctx).and_then(|ch| ch.color),
    };
    // An undefined choice (CR 607.5a): no color is the chosen one.
    let Some(color) = color else {
        return Some(false);
    };
    let Among::ChosenPlayersNontoken = among;
    let Some(player) = g.source_choices(ctx).and_then(|ch| ch.player) else {
        return Some(false);
    };
    let count = |c: Color| {
        g.permanents()
            .filter(|o| o.controller == player && !o.is_token() && o.chars.colors.contains(c))
            .count()
    };
    let n = count(color);
    Some(Color::ALL.iter().filter(|c| **c != color).all(|c| {
        let m = count(*c);
        if strict {
            m < n
        } else {
            m <= n
        }
    }))
}

struct MostCommonChosenColor;

impl super::KeywordRules for MostCommonChosenColor {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_condition(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<bool> {
        if !name.starts_with(PREFIX) {
            return None;
        }
        holds(g, name, ctx)
    }
}

inventory::submit! { super::KeywordRegistration(&MostCommonChosenColor) }
