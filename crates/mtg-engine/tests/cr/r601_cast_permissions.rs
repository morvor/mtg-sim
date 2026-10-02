//! CR 601.3, 601.2, 305.1: permissions to play cards from zones other than the hand, and
//! which one a player uses. An effect's permission "to cast" a card doesn't let it be
//! played as a land (CR 305.9); a permission may require an alternative cost (CR 118.9b),
//! which can't be combined with another (CR 118.9a); it may let spells be cast as though
//! they had flash (CR 702.8a); a player with several permissions chooses the one they're
//! using, and gets its terms (CR 601.2f, 614.1d).

use super::r600_common::*;
use mtg_engine::ability::*;
use mtg_engine::decision::Action;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::*;
use mtg_engine::casting::PERMISSION_COST;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

const PAY_LIFE: CastMethod = CastMethod::Alternative(PERMISSION_COST);

/// A sorcery: "You may play cards you own in exile this turn", the permission coming with
/// `terms`.
fn grant_sorcery(name: &str, terms: PlayTerms) -> CardDef {
    CB::new(name)
        .sorcery()
        .cost("{0}")
        .spell(Body::effect(Effect::WithPlayTerms {
            terms,
            effect: Box::new(Effect::GrantPlayPermission {
                who: PlayerRef::You,
                what: Sel::All(Filter::and(vec![
                    Filter::InZone(ZoneKind::Exile),
                    Filter::OwnedBy(PlayerRel::You),
                ])),
                duration: Duration::EndOfTurn,
                free: false,
            }),
        }))
        .build()
}

/// P0 casts and resolves a [`grant_sorcery`]; returns the spell (the permission's source).
fn grant(t: &mut TestGame, name: &str, terms: PlayTerms) -> ObjectId {
    let s = t.custom(P0, grant_sorcery(name, terms), Zone::Hand(P0));
    t.g.turn.priority = Some(P0);
    let spell = t.cast(P0, s).go();
    t.resolve();
    spell
}

fn cast_methods(t: &mut TestGame, p: PlayerId, card: ObjectId) -> Vec<CastMethod> {
    t.g.turn.priority = Some(p);
    t.g.recompute();
    t.g.legal_actions(p)
        .into_iter()
        .filter_map(|a| match a {
            Action::Cast { card: c, method } if c == card => Some(method),
            _ => None,
        })
        .collect()
}

fn can_play_land(t: &mut TestGame, p: PlayerId, card: ObjectId) -> bool {
    t.g.turn.priority = Some(p);
    t.g.recompute();
    t.g.legal_actions(p).contains(&Action::PlayLand { card })
}

#[test]
fn a_permission_to_cast_a_card_doesnt_let_it_be_played_as_a_land() {
    cr!("305.9", "601.3", "712.11b");
    let cast_only = PlayTerms {
        spells_only: true,
        ..Default::default()
    };
    // "You may cast": a land card can't be played; a modal double-faced card's spell face
    // may be cast, its land face not played.
    let mut t = TestGame::new(2);
    let mountain = t.exile(P0, "Mountain");
    let mammoth = t.exile(P0, "Kazandu Mammoth // Kazandu Valley");
    grant(&mut t, "Cast Permission", cast_only.clone());
    assert!(!can_play_land(&mut t, P0, mountain));
    assert!(t.play_land(P0, mountain).is_err());
    assert!(!can_play_land(&mut t, P0, mammoth));
    t.lands(P0, "Forest", 3);
    assert_eq!(cast_methods(&mut t, P0, mammoth), vec![CastMethod::Normal]);
    // "You may play": both.
    let mut t = TestGame::new(2);
    let mountain = t.exile(P0, "Mountain");
    let mammoth = t.exile(P0, "Kazandu Mammoth // Kazandu Valley");
    grant(&mut t, "Play Permission", PlayTerms::default());
    assert!(can_play_land(&mut t, P0, mountain));
    assert!(can_play_land(&mut t, P0, mammoth));
    t.play_land(P0, mammoth).expect("its land face");
    assert_eq!(t.named_on_battlefield("Kazandu Valley").len(), 1);
}

#[test]
fn a_permission_may_require_an_alternative_cost() {
    cr!("118.9a", "118.9b", "118.9d", "107.3b", "601.2f");
    let life = PlayTerms {
        alt_cost: Some(Cost::free().with(CostPart::PayLife(Value::ManaValueOf(Box::new(
            Sel::This,
        ))))),
        ..Default::default()
    };
    let mut t = TestGame::new(2);
    let bears = t.exile(P0, "Grizzly Bears");
    let md = t.exile(P0, "Mulldrifter");
    let blaze = t.exile(P0, "Blaze");
    grant(&mut t, "Life Permission", life);
    t.lands(P0, "Island", 5);
    t.lands(P0, "Forest", 2);
    // Only for life equal to its mana value: not its mana cost, nor evoke (CR 118.9a).
    assert_eq!(cast_methods(&mut t, P0, bears), vec![PAY_LIFE]);
    assert_eq!(cast_methods(&mut t, P0, md), vec![PAY_LIFE]);
    assert!(t.cast(P0, bears).try_go().is_err());
    assert!(t
        .cast(P0, md)
        .method(CastMethod::Keyword(KeywordKind::Evoke))
        .try_go()
        .is_err());
    t.cast(P0, bears).method(PAY_LIFE).go();
    assert_eq!(t.life(P0), 18);
    t.resolve();
    // Cost increases apply to it (CR 118.9d): Thalia makes a noncreature spell cost {1}
    // more. Blaze ({X}{R}) is cast with X = 0 (CR 107.3b): 1 life and {1}.
    t.battlefield(P1, "Thalia, Guardian of Thraben");
    let before = t.life(P0);
    t.cast(P0, blaze)
        .method(PAY_LIFE)
        .x(3)
        .target(Entity::Player(P1))
        .go();
    assert_eq!(t.life(P0), before - 1);
    t.resolve();
    assert_eq!(t.life(P1), 20, "X was 0");
}

#[test]
fn a_permission_may_come_with_an_additional_cost() {
    cr!("601.2f", "601.2h", "118.9a");
    // "... by paying 2 life in addition to paying its other costs": the mana cost and the
    // life; an alternative cost of the card's own may replace the mana cost.
    let extra = PlayTerms {
        extra_cost: Some(Cost::free().with(CostPart::PayLife(Value::c(2)))),
        ..Default::default()
    };
    let mut t = TestGame::new(2);
    let bears = t.exile(P0, "Grizzly Bears");
    let md = t.exile(P0, "Mulldrifter");
    grant(&mut t, "Costly Permission", extra);
    t.lands(P0, "Forest", 2);
    t.cast(P0, bears).go();
    assert_eq!(t.life(P0), 18);
    t.resolve();
    t.lands(P0, "Island", 3);
    assert!(cast_methods(&mut t, P0, md).contains(&CastMethod::Keyword(KeywordKind::Evoke)));
    t.cast(P0, md)
        .method(CastMethod::Keyword(KeywordKind::Evoke))
        .go();
    assert_eq!(t.life(P0), 16);
}

#[test]
fn a_permission_may_let_spells_be_cast_as_though_they_had_flash() {
    cr!("702.8a", "601.3", "307.1");
    let flash = PlayTerms {
        flash: true,
        ..Default::default()
    };
    // Divination, a sorcery, during combat: only with the permission that gives flash.
    let mut t = TestGame::new(2);
    let div = t.exile(P0, "Divination");
    grant(&mut t, "Plain Permission", PlayTerms::default());
    t.lands(P0, "Island", 3);
    t.advance_to_step(Step::BeginningOfCombat);
    assert!(cast_methods(&mut t, P0, div).is_empty());
    let mut t = TestGame::new(2);
    let div = t.exile(P0, "Divination");
    grant(&mut t, "Flash Permission", flash);
    t.lands(P0, "Island", 3);
    t.advance_to_step(Step::BeginningOfCombat);
    assert_eq!(cast_methods(&mut t, P0, div), vec![CastMethod::Normal]);
    t.cast(P0, div).go();
    t.resolve();
    assert!(t.in_graveyard(P0, "Divination"));
}

#[test]
fn the_player_chooses_which_permission_they_use_and_gets_its_terms() {
    cr!("601.2", "601.3", "601.2f");
    let costlier = PlayTerms {
        cost_increase: 2,
        ..Default::default()
    };
    for choose_costlier in [false, true] {
        let mut t = TestGame::new(2);
        let bears = t.exile(P0, "Grizzly Bears");
        let costly = grant(&mut t, "Costly Permission", costlier.clone());
        grant(&mut t, "Plain Permission", PlayTerms::default());
        t.lands(P0, "Forest", 4);
        // By default, the one without the increase.
        if choose_costlier {
            t.answer_choose(P0, &[Entity::Object(costly)]);
        }
        t.cast(P0, bears).go();
        let tapped = t
            .g
            .battlefield
            .iter()
            .filter(|l| t.g.obj(**l).tapped)
            .count();
        assert_eq!(tapped, if choose_costlier { 4 } else { 2 });
    }
}

#[test]
fn a_land_is_played_with_the_chosen_permission_and_its_terms() {
    cr!("305.1", "601.3", "614.1d");
    let tapped = PlayTerms {
        lands_enter_tapped: true,
        ..Default::default()
    };
    for choose_tapped in [false, true] {
        let mut t = TestGame::new(2);
        let forest = t.exile(P0, "Forest");
        let a = grant(&mut t, "Tapped Permission", tapped.clone());
        grant(&mut t, "Plain Permission", PlayTerms::default());
        if choose_tapped {
            t.answer_choose(P0, &[Entity::Object(a)]);
        }
        t.play_land(P0, forest).expect("play the Forest");
        let land = t.named_on_battlefield("Forest")[0];
        assert_eq!(t.obj(land).tapped, choose_tapped);
    }
}

#[test]
fn a_once_each_turn_permission_is_used_only_by_the_card_played_with_it() {
    cr!("601.3", "305.1");
    // Muldrotha's land permission and Crucible of Worlds's ("You may play lands from your
    // graveyard"): playing a land with Crucible's leaves Muldrotha's land permission.
    let mut t = TestGame::new(2);
    let muldrotha = t.battlefield(P0, "Muldrotha, the Gravetide");
    let crucible = t.battlefield(P0, "Crucible of Worlds");
    let forest = t.graveyard(P0, "Forest");
    t.answer_choose(P0, &[Entity::Object(crucible)]);
    t.play_land(P0, forest).expect("play the Forest");
    assert!(t
        .g
        .history
        .once_permissions_used
        .iter()
        .all(|(o, _)| *o != muldrotha));
    // With Muldrotha's, it's used.
    let mut t = TestGame::new(2);
    let muldrotha = t.battlefield(P0, "Muldrotha, the Gravetide");
    t.battlefield(P0, "Crucible of Worlds");
    let forest = t.graveyard(P0, "Forest");
    t.answer_choose(P0, &[Entity::Object(muldrotha)]);
    t.play_land(P0, forest).expect("play the Forest");
    assert!(t
        .g
        .history
        .once_permissions_used
        .iter()
        .any(|(o, s)| *o == muldrotha && s == "land"));
}
