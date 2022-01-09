# kommandozeilen_argumente

Parser für Kommandozeilen-Argumente mit optionaler, potentiell deutscher, automatischer Hilfe-Generierung.

## Flags

Aktiviert über `--flag` oder `-f`
Können (explizit) über `--kein-flag`, bzw. `--no-flag` deaktiviert werden
Präfix kann konfiguriert werden.

Aktivieren mehrerer Flags auf einmal möglich `-fgh`

## Werte

Langformen: `--arg <Wert>` oder `--arg=<Wert>`
Kurzformen: `-a <Wert>` oder `-a<Wert>`

## derive-Attribute

trait ArgEnum ohne Attribute

trait Parse: alles über kommandozeilen_argumente-Attribut

Global an struct

- `deutsch` | `englisch` | `english`: Standard-Einstellung für einige Strings, standard `deutsch`
- `version`: erzeuge eine `--version` Flag
- `hilfe` | `help`: erzeuge eine Hilfe-Text
- `hilfe(<opts>)`, `help(<opts>)`, `version(<opts>)`:
    Wie die Variante ohne opts, nur Kurzname ist standardmäßig deaktiviert. Mögliche Opts:
  - `lang: <name>`, `long [<namen>]`: Setze Langnamen explizit
  - `kurz`: Setze Kurznamen als erstes Grapheme des originalen Langnamen
  - `kurz: <name>`, `kurz: [<namen>]`: Setze Kurznamen explizit
  - `deutsch` | `englisch` | `english`: Sprache von Hilfe-Text und Standard-Namen
- `meta_var: <string>` | `meta_var: <string>`:
    Setze Standardwert für in der Hilfe angezeigte Meta-Variable, Standard `WERT` oder `VALUE`
- `invertiere_präfix: <string>` | `invert_prefix: <string>`:
    Setze Standardwert für Präfix zum invertieren einer Flag, Standard `kein` oder `no`

Vor Feldern

- hilfe aus docstring
- name wird langer name
- erstes grapheme wird kurzer name
- Es wird eine Implementierung über das ParseArgument-Trait verwendet
  - Flags (bool-Werte) sind standardmäßig deaktiviert
  - Option-Werte sind standardmäßig None
  - Strings und Zahlentypen (u8, i8, f32, ...) haben keinen Standardwert (notwendiges Argument)
  - ArgEnum-Werte haben keinen Standardwert (notwendiges Argument), benötigt Display.
        Derive-Macro für Summen-Typen ohne Daten wird bereitgestellt.
- `glätten`/`flatten`: verwende das Parse-Trait (übernehmen der konfigurierten Argumente)
- `FromStr`: verwende das FromStr-Trait (benötigt Display für Wert und Fehler-Typ)
- `benötigt`/`required`: entferne den konfigurierten Standard-Wert
- `lang: <name>`| `long: <name>`: bestimme Langname explizit
- `lang: [<namen>]` | `long: [<names>]`: bestimme Langnamen explizit (Komma-getrennte Liste)
- `kurz`/`short`: Verwende eine Kurzform, bestehend aus dem ersten Grapheme der Langform
- `kurz: <wert>"`/`short: <value>"`: Verwende die spezifizierte Kurzform
- `kurz: [<namen>]` | `short: [<names>]`: bestimme Kurzformen explizit (Komma-getrennte Liste)
- `standard: <wert>` | `default: <value>`: setzte den Standard-Wert
- `meta_var: <string>`: setzte die in der Hilfe angezeigt Meta-Variable
- `invertiere_präfix: <string>` | `invert_prefix: <string>`: setze Präfix zum invertieren einer Flag
