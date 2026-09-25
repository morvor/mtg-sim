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
        ChoiceKind::Word(words) => {
            let i = g.ask_option(p, Some(src), "Choose one", words.clone());
            g.objects[src.0 as usize].choices.text =
                words.get(i).or(words.first()).map(SmolStr::new);
        }
    }
    // CR 607.2d: the choice belongs to the abilities linked to the one that made it.
    let made = g.obj(src).choices.clone();
    let entry = g.objects[src.0 as usize]
        .linked_choices
        .entry(ctx.link)
        .or_default();
    match kind {
        ChoiceKind::Color => entry.color = made.color,
        ChoiceKind::CreatureType => entry.creature_type = made.creature_type,
        ChoiceKind::BasicLandType => entry.basic_land_type = made.basic_land_type,
        ChoiceKind::CardType => entry.card_type = made.card_type,
        ChoiceKind::CardName | ChoiceKind::CardNameFiltered(_) => entry.card_name = made.card_name,
        ChoiceKind::Number { .. } => entry.number = made.number,
        ChoiceKind::Opponent | ChoiceKind::Player => entry.player = made.player,
        ChoiceKind::OddOrEven | ChoiceKind::Word(_) => entry.text = made.text,
    }
    g.dirty = true;
}
