//! Rulings batch P077 — "When this creature dies, exile it, then return ..." (CR 603.10a,
//! 608.2b: the rest happens even if it can't be exiled), Anafenza, the Foremost's and
//! Malicious Eclipse's exile replacements (CR 614.1a, 614.6).

use crate::r_s01_common::supported;
use crate::r_s02_common::destroy;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn moldgraf_monstrosity_returns_cards_even_if_it_cant_be_exiled() {
    cr!("608.2b", "603.10a");
    ruling!(
        "Moldgraf Monstrosity",
        "If Moldgraf Monstrosity's ability can't exile it (perhaps because it's not still in the graveyard when the ability resolves), the two creature cards are still returned to the battlefield."
    );
    supported("Moldgraf Monstrosity");
    let mut t = TestGame::new(2);
    let mold = t.battlefield(P0, "Moldgraf Monstrosity");
    t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P0, "Hill Giant");
    destroy(&mut t, mold);
    assert_eq!(t.stack_len(), 1);
    crate::r_s05_common::move_to(&mut t, mold, Zone::Hand(P0));
    t.resolve_all();
    assert!(t.in_hand(P0, "Moldgraf Monstrosity"));
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
}

#[test]
fn two_moldgrafs_the_first_may_return_the_second() {
    cr!("603.10a", "400.7", "608.2b");
    ruling!(
        "Moldgraf Monstrosity",
        "If two Moldgraf Monstrosities die simultaneously, the first ability to resolve could return the other Moldgraf Monstrosity to the battlefield. If it does, the second Moldgraf Monstrosity's ability won't exile it but it will return two more creature cards to the battlefield."
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, mtg_engine::turn::Step::PrecombatMain);
    t.battlefield(P0, "Moldgraf Monstrosity");
    t.battlefield(P0, "Moldgraf Monstrosity");
    crate::r_s25_common::cast_new(&mut t, P0, "Wrath of God", &[]);
    t.resolve();
    assert_eq!(t.stack_len(), 2);
    // With only the other Moldgraf as a creature card besides the resolving one, the first
    // ability exiles its own and returns the other one.
    t.resolve();
    assert_eq!(t.named_on_battlefield("Moldgraf Monstrosity").len(), 1);
    assert_eq!(t.g.exile.len(), 1);
    // The second: its Moldgraf isn't in the graveyard (it's back on the battlefield), and
    // two more creature cards return.
    t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P0, "Hill Giant");
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Moldgraf Monstrosity").len(), 1);
    assert_eq!(t.g.exile.len(), 1);
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
}

#[test]
fn mistmoon_griffin_controller_exiles_it_from_any_graveyard_and_returns_their_card() {
    cr!("603.3a", "608.2b", "603.10a");
    ruling!(
        "Mistmoon Griffin",
        "If you control Mistmoon Griffin when it goes to the graveyard, you exile the Griffin and return the top creature card from your graveyard to the battlefield. It doesn't matter whose graveyard the Griffin goes to."
    );
    ruling!(
        "Mistmoon Griffin",
        "You still put the top creature card from your graveyard onto the battlefield even if this card is not in your graveyard when the triggered ability resolves."
    );
    supported("Mistmoon Griffin");
    // P1's Griffin under P0's control dies: it goes to P1's graveyard.
    let mut t = TestGame::new(2);
    let griffin = t.battlefield(P1, "Mistmoon Griffin");
    crate::r_s06_common::give_control(&mut t, griffin, P0);
    t.graveyard(P0, "Hill Giant");
    t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P1, "Craw Wurm");
    destroy(&mut t, griffin);
    t.resolve_all();
    assert!(t.in_exile("Mistmoon Griffin"));
    // The top creature card of P0's graveyard (the last put there), under P0's control.
    let bears = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(bears.len(), 1);
    assert_eq!(t.obj(bears[0]).controller, P0);
    assert!(t.in_graveyard(P0, "Hill Giant"));
    assert!(t.in_graveyard(P1, "Craw Wurm"));
    // Not in the graveyard as the ability resolves: still returns the top creature card.
    let mut t = TestGame::new(2);
    let griffin = t.battlefield(P0, "Mistmoon Griffin");
    t.graveyard(P0, "Hill Giant");
    destroy(&mut t, griffin);
    crate::r_s05_common::move_to(&mut t, griffin, Zone::Hand(P0));
    t.resolve_all();
    assert!(t.in_hand(P0, "Mistmoon Griffin"));
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
}

#[test]
fn anafenza_targets_any_tapped_creature_you_control() {
    cr!("115.1", "508.1");
    ruling!(
        "Anafenza, the Foremost",
        "Anafenza's first ability can target any tapped creature you control. That creature doesn't necessarily have to be attacking."
    );
    supported("Anafenza, the Foremost");
    let mut t = TestGame::new(2);
    let anafenza = t.battlefield(P0, "Anafenza, the Foremost");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.objects[bears.0 as usize].tapped = true;
    t.answer_targets(P0, &[Entity::Object(bears)]);
    crate::r_s01_common::attack_with(&mut t, &[(anafenza, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(t.counters(bears, "+1/+1"), 1);
}

#[test]
fn anafenza_cares_what_the_card_is_where_it_leaves_from() {
    cr!("614.1a", "614.6", "702.103f");
    ruling!(
        "Anafenza, the Foremost",
        "Anafenza's last ability cares only what the card would be in the zone it's moving from, not what it would be in the graveyard. For example, if a land card you control becomes a creature due to an effect and then dies, the land card will be exiled. But if a creature card with bestow is an Aura when it would be put into the graveyard, it ends up in the graveyard."
    );
    // P1's Mutavault, animated, dies with P0's Anafenza out: exiled.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Anafenza, the Foremost");
    let vault = crate::r_p077_common::animate_mutavault(&mut t, P1);
    destroy(&mut t, vault);
    assert!(t.in_exile("Mutavault"));
    // P1's Boon Satyr cast with bestow is an Aura when it's destroyed: graveyard.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Anafenza, the Foremost");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P1, "Forest", 5);
    let satyr = t.hand(P1, "Boon Satyr");
    t.cast(P1, satyr)
        .method(CastMethod::Keyword(KeywordKind::Bestow))
        .target(Entity::Object(bears))
        .go();
    t.resolve_all();
    let aura = t.g.current(satyr);
    assert_eq!(t.zone(aura), Zone::Battlefield);
    assert!(!crate::r_s04_common::is_creature(&t, aura));
    destroy(&mut t, aura);
    assert!(t.in_graveyard(P1, "Boon Satyr"));
}

#[test]
fn malicious_eclipse_exiles_opponents_creatures_that_die_any_way() {
    cr!("614.1a", "611.2a");
    ruling!(
        "Malicious Eclipse",
        "Creatures opponents control will be exiled if they would die for any reason that turn, not just if they die by being reduced to 0 or less toughness by Malicious Eclipse."
    );
    supported("Malicious Eclipse");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let wurm = t.battlefield(P1, "Craw Wurm");
    let mine = t.battlefield(P0, "Grizzly Bears");
    crate::r_s25_common::cast_new(&mut t, P0, "Malicious Eclipse", &[]);
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert!(!t.on_battlefield(mine));
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(t.g.find_in_zone(Zone::Exile, "Grizzly Bears").len(), 1);
    // Later in the turn, destroyed: exiled too.
    destroy(&mut t, wurm);
    assert!(t.in_exile("Craw Wurm"));
    assert!(!t.in_graveyard(P1, "Craw Wurm"));
}

#[test]
fn welcome_to_gone_before_chapter_one_resolves_makes_no_walls() {
    cr!("611.2b", "714.2b");
    ruling!(
        "Welcome to . . . // Jurassic Park",
        "If Welcome to… leaves the battlefield before its first chapter ability resolves, that ability will have no effect. The target noncreature artifacts won't become Walls at all."
    );
    supported("Welcome to . . . // Jurassic Park");
    for leaves in [false, true] {
        let mut t = TestGame::new(2);
        let ring = t.battlefield(P1, "Sol Ring");
        let saga = t.battlefield(P0, "Welcome to . . . // Jurassic Park");
        t.answer_targets(P0, &[Entity::Object(ring)]);
        crate::r_s19_common::add_lore(&mut t, saga, 1);
        assert_eq!(t.stack_len(), 1);
        if leaves {
            destroy(&mut t, saga);
        }
        t.resolve_all();
        let is_wall = crate::r_s04_common::is_creature(&t, ring);
        assert_eq!(is_wall, !leaves);
        if is_wall {
            assert_eq!(t.pt(ring), (0, 4));
            assert!(t.obj_now(ring).chars.has_subtype("Wall"));
        }
    }
}
