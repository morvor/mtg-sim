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
    }
    // CR 607.2d: the choice belongs to the abilities linked to the one that made it.
    let made = g.obj(src).choices.clone();
    let o = &mut g.objects[src.0 as usize];
    let entry = o.linked_choices.entry(ctx.link).or_default();
    copy_choice(kind, &made, entry);
    // Each player's own choice ("each player chooses war or peace"), kept for the
    // object and for the linked abilities.
    record_for_player(&mut o.choices, p, kind, &made);
    record_for_player(entry, p, kind, &made);
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
    }
}

/// Records what `p` chose (the value `kind` chose, in `made`) among `ch`'s per-player
/// choices.
fn record_for_player(
    ch: &mut crate::object::Choices,
    p: PlayerId,
    kind: &ChoiceKind,
    made: &crate::object::Choices,
) {
    let i = match ch.by_player.iter().position(|(q, _)| *q == p) {
        Some(i) => i,
        None => {
            ch.by_player.push((p, Default::default()));
            ch.by_player.len() - 1
        }
    };
    copy_choice(kind, made, &mut ch.by_player[i].1);
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
        || c.creature_type.as_ref().is_some_and(|t| t.to_lowercase() == w)
        || c.basic_land_type.as_ref().is_some_and(|t| t.to_lowercase() == w)
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
            .map(|p| {
                Filter::ControllerMatches(Box::new(crate::ability::PlayerFilter::Is(p)))
            })
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
                .map(|(_, c)| describe(c).join(", "))
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
