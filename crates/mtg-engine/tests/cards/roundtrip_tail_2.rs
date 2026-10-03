//! Round-trip follow-up `roundtrip-tail-2` (cards E–L): renderer checks, and in-game
//! tests for compiler misreads the round trip found (each shows the behavior the
//! corrected compilation has and the old one didn't).

use mtg_engine::ability::*;
use mtg_engine::card::card;
use mtg_engine::oracle::render::compare::{normalize_unit, tokens_match};
use mtg_engine::oracle::render::{render_ability, FaceInfo};
use mtg_engine::*;

#[allow(dead_code)]
fn supported(name: &str) {
    let c = card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name}: {:?}",
        c.unsupported_text()
    );
}

fn same(a: &str, b: &str) -> bool {
    tokens_match(&normalize_unit(a), &normalize_unit(b))
}

#[test]
fn a_choice_made_on_resolution_doesnt_render_as_a_modal_choice() {
    cr!("700.2a", "608.2d");
    // "Choose one —" with a bulleted list is a modal spell: the mode is chosen as it's
    // cast (CR 700.2a). A choice the effect offers as it resolves (CR 608.2d) is a
    // different ability, so a parser that compiled one as the other must mismatch.
    let options = vec![
        (
            String::new(),
            Effect::Draw {
                who: PlayerRef::You,
                n: Value::Const(1),
            },
        ),
        (
            String::new(),
            Effect::GainLife {
                who: PlayerRef::You,
                n: Value::Const(3),
            },
        ),
    ];
    let a = AbilityDef::new(
        AbilityKind::Spell(SpellAbility {
            body: Body::effect(Effect::ChooseOne {
                who: PlayerRef::You,
                options,
            }),
        }),
        "",
    );
    let r = render_ability(&a, &FaceInfo::default()).expect("renders");
    assert!(
        !same("Choose one — • Draw a card. • You gain 3 life.", &r),
        "{r}"
    );
    // The resolution-time choice is still worded as one ("draw a card or gain 3 life").
    assert!(same("Draw a card or gain 3 life.", &r), "{r}");
}
