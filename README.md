# 1\. CodeChat Editor overview

The CodeChat Editor is a GUI-based programmer's word processor / Jupyter for
software developers. This document records its overall design.

These headings are manually numbered; they will be automatically numbered.

## <a id="how-to-run"></a>1.1 How to run

The [CodeChat Editor repository](https://github.com/bjones1/CodeChat_Editor)
contains the code for this application. To try it out:

1.  Clone or download the repository.
2.  [Install the Rust language](https://www.rust-lang.org/tools/install). I
    recommend the 64-bit toolset for Windows.
3.  [Install NPM](https://docs.npmjs.com/downloading-and-installing-node-js-and-npm)
    (the Node.js package manager).
4.  Install all NPM dependencies: in the `client/` directory, run `npm update`.
5.  Package all JavaScript dependencies from NPM: also in the `client/`
    directory, run `npm run build`.
6.  In the `server/` directory, execute `cargo run`.
7.  Open `http://localhost:8080` in your browser.
8.  Open the file `README.md`.

## <a id="vision"></a>1.2 Vision

These form a set of high-level requirements to guide the project.

- View source code as
  <a id="vision-code-blocks-and-doc-blocks"></a>[code blocks and doc blocks](index.md#code-blocks-and-doc-blocks).
  Doc blocks are lines of source which contain only correctly-formatted
  comments.
- Provide support for a
  <a id="vision-programming-language-support"></a>[wide variety of programming languages](index.md#programming-language-support).
- Provide integration with a
  <a id="vision-ide-integration"></a>[wide variety of IDEs/text editors](index.md#ide-integration).
- Load a document from source code, allow edits in a GUI, then save it back to
  source code.
  - Provide word processor GUI tools (insert hyperlink, images, headings,
    change font, etc) for doc blocks.
  - Provide text editor/IDE tools (syntax highlighting, line numbers, show
    linter feedback) for code blocks.
- Zero build: eliminate the traditional project build process -- make it almost
  instantaneous.
- Doc block markup should be readable and well-known.
- Support both a single-file mode and a project mode.
  - A project is a specific directory tree, identified by the presence of a
    TOC. I like
    [mdbook's appearance](https://rust-lang.github.io/mdBook/format/summary.html),
  - A page in a project build is a single-file page plus:
    - A TOC, along with previous/next/up navigation. The TOC is synchronized to
      the current page.
    - Numbering comes from the current page's location within the TOC. Pages
      not in the TOC aren't numbered.
- <a id="authoring-support"></a>Provide
  [authoring support](index.md#authoring-support), which allows authors to
  easily create book/project-like features. In particular:
  - Counters for numbering figures, tables, equations, etc.
  - References (to a section, figure, table, etc.) via auto-titled links: the
    link text is automatically derived from the link's destination (the heading
    text at the link's destination; a figure/table caption, etc.).
  - Bidirectional links, which can be used for footnotes/endnotes/citations and
    indices. The source of a bidirectional link must specify an anchor name, a
    target (file and anchor in the file), the text to display at the target
    (typically, the section which contains the source link), and where to
    locate this text relative to the target anchor.
  - A page-local TOC, in order to produce the overall TOC.
  - Autogenerated anchor names and autogenerated anchors for all headings.
  - Hyperlinks to identifiers in code (use
    [ctags](https://github.com/universal-ctags/ctags)). Perhaps autogenerated
    titles as well.
  - Substitutions.
  - Files/anchors can be freely moved without breaking links.
- Provide a GUI to make picking a file/anchor easy.
- Provide edit, view, and view raw source options.

### Nice to have features

- Simple to install locally; support some sort of web-based IDE. Ideas: allow
  editing a GitHub repo directly.
- Support a static build: producing a set of view-only HTML files which don't
  need a server for a project, or a single HTML file outside a project.

## <a id="specification"></a>1.3 Requirements

The requirements expand on the vision by providing additional details.

### <a id="specification-code-blocks-and-doc-blocks"></a>Code blocks and doc blocks

Comments in most programming languages are either inline comments (which are
terminated by a newline) or block comments, which may span multiple lines. In
C/C++, the opening delimiter for an inline comment is `//`. Likewise, `/*` and
`*/` define the opening and closing delimiters for block comments.

This design treats source code on a line-by-line basis. It does not classify at
any deeper granularity -- for example, it does not support a mix of code block
and doc block on the same line.

A code block consists of all lines in a source file which aren't classified as
a doc block. Note that code blocks may consist entirely of a comment, as
illustrated below.

A doc block consists of a comment (inline or block) optionally preceded by
whitespace and optionally succeeded by whitespace. At least one whitespace
character must separate the opening comment delimiter from the doc block text.
Some examples in C:

<pre>void foo(); // This is not a doc block, because these comments are preceded<br>void bar(); // by non-whitespace characters. Instead, they're a code block.<br>//This is not a doc block, because these inline comments lack<br>//whitespace after the opening comment delimiter //. They're also a code block.<br>/*This is not a doc block, because this block comment lacks<br>  whitespace after the opening comment delimiter /*. It's also a code block. */<br>/* This is not a doc block, because non-whitespace <br>   characters follow the closing comment delimiter. <br>   It's also a code block. */ void food();<br><br>// This is a doc block. It has no whitespace preceding the inline<br>// comment delimiters and one character of whitespace following it.<br>  // This is also a doc block. It has two characters of whitespace <br>  // preceding the comment delimiters and one character of whitespace following it.<br>/* This is a doc block. Because it's based on<br>   a block comment, a single comment can span multiple lines. */<br>/* This is also a doc block, even without the visual alignment<br>or a whitespace before the closing comment delimiter.*/<br>  /* This is a doc block<br>     as well. */</pre>

Doc blocks are differentiated by their indent: the whitespace characters
preceding the opening comment delimiter. Adjacent doc blocks with identical
indents are combined into a single, larger doc block.

<pre>// This is all one doc block, since only the preceding<br>//   whitespace (there is none) matters, not the amount of <br>// whitespace following the opening comment delimiters.<br>  // This is the beginning of a different doc<br>  // block, since the indent is different.<br>    // Here's a third doc block; inline and block comments<br>    /* combine as long as the whitespace preceeding the comment<br>delimiters is identical. Whitespace inside the comment doesn't affect<br>       the classification. */<br>// These are two separate doc blocks,<br>void foo();<br>// since they are separated by a code block.</pre>

### <a id="implementation-programming-language-support"></a>[Programming language support](index.md#programming-language-support)

Initial targets come from the Stack Overflow Developer Survey 2022's section on
[programming, scripting, and markup languages](https://survey.stackoverflow.co/2022/#section-most-popular-technologies-programming-scripting-and-markup-languages)
and IEEE Spectrum's
[Top Programming Languages 2022](https://spectrum.ieee.org/top-programming-languages-2022).

### <a id="specification-ide-integration"></a>IDE/text editor integration

Initial targets come from the Stack Overflow Developer Survey 2022's section on
[integrated development environments](https://survey.stackoverflow.co/2022/#section-most-popular-technologies-integrated-development-environment).

### Zero-build support

The "build" should occur immediately (to any open files) or when when saving a
file (to closed files, which will be updated when they're next opened).
Exception: edits to the TOC are applied only after a save.

### Authoring support

This system should support custom tags to simplify the authoring process. The
GUI must indicate that text enclosed by the tags isn't directly editable,
instead providing an option to edit the underlying tag that produced the text.
When a new tag is inserted, any tag-produced content should be immediately
added.

#### Counters

Counters are not global; they're local to a page. Instead, they take the prefix
from the TOC value assigned to the preceding heading tag.

#### References (auto-titled links)

The link text is automatically derived from the link's destination (the heading
text at the link's destination, for example). The text must not be editable in
the GUI, since it will be overwritten by updates to the linked text.

#### Bidirectional links

The source of a bidirectional link must specify:

- An anchor name
- A target (the globally-unique anchor)
- The text to display at the target (typically, the section which contains the
  source link).
- Where to locate this text relative to the target anchor.

To create a footnote/endnote/citation, pick a place in the document (end of
section / bottom of page / bibliography) then type the text in. Next, create a
bidirectional link to the place to insert the footnote/endnote/citation. The
text of the link should come from the footnote/endnote/bibliography number just
created.

#### Autogenerated anchor names and autogenerated anchors for all headings

The GUI should suggest an autogenerated name for anchors. This is a string of
random characters; HTML allows upper/lowercase ASCII plus the hyphen and
underscore for IDs, meaning that a 5-character string provides >250 million
unique anchors.

To make headings easy to link to, all headings should receive autogenerated
anchors. On hover, the headings should show a link symbol. Scrolling in the
document to another heading should cause the TOC highlight to update.

#### Page-local TOC

A special tag (probably only for use in the TOC). Inputs: file to reference.
Output: the hyperlinked title (`<h1>` tag's text) of the page, with nested
lists for subheadings (with links to each subheading). Subpages of a page show
up after headings on that page. The author may specify what depth of headings
appear in the TOC on a global and per-page basis.

#### Movable files/anchors

To make moving files/anchors (or anything an anchor is attached to -- headings,
figures, etc.) easy, require that all anchor be unique across the entire
project, instead of unique just in the current file. Then, internal links can
be reduced to the anchor; during hydration, the path to the file containing
that anchor) is fetched from the cache. Place an anchor at the top of every
file to uniquely identify it, and change links to a file to instead refer to
this anchor.

How to handle the case where an anchor isn't found in the cache? We'd then need
to update the cache for the entire project, which might be expensive but also
rare. Perhaps perform this update in the background.

### Hyperlink support

This system relies on the user to create a lot of meaningful links; it must
make the creation and maintenance of links simple:

- Make it easy to create a link to another file/anchor via GUI support.
  - Provide a file browser-like dialog to select a file, then an anchor inside
    that file. For files, show the title as well as the file name; for anchors,
    show the surrounding text. Provide an isearch to make searching easier.
  - Allow the user to copy a link to any anchor, then paste that into the GUI
    as another way of creating a link.
- Clicking on an anchor should both browse to the page and also highlight the
  anchor's associated text.
- On hover, hyperlinks should show a preview of the linked web page.

## License

Copyright (C) 2022 Bryan A. Jones.

This file is part of the CodeChat Editor.

The CodeChat Editor is free software: you can redistribute it and/or modify it
under the terms of the GNU General Public License as published by the Free
Software Foundation, either version 3 of the License, or (at your option) any
later version.

The CodeChat Editor is distributed in the hope that it will be useful, but
WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or
FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more
details.

You should have received a [copy](LICENSE.html) of the GNU General Public
License along with the CodeChat Editor. If not, see
[https://www.gnu.org/licenses/](https://www.gnu.org/licenses/).
