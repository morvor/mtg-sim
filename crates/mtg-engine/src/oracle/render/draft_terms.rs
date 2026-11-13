//! The draft abilities of Conspiracy cards (CR 905.2; `crate::draft`, `kw/draft_4.rs`),
//! put into words from what each one does during the draft, and the game's references to
//! what was noted or removed then.

use super::*;
use crate::draft as d;

impl Renderer<'_> {
    /// A static ability that functions during the draft (CR 905.2b, 905.2c).
    pub(crate) fn draft_static(&mut self, name: &str) -> Option<String> {
        // The ":creature" suffix limits "as you draft a card" to creature cards.
        let (base, kind) = match name.strip_suffix(":creature") {
            Some(b) => (b, "creature card"),
            None => (name, "card"),
        };
        Some(match base {
            // CR 905.2c.
            d::DRAFT_FACE_UP => "Draft ~ face up".into(),
            d::DRAFT_NOTE_COUNT => "Reveal ~ as you draft it and note how many cards you've drafted this draft round, including ~".into(),
            d::DRAFT_NOTE_PASSER => {
                "Reveal ~ as you draft it and note the player who passed it to you".into()
            }
            d::DRAFT_NOTE_NEXT_NAME => {
                "Reveal ~ as you draft it. Reveal the next card you draft and note its name".into()
            }
            d::DRAFT_REMOVE_FACE_UP => {
                "As you draft a card, you may remove it from the draft face up".into()
            }
            d::DRAFT_REMOVE_FACE_DOWN => {
                "As you draft a card, you may remove it from the draft face down".into()
            }
            d::DRAFT_NOTE_NAME => format!(
                "As you draft a {kind}, you may reveal it, note its name, then turn ~ face down"
            ),
            // Only creature cards' types are noted (`Draft::pick_with`).
            d::DRAFT_NOTE_TYPES => "As you draft a creature card, you may reveal it, note its creature types, then turn ~ face down".into(),
            d::DRAFT_THREE_COLORS => "Reveal ~ as you draft it. The player to your right chooses a color, you choose another color, then the player to your left chooses a third color".into(),
            d::DRAFT_ADDITIONAL_PUT_BACK => "As you draft a card, you may draft an additional card from that booster pack. If you do, put ~ into that booster pack".into(),
            d::DRAFT_ADDITIONAL_SKIP_NEXT => "As you draft a card, you may draft an additional card from that booster pack. If you do, turn ~ face down, then pass the next booster pack without drafting a card from it".into(),
            d::DRAFT_GUESS_NEXT => "Reveal ~ as you draft it. The next time a player drafts a card from this booster pack, guess that card's name. Then that player reveals the drafted card".into(),
            d::DRAFT_LOOK_NEXT => "Reveal ~ as you draft it. You may look at the next card drafted from this booster pack".into(),
            d::DRAFT_ADD_BOOSTER => "Reveal ~ as you draft it. After you draft ~, you may add a booster pack to the draft".into(),
            d::DRAFT_LAST_CARD_TO_DRAFTER => "Each player passes the last card from each booster pack to a player who drafted a card named ~".into(),
            _ => return None,
        })
    }

    /// A condition about the draft: "you guessed correctly for a card named ~", "you
    /// removed a creature card with flying from the draft with cards named ~".
    pub(crate) fn draft_condition(&mut self, name: &str) -> Option<String> {
        if name == d::GUESSED_CORRECTLY {
            return Some("you guessed correctly for a card named ~".into());
        }
        let kw = name
            .strip_prefix(d::REMOVED_CREATURE_WITH)?
            .strip_prefix(':')?;
        let kind = crate::keywords::KeywordKind::from_name(kw)?;
        Some(format!(
            "you removed a creature card with {} from the draft with cards named ~",
            kind.name().to_lowercase()
        ))
    }

    /// A number about the draft.
    pub(crate) fn draft_value(&mut self, name: &str) -> Option<String> {
        Some(match name {
            d::REMOVED_COUNT => {
                "the number of cards you removed from the draft with cards named ~".into()
            }
            d::HIGHEST_NOTED => "the highest number you noted for cards named ~".into(),
            _ => return None,
        })
    }
}

/// The keyword of a static "~ has the [keyword] abilities of the creature cards you removed
/// from the draft with cards named ~" (`kw/draft_4.rs`), if `a` is one.
fn removed_keyword(a: &Ability) -> Option<crate::keywords::KeywordKind> {
    let AbilityKind::Static(s) = &a.kind else {
        return None;
    };
    let StaticEffect::Continuous {
        affected: Filter::Source,
        mods,
    } = &s.effect
    else {
        return None;
    };
    let [Modification::Custom { name: n, .. }] = mods.as_slice() else {
        return None;
    };
    if s.condition.is_some() {
        return None;
    }
    let kw = n
        .strip_prefix(crate::kw::draft_4::REMOVED_KEYWORDS)?
        .strip_prefix(':')?;
    crate::keywords::KeywordKind::from_name(kw)
}

/// "If you removed a creature card with flying from the draft with cards named ~, ~ has
/// flying. The same is true for first strike, ...": one static ability for each keyword
/// (Animus of Predation), as one line, and the number of abilities it stands for.
pub(crate) fn removed_keywords_group(abilities: &[Ability]) -> Option<(usize, String)> {
    let kinds: Vec<String> = abilities
        .iter()
        .map_while(removed_keyword)
        .map(|k| k.name().to_lowercase())
        .collect();
    let (first, rest) = kinds.split_first()?;
    let mut line = format!(
        "If you removed a creature card with {first} from the draft with cards named ~, ~ has {first}."
    );
    if !rest.is_empty() {
        line.push_str(&format!(
            " The same is true for {}.",
            join_list(rest, "and")
        ));
    }
    Some((kinds.len(), line))
}
