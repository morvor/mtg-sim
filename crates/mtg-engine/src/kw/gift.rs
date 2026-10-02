//! CR 702.174 Gift. "Gift a [something]" represents two abilities (CR 702.174a):
//!
//! * "As an additional cost to cast this spell, you may choose an opponent" — announced as
//!   the spell is cast ([`KeywordRules::announce_choices`], asked as the optional cost
//!   [`GIFT`]). A spell whose controller declared the intention to pay it had its gift
//!   promised (CR 702.174k): [`GIFT`] and the chosen player ([`promised_to_tag`]) are
//!   recorded in its `CastInfo::paid`, which "if the gift was promised" checks
//!   (`Condition::CostPaid`), on the spell and on the permanent it becomes (CR 400.7d).
//!   Targets of a part of the spell that has its effect only if the gift was promised are
//!   chosen only if it was (CR 702.174m; see `TargetSpec::condition`).
//! * On a permanent, "When this permanent enters, if its gift cost was paid, [effect]"; on
//!   an instant or sorcery spell, "If this spell's gift cost was paid, [effect]"
//!   (CR 702.174b). The oracle compiler puts this ability where the keyword is printed
//!   (see `oracle/patterns/k702_168_177.rs`): the gift keyword comes first, so an instant
//!   or sorcery's gift effect happens before its other spell abilities (CR 702.174j). The
//!   effect ([`give_effect`]) is performed by [`KeywordRules::custom_effect`] for the
//!   player the gift was promised to (CR 702.174d–i).
//!
//! A player "gives a gift" as their instant or sorcery spell whose gift cost was paid
//! resolves, or as the gift triggered ability of their permanent resolves (CR 702.174c):
//! the gift effect reports a [`GAVE_GIFT`] event then.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::decision::{Answer, Decision};
use crate::eval::Ctx;
use crate::events::Event;
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::types::*;
use smol_str::SmolStr;

/// The optional cost asked for, and the name recorded in `CastInfo::paid`, when a spell's
/// gift is promised.
pub const GIFT: &str = "gift";
/// `Event::Custom` name: a player (`player`) gave a gift (CR 702.174c).
pub const GAVE_GIFT: &str = "gave a gift";
/// Prefix of the `Effect::Custom` that gives the gift named after it ([`give_effect`]).
const GIVE: &str = "gift:give:";
/// Prefix of the name recorded in `CastInfo::paid` for the player the gift was promised
/// to.
const PROMISED_TO: &str = "gift promised to ";

/// What a gift is: the [something] of "Gift a [something]" (CR 702.174d–i).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GiftKind {
    Food,
    Card,
    TappedFish,
    ExtraTurn,
    Treasure,
    Octopus,
}

impl GiftKind {
    const ALL: [GiftKind; 6] = [
        GiftKind::Food,
        GiftKind::Card,
        GiftKind::TappedFish,
        GiftKind::ExtraTurn,
        GiftKind::Treasure,
        GiftKind::Octopus,
    ];

    /// The words after "Gift", lowercase.
    pub fn words(self) -> &'static str {
        match self {
            GiftKind::Food => "a food",
            GiftKind::Card => "a card",
            GiftKind::TappedFish => "a tapped fish",
            GiftKind::ExtraTurn => "an extra turn",
            GiftKind::Treasure => "a treasure",
            GiftKind::Octopus => "an octopus",
        }
    }

    /// The gift named by "Gift [words]".
    pub fn parse(words: &str) -> Option<GiftKind> {
        let w = words.trim().to_lowercase();
        GiftKind::ALL.into_iter().find(|k| k.words() == w)
    }
}

/// The name recorded in `CastInfo::paid` for the player a gift was promised to.
pub fn promised_to_tag(p: PlayerId) -> SmolStr {
    SmolStr::new(format!("{PROMISED_TO}{}", p.0))
}

/// The player the gift of the spell (or of the spell the permanent was) cast with `paid`
/// was promised to.
pub fn promised_to(paid: &[SmolStr]) -> Option<PlayerId> {
    paid.iter()
        .find_map(|t| t.strip_prefix(PROMISED_TO))
        .and_then(|n| n.parse::<u8>().ok())
        .map(PlayerId)
}

/// "If this spell's gift cost was paid" / "if its gift cost was paid".
pub fn promised() -> Condition {
    Condition::CostPaid(GIFT.into())
}

/// The effect that gives the gift `kind` to the player it was promised to.
pub fn give_effect(kind: GiftKind) -> Effect {
    Effect::Custom(SmolStr::new(format!("{GIVE}{}", kind.words())))
}

/// The second ability "Gift a [something]" represents (CR 702.174b): for an instant or
/// sorcery spell, a spell ability; for a permanent, an enters triggered ability.
pub fn gift_ability(kind: GiftKind, instant_or_sorcery: bool, text: &str) -> Ability {
    let give = give_effect(kind);
    if instant_or_sorcery {
        AbilityDef::new(
            AbilityKind::Spell(SpellAbility {
                body: Body::effect(Effect::If {
                    cond: promised(),
                    then: Box::new(give),
                    otherwise: Box::new(Effect::Noop),
                }),
            }),
            text,
        )
    } else {
        let mut t = TriggeredAbility::new(
            TriggerCond::EntersBattlefield(Filter::Source),
            Body::effect(give),
        );
        t.intervening_if = Some(promised());
        AbilityDef::new(AbilityKind::Triggered(t), text)
    }
}

fn creature_token(subtype: &str, colors: ColorSet, pt: i32) -> TokenSpec {
    TokenSpec {
        name: SmolStr::default(),
        colors,
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: vec![Subtype::new(subtype)],
        power: Some(pt),
        toughness: Some(pt),
        abilities: vec![],
        scryfall_name: None,
        pt_values: None,
    }
}

/// What the chosen player gets (CR 702.174d–i).
fn gift_for(kind: GiftKind, who: PlayerId) -> Effect {
    let who = PlayerRef::Player(who);
    let create = |spec: TokenSpec, tapped: bool| Effect::CreateToken {
        spec,
        count: Value::c(1),
        controller: who.clone(),
        tapped,
        attacking: false,
    };
    let blue = ColorSet::single(Color::Blue);
    match kind {
        // CR 702.174d: "The chosen player creates a Food token."
        GiftKind::Food => {
            crate::tokens_predefined::predefined("food").map_or(Effect::Noop, |s| create(s, false))
        }
        // CR 702.174e: "The chosen player draws a card."
        GiftKind::Card => Effect::Draw {
            who: who.clone(),
            n: Value::c(1),
        },
        // CR 702.174f: "The chosen player creates a tapped 1/1 blue Fish creature token."
        GiftKind::TappedFish => create(creature_token("Fish", blue, 1), true),
        // CR 702.174g: "The chosen player takes an extra turn after this one."
        GiftKind::ExtraTurn => Effect::ExtraTurn { who: who.clone() },
        // CR 702.174h: "The chosen player creates a Treasure token."
        GiftKind::Treasure => crate::tokens_predefined::predefined("treasure")
            .map_or(Effect::Noop, |s| create(s, false)),
        // CR 702.174i: "The chosen player creates an 8/8 blue Octopus creature token."
        GiftKind::Octopus => create(creature_token("Octopus", blue, 8), false),
    }
}

/// The opponents `p` may promise a gift to.
fn candidates(g: &Game, p: PlayerId) -> Vec<PlayerId> {
    g.opponents(p)
        .into_iter()
        .filter(|q| crate::multiplayer::range::player_in_range(g, p, *q))
        .collect()
}

pub struct Gift;

impl KeywordRules for Gift {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Gift]
    }

    /// "As an additional cost to cast this spell, you may choose an opponent."
    /// (CR 702.174a): declaring the intention to pay it promises the gift (CR 702.174k).
    fn announce_choices(
        &self,
        g: &mut Game,
        p: PlayerId,
        spell: ObjectId,
        _kw: &Keyword,
        paid: &mut Vec<SmolStr>,
    ) {
        let opps = candidates(g, p);
        if opps.is_empty() {
            return;
        }
        let promise = matches!(
            g.ask(
                p,
                Decision::OptionalCost {
                    source: spell,
                    name: GIFT.to_string(),
                    repeatable: false,
                },
            ),
            Answer::Bool(true)
        );
        if !promise {
            return;
        }
        let to = if opps.len() == 1 {
            opps[0]
        } else {
            g.ask_entities(
                p,
                Some(spell),
                "Choose an opponent to promise the gift to",
                opps.iter().map(|q| Entity::Player(*q)).collect(),
                1,
                1,
            )
            .first()
            .and_then(|e| e.player())
            .filter(|q| opps.contains(q))
            .unwrap_or(opps[0])
        };
        paid.push(GIFT.into());
        paid.push(promised_to_tag(to));
        g.log(|g| format!("{p} promises {to} a gift for {}", g.describe(spell)));
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        let Some(kind) = name.strip_prefix(GIVE).and_then(GiftKind::parse) else {
            return false;
        };
        let to = g.cast_info(ctx).and_then(|c| promised_to(&c.paid));
        let Some(to) = to else {
            return true;
        };
        if g.player(to).in_game() {
            let effect = gift_for(kind, to);
            g.exec(&effect, ctx);
        }
        // CR 702.174c: the spell's controller (or the permanent's) gave a gift.
        g.emit(Event::Custom {
            name: SmolStr::new(GAVE_GIFT),
            player: Some(ctx.controller),
            obj: ctx.source,
            amount: 0,
        });
        true
    }
}

inventory::submit! { KeywordRegistration(&Gift) }
