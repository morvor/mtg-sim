//! CR 702.106 Hidden agenda, on conspiracy cards: "As you put this conspiracy card into
//! the command zone, turn it face down and secretly choose a card name." (CR 702.106a).
//!
//! * The chosen name is noted with the face-down card (CR 702.106b): it's kept on the
//!   object (its choices), hidden while the card is face down, where the abilities that
//!   refer to "the chosen name" find it. Those abilities are linked to the hidden agenda
//!   ability and refer only to the name it chose (CR 702.106d, 607.2d).
//! * Any time they have priority, its controller may turn it face up, revealing the name:
//!   a special action (CR 702.106c, 116.2j; see `special_actions.rs`).
//! * Face-down conspiracy cards are revealed when their controller leaves the game, and
//!   all of them at the end of the game (CR 702.106e; see `facedown::reveal_all`).
//! * Double agenda chooses two different names, secretly (CR 702.106f): "Double agenda" is
//!   hidden agenda with N = 2. "with one of the chosen names" ([`ONE_OF_CHOSEN_NAMES`]) and
//!   "with the other chosen name" ([`OTHER_CHOSEN_NAME`]) refer to them.

use super::{KeywordRegistration, KeywordRules};
use crate::decision::{Answer, Decision};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::types::*;
use smol_str::SmolStr;

/// `Filter::Custom`: an object with one of the names chosen for the source's double
/// agenda.
pub const ONE_OF_CHOSEN_NAMES: &str = "hidden agenda:with one of the chosen names";
/// `Filter::Custom`: an object with the chosen name other than the name of the trigger
/// event's spell ("a creature card with the other chosen name").
pub const OTHER_CHOSEN_NAME: &str = "hidden agenda:with the other chosen name";

pub struct HiddenAgenda;

impl KeywordRules for HiddenAgenda {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::HiddenAgenda]
    }

    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, ctx: &Ctx) -> Option<bool> {
        let names = || ctx.source.map(|s| chosen_names(g, s)).unwrap_or_default();
        let has = |n: &SmolStr| g.obj(id).chars.has_name(n);
        match name {
            ONE_OF_CHOSEN_NAMES => Some(names().iter().any(has)),
            OTHER_CHOSEN_NAME => {
                let spell = ctx.event.as_ref().and_then(|e| e.spell.or(e.object));
                let others: Vec<SmolStr> = names()
                    .into_iter()
                    .filter(|n| spell.is_none_or(|s| !g.obj(s).chars.has_name(n)))
                    .collect();
                Some(others.iter().any(has))
            }
            _ => None,
        }
    }
}

/// The number of names a conspiracy card's hidden agenda chooses: two for double agenda,
/// none without hidden agenda.
fn names_to_choose(g: &Game, card: ObjectId) -> usize {
    g.obj(card)
        .base
        .keywords()
        .find(|k| k.kind == KeywordKind::HiddenAgenda)
        .map_or(0, |k| k.n.unwrap_or(1).max(1) as usize)
}

/// The names chosen for `card`'s hidden agenda (CR 702.106a, 702.106f).
pub fn chosen_names(g: &Game, card: ObjectId) -> Vec<SmolStr> {
    let ch = &g.obj(card).choices;
    match &ch.text {
        Some(t) if t.contains('|') => t.split('|').map(SmolStr::new).collect(),
        _ => ch.card_name.iter().filter(|n| !n.is_empty()).cloned().collect(),
    }
}

/// CR 702.106a, 702.106f: as `p` puts the conspiracy card `card` with hidden agenda into
/// the command zone, it's turned face down and `p` secretly chooses a card name (two
/// different names for double agenda). An invalid answer names nothing (the choice stays
/// undefined, CR 607.5a).
pub fn as_put_into_command_zone(g: &mut Game, p: PlayerId, card: ObjectId) {
    let n = names_to_choose(g, card);
    if n == 0 {
        return;
    }
    g.objects[card.0 as usize].face_down = true;
    let mut names: Vec<SmolStr> = Vec::new();
    for i in 0..n {
        let prompt = if n == 1 {
            "Secretly choose a card name (hidden agenda)".to_string()
        } else {
            format!("Secretly choose card name {} of {n} (double agenda)", i + 1)
        };
        let answer = match g.ask(
            p,
            Decision::NameCard {
                source: Some(card),
                prompt,
            },
        ) {
            Answer::Text(t) => t.trim().to_string(),
            _ => String::new(),
        };
        if crate::choices::valid_card_name(&answer, None)
            && !names.iter().any(|x| x.eq_ignore_ascii_case(&answer))
        {
            names.push(SmolStr::new(answer));
        }
    }
    let ch = &mut g.objects[card.0 as usize].choices;
    ch.card_name = Some(names.first().cloned().unwrap_or_default());
    if n > 1 {
        ch.text = Some(SmolStr::new(
            names.iter().map(|s| s.as_str()).collect::<Vec<_>>().join("|"),
        ));
    }
    g.dirty = true;
}

inventory::submit! { KeywordRegistration(&HiddenAgenda) }
