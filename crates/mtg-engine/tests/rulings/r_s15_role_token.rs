//! Rulings batch S15 — Role tokens (CR 111.10j–r, 303.7, 704.5z): Monstrous Rage, Embereth
//! Veteran, Return Triumphant, Gylwain, Casting Director, Ellivere of the Wild Court, with
//! Parallel Lives for Roles created at the same time.

use crate::r_s01_common::*;
use crate::r_s02_common::destroy;
use crate::r_s03_common::{in_hand_with_mana, run_effect};
use crate::r_s05_common::enter;
use crate::r_s15_common::*;
use mtg_engine::ability::*;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// Casts Monstrous Rage ("Target creature gets +2/+0 until end of turn. Create a Monster
/// Role token attached to it.") for `p` targeting `target`; returns the spell.
fn monstrous_rage(t: &mut TestGame, p: PlayerId, target: ObjectId) -> ObjectId {
    let rage = in_hand_with_mana(t, p, "Monstrous Rage");
    t.g.turn.priority = Some(p);
    t.cast(p, rage).target(target).go()
}

/// Activates Embereth Veteran ("{1}, Sacrifice this creature: Create a Young Hero Role
/// token attached to another target creature.") for `p` targeting `target`.
fn veteran(t: &mut TestGame, p: PlayerId, target: ObjectId) {
    let vet = t.battlefield(p, "Embereth Veteran");
    t.lands(p, "Wastes", 1);
    t.activate(p, vet, 0, &[Entity::Object(target)]).unwrap();
}

/// With Parallel Lives, one effect creates two Roles attached to the Bears at the same time;
/// `order` is how P0 orders their timestamps. Returns what the kept Role was before it
/// entered (its token object), after checking that only one of the two stays.
fn two_roles_at_once(activate: bool, order: Vec<usize>) -> ObjectId {
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Parallel Lives");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer(P0, DecisionKind::Order, Answer::Indices(order));
    let from = t.asked().len();
    if activate {
        veteran(&mut t, P0, bears);
    } else {
        monstrous_rage(&mut t, P0, bears);
    }
    t.resolve_all();
    // Both were created, as one event: P0 ordered them.
    assert_eq!(tokens_created_by(&t, P0), 2);
    assert_eq!(orders_asked_of(&t, P0, from), 1);
    let roles = roles_on(&t, bears);
    assert_eq!(roles.len(), 1, "only one of the two Roles stays");
    t.obj(roles[0]).prev.expect("a created token")
}

#[test]
fn roles_that_become_attached_at_the_same_time_are_kept_as_their_controller_chooses() {
    cr!("613.7m", "303.7a", "704.5z");
    ruling!(
        "Monstrous Rage",
        "If two or more Roles controlled by the same player become attached to a permanent at the same time (perhaps due to an effect such as that of Doubling Season), that player chooses which one to keep"
    );
    ruling!(
        "Embereth Veteran",
        "If two or more Roles controlled by the same player become attached to a permanent at the same time (perhaps due to an effect such as that of Doubling Season), that player chooses which one to keep"
    );
    supported("Monstrous Rage");
    supported("Embereth Veteran");
    supported("Parallel Lives");
    for activate in [false, true] {
        // The Role ordered last has the most recent timestamp and is kept: ordering them
        // the other way keeps the other one.
        let kept_first_order = two_roles_at_once(activate, vec![0, 1]);
        let kept_reversed = two_roles_at_once(activate, vec![1, 0]);
        assert_ne!(kept_first_order, kept_reversed);
        assert!(kept_reversed < kept_first_order);
    }
}

#[test]
fn a_role_creator_whose_only_target_is_illegal_doesnt_resolve() {
    cr!("608.2b");
    ruling!(
        "Monstrous Rage",
        "Some spells and abilities that create Role tokens require targets. If each target chosen is an illegal target as that spell or ability tries to resolve, it won't resolve. The Role token won't be created."
    );
    ruling!(
        "Embereth Veteran",
        "Some spells and abilities that create Role tokens require targets. If each target chosen is an illegal target as that spell or ability tries to resolve, it won’t resolve. The Role token won’t be created."
    );
    supported("Monstrous Rage");
    supported("Embereth Veteran");
    // Monstrous Rage: the Bears leave the battlefield before it resolves.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    monstrous_rage(&mut t, P0, bears);
    destroy(&mut t, bears);
    t.resolve_all();
    assert!(t.named_on_battlefield("Monster").is_empty());
    assert_eq!(tokens_created_by(&t, P0), 0);
    // Embereth Veteran's ability: its target gains protection from red (the Veteran is
    // red), so it's an illegal target.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    veteran(&mut t, P0, giant);
    run_effect(
        &mut t,
        None,
        P0,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::AddKeyword(
                mtg_engine::keywords::Keyword::with_filter(
                    KeywordKind::Protection,
                    Filter::Color(Color::Red),
                ),
            )],
            duration: Duration::EndOfTurn,
        },
        &[Entity::Object(giant)],
    );
    t.resolve_all();
    assert!(roles_on(&t, giant).is_empty());
    assert_eq!(tokens_created_by(&t, P0), 0);
    // With a legal target, the Role is created.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    veteran(&mut t, P0, giant);
    t.resolve_all();
    assert_eq!(role_names_on(&t, giant), vec!["Young Hero".to_string()]);
}

#[test]
fn there_are_seven_role_tokens() {
    cr!("111.10j", "111.10k", "111.10m", "111.10n", "111.10p", "111.10q", "111.10r");
    ruling!(
        "Monstrous Rage",
        "Cards in the Wilds of Eldraine main set create six different Role tokens: Cursed, Monster, Royal, Sorcerer, Wicked, and Young Hero. A seventh Role token, Virtuous, is created by Ellivere of the Wild Court"
    );
    ruling!(
        "Embereth Veteran",
        "Cards in the Wilds of Eldraine main set create six different Role tokens: Cursed, Monster, Royal, Sorcerer, Wicked, and Young Hero. A seventh Role token, Virtuous, is created by Ellivere of the Wild Court"
    );
    supported("Ellivere of the Wild Court");
    // Each Role, attached to a Grizzly Bears (2/2) of its own: a colorless Aura Role
    // enchantment token with enchant creature, and what it does to the creature.
    let expected: [(&str, (i32, i32)); 7] = [
        ("Cursed", (1, 1)),
        ("Monster", (3, 3)),
        ("Royal", (3, 3)),
        ("Sorcerer", (3, 3)),
        // +1/+1 for each enchantment you control: the Role itself.
        ("Virtuous", (3, 3)),
        ("Wicked", (3, 3)),
        ("Young Hero", (2, 2)),
    ];
    for (name, pt) in expected {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let spec = mtg_engine::tokens::predefined(name).expect("a predefined Role");
        run_effect(
            &mut t,
            None,
            P0,
            Effect::CreateTokenAttached {
                spec,
                count: Value::c(1),
                controller: PlayerRef::You,
                to: Sel::Target(0),
            },
            &[Entity::Object(bears)],
        );
        let roles = roles_on(&t, bears);
        assert_eq!(roles.len(), 1, "{name}");
        let o = t.obj(roles[0]);
        assert_eq!(o.chars.name.as_str(), name);
        assert!(o.is_token() && o.chars.colors == ColorSet::NONE, "{name}");
        assert!(o.chars.is(CardType::Enchantment), "{name}");
        assert!(
            o.chars.has_subtype("Aura") && o.chars.has_subtype("Role"),
            "{name}"
        );
        assert!(o.has_keyword(KeywordKind::Enchant), "{name}");
        assert_eq!(t.pt(bears), pt, "{name}");
    }
    // Monster: trample; Royal: ward {1}.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    monstrous_rage(&mut t, P0, bears);
    t.resolve_all();
    assert_eq!(role_names_on(&t, bears), vec!["Monster".to_string()]);
    assert!(t.obj(bears).has_keyword(KeywordKind::Trample));
    // Ellivere of the Wild Court creates the Virtuous Role: "Whenever Ellivere enters or
    // attacks, create a Virtuous Role token attached to another target creature you
    // control."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    enter(&mut t, P0, "Ellivere of the Wild Court");
    t.resolve_all();
    assert_eq!(role_names_on(&t, bears), vec!["Virtuous".to_string()]);
    assert_eq!(t.pt(bears), (3, 3));
}

#[test]
fn hexproof_and_shroud_dont_stop_a_role_from_being_attached_without_targeting() {
    cr!("702.18a", "111.10k", "111.10r");
    ruling!(
        "Gylwain, Casting Director",
        "Hexproof and shroud won't prevent a Role from becoming attached to a permanent if the ability creating that Role attached to that permanent doesn't target it."
    );
    ruling!(
        "Return Triumphant",
        "Hexproof and shroud won’t prevent a Role from becoming attached to a permanent if the ability creating that Role attached to that permanent doesn’t target it."
    );
    supported("Gylwain, Casting Director");
    supported("Return Triumphant");
    supported("Argothian Enchantress");
    // Gylwain: "Whenever Gylwain or another nontoken creature you control enters, choose
    // one — ... • Create a Monster Role token attached to that creature." Argothian
    // Enchantress has shroud.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Gylwain, Casting Director");
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![2]));
    let ench = enter(&mut t, P0, "Argothian Enchantress");
    assert!(t.obj(ench).has_keyword(KeywordKind::Shroud));
    t.resolve_all();
    assert_eq!(role_names_on(&t, ench), vec!["Monster".to_string()]);
    assert_eq!(t.pt(ench), (1, 2));
    // Return Triumphant: "Return target creature card with mana value 3 or less from your
    // graveyard to the battlefield. Create a Young Hero Role token attached to it."
    let mut t = TestGame::new(2);
    let card = t.graveyard(P0, "Argothian Enchantress");
    let rt = in_hand_with_mana(&mut t, P0, "Return Triumphant");
    t.cast(P0, rt).target(card).go();
    t.resolve_all();
    let ench = t.g.current(card);
    assert!(t.on_battlefield(ench));
    assert!(t.obj(ench).has_keyword(KeywordKind::Shroud));
    assert_eq!(role_names_on(&t, ench), vec!["Young Hero".to_string()]);
}

#[test]
fn a_role_that_cant_enchant_the_permanent_isnt_created() {
    cr!("303.4i", "702.16c");
    ruling!(
        "Return Triumphant",
        "In rare cases, a spell or ability might attempt to create a Role token enchanting a permanent that it can’t legally enchant (because of an ability like protection from enchantments). In such cases, the Role token isn’t created."
    );
    supported("Return Triumphant");
    supported("Azorius First-Wing");
    // Azorius First-Wing (mana value 2) has protection from enchantments: it returns, but
    // no Young Hero Role is created.
    let mut t = TestGame::new(2);
    let card = t.graveyard(P0, "Azorius First-Wing");
    let rt = in_hand_with_mana(&mut t, P0, "Return Triumphant");
    t.cast(P0, rt).target(card).go();
    t.resolve_all();
    let wing = t.g.current(card);
    assert!(t.on_battlefield(wing));
    assert!(roles_on(&t, wing).is_empty());
    assert!(t.named_on_battlefield("Young Hero").is_empty());
    assert_eq!(tokens_created_by(&t, P0), 0);
    assert_eq!(t.zone(card), Zone::Battlefield);
}

#[test]
fn only_the_newest_role_a_player_controls_on_a_permanent_stays() {
    cr!("303.7a", "704.5z");
    ruling!(
        "Embereth Veteran",
        "If a permanent has more than one Role attached to it controlled by the same player, each of those Roles except the one with the most recent timestamp is put into its owner’s graveyard. This is a state-based action."
    );
    supported("Embereth Veteran");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    // P1's Monster Role on the Giant.
    monstrous_rage(&mut t, P1, giant);
    t.resolve_all();
    let monster = roles_on(&t, giant);
    assert_eq!(monster.len(), 1);
    // Two Young Hero Roles from P0, one after the other: the first one is put into the
    // graveyard (a token, it then ceases to exist) as the second one is attached.
    veteran(&mut t, P0, giant);
    t.resolve_all();
    let first: Vec<ObjectId> = roles_on(&t, giant)
        .into_iter()
        .filter(|r| t.obj(*r).controller == P0)
        .collect();
    assert_eq!(first.len(), 1);
    veteran(&mut t, P0, giant);
    t.resolve_all();
    let now = roles_on(&t, giant);
    assert_eq!(now.len(), 2);
    assert!(!t.on_battlefield(first[0]));
    assert!(now.contains(&monster[0]), "the other player's Role stays");
    let mine: Vec<ObjectId> = now
        .into_iter()
        .filter(|r| t.obj(*r).controller == P0)
        .collect();
    assert_eq!(mine.len(), 1);
    assert_ne!(mine[0], first[0]);
    assert!(t.obj(mine[0]).timestamp > t.obj(first[0]).timestamp);
}
