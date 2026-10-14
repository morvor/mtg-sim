# CR exemptions

One `.tsv` file per area (e.g. `2xx-card-parts.tsv`), each line `<rule id><TAB><reason>`.
Exempt only rules with no engine-testable behavior (purely informational or
presentational text, tournament/draft procedures). Everything else needs a test citing it
with `cr!(...)`. `mtg-tools cr-coverage` reads every file here.
