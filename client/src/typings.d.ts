// Copyright (C) 2023 Bryan A. Jones.
//
// This file is part of the CodeChat Editor. The CodeChat Editor is free
// software: you can redistribute it and/or modify it under the terms of the GNU
// General Public License as published by the Free Software Foundation, either
// version 3 of the License, or (at your option) any later version.
//
// The CodeChat Editor is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or
// FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more
// details.
//
// You should have received a copy of the GNU General Public License along with
// the CodeChat Editor. If not, see
// [http://www.gnu.org/licenses](http://www.gnu.org/licenses).
//
// # `typing.d.ts` -- Global type definitions
//
// How a doc block is stored using CodeMirror.
type DocBlockJSON = [
    // From
    number,
    // To
    number,
    // Indent
    string,
    // Delimiter
    string,
    // Contents
    string
];

// These modules keep TypeScript from complaining about missing type definitions
// for Turndown and Turndown plugin imports. See
// [CodeChatEditor.mts](CodeChatEditor.mts).
declare module "@joplin/turndown-plugin-gfm";
declare module "prettier/esm/standalone.mjs";
declare module "prettier/esm/parser-markdown.mjs";
declare module "prettier/esm/parser-html.mjs";
