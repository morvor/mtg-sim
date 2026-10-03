//! Comparing a rendering with the card's Oracle text.
//!
//! Both sides go through the same normalization ([`normalize_unit`]): reminder text
//! removed, the card's own name and "this creature/permanent/spell" shorthand replaced by
//! `~`, case, punctuation, whitespace and numbers in words vs digits made uniform.
//!
//! Wordings that differ but mean the same thing under the Comprehensive Rules are listed
//! in ONE place, [`EQUIVALENCES`] (string rewrites applied to both sides, each with its
//! justification) and [`IGNORED_WORDS`]. Keep the list small: an equivalence must never
//! hide a real difference in meaning.

use super::{render_card, RenderedFace};
use crate::card::CardDef;
use crate::keywords::KeywordKind;
use regex::Regex;
use std::sync::OnceLock;

/// An allowed equivalence between two wordings: every match of `pattern` (a regex over
/// the lowercased, reminder-free text with `~` for self-references) is replaced by
/// `replacement` on both sides.
pub struct Equivalence {
    pub pattern: &'static str,
    pub replacement: &'static str,
    /// Why the two wordings mean the same thing.
    pub why: &'static str,
}

/// The allowed equivalences, applied in order.
pub const EQUIVALENCES: &[Equivalence] = &[
    Equivalence {
        pattern: r"\bcan block an additional ([a-z ]*?creatures?) each combat this turn\b",
        replacement: "can block an additional $1 this turn",
        why: "\"Can block an additional creature this turn\" applies in each combat that \
              turn (CR 509.1b).",
    },
    Equivalence {
        pattern: r"\ba six-sided die\b",
        replacement: "a d6",
        why: "A d6 is a six-sided die (CR 706.1a).",
    },
    Equivalence {
        pattern: r"\bthe owner of ~",
        replacement: "~'s owner",
        why: "\"The owner of ~\" is \"~'s owner\".",
    },
    Equivalence {
        pattern: r"\bif no creatures are on the battlefield\b",
        replacement: "if there are no creatures on the battlefield",
        why: "The same condition, worded two ways.",
    },
    Equivalence {
        pattern: r"\b(adds?) an amount of (\{[a-z0-9]\}) equal to the number of ",
        replacement: "$1 $2 for each ",
        why: "\"Add {R} for each card\" adds one {R} per card: an amount of {R} equal to \
              the number of cards (CR 106.1).",
    },
    Equivalence {
        pattern: r"(^|\n|[.:] )([a-z]+s) can't be blocked by ([a-z]+s)(\.|$)",
        replacement: "$1$3 can't block $2$4",
        why: "\"Warriors can't be blocked by Cowards\" and \"Cowards can't block Warriors\" \
              are the same blocking restriction (CR 509.1b).",
    },
    Equivalence {
        pattern: r"\b(cards? (?:you|they|that player|an opponent|your opponents))(?:'ve| have| has)? drawn this turn\b",
        replacement: "$1 drew this turn",
        why: "\"Cards you've drawn this turn\" and \"cards you drew this turn\" are the \
              same cards (present perfect and past tense of the same draws, CR 121.1).",
    },
    Equivalence {
        pattern: r"\bat least (\d+|x|one|two|three|four|five|six|seven|eight|nine|ten)\b",
        replacement: "$1 or more",
        why: "\"At least N\" is \"N or more\".",
    },
    Equivalence {
        pattern: r"(^|\n)it (has|have) ([^.\n]*) as long as ~ is on the battlefield\b",
        replacement: "$1~ $2 $3",
        why: "\"As long as ~ is on the battlefield, it has ...\" is \"~ has ...\": the \
              abilities of a permanent function only while it is on the battlefield \
              (CR 113.6).",
    },
    Equivalence {
        pattern: r"\bwith the same name as ~(?:it\b)?",
        replacement: "named ~",
        why: "An object \"named ~\" is one with the same name as this object: a name used \
              in an ability refers to the object that has it, whatever its name is \
              (CR 201.5b), and objects share a name per CR 201.2a.",
    },
    Equivalence {
        pattern: r#"(^|[.:—•,] |\n|\bthen |\bif you do, )(target player|target opponent) ([a-z]+s)((?: [^.;",{]*?)?)(,? and|,? then|,? and then|,) ([a-z]+s)\b"#,
        replacement: "$1$2 $3$4$5 that player $6",
        why: "As below, for a targeted player: \"target player draws three cards and \
              loses 3 life\" is \"target player draws three cards and that player loses 3 \
              life\" (the same player, CR 115.1).",
    },
    Equivalence {
        pattern: r#"(^|[.:—•,] |\n|\bthen |\bif you do, )(each opponent|each player|each other player|target player|target opponent|that player|defending player|its controller|its owner|an opponent|a player|\{alt:[^}]*player[^}]*\}) ([a-z]+s)((?: [^.;",{]*?)?)(,? and|,? then|,? and then|,) ([a-z]+s)\b"#,
        replacement: "$1$2 $3$4$5 $2 $6",
        why: "A clause without a subject after \"and\", \"then\", or a comma shares the \
              subject of the clause before it: \"each opponent sacrifices a creature and loses 3 life\" is \
              \"each opponent sacrifices a creature and each opponent loses 3 life\". \
              Written out, so that the subject isn't mistaken for the controller's \
              (see the next entry).",
    },
    Equivalence {
        pattern: r"\b(?:is|are) put into your graveyard from the battlefield\b",
        replacement: "you own dies",
        why: "A card goes to its owner's graveyard (CR 400.3), so a permanent put into your \
              graveyard from the battlefield is one you own that dies (CR 700.4).",
    },
    Equivalence {
        pattern: r"\bcycles? or discards? (a|another|one or more|two or more) ",
        replacement: "discard $1 ",
        why: "Cycling a card discards it (CR 702.29a); an ability that triggers when a player \
              \"cycles or discards\" a card triggers once when a card is cycled (CR 702.29d), \
              as one that triggers on discarding does.",
    },
    Equivalence {
        pattern: r"\bif (?:it|that spell) is countered this way, exile it instead of putting it into its owner's graveyard",
        replacement: "if it would be put into a graveyard from the stack, exile it instead",
        why: "A spell countered by the instruction goes to its owner's graveyard from the \
              stack (CR 701.6a); the replacement (CR 614.1a) applies to that event.",
    },
    Equivalence {
        pattern: r"\bwhenever\b",
        replacement: "when",
        why: "\"When\" and \"whenever\" both introduce a trigger condition (CR 603.1); the \
              choice of word has no rules meaning.",
    },
    Equivalence {
        pattern: r"\bis put into a graveyard from the battlefield\b",
        replacement: "dies",
        why: "CR 700.4: \"dies\" means \"is put into a graveyard from the battlefield\".",
    },
    Equivalence {
        pattern: r"\bare put into a graveyard from the battlefield\b",
        replacement: "die",
        why: "CR 700.4 (plural).",
    },
    Equivalence {
        pattern: r"\b(to|into|on top of|on the bottom of|onto) (your|its owner's|their owners'|their owner's|its owners'|that player's|their) (hands?|library|libraries|graveyards?)\b",
        replacement: "$1 owner's $3",
        why: "A card always goes to its owner's hand, library, or graveyard (CR 400.3); \
              \"your hand\" on a card you own is its owner's hand, and \"their hand\" or \"their \
              library\" for the cards a player moves from their own zones is too.",
    },
    Equivalence {
        pattern: r"\bfrom (your|a|an|their|its owner's|that player's|target player's|target opponent's|an opponent's|defending player's|enchanted player's|each|all|any) ((?:opponent's |player's )?)(graveyards?|hands?|library|libraries)\b",
        replacement: "in $1 $2$3",
        why: "An object description says where the object is: \"a creature card from your \
              graveyard\" and \"a creature card in your graveyard\" describe the same cards.",
    },
    Equivalence {
        pattern: r"\b(discards?) all the cards in (your|their) hand\b",
        replacement: "$1 $2 hand",
        why: "A player's hand is the cards they hold (CR 402.1): discarding all the cards \
              in it is discarding that hand.",
    },
    Equivalence {
        pattern: r"\byou had (an?|one or more|two or more|three or more) ([a-z]+) enter the battlefield under your control\b",
        replacement: "$1 $2 entered the battlefield under your control",
        why: "\"If you had a land enter the battlefield under your control this turn\" asks \
              whether a land entered the battlefield under your control this turn.",
    },
    Equivalence {
        pattern: r"(^|[.:—•] )if ([^,.]+), (~ (?:doesn't|does not) untap during your untap step)\b",
        replacement: "$1$3 as long as $2",
        why: "A static ability's condition (\"~ doesn't untap during your untap step if it \
              has a depletion counter on it\") applies whenever it's true, as \"as long as\" \
              says (CR 604.1).",
    },
    Equivalence {
        pattern: r"\b(opponents?) you have\b",
        replacement: "$1",
        why: "\"For each opponent you have\" counts your opponents (CR 102.2, 102.3).",
    },
    Equivalence {
        pattern: r"\bnon(artifact|creature|land|enchantment|legendary|token),? non(white|blue|black|red|green)\b",
        replacement: "non$2 non$1",
        why: "The order of a noun's qualities doesn't change what it describes: a \
              \"nonartifact, nonblack creature\" is a \"nonblack nonartifact creature\".",
    },
    Equivalence {
        pattern: r#"(^|[.:—•,] |\n|\bthen |\bif you do, )(~it|~|it) (gets|gains) ([^.;",{]*?)(,? and|,? then) (deals)\b"#,
        replacement: "$1$2 $3 $4$5 $2 $6",
        why: "As for players above: \"~ gets +1/+0 until end of turn and deals 1 damage to \
              you\" is \"~ gets +1/+0 until end of turn and ~ deals 1 damage to you\".",
    },
    Equivalence {
        pattern: r"\b(puts?|plus) an additional\b",
        replacement: "$1 a",
        why: "\"Put an additional +1/+1 counter on it\" puts one more counter: a counter \
              (\"additional\" says that it's in addition to the counters it got from \
              another instruction or event).",
    },
    Equivalence {
        pattern: r"\b(draws?) (two|three|four|\d+) additional cards\b",
        replacement: "$1 $2 cards",
        why: "\"That player draws two additional cards\" in their draw step: two more \
              cards (in addition to the draw for the turn, CR 504.1).",
    },
    Equivalence {
        pattern: r"\b(exiles?|shuffles?) (?:all|each) (?:the )?cards (?:in|from) (your|their|its owner's|its controller's|that player's) (hand|graveyard|library)\b",
        replacement: "$1 $2 $3",
        why: "A hand, graveyard, or library is the cards in it (CR 400.1): exiling all the \
              cards in your graveyard is exiling your graveyard.",
    },
    Equivalence {
        pattern: r"\bif you (?:don't|do not) put (?:it|that card|the card|thatit) onto the battlefield,",
        replacement: "if you don't,",
        why: "\"You may put it onto the battlefield. If you don't put it onto the \
              battlefield, ...\" repeats the optional instruction \"if you don't\" refers \
              to.",
    },
    Equivalence {
        pattern: r"\b(controls?|targets?) one or more\b",
        replacement: "$1 a",
        why: "Controlling or targeting one or more Eggs is controlling or targeting an Egg \
              (a condition or quality, not a count of events).",
    },
    Equivalence {
        pattern: r"\b(gains?) x life, where x is the damage dealt this way\b",
        replacement: "$1 that many life",
        why: "\"You gain life equal to the damage dealt this way\": as much life as that \
              damage.",
    },
    Equivalence {
        pattern: r"\b(enters?|entered|entering) the battlefield\b",
        replacement: "$1",
        why: "\"Enters the battlefield\" has been shortened to \"enters\" in rules text \
              (CR glossary, \"Enters the Battlefield\"; CR 603.6a).",
    },
    Equivalence {
        pattern: r"\bthis turn (as though (?:it|they) (?:had|were|weren't|wasn't) [^.,;{}|]+?)(\.|$)",
        replacement: "$1 this turn$2",
        why: "Word order of the duration: \"you may cast spells this turn as though they had \
              flash\" is \"... as though they had flash this turn\".",
    },
    Equivalence {
        pattern: r"\b(return|put) (?:in)?to (your|owner's|its owner's|their owners') hands? (each|all|target) ([^.;]+?)(\.|;|$)",
        replacement: "$1 $3 $4 to $2 hand$5",
        why: "Word order: \"return to your hand all creature cards in your graveyard that \
              ...\" (the long object last) is \"return all creature cards ... to your hand\".",
    },
    Equivalence {
        pattern: r"\bif you have ([^.,;]+? in your (?:graveyard|library|hand))\b",
        replacement: "if there are $1",
        why: "\"If you have four or more creature cards in your graveyard\" asks whether \
              there are that many there.",
    },
    Equivalence {
        pattern: r"\b(cards?|spells?|creatures?|permanents?) that (?:have|has) ",
        replacement: "$1 with ",
        why: "\"A creature that has a -1/-1 counter on it\" is \"a creature with a -1/-1 \
              counter on it\".",
    },
    Equivalence {
        pattern: r"\b(creatures?) with ([a-z]+) (your opponents control|you control|an opponent controls)\b",
        replacement: "$1 $3 with $2",
        why: "The order of a noun's qualities: \"creatures with flying your opponents \
              control\" are \"creatures your opponents control with flying\".",
    },
    Equivalence {
        pattern: r"\bexiled with (~it|~|it) with mana value ([^ .,]+)",
        replacement: "with mana value $2 exiled with $1",
        why: "The order of a noun's qualities: \"a creature card with mana value X exiled \
              with ~\".",
    },
    Equivalence {
        pattern: r"\bactivate no more than\b",
        replacement: "activate only",
        why: "\"Activate no more than twice each turn\" and \"Activate only twice each \
              turn\" state the same restriction on activating the ability (CR 602.5b).",
    },
    Equivalence {
        pattern: r"\bone of your opponents\b",
        replacement: "an opponent",
        why: "Your opponents are the players you're playing against (CR 102.2, 102.3): \
              \"one of your opponents\" is \"an opponent\".",
    },
    Equivalence {
        pattern: r"\b(if [^.,;]+?) or if\b",
        replacement: "$1 or",
        why: "\"Activate only if A or if B\" is \"only if A or B\": the second \"if\" \
              repeats the conjunction's condition word.",
    },
    Equivalence {
        pattern: r"\b(discards?|draws?|mills?) x cards, where x is the number of ([^.;]+)",
        replacement: "$1 a card for each $2",
        why: "\"Discard a card for each Swamp you control\" is discarding X cards, where X \
              is the number of Swamps you control (the number is counted once, as the \
              instruction is followed, CR 608.2h).",
    },
    Equivalence {
        pattern: r"\bx times, where x is the number of ([^.;]+)",
        replacement: "for each $1",
        why: "Doing something X times, where X is the number of things, is doing it once \
              for each of them: \"copy it for each time you've cast your commander\", \
              \"unless you pay {1} for each card in your hand\".",
    },
    Equivalence {
        pattern: r"\bfrom graveyards\b",
        replacement: "in graveyards",
        why: "See \"from your graveyard\".",
    },
    Equivalence {
        pattern: r"\bfrom exile\b",
        replacement: "in exile",
        why: "See \"from your graveyard\".",
    },
    Equivalence {
        pattern: r"\byou (draw|discard|mill|scry|surveil|sacrifice|create|put|return|exile|search|reveal|look|shuffle|tap|untap|destroy|investigate|proliferate|seek|conjure|venture|explore|amass|populate|manifest|cloak|choose|add|counter|attach|transform|gain|lose|get|become|take|may|cast|pay|play|win|skip)\b",
        replacement: "$1",
        why: "An instruction without a subject is performed by the ability's controller \
              (CR 608.2c, 113.8): \"draw a card\" and \"you draw a card\" mean the same. \
              (A subject another player shares with an earlier clause is written out \
              first, see above.)",
    },
    Equivalence {
        pattern: r"(sacrifices? [^.]*?) of (their|his or her|your) choice\b",
        replacement: "$1",
        why: "The player who sacrifices chooses what to sacrifice (CR 701.21a); \"of their \
              choice\" restates it.",
    },
    Equivalence {
        pattern: r"\bthat spell or ability\b",
        replacement: "it",
        why: "Anaphora: \"that spell or ability\" refers back to the one already named.",
    },
    Equivalence {
        pattern: r"\bwould be dealt((?: to (?:[^.{}|]|\{[^{}]*\})+?)?) this turn (by|to) ((?:[^.{}|]|\{[^{}]*\})+?)(\.|$)",
        replacement: "would be dealt$1 $2 $3 this turn$4",
        why: "Word order of the duration in a prevention effect.",
    },
    Equivalence {
        pattern: r"\bthat (creature|permanent|card|spell|land|artifact|enchantment|planeswalker|token|aura|equipment|vehicle|battle|ability|object|source)s?'s\b",
        replacement: "thatit's",
        why: "Anaphora: \"that creature's\" and \"its\" both refer back to the object the \
              text already named; the renderer always uses the pronoun. Kept apart from a \
              plain \"its\" as `thatit`, which doesn't match the object itself (see \
              [`token_eq`]).",
    },
    Equivalence {
        pattern: r"\bthat (creature or planeswalker|creature or vehicle|artifact or creature|spell or ability)\b",
        replacement: "it",
        why: "Anaphora, as below: \"that creature or planeswalker\" refers back to the \
              target already named.",
    },
    Equivalence {
        pattern: r"\b(that|the) (creature|permanent|card|spell|land|artifact|enchantment|planeswalker|token|aura|equipment|vehicle|battle|ability|object|source|copy)\b",
        replacement: "thatit",
        why: "Anaphora: \"that creature\" and \"it\" refer back to the object already named \
              (`thatit` matches \"it\", see [`token_eq`]).",
    },
    Equivalence {
        pattern: r"\bthe (?:exiled|discarded) (card|creature|permanent|artifact)s?\b",
        replacement: "thatit",
        why: "Anaphora: \"Exile target creature. Return the exiled card ...\": \"the exiled \
              card\" and \"it\" refer back to the object the exile instruction just moved \
              (CR 400.7: that object is the card in exile); \"Discard a card. If the \
              discarded card was a land card, ...\" likewise.",
    },
    Equivalence {
        pattern: r"\bif (it|thatit|that-object|~it|~) had\b",
        replacement: "if $1 has",
        why: "As \"if it was\" below (here after \"that creature\" is `thatit`): whether \
              an object that left the battlefield had counters on it is judged by its last \
              known information (CR 608.2h), which is what \"if it has\" asks of it.",
    },
    Equivalence {
        pattern: r"\b(those|the) (creatures|permanents|cards|spells|lands|artifacts|tokens|objects)\b",
        replacement: "them",
        why: "Anaphora (plural).",
    },
    Equivalence {
        pattern: r"(\bon |\bto |^|[.,:—] |\{alt:|\|)each of them\b",
        replacement: "${1}them",
        why: "An instruction about a group of objects is about each of them: \"put a +1/+1 \
              counter on each of them\" and \"put a +1/+1 counter on them\" put one counter on \
              every object of the group (a counter is put on an object, CR 122.1), and \"each \
              of them gets +1/+1\" is \"they get +1/+1\" (CR 611.2c: each affected object).",
    },
    Equivalence {
        pattern: r"(^|[.:—•] |\n)it ([^.]*?) as long as (enchanted creature|enchanted permanent|equipped creature|enchanted land|enchanted artifact) is ",
        replacement: "${1}$3 $2 as long as $3 is ",
        why: "\"Enchanted permanent gets -1/-1 as long as it's a creature\" and \"As long as \
              enchanted permanent is a creature, it gets -1/-1\": the noun and the pronoun \
              name the same object, one way or the other.",
    },
    Equivalence {
        pattern: r"\bat the beginning of the next upkeep\b",
        replacement: "at the beginning of the next turn's upkeep",
        why: "Each turn has one upkeep (CR 500.1), and an effect can't be created during \
              a turn's untap step before its upkeep (no player gets priority then, CR \
              502.4), so the next upkeep is the next turn's.",
    },
    Equivalence {
        pattern: r"\bat the beginning of each of your postcombat main phases\b",
        replacement: "at the beginning of your postcombat main phase",
        why: "CR 505.1a: every main phase of a turn after the first is a postcombat main \
              phase, so an ability that triggers at the beginning of your postcombat main \
              phase triggers at each of them.",
    },
    Equivalence {
        pattern: r"\b(until end of turn|this turn)\. (?:it's|it is) still an? (?:legendary |snow |basic )?(?:(?:artifact|enchantment|creature|planeswalker|kindred|battle|land) )*(?:artifact|enchantment|creature|planeswalker|kindred|battle|land)\b",
        replacement: "in addition to its other types $1",
        why: "CR 205.1b: an effect that says the object is \"still a [type]\" and one that \
              gives types \"in addition to its other types\" both keep all its prior card \
              types, supertypes and subtypes. (The sentence after a duration applies for \
              that duration.)",
    },
    Equivalence {
        pattern: r"(?:\. (?:it's|it is)| (?:that's|that is)) still an? (?:legendary |snow |basic )?(?:(?:artifact|enchantment|creature|planeswalker|kindred|battle|land) )*(?:artifact|enchantment|creature|planeswalker|kindred|battle|land)\b",
        replacement: " in addition to its other types",
        why: "CR 205.1b, as above.",
    },
    Equivalence {
        pattern: r"\buntil end of turn\b",
        replacement: "this turn",
        why: "\"Until end of turn\" and \"this turn\" effects both end in the cleanup step \
              (CR 514.2).",
    },
    Equivalence {
        pattern: r#"\. (it has|thatit has|they have|it gains|thatit gains|they gain) ""#,
        replacement: " with \"",
        why: "A token created \"with\" an ability and one that \"has\" it (a following \
              sentence) are the same token (CR 111.1).",
    },
    Equivalence {
        pattern: r"\breturn(s?)\b",
        replacement: "put$1",
        why: "\"Return\" has no rules meaning of its own: it's a zone change like \
              \"put\" (CR 400.6), described by its destination.",
    },
    Equivalence {
        pattern: r"\b(into|onto)\b",
        replacement: "to",
        why: "Prepositions of a destination zone (\"into your hand\", \"onto the \
              battlefield\").",
    },
    Equivalence {
        pattern: r"\bwith (x|\d+|an?|one|two|three|four|five|six|seven|eight|nine|ten) additional\b",
        replacement: "with $1",
        why: "Counters an object enters with are put on it in addition to any others it \
              would enter with (CR 614.1c, 122.6).",
    },
    Equivalence {
        pattern: r"\balso\b ",
        replacement: "",
        why: "\"Also\" has no rules meaning.",
    },
    Equivalence {
        pattern: r"\beach player's (upkeep|draw step|end step)\b",
        replacement: "each $1",
        why: "Each player has one upkeep (draw step, end step) per turn: \"each upkeep\" \
              and \"each player's upkeep\" are the same steps (CR 501–503, 513).",
    },
    Equivalence {
        pattern: r"\bnumber of of\b",
        replacement: "number of",
        why: "\"For each of its colors\" rewritten to the \"where X is the number of\" form.",
    },
    Equivalence {
        pattern: r"\bat the beginning of the end step\b",
        replacement: "at the beginning of each end step",
        why: "Older wording: \"the end step\" in a trigger condition means every end step \
              (CR 513.1a).",
    },
    Equivalence {
        pattern: r"\b(gets?) an additional ([+-])",
        replacement: "$1 $2",
        why: "P/T modifications add up (CR 613.4c); \"an additional +2/-2\" is +2/-2.",
    },
    Equivalence {
        pattern: r"\b(is|are|was|were|do|does|has|have)n't\b",
        replacement: "$1 not",
        why: "Contraction.",
    },
    Equivalence {
        pattern: r"\ba player taps (an? [^.,]+?) for mana\b",
        replacement: "$1 is tapped for mana",
        why: "Only a player can tap a permanent for mana (CR 106.12): \"a player taps a \
              land for mana\" and \"a land is tapped for mana\" are the same event.",
    },
    Equivalence {
        pattern: r"\b(you|they)'ve\b",
        replacement: "$1",
        why: "\"If you've gained life this turn\" and \"if you gained life this turn\" ask \
              the same thing.",
    },
    Equivalence {
        pattern: r"\b(?:has|have) cast ([^.,;]*?) this turn\b",
        replacement: "cast $1 this turn",
        why: "\"If an opponent cast a blue spell this turn\" and \"if an opponent has cast a \
              blue spell this turn\" ask the same thing (as \"you've\" above).",
    },
    Equivalence {
        pattern: r"\b(draws?) an additional card\b",
        replacement: "$1 a card",
        why: "A triggered draw is in addition to the normal draw anyway (CR 504.1).",
    },
    Equivalence {
        pattern: r"(^|\. )([^.]+?) can't block (~|it)(\.|$)",
        replacement: "$1~ can't be blocked by $2$4",
        why: "\"Creatures with power less than ~'s power can't block it\" and \"~ can't be \
              blocked by creatures with power less than its power\" are the same \
              blocking restriction (CR 509.1b).",
    },
    Equivalence {
        pattern: r"\bthe exiled (card|creature|permanent)s\b",
        replacement: "the exiled $1",
        why: "Grammatical number.",
    },
    Equivalence {
        pattern: r"\bremove any number of (\S+) counters\b",
        replacement: "remove x $1 counters",
        why: "A cost of X counters is paid with any number of them (CR 107.3).",
    },
    Equivalence {
        pattern: r"\b(a|an|another) ([a-z]+) and/or ([a-z]+)\b",
        replacement: "$1 $2 or $3",
        why: "One object that's \"a Villain and/or artifact\" is one that's a Villain or an \
              artifact (or both).",
    },
    Equivalence {
        pattern: r"\band/or\b",
        replacement: "and",
        why: "In a list of object kinds, \"artifacts and/or enchantments\" and \"artifacts \
              and enchantments\" both mean objects that are either.",
    },
    Equivalence {
        pattern: r"(^|[^~\w])(it|that|there|what|he|she)'s\b",
        replacement: "$1$2 is",
        why: "Contraction.",
    },
    Equivalence {
        pattern: r"\bif (it|its|thatit|thatit's|that-object|~it|~it's|~|~'s|the sacrificed (?:creature|permanent|artifact)(?:'s)?|\{alt:the sacrificed [^}]*\})((?: power| toughness| mana value)?) was\b",
        replacement: "if $1$2 is",
        why: "A condition about an object checked after it left its zone (\"Destroy target \
              creature. If it was attacking, ...\", \"When ~ dies, if it was a Human\") is \
              judged by the object's last known information (CR 608.2h, 400.7), which is \
              what the past tense describes; the ability language has one condition for both \
              and the engine evaluates a moved object by its last known information.",
    },
    Equivalence {
        pattern: r"\b(spells?) in your (hand|graveyard) ((?:with|that shares?) [^.,]+?) without paying\b",
        replacement: "$1 $3 in your $2 without paying",
        why: "\"Cast a spell in your hand with mana value 4 or less\" and \"... a spell with \
              mana value 4 or less in your hand\": the same qualities, in either order.",
    },
    Equivalence {
        pattern: r"\b(adds?) an additional\b",
        replacement: "$1",
        why: "A triggered mana ability's mana is added in addition to the mana the \
              permanent produced (CR 605.1b, 106.12a); \"additional\" restates it.",
    },
    Equivalence {
        pattern: r"\bx life,? where x is the life lost this way\b",
        replacement: "that many life",
        why: "\"Life equal to the life lost this way\" is the amount of life the previous \
              instruction made players lose: \"that many life\" (CR 608.2c).",
    },
    Equivalence {
        pattern: r"\{x\} (less|more) to (cast|activate),? where x is the number of ([^.]+)",
        replacement: "{1} $1 to $2 for each $3",
        why: "A cost reduced or increased by {X}, where X is a number of objects, changes by \
              {1} for each of them.",
    },
    Equivalence {
        pattern: r"\bthe number of (white|blue|black|red|green) mana symbols in the mana costs of permanents you control\b",
        replacement: "your devotion to $1",
        why: "CR 700.5: devotion to a color is the number of mana symbols of that color among \
              the mana costs of permanents that player controls.",
    },
    Equivalence {
        pattern: r"\banother\b",
        replacement: "other",
        why: "\"Another creature\" and \"other creatures\" both exclude the object itself; the \
              article and grammatical number are ignored elsewhere.",
    },
    Equivalence {
        pattern: r"\byour (commanders?)\b",
        replacement: "$1 you own",
        why: "A player's commander is the commander they own (CR 903.3); \"your commander\" \
              and \"a commander you own\" are the same objects.",
    },
    Equivalence {
        pattern: r"\bthey\b",
        replacement: "them",
        why: "Pronoun case: \"they\"/\"them\" refer to the same objects.",
    },
    Equivalence {
        pattern: r"\b(he|she|him)\b",
        replacement: "~it",
        why: "Oracle text of named characters refers to the permanent by a gendered \
              pronoun where other cards say \"it\"; the referent is the same object, the \
              character the card names (itself), which is why such a card can say \"him\" \
              right after naming another creature (\"~ becomes a copy of target creature. \
              Prevent all damage that would be dealt to him\"): `~it`, the object itself.",
    },
    Equivalence {
        pattern: r"\bher(\.|,|$| deal\b)",
        replacement: "~it$1",
        why: "See \"he\": \"her\" as an object (\"put a +1/+1 counter on her\"), not the \
              possessive.",
    },
    Equivalence {
        pattern: r"\bhis\b",
        replacement: "~it's",
        why: "See \"he\".",
    },
    Equivalence {
        pattern: r"\bthem\b",
        replacement: "it",
        why: "Pronoun number (grammatical number is ignored).",
    },
    Equivalence {
        pattern: r"\btheir\b",
        replacement: "its",
        why: "Pronoun number: \"their\" and \"its\" (grammatical number is ignored, see \
              singularization).",
    },
    Equivalence {
        pattern: r"\bchoose ((?:up to \w+ )?target [^.;:]+?)\. (put|exile|destroy|tap|untap) (?:it|thatit)\b",
        replacement: "$2 $1",
        why: "\"Choose target creature card in your graveyard. Return it to the \
              battlefield.\" is \"Return target creature card in your graveyard to the \
              battlefield.\": a target is chosen as the spell or ability is put on the \
              stack (CR 601.2c, 602.2b, 115.1), and \"it\" is that target.",
    },
    Equivalence {
        pattern: r"\bwhen you spend this mana to\b",
        replacement: "when that mana is spent to",
        why: "Mana goes to the pool of the player its ability's effect names, here the \
              ability's controller (\"you\", CR 106.4), so \"when you spend this mana\" is \
              when that mana is spent (CR 106.6).",
    },
];

/// The sentence-level rewrites `sentence_rewrites` applies to both sides (word order and
/// the forms of a number), listed with the [`EQUIVALENCES`] in the report.
pub const SENTENCE_FORMS: &[(&str, &str)] = &[
    (
        "\"Until end of turn, X.\" / \"As long as C, X.\" / \"At the beginning of the next end step, X.\" -> \"X until end of turn.\" ...",
        "A leading duration, condition or delayed time applies to the whole sentence wherever it's written.",
    ),
    (
        "\"If C, Y. Otherwise, X.\" -> \"X. If C, Y instead.\"",
        "Both state the same choice between two instructions.",
    ),
    (
        "\"X if C.\" -> \"If C, X.\" (not \"if able\", \"only if\", \"unless\")",
        "A trailing condition is the same condition.",
    ),
    (
        "\"You may pay P. If you don't, X.\" -> \"X unless you pay P.\"",
        "The same optional payment (CR 118.12).",
    ),
    (
        "\"N damage equal to V\", \"N life for each F\", \"+1/+1 for each F\", \"a card for each F\", ... -> \"X ..., where X is V\"",
        "CR 107.3: X is defined by the text; both describe the same number.",
    ),
    (
        "\"+2/+2 for each F\", \"2 damage to P for each F\", \"two cards for each F\" -> \"X ..., where X is 2 times the number of F\"",
        "CR 107.3: the same number, as above.",
    ),
    (
        "\"When C, at end of combat, X.\" -> \"When C, X at end of combat.\" (a single instruction ending the ability)",
        "A delayed trigger's time applies to the instruction wherever it's written (CR 603.7).",
    ),
    (
        "\"Choose target T. [Instruction] it ...\" -> \"[Instruction] target T ...\" (an instruction acting on it, not a condition)",
        "CR 601.2c, 602.2b: the target is chosen as the spell or ability is put on the stack either way.",
    ),
    (
        "\"up to X target T ..., where X is V\" -> \"up to V target T ...\"",
        "CR 107.3: the same number of targets.",
    ),
    (
        "\"with mana value X or less ..., where X is V\" -> \"with mana value less than or equal to V ...\"",
        "CR 107.3: the same comparison, with the number X stands for named in place.",
    ),
];

/// Words dropped from both sides before comparing.
pub const IGNORED_WORDS: &[(&str, &str)] = &[
    (
        "and",
        "Joins clauses and list items; instructions are followed in order (CR 608.2c) \
         whether they're joined by \"and\", \"then\", or a period. A list of alternatives \
         still says \"or\".",
    ),
    (
        "then",
        "Sequencing word: CR 608.2c, instructions are followed in the order written.",
    ),
];

/// Quantifier words that may be left out, but never stand for one another: "each X",
/// "all Xs" and a bare plural ("creatures you control get +1/+1") all mean every object
/// that matches; "a"/"an" vs none is grammatical number ("put a +1/+1 counter on each
/// creature" / "creatures with +1/+1 counters on them"). Normalized to `each` and `a`.
/// A side may leave one out where the other has a word that isn't a quantifier, but "a"
/// and "each" in the same place mismatch: "sacrifice a creature" isn't "sacrifice all
/// creatures", nor "if an opponent has ..." "if each opponent has ...".
pub const OPTIONAL_QUANTIFIERS: &[(&str, &str)] = &[
    (
        "each",
        "Universal quantification: \"each\", \"all\", or a bare plural.",
    ),
    (
        "a",
        "Indefinite article \"a\"/\"an\", or none with a plural.",
    ),
];

fn is_quantifier(t: &str) -> bool {
    t == "a" || t == "each"
}

/// Self-reference shorthand used on cards (CR 201.5a).
const SELF_REFS: &[&str] = &[
    "creature",
    "artifact",
    "enchantment",
    "land",
    "permanent",
    "spell",
    "card",
    "aura",
    "equipment",
    "vehicle",
    "token",
    "planeswalker",
    "saga",
    "battle",
    "class",
    "case",
    "room",
    "fortification",
    "spacecraft",
    "siege",
    "mount",
    "object",
    "scheme",
    "plane",
    "phenomenon",
    "conspiracy",
    "attraction",
    "contraption",
];

/// Removes parenthesized reminder text.
pub fn strip_reminder(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut depth = 0;
    for ch in text.chars() {
        match ch {
            '(' => depth += 1,
            ')' if depth > 0 => depth -= 1,
            c if depth == 0 => out.push(c),
            _ => {}
        }
    }
    out
}

/// Replaces the card's names and "this [object]" with `~`.
pub fn self_refs(text: &str, names: &[String]) -> String {
    let mut s = text.to_string();
    let mut names: Vec<&String> = names.iter().filter(|n| !n.is_empty()).collect();
    names.sort_by_key(|n| std::cmp::Reverse(n.len()));
    for n in names {
        s = s.replace(n.as_str(), "~");
    }
    static RE: OnceLock<Option<Regex>> = OnceLock::new();
    match RE.get_or_init(|| Regex::new(&format!(r"(?i)\bthis ({})\b", SELF_REFS.join("|"))).ok()) {
        Some(re) => re.replace_all(&s, "~").to_string(),
        None => s,
    }
}

/// The names a card's text may use for itself.
pub fn self_names(def: &CardDef, face: usize) -> Vec<String> {
    let mut v = vec![def.name.to_string()];
    let fname = def.faces[face].chars.name.to_string();
    v.push(fname.clone());
    let legendary = def.faces[face]
        .chars
        .supertypes
        .contains(crate::types::Supertype::Legendary);
    if legendary {
        for n in [&fname] {
            if let Some((short, _)) = n.split_once(',') {
                v.push(short.to_string());
            }
            if let Some((short, _)) = n.split_once(" of ") {
                v.push(short.to_string());
            }
            if let Some((short, _)) = n.split_once(" the ") {
                v.push(short.to_string());
            }
            let words: Vec<&str> = n.split(' ').collect();
            if words.len() > 1 && words[0].len() >= 3 && words[0] != "The" {
                v.push(words[0].to_string());
            }
        }
    }
    v
}

fn number_words() -> &'static [(&'static str, &'static str)] {
    &[
        ("zero", "0"),
        ("one", "1"),
        ("two", "2"),
        ("three", "3"),
        ("four", "4"),
        ("five", "5"),
        ("six", "6"),
        ("seven", "7"),
        ("eight", "8"),
        ("nine", "9"),
        ("ten", "10"),
        ("eleven", "11"),
        ("twelve", "12"),
        ("thirteen", "13"),
        ("fourteen", "14"),
        ("fifteen", "15"),
        ("sixteen", "16"),
        ("seventeen", "17"),
        ("eighteen", "18"),
        ("nineteen", "19"),
        ("twenty", "20"),
        ("once", "1 time"),
        ("twice", "2 times"),
    ]
}

/// Singular form of a token (grammatical number is ignored).
fn singular(w: &str) -> String {
    if w.len() <= 3 || w.starts_with('{') || w.contains('/') {
        return match w {
            "has" => "have".into(),
            "is" | "are" => "is".into(),
            "was" | "were" => "was".into(),
            "its" => "it".into(),
            "does" => "do".into(),
            other => other.to_string(),
        };
    }
    match w {
        "does" => return "do".into(),
        "doesn't" | "don't" => return "don't".into(),
        "isn't" | "aren't" => return "isn't".into(),
        "wasn't" | "weren't" => return "wasn't".into(),
        _ => {}
    }
    // "Zombies", "Faeries", "Pixies": nouns ending in "ie".
    if [
        "zombies", "faeries", "pixies", "cookies", "rookies", "zombie's",
    ]
    .contains(&w)
    {
        return w.trim_end_matches('s').trim_end_matches('\'').to_string();
    }
    if let Some(stem) = w.strip_suffix("ies") {
        return format!("{stem}y");
    }
    if let Some(stem) = w.strip_suffix("ves") {
        if ["el", "dwar", "wol", "werewol", "sel", "hal"]
            .iter()
            .any(|s| stem.ends_with(s))
        {
            return format!("{stem}f");
        }
    }
    for suf in ["ches", "shes", "sses", "xes"] {
        if w.ends_with(suf) {
            return w[..w.len() - 2].to_string();
        }
    }
    if w.ends_with("ss") || w.ends_with("us") || w.ends_with("is") {
        return w.to_string();
    }
    if let Some(stem) = w.strip_suffix('s') {
        return stem.to_string();
    }
    w.to_string()
}

/// Normalizes one unit (an ability line or keyword) into comparable tokens.
pub fn normalize_unit(text: &str) -> Vec<String> {
    let mut s = text
        .replace(['\u{2212}', '\u{2013}'], "-")
        .replace('\u{2019}', "'")
        .replace(['\u{201C}', '\u{201D}'], "\"")
        .to_lowercase();
    s = sentence_rewrites(&s);
    s = s.replace('\n', " ").replace('•', " ");
    for (e, re) in EQUIVALENCES.iter().zip(equivalence_regexes()) {
        let Some(re) = re else {
            continue;
        };
        // Until nothing changes (a few rounds): "each player draws two cards, then
        // discards three cards, then loses 4 life" names the subject once per clause.
        for _ in 0..6 {
            let n = re.replace_all(&s, e.replacement).to_string();
            if n == s {
                break;
            }
            s = n;
        }
    }
    // Tokenize: keep {..} symbols, +1/+1, ~, words with apostrophes and hyphens.
    let mut tokens = Vec::new();
    let mut cur = String::new();
    // Braces nest: "{alt:they pay {2}|that player pays {2}}" is one token.
    let mut depth = 0usize;
    for ch in s.chars() {
        if depth > 0 {
            cur.push(ch);
            match ch {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        tokens.push(std::mem::take(&mut cur));
                    }
                }
                _ => {}
            }
            continue;
        }
        match ch {
            '{' => {
                if !cur.is_empty() {
                    tokens.push(std::mem::take(&mut cur));
                }
                cur.push(ch);
                depth = 1;
            }
            c if c.is_alphanumeric() || matches!(c, '\'' | '+' | '-' | '/' | '~' | '*') => {
                cur.push(c)
            }
            // A quotation mark: a quoted ability's "enchanted creature" is another
            // object's (see [`attached_anaphora`]).
            '"' => {
                if !cur.is_empty() {
                    tokens.push(std::mem::take(&mut cur));
                }
                tokens.push(QUOTE.into());
            }
            _ => {
                if !cur.is_empty() {
                    tokens.push(std::mem::take(&mut cur));
                }
            }
        }
    }
    if !cur.is_empty() {
        tokens.push(cur);
    }
    let mut out = Vec::new();
    for t in tokens {
        if t == QUOTE {
            out.push(t);
            continue;
        }
        let t = t.trim_matches('\'').to_string();
        let t = t
            .strip_suffix("'s")
            .or_else(|| t.strip_suffix("s'"))
            .map(|x| x.to_string())
            .unwrap_or(t);
        let t = t.trim_end_matches('-').to_string();
        if t.is_empty() || t == "-" {
            continue;
        }
        let t = match number_words().iter().find(|(w, _)| *w == t) {
            Some((_, d)) => d.to_string(),
            None => t,
        };
        // A brace token ({T}, {alt:...}) is one token, spaces included.
        let parts: Vec<&str> = if t.starts_with('{') {
            vec![t.as_str()]
        } else {
            t.split(' ').collect()
        };
        for part in parts {
            let p = if part == "an" {
                "a".to_string()
            } else if part == "all" {
                "each".to_string()
            } else {
                singular(part)
            };
            if IGNORED_WORDS.iter().any(|(w, _)| *w == p) {
                continue;
            }
            out.push(p);
        }
    }
    attached_anaphora(expand_symbol_counts(out))
}

/// "Pay eight {E}" is "pay {E}{E}{E}{E}{E}{E}{E}{E}": a count of energy or ticket symbols
/// spelled as a number (CR 107.14, 107.17).
fn expand_symbol_counts(tokens: Vec<String>) -> Vec<String> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < tokens.len() {
        if let (Ok(n), Some(next)) = (tokens[i].parse::<usize>(), tokens.get(i + 1)) {
            if (next == "{e}" || next == "{tk}") && (1..=20).contains(&n) {
                for _ in 0..n {
                    out.push(next.clone());
                }
                i += 2;
                continue;
            }
        }
        out.push(tokens[i].clone());
        i += 1;
    }
    out
}

/// A quotation mark among the tokens, for [`attached_anaphora`] only.
const QUOTE: &str = "\"";

/// After the first "enchanted creature" (or "equipped creature", ...) in a unit, later
/// ones may be "it": both refer to the object the source is attached to (anaphora). Not
/// across a quotation mark: in "As long as enchanted permanent is an Equipment, it has
/// \"Equipped creature has flying.\"" the quoted ability's equipped creature is the
/// Equipment's.
fn attached_anaphora(tokens: Vec<String>) -> Vec<String> {
    let heads = ["creature", "permanent", "land", "artifact", "planeswalker"];
    let mut out: Vec<String> = Vec::new();
    let mut seen = false;
    let mut i = 0;
    while i < tokens.len() {
        let t = &tokens[i];
        if t == QUOTE {
            seen = false;
            i += 1;
            continue;
        }
        let is_attached = (t == "enchanted" || t == "equipped" || t == "fortified")
            && tokens
                .get(i + 1)
                .is_some_and(|n| heads.contains(&n.as_str()));
        if is_attached {
            if seen {
                out.push("it".into());
            } else {
                out.push(t.clone());
                out.push(tokens[i + 1].clone());
                seen = true;
            }
            i += 2;
            continue;
        }
        out.push(t.clone());
        i += 1;
    }
    out
}

/// The compiled patterns of [`EQUIVALENCES`], in order (`None` for one that doesn't
/// compile, which the tests check never happens).
pub fn equivalence_regexes() -> &'static [Option<Regex>] {
    static CACHE: OnceLock<Vec<Option<Regex>>> = OnceLock::new();
    CACHE.get_or_init(|| {
        EQUIVALENCES
            .iter()
            .map(|e| Regex::new(e.pattern).ok())
            .collect()
    })
}

/// Sentence-level rewrites (word order): a leading "Until end of turn, ..." or "As long
/// as ..., ..." clause moves to the end of its sentence; "X ..., where X is V" and
/// "equal to V" forms are made uniform.
fn sentence_rewrites(s: &str) -> String {
    static LEAD: OnceLock<Option<Regex>> = OnceLock::new();
    let lead = LEAD.get_or_init(|| {
        Regex::new(r#"(^|[.:—•] |\n|")(until end of turn|until your next turn|this turn|as long as [^,]+|at the beginning of the next end step|until the end of your next turn|during your turn|during turns other than yours|during each of your turns|at the beginning of the next turn's upkeep|at the beginning of the next cleanup step|at the beginning of your next upkeep|at end of combat), ([^.]+)\."#)
            .ok()
    });
    let mut s = s.to_string();
    // "You may pay {3}{B}. If you don't, return it ..." and "Return it ... unless you pay
    // {3}{B}" are the same choice (CR 118.12).
    static MAY_PAY: OnceLock<Option<Regex>> = OnceLock::new();
    if let Some(re) = MAY_PAY.get_or_init(|| {
        Regex::new(r"(^|[.:—•] |\n|, )(?:then )?you may pay ([^.]+)\. if you don't, ([^.]+)\.").ok()
    }) {
        s = re.replace_all(&s, "${1}$3 unless you pay $2.").to_string();
    }
    // "If C, Y. Otherwise, X." and "X. If C, Y instead." state the same choice.
    static OTHERWISE: OnceLock<[Option<Regex>; 2]> = OnceLock::new();
    let [instead, otherwise] = OTHERWISE.get_or_init(|| {
        [
            Regex::new(r"(^|[.:—•] |\n)if ([^,.]+), instead ([^.]+)\.").ok(),
            Regex::new(r"(^|[.:—•] |\n)if ([^,.]+), ([^.]+)\. otherwise, ([^.]+)\.").ok(),
        ]
    });
    if let Some(instead) = instead {
        s = instead
            .replace_all(&s, "${1}if $2, $3 instead.")
            .to_string();
    }
    if let Some(otherwise) = otherwise {
        s = otherwise
            .replace_all(&s, "$1$4. if $2, $3 instead.")
            .to_string();
    }
    // "X if C." and "If C, X." state the same condition (a trailing "if able" or "only
    // if" is something else).
    static TRAILING_IF: OnceLock<Option<Regex>> = OnceLock::new();
    let trailing = TRAILING_IF
        .get_or_init(|| Regex::new(r"(^|[.:—•] |\n)([^.:—•\n]+?) if ([^.,:\n]+)\.").ok());
    let Some(trailing) = trailing else {
        return s;
    };
    s = trailing
        .replace_all(&s, |c: &regex::Captures| {
            let (lead, body, cond) = (&c[1], &c[2], &c[3]);
            let keep = body.starts_with("if ")
                || body.ends_with(" only")
                || body.ends_with(" as though")
                || cond == "able"
                || cond.starts_with("able ")
                || body.contains(" unless ")
                || body.contains("\"");
            if keep {
                c[0].to_string()
            } else {
                format!("{lead}if {cond}, {body}.")
            }
        })
        .to_string();
    for (re, rep) in where_x_rewrites() {
        s = re.replace_all(&s, *rep).to_string();
    }
    s = scaled_for_each(&s);
    s = compared_to_x(&s);
    // "Choose target creature. Put a +1/+1 counter on it." / "Put a +1/+1 counter on
    // target creature.": the target is chosen as the spell or ability is put on the stack
    // either way (CR 601.2c, 602.2b); the instruction that first uses it names it.
    static CHOOSE_TARGET: OnceLock<Option<Regex>> = OnceLock::new();
    if let Some(re) = CHOOSE_TARGET.get_or_init(|| {
        Regex::new(r"(^|[.:—•] |\n|, )choose ((?:up to (?:one|1) |any number of )?target [^.,]+?)\. ((?:[a-z+/0-9{},-]+ ){1,8}?)(?:each of them|it|thatit|them|that creature|that permanent|that card|the chosen creature|the chosen card)\b").ok()
    }) {
        // Only an instruction that acts on it ("tap it", "put a +1/+1 counter on it"),
        // not a condition about something else ("if you control a creature with a
        // counter on it").
        s = re
            .replace_all(&s, |c: &regex::Captures| {
                let lead = &c[3];
                if lead.starts_with("if ") || lead.contains(" with ") || lead.starts_with("as ") {
                    return c[0].to_string();
                }
                format!("{}{}{}", &c[1], lead, &c[2])
            })
            .to_string();
    }
    // "Tap up to X target creatures, where X is V." / "up to V target creatures": the
    // number of targets X stands for, named in place (CR 107.3).
    static TARGETS_X: OnceLock<Option<Regex>> = OnceLock::new();
    if let Some(re) = TARGETS_X.get_or_init(|| {
        Regex::new(r"\b(up to |)x ((?:other )?target [^.]*?), where x is ([^.]+?)(\.|$)").ok()
    }) {
        // Only when that X is the only one the definition is for ("tap X target
        // creatures. They get -X/-0 ..." keeps its X).
        s = re
            .replace_all(&s, |c: &regex::Captures| {
                if c[2]
                    .split(|ch: char| !ch.is_alphanumeric())
                    .any(|w| w == "x")
                {
                    return c[0].to_string();
                }
                format!("{}{} {}{}", &c[1], &c[3], &c[2], &c[4])
            })
            .to_string();
    }
    // "When ~ blocks, at end of combat, destroy it." / "..., destroy it at end of
    // combat.": a delayed trigger's time after a trigger condition, too.
    static DELAYED: OnceLock<Option<Regex>> = OnceLock::new();
    if let Some(re) = DELAYED.get_or_init(|| {
        Regex::new(r#"(, )(at the beginning of the next end step|at the beginning of the next turn's upkeep|at the beginning of the next upkeep|at the beginning of the next cleanup step|at the beginning of your next upkeep|at end of combat), ([^.]+)\.(\n|$)"#).ok()
    }) {
        // Only a single instruction ending the ability: in "..., at end of combat, exile
        // it, then return it" or "... exile it. Return it ..." the time applies to more.
        s = re
            .replace_all(&s, |c: &regex::Captures| {
                if c[3].contains(" then ") || c[3].contains(", then") {
                    return c[0].to_string();
                }
                format!("{}{} {}.{}", &c[1], &c[3], &c[2], &c[4])
            })
            .to_string();
    }
    let Some(lead) = lead else {
        return s;
    };
    for _ in 0..3 {
        // The clause goes to the end of the sentence, before a "where X is" that defines
        // a number in it.
        let n = lead
            .replace_all(&s, |c: &regex::Captures| {
                let (start, clause, body) = (&c[1], &c[2], &c[3]);
                let duration = clause == "until end of turn" || clause == "this turn";
                match body.split_once(", where x is ") {
                    Some((head, x)) if duration && !body.contains('"') => {
                        format!("{start}{head} {clause}, where x is {x}.")
                    }
                    _ => format!("{start}{body} {clause}."),
                }
            })
            .to_string();
        if n == s {
            break;
        }
        s = n;
    }
    s
}

/// "Gets +2/+2 for each F", "deals 2 damage to you for each F", "draw two cards for each
/// F" -> "... X ..., where X is 2 times the number of F" (CR 107.3), as
/// [`where_x_rewrites`] does for one of each.
fn scaled_for_each(s: &str) -> String {
    static R: OnceLock<[Option<Regex>; 3]> = OnceLock::new();
    let [pt, damage, cards] = R.get_or_init(|| {
        [
            Regex::new(r"\b(gets?) ([+-])(\d+)/([+-])(\d+) ((?:until end of turn |this turn )?)for each ([^.]+?)(\.|$)").ok(),
            Regex::new(r"\b(deals?) (\d+) damage to ([^.]+?) for each ([^.]+?)(\.|$)").ok(),
            Regex::new(r"\b(draws?|mills?|discards?) (\d+) cards for each ([^.]+?)(\.|$)").ok(),
        ]
    });
    let mut s = s.to_string();
    if let Some(re) = pt {
        s = re
            .replace_all(&s, |c: &regex::Captures| {
                let (p, t) = (&c[3], &c[5]);
                // One number scales both (or one with the other 0): "+2/+2", "+2/+0".
                let n = match (p, t) {
                    (a, b) if a == b => a,
                    (a, "0") => a,
                    ("0", b) => b,
                    _ => return c[0].to_string(),
                };
                if n == "1" || n == "0" {
                    return c[0].to_string();
                }
                let x = |v: &str| if v == "0" { "0" } else { "x" };
                format!(
                    "{} {}{}/{}{} {}, where x is {n} times the number of {}{}",
                    &c[1],
                    &c[2],
                    x(p),
                    &c[4],
                    x(t),
                    c[6].trim_end(),
                    &c[7],
                    &c[8]
                )
            })
            .to_string();
    }
    if let Some(re) = damage {
        s = re
            .replace_all(&s, |c: &regex::Captures| {
                if &c[2] == "1" {
                    return c[0].to_string();
                }
                format!(
                    "{} x damage to {}, where x is {} times the number of {}{}",
                    &c[1], &c[3], &c[2], &c[4], &c[5]
                )
            })
            .to_string();
    }
    if let Some(re) = cards {
        s = re
            .replace_all(&s, |c: &regex::Captures| {
                if &c[2] == "1" {
                    return c[0].to_string();
                }
                format!(
                    "{} x cards, where x is {} times the number of {}{}",
                    &c[1], &c[2], &c[3], &c[4]
                )
            })
            .to_string();
    }
    s
}

/// "With mana value X or less ..., where X is V" -> "with mana value less than or equal to
/// V ...": the same comparison with the number named in place (CR 107.3).
fn compared_to_x(s: &str) -> String {
    static R: OnceLock<Option<(Regex, Regex)>> = OnceLock::new();
    let Some((re, word)) = R.get_or_init(|| {
        Some((
            Regex::new(r"\bx or (less|greater)\b([^.]*?), where x is ([^.]+?)(\.|$)").ok()?,
            Regex::new(r"\bx\b").ok()?,
        ))
    }) else {
        return s.to_string();
    };
    re.replace_all(s, |c: &regex::Captures| {
        // Only when that X is the only one the definition is for.
        if word.is_match(&c[2]) {
            return c[0].to_string();
        }
        format!("{} than or equal to {}{}{}", &c[1], &c[3], &c[2], &c[4])
    })
    .to_string()
}

/// Amounts stated as "equal to V" or "for each F" are rewritten to the "X ..., where X is
/// V" form (CR 107.3: X is defined by the text): both describe the same number.
fn where_x_rewrites() -> &'static [(Regex, &'static str)] {
    static R: OnceLock<Vec<(Regex, &'static str)>> = OnceLock::new();
    R.get_or_init(|| {
        [
            (r"\bdeals? damage equal to ([^.]+?) to ([^.]+?)(\.|$)", "deals x damage to $2, where x is $1$3"),
            (r"\bdeals? damage to ([^.]+?) equal to ([^.]+?)(\.|$)", "deals x damage to $1, where x is $2$3"),
            (r"\b(gains?|loses?) life equal to ([^.]+?)(\.|$)", "$1 x life, where x is $2$3"),
            (r"\b(gains?|loses?) 1 life for each ([^.]+?)(\.|$)", "$1 x life, where x is the number of $2$3"),
            (r"\b(gains?|loses?) (\d+|two|three|four|five) life for each ([^.]+?)(\.|$)", "$1 x life, where x is $2 times the number of $3$4"),
            (r"\b(gets?) ([+-])1/([+-])1 ((?:until end of turn |this turn )?)for each ([^.]+?)(\.|$)", "$1 ${2}x/${3}x $4, where x is the number of $5$6"),
            (r"\b(gets?) ([+-])1/([+-])0 ((?:until end of turn |this turn )?)for each ([^.]+?)(\.|$)", "$1 ${2}x/${3}0 $4, where x is the number of $5$6"),
            (r"\b(gets?) ([+-])0/([+-])1 ((?:until end of turn |this turn )?)for each ([^.]+?)(\.|$)", "$1 ${2}0/${3}x $4, where x is the number of $5$6"),
            (r"\b(draws?) cards equal to ([^.]+?)(\.|$)", "$1 x cards, where x is $2$3"),
            (r"\b(mills?) cards equal to ([^.]+?)(\.|$)", "$1 x cards, where x is $2$3"),
            (r"\bputs? an? (\S+) counter on ([^.]+?) for each ([^.]+?)(\.|$)", "put x $1 counters on $2, where x is the number of $3$4"),
            (r"\benters? with an? (\S+) counter on it for each ([^.]+?)(\.|$)", "enters with x $1 counters on it, where x is the number of $2$3"),
            (r"\benters? with (two|three|four|\d+) (\S+) counters on it for each ([^.]+?)(\.|$)", "enters with x $2 counters on it, where x is $1 times the number of $3$4"),
            (r"\b(enters?|puts?) (with )?a number of (\S+) counters on ([^.]+?) equal to ([^.]+?)(\.|$)", "$1 ${2}x $3 counters on $4, where x is $5$6"),
            (r"\b(draws?) a card for each ([^.]+?)(\.|$)", "$1 x cards, where x is the number of $2$3"),
            (r"\b(creates?) an? ([^.]+?) tokens?((?: with [^.]+?)?) for each ([^.]+?)(\.|$)", "$1 x $2 tokens$3, where x is the number of $4$5"),
            (r"\b(mills?) a card for each ([^.]+?)(\.|$)", "$1 x cards, where x is the number of $2$3"),
            (r"\b(creates?) a number of ([^.]+?) tokens?((?: with [^.]+?)?) equal to ([^.]+?)(\.|$)", "$1 x $2 tokens$3, where x is $4$5"),
            (r"\b(discards?|draws?|mills?) a number of cards equal to ([^.]+?)(\.|$)", "$1 x cards, where x is $2$3"),
            (r"(^|[.:—•] |\n|, )for each ([^,.]+), (creates?) an? ([^.]+?) tokens?((?: with [^.]+?)?)(\.|$)", "${1}$3 x $4 tokens$5, where x is the number of $2$6"),
            (r"\bputs? a number of (\S+) counters equal to ([^.]+?) on ([^.]+?)(\.|$)", "put x $1 counters on $3, where x is $2$4"),
        ]
        .into_iter()
        .filter_map(|(p, r)| Regex::new(p).ok().map(|re| (re, r)))
        .collect()
    })
}

/// Ability words (CR 207.2c): they have no rules meaning.
const ABILITY_WORDS: &[&str] = &[
    "adamant",
    "addendum",
    "alliance",
    "battalion",
    "bloodrush",
    "celebration",
    "channel",
    "chroma",
    "cohort",
    "constellation",
    "converge",
    "council's dilemma",
    "coven",
    "delirium",
    "descend 4",
    "descend 8",
    "disappear",
    "domain",
    "eerie",
    "eminence",
    "enrage",
    "fateful hour",
    "fathomless descent",
    "ferocious",
    "flurry",
    "formidable",
    "grandeur",
    "hellbent",
    "heroic",
    "imprint",
    "infusion",
    "inspired",
    "join forces",
    "kinship",
    "landfall",
    "lieutenant",
    "magecraft",
    "metalcraft",
    "morbid",
    "opus",
    "pack tactics",
    "paradox",
    "parley",
    "radiance",
    "raid",
    "rally",
    "renew",
    "repartee",
    "revolt",
    "secret council",
    "spell mastery",
    "strive",
    "survival",
    "sweep",
    "tempting offer",
    "threshold",
    "undergrowth",
    "valiant",
    "vivid",
    "void",
    "will of the council",
];

/// Labels before an em dash that aren't ability or flavor words.
const NOT_FLAVOR: &[&str] = &[
    "companion",
    "boast",
    "exhaust",
    "forecast",
    "max speed",
    "power-up",
    "to solve",
    "solved",
    "choose",
    "level up",
    "ward",
    "equip",
    "cumulative upkeep",
    "echo",
];

/// Strips a leading ability word (CR 207.2c) or flavor word (CR 207.2d): "Landfall — ".
fn strip_ability_word(line: &str) -> String {
    let Some((head, rest)) = line.split_once(" — ") else {
        return line.to_string();
    };
    // Saga chapters with a flavor word: "I — Aerial Blast — effect" (CR 714.2b, 207.2d).
    if head
        .split(", ")
        .all(|n| !n.is_empty() && n.chars().all(|c| matches!(c, 'I' | 'V' | 'X')))
    {
        let inner = strip_ability_word(rest);
        return format!("{head} — {inner}");
    }
    let h = head.trim().to_lowercase().replace('\u{2019}', "'");
    if ABILITY_WORDS.contains(&h.as_str()) {
        return rest.to_string();
    }
    let words: Vec<&str> = head.split_whitespace().collect();
    let flavor = !words.is_empty()
        && words.len() <= 7
        && head.chars().next().is_some_and(|c| c.is_uppercase())
        && !head.contains(['{', ':', '"', ',', '~', '\n', '•', '|'])
        && !head.chars().any(|c| c.is_ascii_digit())
        && !NOT_FLAVOR.iter().any(|n| h.starts_with(n))
        && !KeywordKind::ALL
            .iter()
            .any(|k| h.starts_with(&k.name().to_lowercase()))
        && !head
            .split(", ")
            .all(|n| n.chars().all(|c| matches!(c, 'I' | 'V' | 'X')));
    if flavor {
        rest.to_string()
    } else {
        line.to_string()
    }
}

/// Splits a face's normalized Oracle text into comparison units: one per ability line,
/// keyword lines split into one unit per keyword, bullets joined to their modal line.
pub fn oracle_units(text: &str, names: &[String]) -> Vec<String> {
    let text = strip_reminder(text);
    let text = self_refs(&text, names);
    let mut lines: Vec<String> = Vec::new();
    for l in text.lines() {
        let l = l.trim();
        if l.is_empty() {
            continue;
        }
        if l.starts_with('•') || l.starts_with(|c: char| c.is_ascii_digit()) && l.contains('|') {
            if let Some(last) = lines.last_mut() {
                last.push('\n');
                // A mode's flavor word ("• Cure Wounds — You gain 2 life.", CR 207.2d).
                match l.strip_prefix("• ") {
                    Some(m) => {
                        last.push_str("• ");
                        last.push_str(&strip_ability_word(m));
                    }
                    None => last.push_str(l),
                }
                continue;
            }
        }
        lines.push(strip_ability_word(l));
    }
    let mut out = Vec::new();
    for l in lines {
        match keyword_items(&l) {
            Some(items) => out.extend(items),
            None => out.push(l),
        }
    }
    out
}

/// If a line is a keyword line, its keywords (one per protection quality).
fn keyword_items(line: &str) -> Option<Vec<String>> {
    let l = line.trim().trim_end_matches('.');
    if l.contains('"') || l.contains(':') {
        return None;
    }
    let lower = l.to_lowercase();
    let starts_kw = |s: &str| -> bool {
        let s = s.trim();
        s.ends_with("walk")
            || s.contains("cycling")
            || s.starts_with("partner with")
            || s.starts_with("bands with other")
            || s.starts_with("megamorph")
            || s.starts_with("multikicker")
            || s.starts_with("daybound")
            || s.starts_with("nightbound")
            || s.starts_with("totem armor")
            || KeywordKind::ALL.iter().any(|k| {
                let n = k.name().to_lowercase();
                s.starts_with(&n)
                    && s[n.len()..]
                        .chars()
                        .next()
                        .is_none_or(|c| !c.is_alphanumeric())
            })
    };
    if !starts_kw(&lower) {
        return None;
    }
    if l.contains('—') {
        return Some(vec![l.to_string()]);
    }
    // Split on top-level commas/semicolons.
    let mut items: Vec<String> = Vec::new();
    let mut depth = 0;
    let mut cur = String::new();
    for ch in l.chars() {
        match ch {
            '{' => {
                depth += 1;
                cur.push(ch)
            }
            '}' => {
                depth -= 1;
                cur.push(ch)
            }
            ',' | ';' if depth == 0 => items.push(std::mem::take(&mut cur)),
            c => cur.push(c),
        }
    }
    items.push(cur);
    let mut merged: Vec<String> = Vec::new();
    for it in items {
        let t = it.trim().to_string();
        if t.is_empty() {
            continue;
        }
        let tl = t.to_lowercase();
        let continues = merged.last().is_some_and(|p| {
            let p = p.to_lowercase();
            (p.starts_with("protection from") || p.starts_with("hexproof from"))
                && (tl.starts_with("from ") || tl.starts_with("and from ") || !starts_kw(&tl))
        });
        if continues {
            if let Some(last) = merged.last_mut() {
                last.push_str(", ");
                last.push_str(&t);
            }
        } else {
            if !starts_kw(&tl) {
                return None;
            }
            merged.push(t);
        }
    }
    // "Protection from black and from red" stands for one ability per quality
    // (CR 702.16g, 702.11f); "from each color" for one per color (CR 702.16h).
    let mut out = Vec::new();
    for m in merged {
        let ml = m.to_lowercase();
        let head = if ml.starts_with("protection from ") {
            Some("protection from ")
        } else if ml.starts_with("hexproof from ") {
            Some("hexproof from ")
        } else {
            None
        };
        match head {
            Some(h) => {
                let rest = &m[h.len()..];
                let rl = rest.to_lowercase();
                if rl == "each color" || rl == "all colors" {
                    for c in ["white", "blue", "black", "red", "green"] {
                        out.push(format!("{h}{c}"));
                    }
                    continue;
                }
                let parts: Vec<String> = rest
                    .split(", and from ")
                    .flat_map(|p| p.split(", from "))
                    .flat_map(|p| p.split(" and from "))
                    .map(|p| p.trim().to_string())
                    .collect();
                for p in parts {
                    out.push(format!("{h}{p}"));
                }
            }
            None => out.push(m),
        }
    }
    Some(out)
}

/// The result of comparing one card.
#[derive(Clone, Debug)]
pub struct CardCheck {
    pub name: String,
    pub pass: bool,
    /// Per face: (Oracle units, rendered units).
    pub faces: Vec<(Vec<String>, Vec<String>)>,
    /// Gaps found while rendering.
    pub gaps: Vec<String>,
    /// Oracle units with no match, and rendered units with no match.
    pub unmatched_oracle: Vec<String>,
    pub unmatched_rendered: Vec<String>,
    /// Some of the card's abilities are hand-written (`crate::cards`), not compiled from
    /// its text: what they do is in code, which the renderer can't put into words.
    pub hand_written: bool,
}

/// Renders a card and compares it with its Oracle text.
pub fn check_card(def: &CardDef) -> CardCheck {
    let rendered: Vec<RenderedFace> = render_card(def);
    let mut faces = Vec::new();
    let mut gaps = Vec::new();
    let mut unmatched_oracle = Vec::new();
    let mut unmatched_rendered = Vec::new();
    let mut pass = true;
    for (i, (face, r)) in def.faces.iter().zip(rendered.iter()).enumerate() {
        let names = self_names(def, i);
        let oracle = oracle_units(&face.chars.rules_text, &names);
        let mut c = compare_face(&r.lines, &oracle, &names);
        if !c.0.is_empty() || !c.1.is_empty() {
            if let Some(m) = &r.merged {
                let c2 = compare_face(m, &oracle, &names);
                if c2.0.is_empty() && c2.1.is_empty() {
                    c = c2;
                }
            }
        }
        gaps.extend(r.gaps.iter().cloned());
        let (uo, ur, mine) = c;
        if !uo.is_empty() || !ur.is_empty() {
            pass = false;
            unmatched_oracle.extend(uo);
            unmatched_rendered.extend(ur);
        }
        faces.push((oracle, mine));
    }
    if !gaps.is_empty() {
        pass = false;
    }
    CardCheck {
        name: def.name.to_string(),
        pass,
        faces,
        gaps,
        unmatched_oracle,
        unmatched_rendered,
        hand_written: !def.manual_text().is_empty(),
    }
}

/// Compares rendered lines with a face's Oracle units: (unmatched Oracle units,
/// unmatched rendered units, the rendered units), both unmatched lists empty when the face
/// matches.
fn compare_face(
    lines: &[String],
    oracle: &[String],
    names: &[String],
) -> (Vec<String>, Vec<String>, Vec<String>) {
    let mine: Vec<String> = lines
        .iter()
        .flat_map(|l| {
            let l = self_refs(l, names);
            match keyword_items(&l) {
                Some(items) => items,
                None => vec![l],
            }
        })
        .collect();
    let (uo, ur) = diff_units(oracle, &mine);
    if uo.is_empty() && ur.is_empty() {
        return (uo, ur, mine);
    }
    // Fall back to comparing in order (one line may compile to several abilities, or
    // several lines to one): first with the units that matched one to one left out, then
    // the whole face.
    let in_order = |o: &[String], m: &[String]| {
        let all_o: Vec<String> = o.iter().flat_map(|u| normalize_unit(u)).collect();
        let all_m: Vec<String> = m.iter().flat_map(|u| normalize_unit(u)).collect();
        let units_m: Vec<Vec<String>> = m.iter().map(|u| normalize_unit(u)).collect();
        let mut starts = Vec::new();
        let mut n = 0;
        for u in o {
            starts.push(n);
            n += normalize_unit(u).len();
        }
        tokens_match(&all_o, &all_m) || shared_subject_match(&all_o, &starts, &units_m)
    };
    if in_order(&uo, &ur) || in_order(oracle, &mine) {
        return (Vec::new(), Vec::new(), mine);
    }
    (uo, ur, mine)
}

/// Whether two normalized token sequences are the same. The renderer's `~it` (the object
/// itself, just mentioned) matches "~" or "it": cards refer to an object that a trigger
/// condition just named either by its name or by "it".
///
/// A rendering can also say that a part is optional, `{opt:it}`, or give alternative
/// wordings, `{alt:that player|its controller}`, where the AST alone can't tell which one a
/// card uses for the same meaning ("~ gets +1/+0 and can't be blocked" / "~ gets +1/+0.
/// It can't be blocked.").
pub fn tokens_match(a: &[String], b: &[String]) -> bool {
    seq_match(a, b) || seq_match(b, a)
}

/// An `{opt:...}` / `{alt:...|...}` token: (optional, alternatives).
fn special_token(t: &str) -> Option<(bool, Vec<Vec<String>>)> {
    if let Some(inner) = t.strip_prefix("{opt:").and_then(|x| x.strip_suffix('}')) {
        return Some((true, vec![normalize_unit(inner)]));
    }
    if let Some(inner) = t.strip_prefix("{alt:").and_then(|x| x.strip_suffix('}')) {
        return Some((false, split_top_level(inner).map(normalize_unit).collect()));
    }
    None
}

/// The alternatives of an `{alt:...}` token: split at each `|` outside nested braces.
fn split_top_level(s: &str) -> impl Iterator<Item = &str> {
    let mut parts = Vec::new();
    let (mut depth, mut start) = (0usize, 0);
    for (i, ch) in s.char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => depth = depth.saturating_sub(1),
            '|' if depth == 0 => {
                parts.push(&s[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    parts.push(&s[start..]);
    parts.into_iter()
}

/// Matches `r` (which may contain special tokens) against `o`.
fn seq_match(r: &[String], o: &[String]) -> bool {
    fn go(
        r: &[String],
        o: &[String],
        i: usize,
        j: usize,
        failed: &mut std::collections::HashSet<(usize, usize)>,
    ) -> bool {
        if i == r.len() {
            return o[j..].iter().all(|t| is_quantifier(t));
        }
        if failed.contains(&(i, j)) {
            return false;
        }
        // A quantifier one side leaves out (see `OPTIONAL_QUANTIFIERS`).
        let r_q = is_quantifier(&r[i]);
        let o_q = o.get(j).is_some_and(|t| is_quantifier(t));
        if (r_q && !o_q && go(r, o, i + 1, j, failed))
            || (o_q && !r_q && go(r, o, i, j + 1, failed))
        {
            return true;
        }
        let ok = match special_token(&r[i]) {
            Some((optional, alts)) => {
                (optional && go(r, o, i + 1, j, failed))
                    || alts.iter().any(|alt| {
                        (j..=o.len())
                            .any(|k| seq_match(alt, &o[j..k]) && go(r, o, i + 1, k, failed))
                    })
            }
            None => j < o.len() && token_eq(&r[i], &o[j]) && go(r, o, i + 1, j + 1, failed),
        };
        if !ok {
            failed.insert((i, j));
        }
        ok
    }
    // Fast path: no special tokens or quantifiers.
    if !r
        .iter()
        .chain(o)
        .any(|t| t.starts_with("{opt:") || t.starts_with("{alt:") || is_quantifier(t))
    {
        return r.len() == o.len() && r.iter().zip(o).all(|(x, y)| token_eq(x, y));
    }
    go(r, o, 0, 0, &mut std::collections::HashSet::new())
}

/// Token equality for [`tokens_match`].
///
/// "That creature" (normalized to `thatit`) matches "it", but not the object itself
/// (`~it`): once a card has named itself again, it calls another object "that creature"
/// precisely because "it" would now be itself ("Whenever another creature enters,
/// sacrifice ~. If you do, destroy that creature."). The renderer says `that-object` for
/// such an object, which matches only "that creature".
pub fn token_eq(x: &str, y: &str) -> bool {
    x == y
        || (x == "~it" && (y == "~" || y == "it"))
        || (y == "~it" && (x == "~" || x == "it"))
        || (x == "thatit" && y == "it")
        || (y == "thatit" && x == "it")
        || (x == "that-object" && y == "thatit")
        || (y == "that-object" && x == "thatit")
}

/// Whether the rendered units, in order, spell the Oracle tokens when a unit may drop
/// the subject it shares with the previous unit: two abilities printed on one line with
/// one subject ("Enchanted creature gets +1/+0 and can't be blocked.").
///
/// `starts` are the positions where Oracle units (lines) begin: the shared subject is
/// left out only inside an Oracle line, never at the start of one.
fn shared_subject_match(oracle: &[String], starts: &[usize], units: &[Vec<String>]) -> bool {
    // Where a unit can end when it starts at `pos`: a quantifier left out on one side
    // (`OPTIONAL_QUANTIFIERS`) changes the length.
    let ends = |t: &[String], pos: usize| -> Vec<usize> {
        (t.len().saturating_sub(4)..=t.len() + 4)
            .filter(|n| pos + n <= oracle.len() && tokens_match(&oracle[pos..pos + n], t))
            .map(|n| pos + n)
            .collect()
    };
    // A unit may also leave out a trailing condition it shares with the next unit: "~
    // gets +2/+2 and can't block as long as ..." states the condition once for both
    // (a sentence's condition applies to all of it, see `SENTENCE_FORMS`).
    let variants = |i: usize| -> Vec<(&[String], bool)> {
        let u = &units[i];
        let mut v: Vec<(&[String], bool)> = vec![(u.as_slice(), false)];
        if let Some(next) = units.get(i + 1) {
            let lcs = u
                .iter()
                .rev()
                .zip(next.iter().rev())
                .take_while(|(a, b)| a == b)
                .count();
            for k in (3..=lcs).rev() {
                let suf = &u[u.len() - k..];
                if suf[..3] == ["as", "long", "as"] && k < u.len() {
                    v.push((&u[..u.len() - k], true));
                    break;
                }
            }
        }
        v
    };
    // Positions reached, and where the unit ending there began if it left out a
    // condition: then the next unit continues the same sentence, either sharing its
    // subject ("~ gets +2/+2 and can't block as long as ...") or naming another one ("~
    // gets +2/+2 and creatures you control have vigilance"), not repeating the subject as
    // a new sentence would ("~ gets +2/+2. ~ can't block as long as ...", whose condition
    // doesn't apply to the first sentence; the comparison doesn't see the period).
    let mut positions = std::collections::BTreeSet::from([(0usize, None::<usize>)]);
    for i in 0..units.len() {
        let mut next = std::collections::BTreeSet::new();
        for (u, dropped) in variants(i) {
            // A dropped condition is said by the next unit, in the same Oracle line.
            let ends = |t: &[String], pos: usize, from: usize| -> Vec<(usize, Option<usize>)> {
                ends(t, pos)
                    .into_iter()
                    .filter(|e| !dropped || (*e < oracle.len() && !starts.contains(e)))
                    .map(|e| (e, dropped.then_some(from)))
                    .collect()
            };
            for &(pos, dropped_from) in &positions {
                if dropped_from.is_none_or(|f| oracle[f] != oracle[pos]) {
                    next.extend(ends(u, pos, pos));
                }
                if i == 0 || starts.contains(&pos) {
                    continue;
                }
                let prev = &units[i - 1];
                let lcp = prev
                    .iter()
                    .zip(u)
                    .take_while(|(a, b)| token_eq(a, b))
                    .count();
                for k in (1..=lcp.min(6)).rev() {
                    // The dropped words must be the whole subject of both units: a predicate
                    // starts right after them in each ("creatures you control have haste and
                    // attack ..." can't drop just "creatures" of "creatures attack ..."), or
                    // they end with a shared verb ("~ enters tapped and with ... counters").
                    let verb_shared = starts_predicate(&u[k - 1..]);
                    if !verb_shared && (!starts_predicate(&prev[k..]) || !starts_predicate(&u[k..]))
                    {
                        continue;
                    }
                    next.extend(ends(&u[k..], pos, pos));
                }
            }
        }
        if next.is_empty() {
            return false;
        }
        positions = next;
    }
    positions.contains(&(oracle.len(), None))
}

/// Whether normalized tokens begin with a verb (the predicate after a subject).
fn starts_predicate(t: &[String]) -> bool {
    const VERBS: &[&str] = &[
        "have",
        "get",
        "gain",
        "lose",
        "is",
        "was",
        "can't",
        "can",
        "cannot",
        "attack",
        "block",
        "deal",
        "don't",
        "enter",
        "become",
        "must",
        "cost",
        "untap",
        "tap",
        "may",
        "phase",
        "fight",
        "explore",
        "connive",
        "put",
        "draw",
        "discard",
        "create",
        "return",
        "sacrifice",
        "exile",
        "destroy",
        "add",
        "gets",
        "assign",
        "skip",
        "play",
        "cast",
        "look",
        "reveal",
        "search",
        "shuffle",
        "mill",
        "scry",
        "surveil",
        "win",
        "transform",
        "perpetually",
        "also",
        "isn't",
        "wasn't",
        "do",
        "remain",
        "count",
        "loses",
        "deals",
        "would",
        "copy",
        "counter",
        "choose",
        "pay",
        "planeswalk",
        "venture",
        "investigate",
        "proliferate",
        "regenerate",
        "mutate",
    ];
    t.first().is_some_and(|w| VERBS.contains(&w.as_str()))
}

/// Multiset difference of units by normalized tokens.
fn diff_units(oracle: &[String], mine: &[String]) -> (Vec<String>, Vec<String>) {
    let mut mine_left: Vec<(Vec<String>, &String)> =
        mine.iter().map(|m| (normalize_unit(m), m)).collect();
    let mut uo = Vec::new();
    for o in oracle {
        let n = normalize_unit(o);
        if let Some(pos) = mine_left.iter().position(|(m, _)| tokens_match(&n, m)) {
            mine_left.remove(pos);
        } else {
            uo.push(o.clone());
        }
    }
    let ur = mine_left.into_iter().map(|(_, m)| m.clone()).collect();
    (uo, ur)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn units(texts: &[&str]) -> Vec<Vec<String>> {
        texts.iter().map(|t| normalize_unit(t)).collect()
    }

    /// A condition stated once for "A and B" applies to both; for "A. As long as X, B"
    /// it applies only to B, so rendering the condition on A too must not match.
    #[test]
    fn a_shared_condition_covers_only_its_own_sentence() {
        let m = units(&[
            "~ gets +2/+2 as long as you control an Island.",
            "~ has flying as long as you control an Island.",
        ]);
        let same_sentence =
            normalize_unit("As long as you control an Island, ~ gets +2/+2 and has flying.");
        assert!(shared_subject_match(&same_sentence, &[0], &m));
        let other_subject = normalize_unit(
            "As long as you control an Island, ~ gets +2/+2 and creatures you control have flying.",
        );
        let m2 = units(&[
            "~ gets +2/+2 as long as you control an Island.",
            "Creatures you control have flying as long as you control an Island.",
        ]);
        assert!(shared_subject_match(&other_subject, &[0], &m2));
        let two_sentences =
            normalize_unit("~ gets +2/+2. As long as you control an Island, ~ has flying.");
        assert!(!shared_subject_match(&two_sentences, &[0], &m));
    }
}
