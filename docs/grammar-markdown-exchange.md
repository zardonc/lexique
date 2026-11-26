# Single grammar note Markdown exchange

A saved grammar record can be exported as one Markdown file and imported through
the grammar notebook. The file contains content only; categories, review
progress, history and pause state remain local. Imported new cards start paused.

The save dialog suggests a Windows-compatible filename from the saved title and
exchange identity, using the same naming rule as Markdown ZIP exports. Independent
records with the same title receive distinct filenames.

The template starts with `# Lexique grammar note v1` and uses named sections
`exchange_id`, `title`, `body`, `question` and `answer`. Each present section
contains a plain text fence, such as:

````markdown
# Lexique grammar note v1

## exchange_id

```text
example-identity
```

## title

```text
Le subjonctif
```

## body

```text
Une note personnelle.
```
````

The title is required. An absent or empty identity creates a new record; an
existing identity matches that record even when its title or filename changes.
An omitted body keeps an existing body, while a fenced empty body clears it.
Omitted or blank question and answer preserve the existing card; two nonblank
sections replace the pair together. A single nonblank side is invalid. Use a
fence longer than every consecutive backtick run in its content; exports choose
the fence automatically. Review the preview and explicitly select updates.

Markdown is an exchange template, not a full backup. Use a full backup to retain
learning state and categories.
