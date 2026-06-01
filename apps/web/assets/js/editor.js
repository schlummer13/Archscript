import { EditorState } from "@codemirror/state";
import { EditorView, keymap, lineNumbers, highlightActiveLine, highlightActiveLineGutter } from "@codemirror/view";
import { defaultKeymap, history, historyKeymap } from "@codemirror/commands";
import { searchKeymap, highlightSelectionMatches } from "@codemirror/search";
import { autocompletion, completionKeymap } from "@codemirror/autocomplete";
import { StreamLanguage, syntaxHighlighting, HighlightStyle } from "@codemirror/language";
import { tags } from "@lezer/highlight";

const archscriptLanguage = StreamLanguage.define({
  token(stream) {
    if (stream.eatSpace()) return null;

    if (stream.match("#")) {
      stream.skipToEnd();
      return "comment";
    }

    if (stream.match(/"(?:[^"\\]|\\.)*"/)) {
      return "string";
    }

    if (stream.match(/->/)) {
      return "operator";
    }

    if (stream.match(/\[[^\]]*\]/)) {
      return "tag";
    }

    if (
      stream.match(
        /\b(project|element|relation|boundary|domain|environment|as|tags)\b/
      )
    ) {
      return "keyword";
    }

    if (
      stream.match(
        /\b(actor|user|team|frontend|backend|service|microservice|database|queue|event_bus|external|external_api|api|gateway|worker|cache|storage|bucket|vector_db|identity_provider|cdn|cloud)\b/
      )
    ) {
      return "type";
    }

    stream.next();
    return null;
  }
});

const archscriptHighlight = HighlightStyle.define([
  {
    tag: tags.keyword,
    color: "var(--text)",
    fontWeight: "700"
  },
  {
    tag: tags.string,
    color: "var(--muted)"
  },
  {
    tag: tags.comment,
    color: "var(--muted)",
    fontStyle: "italic"
  },
  {
    tag: tags.operator,
    color: "var(--text)",
    fontWeight: "700"
  },
  {
    tag: tags.atom,
    color: "var(--muted)"
  }
]);

const archscriptTheme = EditorView.theme({
  "&": {
    height: "680px",
    backgroundColor: "var(--surface)",
    color: "var(--text)",
    fontSize: "13px"
  },
  ".cm-scroller": {
    fontFamily: '"SF Mono", "SFMono-Regular", Menlo, Consolas, monospace',
    lineHeight: "1.7"
  },
  ".cm-content": {
    padding: "18px 0",
    caretColor: "var(--text)"
  },
  ".cm-line": {
    padding: "0 18px"
  },
  ".cm-gutters": {
    backgroundColor: "var(--surface)",
    color: "var(--muted)",
    borderRight: "1px solid var(--border)"
  },
  ".cm-activeLine": {
    backgroundColor: "var(--surface-soft)"
  },
  ".cm-activeLineGutter": {
    backgroundColor: "var(--surface-soft)",
    color: "var(--text)"
  },
  ".cm-selectionBackground, &.cm-focused .cm-selectionBackground": {
    backgroundColor: "var(--border)"
  },
  "&.cm-focused": {
    outline: "none"
  }
});

const suggestions = [
  "project",
  "element",
  "relation",
  "boundary",
  "domain",
  "environment",
  "tags",
  "actor",
  "frontend",
  "backend",
  "service",
  "database",
  "event_bus",
  "gateway",
  "identity_provider",
  "owner",
  "tech",
  "monitoring",
  "description",
  "critical",
  "production"
].map((label) => ({
  label,
  type: "keyword"
}));

function archscriptCompletions(context) {
  const word = context.matchBefore(/\w*/);

  if (!word || (word.from === word.to && !context.explicit)) {
    return null;
  }

  return {
    from: word.from,
    options: suggestions
  };
}

export function createArchScriptEditor({ parent, value, onChange }) {
  const view = new EditorView({
    parent,
    state: EditorState.create({
      doc: value,
      extensions: [
        lineNumbers(),
        highlightActiveLineGutter(),
        history(),
        archscriptLanguage,
        syntaxHighlighting(archscriptHighlight),
        archscriptTheme,
        highlightActiveLine(),
        highlightSelectionMatches(),
        autocompletion({
          override: [archscriptCompletions]
        }),
        keymap.of([
          ...defaultKeymap,
          ...historyKeymap,
          ...searchKeymap,
          ...completionKeymap
        ]),
        EditorView.lineWrapping,
        EditorView.updateListener.of((update) => {
          if (update.docChanged) {
            onChange?.(update.state.doc.toString());
          }
        })
      ]
    })
  });

  return {
    getValue() {
      return view.state.doc.toString();
    },

    setValue(value) {
      view.dispatch({
        changes: {
          from: 0,
          to: view.state.doc.length,
          insert: value
        }
      });
    },

    focus() {
      view.focus();
    },

    destroy() {
      view.destroy();
    }
  };
}