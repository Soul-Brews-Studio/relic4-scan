# scripts

Operator scripts used to run the relic chain across machines. They are not part of the crate or the app, and they carry no
keys: S3 keys come from `~/.config/relic-scan/s3.env` (mode 600) or the environment. Read a script before you run it; each one
has paths for the 2026-10-24 setup (m5, mba, white, black), so adjust them.

## claude-home: one `~/.claude/projects` on the main machine

The order that was used to fold many `.claude*` homes into one. Nothing here deletes a transcript; the last step is yours.

| script | what it does |
|---|---|
| `claude-merge.py [--dry]` | copies files that are missing into `~/.claude/projects`. A longer copy replaces a shorter prefix (the old one goes to `/tmp/claude-merge/replaced`); a copy that is neither goes to `/tmp/claude-merge/diverged` |
| `claude-union.py plan\|backup-originals\|apply` | for a session split in two, appends the messages (by `uuid`) that only the diverged copy has |
| `claude-verify.py <dir>` | read only: every message of a pulled copy is already in `~/.claude/projects`; exit 1 if not. Run it before deleting the other copy |
| `extra-merge.py` | copies the files an old home has that the main one lacks, then checks the rest by `uuid` |
| `backup-to-s3.sh` | tars a staging folder, uploads it to the bucket with its sha256 in `x-amz-meta-sha256`, and checks the store's size and hash |
| `claude-finish.sh` | backup, then union, then re-plan |
| `claude-move-sources.sh` | moves merged-in mirrors and archives to `/tmp/claude-sources` (a rename on one volume) |

To pull another machine's transcripts: `rsync -a host:.claude/projects/ /tmp/claude-merge/from-<host>/<home>/`, then merge, verify,
back up, and only then delete on that machine. Leave a home that agents still write to.

## export

`relic-export-nightly` is what the nightly timer runs (`studio.soulbrews.relic-export` on macOS, `relic-export.timer` on Linux):
`relic-scan export --to s3://…`. It refuses to replace a sealed month; use `--overwrite` by hand after a month is sealed again.

## release

`relic-release-build.sh` (CLI, Apple-silicon and Intel `.dmg`) and `relic-release-linux.sh` (`.deb` and `.AppImage`, built in
Docker on white) read the version from `/tmp/relic-release-version` and write to `/tmp/relic-release-<version>`.
`relic-app.sh` builds the desktop app. Remove a running binary before copying over it (`rm -f`), or macOS kills it.
