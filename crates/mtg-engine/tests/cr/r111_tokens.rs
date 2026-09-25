//! CR 111: tokens — creation, owner and controller, characteristics and names, tokens
//! leaving the battlefield, tokens by card name, copies.

use super::r105_util::*;
use mtg_engine::ability::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Casts a free sorcery with the given oracle text for `p` and returns the tokens it
/// created.
pub fn run_text(t: &mut TestGame, p: PlayerId, text: &str) -> Vec<ObjectId> {
    let before = tokens_of(t, p);
    let c = put_in_hand(
        t,
        p,
        card_from_text("Token Maker", "{0}", "Sorcery", None, text),
    );
    t.set_step(p, Step::PrecombatMain);
    t.cast(p, c).go();
    t.resolve();
    tokens_of(t, p)
        .into_iter()
        .filter(|x| !before.contains(x))
        .collect()
}

#[test]
fn tokens_represent_permanents_not_represented_by_cards() {
    cr!("111.1", "111.6");
    let mut t = TestGame::new(2);
    let alarm = t.hand(P0, "Raise the Alarm");
    t.lands(P0, "Plains", 2);
    t.cast(P0, alarm).go();
    t.resolve();
    let soldiers = tokens_of(&t, P0);
    assert_eq!(soldiers.len(), 2);
    for s in &soldiers {
        assert!(t.obj(*s).is_token());
        assert!(t.on_battlefield(*s));
        // A token isn't a card.
        assert!(!matches(&t, *s, &Filter::Card, P0));
        assert!(matches(&t, *s, &Filter::Permanent, P0));
    }
    // It's subject to anything that affects permanents or its card type: an anthem
    // pumps it and a creature removal spell can target it.
    t.battlefield(P0, "Glorious Anthem");
    t.g.recompute();
    assert_eq!(t.pt(soldiers[0]), (2, 2));
    let murder = t.hand(P1, "Murder");
    assert!(spell_target_candidates(&t, P1, murder, 0).contains(&Entity::Object(soldiers[0])));
}

#[test]
fn the_player_who_creates_a_token_owns_and_controls_it() {
    cr!("111.2");
    ruling!(
        "Gild",
        "You will control the token, no matter who controls the target creature."
    );
    let mut t = TestGame::new(2);
    // Gild: the caster creates the Gold token.
    let bears = t.battlefield(P1, "Grizzly Bears");
    let gild = t.hand(P0, "Gild");
    t.lands(P0, "Swamp", 4);
    t.cast(P0, gild).target(bears).go();
    t.resolve();
    let gold = tokens_of(&t, P0);
    assert_eq!(gold.len(), 1);
    assert_eq!(t.obj(gold[0]).owner, P0);
    assert!(tokens_of(&t, P1).is_empty());
    // Beast Within: the destroyed permanent's controller creates the Beast.
    let giant = t.battlefield(P1, "Hill Giant");
    let within = t.hand(P0, "Beast Within");
    t.lands(P0, "Forest", 3);
    t.cast(P0, within).target(giant).go();
    t.resolve();
    let beasts = tokens_of(&t, P1);
    assert_eq!(beasts.len(), 1);
    assert_eq!(t.obj(beasts[0]).owner, P1);
    assert_eq!(t.obj(beasts[0]).controller, P1);
    assert_eq!(t.pt(beasts[0]), (3, 3));
}

#[test]
fn a_token_has_only_the_characteristics_its_creator_defines() {
    cr!("111.3");
    ruling!(
        "Spitting Image",
        "if that creature is itself a token, the original characteristics of that token"
    );
    let mut t = TestGame::new(2);
    let mage = t.battlefield(P0, "Jade Mage");
    t.lands(P0, "Forest", 3);
    t.activate(P0, mage, 0, &[]).unwrap();
    t.resolve();
    let sap = tokens_of(&t, P0)[0];
    let c = &t.obj(sap).chars;
    assert!(c.mana_cost.is_none());
    assert_eq!(c.supertypes, SupertypeSet::NONE);
    assert!(c.abilities.is_empty());
    assert_eq!(c.colors, cs("G"));
    assert!(c.card_types.contains(CardType::Creature));
    assert!(c.has_subtype("Saproling"));
    assert_eq!(t.pt(sap), (1, 1));
    assert_eq!(t.obj(sap).chars.mana_value(), 0);
    // Those values are its copiable values: pumping it doesn't change what a copy gets.
    let growth = t.hand(P0, "Giant Growth");
    t.lands(P0, "Forest", 1);
    t.cast(P0, growth).target(sap).go();
    t.resolve();
    assert_eq!(t.pt(sap), (4, 4));
    let image = t.hand(P0, "Spitting Image");
    t.lands(P0, "Island", 6);
    t.cast(P0, image).target(sap).go();
    t.resolve();
    let copies: Vec<ObjectId> = tokens_of(&t, P0)
        .into_iter()
        .filter(|x| *x != sap)
        .collect();
    assert_eq!(copies.len(), 1);
    let cc = &t.obj(copies[0]).chars;
    assert_eq!(t.pt(copies[0]), (1, 1));
    assert!(cc.has_subtype("Saproling"));
    assert_eq!(cc.colors, cs("G"));
    assert!(cc.mana_cost.is_none());
}

#[test]
fn an_unnamed_tokens_name_is_its_subtypes_plus_token() {
    cr!("111.4");
    let mut t = TestGame::new(2);
    // "Create two 1/1 white Soldier creature tokens": each is named Soldier Token.
    let soldiers = run_text(&mut t, P0, "Create two 1/1 white Soldier creature tokens.");
    assert_eq!(soldiers.len(), 2);
    for s in &soldiers {
        assert_eq!(t.obj(*s).chars.name.as_str(), "Soldier Token");
    }
    // Several subtypes: "Dwarf Berserker Token", with the types Dwarf and Berserker.
    let dwarves = run_text(
        &mut t,
        P0,
        "Create two 2/1 red Dwarf Berserker creature tokens.",
    );
    let d = &t.obj(dwarves[0]).chars;
    assert_eq!(d.name.as_str(), "Dwarf Berserker Token");
    assert!(d.has_subtype("Dwarf") && d.has_subtype("Berserker"));
    // A token that's a copy of a creature has that creature's name.
    let dissenter = t.battlefield(P1, "Doomed Dissenter");
    let image = t.hand(P0, "Spitting Image");
    t.lands(P0, "Island", 6);
    t.cast(P0, image).target(dissenter).go();
    t.resolve();
    let copy = *tokens_of(&t, P0)
        .iter()
        .find(|x| t.obj(**x).chars.has_subtype("Human"))
        .unwrap();
    assert_eq!(t.obj(copy).chars.name.as_str(), "Doomed Dissenter");
}

#[test]
fn changing_a_tokens_name_doesnt_change_its_subtypes_and_vice_versa() {
    cr!("111.4");
    let mut t = TestGame::new(2);
    let soldier = run_text(&mut t, P0, "Create a 1/1 white Soldier creature token.")[0];
    let rename = card_with(
        "Rename",
        "{0}",
        "Instant",
        None,
        vec![spell_ab(
            vec![TargetSpec::object(Filter::creature(), "target creature")],
            Effect::Modify {
                what: Sel::Target(0),
                mods: vec![Modification::SetName("Bob".into())],
                duration: Duration::EndOfTurn,
            },
        )],
    );
    let r = put_in_hand(&mut t, P0, rename);
    t.cast(P0, r).target(soldier).go();
    t.resolve();
    let c = &t.obj(soldier).chars;
    assert_eq!(c.name.as_str(), "Bob");
    assert!(c.has_subtype("Soldier"));
    // Changing its creature type doesn't rename it either.
    let retype = card_with(
        "Retype",
        "{0}",
        "Instant",
        None,
        vec![spell_ab(
            vec![TargetSpec::object(Filter::creature(), "target creature")],
            Effect::Modify {
                what: Sel::Target(0),
                mods: vec![Modification::RemoveAllCreatureTypes],
                duration: Duration::EndOfTurn,
            },
        )],
    );
    let soldier2 = run_text(&mut t, P0, "Create a 1/1 white Soldier creature token.")[0];
    let r = put_in_hand(&mut t, P0, retype);
    t.cast(P0, r).target(soldier2).go();
    t.resolve();
    let c = &t.obj(soldier2).chars;
    assert!(!c.has_subtype("Soldier"));
    assert_eq!(c.name.as_str(), "Soldier Token");
}

#[test]
fn a_named_legendary_token_has_the_given_name() {
    cr!("111.4", "111.9");
    let mut t = TestGame::new(2);
    let minsc = t.hand(P0, "Minsc, Beloved Ranger");
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Plains", 1);
    t.cast(P0, minsc).go();
    t.resolve();
    t.resolve();
    let boo = tokens_of(&t, P0);
    assert_eq!(boo.len(), 1);
    let c = &t.obj(boo[0]).chars;
    // Neither "Hamster" nor "Token" is part of its name.
    assert_eq!(c.name.as_str(), "Boo");
    assert!(c.has_supertype(Supertype::Legendary));
    assert!(c.has_subtype("Hamster"));
    assert_eq!(c.colors, cs("R"));
    assert_eq!(t.pt(boo[0]), (1, 1));
    assert!(t.obj(boo[0]).has_keyword(mtg_engine::keywords::KeywordKind::Trample));
    assert!(t.obj(boo[0]).has_keyword(mtg_engine::keywords::KeywordKind::Haste));
}

#[test]
fn a_token_that_cant_enter_the_battlefield_isnt_created() {
    cr!("111.5");
    let mut t = TestGame::new(2);
    // Worms of the Earth: "Lands can't enter the battlefield." Autumn Willow's Forest Dryad
    // land creature token isn't created.
    t.battlefield(P1, "Worms of the Earth");
    let watcher = card_with(
        "Token Watcher",
        "{0}",
        "Enchantment",
        None,
        vec![triggered_ab(
            TriggerCond::TokenCreated(Filter::Any),
            gain_life(5),
        )],
    );
    put(&mut t, P0, watcher);
    let willow = t.hand(P0, "Autumn Willow, Harmony");
    t.lands(P0, "Forest", 5);
    t.cast(P0, willow).go();
    t.resolve();
    t.resolve_all();
    assert!(!t.named_on_battlefield("Autumn Willow, Harmony").is_empty());
    assert!(tokens_of(&t, P0).is_empty());
    assert_eq!(t.life(P0), 20);
}

#[test]
fn no_token_copy_of_an_instant_or_sorcery_card_is_created() {
    cr!("111.5");
    ruling!(
        "Mimic Vat",
        "You can't create a token that's a copy of a nonpermanent card. No token is created in this case."
    );
    let mut t = TestGame::new(2);
    // "Create a token that's a copy of target card in your graveyard."
    let copier = || {
        card_with(
            "Grave Copier",
            "{0}",
            "Sorcery",
            None,
            vec![spell_ab(
                vec![TargetSpec::object(
                    Filter::and(vec![
                        Filter::Card,
                        Filter::InZone(ZoneKind::Graveyard),
                        Filter::OwnedBy(PlayerRel::You),
                    ]),
                    "target card in your graveyard",
                )],
                Effect::CreateTokenCopy {
                    of: Sel::Target(0),
                    count: Value::c(1),
                    controller: PlayerRef::You,
                    tapped: false,
                    attacking: false,
                    mods: vec![],
                },
            )],
        )
    };
    let bolt = t.graveyard(P0, "Lightning Bolt");
    let bears = t.graveyard(P0, "Grizzly Bears");
    let c = put_in_hand(&mut t, P0, copier());
    t.cast(P0, c).target(bolt).go();
    t.resolve();
    assert!(tokens_of(&t, P0).is_empty());
    let c = put_in_hand(&mut t, P0, copier());
    t.cast(P0, c).target(bears).go();
    t.resolve();
    let toks = tokens_of(&t, P0);
    assert_eq!(toks.len(), 1);
    assert_eq!(t.obj(toks[0]).chars.name.as_str(), "Grizzly Bears");
}

#[test]
fn a_token_leaving_the_battlefield_triggers_then_ceases_to_exist() {
    cr!("111.7");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Blood Artist");
    let soldier = run_text(&mut t, P0, "Create a 1/1 white Soldier creature token.")[0];
    t.g.players[1].life = 20;
    // The token dies: Blood Artist's ability triggers before the token ceases to exist.
    let murder = t.hand(P0, "Murder");
    t.lands(P0, "Swamp", 3);
    t.cast(P0, murder).target(soldier).go();
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve();
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    assert_eq!(t.life(P0), 21);
    // The token is no longer in the graveyard (or anywhere).
    assert!(t.g.players[0]
        .graveyard
        .iter()
        .all(|x| !t.obj(*x).is_token()));
    assert!(!t.g.is_live(t.g.current(soldier)));
    // A token returned to its owner's hand ceases to exist too.
    let soldier = run_text(&mut t, P0, "Create a 1/1 white Soldier creature token.")[0];
    let hand_before = t.hand_size(P0);
    let unsummon = t.hand(P0, "Unsummon");
    t.lands(P0, "Island", 1);
    t.cast(P0, unsummon).target(soldier).go();
    t.resolve();
    t.settle();
    assert_eq!(t.hand_size(P0), hand_before);
    assert!(!t.g.is_live(t.g.current(soldier)));
}

#[test]
fn a_token_that_left_the_battlefield_cant_come_back() {
    cr!("111.8");
    ruling!(
        "Kitchen Finks",
        "If a token with no -1/-1 counters on it has persist, the ability will trigger when the token is put into the graveyard. However, the token will cease to exist and can't return to the battlefield."
    );
    let mut t = TestGame::new(2);
    // A token copy of Kitchen Finks has persist.
    let finks = t.battlefield(P1, "Kitchen Finks");
    let image = t.hand(P0, "Spitting Image");
    t.lands(P0, "Island", 6);
    t.cast(P0, image).target(finks).go();
    t.resolve();
    t.resolve_all();
    let token = tokens_of(&t, P0)[0];
    assert_eq!(t.obj(token).chars.name.as_str(), "Kitchen Finks");
    let life = t.life(P0);
    let murder = t.hand(P0, "Murder");
    t.lands(P0, "Swamp", 3);
    t.cast(P0, murder).target(token).go();
    t.resolve();
    // Persist triggered and resolved, but the token stayed where it was and then ceased
    // to exist.
    t.resolve_all();
    assert!(tokens_of(&t, P0).is_empty());
    assert_eq!(t.named_on_battlefield("Kitchen Finks").len(), 1);
    // No new Finks entered, so no life was gained.
    assert_eq!(t.life(P0), life);
}

#[test]
fn a_token_in_a_graveyard_cant_be_moved_to_another_zone() {
    cr!("111.8");
    let mut t = TestGame::new(2);
    let soldier = run_text(&mut t, P0, "Create a 1/1 white Soldier creature token.")[0];
    // Move it to the graveyard without checking state-based actions.
    let in_gy = t
        .g
        .move_object(
            soldier,
            Zone::Graveyard(P0),
            mtg_engine::events::MoveCause::Effect,
            None,
        )
        .unwrap();
    assert_eq!(t.obj(in_gy).zone, Zone::Graveyard(P0));
    // It can't return to the battlefield or go to another zone.
    let r = t.g.move_object(
        in_gy,
        Zone::Battlefield,
        mtg_engine::events::MoveCause::Effect,
        Some(P0),
    );
    assert!(r.is_none());
    let r = t
        .g
        .move_object(in_gy, Zone::Hand(P0), mtg_engine::events::MoveCause::Effect, None);
    assert!(r.is_none());
    assert_eq!(t.obj(in_gy).zone, Zone::Graveyard(P0));
    // It ceases to exist the next time state-based actions are checked.
    t.settle();
    assert!(!t.g.is_live(in_gy));
}

#[test]
fn a_token_by_card_name_uses_the_oracle_card() {
    cr!("111.11");
    ruling!(
        "Disa the Restless",
        "creates a token that's a copy of the card Tarmogoyf in the Oracle card reference"
    );
    let mut t = TestGame::new(2);
    let disa = t.battlefield(P0, "Disa the Restless");
    t.set_step(P0, Step::PrecombatMain);
    t.attack(&[(disa, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 15);
    let goyf = tokens_of(&t, P0);
    assert_eq!(goyf.len(), 1);
    let c = &t.obj(goyf[0]).chars;
    assert_eq!(c.name.as_str(), "Tarmogoyf");
    assert!(c.has_subtype("Lhurgoyf"));
    assert_eq!(c.colors, cs("G"));
    assert_eq!(c.mana_cost.as_ref().unwrap().to_string(), "{1}{G}");
    // Its power and toughness are defined by Tarmogoyf's ability: no card types among
    // cards in graveyards yet.
    assert_eq!(t.pt(goyf[0]), (0, 1));
}

#[test]
fn a_token_copy_of_an_object_that_left_uses_last_known_information() {
    cr!("111.12");
    ruling!(
        "Myr Propagator",
        "If Myr Propagator has left the battlefield by the time its ability resolves, you’ll still put a token onto the battlefield."
    );
    let mut t = TestGame::new(2);
    let myr = t.battlefield(P0, "Myr Propagator");
    t.lands(P0, "Island", 3);
    t.activate(P0, myr, 0, &[]).unwrap();
    // In response, the Propagator is destroyed.
    let murder = t.hand(P1, "Murder");
    t.lands(P1, "Swamp", 3);
    t.g.turn.priority = Some(P1);
    t.cast(P1, murder).target(myr).go();
    t.resolve();
    assert!(t.in_graveyard(P0, "Myr Propagator"));
    t.resolve();
    let toks = tokens_of(&t, P0);
    assert_eq!(toks.len(), 1);
    assert_eq!(t.obj(toks[0]).chars.name.as_str(), "Myr Propagator");
}

#[test]
fn no_token_copy_of_a_nonexistent_object_is_created() {
    cr!("111.12");
    let mut t = TestGame::new(2);
    // "Create a token that's a copy of a card exiled with this artifact." with nothing
    // exiled creates nothing.
    let vat = card_with(
        "Empty Vat",
        "{0}",
        "Artifact",
        None,
        vec![activated_ab(
            Cost::tap(),
            Effect::CreateTokenCopy {
                of: Sel::Linked,
                count: Value::c(1),
                controller: PlayerRef::You,
                tapped: false,
                attacking: false,
                mods: vec![],
            },
        )],
    );
    let v = put(&mut t, P0, vat);
    t.activate(P0, v, 0, &[]).unwrap();
    t.resolve();
    assert!(tokens_of(&t, P0).is_empty());
}

#[test]
fn a_copy_of_a_permanent_spell_becomes_a_token_that_isnt_created() {
    cr!("111.13");
    let mut t = TestGame::new(2);
    let watcher = card_with(
        "Token Watcher",
        "{0}",
        "Enchantment",
        None,
        vec![triggered_ab(
            TriggerCond::TokenCreated(Filter::Any),
            gain_life(5),
        )],
    );
    put(&mut t, P0, watcher);
    let bears = t.hand(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 2);
    let spell = t.cast(P0, bears).go();
    let copier = card_with(
        "Copy Test",
        "{0}",
        "Instant",
        None,
        vec![spell_ab(
            vec![TargetSpec {
                what: TargetKind::Spell(Filter::Any),
                ..TargetSpec::object(Filter::Any, "target spell")
            }],
            Effect::CopySpell {
                what: Sel::Target(0),
                count: Value::c(1),
                new_targets: false,
            },
        )],
    );
    let c = put_in_hand(&mut t, P0, copier);
    t.cast(P0, c).target(spell).go();
    t.resolve_all();
    let all = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(all.len(), 2);
    let tok: Vec<ObjectId> = all
        .iter()
        .copied()
        .filter(|b| t.obj(*b).is_token())
        .collect();
    assert_eq!(tok.len(), 1);
    // It has the characteristics of the spell that became it.
    assert_eq!(t.pt(tok[0]), (2, 2));
    assert_eq!(
        t.obj(tok[0]).chars.mana_cost.as_ref().unwrap().to_string(),
        "{1}{G}"
    );
    // It wasn't "created": the token-creation trigger didn't fire.
    assert_eq!(t.life(P0), 20);
}
