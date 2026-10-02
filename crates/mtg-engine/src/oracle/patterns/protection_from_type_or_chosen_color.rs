//! "[object] gains protection from [card type]s or from the color of your choice until
//! end of turn" (Apostle's Blessing: "Target artifact or creature you control gains
//! protection from artifacts or from the color of your choice until end of turn."). The
//! target is chosen as the spell is cast; the quality — the card type or one of the
//! colors — is chosen as it resolves (CR 702.16a, 608.2c).

use super::EffectPattern;
use crate::ability::*;
use crate::keywords::{Keyword, KeywordKind};
use crate::oracle::effects::{object_ref, Builder};
use crate::oracle::phrases::end;
use crate::types::{CardType, Color};

fn protection_from_type_or_color(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let body = l.strip_suffix(" until end of turn")?;
    let (subj, rest) = body.split_once(" gains protection from ")?;
    let types = rest.strip_suffix(" or from the color of your choice")?;
    let ty = CardType::from_word(types.strip_suffix('s')?)?;
    let (what, r) = object_ref(subj, b)?;
    if !r.trim().is_empty() || matches!(what, Sel::None) {
        return None;
    }
    let mut options = vec![ty.word().to_string()];
    options.extend(Color::ALL.iter().map(|c| c.word().to_string()));
    let protection = |f: Filter| {
        let mut kw = Keyword::new(KeywordKind::Protection);
        kw.filter = Some(f);
        Effect::Modify {
            what: what.clone(),
            mods: vec![Modification::AddKeyword(kw)],
            duration: Duration::EndOfTurn,
        }
    };
    // Only the quality chosen now counts: a color chosen by an earlier resolution of
    // the same ability doesn't linger alongside a chosen card type.
    Some(Effect::Seq(vec![
        Effect::Choose {
            who: PlayerRef::You,
            kind: ChoiceKind::OneOf(options),
        },
        Effect::If {
            cond: Condition::ChosenWord(ty.word().to_string()),
            then: Box::new(protection(Filter::Type(ty))),
            otherwise: Box::new(protection(Filter::ChosenColor)),
        },
    ]))
}

inventory::submit! { EffectPattern { name: "gains protection from [card type]s or from the color of your choice", priority: 60, parse: protection_from_type_or_color } }
