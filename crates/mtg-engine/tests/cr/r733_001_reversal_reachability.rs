//! CR 733.1's exceptions: which fully supported cards can reach them, checked on every
//! test run.
//!
//! CR 733.1 ends: "Players may not reverse actions that moved cards to a library, moved
//! cards from a library to any zone other than the stack, caused a library to be shuffled,
//! or caused cards from a library to be revealed." The engine reverses an illegal action by
//! restoring a copy of the whole game taken before it began (`Game::roll_back`), then
//! performing those library actions again (`reversal::keep_library_actions`; register entry
//! `illegal-action-reversal-restores-libraries`). They matter only when one of those
//! library actions happens while an action is attempted
//! (casting a spell, activating an ability, paying a cost as an ability resolves, taking a
//! special action, declaring attackers or blockers that have costs) and the action still
//! turns out illegal afterwards.
//!
//! [`scan`] checks a card's compiled abilities against what the engine guarantees, [`reach`]
//! follows what the cards of a pool can cause together in the middle of a payment, and
//! `every_fully_supported_card_is_scanned_for_ways_to_reach_the_exceptions` runs both over
//! every fully supported card, naming the card and the ability of each way to reach the
//! difference. Each check mirrors the engine:
//!
//! * **(a) Costs** (`Game::pay_total_cost`): the engine pays the mana first (activating
//!   mana abilities), then checks that each other part can be paid (each on its own), then
//!   pays them in the order written, except that the parts that pay last
//!   (`casting::cost_part_pays_last`, CR 601.2h) come after all the others; a repeated part
//!   is expanded first (`expand_repeated`). A part that touches a library is safe when
//!   nothing paid after it can fail ([`part_can_fail`], checked against the engine by
//!   `the_cost_parts_the_scan_says_cant_fail_dont_fail_in_a_game`), when no other part also
//!   takes cards from a library (the parts aren't checked jointly), and when nothing after
//!   the whole payment can make the action illegal. That last holds for activation costs
//!   (`activate_inner` does nothing that can fail once the cost is paid) and for costs paid
//!   as an ability resolves (`Game::pay_cost`); a library part anywhere else is reported: a
//!   spell's cost is followed by the check that the spell still has the keywords that paid
//!   part of it (`cast_inner`), attack and block costs by the other creatures' and players'
//!   costs. One of several costs (`CostPart::OneOf`) pays the chosen one as a payment of
//!   its own, and so does a cost an effect has a player pay ("[player] may pay [cost]"):
//!   those are checked the same way, nested ([`check_payment`]). A cost modifier's parts
//!   come after an activated ability's own (`Game::ability_total_cost_with`), and an
//!   alternative cost replaces its parts (`activation_costs::with_alternative`): every
//!   activation cost that touches a library is checked again in a game with each modifier
//!   that adds parts to activation costs or offers an alternative to them
//!   ([`with_activation_modifiers`]).
//! * **(b) Mana abilities** (CR 605): they're activated while an action is attempted
//!   (CR 601.2g, 602.2b) and the action can still fail afterwards, so no mana ability
//!   (activated, triggered, or a delayed triggered mana ability an effect creates), and no
//!   special action that can be taken in the middle of a mana payment, may touch a library
//!   in its cost or in what it does right away. (What it creates to happen later, such as
//!   "When you spend this mana ..." or a reflexive "When you do, mill two cards", is a
//!   triggered ability that uses the stack and is dropped with an undone action.)
//! * **(c) Searching, shuffling, revealing** a library, or moving its cards to a public
//!   zone, as part of a cost: such a part must be paid after every other part (CR 601.2h),
//!   by `cost_part_pays_last` or by being the last part written.
//! * **(d) Other paths**: an ability activated from a library reveals the card (CR 602.2a)
//!   before the activation can still fail; a keyword's cost is checked where the abilities
//!   the keyword stands for pay it (`kw::derived`), and one they don't pay is a cost paid as
//!   a spell is cast or a special action is taken, like (a)'s other costs. Casting a card
//!   from the top of a library moves it to the stack, which 733.1 lets be reversed, and
//!   the next card isn't revealed until the spell is cast (CR 401.5):
//!   `a_card_cast_from_the_top_of_a_library_goes_back_when_the_cast_is_reversed`.
//! * **(e) Replacement effects and rules applied in the middle of a payment.** A
//!   replacement effect that does a library action (puts a card into a library instead,
//!   draws, mills, exiles library cards, ...) and applies to an event that can happen while
//!   an action is attempted makes that event a library action. Which events can happen
//!   there is computed over the whole pool, as a fixpoint ([`reach`]): those every payment
//!   can cause ([`event_reach`]: a permanent sacrificed, a card discarded, damage dealt to a
//!   player by a land tapped for mana, life paid, gained or lost, counters, tokens without
//!   abilities, mana), and those ([`Ev`]) something in the pool causes there: a cost or a
//!   mana ability ([`Now::events`]); a replacement effect that applies there (Pariah deals
//!   the damage a player would be dealt to a permanent instead, Nefarious Lich makes a
//!   player who can't exile cards for the damage lose the game); or a rule the engine
//!   applies there: `flush_events`, which ends every mana ability's activation, starts with
//!   `until::check_untils`, which returns the cards exiled "until [a permanent] leaves the
//!   battlefield" as soon as that permanent is sacrificed for mana (onto the battlefield, or
//!   into the library a card was exiled from, a library action itself). Prevention and
//!   regeneration shields an effect creates (`Effect::PreventDamage` with an instruction
//!   that follows, `Effect::RegenerateWith`) are replacement effects too. What follows a
//!   library action isn't followed further: that library action is already reported. The
//!   ways found are [`KNOWN_REACHABLE`], by card and ability, and the events the pool can
//!   cause in the middle of a payment are [`KNOWN_EVENTS`]; the register entry lists both,
//!   and a new one, or one no longer found, fails the scan. The `a_reversed_cast_keeps_`
//!   tests show each kind in a game, kept by the reversal. An event the scan doesn't follow ([`Ev::Unfollowed`]:
//!   a card cast, the game restarted, an unclassified keyword action) that can happen there
//!   is reported as "(e-assumption)" and can't be listed. A permanent that enters there
//!   wasn't cast there, so "if it was kicked" doesn't hold for it ([`not_cast`]). Two
//!   replacement effects apply only to a discard in the middle of a payment made outside
//!   casting and activation ([`Reach::DiscardOutsideCasting`]): Library of Leng's (a discard
//!   an effect makes) and Nephalia Academy's (a cost's discard too). An effect can make one
//!   there ([`Ev::DiscardByEffect`]: "As ~ enters, discard your hand" for a card an "until"
//!   exile returns); whether such a payment can do that before something in it fails is
//!   more than the scan follows, so both count as reaching the exceptions. Without such an
//!   effect, Nephalia Academy would need a cost's discard that something can fail after,
//!   which [`discard_pairs`] reports ([`KNOWN_DISCARD_REPLACEMENTS`]). Nothing else acts in
//!   the middle of a payment: triggered abilities wait (and are dropped with an undone
//!   action, CR 733.1), state-based actions aren't checked (`Game::settle` runs from the
//!   turn loop), and a permanent that phases in (`check_untils`) touches no library and
//!   causes no event a replacement effect watches.
//!
//! Rules implemented in code (`kw::KeywordRules` hooks) are found by the names cards turn
//! them on with (`StaticEffect::Custom`, `PlayerModification::Custom`,
//! `Modification::Custom`): [`custom_rule`] classifies each name, and an unclassified one
//! fails the scan. Ashiok, Wicked Manipulator's (paying life exiles cards from the top of a
//! library instead) is one of (e)'s ways: `a_reversed_cast_keeps_cards_exiled_instead_of_paying_life_for_mana`.
//! The hooks keywords use for themselves (convoke's and delve's payments, exert, gift,
//! miracle's reveal of a drawn card, morph's, foretell's, plot's and suspend's special
//! actions) touch no library while an action is attempted either; that was read from their
//! code, not checked by the scan. (The other custom names answer questions: filters,
//! values, conditions and restrictions; or trigger abilities, which use the stack.)
//!
//! What counts as touching a library is decided in one place, by the exhaustive matches in
//! [`effect_now`], [`part_now`], [`part_can_fail`], [`keyword_action`], [`sel_in_library`],
//! [`replacement_action_now`] and [`event_reach`], so a new AST variant needs a decision
//! here; [`custom_effect_library`] lists the custom effects known to touch no library,
//! [`custom_rule`] the rules implemented in code.

use crate::r506_common::{custom_card, custom_with};
use mtg_engine::ability::*;
use mtg_engine::card::{CardDb, CardDef};
use mtg_engine::eval::Ctx;
use mtg_engine::events::Event;
use mtg_engine::game::EventObserver;
use mtg_engine::keywords::{Keyword, KeywordKind};
use mtg_engine::mana::{ManaCost, ManaType, SpendContext};
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::*;
use serde_json::Value as J;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex};

// ===========================================================================
// What touches a library (CR 733.1)
// ===========================================================================

/// How a library action relates to paying a cost.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LibKind {
    /// Puts cards into a library.
    Into,
    /// Draws: a card from a library to a hand (drawing needs no card, CR 121.4).
    Draw,
    /// Moves cards from a library to a public zone or reveals them: paid after every other
    /// part (CR 601.2h), and needs the cards to be there.
    Take,
    /// Searches, shuffles, looks at or reorders a library, or something the scan can't
    /// tell apart (an unclassified custom effect): paid after every other part.
    Other,
}

/// A library action (CR 733.1): what it is, for messages.
#[derive(Clone, Debug)]
struct Lib {
    kind: LibKind,
    what: String,
}

/// What a payment, a mana ability, or a replacement effect or rule applying in the middle
/// of one can cause beyond what every payment can ([`event_reach`] counts those always): an
/// event a replacement effect watches counts as one that can happen while an action is
/// attempted only when something in the pool causes it there ([`reach`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Ev {
    /// A card, or a token with abilities, put onto the battlefield.
    Enters,
    /// A permanent destroyed.
    Destroy,
    /// A permanent transformed.
    Transform,
    /// A permanent turned face up.
    TurnFaceUp,
    /// Damage dealt to a permanent.
    DamageToPermanent,
    /// Proliferating.
    Proliferate,
    /// A player losing the game.
    LoseGame,
    /// A spell or ability countered.
    Countered,
    /// A player becoming the monarch (which ends "until an opponent becomes the monarch").
    BecomesMonarch,
    /// A card discarded because of an effect (a cost's discard isn't one).
    DiscardByEffect,
    /// What the scan doesn't follow: reported as "(e-assumption)".
    Unfollowed(&'static str),
}

impl Ev {
    fn describe(self) -> &'static str {
        match self {
            Ev::Enters => "a card or a token with abilities put onto the battlefield",
            Ev::Destroy => "a permanent destroyed",
            Ev::Transform => "a permanent transformed",
            Ev::TurnFaceUp => "a permanent turned face up",
            Ev::DamageToPermanent => "damage dealt to a permanent",
            Ev::Proliferate => "proliferating",
            Ev::LoseGame => "a player losing the game",
            Ev::Countered => "a spell or ability countered",
            Ev::BecomesMonarch => "a player becoming the monarch",
            Ev::DiscardByEffect => "a card discarded because of an effect",
            Ev::Unfollowed(s) => s,
        }
    }
}

/// What an effect or a cost part does as it's performed, as far as the scan cares.
#[derive(Default)]
struct Now {
    lib: Vec<Lib>,
    /// What it causes beyond what every payment can ([`Ev`]).
    events: Vec<Ev>,
    /// Costs it has a player pay as it's performed ("[player] may pay [cost]", CR 118.12):
    /// payments of their own (`Game::pay_cost`), reversed by themselves if they fail.
    payments: Vec<Cost>,
}

impl Now {
    fn lib(&mut self, kind: LibKind, what: &str) {
        self.lib.push(Lib {
            kind,
            what: what.to_string(),
        });
    }
    fn whats(&self) -> String {
        let v: Vec<&str> = self.lib.iter().map(|l| l.what.as_str()).collect();
        v.join(", ")
    }
}

fn dedup_events(events: &[Ev]) -> Vec<Ev> {
    let mut v = events.to_vec();
    v.sort();
    v.dedup();
    v
}

/// Whether a value names a library anywhere: a zone (`ZoneKind::Library`,
/// `FunctionZone::Library`), a variant about one (`Sel::TopOfLibrary`, ...) or a custom
/// part whose name mentions one (whatever the case). Used for filters, trigger conditions
/// and search specifications, whose many variants say where cards are only through the
/// zones they name. (A custom filter part doesn't decide where a selection looks: the
/// engine takes that from the filter's zone parts, `Filter::zone`, `alternative_zones`,
/// `Filter::In`, which this finds.)
fn mentions_library<T: serde::Serialize>(t: &T) -> bool {
    fn names(s: &str) -> bool {
        s.to_ascii_lowercase().contains("library")
    }
    fn walk(v: &J) -> bool {
        match v {
            J::String(s) => names(s),
            J::Array(a) => a.iter().any(walk),
            J::Object(m) => m.iter().any(|(k, x)| names(k) || walk(x)),
            _ => false,
        }
    }
    serde_json::to_value(t).is_ok_and(|v| walk(&v))
}

/// Whether a selection takes cards in a library (CR 401): "the top card of your library",
/// "a card from your library". (A variable holds cards an earlier instruction chose: an
/// `Effect::Store` from a library counts as touching it.)
fn sel_in_library(s: &Sel) -> bool {
    match s {
        Sel::TopOfLibrary(..) | Sel::BottomOfLibrary(..) => true,
        Sel::All(f) | Sel::AtRandom { filter: f, .. } | Sel::Choose { filter: f, .. } => {
            mentions_library(f)
        }
        Sel::Matching(s, f) => sel_in_library(s) || mentions_library(f),
        Sel::AttachedAsItLeft(s) | Sel::HostOf(s) | Sel::Before(s) => sel_in_library(s),
        Sel::Union(v) => v.iter().any(sel_in_library),
        Sel::ThisTurn(t) | Sel::EventsDuring(_, t) => mentions_library(&**t),
        // Permanents, spells, players, targets (never in a hidden zone, CR 115.1),
        // graveyards and exile.
        Sel::None
        | Sel::This
        | Sel::Target(_)
        | Sel::AllTargets
        | Sel::Var(_)
        | Sel::TriggerObject
        | Sel::TriggerLki
        | Sel::TriggerOtherObject
        | Sel::TriggerObjects
        | Sel::TriggerPlayer
        | Sel::AttachedTo
        | Sel::AttachedToThis
        | Sel::Players(_)
        | Sel::Linked
        | Sel::CreatorLinked
        | Sel::ExiledWithCardsNamed(_)
        | Sel::TriggerSpell
        | Sel::TopOfGraveyard(_)
        | Sel::LinkedNoted
        | Sel::LinkedNotedLastKnown => false,
    }
}

/// What a keyword action (CR 701) does that the scan cares about: its library action, and
/// what it causes beyond what every payment can ([`Ev`]).
fn keyword_action(a: KeywordAction) -> (Option<(LibKind, &'static str)>, Option<Ev>) {
    use KeywordAction::*;
    use LibKind::{Draw, Into, Take};
    match a {
        Explore => (
            Some((
                Take,
                "explores (reveals the top card of a library, CR 701.44a)",
            )),
            None,
        ),
        Connive => (Some((Draw, "connives (draws, CR 701.50a)")), None),
        // Face-down permanents enter the battlefield.
        Manifest | ManifestDread | Cloak => (
            Some((Take, "manifests or cloaks library cards")),
            Some(Ev::Enters),
        ),
        Fateseal => (Some((LibKind::Other, "fateseals (CR 701.29a)")), None),
        Clash => (
            Some((
                Take,
                "clashes (reveals the top card of a library, CR 701.30a)",
            )),
            None,
        ),
        Discover => (
            Some((Take, "discovers (exiles cards from a library, CR 701.57a)")),
            Some(Ev::Unfollowed("casts a card")),
        ),
        Learn => (Some((Draw, "learns (may draw a card, CR 701.48a)")), None),
        // Draws, and creates a token with no abilities (CR 701.70a).
        Recruit => (Some((Draw, "recruits (draws a card, CR 701.70a)")), None),
        Seek => (
            Some((Draw, "seeks (a card from a library into a hand)")),
            None,
        ),
        Conjure => (
            Some((Into, "conjures (may create a card in a library)")),
            Some(Ev::Enters),
        ),
        Other => (
            Some((LibKind::Other, "an unclassified keyword action")),
            Some(Ev::Unfollowed("an unclassified keyword action")),
        ),
        // Tokens with abilities (a Clue, an Incubator, a Jace planeswalker, copies of
        // tokens).
        Investigate | Incubate | EmpowerJace | Populate => (None, Some(Ev::Enters)),
        Proliferate => (None, Some(Ev::Proliferate)),
        // A melded permanent enters the battlefield (CR 701.42a); converting transforms one
        // (CR 701.28a).
        Meld => (None, Some(Ev::Enters)),
        Convert => (None, Some(Ev::Transform)),
        // Counters, tokens with no abilities, designations, other decks (attractions,
        // planes, schemes, contraptions, dungeons), graveyards, hands and permanents.
        Amass | Goad | Bolster | Adapt | Monstrosity | Support | Detain | Venture
        | CollectEvidence | Suspect | Forage | Endure | TimeTravel | TheRingTemptsYou
        | OpenAttraction | RollAttractions | Planeswalk | SetInMotion | Abandon | Vote | Exert
        | Double | Triple | Assemble | VillainousChoice | Harness | Airbend | Earthbend
        | Waterbend | Blight | Heal | Behold | Exchange | Specialize | Perpetually | Mutate => {
            (None, None)
        }
    }
}

/// Custom effects (`Effect::Custom`, implemented in code) known to touch no library and to
/// cause none of the events [`Ev`] lists. Any other counts as touching a library wherever
/// that matters (a cost, a mana ability, a replacement effect): read its code and add it
/// here.
fn custom_effect_library(name: &str) -> Option<String> {
    const NO_LIBRARY: &[&str] = &[
        // Craft's materials: permanents and graveyard cards (`kw/craft.rs`).
        mtg_engine::kw::craft::EXILE_MATERIALS,
        // Forecast's cost reveals the card from its owner's hand (`kw/forecast.rs`).
        mtg_engine::kw::forecast::REVEAL,
        // Cards revealed from a hand (Metalworker, Sacellum Godspeaker), and the number
        // recorded for the player who revealed them.
        mtg_engine::kw::reveal_from_hand::REVEAL_CHOSEN_ANY,
        mtg_engine::kw::hand_graveyard_actions::RECORD_THAT_MANY,
        // Revealing a choice made for the source ("Reveal the player you chose"), and
        // remembering the objects chosen or revealed from a hand as a spell was cast.
        mtg_engine::choices::REVEAL_CHOSEN_PLAYER,
        mtg_engine::choices::REVEAL_CHOSEN_TYPE,
        mtg_engine::choices::REMEMBER_CHOSEN_OBJECTS,
        // How a permanent enters ("as ~ enters", CR 614.1c), which a card an "until" exile
        // returns in the middle of a payment does too: counters (sunburst, riot, amplify
        // revealing cards from a hand, tribute, devour sacrificing permanents as it
        // enters, `kw/`), secret and random choices (`choices.rs`,
        // `kw/choice_grammar.rs`), life paid (`kw/pay_any_amount_of_life.rs`: with
        // Ashiok, Wicked Manipulator, that's Ashiok's way), entering as a copy (The
        // Mimeoplasm exiles creature cards from graveyards, Vesuvan Doppelganger keeps its
        // color, `cards/`). (Private names are written out.)
        mtg_engine::kw::sunburst::SUNBURST,
        mtg_engine::kw::sunburst::SUNBURST_PLUS1,
        "riot:counter or haste",
        "amplify:",
        "tribute:",
        "devour:",
        mtg_engine::choices::SECRET_NEXT,
        mtg_engine::kw::choice_grammar::RANDOM_NUMBER,
        mtg_engine::kw::pay_any_amount_of_life::PAY_ANY_LIFE,
        "card:The Mimeoplasm:exile two creature cards and enter as a copy of one",
        "card:Vesuvan Doppelganger:enter as a copy, keeping its own color",
        // Damage removed instead of a destruction (umbra armor, `kw/umbra_armor.rs`;
        // Pyramids, `cards/pyramids.rs`).
        "umbra armor:remove all damage from it",
        "card:Pyramids:remove all damage marked on it",
        // Entering attached to a host chosen as it enters, or going to the graveyard
        // instead (`kw/enters_attached.rs`): no library.
        "enter attached to the chosen object",
        "put the entering permanent into its owner's graveyard instead",
    ];
    if NO_LIBRARY.iter().any(|p| name.starts_with(p)) {
        return None;
    }
    Some(format!("an unclassified custom effect {name:?}"))
}

/// How a rule implemented in code bears on CR 733.1's exceptions.
#[derive(Clone, Debug, PartialEq, Eq)]
enum CustomRule {
    /// Its code touches no library while an action is attempted (beyond a library action
    /// it changes, which is found where it's done), and causes none of the events [`Ev`]
    /// lists there.
    NoLibrary,
    /// It turns something a payment does into a library action (a replacement effect, (e)).
    LibraryInstead(&'static str),
    /// "You may cast this card from your graveyard by paying [cost] rather than paying its
    /// mana cost" or "... in addition to paying its other costs": a spell's cost, written
    /// in the name (`kw/cast_self_from_graveyard.rs` parses it as the spell is cast).
    SpellCost(String),
    /// Not classified yet.
    Unknown,
}

/// Rules implemented in code that a card turns on by name (`StaticEffect::Custom`,
/// `PlayerModification::Custom`, `Modification::Custom`): the registry-wide hooks of
/// `kw::KeywordRules` find their sources by these names (the hooks that act while an
/// action is attempted: `pay_life_instead`, `on_event`, `pay_mana_otherwise`,
/// `pay_activation_otherwise`, `announce_choices`, `pay_attack_costs`, `after_draw`,
/// `searching_library`, `search_found`, `perform_special_action`; the others answer
/// questions or act at other times), so what a rule does while an action is attempted is
/// decided here, name by name. An unclassified one fails the scan: read the hooks that look
/// for it and add it here.
fn custom_rule(name: &str) -> CustomRule {
    use mtg_engine::kw::pay_life_exile_instead::PAY_LIFE_EXILE_INSTEAD;
    if name == PAY_LIFE_EXILE_INSTEAD {
        // Ashiok, Wicked Manipulator: `Game::pay_life` asks `kw::pay_life_instead`, which
        // exiles that many cards from the top of the payer's library instead.
        return CustomRule::LibraryInstead(
            "turns paying life into exiling that many cards from the top of a library (`kw::pay_life_instead`): life is paid while an action is attempted (a cost's `PayLife`; mana abilities such as Mana Confluence; Phyrexian mana)",
        );
    }
    for prefix in ["cast from graveyard also:", "cast from graveyard instead:"] {
        if let Some(cost) = name.strip_prefix(prefix) {
            return CustomRule::SpellCost(cost.to_string());
        }
    }
    // Names, by what their hooks do.
    const NO_LIBRARY: &[&str] = &[
        // Combat: attack and block legality and costs (exerting), evasion, combat damage and what
        // damage does.
        "assigns combat damage as though unblocked",
        "may assign combat damage as though unblocked",
        "creatures assign combat damage as though unblocked",
        "can block creatures with shadow as though it had shadow",
        "can block creatures with shadow as though they didn't have shadow",
        "exert as it attacks",
        "card:Arboria:creatures can't attack players who did nothing last turn",
        "card:Become the Pilot:can't be blocked unless attacking its owner",
        "card:Butcher Orgg:divide combat damage among defending player and their creatures",
        "card:Cunning Giant:may assign its combat damage to a creature defending player controls",
        "card:Defensive Formation:you assign the combat damage of creatures attacking you",
        "card:Loot, the Anomaly:assigns combat damage as though negative power were positive",
        "card:Ogre Enforcer:lethal damage only from a single source",
        "card:The Aetherspark:can't be attacked while attached to a creature",
        "card:Zilortha, Strength Incarnate:lethal damage by power",
        "card:Ghostly Flame:black and red sources of damage are colorless",
        "card:Phyrexian Unlife:damage dealt to you as though its source had infect",
        "wither:all damage as though its source had wither",
        // Permanents and players: counters, loyalty, untapping, the legend rule, poison, the
        // monarch, losing for 0 life, radiation, extra turns.
        "card:Bewitching Leechcraft:remove a +1/+1 counter instead of untapping",
        "card:Fear of Sleep Paralysis:stun counters can't be removed from opponents' permanents",
        "card:Sanctum Lurker:planeswalkers you control survive 0 loyalty",
        "untap all permanents you control during each opponent's untap step",
        "legend rule doesn't apply to permanents you control",
        "can't get poison counters",
        "can't become the monarch",
        "don't lose the game for having 0 or less life",
        "radiation: gain life rather than lose life",
        "skip extra turns",
        // Keen-Eared Sentry: a venture that would exceed the limit doesn't happen
        // (`kw::venture_prohibited`); venturing moves a marker, no library card.
        "card:Keen-Eared Sentry:opponents venture at most once each turn",
        "opponents skip extra turns",
        // Targets, hexproof and ward; what may be cast, activated or played, from where, when
        // and how often; what casting and activating cost (life paid instead of mana is
        // Ashiok's case above). Damping Engine's special action sacrifices a permanent, and
        // plotting a card from the top of a library (a special action) exiles it after the
        // plot cost is paid (`kw/plot.rs`): nothing can fail after either.
        "opponents and their creatures targetable as though no hexproof",
        "opponents and their permanents targetable as though no hexproof",
        "opponents' creatures targetable as though no hexproof",
        "opponents' creatures targetable by any player as though no hexproof; their ward doesn't trigger",
        "card:Wall of Shadows:can't be targeted by Wall-only spells or abilities",
        "card:Haakon, Stromgald Scourge:can't be cast from anywhere but your graveyard",
        "card:Mana Maze:can't cast spells sharing a color with the last spell",
        "card:Damping Engine:the player with the most permanents can't play lands or cast permanent spells",
        "bestow:you may cast this card from your graveyard using its bestow ability",
        "you may cast this card from your graveyard using its blitz ability",
        "you may cast this card from your graveyard using its mutate ability",
        "warp:cast from graveyard using warp",
        "teamwork:flash if cast using teamwork",
        "other graveyards as though yours",
        "exhaust:during your turn, you may activate exhaust abilities as though they haven't been activated",
        "boast:creatures you control can boast twice during each of your turns",
        "power-up:can be activated an additional time",
        "plot:you may plot nonland cards from the top of your library",
        "plot:the top card of your library has plot equal to its mana cost",
        "foretell:costs {1} less and can be done on any player's turn",
        "card:Trinisphere:spells cost at least three mana",
        "card:Liesa, Shroud of Dusk:commander tax paid in life",
        "you may pay {0} rather than pay echo costs",
        // Mana: how it's spent, kept, or lost as steps end (mana burn's life loss).
        "spend mana as though any color",
        "mana burn",
        "card:Undiscovered Paradise:returns to its owner's hand during your next untap step",
        // Resolving: where a resolving spell goes, copies, votes, villainous choices, cascade,
        // choices made for the source, planeswalking.
        "put this spell into its owner's hand instead of into a graveyard as it resolves",
        "exile this spell instead of putting it into a graveyard as it resolves; return it to its owner's hand at the beginning of the next end step",
        "plot:exile this spell instead of putting it into your graveyard as it resolves; it becomes plotted",
        "card:Twinning Staff:copying a spell makes an additional copy",
        "villainous choice: opponents face it an additional time",
        "vote: you get an additional vote",
        "vote: you may vote an additional time",
        "cascade:as you cascade, put a land card from among the exiled cards onto the battlefield tapped",
        "choices: as attached to a creature, choose a color",
        "card:Susan Foreman:look at the top two cards of your planar deck before planeswalking",
        // Draws, surveils and searches: library actions themselves, found where they're done (a
        // drawn card revealed is in a hand); the draw step's extra draw.
        "reveal the first card you draw each turn",
        "reveal the first card you draw on each of your turns",
        "you may reveal the first card you draw each turn as you draw it",
        "card:Booby Trap:the chosen player reveals each card they draw",
        "you draw a card during each opponent's draw step",
        "control opponents while searching",
        "spells and abilities your opponents control can't cause their controller to search their library",
        "spells and abilities your opponents control can't cause you to discard cards",
        "card:Opposition Agent:opponents exile the cards they find",
        "card:Panglacial Wurm:may be cast from your library while you search it",
        // Hidden information: looking at cards (not revealing them), revealed hands
        // (`reveal::PLAYS_WITH_HAND_REVEALED` after the merge with origin).
        "may look at the top card of their library any time",
        "may look at face-down creatures you don't control",
        "may look at cards exiled with this",
        // (Each player looks at the face-down cards they exiled with it, `kw/exile_provenance.rs`.)
        "each player may look at cards they exiled with this",
        "facedown: may look at opponents' face-down creatures",
        "plays with their hand revealed",
        // Characteristics (layers), names, devotion, triggers.
        "card:Duplicant:creature types of the last creature card exiled with it",
        "card:Swirl the Mists:color words are changed to the chosen color word",
        "choices: has each of the chosen abilities",
        "choices: is the second chosen type",
        "end this effect: loses this ability",
        "lose all abilities except mana abilities",
        "lose_all_land_types",
        "devotion increased by one",
        // Deck construction, commanders, the draft.
        "commander:can be your commander",
        "partner:choose a background",
        "partner:doctor's companion",
        "draft:reveal and note cards drafted this round",
        // Foretelling exiles a card from a hand (CR 702.143a).
        mtg_engine::kw::foretell::FIRST_EACH_TURN_FREE,
    ];
    // Prefixes of families of names (a name's parameters follow its prefix): landwalk,
    // targeting and activation restrictions, cost reductions, crewing and saddling, how
    // mana is spent and kept, exiling a resolving spell with counters, surveils and searches,
    // revealed hands, copy exceptions, lost abilities, names, triggers (Torpor Orb's
    // family, `kw/torpor.rs`), deck construction.
    const NO_LIBRARY_FAMILIES: &[&str] = &[
        // Abilities that function during the draft (CR 905.2) and the game's reads of what
        // was noted or removed then (`draft.rs`, `kw/draft_4.rs`): no library involved.
        "draft:",
        "blockable as though no landwalk:",
        "cast target forbidden:",
        "tap abilities can't be activated:",
        "power-up:other creatures' cost less:",
        "plot:plotting cards from your hand costs less:",
        "spell cost:unlock costs less:",
        "tap_total_power:",
        "spend rules:",
        "keep unspent mana",
        "unspent mana becomes ",
        "resolve exile with counters:",
        "surveil extra:",
        "search portion:",
        "hands revealed:",
        "copy exception:",
        "lose_ability:",
        "counts as named for spells named:",
        "doesn't cause abilities to trigger:",
        "deck:",
    ];
    if NO_LIBRARY.contains(&name) || NO_LIBRARY_FAMILIES.iter().any(|p| name.starts_with(p)) {
        return CustomRule::NoLibrary;
    }
    CustomRule::Unknown
}

/// What an effect does as it's performed, nested instructions included, into `now`.
/// Abilities it creates that wait (reflexive and delayed triggered abilities, which use the
/// stack and so are dropped with an undone action, CR 733.1; instructions for a later
/// turn) aren't performed now; a delayed triggered mana ability is checked as a mana
/// ability by the scan.
fn effect_now(e: &Effect, now: &mut Now) {
    use LibKind::*;
    match e {
        // --- Performed now: nested instructions ------------------------------------
        Effect::Seq(v) => v.iter().for_each(|x| effect_now(x, now)),
        Effect::If {
            then, otherwise, ..
        } => {
            effect_now(then, now);
            effect_now(otherwise, now);
        }
        Effect::PayOptional {
            cost,
            then,
            otherwise,
            ..
        } => {
            // Its library actions and events are this effect's; the payment is checked on
            // its own (the payments nested in it by that check).
            let mut paying = Now::default();
            cost.parts.iter().for_each(|p| part_now(p, &mut paying));
            now.lib.extend(paying.lib);
            now.events.extend(paying.events);
            now.payments.push(cost.clone());
            effect_now(then, now);
            effect_now(otherwise, now);
        }
        Effect::May { effect, .. }
        | Effect::ForEachPlayer { effect, .. }
        | Effect::InTurnOrder { effect, .. }
        | Effect::AsPlayer { effect, .. }
        | Effect::Repeat { effect, .. }
        | Effect::TokensEnterWithCounters { effect, .. }
        | Effect::TokensEnterAttached { effect, .. }
        | Effect::TokensJoinCombat { effect, .. }
        | Effect::WithPlayTerms { effect, .. }
        | Effect::EnterCopyExtra { effect, .. }
        | Effect::OnEntry(effect)
        | Effect::PersistentMana(effect)
        | Effect::CombatMana(effect) => effect_now(effect, now),
        Effect::RepeatProcess { body } => effect_now(body, now),
        // (A shield's instruction runs when the shield is used; inside a payment that may
        // be the same payment, so it counts as performed now. The scan also treats the
        // shield as a replacement effect, `Scanner::walk`.)
        Effect::ChooseCounterKind { then, .. }
        | Effect::ForEachCounterKind { then, .. }
        | Effect::RegenerateWith { then, .. } => effect_now(then, now),
        Effect::PreventDamage { then, .. } => {
            if let Some(t) = then {
                effect_now(t, now);
            }
        }
        Effect::ChooseOne { options, .. } => {
            options.iter().for_each(|(_, x)| effect_now(x, now))
        }
        Effect::ForEach { sel, effect, .. } => {
            if sel_in_library(sel) {
                now.lib(Take, "goes through library cards");
            }
            effect_now(effect, now);
        }
        Effect::AddManaWithSpentTrigger {
            add,
            body,
            additional,
            ..
        } => {
            effect_now(add, now);
            // "If that mana is spent to cast a spell, [effect]" is applied as the mana is
            // spent; otherwise it's a delayed triggered ability (CR 106.6).
            if *additional {
                effect_now(&body.effect, now);
            }
        }
        // A replacement effect created now may apply to what happens next.
        Effect::AddReplacement { def, .. } => replacement_action_now(&def.action, now),
        Effect::SelfReplace {
            replacement,
            effect,
        } => {
            replacement_action_now(&replacement.action, now);
            effect_now(effect, now);
        }
        Effect::RollDice(d) => d.table.iter().for_each(|r| effect_now(&r.effect, now)),
        Effect::FlipCoins(c) => {
            for x in [&c.on_win, &c.on_lose, &c.on_heads, &c.on_tails] {
                effect_now(x, now);
            }
        }
        Effect::KeywordAction { action, what, .. } => {
            keyword_action_now(*action, what, now);
        }
        Effect::KeywordActionEx(spec) => {
            keyword_action_now(spec.action, &spec.what, now);
            spec.options.iter().for_each(|(_, x)| effect_now(x, now));
        }

        // --- Library actions -------------------------------------------------------
        Effect::Draw { .. } => now.lib(Draw, "draws"),
        Effect::Mill { .. } => now.lib(Take, "mills"),
        Effect::Scry { .. } => now.lib(Other, "scries"),
        Effect::Surveil { .. } => now.lib(Take, "surveils"),
        Effect::Search { .. } => now.lib(Other, "searches a library"),
        Effect::SearchCards(spec) => {
            if mentions_library(&**spec) {
                now.lib(Other, "searches a library");
            }
        }
        Effect::Shuffle { .. } => now.lib(Other, "shuffles a library"),
        Effect::ShuffleInto { .. } | Effect::ShuffleIntoLibrary { .. } => {
            now.lib(Into, "shuffles cards into a library")
        }
        Effect::Dig { .. } | Effect::DigStep(_) => {
            now.lib(Take, "looks at, reveals or takes cards from the top of a library")
        }
        Effect::RevealUntil { .. } => now.lib(Take, "reveals cards from a library"),
        Effect::RestartGame { .. } => {
            now.lib(Other, "restarts the game (every library is shuffled)");
            now.events.push(Ev::Unfollowed("restarts the game"));
        }
        Effect::Move { what, to } => {
            if sel_in_library(what) {
                now.lib(Take, "moves cards from a library");
            }
            match to.zone {
                ZoneKind::Library => now.lib(Into, "puts cards into a library"),
                ZoneKind::Battlefield => now.events.push(Ev::Enters),
                ZoneKind::Stack => now
                    .events
                    .push(Ev::Unfollowed("puts a card onto the stack")),
                _ => {}
            }
        }
        Effect::Exile { what, .. } | Effect::ExileUntil { what, .. } => {
            if sel_in_library(what) {
                now.lib(Take, "exiles library cards");
            }
        }
        Effect::Store { sel, .. } => {
            if sel_in_library(sel) {
                now.lib(Take, "chooses library cards to act on");
            }
        }
        Effect::PlayCard { what, .. } => {
            if sel_in_library(what) {
                now.lib(Take, "plays a card from a library");
            }
            now.events.push(Ev::Unfollowed(
                "plays a card (casts it, or puts a land onto the battlefield)",
            ));
        }
        Effect::CopyCard { what, .. } => {
            if sel_in_library(what) {
                now.lib(Other, "copies a library card");
            }
        }
        Effect::Exchange(x) => {
            if mentions_library(&**x) {
                now.lib(Other, "exchanges a library with another zone");
            }
        }
        Effect::Piles(p) => {
            use mtg_engine::piles::PileAction as P;
            match &**p {
                P::PutOnLibrary { .. } => now.lib(Into, "puts a pile on a library"),
                P::SearchPile { .. } => now.lib(Other, "searches a pile in a library"),
                P::Separate { what, .. }
                | P::SeparateFaceDown { what, .. }
                | P::ExileAsPile { what, .. }
                | P::Look { what }
                | P::Shuffle { what } => {
                    if sel_in_library(what) {
                        now.lib(Take, "puts library cards into piles");
                    }
                }
                P::Top { from } => {
                    if sel_in_library(from) {
                        now.lib(Take, "takes the top card of a pile in a library");
                    }
                }
                P::Choose { .. } | P::TurnFaceUp { .. } | P::Recall { .. } => {}
            }
        }
        Effect::Custom(name) => {
            if let Some(s) = custom_effect_library(name) {
                now.lib(Other, &s);
            }
        }

        // --- What not every payment causes ([`Ev`]) -------------------------------
        // Tokens with no abilities every payment may create; a token with abilities may
        // have its own "as this enters" replacement effect (CR 614.1c).
        Effect::CreateToken { spec, .. }
        | Effect::CreateTokenWithPT { spec, .. }
        | Effect::CreateTokenAttached { spec, .. } => {
            if !spec.abilities.is_empty() {
                now.events.push(Ev::Enters);
            }
        }
        Effect::CreateTokenCopy { .. } => now.events.push(Ev::Enters),
        Effect::CastCard { .. } => now.events.push(Ev::Unfollowed("casts a card")),
        // A discard an effect causes (a cost's isn't one, CR 701.9).
        Effect::Discard { .. } | Effect::DiscardHand { .. } => {
            now.events.push(Ev::DiscardByEffect)
        }
        Effect::Destroy { .. } => now.events.push(Ev::Destroy),
        // Day becoming night (or night day) transforms the daybound and nightbound
        // permanents (CR 702.145).
        Effect::Transform { .. } | Effect::SetDayNight { .. } => now.events.push(Ev::Transform),
        Effect::TurnFaceUp { .. } => now.events.push(Ev::TurnFaceUp),
        Effect::DealDamage { to, .. } | Effect::DealDamageExcess { to, .. } => {
            if !matches!(to, Sel::Players(_) | Sel::TriggerPlayer) {
                now.events.push(Ev::DamageToPermanent);
            }
        }
        Effect::DealDividedDamage { .. } | Effect::Fight { .. } => {
            now.events.push(Ev::DamageToPermanent)
        }
        // A player who wins makes every other player lose (CR 104.2a), which Lich's Mirror
        // replaces like any other loss.
        Effect::LoseGame { .. } | Effect::WinGame { .. } => now.events.push(Ev::LoseGame),
        Effect::CounterSpell { .. } => now.events.push(Ev::Countered),
        // Ends "until an opponent becomes the monarch".
        Effect::BecomeMonarch { .. } => now.events.push(Ev::BecomesMonarch),

        // --- Not performed now: abilities and instructions that wait ---------------
        // Reflexive and delayed triggered abilities use the stack (a delayed triggered
        // mana ability is checked as a mana ability); a later turn's instructions; a
        // special action is a separate action (its definition is scanned on its own).
        Effect::Reflexive { .. }
        | Effect::DelayedTrigger { .. }
        | Effect::AtNext { .. }
        | Effect::DuringNext { .. }
        | Effect::ExtraTurnWith { .. }
        | Effect::OfferSpecialAction { .. } => {}

        // --- No library ------------------------------------------------------------
        // Permanents, spells, players, hands, graveyards, counters, mana, choices, and
        // effects that change things later (whose abilities and costs the scan finds on
        // their own).
        Effect::Noop
        | Effect::AddToPlayerTally { .. }
        | Effect::RepeatThisProcess
        | Effect::StoreValue { .. }
        | Effect::Note { .. }
        | Effect::SetX { .. }
        | Effect::Sacrifice { .. }
        | Effect::SacrificeObjects { .. }
        | Effect::Tap { .. }
        | Effect::Untap { .. }
        | Effect::AddCounters { .. }
        | Effect::RemoveCounters { .. }
        | Effect::RemoveCountersUpTo { .. }
        | Effect::MoveCounters { .. }
        | Effect::PutCountersOf { .. }
        | Effect::CountersFromAmong { .. }
        | Effect::DistributeCounters { .. }
        | Effect::PutEventCounters { .. }
        | Effect::Modify { .. }
        | Effect::AddRestriction { .. }
        | Effect::AddPlayerEffect { .. }
        | Effect::GainControl { .. }
        | Effect::ExchangeControl { .. }
        | Effect::CopySpell { .. }
        | Effect::CopySpellExcept { .. }
        | Effect::PutSticker { .. }
        | Effect::SpendAnyTypeMana { .. }
        | Effect::ChangeTargets { .. }
        | Effect::BecomeCopy { .. }
        | Effect::BecomeCopyExcept { .. }
        | Effect::Regenerate { .. }
        | Effect::Attach { .. }
        | Effect::AttachAsCreature { .. }
        | Effect::Unattach { .. }
        | Effect::PhaseOut { .. }
        | Effect::TurnFaceDown { .. }
        | Effect::RemoveFromCombat { .. }
        | Effect::BecomeUnblocked { .. }
        | Effect::BlockAttackerOfChoice { .. }
        | Effect::Choose { .. }
        | Effect::EnterTapped
        | Effect::EnterWithCounters { .. }
        | Effect::EnterPrepared
        | Effect::EnterCopyExceptions(_)
        | Effect::EnterAs(_)
        | Effect::SetPrepared { .. }
        | Effect::GainLife { .. }
        | Effect::LoseLife { .. }
        | Effect::SetLife { .. }
        | Effect::ExchangeLifeTotals { .. }
        | Effect::AddMana { .. }
        | Effect::SetClassLevel { .. }
        // The mana abilities it activates are mana abilities the scan checks.
        | Effect::ActivateManaAbilities { .. }
        | Effect::LoseUnspentMana { .. }
        | Effect::AddPlayerCounters { .. }
        | Effect::RevealHand { .. }
        | Effect::LookAtHand { .. }
        | Effect::ExtraTurn { .. }
        | Effect::ExtraCombat { .. }
        | Effect::AddTurnParts { .. }
        | Effect::Skip { .. }
        | Effect::NoteLinked { .. }
        | Effect::CreateEmblem { .. }
        | Effect::KeepAndSacrificeRest { .. }
        | Effect::GrantPlayPermission { .. }
        | Effect::PreventDividedDamage { .. }
        | Effect::TakeInitiative { .. }
        | Effect::ChangeText { .. }
        | Effect::PhaseOutUntil { .. }
        | Effect::ChooseSource { .. }
        | Effect::NextSpell { .. }
        | Effect::CopySpellRetargeted { .. } => {}
    }
}

fn keyword_action_now(a: KeywordAction, what: &Sel, now: &mut Now) {
    let (lib, event) = keyword_action(a);
    if let Some((k, s)) = lib {
        now.lib(k, s);
    }
    if sel_in_library(what) {
        now.lib(LibKind::Take, "a keyword action on library cards");
    }
    if let Some(e) = event {
        now.events.push(e);
    }
}

/// What paying a cost part does.
fn part_now(part: &CostPart, now: &mut Now) {
    match part {
        CostPart::Mill(_) => now.lib(LibKind::Take, "mills"),
        CostPart::Exile { zone, filter, .. } => {
            if *zone == ZoneKind::Library || mentions_library(filter) {
                now.lib(LibKind::Take, "exiles library cards");
            }
        }
        CostPart::PutFromHandOnLibrary { .. } => {
            now.lib(LibKind::Into, "puts cards from a hand on a library")
        }
        CostPart::Effect(e) => effect_now(e, now),
        CostPart::Repeated { cost, .. } => cost.parts.iter().for_each(|p| part_now(p, now)),
        CostPart::OneOf(costs) => costs
            .iter()
            .for_each(|c| c.parts.iter().for_each(|p| part_now(p, now))),
        CostPart::PayManaCostOf(s) => {
            if sel_in_library(s) {
                now.lib(LibKind::Other, "pays the mana cost of a library card");
            }
        }
        // Permanents, hands, graveyards, life, counters, energy.
        CostPart::Tap
        | CostPart::Untap
        | CostPart::PayLife(_)
        | CostPart::Loyalty(_)
        | CostPart::SacrificeSelf
        | CostPart::Sacrifice { .. }
        | CostPart::DiscardSelf
        | CostPart::Discard { .. }
        | CostPart::DiscardHand
        | CostPart::ExileSelf
        | CostPart::ReturnToHand { .. }
        | CostPart::ReturnSelfToHand
        | CostPart::RemoveCounters { .. }
        | CostPart::RemoveCountersFromAmong { .. }
        | CostPart::AddCounters { .. }
        | CostPart::TapUntapped { .. }
        | CostPart::TapTotalPower { .. }
        | CostPart::UntapTapped { .. }
        | CostPart::PayEnergy(_)
        | CostPart::PayPlayerCounters { .. }
        | CostPart::RevealFromHand { .. }
        | CostPart::RevealSelf
        | CostPart::RemoveCountersFromOne { .. }
        | CostPart::ExertSelf
        | CostPart::CollectEvidence(_)
        | CostPart::Forage => {}
    }
}

fn part_lib(part: &CostPart) -> Now {
    let mut now = Now::default();
    part_now(part, &mut now);
    now
}

/// Whether paying a part can fail although the check of each part before any is paid
/// (`cost_part_payable`) passed: because an earlier part of the same cost used what it
/// needs. Mirrors `Game::pay_cost_part`, which returns an error for these; the parts said
/// never to fail are paid in a game with what they need gone by
/// `the_cost_parts_the_scan_says_cant_fail_dont_fail_in_a_game`.
fn part_can_fail(part: &CostPart) -> bool {
    match part {
        // Never an error once checked (a mill or a return takes what's left).
        CostPart::DiscardHand
        | CostPart::ExileSelf
        | CostPart::ReturnSelfToHand
        | CostPart::ReturnToHand { .. }
        | CostPart::AddCounters { .. }
        | CostPart::ExertSelf
        | CostPart::Mill(_)
        | CostPart::PutFromHandOnLibrary { .. } => false,
        // Performed; an error only if a player who can't gain life would gain life
        // (`life_totals::cost_life_gain_possible`).
        CostPart::Effect(e) => effect_cost_can_fail(e),
        CostPart::Tap
        | CostPart::Untap
        | CostPart::PayLife(_)
        | CostPart::Loyalty(_)
        | CostPart::SacrificeSelf
        | CostPart::Sacrifice { .. }
        | CostPart::DiscardSelf
        | CostPart::Discard { .. }
        | CostPart::Exile { .. }
        | CostPart::RemoveCounters { .. }
        | CostPart::RemoveCountersFromAmong { .. }
        | CostPart::TapUntapped { .. }
        | CostPart::TapTotalPower { .. }
        | CostPart::UntapTapped { .. }
        | CostPart::PayEnergy(_)
        | CostPart::PayPlayerCounters { .. }
        | CostPart::RevealFromHand { .. }
        | CostPart::RevealSelf
        | CostPart::RemoveCountersFromOne { .. }
        | CostPart::CollectEvidence(_)
        | CostPart::Forage
        | CostPart::PayManaCostOf(_)
        | CostPart::Repeated { .. }
        | CostPart::OneOf(_) => true,
    }
}

fn effect_cost_can_fail(e: &Effect) -> bool {
    match e {
        Effect::Seq(v) => v.iter().any(effect_cost_can_fail),
        Effect::GainLife { .. } => true,
        _ => false,
    }
}

/// Whether `expand_repeated` (`kw/cumulative_upkeep.rs`) scales a repeated part into one
/// part ("pay 2 life" for each age counter: pay 2N life) rather than repeating it.
fn scaled_when_repeated(p: &CostPart) -> bool {
    mtg_engine::kw::cumulative_upkeep::scaled(p, 2).is_some()
}

/// Whether a cost part discards cards (to pay a cost: an effect's discard is guarded by
/// [`Now::events`]).
fn discards(p: &CostPart) -> bool {
    match p {
        CostPart::Discard { .. } | CostPart::DiscardSelf | CostPart::DiscardHand => true,
        // The chosen one of several costs may discard (it's also checked as a payment of
        // its own).
        CostPart::OneOf(options) => options.iter().any(|c| c.parts.iter().any(discards)),
        _ => false,
    }
}

/// What a replacement effect's action does as it applies.
fn replacement_action_now(a: &ReplacementAction, now: &mut Now) {
    match a {
        ReplacementAction::MoveInstead(d) | ReplacementAction::RevealAndMoveInstead(d) => {
            match d.zone {
                ZoneKind::Library => now.lib(LibKind::Into, "puts the card into a library instead"),
                ZoneKind::Battlefield => now.events.push(Ev::Enters),
                ZoneKind::Stack => now
                    .events
                    .push(Ev::Unfollowed("puts a card onto the stack instead")),
                _ => {}
            }
        }
        ReplacementAction::AsEnters(e)
        | ReplacementAction::Instead(e)
        | ReplacementAction::Also(e)
        | ReplacementAction::PreventAndThen(_, e) => effect_now(e, now),
        ReplacementAction::Then(a, e) => {
            replacement_action_now(a, now);
            effect_now(e, now);
        }
        // The damage is dealt to another player or a permanent instead (Pariah).
        ReplacementAction::Redirect(to) | ReplacementAction::RedirectNext(to, _) => {
            if !matches!(to, Sel::Players(_) | Sel::TriggerPlayer) {
                now.events.push(Ev::DamageToPermanent);
            }
        }
        ReplacementAction::PlusTokens { spec, .. } | ReplacementAction::TokensInstead { spec } => {
            if !spec.abilities.is_empty() {
                now.events.push(Ev::Enters);
            }
        }
        ReplacementAction::TokensInsteadOf { specs, .. } => {
            if specs.iter().any(|s| !s.abilities.is_empty()) {
                now.events.push(Ev::Enters);
            }
        }
        ReplacementAction::TokenCopiesInstead { .. } => now.events.push(Ev::Enters),
        ReplacementAction::EnterTapped
        | ReplacementAction::EnterUntapped
        | ReplacementAction::EnterWithCounters(..)
        | ReplacementAction::EnterAsCopy { .. }
        | ReplacementAction::EnterUnderControl(_)
        | ReplacementAction::Prevent
        | ReplacementAction::PreventAmount(_)
        | ReplacementAction::Multiply(_)
        | ReplacementAction::Add(_)
        | ReplacementAction::Subtract(_)
        | ReplacementAction::Regenerate
        | ReplacementAction::EnterTransformed
        | ReplacementAction::LifeFloor(_)
        | ReplacementAction::ManaTypeInstead(_)
        | ReplacementAction::ManaChange(_)
        | ReplacementAction::PreventPortion(_) => {}
    }
}

/// Whether the event a replacement effect watches can happen in the middle of a payment.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reach {
    /// No: nothing in the pool causes it there.
    Never,
    /// Yes: every payment can cause it, or something in the pool causes it there.
    Payment,
    /// A library action itself (a draw, a mill, a card moved to or from a library): reported
    /// where it's caused, and not followed further.
    Library,
    /// Only a discard in the middle of a payment made outside casting a spell and
    /// activating an ability (a cost paid as an ability resolves, an attack or block cost,
    /// a special action's cost): the engine counts a discard as caused by an effect
    /// (`discard_rules::DISCARDED_BY_EFFECT`, "If an effect causes you to discard a card",
    /// Library of Leng, for a discard that isn't a cost's) or by the spell or ability whose
    /// payment it is (`ReplacementEvent::DiscardCausedBy`, "If a spell or ability an
    /// opponent controls causes you to discard a card", Nephalia Academy, whose ruling
    /// counts a choice to discard) only while no spell is being cast and no ability
    /// activated (`special.casting == 0`; mana abilities are activated too). Paired with
    /// such payments by [`discard_pairs`].
    DiscardOutsideCasting,
}

/// Whether a filter has the custom part `name` (`Filter::Custom`), alone or among others.
fn has_custom_filter(f: &Filter, name: &str) -> bool {
    match f {
        Filter::Custom(n) => n == name,
        Filter::And(v) => v.iter().any(|x| has_custom_filter(x, name)),
        _ => false,
    }
}

/// Whether the event a replacement effect watches can happen in the middle of a payment,
/// and the [`Ev`] that decides it, if one does. `has`: whether something in the pool can
/// cause an [`Ev`] there ([`reach`]).
///
/// Every payment can: move a permanent or card from the battlefield, a hand or a graveyard
/// (sacrificed, discarded, exiled, returned to a hand), deal damage to a player (a land that
/// deals 1 damage to you for mana, Overabundance's triggered mana ability), have a player
/// pay, gain or lose life, put counters, create tokens without abilities (Varchild's
/// War-Riders' cumulative upkeep), produce mana. (Cards also come back from exile there,
/// `check_untils`.)
fn event_reach(e: &ReplacementEvent, has: &dyn Fn(Ev) -> bool) -> (Reach, Option<Ev>) {
    let when = |ev: Ev| {
        if has(ev) {
            (Reach::Payment, Some(ev))
        } else {
            (Reach::Never, None)
        }
    };
    match e {
        ReplacementEvent::ZoneChange { from, to, .. } => {
            if matches!(to, Some(ZoneKind::Library)) || matches!(from, Some(ZoneKind::Library)) {
                // Into a library or out of one: a library action itself.
                (Reach::Library, None)
            } else if matches!(to, Some(ZoneKind::Battlefield)) {
                when(Ev::Enters)
            } else if matches!(to, Some(ZoneKind::Stack)) {
                // A card cast: an event the scan doesn't follow, reported where it's
                // caused ((e-assumption)).
                (Reach::Never, None)
            } else if matches!(from, Some(ZoneKind::Stack)) {
                when(Ev::Countered)
            } else {
                (Reach::Payment, None)
            }
        }
        ReplacementEvent::Dies(_) => (Reach::Payment, None),
        // "If an effect causes you to discard a card" (Library of Leng): not a discard paid
        // as a cost, wherever it's paid, and none while a spell is cast or an ability
        // activated (`discard_rules::DISCARDED_BY_EFFECT`).
        ReplacementEvent::Discard(_, f) => {
            if !has_custom_filter(f, mtg_engine::discard_rules::DISCARDED_BY_EFFECT) {
                (Reach::Payment, None)
            } else if has(Ev::DiscardByEffect) {
                (Reach::DiscardOutsideCasting, Some(Ev::DiscardByEffect))
            } else {
                (Reach::Never, None)
            }
        }
        // A cost's discard, or an effect's, outside casting and activation.
        ReplacementEvent::DiscardCausedBy { .. } => (
            Reach::DiscardOutsideCasting,
            has(Ev::DiscardByEffect).then_some(Ev::DiscardByEffect),
        ),
        // Damage to players every payment can deal; damage to a permanent only what the
        // pool makes of it; no combat damage is dealt in the middle of a payment.
        ReplacementEvent::Damage {
            to_players,
            to_objects,
            combat_only,
            ..
        } => {
            if *combat_only {
                (Reach::Never, None)
            } else if to_players.is_some() {
                (Reach::Payment, None)
            } else if to_objects.is_some() {
                when(Ev::DamageToPermanent)
            } else {
                (Reach::Never, None)
            }
        }
        ReplacementEvent::NoncombatDamage {
            to_players,
            to_objects,
            ..
        } => {
            if to_players.is_some() {
                (Reach::Payment, None)
            } else if to_objects.is_some() {
                when(Ev::DamageToPermanent)
            } else {
                (Reach::Never, None)
            }
        }
        ReplacementEvent::GainLife(_)
        | ReplacementEvent::LoseLife(_)
        | ReplacementEvent::LifeLossFromDamage(_)
        | ReplacementEvent::PutCounters { .. }
        | ReplacementEvent::PutCountersBy { .. }
        | ReplacementEvent::PutCountersMatching { .. }
        | ReplacementEvent::CreateTokens(_)
        | ReplacementEvent::CreateTokensMatching { .. }
        | ReplacementEvent::ProduceMana(_) => (Reach::Payment, None),
        // Tokens without abilities enter the battlefield in the middle of a payment; "as ~
        // enters" (the object itself) applies only to what the pool puts there.
        ReplacementEvent::EntersBattlefield(f) => {
            if matches!(f, Filter::Source) {
                when(Ev::Enters)
            } else {
                (Reach::Payment, None)
            }
        }
        ReplacementEvent::Where { event, .. } => event_reach(event, has),
        ReplacementEvent::Destroy(_) => when(Ev::Destroy),
        ReplacementEvent::Transforms => when(Ev::Transform),
        ReplacementEvent::TurnedFaceUp => when(Ev::TurnFaceUp),
        // Library actions themselves: a draw, mill, search or scry in the middle of a
        // payment is reported where it's caused.
        ReplacementEvent::Draw(_)
        | ReplacementEvent::DrawCards { .. }
        | ReplacementEvent::Mill(_)
        | ReplacementEvent::Search(_) => (Reach::Library, None),
        ReplacementEvent::Action { kind, .. } => match kind {
            ReplaceableAction::Mill
            | ReplaceableAction::Scry
            | ReplaceableAction::Explore
            | ReplaceableAction::Connive
            | ReplaceableAction::Learn => (Reach::Library, None),
            ReplaceableAction::Proliferate => when(Ev::Proliferate),
        },
        ReplacementEvent::Countered(_) => when(Ev::Countered),
        // Losing for 0 life or an empty library is a state-based action, not checked in the
        // middle of a payment; an effect's loss is [`Ev::LoseGame`].
        ReplacementEvent::LoseGame(_) => when(Ev::LoseGame),
        // Turns.
        ReplacementEvent::SkipStep { .. } | ReplacementEvent::UntapDuringUntapStep(_) => {
            (Reach::Never, None)
        }
    }
}

// ===========================================================================
// The scan
// ===========================================================================

/// One way a card could reach CR 733.1's exceptions.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Finding {
    pub card: String,
    pub ability: String,
    /// Which guarantee: "(a)" to "(e)"; "(e-assumption)" for an event the scan doesn't
    /// follow that can happen in the middle of a payment; "(scan)" for what the scan can't
    /// read or classify. Only "(e)" findings can be [`KNOWN_REACHABLE`].
    pub check: &'static str,
    pub detail: String,
}

impl std::fmt::Display for Finding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {}: {:?}: {}",
            self.check, self.card, self.ability, self.detail
        )
    }
}

/// A cost that touches a library, paid where nothing can fail afterwards.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct LibraryCost {
    pub card: String,
    pub ability: String,
    pub context: &'static str,
    /// What its library parts do.
    pub what: String,
    /// Its library parts are paid after all its other parts (by `cost_part_pays_last`, or
    /// by coming last). Either way nothing that can fail is paid after them.
    pub paid_last: bool,
}

/// A cost modifier that adds parts to the costs of activated abilities (CR 602.2b,
/// 601.2f), which the engine pays after the ability's own parts
/// (`Game::ability_total_cost_with`), or offers an alternative to them (CR 118.9,
/// `activation_costs::alternative_costs`). (The optional and chosen additional costs and
/// the costs for flash are offered for spells only: `CostChanges::add` leaves them out of
/// activation costs.)
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ActivationModifier {
    pub card: String,
    pub ability: String,
    /// A static ability's, which applies while its source is on the battlefield; otherwise
    /// a player effect an effect creates.
    pub from_static: bool,
    /// An alternative cost rather than parts added.
    pub alternative: bool,
}

/// A replacement effect (or a prevention or regeneration shield an effect creates) and
/// what its action does as it applies ((e), [`reach`]).
#[derive(Clone, Debug)]
pub struct Replacement {
    pub card: String,
    pub ability: String,
    pub event: ReplacementEvent,
    /// What its library actions are, for messages ("" if none).
    pub lib: String,
    /// What else it causes ([`Ev`]).
    pub events: Vec<Ev>,
}

/// Something a cost or a mana ability causes beyond what every payment can ((e),
/// [`reach`]).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Cause {
    pub card: String,
    pub ability: String,
    /// The cost or mana ability, for messages.
    pub what: String,
    pub ev: Ev,
}

/// "Exile [objects] until [event]" (CR 610.3): the engine returns the objects as soon as
/// the event happens, in the middle of a payment too (`until::check_untils`, which
/// `flush_events` runs as each mana ability's activation ends): to the battlefield, or to
/// the hand, graveyard or library they were exiled from ((e), [`reach`]).
#[derive(Clone, Debug)]
pub struct UntilExile {
    pub card: String,
    pub ability: String,
    pub until: UntilEvent,
    /// It can exile library cards, which go back into the library.
    pub from_library: bool,
}

#[derive(Default, Debug)]
pub struct Scan {
    pub findings: Vec<Finding>,
    pub library_costs: Vec<LibraryCost>,
    /// Discards paid outside casting and activation that something in the same action can
    /// still fail after: with a card of [`Reached::discard_replacements`], a way to reach
    /// the exceptions.
    pub late_discards: Vec<Finding>,
    pub activation_modifiers: Vec<ActivationModifier>,
    /// The rules implemented in code the card turns on by name: (what, name).
    pub customs: Vec<(String, String)>,
    /// Every replacement effect, and what it does as it applies.
    pub replacement_effects: Vec<Replacement>,
    /// What costs and mana abilities cause beyond what every payment can.
    pub causes: Vec<Cause>,
    /// The "until" exiles.
    pub untils: Vec<UntilExile>,
    /// How much was checked: abilities (nested and derived ones included), mana abilities
    /// (and mana-timing special actions), costs, replacement effects.
    pub abilities: usize,
    pub mana_abilities: usize,
    pub costs: usize,
    pub replacements: usize,
}

impl Scan {
    fn add(&mut self, r: Scan) {
        self.findings.extend(r.findings);
        self.library_costs.extend(r.library_costs);
        self.late_discards.extend(r.late_discards);
        self.activation_modifiers.extend(r.activation_modifiers);
        self.customs.extend(r.customs);
        self.replacement_effects.extend(r.replacement_effects);
        self.causes.extend(r.causes);
        self.untils.extend(r.untils);
        self.abilities += r.abilities;
        self.mana_abilities += r.mana_abilities;
        self.costs += r.costs;
        self.replacements += r.replacements;
    }

    /// Sorted, so that the pool's order doesn't change what's reported (which card
    /// [`reach`] names as first causing an event).
    fn sort(&mut self) {
        for v in [&mut self.findings, &mut self.late_discards] {
            v.sort();
            v.dedup();
        }
        self.library_costs.sort();
        self.library_costs.dedup();
        self.activation_modifiers.sort();
        self.activation_modifiers.dedup();
        self.customs.sort();
        self.customs.dedup();
        self.causes.sort();
        self.causes.dedup();
        self.replacement_effects
            .sort_by_cached_key(|r| (r.card.clone(), r.ability.clone(), format!("{:?}", r.event)));
        self.untils.sort_by_cached_key(|u| {
            (
                u.card.clone(),
                u.ability.clone(),
                format!("{:?}", u.until),
                u.from_library,
            )
        });
    }
}

/// Where a cost is paid.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum CostContext {
    /// An activated ability's cost: nothing after the payment can fail.
    Activation,
    /// A mana ability's cost.
    ManaAbility,
    /// "[Player] may pay [cost]" as an ability resolves (`Game::pay_cost`): nothing after
    /// the payment can fail (it's reversed by itself if it fails).
    Resolution,
    /// A keyword's cost the abilities it stands for don't pay: paid as a spell is cast
    /// (flashback, kicker, ...) or a special action is taken (turning a permanent face up,
    /// foretelling, ...).
    Keyword,
    /// An attack or block cost: the costs of all the attacking or blocking creatures are
    /// paid together (`combat::total_attack_cost`), so another creature's part can still
    /// fail after it.
    AttackBlock,
    /// A special action's cost; one taken during a mana payment (`mana_timing`) is part of
    /// that payment, which can still fail after it.
    SpecialAction { mana_timing: bool },
    /// Anything else (a spell's costs, the costs of modes and of play permissions, costs a
    /// modifier adds to other costs): paid as a spell is cast or an ability activated, and
    /// something can still fail after it.
    Other,
}

impl CostContext {
    fn label(self) -> &'static str {
        match self {
            CostContext::Activation => "activation cost",
            CostContext::ManaAbility => "mana ability's cost",
            CostContext::Resolution => "cost paid as an ability resolves",
            CostContext::Keyword => "keyword cost",
            CostContext::AttackBlock => "attack or block cost",
            CostContext::SpecialAction { mana_timing: false } => "special action's cost",
            CostContext::SpecialAction { mana_timing: true } => {
                "cost of a special action taken during a mana payment"
            }
            CostContext::Other => "cost",
        }
    }

    /// Whether something in the same action can still fail after the whole payment.
    fn fails_after(self) -> bool {
        matches!(
            self,
            CostContext::AttackBlock | CostContext::SpecialAction { mana_timing: true }
        )
    }

    /// Whether it's paid outside casting a spell and activating an ability, where a
    /// discard counts for [`Reach::DiscardOutsideCasting`]. (A keyword cost may be paid as
    /// a spell is cast, where it doesn't; counted anyway.)
    fn outside_casting(self) -> bool {
        matches!(
            self,
            CostContext::Resolution
                | CostContext::Keyword
                | CostContext::AttackBlock
                | CostContext::SpecialAction { .. }
        )
    }
}

/// What checking one payment found.
struct PaymentCheck {
    /// Ways it reaches the exceptions, (a) and (c).
    problems: Vec<String>,
    /// What its library parts do (its own parts, those of the payments nested in them
    /// included).
    whats: Vec<String>,
    /// What it causes beyond what every payment can ([`Ev`]).
    events: Vec<Ev>,
    /// Every library part comes after all the other parts of its payment.
    paid_last: bool,
    /// Discard parts that something can still fail after.
    late_discards: Vec<String>,
}

/// The parts of a cost as `expand_repeated` leaves them: a repeated part that isn't scaled
/// to one part is paid once for each repetition (here: twice).
fn flatten(c: &Cost, flat: &mut Vec<CostPart>) {
    for p in &c.parts {
        match p {
            CostPart::Repeated { cost, .. } => {
                let mut inner = Vec::new();
                flatten(cost, &mut inner);
                for q in inner {
                    if !scaled_when_repeated(&q) {
                        flat.push(q.clone());
                    }
                    flat.push(q);
                }
            }
            other => flat.push(other.clone()),
        }
    }
}

/// (a), (c) and the late discards of one payment, as `Game::pay_total_cost` makes it: the
/// mana first, then a check that each other part can be paid (each on its own), then the
/// parts in the order written, except that the parts that pay last
/// (`casting::cost_part_pays_last`, CR 601.2h) come after all the others. A part that's one
/// of several costs (`CostPart::OneOf`) pays the chosen one as a payment of its own, and
/// an effect that has a player pay a cost does too (`Game::pay_cost`): those are checked
/// the same way, nested. `fails_after`: something can still fail after this payment.
fn check_payment(c: &Cost, fails_after: bool, top: bool, out: &mut PaymentCheck) {
    let mut flat = Vec::new();
    flatten(c, &mut flat);
    let nows: Vec<Now> = flat.iter().map(part_lib).collect();
    if top {
        for n in &nows {
            out.events.extend(n.events.iter().copied());
            if !n.lib.is_empty() {
                out.whats.push(n.whats());
            }
        }
    }
    // The order the engine pays them in (CR 601.2h).
    let last = |p: &CostPart| mtg_engine::casting::cost_part_pays_last(p);
    let order: Vec<usize> = (0..flat.len())
        .filter(|i| !last(&flat[*i]))
        .chain((0..flat.len()).filter(|i| last(&flat[*i])))
        .collect();
    for (pos, &i) in order.iter().enumerate() {
        let part = &flat[i];
        let after = &order[pos + 1..];
        let fails_next = after.iter().copied().find(|&j| part_can_fail(&flat[j]));
        let fails_later = fails_after || fails_next.is_some();
        // The payments nested in it.
        match part {
            CostPart::OneOf(options) => {
                for o in options {
                    check_payment(o, fails_later, false, out);
                }
            }
            CostPart::Effect(_) => {
                for p in &nows[i].payments {
                    check_payment(p, fails_later, false, out);
                }
            }
            _ => {}
        }
        // A discard that something can fail after, or followed by another discard (each is
        // checked against the whole hand).
        if discards(part) {
            if let Some(&j) = after.iter().find(|&&j| discards(&flat[j])) {
                out.late_discards.push(format!(
                    "{part:?} before {:?} (each is checked against the whole hand)",
                    flat[j]
                ));
            } else if let Some(j) = fails_next {
                out.late_discards.push(format!(
                    "{part:?} before {:?}, which can still fail",
                    flat[j]
                ));
            } else if fails_after {
                out.late_discards.push(format!(
                    "{part:?}, and something in the same action can still fail after it"
                ));
            }
        }
        let lib = &nows[i].lib;
        if lib.is_empty() {
            continue;
        }
        // (a) Nothing that can fail is paid after it.
        if let Some(j) = fails_next {
            out.problems.push(format!(
                "{} ({part:?}) is paid before {:?}, which can still fail",
                nows[i].whats(),
                flat[j]
            ));
        }
        // (c) A part that searches, shuffles, reveals or moves library cards to a public
        // zone comes after every other kind of part (CR 601.2h).
        let late_kind = lib
            .iter()
            .any(|l| matches!(l.kind, LibKind::Take | LibKind::Other));
        if late_kind && !last(part) {
            if let Some(&j) = after.iter().find(|&&j| !last(&flat[j])) {
                out.problems.push(format!(
                    "{} ({part:?}) isn't paid last (`cost_part_pays_last`): {:?} comes after it",
                    nows[i].whats(),
                    flat[j]
                ));
            }
        }
    }
    let takers = nows
        .iter()
        .filter(|n| n.lib.iter().any(|l| l.kind == LibKind::Take))
        .count();
    if takers > 1 {
        out.problems.push(
            "two parts take cards from a library, and each is checked against the whole library, not jointly".into(),
        );
    }
    // Every library part comes after all the other parts.
    out.paid_last &= order.iter().enumerate().all(|(pos, &i)| {
        nows[i].lib.is_empty() || order[pos + 1..].iter().all(|&j| !nows[j].lib.is_empty())
    });
}

/// Whether a replacement effect watches its own source entering the battlefield ("As ~
/// enters").
fn self_entering(e: &ReplacementEvent) -> bool {
    match e {
        ReplacementEvent::EntersBattlefield(Filter::Source) => true,
        ReplacementEvent::Where { event, .. } => self_entering(event),
        _ => false,
    }
}

/// What an "as ~ enters" replacement effect does for a permanent that wasn't cast: what
/// enters the battlefield in the middle of a payment doesn't resolve there (it's a card an
/// "until" exile returns, or a token), so "if it was kicked" and "if you cast it"
/// (`Condition::CostPaid`, `Condition::WasCast`) don't hold, and "for each time it was
/// kicked" (`Value::TimesKicked`) is 0 (Monstrous War-Leech, Urborg Lhurgoyf).
fn not_cast(e: &Effect) -> Effect {
    fn kicks(v: &Value) -> bool {
        match v {
            Value::TimesKicked => true,
            Value::Mul(a, b) => kicks(a) || kicks(b),
            _ => false,
        }
    }
    match e {
        Effect::If {
            cond: Condition::CostPaid(_) | Condition::WasCast,
            otherwise,
            ..
        } => not_cast(otherwise),
        Effect::Seq(v) => Effect::Seq(v.iter().map(not_cast).collect()),
        Effect::Mill { n, .. } | Effect::Draw { n, .. } if kicks(n) => Effect::Noop,
        other => other.clone(),
    }
}

/// What the ability `a` targets in `slot` (its own body's targets).
fn target_kind(a: &AbilityDef, slot: u8) -> Option<&TargetKind> {
    let body = match &a.kind {
        AbilityKind::Activated(x) => &x.body,
        AbilityKind::Triggered(x) => &x.body,
        AbilityKind::Spell(x) => &x.body,
        _ => return None,
    };
    body.targets.get(slot as usize).map(|t| &t.what)
}

/// What a prevention shield's selection can protect, in the ability `a`: (players,
/// objects). A target is what the ability's targets say it is; anything else, both.
fn recipients(a: &AbilityDef, to: &Sel) -> (bool, bool) {
    match to {
        Sel::Players(_) | Sel::TriggerPlayer => (true, false),
        Sel::This
        | Sel::AttachedTo
        | Sel::AttachedToThis
        | Sel::All(_)
        | Sel::TriggerObject
        | Sel::TriggerLki
        | Sel::TriggerOtherObject
        | Sel::TriggerObjects => (false, true),
        Sel::Target(slot) => match target_kind(a, *slot) {
            Some(TargetKind::Object(_)) => (false, true),
            Some(TargetKind::Player(_)) => (true, false),
            _ => (true, true),
        },
        _ => (true, true),
    }
}

/// A replacement effect's event, with the damage it watches made exact where the ability's
/// targets say what a target is: damage that would be dealt to "target creature" isn't
/// dealt to a player (`PlayerFilter::Ref(PlayerRef::Target)` for a target that's an object),
/// nor damage that would be dealt to a target player to a permanent.
fn narrowed(a: &AbilityDef, e: &ReplacementEvent) -> ReplacementEvent {
    let not_a_player = |p: &Option<PlayerFilter>| match p {
        Some(PlayerFilter::Ref(r)) => match **r {
            PlayerRef::Target(slot) => matches!(target_kind(a, slot), Some(TargetKind::Object(_))),
            _ => false,
        },
        _ => false,
    };
    let not_an_object = |f: &Option<Filter>| match f {
        Some(Filter::In(s)) => match **s {
            Sel::Target(slot) => matches!(target_kind(a, slot), Some(TargetKind::Player(_))),
            _ => false,
        },
        _ => false,
    };
    match e {
        ReplacementEvent::Damage {
            source,
            to_players,
            to_objects,
            combat_only,
        } => ReplacementEvent::Damage {
            source: source.clone(),
            to_players: if not_a_player(to_players) {
                None
            } else {
                to_players.clone()
            },
            to_objects: if not_an_object(to_objects) {
                None
            } else {
                to_objects.clone()
            },
            combat_only: *combat_only,
        },
        ReplacementEvent::NoncombatDamage {
            source,
            to_players,
            to_objects,
        } => ReplacementEvent::NoncombatDamage {
            source: source.clone(),
            to_players: if not_a_player(to_players) {
                None
            } else {
                to_players.clone()
            },
            to_objects: if not_an_object(to_objects) {
                None
            } else {
                to_objects.clone()
            },
        },
        other => other.clone(),
    }
}

struct Scanner<'a> {
    card: &'a str,
    out: Scan,
    depth: usize,
}

impl Scanner<'_> {
    fn new(card: &str) -> Scanner<'_> {
        Scanner {
            card,
            out: Scan::default(),
            depth: 0,
        }
    }

    fn finding(&mut self, ability: &str, check: &'static str, detail: String) {
        self.out.findings.push(self.make(ability, check, detail));
    }

    fn make(&self, ability: &str, check: &'static str, detail: String) -> Finding {
        Finding {
            card: self.card.to_string(),
            ability: ability.to_string(),
            check,
            detail,
        }
    }

    /// Checks one ability and everything nested in it.
    fn ability(&mut self, a: &AbilityDef) {
        let text = a.text.as_str();
        if self.depth > 8 {
            self.finding(
                text,
                "(scan)",
                "abilities nested more than 8 deep: raise the scan's limit".into(),
            );
            return;
        }
        self.depth += 1;
        self.out.abilities += 1;
        match &a.kind {
            AbilityKind::Activated(act) => {
                if functions_in_library(act) {
                    self.finding(
                        text,
                        "(d)",
                        "activated from a library, which reveals the card (CR 602.2a) before the activation can still fail".into(),
                    );
                }
                if act.is_mana_ability {
                    self.mana_body(text, &act.body, "an activated mana ability");
                }
            }
            AbilityKind::Triggered(t) => {
                if t.is_mana_ability {
                    self.mana_body(text, &t.body, "a triggered mana ability");
                }
            }
            AbilityKind::Static(s) => match &s.effect {
                StaticEffect::Custom(n) => self.custom(text, "a static ability", n),
                StaticEffect::PlayerEffect {
                    effect: PlayerModification::Custom(n),
                    ..
                } => self.custom(text, "a player effect", n),
                _ => {}
            },
            AbilityKind::Spell(_) | AbilityKind::Keyword(_) | AbilityKind::Unsupported(_) => {}
        }
        match serde_json::to_value(&a.kind) {
            Ok(v) => self.walk(a, &v, &mut Vec::new(), true, false),
            Err(e) => self.finding(text, "(scan)", format!("can't be read: {e}")),
        }
        self.depth -= 1;
    }

    /// A rule implemented in code, turned on by name ([`custom_rule`]).
    fn custom(&mut self, ability: &str, what: &str, name: &str) {
        self.out.customs.push((what.to_string(), name.to_string()));
        match custom_rule(name) {
            CustomRule::NoLibrary => {}
            CustomRule::LibraryInstead(does) => self.finding(
                ability,
                "(e)",
                format!("a rule implemented in code ({what} {name:?}) that {does}"),
            ),
            CustomRule::SpellCost(text) => match mtg_engine::oracle::costs::parse_cost(&text) {
                Some((cost, _)) => self.payment(
                    ability,
                    &cost,
                    CostContext::Other,
                    "the cost of casting it from a graveyard",
                ),
                None => self.finding(
                    ability,
                    "(scan)",
                    format!("the cost in {name:?} can't be read"),
                ),
            },
            CustomRule::Unknown => self.finding(
                ability,
                "(scan)",
                format!(
                    "a rule implemented in code ({what} {name:?}) that `custom_rule` doesn't classify: read the hooks that look for it and classify it"
                ),
            ),
        }
    }

    /// (b) A mana ability touches no library in what it does right away; what else it
    /// causes is (e)'s.
    fn mana_body(&mut self, text: &str, body: &Body, what: &str) {
        self.out.mana_abilities += 1;
        let mut now = Now::default();
        effect_now(&body.effect, &mut now);
        if let Some(m) = &body.modal {
            for mode in &m.modes {
                effect_now(&mode.effect, &mut now);
                if let Some(c) = &mode.cost {
                    c.parts.iter().for_each(|p| part_now(p, &mut now));
                }
            }
        }
        if !now.lib.is_empty() {
            self.finding(
                text,
                "(b)",
                format!("{what} that {} as an action is attempted", now.whats()),
            );
        }
        self.causes(text, &now.events, what);
    }

    /// (e) What a payment or a mana ability causes beyond what every payment can.
    fn causes(&mut self, ability: &str, events: &[Ev], what: &str) {
        for ev in dedup_events(events) {
            self.out.causes.push(Cause {
                card: self.card.to_string(),
                ability: ability.to_string(),
                what: what.to_string(),
                ev,
            });
        }
    }

    /// Keyword abilities: the abilities they stand for, and their costs.
    fn keyword(&mut self, text: &str, kw: &Keyword) {
        let derived = mtg_engine::kw::derived(kw);
        let derived_json: Vec<String> = derived
            .iter()
            .map(|d| serde_json::to_string(&d.kind).unwrap_or_default())
            .collect();
        for d in &derived {
            self.ability(d);
        }
        // A keyword cost one of those abilities pays is checked there (cumulative upkeep's
        // trigger pays it as it resolves); any other is paid as a spell is cast or a
        // special action is taken.
        for c in kw.cost.iter().chain(&kw.costs) {
            let json = serde_json::to_string(c).unwrap_or_default();
            if !derived_json.iter().any(|d| d.contains(&json)) {
                self.payment(text, c, CostContext::Keyword, "the keyword's cost");
            }
        }
    }

    /// Finds the costs, keyword instances, nested abilities, replacement effects (prevention
    /// and regeneration shields included), "until" exiles, delayed triggered mana
    /// abilities, special actions, cost modifiers and rules implemented in code anywhere in
    /// an ability (its serialized form), with the path of fields and variants that leads to
    /// each. `in_cost`: inside a cost, whose nested costs are checked as part of it.
    fn walk(&mut self, a: &AbilityDef, v: &J, path: &mut Vec<String>, root: bool, in_cost: bool) {
        let unreadable = |what: &str, e: serde_json::Error| format!("{what} can't be read: {e}");
        match v {
            J::Array(items) => items
                .iter()
                .for_each(|x| self.walk(a, x, path, root, in_cost)),
            J::Object(m) => {
                let keys: BTreeSet<&str> = m.keys().map(String::as_str).collect();
                let is = |want: &[&str]| {
                    keys.len() == want.len() && want.iter().all(|k| keys.contains(k))
                };
                // A nested ability (granted, a token's, an emblem's): any object with an
                // ability's fields, so a new field doesn't hide them.
                if !root && ["uid", "kind", "text"].iter().all(|k| keys.contains(k)) {
                    match serde_json::from_value::<AbilityDef>(v.clone()) {
                        Ok(nested) => self.ability(&nested),
                        Err(e) => self.finding(&a.text, "(scan)", unreadable("an ability", e)),
                    }
                    return;
                }
                if is(&["kind", "n", "cost", "costs", "filter", "text", "x"]) {
                    match serde_json::from_value::<Keyword>(v.clone()) {
                        Ok(kw) => self.keyword(&a.text, &kw),
                        Err(e) => self.finding(&a.text, "(scan)", unreadable("a keyword", e)),
                    }
                    return;
                }
                let mut child_in_cost = in_cost;
                if is(&["mana", "parts"]) {
                    if !in_cost {
                        match serde_json::from_value::<Cost>(v.clone()) {
                            Ok(mut c) => {
                                let ctx = context(a, path);
                                if let (
                                    CostContext::Activation | CostContext::ManaAbility,
                                    AbilityKind::Activated(act),
                                ) = (ctx, &a.kind)
                                {
                                    // "This ability costs ... more": its own additional
                                    // costs come after its parts (conditions aside).
                                    for oc in &act.own_cost_changes {
                                        if let CostChange::AdditionalCost(extra) = &oc.change {
                                            c.parts.extend(extra.parts.iter().cloned());
                                        }
                                    }
                                }
                                self.payment(&a.text, &c, ctx, &path.join("."));
                            }
                            Err(e) => self.finding(&a.text, "(scan)", unreadable("a cost", e)),
                        }
                    }
                    // The costs nested in it (repeated, one of several, paid by an effect
                    // that's a part) are checked as part of it.
                    child_in_cost = true;
                }
                // A self-replacement effect applies only to the effect it's part of (CR
                // 614.15), which is checked wherever that effect is performed.
                if is(&["event", "action", "self_replacement", "optional"])
                    && path.last().is_none_or(|k| k != "replacement")
                {
                    match serde_json::from_value::<ReplacementDef>(v.clone()) {
                        Ok(r) => self.replacement(a, &r),
                        Err(e) => self.finding(&a.text, "(scan)", unreadable("a replacement", e)),
                    }
                }
                let special = is(&["who", "cost", "action", "mana_timing"]);
                if special {
                    match serde_json::from_value::<SpecialActionDef>(v.clone()) {
                        Ok(s) => self.special_action(&a.text, &s),
                        Err(e) => {
                            self.finding(&a.text, "(scan)", unreadable("a special action", e))
                        }
                    }
                }
                if is(&["applies_to", "who", "change"]) {
                    match serde_json::from_value::<CostModifier>(v.clone()) {
                        Ok(cm) => self.cost_modifier(a, path, &cm),
                        Err(e) => self.finding(&a.text, "(scan)", unreadable("a cost modifier", e)),
                    }
                }
                if keys.len() == 1 && keys.contains("DelayedTrigger") {
                    match serde_json::from_value::<Effect>(v.clone()) {
                        Ok(Effect::DelayedTrigger { trigger, body, .. }) => {
                            if is_triggered_mana_ability(&trigger, &body) {
                                self.mana_body(&a.text, &body, "a delayed triggered mana ability");
                            }
                        }
                        Ok(_) => {}
                        Err(e) => {
                            self.finding(&a.text, "(scan)", unreadable("a delayed trigger", e))
                        }
                    }
                }
                // "Exile ... until ..." (the engine's rule acts in the middle of a payment),
                // and the shields an effect creates (replacement effects).
                if keys.len() == 1
                    && ["ExileUntil", "PreventDamage", "RegenerateWith"]
                        .iter()
                        .any(|k| keys.contains(k))
                {
                    match serde_json::from_value::<Effect>(v.clone()) {
                        Ok(Effect::ExileUntil { what, until }) => {
                            self.out.untils.push(UntilExile {
                                card: self.card.to_string(),
                                ability: a.text.to_string(),
                                until,
                                from_library: sel_in_library(&what),
                            })
                        }
                        Ok(Effect::PreventDamage {
                            to,
                            amount,
                            combat_only,
                            then: Some(then),
                            ..
                        }) => {
                            // "Prevent the next N damage ... For each 1 damage prevented this
                            // way, [then]" (CR 615.5): a replacement effect on damage dealt to
                            // what `to` selects (`resolve.rs` makes it one).
                            let (players, objects) = recipients(a, &to);
                            let def = ReplacementDef {
                                event: ReplacementEvent::Damage {
                                    source: Filter::Any,
                                    to_players: players.then_some(PlayerFilter::Any),
                                    to_objects: objects.then_some(Filter::Any),
                                    combat_only,
                                },
                                action: ReplacementAction::PreventAndThen(amount, then),
                                self_replacement: false,
                                optional: false,
                            };
                            self.replacement(a, &def);
                        }
                        Ok(Effect::RegenerateWith { then, .. }) => {
                            // A regeneration shield replaces destruction (CR 701.19a), and
                            // what follows is performed as it does.
                            let def = ReplacementDef {
                                event: ReplacementEvent::Destroy(Filter::Any),
                                action: ReplacementAction::Then(
                                    Box::new(ReplacementAction::Regenerate),
                                    then,
                                ),
                                self_replacement: false,
                                optional: false,
                            };
                            self.replacement(a, &def);
                        }
                        Ok(_) => {}
                        Err(e) => self.finding(&a.text, "(scan)", unreadable("an effect", e)),
                    }
                }
                // Rules implemented in code: a modification ("has the creature types of
                // ...", `Modification::Custom`) and a player effect an effect creates.
                if keys.len() == 1 {
                    if let Some(J::Object(c)) = m.get("Custom") {
                        if let (Some(J::String(name)), true) =
                            (c.get("name"), c.contains_key("layer"))
                        {
                            self.custom(&a.text, "a modification", name);
                        }
                    }
                    if keys.contains("AddPlayerEffect") {
                        match serde_json::from_value::<Effect>(v.clone()) {
                            Ok(Effect::AddPlayerEffect {
                                effect: PlayerModification::Custom(n),
                                ..
                            }) => self.custom(&a.text, "a player effect", &n),
                            Ok(_) => {}
                            Err(e) => {
                                self.finding(&a.text, "(scan)", unreadable("a player effect", e))
                            }
                        }
                    }
                }
                for (k, x) in m {
                    path.push(k.clone());
                    // A special action's cost is checked by `special_action`.
                    let in_cost = child_in_cost || (special && k == "cost");
                    self.walk(a, x, path, false, in_cost);
                    path.pop();
                }
            }
            _ => {}
        }
    }

    /// (a), (c): a cost's library parts are paid where nothing can fail afterwards; the
    /// discards it pays outside casting and activation that something can fail after; and
    /// what it causes beyond what every payment can ((e)).
    fn payment(&mut self, ability: &str, c: &Cost, ctx: CostContext, where_: &str) {
        self.out.costs += 1;
        let mut check = PaymentCheck {
            problems: Vec::new(),
            whats: Vec::new(),
            events: Vec::new(),
            paid_last: true,
            late_discards: Vec::new(),
        };
        check_payment(c, ctx.fails_after(), true, &mut check);
        let events = std::mem::take(&mut check.events);
        self.causes(ability, &events, ctx.label());
        if ctx.outside_casting() {
            for d in &check.late_discards {
                let f = self.make(ability, "(e)", format!("a {} that pays {d}", ctx.label()));
                self.out.late_discards.push(f);
            }
        }
        if check.whats.is_empty() {
            return;
        }
        let what = check.whats.join(", ");
        let mut problems = check.problems;
        let check_name = match ctx {
            CostContext::ManaAbility => {
                problems.insert(0, format!("a mana ability's cost that {what}"));
                "(b)"
            }
            CostContext::Keyword => {
                problems.insert(
                    0,
                    format!(
                        "a keyword cost that {what}, paid as a spell is cast or a special action is taken: something can still fail after it"
                    ),
                );
                "(d)"
            }
            CostContext::AttackBlock | CostContext::SpecialAction { .. } | CostContext::Other => {
                problems.insert(
                    0,
                    format!(
                        "a {} ({where_}) that {what}: only activation costs and costs paid as an ability resolves have nothing that can fail after them",
                        ctx.label()
                    ),
                );
                "(a)"
            }
            CostContext::Activation | CostContext::Resolution => {
                if problems.iter().any(|p| p.contains("isn't paid last")) {
                    "(c)"
                } else {
                    "(a)"
                }
            }
        };
        if problems.is_empty() {
            self.out.library_costs.push(LibraryCost {
                card: self.card.to_string(),
                ability: ability.to_string(),
                context: ctx.label(),
                what,
                paid_last: check.paid_last,
            });
        } else {
            self.finding(ability, check_name, problems.join("; "));
        }
    }

    /// (e) A replacement effect, and what its action does as it applies: [`reach`] decides
    /// whether its event can happen in the middle of a payment.
    fn replacement(&mut self, a: &AbilityDef, r: &ReplacementDef) {
        let ability = a.text.as_str();
        self.out.replacements += 1;
        let mut now = Now::default();
        match &r.action {
            ReplacementAction::AsEnters(e) if self_entering(&r.event) => {
                effect_now(&not_cast(e), &mut now)
            }
            action => replacement_action_now(action, &mut now),
        }
        let mut event = narrowed(a, &r.event);
        if SPELL_ONLY_REPLACEMENTS
            .iter()
            .any(|(card, text)| *card == self.card && ability.contains(text))
        {
            if let ReplacementEvent::ZoneChange {
                from: from @ None, ..
            } = &mut event
            {
                *from = Some(ZoneKind::Stack);
            }
        }
        self.out.replacement_effects.push(Replacement {
            card: self.card.to_string(),
            ability: ability.to_string(),
            event,
            lib: now.whats(),
            events: dedup_events(&now.events),
        });
    }

    /// A special action's cost; (b) for one that can be taken in the middle of a mana
    /// payment.
    fn special_action(&mut self, ability: &str, s: &SpecialActionDef) {
        self.payment(
            ability,
            &s.cost,
            CostContext::SpecialAction {
                mana_timing: s.mana_timing,
            },
            "the special action's cost",
        );
        if !s.mana_timing {
            return;
        }
        self.out.mana_abilities += 1;
        let mut now = Now::default();
        s.cost.parts.iter().for_each(|p| part_now(p, &mut now));
        if let SpecialActionEffect::Effect(e) = &s.action {
            effect_now(e, &mut now);
        }
        if !now.lib.is_empty() {
            self.finding(
                ability,
                "(b)",
                format!(
                    "a special action taken during a mana payment that {}",
                    now.whats()
                ),
            );
        }
        self.causes(
            ability,
            &now.events,
            "a special action taken during a mana payment",
        );
    }

    /// A cost modifier that adds parts to activation costs, or offers an alternative to
    /// them (paired with the activation costs that touch a library by
    /// [`with_activation_modifiers`]).
    fn cost_modifier(&mut self, a: &AbilityDef, path: &[String], cm: &CostModifier) {
        let abilities = match cm.applies_to {
            CostTarget::Spells(_) | CostTarget::ThisSpell => false,
            CostTarget::Abilities(_)
            | CostTarget::Keyword(_)
            | CostTarget::KeywordAbilitiesOf(..)
            | CostTarget::LoyaltyAbilities(_)
            | CostTarget::ActivatedAbilities(_) => true,
        };
        // What `CostChanges::add` adds to an activation cost, and what
        // `activation_costs::alternative_costs` offers instead of one.
        let alternative = match &cm.change {
            CostChange::AdditionalCost(c) if !c.parts.is_empty() => false,
            CostChange::AlternativeCost(_) => true,
            _ => return,
        };
        if !abilities {
            return;
        }
        let from_static = matches!(a.kind, AbilityKind::Static(_))
            && path
                .iter()
                .map(String::as_str)
                .eq(["Static", "effect", "CostModifier"]);
        self.out.activation_modifiers.push(ActivationModifier {
            card: self.card.to_string(),
            ability: a.text.clone(),
            from_static,
            alternative,
        });
    }
}

/// The context of a cost found at `path` in the ability `a`.
fn context(a: &AbilityDef, path: &[String]) -> CostContext {
    let tail: Vec<&str> = path.iter().rev().take(2).map(String::as_str).collect();
    match (tail.as_slice(), &a.kind) {
        (["cost", "Activated"], AbilityKind::Activated(act)) if path.len() == 2 => {
            if act.is_mana_ability {
                CostContext::ManaAbility
            } else {
                CostContext::Activation
            }
        }
        (["cost", "PayOptional"], _) => CostContext::Resolution,
        (["cost", "AttackCost" | "BlockCost" | "BlockCostAgainst" | "OptionalAttackCost"], _) => {
            CostContext::AttackBlock
        }
        _ => CostContext::Other,
    }
}

/// Whether an activated ability can be activated while its card is in a library: it
/// functions there (CR 113.6), and its activation condition doesn't keep the card in
/// other zones ("Activate only if ~ is on the battlefield or in your graveyard").
fn functions_in_library(act: &ActivatedAbility) -> bool {
    let zone = match act.zone {
        FunctionZone::Library | FunctionZone::Anywhere => true,
        FunctionZone::AnywhereExcept(z) => z != ZoneKind::Library,
        _ => false,
    };
    zone && act
        .condition
        .as_ref()
        .and_then(source_zones)
        .is_none_or(|z| z.contains(&ZoneKind::Library))
}

/// The zones a condition requires the ability's source to be in, if it requires any
/// (`None`: any zone, as far as the scan can tell).
fn source_zones(c: &Condition) -> Option<BTreeSet<ZoneKind>> {
    fn filter_zone(f: &Filter) -> Option<ZoneKind> {
        match f {
            Filter::InZone(z) => Some(*z),
            Filter::And(v) => v.iter().find_map(filter_zone),
            _ => None,
        }
    }
    match c {
        Condition::SelMatches(Sel::This, f) => filter_zone(f).map(|z| BTreeSet::from([z])),
        Condition::Or(v) => {
            let mut all = BTreeSet::new();
            for x in v {
                all.extend(source_zones(x)?);
            }
            Some(all)
        }
        Condition::And(v) => v.iter().filter_map(source_zones).reduce(|a, b| &a & &b),
        _ => None,
    }
}

/// Scans a card's compiled abilities (every face) for ways to reach CR 733.1's exceptions
/// on its own, and records what [`reach`] follows over a pool.
pub fn scan(def: &CardDef) -> Scan {
    let mut s = Scanner::new(def.name.as_str());
    for face in &def.faces {
        for a in &face.chars.abilities {
            s.ability(a);
        }
    }
    s.out.sort();
    s.out
}

// ===========================================================================
// What a pool causes in the middle of a payment (e)
// ===========================================================================

/// What the cards of a scan can cause together in the middle of a payment, and the ways to
/// reach the exceptions that follow ((e)).
pub struct Reached {
    /// The events beyond what every payment can cause ([`Ev`]) that something in the pool
    /// causes in the middle of a payment, each with what causes it first.
    pub events: BTreeMap<Ev, String>,
    /// "(e)": a replacement effect or rule that turns something that can happen there into a
    /// library action; "(e-assumption)": an event the scan doesn't follow that can happen
    /// there.
    pub findings: Vec<Finding>,
    /// Replacement effects that turn a discard paid outside casting and activation into a
    /// library action ([`Reach::DiscardOutsideCasting`]).
    pub discard_replacements: Vec<Finding>,
}

/// Whether the event that ends an "until" effect can happen in the middle of a payment.
fn until_can_end(u: &UntilEvent, has: &dyn Fn(Ev) -> bool) -> bool {
    match u {
        // A permanent leaves the battlefield in the middle of a payment when it's
        // sacrificed for mana (Ashnod's Altar).
        UntilEvent::SourceLeavesBattlefield | UntilEvent::TargetLeavesBattlefield(_) => true,
        UntilEvent::OpponentBecomesMonarch => has(Ev::BecomesMonarch),
    }
}

/// (e) over a pool, as a fixpoint: what costs and mana abilities cause ([`Scan::causes`]),
/// then what the replacement effects that apply in the middle of a payment cause there, and
/// what `until::check_untils` does there, until nothing more can happen there. A
/// replacement effect that does a library action is reported, and what it causes after
/// isn't followed: what follows a library action is reported where that library action is.
pub fn reach(t: &Scan) -> Reached {
    let mut events: BTreeMap<Ev, String> = BTreeMap::new();
    for c in &t.causes {
        events
            .entry(c.ev)
            .or_insert_with(|| format!("{} ({}: {:?})", c.card, c.what, c.ability));
    }
    loop {
        let mut more: Vec<(Ev, String)> = Vec::new();
        {
            let has = |ev: Ev| events.contains_key(&ev);
            for r in &t.replacement_effects {
                if !r.lib.is_empty() {
                    continue;
                }
                if matches!(
                    event_reach(&r.event, &has).0,
                    Reach::Payment | Reach::DiscardOutsideCasting
                ) {
                    for ev in &r.events {
                        more.push((
                            *ev,
                            format!("{}'s replacement effect ({:?})", r.card, r.ability),
                        ));
                    }
                }
            }
            for u in &t.untils {
                if !u.from_library && until_can_end(&u.until, &has) {
                    more.push((
                        Ev::Enters,
                        format!(
                            "{}'s exile until {:?} ({:?}), which ends in the middle of a payment (`until::check_untils`)",
                            u.card, u.until, u.ability
                        ),
                    ));
                }
            }
        }
        let before = events.len();
        for (ev, by) in more {
            events.entry(ev).or_insert(by);
        }
        if events.len() == before {
            break;
        }
    }
    let has = |ev: Ev| events.contains_key(&ev);
    let mut findings = Vec::new();
    let mut discard_replacements = Vec::new();
    for r in &t.replacement_effects {
        let (reach, via) = event_reach(&r.event, &has);
        if matches!(reach, Reach::Never | Reach::Library) {
            continue;
        }
        let through = via
            .map(|ev| format!(" ({}, which {} causes there)", ev.describe(), events[&ev]))
            .unwrap_or_default();
        let finding = |check: &'static str, detail: String| Finding {
            card: r.card.clone(),
            ability: r.ability.clone(),
            check,
            detail,
        };
        if !r.lib.is_empty() {
            match reach {
                // An effect's discard there: what makes one (a card an "until" exile returns
                // entering with "As ~ enters, discard ..."), and whether something in such a
                // payment can still fail after it, is more than the scan follows; counted.
                Reach::DiscardOutsideCasting if via == Some(Ev::DiscardByEffect) => {
                    findings.push(finding(
                        "(e)",
                        format!(
                            "a replacement effect that {} applies to a discard an effect makes in the middle of a payment made outside casting and activation{through}",
                            r.lib
                        ),
                    ))
                }
                Reach::DiscardOutsideCasting => discard_replacements.push(finding(
                    "(e)",
                    format!(
                        "a replacement effect that {} applies to a discard paid outside casting and activation",
                        r.lib
                    ),
                )),
                _ => findings.push(finding(
                    "(e)",
                    format!(
                        "a replacement effect that {} applies to an event that can happen in the middle of a payment{through}",
                        r.lib
                    ),
                )),
            }
            continue;
        }
        for ev in &r.events {
            if let Ev::Unfollowed(s) = ev {
                findings.push(finding(
                    "(e-assumption)",
                    format!(
                        "a replacement effect that {s} applies in the middle of a payment{through}: the scan doesn't follow that"
                    ),
                ));
            }
        }
    }
    for u in &t.untils {
        if u.from_library && until_can_end(&u.until, &has) {
            findings.push(Finding {
                card: u.card.clone(),
                ability: u.ability.clone(),
                check: "(e)",
                detail: format!(
                    "exiles library cards until {:?}, which can happen in the middle of a payment: `until::check_untils` then puts them back into the library",
                    u.until
                ),
            });
        }
    }
    for c in &t.causes {
        if let Ev::Unfollowed(s) = c.ev {
            findings.push(Finding {
                card: c.card.clone(),
                ability: c.ability.clone(),
                check: "(e-assumption)",
                detail: format!(
                    "a {} that {s} as an action is attempted: the scan doesn't follow that",
                    c.what
                ),
            });
        }
    }
    findings.sort();
    findings.dedup();
    discard_replacements.sort();
    discard_replacements.dedup();
    Reached {
        events,
        findings,
        discard_replacements,
    }
}

/// (e) for the discard replacements ([`Reached::discard_replacements`]): with one of them
/// around, a payment made outside casting and activation reaches the exceptions if a
/// discard it pays ([`Scan::late_discards`]) can still be followed by a failure. Either
/// list alone doesn't.
fn discard_pairs(t: &Scan, r: &Reached) -> Vec<Finding> {
    if r.discard_replacements.is_empty() {
        return Vec::new();
    }
    let with: Vec<&str> = r
        .discard_replacements
        .iter()
        .map(|f| f.card.as_str())
        .collect();
    let mut v: Vec<Finding> = t
        .late_discards
        .iter()
        .map(|d| Finding {
            detail: format!(
                "{}; with {} around, the discard is a library action",
                d.detail,
                with.join(" or ")
            ),
            ..d.clone()
        })
        .collect();
    v.sort();
    v.dedup();
    v
}

/// (a) with cost modifiers: the engine adds the parts of an activation cost modifier
/// after an ability's own parts (`Game::ability_total_cost_with`), and an alternative cost
/// replaces them (with the parts a keyword adds after it, `activation_costs::with_alternative`),
/// so a library part of an activation cost could come before a part that can fail. Each
/// activation cost that touches a library is checked again, in a game, with each static
/// modifier's source under either player; a modifier an effect creates is reported with
/// each of them.
fn with_activation_modifiers(t: &Scan) -> Vec<Finding> {
    let costs: Vec<&LibraryCost> = t
        .library_costs
        .iter()
        .filter(|c| c.context == CostContext::Activation.label())
        .collect();
    let mut out = Vec::new();
    for m in &t.activation_modifiers {
        let kind = if m.alternative {
            "alternative cost"
        } else {
            "cost modifier"
        };
        for c in &costs {
            if !m.from_static {
                out.push(Finding {
                    card: c.card.clone(),
                    ability: c.ability.clone(),
                    check: "(a)",
                    detail: format!(
                        "{} ({:?}) creates a player effect with an activation {kind}, which the scan can't place in a game: check this cost with it",
                        m.card, m.ability
                    ),
                });
                continue;
            }
            for owner in [P0, P1] {
                let mut g = TestGame::new(2);
                let src = g.battlefield(P0, &c.card);
                g.battlefield(owner, &m.card);
                g.g.recompute();
                let found =
                    g.g.obj(src)
                        .chars
                        .abilities
                        .iter()
                        .find(|a| a.text == c.ability)
                        .cloned();
                let Some(a) = found else {
                    out.push(Finding {
                        card: c.card.clone(),
                        ability: c.ability.clone(),
                        check: "(scan)",
                        detail: format!(
                            "the ability isn't on the card in a game, to check with {}'s {kind}",
                            m.card
                        ),
                    });
                    continue;
                };
                let AbilityKind::Activated(act) = &a.kind else {
                    continue;
                };
                let totals: Vec<Cost> = if m.alternative {
                    mtg_engine::activation_costs::alternative_costs(&g.g, P0, src, &a, act)
                        .into_iter()
                        .map(|(_, alt)| {
                            g.g.ability_total_cost_with(P0, src, &a, act, None, 0, Some(&alt))
                        })
                        .collect()
                } else {
                    vec![g.g.ability_total_cost_with(P0, src, &a, act, None, 0, None)]
                };
                for total in totals {
                    let mut s = Scanner::new(&c.card);
                    s.payment(
                        &c.ability,
                        &total,
                        CostContext::Activation,
                        "with a cost modifier",
                    );
                    out.extend(s.out.findings.into_iter().map(|f| Finding {
                        detail: format!("with {}'s {kind}: {}", m.card, f.detail),
                        ..f
                    }));
                }
            }
        }
    }
    out.sort();
    out.dedup();
    out
}

/// Every way the cards of a scan reach the exceptions together: each card's own findings,
/// (e) over them as a pool, the discard pairs and the activation cost modifiers.
fn all_findings(t: &Scan) -> Vec<Finding> {
    let r = reach(t);
    let mut v = t.findings.clone();
    v.extend(discard_pairs(t, &r));
    v.extend(r.findings);
    v.extend(with_activation_modifiers(t));
    v.sort();
    v.dedup();
    v
}

/// The ways fully supported cards reach CR 733.1's exceptions (e): (card, a fragment of the
/// ability's text). A replacement effect (or a rule implemented in code) that makes
/// something that can happen in the middle of a payment a library action; a cast or
/// activation in which it applies and that then turns out illegal is reversed, and the
/// reversal keeps that library action (`reversal::keep_library_actions`; the
/// `a_reversed_cast_keeps_` tests). The register entry lists them; an "(e)" finding no entry matches, or an
/// entry no finding matches, fails the scan.
pub const KNOWN_REACHABLE: &[(&str, &str)] = &[
    // A permanent or card a payment moves goes to a library instead: sacrificed for a
    // cost or a mana ability (Ashnod's Altar), discarded, dying.
    (
        "Blightsteel Colossus",
        "shuffle it into its owner's library instead",
    ),
    (
        "Darksteel Colossus",
        "shuffle it into its owner's library instead",
    ),
    (
        "Gravebane Zombie",
        "put it on top of its owner's library instead",
    ),
    (
        "Legacy Weapon",
        "shuffle it into its owner's library instead",
    ),
    (
        "Nexus of Fate",
        "shuffle it into its owner's library instead",
    ),
    (
        "Nissa's Chosen",
        "put it on the bottom of its owner's library instead",
    ),
    ("Progenitus", "shuffle it into its owner's library instead"),
    (
        "Pulmonic Sliver",
        "you may put it on top of its owner's library instead",
    ),
    (
        "Wheel of Sun and Moon",
        "put on the bottom of that player's library",
    ),
    // Life paid while an action is attempted becomes an exile from a library (a rule
    // implemented in code, `kw::pay_life_instead`).
    ("Ashiok, Wicked Manipulator", "If you would pay life"),
    // Damage a land deals to its controller for mana (Shivan Reef), or a triggered mana
    // ability deals to the player who tapped a land (Overabundance), becomes a mill or an
    // exile from a library.
    (
        "Angel of Suffering",
        "prevent that damage and mill twice that many cards",
    ),
    ("Bone Mask", "Exile cards from the top of your library"),
    (
        "Crumbling Sanctuary",
        "exiles that many cards from the top of their library",
    ),
    ("The Mindskinner", "each opponent mills that many cards"),
    // Life a mana ability gains (Pristine Talisman) becomes a draw.
    (
        "Lich",
        "If you would gain life, draw that many cards instead",
    ),
    (
        "Nefarious Lich",
        "If you would gain life, draw that many cards instead",
    ),
    // Through what other replacement effects and rules make of a payment: the damage
    // Pariah, Pariah's Shield or With Great Power deals to a permanent instead becomes a
    // draw; the loss Nefarious Lich (or Enduring Angel) makes of damage or life loss
    // becomes a shuffle and seven draws; a creature an "until" exile returns when its
    // source is sacrificed for mana (Banisher Priest) is shuffled into a library.
    (
        "Swans of Bryn Argoll",
        "The source's controller draws cards",
    ),
    ("Lich's Mirror", "If you would lose the game"),
    // The same loss, made into seven draws (found when the scan first ran on a newer
    // card pool, Oct 22).
    (
        "Stunning Reversal",
        "The next time you would lose the game this turn",
    ),
    ("Don't Blink", "shuffle them into their libraries instead"),
    // A discard an effect makes in the middle of a payment made outside casting and
    // activation goes on top of a library instead: Heightened Awareness ("As ~ enters,
    // discard your hand") or Indominus Rex, Alpha returned by an "until" exile there.
    // Counted: whether such a payment can return one before something in it fails is
    // more than the scan follows ([`Reach::DiscardOutsideCasting`]).
    (
        "Library of Leng",
        "If an effect causes you to discard a card",
    ),
    (
        "Nephalia Academy",
        "If a spell or ability an opponent controls causes you to discard a card",
    ),
];

/// The events beyond what every payment can cause that fully supported cards cause in the
/// middle of a payment ([`reach`]): the register entry describes them. A change fails the
/// scan, so that the entry stays accurate.
const KNOWN_EVENTS: &[Ev] = &[
    // Cards an "until" exile returns when its source is sacrificed for mana (Banisher
    // Priest); as the scan counts it, tokens with abilities a replacement effect creates
    // instead of others (Academy Manufactor).
    Ev::Enters,
    // A creature dealt damage destroyed instead (Kill-Suit Cultist), once damage can be
    // dealt to one.
    Ev::Destroy,
    // Enduring Angel transforms instead of its controller's life reaching 0.
    Ev::Transform,
    // A player's damage dealt to a permanent instead (Pariah, Pariah's Shield, With Great
    // Power . . .).
    Ev::DamageToPermanent,
    // Nefarious Lich's loss for damage it can't exile cards for; Enduring Angel's.
    Ev::LoseGame,
    // "As ~ enters, discard your hand" (Heightened Awareness) for a card an "until" exile
    // returns.
    Ev::DiscardByEffect,
];

/// The cards whose replacement effects turn a discard into a library action only when the
/// discard is a cost's, paid outside casting and activation ([`Reach::DiscardOutsideCasting`]):
/// they'd reach the exceptions with a cost paid as an ability resolves, an attack or block
/// cost or a special action's cost in which something can fail after a discard, and no
/// fully supported card has one ([`discard_pairs`]). None today: Nephalia Academy ("If a
/// spell or ability an opponent controls causes you to discard a card", which its ruling
/// says a choice to discard is) would be one, but an effect can discard a card in the
/// middle of a payment ([`Ev::DiscardByEffect`]), so it's counted with Library of Leng in
/// [`KNOWN_REACHABLE`]. (Library of Leng's "If an effect causes you to discard a card"
/// doesn't apply to a cost's discard, wherever it's paid,
/// `discard_rules::DISCARDED_BY_EFFECT`.)
const KNOWN_DISCARD_REPLACEMENTS: &[&str] = &[];

/// Replacement effects whose event is about a spell, though the compiled event doesn't say
/// so: the replacement an effect creates for "that spell", the spell it has just cast. A
/// spell is put into a graveyard only from the stack, so the scan reads the event as one
/// from the stack (`Ev::Countered`). Read each card before listing it. (card, a fragment of
/// the ability's text)
const SPELL_ONLY_REPLACEMENTS: &[(&str, &str)] = &[(
    // "you may cast an instant or sorcery spell from among cards exiled with it. If that
    // spell would be put into a graveyard, put it on the bottom of its owner's library
    // instead."
    "Kylox's Voltstrider",
    "If that spell would be put into a graveyard",
)];

/// The fully supported cards in scope on Oct 21 (see the register entry): the scan fails
/// if far fewer are scanned, so a broken card iteration or compiler can't pass it.
const SUPPORTED_ON_OCT_21: usize = 31_456;

/// Library-touching costs the scan must keep seeing, as safe and paid after all their
/// other parts (the register entry lists them): if a change to the AST hides them from the
/// scan, it fails. (card, a fragment of the ability's text)
const KNOWN_LIBRARY_COSTS: &[(&str, &str)] = &[
    // Exiling the top cards of a library (paid last, CR 601.2h).
    ("Arc-Slogger", "Exile the top ten cards"),
    ("Royal Herbalist", "Exile the top card"),
    ("Seasoned Tactician", "Exile the top four cards"),
    ("Whirling Catapult", "Exile the top two cards"),
    ("Thought Lash", "Exile the top card"),
    // Milling (paid last); the mill costs keep these from being mana abilities (605.1a).
    ("Rot Farm Skeleton", "Mill four cards"),
    ("Sinister Concoction", "Mill a card"),
    ("Millikin", "Mill a card"),
    ("Deranged Assistant", "Mill a card"),
    ("The Warring Triad", "Mill a card"),
    // Putting cards on a library: from a hand, from a graveyard, the source itself.
    (
        "Penance",
        "Put a card from your hand on top of your library",
    ),
    (
        "Leashling",
        "Put a card from your hand on top of your library",
    ),
    (
        "Hidden Retreat",
        "Put a card from your hand on top of your library",
    ),
    (
        "Battlefield Scrounger",
        "Put three cards from your graveyard",
    ),
    (
        "Timestream Navigator",
        "Put ~ on the bottom of its owner's library",
    ),
    // Costs paid as an ability resolves.
    (
        "Gurzigost",
        "put two cards from your graveyard on the bottom",
    ),
    (
        "Anurid Scavenger",
        "put a card from your graveyard on the bottom",
    ),
    ("Deep Spawn", "mill two cards"),
    ("Thought Lash", "Cumulative Upkeep"),
    ("Psychic Vortex", "Cumulative Upkeep"),
    (
        "Tainted Specter",
        "put a card from their hand on top of their library",
    ),
    ("Decoy Gambit", "has you draw a card"),
    ("Research // Development", "has you draw a card"),
];

/// Whether an "(e)" finding is one of [`KNOWN_REACHABLE`].
fn known_reachable(f: &Finding) -> bool {
    f.check == "(e)"
        && KNOWN_REACHABLE
            .iter()
            .any(|(card, text)| f.card == *card && f.ability.contains(text))
}

// ===========================================================================
// The scan over every fully supported card
// ===========================================================================

/// The cards the scan covers: playable cards legal in some format, sticker sheets aside.
/// (`mtg-tools`'s coverage also requires a paper printing, judged by the one printing the
/// Oracle data lists: that leaves out paper cards such as Lotus Petal and Thought Lash
/// whose listed printing is an MTGO one.) A card legal in no format is in no card pool
/// (random decks, the properties, the API's): Under-Construction Skyscraper is fully
/// supported, and its LEVEL 8+ mana ability scries, but it's outside the scan with them.
fn cards_in_scope() -> Vec<&'static mtg_data::scryfall::ScryfallCard> {
    let mut seen = BTreeSet::new();
    mtg_data::cards()
        .iter()
        .filter(|c| c.is_playable_card() && c.is_legal_somewhere() && !c.is_sticker_sheet())
        .filter(|c| seen.insert(c.name.clone()))
        .collect()
}

struct FullScan {
    cards: usize,
    supported: usize,
    total: Scan,
}

/// Compiles every card in scope (on a few threads: compiling them all takes about two
/// minutes on one in a debug build; no other test compiles every card to share them with)
/// and scans the fully supported ones.
fn full_scan() -> FullScan {
    use std::sync::atomic::{AtomicUsize, Ordering};
    let cards = cards_in_scope();
    let next = AtomicUsize::new(0);
    let supported = AtomicUsize::new(0);
    let total = Mutex::new(Scan::default());
    let threads = std::thread::available_parallelism().map_or(2, |n| n.get().min(4));
    std::thread::scope(|s| {
        for _ in 0..threads {
            s.spawn(|| loop {
                let i = next.fetch_add(1, Ordering::Relaxed);
                let Some(c) = cards.get(i) else { break };
                let def = CardDef::from_scryfall(c);
                if !def.is_fully_supported() {
                    continue;
                }
                supported.fetch_add(1, Ordering::Relaxed);
                let r = scan(&def);
                total.lock().unwrap().add(r);
            });
        }
    });
    let mut total = total.into_inner().unwrap();
    total.sort();
    FullScan {
        cards: cards.len(),
        supported: supported.into_inner(),
        total,
    }
}

#[test]
fn every_fully_supported_card_is_scanned_for_ways_to_reach_the_exceptions() {
    interp!("illegal-action-reversal-restores-libraries");
    let r = full_scan();
    let t = &r.total;
    eprintln!(
        "{} cards in scope, {} fully supported: {} abilities, {} mana abilities, {} costs, {} replacement effects, {} \"until\" exiles, {} rules implemented in code checked",
        r.cards,
        r.supported,
        t.abilities,
        t.mana_abilities,
        t.costs,
        t.replacements,
        t.untils.len(),
        t.customs.len()
    );
    for c in &t.library_costs {
        eprintln!(
            "library cost: {} ({}): {}: {}",
            c.card, c.context, c.ability, c.what
        );
    }
    for m in &t.activation_modifiers {
        eprintln!(
            "activation cost modifier: {} ({}{}): {}",
            m.card,
            if m.from_static { "static" } else { "effect" },
            if m.alternative { ", alternative" } else { "" },
            m.ability
        );
    }
    for (what, name) in &t.customs {
        eprintln!("rule implemented in code: {what} {name:?}");
    }
    let reached = reach(t);
    for (ev, by) in &reached.events {
        eprintln!(
            "can happen in the middle of a payment: {} (first through {by})",
            ev.describe()
        );
    }
    for f in reached.discard_replacements.iter().chain(&t.late_discards) {
        eprintln!("discard: {f}");
    }
    for f in t.findings.iter().chain(&reached.findings) {
        eprintln!("finding: {f}");
    }
    // The scan saw the pool (a broken card iteration or compiler would pass vacuously):
    // more cards become supported over time; far fewer means something broke.
    assert!(
        r.supported >= SUPPORTED_ON_OCT_21 * 98 / 100,
        "only {} fully supported cards (there were {SUPPORTED_ON_OCT_21} on Oct 21)",
        r.supported
    );
    // Every way a fully supported card can reach the exceptions is a known one. Only (e)
    // can be: the discard pairs and the activation cost modifiers never are.
    let mut unexpected: Vec<String> = t
        .findings
        .iter()
        .chain(&reached.findings)
        .filter(|f| !known_reachable(f))
        .map(|f| f.to_string())
        .collect();
    unexpected.extend(
        discard_pairs(t, &reached)
            .iter()
            .chain(&with_activation_modifiers(t))
            .map(|f| f.to_string()),
    );
    assert!(
        unexpected.is_empty(),
        "fully supported cards that reach CR 733.1's exceptions in a way not known: a reversal keeps these library actions (register entry `illegal-action-reversal-restores-libraries`), and each way should be known and tested. For a cost, mana ability or other path (a)-(d), make the engine keep it out of reach (a cost's library part paid last); for a replacement effect or rule (e), list the card and ability in KNOWN_REACHABLE and in the entry, with a game test where the reversal keeps it; or, if the scan misreads the card, correct its classification (this file's module docs):\n{}",
        unexpected.join("\n")
    );
    let stale: Vec<&(&str, &str)> = KNOWN_REACHABLE
        .iter()
        .filter(|(card, text)| {
            !t.findings
                .iter()
                .chain(&reached.findings)
                .any(|f| f.check == "(e)" && f.card == *card && f.ability.contains(text))
        })
        .collect();
    assert!(
        stale.is_empty(),
        "no longer found by the scan (remove them from KNOWN_REACHABLE and the register entry): {stale:?}"
    );
    // What the pool can cause in the middle of a payment is what the entry says.
    assert_eq!(
        reached.events.keys().copied().collect::<Vec<_>>(),
        KNOWN_EVENTS,
        "what fully supported cards can cause in the middle of a payment changed: update KNOWN_EVENTS and the register entry ({:?})",
        reached.events
    );
    // The discard replacements the entry names are the ones the scan sees.
    let discard_cards: BTreeSet<&str> = reached
        .discard_replacements
        .iter()
        .map(|f| f.card.as_str())
        .collect();
    assert_eq!(
        discard_cards,
        KNOWN_DISCARD_REPLACEMENTS.iter().copied().collect(),
        "the cards whose replacement effects turn a discard paid outside casting and activation into a library action changed: update KNOWN_DISCARD_REPLACEMENTS and the register entry"
    );
    // The library-touching costs the entry names are still seen, safe, and paid after
    // all their other parts.
    for (card, text) in KNOWN_LIBRARY_COSTS {
        let c = t
            .library_costs
            .iter()
            .find(|c| c.card == *card && c.ability.contains(text));
        assert!(
            c.is_some_and(|c| c.paid_last),
            "{card} ({text}): the scan no longer sees this library cost, or doesn't find it paid last: {:?}",
            t.library_costs.iter().filter(|c| c.card == *card).collect::<Vec<_>>()
        );
    }
}

// ===========================================================================
// The scan finds each way to reach the exceptions
// ===========================================================================

fn named(text: &str, kind: AbilityKind) -> Ability {
    AbilityDef::new(kind, text)
}

fn activated(text: &str, parts: Vec<CostPart>, effect: Effect, mana: bool) -> Ability {
    let mut a = ActivatedAbility::new(Cost { mana: None, parts }, Body::effect(effect));
    a.is_mana_ability = mana;
    named(text, AbilityKind::Activated(a))
}

fn static_ability(text: &str, effect: StaticEffect) -> Ability {
    named(text, AbilityKind::Static(StaticAbility::new(effect)))
}

fn spell(text: &str, effect: Effect) -> Ability {
    named(
        text,
        AbilityKind::Spell(SpellAbility {
            body: Body::effect(effect),
        }),
    )
}

fn add_g() -> Effect {
    Effect::AddMana {
        who: PlayerRef::You,
        mana: ManaProduction::Fixed(vec![ManaType::G]),
        restriction: None,
    }
}

fn gain_1() -> Effect {
    Effect::GainLife {
        who: PlayerRef::You,
        n: Value::c(1),
    }
}

fn draw_1() -> Effect {
    Effect::Draw {
        who: PlayerRef::You,
        n: Value::c(1),
    }
}

fn mill_1() -> Effect {
    Effect::Mill {
        who: PlayerRef::You,
        n: Value::c(1),
    }
}

fn exile_top() -> CostPart {
    CostPart::Effect(Box::new(Effect::Move {
        what: Sel::TopOfLibrary(PlayerRef::You, Value::c(1)),
        to: Destination::zone(ZoneKind::Exile),
    }))
}

fn sacrifice_a_creature() -> CostPart {
    CostPart::Sacrifice {
        filter: Filter::creature(),
        count: Value::c(1),
    }
}

fn hand_on_top() -> CostPart {
    CostPart::PutFromHandOnLibrary {
        filter: Filter::Any,
        count: Value::c(1),
        top: true,
    }
}

/// The checks the scan reports for one ability on a custom card.
fn reported(ability: Ability) -> Vec<&'static str> {
    checks(&scan(&custom_with(
        "Probe",
        "Artifact",
        None,
        vec![ability],
    )))
}

#[test]
fn the_scan_reports_each_way_to_reach_the_exceptions() {
    interp!("illegal-action-reversal-restores-libraries");
    // Each case is reported under the check it's about (a special action's mill cost is
    // also a cost outside activation and resolution, (a)).
    let reports = |checks: Vec<&'static str>, want: &str, case: &str| {
        assert!(
            checks.contains(&want),
            "{case}: expected {want}, got {checks:?}"
        );
    };
    // (a) A library part paid before a part that can still fail.
    reports(
        reported(activated(
            "Put a card from your hand on top of your library, Sacrifice a creature: You gain 1 life.",
            vec![hand_on_top(), sacrifice_a_creature()],
            gain_1(),
            false,
        )),
        "(a)",
        "library part before a sacrifice",
    );
    // (a) Two parts that take cards from a library: each is checked against the whole
    // library, so one card could pay neither in full.
    reports(
        reported(activated(
            "Exile the top card of your library, Mill a card: You gain 1 life.",
            vec![exile_top(), CostPart::Mill(Value::c(1))],
            gain_1(),
            false,
        )),
        "(a)",
        "two library parts",
    );
    // (a) A spell's additional cost that mills (the cast can still fail after it).
    let toll = custom_card(
        "Library Toll",
        "Sorcery",
        None,
        "As an additional cost to cast this spell, mill two cards.\nDraw a card.",
    );
    reports(checks(&scan(&toll)), "(a)", "a spell's additional cost");
    // (a) A cost paid as an ability resolves, with a library part before a part that can
    // fail.
    let upkeep = TriggeredAbility::new(
        TriggerCond::BeginningOf {
            step: TriggerStep::Upkeep,
            whose: PlayerRel::You,
        },
        Body::effect(Effect::PayOptional {
            who: PlayerRef::You,
            cost: Cost {
                mana: None,
                parts: vec![hand_on_top(), sacrifice_a_creature()],
            },
            then: Box::new(Effect::Noop),
            otherwise: Box::new(Effect::SacrificeObjects { what: Sel::This }),
        }),
    );
    reports(
        reported(named(
            "At the beginning of your upkeep, sacrifice ~ unless you put a card from your hand on top of your library and sacrifice a creature.",
            AbilityKind::Triggered(upkeep),
        )),
        "(a)",
        "a resolution cost",
    );
    // (b) A mana ability that also mills a card.
    reports(
        reported(activated(
            "{T}: Add {G}. Mill a card.",
            vec![CostPart::Tap],
            Effect::Seq(vec![add_g(), mill_1()]),
            true,
        )),
        "(b)",
        "a mana ability that mills",
    );
    // (b) A mana ability whose cost moves a card from a library.
    reports(
        reported(activated(
            "{T}, Exile the top card of your library: Add {G}.",
            vec![CostPart::Tap, exile_top()],
            add_g(),
            true,
        )),
        "(b)",
        "a mana ability's cost",
    );
    // (b) A triggered mana ability that draws (CR 605.1b allows it; the engine compiles
    // it as one).
    let insight = custom_card(
        "Fertile Insight",
        "Enchantment",
        None,
        "Whenever you tap a creature for mana, add an additional {G} and draw a card.",
    );
    reports(
        checks(&scan(&insight)),
        "(b)",
        "a triggered mana ability that draws",
    );
    // (b) A delayed triggered mana ability, created by a spell, that draws.
    let delayed = Effect::DelayedTrigger {
        trigger: TriggerCond::ThisTurn(Box::new(TriggerCond::TappedForMana {
            who: PlayerRel::Any,
            filter: Filter::Type(types::CardType::Land),
        })),
        body: Box::new(Body::effect(Effect::Seq(vec![add_g(), draw_1()]))),
        once: false,
    };
    reports(
        reported(spell(
            "Until end of turn, whenever a player taps a land for mana, they add an additional {G} and draw a card.",
            delayed,
        )),
        "(b)",
        "a delayed triggered mana ability",
    );
    // (b) A special action that can be taken during a mana payment and mills.
    let special = SpecialActionDef {
        who: PlayerFilter::You,
        cost: Cost {
            mana: None,
            parts: vec![CostPart::Mill(Value::c(1))],
        },
        action: SpecialActionEffect::Effect(add_g()),
        mana_timing: true,
    };
    reports(
        reported(static_ability(
            "Any time you could activate a mana ability, you may mill a card. If you do, add {G}.",
            StaticEffect::SpecialAction(special),
        )),
        "(b)",
        "a mana special action",
    );
    // (c) A cost that shuffles a library before another part.
    reports(
        reported(activated(
            "Shuffle your library, {T}: You gain 1 life.",
            vec![
                CostPart::Effect(Box::new(Effect::Shuffle {
                    who: PlayerRef::You,
                })),
                CostPart::Tap,
            ],
            gain_1(),
            false,
        )),
        "(c)",
        "a cost that shuffles",
    );
    // (c) A cost that searches a library before another part.
    let search = Effect::Search {
        who: PlayerRef::You,
        whose: PlayerRef::You,
        filter: Filter::Any,
        count: Value::c(1),
        to: Destination::zone(ZoneKind::Hand),
        reveal: true,
        shuffle: true,
    };
    reports(
        reported(activated(
            "Search your library for a card, put it into your hand, then shuffle, Pay 1 life: You gain 1 life.",
            vec![
                CostPart::Effect(Box::new(search)),
                CostPart::PayLife(Value::c(1)),
            ],
            gain_1(),
            false,
        )),
        "(c)",
        "a cost that searches",
    );
    // (d) An ability activated from a library.
    let mut from_library = ActivatedAbility::new(
        Cost::mana(ManaCost::parse("{1}").unwrap()),
        Body::effect(gain_1()),
    );
    from_library.zone = FunctionZone::Library;
    reports(
        reported(named(
            "{1}: You gain 1 life. Activate only if this card is in your library.",
            AbilityKind::Activated(from_library),
        )),
        "(d)",
        "activated from a library",
    );
    // (d) A keyword cost paid as the spell is cast.
    let flashback = Keyword::with_cost(
        KeywordKind::Flashback,
        Cost {
            mana: None,
            parts: vec![CostPart::Mill(Value::c(2))],
        },
    );
    reports(
        reported(named(
            "Flashback—Mill two cards.",
            AbilityKind::Keyword(flashback),
        )),
        "(d)",
        "a keyword cost",
    );
    // (e) A replacement effect that puts a dying creature into a library.
    let archive = custom_card(
        "Endless Archive",
        "Enchantment",
        None,
        "If a creature you control would die, put it on the bottom of its owner's library instead.",
    );
    reports(checks(&scan(&archive)), "(e)", "a replacement effect");
    // (e) An event not every payment causes counts once something in the pool causes it in
    // the middle of a payment: a mana ability that destroys a permanent, with a replacement
    // effect that puts a destroyed land into a library instead. Either alone reaches
    // nothing.
    let destroying = activated(
        "{T}: Add {G}. Destroy target land.",
        vec![CostPart::Tap],
        Effect::Seq(vec![
            add_g(),
            Effect::Destroy {
                what: Sel::All(Filter::Type(types::CardType::Land)),
                no_regen: false,
            },
        ]),
        true,
    );
    let to_library = static_ability(
        "If a land would be destroyed, put it on the bottom of its owner's library instead.",
        StaticEffect::Replacement(ReplacementDef {
            event: ReplacementEvent::Destroy(Filter::Type(types::CardType::Land)),
            action: ReplacementAction::MoveInstead(Destination::library_bottom()),
            self_replacement: false,
            optional: false,
        }),
    );
    assert!(reported(destroying.clone()).is_empty());
    assert!(reported(to_library.clone()).is_empty());
    reports(
        checks(&probe(vec![destroying, to_library])),
        "(e)",
        "a destruction a mana ability causes, made a library action",
    );
    // (e-assumption) A mana ability that casts a card: the scan doesn't follow that.
    reports(
        reported(activated(
            "{T}: Add {G}. You may cast a card.",
            vec![CostPart::Tap],
            Effect::Seq(vec![
                add_g(),
                Effect::CastCard {
                    who: PlayerRef::You,
                    what: Sel::None,
                    free: true,
                    optional: true,
                },
            ]),
            true,
        )),
        "(e-assumption)",
        "a mana ability that casts a card",
    );

    // Legal in no format, so outside the scan's cards and every card pool (`cards_in_scope`):
    // Under-Construction Skyscraper is fully supported, and its LEVEL 8+ mana ability scries.
    let skyscraper = card::card("Under-Construction Skyscraper");
    assert!(skyscraper.is_fully_supported());
    reports(
        checks(&scan(&skyscraper)),
        "(b)",
        "Under-Construction Skyscraper's scrying mana ability",
    );

    // Safe library actions aren't reported, and a library cost paid last is seen.
    for name in [
        "Royal Herbalist",
        "Penance",
        // "When you spend this mana to cast a creature spell, draw a card": a triggered
        // ability that uses the stack, dropped with an undone action.
        "Gilanra, Caller of Wirewood",
        // "{T}: Add {C}. When you do, mill two cards": a reflexive triggered ability.
        "Shaun & Rebecca, Agents",
    ] {
        let def = card::card(name);
        assert!(def.is_fully_supported(), "{name}");
        let found = all_findings(&scan(&def));
        assert!(found.is_empty(), "{name}: {found:?}");
    }
    let herbalist = scan(&card::card("Royal Herbalist"));
    assert!(
        herbalist
            .library_costs
            .iter()
            .any(|c| c.paid_last && c.context == "activation cost"),
        "{:?}",
        herbalist.library_costs
    );
}

/// A card scanned on its own.
fn probe(abilities: Vec<Ability>) -> Scan {
    scan(&custom_with("Probe", "Artifact", None, abilities))
}

/// The checks of every way the cards of a scan reach the exceptions together.
fn checks(r: &Scan) -> Vec<&'static str> {
    all_findings(r).iter().map(|f| f.check).collect()
}

/// Scans of several cards, as a pool.
fn pool(scans: Vec<Scan>) -> Scan {
    let mut t = Scan::default();
    for s in scans {
        t.add(s);
    }
    t.sort();
    t
}

/// The cards with an "(e)" finding in a pool.
fn reaching(t: &Scan) -> Vec<String> {
    let mut v: Vec<String> = all_findings(t)
        .into_iter()
        .filter(|f| f.check == "(e)")
        .map(|f| f.card)
        .collect();
    v.dedup();
    v
}

/// "At the beginning of your upkeep, sacrifice ~ unless you pay [cost]": a cost paid as an
/// ability resolves.
fn upkeep_unless(parts: Vec<CostPart>) -> Ability {
    let upkeep = TriggeredAbility::new(
        TriggerCond::BeginningOf {
            step: TriggerStep::Upkeep,
            whose: PlayerRel::You,
        },
        Body::effect(Effect::PayOptional {
            who: PlayerRef::You,
            cost: Cost { mana: None, parts },
            then: Box::new(Effect::Noop),
            otherwise: Box::new(Effect::SacrificeObjects { what: Sel::This }),
        }),
    );
    named(
        "At the beginning of your upkeep, sacrifice ~ unless you pay a cost.",
        AbilityKind::Triggered(upkeep),
    )
}

/// "Creatures can't attack you unless their controller pays [cost] for each of them": an
/// attack cost.
fn attack_cost(parts: Vec<CostPart>) -> Ability {
    static_ability(
        "Creatures can't attack you unless their controller pays a cost for each creature they control that's attacking you.",
        StaticEffect::Restriction(Restriction::AttackCost {
            attackers: Filter::creature(),
            defender: PlayerFilter::You,
            planeswalkers: false,
            cost: Cost { mana: None, parts },
        }),
    )
}

fn discard_a_card() -> CostPart {
    CostPart::Discard {
        filter: Filter::Any,
        count: Value::c(1),
        random: false,
    }
}

#[test]
fn the_scan_reports_nested_payments_rules_in_code_and_cost_modifiers() {
    interp!("illegal-action-reversal-restores-libraries");
    let reports = |r: &Scan, want: &str, case: &str| {
        let found = all_findings(r);
        assert!(
            found.iter().any(|f| f.check == want),
            "{case}: expected {want}, got {found:?}"
        );
    };
    // (b) A mana ability granted by a static ability (a nested ability): "Lands you
    // control have '{T}: Add {G}. Mill a card.'" (built by hand: the compiler doesn't make
    // that a mana ability, CR 605.1a).
    let milling = activated(
        "{T}: Add {G}. Mill a card.",
        vec![CostPart::Tap],
        Effect::Seq(vec![add_g(), mill_1()]),
        true,
    );
    let grant = StaticEffect::Continuous {
        affected: Filter::Type(types::CardType::Land),
        mods: vec![Modification::AddAbility(milling)],
    };
    reports(
        &probe(vec![static_ability(
            "Lands you control have \"{T}: Add {G}. Mill a card.\"",
            grant,
        )]),
        "(b)",
        "a granted mana ability",
    );
    // (b) An unclassified custom effect in a mana ability.
    reports(
        &probe(vec![activated(
            "{T}: Add {G}, then something only code does.",
            vec![CostPart::Tap],
            Effect::Seq(vec![
                add_g(),
                Effect::Custom("a probe's unknown effect".into()),
            ]),
            true,
        )]),
        "(b)",
        "an unclassified custom effect in a mana ability",
    );
    // (a) A repeated part paid once for each repetition: cumulative upkeep "Draw a card,
    // pay 1 life" draws before paying life, which can still fail (the life is scaled to
    // one part, the draws aren't).
    let upkeep = Keyword::with_cost(
        KeywordKind::CumulativeUpkeep,
        Cost {
            mana: None,
            parts: vec![
                CostPart::Effect(Box::new(draw_1())),
                CostPart::PayLife(Value::c(1)),
            ],
        },
    );
    reports(
        &probe(vec![named(
            "Cumulative upkeep—Draw a card, Pay 1 life.",
            AbilityKind::Keyword(upkeep),
        )]),
        "(a)",
        "a repeated cost",
    );
    // (a) Inside one of several costs: the chosen one is paid as a payment of its own,
    // the card put on the library before the discard, which can still fail.
    let one_of = CostPart::OneOf(vec![
        Cost {
            mana: None,
            parts: vec![hand_on_top(), discard_a_card()],
        },
        Cost {
            mana: None,
            parts: vec![CostPart::PayLife(Value::c(2))],
        },
    ]);
    reports(
        &probe(vec![activated(
            "Put a card from your hand on top of your library and discard a card or pay 2 life: You gain 1 life.",
            vec![one_of],
            gain_1(),
            false,
        )]),
        "(a)",
        "one of several costs",
    );
    // (a) A cost an effect that's a cost part has a player pay (a payment of its own).
    let inner = Effect::PayOptional {
        who: PlayerRef::You,
        cost: Cost {
            mana: None,
            parts: vec![hand_on_top(), sacrifice_a_creature()],
        },
        then: Box::new(Effect::Noop),
        otherwise: Box::new(Effect::Noop),
    };
    reports(
        &probe(vec![activated(
            "{T}, You may put a card from your hand on top of your library and sacrifice a creature: You gain 1 life.",
            vec![CostPart::Tap, CostPart::Effect(Box::new(inner))],
            gain_1(),
            false,
        )]),
        "(a)",
        "a payment in a cost part's effect",
    );
    // (a) An attack cost that puts a card on a library: another creature's or player's
    // cost can still fail after it (`combat::total_attack_cost`).
    let r = probe(vec![attack_cost(vec![hand_on_top()])]);
    reports(&r, "(a)", "an attack cost");
    assert!(
        all_findings(&r)
            .iter()
            .any(|f| f.detail.contains("attack or block cost")),
        "{:?}",
        r.findings
    );
    // (a) A special action (not taken during a mana payment) whose cost mills.
    let special = SpecialActionDef {
        who: PlayerFilter::You,
        cost: Cost {
            mana: None,
            parts: vec![CostPart::Mill(Value::c(1))],
        },
        action: SpecialActionEffect::Effect(gain_1()),
        mana_timing: false,
    };
    let r = probe(vec![static_ability(
        "Any time you could cast a sorcery, you may mill a card. If you do, you gain 1 life.",
        StaticEffect::SpecialAction(special),
    )]);
    reports(&r, "(a)", "a special action's cost");
    assert!(!checks(&r).contains(&"(b)"), "{:?}", r.findings);
    // (scan) Rules implemented in code that aren't classified: a static ability, a player
    // effect a spell creates, a modification.
    reports(
        &probe(vec![static_ability(
            "A probe's rule only code knows.",
            StaticEffect::Custom("a probe's unknown rule".into()),
        )]),
        "(scan)",
        "an unclassified custom static ability",
    );
    let player_effect = Effect::AddPlayerEffect {
        who: PlayerRef::You,
        effect: PlayerModification::Custom("a probe's unknown player rule".into()),
        duration: Duration::EndOfTurn,
    };
    reports(
        &probe(vec![spell(
            "Until end of turn, you follow a rule only code knows.",
            player_effect,
        )]),
        "(scan)",
        "an unclassified custom player effect",
    );
    let modification = StaticEffect::Continuous {
        affected: Filter::Source,
        mods: vec![Modification::Custom {
            name: "a probe's unknown modification".into(),
            layer: Layer::L6Ability,
        }],
    };
    reports(
        &probe(vec![static_ability(
            "~ has what only code knows.",
            modification,
        )]),
        "(scan)",
        "an unclassified custom modification",
    );
    // (a) A spell's cost written in the name of a rule implemented in code: "You may cast
    // this card from your graveyard by milling two cards in addition to paying its other
    // costs" (`kw/cast_self_from_graveyard.rs`).
    reports(
        &probe(vec![static_ability(
            "You may cast this card from your graveyard by milling two cards in addition to paying its other costs.",
            StaticEffect::Custom("cast from graveyard also:Mill two cards".into()),
        )]),
        "(a)",
        "a spell's cost in a rule implemented in code",
    );
    // (e) A rule implemented in code that turns paying life into a library action.
    let ashiok = card::card("Ashiok, Wicked Manipulator");
    assert!(ashiok.is_fully_supported());
    reports(&scan(&ashiok), "(e)", "Ashiok's rule in code");
    // (d) An ability that functions in any zone but can be activated only from a
    // graveyard isn't activated from a library.
    let mut from_graveyard = ActivatedAbility::new(
        Cost::mana(ManaCost::parse("{1}").unwrap()),
        Body::effect(gain_1()),
    );
    from_graveyard.zone = FunctionZone::Anywhere;
    from_graveyard.condition = Some(Condition::SelMatches(
        Sel::This,
        Filter::InZone(ZoneKind::Graveyard),
    ));
    let r = probe(vec![named(
        "{1}: You gain 1 life. Activate only if this card is in your graveyard.",
        AbilityKind::Activated(from_graveyard),
    )]);
    assert!(all_findings(&r).is_empty(), "{:?}", r.findings);

    // Discards paid outside casting and activation: Nephalia Academy turns one into a
    // library action, which reaches the exceptions only with a payment in which something
    // can fail after the discard. Library of Leng ("If an effect causes you to discard a
    // card") doesn't apply to a discard paid as a cost.
    let leng = scan(&card::card("Library of Leng"));
    assert!(all_findings(&leng).is_empty(), "{:?}", leng.findings);
    assert!(reach(&leng).discard_replacements.is_empty());
    let academy = scan(&card::card("Nephalia Academy"));
    assert!(all_findings(&academy).is_empty(), "{:?}", academy.findings);
    assert_eq!(reach(&academy).discard_replacements.len(), 1);
    let alone = probe(vec![upkeep_unless(vec![discard_a_card()])]);
    assert!(alone.late_discards.is_empty(), "{:?}", alone.late_discards);
    let twice = probe(vec![upkeep_unless(vec![
        discard_a_card(),
        discard_a_card(),
    ])]);
    assert_eq!(twice.late_discards.len(), 1, "{:?}", twice.late_discards);
    // A discard before a part that can fail; an attack cost's discard (another creature's
    // or player's cost can fail after it).
    let late = || {
        [
            probe(vec![upkeep_unless(vec![
                discard_a_card(),
                sacrifice_a_creature(),
            ])]),
            probe(vec![attack_cost(vec![discard_a_card()])]),
        ]
    };
    for (i, paying) in late().into_iter().enumerate() {
        assert_eq!(
            paying.late_discards.len(),
            1,
            "{i}: {:?}",
            paying.late_discards
        );
        let both = pool(vec![scan(&card::card("Nephalia Academy")), paying]);
        let pairs = discard_pairs(&both, &reach(&both));
        assert!(
            pairs.iter().any(|f| f.check == "(e)" && f.card == "Probe"),
            "{i}: {pairs:?}"
        );
    }
    // With Library of Leng instead, nothing.
    for paying in late() {
        let both = pool(vec![scan(&card::card("Library of Leng")), paying]);
        assert!(discard_pairs(&both, &reach(&both)).is_empty());
    }
    // A spell's discard cost is paid while the spell is cast, where it doesn't apply.
    let tax = custom_card(
        "Discard Toll",
        "Sorcery",
        None,
        "As an additional cost to cast this spell, discard a card and sacrifice a creature.\nDraw a card.",
    );
    assert!(scan(&tax).late_discards.is_empty());
    assert!(discard_pairs(&academy, &reach(&academy)).is_empty());
    // An effect that discards in the middle of a payment ("As ~ enters, discard your hand"
    // for a card an "until" exile returns there) makes Library of Leng and Nephalia Academy
    // count as reaching the exceptions; without the "until" exile, nothing does.
    for replacing in ["Library of Leng", "Nephalia Academy"] {
        let t = pool(vec![
            scan(&card::card("Heightened Awareness")),
            scan(&card::card(replacing)),
        ]);
        assert!(
            reaching(&t).is_empty(),
            "{replacing}: {:?}",
            all_findings(&t)
        );
        let t = pool(vec![
            scan(&card::card("Heightened Awareness")),
            scan(&card::card("Banisher Priest")),
            scan(&card::card(replacing)),
        ]);
        assert!(reach(&t).events.contains_key(&Ev::DiscardByEffect));
        assert_eq!(reaching(&t), [replacing], "{:?}", all_findings(&t));
    }

    // Activation cost modifiers: a modifier's parts come after an ability's own, so
    // "Sacrifice a land" (Brutal Suppression, for nontoken Rebels) after a card put on a
    // library reaches the exceptions; for a card that isn't a Rebel it doesn't apply.
    let suppression = scan(&card::card("Brutal Suppression"));
    assert!(
        suppression
            .activation_modifiers
            .iter()
            .any(|m| m.from_static && !m.alternative),
        "{:?}",
        suppression.activation_modifiers
    );
    let text = "Put a card from your hand on top of your library: You gain 1 life.";
    let rebel = |name: &str, rebel: bool| {
        CardDb::global().register(custom_card(
            name,
            if rebel {
                "Creature — Human Rebel"
            } else {
                "Creature — Human Soldier"
            },
            Some((1, 1)),
            text,
        ))
    };
    for (name, is_rebel, expect) in [
        ("Probe Rebel 733", true, true),
        ("Probe Soldier 733", false, false),
    ] {
        let def = rebel(name, is_rebel);
        let mut t = scan(&def);
        assert!(all_findings(&t).is_empty(), "{name}: {:?}", t.findings);
        t.add(scan(&card::card("Brutal Suppression")));
        let found = with_activation_modifiers(&t);
        assert_eq!(
            found.iter().any(|f| f.check == "(a)" && f.card == name),
            expect,
            "{name}: {found:?}"
        );
    }
    // An alternative cost replaces the parts of an activation cost: "You may put a card
    // from your hand on top of your library and sacrifice a creature rather than pay the
    // activation costs of abilities you activate" puts the card on the library before the
    // sacrifice, which can still fail. (Only under P0, whose abilities it's for.)
    let alternative = CostModifier {
        applies_to: CostTarget::Abilities(Filter::Any),
        who: PlayerRel::You,
        change: CostChange::AlternativeCost(Cost {
            mana: None,
            parts: vec![hand_on_top(), sacrifice_a_creature()],
        }),
    };
    let offering = CardDb::global().register(custom_with(
        "Probe Alternative 733",
        "Enchantment",
        None,
        vec![static_ability(
            "You may put a card from your hand on top of your library and sacrifice a creature rather than pay the activation costs of abilities you activate.",
            StaticEffect::CostModifier(alternative.clone()),
        )],
    ));
    let mut t = scan(&rebel("Probe Rebel 733", true));
    t.add(scan(&offering));
    assert!(
        t.activation_modifiers
            .iter()
            .any(|m| m.from_static && m.alternative),
        "{:?}",
        t.activation_modifiers
    );
    let found = with_activation_modifiers(&t);
    assert!(
        found.iter().any(|f| f.check == "(a)"
            && f.card == "Probe Rebel 733"
            && f.detail.contains("alternative cost")),
        "{found:?}"
    );
    // A modifier an effect creates can't be placed in a game: reported with each
    // library-touching activation cost.
    let created = Effect::AddPlayerEffect {
        who: PlayerRef::You,
        effect: PlayerModification::CostModifier(CostModifier {
            change: CostChange::AdditionalCost(Cost {
                mana: None,
                parts: vec![sacrifice_a_creature()],
            }),
            ..alternative
        }),
        duration: Duration::EndOfTurn,
    };
    let mut t = scan(&card::card("Penance"));
    t.add(probe(vec![spell(
        "Until end of turn, abilities you activate cost an additional \"Sacrifice a creature\" to activate.",
        created,
    )]));
    assert!(
        t.activation_modifiers.iter().any(|m| !m.from_static),
        "{:?}",
        t.activation_modifiers
    );
    let found = with_activation_modifiers(&t);
    assert!(
        found
            .iter()
            .any(|f| f.check == "(a)" && f.card == "Penance" && f.detail.contains("player effect")),
        "{found:?}"
    );
}

#[test]
fn the_scan_follows_what_replacement_effects_and_rules_cause_in_the_middle_of_a_payment() {
    interp!("illegal-action-reversal-restores-libraries");
    let real = |name: &str| {
        let def = card::card(name);
        assert!(def.is_fully_supported(), "{name}");
        scan(&def)
    };
    // Pariah deals the damage Shivan Reef would deal its controller to the enchanted
    // creature instead; Swans of Bryn Argoll turns damage dealt to it into a draw. Swans
    // alone reaches nothing (no payment deals damage to a permanent).
    assert!(reaching(&real("Swans of Bryn Argoll")).is_empty());
    for redirect in ["Pariah", "Pariah's Shield", "With Great Power . . ."] {
        let t = pool(vec![real("Swans of Bryn Argoll"), real(redirect)]);
        assert_eq!(reaching(&t), ["Swans of Bryn Argoll"], "{redirect}");
        assert!(
            reach(&t).events.contains_key(&Ev::DamageToPermanent),
            "{redirect}"
        );
    }
    // Nefarious Lich makes a player who can't exile cards for the damage lose the game,
    // and Enduring Angel one whose life would reach 0 if it can't transform: Lich's Mirror
    // turns the loss into a shuffle and seven draws.
    assert!(reaching(&real("Lich's Mirror")).is_empty());
    for losing in ["Nefarious Lich", "Enduring Angel // Angelic Enforcer"] {
        let t = pool(vec![real("Lich's Mirror"), real(losing)]);
        assert!(
            reaching(&t).contains(&"Lich's Mirror".to_string()),
            "{losing}: {:?}",
            all_findings(&t)
        );
    }
    // Winning makes the other players lose: a mana ability that wins does too.
    let winning = probe(vec![activated(
        "{T}: Add {G}. You win the game.",
        vec![CostPart::Tap],
        Effect::Seq(vec![
            add_g(),
            Effect::WinGame {
                who: PlayerRef::You,
            },
        ]),
        true,
    )]);
    assert_eq!(
        reaching(&pool(vec![real("Lich's Mirror"), winning])),
        ["Lich's Mirror"]
    );
    // Day becoming night transforms permanents.
    let mut now = Now::default();
    effect_now(&Effect::SetDayNight { day: false }, &mut now);
    assert_eq!(now.events, [Ev::Transform]);
    // Banisher Priest's exile ends as soon as it's sacrificed for mana
    // (`until::check_untils`): the creature enters the battlefield in the middle of the
    // payment, and Don't Blink shuffles it into its owner's library instead.
    assert!(reaching(&real("Don't Blink")).is_empty());
    let t = pool(vec![real("Don't Blink"), real("Banisher Priest")]);
    assert_eq!(reaching(&t), ["Don't Blink"]);
    // A card returned that way wasn't cast: "As ~ enters, if it was kicked, mill four
    // cards" mills nothing.
    let t = pool(vec![real("Monstrous War-Leech"), real("Banisher Priest")]);
    assert!(reaching(&t).is_empty(), "{:?}", all_findings(&t));
    // A card exiled from a library until its source leaves the battlefield goes back
    // there in the middle of a payment: a library action.
    let from_library = |until: UntilEvent| {
        probe(vec![spell(
            "Exile the top card of your library until an event.",
            Effect::ExileUntil {
                what: Sel::TopOfLibrary(PlayerRef::You, Value::c(1)),
                until,
            },
        )])
    };
    assert_eq!(
        reaching(&from_library(UntilEvent::SourceLeavesBattlefield)),
        ["Probe"]
    );
    // "Until an opponent becomes the monarch" ends there only if something makes a player
    // the monarch there.
    assert!(reaching(&from_library(UntilEvent::OpponentBecomesMonarch)).is_empty());
    let crowning = probe(vec![activated(
        "{T}: Add {G}. You become the monarch.",
        vec![CostPart::Tap],
        Effect::Seq(vec![
            add_g(),
            Effect::BecomeMonarch {
                who: PlayerRef::You,
            },
        ]),
        true,
    )]);
    assert_eq!(
        reaching(&pool(vec![
            from_library(UntilEvent::OpponentBecomesMonarch),
            crowning
        ])),
        ["Probe"]
    );
    // A prevention shield an effect creates is a replacement effect (CR 615.5): "Prevent
    // the next 3 damage that would be dealt to you this turn. For each 1 damage prevented
    // this way, mill a card." applies to Shivan Reef's damage.
    let shield = probe(vec![spell(
        "Prevent the next 3 damage that would be dealt to you this turn. For each 1 damage prevented this way, mill a card.",
        Effect::PreventDamage {
            to: Sel::Players(PlayerRef::You),
            amount: Some(Value::c(3)),
            duration: Duration::EndOfTurn,
            combat_only: false,
            then: Some(Box::new(mill_1())),
        },
    )]);
    assert_eq!(reaching(&shield), ["Probe"]);
    // No combat damage is dealt in the middle of a payment.
    let combat = probe(vec![static_ability(
        "If a source would deal combat damage to you, prevent that damage and mill that many cards.",
        StaticEffect::Replacement(ReplacementDef {
            event: ReplacementEvent::Damage {
                source: Filter::Any,
                to_players: Some(PlayerFilter::You),
                to_objects: None,
                combat_only: true,
            },
            action: ReplacementAction::PreventAndThen(None, Box::new(mill_1())),
            self_replacement: false,
            optional: false,
        }),
    )]);
    assert!(
        all_findings(&combat).is_empty(),
        "{:?}",
        all_findings(&combat)
    );
}

/// [`part_can_fail`] says these parts never fail once the check before any part is paid
/// passed: each is paid twice in one cost here, with what one payment needs, so the second
/// finds it gone (the check looks at each part on its own, `Game::pay_total_cost`). Were
/// the engine to fail one, a library part paid before it would be reachable, and
/// `part_can_fail` would have to say so.
#[test]
fn the_cost_parts_the_scan_says_cant_fail_dont_fail_in_a_game() {
    interp!("illegal-action-reversal-restores-libraries");
    let library = TestGame::new(2).library_size(P0) as i32;
    let cases: Vec<(&str, CostPart)> = vec![
        ("DiscardHand", CostPart::DiscardHand),
        (
            "ReturnToHand",
            CostPart::ReturnToHand {
                filter: Filter::creature(),
                count: Value::c(1),
            },
        ),
        (
            "AddCounters",
            CostPart::AddCounters {
                kind: types::counters::PLUS1.into(),
                count: Value::c(1),
            },
        ),
        ("ExertSelf", CostPart::ExertSelf),
        ("Mill", CostPart::Mill(Value::c(library))),
        ("PutFromHandOnLibrary", hand_on_top()),
        (
            "Effect",
            CostPart::Effect(Box::new(Effect::Draw {
                who: PlayerRef::You,
                n: Value::c(library),
            })),
        ),
    ];
    for (name, part) in cases {
        assert!(!part_can_fail(&part), "{name}");
        let mut t = TestGame::new(2);
        // The source isn't a creature; one creature to return, one card in the hand.
        let src = t.battlefield(P0, "Mind Stone");
        t.battlefield(P0, "Grizzly Bears");
        t.hand(P0, "Forest");
        let cost = Cost {
            mana: None,
            parts: vec![part.clone(), part],
        };
        let paid = t.g.pay_total_cost(
            P0,
            &cost,
            Some(src),
            &SpendContext::default(),
            &Ctx::new(Some(src), P0),
        );
        assert!(paid.is_ok(), "{name}: {:?}", paid.err());
    }
    // The parts paid with the source alone: nothing earlier can take it.
    for (name, part) in [
        ("ExileSelf", CostPart::ExileSelf),
        ("ReturnSelfToHand", CostPart::ReturnSelfToHand),
    ] {
        assert!(!part_can_fail(&part), "{name}");
        let mut t = TestGame::new(2);
        let src = t.battlefield(P0, "Mind Stone");
        let cost = Cost {
            mana: None,
            parts: vec![part],
        };
        let paid = t.g.pay_total_cost(
            P0,
            &cost,
            Some(src),
            &SpendContext::default(),
            &Ctx::new(Some(src), P0),
        );
        assert!(paid.is_ok(), "{name}: {:?}", paid.err());
    }
    // And a part the scan says can fail does: the second sacrifice finds no creature.
    let mut t = TestGame::new(2);
    let src = t.battlefield(P0, "Mind Stone");
    t.battlefield(P0, "Grizzly Bears");
    let part = sacrifice_a_creature();
    assert!(part_can_fail(&part));
    let cost = Cost {
        mana: None,
        parts: vec![part.clone(), part],
    };
    let paid = t.g.pay_total_cost(
        P0,
        &cost,
        Some(src),
        &SpendContext::default(),
        &Ctx::new(Some(src), P0),
    );
    assert!(paid.is_err());
}

// ===========================================================================
// The paths, in a game
// ===========================================================================

/// The zone changes of a game while an action is attempted, and those the reversal keeps
/// (CR 733.1), made again on the restored game. The observer isn't reversed with the game.
#[derive(Default)]
struct Moves {
    attempt: Vec<(ObjectId, Zone, Zone)>,
    kept: Vec<(ObjectId, Zone, Zone)>,
    reversed: bool,
}

/// Records the zone changes of a game (see [`Moves`]).
fn zone_changes(t: &mut TestGame) -> Arc<Mutex<Moves>> {
    let log: Arc<Mutex<Moves>> = Arc::default();
    let (l, l2) = (log.clone(), log.clone());
    t.g.observer = Some(
        EventObserver::new(move |_, ev| {
            if let Event::ZoneChange { old, from, to, .. } = ev {
                let mut m = l.lock().unwrap();
                if m.reversed {
                    m.kept.push((*old, *from, *to));
                } else {
                    m.attempt.push((*old, *from, *to));
                }
            }
        })
        .with_rollback(move |_| l2.lock().unwrap().reversed = true),
    );
    log
}

/// A custom instant with mana cost {R}: "As an additional cost to cast this spell,
/// sacrifice [what].\nYou gain 1 life."
fn toll(t: &mut TestGame, name: &str, what: &str) -> ObjectId {
    let mut def = custom_card(
        name,
        "Instant",
        None,
        &format!("As an additional cost to cast this spell, sacrifice {what}.\nYou gain 1 life."),
    );
    let cost = ManaCost::parse("{R}").unwrap();
    def.faces[0].chars.colors = cost.colors();
    def.faces[0].chars.mana_cost = Some(cost);
    t.custom(P0, def, Zone::Hand(P0))
}

/// The state of the game's random generators.
fn randomness(t: &TestGame) -> String {
    format!("{:?} {:?}", t.g.rng, t.g.player_rngs)
}

#[test]
fn a_reversed_cast_keeps_a_card_shuffled_into_a_library_by_a_replacement_effect() {
    cr!("733.1", "601.2g", "400.7");
    interp!("illegal-action-reversal-restores-libraries");
    interp!("reversal-undoes-mana-abilities");
    let mut t = TestGame::new(2);
    // "Sacrifice a creature: Add {C}{C}."
    let altar = t.battlefield(P0, "Ashnod's Altar");
    // "If ~ would be put into a graveyard from anywhere, reveal ~ and shuffle it into its
    // owner's library instead."
    let colossus = t.battlefield(P0, "Darksteel Colossus");
    let mountain = t.lands(P0, "Mountain", 1)[0];
    // "As an additional cost to cast this spell, sacrifice a creature."
    let fling = t.hand(P0, "Fling");
    let library = t.g.player(P0).library.clone();
    let random = randomness(&t);
    let moves = zone_changes(&mut t);
    // Paying {1}{R} sacrifices the Colossus to the Altar (the Mountain makes only {R}),
    // which shuffles it into the library; then no creature is left to sacrifice for Fling.
    let r = t
        .cast(P0, fling)
        .target(Entity::Player(P1))
        .method(CastMethod::Normal)
        .try_go();
    assert!(r.is_err());
    let m = moves.lock().unwrap();
    assert!(m.attempt.iter().any(|(o, from, to)| *o == colossus
        && *from == Zone::Battlefield
        && *to == Zone::Library(P0)));
    // CR 733.1: moving a card to a library and shuffling it can't be reversed. The
    // Colossus leaves the battlefield for the library again, a new object (CR 400.7), in
    // the shuffled library; the shuffle's randomness stays used.
    assert_eq!(
        m.kept,
        vec![(colossus, Zone::Battlefield, Zone::Library(P0))]
    );
    let now = t.g.current(colossus);
    assert_ne!(now, colossus);
    assert_eq!(t.zone(now), Zone::Library(P0));
    assert!(!t.on_battlefield(colossus));
    let mut expected = library.clone();
    expected.push(now);
    let mut lib = t.g.player(P0).library.clone();
    lib.sort();
    expected.sort();
    assert_eq!(lib, expected);
    assert_ne!(randomness(&t), random);
    assert_eq!(t.g.reversals.kept, 2);
    // Everything else is undone: the mana ability (its mana, the Mountain's tap), the
    // cast.
    assert!(t.on_battlefield(altar));
    assert!(!t.g.obj(mountain).tapped);
    assert!(t.g.player(P0).mana_pool.mana.is_empty());
    assert_eq!(t.g.obj(fling).zone, Zone::Hand(P0));
    assert!(t.g.stack.is_empty());
}

#[test]
fn a_reversed_cast_keeps_a_mill_a_replacement_effect_made_of_a_lands_damage() {
    cr!("733.1", "601.2g", "701.17a");
    interp!("illegal-action-reversal-restores-libraries");
    let mut t = TestGame::new(2);
    // "If damage would be dealt to you, prevent that damage and mill twice that many
    // cards."
    t.battlefield(P0, "Angel of Suffering");
    // "{T}: Add {U} or {R}. ~ deals 1 damage to you."
    let reef = t.battlefield(P0, "Shivan Reef");
    let spell = toll(&mut t, "Scrap Toll", "an artifact");
    let library = t.g.player(P0).library.clone();
    let moves = zone_changes(&mut t);
    // Paying {R} taps Shivan Reef, whose damage the Angel turns into milling two cards;
    // then no artifact is there to sacrifice.
    let r = t.cast(P0, spell).method(CastMethod::Normal).try_go();
    assert!(r.is_err());
    let m = moves.lock().unwrap();
    let milled = |v: &[(ObjectId, Zone, Zone)]| {
        v.iter()
            .filter(|(_, from, to)| *from == Zone::Library(P0) && *to == Zone::Graveyard(P0))
            .map(|(o, _, _)| *o)
            .collect::<Vec<_>>()
    };
    assert_eq!(milled(&m.attempt).len(), 2);
    // The mill stays (CR 733.1): the top two cards are in the graveyard, the rest of the
    // library as it was.
    let top_two: Vec<ObjectId> = library[library.len() - 2..].iter().rev().copied().collect();
    assert_eq!(milled(&m.kept), top_two);
    assert_eq!(m.kept.len(), 2);
    assert_eq!(t.g.player(P0).library, library[..library.len() - 2]);
    assert_eq!(t.graveyard_size(P0), 2);
    assert!(t.g.turn_events.iter().any(
        |e| matches!(e, Event::Milled { player, cards } if *player == P0 && cards.len() == 2)
    ));
    // The damage, its prevention and the mana ability are undone.
    assert_eq!(t.g.player(P0).life, 20);
    assert!(!t.g.obj(reef).tapped);
    assert_eq!(t.g.obj(spell).zone, Zone::Hand(P0));
}

#[test]
fn a_reversed_cast_keeps_cards_exiled_instead_of_paying_life_for_mana() {
    cr!("733.1", "601.2g");
    interp!("illegal-action-reversal-restores-libraries");
    ruling!(
        "Ashiok, Wicked Manipulator",
        "You can't choose to pay life instead of exiling cards"
    );
    let mut t = TestGame::new(2);
    // "If you would pay life while your library has at least that many cards in it, exile
    // that many cards from the top of your library instead."
    t.battlefield(P0, "Ashiok, Wicked Manipulator");
    // "{T}, Pay 1 life: Add one mana of any color."
    let confluence = t.battlefield(P0, "Mana Confluence");
    // "Sacrifice a creature: Add {C}{C}."
    t.battlefield(P0, "Ashnod's Altar");
    let bears = t.battlefield(P0, "Grizzly Bears");
    // "As an additional cost to cast this spell, sacrifice a creature."
    let fling = t.hand(P0, "Fling");
    let library = t.g.player(P0).library.clone();
    let top = *library.last().unwrap();
    let moves = zone_changes(&mut t);
    // Paying {1}{R}: Mana Confluence makes the {R}, and the life it costs becomes the top
    // card of the library exiled; the Altar sacrifices the Bears for the {1}. Then no
    // creature is left to sacrifice for Fling.
    let r = t
        .cast(P0, fling)
        .target(Entity::Player(P1))
        .method(CastMethod::Normal)
        .try_go();
    assert!(r.is_err());
    let m = moves.lock().unwrap();
    let exiled = |v: &[(ObjectId, Zone, Zone)]| {
        v.iter()
            .filter(|(_, from, to)| *from == Zone::Library(P0) && *to == Zone::Exile)
            .count()
    };
    assert_eq!(exiled(&m.attempt), 1);
    // CR 733.1: moving a card from a library to exile can't be reversed; it's the only
    // action kept.
    assert_eq!(m.kept, vec![(top, Zone::Library(P0), Zone::Exile)]);
    assert_eq!(t.zone(t.g.current(top)), Zone::Exile);
    assert_eq!(t.g.player(P0).library, library[..library.len() - 1]);
    // The rest is undone: the Bears are back, Mana Confluence untapped, the life (never
    // paid) as it was.
    assert!(t.on_battlefield(bears));
    assert!(!t.g.obj(confluence).tapped);
    assert_eq!(t.g.player(P0).life, 20);
    assert_eq!(t.g.obj(fling).zone, Zone::Hand(P0));
}

#[test]
fn a_reversed_cast_keeps_a_draw_a_redirected_lands_damage_causes() {
    cr!("733.1", "601.2g", "615.5", "121.1");
    interp!("illegal-action-reversal-restores-libraries");
    ruling!(
        "Swans of Bryn Argoll",
        "checks who that permanent’s controller is at the time that damage is prevented"
    );
    let mut t = TestGame::new(2);
    // "If a source would deal damage to this creature, prevent that damage. The source's
    // controller draws cards equal to the damage prevented this way."
    let swans = t.battlefield(P0, "Swans of Bryn Argoll");
    // "All damage that would be dealt to you is dealt to enchanted creature instead."
    let pariah = t.battlefield(P0, "Pariah");
    assert!(t.g.attach(pariah, Entity::Object(swans)));
    // "{T}: Add {U} or {R}. ~ deals 1 damage to you."
    let reef = t.battlefield(P0, "Shivan Reef");
    let spell = toll(&mut t, "Scrap Toll", "an artifact");
    let library = t.g.player(P0).library.clone();
    let top = *library.last().unwrap();
    let hand = t.g.player(P0).hand.clone();
    let drawn = t.g.history.cards_drawn.get(&P0).copied().unwrap_or(0);
    let moves = zone_changes(&mut t);
    // Paying {R} taps Shivan Reef; its damage is dealt to the Swans instead, which prevent
    // it, and P0 (the Reef's controller) draws a card. Then no artifact is there to
    // sacrifice.
    let r = t.cast(P0, spell).method(CastMethod::Normal).try_go();
    assert!(r.is_err());
    let m = moves.lock().unwrap();
    let draws = |v: &[(ObjectId, Zone, Zone)]| {
        v.iter()
            .filter(|(_, from, to)| *from == Zone::Library(P0) && *to == Zone::Hand(P0))
            .count()
    };
    assert_eq!(draws(&m.attempt), 1);
    // CR 733.1: the draw stays, and counts as a draw this turn.
    assert_eq!(m.kept, vec![(top, Zone::Library(P0), Zone::Hand(P0))]);
    let card = t.g.current(top);
    let mut expected = hand.clone();
    expected.push(card);
    assert_eq!(t.g.player(P0).hand, expected);
    assert_eq!(t.g.player(P0).library, library[..library.len() - 1]);
    assert_eq!(t.g.history.cards_drawn.get(&P0).copied(), Some(drawn + 1));
    // The damage, its redirection and prevention, and the mana ability are undone.
    assert!(t.on_battlefield(swans));
    assert_eq!(t.g.obj(swans).damage, 0);
    assert!(!t.g.obj(reef).tapped);
    assert_eq!(t.g.obj(spell).zone, Zone::Hand(P0));
}

#[test]
fn a_reversed_cast_keeps_what_lichs_mirror_does_with_a_loss_in_the_middle_of_a_payment() {
    cr!("733.1", "601.2g", "104.3e", "603.10a");
    interp!("illegal-action-reversal-restores-libraries");
    interp!("reversal-kept-library-actions-trigger");
    let mut t = TestGame::new(2);
    // "If damage would be dealt to you, exile that many cards from your graveyard instead.
    // If you can't, you lose the game." and "When Nefarious Lich leaves the battlefield,
    // you lose the game."
    let lich = t.battlefield(P0, "Nefarious Lich");
    // "If you would lose the game, instead shuffle your hand, your graveyard, and all
    // permanents you own into your library, then draw seven cards and your life total
    // becomes 20."
    let mirror = t.battlefield(P0, "Lich's Mirror");
    // "{T}: Add {U} or {R}. ~ deals 1 damage to you."
    let reef = t.battlefield(P0, "Shivan Reef");
    assert_eq!(t.graveyard_size(P0), 0);
    t.g.player_mut(P0).life = 7;
    let spell = toll(&mut t, "Creature Toll", "a creature");
    let library = t.g.player(P0).library.len();
    let moves = zone_changes(&mut t);
    // Paying {R} taps Shivan Reef; with no card in the graveyard to exile for its damage,
    // P0 would lose the game, and Lich's Mirror shuffles the three permanents into the
    // library and draws seven. Then no creature is there to sacrifice.
    let r = t.cast(P0, spell).method(CastMethod::Normal).try_go();
    assert!(r.is_err());
    let m = moves.lock().unwrap();
    let count = |v: &[(ObjectId, Zone, Zone)]| {
        let shuffled_in = v
            .iter()
            .filter(|(o, _, to)| [lich, mirror, reef].contains(o) && *to == Zone::Library(P0))
            .count();
        let drawn = v
            .iter()
            .filter(|(_, from, to)| *from == Zone::Library(P0) && *to == Zone::Hand(P0))
            .count();
        (shuffled_in, drawn)
    };
    assert_eq!(count(&m.attempt), (3, 7));
    // CR 733.1: the permanents shuffled into the library and the seven cards drawn stay.
    assert_eq!(count(&m.kept), (3, 7));
    assert_eq!(m.kept.len(), 10);
    for o in [lich, mirror, reef] {
        assert!(!t.on_battlefield(o));
        assert!(matches!(
            t.zone(t.g.current(o)),
            Zone::Library(_) | Zone::Hand(_)
        ));
    }
    assert_eq!(t.g.player(P0).library.len(), library + 3 - 7);
    // The spell was on the stack, not in the hand, as the hand was shuffled away: it
    // returns to the hand with the seven cards drawn.
    assert_eq!(t.hand_size(P0), 8);
    assert_eq!(t.g.obj(spell).zone, Zone::Hand(P0));
    // The loss, the damage and "your life total becomes 20" are undone.
    assert!(t.g.player(P0).in_game());
    assert_eq!(t.g.player(P0).life, 7);
    // Nefarious Lich left the battlefield, which stays: its leaves-the-battlefield
    // ability triggers (CR 603.10a).
    assert!(t
        .g
        .pending_triggers
        .iter()
        .any(|p| p.source == lich && p.controller == P0));
}

#[test]
fn a_reversed_cast_keeps_a_creature_shuffled_into_a_library_as_an_exile_ends() {
    cr!("733.1", "601.2g", "610.3");
    interp!("illegal-action-reversal-restores-libraries");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    // "Until end of turn, if one or more creatures would enter from exile or after being
    // cast from exile, their owners shuffle them into their libraries instead."
    t.lands(P0, "Island", 2);
    let blink = t.hand(P0, "Don't Blink");
    t.cast(P0, blink).go();
    t.resolve();
    // "When this creature enters, exile target creature an opponent controls until this
    // creature leaves the battlefield."
    t.lands(P0, "Plains", 3);
    let priest = t.hand(P0, "Banisher Priest");
    t.cast(P0, priest).go();
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.resolve_all();
    let exiled = t.g.current(bears);
    assert_eq!(t.zone(exiled), Zone::Exile);
    let priest = t.named_on_battlefield("Banisher Priest")[0];
    // "Sacrifice a creature: Add {C}{C}."
    t.battlefield(P0, "Ashnod's Altar");
    t.lands(P0, "Mountain", 1);
    // "As an additional cost to cast this spell, sacrifice a creature."
    let fling = t.hand(P0, "Fling");
    let library = t.g.player(P1).library.clone();
    let moves = zone_changes(&mut t);
    // Paying {1}{R} sacrifices the Priest to the Altar; as that mana ability's activation
    // ends, the Bears return (`until::check_untils`) and Don't Blink shuffles them into
    // P1's library instead. Then no creature is left to sacrifice for Fling.
    let r = t
        .cast(P0, fling)
        .target(Entity::Player(P1))
        .method(CastMethod::Normal)
        .try_go();
    assert!(r.is_err());
    let m = moves.lock().unwrap();
    assert!(
        m.attempt
            .iter()
            .any(|(o, from, to)| *o == exiled && *from == Zone::Exile && *to == Zone::Library(P1)),
        "{:?}",
        m.attempt
    );
    // CR 733.1: moving the card to a library stays, though the Priest's leaving, which
    // made it return, is undone.
    assert_eq!(m.kept, vec![(exiled, Zone::Exile, Zone::Library(P1))]);
    drop(m);
    let now = t.g.current(exiled);
    assert_eq!(t.zone(now), Zone::Library(P1));
    assert_eq!(t.g.player(P1).library.len(), library.len() + 1);
    assert!(t.on_battlefield(priest));
    // The exile has nothing left to return when the Priest leaves.
    let priest_card = t.g.current(priest);
    t.g.move_object(
        priest_card,
        Zone::Graveyard(P0),
        mtg_engine::events::MoveCause::Effect,
        None,
    );
    t.settle();
    assert_eq!(t.zone(t.g.current(exiled)), Zone::Library(P1));
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
}

#[test]
fn a_card_cast_from_the_top_of_a_library_goes_back_when_the_cast_is_reversed() {
    cr!("733.1", "401.5");
    interp!("illegal-action-reversal-restores-libraries");
    let mut t = TestGame::new(2);
    let next = t.library_top(P0, "Hill Giant");
    // "As an additional cost to cast this spell, sacrifice a creature."
    let fling = t.library_top(P0, "Fling");
    // "Play with the top card of your library revealed. You may play lands and cast spells
    // from the top of your library."
    t.battlefield(P0, "Future Sight");
    t.lands(P0, "Mountain", 2);
    t.g.recompute();
    assert_eq!(zones::revealed_top(&t.g, P0), Some(fling));
    let library = t.g.player(P0).library.clone();
    let revealed: Arc<Mutex<Vec<Option<ObjectId>>>> = Arc::default();
    let r2 = revealed.clone();
    t.g.observer = Some(EventObserver::new(move |g, ev| {
        if matches!(ev, Event::Custom { name, .. } if name == zones::TOP_REVEALED) {
            r2.lock().unwrap().push(zones::revealed_top(g, P0));
        }
    }));
    // Fling moves from the library to the stack (CR 733.1 lets that be reversed); P0
    // controls no creature to sacrifice, so the cast is reversed.
    let r = t
        .cast(P0, fling)
        .target(Entity::Player(P1))
        .method(CastMethod::Normal)
        .try_go();
    assert!(r.is_err());
    // The card below was never revealed while Fling was being cast (CR 401.5), and Fling
    // is back on top, still the revealed card.
    assert!(!revealed.lock().unwrap().contains(&Some(next)));
    assert_eq!(t.g.player(P0).library, library);
    assert_eq!(t.g.library_top(P0), Some(fling));
    assert_eq!(zones::revealed_top(&t.g, P0), Some(fling));
}
