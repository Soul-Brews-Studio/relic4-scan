# Proof book — build.  (from /oracle-write-proof-book)
#
# make images  captures/*.txt + diagrams/*.txt -> images/*.png   (/ascii-diagram --png)
# make pdf     Thai word-break -> pandoc -> fixdash -> imgwidth -> typst -> book.pdf
# make check   the gates: a clean compile is NOT enough
# make verify  mechanical review in 0.1 s (REVIEW-auto.md) — run BEFORE any LLM reviewer
# make sheet   contact sheet of pages A..A+15 (A=1 by default) -> build/sheet.png, then LOOK
#
# typst >= 0.15.1 REQUIRED (0.14.x mis-stacks Thai tone marks). Fonts are vendored in ./fonts
# and passed with --font-path: pandoc emits no font directive, so typst would silently fall
# back to a face with no Thai.

CHAPTERS := $(sort $(wildcard [0-9][0-9]-*.md))
BUILD := build
PDF   := book.pdf
RENDER := uv run --quiet --with pillow python $(HOME)/.claude/skills/ascii-diagram/render_png.py
A ?= 1

.PHONY: pdf images check sheet clean verify

pdf: $(PDF)

images:
	python3 cards.py 42
	@for d in diagrams/*.txt; do [ -f "$$d" ] && $(RENDER) $$d images/$$(basename $$d .txt).png; done; true

$(BUILD):
	mkdir -p $(BUILD)

$(BUILD)/zwsp.md: $(CHAPTERS) thai-wordbreak.py | $(BUILD)
	uvx --quiet --from pythainlp python3 thai-wordbreak.py $(CHAPTERS) > $@

# pandoc writes image("images/x.png", alt: …) resolved relative to build/full.typ; with
# --root . a leading "/" makes it root-relative, so the same .md still renders on GitHub.
# fixdash: typst turns -- into an en dash in markup, so every CLI flag outside code broke.
# imgwidth: size each image by its pixel width so terminal text prints at ONE size.
$(BUILD)/body.typ: $(BUILD)/zwsp.md fixdash.py imgwidth.py
	pandoc $< -o $@ -f markdown-smart -t typst-smart   # -smart: keep — as —; else pandoc writes --- and fixdash escapes it to literal ---
	sed -i.bak -e 's/#horizontalrule/#line(length: 100%)/g' -e 's#image("images/#image("/images/#g' $@ && rm -f $@.bak
	python3 fixdash.py $@
	uv run --quiet --with pillow python imgwidth.py $@

$(PDF): book.typ $(BUILD)/body.typ
	cat book.typ $(BUILD)/body.typ > $(BUILD)/full.typ
	typst compile --font-path fonts --root . $(BUILD)/full.typ $@

check: $(PDF)
	@typst compile --font-path fonts --root . $(BUILD)/full.typ /dev/null 2>&1 \
	  | grep -i "unknown font" && { echo "FAIL: font fallback"; exit 1; } || echo "ok: no font fallback"
	@python3 -c "import re,sys; \
d=open('$(BUILD)/zwsp.md',encoding='utf-8').read(); \
n=sum(f.count(chr(0x200b)) for f in re.findall(r'\`\`\`.*?\`\`\`',d,flags=re.S)); \
print('ok: no ZWSP in code' if n==0 else 'FAIL: %d ZWSP inside code blocks'%n); sys.exit(0 if n==0 else 1)"
	@missing=0; for f in $(CHAPTERS); do test -s $$f || { echo "FAIL: $$f empty"; missing=1; }; done; \
	  for p in $$(grep -oh '](images/[A-Za-z0-9_.-]*\.png' $(CHAPTERS) | cut -c3- | sort -u); do \
	    test -f $$p || { echo "FAIL: referenced $$p not on disk"; missing=1; }; done; \
	  test $$missing = 0 && echo "ok: $(words $(CHAPTERS)) chapters, every referenced image exists"
	@n=$$(pdftotext $(PDF) - | grep -cE '[–—][a-z][a-z0-9-]*[a-z0-9]'); \
	  test $$n = 0 && echo 'ok: no CLI flag printed as a dash' || { echo "FAIL: $$n flags mangled into dashes"; exit 1; }
	@test -s images/cover-art.jpg && echo "ok: cover art present (AI-generated: README must say so)" || { echo "FAIL: images/cover-art.jpg missing"; echo "  zsh ~/.claude/skills/oracle-write-proof-book/scripts/make-cover-art.sh $(CURDIR) \"<one quiet object tied to the story>\""; exit 1; }
	@grep -q 'ยกเว้นภาพปก' book.typ && echo "ok: tagline discloses the generated cover art" || { echo "FAIL: book-tagline must say ยกเว้นภาพปกที่ AI วาด (cover art is generated, not a capture)"; exit 1; }
	@pdfinfo $(PDF) | grep Pages
	@python3 page_images.py $(PDF) --code-pages-ok $(if $(WARN),--warn,)   # every content page: 1-2 images, 3 if small; WARN=1 = report only

verify:
	@python3 verify.py   # 0.1 s, 0 tokens: code vs captures, every id/time/size vs captures, steps, images -> REVIEW-auto.md

sheet: $(PDF)
	@rm -f $(BUILD)/th-*.png
	pdftoppm -r 28 -png -f $(A) -l $$(( $(A) + 15 )) $(PDF) $(BUILD)/th
	uv run --quiet --with pillow python $(HOME)/.claude/skills/oracle-write-proof-book/scripts/contact_sheet.py $(BUILD) $(BUILD)/sheet.png
	@echo "-> LOOK at $(BUILD)/sheet.png"

clean:
	rm -rf $(BUILD) $(PDF)
