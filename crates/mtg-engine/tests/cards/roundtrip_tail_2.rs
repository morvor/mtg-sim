//! Round-trip follow-up `roundtrip-tail-2` (cards E–L): renderer checks, and in-game
//! tests for compiler misreads the round trip found (each shows the behavior the
//! corrected compilation has and the old one didn't).

use mtg_engine::ability::*;
use mtg_engine::card::card;
use mtg_engine::oracle::render::compare::{normalize_unit, tokens_match};
use mtg_engine::oracle::render::{render_ability, FaceInfo};
use mtg_engine::testing::*;
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

#[test]
fn a_wish_reveals_the_card_it_puts_into_your_hand() {
    cr!("701.20a", "108.3b");
    // Golden Wish: "You may reveal an artifact or enchantment card you own from outside
    // the game and put it into your hand." The card was put into the hand without being
    // revealed.
    supported("Golden Wish");
    let mut t = TestGame::new(2);
    t.g.logging = true;
    let side = t.g.add_to_sideboard(P0, vec![card("Ornithopter")]);
    let wish = t.hand(P0, "Golden Wish");
    t.lands(P0, "Plains", 5);
    t.answer_choose(P0, &[Entity::Object(side[0])]);
    t.cast(P0, wish).go();
    t.resolve();
    assert!(t.in_hand(P0, "Ornithopter"));
    assert!(
        t.g.log.iter().any(|e| e.text.contains("reveals") && e.text.contains("Ornithopter")),
        "{:?}",
        t.g.log.iter().map(|e| &e.text).collect::<Vec<_>>()
    );
}
