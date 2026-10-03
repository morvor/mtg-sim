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

fn count_subtype(t: &TestGame, p: PlayerId, sub: &str) -> usize {
    t.g.battlefield
        .iter()
        .filter(|id| {
            let o = t.g.obj(**id);
            o.controller == p && o.chars.has_subtype(sub)
        })
        .count()
}

fn treasures(t: &TestGame, p: PlayerId) -> usize {
    count_subtype(t, p, "Treasure")
}

#[test]
fn the_attacking_player_creates_the_treasure_once_per_attack() {
    cr!("508.3e", "603.2c");
    // Jolene, the Plunder Queen: "Whenever a player attacks one or more of your
    // opponents, that attacking player creates a Treasure token." "That attacking player"
    // was the attacked player, and attacking two of your opponents made two Treasures.
    supported("Jolene, the Plunder Queen");
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Jolene, the Plunder Queen");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, mtg_engine::turn::Step::BeginningOfCombat);
    t.attack(
        &[(a, Entity::Player(P1)), (b, Entity::Player(P2))],
        &[],
    );
    t.resolve_all();
    // Jolene doubles nothing here: "one or more Treasure tokens ... plus an additional
    // Treasure token" makes the one Treasure two.
    assert_eq!(treasures(&t, P1), 0);
    assert_eq!(treasures(&t, P2), 0);
    assert_eq!(treasures(&t, P0), 2);
}

#[test]
fn that_attacking_player_is_the_player_who_attacked() {
    cr!("508.3e", "303.4a");
    // Curse of Shallow Graves: "Whenever a player attacks enchanted player with one or
    // more creatures, that attacking player may create a tapped 2/2 black Zombie creature
    // token." "That attacking player" is now the controller of the attacking creatures
    // for every such trigger (it was the trigger's player, the attacked one, for
    // Jolene's); the attacked player gets nothing.
    supported("Curse of Shallow Graves");
    let mut t = TestGame::new(2);
    let curse = t.battlefield(P0, "Curse of Shallow Graves");
    assert!(t.g.attach(curse, Entity::Player(P1)));
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, mtg_engine::turn::Step::BeginningOfCombat);
    t.attack(&[(bears, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert_eq!(count_subtype(&t, P1, "Zombie"), 0);
    assert_eq!(count_subtype(&t, P0, "Zombie"), 1);
}
