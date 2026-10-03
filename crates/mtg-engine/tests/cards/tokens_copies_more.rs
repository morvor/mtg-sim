//! More token grammar: quotes nested in a token's quoted ability, "the exiled card's
//! owner creates ...", "They create ...", "The token created this way ...", "attach it to
//! the token", a land token "that is every basic land type", token copies with half the
//! power and toughness, and tokens for each opponent dealt damage this turn.

use mtg_engine::object::ObjKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn assert_compiles(names: &[&str]) {
    for n in names {
        let u = card(n).unsupported_text().join(" | ");
        assert!(u.is_empty(), "{n} has unsupported text: {u}");
    }
}

#[test]
fn more_token_wordings_compile() {
    assert_compiles(&[
        "Reef Worm",
        "Nesting Dragon",
        "Preston Garvey, Minuteman",
        "Skyclave Apparition",
        "Severance Priest",
        "Vraska, the Cutting Glare",
        "Ajani's Chosen",
        "Determined Iteration",
        "Overlord of the Hauntwoods",
        "You've Been Caught Stealing",
        "Saw in Half",
    ]);
}

fn tokens(t: &TestGame, p: PlayerId, subtype: &str) -> Vec<ObjectId> {
    t.g.battlefield
        .iter()
        .copied()
        .filter(|o| {
            let o = t.g.obj(*o);
            o.kind == ObjKind::Token && o.controller == p && o.chars.has_subtype(subtype)
        })
        .collect()
}

#[test]
fn reef_worm_tokens_each_leave_a_bigger_token() {
    cr!("111.3", "603.6c");
    let mut t = TestGame::new(2);
    let worm = t.battlefield(P0, "Reef Worm");
    t.g.destroy(worm, None);
    t.resolve_all();
    let fish = tokens(&t, P0, "Fish");
    assert_eq!(fish.len(), 1, "{}", t.dump_log());
    assert_eq!(t.pt(fish[0]), (3, 3));
    t.g.destroy(fish[0], None);
    t.resolve_all();
    let whale = tokens(&t, P0, "Whale");
    assert_eq!(whale.len(), 1, "{}", t.dump_log());
    assert_eq!(t.pt(whale[0]), (6, 6));
    t.g.destroy(whale[0], None);
    t.resolve_all();
    let kraken = tokens(&t, P0, "Kraken");
    assert_eq!(kraken.len(), 1, "{}", t.dump_log());
    assert_eq!(t.pt(kraken[0]), (9, 9));
}

#[test]
fn skyclave_apparition_gives_the_exiled_cards_owner_an_illusion() {
    cr!("607.2a", "111.2", "608.2h");
    ruling!(
        "Skyclave Apparition",
        "If there's no exiled card when Skyclave Apparition leaves the battlefield"
    );
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[Entity::Object(giant)]);
    let sky = t.enter(P0, "Skyclave Apparition");
    t.resolve_all();
    assert!(t.in_exile("Hill Giant"));
    t.g.destroy(sky, None);
    t.resolve_all();
    let ill = tokens(&t, P1, "Illusion");
    assert_eq!(ill.len(), 1, "{}", t.dump_log());
    assert_eq!(t.pt(ill[0]), (4, 4));
    // Nothing exiled: no token.
    let mut t = TestGame::new(2);
    let sky = t.enter(P0, "Skyclave Apparition");
    t.resolve_all();
    t.g.destroy(sky, None);
    t.resolve_all();
    assert!(tokens(&t, P0, "Illusion").is_empty() && tokens(&t, P1, "Illusion").is_empty());
}

#[test]
fn vraska_the_cutting_glare_gives_the_destroyed_permanents_controller_a_treasure() {
    cr!("111.2", "111.10a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 6);
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[Entity::Object(giant)]);
    t.enter(P0, "Vraska, the Cutting Glare");
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Hill Giant"), "{}", t.dump_log());
    assert_eq!(tokens(&t, P1, "Treasure").len(), 1);
    assert_eq!(tokens(&t, P0, "Treasure").len(), 0);
}

#[test]
fn ajanis_chosen_may_move_the_aura_onto_the_cat() {
    cr!("303.4", "701.3a");
    ruling!(
        "Ajani's Chosen",
        "You may attach the Aura to the token only if it can legally enchant that token."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ajani's Chosen");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 1);
    let hs = t.hand(P0, "Holy Strength");
    t.cast(P0, hs).target(bears).go();
    t.answer_yes(P0, true);
    t.resolve_all();
    let cat = tokens(&t, P0, "Cat");
    assert_eq!(cat.len(), 1, "{}", t.dump_log());
    assert_eq!(t.pt(cat[0]), (3, 4));
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn determined_iteration_populated_token_gains_haste() {
    cr!("701.36a", "111.2");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Determined Iteration");
    t.lands(P0, "Plains", 2);
    let ra = t.hand(P0, "Raise the Alarm");
    t.cast(P0, ra).go();
    t.resolve_all();
    assert_eq!(tokens(&t, P0, "Soldier").len(), 2);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    let soldiers = tokens(&t, P0, "Soldier");
    assert_eq!(soldiers.len(), 3, "{}", t.dump_log());
    let hasty = soldiers
        .iter()
        .filter(|s| {
            t.g.obj(**s)
                .chars
                .has_keyword(mtg_engine::keywords::KeywordKind::Haste)
        })
        .count();
    assert_eq!(hasty, 1);
}

#[test]
fn overlord_of_the_hauntwoods_everywhere_is_every_basic_land_type() {
    cr!("305.6", "111.3");
    ruling!(
        "Overlord of the Hauntwoods",
        "has the land types Plains, Island, Swamp, Mountain, and Forest"
    );
    let mut t = TestGame::new(2);
    t.enter(P0, "Overlord of the Hauntwoods");
    t.resolve_all();
    let e = t.named_on_battlefield("Everywhere");
    assert_eq!(e.len(), 1, "{}", t.dump_log());
    let o = t.obj_now(e[0]);
    assert!(o.tapped);
    for s in ["Plains", "Island", "Swamp", "Mountain", "Forest"] {
        assert!(o.chars.has_subtype(s), "{s}");
    }
}

#[test]
fn youve_been_caught_stealing_counts_opponents_dealt_damage() {
    cr!("111.10a");
    let mut t = TestGame::new(3);
    let giant = t.battlefield(P0, "Hill Giant");
    t.g.deal_damage(giant, Entity::Player(P1), 1, false);
    t.lands(P0, "Mountain", 2);
    let y = t.hand(P0, "You've Been Caught Stealing");
    t.cast(P0, y).modes(&[1]).go();
    t.resolve_all();
    assert_eq!(tokens(&t, P0, "Treasure").len(), 1, "{}", t.dump_log());
}

#[test]
fn saw_in_half_copies_have_half_the_power_and_toughness_rounded_up() {
    cr!("707.9b", "608.2h", "107.1a");
    ruling!(
        "Saw in Half",
        "Use the power and toughness of the creature from when it was last on the battlefield"
    );
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.lands(P0, "Swamp", 3);
    let s = t.hand(P0, "Saw in Half");
    t.cast(P0, s).target(wurm).go();
    t.resolve_all();
    let copies: Vec<ObjectId> = t
        .named_on_battlefield("Craw Wurm")
        .into_iter()
        .filter(|o| t.obj_now(*o).kind == ObjKind::Token)
        .collect();
    assert_eq!(copies.len(), 2, "{}", t.dump_log());
    for c in copies {
        assert_eq!(t.obj_now(c).controller, P1);
        assert_eq!(t.pt(c), (3, 2));
    }
}
