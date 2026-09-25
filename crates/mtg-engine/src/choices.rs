//! "Choose a color / creature type / card name / number" effects, stored on the source.

use crate::ability::ChoiceKind;
use crate::decision::{Answer, Decision};
use crate::eval::Ctx;
use crate::game::Game;
use crate::types::*;
use smol_str::SmolStr;

pub fn make_choice(g: &mut Game, p: PlayerId, kind: &ChoiceKind, ctx: &mut Ctx) {
    let Some(src) = ctx.source else { return };
    match kind {
        ChoiceKind::Color => {
            let opts: Vec<String> = Color::ALL.iter().map(|c| c.word().to_string()).collect();
            let i = g.ask_option(p, Some(src), "Choose a color", opts);
            g.objects[src.0 as usize].choices.color = Some(Color::ALL[i.min(4)]);
        }
        ChoiceKind::ColorOtherThan(except) => {
            let cols: Vec<Color> = Color::ALL.iter().copied().filter(|c| c != except).collect();
            let opts: Vec<String> = cols.iter().map(|c| c.word().to_string()).collect();
            let i = g.ask_option(p, Some(src), "Choose a color", opts);
            g.objects[src.0 as usize].choices.color =
                cols.get(i).copied().or(cols.first().copied());
        }
        ChoiceKind::CreatureType => {
            let list: Vec<String> = subtype_lists().creature.clone();
            let i = g.ask_option(p, Some(src), "Choose a creature type", list.clone());
            g.objects[src.0 as usize].choices.creature_type = list.get(i).map(SmolStr::new);
        }
        ChoiceKind::BasicLandType => {
            let list: Vec<String> = ["Plains", "Island", "Swamp", "Mountain", "Forest"]
                .iter()
                .map(|s| s.to_string())
                .collect();
            let i = g.ask_option(p, Some(src), "Choose a basic land type", list.clone());
            g.objects[src.0 as usize].choices.basic_land_type = list.get(i).map(SmolStr::new);
        }
        ChoiceKind::CardType => {
            let list: Vec<String> = CardType::ALL.iter().map(|t| t.word().to_string()).collect();
            let i = g.ask_option(p, Some(src), "Choose a card type", list);
            g.objects[src.0 as usize].choices.card_type =
                Some(CardType::ALL[i.min(CardType::ALL.len() - 1)]);
        }
        ChoiceKind::CardName | ChoiceKind::CardNameFiltered(_) => {
            let name = match g.ask(
                p,
                Decision::NameCard {
                    source: Some(src),
                    prompt: "Name a card".into(),
                },
            ) {
                Answer::Text(t) => t,
                _ => String::new(),
            };
            g.objects[src.0 as usize].choices.card_name = Some(SmolStr::new(name));
        }
        ChoiceKind::Number { min, max } => {
            let n = g.ask_number(p, Some(src), "Choose a number", *min as i64, *max as i64);
            g.objects[src.0 as usize].choices.number = Some(n as i32);
        }
        ChoiceKind::Opponent | ChoiceKind::Player => {
            let cands: Vec<Entity> = if *kind == ChoiceKind::Opponent {
                g.opponents(p).into_iter().map(Entity::Player).collect()
            } else {
                g.players_in_game()
                    .into_iter()
                    .map(Entity::Player)
                    .collect()
            };
            if let Some(pl) = g
                .ask_entities(p, Some(src), "Choose a player", cands, 1, 1)
                .first()
                .and_then(|e| e.player())
            {
                g.objects[src.0 as usize].choices.player = Some(pl);
                ctx.chosen_player = Some(pl);
            }
        }
        ChoiceKind::OddOrEven => {
            let i = g.ask_option(
                p,
                Some(src),
                "Choose odd or even",
                vec!["odd".into(), "even".into()],
            );
            g.objects[src.0 as usize].choices.text =
                Some(if i == 0 { "odd".into() } else { "even".into() });
        }
    }
    g.dirty = true;
}

/// Whether a filter refers to a choice made for its source ("of the chosen color").
pub fn filter_mentions_choice(f: &crate::ability::Filter) -> bool {
    use crate::ability::Filter;
    match f {
        Filter::ChosenColor | Filter::ChosenType | Filter::ChosenName | Filter::ChosenCardType => {
            true
        }
        Filter::And(v) | Filter::Or(v) => v.iter().any(filter_mentions_choice),
        Filter::Not(x) => filter_mentions_choice(x),
        _ => false,
    }
}

/// Replaces references to chosen values with the values chosen for a particular object,
/// for abilities granted to other objects (CR 607.2d). An undefined choice matches
/// nothing (CR 607.5a).
pub fn bind_choices(
    f: &crate::ability::Filter,
    ch: &crate::object::Choices,
) -> crate::ability::Filter {
    use crate::ability::Filter;
    let nothing = || Filter::not(Filter::Any);
    match f {
        Filter::ChosenColor => ch.color.map(Filter::Color).unwrap_or_else(nothing),
        Filter::ChosenType => ch
            .creature_type
            .clone()
            .or(ch.basic_land_type.clone())
            .map(Filter::Subtype)
            .or(ch.card_type.map(Filter::Type))
            .unwrap_or_else(nothing),
        Filter::ChosenName => ch
            .card_name
            .clone()
            .filter(|n| !n.is_empty())
            .map(Filter::Named)
            .unwrap_or_else(nothing),
        Filter::ChosenCardType => ch.card_type.map(Filter::Type).unwrap_or_else(nothing),
        Filter::And(v) => Filter::And(v.iter().map(|x| bind_choices(x, ch)).collect()),
        Filter::Or(v) => Filter::Or(v.iter().map(|x| bind_choices(x, ch)).collect()),
        Filter::Not(x) => Filter::Not(Box::new(bind_choices(x, ch))),
        other => other.clone(),
    }
}
