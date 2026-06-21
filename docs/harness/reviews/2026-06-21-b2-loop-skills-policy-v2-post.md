FAIL doctor skill validation can false-pass missing or non-frontmatter repo skills

- [HIGH] `docs/harness/bin/doctor.sh:356` adds a process-critical skills gate, but `docs/harness/bin/doctor.sh:368` only iterates files that already exist and `docs/harness/bin/doctor.sh:371`/`docs/harness/bin/doctor.sh:372` only grep `name:` and `description:` anywhere in the file. This contradicts the new policy in `docs/harness/SKILLS.md:38`-`40` requiring `skills/<name>/SKILL.md` plus YAML frontmatter. Evidence: in a clean temporary clone, deleting `skills/loop-engineering/SKILL.md` still produced `missing_skill_exit=0`; replacing the frontmatter with body-only `name:`/`description:` still produced `body_only_frontmatter_exit=0`. Fix by requiring each inventoried current skill file explicitly and validating the frontmatter block, not just arbitrary matching lines.

REVIEW_VERDICT: FAIL doctor skill validation does not actually enforce inventoried skill existence or YAML frontmatter
