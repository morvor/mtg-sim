//! CR 701.23h, 608.2f: a player who searches the same library once for each of several
//! objects ("for each land an opponent controls, its controller may search ...") finds
//! a different card each time: a card found for one object isn't found again for the next.

use crate::r608_each_player_common::*;
use mtg_engine::ability::*;
use mtg_engine::decision::Decision;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn a_player_searching_once_for_each_of_two_objects_finds_two_cards() {
    cr!("701.23h", "608.2f");
    // "For each land an opponent controls, its controller may search their library for a
    // basic land card and put it into their hand." P1 controls two lands.
    const EACH_LAND: Var = vars::USER + 4271;
    let ctl = || PlayerRef::ControllerOf(Box::new(Sel::Var(EACH_LAND)));
    let body = Effect::ForEach {
        sel: Sel::All(Filter::and(vec![
            Filter::Type(CardType::Land),
            Filter::InZone(ZoneKind::Battlefield),
            Filter::ControlledBy(PlayerRel::Opponent),
        ])),
        var: EACH_LAND,
        effect: Box::new(Effect::May {
            who: ctl(),
            effect: Box::new(Effect::Search {
                who: ctl(),
                whose: ctl(),
                filter: Filter::and(vec![
                    Filter::Supertype(Supertype::Basic),
                    Filter::Type(CardType::Land),
                    Filter::InZone(ZoneKind::Library),
                ]),
                count: Value::c(1),
                to: Destination::zone(ZoneKind::Hand),
                reveal: false,
                shuffle: false,
            }),
        }),
    };
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Ancient Tomb");
    t.battlefield(P1, "Ancient Tomb");
    t.library_top(P1, "Forest");
    let island = t.library_top(P1, "Island");
    t.answer_yes(P1, true);
    t.answer_yes(P1, true);
    let hand = t.hand_size(P1);
    cast_body(&mut t, body);
    let from = t.asked().len();
    t.resolve_all();
    assert_eq!(t.hand_size(P1), hand + 2);
    assert!(t.in_hand(P1, "Forest") && t.in_hand(P1, "Island"));
    // The second search doesn't offer the card the first one found.
    let searches: Vec<Vec<Entity>> = t.asked()[from..]
        .iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseEntities { candidates, .. } if *p == P1 => Some(candidates.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(searches.len(), 2, "{searches:?}");
    assert!(!searches[1].contains(&Entity::Object(island)));
}
