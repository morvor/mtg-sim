use mtg_engine::card::Layout;
use mtg_engine::oracle::{self, CompileContext};
use mtg_engine::types::*;

#[test]
fn zz_probe() {
    for (tl, text) in [
        ("Instant", "You may choose new targets for target instant or sorcery spell."),
        ("Instant", "You may choose new targets for target instant or sorcery spell. Then copy that spell."),
        ("Instant", "Choose new targets for target instant or sorcery spell. Then copy that spell. You may choose new targets for the copy."),
        ("Instant", "You may choose new targets for target instant or sorcery spell. Then copy that spell. You may choose new targets for the copy."),
        ("Creature — Cat Warrior", "This creature must be blocked by exactly one creature if able."),
        ("Creature — Cat Warrior", "Whenever this creature attacks, create X tokens that are copies of it and that are tapped and attacking, where X is the number of creatures defending player controls. Exile the tokens at the beginning of the next end step."),
        ("Artifact — Vehicle", "At the beginning of your end step, if you sacrificed a permanent this turn, create a token that's a copy of this Vehicle."),
        ("Artifact — Vehicle", "As long as you control eight or more permanents named Phoenix Fleet Airship, this Vehicle is an artifact creature."),
        ("Creature — Insect", "Whenever you sacrifice a land, create a tapped token that's a copy of this creature if seven or more land cards are in your graveyard. Otherwise, create a tapped 1/1 black Insect creature token with flying."),
        ("Enchantment", "Whenever you attack, for each opponent, create a 1/1 black Ninja creature token that's tapped and attacking that player."),
        ("Creature — Weird", "Whenever you cast an instant or sorcery spell from your library, copy it. You may choose new targets for the copy."),
        ("Artifact", "If you would copy a spell one or more times, instead copy it that many times plus an additional time. You may choose new targets for the additional copy."),
        ("Creature — Dinosaur", "When this creature dies, if it's not a token, create a token that's a copy of it, except it's an artifact in addition to its other types."),
        ("Creature — Avatar", "Creatures you control enter as a copy of this creature."),
    ] {
        let t = TypeLine::parse(tl);
        let ctx = CompileContext {
            card_name: "X",
            full_name: "X",
            type_line: &t,
            layout: Layout::Normal,
            face_index: 0,
            keywords: &[],
            power: Some("1"),
            toughness: Some("1"),
        };
        let c = oracle::compile(text, &ctx);
        println!("{text}\n   => {:?}", c.unsupported);
    }
}
