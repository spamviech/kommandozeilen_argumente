# kommandozeilen_argumente

Parser für Kommandozeilen-Argumente mit optionaler, automatischer Hilfe-Generierung.

Zum erstellen eines neuen Arguments werden assoziierte Funktionen bereitgestellt,
der Ergebnistyp wird dabei durch eine Typ-Variable festgelegt.
Argumente können mithilfe des `kombiniere!`-Macros, bzw. dedizierten `kombiniereN`-Funktionen,
zu komplexeren Strukturen zusammengefasst werden,
die zum parsen potentiell mehrere Argumente benötigen.

Ein Argument wird durch seinen Langnamen oder potentiellen Kurznamen identifiziert.
Angabe eines Langnamens startet mit zwei Minus `--lang`.
Angabe eines Kurznamens startet mit einem Minus `-k`.
Für Kurznamen wird angenommen, dass sie nur ein Grapheme lang sind.

Alle verwendeten Strings, z.B. für die erzeugte Hilfe-Meldung, sind konfigurierbar.
Sofern es relevant ist werden für Deutsch und Englisch spezialisierte Funktionen bereitgestellt.

## Flags

Flags sind Argumente ohne Wert, sie können entweder aktiviert oder deaktiviert sein.
Meistens repräsentieren sie `bool`-Argumente, andere Typen werden aber ebenfalls unterstützt.

Aktiviert über `--flag` oder `-f`
Können (explizit) über `--kein-flag`, bzw. `--no-flag` deaktiviert werden
Präfix kann konfiguriert werden.
Standardmäßig deaktiviert, kann angepasst werden.

Aktivieren mehrerer Flags auf einmal möglich `-fgh`

### Frühes Beenden

TODO

## Werte

Langformen: `--arg <Wert>` oder `--arg=<Wert>`
Kurzformen: `-a <Wert>`, `-a=<Wert>` oder `-a<Wert>`

Konfigurierbarer Standardwert, potentiell Anzeige möglicher Werte im Hilfe-Text.

## derive-Attribute

trait ArgEnum ohne Attribute, derive-Macro wird für enums ohne Daten bereitgestellt.

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

## Geplante Features

- Unterargumente (subcommands)
- Positions-basierte Argumente
- Unterschiedlicher Standard-Wert für Name ohne Wert und Name kommt nicht vor
- Argumente unter Berücksichtigung von Groß- und Kleinschreibung
- Argument-Gruppen (nur eine dieser N Flags kann gleichzeitig aktiv sein)
