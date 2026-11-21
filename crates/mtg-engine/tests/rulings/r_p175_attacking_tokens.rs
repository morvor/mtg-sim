//! Rulings batch P175 — tokens created tapped and attacking (Kari Zev, Tilonalli's
//! Summoner, the Trebuchets, Dalkovan Encampment, Stangg), myriad from Cybermen Squadron,
//! tokens leaving together with Zurgo Stormrender, and Stadium Headliner's count on
//! resolution.

use crate::r_p175_common::*;
use crate::r_s06_common::{attach_new, attached_to, has_kw};
use mtg_engine::decision::Answer;
use mtg_engine::events::Event;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// What the attacking creature `id` is attacking.
fn attack_target(t: &TestGame, id: ObjectId) -> Option<Entity> {
    let id = t.g.current(id);
    t.g.combat
        .as_ref()?
        .attackers
        .iter()
        .find(|a| a.id == id)
        .and_then(|a| a.target)
}

/// The attackers declared this turn.
fn declared(t: &TestGame) -> Vec<ObjectId> {
    t.g.turn_events
        .iter()
        .flat_map(|e| match e {
            Event::AttackersDeclared { attackers, .. } => {
                attackers.iter().map(|a| a.0).collect::<Vec<_>>()
            }
            _ => vec![],
        })
        .collect()
}

/// `p`'s tokens on the battlefield named `name` or (unnamed tokens) with that subtype.
fn named_tokens(t: &TestGame, p: PlayerId, name: &str) -> Vec<ObjectId> {
    tokens(t, p)
        .into_iter()
        .filter(|id| {
            let c = &t.g.obj(*id).chars;
            c.name.as_str() == name || c.has_subtype(name)
        })
        .collect()
}

#[test]
fn kari_zev_ragavan_wasnt_declared_and_is_exiled_at_end_of_combat() {
    cr!("508.3a", "508.4", "603.7");
    ruling!(
        "Kari Zev, Skyship Raider",
        "Although Ragavan is attacking, it was never declared as an attacking creature (for the purposes of abilities that trigger whenever a creature attacks, for example)."
    );
    ruling!(
        "Kari Zev, Skyship Raider",
        "The delayed triggered ability that exiles Ragavan triggers at end of combat even if Kari Zev is no longer on the battlefield."
    );
    supported("Kari Zev, Skyship Raider");
    let mut t = TestGame::new(2);
    let kari = t.battlefield(P0, "Kari Zev, Skyship Raider");
    // Hellrider: "Whenever a creature you control attacks, Hellrider deals 1 damage to the
    // player or planeswalker it's attacking."
    let hellrider = t.battlefield(P0, "Hellrider");
    attack_with(&mut t, &[(kari, pl(P1)), (hellrider, pl(P1))]);
    t.resolve_all();
    let ragavan = named_tokens(&t, P0, "Ragavan");
    assert_eq!(ragavan.len(), 1);
    assert!(attacking(&t, ragavan[0]));
    assert!(!declared(&t).contains(&ragavan[0]));
    assert_eq!(
        t.life(P1),
        18,
        "Hellrider triggered for Kari Zev and itself only"
    );
    // Kari Zev leaves; Ragavan is still exiled at end of combat.
    kill(&mut t, kari);
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    assert!(named_tokens(&t, P0, "Ragavan").is_empty());
}

#[test]
fn tokens_entering_attacking_can_attack_a_different_player_or_planeswalker() {
    cr!("508.4");
    ruling!(
        "Kari Zev, Skyship Raider",
        "You choose which opponent or opposing planeswalker Ragavan is attacking as you create the token. It doesn't have to be the same player or planeswalker Kari Zev is attacking."
    );
    ruling!(
        "Tilonalli's Summoner",
        "As each Elemental token enters the battlefield, you choose which opponent or opposing planeswalker it’s attacking. It doesn’t have to attack the same player or planeswalker that Tilonalli’s Summoner is attacking."
    );
    ruling!(
        "Fire Navy Trebuchet",
        "Fire Navy Trebuchet's controller chooses which player, planeswalker, or battle the token is attacking. It doesn't need to be the same player, planeswalker, or battle as other creatures are attacking."
    );
    ruling!(
        "Mordor Trebuchet",
        "Mordor Trebuchet's controller chooses which player, planeswalker, or battle the token is attacking. It doesn't need to be the same player, planeswalker, or battle that any of the Goblins and/or Orcs are attacking."
    );
    // (attacker, the creature that attacks with it, the token's name)
    let cases: [(&str, Option<&str>, &str); 4] = [
        ("Kari Zev, Skyship Raider", None, "Ragavan"),
        ("Tilonalli's Summoner", None, "Elemental"),
        (
            "Fire Navy Trebuchet",
            Some("Grizzly Bears"),
            "Ballistic Boulder",
        ),
        (
            "Mordor Trebuchet",
            Some("Goblin Piker"),
            "Ballistic Boulder",
        ),
    ];
    for (card, with, token) in cases {
        supported(card);
        let mut t = TestGame::new(2);
        let pw = t.battlefield(P1, "Ajani Goldmane");
        let c = t.battlefield(P0, card);
        let attacker = match with {
            Some(w) => t.battlefield(P0, w),
            None => c,
        };
        if card == "Tilonalli's Summoner" {
            t.answer_yes(P0, true);
            t.answer(P0, DecisionKind::X, Answer::Number(2));
        }
        for _ in 0..2 {
            t.answer_choose(P0, &[obj(pw)]);
        }
        attack_with(&mut t, &[(attacker, pl(P1))]);
        lots(&mut t, P0);
        t.resolve_all();
        let made = named_tokens(&t, P0, token);
        assert!(!made.is_empty(), "{card}");
        for m in made {
            assert!(attacking(&t, m), "{card}");
            assert_eq!(attack_target(&t, m), Some(obj(pw)), "{card}");
        }
        assert_eq!(attack_target(&t, attacker), Some(pl(P1)), "{card}");
    }
}

#[test]
fn dalkovan_encampment_delayed_trigger_works_each_declaration_this_turn() {
    cr!("603.7a", "603.7b", "508.1m", "500.8");
    ruling!(
        "Dalkovan Encampment",
        "Activating Dalkovan Encampment’s last ability creates a delayed triggered ability that will trigger each time you declare at least one creature as an attacker during a declare attackers step this turn."
    );
    supported("Dalkovan Encampment");
    // No attackers declared: nothing.
    let mut t = TestGame::new(2);
    let camp = t.battlefield(P0, "Dalkovan Encampment");
    act(&mut t, P0, camp, 1, &[]);
    t.set_step(P0, Step::BeginningOfCombat);
    attack_with(&mut t, &[]);
    t.resolve_all();
    assert!(named_tokens(&t, P0, "Warrior").is_empty());
    // The Encampment leaves the battlefield; two combats: two Warriors each time.
    let mut t = TestGame::new(2);
    let camp = t.battlefield(P0, "Dalkovan Encampment");
    let bears = t.battlefield(P0, "Grizzly Bears");
    act(&mut t, P0, camp, 1, &[]);
    exile(&mut t, camp);
    attack_with(&mut t, &[(bears, pl(P1))]);
    t.resolve_all();
    let first = named_tokens(&t, P0, "Warrior");
    assert_eq!(first.len(), 2);
    assert!(first.iter().all(|w| attacking(&t, *w) && tapped(&t, *w)));
    t.advance_to(P0, Step::PostcombatMain);
    let spell = t.hand(P0, "Relentless Assault");
    lots(&mut t, P0);
    t.cast(P0, spell).go();
    t.resolve_all();
    crate::r_s09_common::declare_in_next_combat(&mut t, P0, &[(bears, pl(P1))]);
    t.resolve_all();
    assert_eq!(named_tokens(&t, P0, "Warrior").len(), 4);
}

#[test]
fn dalkovan_encampment_needs_a_swamp_or_mountain_already() {
    cr!("614.1d", "614.12");
    ruling!(
        "Dalkovan Encampment",
        "You must already control a Swamp or Mountain as Dalkovan Encampment enters for it to enter untapped. If it enters at the same time as a Swamp or Mountain when you control no other Swamps or Mountains, it will enter tapped."
    );
    let mut t = TestGame::new(2);
    let both =
        crate::r_p116_common::enter_together(&mut t, P0, &["Dalkovan Encampment", "Mountain"]);
    let camp = both
        .iter()
        .copied()
        .find(|id| t.g.obj(*id).chars.name.as_str() == "Dalkovan Encampment")
        .unwrap();
    assert!(tapped(&t, camp));
    let camp = t.enter(P0, "Dalkovan Encampment");
    assert!(!tapped(&t, camp), "a Mountain already on the battlefield");
}

#[test]
fn zurgo_triggers_for_each_token_leaving_with_it() {
    cr!("603.10a", "603.6c");
    ruling!(
        "Zurgo Stormrender",
        "If Zurgo Stormrender leaves the battlefield at the same time as one or more creature tokens you control, its last ability will trigger once for each of those creature tokens."
    );
    supported("Zurgo Stormrender");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Zurgo Stormrender");
    cast_at(&mut t, P0, "Raise the Alarm", &[]);
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 2);
    cast_at(&mut t, P0, "Wrath of God", &[]);
    t.resolve_all();
    assert_eq!(
        t.life(P1),
        18,
        "one trigger per token (not attacking: lose 1 each)"
    );
}

#[test]
fn stadium_headliner_counts_creatures_on_resolution() {
    cr!("608.2h", "602.2");
    ruling!(
        "Stadium Headliner",
        "The number of creatures you control is counted at the time the activated ability resolves. This means that an opponent can respond by destroying your creatures to reduce the amount of damage dealt."
    );
    let mut t = TestGame::new(2);
    let h = t.battlefield(P0, "Stadium Headliner");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Hill Giant");
    let maw = t.battlefield(P1, "Colossal Dreadmaw");
    act_only(&mut t, P0, h, 0, &[obj(maw)]);
    kill(&mut t, bears);
    t.resolve_all();
    assert_eq!(crate::r_p009_common::dmg(&t, maw), 1);
}

#[test]
fn cybermen_squadron_myriad_on_itself_and_separate_instances() {
    cr!("702.116a", "702.116b", "613.1f");
    ruling!(
        "Cybermen Squadron",
        "Cybermen Squadron will itself have myriad as long as it remains a nonlegendary artifact creature."
    );
    ruling!(
        "Cybermen Squadron",
        "Multiple instances of myriad trigger separately."
    );
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "Cybermen Squadron");
    assert!(has_kw(&t, c, KeywordKind::Myriad));
    t.battlefield(P0, "Leyline of Singularity");
    assert!(!has_kw(&t, c, KeywordKind::Myriad), "legendary: no myriad");
    // Two Squadrons: each gives the other myriad; attacking P1 in a three-player game
    // makes two tokens attacking P2 (one per instance).
    let mut t = TestGame::new(3);
    let a = t.battlefield(P0, "Cybermen Squadron");
    t.battlefield(P0, "Cybermen Squadron");
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    attack_with(&mut t, &[(a, pl(P1))]);
    t.resolve_all();
    let copies = tokens(&t, P0);
    assert_eq!(copies.len(), 2);
    for c in copies {
        assert_eq!(attack_target(&t, c), Some(pl(P2)));
    }
}

/// P0's Stangg (with `attachments` attached, created for `owner` of each) attacks P1 and
/// its trigger resolves. Returns Stangg.
fn stangg_attacks(t: &mut TestGame, attachments: &[(PlayerId, &str)]) -> ObjectId {
    let s = t.battlefield(P0, "Stangg, Echo Warrior");
    for (p, a) in attachments {
        attach_new(t, *p, a, s);
    }
    attack_with(t, &[(s, pl(P1))]);
    t.resolve_all();
    s
}

#[test]
fn stangg_tokens_doubled_by_a_replacement_are_still_sacrificed() {
    cr!("614.1a", "603.7c", "111.1");
    ruling!(
        "Stangg, Echo Warrior",
        "If a replacement effect causes the created tokens to have different characteristics or causes additional tokens to be created this way, those tokens will still be sacrificed at the beginning of the next end step."
    );
    supported("Stangg, Echo Warrior");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Doubling Season");
    stangg_attacks(&mut t, &[(P0, "Bonesplitter")]);
    // Two Bonesplitter tokens (and Stangg Twins, of which the legend rule leaves one).
    assert_eq!(named_tokens(&t, P0, "Bonesplitter").len(), 2);
    assert!(!named_tokens(&t, P0, "Stangg Twin").is_empty());
    end_step(&mut t);
    assert!(tokens(&t, P0).is_empty(), "all sacrificed");
    // Divine Visitation: a 4/4 white Angel is created instead of Stangg Twin (different
    // characteristics); it's sacrificed all the same.
    supported("Divine Visitation");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Divine Visitation");
    stangg_attacks(&mut t, &[]);
    let angels = named_tokens(&t, P0, "Angel");
    assert_eq!(angels.len(), 1);
    assert!(named_tokens(&t, P0, "Stangg Twin").is_empty());
    assert_eq!(t.pt(angels[0]), (4, 4));
    end_step(&mut t);
    assert!(tokens(&t, P0).is_empty(), "the Angel is sacrificed");
}

#[test]
fn stangg_tokens_another_player_controls_arent_sacrificed() {
    cr!("701.21a", "603.7c");
    ruling!(
        "Stangg, Echo Warrior",
        "If another player gains control of the tokens created this way, the ability that causes you to sacrifice them will still trigger. However, any tokens you no longer control can't be sacrificed and will remain on the battlefield."
    );
    let mut t = TestGame::new(2);
    stangg_attacks(&mut t, &[]);
    let twin = named_tokens(&t, P0, "Stangg Twin")[0];
    give_control(&mut t, twin, P1);
    t.advance_to(P0, Step::End);
    t.settle();
    assert!(t.stack_len() >= 1, "the sacrifice ability still triggers");
    t.resolve_all();
    assert!(t.on_battlefield(twin));
    assert_eq!(ctl(&t, twin), P1);
}

#[test]
fn stangg_aura_token_that_cant_attach_isnt_created() {
    cr!("303.4f", "303.4g", "701.3a", "702.16d");
    ruling!(
        "Stangg, Echo Warrior",
        "If, for any reason, an Aura token that would be created this way can't be attached to Stangg Twin, that token isn't created. If an Equipment token that can't be attached to Stangg Twin would be created, that token enters the battlefield unattached."
    );
    // P1's Betrayal ("Enchant creature an opponent controls") on Stangg: P0's copy of it
    // couldn't enchant P0's own Stangg Twin, so it isn't created.
    let mut t = TestGame::new(2);
    stangg_attacks(&mut t, &[(P1, "Betrayal")]);
    assert!(named_tokens(&t, P0, "Betrayal").is_empty());
    assert!(named_tokens(&t, P1, "Betrayal").is_empty());
    assert_eq!(named_tokens(&t, P0, "Stangg Twin").len(), 1);
    // A Bonesplitter token that can't be attached to Stangg Twin (protection from
    // artifacts) enters unattached.
    let mut t = TestGame::new(2);
    let ward = crate::r_s01_common::custom_card(
        "Token Ward",
        "Enchantment",
        "{1}",
        None,
        "Creature tokens have protection from artifacts.",
    );
    t.custom(P0, ward, mtg_engine::object::Zone::Battlefield);
    stangg_attacks(&mut t, &[(P0, "Bonesplitter")]);
    let copies = named_tokens(&t, P0, "Bonesplitter");
    assert_eq!(copies.len(), 1);
    assert_eq!(attached_to(&t, copies[0]), None);
}
