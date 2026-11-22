# Avoiding Claudelish

| Field | Value |
| --- | --- |
| Status | Living document |
| Date | 2026-11-22 |
| Applies to | Every Markdown document and all API documentation in this repository |

## Abstract

This guide defines Claudelish, the recognizable prose habits of text written by Claude and other large language models, and sets the rules that keep it out of this repository. It catalogues each habit with an example and a plain rewrite, and it marks for each rule whether a tool or a reviewer checks it. It lists fixed phrases and six patterns that a reviewer searches for, because no tool checks them yet, and it gives a revision procedure for every draft.

## 1. Introduction

Claudelish is the prose of unedited large language model (LLM) output, as Section 2.2 defines. The name extends "Claudish", which two recent sources use for the habits of Claude in particular [LESSWRONG-ACTORS] [WATERLOO-CLAUDISH]. The wider term for unwanted generated text is "slop" [WILLISON-SLOP].

An agent writes nearly all the prose in this repository. A reader who meets the habits of unedited model output stops trusting the text, and then skims it or rechecks its facts. One article makes the same point about generated READMEs, which look finished before anyone has edited them [HACKERNOON-README]. A news report says that maintainers find the review of AI-assisted pull requests costlier than before [REGISTER-REVIEW].

This guide adds to the writing guide and covers the same text, which is Markdown documents and API documentation. Where the two disagree, the writing guide prevails, as Section 4 shows for headings in title case.

The guide judges the text and never its author. A single tell proves nothing about who wrote a text, and Wikipedia's editors say that text with these indicators is not always generated [WIKI-AI]. A reviewer reports the phrase and its rewrite and does not guess at authorship.

Each rule gives the form to write along with the form to avoid, because Anthropic's guidance advises telling Claude what to do instead of what not to do [ANTHROPIC-PROMPTING].

## 2. Requirements Language

### 2.1. Key Words

Best Current Practice (BCP) 14 defines the key words in this section.

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "NOT RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in BCP 14 [RFC2119] [RFC8174] when, and only when, they appear in all capitals, as shown here.

### 2.2. Terminology

This guide uses the terms of Section 2.2 of the writing guide, such as author and reviewer, and adds these:

- Claudelish: Prose whose recurring habits of vocabulary, sentence construction, structure, formatting, or tone mark it as unedited output of an LLM.
- tell: One such habit, such as a marker word or a list item that opens with a bold label.
- cluster: Two or more tells in one paragraph. A cluster is a stronger sign than a single tell [WIKI-AI].
- actor: The grammatical subject that performs the action of the verb of a sentence.
- phrase list: The fixed phrases and patterns of Section 6, which mark tells that a text search finds.
- writing guide: The document `docs/writing-documentation.md`.

## 3. Catalogue of Tells

Each subsection states its rules and then gives a table of tells. Every example is an illustration written about this project, and its rewrite shows the plain form. The crates' own documentation and design documents, and not a rewrite here, are the record of their behaviour.

No configured tool checks a tell of this section, except that rustdoc checks links between items in API documentation, as the last row of Section 3.6 shows. The column "Checked by" therefore says "Judgement" for the other tells. Where the phrase list of Section 6 covers a tell in whole or in part, the column names the list, and a reviewer searches the text for its entries.

### 3.1. Vocabulary

CLD-1. An author MUST NOT use the marker words, promotional adjectives, or substitutes for "is" and "has" that the phrase list of Section 6.2 holds. They carry no fact, and a reader who meets them reads the sentence around them as filler. The author writes the concrete thing with a plain verb such as "is", "has", "runs", "builds", or "returns".

CLD-2. An author MUST write out in plain words the relation that agent shorthand compresses, such as `gated on`, `load-bearing`, or `byte-identical`, and name both sides of it, because a reader who does not share the shorthand has to guess the relation. Orwell's rule applies: never use a jargon word "if you can think of an everyday English equivalent" [ORWELL].

| Tell | Example | Rewrite | Checked by |
| --- | --- | --- | --- |
| Marker words | `This section delves into the intricate landscape of hyperbolic tilings.` | This section describes how `escher-domain` builds the fundamental domain of a hyperbolic reflection triangle. | Judgement, with the phrase list |
| Promotional adjectives | `escher offers a powerful, seamless, and blazing fast way to render tilings.` | `escher render` writes the tiling of a signature to a PNG file, and the bytes of the file do not depend on the number of threads. | Judgement, with the phrase list for the listed words |
| A substitute for "is" or "has" | `escher-signature serves as the single source of truth for the canonical form.` | `escher-signature` parses a signature and prints its canonical form. | Judgement, with the phrase list |
| Claudish shorthand | `The merge is gated on CI.` | The agent runs `just merge` only after the CI run of the last commit of the branch concluded `success` with every job succeeded. | Judgement, with the phrase list |
| Claudish shorthand | `This check is load-bearing.` | Without `check-workflow-pins`, nothing reports an action that a workflow does not pin to a full commit SHA. | Judgement, with the phrase list |
| Claudish shorthand | `The two PNG files are byte-identical.` | `cmp` reports no difference between the two PNG files. | Judgement, with the phrase list |
| Filler | `It is worth noting that escher simply writes a PNG file.` | `escher render` writes a PNG file. | Judgement, with the phrase list for the listed phrases |

### 3.2. Sentence Constructions

CLD-3. An author MUST make the actor of each sentence a person, a program, or a component that performs the action, such as the user, the command `escher render`, Cargo, or the crate `escher-domain`. An abstract thing that lies, promises, or refuses, such as a check that "will not lie to you", hides which component acts and what it does [LESSWRONG-ACTORS].

CLD-4. An author MUST NOT frame a claim as the denial of a claim that nobody made, as in `it is not X, it is Y` or `not only X but also Y`, because the denied claim plants an idea the reader did not hold, and the reader then has to work out which of the two the document asserts. A contrast is allowed where the reader is likely to hold the wrong view, such as the belief that `escher render` draws every signature that `escher info` accepts.

CLD-5. An author MUST NOT append a participle phrase that comments on a sentence, such as `, highlighting its focus on safety`, because such a phrase asserts significance with no fact behind it, and a reader cannot check it [WIKI-AI]. The author states a consequence or a reason as a clause of its own, joined with "so" or "because".

CLD-6. An author MUST list exactly the items that exist, because a list padded to three items for rhythm adds a claim that nothing backs, and a list cut to three drops a real item.

CLD-7. An author MUST state what is untested or unknown plainly, and what is known with its source or the condition under which it holds. A hedge such as `based on available information` or `could potentially` leaves the reader unable to tell a tested fact from a guess.

| Tell | Example | Rewrite | Checked by |
| --- | --- | --- | --- |
| Abstract thing as actor | `The verification will not lie to you about a broken domain.` | `escher info --check` prints one line for each check of the verification and gives exit status 1 when a check fails. | Judgement |
| Abstract thing as actor | `A diagnostic is honest about a signature without a domain.` | For a signature without a fundamental domain, `escher render` prints a diagnostic, gives exit status 1, and leaves the output file as it was. | Judgement |
| Denial pivot | `escher render is not a vector renderer, it's a raster renderer.` | `escher render` computes the colour of each pixel from samples. Escher has no vector output. | Judgement, with the phrase list for two forms |
| Trailing "-ing" analysis | `escher render writes the file only after rendering succeeds, highlighting its focus on safety.` | `escher render` writes the file only after rendering and encoding succeed, so a failed render leaves an existing file as it was. | Judgement, with the phrase list for the listed participles |
| Reflexive rule of three | `The renderer is fast, accurate, and elegant.` | `docs/design/render.md` states the accuracy of `escher-render`, measured against high-precision reference arithmetic. | Judgement |
| Hedge | `Based on available information, higher corner orders could potentially work.` | `escher-domain` refuses a corner order above its limit with the error kind `LimitExceeded`. | Judgement, with the phrase list for the listed hedges |

### 3.3. Paragraph and Document Structure

CLD-8. An author MUST open each document, section, and paragraph with its content and MUST end it when the content ends. An opening that announces what follows adds reading time and no fact, and a closing sentence that restates the paragraph is a second statement that can diverge from the first (Section 4.5 of the writing guide). Microsoft's style guide gives the same advice, which is to get to the point fast [MS-STYLE].

CLD-9. An author MUST NOT add a closing section of challenges or outlook, and MUST record limits and open questions where the document type puts them, such as the open questions of a design document. A generic closing section holds no requirement and no decision, so an agent cannot act on it [WIKI-AI].

| Tell | Example | Rewrite | Checked by |
| --- | --- | --- | --- |
| Signposting | `Let's dive into how the fold works.` | `escher-render` folds each sample into the fundamental domain before it chooses the colour of the sample. | Judgement, with the phrase list |
| Restating summary | A paragraph about the thread pool of `escher render` that ends `In short, the thread pool keeps renders fast.` | Delete the last sentence. | Judgement, with the phrase list for an opening summary phrase |
| Formulaic closing section | `## Challenges and Future Outlook`, or `Despite these challenges, the tool is ready.` | An open question in the design document, with what decides it. | Judgement, with the phrase list |
| Overlong introduction | A README that opens with three paragraphs on why symmetry matters | The README opens with what the project is, as RDM-6 of `docs/writing-readmes.md` requires. | Judgement |

### 3.4. Formatting

CLD-10. An author MUST NOT open an unordered list item with a label in bold. The item is a complete sentence or a `term: definition` entry, as Section 2.2 of the writing guide shows. A bold label repeats the first words of the item, and a reader who skims the labels skips the sentences that hold the facts.

CLD-11. An author SHOULD use bold only for a warning that the reader must not miss, and SHOULD add a heading only over content that a reader would cite on its own. Bold on many phrases hides the one that matters, and a heading over each short paragraph splits one point into fragments that lack context.

| Tell | Example | Rewrite | Checked by |
| --- | --- | --- | --- |
| Bold-label bullet | `- **Fast:** renders on a thread pool of its own.` | `escher render` renders on a thread pool of its own. | Judgement, with the phrase list for unordered lists |
| Too much bold | `The **hyperbolic** tiling renders in the **Poincare disk**.` | A hyperbolic tiling renders in the Poincare disk. | Judgement |
| A heading on everything | A section with a level-three heading over each one-sentence paragraph | One section with its paragraphs in order | Judgement |
| Leftovers of a chat interface | `contentReference[oaicite:1]`, or a link that ends in `utm_source=chatgpt.com` | Delete the marker, and cite the source by its plain address. | Judgement, with the phrase list |

### 3.5. Tone

CLD-12. An author MUST address the reader of the document in the neutral register of a specification and MUST NOT write to the person who asked for the text. Praise, agreement, offers of more help, and apologies are leftovers of the chat in which the text was written, and they tell the reader that nobody edited it [WIKI-AI] [ANTHROPIC-OPUS4]. Sycophancy is a known behaviour of models trained with human feedback [ANTHROPIC-SYCOPHANCY], and the press has reported it in Claude Code [REGISTER-SYCOPHANCY].

CLD-13. An author MUST state the significance of a change or an item as what a user or an agent can do with it, because an inflated claim of importance gives the reader nothing to check and hides the fact that matters.

| Tell | Example | Rewrite | Checked by |
| --- | --- | --- | --- |
| Sycophantic opener | `Great question! The tool draws the valid colouring with 2 colours by default.` | `escher render` draws the valid colouring with 2 colours when the domain has one, and one colour otherwise. | Judgement, with the phrase list |
| Chat leftover | `I hope this helps! Let me know if you would like more examples.` | Delete the sentences. | Judgement, with the phrase list |
| Apology | `Apologies for the confusion in the earlier section.` | Correct the earlier section. | Judgement, with the phrase list |
| Inflated significance | `This change is a pivotal milestone for symmetry-based art.` | After this change, `escher render` draws each of the 17 Euclidean signatures. | Judgement, with the phrase list for the marker word |

### 3.6. Content and Truthfulness

CLD-14. An author MUST give a checkable source for each claim. A claim about the world outside the repository cites a named source. A claim about the repository names the file, the recipe, the measurement, or the section that records it. Vague attribution such as `experts agree` and an unverifiable superlative give the reader nothing to check, and an agent that repeats them spreads an unsupported claim.

CLD-15. An author MUST check every identifier, command, path, option, version, number, and citation in a draft against the code or the source before the draft is done, and MUST delete what the author cannot check. A plausible name that does not exist becomes a defect when an agent copies it (Section 6.3 of the writing guide), and invented details in generated bug reports cost maintainers time [STENBERG-AI].

| Tell | Example | Rewrite | Checked by |
| --- | --- | --- | --- |
| Vague attribution | `It is widely known that coset enumeration can run forever.` | `enumerate` of `escher-group` stops at the limit of its `CosetLimit` and returns an error that claims nothing about the index, as `docs/design/group.md` records. | Judgement, with the phrase list for the listed forms |
| Unverifiable superlative | `escher is the most complete orbifold renderer.` | `escher render` draws a signature only when `escher-domain` builds its fundamental domain, and `docs/design/domain.md` lists those signatures. | Judgement |
| Invented option | `Set ESCHER_THREADS=8 to change the number of threads.` | `escher` reads no environment variable of its own. The option `--threads` of `escher render` sets the number of threads. | Judgement |
| Self-certification | `Every entry in this list is verified and accurate.` | Each entry names the signature, the command, and the observed output. | Judgement |
| Invented citation or link | A reference to an RFC number or a page that does not exist | A reference whose address the author opened | Judgement, and `rustdoc::broken_intra_doc_links` for links between items in API documentation |

## 4. Tells That the Writing Guide Already Rules

The writing guide already has a rule for each tell in this table, so this guide adds none and cites the section instead.

| Tell | Rule in the writing guide | Checked by |
| --- | --- | --- |
| Em dashes and en dashes | Section 8 | `check-prose` |
| Curly quotes and emoji | Section 8 | `check-prose` |
| Semicolons in prose | Section 8 | `check-prose` |
| Long paragraphs | Section 4.1 | Judgement |
| A list for an argument, or a run of short bullets | Section 4.1 | Judgement |
| Status and diary lines in a design document | Section 3.6 | Judgement |
| Project history in code and API documentation | Section 7.1 | Judgement |
| Metaphor and idiom | Section 6.4 | Judgement |
| A second name for one concept | Section 4.4 | Judgement |
| The same requirement stated twice | Section 4.5 | Judgement |
| "see above" and "as mentioned" | Section 6.1 | Judgement |

RDM-31 of `docs/writing-readmes.md` keeps progress records out of every README.

Wikipedia lists headings in title case as a tell [WIKI-AI]. Section 8 of the writing guide requires title case, so in this repository title case is the rule, and a reviewer does not report it.

## 5. Strength of the Evidence

This section is non-normative. It records how strong the evidence is for each group of tells, so that a reviewer weighs a finding to match.

| Topic | Evidence |
| --- | --- |
| Marker words | Strong. In a study of PubMed abstracts, the excess words of 2024 were mostly style words, with frequency ratios of 28.0 for `delves` and 13.8 for `underscores` [KOBAK-EXCESS]. In peer reviews, `commendable`, `meticulous`, and `intricate` were 9.8, 34.7, and 11.2 times more likely [LIANG-MONITORING]. |
| Single words as proof of origin | Weak. `delve` is ordinary in Nigerian formal English [WILLISON-NIGERIA], and Wikipedia's editors treat only a dense cluster as a strong sign [WIKI-AI]. This guide bans the words because they carry no content, and a reviewer never cites one as evidence of who wrote a text. |
| Em dashes | Weak alone [WIKI-AI]. Section 8 of the writing guide bans them on its own grounds. |
| Differences between models | Each model has a recognizable style, and a classifier told five model families apart with 97.1% accuracy [SUN-IDIOSYNCRASIES]. A list built from one model's habits therefore misses some habits of another. |
| Claudish shorthand and abstract actors | Thin. One forum post and one university article describe them [LESSWRONG-ACTORS] [WATERLOO-CLAUDISH]. |
| README tells | Thin. One article covers generated READMEs [HACKERNOON-README], and no source ties a wall of badges or a `Key Features` list to models beyond Wikipedia's general tells. |
| Lists in Claude's output | Changing. The system prompt of Claude Opus 4 kept lists out of reports and explanations unless asked [ANTHROPIC-OPUS4], and that of Claude Sonnet 5.5 allows them when the content is multifaceted enough [ANTHROPIC-SONNET55]. |

## 6. The Phrase List

This section lists the fixed phrases and patterns of the tells that a text search finds. No configured tool checks them yet, so they are judgement rules, as Section 3.5 of the writing guide and Hard Rule 4 of `AGENTS.md` state, and a reviewer searches each changed file for them, as step 2 of Section 7 requires. For a rule that Section 3 marks with the phrase list, the list covers only the phrases of Section 6.2 and the patterns of Section 6.3, and the reviewer judges the rest of the rule without a list.

### 6.1. Scope

The phrase list applies to the text that `check-prose` reads under Section 8 of the writing guide, which is every Markdown file and the `///` and `//!` comments of every Rust file, outside fenced code blocks and code spans. An entry matches as follows:

- It matches in any case.
- A fixed phrase matches only as whole words. A hyphen joins the words of a phrase, so `byte-identical` is one phrase, and `gated on` does not match inside `investigated on`.
- A phrase that a line break splits still matches.
- The list does not apply to an entry of a References section, because the title of a cited work keeps its own words, such as `Maintaining Robust Protocols` in the writing guide.

A document that names the phrases, such as this guide, puts each of them in a code span or a fenced code block.

### 6.2. Fixed Phrases

This table holds the fixed phrases by tell, in the order of the subsections of Section 3, with the form to write instead. Each apostrophe is the ASCII apostrophe, because the writing guide bans smart quotes.

| Group | Section | Phrases | Write instead |
| --- | --- | --- | --- |
| Marker words | 3.1 | `delve`, `delves`, `delved`, `delving`, `tapestry`, `testament`, `pivotal`, `crucial`, `crucially`, `showcase`, `showcases`, `showcased`, `showcasing`, `meticulous`, `meticulously`, `commendable`, `intricate`, `intricacies`, `realm`, `landscape` | The concrete thing |
| Promotional adjectives | 3.1 | `seamless`, `seamlessly`, `robust`, `robustly`, `robustness`, `powerful`, `groundbreaking`, `vibrant`, `breathtaking`, `nestled`, `blazing fast`, `blazingly fast` | The measurable property |
| Substitutes for "is" and "has" | 3.1 | `serves as`, `stands as`, `boasts`, `boast` | The plain verb |
| Claudish shorthand | 3.1 | `load-bearing`, `byte-identical`, `gated on`, `gated behind` | The relation in plain words, with both of its sides |
| Filler | 3.1 | `it's worth noting`, `it is worth noting`, `worth mentioning`, `it's important to note`, `it is important to note`, `needless to say`, `simply` | The fact, with the filler deleted |
| Hedges | 3.2 | `based on available information`, `while specific details are limited`, `as of my last update`, `as of my knowledge cutoff`, `could potentially`, `may potentially`, `might potentially` | What is known with its source, or a plain statement of what is untested or unknown |
| Signposting | 3.3 | `let's`, `let us`, `dive into`, `deep dive`, `here's` | The content itself |
| Formulaic closings | 3.3 | `despite these challenges`, `future outlook` | Limits and open questions where the document type puts them |
| Chat leftovers and flattery | 3.5 | `great question`, `you're absolutely right`, `you are absolutely right`, `i hope this helps`, `let me know`, `would you like`, `feel free to`, `happy to help`, `apologies for` | Text addressed to the reader of the document |
| Vague attribution | 3.6 | `experts agree`, `experts argue`, `experts say`, `experts believe`, `industry reports`, `observers have cited`, `it is widely known`, `it is widely believed`, `studies show` | A source that the reader can check |
| Leftovers of a chat interface | 3.4 | `oaicite`, `contentreference`, `utm_source=chatgpt.com` | Nothing, with the source cited by its plain address |

An author adds a phrase to the row of its group.

### 6.3. Patterns

These six patterns complete the fixed phrases. The first five apply to the text of a paragraph, with its line breaks read as spaces, and the sixth applies to each line:

| Pattern | Matches | Tell |
| --- | --- | --- |
| First | A comma followed by `highlighting`, `underscoring`, `emphasizing`, `fostering`, `reinforcing`, `solidifying`, or `cementing` | Trailing "-ing" analysis after a comma (CLD-5) |
| Second | `in conclusion`, `in summary`, `in short`, `to summarize`, `to sum up`, or `overall`, followed by a comma, at the start of a paragraph or a sentence | A summary phrase that opens a sentence (CLD-8) |
| Third | `not only` followed by `but also` with no full stop, question mark, or exclamation mark between them | The pivot `not only ... but also` (CLD-4) |
| Fourth | `it is not` or `it's not` followed, with no full stop, question mark, or exclamation mark between them, by a comma and `it is` or `it's` | The pivot `it is not ..., it is` (CLD-4) |
| Fifth | `certainly`, `of course`, `absolutely`, or `sure`, followed directly by an exclamation mark | A chat interjection with an exclamation mark (CLD-12) |
| Sixth | A line that opens an unordered list item with `-`, `*`, or `+` followed by text in bold | An unordered list item that opens with a bold label (CLD-10) |

The first pattern leaves out `showcasing`, because the fixed phrases hold that word.

### 6.4. False Positives

This table lists where a false positive is likely and how an author resolves it, and it names the words that stay off the list on purpose.

| Phrase or pattern | Likely false positive | Resolution |
| --- | --- | --- |
| `robust`, `robustness` | The title of a cited work, and the technical term for Postel's principle of tolerant input | Reference entries are outside the list. In other prose, name the behaviour, such as "accepts malformed input". |
| `serves as` | The sense "is used as", as in the double of a reflection domain that is used as a rotation domain | Write "is used as" or "is". |
| `landscape` | The orientation of an image | Name the size, such as "wider than it is tall". |
| `crucial`, `pivotal`, `testament`, `realm` | A quotation of a source | Paraphrase the quotation. |
| Fourth pattern | A sentence that corrects a view the reader is likely to hold | State the fact in two sentences, as the rewrite of the pivot in Section 3.2 does. |
| Sixth pattern | None expected | The pattern leaves ordered lists out, because the numbered Hard Rules of `AGENTS.md` open with their names in bold. |
| `underscore`, `underscores` | Off the list. The words name the character `_`, which the design documents discuss in identifiers. | The verb stays a judgement rule without a list. |
| `just` | Off the list. It names the task runner, although Google's style guide lists the adverb as a filler word to delete [GOOGLE-WORDS]. | The adverb stays a judgement rule without a list. |
| `easy`, `easily`, `rich` | Off the list, because each has ordinary uses. | Judgement. |
| `reflecting` | Off the list of participles, because reflections in mirrors are a subject of this repository. | Judgement. |

## 7. Revision Procedure

An author runs these steps on a draft before handing it to review, and a reviewer runs steps 2 to 10 again on every draft that it reviews. Step 1 is a tool check, and the other steps are judgement.

1. Run `check-prose` on each changed file and fix every finding.
2. Search each changed file for the phrases of Section 6.2 and the patterns of Section 6.3, and rewrite each hit that is not a false positive of Section 6.4.
3. Read the first sentence of the document and of each section, and delete each one that announces, praises, or restates the title (CLD-8).
4. Read the last sentence of each paragraph, and delete it when it restates the paragraph (CLD-8).
5. Find the actor of each sentence, and replace each abstract actor with the component that acts (CLD-3).
6. For each adjective of quality or size, write the measurement or the property that it stands for, or delete it (CLD-1, CLD-13).
7. Count the items of each list and of each series in a sentence, and keep only the items that exist (CLD-6).
8. Check every identifier, command, path, option, version, number, and citation, and delete what cannot be checked (CLD-15).
9. Rewrite each list item that opens with a bold label (CLD-10).
10. Where a paragraph still holds a cluster, rewrite it from its facts instead of swapping single words, because a synonym in the same sentence keeps the shape of the tell.

This command runs step 1 on this guide:

```sh
just check-prose docs/avoiding-claudelish.md
```

## 8. Security Considerations

The security consideration of this guide is false content. An author that invents a recipe, an option, a limit, or a security property, such as a claim that a command checks its input against a limit that the code does not enforce, leads a reader or an agent to rely on behaviour that does not exist. CLD-14 and CLD-15 address this.

The threat model is an author, human or agent, that writes plausible text without checking it, and a reader who trusts the text. No attacker is assumed. A malicious change to a document is out of scope, because review and access control govern it. So is deciding whether a text was machine-written, because Section 5 shows that a single tell does not support that decision.

The residual risk is an invented fact in fluent, plain prose. No tool checks the phrase list yet, and nothing checks a claim in a Markdown document against the code, so such a fact passes every tool and depends on the reviewer.

## 9. References

### 9.1. Normative References

- [RFC2119] Bradner, S., "Key words for use in RFCs to Indicate Requirement Levels", BCP 14, RFC 2119, March 1997, <https://www.rfc-editor.org/info/rfc2119>.
- [RFC8174] Leiba, B., "Ambiguity of Uppercase vs Lowercase in RFC 2119 Key Words", BCP 14, RFC 8174, May 2017, <https://www.rfc-editor.org/info/rfc8174>.

### 9.2. Informative References

- [ANTHROPIC-OPUS4] Anthropic, system prompt of Claude Opus 4, release notes, 2025, read 2026-11-17, <https://platform.claude.com/docs/en/release-notes/system-prompts/claude-opus-4>.
- [ANTHROPIC-PROMPTING] Anthropic, "Prompting best practices", read 2026-11-17, <https://platform.claude.com/docs/en/build-with-claude/prompt-engineering/claude-prompting-best-practices>.
- [ANTHROPIC-SONNET55] Anthropic, system prompt of Claude Sonnet 5.5, release notes, 2026, read 2026-11-17, <https://platform.claude.com/docs/en/release-notes/system-prompts/claude-sonnet-5-5>.
- [ANTHROPIC-SYCOPHANCY] Anthropic, "Towards understanding sycophancy in language models", 2023-10-23, <https://www.anthropic.com/research/towards-understanding-sycophancy-in-language-models>.
- [GOOGLE-WORDS] Google, "Google developer documentation style guide: Word list", read 2026-11-17, <https://developers.google.com/style/word-list>.
- [HACKERNOON-README] HackerNoon, article on treating generated READMEs as first drafts, 2026-09-09, <https://hackernoon.com/treat-ai-generated-readmes-like-first-drafts>.
- [KOBAK-EXCESS] Kobak et al., "Delving into ChatGPT usage in academic writing through excess vocabulary", arXiv:2406.07016, 2024-06-11, published in Science Advances on 2025-07-02, <https://arxiv.org/abs/2406.07016>.
- [LESSWRONG-ACTORS] LessWrong, post on why Claude makes abstract things into actors, 2026-08-28, <https://www.lesswrong.com/posts/odDt3qkqnbc4r2fu6/why-does-claude-seem-to-make-abstract-things-into-actors>.
- [LIANG-MONITORING] Liang et al., "Monitoring AI-Modified Content at Scale", arXiv:2403.07183, 2024-03-11, <https://arxiv.org/abs/2403.07183>.
- [MS-STYLE] Microsoft, "Microsoft Writing Style Guide: Brand voice", read 2026-11-17, <https://learn.microsoft.com/en-us/style-guide/brand-voice-above-all-simple-human>.
- [ORWELL] Orwell, G., "Politics and the English Language", 1946, <https://www.orwellfoundation.com/the-orwell-foundation/orwell/essays-and-other-works/politics-and-the-english-language/>.
- [REGISTER-REVIEW] The Register, article on a switch for pull requests written with AI on GitHub, 2026-02-03, <https://www.theregister.com/2026/02/03/github_kill_switch_pull_requests_ai>.
- [REGISTER-SYCOPHANCY] The Register, article on the flattery of Claude Code, 2025-08-13, <https://www.theregister.com/2025/08/13/claude_codes_copious_coddling_confounds/>.
- [STENBERG-AI] Stenberg, D., "The I in LLM stands for intelligence", 2024-01-02, <https://daniel.haxx.se/blog/2024/01/02/the-i-in-llm-stands-for-intelligence/>.
- [SUN-IDIOSYNCRASIES] Sun et al., "Idiosyncrasies in Large Language Models", ICML 2025, arXiv:2502.12150, <https://arxiv.org/abs/2502.12150>.
- [WATERLOO-CLAUDISH] University of Waterloo, news interview on Claudish, 2026-09-08, <https://uwaterloo.ca/news/media/q-and-experts-what-claudish>.
- [WIKI-AI] Wikipedia, "Wikipedia:Signs of AI writing", read 2026-11-17, <https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing>.
- [WILLISON-NIGERIA] Willison, S., post on the word "delve", 2024-04-18, <https://simonwillison.net/2024/Apr/18/delve/>.
- [WILLISON-SLOP] Willison, S., post on the term "slop", 2024-05-08, <https://simonwillison.net/2024/May/8/slop/>.

## Appendix A. Requirement Summary

| Label | Requirement | Section | Checked by |
| --- | --- | --- | --- |
| CLD-1 | Use no marker word, promotional adjective, or substitute for "is" or "has" that Section 6.2 lists | 3.1 | Judgement, with the phrase list |
| CLD-2 | Write out the relation that agent shorthand compresses | 3.1 | Judgement, with the phrase list for the listed shorthand |
| CLD-3 | Make a person, program, or component the actor of each sentence | 3.2 | Judgement |
| CLD-4 | Do not deny a claim that nobody made | 3.2 | Judgement, with the phrase list for two forms |
| CLD-5 | Append no participle phrase of commentary to a sentence | 3.2 | Judgement, with the phrase list for the listed participles |
| CLD-6 | List exactly the items that exist | 3.2 | Judgement |
| CLD-7 | State what is untested or unknown plainly, and what is known with its source | 3.2 | Judgement, with the phrase list for the listed hedges |
| CLD-8 | Open with the content and end when the content ends | 3.3 | Judgement, with the phrase list for signposts and opening summary phrases |
| CLD-9 | Add no closing section of challenges or outlook, and put limits and open questions where the document type puts them | 3.3 | Judgement, with the phrase list for the listed phrases |
| CLD-10 | Open no unordered list item with a bold label | 3.4 | Judgement, with the phrase list |
| CLD-11 | Use bold and headings sparingly | 3.4 | Judgement |
| CLD-12 | Address the reader of the document, with no flattery, chat leftovers, or apology | 3.5 | Judgement, with the phrase list for the listed phrases |
| CLD-13 | State significance as what a user or an agent can do | 3.5 | Judgement |
| CLD-14 | Give a checkable source for each claim | 3.6 | Judgement, with the phrase list for the listed attributions |
| CLD-15 | Check every identifier, command, path, option, version, number, and citation, and delete what cannot be checked | 3.6 | Judgement |
