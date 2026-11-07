//! The worked examples of CR 123 (stickers), played out as tests.

use super::r114_common::*;
use mtg_engine::ability::*;
use mtg_engine::object::*;
use mtg_engine::stickers::{self, StickerKind};
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// `p` puts a name sticker with `word` on `id`, choosing to put the word after
/// `position` words of its name.
fn name_sticker(t: &mut TestGame, p: PlayerId, id: ObjectId, word: &str, position: usize) {
    t.answer(p, DecisionKind::Option, Answer::Index(position));
    assert!(stickers::put_sticker(
        &mut t.g,
        p,
        id,
        StickerKind::Name(word.into())
    ));
    t.g.recompute();
}

fn name_of(t: &TestGame, id: ObjectId) -> String {
    t.obj_now(id).chars.name.to_string()
}

#[test]
fn the_controller_chooses_where_a_name_stickers_word_goes() {
    cr!("123.6b");
    cr_example!("123.6b", "\"Dark Bear Cub,\" \"Bear Dark Cub,\" or \"Bear Cub Dark.\"");
    let mut t = TestGame::new(2);
    let cub = t.battlefield(P0, "Bear Cub");
    let log = spy(&mut t, P0, |_g, _p, d| match d {
        mtg_engine::decision::Decision::ChooseOption {
            prompt, options, ..
        } if prompt.contains("name sticker's word") => Some(options.join(" | ")),
        _ => None,
    });
    name_sticker(&mut t, P0, cub, "Dark", 1);
    assert_eq!(
        probe_lines(&log),
        vec!["Dark Bear Cub | Bear Dark Cub | Bear Cub Dark".to_string()]
    );
    assert_eq!(name_of(&t, cub), "Bear Dark Cub");
}

#[test]
fn a_name_stickers_position_is_remembered_as_an_adventurer_is_cast_and_exiled() {
    cr!("123.6c", "123.5");
    cr_example!("123.6c", "Fae of Mana Wishes");
    let mut t = TestGame::new(2);
    let fae = t.exile(P0, "Fae of Wishes // Granted");
    name_sticker(&mut t, P0, fae, "Mana", 2);
    assert_eq!(name_of(&t, fae), "Fae of Mana Wishes");
    // An effect allows P0 to cast spells from exile.
    t.custom(
        P0,
        CB::new("Exile Caster")
            .enchantment()
            .ability(stat(StaticEffect::PlayPermission(PlayPermission {
                who: PlayerRel::You,
                zone: ZoneKind::Exile,
                top_only: false,
                what: Filter::Any,
                lands: false,
                spells: true,
                cost: None,
                flash: false,
                terms: Default::default(),
            })))
            .build(),
        Zone::Battlefield,
    );
    t.lands(P0, "Island", 4);
    let spell = t.cast(P0, fae).method(CastMethod::Half(1)).go();
    t.g.recompute();
    assert_eq!(name_of(&t, spell), "Granted Mana");
    t.resolve();
    t.g.recompute();
    let exiled = t.g.current(spell);
    assert_eq!(t.zone(exiled), Zone::Exile);
    assert_eq!(name_of(&t, exiled), "Fae of Mana Wishes");
}

#[test]
fn a_name_sticker_keeps_its_position_as_its_permanent_becomes_a_copy() {
    cr!("123.6c");
    cr_example!("123.6c", "Seeker of the Eldrazi Way");
    let mut t = TestGame::new(2);
    let it = t.battlefield(P0, "It That Betrays");
    name_sticker(&mut t, P0, it, "Eldrazi", 3);
    assert_eq!(name_of(&t, it), "It That Betrays Eldrazi");
    // Cytoshape: "Choose a nonlegendary creature on the battlefield. Target creature
    // becomes a copy of that creature until end of turn."
    let seeker = t.battlefield(P1, "Seeker of the Way");
    t.lands(P0, "Tropical Island", 3);
    t.answer_choose(P0, &[Entity::Object(seeker)]);
    let cyto = t.hand(P0, "Cytoshape");
    t.cast(P0, cyto).target(it).go();
    t.resolve();
    t.g.recompute();
    assert_eq!(name_of(&t, it), "Seeker of the Eldrazi Way");
}

#[test]
fn a_later_name_setting_effect_overrides_a_name_sticker() {
    cr!("123.6c");
    cr_example!("123.6c", "Legitimate Businessperson");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    name_sticker(&mut t, P0, bears, "Space", 1);
    assert_eq!(name_of(&t, bears), "Grizzly Space Bears");
    // Witness Protection: "Enchanted creature loses all abilities and is a green and white
    // Citizen creature with base power and toughness 1/1 named Legitimate
    // Businessperson."
    t.lands(P1, "Island", 1);
    t.g.turn.priority = Some(P1);
    let wp = t.hand(P1, "Witness Protection");
    t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
    t.cast(P1, wp).target(bears).go();
    t.resolve();
    t.g.recompute();
    assert_eq!(name_of(&t, bears), "Legitimate Businessperson");
    assert!(stickers::is_stickered(&t.g, bears));
}
