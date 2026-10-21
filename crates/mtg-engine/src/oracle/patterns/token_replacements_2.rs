//! Token-creation replacements II (CR 614.1a, 111.10): the tokens an event would create
//! are other tokens instead, keeping how many and how they're created (tapped, attacking).
//!
//! * "If you would create a Fish token, create a 3/3 blue Shark creature token instead."
//!   (Fisher's Talent).
//! * "If you would create a Clue, Food, or Treasure token, instead create one of each."
//!   (Academy Manufactor: that many of each).
//! * "If you would create one or more tokens, you may instead create that many 2/2 green
//!   Cat creature tokens with haste or that many 3/1 green Dog creature tokens with
//!   vigilance." (Jinnie Fay).
//! * "The first time you would create one or more tokens each turn, you may instead create
//!   that many tokens that are copies of enchanted permanent." (Moonlit Meditation,
//!   Mirrormind Crown), "... during each of your turns, you may instead choose a creature
//!   other than ~ and create that many tokens that are copies of that creature." (Esix).

use super::StaticPattern;
use crate::ability::*;
use crate::oracle::effects::{object_ref, Builder};
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;

fn replacement(
    event: ReplacementEvent,
    action: ReplacementAction,
    optional: bool,
) -> ReplacementDef {
    ReplacementDef {
        event,
        action,
        self_replacement: false,
        optional,
    }
}

fn static_ability(def: ReplacementDef, cond: Option<Condition>, text: &str) -> Vec<Ability> {
    let mut st = StaticAbility::new(StaticEffect::Replacement(def));
    st.condition = cond;
    vec![AbilityDef::new(AbilityKind::Static(st), text)]
}

/// "a 3/3 blue Shark creature token", "a Treasure token", "that many 2/2 green Cat
/// creature tokens with haste": one token's description.
fn token_desc(s: &str) -> Option<TokenSpec> {
    let s = end(s).trim();
    let (one, plural) = if let Some(r) = s.strip_prefix("that many ") {
        (r, true)
    } else if let Some(r) = s.strip_prefix("a ").or_else(|| s.strip_prefix("an ")) {
        (r, false)
    } else {
        return None;
    };
    let one = if plural {
        let (head, tail) = one.split_once(" tokens")?;
        format!("{head} token{tail}")
    } else {
        one.to_string()
    };
    let (word, tail) = split_word(&one);
    if tail.trim() == "token" {
        if let Some(spec) = crate::tokens::predefined(word) {
            return Some(spec);
        }
    }
    crate::oracle::effects::parse_token_description(&one)
}

/// "Clue", "Clue, Food, or Treasure", "Fish": the kinds of token named (subtypes).
fn token_kinds(s: &str) -> Option<Vec<String>> {
    let list = s.replace(", or ", ", ").replace(" or ", ", ");
    let words: Vec<String> = list.split(", ").map(|w| w.trim().to_string()).collect();
    if words.iter().any(|w| w.is_empty() || w.contains(' ')) {
        return None;
    }
    Some(words)
}

fn kinds_filter(kinds: &[String]) -> Option<Filter> {
    let fs: Vec<Filter> = kinds
        .iter()
        .map(|k| Some(Filter::Subtype(subtype_word(k)?)))
        .collect::<Option<_>>()?;
    Some(if fs.len() == 1 {
        fs.into_iter().next()?
    } else {
        Filter::Or(fs)
    })
}

/// "if you would create a [kind] token, create [token] instead", "..., instead create
/// [token]", "if you would create a [kind], [kind], or [kind] token, instead create one of
/// each", "if you would create one or more tokens, you may instead create that many [A]
/// or that many [B]".
fn would_create_instead(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let l = end(l);
    if let Some(r) = l.strip_prefix("if you would create one or more tokens, ") {
        let (r, optional) = match r.strip_prefix("you may ") {
            Some(r) => (r, true),
            None => (r, false),
        };
        let r = r.strip_prefix("instead create ")?;
        let specs: Vec<TokenSpec> = r
            .split(" or that many ")
            .enumerate()
            .map(|(i, d)| {
                if i == 0 {
                    token_desc(d)
                } else {
                    token_desc(&format!("that many {d}"))
                }
            })
            .collect::<Option<_>>()?;
        if specs.len() < 2 || !r.starts_with("that many ") {
            return None;
        }
        return Some(static_ability(
            replacement(
                ReplacementEvent::CreateTokens(PlayerFilter::You),
                ReplacementAction::TokensInsteadOf { specs, each: false },
                optional,
            ),
            None,
            text,
        ));
    }
    let r = l
        .strip_prefix("if you would create a ")
        .or_else(|| l.strip_prefix("if you would create an "))?;
    let (kinds, rest) = r.split_once(" token, ")?;
    let kinds = token_kinds(kinds)?;
    let tokens = kinds_filter(&kinds)?;
    let event = ReplacementEvent::CreateTokensMatching {
        who: PlayerFilter::You,
        tokens,
    };
    if rest == "instead create one of each" {
        if kinds.len() < 2 {
            return None;
        }
        let specs: Vec<TokenSpec> = kinds
            .iter()
            .map(|k| crate::tokens::predefined(&k.to_lowercase()))
            .collect::<Option<_>>()?;
        return Some(static_ability(
            replacement(
                event,
                ReplacementAction::TokensInsteadOf { specs, each: true },
                false,
            ),
            None,
            text,
        ));
    }
    if kinds.len() != 1 {
        return None;
    }
    let desc = rest.strip_prefix("instead create ").or_else(|| {
        rest.strip_prefix("create ")
            .and_then(|r| r.strip_suffix(" instead"))
    })?;
    // "a Food token and a Treasure token" is another pattern's (those tokens plus).
    if desc.contains(" and ") {
        return None;
    }
    let spec = token_desc(desc)?;
    Some(static_ability(
        replacement(event, ReplacementAction::TokensInstead { spec }, false),
        None,
        text,
    ))
}

inventory::submit! { StaticPattern { name: "token replacements 2: if you would create [kind] tokens, other tokens instead", priority: 70, parse: would_create_instead } }

/// `Condition::Custom` name: the token-creation event being replaced is the first its
/// player would perform this turn (no token has been created under their control this
/// turn yet).
pub const FIRST_TOKENS: &str = "token replacements:first token creation this turn";

/// "[As long as ~ is attached to a creature, ]the first time you would create one or more
/// tokens [each turn | during each of your turns], you may instead create that many tokens
/// that are copies of [object]" / "... you may instead choose a creature other than ~ and
/// create that many tokens that are copies of that creature".
fn first_time_copies(l: &str, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let l = end(l);
    let (cond, l) = match l.strip_prefix("as long as ~ is attached to a creature, ") {
        Some(r) => (
            Some(Condition::SelMatches(
                Sel::This,
                Filter::Custom(crate::kw::attached_to_creature::ATTACHED_TO_A_CREATURE.into()),
            )),
            r,
        ),
        None => (None, l),
    };
    let r = l.strip_prefix("the first time you would create one or more tokens ")?;
    let (your_turn, r) = if let Some(r) = r.strip_prefix("each turn, ") {
        (false, r)
    } else if let Some(r) = r.strip_prefix("during each of your turns, ") {
        (true, r)
    } else {
        return None;
    };
    let r = r.strip_prefix("you may instead ")?;
    let action = if let Some(obj) = r.strip_prefix("create that many tokens that are copies of ") {
        let mut b = Builder::new(ctx);
        let (sel, tail) = object_ref(obj, &mut b)?;
        if !end(&tail).is_empty() || !b.targets.is_empty() {
            return None;
        }
        ReplacementAction::TokenCopiesInstead {
            of: Some(sel),
            choose: None,
        }
    } else {
        let r = r.strip_prefix("choose ")?;
        let (what, rest) = r.split_once(" and create that many tokens that are copies of ")?;
        let noun = what
            .strip_prefix("a ")
            .or_else(|| what.strip_prefix("an "))?;
        let (f, other) = match noun.strip_suffix(" other than ~") {
            Some(n) => (n, true),
            None => (noun, false),
        };
        let (f, plural, tail) = parse_object_phrase(f)?;
        if plural || !end(tail).is_empty() || !rest.starts_with("that ") {
            return None;
        }
        let f = if other {
            Filter::and(vec![f, Filter::Other])
        } else {
            f
        };
        ReplacementAction::TokenCopiesInstead {
            of: None,
            choose: Some(f),
        }
    };
    let mut first = Condition::Custom(FIRST_TOKENS.into());
    if your_turn {
        first = Condition::And(vec![first, Condition::YourTurn]);
    }
    Some(static_ability(
        replacement(
            ReplacementEvent::Where {
                event: Box::new(ReplacementEvent::CreateTokens(PlayerFilter::You)),
                cond: first,
            },
            action,
            true,
        ),
        cond,
        text,
    ))
}

inventory::submit! { StaticPattern { name: "token replacements 2: the first time you would create tokens, copies instead", priority: 70, parse: first_time_copies } }

// ---------------------------------------------------------------------------
// Token creation variants
// ---------------------------------------------------------------------------

/// Whether `e` creates tokens that join combat ("... tapped and attacking that player").
fn joins_combat(e: &Effect) -> bool {
    match e {
        Effect::TokensJoinCombat { .. } => true,
        Effect::Seq(v) => v.iter().any(joins_combat),
        _ => false,
    }
}

/// Whether `e` creates tokens (and nothing else but storing values).
fn creates_tokens(e: &Effect) -> bool {
    match e {
        Effect::CreateToken { .. } | Effect::CreateTokenWithPT { .. } => true,
        Effect::Seq(v) => v.iter().any(creates_tokens),
        _ => false,
    }
}

/// "create two 3/3 blue and red Elemental creature tokens with flying that attack that
/// opponent this turn if able" (Furygale Flocking, inside "for each opponent, ..."): the
/// tokens, then a requirement that they attack that player this turn (CR 508.1d).
fn create_that_attack_if_able(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let base = l
        .strip_suffix(" that attack that opponent this turn if able")
        .or_else(|| l.strip_suffix(" that attack that player this turn if able"))?;
    if !base.starts_with("create ") {
        return None;
    }
    let defender = b.it_player.clone();
    if !matches!(defender, PlayerRef::Iterated | PlayerRef::TriggerPlayer) {
        return None;
    }
    let saved = (b.targets.len(), b.it.clone());
    let e = crate::oracle::effects::parse_clause(base, b);
    let Some(e) = e.filter(creates_tokens) else {
        b.targets.truncate(saved.0);
        b.it = saved.1;
        return None;
    };
    b.it = Sel::Var(vars::CREATED);
    Some(Effect::seq(vec![
        e,
        Effect::AddRestriction {
            restriction: Restriction::MustAttackPlayer {
                attackers: Filter::In(Box::new(Sel::Var(vars::CREATED))),
                defender: PlayerFilter::Ref(Box::new(defender)),
            },
            duration: Duration::EndOfTurn,
        },
    ]))
}

inventory::submit! { super::EffectPattern { name: "token replacements 2: create tokens that attack that opponent this turn if able", priority: 85, parse: create_that_attack_if_able } }

/// "They gain haste until end of turn." after "For each opponent, create [tokens] ...":
/// the tokens each pass created (performed in each pass, right after they're created).
fn they_after_each_player_creation(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    // The loop: "for each opponent, ..." or "for each creature attacking you, ...".
    fn the_loop(e: &mut Effect) -> Option<&mut Box<Effect>> {
        match e {
            Effect::ForEachPlayer { effect, .. } | Effect::ForEach { effect, .. } => Some(effect),
            Effect::Seq(v) => v.iter_mut().rev().find_map(the_loop),
            _ => None,
        }
    }
    // "For each creature attacking you, create a token ..." gathers the tokens all the
    // passes created as the tokens just created: the instruction follows the loop.
    let gathered = matches!(prev, Effect::Seq(v)
        if matches!(v.last(), Some(Effect::Store { var, .. }) if *var == vars::CREATED)
            && v.iter().any(|e| matches!(e, Effect::ForEach { .. })));
    let creates = match the_loop(prev) {
        Some(effect) => creates_tokens(effect) || joins_combat(effect),
        None => false,
    };
    if !creates {
        return false;
    }
    let lower = l.to_lowercase();
    let text = if lower.starts_with("they ") {
        lower.clone()
    } else if let Some(r) = lower.strip_prefix("those tokens ") {
        format!("they {r}")
    } else if lower.contains(" those tokens") {
        // "Sacrifice those tokens at end of combat.", "At the beginning of your next end
        // step, sacrifice those tokens."
        lower.replacen(" those tokens", " them", 1)
    } else {
        return false;
    };
    let saved = (b.it.clone(), b.targets.len());
    b.it = Sel::Var(vars::CREATED);
    let e = crate::oracle::effects::parse_sentence(&text, b);
    b.it = saved.0;
    let Some(e) = e.filter(|_| b.targets.len() == saved.1) else {
        b.targets.truncate(saved.1);
        return false;
    };
    if !serde_json::to_string(&e)
        .is_ok_and(|j| j.contains(&format!("{{\"Var\":{}}}", vars::CREATED)))
    {
        return false;
    }
    if gathered {
        let old = std::mem::take(prev);
        *prev = Effect::seq(vec![old, e]);
        return true;
    }
    let Some(effect) = the_loop(prev) else {
        return false;
    };
    let body = std::mem::take(effect.as_mut());
    **effect = Effect::seq(vec![body, e]);
    true
}

inventory::submit! { super::FollowupPattern { name: "token replacements 2: they ... after creating tokens for each player", priority: 45, apply: they_after_each_player_creation } }

/// "Otherwise, create one of those tokens." after "If [condition], create three [tokens].
/// [They have ...]" (Brood Birthing): one token of that description.
fn otherwise_one_of_those(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    let lower = l.to_lowercase();
    if end(&lower) != "otherwise, create one of those tokens" {
        return false;
    }
    let Effect::If {
        then, otherwise, ..
    } = prev
    else {
        return false;
    };
    if !matches!(**otherwise, Effect::Noop) {
        return false;
    }
    let one = match then.as_ref() {
        Effect::CreateToken {
            spec,
            controller,
            tapped,
            attacking,
            ..
        } => Effect::CreateToken {
            spec: spec.clone(),
            count: Value::c(1),
            controller: controller.clone(),
            tapped: *tapped,
            attacking: *attacking,
        },
        _ => return false,
    };
    **otherwise = one;
    true
}

inventory::submit! { super::FollowupPattern { name: "token replacements 2: otherwise, create one of those tokens", priority: 45, apply: otherwise_one_of_those } }

/// "create a 1/1 white Human creature token that's tapped and attacking that player or a
/// planeswalker they control" (Adeline, Resplendent Cathar): the tokens attack that
/// player or a planeswalker that player controls, as their controller chooses (CR 508.4).
fn attacking_that_player_or_planeswalker(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let base = l
        .strip_suffix(" or a planeswalker they control")
        .or_else(|| l.strip_suffix(" or a planeswalker that player controls"))?;
    if !base.ends_with(" attacking that player") || !base.starts_with("create ") {
        return None;
    }
    let saved = (b.targets.len(), b.it.clone());
    let e = crate::oracle::effects::parse_clause(base, b);
    let Some(Effect::TokensJoinCombat {
        effect,
        attacking: Some(defender),
        blocking: None,
    }) = e
    else {
        b.targets.truncate(saved.0);
        b.it = saved.1;
        return None;
    };
    Some(Effect::TokensJoinCombat {
        effect: Box::new(Effect::seq(vec![
            Effect::Custom(crate::kw::token_creation_rules::OR_THEIR_PLANESWALKERS.into()),
            *effect,
        ])),
        attacking: Some(defender),
        blocking: None,
    })
}

inventory::submit! { super::EffectPattern { name: "token replacements 2: tokens attacking that player or a planeswalker they control", priority: 85, parse: attacking_that_player_or_planeswalker } }

/// The token specs `e` creates.
fn created_specs(e: &mut Effect) -> Vec<&mut TokenSpec> {
    match e {
        Effect::CreateToken { spec, .. } | Effect::CreateTokenWithPT { spec, .. } => vec![spec],
        Effect::Seq(v) => v.iter_mut().flat_map(created_specs).collect(),
        _ => vec![],
    }
}

/// "create a 1/1 creature token of the chosen color and type" (Sarpadian Empires, Vol.
/// VII; Volrath's Laboratory), "create an X/X creature token of the chosen color and type,
/// where X is ..." (Riptide Replicator), "... of the chosen color", "... of the chosen
/// type": the token's color and creature type are those chosen for the source (CR 607.2d).
fn token_of_the_chosen(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    if !l.starts_with("create ") {
        return None;
    }
    let (phrase, placeholders): (&str, &[&str]) = [
        (
            " of the chosen color and type",
            &[crate::tokens::CHOSEN_COLOR, crate::tokens::CHOSEN_TYPE][..],
        ),
        (" of the chosen color", &[crate::tokens::CHOSEN_COLOR][..]),
        (" of the chosen type", &[crate::tokens::CHOSEN_TYPE][..]),
    ]
    .into_iter()
    .find(|(p, _)| l.contains(&format!("token{p}")) || l.contains(&format!("tokens{p}")))?;
    let text = l.replacen(phrase, "", 1);
    let saved = (b.targets.len(), b.it.clone());
    let mut e = crate::oracle::effects::parse_clause(&text, b)?;
    let specs = created_specs(&mut e);
    if specs.len() != 1 {
        b.targets.truncate(saved.0);
        b.it = saved.1;
        return None;
    }
    for spec in specs {
        // Only a token whose color and creature type the choice gives.
        if placeholders.contains(&crate::tokens::CHOSEN_COLOR) && !spec.colors.is_colorless() {
            return None;
        }
        if placeholders.contains(&crate::tokens::CHOSEN_TYPE)
            && spec.card_types.contains(&crate::types::CardType::Creature)
            && !spec.subtypes.is_empty()
        {
            return None;
        }
        for p in placeholders {
            spec.subtypes.push((*p).into());
        }
    }
    Some(e)
}

inventory::submit! { super::EffectPattern { name: "token replacements 2: a token of the chosen color and type", priority: 85, parse: token_of_the_chosen } }

/// "As ~ enters, choose white Citizen, blue Camarid, black Thrull, red Goblin, or green
/// Saproling." (Sarpadian Empires, Vol. VII): a color and a creature type chosen together
/// (CR 614.12, 607.2d).
fn choose_color_type_pairs(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let r = end(l).strip_prefix("as ~ enters, choose ")?;
    let list = r.replace(", or ", ", ");
    let mut words = Vec::new();
    for w in list.split(", ") {
        let (c, t) = w.split_once(' ')?;
        crate::types::Color::from_word(c)?;
        let t = subtype_word(t)?;
        words.push(format!("{c} {t}"));
    }
    if words.len() < 2 {
        return None;
    }
    Some(static_ability(
        replacement(
            ReplacementEvent::EntersBattlefield(Filter::Source),
            ReplacementAction::AsEnters(Box::new(Effect::Choose {
                who: PlayerRef::You,
                kind: ChoiceKind::OneOf(words),
            })),
            false,
        ),
        None,
        text,
    ))
}

inventory::submit! { StaticPattern { name: "token replacements 2: choose a color and creature type pair", priority: 70, parse: choose_color_type_pairs } }

/// "create a green Fungus Dinosaur creature token with base power and toughness each
/// equal to the total power of those creatures" (The Skullspore Nexus): the token's power
/// and toughness are determined as it's created and are part of its copiable values
/// (CR 111.3, 608.2h).
fn token_with_pt_equal_to(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    if !l.starts_with("create a ") && !l.starts_with("create an ") {
        return None;
    }
    // "with power equal to that card's power and toughness equal to that card's
    // toughness" (Soul Separator).
    let (head, value, separate) = match l
        .split_once(" token with base power and toughness each equal to ")
        .or_else(|| l.split_once(" token with power and toughness each equal to "))
    {
        Some((h, v)) => (h, v, None),
        None => {
            let (h, r) = l.split_once(" token with power equal to ")?;
            let (p, t) = r.split_once(" and toughness equal to ")?;
            (h, p, Some(t))
        }
    };
    let (_, desc) = head.split_once(' ')?;
    let desc = desc
        .strip_prefix("a ")
        .or_else(|| desc.strip_prefix("an "))?;
    if desc.contains('/') || !desc.contains(" creature") {
        return None;
    }
    let saved = (b.targets.len(), b.it.clone());
    let restore = |b: &mut Builder| {
        b.targets.truncate(saved.0);
        b.it = saved.1.clone();
    };
    let Some((v, tail)) = crate::oracle::statics::parse_value_phrase(value, b) else {
        restore(b);
        return None;
    };
    if !end(&tail).is_empty() {
        restore(b);
        return None;
    }
    let t = match separate {
        Some(t) => {
            let Some((t, tail)) = crate::oracle::statics::parse_value_phrase(t, b) else {
                restore(b);
                return None;
            };
            if !end(&tail).is_empty() {
                restore(b);
                return None;
            }
            t
        }
        None => v.clone(),
    };
    let Some(mut e) =
        crate::oracle::effects::parse_clause(&format!("create a 0/0 {desc} token"), b)
    else {
        restore(b);
        return None;
    };
    let Effect::CreateToken { spec, .. } = &mut e else {
        restore(b);
        return None;
    };
    spec.power = None;
    spec.toughness = None;
    spec.pt_values = Some(Box::new((v, t)));
    Some(e)
}

inventory::submit! { super::EffectPattern { name: "token replacements 2: a token with power and toughness each equal to", priority: 85, parse: token_with_pt_equal_to } }

/// All the tokens a run of creations made.
const ALL_CREATED: Var = vars::USER + 5110;
/// The token each pass of "for each of those tokens" is about.
const EACH_TOKEN: Var = vars::USER + 5111;

/// "For each of those tokens, you may put an Aura card from your hand and/or graveyard
/// onto the battlefield attached to it." after "create a 1/1 white Cat creature token
/// with lifelink, a 1/1 white Bird creature token with flying, and a 2/4 white Ox creature
/// token" (Liberated Livestock): the instruction for each token created, in turn.
fn for_each_of_those_tokens(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let lower = l.to_lowercase();
    let Some(r) = end(&lower).strip_prefix("for each of those tokens, ") else {
        return false;
    };
    let creations: Vec<Effect> = match &*prev {
        Effect::Seq(v)
            if !v.is_empty() && v.iter().all(|e| matches!(e, Effect::CreateToken { .. })) =>
        {
            v.clone()
        }
        Effect::CreateToken { .. } => vec![prev.clone()],
        _ => return false,
    };
    let saved = (b.it.clone(), b.targets.len());
    b.it = Sel::Var(EACH_TOKEN);
    let e = crate::oracle::effects::parse_sentence(r, b);
    b.it = saved.0;
    let Some(e) = e.filter(|_| b.targets.len() == saved.1) else {
        b.targets.truncate(saved.1);
        return false;
    };
    let mut v = vec![Effect::Store {
        var: ALL_CREATED,
        sel: Sel::None,
    }];
    for c in creations {
        v.push(c);
        v.push(Effect::Store {
            var: ALL_CREATED,
            sel: Sel::Union(vec![Sel::Var(ALL_CREATED), Sel::Var(vars::CREATED)]),
        });
    }
    v.push(Effect::ForEach {
        sel: Sel::Var(ALL_CREATED),
        var: EACH_TOKEN,
        effect: Box::new(e),
    });
    *prev = Effect::Seq(v);
    true
}

inventory::submit! { super::FollowupPattern { name: "token replacements 2: for each of those tokens, [instruction]", priority: 45, apply: for_each_of_those_tokens } }

/// "The token enters tapped and attacking that player." after creating a token (Echoing
/// Assault): it's created tapped and attacking the player the ability refers to
/// (CR 508.4).
fn token_enters_attacking_that_player(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let lower = l.to_lowercase();
    if !matches!(
        end(&lower),
        "the token enters tapped and attacking that player"
            | "it enters tapped and attacking that player"
            | "the tokens enter tapped and attacking that player"
    ) {
        return false;
    }
    let defender = b.it_player.clone();
    if !matches!(
        defender,
        PlayerRef::TriggerPlayer | PlayerRef::Iterated | PlayerRef::Target(_)
    ) {
        return false;
    }
    let ok = match prev {
        Effect::CreateToken {
            tapped, attacking, ..
        }
        | Effect::CreateTokenCopy {
            tapped, attacking, ..
        } => {
            *tapped = true;
            *attacking = true;
            true
        }
        _ => false,
    };
    if !ok {
        return false;
    }
    let inner = std::mem::take(prev);
    *prev = Effect::TokensJoinCombat {
        effect: Box::new(inner),
        attacking: Some(defender),
        blocking: None,
    };
    true
}

inventory::submit! { super::FollowupPattern { name: "token replacements 2: the token enters tapped and attacking that player", priority: 45, apply: token_enters_attacking_that_player } }

/// "opponent", "player", "opponent with three or more poison counters": the players a
/// "for each [players]" counts.
fn counted_players(s: &str) -> Option<PlayerFilter> {
    let (base, rest) = if let Some(r) = s.strip_prefix("opponent") {
        (PlayerFilter::Opponent, r)
    } else if let Some(r) = s.strip_prefix("player") {
        (PlayerFilter::Any, r)
    } else {
        return None;
    };
    let rest = rest.trim();
    if rest.is_empty() {
        return Some(base);
    }
    let r = rest.strip_prefix("with ")?;
    let (cmp, n, r) = super::counters_resources_counters::amount_cmp(r)?;
    let (kind, r) = crate::oracle::costs::counter_kind(r)?;
    if r.trim() != "counters" && r.trim() != "counter" {
        return None;
    }
    Some(PlayerFilter::And(vec![
        base,
        PlayerFilter::Counters(kind.as_str().into(), cmp, Box::new(n)),
    ]))
}

/// "Then for each opponent with three or more poison counters, you create another one of
/// those tokens." (Wurmquake): one more token of the same description for each such
/// player.
fn another_one_of_those_tokens(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    let lower = l.to_lowercase();
    let l = end(&lower);
    let l = l.strip_prefix("then ").unwrap_or(l);
    let Some(r) = l.strip_prefix("for each ") else {
        return false;
    };
    let Some((who, rest)) = r.split_once(", ") else {
        return false;
    };
    if !matches!(
        rest,
        "you create another one of those tokens" | "create another one of those tokens"
    ) {
        return false;
    }
    let Some(players) = counted_players(who) else {
        return false;
    };
    fn last_create(e: &Effect) -> Option<&Effect> {
        match e {
            Effect::CreateToken { .. } | Effect::CreateTokenWithPT { .. } => Some(e),
            Effect::Seq(v) => v.iter().rev().find_map(last_create),
            _ => None,
        }
    }
    let Some(one) = last_create(prev).cloned() else {
        return false;
    };
    let one = match one {
        Effect::CreateToken {
            spec,
            controller,
            tapped,
            attacking,
            ..
        } => Effect::CreateToken {
            spec,
            count: Value::c(1),
            controller,
            tapped,
            attacking,
        },
        Effect::CreateTokenWithPT {
            spec,
            power,
            toughness,
            controller,
            tapped,
            attacking,
            ..
        } => Effect::CreateTokenWithPT {
            spec,
            power,
            toughness,
            count: Value::c(1),
            controller,
            tapped,
            attacking,
        },
        _ => return false,
    };
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![
        old,
        Effect::Repeat {
            times: Value::CountPlayers(players),
            effect: Box::new(one),
        },
    ]);
    true
}

inventory::submit! { super::FollowupPattern { name: "token replacements 2: for each [player], another one of those tokens", priority: 45, apply: another_one_of_those_tokens } }

/// "create a token that's a copy of ~ tapped and attacking that player, except it isn't
/// legendary" (Shredder, Shadow Master), "create a tapped token that's a copy of the
/// exiled card attacking that opponent" (Mardu Siegebreaker): token copies created tapped
/// and attacking the player the instruction is about (CR 508.4).
fn token_copy_attacking_that_player(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let defender = b.it_player.clone();
    if !matches!(defender, PlayerRef::Iterated | PlayerRef::TriggerPlayer) {
        return None;
    }
    let (body, except) = match l.split_once(", except ") {
        Some((a, e)) => (a, Some(e)),
        None => (l, None),
    };
    let (head, of) = if let Some(r) = body.strip_prefix("create a tapped token that's a copy of ") {
        let of = ["attacking that opponent", "attacking that player"]
            .iter()
            .find_map(|p| r.strip_suffix(&format!(" {p}")))?;
        ("create a tapped and attacking token that's a copy of ", of)
    } else {
        let r = body.strip_prefix("create a token that's a copy of ")?;
        let of = [
            "that's tapped and attacking that player",
            "tapped and attacking that player",
            "that's tapped and attacking that opponent",
            "tapped and attacking that opponent",
        ]
        .iter()
        .find_map(|p| r.strip_suffix(&format!(" {p}")))?;
        ("create a tapped and attacking token that's a copy of ", of)
    };
    let text = match except {
        Some(e) => format!("{head}{of}, except {e}"),
        None => format!("{head}{of}"),
    };
    let saved = (b.targets.len(), b.it.clone());
    let Some(e) = super::tokens_copies_copy::token_copy_with_exceptions(&text, b) else {
        b.targets.truncate(saved.0);
        b.it = saved.1;
        return None;
    };
    Some(Effect::TokensJoinCombat {
        effect: Box::new(e),
        attacking: Some(defender),
        blocking: None,
    })
}

inventory::submit! { super::EffectPattern { name: "token replacements 2: token copy tapped and attacking that player", priority: 85, parse: token_copy_attacking_that_player } }

/// "If the token isn't a creature, it becomes a 2/2 Robot Villain creature in addition to
/// its other types." (Ultron, Artificial Malevolence): "the token" (and "it") is the token
/// just created.
fn if_the_token(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let lower = l.to_lowercase();
    let Some(r) = end(&lower).strip_prefix("if the token ") else {
        return false;
    };
    if super::tokens_copies_create::last_create(prev).is_none() {
        return false;
    }
    let saved = (b.it.clone(), b.targets.len());
    b.it = Sel::Var(vars::CREATED);
    let e = crate::oracle::effects::parse_sentence(&format!("if it {r}"), b);
    if e.is_none() || b.targets.len() != saved.1 {
        b.targets.truncate(saved.1);
        b.it = saved.0;
        return false;
    }
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![old, e.expect("checked")]);
    true
}

inventory::submit! { super::FollowupPattern { name: "token replacements 2: if the token ...", priority: 45, apply: if_the_token } }

/// "[creature] attacking you or a planeswalker you control" (Mirror Match): attacking you
/// or a planeswalker you control (CR 506.3, 508.1b).
fn attacking_you_or_your_planeswalker<'a>(t: &'a str, f: &Filter) -> Option<(Filter, &'a str)> {
    let r = t.strip_prefix("attacking you or a planeswalker you control")?;
    if !(r.is_empty() || r.starts_with([' ', ',', '.'])) {
        return None;
    }
    if !serde_json::to_string(f).is_ok_and(|s| s.contains("\"Creature\"")) {
        return None;
    }
    Some((
        Filter::Or(vec![
            Filter::AttackingPlayer(PlayerRel::You),
            Filter::AttackingObject(Box::new(Filter::and(vec![
                Filter::Type(crate::types::CardType::Planeswalker),
                Filter::ControlledBy(PlayerRel::You),
            ]))),
        ]),
        r,
    ))
}

inventory::submit! { super::FilterSuffixPattern { name: "token replacements 2: attacking you or a planeswalker you control", priority: 90, parse: attacking_you_or_your_planeswalker } }

/// The variable the last instruction of `e` stored chosen objects in ("for each opponent,
/// choose a creature that player controls").
fn last_stored_var(e: &Effect) -> Option<Var> {
    match e {
        Effect::Store { var, .. } => Some(*var),
        Effect::ForEachPlayer { effect, .. } => last_stored_var(effect),
        Effect::Seq(v) => v.last().and_then(last_stored_var),
        _ => None,
    }
}

/// "Create a token that's a copy of one of those creatures, except its power is equal to
/// the total power of those creatures, its toughness is equal to the total toughness of
/// those creatures, and it's a colorless Eldrazi creature." (Benthic Anomaly): one of the
/// chosen creatures is copied, with those exceptions (CR 707.9b, locked in as the token
/// is created).
fn copy_of_one_of_those_creatures(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    let lower = l.to_lowercase();
    let Some(exc) =
        end(&lower).strip_prefix("create a token that's a copy of one of those creatures, except ")
    else {
        return false;
    };
    let Some(var) = last_stored_var(prev) else {
        return false;
    };
    let those = Sel::Var(var);
    let mut power = None;
    let mut toughness = None;
    let mut mods = Vec::new();
    for part in exc.split(", ") {
        let part = part.strip_prefix("and ").unwrap_or(part);
        match part {
            "its power is equal to the total power of those creatures" => {
                power = Some(Value::PowerOf(Box::new(those.clone())))
            }
            "its toughness is equal to the total toughness of those creatures" => {
                toughness = Some(Value::ToughnessOf(Box::new(those.clone())))
            }
            _ => {
                let Some(r) = part.strip_prefix("it's a colorless ") else {
                    return false;
                };
                let Some(sub) = r.strip_suffix(" creature") else {
                    return false;
                };
                let Some(st) = subtype_word(sub) else {
                    return false;
                };
                mods.push(Modification::SetColors(crate::types::ColorSet::NONE));
                mods.push(Modification::SetTypes {
                    types: vec![crate::types::CardType::Creature],
                    subtypes: vec![st],
                });
            }
        }
    }
    if power.is_none() && toughness.is_none() && mods.is_empty() {
        return false;
    }
    if power.is_some() || toughness.is_some() {
        mods.insert(0, Modification::SetPT(power, toughness));
    }
    let create = Effect::CreateTokenCopy {
        of: Sel::Choose {
            chooser: PlayerRef::You,
            filter: Filter::In(Box::new(those)),
            count: Value::c(1),
            up_to: false,
            store: None,
        },
        count: Value::c(1),
        controller: PlayerRef::You,
        tapped: false,
        attacking: false,
        mods,
    };
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![old, create]);
    true
}

inventory::submit! { super::FollowupPattern { name: "token replacements 2: copy of one of those creatures, except", priority: 90, apply: copy_of_one_of_those_creatures } }
