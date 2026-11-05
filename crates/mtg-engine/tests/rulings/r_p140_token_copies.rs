//! Rulings batch P140 — token copies of artifacts: Mechanized Production, Court of
//! Vantress, Mirrorworks, Schema Thief, Chrome Dome, Donatello, Gadget Master, Prototype
//! Portal and Daretti, Ingenious Iconoclast.

use crate::r_p140_common::*;
use crate::r_p141_common::{into_upkeep, pump, yes};
use mtg_engine::designations::become_monarch;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::{counters, CardType, Color, ColorSet};
use mtg_engine::*;

/// A 1/1 colorless Thopter artifact creature token with flying for `p`, made by a real
/// card (Fairgrounds Patrol's graveyard ability).
fn thopter(t: &mut TestGame, p: PlayerId) -> ObjectId {
    let before = tokens_typed(t, p, "Thopter");
    let patrol = t.graveyard(p, "Fairgrounds Patrol");
    let (active, step) = (t.g.turn.active, t.g.turn.step);
    t.set_step(p, mtg_engine::turn::Step::PrecombatMain);
    activate_lots(t, p, patrol, 0, &[]);
    t.resolve_all();
    t.set_step(active, step);
    *tokens_typed(t, p, "Thopter")
        .iter()
        .find(|x| !before.contains(x))
        .expect("a Thopter")
}

/// A Thopter token that's been changed: a +1/+1 counter, +2/+2 and tapped.
fn changed_thopter(t: &mut TestGame, p: PlayerId) -> ObjectId {
    let th = thopter(t, p);
    put_counters(t, th, counters::PLUS1, 1);
    pump(t, th, 2, 2);
    t.g.objects[th.0 as usize].tapped = true;
    assert_eq!(t.pt(th), (4, 4));
    th
}

/// The tokens `p` controls that weren't there before.
fn new_tokens(t: &TestGame, p: PlayerId, before: &[ObjectId]) -> Vec<ObjectId> {
    tokens_of(t, p)
        .into_iter()
        .filter(|x| !before.contains(x))
        .collect()
}

/// Asserts `tok` is a plain copy of a Fairgrounds Patrol Thopter token: a 1/1 flying
/// Thopter token, untapped, with no counters.
fn assert_plain_thopter(t: &mut TestGame, tok: ObjectId) {
    t.g.recompute();
    let o = t.obj(tok);
    assert!(o.is_token());
    assert!(o.chars.has_subtype("Thopter"));
    assert!(o.chars.is(CardType::Artifact));
    assert!(o.has_keyword(mtg_engine::keywords::KeywordKind::Flying));
    assert!(!o.tapped);
    assert_eq!(t.counters(tok, counters::PLUS1), 0);
    assert_eq!(t.pt(tok), (1, 1));
}

/// Puts Mechanized Production on `p`'s artifact `what` (cast for real) and goes to `p`'s
/// upkeep with its trigger on the stack.
fn produce(t: &mut TestGame, p: PlayerId, what: ObjectId) {
    cast_lots(t, p, "Mechanized Production", &[obj(what)]);
    t.resolve_all();
    assert!(!t.named_on_battlefield("Mechanized Production").is_empty());
    into_upkeep(t, p);
}

/// The single token `p` got since `before`.
fn one_new_token(t: &TestGame, p: PlayerId, before: &[ObjectId]) -> ObjectId {
    let n = new_tokens(t, p, before);
    assert_eq!(n.len(), 1, "{n:?}");
    n[0]
}

#[test]
fn mechanized_production_wins_only_while_its_trigger_resolves_before_token_triggers() {
    cr!("603.3", "608.2c", "104.2b");
    ruling!(
        "Mechanized Production",
        "If you control eight or more artifacts that share a name while you control Mechanized Production, you won't win the game yet. You'll win the game while resolving its triggered ability during your upkeep."
    );
    ruling!(
        "Mechanized Production",
        "Any abilities that trigger on the token being created won't resolve until after Mechanized Production's triggered ability has finished resolving entirely and performed its check for eight artifacts with the same name."
    );
    supported("Mechanized Production");
    // Eight Ichor Wellsprings: no win yet.
    let mut t = TestGame::new(2);
    let mut wells: Vec<ObjectId> = (0..8)
        .map(|_| t.battlefield(P0, "Ichor Wellspring"))
        .collect();
    cast_lots(&mut t, P0, "Mechanized Production", &[obj(wells[0])]);
    t.resolve_all();
    assert!(!t.has_lost(P1) && t.g.result.is_none(), "not yet");
    // Seven: the eighth is the token, and P0 wins before its enters trigger resolves.
    let mut t = TestGame::new(2);
    wells.clear();
    wells.extend((0..7).map(|_| t.battlefield(P0, "Ichor Wellspring")));
    produce(&mut t, P0, wells[0]);
    assert!(t.g.result.is_none());
    let hand = t.hand_size(P0);
    t.g.resolve_top();
    assert!(t.has_lost(P1), "won while resolving");
    assert_eq!(
        t.hand_size(P0),
        hand,
        "the token's enters trigger hasn't resolved"
    );
}

#[test]
fn enters_abilities_work_for_copied_artifact_tokens() {
    cr!("707.2", "614.1c", "603.6a");
    ruling!(
        "Mechanized Production",
        "Any enters-the-battlefield abilities of the copied artifact trigger when the artifact token enters the battlefield. The artifact token also has any \"this enters the battlefield with\" or \"as this enters the battlefield\" abilities that the copied artifact has."
    );
    ruling!(
        "Court of Vantress",
        "Any enters-the-battlefield abilities of the copied permanent will trigger when the token enters the battlefield. Any \"as [this permanent] enters the battlefield\" or \"[this permanent] enters the battlefield with\" abilities of the permanent will also work."
    );
    ruling!(
        "Chrome Dome",
        "If the artifact copied by the token had any \"when [this artifact] enters\" abilities, the token also has those abilities, and they'll trigger when it's created. Similarly, any \"as [this artifact] enters\" or \"[this artifact] enters with\" abilities that the token has copied will also work."
    );
    ruling!(
        "Donatello, Gadget Master",
        "If the artifact copied by the token had any \"when [this artifact] enters\" abilities, the token also has those abilities, and they'll trigger when it's created. Similarly, any \"as [this artifact] enters\" or \"[this artifact] enters with\" abilities that the token has copied will also work."
    );
    supported("Pithing Needle");
    // Ichor Wellspring: "When this artifact enters ..., draw a card." Tetravus: "enters with
    // three +1/+1 counters". Pithing Needle: "As this artifact enters, choose a card name."
    for how in [
        "Mechanized Production",
        "Court of Vantress",
        "Chrome Dome",
        "Donatello, Gadget Master",
    ] {
        supported(how);
        for what in ["Ichor Wellspring", "Tetravus", "Pithing Needle"] {
            let mut t = TestGame::new(2);
            let orig = t.battlefield(P0, what);
            let stone = t.battlefield(P1, "Millstone");
            lots(&mut t, P1);
            assert!(
                can_activate(&mut t, P1, stone),
                "no name chosen for the original"
            );
            t.g.turn.priority = Some(P0);
            t.answer(
                P0,
                DecisionKind::Name,
                mtg_engine::decision::Answer::Text("Millstone".into()),
            );
            let before = tokens_of(&t, P0);
            let hand = t.hand_size(P0);
            match how {
                "Mechanized Production" => {
                    produce(&mut t, P0, orig);
                }
                "Court of Vantress" => {
                    t.battlefield(P0, "Court of Vantress");
                    become_monarch(&mut t.g, P0);
                    t.answer_targets(P0, &[obj(orig)]);
                    yes(&mut t, P0);
                    into_upkeep(&mut t, P0);
                }
                "Chrome Dome" => {
                    let dome = t.battlefield(P0, "Chrome Dome");
                    activate_lots(&mut t, P0, dome, 0, &[obj(orig)]);
                }
                _ => {
                    let don = t.battlefield(P0, "Donatello, Gadget Master");
                    t.answer_targets(P0, &[obj(orig)]);
                    t.attack(&[(don, Entity::Player(P1))], &[]);
                }
            }
            // Decline Tetravus's own upkeep options.
            t.answer(
                P0,
                DecisionKind::YesNo,
                mtg_engine::decision::Answer::Bool(false),
            );
            t.resolve_all();
            let tok = *new_tokens(&t, P0, &before)
                .iter()
                .find(|x| t.obj(**x).chars.name.as_str() == what)
                .unwrap_or_else(|| panic!("{how}: no token copy of {what}"));
            match what {
                "Tetravus" => {
                    assert_eq!(t.counters(tok, counters::PLUS1), 3, "{how}: enters with")
                }
                "Pithing Needle" => {
                    lots(&mut t, P1);
                    assert!(
                        !can_activate(&mut t, P1, stone),
                        "{how}: a name chosen as it entered"
                    );
                }
                _ => assert_eq!(t.hand_size(P0), hand + 1, "{how}: its enters trigger"),
            }
        }
    }
}

#[test]
fn token_copies_use_the_printed_values_plus_copy_effects() {
    cr!("707.2", "707.3");
    ruling!(
        "Court of Vantress",
        "Court of Vantress copies the printed values of the enchantment or artifact plus any copy effects that have been applied to it."
    );
    ruling!(
        "Mirrorworks",
        "As the token is created, it checks the printed values of the artifact it's copying, as well as any copy effects that have been applied to it."
    );
    ruling!(
        "Schema Thief",
        "Each of the tokens copy exactly what was printed on the original artifact and nothing else"
    );
    ruling!(
        "Schema Thief",
        "It doesn’t copy whether that artifact is tapped or untapped, whether it has any counters on it or Auras and Equipment attached to it, or any non-copy effects that have changed its power, toughness, types, color, and so on."
    );
    supported("Schema Thief");
    // Court: Sculpting Steel copying Ornithopter, with counters and a pump.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Court of Vantress");
    become_monarch(&mut t.g, P0);
    let o = t.battlefield(P1, "Ornithopter");
    t.answer_choose(P0, &[obj(o)]);
    let steel = t.enter(P0, "Sculpting Steel");
    t.settle();
    put_counters(&mut t, steel, counters::PLUS1, 2);
    pump(&mut t, steel, 1, 1);
    let before = tokens_of(&t, P0);
    t.answer_targets(P0, &[obj(steel)]);
    yes(&mut t, P0);
    into_upkeep(&mut t, P0);
    t.resolve_all();
    let tok = one_new_token(&t, P0, &before);
    assert_eq!(t.obj(tok).chars.name.as_str(), "Ornithopter");
    assert_eq!(t.pt(tok), (0, 2));
    // Mirrorworks: Phyrexian Metamorph entering as a copy of Grizzly Bears.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mirrorworks");
    let bears = t.battlefield(P1, "Grizzly Bears");
    lots(&mut t, P0);
    t.answer_choose(P0, &[obj(bears)]);
    yes(&mut t, P0);
    let before = tokens_of(&t, P0);
    let meta = t.enter(P0, "Phyrexian Metamorph");
    t.settle();
    assert_eq!(t.stack_len(), 1, "Mirrorworks's trigger");
    // A non-copy effect on it before the token is created isn't copied.
    pump(&mut t, meta, 2, 2);
    t.resolve_all();
    let tok = one_new_token(&t, P0, &before);
    let o = t.obj(tok);
    assert_eq!(o.chars.name.as_str(), "Grizzly Bears");
    assert!(
        o.chars.is(CardType::Artifact),
        "the Metamorph's exception is copied too"
    );
    assert_eq!(t.pt(tok), (2, 2));
    // Schema Thief: a tapped Ornithopter with counters and Equipment, pumped, turned blue
    // and made an enchantment by non-copy effects.
    let mut t = TestGame::new(2);
    let thief = t.battlefield(P0, "Schema Thief");
    let o = t.battlefield(P1, "Ornithopter");
    put_counters(&mut t, o, counters::PLUS1, 2);
    pump(&mut t, o, 1, 1);
    crate::r_p117_common::modify(
        &mut t,
        o,
        vec![
            mtg_engine::ability::Modification::SetColors(ColorSet::single(Color::Blue)),
            mtg_engine::ability::Modification::AddTypes(vec![CardType::Enchantment]),
        ],
    );
    crate::r_s06_common::attach_new(&mut t, P1, "Bonesplitter", obj(o));
    t.g.objects[o.0 as usize].tapped = true;
    assert_eq!(t.pt(o), (5, 5));
    let before = tokens_of(&t, P0);
    t.answer_targets(P0, &[obj(o)]);
    t.attack(&[(thief, Entity::Player(P1))], &[]);
    t.resolve_all();
    let tok = one_new_token(&t, P0, &before);
    assert_eq!(t.obj(tok).chars.name.as_str(), "Ornithopter");
    assert!(!t.obj(tok).tapped);
    assert_eq!(t.counters(tok, counters::PLUS1), 0);
    assert_eq!(t.pt(tok), (0, 2));
    assert_eq!(
        t.obj(tok).chars.colors,
        ColorSet::NONE,
        "colorless as printed"
    );
    assert!(
        !t.obj(tok).chars.is(CardType::Enchantment),
        "its printed types"
    );
    assert!(
        t.g.battlefield
            .iter()
            .all(|a| t.obj(*a).attached_to != Some(obj(tok))),
        "nothing attached to it"
    );
}

#[test]
fn copying_a_token_uses_the_original_characteristics_of_that_token() {
    cr!("707.2", "111.3");
    ruling!(
        "Schema Thief",
        "If the copied artifact is a token, the token that’s created copies the original characteristics of that token as stated by the effect that created that token."
    );
    ruling!(
        "Donatello, Gadget Master",
        "If the copied artifact is a token, the new token that's created copies the original characteristics of that token as stated by the effect that created the token."
    );
    ruling!(
        "Mechanized Production",
        "If the copied artifact is a token, the token that's created copies the original characteristics of that token as stated by the effect that created it."
    );
    ruling!(
        "Chrome Dome",
        "If the copied artifact is a token, the new token that's created copies the original characteristics of that token as stated by the effect that created the token."
    );
    for how in [
        "Schema Thief",
        "Donatello, Gadget Master",
        "Mechanized Production",
        "Chrome Dome",
    ] {
        let mut t = TestGame::new(2);
        let owner = if how == "Schema Thief" { P1 } else { P0 };
        let th = changed_thopter(&mut t, owner);
        let before = tokens_of(&t, P0);
        match how {
            "Mechanized Production" => {
                produce(&mut t, P0, th);
                t.resolve_all();
            }
            "Chrome Dome" => {
                let dome = t.battlefield(P0, how);
                activate_lots(&mut t, P0, dome, 0, &[obj(th)]);
                t.resolve_all();
                // Its "+1/+0" for other artifact creatures aside.
                move_to(&mut t, dome, Zone::Graveyard(P0));
            }
            _ => {
                let c = t.battlefield(P0, how);
                t.answer_targets(P0, &[obj(th)]);
                t.attack(&[(c, Entity::Player(P1))], &[]);
                t.resolve_all();
            }
        }
        let tok = one_new_token(&t, P0, &before);
        assert_plain_thopter(&mut t, tok);
    }
}

#[test]
fn copying_x_costs_uses_zero() {
    cr!("107.3g", "107.3m", "707.2");
    ruling!(
        "Mechanized Production",
        "If the copied artifact has {X} in its mana cost, X is considered to be zero."
    );
    ruling!(
        "Prototype Portal",
        "If the exiled card has {X} in its mana cost (such as Chalice of the Void), that X is considered to be 0. The {X} in Prototype Portal's activation cost takes this into account"
    );
    supported("Prototype Portal");
    // Mechanized Production on a Chalice of the Void with two charge counters.
    let mut t = TestGame::new(2);
    let chalice = t.battlefield(P0, "Chalice of the Void");
    put_counters(&mut t, chalice, counters::CHARGE, 2);
    let before = tokens_of(&t, P0);
    produce(&mut t, P0, chalice);
    t.resolve_all();
    let tok = one_new_token(&t, P0, &before);
    assert_eq!(t.obj(tok).chars.name.as_str(), "Chalice of the Void");
    assert_eq!(t.counters(tok, counters::CHARGE), 0);
    // Prototype Portal: Chalice of the Void (X is 0, activation for free) and Riptide
    // Replicator ({X}{4}: X is 4).
    for (card_, cost) in [("Chalice of the Void", 0usize), ("Riptide Replicator", 4)] {
        let mut t = TestGame::new(2);
        let c = t.hand(P0, card_);
        t.answer_choose_moved(P0, &[obj(c)]);
        yes(&mut t, P0);
        let portal = t.enter(P0, "Prototype Portal");
        t.resolve_all();
        assert!(t.in_exile(card_));
        if cost > 0 {
            t.lands(P0, "Wastes", cost - 1);
            assert!(!can_activate(&mut t, P0, portal), "{card_}: needs {cost}");
            t.lands(P0, "Wastes", 1);
        }
        assert!(can_activate(&mut t, P0, portal), "{card_}");
        t.activate(P0, portal, 0, &[]).expect("activate");
        t.resolve_all();
        let tok = *tokens_of(&t, P0).first().expect("a token");
        assert_eq!(t.obj(tok).chars.name.as_str(), card_);
        assert_eq!(t.counters(tok, counters::CHARGE), 0, "{card_}: X is 0");
    }
}

#[test]
fn prototype_portal_gone_still_makes_the_token_but_not_without_an_exiled_card() {
    cr!("113.7a", "607.2a");
    ruling!(
        "Prototype Portal",
        "If Prototype Portal has left the battlefield by the time its second ability resolves, you'll still put a token onto the battlefield that's a copy of the exiled card. On the other hand, if Prototype Portal is still on the battlefield at this time but there is no exiled card"
    );
    for remove_card in [false, true] {
        let mut t = TestGame::new(2);
        let o = t.hand(P0, "Ornithopter");
        t.answer_choose_moved(P0, &[obj(o)]);
        yes(&mut t, P0);
        let portal = t.enter(P0, "Prototype Portal");
        t.resolve_all();
        t.activate(P0, portal, 0, &[]).expect("activate");
        if remove_card {
            let exiled = t.g.find_in_zone(Zone::Exile, "Ornithopter")[0];
            move_to(&mut t, exiled, Zone::Library(P0));
        } else {
            destroy(&mut t, portal);
            assert!(!t.on_battlefield(portal));
        }
        t.resolve_all();
        let n = t.named_on_battlefield("Ornithopter").len();
        assert_eq!(
            n,
            if remove_card { 0 } else { 1 },
            "remove_card={remove_card}"
        );
    }
}

#[test]
fn court_of_vantress_becoming_a_copy_of_a_token_isnt_a_token() {
    cr!("707.2", "707.9a", "111.3");
    ruling!(
        "Court of Vantress",
        "If Court of Vantress becomes a copy of a token, it copies the original characteristics of that token as defined by the effect that put it onto the battlefield. It won't become a token."
    );
    let mut t = TestGame::new(2);
    let court = t.battlefield(P0, "Court of Vantress");
    let th = changed_thopter(&mut t, P0);
    t.answer_targets(P0, &[obj(th)]);
    yes(&mut t, P0);
    into_upkeep(&mut t, P0);
    t.resolve_all();
    let c = t.g.current(court);
    assert!(t.obj(c).chars.has_subtype("Thopter"));
    assert!(!t.obj(c).is_token());
    assert_eq!(t.pt(c), (1, 1));
    assert!(
        t.obj(c)
            .chars
            .abilities
            .iter()
            .any(|a| a.text.contains("monarch")),
        "it has this ability"
    );
}

#[test]
fn court_of_vantress_as_an_aura_or_equipment_follows_the_attachment_rules() {
    cr!("704.5m", "704.5p", "707.9a");
    ruling!(
        "Court of Vantress",
        "If Court of Vantress becomes a copy of an Aura, it's put into its owner's graveyard unless it's somehow attached to an appropriate object or player already. If it becomes a copy of an Equipment and is attached to a creature, it'll become unattached when it becomes a non-Equipment, non-Aura permanent."
    );
    // An Aura.
    let mut t = TestGame::new(2);
    let court = t.battlefield(P0, "Court of Vantress");
    let drake = t.battlefield(P1, "Wind Drake");
    let grounded = cast_lots(&mut t, P0, "Grounded", &[obj(drake)]);
    t.resolve_all();
    t.answer_targets(P0, &[obj(t.g.current(grounded))]);
    yes(&mut t, P0);
    into_upkeep(&mut t, P0);
    t.resolve_all();
    assert!(
        t.in_graveyard(P0, "Court of Vantress"),
        "an unattached Aura"
    );
    let _ = court;
    // An Equipment, attached, then a copy of Ornithopter: it becomes unattached.
    let mut t = TestGame::new(2);
    let court = t.battlefield(P0, "Court of Vantress");
    let splitter = t.battlefield(P0, "Bonesplitter");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let o = t.battlefield(P0, "Ornithopter");
    t.answer_targets(P0, &[obj(splitter)]);
    yes(&mut t, P0);
    into_upkeep(&mut t, P0);
    t.resolve_all();
    let c = t.g.current(court);
    assert!(t.obj(c).chars.has_subtype("Equipment"));
    t.advance_to(P0, mtg_engine::turn::Step::PrecombatMain);
    activate_lots(&mut t, P0, c, 0, &[obj(bears)]);
    t.resolve_all();
    assert_eq!(t.obj(c).attached_to, Some(obj(bears)));
    assert_eq!(t.pt(bears), (4, 2));
    t.answer_targets(P0, &[obj(o)]);
    yes(&mut t, P0);
    into_upkeep(&mut t, P0);
    t.resolve_all();
    let c = t.g.current(court);
    assert_eq!(t.obj(c).chars.name.as_str(), "Ornithopter");
    assert_eq!(t.obj(c).attached_to, None, "unattached");
    assert_eq!(t.pt(bears), (2, 2));
    // An equipped Equipment that becomes a copy of an Aura ("Enchant creature"): it's
    // already attached to an appropriate object, so it stays, enchanting it.
    let mut t = TestGame::new(2);
    let court = t.battlefield(P0, "Court of Vantress");
    let splitter = t.battlefield(P0, "Bonesplitter");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let drake = t.battlefield(P1, "Wind Drake");
    let grounded = crate::r_s06_common::attach_new(&mut t, P1, "Grounded", obj(drake));
    t.answer_targets(P0, &[obj(splitter)]);
    yes(&mut t, P0);
    into_upkeep(&mut t, P0);
    t.resolve_all();
    t.advance_to(P0, mtg_engine::turn::Step::PrecombatMain);
    let c = t.g.current(court);
    activate_lots(&mut t, P0, c, 0, &[obj(bears)]);
    t.resolve_all();
    assert_eq!(t.obj(c).attached_to, Some(obj(bears)));
    t.answer_targets(P0, &[obj(grounded)]);
    yes(&mut t, P0);
    into_upkeep(&mut t, P0);
    t.resolve_all();
    let c = t.g.current(court);
    assert!(t.on_battlefield(c), "an Aura attached to a creature stays");
    assert_eq!(t.obj(c).chars.name.as_str(), "Grounded");
    assert_eq!(t.obj(c).attached_to, Some(obj(bears)));
}

#[test]
fn copying_court_of_vantress_copies_what_it_is_copying_with_its_ability() {
    cr!("707.9a", "707.3");
    ruling!(
        "Court of Vantress",
        "If another permanent becomes a copy of Court of Vantress, it will become a copy of whatever Court of Vantress is currently copying (if anything) that also has Court of Vantress's last ability."
    );
    let mut t = TestGame::new(2);
    let court = t.battlefield(P0, "Court of Vantress");
    let o = t.battlefield(P0, "Ornithopter");
    t.answer_targets(P0, &[obj(o)]);
    yes(&mut t, P0);
    into_upkeep(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.obj(t.g.current(court)).chars.name.as_str(), "Ornithopter");
    let bears = t.battlefield(P0, "Grizzly Bears");
    become_copy(&mut t, bears, court);
    let b = t.g.current(bears);
    assert_eq!(t.obj(b).chars.name.as_str(), "Ornithopter");
    assert_eq!(t.pt(b), (0, 2));
    assert!(t
        .obj(b)
        .chars
        .abilities
        .iter()
        .any(|a| a.text.contains("monarch")));
    assert!(
        !t.obj(b)
            .chars
            .abilities
            .iter()
            .any(|a| a.text.contains("you become the monarch")),
        "not Court's first ability"
    );
}

#[test]
fn mirrorworks_one_payment_one_token_and_it_uses_last_known_information() {
    cr!("608.2h", "118.12", "707.2");
    ruling!(
        "Mirrorworks",
        "Each time the ability triggers, you can pay {2} only one time to get one token."
    );
    ruling!(
        "Mirrorworks",
        "If the artifact that caused Mirrorworks's ability to trigger has already left the battlefield by the time the ability resolves, you can still pay {2}. If you do, you'll still put a token onto the battlefield. That token has the copiable values of the characteristics of that nontoken artifact as it last existed on the battlefield."
    );
    supported("Mirrorworks");
    // Sculpting Steel enters as a copy of Millstone; it may leave the battlefield (back
    // to being a Sculpting Steel card) before the ability resolves.
    for leave in [false, true] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Mirrorworks");
        let model = t.battlefield(P1, "Millstone");
        lots(&mut t, P0);
        t.answer_choose(P0, &[obj(model)]);
        let steel = t.enter(P0, "Sculpting Steel");
        t.settle();
        assert_eq!(t.obj(t.g.current(steel)).chars.name.as_str(), "Millstone");
        assert_eq!(t.stack_len(), 1);
        if leave {
            move_to(&mut t, steel, Zone::Exile);
            assert!(t.in_exile("Sculpting Steel"));
        }
        // Two yeses queued: only one payment is offered.
        yes(&mut t, P0);
        yes(&mut t, P0);
        t.resolve_all();
        let toks = tokens_of(&t, P0);
        assert_eq!(toks.len(), 1, "leave={leave}");
        assert_eq!(
            t.obj(toks[0]).chars.name.as_str(),
            "Millstone",
            "leave={leave}"
        );
    }
}

#[test]
fn daretti_targets_on_activation_and_sacrifices_on_resolution() {
    cr!("601.2c", "608.2d", "608.2b");
    ruling!(
        "Daretti, Ingenious Iconoclast",
        "For the second ability, you choose the target artifact or creature as you activate the ability. You choose whether to sacrifice an artifact and which artifact to sacrifice as the ability resolves."
    );
    ruling!(
        "Daretti, Ingenious Iconoclast",
        "For the last ability, if you choose an artifact on the battlefield as the target, and that artifact is no longer on the battlefield as the ability tries to resolve, the ability won’t resolve and none of its effects will happen. You won’t get any tokens."
    );
    supported("Daretti, Ingenious Iconoclast");
    // −1: no artifact on activation; one arrives in response and is sacrificed.
    for sac in [true, false] {
        let mut t = TestGame::new(2);
        let d = t.battlefield(P0, "Daretti, Ingenious Iconoclast");
        set_loyalty(&mut t, d, 5);
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.activate(P0, d, 1, &[obj(bears)]).expect("activate");
        let o = t.battlefield(P0, "Ornithopter");
        t.answer_yes(P0, sac);
        t.answer_choose(P0, &[obj(o)]);
        t.resolve_all();
        assert_eq!(t.on_battlefield(bears), !sac, "sac={sac}");
        assert_eq!(t.on_battlefield(o), !sac);
    }
    // −6: the targeted artifact is destroyed in response: no tokens.
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Daretti, Ingenious Iconoclast");
    set_loyalty(&mut t, d, 10);
    let o = t.battlefield(P1, "Ornithopter");
    t.activate(P0, d, 2, &[obj(o)]).expect("activate");
    destroy(&mut t, o);
    assert!(t.in_graveyard(P1, "Ornithopter"));
    t.resolve_all();
    assert!(tokens_of(&t, P0).is_empty());
    // Positive control.
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Daretti, Ingenious Iconoclast");
    set_loyalty(&mut t, d, 10);
    let o = t.battlefield(P1, "Ornithopter");
    t.activate(P0, d, 2, &[obj(o)]).expect("activate");
    t.resolve_all();
    assert_eq!(tokens_of(&t, P0).len(), 3);
}
