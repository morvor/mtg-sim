//! "Choose a color / creature type / card name / number" effects, stored on the source.

use crate::ability::ChoiceKind;
use crate::decision::{Answer, Decision};
use crate::eval::Ctx;
use crate::game::Game;
use crate::types::*;
use smol_str::SmolStr;

/// The greatest number a player may choose when the text sets no upper limit ("choose a
/// number", "a number 0 or greater", "a number greater than 0", hidden items): far beyond
/// any life total, library or mana value in a game, and small enough that the engine's
/// arithmetic on it (life totals, damage) can't overflow (interpretation
/// `unbounded-number-ceiling`).
pub const MAX_CHOSEN_NUMBER: i32 = 1_000_000;

pub fn make_choice(g: &mut Game, p: PlayerId, kind: &ChoiceKind, ctx: &mut Ctx) {
    let Some(src) = ctx.source else { return };
    match kind {
        ChoiceKind::Color => {
            let opts: Vec<String> = Color::ALL.iter().map(|c| c.word().to_string()).collect();
            let i = g.ask_option(p, Some(src), "Choose a color", opts);
            g.objects[src.0 as usize].choices.color = Some(Color::ALL[i.min(4)]);
        }
        ChoiceKind::Colors => {
            // Any single color or any combination of them; not colorless (CR 105.4).
            let sets: Vec<ColorSet> = (1..32u8).map(ColorSet).collect();
            let opts: Vec<String> = sets
                .iter()
                .map(|s| s.iter().map(|c| c.word()).collect::<Vec<_>>().join(" and "))
                .collect();
            let i = g.ask_option(p, Some(src), "Choose a color or colors", opts);
            g.objects[src.0 as usize].choices.colors = Some(sets[i.min(sets.len() - 1)]);
        }
        ChoiceKind::ColorsExactly(n) => {
            let sets: Vec<ColorSet> = (1..32u8)
                .map(ColorSet)
                .filter(|s| s.iter().count() == *n as usize)
                .collect();
            if sets.is_empty() {
                return;
            }
            let opts: Vec<String> = sets
                .iter()
                .map(|s| s.iter().map(|c| c.word()).collect::<Vec<_>>().join(" and "))
                .collect();
            let i = g.ask_option(p, Some(src), "Choose colors", opts);
            g.objects[src.0 as usize].choices.colors = Some(sets[i.min(sets.len() - 1)]);
        }
        ChoiceKind::ColorOtherThan(except) => {
            let cols: Vec<Color> = Color::ALL.iter().copied().filter(|c| c != except).collect();
            let opts: Vec<String> = cols.iter().map(|c| c.word().to_string()).collect();
            let i = g.ask_option(p, Some(src), "Choose a color", opts);
            g.objects[src.0 as usize].choices.color =
                cols.get(i).copied().or(cols.first().copied());
        }
        ChoiceKind::OneOf(words) => {
            let i = g.ask_option(p, Some(src), "Choose one", words.clone());
            let Some(w) = words.get(i).or(words.first()) else {
                return;
            };
            let ch = &mut g.objects[src.0 as usize].choices;
            ch.text = Some(SmolStr::new(w));
            if let Some(c) = Color::from_word(w) {
                ch.color = Some(c);
            } else if let Some(t) = CardType::from_word(w) {
                ch.card_type = Some(t);
            } else if is_land_type(w) {
                // "choose a land type": basic or not (stored with the basic land types).
                ch.basic_land_type = Some(SmolStr::new(w));
            } else if is_creature_type(w) {
                ch.creature_type = Some(SmolStr::new(w));
            }
        }
        ChoiceKind::CreatureTypeOtherThan(except) => {
            let list: Vec<String> = subtype_lists()
                .creature
                .iter()
                .filter(|t| !except.contains(t))
                .cloned()
                .collect();
            let i = g.ask_option(p, Some(src), "Choose a creature type", list.clone());
            g.objects[src.0 as usize].choices.creature_type = list.get(i).map(SmolStr::new);
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
                Answer::Text(t) => t.trim().to_string(),
                _ => String::new(),
            };
            let filter = match kind {
                ChoiceKind::CardNameFiltered(f) => Some(f.as_str()),
                _ => None,
            };
            // An invalid answer names nothing (the choice stays undefined, CR 607.5a).
            let name = if valid_card_name(&name, filter) {
                name
            } else {
                String::new()
            };
            g.objects[src.0 as usize].choices.card_name = Some(SmolStr::new(name));
        }
        ChoiceKind::Number { min, max } => {
            let n = g.ask_number(p, Some(src), "Choose a number", *min as i64, *max as i64);
            g.objects[src.0 as usize].choices.number = Some(n as i32);
        }
        ChoiceKind::Opponent | ChoiceKind::Player => {
            let cands: Vec<Entity> = if *kind == ChoiceKind::Opponent {
                g.opponents(p)
            } else {
                g.players_in_game()
            }
            .into_iter()
            // CR 801.5a: a player within the chooser's range of influence.
            .filter(|q| crate::multiplayer::range::player_in_range(g, p, *q))
            .map(Entity::Player)
            .collect();
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
        ChoiceKind::Players(n) => {
            let cands: Vec<Entity> = g
                .players_in_game()
                .into_iter()
                .filter(|q| crate::multiplayer::range::player_in_range(g, p, *q))
                .map(Entity::Player)
                .collect();
            let n = (*n as usize).min(cands.len()) as u32;
            let mut picked: Vec<PlayerId> = g
                .ask_entities(p, Some(src), "Choose players", cands.clone(), n, n)
                .into_iter()
                .filter_map(|e| e.player())
                .collect();
            picked.sort_by_key(|q| q.0);
            picked.dedup();
            // An invalid answer: the first players.
            if picked.len() != n as usize {
                picked = cands
                    .iter()
                    .take(n as usize)
                    .filter_map(|e| e.player())
                    .collect();
            }
            g.objects[src.0 as usize].choices.players = picked;
        }
        ChoiceKind::Distinct(n, words) => {
            let mut left = words.clone();
            let mut chosen: Vec<SmolStr> = Vec::new();
            for k in 0..*n {
                if left.is_empty() {
                    break;
                }
                let prompt = format!("Choose ({} of {n})", k + 1);
                let i = g.ask_option(p, Some(src), &prompt, left.clone());
                let w = left.remove(i.min(left.len() - 1));
                chosen.push(SmolStr::new(w));
            }
            let ch = &mut g.objects[src.0 as usize].choices;
            ch.text = chosen.first().cloned();
            ch.list = chosen;
        }
    }
    // The ordered list of the words and types chosen (for "the first/second chosen
    // type").
    {
        let ch = &mut g.objects[src.0 as usize].choices;
        let word = match kind {
            ChoiceKind::OneOf(_) | ChoiceKind::Word(_) | ChoiceKind::OddOrEven => ch.text.clone(),
            ChoiceKind::BasicLandType => ch.basic_land_type.clone(),
            ChoiceKind::CreatureType | ChoiceKind::CreatureTypeOtherThan(_) => {
                ch.creature_type.clone()
            }
            ChoiceKind::CardName | ChoiceKind::CardNameFiltered(_) => ch.card_name.clone(),
            _ => None,
        };
        if let Some(w) = word {
            ch.list.push(w);
        }
    }
    // CR 607.2d: the choice belongs to the abilities linked to the one that made it.
    let made = g.obj(src).choices.clone();
    let o = &mut g.objects[src.0 as usize];
    let entry = o.linked_choices.entry(ctx.link).or_default();
    copy_choice(kind, &made, entry);
    // Each player's own choice ("each player chooses war or peace"), kept for the
    // object and for the linked abilities.
    record_for_player(&mut o.choices, p, kind, &made, ctx.stack_obj);
    record_for_player(entry, p, kind, &made, ctx.stack_obj);
    // A secret choice isn't shown to the other players until it's revealed.
    if made.secret_next {
        for ch in [&mut o.choices, entry] {
            if !ch.secret.contains(&p) {
                ch.secret.push(p);
            }
        }
    }
    g.dirty = true;
}

/// Copies the value `kind` chose from `made` into `to`.
fn copy_choice(kind: &ChoiceKind, made: &crate::object::Choices, to: &mut crate::object::Choices) {
    // The ordered list of what was chosen ("the first chosen type").
    to.list = made.list.clone();
    match kind {
        ChoiceKind::Color | ChoiceKind::ColorOtherThan(_) => to.color = made.color,
        ChoiceKind::Colors | ChoiceKind::ColorsExactly(_) => to.colors = made.colors,
        // A word choice may also name a color or type (see above).
        ChoiceKind::OneOf(_) => {
            if let Some(w) = made.text.clone() {
                if Color::from_word(&w).is_some() {
                    to.color = made.color;
                } else if CardType::from_word(&w).is_some() {
                    to.card_type = made.card_type;
                } else if is_land_type(&w) {
                    to.basic_land_type = made.basic_land_type.clone();
                } else if is_creature_type(&w) {
                    to.creature_type = made.creature_type.clone();
                }
                to.text = Some(w);
            }
        }
        ChoiceKind::CreatureType | ChoiceKind::CreatureTypeOtherThan(_) => {
            to.creature_type = made.creature_type.clone()
        }
        ChoiceKind::BasicLandType => to.basic_land_type = made.basic_land_type.clone(),
        ChoiceKind::CardType => to.card_type = made.card_type,
        ChoiceKind::CardName | ChoiceKind::CardNameFiltered(_) => {
            to.card_name = made.card_name.clone()
        }
        ChoiceKind::Number { .. } => to.number = made.number,
        ChoiceKind::Opponent | ChoiceKind::Player => to.player = made.player,
        ChoiceKind::OddOrEven | ChoiceKind::Word(_) => to.text = made.text.clone(),
        ChoiceKind::Distinct(..) => to.text = made.text.clone(),
        ChoiceKind::Players(_) => to.players = made.players.clone(),
    }
}

/// Records what `p` chose (the value `kind` chose, in `made`) among `ch`'s per-player
/// choices.
fn record_for_player(
    ch: &mut crate::object::Choices,
    p: PlayerId,
    kind: &ChoiceKind,
    made: &crate::object::Choices,
    during: Option<ObjectId>,
) {
    let i = match ch.by_player.iter().position(|(q, _)| *q == p) {
        Some(i) => i,
        None => {
            ch.by_player.push((p, Default::default()));
            ch.by_player.len() - 1
        }
    };
    let mine = &mut ch.by_player[i].1;
    copy_choice(kind, made, mine);
    mine.made_during = during;
}

/// What `p` chose for an object, as a word ("war", "red", "Goblin", a card name, a
/// number): the most recent of that player's choices that is a word.
pub fn player_choice_matches(ch: &crate::object::Choices, p: PlayerId, word: &str) -> bool {
    let Some((_, c)) = ch.by_player.iter().find(|(q, _)| *q == p) else {
        return false;
    };
    let w = word.to_lowercase();
    c.text.as_ref().is_some_and(|t| t.to_lowercase() == w)
        || c.color.is_some_and(|x| x.word() == w)
        || c.creature_type
            .as_ref()
            .is_some_and(|t| t.to_lowercase() == w)
        || c.basic_land_type
            .as_ref()
            .is_some_and(|t| t.to_lowercase() == w)
        || c.card_type.is_some_and(|t| t.word() == w)
        || c.card_name.as_ref().is_some_and(|t| t.to_lowercase() == w)
        || c.number.is_some_and(|n| n.to_string() == w)
}

/// Whether a filter refers to a choice made for its source ("of the chosen color").
pub fn filter_mentions_choice(f: &crate::ability::Filter) -> bool {
    use crate::ability::Filter;
    match f {
        Filter::ChosenColor | Filter::ChosenType | Filter::ChosenName | Filter::ChosenCardType => {
            true
        }
        // "protection from the chosen player": what that player controls.
        Filter::ControlledBy(crate::ability::PlayerRel::Chosen) => true,
        // "with mana value equal to the chosen number"
        Filter::ManaValue(_, v) | Filter::Power(_, v) | Filter::Toughness(_, v) => {
            matches!(**v, crate::ability::Value::Chosen)
        }
        Filter::And(v) | Filter::Or(v) => v.iter().any(filter_mentions_choice),
        Filter::Not(x) => filter_mentions_choice(x),
        _ => false,
    }
}

/// Whether a filter refers to a choice that is undefined for its source (no such choice
/// was made, CR 607.5a).
pub fn filter_refers_to_undefined_choice(
    f: &crate::ability::Filter,
    ch: Option<&crate::object::Choices>,
) -> bool {
    use crate::ability::Filter;
    match f {
        Filter::ChosenColor => ch.and_then(|c| c.color).is_none(),
        Filter::ChosenType => ch.is_none_or(|c| {
            c.creature_type.is_none() && c.basic_land_type.is_none() && c.card_type.is_none()
        }),
        Filter::ChosenName => ch
            .and_then(|c| c.card_name.as_ref())
            .is_none_or(|n| n.is_empty()),
        Filter::ChosenCardType => ch.and_then(|c| c.card_type).is_none(),
        Filter::ControlledBy(crate::ability::PlayerRel::Chosen) => {
            ch.and_then(|c| c.player).is_none()
        }
        Filter::ManaValue(_, v) | Filter::Power(_, v) | Filter::Toughness(_, v)
            if matches!(**v, crate::ability::Value::Chosen) =>
        {
            ch.and_then(|c| c.number).is_none()
        }
        Filter::And(v) | Filter::Or(v) => {
            v.iter().any(|x| filter_refers_to_undefined_choice(x, ch))
        }
        Filter::Not(x) => filter_refers_to_undefined_choice(x, ch),
        _ => false,
    }
}

/// Whether a condition ("you control no permanents of the chosen color") refers to a
/// choice that is undefined for its source (CR 607.5a).
pub fn condition_refers_to_undefined_choice(
    c: &crate::ability::Condition,
    ch: Option<&crate::object::Choices>,
) -> bool {
    use crate::ability::Condition;
    match c {
        Condition::Exists(f) => filter_refers_to_undefined_choice(f, ch),
        Condition::Not(x) => condition_refers_to_undefined_choice(x, ch),
        Condition::And(v) | Condition::Or(v) => v
            .iter()
            .any(|x| condition_refers_to_undefined_choice(x, ch)),
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
        Filter::ControlledBy(crate::ability::PlayerRel::Chosen) => ch
            .player
            .map(|p| Filter::ControllerMatches(Box::new(crate::ability::PlayerFilter::Is(p))))
            .unwrap_or_else(nothing),
        Filter::ManaValue(c, v) | Filter::Power(c, v) | Filter::Toughness(c, v)
            if matches!(**v, crate::ability::Value::Chosen) =>
        {
            let Some(n) = ch.number else {
                return nothing();
            };
            let v = Box::new(crate::ability::Value::Const(n));
            match f {
                Filter::ManaValue(..) => Filter::ManaValue(*c, v),
                Filter::Power(..) => Filter::Power(*c, v),
                _ => Filter::Toughness(*c, v),
            }
        }
        Filter::And(v) => Filter::And(v.iter().map(|x| bind_choices(x, ch)).collect()),
        Filter::Or(v) => Filter::Or(v.iter().map(|x| bind_choices(x, ch)).collect()),
        Filter::Not(x) => Filter::Not(Box::new(bind_choices(x, ch))),
        other => other.clone(),
    }
}

/// Marker text on a granted protection keyword: "This effect doesn't remove [this
/// object]" (CR 702.16n). Bound to the granting object's id when applied.
pub const DOESNT_REMOVE_SOURCE: &str = "doesn't remove source";

/// The bound form of [`DOESNT_REMOVE_SOURCE`] for a particular object.
pub fn doesnt_remove_marker(src: crate::types::ObjectId) -> SmolStr {
    SmolStr::new(format!("doesn't remove #{}", src.0))
}

/// Whether `name` is the name of a real card (any face) satisfying the restriction of a
/// "choose a [nonland/creature] card name" instruction (CR 201.4, 201.4a); a split
/// card's combined name isn't a name (CR 201.4b).
pub fn valid_card_name(name: &str, filter: Option<&str>) -> bool {
    if name.is_empty() || name.contains("//") {
        return false;
    }
    let Some(c) = mtg_data::cards().by_name(name) else {
        return false;
    };
    if !c.is_playable_card() {
        return false;
    }
    let faces = c.faces();
    let type_line = faces
        .iter()
        .find(|f| f.name.eq_ignore_ascii_case(name))
        .and_then(|f| f.type_line.clone())
        .or_else(|| c.type_line.clone())
        .unwrap_or_default();
    // "nonland", "creature", "noncreature, nonland", "nonbasic land": each word is a
    // (super)type the card must have, or with "non", must not have.
    let type_line = type_line.to_lowercase();
    let has = |w: &str| type_line.split_whitespace().any(|t| t == w);
    let matches = |f: &str| {
        f.split([',', ' '])
            .filter(|w| !w.is_empty())
            .all(|w| match w.strip_prefix("non") {
                Some(t) => !has(t),
                None => has(w),
            })
    };
    // "not:basic land": a name other than a basic land card name.
    filter.is_none_or(|f| match f.strip_prefix("not:") {
        Some(inner) => !matches(inner),
        None => matches(f),
    })
}

/// Objects matching `filter` chosen at random ([`crate::ability::Sel::AtRandom`]): `count`
/// different ones (all of them if there are fewer), each equally likely; nobody chooses.
pub fn pick_at_random(
    g: &mut Game,
    filter: &crate::ability::Filter,
    count: &crate::ability::Value,
    ctx: &Ctx,
) -> Vec<ObjectId> {
    use rand::seq::SliceRandom;
    let n = g.eval_value(count, ctx).max(0) as usize;
    // CR 614.13a: objects entering the battlefield right now can't be chosen.
    let mut cands: Vec<ObjectId> = g
        .objects_matching(filter, ctx)
        .into_iter()
        .filter(|o| !g.entering.contains(o))
        .collect();
    cands.shuffle(&mut g.rng);
    cands.truncate(n);
    cands
}

/// `Effect::Custom` names for secret choices ("secretly choose an opponent", "each player
/// secretly chooses a number. Then those numbers are revealed."): the choices that follow
/// [`SECRET_NEXT`] are made without being shown to the other players, until
/// [`REVEAL_CHOICES`] (or a cost that reveals them) shows them to everyone. While it's
/// secret, only the player who made a choice can see it (`mtg-api` observations); the
/// choice is still the value the abilities that refer to it use (Emissary of Grudges's
/// rulings).
pub const SECRET_NEXT: &str = "choices: the next choices are secret";
/// "then those choices are revealed": every secret choice made for the source.
pub const REVEAL_CHOICES: &str = "choices: reveal the secret choices";
/// The costs "Reveal the player you chose" and "Reveal the creature type you chose": they
/// can be paid only if that choice was made for the source (Emissary of Grudges's
/// rulings: "If you don't have a chosen opponent ... you can't pay the cost").
pub const REVEAL_CHOSEN_PLAYER: &str = "choices: reveal the player you chose";
pub const REVEAL_CHOSEN_TYPE: &str = "choices: reveal the creature type you chose";
/// `Event::Custom` name reported for each secret choice revealed (the player who made it
/// and the object it was made for).
pub const REVEALED_CHOICE: &str = "choice revealed";

/// Whether `viewer` may see what `chooser` chose for `obj`: anyone, unless the choice is
/// still secret, which only the player who made it sees.
pub fn choice_visible(g: &Game, obj: ObjectId, chooser: PlayerId, viewer: PlayerId) -> bool {
    viewer == chooser || !g.obj(obj).choices.secret.contains(&chooser)
}

/// Records the objects `p` chose for the source while its choices are secret ("you and
/// target opponent each secretly choose a creature that player controls"): kept among
/// `p`'s own choices for the source, which only `p` sees until they're revealed.
pub fn record_secret_objects(g: &mut Game, p: PlayerId, picked: &[Entity], ctx: &Ctx) {
    let Some(src) = ctx.source else {
        return;
    };
    if !g.obj(src).choices.secret_next {
        return;
    }
    let objs: Vec<ObjectId> = picked.iter().filter_map(|e| e.object()).collect();
    let ch = &mut g.objects[src.0 as usize].choices;
    // Each secret choice of objects has its own `SECRET_NEXT` (`mark_secret`): a later
    // choice of objects for the source isn't secret.
    ch.secret_next = false;
    let i = match ch.by_player.iter().position(|(q, _)| *q == p) {
        Some(i) => i,
        None => {
            ch.by_player.push((p, Default::default()));
            ch.by_player.len() - 1
        }
    };
    let mine = &mut ch.by_player[i].1;
    mine.objects = objs;
    mine.made_during = ctx.stack_obj;
    if !ch.secret.contains(&p) {
        ch.secret.push(p);
    }
    g.dirty = true;
}

/// What a player chose ([`describe`]), with the objects they chose by name.
pub fn describe_in(g: &Game, c: &crate::object::Choices) -> Vec<String> {
    let mut out = describe(c);
    out.extend(c.objects.iter().map(|o| g.obj(*o).chars.name.to_string()));
    out
}

/// Reveals the secret choices made for `src`: they become public.
pub fn reveal_choices(g: &mut Game, src: ObjectId) {
    let o = &mut g.objects[src.0 as usize];
    let secret = std::mem::take(&mut o.choices.secret);
    o.choices.secret_next = false;
    for ch in o.linked_choices.values_mut() {
        ch.secret.clear();
        ch.secret_next = false;
    }
    for p in secret {
        g.log(|g| {
            let what = g
                .obj(src)
                .choices
                .by_player
                .iter()
                .find(|(q, _)| *q == p)
                .map(|(_, c)| describe_in(g, c).join(", "))
                .unwrap_or_default();
            format!("{p} reveals their choice for {}: {what}", g.describe(src))
        });
        g.emit(crate::events::Event::Custom {
            name: SmolStr::new(REVEALED_CHOICE),
            player: Some(p),
            obj: Some(src),
            amount: 0,
        });
    }
}

/// The values in a set of choices, as words ("war", "red", "Goblin", "Player 1").
pub fn describe(c: &crate::object::Choices) -> Vec<String> {
    let mut out = Vec::new();
    if let Some(t) = &c.text {
        out.push(t.to_string());
    }
    if let Some(x) = c.color {
        if c.text.as_deref() != Some(x.word()) {
            out.push(x.word().to_string());
        }
    }
    if let Some(s) = &c.colors {
        out.push(s.iter().map(|c| c.word()).collect::<Vec<_>>().join(" and "));
    }
    for t in [&c.creature_type, &c.basic_land_type].into_iter().flatten() {
        if c.text.as_deref() != Some(t.as_str()) {
            out.push(t.to_string());
        }
    }
    if let Some(t) = c.card_type {
        if c.text.as_deref() != Some(t.word()) {
            out.push(t.word().to_string());
        }
    }
    if let Some(n) = &c.card_name {
        out.push(n.to_string());
    }
    if let Some(n) = c.number {
        out.push(n.to_string());
    }
    if let Some(p) = c.player {
        out.push(format!("{p}"));
    }
    for p in &c.players {
        out.push(format!("{p}"));
    }
    out
}

/// Secret choices and the costs that reveal them.
pub struct ChoiceRules;

impl crate::kw::KeywordRules for ChoiceRules {
    fn kinds(&self) -> &'static [crate::keywords::KeywordKind] {
        &[]
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        let Some(src) = ctx.source else {
            return matches!(
                name,
                SECRET_NEXT | REVEAL_CHOICES | REVEAL_CHOSEN_PLAYER | REVEAL_CHOSEN_TYPE
            );
        };
        match name {
            SECRET_NEXT => g.objects[src.0 as usize].choices.secret_next = true,
            REVEAL_CHOICES | REVEAL_CHOSEN_PLAYER | REVEAL_CHOSEN_TYPE => reveal_choices(g, src),
            _ => return false,
        }
        true
    }

    fn custom_effect_possible(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<bool> {
        // Only the player who made the choice can reveal it: a player who gained control
        // of the permanent made no choice for it (A Killer Among Us's and Stalking
        // Leonin's rulings).
        let mine = ctx.source.and_then(|s| {
            g.obj(s)
                .choices
                .by_player
                .iter()
                .find(|(q, _)| *q == ctx.controller)
                .map(|(_, c)| c)
        });
        match name {
            REVEAL_CHOSEN_PLAYER => Some(mine.is_some_and(|c| c.player.is_some())),
            REVEAL_CHOSEN_TYPE => Some(mine.is_some_and(|c| c.creature_type.is_some())),
            _ => None,
        }
    }
}

inventory::submit! { crate::kw::KeywordRegistration(&ChoiceRules) }

/// `Value::Custom` names: the highest and the lowest of the numbers players chose for the
/// source ("each player with the highest number loses that much life", "each player who
/// didn't choose the lowest number"), and the number the player performing the
/// instruction chose ("loses life equal to the number of items they revealed").
pub const HIGHEST_NUMBER: &str = "choices: the highest number chosen";
pub const LOWEST_NUMBER: &str = "choices: the lowest number chosen";
pub const OWN_NUMBER: &str = "choices: the number they chose";
/// `Condition::Custom`: the numbers chosen for the source are all the same ("If they
/// match").
pub const NUMBERS_MATCH: &str = "choices: the chosen numbers match";

/// The numbers players chose for the source, by player: during the resolution of a
/// spell or ability that had numbers chosen, only those chosen in it ("If they match":
/// the numbers you and the target opponent just chose).
fn numbers_chosen(g: &Game, ctx: &Ctx) -> Vec<(PlayerId, i64)> {
    g.source_choices(ctx)
        .map(|ch| {
            let now = ctx.stack_obj.filter(|s| {
                ch.by_player
                    .iter()
                    .any(|(_, c)| c.number.is_some() && c.made_during == Some(*s))
            });
            ch.by_player
                .iter()
                .filter(|(_, c)| now.is_none() || c.made_during == now)
                .filter_map(|(p, c)| c.number.map(|n| (*p, n as i64)))
                .collect()
        })
        .unwrap_or_default()
}

/// The number `p` chose for the source, if any.
pub fn number_chosen_by(g: &Game, ctx: &Ctx, p: PlayerId) -> Option<i64> {
    numbers_chosen(g, ctx)
        .into_iter()
        .find(|(q, _)| *q == p)
        .map(|(_, n)| n)
}

/// The values and conditions about the numbers chosen.
pub struct ChosenNumbers;

impl crate::kw::KeywordRules for ChosenNumbers {
    fn kinds(&self) -> &'static [crate::keywords::KeywordKind] {
        &[]
    }

    fn custom_value(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<i64> {
        let nums = || numbers_chosen(g, ctx).into_iter().map(|(_, n)| n);
        match name {
            HIGHEST_NUMBER => Some(nums().max().unwrap_or(0)),
            LOWEST_NUMBER => Some(nums().min().unwrap_or(0)),
            OWN_NUMBER => Some(number_chosen_by(g, ctx, ctx.controller).unwrap_or(0)),
            _ => None,
        }
    }

    fn custom_condition(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<bool> {
        (name == NUMBERS_MATCH).then(|| {
            let n: Vec<i64> = numbers_chosen(g, ctx).into_iter().map(|(_, n)| n).collect();
            n.len() >= 2 && n.iter().all(|x| *x == n[0])
        })
    }
}

inventory::submit! { crate::kw::KeywordRegistration(&ChosenNumbers) }

/// `StaticEffect::Custom`: "As ~ becomes attached to a creature, choose a color." The
/// choice is made each time the permanent becomes attached to a creature, before anything
/// else happens (it's made as the attachment is done, not by a triggered ability); the
/// abilities that refer to "the last chosen color" see the most recent one.
pub const AS_ATTACHED_CHOOSE_COLOR: &str = "choices: as attached to a creature, choose a color";

/// The choices made as a permanent becomes attached.
pub struct AttachChoices;

impl crate::kw::KeywordRules for AttachChoices {
    fn kinds(&self) -> &'static [crate::keywords::KeywordKind] {
        &[]
    }

    fn on_event(&self, g: &mut Game, ev: &crate::events::Event) {
        let crate::events::Event::Attached {
            obj,
            to: Entity::Object(host),
        } = ev
        else {
            return;
        };
        let (obj, host) = (*obj, *host);
        if !g.is_live(obj) || !g.is_live(host) || !g.obj(host).chars.is_creature() {
            return;
        }
        let has = g.obj(obj).chars.abilities.iter().any(|a| {
            matches!(&a.kind, crate::ability::AbilityKind::Static(s)
                if matches!(&s.effect, crate::ability::StaticEffect::Custom(n) if n == AS_ATTACHED_CHOOSE_COLOR))
        });
        if !has {
            return;
        }
        let p = g.obj(obj).controller;
        let mut ctx = Ctx::new(Some(obj), p);
        make_choice(g, p, &ChoiceKind::Color, &mut ctx);
    }
}

inventory::submit! { crate::kw::KeywordRegistration(&AttachChoices) }

/// `Filter::Custom`: has the first type chosen for the source ("basic lands of the first
/// chosen type", "each land of the first chosen type").
pub const FIRST_CHOSEN_TYPE: &str = "choices: of the first chosen type";
/// `Modification::Custom` (layer 4): "[lands] are / become the second chosen type": the
/// second land type chosen for the source replaces their land types (CR 305.7).
pub const SECOND_CHOSEN_TYPE: &str = "choices: is the second chosen type";
/// `Modification::Custom` (layer 6): "have each of the chosen abilities".
pub const CHOSEN_ABILITIES: &str = "choices: has each of the chosen abilities";

/// The `i`th word chosen for the source.
fn nth_chosen(g: &Game, ctx: &Ctx, i: usize) -> Option<SmolStr> {
    g.source_choices(ctx).and_then(|ch| ch.list.get(i).cloned())
}

/// Readers of the ordered list of choices.
pub struct ChosenList;

impl crate::kw::KeywordRules for ChosenList {
    fn kinds(&self) -> &'static [crate::keywords::KeywordKind] {
        &[]
    }

    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, ctx: &Ctx) -> Option<bool> {
        (name == FIRST_CHOSEN_TYPE)
            .then(|| nth_chosen(g, ctx, 0).is_some_and(|t| g.obj(id).chars.has_subtype(&t)))
    }

    fn custom_modification(
        &self,
        g: &Game,
        name: &str,
        chars: &mut crate::object::Characteristics,
        ctx: &Ctx,
        _target: ObjectId,
    ) -> bool {
        match name {
            SECOND_CHOSEN_TYPE => {
                // CR 305.7: the new land type replaces the old ones (and with them the
                // abilities they gave).
                if let Some(t) = nth_chosen(g, ctx, 1).filter(|t| is_land_type(t)) {
                    chars
                        .subtypes
                        .retain(|s| !subtype_lists().land.contains(s.as_str()));
                    chars.subtypes.push(t);
                    chars.abilities.clear();
                }
                true
            }
            CHOSEN_ABILITIES => {
                let list = g
                    .source_choices(ctx)
                    .map(|c| c.list.clone())
                    .unwrap_or_default();
                for w in list {
                    if let Some(k) = crate::keywords::KeywordKind::from_name(&w) {
                        let kw = crate::keywords::Keyword::new(k);
                        chars.abilities.push(crate::ability::AbilityDef::new(
                            crate::ability::AbilityKind::Keyword(kw),
                            k.name(),
                        ));
                    }
                }
                true
            }
            _ => false,
        }
    }
}

inventory::submit! { crate::kw::KeywordRegistration(&ChosenList) }

/// `Value::Custom`: how many of the colors chosen for the source the triggering object
/// is ("if it's at least one of the chosen colors, you gain 1 life for each of the chosen
/// colors it is", Tablet of the Guilds).
pub const CHOSEN_COLORS_IT_IS: &str = "choices: the chosen colors it is";

pub struct ChosenColorsCount;

impl crate::kw::KeywordRules for ChosenColorsCount {
    fn kinds(&self) -> &'static [crate::keywords::KeywordKind] {
        &[]
    }

    fn custom_value(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<i64> {
        if name != CHOSEN_COLORS_IT_IS {
            return None;
        }
        let chosen = g.source_choices(ctx).and_then(|c| c.colors)?;
        let it = ctx.event.as_ref().and_then(|e| e.spell.or(e.object))?;
        let colors = g.obj(it).chars.colors;
        Some(chosen.iter().filter(|c| colors.contains(*c)).count() as i64)
    }
}

inventory::submit! { crate::kw::KeywordRegistration(&ChosenColorsCount) }

/// `Filter::Custom`: has one of the names chosen for the source ("spells with the chosen
/// names", "all creatures with a name chosen for ~"): the names in its ordered list of
/// choices (each choice adds one, Day of the Moon's chapters accumulate).
pub const CHOSEN_NAMES: &str = "choices: has one of the chosen names";

pub struct ChosenNames;

impl crate::kw::KeywordRules for ChosenNames {
    fn kinds(&self) -> &'static [crate::keywords::KeywordKind] {
        &[]
    }

    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, ctx: &Ctx) -> Option<bool> {
        if name != CHOSEN_NAMES {
            return None;
        }
        let Some(ch) = g.source_choices(ctx) else {
            return Some(false);
        };
        let n = &g.obj(id).chars.name;
        Some(!n.is_empty() && ch.list.iter().any(|x| x.eq_ignore_ascii_case(n)))
    }
}

inventory::submit! { crate::kw::KeywordRegistration(&ChosenNames) }

/// `Filter::Custom`: "it's only the color its controller chose" (Selective Obliteration):
/// exactly that one color, the one its controller chose for the source.
pub const ONLY_COLOR_ITS_CONTROLLER_CHOSE: &str = "choices: only the color its controller chose";

pub struct ControllerChoice;

impl crate::kw::KeywordRules for ControllerChoice {
    fn kinds(&self) -> &'static [crate::keywords::KeywordKind] {
        &[]
    }

    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, ctx: &Ctx) -> Option<bool> {
        if name != ONLY_COLOR_ITS_CONTROLLER_CHOSE {
            return None;
        }
        let o = g.obj(id);
        let chosen = g.source_choices(ctx).and_then(|ch| {
            ch.by_player
                .iter()
                .find(|(p, _)| *p == o.controller)
                .and_then(|(_, c)| c.color)
        });
        Some(chosen.is_some_and(|c| o.chars.colors == ColorSet::single(c)))
    }
}

inventory::submit! { crate::kw::KeywordRegistration(&ControllerChoice) }

/// `Filter::Custom`: of the card type the word chosen for the source names ("artifact",
/// "creature", "land", "non-Aura enchantment": an enchantment that isn't an Aura).
pub const OF_THE_CHOSEN_TYPE_WORD: &str = "choices: of the chosen card type word";

pub struct ChosenTypeWord;

impl crate::kw::KeywordRules for ChosenTypeWord {
    fn kinds(&self) -> &'static [crate::keywords::KeywordKind] {
        &[]
    }

    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, ctx: &Ctx) -> Option<bool> {
        if name != OF_THE_CHOSEN_TYPE_WORD {
            return None;
        }
        let Some(w) = g.source_choices(ctx).and_then(|c| c.text.clone()) else {
            return Some(false);
        };
        let c = &g.obj(id).chars;
        Some(match w.to_lowercase().as_str() {
            "non-aura enchantment" => c.is(CardType::Enchantment) && !c.has_subtype("Aura"),
            other => CardType::from_word(other).is_some_and(|t| c.is(t)),
        })
    }
}

inventory::submit! { crate::kw::KeywordRegistration(&ChosenTypeWord) }

/// `Filter::Custom`: has the name the player performing the instruction (`ctx.controller`)
/// chose for the source ("If the card a player revealed has the name they chose").
pub const NAME_THEY_CHOSE: &str = "choices: has the name they chose";

pub struct NameTheyChose;

impl crate::kw::KeywordRules for NameTheyChose {
    fn kinds(&self) -> &'static [crate::keywords::KeywordKind] {
        &[]
    }

    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, ctx: &Ctx) -> Option<bool> {
        if name != NAME_THEY_CHOSE {
            return None;
        }
        let chosen = g.source_choices(ctx).and_then(|ch| {
            ch.by_player
                .iter()
                .find(|(p, _)| *p == ctx.controller)
                .and_then(|(_, c)| c.card_name.clone())
        });
        Some(chosen.is_some_and(|n| !n.is_empty() && g.obj(id).chars.has_name(&n)))
    }
}

inventory::submit! { crate::kw::KeywordRegistration(&NameTheyChose) }

/// `Effect::Custom` (part of a cost): the objects just beheld ("choose a creature you
/// control or reveal a creature card from your hand", `vars::IT`) are remembered for the
/// spell (`Choices::objects`).
pub const REMEMBER_CHOSEN_OBJECTS: &str = "choices: remember the objects chosen as it was cast";
/// `Value::Custom`: the power of the object chosen or revealed as the spell was cast: its
/// current power, or, if it has left its zone since, its last known power (Monstrous
/// Emergence's rulings).
pub const CHOSEN_OBJECT_POWER: &str = "choices: the power of the chosen object";

pub struct ChosenObjects;

impl crate::kw::KeywordRules for ChosenObjects {
    fn kinds(&self) -> &'static [crate::keywords::KeywordKind] {
        &[]
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != REMEMBER_CHOSEN_OBJECTS {
            return false;
        }
        let objs: Vec<ObjectId> = ctx
            .vars
            .get(&crate::ability::vars::IT)
            .map(|v| v.iter().filter_map(|e| e.object()).collect())
            .unwrap_or_default();
        if let Some(src) = ctx.source {
            g.objects[src.0 as usize].choices.objects = objs;
        }
        true
    }

    fn custom_value(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<i64> {
        if name != CHOSEN_OBJECT_POWER {
            return None;
        }
        let src = ctx.source?;
        let o = g.obj(src).choices.objects.first().copied()?;
        // The same object if it's still where it was; otherwise its last known power.
        Some(g.obj(o).chars.power.unwrap_or(0) as i64)
    }
}

inventory::submit! { crate::kw::KeywordRegistration(&ChosenObjects) }

/// The variable holding objects chosen by an instruction, collected from every player
/// ("each other player chooses a nonland permanent ... that hasn't been chosen this way",
/// "choose a creature with that power"), for "the others" / "not chosen this way".
pub const CHOSEN_OBJECTS_VAR: crate::ability::Var = crate::ability::vars::USER + 30529;
/// `Effect::Custom`: "For each different power among creatures on the battlefield, choose
/// a creature with that power." (Celestial Judgment): the controller chooses one creature
/// for each power any creature has, collected in [`CHOSEN_OBJECTS_VAR`].
pub const FOR_EACH_DIFFERENT_POWER: &str =
    "choices: for each different power, choose a creature with that power";
/// `Filter::Custom` prefix: of the card type that is the `i`th word chosen for the source
/// ("of the first type" after "choose an order for artifacts, creatures, and lands").
pub const NTH_CHOSEN_CARD_TYPE: &str = "choices: of the chosen card type #";

/// The filter name for the `i`th chosen card type.
pub fn nth_chosen_card_type(i: usize) -> String {
    format!("{NTH_CHOSEN_CARD_TYPE}{i}")
}

pub struct ObjectChoices;

impl crate::kw::KeywordRules for ObjectChoices {
    fn kinds(&self) -> &'static [crate::keywords::KeywordKind] {
        &[]
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != FOR_EACH_DIFFERENT_POWER {
            return false;
        }
        let creatures: Vec<ObjectId> = g
            .permanents()
            .filter(|o| o.chars.is_creature())
            .map(|o| o.id)
            .collect();
        let mut powers: Vec<i32> = creatures
            .iter()
            .map(|c| g.obj(*c).chars.power.unwrap_or(0))
            .collect();
        powers.sort_unstable();
        powers.dedup();
        let mut chosen = Vec::new();
        for pw in powers {
            let cands: Vec<ObjectId> = creatures
                .iter()
                .copied()
                .filter(|c| g.is_live(*c) && g.obj(*c).chars.power.unwrap_or(0) == pw)
                .collect();
            let pick = g
                .ask_objects(
                    ctx.controller,
                    ctx.source,
                    &format!("Choose a creature with power {pw}"),
                    cands.clone(),
                    1,
                    1,
                )
                .first()
                .copied()
                .or(cands.first().copied());
            if let Some(c) = pick {
                chosen.push(Entity::Object(c));
            }
        }
        ctx.set_var(CHOSEN_OBJECTS_VAR, chosen);
        true
    }

    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, ctx: &Ctx) -> Option<bool> {
        let i: usize = name.strip_prefix(NTH_CHOSEN_CARD_TYPE)?.parse().ok()?;
        let w = g.source_choices(ctx).and_then(|c| c.list.get(i).cloned());
        Some(
            w.and_then(|w| CardType::from_word(&w))
                .is_some_and(|t| g.obj(id).chars.is(t)),
        )
    }
}

inventory::submit! { crate::kw::KeywordRegistration(&ObjectChoices) }
